#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#if defined(_WIN32)

#include "windows_completion.h"
#include "windows_iocp.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define PROBE_CHECK(expression, code)                                         \
    do {                                                                      \
        if (!(expression)) {                                                  \
            return (code);                                                    \
        }                                                                     \
    } while (0)

static unsigned ready_count;
static void *last_ready_frame;
static unsigned descriptor_lease_releases;

enum probe_iocp_call_mode {
    PROBE_IOCP_CALL_REAL = 0,
    PROBE_IOCP_CALL_INLINE_SUCCESS = 1,
    PROBE_IOCP_CALL_INLINE_RESULT_ERROR = 2,
    PROBE_IOCP_CALL_PENDING = 3,
    PROBE_IOCP_CALL_IMMEDIATE_ERROR = 4
};

static unsigned probe_iocp_call_mode;
static unsigned probe_iocp_notification_failure;

BOOL WINAPI wf_windows_iocp_probe_read(
    HANDLE handle,
    LPVOID buffer,
    DWORD count,
    LPDWORD transferred,
    LPOVERLAPPED overlapped
) {
    static const unsigned char bytes[4] = {'i', 'n', 'l', 'n'};
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_REAL) {
        return ReadFile(handle, buffer, count, transferred, overlapped);
    }
    if (handle == NULL || handle == INVALID_HANDLE_VALUE || buffer == NULL
        || count != sizeof(bytes) || transferred != NULL
        || overlapped == NULL) {
        abort();
    }
    memcpy(buffer, bytes, sizeof(bytes));
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_INLINE_SUCCESS
        || probe_iocp_call_mode == PROBE_IOCP_CALL_INLINE_RESULT_ERROR) {
        return TRUE;
    }
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_PENDING) {
        SetLastError(ERROR_IO_PENDING);
        return FALSE;
    }
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_IMMEDIATE_ERROR) {
        SetLastError(ERROR_ACCESS_DENIED);
        return FALSE;
    }
    abort();
}

BOOL WINAPI wf_windows_iocp_probe_write(
    HANDLE handle,
    LPCVOID buffer,
    DWORD count,
    LPDWORD transferred,
    LPOVERLAPPED overlapped
) {
    if (probe_iocp_call_mode != PROBE_IOCP_CALL_REAL) {
        abort();
    }
    return WriteFile(handle, buffer, count, transferred, overlapped);
}

BOOL WINAPI wf_windows_iocp_probe_result(
    HANDLE handle,
    LPOVERLAPPED overlapped,
    LPDWORD transferred,
    BOOL wait
) {
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_REAL) {
        return GetOverlappedResult(handle, overlapped, transferred, wait);
    }
    if (handle == NULL || handle == INVALID_HANDLE_VALUE
        || overlapped == NULL || transferred == NULL || wait != FALSE) {
        abort();
    }
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_INLINE_SUCCESS) {
        *transferred = 4;
        return TRUE;
    }
    if (probe_iocp_call_mode == PROBE_IOCP_CALL_INLINE_RESULT_ERROR) {
        SetLastError(ERROR_CRC);
        return FALSE;
    }
    abort();
}

BOOL WINAPI wf_windows_iocp_probe_notification(HANDLE handle, UCHAR flags) {
    UCHAR expected = (UCHAR)(FILE_SKIP_COMPLETION_PORT_ON_SUCCESS
        | FILE_SKIP_SET_EVENT_ON_HANDLE);
    if (handle == NULL || handle == INVALID_HANDLE_VALUE
        || flags != expected) {
        abort();
    }
    if (probe_iocp_notification_failure != 0) {
        SetLastError(ERROR_NOT_SUPPORTED);
        return FALSE;
    }
    return SetFileCompletionNotificationModes(handle, flags);
}

void wf__windows_completion_descriptor_lease_release(
    wf_windows_descriptor_lease *lease
) {
    if (lease == NULL || lease->descriptor != 19 || lease->generation != 1
        || lease->handle == NULL || lease->handle == INVALID_HANDLE_VALUE
        || lease->completion_owner == NULL
        || lease->descriptor_class != WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE
        || lease->mode != WF_WINDOWS_DESCRIPTOR_LEASE_SHARED) {
        abort();
    }
    descriptor_lease_releases += 1;
    memset(lease, 0, sizeof(*lease));
}

#define PROBE_IOCP_WAITERS 4u
#define PROBE_IOCP_NOTIFY_STORM 4096u
#define PROBE_IOCP_TIMEOUT_MILLISECONDS 10000u

typedef struct probe_iocp_waiter {
    wf_completion_runtime *runtime;
    wf_windows_iocp_adapter *adapter;
    HANDLE release;
    enum wf_completion_park_result begin_result;
    int progress_error;
    size_t published;
} probe_iocp_waiter;

typedef struct probe_iocp_progress {
    wf_windows_iocp_adapter *adapter;
    int error;
    size_t published;
} probe_iocp_progress;

static void record_ready_frame(void *frame) {
    ready_count += 1;
    last_ready_frame = frame;
}

