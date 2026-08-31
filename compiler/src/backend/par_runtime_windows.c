/* Whitefoot native Windows parallel runtime.
 *
 * The compiler has already proved that every published call may overlap. This
 * file supplies only the scheduling half of that contract: one Chase-Lev deque
 * and a bounded frame store per scheduler lane, work-first joins, and worker
 * threads with the same reserved-stack/exhaustion boundary as the entry.
 *
 * A Windows --par executable has no sequential-backend escape hatch. Invalid
 * configuration, an unavailable processor topology, a frame that exceeds the
 * runtime contract, or a partially-created pool terminates with an explicit
 * diagnostic. The only ordinary refusal is a full per-lane frame store; that
 * is transient backpressure, and the emitted work-first edge runs the same
 * permitted call on its owner.
 */

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include <windows.h>

#include <intrin.h>
#include <process.h>
#include <errno.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#if !defined(_WIN32)
#error "par_runtime_windows.c requires a Windows target"
#endif

#define WF_PAR_MAX_LANES 64
#define WF_PAR_FRAME_BYTES 256
#define WF_PAR_LANE_SLOTS 64
#define WF_PAR_CACHE_LINE 128

#define WF_PAR_SPIN_ROUNDS 4096
#define WF_PAR_YIELD_ROUNDS 16

#define WF_PAR_SPLIT_OVERSUBSCRIBE UINT64_C(16)
#define WF_PAR_SPLIT_WORK_PER_CHUNK UINT64_C(1200000)

#define WF_PAR_SLOT_FREE 0
#define WF_PAR_SLOT_PENDING 1
#define WF_PAR_SLOT_DONE 2

#if defined(_MSC_VER)
#define WF_PAR_THREAD_LOCAL __declspec(thread)
#else
#define WF_PAR_THREAD_LOCAL _Thread_local
#endif

_Static_assert(sizeof(LONG) == sizeof(unsigned long), "Windows counter ABI");
_Static_assert(sizeof(LONG64) == sizeof(uint64_t), "64-bit scheduler words");
_Static_assert(
    (WF_PAR_LANE_SLOTS & (WF_PAR_LANE_SLOTS - 1)) == 0,
    "lane slot count must be a power of two"
);

struct wf__par_lane;

struct wf__par_slot {
    _Alignas(16) unsigned char frame[WF_PAR_FRAME_BYTES];
    void (*run)(void *);
    volatile LONG state;
    struct wf__par_lane *home;
    int next_free;
};

struct wf__par_lane {
    _Alignas(WF_PAR_CACHE_LINE) volatile LONG64 top;

    _Alignas(WF_PAR_CACHE_LINE) volatile LONG64 bottom;
    struct wf__par_slot *volatile buffer[WF_PAR_LANE_SLOTS];
    int free_head;
    uint64_t seed;

    /* WaitOnAddress sleeps on this generation. A publisher increments it
     * before waking this lane, so a wake between the final scan and the wait
     * is observed as a changed comparison value rather than lost. */
    _Alignas(WF_PAR_CACHE_LINE) volatile LONG wake_generation;

#if defined(WF_PAR_IDENTITY_PROBE)
    /* Test-only identity evidence is written only by this lane's owner and read
     * after source joins have completed.  Delivered runtimes do not carry
     * these fields or either per-task write. */
    _Alignas(WF_PAR_CACHE_LINE) uint64_t steals;
    uint64_t executions;
#endif

    _Alignas(WF_PAR_CACHE_LINE)
        struct wf__par_slot slots[WF_PAR_LANE_SLOTS];
};

static struct wf__par_lane wf__par_lanes[WF_PAR_MAX_LANES];
static volatile LONG wf__par_lane_count;
static INIT_ONCE wf__par_started = INIT_ONCE_STATIC_INIT;

/* One bit per parked lane. The fixed word and WF_PAR_MAX_LANES are one
 * invariant; increasing either requires increasing both. */
_Alignas(WF_PAR_CACHE_LINE) static volatile LONG64 wf__par_idle;

/* Worker readiness is the startup rendezvous.  Probe builds also expose it
 * through the aggregate identity accessors below. */
_Alignas(WF_PAR_CACHE_LINE) static volatile LONG wf__par_ready_workers;

