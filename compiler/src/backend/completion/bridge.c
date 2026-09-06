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
 *
 * One implementation for every platform.  The routing, the in-place wait, the
 * own-record run, the joins, the statistics and the `wf__sched_host_*` seam
 * are written once; the only thing that differs is the platform's kernel
 * completion ring, which is behind the eight names of "the ring" below --
 * io_uring on Linux, the completion port on Windows, and none elsewhere.  The
 * one `#if` outside that section is the extra `descriptor_class` argument the
 * emitter emits per target on `wf__completion_file_open_at_submit`.
 */

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
#include "../sched/core.h"
#include "../sched/entry.h"
#include "../sched/prim.h"
#if defined(__linux__)
#include "linux_io_uring.h"
#elif defined(_WIN32)
#include "windows_iocp.h"
#include "../windows_runtime.h"
#endif

#include <errno.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WF_BRIDGE_MAX_HELPERS 8u
/* The policy's ceiling and the adapter's storage are two names for one number,
 * and the adapter's is the one that decides: it carries an entry record per
 * helper (`file_adapter.h`), and a policy asking for more than it can hold
 * would be refused at init rather than granted. */
_Static_assert(
    WF_BRIDGE_MAX_HELPERS <= WF_FILE_MAX_HELPERS,
    "the helper policy may not ask for more helpers than the adapter holds"
);
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
 * wait to be pushed onto, and every submitted operation's batch carries the
 * compiler's own ceiling of two, so this number reaches one form only: a
 * staged loop whose call is a lane hand-out, whose ceiling is the lane's slot
 * count.  It is that count, so that a server keeping 1024 connections in
 * flight is bounded by its own trip count and its stacks rather than by a
 * number chosen here; it moves no file measurement because no file batch
 * reaches it. */
#define WF_BRIDGE_WINDOW_DEFAULT 1024u
/* How many completions one progress pass reaps before it returns to the
 * scheduler loop.  It was one, and one is what made the reap the serial
 * resource of the TCP echo control test: every idle thread took the
 * submission lock to kick and the completion lock to read a single entry,
 * and 64 connections ping-ponging through one ring made 720 thousand futex
 * calls a run.  At 64 the same run makes 19 thousand and the round-trip rate
 * doubles, 35 to 69 thousand a second on the development host; 1024 measures
 * the same as 64 (`research/investigations/io-model/RESULTS.md` section 6). */
#define WF_BRIDGE_REAP_BUDGET 64u
static wf_completion_runtime wf_bridge_runtime;
static wf_file_adapter wf_bridge_adapter;
static unsigned wf_bridge_once;
static unsigned wf_bridge_file_once;
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
#if !defined(WF_COMPLETION_LOCAL_INLINE)
#define WF_COMPLETION_LOCAL_INLINE 0
#endif
static int wf_bridge_progress(void);
static void wf_bridge_park(uint64_t observed_epoch);

/* The bridge's one fail-stop.
 *
 * Every `abort` in this unit is a trusted-computing-base defect rather than an
 * operation outcome, and each one is reached from a different place, so the
 * only thing that tells a reader of a crash log which one fired is the line it
 * wrote on the way out.  A bare `abort` writes nothing, and on Windows the
 * release UCRT ends such a process through the fast-fail path, which a shell
 * reports as a bare status with no message at all -- a fail-stop nobody can
 * diagnose.  This changes no control flow: it writes one line and then aborts
 * exactly where the bare call did.
 *
 * The channel is `stderr` and the write is unbuffered, because the next thing
 * this process does is die. */
static _Noreturn void wf_bridge_fail(const char *reason) {
    (void)fprintf(stderr, "whitefoot completion: %s\n", reason);
    (void)fflush(stderr);
    abort();
}

/* The platform's kernel completion ring, behind eight names; see "the ring"
 * below for what each one owes and why this is the only part of the bridge
 * that is written more than once. */
static int wf_bridge_ring_start(void);
static int wf_bridge_ring_ready(void);
static int wf_bridge_ring_offer(wf_completion_record *record);
static int wf_bridge_ring_progress(void);
static void wf_bridge_ring_flush(void);
static int wf_bridge_ring_park(uint64_t observed_epoch);
static void wf_bridge_ring_shutdown(void);
static uint64_t wf_bridge_ring_submissions(void);
static uint64_t wf_bridge_ring_submission_enters(void);


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
    unsigned long written = 0;
    /* One rule for every startup setting this runtime reads (`sched/entry.h`):
     * unset means the policy below chooses, an integer from 0 through this
     * bridge's own ceiling pins the pool, and anything else has already ended
     * the run at the core's entry. */
    if (wf__sched_setting("WF_IO_HELPERS", WF_BRIDGE_MAX_HELPERS, &written)) {
        *initial = (size_t)written;
        *cap = (size_t)written;
        /* A written setting is an instruction about how to run, so the
         * runtime stops choosing: it pins the pool exactly and keeps every
         * admitted operation on the queued completion path.  That is what
         * makes a pinned line of a measurement a measurement of the
         * completion path rather than of the policy that may run an
         * operation inline instead. */
        wf_bridge_helpers_pinned = 1;
        return;
    }
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
    if (wf_bridge_ring_ready()) {
        *initial = 0u;
        *cap = 0u;
        return;
    }
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
        WF_BRIDGE_MAX_HELPERS,
        requested
    );
    if (wf_bridge_file_error == 0
        && wf_file_adapter_set_helper_cap(&wf_bridge_adapter, cap) == 0) {
        wf_bridge_file_ready = 1;
    }
}

static int wf_bridge_ensure_file(void) {
    wf__sched_once(&wf_bridge_file_once, wf_bridge_initialize_file);
    return wf_bridge_file_ready != 0;
}

