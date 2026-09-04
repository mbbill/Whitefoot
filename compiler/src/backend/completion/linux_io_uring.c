#define _GNU_SOURCE

#include "linux_io_uring.h"

#if defined(__linux__)

#include <errno.h>
#include <limits.h>
#include <poll.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <unistd.h>

_Static_assert(
    sizeof(wf_linux_file_result) <= WF_COMPLETION_RESULT_CAPACITY,
    "completion result cell must hold a Linux file result"
);

static unsigned wf_linux_load_acquire(const unsigned *value) {
    return __atomic_load_n(value, __ATOMIC_ACQUIRE);
}

static unsigned wf_linux_load_relaxed(const unsigned *value) {
    return __atomic_load_n(value, __ATOMIC_RELAXED);
}

static void wf_linux_store_release(unsigned *target, unsigned value) {
    __atomic_store_n(target, value, __ATOMIC_RELEASE);
}

static int wf_linux_enter(
    int ring_descriptor,
    unsigned submissions,
    unsigned minimum_completions,
    unsigned flags
) {
    long result = syscall(
        __NR_io_uring_enter,
        ring_descriptor,
        submissions,
        minimum_completions,
        flags,
        NULL,
        0u
    );
    if (result < 0) {
        return -errno;
    }
    return (int)result;
}

static void wf_linux_unmap(wf_linux_io_uring_adapter *adapter) {
    if (adapter->submission_entries != MAP_FAILED
        && adapter->submission_entries != NULL) {
        (void)munmap(
            adapter->submission_entries,
            adapter->submission_entries_size
        );
    }
    if (adapter->completion_mapping != MAP_FAILED
        && adapter->completion_mapping != NULL
        && adapter->completion_mapping != adapter->submission_mapping) {
        (void)munmap(
            adapter->completion_mapping,
            adapter->completion_mapping_size
        );
    }
    if (adapter->submission_mapping != MAP_FAILED
        && adapter->submission_mapping != NULL) {
        (void)munmap(
            adapter->submission_mapping,
            adapter->submission_mapping_size
        );
    }
    adapter->submission_entries = NULL;
    adapter->submission_mapping = NULL;
    adapter->completion_mapping = NULL;
}

enum wf_linux_wait_source {
    WF_LINUX_WAIT_WAKE = 1,
    WF_LINUX_WAIT_CQ = 2
};

static void wf_linux_record_progress_error(
    wf_linux_io_uring_adapter *adapter,
    int error
) {
    int expected = 0;
    if (error == 0) {
        return;
    }
    (void)atomic_compare_exchange_strong_explicit(
        &adapter->progress_error,
        &expected,
        error,
        memory_order_release,
        memory_order_relaxed
    );
    /* The error is target progress too. Advancing the core epoch closes an
     * error-before-park race, while the core emits an eventfd write only when
     * a scheduler has actually announced sleep. */
    wf_completion_notify_target(adapter->runtime);
}

static void wf_linux_close_wait_set(wf_linux_io_uring_adapter *adapter) {
    if (adapter->wake_descriptor >= 0) {
        (void)close(adapter->wake_descriptor);
        adapter->wake_descriptor = -1;
    }
    if (adapter->wait_descriptor >= 0) {
        (void)close(adapter->wait_descriptor);
        adapter->wait_descriptor = -1;
    }
}

static int wf_linux_init_wait_set(wf_linux_io_uring_adapter *adapter) {
    struct epoll_event event;
    adapter->wait_descriptor = epoll_create1(EPOLL_CLOEXEC);
    if (adapter->wait_descriptor < 0) {
        return errno;
    }
    adapter->wake_descriptor = eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK);
    if (adapter->wake_descriptor < 0) {
        int error = errno;
        wf_linux_close_wait_set(adapter);
        return error;
    }

    memset(&event, 0, sizeof(event));
    event.events = EPOLLIN;
    event.data.u32 = WF_LINUX_WAIT_WAKE;
    if (epoll_ctl(
            adapter->wait_descriptor,
            EPOLL_CTL_ADD,
            adapter->wake_descriptor,
            &event
        ) != 0) {
        int error = errno;
        wf_linux_close_wait_set(adapter);
        return error;
    }
    memset(&event, 0, sizeof(event));
    event.events = EPOLLIN;
    event.data.u32 = WF_LINUX_WAIT_CQ;
    if (epoll_ctl(
            adapter->wait_descriptor,
            EPOLL_CTL_ADD,
            adapter->ring_descriptor,
            &event
        ) != 0) {
        int error = errno;
        wf_linux_close_wait_set(adapter);
        return error;
    }
    return 0;
}

static void wf_linux_init_statistics(wf_linux_io_uring_adapter *adapter) {
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_submission_enters, 0);
    atomic_init(&adapter->stat_capacity_waits, 0);
    atomic_init(&adapter->stat_completions, 0);
    atomic_init(&adapter->stat_publication_failures, 0);
    atomic_init(&adapter->stat_enter_failures, 0);
    atomic_init(&adapter->stat_kernel_waits, 0);
    atomic_init(&adapter->stat_kernel_wakes, 0);
    atomic_init(&adapter->stat_host_wake_writes, 0);
    atomic_init(&adapter->progress_error, 0);
}

