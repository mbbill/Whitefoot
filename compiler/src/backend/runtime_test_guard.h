#ifndef WHITEFOOT_RUNTIME_TEST_GUARD_H
#define WHITEFOOT_RUNTIME_TEST_GUARD_H

/* A bounded process guard for native test drivers on POSIX and Windows.
 * Keep it outside production units; retire when these drivers share a caller
 * that provides the same deadline and phase report. */
#include "sched/prim.h"
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#if !defined(_WIN32)
#include <time.h>
#endif

static wf_prim_thread wf_test_guard_thread;
static _Atomic int wf_test_guard_done;
static _Atomic(const char *) wf_test_guard_current = "startup";
static uint64_t wf_test_guard_deadline;

static inline void wf_test_guard_phase(const char *phase) {
    atomic_store(&wf_test_guard_current, phase);
}

static void wf_test_guard_wait(void *unused) {
    (void)unused;
    while (!atomic_load(&wf_test_guard_done)) {
        uint64_t now = wf_prim_monotonic_us();
        if (!now || now >= wf_test_guard_deadline) {
            fprintf(stderr, "runtime test: deadline exceeded in %s\n",
                    atomic_load(&wf_test_guard_current));
            fflush(stderr);
            _Exit(9);
        }
#if defined(_WIN32)
        Sleep(100);
#else
        struct timespec pause = {0, 100000000};
        (void)nanosleep(&pause, NULL);
#endif
    }
}

static inline void wf_test_guard_start(unsigned seconds) {
    uint64_t now = wf_prim_monotonic_us();
    if (!now) { fputs("runtime test: no monotonic clock\n", stderr); exit(2); }
    wf_test_guard_deadline = now + (uint64_t)seconds * 1000000;
    if (wf_prim_thread_start(&wf_test_guard_thread, wf_test_guard_wait, NULL, 0)) {
        fputs("runtime test: could not start deadline guard\n", stderr);
        exit(2);
    }
}

static inline void wf_test_guard_finish(void) {
    atomic_store(&wf_test_guard_done, 1);
}
#endif
