#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* The online-CPU query the helper budget is checked against is a BSD
 * extension the POSIX macro above hides on Darwin, exactly as in bridge.c. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
#include "native_contract.h"
#include "../sched/core.h"

#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <pthread.h>
#include <poll.h>
#include <sched.h>
#include <stdarg.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

/* The runtime's own answer to the window query when nothing else bounds it. A
 * test which means to reach that boundary must name the same number the bridge
 * does.
 *
 * It used to be half the process-wide operation capacity.  There is no
 * operation capacity any more -- the record is a block of the submitting frame
 * -- so the number is now a throughput choice of the bridge's own, and it is
 * the same number (design §7). */
#define WF_HARNESS_WINDOW_DEFAULT 32u
/* The private storage the window query affords one loop before the compiler's
 * ceiling and the loop's own slot size apply.  A test which means to reach
 * that boundary must name the same number the bridge does. */
#define WF_HARNESS_WINDOW_BYTE_BUDGET (4u * 1024u * 1024u)

/* One operation record block, exactly as an emitted frame reserves it: the
 * size and alignment the contract states, and no knowledge of the layout. */
typedef struct wf_harness_record {
    _Alignas(WF_COMPLETION_RECORD_ALIGN)
        unsigned char bytes[WF_COMPLETION_RECORD_BYTES];
} wf_harness_record;

#define CHECK(condition)                                                      \
    do {                                                                      \
        if (!(condition)) {                                                   \
            fprintf(                                                          \
                stderr,                                                       \
                "completion harness: %s:%d: check failed: %s\n",            \
                __FILE__,                                                     \
                __LINE__,                                                     \
                #condition                                                    \
            );                                                                \
            return 1;                                                         \
        }                                                                     \
    } while (0)

#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
static unsigned wf_directory_host_calls;
static unsigned wf_directory_poll_calls;
static int wf_directory_poll_descriptor;
static short wf_directory_poll_events;
static int wf_directory_poll_timeout;

/* Deterministic enumeration answers for whichever family this build has:
 * interruption, readiness refusal, then one successful byte. The pipe
 * descriptor used by the test is already readable when the adapter performs
 * its exact POLLIN wait.
 *
 * One body serves both families; only the signature differs, because Darwin's
 * facility carries a base-position cell and Linux's keeps the whole cursor in
 * the descriptor. The facility this stands in for is selected by the same
 * macro the file adapter uses, so the adapter is exercised exactly as shipped
 * with only the host call replaced. */
static ssize_t wf_completion_test_directory_batch(void *buffer, size_t count) {
    wf_directory_host_calls += 1;
    if (wf_directory_host_calls == 1) {
        errno = EINTR;
        return -1;
    }
    if (wf_directory_host_calls == 2) {
        errno = EAGAIN;
        return -1;
    }
    if (buffer == NULL || count == 0) {
        errno = EINVAL;
        return -1;
    }
    ((unsigned char *)buffer)[0] = 'd';
    return 1;
}

#if defined(__APPLE__)
ssize_t wf_completion_test_getdirentries64(
    int descriptor,
    void *buffer,
    size_t count,
    int64_t *position
) {
    ssize_t reported;
    (void)descriptor;
    if (position == NULL) {
        wf_directory_host_calls += 1;
        errno = EINVAL;
        return -1;
    }
    reported = wf_completion_test_directory_batch(buffer, count);
    if (reported > 0) {
        *position = 17;
    }
    return reported;
}
#else
ssize_t wf_completion_test_getdents64(int descriptor, void *buffer, size_t count) {
    (void)descriptor;
    return wf_completion_test_directory_batch(buffer, count);
}
#endif
#endif

/* The adapter's own `openat`, named so this build performs the same host
 * call the shipped build does. */
int wf_completion_test_openat(
    int directory,
    const char *path,
    int flags,
    ...
) {
    if ((flags & O_CREAT) != 0) {
        va_list arguments;
        mode_t mode;
        va_start(arguments, flags);
        mode = (mode_t)va_arg(arguments, int);
        va_end(arguments);
        return openat(directory, path, flags, mode);
    }
    return openat(directory, path, flags);
}

int wf_completion_test_poll(
    struct pollfd *descriptors,
    nfds_t count,
    int timeout
) {
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    if (wf_directory_host_calls != 0) {
        wf_directory_poll_calls += 1;
        wf_directory_poll_descriptor = count == 1 ? descriptors[0].fd : -1;
        wf_directory_poll_events = count == 1 ? descriptors[0].events : 0;
        wf_directory_poll_timeout = timeout;
    }
#endif
    return poll(descriptors, count, timeout);
}

/* ------------------------------------------------------------- retirements
 *
 * Retired with the record pool (design §7, "The record's pool machinery:
 * deleted, not answered").  Every property each of these asserted was a
 * property of that pool, and the pool is gone rather than answered: the record
 * is a block of the submitting frame, nothing claims it, nothing recycles it,
 * and no queue anywhere can refuse an operation.
 *
 *   - `test_capacity_and_product_state`: the slot phases, the milestone
 *     product and WF_COMPLETION_OWNERSHIP_COMPLETE.  A record has one state
 *     that goes PENDING to DONE once, and no product of independent facts.
 *   - `test_generation_and_duplicate_terminal`: the generation and the
 *     duplicate-terminal refusal.  Exactly one terminal per submission is now
 *     an impossibility rather than a checked refusal, and it is tested in its
 *     new form by `test_exactly_one_completion_per_submission_under_race`.
 *   - `test_named_drain_refuses_a_recycled_slot`: a recycled slot.  There is
 *     no slot and no recycling; the frame joins its record before any
 *     terminator, so no publisher can meet a reused one.
 *   - `test_concurrent_single_claims_are_unique`: claim uniqueness.  There is
 *     nothing to claim; each frame supplies its own record, which is what the
 *     race test above submits from many threads at once.
 *   - `test_bounded_drain_and_lane_independence`: the bounded drain and the
 *     ready-event count it moved.  A completion is published straight into its
 *     record and there is no event to drain.
 *   - `test_drain_wakes_the_registered_token_owner`: the consume-wait
 *     registration.  Its property -- a completion elsewhere wakes the thread
 *     that registered for it -- survives as the in-place waiter, and is tested
 *     by `test_a_completion_claims_an_in_place_registration` and
 *     `test_a_helper_completion_wakes_a_waiting_join`.
 *   - `test_bridge_capacity_falls_back_per_operation` and
 *     `test_open_capacity_refuses_and_resubmits`: the per-operation capacity
 *     fallback and the readmission after a release.  There is no capacity to
 *     exhaust, no refusal to fall back from, and the owner's rule forbids a
 *     per-operation blocking fallback; the property that replaces both is
 *     `test_more_operations_outstanding_than_the_old_capacity`.
 *   - `test_capacity_release_wakes_before_blocking_work`: the capacity
 *     notification and the ordering it bought.  `wf_completion_notify_capacity`
 *     and its six callers are deleted with the capacity they announced.
 *   - `test_a_name_no_record_can_hold_opens_directly`: the path-does-not-fit
 *     demotion and its counter.  An open's path bytes are the submitting
 *     frame's own and are never copied, so no name can be too long for a
 *     record; the property that replaces it is
 *     `test_a_name_no_pool_record_could_hold_takes_the_completion_path`.
 *   - The overwrite arm of `test_submitted_open_owns_its_path_bytes`, which
 *     rewrote the caller's buffer immediately after submitting.  The bytes are
 *     the frame's and [SYS-2]'s loan on them holds until the join (design §5),
 *     so rewriting them while the operation is outstanding is no longer a
 *     thing a conforming caller does; what remains -- that an open resolves
 *     the name it was given -- is asserted by
 *     `test_submitted_open_resolves_the_submitters_bytes`.
 */

/* ---------------------------------------------------------------- helpers */

typedef struct park_context {
    wf_completion_runtime *runtime;
    uint64_t epoch;
    enum wf_completion_park_result result;
} park_context;

static void *park_thread(void *opaque) {
    park_context *context = opaque;
    context->result = wf_completion_park_if_unchanged(
        context->runtime,
        context->epoch,
        5000
    );
    return NULL;
}

static int wait_until_parked_count(
    wf_completion_runtime *runtime,
    unsigned expected
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;
    for (attempts = 0; attempts < 5000; ++attempts) {
        if (wf_completion_parked_scheduler_count(runtime) == expected) {
            return 0;
        }
        (void)nanosleep(&delay, NULL);
    }
    return 1;
}

static int wait_until_parked(wf_completion_runtime *runtime) {
    return wait_until_parked_count(runtime, 1u);
}

static void record_host_wake(void *context) {
    _Atomic unsigned *count = context;
    atomic_fetch_add_explicit(count, 1, memory_order_relaxed);
}

/* One record, filled the way a submit fills it, for the cases that drive an
 * adapter directly rather than through the bridge. */
static void harness_record_init(
    wf_completion_record *record,
    enum wf_file_operation_kind kind
) {
    memset(record, 0, sizeof(*record));
    wf_sched_record_init(&record->sched);
    record->request.kind = kind;
    record->opened_descriptor = -1;
    record->open_outcome = WF_FILE_OPEN_SUCCEEDED;
}

static int harness_record_done(const wf_completion_record *record) {
    return __atomic_load_n(&record->sched.state, __ATOMIC_ACQUIRE)
        == WF_SCHED_DONE;
}

/* Submits one record to the bounded adapter and runs it on this thread. */
static int harness_run_on_adapter(
    wf_file_adapter *adapter,
    wf_completion_record *record
) {
    CHECK(wf_file_adapter_submit(adapter, record) == WF_FILE_TARGET_OWNS);
    CHECK(wf_file_adapter_progress(adapter, 1) == 1);
    CHECK(harness_record_done(record));
    return 0;
}

/* ------------------------------------------------- the record protocol */

#define WF_HARNESS_RACERS 12
#define WF_HARNESS_RACER_ROUNDS 64

typedef struct submission_race_context {
    int descriptor;
    unsigned lane;
    _Atomic unsigned *start;
    int failed;
} submission_race_context;

/* One lane submitting and joining through records on its own frames.
 *
 * Every record here is a block of this thread's own stack frame, which is what
 * the emitter reserves (design §5).  No two lanes can name one record, so the
 * property the old token pool checked by refusing a second terminal is checked
 * here by construction: each submission has exactly one publication, and the
 * whole run's publication count says so. */
