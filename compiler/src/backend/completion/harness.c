#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
#include "native_contract.h"

#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <pthread.h>
#include <poll.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

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

#if defined(__APPLE__)
static unsigned wf_directory_host_calls;
static unsigned wf_directory_poll_calls;
static int wf_directory_poll_descriptor;
static short wf_directory_poll_events;
static int wf_directory_poll_timeout;

/* Deterministic Darwin target answers: interruption, readiness refusal, then
 * one successful byte. The pipe descriptor used by the test is already
 * readable when the adapter performs its exact POLLIN wait. */
ssize_t wf_completion_test_getdirentries64(
    int descriptor,
    void *buffer,
    size_t count,
    int64_t *position
) {
    (void)descriptor;
    wf_directory_host_calls += 1;
    if (wf_directory_host_calls == 1) {
        errno = EINTR;
        return -1;
    }
    if (wf_directory_host_calls == 2) {
        errno = EAGAIN;
        return -1;
    }
    if (buffer == NULL || count == 0 || position == NULL) {
        errno = EINVAL;
        return -1;
    }
    ((unsigned char *)buffer)[0] = 'd';
    *position = 17;
    return 1;
}
#endif

int wf_completion_test_poll(
    struct pollfd *descriptors,
    nfds_t count,
    int timeout
) {
#if defined(__APPLE__)
    if (wf_directory_host_calls != 0) {
        wf_directory_poll_calls += 1;
        wf_directory_poll_descriptor = count == 1 ? descriptors[0].fd : -1;
        wf_directory_poll_events = count == 1 ? descriptors[0].events : 0;
        wf_directory_poll_timeout = timeout;
    }
#endif
    return poll(descriptors, count, timeout);
}

static wf_completion_publication integer_publication(const int *value) {
    wf_completion_publication publication = {
        .milestones = WF_COMPLETION_OWNERSHIP_COMPLETE,
        .terminal_kind = 1,
        .result = value,
        .result_size = sizeof(*value),
    };
    return publication;
}

