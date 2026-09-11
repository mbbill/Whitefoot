/* Ordinary-call, current-stack compute scheduling. Derived from main
 * 9051576f; atomic ring cells and SC thief reads repair the two deque races.
 * Slots and lanes live until process exit. Only the offering thread joins and
 * releases its slots; callbacks must not block on I/O or migrate stacks. */
#include "entry.h"
#include "prim.h"
#include <stdlib.h>
#include <stdio.h>

#define WF_PAR_MAX_LANES WF_SCHED_MAX_THREADS
#define WF_PAR_FRAME_BYTES WF_SCHED_FRAME_BYTES
#define WF_PAR_LANE_SLOTS WF_SCHED_LANE_SLOTS
#define WF_PAR_CACHE_LINE 128
/* How long a lane with nothing to run stays hot before it parks. Two numbers
 * answer that: how often the lane may reconsider, and how long it goes on
 * reconsidering.
 *
 * HOW OFTEN is a count of misses, and it is unchanged. A lane should not pay
 * for a sleep in order to save less than the sleep costs, so the count is only
 * meaningful against what a round costs and what a park costs, and both are
 * measured by wake_probe.c on the host the choice is made for. On the four-CPU
 * Linux development host that probe reads a park-and-wake of 16.3 us -- the
 * median of seven runs of 2,000 alternating parks through this file's own
 * wf__par_signal and wf_prim_wait_sleep -- and one wf__par_find round at
 * 18.9 ns uncontended at four lanes, so 1,024 rounds is about 19.4 us. A
 * monotonic clock read costs far more than a round, so the window below is
 * sampled once per 1,024 rounds and this count is now the window's resolution
 * rather than the whole of a lane's patience. Re-measure both numbers before
 * moving it: a change that makes a round cheaper shortens the sampling
 * interval by the same factor and is not a separate variable.
 *
 * HOW LONG is a time, and it depends on whether the pool fits the machine.
 * On the hosted ubuntu-24.04 runners -- two cores, two SMT siblings each, four
 * online CPUs -- a lane that parks on a condvar and is woken by a publisher
 * comes back slow. The per-call evidence is in the raw.tsv of run 34631106340.
 * Fir at W=2, each call against the fastest call of its own pass: the first
 * call reads 1.31 to 2.21, and in three of the five passes calls one through
 * four are still 1.06 to 1.98. Its first pass spends 8.5, 7.6 and 6.5 ms on
 * the first three calls against 3.8 ms on the last, while process CPU rises
 * 2.04 times against the wall's 2.21 -- so the extra wall is two lanes running
 * slowly, not one lane waiting. The `static` reference, whose helpers busy-wait
 * and never sleep, reads 1.21 to 1.24 on its first call and 1.00 to 1.04 on
 * every call after, in every pass. The reading is that the woken lane is
 * placed next to its waker until the load balancer separates them; what is
 * measured is that it costs the first few calls of every process, tens of
 * milliseconds, and that the arm which never sleeps does not pay it.
 *
 * Two twins measured what a longer window is worth. Lanes that never park
 * (spin bound 10^9, run 34631106340) took mandelbrot's W=4 paired wall to
 * 0.922 of the shipped runtime with four of five pairs lower and fir's W=2 to
 * 0.934 with four of five, and collapsed once the pool no longer fit: at eight
 * lanes on four CPUs they read 2.699, 2.945, 3.487 and 6.702. A 16,384-round
 * bound (run 34632558439) kept the saving in CPU where the pool did fit --
 * paired CPU 0.934 at fir W=2, 0.952 at fir W=4, 0.963 and 0.970 at records --
 * and still cost 1.158 to 1.304 in paired wall at W=8 on three of the four
 * kernels. So a long idle window is right when the pool is not oversubscribed
 * and wrong when it is, which is the rule below: lanes at pool start against
 * the CPUs this process may run on
 * (research/investigations/compute-runtime/RESULTS.md, run 34631106340 with
 * the A/B control -DWF_PAR_SPIN_ROUNDS=1000000000, and run 34632558439 with
 * the A/B control -DWF_PAR_SPIN_ROUNDS=16384; the per-call series that the
 * placement evidence is read off are in each run's raw.tsv).
 *
 * 1,000 us is not a swept value and nothing here selects it. It is two orders
 * of magnitude above the park-and-wake this file's own probe measures and an
 * order of magnitude below the tens of milliseconds the hosted per-call series
 * takes to settle, so it is long enough to cover a wake and short enough that
 * a lane no longer needed stops burning a CPU within a millisecond. The guard
 * is what lets the compute scoreboard build its A/B twin at another value
 * through WF_RUNTIME_CONTROL_FLAGS and measure it against the shipped runtime
 * inside one set of passes.
 *
 * The rule was then measured as shipped, against a control built at
 * WF_PAR_IDLE_WINDOW_US=0 on the same SMT runner class (run 34638747514,
 * whose machine is a slow one of that class -- mandelbrot W=4 reads 5,676 us
 * on oneTBB). Read as control over windowed, so above 1.000 is the window
 * ahead: mandelbrot 1.005 / 1.007 / 0.986 at W=2/4/8, fir 1.000 / 1.006 /
 * 0.975 (five of five lower at W=8), quadrature 1.016 / 0.995 / 1.003,
 * records 1.014 / 0.955 / 1.029. Three things in that: the oversubscription
 * guard holds -- W=8 does not collapse the way the never-park twin did -- the
 * window buys half a percent to one and a half on mandelbrot and fir at the
 * fitted widths, and it is NOT free. The paired CPU column reads 0.972 and
 * 0.958 at mandelbrot W=2/W=4 and 0.977 and 0.956 at fir, so the window
 * spends three to four percent more process CPU than the control for that
 * wall, and records W=4 loses 4.5 percent of wall outright with the control
 * lower in four of five pairs.
 *
 * That cost is what the publish epoch below answers, and the reason is stated
 * beside it: a lane spinning out the window called wf__par_find every round,
 * and that reads two lines per other lane -- the lines a working lane writes
 * on every push and pop. Records is memory-bound and crosses 64 chunk
 * boundaries a call, which is why it pays most. An idle lane past the first
 * spin bound now reads one counter instead, and scans only when it moves.
 *
 * Each spin round also issues the hardware hint an SMT sibling needs from its
 * partner: wf_prim_spin_hint() is YieldProcessor() on Windows -- _mm_pause on
 * x86 and x64, a store barrier plus yield on arm -- and, since the same
 * hosted evidence, `pause` on x86 and `yield` on aarch64 for the POSIX hosts
 * too (prim.h); before that a POSIX spin round emitted no pause at all, which
 * is the tightest loop an SMT sibling can be asked to share a core with.
 *
 * The scheduler probes compile this file with WF_SCHED_TEST and hold native
 * threads at the park protocol's own race windows -- wf_sched_test_before_wait
 * fires from inside the sleep loop, and a coordinator thread waits on what it
 * publishes. Those probes assert the protocol, not this policy, and a window
 * would make when a lane arrives at that hook depend on a wall clock rather
 * than on a fixed number of misses. Under WF_SCHED_TEST the window is
 * therefore compile-time zero: the loops then take exactly the fixed bound
 * below, which is the arrival those hooks were written against. The epoch
 * shortcut needs no separate switch for the same reason -- it is gated on the
 * window having opened, so a zero window leaves every round a full scan and
 * the enumerating probes reach wf__par_find, wf__par_steal and their hooks on
 * exactly the rounds they always did.
 *
 * This selects how an already admitted program waits. It is not a timeout, a
 * fuel bound or a proof-work budget; no acceptance path reads it, it cannot
 * reject a program, and with the window at zero the loops behave as they did
 * before it existed. */