static WF_PAR_THREAD_LOCAL struct wf__par_lane *wf__par_self;
static WF_PAR_THREAD_LOCAL int wf__par_attached;

#if defined(WF_PAR_WITH_WRITER_SCHEDULER)
extern int wf__writer_scheduler_help_once(void);
#define WF_PAR_WRITER_HELP_ONCE() wf__writer_scheduler_help_once()
#else
#define WF_PAR_WRITER_HELP_ONCE() 0
#endif

extern size_t wf__floor_stack_bytes(void);
extern void wf__floor_attach_thread(void);

/* ---------------------------------------------------------------- atomics */

/* The shipped driver compiles this source with Clang for the MSVC ABI. Clang's
 * atomics preserve the relaxed/acquire distinctions on the deque hot path.
 * The Interlocked branch keeps the source valid for cl.exe as well; its full
 * barriers are stronger than required but retain the same algorithm. */
static LONG64 wf__par_load64_relaxed(volatile LONG64 *value) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_load_n(value, __ATOMIC_RELAXED);
#else
    return InterlockedCompareExchange64(value, 0, 0);
#endif
}

static LONG64 wf__par_load64_acquire(volatile LONG64 *value) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_load_n(value, __ATOMIC_ACQUIRE);
#else
    return InterlockedCompareExchange64(value, 0, 0);
#endif
}

static LONG64 wf__par_load64_seq(volatile LONG64 *value) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_load_n(value, __ATOMIC_SEQ_CST);
#else
    return InterlockedCompareExchange64(value, 0, 0);
#endif
}

static void wf__par_store64_relaxed(volatile LONG64 *value, LONG64 desired) {
#if defined(__clang__) || defined(__GNUC__)
    __atomic_store_n(value, desired, __ATOMIC_RELAXED);
#else
    (void)InterlockedExchange64(value, desired);
#endif
}

static void wf__par_store64_seq(volatile LONG64 *value, LONG64 desired) {
#if defined(__clang__) || defined(__GNUC__)
    __atomic_store_n(value, desired, __ATOMIC_SEQ_CST);
#else
    (void)InterlockedExchange64(value, desired);
#endif
}

static int wf__par_compare_exchange64_seq(
    volatile LONG64 *value,
    LONG64 *expected,
    LONG64 desired
) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_compare_exchange_n(
        value,
        expected,
        desired,
        0,
        __ATOMIC_SEQ_CST,
        __ATOMIC_RELAXED
    );
#else
    LONG64 observed = InterlockedCompareExchange64(value, desired, *expected);
    if (observed == *expected) {
        return 1;
    }
    *expected = observed;
    return 0;
#endif
}

static LONG64 wf__par_fetch_or64_seq(
    volatile LONG64 *value,
    LONG64 mask
) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_fetch_or(value, mask, __ATOMIC_SEQ_CST);
#else
    return InterlockedOr64(value, mask);
#endif
}

static LONG64 wf__par_fetch_and64_seq(
    volatile LONG64 *value,
    LONG64 mask
) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_fetch_and(value, mask, __ATOMIC_SEQ_CST);
#else
    return InterlockedAnd64(value, mask);
#endif
}

static LONG wf__par_load32_relaxed(volatile LONG *value) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_load_n(value, __ATOMIC_RELAXED);
#else
    return InterlockedCompareExchange(value, 0, 0);
#endif
}

static LONG wf__par_load32_acquire(volatile LONG *value) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_load_n(value, __ATOMIC_ACQUIRE);
#else
    return InterlockedCompareExchange(value, 0, 0);
#endif
}

static void wf__par_store32_relaxed(volatile LONG *value, LONG desired) {
#if defined(__clang__) || defined(__GNUC__)
    __atomic_store_n(value, desired, __ATOMIC_RELAXED);
#else
    (void)InterlockedExchange(value, desired);
#endif
}

static void wf__par_store32_seq(volatile LONG *value, LONG desired) {
#if defined(__clang__) || defined(__GNUC__)
    __atomic_store_n(value, desired, __ATOMIC_SEQ_CST);
#else
    (void)InterlockedExchange(value, desired);
#endif
}

static LONG wf__par_increment32(volatile LONG *value) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_add_fetch(value, 1, __ATOMIC_RELAXED);
#else
    return InterlockedIncrement(value);
