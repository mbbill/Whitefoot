#include "windows_blocking.h"

#if defined(_WIN32)

#include "windows_runtime.h"

#include <errno.h>
#include <limits.h>
#include <process.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

_Static_assert(
    WF_WINDOWS_BLOCKING_CAPACITY == 64u,
    "blocking capacity must match the production completion core"
);
_Static_assert(
    sizeof(wf_windows_file_result) <= WF_COMPLETION_RESULT_CAPACITY,
    "completion result cell must hold a Windows blocking result"
);

#define WF_WINDOWS_BLOCKING_DEFAULT_MAX_WORKERS 16u

static size_t wf_windows_blocking_worker_policy(int *error_code) {
    WCHAR configured[16];
    DWORD length = GetEnvironmentVariableW(
        L"WF_WINDOWS_BLOCKING_WORKERS",
        configured,
        (DWORD)(sizeof(configured) / sizeof(configured[0]))
    );
    size_t workers;
    DWORD environment_error;
    if (error_code == NULL) {
        return 0;
    }
    *error_code = 0;
    environment_error = length == 0 ? GetLastError() : ERROR_SUCCESS;
    if (length != 0) {
        size_t index;
        workers = 0;
        if (length >= sizeof(configured) / sizeof(configured[0])) {
            *error_code = ERROR_INVALID_PARAMETER;
            return 0;
        }
        for (index = 0; index < (size_t)length; ++index) {
            unsigned digit;
            if (configured[index] < L'0' || configured[index] > L'9') {
                *error_code = ERROR_INVALID_PARAMETER;
                return 0;
            }
            digit = (unsigned)(configured[index] - L'0');
            if (workers > (SIZE_MAX - digit) / 10u) {
                *error_code = ERROR_INVALID_PARAMETER;
                return 0;
            }
            workers = workers * 10u + digit;
        }
        if (workers == 0 || workers > WF_WINDOWS_BLOCKING_MAX_WORKERS) {
            *error_code = ERROR_INVALID_PARAMETER;
            return 0;
        }
        return workers;
    }
    if (environment_error != ERROR_ENVVAR_NOT_FOUND) {
        *error_code = environment_error == ERROR_SUCCESS
            ? ERROR_INVALID_PARAMETER
            : (int)environment_error;
        return 0;
    }
    workers = (size_t)GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
    if (workers == 0) {
        workers = 1;
    }
    if (workers > WF_WINDOWS_BLOCKING_DEFAULT_MAX_WORKERS) {
        workers = WF_WINDOWS_BLOCKING_DEFAULT_MAX_WORKERS;
    }
    return workers;
}

static int wf_windows_blocking_request_valid(
    const wf_windows_blocking_request *request
) {
    if (request == NULL || request->lease.generation == 0
        || request->lease.handle == NULL
        || request->lease.handle == INVALID_HANDLE_VALUE) {
        return 0;
    }
    switch (request->kind) {
    case WF_WINDOWS_FILE_WRITE:
        return request->operation.write.descriptor >= 0
            && request->lease.descriptor
                == request->operation.write.descriptor
            && request->lease.descriptor_class
                == WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT
            && request->lease.mode
                == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL
            && (request->operation.write.buffer != NULL
                || request->operation.write.count == 0);
    case WF_WINDOWS_FILE_OPEN_AT:
        return request->operation.open_at.directory >= 0
            && request->lease.descriptor
                == request->operation.open_at.directory
            && request->lease.descriptor_class
                == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT
            && request->lease.mode
                == WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
            && (request->operation.open_at.descriptor_class
                    == WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE
                || request->operation.open_at.descriptor_class
                    == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT
                || request->operation.open_at.descriptor_class
                    == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE)
            && request->operation.open_at.path != NULL
            && request->operation.open_at.path_units
                < WF_WINDOWS_BLOCKING_PATH_UNITS;
    case WF_WINDOWS_FILE_DIRECTORY_NEXT:
        return request->operation.directory_next.descriptor >= 0
            && request->lease.descriptor
                == request->operation.directory_next.descriptor
            && request->lease.descriptor_class
                == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE
            && request->lease.mode
                == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL
            && request->operation.directory_next.position != NULL
            && (request->operation.directory_next.buffer != NULL
                || request->operation.directory_next.count == 0);
    default:
        return 0;
    }
}