#ifndef WF_PAR_SPIN_ROUNDS
#define WF_PAR_SPIN_ROUNDS 1024
#endif
#if defined(WF_SCHED_TEST)
#undef WF_PAR_IDLE_WINDOW_US
#define WF_PAR_IDLE_WINDOW_US 0
#endif
#ifndef WF_PAR_IDLE_WINDOW_US
#define WF_PAR_IDLE_WINDOW_US 1000
#endif
/* Yields are cheap next to a park and the sweep gave no reason to move them. */
#ifndef WF_PAR_YIELD_ROUNDS
#define WF_PAR_YIELD_ROUNDS 16
#endif
/* How many chunks an independent map may be split into, as a multiple of the
 * lane count. `wf__par_split_budget` below combines this with the work term --
 * span / ceil(work unit / weight), the floor on chunk size entry.c owns -- by
 * a MINIMUM, so this is a ceiling on chunk count and never a floor on chunk
 * size: a range too small to be worth splitting stays bounded by the work term
 * however much oversubscription this allows.
 *
 * What is known about 16, which arrived with the splitter: on the compute
 * scoreboard it is the BINDING term for two of the three map kernels at the
 * widths that host records. At weight 812 over
 * 131,072 records and weight 150 over 524,288 outputs the work term affords
 * far more chunks than `16 * lanes`, so records and fir read exactly 32 chunks
 * at two lanes and 64 at four -- the cap, not the grain -- which is why the
 * work unit's own sweep could not move either row at either width
 * (research/investigations/compute-runtime/RESULTS.md, the split work unit
 * swept against the Mandelbrot grain). Mandelbrot is the opposite case: at
 * weight 219 over 98,304 points the work term affords 17, so its 16 chunks are
 * the grain and this cap is slack there; against it oneTBB's auto_partitioner
 * hands the same skewed map out as about 1,536 callbacks.
 *
 * It was swept and it does not move. The compute scoreboard ran three
 * candidate pairs against the shipped runtime through its A/B twin -- the cap
 * and the work unit raised together, so mandelbrot's chunk count could pass its
 * own cap rather than stop at it -- one `compare PASSES=5 CALLS=5` each on the
 * four-CPU Linux development host, the plain image byte-identical in all three.
 * The chunk counts moved exactly as the rule predicts: (64, 300,000) gave
 * mandelbrot 64 chunks at every width with records and fir at 128/256/256,
 * (256, 75,000) gave 256 with 512/1024/1024, and (1024, 20,000) gave 1,024
 * with records 2048/4096/4096 and fir 2,048 at every width. The acceptance was
 * mandelbrot's W=4 `wf-b/wf` wall below 1.000 with at least four of five
 * paired passes lower, no kernel reproducibly worse at W=2 or W=4, W=4 paired
 * CPU no worse than 1.05, and W=1 unchanged. It read 0.999 (3/5), 0.992 (3/5)
 * and 0.970 (5/5), and the third is not selectable, because QUADRATURE IS A
 * NULL ARM IN EVERY ONE OF THOSE RUNS -- it emits no split call at all, so its
 * two images do identical work -- and in that same third run quadrature's own
 * W=4 pair read 0.940 with five of five lower. Over the four runs quadrature's
 * identical-work W=4 pair spans 0.936 to 1.086, including both a 5/5-lower and
 * a 0/5-lower reading, which is the one thing a within-pass twin cannot remove:
 * the two images differ in bytes and therefore in code placement, this bundle's
 * largest confound. (1024, 20,000) also took fir's W=2 pair to 1.053 with one
 * of five lower. So nothing here selects a cap, and 16 stands unmeasured rather
 * than measured-and-kept (research/investigations/compute-runtime/RESULTS.md,
 * the oversubscription cap measured with the A/B twin). Reopen on a host whose
 * null arm holds inside one percent, or on an eight-CPU host where W=8 is the
 * recorded block -- at the oversubscribed W=8 here the middle candidate read
 * fir 0.909 with five of five lower and paired CPU 0.922.
 *
 * The guard is what lets the compute scoreboard build a runtime at another
 * value through its WF_RUNTIME_CONTROL_FLAGS and A/B twin, so the cap can be
 * swept against the shipped runtime inside one set of passes instead of edited
 * per run. This selects how finely an admitted program is actualized in
 * parallel; no acceptance path reads it. */
