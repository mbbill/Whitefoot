/* POSIX thread creation and per-lane waiting; no stack switching. */
/* Feature selection, and it has to precede every include. The monotonic clock
 * is POSIX and the CPU affinity mask is a glibc extension, and `-std=c11`
 * -- which the gate, the emitted link and this file's probes all compile with
 * -- hides both unless one of these is named first. macOS selects the Darwin
 * set for `sysctlbyname`, the same choice `wake_probe.c` makes. Each is
 * guarded so naming it again on a command line is not a redefinition. */
#if defined(__linux__)
#if !defined(_GNU_SOURCE)
#define _GNU_SOURCE 1
#endif
#elif defined(__APPLE__)
#if !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif
#elif !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
#include "prim.h"
#include <sched.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <sys/sysctl.h>
#endif

static void *wf_prim_thread_main(void *opaque) {
    wf_prim_thread *thread = opaque;
    thread->entry(thread->argument);
    return NULL;
}

int wf_prim_thread_start(
    wf_prim_thread *thread,
    void (*entry)(void *),
    void *argument,
    size_t stack_bytes
) {
    pthread_attr_t attributes;
    pthread_t created;
    int error;
    if (thread == NULL || entry == NULL) {
        return 1;
    }
    thread->entry = entry;
    thread->argument = argument;
    if (pthread_attr_init(&attributes) != 0) {
        return 1;
    }
    if (stack_bytes != 0
        && pthread_attr_setstacksize(&attributes, stack_bytes) != 0) {
        (void)pthread_attr_destroy(&attributes);
        return 1;
    }
    (void)pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    error = pthread_create(&created, &attributes, wf_prim_thread_main, thread);
    (void)pthread_attr_destroy(&attributes);
    return error != 0 ? 1 : 0;
}

/* How many CPUs this process may actually run on. The affinity mask is the
 * honest answer where it is cheap to read, because a process confined to two
 * of a machine's four CPUs is oversubscribed at four lanes however many CPUs
 * are online; sysconf answers the rest. Zero means the count is unknown, and
 * a caller must then choose the behaviour that assumes nothing about it. */
unsigned wf_prim_online_cpus(void) {
    long online;
#if defined(__linux__)
    cpu_set_t affinity;
    CPU_ZERO(&affinity);
    if (sched_getaffinity(0, sizeof affinity, &affinity) == 0) {
        int permitted = CPU_COUNT(&affinity);
        if (permitted > 0) {
            return (unsigned)permitted;
        }
    }
#endif
#if defined(__APPLE__)
    int logical = 0;
    size_t width = sizeof(logical);
    if (sysctlbyname("hw.logicalcpu", &logical, &width, NULL, 0) == 0
        && logical > 0) {
        return (unsigned)logical;
    }
#endif
    online = sysconf(_SC_NPROCESSORS_ONLN);
    return online > 0 ? (unsigned)online : 0u;
}

uint64_t wf_prim_monotonic_us(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return 0;
    }
    return (uint64_t)now.tv_sec * UINT64_C(1000000)
        + (uint64_t)now.tv_nsec / UINT64_C(1000);
}

int wf_prim_setting_text(const char *name, char *buffer, size_t capacity) {
    const char *text;
    size_t length;
    if (name == NULL || buffer == NULL || capacity == 0) {
        return -1;
    }
    buffer[0] = '\0';
    text = getenv(name);
    if (text == NULL) {
        return 0;
    }
    length = strlen(text);
    if (length >= capacity) {
        return -1;
    }
    memcpy(buffer, text, length + 1);
    return 1;
}

__attribute__((weak)) void wf__floor_attach_thread(void) {}
void wf_prim_floor_attach(void) { wf__floor_attach_thread(); }
void wf_prim_yield(void) { sched_yield(); }
int wf_prim_wait_init(wf_prim_wait *wait) {
    if (pthread_mutex_init(&wait->lock, NULL) != 0) return 1;
    if (pthread_cond_init(&wait->signal, NULL) == 0) return 0;
    pthread_mutex_destroy(&wait->lock);
    return 1;
}
void wf_prim_wait_destroy(wf_prim_wait *wait) {
    pthread_cond_destroy(&wait->signal);
    pthread_mutex_destroy(&wait->lock);
}
void wf_prim_wait_lock(wf_prim_wait *wait) { pthread_mutex_lock(&wait->lock); }
void wf_prim_wait_unlock(wf_prim_wait *wait) { pthread_mutex_unlock(&wait->lock); }
void wf_prim_wait_sleep(wf_prim_wait *wait) { pthread_cond_wait(&wait->signal, &wait->lock); }
void wf_prim_wait_signal(wf_prim_wait *wait) { pthread_cond_signal(&wait->signal); }
