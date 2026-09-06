/* The seven primitives the scheduler core reaches shared state through
 * (PARK-ON-MISS.md §7.1), and nothing else.
 *
 *   1. atomic load, store and compare-exchange, each with its ordering;
 *   2. the stack switch, with the floor bounds it writes on the way;
 *   3. the sleep and the wake on the one epoch;
 *   4. the stack reservation, made once at the core's entry;
 *   5. lock and unlock of the core's one mutex;
 *   6. the yield the exhausted compute arm keeps, and the pause of a spin;
 *   7. target progress and the drain.
 *
 * Three implementations, selected at compile time and never at run time. The
 * host implementation (`prim_host.c`) is the POSIX runtime's: C11 atomics
 * inline, a hand-written switch, an epoch park on a mutex and condition
 * variable, an `mmap` reservation with guard pages. `prim_windows.c` is the
 * same set on fibers, an SRWLOCK and a condition variable, and a `VirtualAlloc`
 * reservation. The enumerator's replacement (`enumerate.c`, compiled with
 * `WF_SCHED_ENUMERATE`) supplies every one of these names as a call into a
 * controlled scheduler, so the same `core.c` runs under every interleaving of
 * primitive steps (§11).
 *
 * Thread identity is not shared state, so it is not one of the seven; it is
 * the platform's, supplied here beside them -- and so are the three platform
 * facts at the end of this header, which the platform layer over the core
 * (`entry.c`) needs and which no unit above it may ask the host for directly.
 * That is what lets `entry.c` carry one startup policy for every platform with
 * no `#if` of its own. */

#ifndef WHITEFOOT_SCHED_PRIM_H
#define WHITEFOOT_SCHED_PRIM_H

#include <stddef.h>
#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

struct wf_sched_core;

/* Memory orderings, named so the enumerator can record them beside each
 * step even though it schedules sequentially consistently. */
enum wf_prim_order {
    WF_PRIM_RELAXED = 0,
    WF_PRIM_ACQUIRE = 1,
    WF_PRIM_RELEASE = 2,
    WF_PRIM_ACQ_REL = 3,
    WF_PRIM_SEQ_CST = 4
};

/* Every primitive violation the core can detect ends the run here: the host
 * aborts, the enumerator records the failure and stops the execution. */
void wf_prim_fail(const char *reason);

/* What a critical section of the one mutex is for (item 5). The host ignores
 * it; the enumerator classifies the section by it: a push always writes its
 * list, a pop writes only when the list is non-empty, and two sections that
 * only read commute, which is what keeps two idle threads' polling from
 * multiplying the interleavings to walk. */
enum wf_prim_section {
    WF_PRIM_SECTION_FREE_POP = 0,
    WF_PRIM_SECTION_FREE_PUSH = 1,
    WF_PRIM_SECTION_READY_POP = 2,
    WF_PRIM_SECTION_READY_PUSH = 3
};

/* The calling thread's index in the core, 0 for the entry thread. Set once
 * by `wf_sched_run` before the thread's first switch. */
unsigned wf_prim_thread_index(void);
void wf_prim_set_thread_index(unsigned index);

#if defined(WF_SCHED_ENUMERATE)

/* ---------------------------------------------- the enumerator's set */

unsigned wf_prim_load_u(const unsigned *word, enum wf_prim_order order);
void wf_prim_store_u(unsigned *word, unsigned value, enum wf_prim_order order);
int wf_prim_cas_u(
    unsigned *word,
    unsigned *expected,
    unsigned desired,
    enum wf_prim_order success,
    enum wf_prim_order failure
);
void *wf_prim_load_p(void *const *word, enum wf_prim_order order);
void wf_prim_store_p(void **word, void *value, enum wf_prim_order order);
int wf_prim_cas_p(
    void **word,
    void **expected,
    void *desired,
    enum wf_prim_order success,
    enum wf_prim_order failure
);
unsigned long long wf_prim_load_q(const unsigned long long *word, enum wf_prim_order order);
void wf_prim_store_q(unsigned long long *word, unsigned long long value, enum wf_prim_order order);
int wf_prim_cas_q(
    unsigned long long *word,
    unsigned long long *expected,
    unsigned long long desired,
    enum wf_prim_order success,
    enum wf_prim_order failure
);
/* The two read-modify-writes of item 1, for the idle bitmap: one thread's bit
 * goes up or down in one step, so two threads' bits commute. */
unsigned long long wf_prim_or_q(unsigned long long *word, unsigned long long bits, enum wf_prim_order order);
unsigned long long wf_prim_and_q(unsigned long long *word, unsigned long long bits, enum wf_prim_order order);

#else

/* ------------------------------------------------- the host's set (1) */