#ifndef WF_PAR_SPLIT_OVERSUBSCRIBE
#define WF_PAR_SPLIT_OVERSUBSCRIBE 16
#endif
/* Sequential leaves a recursive component is cut into, per lane, by
 * wf__par_recursion_budget. Measured on the compute scoreboard's quadrature
 * kernel (research/investigations/compute-runtime/RESULTS.md, the
 * recursive-frontier depth sweep): at four lanes, 16 leaves per lane -- the
 * budget of 6 that sweep pinned -- read 1.221 against the best per-pass
 * reference, and 64 leaves per lane -- its budget of 8 -- read 1.017, the
 * fastest median in the block. Sixty-four also gives budget 7 at two lanes,
 * which that sweep measured as the best budget at that width, 8 at four and 9
 * at eight lanes. This selects how much of an admitted program is actualized
 * in parallel; no acceptance path reads it. */
#ifndef WF_PAR_RECURSION_LEAVES_PER_LANE
#define WF_PAR_RECURSION_LEAVES_PER_LANE 64
#endif
/* The deepest budget handed out, so a very wide pool cannot ask a component
 * for more private levels than any measured configuration needed. */
#define WF_PAR_RECURSION_MAX_BUDGET 24
#define WF_PAR_SLOT_FREE 0
#define WF_PAR_SLOT_PENDING 1
#define WF_PAR_SLOT_DONE 2
struct wf__par_lane;

struct wf__par_slot {
    /* First member: the emitted ABI carries only this opaque frame address. */
    _Alignas(16) unsigned char frame[WF_PAR_FRAME_BYTES];
    void (*run)(void *);
    int state;
    /* Atomically registered by the owner: either NULL or immutable home. */
    struct wf__par_lane *waiter;
    struct wf__par_lane *home;
    int next_free;
};

struct wf__par_lane {
    /* Thieves advance top; only this lane's owner writes bottom/free_head.
     * Ring cells are atomic because a losing thief may read across reuse. */
    _Alignas(WF_PAR_CACHE_LINE) unsigned long long top;
    _Alignas(WF_PAR_CACHE_LINE) unsigned long long bottom;
    struct wf__par_slot *buffer[WF_PAR_LANE_SLOTS];
    int free_head;
    unsigned long long seed;
    _Alignas(WF_PAR_CACHE_LINE) wf_prim_wait wait;
    /* Protected by wait.lock; closes notification-before-sleep races. */
    int posted;
    _Alignas(WF_PAR_CACHE_LINE) uint64_t steals;
    _Alignas(WF_PAR_CACHE_LINE) struct wf__par_slot slots[WF_PAR_LANE_SLOTS];
};

static struct wf__par_lane wf__par_lanes[WF_PAR_MAX_LANES];

/* Relaxed scan bound, reduced after partial startup. Even a stale larger
 * value names initialized empty deques; it never grants an unstarted lane. */
static int wf__par_lane_count;
static unsigned wf__par_started;

_Alignas(WF_PAR_CACHE_LINE) static unsigned long long wf__par_idle;

