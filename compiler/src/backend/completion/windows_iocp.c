#include "windows_iocp.h"

#if defined(_WIN32)

#include <errno.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

_Static_assert(
    offsetof(wf_windows_iocp_entry, overlapped) == 0,
    "OVERLAPPED must identify its preallocated operation entry"
);
_Static_assert(
    sizeof(wf_windows_file_result) <= WF_COMPLETION_RESULT_CAPACITY,
    "completion result cell must hold a Windows file result"
);

#define WF_WINDOWS_IOCP_PROGRESS_CLOSED (SIZE_MAX - 1u)
#define WF_WINDOWS_IOCP_PROGRESS_CLOSING SIZE_MAX

static void wf_windows_init_statistics(wf_windows_iocp_adapter *adapter) {
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_capacity_waits, 0);
    atomic_init(&adapter->stat_immediate_failures, 0);
    atomic_init(&adapter->stat_completions, 0);
    atomic_init(&adapter->stat_publication_failures, 0);
}

int wf_windows_iocp_init(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_windows_iocp_entry *entry_storage,
    size_t entry_capacity,
    DWORD concurrency
) {
    size_t index;
    HANDLE port;
    if (adapter == NULL || runtime == NULL || entry_storage == NULL
        || entry_capacity == 0) {
        return EINVAL;
    }
    memset(adapter, 0, sizeof(*adapter));
    atomic_init(
        &adapter->progress_gate,
        WF_WINDOWS_IOCP_PROGRESS_CLOSED
    );
    port = CreateIoCompletionPort(
        INVALID_HANDLE_VALUE,
        NULL,
        0,
        concurrency
    );
    if (port == NULL) {
        return (int)GetLastError();
    }
    adapter->runtime = runtime;
    adapter->port = port;
    adapter->wake_key = (ULONG_PTR)runtime;
    adapter->entries = entry_storage;
    adapter->entry_capacity = entry_capacity;
    atomic_init(&adapter->entry_cursor, 0);
    atomic_init(&adapter->in_flight, 0);
    wf_windows_init_statistics(adapter);
    for (index = 0; index < entry_capacity; ++index) {
        memset(&entry_storage[index].overlapped, 0, sizeof(OVERLAPPED));
        atomic_init(
            &entry_storage[index].state,
            WF_WINDOWS_IOCP_ENTRY_FREE
        );
    }
    adapter->initialized = 1;
    atomic_store_explicit(
        &adapter->progress_gate,
        0,
        memory_order_release
    );
    return 0;
}

int wf_windows_iocp_attach_completion(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_windows_iocp_wait_finished_hook wait_finished,
    wf_windows_iocp_close_hook close_bound_port
) {
    if (adapter == NULL || adapter->initialized == 0
        || adapter->runtime != runtime || wait_finished == NULL
        || close_bound_port == NULL) {
        return EINVAL;
    }
    if ((adapter->wait_finished != NULL
         && adapter->wait_finished != wait_finished)
        || (adapter->close_bound_port != NULL
            && adapter->close_bound_port != close_bound_port)) {
        return EBUSY;
    }
    adapter->wait_finished = wait_finished;
    adapter->close_bound_port = close_bound_port;
    return 0;
}

int wf_windows_iocp_associate_file(
    wf_windows_iocp_adapter *adapter,
    HANDLE handle,
    wf_windows_iocp_file *file
) {
    HANDLE associated;
    if (adapter == NULL || adapter->initialized == 0 || file == NULL
        || handle == NULL || handle == INVALID_HANDLE_VALUE) {
        return EINVAL;
    }
    associated = CreateIoCompletionPort(
        handle,
        adapter->port,
        (ULONG_PTR)adapter,
        0
    );
    if (associated != adapter->port) {
        return (int)GetLastError();
    }
    file->handle = handle;
    file->adapter = adapter;
    return 0;
}