static void wf_bridge_shutdown(void) {
    if (wf_bridge_ready == 0) {
        return;
    }
    /* A pool the scheduler core started is detached and may be asleep inside
     * the engine this handler would pull down: a thread with nothing to run
     * sleeps on the one primitive (design §6), and on a completion link that
     * primitive is this bridge's park -- the ring's own wait, on descriptors,
     * mappings or a port this shutdown would take away.  There is no
     * stop protocol for a detached worker and this handler is hygiene rather
     * than contract -- `wf_bridge_initialize` already treats a failed `atexit`
     * as changing cleanup and nothing else -- so a process whose pool is still
     * running leaves its descriptors and mappings to the kernel instead of
     * taking them out from under a sleeping worker. */
    if (wf__sched_pool_running() != 0u) {
        return;
    }
    if (wf_bridge_file_ready != 0) {
        (void)wf_file_adapter_shutdown(&wf_bridge_adapter);
        wf_bridge_file_ready = 0;
    }
    wf_bridge_ring_shutdown();
    (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
    wf_bridge_ready = 0;
}

/* ------------------------------------------------------------- the ring */

/* The platform's kernel completion ring, behind eight names.
 *
 * This is the whole of what the bridge cannot write once.  Everything else in
 * this unit -- the routing, the helper policy, the in-place wait, the
 * own-record run, the joins, the window, the statistics and the
 * `wf__sched_host_*` seam -- is one implementation, and each of the three arms
 * below is exactly its platform's ring behind these names:
 *
 *   start      builds it, or answers that this run has none;
 *   ready      whether it exists;
 *   offer      hands it one record, or answers that it has no form for it;
 *   progress   reaps what is ready, without waiting;
 *   flush      rings a deferred doorbell before this thread blocks elsewhere;
 *   park       sleeps on it until an event arrives;
 *   shutdown   takes it down at process exit;
 *   submissions / submission_enters, its two counters.
 *
 * A platform with no ring answers "no" to all of them and every operation
 * takes the bounded adapter, which is the Darwin route and the route
 * WF_IO_NO_NATIVE_RING selects on either of the other two. */

#if defined(__linux__) || defined(_WIN32)

/* `wf_bridge_fail` for a fail-stop whose site holds the code the target
 * answered.  It lives with the ring rather than beside its sibling above
 * because its every caller is one of the ring seam's calls, and a target with
 * no ring makes none of them.
 *
 * The name of the site says which of the ring's calls failed; the code says
 * what it failed with, and the two are different questions.  EPROTO from a
 * reaping pass means the port handed back something that is not one of this
 * runtime's records, which is a different defect from any Win32 or errno value
 * in the same place -- and a log that carries only the site cannot tell them
 * apart without another run. */
static _Noreturn void wf_bridge_fail_with_code(const char *reason, int code) {
    (void)fprintf(stderr, "whitefoot completion: %s: error %d\n", reason, code);
    (void)fflush(stderr);
    abort();
}

/* WF_IO_NO_NATIVE_RING: run this process on the bounded adapter route.
 *
 * Runtime policy of the same class as WF_IO_NOCACHE and WF_IO_HELPERS, and
 * test-only within that class: no Whitefoot source names it, no accepted
 * program changes meaning under it, and no byte any operation produces
 * differs with it set.  Absent -- and any value other than the exact text "1"
 * -- is today's behaviour exactly.
 *
 * It exists because the route a host takes is not a choice a test can
 * otherwise make.  A host with a kernel completion ring takes every positioned
 * read into it, so the adapter branch of `bridge_default_probe`'s route
 * assertion cannot fire there at all, and the negative control it provides was
 * carried by the Darwin CI host alone.  Skipping the ring's start reaches the
 * same state a host without a ring reaches, which is a path the runtime
 * already has rather than one this setting adds.
 *
 * It is read once, before the only call that starts the ring, through the
 * platform layer's own setting read (`../sched/prim.h`, P4) rather than
 * `getenv`, because a shared unit that names `getenv` does not compile under
 * the MSVC ucrt. */
static int wf_bridge_native_ring_refused(void) {
    char text[WF_PRIM_SETTING_BYTES];
    return wf_prim_setting_text("WF_IO_NO_NATIVE_RING", text, sizeof(text)) == 1
        && text[0] == '1' && text[1] == 0;
}

#endif

#if defined(__linux__)

static wf_linux_io_uring_adapter wf_bridge_linux_adapter;
static _Atomic unsigned wf_bridge_linux_ready;
/* Diagnostic counters have static storage and survive engine teardown. A
 * one-worker process tears down its ring before the constructor-registered
 * exit observer runs; it must still report that ring's actual activity.
 * This flag never selects an operation route or authorizes a ring access. */
static _Atomic unsigned wf_bridge_linux_reportable;
/* The one piece of bridge readiness a thread may observe without running the
 * initializer itself.  `wf_bridge_ring_flush` must not create a ring for a
 * program that only ever makes direct calls, so it cannot go through the
 * once-control; this release/acquire pair is what orders the ring's
 * construction before another thread's flush of it. */
static _Atomic unsigned wf_bridge_doorbell_ready;

static int wf_bridge_ring_ready(void) {
    return wf_bridge_linux_ready != 0;
}

static int wf_bridge_ring_start(void) {
    if (wf_bridge_native_ring_refused()) {
        return 0;
    }
    if (wf_linux_io_uring_init(
            &wf_bridge_linux_adapter,
            &wf_bridge_runtime,
            WF_LINUX_IO_URING_DEPTH,
            WF_LINUX_IO_URING_COMPLETIONS
        ) != 0) {
        return 0;
    }
    if (wf_completion_set_wake_callback(
            &wf_bridge_runtime,
            wf_linux_io_uring_notify,
            &wf_bridge_linux_adapter
        ) != 0) {
        (void)wf_linux_io_uring_destroy(&wf_bridge_linux_adapter);
        return 0;
    }
    wf_bridge_linux_ready = 1;
    atomic_store_explicit(&wf_bridge_linux_reportable, 1u, memory_order_release);
    atomic_store_explicit(&wf_bridge_doorbell_ready, 1, memory_order_release);
    return 1;
}

static int wf_bridge_ring_offer(wf_completion_record *record) {
    if (!wf_bridge_ring_ready() || !wf_linux_io_uring_carries(record)) {
        return 0;
    }
    if (wf_linux_io_uring_submit(&wf_bridge_linux_adapter, record)
        != WF_LINUX_IO_URING_TARGET_OWNS) {
        /* The kind and shape were both answered before the record was
         * offered, so any other answer is a target-runtime failure. */
        wf_bridge_fail(
            "the io_uring target refused a record whose kind and shape it had already accepted"
        );
    }
    {
        int progress_error =
            wf_linux_io_uring_progress_error(&wf_bridge_linux_adapter);
        if (progress_error != 0) {
            wf_bridge_fail_with_code(
                "the io_uring target reported a failure while submitting",
                progress_error
            );
        }
    }
    return 1;
}

static int wf_bridge_ring_progress(void) {
    size_t published = 0;
    if (!wf_bridge_ring_ready()) {
        return 0;
    }
    {
        int reap_error = wf_linux_io_uring_progress(
            &wf_bridge_linux_adapter,
            WF_BRIDGE_REAP_BUDGET,
            0,
            &published
        );
        if (reap_error != 0) {
            /* Target ownership has already transferred. Falling back now would
             * duplicate an operation, and ignoring the error would strand its
             * owned operation forever. A target-runtime failure is a fail-stop
             * TCB defect, not a writer-visible IoError. */
            wf_bridge_fail_with_code(
                "the io_uring target failed while reaping completions",
                reap_error
            );
        }
    }
    return published != 0;
}

static void wf_bridge_ring_flush(void) {
    if (atomic_load_explicit(&wf_bridge_doorbell_ready, memory_order_acquire)
        != 0) {
        (void)wf_linux_io_uring_flush(&wf_bridge_linux_adapter);
    }
}

static int wf_bridge_ring_park(uint64_t observed_epoch) {
    if (!wf_bridge_ring_ready()) {
        return 0;
    }
    {
        int park_error = wf_linux_io_uring_park(
            &wf_bridge_linux_adapter,
            observed_epoch,
            UINT32_MAX
        );
        if (park_error != 0) {
            wf_bridge_fail_with_code(
                "the io_uring target failed while parking on the ring",
                park_error
            );
        }
    }
    return 1;
}

static void wf_bridge_ring_shutdown(void) {
    if (!wf_bridge_ring_ready()) {
        return;
    }
    atomic_store_explicit(&wf_bridge_doorbell_ready, 0, memory_order_release);
    (void)wf_linux_io_uring_destroy(&wf_bridge_linux_adapter);
    wf_bridge_linux_ready = 0;
}

static uint64_t wf_bridge_ring_submissions(void) {
    return wf_bridge_ring_ready()
        ? wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter)
              .submissions
        : 0u;
}

