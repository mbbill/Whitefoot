#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include "contract.h"

#include <errno.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Fixed scheduler injection. The completion core publishes only an opaque
 * frame. A link which registers no frame dependency needs no scheduler unit. */
__attribute__((weak)) void wf__writer_scheduler_ready(void *frame) {
    (void)frame;
    abort();
}

static void wf_completion_notify_scheduler(wf_completion_runtime *runtime);

static int wf_completion_token_slot(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    wf_completion_slot **slot
) {
    if (runtime == NULL || slot == NULL || token.slot >= runtime->slot_count) {
        return 0;
    }
    *slot = &runtime->slots[token.slot];
    return 1;
}

static void wf_completion_clear_statistics(wf_completion_runtime *runtime) {
    atomic_init(&runtime->stat_claims, 0);
    atomic_init(&runtime->stat_claim_capacity_waits, 0);
    atomic_init(&runtime->stat_target_capacity_waits, 0);
    atomic_init(&runtime->stat_publications, 0);
    atomic_init(&runtime->stat_stale_publications, 0);
    atomic_init(&runtime->stat_duplicate_terminals, 0);
    atomic_init(&runtime->stat_drained_events, 0);
    atomic_init(&runtime->stat_consumptions, 0);
    atomic_init(&runtime->stat_parks, 0);
    atomic_init(&runtime->stat_wake_signals, 0);
    atomic_init(&runtime->stat_compute_notifications, 0);
    atomic_init(&runtime->stat_target_notifications, 0);
    atomic_init(&runtime->stat_capacity_notifications, 0);
}

int wf_completion_runtime_init(
    wf_completion_runtime *runtime,
    wf_completion_slot *slots,
    size_t slot_count
) {
    size_t initialized = 0;
    int error;

    if (runtime == NULL || slots == NULL || slot_count == 0
        || slot_count > UINT32_MAX) {
        return EINVAL;
    }

    memset(runtime, 0, sizeof(*runtime));
    runtime->slots = slots;
    runtime->slot_count = slot_count;
    atomic_init(&runtime->claim_cursor, 0);
    atomic_init(&runtime->drain_cursor, 0);
    atomic_init(&runtime->ready_events, 0);
    atomic_init(&runtime->wake_epoch, 0);
    atomic_init(&runtime->parked_schedulers, 0);
    runtime->wake_callback = NULL;
    runtime->wake_context = NULL;
    wf_completion_clear_statistics(runtime);

    error = pthread_mutex_init(&runtime->wake_lock, NULL);
    if (error != 0) {
        return error;
    }
    error = pthread_cond_init(&runtime->wake_condition, NULL);
    if (error != 0) {
        (void)pthread_mutex_destroy(&runtime->wake_lock);
        return error;
    }

    for (initialized = 0; initialized < slot_count; ++initialized) {
        wf_completion_slot *slot = &slots[initialized];
        memset(slot, 0, sizeof(*slot));
        error = pthread_mutex_init(&slot->publication_lock, NULL);
        if (error != 0) {
            size_t rollback;
            for (rollback = 0; rollback < initialized; ++rollback) {
                (void)pthread_mutex_destroy(&slots[rollback].publication_lock);
            }
            (void)pthread_cond_destroy(&runtime->wake_condition);
            (void)pthread_mutex_destroy(&runtime->wake_lock);
            return error;
        }
        atomic_init(&slot->generation, 0);
        atomic_init(&slot->phase, WF_COMPLETION_FREE);
        atomic_init(&slot->milestones, 0);
        atomic_init(&slot->event_pending, 0);
        atomic_init(&slot->event_drained, 0);
        slot->consume_waiting = 0;
        slot->dependent_frame = NULL;
        slot->dependent_requirement = 0;
    }
    return 0;
}

int wf_completion_set_wake_callback(
    wf_completion_runtime *runtime,
    wf_completion_wake_callback wake,
    void *context
) {
    if (runtime == NULL || runtime->slots == NULL || wake == NULL) {
        return EINVAL;
    }
    if (runtime->wake_callback != NULL) {
        return EBUSY;
    }
    runtime->wake_context = context;
    runtime->wake_callback = wake;
    return 0;
}

