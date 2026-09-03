/* Native Windows proof for the bounded blocking adapter.
 *
 * The production adapter and completion core are linked unchanged.  The host
 * operations, descriptor-lease release, and process-wide open order are
 * deterministic probe stubs; every scheduler, queue, publication, retirement,
 * and consume transition remains production code. */

#if defined(_WIN32)

#include "windows_blocking.h"
#include "windows_runtime.h"

#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WF_WINDOWS_BLOCKING_PROBE_SLOTS 8u
#define WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS 10000u
#define WF_WINDOWS_BLOCKING_PROBE_JOBS 5u

#define PROBE_CHECK(expression, message)                                     \
    do {                                                                      \
        if (!(expression)) {                                                  \
            return wf_windows_blocking_probe_fail(message);                   \
        }                                                                     \
    } while (0)

static DWORD wf_probe_main_thread;
static HANDLE wf_probe_refusal_seen;
static HANDLE wf_probe_refusal_release;
static HANDLE wf_probe_write_entered;
static HANDLE wf_probe_write_release;
static HANDLE wf_probe_ticket_a_entered;
static HANDLE wf_probe_ticket_b_rotated;
static HANDLE wf_probe_ticket_release;
static volatile LONG wf_probe_wrong_thread;
static volatile LONG wf_probe_scenario_sequence;
static volatile LONG wf_probe_refusal_sequence;
static volatile LONG wf_probe_write_sequence;
static volatile LONG wf_probe_retry_sequence;
static volatile LONG wf_probe_retry_open_calls;
static volatile LONG wf_probe_write_calls;
static volatile LONG wf_probe_directory_calls;
static volatile LONG wf_probe_ticket_sequence;
static volatile LONG wf_probe_ticket_a_sequence;
static volatile LONG wf_probe_ticket_b_sequence;
static volatile LONG wf_probe_ticket_a_calls;
static volatile LONG wf_probe_ticket_b_calls;
static volatile LONG wf_probe_stub_failure;
static volatile LONG wf_probe_lease_releases;
static volatile LONG wf_probe_root_lease_releases;
static volatile LONG wf_probe_output_lease_releases;
static volatile LONG wf_probe_directory_lease_releases;
static SRWLOCK wf_probe_open_order_lock = SRWLOCK_INIT;
static CONDITION_VARIABLE wf_probe_open_order_ready =
    CONDITION_VARIABLE_INIT;
static _Atomic uint64_t wf_probe_next_open_ticket;
static uint64_t wf_probe_serving_open_ticket;
static volatile LONG wf_probe_open_order_enters;
static volatile LONG wf_probe_open_order_leaves;
static volatile LONG wf_probe_open_order_rotations;
static SRWLOCK wf_probe_resource_attempt_lock = SRWLOCK_INIT;
static volatile LONG wf_probe_resource_attempt_active;
static volatile LONG wf_probe_resource_attempt_enters;
static volatile LONG wf_probe_resource_attempt_leaves;
static int wf_probe_error;
static char wf_probe_root_handle_storage;
static char wf_probe_output_handle_storage;
static char wf_probe_directory_handle_storage;
static char wf_probe_completion_owner_storage;

static int wf_windows_blocking_probe_fail(const char *message) {
    fprintf(stderr, "windows blocking adapter probe: %s\n", message);
    return 1;
}

static LONG wf_windows_blocking_probe_load(const volatile LONG *value) {
    return InterlockedCompareExchange((volatile LONG *)value, 0, 0);
}

static void wf_windows_blocking_probe_worker_call(void) {
    if (GetCurrentThreadId() == wf_probe_main_thread) {
        InterlockedExchange(&wf_probe_wrong_thread, 1);
    }
}

static HANDLE wf_windows_blocking_probe_handle(int descriptor) {
    switch (descriptor) {
    case 1:
        return (HANDLE)(void *)&wf_probe_root_handle_storage;
    case 2:
        return (HANDLE)(void *)&wf_probe_output_handle_storage;
    case 3:
        return (HANDLE)(void *)&wf_probe_directory_handle_storage;
    default:
        return NULL;
    }
}

static uint64_t wf_windows_blocking_probe_generation(int descriptor) {
    return UINT64_C(0x7766000000000000) + (uint64_t)(unsigned)descriptor;
}

