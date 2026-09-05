/* The scheduler core as the process's runtime. See `entry.h` for the shape and
 * `research/investigations/io-model/PARK-ON-MISS.md` §5 and §7 for the design
 * each part cites.
 *
 * What this unit is, in one line: the platform layer over `core.c`, carrying
 * the startup policy and the module ABI that `par_runtime.c` used to carry
 * over a second scheduler of its own.
 *
 * POSIX only. Windows takes the same core behind `prim_windows.c` and its own
 * floor, which is a later step; nothing here is `#if`-forked for it. */

#define _DARWIN_C_SOURCE 1

#include "entry.h"
#include "prim.h"

#include <pthread.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <sys/sysctl.h>
#endif

/* ---------------------------------------------------------- the floor */

/* The exhaustion floor, which is linked into every Whitefoot program and
 * defines all three of these strongly.
 *
 * The weak answers below are for the links that carry the core without the
 * floor -- the completion harness, the bridge and core/read probes, the smoke
 * -- so that the core is linkable on its own, which is the property `prim.h`
 * already keeps for the switch's bounds. None of those links reaches
 * `wf__sched_start`, so the fallback stack size is a size no shipped program
 * ever uses; it is stated rather than left to a null call. */
#define WF_SCHED_ENTRY_FALLBACK_STACK_BYTES ((size_t)1024u * 1024u)

__attribute__((weak)) size_t wf__floor_stack_bytes(void) {
    return WF_SCHED_ENTRY_FALLBACK_STACK_BYTES;
}

/* The per-thread half of the floor's attach: the alternate signal stack a
 * guard-page handler runs on, installed once at thread start (§7's
 * `wf__floor_attach_thread` bullet). The per-stack half is the bounds, and
 * those the switch writes. */
__attribute__((weak)) void wf__floor_attach_thread(void) {}

/* The per-stack half, called here for one case the switch does not cover: the
 * entry thread's return to its own host stack, after which the bounds the last
 * switch wrote name a pool stack this thread is no longer on. A null pair
 * means "this thread's host stack", whose bounds the attach above captured. */
extern void wf__floor_set_stack_bounds(unsigned char *low, unsigned char *high);

/* --------------------------------------------------------- the policy */

/* An upper bound on threads of execution, so an oversized WF_WORKERS value
 * cannot ask for unbounded threads. It is a resource ceiling, not a language
 * constant, and it is the core's own thread ceiling: a lane is a thread. */
#define WF_PAR_MAX_LANES ((int)WF_SCHED_MAX_THREADS)

/* How many stacks a run reserves beyond the thread count.
 *
 * The floor is the thread count plus one and the core enforces it (§5): every
 * thread holds one pool stack while it runs, so a first park needs one more.
 * Above that floor the count is free and nothing ties it to WF_WORKERS, and
 * this is the default the corpus runs on -- eight stacks that may be parked at
 * once, which is what makes a program with several outstanding operations park
 * rather than take §2's fourth line. WF_STACKS names another number. */
#define WF_SCHED_SPARE_STACKS 8u

/* How far a permitted counted loop's range split may descend, in two policy
 * constants. `wf__par_split_budget` is where they are applied and what the
 * argument for each of them is; both are measured machine numbers, and both
 * are the lowering's only tuning surface -- no constant of this kind is in the
 * compiler and none is in the language.
 *
 * OVERSUBSCRIBE is how many chunks the pool wants per lane. One per lane
 * leaves a lane that finishes early with nothing to steal, and this machine's
 * four performance and six efficiency cores make that costly: the slow cores
 * straggle, and the fast ones have nothing left to take. Measured on the grid
 * workload at the shipped default of ten lanes, minimum of forty interleaved
 * runs: 4 chunks per lane read 0.0822 s, 16 read 0.0754 s, and 64 read
 * 0.0732 s, against 0.0763 s for the hand-written recursive twin of the same
 * program. Sixteen is the near edge of that flat region; going further buys
 * three percent and cuts the range finer than any measurement asks for.
 *
 * It cannot reopen the hazard the other constant closes, because the two are
 * combined with a minimum: a range too small to be worth splitting is bounded
 * by the work term whatever the pool would like.
 *
 * WORK_PER_CHUNK is how much work, in the estimated instruction-equivalents of
 * one iteration, a chunk must be worth before publishing it can pay for
 * itself. It is the constant that closes the over-split hazard: splitting a
 * range that is not worth splitting is a real regression, not a wash, and an
 * allowance derived from the lane count alone produces it on every small loop.
 * Measured directly, by sweeping the width of a counted loop of estimated
 * weight 150 over a fixed total amount of work and comparing the split against
 * the same program's sequential build at the shipped default:
 *
 *     width     2 000   8 000  32 000  128 000  512 000
 *     split    0.0831  0.0269  0.0116   0.0084   0.0072
 *     plain    0.0205  0.0196  0.0201   0.0201   0.0210
 *     ratio     4.1x    1.37x   0.58x    0.42x    0.34x
 *                loss    loss     win      win      win
 *
 * The crossing is near a width of 16 000, so a whole range first pays for two
 * chunks at about 16 000 x 150 = 2.4e6 instruction-equivalents, and one chunk
 * is worth publishing at about half that. */
