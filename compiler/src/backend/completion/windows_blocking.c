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
    wf_windows_blocking_entry *entry,
    unsigned awarded_resource,
    int on_an_award
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
        unsigned took_resources = 0;
        unsigned returned_resources = 0;
        unsigned refused_resource = WF_WINDOWS_OPEN_REFUSED_NONE;
        result.value = wf__windows_completion_file_open_at_worker(
            entry->request.lease.handle,
            (const char *)(const void *)entry->path,
            entry->request.operation.open_at.flags,
            entry->request.operation.open_at.mode,
            entry->request.operation.open_at.has_mode,
            entry->request.operation.open_at.expected_kind,
            entry->request.operation.open_at.descriptor_class,
            &error_code,
            &open_outcome,
            &took_resources,
            &returned_resources,
            &refused_resource,
            awarded_resource,
            on_an_award
        );
        result.error_code = (uint32_t)error_code;
        result.open_outcome = open_outcome;
        if ((took_resources & WF_WINDOWS_OPEN_RESOURCE_NATIVE_HANDLE) != 0) {
            result.runtime_flags |= WF_WINDOWS_FILE_TOOK_NATIVE_HANDLE;
        }
        if ((took_resources & WF_WINDOWS_OPEN_RESOURCE_CRT_DESCRIPTOR) != 0) {
            result.runtime_flags |= WF_WINDOWS_FILE_TOOK_CRT_DESCRIPTOR;
        }
        if ((returned_resources & WF_WINDOWS_OPEN_RESOURCE_NATIVE_HANDLE)
            != 0) {
            result.runtime_flags |= WF_WINDOWS_FILE_RETURNED_NATIVE_HANDLE;
        }
        if ((returned_resources & WF_WINDOWS_OPEN_RESOURCE_CRT_DESCRIPTOR)
            != 0) {
            result.runtime_flags |= WF_WINDOWS_FILE_RETURNED_CRT_DESCRIPTOR;
        }
        if (refused_resource == WF_WINDOWS_OPEN_REFUSED_NATIVE_HANDLE) {
            result.runtime_flags |= WF_WINDOWS_FILE_REFUSED_NATIVE_HANDLE;
        } else if (refused_resource
                   == WF_WINDOWS_OPEN_REFUSED_CRT_DESCRIPTOR) {
            result.runtime_flags |= WF_WINDOWS_FILE_REFUSED_CRT_DESCRIPTOR;
        } else if (refused_resource
                   == WF_WINDOWS_OPEN_REFUSED_GENERAL_RESOURCE) {
            result.runtime_flags |= WF_WINDOWS_FILE_REFUSED_GENERAL_RESOURCE;
        }
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

static void wf_windows_blocking_run_index(
    wf_windows_blocking_adapter *adapter,
    size_t index,
    int may_run_owed_work
);

static void wf_windows_blocking_open_gate_enter(
    wf_windows_blocking_adapter *adapter,
    const wf_windows_blocking_entry *entry
) {
    if (adapter == NULL || entry == NULL
        || entry->request.kind != WF_WINDOWS_FILE_OPEN_AT) {
        abort();
    }
    wf__windows_open_order_enter(entry->open_ticket);

    /* Later open tickets are deliberately absent from the retirement ledger.
     * Otherwise the current ticket could wait forever for operations which
     * are forbidden to acquire a host resource until it advances the gate. */
    wf_completion_operation_accepted();
}

static void wf_windows_blocking_open_gate_leave(
    wf_windows_blocking_adapter *adapter,
    const wf_windows_blocking_entry *entry
) {
    if (adapter == NULL || entry == NULL
        || entry->request.kind != WF_WINDOWS_FILE_OPEN_AT) {
        abort();
    }
    wf__windows_open_order_leave(entry->open_ticket);
}

static wf_windows_file_result wf_windows_blocking_execute_host_once(
    wf_windows_blocking_entry *entry,
    unsigned awarded_resource,
    int on_an_award
) {
    return wf_windows_blocking_execute_once(
        entry,
        awarded_resource,
        on_an_award
    );
}

static size_t wf_windows_blocking_owed(void *context) {
    wf_windows_blocking_adapter *adapter =
        (wf_windows_blocking_adapter *)context;
    return adapter == NULL
        ? 0
        : atomic_load_explicit(
              &adapter->retirement_queued,
              memory_order_seq_cst
          );
}

