/*
 * Standalone target-contract probe.
 *
 * Linux links this file with contract runtime.c and linux_io_uring.c and runs
 * real positioned reads/writes.  Windows links it with windows_iocp.c; its
 * tiny publication sink isolates the target adapter so a PE executable can
 * be compile- and link-checked before a Windows runner is available.
 */

#if defined(__linux__)

#define _GNU_SOURCE

#include "linux_io_uring.h"

#include <errno.h>
#include <fcntl.h>
#include <linux/time_types.h>
#include <pthread.h>
#include <sched.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

#define PROBE_CHECK(condition)                                                \
    do {                                                                      \
        if (!(condition)) {                                                   \
            fprintf(                                                          \
                stderr,                                                       \
                "native adapter probe failed: %s:%d: %s\n",                 \
                __FILE__,                                                     \
                __LINE__,                                                     \
                #condition                                                    \
            );                                                                \
            return 1;                                                         \
        }                                                                     \
    } while (0)

static int probe_drain(
    wf_completion_runtime *runtime,
    wf_completion_event *events,
    size_t expected
) {
    size_t total = 0;
    while (total < expected) {
        size_t drained = wf_completion_drain(
            runtime,
            events + total,
            expected - total,
            runtime->slot_count
        );
        if (drained == 0) {
            return 1;
        }
        total += drained;
    }
    return 0;
}

static int probe_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    enum wf_linux_file_operation_kind expected_kind,
    int64_t expected_value
) {
    wf_linux_file_result result;
    wf_completion_outcome outcome;
    PROBE_CHECK(
        wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    PROBE_CHECK(result.kind == expected_kind);
    PROBE_CHECK(result.value == expected_value);
    PROBE_CHECK(result.error_code == 0);
    PROBE_CHECK(outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE);
    return 0;
}

typedef struct probe_park_context {
    wf_linux_io_uring_adapter *adapter;
    uint64_t epoch;
    int result;
} probe_park_context;

static void *probe_park_scheduler(void *opaque) {
    probe_park_context *context = opaque;
    context->result = wf_linux_io_uring_park(
        context->adapter,
        context->epoch,
        UINT32_MAX
    );
    return NULL;
}

static int probe_wait_until_announced(wf_completion_runtime *runtime) {
    unsigned attempt;
    for (attempt = 0; attempt < 1000000u; ++attempt) {
        if (wf_completion_parked_scheduler_count(runtime) != 0) {
            return 0;
        }
        (void)sched_yield();
    }
    return 1;
}

/* A real delayed CQE lets the probe distinguish a ring-fd wake from the
 * completion core's eventfd.  It is not an adapter operation and is consumed
 * directly by this target-contract probe. */
static int probe_submit_timeout(
    wf_linux_io_uring_adapter *adapter,
    struct __kernel_timespec *delay
) {
    unsigned head;
    unsigned tail;
    unsigned ring_index;
    struct io_uring_sqe *submission;
    long entered;
    (void)pthread_mutex_lock(&adapter->submission_lock);
    head = __atomic_load_n(adapter->submission_head, __ATOMIC_ACQUIRE);
    tail = __atomic_load_n(adapter->submission_tail, __ATOMIC_RELAXED);
    if (tail - head >= *adapter->submission_count) {
        (void)pthread_mutex_unlock(&adapter->submission_lock);
        return EBUSY;
    }
    ring_index = tail & *adapter->submission_mask;
    submission = &adapter->submission_entries[ring_index];
    memset(submission, 0, sizeof(*submission));
    submission->opcode = IORING_OP_TIMEOUT;
    submission->addr = (uint64_t)(uintptr_t)delay;
    submission->len = 1;
    submission->user_data = 0;
    adapter->submission_array[ring_index] = ring_index;
    __atomic_store_n(adapter->submission_tail, tail + 1u, __ATOMIC_RELEASE);
    do {
        entered = syscall(
            __NR_io_uring_enter,
            adapter->ring_descriptor,
            1u,
            0u,
            0u,
            NULL,
            0u
        );
    } while (entered < 0 && errno == EINTR);
    (void)pthread_mutex_unlock(&adapter->submission_lock);
    return entered < 0 ? errno : entered == 1 ? 0 : EIO;
}

static int probe_reap_timeout(wf_linux_io_uring_adapter *adapter) {
    unsigned head = __atomic_load_n(
        adapter->completion_head,
        __ATOMIC_RELAXED
    );
    unsigned tail = __atomic_load_n(
        adapter->completion_tail,
        __ATOMIC_ACQUIRE
    );
    struct io_uring_cqe completion;
    if (head == tail) {
        return EAGAIN;
    }
    completion = adapter->completion_entries[head & *adapter->completion_mask];
    if (completion.user_data != 0 || completion.res != -ETIME) {
        return EPROTO;
    }
    __atomic_store_n(adapter->completion_head, head + 1u, __ATOMIC_RELEASE);
    return 0;
}

enum { PROBE_TOKEN_WAITERS = 4 };

typedef struct probe_token_wait_context {
    wf_linux_io_uring_adapter *adapter;
    wf_completion_runtime *runtime;
    wf_completion_token token;
    int64_t expected;
    int result;
} probe_token_wait_context;

static void *probe_wait_for_own_token(void *opaque) {
    probe_token_wait_context *context = opaque;
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(context->runtime);
        uint32_t milestones = 0;
        unsigned phase = 0;
        enum wf_completion_transition_result observed = wf_completion_observe(
            context->runtime,
            context->token,
            &milestones,
            &phase
        );
        if (observed != WF_COMPLETION_TRANSITIONED) {
            context->result = 1;
            return NULL;
        }
        if (phase == WF_COMPLETION_TERMINAL_PHASE) {
            wf_linux_file_result result;
            wf_completion_outcome outcome;
            enum wf_completion_consume_result consumed;
            do {
                wf_completion_event events[PROBE_TOKEN_WAITERS];
                (void)wf_completion_drain(
                    context->runtime,
                    events,
                    PROBE_TOKEN_WAITERS,
                    context->runtime->slot_count
                );
                consumed = wf_completion_consume(
                    context->runtime,
                    context->token,
                    &result,
                    sizeof(result),
                    &outcome
                );
            } while (consumed == WF_COMPLETION_CONSUME_NOT_DRAINED);
            if (consumed != WF_COMPLETION_CONSUMED
                || result.kind != WF_LINUX_FILE_READ_AT
                || result.value != context->expected
                || result.error_code != 0
                || outcome.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE) {
                context->result = 2;
                return NULL;
            }
            context->result = 0;
            return NULL;
        }
        context->result = wf_linux_io_uring_park(
            context->adapter,
            epoch,
            UINT32_MAX
        );
        if (context->result != 0) {
            return NULL;
        }
    }
}

