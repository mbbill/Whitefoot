#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* F_NOCACHE, the Darwin half of the WF_IO_NOCACHE target policy, is a BSD
 * extension the POSIX macro above hides, exactly as in bridge.c. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

#include "file_adapter.h"

#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <poll.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#if !defined(WF_COMPLETION_POLL)
#define WF_COMPLETION_POLL poll
#else
extern int WF_COMPLETION_POLL(struct pollfd *, nfds_t, int);
#endif

#if !defined(WF_COMPLETION_PREAD)
#define WF_COMPLETION_PREAD pread
#else
extern ssize_t WF_COMPLETION_PREAD(int, void *, size_t, off_t);
#endif

/* The one host call an adapter open makes, named so a build may observe it.
 * The observing build is expected to perform the same host call; nothing else
 * about the open changes, and the default build calls `openat` directly.
 *
 * Retire-and-retry is decided by what this call answers, and the interesting
 * schedules are the ones where a second engine is mid-operation at exactly the
 * moment it answers `EMFILE`.  Naming the call is what lets a test stand at
 * that moment instead of guessing at it with a delay. */
#if !defined(WF_FILE_OPENAT)
#define WF_FILE_OPENAT openat
#else
extern int WF_FILE_OPENAT(int, const char *, int, ...);
#endif

#if defined(__APPLE__)
/* libSystem exports the exact facility used by the qualified Darwin target;
 * it is intentionally not replaced with an opendir/readdir loop. */
#if !defined(WF_COMPLETION_GETDIRENTRIES64)
#define WF_COMPLETION_GETDIRENTRIES64 __getdirentries64
#endif
extern ssize_t WF_COMPLETION_GETDIRENTRIES64(
    int,
    void *,
    size_t,
    int64_t *
);
#elif defined(__linux__)
/* glibc exports the exact facility used by the qualified Linux target.  It is
 * declared here rather than reached through <dirent.h> for the same reason
 * the Darwin entry above is: the declaration is behind _GNU_SOURCE, which
 * this unit does not ask for, and the prototype is fixed by the ABI.  It is
 * intentionally not replaced with an opendir/readdir loop, which is a scan
 * built out of other operations [QUAL-2, QUAL-3]. */
#if !defined(WF_COMPLETION_GETDENTS64)
#define WF_COMPLETION_GETDENTS64 getdents64
#endif
extern ssize_t WF_COMPLETION_GETDENTS64(int, void *, size_t);
#endif

_Static_assert(
    sizeof(struct stat) <= WF_FILE_STATUS_CAPACITY,
    "WF_FILE_STATUS_CAPACITY must hold the host stat record"
);
_Static_assert(
    sizeof(wf_file_result) <= WF_COMPLETION_RESULT_CAPACITY,
    "completion result cell must hold a file result"
);

static int wf_file_request_valid(const wf_file_request *request) {
    if (request == NULL) {
        return 0;
    }
    switch (request->kind) {
    case WF_FILE_OPEN_AT:
        return request->operation.open_at.path != NULL
            && request->operation.open_at.expected_kind
                <= WF_FILE_EXPECT_DIRECTORY;
    case WF_FILE_READ:
        return request->operation.read.buffer != NULL
            || request->operation.read.count == 0;
    case WF_FILE_WRITE:
        return request->operation.write.buffer != NULL
            || request->operation.write.count == 0;
    case WF_FILE_PREAD:
        return request->operation.pread.buffer != NULL
            || request->operation.pread.count == 0;
    case WF_FILE_PWRITE:
        return request->operation.pwrite.buffer != NULL
            || request->operation.pwrite.count == 0;
    case WF_FILE_STATUS:
    case WF_FILE_CLOSE:
        return 1;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        /* The position cell is required on both families even though only
         * Darwin's facility writes it: one validity rule keeps a request
         * record that is well formed on one host well formed on the other. */
        return (request->operation.directory_next.buffer != NULL
                || request->operation.directory_next.count == 0)
            && request->operation.directory_next.position != NULL;
#endif
    default:
        return 0;
    }
}