/* The idle window this pool runs with, in microseconds, or zero for the fixed
 * round bound alone. Written once by wf__par_start before any worker thread
 * exists and read-only afterwards, so no lane can observe it changing. */
static uint64_t wf__par_idle_window_us;

/* The publish epoch: one counter, on a line of its own, that advances at every
 * transition which can make a deque's stealable content differ from what a
 * scan just saw. It exists so that a lane spinning out a long idle window does
 * not have to read every other lane's deque to learn that nothing changed.
 *
 * wf__par_find loads each victim's `top` and `bottom` -- two lines per lane,
 * and exactly the lines a working lane writes on every push and pop. Three
 * idle lanes doing that every round at the tail of a call take those lines
 * away from the one lane still working, on its SMT sibling above all. The
 * hosted A/B of the window against a WF_PAR_IDLE_WINDOW_US=0 control (run
 * 34638747514) measures the bill: the control's process CPU is 0.956 to 0.977
 * of the windowed runtime's at fir W=2 and W=4 and 0.958 to 0.972 at
 * mandelbrot -- the window spends three to four percent more CPU for its wall
 * -- and records W=4, which is memory-bound and crosses 64 chunk boundaries a
 * call, loses 4.5 percent of wall outright (0.955, control lower in four of
 * five pairs). An idle lane inside the window now reads ONE shared line per
 * round instead of two per other lane, and scans only when that line changes.
 *
 * The counter is padded to a full line because it is written on every push and
 * every successful steal: sharing a line with anything else would hand that
 * neighbour the invalidation traffic this exists to concentrate. */
struct wf__par_epoch_cell {
    _Alignas(WF_PAR_CACHE_LINE) uint64_t value;
    unsigned char padding[WF_PAR_CACHE_LINE - sizeof(uint64_t)];
};
static struct wf__par_epoch_cell wf__par_epoch;

/* Release, so that everything this thread wrote before the bump -- the ring
 * cell and the published `bottom` above all -- is visible to a lane that
 * acquire-reads the new value. */
static void wf__par_epoch_bump(void) {
    (void)__atomic_add_fetch(&wf__par_epoch.value, UINT64_C(1), __ATOMIC_RELEASE);
}

/* Acquire, the other half of that pair. */
static uint64_t wf__par_epoch_read(void) {
    return __atomic_load_n(&wf__par_epoch.value, __ATOMIC_ACQUIRE);
}

static _Thread_local struct wf__par_lane *wf__par_self;

static _Thread_local int wf__par_attached;

static unsigned wf__par_ready;
static wf_prim_thread wf__par_threads[WF_PAR_MAX_LANES];

extern size_t wf__floor_stack_bytes(void);

#ifdef WF_PLACEMENT_PAD
/* A MEASUREMENT INSTRUMENT, compiled only when this macro is defined and
 * defined by nothing that ships. No shipped build, no gate target and no test
 * defines it; whitefootc never passes it. It exists so the compute scoreboard's
 * A/B twin can hold its two arms byte-identical in behaviour while moving one
 * arm's CODE PLACEMENT, which is that bundle's largest confound and which its
 * within-pass twin cannot otherwise separate from a runtime change: every
 * candidate the twin has measured so far differed in bytes as well as in
 * behaviour, so a reading could never be attributed to one rather than the
 * other. Defined, it adds this never-called function ahead of every function
 * below it in this file, so the scheduler core's text moves by its size and
 * nothing else about the program changes.
 *
 * It is external and noinline so it survives to the link with nothing calling
 * it, and its body is a dependent chain on an argument so no optimizer folds
 * it away. Remove it when the scoreboard no longer needs a placement-only arm
 * -- there is no other reader. The measurement is in
 * research/investigations/compute-runtime/RESULTS.md, the placement
 * sensitivity measured with a shifted null arm. */
uint64_t wf__par_placement_pad(uint64_t seed);
#define WF_PLACEMENT_PAD_STEP(k)                                               \
    acc = acc * 6364136223846793005ULL + (uint64_t)(k);                        \
    acc ^= acc >> 29;
#define WF_PLACEMENT_PAD_EIGHT(k)                                              \
    WF_PLACEMENT_PAD_STEP((k) + 1)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 2)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 3)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 4)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 5)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 6)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 7)                                             \
    WF_PLACEMENT_PAD_STEP((k) + 8)
__attribute__((noinline)) uint64_t wf__par_placement_pad(uint64_t seed) {
    uint64_t acc = seed;
    WF_PLACEMENT_PAD_EIGHT(0)
    WF_PLACEMENT_PAD_EIGHT(8)
    WF_PLACEMENT_PAD_EIGHT(16)
    WF_PLACEMENT_PAD_EIGHT(24)
    return acc;
}
#endif