static wf_windows_descriptor_lease wf_windows_blocking_probe_lease(
    int descriptor,
    unsigned descriptor_class,
    unsigned mode
) {
    wf_windows_descriptor_lease lease;
    memset(&lease, 0, sizeof(lease));
    lease.descriptor = descriptor;
    lease.generation = wf_windows_blocking_probe_generation(descriptor);
    lease.handle = wf_windows_blocking_probe_handle(descriptor);
    lease.completion_owner = &wf_probe_completion_owner_storage;
    lease.descriptor_class = descriptor_class;
    lease.mode = mode;
    return lease;
}

static int wf_windows_blocking_probe_lease_is_zero(
    const wf_windows_descriptor_lease *lease
) {
    const unsigned char *bytes = (const unsigned char *)(const void *)lease;
    size_t index;
    if (lease == NULL) {
        return 0;
    }
    for (index = 0; index < sizeof(*lease); ++index) {
        if (bytes[index] != 0) {
            return 0;
        }
    }
    return 1;
}

void wf__windows_completion_descriptor_lease_release(
    wf_windows_descriptor_lease *lease
) {
    int valid = lease != NULL
        && lease->descriptor >= 1
        && lease->descriptor <= 3
        && lease->generation
            == wf_windows_blocking_probe_generation(lease->descriptor)
        && lease->handle
            == wf_windows_blocking_probe_handle(lease->descriptor)
        && lease->completion_owner == &wf_probe_completion_owner_storage;
    if (valid && lease->descriptor == 1) {
        valid = lease->descriptor_class
                == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT
            && lease->mode == WF_WINDOWS_DESCRIPTOR_LEASE_SHARED;
        if (valid) {
            InterlockedIncrement(&wf_probe_root_lease_releases);
        }
    } else if (valid && lease->descriptor == 2) {
        valid = lease->descriptor_class == WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT
            && lease->mode == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL;
        if (valid) {
            InterlockedIncrement(&wf_probe_output_lease_releases);
        }
    } else if (valid && lease->descriptor == 3) {
        valid = lease->descriptor_class
                == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE
            && lease->mode == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL;
        if (valid) {
            InterlockedIncrement(&wf_probe_directory_lease_releases);
        }
    }
    if (!valid) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
    }
    if (lease != NULL) {
        memset(lease, 0, sizeof(*lease));
        if (!wf_windows_blocking_probe_lease_is_zero(lease)) {
            InterlockedExchange(&wf_probe_stub_failure, 1);
        }
    }
    InterlockedIncrement(&wf_probe_lease_releases);
}

static uint16_t wf_windows_blocking_probe_path_tag(const char *path) {
    const uint16_t *units = (const uint16_t *)(const void *)path;
    return units == NULL ? 0 : units[0];
}

int *wf__windows_error_location(void) {
    return &wf_probe_error;
}

/* `windows_runtime.c` owns this process-wide order in production.  The probe
 * cannot link that unit because its deliberately injected host operations use
 * the same symbols.  These four local definitions retain the real ticket
 * semantics and expose exact enter/leave counts; the adapter under test still
 * decides when tickets are reserved and crossed. */
uint64_t wf__windows_open_order_reserve(void) {
    uint64_t ticket = atomic_fetch_add_explicit(
        &wf_probe_next_open_ticket,
        1,
        memory_order_seq_cst
    );
    if (ticket == UINT64_MAX) {
        abort();
    }
    return ticket;
}

int wf__windows_open_order_is_serving(uint64_t ticket) {
    int serving;
    AcquireSRWLockShared(&wf_probe_open_order_lock);
    serving = ticket == wf_probe_serving_open_ticket;
    ReleaseSRWLockShared(&wf_probe_open_order_lock);
    if (!serving) {
        InterlockedIncrement(&wf_probe_open_order_rotations);
        if (SetEvent(wf_probe_ticket_b_rotated) == FALSE) {
            InterlockedExchange(&wf_probe_stub_failure, 1);
        }
    }
    return serving;
}

void wf__windows_open_order_enter(uint64_t ticket) {
    BOOL slept = TRUE;
    AcquireSRWLockExclusive(&wf_probe_open_order_lock);
    while (ticket != wf_probe_serving_open_ticket && slept != FALSE) {
        slept = SleepConditionVariableSRW(
            &wf_probe_open_order_ready,
            &wf_probe_open_order_lock,
            INFINITE,
            0
        );
    }
    ReleaseSRWLockExclusive(&wf_probe_open_order_lock);
    if (slept == FALSE) {
        abort();
    }
    InterlockedIncrement(&wf_probe_open_order_enters);
}

