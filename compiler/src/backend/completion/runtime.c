#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include "contract.h"

#include <errno.h>
#include <limits.h>
#include <string.h>
#include <time.h>

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
    }
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
    for (offset = 0; offset < runtime->slot_count; ++offset) {
        size_t index = (start + offset) % runtime->slot_count;
        wf_completion_slot *slot = &runtime->slots[index];
        uint64_t generation;
        unsigned phase;

        (void)pthread_mutex_lock(&slot->publication_lock);
        phase = atomic_load_explicit(&slot->phase, memory_order_relaxed);
        if (phase != WF_COMPLETION_FREE) {
            (void)pthread_mutex_unlock(&slot->publication_lock);
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
            continue;
        }
        generation += 1;
        slot->terminal_kind = 0;
        slot->result_size = 0;
        atomic_store_explicit(&slot->milestones, 0, memory_order_relaxed);
        atomic_store_explicit(&slot->event_pending, 0, memory_order_relaxed);
        atomic_store_explicit(&slot->event_drained, 0, memory_order_relaxed);
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

static void wf_completion_notify_scheduler(wf_completion_runtime *runtime) {
    atomic_fetch_add_explicit(&runtime->wake_epoch, 1, memory_order_release);
    (void)pthread_mutex_lock(&runtime->wake_lock);
    if (atomic_load_explicit(
            &runtime->parked_schedulers,
            memory_order_relaxed
        ) != 0) {
        (void)pthread_cond_signal(&runtime->wake_condition);
        atomic_fetch_add_explicit(
            &runtime->stat_wake_signals,
            1,
            memory_order_relaxed
        );
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

    /* Result bytes precede the release-published milestone product. */
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
    size_t cursor;

    if (runtime == NULL || runtime->slots == NULL || events == NULL
        || event_capacity == 0 || scan_budget == 0) {
        return 0;
    }
    cursor = atomic_fetch_add_explicit(
        &runtime->drain_cursor,
        scan_budget,
        memory_order_relaxed
    );
    while (scanned < scan_budget && produced < event_capacity) {
        size_t index = (cursor + scanned) % runtime->slot_count;
        wf_completion_slot *slot = &runtime->slots[index];
        unsigned pending = 1;
        scanned += 1;
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
        events[produced].token.slot = (uint32_t)index;
        events[produced].token.generation = atomic_load_explicit(
            &slot->generation,
            memory_order_relaxed
        );
        events[produced].milestones = atomic_load_explicit(
            &slot->milestones,
            memory_order_acquire
        );
        events[produced].terminal_kind = slot->terminal_kind;
        atomic_store_explicit(&slot->event_drained, 1, memory_order_release);
        (void)pthread_mutex_unlock(&slot->publication_lock);

        atomic_fetch_sub_explicit(&runtime->ready_events, 1, memory_order_acq_rel);
        atomic_fetch_add_explicit(
            &runtime->stat_drained_events,
            1,
            memory_order_relaxed
        );
        produced += 1;
    }
    return produced;
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
    outcome->result_size = slot->result_size;

    slot->terminal_kind = 0;
    slot->result_size = 0;
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
        memory_order_relaxed
    );
    atomic_fetch_add_explicit(&runtime->stat_parks, 1, memory_order_relaxed);

    /* This second observation closes empty-check -> sleep against a publisher
     * which incremented the epoch just before taking wake_lock. */
    if (atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire)
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
    statistics.capacity_notifications = atomic_load_explicit(
        &runtime->stat_capacity_notifications,
        memory_order_relaxed
    );
    return statistics;
}
