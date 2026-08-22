/* Whitefoot parallel runtime.
 *
 * The actualization half of the [PAR-1 candidate] permission judgment. The
 * compiler proves, before anything here runs, that the two statements of an
 * overlapped pair touch disjoint storage, carry no external or blocking row,
 * and reach no claim site; this file only decides where the work runs. It
 * never chooses what may overlap.
 *
 * Contract with the emitted module, in the order the module uses it:
 *
 *   void *wf__par_claim(unsigned long bytes)
 *       Takes one of this thread's own frame slots and returns it — `bytes` of
 *       storage the caller fills with the handed-out call's arguments and later
 *       reads its result out of — or returns NULL when this thread holds no
 *       free slot or no slot's frame is that large. It never blocks and never
 *       runs anything.
 *
 *   void wf__par_publish(void *frame, void (*fn)(void *))
 *       Offers `fn(frame)` to the pool by pushing the frame onto this thread's
 *       own deque. It hands the work to nobody: an idle thread takes it, or the
 *       joining thread takes it back. Only a frame a claim returned reaches it.
 *
 *   void wf__par_join(void *frame)
 *       Returns once that task has run, after which the frame holds its result.
 *       It runs the task itself when no other thread took it, which is the
 *       common case, so the offer costs the caller nothing it would not have
 *       spent running the call in source order.
 *
 *   void wf__par_release(void *frame)
 *       Gives the slot back, once the caller has read the result out. The
 *       frame is the caller's until here, which is why the join does not
 *       release: a slot handed on at the join could be refilled under the read
 *       that follows it.
 *
 * Beside the protocol, and not part of it, one question the module asks once:
 *
 *   int wf__par_pool_active(void)
 *       Whether this run was asked for a pool at all. The module carries two
 *       lowerings of everything that can reach a hand-out — the overlapped one
 *       and the plain sequential one — and its bootstrap asks this to choose
 *       between them, once, before anything runs. It moves no work, takes no
 *       frame, and starts nothing: the pool is still created lazily by the
 *       first claim. See its definition at the bottom of this file for why the
 *       answer may be kept for the life of the process, why it reads the
 *       setting rather than the pool, and why that makes it a different thing
 *       from a demand signal.
 *
 * The claim comes first because it is what makes the hand-out cost
 * conditional. The frame lives in per-thread storage rather than in the calling
 * function, so an activation that is never granted a slot never builds one: no
 * stack slot, no argument spills, and the recursion depth of a `--par` build
 * stays the recursion depth of the sequential build.
 *
 * ## Why a deque and not a hand-off
 *
 * The earlier shape searched the lane array for an idle thread and pushed the
 * work at it: a claim was an O(lanes) atomic scan over lines other threads were
 * using, and a grant was a mutex round trip plus a condition-variable wake. The
 * cost of that fell on *every* offer, granted or refused, so a program that
 * offered more than the pool could absorb paid the whole search for nothing —
 * the measured shape was a per-offer cost that rose with lane count.
 *
 * Here an offer is a push onto the offering thread's own deque: two stores to
 * its own cache line, no read-modify-write, no other thread's memory, and no
 * syscall. The expensive operations are steals, and a steal happens only when a
 * thread runs out of work, so their number is proportional to how unbalanced
 * the program turned out to be rather than to how many times it offered. Idle
 * threads take from the *oldest* end, which on a recursive descent is the
 * shallowest fork point and so the largest remaining subtree; the owner takes
 * from the newest end, which is the one it was about to run anyway.
 *
 * The pool is process-lifetime trusted computing base, like malloc's
 * internals: no Whitefoot construct names it, no Whitefoot value reaches it,
 * and it writes nothing to any output. WF_WORKERS selects the thread count.
 * Unset — the shape a `--par` binary is actually handed to somebody in — asks
 * for this machine's logical CPUs, so a program that was built for overlap
 * overlaps. An explicit `0`, `1`, or anything that does not parse leaves the
 * pool unstarted, so every claim returns NULL and every program runs exactly
 * the sequential schedule it runs today — and, since the module can ask,
 * exactly the sequential code as well.
 */

