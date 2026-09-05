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
 * about the open changes, and the default build calls `openat` directly. */
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
    result.head.kind = request->kind;
    result.head.value = -1;

    switch (request->kind) {
    case WF_FILE_READ:
        if (request->operation.read.count > (size_t)SSIZE_MAX) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_WRITE:
        if (request->operation.write.count > (size_t)SSIZE_MAX) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PREAD:
        /* An empty transfer has no external action.  This must be decided
         * before descriptor and offset validation reaches the host: callers
         * rely on zero length completing without touching
         * either the destination or the file facility. */
        if (request->operation.pread.count == 0) {
            result.head.value = 0;
            return result;
        }
        if (request->operation.pread.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pread.offset
                != request->operation.pread.offset) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PWRITE:
        if (request->operation.pwrite.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pwrite.offset
                != request->operation.pwrite.offset) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        if (request->operation.directory_next.count > (size_t)SSIZE_MAX) {
            result.head.error_code = EINVAL;
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
            result.head.error_code = EINVAL;
            result.head.open_outcome = WF_FILE_OPEN_FAILED;
            return result;
        }
        if (request->operation.open_at.has_mode != 0) {
            result.head.value = WF_FILE_OPENAT(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      ),
                (mode_t)request->operation.open_at.mode
            );
        } else {
            result.head.value = WF_FILE_OPENAT(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      )
            );
        }
        if (result.head.value < 0) {
            result.head.open_outcome = WF_FILE_OPEN_FAILED;
            break;
        }
        if (request->operation.open_at.expected_kind != WF_FILE_EXPECT_ANY) {
            struct stat status;
            int descriptor = (int)result.head.value;
            if (fstat(descriptor, &status) != 0) {
                int status_error = errno;
                (void)close(descriptor);
                result.head.error_code = status_error;
                result.head.open_outcome = WF_FILE_OPEN_STATUS_FAILED;
                return result;
            }
            result.head.open_outcome = wf_file_kind_outcome(
                request->operation.open_at.expected_kind,
                (unsigned int)status.st_mode
            );
            if (result.head.open_outcome != WF_FILE_OPEN_SUCCEEDED) {
                (void)close(descriptor);
                return result;
            }
        }
        /* The descriptor is the one this open hands back: the kind check has
         * either passed or was not asked for. WF_IO_NOCACHE is applied here,
         * once, so a refused descriptor never receives it. */
        wf_file_apply_uncached_reads((int)result.head.value);
        break;
    case WF_FILE_READ:
        result.head.value = read(
            request->operation.read.descriptor,
            request->operation.read.buffer,
            request->operation.read.count
        );
        break;
    case WF_FILE_WRITE:
        result.head.value = write(
            request->operation.write.descriptor,
            request->operation.write.buffer,
            request->operation.write.count
        );
        break;
    case WF_FILE_PREAD:
        result.head.value = WF_COMPLETION_PREAD(
            request->operation.pread.descriptor,
            request->operation.pread.buffer,
            request->operation.pread.count,
            (off_t)request->operation.pread.offset
        );
        break;
    case WF_FILE_PWRITE:
        result.head.value = pwrite(
            request->operation.pwrite.descriptor,
            request->operation.pwrite.buffer,
            request->operation.pwrite.count,
            (off_t)request->operation.pwrite.offset
        );
        break;
    case WF_FILE_STATUS: {
        struct stat status;
        result.head.value = fstat(request->operation.status.descriptor, &status);
        if (result.head.value == 0) {
            result.status_size = sizeof(status);
            memcpy(result.status, &status, sizeof(status));
        }
        break;
    }
    case WF_FILE_CLOSE:
        result.head.value = close(request->operation.close.descriptor);
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
#if defined(__APPLE__)
        result.head.value = WF_COMPLETION_GETDIRENTRIES64(
            request->operation.directory_next.descriptor,
            request->operation.directory_next.buffer,
            request->operation.directory_next.count,
            request->operation.directory_next.position
        );
#else
        /* Linux keeps the whole enumeration cursor in the descriptor, so the
         * facility takes no base-position argument and this cell is left as
         * the caller gave it. */
        result.head.value = WF_COMPLETION_GETDENTS64(
            request->operation.directory_next.descriptor,
            request->operation.directory_next.buffer,
            request->operation.directory_next.count
        );
#endif
        break;
