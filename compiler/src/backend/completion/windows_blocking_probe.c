/* Native Windows proof for the bounded blocking adapter.
 *
 * The production adapter and completion core are linked unchanged.  The host
 * operations and descriptor-lease release are deterministic probe stubs;
 * every scheduler, queue, publication, and consume transition remains
 * production code.  An open the host refuses is published as the refusal it
 * is, exactly like any other host outcome, so the probe scripts no refusal:
 * the descriptor an open needs is owned by its `FilePermit` [SYS-10] before
 * the open is ever submitted. */

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
#define WF_WINDOWS_BLOCKING_PROBE_JOBS 3u

#define PROBE_CHECK(expression, message)                                     \
    do {                                                                      \
        if (!(expression)) {                                                  \
            return wf_windows_blocking_probe_fail(message);                   \
        }                                                                     \
    } while (0)

static DWORD wf_probe_main_thread;
static HANDLE wf_probe_write_entered;
static HANDLE wf_probe_write_release;
static volatile LONG wf_probe_wrong_thread;
static volatile LONG wf_probe_open_calls;
static volatile LONG wf_probe_write_calls;
static volatile LONG wf_probe_directory_calls;
static volatile LONG wf_probe_stub_failure;
static volatile LONG wf_probe_lease_releases;
static volatile LONG wf_probe_root_lease_releases;
static volatile LONG wf_probe_output_lease_releases;
static volatile LONG wf_probe_directory_lease_releases;
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

/* The write parks inside its host call until the probe releases it, which is
 * what proves the job is genuinely running on a worker while the probe goes
 * on submitting. */
int64_t wf__windows_completion_file_write_worker(
    HANDLE handle,
    const void *buffer,
    uint64_t count
) {
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
    unsigned *open_outcome
) {
    uint16_t tag = wf_windows_blocking_probe_path_tag(path);
    wf_windows_blocking_probe_worker_call();
    if (error_code == NULL || open_outcome == NULL) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        return -1;
    }
    InterlockedIncrement(&wf_probe_open_calls);
    if (root != wf_windows_blocking_probe_handle(1)
        || flags != 0 || mode != 0 || has_mode != 0
        || tag != (uint16_t)'R'
        || expected_kind != 1u
        || descriptor_class != WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE) {
        InterlockedExchange(&wf_probe_stub_failure, 1);
        *error_code = ERROR_INVALID_PARAMETER;
        *open_outcome = 1u;
        return -1;
    }
    *error_code = 0;
    *open_outcome = 0;
    return 701;
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

int main(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[WF_WINDOWS_BLOCKING_PROBE_SLOTS];
    static wf_windows_blocking_adapter adapter;
    wf_windows_blocking_request request;
    wf_completion_token open;
    wf_completion_token write;
    wf_completion_token directory;
    wf_windows_file_result result;
    wf_windows_blocking_statistics blocking_statistics;
    wf_completion_statistics completion_statistics;
    static const uint16_t open_path[] = {'R', 0};
    static const unsigned char write_byte = (unsigned char)'W';
    unsigned char directory_byte = 0;
    int64_t directory_position = 0;

    wf_probe_main_thread = GetCurrentThreadId();
    wf_probe_write_entered = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_write_release = CreateEventW(NULL, TRUE, FALSE, NULL);
    PROBE_CHECK(
        wf_probe_write_entered != NULL && wf_probe_write_release != NULL,
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
    PROBE_CHECK(
        wf_windows_blocking_init(&adapter, &runtime) == 0,
        "blocking adapter initialization failed"
    );

    /* The write is submitted first and parks on its worker, so the open and
     * the directory job that follow run beside a job still in flight. */
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
            &write
        ) == 0,
        "write was not accepted"
    );
    PROBE_CHECK(
        WaitForSingleObject(
            wf_probe_write_entered,
            WF_WINDOWS_BLOCKING_PROBE_TIMEOUT_MS
        ) == WAIT_OBJECT_0,
        "write did not enter a worker"
    );

    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_OPEN_AT;
    request.operation.open_at.directory = 1;
    request.operation.open_at.path = (const char *)(const void *)open_path;
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
            &open
        ) == 0,
        "open was not accepted"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            open,
            WF_WINDOWS_FILE_OPEN_AT,
            701,
            &result
        ) == 0,
        "open did not publish and consume beside the parked write"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[open.slot].request.lease
        ),
        "open lease was not cleared before publication"
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

    PROBE_CHECK(
        SetEvent(wf_probe_write_release) != FALSE,
        "could not release the parked write"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_consume(
            &runtime,
            write,
            WF_WINDOWS_FILE_WRITE,
            1,
            &result
        ) == 0,
        "write did not publish and consume"
    );
    PROBE_CHECK(
        wf_windows_blocking_probe_lease_is_zero(
            &adapter.entries[write.slot].request.lease
        ),
        "write lease was not cleared before publication"
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
        wf_windows_blocking_probe_load(&wf_probe_open_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_write_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_directory_calls) == 1
            && wf_windows_blocking_probe_load(&wf_probe_lease_releases)
                == WF_WINDOWS_BLOCKING_PROBE_JOBS
            && wf_windows_blocking_probe_load(
                   &wf_probe_root_lease_releases
               ) == 1
            && wf_windows_blocking_probe_load(
                   &wf_probe_output_lease_releases
               ) == 1
            && wf_windows_blocking_probe_load(
                   &wf_probe_directory_lease_releases
               ) == 1
            && wf_windows_blocking_probe_load(&wf_probe_wrong_thread) == 0
            && wf_windows_blocking_probe_load(&wf_probe_stub_failure) == 0,
        "a host job did not execute exactly once on a worker"
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
        CloseHandle(wf_probe_write_entered) != FALSE
            && CloseHandle(wf_probe_write_release) != FALSE,
        "probe synchronization handles did not close"
    );
    printf(
        "windows-blocking-probe status=pass workers=%llu submissions=%llu "
        "executions=%llu publications=%llu consumes=%llu leases=%llu\n",
        (unsigned long long)blocking_statistics.worker_count,
        (unsigned long long)blocking_statistics.submissions,
        (unsigned long long)blocking_statistics.executions,
        (unsigned long long)blocking_statistics.publications,
        (unsigned long long)completion_statistics.consumptions,
        (unsigned long long)wf_windows_blocking_probe_load(
            &wf_probe_lease_releases
        )
    );
    return 0;
}

#else

int main(void) {
    return 0;
}

#endif