#define WF_PAR_SPLIT_OVERSUBSCRIBE 16
#define WF_PAR_SPLIT_WORK_PER_CHUNK 1200000

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
static int wf__sched_default_lanes(void) {
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

/* The lane count this run asked for: 0 for the sequential world, or at least
 * two. A value below two is the explicit opt-out. */
static int wf__sched_requested_lanes(void) {
    const char *setting = getenv("WF_WORKERS");
    char *end = NULL;
    long requested;
    if (setting == NULL || setting[0] == '\0') {
        return wf__sched_default_lanes();
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

/* The stack count: WF_STACKS when it is written and readable, otherwise the
 * thread count plus the spare above. Either way the core raises it to its own
 * floor of the thread count plus one and refuses a count past its ceiling, so
 * this only has to state a number and clamp it. */
static unsigned wf__sched_stack_count(unsigned threads) {
    const char *setting = getenv("WF_STACKS");
    char *end = NULL;
    long written;
    unsigned count = threads + WF_SCHED_SPARE_STACKS;
    if (setting != NULL && setting[0] != '\0') {
        written = strtol(setting, &end, 10);
        if (end != setting && *end == '\0' && written > 0) {
            count = (unsigned long)written > (unsigned long)WF_SCHED_MAX_STACKS
                ? WF_SCHED_MAX_STACKS
                : (unsigned)written;
        }
    }
    if (count > WF_SCHED_MAX_STACKS) {
        count = WF_SCHED_MAX_STACKS;
    }
    return count;
}

/* ----------------------------------------------------------- the core */

wf_sched_core wf__sched_core;

/* Whether this thread is one the core runs, and so whether it is on a pool
 * stack at all. Set by `wf__sched_enter` and by nothing else, which is what
 * makes it exact: `wf_prim_thread_index` answers 0 on every thread that never
 * entered the core, so a target helper asking `wf_sched_current_stack` would
 * otherwise be told it is on the entry thread's stack. */
static _Thread_local int wf__sched_on_core;

/* Worker threads that reached their entry, so a shutdown can tell a process
 * whose pool is still running from one that never started a thread. */
static unsigned wf__sched_workers;

/* The rendezvous `par_runtime.c` made and this keeps: "the pool started" means
 * "the pool can take work". A pool whose threads are still being created
 * cannot take the first offers a program makes, and for a short program those
 * are all the offers there are; this costs one rendezvous per process. */
static pthread_mutex_t wf__sched_ready_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf__sched_ready_signal = PTHREAD_COND_INITIALIZER;
static unsigned wf__sched_ready;

int wf__sched_enter(unsigned thread, void (*entry)(void *), void *argument) {
    int status;
    wf__sched_on_core = 1;
    status = wf_sched_run(&wf__sched_core, thread, entry, argument);
    /* Only thread 0 arrives here, on its host stack (§5). The last switch
     * wrote a pool stack's bounds into this thread's floor state, and this
     * thread is no longer on that stack, so the host stack's own bounds go
     * back before anything runs here. */
    wf__sched_on_core = 0;
    wf__floor_set_stack_bounds(NULL, NULL);
    return status;
}

wf_sched_stack *wf__sched_current_stack(void) {
    if (!wf__sched_on_core) {
        return NULL;
    }
    return wf_sched_current_stack(&wf__sched_core);
}

unsigned wf__sched_pool_running(void) {
    return __atomic_load_n(&wf__sched_workers, __ATOMIC_ACQUIRE);
}

static wf_sched_lane *wf__sched_current_lane(void) {
    if (!wf__sched_on_core) {
        return NULL;
    }
    return wf_sched_current_thread(&wf__sched_core)->lane;
}

/* ------------------------------------------------------- the workers */

static void *wf__sched_worker_main(void *argument) {
    unsigned index = (unsigned)(uintptr_t)argument;
    /* The floor's per-thread half, before anything else: a worker runs
     * ordinary Whitefoot calls on a pool stack and can recurse exactly as deep
     * as the offering thread would have, so it needs the same alternate signal
     * stack. The bounds it captures here are this thread's *host* stack, which
     * is where the handler runs until the first switch and where nothing
     * Whitefoot ever runs (§5). */
    wf__floor_attach_thread();
    pthread_mutex_lock(&wf__sched_ready_lock);
    wf__sched_ready += 1u;
    pthread_cond_signal(&wf__sched_ready_signal);
    pthread_mutex_unlock(&wf__sched_ready_lock);
    /* Never returns: a worker's scheduler loop is the program's, and the
     * thread is detached. */
    (void)wf__sched_enter(index, NULL, NULL);
    return NULL;
}

/* Starts the pool at the first hand-out, once per process (§5).
 *
 * The entry thread's loop and every worker's loop are the same loop, so a
 * program that submits I/O and never hands out needs no worker at all: the
 * entry thread parks its own stack at the miss and its own loop resumes that
 * stack when the completion arrives. Starting the workers eagerly at the
 * core's entry instead was measured on the io bench at the shipped default:
 * 0.2851 s eager against 0.1083 s for this lazy start and 0.1063 s before the
 * core became the runtime. The threads a short program never uses were the
 * whole difference, so the pool starts under a `pthread_once` inside the first
 * lane acquisition, as `par_runtime.c` started it. `wf__par_pool_active` still
 * answers from the setting, so the module's choice between its two lowerings
 * does not depend on whether the pool has started yet.
 *
 * A refused stack request is a silent downgrade to the pthread default, which
 * is exactly the hazard this call exists to close, so it stops the pool
 * instead: no lane is better than a lane that overflows. A thread that cannot
 * be created stops the loop at the count that did start; the lanes past it
 * stay empty, which costs a steal pass and nothing else. */
static void wf__sched_start_workers(unsigned threads) {
    pthread_attr_t attributes;
    unsigned index;
    unsigned started = 0;
    if (pthread_attr_init(&attributes) != 0) {
        return;
    }
    if (pthread_attr_setstacksize(&attributes, wf__floor_stack_bytes()) != 0) {
        (void)pthread_attr_destroy(&attributes);
        return;
    }
    (void)pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    for (index = 1u; index < threads; index += 1u) {
        pthread_t thread;
        /* The worker's pool stack is taken here, on the thread that creates
         * it: its own start may come arbitrarily late, after the entry thread
         * has parked every free stack, and a thread's first act is not a join
         * and has no rule to fall to (§5, and the enumerator's sixth finding). */
        if (wf_sched_start_thread(&wf__sched_core, index) != 0) {
            break;
        }
        if (pthread_create(
                &thread, &attributes, wf__sched_worker_main, (void *)(uintptr_t)index
            ) != 0) {
            break;
        }
        started += 1u;
    }
    (void)pthread_attr_destroy(&attributes);
    __atomic_store_n(&wf__sched_workers, started, __ATOMIC_RELEASE);
    pthread_mutex_lock(&wf__sched_ready_lock);
    while (wf__sched_ready < started) {
        pthread_cond_wait(&wf__sched_ready_signal, &wf__sched_ready_lock);
    }
    pthread_mutex_unlock(&wf__sched_ready_lock);
}

/* The thread count the core was initialized with, which is what the first
 * hand-out starts. One until `wf__sched_start` has run, so a hand-out on a
 * link whose core never started (there is none: every such link refuses the
 * hand-out first, above) would start nothing. */
static unsigned wf__sched_threads = 1u;
static pthread_once_t wf__sched_workers_once = PTHREAD_ONCE_INIT;

static void wf__sched_start_workers_once(void) {
    if (wf__sched_threads >= 2u) {
        wf__sched_start_workers(wf__sched_threads);
    }
}

/* --------------------------------------------------------- the entry */

int wf__sched_start(void) {
    int requested = wf__sched_requested_lanes();
    unsigned threads = requested >= 2 ? (unsigned)requested : 1u;
    /* The stack reservation is made here, at the core's entry, before the
     * program's body runs and in every core link (§5). It is one mapping with
     * a guard page per slot and its pages committed on touch, so a run that
     * never parks pays for address space and nothing else. The workers are
     * not started here; see `wf__sched_start_workers`. */
    if (wf_sched_init(
            &wf__sched_core, threads, wf__sched_stack_count(threads), wf__floor_stack_bytes()
        ) != 0) {
        return 1;
    }
    wf__sched_threads = threads;
    return 0;
}

struct wf__sched_entry_call {
    int (*body)(int, char **);
    int argc;
    char **argv;
};

/* The program's body, on a pool stack whose bottom is the scheduler loop.
 *
 * The post is what lets the entry thread's loop leave and what wakes it if it
 * is asleep somewhere else: after main's stack parks, a worker may resume it,
 * run it to its return and post from there, and nothing else in §6 would wake
 * the thread that has to return the status. */
static void wf__sched_entry_body(void *argument) {
    struct wf__sched_entry_call *call = argument;
    int status = call->body(call->argc, call->argv);
    wf_sched_post_status(&wf__sched_core, status);
}

int wf__sched_entry_stack(
    int (*body)(int, char **),
    int argc,
    char **argv,
    int *status
) {
    struct wf__sched_entry_call call;
    if (body == NULL || status == NULL) {
        return 0;
    }
    if (wf__sched_start() != 0) {
        /* The reservation could not be made. There is no core to run on, so
         * the caller keeps the shape it has without one; every hand-out is
         * refused below and every join waits in place. */
        return 0;
    }
    call.body = body;
    call.argc = argc;
    call.argv = argv;
    *status = wf__sched_enter(0u, wf__sched_entry_body, &call);
    return 1;
}

/* ------------------------------------------------------ the module ABI */

void *wf__par_acquire_lane(unsigned long bytes) {
    if (!wf__sched_on_core) {
        /* No pool stack, so no lane: this thread is not one the core runs,
         * which is the state a link whose core could not start leaves every
         * thread in. Refusing is a schedule the program is already correct
         * under -- the refused edge is the same call the granted edge makes. */
        return NULL;
    }
    (void)pthread_once(&wf__sched_workers_once, wf__sched_start_workers_once);
    return wf_sched_acquire(&wf__sched_core, bytes);
}

void wf__par_publish(void *frame, void (*run)(void *)) {
    wf_sched_publish(&wf__sched_core, frame, run);
}

void wf__par_join(void *frame) {
    wf_sched_join_frame(&wf__sched_core, frame);
}

void wf__par_release(void *frame) {
    wf_sched_release(&wf__sched_core, frame);
}

/* Whether this run was asked for a pool, answered once at the bootstrap.
 *
 * Not part of the lane protocol: it takes no frame, publishes nothing and
 * moves no work. It answers the one question the module cannot answer for
 * itself, so that a program built with `--par` and run without a pool takes
 * the sequential lowering of itself rather than the refused edge of the
 * overlapped one.
 *
 * It reads the setting rather than the pool, as it always did. The answer is
 * fixed for the life of the process: the environment is the process's own, the
 * core count it falls back to when the setting is absent is the machine's, the
 * pool's own start reads both through the same function, and neither a pool
 * that started nor one that did not ever changes. That is what makes it safe
 * to keep forever, which is what the caller does -- it asks at the entry and
 * never again -- and what distinguishes it from a demand signal: nothing here
 * measures what the pool is doing, so no thread ever writes a word another
 * reads on account of it.
 *
 * The one case where "asked for" and "got" differ is a pool that was requested
 * and failed to start. That run takes the overlapped world and has every
 * acquisition refused, which is a correct schedule. */
int wf__par_pool_active(void) {
    return wf__sched_requested_lanes() >= 2;
}

/* The lane count, settled once per process.
 *
 * `wf__sched_requested_lanes` reads the environment and, when nothing is set,
 * asks the machine for its core count. Both answers are fixed for the life of
 * the process and both are expensive to ask for repeatedly: measured at 129 ns
 * per call with WF_WORKERS set and 1121 ns with it unset -- which is the
 * shipped default. The allowance below is asked once per loop *entry*, which
 * is often enough for a microsecond to be a disaster, so it reads this
 * instead: 2.5-4.1 ns. Two threads that settle it at once compute the same
 * number, so the race is benign and needs no lock. */
static int wf__sched_cached_lanes = -1;

static int wf__sched_lanes_once(void) {
    int lanes = __atomic_load_n(&wf__sched_cached_lanes, __ATOMIC_RELAXED);
    if (lanes < 0) {
        lanes = wf__sched_requested_lanes();
        __atomic_store_n(&wf__sched_cached_lanes, lanes, __ATOMIC_RELAXED);
    }
    return lanes;
}

/* How many times a permitted counted loop's range may be halved [PAR-2].
 *
 * Asked once per loop entry, never per iteration, and answered from three
 * things: how many chunks this pool could use, how much work the range is
 * worth, and whether this thread is already inside parallel work.
 *
 * **Why it is sound for a schedule to depend on any of that.** Every operation
 * a permitted loop may recombine under is exactly associative on its type's
 * complete value set, so the value of the fold does not depend on the shape of
 * the combination tree. The number of chunks is therefore not observable, and
 * the same program publishes the same bytes at every worker count and at every
 * answer this function can give. If a non-associative combine were ever
 * admitted, this function would become unsound the same day.
 *
 * **Why the work term is mandatory rather than a refinement.** The alternative
 * -- deriving the descent from the lane count alone -- was measured on a small
 * loop at 15x slower with eight chunks and 53x with 1024, getting *worse* with
 * more lanes. Splitting is only free when the range is worth splitting, and
 * only the caller's static estimate of one iteration says whether it is.
 *
 * **Why a busy lane refuses.** A thread inside a chunk of some outer split is
 * already parallel; splitting again multiplies offers without adding a lane to
 * run them, and the outer split has already used the pool. One relaxed load of
 * this thread's own deque says so. */
unsigned long wf__par_split_budget(unsigned long span, unsigned long weight) {
    wf_sched_lane *lane = wf__sched_current_lane();
    int lanes = wf__sched_lanes_once();
    unsigned long want;
    unsigned long affordable;
    unsigned long chunks;
    unsigned long budget;
    if (lanes < 2) {
        return 0;
    }
    if (lane != NULL) {
        unsigned long long bottom = wf_prim_load_q(&lane->bottom, WF_PRIM_RELAXED);
        unsigned long long top = wf_prim_load_q(&lane->top, WF_PRIM_RELAXED);
        if ((long long)(bottom - top) > 0) {
            return 0;
        }
    }
    if (weight == 0) {
        weight = 1;
    }
    want = (unsigned long)lanes * WF_PAR_SPLIT_OVERSUBSCRIBE;
    /* `span * weight / WORK_PER_CHUNK`, computed so a span near the whole u64
     * range cannot wrap into a small answer. */
    if (weight >= (unsigned long)WF_PAR_SPLIT_WORK_PER_CHUNK) {
        affordable = span;
    } else {
        affordable = span / (((unsigned long)WF_PAR_SPLIT_WORK_PER_CHUNK + weight - 1) / weight);
    }
    chunks = (want < affordable) ? want : affordable;
    /* The descent is the base-two logarithm of the chunk count, so the depth
     * of the splitter's recursion is this answer and nothing else: bounded
     * above by log2(WF_PAR_MAX_LANES * OVERSUBSCRIBE), which is ten. That is a
     * theorem about the emitted shape rather than a policy -- the recursion is
     * over the allowance, never over the range, so a loop of 2^64 iterations
     * costs ten frames. */
    budget = 0;
    while ((chunks >> 1) != 0) {
        chunks >>= 1;
        budget += 1;
    }
    return budget;
}

unsigned long wf__par_grants(void) {
    wf_sched_statistics counts;
    wf_sched_statistics_sum(&wf__sched_core, &counts);
    return (unsigned long)counts.steals;
}