static int wf_windows_blocking_take_owed(
    wf_windows_blocking_adapter *adapter,
    wf_retirement_waiter *waiter
) {
    DWORD bytes = 0;
    DWORD error;
    ULONG_PTR key = 0;
    OVERLAPPED *overlapped = NULL;
    BOOL dequeued = GetQueuedCompletionStatus(
        adapter->port,
        &bytes,
        &key,
        &overlapped,
        0
    );
    if (dequeued == FALSE) {
        error = GetLastError();
        if (error == WAIT_TIMEOUT && overlapped == NULL) {
            return 0;
        }
        abort();
    }
    /* Stop packets are posted only after all accepted work has retired, so a
     * live exhaustion waiter can never consume one. */
    if (key == 0 || key > WF_WINDOWS_BLOCKING_CAPACITY
        || bytes != 0 || overlapped != NULL) {
        abort();
    }
    {
        wf_windows_blocking_entry *entry = &adapter->entries[key - 1u];
        if (atomic_load_explicit(&entry->state, memory_order_acquire)
                != WF_WINDOWS_BLOCKING_ENTRY_QUEUED) {
            abort();
        }
        if (entry->request.kind == WF_WINDOWS_FILE_OPEN_AT) {
            /* The caller already owns the serving open ticket. Running a
             * later open as owed work would deadlock on that same ticket and
             * would let a future source operation participate in the current
             * quiescence decision. Rotate it behind genuinely owed work. */
            if (PostQueuedCompletionStatus(
                    adapter->port,
                    0,
                    key,
                    NULL
                ) == FALSE) {
                abort();
            }
            return 0;
        }
    }
    if (atomic_fetch_sub_explicit(
            &adapter->queued,
            1,
            memory_order_seq_cst
        ) == 0) {
        abort();
    }
    if (atomic_fetch_sub_explicit(
            &adapter->retirement_queued,
            1,
            memory_order_seq_cst
        ) == 0) {
        abort();
    }
    wf_completion_retirement_defer_begin(waiter);
    wf_windows_blocking_run_index(adapter, (size_t)key - 1u, 0);
    wf_completion_retirement_defer_end(waiter);
    return 1;
}

static wf_windows_file_result wf_windows_blocking_retire_and_retry(
    wf_windows_blocking_adapter *adapter,
    wf_windows_blocking_entry *entry,
    int may_run_owed_work,
    uint64_t seen,
    unsigned resource
) {
    wf_retirement_waiter waiter;
    unsigned awarded_resource = WF_WINDOWS_OPEN_REFUSED_NONE;
    if (resource == WF_RETIREMENT_NATIVE_HANDLE) {
        awarded_resource = WF_WINDOWS_OPEN_REFUSED_NATIVE_HANDLE;
    } else if (resource == WF_RETIREMENT_CRT_DESCRIPTOR) {
        awarded_resource = WF_WINDOWS_OPEN_REFUSED_CRT_DESCRIPTOR;
    }
    wf_completion_retirement_wait_begin_resource(
        &waiter,
        seen,
        wf_windows_blocking_owed,
        adapter,
        may_run_owed_work,
        resource
    );
    for (;;) {
        if (may_run_owed_work != 0) {
            while (wf_windows_blocking_take_owed(adapter, &waiter)) {
            }
        }
        if (wf_completion_retirement_state(&waiter)
            != WF_RETIREMENT_AWAITED) {
            break;
        }
        /* Another worker can dequeue before it lowers `queued`. In that
         * bounded window the no-packet result is not proof that work vanished;
         * yield and re-read rather than sleeping through owed work. */
        if (may_run_owed_work != 0
            && wf_windows_blocking_owed(adapter) != 0) {
            (void)SwitchToThread();
            continue;
        }
        wf_completion_retirement_sleep(&waiter);
    }
    wf_completion_retirement_wait_end(&waiter);
    atomic_fetch_add_explicit(
        &adapter->stat_exhaustion_retries,
        1,
        memory_order_relaxed
    );
    {
        wf_windows_file_result result = wf_windows_blocking_execute_host_once(
            entry,
            awarded_resource,
            waiter.awarded
        );
        return result;
    }
}

