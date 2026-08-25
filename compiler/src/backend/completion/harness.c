#define _POSIX_C_SOURCE 200809L

#include "contract.h"
#include "platform.h"

#include <stdio.h>
#include <stdlib.h>

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#else
#include <time.h>
#endif

#define WF_IO_HARNESS_FRAMES 257u
#define WF_IO_HARNESS_REUSES 128u
#define WF_IO_HARNESS_LANE_THREADS 3u

typedef struct harness_task {
    intptr_t input;
    unsigned delay_milliseconds;
} harness_task;

static _Atomic unsigned harness_help_active;
static _Atomic unsigned harness_help_max;
static _Atomic uint64_t harness_help_count;
static _Atomic uint64_t harness_mixed_remaining;
static _Atomic uint64_t harness_mixed_steps;

typedef struct harness_condvar_path {
    wf_io_mutex mutex;
    wf_io_condition changed;
    uint64_t requested;
    uint64_t completed;
    uint64_t rounds;
    unsigned exited;
} harness_condvar_path;

typedef struct harness_lane_group {
    wf_io_mutex mutex;
    wf_io_condition changed;
    unsigned completed;
} harness_lane_group;

typedef struct harness_lane_context {
    harness_lane_group *group;
    unsigned index;
    unsigned owner_lane;
    intptr_t result;
} harness_lane_context;

static void harness_fail(const char *message, int line) {
    fprintf(stderr, "completion harness failure at line %d: %s\n", line, message);
    exit(1);
}

#define HARNESS_CHECK(condition, message) \
    do { \
        if (!(condition)) { \
            harness_fail((message), __LINE__); \
        } \
    } while (0)

static void harness_delay(unsigned milliseconds) {
#if defined(_WIN32)
    Sleep((DWORD)milliseconds);
#else
    struct timespec duration;
    duration.tv_sec = (time_t)(milliseconds / 1000u);
    duration.tv_nsec = (long)(milliseconds % 1000u) * 1000000L;
    while (nanosleep(&duration, &duration) != 0) {
    }
#endif
}

static uint64_t harness_now_nanoseconds(void) {
#if defined(_WIN32)
    LARGE_INTEGER frequency;
    LARGE_INTEGER counter;
    HARNESS_CHECK(QueryPerformanceFrequency(&frequency), "performance frequency unavailable");
    HARNESS_CHECK(QueryPerformanceCounter(&counter), "performance counter unavailable");
    return (uint64_t)((counter.QuadPart * 1000000000ull) / frequency.QuadPart);
#else
    struct timespec now;
    HARNESS_CHECK(clock_gettime(CLOCK_MONOTONIC, &now) == 0, "monotonic clock unavailable");
    return (uint64_t)now.tv_sec * 1000000000ull + (uint64_t)now.tv_nsec;
#endif
}

static wf_io_result harness_work(void *opaque) {
    harness_task *task = (harness_task *)opaque;
    wf_io_result result;
    if (task->delay_milliseconds != 0) {
        harness_delay(task->delay_milliseconds);
    }
    result.value = task->input * 3 + 1;
    result.error_code = 0;
    return result;
}

static void harness_lane_worker(void *opaque) {
    harness_lane_context *context = (harness_lane_context *)opaque;
    wf_io_frame frame;
    harness_task task = { (intptr_t)context->index + 100, 2 };
    wf_io_frame_init(&frame);
    HARNESS_CHECK(wf_io_submit(&frame, harness_work, &task) == 0, "lane work did not submit");
    context->owner_lane = wf_io_frame_owner_lane(&frame);
    wf_io_wait(&frame);
    context->result = wf_io_frame_result(&frame);
    HARNESS_CHECK(
        wf_io_frame_state(&frame) == WF_IO_FRAME_REAPED
            && wf_io_frame_loan(&frame) == WF_IO_LOAN_RETURNED,
        "a lane worker returned before its frame and loan"
    );
    wf_io_platform_mutex_lock(&context->group->mutex);
    context->group->completed += 1;
    wf_io_platform_condition_signal(&context->group->changed);
    wf_io_platform_mutex_unlock(&context->group->mutex);
}