static void wf__par_signal(struct wf__par_lane *lane) {
    wf_prim_wait_lock(&lane->wait);
    lane->posted = 1;
    /* BUMP SITE. This is the one path that makes a lane's wait satisfiable
     * without a push: wf__par_wake_one reaches it from wf__par_publish, and
     * wf__par_execute reaches it when a finished callback notifies a
     * registered waiter. A registered waiter is always already parked, so no
     * lane spinning on the epoch can be waiting on this today; the bump is
     * here so the epoch stays a superset of "something a non-running lane
     * could care about" if a waiter ever registers before it parks. It costs
     * nothing measurable: this path already takes the station's lock. */
    wf__par_epoch_bump();
#if defined(WF_SCHED_TEST)
    wf_sched_test_signal_locked();
#endif
    wf_prim_wait_signal(&lane->wait);
    wf_prim_wait_unlock(&lane->wait);
}

static void wf__par_wake_one(void) {
    unsigned long long parked = __atomic_load_n(&wf__par_idle, __ATOMIC_RELAXED);
    while (parked != 0) {
        int index = __builtin_ctzll(parked);
        unsigned long long bit = 1ull << index;

        if ((__atomic_fetch_and(&wf__par_idle, ~bit, __ATOMIC_ACQ_REL) & bit) != 0) {
            wf__par_signal(&wf__par_lanes[index]);
            return;
        }
        parked &= ~bit;
    }
}

/* The ring cell is atomic even for losing thieves delayed across wraparound.
 * Index publication orders the frame; only the successful claim permits a
 * thief to dereference it. SC thief index loads share the owner's pop order. */
static void wf__par_push(struct wf__par_lane *lane, struct wf__par_slot *slot) {
    unsigned long long bottom = lane->bottom;
    __atomic_store_n(&lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)], slot, __ATOMIC_RELAXED);
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_SEQ_CST);
    /* BUMP SITE, and the only one where a slot becomes stealable. It is after
     * the publication above and not before: the release bump orders the ring
     * cell and this `bottom` ahead of itself, so a lane that acquire-reads the
     * new epoch and then scans sees the item. wf__par_publish is this
     * function's only caller. */
    wf__par_epoch_bump();
}

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

        __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
        return NULL;
    }
    slot = __atomic_load_n(&lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)], __ATOMIC_RELAXED);
    if ((long long)(bottom - top) > 0) {
        return slot;
    }

    if (!__atomic_compare_exchange_n(&lane->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
        slot = NULL;
    }
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
    return slot;
}