static DWORD WINAPI probe_iocp_waiter_main(void *opaque) {
    probe_iocp_waiter *waiter = (probe_iocp_waiter *)opaque;
    uint64_t epoch = wf_completion_wake_epoch(waiter->runtime);
    DWORD released;
    waiter->begin_result = wf_windows_completion_iocp_wait_begin(
        waiter->runtime,
        epoch
    );
    if (waiter->begin_result != WF_COMPLETION_PARK_WOKEN) {
        return 1;
    }
    released = WaitForSingleObject(
        waiter->release,
        PROBE_IOCP_TIMEOUT_MILLISECONDS
    );
    if (released != WAIT_OBJECT_0) {
        wf_windows_completion_iocp_wait_end(waiter->runtime);
        return 2;
    }
    waiter->progress_error = wf_windows_iocp_progress(
        waiter->adapter,
        PROBE_IOCP_WAITERS,
        PROBE_IOCP_TIMEOUT_MILLISECONDS,
        &waiter->published
    );
    return waiter->progress_error == 0 && waiter->published == 0 ? 0 : 3;
}

static DWORD WINAPI probe_iocp_progress_main(void *opaque) {
    probe_iocp_progress *progress = (probe_iocp_progress *)opaque;
    progress->error = wf_windows_iocp_progress(
        progress->adapter,
        1,
        INFINITE,
        &progress->published
    );
    return progress->error == 0 && progress->published == 0 ? 0 : 1;
}

static int probe_wait_for_iocp_waiters(
    const wf_completion_runtime *runtime,
    unsigned expected
) {
    ULONGLONG deadline = GetTickCount64() + PROBE_IOCP_TIMEOUT_MILLISECONDS;
    while (wf_windows_completion_iocp_waiter_count(runtime) != expected) {
        if (GetTickCount64() >= deadline) {
            return 1;
        }
        Sleep(1);
    }
    return 0;
}

static int probe_wait_for_active_progress(
    const wf_windows_iocp_adapter *adapter,
    size_t expected
) {
    ULONGLONG deadline = GetTickCount64() + PROBE_IOCP_TIMEOUT_MILLISECONDS;
    while (wf_windows_iocp_active_progress(adapter) != expected) {
        if (GetTickCount64() >= deadline) {
            return 1;
        }
        Sleep(1);
    }
    return 0;
}

static wf_completion_publication value_publication(uint64_t *value) {
    wf_completion_publication publication;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = 1;
    publication.result = value;
    publication.result_size = sizeof(*value);
    return publication;
}

static int finish_inline(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint64_t value
) {
    wf_completion_publication publication = value_publication(&value);
    return wf_completion_begin_submit(runtime, token)
                == WF_COMPLETION_TRANSITIONED
            && wf_completion_publish_inline_terminal(
                   runtime,
                   token,
                   &publication
               ) == WF_COMPLETION_PUBLISHED
        ? 0
        : 1;
}

