#if !defined(__linux__)
#error "io_uring.c is the Linux completion backend"
#endif

#define _GNU_SOURCE

#include "contract.h"
#include "platform.h"

#include <errno.h>
#include <linux/io_uring.h>
#include <poll.h>
#include <sched.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <sys/eventfd.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <unistd.h>

#ifndef IORING_POLL_ADD_MULTI
#define IORING_POLL_ADD_MULTI (1U << 0)
#endif
#ifndef IORING_CQE_F_MORE
#define IORING_CQE_F_MORE (1U << 1)
#endif

#define WF_IO_RING_ENTRIES 8u
#define WF_IO_PLATFORM_THREADS 16u

#if defined(WF_IO_TEST_FORCE_ONESHOT) && !defined(WF_IO_TESTING)
#error "WF_IO_TEST_FORCE_ONESHOT is a harness-only fault mode"
#endif

#if defined(WF_IO_TEST_FORCE_ONESHOT)
#define WF_IO_INITIAL_MULTISHOT 0u
#else
#define WF_IO_INITIAL_MULTISHOT 1u
#endif

typedef struct wf_io_uring_lane {
    int ring;
    int event;
    void *sq_mapping;
    size_t sq_mapping_length;
    void *cq_mapping;
    size_t cq_mapping_length;
    struct io_uring_sqe *sqes;
    size_t sqes_length;
    unsigned *sq_head;
    unsigned *sq_tail;
    unsigned *sq_mask;
    unsigned *sq_entries;
    unsigned *sq_array;
    unsigned *cq_head;
    unsigned *cq_tail;
    unsigned *cq_mask;
    struct io_uring_cqe *cqes;
    unsigned multishot;
    _Atomic unsigned initialized;
} wf_io_uring_lane;

typedef struct wf_io_thread_start {
    wf_io_thread_fn function;
    void *context;
} wf_io_thread_start;

static wf_io_uring_lane wf_io_uring_lanes[WF_IO_MAX_LANES];
static wf_io_thread_start wf_io_thread_starts[WF_IO_PLATFORM_THREADS];
static _Atomic unsigned wf_io_thread_start_count;
static _Atomic unsigned wf_io_all_multishot = WF_IO_INITIAL_MULTISHOT;
static _Atomic uint64_t wf_io_poll_arms;

static void wf_io_uring_defect(void) {
    abort();
}