static int accept_operation(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    CHECK(
        wf_completion_begin_submit(runtime, token)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(
        wf_completion_target_accepted(runtime, token)
        == WF_COMPLETION_TRANSITIONED
    );
    return 0;
}

static int drain_exact(
    wf_completion_runtime *runtime,
    size_t expected,
    wf_completion_event *events
) {
    size_t total = 0;
    size_t rounds = 0;
    while (total < expected && rounds < runtime->slot_count + 2) {
        total += wf_completion_drain(
            runtime,
            events + total,
            expected - total,
            runtime->slot_count
        );
        rounds += 1;
    }
    CHECK(total == expected);
    return 0;
}

static int consume_integer(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    int expected
) {
    int result = 0;
    wf_completion_outcome outcome;
    CHECK(
        wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result == expected);
    CHECK(
        (outcome.milestones & WF_COMPLETION_OWNERSHIP_COMPLETE)
        == WF_COMPLETION_OWNERSHIP_COMPLETE
    );
    CHECK(outcome.terminal_kind == 1);
    CHECK(outcome.result_size == sizeof(result));
    return 0;
}

static int test_capacity_and_product_state(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_completion_token first;
    wf_completion_token second;
    wf_completion_token refused;
    wf_completion_event event;
    wf_completion_outcome outcome;
    wf_completion_publication publication;
    uint32_t milestones = 0;
    unsigned phase = 0;
    int value = 37;
    int result = 0;

    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(wf_completion_claim(&runtime, &first) == WF_COMPLETION_CLAIMED);
    CHECK(wf_completion_claim(&runtime, &second) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_completion_claim(&runtime, &refused)
        == WF_COMPLETION_CLAIM_WAIT_CAPACITY
    );

    CHECK(
        wf_completion_begin_submit(&runtime, first)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(
        wf_completion_mark_wait_capacity(&runtime, first)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(
        wf_completion_observe(&runtime, first, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(phase == WF_COMPLETION_WAIT_CAPACITY);
    CHECK(milestones == 0);
    CHECK(accept_operation(&runtime, first) == 0);

    publication = integer_publication(&value);
    publication.milestones = WF_COMPLETION_RESULT_READY
        | WF_COMPLETION_TERMINAL;
    CHECK(
        wf_completion_publish_terminal(&runtime, first, &publication)
        == WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL
    );
    publication = integer_publication(&value);
    publication.result_size = WF_COMPLETION_RESULT_CAPACITY + 1u;
    CHECK(
        wf_completion_publish_terminal(&runtime, first, &publication)
        == WF_COMPLETION_PUBLISH_RESULT_TOO_LARGE
    );
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, first, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(
        wf_completion_observe(&runtime, first, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(phase == WF_COMPLETION_TERMINAL_PHASE);
    CHECK((milestones & WF_COMPLETION_RESULT_READY) != 0);
    CHECK((milestones & WF_COMPLETION_PAYLOAD_RELEASED) != 0);
    CHECK((milestones & WF_COMPLETION_AUTHORITY_RELEASED) != 0);
    CHECK((milestones & WF_COMPLETION_TERMINAL) != 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            first,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUME_NOT_DRAINED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(event.token.slot == first.slot);
    CHECK(event.token.generation == first.generation);
    CHECK(consume_integer(&runtime, first, value) == 0);

    /* Release the second slot through the true inline/no-future-event route. */
    CHECK(
        wf_completion_begin_submit(&runtime, second)
        == WF_COMPLETION_TRANSITIONED
    );
    value = 41;
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_inline_terminal(&runtime, second, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, second, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_generation_and_duplicate_terminal(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token old_token;
    wf_completion_token new_token;
    wf_completion_event event;
    wf_completion_publication publication;
    int first = 111;
    int malicious = 999;
    int second = 222;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &old_token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, old_token) == 0);
    publication = integer_publication(&first);
    CHECK(
        wf_completion_publish_terminal(&runtime, old_token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    publication = integer_publication(&malicious);
    CHECK(
        wf_completion_publish_terminal(&runtime, old_token, &publication)
        == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, old_token, first) == 0);

    CHECK(wf_completion_claim(&runtime, &new_token) == WF_COMPLETION_CLAIMED);
    CHECK(new_token.slot == old_token.slot);
    CHECK(new_token.generation == old_token.generation + 1);
    CHECK(accept_operation(&runtime, new_token) == 0);

    /* The stale token is rejected while holding the slot lock and before the
     * malicious result can touch the new logical operation's result cell. */
    publication = integer_publication(&malicious);
    CHECK(
        wf_completion_publish_terminal(&runtime, old_token, &publication)
        == WF_COMPLETION_PUBLISH_STALE
    );
    publication = integer_publication(&second);
    CHECK(
        wf_completion_publish_terminal(&runtime, new_token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, new_token, second) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

#define TERMINAL_RACERS 12

typedef struct terminal_race_context {
    wf_completion_runtime *runtime;
    wf_completion_token token;
    _Atomic unsigned *start;
    int value;
    enum wf_completion_publish_result result;
} terminal_race_context;

static void *terminal_racer(void *opaque) {
    terminal_race_context *context = opaque;
    wf_completion_publication publication;
    while (atomic_load_explicit(context->start, memory_order_acquire) == 0) {
        sched_yield();
    }
    publication = integer_publication(&context->value);
    context->result = wf_completion_publish_terminal(
        context->runtime,
        context->token,
        &publication
    );
    return NULL;
}

static int test_exactly_one_terminal_under_race(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_event event;
    pthread_t threads[TERMINAL_RACERS];
    terminal_race_context contexts[TERMINAL_RACERS];
    _Atomic unsigned start;
    size_t index;
    size_t winners = 0;
    size_t duplicates = 0;
    int winning_value = 0;

    atomic_init(&start, 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    for (index = 0; index < TERMINAL_RACERS; ++index) {
        contexts[index].runtime = &runtime;
        contexts[index].token = token;
        contexts[index].start = &start;
        contexts[index].value = (int)index + 1;
        contexts[index].result = WF_COMPLETION_PUBLISH_INVALID_STATE;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                terminal_racer,
                &contexts[index]
            ) == 0
        );
    }
    atomic_store_explicit(&start, 1, memory_order_release);
    for (index = 0; index < TERMINAL_RACERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        if (contexts[index].result == WF_COMPLETION_PUBLISHED) {
            winners += 1;
            winning_value = contexts[index].value;
        } else if (
            contexts[index].result
            == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL
        ) {
            duplicates += 1;
        }
    }
    CHECK(winners == 1);
    CHECK(duplicates == TERMINAL_RACERS - 1);
    CHECK(wf_completion_ready_event_count(&runtime) == 1);
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, token, winning_value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

typedef struct park_context {
    wf_completion_runtime *runtime;
    uint64_t epoch;
    enum wf_completion_park_result result;
} park_context;

static void *park_scheduler(void *opaque) {
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

typedef struct claim_context {
    wf_completion_runtime *runtime;
    wf_completion_token token;
    enum wf_completion_claim_result result;
} claim_context;

static void *claim_on_other_lane(void *opaque) {
    claim_context *context = opaque;
    context->result = wf_completion_claim(context->runtime, &context->token);
    return NULL;
}

typedef struct batch_claim_context {
    wf_completion_runtime *runtime;
    wf_completion_token tokens[2];
    enum wf_completion_claim_result result;
} batch_claim_context;

static void *claim_batch_on_other_lane(void *opaque) {
    batch_claim_context *context = opaque;
    context->result = wf_completion_claim_many(
        context->runtime,
        context->tokens,
        2
    );
    return NULL;
}

static int wait_until_atomic_mask(
    const _Atomic unsigned *value,
    unsigned mask
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;
    for (attempts = 0; attempts < 5000; ++attempts) {
        if ((atomic_load_explicit(value, memory_order_acquire) & mask) != 0) {
            return 0;
        }
        (void)nanosleep(&delay, NULL);
    }
    return 1;
}

static int test_batch_admission_wake_is_not_capacity(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_completion_token retry;
    wf_completion_event events[4];
    wf_completion_publication publication;
    wf_completion_statistics before;
    wf_completion_statistics after;
    claim_context single;
    batch_claim_context batch;
    park_context parked;
    pthread_t single_thread;
    pthread_t batch_thread;
    pthread_t park_thread;
    _Atomic unsigned host_wakes;
    uint64_t epoch_before;
    int values[4] = {11, 12, 13, 14};
    size_t index;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            record_host_wake,
            &host_wakes
        ) == 0
    );

    /* Pin a real single claim inside its slot critical section. The batch can
     * then close its gate but cannot reserve until that claimant leaves. */
    CHECK(pthread_mutex_lock(&slots[0].publication_lock) == 0);
    single.runtime = &runtime;
    single.result = WF_COMPLETION_CLAIM_INVALID;
    CHECK(
        pthread_create(
            &single_thread,
            NULL,
            claim_on_other_lane,
            &single
        ) == 0
    );
    CHECK(wait_until_atomic_mask(&runtime.single_claimers, 1u) == 0);

    batch.runtime = &runtime;
    batch.result = WF_COMPLETION_CLAIM_INVALID;
    CHECK(
        pthread_create(
            &batch_thread,
            NULL,
            claim_batch_on_other_lane,
            &batch
        ) == 0
    );
    CHECK(wait_until_atomic_mask(&runtime.batch_claiming, 1u) == 0);

    before = wf_completion_statistics_snapshot(&runtime);
    epoch_before = wf_completion_wake_epoch(&runtime);
    CHECK(
        wf_completion_claim(&runtime, &retry)
        == WF_COMPLETION_CLAIM_WAIT_ADMISSION
    );
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.claim_admission_waits == before.claim_admission_waits + 1);
    CHECK(after.claim_capacity_waits == before.claim_capacity_waits);

    parked.runtime = &runtime;
    parked.epoch = epoch_before;
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&park_thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);

    CHECK(pthread_mutex_unlock(&slots[0].publication_lock) == 0);
    CHECK(pthread_join(single_thread, NULL) == 0);
    CHECK(pthread_join(batch_thread, NULL) == 0);
    CHECK(pthread_join(park_thread, NULL) == 0);
    CHECK(single.result == WF_COMPLETION_CLAIMED);
    CHECK(batch.result == WF_COMPLETION_CLAIMED);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);

    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.claims == before.claims + 3);
    CHECK(after.claim_admission_waits == before.claim_admission_waits + 1);
    CHECK(after.claim_capacity_waits == before.claim_capacity_waits);
    CHECK(after.admission_notifications == before.admission_notifications + 1);
    CHECK(after.capacity_notifications == before.capacity_notifications);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 1);
    CHECK(wf_completion_claim(&runtime, &retry) == WF_COMPLETION_CLAIMED);

    CHECK(accept_operation(&runtime, single.token) == 0);
    publication = integer_publication(&values[0]);
    CHECK(
        wf_completion_publish_terminal(&runtime, single.token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    for (index = 0; index < 2; ++index) {
        CHECK(accept_operation(&runtime, batch.tokens[index]) == 0);
        publication = integer_publication(&values[index + 1]);
        CHECK(
            wf_completion_publish_terminal(
                &runtime,
                batch.tokens[index],
                &publication
            ) == WF_COMPLETION_PUBLISHED
        );
    }
    CHECK(accept_operation(&runtime, retry) == 0);
    publication = integer_publication(&values[3]);
    CHECK(
        wf_completion_publish_terminal(&runtime, retry, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(drain_exact(&runtime, 4, events) == 0);
    CHECK(consume_integer(&runtime, single.token, values[0]) == 0);
    for (index = 0; index < 2; ++index) {
        CHECK(
            consume_integer(
                &runtime,
                batch.tokens[index],
                values[index + 1]
            ) == 0
        );
    }
    CHECK(consume_integer(&runtime, retry, values[3]) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.capacity_notifications == before.capacity_notifications + 4);
    CHECK(after.admission_notifications == before.admission_notifications + 1);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_failed_batch_does_not_invent_capacity(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_completion_token occupied;
    wf_completion_token refused[2];
    wf_completion_event event;
    wf_completion_publication publication;
    wf_completion_statistics before;
    wf_completion_statistics after;
    uint64_t epoch_before;
    int value = 29;

    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(wf_completion_claim(&runtime, &occupied) == WF_COMPLETION_CLAIMED);
    before = wf_completion_statistics_snapshot(&runtime);
    epoch_before = wf_completion_wake_epoch(&runtime);
    CHECK(
        wf_completion_claim_many(&runtime, refused, 2)
        == WF_COMPLETION_CLAIM_WAIT_CAPACITY
    );
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.claims == before.claims);
    CHECK(after.claim_capacity_waits == before.claim_capacity_waits + 1);
    CHECK(after.claim_admission_waits == before.claim_admission_waits);
    CHECK(after.capacity_notifications == before.capacity_notifications);
    CHECK(after.admission_notifications == before.admission_notifications);
    CHECK(wf_completion_wake_epoch(&runtime) == epoch_before);

    CHECK(accept_operation(&runtime, occupied) == 0);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, occupied, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, occupied, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_unified_wake_epoch(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_event event;
    wf_completion_publication publication;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked;
    pthread_t thread;
    _Atomic unsigned host_wakes;
    uint64_t epoch_before;
    int value = 73;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            record_host_wake,
            &host_wakes
        ) == 0
    );
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    epoch_before = wf_completion_wake_epoch(&runtime);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.wake_signals == before.wake_signals);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 0);
    CHECK(wf_completion_wake_epoch(&runtime) == epoch_before + 1);
    CHECK(
        wf_completion_park_if_unchanged(&runtime, epoch_before, 1)
        == WF_COMPLETION_PARK_EPOCH_CHANGED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, token, value) == 0);

    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    wf_completion_notify_compute(&runtime);
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(after.compute_notifications == before.compute_notifications + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 1);

    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 2);
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, token, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_linux_native_free_batch_avoids_fallback_thread(
    const char *scratch_directory
) {
#if defined(__linux__)
    wf_completion_token tokens[2];
    unsigned char bytes[2] = {0};
    int64_t value;
    int error_code;
    uint64_t native_before;
    uint64_t fallback_before;
    char path[256];
    int descriptor;
    int claimed;

    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-native-batch-%ld",
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
    claimed = wf__completion_file_batch_claim(tokens, 2, 0);
    if (claimed == 0) {
        CHECK(getenv("WF_REQUIRE_LINUX_IO_URING") == NULL);
        CHECK(close(descriptor) == 0);
        CHECK(unlink(path) == 0);
        return 0;
    }
    CHECK(wf__completion_target_helper_count() == 0);
    wf__completion_file_pread_submit_reserved(
        descriptor,
        &bytes[0],
        1,
        0,
        &tokens[0]
    );
    wf__completion_file_pread_submit_reserved(
        descriptor,
        &bytes[1],
        1,
        1,
        &tokens[1]
    );
    wf__completion_file_join(&tokens[0], &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    wf__completion_file_join(&tokens[1], &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(bytes[0] == 'x' && bytes[1] == 'y');
    CHECK(wf__completion_linux_io_uring_submissions() == native_before + 2);
    CHECK(wf__completion_file_fallback_submissions() == fallback_before);
    CHECK(wf__completion_target_helper_count() == 0);
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
#else
    (void)scratch_directory;
#endif
    return 0;
}

static int test_one_epoch_wakes_every_announced_scheduler(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked[2];
    pthread_t threads[2];
    _Atomic unsigned host_wakes;
    uint64_t epoch;
    size_t index;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
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
                park_scheduler,
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

static int test_drain_wakes_the_registered_token_owner(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_publication publication;
    wf_completion_event event;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked;
    pthread_t thread;
    uint64_t epoch;
    int value = 83;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    epoch = wf_completion_wake_epoch(&runtime);
    CHECK(
        wf_completion_wait_to_consume(&runtime, token)
        == WF_COMPLETION_CONSUME_WAIT_REGISTERED
    );
    parked.runtime = &runtime;
    parked.epoch = epoch;
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(
        wf_completion_wait_to_consume(&runtime, token)
        == WF_COMPLETION_CONSUME_READY
    );
    CHECK(consume_integer(&runtime, token, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

typedef struct consume_context {
    wf_completion_runtime *runtime;
    wf_completion_token token;
    int expected;
    int failed;
} consume_context;

static void *consume_on_other_lane(void *opaque) {
    consume_context *context = opaque;
    context->failed = consume_integer(
        context->runtime,
        context->token,
        context->expected
    );
    return NULL;
}

static int test_bounded_drain_and_lane_independence(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_completion_token tokens[4];
    wf_completion_event events[4];
    int values[4] = {5, 7, 11, 13};
    size_t index;
    size_t drained;
    consume_context other;
    pthread_t consumer;

    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    for (index = 0; index < 4; ++index) {
        wf_completion_publication publication;
        CHECK(wf_completion_claim(&runtime, &tokens[index]) == WF_COMPLETION_CLAIMED);
        CHECK(accept_operation(&runtime, tokens[index]) == 0);
        publication = integer_publication(&values[index]);
        CHECK(
            wf_completion_publish_terminal(&runtime, tokens[index], &publication)
            == WF_COMPLETION_PUBLISHED
        );
    }
    drained = wf_completion_drain(&runtime, events, 2, 4);
    CHECK(drained <= 2);
    CHECK(drained == 2);
    CHECK(wf_completion_ready_event_count(&runtime) == 2);
    CHECK(drain_exact(&runtime, 2, events + 2) == 0);
    CHECK(wf_completion_ready_event_count(&runtime) == 0);

    other.runtime = &runtime;
    other.token = tokens[0];
    other.expected = values[0];
    other.failed = 1;
    CHECK(pthread_create(&consumer, NULL, consume_on_other_lane, &other) == 0);
    CHECK(pthread_join(consumer, NULL) == 0);
    CHECK(other.failed == 0);
    for (index = 1; index < 4; ++index) {
        CHECK(consume_integer(&runtime, tokens[index], values[index]) == 0);
    }
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int drain_and_consume_file(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    wf_file_result *result
) {
    wf_completion_event event;
    wf_completion_outcome outcome;
    CHECK(drain_exact(runtime, 1, &event) == 0);
    CHECK(event.token.slot == token.slot);
    CHECK(event.token.generation == token.generation);
    CHECK(
        wf_completion_consume(
            runtime,
            token,
            result,
            sizeof(*result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(outcome.result_size == sizeof(*result));
    CHECK(result->kind != 0);
    return 0;
}

static int test_single_thread_file_progress_and_backpressure(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token open_token;
    wf_completion_token capacity_token;
    wf_completion_token operation_token;
    wf_file_request request;
    wf_file_result result;
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

    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0) == 0
    );
    CHECK(wf_completion_claim(&runtime, &open_token) == WF_COMPLETION_CLAIMED);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = root;
    request.operation.open_at.path = name;
    request.operation.open_at.flags = O_CREAT | O_EXCL | O_RDWR;
    request.operation.open_at.mode = 0600;
    request.operation.open_at.has_mode = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, open_token, &request)
        == WF_FILE_TARGET_OWNS
    );

    CHECK(
        wf_completion_claim(&runtime, &capacity_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = -1;
    CHECK(
        wf_file_adapter_submit(&adapter, capacity_token, &request)
        == WF_FILE_WAIT_CAPACITY
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(
        wf_file_adapter_submit(&adapter, capacity_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);

    CHECK(drain_and_consume_file(&runtime, open_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value >= 0);
    descriptor = (int)result.value;
    CHECK(drain_and_consume_file(&runtime, capacity_token, &result) == 0);
    CHECK(result.value == -1);
    CHECK(result.error_code == EBADF);

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PWRITE;
    request.operation.pwrite.descriptor = descriptor;
    request.operation.pwrite.buffer = payload;
    request.operation.pwrite.count = sizeof(payload) - 1;
    request.operation.pwrite.offset = 0;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == (int64_t)(sizeof(payload) - 1));

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = read_back;
    request.operation.pread.count = sizeof(payload) - 1;
    request.operation.pread.offset = 0;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == (int64_t)(sizeof(payload) - 1));
    CHECK(memcmp(read_back, payload, sizeof(payload) - 1) == 0);

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_STATUS;
    request.operation.status.descriptor = descriptor;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 0);
    CHECK(result.status_size == sizeof(struct stat));

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = descriptor;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 0);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(unlinkat(root, name, 0) == 0);
    CHECK(close(root) == 0);
    return 0;
}

static int test_bridge_independent_positioned_reads(
    const char *scratch_directory
) {
    wf_completion_token first_token;
    wf_completion_token second_token;
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

    /* Both stable tokens are submitted before either result is joined.  The
     * two requests name one descriptor but independent file offsets; pread
     * leaves the shared cursor out of their semantics. */
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            4,
            4,
            &first_token
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            second,
            4,
            11,
            &second_token
        ) == 1
    );
    wf__completion_file_join(
        &second_token,
        &second_value,
        &second_error
    );
    wf__completion_file_join(&first_token, &first_value, &first_error);
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

    /* A u64 which cannot be represented by the signed native offset never
     * enters the target queue and therefore owns no operation token. */
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            1,
            UINT64_MAX,
            &first_token
        ) == 0
    );
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_ordered_output_batch(const char *scratch_directory) {
    wf_completion_token first;
    wf_completion_token second;
    int64_t first_value = -1;
    int64_t second_value = -1;
    int first_error = -1;
    int second_error = -1;
    const unsigned char a = 'A';
    const unsigned char b = 'B';
    unsigned char bytes[2] = {0};
    char path[256];
    struct stat status;
    uint64_t logical_root = (uint64_t)(uintptr_t)&first;
    uint64_t batch;
    int descriptor;

    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-ordered-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    batch = wf__completion_output_batch_begin(logical_root, 2);
    CHECK(batch != 0);
    wf__completion_output_batch_submit(batch, descriptor, &a, 1, &first);
    wf__completion_output_batch_submit(batch, descriptor, &b, 1, &second);
    CHECK(fstat(descriptor, &status) == 0);
    CHECK(status.st_size == 0);
    wf__completion_output_batch_commit(batch);

    if (wf__completion_target_helper_count() != 0) {
        unsigned attempt;
        struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
        for (attempt = 0; attempt < 5000; ++attempt) {
            CHECK(fstat(descriptor, &status) == 0);
            /* The helper records its execution immediately after the target
             * call returns. File size can become visible in the few
             * instructions before that counter increment, so observe both
             * facts before treating the helper-path witness as complete. */
            if (status.st_size == 2
                && wf__completion_target_helper_executions() >= 2) {
                break;
            }
            (void)nanosleep(&delay, NULL);
        }
        CHECK(status.st_size == 2);
        CHECK(wf__completion_target_helper_executions() >= 2);
    } else {
        CHECK(wf__completion_target_helper_executions() == 0);
    }

    /* Joining the second reservation first still cannot execute it ahead of
     * the first physical write. */
    wf__completion_file_join(&second, &second_value, &second_error);
    wf__completion_file_join(&first, &first_value, &first_error);
    CHECK(first_value == 1 && first_error == 0);
    CHECK(second_value == 1 && second_error == 0);
    CHECK(pread(descriptor, bytes, sizeof(bytes), 0) == 2);
    CHECK(bytes[0] == 'A' && bytes[1] == 'B');
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_reserved_free_batch_inline_outcomes(void) {
    wf_completion_token tokens[2];
    int64_t value;
    int error_code;
    unsigned char byte = 0;
    uint64_t submissions = wf__completion_file_submissions();

    CHECK(wf__completion_file_batch_claim(tokens, 2, 1) == 1);
    /* Empty wins before offset validation and performs no host transfer. */
    wf__completion_file_pread_submit_reserved(
        -1,
        &byte,
        0,
        UINT64_MAX,
        &tokens[0]
    );
    /* A nonempty unrepresentable offset completes the already-reserved token
     * with the portable EINVAL input class, also without target handoff. */
    wf__completion_file_pread_submit_reserved(
        -1,
        &byte,
        1,
        UINT64_MAX,
        &tokens[1]
    );
    wf__completion_file_join(&tokens[0], &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    wf__completion_file_join(&tokens[1], &value, &error_code);
    CHECK(value == -1 && error_code == EINVAL);
    CHECK(wf__completion_file_submissions() == submissions);
    return 0;
}

static int test_ordered_failure_releases_the_later_bundle(
    const char *scratch_directory
) {
    wf_completion_token first;
    wf_completion_token second;
    int64_t first_value = 0;
    int64_t second_value = 0;
    int first_error = 0;
    int second_error = 0;
    const unsigned char b = 'B';
    unsigned char observed = 0;
    char path[256];
    uint64_t logical_root = (uint64_t)(uintptr_t)&second;
    uint64_t batch;
    int descriptor;

    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-ordered-failure-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    batch = wf__completion_output_batch_begin(logical_root, 2);
    CHECK(batch != 0);
    wf__completion_output_batch_submit(batch, -1, &b, 1, &first);
    wf__completion_output_batch_submit(batch, descriptor, &b, 1, &second);
    wf__completion_output_batch_commit(batch);
    wf__completion_file_join(&first, &first_value, &first_error);
    wf__completion_file_join(&second, &second_value, &second_error);
    CHECK(first_value == -1 && first_error == EBADF);
    CHECK(second_value == 1 && second_error == 0);
    CHECK(pread(descriptor, &observed, 1, 0) == 1);
    CHECK(observed == 'B');
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

typedef struct ordered_drain_context {
    int descriptor;
    size_t count;
} ordered_drain_context;

static void *drain_ordered_pipe_later(void *opaque) {
    ordered_drain_context *context = opaque;
    unsigned char bytes[4096];
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 5000000};
    size_t total = 0;
    (void)nanosleep(&delay, NULL);
    while (total < context->count) {
        ssize_t received = read(
            context->descriptor,
            bytes + total,
            context->count - total
        );
        if (received <= 0) {
            break;
        }
        total += (size_t)received;
    }
    return NULL;
}

static int test_ordered_partial_write_releases_the_later_bundle(
    const char *scratch_directory
) {
    wf_completion_token first;
    wf_completion_token second;
    int64_t first_value = 0;
    int64_t second_value = 0;
    int first_error = 0;
    int second_error = 0;
    unsigned char fill[4096];
    unsigned char drain[4096];
    unsigned char *payload;
    const unsigned char b = 'B';
    unsigned char observed = 0;
    char path[256];
    uint64_t logical_root = (uint64_t)(uintptr_t)&first;
    uint64_t batch;
    long pipe_buf;
    size_t drained = 0;
    size_t request_count;
    int pipe_descriptors[2];
    int flags;
    int descriptor;
    pthread_t reader;
    ordered_drain_context reader_context;

    memset(fill, 'f', sizeof(fill));
    CHECK(pipe(pipe_descriptors) == 0);
    flags = fcntl(pipe_descriptors[1], F_GETFL, 0);
    CHECK(flags >= 0);
    CHECK(fcntl(pipe_descriptors[1], F_SETFL, flags | O_NONBLOCK) == 0);
    for (;;) {
        ssize_t written = write(
            pipe_descriptors[1],
            fill,
            sizeof(fill)
        );
        if (written > 0) {
            continue;
        }
        CHECK(written < 0 && (errno == EAGAIN || errno == EWOULDBLOCK));
        break;
    }
    pipe_buf = fpathconf(pipe_descriptors[1], _PC_PIPE_BUF);
    CHECK(pipe_buf > 0 && (size_t)pipe_buf <= sizeof(drain));
    while (drained < (size_t)pipe_buf) {
        ssize_t received = read(
            pipe_descriptors[0],
            drain + drained,
            (size_t)pipe_buf - drained
        );
        CHECK(received > 0);
        drained += (size_t)received;
    }
    request_count = (size_t)pipe_buf * 2u;
    payload = malloc(request_count);
    CHECK(payload != NULL);
    memset(payload, 'A', request_count);

    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-ordered-partial-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    batch = wf__completion_output_batch_begin(logical_root, 2);
    CHECK(batch != 0);
    wf__completion_output_batch_submit(
        batch,
        pipe_descriptors[1],
        payload,
        request_count,
        &first
    );
    wf__completion_output_batch_submit(
        batch,
        descriptor,
        &b,
        1,
        &second
    );
    reader_context.descriptor = pipe_descriptors[0];
    reader_context.count = (size_t)pipe_buf;
    CHECK(
        pthread_create(
            &reader,
            NULL,
            drain_ordered_pipe_later,
            &reader_context
        ) == 0
    );
    wf__completion_output_batch_commit(batch);
    wf__completion_file_join(&first, &first_value, &first_error);
    wf__completion_file_join(&second, &second_value, &second_error);
    CHECK(pthread_join(reader, NULL) == 0);
    CHECK(first_error == 0);
    CHECK(first_value > 0 && (uint64_t)first_value < request_count);
    CHECK(second_value == 1 && second_error == 0);
    CHECK(pread(descriptor, &observed, 1, 0) == 1);
    CHECK(observed == 'B');
    free(payload);
    CHECK(close(descriptor) == 0);
    CHECK(close(pipe_descriptors[0]) == 0);
    CHECK(close(pipe_descriptors[1]) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

typedef struct ordered_capacity_context {
    uint64_t key;
    uint32_t expected;
    _Atomic unsigned returned;
    uint64_t result;
} ordered_capacity_context;

static void *begin_after_ordered_capacity(void *opaque) {
    ordered_capacity_context *context = opaque;
    context->result = wf__completion_output_batch_begin(
        context->key,
        context->expected
    );
    atomic_store_explicit(&context->returned, 1, memory_order_release);
    return NULL;
}

static int test_ordered_root_capacity_makes_progress(void) {
    wf_completion_token tokens[WF_ORDERED_OUTPUT_ROOT_CAPACITY][2];
    uint64_t batches[WF_ORDERED_OUTPUT_ROOT_CAPACITY];
    wf_completion_token extra[2];
    ordered_capacity_context context;
    pthread_t waiter;
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 2000000};
    size_t root;
    size_t member;

    memset(&context, 0, sizeof(context));
    atomic_init(&context.returned, 0);
    context.key = (uint64_t)(uintptr_t)&extra[0];
    context.expected = 2;
    for (root = 0; root < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++root) {
        uint64_t key = (uint64_t)(uintptr_t)&tokens[root][0];
        batches[root] = wf__completion_output_batch_begin(key, 2);
        CHECK(batches[root] != 0);
        for (member = 0; member < 2; ++member) {
            wf__completion_output_batch_submit(
                batches[root],
                -1,
                NULL,
                0,
                &tokens[root][member]
            );
        }
    }
    CHECK(
        pthread_create(
            &waiter,
            NULL,
            begin_after_ordered_capacity,
            &context
        ) == 0
    );
    (void)nanosleep(&delay, NULL);
    CHECK(atomic_load_explicit(&context.returned, memory_order_acquire) == 0);
    wf__completion_output_batch_commit(batches[0]);
    CHECK(pthread_join(waiter, NULL) == 0);
    CHECK(context.result != 0);
    for (member = 0; member < 2; ++member) {
        wf__completion_output_batch_submit(
            context.result,
            -1,
            NULL,
            0,
            &extra[member]
        );
    }
    wf__completion_output_batch_commit(context.result);
    for (root = 1; root < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++root) {
        wf__completion_output_batch_commit(batches[root]);
    }
    for (root = 0; root < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++root) {
        for (member = 0; member < 2; ++member) {
            int64_t value;
            int error_code;
            wf__completion_file_join(
                &tokens[root][member],
                &value,
                &error_code
            );
            CHECK(value == 0 && error_code == 0);
        }
    }
    for (member = 0; member < 2; ++member) {
        int64_t value;
        int error_code;
        wf__completion_file_join(
            &extra[member],
            &value,
            &error_code
        );
        CHECK(value == 0 && error_code == 0);
    }
    return 0;
}

static int test_ordered_batch_claims_are_all_or_none(void) {
    enum { BATCHES = 4, MEMBERS = 16 };
    wf_completion_token tokens[BATCHES][MEMBERS];
    wf_completion_token extra[MEMBERS];
    uint64_t batches[BATCHES];
    ordered_capacity_context context;
    pthread_t waiter;
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 2000000};
    uint64_t before;
    size_t batch;
    size_t member;

    memset(&context, 0, sizeof(context));
    atomic_init(&context.returned, 0);
    context.key = (uint64_t)(uintptr_t)&extra[0];
    context.expected = MEMBERS;
    before = wf__completion_publications();
    for (batch = 0; batch < BATCHES; ++batch) {
        if (getenv("WF_COMPLETION_TRACE") != NULL) {
            fprintf(stderr, "completion harness: claim pressure batch %zu\n", batch);
        }
        batches[batch] = wf__completion_output_batch_begin(
            (uint64_t)(uintptr_t)&tokens[batch][0],
            MEMBERS
        );
        CHECK(batches[batch] != 0);
        for (member = 0; member < MEMBERS; ++member) {
            wf__completion_output_batch_submit(
                batches[batch],
                -1,
                NULL,
                0,
                &tokens[batch][member]
            );
        }
    }
    CHECK(wf__completion_publications() == before);
    if (getenv("WF_COMPLETION_TRACE") != NULL) {
        fprintf(stderr, "completion harness: pressure slots full\n");
    }
    CHECK(
        pthread_create(
            &waiter,
            NULL,
            begin_after_ordered_capacity,
            &context
        ) == 0
    );
    (void)nanosleep(&delay, NULL);
    CHECK(atomic_load_explicit(&context.returned, memory_order_acquire) == 0);
    CHECK(wf__completion_publications() == before);

    if (getenv("WF_COMPLETION_TRACE") != NULL) {
        fprintf(stderr, "completion harness: commit first pressure batch\n");
    }
    wf__completion_output_batch_commit(batches[0]);
    for (member = 0; member < MEMBERS; ++member) {
        int64_t value;
        int error_code;
        wf__completion_file_join(
            &tokens[0][member],
            &value,
            &error_code
        );
        CHECK(value == 0 && error_code == 0);
        if (getenv("WF_COMPLETION_TRACE") != NULL) {
            fprintf(stderr, "completion harness: joined first member %zu\n", member);
        }
    }
    if (getenv("WF_COMPLETION_TRACE") != NULL) {
        fprintf(stderr, "completion harness: join capacity waiter\n");
    }
    CHECK(pthread_join(waiter, NULL) == 0);
    CHECK(context.result != 0);
    for (member = 0; member < MEMBERS; ++member) {
        wf__completion_output_batch_submit(
            context.result,
            -1,
            NULL,
            0,
            &extra[member]
        );
    }
    wf__completion_output_batch_commit(context.result);
    for (batch = 1; batch < BATCHES; ++batch) {
        wf__completion_output_batch_commit(batches[batch]);
    }
    for (batch = 1; batch < BATCHES; ++batch) {
        for (member = 0; member < MEMBERS; ++member) {
            int64_t value;
            int error_code;
            wf__completion_file_join(
                &tokens[batch][member],
                &value,
                &error_code
            );
            CHECK(value == 0 && error_code == 0);
        }
    }
    for (member = 0; member < MEMBERS; ++member) {
        int64_t value;
        int error_code;
        wf__completion_file_join(
            &extra[member],
            &value,
            &error_code
        );
        CHECK(value == 0 && error_code == 0);
    }
    return 0;
}

static int test_ordered_logical_root_reuse_is_generation_bound(void) {
    wf_completion_token tokens[2];
    uint64_t logical_root = (uint64_t)(uintptr_t)&tokens[0];
    uint64_t previous = 0;
    unsigned iteration;
    for (iteration = 0; iteration < 128; ++iteration) {
        uint64_t batch = wf__completion_output_batch_begin(logical_root, 2);
        size_t member;
        CHECK(batch != 0 && batch != previous);
        previous = batch;
        for (member = 0; member < 2; ++member) {
            wf__completion_output_batch_submit(
                batch,
                -1,
                NULL,
                0,
                &tokens[member]
            );
        }
        wf__completion_output_batch_commit(batch);
        for (member = 0; member < 2; ++member) {
            int64_t value;
            int error_code;
            wf__completion_file_join(
                &tokens[member],
                &value,
                &error_code
            );
            CHECK(value == 0 && error_code == 0);
        }
    }
    return 0;
}

static int test_process_wide_target_helper_budget(void) {
    const char *text = getenv("WF_IO_HELPERS");
    unsigned long expected = 1;
    if (text != NULL && *text != '\0') {
        char *end = NULL;
        errno = 0;
        expected = strtoul(text, &end, 10);
        if (errno != 0 || end == text || *end != '\0') {
            expected = 1;
        }
    }
    if (expected > 8) {
        expected = 8;
    }
    CHECK(wf__completion_target_helper_count() == expected);
    return 0;
}

static int test_helper_completion_wakes_scheduler(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_file_adapter adapter;
    wf_file_work queue[2];
    pthread_t helper;
    pthread_t scheduler;
    wf_completion_token token;
    wf_file_request request;
    wf_file_result result;
    park_context parked;
    int pipe_descriptors[2];
    char byte = 0;
    const char sent = 'x';
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;

    CHECK(pipe(pipe_descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 2, &helper, 1) == 0
    );
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = pipe_descriptors[0];
    request.operation.read.buffer = &byte;
    request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, token, &request)
        == WF_FILE_TARGET_OWNS
    );
    for (attempts = 0; attempts < 5000; ++attempts) {
        if (wf_file_adapter_queued(&adapter) == 0) {
            break;
        }
        (void)nanosleep(&delay, NULL);
    }
    CHECK(wf_file_adapter_queued(&adapter) == 0);

    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&scheduler, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    CHECK(write(pipe_descriptors[1], &sent, 1) == 1);
    CHECK(pthread_join(scheduler, NULL) == 0);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(drain_and_consume_file(&runtime, token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 1);
    CHECK(byte == sent);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(pipe_descriptors[0]) == 0);
    CHECK(close(pipe_descriptors[1]) == 0);
    return 0;
}

typedef struct readiness_writer_context {
    int descriptor;
    unsigned char byte;
} readiness_writer_context;

static void *write_after_readiness_wait(void *opaque) {
    readiness_writer_context *context = opaque;
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 2000000};
    ssize_t written;
    (void)nanosleep(&delay, NULL);
    written = write(context->descriptor, &context->byte, 1);
    (void)written;
    return NULL;
}

static int test_readiness_refusal_is_not_a_terminal_outcome(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token token;
    wf_file_request request;
    wf_file_result result;
    wf_completion_event event;
    wf_completion_outcome outcome;
    readiness_writer_context writer_context;
    pthread_t writer;
    int descriptors[2];
    int flags;
    unsigned char byte = 0;

    CHECK(pipe(descriptors) == 0);
    flags = fcntl(descriptors[0], F_GETFL, 0);
    CHECK(flags >= 0);
    CHECK(fcntl(descriptors[0], F_SETFL, flags | O_NONBLOCK) == 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = descriptors[0];
    request.operation.read.buffer = &byte;
    request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, token, &request)
        == WF_FILE_TARGET_OWNS
    );
    writer_context.descriptor = descriptors[1];
    writer_context.byte = 'r';
    CHECK(
        pthread_create(
            &writer,
            NULL,
            write_after_readiness_wait,
            &writer_context
        ) == 0
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(pthread_join(writer, NULL) == 0);
    CHECK(byte == 'r');
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
    CHECK(event.token.slot == token.slot);
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result.value == 1);
    CHECK(result.error_code == 0);
    CHECK(wf_completion_statistics_snapshot(&runtime).publications == 1);
    CHECK(wf_completion_statistics_snapshot(&runtime).consumptions == 1);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

typedef struct progress_context {
    wf_file_adapter *adapter;
    size_t progressed;
} progress_context;

static void *progress_one_file_operation(void *opaque) {
    progress_context *context = opaque;
    context->progressed = wf_file_adapter_progress(context->adapter, 1);
    return NULL;
}

static int test_capacity_release_wakes_before_blocking_work(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token read_token;
    wf_completion_token write_token;
    wf_completion_event events[2];
    wf_completion_outcome outcome;
    wf_file_request read_request;
    wf_file_request write_request;
    wf_file_result read_result;
    wf_file_result write_result;
    park_context parked;
    progress_context progress;
    pthread_t scheduler;
    pthread_t target_progress;
    int pipe_descriptors[2];
    char received = 0;
    const char sent = 'c';

    CHECK(pipe(pipe_descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0) == 0);
    CHECK(wf_completion_claim(&runtime, &read_token) == WF_COMPLETION_CLAIMED);
    memset(&read_request, 0, sizeof(read_request));
    read_request.kind = WF_FILE_READ;
    read_request.operation.read.descriptor = pipe_descriptors[0];
    read_request.operation.read.buffer = &received;
    read_request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, read_token, &read_request)
        == WF_FILE_TARGET_OWNS
    );

    CHECK(wf_completion_claim(&runtime, &write_token) == WF_COMPLETION_CLAIMED);
    memset(&write_request, 0, sizeof(write_request));
    write_request.kind = WF_FILE_WRITE;
    write_request.operation.write.descriptor = pipe_descriptors[1];
    write_request.operation.write.buffer = &sent;
    write_request.operation.write.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, write_token, &write_request)
        == WF_FILE_WAIT_CAPACITY
    );

    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&scheduler, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);

    /* This scheduler-driven target progress thread removes the read from the
     * full queue, publishes capacity, then blocks in the typed read.  The
     * capacity notification must wake the scheduler before the read can ever
     * complete, because submitting the waiting write is what unblocks it. */
    progress.adapter = &adapter;
    progress.progressed = 0;
    CHECK(
        pthread_create(
            &target_progress,
            NULL,
            progress_one_file_operation,
            &progress
        ) == 0
    );
    CHECK(pthread_join(scheduler, NULL) == 0);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(
        wf_file_adapter_submit(&adapter, write_token, &write_request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(pthread_join(target_progress, NULL) == 0);
    CHECK(progress.progressed == 1);
    CHECK(drain_exact(&runtime, 2, events) == 0);

    CHECK(
        wf_completion_consume(
            &runtime,
            read_token,
            &read_result,
            sizeof(read_result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(read_result.error_code == 0);
    CHECK(read_result.value == 1);
    CHECK(received == sent);
    CHECK(
        wf_completion_consume(
            &runtime,
            write_token,
            &write_result,
            sizeof(write_result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(write_result.error_code == 0);
    CHECK(write_result.value == 1);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(pipe_descriptors[0]) == 0);
    CHECK(close(pipe_descriptors[1]) == 0);
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
    CHECK(windows.implemented == 0);
    CHECK(windows.native_completion == 1);
    return 0;
}

static int test_darwin_directory_progress_is_internal(void) {
#if defined(__APPLE__)
    int descriptors[2];
    unsigned char readiness = 'r';
    unsigned char byte = 0;
    int64_t position = 0;
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
    CHECK(position == 17);
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

static int benchmark_core_roundtrip(uint64_t *nanoseconds_per_operation) {
    enum { ITERATIONS = 100000 };
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_event event;
    uint64_t start;
    uint64_t elapsed;
    int iteration;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    start = monotonic_nanoseconds();
    for (iteration = 0; iteration < ITERATIONS; ++iteration) {
        wf_completion_token token;
        wf_completion_publication publication;
        wf_completion_outcome outcome;
        int result = 0;
        CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
        CHECK(accept_operation(&runtime, token) == 0);
        publication = integer_publication(&iteration);
        CHECK(
            wf_completion_publish_terminal(&runtime, token, &publication)
            == WF_COMPLETION_PUBLISHED
        );
        CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
        CHECK(
            wf_completion_consume(
                &runtime,
                token,
                &result,
                sizeof(result),
                &outcome
            ) == WF_COMPLETION_CONSUMED
        );
        CHECK(result == iteration);
    }
    elapsed = monotonic_nanoseconds() - start;
    *nanoseconds_per_operation = elapsed / ITERATIONS;
    CHECK(
        wf_completion_statistics_snapshot(&runtime).wake_signals == 0
    );
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

int main(int argc, char **argv) {
    uint64_t roundtrip_ns = 0;
    int trace = getenv("WF_COMPLETION_TRACE") != NULL;
#define RUN_TEST(...)                                                         \
    do {                                                                      \
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
    RUN_TEST(test_capacity_and_product_state());
    RUN_TEST(test_generation_and_duplicate_terminal());
    RUN_TEST(test_exactly_one_terminal_under_race());
    RUN_TEST(test_batch_admission_wake_is_not_capacity());
    RUN_TEST(test_failed_batch_does_not_invent_capacity());
    RUN_TEST(test_unified_wake_epoch());
    RUN_TEST(test_one_epoch_wakes_every_announced_scheduler());
    RUN_TEST(test_drain_wakes_the_registered_token_owner());
    RUN_TEST(test_bounded_drain_and_lane_independence());
    RUN_TEST(test_linux_native_free_batch_avoids_fallback_thread(argv[1]));
    RUN_TEST(test_single_thread_file_progress_and_backpressure(argv[1]));
    RUN_TEST(test_bridge_independent_positioned_reads(argv[1]));
    RUN_TEST(test_ordered_output_batch(argv[1]));
    RUN_TEST(test_reserved_free_batch_inline_outcomes());
    RUN_TEST(test_ordered_failure_releases_the_later_bundle(argv[1]));
    RUN_TEST(test_ordered_partial_write_releases_the_later_bundle(argv[1]));
    RUN_TEST(test_ordered_root_capacity_makes_progress());
    RUN_TEST(test_ordered_batch_claims_are_all_or_none());
    RUN_TEST(test_ordered_logical_root_reuse_is_generation_bound());
    RUN_TEST(test_process_wide_target_helper_budget());
    RUN_TEST(test_helper_completion_wakes_scheduler());
    RUN_TEST(test_readiness_refusal_is_not_a_terminal_outcome());
    RUN_TEST(test_capacity_release_wakes_before_blocking_work());
    RUN_TEST(test_native_contract_inventory());
    RUN_TEST(test_darwin_directory_progress_is_internal());
    RUN_TEST(benchmark_core_roundtrip(&roundtrip_ns));
#undef RUN_TEST
    printf(
        "completion-core-harness: PASS core_roundtrip_ns=%" PRIu64
        " terminal_racers=%d\n",
        roundtrip_ns,
        TERMINAL_RACERS
    );
    return 0;
}
