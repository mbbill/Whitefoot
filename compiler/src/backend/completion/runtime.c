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

/* ------------------------------------------------------------------------
 * The process's one retirement ledger (contract.h).
 *
 * Every target engine reports two things here and asks one question of them,
 * which is what makes the exhaustion rule one rule rather than one per
 * engine.  The counters are lock-free because they sit on the submission and
 * publication path of every operation; the lock exists only so a waiter can
 * sleep and a retirement can wake it.
 * ---------------------------------------------------------------------- */

/* A named point between the two reads a give-up decision makes.
 *
 * The decision reads the descriptor-return count, then the in-flight and
 * waiter counts.  A retirement whose return increment and whose in-flight
 * decrement both land between those reads is the one schedule that can make a
 * waiter answer from a world where nothing came back when something did, and
 * the re-read below is what closes it.  Naming the point is what lets a test
 * stand exactly there instead of racing for it, exactly as the adapter's
 * `openat` is named; the default expansion is nothing at all. */
#if !defined(WF_COMPLETION_RETIREMENT_POINT)
#define WF_COMPLETION_RETIREMENT_POINT wf_completion_retirement_point_absent
static void wf_completion_retirement_point_absent(void) {
}
#else
extern void WF_COMPLETION_RETIREMENT_POINT(void);
#endif

static pthread_mutex_t wf_retirement_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf_retirement_signal = PTHREAD_COND_INITIALIZER;
/* Descriptors this runtime has put back in the host's table.  Separate from
 * the in-flight count below because the two answer different questions: this
 * one is what a re-attempt may be spent on, that one is what waiting is for. */
static _Atomic uint64_t wf_retirement_returns;
static _Atomic size_t wf_retirement_in_flight;
static _Atomic size_t wf_retirement_waiter_count;
static _Atomic uint64_t wf_retirement_wait_starts;
/* The waiter order.  Mutated and read under the lock, which is also where a
 * give-up decision is made: the decision asks both whether this waiter is the
 * earliest one and whether an earlier one is owed the descriptor that has
 * come back, and the second question reads the order rather than its head. */
static _Atomic(wf_retirement_waiter *) wf_retirement_first;
static wf_retirement_waiter *wf_retirement_last;
/* How far into the return count this ledger has promised descriptors away.
 * Returns past it are unspent, and both things that can consume one move it by
 * one: an award to a waiter, and an open the host satisfied taking a
 * descriptor.  So a returned descriptor is promised to exactly one consumer,
 * and two refused opens never spend their one re-attempt each on the same one.
 *
 * It never moves backwards and never passes the return count, which is what
 * makes "unspent" mean the same thing to every waiter however they arrive and
 * leave: a waiter that opens the awarding raises it to its own `seen` and no
 * lower, and a charge is dropped rather than pushing it past what has actually
 * come back.  Under the lock. */
static uint64_t wf_retirement_awarded;
/* The runtime endpoint a waiter inside a blocking direct call parks on, told
 * to the ledger by the unit that owns it (contract.h). */
static _Atomic(wf_completion_runtime *) wf_retirement_endpoint;

uint64_t wf_completion_descriptor_returns(void) {
    return atomic_load_explicit(
        &wf_retirement_returns,
        memory_order_seq_cst
    );
}

void wf_completion_retirement_announces_on(wf_completion_runtime *runtime) {
    atomic_store_explicit(
        &wf_retirement_endpoint,
        runtime,
        memory_order_seq_cst
    );
}

/* Tells every place a waiter can sleep that this ledger has changed.
 *
 * The condition variable is where a waiter on the bounded adapter or the ring
 * sleeps; the endpoint above is where a waiter inside a blocking direct call
 * parks.  Both, because a transition that changes one waiter's answer changes
 * another's, and the two sleep in different places.
 *
 * Both, and exactly where the rule in contract.h asks for it: a transition
 * announces where it can leave *another* waiter asleep on an answer that is no
 * longer the best one available to it.  Which transitions those are is derived
 * there from the inputs the answer is made of; every caller of this and of
 * `wf_retirement_announce_on_the_endpoint` below is one of them.
 *
 * A thread never needs to wake itself, and this is the call that would do it —
 * announcing between a thread's own read of the wake epoch and its park on that
 * epoch cancels the park, which turns a wait into a spin.  That is a property
 * of where a route announces rather than of this call, and the direct route in
 * bridge.c is written to keep everything it says behind its own epoch read.
 *
 * The endpoint is notified after the lock is released, which is also the order
 * the park protocol needs: the ledger change is complete before the epoch that
 * a parking thread compares against moves, so a waiter that reads the epoch
 * and then asks the ledger either sees this change or is woken from the park
 * it takes. */