#include <pthread.h>
#include <sched.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <sys/sysctl.h>
#endif

/* An upper bound on threads of execution, so a hostile WF_WORKERS cannot ask
 * for unbounded threads. It is a resource ceiling, not a language constant. */
#define WF_PAR_MAX_LANES 64

/* How large a handed-out call's frame a slot can hold: its arguments followed
 * by its result. A call whose frame is larger is simply never granted a slot
 * and runs on the calling thread, which is a schedule the program already has
 * to be correct under. Every argument type the backend emits is a scalar, a
 * pointer, or a small aggregate of those, so this is far above what a real
 * call asks for; it exists so the bound is stated rather than assumed. */
#define WF_PAR_FRAME_BYTES 256

/* How many offers one thread can have outstanding at once. Like the frame
 * size above, this is a stated bound rather than a tuned policy: the storage
 * has to be finite, and refusing past it is already correct, because the
 * refused edge is the same call the granted edge makes. It is not a grain
 * knob and nothing here consults it to decide whether an offer is worth
 * making. What it does mean, and is worth noticing rather than designing
 * around: pushes happen at the newest end, so a thread that has run out of
 * slots refuses its *deepest* fork points and keeps the shallow ones it
 * already offered, which are the large ones. */
#define WF_PAR_LANE_SLOTS 64

/* The machine's coherence granule, used to keep one thread's hot words off
 * another's. Two lanes that shared a line would trade it on every push. */
#define WF_PAR_CACHE_LINE 128

/* How long a thread looks for work before it stops burning a core. The first
 * phase spins, the second yields the processor, and only then does a thread
 * reach a condition variable. Waiting is converted into work wherever there is
 * work to convert it into, so these bound the case where there is none.
 *
 * The floor under these is measured rather than chosen: a park and its wake
 * cost about 2.2 microseconds on this machine, so a thread that looks for less
 * time than that goes to sleep to save less than the sleep costs, and pays the
 * wake anyway the moment work appears. A round of looking is a handful of
 * loads, so the spin phase below is a few multiples of that round trip — long
 * enough that a thread does not sleep through work that is about to arrive,
 * short enough that an idle pool is not a busy one. */
#define WF_PAR_SPIN_ROUNDS 4096
#define WF_PAR_YIELD_ROUNDS 16

/* A slot's execution state. FREE is the resting state of an unclaimed slot;
 * PENDING says the frame is filled and pushed; DONE says the task has run and
 * the result is in the frame. */
#define WF_PAR_SLOT_FREE 0
#define WF_PAR_SLOT_PENDING 1
#define WF_PAR_SLOT_DONE 2

struct wf__par_lane;

/* One offer's storage. */
struct wf__par_slot {
    /* The handed-out call's frame, and the first member: a pointer to the
     * frame is a pointer to the slot, so the emitted module holds one opaque
     * pointer for the whole protocol and never learns this layout. The
     * alignment covers every type the backend puts in a frame, whose widest
     * member is eight bytes. */
    _Alignas(16) unsigned char frame[WF_PAR_FRAME_BYTES];
    /* The outlined call over the frame. Written before the push that publishes
     * it, so a thief that sees the push sees this too. */
    void (*run)(void *);
    /* FREE / PENDING / DONE, atomic. Only a task some *other* thread took ever
     * has to report DONE, because a thread that runs its own offer at the join
     * is the only thread that could observe it. */
    int state;
    /* The lane sleeping on this slot, or NULL. Written by the slot's owner,
     * read by whichever thread finishes the task. */
    struct wf__par_lane *waiter;
    /* The lane whose storage this is, so a release finds its free list from
     * the frame pointer alone. */
    struct wf__par_lane *home;
    /* Free-list link, valid while the slot is FREE. */
    int next_free;
};

/* One thread of execution: its deque of offers, its slot storage, and the
 * station it sleeps at. Every field a *thief* touches is on a different line
 * from every field the *owner* touches on its hot path, because the whole
 * point of pushing locally is that the push stays local. */