static int test_core_product_and_generation(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[3];
    wf_completion_token operations[2];
    wf_completion_token third;
    wf_completion_token replacement;
    wf_completion_event events[3];
    wf_completion_publication first_publication;
    wf_completion_publication stale_publication;
    wf_completion_outcome outcome;
    wf_completion_statistics statistics;
    uint64_t first_value = UINT64_C(0x1020304050607080);
    uint64_t stale_value = UINT64_C(0xffffffffffffffff);
    uint64_t consumed = 0;
    LONG64 cursor_before;
    int first_frame = 1;
    int second_frame = 2;
    size_t drained;
    size_t index;

    ready_count = 0;
    last_ready_frame = NULL;
    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 3) == 0, 20);
    PROBE_CHECK(
        wf_completion_set_ready_callback(&runtime, record_ready_frame) == 0,
        21
    );
    PROBE_CHECK(
        wf_completion_set_ready_callback(&runtime, record_ready_frame)
            == EBUSY,
        22
    );

    PROBE_CHECK(
        wf_completion_claim(&runtime, &operations[0]) == WF_COMPLETION_CLAIMED
            && wf_completion_claim(&runtime, &operations[1])
                == WF_COMPLETION_CLAIMED,
        23
    );
    PROBE_CHECK(operations[0].slot != operations[1].slot, 24);
    PROBE_CHECK(
        wf_completion_claim(&runtime, &third) == WF_COMPLETION_CLAIMED,
        26
    );
    PROBE_CHECK(
        wf_completion_claim(&runtime, &replacement)
            == WF_COMPLETION_CLAIM_WAIT_CAPACITY,
        27
    );

    PROBE_CHECK(
        wf_completion_set_adapter_tag(&runtime, operations[0], 73)
            == WF_COMPLETION_TRANSITIONED,
        28
    );
    PROBE_CHECK(
        wf_completion_depend(
            &runtime,
            operations[0],
            WF_COMPLETION_RESOURCE_RELEASED,
            &first_frame
        ) == WF_COMPLETION_DEPEND_REGISTERED,
        29
    );
    PROBE_CHECK(
        wf_completion_depend(
            &runtime,
            operations[0],
            WF_COMPLETION_RESULT_READY,
            &second_frame
        ) == WF_COMPLETION_DEPEND_DUPLICATE,
        30
    );
    PROBE_CHECK(
        wf_completion_begin_submit(&runtime, operations[0])
            == WF_COMPLETION_TRANSITIONED
            && wf_completion_target_accepted(&runtime, operations[0])
                == WF_COMPLETION_TRANSITIONED,
        31
    );
    first_publication = value_publication(&first_value);
    first_publication.milestones &= ~WF_COMPLETION_RESOURCE_RELEASED;
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            operations[0],
            &first_publication
        ) == WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL
            && wf_completion_set_adapter_tag(&runtime, operations[0], 74)
                == WF_COMPLETION_TRANSITION_INVALID_STATE,
        32
    );
    first_publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            operations[0],
            &first_publication
        ) == WF_COMPLETION_PUBLISHED,
        33
    );
    PROBE_CHECK(ready_count == 0 && last_ready_frame == NULL, 134);
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            operations[0],
            &first_publication
        ) == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL,
        35
    );
    PROBE_CHECK(
        wf_completion_drain_token(&runtime, operations[0], &events[0]) == 1
            && events[0].token.slot == operations[0].slot
            && events[0].token.generation == operations[0].generation
            && wf_completion_ready_event_count(&runtime) == 0,
        38
    );
    PROBE_CHECK(
        wf_completion_drain_token(&runtime, operations[0], &events[0]) == 0,
        139
    );
    PROBE_CHECK(ready_count == 1 && last_ready_frame == &first_frame, 136);
    PROBE_CHECK(
        wf_completion_depend(
            &runtime,
            operations[0],
            WF_COMPLETION_RESULT_READY,
            &second_frame
        ) == WF_COMPLETION_DEPEND_DUPLICATE,
        137
    );
    PROBE_CHECK(ready_count == 1 && last_ready_frame == &first_frame, 138);
    PROBE_CHECK(
        wf_completion_consume(
            &runtime,
            operations[0],
            &consumed,
            sizeof(consumed),
            &outcome
        ) == WF_COMPLETION_CONSUMED,
        39
    );
    PROBE_CHECK(
        consumed == first_value
            && outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE
            && outcome.terminal_kind == 1 && outcome.adapter_tag == 73
            && outcome.result_size == sizeof(consumed),
        40
    );

    /* Reuse the exact slot, then prove that its old generation can neither
     * publish bytes nor win a second terminal transition. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &replacement) == WF_COMPLETION_CLAIMED
            && replacement.slot == operations[0].slot
            && replacement.generation == operations[0].generation + 1,
        41
    );
    stale_publication = value_publication(&stale_value);
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            operations[0],
            &stale_publication
        ) == WF_COMPLETION_PUBLISH_STALE,
        42
    );

    PROBE_CHECK(finish_inline(&runtime, operations[1], 2) == 0, 43);
    PROBE_CHECK(finish_inline(&runtime, third, 3) == 0, 44);
    PROBE_CHECK(finish_inline(&runtime, replacement, 4) == 0, 45);
    PROBE_CHECK(
        wf_completion_drain_token(&runtime, operations[0], &events[0]) == 0,
        140
    );
    drained = wf_completion_drain(&runtime, events, 3, 3);
    PROBE_CHECK(drained == 3, 46);
    for (index = 0; index < drained; ++index) {
        consumed = 0;
        PROBE_CHECK(
            wf_completion_consume(
                &runtime,
                events[index].token,
                &consumed,
                sizeof(consumed),
                &outcome
            ) == WF_COMPLETION_CONSUMED,
            47
        );
        PROBE_CHECK(consumed >= 2 && consumed <= 4, 48);
    }
    statistics = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        statistics.claims == 4 && statistics.claim_capacity_waits == 1
            && statistics.publications == 4
            && statistics.stale_publications == 1
            && statistics.duplicate_terminals == 1
            && statistics.consumptions == 4,
        49
    );
    cursor_before = InterlockedCompareExchange64(
        &runtime.drain_cursor,
        0,
        0
    );
    PROBE_CHECK(
        wf_completion_drain(&runtime, events, 1, 3) == 0
            && InterlockedCompareExchange64(
                   &runtime.drain_cursor,
                   0,
                   0
               ) == cursor_before,
        155
    );
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 50);
    return 0;
}

static int drain_file_result(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    enum wf_windows_file_operation_kind kind,
    int64_t expected,
    uint32_t expected_adapter_tag
) {
    wf_completion_event event;
    wf_windows_file_result result;
    wf_completion_outcome outcome;
    if (wf_completion_drain_token(runtime, token, &event) != 1
        || event.token.slot != token.slot
        || event.token.generation != token.generation
        || event.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE
        || event.terminal_kind != 1
        || wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) != WF_COMPLETION_CONSUMED
        || result.kind != kind || result.value != expected
        || result.error_code != 0
        || outcome.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE
        || outcome.terminal_kind != 1
        || outcome.adapter_tag != expected_adapter_tag
        || outcome.result_size != sizeof(result)) {
        return 1;
    }
    return 0;
}

static int drain_file_error(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t expected_error,
    uint32_t expected_adapter_tag
) {
    wf_completion_event event;
    wf_windows_file_result result;
    wf_completion_outcome outcome;
    if (wf_completion_drain_token(runtime, token, &event) != 1
        || event.token.slot != token.slot
        || event.token.generation != token.generation
        || event.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE
        || event.terminal_kind != 2
        || wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) != WF_COMPLETION_CONSUMED
        || result.kind != WF_WINDOWS_FILE_READ_AT || result.value != -1
        || result.error_code != expected_error
        || outcome.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE
        || outcome.terminal_kind != 2
        || outcome.adapter_tag != expected_adapter_tag
        || outcome.result_size != sizeof(result)) {
        return 1;
    }
    return 0;
}

/* The adapter withdraws the announced waiter before publishing the first real
 * completion. The publication and capacity notifications must therefore not
 * enqueue a self-wake which survives after progress returns. */
