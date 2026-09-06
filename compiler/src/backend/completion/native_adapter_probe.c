/*
 * Standalone target-contract probe.
 *
 * Linux links this file with `runtime.c` and `linux_io_uring.c`; Windows with
 * `runtime.c`, `wait_windows.c` and `windows_iocp.c`.  Both arms link the
 * scheduler core and no bridge, so what they prove is the ring alone: a real
 * positioned transfer, found by the record's own address, published through
 * `wf_sched_complete`, and waited for on the ring's own park.
 */
#if defined(__linux__)

#define _GNU_SOURCE

#include "linux_io_uring.h"
#include "../sched/core.h"

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

/* The bridge's three parts of the runtime, supplied here because this probe
 * links no bridge on purpose.
 *
 * `wf_completion_record_complete` is the one publication, and the three
 * `wf__sched_host_*` hooks are design §7's platform item 2 -- one wait and
 * wake primitive -- bound to this probe's own runtime and ring exactly as the
 * bridge binds them to its own, so a waiter that registers itself in place on
 * a record is woken through the same eventfd the ring's park waits on. */
static wf_sched_core probe_core;
static wf_completion_runtime *probe_runtime;
static wf_linux_io_uring_adapter *probe_adapter;

void wf_completion_record_complete(wf_completion_record *record) {
    wf_sched_complete(&probe_core, &record->sched);
}

int wf__sched_host_epoch(uint64_t *epoch) {
    if (probe_runtime == NULL) {
        return 0;
    }
    *epoch = wf_completion_wake_epoch(probe_runtime);
    return 1;
}

int wf__sched_host_park(uint64_t observed) {
    if (probe_runtime == NULL || probe_adapter == NULL) {
        return 0;
    }
    (void)wf_linux_io_uring_park(probe_adapter, observed, UINT32_MAX);
    return 1;
}

int wf__sched_host_wake(void) {
    if (probe_runtime == NULL) {
        return 0;
    }
    wf_completion_notify_target(probe_runtime);
    return 1;
}

static void probe_record_init(
    wf_completion_record *record,
    enum wf_file_operation_kind kind
) {
    memset(record, 0, sizeof(*record));
    wf_sched_record_init(&record->sched);
    record->request.kind = kind;
    record->opened_descriptor = -1;
    record->open_outcome = WF_FILE_OPEN_SUCCEEDED;
}

static int probe_record_done(const wf_completion_record *record) {
    return __atomic_load_n(&record->sched.state, __ATOMIC_ACQUIRE)
        == WF_SCHED_DONE;
}

typedef struct probe_park_context {
    wf_linux_io_uring_adapter *adapter;
    uint64_t epoch;
    int result;
} probe_park_context;