int wf_completion_runtime_destroy(wf_completion_runtime *runtime) {
    size_t index;
    int first_error = 0;

    if (runtime == NULL || runtime->slots == NULL || runtime->slot_count == 0) {
        return EINVAL;
    }
    if (atomic_load_explicit(&runtime->parked_schedulers, memory_order_acquire) != 0
        || atomic_load_explicit(&runtime->ready_events, memory_order_acquire) != 0) {
        return EBUSY;
    }
    for (index = 0; index < runtime->slot_count; ++index) {
        unsigned phase = atomic_load_explicit(
            &runtime->slots[index].phase,
            memory_order_acquire
        );
        if (phase != WF_COMPLETION_FREE && phase != WF_COMPLETION_RETIRED) {
            return EBUSY;
        }
    }
    for (index = 0; index < runtime->slot_count; ++index) {
        int error = pthread_mutex_destroy(&runtime->slots[index].publication_lock);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    {
        int error = pthread_cond_destroy(&runtime->wake_condition);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    {
        int error = pthread_mutex_destroy(&runtime->wake_lock);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    if (first_error == 0) {
        runtime->slots = NULL;
        runtime->slot_count = 0;
    }
    return first_error;
}

enum wf_completion_claim_result wf_completion_claim(
    wf_completion_runtime *runtime,
    wf_completion_token *token
) {
    size_t offset;
    size_t index;
    size_t start;

    if (runtime == NULL || token == NULL || runtime->slots == NULL
        || runtime->slot_count == 0) {
        return WF_COMPLETION_CLAIM_INVALID;
    }
    start = atomic_fetch_add_explicit(
        &runtime->claim_cursor,
        1,
        memory_order_relaxed
    );
    index = start % runtime->slot_count;
    for (offset = 0; offset < runtime->slot_count; ++offset) {
        wf_completion_slot *slot = &runtime->slots[index];
        uint64_t generation;
        unsigned phase;

        (void)pthread_mutex_lock(&slot->publication_lock);
        phase = atomic_load_explicit(&slot->phase, memory_order_relaxed);
        if (phase != WF_COMPLETION_FREE) {
            (void)pthread_mutex_unlock(&slot->publication_lock);
            index = index + 1 == runtime->slot_count ? 0 : index + 1;
            continue;
        }
        generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
        if (generation == UINT64_MAX) {
            atomic_store_explicit(
                &slot->phase,
                WF_COMPLETION_RETIRED,
                memory_order_release
            );
            (void)pthread_mutex_unlock(&slot->publication_lock);
            index = index + 1 == runtime->slot_count ? 0 : index + 1;
            continue;
        }
        generation += 1;
        slot->terminal_kind = 0;
        slot->adapter_tag = 0;
        slot->result_size = 0;
        slot->dependent_frame = NULL;
        slot->dependent_requirement = 0;
        atomic_store_explicit(&slot->milestones, 0, memory_order_relaxed);
        atomic_store_explicit(&slot->event_pending, 0, memory_order_relaxed);
        atomic_store_explicit(&slot->event_drained, 0, memory_order_relaxed);
        slot->consume_waiting = 0;
        atomic_store_explicit(&slot->generation, generation, memory_order_relaxed);
        atomic_store_explicit(&slot->phase, WF_COMPLETION_READY, memory_order_release);
        (void)pthread_mutex_unlock(&slot->publication_lock);

        token->slot = (uint32_t)index;
        token->generation = generation;
        atomic_fetch_add_explicit(&runtime->stat_claims, 1, memory_order_relaxed);
        return WF_COMPLETION_CLAIMED;
    }

    atomic_fetch_add_explicit(
        &runtime->stat_claim_capacity_waits,
        1,
        memory_order_relaxed
    );
    return WF_COMPLETION_CLAIM_WAIT_CAPACITY;
}

static enum wf_completion_transition_result wf_completion_transition(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    unsigned first_expected,
    unsigned second_expected,
    unsigned desired
) {
    wf_completion_slot *slot;
    uint64_t generation;
    unsigned phase;

    if (!wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_TRANSITION_STALE;
    }
    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    if (generation != token.generation) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_STALE;
    }
    phase = atomic_load_explicit(&slot->phase, memory_order_relaxed);
    if (phase != first_expected && phase != second_expected) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_INVALID_STATE;
    }
    atomic_store_explicit(&slot->phase, desired, memory_order_release);
    (void)pthread_mutex_unlock(&slot->publication_lock);
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_transition_result wf_completion_begin_submit(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    return wf_completion_transition(
        runtime,
        token,
        WF_COMPLETION_READY,
        WF_COMPLETION_WAIT_CAPACITY,
        WF_COMPLETION_SUBMITTING
    );
}

enum wf_completion_transition_result wf_completion_mark_wait_capacity(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    enum wf_completion_transition_result result = wf_completion_transition(
        runtime,
        token,
        WF_COMPLETION_SUBMITTING,
        WF_COMPLETION_SUBMITTING,
        WF_COMPLETION_WAIT_CAPACITY
    );
    if (result == WF_COMPLETION_TRANSITIONED) {
        atomic_fetch_add_explicit(
            &runtime->stat_target_capacity_waits,
            1,
            memory_order_relaxed
        );
    }
    return result;
}

enum wf_completion_transition_result wf_completion_target_accepted(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    return wf_completion_transition(
        runtime,
        token,
        WF_COMPLETION_SUBMITTING,
        WF_COMPLETION_SUBMITTING,
        WF_COMPLETION_IN_FLIGHT
    );
}

enum wf_completion_transition_result wf_completion_set_adapter_tag(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t adapter_tag
) {
    wf_completion_slot *slot;
    uint64_t generation;
    unsigned phase;
    if (adapter_tag == 0 || !wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_TRANSITION_STALE;
    }
    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    phase = atomic_load_explicit(&slot->phase, memory_order_relaxed);
    if (generation != token.generation) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_STALE;
    }
    if (phase != WF_COMPLETION_READY && phase != WF_COMPLETION_WAIT_CAPACITY
        && phase != WF_COMPLETION_SUBMITTING) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_INVALID_STATE;
    }
    slot->adapter_tag = adapter_tag;
    (void)pthread_mutex_unlock(&slot->publication_lock);
    return WF_COMPLETION_TRANSITIONED;
}

