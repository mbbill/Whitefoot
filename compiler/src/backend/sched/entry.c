/* The scheduler core as the process's runtime. See `entry.h` for the shape and
 * `research/investigations/io-model/PARK-ON-MISS.md` §5 and §7 for the design
 * each part cites.
 *
 * What this unit is, in one line: the platform layer over `core.c`, carrying
 * the startup policy and the module ABI that `par_runtime.c` used to carry
 * over a second scheduler of its own.
 *
 * One implementation for every platform, with no `#if` of its own. Everything
 * that differs by host -- thread creation with a reserved host stack, the
 * thread's attach and detach for the switch, the machine's core count, the
 * atomics, the yield, the read of a setting's text -- is behind `prim.h`,
 * answered by `prim_host.c` on POSIX and `prim_windows.c` on Windows. `strtol`
 * is the C standard library on both and is used directly. */

#include "entry.h"
#include "prim.h"

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

/* The per-thread half of the floor's attach -- the alternate signal stack a
 * guard-page handler runs on, installed once at thread start (§7's
 * `wf__floor_attach_thread` bullet), or on Windows the emergency stack a
 * guarantee reserves -- is reached through `wf_prim_floor_attach`, primitive
 * P2's second name, exactly as the per-stack half is reached through
 * `wf_prim_set_bounds`. This unit names the floor's attach nowhere: the
 * platform leaf is the one unit that references it and the one that carries
 * its weak answer, because the Windows leaf has to arm every pool fiber
 * itself, one link admits one weak default per symbol (MSVC, LNK1227), and a
 * PE weak default satisfies only references from its own unit (GNU ld). */

/* The per-stack half is the bounds, and this unit reaches them through
 * `wf_prim_set_bounds` -- primitive 2's third name -- rather than through the
 * floor's own symbol.
 *
 * Two reasons, and the second is what settles it. The primitive is where the
 * bounds belong: the switch owns them and writes them from the reservation
 * record, so a caller outside the switch should be asking the same primitive
 * and not the floor. And a weak definition satisfies a reference from its own
 * translation unit on both platforms, but a PE weak external does not satisfy
 * an undefined reference from another object at all -- so a link without the
 * floor, which is every probe, does not link if this unit names the floor's
 * symbol directly. */

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

/* The one rule every startup setting of this runtime follows.
 *
 * Unset or empty means "this setting places no instruction", and each caller
 * reads its own default from that. A written value is an integer from 0
 * through the setting's own ceiling and nothing else: not a negative number,
 * not a number above the ceiling, and not text that is not a number. Anything
 * else is a configuration error rather than a value to repair, because
 * repairing it silently runs a different program from the one the caller asked
 * for -- a clamp turns "give me 4096 workers" into a pool the caller never
 * chose, and treating unparseable text as the sequential world turns a typo
 * into a measurement of the wrong thing.
 *
 * A configuration error ends the run here, before the program body starts,
 * with one line on the diagnostic channel, nothing on the output channel, and
 * a nonzero status. The ceiling in that line is spelled from the constant, so
 * a changed ceiling changes the message with it.
 *
 * Answers 1 and writes `*value` when the setting named a number, 0 when it is
 * unset or empty, and does not return otherwise.
 *
 * A setting whose text is longer than the buffer below reaches the same
 * diagnostic as any other text that is not a number in range, because that is
 * what it is: `prim.h`'s P4 refuses to truncate, and no number this rule
 * admits is anywhere near that long. */
int wf__sched_setting(
    const char *name,
    unsigned long ceiling,
    unsigned long *value
) {
    char text[WF_PRIM_SETTING_BYTES];
    int state = wf_prim_setting_text(name, text, sizeof(text));
    char *end = NULL;
    long written;
    if (state == 0 || (state == 1 && text[0] == '\0')) {
        return 0;
    }
    if (state == 1) {
        written = strtol(text, &end, 10);
        if (end != text && *end == '\0' && written >= 0
            && (unsigned long)written <= ceiling) {
            *value = (unsigned long)written;
            return 1;
        }
    }
    (void)fprintf(
        stderr,
        "whitefoot scheduler: %s must be an integer from 0 through %lu\n",
        name,
        ceiling
    );
    exit(1);
}

