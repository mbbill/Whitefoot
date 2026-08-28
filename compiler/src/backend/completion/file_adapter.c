#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* F_NOCACHE, the Darwin half of the WF_IO_NOCACHE target policy, is a BSD
 * extension the POSIX macro above hides, exactly as in bridge.c. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

#include "file_adapter.h"

#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <poll.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#if !defined(WF_COMPLETION_POLL)
#define WF_COMPLETION_POLL poll
#else
extern int WF_COMPLETION_POLL(struct pollfd *, nfds_t, int);
#endif

#if !defined(WF_COMPLETION_PREAD)
#define WF_COMPLETION_PREAD pread
#else
extern ssize_t WF_COMPLETION_PREAD(int, void *, size_t, off_t);
#endif

#if defined(__APPLE__)
/* libSystem exports the exact facility used by the qualified Darwin target;
 * it is intentionally not replaced with an opendir/readdir loop. */
#if !defined(WF_COMPLETION_GETDIRENTRIES64)
#define WF_COMPLETION_GETDIRENTRIES64 __getdirentries64
#endif
extern ssize_t WF_COMPLETION_GETDIRENTRIES64(
    int,
    void *,
    size_t,
    int64_t *
);
#endif

_Static_assert(
    sizeof(struct stat) <= WF_FILE_STATUS_CAPACITY,
    "WF_FILE_STATUS_CAPACITY must hold the host stat record"
);
_Static_assert(
    sizeof(wf_file_result) <= WF_COMPLETION_RESULT_CAPACITY,
    "completion result cell must hold a file result"
);

static int wf_file_request_valid(const wf_file_request *request) {
    if (request == NULL) {
        return 0;
    }
    switch (request->kind) {
    case WF_FILE_OPEN_AT:
        return request->operation.open_at.path != NULL
            && request->operation.open_at.expected_kind
                <= WF_FILE_EXPECT_DIRECTORY;
    case WF_FILE_READ:
        return request->operation.read.buffer != NULL
            || request->operation.read.count == 0;
    case WF_FILE_WRITE:
        return request->operation.write.buffer != NULL
            || request->operation.write.count == 0;
    case WF_FILE_PREAD:
        return request->operation.pread.buffer != NULL
            || request->operation.pread.count == 0;
    case WF_FILE_PWRITE:
        return request->operation.pwrite.buffer != NULL
            || request->operation.pwrite.count == 0;
    case WF_FILE_STATUS:
    case WF_FILE_CLOSE:
        return 1;
#if defined(__APPLE__)
    case WF_FILE_GETDIRENTRIES64:
        return (request->operation.getdirentries64.buffer != NULL
                || request->operation.getdirentries64.count == 0)
            && request->operation.getdirentries64.position != NULL;
#endif
    default:
        return 0;
    }
}