static void *submission_racer(void *opaque) {
    submission_race_context *context = opaque;
    unsigned round;
    while (atomic_load_explicit(context->start, memory_order_acquire) == 0) {
        sched_yield();
    }
    for (round = 0; round < WF_HARNESS_RACER_ROUNDS; ++round) {
        wf_harness_record record;
        unsigned char byte = 0xff;
        uint64_t offset = (uint64_t)((context->lane + round) % 8u);
        int64_t value = -1;
        int error = -1;
        if (wf__completion_file_pread_submit(
                context->descriptor,
                &byte,
                1,
                offset,
                record.bytes
            ) != 1) {
            context->failed = 1;
            return NULL;
        }
        wf__completion_file_join(record.bytes, &value, &error);
        if (value != 1 || error != 0
            || byte != (unsigned char)('a' + offset)) {
            context->failed = 1;
            return NULL;
        }
    }
    return NULL;
}

/* Exactly one completion per submission, under a race.
 *
 * This is what replaces the token pool's duplicate-terminal refusal and its
 * claim-uniqueness check: twelve threads submit and join at once, each through
 * records on its own frames, and the runtime publishes exactly one completion
 * for each submission.  A second publication would show as a publication count
 * above the submission count; a lost one would hang the join, which the
 * harness watchdog reports by name. */
static int test_exactly_one_completion_per_submission_under_race(
    const char *scratch_directory
) {
    pthread_t threads[WF_HARNESS_RACERS];
    submission_race_context contexts[WF_HARNESS_RACERS];
    _Atomic unsigned start;
    uint64_t publications_before;
    uint64_t publications_after;
    char path[256];
    int descriptor;
    unsigned index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-race-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, "abcdefgh", 8) == 8);

    atomic_init(&start, 0);
    publications_before = wf__completion_publications();
    for (index = 0; index < WF_HARNESS_RACERS; ++index) {
        contexts[index].descriptor = descriptor;
        contexts[index].lane = index;
        contexts[index].start = &start;
        contexts[index].failed = 0;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                submission_racer,
                &contexts[index]
            ) == 0
        );
    }
    atomic_store_explicit(&start, 1, memory_order_release);
    for (index = 0; index < WF_HARNESS_RACERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(contexts[index].failed == 0);
    }
    publications_after = wf__completion_publications();
    CHECK(
        publications_after - publications_before
        == (uint64_t)WF_HARNESS_RACERS * WF_HARNESS_RACER_ROUNDS
    );

    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* A publisher claims an in-place registration before it stores DONE.
 *
 * This is the I/O half of §6's handshake with no stack to park: a thread that
 * waits in place registers `WF_SCHED_WAITER_IN_PLACE` on its own record, and
 * the one publisher takes that registration back with a compare-exchange
 * before storing DONE, so exactly one of the two owns the wake.  Checked
 * without a park so the answer is a fact rather than a sample: after the
 * publication the registration is gone and the state is DONE. */
static int test_a_completion_claims_an_in_place_registration(void) {
    wf_completion_record record;

    harness_record_init(&record, WF_FILE_PREAD);
    CHECK(record.sched.state == WF_SCHED_PENDING);
    CHECK(record.sched.waiter == NULL);
    record.sched.waiter = WF_SCHED_WAITER_IN_PLACE;
    record.result.kind = WF_FILE_PREAD;
    record.result.value = 7;
    wf_completion_record_complete(&record);
    CHECK(harness_record_done(&record));
    CHECK(record.sched.waiter == NULL);
    CHECK(record.result.value == 7);

    /* A record no one waits on is published just the same, and the publisher
     * touches nothing after DONE. */
    harness_record_init(&record, WF_FILE_CLOSE);
    record.result.kind = WF_FILE_CLOSE;
    wf_completion_record_complete(&record);
    CHECK(harness_record_done(&record));
    CHECK(record.sched.waiter == NULL);
    return 0;
}

/* ------------------------------------------------------- the wake epoch */

static int test_unified_wake_epoch(void) {
    wf_completion_runtime runtime;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked;
    pthread_t thread;
    _Atomic unsigned host_wakes;
    uint64_t epoch_before;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            record_host_wake,
            &host_wakes
        ) == 0
    );
    before = wf_completion_statistics_snapshot(&runtime);
    epoch_before = wf_completion_wake_epoch(&runtime);
    /* A transition with no announced sleeper raises the epoch and performs no
     * host wake at all. */
    wf_completion_notify_target(&runtime);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.wake_signals == before.wake_signals);
    CHECK(after.target_notifications == before.target_notifications + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 0);
    CHECK(wf_completion_wake_epoch(&runtime) == epoch_before + 1);
    /* And a park against the epoch it raised does not sleep. */
    CHECK(
        wf_completion_park_if_unchanged(&runtime, epoch_before, 1)
        == WF_COMPLETION_PARK_EPOCH_CHANGED
    );

    /* An announced sleeper is woken once, through the host endpoint and the
     * condition variable both. */
    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_thread, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    wf_completion_notify_compute(&runtime);
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(after.compute_notifications == before.compute_notifications + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 1);

    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_one_epoch_wakes_every_announced_thread(void) {
    wf_completion_runtime runtime;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked[2];
    pthread_t threads[2];
    _Atomic unsigned host_wakes;
    uint64_t epoch;
    size_t index;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            record_host_wake,
            &host_wakes
        ) == 0
    );
    epoch = wf_completion_wake_epoch(&runtime);
    for (index = 0; index < 2; ++index) {
        parked[index].runtime = &runtime;
        parked[index].epoch = epoch;
        parked[index].result = WF_COMPLETION_PARK_FAILED;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                park_thread,
                &parked[index]
            ) == 0
        );
    }
    CHECK(wait_until_parked_count(&runtime, 2u) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    wf_completion_notify_compute(&runtime);
    for (index = 0; index < 2; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(parked[index].result == WF_COMPLETION_PARK_WOKEN);
    }
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.compute_notifications == before.compute_notifications + 1);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 1);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

/* --------------------------------------------------------- the adapters */

static int test_linux_independent_operations_use_available_target(
    const char *scratch_directory
) {
#if defined(__linux__)
    wf_harness_record records[2];
    unsigned char bytes[2] = {0};
    int64_t value;
    int error_code;
    uint64_t native_before;
    uint64_t fallback_before;
    uint64_t native_after;
    uint64_t fallback_after;
    char path[256];
    int descriptor;

    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-native-operations-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, "xy", 2) == 2);
    native_before = wf__completion_linux_io_uring_submissions();
    fallback_before = wf__completion_file_fallback_submissions();

    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            &bytes[0],
            1,
            0,
            records[0].bytes
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            &bytes[1],
            1,
            1,
            records[1].bytes
        ) == 1
    );
    wf__completion_file_join(records[0].bytes, &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    wf__completion_file_join(records[1].bytes, &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(bytes[0] == 'x' && bytes[1] == 'y');

    native_after = wf__completion_linux_io_uring_submissions();
    fallback_after = wf__completion_file_fallback_submissions();
    if (native_after == native_before + 2) {
        CHECK(fallback_after == fallback_before);
        CHECK(wf__completion_target_helper_count() == 0);
    } else {
        CHECK(getenv("WF_REQUIRE_LINUX_IO_URING") == NULL);
        CHECK(native_after == native_before);
        CHECK(fallback_after == fallback_before + 2);
    }
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
#else
    (void)scratch_directory;
#endif
    return 0;
}

/* Every typed operation the bounded adapter carries, driven by one thread
 * that is itself the queue's only engine.
 *
 * The backpressure arm this case used to carry -- a queue of one, a second
 * submission answered WAIT_CAPACITY, and the same request accepted after a
 * progress pass -- is retired with the fixed queue (design §7).  The list is
 * threaded through the records, so a second submission is queued rather than
 * refused, and this case now submits two before running either. */
static int test_single_thread_file_progress(const char *scratch_directory) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    wf_completion_record open_record;
    wf_completion_record bad_close;
    wf_completion_record operation;
    unsigned char status[WF_FILE_STATUS_CAPACITY];
    char name[96];
    char read_back[32] = {0};
    const char payload[] = "completion-core";
    int root;
    int descriptor;

    CHECK(scratch_directory != NULL);
    root = open(scratch_directory, O_RDONLY | O_DIRECTORY);
    CHECK(root >= 0);
    (void)snprintf(
        name,
        sizeof(name),
        "wf-completion-harness-%ld",
        (long)getpid()
    );
    (void)unlinkat(root, name, 0);

    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, NULL, 0, 0) == 0);

    harness_record_init(&open_record, WF_FILE_OPEN_AT);
    open_record.request.operation.open_at.directory = root;
    open_record.request.operation.open_at.path = name;
    open_record.request.operation.open_at.flags = O_CREAT | O_EXCL | O_RDWR;
    open_record.request.operation.open_at.mode = 0600;
    open_record.request.operation.open_at.has_mode = 1;
    harness_record_init(&bad_close, WF_FILE_CLOSE);
    bad_close.request.operation.close.descriptor = -1;

    /* Two operations queued at once: the list has no capacity to refuse. */
    CHECK(
        wf_file_adapter_submit(&adapter, &open_record) == WF_FILE_TARGET_OWNS
    );
    CHECK(
        wf_file_adapter_submit(&adapter, &bad_close) == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_queued(&adapter) == 2);
    CHECK(wf_file_adapter_progress(&adapter, 2) == 2);
    CHECK(harness_record_done(&open_record));
    CHECK(harness_record_done(&bad_close));
    CHECK(open_record.result.error_code == 0);
    CHECK(open_record.result.value >= 0);
    descriptor = (int)open_record.result.value;
    CHECK(bad_close.result.value == -1);
    CHECK(bad_close.result.error_code == EBADF);

    harness_record_init(&operation, WF_FILE_PWRITE);
    operation.request.operation.pwrite.descriptor = descriptor;
    operation.request.operation.pwrite.buffer = payload;
    operation.request.operation.pwrite.count = sizeof(payload) - 1;
    operation.request.operation.pwrite.offset = 0;
    CHECK(harness_run_on_adapter(&adapter, &operation) == 0);
    CHECK(operation.result.error_code == 0);
    CHECK(operation.result.value == (int64_t)(sizeof(payload) - 1));

    harness_record_init(&operation, WF_FILE_PREAD);
    operation.request.operation.pread.descriptor = descriptor;
    operation.request.operation.pread.buffer = read_back;
    operation.request.operation.pread.count = sizeof(payload) - 1;
    operation.request.operation.pread.offset = 0;
    CHECK(harness_run_on_adapter(&adapter, &operation) == 0);
    CHECK(operation.result.error_code == 0);
    CHECK(operation.result.value == (int64_t)(sizeof(payload) - 1));
    CHECK(memcmp(read_back, payload, sizeof(payload) - 1) == 0);

    /* A status writes into the destination its own request named; the record
     * carries the size and never the bytes. */
    memset(status, 0, sizeof(status));
    harness_record_init(&operation, WF_FILE_STATUS);
    operation.request.operation.status.descriptor = descriptor;
    operation.request.operation.status.destination = status;
    operation.request.operation.status.capacity = sizeof(status);
    CHECK(harness_run_on_adapter(&adapter, &operation) == 0);
    CHECK(operation.result.error_code == 0);
    CHECK(operation.result.value == 0);
    CHECK(operation.status_written == sizeof(struct stat));

    harness_record_init(&operation, WF_FILE_CLOSE);
    operation.request.operation.close.descriptor = descriptor;
    CHECK(harness_run_on_adapter(&adapter, &operation) == 0);
    CHECK(operation.result.error_code == 0);
    CHECK(operation.result.value == 0);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(unlinkat(root, name, 0) == 0);
    CHECK(close(root) == 0);
    return 0;
}