/* Announces one epoch change, and takes the wake lock only when there is a
 * sleeper to wake.
 *
 * The lock is what used to order "raise the epoch, then look for a sleeper"
 * against a scheduler's "announce sleep, then look at the epoch".  Taking it
 * on every publication made a global mutex out of a path a program crosses
 * three times per file operation — once at submission, once at publication,
 * once at consumption — and on a host where a contended mutex is a system
 * call that was the largest single cost of the Darwin helper path.
 *
 * The two stores and the two loads are sequentially consistent instead, which
 * is the same exclusion Dekker's algorithm gets without a lock: this thread
 * raises the epoch and then reads the sleeper count; a parking scheduler
 * raises the sleeper count and then reads the epoch.  Both cannot read the
 * old value, so either this call sees a sleeper and takes the lock and wakes
 * it, or the scheduler sees the new epoch and does not sleep at all.  Both
 * sides must stay sequentially consistent for that to hold, which is why the
 * two park paths — this core's and the Linux target's external wait — name
 * the order explicitly at their own increment and recheck. */
static void wf_completion_notify_scheduler(wf_completion_runtime *runtime) {
    atomic_fetch_add_explicit(&runtime->wake_epoch, 1, memory_order_seq_cst);
    if (atomic_load_explicit(&runtime->parked_schedulers, memory_order_seq_cst)
        == 0) {
        return;
    }
    (void)pthread_mutex_lock(&runtime->wake_lock);
    {
        unsigned parked = atomic_load_explicit(
            &runtime->parked_schedulers,
            memory_order_relaxed
        );
        if (parked != 0) {
            /* An active scheduler observes the epoch or ready source directly.
             * Only an announced sleeper needs an explicit host wake. External
             * target waits increment parked_schedulers under this same lock,
             * closing their empty-check -> sleep race. */
            if (runtime->wake_callback != NULL) {
                runtime->wake_callback(runtime->wake_context);
            }
            /* One waiter needs one signal. Two or more captured the same old
             * global epoch, so all of them must recheck after this transition. */
            if (parked == 1) {
                (void)pthread_cond_signal(&runtime->wake_condition);
            } else {
                (void)pthread_cond_broadcast(&runtime->wake_condition);
            }
            atomic_fetch_add_explicit(
                &runtime->stat_wake_signals,
                1,
                memory_order_relaxed
            );
        }
    }
    (void)pthread_mutex_unlock(&runtime->wake_lock);
}