static void *probe_park_thread(void *opaque) {
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

#if WF_IO_OWNER_RINGS
/* Two independent eventfds share one logical epoch. Each must stay readable
 * for its own announced waiters, and both must be drained after they leave. */
static void probe_notify_two_rings(void *context) {
    wf_linux_io_uring_adapter *adapters = context;
    wf_linux_io_uring_notify(&adapters[0]);
    wf_linux_io_uring_notify(&adapters[1]);
}

static int probe_two_rings_share_one_epoch(void) {
    wf_completion_runtime runtime;
    wf_linux_io_uring_adapter adapters[2];
    probe_park_context contexts[4];
    pthread_t threads[4];
    unsigned announced = 0;
    PROBE_CHECK(wf_completion_runtime_init(&runtime) == 0);
    for (unsigned index = 0; index < 2u; ++index) {
        PROBE_CHECK(wf_linux_io_uring_init(&adapters[index], &runtime, 8u, 16u) == 0);
    }
    PROBE_CHECK(wf_completion_set_wake_callback(&runtime, probe_notify_two_rings, adapters) == 0);
    for (unsigned index = 0; index < 4u; ++index) {
        contexts[index].adapter = &adapters[index / 2u];
        contexts[index].epoch = wf_completion_wake_epoch(&runtime);
        contexts[index].result = -1;
        PROBE_CHECK(pthread_create(&threads[index], NULL, probe_park_thread, &contexts[index]) == 0);
    }
    for (unsigned attempt = 0; attempt < 1000000u; ++attempt) {
        if (wf_completion_parked_scheduler_count(&runtime) == 4u) {
            announced = 1u;
            break;
        }
        (void)sched_yield();
    }
    PROBE_CHECK(announced != 0u);
    wf_completion_notify_compute(&runtime);
    for (unsigned index = 0; index < 4u; ++index) {
        PROBE_CHECK(pthread_join(threads[index], NULL) == 0);
        PROBE_CHECK(contexts[index].result == 0);
    }
    PROBE_CHECK(wf_completion_parked_scheduler_count(&runtime) == 0u);
    for (unsigned index = 0; index < 2u; ++index) {
        uint64_t counter;
        errno = 0;
        PROBE_CHECK(read(adapters[index].wake_descriptor, &counter, sizeof(counter)) < 0 && errno == EAGAIN);
        PROBE_CHECK(wf_linux_io_uring_statistics_snapshot(&adapters[index]).host_wake_writes == 1u);
        PROBE_CHECK(wf_linux_io_uring_destroy(&adapters[index]) == 0);
    }
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

#endif

/* A real delayed CQE lets the probe distinguish a ring-fd wake from the
 * completion runtime's eventfd.  It is not an adapter operation and is
 * consumed directly by this target-contract probe. */
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

enum { PROBE_RECORD_WAITERS = 4 };

typedef struct probe_record_wait_context {
    wf_linux_io_uring_adapter *adapter;
    wf_completion_runtime *runtime;
    wf_completion_record record;
    int64_t expected;
    int result;
} probe_record_wait_context;

/* One lane waiting in place on its own record: register the in-place marker,
 * capture the epoch, re-check, and park -- the same sequence the bridge's join
 * runs (design §2's fourth line, §6 step 4). */
static void *probe_wait_for_own_record(void *opaque) {
    probe_record_wait_context *context = opaque;
    for (;;) {
        uint64_t epoch;
        void *expected = NULL;
        void *marker = WF_SCHED_WAITER_IN_PLACE;
        if (probe_record_done(&context->record)) {
            context->result = context->record.result.value == context->expected
                    && context->record.result.error_code == 0
                ? 0
                : 2;
            return NULL;
        }
        (void)__atomic_compare_exchange_n(
            (uintptr_t *)&context->record.sched.waiter,
            (uintptr_t *)&expected,
            (uintptr_t)WF_SCHED_WAITER_IN_PLACE,
            0,
            __ATOMIC_SEQ_CST,
            __ATOMIC_SEQ_CST
        );
        epoch = wf_completion_wake_epoch(context->runtime);
        if (!probe_record_done(&context->record)) {
            context->result = wf_linux_io_uring_park(
                context->adapter,
                epoch,
                UINT32_MAX
            );
            if (context->result != 0) {
                return NULL;
            }
        }
        (void)__atomic_compare_exchange_n(
            (uintptr_t *)&context->record.sched.waiter,
            (uintptr_t *)&marker,
            (uintptr_t)NULL,
            0,
            __ATOMIC_SEQ_CST,
            __ATOMIC_SEQ_CST
        );
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

/* Drives one already-owned record to its terminal.  A kind-checked open
 * passes through more than one ring operation on the way, which this loop
 * deliberately does not need to know: it waits for the one terminal the
 * contract promises. */
static int probe_drive_to_terminal(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_record *record
) {
    unsigned attempt;
    for (attempt = 0; attempt < 100000u; ++attempt) {
        size_t published = 0;
        if (probe_record_done(record)) {
            return 0;
        }
        PROBE_CHECK(
            wf_linux_io_uring_progress(adapter, 4, 1, &published) == 0
        );
    }
    PROBE_CHECK(probe_record_done(record));
    return 0;
}

static int probe_run_one(
    wf_linux_io_uring_adapter *adapter,
    wf_completion_record *record
) {
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, record)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    return probe_drive_to_terminal(adapter, record);
}

static void probe_open_record(
    wf_completion_record *record,
    const char *path,
    enum wf_file_expected_kind expected
) {
    probe_record_init(record, WF_FILE_OPEN_AT);
    record->request.operation.open_at.directory = AT_FDCWD;
    record->request.operation.open_at.path = path;
    record->request.operation.open_at.flags = O_RDONLY;
    record->request.operation.open_at.expected_kind = expected;
}

static void probe_close_record(wf_completion_record *record, int descriptor) {
    probe_record_init(record, WF_FILE_CLOSE);
    record->request.operation.close.descriptor = descriptor;
}

/* Opens and closes are ring operations with the same typed answers the
 * bounded POSIX adapter gives, including the kind refusal that disposes of a
 * descriptor the writer must never receive. */
static int probe_open_and_close_cases(
    wf_linux_io_uring_adapter *adapter,
    const char *data_path
) {
    wf_completion_record record;
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
    probe_open_record(&record, data_path, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.kind == WF_FILE_OPEN_AT);
    PROBE_CHECK(record.result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(record.result.error_code == 0);
    PROBE_CHECK(record.result.value >= 0);
    descriptor = (int)record.result.value;
    PROBE_CHECK(fcntl(descriptor, F_GETFD) != -1);

    probe_close_record(&record, descriptor);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.kind == WF_FILE_CLOSE);
    PROBE_CHECK(record.result.value == 0 && record.result.error_code == 0);

    /* Closing a descriptor whose authority is already gone is a typed
     * refusal, never a second disposal of whatever reused the number. */
    probe_close_record(&record, descriptor);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.value < 0 && record.result.error_code == EBADF);

    /* A name that does not resolve fails before a descriptor exists. */
    probe_open_record(&record, missing, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.open_outcome == WF_FILE_OPEN_FAILED);
    PROBE_CHECK(record.result.error_code == ENOENT);
    PROBE_CHECK(record.result.value < 0);

    /* A directory is refused for a regular open, and the descriptor the ring
     * opened is disposed of rather than leaked. */
    probe_open_record(&record, directory, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.open_outcome == WF_FILE_OPEN_IS_DIRECTORY);
    PROBE_CHECK(record.result.error_code == 0);
    PROBE_CHECK(record.result.value >= 0);
    errno = 0;
    PROBE_CHECK(
        fcntl((int)record.result.value, F_GETFD) == -1 && errno == EBADF
    );

    /* Any other kind is refused the same way. */
    probe_open_record(&record, fifo, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.open_outcome == WF_FILE_OPEN_OTHER_KIND);
    PROBE_CHECK(record.result.error_code == 0);
    PROBE_CHECK(record.result.value >= 0);
    errno = 0;
    PROBE_CHECK(
        fcntl((int)record.result.value, F_GETFD) == -1 && errno == EBADF
    );

    /* The refusals above are about the kind, not the request shape: the same
     * directory opens when a directory is what the operation asked for. */
    probe_open_record(&record, directory, WF_FILE_EXPECT_DIRECTORY);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(record.result.value >= 0);
    descriptor = (int)record.result.value;
    probe_close_record(&record, descriptor);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.value == 0 && record.result.error_code == 0);

    PROBE_CHECK(unlink(fifo) == 0);
    return 0;
}

/* A submitted open resolves the submitting frame's own bytes.
 *
 * The SQE names the caller's buffer and the caller keeps it live until the
 * join, because the record is a block of that frame and [SYS-2]'s loan on the
 * path component holds for exactly that interval (design §5).  What used to be
 * checked here -- that the adapter had copied the name into a pool entry of
 * its own, and had published `loan-released(path)` to say so -- is retired
 * with the copy: there is no entry to copy into, no `WF_FILE_PATH_CAPACITY` to
 * exceed, no "path does not fit" refusal and no demoted-open counter (design
 * §7, "The record's pool machinery: deleted, not answered").  What remains is
 * that the SQE names the record's request, which is what this checks. */
static int probe_open_names_the_submitters_bytes(
    wf_linux_io_uring_adapter *adapter,
    const char *data_path
) {
    wf_completion_record record;
    char requested[512];

    PROBE_CHECK(
        (size_t)snprintf(requested, sizeof(requested), "%s", data_path)
        < sizeof(requested)
    );
    probe_open_record(&record, requested, WF_FILE_EXPECT_REGULAR);
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, &record)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    PROBE_CHECK(record.request.operation.open_at.path == requested);
    PROBE_CHECK(probe_drive_to_terminal(adapter, &record) == 0);
    PROBE_CHECK(record.result.kind == WF_FILE_OPEN_AT);
    PROBE_CHECK(record.result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    PROBE_CHECK(record.result.error_code == 0 && record.result.value >= 0);

    probe_close_record(&record, (int)record.result.value);
    PROBE_CHECK(probe_run_one(adapter, &record) == 0);
    PROBE_CHECK(record.result.value == 0 && record.result.error_code == 0);
    return 0;
}

/* More operations in flight than the ring is deep.
 *
 * This is what replaces `probe_open_capacity_case`, whose WAIT_CAPACITY answer
 * and readmission-after-release are both gone: the ring's depth is a
 * throughput parameter and no queue can refuse an operation, because a full
 * submission queue is emptied by the submitting call's own `io_uring_enter`
 * (design §7).  So the property to test is the opposite one -- that submitting
 * more operations than the ring is deep is accepted and every one of them
 * completes. */
static int probe_more_in_flight_than_the_ring_is_deep(
    wf_linux_io_uring_adapter *adapter,
    int descriptor
) {
    enum { OPERATIONS = 12 };
    wf_completion_record records[OPERATIONS];
    unsigned char bytes[OPERATIONS];
    unsigned index;

    PROBE_CHECK(wf_linux_io_uring_capacity(adapter) < OPERATIONS);
    for (index = 0; index < OPERATIONS; ++index) {
        bytes[index] = 0xa5u;
        probe_record_init(&records[index], WF_FILE_PREAD);
        records[index].request.operation.pread.descriptor = descriptor;
        records[index].request.operation.pread.buffer = &bytes[index];
        records[index].request.operation.pread.count = 1;
        records[index].request.operation.pread.offset = (int64_t)(index % 8u);
        PROBE_CHECK(
            wf_linux_io_uring_submit(adapter, &records[index])
            == WF_LINUX_IO_URING_TARGET_OWNS
        );
    }
    for (index = 0; index < OPERATIONS; ++index) {
        PROBE_CHECK(probe_drive_to_terminal(adapter, &records[index]) == 0);
        PROBE_CHECK(records[index].result.value == 1);
        PROBE_CHECK(records[index].result.error_code == 0);
        PROBE_CHECK(bytes[index] == (unsigned char)('a' + (index % 8u)));
    }
    return 0;
}

/* More completions than the completion queue holds, reaped without a wait.
 *
 * The queue is sixteen entries here.  Twenty-four one-byte reads are staged
 * and kicked together, so the kernel posts sixteen completions into the queue
 * and keeps eight on its overflow list under IORING_FEAT_NODROP.  Nothing but
 * an `io_uring_enter` moves those eight into the queue, and a program whose
 * every submitter is parked on one of them makes no further submission, so
 * the reaper has to make that call itself when the kernel raises
 * IORING_SQ_CQ_OVERFLOW.  This is the stall the network control test found
 * at 129 connections against a 128-entry queue: every thread idle, the ring
 * descriptor readable, and nothing reaped.  The reap here is the
 * non-waiting pass, which is the one a scheduler thread's progress makes. */
static int probe_completions_past_the_queue_are_flushed_from_the_kernel(
    wf_linux_io_uring_adapter *adapter,
    int descriptor
) {
    enum { OPERATIONS = 24 };
    wf_completion_record records[OPERATIONS];
    unsigned char bytes[OPERATIONS];
    wf_linux_io_uring_statistics before;
    wf_linux_io_uring_statistics after;
    size_t published = 0;
    unsigned attempt;
    unsigned index;

    PROBE_CHECK(*adapter->completion_count < OPERATIONS);
    before = wf_linux_io_uring_statistics_snapshot(adapter);
    for (index = 0; index < OPERATIONS; ++index) {
        bytes[index] = 0x5au;
        probe_record_init(&records[index], WF_FILE_PREAD);
        records[index].request.operation.pread.descriptor = descriptor;
        records[index].request.operation.pread.buffer = &bytes[index];
        records[index].request.operation.pread.count = 1;
        records[index].request.operation.pread.offset = (int64_t)(index % 8u);
        PROBE_CHECK(
            wf_linux_io_uring_submit(adapter, &records[index])
            == WF_LINUX_IO_URING_TARGET_OWNS
        );
    }
    PROBE_CHECK(wf_linux_io_uring_flush(adapter) == 0);
    for (attempt = 0; attempt < 100000u && published < OPERATIONS; ++attempt) {
        size_t step = 0;
        PROBE_CHECK(
            wf_linux_io_uring_progress(adapter, OPERATIONS, 0, &step) == 0
        );
        published += step;
    }
    PROBE_CHECK(published == OPERATIONS);
    for (index = 0; index < OPERATIONS; ++index) {
        PROBE_CHECK(probe_record_done(&records[index]));
        PROBE_CHECK(records[index].result.value == 1);
        PROBE_CHECK(records[index].result.error_code == 0);
        PROBE_CHECK(bytes[index] == (unsigned char)('a' + (index % 8u)));
    }
    after = wf_linux_io_uring_statistics_snapshot(adapter);
    PROBE_CHECK(after.overflow_flushes > before.overflow_flushes);
    return 0;
}

/* One loopback round trip on the ring: connect, accept, send, receive.
 *
 * This is the harness's own shape as a runtime probe -- the peer is the same
 * process, so nothing here waits on anything outside it -- and it is the one
 * place the four socket opcodes are exercised against the kernel without a
 * compiled Whitefoot program in front of them.  The listener is bound and
 * listened here rather than through the ring, exactly as the bridge does it:
 * a bind and a listen are immediate host calls with nothing to wait for
 * (`research/investigations/io-model/NETWORK.md` §5).
 *
 * The connect and the accept are submitted together, because a loopback
 * connect completes only once something accepts, and the record is a block of
 * this frame either way. */
static int probe_loopback_round_trip(wf_linux_io_uring_adapter *adapter) {
    wf_completion_record connecting;
    wf_completion_record accepting;
    wf_completion_record sending;
    wf_completion_record receiving;
    wf_socket_address address;
    wf_socket_native_address bound;
    struct sockaddr_in local;
    socklen_t local_length = (socklen_t)sizeof(local);
    const unsigned char message[5] = {'r', 'i', 'n', 'g', '!'};
    unsigned char received[5] = {0};
    unsigned length;
    int listener;
    int connected;
    int taken;

    memset(&address, 0, sizeof(address));
    /* 127.0.0.1 with the port the kernel chooses, filled in below. */
    address.words[0] = 0x0100007fu;
    address.port_and_family = 0;
    length = wf_socket_native_from_address(&address, &bound);
    listener = wf_socket_open(&address);
    PROBE_CHECK(listener >= 0);
    PROBE_CHECK(
        bind(listener, (const struct sockaddr *)bound.bytes, (socklen_t)length)
        == 0
    );
    PROBE_CHECK(listen(listener, 8) == 0);
    PROBE_CHECK(
        getsockname(listener, (struct sockaddr *)&local, &local_length) == 0
    );
    address.port_and_family = (unsigned)ntohs(local.sin_port);

    probe_record_init(&accepting, WF_FILE_SOCKET_ACCEPT);
    accepting.request.operation.accept.descriptor = listener;
    accepting.request.operation.accept.peer_length =
        (unsigned)sizeof(accepting.request.operation.accept.peer.native);
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, &accepting)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );

    probe_record_init(&connecting, WF_FILE_SOCKET_CONNECT);
    connecting.request.operation.endpoint.descriptor = -1;
    connecting.request.operation.endpoint.address.portable = address;
    PROBE_CHECK(
        wf_linux_io_uring_submit(adapter, &connecting)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );

    PROBE_CHECK(probe_drive_to_terminal(adapter, &connecting) == 0);
    PROBE_CHECK(probe_drive_to_terminal(adapter, &accepting) == 0);
    PROBE_CHECK(connecting.result.error_code == 0);
    PROBE_CHECK(accepting.result.error_code == 0);
    connected = (int)connecting.result.value;
    taken = (int)accepting.result.value;
    PROBE_CHECK(connected >= 0 && taken >= 0);
