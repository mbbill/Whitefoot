#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
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
        return request->operation.open_at.path != NULL;
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
        if (request->operation.open_at.has_mode != 0) {
            result.value = openat(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags,
                (mode_t)request->operation.open_at.mode
            );
        } else {
            result.value = openat(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
            );
        }
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
        *work = adapter->queue[adapter->queue_tail];
        adapter->queue_count -= 1;
        present = 1;
    }
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    if (released_capacity != 0) {
        wf_completion_notify_capacity(adapter->runtime);
    }
    return present;
}

static void wf_file_run_work(
    wf_file_adapter *adapter,
    const wf_file_work *work,
    int helper
) {
    wf_file_result result = wf_file_execute_direct(&work->request);
    wf_completion_publication publication = {
        .milestones = WF_COMPLETION_OWNERSHIP_COMPLETE,
        .terminal_kind = result.error_code == 0 ? 1u : 2u,
        .result = &result,
        .result_size = sizeof(result),
    };
    enum wf_completion_publish_result published = wf_completion_publish_terminal(
        adapter->runtime,
        work->token,
        &publication
    );
    if (helper != 0) {
        atomic_fetch_add_explicit(
            &adapter->stat_helper_executions,
            1,
            memory_order_relaxed
        );
    } else {
        atomic_fetch_add_explicit(
            &adapter->stat_scheduler_executions,
            1,
            memory_order_relaxed
        );
    }
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
            (void)pthread_cond_wait(
                &adapter->queue_available,
                &adapter->queue_lock
            );
        }
        if (adapter->queue_count == 0 && adapter->stopping != 0) {
            (void)pthread_mutex_unlock(&adapter->queue_lock);
            return NULL;
        }
        released_capacity = adapter->queue_count == adapter->queue_capacity;
        work = adapter->queue[adapter->queue_head];
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
    adapter->helper_count = helper_count;
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
    }
    return 0;
}

enum wf_file_submit_result wf_file_adapter_submit(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
) {
    enum wf_completion_transition_result transition;

    if (adapter == NULL || adapter->initialized == 0
        || !wf_file_request_valid(request)) {
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

    adapter->queue[adapter->queue_tail].token = token;
    adapter->queue[adapter->queue_tail].request = *request;
    adapter->queue_tail = (adapter->queue_tail + 1) % adapter->queue_capacity;
    adapter->queue_count += 1;
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );
    if (adapter->helper_count != 0) {
        (void)pthread_cond_signal(&adapter->queue_available);
    }
    (void)pthread_mutex_unlock(&adapter->queue_lock);
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
    int first_error = 0;
    if (adapter == NULL || adapter->initialized == 0) {
        return EINVAL;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    adapter->stopping = 1;
    (void)pthread_cond_broadcast(&adapter->queue_available);
    (void)pthread_mutex_unlock(&adapter->queue_lock);

    if (adapter->helper_count == 0) {
        while (wf_file_adapter_progress(adapter, 1) != 0) {
        }
    }
    for (helper = 0; helper < adapter->helper_count; ++helper) {
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