static int progress_one_real_completion(
    wf_completion_runtime *runtime,
    wf_windows_iocp_adapter *adapter
) {
    enum wf_completion_park_result park;
    uint64_t epoch = wf_completion_wake_epoch(runtime);
    size_t published = SIZE_MAX;
    int error;
    park = wf_windows_completion_iocp_wait_begin(runtime, epoch);
    if (park != WF_COMPLETION_PARK_WOKEN) {
        return 1;
    }
    error = wf_windows_iocp_progress(adapter, 1, INFINITE, &published);
    return error == 0 && published == 1
            && wf_windows_completion_iocp_waiter_count(runtime) == 0
            && wf_windows_completion_iocp_wake_packet_count(runtime) == 0
        ? 0
        : 1;
}

static int test_iocp_waiter_broadcast(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[1];
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[1];
    probe_iocp_waiter waiters[PROBE_IOCP_WAITERS];
    HANDLE threads[PROBE_IOCP_WAITERS] = {0};
    HANDLE release;
    wf_completion_statistics before;
    wf_completion_statistics after;
    DWORD joined;
    size_t index;

    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 1) == 0, 89);
    /* One active IOCP thread makes the anti-drain assertion deterministic: an
     * adapter which continued after its first wake could consume all four
     * packets before Windows released a peer. The fixed adapter returns after
     * one, letting each announced real thread receive its own persistent wake. */
    PROBE_CHECK(
        wf_windows_iocp_init(&adapter, &runtime, entries, 1, 1, 0) == 0,
        90
    );
    PROBE_CHECK(
        wf_windows_completion_bind_iocp(
            &runtime,
            &adapter,
            wf_windows_iocp_port(&adapter),
            wf_windows_iocp_wake_key(&adapter)
        ) == 0,
        91
    );
    release = CreateEventA(NULL, TRUE, FALSE, NULL);
    PROBE_CHECK(release != NULL, 92);
    memset(waiters, 0, sizeof(waiters));
    for (index = 0; index < PROBE_IOCP_WAITERS; ++index) {
        waiters[index].runtime = &runtime;
        waiters[index].adapter = &adapter;
        waiters[index].release = release;
        threads[index] = CreateThread(
            NULL,
            0,
            probe_iocp_waiter_main,
            &waiters[index],
            0,
            NULL
        );
        PROBE_CHECK(threads[index] != NULL, 93);
    }
    PROBE_CHECK(
        probe_wait_for_iocp_waiters(&runtime, PROBE_IOCP_WAITERS) == 0,
        94
    );

    before = wf_completion_statistics_snapshot(&runtime);
    wf_completion_notify_compute(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wake_packet_count(&runtime)
            == PROBE_IOCP_WAITERS,
        95
    );
    for (index = 0; index < PROBE_IOCP_NOTIFY_STORM; ++index) {
        wf_completion_notify_compute(&runtime);
    }
    after = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wake_packet_count(&runtime)
                == PROBE_IOCP_WAITERS
            && after.wake_signals - before.wake_signals
                == PROBE_IOCP_WAITERS
            && after.compute_notifications - before.compute_notifications
                == PROBE_IOCP_NOTIFY_STORM + 1u,
        96
    );

    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == EBUSY, 97);
    PROBE_CHECK(SetEvent(release) != FALSE, 98);
    joined = WaitForMultipleObjects(
        PROBE_IOCP_WAITERS,
        threads,
        TRUE,
        PROBE_IOCP_TIMEOUT_MILLISECONDS
    );
    PROBE_CHECK(joined == WAIT_OBJECT_0, 99);
    for (index = 0; index < PROBE_IOCP_WAITERS; ++index) {
        DWORD exit_code = UINT32_MAX;
        PROBE_CHECK(
            GetExitCodeThread(threads[index], &exit_code) != FALSE
                && exit_code == 0,
            100
        );
        PROBE_CHECK(CloseHandle(threads[index]) != FALSE, 101);
    }
    PROBE_CHECK(
        wf_windows_completion_iocp_waiter_count(&runtime) == 0
            && wf_windows_completion_iocp_wake_packet_count(&runtime) == 0,
        102
    );

    /* The runtime must not forget a still-live port. The adapter owns and
     * detaches it first; only then may the core's storage disappear. */
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == EBUSY, 103);
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == 0, 104);
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 105);
    PROBE_CHECK(CloseHandle(release) != FALSE, 106);
    return 0;
}