#if WF_TCP_NODELAY
    {
        int sockets[3] = {listener, connected, taken};
        for (unsigned index = 0; index < 3u; index++) {
            int enabled = 0;
            socklen_t bytes = (socklen_t)sizeof(enabled);
            PROBE_CHECK(getsockopt(sockets[index], IPPROTO_TCP, TCP_NODELAY, &enabled, &bytes) == 0);
            PROBE_CHECK(enabled != 0);
        }
    }
#endif
    /* The accept published the peer's own address in the portable form the
     * accept join reads, and a loopback peer is 127.0.0.1 with an ephemeral
     * port the target chose. */
    PROBE_CHECK(
        accepting.request.operation.accept.peer.portable.words[0]
        == 0x0100007fu
    );
    PROBE_CHECK(
        (accepting.request.operation.accept.peer.portable.port_and_family
         & WF_SOCKET_FAMILY_V6)
        == 0u
    );
    PROBE_CHECK(
        (accepting.request.operation.accept.peer.portable.port_and_family
         & WF_SOCKET_PORT_MASK)
        != 0u
    );

    probe_record_init(&sending, WF_FILE_SOCKET_SEND);
    sending.request.operation.send.descriptor = connected;
    sending.request.operation.send.buffer = message;
    sending.request.operation.send.count = sizeof(message);
    PROBE_CHECK(probe_run_one(adapter, &sending) == 0);
    PROBE_CHECK(sending.result.value == (int64_t)sizeof(message));

    probe_record_init(&receiving, WF_FILE_SOCKET_RECEIVE);
    receiving.request.operation.receive.descriptor = taken;
    receiving.request.operation.receive.buffer = received;
    receiving.request.operation.receive.count = sizeof(received);
    PROBE_CHECK(probe_run_one(adapter, &receiving) == 0);
    PROBE_CHECK(receiving.result.value == (int64_t)sizeof(message));
    PROBE_CHECK(memcmp(received, message, sizeof(message)) == 0);

    /* This probe links the ring and nothing else, so the pair's own two-count
     * -- which lives in the shared adapter and is what a half-close consults
     * [SYS-18] -- is exercised by `harness.c` instead. Here the three
     * descriptors are simply given back. */
    PROBE_CHECK(close(connected) == 0);
    PROBE_CHECK(close(taken) == 0);
    PROBE_CHECK(close(listener) == 0);
    return 0;
}