static int probe_wait_for_parked_count(
    wf_completion_runtime *runtime,
    unsigned expected
) {
    unsigned attempt;
    for (attempt = 0; attempt < 1000000u; ++attempt) {
        if (wf_completion_parked_scheduler_count(runtime) == expected) {
            return 0;
        }
        (void)sched_yield();
    }
    return 1;
}

/* Drives one already-owned operation to its terminal and consumes it.  A
 * kind-checked open passes through more than one ring operation on the way,
 * which this loop deliberately does not need to know: it waits for the one
 * terminal the contract promises. */
static int probe_drive_to_terminal(
    wf_completion_runtime *runtime,
    wf_linux_io_uring_adapter *adapter,
    wf_completion_token token,
    wf_linux_file_result *result
) {
    wf_completion_event event;
    wf_completion_outcome outcome;
    unsigned attempt;
    for (attempt = 0; attempt < 100000u; ++attempt) {
        size_t published = 0;
        uint32_t milestones = 0;
        unsigned phase = 0;
        PROBE_CHECK(
            wf_completion_observe(runtime, token, &milestones, &phase)
            == WF_COMPLETION_TRANSITIONED
        );
        if (phase == WF_COMPLETION_TERMINAL_PHASE) {
            break;
        }
        PROBE_CHECK(
            wf_linux_io_uring_progress(adapter, 4, 1, &published) == 0
        );
    }
    PROBE_CHECK(probe_drain(runtime, &event, 1) == 0);
    PROBE_CHECK(
        wf_completion_consume(
            runtime,
            token,
            result,
            sizeof(*result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    PROBE_CHECK(outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE);
    return 0;
}

static int probe_run_one(
    wf_completion_runtime *runtime,
    wf_linux_io_uring_adapter *adapter,
    const wf_linux_file_request *request,
    wf_linux_file_result *result
) {
    wf_completion_token token;
    PROBE_CHECK(wf_completion_claim(runtime, &token) == WF_COMPLETION_CLAIMED);
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, token, request)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    return probe_drive_to_terminal(runtime, adapter, token, result);
}

static void probe_open_request(
    wf_linux_file_request *request,
    const char *path,
    enum wf_file_expected_kind expected
) {
    memset(request, 0, sizeof(*request));
    request->kind = WF_LINUX_FILE_OPEN_AT;
    request->descriptor = AT_FDCWD;
    request->buffer.path = path;
    request->open_flags = O_RDONLY;
    request->expected_kind = expected;
}

static void probe_close_request(
    wf_linux_file_request *request,
    int descriptor
) {
    memset(request, 0, sizeof(*request));
    request->kind = WF_LINUX_FILE_CLOSE;
    request->descriptor = descriptor;
}

/* Opens and closes are ring operations with the same typed answers the
 * bounded POSIX adapter gives, including the kind refusal that disposes of a
 * descriptor the writer must never receive. */
static int probe_open_and_close_cases(
    wf_completion_runtime *runtime,
    wf_linux_io_uring_adapter *adapter,
    const char *data_path
) {
    wf_linux_file_request request;
    wf_linux_file_result result;
    char directory[512];
    char missing[512];
    char fifo[512];
    char *separator;
    int descriptor;

    PROBE_CHECK(
        (size_t)snprintf(directory, sizeof(directory), "%s", data_path)
        < sizeof(directory)
    );
    separator = strrchr(directory, '/');
    PROBE_CHECK(separator != NULL);
    *separator = 0;
    PROBE_CHECK(
        (size_t)snprintf(
            missing,
            sizeof(missing),
            "%s/wf-probe-absent",
            directory
        ) < sizeof(missing)
    );
    PROBE_CHECK(
        (size_t)snprintf(fifo, sizeof(fifo), "%s/wf-probe-fifo", directory)
        < sizeof(fifo)
    );
    (void)unlink(missing);
    (void)unlink(fifo);
    PROBE_CHECK(mkfifo(fifo, 0600) == 0);

    /* A regular file the operation asked for opens and closes on the ring. */
    probe_open_request(&request, data_path, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.kind == WF_LINUX_FILE_OPEN_AT);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(result.error_code == 0);
    PROBE_CHECK(result.value >= 0);
    descriptor = (int)result.value;
    PROBE_CHECK(fcntl(descriptor, F_GETFD) != -1);

    probe_close_request(&request, descriptor);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.kind == WF_LINUX_FILE_CLOSE);
    PROBE_CHECK(result.value == 0 && result.error_code == 0);

    /* Closing a descriptor whose authority is already gone is a typed
     * refusal, never a second disposal of whatever reused the number. */
    probe_close_request(&request, descriptor);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.value < 0 && result.error_code == EBADF);

    /* A name that does not resolve fails before a descriptor exists. */
    probe_open_request(&request, missing, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_FAILED);
    PROBE_CHECK(result.error_code == ENOENT);
    PROBE_CHECK(result.value < 0);

    /* A directory is refused for a regular open, and the descriptor the ring
     * opened is disposed of on the same ring rather than leaked. */
    probe_open_request(&request, directory, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_IS_DIRECTORY);
    PROBE_CHECK(result.error_code == 0);
    PROBE_CHECK(result.value >= 0);
    errno = 0;
    PROBE_CHECK(fcntl((int)result.value, F_GETFD) == -1 && errno == EBADF);

    /* Any other kind is refused the same way. */
    probe_open_request(&request, fifo, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_OTHER_KIND);
    PROBE_CHECK(result.error_code == 0);
    PROBE_CHECK(result.value >= 0);
    errno = 0;
    PROBE_CHECK(fcntl((int)result.value, F_GETFD) == -1 && errno == EBADF);

    /* The refusals above are about the kind, not the request shape: the same
     * directory opens when a directory is what the operation asked for. */
    probe_open_request(&request, directory, WF_FILE_EXPECT_DIRECTORY);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(result.value >= 0);
    probe_close_request(&request, (int)result.value);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.value == 0 && result.error_code == 0);

    PROBE_CHECK(unlink(fifo) == 0);
    return 0;
}