static int wf_windows_request_valid(
    wf_windows_iocp_adapter *adapter,
    const wf_windows_file_request *request
) {
    if (request == NULL || request->file.adapter != adapter
        || request->file.handle == NULL
        || request->file.handle == INVALID_HANDLE_VALUE
        || request->lease.generation == 0
        || request->lease.handle != request->file.handle
        || request->lease.completion_owner != adapter
        || request->lease.descriptor_class
            != WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE
        || request->lease.mode != WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
        || request->count > MAXDWORD) {
        return 0;
    }
    switch (request->kind) {
    case WF_WINDOWS_FILE_READ_AT:
        return request->buffer.read_buffer != NULL || request->count == 0;
    case WF_WINDOWS_FILE_WRITE_AT:
        return request->buffer.write_buffer != NULL || request->count == 0;
    default:
        return 0;
    }
}

static wf_windows_iocp_entry *wf_windows_reserve_entry(
    wf_windows_iocp_adapter *adapter
) {
    size_t start = atomic_fetch_add_explicit(
        &adapter->entry_cursor,
        1,
        memory_order_relaxed
    );
    size_t offset;
    size_t index = start % adapter->entry_capacity;
    for (offset = 0; offset < adapter->entry_capacity; ++offset) {
        unsigned expected = WF_WINDOWS_IOCP_ENTRY_FREE;
        if (atomic_compare_exchange_strong_explicit(
                &adapter->entries[index].state,
                &expected,
                WF_WINDOWS_IOCP_ENTRY_RESERVED,
                memory_order_acquire,
                memory_order_relaxed
            )) {
            return &adapter->entries[index];
        }
        index = index + 1u == adapter->entry_capacity ? 0u : index + 1u;
    }
    return NULL;
}

static enum wf_windows_iocp_submit_result wf_windows_wait_capacity(
    wf_windows_iocp_adapter *adapter,
    wf_completion_token token
) {
    enum wf_completion_transition_result transition =
        wf_completion_begin_submit(adapter->runtime, token);
    if (transition == WF_COMPLETION_TRANSITION_STALE) {
        return WF_WINDOWS_IOCP_SUBMIT_STALE;
    }
    if (transition != WF_COMPLETION_TRANSITIONED
        || wf_completion_mark_wait_capacity(adapter->runtime, token)
            != WF_COMPLETION_TRANSITIONED) {
        return WF_WINDOWS_IOCP_SUBMIT_INVALID;
    }
    atomic_fetch_add_explicit(
        &adapter->stat_capacity_waits,
        1,
        memory_order_relaxed
    );
    return WF_WINDOWS_IOCP_WAIT_CAPACITY;
}

static int wf_windows_publish(
    wf_windows_iocp_adapter *adapter,
    wf_windows_iocp_entry *entry,
    uint32_t transferred,
    uint32_t error_code
) {
    wf_windows_file_result result;
    wf_completion_publication publication;
    enum wf_completion_publish_result published;
    wf_completion_token token = entry->token;
    enum wf_windows_file_operation_kind kind = entry->kind;
    wf_windows_descriptor_lease lease = entry->lease;
    /* Positioned file reads use the Whitefoot/POSIX-shaped result contract:
     * reaching the file boundary is a successful zero-byte read.  Windows
     * reports that boundary as ERROR_HANDLE_EOF for overlapped disk I/O. */
    if (kind == WF_WINDOWS_FILE_READ_AT
        && error_code == ERROR_HANDLE_EOF) {
        error_code = 0;
        transferred = 0;
    }
    memset(&result, 0, sizeof(result));
    result.kind = kind;
    result.value = error_code == 0 ? (int64_t)transferred : -1;
    result.error_code = error_code;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = error_code == 0 ? 1u : 2u;
    publication.result = &result;
    publication.result_size = sizeof(result);
    /* The target no longer touches the descriptor after GQCS. Relinquish the
     * generation lease and operation entry before terminal publication: a
     * fast source consume may immediately run the derived close and reclaim
     * both the descriptor and this fixed-capacity entry. */
    wf__windows_completion_descriptor_lease_release(&lease);
    memset(&entry->lease, 0, sizeof(entry->lease));
    atomic_store_explicit(
        &entry->state,
        WF_WINDOWS_IOCP_ENTRY_FREE,
        memory_order_release
    );
    atomic_fetch_sub_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    published = wf_completion_publish_terminal(
        adapter->runtime,
        token,
        &publication
    );
    if (published != WF_COMPLETION_PUBLISHED) {
        atomic_fetch_add_explicit(
            &adapter->stat_publication_failures,
            1,
            memory_order_relaxed
        );
    } else {
        wf_completion_operation_retired(0);
    }
    atomic_fetch_add_explicit(
        &adapter->stat_completions,
        1,
        memory_order_relaxed
    );
    wf_completion_notify_capacity(adapter->runtime);
    return published == WF_COMPLETION_PUBLISHED ? 0 : EPROTO;
}