static int wf_windows_blocking_error(void) {
    int *location = wf__windows_error_location();
    return location != NULL && *location != 0 ? *location : ERROR_GEN_FAILURE;
}

static wf_windows_file_result wf_windows_blocking_execute_once(
    wf_windows_blocking_entry *entry
) {
    wf_windows_file_result result;
    memset(&result, 0, sizeof(result));
    result.kind = entry->request.kind;
    switch (entry->request.kind) {
    case WF_WINDOWS_FILE_WRITE:
        result.value = wf__windows_completion_file_write_worker(
            entry->request.lease.handle,
            entry->request.operation.write.buffer,
            entry->request.operation.write.count
        );
        if (result.value < 0) {
            result.error_code = (uint32_t)wf_windows_blocking_error();
        }
        break;
    case WF_WINDOWS_FILE_OPEN_AT: {
        int error_code = 0;
        unsigned open_outcome = 1u;
        result.value = wf__windows_completion_file_open_at_worker(
            entry->request.lease.handle,
            (const char *)(const void *)entry->path,
            entry->request.operation.open_at.flags,
            entry->request.operation.open_at.mode,
            entry->request.operation.open_at.has_mode,
            entry->request.operation.open_at.expected_kind,
            entry->request.operation.open_at.descriptor_class,
            &error_code,
            &open_outcome
        );
        result.error_code = (uint32_t)error_code;
        result.open_outcome = open_outcome;
        break;
    }
    case WF_WINDOWS_FILE_DIRECTORY_NEXT:
        result.value = wf__windows_completion_directory_next_worker(
            entry->request.lease.handle,
            entry->request.operation.directory_next.buffer,
            entry->request.operation.directory_next.count,
            entry->request.operation.directory_next.position
        );
        if (result.value < 0) {
            result.error_code = (uint32_t)wf_windows_blocking_error();
        }
        break;
    default:
        abort();
    }
    return result;
}

static void wf_windows_blocking_publish(
    wf_windows_blocking_adapter *adapter,
    wf_completion_token token,
    const wf_windows_file_result *result
) {
    wf_completion_publication publication;
    enum wf_completion_publish_result published;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = result->error_code == 0
            && (result->kind != WF_WINDOWS_FILE_OPEN_AT
                || result->open_outcome == 0u)
        ? 1u
        : 2u;
    publication.result = result;
    publication.result_size = sizeof(*result);
    published = wf_completion_publish_terminal(
        adapter->runtime,
        token,
        &publication
    );
    if (published != WF_COMPLETION_PUBLISHED) {
        abort();
    }
    atomic_fetch_add_explicit(
        &adapter->stat_publications,
        1,
        memory_order_relaxed
    );
    atomic_fetch_sub_explicit(&adapter->in_flight, 1, memory_order_relaxed);
}

static void wf_windows_blocking_run_index(
    wf_windows_blocking_adapter *adapter,
    size_t index
) {
    wf_windows_blocking_entry *entry;
    wf_completion_token token;
    wf_windows_file_result result;
    unsigned expected = WF_WINDOWS_BLOCKING_ENTRY_QUEUED;
    if (adapter == NULL || index >= adapter->runtime->slot_count
        || index >= WF_WINDOWS_BLOCKING_CAPACITY) {
        abort();
    }
    entry = &adapter->entries[index];
    if (!atomic_compare_exchange_strong_explicit(
            &entry->state,
            &expected,
            WF_WINDOWS_BLOCKING_ENTRY_RUNNING,
            memory_order_acquire,
            memory_order_relaxed
        )) {
        abort();
    }
    atomic_fetch_add_explicit(
        &adapter->stat_executions,
        1,
        memory_order_relaxed
    );
    result = wf_windows_blocking_execute_once(entry);
    wf__windows_completion_descriptor_lease_release(
        &entry->request.lease
    );
    token = entry->token;
    /* No job byte is needed after the host call. Free the job before the
     * terminal publication makes this core token consumable: otherwise a
     * fast consume/reclaim could require a 65th job entry while this old
     * worker still owned one, or a late FREE could overwrite its reuse. */
    atomic_store_explicit(
        &entry->state,
        WF_WINDOWS_BLOCKING_ENTRY_FREE,
        memory_order_release
    );
    wf_windows_blocking_publish(adapter, token, &result);
}