static int harness_help(void *unused) {
    unsigned active;
    unsigned recorded;
    (void)unused;
    active = atomic_fetch_add_explicit(&harness_help_active, 1, memory_order_acq_rel) + 1;
    recorded = atomic_load_explicit(&harness_help_max, memory_order_relaxed);
    while (recorded < active
        && !atomic_compare_exchange_weak_explicit(
            &harness_help_max,
            &recorded,
            active,
            memory_order_release,
            memory_order_relaxed
        )) {
    }
    atomic_fetch_add_explicit(&harness_help_count, 1, memory_order_relaxed);
    atomic_fetch_sub_explicit(&harness_help_active, 1, memory_order_acq_rel);
    return 0;
}

/* One bounded compute step. A large backlog remains available while a delayed
 * completion races it; the wait loop must both make compute progress and
 * observe the completion before draining the whole backlog. */
static int harness_mixed_help(void *unused) {
    uint64_t remaining = atomic_load_explicit(&harness_mixed_remaining, memory_order_relaxed);
    (void)unused;
    while (remaining != 0) {
        if (atomic_compare_exchange_weak_explicit(
                &harness_mixed_remaining,
                &remaining,
                remaining - 1,
                memory_order_acq_rel,
                memory_order_relaxed
            )) {
            atomic_fetch_add_explicit(&harness_mixed_steps, 1, memory_order_relaxed);
            return 1;
        }
    }
    return 0;
}

static void harness_condvar_worker(void *opaque) {
    harness_condvar_path *path = (harness_condvar_path *)opaque;
    wf_io_platform_mutex_lock(&path->mutex);
    while (path->completed < path->rounds) {
        while (path->requested == path->completed) {
            wf_io_platform_condition_wait(&path->changed, &path->mutex);
        }
        path->completed = path->requested;
        wf_io_platform_condition_signal(&path->changed);
    }
    path->exited = 1;
    wf_io_platform_condition_signal(&path->changed);
    wf_io_platform_mutex_unlock(&path->mutex);
}

static uint64_t harness_condvar_roundtrips(uint64_t rounds) {
    harness_condvar_path path;
    uint64_t started;
    uint64_t finished;
    uint64_t round;
    path.requested = 0;
    path.completed = 0;
    path.rounds = rounds;
    path.exited = 0;
    HARNESS_CHECK(wf_io_platform_mutex_init(&path.mutex) == 0, "condvar mutex init failed");
    HARNESS_CHECK(
        wf_io_platform_condition_init(&path.changed) == 0,
        "condvar condition init failed"
    );
    HARNESS_CHECK(
        wf_io_platform_thread_start(harness_condvar_worker, &path) == 0,
        "condvar baseline worker did not start"
    );
    started = harness_now_nanoseconds();
    wf_io_platform_mutex_lock(&path.mutex);
    for (round = 1; round <= rounds; round += 1) {
        path.requested = round;
        wf_io_platform_condition_signal(&path.changed);
        while (path.completed != round) {
            wf_io_platform_condition_wait(&path.changed, &path.mutex);
        }
    }
    while (path.exited == 0) {
        wf_io_platform_condition_wait(&path.changed, &path.mutex);
    }
    wf_io_platform_mutex_unlock(&path.mutex);
    finished = harness_now_nanoseconds();
    return finished - started;
}

static void harness_backend_contract(void) {
    unsigned features = wf_io_backend_features();
    HARNESS_CHECK(
        (features & WF_IO_BACKEND_FIXED_DISK_POOL) != 0,
        "the shared fixed-depth disk pool is absent"
    );
#if defined(__APPLE__)
    HARNESS_CHECK(
        (features & (WF_IO_BACKEND_KQUEUE | WF_IO_BACKEND_WAITER_THREAD))
            == (WF_IO_BACKEND_KQUEUE | WF_IO_BACKEND_WAITER_THREAD),
        "macOS did not select kqueue plus its nonblocking waiter"
    );
#elif defined(__linux__)
    HARNESS_CHECK(
        (features
            & (WF_IO_BACKEND_IO_URING
                | WF_IO_BACKEND_EVENTFD_POLL
                | WF_IO_BACKEND_POLL_REARM
                | WF_IO_BACKEND_UNIFIED_PARK))
            == (WF_IO_BACKEND_IO_URING
                | WF_IO_BACKEND_EVENTFD_POLL
                | WF_IO_BACKEND_POLL_REARM
                | WF_IO_BACKEND_UNIFIED_PARK),
        "Linux did not select the io_uring/eventfd unified park"
    );
#elif defined(_WIN32)
    HARNESS_CHECK(
        (features & (WF_IO_BACKEND_IOCP | WF_IO_BACKEND_UNIFIED_PARK))
            == (WF_IO_BACKEND_IOCP | WF_IO_BACKEND_UNIFIED_PARK),
        "Windows did not select the IOCP unified park"
    );
#else
#error "the completion harness has no expected backend on this host"
#endif
}