int wf_linux_io_uring_init(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_linux_io_uring_entry *entry_storage,
    size_t entry_capacity
) {
    struct io_uring_params parameters;
    size_t submission_size;
    size_t completion_size;
    size_t entry_index;
    int error;

    if (adapter == NULL || runtime == NULL || entry_storage == NULL
        || entry_capacity == 0 || entry_capacity > UINT_MAX) {
        return EINVAL;
    }
    memset(adapter, 0, sizeof(*adapter));
    adapter->ring_descriptor = -1;
    adapter->wait_descriptor = -1;
    adapter->wake_descriptor = -1;
    adapter->submission_mapping = MAP_FAILED;
    adapter->completion_mapping = MAP_FAILED;
    adapter->submission_entries = MAP_FAILED;
    adapter->runtime = runtime;
    adapter->entries = entry_storage;
    adapter->entry_capacity = entry_capacity;
    atomic_init(&adapter->entry_cursor, 0);
    atomic_init(&adapter->in_flight, 0);
    wf_linux_init_statistics(adapter);
    for (entry_index = 0; entry_index < entry_capacity; ++entry_index) {
        atomic_init(
            &entry_storage[entry_index].state,
            WF_LINUX_IO_URING_ENTRY_FREE
        );
    }

    memset(&parameters, 0, sizeof(parameters));
    adapter->ring_descriptor = (int)syscall(
        __NR_io_uring_setup,
        (unsigned)entry_capacity,
        &parameters
    );
    if (adapter->ring_descriptor < 0) {
        error = errno;
        adapter->ring_descriptor = -1;
        return error;
    }
    if ((parameters.features & IORING_FEAT_NODROP) == 0
        || parameters.sq_entries < entry_capacity) {
        (void)close(adapter->ring_descriptor);
        adapter->ring_descriptor = -1;
        return ENOTSUP;
    }

    submission_size = parameters.sq_off.array
        + parameters.sq_entries * sizeof(unsigned);
    completion_size = parameters.cq_off.cqes
        + parameters.cq_entries * sizeof(struct io_uring_cqe);
    if ((parameters.features & IORING_FEAT_SINGLE_MMAP) != 0) {
        size_t shared_size = submission_size > completion_size
            ? submission_size
            : completion_size;
        adapter->submission_mapping = mmap(
            NULL,
            shared_size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            adapter->ring_descriptor,
            IORING_OFF_SQ_RING
        );
        if (adapter->submission_mapping == MAP_FAILED) {
            error = errno;
            (void)close(adapter->ring_descriptor);
            adapter->ring_descriptor = -1;
            return error;
        }
        adapter->submission_mapping_size = shared_size;
        adapter->completion_mapping = adapter->submission_mapping;
        adapter->completion_mapping_size = shared_size;
    } else {
        adapter->submission_mapping = mmap(
            NULL,
            submission_size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            adapter->ring_descriptor,
            IORING_OFF_SQ_RING
        );
        if (adapter->submission_mapping == MAP_FAILED) {
            error = errno;
            (void)close(adapter->ring_descriptor);
            adapter->ring_descriptor = -1;
            return error;
        }
        adapter->submission_mapping_size = submission_size;
        adapter->completion_mapping = mmap(
            NULL,
            completion_size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            adapter->ring_descriptor,
            IORING_OFF_CQ_RING
        );
        if (adapter->completion_mapping == MAP_FAILED) {
            error = errno;
            wf_linux_unmap(adapter);
            (void)close(adapter->ring_descriptor);
            adapter->ring_descriptor = -1;
            return error;
        }
        adapter->completion_mapping_size = completion_size;
    }

    adapter->submission_entries_size = parameters.sq_entries
        * sizeof(struct io_uring_sqe);
    adapter->submission_entries = mmap(
        NULL,
        adapter->submission_entries_size,
        PROT_READ | PROT_WRITE,
        MAP_SHARED | MAP_POPULATE,
        adapter->ring_descriptor,
        IORING_OFF_SQES
    );
    if (adapter->submission_entries == MAP_FAILED) {
        error = errno;
        wf_linux_unmap(adapter);
        (void)close(adapter->ring_descriptor);
        adapter->ring_descriptor = -1;
        return error;
    }

#define WF_RING_POINTER(mapping, offset, type)                                \
    ((type *)((unsigned char *)(mapping) + (offset)))
    adapter->submission_head = WF_RING_POINTER(
        adapter->submission_mapping,
        parameters.sq_off.head,
        unsigned
    );
    adapter->submission_tail = WF_RING_POINTER(
        adapter->submission_mapping,
        parameters.sq_off.tail,
        unsigned
    );
    adapter->submission_mask = WF_RING_POINTER(
        adapter->submission_mapping,
        parameters.sq_off.ring_mask,
        unsigned
    );
    adapter->submission_count = WF_RING_POINTER(
        adapter->submission_mapping,
        parameters.sq_off.ring_entries,
        unsigned
    );
    adapter->submission_array = WF_RING_POINTER(
        adapter->submission_mapping,
        parameters.sq_off.array,
        unsigned
    );
    adapter->completion_head = WF_RING_POINTER(
        adapter->completion_mapping,
        parameters.cq_off.head,
        unsigned
    );
    adapter->completion_tail = WF_RING_POINTER(
        adapter->completion_mapping,
        parameters.cq_off.tail,
        unsigned
    );
    adapter->completion_mask = WF_RING_POINTER(
        adapter->completion_mapping,
        parameters.cq_off.ring_mask,
        unsigned
    );
    adapter->completion_count = WF_RING_POINTER(
        adapter->completion_mapping,
        parameters.cq_off.ring_entries,
        unsigned
    );
    adapter->completion_overflow = WF_RING_POINTER(
        adapter->completion_mapping,
        parameters.cq_off.overflow,
        unsigned
    );
    adapter->completion_entries = WF_RING_POINTER(
        adapter->completion_mapping,
        parameters.cq_off.cqes,
        struct io_uring_cqe
    );
#undef WF_RING_POINTER

    error = pthread_mutex_init(&adapter->submission_lock, NULL);
    if (error != 0) {
        wf_linux_unmap(adapter);
        (void)close(adapter->ring_descriptor);
        adapter->ring_descriptor = -1;
        return error;
    }
    error = pthread_mutex_init(&adapter->completion_lock, NULL);
    if (error != 0) {
        (void)pthread_mutex_destroy(&adapter->submission_lock);
        wf_linux_unmap(adapter);
        (void)close(adapter->ring_descriptor);
        adapter->ring_descriptor = -1;
        return error;
    }
    error = wf_linux_init_wait_set(adapter);
    if (error != 0) {
        (void)pthread_mutex_destroy(&adapter->completion_lock);
        (void)pthread_mutex_destroy(&adapter->submission_lock);
        wf_linux_unmap(adapter);
        (void)close(adapter->ring_descriptor);
        adapter->ring_descriptor = -1;
        return error;
    }
    adapter->initialized = 1;
    return 0;
}

