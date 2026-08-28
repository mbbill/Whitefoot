#ifndef WHITEFOOT_COMPLETION_CONTRACT_H
#define WHITEFOOT_COMPLETION_CONTRACT_H

/*
 * Finite one-shot completion core.
 *
 * This is an internal compiler/runtime ABI, not a writer-visible API.  It
 * deliberately contains no function pointer which could name Whitefoot code:
 * target adapters publish bytes and milestone facts, and the scheduler alone
 * decides which saved Whitefoot frame becomes runnable.
 *
 * All storage is supplied by the embedding runtime.  Claiming one operation
 * can therefore ask the scheduler to retry only after real slot-capacity
 * release; it can never grow a hidden queue or reserve a compiler-invented
 * operation group.  A token names both a slot and its generation.  The
 * generation is checked while the slot publication lock is held and before
 * any result byte is written.
 */

#include <pthread.h>
#include <stdalign.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

#define WF_COMPLETION_RESULT_CAPACITY 256u

/* These facts are independent even when a simple adapter publishes all four
 * in one terminal transition.  Future split-milestone operations must not turn
 * this product back into one DONE bit. */
enum wf_completion_milestone {
    WF_COMPLETION_RESULT_READY = 1u << 0,
    WF_COMPLETION_PAYLOAD_RELEASED = 1u << 1,
    WF_COMPLETION_RESOURCE_RELEASED = 1u << 2,
    WF_COMPLETION_TERMINAL = 1u << 3
};

#define WF_COMPLETION_OWNERSHIP_COMPLETE                                      \
    (WF_COMPLETION_RESULT_READY | WF_COMPLETION_PAYLOAD_RELEASED               \
     | WF_COMPLETION_RESOURCE_RELEASED | WF_COMPLETION_TERMINAL)

enum wf_completion_phase {
    WF_COMPLETION_FREE = 0,
    WF_COMPLETION_READY = 1,
    WF_COMPLETION_WAIT_CAPACITY = 2,
    WF_COMPLETION_SUBMITTING = 3,
    WF_COMPLETION_IN_FLIGHT = 4,
    WF_COMPLETION_TERMINAL_PHASE = 5,
    /* A slot reaches RETIRED instead of wrapping its generation. */
    WF_COMPLETION_RETIRED = 6
};

typedef struct wf_completion_token {
    uint32_t slot;
    uint64_t generation;
} wf_completion_token;

enum wf_completion_claim_result {
    WF_COMPLETION_CLAIMED = 0,
    WF_COMPLETION_CLAIM_WAIT_CAPACITY = 1,
    WF_COMPLETION_CLAIM_INVALID = 2
};

enum wf_completion_transition_result {
    WF_COMPLETION_TRANSITIONED = 0,
    WF_COMPLETION_TRANSITION_STALE = 1,
    WF_COMPLETION_TRANSITION_INVALID_STATE = 2
};

enum wf_completion_publish_result {
    WF_COMPLETION_PUBLISHED = 0,
    WF_COMPLETION_PUBLISH_STALE = 1,
    WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL = 2,
    WF_COMPLETION_PUBLISH_INVALID_STATE = 3,
    WF_COMPLETION_PUBLISH_RESULT_TOO_LARGE = 4,
    WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL = 5,
    WF_COMPLETION_PUBLISH_INVALID_ARGUMENT = 6
};

enum wf_completion_consume_result {
    WF_COMPLETION_CONSUMED = 0,
    WF_COMPLETION_CONSUME_STALE = 1,
    WF_COMPLETION_CONSUME_NOT_TERMINAL = 2,
    WF_COMPLETION_CONSUME_NOT_DRAINED = 3,
    WF_COMPLETION_CONSUME_RESULT_TOO_LARGE = 4,
    WF_COMPLETION_CONSUME_INVALID_ARGUMENT = 5
};

enum wf_completion_consume_wait_result {
    WF_COMPLETION_CONSUME_READY = 0,
    WF_COMPLETION_CONSUME_WAIT_REGISTERED = 1,
    WF_COMPLETION_CONSUME_WAIT_STALE = 2
};