#endif
}

static int wf__par_compare_exchange32(
    volatile LONG *value,
    LONG *expected,
    LONG desired
) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_compare_exchange_n(
        value,
        expected,
        desired,
        0,
        __ATOMIC_ACQ_REL,
        __ATOMIC_RELAXED
    );
#else
    LONG observed = InterlockedCompareExchange(value, desired, *expected);
    if (observed == *expected) {
        return 1;
    }
    *expected = observed;
    return 0;
#endif
}

#if defined(WF_PAR_IDENTITY_PROBE)
static unsigned long wf__par_narrow_counter(uint64_t total) {
    return total > (uint64_t)ULONG_MAX ? ULONG_MAX : (unsigned long)total;
}

static unsigned long wf__par_grant_total(void) {
    int count = (int)wf__par_load32_acquire(&wf__par_lane_count);
    uint64_t total = 0;
    int index;
    for (index = 0; index < count; index += 1) {
        uint64_t value = wf__par_lanes[index].steals;
        if (UINT64_MAX - total < value) {
            return ULONG_MAX;
        }
        total += value;
    }
    return wf__par_narrow_counter(total);
}

static unsigned long wf__par_worker_execution_total(void) {
    int count = (int)wf__par_load32_acquire(&wf__par_lane_count);
    uint64_t total = 0;
    int index;
    for (index = 1; index < count; index += 1) {
        uint64_t value = wf__par_lanes[index].executions;
        if (UINT64_MAX - total < value) {
            return ULONG_MAX;
        }
        total += value;
    }
    return wf__par_narrow_counter(total);
}
#endif

/* A thief reads a ring cell before it settles ownership on `top`.  The owner
 * may cycle far enough to reuse that cell while a preempted thief is still
 * holding its old observation, so the cell itself must be atomic even though
 * a failed thief never dereferences it.  The shipped Clang/MSVC-ABI build gets
 * true relaxed operations; cl.exe's Interlocked fallback is stronger but
 * preserves the same protocol. */
static void wf__par_store_slot_relaxed(
    struct wf__par_slot *volatile *value,
    struct wf__par_slot *desired
) {
#if defined(__clang__) || defined(__GNUC__)
    __atomic_store_n(value, desired, __ATOMIC_RELAXED);
#else
    (void)InterlockedExchangePointer(
        (PVOID volatile *)value,
        (PVOID)desired
    );
#endif
}

static struct wf__par_slot *wf__par_load_slot_relaxed(
    struct wf__par_slot *volatile *value
) {
#if defined(__clang__) || defined(__GNUC__)
    return __atomic_load_n(value, __ATOMIC_RELAXED);
#else
    return (struct wf__par_slot *)InterlockedCompareExchangePointer(
        (PVOID volatile *)value,
        NULL,
        NULL
    );
#endif
}

/* --------------------------------------------------------------- failures */

static void wf__par_write_error(HANDLE error_handle, const char *text) {
    size_t remaining = strlen(text);
    const char *cursor = text;
    while (remaining != 0) {
        DWORD requested = remaining > (size_t)MAXDWORD
            ? MAXDWORD
            : (DWORD)remaining;
        DWORD written = 0;
        if (WriteFile(error_handle, cursor, requested, &written, NULL) == FALSE
            || written == 0) {
            return;
        }
        cursor += written;
        remaining -= written;
    }
}

__declspec(noreturn) static void wf__par_fatal(const char *reason) {
    HANDLE error_handle = GetStdHandle(STD_ERROR_HANDLE);
    if (error_handle != NULL && error_handle != INVALID_HANDLE_VALUE) {
        wf__par_write_error(error_handle, "whitefoot parallel runtime: ");
        wf__par_write_error(error_handle, reason);
        wf__par_write_error(error_handle, "\n");
    }
    abort();
}

/* --------------------------------------------------------------- sleeping */

static void wf__par_signal(struct wf__par_lane *lane) {
    (void)wf__par_increment32(&lane->wake_generation);
    WakeByAddressSingle((PVOID)&lane->wake_generation);
}