static int wf_linux_request_valid(const wf_linux_file_request *request) {
    if (request == NULL || request->count > UINT32_MAX
        || request->offset > INT64_MAX) {
        return 0;
    }
    switch (request->kind) {
    case WF_LINUX_FILE_READ_AT:
        return request->descriptor >= 0
            && (request->buffer.read_buffer != NULL || request->count == 0);
    case WF_LINUX_FILE_WRITE_AT:
        return request->descriptor >= 0
            && (request->buffer.write_buffer != NULL || request->count == 0);
    case WF_LINUX_FILE_OPEN_AT:
        /* An open resolves its path against a directory descriptor, and
         * AT_FDCWD is a negative one, so the transfer descriptor check does
         * not apply here.  The name must fit the entry that will own it,
         * because that copy is what the kernel resolves. */
        return wf_file_path_fits(request->buffer.path)
            && request->has_open_mode <= 1u
            && request->expected_kind <= WF_FILE_EXPECT_DIRECTORY;
    case WF_LINUX_FILE_CLOSE:
        return request->descriptor >= 0;
    default:
        return 0;
    }
}

static wf_linux_io_uring_entry *wf_linux_reserve_entry(
    wf_linux_io_uring_adapter *adapter,
    size_t *entry_index
) {
    size_t start = atomic_fetch_add_explicit(
        &adapter->entry_cursor,
        1,
        memory_order_relaxed
    );
    size_t offset;
    size_t index = start % adapter->entry_capacity;
    for (offset = 0; offset < adapter->entry_capacity; ++offset) {
        unsigned expected = WF_LINUX_IO_URING_ENTRY_FREE;
        if (atomic_compare_exchange_strong_explicit(
                &adapter->entries[index].state,
                &expected,
                WF_LINUX_IO_URING_ENTRY_RESERVED,
                memory_order_acquire,
                memory_order_relaxed
            )) {
            *entry_index = index;
            return &adapter->entries[index];
        }
        index = index + 1u == adapter->entry_capacity ? 0u : index + 1u;
    }
    return NULL;
}

static enum wf_linux_io_uring_submit_result wf_linux_wait_capacity(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_token token
) {
    enum wf_completion_transition_result transition =
        wf_completion_begin_submit(adapter->runtime, token);
    if (transition == WF_COMPLETION_TRANSITION_STALE) {
        return WF_LINUX_IO_URING_SUBMIT_STALE;
    }
    if (transition != WF_COMPLETION_TRANSITIONED) {
        return WF_LINUX_IO_URING_SUBMIT_INVALID;
    }
    if (wf_completion_mark_wait_capacity(adapter->runtime, token)
        != WF_COMPLETION_TRANSITIONED) {
        return WF_LINUX_IO_URING_SUBMIT_INVALID;
    }
    atomic_fetch_add_explicit(
        &adapter->stat_capacity_waits,
        1,
        memory_order_relaxed
    );
    return WF_LINUX_IO_URING_WAIT_CAPACITY;
}

static void wf_linux_stage_entry_locked(
    wf_linux_io_uring_adapter *adapter,
    size_t entry_index,
    wf_linux_io_uring_entry *entry
) {
    unsigned tail = wf_linux_load_relaxed(adapter->submission_tail);
    unsigned ring_index = tail & *adapter->submission_mask;
    struct io_uring_sqe *submission =
        &adapter->submission_entries[ring_index];
    memset(submission, 0, sizeof(*submission));
    if (entry->waiting_readiness != 0) {
        submission->opcode = IORING_OP_POLL_ADD;
        submission->fd = entry->request.descriptor;
        submission->poll_events = entry->request.kind == WF_LINUX_FILE_READ_AT
            ? POLLIN
            : POLLOUT;
    } else {
        switch (entry->request.kind) {
        case WF_LINUX_FILE_OPEN_AT:
            submission->opcode = IORING_OP_OPENAT;
            submission->fd = entry->request.descriptor;
            submission->addr = (uint64_t)(uintptr_t)entry->request.buffer.path;
            submission->open_flags = (uint32_t)(
                entry->request.open_flags
                | wf_file_open_kind_flags(entry->request.expected_kind)
            );
            submission->len = entry->request.has_open_mode != 0
                ? (uint32_t)entry->request.open_mode
                : 0u;
            break;
        case WF_LINUX_FILE_CLOSE:
            submission->opcode = IORING_OP_CLOSE;
            submission->fd = entry->request.descriptor;
            break;
        case WF_LINUX_FILE_READ_AT:
        case WF_LINUX_FILE_WRITE_AT:
        default:
            submission->opcode = entry->request.kind == WF_LINUX_FILE_READ_AT
                ? IORING_OP_READ
                : IORING_OP_WRITE;
            submission->fd = entry->request.descriptor;
            submission->off = entry->request.offset;
            submission->addr = entry->request.kind == WF_LINUX_FILE_READ_AT
                ? (uint64_t)(uintptr_t)entry->request.buffer.read_buffer
                : (uint64_t)(uintptr_t)entry->request.buffer.write_buffer;
            submission->len = (uint32_t)entry->request.count;
            break;
        }
    }
    submission->user_data = (uint64_t)entry_index + 1u;
    adapter->submission_array[ring_index] = ring_index;
    atomic_store_explicit(
        &entry->state,
        WF_LINUX_IO_URING_ENTRY_IN_FLIGHT,
        memory_order_release
    );
    wf_linux_store_release(adapter->submission_tail, tail + 1u);
}

