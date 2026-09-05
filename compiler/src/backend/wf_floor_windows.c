/* Windows resource-exhaustion floor, and `wf_floor.c`'s twin.
 *
 * The entry body runs on a stack the program chose rather than on an inherited
 * executable default: a pool stack of the scheduler core's own reservation
 * where that core is linked, and a thread of its own where it is not. A
 * process-wide vectored exception handler classifies only
 * EXCEPTION_STACK_OVERFLOW as resource exhaustion. Every other exception keeps
 * the normal Windows search order and therefore its original failure identity.
 *
 * The three seams `wf_floor.c` has, with the same names and the same meanings,
 * because the unit above them -- `sched/entry.c` -- is one implementation for
 * both platforms:
 *
 *   `wf__floor_run` tests the weak `wf__sched_entry_stack` and returns the
 *   status the core answers with (design section 5);
 *   `wf__floor_attach_thread` is the per-thread half, which here is the
 *   emergency stack `SetThreadStackGuarantee` reserves;
 *   `wf__floor_set_stack_bounds` is the per-stack half, which here does
 *   nothing at all -- Windows classifies an overflow by exception code and
 *   never by address, so this floor has no bounds to write (design section 7,
 *   "Two platform facts fall out of that split").
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

extern int wf__main_body(int argc, char **argv);

/* The scheduler core's entry, and the link-time fact that selects the shape
 * below (design section 5).
 *
 * Strong in `sched/entry.c`, which every link that carries the core carries:
 * it starts the core, runs the body on a pool stack whose bottom is the
 * scheduler loop, and returns on this thread's own host stack with the status.
 * The weak answer here is "no core is linked", and then the entry keeps the
 * shape it has always had. The linker resolves this once and no runtime unit
 * has to ask anything.
 *
 * The body arrives as a pointer rather than as a symbol `entry.c` would have
 * to name, because a link that carries the core without an emitted module --
 * the probes -- would otherwise carry an undefined `wf__main_body`. */
__attribute__((weak)) int wf__sched_entry_stack(
    int (*body)(int, char **),
    int argc,
    char **argv,
    int *status
) {
    (void)body;
    (void)argc;
    (void)argv;
    (void)status;
    return 0;
}

/* The per-stack half of the attach, and the scheduler core's switch is its one
 * caller. It does nothing here, and that is the platform fact rather than an
 * omission: the handler below classifies by exception code, so this floor
 * reads no bounds and there is nothing for a switch to write. `prim_windows.c`
 * carries the weak answer for a core linked without this file; this is the
 * strong one, and it is why the floor is compiled into every program that
 * carries the core. */
void wf__floor_set_stack_bounds(unsigned char *low, unsigned char *high) {
    (void)low;
    (void)high;
}

#define WF_FLOOR_STACK_BYTES ((size_t)1024u * 1024u * 1024u)
#define WF_FLOOR_EXCEPTION_STACK_BYTES ((ULONG)64u * 1024u)

size_t wf__floor_stack_bytes(void) { return WF_FLOOR_STACK_BYTES; }

/* The `FileFactory`'s one native fact [SYS-10]: the credits this program may
 * still spend on opens. Windows grants a process on the order of sixteen
 * million handles, so a fixed capacity far below that is a true lower bound
 * on what the target provides; an open holding a permit is never refused a
 * handle by this process's own consumption. Nothing raises the count again:
 * an explicit close hands the credit back as the permit it returns [SYS-10].
 * Atomic because a permit may be reserved on whichever thread resumed the
 * reserving frame. */
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

static volatile int wf__floor_latch;

_Static_assert(sizeof(int) == sizeof(LONG), "floor latch width");
_Static_assert(_Alignof(int) == _Alignof(LONG), "floor latch alignment");

volatile int *wf__floor_record_latch(void) { return &wf__floor_latch; }

static const char WF_FLOOR_STACK_RECORD[] = "{\"resource\":\"stack\"}\n";

/* The floor's own writer.
 *
 * `WriteFile` on the standard error handle rather than `stdio`, because the
 * record's first writer is an exception handler running on a stack that has
 * just overflowed: the emergency stack `SetThreadStackGuarantee` reserves is
 * enough for a system call and not for the CRT's buffered path. */