/* Each executing submitter receives its own mailbox/park endpoint. This
 * catches a backend that accidentally reuses a frame's home slot or one
 * process-global ring/port as execution-lane identity. */
static void harness_per_lane_affinity(void) {
    harness_lane_group group;
    harness_lane_context contexts[WF_IO_HARNESS_LANE_THREADS];
    unsigned left;
    unsigned right;
    group.completed = 0;
    HARNESS_CHECK(wf_io_platform_mutex_init(&group.mutex) == 0, "lane mutex init failed");
    HARNESS_CHECK(
        wf_io_platform_condition_init(&group.changed) == 0,
        "lane condition init failed"
    );
    for (left = 0; left < WF_IO_HARNESS_LANE_THREADS; left += 1) {
        contexts[left].group = &group;
        contexts[left].index = left;
        contexts[left].owner_lane = WF_IO_MAX_LANES;
        contexts[left].result = 0;
        HARNESS_CHECK(
            wf_io_platform_thread_start(harness_lane_worker, &contexts[left]) == 0,
            "lane-affinity worker did not start"
        );
    }
    wf_io_platform_mutex_lock(&group.mutex);
    while (group.completed != WF_IO_HARNESS_LANE_THREADS) {
        wf_io_platform_condition_wait(&group.changed, &group.mutex);
    }
    wf_io_platform_mutex_unlock(&group.mutex);
    for (left = 0; left < WF_IO_HARNESS_LANE_THREADS; left += 1) {
        HARNESS_CHECK(
            contexts[left].result == ((intptr_t)left + 100) * 3 + 1,
            "a lane-affinity result moved"
        );
        HARNESS_CHECK(
            contexts[left].owner_lane < WF_IO_MAX_LANES,
            "a lane-affinity worker received no owner lane"
        );
        for (right = left + 1; right < WF_IO_HARNESS_LANE_THREADS; right += 1) {
            HARNESS_CHECK(
                contexts[left].owner_lane != contexts[right].owner_lane,
                "different executing lanes shared one mailbox endpoint"
            );
        }
    }
}

/* Four fixed disk workers race publications into one lane's intrusive MPSC
 * mailbox.  The odd count crosses the bounded batch edge repeatedly. */
static void harness_mpsc_and_publication(void) {
    wf_io_frame frames[WF_IO_HARNESS_FRAMES];
    harness_task tasks[WF_IO_HARNESS_FRAMES];
    unsigned owner = WF_IO_MAX_LANES;
    unsigned index;
    for (index = 0; index < WF_IO_HARNESS_FRAMES; index += 1) {
        wf_io_frame_init(&frames[index]);
        tasks[index].input = (intptr_t)index;
        tasks[index].delay_milliseconds = (index % 31u == 0) ? 2u : 0u;
        HARNESS_CHECK(
            wf_io_submit(&frames[index], harness_work, &tasks[index]) == 0,
            "a valid preallocated frame was not submitted"
        );
        if (owner == WF_IO_MAX_LANES) {
            owner = wf_io_frame_owner_lane(&frames[index]);
        }
        HARNESS_CHECK(
            wf_io_frame_owner_lane(&frames[index]) == owner,
            "mailbox affinity did not follow the executing lane"
        );
    }
    for (index = 0; index < WF_IO_HARNESS_FRAMES; index += 1) {
        wf_io_wait(&frames[index]);
        HARNESS_CHECK(
            wf_io_frame_result(&frames[index]) == (intptr_t)index * 3 + 1,
            "release/acquire publication moved a result"
        );
        HARNESS_CHECK(wf_io_frame_error(&frames[index]) == 0, "a success gained an error");
        HARNESS_CHECK(
            wf_io_frame_state(&frames[index]) == WF_IO_FRAME_REAPED,
            "wait returned before the terminal frame was reaped"
        );
        HARNESS_CHECK(
            wf_io_frame_loan(&frames[index]) == WF_IO_LOAN_RETURNED,
            "wait returned before the in-flight loan ended"
        );
    }
}