/* ---------------------------------------------------------- the bridge */

static int test_bridge_independent_positioned_reads(
    const char *scratch_directory
) {
    wf_harness_record first_record;
    wf_harness_record second_record;
    int64_t first_value = -1;
    int64_t second_value = -1;
    int first_error = -1;
    int second_error = -1;
    char first[5] = {0};
    char second[5] = {0};
    const char payload[] = "zeroONE-twoTHREE";
    char path[256];
    int descriptor;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-pread-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(
        pwrite(descriptor, payload, sizeof(payload) - 1, 0)
        == (ssize_t)(sizeof(payload) - 1)
    );

    /* Both records are submitted before either result is joined.  The two
     * requests name one descriptor but independent file offsets; pread leaves
     * the shared cursor out of their semantics.
     *
     * A written WF_IO_HELPERS keeps both on the queued route, which is what
     * the submission count below asserts. */
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            4,
            4,
            first_record.bytes
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            second,
            4,
            11,
            second_record.bytes
        ) == 1
    );
    wf__completion_file_join(
        second_record.bytes,
        &second_value,
        &second_error
    );
    wf__completion_file_join(first_record.bytes, &first_value, &first_error);
    CHECK(first_error == 0);
    CHECK(second_error == 0);
    CHECK(first_value == 4);
    CHECK(second_value == 4);
    CHECK(memcmp(first, "ONE-", 4) == 0);
    CHECK(memcmp(second, "THRE", 4) == 0);
    CHECK(wf__completion_file_submissions() >= 2);
#if defined(__linux__)
    if (getenv("WF_REQUIRE_LINUX_IO_URING") != NULL) {
        CHECK(wf__completion_linux_io_uring_submissions() >= 2);
    }
