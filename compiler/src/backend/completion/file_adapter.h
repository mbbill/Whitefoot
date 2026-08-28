#ifndef WHITEFOOT_COMPLETION_FILE_ADAPTER_H
#define WHITEFOOT_COMPLETION_FILE_ADAPTER_H

/*
 * Bounded POSIX file fallback adapter.
 *
 * A queue entry is a closed, typed descriptor.  There is no callback or
 * writer thunk in this interface.  Helper threads execute only the switch in
 * file_adapter.c, publish a terminal result into the completion core, and
 * return to the target queue.  A configuration with zero helpers is valid:
 * the scheduler advances the same queue with wf_file_adapter_progress.
 */

#include "contract.h"

#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <sys/stat.h>

#if defined(__cplusplus)
extern "C" {
#endif

#define WF_FILE_STATUS_CAPACITY 192u

/* The path bytes one submitted open resolves, held by the operation record.
 *
 * A submitted open outlives the call that formed it, so the caller's name
 * buffer must stop being the operation's storage the moment submission
 * returns: the writer regains that buffer then and may rewrite it while the
 * host is still resolving the name.  Every target adapter therefore copies
 * the bytes into its own record at submission and resolves that copy.
 *
 * The bound is every admitted name.  A generated open resolves exactly one
 * relative component, and the widest qualified component limit is Darwin's
 * 1023 bytes, which this holds together with its terminator; the Linux family
 * admits 255.  The runtime's own harness resolves absolute scratch paths
 * instead, and 1024 is Darwin's whole `PATH_MAX`.  Storage is bounded and
 * static because a submission may not allocate.
 *
 * A name that does not fit is refused before an operation is claimed, and the
 * caller opens it directly instead — that path resolves the caller's buffer
 * inside its own call and needs no copy.  It is a throughput fallback of the
 * same class as a full queue, never a changed outcome. */
#define WF_FILE_PATH_CAPACITY 1024u

/* Copies one path into an operation record's own storage.  Returns zero for a
 * name that does not fit, which every caller must answer before claiming an
 * operation; truncating a name would resolve a different file. */
static inline int wf_file_stage_path(char *storage, const char *path) {
    size_t length;
    if (storage == NULL || path == NULL) {
        return 0;
    }
    length = strlen(path);
    if (length >= (size_t)WF_FILE_PATH_CAPACITY) {
        return 0;
    }
    memcpy(storage, path, length + 1u);
    return 1;
}

/* Whether one path can become an operation record's own bytes.  Asked before
 * an operation is claimed, so a refusal is an honest fallback to the direct
 * path rather than a fail-stop after ownership moved. */
static inline int wf_file_path_fits(const char *path) {
    return path != NULL && strlen(path) < (size_t)WF_FILE_PATH_CAPACITY;
}

enum wf_file_operation_kind {
    WF_FILE_OPEN_AT = 1,
    WF_FILE_READ = 2,
    WF_FILE_WRITE = 3,
    WF_FILE_PREAD = 4,
    WF_FILE_PWRITE = 5,
    WF_FILE_STATUS = 6,
    WF_FILE_CLOSE = 7,
#if defined(__APPLE__)
    WF_FILE_GETDIRENTRIES64 = 8,
#endif
};

enum wf_file_expected_kind {
    WF_FILE_EXPECT_ANY = 0,
    WF_FILE_EXPECT_REGULAR = 1,
    WF_FILE_EXPECT_DIRECTORY = 2
};

enum wf_file_open_outcome {
    WF_FILE_OPEN_SUCCEEDED = 0,
    WF_FILE_OPEN_FAILED = 1,
    WF_FILE_OPEN_STATUS_FAILED = 2,
    WF_FILE_OPEN_IS_DIRECTORY = 3,
    WF_FILE_OPEN_OTHER_KIND = 4
};

/* The one rule deciding whether an opened descriptor is the kind the
 * operation asked for.  Every target adapter answers with this function, so a
 * FIFO is refused identically whether the open ran on a helper thread, on the
 * scheduler itself, or on a kernel completion ring.  `file_mode` is the host
 * mode word of the descriptor that was actually opened, never of a path
 * inspected a second time. */
static inline enum wf_file_open_outcome wf_file_kind_outcome(
    enum wf_file_expected_kind expected,
    unsigned int file_mode
) {
    if (expected == WF_FILE_EXPECT_ANY
        || (expected == WF_FILE_EXPECT_REGULAR && S_ISREG(file_mode))
        || (expected == WF_FILE_EXPECT_DIRECTORY && S_ISDIR(file_mode))) {
        return WF_FILE_OPEN_SUCCEEDED;
    }
    return S_ISDIR(file_mode) ? WF_FILE_OPEN_IS_DIRECTORY
                              : WF_FILE_OPEN_OTHER_KIND;
}

/* The extra open flag a kind-checked open needs so that opening a waiting
 * facility such as a FIFO cannot block before the kind is known. */
static inline int wf_file_open_kind_flags(enum wf_file_expected_kind expected) {
    return expected == WF_FILE_EXPECT_REGULAR ? O_NONBLOCK : 0;
}

typedef struct wf_file_request {
    enum wf_file_operation_kind kind;
    union {
        struct {
            int directory;
            /* Names the operation record's own bytes once the request is
             * queued; the caller's buffer only while the request is still the
             * caller's, which is the direct path and the moment before
             * `wf_file_work_bind_path` runs. */
            const char *path;
            int flags;
            unsigned mode;
            unsigned has_mode;
            enum wf_file_expected_kind expected_kind;
        } open_at;
        struct {
            int descriptor;
            void *buffer;
            size_t count;
        } read;
        struct {
            int descriptor;
            const void *buffer;
            size_t count;
        } write;
        struct {
            int descriptor;
            void *buffer;
            size_t count;
            int64_t offset;
        } pread;
        struct {
            int descriptor;
            const void *buffer;
            size_t count;
            int64_t offset;
        } pwrite;
        struct {
            int descriptor;
        } status;
        struct {
            int descriptor;
        } close;
#if defined(__APPLE__)
        struct {
            int descriptor;
            void *buffer;
            size_t count;
            int64_t *position;
        } getdirentries64;
#endif
    } operation;
} wf_file_request;

typedef struct wf_file_result {
    enum wf_file_operation_kind kind;
    int64_t value;
    int error_code;
    enum wf_file_open_outcome open_outcome;
    size_t status_size;
    unsigned char status[WF_FILE_STATUS_CAPACITY];
} wf_file_result;

typedef struct wf_file_work {
    wf_completion_token token;
    wf_file_request request;
    /* An open's path bytes, owned by this record. */
    char path_storage[WF_FILE_PATH_CAPACITY];
} wf_file_work;

/* Points a staged open at this record's own path bytes.
 *
 * A work record is copied whenever it moves — into the bounded queue at
 * submission, out of it at execution — and a pointer into the record it was
 * copied *from* would name storage the queue is free to reuse.  Every copy
 * therefore rebinds, and the invariant is that a work record's open never
 * names bytes outside itself. */
static inline void wf_file_work_bind_path(wf_file_work *work) {
    if (work != NULL && work->request.kind == WF_FILE_OPEN_AT) {
        work->request.operation.open_at.path = work->path_storage;
    }
}

enum wf_file_submit_result {
    WF_FILE_TARGET_OWNS = 0,
    WF_FILE_WAIT_CAPACITY = 1,
    WF_FILE_SUBMIT_STALE = 2,
    WF_FILE_SUBMIT_INVALID = 3,
    WF_FILE_ADAPTER_STOPPING = 4
};

typedef struct wf_file_adapter_statistics {
    uint64_t submissions;
    uint64_t capacity_waits;
    uint64_t helper_executions;
    uint64_t scheduler_executions;
    uint64_t publication_failures;
} wf_file_adapter_statistics;

typedef struct wf_file_adapter {
    wf_completion_runtime *runtime;
    wf_file_work *queue;
    size_t queue_capacity;
    size_t queue_head;
    size_t queue_tail;
    size_t queue_count;
    pthread_t *helpers;
    /* Grown under queue_lock by a submitting thread and read without it by a
     * scheduler deciding whether it is itself this queue's engine. */
    _Atomic size_t helper_count;
    size_t helper_cap;
    pthread_mutex_t queue_lock;
    pthread_cond_t queue_available;
    unsigned stopping;
    unsigned initialized;

    _Atomic uint64_t stat_submissions;
    _Atomic uint64_t stat_capacity_waits;
    _Atomic uint64_t stat_helper_executions;
    _Atomic uint64_t stat_scheduler_executions;
    _Atomic uint64_t stat_publication_failures;
} wf_file_adapter;

/* The caller owns queue_storage and helper_storage until shutdown completes.
 * helper_count is policy, not a fixed architecture constant; zero selects
 * scheduler-driven single-thread progress. */
int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_file_work *queue_storage,
    size_t queue_capacity,
    pthread_t *helper_storage,
    size_t helper_count
);