static void wf__par_wake_one(void) {
    uint64_t parked = (uint64_t)wf__par_load64_relaxed(&wf__par_idle);
    while (parked != 0) {
        int index;
        for (index = 0; index < WF_PAR_MAX_LANES; index += 1) {
            uint64_t bit = UINT64_C(1) << (unsigned)index;
            if ((parked & bit) != 0) {
                LONG64 observed = wf__par_fetch_and64_seq(
                    &wf__par_idle,
                    (LONG64)~bit
                );
                if (((uint64_t)observed & bit) != 0) {
                    wf__par_signal(&wf__par_lanes[index]);
                    return;
                }
                parked &= ~bit;
                break;
            }
        }
    }
}

static void wf__par_park(
    struct wf__par_lane *lane,
    LONG observed_generation
) {
    if (WaitOnAddress(
            (volatile VOID *)&lane->wake_generation,
            &observed_generation,
            sizeof(observed_generation),
            INFINITE
        ) == FALSE) {
        wf__par_fatal("WaitOnAddress failed while parking a worker");
    }
}

/* ------------------------------------------------------------------ deque */

static void wf__par_push(
    struct wf__par_lane *lane,
    struct wf__par_slot *slot
) {
    uint64_t bottom = (uint64_t)wf__par_load64_relaxed(&lane->bottom);
    wf__par_store_slot_relaxed(
        &lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)],
        slot
    );
    wf__par_store64_seq(&lane->bottom, (LONG64)(bottom + 1));
}

static struct wf__par_slot *wf__par_pop(struct wf__par_lane *lane) {
    uint64_t bottom = (uint64_t)wf__par_load64_relaxed(&lane->bottom);
    uint64_t top = (uint64_t)wf__par_load64_relaxed(&lane->top);
    struct wf__par_slot *slot;
    LONG64 expected;
    if ((int64_t)(bottom - top) <= 0) {
        return NULL;
    }
    bottom -= 1;
    wf__par_store64_seq(&lane->bottom, (LONG64)bottom);
    top = (uint64_t)wf__par_load64_seq(&lane->top);
    if ((int64_t)(bottom - top) < 0) {
        wf__par_store64_relaxed(&lane->bottom, (LONG64)(bottom + 1));
        return NULL;
    }
    slot = wf__par_load_slot_relaxed(
        &lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)]
    );
    if ((int64_t)(bottom - top) > 0) {
        return slot;
    }
    expected = (LONG64)top;
    if (!wf__par_compare_exchange64_seq(
            &lane->top,
            &expected,
            (LONG64)(top + 1)
        )) {
        slot = NULL;
    }
    wf__par_store64_relaxed(&lane->bottom, (LONG64)(bottom + 1));
    return slot;
}

static struct wf__par_slot *wf__par_steal(
    struct wf__par_lane *thief,
    struct wf__par_lane *victim
) {
    uint64_t top = (uint64_t)wf__par_load64_acquire(&victim->top);
    uint64_t bottom = (uint64_t)wf__par_load64_acquire(&victim->bottom);
    struct wf__par_slot *slot;
    LONG64 expected;
    if ((int64_t)(bottom - top) <= 0) {
        return NULL;
    }
    slot = wf__par_load_slot_relaxed(
        &victim->buffer[top & (WF_PAR_LANE_SLOTS - 1)]
    );
    expected = (LONG64)top;
    if (!wf__par_compare_exchange64_seq(
            &victim->top,
            &expected,
            (LONG64)(top + 1)
        )) {
        return NULL;
    }
#if defined(WF_PAR_IDENTITY_PROBE)
    thief->steals += 1;
#else
    (void)thief;
#endif
    return slot;
}

static struct wf__par_slot *wf__par_find(struct wf__par_lane *lane) {
    int count = (int)wf__par_load32_relaxed(&wf__par_lane_count);
    int offset;
    int step;
    if (count < 2) {
        return NULL;
    }
    lane->seed = lane->seed * UINT64_C(6364136223846793005)
        + UINT64_C(1442695040888963407);
    offset = (int)((lane->seed >> 33) % (uint64_t)count);
    for (step = 0; step < count; step += 1) {
        int index = offset + step;
        struct wf__par_lane *victim;
        struct wf__par_slot *slot;
        if (index >= count) {
            index -= count;
        }
        victim = &wf__par_lanes[index];
        if (victim == lane) {
            continue;
        }
        slot = wf__par_steal(lane, victim);
        if (slot != NULL) {
            return slot;
        }
    }
    return NULL;
}