struct wf__par_lane {
    /* The oldest end. Thieves advance it; the owner reads it and, only when
     * taking the last entry, races them for it. */
    _Alignas(WF_PAR_CACHE_LINE) unsigned long long top;

    /* The newest end, and the owner's own line: the push that publishes an
     * offer is a store to `buffer` and a store to this word, and touches
     * nothing else. */
    _Alignas(WF_PAR_CACHE_LINE) unsigned long long bottom;
    struct wf__par_slot *buffer[WF_PAR_LANE_SLOTS];
    /* Head of this lane's free-slot list. Claim and release are the only
     * writers and both run on the owning thread, so it needs no atomic. */
    int free_head;
    /* Victim selection state, so idle threads do not all descend on the same
     * lane in the same order. */
    unsigned long long seed;

    /* Where this lane sleeps. At most one thread ever waits here — the lane's
     * own — so a completion signals rather than broadcasts. */
    _Alignas(WF_PAR_CACHE_LINE) pthread_mutex_t lock;
    pthread_cond_t signal;
    /* A wake delivered to this lane, guarded by `lock`. It is what keeps a
     * wake that arrives just before the sleep from being lost. */
    int posted;

    _Alignas(WF_PAR_CACHE_LINE) struct wf__par_slot slots[WF_PAR_LANE_SLOTS];
};

static struct wf__par_lane wf__par_lanes[WF_PAR_MAX_LANES];
static int wf__par_lane_count;
static pthread_once_t wf__par_started = PTHREAD_ONCE_INIT;

/* Which lanes are parked waiting for work to appear anywhere. A publish reads
 * this and wakes nobody when it is empty, which in a warm pool is every
 * publish. One bit per lane, so WF_PAR_MAX_LANES is exactly one word. */
_Alignas(WF_PAR_CACHE_LINE) static unsigned long long wf__par_idle;

/* Lanes granted since process start: tasks that ran on a thread other than the
 * one that offered them.
 *
 * No Whitefoot construct can name it and no program reads it; it exists so
 * that a measurement, or a gate, can tell a pool that grants lanes from one
 * that silently never does. Without it an overlap test passes just as well
 * against a runtime that refuses everything, which is the one way this could
 * be broken without any test noticing.
 *
 * It counts *steals* and nothing else. A task the offering thread ran itself
 * at its own join did not overlap with anything, so counting it here would be
 * counting offers — which is exactly the false green the counter exists to
 * catch. Because a steal happens only when a thread runs out of work, this is
 * off the hot path by construction rather than by an accumulator: the number
 * of increments is the number of times the pool actually rebalanced. */
_Alignas(WF_PAR_CACHE_LINE) unsigned long wf__par_grants;

/* This thread's lane, or NULL when it has none. */
static _Thread_local struct wf__par_lane *wf__par_self;
/* Whether this thread has already tried to take a lane, so a program running
 * with no pool pays one thread-local load and a branch per offer and never
 * reaches the pool's initialization again. */
static _Thread_local int wf__par_attached;

/* Set once the pool's threads are all in their steal loops, so the first offer
 * of a program meets a pool that can take it rather than one still starting. */
static int wf__par_ready;
static pthread_mutex_t wf__par_ready_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf__par_ready_signal = PTHREAD_COND_INITIALIZER;

/* A worker stack is at least as large as the calling thread's own stack, and
 * never below this floor: a forked task is an ordinary Whitefoot call that may
 * recurse exactly as deep as it would have on the calling thread, so the much
 * smaller pthread default (512 KB on some platforms) would turn a recursion
 * that succeeds sequentially into an overflow on a lane. */
#define WF_PAR_STACK_FLOOR ((size_t)8u * 1024u * 1024u)

/* The calling thread's stack size, or the floor when the platform does not
 * report one. RLIMIT_STACK is the main thread's own limit on every platform
 * this compiler targets; RLIM_INFINITY and absurd values fall back to the
 * floor rather than asking for an unbounded worker stack. */