static enum wf_completion_publish_result wf_completion_publish(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication,
    unsigned expected_phase
) {
    wf_completion_slot *slot;
    uint64_t generation;
    unsigned phase;

    if (publication == NULL || publication->terminal_kind == 0
        || (publication->result_size != 0 && publication->result == NULL)) {
        return WF_COMPLETION_PUBLISH_INVALID_ARGUMENT;
    }
    if (publication->result_size > WF_COMPLETION_RESULT_CAPACITY) {
        return WF_COMPLETION_PUBLISH_RESULT_TOO_LARGE;
    }
    if ((publication->milestones & WF_COMPLETION_OWNERSHIP_COMPLETE)
        != WF_COMPLETION_OWNERSHIP_COMPLETE) {
        return WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL;
    }
    if (!wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_PUBLISH_STALE;
    }

    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    if (generation != token.generation) {
        atomic_fetch_add_explicit(
            &runtime->stat_stale_publications,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_STALE;
    }
    phase = atomic_load_explicit(&slot->phase, memory_order_relaxed);
    if (phase == WF_COMPLETION_TERMINAL_PHASE) {
        atomic_fetch_add_explicit(
            &runtime->stat_duplicate_terminals,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL;
    }
    if (phase != expected_phase) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_INVALID_STATE;
    }

    /* Generation and phase have both been validated while reuse is excluded.
     * Only now may result bytes belonging to this logical operation change. */
    if (publication->result_size != 0) {
        memcpy(
            slot->result.bytes,
            publication->result,
            publication->result_size
        );
    }
    slot->result_size = publication->result_size;
    slot->terminal_kind = publication->terminal_kind;

    /* Result bytes precede the release-published milestone product.  A store
     * rather than an accumulation: the terminal product was checked complete
     * above, so it already contains every fact
     * `wf_completion_publish_milestone` may have published earlier. */
    atomic_store_explicit(
        &slot->milestones,
        publication->milestones,
        memory_order_release
    );
    atomic_store_explicit(
        &slot->phase,
        WF_COMPLETION_TERMINAL_PHASE,
        memory_order_release
    );
    atomic_store_explicit(&slot->event_pending, 1, memory_order_release);
    atomic_fetch_add_explicit(&runtime->ready_events, 1, memory_order_release);
    atomic_fetch_add_explicit(
        &runtime->stat_publications,
        1,
        memory_order_relaxed
    );
    (void)pthread_mutex_unlock(&slot->publication_lock);
    wf_completion_notify_scheduler(runtime);
    return WF_COMPLETION_PUBLISHED;
}

enum wf_completion_publish_result wf_completion_publish_milestone(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t milestones
) {
    wf_completion_slot *slot;
    uint64_t generation;
    unsigned phase;

    /* Only facts of this operation, and never the terminal one: a terminal
     * fact without the result bytes it stands for would let a consumer read a
     * slot that has nothing in it. */
    if (milestones == 0
        || (milestones & ~(uint32_t)WF_COMPLETION_OWNERSHIP_COMPLETE) != 0
        || (milestones & (uint32_t)WF_COMPLETION_TERMINAL) != 0) {
        return WF_COMPLETION_PUBLISH_INVALID_ARGUMENT;
    }
    if (!wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_PUBLISH_STALE;
    }

    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    if (generation != token.generation) {
        atomic_fetch_add_explicit(
            &runtime->stat_stale_publications,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_STALE;
    }
    phase = atomic_load_explicit(&slot->phase, memory_order_relaxed);
    if (phase == WF_COMPLETION_TERMINAL_PHASE) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL;
    }
    if (phase != WF_COMPLETION_SUBMITTING && phase != WF_COMPLETION_IN_FLIGHT) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_INVALID_STATE;
    }
    /* Accumulated, not stored: this route publishes one fact of a product the
     * terminal route later completes, and the terminal product is a superset
     * of everything published here. */
    atomic_fetch_or_explicit(
        &slot->milestones,
        milestones,
        memory_order_release
    );
    (void)pthread_mutex_unlock(&slot->publication_lock);
    return WF_COMPLETION_PUBLISHED;
}

enum wf_completion_publish_result wf_completion_publish_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
) {
    return wf_completion_publish(
        runtime,
        token,
        publication,
        WF_COMPLETION_IN_FLIGHT
    );
}

enum wf_completion_publish_result wf_completion_publish_inline_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
) {
    return wf_completion_publish(
        runtime,
        token,
        publication,
        WF_COMPLETION_SUBMITTING
    );
}