static int test_iocp_progress_lifetime(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[1];
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[1];
    probe_iocp_progress progress = {0};
    HANDLE thread;
    DWORD joined;
    DWORD exit_code = UINT32_MAX;

    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 1) == 0, 107);
    PROBE_CHECK(
        wf_windows_iocp_init(&adapter, &runtime, entries, 1, 1, 0) == 0,
        108
    );
    progress.adapter = &adapter;
    thread = CreateThread(
        NULL,
        0,
        probe_iocp_progress_main,
        &progress,
        0,
        NULL
    );
    PROBE_CHECK(thread != NULL, 109);
    PROBE_CHECK(probe_wait_for_active_progress(&adapter, 1) == 0, 110);
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == EBUSY, 111);
    PROBE_CHECK(
        PostQueuedCompletionStatus(
            wf_windows_iocp_port(&adapter),
            0,
            wf_windows_iocp_wake_key(&adapter),
            NULL
        ) != FALSE,
        112
    );
    joined = WaitForSingleObject(thread, PROBE_IOCP_TIMEOUT_MILLISECONDS);
    PROBE_CHECK(joined == WAIT_OBJECT_0, 113);
    PROBE_CHECK(
        GetExitCodeThread(thread, &exit_code) != FALSE && exit_code == 0,
        114
    );
    PROBE_CHECK(CloseHandle(thread) != FALSE, 115);
    PROBE_CHECK(
        wf_windows_iocp_active_progress(&adapter) == 0
            && wf_windows_iocp_destroy(&adapter) == 0
            && wf_completion_runtime_destroy(&runtime) == 0,
        116
    );
    return 0;
}

static int test_real_iocp(const char *path) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[1];
    wf_windows_iocp_file file;
    wf_windows_file_request request;
    wf_completion_token first_write;
    wf_completion_token second_write;
    wf_completion_token read_token;
    wf_completion_token eof_token;
    wf_windows_iocp_statistics adapter_statistics;
    wf_completion_statistics core_statistics;
    const unsigned char payload[4] = {'w', 'i', 'n', '!'};
    unsigned char result_bytes[4] = {0};
    size_t published = SIZE_MAX;
    HANDLE handle;
    uint64_t epoch;
    int error;

    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0, 60);
    PROBE_CHECK(
        wf_windows_iocp_init(&adapter, &runtime, entries, 1, 0, 2u)
            == EINVAL,
        141
    );
    PROBE_CHECK(
        wf_windows_iocp_init(&adapter, &runtime, entries, 1, 0, 0) == 0,
        61
    );
    PROBE_CHECK(
        wf_windows_completion_bind_iocp(
            &runtime,
            &adapter,
            wf_windows_iocp_port(&adapter),
            wf_windows_iocp_wake_key(&adapter)
        ) == 0,
        62
    );
    handle = CreateFileA(
        path,
        GENERIC_READ | GENERIC_WRITE,
        0,
        NULL,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    PROBE_CHECK(
        handle != INVALID_HANDLE_VALUE
            && wf_windows_iocp_associate_file(&adapter, handle, &file) == 0,
        63
    );

    PROBE_CHECK(
        wf_completion_claim(&runtime, &first_write) == WF_COMPLETION_CLAIMED
            && wf_completion_claim(&runtime, &second_write)
                == WF_COMPLETION_CLAIMED,
        64
    );
    PROBE_CHECK(
        wf_completion_set_adapter_tag(&runtime, first_write, 101)
                == WF_COMPLETION_TRANSITIONED
            && wf_completion_set_adapter_tag(&runtime, second_write, 102)
                == WF_COMPLETION_TRANSITIONED,
        65
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_WRITE_AT;
    request.file = file;
    request.lease.descriptor = 19;
    request.lease.generation = 1;
    request.lease.handle = handle;
    request.lease.completion_owner = &adapter;
    request.lease.descriptor_class = WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE;
    request.lease.mode = WF_WINDOWS_DESCRIPTOR_LEASE_SHARED;
    request.buffer.write_buffer = payload;
    request.count = 2;
    request.offset = 0;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, first_write, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        66
    );
    request.buffer.write_buffer = payload + 2;
    request.offset = 2;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, second_write, &request)
            == WF_WINDOWS_IOCP_WAIT_CAPACITY,
        67
    );
    PROBE_CHECK(
        progress_one_real_completion(&runtime, &adapter) == 0,
        68
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            first_write,
            WF_WINDOWS_FILE_WRITE_AT,
            2,
            101
        ) == 0,
        69
    );

    /* Queue a compute wake before the second real operation. The first
     * bounded progress step must consume only that wake; the next one receives
     * the real completion after withdrawing its own waiter announcement. */
    epoch = wf_completion_wake_epoch(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wait_begin(&runtime, epoch)
            == WF_COMPLETION_PARK_WOKEN,
        70
    );
    wf_completion_notify_compute(&runtime);
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, second_write, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        71
    );
    published = SIZE_MAX;
    error = wf_windows_iocp_progress(&adapter, 1, INFINITE, &published);
    PROBE_CHECK(error == 0 && published == 0, 72);
    PROBE_CHECK(
        progress_one_real_completion(&runtime, &adapter) == 0,
        73
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            second_write,
            WF_WINDOWS_FILE_WRITE_AT,
            2,
            102
        ) == 0,
        74
    );

    PROBE_CHECK(
        wf_completion_claim(&runtime, &read_token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, read_token, 103)
                == WF_COMPLETION_TRANSITIONED,
        75
    );
    request.kind = WF_WINDOWS_FILE_READ_AT;
    request.buffer.read_buffer = result_bytes;
    request.count = sizeof(result_bytes);
    request.offset = 0;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, read_token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        76
    );
    PROBE_CHECK(
        progress_one_real_completion(&runtime, &adapter) == 0,
        77
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            read_token,
            WF_WINDOWS_FILE_READ_AT,
            sizeof(result_bytes),
            103
        ) == 0
            && memcmp(result_bytes, payload, sizeof(payload)) == 0,
        78
    );

    /* Overlapped disk EOF arrives as ERROR_HANDLE_EOF on Windows, but the
     * language file-read contract observes it as successful progress of zero
     * bytes. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &eof_token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, eof_token, 104)
                == WF_COMPLETION_TRANSITIONED,
        79
    );
    request.buffer.read_buffer = result_bytes;
    request.count = 1;
    request.offset = sizeof(payload);
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, eof_token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        80
    );
    /* ReadFile may report ERROR_HANDLE_EOF immediately, in which case the
     * adapter has already published and there is deliberately no kernel
     * completion packet to wait for. Otherwise drive the pending request. */
    if (wf_completion_ready_event_count(&runtime) == 0) {
        PROBE_CHECK(
            progress_one_real_completion(&runtime, &adapter) == 0,
            81
        );
    }
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            eof_token,
            WF_WINDOWS_FILE_READ_AT,
            0,
            104
        ) == 0,
        82
    );

    /* A null OVERLAPPED is accepted only with the runtime's reserved wake
     * key. The malformed packet is consumed, reported, and publishes no
     * terminal. */
    PROBE_CHECK(
        PostQueuedCompletionStatus(
            wf_windows_iocp_port(&adapter),
            0,
            (ULONG_PTR)1,
            NULL
        ) != FALSE,
        83
    );
    epoch = wf_completion_wake_epoch(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wait_begin(&runtime, epoch)
            == WF_COMPLETION_PARK_WOKEN,
        84
    );
    published = SIZE_MAX;
    PROBE_CHECK(
        wf_windows_iocp_progress(&adapter, 1, 0, &published) == EPROTO
            && published == 0,
        85
    );

    adapter_statistics = wf_windows_iocp_statistics_snapshot(&adapter);
    core_statistics = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        adapter_statistics.submissions == 4
            && adapter_statistics.capacity_waits == 1
            && adapter_statistics.inline_completions == 0
            && adapter_statistics.dequeued_completions
                    + adapter_statistics.immediate_failures
                == adapter_statistics.submissions
            && adapter_statistics.completions == 4
            && adapter_statistics.publication_failures == 0
            && core_statistics.publications == 4
            && core_statistics.target_capacity_waits == 1
            && core_statistics.compute_notifications == 1
            && wf_windows_iocp_in_flight(&adapter) == 0
            && descriptor_lease_releases == 4
            && entries[0].lease.generation == 0,
        86
    );
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == 0, 87);
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 88);
    PROBE_CHECK(CloseHandle(handle) != FALSE, 117);
    return 0;
}