static void wf_retirement_announce(void) {
    wf_completion_runtime *endpoint;
    (void)pthread_mutex_lock(&wf_retirement_lock);
    (void)pthread_cond_broadcast(&wf_retirement_signal);
    (void)pthread_mutex_unlock(&wf_retirement_lock);
    endpoint = atomic_load_explicit(
        &wf_retirement_endpoint,
        memory_order_seq_cst
    );
    if (endpoint != NULL) {
        wf_completion_notify_capacity(endpoint);
    }
}

/* The endpoint half of the announcement, for the transitions that make their
 * change and broadcast under the lock they already hold.  Each of them owes the
 * announcement the rule above asks for: a waiter entering or leaving the order
 * changes what every other waiter is waiting for, and a waiter standing aside
 * hands the earliest deciding place to the next one — and any of those others
 * may be parked on the endpoint rather than on the condition variable. */
static void wf_retirement_announce_on_the_endpoint(void) {
    wf_completion_runtime *endpoint = atomic_load_explicit(
        &wf_retirement_endpoint,
        memory_order_seq_cst
    );
    if (endpoint != NULL) {
        wf_completion_notify_capacity(endpoint);
    }
}

void wf_completion_operation_accepted(void) {
    atomic_fetch_add_explicit(
        &wf_retirement_in_flight,
        1,
        memory_order_seq_cst
    );
    /* A waiter has to be told, and this is the only place that can tell it: a
     * request queued behind a refused open is one that engine must run before
     * anything can retire, and a waiter asleep on the ledger is not running
     * it.  Where no open is refused — which is every ordinary submission —
     * this is one relaxed-cost load and nothing else. */
    if (atomic_load_explicit(
            &wf_retirement_waiter_count,
            memory_order_seq_cst
        ) == 0) {
        return;
    }
    wf_retirement_announce();
}

void wf_completion_operation_retired(int returned_a_descriptor) {
    /* The return count moves first: a waiter reads it before the in-flight
     * count, so this order is what lets the re-read below see a retirement
     * whose two halves straddle the reads.
     *
     * An operation that returned nothing moves only the in-flight count.  It
     * still ends, so a waiter for which it was the last operation in flight
     * anywhere else stops waiting and answers, which is the rule's fourth step.
     * What it must not do is answer the second step: a re-attempt spent here
     * is spent on nothing this runtime gave back, and a refused open has one. */
    if (returned_a_descriptor != 0) {
        atomic_fetch_add_explicit(
            &wf_retirement_returns,
            1,
            memory_order_seq_cst
        );
    }
    atomic_fetch_sub_explicit(
        &wf_retirement_in_flight,
        1,
        memory_order_seq_cst
    );
    /* The announcement below and this load are sequentially consistent in
     * both directions, which is what makes the pair race-free: in the single
     * total order either the waiter's announcement precedes this load — so
     * the wake is delivered — or both updates above precede the waiter's own
     * reads, so the waiter sees this retirement and never sleeps.  The wake is
     * owed whether or not a descriptor came back, because an operation merely
     * ending can be what leaves nothing in flight anywhere else. */
    if (atomic_load_explicit(
            &wf_retirement_waiter_count,
            memory_order_seq_cst
        ) == 0) {
        return;
    }
    wf_retirement_announce();
}

size_t wf_completion_retirement_waiters(void) {
    return atomic_load_explicit(
        &wf_retirement_waiter_count,
        memory_order_acquire
    );
}

uint64_t wf_completion_retirement_waits(void) {
    return atomic_load_explicit(
        &wf_retirement_wait_starts,
        memory_order_relaxed
    );
}

