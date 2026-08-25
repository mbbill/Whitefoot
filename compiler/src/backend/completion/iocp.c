#if !defined(_WIN32)
#error "iocp.c is the Windows completion backend"
#endif

#include "contract.h"
#include "platform.h"

#include <stdlib.h>

#define WF_IO_PLATFORM_THREADS 16u

typedef struct wf_io_iocp_lane {
    HANDLE port;
    _Atomic unsigned initialized;
} wf_io_iocp_lane;

typedef struct wf_io_thread_start {
    wf_io_thread_fn function;
    void *context;
} wf_io_thread_start;

static wf_io_iocp_lane wf_io_iocp_lanes[WF_IO_MAX_LANES];
static wf_io_thread_start wf_io_thread_starts[WF_IO_PLATFORM_THREADS];
static _Atomic unsigned wf_io_thread_start_count;
static _Atomic uint64_t wf_io_iocp_ports;

static void wf_io_iocp_defect(void) {
    abort();
}

static DWORD WINAPI wf_io_windows_thread_entry(LPVOID opaque) {
    wf_io_thread_start *start = (wf_io_thread_start *)opaque;
    start->function(start->context);
    return 0;
}

int wf_io_platform_thread_start(wf_io_thread_fn function, void *context) {
    HANDLE thread;
    unsigned slot = atomic_fetch_add_explicit(
        &wf_io_thread_start_count,
        1,
        memory_order_acq_rel
    );
    if (slot >= WF_IO_PLATFORM_THREADS) {
        return -1;
    }
    wf_io_thread_starts[slot].function = function;
    wf_io_thread_starts[slot].context = context;
    thread = CreateThread(
        NULL,
        0,
        wf_io_windows_thread_entry,
        &wf_io_thread_starts[slot],
        0,
        NULL
    );
    if (thread == NULL) {
        return -1;
    }
    CloseHandle(thread);
    return 0;
}

int wf_io_platform_global_init(void) {
    return 0;
}

int wf_io_platform_lane_init(unsigned lane) {
    HANDLE port;
    if (lane >= WF_IO_MAX_LANES) {
        return -1;
    }
    port = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 1);
    if (port == NULL) {
        return -1;
    }
    wf_io_iocp_lanes[lane].port = port;
    atomic_init(&wf_io_iocp_lanes[lane].initialized, 0);
    atomic_store_explicit(&wf_io_iocp_lanes[lane].initialized, 1, memory_order_release);
    atomic_fetch_add_explicit(&wf_io_iocp_ports, 1, memory_order_relaxed);
    return 0;
}

void wf_io_platform_wake(unsigned lane) {
    wf_io_iocp_lane *endpoint;
    if (lane >= WF_IO_MAX_LANES) {
        wf_io_iocp_defect();
    }
    endpoint = &wf_io_iocp_lanes[lane];
    if (atomic_load_explicit(&endpoint->initialized, memory_order_acquire) == 0
        || !PostQueuedCompletionStatus(endpoint->port, 0, (ULONG_PTR)lane + 1, NULL)) {
        wf_io_iocp_defect();
    }
}

void wf_io_platform_park(unsigned lane) {
    wf_io_iocp_lane *endpoint;
    DWORD bytes = 0;
    ULONG_PTR key = 0;
    OVERLAPPED *overlapped = NULL;
    BOOL completed;
    if (lane >= WF_IO_MAX_LANES) {
        wf_io_iocp_defect();
    }
    endpoint = &wf_io_iocp_lanes[lane];
    completed = GetQueuedCompletionStatus(
        endpoint->port,
        &bytes,
        &key,
        &overlapped,
        INFINITE
    );
    if (!completed || bytes != 0 || key != (ULONG_PTR)lane + 1 || overlapped != NULL) {
        wf_io_iocp_defect();
    }
}

const char *wf_io_platform_name(void) {
    return "iocp";
}

unsigned wf_io_platform_features(void) {
    return WF_IO_BACKEND_IOCP | WF_IO_BACKEND_UNIFIED_PARK;
}

uint64_t wf_io_platform_poll_arms(void) {
    return atomic_load_explicit(&wf_io_iocp_ports, memory_order_relaxed);
}

int wf_io_platform_mutex_init(wf_io_mutex *mutex) {
    InitializeCriticalSection(mutex);
    return 0;
}

int wf_io_platform_condition_init(wf_io_condition *condition) {
    InitializeConditionVariable(condition);
    return 0;
}

void wf_io_platform_mutex_lock(wf_io_mutex *mutex) {
    EnterCriticalSection(mutex);
}

void wf_io_platform_mutex_unlock(wf_io_mutex *mutex) {
    LeaveCriticalSection(mutex);
}

void wf_io_platform_condition_wait(wf_io_condition *condition, wf_io_mutex *mutex) {
    if (!SleepConditionVariableCS(condition, mutex, INFINITE)) {
        wf_io_iocp_defect();
    }
}

void wf_io_platform_condition_signal(wf_io_condition *condition) {
    WakeConditionVariable(condition);
}

void wf_io_platform_pause(void) {
    if (!SwitchToThread()) {
        Sleep(0);
    }
}
