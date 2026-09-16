/* Calls the exact assembly measured by stack_ledger.rs. Runtime depth and
 * observed real-worker entry make one image serve all boundary executions. */
#define _GNU_SOURCE
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#ifndef WF_TEST_WIDE
#define WF_TEST_WIDE 0
#endif
#if WF_TEST_WIDE
extern uint64_t wf_spine(uint64_t, uint64_t, uint8_t);
#else
extern double wf_spine(uint64_t, double);
#endif
extern int wf__floor_run(int, char **);
extern size_t wf__floor_stack_bytes(void);
extern void *wf__par_acquire_lane(uint64_t);
extern void wf__par_publish(void *, void (*)(void *));
extern void wf__par_join(void *);
extern void wf__par_release(void *);
extern uint64_t wf_prim_monotonic_us(void);
static uint64_t depth;
static int query, on_worker;
static pthread_t initial, owner;
static _Atomic int finished;

static void boundary_call(void *unused) {
    (void)unused;
    volatile unsigned char marker = 0;
    size_t size;
    uintptr_t low;
    if (on_worker && pthread_equal(pthread_self(), owner)) _Exit(81);
#if defined(__APPLE__)
    size = pthread_get_stacksize_np(pthread_self());
    low = (uintptr_t)pthread_get_stackaddr_np(pthread_self()) - size;
#else
    pthread_attr_t attr;
    void *base;
    if (pthread_getattr_np(pthread_self(), &attr)
        || pthread_attr_getstack(&attr, &base, &size)) _Exit(82);
    pthread_attr_destroy(&attr);
    low = (uintptr_t)base;
#endif
    uintptr_t here = (uintptr_t)&marker;
    if (here < low || here >= low + size || size < 1024 * 1024
        || wf__floor_stack_bytes() != 1024 * 1024) _Exit(83);
    puts(on_worker ? "worker" : "entry");
    if (query) {
        printf("room=%llu stack=%llu page=%lu\n", (unsigned long long)(here - low),
               (unsigned long long)size, (unsigned long)sysconf(_SC_PAGESIZE));
    }
    fflush(stdout);
    if (!query) {
#if WF_TEST_WIDE
        volatile uint64_t answer = wf_spine(depth, 3, 1);
#else
        volatile double answer = wf_spine(depth, 1.0009765625);
#endif
        (void)answer;
    }
    atomic_store_explicit(&finished, 1, memory_order_release);
}
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    if (pthread_equal(pthread_self(), initial)) return 84;
    owner = pthread_self();
    if (!on_worker) {
        boundary_call(NULL);
        return 0;
    }
    void *frame = wf__par_acquire_lane(8);
    if (!frame) return 85;
    wf__par_publish(frame, boundary_call);
    uint64_t deadline = wf_prim_monotonic_us() + 5000000;
    /* The owner may not steal its own task while establishing this premise. */
    while (!atomic_load_explicit(&finished, memory_order_acquire)) {
        if (wf_prim_monotonic_us() >= deadline) return 86;
        sched_yield();
    }
    wf__par_join(frame);
    wf__par_release(frame);
    return 0;
}
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    alarm(10);
    query = !strncmp(argv[1], "bounds-", 7);
    on_worker = strstr(argv[1], "worker") != NULL;
    char *end;
    depth = strtoull(argv[2], &end, 10);
    if (*end) return 2;
    initial = pthread_self();
    return wf__floor_run(argc, argv);
}