size_t wf_completion_drain(
    wf_completion_runtime *runtime,
    wf_completion_event *events,
    size_t event_capacity,
    size_t scan_budget
) {
    size_t scanned = 0;
    size_t produced = 0;
    int wake_consumer = 0;
    size_t cursor;
    size_t index;

    if (runtime == NULL || runtime->slots == NULL || events == NULL
        || event_capacity == 0 || scan_budget == 0) {
        return 0;
    }
    /* Nothing published means nothing to find, and finding nothing used to
     * cost a compare-exchange on every slot in the scanned window.  The count
     * is durable: a publisher raises it before announcing the epoch, so a
     * scheduler that reads zero here and then parks is parking against the
     * epoch it snapshotted before this call. */
    if (atomic_load_explicit(&runtime->ready_events, memory_order_acquire)
        == 0) {
        return 0;
    }
    cursor = atomic_fetch_add_explicit(
        &runtime->drain_cursor,
        scan_budget,
        memory_order_relaxed
    );
    index = cursor % runtime->slot_count;
    while (scanned < scan_budget && produced < event_capacity) {
        size_t slot_index = index;
        wf_completion_slot *slot = &runtime->slots[slot_index];
        void *dependent_frame = NULL;
        unsigned pending = 1;
        scanned += 1;
        index = index + 1 == runtime->slot_count ? 0 : index + 1;
        if (!atomic_compare_exchange_strong_explicit(
                &slot->event_pending,
                &pending,
                0,
                memory_order_acquire,
                memory_order_relaxed
            )) {
            continue;
        }

        (void)pthread_mutex_lock(&slot->publication_lock);
        events[produced].token.slot = (uint32_t)slot_index;
        events[produced].token.generation = atomic_load_explicit(
            &slot->generation,
            memory_order_relaxed
        );
        events[produced].milestones = atomic_load_explicit(
            &slot->milestones,
            memory_order_acquire
        );
        events[produced].terminal_kind = slot->terminal_kind;
        if (slot->dependent_frame != NULL
            && (events[produced].milestones & slot->dependent_requirement)
                == slot->dependent_requirement) {
            dependent_frame = slot->dependent_frame;
            slot->dependent_frame = NULL;
            slot->dependent_requirement = 0;
        }
        atomic_store_explicit(&slot->event_drained, 1, memory_order_release);
        if (slot->consume_waiting != 0) {
            slot->consume_waiting = 0;
            wake_consumer = 1;
        }
        (void)pthread_mutex_unlock(&slot->publication_lock);

        atomic_fetch_sub_explicit(&runtime->ready_events, 1, memory_order_acq_rel);
        atomic_fetch_add_explicit(
            &runtime->stat_drained_events,
            1,
            memory_order_relaxed
        );
        /* A writer frame becomes runnable only after its exact event is
         * scheduler-owned. Its resume can therefore consume immediately. */
        if (dependent_frame != NULL) {
            wf__writer_scheduler_ready(dependent_frame);
        }
        produced += 1;
    }
    if (wake_consumer != 0) {
        /* Only a token owner which completed the final recheck-to-park
         * handshake needs this second publication. Uncontended drain remains
         * a local state change. */
        wf_completion_notify_scheduler(runtime);
    }
    return produced;
}

