/* Test-only scheduling control. The real pool, frame, thunk and join execute;
 * withholding the offering thread's join lets a real worker enter one task.
 * Retire this helper when no native test needs a selected worker execution. */
#define _POSIX_C_SOURCE 200809L
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

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
static pthread_t schedule_offerer;

/* These controls may only bracket a fully joined caller, never an in-flight
 * task. Scatter uses them to observe output packing rather than an earlier
 * count/partition stage. Other fixtures observe their first publication. */
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
        pthread_equal(pthread_self(), schedule_offerer)) abort();
    atomic_store_explicit(&schedule_entered, 1, memory_order_release);
    schedule_run(frame);
}

void wf_test_worker_publish(void *frame, void (*run)(void *)) {
    unsigned unclaimed = 0;
    if (atomic_load(&schedule_enabled) &&
        atomic_compare_exchange_strong(&schedule_claimed, &unclaimed, 1)) {
        if (!wf__sched_pool_running()) abort();
        schedule_run = run;
        schedule_offerer = pthread_self();
        atomic_store_explicit(&schedule_frame, frame, memory_order_release);
        wf__par_publish(frame, schedule_observed_run);
    } else {
        wf__par_publish(frame, run);
    }
}

void wf_test_worker_join(void *frame) {
    if (frame == atomic_load_explicit(&schedule_frame, memory_order_acquire)) {
        struct timespec start, now;
        if (clock_gettime(CLOCK_MONOTONIC, &start)) abort();
        while (!atomic_load_explicit(&schedule_entered, memory_order_acquire)) {
            if (clock_gettime(CLOCK_MONOTONIC, &now)) abort();
            if (now.tv_sec - start.tv_sec >= 5) {
                fputs("worker schedule: no worker entered the published task\n", stderr);
                exit(111);
            }
            sched_yield();
        }
    }
    wf__par_join(frame);
}
