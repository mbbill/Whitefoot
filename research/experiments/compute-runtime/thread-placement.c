/* Linux measurement control for mandelbrot-command.sh profile, not a WF
 * runtime. Retire when the short-command CPU-placement question is settled.
 * Both bound and unbound runs use the same pthread_create wrapper. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

enum { participants = 4 };
static int cpus[participants], bind_threads, report;
static _Atomic unsigned created;
static struct {
    long tid;
    int cpu;
    _Atomic int ready;
} records[participants];
static int (*real_create)(pthread_t *, const pthread_attr_t *,
                          void *(*)(void *), void *);

static void fail(const char *message) {
    fprintf(stderr, "thread placement: %s\n", message);
    _Exit(2);
}

static void place(unsigned index) {
    if (index >= participants) fail("more than four participants");
    if (bind_threads) {
        cpu_set_t mask, actual;
        CPU_ZERO(&mask);
        CPU_SET(cpus[index], &mask);
        if (sched_setaffinity(0, sizeof(mask), &mask) != 0 ||
            sched_getaffinity(0, sizeof(actual), &actual) != 0 ||
            !CPU_EQUAL(&mask, &actual)) fail("cannot establish CPU affinity");
    }
    records[index].tid = syscall(SYS_gettid);
    records[index].cpu = sched_getcpu();
    atomic_store_explicit(&records[index].ready, 1, memory_order_release);
}

static void finish(void) {
    if (atomic_load_explicit(&created, memory_order_relaxed) != participants)
        fail("expected exactly four participants");
    for (unsigned i = 0; i < participants; ++i) {
        if (!atomic_load_explicit(&records[i].ready, memory_order_acquire))
            fail("participant never entered its callback");
        if (report)
            fprintf(stderr, "placement: index=%u tid=%ld bound=%d target=%d entry_cpu=%d\n",
                    i, records[i].tid, bind_threads, cpus[i], records[i].cpu);
    }
}

__attribute__((constructor)) static void initialize(void) {
    const char *binding = getenv("PLACEMENT_BIND");
    const char *main_role = getenv("PLACEMENT_MAIN");
    if (!binding || (strcmp(binding, "0") && strcmp(binding, "1")))
        fail("PLACEMENT_BIND must be 0 or 1");
    if (!main_role || (strcmp(main_role, "caller") && strcmp(main_role, "launcher")))
        fail("PLACEMENT_MAIN must be caller or launcher");
    bind_threads = !strcmp(binding, "1");
    const char *reporting = getenv("PLACEMENT_REPORT");
    report = reporting && !strcmp(reporting, "1");
    cpu_set_t allowed;
    if (sched_getaffinity(0, sizeof(allowed), &allowed) != 0)
        fail("cannot read initial CPU affinity");
    unsigned found = 0;
    for (int cpu = 0; cpu < CPU_SETSIZE && found < participants; ++cpu)
        if (CPU_ISSET(cpu, &allowed)) cpus[found++] = cpu;
    if (found != participants) fail("four allowed CPUs required");
    void *symbol = dlsym(RTLD_NEXT, "pthread_create");
    _Static_assert(sizeof(real_create) == sizeof(symbol), "function pointer size");
    if (!symbol) fail("cannot resolve pthread_create");
    memcpy(&real_create, &symbol, sizeof(real_create));
    if (!strcmp(main_role, "caller")) {
        atomic_store_explicit(&created, 1, memory_order_relaxed);
        place(0);
    }
    if (atexit(finish) != 0) fail("cannot register verification");
}

struct invocation {
    void *(*callback)(void *);
    void *argument;
    unsigned index;
};

static void *enter(void *opaque) {
    struct invocation invocation = *(struct invocation *)opaque;
    free(opaque);
    place(invocation.index);
    return invocation.callback(invocation.argument);
}

int pthread_create(pthread_t *thread, const pthread_attr_t *attributes,
                   void *(*callback)(void *), void *argument) {
    if (!real_create) fail("pthread_create before measurement initialization");
    struct invocation *invocation = malloc(sizeof(*invocation));
    if (!invocation) fail("cannot allocate wrapper context");
    invocation->callback = callback;
    invocation->argument = argument;
    invocation->index = atomic_fetch_add_explicit(&created, 1, memory_order_relaxed);
    if (invocation->index >= participants) fail("unexpected extra pthread");
    int result = real_create(thread, attributes, enter, invocation);
    if (result != 0) {
        free(invocation);
        fail("pthread_create failed");
    }
    return result;
}
