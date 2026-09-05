#ifndef WHITEFOOT_COMPLETION_FILE_ADAPTER_H
#define WHITEFOOT_COMPLETION_FILE_ADAPTER_H

/*
 * The bounded file adapter, one implementation for every platform.
 *
 * A queue entry is a completion record: the typed request it carries is a
 * closed, typed descriptor and there is no callback or writer thunk in this
 * interface.  Helper threads execute only `wf_file_execute_direct`, complete
 * the record, and return to the queue.  A configuration with zero helpers is
 * valid: the joining thread advances the same queue with
 * wf_file_adapter_progress.
 *
 * The queue, the helpers, the claim of one's own queued record and the
 * in-place progress are written once here.  The one thing that is not shared
 * is the host call each request kind makes, which is `wf_file_execute_direct`
 * below and lives in one leaf per platform (`file_posix.c`,
 * `file_windows.c`).
 *
 * The queue is intrusive and threaded through the records themselves
 * (`wf_completion_record::next`), so it has no capacity of its own: an
 * operation that reaches here is queued, never refused
 * (`research/investigations/io-model/PARK-ON-MISS.md` §7).
 */

#include "contract.h"
/* For `wf_prim_thread`, the caller-owned record one helper's start needs: this
 * adapter's helpers are the runtime's own threads, and this adapter is where
 * their entry records live (`../sched/prim.h`, P1). */
#include "../sched/prim.h"

#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#if defined(__cplusplus)
extern "C" {
#endif

#define WF_FILE_STATUS_CAPACITY 192u

/* The most helpers one adapter may ever hold.
 *
 * It is here rather than at the caller because it sizes storage this record
 * carries: one `wf_prim_thread` per helper, which is where a helper's entry
 * pair lives for the life of that helper (`../sched/prim.h`, P1). The caller
 * still states its own bound at `wf_file_adapter_init`, and a bound above this
 * one is refused rather than clamped, because a caller asking for more helpers
 * than this adapter can hold is asking for a pool it will not get. */
#define WF_FILE_MAX_HELPERS 8u

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
    /* The queue's lock and the condition a waiting helper sleeps on: the
     * platform's one wait set, exactly as the completion runtime's is. */
    wf_completion_wait queue_wait;
    /* The pending list, oldest at `queue_head`, newest at `queue_tail`,
     * threaded through each record's own `next`.  Mutated only under
     * queue_lock. */
    wf_completion_record *queue_head;
    wf_completion_record *queue_tail;
    /* Mutated only under queue_lock, and atomic so the decline check can read
     * it without taking the lock. */
    _Atomic size_t queue_count;
    /* Grown under the queue lock by a submitting thread and read without it by
     * a thread deciding whether it is itself this queue's engine. */
    _Atomic size_t helper_count;
    size_t helper_cap;
    /* Helpers currently asleep on the queue condition, maintained under the
     * queue lock by the sleeper itself.  An enqueue holds that same lock, so it
     * knows exactly whether a signal has anyone to reach: a helper still
     * spinning for work needs no host wake, and issuing one for it was a system
     * call per operation that woke a thread which was already awake. */
    size_t blocked_helpers;
    /* One entry record per helper, written by the starting thread under the
     * queue lock before the helper exists and never written again. */
    wf_prim_thread helper_threads[WF_FILE_MAX_HELPERS];
    /* Helpers that have started and not yet returned, maintained under the
     * queue lock.  Shutdown waits for this to reach zero rather than joining
     * threads by handle: the helpers are detached, as every thread this
     * runtime starts is, and a helper's last act under the lock is to drop
     * this count and wake whoever is waiting for it. */
    size_t live_helpers;
    /* What one host call on this adapter has recently cost, in nanoseconds, as
     * a smoothed average over sampled executions.  It is policy input only —
     * no operation's outcome depends on it — and it answers the one question
     * the helper policy has to answer: whether this program's operations wait
     * long enough for handing one to another thread to be worth the handoff. */
    _Atomic uint64_t mean_execute_ns;
    _Atomic uint64_t execute_ticks;
    unsigned stopping;
    /* How many helpers this adapter may ever hold.  It is the caller's stated
     * bound on the pool rather than the policy's wish, so it, and not the wish,
     * is what bounds the ceiling `wf_file_adapter_set_helper_cap` installs. */
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

/* helper_count is policy, not a fixed architecture constant; zero selects
 * joiner-driven single-thread progress.
 *
 * helper_capacity is how many helpers this adapter may ever hold: the pool may
 * later be told to grow, and this is the bound that says how far.  It is
 * refused below helper_count and above WF_FILE_MAX_HELPERS, which is the
 * number of entry records this record carries.  The helper threads themselves are the
 * platform's, started through `wf_prim_thread_start` and detached, so no
 * caller owns storage for them. */
int wf_file_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
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

/* The one platform leaf of this adapter: executes one typed request against
 * the host and reports its first terminal answer.
 *
 * `file_posix.c` and `file_windows.c` are the two implementations, and each is
 * exactly the switch of host calls its platform makes.  Interruption and
 * readiness refusal are adapter progress and are absorbed there; close is never
 * retried because one ambiguous close attempt has already consumed authority;
 * other non-transfer operations remain exactly one host attempt.  A request
 * kind a platform's qualified target row does not admit is reported as a failed
 * outcome with the host's own refusal code, never as a terminated process: the
 * emitter can produce a shape a target refuses, and refusing it is an outcome
 * the program sees. */
wf_file_result wf_file_execute_direct(const wf_file_request *request);

/* Whether one typed request's shape is one this ABI can mean at all.  It is
 * the adapter's own check rather than a host's, so it is shared; the leaf runs
 * it before it makes any host call. */
int wf_file_request_valid(const wf_file_request *request);

/* The monotonic clock in nanoseconds, or zero when the host would not answer.
 *
 * It is a host call, so it is the platform leaf's like the execution switch
 * above.  Its two readers are both shared: the helper policy's measurement of
 * what this program's operations cost, and the bridge's bounded look at its own
 * record before it announces sleep. */
uint64_t wf_file_monotonic_ns(void);

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
 * refused: the caller is stating a policy, and the bound it gave at init is the
 * fact that limits it.  The clamp is the difference between a pool that stops
 * growing and one that grows past the number of helpers this adapter was told
 * it may ever hold. */
int wf_file_adapter_set_helper_cap(wf_file_adapter *adapter, size_t cap);

/* Read without the queue lock. Zero means the calling thread is itself the
 * only engine this queue has. */
size_t wf_file_adapter_helper_count(const wf_file_adapter *adapter);

/* Stops admission and drains accepted queue entries before waiting for the
 * helpers to leave.  With zero helpers, the calling thread performs the bounded
 * typed work one entry at a time until the accepted queue is empty.
 *
 * Precondition: no thread is inside any entry point of this adapter, and none
 * will enter one, when this is called.  That is not a caution, it is the
 * design.  The entry points that take the queue lock behind the record's
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