static int wf_io_enter(int ring, unsigned submit, unsigned complete, unsigned flags) {
    return (int)syscall(
        __NR_io_uring_enter,
        ring,
        submit,
        complete,
        flags,
        NULL,
        (size_t)0
    );
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

static int wf_io_arm_poll(wf_io_uring_lane *lane) {
    unsigned head = __atomic_load_n(lane->sq_head, __ATOMIC_ACQUIRE);
    unsigned tail = __atomic_load_n(lane->sq_tail, __ATOMIC_RELAXED);
    unsigned index;
    struct io_uring_sqe *sqe;
    if (tail - head >= *lane->sq_entries) {
        return -1;
    }
    index = tail & *lane->sq_mask;
    sqe = &lane->sqes[index];
    memset(sqe, 0, sizeof(*sqe));
    sqe->opcode = IORING_OP_POLL_ADD;
    sqe->fd = lane->event;
    sqe->poll32_events = (uint32_t)POLLIN;
    if (lane->multishot != 0) {
        sqe->len = IORING_POLL_ADD_MULTI;
    }
    sqe->user_data = 1;
    lane->sq_array[index] = index;
    __atomic_store_n(lane->sq_tail, tail + 1, __ATOMIC_RELEASE);
    if (wf_io_enter(lane->ring, 1, 0, 0) != 1) {
        return -1;
    }
    atomic_fetch_add_explicit(&wf_io_poll_arms, 1, memory_order_relaxed);
    return 0;
}

int wf_io_platform_global_init(void) {
    return 0;
}

int wf_io_platform_lane_init(unsigned lane_id) {
    struct io_uring_params parameters;
    wf_io_uring_lane *lane;
    size_t sq_length;
    size_t cq_length;
    void *shared;
    if (lane_id >= WF_IO_MAX_LANES) {
        return -1;
    }
    lane = &wf_io_uring_lanes[lane_id];
    memset(lane, 0, sizeof(*lane));
    lane->ring = -1;
    lane->event = -1;
    memset(&parameters, 0, sizeof(parameters));
    lane->ring = (int)syscall(__NR_io_uring_setup, WF_IO_RING_ENTRIES, &parameters);
    if (lane->ring < 0) {
        return -1;
    }
    sq_length = parameters.sq_off.array + parameters.sq_entries * sizeof(unsigned);
    cq_length = parameters.cq_off.cqes
        + parameters.cq_entries * sizeof(struct io_uring_cqe);
    lane->sq_mapping_length = sq_length;
    lane->cq_mapping_length = cq_length;
    if ((parameters.features & IORING_FEAT_SINGLE_MMAP) != 0) {
        size_t length = sq_length > cq_length ? sq_length : cq_length;
        shared = mmap(
            NULL,
            length,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            lane->ring,
            IORING_OFF_SQ_RING
        );
        if (shared == MAP_FAILED) {
            close(lane->ring);
            return -1;
        }
        lane->sq_mapping = shared;
        lane->cq_mapping = shared;
        lane->sq_mapping_length = length;
        lane->cq_mapping_length = length;
    } else {
        lane->sq_mapping = mmap(
            NULL,
            sq_length,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            lane->ring,
            IORING_OFF_SQ_RING
        );
        lane->cq_mapping = mmap(
            NULL,
            cq_length,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            lane->ring,
            IORING_OFF_CQ_RING
        );
        if (lane->sq_mapping == MAP_FAILED || lane->cq_mapping == MAP_FAILED) {
            close(lane->ring);
            return -1;
        }
    }
    lane->sqes_length = parameters.sq_entries * sizeof(struct io_uring_sqe);
    lane->sqes = mmap(
        NULL,
        lane->sqes_length,
        PROT_READ | PROT_WRITE,
        MAP_SHARED | MAP_POPULATE,
        lane->ring,
        IORING_OFF_SQES
    );
    if (lane->sqes == MAP_FAILED) {
        close(lane->ring);
        return -1;
    }
    lane->sq_head = (unsigned *)((char *)lane->sq_mapping + parameters.sq_off.head);
    lane->sq_tail = (unsigned *)((char *)lane->sq_mapping + parameters.sq_off.tail);
    lane->sq_mask = (unsigned *)((char *)lane->sq_mapping + parameters.sq_off.ring_mask);
    lane->sq_entries = (unsigned *)((char *)lane->sq_mapping + parameters.sq_off.ring_entries);
    lane->sq_array = (unsigned *)((char *)lane->sq_mapping + parameters.sq_off.array);
    lane->cq_head = (unsigned *)((char *)lane->cq_mapping + parameters.cq_off.head);
    lane->cq_tail = (unsigned *)((char *)lane->cq_mapping + parameters.cq_off.tail);
    lane->cq_mask = (unsigned *)((char *)lane->cq_mapping + parameters.cq_off.ring_mask);
    lane->cqes = (struct io_uring_cqe *)((char *)lane->cq_mapping + parameters.cq_off.cqes);
    lane->event = eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK);
    if (lane->event < 0) {
        close(lane->ring);
        return -1;
    }
    lane->multishot = WF_IO_INITIAL_MULTISHOT;
    atomic_init(&lane->initialized, 0);
    if (wf_io_arm_poll(lane) != 0) {
        return -1;
    }
    atomic_store_explicit(&lane->initialized, 1, memory_order_release);
    return 0;
}

void wf_io_platform_wake(unsigned lane_id) {
    wf_io_uring_lane *lane;
    uint64_t one = 1;
    ssize_t written;
    if (lane_id >= WF_IO_MAX_LANES) {
        wf_io_uring_defect();
    }
    lane = &wf_io_uring_lanes[lane_id];
    if (atomic_load_explicit(&lane->initialized, memory_order_acquire) == 0) {
        wf_io_uring_defect();
    }
    written = write(lane->event, &one, sizeof(one));
    if (written != (ssize_t)sizeof(one) && errno != EAGAIN) {
        wf_io_uring_defect();
    }
}