static struct wf__par_slot *wf__par_steal(struct wf__par_lane *victim) {
    unsigned long long top = __atomic_load_n(&victim->top, __ATOMIC_SEQ_CST);
    unsigned long long bottom = __atomic_load_n(&victim->bottom, __ATOMIC_SEQ_CST);
    struct wf__par_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
#if defined(WF_SCHED_TEST)
    wf_sched_test_before_steal_read();
#endif
    slot = __atomic_load_n(&victim->buffer[top & (WF_PAR_LANE_SLOTS - 1)], __ATOMIC_RELAXED);
    if (!__atomic_compare_exchange_n(&victim->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
#if defined(WF_SCHED_TEST)
        wf_sched_test_after_steal(0);
#endif
        return NULL;
    }
    /* BUMP SITE, and the one that closes the epoch's only hole. A thief that
     * loses this CAS gives up on that victim for the rest of its scan and can
     * finish the scan empty-handed while the victim still holds work. The
     * winner's CAS is strictly between the loser's `top` read and the loser's
     * own CAS, so it is strictly after the loser read the epoch, so this bump
     * makes the loser's next epoch load differ and sends it back for another
     * scan. Without it a loser could idle on an unchanged epoch until the next
     * push -- never losing the work, which the victim's owner pops at its own
     * join, but declining to help. */
    wf__par_epoch_bump();
#if defined(WF_SCHED_TEST)
    wf_sched_test_after_steal(1);
#endif
#if WF_SCHED_STATS
    uint64_t n = __atomic_load_n(&wf__par_self->steals, __ATOMIC_RELAXED);
    __atomic_store_n(&wf__par_self->steals, n + 1, __ATOMIC_RELAXED);
#endif
    return slot;
}

static struct wf__par_slot *wf__par_find(struct wf__par_lane *lane) {
    int count = __atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED);
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

/* After DONE, only atomic waiter metadata in permanent slot storage may be
 * read. The owner can already reuse the frame; a late signal is harmless. */
static void wf__par_execute(struct wf__par_slot *slot) {
    struct wf__par_lane *waiter;
    slot->run(slot->frame);
#if defined(WF_SCHED_TEST)
    wf_sched_test_before_done(slot->frame);
#endif
    __atomic_store_n(&slot->state, WF_PAR_SLOT_DONE, __ATOMIC_SEQ_CST);
#if defined(WF_SCHED_TEST)
    wf_sched_test_after_done();
#endif
    waiter = __atomic_load_n(&slot->waiter, __ATOMIC_SEQ_CST);
    if (waiter != NULL) {
        wf__par_signal(waiter);
    }
#if defined(WF_SCHED_TEST)
    wf_sched_test_after_notify();
#endif
}

/* One lane's idle stretch: misses since the last unit of work, the clock
 * reading taken when those misses first reached the spin bound, and the
 * publish epoch as it stood when this lane last began a scan. `entered` is
 * zero until that first crossing, and zero is also what a host with no
 * monotonic clock answers, which is why the window is skipped on a zero
 * reading rather than treating it as an origin. */
struct wf__par_idling {
    int rounds;
    uint64_t entered;
    uint64_t seen;
};

/* A lane that has just run something is no longer idle. */
static void wf__par_idling_reset(struct wf__par_idling *idling) {
    idling->rounds = 0;
    idling->entered = 0;
    idling->seen = 0;
}

/* Whether this round scans the deques at all.
 *
 * Three regimes, and only the third is new:
 *
 *   - No window (an oversubscribed pool, a host with no CPU count, or any
 *     build with WF_PAR_IDLE_WINDOW_US at zero, which includes every
 *     WF_SCHED_TEST build). Scan every round and never touch the epoch: this
 *     is the loop exactly as it was, down to the loads it performs.
 *   - Inside the window, first spin bound. `entered` is still zero, so scan
 *     every round and keep the fast path for work that arrives immediately.
 *     The epoch is read before each of those scans, which is what makes the
 *     first gated round compare against a value that was current BEFORE the
 *     last full scan began rather than after it.
 *   - Inside the window, past the first spin bound. Read the epoch; scan only
 *     when it differs from the value read before the previous scan.
 *
 * WHY THE EPOCH CANNOT BE MISSED. Say this lane read epoch v, scanned, and
 * found nothing, and some slot is stealable afterwards. For each victim the
 * scan either saw an empty deque or lost the claim CAS. If it saw empty and
 * the victim is not empty now, a push happened after that observation, and
 * the push's release bump therefore produced a value above v. If it lost the
 * CAS, the winner's CAS lay between this lane's SEQ_CST `top` read and its own
 * CAS -- both after the epoch read -- so the winner's bump also produced a
 * value above v. Either way the next acquire load differs from v and the lane
 * scans again, and because that load acquires what the bump released, the
 * rescan sees the ring cell and the `bottom` the push wrote. The counter only
 * ever increases, so no bump can be cancelled by another, and reading it
 * BEFORE the scan rather than after is what makes "scanned at v" mean "the
 * scan began no earlier than v was current". */
static int wf__par_should_scan(struct wf__par_idling *idling) {
    uint64_t now;
    if (wf__par_idle_window_us == 0) {
        return 1;
    }
    if (idling->entered == 0) {
        idling->seen = wf__par_epoch_read();
        return 1;
    }
    now = wf__par_epoch_read();
    if (now == idling->seen) {
        return 0;
    }
    idling->seen = now;
    return 1;
}

/* One idle step, shared by both wait loops so they cannot drift apart.
 *
 * Returns 1 when the lane stayed hot and the caller must re-check its own work
 * and exit conditions -- every call performs at most one spin hint or one
 * yield, so the top of each loop is reached on every round -- and 0 when the
 * lane has exhausted spinning and yielding and must park. What each loop
 * re-checks there is its own: the join wait loads its target's state on every
 * round, separately from wf__par_should_scan and never gated by the epoch, so
 * a completion it is waiting for is seen as promptly as before; the worker
 * loop has no exit of its own, since its lanes live until the process does.
 *
 * Both loops take the window. The worker idle loop is the parked-helper case
 * the placement evidence is about, and the join wait is the lane that owns the
 * frame and can help; the CPU the 16,384-round twin saved at W=2 and W=4 came
 * from both loops parking and re-waking less often inside one call, and a rule
 * that held for only one of them would leave the other parking on the same
 * host for the same reason. Neither loop has a reason of its own to differ,
 * and the probes that drive the park protocol run with the window at zero. */
static int wf__par_stay_hot(struct wf__par_idling *idling) {
    if (idling->rounds < WF_PAR_SPIN_ROUNDS) {
        idling->rounds += 1;
        wf_prim_spin_hint();
        return 1;
    }
    if (idling->rounds == WF_PAR_SPIN_ROUNDS && wf__par_idle_window_us != 0) {
        /* One clock read per spin bound, never per round. The window runs from
         * the first crossing, so a lane stays hot for the window plus the one
         * bound that opened it. */
        uint64_t now = wf_prim_monotonic_us();
        if (now != 0) {
            if (idling->entered == 0) {
                idling->entered = now;
            }
            if (now - idling->entered < wf__par_idle_window_us) {
                idling->rounds = 0;
                wf_prim_spin_hint();
                return 1;
            }
        }
    }
    if (idling->rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
        idling->rounds += 1;
        wf_prim_yield();
        return 1;
    }
    return 0;
}

/* Nested helping is safe for structured compute calls: no I/O continuation
 * can strand this stack. Register the waiter before the final SC DONE check;
 * signal takes the same native lock as sleep, so no wake can be lost. */
static void wf__par_wait(struct wf__par_lane *lane, struct wf__par_slot *target) {
    struct wf__par_idling idling;
    wf__par_idling_reset(&idling);
    for (;;) {
        /* Every round, and never behind the epoch gate: this is the lane's own
         * completion flag. It shares a line with its neighbouring slots in the
         * owner's slot array, but its writer is the thief that will finish the
         * very task being waited on, which is the one line this lane has to
         * watch. */
        if (__atomic_load_n(&target->state, __ATOMIC_ACQUIRE) == WF_PAR_SLOT_DONE) {
            return;
        }

        if (wf__par_should_scan(&idling)) {
            struct wf__par_slot *slot = wf__par_pop(lane);
            if (slot == NULL) {
                slot = wf__par_find(lane);
            }
            if (slot != NULL) {
                wf__par_execute(slot);
                wf__par_idling_reset(&idling);
                continue;
            }
        }
        if (wf__par_stay_hot(&idling)) {
            continue;
        }

        wf_prim_wait_lock(&lane->wait);
        __atomic_store_n(&target->waiter, lane, __ATOMIC_SEQ_CST);
        while (__atomic_load_n(&target->state, __ATOMIC_SEQ_CST) != WF_PAR_SLOT_DONE) {
#if defined(WF_SCHED_TEST)
            wf_sched_test_before_wait(target->frame);
#endif
            wf_prim_wait_sleep(&lane->wait);
        }
        __atomic_store_n(&target->waiter, NULL, __ATOMIC_RELAXED);
        lane->posted = 0;
        wf_prim_wait_unlock(&lane->wait);
        return;
    }
}

static void wf__par_worker_main(void *opaque) {
    struct wf__par_lane *lane = (struct wf__par_lane *)opaque;
    struct wf__par_idling idling;
    wf__par_idling_reset(&idling);
    wf__par_self = lane;
    wf__par_attached = 1;
    wf_prim_floor_attach();

    __atomic_add_fetch(&wf__par_ready, 1u, __ATOMIC_RELEASE);

    for (;;) {
        struct wf__par_slot *slot;
        if (wf__par_should_scan(&idling)) {
            slot = wf__par_find(lane);
            if (slot != NULL) {
                wf__par_execute(slot);
                wf__par_idling_reset(&idling);
                continue;
            }
        }
        if (wf__par_stay_hot(&idling)) {
            continue;
        }

        /* Announce before the final scan: a publisher either sees our bit,
         * or its SC publication is visible to that scan. */
        __atomic_fetch_or(&wf__par_idle, 1ull << (lane - wf__par_lanes), __ATOMIC_SEQ_CST);
        slot = wf__par_find(lane);
        if (slot != NULL) {
            __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)),
                               __ATOMIC_ACQ_REL);
            wf__par_execute(slot);
            wf__par_idling_reset(&idling);
            continue;
        }
        wf_prim_wait_lock(&lane->wait);
        if (!lane->posted) {

            wf_prim_wait_sleep(&lane->wait);
        }
        lane->posted = 0;
        wf_prim_wait_unlock(&lane->wait);
        __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)), __ATOMIC_ACQ_REL);
        wf__par_idling_reset(&idling);
    }

}