enum wf_completion_depend_result {
    WF_COMPLETION_DEPEND_REGISTERED = 0,
    WF_COMPLETION_DEPEND_ALREADY_READY = 1,
    WF_COMPLETION_DEPEND_STALE = 2,
    WF_COMPLETION_DEPEND_DUPLICATE = 3,
    WF_COMPLETION_DEPEND_INVALID_ARGUMENT = 4
};

/* Runtime-wide scheduler injection. It accepts only an opaque ready frame and
 * never names or invokes writer code. */

/* Compiler-owned host-wait notification.  This is separate from ready-frame
 * injection: every epoch change (compute, completion, or released capacity)
 * reaches the callback so a target adapter can join the core's wake source to
 * its native completion wait set.  The callback may only announce a host wait
 * endpoint; it must not run a writer continuation. */
typedef void (*wf_completion_wake_callback)(void *context);

enum wf_completion_park_result {
    WF_COMPLETION_PARK_WOKEN = 0,
    WF_COMPLETION_PARK_EPOCH_CHANGED = 1,
    WF_COMPLETION_PARK_TIMED_OUT = 2,
    WF_COMPLETION_PARK_FAILED = 3
};

typedef struct wf_completion_publication {
    uint32_t milestones;
    /* Operation-defined, nonzero terminal outcome discriminator. */
    uint32_t terminal_kind;
    const void *result;
    size_t result_size;
} wf_completion_publication;

typedef struct wf_completion_event {
    wf_completion_token token;
    uint32_t milestones;
    uint32_t terminal_kind;
} wf_completion_event;

typedef struct wf_completion_outcome {
    uint32_t milestones;
    uint32_t terminal_kind;
    uint32_t adapter_tag;
    size_t result_size;
} wf_completion_outcome;

/* Public layout permits exact static/frame allocation.  Every field is
 * runtime-owned between init and destroy. */
typedef struct wf_completion_slot {
    pthread_mutex_t publication_lock;
    _Atomic uint64_t generation;
    _Atomic unsigned phase;
    _Atomic uint32_t milestones;
    _Atomic unsigned event_pending;
    _Atomic unsigned event_drained;
    /* Protected by publication_lock. A token owner sets this only across the
     * final recheck-to-park handshake; the drainer clears and wakes it. */
    unsigned consume_waiting;
    uint32_t terminal_kind;
    uint32_t adapter_tag;
    size_t result_size;
    void *dependent_frame;
    uint32_t dependent_requirement;
    union {
        max_align_t alignment;
        unsigned char bytes[WF_COMPLETION_RESULT_CAPACITY];
    } result;
} wf_completion_slot;

typedef struct wf_completion_statistics {
    uint64_t claims;
    uint64_t claim_capacity_waits;
    uint64_t target_capacity_waits;
    uint64_t publications;
    uint64_t stale_publications;
    uint64_t duplicate_terminals;
    uint64_t drained_events;
    uint64_t consumptions;
    uint64_t parks;
    uint64_t wake_signals;
    uint64_t compute_notifications;
    uint64_t target_notifications;
    uint64_t capacity_notifications;
} wf_completion_statistics;

typedef struct wf_completion_runtime {
    wf_completion_slot *slots;
    size_t slot_count;
    _Atomic size_t claim_cursor;
    _Atomic size_t drain_cursor;
    _Atomic size_t ready_events;

    pthread_mutex_t wake_lock;
    pthread_cond_t wake_condition;
    _Atomic uint64_t wake_epoch;
    _Atomic unsigned parked_schedulers;

    _Atomic uint64_t stat_claims;
    _Atomic uint64_t stat_claim_capacity_waits;
    _Atomic uint64_t stat_target_capacity_waits;
    _Atomic uint64_t stat_publications;
    _Atomic uint64_t stat_stale_publications;
    _Atomic uint64_t stat_duplicate_terminals;
    _Atomic uint64_t stat_drained_events;
    _Atomic uint64_t stat_consumptions;
    _Atomic uint64_t stat_parks;
    _Atomic uint64_t stat_wake_signals;
    _Atomic uint64_t stat_compute_notifications;
    _Atomic uint64_t stat_target_notifications;
    _Atomic uint64_t stat_capacity_notifications;
    wf_completion_wake_callback wake_callback;
    void *wake_context;
} wf_completion_runtime;