void wf_io_platform_park(unsigned lane_id) {
    wf_io_uring_lane *lane;
    if (lane_id >= WF_IO_MAX_LANES) {
        wf_io_uring_defect();
    }
    lane = &wf_io_uring_lanes[lane_id];
    for (;;) {
        unsigned head;
        unsigned tail;
        unsigned rearm = 0;
        uint64_t count;
        ssize_t read_result;
        int entered = wf_io_enter(lane->ring, 0, 1, IORING_ENTER_GETEVENTS);
        if (entered < 0) {
            if (errno == EINTR) {
                continue;
            }
            wf_io_uring_defect();
        }
        head = __atomic_load_n(lane->cq_head, __ATOMIC_RELAXED);
        tail = __atomic_load_n(lane->cq_tail, __ATOMIC_ACQUIRE);
        if (head == tail) {
            continue;
        }
        while (head != tail) {
            struct io_uring_cqe *cqe = &lane->cqes[head & *lane->cq_mask];
            if (cqe->user_data != 1) {
                wf_io_uring_defect();
            }
            if (cqe->res == -EINVAL && lane->multishot != 0) {
                /* Older kernels reject POLL_ADD_MULTI.  The same ring and
                 * eventfd remain the park path; only the poll becomes
                 * one-shot and is rearmed after every consumption. */
                lane->multishot = 0;
                atomic_store_explicit(&wf_io_all_multishot, 0, memory_order_release);
                rearm = 1;
            } else if (cqe->res < 0) {
                wf_io_uring_defect();
            } else if ((cqe->flags & IORING_CQE_F_MORE) == 0) {
                rearm = 1;
            }
            head += 1;
        }
        __atomic_store_n(lane->cq_head, head, __ATOMIC_RELEASE);
        do {
            read_result = read(lane->event, &count, sizeof(count));
        } while (read_result < 0 && errno == EINTR);
        if (read_result < 0 && errno != EAGAIN) {
            wf_io_uring_defect();
        }
        if (rearm != 0 && wf_io_arm_poll(lane) != 0) {
            wf_io_uring_defect();
        }
        if (read_result == (ssize_t)sizeof(count)) {
            return;
        }
        /* A rejected multishot request produced a CQE but consumed no wake.
         * Its one-shot replacement is armed now; wait for the retained
         * eventfd count through the same io_uring_enter park point. */
    }
}

const char *wf_io_platform_name(void) {
    return "io_uring";
}

unsigned wf_io_platform_features(void) {
    unsigned features = WF_IO_BACKEND_IO_URING
        | WF_IO_BACKEND_EVENTFD_POLL
        | WF_IO_BACKEND_POLL_REARM
        | WF_IO_BACKEND_UNIFIED_PARK;
    if (atomic_load_explicit(&wf_io_all_multishot, memory_order_acquire) != 0) {
        features |= WF_IO_BACKEND_POLL_MULTISHOT;
    }
    return features;
}

uint64_t wf_io_platform_poll_arms(void) {
    return atomic_load_explicit(&wf_io_poll_arms, memory_order_relaxed);
}

int wf_io_platform_mutex_init(wf_io_mutex *mutex) {
    return pthread_mutex_init(mutex, NULL);
}

int wf_io_platform_condition_init(wf_io_condition *condition) {
    return pthread_cond_init(condition, NULL);
}

void wf_io_platform_mutex_lock(wf_io_mutex *mutex) {
    if (pthread_mutex_lock(mutex) != 0) {
        wf_io_uring_defect();
    }
}

void wf_io_platform_mutex_unlock(wf_io_mutex *mutex) {
    if (pthread_mutex_unlock(mutex) != 0) {
        wf_io_uring_defect();
    }
}

void wf_io_platform_condition_wait(wf_io_condition *condition, wf_io_mutex *mutex) {
    if (pthread_cond_wait(condition, mutex) != 0) {
        wf_io_uring_defect();
    }
}

void wf_io_platform_condition_signal(wf_io_condition *condition) {
    if (pthread_cond_signal(condition) != 0) {
        wf_io_uring_defect();
    }
}

void wf_io_platform_pause(void) {
    sched_yield();
}