static void wf__par_prepare(struct wf__par_lane *lane, int index) {
    int slot;
    lane->top = 0;
    lane->bottom = 0;
    lane->seed = 0x9e3779b97f4a7c15ull * (unsigned long long)(index + 1);
    lane->posted = 0;
    for (slot = 0; slot < (int)WF_PAR_LANE_SLOTS; slot += 1) {
        lane->slots[slot].home = lane;
        lane->slots[slot].state = WF_PAR_SLOT_FREE;
        lane->slots[slot].waiter = NULL;
        lane->slots[slot].next_free = slot + 1;
    }
    lane->slots[WF_PAR_LANE_SLOTS - 1].next_free = -1;
    lane->free_head = 0;
}

static void wf__par_start(void) {
    int requested = wf__sched_lanes();
    unsigned cpus;
    int started = 0;
    if (requested < 2) return;
    /* The oversubscription test, answered once, here, because it is a property
     * of the pool and not of a lane's current state: the lanes this pool is
     * about to run against the CPUs this process may actually use. Lanes at or
     * below that count take the idle window; more lanes than CPUs keep the
     * fixed round bound, which is what the never-park twin's 2.7x to 6.7x at
     * eight lanes on four CPUs says to do. An unknown CPU count -- zero from
     * the primitive -- also keeps the fixed bound, since a window would then
     * rest on an assumption nothing checked. `requested` is the count asked
     * for; a partial startup can only leave fewer lanes than that, so this
     * never turns the window on for a pool that is oversubscribed. Written
     * before the first worker thread exists, and never written again. */
    cpus = wf_prim_online_cpus();
    wf__par_idle_window_us =
        (cpus != 0u && (unsigned)requested <= cpus) ? (uint64_t)WF_PAR_IDLE_WINDOW_US : 0u;
    /* Prepare every deque before publishing a scan bound. Initialize the
     * owner's wait before any worker exists; preserve started workers on
     * later failure, and never destroy a station a worker may reach. */
    for (int i = 0; i < requested; ++i) wf__par_prepare(&wf__par_lanes[i], i);
#if defined(WF_SCHED_TEST)
    if (!wf_sched_test_allow_owner_wait()) return;
#endif
    if (wf_prim_wait_init(&wf__par_lanes[0].wait) != 0) return;
    __atomic_store_n(&wf__par_lane_count, requested, __ATOMIC_RELAXED);
    for (int i = 1; i < requested; ++i) {
        struct wf__par_lane *lane = &wf__par_lanes[i];
        if (wf_prim_wait_init(&lane->wait) != 0) break;
#if defined(WF_SCHED_TEST)
        if (!wf_sched_test_allow_worker((unsigned)i)) {
            wf_prim_wait_destroy(&lane->wait);
            break;
        }
#endif
        if (wf_prim_thread_start(&wf__par_threads[i], wf__par_worker_main,
                lane, wf__floor_stack_bytes()) != 0) {
            wf_prim_wait_destroy(&lane->wait);
            break;
        }
        started = i;
    }
    __atomic_store_n(&wf__par_lane_count, started ? started + 1 : 0, __ATOMIC_RELAXED);
    if (!started) {
        wf_prim_wait_destroy(&wf__par_lanes[0].wait);
        return;
    }
    while (__atomic_load_n(&wf__par_ready, __ATOMIC_ACQUIRE) < (unsigned)started)
        wf_prim_yield();
}