static void wf__par_execute(struct wf__par_slot *slot) {
    slot->run(slot->frame);
#if defined(WF_PAR_IDENTITY_PROBE)
    if (wf__par_self != NULL) {
        wf__par_self->executions += 1;
    }
#endif
    wf__par_store32_seq(&slot->state, WF_PAR_SLOT_DONE);
    WakeByAddressAll((PVOID)&slot->state);
}

/* ---------------------------------------------------------------- waiting */

static void wf__par_wait(
    struct wf__par_lane *lane,
    struct wf__par_slot *target
) {
    int rounds = 0;
    for (;;) {
        struct wf__par_slot *slot;
        LONG pending = WF_PAR_SLOT_PENDING;
        if (wf__par_load32_acquire(&target->state) == WF_PAR_SLOT_DONE) {
            return;
        }
        if (WF_PAR_WRITER_HELP_ONCE()) {
            rounds = 0;
            continue;
        }
        slot = wf__par_pop(lane);
        if (slot == NULL) {
            slot = wf__par_find(lane);
        }
        if (slot != NULL) {
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS) {
            rounds += 1;
            YieldProcessor();
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
            rounds += 1;
            (void)SwitchToThread();
            continue;
        }
        if (WaitOnAddress(
                (volatile VOID *)&target->state,
                &pending,
                sizeof(pending),
                INFINITE
            ) == FALSE) {
            wf__par_fatal("WaitOnAddress failed while joining a task");
        }
        rounds = 0;
    }
}

/* ------------------------------------------------------------------ lanes */

