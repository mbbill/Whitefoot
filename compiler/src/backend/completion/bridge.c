#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* F_NOCACHE, the Darwin half of the WF_IO_NOCACHE target policy applied by the
 * inline helper in file_adapter.h, is a BSD extension the POSIX macro above
 * hides, so the Darwin namespace is asked for as well. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

/*
 * Compiler-owned finite file-completion bridge.
 *
 * The emitted module can submit only the typed open/read/write/status/close
 * and directory descriptors below. There is deliberately no callback,
 * function pointer, or generic thunk in this ABI: a file helper can execute
 * only file_adapter.c's closed switch.
 *
 * Every submit ends in a published record.  The record is a block of the
 * submitting frame, so there is nothing to claim and nothing to refuse: the
 * bridge either hands the record to the ring, or queues it on the bounded
 * POSIX adapter, or executes the operation here and completes the record
 * itself.  All three return 1 and the join reads the record
 * (`research/investigations/io-model/PARK-ON-MISS.md` §7).
 *
 * A weak LLVM fallback makes an emitted module independently linkable.
 */

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
#include "writer_scheduler.h"
#include "../sched/core.h"
#include "../sched/prim.h"
#if defined(__linux__)
#include "linux_io_uring.h"
#endif

#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define WF_BRIDGE_MAX_HELPERS 8u
/* Private storage one loop may hold for its in-flight iterations, before the
 * compiler's own ceiling and the loop's per-iteration size are applied.  It
 * exists so a loop whose iteration owns a large buffer gets a small window
 * instead of a large multiple of that buffer: at 64 KiB an iteration this
 * budget affords 64 of them, and at 16 MiB it affords none, which the K >= 1
 * floor turns into the sequential program. */
#define WF_BRIDGE_WINDOW_BYTE_BUDGET (4u * 1024u * 1024u)
/* The runtime's own answer when no argument of `wf__completion_window` bounds
 * it and the ring is the engine.
 *
 * It was half the process-wide operation capacity, because a loop that owned
 * every record would push every other operation in the program onto the
 * capacity-wait path.  There is no operation capacity any more and no capacity
 * wait to be pushed onto, so this is now a throughput choice of its own.  It
 * keeps the number the old rule produced -- half of 64 -- which is also where
 * the hand-written ceiling program measured its optimum (LOOP-PIPELINE.md
 * §9.2), so the change moves no measurement. */
#define WF_BRIDGE_WINDOW_DEFAULT 32u
/* The pool stacks the scheduler core reserves at the bridge's start.
 *
 * Nothing runs on one in this step: the core is linked for its record
 * protocol -- `wf_sched_record_init`, `wf_sched_complete`, and the in-place
 * waiter of §2's fourth line -- and no code here calls `wf_sched_run`.  The
 * count is the floor `wf_sched_init` enforces for one thread (design §5), and
 * the size is the smoke build's. */
#define WF_BRIDGE_SCHED_THREADS 1u
#define WF_BRIDGE_SCHED_STACKS 2u
#define WF_BRIDGE_SCHED_STACK_BYTES (256u * 1024u)

static wf_completion_runtime wf_bridge_runtime;
static wf_sched_core wf_bridge_core;
static wf_file_adapter wf_bridge_adapter;
static pthread_t wf_bridge_helpers[WF_BRIDGE_MAX_HELPERS];
static pthread_once_t wf_bridge_once = PTHREAD_ONCE_INIT;
static pthread_once_t wf_bridge_file_once = PTHREAD_ONCE_INIT;
static int wf_bridge_error;
static int wf_bridge_file_error;
/* The readiness flags below are atomic, and that is not decoration.
 *
 * Each is written by one of the once-initializers and read by entry points
 * that run no once at all: `wf__completion_file_pread_submit` asks whether the
 * pool is pinned and whether the adapter exists *before* it reaches the
 * routing decision, and the statistics entries read them from wherever a
 * program calls them.  A program with two threads therefore has an ordinary
 * unsynchronized read of a flag another thread is writing, which is a data
 * race whatever the values happen to be.  Declaring them `_Atomic` makes the
 * plain reads and writes below sequentially consistent accesses, which on the
 * hosts this runs on is at worst a load-acquire instruction and on x86-64 an
 * ordinary load.
 *
 * Whether WF_IO_HELPERS named the pool, which pins the route as well as the
 * count: see wf_bridge_helper_policy. */
static _Atomic int wf_bridge_helpers_pinned;
static _Atomic unsigned wf_bridge_ready;
static _Atomic unsigned wf_bridge_file_ready;
/* Records completed, and of those the ones the engine executed inside the
 * submitting call itself. */
static _Atomic uint64_t wf_bridge_publications;
static _Atomic uint64_t wf_bridge_inline_executions;
#if defined(__linux__)
static wf_linux_io_uring_adapter wf_bridge_linux_adapter;
static _Atomic unsigned wf_bridge_linux_ready;
/* The one piece of bridge readiness a thread may observe without running the
 * initializer itself.  `wf_bridge_flush_target` must not create a ring for a
 * program that only ever makes direct calls, so it cannot go through
 * `pthread_once`; this release/acquire pair is what orders the ring's
 * construction before another thread's flush of it. */
static _Atomic unsigned wf_bridge_doorbell_ready;
#endif

static int wf_bridge_progress(void);
static void wf_bridge_park(uint64_t observed_epoch);

