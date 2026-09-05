#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "windows_iocp.h"

#if defined(_WIN32)

#include <errno.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(WF_WINDOWS_IOCP_READ_CALL)
extern BOOL WINAPI WF_WINDOWS_IOCP_READ_CALL(
    HANDLE handle,
    LPVOID buffer,
    DWORD count,
    LPDWORD transferred,
    LPOVERLAPPED overlapped
);
#else
#define WF_WINDOWS_IOCP_READ_CALL ReadFile
#endif

#if defined(WF_WINDOWS_IOCP_RESULT_CALL)
extern BOOL WINAPI WF_WINDOWS_IOCP_RESULT_CALL(
    HANDLE handle,
    LPOVERLAPPED overlapped,
    LPDWORD transferred,
    BOOL wait
);
#else
#define WF_WINDOWS_IOCP_RESULT_CALL GetOverlappedResult
#endif

#if defined(WF_WINDOWS_IOCP_NOTIFICATION_CALL)
extern BOOL WINAPI WF_WINDOWS_IOCP_NOTIFICATION_CALL(
    HANDLE handle,
    UCHAR flags
);
#else
#define WF_WINDOWS_IOCP_NOTIFICATION_CALL SetFileCompletionNotificationModes
#endif

/* The ring's own state inside the record.
 *
 * The `OVERLAPPED` is first, so the completion packet's `OVERLAPPED` address
 * minus the record's ring offset is the record, and there is no side table and
 * no allocation between a submission and its completion.  The handle beside it
 * is what the synchronous-success path asks `GetOverlappedResult` about; the
 * record is the only place it can live, because there is no entry pool left to
 * keep it in (design section 7). */
typedef struct wf_windows_iocp_state {
    OVERLAPPED overlapped;
    HANDLE handle;
} wf_windows_iocp_state;

_Static_assert(
    offsetof(wf_windows_iocp_state, overlapped) == 0,
    "a completion packet's OVERLAPPED must identify its record by subtraction"
);
_Static_assert(
    sizeof(wf_windows_iocp_state) <= sizeof(wf_completion_ring_state),
    "the IOCP ring state must fit the record's ring block"
);
_Static_assert(
    _Alignof(wf_windows_iocp_state) <= _Alignof(wf_completion_ring_state),
    "the IOCP ring state must not out-align the record's ring block"
);

/* This leaf's one fail-stop.
 *
 * Each `abort` below is a trusted-computing-base defect, not an operation
 * outcome, and a fail-stop that writes nothing cannot be diagnosed from a
 * crash log: the release UCRT ends such a process through the fast-fail path,
 * which a shell reports as a bare status with no message at all. This writes
 * one line and then aborts exactly where the bare call did, and changes no
 * control flow. */
static _Noreturn void wf_windows_iocp_fail(const char *reason) {
    (void)fprintf(stderr, "whitefoot completion port: %s\n", reason);
    (void)fflush(stderr);
    abort();
}

static wf_windows_iocp_state *wf_windows_state_of(
    wf_completion_record *record
) {
    return (wf_windows_iocp_state *)(void *)&record->ring;
}

static wf_completion_record *wf_windows_record_of(OVERLAPPED *overlapped) {
    unsigned char *bytes = (unsigned char *)(void *)overlapped;
    return (wf_completion_record *)(void *)(
        bytes - offsetof(wf_completion_record, ring)
    );
}

static void wf_windows_record_progress_error(
    wf_windows_iocp_adapter *adapter,
    int error
) {
    int expected = 0;
    (void)atomic_compare_exchange_strong_explicit(
        &adapter->progress_error,
        &expected,
        error,
        memory_order_acq_rel,
        memory_order_acquire
    );
}

int wf_windows_iocp_progress_error(const wf_windows_iocp_adapter *adapter) {
    if (adapter == NULL) {
        return EINVAL;
    }
    return atomic_load_explicit(&adapter->progress_error, memory_order_acquire);
}

