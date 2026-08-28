#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* The online-CPU query the helper policy reads is a BSD extension that the
 * POSIX macro above hides on Darwin, so the Darwin namespace is asked for as
 * well. glibc declares it unconditionally. */
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
 * A weak LLVM fallback makes an emitted module independently linkable.  When
 * this strong unit is linked, one bounded process runtime supplies stable
 * operation slots before target handoff.  A zero-helper configuration is a
 * real supported mode: the joining scheduler advances the same adapter queue.
 */

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
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
#include <unistd.h>

#define WF_BRIDGE_OPERATION_CAPACITY 64u
#define WF_BRIDGE_SLOT_COUNT WF_BRIDGE_OPERATION_CAPACITY
#define WF_BRIDGE_QUEUE_COUNT WF_BRIDGE_OPERATION_CAPACITY
#define WF_BRIDGE_MAX_HELPERS 8u
#define WF_BRIDGE_DRAIN_BUDGET 16u
/* Private storage one loop may hold for its in-flight iterations, before the
 * compiler's own ceiling and the loop's per-iteration size are applied.  It
 * exists so a loop whose iteration owns a large buffer gets a small window
 * instead of a large multiple of that buffer: at 64 KiB an iteration this
 * budget affords 64 of them, and at 16 MiB it affords none, which the K >= 1
 * floor turns into the sequential program. */
#define WF_BRIDGE_WINDOW_BYTE_BUDGET (4u * 1024u * 1024u)

static wf_completion_runtime wf_bridge_runtime;
static wf_completion_slot wf_bridge_slots[WF_BRIDGE_SLOT_COUNT];
static wf_file_adapter wf_bridge_adapter;
static wf_file_work wf_bridge_queue[WF_BRIDGE_QUEUE_COUNT];
static pthread_t wf_bridge_helpers[WF_BRIDGE_MAX_HELPERS];
static pthread_once_t wf_bridge_once = PTHREAD_ONCE_INIT;
static pthread_once_t wf_bridge_file_once = PTHREAD_ONCE_INIT;
static int wf_bridge_error;
static int wf_bridge_file_error;
static unsigned wf_bridge_ready;
static unsigned wf_bridge_file_ready;
/* Opens the completion path refused because no operation record can hold the
 * name, each of which its caller then performs as a direct blocking open.
 * `open_read` resolves a caller path buffer of any admitted length [PATH-1],
 * so a program reaches this without writing anything wrong, and the demotion
 * is otherwise invisible: neither the completion counter nor the fallback
 * adapter's counter moves for an operation that was never submitted. */
static _Atomic uint64_t wf_bridge_demoted_opens;
/* Opens a blocking direct call made, which the host refused for want of a
 * descriptor and which asked it again after this runtime gave one back.  The
 * two target engines count their own; together the three are what
 * `wf__completion_open_exhaustion_retries` reports, because the rule they
 * follow is one rule. */
static _Atomic uint64_t wf_bridge_direct_exhaustion_retries;
#if defined(__linux__)
static wf_linux_io_uring_adapter wf_bridge_linux_adapter;
static wf_linux_io_uring_entry wf_bridge_linux_entries[WF_BRIDGE_SLOT_COUNT];
static unsigned wf_bridge_linux_ready;
/* The one piece of bridge readiness a thread may observe without running the
 * initializer itself.  `wf_bridge_flush_target` must not create a ring for a
 * program that only ever makes direct calls, so it cannot go through
 * `pthread_once`; this release/acquire pair is what orders the ring's
 * construction before another thread's flush of it. */
static _Atomic unsigned wf_bridge_doorbell_ready;
#endif

enum wf_bridge_route {
    WF_BRIDGE_ROUTE_NONE = 0,
    WF_BRIDGE_ROUTE_FILE_FALLBACK = 1,
    WF_BRIDGE_ROUTE_LINUX_IO_URING = 2
};

/* Supplied by the compute runtime when one is linked.  The bridge's own weak
 * answer keeps completion-only programs independent of that runtime. */
__attribute__((weak)) int wf__par_help_once(void) {
    return 0;
}

/* The helper policy, in one place.
 *
 * A written WF_IO_HELPERS pins the count: `*initial` and `*cap` are both the
 * written value, so a program asked for four helpers gets four and never a
 * fifth, and a program asked for none keeps the zero-helper path where a
 * waiting scheduler is itself the target engine.
 *
 * Unset asks for demand-driven growth from one. One helper is the right
 * answer for a program with a single operation outstanding, and it was the
 * old fixed default; it is the wrong answer for a program that exposes real
 * width, because the submitting scheduler then waits on a queue only one
 * thread is draining. On the four-wide many-file workload the fixed default
 * finished 1.6x slower than the same program at four helpers, so growth is
 * bounded by the machine rather than pinned at one. */