static size_t wf__par_stack_bytes(void) {
    struct rlimit limit;
    if (getrlimit(RLIMIT_STACK, &limit) == 0 && limit.rlim_cur != RLIM_INFINITY
        && limit.rlim_cur > (rlim_t)WF_PAR_STACK_FLOOR
        && limit.rlim_cur <= (rlim_t)((size_t)512u * 1024u * 1024u)) {
        return (size_t)limit.rlim_cur;
    }
    return WF_PAR_STACK_FLOOR;
}

/* ---------------------------------------------------------------- sleeping */

/* Wakes one lane. Only the lane's own thread ever waits on its condition
 * variable, so there is exactly one thread to wake and `signal` is the whole
 * of it. `posted` closes the window between a thread deciding to sleep and
 * actually sleeping. */
static void wf__par_signal(struct wf__par_lane *lane) {
    pthread_mutex_lock(&lane->lock);
    lane->posted = 1;
    pthread_cond_signal(&lane->signal);
    pthread_mutex_unlock(&lane->lock);
}

/* Wakes one lane that is parked waiting for work to appear, if any is.
 *
 * Off the hot path: a publish only reaches here when the idle word is
 * non-empty, which in a saturated pool is never. */
static void wf__par_wake_one(void) {
    unsigned long long parked = __atomic_load_n(&wf__par_idle, __ATOMIC_RELAXED);
    while (parked != 0) {
        int index = __builtin_ctzll(parked);
        unsigned long long bit = 1ull << index;
        /* Claim the wake by clearing the bit, so two publishes do not both
         * spend a syscall on the same sleeper. */
        if ((__atomic_fetch_and(&wf__par_idle, ~bit, __ATOMIC_ACQ_REL) & bit) != 0) {
            wf__par_signal(&wf__par_lanes[index]);
            return;
        }
        parked &= ~bit;
    }
}

/* ------------------------------------------------------------------- deque */

/* Pushes one filled frame onto the owner's own deque.
 *
 * The release store of `bottom` is the edge that publishes the caller's
 * argument stores and the buffer write to whichever thread takes the entry;
 * it is sequentially consistent rather than merely releasing so that the idle
 * word read that follows cannot be observed before it, which is what keeps a
 * thread from parking beside work that was pushed a moment earlier. */
static void wf__par_push(struct wf__par_lane *lane, struct wf__par_slot *slot) {
    unsigned long long bottom = lane->bottom;
    lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)] = slot;
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_SEQ_CST);
}

/* Takes the newest entry, the one the owner was about to run anyway.
 *
 * Chase-Lev's owner side: the decrement and the read of `top` must not be
 * reordered against each other, and only the last entry can be contested, so
 * the atomic exchange is paid once per emptying rather than once per pop. */
static struct wf__par_slot *wf__par_pop(struct wf__par_lane *lane) {
    unsigned long long bottom = lane->bottom;
    unsigned long long top = __atomic_load_n(&lane->top, __ATOMIC_RELAXED);
    struct wf__par_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
    bottom -= 1;
    __atomic_store_n(&lane->bottom, bottom, __ATOMIC_SEQ_CST);
    top = __atomic_load_n(&lane->top, __ATOMIC_SEQ_CST);
    if ((long long)(bottom - top) < 0) {
        /* A thief took the last one out from under us. */
        __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
        return NULL;
    }
    slot = lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)];
    if ((long long)(bottom - top) > 0) {
        return slot;
    }
    /* The last entry: a thief may be reading it right now, so the two sides
     * settle it on `top`. */
    if (!__atomic_compare_exchange_n(&lane->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
        slot = NULL;
    }
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
    return slot;
}

/* Takes the oldest entry of another thread's deque — on a recursive descent
 * the shallowest fork point that thread has offered, and so the largest
 * subtree still to run.
 *
 * The emptiness test is a pair of ordinary loads: a thief must not take the
 * victim's line away from it just to discover there is nothing there. */