int wf_windows_iocp_init(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    DWORD concurrency
) {
    HANDLE port;
    if (adapter == NULL || runtime == NULL) {
        return EINVAL;
    }
    memset(adapter, 0, sizeof(*adapter));
    port = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, concurrency);
    if (port == NULL) {
        return (int)GetLastError();
    }
    adapter->runtime = runtime;
    adapter->port = port;
    /* Two keys, and they must differ: a packet with no `OVERLAPPED` is a wake
     * and a packet with one is a completion, and the key is what says which
     * queue a packet came from when a protocol failure has to be named. */
    adapter->wake_key = (ULONG_PTR)(void *)runtime;
    adapter->file_key = (ULONG_PTR)(void *)adapter;
    if (adapter->wake_key == adapter->file_key) {
        (void)CloseHandle(port);
        return EINVAL;
    }
    atomic_init(&adapter->in_flight, 0);
    atomic_init(&adapter->progress_error, 0);
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_inline_completions, 0);
    atomic_init(&adapter->stat_dequeued_completions, 0);
    atomic_init(&adapter->stat_completions, 0);
    atomic_init(&adapter->stat_kernel_waits, 0);
    atomic_init(&adapter->stat_host_wake_posts, 0);
    adapter->initialized = 1;
    return 0;
}

int wf_windows_iocp_associate(
    wf_windows_iocp_adapter *adapter,
    HANDLE handle
) {
    HANDLE associated;
    if (adapter == NULL || adapter->initialized == 0 || handle == NULL
        || handle == INVALID_HANDLE_VALUE) {
        return EINVAL;
    }
    /* A request the kernel answers synchronously is already complete, so the
     * submitting thread publishes it and the port is spared a round trip that
     * carries no news.  A pending request still has exactly one packet.
     *
     * This is asked *before* the port takes the handle, and the order is the
     * whole of this function's failure story.  A handle cannot be taken off a
     * completion port: there is no call for it, and the association lasts as
     * long as the handle does.  Asking second would mean that a host which
     * refuses these modes -- a file system that does not carry them is the
     * documented case -- leaves the handle bound to this port with the skip it
     * needs unset, and every later transfer on it belongs to no engine: the
     * ring is refused for it here, so it goes to the bounded adapter, whose
     * overlapped read would then post its completion to this port instead of
     * signalling its own event, wedging that read and handing the reaper an
     * `OVERLAPPED` that is not a record.  Asking first leaves a refused handle
     * exactly as it was found, which is what makes `file_windows.c`'s
     * "the handle it reads is one the port never took" true rather than
     * hopeful. */
    if (WF_WINDOWS_IOCP_NOTIFICATION_CALL(
            handle,
            (UCHAR)(FILE_SKIP_COMPLETION_PORT_ON_SUCCESS
                | FILE_SKIP_SET_EVENT_ON_HANDLE)
        ) == FALSE) {
        return (int)GetLastError();
    }
    associated = CreateIoCompletionPort(
        handle,
        adapter->port,
        adapter->file_key,
        0
    );
    if (associated != adapter->port) {
        return (int)GetLastError();
    }
    return 0;
}

/* The one kind this ring carries.
 *
 * A positioned read is the whole of it, and that is not a gap: `write_once` is
 * an unpositioned stream write whose current-position contract the port's
 * positioned write does not have, an open is a namespace operation with no
 * overlapped form here, and a close and a status are in-memory answers.  Every
 * one of those goes to the shared file adapter, exactly as the POSIX bridge
 * routes what its ring refuses. */
int wf_windows_iocp_carries(const wf_completion_record *record) {
    if (record == NULL) {
        return 0;
    }
    if (record->request.kind != WF_FILE_PREAD) {
        return 0;
    }
    return record->request.operation.pread.descriptor >= 0
        && record->request.operation.pread.count != 0
        && record->request.operation.pread.count <= (size_t)MAXDWORD
        && record->request.operation.pread.offset >= 0
        && record->request.operation.pread.buffer != NULL;
}

/* Stores one record's terminal result and completes it.
 *
 * Every route out of a ring-owned operation ends here, so an operation's
 * completion is counted in exactly one place, and the store of DONE inside
 * `wf_completion_record_complete` is this adapter's last touch of a record
 * that is a block of the joiner's frame. */
static void wf_windows_complete_record(
    wf_windows_iocp_adapter *adapter,
    wf_completion_record *record,
    DWORD transferred,
    DWORD error_code
) {
    wf_file_result_head result;
    /* A positioned read uses the Whitefoot/POSIX-shaped result contract:
     * reaching the file boundary is a successful zero-byte read.  Windows
     * reports that boundary as ERROR_HANDLE_EOF for overlapped disk I/O. */
    if (error_code == ERROR_HANDLE_EOF) {
        error_code = 0;
        transferred = 0;
    }
    memset(&result, 0, sizeof(result));
    result.kind = record->request.kind;
    if (error_code != 0) {
        result.value = -1;
        result.error_code = (int)error_code;
    } else {
        result.value = (int64_t)transferred;
    }
    atomic_fetch_sub_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(
        &adapter->stat_completions,
        1,
        memory_order_relaxed
    );
    record->result = result;
    wf_completion_record_complete(record);
}