static wf_file_result wf_file_execute_once(const wf_file_request *request) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.kind = request->kind;
    result.value = -1;

    switch (request->kind) {
    case WF_FILE_READ:
        if (request->operation.read.count > (size_t)SSIZE_MAX) {
            result.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_WRITE:
        if (request->operation.write.count > (size_t)SSIZE_MAX) {
            result.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PREAD:
        /* An empty transfer has no external action.  This must be decided
         * before descriptor and offset validation reaches the host: callers
         * rely on zero length completing without touching
         * either the destination or the file facility. */
        if (request->operation.pread.count == 0) {
            result.value = 0;
            return result;
        }
        if (request->operation.pread.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pread.offset
                != request->operation.pread.offset) {
            result.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PWRITE:
        if (request->operation.pwrite.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pwrite.offset
                != request->operation.pwrite.offset) {
            result.error_code = EINVAL;
            return result;
        }
        break;
#if defined(__APPLE__)
    case WF_FILE_GETDIRENTRIES64:
        if (request->operation.getdirentries64.count > (size_t)SSIZE_MAX) {
            result.error_code = EINVAL;
            return result;
        }
        break;
#endif
    default:
        break;
    }

    /* Each pass is one qualified host attempt. No-progress interruption and
     * readiness refusal are absorbed by wf_file_execute_direct below and
     * never become writer-visible outcomes. */
    switch (request->kind) {
    case WF_FILE_OPEN_AT:
        if (request->operation.open_at.expected_kind
            > WF_FILE_EXPECT_DIRECTORY) {
            result.error_code = EINVAL;
            result.open_outcome = WF_FILE_OPEN_FAILED;
            return result;
        }
        if (request->operation.open_at.has_mode != 0) {
            result.value = openat(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      ),
                (mode_t)request->operation.open_at.mode
            );
        } else {
            result.value = openat(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      )
            );
        }
        if (result.value < 0) {
            result.open_outcome = WF_FILE_OPEN_FAILED;
            break;
        }
        if (request->operation.open_at.expected_kind != WF_FILE_EXPECT_ANY) {
            struct stat status;
            int descriptor = (int)result.value;
            if (fstat(descriptor, &status) != 0) {
                int status_error = errno;
                (void)close(descriptor);
                result.error_code = status_error;
                result.open_outcome = WF_FILE_OPEN_STATUS_FAILED;
                return result;
            }
            result.open_outcome = wf_file_kind_outcome(
                request->operation.open_at.expected_kind,
                (unsigned int)status.st_mode
            );
            if (result.open_outcome != WF_FILE_OPEN_SUCCEEDED) {
                (void)close(descriptor);
                return result;
            }
        }
        /* The descriptor is the one this open hands back: the kind check has
         * either passed or was not asked for. WF_IO_NOCACHE is applied here,
         * once, so a refused descriptor never receives it. */
        wf_file_apply_uncached_reads((int)result.value);
        break;
    case WF_FILE_READ:
        result.value = read(
            request->operation.read.descriptor,
            request->operation.read.buffer,
            request->operation.read.count
        );
        break;
    case WF_FILE_WRITE:
        result.value = write(
            request->operation.write.descriptor,
            request->operation.write.buffer,
            request->operation.write.count
        );
        break;
    case WF_FILE_PREAD:
        result.value = WF_COMPLETION_PREAD(
            request->operation.pread.descriptor,
            request->operation.pread.buffer,
            request->operation.pread.count,
            (off_t)request->operation.pread.offset
        );
        break;
    case WF_FILE_PWRITE:
        result.value = pwrite(
            request->operation.pwrite.descriptor,
            request->operation.pwrite.buffer,
            request->operation.pwrite.count,
            (off_t)request->operation.pwrite.offset
        );
        break;
    case WF_FILE_STATUS: {
        struct stat status;
        result.value = fstat(request->operation.status.descriptor, &status);
        if (result.value == 0) {
            result.status_size = sizeof(status);
            memcpy(result.status, &status, sizeof(status));
        }
        break;
    }
    case WF_FILE_CLOSE:
        result.value = close(request->operation.close.descriptor);
        break;
#if defined(__APPLE__)
    case WF_FILE_GETDIRENTRIES64:
        result.value = WF_COMPLETION_GETDIRENTRIES64(
            request->operation.getdirentries64.descriptor,
            request->operation.getdirentries64.buffer,
            request->operation.getdirentries64.count,
            request->operation.getdirentries64.position
        );
        break;
#endif
    default:
        result.error_code = EINVAL;
        return result;
    }
    if (result.value < 0) {
        result.error_code = errno;
    }
    return result;
}

static int wf_file_wait_ready(const wf_file_request *request) {
    struct pollfd descriptor;
    int waited;
    memset(&descriptor, 0, sizeof(descriptor));
    switch (request->kind) {
    case WF_FILE_READ:
        descriptor.fd = request->operation.read.descriptor;
        descriptor.events = POLLIN;
        break;
    case WF_FILE_PREAD:
        descriptor.fd = request->operation.pread.descriptor;
        descriptor.events = POLLIN;
        break;
#if defined(__APPLE__)
    case WF_FILE_GETDIRENTRIES64:
        descriptor.fd = request->operation.getdirentries64.descriptor;
        descriptor.events = POLLIN;
        break;
#endif
    case WF_FILE_WRITE:
        descriptor.fd = request->operation.write.descriptor;
        descriptor.events = POLLOUT;
        break;
    case WF_FILE_PWRITE:
        descriptor.fd = request->operation.pwrite.descriptor;
        descriptor.events = POLLOUT;
        break;
    default:
        return EINVAL;
    }
    do {
        waited = WF_COMPLETION_POLL(&descriptor, 1, -1);
    } while (waited < 0 && errno == EINTR);
    return waited > 0 ? 0 : errno;
}