void wf_completion_retirement_wait_begin(
    wf_retirement_waiter *waiter,
    uint64_t seen,
    size_t (*owed)(void *context),
    void *owed_context,
    int runs_owed
) {
    if (waiter == NULL) {
        return;
    }
    waiter->seen = seen;
    waiter->owed = owed;
    waiter->owed_context = owed_context;
    waiter->runs_owed = runs_owed;
    waiter->awarded = 0;
    waiter->aside = 0;
    waiter->next = NULL;
    (void)pthread_mutex_lock(&wf_retirement_lock);
    if (wf_retirement_last == NULL) {
        /* The order was empty, so this waiter opens the awarding: descriptors
         * that came back before its own host attempt are owed to nobody, and
         * every one that has come back since is unspent.  The mark is what
         * rations them, never what grants one — a waiter still has to have
         * seen the count move since its own attempt.
         *
         * Forward only.  A waiter's `seen` is read before its own host
         * attempt, and on the ring that is its submission, so a waiter can
         * arrive with a `seen` older than a descriptor this ledger has already
         * promised to somebody — and moving the mark back to it would promise
         * that descriptor a second time.  Three refused ring opens registering
         * one after another, each finding the order empty because the last one
         * had left to spend its award, were awarded one single returned
         * descriptor three times over; two of them re-attempted against a
         * descriptor that was gone and published the refusal.  The mark
         * therefore only ever rises: what it has spent stays spent, and what
         * this waiter opens the awarding on is whatever is unspent now. */
        if (seen > wf_retirement_awarded) {
            wf_retirement_awarded = seen;
        }
        atomic_store_explicit(
            &wf_retirement_first,
            waiter,
            memory_order_seq_cst
        );
    } else {
        wf_retirement_last->next = waiter;
    }
    wf_retirement_last = waiter;
    atomic_fetch_add_explicit(
        &wf_retirement_waiter_count,
        1,
        memory_order_seq_cst
    );
    atomic_fetch_add_explicit(
        &wf_retirement_wait_starts,
        1,
        memory_order_relaxed
    );
    /* One more waiter is one fewer operation that can retire, so an earlier
     * waiter may have just become the one that must publish — and an earlier
     * waiter parked on the runtime endpoint hears nothing said here, which is
     * why the announcement below is owed to it too. */
    (void)pthread_cond_broadcast(&wf_retirement_signal);
    (void)pthread_mutex_unlock(&wf_retirement_lock);
    wf_retirement_announce_on_the_endpoint();
}

void wf_completion_retirement_wait_end(wf_retirement_waiter *waiter) {
    wf_retirement_waiter *scan;
    wf_retirement_waiter *previous = NULL;
    if (waiter == NULL) {
        return;
    }
    (void)pthread_mutex_lock(&wf_retirement_lock);
    scan = atomic_load_explicit(&wf_retirement_first, memory_order_relaxed);
    while (scan != NULL && scan != waiter) {
        previous = scan;
        scan = scan->next;
    }
    if (scan != NULL) {
        if (previous == NULL) {
            atomic_store_explicit(
                &wf_retirement_first,
                waiter->next,
                memory_order_seq_cst
            );
        } else {
            previous->next = waiter->next;
        }
        if (wf_retirement_last == waiter) {
            wf_retirement_last = previous;
        }
        waiter->next = NULL;
        atomic_fetch_sub_explicit(
            &wf_retirement_waiter_count,
            1,
            memory_order_seq_cst
        );
    }
    /* Leaving hands the earliest place, and any return this waiter did not
     * take, to somebody else, whose answer may change with it. */
    (void)pthread_cond_broadcast(&wf_retirement_signal);
    (void)pthread_mutex_unlock(&wf_retirement_lock);
    wf_retirement_announce_on_the_endpoint();
}

/* The work this waiter's own thread is answerable for, read here rather than
 * taken from the caller so that every decision — including the one made inside
 * the sleep below, under the lock every wake takes — is made on the queue as
 * it is at that instant. */
static size_t wf_retirement_owed(const wf_retirement_waiter *waiter) {
    if (waiter == NULL || waiter->owed == NULL) {
        return 0;
    }
    return waiter->owed(waiter->owed_context);
}

/* True when this waiter's answer is work rather than waiting: its own thread
 * is the engine for a queue that is not empty. */
static int wf_retirement_must_run_its_own_work(
    const wf_retirement_waiter *waiter
) {
    return waiter != NULL && waiter->runs_owed != 0
        && wf_retirement_owed(waiter) != 0;
}

