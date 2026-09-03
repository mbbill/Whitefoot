/* Native Windows proof that a host resource release and its retirement-ledger
 * publication cannot be interleaved with a new open attempt.
 *
 * The production runtime is compiled with three observation points. The close
 * thread stops after _close has released its HANDLE and CRT descriptor but
 * before either return is counted. A concurrent direct open must reach the
 * pre-lock point and remain unable to reach the acquired-lock point. Once the
 * close is released, the open observes both counters advanced before its host
 * attempt and consumes both credits. A synthetic waiter then proves neither
 * return remains awardable a second time. */

#if defined(_WIN32)

#include "windows_completion.h"
#include "windows_runtime.h"

#include <stdint.h>
#include <stdio.h>
#include <windows.h>

#define WF_WINDOWS_RESOURCE_PROBE_TIMEOUT_MS 10000u

typedef struct wf_windows_resource_probe_close {
    int descriptor;
    int result;
} wf_windows_resource_probe_close;

typedef struct wf_windows_resource_probe_open {
    int directory;
    const WCHAR *path;
    int descriptor;
    int error_code;
    unsigned outcome;
} wf_windows_resource_probe_open;

static HANDLE wf_probe_close_released;
static HANDLE wf_probe_allow_close_accounting;
static HANDLE wf_probe_open_waiting;
static HANDLE wf_probe_open_acquired;
static volatile LONG wf_probe_enabled;
static volatile LONG wf_probe_open_observation_enabled;
static volatile LONG wf_probe_failure;
static volatile LONG wf_probe_close_points;
static volatile LONG wf_probe_open_wait_points;
static volatile LONG wf_probe_open_points;
static uint64_t wf_probe_open_native_returns;
static uint64_t wf_probe_open_crt_returns;

static int wf_windows_resource_probe_fail(const char *message) {
    fprintf(stderr, "windows resource attempt probe: %s\n", message);
    return 1;
}

static int wf_windows_resource_probe_signal(HANDLE event) {
    if (event == NULL || SetEvent(event) == FALSE) {
        InterlockedExchange(&wf_probe_failure, 1);
        return 0;
    }
    return 1;
}

void wf_windows_resource_probe_close_release(void) {
    DWORD waited;
    if (InterlockedCompareExchange(&wf_probe_enabled, 0, 0) == 0) {
        return;
    }
    InterlockedIncrement(&wf_probe_close_points);
    InterlockedExchange(&wf_probe_open_observation_enabled, 1);
    if (!wf_windows_resource_probe_signal(wf_probe_close_released)) {
        return;
    }
    waited = WaitForSingleObject(
        wf_probe_allow_close_accounting,
        WF_WINDOWS_RESOURCE_PROBE_TIMEOUT_MS
    );
    if (waited != WAIT_OBJECT_0) {
        InterlockedExchange(&wf_probe_failure, 1);
    }
}

void wf_windows_resource_probe_open_wait(void) {
    if (InterlockedCompareExchange(
            &wf_probe_open_observation_enabled,
            0,
            0
        ) == 0) {
        return;
    }
    InterlockedIncrement(&wf_probe_open_wait_points);
    (void)wf_windows_resource_probe_signal(wf_probe_open_waiting);
}

void wf_windows_resource_probe_open_acquired(void) {
    if (InterlockedCompareExchange(
            &wf_probe_open_observation_enabled,
            0,
            0
        ) == 0) {
        return;
    }
    wf_probe_open_native_returns = wf_completion_resource_returns(
        WF_RETIREMENT_NATIVE_HANDLE
    );
    wf_probe_open_crt_returns = wf_completion_resource_returns(
        WF_RETIREMENT_CRT_DESCRIPTOR
    );
    InterlockedIncrement(&wf_probe_open_points);
    (void)wf_windows_resource_probe_signal(wf_probe_open_acquired);
}

static DWORD WINAPI wf_windows_resource_probe_close_main(void *opaque) {
    wf_windows_resource_probe_close *close =
        (wf_windows_resource_probe_close *)opaque;
    close->result = wf__completion_file_close_direct(close->descriptor);
    return 0;
}

