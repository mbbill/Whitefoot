/* Windows's primitives for the scheduler core (`prim.h`, items 2 to 7, thread
 * identity, and the platform layer's own three; item 1 is inline in the
 * header).
 *
 * This is `prim_host.c`'s twin and nothing more: the same seven names, the
 * same seam, and the same contract, answered with the facilities Windows has.
 * Everything above it -- `core.c`, `entry.c`, the bridge -- is one
 * implementation shared by both platforms, which is the whole point of the
 * boundary design section 7.1 draws.
 *
 * The four platform choices, each named where design
 * `research/investigations/io-model/PARK-ON-MISS.md` section 7 makes it:
 *
 *   item 1, threads: `_beginthreadex` with STACK_SIZE_PARAM_IS_A_RESERVATION;
 *   item 2, the wait: an SRWLOCK and a CONDITION_VARIABLE on this file's own
 *     epoch, behind the same weak `wf__sched_host_*` seam the host has, so a
 *     completion link sleeps on the completion port instead;
 *   item 3, the switch: fibers, with `ConvertThreadToFiber` at a thread's
 *     first switch and `ConvertFiberToThread` at the entry thread's second
 *     entry to its host stack;
 *   item 4, the reservation: address space only, because a fiber owns its
 *     stack (see `wf_prim_reserve`).
 */

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "prim.h"
#include "core.h"

#include <windows.h>

#include <limits.h>
#include <process.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void wf_prim_fail(const char *reason) {
    (void)fputs("whitefoot scheduler: ", stderr);
    (void)fputs(reason, stderr);
    (void)fputs("\n", stderr);
    abort();
}

/* ---------------------------------------------------------- thread index */

static _Thread_local unsigned wf_prim_thread_ordinal;

unsigned wf_prim_thread_index(void) {
    return wf_prim_thread_ordinal;
}

void wf_prim_set_thread_index(unsigned index) {
    wf_prim_thread_ordinal = index;
}

/* ------------------------------------------------------------ 2. switch */

/* The switch is `SwitchToFiber`, and what the core stores through `save` is a
 * fiber handle rather than a stack pointer. Both are opaque to the core: it
 * writes one word before a switch and reads it back to switch again, and
 * `saved_sp` and `host_sp` are `void *` for exactly that reason (`core.h`).
 *
 * The thread's own host stack has to be a fiber before anything can switch
 * back to it, which is `wf_prim_thread_attach` below (design section 7,
 * platform item 3).
 *
 * Which fiber this thread is on is kept here rather than asked for with
 * `GetCurrentFiber`. The invariant is one line -- the switch names the fiber it
 * is about to enter, on the thread that enters it -- and it holds across a
 * stack that migrates, because a migrating stack arrives through this same
 * switch on its new thread. Keeping it also avoids the segment-register read
 * `GetCurrentFiber` expands to, which one of the two compilers this file must
 * be clean under reports as an out-of-bounds array access. */
static _Thread_local void *wf_prim_current_fiber;

void wf_prim_switch(void **save, void *load) {
    *save = wf_prim_current_fiber;
    wf_prim_current_fiber = load;
    SwitchToFiber(load);
}

/* The floor's two halves, as this platform's core sees them. The Windows
 * floor defines both strongly: the per-stack half does nothing there, because
 * that floor classifies an overflow by exception code and never by address,
 * and the attach is the emergency stack `SetThreadStackGuarantee` reserves for
 * the calling thread or fiber, which is why `wf_prim_fiber_main` below calls
 * it. The weak answers here are for a core linked without the floor -- the
 * adapter probe -- exactly as `prim_host.c` carries the same pair on the
 * host. This leaf is the one unit of a link that names the floor's attach,
 * and `wf_prim_floor_attach` is how `entry.c` reaches it: one link admits one
 * weak default per symbol (MSVC, LNK1227), and a PE weak default satisfies
 * only references from its own unit (GNU ld). */
__attribute__((weak)) void wf__floor_attach_thread(void) {}

__attribute__((weak)) void wf__floor_set_stack_bounds(
    unsigned char *low,
    unsigned char *high
) {
    (void)low;
    (void)high;
}

void wf_prim_floor_attach(void) {
    wf__floor_attach_thread();
}

/* What a prepared stack runs, and where its two words live.
 *
 * `CreateFiberEx` carries one `void *` to the fiber, and the core's entry is a
 * pair -- a function and its argument. The pair is written into the slot's own
 * top, just below the state header the core puts there, rather than into an
 * allocation: on Windows the slot's bytes are not a stack at all (the fiber
 * owns one of its own), so the only user of that page is the core's header and
 * this pair. Nothing here allocates, at start or after it. */
typedef struct wf_prim_fiber_entry {
    void (*entry)(void *);
    void *argument;
} wf_prim_fiber_entry;

_Static_assert(
    sizeof(wf_sched_stack) + sizeof(wf_prim_fiber_entry) + 32u <= 4096u,
    "the core's state header and the fiber's entry pair share the slot's top page"
);