void wf__windows_open_order_leave(uint64_t ticket) {
    AcquireSRWLockExclusive(&wf_probe_open_order_lock);
    if (ticket != wf_probe_serving_open_ticket
        || wf_probe_serving_open_ticket == UINT64_MAX) {
        ReleaseSRWLockExclusive(&wf_probe_open_order_lock);
        abort();
    }
    wf_probe_serving_open_ticket += 1u;
    InterlockedIncrement(&wf_probe_open_order_leaves);
    WakeAllConditionVariable(&wf_probe_open_order_ready);
    ReleaseSRWLockExclusive(&wf_probe_open_order_lock);
}

void wf__windows_open_resource_attempt_enter(void) {
    AcquireSRWLockExclusive(&wf_probe_resource_attempt_lock);
    if (InterlockedExchange(&wf_probe_resource_attempt_active, 1) != 0) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
    }
    InterlockedIncrement(&wf_probe_resource_attempt_enters);
}

void wf__windows_open_resource_attempt_leave(void) {
    if (InterlockedExchange(&wf_probe_resource_attempt_active, 0) != 1) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
    }
    InterlockedIncrement(&wf_probe_resource_attempt_leaves);
    ReleaseSRWLockExclusive(&wf_probe_resource_attempt_lock);
}

int64_t wf__windows_completion_file_write_worker(
    HANDLE handle,
    const void *buffer,
    uint64_t count
) {
    LONG sequence;
    DWORD waited;
    wf_windows_blocking_probe_worker_call();
    if (handle != wf_windows_blocking_probe_handle(2)
        || (buffer == NULL && count != 0)
        || count > (uint64_t)INT64_MAX) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        wf_probe_error = ERROR_INVALID_PARAMETER;
        return -1;
    }
    InterlockedIncrement(&wf_probe_write_calls);
    sequence = InterlockedIncrement(&wf_probe_scenario_sequence);
    InterlockedExchange(&wf_probe_write_sequence, sequence);
    if (SetEvent(wf_probe_write_entered) == FALSE) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        return -1;
    }
    waited = WaitForSingleObject(
        wf_probe_write_release,
        WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
    );
    if (waited != WAIT_OBJECT_0) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        wf_probe_error = ERROR_TIMEOUT;
        return -1;
    }
    return (int64_t)count;
}

int64_t wf__windows_completion_directory_next_worker(
    HANDLE directory,
    void *buffer,
    uint64_t count,
    int64_t *position
) {
    wf_windows_blocking_probe_worker_call();
    InterlockedIncrement(&wf_probe_directory_calls);
    if (directory != wf_windows_blocking_probe_handle(3)
        || buffer == NULL || count == 0 || position == NULL) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        wf_probe_error = ERROR_INVALID_PARAMETER;
        return -1;
    }
    ((unsigned char *)buffer)[0] = (unsigned char)'D';
    *position += 1;
    return 1;
}