static wf_file_result wf_file_execute_once(const wf_file_request *request) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.kind = request->kind;
    result.value = -1;

    switch (request->kind) {
    case WF_FILE_READ:
        if (request->operation.read.count > (size_t)SSIZE_MAX) {
            result.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_WRITE:
        if (request->operation.write.count > (size_t)SSIZE_MAX) {
            result.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PREAD:
        /* An empty transfer has no external action.  This must be decided
         * before descriptor and offset validation reaches the host: callers
         * rely on zero length completing without touching
         * either the destination or the file facility. */
        if (request->operation.pread.count == 0) {
            result.value = 0;
            return result;
        }
        if (request->operation.pread.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pread.offset
                != request->operation.pread.offset) {
            result.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PWRITE:
        if (request->operation.pwrite.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pwrite.offset
                != request->operation.pwrite.offset) {
            result.error_code = EINVAL;
            return result;
        }
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        if (request->operation.directory_next.count > (size_t)SSIZE_MAX) {
            result.error_code = EINVAL;
            return result;
        }
        break;
#endif
    default:
        break;
    }

    /* Each pass is one qualified host attempt. No-progress interruption and
     * readiness refusal are absorbed by wf_file_execute_direct below and
     * never become writer-visible outcomes. */
    switch (request->kind) {
    case WF_FILE_OPEN_AT:
        if (request->operation.open_at.expected_kind
            > WF_FILE_EXPECT_DIRECTORY) {
            result.error_code = EINVAL;
            result.open_outcome = WF_FILE_OPEN_FAILED;
            return result;
        }
        if (request->operation.open_at.has_mode != 0) {
            result.value = WF_FILE_OPENAT(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      ),
                (mode_t)request->operation.open_at.mode
            );
        } else {
            result.value = WF_FILE_OPENAT(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      )
            );
        }
        if (result.value < 0) {
            result.open_outcome = WF_FILE_OPEN_FAILED;
            break;
        }
        if (request->operation.open_at.expected_kind != WF_FILE_EXPECT_ANY) {
            struct stat status;
            int descriptor = (int)result.value;
            if (fstat(descriptor, &status) != 0) {
                int status_error = errno;
                (void)close(descriptor);
                result.error_code = status_error;
                result.open_outcome = WF_FILE_OPEN_STATUS_FAILED;
                return result;
            }
            result.open_outcome = wf_file_kind_outcome(
                request->operation.open_at.expected_kind,
                (unsigned int)status.st_mode
            );
            if (result.open_outcome != WF_FILE_OPEN_SUCCEEDED) {
                (void)close(descriptor);
                return result;
            }
        }
        /* The descriptor is the one this open hands back: the kind check has
         * either passed or was not asked for. WF_IO_NOCACHE is applied here,
         * once, so a refused descriptor never receives it. */
        wf_file_apply_uncached_reads((int)result.value);
        break;
    case WF_FILE_READ:
        result.value = read(
            request->operation.read.descriptor,
            request->operation.read.buffer,
            request->operation.read.count
        );
        break;
    case WF_FILE_WRITE:
        result.value = write(
            request->operation.write.descriptor,
            request->operation.write.buffer,
            request->operation.write.count
        );
        break;
    case WF_FILE_PREAD:
        result.value = WF_COMPLETION_PREAD(
            request->operation.pread.descriptor,
            request->operation.pread.buffer,
            request->operation.pread.count,
            (off_t)request->operation.pread.offset
        );
        break;
    case WF_FILE_PWRITE:
        result.value = pwrite(
            request->operation.pwrite.descriptor,
            request->operation.pwrite.buffer,
            request->operation.pwrite.count,
            (off_t)request->operation.pwrite.offset
        );
        break;
    case WF_FILE_STATUS: {
        struct stat status;
        result.value = fstat(request->operation.status.descriptor, &status);
        if (result.value == 0) {
            result.status_size = sizeof(status);
            memcpy(result.status, &status, sizeof(status));
        }
        break;
    }
    case WF_FILE_CLOSE:
        result.value = close(request->operation.close.descriptor);
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
#if defined(__APPLE__)
        result.value = WF_COMPLETION_GETDIRENTRIES64(
            request->operation.directory_next.descriptor,
            request->operation.directory_next.buffer,
            request->operation.directory_next.count,
            request->operation.directory_next.position
        );
#else
        /* Linux keeps the whole enumeration cursor in the descriptor, so the
         * facility takes no base-position argument and this cell is left as
         * the caller gave it. */
        result.value = WF_COMPLETION_GETDENTS64(
            request->operation.directory_next.descriptor,
            request->operation.directory_next.buffer,
            request->operation.directory_next.count
        );
#endif
        break;
#endif
    default:
        result.error_code = EINVAL;
        return result;
    }
    if (result.value < 0) {
        result.error_code = errno;
    }
    return result;
}

static int wf_file_wait_ready(const wf_file_request *request) {
    struct pollfd descriptor;
    int waited;
    memset(&descriptor, 0, sizeof(descriptor));
    switch (request->kind) {
    case WF_FILE_READ:
        descriptor.fd = request->operation.read.descriptor;
        descriptor.events = POLLIN;
        break;
    case WF_FILE_PREAD:
        descriptor.fd = request->operation.pread.descriptor;
        descriptor.events = POLLIN;
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        descriptor.fd = request->operation.directory_next.descriptor;
        descriptor.events = POLLIN;
        break;
#endif
    case WF_FILE_WRITE:
        descriptor.fd = request->operation.write.descriptor;
        descriptor.events = POLLOUT;
        break;
    case WF_FILE_PWRITE:
        descriptor.fd = request->operation.pwrite.descriptor;
        descriptor.events = POLLOUT;
        break;
    default:
        return EINVAL;
    }
    do {
        waited = WF_COMPLETION_POLL(&descriptor, 1, -1);
    } while (waited < 0 && errno == EINTR);
    return waited > 0 ? 0 : errno;
}

wf_file_result wf_file_execute_direct(const wf_file_request *request) {
    wf_file_result result;
    if (!wf_file_request_valid(request)) {
        memset(&result, 0, sizeof(result));
        result.kind = request == NULL ? 0 : request->kind;
        result.value = -1;
        result.error_code = EINVAL;
        return result;
    }
    for (;;) {
        int readiness_error;
        result = wf_file_execute_once(request);
        if (result.value >= 0 || result.error_code == 0) {
            return result;
        }
        switch (request->kind) {
        case WF_FILE_READ:
        case WF_FILE_WRITE:
        case WF_FILE_PREAD:
        case WF_FILE_PWRITE:
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
        case WF_FILE_DIRECTORY_NEXT:
#endif
            break;
        default:
            return result;
        }
        if (result.error_code == EINTR) {
            continue;
        }
        if (result.error_code != EAGAIN
#if defined(EWOULDBLOCK) && EWOULDBLOCK != EAGAIN
            && result.error_code != EWOULDBLOCK
#endif
        ) {
            return result;
        }
        readiness_error = wf_file_wait_ready(request);
        if (readiness_error != 0) {
            result.error_code = readiness_error;
            return result;
        }
    }
}