/* The helper policy, in one place.
 *
 * A written WF_IO_HELPERS pins the count: `*initial` and `*cap` are both the
 * written value, so a program asked for four helpers gets four and never a
 * fifth, and a program asked for none keeps the zero-helper path where a
 * waiting thread is itself the target engine.
 *
 * Unset asks for demand-driven growth from one. One helper is the right
 * answer for a program with a single operation outstanding, and it was the
 * old fixed default; it is the wrong answer for a program that exposes real
 * width, because the submitting thread then waits on a queue only one
 * thread is draining. On the four-wide many-file workload the fixed default
 * finished 1.6x slower than the same program at four helpers, so growth is
 * bounded by the machine rather than pinned at one. */
static void wf_bridge_helper_policy(size_t *initial, size_t *cap) {
    const char *text = getenv("WF_IO_HELPERS");
    char *end = NULL;
    unsigned long parsed;
    if (text != NULL && *text != 0) {
        errno = 0;
        parsed = strtoul(text, &end, 10);
        if (errno == 0 && end != text && *end == 0) {
            if (parsed > WF_BRIDGE_MAX_HELPERS) {
                parsed = WF_BRIDGE_MAX_HELPERS;
            }
            *initial = (size_t)parsed;
            *cap = (size_t)parsed;
            /* A written setting is an instruction about how to run, so the
             * runtime stops choosing: it pins the pool exactly and keeps every
             * admitted operation on the queued completion path.  That is what
             * makes a pinned line of a measurement a measurement of the
             * completion path rather than of the policy that may run an
             * operation inline instead. */
            wf_bridge_helpers_pinned = 1;
            return;
        }
    }
#if defined(__linux__)
    /* A ready ring already carries every transfer without a thread handoff, so
     * a helper can only serve the operations the ring does not take — today,
     * open and close. Those are exactly the operations a warm page cache
     * answers immediately, and handing one to a helper costs a wake and a
     * publication to save nothing. Measured on the four-wide many-file
     * workload with a warm cache: 115 ms at zero helpers against 171 ms at
     * one, two, or four, the whole difference being about 7 us per open. So a
     * host with a native completion path starts with none, and the waiting
     * thread runs those operations itself. WF_IO_HELPERS still pins a pool
     * for a target whose opens really do wait. */
    if (wf_bridge_linux_ready != 0) {
        *initial = 0u;
        *cap = 0u;
        return;
    }
#endif
    /* No helper until the adapter has measured an operation that waits.
     *
     * A helper exists to overlap a wait.  Starting with one and growing on
     * queue depth alone gave every program a thread handoff whether or not it
     * had anything to overlap, and on a warm page cache — where a read is a
     * memory copy — that handoff was the whole cost of the completion path.
     * With none, a waiting thread is the queue's own engine and the path
     * costs a queue crossing; the adapter's growth rule adds the first helper
     * as soon as the operations it has executed show a real wait.
     *
     * The ceiling is the bridge's own, not the machine's core count.  A helper
     * inside a host call holds no CPU, so what bounds useful I/O concurrency
     * here is how many operations a program can have outstanding.  Sizing the
     * pool by cores capped a three-core host at three outstanding reads for a
     * program that stated eight, which is a device left idle rather than a
     * machine kept busy. */
    *cap = WF_BRIDGE_MAX_HELPERS;
    *initial = 0u;
}

static int wf_bridge_target_progress_one(void) {
    return wf_bridge_file_ready != 0
        && wf_file_adapter_progress(&wf_bridge_adapter, 1u) != 0;
}

/* One newly queued request is announced once, under the queue lock the
 * enqueue already holds, and reaches exactly one helper.  With zero helpers a
 * sleeping thread is itself the target's engine, so the same announcement
 * has to reach the wake epoch as well. */
static void wf_bridge_notify_target(void) {
    wf_completion_notify_target(&wf_bridge_runtime);
}

static void wf_bridge_initialize_file(void) {
    size_t requested = 0;
    size_t cap = 0;
    wf_bridge_helper_policy(&requested, &cap);
    wf_bridge_file_error = wf_file_adapter_init(
        &wf_bridge_adapter,
        &wf_bridge_runtime,
        wf_bridge_helpers,
        WF_BRIDGE_MAX_HELPERS,
        requested
    );
    if (wf_bridge_file_error == 0
        && wf_file_adapter_set_helper_cap(&wf_bridge_adapter, cap) == 0) {
        wf_bridge_file_ready = 1;
    }
}

static int wf_bridge_ensure_file(void) {
    return pthread_once(&wf_bridge_file_once, wf_bridge_initialize_file) == 0
        && wf_bridge_file_ready != 0;
}

static void wf_bridge_shutdown(void) {
    if (wf_bridge_ready == 0) {
        return;
    }
    if (wf_bridge_file_ready != 0) {
        (void)wf_file_adapter_shutdown(&wf_bridge_adapter);
        wf_bridge_file_ready = 0;
    }
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        atomic_store_explicit(
            &wf_bridge_doorbell_ready,
            0,
            memory_order_release
        );
        (void)wf_linux_io_uring_destroy(&wf_bridge_linux_adapter);
        wf_bridge_linux_ready = 0;
    }
#endif
    (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
    wf_bridge_ready = 0;
}