/* Returns zero on success.  `slots` is the complete operation and completion
 * capacity; it remains owned by `runtime` until a successful destroy. */
int wf_completion_runtime_init(
    wf_completion_runtime *runtime,
    wf_completion_slot *slots,
    size_t slot_count
);

/* Destroy refuses while any operation, ready event, or parked scheduler still
 * exists.  It returns zero on success and EBUSY/EINVAL otherwise. */
int wf_completion_runtime_destroy(wf_completion_runtime *runtime);

/* Installs the target's host-wait announcer before any scheduler can park.
 * At most one announcer may be installed. */
int wf_completion_set_wake_callback(
    wf_completion_runtime *runtime,
    wf_completion_wake_callback wake,
    void *context
);

enum wf_completion_claim_result wf_completion_claim(
    wf_completion_runtime *runtime,
    wf_completion_token *token
);

/* Adapter handoff protocol.  begin accepts READY or WAIT_CAPACITY.  A target
 * queue reserves its own bounded entry, calls begin, then either records
 * WAIT_CAPACITY or marks the operation accepted before exposing the queue
 * entry to a helper/kernel. */
enum wf_completion_transition_result wf_completion_begin_submit(
    wf_completion_runtime *runtime,
    wf_completion_token token
);
enum wf_completion_transition_result wf_completion_mark_wait_capacity(
    wf_completion_runtime *runtime,
    wf_completion_token token
);
enum wf_completion_transition_result wf_completion_target_accepted(
    wf_completion_runtime *runtime,
    wf_completion_token token
);

/* Binds target-adapter decoding metadata to this exact slot generation. It is
 * read and cleared by consume while reuse remains excluded. */
enum wf_completion_transition_result wf_completion_set_adapter_tag(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t adapter_tag
);

/* Publishes one non-terminal milestone fact for an operation the target is
 * submitting or already holds.
 *
 * The facts of one operation are independent, and one of them can hold long
 * before the operation is terminal: once a submitted open's path bytes are the
 * operation record's own, the caller's name buffer is free whatever the host
 * does next, so `loan-released(path)` is true from that moment.  Publishing it
 * here is what makes that an observable fact rather than a comment inside an
 * adapter.
 *
 * This is not a terminal route.  It writes no result byte, does not move the
 * phase, and raises no completion event, so a one-shot operation still has
 * exactly one event and exactly one terminal.  A fact published here is
 * visible to wf_completion_observe immediately and is carried by the
 * operation's own terminal event; by itself it wakes nothing, so no frame may
 * be left depending on it alone.
 *
 * WF_COMPLETION_TERMINAL is refused: the terminal fact is published only by
 * the routes that carry the result. */
enum wf_completion_publish_result wf_completion_publish_milestone(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t milestones
);

/* The asynchronous terminal route is valid only after target acceptance. */
enum wf_completion_publish_result wf_completion_publish_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
);

/* The inline route is valid only while SUBMITTING and proves that no later
 * target event can exist (including a typed rejection before ownership). */
enum wf_completion_publish_result wf_completion_publish_inline_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
);

/* Drains at most `scan_budget` slots and at most `event_capacity` events.
 * Every call is bounded.  Any scheduler lane may drain; no event is tied to
 * the lane which submitted it. */
size_t wf_completion_drain(
    wf_completion_runtime *runtime,
    wf_completion_event *events,
    size_t event_capacity,
    size_t scan_budget
);