/* The measured host-call time above which handing an operation to another
 * thread is worth the handoff.
 *
 * Below it the operation does not wait: the host answers from memory, and a
 * helper can only add a queue crossing and two thread wakes to work that was
 * already done by the time the wake arrived.  This is the number that decides
 * whether the pool grows at all, and it is a measurement of the program's own
 * operations rather than an assumption about the host.
 *
 * Twenty microseconds is about ten times what one handoff costs on the host
 * this was measured on — a wake on the submitting thread, a wake-up on the
 * helper, and a queue crossing.  Sizing it at the handoff itself would admit
 * every operation whose overlap merely breaks even, and a pool that breaks
 * even still costs the CPU it spends.  Every population this project has
 * measured falls clearly on one side: a warm 4 KiB read is about 1 us, a warm
 * 64 KiB read 5 to 7, and a read that reaches the device 130 or more. */
#define WF_FILE_OVERLAP_WAIT_NS 20000u

/* How often an execution is timed for the average above: one in this many.
 * The clock is read twice per sampled execution and not at all otherwise. */
#define WF_FILE_EXECUTE_SAMPLE_INTERVAL 16u

/* The clock the helper policy measures host calls with.
 *
 * It is a named seam of the same class as WF_COMPLETION_PREAD and
 * WF_COMPLETION_POLL: a build may answer it with a scripted clock so that the
 * growth rule can be tested for what it decides rather than for how fast the
 * machine running the test happened to be.  The shipped build reads the host
 * monotonic clock and nothing else. */
#if !defined(WF_FILE_MONOTONIC_NS)
static uint64_t wf_file_monotonic_ns_host(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return 0;
    }
    return (uint64_t)now.tv_sec * 1000000000u + (uint64_t)now.tv_nsec;
}
#define WF_FILE_MONOTONIC_NS wf_file_monotonic_ns_host
#else
extern uint64_t WF_FILE_MONOTONIC_NS(void);
#endif

static uint64_t wf_file_monotonic_ns(void) {
    return WF_FILE_MONOTONIC_NS();
}

/* Records one host call's duration into the smoothed average, from one in
 * every WF_FILE_EXECUTE_SAMPLE_INTERVAL executions.  Two threads that sample
 * at once may lose one another's contribution; the average is a policy hint
 * and no operation's outcome depends on it. */
static void wf_file_note_execution(wf_file_adapter *adapter, uint64_t started) {
    uint64_t mean;
    uint64_t sample = wf_file_monotonic_ns();
    sample = sample > started ? sample - started : 0u;
    mean = atomic_load_explicit(&adapter->mean_execute_ns, memory_order_relaxed);
    mean = mean == 0 ? sample : mean - mean / 4u + sample / 4u;
    atomic_store_explicit(&adapter->mean_execute_ns, mean, memory_order_relaxed);
}

static int wf_file_execution_should_be_timed(wf_file_adapter *adapter) {
    uint64_t tick = atomic_fetch_add_explicit(
        &adapter->execute_ticks,
        1,
        memory_order_relaxed
    );
    return tick % WF_FILE_EXECUTE_SAMPLE_INTERVAL == 0;
}

/* Moves one queue entry into the executing thread's own storage.
 *
 * The record is copied rather than executed in place because the queue slot
 * becomes free the moment the lock is released.  Only an open needs its path
 * bytes, and only an open pays for them: copying the whole record moved a
 * kilobyte of path storage for every read and write as well, inside the one
 * lock every submission and every execution has to take. */
static void wf_file_copy_out(wf_file_work *work, const wf_file_work *entry) {
    work->token = entry->token;
    work->request = entry->request;
    if (entry->request.kind == WF_FILE_OPEN_AT) {
        (void)wf_file_stage_path(work->path_storage, entry->path_storage);
        wf_file_work_bind_path(work);
    }
}

/* Takes one queued request off this adapter's queue.
 *
 * `from_head` chooses which end.  A scheduler visit takes the newest request,
 * so a blocked earlier host facility cannot hide every later free operation
 * behind it, and helper threads take the oldest, so several target contexts
 * spread across both ends of the bounded queue.  Work this adapter still owes,
 * run on the way to re-attempting a refused open, is taken from the head
 * whichever thread does it: that is the order the program wrote, and it is the
 * order that reproduces the sequential outcome. */
static int wf_file_take_work(
    wf_file_adapter *adapter,
    wf_file_work *work,
    int from_head
) {
    int present = 0;
    int released_capacity = 0;
    (void)pthread_mutex_lock(&adapter->queue_lock);
    if (adapter->queue_count != 0) {
        released_capacity = adapter->queue_count == adapter->queue_capacity;
        if (from_head != 0) {
            wf_file_copy_out(work, &adapter->queue[adapter->queue_head]);
            adapter->queue_head =
                (adapter->queue_head + 1) % adapter->queue_capacity;
        } else {
            adapter->queue_tail = adapter->queue_tail == 0
                ? adapter->queue_capacity - 1
                : adapter->queue_tail - 1;
            wf_file_copy_out(work, &adapter->queue[adapter->queue_tail]);
        }
        adapter->queue_count -= 1;
        present = 1;
    }
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    if (released_capacity != 0) {
        wf_completion_notify_capacity(adapter->runtime);
    }
    return present;
}

