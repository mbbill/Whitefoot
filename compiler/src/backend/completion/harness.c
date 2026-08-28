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

/* The bridge's whole process-wide operation capacity. A test which means to
 * reach the capacity boundary must name the same number the bridge does. */
#define WF_HARNESS_OPERATION_CAPACITY 64u

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
    CHECK((milestones & WF_COMPLETION_RESOURCE_RELEASED) != 0);
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

#define CLAIM_RACERS 32

typedef struct claim_race_context {
    wf_completion_runtime *runtime;
    _Atomic unsigned *start;
    wf_completion_token token;
    enum wf_completion_claim_result result;
} claim_race_context;

static void *claim_racer(void *opaque) {
    claim_race_context *context = opaque;
    while (atomic_load_explicit(context->start, memory_order_acquire) == 0) {
        sched_yield();
    }
    context->result = wf_completion_claim(
        context->runtime,
        &context->token
    );
    return NULL;
}

static int test_concurrent_single_claims_are_unique(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[CLAIM_RACERS];
    wf_completion_event events[CLAIM_RACERS];
    pthread_t threads[CLAIM_RACERS];
    claim_race_context contexts[CLAIM_RACERS];
    unsigned char seen[CLAIM_RACERS] = {0};
    int values[CLAIM_RACERS];
    wf_completion_publication publication;
    wf_completion_token refused;
    _Atomic unsigned start;
    size_t index;

    atomic_init(&start, 0);
    CHECK(
        wf_completion_runtime_init(&runtime, slots, CLAIM_RACERS) == 0
    );
    for (index = 0; index < CLAIM_RACERS; ++index) {
        contexts[index].runtime = &runtime;
        contexts[index].start = &start;
        contexts[index].result = WF_COMPLETION_CLAIM_INVALID;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                claim_racer,
                &contexts[index]
            ) == 0
        );
    }
    atomic_store_explicit(&start, 1, memory_order_release);
    for (index = 0; index < CLAIM_RACERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(contexts[index].result == WF_COMPLETION_CLAIMED);
        CHECK(contexts[index].token.slot < CLAIM_RACERS);
        CHECK(contexts[index].token.generation == 1);
        CHECK(seen[contexts[index].token.slot] == 0);
        seen[contexts[index].token.slot] = 1;
    }
    CHECK(
        wf_completion_claim(&runtime, &refused)
        == WF_COMPLETION_CLAIM_WAIT_CAPACITY
    );

    for (index = 0; index < CLAIM_RACERS; ++index) {
        values[index] = (int)index;
        CHECK(accept_operation(&runtime, contexts[index].token) == 0);
        publication = integer_publication(&values[index]);
        CHECK(
            wf_completion_publish_terminal(
                &runtime,
                contexts[index].token,
                &publication
            ) == WF_COMPLETION_PUBLISHED
        );
    }
    CHECK(drain_exact(&runtime, CLAIM_RACERS, events) == 0);
    for (index = 0; index < CLAIM_RACERS; ++index) {
        CHECK(
            consume_integer(
                &runtime,
                contexts[index].token,
                values[index]
            ) == 0
        );
    }
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