#endif

    /* An offset the signed native type cannot represent is an argument this
     * ABI cannot mean, and a submit that receives one terminates rather than
     * answering: there is no `0` left for a caller to interpret, because there
     * is no second lowering to fall to (design §7, §8).  Reaching it needs a
     * child process, which this in-process harness deliberately does not
     * spawn; what it does assert is that every representable submission is
     * accepted. */

    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_bridge_open_status_and_close_are_typed_operations(
    const char *scratch_directory
) {
    wf_harness_record record;
    unsigned char status[WF_FILE_STATUS_CAPACITY];
    uint64_t status_size = 0;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char path[256];
    int descriptor;
    uint64_t native_before = wf__completion_linux_io_uring_submissions();

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-typed-lifecycle-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);

    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_CREAT | O_EXCL | O_RDWR,
            0600u,
            1u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    descriptor = (int)value;

    memset(status, 0, sizeof(status));
    CHECK(
        wf__completion_file_status_submit(
            descriptor,
            status,
            sizeof(status),
            record.bytes
        ) == 1
    );
    wf__completion_file_status_join(
        record.bytes,
        &value,
        &error_code,
        status,
        sizeof(status),
        &status_size
    );
    CHECK(value == 0 && error_code == 0);
    CHECK(status_size == sizeof(struct stat));

    CHECK(wf__completion_file_close_submit(descriptor, record.bytes) == 1);
    wf__completion_file_join(record.bytes, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    /* Closing a descriptor whose authority this program already gave up is a
     * typed refusal, not a crash and not a silent success. */
    CHECK(wf__completion_file_close_submit(descriptor, record.bytes) == 1);
    wf__completion_file_join(record.bytes, &value, &error_code);
    CHECK(value < 0 && error_code == EBADF);

#if defined(__linux__)
    /* Where the ring is the qualified target, the open and both closes above
     * are ring operations. Without this the whole test would still pass on a
     * silent fallback to the bounded POSIX adapter, which is exactly the
     * regression the ring path exists to prevent. */
    if (getenv("WF_REQUIRE_LINUX_IO_URING") != NULL) {
        CHECK(wf__completion_linux_io_uring_submissions() >= native_before + 3);
    }
#endif
    (void)native_before;

    descriptor = (int)wf__completion_file_open_at_direct(
        AT_FDCWD,
        path,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(
        descriptor >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    CHECK(
        wf__completion_file_status_direct(
            descriptor,
            status,
            sizeof(status)
        ) == 0
    );
    CHECK(wf__completion_file_close_direct(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* One file of exactly one byte, so the byte a later read returns names which
 * file was opened. */
static int wf_harness_write_marker_file(int root, const char *name, char byte) {
    int descriptor = openat(root, name, O_CREAT | O_TRUNC | O_WRONLY, 0600);
    if (descriptor < 0) {
        return -1;
    }
    if (write(descriptor, &byte, 1) != 1) {
        (void)close(descriptor);
        return -1;
    }
    return close(descriptor);
}

/* A submitted open resolves the bytes the submitting frame gave it, and two
 * outstanding opens resolve their own.
 *
 * Nothing is copied any more: the path is the frame's own storage and
 * [SYS-2]'s loan on it holds until the join, so the kernel or the helper
 * resolves the caller's bytes in place (design §5).  The property that
 * survives the copy's removal is the one an emitted program depends on --
 * that two independent opens outstanding at once each resolve their own name
 * -- and it is checked with two marker files whose byte says which was
 * opened.  Both legs run on every platform: the bridge leg reaches the
 * io_uring ring on Linux and the bounded POSIX adapter on Darwin. */
static int test_submitted_open_resolves_the_submitters_bytes(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    wf_completion_record record;
    wf_harness_record first_block;
    wf_harness_record second_block;
    char first[64];
    char second[64];
    char first_directory[64];
    char second_directory[64];
    char marker = 0;
    int64_t value = -1;
    int64_t other_value = -1;
    int error_code = -1;
    int other_error = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    unsigned other_outcome = WF_FILE_OPEN_FAILED;
    int root;
    int descriptor;
    int marker_descriptor;
    uint64_t native_before = wf__completion_linux_io_uring_submissions();

    CHECK(scratch_directory != NULL);
    root = open(scratch_directory, O_RDONLY | O_DIRECTORY);
    CHECK(root >= 0);
    CHECK(
        snprintf(first, sizeof(first), "wf-open-name-a-%ld", (long)getpid())
        > 0
    );
    CHECK(
        snprintf(second, sizeof(second), "wf-open-name-b-%ld", (long)getpid())
        > 0
    );
    CHECK(wf_harness_write_marker_file(root, first, 'A') == 0);
    CHECK(wf_harness_write_marker_file(root, second, 'B') == 0);

    /* The bounded POSIX adapter, driven by this thread alone, so the host call
     * demonstrably runs after the submission returned. */
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, NULL, 0, 0) == 0);
    harness_record_init(&record, WF_FILE_OPEN_AT);
    record.request.operation.open_at.directory = root;
    record.request.operation.open_at.path = first;
    record.request.operation.open_at.flags = O_RDONLY;
    record.request.operation.open_at.expected_kind = WF_FILE_EXPECT_REGULAR;
    CHECK(wf_file_adapter_submit(&adapter, &record) == WF_FILE_TARGET_OWNS);
    /* The request names the caller's own bytes; nothing was copied. */
    CHECK(record.request.operation.open_at.path == first);
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(harness_record_done(&record));
    CHECK(record.result.error_code == 0 && record.result.value >= 0);
    CHECK(record.result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    descriptor = (int)record.result.value;
    CHECK(pread(descriptor, &marker, 1, 0) == 1);
    CHECK(marker == 'A');
    CHECK(close(descriptor) == 0);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);

    /* The shipped bridge, with both opens outstanding at once. */
    CHECK(
        wf__completion_file_open_at_submit(
            root,
            first,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            first_block.bytes
        ) == 1
    );
    CHECK(
        wf__completion_file_open_at_submit(
            root,
            second,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            second_block.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        second_block.bytes,
        &other_value,
        &other_error,
        &other_outcome
    );
    wf__completion_file_open_join(
        first_block.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    CHECK(
        other_value >= 0 && other_error == 0
        && other_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    marker = 0;
    CHECK(pread((int)value, &marker, 1, 0) == 1);
    CHECK(marker == 'A');
    marker = 0;
    CHECK(pread((int)other_value, &marker, 1, 0) == 1);
    CHECK(marker == 'B');
    CHECK(wf__completion_file_close_direct((int)value) == 0);
    CHECK(wf__completion_file_close_direct((int)other_value) == 0);

    /* `open_directory` carries its name through exactly the same request, so
     * it is checked the same way: the marker file exists only under the first
     * directory, and finding it proves which name was resolved. */
    CHECK(
        snprintf(
            first_directory,
            sizeof(first_directory),
            "wf-open-dir-a-%ld",
            (long)getpid()
        ) > 0
    );
    CHECK(
        snprintf(
            second_directory,
            sizeof(second_directory),
            "wf-open-dir-b-%ld",
            (long)getpid()
        ) > 0
    );
    (void)unlinkat(root, first_directory, AT_REMOVEDIR);
    (void)unlinkat(root, second_directory, AT_REMOVEDIR);
    CHECK(mkdirat(root, first_directory, 0700) == 0);
    CHECK(mkdirat(root, second_directory, 0700) == 0);
    marker_descriptor = openat(root, first_directory, O_RDONLY | O_DIRECTORY);
    CHECK(marker_descriptor >= 0);
    CHECK(wf_harness_write_marker_file(marker_descriptor, "marker", 'A') == 0);
    CHECK(close(marker_descriptor) == 0);

    CHECK(
        wf__completion_file_open_at_submit(
            root,
            first_directory,
            O_RDONLY | O_DIRECTORY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            first_block.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        first_block.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    descriptor = (int)value;
    marker_descriptor = openat(descriptor, "marker", O_RDONLY);
    CHECK(marker_descriptor >= 0);
    CHECK(close(marker_descriptor) == 0);
    CHECK(wf__completion_file_close_direct(descriptor) == 0);

#if defined(__linux__)
    /* Where the ring is the qualified target, the bridge opens above are ring
     * operations. Without this the leg would still pass on a silent fallback
     * to the bounded adapter, which is the other half of what this test is
     * about. */
    if (getenv("WF_REQUIRE_LINUX_IO_URING") != NULL) {
        CHECK(wf__completion_linux_io_uring_submissions() >= native_before + 3);
    }
#endif
    (void)native_before;

    marker_descriptor = openat(root, first_directory, O_RDONLY | O_DIRECTORY);
    CHECK(marker_descriptor >= 0);
    CHECK(unlinkat(marker_descriptor, "marker", 0) == 0);
    CHECK(close(marker_descriptor) == 0);
    CHECK(unlinkat(root, first, 0) == 0);
    CHECK(unlinkat(root, second, 0) == 0);
    CHECK(unlinkat(root, first_directory, AT_REMOVEDIR) == 0);
    CHECK(unlinkat(root, second_directory, AT_REMOVEDIR) == 0);
    CHECK(close(root) == 0);
    return 0;
}

#define WF_HARNESS_LONG_COMPONENTS 5u
#define WF_HARNESS_LONG_COMPONENT_BYTES 240u
#define WF_HARNESS_LONG_PATH_BYTES                                            \
    (WF_HARNESS_LONG_COMPONENTS * (WF_HARNESS_LONG_COMPONENT_BYTES + 1u) + 16u)
/* The path storage the deleted pool record held.  No runtime constant names
 * it any more; the number is written here so the case that used to be about
 * exceeding it can still say what it exceeds. */
#define WF_HARNESS_RETIRED_PATH_CAPACITY 1024u

/* A name no pool record could have held takes the completion path.
 *
 * This is the inverse of the case it replaces.  An operation record used to
 * copy an open's path into 1024 bytes of its own and refuse a longer name
 * before anything was claimed, so `open_read` -- which resolves the caller's
 * whole path buffer, and which [PATH-1] admits at any length -- lost the
 * completion path for a name an ordinary program can write, and the runtime
 * counted the demotion.  The path bytes are the submitting frame's own now and
 * are never copied, so there is no length a record cannot hold, no refusal,
 * and no counter: the same name is submitted, joined, and answered by the
 * engine like any other operation (design §5, §7).
 *
 * Linux resolves the whole 1200-byte name (`PATH_MAX` is 4096) and the marker
 * file is really opened.  Darwin's `PATH_MAX` is smaller, so there the host
 * itself refuses the name -- and the point holds either way: the outcome is
 * the host's own, delivered through the completion path rather than around
 * it. */
static int test_a_name_no_pool_record_could_hold_takes_the_completion_path(
    const char *scratch_directory
) {
    char relative[WF_HARNESS_LONG_PATH_BYTES];
    wf_harness_record record;
    size_t length = 0;
    size_t built = 0;
    size_t level;
    uint64_t submissions_before;
    int root;
    int descriptor;
    int host_error = 0;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char marker = 0;

    CHECK(scratch_directory != NULL);
    root = open(scratch_directory, O_RDONLY | O_DIRECTORY);
    CHECK(root >= 0);
    /* One component is at most NAME_MAX on every host this runs on, so the
     * length that used to exceed the record comes from nesting rather than
     * from one outsized name.  A host that refuses the depth stops the tree
     * there and `built` records how much of it exists. */
    for (level = 0; level < WF_HARNESS_LONG_COMPONENTS; ++level) {
        if (level != 0) {
            relative[length] = '/';
            length += 1u;
        }
        memset(relative + length, 'd', WF_HARNESS_LONG_COMPONENT_BYTES);
        length += WF_HARNESS_LONG_COMPONENT_BYTES;
        relative[length] = 0;
        if (mkdirat(root, relative, 0700) == 0 || errno == EEXIST) {
            built = length;
            continue;
        }
        CHECK(errno == ENAMETOOLONG);
    }
    memcpy(relative + length, "/marker", sizeof("/marker"));
    length += sizeof("/marker") - 1u;
    CHECK(length > (size_t)WF_HARNESS_RETIRED_PATH_CAPACITY);
    descriptor = openat(root, relative, O_CREAT | O_TRUNC | O_WRONLY, 0600);
    if (descriptor < 0) {
        host_error = errno;
        CHECK(host_error == ENAMETOOLONG);
    } else {
        CHECK(write(descriptor, "L", 1) == 1);
        CHECK(close(descriptor) == 0);
    }

    submissions_before = wf__completion_file_submissions();
    CHECK(
        wf__completion_file_open_at_submit(
            root,
            relative,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    /* The operation really went to an engine, rather than being executed here
     * because a record could not hold its name. */
    CHECK(wf__completion_file_submissions() > submissions_before);
    if (host_error == 0) {
        CHECK(
            value >= 0 && error_code == 0
            && open_outcome == WF_FILE_OPEN_SUCCEEDED
        );
        CHECK(pread((int)value, &marker, 1, 0) == 1);
        CHECK(marker == 'L');
        CHECK(wf__completion_file_close_direct((int)value) == 0);
        CHECK(unlinkat(root, relative, 0) == 0);
    } else {
        CHECK(
            value < 0 && error_code == host_error
            && open_outcome == WF_FILE_OPEN_FAILED
        );
    }

    /* Remove exactly the directories that were created, deepest first. */
    length = built;
    while (length != 0) {
        relative[length] = 0;
        CHECK(unlinkat(root, relative, AT_REMOVEDIR) == 0);
        while (length != 0 && relative[length - 1u] != '/') {
            length -= 1u;
        }
        if (length != 0) {
            length -= 1u;
        }
    }
    CHECK(close(root) == 0);
    return 0;
}

/* More operations outstanding at once than the deleted pool could hold, and
 * deeper than the ring is.
 *
 * This replaces both capacity cases.  The old pool held 64 operations and
 * refused the sixty-fifth, and the ring refused a submission its entry array
 * could not take; the record is a block of the submitting frame now, so the
 * only bound is the frames that hold the records, and a full submission queue
 * is emptied by the submitting call's own `io_uring_enter` (design §7).  A
 * count past both old bounds is therefore the property to assert: every one is
 * accepted and every one completes with the byte at its own offset. */
#define WF_HARNESS_OUTSTANDING 96u

static int test_more_operations_outstanding_than_the_old_capacity(
    const char *scratch_directory
) {
    static wf_harness_record records[WF_HARNESS_OUTSTANDING];
    static unsigned char bytes[WF_HARNESS_OUTSTANDING];
    char path[256];
    int descriptor;
    unsigned index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-outstanding-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, "abcdefgh", 8) == 8);

    for (index = 0; index < WF_HARNESS_OUTSTANDING; ++index) {
        bytes[index] = 0xa5u;
        CHECK(
            wf__completion_file_pread_submit(
                descriptor,
                &bytes[index],
                1,
                (uint64_t)(index % 8u),
                records[index].bytes
            ) == 1
        );
    }
    for (index = 0; index < WF_HARNESS_OUTSTANDING; ++index) {
        int64_t value = -1;
        int error_code = -1;
        wf__completion_file_join(records[index].bytes, &value, &error_code);
        CHECK(value == 1 && error_code == 0);
        CHECK(bytes[index] == (unsigned char)('a' + (index % 8u)));
    }

    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_checked_open_rejects_and_closes_nonregular_descriptors(
    const char *scratch_directory
) {
    wf_harness_record record;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char path[256];

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-nonregular-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    CHECK(mkfifo(path, 0600) == 0);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        path,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        scratch_directory,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_IS_DIRECTORY);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    /* The same submitted refusal, so a target which carries opens natively
     * answers the same discriminator as one which does not. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            scratch_directory,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_IS_DIRECTORY);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    CHECK(unlink(path) == 0);
    return 0;
}

/* Every way an open can fail names its own terminal discriminator, on the
 * submitted path and on the direct one.  The generated program switches on
 * exactly this value and aborts on anything outside the set, so a target
 * which answers the wrong one is a fail-stop defect rather than a wrong
 * program. */
static int test_open_failure_classes_are_typed_outcomes(
    const char *scratch_directory
) {
    wf_harness_record record;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_SUCCEEDED;
    char missing[256];
    char regular[256];
    int descriptor;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            missing,
            sizeof(missing),
            "%s/wf-completion-absent-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    CHECK(
        snprintf(
            regular,
            sizeof(regular),
            "%s/wf-completion-regular-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(missing);
    (void)unlink(regular);
    descriptor = open(regular, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(close(descriptor) == 0);

    /* A name that does not resolve fails before any descriptor exists. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            missing,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value < 0);
    CHECK(error_code == ENOENT);
    CHECK(open_outcome == WF_FILE_OPEN_FAILED);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        missing,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value < 0);
    CHECK(error_code == ENOENT);
    CHECK(open_outcome == WF_FILE_OPEN_FAILED);

    /* A directory open of a regular file is refused by the same one rule
     * that refuses a regular open of a directory. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        regular,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_DIRECTORY,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    /* The directory the refusal above named really does open as a directory,
     * so the refusal is about the kind and not about the request shape. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            scratch_directory,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf__completion_file_close_submit((int)value, record.bytes) == 1);
    wf__completion_file_join(record.bytes, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(unlink(regular) == 0);
    return 0;
}

typedef struct wf_open_waiter {
    const char *path;
    unsigned yields;
    int result;
} wf_open_waiter;

static void *wf_open_waiter_main(void *opaque) {
    wf_open_waiter *waiter = opaque;
    wf_harness_record record;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    unsigned yield;
    waiter->result = 1;
    if (wf__completion_file_open_at_submit(
            AT_FDCWD,
            waiter->path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) != 1) {
        return NULL;
    }
    /* Give the engine every chance to finish before this owner asks for the
     * result, so the join exercises the already-published path rather than
     * only the park-and-wake one. */
    for (yield = 0; yield < waiter->yields; ++yield) {
        (void)sched_yield();
    }
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    if (value < 0 || error_code != 0
        || open_outcome != WF_FILE_OPEN_SUCCEEDED) {
        return NULL;
    }
    if (wf__completion_file_close_submit((int)value, record.bytes) != 1) {
        return NULL;
    }
    wf__completion_file_join(record.bytes, &value, &error_code);
    waiter->result = value == 0 && error_code == 0 ? 0 : 1;
    return NULL;
}

/* Independent owners each hold their own open operation at once. No owner
 * observes another's result, and an owner whose operation finished before it
 * asked still reads exactly its own record. */
static int test_open_results_reach_every_independent_owner(
    const char *scratch_directory
) {
    enum { WF_OPEN_WAITERS = 6u };
    pthread_t threads[WF_OPEN_WAITERS];
    wf_open_waiter waiters[WF_OPEN_WAITERS];
    char path[256];
    unsigned index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-open-waiters-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    {
        int seed = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
        CHECK(seed >= 0);
        CHECK(close(seed) == 0);
    }
    for (index = 0; index < WF_OPEN_WAITERS; ++index) {
        waiters[index].path = path;
        /* Half join immediately and half only after yielding, so both the
         * already-completed and the still-in-flight join are covered. */
        waiters[index].yields = index % 2u == 0u ? 0u : 64u;
        waiters[index].result = 1;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                wf_open_waiter_main,
                &waiters[index]
            ) == 0
        );
    }
    for (index = 0; index < WF_OPEN_WAITERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(waiters[index].result == 0);
    }
    CHECK(unlink(path) == 0);
    return 0;
}
/* The scripted clock the helper policy measures host calls with.
 *
 * Unscripted it is the host's own monotonic clock, so every other case in this
 * file measures what the shipped build measures.  A case that is about what
 * the growth rule *decides* scripts it instead, and then the decision is a
 * property of the rule rather than of how loaded the machine running the test
 * happened to be. */
static _Atomic uint64_t wf_scripted_clock_step_ns;
static _Atomic uint64_t wf_scripted_clock_now_ns;

uint64_t wf_completion_test_monotonic_ns(void) {
    uint64_t step = atomic_load_explicit(
        &wf_scripted_clock_step_ns,
        memory_order_relaxed
    );
    if (step == 0) {
        struct timespec now;
        if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
            return 0;
        }
        return (uint64_t)now.tv_sec * 1000000000u + (uint64_t)now.tv_nsec;
    }
    /* Every reading advances by the scripted step, so the pair of readings
     * around one host call measures exactly that step. */
    return atomic_fetch_add_explicit(
        &wf_scripted_clock_now_ns,
        step,
        memory_order_relaxed
    ) + step;
}

static void wf_script_clock(uint64_t step_ns) {
    atomic_store_explicit(
        &wf_scripted_clock_step_ns,
        step_ns,
        memory_order_relaxed
    );
}

/* Counts every application of the WF_IO_NOCACHE target policy and then makes
 * the same host call the shipped build makes.  The harness build names this
 * function as WF_FILE_UNCACHED_APPLY, so the policy under test is the one in
 * file_adapter.h and only the counting is added. */
static _Atomic unsigned wf_uncached_applications;

void wf_completion_test_uncached_apply(int descriptor) {
    atomic_fetch_add_explicit(
        &wf_uncached_applications,
        1u,
        memory_order_relaxed
    );
    wf_file_uncached_apply_host(descriptor);
}

static unsigned wf_uncached_applications_now(void) {
    return atomic_load_explicit(
        &wf_uncached_applications,
        memory_order_relaxed
    );
}

/* The open flags an open really carried, with the bits that are a property of
 * the host rather than of the request masked away.  Both runs of this test
 * assert the same values, which is what makes the pair a statement that the
 * policy adds nothing to and removes nothing from an open. */
static int wf_uncached_open_flags(int descriptor) {
    int flags = fcntl(descriptor, F_GETFL);
    if (flags < 0) {
        return -1;
    }
    return flags & (O_ACCMODE | O_NONBLOCK | O_APPEND
#if defined(O_DIRECT)
                    | O_DIRECT
#endif
    );
}

/* WF_IO_NOCACHE is target policy, not a language surface: it changes how long
 * a read takes and nothing else.
 *
 * The same test runs twice from the Makefile, once with the setting absent
 * and once with it written.  Absent, the policy makes no host call at all and
 * the count does not move; written, it is applied exactly once for each
 * descriptor an open hands back and never for one a kind check refused.  Both
 * runs assert the identical open flags, the identical typed outcomes, and the
 * identical bytes, so what separates them is a cache hint and nothing else. */
static int test_uncached_reads_are_target_policy_only(
    const char *scratch_directory
) {
    const char *setting = getenv("WF_IO_NOCACHE");
    int asked = setting != NULL && setting[0] == '1' && setting[1] == 0;
    wf_harness_record record;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_SUCCEEDED;
    unsigned char content[8] = {'n', 'o', 'c', 'a', 'c', 'h', 'e', '!'};
    unsigned char window[8];
    char regular[256];
    char missing[256];
    unsigned before;
    unsigned after;
    int descriptor;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            regular,
            sizeof(regular),
            "%s/wf-completion-nocache-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    CHECK(
        snprintf(
            missing,
            sizeof(missing),
            "%s/wf-completion-nocache-absent-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(regular);
    (void)unlink(missing);
    descriptor = open(regular, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, content, sizeof(content)) == (ssize_t)sizeof(content));
    CHECK(close(descriptor) == 0);

    before = wf_uncached_applications_now();

    /* One kind-checked open through the submitted path.  Its flags are the
     * request's O_RDONLY plus the O_NONBLOCK a kind check needs, and nothing
     * else; the bytes it reads are the bytes that were written. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf_uncached_open_flags((int)value) == (O_RDONLY | O_NONBLOCK));
    memset(window, 0, sizeof(window));
    CHECK(
        wf__completion_file_pread_submit(
            (int)value,
            window,
            (uint64_t)sizeof(window),
            0u,
            record.bytes
        ) == 1
    );
    {
        int64_t moved = -1;
        int read_error = -1;
        wf__completion_file_join(record.bytes, &moved, &read_error);
        CHECK(moved == (int64_t)sizeof(window));
        CHECK(read_error == 0);
        CHECK(memcmp(window, content, sizeof(content)) == 0);
    }
    CHECK(close((int)value) == 0);

    /* The same open through the direct path. */
    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        regular,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf_uncached_open_flags((int)value) == (O_RDONLY | O_NONBLOCK));
    memset(window, 0, sizeof(window));
    CHECK(
        wf__completion_file_pread_direct(
            (int)value,
            window,
            (uint64_t)sizeof(window),
            0
        ) == (int64_t)sizeof(window)
    );
    CHECK(memcmp(window, content, sizeof(content)) == 0);
    CHECK(close((int)value) == 0);

    /* An unchecked open carries exactly the request's own flags. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_ANY,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf_uncached_open_flags((int)value) == O_RDONLY);
    CHECK(close((int)value) == 0);

    /* A refused open hands back no descriptor, so the policy has nothing to
     * apply to it. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        missing,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value < 0);
    CHECK(open_outcome == WF_FILE_OPEN_FAILED);

    after = wf_uncached_applications_now();
    CHECK(after - before == (asked ? 3u : 0u));

    CHECK(unlink(regular) == 0);
    return 0;
}

static int test_process_wide_target_helper_budget(void) {
    const char *text = getenv("WF_IO_HELPERS");
    uint64_t held = wf__completion_target_helper_count();
    unsigned long ceiling = 8;
    if (text != NULL && *text != '\0') {
        char *end = NULL;
        unsigned long pinned;
        errno = 0;
        pinned = strtoul(text, &end, 10);
        if (errno == 0 && end != text && *end == '\0') {
            if (pinned > ceiling) {
                pinned = ceiling;
            }
            CHECK(held == pinned);
            return 0;
        }
    }
    CHECK(held <= ceiling);
    return 0;
}


/* Submits `count` positioned reads of one open descriptor, one at a time, and
 * waits for each to complete.
 *
 * The wait is on the record rather than on the queue emptying: a helper takes
 * a request out of the queue before it executes it, so an empty queue is not
 * an answer, and waiting for one is a race this thread loses whenever the pool
 * has grown.  This thread also offers to execute, so the case runs the same
 * either way. */
static int drive_reads_to_completion(
    wf_file_adapter *adapter,
    int descriptor,
    unsigned count
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned index;
    for (index = 0; index < count; ++index) {
        wf_completion_record record;
        unsigned char byte = 0;
        unsigned attempts;
        harness_record_init(&record, WF_FILE_PREAD);
        record.request.operation.pread.descriptor = descriptor;
        record.request.operation.pread.buffer = &byte;
        record.request.operation.pread.count = 1;
        record.request.operation.pread.offset = 0;
        CHECK(
            wf_file_adapter_submit(adapter, &record) == WF_FILE_TARGET_OWNS
        );
        for (attempts = 0; attempts < 5000; ++attempts) {
            if (harness_record_done(&record)) {
                break;
            }
            if (wf_file_adapter_progress(adapter, 1) == 0) {
                (void)nanosleep(&delay, NULL);
            }
        }
        CHECK(harness_record_done(&record));
        CHECK(record.result.error_code == 0);
    }
    return 0;
}

/* A program whose operations do not wait gets no helper, however much width it
 * states.
 *
 * This is the half of the growth rule batch 0096 added, and it is the half
 * that decides what the warm path costs: queue depth says a program stated
 * independent work, not that the work waits on anything, and a helper handed
 * an operation the host has already answered adds a queue crossing and two
 * thread wakes to nothing.  The clock is scripted so the case is about the
 * rule rather than about this machine's page cache. */
static int test_pool_stays_empty_when_operations_do_not_wait(
    const char *directory
) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    pthread_t helpers[4];
    char path[512];
    int descriptor;

    CHECK(directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-pool-no-wait-%ld",
            directory,
            (long)getpid()
        ) > 0
    );
    descriptor = open(path, O_RDWR | O_CREAT | O_TRUNC, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, "wf", 2) == 2);

    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, helpers, 4, 0) == 0);
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 4) == 0);

    /* Nothing measured yet is not evidence of anything, and neither policy
     * fires on it: the first operations take the completion path unchanged. */
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_UNMEASURED);
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 0);

    /* One microsecond a call: real work, and an order of magnitude under the
     * wait that would make a second thread worth its handoff. */
    wf_script_clock(1000u);
    CHECK(drive_reads_to_completion(&adapter, descriptor, 32) == 0);
    wf_script_clock(0);
    CHECK(wf_file_adapter_helper_count(&adapter) == 0);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_SHORT);
    /* With no helper, nothing queued and no wait measured, a transfer
     * submitted now would be executed by the submitting thread itself. */
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 1);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* How many requests the two growth cases queue behind a pool that cannot
 * empty it.
 *
 * Growth adds at most one helper per submission and only while the queue is
 * deeper than the pool, so a pool of `n` needs a submission numbered past
 * `2n` to reach `n + 1` in the worst interleaving, where every helper takes
 * one entry the instant it is created.  Twenty carries a pool of eight, which
 * is the largest ceiling either case below asks for. */