static void wf_linux_stage_pending_locked(
    wf_linux_io_uring_adapter *adapter
) {
    size_t index;
    for (index = 0; index < adapter->entry_capacity; ++index) {
        unsigned head = wf_linux_load_acquire(adapter->submission_head);
        unsigned tail = wf_linux_load_relaxed(adapter->submission_tail);
        if (tail - head >= *adapter->submission_count) {
            return;
        }
        if (atomic_load_explicit(
                &adapter->entries[index].state,
                memory_order_acquire
            ) == WF_LINUX_IO_URING_ENTRY_RETRY_PENDING) {
            wf_linux_stage_entry_locked(
                adapter,
                index,
                &adapter->entries[index]
            );
        }
    }
}

static int wf_linux_kick_locked(wf_linux_io_uring_adapter *adapter) {
    wf_linux_stage_pending_locked(adapter);
    unsigned head = wf_linux_load_acquire(adapter->submission_head);
    unsigned tail = wf_linux_load_acquire(adapter->submission_tail);
    unsigned pending = tail - head;
    int result;
    if (pending == 0) {
        return 0;
    }
    atomic_fetch_add_explicit(
        &adapter->stat_submission_enters,
        1,
        memory_order_relaxed
    );
    do {
        result = wf_linux_enter(adapter->ring_descriptor, pending, 0, 0);
    } while (result == -EINTR);
    if (result < 0) {
        atomic_fetch_add_explicit(
            &adapter->stat_enter_failures,
            1,
            memory_order_relaxed
        );
        return -result;
    }
    return 0;
}

enum wf_linux_io_uring_submit_result wf_linux_io_uring_submit(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_token token,
    const wf_linux_file_request *request
) {
    wf_linux_io_uring_entry *entry;
    enum wf_completion_transition_result transition;
    unsigned head;
    unsigned tail;
    size_t entry_index;

    if (adapter == NULL || adapter->initialized == 0
        || !wf_linux_request_valid(request)) {
        return adapter == NULL || adapter->initialized == 0
            ? WF_LINUX_IO_URING_UNAVAILABLE
            : WF_LINUX_IO_URING_SUBMIT_INVALID;
    }

    (void)pthread_mutex_lock(&adapter->submission_lock);
    head = wf_linux_load_acquire(adapter->submission_head);
    tail = wf_linux_load_relaxed(adapter->submission_tail);
    if (tail - head >= *adapter->submission_count) {
        /* A deferred doorbell fills the submission queue with entries the
         * kernel has not been told about, so a full queue is first this
         * thread's own backlog and only then real backpressure.  Ring the
         * doorbell, which is what advances the head, and re-read it once
         * before declaring a capacity wait. */
        int error = wf_linux_kick_locked(adapter);
        if (error != 0) {
            wf_linux_record_progress_error(adapter, error);
        }
        head = wf_linux_load_acquire(adapter->submission_head);
        if (tail - head >= *adapter->submission_count) {
            (void)pthread_mutex_unlock(&adapter->submission_lock);
            return wf_linux_wait_capacity(adapter, token);
        }
    }
    entry = wf_linux_reserve_entry(adapter, &entry_index);
    if (entry == NULL) {
        (void)pthread_mutex_unlock(&adapter->submission_lock);
        return wf_linux_wait_capacity(adapter, token);
    }

    transition = wf_completion_begin_submit(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        atomic_store_explicit(
            &entry->state,
            WF_LINUX_IO_URING_ENTRY_FREE,
            memory_order_release
        );
        (void)pthread_mutex_unlock(&adapter->submission_lock);
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_LINUX_IO_URING_SUBMIT_STALE
            : WF_LINUX_IO_URING_SUBMIT_INVALID;
    }

    entry->token = token;
    entry->kind = request->kind;
    entry->request = *request;
    if (request->kind == WF_LINUX_FILE_OPEN_AT) {
        /* The SQE keeps this pointer until the kernel resolves the name, and
         * the caller regains its buffer as soon as submission returns, so the
         * bytes become the entry's before any SQE names them.  The length was
         * checked by wf_linux_request_valid before anything was claimed. */
        (void)wf_file_stage_path(entry->path_storage, request->buffer.path);
        entry->request.buffer.path = entry->path_storage;
    }
    entry->waiting_readiness = 0;
    entry->opened_descriptor = -1;
    entry->open_outcome = WF_FILE_OPEN_SUCCEEDED;
    entry->open_error = 0;
    transition = wf_completion_target_accepted(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        atomic_store_explicit(
            &entry->state,
            WF_LINUX_IO_URING_ENTRY_FREE,
            memory_order_release
        );
        (void)pthread_mutex_unlock(&adapter->submission_lock);
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_LINUX_IO_URING_SUBMIT_STALE
            : WF_LINUX_IO_URING_SUBMIT_INVALID;
    }

    if (request->kind == WF_LINUX_FILE_OPEN_AT) {
        /* [SYS-2]'s `loan-released(path)` for this open holds from here: the
         * name the kernel will resolve is the entry's own, so the caller's
         * buffer is free before the SQE exists.  A refusal is an
         * adapter/core contract defect; it is counted, never retried. */
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
    atomic_fetch_add_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    wf_linux_stage_entry_locked(adapter, entry_index, entry);
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );

    /* The doorbell is deferred.  Staging an SQE is a store to a shared page
     * the kernel already maps, so a submission costs no syscall at all until
     * some thread flushes; `io_uring_enter` is then one call for however many
     * entries have accumulated.  Measured on the eight-wide many-file
     * benchmark, deferring turned 15,360 enters into 2,048 and took 15.6 ms
     * of system CPU off a program whose wall time fell by the same amount
     * (LOOP-PIPELINE.md §9.1).
     *
     * What makes it safe is that every path which can wait flushes first:
     * `wf_linux_io_uring_progress` kicks before it reaps or sleeps, and every
     * join and park in the bridge reaches the kernel through it.  The bridge
     * also flushes before a blocking direct host call, so a submitted open can
     * never sit unkicked behind one.
     *
     * A full submission queue is the one case the submitting thread must
     * resolve itself, and it does so above, before it declares a capacity
     * wait. */
    (void)pthread_mutex_unlock(&adapter->submission_lock);
    return WF_LINUX_IO_URING_TARGET_OWNS;
}