/* Whether this adapter's record has been published, read the way a thread
 * that ran no initialization of its own has to read it.
 *
 * The direct execution route reaches `wf_file_execute_timed` straight from a
 * generated call, with no once-control of its own, while another thread may
 * be inside `wf_file_adapter_init`.  Pairing this acquire load with the
 * release store init ends on is what makes the rest of the record safe to
 * read for a thread that finds it set, and makes the read itself a
 * synchronizing one rather than a race. */
static int wf_file_adapter_initialized(const wf_file_adapter *adapter) {
    return adapter != NULL
        && atomic_load_explicit(&adapter->initialized, memory_order_acquire)
            != 0;
}

/* Records that one operation this adapter owned has finished its host work.
 *
 * A refused open asks the process-wide ledger, not this adapter, whether a
 * descriptor came back, so every operation reports there as well as into this
 * adapter's own execution counts.  That is the whole of what makes the
 * exhaustion rule one rule: an open refused on the kernel ring can see a read
 * finishing on a helper thread here, and this open can see that ring's close.
 *
 * The report says both things the rule asks for: that this operation has ended,
 * and whether its ending put a descriptor back.  A read finishing here ends
 * without returning one, so it is not a reason for an open refused on the ring
 * to spend its single re-attempt.
 */
static void wf_file_finish_execution(
    wf_file_adapter *adapter,
    int helper,
    const wf_file_result *result
) {
    atomic_fetch_add_explicit(
        helper != 0 ? &adapter->stat_helper_executions
                    : &adapter->stat_scheduler_executions,
        1,
        memory_order_relaxed
    );
    wf_completion_operation_retired(wf_file_returned_a_descriptor(result));
    /* A held open is released by whichever thread next asks the ledger, and
     * on a target with a kernel ring that thread is a scheduler parked on the
     * completion endpoint.  The publication above already woke it, but it may
     * have asked and found nothing in the instant before the line above, so
     * the retirement gets a wake of its own.  Only where something is
     * actually waiting: on the ordinary path this is one atomic load. */
    if (wf_completion_retirement_waiters() != 0) {
        wf_completion_notify_capacity(adapter->runtime);
    }
}

/* True exactly for the results of an open the host refused because it had no
 * descriptor to give: the process limit and the system limit.  Both are
 * [SYS-7] `ResourceExhausted`, and both are recoverable by giving a
 * descriptor back. */
static int wf_file_open_lacked_a_descriptor(const wf_file_result *result) {
    return result->kind == WF_FILE_OPEN_AT && result->value < 0
        && (result->error_code == EMFILE || result->error_code == ENFILE);
}

wf_file_result wf_file_execute_timed(
    wf_file_adapter *adapter,
    const wf_file_request *request
) {
    int timed;
    uint64_t started;
    wf_file_result result;
    if (!wf_file_adapter_initialized(adapter)) {
        return wf_file_execute_direct(request);
    }
    timed = wf_file_execution_should_be_timed(adapter);
    started = timed ? wf_file_monotonic_ns() : 0u;
    result = wf_file_execute_direct(request);
    if (timed) {
        wf_file_note_execution(adapter, started);
    }
    return result;
}

enum wf_file_wait_verdict wf_file_adapter_wait_verdict(
    const wf_file_adapter *adapter
) {
    uint64_t mean;
    if (!wf_file_adapter_initialized(adapter)) {
        return WF_FILE_WAIT_UNMEASURED;
    }
    mean = atomic_load_explicit(
        &adapter->mean_execute_ns,
        memory_order_relaxed
    );
    if (mean == 0) {
        return WF_FILE_WAIT_UNMEASURED;
    }
    return mean >= WF_FILE_OVERLAP_WAIT_NS ? WF_FILE_WAIT_LONG
                                           : WF_FILE_WAIT_SHORT;
}

int wf_file_adapter_transfer_runs_on_caller(const wf_file_adapter *adapter) {
    if (wf_file_adapter_wait_verdict(adapter) != WF_FILE_WAIT_SHORT
        || wf_file_adapter_helper_count(adapter) != 0) {
        return 0;
    }
    return wf_file_adapter_queued(adapter) == 0;
}

static void wf_file_run_work(
    wf_file_adapter *adapter,
    const wf_file_work *work,
    int helper,
    int may_run_owed_work
);

/* The queue as the retirement ledger asks for it, while the ledger holds its
 * own lock.  It takes no lock, which is the whole requirement the ledger puts
 * on this callback. */
static size_t wf_file_adapter_owed_work(void *context) {
    return wf_file_adapter_queued((const wf_file_adapter *)context);
}