#if defined(__linux__)
/* WF_IO_NO_NATIVE_RING: run this Linux process on the POSIX adapter route.
 *
 * Runtime policy of the same class as WF_IO_NOCACHE and WF_IO_HELPERS, and
 * test-only within that class: no Whitefoot source names it, no accepted
 * program changes meaning under it, and no byte any operation produces
 * differs with it set.  Absent -- and any value other than the exact text "1"
 * -- is today's behaviour exactly.
 *
 * It exists because the route a Linux host takes is not a choice a test can
 * otherwise make.  A kernel with io_uring available takes every positioned
 * read into the ring, so the adapter branch of `bridge_default_probe`'s route
 * assertion cannot fire there at all, and the negative control it provides was
 * carried by the Darwin CI host alone.  Skipping the ring init reaches the
 * same state a kernel without io_uring reaches, which is a path the runtime
 * already has rather than one this setting adds.
 *
 * It is read once, here, before the only call that starts the ring. */
static int wf_bridge_native_ring_refused(void) {
    const char *text = getenv("WF_IO_NO_NATIVE_RING");
    return text != NULL && text[0] == '1' && text[1] == 0;
}
#endif

static void wf_bridge_initialize(void) {
    wf_bridge_error = wf_completion_runtime_init(&wf_bridge_runtime);
    if (wf_bridge_error != 0) {
        return;
    }
    if (wf_sched_init(
            &wf_bridge_core,
            WF_BRIDGE_SCHED_THREADS,
            WF_BRIDGE_SCHED_STACKS,
            WF_BRIDGE_SCHED_STACK_BYTES
        ) != 0) {
        wf_bridge_error = ENOMEM;
        (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
        return;
    }
#if defined(__linux__)
    if (!wf_bridge_native_ring_refused()
        && wf_linux_io_uring_init(
            &wf_bridge_linux_adapter,
            &wf_bridge_runtime,
            WF_LINUX_IO_URING_DEPTH
        ) == 0) {
        if (wf_completion_set_wake_callback(
                &wf_bridge_runtime,
                wf_linux_io_uring_notify,
                &wf_bridge_linux_adapter
            ) == 0) {
            wf_bridge_linux_ready = 1;
        } else {
            (void)wf_linux_io_uring_destroy(&wf_bridge_linux_adapter);
        }
    }
#else
    /* Darwin has no regular-file kernel completion facility in the supported
     * target set, so its qualified path is the bounded typed adapter. */
    if (!wf_bridge_ensure_file()) {
        (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
        return;
    }
#endif
    wf_bridge_ready = 1;
#if defined(__linux__)
    atomic_store_explicit(
        &wf_bridge_doorbell_ready,
        wf_bridge_linux_ready,
        memory_order_release
    );
#endif
    if (atexit(wf_bridge_shutdown) != 0) {
        /* Registration failure changes cleanup at process exit, not the
         * completion contract of any admitted operation. */
    }
}

/* The bridge, or nothing.
 *
 * A bridge that cannot initialize leaves no engine to run the operation and
 * no drain to publish it, so it is a trusted-computing-base failure and
 * terminates deterministically where the floor does.  It is not an operation
 * outcome and there is no arm left to fall to (design §7). */
static void wf_bridge_require(void) {
    if (pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0) {
        abort();
    }
}

/* -------------------------------------------------- the one publication */

/* Stores DONE into the record and wakes whoever waits on it.
 *
 * There is exactly one call of this per submission, made by whichever engine
 * finished the operation: the CQE reaper, a helper thread, or the submitting
 * thread itself.  The record is still there for it because the emitter joins
 * every outstanding operation before any terminator, so no exit edge and
 * therefore no frame teardown precedes the join that consumes the terminal
 * (design §7, "Exactly one terminal completion per submission").
 *
 * `wf_sched_complete` stores COMPLETING, claims the waiter, stores DONE, and
 * touches the record no further -- which is what lets the joiner's frame die
 * the moment DONE is read. */
void wf_completion_record_complete(wf_completion_record *record) {
    atomic_fetch_add_explicit(
        &wf_bridge_publications,
        1,
        memory_order_relaxed
    );
    wf_sched_complete(&wf_bridge_core, &record->sched);
}

/* --------------------------------- the one wait and wake (§7, platform 2) */

/* The scheduler core sleeps and wakes on one primitive, and on a completion
 * link that primitive is the bridge's, not `prim_host.c`'s own epoch.  These
 * three definitions are the mapping, stated here once:
 *
 *   epoch  -> `wf_completion_wake_epoch`, the runtime's own wake epoch, which
 *             the Linux ring's park also announces itself against;
 *   park   -> `wf_bridge_park`, which is the io_uring epoll wait over the CQ
 *             and the wake eventfd where a ring exists, and the runtime's
 *             condition-variable park everywhere else;
 *   wake   -> `wf_completion_notify_target`, which raises that epoch and, when
 *             a sleeper has announced itself, calls the installed wake
 *             callback -- `wf_linux_io_uring_notify`, the eventfd write the
 *             ring's park is waiting on -- before signalling the condition
 *             variable.  So one call rings the ring's wake descriptor and the
 *             condvar both, and the two routes agree in this one place.
 *
 * `prim_host.c` declares all three weak, so a core linked without a
 * completion runtime -- the smoke build, the enumerator -- keeps its own
 * epoch, and a completion link gets these. */
int wf__sched_host_epoch(uint64_t *epoch) {
    if (wf_bridge_ready == 0) {
        return 0;
    }
    *epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
    return 1;
}

int wf__sched_host_park(uint64_t observed) {
    if (wf_bridge_ready == 0) {
        return 0;
    }
    wf_bridge_park(observed);
    return 1;
}

int wf__sched_host_wake(void) {
    if (wf_bridge_ready == 0) {
        return 0;
    }
    wf_completion_notify_target(&wf_bridge_runtime);
    return 1;
}

/* Primitive 7 of §7.1: target progress and the drain, exported once where the
 * bridge has statics.  Nothing calls the scheduler loop in this step, so this
 * is the core's view of the same pass the in-place join arm runs. */
int wf__sched_target_progress(struct wf_sched_core *core) {
    (void)core;
    return wf_bridge_progress();
}

/* --------------------------------------------------------- target progress */

/* Rings the deferred io_uring doorbell before this thread does something the
 * ring cannot see through.
 *
 * Staging an SQE costs no syscall, so a submission leaves work the kernel has
 * not been told about.  That is exactly what makes deferring worth 15 % of the
 * eight-wide benchmark's wall time, and exactly what makes an unguarded
 * blocking call a hazard: an open the program has already submitted would sit
 * in the submission queue, untouched, for as long as this thread waits in a
 * direct `openat`.  Every entry point below that can block outside the ring
 * flushes first.  On a target with no ring this is nothing. */
static void wf_bridge_flush_target(void) {
#if defined(__linux__)
    if (atomic_load_explicit(&wf_bridge_doorbell_ready, memory_order_acquire)
        != 0) {
        (void)wf_linux_io_uring_flush(&wf_bridge_linux_adapter);
    }
#endif
}

/* Flush the deferred doorbell, reap the ring, and run one queued request when
 * this thread is the queue's only engine.  Returns nonzero when it moved
 * something.
 *
 * This is primitive 7 (§7.1) and it may block: with no helper the bounded
 * pass executes a queued host `open` or `close` on the calling thread.  That
 * is now the only place a host call is made for a queued operation. */
static int wf_bridge_progress(void) {
    int progressed = 0;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        size_t published = 0;
        int error = wf_linux_io_uring_progress(
            &wf_bridge_linux_adapter,
            1u,
            0,
            &published
        );
        if (error != 0) {
            /* Target ownership has already transferred. Falling back now
             * would duplicate an operation, and ignoring the error would
             * strand its owned operation forever. A target-runtime failure is a
             * fail-stop TCB defect, not a writer-visible IoError. */
            abort();
        }
        progressed |= published != 0;
    }
#endif
    /* A thread executes a queued request only when no target helper exists.
     * With helpers, taking an unrelated request here could block the exact
     * frame which is waiting on a completion they have already published. */
    if (wf_file_adapter_helper_count(&wf_bridge_adapter) == 0) {
        progressed |= wf_bridge_target_progress_one();
    }
    return progressed;
}

static void wf_bridge_park(uint64_t observed_epoch) {
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        int error = wf_linux_io_uring_park(
            &wf_bridge_linux_adapter,
            observed_epoch,
            UINT32_MAX
        );
        if (error != 0) {
            abort();
        }
        /* Reap the CQ first; the caller's next turn re-reads its record. */
        (void)wf_bridge_progress();
        return;
    }
#endif
    {
        enum wf_completion_park_result parked =
            wf_completion_park_if_unchanged(
                &wf_bridge_runtime,
                observed_epoch,
                UINT32_MAX
            );
        if (parked == WF_COMPLETION_PARK_FAILED) {
            abort();
        }
    }
}

