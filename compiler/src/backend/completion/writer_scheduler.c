/* Bounded stackless-writer ready queue.
 *
 * This is scheduler code, not a target adapter. It creates no thread. Target
 * publication may call only wf__writer_scheduler_ready, which enqueues an
 * opaque frame. A normal scheduler lane later calls help_once, atomically
 * claims that frame, and only then reads and invokes its resume entry.
 *
 * Capacity is reserved with operation admission: the queue has one cell for
 * each completion slot and one finite operation owns at most one ready frame.
 * Enqueue after that operation's completion event is drained therefore cannot
 * fail in a correct run.
 */

#include "writer_scheduler.h"

#include <pthread.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdlib.h>

enum wf_writer_phase {
    WF_WRITER_RUNNING = 1,
    WF_WRITER_SUSPENDING = 2,
    WF_WRITER_SUSPENDED = 3,
    WF_WRITER_NOTIFIED = 4,
    WF_WRITER_READY = 5,
    WF_WRITER_DONE = 6
};

typedef struct wf_writer_header {
    _Atomic unsigned phase;
    wf__writer_resume_fn resume;
    pthread_t origin;
} wf_writer_header;

_Static_assert(
    sizeof(wf_writer_header) <= WF_WRITER_HEADER_STORAGE_BYTES,
    "writer frame header storage"
);
_Static_assert(
    _Alignof(wf_writer_header) <= WF_WRITER_HEADER_STORAGE_ALIGNMENT,
    "writer frame header alignment"
);

static pthread_mutex_t wf_writer_lock = PTHREAD_MUTEX_INITIALIZER;
static void *wf_writer_ready[WF_WRITER_READY_CAPACITY];
static size_t wf_writer_head;
static size_t wf_writer_tail;
static size_t wf_writer_count;
static _Atomic uint64_t wf_writer_migrations;
static _Atomic uint64_t wf_writer_resumes;

/* The parallel runtime supplies strong hooks. A completion-only link keeps the
 * same scheduler queue and lets its calling-thread root pump consume it. */
__attribute__((weak)) void wf__writer_scheduler_prepare_lanes(void) {}
__attribute__((weak)) void wf__writer_scheduler_wake_lane(void) {}
__attribute__((weak)) void wf__writer_scheduler_notify(void) {}
static wf_writer_header *wf_writer_header_for(void *frame) {
    return (wf_writer_header *)frame;
}

static void wf_writer_enqueue(void *frame) {
    pthread_mutex_lock(&wf_writer_lock);
    if (wf_writer_count == WF_WRITER_READY_CAPACITY) {
        abort();
    }
    wf_writer_ready[wf_writer_tail] = frame;
    wf_writer_tail = (wf_writer_tail + 1u) % WF_WRITER_READY_CAPACITY;
    wf_writer_count += 1u;
    pthread_mutex_unlock(&wf_writer_lock);
    wf__writer_scheduler_wake_lane();
    /* A completion-only root may be parked on the unified completion epoch
     * rather than a par-lane condition. Publishing writer work is therefore
     * a compute notification in its own right. */
    wf__writer_scheduler_notify();
}

static void *wf_writer_dequeue(void) {
    void *frame = NULL;
    pthread_mutex_lock(&wf_writer_lock);
    if (wf_writer_count != 0) {
        frame = wf_writer_ready[wf_writer_head];
        wf_writer_head = (wf_writer_head + 1u) % WF_WRITER_READY_CAPACITY;
        wf_writer_count -= 1u;
    }
    pthread_mutex_unlock(&wf_writer_lock);
    return frame;
}

void wf__writer_frame_init(void *frame) {
    wf_writer_header *header;
    if (frame == NULL) {
        abort();
    }
    wf__writer_scheduler_prepare_lanes();
    header = wf_writer_header_for(frame);
    header->resume = NULL;
    header->origin = pthread_self();
    atomic_init(&header->phase, WF_WRITER_RUNNING);
}