static inline int wf_prim_host_order(enum wf_prim_order order) {
    switch (order) {
    case WF_PRIM_RELAXED:
        return __ATOMIC_RELAXED;
    case WF_PRIM_ACQUIRE:
        return __ATOMIC_ACQUIRE;
    case WF_PRIM_RELEASE:
        return __ATOMIC_RELEASE;
    case WF_PRIM_ACQ_REL:
        return __ATOMIC_ACQ_REL;
    case WF_PRIM_SEQ_CST:
    default:
        return __ATOMIC_SEQ_CST;
    }
}

/* The builtins take the ordering as a constant, so each primitive spells the
 * five cases out once. */
#define WF_PRIM_DEFINE_LOAD(name, type)                                        \
    static inline type name(const type *word, enum wf_prim_order order) {     \
        switch (order) {                                                       \
        case WF_PRIM_RELAXED:                                                  \
            return __atomic_load_n(word, __ATOMIC_RELAXED);                    \
        case WF_PRIM_ACQUIRE:                                                  \
        case WF_PRIM_ACQ_REL:                                                  \
        case WF_PRIM_RELEASE:                                                  \
            return __atomic_load_n(word, __ATOMIC_ACQUIRE);                    \
        case WF_PRIM_SEQ_CST:                                                  \
        default:                                                               \
            return __atomic_load_n(word, __ATOMIC_SEQ_CST);                    \
        }                                                                      \
    }
#define WF_PRIM_DEFINE_STORE(name, type)                                       \
    static inline void name(type *word, type value, enum wf_prim_order order) { \
        switch (order) {                                                       \
        case WF_PRIM_RELAXED:                                                  \
            __atomic_store_n(word, value, __ATOMIC_RELAXED);                   \
            return;                                                            \
        case WF_PRIM_RELEASE:                                                  \
        case WF_PRIM_ACQ_REL:                                                  \
        case WF_PRIM_ACQUIRE:                                                  \
            __atomic_store_n(word, value, __ATOMIC_RELEASE);                   \
            return;                                                            \
        case WF_PRIM_SEQ_CST:                                                  \
        default:                                                               \
            __atomic_store_n(word, value, __ATOMIC_SEQ_CST);                   \
            return;                                                            \
        }                                                                      \
    }
#define WF_PRIM_DEFINE_CAS(name, type)                                         \
    static inline int name(                                                    \
        type *word,                                                            \
        type *expected,                                                        \
        type desired,                                                          \
        enum wf_prim_order success,                                            \
        enum wf_prim_order failure                                             \
    ) {                                                                        \
        (void)failure;                                                         \
        switch (success) {                                                     \
        case WF_PRIM_RELAXED:                                                  \
            return __atomic_compare_exchange_n(                                \
                word, expected, desired, 0, __ATOMIC_RELAXED, __ATOMIC_RELAXED \
            );                                                                 \
        case WF_PRIM_ACQUIRE:                                                  \
            return __atomic_compare_exchange_n(                                \
                word, expected, desired, 0, __ATOMIC_ACQUIRE, __ATOMIC_ACQUIRE \
            );                                                                 \
        case WF_PRIM_RELEASE:                                                  \
            return __atomic_compare_exchange_n(                                \
                word, expected, desired, 0, __ATOMIC_RELEASE, __ATOMIC_RELAXED \
            );                                                                 \
        case WF_PRIM_ACQ_REL:                                                  \
            return __atomic_compare_exchange_n(                                \
                word, expected, desired, 0, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE \
            );                                                                 \
        case WF_PRIM_SEQ_CST:                                                  \
        default:                                                               \
            return __atomic_compare_exchange_n(                                \
                word, expected, desired, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST \
            );                                                                 \
        }                                                                      \
    }

WF_PRIM_DEFINE_LOAD(wf_prim_load_u, unsigned)
WF_PRIM_DEFINE_STORE(wf_prim_store_u, unsigned)
WF_PRIM_DEFINE_CAS(wf_prim_cas_u, unsigned)
WF_PRIM_DEFINE_LOAD(wf_prim_load_q, unsigned long long)
WF_PRIM_DEFINE_STORE(wf_prim_store_q, unsigned long long)
WF_PRIM_DEFINE_CAS(wf_prim_cas_q, unsigned long long)

static inline unsigned long long wf_prim_or_q(unsigned long long *word, unsigned long long bits, enum wf_prim_order order) {
    (void)order;
    return __atomic_fetch_or(word, bits, __ATOMIC_SEQ_CST);
}

static inline unsigned long long wf_prim_and_q(unsigned long long *word, unsigned long long bits, enum wf_prim_order order) {
    (void)order;
    return __atomic_fetch_and(word, bits, __ATOMIC_SEQ_CST);
}