static int wf_linux_kick(wf_linux_io_uring_adapter *adapter) {
    int result;
    (void)pthread_mutex_lock(&adapter->submission_lock);
    result = wf_linux_kick_locked(adapter);
    (void)pthread_mutex_unlock(&adapter->submission_lock);
    return result;
}

static int wf_linux_resubmit_entry(
    wf_linux_io_uring_adapter *adapter,
    size_t entry_index,
    wf_linux_io_uring_entry *entry,
    unsigned waiting_readiness
) {
    unsigned head;
    unsigned tail;
    int result;
    entry->waiting_readiness = waiting_readiness;
    (void)pthread_mutex_lock(&adapter->submission_lock);
    head = wf_linux_load_acquire(adapter->submission_head);
    tail = wf_linux_load_relaxed(adapter->submission_tail);
    result = 0;
    if (tail - head >= *adapter->submission_count) {
        /* A deferred doorbell fills the submission queue with entries the
         * kernel has not been told about, so a full queue is first this
         * thread's own backlog.  Ring it, which is what advances the head,
         * before deciding this entry cannot be staged. */
        result = wf_linux_kick_locked(adapter);
        head = wf_linux_load_acquire(adapter->submission_head);
    }
    if (tail - head < *adapter->submission_count) {
        int kicked;
        wf_linux_stage_entry_locked(adapter, entry_index, entry);
        /* The doorbell rings here and not at some later pass.  A re-attempt
         * left staged behind a completion-only sleep would wait for a CQE the
         * kernel has not been asked to produce. */
        kicked = wf_linux_kick_locked(adapter);
        if (result == 0) {
            result = kicked;
        }
    } else {
        atomic_store_explicit(
            &entry->state,
            WF_LINUX_IO_URING_ENTRY_RETRY_PENDING,
            memory_order_release
        );
    }
    (void)pthread_mutex_unlock(&adapter->submission_lock);
    return result;
}

static int wf_linux_transfer_kind(enum wf_linux_file_operation_kind kind) {
    return kind == WF_LINUX_FILE_READ_AT || kind == WF_LINUX_FILE_WRITE_AT;
}

/* Decides a completed open's typed outcome.
 *
 * The path resolution — the part of an open that can genuinely wait — is what
 * the ring carries. What remains is reading the mode of the descriptor the
 * kernel already produced, and that is an in-memory inode read which cannot
 * wait on anything, so it is one `fstat` here rather than a second ring
 * operation.
 *
 * That is a measurement, not a preference. The kind check was first written
 * as a linked `IORING_OP_STATX` of the same descriptor, which keeps the
 * reaping thread free of host calls entirely. Two ring round trips per open
 * cost more than the open they wrap: on the two-CPU Linux container the
 * eight-wide many-file program ran 152 ms against 116 ms for the bounded
 * adapter it replaced, and the four-wide one 203 ms against 140 ms. With the
 * check done here instead, the same programs run 119 ms and 141 ms — parity,
 * with the blocking `openat` gone from the scheduler thread. A descriptor the
 * check refuses is disposed of by the same single close the direct path
 * makes, on the error path where nothing is waiting for throughput. */
static void wf_linux_decide_open(
    wf_linux_io_uring_entry *entry,
    int32_t completion_result
) {
    struct stat status;
    if (completion_result < 0) {
        entry->opened_descriptor = -1;
        entry->open_error = -completion_result;
        entry->open_outcome = WF_FILE_OPEN_FAILED;
        return;
    }
    entry->opened_descriptor = completion_result;
    entry->open_error = 0;
    entry->open_outcome = WF_FILE_OPEN_SUCCEEDED;
    if (entry->request.expected_kind == WF_FILE_EXPECT_ANY) {
        /* WF_IO_NOCACHE, applied to the descriptor the ring produced,
         * exactly where the bounded adapter applies it to the one openat
         * produced, so both backends answer the same policy. */
        wf_file_apply_uncached_reads(entry->opened_descriptor);
        return;
    }
    if (fstat(entry->opened_descriptor, &status) != 0) {
        entry->open_error = errno;
        entry->open_outcome = WF_FILE_OPEN_STATUS_FAILED;
    } else {
        entry->open_outcome = wf_file_kind_outcome(
            entry->request.expected_kind,
            (unsigned int)status.st_mode
        );
        if (entry->open_outcome == WF_FILE_OPEN_SUCCEEDED) {
            wf_file_apply_uncached_reads(entry->opened_descriptor);
            return;
        }
    }
    /* One close attempt whose diagnostic is discarded, exactly as the direct
     * path makes it, and never retried. */
    (void)close(entry->opened_descriptor);
}