int main(int argc, char **argv) {
    wf_completion_runtime runtime;
    wf_linux_io_uring_adapter adapter;
    wf_completion_record first;
    wf_completion_record second;
    wf_completion_record third;
    unsigned char first_bytes[4] = {0};
    unsigned char second_bytes[4] = {0};
    const unsigned char seed[8] = {'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'};
    const unsigned char suffix[2] = {'I', 'J'};
    unsigned char final_bytes[10] = {0};
    size_t published = 0;
    int descriptor;
    int error;

    PROBE_CHECK(argc == 2);
    PROBE_CHECK(wf_sched_init(&probe_core, 1u, 2u, 256u * 1024u) == 0);
    PROBE_CHECK(wf_completion_runtime_init(&runtime) == 0);
    /* The ring is deliberately shallower than the number of operations this
     * probe puts in flight, so that the submitting call's own kick is what
     * makes room rather than a capacity answer that no longer exists. */
    error = wf_linux_io_uring_init(&adapter, &runtime, 8u, 16u);
    if (error != 0) {
        fprintf(
            stderr,
            "native adapter probe: io_uring qualification unavailable: %s\n",
            strerror(error)
        );
        PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0);
        return 77;
    }
#if WF_IO_OWNER_RINGS
    PROBE_CHECK(probe_two_rings_share_one_epoch() == 0);