void wf__writer_begin_suspend(void *frame, wf__writer_resume_fn resume) {
    wf_writer_header *header;
    unsigned expected = WF_WRITER_RUNNING;
    if (frame == NULL || resume == NULL) {
        abort();
    }
    header = wf_writer_header_for(frame);
    header->resume = resume;
    if (!atomic_compare_exchange_strong_explicit(
            &header->phase,
            &expected,
            WF_WRITER_SUSPENDING,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
        abort();
    }
}

void wf__writer_scheduler_ready(void *frame) {
    wf_writer_header *header = wf_writer_header_for(frame);
    unsigned phase = atomic_load_explicit(&header->phase, memory_order_acquire);
    for (;;) {
        if (phase == WF_WRITER_SUSPENDING) {
            if (atomic_compare_exchange_weak_explicit(
                    &header->phase,
                    &phase,
                    WF_WRITER_NOTIFIED,
                    memory_order_acq_rel,
                    memory_order_acquire
                )) {
                return;
            }
            continue;
        }
        if (phase == WF_WRITER_SUSPENDED) {
            if (atomic_compare_exchange_weak_explicit(
                    &header->phase,
                    &phase,
                    WF_WRITER_READY,
                    memory_order_acq_rel,
                    memory_order_acquire
                )) {
                wf_writer_enqueue(frame);
                return;
            }
            continue;
        }
        abort();
    }
}

int wf__writer_commit_suspend(void *frame) {
    wf_writer_header *header = wf_writer_header_for(frame);
    unsigned expected = WF_WRITER_SUSPENDING;
    if (atomic_compare_exchange_strong_explicit(
            &header->phase,
            &expected,
            WF_WRITER_SUSPENDED,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
        return 1;
    }
    if (expected == WF_WRITER_NOTIFIED) {
        expected = WF_WRITER_NOTIFIED;
        if (!atomic_compare_exchange_strong_explicit(
                &header->phase,
                &expected,
                WF_WRITER_READY,
                memory_order_acq_rel,
                memory_order_acquire
            )) {
            abort();
        }
        wf_writer_enqueue(frame);
        return 1;
    }
    abort();
}

void wf__writer_cancel_suspend(void *frame) {
    wf_writer_header *header = wf_writer_header_for(frame);
    unsigned expected = WF_WRITER_SUSPENDING;
    if (!atomic_compare_exchange_strong_explicit(
            &header->phase,
            &expected,
            WF_WRITER_RUNNING,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
        abort();
    }
}

void wf__writer_complete(void *frame) {
    wf_writer_header *header = wf_writer_header_for(frame);
    unsigned expected = WF_WRITER_RUNNING;
    if (!atomic_compare_exchange_strong_explicit(
            &header->phase,
            &expected,
            WF_WRITER_DONE,
            memory_order_release,
            memory_order_acquire
    )) {
        abort();
    }
    /* The root waiting for its final state may be distinct from the lane that
     * resumed it, so completion is a second scheduler-state transition. */
    wf__writer_scheduler_notify();
}

int wf__writer_scheduler_help_once(void) {
    void *frame = wf_writer_dequeue();
    wf_writer_header *header;
    unsigned expected = WF_WRITER_READY;
    if (frame == NULL) {
        return 0;
    }
    header = wf_writer_header_for(frame);
    if (!atomic_compare_exchange_strong_explicit(
            &header->phase,
            &expected,
            WF_WRITER_RUNNING,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
        return 0;
    }
    if (!pthread_equal(header->origin, pthread_self())) {
        atomic_fetch_add_explicit(&wf_writer_migrations, 1, memory_order_relaxed);
    }
    atomic_fetch_add_explicit(&wf_writer_resumes, 1, memory_order_relaxed);
    header->resume(frame);
    return 1;
}

int wf__writer_is_done(const void *frame) {
    const wf_writer_header *header = (const wf_writer_header *)frame;
    return frame != NULL
        && atomic_load_explicit(&header->phase, memory_order_acquire)
            == WF_WRITER_DONE;
}

uint64_t wf__writer_resume_migrations(void) {
    return atomic_load_explicit(&wf_writer_migrations, memory_order_relaxed);
}

uint64_t wf__writer_resume_count(void) {
    return atomic_load_explicit(&wf_writer_resumes, memory_order_relaxed);
}