/* The earliest waiter that is deciding: the first in the order that is not
 * standing aside inside another operation.
 *
 * Publication is the one duty a waiter standing aside gives up, and it has to
 * be, because the operation it is running can be an open that is refused and
 * registers behind it.  That one has to be able to answer; a waiter that could
 * not publish while an earlier one stood aside would be waiting for the thread
 * that is waiting for it. */
static wf_retirement_waiter *wf_retirement_earliest_deciding(void) {
    wf_retirement_waiter *scan = atomic_load_explicit(
        &wf_retirement_first,
        memory_order_seq_cst
    );
    while (scan != NULL && scan->aside != 0) {
        scan = scan->next;
    }
    return scan;
}

/* True when a waiter registered before this one is owed a re-attempt: the
 * return count has moved since that waiter's own host attempt.  Read under the
 * lock, which is what makes walking the order safe — a waiter's record lives on
 * the stack of the thread it belongs to and leaves the order when that thread
 * answers.
 *
 * A waiter standing aside is walked like any other.  Its claim on a return is
 * registration order, and standing aside is not leaving: the open it is running
 * takes the refusal that sequential execution gives it, and this one takes the
 * descriptor when it comes back to ask.  Passing over it here would hand the
 * program's one `Ok` to the later open, which is the whole defect this order
 * exists to prevent. */
static int wf_retirement_an_earlier_waiter_is_owed(
    const wf_retirement_waiter *waiter,
    uint64_t returns
) {
    const wf_retirement_waiter *scan = atomic_load_explicit(
        &wf_retirement_first,
        memory_order_seq_cst
    );
    while (scan != NULL && scan != waiter) {
        if (scan->seen != returns) {
            return 1;
        }
        scan = scan->next;
    }
    return 0;
}

/* Whether this waiter is the one a returned descriptor is awarded to, and,
 * where `award` is set, awarding it.
 *
 * Three things have to hold: the count has moved since this waiter's own host
 * attempt, so a second attempt could answer differently; a return is unspent,
 * so this re-attempt is not the second one aimed at the same descriptor; and
 * no waiter registered earlier is owed one, because the ledger hands a return
 * to the earliest waiter exactly as it hands publication to the earliest.  An
 * award already made to this waiter stands, however many times it asks. */
static int wf_retirement_award_locked(
    wf_retirement_waiter *waiter,
    uint64_t returns,
    int award
) {
    if (waiter->awarded != 0) {
        return 1;
    }
    if (returns == waiter->seen || returns <= wf_retirement_awarded
        || wf_retirement_an_earlier_waiter_is_owed(waiter, returns) != 0) {
        return 0;
    }
    if (award != 0) {
        wf_retirement_awarded += 1;
        waiter->awarded = 1;
    }
    return 1;
}

/* One open succeeded on a host attempt, and the descriptor it holds now came
 * out of the host's table.
 *
 * The other half of the award rule, and the half that makes it a promise: a
 * descriptor this runtime returned is offered to at most one open, and an open
 * that has taken one is charged for it whether or not it ever waited.  An open
 * whose *first* attempt succeeds never registers as a waiter and never spends
 * an award, so without this the mark still offers the return that open
 * consumed to a refused one, which spends its single re-attempt on a
 * descriptor that is already gone and publishes a refusal the next close makes
 * untrue.  Measured before this: 25 lost `Ok`s in 280,000 repetitions of the
 * interleave shape, every one of them with a descriptor left over at the end.
 *
 * `on_an_award` is the attempt a waiter makes with an award already granted to
 * it.  The mark moved when the award was granted, and moving it again here
 * would charge one descriptor twice.
 *
 * Charged whenever a return is unspent, and whether or not a waiter is
 * registered at the time.  The order being empty is no reason to skip it: the
 * refused open that registers next may have made its own attempt long before
 * this take — on the ring, before it was even submitted — and the mark is what
 * carries the take across to it.  What the charge is bounded by is the unspent
 * returns themselves: with every return already spent, the descriptor this
 * open took was not one of this runtime's returns to give — the host had it
 * for a reason no ledger can see — and charging it would take from a waiter
 * that is owed one.  So the mark never passes the return count, and every
 * descriptor this runtime put back is promised exactly once.
 *
 * A waiter deprived here by a free that came from outside after all, which no
 * ledger can tell from its own, still cannot lose an `Ok` it is owed: this
 * refuses it nothing.  It keeps waiting while anything is in flight, and makes
 * its one attempt at the moment nothing is left that could bring a descriptor
 * back — which is the moment source order makes it, with every close the
 * program wrote before it already run.
 *
 * The mark moving can only make another waiter less awardable, so, exactly as
 * for the award that moves it inside a decision, it can turn no waiter's
 * `AWAITED` into a terminal answer and owes no announcement (contract.h). */
