#if !defined(__APPLE__)
#error "kqueue.c is the macOS completion backend"
#endif

#include "contract.h"
#include "platform.h"

#include <errno.h>
#include <sched.h>
#include <stdlib.h>
#include <string.h>
#include <sys/event.h>
#include <sys/time.h>
#include <unistd.h>

#define WF_IO_KQUEUE_EVENTS 64
#define WF_IO_PLATFORM_THREADS 16

typedef struct wf_io_kqueue_lane {
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    unsigned posted;
    _Atomic unsigned initialized;
} wf_io_kqueue_lane;

typedef struct wf_io_thread_start {
    wf_io_thread_fn function;
    void *context;
} wf_io_thread_start;

static int wf_io_kqueue_descriptor = -1;
static wf_io_kqueue_lane wf_io_kqueue_lanes[WF_IO_MAX_LANES];
static wf_io_thread_start wf_io_thread_starts[WF_IO_PLATFORM_THREADS];
static _Atomic unsigned wf_io_thread_start_count;
static _Atomic uint64_t wf_io_kqueue_arms;

static void wf_io_kqueue_defect(void) {
    abort();
}

static void *wf_io_pthread_entry(void *opaque) {
    wf_io_thread_start *start = (wf_io_thread_start *)opaque;
    start->function(start->context);
    return NULL;
}

int wf_io_platform_thread_start(wf_io_thread_fn function, void *context) {
    pthread_attr_t attributes;
    pthread_t thread;
    unsigned slot = atomic_fetch_add_explicit(
        &wf_io_thread_start_count,
        1,
        memory_order_acq_rel
    );
    if (slot >= WF_IO_PLATFORM_THREADS || pthread_attr_init(&attributes) != 0) {
        return -1;
    }
    wf_io_thread_starts[slot].function = function;
    wf_io_thread_starts[slot].context = context;
    if (pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED) != 0
        || pthread_create(
            &thread,
            &attributes,
            wf_io_pthread_entry,
            &wf_io_thread_starts[slot]
        ) != 0) {
        pthread_attr_destroy(&attributes);
        return -1;
    }
    pthread_attr_destroy(&attributes);
    return 0;
}

static void wf_io_kqueue_waiter(void *unused) {
    struct kevent events[WF_IO_KQUEUE_EVENTS];
    (void)unused;
    for (;;) {
        int count = kevent(
            wf_io_kqueue_descriptor,
            NULL,
            0,
            events,
            WF_IO_KQUEUE_EVENTS,
            NULL
        );
        int index;
        if (count < 0) {
            if (errno == EINTR) {
                continue;
            }
            wf_io_kqueue_defect();
        }
        for (index = 0; index < count; index += 1) {
            uintptr_t ident = events[index].ident;
            wf_io_kqueue_lane *lane;
            if (events[index].filter != EVFILT_USER || ident == 0 || ident > WF_IO_MAX_LANES) {
                wf_io_kqueue_defect();
            }
            lane = &wf_io_kqueue_lanes[ident - 1];
            if (atomic_load_explicit(&lane->initialized, memory_order_acquire) == 0) {
                wf_io_kqueue_defect();
            }
            pthread_mutex_lock(&lane->mutex);
            lane->posted = 1;
            pthread_cond_signal(&lane->condition);
            pthread_mutex_unlock(&lane->mutex);
        }
    }
}

int wf_io_platform_global_init(void) {
    wf_io_kqueue_descriptor = kqueue();
    if (wf_io_kqueue_descriptor < 0) {
        return -1;
    }
    if (wf_io_platform_thread_start(wf_io_kqueue_waiter, NULL) != 0) {
        close(wf_io_kqueue_descriptor);
        wf_io_kqueue_descriptor = -1;
        return -1;
    }
    return 0;
}

int wf_io_platform_lane_init(unsigned lane_id) {
    struct kevent registration;
    wf_io_kqueue_lane *lane;
    if (lane_id >= WF_IO_MAX_LANES || wf_io_kqueue_descriptor < 0) {
        return -1;
    }
    lane = &wf_io_kqueue_lanes[lane_id];
    if (pthread_mutex_init(&lane->mutex, NULL) != 0
        || pthread_cond_init(&lane->condition, NULL) != 0) {
        return -1;
    }
    lane->posted = 0;
    atomic_init(&lane->initialized, 0);
    EV_SET(&registration, (uintptr_t)lane_id + 1, EVFILT_USER, EV_ADD | EV_CLEAR, 0, 0, NULL);
    if (kevent(wf_io_kqueue_descriptor, &registration, 1, NULL, 0, NULL) != 0) {
        return -1;
    }
    atomic_store_explicit(&lane->initialized, 1, memory_order_release);
    atomic_fetch_add_explicit(&wf_io_kqueue_arms, 1, memory_order_relaxed);
    return 0;
}

void wf_io_platform_wake(unsigned lane) {
    struct kevent wake;
    if (lane >= WF_IO_MAX_LANES
        || atomic_load_explicit(
               &wf_io_kqueue_lanes[lane].initialized,
               memory_order_acquire
           ) == 0) {
        wf_io_kqueue_defect();
    }
    EV_SET(&wake, (uintptr_t)lane + 1, EVFILT_USER, 0, NOTE_TRIGGER, 0, NULL);
    if (kevent(wf_io_kqueue_descriptor, &wake, 1, NULL, 0, NULL) != 0) {
        wf_io_kqueue_defect();
    }
}

void wf_io_platform_park(unsigned lane_id) {
    wf_io_kqueue_lane *lane;
    if (lane_id >= WF_IO_MAX_LANES) {
        wf_io_kqueue_defect();
    }
    lane = &wf_io_kqueue_lanes[lane_id];
    pthread_mutex_lock(&lane->mutex);
    while (lane->posted == 0) {
        pthread_cond_wait(&lane->condition, &lane->mutex);
    }
    lane->posted = 0;
    pthread_mutex_unlock(&lane->mutex);
}

const char *wf_io_platform_name(void) {
    return "kqueue";
}

unsigned wf_io_platform_features(void) {
    return WF_IO_BACKEND_KQUEUE | WF_IO_BACKEND_WAITER_THREAD;
}

uint64_t wf_io_platform_poll_arms(void) {
    return atomic_load_explicit(&wf_io_kqueue_arms, memory_order_relaxed);
}

int wf_io_platform_mutex_init(wf_io_mutex *mutex) {
    return pthread_mutex_init(mutex, NULL);
}

int wf_io_platform_condition_init(wf_io_condition *condition) {
    return pthread_cond_init(condition, NULL);
}

void wf_io_platform_mutex_lock(wf_io_mutex *mutex) {
    if (pthread_mutex_lock(mutex) != 0) {
        wf_io_kqueue_defect();
    }
}

void wf_io_platform_mutex_unlock(wf_io_mutex *mutex) {
    if (pthread_mutex_unlock(mutex) != 0) {
        wf_io_kqueue_defect();
    }
}

void wf_io_platform_condition_wait(wf_io_condition *condition, wf_io_mutex *mutex) {
    if (pthread_cond_wait(condition, mutex) != 0) {
        wf_io_kqueue_defect();
    }
}

void wf_io_platform_condition_signal(wf_io_condition *condition) {
    if (pthread_cond_signal(condition) != 0) {
        wf_io_kqueue_defect();
    }
}

void wf_io_platform_pause(void) {
    sched_yield();
}