/* Drains the completion event of one named operation, and only that one.
 *
 * `wf_completion_drain` sweeps slots because a scheduler harvesting on behalf
 * of everybody cannot know where the next event is.  A token owner does know:
 * it is waiting on exactly one operation and can name its slot.  Sweeping on
 * its behalf makes the cost of consuming one completion proportional to the
 * whole slot array rather than to the one event, which on a program that
 * joins its own operations is the largest cost in the path.
 *
 * The transition is the same one the sweep makes — the pending event is taken
 * once, the drained fact is published under the slot's publication lock, and
 * a frame whose requirement this event satisfies is made runnable — so an
 * event drained here is indistinguishable from one the sweep drained.
 *
 * Naming a slot, however, is not naming an operation, and this entry is
 * token-named: like every other token-named entry it refuses a token whose
 * generation no longer matches the slot's, so an operation that has already
 * ended cannot take the event of the operation that reused its slot.  The
 * sweep needs no such check because it names no operation at all.
 *
 * Returns 1 when this call took the event, 0 when there was none of this
 * operation's to take. */
size_t wf_completion_drain_token(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    wf_completion_event *event
);

size_t wf_completion_ready_event_count(const wf_completion_runtime *runtime);

/* Observes milestone facts without consuming the result. */
enum wf_completion_transition_result wf_completion_observe(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t *milestones,
    unsigned *phase
);

/* Registers one exact frame requirement before target handoff. The scheduler
 * injects that opaque frame at most once, outside the publication lock, after
 * the requirement is satisfied and this exact completion event is drained.
 * A resumed frame can therefore consume its result immediately. */
enum wf_completion_depend_result wf_completion_depend(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t requirement,
    void *frame
);

/* Consumption transfers the result and recycles the slot.  The completion
 * event must first have been drained so a reused slot can never leave an old
 * event in the scheduler's ready set. */
enum wf_completion_consume_result wf_completion_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    void *result,
    size_t result_capacity,
    wf_completion_outcome *outcome
);

/* Atomically performs a token owner's final consumability recheck and, when
 * still unavailable, registers the exact drain transition which must wake it. */
enum wf_completion_consume_wait_result wf_completion_wait_to_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token
);

/* One epoch covers both compute publication and target completion.  Scheduler
 * protocol is: process/drain/progress; snapshot the epoch; recheck all sources;
 * then park_if_unchanged.  The park function announces sleep under the same
 * lock used by publishers, closes the final race, and tolerates spurious host
 * wakes.  UINT32_MAX requests an unbounded wait. */
uint64_t wf_completion_wake_epoch(const wf_completion_runtime *runtime);
void wf_completion_notify_compute(wf_completion_runtime *runtime);
/* Newly runnable bounded target work uses the same epoch and host endpoint. */
void wf_completion_notify_target(wf_completion_runtime *runtime);
/* A released operation slot or target-queue credit is scheduler progress too.
 * It shares the same epoch and park endpoint as compute and completion. */
void wf_completion_notify_capacity(wf_completion_runtime *runtime);
enum wf_completion_park_result wf_completion_park_if_unchanged(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
);
unsigned wf_completion_parked_scheduler_count(
    const wf_completion_runtime *runtime
);

wf_completion_statistics wf_completion_statistics_snapshot(
    const wf_completion_runtime *runtime
);

