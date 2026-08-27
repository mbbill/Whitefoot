#ifndef WHITEFOOT_WINDOWS_IOCP_H
#define WHITEFOOT_WINDOWS_IOCP_H

#if defined(_WIN32)

#include "native_completion_api.h"

#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <windows.h>

#if defined(__cplusplus)
extern "C" {
#endif

enum wf_windows_file_operation_kind {
    WF_WINDOWS_FILE_READ_AT = 1,
    WF_WINDOWS_FILE_WRITE_AT = 2
};

struct wf_windows_iocp_adapter;

/* Only the adapter association function can mint this internal handle.  The
 * underlying file must have been opened with FILE_FLAG_OVERLAPPED and must
 * remain adapter-owned until every operation using it is terminal. */
typedef struct wf_windows_iocp_file {
    HANDLE handle;
    struct wf_windows_iocp_adapter *adapter;
} wf_windows_iocp_file;

typedef struct wf_windows_file_request {
    enum wf_windows_file_operation_kind kind;
    wf_windows_iocp_file file;
    union {
        void *read_buffer;
        const void *write_buffer;
    } buffer;
    size_t count;
    uint64_t offset;
} wf_windows_file_request;

typedef struct wf_windows_file_result {
    enum wf_windows_file_operation_kind kind;
    int64_t value;
    uint32_t error_code;
} wf_windows_file_result;

enum wf_windows_iocp_submit_result {
    WF_WINDOWS_IOCP_TARGET_OWNS = 0,
    WF_WINDOWS_IOCP_WAIT_CAPACITY = 1,
    WF_WINDOWS_IOCP_SUBMIT_STALE = 2,
    WF_WINDOWS_IOCP_SUBMIT_INVALID = 3,
    WF_WINDOWS_IOCP_UNAVAILABLE = 4
};

enum wf_windows_iocp_entry_state {
    WF_WINDOWS_IOCP_ENTRY_FREE = 0,
    WF_WINDOWS_IOCP_ENTRY_RESERVED = 1,
    WF_WINDOWS_IOCP_ENTRY_IN_FLIGHT = 2
};

/* OVERLAPPED is first so the completion packet maps to a stable preallocated
 * operation entry without a side table or allocation. */
typedef struct wf_windows_iocp_entry {
    OVERLAPPED overlapped;
    _Atomic unsigned state;
    wf_completion_token token;
    enum wf_windows_file_operation_kind kind;
} wf_windows_iocp_entry;

typedef struct wf_windows_iocp_statistics {
    uint64_t submissions;
    uint64_t capacity_waits;
    uint64_t immediate_failures;
    uint64_t completions;
    uint64_t publication_failures;
} wf_windows_iocp_statistics;

/* Target-private protocol state.  The shared completion port and OVERLAPPED
 * entries are not Whitefoot memory and never cross the generated-code ABI.
 * Only the buffer loan carried by a typed request enters an owned operation. */
typedef struct wf_windows_iocp_adapter {
    wf_completion_runtime *runtime;
    HANDLE port;
    ULONG_PTR wake_key;
    wf_windows_iocp_entry *entries;
    size_t entry_capacity;
    _Atomic size_t entry_cursor;
    _Atomic size_t in_flight;
    unsigned initialized;

    _Atomic uint64_t stat_submissions;
    _Atomic uint64_t stat_capacity_waits;
    _Atomic uint64_t stat_immediate_failures;
    _Atomic uint64_t stat_completions;
    _Atomic uint64_t stat_publication_failures;
} wf_windows_iocp_adapter;

/* Creates one shared IOCP.  `entry_storage` is the entire userspace operation
 * capacity; there is no per-operation allocation. */
int wf_windows_iocp_init(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_windows_iocp_entry *entry_storage,
    size_t entry_capacity,
    DWORD concurrency
);

/* Associates an overlapped file with this adapter's shared port before any
 * completion token changes ownership. */
int wf_windows_iocp_associate_file(
    wf_windows_iocp_adapter *adapter,
    HANDLE handle,
    wf_windows_iocp_file *file
);

enum wf_windows_iocp_submit_result wf_windows_iocp_submit(
    wf_windows_iocp_adapter *adapter,
    wf_completion_token token,
    const wf_windows_file_request *request
);

/* Dequeues at most `budget` packets and reports only successfully published
 * terminal operations.  The first dequeue may wait for
 * `timeout_milliseconds`; subsequent dequeues are nonblocking.  A validated
 * null-OVERLAPPED scheduler wake consumes budget but is not a terminal. */
int wf_windows_iocp_progress(
    wf_windows_iocp_adapter *adapter,
    size_t budget,
    DWORD timeout_milliseconds,
    size_t *published
);

HANDLE wf_windows_iocp_port(const wf_windows_iocp_adapter *adapter);
ULONG_PTR wf_windows_iocp_wake_key(const wf_windows_iocp_adapter *adapter);
size_t wf_windows_iocp_in_flight(const wf_windows_iocp_adapter *adapter);
int wf_windows_iocp_destroy(wf_windows_iocp_adapter *adapter);

wf_windows_iocp_statistics wf_windows_iocp_statistics_snapshot(
    const wf_windows_iocp_adapter *adapter
);

#if defined(__cplusplus)
}
#endif

#endif

#endif
