#ifndef WHITEFOOT_COMPLETION_FILE_ADAPTER_H
#define WHITEFOOT_COMPLETION_FILE_ADAPTER_H

/*
 * Bounded POSIX file fallback adapter.
 *
 * A queue entry is a closed, typed descriptor.  There is no callback or
 * writer thunk in this interface.  Helper threads execute only the switch in
 * file_adapter.c, publish a terminal result into the completion core, and
 * return to the target queue.  A configuration with zero helpers is valid:
 * the scheduler advances the same queue with wf_file_adapter_progress.
 */

#include "contract.h"

#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#if defined(__cplusplus)
extern "C" {
#endif

/* Whether this build's family has the [QUAL-2] directory-enumeration
 * facility: one host call that reports a bounded batch of an open directory's
 * entries and advances that descriptor's own position.  Darwin and Linux both
 * do, through different calls writing different records; a family that has
 * neither compiles no enumeration request kind at all, which is the C side of
 * the same refusal `backend/qualification.rs` makes for such a target. */
#if defined(__APPLE__) || defined(__linux__)
#define WF_FILE_HAS_DIRECTORY_NEXT 1
#endif

#define WF_FILE_STATUS_CAPACITY 192u

/* The path bytes one submitted open resolves, held by the operation record.
 *
 * A submitted open outlives the call that formed it, so the caller's name
 * buffer must stop being the operation's storage the moment submission
 * returns: the writer regains that buffer then and may rewrite it while the
 * host is still resolving the name.  Every target adapter therefore copies
 * the bytes into its own record at submission and resolves that copy.
 *
 * The bound is not every admitted name.  `open_file` and `open_directory`
 * resolve exactly one relative component, which the emitter clamps to the
 * target's own component limit — Darwin's 1023 bytes, the widest qualified
 * one, which this holds together with its terminator; the Linux family admits
 * 255.  `open_read` resolves the caller's whole path buffer, and [PATH-1]
 * admits a relative path of any length, so a name longer than this bound is
 * one a program can write.  The runtime's own harness resolves absolute
 * scratch paths as well, and 1024 is Darwin's whole `PATH_MAX`.  Storage is
 * bounded and static because a submission may not allocate.
 *
 * A name that does not fit is refused before an operation is claimed, and the
 * caller opens it directly instead — that path resolves the caller's buffer
 * inside its own call and needs no copy.  The outcome is the same open; what
 * the program loses is the completion path for it, so the demotion is a
 * throughput event of the same class as a full queue and is counted as one:
 * `wf__completion_file_demoted_opens` reports how many opens took it. */
#define WF_FILE_PATH_CAPACITY 1024u

/* Copies one path into an operation record's own storage.  Returns zero for a
 * name that does not fit, which every caller must answer before claiming an
 * operation; truncating a name would resolve a different file. */
static inline int wf_file_stage_path(char *storage, const char *path) {
    size_t length;
    if (storage == NULL || path == NULL) {
        return 0;
    }
    length = strlen(path);
    if (length >= (size_t)WF_FILE_PATH_CAPACITY) {
        return 0;
    }
    memcpy(storage, path, length + 1u);
    return 1;
}

/* Whether one path can become an operation record's own bytes.  Asked before
 * an operation is claimed, so a refusal is an honest fallback to the direct
 * path rather than a fail-stop after ownership moved. */
static inline int wf_file_path_fits(const char *path) {
    return path != NULL && strlen(path) < (size_t)WF_FILE_PATH_CAPACITY;
}

enum wf_file_operation_kind {
    WF_FILE_OPEN_AT = 1,
    WF_FILE_READ = 2,
    WF_FILE_WRITE = 3,
    WF_FILE_PREAD = 4,
    WF_FILE_PWRITE = 5,
    WF_FILE_STATUS = 6,
    WF_FILE_CLOSE = 7,
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    /* One bounded batch of directory entries, advancing the descriptor's own
     * enumeration position.  The host call behind it differs by family --
     * `__getdirentries64` on Darwin, `getdents64` on Linux -- and so does the
     * record it writes; the request record does not, because everything that
     * differs is either the call itself or the emitted shim's decoding of the
     * bytes it left behind. */
    WF_FILE_DIRECTORY_NEXT = 8,
#endif
};

enum wf_file_expected_kind {
    WF_FILE_EXPECT_ANY = 0,
    WF_FILE_EXPECT_REGULAR = 1,
    WF_FILE_EXPECT_DIRECTORY = 2
};

enum wf_file_open_outcome {
    WF_FILE_OPEN_SUCCEEDED = 0,
    WF_FILE_OPEN_FAILED = 1,
    WF_FILE_OPEN_STATUS_FAILED = 2,
    WF_FILE_OPEN_IS_DIRECTORY = 3,
    WF_FILE_OPEN_OTHER_KIND = 4
};

/* The one rule deciding whether an opened descriptor is the kind the
 * operation asked for.  Every target adapter answers with this function, so a
 * FIFO is refused identically whether the open ran on a helper thread, on the
 * scheduler itself, or on a kernel completion ring.  `file_mode` is the host
 * mode word of the descriptor that was actually opened, never of a path
 * inspected a second time. */
static inline enum wf_file_open_outcome wf_file_kind_outcome(
    enum wf_file_expected_kind expected,
    unsigned int file_mode
) {
    if (expected == WF_FILE_EXPECT_ANY
        || (expected == WF_FILE_EXPECT_REGULAR && S_ISREG(file_mode))
        || (expected == WF_FILE_EXPECT_DIRECTORY && S_ISDIR(file_mode))) {
        return WF_FILE_OPEN_SUCCEEDED;
    }
    return S_ISDIR(file_mode) ? WF_FILE_OPEN_IS_DIRECTORY
                              : WF_FILE_OPEN_OTHER_KIND;
}

/* The extra open flag a kind-checked open needs so that opening a waiting
 * facility such as a FIFO cannot block before the kind is known. */
static inline int wf_file_open_kind_flags(enum wf_file_expected_kind expected) {
    return expected == WF_FILE_EXPECT_REGULAR ? O_NONBLOCK : 0;
}

/* WF_IO_NOCACHE: a target policy that makes a program's reads genuinely wait.
 *
 * This is runtime policy of the same class as WF_IO_HELPERS and WF_WORKERS,
 * not a language surface. No Whitefoot source names it, no accepted program
 * changes meaning under it, and no byte any read produces differs with it
 * set: it asks the host to keep this file's pages out of its cache, which
 * changes only how long a read takes. Absent — and any value other than the
 * exact text "1" — is today's behaviour exactly, with no host call made at
 * all.
 *
 * It exists because every file measurement in docs/done/0084 and 0086 ran
 * against a warm page cache, where a read is a memory copy and a completion
 * model that overlaps waits has nothing to overlap. Measuring the completion
 * framework needs reads that wait on a device.
 *
 * Darwin gets F_NOCACHE, which is a mode of the descriptor: every read
 * through it bypasses the unified buffer cache for the whole life of the
 * open. Linux has no such per-descriptor mode, so it gets one
 * POSIX_FADV_DONTNEED of the whole file, which evicts what is cached at the
 * moment of the open. O_DIRECT is deliberately not used: it constrains
 * buffer address, offset, and length alignment, which would change the
 * program's own buffers and so change what is being measured.
 *
 * The setting is read once per process and cached. Two threads that race here
 * compute the same answer from the same environment, so the race is benign. */
static inline int wf_file_uncached_reads_requested(void) {
    /* 0 not yet decided, 1 asked for, 2 not asked for. */
    static _Atomic int decided;
    int state = atomic_load_explicit(&decided, memory_order_relaxed);
    if (state == 0) {
        const char *text = getenv("WF_IO_NOCACHE");
        state = (text != NULL && text[0] == '1' && text[1] == 0) ? 1 : 2;
        atomic_store_explicit(&decided, state, memory_order_relaxed);
    }
    return state == 1;
}

/* The one host call the policy makes, in one place so both adapters make the
 * same one. A build may name a different function here to observe it; the
 * observing build is expected to perform the same host call. */
#if !defined(WF_FILE_UNCACHED_APPLY)
#define WF_FILE_UNCACHED_APPLY wf_file_uncached_apply_host
#else
extern void WF_FILE_UNCACHED_APPLY(int);
#endif

static inline void wf_file_uncached_apply_host(int descriptor) {
#if defined(__APPLE__)
    (void)fcntl(descriptor, F_NOCACHE, 1);
#elif defined(__linux__)
    (void)posix_fadvise(descriptor, 0, 0, POSIX_FADV_DONTNEED);
#else
    (void)descriptor;
#endif
}

/* Applied exactly once to each descriptor an open hands back, by whichever
 * adapter produced it. A descriptor the kind check refuses never reaches
 * here, so the count of applications equals the count of successful opens. */
static inline void wf_file_apply_uncached_reads(int descriptor) {
    if (descriptor < 0 || !wf_file_uncached_reads_requested()) {
        return;
    }
    WF_FILE_UNCACHED_APPLY(descriptor);
}

typedef struct wf_file_request {
    enum wf_file_operation_kind kind;
    union {
        struct {
            int directory;
            /* Names the operation record's own bytes once the request is
             * queued; the caller's buffer only while the request is still the
             * caller's, which is the direct path and the moment before
             * `wf_file_work_bind_path` runs. */
            const char *path;
            int flags;
            unsigned mode;
            unsigned has_mode;
            enum wf_file_expected_kind expected_kind;
        } open_at;
        struct {
            int descriptor;
            void *buffer;
            size_t count;
        } read;
        struct {
            int descriptor;
            const void *buffer;
            size_t count;
        } write;
        struct {
            int descriptor;
            void *buffer;
            size_t count;
            int64_t offset;
        } pread;
        struct {
            int descriptor;
            const void *buffer;
            size_t count;
            int64_t offset;
        } pwrite;
        struct {
            int descriptor;
        } status;
        struct {
            int descriptor;
        } close;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
        struct {
            int descriptor;
            void *buffer;
            size_t count;
            /* Darwin's facility requires a base-position cell and writes the
             * position of the batch it reported into it.  Linux's takes no
             * such argument and keeps the whole cursor in the descriptor, so
             * on that family this cell is left exactly as the caller gave it.
             * Either way it is scratch storage of the emitted shim's, never a
             * component of the `DirectorySource` value. */
            int64_t *position;
        } directory_next;
#endif
    } operation;
} wf_file_request;

typedef struct wf_file_result {
    enum wf_file_operation_kind kind;
    int64_t value;
    int error_code;
    enum wf_file_open_outcome open_outcome;
    size_t status_size;
    unsigned char status[WF_FILE_STATUS_CAPACITY];
} wf_file_result;

/* Whether this finished operation put a descriptor back in the host's table,
 * which is the one thing `wf_completion_operation_retired` asks of it.
 *
 * A close ran the host call that returns one.  An open whose descriptor this
 * runtime obtained and then disposed of returns one too: the kind check refused
 * the descriptor, and the close made for it is a descriptor coming back, which
 * a refused open waiting on the ledger is entitled to see.  Every other ending
 * returns nothing — an open the host refused never held one, an open the
 * program now holds has not given it back, and a transfer, a status and a
 * directory batch were only lent one.
 *
 * It reads the finished operation's own record, so the answer follows the
 * operation rather than the route it took; the ring answers the same question
 * from its entry. */
static inline int wf_file_returned_a_descriptor(const wf_file_result *result) {
    if (result == NULL) {
        return 0;
    }
    if (result->kind == WF_FILE_CLOSE) {
        /* Only a close the host performed put a descriptor back; a refused
         * close (EBADF) freed nothing, and counting it spends a refused
         * open's one re-attempt on a return that never happened. */
        return result->error_code == 0;
    }
    return result->kind == WF_FILE_OPEN_AT && result->value >= 0
        && result->open_outcome != WF_FILE_OPEN_SUCCEEDED;
}

/* Whether the host satisfied this open, which is the one thing
 * `wf_completion_retirement_open_took_a_descriptor` asks of it: there is a
 * descriptor in this open's hand that was in the host's table before it ran.
 *
 * The kind check does not come into it.  An open the check refuses held the
 * descriptor all the same, and the close made for it is reported separately as
 * the return it is — so such an open is charged once here and counted once
 * there, and the two cancel exactly as they should. */
static inline int wf_file_open_took_a_descriptor(
    const wf_file_result *result
) {
    return result != NULL && result->kind == WF_FILE_OPEN_AT
        && result->value >= 0;
}

typedef struct wf_file_work {
    wf_completion_token token;
    wf_file_request request;
    /* An open's path bytes, owned by this record. */
    char path_storage[WF_FILE_PATH_CAPACITY];
} wf_file_work;

/* Points a staged open at this record's own path bytes.
 *
 * A work record is copied whenever it moves — into the bounded queue at
 * submission, out of it at execution — and a pointer into the record it was
 * copied *from* would name storage the queue is free to reuse.  Every copy
 * therefore rebinds, and the invariant is that a work record's open never
 * names bytes outside itself. */
static inline void wf_file_work_bind_path(wf_file_work *work) {
    if (work != NULL && work->request.kind == WF_FILE_OPEN_AT) {
        work->request.operation.open_at.path = work->path_storage;
    }
}

enum wf_file_submit_result {
    WF_FILE_TARGET_OWNS = 0,
    WF_FILE_WAIT_CAPACITY = 1,
    WF_FILE_SUBMIT_STALE = 2,
    WF_FILE_SUBMIT_INVALID = 3,
    WF_FILE_ADAPTER_STOPPING = 4
};

typedef struct wf_file_adapter_statistics {
    uint64_t submissions;
    /* Opens refused for want of a host descriptor that this adapter asked the
     * host about a second time, after running work it still owed or after
     * waiting for an operation in flight anywhere else to retire.  A refusal
     * published without a second attempt is not counted here. */
    uint64_t exhaustion_retries;
    uint64_t capacity_waits;
    uint64_t helper_executions;
    uint64_t scheduler_executions;
    uint64_t publication_failures;
} wf_file_adapter_statistics;

typedef struct wf_file_adapter {
    wf_completion_runtime *runtime;
    wf_file_work *queue;
    size_t queue_capacity;
    size_t queue_head;
    size_t queue_tail;
    /* Mutated only under queue_lock, and atomic so the retirement ledger can
     * read it while holding its own lock instead of this one: a refused open
     * deciding whether to sleep must read the queue at the moment it decides,
     * and reaching for queue_lock there would invert the order every
     * submission takes. */
    _Atomic size_t queue_count;
    pthread_t *helpers;
    /* Grown under queue_lock by a submitting thread and read without it by a
     * scheduler deciding whether it is itself this queue's engine. */
    _Atomic size_t helper_count;
    size_t helper_cap;
    pthread_mutex_t queue_lock;
    pthread_cond_t queue_available;
    /* Helpers currently inside pthread_cond_wait, maintained under queue_lock
     * by the waiter itself.  An enqueue holds that same lock, so it knows
     * exactly whether a signal has anyone to reach: a helper still spinning
     * for work needs no host wake, and issuing one for it was a system call
     * per operation that woke a thread which was already awake. */
    size_t blocked_helpers;
    /* What one host call on this adapter has recently cost, in nanoseconds, as
     * a smoothed average over sampled executions.  It is policy input only —
     * no operation's outcome depends on it — and it answers the one question
     * the helper policy has to answer: whether this program's operations wait
     * long enough for handing one to another thread to be worth the handoff. */
    _Atomic uint64_t mean_execute_ns;
    _Atomic uint64_t execute_ticks;
    unsigned stopping;
    /* How many helpers `helper_storage` can hold.  The growth rule writes
     * `helpers[held]`, so this, and not the policy's wish, is what bounds the
     * ceiling `wf_file_adapter_set_helper_cap` may install. */
    size_t helper_capacity;
    /* Whether every field above has been published.  It is atomic because the
     * direct execution route reads it on a thread that has run no
     * initialization of its own: `wf_file_execute_timed` is reached straight
     * from a generated direct call, with no once-control between it and this
     * adapter, while another thread may be inside `wf_file_adapter_init`.  A
     * release store here, paired with the acquire loads, is what makes the
     * rest of the record safe to read for a thread that finds it set. */
    _Atomic unsigned initialized;

    _Atomic uint64_t stat_submissions;
    _Atomic uint64_t stat_exhaustion_retries;
    _Atomic uint64_t stat_capacity_waits;
    _Atomic uint64_t stat_helper_executions;
    _Atomic uint64_t stat_scheduler_executions;
    _Atomic uint64_t stat_publication_failures;
} wf_file_adapter;

/* The caller owns queue_storage and helper_storage until shutdown completes.
 * helper_count is policy, not a fixed architecture constant; zero selects
 * scheduler-driven single-thread progress.
 *
 * helper_capacity is how many helpers `helper_storage` holds, which is a fact
 * about the caller's storage rather than a policy: the pool may later be told
 * to grow, and the only thing that can say how far is the array it grows
 * into.  It is refused below helper_count. */
int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_file_work *queue_storage,
    size_t queue_capacity,
    pthread_t *helper_storage,
    size_t helper_capacity,
    size_t helper_count
);