/* Gives back what this runtime is still holding, then re-attempts one open the
 * host refused for want of a descriptor.
 *
 * This is one half of the rule stated in contract.h, on the route a bounded
 * helper pool carries.  What this thread can give back has two parts and they
 * are the same fact seen twice.  The first is work this adapter has accepted
 * and nobody has started: running it is work the sequential execution performs
 * anyway, it publishes each outcome unchanged, and a close among them returns
 * a descriptor.  The second is every operation in flight anywhere else — on
 * another helper, on the kernel ring, inside a blocking direct call — which
 * this thread cannot run but can wait for.  An adapter that only drained its
 * own queue would publish a refusal while its own descriptors were still in
 * hand, which is exactly what a four-helper configuration does to three
 * simultaneous opens: each is taken by a different thread and each looks at an
 * empty queue.
 *
 * The waiter is registered before any owed work runs, so a second open refused
 * while this thread is running that work counts this one out of "in flight
 * elsewhere" instead of waiting for it.  Owed work runs with
 * `may_run_owed_work` cleared: it may wait for a retirement like any other
 * refused open, but it may not run the queue in its turn, because the caller
 * suspended inside it still holds that duty — which is why it passes the queue
 * as its own.
 *
 * Either way the queue is the waiter's `owed`, and the ledger reads it at the
 * moment it decides rather than being handed a reading taken before it: a
 * thread that will run that queue must not sleep while an item is in it, and a
 * thread whose caller is suspended inside it must not wait for one.
 *
 * Exactly one re-attempt, counted exactly where it is made, and made at the
 * moment the answer can no longer improve: a descriptor has come back, or
 * nothing is left in this runtime that could bring one.  The second reason is
 * an attempt too, because a descriptor can come back from outside this runtime
 * — a thread of the program's own closing one while this runtime carries the
 * read that thread is answering — and no ledger can see that.  If the attempt
 * also fails, that is the outcome source-order execution produces and the
 * program is entitled to see it. */
static wf_file_result wf_file_retire_and_retry(
    wf_file_adapter *adapter,
    const wf_file_work *work,
    int helper,
    uint64_t seen,
    int may_run_owed_work
) {
    wf_retirement_waiter waiter;
    wf_completion_retirement_wait_begin(
        &waiter,
        seen,
        wf_file_adapter_owed_work,
        adapter,
        may_run_owed_work
    );
    for (;;) {
        if (may_run_owed_work != 0) {
            wf_file_work owed;
            /* Run the queue before every decision and after every wake, not
             * once: this thread is one of this adapter's engines, and work
             * queued while it waits is work only an engine can retire.  This
             * waiter stands aside for each item it runs and no longer, so an
             * open among them that is refused in its turn can answer instead of
             * waiting behind the thread that is running it.
             *
             * It stands aside; it does not leave.  Leaving and registering
             * again puts this open at the back of the order every time it runs
             * the work it owes, which hands a returned descriptor — and with it
             * the program's one `Ok` — to an open that asked later: on a
             * scripted two-waiter shape, to the later one in 200 runs of
             * 200. */
            while (wf_file_take_work(adapter, &owed, 1)) {
                wf_completion_retirement_defer_begin(&waiter);
                wf_file_run_work(adapter, &owed, helper, 0);
                wf_completion_retirement_defer_end(&waiter);
            }
        }
        if (wf_completion_retirement_state(&waiter) != WF_RETIREMENT_AWAITED) {
            break;
        }
        wf_completion_retirement_sleep(&waiter);
    }
    wf_completion_retirement_wait_end(&waiter);
    /* Either a descriptor came back, or nothing is left in this runtime that
     * could bring one and this is the last moment a second attempt could see
     * more than the first — including a descriptor a thread of the program's
     * own gave back while this one waited.  One attempt, here, either way. */
    atomic_fetch_add_explicit(
        &adapter->stat_exhaustion_retries,
        1,
        memory_order_relaxed
    );
    return wf_file_execute_timed(adapter, &work->request);
}

static void wf_file_run_work(
    wf_file_adapter *adapter,
    const wf_file_work *work,
    int helper,
    int may_run_owed_work
) {
    /* Read before the host attempt, because that is the moment the answer is
     * about: a descriptor returned while this one is inside `openat` is one
     * the attempt could not use, and a refusal decided by the state afterwards
     * would miss it. */
    uint64_t seen = wf_completion_descriptor_returns();
    wf_file_result result = wf_file_execute_timed(adapter, &work->request);
    wf_completion_publication publication;
    if (wf_file_open_lacked_a_descriptor(&result)) {
        result = wf_file_retire_and_retry(
            adapter,
            work,
            helper,
            seen,
            may_run_owed_work
        );
    }
    publication = (wf_completion_publication) {
        .milestones = WF_COMPLETION_OWNERSHIP_COMPLETE,
        .terminal_kind = result.error_code == 0
                && result.open_outcome == WF_FILE_OPEN_SUCCEEDED
            ? 1u
            : 2u,
        .result = &result,
        .result_size = sizeof(result),
    };
    enum wf_completion_publish_result published = wf_completion_publish_terminal(
        adapter->runtime,
        work->token,
        &publication
    );
    wf_file_finish_execution(adapter, helper, &result);
    if (published != WF_COMPLETION_PUBLISHED) {
        /* A legitimate accepted work item owns the unique terminal route.  A
         * failure here records an adapter/core defect; it never invokes writer
         * code or attempts a second publication. */
        atomic_fetch_add_explicit(
            &adapter->stat_publication_failures,
            1,
            memory_order_relaxed
        );
    }
}