wf_file_result wf_file_execute_direct(const wf_file_request *request) {
    wf_file_result result;
    if (!wf_file_request_valid(request)) {
        memset(&result, 0, sizeof(result));
        result.kind = request == NULL ? 0 : request->kind;
        result.value = -1;
        result.error_code = EINVAL;
        return result;
    }
    for (;;) {
        int readiness_error;
        result = wf_file_execute_once(request);
        if (result.value >= 0 || result.error_code == 0) {
            return result;
        }
        switch (request->kind) {
        case WF_FILE_READ:
        case WF_FILE_WRITE:
        case WF_FILE_PREAD:
        case WF_FILE_PWRITE:
#if defined(__APPLE__)
        case WF_FILE_GETDIRENTRIES64:
#endif
            break;
        default:
            return result;
        }
        if (result.error_code == EINTR) {
            continue;
        }
        if (result.error_code != EAGAIN
#if defined(EWOULDBLOCK) && EWOULDBLOCK != EAGAIN
            && result.error_code != EWOULDBLOCK
#endif
        ) {
            return result;
        }
        readiness_error = wf_file_wait_ready(request);
        if (readiness_error != 0) {
            result.error_code = readiness_error;
            return result;
        }
    }
}

/* The measured host-call time above which handing an operation to another
 * thread is worth the handoff.
 *
 * Below it the operation does not wait: the host answers from memory, and a
 * helper can only add a queue crossing and two thread wakes to work that was
 * already done by the time the wake arrived.  This is the number that decides
 * whether the pool grows at all, and it is a measurement of the program's own
 * operations rather than an assumption about the host.
 *
 * Twenty microseconds is about ten times what one handoff costs on the host
 * this was measured on — a wake on the submitting thread, a wake-up on the
 * helper, and a queue crossing.  Sizing it at the handoff itself would admit
 * every operation whose overlap merely breaks even, and a pool that breaks
 * even still costs the CPU it spends.  Every population this project has
 * measured falls clearly on one side: a warm 4 KiB read is about 1 us, a warm
 * 64 KiB read 5 to 7, and a read that reaches the device 130 or more. */
#define WF_FILE_OVERLAP_WAIT_NS 20000u

/* How often an execution is timed for the average above: one in this many.
 * The clock is read twice per sampled execution and not at all otherwise. */
#define WF_FILE_EXECUTE_SAMPLE_INTERVAL 16u

static uint64_t wf_file_monotonic_ns(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return 0;
    }
    return (uint64_t)now.tv_sec * 1000000000u + (uint64_t)now.tv_nsec;
}

/* Records one host call's duration into the smoothed average, from one in
 * every WF_FILE_EXECUTE_SAMPLE_INTERVAL executions.  Two threads that sample
 * at once may lose one another's contribution; the average is a policy hint
 * and no operation's outcome depends on it. */
static void wf_file_note_execution(wf_file_adapter *adapter, uint64_t started) {
    uint64_t mean;
    uint64_t sample = wf_file_monotonic_ns();
    sample = sample > started ? sample - started : 0u;
    mean = atomic_load_explicit(&adapter->mean_execute_ns, memory_order_relaxed);
    mean = mean == 0 ? sample : mean - mean / 4u + sample / 4u;
    atomic_store_explicit(&adapter->mean_execute_ns, mean, memory_order_relaxed);
}

static int wf_file_execution_should_be_timed(wf_file_adapter *adapter) {
    uint64_t tick = atomic_fetch_add_explicit(
        &adapter->execute_ticks,
        1,
        memory_order_relaxed
    );
    return tick % WF_FILE_EXECUTE_SAMPLE_INTERVAL == 0;
}

/* Moves one queue entry into the executing thread's own storage.
 *
 * The record is copied rather than executed in place because the queue slot
 * becomes free the moment the lock is released.  Only an open needs its path
 * bytes, and only an open pays for them: copying the whole record moved a
 * kilobyte of path storage for every read and write as well, inside the one
 * lock every submission and every execution has to take. */
