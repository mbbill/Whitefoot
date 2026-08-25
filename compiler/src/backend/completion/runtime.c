#if !defined(_WIN32) && !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

/* Shared Whitefoot completion state machine.
 *
 * Host backends provide only lane registration and one lossless park/wake
 * endpoint.  Submission, the fixed-depth blocking pool, generation checking,
 * the intrusive MPSC mailbox, loan lifetime, and bounded join helping live
 * here and are therefore identical on kqueue, io_uring, and IOCP.
 */

#include "contract.h"
#include "platform.h"

#include <stdlib.h>
#include <string.h>

#if !defined(_WIN32)
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#endif

#define WF_IO_INIT_UNSTARTED 0
#define WF_IO_INIT_STARTING 1
#define WF_IO_INIT_READY 2
#define WF_IO_INIT_FAILED 3
#define WF_IO_MAILBOX_BATCH 64u

typedef struct wf_io_lane {
    _Atomic(wf_io_frame *) mailbox;
    _Atomic unsigned announced;
    unsigned id;
} wf_io_lane;

typedef struct wf_io_disk_queue {
    wf_io_mutex mutex;
    wf_io_condition available;
    wf_io_frame *head;
    wf_io_frame *tail;
} wf_io_disk_queue;

static wf_io_lane wf_io_lanes[WF_IO_MAX_LANES];
static _Atomic unsigned wf_io_lane_count;
static _Thread_local wf_io_lane *wf_io_current_lane;
static _Thread_local int wf_io_disk_worker;
static _Thread_local unsigned wf_io_help_depth;

static wf_io_disk_queue wf_io_disk;
static _Atomic unsigned wf_io_init_state;

static _Atomic(wf_io_help_fn) wf_io_help;
static _Atomic(void *) wf_io_help_context;

static _Atomic uint64_t wf_io_stat_submissions;
static _Atomic uint64_t wf_io_stat_submission_failures;
static _Atomic uint64_t wf_io_stat_executions;
static _Atomic uint64_t wf_io_stat_completions;
static _Atomic uint64_t wf_io_stat_mailbox_batches;
static _Atomic uint64_t wf_io_stat_parks;
static _Atomic uint64_t wf_io_stat_wakes;
static _Atomic uint64_t wf_io_stat_help_calls;
static _Atomic uint64_t wf_io_stat_stale_publications;
static _Atomic uint64_t wf_io_stat_duplicate_publications;

#if defined(WF_IO_TESTING)
static _Atomic unsigned wf_io_fail_submission;
#endif

static void wf_io_defect(void) {
    abort();
}

static void wf_io_mailbox_push(wf_io_lane *lane, wf_io_frame *frame) {
    wf_io_frame *head = atomic_load_explicit(&lane->mailbox, memory_order_relaxed);
    do {
        atomic_store_explicit(&frame->mailbox_next, head, memory_order_relaxed);
    } while (!atomic_compare_exchange_weak_explicit(
        &lane->mailbox,
        &head,
        frame,
        memory_order_release,
        memory_order_relaxed
    ));
}

static int wf_io_validate_publication(wf_io_frame *frame, uint64_t generation) {
    uint64_t current = atomic_load_explicit(&frame->generation, memory_order_acquire);
    uint64_t published;
    uint64_t observed;
    if (generation != current) {
        atomic_fetch_add_explicit(&wf_io_stat_stale_publications, 1, memory_order_relaxed);
        return WF_IO_TEST_PUBLICATION_STALE;
    }
    published = atomic_load_explicit(&frame->published_generation, memory_order_acquire);
    observed = atomic_load_explicit(&frame->observed_generation, memory_order_acquire);
    if (observed == generation) {
        atomic_fetch_add_explicit(&wf_io_stat_duplicate_publications, 1, memory_order_relaxed);
        return WF_IO_TEST_PUBLICATION_DUPLICATE;
    }
    if (published != generation
        || atomic_load_explicit(&frame->state, memory_order_acquire) != WF_IO_FRAME_TERMINAL
        || atomic_load_explicit(&frame->loan, memory_order_acquire) != WF_IO_LOAN_TERMINAL) {
        return WF_IO_TEST_PUBLICATION_MALFORMED;
    }
    atomic_store_explicit(&frame->observed_generation, generation, memory_order_release);
    return WF_IO_TEST_PUBLICATION_VALID;
}

