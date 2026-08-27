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

_Static_assert(sizeof(wf_completion_token) == 16u, "completion token ABI");
_Static_assert(alignof(wf_completion_token) >= alignof(uint64_t), "completion token alignment");

#if defined(__cplusplus)
}
#endif

#endif