enum wf_windows_iocp_submit_result wf_windows_iocp_submit(
    wf_windows_iocp_adapter *adapter,
    wf_completion_record *record,
    HANDLE handle
) {
    wf_windows_iocp_state *state;
    ULARGE_INTEGER offset;
    DWORD transferred = 0;
    DWORD error_code;
    BOOL started;

    if (adapter == NULL || adapter->initialized == 0) {
        return WF_WINDOWS_IOCP_UNAVAILABLE;
    }
    if (!wf_windows_iocp_carries(record) || handle == NULL
        || handle == INVALID_HANDLE_VALUE) {
        return WF_WINDOWS_IOCP_SUBMIT_INVALID;
    }

    record->route = WF_COMPLETION_ROUTE_WINDOWS_IOCP;
    record->opened_descriptor = -1;
    record->open_outcome = WF_FILE_OPEN_SUCCEEDED;
    record->open_error = 0;
    state = wf_windows_state_of(record);
    memset(state, 0, sizeof(*state));
    offset.QuadPart = (ULONGLONG)record->request.operation.pread.offset;
    state->overlapped.Offset = offset.LowPart;
    state->overlapped.OffsetHigh = offset.HighPart;
    state->handle = handle;

    atomic_fetch_add_explicit(&adapter->in_flight, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );
    /* Every word of the record the reaper will read is written above and by
     * the submitting bridge before this call; this release is what orders
     * them before the reaper's acquire (`contract.h`, `issued`). */
    atomic_store_explicit(&record->issued, 1u, memory_order_release);
    started = WF_WINDOWS_IOCP_READ_CALL(
        handle,
        record->request.operation.pread.buffer,
        (DWORD)record->request.operation.pread.count,
        NULL,
        &state->overlapped
    );
    if (started != FALSE) {
        /* The association asked for no packet on a synchronous success, so
         * this operation is already complete and nobody else will publish it.
         * Its exact byte count is queried without waiting. */
        if (WF_WINDOWS_IOCP_RESULT_CALL(
                handle,
                &state->overlapped,
                &transferred,
                FALSE
            ) == FALSE) {
            error_code = GetLastError();
            /* ReadFile returned TRUE, which guarantees the operation is
             * complete.  Windows reporting otherwise would leave the record's
             * OVERLAPPED and the caller's buffer live with no packet to come,
             * so this is an invariant failure rather than an I/O result. */
            if (error_code == ERROR_IO_INCOMPLETE) {
                wf_windows_iocp_fail(
                    "an inline-completed transfer reported that it is still incomplete"
                );
            }
            if (error_code == ERROR_SUCCESS) {
                error_code = ERROR_GEN_FAILURE;
            }
            transferred = 0;
        } else {
            error_code = 0;
        }
        atomic_fetch_add_explicit(
            &adapter->stat_inline_completions,
            1,
            memory_order_relaxed
        );
        wf_windows_complete_record(adapter, record, transferred, error_code);
        return WF_WINDOWS_IOCP_TARGET_OWNS;
    }
    error_code = GetLastError();
    if (error_code == ERROR_IO_PENDING) {
        return WF_WINDOWS_IOCP_TARGET_OWNS;
    }
    /* No operation is pending and Windows promises no packet for a failed
     * initiation, so this thread owns the unique terminal publication. */
    wf_windows_complete_record(adapter, record, 0, error_code);
    return WF_WINDOWS_IOCP_TARGET_OWNS;
}

/* Publishes one dequeued completion packet.  Returns zero, or a protocol
 * failure for a packet naming a record that was never issued. */
static int wf_windows_publish_packet(
    wf_windows_iocp_adapter *adapter,
    OVERLAPPED *overlapped,
    DWORD transferred,
    DWORD error_code
) {
    wf_completion_record *record = wf_windows_record_of(overlapped);
    if (record == NULL
        || atomic_load_explicit(&record->issued, memory_order_acquire) == 0
        || record->route != WF_COMPLETION_ROUTE_WINDOWS_IOCP) {
        return EPROTO;
    }
    atomic_fetch_add_explicit(
        &adapter->stat_dequeued_completions,
        1,
        memory_order_relaxed
    );
    wf_windows_complete_record(adapter, record, transferred, error_code);
    return 0;
}

