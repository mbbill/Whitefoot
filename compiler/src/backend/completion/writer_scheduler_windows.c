/* Windows bounded stackless-writer ready queue.
 *
 * This is a direct platform port of writer_scheduler.c: the six-state frame
 * protocol and fixed 64-cell admission bound are unchanged. Publication only
 * enqueues an opaque frame. A normal scheduler lane calls help_once and is the
 * sole place that reads and invokes the resume entry. This file creates no
 * thread.
 */

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0600
#endif

#include "writer_scheduler.h"

#include <windows.h>

#include <stddef.h>
#include <stdlib.h>

#define WF_WRITER_READY_COUNT 64u

enum wf_writer_phase {
    WF_WRITER_RUNNING = 1,
    WF_WRITER_SUSPENDING = 2,
    WF_WRITER_SUSPENDED = 3,
    WF_WRITER_NOTIFIED = 4,
    WF_WRITER_READY = 5,
    WF_WRITER_DONE = 6
};

typedef struct wf_writer_header {
    volatile LONG phase;
    wf__writer_resume_fn resume;
    DWORD origin;
} wf_writer_header;

_Static_assert(
    sizeof(wf_writer_header) <= WF_WRITER_HEADER_STORAGE_BYTES,
    "writer frame header storage"
);
_Static_assert(
    _Alignof(wf_writer_header) <= WF_WRITER_HEADER_STORAGE_ALIGNMENT,
    "writer frame header alignment"
);

static SRWLOCK wf_writer_lock = SRWLOCK_INIT;
static void *wf_writer_ready[WF_WRITER_READY_COUNT];
static size_t wf_writer_head;
static size_t wf_writer_tail;
static size_t wf_writer_count;
_Alignas(8) static volatile LONG64 wf_writer_migrations;
_Alignas(8) static volatile LONG64 wf_writer_resumes;

/* COFF has no ELF-style weak function definition. These linker aliases supply
 * completion-only no-ops only when no strong hook is present. A future Windows
 * parallel runtime overrides a hook by defining its public name, exactly as
 * par_runtime.c does on POSIX. Both link.exe and lld-link honor alternatename.
 */
void wf__writer_scheduler_wake_lane(void);
void wf__writer_scheduler_notify(void);

#if defined(_MSC_VER)
void wf__writer_scheduler_prepare_lanes_default(void) {}
void wf__writer_scheduler_wake_lane_default(void) {}
void wf__writer_scheduler_notify_default(void) {}
#if defined(_M_IX86)
#pragma comment(linker, "/alternatename:_wf__writer_scheduler_prepare_lanes=_wf__writer_scheduler_prepare_lanes_default")
#pragma comment(linker, "/alternatename:_wf__writer_scheduler_wake_lane=_wf__writer_scheduler_wake_lane_default")
#pragma comment(linker, "/alternatename:_wf__writer_scheduler_notify=_wf__writer_scheduler_notify_default")
#else
#pragma comment(linker, "/alternatename:wf__writer_scheduler_prepare_lanes=wf__writer_scheduler_prepare_lanes_default")
#pragma comment(linker, "/alternatename:wf__writer_scheduler_wake_lane=wf__writer_scheduler_wake_lane_default")
#pragma comment(linker, "/alternatename:wf__writer_scheduler_notify=wf__writer_scheduler_notify_default")
#endif
#elif defined(__clang__) || defined(__GNUC__)
__attribute__((weak)) void wf__writer_scheduler_prepare_lanes(void) {}
__attribute__((weak)) void wf__writer_scheduler_wake_lane(void) {}
__attribute__((weak)) void wf__writer_scheduler_notify(void) {}
#else
#error "writer scheduler requires MSVC, Clang, or GCC-compatible weak linkage"
#endif

static wf_writer_header *wf_writer_header_for(void *frame) {
    return (wf_writer_header *)frame;
}

static LONG wf_writer_phase_load(volatile LONG *phase) {
    return InterlockedCompareExchange(phase, 0, 0);
}

static int wf_writer_phase_compare_exchange(
    volatile LONG *phase,
    LONG *expected,
    LONG desired
) {
    LONG observed = InterlockedCompareExchange(phase, desired, *expected);
    if (observed == *expected) {
        return 1;
    }
    *expected = observed;
    return 0;
}

