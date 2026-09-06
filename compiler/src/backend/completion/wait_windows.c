#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

/*
 * Windows's one wait set: an SRWLOCK and a CONDITION_VARIABLE.
 *
 * This is `wait_host.c`'s twin and the whole of what `runtime.c` cannot write
 * once for both platforms (`contract.h`, "the wait").  Nothing here knows
 * about the epoch, the announcement or the park protocol.
 *
 * A completion link on Windows sleeps on the completion port rather than on
 * this condition variable, because the bridge supplies the one park (design
 * §7, platform item 2).  This is still needed there: it is the lock a
 * publisher and a sleeper announce themselves under, and it is the park a run
 * with no ring at all takes (`WF_IO_NO_NATIVE_RING`).
 */

#include "contract.h"

#include <windows.h>

#include <stdint.h>
#include <string.h>

typedef struct wf_completion_wait_windows {
    SRWLOCK lock;
    CONDITION_VARIABLE condition;
} wf_completion_wait_windows;

_Static_assert(
    sizeof(wf_completion_wait_windows) <= sizeof(wf_completion_wait),
    "the Windows wait set must fit the block the shared contract reserves"
);
_Static_assert(
    _Alignof(wf_completion_wait_windows) <= _Alignof(wf_completion_wait),
    "the Windows wait set must not out-align the block the contract reserves"
);

static wf_completion_wait_windows *wf_completion_wait_of(
    wf_completion_wait *wait
) {
    return (wf_completion_wait_windows *)(void *)wait;
}

int wf_completion_wait_init(wf_completion_wait *wait) {
    wf_completion_wait_windows *windows;
    if (wait == NULL) {
        return ERROR_INVALID_PARAMETER;
    }
    memset(wait, 0, sizeof(*wait));
    windows = wf_completion_wait_of(wait);
    InitializeSRWLock(&windows->lock);
    InitializeConditionVariable(&windows->condition);
    return 0;
}

/* Neither object owns a kernel resource until it is contended, and neither
 * has a destroy call: Windows reclaims both with their storage. */
int wf_completion_wait_destroy(wf_completion_wait *wait) {
    if (wait == NULL) {
        return ERROR_INVALID_PARAMETER;
    }
    return 0;
}

void wf_completion_wait_lock(wf_completion_wait *wait) {
    AcquireSRWLockExclusive(&wf_completion_wait_of(wait)->lock);
}

void wf_completion_wait_unlock(wf_completion_wait *wait) {
    ReleaseSRWLockExclusive(&wf_completion_wait_of(wait)->lock);
}

enum wf_completion_wait_result wf_completion_wait_sleep(
    wf_completion_wait *wait,
    uint32_t timeout_milliseconds
) {
    wf_completion_wait_windows *windows = wf_completion_wait_of(wait);
    DWORD milliseconds = timeout_milliseconds == UINT32_MAX
        ? INFINITE
        : (DWORD)timeout_milliseconds;
    if (SleepConditionVariableSRW(
            &windows->condition,
            &windows->lock,
            milliseconds,
            0
        ) != FALSE) {
        return WF_COMPLETION_WAIT_WOKEN;
    }
    return GetLastError() == ERROR_TIMEOUT
        ? WF_COMPLETION_WAIT_TIMED_OUT
        : WF_COMPLETION_WAIT_FAILED;
}

void wf_completion_wait_wake(wf_completion_wait *wait, int all) {
    wf_completion_wait_windows *windows = wf_completion_wait_of(wait);
    if (all != 0) {
        WakeAllConditionVariable(&windows->condition);
        return;
    }
    WakeConditionVariable(&windows->condition);
}