static unsigned __stdcall wf__par_worker_main(void *opaque) {
    struct wf__par_lane *lane = (struct wf__par_lane *)opaque;
    int rounds = 0;
    wf__par_self = lane;
    wf__par_attached = 1;
    wf__floor_attach_thread();

    (void)wf__par_increment32(&wf__par_ready_workers);
    WakeByAddressAll((PVOID)&wf__par_ready_workers);

    for (;;) {
        struct wf__par_slot *slot;
        uint64_t bit;
        LONG observed_generation;
        if (WF_PAR_WRITER_HELP_ONCE()) {
            rounds = 0;
            continue;
        }
        slot = wf__par_find(lane);
        if (slot != NULL) {
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS) {
            rounds += 1;
            YieldProcessor();
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
            rounds += 1;
            (void)SwitchToThread();
            continue;
        }

        bit = UINT64_C(1) << (unsigned)(lane - wf__par_lanes);
        /* Capture before publishing the idle bit. A publisher either misses
         * the bit and is seen by the final deque scan, or claims the bit and
         * changes this generation, making the wait return immediately. */
        observed_generation = wf__par_load32_relaxed(
            &lane->wake_generation
        );
        (void)wf__par_fetch_or64_seq(&wf__par_idle, (LONG64)bit);
        slot = wf__par_find(lane);
        if (slot != NULL) {
            (void)wf__par_fetch_and64_seq(
                &wf__par_idle,
                (LONG64)~bit
            );
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (WF_PAR_WRITER_HELP_ONCE()) {
            (void)wf__par_fetch_and64_seq(
                &wf__par_idle,
                (LONG64)~bit
            );
            rounds = 0;
            continue;
        }
        wf__par_park(lane, observed_generation);
        (void)wf__par_fetch_and64_seq(&wf__par_idle, (LONG64)~bit);
        rounds = 0;
    }
}

static void wf__par_prepare(struct wf__par_lane *lane, int index) {
    int slot;
    lane->top = 0;
    lane->bottom = 0;
    lane->seed = UINT64_C(0x9e3779b97f4a7c15)
        * (uint64_t)(index + 1);
    lane->wake_generation = 0;
#if defined(WF_PAR_IDENTITY_PROBE)
    lane->steals = 0;
    lane->executions = 0;
#endif
    for (slot = 0; slot < WF_PAR_LANE_SLOTS; slot += 1) {
        lane->slots[slot].home = lane;
        lane->slots[slot].state = WF_PAR_SLOT_FREE;
        lane->slots[slot].next_free = slot + 1;
    }
    lane->slots[WF_PAR_LANE_SLOTS - 1].next_free = -1;
    lane->free_head = 0;
}

static int wf__par_default_lanes(void) {
    DWORD online = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
    if (online < 2) {
        wf__par_fatal("Windows reported fewer than two active processors");
    }
    if (online > WF_PAR_MAX_LANES) {
        wf__par_fatal(
            "active processor count exceeds 64; set WF_WORKERS from 2 through 64"
        );
    }
    return (int)online;
}

static int wf__par_requested_lanes(void) {
    const char *setting = getenv("WF_WORKERS");
    char *end = NULL;
    long requested;
    if (setting == NULL) {
        return wf__par_default_lanes();
    }
    errno = 0;
    requested = strtol(setting, &end, 10);
    if (errno == ERANGE || end == setting || *end != '\0'
        || requested < 2 || requested > WF_PAR_MAX_LANES) {
        wf__par_fatal("WF_WORKERS must be an integer from 2 through 64");
    }
    return (int)requested;
}

static BOOL CALLBACK wf__par_start_once(
    PINIT_ONCE once,
    PVOID parameter,
    PVOID *context
) {
    int requested = wf__par_requested_lanes();
    size_t stack_bytes = wf__floor_stack_bytes();
    int index;
    (void)once;
    (void)parameter;
    (void)context;

    if (stack_bytes == 0 || stack_bytes > (size_t)UINT_MAX) {
        wf__par_fatal("worker stack reservation does not fit _beginthreadex");
    }
    for (index = 0; index < requested; index += 1) {
        wf__par_prepare(&wf__par_lanes[index], index);
    }
    wf__par_store32_relaxed(&wf__par_lane_count, (LONG)requested);

    for (index = 1; index < requested; index += 1) {
        uintptr_t thread_value = _beginthreadex(
            NULL,
            (unsigned)stack_bytes,
            wf__par_worker_main,
            &wf__par_lanes[index],
            STACK_SIZE_PARAM_IS_A_RESERVATION,
            NULL
        );
        HANDLE thread;
        if (thread_value == 0) {
            wf__par_fatal("could not create every requested worker thread");
        }
        thread = (HANDLE)thread_value;
        if (CloseHandle(thread) == FALSE) {
            wf__par_fatal("could not detach a worker thread handle");
        }
    }

    {
        LONG target = (LONG)(requested - 1);
        LONG observed = wf__par_load32_acquire(&wf__par_ready_workers);
        while (observed != target) {
            if (WaitOnAddress(
                    (volatile VOID *)&wf__par_ready_workers,
                    &observed,
                    sizeof(observed),
                    INFINITE
                ) == FALSE) {
                wf__par_fatal("worker readiness rendezvous failed");
            }
            observed = wf__par_load32_acquire(&wf__par_ready_workers);
        }
    }
    return TRUE;
}

static struct wf__par_lane *wf__par_attach(void) {
    static volatile LONG taken;
    LONG expected = 0;
    wf__par_attached = 1;
    if (InitOnceExecuteOnce(
            &wf__par_started,
            wf__par_start_once,
            NULL,
            NULL
        ) == FALSE) {
        wf__par_fatal("InitOnceExecuteOnce could not initialize the pool");
    }
    if (wf__par_load32_relaxed(&wf__par_lane_count) < 2) {
        wf__par_fatal("the initialized pool has no worker lane");
    }
    if (!wf__par_compare_exchange32(&taken, &expected, 1)) {
        wf__par_fatal("a non-scheduler thread attempted to acquire lane zero");
    }
    wf__par_self = &wf__par_lanes[0];
    wf__floor_attach_thread();
    return wf__par_self;
}

#if defined(WF_PAR_WITH_WRITER_SCHEDULER)
void wf__writer_scheduler_prepare_lanes(void) {
    if (wf__par_self == NULL && !wf__par_attached) {
        (void)wf__par_attach();
    }
}

void wf__writer_scheduler_wake_lane(void) {
    if (wf__par_load64_seq(&wf__par_idle) != 0) {
        wf__par_wake_one();
    }
}
#endif

/* -------------------------------------------------------- emitted-module ABI */

void *wf__par_claim(uint64_t bytes) {
    struct wf__par_lane *lane = wf__par_self;
    struct wf__par_slot *slot;
    int index;
    if (bytes > (uint64_t)WF_PAR_FRAME_BYTES) {
        wf__par_fatal("published frame exceeds the 256-byte runtime contract");
    }
    if (lane == NULL) {
        if (wf__par_attached) {
            wf__par_fatal("an attached scheduler thread has no lane");
        }
        lane = wf__par_attach();
    }
    index = lane->free_head;
    if (index < 0) {
        /* Required work-first backpressure, not backend degradation. */
        return NULL;
    }
    slot = &lane->slots[index];
    lane->free_head = slot->next_free;
    return slot->frame;
}

void wf__par_publish(void *frame, void (*fn)(void *)) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    slot->run = fn;
    wf__par_store32_relaxed(&slot->state, WF_PAR_SLOT_PENDING);
    wf__par_push(slot->home, slot);
    if (wf__par_load64_seq(&wf__par_idle) != 0) {
        wf__par_wake_one();
    }
}