static uint64_t harness_generation_reuse(void) {
    wf_io_frame frame;
    harness_task task = { 0, 0 };
    uint64_t previous = 0;
    uint64_t started;
    unsigned iteration;
    wf_io_frame_init(&frame);
    started = harness_now_nanoseconds();
    for (iteration = 0; iteration < WF_IO_HARNESS_REUSES; iteration += 1) {
        task.input = (intptr_t)iteration;
        HARNESS_CHECK(wf_io_submit(&frame, harness_work, &task) == 0, "frame reuse failed");
        HARNESS_CHECK(
            wf_io_frame_generation(&frame) == previous + 1,
            "a reused frame did not advance its generation"
        );
        previous = wf_io_frame_generation(&frame);
        wf_io_wait(&frame);
        HARNESS_CHECK(
            wf_io_frame_result(&frame) == (intptr_t)iteration * 3 + 1,
            "a generation observed another generation's result"
        );
    }
    return harness_now_nanoseconds() - started;
}

static void harness_submission_failure_and_late_completion(void) {
    wf_io_frame frame;
    harness_task task = { 7, 20 };
    uint64_t first;
    uint64_t second;
    wf_io_frame_init(&frame);
    wf_io_test_fail_next_submission();
    HARNESS_CHECK(
        wf_io_submit(&frame, harness_work, &task) == -1,
        "the forced submission failure was not reported"
    );
    HARNESS_CHECK(
        wf_io_frame_state(&frame) == WF_IO_FRAME_REAPED
            && wf_io_frame_loan(&frame) == WF_IO_LOAN_RETURNED,
        "submission failure retained a frame or loan"
    );

    HARNESS_CHECK(wf_io_submit(&frame, harness_work, &task) == 0, "retry did not submit");
    first = wf_io_frame_generation(&frame);
    HARNESS_CHECK(
        wf_io_frame_loan(&frame) == WF_IO_LOAN_IN_FLIGHT,
        "submission did not establish the exclusive in-flight loan"
    );
    HARNESS_CHECK(
        wf_io_submit(&frame, harness_work, &task) == -2,
        "an in-flight frame was admitted for reuse"
    );
    wf_io_wait(&frame);

    task.delay_milliseconds = 0;
    HARNESS_CHECK(wf_io_submit(&frame, harness_work, &task) == 0, "second generation failed");
    second = wf_io_frame_generation(&frame);
    HARNESS_CHECK(second == first + 1, "the retry generation did not advance exactly once");
    wf_io_wait(&frame);
    HARNESS_CHECK(
        wf_io_test_validate_publication(&frame, first) == WF_IO_TEST_PUBLICATION_STALE,
        "a late completion was not quarantined by generation"
    );
    HARNESS_CHECK(
        wf_io_test_validate_publication(&frame, second) == WF_IO_TEST_PUBLICATION_DUPLICATE,
        "a duplicate terminal publication was not identified"
    );
}

static void harness_park_and_bounded_help(void) {
    wf_io_frame frame;
    harness_task task = { 11, 25 };
    wf_io_statistics before;
    wf_io_statistics after;
    wf_io_frame_init(&frame);
    wf_io_install_help_hook(harness_help, NULL);
    before = wf_io_statistics_snapshot();
    HARNESS_CHECK(wf_io_submit(&frame, harness_work, &task) == 0, "delayed work failed");
    wf_io_wait(&frame);
    after = wf_io_statistics_snapshot();
    HARNESS_CHECK(after.parks > before.parks, "the forced wait never exercised park/wake");
    HARNESS_CHECK(
        atomic_load_explicit(&harness_help_count, memory_order_relaxed) != 0,
        "completion join never offered its bounded compute-help slot"
    );
    HARNESS_CHECK(
        atomic_load_explicit(&harness_help_max, memory_order_relaxed) == 1,
        "join helping nested beyond its one-frame depth bound"
    );
    wf_io_install_help_hook(NULL, NULL);
}