/* Production associations suppress the port packet only when ReadFile says
 * the overlapped request already completed. A filesystem may still choose the
 * pending path, so this probe accepts either native outcome and proves both
 * converge on one terminal, one lease release, and the same result bytes. */
static int test_inline_success_mode(const char *path) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[1];
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[1];
    wf_windows_iocp_file file;
    wf_windows_file_request request;
    wf_completion_token token;
    wf_windows_iocp_statistics statistics;
    unsigned char bytes[4] = {0};
    const unsigned char expected[4] = {'w', 'i', 'n', '!'};
    unsigned releases_before = descriptor_lease_releases;
    wf_windows_iocp_file failed_file = {INVALID_HANDLE_VALUE, NULL};
    size_t published = SIZE_MAX;
    uint64_t epoch;
    HANDLE failed_handle;
    HANDLE handle;
    int association_error;

    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 1) == 0, 142);
    PROBE_CHECK(
        wf_windows_iocp_init(
            &adapter,
            &runtime,
            entries,
            1,
            0,
            WF_WINDOWS_IOCP_INLINE_SYNCHRONOUS_SUCCESS
        ) == 0,
        143
    );
    PROBE_CHECK(
        wf_windows_completion_bind_iocp(
            &runtime,
            &adapter,
            wf_windows_iocp_port(&adapter),
            wf_windows_iocp_wake_key(&adapter)
        ) == 0,
        144
    );
    failed_handle = CreateFileA(
        path,
        GENERIC_READ,
        FILE_SHARE_READ,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    PROBE_CHECK(failed_handle != INVALID_HANDLE_VALUE, 156);
    probe_iocp_notification_failure = 1;
    association_error = wf_windows_iocp_associate_file(
        &adapter,
        failed_handle,
        &failed_file
    );
    probe_iocp_notification_failure = 0;
    PROBE_CHECK(
        association_error == ERROR_NOT_SUPPORTED
            && failed_file.handle == INVALID_HANDLE_VALUE
            && failed_file.adapter == NULL,
        157
    );
    PROBE_CHECK(CloseHandle(failed_handle) != FALSE, 158);
    handle = CreateFileA(
        path,
        GENERIC_READ,
        FILE_SHARE_READ,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    PROBE_CHECK(
        handle != INVALID_HANDLE_VALUE
            && wf_windows_iocp_associate_file(&adapter, handle, &file) == 0,
        145
    );
    PROBE_CHECK(
        wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, token, 105)
                == WF_COMPLETION_TRANSITIONED,
        146
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_READ_AT;
    request.file = file;
    request.lease.descriptor = 19;
    request.lease.generation = 1;
    request.lease.handle = handle;
    request.lease.completion_owner = &adapter;
    request.lease.descriptor_class = WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE;
    request.lease.mode = WF_WINDOWS_DESCRIPTOR_LEASE_SHARED;
    request.buffer.read_buffer = bytes;
    request.count = sizeof(bytes);
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        147
    );
    statistics = wf_windows_iocp_statistics_snapshot(&adapter);
    PROBE_CHECK(statistics.inline_completions <= 1, 148);
    if (statistics.inline_completions == 0) {
        PROBE_CHECK(
            progress_one_real_completion(&runtime, &adapter) == 0,
            149
        );
    }
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            token,
            WF_WINDOWS_FILE_READ_AT,
            sizeof(bytes),
            105
        ) == 0
            && memcmp(bytes, expected, sizeof(expected)) == 0,
        150
    );
    if (statistics.inline_completions != 0) {
        /* An inline terminal is safe only if skip-on-success suppressed the
         * kernel packet. Poll after the entry is free: a redundant packet
         * would identify that free/reusable OVERLAPPED and return EPROTO. */
        epoch = wf_completion_wake_epoch(&runtime);
        PROBE_CHECK(
            wf_windows_completion_iocp_wait_begin(&runtime, epoch)
                == WF_COMPLETION_PARK_WOKEN,
            184
        );
        published = SIZE_MAX;
        PROBE_CHECK(
            wf_windows_iocp_progress(&adapter, 1, 0, &published) == 0
                && published == 0
                && wf_windows_completion_iocp_waiter_count(&runtime) == 0
                && wf_windows_completion_iocp_wake_packet_count(&runtime)
                    == 0,
            185
        );
    }
    statistics = wf_windows_iocp_statistics_snapshot(&adapter);
    PROBE_CHECK(
        statistics.submissions == 1 && statistics.completions == 1
            && statistics.inline_completions <= 1
            && statistics.inline_completions
                    + statistics.dequeued_completions
                    + statistics.immediate_failures
                == statistics.submissions
            && statistics.publication_failures == 0
            && wf_windows_iocp_in_flight(&adapter) == 0
            && descriptor_lease_releases == releases_before + 1,
        151
    );
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == 0, 152);
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 153);
    PROBE_CHECK(CloseHandle(handle) != FALSE, 154);
    return 0;
}