/* The ceiling of the completion runtime's own helper setting.
 *
 * WF_IO_HELPERS is the bridge's, and the bridge is the only unit that knows
 * how many helpers its storage holds; but the rule above says a bad setting
 * ends the run *before the program body*, and the bridge does not initialize
 * until the body's first operation. So the bridge answers this weakly-declared
 * question strongly, and the check below runs where every other one runs. A
 * link with no completion runtime has no such setting and answers zero. */
__attribute__((weak)) unsigned long wf__sched_helper_ceiling(void) {
    return 0;
}

/* What an unset WF_WORKERS asks for: this machine's logical CPUs, clamped to
 * the lane ceiling above.
 *
 * The count is `wf_prim_online_cpus`, which reports the cores this process may
 * be scheduled on right now -- `hw.logicalcpu` on Darwin, `_SC_NPROCESSORS_ONLN`
 * elsewhere, `GetActiveProcessorCount` on Windows. A machine that reports
 * fewer than two gets 0, which is what an explicit opt-out gets, because one
 * thread of execution is the sequential world either way.
 *
 * Naming every core rather than only the performance ones is a measured
 * choice, not an assumption: on the 4P+6E machine this was built on, the coarse
 * oracle configurations improve monotonically from 4 lanes to 10 (bal_d12_w192
 * 4.71x at 8 lanes to 5.16x at 10), and the fine ones do not fall below their
 * own sequential build there. An efficiency core that runs a stolen subtree at
 * a fraction of a performance core's rate still finishes it sooner than not
 * starting it. */
static int wf__sched_default_lanes(void) {
    unsigned online = wf_prim_online_cpus();
    if (online < 2u) {
        return 0;
    }
    return online > (unsigned)WF_PAR_MAX_LANES ? WF_PAR_MAX_LANES : (int)online;
}

/* The lane count this run asked for: 0 for the sequential world, or at least
 * two. A written 0 or 1 is the explicit opt-out, which is what the corpus and
 * every sequential reference build use. */
static int wf__sched_requested_lanes(void) {
    unsigned long requested = 0;
    if (!wf__sched_setting("WF_WORKERS", (unsigned long)WF_PAR_MAX_LANES, &requested)) {
        return wf__sched_default_lanes();
    }
    return requested < 2u ? 0 : (int)requested;
}

/* The stack count: WF_STACKS when it is written, otherwise the thread count
 * plus the spare above. The core raises it to its own floor of the thread
 * count plus one, so this only has to state a number. */