static int test_linux_independent_operations_use_available_target(
    const char *scratch_directory
) {
#if defined(__linux__)
    wf_completion_token tokens[2];
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
            &tokens[0]
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            &bytes[1],
            1,
            1,
            &tokens[1]
        ) == 1
    );
    wf__completion_file_join(&tokens[0], &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    wf__completion_file_join(&tokens[1], &value, &error_code);
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

static int test_bridge_open_status_and_close_are_typed_operations(
    const char *scratch_directory
) {
    wf_completion_token token;
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(
        &token,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    descriptor = (int)value;

    CHECK(wf__completion_file_status_submit(descriptor, &token) == 1);
    wf__completion_file_status_join(
        &token,
        &value,
        &error_code,
        status,
        sizeof(status),
        &status_size
    );
    CHECK(value == 0 && error_code == 0);
    CHECK(status_size == sizeof(struct stat));

    CHECK(wf__completion_file_close_submit(descriptor, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    /* Closing a descriptor whose authority this program already gave up is a
     * typed refusal, not a crash and not a silent success. */
    CHECK(wf__completion_file_close_submit(descriptor, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
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

static int test_bridge_capacity_falls_back_per_operation(void) {
    wf_completion_token tokens[WF_HARNESS_OPERATION_CAPACITY];
    wf_completion_token refused;
    int64_t value = -1;
    int error_code = -1;
    size_t index;

    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        CHECK(
            wf__completion_file_pread_submit(
                -1,
                NULL,
                0,
                0,
                &tokens[index]
            ) == 1
        );
    }
    CHECK(
        wf__completion_file_pread_submit(-1, NULL, 0, 0, &refused)
        == 0
    );
    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        wf__completion_file_join(&tokens[index], &value, &error_code);
        CHECK(value == 0 && error_code == 0);
    }
    return 0;
}

static int test_checked_open_rejects_and_closes_nonregular_descriptors(
    const char *scratch_directory
) {
    wf_completion_token token;
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(
        &token,
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
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
    wf_completion_token token;
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(unlink(regular) == 0);
    return 0;
}

/* Opens exhaust the same bounded operation capacity every other completion
 * operation draws on, refuse honestly at the boundary, and the refused
 * request succeeds once capacity returns. */
static int test_open_capacity_refuses_and_resubmits(
    const char *scratch_directory
) {
    wf_completion_token tokens[WF_HARNESS_OPERATION_CAPACITY];
    wf_completion_token refused;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char path[256];
    size_t index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-open-capacity-%ld",
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

    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        CHECK(
            wf__completion_file_open_at_submit(
                AT_FDCWD,
                path,
                O_RDONLY,
                0u,
                0u,
                WF_FILE_EXPECT_REGULAR,
                &tokens[index]
            ) == 1
        );
    }
    /* No operation is left to claim, so the bridge refuses before touching a
     * token and the generated program takes its qualified direct call. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &refused
        ) == 0
    );
    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        wf__completion_file_open_join(
            &tokens[index],
            &value,
            &error_code,
            &open_outcome
        );
        CHECK(value >= 0 && error_code == 0);
        CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
        CHECK(wf__completion_file_close_submit((int)value, &tokens[index]) == 1);
    }
    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        wf__completion_file_join(&tokens[index], &value, &error_code);
        CHECK(value == 0 && error_code == 0);
    }
    /* Released capacity readmits the request that was refused. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &refused
        ) == 1
    );
    wf__completion_file_open_join(&refused, &value, &error_code, &open_outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf__completion_file_close_submit((int)value, &refused) == 1);
    wf__completion_file_join(&refused, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(unlink(path) == 0);
    return 0;
}

typedef struct wf_open_waiter {
    const char *path;
    unsigned yields;
    int result;
} wf_open_waiter;

static void *wf_open_waiter_main(void *opaque) {
    wf_open_waiter *waiter = opaque;
    wf_completion_token token;
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
            &token
        ) != 1) {
        return NULL;
    }
    /* Give the target every chance to reach terminal before this owner asks
     * for the result, so the join exercises the already-published path rather
     * than only the park-and-wake one. */
    for (yield = 0; yield < waiter->yields; ++yield) {
        (void)sched_yield();
    }
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    if (value < 0 || error_code != 0
        || open_outcome != WF_FILE_OPEN_SUCCEEDED) {
        return NULL;
    }
    if (wf__completion_file_close_submit((int)value, &token) != 1) {
        return NULL;
    }
    wf__completion_file_join(&token, &value, &error_code);
    waiter->result = value == 0 && error_code == 0 ? 0 : 1;
    return NULL;
}

/* Independent owners each hold their own open operation at once. No owner
 * observes another's result, and an owner whose operation finished before it
 * asked still consumes exactly its own. */
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
         * already-terminal and the still-in-flight join are covered. */
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

/* The helper policy makes two different promises and this pins both.
 *
 * A written WF_IO_HELPERS pins the count exactly, which is what every test
 * that names a helper configuration depends on. An unset one asks for
 * demand-driven growth: it starts at one and may reach the machine's own CPU
 * count, never past it and never past the process ceiling.
 *
 * The unset arm previously asserted exactly one helper. That was the fixed
 * default before growth existed, and it kept passing after growth shipped
 * only because the growth rule compared queue depth against helper count and
 * so almost never fired. Asserting the range is what the policy actually
 * promises; asserting one would now pin the defect instead of the contract. */
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
    wf_completion_token token;
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
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
            &token
        ) == 1
    );
    {
        int64_t moved = -1;
        int read_error = -1;
        wf__completion_file_join(&token, &moved, &read_error);
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
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
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
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
    long online;
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
    online = sysconf(_SC_NPROCESSORS_ONLN);
    if (online < 1) {
        online = 1;
    }
    if ((unsigned long)online < ceiling) {
        ceiling = (unsigned long)online;
    }
    /* Where a native ring carries the operations, the same unset policy asks
     * for no helpers at all and a waiting scheduler is the engine, so only
     * the ceiling applies there. */
    if (wf__completion_linux_io_uring_submissions() == 0) {
        CHECK(held >= 1);
    }
    CHECK(held <= ceiling);
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
    wf_completion_token token;
    unsigned char readiness = 'r';
    unsigned char byte = 0;
    int64_t position = 0;
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
    CHECK(position == 17);

    wf_directory_host_calls = 0;
    wf_directory_poll_calls = 0;
    byte = 0;
    position = 0;
    CHECK(
        wf__completion_directory_next_submit(
            descriptors[0],
            &byte,
            1,
            &position,
            &token
        ) == 1
    );
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(wf_directory_host_calls == 3);
    CHECK(wf_directory_poll_calls == 1);
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
    RUN_TEST(test_concurrent_single_claims_are_unique());
    RUN_TEST(test_unified_wake_epoch());
    RUN_TEST(test_one_epoch_wakes_every_announced_scheduler());
    RUN_TEST(test_drain_wakes_the_registered_token_owner());
    RUN_TEST(test_bounded_drain_and_lane_independence());
    RUN_TEST(test_linux_independent_operations_use_available_target(argv[1]));
    RUN_TEST(test_single_thread_file_progress_and_backpressure(argv[1]));
    RUN_TEST(test_bridge_independent_positioned_reads(argv[1]));
    RUN_TEST(test_bridge_open_status_and_close_are_typed_operations(argv[1]));
    RUN_TEST(test_bridge_capacity_falls_back_per_operation());
    RUN_TEST(test_checked_open_rejects_and_closes_nonregular_descriptors(argv[1]));
    RUN_TEST(test_open_failure_classes_are_typed_outcomes(argv[1]));
    RUN_TEST(test_open_capacity_refuses_and_resubmits(argv[1]));
    RUN_TEST(test_open_results_reach_every_independent_owner(argv[1]));
    RUN_TEST(test_uncached_reads_are_target_policy_only(argv[1]));
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
