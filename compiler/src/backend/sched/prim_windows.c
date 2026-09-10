/* Windows threads and per-lane waiting; no fibers. */
#include "prim.h"
#include <process.h>
#include <limits.h>
#include <stdlib.h>

static unsigned __stdcall wf_prim_thread_main(void *opaque) {
    wf_prim_thread *thread = (wf_prim_thread *)opaque;
    thread->entry(thread->argument);
    return 0u;
}

int wf_prim_thread_start(
    wf_prim_thread *thread,
    void (*entry)(void *),
    void *argument,
    size_t stack_bytes
) {
    uintptr_t created;
    if (thread == NULL || entry == NULL || stack_bytes > (size_t)UINT_MAX) {
        return 1;
    }
    thread->entry = entry;
    thread->argument = argument;
    created = _beginthreadex(
        NULL,
        (unsigned)stack_bytes,
        wf_prim_thread_main,
        thread,
        STACK_SIZE_PARAM_IS_A_RESERVATION,
        NULL
    );
    if (created == 0) {
        return 1;
    }
    (void)CloseHandle((HANDLE)created);
    return 0;
}

unsigned wf_prim_online_cpus(void) {
    DWORD active = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
    return active > 0 ? (unsigned)active : 0u;
}

int wf_prim_setting_text(const char *name, char *buffer, size_t capacity) {
    DWORD written;
    if (name == NULL || buffer == NULL || capacity == 0
        || capacity > (size_t)MAXDWORD) {
        return -1;
    }
    buffer[0] = '\0';
    SetLastError(ERROR_SUCCESS);
    written = GetEnvironmentVariableA(name, buffer, (DWORD)capacity);
    if (written == 0) {
        buffer[0] = '\0';
        return GetLastError() == ERROR_ENVVAR_NOT_FOUND ? 0 : 1;
    }
    if (written >= (DWORD)capacity) {
        buffer[0] = '\0';
        return -1;
    }
    return 1;
}

__attribute__((weak)) void wf__floor_attach_thread(void) {}
void wf_prim_floor_attach(void) { wf__floor_attach_thread(); }
void wf_prim_yield(void) { SwitchToThread(); }
int wf_prim_wait_init(wf_prim_wait *wait) {
    InitializeSRWLock(&wait->lock);
    InitializeConditionVariable(&wait->signal);
    return 0;
}
void wf_prim_wait_destroy(wf_prim_wait *wait) { (void)wait; }
void wf_prim_wait_lock(wf_prim_wait *wait) { AcquireSRWLockExclusive(&wait->lock); }
void wf_prim_wait_unlock(wf_prim_wait *wait) { ReleaseSRWLockExclusive(&wait->lock); }
void wf_prim_wait_sleep(wf_prim_wait *wait) {
    if (!SleepConditionVariableSRW(&wait->signal, &wait->lock, INFINITE, 0)) abort();
}
void wf_prim_wait_signal(wf_prim_wait *wait) { WakeConditionVariable(&wait->signal); }