void wf__writer_scheduler_notify(void) {
    if (wf_bridge_ready != 0) {
        wf_completion_notify_compute(&wf_bridge_runtime);
    }
}

/* ------------------------------------------------------------- the join */

/* How long a joining thread looks at its own record before announcing sleep.
 *
 * Announcing sleep and being woken is two system calls, paid by the waiter and
 * by whichever thread publishes.  A helper pool only exists when the adapter
 * measured operations that wait, and those waits end while this thread has
 * nothing else to do, so a bounded look before sleeping trades a little idle
 * CPU for both of those calls.  It is a bound on wasted CPU: a wait longer
 * than this window still ends in a sleep, and one shorter than it never
 * becomes a pair of system calls.
 *
 * It watches the one record this thread is waiting on, where it used to watch
 * a process-wide count of ready events that no longer exists (design §7: a
 * completion is published straight into its record). */
#define WF_BRIDGE_JOIN_SPIN_NS 10000u

/* Zero means the clock could not be read.  A monotonic clock that answers
 * exactly zero is the same answer as a broken one for every use here, and both
 * have to end a bounded wait rather than extend it. */
static uint64_t wf_bridge_monotonic_ns(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return 0;
    }
    return (uint64_t)now.tv_sec * 1000000000u + (uint64_t)now.tv_nsec;
}

static unsigned wf_bridge_record_state(const wf_completion_record *record) {
    return wf_prim_load_u(&record->sched.state, WF_PRIM_ACQUIRE);
}

/* Waits a bounded time for this record to be completed by someone else.
 * Returns 1 when it was, so the caller re-reads it instead of parking. */