static unsigned __stdcall wf_windows_blocking_worker(void *context) {
    wf_windows_blocking_adapter *adapter =
        (wf_windows_blocking_adapter *)context;
    size_t ready = atomic_fetch_add_explicit(
        &adapter->ready_workers,
        1,
        memory_order_acq_rel
    ) + 1u;
    if (ready > adapter->worker_count
        || (ready == adapter->worker_count
            && SetEvent(adapter->ready_event) == FALSE)) {
        abort();
    }
    for (;;) {
        DWORD bytes = 0;
        ULONG_PTR key = 0;
        OVERLAPPED *overlapped = NULL;
        wf_windows_blocking_entry *entry;
        if (GetQueuedCompletionStatus(
                adapter->port,
                &bytes,
                &key,
                &overlapped,
                INFINITE
            ) == FALSE) {
            abort();
        }
        if (key == 0 && bytes == 0 && overlapped == NULL) {
            return 0;
        }
        if (key == 0 || key > WF_WINDOWS_BLOCKING_CAPACITY
            || bytes != 0 || overlapped != NULL) {
            abort();
        }
        entry = &adapter->entries[key - 1u];
        if (atomic_load_explicit(&entry->state, memory_order_acquire)
                != WF_WINDOWS_BLOCKING_ENTRY_QUEUED) {
            abort();
        }
        if (atomic_fetch_sub_explicit(
                &adapter->queued,
                1,
                memory_order_seq_cst
            ) == 0) {
            abort();
        }
        wf_windows_blocking_run_index(adapter, (size_t)key - 1u);
    }
}

static void wf_windows_blocking_stop_started(
    wf_windows_blocking_adapter *adapter,
    size_t started
) {
    size_t index;
    for (index = 0; index < started; ++index) {
        if (PostQueuedCompletionStatus(adapter->port, 0, 0, NULL) == FALSE) {
            abort();
        }
    }
    for (index = 0; index < started; ++index) {
        if (WaitForSingleObject(adapter->workers[index], INFINITE)
            != WAIT_OBJECT_0) {
            abort();
        }
        if (CloseHandle(adapter->workers[index]) == FALSE) {
            abort();
        }
        adapter->workers[index] = NULL;
    }
}

int wf_windows_blocking_init(
    wf_windows_blocking_adapter *adapter,
    wf_completion_runtime *runtime
) {
    size_t index;
    int error_code;
    size_t worker_count;
    if (adapter == NULL || runtime == NULL || runtime->slot_count == 0
        || runtime->slot_count > WF_WINDOWS_BLOCKING_CAPACITY) {
        return EINVAL;
    }
    memset(adapter, 0, sizeof(*adapter));
    adapter->runtime = runtime;
    atomic_init(&adapter->ready_workers, 0);
    atomic_init(&adapter->queued, 0);
    atomic_init(&adapter->in_flight, 0);
    atomic_init(&adapter->stopping, 0);
    atomic_init(&adapter->initialized, 0);
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_executions, 0);
    atomic_init(&adapter->stat_publications, 0);
    atomic_init(&adapter->stat_queue_capacity_failures, 0);
    for (index = 0; index < WF_WINDOWS_BLOCKING_CAPACITY; ++index) {
        atomic_init(
            &adapter->entries[index].state,
            WF_WINDOWS_BLOCKING_ENTRY_FREE
        );
    }
    worker_count = wf_windows_blocking_worker_policy(&error_code);
    if (error_code != 0) {
        return error_code;
    }
    adapter->worker_count = worker_count;
    adapter->port = CreateIoCompletionPort(
        INVALID_HANDLE_VALUE,
        NULL,
        0,
        (DWORD)worker_count
    );
    if (adapter->port == NULL) {
        return (int)GetLastError();
    }
    adapter->ready_event = CreateEventW(NULL, TRUE, FALSE, NULL);
    if (adapter->ready_event == NULL) {
        error_code = (int)GetLastError();
        (void)CloseHandle(adapter->port);
        adapter->port = NULL;
        return error_code;
    }
    for (index = 0; index < worker_count; ++index) {
        uintptr_t thread = _beginthreadex(
            NULL,
            0,
            wf_windows_blocking_worker,
            adapter,
            0,
            NULL
        );
        if (thread == 0) {
            error_code = errno == ENOMEM
                ? ERROR_NOT_ENOUGH_MEMORY
                : ERROR_GEN_FAILURE;
            wf_windows_blocking_stop_started(adapter, index);
            (void)CloseHandle(adapter->ready_event);
            (void)CloseHandle(adapter->port);
            adapter->ready_event = NULL;
            adapter->port = NULL;
            return error_code;
        }
        adapter->workers[index] = (HANDLE)thread;
    }
    if (WaitForSingleObject(adapter->ready_event, INFINITE)
        != WAIT_OBJECT_0) {
        error_code = (int)GetLastError();
        wf_windows_blocking_stop_started(adapter, worker_count);
        (void)CloseHandle(adapter->ready_event);
        (void)CloseHandle(adapter->port);
        adapter->ready_event = NULL;
        adapter->port = NULL;
        return error_code == ERROR_SUCCESS ? ERROR_GEN_FAILURE : error_code;
    }
    atomic_store_explicit(&adapter->initialized, 1u, memory_order_release);
    return 0;
}