#endif
    probe_runtime = &runtime;
    probe_adapter = &adapter;
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

    probe_record_init(&first, WF_FILE_PREAD);
    first.request.operation.pread.descriptor = descriptor;
    first.request.operation.pread.buffer = first_bytes;
    first.request.operation.pread.count = sizeof(first_bytes);
    first.request.operation.pread.offset = 0;
    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, &first)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    probe_record_init(&second, WF_FILE_PREAD);
    second.request.operation.pread.descriptor = descriptor;
    second.request.operation.pread.buffer = second_bytes;
    second.request.operation.pread.count = sizeof(second_bytes);
    second.request.operation.pread.offset = 4;
    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, &second)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );

    while (published < 2) {
        size_t step = 0;
        PROBE_CHECK(
            wf_linux_io_uring_progress(&adapter, 2 - published, 1, &step) == 0
        );
        published += step;
    }
    PROBE_CHECK(probe_record_done(&first) && probe_record_done(&second));
    PROBE_CHECK(first.result.kind == WF_FILE_PREAD);
    PROBE_CHECK(first.result.value == 4 && first.result.error_code == 0);
    PROBE_CHECK(second.result.value == 4 && second.result.error_code == 0);
    PROBE_CHECK(memcmp(first_bytes, seed, 4) == 0);
    PROBE_CHECK(memcmp(second_bytes, seed + 4, 4) == 0);

    probe_record_init(&third, WF_FILE_PWRITE);
    third.request.operation.pwrite.descriptor = descriptor;
    third.request.operation.pwrite.buffer = suffix;
    third.request.operation.pwrite.count = sizeof(suffix);
    third.request.operation.pwrite.offset = (int64_t)sizeof(seed);
    PROBE_CHECK(
        wf_linux_io_uring_submit(&adapter, &third)
        == WF_LINUX_IO_URING_TARGET_OWNS
    );
    published = 0;
    while (published == 0) {
        PROBE_CHECK(
            wf_linux_io_uring_progress(&adapter, 1, 1, &published) == 0
        );
    }
    PROBE_CHECK(probe_record_done(&third));
    PROBE_CHECK(third.result.kind == WF_FILE_PWRITE);
    PROBE_CHECK(third.result.value == 2 && third.result.error_code == 0);
    PROBE_CHECK(pread(descriptor, final_bytes, sizeof(final_bytes), 0) == 10);
    PROBE_CHECK(memcmp(final_bytes, seed, sizeof(seed)) == 0);
    PROBE_CHECK(memcmp(final_bytes + sizeof(seed), suffix, sizeof(suffix)) == 0);

    PROBE_CHECK(
        probe_more_in_flight_than_the_ring_is_deep(&adapter, descriptor) == 0
    );
    PROBE_CHECK(
        probe_completions_past_the_queue_are_flushed_from_the_kernel(
            &adapter,
            descriptor
        ) == 0
    );

    PROBE_CHECK(probe_loopback_round_trip(&adapter) == 0);

    /* An epoch change before sleep performs no explicit host wake while
     * nobody has announced one. The subsequent park observes the epoch and
     * returns without entering epoll. */
    {
        uint64_t epoch = wf_completion_wake_epoch(&runtime);
        uint64_t writes = wf_linux_io_uring_statistics_snapshot(&adapter)
            .host_wake_writes;
        wf_completion_notify_target(&runtime);
        PROBE_CHECK(
            wf_linux_io_uring_statistics_snapshot(&adapter)
                .host_wake_writes == writes
        );
        PROBE_CHECK(
            wf_linux_io_uring_park(&adapter, epoch, UINT32_MAX) == 0
        );
    }

    /* Compute and native CQ facts share one kernel wait set. A runtime
     * notification wakes an announced sleeper through eventfd, and the
     * eventfd is empty again before that sleeper becomes active. */
    {
        probe_park_context parked = {
            .adapter = &adapter,
            .epoch = wf_completion_wake_epoch(&runtime),
            .result = -1,
        };
        pthread_t sleeper;
        uint64_t writes = wf_linux_io_uring_statistics_snapshot(&adapter)
            .host_wake_writes;
        uint64_t counter;
        PROBE_CHECK(
            pthread_create(&sleeper, NULL, probe_park_thread, &parked) == 0
        );
        PROBE_CHECK(probe_wait_until_announced(&runtime) == 0);
        wf_completion_notify_compute(&runtime);
        PROBE_CHECK(pthread_join(sleeper, NULL) == 0);
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
     * eventfd, wakes the sleeper when a delayed CQE appears. */
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

    /* Four lanes wait in place on four distinct records. Completions may race
     * and coalesce into one eventfd level, but no lane may consume that
     * broadcast fact before every already-announced waiter has left epoll. */
    {
        probe_record_wait_context contexts[PROBE_RECORD_WAITERS];
        pthread_t waiters[PROBE_RECORD_WAITERS];
        unsigned index;
        uint64_t writes = wf_linux_io_uring_statistics_snapshot(&adapter)
            .host_wake_writes;
        for (index = 0; index < PROBE_RECORD_WAITERS; ++index) {
            probe_record_init(&contexts[index].record, WF_FILE_PREAD);
            contexts[index].adapter = &adapter;
            contexts[index].runtime = &runtime;
            contexts[index].expected = (int64_t)index + 100;
            contexts[index].result = -1;
            PROBE_CHECK(
                pthread_create(
                    &waiters[index],
                    NULL,
                    probe_wait_for_own_record,
                    &contexts[index]
                ) == 0
            );
        }
        PROBE_CHECK(
            probe_wait_for_parked_count(&runtime, PROBE_RECORD_WAITERS) == 0
        );
        for (index = 0; index < PROBE_RECORD_WAITERS; ++index) {
            contexts[index].record.result.kind = WF_FILE_PREAD;
            contexts[index].record.result.value = contexts[index].expected;
            contexts[index].record.result.error_code = 0;
            wf_completion_record_complete(&contexts[index].record);
            PROBE_CHECK(pthread_join(waiters[index], NULL) == 0);
            PROBE_CHECK(contexts[index].result == 0);
            PROBE_CHECK(
                probe_wait_for_parked_count(
                    &runtime,
                    PROBE_RECORD_WAITERS - index - 1u
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

    PROBE_CHECK(probe_open_and_close_cases(&adapter, argv[1]) == 0);
    PROBE_CHECK(probe_open_names_the_submitters_bytes(&adapter, argv[1]) == 0);

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

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "native_contract.h"
#include "windows_iocp.h"
#include "../sched/core.h"

#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <windows.h>

#define PROBE_CHECK(condition)                                                \
    do {                                                                      \
        if (!(condition)) {                                                   \
            (void)fprintf(                                                    \
                stderr,                                                       \
                "native adapter probe failed: %s:%d: %s\n",                   \
                __FILE__,                                                     \
                __LINE__,                                                     \
                #condition                                                    \
            );                                                                \
            return 1;                                                         \
        }                                                                     \
    } while (0)

/* The bridge's three parts of the runtime, supplied here because this probe
 * links no bridge on purpose, exactly as the Linux arm above supplies them:
 * `wf_completion_record_complete` is the one publication, and the three
 * `wf__sched_host_*` hooks are design section 7's platform item 2 -- one wait
 * and wake primitive -- bound to this probe's own runtime and port. */
static wf_sched_core probe_core;
static wf_completion_runtime *probe_runtime;
static wf_windows_iocp_adapter *probe_adapter;

void wf_completion_record_complete(wf_completion_record *record) {
    wf_sched_complete(&probe_core, &record->sched);
}

int wf__sched_host_epoch(uint64_t *epoch) {
    if (probe_runtime == NULL) {
        return 0;
    }
    *epoch = wf_completion_wake_epoch(probe_runtime);
    return 1;
}

int wf__sched_host_park(uint64_t observed) {
    if (probe_runtime == NULL || probe_adapter == NULL) {
        return 0;
    }
    (void)wf_windows_iocp_park(probe_adapter, observed, UINT32_MAX);
    return 1;
}

int wf__sched_host_wake(void) {
    if (probe_runtime == NULL) {
        return 0;
    }
    wf_completion_notify_target(probe_runtime);
    return 1;
}

static unsigned probe_state(const wf_completion_record *record) {
    return __atomic_load_n(&record->sched.state, __ATOMIC_ACQUIRE);
}

int main(int argc, char **argv) {
    wf_completion_runtime runtime;
    wf_windows_iocp_adapter adapter;
    wf_completion_record record;
    wf_completion_target_contract contract = wf_completion_target_contract_for(
        WF_TARGET_WINDOWS_IOCP
    );
    const unsigned char written[4] = {'i', 'o', 'c', 'p'};
    unsigned char read_back[4];
    HANDLE handle;
    DWORD transferred = 0;
    WCHAR path[MAX_PATH];
    int index;

    if (argc != 2) {
        return 2;
    }
    PROBE_CHECK(contract.implemented == 1);
    PROBE_CHECK(contract.native_completion == 1);
    PROBE_CHECK(contract.may_use_blocking_helpers == 0);
    PROBE_CHECK(contract.supports_scheduler_progress == 1);

    for (index = 0; index < MAX_PATH && argv[1][index] != 0; ++index) {
        path[index] = (WCHAR)(unsigned char)argv[1][index];
    }
    PROBE_CHECK(index < MAX_PATH);
    path[index] = 0;

    PROBE_CHECK(wf_completion_runtime_init(&runtime) == 0);
    probe_runtime = &runtime;
    PROBE_CHECK(wf_windows_iocp_init(&adapter, &runtime, 0) == 0);
    probe_adapter = &adapter;
    PROBE_CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            wf_windows_iocp_notify,
            &adapter
        ) == 0
    );

    /* The fixture is written first, through a synchronous handle of its own,
     * and only then opened for overlapped reading: the handle the port takes
     * is the one this probe submits on. */
    {
        HANDLE writer = CreateFileW(
            path,
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            NULL,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            NULL
        );
        PROBE_CHECK(writer != INVALID_HANDLE_VALUE);
        PROBE_CHECK(
            WriteFile(writer, written, sizeof(written), &transferred, NULL)
                != FALSE
            && transferred == (DWORD)sizeof(written)
        );
        PROBE_CHECK(FlushFileBuffers(writer) != FALSE);
        PROBE_CHECK(CloseHandle(writer) != FALSE);
    }
    handle = CreateFileW(
        path,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    PROBE_CHECK(handle != INVALID_HANDLE_VALUE);
    PROBE_CHECK(wf_windows_iocp_associate(&adapter, handle) == 0);

    memset(&record, 0, sizeof(record));
    memset(read_back, 0, sizeof(read_back));
    wf_sched_record_init(&record.sched);
    record.request.kind = WF_FILE_PREAD;
    record.request.operation.pread.descriptor = 0;
    record.request.operation.pread.buffer = read_back;
    record.request.operation.pread.count = sizeof(read_back);
    record.request.operation.pread.offset = 0;
    record.opened_descriptor = -1;
    PROBE_CHECK(wf_windows_iocp_carries(&record) != 0);
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, &record, handle)
            == WF_WINDOWS_IOCP_TARGET_OWNS
    );

    /* The in-place wait: read the record, make one non-blocking pass, and only
     * then sleep on the port.  A read the kernel answered synchronously is
     * already published by the submit above and never reaches the park. */
    for (;;) {
        uint64_t epoch;
        size_t published = 0;
        if (probe_state(&record) == WF_SCHED_DONE) {
            break;
        }
        epoch = wf_completion_wake_epoch(&runtime);
        PROBE_CHECK(wf_windows_iocp_progress(&adapter, 4u, &published) == 0);
        if (published != 0 || probe_state(&record) == WF_SCHED_DONE) {
            continue;
        }
        PROBE_CHECK(wf_windows_iocp_park(&adapter, epoch, UINT32_MAX) == 0);
    }

    PROBE_CHECK(record.result.kind == WF_FILE_PREAD);
    PROBE_CHECK(record.result.error_code == 0);
    PROBE_CHECK(record.result.value == (int64_t)sizeof(written));
    PROBE_CHECK(memcmp(read_back, written, sizeof(written)) == 0);
    PROBE_CHECK(wf_windows_iocp_in_flight(&adapter) == 0);
    PROBE_CHECK(wf_windows_iocp_progress_error(&adapter) == 0);
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == 0);
    probe_adapter = NULL;
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    probe_runtime = NULL;
    PROBE_CHECK(CloseHandle(handle) != FALSE);
    (void)printf("native-adapter-probe target=windows-iocp status=pass\n");
    return 0;
}

#else

int main(void) {
    return 77;
}

#endif
