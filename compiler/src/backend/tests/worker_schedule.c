/* Test-only scheduling control. The real pool, frame, thunk and join execute;
 * withholding the offering thread's join lets a real worker enter one task.
 * Retire this helper when no native test needs a selected worker execution. */
#define _POSIX_C_SOURCE 200809L
#if defined(_WIN32)
#include <windows.h>
typedef DWORD wf_test_thread;
#define wf_test_self() GetCurrentThreadId()
#define wf_test_equal(a, b) ((a) == (b))
#define wf_test_yield() SwitchToThread()
#else
#include <pthread.h>
#include <sched.h>
typedef pthread_t wf_test_thread;
#define wf_test_self() pthread_self()
#define wf_test_equal(a, b) pthread_equal(a, b)
#define wf_test_yield() sched_yield()
#endif
#include <stdint.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
extern uint64_t wf_prim_monotonic_us(void);

extern void wf__par_publish(void *, void (*)(void *));
extern void wf__par_join(void *);
extern unsigned wf__sched_pool_running(void);

#ifdef WF_TEST_SCHEDULE_MANUAL
static _Atomic unsigned schedule_enabled;
#else
static _Atomic unsigned schedule_enabled = 1;
#endif
static _Atomic unsigned schedule_claimed, schedule_entered;
static _Atomic(void *) schedule_frame;
static void (*schedule_run)(void *);
static wf_test_thread schedule_offerer;

/* These controls may only bracket a fully joined caller, never an in-flight
 * task. Scatter resets them between input partitioning and output packing
 * to observe both phases. Other fixtures observe their first publication. */
void wf_test_worker_schedule_begin(void) {
    atomic_store(&schedule_frame, NULL);
    atomic_store(&schedule_claimed, 0);
    atomic_store(&schedule_entered, 0);
    atomic_store(&schedule_enabled, 1);
}

void wf_test_worker_schedule_end(void) {
    atomic_store(&schedule_enabled, 0);
}

static void schedule_observed_run(void *frame) {
    if (frame != atomic_load(&schedule_frame) ||
        wf_test_equal(wf_test_self(), schedule_offerer)) abort();
    atomic_store_explicit(&schedule_entered, 1, memory_order_release);
    schedule_run(frame);
}

void wf_test_worker_publish(void *frame, void (*run)(void *)) {
    unsigned unclaimed = 0;
    if (atomic_load(&schedule_enabled) &&
        atomic_compare_exchange_strong(&schedule_claimed, &unclaimed, 1)) {
        if (!wf__sched_pool_running()) abort();
        schedule_run = run;
        schedule_offerer = wf_test_self();
        atomic_store_explicit(&schedule_frame, frame, memory_order_release);
        wf__par_publish(frame, schedule_observed_run);
    } else {
        wf__par_publish(frame, run);
    }
}

void wf_test_worker_join(void *frame) {
    if (frame == atomic_load_explicit(&schedule_frame, memory_order_acquire)) {
        const uint64_t deadline = wf_prim_monotonic_us() + 5000000;
        while (!atomic_load_explicit(&schedule_entered, memory_order_acquire)) {
            if (wf_prim_monotonic_us() >= deadline) {
                fputs("worker schedule: no worker entered the published task\n", stderr);
                exit(111);
            }
            wf_test_yield();
        }
    }
    wf__par_join(frame);
}