static uint64_t wf_bridge_ring_submission_enters(void) {
    return wf_bridge_ring_ready()
        ? wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter)
              .submission_enters
        : 0u;
}

/* The ring's counters as one line, for the same observer that prints the
 * core's: what the kernel was asked and how often a thread slept for it. */
int wf__bridge_report(char *buffer, size_t capacity) {
    wf_linux_io_uring_statistics ring;
    int written;
    if (buffer == NULL || capacity == 0u
        || atomic_load_explicit(&wf_bridge_linux_reportable, memory_order_acquire) == 0u) {
        return 0;
    }
    ring = wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter);
    written = snprintf(
        buffer,
        capacity,
        "ring: submissions=%llu submission_enters=%llu completions=%llu "
        "kernel_waits=%llu kernel_wakes=%llu host_wake_writes=%llu "
        "overflow_flushes=%llu runtime_parks=%llu inline=%llu tcp_nodelay=%u",
        (unsigned long long)ring.submissions,
        (unsigned long long)ring.submission_enters,
        (unsigned long long)ring.completions,
        (unsigned long long)ring.kernel_waits,
        (unsigned long long)ring.kernel_wakes,
        (unsigned long long)ring.host_wake_writes,
        (unsigned long long)ring.overflow_flushes,
        (unsigned long long)atomic_load_explicit(
            &wf_bridge_runtime.stat_parks,
            memory_order_relaxed
        ),
        (unsigned long long)atomic_load_explicit(
            &wf_bridge_inline_executions,
            memory_order_relaxed
        ),
        (unsigned)WF_TCP_NODELAY
    );
    return written > 0 && (size_t)written < capacity;
}

#elif defined(_WIN32)

static wf_windows_iocp_adapter wf_bridge_windows_adapter;
static _Atomic unsigned wf_bridge_windows_ready;

static int wf_bridge_ring_ready(void) {
    return atomic_load_explicit(&wf_bridge_windows_ready, memory_order_acquire)
        != 0u;
}

/* WF_REQUIRE_WINDOWS_IOCP: the one environment check this bridge makes that
 * has no counterpart on the other platform.
 *
 * The POSIX side has no such facility -- its own required-ring runs are the
 * harness's WF_REQUIRE_LINUX_IO_URING, which is the harness's setting and not
 * the bridge's -- so this stays a Windows-only check rather than being given a
 * shared name that one platform would never answer.  It exists because correct
 * bytes alone would also be produced by a run that never reached the port at
 * all: the `completion-windows` job's "Compile and run a real Whitefoot program
 * through IOCP" and "Open target-native Windows components from compiler-emitted
 * buffers" steps set it, and this exit-time assertion is what makes those steps
 * evidence about the ring rather than about the adapter. */