/* Puts one wake packet back for the sleeper it belongs to.
 *
 * A wake is a consumable packet on this port, not a level fact as the Linux
 * eventfd is, so a reaping thread that swallowed one would leave an announced
 * sleeper asleep. It is re-posted while a sleeper is announced and dropped
 * when none is: the epoch, and never the packet, is the durable wake fact. */
static void wf_windows_return_wake(wf_windows_iocp_adapter *adapter) {
    if (wf_completion_parked_scheduler_count(adapter->runtime) == 0u) {
        return;
    }
    if (PostQueuedCompletionStatus(adapter->port, 0, adapter->wake_key, NULL)
        == FALSE) {
        wf_windows_record_progress_error(adapter, (int)GetLastError());
    }
}

int wf_windows_iocp_progress(
    wf_windows_iocp_adapter *adapter,
    size_t budget,
    size_t *published
) {
    size_t total = 0;
    size_t processed = 0;
    int first_error;

    if (published != NULL) {
        *published = 0;
    }
    if (adapter == NULL || published == NULL || budget == 0) {
        return EINVAL;
    }
    if (adapter->initialized == 0) {
        return EINVAL;
    }
    first_error = wf_windows_iocp_progress_error(adapter);
    if (first_error != 0) {
        return first_error;
    }
    while (processed < budget) {
        DWORD transferred = 0;
        ULONG_PTR key = 0;
        OVERLAPPED *overlapped = NULL;
        BOOL succeeded = GetQueuedCompletionStatus(
            adapter->port,
            &transferred,
            &key,
            &overlapped,
            0
        );
        DWORD error_code = succeeded != FALSE ? 0u : GetLastError();
        int error;
        if (overlapped == NULL) {
            if (succeeded == FALSE && error_code == WAIT_TIMEOUT) {
                break;
            }
            if (succeeded == FALSE) {
                if (first_error == 0) {
                    first_error = (int)error_code;
                }
                break;
            }
            if (key != adapter->wake_key) {
                if (first_error == 0) {
                    first_error = EPROTO;
                }
                break;
            }
            wf_windows_return_wake(adapter);
            break;
        }
        if (key != adapter->file_key) {
            if (first_error == 0) {
                first_error = EPROTO;
            }
            break;
        }
        processed += 1;
        error = wf_windows_publish_packet(
            adapter,
            overlapped,
            transferred,
            error_code
        );
        if (error != 0) {
            if (first_error == 0) {
                first_error = error;
            }
            break;
        }
        total += 1;
    }
    if (first_error != 0) {
        wf_windows_record_progress_error(adapter, first_error);
    }
    *published = total;
    return first_error;
}

/* One packet per announced sleeper, because a posted packet wakes exactly one
 * `GetQueuedCompletionStatus` waiter.  It is called by the completion core
 * under the wait lock a sleeper announces itself under, so the count read here
 * is the set of sleepers this wake has to reach. */
void wf_windows_iocp_notify(void *context) {
    wf_windows_iocp_adapter *adapter = (wf_windows_iocp_adapter *)context;
    unsigned parked;
    unsigned index;
    if (adapter == NULL || adapter->initialized == 0) {
        return;
    }
    parked = wf_completion_parked_scheduler_count(adapter->runtime);
    for (index = 0; index < parked; index += 1u) {
        if (PostQueuedCompletionStatus(
                adapter->port,
                0,
                adapter->wake_key,
                NULL
            ) == FALSE) {
            /* A port this process owns that will not take a packet leaves the
             * announced sleeper asleep forever, and no later wake can repair
             * it; fail-stop here rather than hang. */
            wf_windows_iocp_fail(
                "a port this process owns refused a wake packet, which would leave an announced sleeper asleep forever"
            );
        }
        atomic_fetch_add_explicit(
            &adapter->stat_host_wake_posts,
            1,
            memory_order_relaxed
        );
    }
}