static void wf_file_copy_out(wf_file_work *work, const wf_file_work *entry) {
    work->token = entry->token;
    work->request = entry->request;
    if (entry->request.kind == WF_FILE_OPEN_AT) {
        (void)wf_file_stage_path(work->path_storage, entry->path_storage);
        wf_file_work_bind_path(work);
    }
    work->queued_ns = entry->queued_ns;
}

static int wf_file_take_work(wf_file_adapter *adapter, wf_file_work *work) {
    int present = 0;
    int released_capacity = 0;
    (void)pthread_mutex_lock(&adapter->queue_lock);
    if (adapter->queue_count != 0) {
        released_capacity = adapter->queue_count == adapter->queue_capacity;
        /* The scheduler is the only progress source when helper_count is
         * zero. Take the newest independent request so a blocked earlier host
         * facility cannot hide every later free operation behind it. Helper
         * threads take the oldest request below, so multiple target contexts
         * naturally spread across both ends of the bounded queue. */
        adapter->queue_tail = adapter->queue_tail == 0
            ? adapter->queue_capacity - 1
            : adapter->queue_tail - 1;
        wf_file_copy_out(work, &adapter->queue[adapter->queue_tail]);
        adapter->queue_count -= 1;
        present = 1;
    }
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    if (released_capacity != 0) {
        wf_completion_notify_capacity(adapter->runtime);
    }
    return present;
}

static void wf_file_count_execution(wf_file_adapter *adapter, int helper) {
    atomic_fetch_add_explicit(
        helper != 0 ? &adapter->stat_helper_executions
                    : &adapter->stat_scheduler_executions,
        1,
        memory_order_relaxed
    );
}

static void wf_file_run_work(
    wf_file_adapter *adapter,
    const wf_file_work *work,
    int helper
) {
    wf_file_result result;
    uint64_t traced = 0;
    if (wf_completion_trace_enabled()) {
        traced = wf_completion_trace_now();
        if (work->queued_ns != 0) {
            atomic_fetch_add_explicit(
                &wf__completion_trace.queue_latency_ns,
                traced > work->queued_ns ? traced - work->queued_ns : 0u,
                memory_order_relaxed
            );
            atomic_fetch_add_explicit(
                &wf__completion_trace.queue_latency_count,
                1,
                memory_order_relaxed
            );
        }
    }
    {
        int timed = wf_file_execution_should_be_timed(adapter);
        uint64_t started = timed ? wf_file_monotonic_ns() : 0u;
        result = wf_file_execute_direct(&work->request);
        if (timed) {
            wf_file_note_execution(adapter, started);
        }
    }
    if (traced != 0) {
        wf_completion_trace_add(
            &wf__completion_trace.execute_ns,
            &wf__completion_trace.execute_count,
            traced
        );
    }
    wf_completion_publication publication = {
        .milestones = WF_COMPLETION_OWNERSHIP_COMPLETE,
        .terminal_kind = result.error_code == 0
                && result.open_outcome == WF_FILE_OPEN_SUCCEEDED
            ? 1u
            : 2u,
        .result = &result,
        .result_size = sizeof(result),
    };
    enum wf_completion_publish_result published = wf_completion_publish_terminal(
        adapter->runtime,
        work->token,
        &publication
    );
    wf_file_count_execution(adapter, helper);
    if (published != WF_COMPLETION_PUBLISHED) {
        /* A legitimate accepted work item owns the unique terminal route.  A
         * failure here records an adapter/core defect; it never invokes writer
         * code or attempts a second publication. */
        atomic_fetch_add_explicit(
            &adapter->stat_publication_failures,
            1,
            memory_order_relaxed
        );
    }
}

static void *wf_file_helper_main(void *context) {
    wf_file_adapter *adapter = context;
    for (;;) {
        wf_file_work work;
        int released_capacity;
        (void)pthread_mutex_lock(&adapter->queue_lock);
        while (adapter->queue_count == 0 && adapter->stopping == 0) {
            if (wf_completion_trace_enabled()) {
                wf_completion_trace_count(
                    &wf__completion_trace.helper_blocks,
                    1
                );
            }
            adapter->blocked_helpers += 1;
            (void)pthread_cond_wait(
                &adapter->queue_available,
                &adapter->queue_lock
            );
            adapter->blocked_helpers -= 1;
        }
        if (adapter->queue_count == 0 && adapter->stopping != 0) {
            (void)pthread_mutex_unlock(&adapter->queue_lock);
            return NULL;
        }
        released_capacity = adapter->queue_count == adapter->queue_capacity;
        wf_file_copy_out(&work, &adapter->queue[adapter->queue_head]);
        adapter->queue_head = (adapter->queue_head + 1) % adapter->queue_capacity;
        adapter->queue_count -= 1;
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        if (released_capacity != 0) {
            wf_completion_notify_capacity(adapter->runtime);
        }
        wf_file_run_work(adapter, &work, 1);
    }
}