static int wf_bridge_spin_for_completion(const wf_completion_record *record) {
    uint64_t started = wf_bridge_monotonic_ns();
    uint64_t deadline;
    unsigned turn = 0;
    /* A clock this thread cannot read is bounded by the `now == 0` term of
     * the periodic sample below, not by this early return.  That sample
     * treats a zero reading exactly as it treats a passed deadline, so a
     * failed clock ends the spin within 64 turns whatever happens here. */
    if (started == 0) {
        return wf_bridge_record_state(record) != WF_SCHED_PENDING;
    }
    deadline = started + WF_BRIDGE_JOIN_SPIN_NS;
    for (;;) {
        if (wf_bridge_record_state(record) != WF_SCHED_PENDING) {
            return 1;
        }
        turn += 1;
        if ((turn & 0x3fu) == 0) {
            uint64_t now = wf_bridge_monotonic_ns();
            if (now == 0 || now >= deadline) {
                return 0;
            }
        }
    }
}

/* The I/O arm of §2's fourth line: wait in place on the record, with nothing
 * running above this join.
 *
 * It is the scheduler core's own protocol with no stack to park (§6): read the
 * record's state; yield through COMPLETING, which is DONE a few instructions
 * away; make one pass of target progress; and only then register this thread
 * as the record's in-place waiter, capture the wake epoch, re-check the state
 * -- which catches a publisher that stored DONE before the registration -- and
 * sleep on the one primitive.  A publisher that claims the registration calls
 * `wf_prim_wake`, which on a completion link is the bridge's own wake, so the
 * sleep ends.  The marker is taken back on every exit from the park; a failed
 * compare-exchange there means the publisher took it and DONE follows.
 *
 * The registration goes up before the epoch is captured and before the last
 * look at the state, so a publisher that misses the marker is one whose DONE
 * that look finds. */
/* Runs the waiting thread's own submission here, if it is still queued.
 *
 * A thread that has submitted an operation and is now waiting for exactly that
 * operation runs it rather than waiting behind whatever the helpers are
 * already inside.  It is the one queue visit a joining thread may make while
 * helpers exist, and it takes nothing that belongs to another frame: the
 * record is this frame's own and this thread has nothing else to do until it
 * is DONE (design §7).  The host call it then makes is a blocking one, so the
 * doorbell rings before it, exactly as it does for every other host call this
 * bridge makes on a caller's thread. */
static int wf_bridge_run_own(wf_completion_record *record) {
    if (wf_bridge_file_ready == 0
        || !wf_file_adapter_claim_own(&wf_bridge_adapter, record)) {
        return 0;
    }
    /* The host call below is outside every path the ring can see through. */
    wf_bridge_flush_target();
    wf_file_adapter_run_claimed(&wf_bridge_adapter, record);
    return 1;
}

static void wf_bridge_wait_in_place(wf_completion_record *record) {
    for (;;) {
        unsigned state = wf_bridge_record_state(record);
        uint64_t epoch;
        void *expected;
        void *marker;
        if (state == WF_SCHED_DONE) {
            return;
        }
        if (state == WF_SCHED_COMPLETING) {
            wf_prim_yield();
            continue;
        }
        if (wf_bridge_run_own(record)) {
            continue;
        }
        if (wf_bridge_progress()) {
            continue;
        }
        expected = NULL;
        (void)wf_prim_cas_p(
            (void **)&record->sched.waiter,
            &expected,
            WF_SCHED_WAITER_IN_PLACE,
            WF_PRIM_SEQ_CST,
            WF_PRIM_SEQ_CST
        );
        epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        if (wf_bridge_record_state(record) == WF_SCHED_PENDING
            && !wf_bridge_spin_for_completion(record)) {
            wf_bridge_park(epoch);
        }
        marker = WF_SCHED_WAITER_IN_PLACE;
        (void)wf_prim_cas_p(
            (void **)&record->sched.waiter,
            &marker,
            NULL,
            WF_PRIM_SEQ_CST,
            WF_PRIM_SEQ_CST
        );
    }
}

static wf_completion_record *wf_bridge_record_of(const void *record) {
    if (record == NULL) {
        abort();
    }
    return (wf_completion_record *)(uintptr_t)record;
}

void wf__completion_file_join(
    const void *record,
    int64_t *value,
    int *error_code
) {
    wf_completion_record *held = wf_bridge_record_of(record);
    if (value == NULL || error_code == NULL) {
        abort();
    }
    wf_bridge_wait_in_place(held);
    *value = held->result.value;
    *error_code = held->result.error_code;
}

void wf__completion_file_open_join(
    const void *record,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
) {
    wf_completion_record *held = wf_bridge_record_of(record);
    if (value == NULL || error_code == NULL || open_outcome == NULL) {
        abort();
    }
    wf_bridge_wait_in_place(held);
    if (held->result.kind != WF_FILE_OPEN_AT) {
        abort();
    }
    *value = held->result.value;
    *error_code = held->result.error_code;
    *open_outcome = (unsigned)held->result.open_outcome;
}

/* The status join copies nothing: the engine already wrote the bytes into the
 * destination the submit named, and the record carries how many (design §7). */
void wf__completion_file_status_join(
    const void *record,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
) {
    wf_completion_record *held = wf_bridge_record_of(record);
    if (value == NULL || error_code == NULL || status == NULL
        || status_size == NULL
        || (uint64_t)(size_t)status_capacity != status_capacity) {
        abort();
    }
    wf_bridge_wait_in_place(held);
    if (held->result.kind != WF_FILE_STATUS
        || held->request.operation.status.destination != status
        || held->request.operation.status.capacity != (size_t)status_capacity) {
        abort();
    }
    *value = held->result.value;
    *error_code = held->result.error_code;
    *status_size = (uint64_t)held->status_written;
}