/* The process's one record of host descriptors coming back.
 *
 * An open the host refuses for want of a descriptor (`EMFILE`, `ENFILE`) is
 * the one outcome a pipeline can invent.  A schedule that keeps several
 * iterations of a loop in flight holds several descriptors where source order
 * holds one, so a limit the sequential program never reaches turns a correct
 * program's `Ok` into an `Err(ResourceExhausted)`.  [SYS-10] is explicit that
 * a `FilePermit` promises no native descriptor, and [PAR-1]'s exhaustion
 * clause excuses only the resources an implementation spends on overlapping —
 * never the descriptors the program's own opens consume.
 *
 * Deciding that needs one fact no single engine has.  A refused open must ask
 * "is this runtime still holding a descriptor it could give back", and the
 * answer is about the whole process: a read running on a helper thread, a
 * close the kernel ring has not posted yet, and a blocking direct call are all
 * operations that end with a descriptor returned.  An engine that asks only
 * about its own operations answers a different question and publishes an
 * exhaustion that does not exist.  So there is one ledger here, and every
 * target engine reports into it.
 *
 * The rule every refused open follows, whichever engine attempted it:
 *   1. read `wf_completion_descriptor_returns` before the host attempt;
 *   2. if that count has moved when the refusal arrives, a descriptor came
 *      back while the host was refusing — re-attempt once;
 *   3. otherwise, while an operation is in flight anywhere else, wait for the
 *      next descriptor to come back and then re-attempt once;
 *   4. otherwise publish the refusal, which is the outcome source order
 *      produces too.
 * Exactly one re-attempt per refused open.  If the second attempt also fails,
 * that is the program's own outcome and it is published unchanged.
 *
 * The rule turns on two different facts, and a ledger that counts them together
 * gets the rule wrong.  An operation *ending* is what "in flight anywhere else"
 * is about: while one runs it may still give a descriptor back, so the refusal
 * is not the program's outcome yet, and when the last one ends it is.  A
 * descriptor *coming back* is the only event that can make a second host
 * attempt answer differently from the first, so it is the only thing that
 * justifies spending the one re-attempt.  A read, a write, a status or a
 * directory batch ends without returning anything: it leaves the in-flight
 * count and grants no re-attempt.  A close, and an open whose descriptor this
 * runtime obtained and then disposed of, do both.  One count for both facts
 * spends a refused open's single re-attempt on a read that happened to finish,
 * and then publishes the refusal while the close it was waiting for is still in
 * flight — which is the `Ok` this whole ledger exists to keep.
 *
 * Waiting is safe because it terminates: a waiter counts itself out of "in
 * flight anywhere else", so when every operation still in flight is a waiter
 * the earliest of them publishes its refusal, and leaving the waiter order
 * hands that place, and the same answer, to the next one.
 *
 * This lives with the completion core because the core is the one unit every
 * target engine and the bridge link.  The bounded POSIX adapter and the Linux
 * ring are qualified separately and deliberately share no other code.
 */
typedef struct wf_retirement_waiter {
    struct wf_retirement_waiter *next;
    /* The descriptor-return count read before the host attempt this waiter is
     * deciding.  The question is not "is anything running now" — a descriptor
     * returned between the attempt and the wait is one this open never saw —
     * but "has a descriptor come back since the attempt". */
    uint64_t seen;
    /* In-flight work this waiter's own thread is answerable for: the queue a
     * refused adapter open must run, or that its suspended caller must.
     *
     * The ledger asks for it at the moment it decides, and never accepts it
     * as a reading taken earlier.  A decision is made under the retirement
     * lock, which is the lock every wake takes, so a number read before that
     * lock was held can already be out of date when it is used — and a waiter
     * that sleeps on such a number sleeps on work that has already arrived,
     * whose wake it has already missed.  The callback is therefore required
     * to take no lock of its own. */
    size_t (*owed)(void *context);
    void *owed_context;
    /* Whether this waiter's own thread is the engine for that work.  It is
     * the same queue seen from two sides: a thread that will run it must go
     * and run it rather than sleep, and one whose caller is suspended inside
     * that work cannot run any of it, so nothing may wait for it either. */
    int runs_owed;
} wf_retirement_waiter;

enum wf_retirement_state {
    /* A descriptor came back since the attempt: re-attempt once. */
    WF_RETIREMENT_HAPPENED = 0,
    /* An operation is in flight elsewhere: it can still give a descriptor
     * back, so this refusal is not the program's outcome yet. */
    WF_RETIREMENT_AWAITED = 1,
    /* Nothing is left that could retire, and this is the earliest waiter:
     * publish the refusal. */
    WF_RETIREMENT_UNREACHABLE = 2
};

/* How many descriptors this runtime has put back in the host's table, over
 * every engine.  Read before a host attempt, compared after it.  An operation
 * that ends without returning one does not move this, however long it ran and
 * whichever engine ran it. */