size_t wf_completion_drain_token(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    wf_completion_event *event
) {
    wf_completion_slot *slot;
    void *dependent_frame = NULL;
    int wake_consumer = 0;
    unsigned pending = 1;

    if (event == NULL || !wf_completion_token_slot(runtime, token, &slot)) {
        return 0;
    }
    /* The cheap negative first: a join loop asks on every turn, and a slot
     * with no pending event is the answer nearly every time.  A relaxed load
     * is enough to decide there is nothing to take, because the taking itself
     * is decided again below. */
    if (atomic_load_explicit(&slot->event_pending, memory_order_relaxed) == 0) {
        return 0;
    }
    /* Generation and the take are decided together, under the publication
     * lock.  Naming a slot is not naming an operation: the slot outlives the
     * operation that used it, so a token whose operation has already ended
     * names whatever operation reused its slot.  `wf_completion_claim` and
     * `wf_completion_publish` are the only other writers of both the
     * generation and the pending flag, and both hold this lock, so checking
     * the generation here excludes taking a live operation's event with a
     * token that no longer stands for it.  Every other token-named entry
     * makes the same check; this one is the sweep's transition, so it kept
     * the sweep's generation-agnostic shape by mistake. */
    (void)pthread_mutex_lock(&slot->publication_lock);
    if (atomic_load_explicit(&slot->generation, memory_order_relaxed)
        != token.generation) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return 0;
    }
    if (!atomic_compare_exchange_strong_explicit(
            &slot->event_pending,
            &pending,
            0,
            memory_order_acquire,
            memory_order_relaxed
        )) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return 0;
    }

    event->token.slot = token.slot;
    event->token.generation = atomic_load_explicit(
        &slot->generation,
        memory_order_relaxed
    );
    event->milestones = atomic_load_explicit(
        &slot->milestones,
        memory_order_acquire
    );
    event->terminal_kind = slot->terminal_kind;
    if (slot->dependent_frame != NULL
        && (event->milestones & slot->dependent_requirement)
            == slot->dependent_requirement) {
        dependent_frame = slot->dependent_frame;
        slot->dependent_frame = NULL;
        slot->dependent_requirement = 0;
    }
    atomic_store_explicit(&slot->event_drained, 1, memory_order_release);
    if (slot->consume_waiting != 0) {
        slot->consume_waiting = 0;
        wake_consumer = 1;
    }
    (void)pthread_mutex_unlock(&slot->publication_lock);

    atomic_fetch_sub_explicit(&runtime->ready_events, 1, memory_order_acq_rel);
    atomic_fetch_add_explicit(
        &runtime->stat_drained_events,
        1,
        memory_order_relaxed
    );
    if (dependent_frame != NULL) {
        wf__writer_scheduler_ready(dependent_frame);
    }
    if (wake_consumer != 0) {
        wf_completion_notify_scheduler(runtime);
    }
    return 1;
}

size_t wf_completion_ready_event_count(const wf_completion_runtime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    return atomic_load_explicit(&runtime->ready_events, memory_order_acquire);
}

enum wf_completion_transition_result wf_completion_observe(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t *milestones,
    unsigned *phase
) {
    wf_completion_slot *slot;
    uint64_t generation;

    if (milestones == NULL || phase == NULL
        || !wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_TRANSITION_STALE;
    }
    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    if (generation != token.generation) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_STALE;
    }
    *milestones = atomic_load_explicit(&slot->milestones, memory_order_acquire);
    *phase = atomic_load_explicit(&slot->phase, memory_order_acquire);
    (void)pthread_mutex_unlock(&slot->publication_lock);
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_depend_result wf_completion_depend(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t requirement,
    void *frame
) {
    wf_completion_slot *slot;
    uint64_t generation;
    uint32_t milestones;
    unsigned phase;
    int already_ready = 0;

    if (requirement == 0 || frame == NULL || runtime == NULL
        || !wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_DEPEND_INVALID_ARGUMENT;
    }
    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    if (generation != token.generation) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_DEPEND_STALE;
    }
    if (slot->dependent_frame != NULL) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_DEPEND_DUPLICATE;
    }
    phase = atomic_load_explicit(&slot->phase, memory_order_acquire);
    if (phase == WF_COMPLETION_FREE || phase == WF_COMPLETION_RETIRED) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_DEPEND_STALE;
    }
    milestones = atomic_load_explicit(&slot->milestones, memory_order_acquire);
    if ((milestones & requirement) == requirement
        && atomic_load_explicit(
               &slot->event_drained,
               memory_order_acquire
           ) != 0) {
        already_ready = 1;
    } else {
        slot->dependent_frame = frame;
        slot->dependent_requirement = requirement;
    }
    (void)pthread_mutex_unlock(&slot->publication_lock);
    if (already_ready) {
        wf__writer_scheduler_ready(frame);
        return WF_COMPLETION_DEPEND_ALREADY_READY;
    }
    return WF_COMPLETION_DEPEND_REGISTERED;
}