/* ----------------------------------------------------------- the submits */

/* Whether the operation has no external action at all, because its transfer
 * range is empty.
 *
 * The emitted code no longer holds this back: with one lowering per operation
 * an empty range is submitted like any other and the runtime completes it, so
 * every kind that carries a byte count answers here -- directory enumeration
 * included, whose host facility refuses a zero-sized batch rather than
 * reporting an empty one (design section 8, "One lowering for every I/O
 * operation"). */
static int wf_bridge_file_request_is_empty(const wf_file_request *request) {
    switch (request->kind) {
        case WF_FILE_READ:
            return request->operation.read.count == 0;
        case WF_FILE_WRITE:
            return request->operation.write.count == 0;
        case WF_FILE_PREAD:
            return request->operation.pread.count == 0;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
        case WF_FILE_DIRECTORY_NEXT:
            return request->operation.directory_next.count == 0;
#endif
        default:
            return 0;
    }
}

/* Executes the record's operation here and publishes it.
 *
 * This is where an operation with no kernel completion form ends up, and it
 * is the runtime's engine running the operation rather than a path an emitted
 * program can take (design §7.1, primitive 7).  The blocking host call is the
 * one the deleted direct family used to make on the caller's behalf; what
 * changed is that the record is published at the end, so every submit path
 * ends in a published record (design §7).  A refusal the host gives it,
 * including an open that found no descriptor, is the outcome the program
 * sees: the descriptor an open needs is owned by its `FilePermit` [SYS-10],
 * drawn from a factory whose capacity is inside what the host provides, so a
 * refusal here is a genuine exhaustion and not a schedule's invention. */
static void wf_bridge_execute_here(wf_completion_record *record) {
    wf_file_result result;
    record->route = WF_COMPLETION_ROUTE_INLINE;
    atomic_fetch_add_explicit(
        &wf_bridge_inline_executions,
        1,
        memory_order_relaxed
    );
    /* The host call below is outside every path the ring can see through. */
    wf_bridge_flush_target();
    result = wf_file_execute_timed(&wf_bridge_adapter, &record->request);
    wf_file_complete_record(record, &result);
}

/* An empty transfer has no external action at all, so there is nothing to
 * execute and nothing to overlap: the record is completed here. */
static void wf_bridge_complete_empty(wf_completion_record *record) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.head.kind = record->request.kind;
    result.head.value = 0;
    record->route = WF_COMPLETION_ROUTE_INLINE;
    atomic_fetch_add_explicit(
        &wf_bridge_inline_executions,
        1,
        memory_order_relaxed
    );
    wf_file_complete_record(record, &result);
}

/* Queues the record on the bounded POSIX adapter, or executes it here when
 * that adapter cannot be built.  Either way the record is the runtime's. */
static void wf_bridge_submit_file(wf_completion_record *record) {
    if (!wf_bridge_ensure_file()) {
        wf_bridge_execute_here(record);
        return;
    }
    if (wf_file_adapter_submit(&wf_bridge_adapter, record)
        != WF_FILE_TARGET_OWNS) {
        /* The shape was checked before the record was filled, so the only
         * answer left is an adapter that has stopped admitting -- the process
         * is exiting -- and the operation is executed here instead of being
         * left unpublished. */
        wf_bridge_execute_here(record);
        return;
    }
    wf_bridge_notify_target();
}

/* Whether a positioned transfer is better made where it was stated than
 * queued.
 *
 * The completion path exists so that a program is not stalled by a wait it
 * could have overlapped.  When the bounded adapter holds no helper, has
 * nothing queued, and has measured its own operations as not waiting, there is
 * no wait to overlap and no other thread to overlap it on: the queued
 * operation would be executed by this very thread, at its join, after a queue
 * crossing.  Executing it here is the same host call without any of that, and
 * the record is published at the end either way, so this is a throughput
 * choice and never an outcome.
 *
 * Only a *positioned* transfer takes it, and that is the whole liveness
 * argument.  An offset is meaningful only on a seekable object, and the typed
 * opens that produce one admit nothing but a regular file, so a positioned
 * read waits on storage.  A non-positioned read or write may be waiting on
 * something another part of the same program has to do — a pipe the program
 * itself must drain — and running one where it was stated could stall the very
 * thread that would unblock it.  Those keep the queue.
 *
 * A written WF_IO_HELPERS takes nothing inline: it pins the route with the
 * count.
 *
 * The measurement keeps running while this is true, because every inline
 * execution is timed by the same adapter, so a program whose reads start
 * waiting is queueing again within a few operations. */
static int wf_bridge_positioned_read_runs_on_caller(uint64_t count) {
    return count != 0
        && wf_bridge_helpers_pinned == 0
        && wf_bridge_file_ready != 0
        && wf_file_adapter_transfer_runs_on_caller(&wf_bridge_adapter);
}

/* Fills the record's scheduler words and clears the engine state every route
 * reads.  Called once, by submit, before any engine can see the record. */
static wf_completion_record *wf_bridge_begin(void *record) {
    wf_completion_record *held;
    if (record == NULL) {
        abort();
    }
    wf_bridge_require();
    held = (wf_completion_record *)record;
    memset(held, 0, sizeof(*held));
    wf_sched_record_init(&held->sched);
    held->route = WF_COMPLETION_ROUTE_NONE;
    held->opened_descriptor = -1;
    held->open_outcome = WF_FILE_OPEN_SUCCEEDED;
    return held;
}

