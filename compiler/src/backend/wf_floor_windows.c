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

#include <stddef.h>
#include <stdlib.h>

extern int wf__main_body(int argc, void *argv);

#define WF_FLOOR_STACK_BYTES ((size_t)1024u * 1024u * 1024u)
#define WF_FLOOR_EXCEPTION_STACK_BYTES ((ULONG)64u * 1024u)

size_t wf__floor_stack_bytes(void) { return WF_FLOOR_STACK_BYTES; }

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

static BOOL CALLBACK wf__floor_install_handler(
    PINIT_ONCE once,
    PVOID parameter,
    PVOID *context
) {
    (void)once;
    (void)parameter;
    (void)context;
    (void)AddVectoredExceptionHandler(1u, wf__floor_exception_handler);
    return TRUE;
}

/* The handler is process-wide. The guarantee is per-thread, so every runtime
 * thread that can execute Whitefoot code calls this function on itself. */
void wf__floor_attach_thread(void) {
    ULONG stack_guarantee = WF_FLOOR_EXCEPTION_STACK_BYTES;
    (void)InitOnceExecuteOnce(
        &wf__floor_install_once,
        wf__floor_install_handler,
        NULL,
        NULL
    );
    (void)SetThreadStackGuarantee(&stack_guarantee);
}

typedef struct wf__floor_call {
    int argc;
    void *argv;
    int status;
} wf__floor_call;

static DWORD WINAPI wf__floor_entry(LPVOID opaque) {
    wf__floor_call *call = (wf__floor_call *)opaque;
    wf__floor_attach_thread();
    call->status = wf__main_body(call->argc, call->argv);
    return 0;
}

int wf__floor_run(int argc, void *argv) {
    wf__floor_call call;
    HANDLE thread;

    call.argc = argc;
    call.argv = argv;
    call.status = 0;

    /* This also gives the host-created thread the same precise classification
     * if reserving the dedicated entry stack fails and execution falls back. */
    wf__floor_attach_thread();
    thread = CreateThread(
        NULL,
        WF_FLOOR_STACK_BYTES,
        wf__floor_entry,
        &call,
        STACK_SIZE_PARAM_IS_A_RESERVATION,
        NULL
    );
    if (thread == NULL) {
        return wf__main_body(argc, argv);
    }

    (void)WaitForSingleObject(thread, INFINITE);
    (void)CloseHandle(thread);
    return call.status;
}