static void *wf_file_helper_main(void *context) {
    wf_file_adapter *adapter = context;
    for (;;) {
        wf_file_work work;
        int released_capacity;
        (void)pthread_mutex_lock(&adapter->queue_lock);
        while (adapter->queue_count == 0 && adapter->stopping == 0) {
            adapter->blocked_helpers += 1;
            (void)pthread_cond_wait(
                &adapter->queue_available,
                &adapter->queue_lock
            );
            adapter->blocked_helpers -= 1;
        }
        if (adapter->queue_count == 0 && adapter->stopping != 0) {
            (void)pthread_mutex_unlock(&adapter->queue_lock);
            return NULL;
        }
        released_capacity = adapter->queue_count == adapter->queue_capacity;
        wf_file_copy_out(&work, &adapter->queue[adapter->queue_head]);
        adapter->queue_head = (adapter->queue_head + 1) % adapter->queue_capacity;
        adapter->queue_count -= 1;
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        if (released_capacity != 0) {
            wf_completion_notify_capacity(adapter->runtime);
        }
        wf_file_run_work(adapter, &work, 1, 1);
    }
}

int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_file_work *queue_storage,
    size_t queue_capacity,
    pthread_t *helper_storage,
    size_t helper_capacity,
    size_t helper_count
) {
    size_t created = 0;
    int error;

    if (adapter == NULL || runtime == NULL || queue_storage == NULL
        || queue_capacity == 0 || helper_count > helper_capacity
        || (helper_capacity != 0 && helper_storage == NULL)) {
        return EINVAL;
    }
    /* Cleared before anything else is written, so that a record left half
     * built by a failure below is never mistaken for a usable one, and set
     * again only when every field is in place.
     *
     * That pair of stores is what makes the record safe to read, and it is the
     * only thing that does.  The field-by-field assignment below is not:
     * `atomic_init` is a plain write by definition, and adjacent plain writes
     * may be merged.  At -O2 on aarch64 clang merges these -- one 16-byte
     * store covering `mean_execute_ns` and `execute_ticks`, two more covering
     * the four statistics counters -- so a `memset` here would differ in
     * nothing that matters.  What excludes the race is that no reader touches
     * any of these fields without first passing
     * `wf_file_adapter_initialized`, whose acquire load pairs with the release
     * store this function ends on.
     *
     * A reader can therefore see one of these writes only if the record it
     * already holds is initialized *again* underneath it, which this adapter's
     * contract does not allow and the bridge never does: it initializes once,
     * under a `pthread_once`, with the flag still zero.  A probe that breaks
     * that contract on purpose -- readers running across repeated
     * init/shutdown cycles -- does draw a ThreadSanitizer report, between
     * `atomic_init(&adapter->mean_execute_ns, 0)` below and the acquire load
     * of that same field in `wf_file_adapter_wait_verdict`.  What that report
     * names is the re-initialization, not this line. */
    atomic_store_explicit(&adapter->initialized, 0, memory_order_release);
    adapter->runtime = runtime;
    adapter->queue = queue_storage;
    adapter->queue_capacity = queue_capacity;
    adapter->queue_head = 0;
    adapter->queue_tail = 0;
    adapter->queue_count = 0;
    adapter->helpers = helper_storage;
    atomic_init(&adapter->helper_count, 0);
    atomic_init(&adapter->mean_execute_ns, 0);
    atomic_init(&adapter->execute_ticks, 0);
    adapter->blocked_helpers = 0;
    adapter->stopping = 0;
    adapter->helper_capacity = helper_capacity;
    adapter->helper_cap = helper_count;
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_exhaustion_retries, 0);
    atomic_init(&adapter->stat_capacity_waits, 0);
    atomic_init(&adapter->stat_helper_executions, 0);
    atomic_init(&adapter->stat_scheduler_executions, 0);
    atomic_init(&adapter->stat_publication_failures, 0);

    error = pthread_mutex_init(&adapter->queue_lock, NULL);
    if (error != 0) {
        return error;
    }
    error = pthread_cond_init(&adapter->queue_available, NULL);
    if (error != 0) {
        (void)pthread_mutex_destroy(&adapter->queue_lock);
        return error;
    }
    /* Every field above is published by this one store, and nothing reads the
     * record without the acquire load that pairs with it. */
    atomic_store_explicit(&adapter->initialized, 1, memory_order_release);

    for (created = 0; created < helper_count; ++created) {
        error = pthread_create(
            &helper_storage[created],
            NULL,
            wf_file_helper_main,
            adapter
        );
        if (error != 0) {
            size_t join;
            (void)pthread_mutex_lock(&adapter->queue_lock);
            adapter->stopping = 1;
            (void)pthread_cond_broadcast(&adapter->queue_available);
            (void)pthread_mutex_unlock(&adapter->queue_lock);
            for (join = 0; join < created; ++join) {
                (void)pthread_join(helper_storage[join], NULL);
            }
            (void)pthread_cond_destroy(&adapter->queue_available);
            (void)pthread_mutex_destroy(&adapter->queue_lock);
            atomic_store_explicit(
                &adapter->initialized,
                0,
                memory_order_release
            );
            return error;
        }
        atomic_store_explicit(
            &adapter->helper_count,
            created + 1,
            memory_order_release
        );
    }
    return 0;
}

