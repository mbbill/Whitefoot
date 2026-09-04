/* Windows resource-exhaustion floor.
 *
 * The entry body runs on a thread whose stack reservation is part of the
 * program rather than an inherited executable default. A process-wide
 * vectored exception handler classifies only EXCEPTION_STACK_OVERFLOW as
 * resource exhaustion. Every other exception keeps the normal Windows search
 * order and therefore its original failure identity.
 *
 * The exception path uses only a fixed record, InterlockedCompareExchange,
 * GetStdHandle, WriteFile, Sleep, and abort. It performs no allocation, stdio,
 * formatting, locking, or thread discovery.
 */

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0600
#endif

#include <windows.h>

#include <process.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

extern int wf__main_body(int argc, void *argv);

#define WF_FLOOR_STACK_BYTES ((size_t)1024u * 1024u * 1024u)
#define WF_FLOOR_EXCEPTION_STACK_BYTES ((ULONG)64u * 1024u)

size_t wf__floor_stack_bytes(void) { return WF_FLOOR_STACK_BYTES; }

/* The `FileFactory`'s one native fact [SYS-10]: the credits this program may
 * still spend on opens. Windows grants a process on the order of sixteen
 * million handles, so a fixed capacity far below that is a true lower bound
 * on what the target provides; an open holding a permit is never refused a
 * handle by this process's own consumption. Atomic because explicit closes
 * return credits on whichever thread resumed the closing frame. */
#define WF_FILE_CAPACITY 4096L

static volatile LONG wf__file_credits = (LONG)WF_FILE_CAPACITY;

int wf__file_reserve(void) {
    LONG credits = wf__file_credits;
    while (credits > 0) {
        LONG seen = InterlockedCompareExchange(&wf__file_credits, credits - 1, credits);
        if (seen == credits) {
            return 1;
        }
        credits = seen;
    }
    return 0;
}

void wf__file_credit_return(void) {
    (void)InterlockedIncrement(&wf__file_credits);
}

static volatile int wf__floor_latch;

_Static_assert(sizeof(int) == sizeof(LONG), "floor latch width");
_Static_assert(_Alignof(int) == _Alignof(LONG), "floor latch alignment");

volatile int *wf__floor_record_latch(void) { return &wf__floor_latch; }

static const char WF_FLOOR_STACK_RECORD[] = "{\"resource\":\"stack\"}\n";

static void wf__floor_emit_stack_record(void) {
    HANDLE error_handle = GetStdHandle(STD_ERROR_HANDLE);
    DWORD offset = 0;
    const DWORD length = (DWORD)(sizeof(WF_FLOOR_STACK_RECORD) - 1u);

    if (error_handle == NULL || error_handle == INVALID_HANDLE_VALUE) {
        return;
    }
    while (offset < length) {
        DWORD written = 0;
        if (WriteFile(
                error_handle,
                WF_FLOOR_STACK_RECORD + offset,
                length - offset,
                &written,
                NULL
            ) == FALSE
            || written == 0) {
            return;
        }
        offset += written;
    }
}

static LONG CALLBACK wf__floor_exception_handler(
    EXCEPTION_POINTERS *exception
) {
    if (exception == NULL || exception->ExceptionRecord == NULL
        || exception->ExceptionRecord->ExceptionCode
            != EXCEPTION_STACK_OVERFLOW) {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    if (InterlockedCompareExchange(
            (volatile LONG *)&wf__floor_latch,
            1,
            0
        ) == 0) {
        wf__floor_emit_stack_record();
        abort();
    }

    /* A different record writer owns the one process-wide channel. It will
     * abort the process; this thread must not race it with a second record. */
    for (;;) {
        Sleep(INFINITE);
    }
}

static INIT_ONCE wf__floor_install_once = INIT_ONCE_STATIC_INIT;
static PVOID wf__floor_handler;

static BOOL CALLBACK wf__floor_install_handler(
    PINIT_ONCE once,
    PVOID parameter,
    PVOID *context
) {
    (void)once;
    (void)parameter;
    (void)context;
    wf__floor_handler = AddVectoredExceptionHandler(
        1u,
        wf__floor_exception_handler
    );
    return wf__floor_handler != NULL;
}

/* The handler is process-wide. The guarantee is per-thread, so every runtime
 * thread that can execute Whitefoot code calls this function on itself. */
void wf__floor_attach_thread(void) {
    ULONG stack_guarantee = WF_FLOOR_EXCEPTION_STACK_BYTES;
    if (InitOnceExecuteOnce(
            &wf__floor_install_once,
            wf__floor_install_handler,
            NULL,
            NULL
        ) == FALSE
        || SetThreadStackGuarantee(&stack_guarantee) == FALSE) {
        /* A runtime thread without the process handler or its emergency stack
         * cannot preserve Whitefoot's one classified exhaustion boundary.
         * Continuing would be a silent change of runtime semantics, so the
         * native backend is unavailable rather than degraded. */
        abort();
    }
}

typedef struct wf__floor_call {
    int argc;
    void *argv;
    int status;
} wf__floor_call;

static unsigned __stdcall wf__floor_entry(void *opaque) {
    wf__floor_call *call = (wf__floor_call *)opaque;
    wf__floor_attach_thread();
    call->status = wf__main_body(call->argc, call->argv);
    return 0;
}

int wf__floor_run(int argc, void *argv) {
    wf__floor_call call;
    uintptr_t thread_value;
    HANDLE thread;

    call.argc = argc;
    call.argv = argv;
    call.status = 0;

    /* The host-created thread is armed too because it owns runtime startup and
     * the failure path. The Whitefoot body itself never falls back to this
     * inherited stack: failure to reserve its specified stack makes the
     * native backend unavailable. */
    wf__floor_attach_thread();
    thread_value = _beginthreadex(
        NULL,
        (unsigned)WF_FLOOR_STACK_BYTES,
        wf__floor_entry,
        &call,
        STACK_SIZE_PARAM_IS_A_RESERVATION,
        NULL
    );
    if (thread_value == 0) {
        abort();
    }
    thread = (HANDLE)thread_value;

    if (WaitForSingleObject(thread, INFINITE) != WAIT_OBJECT_0) {
        abort();
    }
    (void)CloseHandle(thread);
    return call.status;
}