static unsigned wf_bridge_windows_require_ring;

static void wf_bridge_verify_required_ring(void) {
    wf_windows_iocp_statistics statistics;
    if (wf_bridge_windows_require_ring == 0u) {
        return;
    }
    statistics = wf_windows_iocp_statistics_snapshot(
        &wf_bridge_windows_adapter
    );
    if (statistics.submissions == 0
        || statistics.completions != statistics.submissions) {
        wf_bridge_fail(
            "WF_REQUIRE_WINDOWS_IOCP was set and this run did not complete every operation it submitted to the port"
        );
    }
}

static int wf_bridge_windows_ring_required(void) {
    char required[WF_PRIM_SETTING_BYTES];
    return wf_prim_setting_text(
               "WF_REQUIRE_WINDOWS_IOCP",
               required,
               sizeof(required)
           ) == 1
        && required[0] == '1' && required[1] == 0;
}

static int wf_bridge_ring_start(void) {
    if (wf_bridge_native_ring_refused()) {
        return 0;
    }
    if (wf_windows_iocp_init(
            &wf_bridge_windows_adapter,
            &wf_bridge_runtime,
            0
        ) != 0) {
        return 0;
    }
    if (wf_completion_set_wake_callback(
            &wf_bridge_runtime,
            wf_windows_iocp_notify,
            &wf_bridge_windows_adapter
        ) != 0) {
        (void)wf_windows_iocp_destroy(&wf_bridge_windows_adapter);
        return 0;
    }
    atomic_store_explicit(&wf_bridge_windows_ready, 1u, memory_order_release);
    if (wf_bridge_windows_ring_required()) {
        wf_bridge_windows_require_ring = 1u;
        if (atexit(wf_bridge_verify_required_ring) != 0) {
            wf_bridge_fail(
                "the completion port's exit-time check could not be registered"
            );
        }
    }
    return 1;
}

/* Binds one handle to this run's port.  The body of the association, without
 * the question of whether it has already been made: that question and this
 * answer are one critical section, and the section is the descriptor table's,
 * which is why this arrives there as a function rather than being written
 * there (`../windows_runtime.h`, `wf__windows_completion_ring_handle`). */
static int wf_bridge_windows_bind(HANDLE handle, void *context) {
    (void)context;
    return wf_windows_iocp_associate(&wf_bridge_windows_adapter, handle) == 0;
}

/* The handle the port may take for this descriptor, or none.
 *
 * A descriptor this runtime opened as a regular file for reading is the
 * ordinary case and its class says so.  A descriptor the process made some
 * other way -- a probe's own fixture -- is admitted on the one fact the port
 * needs, that it names a disk file.  Either way the association is made once
 * and remembered, because `CreateIoCompletionPort` takes a handle exactly once
 * and no host call asks whether it already has; and it is made under the
 * table's lock, because every lane of a program may offer its first record on
 * one descriptor at the same moment. */
static int wf_bridge_windows_port_handle(int descriptor, HANDLE *handle) {
    return wf__windows_completion_ring_handle(
        descriptor,
        wf_bridge_windows_bind,
        NULL,
        handle
    );
}

static int wf_bridge_ring_offer(wf_completion_record *record) {
    HANDLE handle;
    int descriptor;
    if (!wf_bridge_ring_ready() || !wf_windows_iocp_carries(record)) {
        return 0;
    }
    /* The descriptor this request will be issued on.  A connect has none until
     * the ring makes its socket, which it does here for the same reason the
     * Linux ring makes it in its own submit and for one more: the port takes a
     * handle before a request is issued on it.  A record the port then refuses
     * has that socket taken back, so the bounded adapter sees exactly what was
     * offered (`windows_iocp.h`). */
    descriptor = wf_windows_iocp_issue_descriptor(record);
    if (descriptor < 0) {
        return 0;
    }
    if (!wf_bridge_windows_port_handle(descriptor, &handle)) {
        wf_windows_iocp_withdraw(record);
        return 0;
    }
    if (wf_windows_iocp_submit(&wf_bridge_windows_adapter, record, handle)
        != WF_WINDOWS_IOCP_TARGET_OWNS) {
        wf_bridge_fail(
            "the completion port refused a record whose kind and shape it had already accepted"
        );
    }
    {
        int progress_error =
            wf_windows_iocp_progress_error(&wf_bridge_windows_adapter);
        if (progress_error != 0) {
            wf_bridge_fail_with_code(
                "the completion port reported a failure while submitting",
                progress_error
            );
        }
    }
    return 1;
}

static int wf_bridge_ring_progress(void) {
    size_t published = 0;
    if (!wf_bridge_ring_ready()) {
        return 0;
    }
    {
        int reap_error = wf_windows_iocp_progress(
            &wf_bridge_windows_adapter,
            1u,
            &published
        );
        if (reap_error != 0) {
            wf_bridge_fail_with_code(
                "the completion port failed while reaping completions",
                reap_error
            );
        }
    }
    return published != 0;
}

/* Nothing is deferred on this port: a request reaches the kernel inside the
 * call that issues it, so there is no doorbell to ring. */
static void wf_bridge_ring_flush(void) {}

static int wf_bridge_ring_park(uint64_t observed_epoch) {
    if (!wf_bridge_ring_ready()) {
        return 0;
    }
    {
        int park_error = wf_windows_iocp_park(
            &wf_bridge_windows_adapter,
            observed_epoch,
            UINT32_MAX
        );
        if (park_error != 0) {
            wf_bridge_fail_with_code(
                "the completion port failed while parking on the port",
                park_error
            );
        }
    }
    return 1;
}

