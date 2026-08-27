#ifndef WHITEFOOT_LINUX_IO_URING_H
#define WHITEFOOT_LINUX_IO_URING_H

#if defined(__linux__)

#include "contract.h"
/* The typed file vocabulary — expected kind, open outcome, and the one rule
 * that decides them — is shared with the bounded POSIX adapter so that one
 * open states one contract on every target.  Only the header is used: this
 * adapter calls no function defined in file_adapter.c. */
#include "file_adapter.h"

#include <linux/io_uring.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

enum wf_linux_file_operation_kind {
    WF_LINUX_FILE_READ_AT = 1,
    WF_LINUX_FILE_WRITE_AT = 2,
    WF_LINUX_FILE_OPEN_AT = 3,
    WF_LINUX_FILE_CLOSE = 4
};

typedef struct wf_linux_file_request {
    enum wf_linux_file_operation_kind kind;
    /* A transfer or a close names the operated descriptor; an open names the
     * directory its path is resolved against. */
    int descriptor;
    union {
        void *read_buffer;
        const void *write_buffer;
        /* Caller-owned until the operation reaches terminal, exactly like a
         * transfer buffer. */
        const char *path;
    } buffer;
    size_t count;
    uint64_t offset;
    int open_flags;
    unsigned open_mode;
    unsigned has_open_mode;
    enum wf_file_expected_kind expected_kind;
} wf_linux_file_request;

typedef struct wf_linux_file_result {
    enum wf_linux_file_operation_kind kind;
    int64_t value;
    int error_code;
    /* Meaningful for WF_LINUX_FILE_OPEN_AT; WF_FILE_OPEN_SUCCEEDED otherwise. */
    enum wf_file_open_outcome open_outcome;
} wf_linux_file_result;

enum wf_linux_io_uring_submit_result {
    WF_LINUX_IO_URING_TARGET_OWNS = 0,
    WF_LINUX_IO_URING_WAIT_CAPACITY = 1,
    WF_LINUX_IO_URING_SUBMIT_STALE = 2,
    WF_LINUX_IO_URING_SUBMIT_INVALID = 3,
    WF_LINUX_IO_URING_UNAVAILABLE = 4
};

enum wf_linux_io_uring_entry_state {
    WF_LINUX_IO_URING_ENTRY_FREE = 0,
    WF_LINUX_IO_URING_ENTRY_RESERVED = 1,
    WF_LINUX_IO_URING_ENTRY_IN_FLIGHT = 2,
    WF_LINUX_IO_URING_ENTRY_RETRY_PENDING = 3
};

typedef struct wf_linux_io_uring_entry {
    _Atomic unsigned state;
    wf_completion_token token;
    enum wf_linux_file_operation_kind kind;
    wf_linux_file_request request;
    unsigned waiting_readiness;
    /* An open's answer, decided when its completion is reaped. The descriptor
     * is named even where the kind check refused and disposed of it, which is
     * what the direct path reports too. */
    int opened_descriptor;
    enum wf_file_open_outcome open_outcome;
    int open_error;
} wf_linux_io_uring_entry;

typedef struct wf_linux_io_uring_statistics {
    uint64_t submissions;
    uint64_t capacity_waits;
    uint64_t completions;
    uint64_t publication_failures;
    uint64_t enter_failures;
    uint64_t kernel_waits;
    uint64_t kernel_wakes;
    uint64_t host_wake_writes;
} wf_linux_io_uring_statistics;

/* Target-private protocol state.  The mmap pointers below name shared
 * kernel/runtime queue pages; they are never Whitefoot buffers and never
 * cross the generated-code ABI.  A request buffer crosses this boundary only
 * through wf_linux_file_request after its loan has moved into one owned
 * completion operation. */
typedef struct wf_linux_io_uring_adapter {
    wf_completion_runtime *runtime;
    wf_linux_io_uring_entry *entries;
    size_t entry_capacity;
    _Atomic size_t entry_cursor;
    _Atomic size_t in_flight;

    int ring_descriptor;
    int wait_descriptor;
    int wake_descriptor;
    void *submission_mapping;
    size_t submission_mapping_size;
    void *completion_mapping;
    size_t completion_mapping_size;
    struct io_uring_sqe *submission_entries;
    size_t submission_entries_size;

    unsigned *submission_head;
    unsigned *submission_tail;
    unsigned *submission_mask;
    unsigned *submission_count;
    unsigned *submission_array;
    unsigned *completion_head;
    unsigned *completion_tail;
    unsigned *completion_mask;
    unsigned *completion_count;
    unsigned *completion_overflow;
    struct io_uring_cqe *completion_entries;

    pthread_mutex_t submission_lock;
    pthread_mutex_t completion_lock;
    unsigned initialized;

    _Atomic uint64_t stat_submissions;
    _Atomic uint64_t stat_capacity_waits;
    _Atomic uint64_t stat_completions;
    _Atomic uint64_t stat_publication_failures;
    _Atomic uint64_t stat_enter_failures;
    _Atomic uint64_t stat_kernel_waits;
    _Atomic uint64_t stat_kernel_wakes;
    _Atomic uint64_t stat_host_wake_writes;
    _Atomic int progress_error;
} wf_linux_io_uring_adapter;

/* `entry_storage` is the entire userspace operation capacity.  Initialization
 * requires IORING_FEAT_NODROP so an accepted operation can never silently
 * lose its unique CQE.  Any setup or qualification failure occurs before a
 * completion token is touched. */
int wf_linux_io_uring_init(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_linux_io_uring_entry *entry_storage,
    size_t entry_capacity
);

enum wf_linux_io_uring_submit_result wf_linux_io_uring_submit(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_token token,
    const wf_linux_file_request *request
);

/* Kicks staged SQEs, then publishes at most `budget` CQEs.  If `wait` is
 * nonzero and no CQE is ready, one scheduler lane may wait in io_uring_enter;
 * submissions use a different lock and remain possible while it sleeps.
 * Returns zero or an errno value and always reports the number published. */
int wf_linux_io_uring_progress(
    wf_linux_io_uring_adapter *adapter,
    size_t budget,
    unsigned wait,
    size_t *published
);

/* Announces any completion-core epoch change to the Linux wait set.  This is
 * the callback installed with wf_completion_set_wake_callback; it only writes
 * the bounded eventfd counter and never runs a continuation. */
void wf_linux_io_uring_notify(void *context);

/* Parks one scheduler lane on the union of the ring CQ and the completion
 * core's compute/capacity endpoint.  `observed_epoch` is rechecked before the
 * kernel wait, closing completion-before-park without a polling timeout.
 * UINT32_MAX means no deadline. */
int wf_linux_io_uring_park(
    wf_linux_io_uring_adapter *adapter,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
);

/* A nonzero value is a sticky target-owned progress failure.  Once an
 * operation has been accepted, the bridge must fail-stop on this condition;
 * it may neither fall back nor leave the operation waiting silently. */
int wf_linux_io_uring_progress_error(
    const wf_linux_io_uring_adapter *adapter
);

size_t wf_linux_io_uring_in_flight(
    const wf_linux_io_uring_adapter *adapter
);

/* Refuses with EBUSY until every accepted operation has produced and
 * published its CQE. */
int wf_linux_io_uring_destroy(wf_linux_io_uring_adapter *adapter);

wf_linux_io_uring_statistics wf_linux_io_uring_statistics_snapshot(
    const wf_linux_io_uring_adapter *adapter
);

#if defined(__cplusplus)
}
#endif

#endif

#endif
