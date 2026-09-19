/* Calls the exact assembly measured by stack_ledger.rs. Runtime depth and
 * observed real-worker entry make one image serve all boundary executions. */
#define _GNU_SOURCE
#if defined(_WIN32)
#define _WIN32_WINNT 0x0602
#include <windows.h>
#include <io.h>
#include <fcntl.h>
typedef DWORD wf_boundary_thread;
#define wf_boundary_self() GetCurrentThreadId()
#define wf_boundary_equal(a, b) ((a) == (b))
#define wf_boundary_yield() SwitchToThread()
#else
#include <pthread.h>
#include <sched.h>
#include <unistd.h>
typedef pthread_t wf_boundary_thread;
#define wf_boundary_self() pthread_self()
#define wf_boundary_equal(a, b) pthread_equal(a, b)
#define wf_boundary_yield() sched_yield()
#endif
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifndef WF_TEST_WIDE
#define WF_TEST_WIDE 0
#endif
#if WF_TEST_WIDE
extern uint64_t wf_spine(uint64_t, uint64_t, uint8_t);
#else
extern double wf_spine(uint64_t, double);
#endif
#ifndef WF_TEST_STACK_BYTES
#define WF_TEST_STACK_BYTES (1024u * 1024u)
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
static wf_boundary_thread initial, owner;
static _Atomic int finished;

static void boundary_call(void *unused) {
    (void)unused;
    volatile unsigned char marker = 0;
    size_t size;
    uintptr_t low;
    unsigned long page;
    size_t guarantee = 0;
    if (on_worker && wf_boundary_equal(wf_boundary_self(), owner)) _Exit(81);
#if defined(_WIN32)
    ULONG_PTR stack_low, stack_high;
    ULONG reserved = 0;
    SYSTEM_INFO system;
    GetCurrentThreadStackLimits(&stack_low, &stack_high);
    if (!SetThreadStackGuarantee(&reserved) || reserved < 64u * 1024u) _Exit(87);
    low = (uintptr_t)stack_low;
    size = (size_t)(stack_high - stack_low);
    guarantee = reserved;
    GetSystemInfo(&system);
    page = system.dwPageSize;
#elif defined(__APPLE__)
    page = (unsigned long)sysconf(_SC_PAGESIZE);
    size = pthread_get_stacksize_np(pthread_self());
    low = (uintptr_t)pthread_get_stackaddr_np(pthread_self()) - size;
#else
    page = (unsigned long)sysconf(_SC_PAGESIZE);
    pthread_attr_t attr;
    void *base;
    if (pthread_getattr_np(pthread_self(), &attr)
        || pthread_attr_getstack(&attr, &base, &size)) _Exit(82);
    pthread_attr_destroy(&attr);
    low = (uintptr_t)base;
#endif
    uintptr_t here = (uintptr_t)&marker;
    if (here < low || here >= low + size || size < WF_TEST_STACK_BYTES
        || wf__floor_stack_bytes() != WF_TEST_STACK_BYTES
        || here - low <= guarantee) _Exit(83);
    puts(on_worker ? "worker" : "entry");
    if (query) {
        printf("room=%llu stack=%llu page=%lu\n", (unsigned long long)(here - low - guarantee),
               (unsigned long long)size, page);
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
    if (wf_boundary_equal(wf_boundary_self(), initial)) return 84;
    owner = wf_boundary_self();
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
        wf_boundary_yield();
    }
    wf__par_join(frame);
    wf__par_release(frame);
    return 0;
}
int main(int argc, char **argv) {
    if (argc != 3) return 2;
#if defined(_WIN32)
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
    if (_setmode(_fileno(stdout), _O_BINARY) < 0) return 88;
    if (!strcmp(argv[1], "abort-control")) abort();
#else
    alarm(10);
#endif
    query = !strncmp(argv[1], "bounds-", 7);
    on_worker = strstr(argv[1], "worker") != NULL;
    char *end;
    depth = strtoull(argv[2], &end, 10);
    if (*end) return 2;
    initial = wf_boundary_self();
    return wf__floor_run(argc, argv);
}
