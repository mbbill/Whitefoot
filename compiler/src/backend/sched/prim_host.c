/* POSIX thread creation and per-lane waiting; no stack switching. */
#include "prim.h"
#include <sched.h>
#include <stdlib.h>
#include <string.h>
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

unsigned wf_prim_online_cpus(void) {
    long online;
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