enum wf_file_submit_result wf_file_adapter_submit(
    wf_file_adapter *adapter,
    wf_completion_token token,
    const wf_file_request *request
);

/* What this adapter has measured about how long its own host calls take.
 *
 * Both policies this answers are conditional on a *measurement*, never on the
 * absence of one: an adapter that has executed nothing knows nothing, and a
 * program's first operations therefore take the completion path unchanged and
 * are measured there. */
enum wf_file_wait_verdict {
    /* Nothing executed yet. */
    WF_FILE_WAIT_UNMEASURED = 0,
    /* The host answers without waiting: overlapping buys nothing. */
    WF_FILE_WAIT_SHORT = 1,
    /* The host makes these operations wait: overlapping is worth its handoff. */
    WF_FILE_WAIT_LONG = 2
};

enum wf_file_wait_verdict wf_file_adapter_wait_verdict(
    const wf_file_adapter *adapter
);

/* Whether a transfer submitted now would simply be executed by the submitting
 * thread, so that submitting it can only add a queue crossing to a host call
 * the caller is about to make anyway.
 *
 * True when this adapter has no helper, nothing queued, and has measured its
 * own operations as not waiting.  It is the adapter's half of the answer; the
 * caller decides whether the operation is one whose wait no part of the same
 * program has to satisfy.
 *
 * Cost, stated because this is asked on a hot path: every term of it is an
 * atomic load of this record and none of them takes a lock, so a positioned
 * read that reaches this question pays a handful of loads and no lock at all.
 * The question is asked once per
 * positioned read on a program the bridge has not pinned, and only after the
 * two cheaper terms have both held; what it saves when it answers yes is a
 * queue crossing, a slot claim, four slot transitions and a drain. */
