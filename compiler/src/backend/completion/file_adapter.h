#ifndef WHITEFOOT_COMPLETION_FILE_ADAPTER_H
#define WHITEFOOT_COMPLETION_FILE_ADAPTER_H

/*
 * Bounded POSIX file fallback adapter.
 *
 * A queue entry is a completion record: the typed request it carries is a
 * closed, typed descriptor and there is no callback or writer thunk in this
 * interface.  Helper threads execute only the switch in file_adapter.c,
 * complete the record, and return to the queue.  A configuration with zero
 * helpers is valid: the joining thread advances the same queue with
 * wf_file_adapter_progress.
 *
 * The queue is intrusive and threaded through the records themselves
 * (`wf_completion_record::next`), so it has no capacity of its own: an
 * operation that reaches here is queued, never refused
 * (`research/investigations/io-model/PARK-ON-MISS.md` §7).
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

#define WF_FILE_STATUS_CAPACITY 192u

/* The one rule deciding whether an opened descriptor is the kind the
 * operation asked for.  Every target adapter answers with this function, so a
 * FIFO is refused identically whether the open ran on a helper thread, on the
 * joining thread itself, or on a kernel completion ring.  `file_mode` is the
 * host mode word of the descriptor that was actually opened, never of a path
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

/* What one execution of a typed request produced.
 *
 * The head is what a completion record carries and what a join reads back.
 * The status bytes below it are the direct executor's alone: a submitted
 * status writes them into the destination its request names, so no frame pays
 * 192 bytes per operation for a facility one direct call uses (design §7). */
typedef struct wf_file_result {
    wf_file_result_head head;
    size_t status_size;
    unsigned char status[WF_FILE_STATUS_CAPACITY];
} wf_file_result;

enum wf_file_submit_result {
    WF_FILE_TARGET_OWNS = 0,
    WF_FILE_SUBMIT_INVALID = 1,
    WF_FILE_ADAPTER_STOPPING = 2
};

typedef struct wf_file_adapter_statistics {
    uint64_t submissions;
    uint64_t helper_executions;
    uint64_t scheduler_executions;
} wf_file_adapter_statistics;

typedef struct wf_file_adapter {
    wf_completion_runtime *runtime;
    /* The pending list, oldest at `queue_head`, newest at `queue_tail`,
     * threaded through each record's own `next`.  Mutated only under
     * queue_lock. */
    wf_completion_record *queue_head;
    wf_completion_record *queue_tail;
    /* Mutated only under queue_lock, and atomic so the decline check can read
     * it without taking the lock. */
    _Atomic size_t queue_count;
    pthread_t *helpers;
    /* Grown under queue_lock by a submitting thread and read without it by a
     * thread deciding whether it is itself this queue's engine. */
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
    _Atomic uint64_t stat_helper_executions;
    _Atomic uint64_t stat_scheduler_executions;
} wf_file_adapter;

/* The caller owns helper_storage until shutdown completes.  helper_count is
 * policy, not a fixed architecture constant; zero selects joiner-driven
 * single-thread progress.
 *
 * helper_capacity is how many helpers `helper_storage` holds, which is a fact
 * about the caller's storage rather than a policy: the pool may later be told
 * to grow, and the only thing that can say how far is the array it grows
 * into.  It is refused below helper_count. */
int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    pthread_t *helper_storage,
    size_t helper_capacity,
    size_t helper_count
);

/* Queues one record.  There is no capacity answer: the list is threaded
 * through the records and a record is the submitting frame's, so the only way
 * this refuses is a request whose shape the ABI cannot mean, or an adapter
 * that has stopped admitting. */
enum wf_file_submit_result wf_file_adapter_submit(
    wf_file_adapter *adapter,
    wf_completion_record *record
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
 * thread, so that queueing it can only add a queue crossing to a host call
 * the caller is about to make anyway.
 *
 * True when this adapter has no helper, nothing queued, and has measured its
 * own operations as not waiting.  It is the adapter's half of the answer; the
 * caller decides whether the operation is one whose wait no part of the same
 * program has to satisfy.  What the caller does with a yes is now to execute
 * the operation inside submit and publish its completion, not to refuse it:
 * the same host call on the same thread, with the record published at the end
 * (design §7).
 *
 * Cost, stated because this is asked on a hot path: every term of it is an
 * atomic load of this record and none of them takes a lock. */
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

/* Stores one execution's answer into the record and publishes it: the status
 * bytes go to the destination the request named, the head goes into the
 * record, and `wf_completion_record_complete` stores DONE and wakes the
 * waiter.  Exactly one call of this per submission. */
void wf_file_complete_record(
    wf_completion_record *record,
    const wf_file_result *result
);

/* Executes at most `budget` queued requests on the calling thread.  It never
 * runs a Whitefoot continuation. */
/* Takes one still-queued record off the queue for the thread waiting on it,
 * and runs a record so taken.
 *
 * The caller must be the thread waiting on this exact record, and must run
 * every record the claim answered 1 for.  See the definitions for why this is
 * the one queue visit a joining thread may make while helpers exist. */
int wf_file_adapter_claim_own(
    wf_file_adapter *adapter,
    wf_completion_record *record
);
void wf_file_adapter_run_claimed(
    wf_file_adapter *adapter,
    wf_completion_record *record
);

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

/* Read without the queue lock. Zero means the calling thread is itself the
 * only engine this queue has. */
size_t wf_file_adapter_helper_count(const wf_file_adapter *adapter);

/* Stops admission and drains accepted queue entries before joining helpers.
 * With zero helpers, the calling thread performs the bounded typed work one
 * entry at a time until the accepted queue is empty.
 *
 * Precondition: no thread is inside any entry point of this adapter, and none
 * will enter one, when this is called.  That is not a caution, it is the
 * design.  The entry points that take `queue_lock` behind the record's
 * `initialized` flag -- submission, the queue readers, the cap setter, the
 * decline check -- touch storage this call tears down.
 *
 * A submission announces its queue entry *after* releasing the queue lock, on
 * purpose -- signalling under the lock wakes a helper whose next act is to
 * block on the same lock, which is a system call spent to start a thread and
 * immediately stall it -- so between that release and that signal the
 * submitter holds no lock, and a shutdown running in the window destroys the
 * condition variable the submitter is about to signal.  Shutdown clears the
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