static void wf_bridge_helper_policy(size_t *initial, size_t *cap) {
    const char *text = getenv("WF_IO_HELPERS");
    char *end = NULL;
    unsigned long parsed;
    long online;
    if (text != NULL && *text != 0) {
        errno = 0;
        parsed = strtoul(text, &end, 10);
        if (errno == 0 && end != text && *end == 0) {
            if (parsed > WF_BRIDGE_MAX_HELPERS) {
                parsed = WF_BRIDGE_MAX_HELPERS;
            }
            *initial = (size_t)parsed;
            *cap = (size_t)parsed;
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
     * scheduler runs those operations itself. WF_IO_HELPERS still pins a pool
     * for a target whose opens really do wait. */
    if (wf_bridge_linux_ready != 0) {
        *initial = 0u;
        *cap = 0u;
        return;
    }
#endif
    online = sysconf(_SC_NPROCESSORS_ONLN);
    if (online < 1) {
        online = 1;
    }
    *cap = (size_t)online < WF_BRIDGE_MAX_HELPERS ? (size_t)online
                                                  : WF_BRIDGE_MAX_HELPERS;
    *initial = 1u;
}

static int wf_bridge_target_progress_one(void) {
    return wf_bridge_file_ready != 0
        && wf_file_adapter_progress(&wf_bridge_adapter, 1u) != 0;
}

/* One newly queued request is announced once, under the queue lock the
 * enqueue already holds, and reaches exactly one helper.  With zero helpers a
 * sleeping scheduler is itself the target's engine, so the same announcement
 * has to reach the completion core's park endpoint as well. */
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
        wf_bridge_queue,
        WF_BRIDGE_QUEUE_COUNT,
        wf_bridge_helpers,
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

static void wf_bridge_initialize(void) {
    wf_bridge_error = wf_completion_runtime_init(
        &wf_bridge_runtime,
        wf_bridge_slots,
        WF_BRIDGE_SLOT_COUNT
    );
    if (wf_bridge_error != 0) {
        return;
    }
#if defined(__linux__)
    if (wf_linux_io_uring_init(
            &wf_bridge_linux_adapter,
            &wf_bridge_runtime,
            wf_bridge_linux_entries,
            WF_BRIDGE_SLOT_COUNT
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

static size_t wf_bridge_drain(void) {
    wf_completion_event events[WF_BRIDGE_DRAIN_BUDGET];
    return wf_completion_drain(
        &wf_bridge_runtime,
        events,
        WF_BRIDGE_DRAIN_BUDGET,
        WF_BRIDGE_DRAIN_BUDGET
    );
}

/* Progresses both bounded target work and one ready compute offer before a
 * park.  The compute hook never enters file helpers and the file adapter never
 * receives a Whitefoot function pointer. */
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
    /* Native CQ reaping comes first after a kernel wake. It can make a saved
     * writer frame ready before the bounded target/compute passes below. */
    progressed |= wf_bridge_drain() != 0;
    /* A scheduler lane executes a blocking fallback request only when no
     * target helper exists. With helpers, taking an unrelated request here
     * could block the exact writer which is waiting to consume a completion
     * they have already published. */
    if (wf_file_adapter_helper_count(&wf_bridge_adapter) == 0) {
        progressed |= wf_bridge_target_progress_one();
    }
    progressed |= wf__par_help_once() != 0;
    return progressed;
}

/* True exactly when this waiting scheduler is the only thread that can still
 * execute a queued target request, so parking here would strand it.
 *
 * That is the zero-helper configuration and nothing else. With no helper the
 * bounded target pass inside `wf_bridge_progress` is the engine, and a queued
 * request moves only when some scheduler runs it. With helpers the request
 * already has an owner: every enqueue signals one helper under the adapter's
 * own queue lock, that helper drains the queue before it sleeps again, and the
 * terminal publication is this waiter's wake.
 *
 * Treating a non-empty queue as a reason not to park regardless of helpers is
 * therefore not caution, it is a busy wait for exactly as long as a helper
 * keeps the queue non-empty. Measured on the four-wide many-file workload,
 * the one-helper default spun about 270 ms of user CPU against 71 ms for the
 * same run at four helpers, and finished 1.6x slower than its own best
 * setting. */
static int wf_bridge_target_work_needs_this_thread(void) {
    return wf_bridge_file_ready != 0
        && wf_file_adapter_helper_count(&wf_bridge_adapter) == 0
        && wf_file_adapter_queued(&wf_bridge_adapter) != 0;
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
        /* Reap the CQ first, then rerun the bounded target/drain/compute pass.
         * The caller's next loop iteration handles any writer-ready frame. */
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

static int wf_bridge_claim(wf_completion_token *token) {
    return wf_completion_claim(&wf_bridge_runtime, token)
        == WF_COMPLETION_CLAIMED;
}

static int wf_bridge_file_request_is_empty(const wf_file_request *request) {
    switch (request->kind) {
        case WF_FILE_READ:
            return request->operation.read.count == 0;
        case WF_FILE_WRITE:
            return request->operation.write.count == 0;
        case WF_FILE_PREAD:
            return request->operation.pread.count == 0;
        default:
            return 0;
    }
}

static int wf_bridge_publish_inline_file(
    wf_completion_token token,
    void *dependent_frame,
    enum wf_file_operation_kind kind,
    int64_t value,
    int error_code
) {
    wf_file_result result;
    wf_completion_publication publication;
    memset(&result, 0, sizeof(result));
    result.kind = kind;
    result.value = value;
    result.error_code = error_code;
    if (wf_completion_set_adapter_tag(
            &wf_bridge_runtime,
            token,
            WF_BRIDGE_ROUTE_FILE_FALLBACK
        ) != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_bridge_runtime,
            token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    if (wf_completion_begin_submit(&wf_bridge_runtime, token)
        != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = error_code == 0 ? 1u : 2u;
    publication.result = &result;
    publication.result_size = sizeof(result);
    if (wf_completion_publish_inline_terminal(
            &wf_bridge_runtime,
            token,
            &publication
        ) != WF_COMPLETION_PUBLISHED) {
        abort();
    }
    return 1;
}

static int wf_bridge_submit_file(
    const wf_file_request *request,
    wf_completion_token *token,
    void *dependent_frame
) {
    if (pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0 || !wf_bridge_ensure_file()) {
        return 0;
    }
    if (!wf_bridge_claim(token)) {
        return 0;
    }
    if (wf_bridge_file_request_is_empty(request)) {
        return wf_bridge_publish_inline_file(
            *token,
            dependent_frame,
            request->kind,
            0,
            0
        );
    }
    if (wf_completion_set_adapter_tag(
            &wf_bridge_runtime,
            *token,
            WF_BRIDGE_ROUTE_FILE_FALLBACK
        ) != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_bridge_runtime,
            *token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        enum wf_file_submit_result submitted = wf_file_adapter_submit(
            &wf_bridge_adapter,
            *token,
            request
        );
        if (submitted == WF_FILE_TARGET_OWNS) {
            wf_bridge_notify_target();
            return 1;
        }
        if (submitted != WF_FILE_WAIT_CAPACITY) {
            return 0;
        }
        if (!wf_bridge_progress()) {
            wf_bridge_park(epoch);
        }
    }
}

#if defined(__linux__)
/* Submits one typed request to the native ring.
 *
 * Returns -1 only while target ownership is still available to the caller, so
 * the caller may honestly choose the bounded POSIX adapter; 1 once the ring
 * owns the operation; 0 when the operation could not be claimed at all. */
static int wf_bridge_submit_linux(
    const wf_linux_file_request *request,
    wf_completion_token *token,
    void *dependent_frame
) {
    if (pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0 || wf_bridge_linux_ready == 0) {
        return -1;
    }
    if (!wf_bridge_claim(token)) {
        return 0;
    }
    if (wf_completion_set_adapter_tag(
            &wf_bridge_runtime,
            *token,
            WF_BRIDGE_ROUTE_LINUX_IO_URING
        ) != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_bridge_runtime,
            *token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        enum wf_linux_io_uring_submit_result submitted =
            wf_linux_io_uring_submit(
                &wf_bridge_linux_adapter,
                *token,
                request
            );
        if (submitted == WF_LINUX_IO_URING_TARGET_OWNS) {
            if (wf_linux_io_uring_progress_error(
                    &wf_bridge_linux_adapter
                ) != 0) {
                abort();
            }
            return 1;
        }
        if (submitted != WF_LINUX_IO_URING_WAIT_CAPACITY) {
            /* `linux_ready` and the request checks above close every honest
             * pre-ownership fallback. Reaching another result after claiming
             * the operation is an internal adapter/core contract defect. */
            abort();
        }
        if (!wf_bridge_progress()) {
            wf_bridge_park(epoch);
        }
    }
}

/* The ring's READ_AT fixes an explicit offset, so only a nonempty,
 * representable positioned transfer qualifies for it. */
static int wf_bridge_submit_linux_pread(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    wf_completion_token *token,
    void *dependent_frame
) {
    wf_linux_file_request request;
    /* Anything the ring would refuse is answered before an operation is
     * claimed, so a refusal is an honest fallback to the bounded POSIX
     * adapter rather than a fail-stop after ownership moved. */
    if (count == 0 || count > UINT32_MAX || descriptor < 0) {
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_LINUX_FILE_READ_AT;
    request.descriptor = descriptor;
    request.buffer.read_buffer = buffer;
    request.count = (size_t)count;
    request.offset = file_offset;
    return wf_bridge_submit_linux(&request, token, dependent_frame);
}

/* An open reaches the ring as IORING_OP_OPENAT, and its kind check as a
 * further IORING_OP_STATX of the descriptor the open produced.  Both are
 * ordinary ring operations, so no scheduler thread blocks in `openat` and no
 * helper handoff stands between a program's independent opens. */
static int wf_bridge_submit_linux_open_at(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    wf_completion_token *token,
    void *dependent_frame
) {
    wf_linux_file_request request;
    /* Anything the ring would refuse is answered before an operation is
     * claimed, and a name the ring entry cannot hold is one of them. */
    if (!wf_file_path_fits(path) || has_mode > 1u
        || expected_kind > WF_FILE_EXPECT_DIRECTORY) {
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_LINUX_FILE_OPEN_AT;
    request.descriptor = directory;
    request.buffer.path = path;
    request.open_flags = flags;
    request.open_mode = mode;
    request.has_open_mode = has_mode;
    request.expected_kind = (enum wf_file_expected_kind)expected_kind;
    return wf_bridge_submit_linux(&request, token, dependent_frame);
}

static int wf_bridge_submit_linux_close(
    int descriptor,
    wf_completion_token *token,
    void *dependent_frame
) {
    wf_linux_file_request request;
    /* A descriptor this program never held is refused by the bounded POSIX
     * adapter as an ordinary typed EBADF, which is a writer-visible outcome;
     * the ring would refuse it as a request-shape defect, which is not. */
    if (descriptor < 0) {
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_LINUX_FILE_CLOSE;
    request.descriptor = descriptor;
    return wf_bridge_submit_linux(&request, token, dependent_frame);
}
#endif

/* Rings the deferred io_uring doorbell before this thread does something the
 * ring cannot see through.
 *
 * Staging an SQE costs no syscall, so a submission leaves work the kernel has
 * not been told about.  That is exactly what makes deferring worth 15 % of the
 * eight-wide benchmark's wall time, and exactly what makes an unguarded
 * blocking call a hazard: an open the program has already submitted would sit
 * in the submission queue, untouched, for as long as this thread waits in a
 * direct `openat` that the completion path refused to carry.  Every entry
 * point below that can block outside the ring flushes first.  On a target with
 * no ring this is nothing. */
static void wf_bridge_flush_target(void) {
#if defined(__linux__)
    if (atomic_load_explicit(&wf_bridge_doorbell_ready, memory_order_acquire)
        != 0) {
        (void)wf_linux_io_uring_flush(&wf_bridge_linux_adapter);
    }
#endif
}

/* How many iterations of one loop the runtime will carry in flight at once.
 *
 * Asked once per loop entry and never per iteration, exactly as
 * `wf__par_split_budget` is (`par_runtime.c`).  The writer never sees this
 * number, never spells it, and cannot influence it: there is no attribute, no
 * environment variable, and no source form for a window.
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
 * The runtime's own bound is half its process-wide operation capacity.  The
 * whole capacity is the wrong answer: a loop that owned every record would
 * push every other operation in the program onto the capacity-wait path, and a
 * full ring degrades to a blocking direct call rather than to a slower
 * pipeline (LOOP-PIPELINE.md §3.9 item 6).  Half of 64 is 32, which is also
 * where the hand-written ceiling program measured its optimum (§9.2).
 *
 * Where the ring is the target engine the ring's capacity bounds it as well;
 * where a bounded helper pool is, the pool's width does. */
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
    window = (uint64_t)WF_BRIDGE_OPERATION_CAPACITY / 2u;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        uint64_t ring =
            (uint64_t)wf_linux_io_uring_capacity(&wf_bridge_linux_adapter);
        if (ring < window) {
            window = ring;
        }
    } else if ((uint64_t)WF_BRIDGE_MAX_HELPERS < window) {
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

int wf__completion_file_read_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || (buffer == NULL && count != 0)) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = descriptor;
    request.operation.read.buffer = buffer;
    request.operation.read.count = (size_t)count;
    if ((uint64_t)request.operation.read.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || (buffer == NULL && count != 0)
        || file_offset > (uint64_t)INT64_MAX) {
        return 0;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            file_offset,
            (wf_completion_token *)token_storage,
            NULL
        );
        if (native >= 0) {
            return native;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = (int64_t)file_offset;
    if ((uint64_t)request.operation.pread.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || (buffer == NULL && count != 0)) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count) {
        return 0;
    }
    /* write_once is an unpositioned Output operation. The native adapter's
     * WRITE_AT request fixes an explicit offset, which would change regular
     * file-offset semantics and is not a meaningful stream offset. Until a
     * Linux request kind is qualified for exact write(2) current-position and
     * append/stream behavior, keep this on the bounded typed fallback. */
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || has_mode > 1u
        || expected_kind > WF_FILE_EXPECT_DIRECTORY) {
        return 0;
    }
    /* A submitted open outlives this call, so its name must fit the operation
     * record that will own the bytes.  A longer name is refused here, before
     * anything is claimed, and the caller's direct open carries it instead —
     * that path resolves the caller's buffer inside its own call and needs no
     * copy at all.  It is counted apart from the shape refusals above because
     * it is the one an admitted program reaches: the outcome is unchanged and
     * only the completion path is lost, which is a throughput fact a reader
     * measuring this runtime needs to see. */
    if (!wf_file_path_fits(path)) {
        atomic_fetch_add_explicit(
            &wf_bridge_demoted_opens,
            1,
            memory_order_relaxed
        );
        /* The caller answers a refusal with a direct blocking open, so no
         * already-submitted operation may be left waiting on a doorbell this
         * thread is about to stop ringing. */
        wf_bridge_flush_target();
        return 0;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_open_at(
            directory,
            path,
            flags,
            mode,
            has_mode,
            expected_kind,
            (wf_completion_token *)token_storage,
            NULL
        );
        if (native >= 0) {
            return native;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = directory;
    request.operation.open_at.path = path;
    request.operation.open_at.flags = flags;
    request.operation.open_at.mode = mode;
    request.operation.open_at.has_mode = has_mode;
    request.operation.open_at.expected_kind =
        (enum wf_file_expected_kind)expected_kind;
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_status_submit(
    int descriptor,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_STATUS;
    request.operation.status.descriptor = descriptor;
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_close_submit(
    int descriptor,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL) {
        return 0;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_close(
            descriptor,
            (wf_completion_token *)token_storage,
            NULL
        );
        if (native >= 0) {
            return native;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = descriptor;
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_directory_next_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position,
    void *token_storage
) {
#if defined(__APPLE__)
    wf_file_request request;
    if (token_storage == NULL || position == NULL
        || (buffer == NULL && count != 0)
        || (uint64_t)(size_t)count != count) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_GETDIRENTRIES64;
    request.operation.getdirentries64.descriptor = descriptor;
    request.operation.getdirentries64.buffer = buffer;
    request.operation.getdirentries64.count = (size_t)count;
    request.operation.getdirentries64.position = position;
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
#else
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)position;
    (void)token_storage;
    return 0;
#endif
}

int wf__completion_file_pread_submit_writer(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage,
    void *frame
) {
    wf_file_request request;
    if (token_storage == NULL || frame == NULL
        || (buffer == NULL && count != 0)
        || file_offset > (uint64_t)INT64_MAX) {
        return 0;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            file_offset,
            (wf_completion_token *)token_storage,
            frame
        );
        if (native >= 0) {
            return native;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = (int64_t)file_offset;
    if ((uint64_t)request.operation.pread.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        frame
    );
}

int wf__completion_file_write_submit_writer(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage,
    void *frame
) {
    wf_file_request request;
    if (token_storage == NULL || frame == NULL
        || (buffer == NULL && count != 0)) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        frame
    );
}

/* True exactly for the result of an open the host refused because it had no
 * descriptor to give: the process limit and the system limit. */
static int wf_bridge_open_lacked_a_descriptor(const wf_file_result *result) {
    return result->kind == WF_FILE_OPEN_AT && result->value < 0
        && (result->error_code == EMFILE || result->error_code == ENFILE);
}

/* Waits for this runtime to give a descriptor back, then re-attempts one open
 * a blocking direct call had refused.
 *
 * The rule stated in contract.h, on the third route an open can take.  What is
 * different here is only how the waiting is done: this thread cannot sleep on
 * the ledger, because on a target with a kernel completion ring it may be the
 * only thread that can reap the very completion it is waiting for.  So it
 * waits by driving the engines and parking on the runtime's own endpoint,
 * which every publication already wakes.  The epoch is read before the state,
 * so a retirement that lands between the two makes the park return at once
 * instead of sleeping through it.
 *
 * Exactly one re-attempt, as on either target engine. */
static wf_file_result wf_bridge_retire_and_retry_direct(
    const wf_file_request *request,
    wf_file_result refused,
    uint64_t seen
) {
    wf_retirement_waiter waiter;
    enum wf_retirement_state state;
    wf_completion_retirement_wait_begin(&waiter, seen);
    for (;;) {
        uint64_t epoch = wf_bridge_ready == 0
            ? 0
            : wf_completion_wake_epoch(&wf_bridge_runtime);
        state = wf_completion_retirement_state(&waiter, 0);
        if (state != WF_RETIREMENT_AWAITED) {
            break;
        }
        if (wf_bridge_ready == 0) {
            /* No runtime to drive: whatever is in flight belongs to an
             * adapter this bridge did not build, and the ledger's own wake is
             * the only one there is. */
            wf_completion_retirement_sleep(&waiter, 0);
        } else {
            /* This open cannot retire while its own thread is inside the
             * engines, so nothing may wait for it to. */
            wf_completion_retirement_defer_begin();
            if (!wf_bridge_progress()) {
                wf_bridge_park(epoch);
            }
            wf_completion_retirement_defer_end();
        }
    }
    wf_completion_retirement_wait_end(&waiter);
    if (state != WF_RETIREMENT_HAPPENED) {
        return refused;
    }
    atomic_fetch_add_explicit(
        &wf_bridge_direct_exhaustion_retries,
        1,
        memory_order_relaxed
    );
    return wf_file_execute_direct(request);
}

/* One blocking direct host call, entered in the process-wide ledger.
 *
 * A direct call is an operation like any other while it runs: it can be
 * holding the descriptor a refused open elsewhere is waiting for, and when it
 * ends it may be giving one back.  Leaving it out of the ledger would make
 * both facts invisible and let another engine publish an exhaustion that this
 * call was about to answer. */
static wf_file_result wf_bridge_execute_direct(
    const wf_file_request *request
) {
    uint64_t seen = wf_completion_retirements();
    wf_file_result result;
    wf_completion_operation_accepted();
    result = wf_file_execute_direct(request);
    if (wf_bridge_open_lacked_a_descriptor(&result)) {
        result = wf_bridge_retire_and_retry_direct(request, result, seen);
    }
    wf_completion_operation_retired();
    /* A direct call publishes nothing, so nothing else tells a scheduler
     * parked on the completion endpoint that this runtime just gave a
     * descriptor back.  A held open waiting for exactly that is released by a
     * scheduler asking the ledger again, and this is what gets it there. */
    if (wf_bridge_ready != 0 && wf_completion_retirement_waiters() != 0) {
        wf_completion_notify_capacity(&wf_bridge_runtime);
    }
    return result;
}

static int64_t wf_bridge_return_direct(wf_file_result result) {
    if (result.value < 0) {
        errno = result.error_code;
    }
    return result.value;
}

int64_t wf__completion_file_pread_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t file_offset
) {
    wf_file_request request;
    /* The blocking host call below is outside every path the ring can see
     * through, so the doorbell rings first. */
    wf_bridge_flush_target();
#if defined(__linux__)
    if (file_offset >= 0) {
        wf_completion_token token;
        int submitted = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            (uint64_t)file_offset,
            &token,
            NULL
        );
        if (submitted == 1) {
            int64_t value;
            int error_code;
            wf__completion_file_join(&token, &value, &error_code);
            if (value < 0) {
                errno = error_code;
            }
            return value;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = file_offset;
    if ((uint64_t)request.operation.pread.count != count) {
        errno = EINVAL;
        return -1;
    }
    return wf_bridge_return_direct(wf_bridge_execute_direct(&request));
}

int64_t wf__completion_file_write_direct(
    int descriptor,
    const void *buffer,
    uint64_t count
) {
    wf_file_request request;
    /* The blocking host call below is outside every path the ring can see
     * through, so the doorbell rings first. */
    wf_bridge_flush_target();
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count) {
        errno = EINVAL;
        return -1;
    }
    return wf_bridge_return_direct(wf_bridge_execute_direct(&request));
}

int wf__completion_file_open_at_direct(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    int *error_code,
    unsigned *open_outcome
) {
    wf_file_request request;
    wf_file_result result;
    if (path == NULL || has_mode > 1u
        || expected_kind > WF_FILE_EXPECT_DIRECTORY || error_code == NULL
        || open_outcome == NULL) {
        errno = EINVAL;
        return -1;
    }
    /* The blocking host call below is outside every path the ring can see
     * through, so the doorbell rings first. */
    wf_bridge_flush_target();
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = directory;
    request.operation.open_at.path = path;
    request.operation.open_at.flags = flags;
    request.operation.open_at.mode = mode;
    request.operation.open_at.has_mode = has_mode;
    request.operation.open_at.expected_kind =
        (enum wf_file_expected_kind)expected_kind;
    result = wf_bridge_execute_direct(&request);
    *error_code = result.error_code;
    *open_outcome = (unsigned)result.open_outcome;
    return (int)wf_bridge_return_direct(result);
}

int wf__completion_file_status_direct(
    int descriptor,
    void *status,
    uint64_t status_capacity
) {
    wf_file_request request;
    wf_file_result result;
    /* The blocking host call below is outside every path the ring can see
     * through, so the doorbell rings first. */
    wf_bridge_flush_target();
    if (status == NULL
        || (uint64_t)(size_t)status_capacity != status_capacity) {
        errno = EINVAL;
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_STATUS;
    request.operation.status.descriptor = descriptor;
    result = wf_bridge_execute_direct(&request);
    if (result.value == 0) {
        if ((uint64_t)result.status_size > status_capacity) {
            errno = EOVERFLOW;
            return -1;
        }
        memcpy(status, result.status, result.status_size);
    }
    return (int)wf_bridge_return_direct(result);
}

int wf__completion_file_close_direct(int descriptor) {
    wf_file_request request;
    /* The blocking host call below is outside every path the ring can see
     * through, so the doorbell rings first. */
    wf_bridge_flush_target();
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = descriptor;
    return (int)wf_bridge_return_direct(wf_bridge_execute_direct(&request));
}


int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
) {
#if defined(__APPLE__)
    wf_file_request request;
    if ((buffer == NULL && count != 0) || position == NULL
        || (uint64_t)(size_t)count != count) {
        errno = EINVAL;
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_GETDIRENTRIES64;
    request.operation.getdirentries64.descriptor = descriptor;
    request.operation.getdirentries64.buffer = buffer;
    request.operation.getdirentries64.count = (size_t)count;
    request.operation.getdirentries64.position = position;
    return wf_bridge_return_direct(wf_bridge_execute_direct(&request));
#else
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)position;
    errno = ENOTSUP;
    return -1;
#endif
}

static int wf_bridge_take_file_result(
    const void *token_storage,
    wf_file_result *file_result
) {
    wf_completion_token token;
    enum wf_bridge_route route;
    union {
        wf_file_result file;
#if defined(__linux__)
        /* Not `linux`: GNU C predefines that spelling as an object-like macro
           on a Linux host outside a strict `-std=cNN` dialect, so a member of
           that name compiles under the gate's `-std=c11` and fails wherever
           the default dialect is in force. */
        wf_linux_file_result ring;
#endif
        max_align_t alignment;
        unsigned char bytes[WF_COMPLETION_RESULT_CAPACITY];
    } result;
    wf_completion_outcome outcome;
    enum wf_completion_consume_result consumed;
    if (token_storage == NULL || file_result == NULL) {
        abort();
    }
    token = *(const wf_completion_token *)token_storage;
    (void)wf_bridge_drain();
    consumed = wf_completion_consume(
        &wf_bridge_runtime,
        token,
        &result,
        sizeof(result),
        &outcome
    );
    if (consumed == WF_COMPLETION_CONSUME_NOT_TERMINAL
        || consumed == WF_COMPLETION_CONSUME_NOT_DRAINED) {
        return 0;
    }
    if (consumed != WF_COMPLETION_CONSUMED) {
        abort();
    }
    route = (enum wf_bridge_route)outcome.adapter_tag;
    if (route == WF_BRIDGE_ROUTE_FILE_FALLBACK) {
        if (outcome.result_size != sizeof(result.file)) {
            abort();
        }
        *file_result = result.file;
#if defined(__linux__)
    } else if (route == WF_BRIDGE_ROUTE_LINUX_IO_URING) {
        if (outcome.result_size != sizeof(result.ring)) {
            abort();
        }
        memset(file_result, 0, sizeof(*file_result));
        switch (result.ring.kind) {
        case WF_LINUX_FILE_READ_AT:
            file_result->kind = WF_FILE_PREAD;
            break;
        case WF_LINUX_FILE_WRITE_AT:
            file_result->kind = WF_FILE_PWRITE;
            break;
        case WF_LINUX_FILE_OPEN_AT:
            file_result->kind = WF_FILE_OPEN_AT;
            break;
        case WF_LINUX_FILE_CLOSE:
            file_result->kind = WF_FILE_CLOSE;
            break;
        default:
            abort();
        }
        file_result->value = result.ring.value;
        file_result->error_code = result.ring.error_code;
        file_result->open_outcome = result.ring.open_outcome;
#endif
    } else {
        abort();
    }
    return 1;
}

int wf__completion_file_take(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    wf_file_result result;
    if (value == NULL || error_code == NULL) {
        abort();
    }
    if (!wf_bridge_take_file_result(token_storage, &result)) {
        return 0;
    }
    *value = result.value;
    *error_code = result.error_code;
    return 1;
}

int wf__completion_file_take_status(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
) {
    wf_file_result result;
    if (value == NULL || error_code == NULL || status == NULL
        || status_size == NULL
        || (uint64_t)(size_t)status_capacity != status_capacity) {
        abort();
    }
    if (!wf_bridge_take_file_result(token_storage, &result)) {
        return 0;
    }
    if (result.kind != WF_FILE_STATUS
        || (uint64_t)result.status_size > status_capacity) {
        abort();
    }
    if (result.status_size != 0) {
        memcpy(status, result.status, result.status_size);
    }
    *value = result.value;
    *error_code = result.error_code;
    *status_size = (uint64_t)result.status_size;
    return 1;
}

static int wf_bridge_take_open(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
) {
    wf_file_result result;
    if (value == NULL || error_code == NULL || open_outcome == NULL) {
        abort();
    }
    if (!wf_bridge_take_file_result(token_storage, &result)) {
        return 0;
    }
    if (result.kind != WF_FILE_OPEN_AT) {
        abort();
    }
    *value = result.value;
    *error_code = result.error_code;
    *open_outcome = (unsigned)result.open_outcome;
    return 1;
}

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        if (wf__completion_file_take(token_storage, value, error_code)) {
            return;
        }
        if (!wf_bridge_progress()) {
            if (wf_bridge_drain() != 0) {
                continue;
            }
            if (wf_completion_ready_event_count(&wf_bridge_runtime) == 0
                && !wf_bridge_target_work_needs_this_thread()) {
                enum wf_completion_consume_wait_result wait =
                    wf_completion_wait_to_consume(
                        &wf_bridge_runtime,
                        *(const wf_completion_token *)token_storage
                    );
                if (wait == WF_COMPLETION_CONSUME_READY) {
                    continue;
                }
                if (wait != WF_COMPLETION_CONSUME_WAIT_REGISTERED) {
                    abort();
                }
                wf_bridge_park(epoch);
            }
        }
    }
}

void wf__completion_file_open_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
) {
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        if (wf_bridge_take_open(
                token_storage,
                value,
                error_code,
                open_outcome
            )) {
            return;
        }
        if (!wf_bridge_progress()) {
            if (wf_bridge_drain() != 0) {
                continue;
            }
            if (wf_completion_ready_event_count(&wf_bridge_runtime) == 0
                && !wf_bridge_target_work_needs_this_thread()) {
                enum wf_completion_consume_wait_result wait =
                    wf_completion_wait_to_consume(
                        &wf_bridge_runtime,
                        *(const wf_completion_token *)token_storage
                    );
                if (wait == WF_COMPLETION_CONSUME_READY) {
                    continue;
                }
                if (wait != WF_COMPLETION_CONSUME_WAIT_REGISTERED) {
                    abort();
                }
                wf_bridge_park(epoch);
            }
        }
    }
}

void wf__completion_file_status_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
) {
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        if (wf__completion_file_take_status(
                token_storage,
                value,
                error_code,
                status,
                status_capacity,
                status_size
            )) {
            return;
        }
        if (!wf_bridge_progress()) {
            if (wf_bridge_drain() != 0) {
                continue;
            }
            if (wf_completion_ready_event_count(&wf_bridge_runtime) == 0
                && !wf_bridge_target_work_needs_this_thread()) {
                enum wf_completion_consume_wait_result wait =
                    wf_completion_wait_to_consume(
                        &wf_bridge_runtime,
                        *(const wf_completion_token *)token_storage
                    );
                if (wait == WF_COMPLETION_CONSUME_READY) {
                    continue;
                }
                if (wait != WF_COMPLETION_CONSUME_WAIT_REGISTERED) {
                    abort();
                }
                wf_bridge_park(epoch);
            }
        }
    }
}

void wf__writer_run_root(void *frame) {
    while (!wf__writer_is_done(frame)) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        int progressed = wf_bridge_progress();
        progressed |= wf__writer_scheduler_help_once();
        if (!progressed && !wf__writer_is_done(frame)) {
            /* A bounded drain can stop before another ready slot. Keep
             * harvesting while the durable count says such an event exists;
             * drain-to-writer enqueue advances the epoch and closes the final
             * ready-count-to-park race. */
            if (wf_completion_ready_event_count(&wf_bridge_runtime) != 0) {
                continue;
            }
            wf_bridge_park(epoch);
        }
    }
}

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

/* Opens the host refused for want of a descriptor that this runtime gave one
 * back for and re-attempted once, over every route an open can take. */
uint64_t wf__completion_open_exhaustion_retries(void) {
    uint64_t retries = atomic_load_explicit(
        &wf_bridge_direct_exhaustion_retries,
        memory_order_relaxed
    );
    retries += wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter)
            .exhaustion_retries;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        retries += wf_linux_io_uring_statistics_snapshot(
            &wf_bridge_linux_adapter
        ).exhaustion_retries;
    }
#endif
    return retries;
}

/* Opens this runtime held rather than published, over every route, because
 * something in flight could still give the descriptor they needed back.  A
 * test that has to stand at exactly that moment reads this; nothing in the
 * runtime decides anything on it. */
uint64_t wf__completion_open_exhaustion_waits(void) {
    return wf_completion_retirement_waits();
}

uint64_t wf__completion_file_demoted_opens(void) {
    return atomic_load_explicit(&wf_bridge_demoted_opens, memory_order_relaxed);
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
    return wf_completion_statistics_snapshot(&wf_bridge_runtime).publications;
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