static struct wf__par_slot *wf__par_steal(struct wf__par_lane *victim) {
    unsigned long long top = __atomic_load_n(&victim->top, __ATOMIC_ACQUIRE);
    unsigned long long bottom = __atomic_load_n(&victim->bottom, __ATOMIC_ACQUIRE);
    struct wf__par_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
    slot = victim->buffer[top & (WF_PAR_LANE_SLOTS - 1)];
    if (!__atomic_compare_exchange_n(&victim->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
        return NULL;
    }
    __atomic_add_fetch(&wf__par_grants, 1, __ATOMIC_RELAXED);
    return slot;
}

/* One round of looking for work anywhere but here, starting somewhere
 * different on each thread and each round. */
static struct wf__par_slot *wf__par_find(struct wf__par_lane *lane) {
    int count = wf__par_lane_count;
    int offset;
    int step;
    if (count < 2) {
        return NULL;
    }
    lane->seed = lane->seed * 6364136223846793005ull + 1442695040888963407ull;
    offset = (int)((lane->seed >> 33) % (unsigned long long)count);
    for (step = 0; step < count; step += 1) {
        int index = offset + step;
        struct wf__par_lane *victim;
        struct wf__par_slot *slot;
        if (index >= count) {
            index -= count;
        }
        victim = &wf__par_lanes[index];
        if (victim == lane) {
            continue;
        }
        slot = wf__par_steal(victim);
        if (slot != NULL) {
            return slot;
        }
    }
    return NULL;
}

/* Runs a task somebody may be waiting for, and tells them when it is done.
 *
 * The store of DONE is sequentially consistent so that the read of `waiter`
 * cannot be observed before it: between them and the owner's own store of
 * `waiter` before its read of the state, one of the two threads is guaranteed
 * to see the other, which is what makes the sleep safe without a lock on the
 * common path. */
static void wf__par_execute(struct wf__par_slot *slot) {
    struct wf__par_lane *waiter;
    slot->run(slot->frame);
    __atomic_store_n(&slot->state, WF_PAR_SLOT_DONE, __ATOMIC_SEQ_CST);
    waiter = __atomic_load_n(&slot->waiter, __ATOMIC_SEQ_CST);
    if (waiter != NULL) {
        wf__par_signal(waiter);
    }
}

/* ------------------------------------------------------------------ waiting */

/* Waits for a task another thread took, doing whatever work is available
 * instead of sleeping for as long as there is any. */
static void wf__par_wait(struct wf__par_lane *lane, struct wf__par_slot *target) {
    int rounds = 0;
    for (;;) {
        struct wf__par_slot *slot;
        if (__atomic_load_n(&target->state, __ATOMIC_ACQUIRE) == WF_PAR_SLOT_DONE) {
            return;
        }
        /* Our own offers first: they are the ones this thread was going to run
         * anyway, and taking them back costs no coherence traffic. */
        slot = wf__par_pop(lane);
        if (slot == NULL) {
            slot = wf__par_find(lane);
        }
        if (slot != NULL) {
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS) {
            rounds += 1;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
            rounds += 1;
            sched_yield();
            continue;
        }
        /* There is nothing to do but wait for the thread that took it. Say so
         * before looking at the state one last time: the finishing thread
         * stores DONE before it reads this, so whichever of the two stores
         * lands second, the other thread sees it. */
        pthread_mutex_lock(&lane->lock);
        __atomic_store_n(&target->waiter, lane, __ATOMIC_SEQ_CST);
        while (__atomic_load_n(&target->state, __ATOMIC_SEQ_CST) != WF_PAR_SLOT_DONE) {
            pthread_cond_wait(&lane->signal, &lane->lock);
        }
        __atomic_store_n(&target->waiter, NULL, __ATOMIC_RELAXED);
        lane->posted = 0;
        pthread_mutex_unlock(&lane->lock);
        return;
    }
}

/* ------------------------------------------------------------------- lanes */

static void *wf__par_worker_main(void *opaque) {
    struct wf__par_lane *lane = (struct wf__par_lane *)opaque;
    int rounds = 0;
    wf__par_self = lane;
    wf__par_attached = 1;

    pthread_mutex_lock(&wf__par_ready_lock);
    wf__par_ready += 1;
    pthread_cond_signal(&wf__par_ready_signal);
    pthread_mutex_unlock(&wf__par_ready_lock);

    for (;;) {
        struct wf__par_slot *slot = wf__par_find(lane);
        if (slot != NULL) {
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS) {
            rounds += 1;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
            rounds += 1;
            sched_yield();
            continue;
        }
        /* Park. The bit goes up *before* the last look, so a publisher that
         * misses the bit is a publisher whose push this look will find. */
        __atomic_fetch_or(&wf__par_idle, 1ull << (lane - wf__par_lanes), __ATOMIC_SEQ_CST);
        slot = wf__par_find(lane);
        if (slot != NULL) {
            __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)),
                               __ATOMIC_ACQ_REL);
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        pthread_mutex_lock(&lane->lock);
        if (!lane->posted) {
            /* One wait, not a loop: any wake at all sends this thread back to
             * looking, which is the only predicate it has. */
            pthread_cond_wait(&lane->signal, &lane->lock);
        }
        lane->posted = 0;
        pthread_mutex_unlock(&lane->lock);
        __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)), __ATOMIC_ACQ_REL);
        rounds = 0;
    }
    return NULL;
}