/* Publishes one entry's terminal result, frees it, and counts it.
 *
 * Every route out of an owned operation ends here, so an operation's
 * completion is counted in exactly one place. */
static int wf_linux_publish_entry_locked(
    wf_linux_io_uring_adapter *adapter,
    wf_linux_io_uring_entry *entry,
    int32_t completion_result
) {
    wf_linux_file_result result;
    wf_completion_publication publication;
    enum wf_completion_publish_result published;
    memset(&result, 0, sizeof(result));
    result.kind = entry->kind;
    if (entry->request.kind == WF_LINUX_FILE_OPEN_AT) {
        /* The kind decision is the whole answer, and it names the descriptor
         * even where it refused it, exactly as the direct path does. */
        result.open_outcome = entry->open_outcome;
        result.error_code = entry->open_error;
        result.value = entry->opened_descriptor;
    } else if (completion_result < 0) {
        result.value = -1;
        result.error_code = -completion_result;
    } else {
        result.value = completion_result;
    }
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = result.error_code == 0 ? 1u : 2u;
    publication.result = &result;
    publication.result_size = sizeof(result);
    published = wf_completion_publish_terminal(
        adapter->runtime,
        entry->token,
        &publication
    );
    if (published != WF_COMPLETION_PUBLISHED) {
        atomic_fetch_add_explicit(
            &adapter->stat_publication_failures,
            1,
            memory_order_relaxed
        );
    }
    atomic_store_explicit(
        &entry->state,
        WF_LINUX_IO_URING_ENTRY_FREE,
        memory_order_release
    );
    atomic_fetch_sub_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(
        &adapter->stat_completions,
        1,
        memory_order_relaxed
    );
    return published == WF_COMPLETION_PUBLISHED ? 0 : EPROTO;
}

static int wf_linux_publish_completion(
    wf_linux_io_uring_adapter *adapter,
    const struct io_uring_cqe *completion,
    int *terminal_published
) {
    uint64_t encoded = completion->user_data;
    size_t entry_index;
    wf_linux_io_uring_entry *entry;

    *terminal_published = 0;
    if (encoded == 0 || encoded > adapter->entry_capacity) {
        return EPROTO;
    }
    entry_index = (size_t)(encoded - 1u);
    entry = &adapter->entries[entry_index];
    if (atomic_load_explicit(&entry->state, memory_order_acquire)
        != WF_LINUX_IO_URING_ENTRY_IN_FLIGHT) {
        return EPROTO;
    }

    if (wf_linux_transfer_kind(entry->kind)) {
        /* Interruption and readiness refusal are adapter progress on a
         * transfer, exactly as on the bounded POSIX adapter, and never a
         * writer-visible outcome. */
        if (entry->waiting_readiness != 0) {
            if (completion->res >= 0) {
                return wf_linux_resubmit_entry(adapter, entry_index, entry, 0);
            }
            if (completion->res == -EINTR || completion->res == -EAGAIN) {
                return wf_linux_resubmit_entry(adapter, entry_index, entry, 1);
            }
        } else if (completion->res == -EINTR) {
            return wf_linux_resubmit_entry(adapter, entry_index, entry, 0);
        } else if (completion->res == -EAGAIN) {
            return wf_linux_resubmit_entry(adapter, entry_index, entry, 1);
        }
    } else if (entry->request.kind == WF_LINUX_FILE_OPEN_AT) {
        wf_linux_decide_open(entry, completion->res);
    }

    *terminal_published = 1;
    return wf_linux_publish_entry_locked(adapter, entry, completion->res);
}

int wf_linux_io_uring_progress(
    wf_linux_io_uring_adapter *adapter,
    size_t budget,
    unsigned wait,
    size_t *published
) {
    size_t total = 0;
    size_t processed = 0;
    int first_error = 0;
    unsigned head;
    unsigned tail;

    if (published != NULL) {
        *published = 0;
    }
    if (adapter == NULL || adapter->initialized == 0 || published == NULL
        || budget == 0) {
        return EINVAL;
    }

    first_error = atomic_load_explicit(
        &adapter->progress_error,
        memory_order_acquire
    );
    if (first_error != 0) {
        return first_error;
    }

    first_error = wf_linux_kick(adapter);
    (void)pthread_mutex_lock(&adapter->completion_lock);
    head = wf_linux_load_relaxed(adapter->completion_head);
    tail = wf_linux_load_acquire(adapter->completion_tail);
    /* Never enter a completion-only sleep while staged SQEs failed to reach
     * the kernel; the scheduler must observe and resolve those rather than
     * waiting for a CQE which cannot yet exist. */
    if (head == tail && wait != 0 && first_error == 0 && total == 0) {
        int entered;
        do {
            entered = wf_linux_enter(
                adapter->ring_descriptor,
                0,
                1,
                IORING_ENTER_GETEVENTS
            );
        } while (entered == -EINTR);
        if (entered < 0 && first_error == 0) {
            first_error = -entered;
            atomic_fetch_add_explicit(
                &adapter->stat_enter_failures,
                1,
                memory_order_relaxed
            );
        }
        tail = wf_linux_load_acquire(adapter->completion_tail);
    }

    while (head != tail && processed < budget) {
        unsigned completion_index = head & *adapter->completion_mask;
        struct io_uring_cqe completion =
            adapter->completion_entries[completion_index];
        int terminal_published = 0;
        int error = wf_linux_publish_completion(
            adapter,
            &completion,
            &terminal_published
        );
        head += 1u;
        wf_linux_store_release(adapter->completion_head, head);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
        total += (size_t)terminal_published;
        processed += 1;
        tail = wf_linux_load_acquire(adapter->completion_tail);
    }
    (void)pthread_mutex_unlock(&adapter->completion_lock);

    if (first_error != 0) {
        wf_linux_record_progress_error(adapter, first_error);
    }

    if (total != 0) {
        wf_completion_notify_capacity(adapter->runtime);
    }
    *published = total;
    return first_error;
}