int wf__windows_completion_file_open_at_worker(
    HANDLE root,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    unsigned descriptor_class,
    int *error_code,
    unsigned *open_outcome,
    unsigned *took_resources,
    unsigned *returned_resources,
    unsigned *refused_resource,
    unsigned awarded_resource,
    int on_an_award
) {
    uint16_t tag = wf_windows_blocking_probe_path_tag(path);
    (void)awarded_resource;
    (void)on_an_award;
    wf_windows_blocking_probe_worker_call();
    if (error_code == NULL || open_outcome == NULL
        || took_resources == NULL || returned_resources == NULL
        || refused_resource == NULL) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        return -1;
    }
    if (root != wf_windows_blocking_probe_handle(1)
        || flags != 0 || mode != 0 || has_mode != 0
        || (tag == (uint16_t)'R'
            && (expected_kind != 1u
                || descriptor_class
                    != WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE))
        || (tag == (uint16_t)'A'
            && (expected_kind != 2u
                || descriptor_class
                    != WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT))
        || (tag == (uint16_t)'B'
            && (expected_kind != 2u
                || descriptor_class
                    != WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE))) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        *error_code = ERROR_INVALID_PARAMETER;
        *open_outcome = 1u;
        return -1;
    }
    *error_code = 0;
    *open_outcome = 0;
    *took_resources = 0;
    *returned_resources = 0;
    *refused_resource = WF_WINDOWS_OPEN_REFUSED_NONE;
    /* Model the production worker's short NtCreateFile/take boundary. Any
     * injected refusal or ticket wait below must execute after it is released. */
    wf__windows_open_resource_attempt_enter();
    if (wf_windows_blocking_probe_load(
            &wf_probe_resource_attempt_active
        ) != 1) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
    }
    wf__windows_open_resource_attempt_leave();
    if (tag == (uint16_t)'R') {
        LONG call = InterlockedIncrement(&wf_probe_retry_open_calls);
        LONG sequence = InterlockedIncrement(&wf_probe_scenario_sequence);
        if (call == 1) {
            DWORD waited;
            InterlockedExchange(&wf_probe_refusal_sequence, sequence);
            *error_code = ERROR_TOO_MANY_OPEN_FILES;
            *open_outcome = 1;
            *refused_resource = WF_WINDOWS_OPEN_REFUSED_GENERAL_RESOURCE;
            if (SetEvent(wf_probe_refusal_seen) == FALSE) {
                InterlockedExchange(&wf_probe_stub_failure, 1);
            }
            waited = WaitForSingleObject(
                wf_probe_refusal_release,
                WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
            );
            if (waited != WAIT_OBJECT_0) {
                InterlockedExchange(&wf_probe_stub_failure, 1);
            }
            return -1;
        }
        InterlockedExchange(&wf_probe_retry_sequence, sequence);
        return 701;
    }
    if (tag == (uint16_t)'A') {
        DWORD waited;
        InterlockedIncrement(&wf_probe_ticket_a_calls);
        InterlockedExchange(
            &wf_probe_ticket_a_sequence,
            InterlockedIncrement(&wf_probe_ticket_sequence)
        );
        if (SetEvent(wf_probe_ticket_a_entered) == FALSE) {
            InterlockedExchange(&wf_probe_stub_failure, 1);
            return -1;
        }
        waited = WaitForSingleObject(
            wf_probe_ticket_release,
            WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
        );
        if (waited != WAIT_OBJECT_0) {
            InterlockedExchange(&wf_probe_stub_failure, 1);
            *error_code = ERROR_TIMEOUT;
            *open_outcome = 1;
            return -1;
        }
        return 702;
    }
    if (tag == (uint16_t)'B') {
        InterlockedIncrement(&wf_probe_ticket_b_calls);
        InterlockedExchange(
            &wf_probe_ticket_b_sequence,
            InterlockedIncrement(&wf_probe_ticket_sequence)
        );
        return 703;
    }
    InterlockedExchange(&wf_probe_stub_failure, 1);
    *error_code = ERROR_INVALID_NAME;
    *open_outcome = 1;
    return -1;
}

static int wf_windows_blocking_probe_submit(
    wf_completion_runtime *runtime,
    wf_windows_blocking_adapter *adapter,
    const wf_windows_blocking_request *request,
    wf_completion_token *token
) {
    return wf_completion_claim(runtime, token) == WF_COMPLETION_CLAIMED
            && wf_windows_blocking_submit(adapter, *token, request)
                == WF_WINDOWS_BLOCKING_TARGET_OWNS
        ? 0
        : 1;
}

static int wf_windows_blocking_probe_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    enum wf_windows_file_operation_kind kind,
    int64_t expected_value,
    wf_windows_file_result *result
) {
    ULONGLONG deadline = GetTickCount64()
        + WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS;
    wf_completion_outcome outcome;
    for (;;) {
        wf_completion_event events[WF_WINDOWS_BLOCKING_PROBE_SLOTS];
        enum wf_completion_consume_result consumed;
        (void)wf_completion_drain(
            runtime,
            events,
            WF_WINDOWS_BLOCKING_PROBE_SLOTS,
            runtime->slot_count
        );
        memset(result, 0, sizeof(*result));
        memset(&outcome, 0, sizeof(outcome));
        consumed = wf_completion_consume(
            runtime,
            token,
            result,
            sizeof(*result),
            &outcome
        );
        if (consumed == WF_COMPLETION_CONSUMED) {
            return result->kind == kind && result->value == expected_value
                    && result->error_code == 0
                    && outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE
                    && outcome.terminal_kind == 1
                    && outcome.result_size == sizeof(*result)
                ? 0
                : 1;
        }
        if (consumed != WF_COMPLETION_CONSUME_NOT_TERMINAL
            && consumed != WF_COMPLETION_CONSUME_NOT_DRAINED) {
            return 1;
        }
        if (GetTickCount64() >= deadline) {
            return 1;
        }
        Sleep(1);
    }
}