enum wf_windows_iocp_submit_result wf_windows_iocp_submit(
    wf_windows_iocp_adapter *adapter,
    wf_completion_token token,
    const wf_windows_file_request *request
) {
    wf_windows_iocp_entry *entry;
    enum wf_completion_transition_result transition;
    BOOL started;
    DWORD error_code;
    ULARGE_INTEGER offset;

    if (adapter == NULL || adapter->initialized == 0) {
        return WF_WINDOWS_IOCP_UNAVAILABLE;
    }
    if (!wf_windows_request_valid(adapter, request)) {
        return WF_WINDOWS_IOCP_SUBMIT_INVALID;
    }
    entry = wf_windows_reserve_entry(adapter);
    if (entry == NULL) {
        return wf_windows_wait_capacity(adapter, token);
    }
    transition = wf_completion_begin_submit(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        atomic_store_explicit(
            &entry->state,
            WF_WINDOWS_IOCP_ENTRY_FREE,
            memory_order_release
        );
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_WINDOWS_IOCP_SUBMIT_STALE
            : WF_WINDOWS_IOCP_SUBMIT_INVALID;
    }

    memset(&entry->overlapped, 0, sizeof(entry->overlapped));
    offset.QuadPart = request->offset;
    entry->overlapped.Offset = offset.LowPart;
    entry->overlapped.OffsetHigh = offset.HighPart;
    entry->token = token;
    entry->kind = request->kind;
    transition = wf_completion_target_accepted(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        atomic_store_explicit(
            &entry->state,
            WF_WINDOWS_IOCP_ENTRY_FREE,
            memory_order_release
        );
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_WINDOWS_IOCP_SUBMIT_STALE
            : WF_WINDOWS_IOCP_SUBMIT_INVALID;
    }

    entry->lease = request->lease;
    wf_completion_operation_accepted();

    atomic_store_explicit(
        &entry->state,
        WF_WINDOWS_IOCP_ENTRY_IN_FLIGHT,
        memory_order_release
    );
    atomic_fetch_add_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );
    if (request->kind == WF_WINDOWS_FILE_READ_AT) {
        started = ReadFile(
            request->file.handle,
            request->buffer.read_buffer,
            (DWORD)request->count,
            NULL,
            &entry->overlapped
        );
    } else {
        started = WriteFile(
            request->file.handle,
            request->buffer.write_buffer,
            (DWORD)request->count,
            NULL,
            &entry->overlapped
        );
    }
    if (started != FALSE) {
        /* An overlapped handle associated with IOCP still queues one packet
         * for synchronous success.  Adapter-owned handles must never enable
         * FILE_SKIP_COMPLETION_PORT_ON_SUCCESS. */
        return WF_WINDOWS_IOCP_TARGET_OWNS;
    }
    error_code = GetLastError();
    if (error_code == ERROR_IO_PENDING) {
        return WF_WINDOWS_IOCP_TARGET_OWNS;
    }

    /* No operation was pending and Windows promises no completion packet for
     * this failed initiation.  The already target-owned entry therefore owns
     * the unique terminal publication directly. */
    atomic_fetch_add_explicit(
        &adapter->stat_immediate_failures,
        1,
        memory_order_relaxed
    );
    (void)wf_windows_publish(adapter, entry, 0, error_code);
    return WF_WINDOWS_IOCP_TARGET_OWNS;
}