void wf_linux_io_uring_notify(void *context) {
    wf_linux_io_uring_adapter *adapter = context;
    uint64_t one = 1;
    ssize_t written;
    if (adapter == NULL || adapter->initialized == 0
        || adapter->wake_descriptor < 0) {
        return;
    }
    do {
        written = write(adapter->wake_descriptor, &one, sizeof(one));
    } while (written < 0 && errno == EINTR);
    if (written == (ssize_t)sizeof(one)) {
        atomic_fetch_add_explicit(
            &adapter->stat_host_wake_writes,
            1,
            memory_order_relaxed
        );
        return;
    }
    if (written < 0 && errno == EAGAIN) {
        /* A saturated eventfd is already readable, so no wake can be lost. */
        return;
    }
    /* A valid nonblocking eventfd accepts exactly eight bytes or reports
     * EAGAIN while already readable. Anything else is corruption of the
     * compiler-owned wait endpoint; fail-stop here instead of leaving the
     * announced scheduler asleep forever. */
    abort();
}

static int wf_linux_drain_wake_descriptor(
    wf_linux_io_uring_adapter *adapter
) {
    for (;;) {
        uint64_t count;
        ssize_t received;
        do {
            received = read(
                adapter->wake_descriptor,
                &count,
                sizeof(count)
            );
        } while (received < 0 && errno == EINTR);
        if (received == (ssize_t)sizeof(count)) {
            continue;
        }
        if (received < 0 && errno == EAGAIN) {
            return 0;
        }
        return received < 0 ? errno : EIO;
    }
}

int wf_linux_io_uring_park(
    wf_linux_io_uring_adapter *adapter,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
) {
    struct epoll_event events[2];
    int timeout;
    int ready;
    int error;
    int index;
    int announced = 0;
    if (adapter == NULL || adapter->initialized == 0
        || adapter->wait_descriptor < 0 || adapter->wake_descriptor < 0) {
        return EINVAL;
    }
    error = wf_linux_io_uring_progress_error(adapter);
    if (error != 0) {
        return error;
    }
    /* Announce sleep under the same lock used by core publishers.  A
     * publisher before this point leaves only an epoch/CQ fact and performs
     * no host wake. A publisher after the second recheck sees the announced
     * sleeper and leaves a level-readable eventfd. */
    error = pthread_mutex_lock(&adapter->runtime->wake_lock);
    if (error != 0) {
        /* A broken compiler-owned wake mutex cannot be reported through that
         * same mutex without risking a silent recursive stall. */
        abort();
    }
    if (wf_completion_wake_epoch(adapter->runtime) != observed_epoch
        || wf_linux_load_acquire(adapter->completion_head)
            != wf_linux_load_acquire(adapter->completion_tail)) {
        (void)pthread_mutex_unlock(&adapter->runtime->wake_lock);
        return 0;
    }
    /* Sequentially consistent, and paired with the sequentially consistent
     * epoch load below: a core publisher raises the epoch and then reads this
     * count without taking wake_lock, so this announcement and that read are
     * what keeps an eventfd wake from being lost. */
    atomic_fetch_add_explicit(
        &adapter->runtime->parked_schedulers,
        1,
        memory_order_seq_cst
    );
    atomic_fetch_add_explicit(
        &adapter->runtime->stat_parks,
        1,
        memory_order_relaxed
    );
    announced = 1;
    if (atomic_load_explicit(
            &adapter->runtime->wake_epoch,
            memory_order_seq_cst
        ) != observed_epoch
        || wf_linux_load_acquire(adapter->completion_head)
            != wf_linux_load_acquire(adapter->completion_tail)) {
        atomic_fetch_sub_explicit(
            &adapter->runtime->parked_schedulers,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&adapter->runtime->wake_lock);
        return 0;
    }
    (void)pthread_mutex_unlock(&adapter->runtime->wake_lock);
    timeout = timeout_milliseconds == UINT32_MAX
        ? -1
        : timeout_milliseconds > (uint32_t)INT_MAX
            ? INT_MAX
            : (int)timeout_milliseconds;
    atomic_fetch_add_explicit(
        &adapter->stat_kernel_waits,
        1,
        memory_order_relaxed
    );
    do {
        ready = epoll_wait(adapter->wait_descriptor, events, 2, timeout);
    } while (ready < 0 && errno == EINTR
             && wf_completion_wake_epoch(adapter->runtime)
                 == observed_epoch);
    if (ready < 0 && errno == EINTR
        && wf_completion_wake_epoch(adapter->runtime) != observed_epoch) {
        /* A signal raced with a real core announcement. The epoch is the
         * durable wake fact; EINTR itself never becomes a progress failure. */
        error = 0;
    } else if (ready < 0) {
        error = errno;
    } else if (ready == 0) {
        error = ETIMEDOUT;
    } else {
        error = 0;
    }
    if (ready > 0) {
        atomic_fetch_add_explicit(
            &adapter->stat_kernel_wakes,
            1,
            memory_order_relaxed
        );
        for (index = 0; index < ready; ++index) {
            if ((events[index].events & (EPOLLERR | EPOLLHUP)) != 0) {
                error = EIO;
                break;
            }
            if (events[index].data.u32 != WF_LINUX_WAIT_WAKE
                && events[index].data.u32 != WF_LINUX_WAIT_CQ) {
                error = EPROTO;
                break;
            }
        }
    }

    /* Stop advertising this lane before removing the persistent wake record.
     * Both happen while publishers are excluded by wake_lock, so no eventfd
     * write can land after the drain for a lane that is already active. */
    if (announced != 0) {
        int lock_error = pthread_mutex_lock(&adapter->runtime->wake_lock);
        if (lock_error != 0) {
            abort();
        } else {
            unsigned previous = atomic_fetch_sub_explicit(
                &adapter->runtime->parked_schedulers,
                1,
                memory_order_relaxed
            );
            if (previous == 0) {
                abort();
            }
            /* The eventfd is one broadcast level fact, not a consumable wake
             * for whichever lane happens to run first. Keep it readable until
             * every scheduler that was announced has left epoll; otherwise a
             * lane waiting for unrelated capacity can steal the only wake
             * from the lane that owns a completed token. */
            if (previous == 1) {
                int drain_error = wf_linux_drain_wake_descriptor(adapter);
                if (error == 0 && drain_error != 0) {
                    error = drain_error;
                }
            }
            (void)pthread_mutex_unlock(&adapter->runtime->wake_lock);
        }
    }
    if (error != 0 && error != ETIMEDOUT) {
        wf_linux_record_progress_error(adapter, error);
        return error;
    }
    if (error == ETIMEDOUT) {
        return error;
    }
    return wf_linux_io_uring_progress_error(adapter);
}