#endif
    default:
        result.head.error_code = EINVAL;
        return result;
    }
    if (result.head.value < 0) {
        result.head.error_code = errno;
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
        result.head.kind = request == NULL ? 0 : request->kind;
        result.head.value = -1;
        result.head.error_code = EINVAL;
        return result;
    }
    for (;;) {
        int readiness_error;
        result = wf_file_execute_once(request);
        if (result.head.value >= 0 || result.head.error_code == 0) {
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
        if (result.head.error_code == EINTR) {
            continue;
        }
        if (result.head.error_code != EAGAIN
#if defined(EWOULDBLOCK) && EWOULDBLOCK != EAGAIN
            && result.head.error_code != EWOULDBLOCK
#endif
        ) {
            return result;
        }
        readiness_error = wf_file_wait_ready(request);
        if (readiness_error != 0) {
            result.head.error_code = readiness_error;
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

/* Takes one queued record off this adapter's list.
 *
 * `from_head` chooses which end.  A joining thread's visit takes the newest
 * request, so a blocked earlier host facility cannot hide every later free
 * operation behind it, and helper threads take the oldest, so several target
 * contexts spread across both ends of the list.
 *
 * The list is singly linked through each record's own `next`, so taking the
 * newest walks it; that walk is bounded by what this program has outstanding,
 * which is bounded by the frames that hold the records, and it costs nothing
 * at all in the common case of one queued operation.  Nothing is copied out:
 * the record is the submitting frame's and outlives the operation, so the
 * executing thread runs it in place (design §5, §7).
 */
static wf_completion_record *wf_file_take_work(
    wf_file_adapter *adapter,
    int from_head
) {
    wf_completion_record *taken = NULL;
    (void)pthread_mutex_lock(&adapter->queue_lock);
    if (adapter->queue_head != NULL) {
        if (from_head != 0 || adapter->queue_head == adapter->queue_tail) {
            taken = adapter->queue_head;
            adapter->queue_head = taken->next;
            if (adapter->queue_head == NULL) {
                adapter->queue_tail = NULL;
            }
        } else {
            wf_completion_record *previous = adapter->queue_head;
            while (previous->next != adapter->queue_tail) {
                previous = previous->next;
            }
            taken = adapter->queue_tail;
            previous->next = NULL;
            adapter->queue_tail = previous;
        }
        taken->next = NULL;
        atomic_store_explicit(
            &adapter->queue_count,
            atomic_load_explicit(&adapter->queue_count, memory_order_relaxed)
                - 1u,
            memory_order_seq_cst
        );
    }
    (void)pthread_mutex_unlock(&adapter->queue_lock);
    return taken;
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
    int helper
) {
    atomic_fetch_add_explicit(
        helper != 0 ? &adapter->stat_helper_executions
                    : &adapter->stat_scheduler_executions,
        1,
        memory_order_relaxed
    );
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

/* Stores one execution's answer into the record and publishes it.
 *
 * Bytes first, head second, DONE last.  A submitted status writes into the
 * destination its own request named, because the record carries a size and
 * never the bytes (design §7); everything else has already written into
 * storage the caller named.  `wf_completion_record_complete` then stores the
 * result head's DONE through the scheduler core, which is the publisher's
 * last touch of a record that is a block of the joiner's frame. */
void wf_file_complete_record(
    wf_completion_record *record,
    const wf_file_result *result
) {
    if (record->request.kind == WF_FILE_STATUS
        && result->status_size != 0
        && record->request.operation.status.destination != NULL) {
        size_t written = result->status_size;
        if (written > record->request.operation.status.capacity) {
            written = record->request.operation.status.capacity;
        }
        memcpy(
            record->request.operation.status.destination,
            result->status,
            written
        );
        record->status_written = written;
    }
    record->result = result->head;
    wf_completion_record_complete(record);
}

static void wf_file_run_work(
    wf_file_adapter *adapter,
    wf_completion_record *record,
    int helper
) {
    wf_file_result result = wf_file_execute_timed(adapter, &record->request);
    wf_file_finish_execution(adapter, helper);
    wf_file_complete_record(record, &result);
}

static void *wf_file_helper_main(void *context) {
    wf_file_adapter *adapter = context;
    for (;;) {
        wf_completion_record *record;
        (void)pthread_mutex_lock(&adapter->queue_lock);
        while (adapter->queue_head == NULL && adapter->stopping == 0) {
            adapter->blocked_helpers += 1;
            (void)pthread_cond_wait(
                &adapter->queue_available,
                &adapter->queue_lock
            );
            adapter->blocked_helpers -= 1;
        }
        if (adapter->queue_head == NULL && adapter->stopping != 0) {
            (void)pthread_mutex_unlock(&adapter->queue_lock);
            return NULL;
        }
        record = adapter->queue_head;
        adapter->queue_head = record->next;
        if (adapter->queue_head == NULL) {
            adapter->queue_tail = NULL;
        }
        record->next = NULL;
        atomic_store_explicit(
            &adapter->queue_count,
            atomic_load_explicit(&adapter->queue_count, memory_order_relaxed)
                - 1u,
            memory_order_seq_cst
        );
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        wf_file_run_work(adapter, record, 1);
    }
}

int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    pthread_t *helper_storage,
    size_t helper_capacity,
    size_t helper_count
) {
    size_t created = 0;
    int error;

    if (adapter == NULL || runtime == NULL || helper_count > helper_capacity
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
     * may be merged.  What excludes the race is that no reader touches any of
     * these fields without first passing `wf_file_adapter_initialized`, whose
     * acquire load pairs with the release store this function ends on.
     *
     * A reader can therefore see one of these writes only if the record it
     * already holds is initialized *again* underneath it, which this adapter's
     * contract does not allow and the bridge never does: it initializes once,
     * under a `pthread_once`, with the flag still zero. */
    atomic_store_explicit(&adapter->initialized, 0, memory_order_release);
    adapter->runtime = runtime;
    adapter->queue_head = NULL;
    adapter->queue_tail = NULL;
    atomic_init(&adapter->queue_count, 0);
    adapter->helpers = helper_storage;
    atomic_init(&adapter->helper_count, 0);
    atomic_init(&adapter->mean_execute_ns, 0);
    atomic_init(&adapter->execute_ticks, 0);
    adapter->blocked_helpers = 0;
    adapter->stopping = 0;
    adapter->helper_capacity = helper_capacity;
    adapter->helper_cap = helper_count;
    atomic_init(&adapter->stat_submissions, 0);
    atomic_init(&adapter->stat_helper_executions, 0);
    atomic_init(&adapter->stat_scheduler_executions, 0);

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

/* Appends one record to the pending list and grows the pool when the list has
 * outrun it.  The caller holds the queue lock.
 *
 * There is no capacity here to answer: the list is threaded through the
 * records, each of which is a block of the frame that submitted it, so the
 * only bound is how many operations the program's own frames can hold
 * (design §7).
 *
 * Returns whether a sleeping helper has to be woken for it.  The wake itself
 * is the caller's, after the lock: a signal issued while holding the queue
 * lock wakes a helper whose very next act is to block on that same lock, so
 * the submission pays a system call to start a thread it then stalls. */
static int wf_file_enqueue_locked(
    wf_file_adapter *adapter,
    wf_completion_record *record
) {
    int wake;
    record->next = NULL;
    if (adapter->queue_tail == NULL) {
        adapter->queue_head = record;
    } else {
        adapter->queue_tail->next = record;
    }
    adapter->queue_tail = record;
    atomic_store_explicit(
        &adapter->queue_count,
        atomic_load_explicit(&adapter->queue_count, memory_order_relaxed) + 1u,
        memory_order_seq_cst
    );
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
    wf_completion_record *record
) {
    int wake;

    if (!wf_file_adapter_initialized(adapter) || record == NULL
        || !wf_file_request_valid(&record->request)) {
        return WF_FILE_SUBMIT_INVALID;
    }
    record->route = WF_COMPLETION_ROUTE_FILE_ADAPTER;
    (void)pthread_mutex_lock(&adapter->queue_lock);
    if (adapter->stopping != 0) {
        (void)pthread_mutex_unlock(&adapter->queue_lock);
        return WF_FILE_ADAPTER_STOPPING;
    }
    wake = wf_file_enqueue_locked(adapter, record);
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
        wf_completion_record *record = wf_file_take_work(adapter, 0);
        if (record == NULL) {
            break;
        }
        wf_file_run_work(adapter, record, 0);
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
     * and then uses storage this teardown destroys: `queue_lock` and the
     * condition variable beside it.  The queue count is not among them — it is
     * an atomic this teardown never writes, which is what lets
     * `wf_file_adapter_queued` be read with no lock at all, for the check
     * `wf_file_adapter_transfer_runs_on_caller` makes.  Storing zero after the
     * destroys would leave a window in which that guard still answers yes and
     * the mutex it guards no longer exists, so a reader admitted through it
     * locks destroyed storage.  Storing zero first closes the window to the
     * ordinary one this function's precondition already covers: a reader that
     * passed the guard before this store.
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
    statistics.helper_executions = atomic_load_explicit(
        &adapter->stat_helper_executions,
        memory_order_relaxed
    );
    statistics.scheduler_executions = atomic_load_explicit(
        &adapter->stat_scheduler_executions,
        memory_order_relaxed
    );
    return statistics;
}