static VOID WINAPI wf_prim_fiber_main(LPVOID opaque) {
    wf_prim_fiber_entry *call = (wf_prim_fiber_entry *)opaque;
    /* The floor's attach, on this fiber's own stack and before any frame of
     * the program is on it. A stack guarantee belongs to the calling thread
     * *or fiber*: "to set the stack guarantee for a fiber, you must first call
     * the SwitchToFiber function to execute the fiber", and the guarantee the
     * thread set on its host stack does not carry to a fiber it runs. Without
     * this call a pool stack overflows with no emergency stack under the
     * handler, which then has whatever is left of the page the guard was,
     * and whether that is enough depends on where in the page the descent
     * stopped -- the io-hosts overflow proof passed and failed on the same
     * bytes. This is the first frame of every pool stack, so it runs once per
     * fiber, and the guarantee then follows the fiber to whichever thread
     * runs it. */
    wf__floor_attach_thread();
    call->entry(call->argument);
    /* A prepared stack's entry is the scheduler loop and never returns; a
     * fiber that returned would end its thread with no scheduler on it. */
    wf_prim_fail("a pool stack's entry returned");
}

void *wf_prim_prepare_stack(
    void *top,
    size_t bytes,
    void (*entry)(void *),
    void *argument
) {
    unsigned char *base = (unsigned char *)top - sizeof(wf_prim_fiber_entry);
    wf_prim_fiber_entry *call;
    LPVOID fiber;
    base = (unsigned char *)((uintptr_t)base & ~(uintptr_t)15);
    call = (wf_prim_fiber_entry *)base;
    call->entry = entry;
    call->argument = argument;
    fiber = CreateFiberEx(
        0,
        bytes,
        FIBER_FLAG_FLOAT_SWITCH,
        wf_prim_fiber_main,
        call
    );
    if (fiber == NULL) {
        wf_prim_fail("a pool stack's fiber could not be created");
    }
    return fiber;
}

void wf_prim_set_bounds(unsigned char *low, unsigned char *high) {
    wf__floor_set_stack_bounds(low, high);
}

/* -------------------------------------------------------------- 3. park */

static SRWLOCK wf_prim_wake_lock = SRWLOCK_INIT;
static CONDITION_VARIABLE wf_prim_wake_signal = CONDITION_VARIABLE_INIT;
static uint64_t wf_prim_wake_epoch;
static unsigned wf_prim_sleepers;

/* Platform item 2 of design section 7, the Windows half.
 *
 * The core sleeps and wakes on whatever the link's owner of the host wait set
 * supplies. A completion link has one -- the bridge, whose park on Windows is
 * the completion port itself, with a wake delivered as a
 * `PostQueuedCompletionStatus` packet, so a ready stack and an I/O completion
 * arrive on one queue -- so the bridge defines these three strongly and the
 * mapping is written down at that definition. A core linked without a
 * completion runtime keeps the epoch on this file's own lock and condition
 * variable. Each returns nonzero when it handled the call. */
__attribute__((weak)) int wf__sched_host_epoch(uint64_t *epoch) {
    (void)epoch;
    return 0;
}

__attribute__((weak)) int wf__sched_host_park(uint64_t observed) {
    (void)observed;
    return 0;
}

__attribute__((weak)) int wf__sched_host_wake(void) {
    return 0;
}

uint64_t wf_prim_epoch(void) {
    uint64_t supplied = 0;
    if (wf__sched_host_epoch(&supplied)) {
        return supplied;
    }
    return (uint64_t)InterlockedCompareExchange64(
        (volatile LONG64 *)&wf_prim_wake_epoch,
        0,
        0
    );
}

void wf_prim_park(uint64_t observed) {
    if (wf__sched_host_park(observed)) {
        return;
    }
    AcquireSRWLockExclusive(&wf_prim_wake_lock);
    wf_prim_sleepers += 1u;
    while ((uint64_t)InterlockedCompareExchange64(
               (volatile LONG64 *)&wf_prim_wake_epoch,
               0,
               0
           ) == observed) {
        if (SleepConditionVariableSRW(
                &wf_prim_wake_signal,
                &wf_prim_wake_lock,
                INFINITE,
                0
            ) == FALSE) {
            /* The only documented failure of an unbounded wait on a lock this
             * process owns is a broken wait set, which no later wake can
             * repair; reporting it through the same lock would stall. */
            ReleaseSRWLockExclusive(&wf_prim_wake_lock);
            wf_prim_fail("the wake condition failed");
        }
    }
    wf_prim_sleepers -= 1u;
    ReleaseSRWLockExclusive(&wf_prim_wake_lock);
}

void wf_prim_wake(void) {
    if (wf__sched_host_wake()) {
        return;
    }
    AcquireSRWLockExclusive(&wf_prim_wake_lock);
    (void)InterlockedIncrement64((volatile LONG64 *)&wf_prim_wake_epoch);
    if (wf_prim_sleepers != 0u) {
        WakeAllConditionVariable(&wf_prim_wake_signal);
    }
    ReleaseSRWLockExclusive(&wf_prim_wake_lock);
}