static wf_windows_iocp_entry *wf_windows_entry_from_overlapped(
    wf_windows_iocp_adapter *adapter,
    OVERLAPPED *overlapped
) {
    uintptr_t first = (uintptr_t)&adapter->entries[0];
    uintptr_t end = (uintptr_t)&adapter->entries[adapter->entry_capacity];
    uintptr_t address = (uintptr_t)overlapped;
    if (address < first || address >= end
        || (address - first) % sizeof(wf_windows_iocp_entry) != 0) {
        return NULL;
    }
    return (wf_windows_iocp_entry *)overlapped;
}

int wf_windows_iocp_progress(
    wf_windows_iocp_adapter *adapter,
    size_t budget,
    DWORD timeout_milliseconds,
    size_t *published
) {
    size_t processed = 0;
    size_t total = 0;
    size_t gate;
    int first_error = 0;
    int enter_error = 0;
    HANDLE port;
    ULONG_PTR wake_key;
    wf_completion_runtime *runtime;
    wf_windows_iocp_wait_finished_hook wait_finished;
    if (published != NULL) {
        *published = 0;
    }
    if (adapter == NULL || published == NULL || budget == 0) {
        return EINVAL;
    }
    gate = atomic_load_explicit(
        &adapter->progress_gate,
        memory_order_acquire
    );
    for (;;) {
        if (gate == WF_WINDOWS_IOCP_PROGRESS_CLOSED) {
            return EINVAL;
        }
        if (gate == WF_WINDOWS_IOCP_PROGRESS_CLOSING) {
            /* A bound destroy which races an announced waiter must fail its
             * core close check and reopen this gate. Wait for that decision so
             * the caller does not have to cancel an untouched announcement. */
            (void)SwitchToThread();
            gate = atomic_load_explicit(
                &adapter->progress_gate,
                memory_order_acquire
            );
            continue;
        }
        if (gate + 1u >= WF_WINDOWS_IOCP_PROGRESS_CLOSED) {
            return EOVERFLOW;
        }
        if (atomic_compare_exchange_weak_explicit(
                &adapter->progress_gate,
                &gate,
                gate + 1u,
                memory_order_acq_rel,
                memory_order_acquire
            )) {
            break;
        }
    }
    if (adapter->initialized == 0) {
        enter_error = EINVAL;
    }
    if (enter_error != 0) {
        (void)atomic_fetch_sub_explicit(
            &adapter->progress_gate,
            1,
            memory_order_release
        );
        return enter_error;
    }
    port = adapter->port;
    wake_key = adapter->wake_key;
    runtime = adapter->runtime;
    wait_finished = adapter->wait_finished;
    while (processed < budget) {
        DWORD transferred = 0;
        ULONG_PTR completion_key = 0;
        OVERLAPPED *overlapped = NULL;
        BOOL succeeded = GetQueuedCompletionStatus(
            port,
            &transferred,
            &completion_key,
            &overlapped,
            processed == 0 ? timeout_milliseconds : 0
        );
        DWORD error_code = succeeded != FALSE ? 0 : GetLastError();
        wf_windows_iocp_entry *entry;
        int error;
        unsigned legal_wake = overlapped == NULL && succeeded != FALSE
            && completion_key == wake_key;
        if (wait_finished != NULL) {
            error = wait_finished(
                runtime,
                port,
                wake_key,
                legal_wake
            );
            if (first_error == 0 && error != 0) {
                first_error = error;
            }
        }
        if (overlapped == NULL) {
            if (error_code == WAIT_TIMEOUT) {
                break;
            }
            if (succeeded == FALSE) {
                if (first_error == 0) {
                    first_error = (int)error_code;
                }
                break;
            }
            if (completion_key != wake_key) {
                if (first_error == 0) {
                    first_error = EPROTO;
                }
                break;
            }
            /* Scheduler wake packets share the port with target completions,
             * but they carry no operation and publish no terminal. The bound
             * core has already withdrawn this waiter and consumed the exact
             * packet in one lock critical section. */
            processed += 1;
            if (wait_finished != NULL) {
                break;
            }
            continue;
        }
        if (completion_key != (ULONG_PTR)adapter) {
            if (first_error == 0) {
                first_error = EPROTO;
            }
            break;
        }
        entry = wf_windows_entry_from_overlapped(adapter, overlapped);
        if (entry == NULL
            || atomic_load_explicit(&entry->state, memory_order_acquire)
                != WF_WINDOWS_IOCP_ENTRY_IN_FLIGHT) {
            first_error = EPROTO;
            break;
        }
        processed += 1;
        error = wf_windows_publish(
            adapter,
            entry,
            transferred,
            error_code
        );
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
        if (error == 0) {
            total += 1;
        }
        if (wait_finished != NULL) {
            break;
        }
    }
    *published = total;
    (void)atomic_fetch_sub_explicit(
        &adapter->progress_gate,
        1,
        memory_order_release
    );
    return first_error;
}