static void harness_mixed_load_fairness(void) {
    wf_io_frame frame;
    harness_task task = { 13, 10 };
    wf_io_frame_init(&frame);
    atomic_store_explicit(&harness_mixed_remaining, 10000000, memory_order_relaxed);
    atomic_store_explicit(&harness_mixed_steps, 0, memory_order_relaxed);
    wf_io_install_help_hook(harness_mixed_help, NULL);
    HARNESS_CHECK(wf_io_submit(&frame, harness_work, &task) == 0, "mixed work did not submit");
    wf_io_wait(&frame);
    wf_io_install_help_hook(NULL, NULL);
    HARNESS_CHECK(wf_io_frame_result(&frame) == 40, "mixed completion result moved");
    HARNESS_CHECK(
        atomic_load_explicit(&harness_mixed_steps, memory_order_relaxed) != 0,
        "mixed wait made no compute progress"
    );
    HARNESS_CHECK(
        atomic_load_explicit(&harness_mixed_remaining, memory_order_relaxed) != 0,
        "completion starved behind the entire compute backlog"
    );
}

int main(void) {
    wf_io_statistics statistics;
    uint64_t completion_elapsed;
    uint64_t condvar_elapsed;
    uint64_t completion_per_round;
    uint64_t condvar_per_round;
    uint64_t overhead_milli;
    harness_backend_contract();
    harness_mpsc_and_publication();
    harness_per_lane_affinity();
    completion_elapsed = harness_generation_reuse();
    harness_submission_failure_and_late_completion();
    harness_park_and_bounded_help();
    harness_mixed_load_fairness();
    condvar_elapsed = harness_condvar_roundtrips(WF_IO_HARNESS_REUSES);
    statistics = wf_io_statistics_snapshot();
    HARNESS_CHECK(
        statistics.submissions == statistics.executions
            && statistics.executions == statistics.completions,
        "a successful submission did not make exactly one terminal transition"
    );
    HARNESS_CHECK(statistics.submission_failures == 1, "submission failures drifted");
    HARNESS_CHECK(statistics.mailbox_batches != 0, "no mailbox batch was consumed");
    HARNESS_CHECK(statistics.wakes >= statistics.completions, "enqueue preceded no wake");
    HARNESS_CHECK(statistics.stale_publications == 1, "late-completion count drifted");
    HARNESS_CHECK(statistics.duplicate_publications == 1, "duplicate count drifted");
    HARNESS_CHECK(statistics.backend_poll_arms != 0, "the backend installed no park endpoint");
#if defined(__linux__)
    if ((wf_io_backend_features() & WF_IO_BACKEND_POLL_MULTISHOT) == 0) {
        HARNESS_CHECK(
            statistics.backend_poll_arms > 1,
            "the one-shot eventfd poll was not rearmed after consumption"
        );
    }
#endif
    completion_per_round = completion_elapsed / WF_IO_HARNESS_REUSES;
    condvar_per_round = condvar_elapsed / WF_IO_HARNESS_REUSES;
    HARNESS_CHECK(completion_per_round != 0, "completion timing had no resolution");
    HARNESS_CHECK(condvar_per_round != 0, "condvar timing had no resolution");
    overhead_milli = (completion_per_round * 1000) / condvar_per_round;
    printf(
        "completion-harness backend=%s features=0x%x submissions=%llu parks=%llu "
        "mailbox_batches=%llu poll_arms=%llu completion_ns=%llu condvar_ns=%llu "
        "overhead_milli=%llu mixed_steps=%llu\n",
        wf_io_backend_name(),
        wf_io_backend_features(),
        (unsigned long long)statistics.submissions,
        (unsigned long long)statistics.parks,
        (unsigned long long)statistics.mailbox_batches,
        (unsigned long long)statistics.backend_poll_arms,
        (unsigned long long)completion_per_round,
        (unsigned long long)condvar_per_round,
        (unsigned long long)overhead_milli,
        (unsigned long long)atomic_load_explicit(&harness_mixed_steps, memory_order_relaxed)
    );
    return 0;
}