int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_file_work *queue_storage,
    size_t queue_capacity,
    pthread_t *helper_storage,
    size_t helper_count
) {
    size_t created = 0;
    int error;

    if (adapter == NULL || runtime == NULL || queue_storage == NULL
        || queue_capacity == 0 || (helper_count != 0 && helper_storage == NULL)) {
        return EINVAL;
    }
    memset(adapter, 0, sizeof(*adapter));
    adapter->runtime = runtime;
    adapter->queue = queue_storage;
    adapter->queue_capacity = queue_capacity;
    adapter->helpers = helper_storage;
    atomic_init(&adapter->helper_count, 0);
    atomic_init(&adapter->mean_execute_ns, 0);
    atomic_init(&adapter->execute_ticks, 0);
    adapter->blocked_helpers = 0;
    adapter->helper_cap = helper_count;
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_capacity_waits, 0);
    atomic_init(&adapter->stat_helper_executions, 0);
    atomic_init(&adapter->stat_scheduler_executions, 0);
    atomic_init(&adapter->stat_publication_failures, 0);

    error = pthread_mutex_init(&adapter->queue_lock, NULL);
    if (error != 0) {
        return error;
    }
    error = pthread_cond_init(&adapter->queue_available, NULL);
    if (error != 0) {
        (void)pthread_mutex_destroy(&adapter->queue_lock);
        return error;
    }
    adapter->initialized = 1;

    for (created = 0; created < helper_count; ++created) {
        error = pthread_create(
            &helper_storage[created],
            NULL,
            wf_file_helper_main,
            adapter
        );
        if (error != 0) {
            size_t join;
            (void)pthread_mutex_lock(&adapter->queue_lock);
            adapter->stopping = 1;
            (void)pthread_cond_broadcast(&adapter->queue_available);
            (void)pthread_mutex_unlock(&adapter->queue_lock);
            for (join = 0; join < created; ++join) {
                (void)pthread_join(helper_storage[join], NULL);
            }
            (void)pthread_cond_destroy(&adapter->queue_available);
            (void)pthread_mutex_destroy(&adapter->queue_lock);
            adapter->initialized = 0;
            return error;
        }
        atomic_store_explicit(
            &adapter->helper_count,
            created + 1,
            memory_order_release
        );
    }
    return 0;
}