#define WF_POOL_GROWTH_DEPTH 20u

/* Drives the helper pool against a queue it cannot empty and answers how
 * large the pool became.
 *
 * `drive_reads_to_completion` waits for each request before submitting the
 * next, so its queue never holds more than one entry.  Growth is gated on
 * `queue_count > held`, which that shape satisfies only while the pool is
 * empty: the first helper it creates is also its last, however high the cap
 * is.  A case that asserts an upper bound above one therefore asserts nothing
 * under that driver -- the bound is not approached, let alone reached, and
 * deleting the code that enforces it changes no verdict.
 *
 * These requests are reads of an empty pipe instead, so a helper that takes
 * one blocks in the host call and never returns for another.  At most one
 * entry per helper ever leaves the queue, so after `i` submissions the queue
 * holds at least `i - held` entries and every submission past twice the
 * current pool size finds `queue_count > held` and adds a helper.  Growth
 * then stops only where the cap stops it, which is the question both cases
 * below ask.
 *
 * The first read is served by a byte written before it, because growth also
 * requires a measured wait and an adapter that has executed nothing has
 * measured none.  Its execution is the sampled one -- the interval starts at
 * the first -- and the caller performs it, because the pool is still empty.
 * The pipe is fed the rest of its bytes only after the pool count has been
 * read, so that count is the pool's final one. */
