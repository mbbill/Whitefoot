#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include "contract.h"
#include "file_adapter.h"
#include "native_contract.h"

#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <pthread.h>
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

static int wait_until_parked(wf_completion_runtime *runtime) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;
    for (attempts = 0; attempts < 5000; ++attempts) {
        if (wf_completion_parked_scheduler_count(runtime) != 0) {
            return 0;
        }
        (void)nanosleep(&delay, NULL);
    }
    return 1;
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
    uint64_t epoch_before;
    int value = 73;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
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
    CHECK(drain_exact(&runtime, 1, &event) == 0);
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
    CHECK(linux.implemented == 0);
    CHECK(linux.native_completion == 1);
    CHECK(windows.implemented == 0);
    CHECK(windows.native_completion == 1);
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
    if (argc != 2) {
        fprintf(stderr, "usage: %s SCRATCH_DIRECTORY\n", argv[0]);
        return 2;
    }
    if (test_capacity_and_product_state() != 0
        || test_generation_and_duplicate_terminal() != 0
        || test_exactly_one_terminal_under_race() != 0
        || test_unified_wake_epoch() != 0
        || test_bounded_drain_and_lane_independence() != 0
        || test_single_thread_file_progress_and_backpressure(argv[1]) != 0
        || test_helper_completion_wakes_scheduler() != 0
        || test_capacity_release_wakes_before_blocking_work() != 0
        || test_native_contract_inventory() != 0
        || benchmark_core_roundtrip(&roundtrip_ns) != 0) {
        return 1;
    }
    printf(
        "completion-core-harness: PASS core_roundtrip_ns=%" PRIu64
        " terminal_racers=%d\n",
        roundtrip_ns,
        TERMINAL_RACERS
    );
    return 0;
}