int wf_file_adapter_set_helper_cap(wf_file_adapter *adapter, size_t cap) {
    if (adapter == NULL || adapter->initialized == 0
        || (cap != 0 && adapter->helpers == NULL)) {
        return EINVAL;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    adapter->helper_cap = cap;
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    return 0;
}

size_t wf_file_adapter_helper_count(const wf_file_adapter *adapter) {
    if (adapter == NULL || adapter->initialized == 0) {
        return 0;
    }
    return atomic_load_explicit(&adapter->helper_count, memory_order_acquire);
}

/* Adds one helper when the queue holds more accepted requests than there are
 * helpers to take them **and** this program's operations wait long enough for
 * another thread to be worth its handoff.
 *
 * Queue depth alone is not enough, and measuring it was what made this pool
 * cost more than it saved.  Depth says a program stated width; it does not say
 * the width is over anything.  A program reading a warm page cache states the
 * same eight independent reads as one reading a device and exposes the same
 * queue, but each of its reads is a memory copy that is finished before a
 * woken helper is scheduled — so every helper the depth rule added there
 * bought a wake and a queue crossing and overlapped nothing.  The measured
 * host-call time is the missing half of the condition, and it is a
 * measurement of the program's own operations rather than a guess about the
 * host.
 *
 * A rule that instead grew whenever a submission found no helper *waiting*
 * was tried and measured worse: a helper that has been signalled but not yet
 * scheduled still counts as waiting, so a run of consecutive submissions sees
 * one available helper every time and the pool never grows at all. On the
 * quiet macOS host that left the four-wide program at 919 ms against 625 ms
 * for the depth rule.  Queue depth is a lagging signal, but it is a true one.
 *
 * Growth starts from no helpers at all, because a program whose operations do
 * not wait wants none: with an empty pool the waiting scheduler is the queue's
 * engine and the completion path costs a queue crossing and nothing else.  It
 * runs on the submitting thread with the queue lock already held, so at most
 * one helper appears per submission and the count never passes the policy's
 * cap. A pinned helper policy sets the cap equal to the initial count, making
 * this a no-op. */
static void wf_file_grow_helpers_locked(wf_file_adapter *adapter) {
    size_t held = atomic_load_explicit(
        &adapter->helper_count,
        memory_order_relaxed
    );
    if (held >= adapter->helper_cap
        || adapter->queue_count <= held) {
        return;
    }
    if (atomic_load_explicit(&adapter->mean_execute_ns, memory_order_relaxed)
        < WF_FILE_OVERLAP_WAIT_NS) {
        return;
    }
    if (pthread_create(
            &adapter->helpers[held],
            NULL,
            wf_file_helper_main,
            adapter
        ) != 0) {
        return;
    }
    atomic_store_explicit(
        &adapter->helper_count,
        held + 1,
        memory_order_release
    );
}

/* Appends one accepted queue entry, announces it to one helper, and grows the
 * pool when the queue has outrun it.  The caller holds the queue lock and has
 * already made the entry's ownership transition. */
static void wf_file_enqueue_locked(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
) {
    wf_file_work *entry = &adapter->queue[adapter->queue_tail];
    entry->token = token;
    entry->request = *request;
    entry->queued_ns = wf_completion_trace_enabled()
        ? wf_completion_trace_now()
        : 0u;
    if (request->kind == WF_FILE_OPEN_AT) {
        /* The path becomes this record's own before any helper can see the
         * entry.  `wf_file_adapter_submit` refused a name that does not fit
         * before the operation was claimed, so this copy cannot fail. */
        (void)wf_file_stage_path(
            entry->path_storage,
            request->operation.open_at.path
        );
        wf_file_work_bind_path(entry);
        /* [SYS-2]'s `loan-released(path)` for this open holds from here: the
         * name the host will resolve is the record's own, so the caller's
         * buffer is free whatever the host does next.  Publishing it makes
         * that an observable fact rather than an adapter comment.  A refusal
         * is an adapter/core contract defect: it is counted, never retried,
         * and never turned into writer-visible behaviour. */
        if (wf_completion_publish_milestone(
                adapter->runtime,
                token,
                WF_COMPLETION_PAYLOAD_RELEASED
            ) != WF_COMPLETION_PUBLISHED) {
            atomic_fetch_add_explicit(
                &adapter->stat_publication_failures,
                1,
                memory_order_relaxed
            );
        }
    }
    adapter->queue_tail = (adapter->queue_tail + 1) % adapter->queue_capacity;
    adapter->queue_count += 1;
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );
    /* One newly queued request needs exactly one helper woken, and only a
     * helper that is actually asleep needs a host wake at all.  Announcing to
     * every helper would cost a wake per helper per submission — a thundering
     * herd rather than progress — and announcing to a helper still spinning
     * for work costs a system call to wake a thread that is awake.  This lock
     * is the one the sleeper counts itself under, so the count read here is
     * exact rather than a guess. */
    if (adapter->blocked_helpers != 0) {
        if (wf_completion_trace_enabled()) {
            wf_completion_trace_count(&wf__completion_trace.helper_signals, 1);
        }
        (void)pthread_cond_signal(&adapter->queue_available);
    }
    wf_file_grow_helpers_locked(adapter);
}