/* Takes one release-published producer batch, reverses its LIFO links, then
 * validates a bounded number of nodes.  A larger batch is placed back on the
 * mailbox before returning, so completion work cannot starve compute work. */
static unsigned wf_io_drain_mailbox(wf_io_lane *lane) {
    wf_io_frame *batch = atomic_exchange_explicit(&lane->mailbox, NULL, memory_order_acquire);
    wf_io_frame *ordered = NULL;
    wf_io_frame *rest;
    unsigned count = 0;
    while (batch != NULL) {
        wf_io_frame *next = atomic_load_explicit(&batch->mailbox_next, memory_order_relaxed);
        batch->work_next = ordered;
        ordered = batch;
        batch = next;
    }
    rest = ordered;
    while (rest != NULL && count < WF_IO_MAILBOX_BATCH) {
        wf_io_frame *next = rest->work_next;
        uint64_t generation = atomic_load_explicit(
            &rest->published_generation,
            memory_order_acquire
        );
        if (wf_io_validate_publication(rest, generation) != WF_IO_TEST_PUBLICATION_VALID) {
            wf_io_defect();
        }
        rest = next;
        count += 1;
    }
    if (rest != NULL) {
        do {
            wf_io_frame *next = rest->work_next;
            wf_io_mailbox_push(lane, rest);
            rest = next;
        } while (rest != NULL);
        wf_io_platform_wake(lane->id);
        atomic_fetch_add_explicit(&wf_io_stat_wakes, 1, memory_order_relaxed);
    }
    if (count != 0) {
        atomic_fetch_add_explicit(&wf_io_stat_mailbox_batches, 1, memory_order_relaxed);
    }
    return count;
}

static void wf_io_complete(wf_io_frame *frame, wf_io_result result) {
    uint64_t generation;
    unsigned owner_lane;
    unsigned expected = WF_IO_FRAME_EXECUTING;
    if (!atomic_compare_exchange_strong_explicit(
        &frame->state,
        &expected,
        WF_IO_FRAME_COMPLETING,
        memory_order_acq_rel,
        memory_order_acquire
    )) {
        wf_io_defect();
    }
    frame->result = result.value;
    frame->error_code = result.error_code;
    generation = atomic_load_explicit(&frame->generation, memory_order_relaxed);
    owner_lane = atomic_load_explicit(&frame->owner_lane, memory_order_acquire);
    if (owner_lane >= WF_IO_MAX_LANES) {
        wf_io_defect();
    }
    atomic_store_explicit(&frame->published_generation, generation, memory_order_release);
    atomic_store_explicit(&frame->loan, WF_IO_LOAN_TERMINAL, memory_order_release);
    atomic_store_explicit(&frame->state, WF_IO_FRAME_TERMINAL, memory_order_release);
    wf_io_mailbox_push(&wf_io_lanes[owner_lane], frame);
    /* Enqueue is the linearization point; the wake is deliberately later. */
    wf_io_platform_wake(owner_lane);
    atomic_fetch_add_explicit(&wf_io_stat_wakes, 1, memory_order_relaxed);
    atomic_fetch_add_explicit(&wf_io_stat_completions, 1, memory_order_relaxed);
}