/* A submitted open's path bytes belong to the entry, not to the caller.
 *
 * The SQE names the bytes until the kernel resolves them, and the caller
 * regains its name buffer the moment submission returns.  This is asserted of
 * the entry rather than of the outcome, and deliberately: on this kernel
 * `IORING_OP_OPENAT` copies the name during the `io_uring_enter` the submit
 * performs, so a behavioural probe would pass over a caller-owned pointer
 * today and only start failing once the doorbell is deferred.  The property
 * that has to hold is that the record owns the bytes, so that is what is
 * checked.
 *
 * The release milestone is checked in the same place and for the same reason:
 * `loan-released(path)` is exactly the claim that the caller's buffer is free
 * while the operation is still outstanding. */
static int probe_open_stages_its_own_path_case(
    wf_completion_runtime *runtime,
    wf_linux_io_uring_adapter *adapter,
    const char *data_path
) {
    wf_linux_file_request request;
    wf_linux_file_result result;
    wf_completion_token token;
    char requested[512];
    size_t index;
    size_t staged = 0;
    uint32_t milestones = 0;
    unsigned phase = 0;

    PROBE_CHECK(
        (size_t)snprintf(requested, sizeof(requested), "%s", data_path)
        < sizeof(requested)
    );
    probe_open_request(&request, requested, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(wf_completion_claim(runtime, &token) == WF_COMPLETION_CLAIMED);
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, token, &request)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );

    for (index = 0; index < adapter->entry_capacity; ++index) {
        wf_linux_io_uring_entry *entry = &adapter->entries[index];
        if (entry->token.slot != token.slot
            || entry->token.generation != token.generation
            || entry->kind != WF_LINUX_FILE_OPEN_AT) {
            continue;
        }
        PROBE_CHECK(entry->request.buffer.path == entry->path_storage);
        PROBE_CHECK(strcmp(entry->path_storage, data_path) == 0);
        staged += 1;
    }
    PROBE_CHECK(staged == 1);

    PROBE_CHECK(
        wf_completion_observe(runtime, token, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    PROBE_CHECK((milestones & WF_COMPLETION_PAYLOAD_RELEASED) != 0);
    PROBE_CHECK((milestones & WF_COMPLETION_TERMINAL) == 0);

    /* The caller's buffer really is free from here. */
    memset(requested, 'z', sizeof(requested) - 1u);
    requested[sizeof(requested) - 1u] = 0;
    PROBE_CHECK(probe_drive_to_terminal(runtime, adapter, token, &result) == 0);
    PROBE_CHECK(result.kind == WF_LINUX_FILE_OPEN_AT);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(result.error_code == 0 && result.value >= 0);

    probe_close_request(&request, (int)result.value);
    PROBE_CHECK(probe_run_one(runtime, adapter, &request, &result) == 0);
    PROBE_CHECK(result.value == 0 && result.error_code == 0);
    return 0;
}

/* An open exhausts the adapter's bounded entries like any other operation,
 * says so without taking ownership, and succeeds once an entry returns. */
static int probe_open_capacity_case(
    wf_completion_runtime *runtime,
    wf_linux_io_uring_adapter *adapter,
    const char *data_path
) {
    wf_linux_file_request request;
    wf_linux_file_result result;
    wf_completion_token held[2];
    wf_completion_token refused;
    unsigned index;

    probe_open_request(&request, data_path, WF_FILE_EXPECT_REGULAR);
    for (index = 0; index < 2u; ++index) {
        PROBE_CHECK(
            wf_completion_claim(runtime, &held[index]) == WF_COMPLETION_CLAIMED
        );
        PROBE_CHECK(
            wf_linux_io_uring_submit(adapter, held[index], &request)
            == WF_LINUX_IO_URING_TARGET_OWNS
        );
    }
    PROBE_CHECK(wf_completion_claim(runtime, &refused) == WF_COMPLETION_CLAIMED);
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, refused, &request)
        == WF_LINUX_IO_URING_WAIT_CAPACITY
    );
    for (index = 0; index < 2u; ++index) {
        PROBE_CHECK(
            probe_drive_to_terminal(runtime, adapter, held[index], &result) == 0
        );
        PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
        PROBE_CHECK(close((int)result.value) == 0);
    }
    /* Released capacity readmits the exact operation that was refused, with
     * its own token still owned by the caller. */
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, refused, &request)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    PROBE_CHECK(probe_drive_to_terminal(runtime, adapter, refused, &result) == 0);
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(close((int)result.value) == 0);
    return 0;
}