enum wf_completion_consume_result wf_completion_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    void *result,
    size_t result_capacity,
    wf_completion_outcome *outcome
) {
    wf_completion_slot *slot;
    uint64_t generation;
    unsigned phase;

    if (outcome == NULL || (result_capacity != 0 && result == NULL)
        || !wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_CONSUME_INVALID_ARGUMENT;
    }
    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    if (generation != token.generation) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_STALE;
    }
    phase = atomic_load_explicit(&slot->phase, memory_order_acquire);
    if (phase != WF_COMPLETION_TERMINAL_PHASE) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_NOT_TERMINAL;
    }
    if (atomic_load_explicit(&slot->event_drained, memory_order_acquire) == 0) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_NOT_DRAINED;
    }
    if (result_capacity < slot->result_size) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_RESULT_TOO_LARGE;
    }
    if (slot->result_size != 0) {
        memcpy(result, slot->result.bytes, slot->result_size);
    }
    outcome->milestones = atomic_load_explicit(
        &slot->milestones,
        memory_order_acquire
    );
    outcome->terminal_kind = slot->terminal_kind;
    outcome->adapter_tag = slot->adapter_tag;
    outcome->result_size = slot->result_size;

    slot->terminal_kind = 0;
    slot->adapter_tag = 0;
    slot->result_size = 0;
    slot->dependent_frame = NULL;
    slot->dependent_requirement = 0;
    slot->consume_waiting = 0;
    atomic_store_explicit(&slot->milestones, 0, memory_order_relaxed);
    atomic_store_explicit(&slot->event_drained, 0, memory_order_relaxed);
    atomic_store_explicit(&slot->phase, WF_COMPLETION_FREE, memory_order_release);
    (void)pthread_mutex_unlock(&slot->publication_lock);
    atomic_fetch_add_explicit(
        &runtime->stat_consumptions,
        1,
        memory_order_relaxed
    );
    /* A slot-capacity waiter may live on another scheduler lane. */
    wf_completion_notify_capacity(runtime);
    return WF_COMPLETION_CONSUMED;
}

enum wf_completion_consume_wait_result wf_completion_wait_to_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    wf_completion_slot *slot;
    uint64_t generation;
    unsigned phase;
    unsigned drained;
    if (!wf_completion_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_CONSUME_WAIT_STALE;
    }
    (void)pthread_mutex_lock(&slot->publication_lock);
    generation = atomic_load_explicit(&slot->generation, memory_order_relaxed);
    phase = atomic_load_explicit(&slot->phase, memory_order_acquire);
    drained = atomic_load_explicit(&slot->event_drained, memory_order_acquire);
    if (generation != token.generation || phase == WF_COMPLETION_FREE
        || phase == WF_COMPLETION_RETIRED) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_WAIT_STALE;
    }
    if (phase == WF_COMPLETION_TERMINAL_PHASE && drained != 0) {
        (void)pthread_mutex_unlock(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_READY;
    }
    slot->consume_waiting = 1;
    (void)pthread_mutex_unlock(&slot->publication_lock);
    return WF_COMPLETION_CONSUME_WAIT_REGISTERED;
}

uint64_t wf_completion_wake_epoch(const wf_completion_runtime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    return atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire);
}

void wf_completion_notify_compute(wf_completion_runtime *runtime) {
    if (runtime == NULL) {
        return;
    }
    atomic_fetch_add_explicit(
        &runtime->stat_compute_notifications,
        1,
        memory_order_relaxed
    );
    wf_completion_notify_scheduler(runtime);
}

void wf_completion_notify_target(wf_completion_runtime *runtime) {
    if (runtime == NULL) {
        return;
    }
    atomic_fetch_add_explicit(
        &runtime->stat_target_notifications,
        1,
        memory_order_relaxed
    );
    wf_completion_notify_scheduler(runtime);
}

void wf_completion_notify_capacity(wf_completion_runtime *runtime) {
    if (runtime == NULL) {
        return;
    }
    atomic_fetch_add_explicit(
        &runtime->stat_capacity_notifications,
        1,
        memory_order_relaxed
    );
    wf_completion_notify_scheduler(runtime);
}

static struct timespec wf_completion_deadline(uint32_t milliseconds) {
    struct timespec deadline;
    uint64_t nanoseconds;
    (void)clock_gettime(CLOCK_REALTIME, &deadline);
    nanoseconds = (uint64_t)deadline.tv_nsec
        + (uint64_t)(milliseconds % 1000u) * 1000000u;
    deadline.tv_sec += (time_t)(milliseconds / 1000u)
        + (time_t)(nanoseconds / 1000000000u);
    deadline.tv_nsec = (long)(nanoseconds % 1000000000u);
    return deadline;
}