static void wf__floor_write_error(const char *text, DWORD length) {
    HANDLE error_handle = GetStdHandle(STD_ERROR_HANDLE);
    DWORD offset = 0;

    if (error_handle == NULL || error_handle == INVALID_HANDLE_VALUE) {
        return;
    }
    while (offset < length) {
        DWORD written = 0;
        if (WriteFile(
                error_handle,
                text + offset,
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

static void wf__floor_emit_stack_record(void) {
    wf__floor_write_error(
        WF_FLOOR_STACK_RECORD,
        (DWORD)(sizeof(WF_FLOOR_STACK_RECORD) - 1u)
    );
}

/* This floor's one fail-stop, for the ends that are *not* a classified
 * exhaustion.
 *
 * A floor that cannot install its handler, or cannot start or join the thread
 * it runs the program on, has no classified boundary left to offer, and that
 * is a trusted-computing-base defect rather than a program outcome.  It says
 * which one before it ends the process: a bare `abort` under the release UCRT
 * takes the fast-fail path, which a shell reports as a bare status with no
 * message at all.
 *
 * The classified exhaustion itself does not come through here.  It has already
 * written the one record the boundary is defined as, and a second line beside
 * it would be a second record on the one channel that is allowed exactly
 * one. */
static _Noreturn void wf__floor_fail(const char *reason, DWORD length) {
    static const char prefix[] = "whitefoot floor: ";
    wf__floor_write_error(prefix, (DWORD)(sizeof(prefix) - 1u));
    wf__floor_write_error(reason, length);
    wf__floor_write_error("\n", 1u);
    abort();
}

#define WF_FLOOR_FAIL(text) wf__floor_fail((text), (DWORD)(sizeof(text) - 1u))

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
        /* The one classified exhaustion: the record above *is* the message,
         * and this abort is deliberately bare so the channel carries exactly
         * that one line (see `wf__floor_fail`). */
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
        WF_FLOOR_FAIL(
            "the exhaustion handler or its emergency stack could not be installed"
        );
    }
}

typedef struct wf__floor_call {
    int argc;
    char **argv;
    int status;
} wf__floor_call;

static unsigned __stdcall wf__floor_entry(void *opaque) {
    wf__floor_call *call = (wf__floor_call *)opaque;
    wf__floor_attach_thread();
    call->status = wf__main_body(call->argc, call->argv);
    return 0;
}

/* Runs the program's entry on a stack of this file's choosing.
 *
 * With the scheduler core linked that stack is a pool stack of the core's own
 * reservation, whose bottom frame is the scheduler loop. It has to be: a parked
 * entry stack resumed by another thread and run to its end would return from a
 * function that thread never called, so `wf__main_body` cannot bottom out in a
 * thread whose creator waits on `WaitForSingleObject`. The status is posted
 * from wherever the body finishes and `wf_sched_run` returns it here on this
 * thread's own host stack, which is the `WaitForSingleObject` this design
 * removes (design section 5).
 *
 * Without the core the function is exactly what it was, including the one
 * place this platform differs from the POSIX floor: a failed `_beginthreadex`
 * aborts, where `wf_floor.c` falls back to running the body on the host thread,
 * because failure to reserve its specified stack makes the native backend
 * unavailable. That abort is not ported away and has no POSIX counterpart to
 * reach -- there was never a second shape here. */
int wf__floor_run(int argc, void *argv) {
    wf__floor_call call;
    uintptr_t thread_value;
    HANDLE thread;

    call.argc = argc;
    call.argv = (char **)argv;
    call.status = 0;

    /* The host-created thread is armed too because it owns runtime startup and
     * the failure path. */
    wf__floor_attach_thread();

    if (wf__sched_entry_stack(wf__main_body, argc, call.argv, &call.status)) {
        return call.status;
    }

    thread_value = _beginthreadex(
        NULL,
        (unsigned)WF_FLOOR_STACK_BYTES,
        wf__floor_entry,
        &call,
        STACK_SIZE_PARAM_IS_A_RESERVATION,
        NULL
    );
    if (thread_value == 0) {
        WF_FLOOR_FAIL("the program's own thread could not be started");
    }
    thread = (HANDLE)thread_value;

    if (WaitForSingleObject(thread, INFINITE) != WAIT_OBJECT_0) {
        WF_FLOOR_FAIL("the program's own thread could not be joined");
    }
    (void)CloseHandle(thread);
    return call.status;
}