static int grow_pool_against_a_blocked_queue(
    wf_file_adapter *adapter,
    int read_descriptor,
    int write_descriptor,
    size_t *held
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    static wf_completion_record records[WF_POOL_GROWTH_DEPTH];
    unsigned char bytes[WF_POOL_GROWTH_DEPTH];
    char feed[WF_POOL_GROWTH_DEPTH];
    wf_completion_record primer;
    unsigned char primer_byte = 0;
    unsigned index;

    CHECK(write(write_descriptor, "w", 1) == 1);
    harness_record_init(&primer, WF_FILE_READ);
    primer.request.operation.read.descriptor = read_descriptor;
    primer.request.operation.read.buffer = &primer_byte;
    primer.request.operation.read.count = 1;
    /* Nothing is measured yet, so this submission cannot grow the pool
     * whatever its cap says, and the caller is the queue's only engine. */
    CHECK(wf_file_adapter_submit(adapter, &primer) == WF_FILE_TARGET_OWNS);
    CHECK(wf_file_adapter_helper_count(adapter) == 0);
    CHECK(wf_file_adapter_progress(adapter, 1) == 1);
    CHECK(harness_record_done(&primer));
    CHECK(primer.result.error_code == 0);
    CHECK(primer.result.value == 1);
    CHECK(primer_byte == 'w');
    /* One scripted millisecond a call is a wait by any reading of the
     * threshold, so the growth rule's measured half now holds. */
    CHECK(wf_file_adapter_wait_verdict(adapter) == WF_FILE_WAIT_LONG);

    /* The deep queue.  Nothing feeds the pipe from here, so every helper the
     * pool gains blocks on its first read and the queue only deepens. */
    for (index = 0; index < WF_POOL_GROWTH_DEPTH; ++index) {
        bytes[index] = 0;
        feed[index] = 'q';
        harness_record_init(&records[index], WF_FILE_READ);
        records[index].request.operation.read.descriptor = read_descriptor;
        records[index].request.operation.read.buffer = &bytes[index];
        records[index].request.operation.read.count = 1;
        CHECK(
            wf_file_adapter_submit(adapter, &records[index])
            == WF_FILE_TARGET_OWNS
        );
    }
    /* Admission has stopped and growth happens only on admission, so this is
     * the size the policy settled on. */
    *held = wf_file_adapter_helper_count(adapter);

    /* One byte for each submitted read: from here no read of this pipe can
     * block, so the pool -- whatever size it reached -- drains the queue, and
     * the caller helps in case the pool is empty. */
    CHECK(
        write(write_descriptor, feed, sizeof(feed))
        == (ssize_t)sizeof(feed)
    );
    for (index = 0; index < WF_POOL_GROWTH_DEPTH; ++index) {
        unsigned attempts;
        for (attempts = 0; attempts < 100000u; ++attempts) {
            if (harness_record_done(&records[index])) {
                break;
            }
            if (wf_file_adapter_progress(adapter, 1) == 0) {
                (void)nanosleep(&delay, NULL);
            }
        }
        CHECK(harness_record_done(&records[index]));
        CHECK(records[index].result.error_code == 0);
        CHECK(records[index].result.value == 1);
        CHECK(bytes[index] == 'q');
    }
    return 0;
}

/* A program whose operations do wait gets helpers, up to the cap and no
 * further.
 *
 * Both ends of that promise need a queue the pool cannot empty.  With one
 * request in flight at a time the pool stops itself at one helper, so a cap of
 * four is neither reached nor tested; against the blocked queue the pool
 * climbs until the cap is the only thing left stopping it, and the count it
 * settles on is the cap exactly. */
static int test_pool_grows_when_operations_wait(void) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    pthread_t helpers[4];
    int descriptors[2];
    size_t held = 0;

    CHECK(pipe(descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, helpers, 4, 0) == 0);
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 4) == 0);

    /* Nothing measured yet is not evidence of anything, and neither policy
     * fires on it. */
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_UNMEASURED);

    wf_script_clock(1000000u);
    CHECK(
        grow_pool_against_a_blocked_queue(
            &adapter,
            descriptors[0],
            descriptors[1],
            &held
        ) == 0
    );
    wf_script_clock(0);
    CHECK(held == 4);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_LONG);
    /* A transfer submitted now would be overlapped by a helper, so the
     * submitting thread must not take it itself. */
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 0);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

/* The ceiling a policy asks for is a wish; the storage the caller handed to
 * `wf_file_adapter_init` is the fact.  The growth rule writes `helpers[held]`,
 * so a cap above that storage is a `pthread_create` past its end -- which is
 * why `wf_file_adapter_set_helper_cap` clamps the cap to the storage rather
 * than trusting it.  A cap of eight against storage of two leaves the clamp as
 * the only thing keeping the pool inside the caller's array.
 *
 * The array below is deliberately longer than the two entries init is told
 * about.  The bound under test is the declared capacity, and the slack decides
 * only whether a violation of it is *reported* -- a pool larger than the
 * storage it was given -- or executed as a write past the end of this frame,
 * which ends the run before any check is reached.  With the clamp in place
 * nothing beyond `helpers[1]` is ever written, so the slack is unreachable in
 * a passing run. */
static int test_helper_growth_stops_at_the_helper_storage(void) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    pthread_t helpers[WF_POOL_GROWTH_DEPTH];
    int descriptors[2];
    size_t held = 0;

    CHECK(pipe(descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, helpers, 2, 0) == 0);
    /* Accepted, and silently reduced to the two helpers init was given. */
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 8) == 0);

    wf_script_clock(1000000u);
    CHECK(
        grow_pool_against_a_blocked_queue(
            &adapter,
            descriptors[0],
            descriptors[1],
            &held
        ) == 0
    );
    wf_script_clock(0);
    /* Two, not the eight that were asked for: the queue stayed deeper than
     * the pool for every one of the twenty submissions, so the cap is the
     * only thing that stopped it, and the cap is the storage. */
    CHECK(held == 2);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

/* An initial count the storage cannot hold is refused outright: unlike the
 * cap, it is not a ceiling to grow towards but threads to start right now. */
static int test_helper_count_above_its_storage_is_refused(void) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    pthread_t helpers[1];

    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, helpers, 1, 2) == EINVAL);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

/* A shut-down record answers "not usable" to every entry, and it answers that
 * before the lock behind the answer is destroyed.
 *
 * `wf_file_adapter_shutdown` destroys the condition variable and the mutex,
 * and every entry of this adapter reads the `initialized` flag and then
 * touches storage behind them.  The flag is therefore what stands between a
 * later caller and destroyed storage, which makes two things testable without
 * a race: after a shutdown every entry is refused at the guard, and a second
 * shutdown is refused rather than joining threads that are gone and
 * destroying a mutex twice.
 *
 * The pool is grown first on purpose.  A shutdown of an adapter that never
 * started a helper joins nothing, so the interesting teardown -- helpers
 * joined, then the objects they waited on destroyed -- would not run at all. */
static int test_shutdown_refuses_every_later_entry(void) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    pthread_t helpers[4];
    int descriptors[2];
    size_t held = 0;

    CHECK(pipe(descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, helpers, 4, 0) == 0);
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 4) == 0);
    wf_script_clock(1000000u);
    CHECK(
        grow_pool_against_a_blocked_queue(
            &adapter,
            descriptors[0],
            descriptors[1],
            &held
        ) == 0
    );
    wf_script_clock(0);
    /* The record is live, and says so, before the shutdown below. */
    CHECK(held == 4);
    CHECK(wf_file_adapter_helper_count(&adapter) == 4);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_LONG);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);

    /* Every entry that would take `queue_lock` or read the record's measured
     * state is turned away by the flag instead. */
    CHECK(wf_file_adapter_queued(&adapter) == 0);
    CHECK(wf_file_adapter_helper_count(&adapter) == 0);
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 0);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_UNMEASURED);
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 2) == EINVAL);
    /* And shutdown itself is one of those entries. */
    CHECK(wf_file_adapter_shutdown(&adapter) == EINVAL);

    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

typedef struct wf_pipe_writer {
    int descriptor;
    unsigned char byte;
} wf_pipe_writer;

static void *write_after_a_delay(void *opaque) {
    wf_pipe_writer *context = opaque;
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 20000000};
    ssize_t written;
    (void)nanosleep(&delay, NULL);
    written = write(context->descriptor, &context->byte, 1);
    (void)written;
    return NULL;
}