enum wf_completion_park_result wf_completion_park_if_unchanged(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
) {
    int error = 0;
    struct timespec deadline = {0, 0};

    if (runtime == NULL) {
        return WF_COMPLETION_PARK_FAILED;
    }
    if (timeout_milliseconds != UINT32_MAX) {
        deadline = wf_completion_deadline(timeout_milliseconds);
    }

    error = pthread_mutex_lock(&runtime->wake_lock);
    if (error != 0) {
        return WF_COMPLETION_PARK_FAILED;
    }
    if (atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire)
        != observed_epoch) {
        (void)pthread_mutex_unlock(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }

    atomic_fetch_add_explicit(
        &runtime->parked_schedulers,
        1,
        memory_order_seq_cst
    );
    atomic_fetch_add_explicit(&runtime->stat_parks, 1, memory_order_relaxed);

    /* This second observation closes announce-sleep -> sleep against a
     * publisher which raised the epoch and then looked for a sleeper.  It is
     * sequentially consistent, and so is the increment above it, because that
     * pair against the publisher's own pair is the whole of what keeps a wake
     * from being lost now that the publisher no longer takes this lock. */
    if (atomic_load_explicit(&runtime->wake_epoch, memory_order_seq_cst)
        != observed_epoch) {
        atomic_fetch_sub_explicit(
            &runtime->parked_schedulers,
            1,
            memory_order_relaxed
        );
        (void)pthread_mutex_unlock(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }

    while (atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire)
           == observed_epoch) {
        if (timeout_milliseconds == UINT32_MAX) {
            error = pthread_cond_wait(
                &runtime->wake_condition,
                &runtime->wake_lock
            );
        } else {
            error = pthread_cond_timedwait(
                &runtime->wake_condition,
                &runtime->wake_lock,
                &deadline
            );
        }
        if (error == ETIMEDOUT || error != 0) {
            break;
        }
    }
    atomic_fetch_sub_explicit(
        &runtime->parked_schedulers,
        1,
        memory_order_relaxed
    );
    (void)pthread_mutex_unlock(&runtime->wake_lock);

    if (error == ETIMEDOUT) {
        return WF_COMPLETION_PARK_TIMED_OUT;
    }
    if (error != 0) {
        return WF_COMPLETION_PARK_FAILED;
    }
    return WF_COMPLETION_PARK_WOKEN;
}

unsigned wf_completion_parked_scheduler_count(
    const wf_completion_runtime *runtime
) {
    if (runtime == NULL) {
        return 0;
    }
    return atomic_load_explicit(
        &runtime->parked_schedulers,
        memory_order_acquire
    );
}

wf_completion_statistics wf_completion_statistics_snapshot(
    const wf_completion_runtime *runtime
) {
    wf_completion_statistics statistics = {0};
    if (runtime == NULL) {
        return statistics;
    }
    statistics.claims = atomic_load_explicit(
        &runtime->stat_claims,
        memory_order_relaxed
    );
    statistics.claim_capacity_waits = atomic_load_explicit(
        &runtime->stat_claim_capacity_waits,
        memory_order_relaxed
    );
    statistics.target_capacity_waits = atomic_load_explicit(
        &runtime->stat_target_capacity_waits,
        memory_order_relaxed
    );
    statistics.publications = atomic_load_explicit(
        &runtime->stat_publications,
        memory_order_relaxed
    );
    statistics.stale_publications = atomic_load_explicit(
        &runtime->stat_stale_publications,
        memory_order_relaxed
    );
    statistics.duplicate_terminals = atomic_load_explicit(
        &runtime->stat_duplicate_terminals,
        memory_order_relaxed
    );
    statistics.drained_events = atomic_load_explicit(
        &runtime->stat_drained_events,
        memory_order_relaxed
    );
    statistics.consumptions = atomic_load_explicit(
        &runtime->stat_consumptions,
        memory_order_relaxed
    );
    statistics.parks = atomic_load_explicit(
        &runtime->stat_parks,
        memory_order_relaxed
    );
    statistics.wake_signals = atomic_load_explicit(
        &runtime->stat_wake_signals,
        memory_order_relaxed
    );
    statistics.compute_notifications = atomic_load_explicit(
        &runtime->stat_compute_notifications,
        memory_order_relaxed
    );
    statistics.target_notifications = atomic_load_explicit(
        &runtime->stat_target_notifications,
        memory_order_relaxed
    );
    statistics.capacity_notifications = atomic_load_explicit(
        &runtime->stat_capacity_notifications,
        memory_order_relaxed
    );
    return statistics;
}