static void wf_writer_enqueue(void *frame) {
    AcquireSRWLockExclusive(&wf_writer_lock);
    if (wf_writer_count == WF_WRITER_READY_COUNT) {
        abort();
    }
    wf_writer_ready[wf_writer_tail] = frame;
    wf_writer_tail = (wf_writer_tail + 1u) % WF_WRITER_READY_COUNT;
    wf_writer_count += 1u;
    ReleaseSRWLockExclusive(&wf_writer_lock);
    wf__writer_scheduler_wake_lane();
    wf__writer_scheduler_notify();
}

static void *wf_writer_dequeue(void) {
    void *frame = NULL;
    AcquireSRWLockExclusive(&wf_writer_lock);
    if (wf_writer_count != 0) {
        frame = wf_writer_ready[wf_writer_head];
        wf_writer_head = (wf_writer_head + 1u) % WF_WRITER_READY_COUNT;
        wf_writer_count -= 1u;
    }
    ReleaseSRWLockExclusive(&wf_writer_lock);
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
    header->origin = GetCurrentThreadId();
    InterlockedExchange(&header->phase, WF_WRITER_RUNNING);
}

void wf__writer_begin_suspend(void *frame, wf__writer_resume_fn resume) {
    wf_writer_header *header;
    LONG expected = WF_WRITER_RUNNING;
    if (frame == NULL || resume == NULL) {
        abort();
    }
    header = wf_writer_header_for(frame);
    header->resume = resume;
    if (!wf_writer_phase_compare_exchange(
            &header->phase,
            &expected,
            WF_WRITER_SUSPENDING
        )) {
        abort();
    }
}

void wf__writer_scheduler_ready(void *frame) {
    wf_writer_header *header = wf_writer_header_for(frame);
    LONG phase = wf_writer_phase_load(&header->phase);
    for (;;) {
        if (phase == WF_WRITER_SUSPENDING) {
            if (wf_writer_phase_compare_exchange(
                    &header->phase,
                    &phase,
                    WF_WRITER_NOTIFIED
                )) {
                return;
            }
            continue;
        }
        if (phase == WF_WRITER_SUSPENDED) {
            if (wf_writer_phase_compare_exchange(
                    &header->phase,
                    &phase,
                    WF_WRITER_READY
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
    LONG expected = WF_WRITER_SUSPENDING;
    if (wf_writer_phase_compare_exchange(
            &header->phase,
            &expected,
            WF_WRITER_SUSPENDED
        )) {
        return 1;
    }
    if (expected == WF_WRITER_NOTIFIED) {
        expected = WF_WRITER_NOTIFIED;
        if (!wf_writer_phase_compare_exchange(
                &header->phase,
                &expected,
                WF_WRITER_READY
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
    LONG expected = WF_WRITER_SUSPENDING;
    if (!wf_writer_phase_compare_exchange(
            &header->phase,
            &expected,
            WF_WRITER_RUNNING
        )) {
        abort();
    }
}

void wf__writer_complete(void *frame) {
    wf_writer_header *header = wf_writer_header_for(frame);
    LONG expected = WF_WRITER_RUNNING;
    if (!wf_writer_phase_compare_exchange(
            &header->phase,
            &expected,
            WF_WRITER_DONE
        )) {
        abort();
    }
    wf__writer_scheduler_notify();
}

int wf__writer_scheduler_help_once(void) {
    void *frame = wf_writer_dequeue();
    wf_writer_header *header;
    LONG expected = WF_WRITER_READY;
    if (frame == NULL) {
        return 0;
    }
    header = wf_writer_header_for(frame);
    if (!wf_writer_phase_compare_exchange(
            &header->phase,
            &expected,
            WF_WRITER_RUNNING
        )) {
        return 0;
    }
    if (header->origin != GetCurrentThreadId()) {
        (void)InterlockedIncrement64(&wf_writer_migrations);
    }
    (void)InterlockedIncrement64(&wf_writer_resumes);
    header->resume(frame);
    return 1;
}

int wf__writer_is_done(const void *frame) {
    const wf_writer_header *header = (const wf_writer_header *)frame;
    return frame != NULL
        && wf_writer_phase_load((volatile LONG *)&header->phase)
            == WF_WRITER_DONE;
}

uint64_t wf__writer_resume_migrations(void) {
    return (uint64_t)InterlockedCompareExchange64(
        &wf_writer_migrations,
        0,
        0
    );
}

uint64_t wf__writer_resume_count(void) {
    return (uint64_t)InterlockedCompareExchange64(
        &wf_writer_resumes,
        0,
        0
    );
}