int wf_file_adapter_set_helper_cap(wf_file_adapter *adapter, size_t cap) {
    if (!wf_file_adapter_initialized(adapter)
        || (cap != 0 && adapter->helpers == NULL)) {
        return EINVAL;
    }
    /* The policy asks for a ceiling; the caller's storage decides how much of
     * it exists.  `wf_file_grow_helpers_locked` writes `helpers[held]`, so a
     * cap the array cannot hold is a write past its end. */
    if (cap > adapter->helper_capacity) {
        cap = adapter->helper_capacity;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    adapter->helper_cap = cap;
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    return 0;
}

size_t wf_file_adapter_helper_count(const wf_file_adapter *adapter) {
    if (!wf_file_adapter_initialized(adapter)) {
        return 0;
    }
    return atomic_load_explicit(&adapter->helper_count, memory_order_acquire);
}

/* Adds one helper when the queue holds more accepted requests than there are
 * helpers to take them **and** this program's operations wait long enough for
 * another thread to be worth its handoff.
 *
 * Queue depth alone is not enough, and measuring it was what made this pool
 * cost more than it saved.  Depth says a program stated width; it does not say
 * the width is over anything.  A program reading a warm page cache states the
 * same eight independent reads as one reading a device and exposes the same
 * queue, but each of its reads is a memory copy that is finished before a
 * woken helper is scheduled — so every helper the depth rule added there
 * bought a wake and a queue crossing and overlapped nothing.  The measured
 * host-call time is the missing half of the condition, and it is a
 * measurement of the program's own operations rather than a guess about the
 * host.
 *
 * A rule that instead grew whenever a submission found no helper *waiting*
 * was tried and measured worse: a helper that has been signalled but not yet
 * scheduled still counts as waiting, so a run of consecutive submissions sees
 * one available helper every time and the pool never grows at all. On the
 * quiet macOS host that left the four-wide program at 919 ms against 625 ms
 * for the depth rule.  Queue depth is a lagging signal, but it is a true one.
 *
 * Growth starts from no helpers at all, because a program whose operations do
 * not wait wants none: with an empty pool the waiting scheduler is the queue's
 * engine and the completion path costs a queue crossing and nothing else.  It
 * runs on the submitting thread with the queue lock already held, so at most
 * one helper appears per submission and the count never passes the policy's
 * cap. A pinned helper policy sets the cap equal to the initial count, making
 * this a no-op. */
static void wf_file_grow_helpers_locked(wf_file_adapter *adapter) {
    size_t held = atomic_load_explicit(
        &adapter->helper_count,
        memory_order_relaxed
    );
    if (held >= adapter->helper_cap
        || adapter->queue_count <= held) {
        return;
    }
    if (wf_file_adapter_wait_verdict(adapter) != WF_FILE_WAIT_LONG) {
        return;
    }
    if (pthread_create(
            &adapter->helpers[held],
            NULL,
            wf_file_helper_main,
            adapter
        ) != 0) {
        return;
    }
    atomic_store_explicit(
        &adapter->helper_count,
        held + 1,
        memory_order_release
    );
}

/* Appends one accepted queue entry and grows the pool when the queue has
 * outrun it.  The caller holds the queue lock and has already made the entry's
 * ownership transition.
 *
 * Returns whether a sleeping helper has to be woken for it.  The wake itself
 * is the caller's, after the lock: a signal issued while holding the queue
 * lock wakes a helper whose very next act is to block on that same lock, so
 * the submission pays a system call to start a thread it then stalls. */
static int wf_file_enqueue_locked(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
) {
    int wake;
    wf_file_work *entry = &adapter->queue[adapter->queue_tail];
    entry->token = token;
    entry->request = *request;
    if (request->kind == WF_FILE_OPEN_AT) {
        /* The path becomes this record's own before any helper can see the
         * entry.  `wf_file_adapter_submit` refused a name that does not fit
         * before the operation was claimed, so this copy cannot fail. */
        (void)wf_file_stage_path(
            entry->path_storage,
            request->operation.open_at.path
        );
        wf_file_work_bind_path(entry);
        /* [SYS-2]'s `loan-released(path)` for this open holds from here: the
         * name the host will resolve is the record's own, so the caller's
         * buffer is free whatever the host does next.  Publishing it makes
         * that an observable fact rather than an adapter comment.  A refusal
         * is an adapter/core contract defect: it is counted, never retried,
         * and never turned into writer-visible behaviour. */
        if (wf_completion_publish_milestone(
                adapter->runtime,
                token,
                WF_COMPLETION_PAYLOAD_RELEASED
            ) != WF_COMPLETION_PUBLISHED) {
            atomic_fetch_add_explicit(
                &adapter->stat_publication_failures,
                1,
                memory_order_relaxed
            );
        }
    }
    adapter->queue_tail = (adapter->queue_tail + 1) % adapter->queue_capacity;
    adapter->queue_count += 1;
    /* Accepted from here: a queue with an engine behind it is an operation
     * that will retire, and a refused open anywhere in this process is
     * entitled to wait for it.  It is reported before the entry becomes
     * visible to a helper, so its retirement can never precede its
     * acceptance. */
    wf_completion_operation_accepted();
    atomic_fetch_add_explicit(
        &adapter->stat_submissions,
        1,
        memory_order_relaxed
    );
    /* One newly queued request needs exactly one helper woken, and only a
     * helper that is actually asleep needs a host wake at all.  Announcing to
     * every helper would cost a wake per helper per submission — a thundering
     * herd rather than progress.  This lock is the one a sleeper counts itself
     * under, so the answer read here is exact rather than a guess; a helper
     * that wakes on its own between here and the caller's signal only makes
     * that signal a spurious one, which its predicate loop already tolerates. */
    wake = adapter->blocked_helpers != 0;
    wf_file_grow_helpers_locked(adapter);
    return wake;
}

enum wf_file_submit_result wf_file_adapter_submit(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
) {
    enum wf_completion_transition_result transition;
    int wake;

    if (!wf_file_adapter_initialized(adapter)
        || !wf_file_request_valid(request)) {
        return WF_FILE_SUBMIT_INVALID;
    }
    /* A submitted open's path must fit the operation record, because the
     * record is what resolves it.  The direct executor carries any length and
     * needs no copy, so refusing here before anything is claimed leaves the
     * caller an honest fallback rather than a truncated name. */
    if (request->kind == WF_FILE_OPEN_AT
        && !wf_file_path_fits(request->operation.open_at.path)) {
        return WF_FILE_SUBMIT_INVALID;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    if (adapter->stopping != 0) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_ADAPTER_STOPPING;
    }
    transition = wf_completion_begin_submit(adapter->runtime, token);
    if (transition == WF_COMPLETION_TRANSITION_STALE) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_SUBMIT_STALE;
    }
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_SUBMIT_INVALID;
    }
    if (adapter->queue_count == adapter->queue_capacity) {
        (void)wf_completion_mark_wait_capacity(adapter->runtime, token);
        atomic_fetch_add_explicit(
            &adapter->stat_capacity_waits,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_WAIT_CAPACITY;
    }
    transition = wf_completion_target_accepted(adapter->runtime, token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return transition == WF_COMPLETION_TRANSITION_STALE
            ? WF_FILE_SUBMIT_STALE
            : WF_FILE_SUBMIT_INVALID;
    }

    wake = wf_file_enqueue_locked(adapter, token, request);
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    if (wake != 0) {
        (void)pthread_cond_signal(&adapter->queue_available);
    }
    return WF_FILE_TARGET_OWNS;
}