/* ------------------------------------------------------- 4. reservation */

static size_t wf_prim_page(void) {
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    return info.dwPageSize > 0 ? (size_t)info.dwPageSize : 4096u;
}

size_t wf_prim_stack_stride(size_t bytes) {
    size_t page = wf_prim_page();
    size_t rounded = (bytes + page - 1u) / page * page;
    return page + rounded;
}

/* The reservation is address space and nothing else, because on Windows a
 * fiber owns its stack.
 *
 * `wf_prim_prepare_stack` asks `CreateFiberEx` for a stack of the slot's size,
 * and that is the memory a pool stack actually runs on. What this call is for
 * is the other half of a slot: a distinct `low`/`high` pair the core can name,
 * and somewhere to put the state header the core writes at the slot's top. So
 * the whole range is reserved and never committed -- no page of a slot's body
 * is ever touched, and the guard page every POSIX slot carries has no
 * counterpart because nothing descends here -- except the one page at each
 * slot's top, which holds the core's `wf_sched_stack` and the fiber's entry
 * pair and is therefore real storage rather than address space.
 *
 * A run that never parks pays for address space and one page per slot. */
unsigned char *wf_prim_reserve(unsigned count, size_t bytes) {
    size_t page = wf_prim_page();
    size_t stride = wf_prim_stack_stride(bytes);
    size_t total = stride * (size_t)count;
    unsigned char *base;
    unsigned index;
    base = (unsigned char *)VirtualAlloc(NULL, total, MEM_RESERVE, PAGE_NOACCESS);
    if (base == NULL) {
        return NULL;
    }
    for (index = 0; index < count; index += 1u) {
        unsigned char *header = base + (size_t)index * stride + stride - page;
        if (VirtualAlloc(header, page, MEM_COMMIT, PAGE_READWRITE) == NULL) {
            (void)VirtualFree(base, 0, MEM_RELEASE);
            return NULL;
        }
    }
    return base;
}

/* -------------------------------------------------------------- 5. lock */

static SRWLOCK wf_prim_core_lock = SRWLOCK_INIT;

void wf_prim_lock(enum wf_prim_section section) {
    (void)section;
    AcquireSRWLockExclusive(&wf_prim_core_lock);
}

void wf_prim_unlock(void) {
    ReleaseSRWLockExclusive(&wf_prim_core_lock);
}

/* ------------------------------------------------------------- 6. yield */

void wf_prim_yield(void) {
    (void)SwitchToThread();
}

/* ---------------------------------------------------------- 7. progress */

__attribute__((weak)) int wf__sched_target_progress(struct wf_sched_core *core) {
    (void)core;
    return 0;
}

int wf_prim_progress(struct wf_sched_core *core) {
    return wf__sched_target_progress(core);
}

/* ------------------------------------- the platform layer's own three */

/* P1. One detached thread on a host stack this call reserves.
 *
 * STACK_SIZE_PARAM_IS_A_RESERVATION is what makes the size a reservation
 * rather than a commitment, which is the same promise `MAP_NORESERVE` carries
 * on the host: the pages are committed as the thread descends. The handle is
 * closed at once, which is how a Windows thread is detached.
 *
 * The entry pair is the caller's record and nothing is allocated here; see
 * `prim.h` for why the runtime cannot take that pair from anywhere else. */
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

/* P2. A Windows thread's host stack has to become a fiber before anything can
 * switch back to it, and has to stop being one before the thread ends. Attach
 * runs once at a thread's start and detach once at the entry thread's second
 * and last entry to its host stack (design section 5). */
void wf_prim_thread_attach(void) {
    LPVOID fiber;
    if (wf_prim_current_fiber != NULL) {
        return;
    }
    fiber = ConvertThreadToFiberEx(NULL, FIBER_FLAG_FLOAT_SWITCH);
    if (fiber == NULL) {
        wf_prim_fail("a thread's host stack could not become a fiber");
    }
    wf_prim_current_fiber = fiber;
}

void wf_prim_thread_detach(void) {
    if (wf_prim_current_fiber == NULL) {
        return;
    }
    (void)ConvertFiberToThread();
    wf_prim_current_fiber = NULL;
}

/* P3. The CPUs this process may be scheduled on right now. Zero means the
 * platform would not say, and the caller's own policy decides what to do with
 * that. */
unsigned wf_prim_online_cpus(void) {
    DWORD active = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
    return active > 0 ? (unsigned)active : 0u;
}

/* P4. The text of one runtime setting.
 *
 * `GetEnvironmentVariableA` reports an unset name by answering 0 with a last
 * error of ERROR_ENVVAR_NOT_FOUND, and a name whose value is the empty string
 * by answering 0 with the last error untouched -- so the error is cleared
 * before the call and read after it, which is the only way the two are told
 * apart. A value the buffer cannot hold answers with the length it would need,
 * including the terminator, which is why "at least the capacity" is the
 * doesn't-fit test and "less than the capacity" is a real length. */
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