/* An open result is one terminal publication into one generation. A second
 * publication is refused as a duplicate, and a token copied before the slot
 * was recycled is refused as stale rather than overwriting a live open. */
static int probe_open_generation_cases(void) {
    /* One slot, so the operation claimed after the first is recycled onto
     * exactly the storage the stale token still names. */
    wf_completion_runtime storage;
    wf_completion_runtime *runtime = &storage;
    wf_completion_slot only;
    wf_completion_token token;
    wf_completion_token stale;
    wf_completion_event event;
    wf_completion_outcome outcome;
    wf_linux_file_result result;
    wf_linux_file_result published;
    wf_completion_publication publication;

    PROBE_CHECK(wf_completion_runtime_init(runtime, &only, 1) == 0);
    memset(&published, 0, sizeof(published));
    published.kind = WF_LINUX_FILE_OPEN_AT;
    published.value = 7;
    published.open_outcome = WF_FILE_OPEN_SUCCEEDED;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = 1;
    publication.result = &published;
    publication.result_size = sizeof(published);

    PROBE_CHECK(wf_completion_claim(runtime, &token) == WF_COMPLETION_CLAIMED);
    stale = token;
    PROBE_CHECK(
        wf_completion_begin_submit(runtime, token) == WF_COMPLETION_TRANSITIONED
    );
    PROBE_CHECK(
        wf_completion_target_accepted(runtime, token)
        == WF_COMPLETION_TRANSITIONED
    );
    PROBE_CHECK(
        wf_completion_publish_terminal(runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    PROBE_CHECK(
        wf_completion_publish_terminal(runtime, token, &publication)
        == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL
    );
    PROBE_CHECK(probe_drain(runtime, &event, 1) == 0);
    PROBE_CHECK(
        wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    PROBE_CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(result.value == 7);

    /* The recycled slot now belongs to a different operation, and the copied
     * token bits must fail generation validation before any result byte of
     * that operation changes. */
    {
        wf_completion_token reused;
        PROBE_CHECK(
            wf_completion_claim(runtime, &reused) == WF_COMPLETION_CLAIMED
        );
        PROBE_CHECK(reused.slot == stale.slot);
        PROBE_CHECK(reused.generation != stale.generation);
        PROBE_CHECK(
            wf_completion_begin_submit(runtime, reused)
            == WF_COMPLETION_TRANSITIONED
        );
        PROBE_CHECK(
            wf_completion_target_accepted(runtime, reused)
            == WF_COMPLETION_TRANSITIONED
        );
        PROBE_CHECK(
            wf_completion_publish_terminal(runtime, stale, &publication)
            == WF_COMPLETION_PUBLISH_STALE
        );
        published.value = 11;
        PROBE_CHECK(
            wf_completion_publish_terminal(runtime, reused, &publication)
            == WF_COMPLETION_PUBLISHED
        );
        PROBE_CHECK(probe_drain(runtime, &event, 1) == 0);
        PROBE_CHECK(
            wf_completion_consume(
                runtime,
                reused,
                &result,
                sizeof(result),
                &outcome
            ) == WF_COMPLETION_CONSUMED
        );
        PROBE_CHECK(result.value == 11);
    }
    PROBE_CHECK(wf_completion_runtime_destroy(runtime) == 0);
    return 0;
}

int main(int argc, char **argv) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[8];
    wf_linux_io_uring_adapter adapter;
    wf_linux_io_uring_entry adapter_entries[2];
    wf_completion_token first;
    wf_completion_token second;
    wf_completion_token capacity;
    wf_completion_event events[2];
    wf_linux_file_request request;
    unsigned char first_bytes[4] = {0};
    unsigned char second_bytes[4] = {0};
    const unsigned char seed[8] = {'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'};
    const unsigned char suffix[2] = {'I', 'J'};
    unsigned char final_bytes[10] = {0};
    size_t published = 0;
    int descriptor;
    int error;

    PROBE_CHECK(argc == 2);
    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 8) == 0);
    error = wf_linux_io_uring_init(
        &adapter,
        &runtime,
        adapter_entries,
        2
    );
    if (error != 0) {
        fprintf(
            stderr,
            "native adapter probe: io_uring qualification unavailable: %s\n",
            strerror(error)
        );
        PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0);
        return 77;
    }
    PROBE_CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            wf_linux_io_uring_notify,
            &adapter
        ) == 0
    );
    descriptor = open(argv[1], O_CREAT | O_TRUNC | O_RDWR, 0600);
    PROBE_CHECK(descriptor >= 0);
    PROBE_CHECK(pwrite(descriptor, seed, sizeof(seed), 0) == (ssize_t)sizeof(seed));

    PROBE_CHECK(wf_completion_claim(&runtime, &first) == WF_COMPLETION_CLAIMED);
    PROBE_CHECK(wf_completion_claim(&runtime, &second) == WF_COMPLETION_CLAIMED);
    PROBE_CHECK(wf_completion_claim(&runtime, &capacity) == WF_COMPLETION_CLAIMED);

    memset(&request, 0, sizeof(request));
    request.kind = WF_LINUX_FILE_READ_AT;
    request.descriptor = descriptor;
    request.buffer.read_buffer = first_bytes;
    request.count = sizeof(first_bytes);
    request.offset = 0;
    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, first, &request)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    request.buffer.read_buffer = second_bytes;
    request.offset = 4;
    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, second, &request)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );

    request.kind = WF_LINUX_FILE_WRITE_AT;
    request.buffer.write_buffer = suffix;
    request.count = sizeof(suffix);
    request.offset = sizeof(seed);
    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, capacity, &request)
        == WF_LINUX_IO_URING_WAIT_CAPACITY
    );

    while (published < 2) {
        size_t step = 0;
        PROBE_CHECK(
            wf_linux_io_uring_progress(&adapter, 2 - published, 1, &step) == 0
        );
        published += step;
    }
    PROBE_CHECK(probe_drain(&runtime, events, 2) == 0);
    PROBE_CHECK(probe_consume(&runtime, first, WF_LINUX_FILE_READ_AT, 4) == 0);
    PROBE_CHECK(probe_consume(&runtime, second, WF_LINUX_FILE_READ_AT, 4) == 0);
    PROBE_CHECK(memcmp(first_bytes, seed, 4) == 0);
    PROBE_CHECK(memcmp(second_bytes, seed + 4, 4) == 0);

    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, capacity, &request)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    published = 0;
    while (published == 0) {
        PROBE_CHECK(
            wf_linux_io_uring_progress(&adapter, 1, 1, &published) == 0
        );
    }
    PROBE_CHECK(probe_drain(&runtime, events, 1) == 0);
    PROBE_CHECK(probe_consume(&runtime, capacity, WF_LINUX_FILE_WRITE_AT, 2) == 0);
    PROBE_CHECK(pread(descriptor, final_bytes, sizeof(final_bytes), 0) == 10);
    PROBE_CHECK(memcmp(final_bytes, seed, sizeof(seed)) == 0);
    PROBE_CHECK(memcmp(final_bytes + sizeof(seed), suffix, sizeof(suffix)) == 0);

    /* Completion before sleep changes the epoch but performs no explicit host
     * wake. The subsequent park observes that fact and returns without
     * entering epoll. */
    {
        wf_completion_token early;
        wf_completion_publication publication;
        wf_linux_file_result result = {
            .kind = WF_LINUX_FILE_READ_AT,
            .value = 0,
            .error_code = 0,
        };
        uint64_t epoch;
        uint64_t writes;
        PROBE_CHECK(
            wf_completion_claim(&runtime, &early) == WF_COMPLETION_CLAIMED
        );
        PROBE_CHECK(
            wf_completion_begin_submit(&runtime, early)
                == WF_COMPLETION_TRANSITIONED
        );
        PROBE_CHECK(
            wf_completion_target_accepted(&runtime, early)
                == WF_COMPLETION_TRANSITIONED
        );
        epoch = wf_completion_wake_epoch(&runtime);
        writes = wf_linux_io_uring_statistics_snapshot(&adapter)
            .host_wake_writes;
        publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
        publication.terminal_kind = 1;
        publication.result = &result;
        publication.result_size = sizeof(result);
        PROBE_CHECK(
            wf_completion_publish_terminal(&runtime, early, &publication)
                == WF_COMPLETION_PUBLISHED
        );
        PROBE_CHECK(
            wf_linux_io_uring_statistics_snapshot(&adapter)
                .host_wake_writes == writes
        );
        PROBE_CHECK(
            wf_linux_io_uring_park(&adapter, epoch, UINT32_MAX) == 0
        );
        PROBE_CHECK(probe_drain(&runtime, events, 1) == 0);
        PROBE_CHECK(
            probe_consume(&runtime, early, WF_LINUX_FILE_READ_AT, 0) == 0
        );
    }

    /* Compute/capacity and native CQ facts share one kernel wait set. A core
     * notification wakes an announced scheduler through eventfd, and the
     * eventfd is empty again before that scheduler becomes active. */
    {
        probe_park_context parked = {
            .adapter = &adapter,
            .epoch = wf_completion_wake_epoch(&runtime),
            .result = -1,
        };
        pthread_t scheduler;
        uint64_t writes = wf_linux_io_uring_statistics_snapshot(&adapter)
            .host_wake_writes;
        uint64_t counter;
        PROBE_CHECK(
            pthread_create(
                &scheduler,
                NULL,
                probe_park_scheduler,
                &parked
            ) == 0
        );
        PROBE_CHECK(probe_wait_until_announced(&runtime) == 0);
        wf_completion_notify_compute(&runtime);
        PROBE_CHECK(pthread_join(scheduler, NULL) == 0);
        PROBE_CHECK(parked.result == 0);
        PROBE_CHECK(
            wf_linux_io_uring_statistics_snapshot(&adapter)
                .host_wake_writes == writes + 1u
        );
        errno = 0;
        PROBE_CHECK(
            read(adapter.wake_descriptor, &counter, sizeof(counter)) < 0
            && errno == EAGAIN
        );
    }

    /* The ring descriptor itself, not a millisecond condvar poll and not the
     * eventfd, wakes the scheduler when a delayed CQE appears. */
    {
        struct __kernel_timespec delay = {
            .tv_sec = 0,
            .tv_nsec = 100000000,
        };
        wf_linux_io_uring_statistics before =
            wf_linux_io_uring_statistics_snapshot(&adapter);
        uint64_t epoch = wf_completion_wake_epoch(&runtime);
        PROBE_CHECK(probe_submit_timeout(&adapter, &delay) == 0);
        PROBE_CHECK(
            wf_linux_io_uring_park(&adapter, epoch, UINT32_MAX) == 0
        );
        PROBE_CHECK(probe_reap_timeout(&adapter) == 0);
        {
            wf_linux_io_uring_statistics after =
                wf_linux_io_uring_statistics_snapshot(&adapter);
            PROBE_CHECK(after.kernel_waits == before.kernel_waits + 1u);
            PROBE_CHECK(after.kernel_wakes == before.kernel_wakes + 1u);
            PROBE_CHECK(after.host_wake_writes == before.host_wake_writes);
        }
    }

    /* Four lanes wait for four distinct terminal products. Publications may
     * race and coalesce into one eventfd level, but no lane may consume that
     * broadcast fact before every already-announced waiter has left epoll. */
    {
        probe_token_wait_context contexts[PROBE_TOKEN_WAITERS];
        wf_linux_file_result results[PROBE_TOKEN_WAITERS];
        pthread_t waiters[PROBE_TOKEN_WAITERS];
        unsigned index;
        uint64_t writes = wf_linux_io_uring_statistics_snapshot(&adapter)
            .host_wake_writes;
        for (index = 0; index < PROBE_TOKEN_WAITERS; ++index) {
            PROBE_CHECK(
                wf_completion_claim(&runtime, &contexts[index].token)
                    == WF_COMPLETION_CLAIMED
            );
            PROBE_CHECK(
                wf_completion_begin_submit(&runtime, contexts[index].token)
                    == WF_COMPLETION_TRANSITIONED
            );
            PROBE_CHECK(
                wf_completion_target_accepted(
                    &runtime,
                    contexts[index].token
                ) == WF_COMPLETION_TRANSITIONED
            );
            contexts[index].adapter = &adapter;
            contexts[index].runtime = &runtime;
            contexts[index].expected = (int64_t)index + 100;
            contexts[index].result = -1;
            memset(&results[index], 0, sizeof(results[index]));
            results[index].kind = WF_LINUX_FILE_READ_AT;
            results[index].value = contexts[index].expected;
            PROBE_CHECK(
                pthread_create(
                    &waiters[index],
                    NULL,
                    probe_wait_for_own_token,
                    &contexts[index]
                ) == 0
            );
        }
        PROBE_CHECK(
            probe_wait_for_parked_count(&runtime, PROBE_TOKEN_WAITERS) == 0
        );
        for (index = 0; index < PROBE_TOKEN_WAITERS; ++index) {
            wf_completion_publication publication = {
                .milestones = WF_COMPLETION_OWNERSHIP_COMPLETE,
                .terminal_kind = 1,
                .result = &results[index],
                .result_size = sizeof(results[index]),
            };
            PROBE_CHECK(
                wf_completion_publish_terminal(
                    &runtime,
                    contexts[index].token,
                    &publication
                ) == WF_COMPLETION_PUBLISHED
            );
            PROBE_CHECK(pthread_join(waiters[index], NULL) == 0);
            PROBE_CHECK(contexts[index].result == 0);
            PROBE_CHECK(
                probe_wait_for_parked_count(
                    &runtime,
                    PROBE_TOKEN_WAITERS - index - 1u
                ) == 0
            );
        }
        PROBE_CHECK(wf_completion_parked_scheduler_count(&runtime) == 0);
        PROBE_CHECK(
            wf_linux_io_uring_statistics_snapshot(&adapter)
                .host_wake_writes > writes
        );
        {
            uint64_t counter;
            errno = 0;
            PROBE_CHECK(
                read(adapter.wake_descriptor, &counter, sizeof(counter)) < 0
                && errno == EAGAIN
            );
        }
    }

    PROBE_CHECK(probe_open_and_close_cases(&runtime, &adapter, argv[1]) == 0);
    PROBE_CHECK(
        probe_open_stages_its_own_path_case(&runtime, &adapter, argv[1]) == 0
    );
    PROBE_CHECK(probe_open_capacity_case(&runtime, &adapter, argv[1]) == 0);
    PROBE_CHECK(probe_open_generation_cases() == 0);

    /* Once target ownership exists, a fatal progress condition is sticky and
     * observable by both progress and park. The production bridge fail-stops
     * on this result instead of falling back or waiting forever. */
    atomic_store_explicit(
        &adapter.progress_error,
        EIO,
        memory_order_release
    );
    published = 99;
    PROBE_CHECK(
        wf_linux_io_uring_progress(&adapter, 1, 0, &published) == EIO
    );
    PROBE_CHECK(published == 0);
    PROBE_CHECK(
        wf_linux_io_uring_park(
            &adapter,
            wf_completion_wake_epoch(&runtime),
            UINT32_MAX
        ) == EIO
    );

    PROBE_CHECK(wf_linux_io_uring_in_flight(&adapter) == 0);
    PROBE_CHECK(wf_linux_io_uring_destroy(&adapter) == 0);
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    PROBE_CHECK(close(descriptor) == 0);
    printf("native-adapter-probe target=linux-io-uring status=pass\n");
    return 0;
}