int wf_linux_io_uring_progress_error(
    const wf_linux_io_uring_adapter *adapter
) {
    if (adapter == NULL) {
        return EINVAL;
    }
    return atomic_load_explicit(
        &adapter->progress_error,
        memory_order_acquire
    );
}

size_t wf_linux_io_uring_in_flight(
    const wf_linux_io_uring_adapter *adapter
) {
    if (adapter == NULL) {
        return 0;
    }
    return atomic_load_explicit(&adapter->in_flight, memory_order_acquire);
}

size_t wf_linux_io_uring_capacity(
    const wf_linux_io_uring_adapter *adapter
) {
    size_t submission;
    if (adapter == NULL || adapter->initialized == 0) {
        return 0;
    }
    submission = (size_t)*adapter->submission_count;
    return submission < adapter->entry_capacity ? submission
                                                : adapter->entry_capacity;
}

int wf_linux_io_uring_flush(wf_linux_io_uring_adapter *adapter) {
    int error;
    if (adapter == NULL || adapter->initialized == 0) {
        return EINVAL;
    }
    error = wf_linux_kick(adapter);
    if (error != 0) {
        wf_linux_record_progress_error(adapter, error);
    }
    return error;
}

int wf_linux_io_uring_destroy(wf_linux_io_uring_adapter *adapter) {
    int first_error = 0;
    if (adapter == NULL || adapter->initialized == 0) {
        return EINVAL;
    }
    if (wf_linux_io_uring_in_flight(adapter) != 0
        || wf_linux_load_acquire(adapter->submission_head)
            != wf_linux_load_acquire(adapter->submission_tail)
        || wf_linux_load_acquire(adapter->completion_head)
            != wf_linux_load_acquire(adapter->completion_tail)) {
        return EBUSY;
    }
    {
        int error = pthread_mutex_destroy(&adapter->completion_lock);
        if (error != 0) {
            first_error = error;
        }
    }
    {
        int error = pthread_mutex_destroy(&adapter->submission_lock);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    if (first_error != 0) {
        return first_error;
    }
    wf_linux_close_wait_set(adapter);
    wf_linux_unmap(adapter);
    if (close(adapter->ring_descriptor) != 0) {
        return errno;
    }
    adapter->ring_descriptor = -1;
    adapter->initialized = 0;
    return 0;
}

wf_linux_io_uring_statistics wf_linux_io_uring_statistics_snapshot(
    const wf_linux_io_uring_adapter *adapter
) {
    wf_linux_io_uring_statistics statistics = {0};
    if (adapter == NULL) {
        return statistics;
    }
    statistics.submissions = atomic_load_explicit(
        &adapter->stat_submissions,
        memory_order_relaxed
    );
    statistics.submission_enters = atomic_load_explicit(
        &adapter->stat_submission_enters,
        memory_order_relaxed
    );
    statistics.capacity_waits = atomic_load_explicit(
        &adapter->stat_capacity_waits,
        memory_order_relaxed
    );
    statistics.completions = atomic_load_explicit(
        &adapter->stat_completions,
        memory_order_relaxed
    );
    statistics.publication_failures = atomic_load_explicit(
        &adapter->stat_publication_failures,
        memory_order_relaxed
    );
    statistics.enter_failures = atomic_load_explicit(
        &adapter->stat_enter_failures,
        memory_order_relaxed
    );
    statistics.kernel_waits = atomic_load_explicit(
        &adapter->stat_kernel_waits,
        memory_order_relaxed
    );
    statistics.kernel_wakes = atomic_load_explicit(
        &adapter->stat_kernel_wakes,
        memory_order_relaxed
    );
    statistics.host_wake_writes = atomic_load_explicit(
        &adapter->stat_host_wake_writes,
        memory_order_relaxed
    );
    return statistics;
}

#else

typedef int wf_linux_io_uring_translation_unit_is_target_guarded;

#endif