/* These injected host results exercise every ownership route independent of
 * the filesystem cache policy on the runner. The production translation unit
 * has no function-pointer dispatch: only this probe build names the calls at
 * compile time. */
static int test_injected_iocp_outcomes(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[1];
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[1];
    wf_windows_iocp_file file;
    wf_windows_file_request request;
    wf_completion_token token;
    wf_windows_iocp_statistics adapter_statistics;
    wf_completion_statistics core_statistics;
    unsigned char bytes[4] = {0};
    const unsigned char expected[4] = {'i', 'n', 'l', 'n'};
    unsigned releases_before = descriptor_lease_releases;
    size_t published = SIZE_MAX;
    uint64_t epoch;
    HANDLE fake_handle;

    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 1) == 0, 159);
    PROBE_CHECK(
        wf_windows_iocp_init(
            &adapter,
            &runtime,
            entries,
            1,
            1,
            WF_WINDOWS_IOCP_INLINE_SYNCHRONOUS_SUCCESS
        ) == 0,
        160
    );
    PROBE_CHECK(
        wf_windows_completion_bind_iocp(
            &runtime,
            &adapter,
            wf_windows_iocp_port(&adapter),
            wf_windows_iocp_wake_key(&adapter)
        ) == 0,
        161
    );
    fake_handle = CreateEventA(NULL, TRUE, FALSE, NULL);
    PROBE_CHECK(fake_handle != NULL, 162);
    file.handle = fake_handle;
    file.adapter = &adapter;
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_READ_AT;
    request.file = file;
    request.lease.descriptor = 19;
    request.lease.generation = 1;
    request.lease.handle = fake_handle;
    request.lease.completion_owner = &adapter;
    request.lease.descriptor_class = WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE;
    request.lease.mode = WF_WINDOWS_DESCRIPTOR_LEASE_SHARED;
    request.buffer.read_buffer = bytes;
    request.count = sizeof(bytes);

    /* A synchronous success must publish on the submitter and wake a
     * scheduler which had already committed to GQCS. That wake carries no
     * second terminal publication. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, token, 201)
                == WF_COMPLETION_TRANSITIONED,
        163
    );
    epoch = wf_completion_wake_epoch(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wait_begin(&runtime, epoch)
            == WF_COMPLETION_PARK_WOKEN,
        164
    );
    probe_iocp_call_mode = PROBE_IOCP_CALL_INLINE_SUCCESS;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, token, &request)
                == WF_WINDOWS_IOCP_TARGET_OWNS
            && wf_completion_ready_event_count(&runtime) == 1
            && wf_windows_iocp_in_flight(&adapter) == 0
            && wf_windows_completion_iocp_waiter_count(&runtime) == 1
            && wf_windows_completion_iocp_wake_packet_count(&runtime) == 1,
        165
    );
    published = SIZE_MAX;
    PROBE_CHECK(
        wf_windows_iocp_progress(&adapter, 1, 0, &published) == 0
            && published == 0
            && wf_windows_completion_iocp_waiter_count(&runtime) == 0
            && wf_windows_completion_iocp_wake_packet_count(&runtime) == 0,
        166
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            token,
            WF_WINDOWS_FILE_READ_AT,
            sizeof(bytes),
            201
        ) == 0
            && memcmp(bytes, expected, sizeof(expected)) == 0,
        167
    );

    /* TRUE proves the operation terminal even if the result query reports a
     * terminal host error. It must still retire the sole entry and lease. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, token, 202)
                == WF_COMPLETION_TRANSITIONED,
        168
    );
    probe_iocp_call_mode = PROBE_IOCP_CALL_INLINE_RESULT_ERROR;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        169
    );
    PROBE_CHECK(
        drain_file_error(&runtime, token, ERROR_CRC, 202) == 0,
        170
    );

    /* A failed initiation other than ERROR_IO_PENDING owns no kernel packet;
     * the submitter publishes that terminal exactly once. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, token, 203)
                == WF_COMPLETION_TRANSITIONED,
        171
    );
    probe_iocp_call_mode = PROBE_IOCP_CALL_IMMEDIATE_ERROR;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        172
    );
    PROBE_CHECK(
        drain_file_error(&runtime, token, ERROR_ACCESS_DENIED, 203) == 0,
        173
    );

    /* ERROR_IO_PENDING transfers ownership to the port. Only the dequeued
     * packet may publish and release the entry. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, token, 204)
                == WF_COMPLETION_TRANSITIONED,
        174
    );
    epoch = wf_completion_wake_epoch(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wait_begin(&runtime, epoch)
            == WF_COMPLETION_PARK_WOKEN,
        175
    );
    probe_iocp_call_mode = PROBE_IOCP_CALL_PENDING;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, token, &request)
                == WF_WINDOWS_IOCP_TARGET_OWNS
            && wf_windows_iocp_in_flight(&adapter) == 1
            && wf_completion_ready_event_count(&runtime) == 0,
        176
    );
    PROBE_CHECK(
        PostQueuedCompletionStatus(
            wf_windows_iocp_port(&adapter),
            (DWORD)sizeof(bytes),
            (ULONG_PTR)&adapter,
            &entries[0].overlapped
        ) != FALSE,
        177
    );
    published = SIZE_MAX;
    PROBE_CHECK(
        wf_windows_iocp_progress(&adapter, 1, 0, &published) == 0
            && published == 1
            && wf_windows_completion_iocp_waiter_count(&runtime) == 0
            && wf_windows_completion_iocp_wake_packet_count(&runtime) == 0,
        178
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            token,
            WF_WINDOWS_FILE_READ_AT,
            sizeof(bytes),
            204
        ) == 0,
        179
    );

    probe_iocp_call_mode = PROBE_IOCP_CALL_REAL;
    adapter_statistics = wf_windows_iocp_statistics_snapshot(&adapter);
    core_statistics = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        adapter_statistics.submissions == 4
            && adapter_statistics.inline_completions == 2
            && adapter_statistics.dequeued_completions == 1
            && adapter_statistics.immediate_failures == 1
            && adapter_statistics.submissions
                == adapter_statistics.inline_completions
                    + adapter_statistics.dequeued_completions
                    + adapter_statistics.immediate_failures
            && adapter_statistics.completions == 4
            && adapter_statistics.publication_failures == 0
            && core_statistics.publications == 4
            && core_statistics.drained_events == 4
            && core_statistics.consumptions == 4
            && wf_windows_iocp_in_flight(&adapter) == 0
            && descriptor_lease_releases == releases_before + 4
            && entries[0].lease.generation == 0,
        180
    );
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == 0, 181);
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 182);
    PROBE_CHECK(CloseHandle(fake_handle) != FALSE, 183);
    return 0;
}

int main(int argc, char **argv) {
    int error;
    if (argc != 2) {
        return 2;
    }
    error = test_core_product_and_generation();
    if (error != 0) {
        return error;
    }
    error = test_iocp_waiter_broadcast();
    if (error != 0) {
        return error;
    }
    error = test_iocp_progress_lifetime();
    if (error != 0) {
        return error;
    }
    error = test_real_iocp(argv[1]);
    if (error != 0) {
        return error;
    }
    error = test_inline_success_mode(argv[1]);
    if (error != 0) {
        return error;
    }
    error = test_injected_iocp_outcomes();
    if (error != 0) {
        return error;
    }
    printf("windows-native-completion-probe status=pass\n");
    return 0;
}

#else

int main(void) {
    return 77;
}

#endif