size_t wf_file_adapter_progress(wf_file_adapter *adapter, size_t budget) {
    size_t executed = 0;
    if (!wf_file_adapter_initialized(adapter)) {
        return 0;
    }
    while (executed < budget) {
        wf_file_work work;
        if (!wf_file_take_work(adapter, &work, 0)) {
            break;
        }
        wf_file_run_work(adapter, &work, 0, 1);
        executed += 1;
    }
    return executed;
}

size_t wf_file_adapter_queued(const wf_file_adapter *adapter) {
    if (!wf_file_adapter_initialized(adapter)) {
        return 0;
    }
    return atomic_load_explicit(&adapter->queue_count, memory_order_seq_cst);
}

int wf_file_adapter_shutdown(wf_file_adapter *adapter) {
    size_t helper;
    size_t held;
    int first_error = 0;
    if (!wf_file_adapter_initialized(adapter)) {
        return EINVAL;
    }
    (void)pthread_mutex_lock(&adapter->queue_lock);
    adapter->stopping = 1;
    /* Growth stops with admission, so the count read below is final. */
    adapter->helper_cap = 0;
    (void)pthread_cond_broadcast(&adapter->queue_available);
    (void)pthread_mutex_unlock(&adapter->queue_lock);

    held = atomic_load_explicit(&adapter->helper_count, memory_order_acquire);
    if (held == 0) {
        while (wf_file_adapter_progress(adapter, 1) != 0) {
        }
    }
    for (helper = 0; helper < held; ++helper) {
        int error = pthread_join(adapter->helpers[helper], NULL);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    /* The record stops answering "usable" before the lock behind that answer
     * is destroyed, not after.
     *
     * Every reader of this adapter passes `wf_file_adapter_initialized` first
     * and then uses storage this teardown destroys: `queue_lock` itself, or the
     * queue count beside it, which is what `wf_file_adapter_queued` reads for
     * the retirement ledger and for the decline check
     * `wf_file_adapter_transfer_runs_on_caller`.  Storing zero after the
     * destroys would leave a window in which that
     * guard still answers yes and the mutex it guards no longer exists, so a
     * reader admitted through it locks destroyed storage.  Storing zero first
     * closes the window to the ordinary one this function's precondition
     * already covers: a reader that passed the guard before this store.
     *
     * It is stored whatever the joins and destroys reported.  A record whose
     * condition variable and mutex have been destroyed is not usable, and
     * saying so is not conditional on the teardown having been clean. */
    atomic_store_explicit(&adapter->initialized, 0, memory_order_release);
    {
        int error = pthread_cond_destroy(&adapter->queue_available);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    {
        int error = pthread_mutex_destroy(&adapter->queue_lock);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    return first_error;
}

wf_file_adapter_statistics wf_file_adapter_statistics_snapshot(
    const wf_file_adapter *adapter
) {
    wf_file_adapter_statistics statistics = {0};
    if (adapter == NULL) {
        return statistics;
    }
    statistics.submissions = atomic_load_explicit(
        &adapter->stat_submissions,
        memory_order_relaxed
    );
    statistics.exhaustion_retries = atomic_load_explicit(
        &adapter->stat_exhaustion_retries,
        memory_order_relaxed
    );
    statistics.capacity_waits = atomic_load_explicit(
        &adapter->stat_capacity_waits,
        memory_order_relaxed
    );
    statistics.helper_executions = atomic_load_explicit(
        &adapter->stat_helper_executions,
        memory_order_relaxed
    );
    statistics.scheduler_executions = atomic_load_explicit(
        &adapter->stat_scheduler_executions,
        memory_order_relaxed
    );
    statistics.publication_failures = atomic_load_explicit(
        &adapter->stat_publication_failures,
        memory_order_relaxed
    );
    return statistics;
}