static int wf_windows_blocking_probe_wait_for_retirement_waiter(void) {
    ULONGLONG deadline = GetTickCount64()
        + WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS;
    while (wf_completion_retirement_waiters() != 1) {
        if (GetTickCount64() >= deadline) {
            return 1;
        }
        Sleep(1);
    }
    return 0;
}

int main(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[WF_WINDOWS_BLOCKING_PROBE_SLOTS];
    static wf_windows_blocking_adapter adapter;
    wf_windows_blocking_request request;
    wf_completion_token retry_open;
    wf_completion_token owed_write;
    wf_completion_token directory;
    wf_completion_token ticket_a;
    wf_completion_token ticket_b;
    wf_windows_file_result result;
    wf_windows_blocking_statistics blocking_statistics;
    wf_completion_statistics completion_statistics;
    static const uint16_t retry_path[] = {'R', 0};
    static const uint16_t ticket_a_path[] = {'A', 0};
    static const uint16_t ticket_b_path[] = {'B', 0};
    static const unsigned char write_byte = (unsigned char)'W';
    unsigned char directory_byte = 0;
    int64_t directory_position = 0;

    wf_probe_main_thread = GetCurrentThreadId();
    wf_probe_refusal_seen = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_refusal_release = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_write_entered = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_write_release = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_ticket_a_entered = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_ticket_b_rotated = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_ticket_release = CreateEventW(NULL, TRUE, FALSE, NULL);
    PROBE_CHECK(
        wf_probe_refusal_seen != NULL && wf_probe_refusal_release != NULL
            && wf_probe_write_entered != NULL
            && wf_probe_write_release != NULL
            && wf_probe_ticket_a_entered != NULL
            && wf_probe_ticket_b_rotated != NULL
            && wf_probe_ticket_release != NULL,
        "could not create synchronization events"
    );
    PROBE_CHECK(
        SetEnvironmentVariableW(L"WF_WINDOWS_BLOCKING_WORKERS", L"2")
            != FALSE,
        "could not pin two blocking workers"
    );
    PROBE_CHECK(
        wf_completion_runtime_init(
            &runtime,
            slots,
            WF_WINDOWS_BLOCKING_PROBE_SLOTS
        ) == 0,
        "completion core initialization failed"
    );
    wf_completion_retirement_announces_on(&runtime);
    PROBE_CHECK(
        wf_windows_blocking_init(&adapter, &runtime) == 0,
        "blocking adapter initialization failed"
    );

    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_OPEN_AT;
    request.operation.open_at.directory = 1;
    request.operation.open_at.path = (const char *)(const void *)retry_path;
    request.operation.open_at.path_units = 1;
    request.operation.open_at.expected_kind = 1;
    request.operation.open_at.descriptor_class =
        WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE;
    request.lease = wf_windows_blocking_probe_lease(
        1,
        WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT,
        WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_submit(
            &runtime,
            &adapter,
            &request,
            &retry_open
        ) == 0,
        "resource-refused open was not accepted"
    );
    PROBE_CHECK(
        WaitForSingleObject(
            wf_probe_refusal_seen,
            WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
        ) == WAIT_OBJECT_0,
        "the first open attempt did not reach the worker"
    );

    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_WRITE;
    request.operation.write.descriptor = 2;
    request.operation.write.buffer = &write_byte;
    request.operation.write.count = 1;
    request.lease = wf_windows_blocking_probe_lease(
        2,
        WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT,
        WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_submit(
            &runtime,
            &adapter,
            &request,
            &owed_write
        ) == 0,
        "owed write was not accepted"
    );
    PROBE_CHECK(
        WaitForSingleObject(
            wf_probe_write_entered,
            WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
        ) == WAIT_OBJECT_0,
        "owed write did not enter a worker"
    );
    PROBE_CHECK(
        SetEvent(wf_probe_refusal_release) != FALSE,
        "could not release the resource-refused host call"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_wait_for_retirement_waiter() == 0,
        "the resource-refused open did not register its retirement waiter"
    );
    PROBE_CHECK(
        SetEvent(wf_probe_write_release) != FALSE,
        "could not retire the owed write"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            owed_write,
            WF_WINDOWS_FILE_WRITE,
            1,
            &result
        ) == 0,
        "owed write did not publish and consume"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[owed_write.slot].request.lease
        ),
        "owed write lease was not cleared before publication"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            retry_open,
            WF_WINDOWS_FILE_OPEN_AT,
            701,
            &result
        ) == 0,
        "resource-refused open did not retry successfully"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[retry_open.slot].request.lease
        ),
        "retried open lease was not cleared before publication"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_load(&wf_probe_refusal_sequence) == 1
            && wf_windows_blocking_probe_load(&wf_probe_write_sequence) == 2
            && wf_windows_blocking_probe_load(&wf_probe_retry_sequence) == 3
            && wf_windows_blocking_probe_load(&wf_probe_retry_open_calls) == 2,
        "refusal, owed retirement, and the one retry were not ordered exactly"
    );

    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_DIRECTORY_NEXT;
    request.operation.directory_next.descriptor = 3;
    request.operation.directory_next.buffer = &directory_byte;
    request.operation.directory_next.count = 1;
    request.operation.directory_next.position = &directory_position;
    request.lease = wf_windows_blocking_probe_lease(
        3,
        WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE,
        WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_submit(
            &runtime,
            &adapter,
            &request,
            &directory
        ) == 0,
        "directory job was not accepted"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            directory,
            WF_WINDOWS_FILE_DIRECTORY_NEXT,
            1,
            &result
        ) == 0
            && directory_byte == (unsigned char)'D'
            && directory_position == 1,
        "directory job did not execute on the blocking worker"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[directory.slot].request.lease
        ),
        "directory lease was not cleared before publication"
    );

    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_OPEN_AT;
    request.operation.open_at.directory = 1;
    request.operation.open_at.path =
        (const char *)(const void *)ticket_a_path;
    request.operation.open_at.path_units = 1;
    request.operation.open_at.expected_kind = 2;
    request.operation.open_at.descriptor_class =
        WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT;
    request.lease = wf_windows_blocking_probe_lease(
        1,
        WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT,
        WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_submit(
            &runtime,
            &adapter,
            &request,
            &ticket_a
        ) == 0,
        "first ordered open was not accepted"
    );
    PROBE_CHECK(
        WaitForSingleObject(
            wf_probe_ticket_a_entered,
            WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
        ) == WAIT_OBJECT_0,
        "first ordered open did not enter its worker"
    );
    request.operation.open_at.path =
        (const char *)(const void *)ticket_b_path;
    request.operation.open_at.descriptor_class =
        WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE;
    request.lease = wf_windows_blocking_probe_lease(
        1,
        WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT,
        WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_submit(
            &runtime,
            &adapter,
            &request,
            &ticket_b
        ) == 0,
        "second ordered open was not accepted"
    );
    PROBE_CHECK(
        WaitForSingleObject(
            wf_probe_ticket_b_rotated,
            WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
        ) == WAIT_OBJECT_0
            && atomic_load_explicit(
                   &adapter.entries[ticket_b.slot].state,
                   memory_order_acquire
               ) == WF_WINDOWS_BLOCKING_ENTRY_QUEUED,
        "second ordered open was not rotated behind the serving ticket"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_load(&wf_probe_ticket_b_calls) == 0,
        "the second open crossed the first open's ticket"
    );
    PROBE_CHECK(
        SetEvent(wf_probe_ticket_release) != FALSE,
        "could not release the first ordered open"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            ticket_a,
            WF_WINDOWS_FILE_OPEN_AT,
            702,
            &result
        ) == 0,
        "first ordered open did not publish and consume"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[ticket_a.slot].request.lease
        ),
        "first ordered open lease was not cleared before publication"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            ticket_b,
            WF_WINDOWS_FILE_OPEN_AT,
            703,
            &result
        ) == 0,
        "second ordered open did not publish and consume"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[ticket_b.slot].request.lease
        ),
        "second ordered open lease was not cleared before publication"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_load(&wf_probe_ticket_a_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_ticket_b_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_ticket_a_sequence) == 1
            && wf_windows_blocking_probe_load(&wf_probe_ticket_b_sequence) == 2,
        "open ticket execution order did not match submission order"
    );

    blocking_statistics = wf_windows_blocking_statistics_snapshot(&adapter);
    completion_statistics = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        blocking_statistics.worker_count == 2
            && blocking_statistics.submissions
                == WF_WINDOWS_BLOCKING_PROBE_JOBS
            && blocking_statistics.executions
                == WF_WINDOWS_BLOCKING_PROBE_JOBS
            && blocking_statistics.publications
                == WF_WINDOWS_BLOCKING_PROBE_JOBS
            && blocking_statistics.exhaustion_retries == 1
            && blocking_statistics.queue_capacity_failures == 0
            && blocking_statistics.in_flight == 0,
        "blocking worker statistics did not close"
    );
    PROBE_CHECK(
        completion_statistics.publications
                == WF_WINDOWS_BLOCKING_PROBE_JOBS
            && completion_statistics.consumptions
                == WF_WINDOWS_BLOCKING_PROBE_JOBS,
        "completion publications and consumes did not close"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_load(&wf_probe_write_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_directory_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_open_order_enters) == 3
            && wf_windows_blocking_probe_load(&wf_probe_open_order_leaves) == 3
            && wf_windows_blocking_probe_load(
                   &wf_probe_open_order_rotations
               ) > 0
            && wf_windows_blocking_probe_load(&wf_probe_lease_releases)
                == WF_WINDOWS_BLOCKING_PROBE_JOBS
            && wf_windows_blocking_probe_load(
                   &wf_probe_root_lease_releases
               ) == 3
            && wf_windows_blocking_probe_load(
                   &wf_probe_output_lease_releases
               ) == 1
            && wf_windows_blocking_probe_load(
                   &wf_probe_directory_lease_releases
               ) == 1
            && wf_windows_blocking_probe_load(
                   &wf_probe_resource_attempt_enters
               ) == 4
            && wf_windows_blocking_probe_load(
                   &wf_probe_resource_attempt_leaves
               ) == 4
            && wf_windows_blocking_probe_load(
                   &wf_probe_resource_attempt_active
               ) == 0
            && wf_windows_blocking_probe_load(&wf_probe_wrong_thread) == 0
            && wf_windows_blocking_probe_load(&wf_probe_stub_failure) == 0,
        "a host job did not execute exactly once on a worker"
    );
    PROBE_CHECK(
        wf_completion_retirement_waiters() == 0,
        "the resource-refusal waiter leaked"
    );
    PROBE_CHECK(
        wf_windows_blocking_shutdown(&adapter) == 0,
        "blocking adapter shutdown did not close"
    );
    PROBE_CHECK(
        wf_completion_runtime_destroy(&runtime) == 0,
        "completion core shutdown did not close"
    );
    PROBE_CHECK(
        CloseHandle(wf_probe_refusal_seen) != FALSE
            && CloseHandle(wf_probe_refusal_release) != FALSE
            && CloseHandle(wf_probe_write_entered) != FALSE
            && CloseHandle(wf_probe_write_release) != FALSE
            && CloseHandle(wf_probe_ticket_a_entered) != FALSE
            && CloseHandle(wf_probe_ticket_b_rotated) != FALSE
            && CloseHandle(wf_probe_ticket_release) != FALSE,
        "probe synchronization handles did not close"
    );
    printf(
        "windows-blocking-probe status=pass workers=%llu submissions=%llu "
        "executions=%llu publications=%llu consumes=%llu retries=%llu "
        "leases=%llu attempts=%llu\n",
        (unsigned long long)blocking_statistics.worker_count,
        (unsigned long long)blocking_statistics.submissions,
        (unsigned long long)blocking_statistics.executions,
        (unsigned long long)blocking_statistics.publications,
        (unsigned long long)completion_statistics.consumptions,
        (unsigned long long)blocking_statistics.exhaustion_retries,
        (unsigned long long)wf_windows_blocking_probe_load(
            &wf_probe_lease_releases
        ),
        (unsigned long long)wf_windows_blocking_probe_load(
            &wf_probe_resource_attempt_enters
        )
    );
    return 0;
}

#else

typedef int wf_windows_blocking_probe_translation_unit_is_target_guarded;

#endif