void wf_completion_retirement_open_took_a_descriptor(int on_an_award) {
    uint64_t returns;
    if (on_an_award != 0) {
        return;
    }
    (void)pthread_mutex_lock(&wf_retirement_lock);
    returns = atomic_load_explicit(
        &wf_retirement_returns,
        memory_order_seq_cst
    );
    if (returns > wf_retirement_awarded) {
        wf_retirement_awarded += 1;
    }
    (void)pthread_mutex_unlock(&wf_retirement_lock);
}

/* Where this waiter stands, decided under the retirement lock.
 *
 * `returns_before` is the decision's first read of the return count.  The
 * caller takes it before the schedule point and hands it in, so a retirement
 * whose two halves straddle the point is still one this decision can miss with
 * its first read and catch with its second — which is the interleaving the
 * second read below exists for. */
static enum wf_retirement_state wf_retirement_state_locked(
    wf_retirement_waiter *waiter,
    uint64_t returns_before,
    int award
) {
    size_t in_flight;
    size_t idle;
    size_t owed;
    if (waiter == NULL) {
        return WF_RETIREMENT_UNREACHABLE;
    }
    if (wf_retirement_award_locked(waiter, returns_before, award) != 0) {
        return WF_RETIREMENT_HAPPENED;
    }
    owed = wf_retirement_owed(waiter);
    if (waiter->runs_owed != 0 && owed != 0) {
        /* This thread is that queue's engine, and running it is work source
         * order performs anyway — one item of it may be the close that ends
         * this exhaustion.  So the answer is neither "publish" nor "sleep":
         * go and run it, then ask again. */
        return WF_RETIREMENT_AWAITED;
    }
    in_flight = atomic_load_explicit(
        &wf_retirement_in_flight,
        memory_order_seq_cst
    );
    /* Operations that cannot retire on their own: the waiters — one standing
     * aside inside another operation included, counted here once and only here
     * — and the ones this waiter's own thread would have to run.  Anything left
     * over is an operation running somewhere that can still give a descriptor
     * back, which is what makes waiting for it source order rather than a
     * guess. */
    idle = atomic_load_explicit(
        &wf_retirement_waiter_count,
        memory_order_seq_cst
    );
    if (waiter->runs_owed == 0) {
        idle += owed;
    }
    if (in_flight > idle) {
        return WF_RETIREMENT_AWAITED;
    }
    if (wf_retirement_earliest_deciding() != waiter) {
        /* Every operation left is a waiter, and an earlier one that is asking
         * answers first.  Leaving the waiter order is what hands this one the
         * same answer; its publication returns no descriptor and grants
         * nothing. */
        return WF_RETIREMENT_AWAITED;
    }
    /* Read the return count once more.  A returning retirement whose increment
     * landed after the first read and whose in-flight decrement landed before
     * the read above is invisible to both of them, and publishing a refusal it
     * had already answered would lose the program's `Ok`.  This waiter is the
     * earliest one that is asking, and the award below asks again whether an
     * earlier one standing aside is owed what this read finds — if one is, the
     * refusal published here is the one sequential execution gives this later
     * open, and the descriptor is still there for the earlier one; a return
     * already awarded to a waiter that has left the order to spend it is one
     * this waiter may not take, and the operation that waiter is spending it on
     * is in flight, so the in-flight test above is what keeps this one here. */
    if (wf_retirement_award_locked(
            waiter,
            atomic_load_explicit(
                &wf_retirement_returns,
                memory_order_seq_cst
            ),
            award
        ) != 0) {
        return WF_RETIREMENT_HAPPENED;
    }
    return WF_RETIREMENT_UNREACHABLE;
}