static DWORD WINAPI wf_windows_resource_probe_open_main(void *opaque) {
    wf_windows_resource_probe_open *open =
        (wf_windows_resource_probe_open *)opaque;
    open->descriptor = wf__completion_file_open_at_direct(
        open->directory,
        (const char *)(const void *)open->path,
        0,
        0,
        0,
        1,
        WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
        &open->error_code,
        &open->outcome
    );
    return 0;
}

static int wf_windows_resource_probe_join(HANDLE thread) {
    DWORD exit_code = 1;
    return thread != NULL
        && WaitForSingleObject(
               thread,
               WF_WINDOWS_RESOURCE_PROBE_TIMEOUT_MS
           ) == WAIT_OBJECT_0
        && GetExitCodeThread(thread, &exit_code) != FALSE
        && exit_code == 0;
}

static int wf_windows_resource_probe_credit_consumed(
    unsigned resource,
    uint64_t seen
) {
    wf_retirement_waiter waiter;
    enum wf_retirement_state state;
    wf_completion_retirement_wait_begin_resource(
        &waiter,
        seen,
        NULL,
        NULL,
        0,
        resource
    );
    state = wf_completion_retirement_state(&waiter);
    wf_completion_retirement_wait_end(&waiter);
    return state == WF_RETIREMENT_UNREACHABLE;
}