static void wf__par_prepare(struct wf__par_lane *lane, int index) {
    int slot;
    lane->top = 0;
    lane->bottom = 0;
    lane->seed = 0x9e3779b97f4a7c15ull * (unsigned long long)(index + 1);
    lane->posted = 0;
    for (slot = 0; slot < WF_PAR_LANE_SLOTS; slot += 1) {
        lane->slots[slot].home = lane;
        lane->slots[slot].state = WF_PAR_SLOT_FREE;
        lane->slots[slot].waiter = NULL;
        lane->slots[slot].next_free = slot + 1;
    }
    lane->slots[WF_PAR_LANE_SLOTS - 1].next_free = -1;
    lane->free_head = 0;
}

/* What an unset WF_WORKERS asks for: this machine's logical CPUs, clamped to
 * the lane ceiling above.
 *
 * `hw.logicalcpu` is the Darwin spelling and reports the cores this process may
 * be scheduled on right now, which is the number the pool wants; the portable
 * `_SC_NPROCESSORS_ONLN` answers on a host that has no such name. A machine
 * that reports fewer than two gets 0, which is what an explicit opt-out gets,
 * because one thread of execution is the sequential world either way.
 *
 * Naming every core rather than only the performance ones is a measured
 * choice, not an assumption: on the 4P+6E machine this was built on, the coarse
 * oracle configurations improve monotonically from 4 lanes to 10 (bal_d12_w192
 * 4.71x at 8 lanes to 5.16x at 10), and the fine ones do not fall below their
 * own sequential build there. An efficiency core that runs a stolen subtree at
 * a fraction of a performance core's rate still finishes it sooner than not
 * starting it. */
static int wf__par_default_lanes(void) {
    long online;
#if defined(__APPLE__)
    int logical = 0;
    size_t width = sizeof(logical);
    if (sysctlbyname("hw.logicalcpu", &logical, &width, NULL, 0) == 0 && logical >= 2) {
        return (logical > WF_PAR_MAX_LANES) ? WF_PAR_MAX_LANES : logical;
    }
#endif
    online = sysconf(_SC_NPROCESSORS_ONLN);
    if (online < 2) {
        return 0;
    }
    return (online > WF_PAR_MAX_LANES) ? WF_PAR_MAX_LANES : (int)online;
}

static int wf__par_requested_lanes(void) {
    const char *setting = getenv("WF_WORKERS");
    char *end = NULL;
    long requested;
    if (setting == NULL || setting[0] == '\0') {
        return wf__par_default_lanes();
    }
    requested = strtol(setting, &end, 10);
    if (end == setting || *end != '\0' || requested < 2) {
        return 0;
    }
    if (requested > WF_PAR_MAX_LANES) {
        requested = WF_PAR_MAX_LANES;
    }
    return (int)requested;
}