int wf_windows_iocp_park(
    wf_windows_iocp_adapter *adapter,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
) {
    DWORD transferred = 0;
    ULONG_PTR key = 0;
    OVERLAPPED *overlapped = NULL;
    DWORD timeout;
    BOOL succeeded;
    DWORD error_code;
    int error;

    if (adapter == NULL || adapter->initialized == 0) {
        return EINVAL;
    }
    error = wf_windows_iocp_progress_error(adapter);
    if (error != 0) {
        return error;
    }
    /* Announce sleep under the same lock core publishers take.  A publisher
     * before this point leaves only an epoch fact and posts no packet; one
     * after the recheck sees the announced sleeper and posts for it. */
    wf_completion_wait_lock(&adapter->runtime->wait);
    if (wf_completion_wake_epoch(adapter->runtime) != observed_epoch) {
        wf_completion_wait_unlock(&adapter->runtime->wait);
        return 0;
    }
    /* Sequentially consistent, and paired with the sequentially consistent
     * epoch load below: a core publisher raises the epoch and then reads this
     * count without taking the wait's lock, so this announcement and that read
     * are what keeps a posted wake from being lost. */
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
    if (atomic_load_explicit(
            &adapter->runtime->wake_epoch,
            memory_order_seq_cst
        ) != observed_epoch) {
        atomic_fetch_sub_explicit(
            &adapter->runtime->parked_schedulers,
            1,
            memory_order_relaxed
        );
        wf_completion_wait_unlock(&adapter->runtime->wait);
        return 0;
    }
    wf_completion_wait_unlock(&adapter->runtime->wait);

    timeout = timeout_milliseconds == UINT32_MAX
        ? INFINITE
        : (DWORD)timeout_milliseconds;
    atomic_fetch_add_explicit(
        &adapter->stat_kernel_waits,
        1,
        memory_order_relaxed
    );
    succeeded = GetQueuedCompletionStatus(
        adapter->port,
        &transferred,
        &key,
        &overlapped,
        timeout
    );
    error_code = succeeded != FALSE ? 0u : GetLastError();

    wf_completion_wait_lock(&adapter->runtime->wait);
    {
        unsigned previous = atomic_fetch_sub_explicit(
            &adapter->runtime->parked_schedulers,
            1,
            memory_order_relaxed
        );
        if (previous == 0) {
            wf_windows_iocp_fail(
                "a thread left the parked count when the count was already zero"
            );
        }
    }
    wf_completion_wait_unlock(&adapter->runtime->wait);

    if (overlapped != NULL) {
        if (key != adapter->file_key) {
            wf_windows_record_progress_error(adapter, EPROTO);
            return EPROTO;
        }
        error = wf_windows_publish_packet(
            adapter,
            overlapped,
            transferred,
            error_code
        );
        if (error != 0) {
            wf_windows_record_progress_error(adapter, error);
        }
        return error;
    }
    if (succeeded != FALSE) {
        /* A wake packet: this thread's own, consumed here. */
        if (key != adapter->wake_key) {
            wf_windows_record_progress_error(adapter, EPROTO);
            return EPROTO;
        }
        return 0;
    }
    if (error_code == WAIT_TIMEOUT) {
        return 0;
    }
    wf_windows_record_progress_error(adapter, (int)error_code);
    return (int)error_code;
}

size_t wf_windows_iocp_in_flight(const wf_windows_iocp_adapter *adapter) {
    if (adapter == NULL) {
        return 0;
    }
    return atomic_load_explicit(&adapter->in_flight, memory_order_acquire);
}

int wf_windows_iocp_destroy(wf_windows_iocp_adapter *adapter) {
    if (adapter == NULL || adapter->initialized == 0) {
        return EINVAL;
    }
    if (wf_windows_iocp_in_flight(adapter) != 0) {
        return EBUSY;
    }
    if (CloseHandle(adapter->port) == FALSE) {
        return (int)GetLastError();
    }
    adapter->port = NULL;
    adapter->initialized = 0;
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
    statistics.inline_completions = atomic_load_explicit(
        &adapter->stat_inline_completions,
        memory_order_relaxed
    );
    statistics.dequeued_completions = atomic_load_explicit(
        &adapter->stat_dequeued_completions,
        memory_order_relaxed
    );
    statistics.completions = atomic_load_explicit(
        &adapter->stat_completions,
        memory_order_relaxed
    );
    statistics.kernel_waits = atomic_load_explicit(
        &adapter->stat_kernel_waits,
        memory_order_relaxed
    );
    statistics.host_wake_posts = atomic_load_explicit(
        &adapter->stat_host_wake_posts,
        memory_order_relaxed
    );
    return statistics;
}

#else

typedef int wf_windows_iocp_translation_unit_is_target_guarded;

#endif