enum wf_windows_blocking_submit_result wf_windows_blocking_submit(
    wf_windows_blocking_adapter *adapter,
    wf_completion_token token,
    const wf_windows_blocking_request *request
) {
    size_t index;
    unsigned expected;
    wf_windows_blocking_entry *entry;
    enum wf_completion_transition_result transition;
    if (adapter == NULL
        || atomic_load_explicit(&adapter->initialized, memory_order_acquire)
            == 0
        || atomic_load_explicit(&adapter->stopping, memory_order_acquire) != 0
        || !wf_windows_blocking_request_valid(request)) {
        return WF_WINDOWS_BLOCKING_SUBMIT_INVALID;
    }
    index = (size_t)token.slot;
    if (index >= adapter->runtime->slot_count
        || index >= WF_WINDOWS_BLOCKING_CAPACITY) {
        return WF_WINDOWS_BLOCKING_SUBMIT_INVALID;
    }
    entry = &adapter->entries[index];
    expected = WF_WINDOWS_BLOCKING_ENTRY_FREE;
    if (!atomic_compare_exchange_strong_explicit(
            &entry->state,
            &expected,
            WF_WINDOWS_BLOCKING_ENTRY_RESERVED,
            memory_order_acquire,
            memory_order_relaxed
        )) {
        atomic_fetch_add_explicit(
            &adapter->stat_queue_capacity_failures,
            1,
            memory_order_relaxed
        );
        return WF_WINDOWS_BLOCKING_CAPACITY_INVARIANT_BROKEN;
    }
    entry->token = token;
    entry->request = *request;
    if (request->kind == WF_WINDOWS_FILE_OPEN_AT) {
        size_t bytes = request->operation.open_at.path_units
            * sizeof(uint16_t);
        memcpy(entry->path, request->operation.open_at.path, bytes);
        entry->path[request->operation.open_at.path_units] = 0;
        entry->request.operation.open_at.path =
            (const char *)(const void *)entry->path;
    }
    transition = wf_completion_begin_submit(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        atomic_store_explicit(
            &entry->state,
            WF_WINDOWS_BLOCKING_ENTRY_FREE,
            memory_order_release
        );
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_WINDOWS_BLOCKING_SUBMIT_STALE
            : WF_WINDOWS_BLOCKING_SUBMIT_INVALID;
    }
    if (request->kind == WF_WINDOWS_FILE_OPEN_AT
        && wf_completion_publish_milestone(
               adapter->runtime,
               token,
               WF_COMPLETION_PAYLOAD_RELEASED
           ) != WF_COMPLETION_PUBLISHED) {
        abort();
    }
    transition = wf_completion_target_accepted(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        atomic_store_explicit(
            &entry->state,
            WF_WINDOWS_BLOCKING_ENTRY_FREE,
            memory_order_release
        );
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_WINDOWS_BLOCKING_SUBMIT_STALE
            : WF_WINDOWS_BLOCKING_SUBMIT_INVALID;
    }
    atomic_fetch_add_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );
    atomic_store_explicit(
        &entry->state,
        WF_WINDOWS_BLOCKING_ENTRY_QUEUED,
        memory_order_release
    );
    atomic_fetch_add_explicit(&adapter->queued, 1, memory_order_relaxed);
    if (PostQueuedCompletionStatus(
            adapter->port,
            0,
            (ULONG_PTR)(index + 1u),
            NULL
        ) == FALSE) {
        abort();
    }
    return WF_WINDOWS_BLOCKING_TARGET_OWNS;
}