uint64_t wf_completion_descriptor_returns(void);

/* One operation an engine has accepted and will run.  Queued bounded-adapter
 * work counts from submission, because a queue with an engine behind it is
 * work that will retire; a ring operation counts from the moment the kernel
 * has it; a blocking direct call counts while it executes. */
void wf_completion_operation_accepted(void);

/* One accepted operation has published its result and given back whatever the
 * host lent it.  Called after the terminal publication, never before.
 *
 * `returned_a_descriptor` is the answer to the one question the rule above asks
 * of an ending operation: is there a descriptor in the host's table now that
 * this operation was holding?  A close that ran says yes, and so does an open
 * whose descriptor this runtime obtained and then disposed of — the kind check
 * refused it, so the close the runtime made for it is a descriptor coming back.
 * An open that the host refused, an open whose descriptor the program now
 * holds, and every transfer, status and directory batch say no: they end, which
 * the in-flight count records, but they return nothing, which is what the one
 * re-attempt may be spent on.  Every engine answers it from its own record of
 * the operation, because it is a fact about the operation and not about the
 * engine. */
void wf_completion_operation_retired(int returned_a_descriptor);

/* Opens this runtime held rather than published, over every route.  A test
 * standing at that moment reads this; nothing in the runtime decides on it. */
uint64_t wf_completion_retirement_waits(void);

/* How many refused opens are waiting for a retirement right now.
 *
 * An engine reads this after it retires an operation, to decide whether the
 * retirement is worth announcing on the scheduler's own wake endpoint.  It is
 * worth it exactly when something is waiting: a held open is released by some
 * thread asking the ledger again, and on the ordinary path — nothing refused,
 * nobody waiting — this is one atomic load and no wake at all. */
size_t wf_completion_retirement_waiters(void);

/* Enters and leaves the process-wide waiter order.  Every refused open that
 * does not publish immediately is registered, whether a thread is blocked on
 * it (the bounded adapter, a direct call) or an entry is holding it (the
 * ring), because the give-up decision is about all of them together. */
void wf_completion_retirement_wait_begin(
    wf_retirement_waiter *waiter,
    uint64_t seen,
    size_t (*owed)(void *context),
    void *owed_context,
    int runs_owed
);
void wf_completion_retirement_wait_end(wf_retirement_waiter *waiter);

/* An operation whose retirement is blocked because the thread that owns it is
 * running another operation first — a refused open running the work its own
 * adapter still owes, or a direct call driving the engines while it waits.  It
 * cannot give its descriptor back until that finishes, so nothing may wait for
 * it to: a waiter that counted it would wait for a thread that is waiting for
 * the waiter.  Told to the ledger for exactly as long as that holds. */
void wf_completion_retirement_defer_begin(void);
void wf_completion_retirement_defer_end(void);

/* Where this waiter stands, without blocking.
 *
 * "In flight *elsewhere*" is what the answer turns on, and its local half is
 * the waiter's own `owed` above: a held ring entry blocks no thread and owes
 * nothing, an adapter open that runs its own queue owes that queue and must
 * run it before it decides, and one running as owed work owes the queue its
 * suspended caller cannot reach.  The global half — every operation deferred
 * above — the ledger subtracts itself. */
enum wf_retirement_state wf_completion_retirement_state(
    const wf_retirement_waiter *waiter
);

/* Sleeps until the ledger changes, rechecking the state under the same lock
 * the retirement wake takes so no wake is lost.  Every input to that recheck
 * is read inside that lock, `owed` included, which is what makes the wake
 * unmissable rather than merely unlikely to be missed.  Callers loop on
 * `wf_completion_retirement_state`; this never spins. */
void wf_completion_retirement_sleep(
    const wf_retirement_waiter *waiter
);

_Static_assert(sizeof(wf_completion_token) == 16u, "completion token ABI");
_Static_assert(alignof(wf_completion_token) >= alignof(uint64_t), "completion token alignment");

#if defined(__cplusplus)
}
#endif

#endif
