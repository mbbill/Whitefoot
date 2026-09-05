/* The seven primitives the scheduler core reaches shared state through
 * (PARK-ON-MISS.md §7.1), and nothing else.
 *
 *   1. atomic load, store and compare-exchange, each with its ordering;
 *   2. the stack switch, with the floor bounds it writes on the way;
 *   3. the sleep and the wake on the one epoch;
 *   4. the stack reservation, made once at the core's entry;
 *   5. lock and unlock of the core's one mutex;
 *   6. the yield the exhausted compute arm keeps;
 *   7. target progress and the drain.
 *
 * Two implementations, selected at compile time and never at run time. The
 * host implementation below is the runtime's: C11 atomics inline, a
 * hand-written switch, an epoch park on a mutex and condition variable, an
 * `mmap` reservation with guard pages. The enumerator's replacement
 * (`enumerate.c`, compiled with `WF_SCHED_ENUMERATE`) supplies every one of
 * these names as a call into a controlled scheduler, so the same `core.c`
 * runs under every interleaving of primitive steps (§11).
 *
 * Thread identity is not shared state, so it is not one of the seven; it is
 * the platform's, supplied here beside them. */

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
 * reservation record. */
void wf_prim_switch(void **save, void *load);
void *wf_prim_prepare_stack(void *top, void (*entry)(void *), void *argument);
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

/* 6. The yield. */
void wf_prim_yield(void);

/* 7. Target progress and the drain: flush the deferred doorbell, run one
 * bounded target pass, and deliver every ready completion through
 * `wf_sched_complete`. Returns nonzero when it made progress. May block. */
int wf_prim_progress(struct wf_sched_core *core);

#if defined(__cplusplus)
}
#endif

#endif
