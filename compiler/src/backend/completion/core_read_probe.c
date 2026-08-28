#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* F_NOCACHE, the Darwin half of the WF_IO_NOCACHE target policy, is a BSD
 * extension the POSIX macro above hides, exactly as in bridge.c. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

#include "contract.h"
#include "file_adapter.h"

#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

/*
 * Isolated generic-core and positioned-read qualification probe.
 *
 * This executable deliberately links only runtime.c and file_adapter.c.  It
 * must remain independent of bridge.c and the generated-code ABI.  The
 * Makefile fixes the complete source whitelist and supplements it with an nm
 * guard after these behavioural checks pass.
 */

#define CHECK(condition)                                                      \
    do {                                                                      \
        if (!(condition)) {                                                   \
            fprintf(                                                          \
                stderr,                                                       \
                "completion core/read probe: %s:%d: check failed: %s\n",    \
                __FILE__,                                                     \
                __LINE__,                                                     \
                #condition                                                    \
            );                                                                \
            return 1;                                                         \
        }                                                                     \
    } while (0)

static _Atomic uint64_t positioned_read_host_calls;

/* Narrow link-time hook used only by this probe.  Nonempty positioned reads
 * still reach the real host facility, while the empty case can prove that it
 * never crosses this boundary. */
ssize_t wf_completion_test_pread(
    int descriptor,
    void *buffer,
    size_t count,
    off_t offset
) {
    atomic_fetch_add_explicit(
        &positioned_read_host_calls,
        1,
        memory_order_relaxed
    );
    return pread(descriptor, buffer, count, offset);
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
    wf_completion_event *events,
    size_t expected
) {
    size_t attempts = 0;
    size_t total = 0;
    while (total < expected && attempts < runtime->slot_count + 2u) {
        total += wf_completion_drain(
            runtime,
            events + total,
            expected - total,
            runtime->slot_count
        );
        attempts += 1;
    }
    CHECK(total == expected);
    return 0;
}