enum wf_file_submit_result wf_file_adapter_submit(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
);

/* Executes one typed request to its first terminal host answer. EINTR and
 * read/write/directory readiness refusal are adapter progress; close is never
 * retried because one ambiguous close attempt has already consumed authority.
 * Other non-transfer operations remain exactly one host attempt. */
wf_file_result wf_file_execute_direct(const wf_file_request *request);

/* Executes at most `budget` typed requests on the calling scheduler thread.
 * It never runs a Whitefoot continuation. */
size_t wf_file_adapter_progress(wf_file_adapter *adapter, size_t budget);

size_t wf_file_adapter_queued(const wf_file_adapter *adapter);

/* Raises the ceiling the helper pool may grow to when a program exposes more
 * independent queued work than there are helpers to take it. `helper_storage`
 * given to init must hold `cap` helpers. A cap at or below the initial count
 * disables growth, which is what a pinned helper policy asks for. */
int wf_file_adapter_set_helper_cap(wf_file_adapter *adapter, size_t cap);

/* Read without the queue lock. Zero means the calling scheduler is itself the
 * only engine this queue has. */
size_t wf_file_adapter_helper_count(const wf_file_adapter *adapter);

/* Stops admission and drains accepted queue entries before joining helpers.
 * With zero helpers, the calling thread performs the bounded typed work one
 * entry at a time until the accepted queue is empty. */
int wf_file_adapter_shutdown(wf_file_adapter *adapter);

wf_file_adapter_statistics wf_file_adapter_statistics_snapshot(
    const wf_file_adapter *adapter
);

#if defined(__cplusplus)
}
#endif

#endif