int wf_file_adapter_transfer_runs_on_caller(const wf_file_adapter *adapter);

/* Executes one typed request and records what the host call cost, from a
 * sample of executions, into the average the policy above reads.  Every route
 * that makes a host call for this adapter goes through here, so the average is
 * of the program's operations and not of one route's. */
wf_file_result wf_file_execute_timed(
    wf_file_adapter *adapter,
    const wf_file_request *request
);

/* Executes one typed request to its first terminal host answer. EINTR and
 * read/write/directory readiness refusal are adapter progress; close is never
 * retried because one ambiguous close attempt has already consumed authority.
 * Other non-transfer operations remain exactly one host attempt. */
wf_file_result wf_file_execute_direct(const wf_file_request *request);

/* Executes at most `budget` typed requests on the calling scheduler thread.
 * It never runs a Whitefoot continuation. */
size_t wf_file_adapter_progress(wf_file_adapter *adapter, size_t budget);

size_t wf_file_adapter_queued(const wf_file_adapter *adapter);

/* Raises the ceiling the helper pool may grow to when a program exposes more
 * independent queued work than there are helpers to take it. A cap at or below
 * the initial count disables growth, which is what a pinned helper policy asks
 * for.
 *
 * A cap above the `helper_capacity` given to init is clamped to it rather than
 * refused: the caller is stating a policy, and the storage it handed over is
 * the fact that bounds it.  The clamp is the difference between a pool that
 * stops growing and a `pthread_create` past the end of the caller's array. */