int wf__par_help_once(void) {
    struct wf__par_lane *lane = wf__par_self;
    struct wf__par_slot *slot;
    if (WF_PAR_WRITER_HELP_ONCE()) {
        return 1;
    }
    if (lane == NULL) {
        return 0;
    }
    slot = wf__par_pop(lane);
    if (slot == NULL) {
        return 0;
    }
    wf__par_execute(slot);
    return 1;
}

void wf__par_join(void *frame) {
    struct wf__par_slot *target = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = target->home;
    struct wf__par_slot *slot = wf__par_pop(lane);

    if (slot == target) {
        target->run(target->frame);
        return;
    }
    while (slot != NULL) {
        wf__par_execute(slot);
        if (wf__par_load32_acquire(&target->state) == WF_PAR_SLOT_DONE) {
            return;
        }
        slot = wf__par_pop(lane);
        if (slot == target) {
            target->run(target->frame);
            return;
        }
    }
    wf__par_wait(lane, target);
}

void wf__par_release(void *frame) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = slot->home;
    wf__par_store32_relaxed(&slot->state, WF_PAR_SLOT_FREE);
    slot->next_free = lane->free_head;
    lane->free_head = (int)(slot - lane->slots);
}

int wf__par_pool_active(void) {
    (void)wf__par_requested_lanes();
    return 1;
}

static volatile LONG wf__par_cached_lanes;

static int wf__par_lanes_once(void) {
    LONG lanes = wf__par_load32_relaxed(&wf__par_cached_lanes);
    if (lanes == 0) {
        LONG desired = (LONG)wf__par_requested_lanes();
        LONG expected = 0;
        (void)wf__par_compare_exchange32(
            &wf__par_cached_lanes,
            &expected,
            desired
        );
        lanes = wf__par_load32_relaxed(&wf__par_cached_lanes);
    }
    return (int)lanes;
}

uint64_t wf__par_split_budget(uint64_t span, uint64_t weight) {
    struct wf__par_lane *lane = wf__par_self;
    int lanes = wf__par_lanes_once();
    uint64_t want;
    uint64_t affordable;
    uint64_t chunks;
    uint64_t budget;
    if (lane != NULL) {
        uint64_t bottom = (uint64_t)wf__par_load64_relaxed(&lane->bottom);
        uint64_t top = (uint64_t)wf__par_load64_relaxed(&lane->top);
        if ((int64_t)(bottom - top) > 0) {
            return 0;
        }
    }
    if (weight == 0) {
        weight = 1;
    }
    want = (uint64_t)lanes * WF_PAR_SPLIT_OVERSUBSCRIBE;
    if (weight >= WF_PAR_SPLIT_WORK_PER_CHUNK) {
        affordable = span;
    } else {
        affordable = span
            / ((WF_PAR_SPLIT_WORK_PER_CHUNK + weight - 1) / weight);
    }
    chunks = want < affordable ? want : affordable;
    budget = 0;
    while ((chunks >> 1) != 0) {
        chunks >>= 1;
        budget += 1;
    }
    return budget;
}

#if defined(WF_PAR_IDENTITY_PROBE)
unsigned long wf__par_started_worker_count(void) {
    return (unsigned long)wf__par_load32_acquire(&wf__par_ready_workers);
}

unsigned long wf__par_grant_count(void) {
    return wf__par_grant_total();
}

unsigned long wf__par_worker_execution_count(void) {
    return wf__par_worker_execution_total();
}
#endif