static inline void *wf_prim_load_p(void *const *word, enum wf_prim_order order) {
    return (void *)__atomic_load_n(
        (const uintptr_t *)word, wf_prim_host_order(order) == __ATOMIC_RELAXED
            ? __ATOMIC_RELAXED
            : (wf_prim_host_order(order) == __ATOMIC_SEQ_CST ? __ATOMIC_SEQ_CST : __ATOMIC_ACQUIRE)
    );
}
static inline void wf_prim_store_p(void **word, void *value, enum wf_prim_order order) {
    __atomic_store_n(
        (uintptr_t *)word, (uintptr_t)value,
        wf_prim_host_order(order) == __ATOMIC_RELAXED
            ? __ATOMIC_RELAXED
            : (wf_prim_host_order(order) == __ATOMIC_SEQ_CST ? __ATOMIC_SEQ_CST : __ATOMIC_RELEASE)
    );
}
static inline int wf_prim_cas_p(
    void **word,
    void **expected,
    void *desired,
    enum wf_prim_order success,
    enum wf_prim_order failure
) {
    uintptr_t seen = (uintptr_t)*expected;
    int done;
    (void)failure;
    done = __atomic_compare_exchange_n(
        (uintptr_t *)word, &seen, (uintptr_t)desired, 0,
        wf_prim_host_order(success) == __ATOMIC_RELAXED ? __ATOMIC_RELAXED
            : (wf_prim_host_order(success) == __ATOMIC_RELEASE ? __ATOMIC_RELEASE
                : (wf_prim_host_order(success) == __ATOMIC_SEQ_CST ? __ATOMIC_SEQ_CST : __ATOMIC_ACQ_REL)),
        __ATOMIC_ACQUIRE
    );
    *expected = (void *)seen;
    return done;
}

#endif

/* ------------------------------------------------------- the rest (2-7) */

/* 2. The switch. Spills the callee-saved state of the calling stack, stores
 * its stack pointer through `save`, adopts `load`, and returns into it. A
 * stack prepared by `wf_prim_prepare_stack` runs `entry(argument)` at its
 * first switch and must never return from it. `wf_prim_set_bounds` is the
 * floor's per-stack half, written by the core before every switch from the
 * reservation record.
 *
 * `top` and `bytes` are the slot's own top address and its size, both from
 * the reservation record. The host prepares a frame at `top` and ignores the
 * size, because the slot *is* the stack; Windows makes a fiber, which owns a
 * stack of its own, and needs the size to ask for one (design §7, platform
 * item 3). What the core stores is opaque either way: a stack pointer on the
 * host, a fiber handle on Windows. */
void wf_prim_switch(void **save, void *load);
void *wf_prim_prepare_stack(
    void *top,
    size_t bytes,
    void (*entry)(void *),
    void *argument
);
void wf_prim_set_bounds(unsigned char *low, unsigned char *high);

/* 3. The one sleep. `wf_prim_epoch` reads the wake epoch; `wf_prim_park`
 * sleeps while the epoch still equals `observed`; `wf_prim_wake` raises the
 * epoch and wakes every sleeper. */
uint64_t wf_prim_epoch(void);
void wf_prim_park(uint64_t observed);
void wf_prim_wake(void);

/* The seam through which the owner of the host's wait set supplies that one
 * sleep and wake (design §7, platform item 2).  The host implementation
 * declares all three weak and answers zero, keeping its own epoch; a link that
 * carries a completion runtime defines them strongly, so the core and the
 * bridge sleep and wake on one primitive.  Each returns nonzero when it
 * handled the call.  They are not an eighth primitive: they are how item 3 is
 * bound to a platform. */
int wf__sched_host_epoch(uint64_t *epoch);
int wf__sched_host_park(uint64_t observed);
int wf__sched_host_wake(void);

/* 4. The reservation: `count` stacks of `bytes` each, each with a guard page
 * below it, in one mapping. Returns the base of the first stack's usable
 * range or NULL when the reservation cannot be made. */
unsigned char *wf_prim_reserve(unsigned count, size_t bytes);
size_t wf_prim_stack_stride(size_t bytes);

/* 5. The core's one mutex. The lock names the section it opens. */
void wf_prim_lock(enum wf_prim_section section);
void wf_prim_unlock(void);

/* 6. The yield the exhausted compute arm keeps, and the pause a bounded spin
 * puts between two of its looks. The pause reaches no shared state and can
 * observe nothing; it is a hint to the machine that these cycles are a wait,
 * and it is here with the yield because the core may name no instruction of
 * any platform. The host answers it with `pause` on x86 and `isb` on aarch64,
 * Windows with `YieldProcessor`, and the enumerator with nothing, because a
 * step that neither reads nor writes shared state is not a step it schedules.
 * `WF_SCHED_IDLE_SPIN_ROUNDS` in `core.h` is the one caller. */
void wf_prim_yield(void);
void wf_prim_pause(void);