int wf_file_adapter_set_helper_cap(wf_file_adapter *adapter, size_t cap);

/* Read without the queue lock. Zero means the calling scheduler is itself the
 * only engine this queue has. */
size_t wf_file_adapter_helper_count(const wf_file_adapter *adapter);

/* Stops admission and drains accepted queue entries before joining helpers.
 * With zero helpers, the calling thread performs the bounded typed work one
 * entry at a time until the accepted queue is empty.
 *
 * Precondition: no thread is inside `wf_file_adapter_submit` or
 * `wf_file_adapter_transfer_runs_on_caller`, and none will enter either, when
 * this is called.  That is not a caution, it is the design.
 *
 * A submission announces its queue entry *after* releasing the queue lock, on
 * purpose -- signalling under the lock wakes a helper whose next act is to
 * block on the same lock, which is a system call spent to start a thread and
 * immediately stall it -- so between that release and that signal the
 * submitter holds no lock, and a shutdown running in the window destroys the
 * condition variable the submitter is about to signal.  The decline check is
 * named beside it because it is the other entry a delivered program reaches
 * without holding anything, and it holds nothing at any point: it asks
 * `wf_file_adapter_queued`, which is a plain atomic load of `queue_count` and
 * takes no lock — the retirement ledger requires exactly that of the same
 * read, because it asks for the count while holding its own lock — so a
 * shutdown in its window destroys nothing it touches, and the most it can do
 * to it is leave it a count nothing maintains any more.  Shutdown clears the
 * record's `initialized` flag before destroying the condition variable and the
 * mutex, which is what bounds the submission window to a caller that had
 * already passed the flag.
 *
 * Closing them completely would mean either signalling under the lock, which
 * is the cost this shape exists to remove, or a second lock on the submission
 * path to serialize against a shutdown that happens once per process.
 * Neither is worth it for an overlap no caller has: the bridge's only
 * shutdown is its `atexit` handler, and a program still submitting operations
 * while the process exits has no defined completion for them anyway. */
int wf_file_adapter_shutdown(wf_file_adapter *adapter);

wf_file_adapter_statistics wf_file_adapter_statistics_snapshot(
    const wf_file_adapter *adapter
);

#if defined(__cplusplus)
}
#endif

#endif