static unsigned wf__sched_stack_count(unsigned threads) {
    unsigned long written = 0;
    if (wf__sched_setting("WF_STACKS", (unsigned long)WF_SCHED_MAX_STACKS, &written)) {
        return (unsigned)written;
    }
    return threads + WF_SCHED_SPARE_STACKS;
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

/* One entry record per worker, indexed by the worker's own thread index.
 *
 * `wf_prim_thread_start` reads the entry and its argument from storage the
 * caller keeps alive for the life of the thread (`prim.h`, P1), and a worker's
 * life is the process's, so this is the storage: it is written once by the
 * creating thread before the worker exists and is never written again. Index 0
 * is the entry thread's and stays unused, which keeps the index the worker's
 * own rather than a second numbering to keep in step. */
static wf_prim_thread wf__sched_worker_threads[WF_SCHED_MAX_THREADS];

/* The rendezvous `par_runtime.c` made and this keeps: "the pool started" means
 * "the pool can take work". A pool whose threads are still being created
 * cannot take the first offers a program makes, and for a short program those
 * are all the offers there are; this costs one rendezvous per process.
 *
 * It is a counter and the yield, which are primitives 1 and 6, rather than a
 * condition variable of this unit's own. Two reasons, and the second is the
 * one that decides it: the wait is bounded by the creation of at most
 * WF_SCHED_MAX_THREADS threads and is taken once per process, so what it costs
 * is a few passes of a starting thread's scheduler; and a condition variable
 * here would be a second sleep mechanism in a design whose whole shape is one
 * (§6), owned by neither the platform nor the core. There is no deadline in it
 * and no budget: it yields until the count it is waiting for is published. */
static unsigned wf__sched_ready;

static void wf__sched_ready_report(void) {
    for (;;) {
        unsigned seen = wf_prim_load_u(&wf__sched_ready, WF_PRIM_ACQUIRE);
        if (wf_prim_cas_u(
                &wf__sched_ready,
                &seen,
                seen + 1u,
                WF_PRIM_ACQ_REL,
                WF_PRIM_ACQUIRE
            )) {
            return;
        }
    }
}

static void wf__sched_ready_await(unsigned started) {
    while (wf_prim_load_u(&wf__sched_ready, WF_PRIM_ACQUIRE) < started) {
        wf_prim_yield();
    }
}

/* The once every start in this unit runs under, over primitives 1 and 6.
 *
 * `pthread_once` was what this unit used and Windows has `InitOnceExecuteOnce`,
 * which is two implementations of one thing; the compare-exchange below is one.
 * The fast path a hand-out pays for is the acquire load at the top, which is
 * what a hot lane acquisition sees on every call after the first. A thread that
 * finds the body running yields until the body's thread publishes DONE: no
 * deadline, no budget, and no second sleep mechanism. */
#define WF_SCHED_ONCE_IDLE 0u
#define WF_SCHED_ONCE_RUNNING 1u
#define WF_SCHED_ONCE_DONE 2u

void wf__sched_once(unsigned *state, void (*body)(void)) {
    if (wf_prim_load_u(state, WF_PRIM_ACQUIRE) == WF_SCHED_ONCE_DONE) {
        return;
    }
    for (;;) {
        unsigned seen = WF_SCHED_ONCE_IDLE;
        if (wf_prim_cas_u(
                state,
                &seen,
                WF_SCHED_ONCE_RUNNING,
                WF_PRIM_ACQ_REL,
                WF_PRIM_ACQUIRE
            )) {
            body();
            wf_prim_store_u(state, WF_SCHED_ONCE_DONE, WF_PRIM_RELEASE);
            return;
        }
        if (seen == WF_SCHED_ONCE_DONE) {
            return;
        }
        wf_prim_yield();
    }
}

int wf__sched_enter(unsigned thread, void (*entry)(void *), void *argument) {
    int status;
    /* A thread's host stack has to be switchable-to before its first switch
     * away from it (§7's platform item 3); on the host this is nothing. */
    wf_prim_thread_attach();
    wf__sched_on_core = 1;
    status = wf_sched_run(&wf__sched_core, thread, entry, argument);
    /* Only thread 0 arrives here, on its host stack (§5). The last switch
     * wrote a pool stack's bounds into this thread's floor state, and this
     * thread is no longer on that stack, so the host stack's own bounds go
     * back before anything runs here. */
    wf__sched_on_core = 0;
    wf_prim_set_bounds(NULL, NULL);
    wf_prim_thread_detach();
    return status;
}

wf_sched_stack *wf__sched_current_stack(void) {
    if (!wf__sched_on_core) {
        return NULL;
    }
    return wf_sched_current_stack(&wf__sched_core);
}

unsigned wf__sched_pool_running(void) {
    return wf_prim_load_u(&wf__sched_workers, WF_PRIM_ACQUIRE);
}

static wf_sched_lane *wf__sched_current_lane(void) {
    if (!wf__sched_on_core) {
        return NULL;
    }
    return wf_sched_current_thread(&wf__sched_core)->lane;
}

/* ------------------------------------------------------- the workers */

static void wf__sched_worker_main(void *argument) {
    unsigned index = (unsigned)(uintptr_t)argument;
    /* The floor's per-thread half, before anything else: a worker runs
     * ordinary Whitefoot calls on a pool stack and can recurse exactly as deep
     * as the offering thread would have, so it needs the same alternate signal
     * stack. The bounds it captures here are this thread's *host* stack, which
     * is where the handler runs until the first switch and where nothing
     * Whitefoot ever runs (§5). */
    wf_prim_floor_attach();
    wf__sched_ready_report();
    /* Never returns: a worker's scheduler loop is the program's, and the
     * thread is detached. */
    (void)wf__sched_enter(index, NULL, NULL);
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
 * whole difference, so the pool starts under the once above, inside the first
 * lane acquisition, as `par_runtime.c` started it. `wf__par_pool_active` still
 * answers from the setting, so the module's choice between its two lowerings
 * does not depend on whether the pool has started yet.
 *
 * A refused stack reservation is a silent downgrade to the platform default,
 * which is exactly the hazard the thread primitive exists to close, so it
 * reports the failure instead: no lane is better than a lane that overflows.
 * A thread that cannot be created stops the loop at the count that did start;
 * the lanes past it stay empty, which costs a steal pass and nothing else. */
static void wf__sched_start_workers(unsigned threads) {
    unsigned index;
    unsigned started = 0;
    for (index = 1u; index < threads; index += 1u) {
        /* The worker's pool stack is taken here, on the thread that creates
         * it: its own start may come arbitrarily late, after the entry thread
         * has parked every free stack, and a thread's first act is not a join
         * and has no rule to fall to (§5, and the enumerator's sixth finding). */
        if (wf_sched_start_thread(&wf__sched_core, index) != 0) {
            break;
        }
        if (wf_prim_thread_start(
                &wf__sched_worker_threads[index],
                wf__sched_worker_main,
                (void *)(uintptr_t)index,
                wf__floor_stack_bytes()
            ) != 0) {
            break;
        }
        started += 1u;
    }
    wf_prim_store_u(&wf__sched_workers, started, WF_PRIM_RELEASE);
    wf__sched_ready_await(started);
}

/* The thread count the core was initialized with, which is what the first
 * hand-out starts. One until `wf__sched_start` has run, so a hand-out on a
 * link whose core never started (there is none: every such link refuses the
 * hand-out first, above) would start nothing. */
static unsigned wf__sched_threads = 1u;
static unsigned wf__sched_workers_once;

static void wf__sched_start_workers_once(void) {
    if (wf__sched_threads >= 2u) {
        wf__sched_start_workers(wf__sched_threads);
    }
}

/* --------------------------------------------------------- the entry */

int wf__sched_start(void) {
    int requested;
    unsigned threads;
    unsigned long helpers = 0;
    unsigned long ceiling = wf__sched_helper_ceiling();
    /* Every startup setting this process reads is checked here, at the core's
     * entry, so a configuration error ends the run before the program body
     * starts whichever unit the setting belongs to. The completion runtime's
     * own helper setting is checked through the ceiling it answers, and its
     * value is read again where it is used. */
    if (ceiling != 0u) {
        (void)wf__sched_setting("WF_IO_HELPERS", ceiling, &helpers);
    }
    requested = wf__sched_requested_lanes();
    threads = requested >= 2 ? (unsigned)requested : 1u;
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
    wf__sched_once(&wf__sched_workers_once, wf__sched_start_workers_once);
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
#define WF_SCHED_LANES_UNSETTLED (~0u)

static unsigned wf__sched_cached_lanes = WF_SCHED_LANES_UNSETTLED;

static int wf__sched_lanes_once(void) {
    unsigned lanes = wf_prim_load_u(&wf__sched_cached_lanes, WF_PRIM_RELAXED);
    if (lanes == WF_SCHED_LANES_UNSETTLED) {
        lanes = (unsigned)wf__sched_requested_lanes();
        wf_prim_store_u(&wf__sched_cached_lanes, lanes, WF_PRIM_RELAXED);
    }
    return (int)lanes;
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