/* A completion elsewhere ends a join that is waiting in place.
 *
 * This is the property `test_drain_wakes_the_registered_token_owner` held over
 * the deleted consume-wait registration, in the form the design gives it: the
 * join registers `WF_SCHED_WAITER_IN_PLACE` on its own record, sleeps on the
 * one primitive, and the thread that finishes the operation -- a helper where
 * the pool has one, this thread's own progress pass where it has none -- calls
 * `wf_sched_complete`, which claims the registration and wakes it (design §2's
 * fourth line, §6).
 *
 * The operation is a read of an empty pipe, so it genuinely cannot finish
 * until another thread writes: a join that did not wait, or a completion that
 * did not wake, is a hang the harness watchdog reports by name rather than a
 * wrong answer.  It runs on all four helper settings the gate uses, which is
 * what covers both engines. */
static int test_a_helper_completion_wakes_a_waiting_join(void) {
    wf_harness_record record;
    wf_pipe_writer writer_context;
    pthread_t writer;
    int descriptors[2];
    unsigned char byte = 0;
    int64_t value = -1;
    int error_code = -1;
    uint64_t publications_before;

    CHECK(pipe(descriptors) == 0);
    publications_before = wf__completion_publications();
    CHECK(
        wf__completion_file_read_submit(
            descriptors[0],
            &byte,
            1,
            record.bytes
        ) == 1
    );
    writer_context.descriptor = descriptors[1];
    writer_context.byte = 'w';
    CHECK(
        pthread_create(&writer, NULL, write_after_a_delay, &writer_context)
        == 0
    );
    wf__completion_file_join(record.bytes, &value, &error_code);
    CHECK(pthread_join(writer, NULL) == 0);
    CHECK(value == 1 && error_code == 0);
    CHECK(byte == 'w');
    CHECK(wf__completion_publications() == publications_before + 1u);
    /* A non-positioned read is never executed inside submit, so this really
     * did go to the queue and come back from an engine. */
    CHECK(wf__completion_file_fallback_submissions() >= 1u);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

/* Interruption and readiness refusal are adapter progress, never an outcome.
 *
 * The read below is of a nonblocking empty pipe, so its first host attempt is
 * refused with EAGAIN; the adapter waits for exactly one POLLIN and repeats
 * it.  One submission still produces exactly one completion, which the
 * publication count says. */
static int test_readiness_refusal_is_not_a_terminal_outcome(void) {
    wf_completion_runtime runtime;
    wf_file_adapter adapter;
    wf_completion_record record;
    wf_pipe_writer writer_context;
    pthread_t writer;
    uint64_t publications_before;
    int descriptors[2];
    int flags;
    unsigned char byte = 0;

    CHECK(pipe(descriptors) == 0);
    flags = fcntl(descriptors[0], F_GETFL, 0);
    CHECK(flags >= 0);
    CHECK(fcntl(descriptors[0], F_SETFL, flags | O_NONBLOCK) == 0);
    CHECK(wf_completion_runtime_init(&runtime) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, NULL, 0, 0) == 0);
    harness_record_init(&record, WF_FILE_READ);
    record.request.operation.read.descriptor = descriptors[0];
    record.request.operation.read.buffer = &byte;
    record.request.operation.read.count = 1;
    publications_before = wf__completion_publications();
    CHECK(wf_file_adapter_submit(&adapter, &record) == WF_FILE_TARGET_OWNS);
    writer_context.descriptor = descriptors[1];
    writer_context.byte = 'r';
    CHECK(
        pthread_create(&writer, NULL, write_after_a_delay, &writer_context)
        == 0
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(pthread_join(writer, NULL) == 0);
    CHECK(byte == 'r');
    CHECK(harness_record_done(&record));
    CHECK(record.result.value == 1);
    CHECK(record.result.error_code == 0);
    CHECK(wf__completion_publications() == publications_before + 1u);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}
/* The runtime's own bound on how many iterations of one loop may be in flight
 * at once.
 *
 * The rules this checks are the only ones a lowering may rely on: every
 * argument is a bound and none can raise the answer, a zero argument places no
 * bound, and one is always a legal answer.  That last one is what makes the
 * query safe to emit at all — a loop whose window came back as zero would have
 * no legal schedule, and the sequential program is always a legal schedule. */
static int test_completion_window_answers_at_the_boundaries(void) {
    uint64_t unconstrained = wf__completion_window(0, 0, 0);
    uint64_t budget = WF_HARNESS_WINDOW_BYTE_BUDGET;
    uint64_t two;

    /* The runtime's unconstrained answer is its own throughput choice.  It
     * used to be half the process operation capacity, because a loop holding
     * every record would have pushed the rest of the program onto the
     * capacity-wait path; there is no capacity and no wait to be pushed onto
     * any more, and the number is unchanged (design §7). */
    CHECK(unconstrained >= 1u);
    CHECK(unconstrained <= WF_HARNESS_WINDOW_DEFAULT);

    /* A trip count of one is a loop with nothing to overlap. */
    CHECK(wf__completion_window(1, 0, 0) == 1u);
    /* The compiler's own static cap is honoured exactly. */
    CHECK(wf__completion_window(0, 0, 1) == 1u);
    CHECK(wf__completion_window(0, 0, 2) == (unconstrained < 2u ? unconstrained : 2u));
    CHECK(wf__completion_window(3, 0, 0) == (unconstrained < 3u ? unconstrained : 3u));
    /* Neither a huge trip count nor a huge ceiling raises the runtime's own
     * answer: every argument is a minimum with it, never a request. */
    CHECK(wf__completion_window(UINT64_MAX, 0, 0) == unconstrained);
    CHECK(wf__completion_window(0, 0, UINT64_MAX) == unconstrained);
    CHECK(wf__completion_window(UINT64_MAX, 0, UINT64_MAX) == unconstrained);

    /* The byte budget. One slot of the whole budget affords one iteration;
     * half of it affords two; a slot larger than the budget affords none, and
     * the floor turns that into the sequential program rather than into a
     * schedule with no slots. */
    CHECK(wf__completion_window(0, budget, 0) == 1u);
    CHECK(wf__completion_window(0, budget + 1u, 0) == 1u);
    CHECK(wf__completion_window(0, UINT64_MAX, 0) == 1u);
    /* The design's own example: a loop privatizing a 16 MiB buffer gets one. */
    CHECK(wf__completion_window(0, 16u * 1024u * 1024u, 0) == 1u);
    two = budget / 2u;
    CHECK(wf__completion_window(0, two, 0) == (unconstrained < 2u ? unconstrained : 2u));

    /* The smallest argument decides, whichever one it is. */
    CHECK(wf__completion_window(2, budget / 8u, 8) == 2u);
    CHECK(wf__completion_window(8, budget / 2u, 8) == (unconstrained < 2u ? unconstrained : 2u));
    CHECK(wf__completion_window(8, budget / 8u, 2) == 2u);

    /* One is always legal: no combination of arguments answers zero. */
    CHECK(wf__completion_window(0, 0, 0) >= 1u);
    CHECK(wf__completion_window(1, UINT64_MAX, 1) == 1u);
    CHECK(wf__completion_window(UINT64_MAX, UINT64_MAX, UINT64_MAX) == 1u);
    return 0;
}

/* The io_uring doorbell is deferred, and every path that can stop reaches the
 * kernel before it does.
 *
 * Staging a submission-queue entry is a store to a page the kernel already
 * maps, so it costs no system call; `io_uring_enter` is what tells the kernel
 * the entry exists. Ringing it once per submission is 15,360 calls on the
 * eight-wide benchmark and 2,048 when it is deferred (LOOP-PIPELINE.md §9.1).
 * Deferring is only safe while the two facts below hold, and they are what
 * this checks: a submission alone rings nothing, and the first thing that
 * could wait — a join, or a blocking direct host call — rings it first.
 *
 * The enter counter only exists where the ring is the target route, so the
 * counted half of this test runs there and the functional half runs
 * everywhere. */
static int test_a_submitted_operation_is_kicked_before_it_waits(
    const char *scratch_directory
) {
    char path[256];
    unsigned char first[8];
    unsigned char second[8];
    wf_harness_record open_record;
    wf_harness_record read_record;
    wf_harness_record other_record;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    int descriptor;
    int writer;
    uint64_t submissions_before;
    uint64_t enters;
    int ring_route;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-doorbell-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(write(writer, "doorbell", 8) == 8);
    CHECK(close(writer) == 0);

    submissions_before = wf__completion_linux_io_uring_submissions();
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            open_record.bytes
        ) == 1
    );
    wf__completion_file_open_join(
        open_record.bytes,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0 && error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    descriptor = (int)value;
    ring_route =
        wf__completion_linux_io_uring_submissions() > submissions_before;

    /* A submission alone rings nothing. */
    enters = wf__completion_linux_io_uring_submission_enters();
    memset(first, 0, sizeof(first));
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            sizeof(first),
            0,
            read_record.bytes
        ) == 1
    );
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters);
    }

    /* A blocking direct host call rings it first. This one is refused
     * immediately by the host, so what it proves is the ordering and not the
     * waiting: the doorbell rang before the call, not after it. */
    CHECK(wf__completion_file_close_direct(-1) < 0);
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters + 1u);
    }
    wf__completion_file_join(read_record.bytes, &value, &error_code);
    CHECK(value == 8 && error_code == 0);
    CHECK(memcmp(first, "doorbell", 8) == 0);

    /* Two submissions, one doorbell: the first join carries both, and the
     * second join needs no further call. This is the whole of what deferring
     * buys, stated as a count. */
    enters = wf__completion_linux_io_uring_submission_enters();
    memset(first, 0, sizeof(first));
    memset(second, 0, sizeof(second));
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            4,
            0,
            read_record.bytes
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            second,
            4,
            4,
            other_record.bytes
        ) == 1
    );
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters);
    }
    wf__completion_file_join(read_record.bytes, &value, &error_code);
    CHECK(value == 4 && error_code == 0);
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters + 1u);
    }
    wf__completion_file_join(other_record.bytes, &value, &error_code);
    CHECK(value == 4 && error_code == 0);
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters + 1u);
    }
    CHECK(memcmp(first, "door", 4) == 0);
    CHECK(memcmp(second, "bell", 4) == 0);

    CHECK(wf__completion_file_close_submit(descriptor, read_record.bytes) == 1);
    wf__completion_file_join(read_record.bytes, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_native_contract_inventory(void) {
    wf_completion_target_contract darwin = wf_completion_target_contract_for(
        WF_TARGET_DARWIN_FILE_FALLBACK
    );
    wf_completion_target_contract linux = wf_completion_target_contract_for(
        WF_TARGET_LINUX_IO_URING
    );
    wf_completion_target_contract windows = wf_completion_target_contract_for(
        WF_TARGET_WINDOWS_IOCP
    );
    CHECK(darwin.native_completion == 0);
    CHECK(darwin.may_use_blocking_helpers == 1);
    CHECK(darwin.supports_scheduler_progress == 1);
#if defined(__APPLE__)
    CHECK(darwin.implemented == 1);
#else
    CHECK(darwin.implemented == 0);
#endif
#if defined(__linux__)
    CHECK(linux.implemented == 1);
#else
    CHECK(linux.implemented == 0);
#endif
    CHECK(linux.native_completion == 1);
#if defined(_WIN32)
    CHECK(windows.implemented == 1);
#else
    CHECK(windows.implemented == 0);
#endif
    CHECK(windows.native_completion == 1);
    return 0;
}


#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
/* The base-position cell after one progressing attempt: the value the Darwin
 * stub above writes, and the caller's own sentinel on a family whose facility
 * takes no such cell. */
static int64_t wf_expected_position(void) {
#if defined(__APPLE__)
    return 17;
#else
    return 4242;
#endif
}
#endif

/* One admitted enumeration is one progress-producing host attempt, whichever
 * family supplies the facility: interruption repeats the attempt, readiness
 * refusal waits for exactly one POLLIN on the enumerated descriptor and
 * repeats it, and only progress crosses the ABI. The Darwin facility's
 * base-position cell is written on the progressing attempt; Linux's facility
 * has none, and this case asserts the cell is left as the caller gave it. */
static int test_directory_progress_is_internal(void) {
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    int descriptors[2];
    wf_harness_record record;
    unsigned char readiness = 'r';
    unsigned char byte = 0;
    int64_t position = 4242;
    int64_t value = -1;
    int error_code = -1;
    wf_directory_host_calls = 0;
    wf_directory_poll_calls = 0;
    wf_directory_poll_descriptor = -1;
    wf_directory_poll_events = 0;
    wf_directory_poll_timeout = 0;
    CHECK(pipe(descriptors) == 0);
    CHECK(write(descriptors[1], &readiness, 1) == 1);
    CHECK(
        wf__completion_directory_next_direct(
            descriptors[0],
            &byte,
            1,
            &position
        ) == 1
    );
    CHECK(wf_directory_host_calls == 3);
    CHECK(wf_directory_poll_calls == 1);
    CHECK(wf_directory_poll_descriptor == descriptors[0]);
    CHECK(wf_directory_poll_events == POLLIN);
    CHECK(wf_directory_poll_timeout == -1);
    CHECK(byte == 'd');
    CHECK(position == wf_expected_position());

    wf_directory_host_calls = 0;
    wf_directory_poll_calls = 0;
    byte = 0;
    position = 4242;
    CHECK(
        wf__completion_directory_next_submit(
            descriptors[0],
            &byte,
            1,
            &position,
            record.bytes
        ) == 1
    );
    wf__completion_file_join(record.bytes, &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(wf_directory_host_calls == 3);
    CHECK(wf_directory_poll_calls == 1);
    CHECK(byte == 'd');
    CHECK(position == wf_expected_position());
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
#endif
    return 0;
}


static uint64_t monotonic_nanoseconds(void) {
    struct timespec now;
    CHECK(clock_gettime(CLOCK_MONOTONIC, &now) == 0);
    return (uint64_t)now.tv_sec * UINT64_C(1000000000)
        + (uint64_t)now.tv_nsec;
}

/* What one submission-to-join round trip costs when the operation itself
 * costs nothing.
 *
 * It is the record protocol and nothing else: initialise the record, store a
 * result head, publish through `wf_sched_complete`, read DONE.  The claim,
 * the drain and the consume it used to include are deleted with the pool. */
static int benchmark_record_roundtrip(uint64_t *nanoseconds_per_operation) {
    enum { ITERATIONS = 100000 };
    wf_completion_record record;
    uint64_t start;
    uint64_t elapsed;
    int iteration;

    start = monotonic_nanoseconds();
    for (iteration = 0; iteration < ITERATIONS; ++iteration) {
        harness_record_init(&record, WF_FILE_PREAD);
        record.result.kind = WF_FILE_PREAD;
        record.result.value = iteration;
        wf_completion_record_complete(&record);
        CHECK(harness_record_done(&record));
        CHECK(record.result.value == iteration);
    }
    elapsed = monotonic_nanoseconds() - start;
    *nanoseconds_per_operation = elapsed / ITERATIONS;
    return 0;
}

/* The name of the test now running, for the watchdog below.  Written by the
 * main thread and read by a signal handler, which is why it is a plain
 * pointer to a string literal: the store is a single aligned word and the
 * handler only prints it. */
static const char *volatile wf_harness_running = "startup";

/* A deadlock is a failure mode of this suite, so it gets a name.
 *
 * A join that waits in place makes threads wait for each other on purpose,
 * and a mistake in the wake does not produce a wrong answer — it produces no
 * answer, which without this would surface as a build job that never ends and
 * a log with nothing in it. The bound is three orders of magnitude above what
 * the whole suite takes, so it can only fire on a schedule that has genuinely
 * stopped. */
static void wf_harness_watchdog(int signal_number) {
    static const char message[] = "completion harness: no progress in test: ";
    ssize_t ignored;
    (void)signal_number;
    ignored = write(2, message, sizeof(message) - 1u);
    ignored = write(2, wf_harness_running, strlen(wf_harness_running));
    ignored = write(2, "\n", 1);
    (void)ignored;
    _exit(9);
}

int main(int argc, char **argv) {
    uint64_t roundtrip_ns = 0;
    int trace = getenv("WF_COMPLETION_TRACE") != NULL;
    struct sigaction watchdog;
    memset(&watchdog, 0, sizeof(watchdog));
    watchdog.sa_handler = wf_harness_watchdog;
    if (sigaction(SIGALRM, &watchdog, NULL) != 0) {
        fprintf(stderr, "completion harness: no watchdog\n");
        return 2;
    }
    (void)alarm(300u);
#define RUN_TEST(...)                                                         \
    do {                                                                      \
        wf_harness_running = #__VA_ARGS__;                                    \
        if (trace) {                                                          \
            fprintf(stderr, "completion harness: begin %s\n", #__VA_ARGS__); \
        }                                                                     \
        if ((__VA_ARGS__) != 0) {                                             \
            return 1;                                                         \
        }                                                                     \
        if (trace) {                                                          \
            fprintf(stderr, "completion harness: end %s\n", #__VA_ARGS__);   \
        }                                                                     \
    } while (0)
    if (argc != 2) {
        fprintf(stderr, "usage: %s SCRATCH_DIRECTORY\n", argv[0]);
        return 2;
    }
    /* The bridge cases below assert which route an operation took, and under
     * the demand-driven policy that is the runtime's choice: it runs a
     * positioned read inside submit once it has measured this host's reads as
     * not waiting, so whether a case sees the queued route would depend on
     * what the cases before it happened to execute. A written WF_IO_HELPERS
     * pins the route with the count, so an invocation that did not name one
     * gets one here and every case asserts a route that is fixed.
     *
     * What that leaves untested here is the policy itself, and it is tested
     * where it can be decided rather than sampled:
     * `test_pool_stays_empty_when_operations_do_not_wait` and
     * `test_pool_grows_when_operations_wait` script the clock the policy
     * measures with, `bridge_default_probe.c` runs the unset policy end to
     * end, and `compiler/tests/programs` runs whole compiled programs under
     * it. */
    if (getenv("WF_IO_HELPERS") == NULL) {
        (void)setenv("WF_IO_HELPERS", "1", 0);
    }
    RUN_TEST(test_exactly_one_completion_per_submission_under_race(argv[1]));
    RUN_TEST(test_a_completion_claims_an_in_place_registration());
    RUN_TEST(test_unified_wake_epoch());
    RUN_TEST(test_one_epoch_wakes_every_announced_thread());
    RUN_TEST(test_linux_independent_operations_use_available_target(argv[1]));
    RUN_TEST(test_single_thread_file_progress(argv[1]));
    RUN_TEST(test_bridge_independent_positioned_reads(argv[1]));
    RUN_TEST(test_bridge_open_status_and_close_are_typed_operations(argv[1]));
    RUN_TEST(test_submitted_open_resolves_the_submitters_bytes(argv[1]));
    RUN_TEST(
        test_a_name_no_pool_record_could_hold_takes_the_completion_path(argv[1])
    );
    RUN_TEST(test_more_operations_outstanding_than_the_old_capacity(argv[1]));
    RUN_TEST(test_checked_open_rejects_and_closes_nonregular_descriptors(argv[1]));
    RUN_TEST(test_open_failure_classes_are_typed_outcomes(argv[1]));
    RUN_TEST(test_open_results_reach_every_independent_owner(argv[1]));
    RUN_TEST(test_uncached_reads_are_target_policy_only(argv[1]));
    RUN_TEST(test_process_wide_target_helper_budget());
    RUN_TEST(test_pool_stays_empty_when_operations_do_not_wait(argv[1]));
    RUN_TEST(test_pool_grows_when_operations_wait());
    RUN_TEST(test_helper_growth_stops_at_the_helper_storage());
    RUN_TEST(test_helper_count_above_its_storage_is_refused());
    RUN_TEST(test_shutdown_refuses_every_later_entry());
    RUN_TEST(test_a_helper_completion_wakes_a_waiting_join());
    RUN_TEST(test_readiness_refusal_is_not_a_terminal_outcome());
    RUN_TEST(test_completion_window_answers_at_the_boundaries());
    RUN_TEST(test_a_submitted_operation_is_kicked_before_it_waits(argv[1]));
    RUN_TEST(test_native_contract_inventory());
    RUN_TEST(test_directory_progress_is_internal());
    RUN_TEST(benchmark_record_roundtrip(&roundtrip_ns));
#undef RUN_TEST
    printf(
        "completion-core-harness: PASS core_roundtrip_ns=%" PRIu64
        " racers=%d\n",
        roundtrip_ns,
        WF_HARNESS_RACERS
    );
    return 0;
}