HANDLE wf_windows_iocp_port(const wf_windows_iocp_adapter *adapter) {
    return adapter == NULL ? NULL : adapter->port;
}

ULONG_PTR wf_windows_iocp_wake_key(const wf_windows_iocp_adapter *adapter) {
    return adapter == NULL ? 0 : adapter->wake_key;
}

size_t wf_windows_iocp_in_flight(const wf_windows_iocp_adapter *adapter) {
    if (adapter == NULL) {
        return 0;
    }
    return atomic_load_explicit(&adapter->in_flight, memory_order_acquire);
}

size_t wf_windows_iocp_active_progress(
    const wf_windows_iocp_adapter *adapter
) {
    size_t gate;
    if (adapter == NULL) {
        return 0;
    }
    gate = atomic_load_explicit(
        &adapter->progress_gate,
        memory_order_acquire
    );
    return gate >= WF_WINDOWS_IOCP_PROGRESS_CLOSED ? 0 : gate;
}

int wf_windows_iocp_destroy(wf_windows_iocp_adapter *adapter) {
    int error;
    size_t expected = 0;
    if (adapter == NULL) {
        return EINVAL;
    }
    if (!atomic_compare_exchange_strong_explicit(
            &adapter->progress_gate,
            &expected,
            WF_WINDOWS_IOCP_PROGRESS_CLOSING,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
        return expected == WF_WINDOWS_IOCP_PROGRESS_CLOSED
            ? EINVAL
            : EBUSY;
    }
    if (adapter->initialized == 0) {
        atomic_store_explicit(
            &adapter->progress_gate,
            WF_WINDOWS_IOCP_PROGRESS_CLOSED,
            memory_order_release
        );
        return EINVAL;
    }
    if (wf_windows_iocp_in_flight(adapter) != 0) {
        atomic_store_explicit(
            &adapter->progress_gate,
            0,
            memory_order_release
        );
        return EBUSY;
    }
    if (adapter->close_bound_port != NULL) {
        error = adapter->close_bound_port(
            adapter->runtime,
            adapter->port,
            adapter->wake_key
        );
        if (error != 0) {
            atomic_store_explicit(
                &adapter->progress_gate,
                0,
                memory_order_release
            );
            return error;
        }
    } else if (CloseHandle(adapter->port) == FALSE) {
        error = (int)GetLastError();
        atomic_store_explicit(
            &adapter->progress_gate,
            0,
            memory_order_release
        );
        return error;
    }
    adapter->wait_finished = NULL;
    adapter->close_bound_port = NULL;
    adapter->port = NULL;
    adapter->initialized = 0;
    atomic_store_explicit(
        &adapter->progress_gate,
        WF_WINDOWS_IOCP_PROGRESS_CLOSED,
        memory_order_release
    );
    return 0;
}

wf_windows_iocp_statistics wf_windows_iocp_statistics_snapshot(
    const wf_windows_iocp_adapter *adapter
) {
    wf_windows_iocp_statistics statistics = {0};
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
    statistics.immediate_failures = atomic_load_explicit(
        &adapter->stat_immediate_failures,
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
    return statistics;
}

#else

typedef int wf_windows_iocp_translation_unit_is_target_guarded;

#endif
