#ifndef WHITEFOOT_WINDOWS_BLOCKING_H
#define WHITEFOOT_WINDOWS_BLOCKING_H

#if defined(_WIN32)

#include "windows_completion.h"
#include "windows_iocp.h"

#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <windows.h>

#if defined(__cplusplus)
extern "C" {
#endif

/* Production gives the blocking adapter and completion core the same fixed
 * capacity. Every occupied adapter entry owns one live core token, so a
 * successful core claim proves that at least one adapter entry is free. A
 * smaller test-only core preserves the same inequality. */
#define WF_WINDOWS_BLOCKING_CAPACITY 64u
#define WF_WINDOWS_BLOCKING_MAX_WORKERS 64u
#define WF_WINDOWS_BLOCKING_PATH_UNITS 32768u

typedef struct wf_windows_blocking_request {
    enum wf_windows_file_operation_kind kind;
    union {
        struct {
            int descriptor;
            const void *buffer;
            uint64_t count;
        } write;
        struct {
            int directory;
            const char *path;
            size_t path_units;
            int flags;
            unsigned mode;
            unsigned has_mode;
            unsigned expected_kind;
            unsigned descriptor_class;
        } open_at;
        struct {
            int descriptor;
            void *buffer;
            uint64_t count;
            int64_t *position;
        } directory_next;
    } operation;
    wf_windows_descriptor_lease lease;
} wf_windows_blocking_request;

enum wf_windows_blocking_entry_state {
    WF_WINDOWS_BLOCKING_ENTRY_FREE = 0,
    WF_WINDOWS_BLOCKING_ENTRY_RESERVED = 1,
    WF_WINDOWS_BLOCKING_ENTRY_QUEUED = 2,
    WF_WINDOWS_BLOCKING_ENTRY_RUNNING = 3
};

typedef struct wf_windows_blocking_entry {
    _Atomic unsigned state;
    wf_completion_token token;
    wf_windows_blocking_request request;
    uint16_t path[WF_WINDOWS_BLOCKING_PATH_UNITS];
} wf_windows_blocking_entry;

typedef struct wf_windows_blocking_statistics {
    uint64_t submissions;
    uint64_t executions;
    uint64_t publications;
    uint64_t queue_capacity_failures;
    size_t in_flight;
    size_t worker_count;
} wf_windows_blocking_statistics;

typedef struct wf_windows_blocking_adapter {
    wf_completion_runtime *runtime;
    wf_windows_blocking_entry entries[WF_WINDOWS_BLOCKING_CAPACITY];
    HANDLE port;
    HANDLE ready_event;
    HANDLE workers[WF_WINDOWS_BLOCKING_MAX_WORKERS];
    size_t worker_count;
    _Atomic size_t ready_workers;
    _Atomic size_t queued;
    _Atomic size_t in_flight;
    _Atomic uint64_t stat_submissions;
    _Atomic uint64_t stat_executions;
    _Atomic uint64_t stat_publications;
    _Atomic uint64_t stat_queue_capacity_failures;
    _Atomic unsigned stopping;
    _Atomic unsigned initialized;
} wf_windows_blocking_adapter;

enum wf_windows_blocking_submit_result {
    WF_WINDOWS_BLOCKING_TARGET_OWNS = 0,
    WF_WINDOWS_BLOCKING_SUBMIT_STALE = 1,
    WF_WINDOWS_BLOCKING_SUBMIT_INVALID = 2,
    WF_WINDOWS_BLOCKING_CAPACITY_INVARIANT_BROKEN = 3
};

int wf_windows_blocking_init(
    wf_windows_blocking_adapter *adapter,
    wf_completion_runtime *runtime
);

enum wf_windows_blocking_submit_result wf_windows_blocking_submit(
    wf_windows_blocking_adapter *adapter,
    wf_completion_token token,
    const wf_windows_blocking_request *request
);

int wf_windows_blocking_shutdown(wf_windows_blocking_adapter *adapter);

wf_windows_blocking_statistics wf_windows_blocking_statistics_snapshot(
    const wf_windows_blocking_adapter *adapter
);

#if defined(__cplusplus)
}
#endif

#endif

#endif