static wf_windows_file_result wf_windows_blocking_execute(
    wf_windows_blocking_adapter *adapter,
    wf_windows_blocking_entry *entry,
    int may_run_owed_work
) {
    uint64_t seen = wf_completion_resource_returns(
        WF_RETIREMENT_OPEN_QUIESCENCE
    );
    wf_windows_file_result result = wf_windows_blocking_execute_host_once(
        entry,
        WF_WINDOWS_OPEN_REFUSED_NONE,
        0
    );
    if (result.kind != WF_WINDOWS_FILE_OPEN_AT || result.value >= 0) {
        return result;
    }
    if ((result.runtime_flags
         & (WF_WINDOWS_FILE_REFUSED_NATIVE_HANDLE
            | WF_WINDOWS_FILE_REFUSED_CRT_DESCRIPTOR
            | WF_WINDOWS_FILE_REFUSED_GENERAL_RESOURCE)) != 0) {
        return wf_windows_blocking_retire_and_retry(
            adapter,
            entry,
            may_run_owed_work,
            seen,
            WF_RETIREMENT_OPEN_QUIESCENCE
        );
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
    /* Open's provisional returns were announced at their exact host release
     * sites. Terminal retirement ends this accepted operation only. */
    wf_completion_operation_retired_resources(0);
    atomic_fetch_add_explicit(
        &adapter->stat_publications,
        1,
        memory_order_relaxed
    );
    atomic_fetch_sub_explicit(&adapter->in_flight, 1, memory_order_relaxed);
}

static void wf_windows_blocking_run_index(
    wf_windows_blocking_adapter *adapter,
    size_t index,
    int may_run_owed_work
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
    if (entry->request.kind == WF_WINDOWS_FILE_OPEN_AT) {
        wf_windows_blocking_open_gate_enter(adapter, entry);
    }
    result = wf_windows_blocking_execute(
        adapter,
        entry,
        may_run_owed_work
    );
    if (entry->request.kind == WF_WINDOWS_FILE_OPEN_AT) {
        wf_windows_blocking_open_gate_leave(adapter, entry);
    }
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
        if (entry->request.kind == WF_WINDOWS_FILE_OPEN_AT
            && !wf__windows_open_order_is_serving(entry->open_ticket)) {
            /* A submitter can reserve the preceding ticket and be preempted
             * before it posts that packet. Never let later tickets occupy the
             * finite worker pool while waiting for it: rotate them until the
             * serving packet becomes runnable. */
            if (PostQueuedCompletionStatus(
                    adapter->port,
                    0,
                    key,
                    NULL
                ) == FALSE) {
                abort();
            }
            (void)SwitchToThread();
            continue;
        }
        if (atomic_fetch_sub_explicit(
                &adapter->queued,
                1,
                memory_order_seq_cst
            ) == 0) {
            abort();
        }
        if (entry->request.kind != WF_WINDOWS_FILE_OPEN_AT
            && atomic_fetch_sub_explicit(
                   &adapter->retirement_queued,
                   1,
                   memory_order_seq_cst
               ) == 0) {
            abort();
        }
        wf_windows_blocking_run_index(
            adapter,
            (size_t)key - 1u,
            1
        );
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
    atomic_init(&adapter->retirement_queued, 0);
    atomic_init(&adapter->in_flight, 0);
    atomic_init(&adapter->stopping, 0);
    atomic_init(&adapter->initialized, 0);
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_executions, 0);
    atomic_init(&adapter->stat_publications, 0);
    atomic_init(&adapter->stat_queue_capacity_failures, 0);
    atomic_init(&adapter->stat_exhaustion_retries, 0);
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
    if (request->kind == WF_WINDOWS_FILE_OPEN_AT) {
        entry->open_ticket = wf__windows_open_order_reserve();
    } else {
        /* Publish owed work before announcing acceptance. A sole worker may
         * already be waiting for quiescence; the acceptance wake must let it
         * observe and execute this job rather than go back to sleep. */
        atomic_fetch_add_explicit(
            &adapter->retirement_queued,
            1,
            memory_order_seq_cst
        );
        wf_completion_operation_accepted();
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
     * publication while the publishing worker is still retiring its ledger
     * and adapter counters. Stop packets must not enter the work port during
     * that tail, because an exhaustion helper is also allowed to dequeue. */
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
    if (atomic_load_explicit(
            &adapter->retirement_queued,
            memory_order_acquire
        ) != 0) {
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
        || atomic_load_explicit(
               &adapter->retirement_queued,
               memory_order_acquire
           ) != 0
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
    statistics.exhaustion_retries = atomic_load_explicit(
        &adapter->stat_exhaustion_retries,
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