/* 7. Target progress and the drain: flush the deferred doorbell, run one
 * bounded target pass, and deliver every ready completion through
 * `wf_sched_complete`. Returns nonzero when it made progress. May block. */
int wf_prim_progress(struct wf_sched_core *core);

/* -------------------------------------- the platform layer's own four */

/* Not primitives: the core reaches no shared state through any of these and
 * the enumerator never calls one. They are the parts of design §7's platform
 * layer that the layer *over* the core needs -- thread creation with a
 * reserved host stack (item 1), the thread's attach and detach for the switch
 * (item 3), the machine's own core count, and the read of one runtime setting
 * -- stated here so `entry.c` carries one startup policy on every platform.
 *
 * P1. Creates one detached thread running `entry(argument)` on a host stack of
 * `stack_bytes`, or the platform's own default when that is zero. Returns 0
 * on success.
 *
 * `thread` is the caller's record, and it holds the only two words the new
 * thread reads before it runs anything: the entry and its argument. It is the
 * caller's rather than this primitive's for one reason, and it is the rule
 * this runtime is built on -- nothing allocates at run time, and a pool starts
 * at the first hand-out, which is run time. So the pair cannot come from an
 * allocation here; and it cannot live on the creating frame either, which may
 * return before the new thread has read it. The caller keeps the record alive
 * for the whole life of the thread, and every caller has somewhere that
 * already lives that long: the core's own per-thread storage, the adapter's
 * own record, a probe's own static.
 *
 * The stack this reserves is the thread's *host* stack and is not a Whitefoot
 * stack: the thread switches to a pool stack before it runs anything, and its
 * scheduler loop lives there (§5). POSIX reserves it with
 * `pthread_attr_setstacksize`; Windows with `_beginthreadex` and
 * STACK_SIZE_PARAM_IS_A_RESERVATION. A refused reservation is a failure here
 * rather than a silent downgrade to the platform default. */
typedef struct wf_prim_thread {
    void (*entry)(void *);
    void *argument;
} wf_prim_thread;

int wf_prim_thread_start(
    wf_prim_thread *thread,
    void (*entry)(void *),
    void *argument,
    size_t stack_bytes
);

/* P2. The other half of item 3. `wf_prim_thread_attach` runs on a thread
 * before its first switch and `wf_prim_thread_detach` at the entry thread's
 * second and last entry to its host stack (§5). Windows is
 * `ConvertThreadToFiber` and `ConvertFiberToThread`, because a Windows
 * thread's host stack has to become a fiber before anything can switch back
 * to it; the host's hand-written switch needs neither and both do nothing
 * there. */
void wf_prim_thread_attach(void);
void wf_prim_thread_detach(void);

/* P2's second name: the floor's per-thread half, reached through this layer
 * as the per-stack half is reached through `wf_prim_set_bounds`. `entry.c`
 * calls it once at each worker's start, and the floor calls its own attach on
 * the entry thread before the core starts. On the host it is the alternate
 * signal stack the guard-page handler runs on; on Windows it is the stack
 * guarantee, which that leaf also arms on every pool fiber it starts, because
 * Windows keeps a guarantee per thread or fiber. The platform leaf is the one
 * unit of a link that names the floor's attach and the one that carries its
 * weak answer for a core linked without the floor. */
void wf_prim_floor_attach(void);

/* P3. How many CPUs this process may be scheduled on right now, or 0 when the
 * platform will not say. `sysctlbyname`/`sysconf` on the host,
 * `GetActiveProcessorCount` on Windows. */
unsigned wf_prim_online_cpus(void);

/* P4. The text of one runtime setting, copied into the caller's buffer.
 *
 * Answers 1 when the setting is set and its text -- which may be empty --
 * fits, 0 when it is not set, and -1 when it is set but the caller's buffer
 * cannot hold it. `buffer[0]` is written in every case, so a caller that only
 * asks whether a setting is set never reads uninitialized bytes. The -1 answer
 * is the honest one for a value this runtime cannot represent: truncating
 * would silently turn a value the caller would have refused into one it
 * accepts, and every caller here treats "set but not delivered" the way it
 * already treats a value it cannot use. A malformed call -- no name, no
 * buffer, or no capacity -- is the same answer for the same reason.
 *
 * It is behind this seam rather than in the shared units for one reason:
 * `getenv` is the C standard library on POSIX and mingw, and the MSVC ucrt
 * marks it deprecated, so a shared unit that names it does not compile on the
 * shipped Windows toolchain. POSIX answers with `getenv`, Windows with
 * `GetEnvironmentVariableA`. Every setting name and setting value this runtime
 * reads is ASCII, which is what makes the A form exact rather than a narrowing
 * of the W one. */
#define WF_PRIM_SETTING_BYTES 64u

int wf_prim_setting_text(const char *name, char *buffer, size_t capacity);

#if defined(__cplusplus)
}
#endif

#endif