/* Hands the record to whichever engine can take it, in the one order every
 * submit uses: the ring where it has a form for this kind, then the bounded
 * adapter, and the engine here when neither applies.  Every path ends in a
 * record the runtime owns, so there is nothing to answer. */
static void wf_bridge_dispatch(wf_completion_record *record) {
    if (wf_bridge_file_request_is_empty(&record->request)) {
        wf_bridge_complete_empty(record);
        return;
    }
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0
        && wf_linux_io_uring_carries(record)) {
        enum wf_linux_io_uring_submit_result submitted =
            wf_linux_io_uring_submit(&wf_bridge_linux_adapter, record);
        if (submitted != WF_LINUX_IO_URING_TARGET_OWNS) {
            /* The kind and shape were both answered before the record was
             * offered, so any other answer is a target-runtime failure. */
            abort();
        }
        if (wf_linux_io_uring_progress_error(&wf_bridge_linux_adapter) != 0) {
            abort();
        }
        return;
    }
#endif
    wf_bridge_submit_file(record);
}

void wf__completion_file_read_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if ((buffer == NULL && count != 0) || (uint64_t)(size_t)count != count) {
        abort();
    }
    held->request.kind = WF_FILE_READ;
    held->request.operation.read.descriptor = descriptor;
    held->request.operation.read.buffer = buffer;
    held->request.operation.read.count = (size_t)count;
    wf_bridge_dispatch(held);
}

/* Publishes a refusal the host itself would have made, without asking it.
 *
 * A failed outcome and a terminated process are different things, and only
 * one of them is what an offset the target ABI cannot express deserves: the
 * writer may spell any `u64` offset, and the host answers an offset above
 * `INT64_MAX` with EINVAL.  The record is completed with exactly that answer,
 * so the emitted mapper builds the same failed outcome it built when this
 * shape still went to the direct wrapper (design section 8, "One lowering for
 * every I/O operation").  Nothing is executed, so the inline-execution count
 * is untouched; the publication count is not, because this is one record's
 * one terminal completion. */
static void wf_bridge_complete_refused(
    wf_completion_record *record,
    int error_code
) {
    record->route = WF_COMPLETION_ROUTE_INLINE;
    record->result.kind = record->request.kind;
    record->result.value = -1;
    record->result.error_code = error_code;
    wf_completion_record_complete(record);
}

void wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if ((buffer == NULL && count != 0) || (uint64_t)(size_t)count != count) {
        abort();
    }
    held->request.kind = WF_FILE_PREAD;
    held->request.operation.pread.descriptor = descriptor;
    held->request.operation.pread.buffer = buffer;
    held->request.operation.pread.count = (size_t)count;
    if (file_offset > (uint64_t)INT64_MAX) {
        wf_bridge_complete_refused(held, EINVAL);
        return;
    }
    held->request.operation.pread.offset = (int64_t)file_offset;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0 && wf_linux_io_uring_carries(held)) {
        wf_bridge_dispatch(held);
        return;
    }
#endif
    if (wf_bridge_positioned_read_runs_on_caller(count)) {
        wf_bridge_execute_here(held);
        return;
    }
    wf_bridge_dispatch(held);
}

void wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if ((buffer == NULL && count != 0) || (uint64_t)(size_t)count != count) {
        abort();
    }
    /* write_once is an unpositioned Output operation. The native adapter's
     * write request fixes an explicit offset, which would change regular
     * file-offset semantics and is not a meaningful stream offset. Until a
     * Linux request kind is qualified for exact write(2) current-position and
     * append/stream behavior, this stays on the bounded typed adapter, which
     * `wf_linux_io_uring_carries` answers by having no form for WF_FILE_WRITE. */
    held->request.kind = WF_FILE_WRITE;
    held->request.operation.write.descriptor = descriptor;
    held->request.operation.write.buffer = buffer;
    held->request.operation.write.count = (size_t)count;
    wf_bridge_dispatch(held);
}

void wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if (path == NULL || has_mode > 1u
        || expected_kind > WF_FILE_EXPECT_DIRECTORY) {
        abort();
    }
    /* The name is the submitting frame's own and stays live until the join,
     * so nothing is copied and no length can refuse the completion path
     * (design §5). */
    held->request.kind = WF_FILE_OPEN_AT;
    held->request.operation.open_at.directory = directory;
    held->request.operation.open_at.path = path;
    held->request.operation.open_at.flags = flags;
    held->request.operation.open_at.mode = mode;
    held->request.operation.open_at.has_mode = has_mode;
    held->request.operation.open_at.expected_kind =
        (enum wf_file_expected_kind)expected_kind;
    wf_bridge_dispatch(held);
}

void wf__completion_file_status_submit(
    int descriptor,
    void *status,
    uint64_t status_capacity,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if (status == NULL
        || (uint64_t)(size_t)status_capacity != status_capacity) {
        abort();
    }
    held->request.kind = WF_FILE_STATUS;
    held->request.operation.status.descriptor = descriptor;
    held->request.operation.status.destination = status;
    held->request.operation.status.capacity = (size_t)status_capacity;
    wf_bridge_dispatch(held);
}