/* This waiter's thread has gone into another operation, and is not asking until
 * it comes back (contract.h).  Under the lock, because the decision that reads
 * the flag is made under it.
 *
 * The flag is read in exactly one place — which waiter is the earliest
 * *deciding* one — so all standing aside can move is that, and only for the one
 * waiter it promotes.  That waiter may have registered long before this aside
 * began and be asleep already, so this is a transition another waiter can sleep
 * through, and it owes an announcement wherever the promotion turns that
 * waiter's "keep waiting" into "publish".
 *
 * Announced there and nowhere else: this waiter held the earliest deciding
 * place, and the waiter it hands that place to now answers `UNREACHABLE`.  A
 * wake made anywhere else would be a wake for its own sake — two threads
 * standing aside in turn would trade them instead of sleeping — while a wake
 * made here is owed to a waiter that answers `UNREACHABLE` at the moment it is
 * made.  It need not still answer that when the wake arrives: coming back
 * demotes it again, and an aside that reverses quickly wakes a waiter that
 * finds nothing to do.  A thousand cycles here announce a thousand times and
 * hand out no answer, so the bound is not one wake per answer; it is that such
 * a wake costs one re-evaluation and one park, because every route reads the
 * epoch it parks on after its own announcements (contract.h). */
void wf_completion_retirement_defer_begin(wf_retirement_waiter *waiter) {
    wf_retirement_waiter *promoted;
    int announce = 0;
    if (waiter == NULL) {
        return;
    }
    (void)pthread_mutex_lock(&wf_retirement_lock);
    promoted = (wf_retirement_earliest_deciding() == waiter) ? waiter : NULL;
    waiter->aside = 1;
    if (promoted != NULL) {
        promoted = wf_retirement_earliest_deciding();
    }
    if (promoted != NULL
        && wf_retirement_state_locked(
               promoted,
               atomic_load_explicit(
                   &wf_retirement_returns,
                   memory_order_seq_cst
               ),
               0
           ) == WF_RETIREMENT_UNREACHABLE) {
        announce = 1;
        (void)pthread_cond_broadcast(&wf_retirement_signal);
    }
    (void)pthread_mutex_unlock(&wf_retirement_lock);
    if (announce != 0) {
        wf_retirement_announce_on_the_endpoint();
    }
}

/* And back.  All this can do to the waiter the aside promoted is demote it
 * again, from "publish" to "keep waiting" — which is what a waiter asleep on
 * that answer is already doing.  No other waiter's answer becomes terminal
 * here, so nothing is owed and nothing is said. */
void wf_completion_retirement_defer_end(wf_retirement_waiter *waiter) {
    if (waiter == NULL) {
        return;
    }
    (void)pthread_mutex_lock(&wf_retirement_lock);
    waiter->aside = 0;
    (void)pthread_mutex_unlock(&wf_retirement_lock);
}

enum wf_retirement_state wf_completion_retirement_state(
    wf_retirement_waiter *waiter
) {
    enum wf_retirement_state state;
    uint64_t returns_before;
    if (waiter == NULL) {
        return WF_RETIREMENT_UNREACHABLE;
    }
    /* The first of the decision's two reads of the return count, and the named
     * point between them, are taken here rather than inside: a test standing
     * at the point retires an operation from there, and retiring one takes the
     * lock the rest of this decision is made under. */
    returns_before = atomic_load_explicit(
        &wf_retirement_returns,
        memory_order_seq_cst
    );
    WF_COMPLETION_RETIREMENT_POINT();
    (void)pthread_mutex_lock(&wf_retirement_lock);
    state = wf_retirement_state_locked(waiter, returns_before, 1);
    (void)pthread_mutex_unlock(&wf_retirement_lock);
    return state;
}

void wf_completion_retirement_sleep(wf_retirement_waiter *waiter) {
    (void)pthread_mutex_lock(&wf_retirement_lock);
    /* The same decision the caller's loop makes, made again under the lock
     * every wake takes, and awarding nothing: this asks only whether to sleep,
     * and the caller's next question is the one that spends anything.  A waiter
     * that owes work it can run itself never sleeps — the work is the answer,
     * and the queue is read here, so an item that arrives while this thread is
     * deciding either is seen or arrives with a wake this thread is already
     * positioned to receive. */
    if (wf_retirement_state_locked(
            waiter,
            atomic_load_explicit(
                &wf_retirement_returns,
                memory_order_seq_cst
            ),
            0
        ) == WF_RETIREMENT_AWAITED
        && !wf_retirement_must_run_its_own_work(waiter)) {
        (void)pthread_cond_wait(&wf_retirement_signal, &wf_retirement_lock);
    }
    (void)pthread_mutex_unlock(&wf_retirement_lock);
}
