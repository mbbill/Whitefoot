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
 * own-record run, the joins, the statistics and process configuration helpers
 * are written once; the only thing that differs is the platform's kernel
 * completion ring, which is behind the eight names of "the ring" below --
 * io_uring on Linux, the completion port on Windows, and none elsewhere.  The
 * one `#if` outside that section is the extra `descriptor_class` argument the
 * emitter emits per target on `wf__completion_file_open_at_submit`.
 */

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"

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
 * own-record run, the joins, the statistics and the
 * process configuration helpers -- is one implementation, and each of the three arms
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
    if (buffer == NULL || capacity == 0u || !wf_bridge_ring_ready()) {
        return 0;
    }
    ring = wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter);
    written = snprintf(
        buffer,
        capacity,
        "ring: submissions=%llu submission_enters=%llu completions=%llu "
        "kernel_waits=%llu kernel_wakes=%llu host_wake_writes=%llu "
        "overflow_flushes=%llu runtime_parks=%llu inline=%llu",
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
        )
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

/* Publish exactly once, then notify using only permanent engine storage.
 * The waiter may reclaim its frame as soon as it observes DONE. */
void wf_completion_record_complete(wf_completion_record *record) {
    if (!wf_bridge_ensure_wake()) wf_bridge_fail("completion wake initialization failed");
    atomic_fetch_add_explicit(&wf_bridge_publications, 1, memory_order_relaxed);
    wf_completion_record_publish(record);
    wf_completion_notify_target(&wf_bridge_runtime);
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
    return atomic_load_explicit(&record->state, memory_order_acquire);
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
        return wf_bridge_record_state(record) != WF_COMPLETION_PENDING;
    }
    deadline = started + WF_BRIDGE_JOIN_SPIN_NS;
    for (;;) {
        if (wf_bridge_record_state(record) != WF_COMPLETION_PENDING) {
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

/* Join may execute its own queued request, but never another frame's
 * potentially blocking request while helpers exist. Flush deferred native
 * submissions before entering a blocking host call. */
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

static void wf_bridge_join(wf_completion_record *record) {
    for (;;) {
        if (wf_bridge_record_state(record) == WF_COMPLETION_DONE) return;
        if (wf_bridge_run_own(record) || wf_bridge_progress()) continue;
        /* Capture before checking DONE: publication either precedes this
         * epoch (and its acquire orders the result), or advances the epoch
         * and prevents sleep. The wait implementation registers/rechecks
         * under its native lock, closing the notification-before-park race. */
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        if (wf_bridge_record_state(record) != WF_COMPLETION_DONE
            && !wf_bridge_spin_for_completion(record)) {
            wf_bridge_park(epoch);
        }
    }
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

/* The accept's join returns the peer address as three scalars. The ordinary
 * linked implementation builds its SocketAddress value from them, so neither
 * side holds a pointer into the other's layout.
 *
 * A refused accept publishes the all-zero address, which the linked caller
 * ignores: AcceptFailed carries an error and no peer address. */
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
 * sees. Factory quota is ordinary library state outside this private engine;
 * native descriptor refusal is reported unchanged to that library. */
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
    wf_completion_record_init(held);
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
    wf_file_complete_record(record, &result);
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
 * so the ordinary linked caller builds its failed outcome from that error.
 * No host request is executed, so the inline-execution count
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
    /* This private engine build has no enumeration request kind. A linked
     * library must not call an engine facility absent from its build. */
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

/* ------------------------------------------------------- the statistics */

uint64_t wf__completion_wait_announcements(void) {
    return atomic_load_explicit(&wf_bridge_wake_ready, memory_order_acquire) == 0
        ? 0
        : atomic_load_explicit(&wf_bridge_runtime.stat_parks, memory_order_relaxed);
}

uint64_t wf__completion_wait_signals(void) {
    return atomic_load_explicit(&wf_bridge_wake_ready, memory_order_acquire) == 0
        ? 0
        : atomic_load_explicit(&wf_bridge_runtime.stat_wake_signals, memory_order_relaxed);
}

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