static void wf__par_start(void) {
    pthread_attr_t attributes;
    int requested = wf__par_requested_lanes();
    int started = 0;
    int index;
    if (requested < 2) {
        return;
    }
    if (pthread_attr_init(&attributes) != 0) {
        return;
    }
    /* A refused stack request is a silent downgrade to the pthread default,
     * which is exactly the hazard this call exists to close, so it stops the
     * pool instead: no lane is better than a lane that overflows. */
    if (pthread_attr_setstacksize(&attributes, wf__par_stack_bytes()) != 0) {
        pthread_attr_destroy(&attributes);
        return;
    }
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    for (index = 0; index < requested; index += 1) {
        wf__par_prepare(&wf__par_lanes[index], index);
    }
    /* Lane 0 belongs to the thread that got here — the calling thread is
     * itself a lane, so `WF_WORKERS=2` means two threads of execution in
     * total. The count is published before the threads start so a worker's
     * first look sees the whole pool. */
    wf__par_lane_count = requested;
    for (index = 1; index < requested; index += 1) {
        pthread_t thread;
        struct wf__par_lane *lane = &wf__par_lanes[index];
        if (pthread_mutex_init(&lane->lock, NULL) != 0) {
            break;
        }
        if (pthread_cond_init(&lane->signal, NULL) != 0) {
            pthread_mutex_destroy(&lane->lock);
            break;
        }
        if (pthread_create(&thread, &attributes, wf__par_worker_main, lane) != 0) {
            pthread_cond_destroy(&lane->signal);
            pthread_mutex_destroy(&lane->lock);
            break;
        }
        started = index;
    }
    wf__par_lane_count = started + 1;
    pthread_attr_destroy(&attributes);
    if (started == 0) {
        wf__par_lane_count = 0;
        return;
    }
    if (pthread_mutex_init(&wf__par_lanes[0].lock, NULL) != 0) {
        wf__par_lane_count = 0;
        return;
    }
    if (pthread_cond_init(&wf__par_lanes[0].signal, NULL) != 0) {
        pthread_mutex_destroy(&wf__par_lanes[0].lock);
        wf__par_lane_count = 0;
        return;
    }
    /* Wait for every thread to reach its steal loop. A pool whose threads are
     * still being created cannot take the first offers a program makes, and
     * for a short program those are all the offers there are; this makes
     * "the pool started" mean "the pool can take work" and costs one
     * rendezvous per process. */
    pthread_mutex_lock(&wf__par_ready_lock);
    while (wf__par_ready < started) {
        pthread_cond_wait(&wf__par_ready_signal, &wf__par_ready_lock);
    }
    pthread_mutex_unlock(&wf__par_ready_lock);
}

/* Takes lane 0 for the thread that first offers work. Any further thread that
 * offers gets no lane and every claim it makes is refused, which is a schedule
 * the program is already correct under. */
static struct wf__par_lane *wf__par_attach(void) {
    static int taken;
    int expected = 0;
    wf__par_attached = 1;
    pthread_once(&wf__par_started, wf__par_start);
    if (wf__par_lane_count == 0) {
        return NULL;
    }
    if (!__atomic_compare_exchange_n(&taken, &expected, 1, 0, __ATOMIC_ACQ_REL,
                                     __ATOMIC_RELAXED)) {
        return NULL;
    }
    wf__par_self = &wf__par_lanes[0];
    return wf__par_self;
}

/* --------------------------------------------------------- the module's ABI */

void *wf__par_claim(unsigned long bytes) {
    struct wf__par_lane *lane = wf__par_self;
    struct wf__par_slot *slot;
    int index;
    if (bytes > (unsigned long)WF_PAR_FRAME_BYTES) {
        return NULL;
    }
    if (lane == NULL) {
        if (wf__par_attached) {
            /* No pool, or no lane for this thread: one thread-local load and a
             * branch, and never the pool's initialization again. */
            return NULL;
        }
        lane = wf__par_attach();
        if (lane == NULL) {
            return NULL;
        }
    }
    index = lane->free_head;
    if (index < 0) {
        return NULL;
    }
    slot = &lane->slots[index];
    lane->free_head = slot->next_free;
    return slot->frame;
}