static void wf_io_disk_main(void *unused) {
    (void)unused;
    wf_io_disk_worker = 1;
    for (;;) {
        wf_io_frame *frame;
        unsigned expected = WF_IO_FRAME_SUBMITTED;
        wf_io_platform_mutex_lock(&wf_io_disk.mutex);
        while (wf_io_disk.head == NULL) {
            wf_io_platform_condition_wait(&wf_io_disk.available, &wf_io_disk.mutex);
        }
        frame = wf_io_disk.head;
        wf_io_disk.head = frame->work_next;
        if (wf_io_disk.head == NULL) {
            wf_io_disk.tail = NULL;
        }
        frame->work_next = NULL;
        wf_io_platform_mutex_unlock(&wf_io_disk.mutex);

        if (!atomic_compare_exchange_strong_explicit(
            &frame->state,
            &expected,
            WF_IO_FRAME_EXECUTING,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
            wf_io_defect();
        }
        atomic_fetch_add_explicit(&wf_io_stat_executions, 1, memory_order_relaxed);
        wf_io_complete(frame, frame->work(frame->context));
    }
}

static int wf_io_initialize(void) {
    unsigned expected = WF_IO_INIT_UNSTARTED;
    unsigned worker;
    if (atomic_compare_exchange_strong_explicit(
        &wf_io_init_state,
        &expected,
        WF_IO_INIT_STARTING,
        memory_order_acq_rel,
        memory_order_acquire
    )) {
        if (wf_io_platform_global_init() != 0
            || wf_io_platform_mutex_init(&wf_io_disk.mutex) != 0
            || wf_io_platform_condition_init(&wf_io_disk.available) != 0) {
            atomic_store_explicit(&wf_io_init_state, WF_IO_INIT_FAILED, memory_order_release);
            return -1;
        }
        wf_io_disk.head = NULL;
        wf_io_disk.tail = NULL;
        for (worker = 0; worker < WF_IO_DISK_THREADS; worker += 1) {
            if (wf_io_platform_thread_start(wf_io_disk_main, NULL) != 0) {
                atomic_store_explicit(&wf_io_init_state, WF_IO_INIT_FAILED, memory_order_release);
                return -1;
            }
        }
        atomic_store_explicit(&wf_io_init_state, WF_IO_INIT_READY, memory_order_release);
        return 0;
    }
    for (;;) {
        unsigned state = atomic_load_explicit(&wf_io_init_state, memory_order_acquire);
        if (state == WF_IO_INIT_READY) {
            return 0;
        }
        if (state == WF_IO_INIT_FAILED) {
            return -1;
        }
        wf_io_platform_pause();
    }
}

static wf_io_lane *wf_io_lane_current(void) {
    wf_io_lane *lane = wf_io_current_lane;
    unsigned id;
    if (lane != NULL) {
        return lane;
    }
    id = atomic_fetch_add_explicit(&wf_io_lane_count, 1, memory_order_acq_rel);
    if (id >= WF_IO_MAX_LANES || wf_io_platform_lane_init(id) != 0) {
        wf_io_defect();
    }
    lane = &wf_io_lanes[id];
    lane->id = id;
    atomic_init(&lane->mailbox, NULL);
    atomic_init(&lane->announced, 0);
    wf_io_current_lane = lane;
    return lane;
}

void wf_io_frame_init(wf_io_frame *frame) {
    if (frame == NULL) {
        wf_io_defect();
    }
    memset(frame, 0, sizeof(*frame));
    atomic_init(&frame->state, WF_IO_FRAME_FREE);
    atomic_init(&frame->loan, WF_IO_LOAN_RETURNED);
    atomic_init(&frame->generation, 0);
    atomic_init(&frame->published_generation, 0);
    atomic_init(&frame->observed_generation, 0);
    atomic_init(&frame->mailbox_next, NULL);
    atomic_init(&frame->owner_lane, WF_IO_MAX_LANES);
}

int wf_io_submit(wf_io_frame *frame, wf_io_work_fn work, void *context) {
    wf_io_lane *lane;
    uint64_t generation;
    unsigned state;
    if (frame == NULL || work == NULL) {
        return -2;
    }
    if (wf_io_initialize() != 0) {
        wf_io_defect();
    }
    lane = wf_io_lane_current();
    state = atomic_load_explicit(&frame->state, memory_order_acquire);
    for (;;) {
        unsigned expected = state;
        if (state != WF_IO_FRAME_FREE && state != WF_IO_FRAME_REAPED) {
            return -2;
        }
        if (atomic_compare_exchange_weak_explicit(
            &frame->state,
            &expected,
            WF_IO_FRAME_PREPARING,
            memory_order_acq_rel,
            memory_order_acquire
        )) {
            break;
        }
        state = expected;
    }
    generation = atomic_fetch_add_explicit(&frame->generation, 1, memory_order_relaxed) + 1;
    if (generation == 0) {
        wf_io_defect();
    }
    frame->work = work;
    frame->context = context;
    frame->result = 0;
    frame->error_code = 0;
    atomic_store_explicit(&frame->owner_lane, lane->id, memory_order_release);
    frame->work_next = NULL;
    atomic_store_explicit(&frame->mailbox_next, NULL, memory_order_relaxed);
    atomic_store_explicit(&frame->loan, WF_IO_LOAN_IN_FLIGHT, memory_order_release);

#if defined(WF_IO_TESTING)
    if (atomic_exchange_explicit(&wf_io_fail_submission, 0, memory_order_acq_rel) != 0) {
        atomic_store_explicit(&frame->loan, WF_IO_LOAN_RETURNED, memory_order_release);
        atomic_store_explicit(&frame->state, WF_IO_FRAME_REAPED, memory_order_release);
        atomic_fetch_add_explicit(&wf_io_stat_submission_failures, 1, memory_order_relaxed);
        return -1;
    }
#endif

    atomic_store_explicit(&frame->state, WF_IO_FRAME_SUBMITTED, memory_order_release);
    wf_io_platform_mutex_lock(&wf_io_disk.mutex);
    if (wf_io_disk.tail == NULL) {
        wf_io_disk.head = frame;
    } else {
        wf_io_disk.tail->work_next = frame;
    }
    wf_io_disk.tail = frame;
    wf_io_platform_condition_signal(&wf_io_disk.available);
    wf_io_platform_mutex_unlock(&wf_io_disk.mutex);
    atomic_fetch_add_explicit(&wf_io_stat_submissions, 1, memory_order_relaxed);
    return 0;
}

void wf_io_wait(wf_io_frame *frame) {
    wf_io_lane *lane;
    uint64_t generation;
    if (frame == NULL || wf_io_initialize() != 0) {
        wf_io_defect();
    }
    lane = wf_io_lane_current();
    generation = atomic_load_explicit(&frame->generation, memory_order_acquire);
    if (atomic_load_explicit(&frame->owner_lane, memory_order_acquire) != lane->id
        || atomic_load_explicit(&frame->loan, memory_order_acquire) == WF_IO_LOAN_RETURNED) {
        wf_io_defect();
    }
    for (;;) {
        unsigned progress = wf_io_drain_mailbox(lane);
        if (atomic_load_explicit(&frame->observed_generation, memory_order_acquire) == generation) {
            if (atomic_load_explicit(&frame->state, memory_order_acquire) != WF_IO_FRAME_TERMINAL
                || atomic_load_explicit(&frame->loan, memory_order_acquire)
                    != WF_IO_LOAN_TERMINAL) {
                wf_io_defect();
            }
            atomic_store_explicit(&frame->loan, WF_IO_LOAN_RETURNED, memory_order_release);
            atomic_store_explicit(&frame->state, WF_IO_FRAME_REAPED, memory_order_release);
            return;
        }
        if (progress != 0) {
            /* Progress always rescans from the top before parking. */
            continue;
        }
        if (wf_io_help_depth == 0) {
            wf_io_help_fn help = atomic_load_explicit(&wf_io_help, memory_order_acquire);
            if (help != NULL) {
                void *context = atomic_load_explicit(&wf_io_help_context, memory_order_acquire);
                wf_io_help_depth = 1;
                progress = (unsigned)(help(context) != 0);
                wf_io_help_depth = 0;
                atomic_fetch_add_explicit(&wf_io_stat_help_calls, 1, memory_order_relaxed);
                if (progress != 0) {
                    continue;
                }
            }
        }

        /* Announce before the last scan.  A wake that races this scan remains
         * queued in kqueue/eventfd/IOCP and therefore cannot be lost. */
        atomic_store_explicit(&lane->announced, 1, memory_order_seq_cst);
        progress = wf_io_drain_mailbox(lane);
        if (progress != 0
            || atomic_load_explicit(&frame->observed_generation, memory_order_acquire)
                == generation) {
            atomic_store_explicit(&lane->announced, 0, memory_order_release);
            continue;
        }
        atomic_fetch_add_explicit(&wf_io_stat_parks, 1, memory_order_relaxed);
        wf_io_platform_park(lane->id);
        atomic_store_explicit(&lane->announced, 0, memory_order_release);
        /* Consuming any wake hint is progress: return to the top and scan all
         * sources rather than attempting a second park here. */
    }
}

unsigned wf_io_frame_state(const wf_io_frame *frame) {
    return atomic_load_explicit(&frame->state, memory_order_acquire);
}

unsigned wf_io_frame_loan(const wf_io_frame *frame) {
    return atomic_load_explicit(&frame->loan, memory_order_acquire);
}

uint64_t wf_io_frame_generation(const wf_io_frame *frame) {
    return atomic_load_explicit(&frame->generation, memory_order_acquire);
}

intptr_t wf_io_frame_result(const wf_io_frame *frame) {
    return frame->result;
}

int wf_io_frame_error(const wf_io_frame *frame) {
    return frame->error_code;
}

unsigned wf_io_frame_owner_lane(const wf_io_frame *frame) {
    return atomic_load_explicit(&frame->owner_lane, memory_order_acquire);
}

void wf_io_install_help_hook(wf_io_help_fn help, void *context) {
    atomic_store_explicit(&wf_io_help_context, context, memory_order_release);
    atomic_store_explicit(&wf_io_help, help, memory_order_release);
}

const char *wf_io_backend_name(void) {
    if (wf_io_initialize() != 0) {
        wf_io_defect();
    }
    return wf_io_platform_name();
}

unsigned wf_io_backend_features(void) {
    if (wf_io_initialize() != 0) {
        wf_io_defect();
    }
    return wf_io_platform_features() | WF_IO_BACKEND_FIXED_DISK_POOL;
}

wf_io_statistics wf_io_statistics_snapshot(void) {
    wf_io_statistics result;
    result.submissions = atomic_load_explicit(&wf_io_stat_submissions, memory_order_relaxed);
    result.submission_failures = atomic_load_explicit(
        &wf_io_stat_submission_failures,
        memory_order_relaxed
    );
    result.executions = atomic_load_explicit(&wf_io_stat_executions, memory_order_relaxed);
    result.completions = atomic_load_explicit(&wf_io_stat_completions, memory_order_relaxed);
    result.mailbox_batches = atomic_load_explicit(
        &wf_io_stat_mailbox_batches,
        memory_order_relaxed
    );
    result.parks = atomic_load_explicit(&wf_io_stat_parks, memory_order_relaxed);
    result.wakes = atomic_load_explicit(&wf_io_stat_wakes, memory_order_relaxed);
    result.help_calls = atomic_load_explicit(&wf_io_stat_help_calls, memory_order_relaxed);
    result.stale_publications = atomic_load_explicit(
        &wf_io_stat_stale_publications,
        memory_order_relaxed
    );
    result.duplicate_publications = atomic_load_explicit(
        &wf_io_stat_duplicate_publications,
        memory_order_relaxed
    );
    result.backend_poll_arms = wf_io_platform_poll_arms();
    return result;
}

int wf_io_is_disk_worker(void) {
    return wf_io_disk_worker;
}

#if defined(WF_IO_TESTING)
void wf_io_test_fail_next_submission(void) {
    atomic_store_explicit(&wf_io_fail_submission, 1, memory_order_release);
}

int wf_io_test_validate_publication(wf_io_frame *frame, uint64_t generation) {
    return wf_io_validate_publication(frame, generation);
}
#endif

#if !defined(_WIN32)
typedef struct wf_io_openat_context {
    int directory;
    const char *path;
    int flags;
} wf_io_openat_context;

typedef struct wf_io_transfer_context {
    int descriptor;
    void *buffer;
    uintptr_t count;
} wf_io_transfer_context;

typedef struct wf_io_fstat_context {
    int descriptor;
    void *status;
} wf_io_fstat_context;

typedef struct wf_io_close_context {
    int descriptor;
} wf_io_close_context;

static wf_io_result wf_io_openat_work(void *opaque) {
    wf_io_openat_context *context = (wf_io_openat_context *)opaque;
    int value = openat(context->directory, context->path, context->flags);
    wf_io_result result = { (intptr_t)value, value < 0 ? errno : 0 };
    return result;
}

static wf_io_result wf_io_read_work(void *opaque) {
    wf_io_transfer_context *context = (wf_io_transfer_context *)opaque;
    ssize_t value = read(context->descriptor, context->buffer, (size_t)context->count);
    wf_io_result result = { (intptr_t)value, value < 0 ? errno : 0 };
    return result;
}

static wf_io_result wf_io_write_work(void *opaque) {
    wf_io_transfer_context *context = (wf_io_transfer_context *)opaque;
    ssize_t value = write(context->descriptor, context->buffer, (size_t)context->count);
    wf_io_result result = { (intptr_t)value, value < 0 ? errno : 0 };
    return result;
}

static wf_io_result wf_io_fstat_work(void *opaque) {
    wf_io_fstat_context *context = (wf_io_fstat_context *)opaque;
    int value = fstat(context->descriptor, (struct stat *)context->status);
    wf_io_result result = { (intptr_t)value, value < 0 ? errno : 0 };
    return result;
}

static wf_io_result wf_io_close_work(void *opaque) {
    wf_io_close_context *context = (wf_io_close_context *)opaque;
    int value = close(context->descriptor);
    wf_io_result result = { (intptr_t)value, value < 0 ? errno : 0 };
    return result;
}

static intptr_t wf_io_run_adapter(wf_io_frame *frame, wf_io_work_fn work, void *context) {
    wf_io_result direct;
    int submitted;
    if (wf_io_is_disk_worker()) {
        direct = work(context);
        errno = direct.error_code;
        return direct.value;
    }
    wf_io_frame_init(frame);
    submitted = wf_io_submit(frame, work, context);
    if (submitted != 0) {
        if (submitted == -1) {
            errno = EAGAIN;
            return -1;
        }
        wf_io_defect();
    }
    wf_io_wait(frame);
    errno = wf_io_frame_error(frame);
    return wf_io_frame_result(frame);
}

int wf__io_openat(int directory, const char *path, int flags, ...) {
    wf_io_frame frame;
    wf_io_openat_context context = { directory, path, flags };
    return (int)wf_io_run_adapter(&frame, wf_io_openat_work, &context);
}

intptr_t wf__io_read(int descriptor, void *buffer, uintptr_t count) {
    wf_io_frame frame;
    wf_io_transfer_context context = { descriptor, buffer, count };
    return wf_io_run_adapter(&frame, wf_io_read_work, &context);
}

intptr_t wf__io_write(int descriptor, const void *buffer, uintptr_t count) {
    wf_io_frame frame;
    wf_io_transfer_context context = { descriptor, (void *)buffer, count };
    return wf_io_run_adapter(&frame, wf_io_write_work, &context);
}

int wf__io_fstat(int descriptor, void *status) {
    wf_io_frame frame;
    wf_io_fstat_context context = { descriptor, status };
    return (int)wf_io_run_adapter(&frame, wf_io_fstat_work, &context);
}

int wf__io_close(int descriptor) {
    wf_io_frame frame;
    wf_io_close_context context = { descriptor };
    return (int)wf_io_run_adapter(&frame, wf_io_close_work, &context);
}

#if defined(__APPLE__)
typedef struct wf_io_enumerate_context {
    int descriptor;
    void *buffer;
    uintptr_t count;
    void *position;
} wf_io_enumerate_context;

extern ssize_t __getdirentries64(int, char *, size_t, off_t *);

static wf_io_result wf_io_enumerate_work(void *opaque) {
    wf_io_enumerate_context *context = (wf_io_enumerate_context *)opaque;
    ssize_t value = __getdirentries64(
        context->descriptor,
        (char *)context->buffer,
        (size_t)context->count,
        (off_t *)context->position
    );
    wf_io_result result = { (intptr_t)value, value < 0 ? errno : 0 };
    return result;
}

intptr_t wf__io_getdirentries64(
    int descriptor,
    void *buffer,
    uintptr_t count,
    void *position
) {
    wf_io_frame frame;
    wf_io_enumerate_context context = { descriptor, buffer, count, position };
    return wf_io_run_adapter(&frame, wf_io_enumerate_work, &context);
}
#endif
#endif