int wf_windows_blocking_shutdown(wf_windows_blocking_adapter *adapter) {
    size_t index;
    DWORD waited;
    ULONGLONG started;
    unsigned expected = 0;
    if (adapter == NULL
        || atomic_load_explicit(&adapter->initialized, memory_order_acquire)
            == 0) {
        return EINVAL;
    }
    if (!atomic_compare_exchange_strong_explicit(
            &adapter->stopping,
            &expected,
            1u,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
        return EBUSY;
    }
    /* A consumer can reach process exit immediately after terminal
     * publication while the publishing worker is still lowering its adapter
     * counters. Stop packets must not enter the work port during that
     * tail. */
    started = GetTickCount64();
    while (atomic_load_explicit(&adapter->in_flight, memory_order_acquire)
           != 0) {
        if (GetTickCount64() - started > 30000u) {
            return EBUSY;
        }
        (void)SwitchToThread();
    }
    if (atomic_load_explicit(&adapter->queued, memory_order_acquire) != 0) {
        abort();
    }
    for (index = 0; index < adapter->worker_count; ++index) {
        if (PostQueuedCompletionStatus(adapter->port, 0, 0, NULL) == FALSE) {
            abort();
        }
    }
    waited = WaitForMultipleObjects(
        (DWORD)adapter->worker_count,
        adapter->workers,
        TRUE,
        INFINITE
    );
    if (waited != WAIT_OBJECT_0) {
        return (int)GetLastError();
    }
    if (atomic_load_explicit(&adapter->queued, memory_order_acquire) != 0
        || atomic_load_explicit(&adapter->in_flight, memory_order_acquire)
            != 0
        || atomic_load_explicit(
               &adapter->stat_submissions,
               memory_order_acquire
           ) != atomic_load_explicit(
               &adapter->stat_publications,
               memory_order_acquire
           )) {
        return EBUSY;
    }
    for (index = 0; index < WF_WINDOWS_BLOCKING_CAPACITY; ++index) {
        if (atomic_load_explicit(
                &adapter->entries[index].state,
                memory_order_acquire
            ) != WF_WINDOWS_BLOCKING_ENTRY_FREE) {
            return EBUSY;
        }
    }
    for (index = 0; index < adapter->worker_count; ++index) {
        if (CloseHandle(adapter->workers[index]) == FALSE) {
            return (int)GetLastError();
        }
        adapter->workers[index] = NULL;
    }
    if (CloseHandle(adapter->ready_event) == FALSE
        || CloseHandle(adapter->port) == FALSE) {
        return (int)GetLastError();
    }
    adapter->ready_event = NULL;
    adapter->port = NULL;
    atomic_store_explicit(&adapter->initialized, 0u, memory_order_release);
    return 0;
}

wf_windows_blocking_statistics wf_windows_blocking_statistics_snapshot(
    const wf_windows_blocking_adapter *adapter
) {
    wf_windows_blocking_statistics statistics;
    memset(&statistics, 0, sizeof(statistics));
    if (adapter == NULL) {
        return statistics;
    }
    statistics.submissions = atomic_load_explicit(
        &adapter->stat_submissions,
        memory_order_relaxed
    );
    statistics.executions = atomic_load_explicit(
        &adapter->stat_executions,
        memory_order_relaxed
    );
    statistics.publications = atomic_load_explicit(
        &adapter->stat_publications,
        memory_order_relaxed
    );
    statistics.queue_capacity_failures = atomic_load_explicit(
        &adapter->stat_queue_capacity_failures,
        memory_order_relaxed
    );
    statistics.in_flight = atomic_load_explicit(
        &adapter->in_flight,
        memory_order_relaxed
    );
    statistics.worker_count = adapter->worker_count;
    return statistics;
}

#endif