void wf__par_publish(void *frame, void (*fn)(void *)) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    slot->run = fn;
    __atomic_store_n(&slot->state, WF_PAR_SLOT_PENDING, __ATOMIC_RELAXED);
    wf__par_push(slot->home, slot);
    if (__atomic_load_n(&wf__par_idle, __ATOMIC_SEQ_CST) != 0) {
        wf__par_wake_one();
    }
}

void wf__par_join(void *frame) {
    struct wf__par_slot *target = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = target->home;
    struct wf__par_slot *slot = wf__par_pop(lane);

    if (slot == target) {
        /* Nobody took it. This is the common case and the whole reason an
         * offer is cheap: the task runs here, now, as an ordinary call, and
         * the offer cost the caller two stores and a pop. Nothing observes
         * this slot's state — the thread that would read it is this one — so
         * there is no completion to record, which leaves the call in tail
         * position and the join's own frame out of the recursion. */
        target->run(target->frame);
        return;
    }
    while (slot != NULL) {
        /* Some other offer of ours came off first, which happens when a group
         * hands out more than one member. Running it is progress: the group's
         * members are disjoint by the judgment that permitted them, so the
         * order they run in is not observable. */
        wf__par_execute(slot);
        if (__atomic_load_n(&target->state, __ATOMIC_ACQUIRE) == WF_PAR_SLOT_DONE) {
            return;
        }
        slot = wf__par_pop(lane);
        if (slot == target) {
            target->run(target->frame);
            return;
        }
    }
    wf__par_wait(lane, target);
}

void wf__par_release(void *frame) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = slot->home;
    __atomic_store_n(&slot->state, WF_PAR_SLOT_FREE, __ATOMIC_RELAXED);
    slot->next_free = lane->free_head;
    lane->free_head = (int)(slot - lane->slots);
}

/* Whether this run was asked for a pool, answered once at the bootstrap.
 *
 * Not part of the lane protocol: it takes no frame, publishes nothing, moves
 * no work, and — deliberately — starts nothing. It answers the one question
 * the module cannot answer for itself, so that a program built with `--par`
 * and run without a pool takes the sequential lowering of itself rather than
 * the refused edge of the overlapped one.
 *
 * **It reads the setting rather than the pool, and that is the point.** The
 * pool is still created lazily, by the first claim, exactly where it was
 * created before this question existed, so a program that asks this is
 * scheduled exactly as it was. The alternative — answering by *starting* the
 * pool here — was built and measured, and it costs real time: it moves the
 * pool's creation ahead of whatever the program does before its first
 * hand-out, so those threads spin alongside that work instead of being
 * created after it. On `tests/programs/par_layout.wf`, which builds its tree
 * before it folds, starting the pool here read 0.4663 s against 0.3981 s at
 * `WF_WORKERS=4` and 0.4404 s against 0.3724 s at `W=8` — 17% and 18% — while
 * this version reads 0.3984 s and 0.3752 s, which is parity. (The first
 * attempt to price it used `WF_WORKERS=64`, where a single binary's own
 * spread across an interleaved pass is 47% to 308% and nothing is resolvable;
 * that reading was withdrawn.) Answering without starting anything also means
 * a `--par` run that never reaches a hand-out creates no threads at all.
 *
 * The answer is fixed for the life of the process: the environment is the
 * process's own, the core count it falls back to when the setting is absent is
 * the machine's, `wf__par_start` reads both through the same function, and
 * neither a pool that started nor one that did not ever changes. That is what
 * makes it safe to keep forever, which is what the
 * caller does — it asks at the entry and never again — and what distinguishes
 * it from a demand signal: nothing here measures what the pool is *doing*, so
 * no thread ever writes a word another reads on account of it.
 *
 * The one case where "asked for" and "got" differ is a pool that was
 * requested and failed to start — no thread could be created. That run takes
 * the overlapped world and has every claim refused, which is a correct
 * schedule and exactly what such a run did before this question existed.
 */
int wf__par_pool_active(void) {
    return wf__par_requested_lanes() >= 2;
}