static struct wf__par_lane *wf__par_attach(void) {
    static int taken;
    int expected = 0;
    wf__par_attached = 1;
    wf__sched_once(&wf__par_started, wf__par_start);
    if (__atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED) == 0) {
        return NULL;
    }
    if (!__atomic_compare_exchange_n(&taken, &expected, 1, 0, __ATOMIC_ACQ_REL,
                                     __ATOMIC_RELAXED)) {
        return NULL;
    }
    wf__par_self = &wf__par_lanes[0];
    return wf__par_self;
}

void *wf__par_acquire_lane(uint64_t bytes) {
    struct wf__par_lane *lane = wf__par_self;
    struct wf__par_slot *slot;
    int index;
    if (bytes > (unsigned long)WF_PAR_FRAME_BYTES) {
        return NULL;
    }
    if (lane == NULL) {
        if (wf__par_attached) {

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

        target->run(target->frame);
        return;
    }
    while (slot != NULL) {

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

int wf__par_pool_active(void) { return wf__sched_lanes() >= 2; }

uint64_t wf__par_split_budget(uint64_t span, uint64_t weight) {
    struct wf__par_lane *lane = wf__par_self;
    int lanes = wf__sched_lanes();
    uint64_t work = wf__sched_split_work();
    uint64_t want;
    uint64_t affordable;
    uint64_t chunks;
    uint64_t budget;
    if (lanes < 2 || work == 0) {
        return 0;
    }
    if (lane != NULL) {
        unsigned long long bottom = __atomic_load_n(&lane->bottom, __ATOMIC_RELAXED);
        unsigned long long top = __atomic_load_n(&lane->top, __ATOMIC_RELAXED);
        if ((long long)(bottom - top) > 0) {
            return 0;
        }
    }
    if (weight == 0) {
        weight = 1;
    }
    want = (uint64_t)lanes * WF_PAR_SPLIT_OVERSUBSCRIBE;

    if (weight >= work) {
        affordable = span;
    } else {
        affordable = span / ((work + weight - 1) / weight);
    }
    chunks = (want < affordable) ? want : affordable;

    budget = 0;
    while ((chunks >> 1) != 0) {
        chunks >>= 1;
        budget += 1;
    }
    return budget;
}

/* How many times a call into an ordinary recursive component may hand work
 * out before its callees enter the sequential clone.
 *
 * The loop splitter's allowance above answers the same question for a counted
 * range and is read the same way: one query at entry, an ordinary value
 * carried down, and a leaf world with no scheduler test below the cut. With no
 * pool -- fewer than two lanes -- one lane's worth of leaves is still the
 * honest answer, because the compiled entry may run without a pool and its
 * sequential clone is what it then descends into. A scheduler-less link never
 * reaches this definition at all: the module's own weak stub answers zero and
 * the first node runs the clone. */
uint64_t wf__par_recursion_budget(void) {
    int lanes = wf__sched_lanes();
    uint64_t leaves;
    uint64_t budget;
    if (lanes < 2) {
        lanes = 1;
    }
    leaves = (uint64_t)lanes * WF_PAR_RECURSION_LEAVES_PER_LANE;
    budget = 0;
    while ((leaves >> 1) != 0) {
        leaves >>= 1;
        budget += 1;
    }
    if (budget > WF_PAR_RECURSION_MAX_BUDGET) {
        budget = WF_PAR_RECURSION_MAX_BUDGET;
    }
    return budget;
}

unsigned wf__sched_pool_running(void) {
    int count = __atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED);
    return count > 1 ? (unsigned)count - 1 : 0;
}
unsigned long wf__par_grants(void) {
    uint64_t sum = 0;
    for (unsigned i = 0; i < WF_PAR_MAX_LANES; ++i)
        sum += __atomic_load_n(&wf__par_lanes[i].steals, __ATOMIC_RELAXED);
    return (unsigned long)sum;
}