static void wf_bridge_ring_shutdown(void) {
    if (!wf_bridge_ring_ready()) {
        return;
    }
    (void)wf_windows_iocp_destroy(&wf_bridge_windows_adapter);
    atomic_store_explicit(&wf_bridge_windows_ready, 0u, memory_order_release);
}

static uint64_t wf_bridge_ring_submissions(void) {
    return wf_bridge_ring_ready()
        ? wf_windows_iocp_statistics_snapshot(&wf_bridge_windows_adapter)
              .submissions
        : 0u;
}

/* The port takes each request inside the call that issues it, so there is no
 * deferred doorbell and no count of the calls that carried one. */
static uint64_t wf_bridge_ring_submission_enters(void) {
    return 0u;
}

/* The port's counters are the probe's to print; the observer's ring line is
 * the Linux ring's alone. */
int wf__bridge_report(char *buffer, size_t capacity) {
    (void)buffer;
    (void)capacity;
    return 0;
}

#else

/* A target with no kernel completion ring in the supported set: its qualified
 * path is the bounded typed adapter, and every one of these answers "no".
 *
 * This arm consults WF_IO_NO_NATIVE_RING nowhere, and its reader is not
 * compiled here: a run on a target with no ring is already the route that
 * setting selects, so a start that read it could only agree with itself.  The
 * reader sits above with the two arms that have something to refuse. */
static int wf_bridge_ring_ready(void) {
    return 0;
}

static int wf_bridge_ring_start(void) {
    return 0;
}

static int wf_bridge_ring_offer(wf_completion_record *record) {
    (void)record;
    return 0;
}

static int wf_bridge_ring_progress(void) {
    return 0;
}

static void wf_bridge_ring_flush(void) {}

static int wf_bridge_ring_park(uint64_t observed_epoch) {
    (void)observed_epoch;
    return 0;
}

static void wf_bridge_ring_shutdown(void) {}

static uint64_t wf_bridge_ring_submissions(void) {
    return 0u;
}

static uint64_t wf_bridge_ring_submission_enters(void) {
    return 0u;
}

int wf__bridge_report(char *buffer, size_t capacity) {
    (void)buffer;
    (void)capacity;
    return 0;
}

#endif

/* The wake epoch, and it comes up before everything else the bridge has.
 *
 * The core sleeps and wakes on one primitive (design §7, platform item 2), and
 * "one" has to mean one for the life of the process rather than one at a time.
 * The three seam functions below answer from `wf_bridge_runtime`, so a thread
 * that parked before this unit had a runtime would sleep on `prim_host.c`'s own
 * condition variable while every wake after it went to this one -- a lost wake
 * and, with no timeout anywhere in this design, a hang.  Two things make that
 * reachable rather than theoretical: a worker enters its scheduler loop at the
 * core's entry, before the program's first operation, and the first operation's
 * once-control is a window another thread can park inside.
 *
 * So the epoch has its own start, taken by whichever of the two arrives first:
 * a seam call, or the bridge's own initializer.  It is a mutex, a condition
 * variable and a counter -- no ring, no helper, no descriptor -- so a program
 * that never submits anything pays for those and nothing else.  Both sleep
 * mechanisms announce themselves against this one epoch under this one lock,
 * the ring's `epoll_wait` included, so one wake reaches a sleeper on either. */
static unsigned wf_bridge_wake_once;
static int wf_bridge_wake_error;
static _Atomic unsigned wf_bridge_wake_ready;

static void wf_bridge_initialize_wake(void) {
    wf_bridge_wake_error = wf_completion_runtime_init(&wf_bridge_runtime);
    if (wf_bridge_wake_error == 0) {
        wf_bridge_wake_ready = 1;
    }
}

static int wf_bridge_ensure_wake(void) {
    wf__sched_once(&wf_bridge_wake_once, wf_bridge_initialize_wake);
    return wf_bridge_wake_ready != 0;
}