#elif defined(_WIN32)

#include "native_contract.h"
#include "windows_iocp.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct wf_completion_runtime {
    unsigned publications;
};

static unsigned probe_operations_accepted;
static unsigned probe_operations_retired;
static unsigned probe_descriptor_lease_releases;

void wf__windows_completion_descriptor_lease_release(
    wf_windows_descriptor_lease *lease
) {
    if (lease == NULL || lease->descriptor != 17 || lease->generation != 1
        || lease->handle == NULL || lease->handle == INVALID_HANDLE_VALUE
        || lease->completion_owner == NULL
        || lease->descriptor_class != WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE
        || lease->mode != WF_WINDOWS_DESCRIPTOR_LEASE_SHARED) {
        abort();
    }
    probe_descriptor_lease_releases += 1;
    memset(lease, 0, sizeof(*lease));
}

void wf_completion_operation_accepted(void) {
    probe_operations_accepted += 1;
}

void wf_completion_operation_retired(int returned_a_descriptor) {
    if (returned_a_descriptor != 0) {
        abort();
    }
    probe_operations_retired += 1;
}

enum wf_completion_transition_result wf_completion_begin_submit(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    (void)runtime;
    (void)token;
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_transition_result wf_completion_mark_wait_capacity(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    (void)runtime;
    (void)token;
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_transition_result wf_completion_target_accepted(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    (void)runtime;
    (void)token;
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_publish_result wf_completion_publish_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
) {
    (void)token;
    if (publication == NULL
        || publication->milestones != WF_COMPLETION_OWNERSHIP_COMPLETE) {
        return WF_COMPLETION_PUBLISH_INVALID_ARGUMENT;
    }
    runtime->publications += 1;
    return WF_COMPLETION_PUBLISHED;
}

void wf_completion_notify_capacity(wf_completion_runtime *runtime) {
    (void)runtime;
}

int main(int argc, char **argv) {
    wf_completion_runtime runtime = {0};
    wf_completion_target_contract contract = wf_completion_target_contract_for(
        WF_TARGET_WINDOWS_IOCP
    );
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[2];
    wf_windows_iocp_file file;
    wf_windows_file_request request;
    wf_completion_token token = {.slot = 0, .generation = 1};
    const unsigned char bytes[4] = {'i', 'o', 'c', 'p'};
    size_t published = 0;
    HANDLE handle;

    if (argc != 2) {
        return 2;
    }
    if (contract.implemented != 1
        || contract.native_completion != 1
        || contract.may_use_blocking_helpers != 0
        || contract.supports_scheduler_progress != 1) {
        return 8;
    }
    memset(entries, 0xa5, sizeof(entries));
    if (wf_windows_iocp_init(&adapter, &runtime, entries, 2, 0, 0) != 0) {
        return 3;
    }
    handle = CreateFileA(
        argv[1],
        GENERIC_READ | GENERIC_WRITE,
        0,
        NULL,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    if (handle == INVALID_HANDLE_VALUE
        || wf_windows_iocp_associate_file(&adapter, handle, &file) != 0) {
        return 4;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_WRITE_AT;
    request.file = file;
    request.lease.descriptor = 17;
    request.lease.generation = 1;
    request.lease.handle = handle;
    request.lease.completion_owner = &adapter;
    request.lease.descriptor_class = WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE;
    request.lease.mode = WF_WINDOWS_DESCRIPTOR_LEASE_SHARED;
    request.buffer.write_buffer = bytes;
    request.count = sizeof(bytes);
    request.offset = 0;
    if (wf_windows_iocp_submit(&adapter, token, &request)
        != WF_WINDOWS_IOCP_TARGET_OWNS) {
        return 5;
    }
    while (published == 0) {
        if (wf_windows_iocp_progress(&adapter, 1, INFINITE, &published) != 0) {
            return 6;
        }
    }
    if (runtime.publications != 1
        || probe_operations_accepted != 1
        || probe_operations_retired != 1
        || probe_descriptor_lease_releases != 1
        || entries[0].lease.generation != 0
        || entries[1].lease.generation != 0
        || wf_windows_iocp_destroy(&adapter) != 0
        || CloseHandle(handle) == FALSE) {
        return 7;
    }
    printf("native-adapter-probe target=windows-iocp status=pass\n");
    return 0;
}

#else

int main(void) {
    return 77;
}

#endif