int main(void) {
    static const WCHAR fixture_name[] = L"wf-resource-attempt.data";
    HANDLE fixture;
    HANDLE close_thread;
    HANDLE open_thread;
    wf_windows_resource_probe_close close_context;
    wf_windows_resource_probe_open open_context;
    uint64_t native_before;
    uint64_t crt_before;
    int directory;
    int descriptor;
    int error_code = 0;
    unsigned outcome = 1;

    wf_probe_close_released = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_allow_close_accounting = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_open_waiting = CreateEventW(NULL, TRUE, FALSE, NULL);
    wf_probe_open_acquired = CreateEventW(NULL, TRUE, FALSE, NULL);
    if (wf_probe_close_released == NULL
        || wf_probe_allow_close_accounting == NULL
        || wf_probe_open_waiting == NULL
        || wf_probe_open_acquired == NULL) {
        return wf_windows_resource_probe_fail("could not create events");
    }

    fixture = CreateFileW(
        fixture_name,
        GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        NULL,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );
    if (fixture == INVALID_HANDLE_VALUE || CloseHandle(fixture) == FALSE) {
        return wf_windows_resource_probe_fail("could not create fixture");
    }
    directory = wf__windows_open_cwd(NULL, 0);
    if (directory < 0) {
        return wf_windows_resource_probe_fail("could not open cwd");
    }
    descriptor = wf__completion_file_open_at_direct(
        directory,
        (const char *)(const void *)fixture_name,
        0,
        0,
        0,
        1,
        WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
        &error_code,
        &outcome
    );
    if (descriptor < 0 || error_code != 0 || outcome != 0) {
        return wf_windows_resource_probe_fail("could not open fixture");
    }

    native_before = wf_completion_resource_returns(
        WF_RETIREMENT_NATIVE_HANDLE
    );
    crt_before = wf_completion_resource_returns(
        WF_RETIREMENT_CRT_DESCRIPTOR
    );
    close_context.descriptor = descriptor;
    close_context.result = -1;
    open_context.directory = directory;
    open_context.path = fixture_name;
    open_context.descriptor = -1;
    open_context.error_code = -1;
    open_context.outcome = 1;
    InterlockedExchange(&wf_probe_enabled, 1);

    close_thread = CreateThread(
        NULL,
        0,
        wf_windows_resource_probe_close_main,
        &close_context,
        0,
        NULL
    );
    if (close_thread == NULL
        || WaitForSingleObject(
               wf_probe_close_released,
               WF_WINDOWS_RESOURCE_PROBE_TIMEOUT_MS
           ) != WAIT_OBJECT_0) {
        return wf_windows_resource_probe_fail(
            "close did not reach the release/accounting boundary"
        );
    }
    if (wf_completion_resource_returns(WF_RETIREMENT_NATIVE_HANDLE)
            != native_before
        || wf_completion_resource_returns(WF_RETIREMENT_CRT_DESCRIPTOR)
            != crt_before) {
        return wf_windows_resource_probe_fail(
            "close counted a return before the injected boundary"
        );
    }

    open_thread = CreateThread(
        NULL,
        0,
        wf_windows_resource_probe_open_main,
        &open_context,
        0,
        NULL
    );
    if (open_thread == NULL
        || WaitForSingleObject(
               wf_probe_open_waiting,
               WF_WINDOWS_RESOURCE_PROBE_TIMEOUT_MS
           ) != WAIT_OBJECT_0) {
        return wf_windows_resource_probe_fail(
            "open did not reach the resource-attempt boundary"
        );
    }
    if (WaitForSingleObject(wf_probe_open_acquired, 500) != WAIT_TIMEOUT) {
        return wf_windows_resource_probe_fail(
            "open crossed a close release before its return was counted"
        );
    }
    if (!wf_windows_resource_probe_signal(wf_probe_allow_close_accounting)
        || WaitForSingleObject(
               wf_probe_open_acquired,
               WF_WINDOWS_RESOURCE_PROBE_TIMEOUT_MS
           ) != WAIT_OBJECT_0
        || !wf_windows_resource_probe_join(close_thread)
        || !wf_windows_resource_probe_join(open_thread)) {
        return wf_windows_resource_probe_fail(
            "close and open did not finish after accounting"
        );
    }
    InterlockedExchange(&wf_probe_enabled, 0);
    InterlockedExchange(&wf_probe_open_observation_enabled, 0);
    (void)CloseHandle(close_thread);
    (void)CloseHandle(open_thread);

    if (close_context.result != 0 || open_context.descriptor < 0
        || open_context.error_code != 0 || open_context.outcome != 0
        || wf_probe_open_native_returns != native_before + 1u
        || wf_probe_open_crt_returns != crt_before + 1u
        || InterlockedCompareExchange(&wf_probe_close_points, 0, 0) != 1
        || InterlockedCompareExchange(&wf_probe_open_wait_points, 0, 0) != 1
        || InterlockedCompareExchange(&wf_probe_open_points, 0, 0) != 1
        || InterlockedCompareExchange(&wf_probe_failure, 0, 0) != 0) {
        return wf_windows_resource_probe_fail(
            "the serialized release/open observations were not exact"
        );
    }
    if (!wf_windows_resource_probe_credit_consumed(
            WF_RETIREMENT_NATIVE_HANDLE,
            native_before
        )
        || !wf_windows_resource_probe_credit_consumed(
            WF_RETIREMENT_CRT_DESCRIPTOR,
            crt_before
        )) {
        return wf_windows_resource_probe_fail(
            "a resource taken by the open remained awardable"
        );
    }
    if (wf__windows_completion_active_descriptor_leases() != 0) {
        return wf_windows_resource_probe_fail("a descriptor lease leaked");
    }

    if (wf__completion_file_close_direct(open_context.descriptor) != 0
        || wf__completion_file_close_direct(directory) != 0
        || DeleteFileW(fixture_name) == FALSE) {
        return wf_windows_resource_probe_fail("cleanup did not close");
    }
    (void)CloseHandle(wf_probe_close_released);
    (void)CloseHandle(wf_probe_allow_close_accounting);
    (void)CloseHandle(wf_probe_open_waiting);
    (void)CloseHandle(wf_probe_open_acquired);
    printf(
        "windows-resource-attempt-probe status=pass blocked=1 "
        "native-credit=consumed crt-credit=consumed leases=0\n"
    );
    return 0;
}

#else

typedef int wf_windows_resource_attempt_probe_is_target_guarded;

#endif