enum wf_file_submit_result wf_file_adapter_submit(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
) {
    enum wf_completion_transition_result transition;
    uint64_t traced = 0;

    if (adapter == NULL || adapter->initialized == 0
        || !wf_file_request_valid(request)) {
        return WF_FILE_SUBMIT_INVALID;
    }
    if (wf_completion_trace_enabled()) {
        traced = wf_completion_trace_now();
    }
    /* A submitted open's path must fit the operation record, because the
     * record is what resolves it.  The direct executor carries any length and
     * needs no copy, so refusing here before anything is claimed leaves the
     * caller an honest fallback rather than a truncated name. */
    if (request->kind == WF_FILE_OPEN_AT
        && !wf_file_path_fits(request->operation.open_at.path)) {
        return WF_FILE_SUBMIT_INVALID;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    if (adapter->stopping != 0) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_ADAPTER_STOPPING;
    }
    transition = wf_completion_begin_submit(adapter->runtime, token);
    if (transition == WF_COMPLETION_TRANSITION_STALE) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_SUBMIT_STALE;
    }
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_SUBMIT_INVALID;
    }
    if (adapter->queue_count == adapter->queue_capacity) {
        (void)wf_completion_mark_wait_capacity(adapter->runtime, token);
        atomic_fetch_add_explicit(
            &adapter->stat_capacity_waits,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_WAIT_CAPACITY;
    }
    transition = wf_completion_target_accepted(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_FILE_SUBMIT_STALE
            : WF_FILE_SUBMIT_INVALID;
    }

    wf_file_enqueue_locked(adapter, token, request);
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    if (traced != 0) {
        wf_completion_trace_add(
            &wf__completion_trace.submit_ns,
            &wf__completion_trace.submit_count,
            traced
        );
    }
    return WF_FILE_TARGET_OWNS;
}

size_t wf_file_adapter_progress(wf_file_adapter *adapter, size_t budget) {
    size_t executed = 0;
    if (adapter == NULL || adapter->initialized == 0) {
        return 0;
    }
    while (executed < budget) {
        wf_file_work work;
        if (!wf_file_take_work(adapter, &work)) {
            break;
        }
        wf_file_run_work(adapter, &work, 0);
        executed += 1;
    }
    return executed;
}

size_t wf_file_adapter_queued(const wf_file_adapter *adapter) {
    size_t queued;
    wf_file_adapter *mutable_adapter = (wf_file_adapter *)adapter;
    if (adapter == NULL || adapter->initialized == 0) {
        return 0;
    }
    (void)pthread_mutex_lock(&mutable_adapter->queue_lock);
    queued = mutable_adapter->queue_count;
    (void)pthread_mutex_unlock(&mutable_adapter->queue_lock);
    return queued;
}

int wf_file_adapter_shutdown(wf_file_adapter *adapter) {
    size_t helper;
    size_t held;
    int first_error = 0;
    if (adapter == NULL || adapter->initialized == 0) {
        return EINVAL;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    adapter->stopping = 1;
    /* Growth stops with admission, so the count read below is final. */
    adapter->helper_cap = 0;
    (void)pthread_cond_broadcast(&adapter->queue_available);
    (void)pthread_mutex_unlock(&adapter->queue_lock);

    held = atomic_load_explicit(&adapter->helper_count, memory_order_acquire);
    if (held == 0) {
        while (wf_file_adapter_progress(adapter, 1) != 0) {
        }
    }
    for (helper = 0; helper < held; ++helper) {
        int error = pthread_join(adapter->helpers[helper], NULL);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    {
        int error = pthread_cond_destroy(&adapter->queue_available);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    {
        int error = pthread_mutex_destroy(&adapter->queue_lock);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    if (first_error == 0) {
        adapter->initialized = 0;
    }
    return first_error;
}

wf_file_adapter_statistics wf_file_adapter_statistics_snapshot(
    const wf_file_adapter *adapter
) {
    wf_file_adapter_statistics statistics = {0};
    if (adapter == NULL) {
        return statistics;
    }
    statistics.submissions = atomic_load_explicit(
        &adapter->stat_submissions,
        memory_order_relaxed
    );
    statistics.capacity_waits = atomic_load_explicit(
        &adapter->stat_capacity_waits,
        memory_order_relaxed
    );
    statistics.helper_executions = atomic_load_explicit(
        &adapter->stat_helper_executions,
        memory_order_relaxed
    );
    statistics.scheduler_executions = atomic_load_explicit(
        &adapter->stat_scheduler_executions,
        memory_order_relaxed
    );
    statistics.publication_failures = atomic_load_explicit(
        &adapter->stat_publication_failures,
        memory_order_relaxed
    );
    return statistics;
}