static int consume_integer(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    int expected
) {
    int value = 0;
    wf_completion_outcome outcome;
    CHECK(
        wf_completion_consume(
            runtime,
            token,
            &value,
            sizeof(value),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(value == expected);
    CHECK(outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE);
    CHECK(outcome.terminal_kind == 1);
    CHECK(outcome.result_size == sizeof(value));
    return 0;
}

static int consume_file_result(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    wf_file_result *result
) {
    wf_completion_outcome outcome;
    memset(result, 0, sizeof(*result));
    CHECK(
        wf_completion_consume(
            runtime,
            token,
            result,
            sizeof(*result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE);
    CHECK(outcome.result_size == sizeof(*result));
    CHECK(outcome.terminal_kind == (result->error_code == 0 ? 1u : 2u));
    return 0;
}

static int complete_positioned_read(
    wf_completion_runtime *runtime,
    wf_file_adapter *adapter,
    int descriptor,
    void *buffer,
    size_t count,
    int64_t offset,
    wf_file_result *result
) {
    wf_completion_token token;
    wf_completion_event event;
    wf_file_request request;

    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = count;
    request.operation.pread.offset = offset;

    CHECK(wf_completion_claim(runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_file_adapter_submit(adapter, token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(adapter, 1) == 1);
    CHECK(drain_exact(runtime, &event, 1) == 0);
    CHECK(event.token.slot == token.slot);
    CHECK(event.token.generation == token.generation);
    CHECK(consume_file_result(runtime, token, result) == 0);
    return 0;
}

static int create_payload_file(
    const char *scratch_directory,
    char *path,
    size_t path_capacity
) {
    static const char payload[] = "abcdef";
    int descriptor;
    int written = snprintf(
        path,
        path_capacity,
        "%s/wf-completion-core-read-%ld",
        scratch_directory,
        (long)getpid()
    );
    CHECK(written > 0 && (size_t)written < path_capacity);
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, payload, sizeof(payload) - 1u)
          == (ssize_t)(sizeof(payload) - 1u));
    return descriptor;
}

static int test_positioned_read_result_boundaries(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_file_adapter adapter;
    wf_file_work queue[2];
    wf_file_result result;
    unsigned char empty_destination = 0xa5u;
    unsigned char full[3] = {0xa5u, 0xa5u, 0xa5u};
    unsigned char short_read[4] = {0xa5u, 0xa5u, 0xa5u, 0xa5u};
    unsigned char eof[3] = {0xa5u, 0xa5u, 0xa5u};
    unsigned char error[3] = {0xa5u, 0xa5u, 0xa5u};
    unsigned char negative[3] = {0xa5u, 0xa5u, 0xa5u};
    unsigned char unrepresentable[3] = {0xa5u, 0xa5u, 0xa5u};
    uint64_t host_calls_before;
    char path[256];
    int descriptor = create_payload_file(
        scratch_directory,
        path,
        sizeof(path)
    );

    CHECK(descriptor >= 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 2, NULL, 0) == 0
    );

    host_calls_before = atomic_load_explicit(
        &positioned_read_host_calls,
        memory_order_relaxed
    );
    CHECK(
        complete_positioned_read(
            &runtime,
            &adapter,
            -1,
            &empty_destination,
            0,
            -1,
            &result
        ) == 0
    );
    CHECK(result.kind == WF_FILE_PREAD);
    CHECK(result.value == 0 && result.error_code == 0);
    CHECK(empty_destination == 0xa5u);
    CHECK(
        atomic_load_explicit(
            &positioned_read_host_calls,
            memory_order_relaxed
        ) == host_calls_before
    );

    CHECK(
        complete_positioned_read(
            &runtime,
            &adapter,
            descriptor,
            full,
            sizeof(full),
            0,
            &result
        ) == 0
    );
    CHECK(result.value == 3 && result.error_code == 0);
    CHECK(memcmp(full, "abc", sizeof(full)) == 0);

    CHECK(
        complete_positioned_read(
            &runtime,
            &adapter,
            descriptor,
            short_read,
            sizeof(short_read),
            4,
            &result
        ) == 0
    );
    CHECK(result.value == 2 && result.error_code == 0);
    CHECK(short_read[0] == 'e' && short_read[1] == 'f');
    CHECK(short_read[2] == 0xa5u && short_read[3] == 0xa5u);

    CHECK(
        complete_positioned_read(
            &runtime,
            &adapter,
            descriptor,
            eof,
            sizeof(eof),
            6,
            &result
        ) == 0
    );
    CHECK(result.value == 0 && result.error_code == 0);
    CHECK(eof[0] == 0xa5u && eof[1] == 0xa5u && eof[2] == 0xa5u);

    CHECK(
        complete_positioned_read(
            &runtime,
            &adapter,
            -1,
            error,
            sizeof(error),
            0,
            &result
        ) == 0
    );
    CHECK(result.value == -1 && result.error_code == EBADF);
    CHECK(error[0] == 0xa5u && error[1] == 0xa5u && error[2] == 0xa5u);

    host_calls_before = atomic_load_explicit(
        &positioned_read_host_calls,
        memory_order_relaxed
    );
    CHECK(
        complete_positioned_read(
            &runtime,
            &adapter,
            descriptor,
            negative,
            sizeof(negative),
            -1,
            &result
        ) == 0
    );
    CHECK(result.value == -1 && result.error_code == EINVAL);
    CHECK(
        negative[0] == 0xa5u && negative[1] == 0xa5u
        && negative[2] == 0xa5u
    );
    CHECK(
        atomic_load_explicit(
            &positioned_read_host_calls,
            memory_order_relaxed
        ) == host_calls_before + 1u
    );

    /* file_adapter stores an int64_t offset.  On the supported 64-bit POSIX
     * targets every stored value is representable by off_t.  Keep the narrow
     * ABI case qualified for any target where off_t is smaller: rejection must
     * happen before the hook and must leave the destination untouched. */
    if ((int64_t)(off_t)INT64_MAX != INT64_MAX) {
        host_calls_before = atomic_load_explicit(
            &positioned_read_host_calls,
            memory_order_relaxed
        );
        CHECK(
            complete_positioned_read(
                &runtime,
                &adapter,
                descriptor,
                unrepresentable,
                sizeof(unrepresentable),
                INT64_MAX,
                &result
            ) == 0
        );
        CHECK(result.value == -1 && result.error_code == EINVAL);
        CHECK(
            unrepresentable[0] == 0xa5u
            && unrepresentable[1] == 0xa5u
            && unrepresentable[2] == 0xa5u
        );
        CHECK(
            atomic_load_explicit(
                &positioned_read_host_calls,
                memory_order_relaxed
            ) == host_calls_before
        );
    }

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_independent_reads_complete_in_reverse_order(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_completion_token first;
    wf_completion_token second;
    wf_completion_event event;
    wf_file_adapter adapter;
    wf_file_work queue[2];
    wf_file_request first_request;
    wf_file_request second_request;
    wf_file_result result;
    unsigned char first_byte = 0xa5u;
    unsigned char second_byte = 0xa5u;
    char path[256];
    int descriptor = create_payload_file(
        scratch_directory,
        path,
        sizeof(path)
    );

    CHECK(descriptor >= 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 2, NULL, 0) == 0
    );

    memset(&first_request, 0, sizeof(first_request));
    first_request.kind = WF_FILE_PREAD;
    first_request.operation.pread.descriptor = descriptor;
    first_request.operation.pread.buffer = &first_byte;
    first_request.operation.pread.count = 1;
    first_request.operation.pread.offset = 0;

    second_request = first_request;
    second_request.operation.pread.buffer = &second_byte;
    second_request.operation.pread.offset = 5;

    CHECK(wf_completion_claim(&runtime, &first) == WF_COMPLETION_CLAIMED);
    CHECK(wf_completion_claim(&runtime, &second) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_file_adapter_submit(&adapter, first, &first_request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(
        wf_file_adapter_submit(&adapter, second, &second_request)
        == WF_FILE_TARGET_OWNS
    );

    /* Zero-helper scheduler progress deliberately takes the newest request.
     * This makes the reverse completion observable instead of merely allowed. */
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(event.token.slot == second.slot);
    CHECK(event.token.generation == second.generation);
    CHECK(consume_file_result(&runtime, second, &result) == 0);
    CHECK(result.value == 1 && result.error_code == 0);
    CHECK(second_byte == 'f');
    CHECK(first_byte == 0xa5u);

    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(event.token.slot == first.slot);
    CHECK(event.token.generation == first.generation);
    CHECK(consume_file_result(&runtime, first, &result) == 0);
    CHECK(result.value == 1 && result.error_code == 0);
    CHECK(first_byte == 'a');

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_capacity_wait_and_retry(const char *scratch_directory) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_completion_token first;
    wf_completion_token waiting;
    wf_completion_token refused;
    wf_completion_token retried;
    wf_completion_event event;
    wf_completion_outcome outcome;
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_file_request first_request;
    wf_file_request waiting_request;
    wf_file_result result;
    uint32_t milestones = 0;
    unsigned phase = 0;
    unsigned char first_byte = 0xa5u;
    unsigned char waiting_byte = 0xa5u;
    int inline_value = 29;
    int inline_result = 0;
    char path[256];
    int descriptor = create_payload_file(
        scratch_directory,
        path,
        sizeof(path)
    );

    CHECK(descriptor >= 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0) == 0
    );

    memset(&first_request, 0, sizeof(first_request));
    first_request.kind = WF_FILE_PREAD;
    first_request.operation.pread.descriptor = descriptor;
    first_request.operation.pread.buffer = &first_byte;
    first_request.operation.pread.count = 1;
    first_request.operation.pread.offset = 1;
    waiting_request = first_request;
    waiting_request.operation.pread.buffer = &waiting_byte;
    waiting_request.operation.pread.offset = 2;

    CHECK(wf_completion_claim(&runtime, &first) == WF_COMPLETION_CLAIMED);
    CHECK(wf_completion_claim(&runtime, &waiting) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_file_adapter_submit(&adapter, first, &first_request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(
        wf_file_adapter_submit(&adapter, waiting, &waiting_request)
        == WF_FILE_WAIT_CAPACITY
    );
    CHECK(
        wf_completion_observe(&runtime, waiting, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(milestones == 0 && phase == WF_COMPLETION_WAIT_CAPACITY);
    CHECK(
        wf_completion_claim(&runtime, &refused)
        == WF_COMPLETION_CLAIM_WAIT_CAPACITY
    );

    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(event.token.slot == first.slot);
    CHECK(consume_file_result(&runtime, first, &result) == 0);
    CHECK(result.value == 1 && first_byte == 'b');

    /* Consuming the first operation releases core capacity; dequeuing it
     * released target capacity.  Both formerly refused admissions now retry. */
    CHECK(wf_completion_claim(&runtime, &retried) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_completion_begin_submit(&runtime, retried)
        == WF_COMPLETION_TRANSITIONED
    );
    {
        wf_completion_publication publication =
            integer_publication(&inline_value);
        CHECK(
            wf_completion_publish_inline_terminal(
                &runtime,
                retried,
                &publication
            ) == WF_COMPLETION_PUBLISHED
        );
    }
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            retried,
            &inline_result,
            sizeof(inline_result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(inline_result == inline_value);

    CHECK(
        wf_file_adapter_submit(&adapter, waiting, &waiting_request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(event.token.slot == waiting.slot);
    CHECK(consume_file_result(&runtime, waiting, &result) == 0);
    CHECK(result.value == 1 && waiting_byte == 'c');

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_inline_stale_and_duplicate_terminal(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token old_token;
    wf_completion_token new_token;
    wf_completion_event event;
    wf_completion_outcome outcome;
    wf_completion_publication publication;
    int first = 41;
    int malicious = 99;
    int second = 43;
    int observed = 0;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &old_token) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_completion_begin_submit(&runtime, old_token)
        == WF_COMPLETION_TRANSITIONED
    );
    publication = integer_publication(&first);
    CHECK(
        wf_completion_publish_inline_terminal(
            &runtime,
            old_token,
            &publication
        ) == WF_COMPLETION_PUBLISHED
    );
    publication = integer_publication(&malicious);
    CHECK(
        wf_completion_publish_inline_terminal(
            &runtime,
            old_token,
            &publication
        ) == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL
    );
    CHECK(
        wf_completion_consume(
            &runtime,
            old_token,
            &observed,
            sizeof(observed),
            &outcome
        ) == WF_COMPLETION_CONSUME_NOT_DRAINED
    );
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(consume_integer(&runtime, old_token, first) == 0);

    CHECK(wf_completion_claim(&runtime, &new_token) == WF_COMPLETION_CLAIMED);
    CHECK(new_token.slot == old_token.slot);
    CHECK(new_token.generation == old_token.generation + 1u);
    CHECK(accept_operation(&runtime, new_token) == 0);
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
    CHECK(drain_exact(&runtime, &event, 1) == 0);
    CHECK(consume_integer(&runtime, new_token, second) == 0);
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
        2000
    );
    return NULL;
}

static int wait_until_parked(
    wf_completion_runtime *runtime,
    unsigned expected
) {
    struct timespec pause = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;
    for (attempts = 0; attempts < 2000; ++attempts) {
        if (wf_completion_parked_scheduler_count(runtime) == expected) {
            return 0;
        }
        if ((attempts & 15u) == 0) {
            (void)nanosleep(&pause, NULL);
        } else {
            sched_yield();
        }
    }
    return 1;
}

static int test_drain_consume_and_scheduler_parking(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_event event;
    wf_completion_publication publication;
    park_context single;
    park_context multiple[2];
    pthread_t single_thread;
    pthread_t multiple_threads[2];
    uint64_t epoch;
    int value = 53;
    size_t index;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(
        wf_completion_wait_to_consume(&runtime, token)
        == WF_COMPLETION_CONSUME_WAIT_REGISTERED
    );
    epoch = wf_completion_wake_epoch(&runtime);
    single.runtime = &runtime;
    single.epoch = epoch;
    single.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&single_thread, NULL, park_scheduler, &single) == 0);
    CHECK(wait_until_parked(&runtime, 1) == 0);
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
    CHECK(pthread_join(single_thread, NULL) == 0);
    CHECK(single.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(
        wf_completion_wait_to_consume(&runtime, token)
        == WF_COMPLETION_CONSUME_READY
    );
    CHECK(consume_integer(&runtime, token, value) == 0);

    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    value = 59;
    publication = integer_publication(&value);
    epoch = wf_completion_wake_epoch(&runtime);
    for (index = 0; index < 2; ++index) {
        multiple[index].runtime = &runtime;
        multiple[index].epoch = epoch;
        multiple[index].result = WF_COMPLETION_PARK_FAILED;
        CHECK(
            pthread_create(
                &multiple_threads[index],
                NULL,
                park_scheduler,
                &multiple[index]
            ) == 0
        );
    }
    CHECK(wait_until_parked(&runtime, 2) == 0);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    for (index = 0; index < 2; ++index) {
        CHECK(pthread_join(multiple_threads[index], NULL) == 0);
        CHECK(multiple[index].result == WF_COMPLETION_PARK_WOKEN);
    }
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
    CHECK(consume_integer(&runtime, token, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s SCRATCH_DIRECTORY\n", argv[0]);
        return 2;
    }
    atomic_init(&positioned_read_host_calls, 0);
    CHECK(test_positioned_read_result_boundaries(argv[1]) == 0);
    CHECK(test_independent_reads_complete_in_reverse_order(argv[1]) == 0);
    CHECK(test_capacity_wait_and_retry(argv[1]) == 0);
    CHECK(test_inline_stale_and_duplicate_terminal() == 0);
    CHECK(test_drain_consume_and_scheduler_parking() == 0);
    puts("completion-core-read-probe: PASS");
    return 0;
}