static void wf_bridge_initialize(void) {
    if (!wf_bridge_ensure_wake()) {
        wf_bridge_error = wf_bridge_wake_error != 0 ? wf_bridge_wake_error : EAGAIN;
        return;
    }
    /* The ring where this run has one, and the bounded typed adapter where it
     * does not: a target with no kernel completion facility for regular files
     * -- Darwin in the supported set, and either of the other two under
     * WF_IO_NO_NATIVE_RING -- must have the adapter before it is ready, because
     * that adapter is then the whole engine. */
    if (!wf_bridge_ring_start() && !wf_bridge_ensure_file()) {
        (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
        return;
    }
    wf_bridge_ready = 1;
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
    wf__sched_once(&wf_bridge_once, wf_bridge_initialize);
    if (wf_bridge_ready == 0) {
        wf_bridge_fail(
            "the completion bridge could not initialize"
        );
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
    wf_sched_complete(&wf__sched_core, &record->sched);
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
    if (!wf_bridge_ensure_wake()) {
        return 0;
    }
    *epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
    return 1;
}

int wf__sched_host_park(uint64_t observed) {
    if (!wf_bridge_ensure_wake()) {
        return 0;
    }
    wf_bridge_park(observed);
    return 1;
}

int wf__sched_host_wake(void) {
    if (!wf_bridge_ensure_wake()) {
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
    wf_bridge_ring_flush();
}

/* Flush the deferred doorbell, reap the ring, and run one queued request when
 * this thread is the queue's only engine.  Returns nonzero when it moved
 * something.
 *
 * This is primitive 7 (§7.1) and it may block: with no helper the bounded
 * pass executes a queued host `open` or `close` on the calling thread.  That
 * is now the only place a host call is made for a queued operation. */
static int wf_bridge_progress(void) {
    int progressed = wf_bridge_ring_progress();
    /* A thread executes a queued request only when no target helper exists.
     * With helpers, taking an unrelated request here could block the exact
     * frame which is waiting on a completion they have already published. */
    if (wf_file_adapter_helper_count(&wf_bridge_adapter) == 0) {
        progressed |= wf_bridge_target_progress_one();
    }
    return progressed;
}

static void wf_bridge_park(uint64_t observed_epoch) {
    if (wf_bridge_ring_park(observed_epoch)) {
        /* Reap what the ring has first; the caller's next turn re-reads its
         * own record. */
        (void)wf_bridge_progress();
        return;
    }
    {
        enum wf_completion_park_result parked =
            wf_completion_park_if_unchanged(
                &wf_bridge_runtime,
                observed_epoch,
                UINT32_MAX
            );
        if (parked == WF_COMPLETION_PARK_FAILED) {
            wf_bridge_fail(
                "the completion runtime's park failed"
            );
        }
    }
}

/* ------------------------------------------------------------- the join */

/* How long a joining thread looks at its own record before announcing sleep.
 *
 * The clock is `wf_file_monotonic_ns`, the adapter's platform leaf, because a
 * monotonic clock is a host call: zero from it means the clock could not be
 * read, which is the same answer as a broken one for every use here, and both
 * have to end a bounded wait rather than extend it.
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

static unsigned wf_bridge_record_state(const wf_completion_record *record) {
    return wf_prim_load_u(&record->sched.state, WF_PRIM_ACQUIRE);
}

/* Waits a bounded time for this record to be completed by someone else.
 * Returns 1 when it was, so the caller re-reads it instead of parking. */
static int wf_bridge_spin_for_completion(const wf_completion_record *record) {
    uint64_t started = wf_file_monotonic_ns();
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
            uint64_t now = wf_file_monotonic_ns();
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

/* Every join parks, and this is where the rule of design §2 is entered.
 *
 * A thread on a pool stack has a stack to park, so it runs the rule: read the
 * record if it is DONE, else park this stack and continue on another, else --
 * with no free stack and nothing READY -- the I/O arm of the fourth line,
 * which is the same window `wf_bridge_wait_in_place` runs and nothing above
 * the join.  A completion resumes the parked stack from wherever it is
 * published, so a completed I/O's continuation is never buried.
 *
 * A thread that is not on a pool stack has no stack to park and waits in
 * place.  That is the harness and the probes, which call these joins from
 * plain threads; a linked program never does, because the floor runs its entry
 * on a pool stack and every worker's loop lives on one (design §5).
 *
 * Before either arm, this thread runs its *own* submission if it is still
 * queued.  It is the engine executing an operation that has no other engine,
 * not a schedule decision and not a fallback: with the helper count pinned, an
 * ordinary write can sit in the queue behind an unrelated blocked one, and no
 * helper, no ring and no progress pass will ever reach it --
 * `wf_bridge_progress` deliberately takes nothing from the queue while helpers
 * exist, because an unrelated request could block the exact frame whose
 * completion they have already published.  The record is this frame's own, so
 * taking it here takes nothing that belongs to another frame.  One attempt is
 * the whole of it: after it, the record is either DONE, or owned by an engine
 * that will finish it, and the rule below waits for that.
 *
 * What licenses that host call is the second half of the sentence: this thread
 * has nothing else to do until the record is DONE.  On the wait-in-place arm
 * that is simply true -- there is no stack to park and no other work reachable
 * from here.  On a pool stack it is not: the join below parks this stack and
 * the thread goes on to another, so a host call made here holds a *worker*,
 * and with it every continuation that worker would have run.  For a wait this
 * runtime can end that costs some overlap; for a peer-bound wait, which
 * another program ends whenever it likes, it costs the worker for as long as
 * that program is quiet.  Three workers each inside a receive from a silent
 * peer is `tests/programs/tcp_fanout.wf` with nobody left to accept its fourth
 * connection.
 *
 * So a pool stack leaves a peer-bound record to the helper pool -- once there
 * is a pool.  With no helper the claim happens on either arm, because then
 * nothing else would run it: the pool may be pinned at zero, which makes the
 * waiting thread the queue's own engine, or a helper start may have failed,
 * and in both cases refusing the record here would strand it.  That is the
 * same condition `wf_bridge_progress` asks before it takes anything, and it is
 * why the adapter's growth rule starts a helper on the peer-bound submission
 * itself (`file_adapter.c`). */
static int wf_bridge_own_runs_on_this_thread(
    const wf_completion_record *record,
    int on_pool_stack
) {
    return on_pool_stack == 0
        || !wf_file_request_is_peer_bound(&record->request)
        || wf_file_adapter_helper_count(&wf_bridge_adapter) == 0;
}

static void wf_bridge_join(wf_completion_record *record) {
    int on_pool_stack = wf__sched_current_stack() != NULL;
    if (wf_bridge_own_runs_on_this_thread(record, on_pool_stack)) {
        (void)wf_bridge_run_own(record);
    }
    if (on_pool_stack) {
        wf_sched_join(&wf__sched_core, &record->sched, 1);
        return;
    }
    wf_bridge_wait_in_place(record);
}

static wf_completion_record *wf_bridge_record_of(const void *record) {
    if (record == NULL) {
        wf_bridge_fail(
            "a join was given no record"
        );
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
        wf_bridge_fail(
            "a join was given no place to publish its result"
        );
    }
    wf_bridge_join(held);
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
        wf_bridge_fail(
            "an open join was given no place to publish its result"
        );
    }
    wf_bridge_join(held);
    if (held->result.kind != WF_FILE_OPEN_AT) {
        wf_bridge_fail(
            "an open join was given a record that is not an open"
        );
    }
    *value = held->result.value;
    *error_code = held->result.error_code;
    *open_outcome = (unsigned)held->result.open_outcome;
}

/* The accept's join.  The peer address is three scalars because that is what
 * an emitted `SocketAddress` value is, so the emitted wrapper builds its own
 * value out of them and neither side holds a pointer into the other's layout.
 *
 * A refused accept publishes the all-zero address, which the emitted mapper
 * never reads: `AcceptFailed` carries the error and the permit and no peer
 * [SYS-17]. */
void wf__completion_socket_accept_join(
    const void *record,
    int64_t *value,
    int *error_code,
    uint64_t *peer_low,
    uint64_t *peer_high,
    uint32_t *peer_tag
) {
    wf_completion_record *held = wf_bridge_record_of(record);
    if (value == NULL || error_code == NULL || peer_low == NULL
        || peer_high == NULL || peer_tag == NULL) {
        wf_bridge_fail(
            "an accept join was given no place to publish its result"
        );
    }
    wf_bridge_join(held);
    if (held->result.kind != WF_FILE_SOCKET_ACCEPT) {
        wf_bridge_fail(
            "an accept join was given a record that is not an accept"
        );
    }
    *value = held->result.value;
    *error_code = held->result.error_code;
    *peer_low = held->request.operation.accept.peer.portable.words[0];
    *peer_high = held->request.operation.accept.peer.portable.words[1];
    *peer_tag = held->request.operation.accept.peer.portable.port_and_family;
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
        wf_bridge_fail(
            "a status join was given no place to publish its result"
        );
    }
    wf_bridge_join(held);
    if (held->result.kind != WF_FILE_STATUS
        || held->request.operation.status.destination != status
        || held->request.operation.status.capacity != (size_t)status_capacity) {
        wf_bridge_fail(
            "a status join was given a record that is not the status it submitted"
        );
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
        case WF_FILE_SOCKET_RECEIVE:
            return request->operation.receive.count == 0;
        case WF_FILE_SOCKET_SEND:
            return request->operation.send.count == 0;
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
 * sees: the descriptor an open needs is owned by its `HandlePermit` [SYS-10],
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
        wf_bridge_fail(
            "a submit was given no record"
        );
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
/* Completes a socket transfer here when the host answers it without waiting:
 * the operation's outcome is the host's own, and nothing parks, wakes, or
 * crosses a ring for an answer that was already there. */
static int wf_bridge_transfer_now(wf_completion_record *record) {
    wf_file_result result;
    if (!wf_file_transfer_now(&record->request, &result)) {
        return 0;
    }
    record->route = WF_COMPLETION_ROUTE_INLINE;
    atomic_fetch_add_explicit(
        &wf_bridge_inline_executions,
        1,
        memory_order_relaxed
    );
#if WF_COMPLETION_LOCAL_INLINE
    /* The fresh record has never been offered to an engine or a waiter.
     * This call is still inside submit; its caller cannot reach join until
     * it returns. A socket transfer writes no extra status buffer, so its
     * complete result is the head. Publish DONE without the shared record's
     * COMPLETING/waiter-claim protocol. The pending path below still uses
     * that full protocol. Keep diagnostic counts identical in both forms. */
    record->result = result.head;
    atomic_fetch_add_explicit(&wf_bridge_publications, 1, memory_order_relaxed);
    wf_prim_store_u(&record->sched.state, WF_SCHED_DONE, WF_PRIM_RELEASE);
#else
    wf_file_complete_record(record, &result);
#endif
    return 1;
}

static void wf_bridge_dispatch(wf_completion_record *record) {
    if (wf_bridge_file_request_is_empty(&record->request)) {
        wf_bridge_complete_empty(record);
        return;
    }
    if (wf_bridge_ring_offer(record)) {
        return;
    }
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
        wf_bridge_fail(
            "a read was submitted with a buffer and a count that do not describe a range"
        );
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
        wf_bridge_fail(
            "a positioned read was submitted with a buffer and a count that do not describe a range"
        );
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
    if (wf_bridge_file_request_is_empty(&held->request)) {
        wf_bridge_complete_empty(held);
        return;
    }
    /* The ring first, then the throughput choice below, then the queue.  A
     * positioned read the ring takes is never a candidate for running inside
     * submit: the ring has no wait for this thread to overlap. */
    if (wf_bridge_ring_offer(held)) {
        return;
    }
    if (wf_bridge_positioned_read_runs_on_caller(count)) {
        wf_bridge_execute_here(held);
        return;
    }
    wf_bridge_submit_file(held);
}

void wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if ((buffer == NULL && count != 0) || (uint64_t)(size_t)count != count) {
        wf_bridge_fail(
            "a write was submitted with a buffer and a count that do not describe a range"
        );
    }
    /* write_once is an unpositioned OutputStream operation. The native adapter's
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

/* The one place an ABI the emitter emits per target reaches this unit.
 *
 * A Windows open has to know which resource the descriptor will become before
 * it opens, because the access and the create options it asks the namespace
 * for differ by that answer; no other target's open does, and every other
 * target's leaf ignores the field the argument fills.  So the emitter emits
 * one more argument on that target alone (`emitter/completion.rs`,
 * COMPLETION_WINDOWS_RUNTIME_DECLARATIONS) and this signature follows it.
 * Nothing else in this unit is `#if`-forked, and this is a difference in an
 * ABI rather than a fork of any logic: both arms fill the same record and both
 * fall into the same routing below. */
void wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
#if defined(_WIN32)
    unsigned descriptor_class,
#endif
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if (path == NULL || has_mode > 1u
        || expected_kind > WF_FILE_EXPECT_DIRECTORY) {
        wf_bridge_fail(
            "an open was submitted with no path, or with a mode or expected kind out of range"
        );
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
#if defined(_WIN32)
    held->request.operation.open_at.descriptor_class = descriptor_class;
#endif
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
        wf_bridge_fail(
            "a status was submitted with no destination, or with a capacity out of range"
        );
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

/* The six TCP submits.
 *
 * Each fills one arm of the request union and falls into the one routing
 * `wf_bridge_dispatch` performs for every kind: the ring where it has a form,
 * the bounded adapter otherwise, and the engine here when neither applies.
 * The addresses arrive as the three scalars an emitted `SocketAddress` value
 * is and are stored in the record in that form; whichever engine takes the
 * operation converts them into the host's own record, because the shape of
 * that record is the leaf's business and not this unit's. */
static void wf_bridge_socket_endpoint(
    wf_completion_record *held,
    uint64_t address_low,
    uint64_t address_high,
    uint32_t port_and_family
) {
    held->request.operation.endpoint.descriptor = -1;
    held->request.operation.endpoint.address_length = 0u;
    held->request.operation.endpoint.address.portable.words[0] = address_low;
    held->request.operation.endpoint.address.portable.words[1] = address_high;
    held->request.operation.endpoint.address.portable.port_and_family =
        port_and_family;
}

void wf__completion_socket_listen_submit(
    uint64_t address_low,
    uint64_t address_high,
    uint32_t port_and_family,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    held->request.kind = WF_FILE_SOCKET_LISTEN;
    wf_bridge_socket_endpoint(held, address_low, address_high, port_and_family);
    wf_bridge_dispatch(held);
}

void wf__completion_socket_accept_submit(
    int listener,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if (listener < 0) {
        wf_bridge_fail(
            "an accept was submitted with no listener"
        );
    }
    held->request.kind = WF_FILE_SOCKET_ACCEPT;
    held->request.operation.accept.descriptor = listener;
    /* The whole of the storage the host may write, so a peer of either family
     * fits and the host reports back what it actually used. */
    held->request.operation.accept.peer_length =
        (unsigned)sizeof(held->request.operation.accept.peer.native);
    wf_bridge_dispatch(held);
}

void wf__completion_socket_connect_submit(
    uint64_t address_low,
    uint64_t address_high,
    uint32_t port_and_family,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    held->request.kind = WF_FILE_SOCKET_CONNECT;
    wf_bridge_socket_endpoint(held, address_low, address_high, port_and_family);
    wf_bridge_dispatch(held);
}

void wf__completion_socket_receive_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if ((buffer == NULL && count != 0) || (uint64_t)(size_t)count != count) {
        wf_bridge_fail(
            "a receive was submitted with a buffer and a count that do not describe a range"
        );
    }
    held->request.kind = WF_FILE_SOCKET_RECEIVE;
    held->request.operation.receive.descriptor = descriptor;
    held->request.operation.receive.buffer = buffer;
    held->request.operation.receive.count = (size_t)count;
    if (wf_bridge_transfer_now(held)) {
        return;
    }
    wf_bridge_dispatch(held);
}

void wf__completion_socket_send_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if ((buffer == NULL && count != 0) || (uint64_t)(size_t)count != count) {
        wf_bridge_fail(
            "a send was submitted with a buffer and a count that do not describe a range"
        );
    }
    held->request.kind = WF_FILE_SOCKET_SEND;
    held->request.operation.send.descriptor = descriptor;
    held->request.operation.send.buffer = buffer;
    held->request.operation.send.count = (size_t)count;
    if (wf_bridge_transfer_now(held)) {
        return;
    }
    wf_bridge_dispatch(held);
}

void wf__completion_socket_shutdown_submit(
    int descriptor,
    unsigned direction,
    void *record
) {
    wf_completion_record *held = wf_bridge_begin(record);
    if (direction > (unsigned)WF_SOCKET_DIRECTION_SEND) {
        wf_bridge_fail(
            "a half-close was submitted with a direction this contract cannot mean"
        );
    }
    held->request.kind = WF_FILE_SOCKET_SHUTDOWN;
    held->request.operation.shutdown.descriptor = descriptor;
    held->request.operation.shutdown.direction =
        (enum wf_socket_direction)direction;
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
        wf_bridge_fail(
            "a directory read was submitted with no position, or with a buffer and a count that do not describe a range"
        );
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
    wf_bridge_fail(
        "this target has no directory enumeration facility and no such request may reach this entry"
    );
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
    wf__sched_once(&wf_bridge_once, wf_bridge_initialize);
    if (wf_bridge_ready == 0) {
        /* With no completion runtime every operation is a blocking direct
         * call, so depth buys nothing and one is the honest answer. */
        return 1;
    }
    window = WF_BRIDGE_WINDOW_DEFAULT;
    if (!wf_bridge_ring_ready()
        && (uint64_t)WF_BRIDGE_MAX_HELPERS < window) {
        window = (uint64_t)WF_BRIDGE_MAX_HELPERS;
    }
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

/* ------------------------------------------------------- the statistics */

uint64_t wf__completion_file_submissions(void) {
    uint64_t submissions = wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter).submissions;
    return submissions + wf_bridge_ring_submissions();
}

uint64_t wf__completion_file_fallback_submissions(void) {
    return wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter).submissions;
}

/* Calls this process made to carry staged submissions to the kernel, where the
 * ring defers its doorbell.  With `io_uring`'s doorbell deferred this stays far
 * below the submission count, and the distance between the two is what
 * deferring bought; a ring that carries each request inside the call that
 * issues it, as the completion port does, answers zero. */
uint64_t wf__completion_native_ring_submission_enters(void) {
    return wf_bridge_ring_submission_enters();
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

uint64_t wf__completion_native_ring_submissions(void) {
    return wf_bridge_ring_submissions();
}

/* The ceiling of this bridge's own WF_IO_HELPERS setting, answered for the
 * core's entry so that a setting this runtime cannot mean ends the run before
 * the program body rather than at its first operation (`sched/entry.h`). */
unsigned long wf__sched_helper_ceiling(void) {
    return (unsigned long)WF_BRIDGE_MAX_HELPERS;
}