void wf__completion_file_close_submit(
    int descriptor,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    held->request.kind = WF_FILE_CLOSE;
    held->request.operation.close.descriptor = descriptor;
    wf_bridge_dispatch(held);
}

void wf__completion_directory_next_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position,
    void *record
) {
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    wf_completion_record *held = wf_bridge_begin(record);
    if (position == NULL || (buffer == NULL && count != 0)
        || (uint64_t)(size_t)count != count) {
        abort();
    }
    held->request.kind = WF_FILE_DIRECTORY_NEXT;
    held->request.operation.directory_next.descriptor = descriptor;
    held->request.operation.directory_next.buffer = buffer;
    held->request.operation.directory_next.count = (size_t)count;
    held->request.operation.directory_next.position = position;
    wf_bridge_dispatch(held);
#else
    /* A family with no [QUAL-2] enumeration facility compiles no such request
     * kind, and `backend/qualification.rs` refuses the operation for that
     * target, so reaching this entry at all is a contract violation. */
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)position;
    (void)record;
    abort();
#endif
}

/* ------------------------------------------------------------ the window */

/* How many iterations of one loop the runtime will carry in flight at once.
 *
 * Asked once per loop entry and never per iteration, exactly as
 * `wf__par_split_budget` is.  The writer never sees this number, never spells
 * it, and cannot influence it: there is no attribute, no environment variable,
 * and no source form for a window.
 *
 * `span` is the loop's trip count where it is statically known and zero where
 * it is not; `slot_bytes` is the private storage one in-flight iteration owns;
 * `ceiling` is the compiler's own static cap from that storage's cost.  A zero
 * in any of the three means "this one places no bound", so
 * `wf__completion_window(0, 0, 0)` is the runtime's unconstrained answer.
 *
 * **One is always a legal answer**, and it reproduces the sequential program
 * exactly, so this query can never make a correct program fail.  That is why
 * the fallback a link without this unit gets returns one.
 *
 * Every term that read an operation capacity is gone with the capacity: the
 * record is a block of the submitting frame, so no number of in-flight
 * iterations can exhaust a pool, and the ring's depth is a throughput
 * parameter rather than a bound on operations in flight (design §7).  What is
 * left is the byte budget, the span, the compiler's ceiling, and — where a
 * bounded helper pool is the engine — the pool's width. */
uint64_t wf__completion_window(
    uint64_t span,
    uint64_t slot_bytes,
    uint64_t ceiling
) {
    uint64_t window;
    if (pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0) {
        /* With no completion runtime every operation is a blocking direct
         * call, so depth buys nothing and one is the honest answer. */
        return 1;
    }
    window = WF_BRIDGE_WINDOW_DEFAULT;
#if defined(__linux__)
    if (wf_bridge_linux_ready == 0
        && (uint64_t)WF_BRIDGE_MAX_HELPERS < window) {
        window = (uint64_t)WF_BRIDGE_MAX_HELPERS;
    }
#else
    if ((uint64_t)WF_BRIDGE_MAX_HELPERS < window) {
        window = (uint64_t)WF_BRIDGE_MAX_HELPERS;
    }
#endif
    if (ceiling != 0 && ceiling < window) {
        window = ceiling;
    }
    if (span != 0 && span < window) {
        window = span;
    }
    if (slot_bytes != 0) {
        uint64_t affordable =
            (uint64_t)WF_BRIDGE_WINDOW_BYTE_BUDGET / slot_bytes;
        if (affordable < window) {
            window = affordable;
        }
    }
    return window == 0 ? 1u : window;
}

void wf__writer_run_root(void *frame) {
    while (!wf__writer_is_done(frame)) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        int progressed = wf_bridge_progress();
        progressed |= wf__writer_scheduler_help_once();
        if (!progressed && !wf__writer_is_done(frame)) {
            wf_bridge_park(epoch);
        }
    }
}

/* ------------------------------------------------------- the statistics */

uint64_t wf__completion_file_submissions(void) {
    uint64_t submissions = wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter).submissions;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        submissions += wf_linux_io_uring_statistics_snapshot(
            &wf_bridge_linux_adapter
        ).submissions;
    }
#endif
    return submissions;
}

uint64_t wf__completion_file_fallback_submissions(void) {
    return wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter).submissions;
}

/* `io_uring_enter` calls this process made to carry staged submissions.  With
 * the doorbell deferred this stays far below the submission count, and the
 * distance between the two is what deferring bought. */
uint64_t wf__completion_linux_io_uring_submission_enters(void) {
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        return wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter)
            .submission_enters;
    }
#endif
    return 0;
}

uint64_t wf__completion_file_helper_executions(void) {
    return wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter)
            .helper_executions;
}

uint64_t wf__completion_target_helper_count(void) {
    return (uint64_t)wf_file_adapter_helper_count(&wf_bridge_adapter);
}

uint64_t wf__completion_target_helper_executions(void) {
    return wf__completion_file_helper_executions();
}

uint64_t wf__completion_publications(void) {
    return atomic_load_explicit(&wf_bridge_publications, memory_order_relaxed);
}

uint64_t wf__completion_inline_executions(void) {
    return atomic_load_explicit(
        &wf_bridge_inline_executions,
        memory_order_relaxed
    );
}

uint64_t wf__completion_linux_io_uring_submissions(void) {
#if defined(__linux__)
    return wf_bridge_linux_ready == 0
        ? 0
        : wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter)
            .submissions;
#else
    return 0;
#endif
}
