#include "windows_completion.h"

#if defined(_WIN32)

#include <errno.h>
#include <limits.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static LONG64 wf_windows_load64(const volatile LONG64 *value) {
    return InterlockedCompareExchange64((volatile LONG64 *)value, 0, 0);
}

static LONG wf_windows_load32(const volatile LONG *value) {
    return InterlockedCompareExchange((volatile LONG *)value, 0, 0);
}

/* Keep cursors in [0, modulo) instead of relying on signed LONG64 overflow.
 * The returned value is the caller's scan start and every update is bounded
 * because runtime slot counts are at most UINT32_MAX. */
static size_t wf_windows_advance_cursor(
    volatile LONG64 *cursor,
    size_t modulo,
    size_t distance
) {
    LONG64 observed;
    LONG64 desired;
    size_t step = distance % modulo;
    do {
        observed = wf_windows_load64(cursor);
        desired = (LONG64)(((size_t)observed + step) % modulo);
    } while (InterlockedCompareExchange64(cursor, desired, observed)
             != observed);
    return (size_t)observed;
}

static int wf_windows_token_slot(
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

int wf_completion_runtime_init(
    wf_completion_runtime *runtime,
    wf_completion_slot *slots,
    size_t slot_count
) {
    size_t index;
    if (runtime == NULL || slots == NULL || slot_count == 0
        || slot_count > UINT32_MAX) {
        return EINVAL;
    }
    memset(runtime, 0, sizeof(*runtime));
    runtime->slots = slots;
    runtime->slot_count = slot_count;
    InitializeSRWLock(&runtime->claim_lock);
    InitializeSRWLock(&runtime->wake_lock);
    InitializeConditionVariable(&runtime->wake_condition);
    runtime->ready_frame = NULL;
    for (index = 0; index < slot_count; ++index) {
        memset(&slots[index], 0, sizeof(slots[index]));
        InitializeSRWLock(&slots[index].publication_lock);
        slots[index].phase = WF_COMPLETION_FREE;
    }
    return 0;
}

int wf_completion_set_ready_callback(
    wf_completion_runtime *runtime,
    wf_completion_ready_callback ready
) {
    if (runtime == NULL || runtime->slots == NULL || ready == NULL) {
        return EINVAL;
    }
    AcquireSRWLockExclusive(&runtime->claim_lock);
    if (runtime->ready_frame != NULL) {
        ReleaseSRWLockExclusive(&runtime->claim_lock);
        return EBUSY;
    }
    runtime->ready_frame = ready;
    ReleaseSRWLockExclusive(&runtime->claim_lock);
    return 0;
}

int wf_completion_runtime_destroy(wf_completion_runtime *runtime) {
    size_t index;
    if (runtime == NULL || runtime->slots == NULL || runtime->slot_count == 0) {
        return EINVAL;
    }
    if (wf_windows_load32(&runtime->parked_schedulers) != 0
        || wf_windows_load32(&runtime->iocp_waiters) != 0
        || wf_windows_load64(&runtime->ready_events) != 0) {
        return EBUSY;
    }
    for (index = 0; index < runtime->slot_count; ++index) {
        unsigned phase;
        AcquireSRWLockExclusive(&runtime->slots[index].publication_lock);
        phase = runtime->slots[index].phase;
        ReleaseSRWLockExclusive(&runtime->slots[index].publication_lock);
        if (phase != WF_COMPLETION_FREE && phase != WF_COMPLETION_RETIRED) {
            return EBUSY;
        }
    }
    runtime->slots = NULL;
    runtime->slot_count = 0;
    runtime->wake_port = NULL;
    return 0;
}

int wf_windows_completion_bind_iocp(
    wf_completion_runtime *runtime,
    HANDLE port,
    ULONG_PTR wake_key
) {
    if (runtime == NULL || runtime->slots == NULL || port == NULL
        || port == INVALID_HANDLE_VALUE || wake_key == 0) {
        return EINVAL;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (runtime->wake_port != NULL
        && (runtime->wake_port != port || runtime->wake_key != wake_key)) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EBUSY;
    }
    runtime->wake_port = port;
    runtime->wake_key = wake_key;
    ReleaseSRWLockExclusive(&runtime->wake_lock);
    return 0;
}

static enum wf_completion_claim_result wf_windows_claim_one_locked(
    wf_completion_runtime *runtime,
    wf_completion_token *token
) {
    size_t start;
    size_t offset;
    if (runtime == NULL || token == NULL || runtime->slots == NULL
        || runtime->slot_count == 0) {
        return WF_COMPLETION_CLAIM_INVALID;
    }
    start = wf_windows_advance_cursor(
        &runtime->claim_cursor,
        runtime->slot_count,
        1
    );
    for (offset = 0; offset < runtime->slot_count; ++offset) {
        size_t index = start;
        wf_completion_slot *slot = &runtime->slots[index];
        AcquireSRWLockExclusive(&slot->publication_lock);
        if (slot->phase != WF_COMPLETION_FREE) {
            ReleaseSRWLockExclusive(&slot->publication_lock);
            start = start + 1u == runtime->slot_count ? 0u : start + 1u;
            continue;
        }
        if (slot->generation == UINT64_MAX) {
            slot->phase = WF_COMPLETION_RETIRED;
            ReleaseSRWLockExclusive(&slot->publication_lock);
            start = start + 1u == runtime->slot_count ? 0u : start + 1u;
            continue;
        }
        slot->generation += 1;
        slot->phase = WF_COMPLETION_READY;
        slot->milestones = 0;
        slot->event_pending = 0;
        slot->event_drained = 0;
        slot->consume_waiting = 0;
        slot->terminal_kind = 0;
        slot->adapter_tag = 0;
        slot->result_size = 0;
        slot->dependent_frame = NULL;
        slot->dependent_requirement = 0;
        token->slot = (uint32_t)index;
        token->generation = slot->generation;
        ReleaseSRWLockExclusive(&slot->publication_lock);
        InterlockedIncrement64(&runtime->stat_claims);
        return WF_COMPLETION_CLAIMED;
    }
    InterlockedIncrement64(&runtime->stat_claim_capacity_waits);
    return WF_COMPLETION_CLAIM_WAIT_CAPACITY;
}

enum wf_completion_claim_result wf_completion_claim(
    wf_completion_runtime *runtime,
    wf_completion_token *token
) {
    enum wf_completion_claim_result result;
    if (runtime == NULL || token == NULL || runtime->slots == NULL) {
        return WF_COMPLETION_CLAIM_INVALID;
    }
    AcquireSRWLockExclusive(&runtime->claim_lock);
    result = wf_windows_claim_one_locked(runtime, token);
    ReleaseSRWLockExclusive(&runtime->claim_lock);
    return result;
}

static enum wf_completion_transition_result wf_windows_transition(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    unsigned first_expected,
    unsigned second_expected,
    unsigned desired
) {
    wf_completion_slot *slot;
    if (!wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_TRANSITION_STALE;
    }
    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_STALE;
    }
    if (slot->phase != first_expected && slot->phase != second_expected) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_INVALID_STATE;
    }
    slot->phase = desired;
    ReleaseSRWLockExclusive(&slot->publication_lock);
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_transition_result wf_completion_begin_submit(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    return wf_windows_transition(
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
    enum wf_completion_transition_result result = wf_windows_transition(
        runtime,
        token,
        WF_COMPLETION_SUBMITTING,
        WF_COMPLETION_SUBMITTING,
        WF_COMPLETION_WAIT_CAPACITY
    );
    if (result == WF_COMPLETION_TRANSITIONED) {
        InterlockedIncrement64(&runtime->stat_target_capacity_waits);
    }
    return result;
}

enum wf_completion_transition_result wf_completion_target_accepted(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    return wf_windows_transition(
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
    if (adapter_tag == 0 || !wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_TRANSITION_STALE;
    }
    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_STALE;
    }
    if (slot->phase != WF_COMPLETION_READY
        && slot->phase != WF_COMPLETION_WAIT_CAPACITY
        && slot->phase != WF_COMPLETION_SUBMITTING) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_INVALID_STATE;
    }
    slot->adapter_tag = adapter_tag;
    ReleaseSRWLockExclusive(&slot->publication_lock);
    return WF_COMPLETION_TRANSITIONED;
}

static void wf_windows_notify_scheduler(wf_completion_runtime *runtime) {
    HANDLE port;
    ULONG_PTR key;
    InterlockedIncrement64(&runtime->wake_epoch);
    AcquireSRWLockExclusive(&runtime->wake_lock);
    port = runtime->wake_port;
    key = runtime->wake_key;
    if (wf_windows_load32(&runtime->parked_schedulers) != 0) {
        WakeConditionVariable(&runtime->wake_condition);
        InterlockedIncrement64(&runtime->stat_wake_signals);
    }
    if (port != NULL && wf_windows_load32(&runtime->iocp_waiters) != 0) {
        if (PostQueuedCompletionStatus(port, 0, key, NULL) != FALSE) {
            InterlockedIncrement64(&runtime->stat_wake_signals);
        }
    }
    ReleaseSRWLockExclusive(&runtime->wake_lock);
}

enum wf_completion_park_result wf_windows_completion_iocp_wait_begin(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch
) {
    if (runtime == NULL || runtime->wake_port == NULL) {
        return WF_COMPLETION_PARK_FAILED;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (wf_completion_wake_epoch(runtime) != observed_epoch) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }
    InterlockedIncrement(&runtime->iocp_waiters);
    if (wf_completion_wake_epoch(runtime) != observed_epoch) {
        InterlockedDecrement(&runtime->iocp_waiters);
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }
    ReleaseSRWLockExclusive(&runtime->wake_lock);
    return WF_COMPLETION_PARK_WOKEN;
}

void wf_windows_completion_iocp_wait_end(wf_completion_runtime *runtime) {
    if (runtime != NULL) {
        (void)InterlockedDecrement(&runtime->iocp_waiters);
    }
}

static enum wf_completion_publish_result wf_windows_publish(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication,
    unsigned expected_phase
) {
    wf_completion_slot *slot;
    if (publication == NULL || publication->terminal_kind == 0
        || publication->result_size > WF_COMPLETION_RESULT_CAPACITY
        || (publication->result_size != 0 && publication->result == NULL)) {
        return publication != NULL
                && publication->result_size > WF_COMPLETION_RESULT_CAPACITY
            ? WF_COMPLETION_PUBLISH_RESULT_TOO_LARGE
            : WF_COMPLETION_PUBLISH_INVALID_ARGUMENT;
    }
    if ((publication->milestones & WF_COMPLETION_OWNERSHIP_COMPLETE)
        != WF_COMPLETION_OWNERSHIP_COMPLETE) {
        return WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL;
    }
    if (!wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_PUBLISH_STALE;
    }
    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation) {
        InterlockedIncrement64(&runtime->stat_stale_publications);
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_STALE;
    }
    if (slot->phase == WF_COMPLETION_TERMINAL_PHASE) {
        InterlockedIncrement64(&runtime->stat_duplicate_terminals);
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL;
    }
    if (slot->phase != expected_phase) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_INVALID_STATE;
    }

    /* Captured generation and phase are proven under exclusive reuse lock
     * before the first result byte changes. */
    if (publication->result_size != 0) {
        memcpy(
            slot->result.bytes,
            publication->result,
            publication->result_size
        );
    }
    slot->result_size = publication->result_size;
    slot->terminal_kind = publication->terminal_kind;
    slot->milestones = publication->milestones;
    slot->phase = WF_COMPLETION_TERMINAL_PHASE;
    InterlockedExchange(&slot->event_pending, 1);
    InterlockedIncrement64(&runtime->ready_events);
    InterlockedIncrement64(&runtime->stat_publications);
    ReleaseSRWLockExclusive(&slot->publication_lock);
    wf_windows_notify_scheduler(runtime);
    return WF_COMPLETION_PUBLISHED;
}

/* The non-terminal milestone route, on the same terms as the POSIX core in
 * runtime.c: one fact of an operation the target is submitting or already
 * holds, accumulated into the slot's product under the same exclusive lock the
 * terminal routes take.  It writes no result byte, does not move the phase and
 * raises no completion event.
 *
 * This target has no adapter that calls it today — the Windows IOCP adapter
 * carries transfers only, and no open on this target is submitted through a
 * path-staging adapter — so it is the core contract being complete on every
 * target it is compiled for rather than a live route.  The contract in
 * contract.h declares it unconditionally, and a core that declared it and did
 * not define it would fail to link the first time an adapter here published a
 * milestone. */
enum wf_completion_publish_result wf_completion_publish_milestone(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t milestones
) {
    wf_completion_slot *slot;

    /* Only facts of this operation, and never the terminal one: a terminal
     * fact without the result bytes it stands for would let a consumer read a
     * slot that has nothing in it. */
    if (milestones == 0
        || (milestones & ~(uint32_t)WF_COMPLETION_OWNERSHIP_COMPLETE) != 0
        || (milestones & (uint32_t)WF_COMPLETION_TERMINAL) != 0) {
        return WF_COMPLETION_PUBLISH_INVALID_ARGUMENT;
    }
    if (!wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_PUBLISH_STALE;
    }

    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation) {
        InterlockedIncrement64(&runtime->stat_stale_publications);
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_STALE;
    }
    if (slot->phase == WF_COMPLETION_TERMINAL_PHASE) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL;
    }
    if (slot->phase != WF_COMPLETION_SUBMITTING
        && slot->phase != WF_COMPLETION_IN_FLIGHT) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_PUBLISH_INVALID_STATE;
    }
    /* Accumulated, not stored: this route publishes one fact of a product the
     * terminal route later completes, and the terminal product is a superset
     * of everything published here. */
    slot->milestones |= milestones;
    ReleaseSRWLockExclusive(&slot->publication_lock);
    return WF_COMPLETION_PUBLISHED;
}

enum wf_completion_publish_result wf_completion_publish_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
) {
    return wf_windows_publish(
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
    return wf_windows_publish(
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
    size_t start;
    size_t scanned = 0;
    size_t produced = 0;
    int wake_consumer = 0;
    if (runtime == NULL || runtime->slots == NULL || events == NULL
        || event_capacity == 0 || scan_budget == 0) {
        return 0;
    }
    start = wf_windows_advance_cursor(
        &runtime->drain_cursor,
        runtime->slot_count,
        scan_budget
    );
    while (scanned < scan_budget && produced < event_capacity) {
        size_t index = start;
        wf_completion_slot *slot = &runtime->slots[index];
        void *dependent_frame = NULL;
        scanned += 1;
        start = start + 1u == runtime->slot_count ? 0u : start + 1u;
        if (InterlockedCompareExchange(&slot->event_pending, 0, 1) != 1) {
            continue;
        }
        AcquireSRWLockExclusive(&slot->publication_lock);
        events[produced].token.slot = (uint32_t)index;
        events[produced].token.generation = slot->generation;
        events[produced].milestones = slot->milestones;
        events[produced].terminal_kind = slot->terminal_kind;
        if (slot->dependent_frame != NULL
            && (slot->milestones & slot->dependent_requirement)
                == slot->dependent_requirement) {
            dependent_frame = slot->dependent_frame;
            slot->dependent_frame = NULL;
            slot->dependent_requirement = 0;
        }
        slot->event_drained = 1;
        if (slot->consume_waiting != 0) {
            slot->consume_waiting = 0;
            wake_consumer = 1;
        }
        ReleaseSRWLockExclusive(&slot->publication_lock);
        InterlockedDecrement64(&runtime->ready_events);
        InterlockedIncrement64(&runtime->stat_drained_events);
        if (dependent_frame != NULL) {
            if (runtime->ready_frame == NULL) {
                abort();
            }
            runtime->ready_frame(dependent_frame);
        }
        produced += 1;
    }
    if (wake_consumer != 0) {
        wf_windows_notify_scheduler(runtime);
    }
    return produced;
}

size_t wf_completion_ready_event_count(const wf_completion_runtime *runtime) {
    return runtime == NULL
        ? 0
        : (size_t)wf_windows_load64(&runtime->ready_events);
}

enum wf_completion_transition_result wf_completion_observe(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t *milestones,
    unsigned *phase
) {
    wf_completion_slot *slot;
    if (milestones == NULL || phase == NULL
        || !wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_TRANSITION_STALE;
    }
    AcquireSRWLockShared(&slot->publication_lock);
    if (slot->generation != token.generation) {
        ReleaseSRWLockShared(&slot->publication_lock);
        return WF_COMPLETION_TRANSITION_STALE;
    }
    *milestones = slot->milestones;
    *phase = slot->phase;
    ReleaseSRWLockShared(&slot->publication_lock);
    return WF_COMPLETION_TRANSITIONED;
}

enum wf_completion_depend_result wf_completion_depend(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t requirement,
    void *frame
) {
    wf_completion_slot *slot;
    int already_ready = 0;
    if (requirement == 0 || frame == NULL || runtime == NULL
        || runtime->ready_frame == NULL
        || !wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_DEPEND_INVALID_ARGUMENT;
    }
    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_DEPEND_STALE;
    }
    if (slot->dependent_frame != NULL) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_DEPEND_DUPLICATE;
    }
    if (slot->phase == WF_COMPLETION_FREE
        || slot->phase == WF_COMPLETION_RETIRED) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_DEPEND_STALE;
    }
    if ((slot->milestones & requirement) == requirement
        && slot->event_drained != 0) {
        already_ready = 1;
    } else {
        slot->dependent_frame = frame;
        slot->dependent_requirement = requirement;
    }
    ReleaseSRWLockExclusive(&slot->publication_lock);
    if (already_ready) {
        runtime->ready_frame(frame);
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
    if (outcome == NULL || (result_capacity != 0 && result == NULL)
        || !wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_CONSUME_INVALID_ARGUMENT;
    }
    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_STALE;
    }
    if (slot->phase != WF_COMPLETION_TERMINAL_PHASE) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_NOT_TERMINAL;
    }
    if (slot->event_drained == 0) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_NOT_DRAINED;
    }
    if (result_capacity < slot->result_size) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_RESULT_TOO_LARGE;
    }
    if (slot->result_size != 0) {
        memcpy(result, slot->result.bytes, slot->result_size);
    }
    outcome->milestones = slot->milestones;
    outcome->terminal_kind = slot->terminal_kind;
    outcome->adapter_tag = slot->adapter_tag;
    outcome->result_size = slot->result_size;
    slot->milestones = 0;
    slot->terminal_kind = 0;
    slot->adapter_tag = 0;
    slot->result_size = 0;
    slot->dependent_frame = NULL;
    slot->dependent_requirement = 0;
    slot->consume_waiting = 0;
    slot->event_drained = 0;
    slot->phase = WF_COMPLETION_FREE;
    ReleaseSRWLockExclusive(&slot->publication_lock);
    InterlockedIncrement64(&runtime->stat_consumptions);
    wf_completion_notify_capacity(runtime);
    return WF_COMPLETION_CONSUMED;
}

enum wf_completion_consume_wait_result wf_completion_wait_to_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    wf_completion_slot *slot;
    if (!wf_windows_token_slot(runtime, token, &slot)) {
        return WF_COMPLETION_CONSUME_WAIT_STALE;
    }
    AcquireSRWLockExclusive(&slot->publication_lock);
    if (slot->generation != token.generation
        || slot->phase == WF_COMPLETION_FREE
        || slot->phase == WF_COMPLETION_RETIRED) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_WAIT_STALE;
    }
    if (slot->phase == WF_COMPLETION_TERMINAL_PHASE
        && slot->event_drained != 0) {
        ReleaseSRWLockExclusive(&slot->publication_lock);
        return WF_COMPLETION_CONSUME_READY;
    }
    slot->consume_waiting = 1;
    ReleaseSRWLockExclusive(&slot->publication_lock);
    return WF_COMPLETION_CONSUME_WAIT_REGISTERED;
}

uint64_t wf_completion_wake_epoch(const wf_completion_runtime *runtime) {
    return runtime == NULL
        ? 0
        : (uint64_t)wf_windows_load64(&runtime->wake_epoch);
}

void wf_completion_notify_compute(wf_completion_runtime *runtime) {
    if (runtime != NULL) {
        InterlockedIncrement64(&runtime->stat_compute_notifications);
        wf_windows_notify_scheduler(runtime);
    }
}

void wf_completion_notify_capacity(wf_completion_runtime *runtime) {
    if (runtime != NULL) {
        InterlockedIncrement64(&runtime->stat_capacity_notifications);
        wf_windows_notify_scheduler(runtime);
    }
}

enum wf_completion_park_result wf_completion_park_if_unchanged(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
) {
    BOOL slept;
    DWORD error;
    DWORD remaining = timeout_milliseconds;
    ULONGLONG started = 0;
    if (runtime == NULL) {
        return WF_COMPLETION_PARK_FAILED;
    }
    if (timeout_milliseconds != UINT32_MAX) {
        started = GetTickCount64();
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (wf_completion_wake_epoch(runtime) != observed_epoch) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }
    InterlockedIncrement(&runtime->parked_schedulers);
    InterlockedIncrement64(&runtime->stat_parks);
    if (wf_completion_wake_epoch(runtime) != observed_epoch) {
        InterlockedDecrement(&runtime->parked_schedulers);
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }
    do {
        slept = SleepConditionVariableSRW(
            &runtime->wake_condition,
            &runtime->wake_lock,
            remaining,
            0
        );
        error = slept != FALSE ? ERROR_SUCCESS : GetLastError();
        if (slept != FALSE && timeout_milliseconds != UINT32_MAX) {
            ULONGLONG elapsed = GetTickCount64() - started;
            if (elapsed >= timeout_milliseconds) {
                slept = FALSE;
                error = ERROR_TIMEOUT;
            } else {
                remaining = timeout_milliseconds - (DWORD)elapsed;
            }
        }
    } while (slept != FALSE
             && wf_completion_wake_epoch(runtime) == observed_epoch);
    InterlockedDecrement(&runtime->parked_schedulers);
    ReleaseSRWLockExclusive(&runtime->wake_lock);
    if (slept == FALSE && error == ERROR_TIMEOUT) {
        return WF_COMPLETION_PARK_TIMED_OUT;
    }
    if (slept == FALSE) {
        return WF_COMPLETION_PARK_FAILED;
    }
    return WF_COMPLETION_PARK_WOKEN;
}

unsigned wf_completion_parked_scheduler_count(
    const wf_completion_runtime *runtime
) {
    return runtime == NULL
        ? 0u
        : (unsigned)wf_windows_load32(&runtime->parked_schedulers);
}

wf_completion_statistics wf_completion_statistics_snapshot(
    const wf_completion_runtime *runtime
) {
    wf_completion_statistics statistics = {0};
    if (runtime == NULL) {
        return statistics;
    }
#define WF_WINDOWS_STAT(field) ((uint64_t)wf_windows_load64(&(field)))
    statistics.claims = WF_WINDOWS_STAT(runtime->stat_claims);
    statistics.claim_capacity_waits =
        WF_WINDOWS_STAT(runtime->stat_claim_capacity_waits);
    statistics.target_capacity_waits =
        WF_WINDOWS_STAT(runtime->stat_target_capacity_waits);
    statistics.publications = WF_WINDOWS_STAT(runtime->stat_publications);
    statistics.stale_publications =
        WF_WINDOWS_STAT(runtime->stat_stale_publications);
    statistics.duplicate_terminals =
        WF_WINDOWS_STAT(runtime->stat_duplicate_terminals);
    statistics.drained_events =
        WF_WINDOWS_STAT(runtime->stat_drained_events);
    statistics.consumptions = WF_WINDOWS_STAT(runtime->stat_consumptions);
    statistics.parks = WF_WINDOWS_STAT(runtime->stat_parks);
    statistics.wake_signals = WF_WINDOWS_STAT(runtime->stat_wake_signals);
    statistics.compute_notifications =
        WF_WINDOWS_STAT(runtime->stat_compute_notifications);
    statistics.capacity_notifications =
        WF_WINDOWS_STAT(runtime->stat_capacity_notifications);
#undef WF_WINDOWS_STAT
    return statistics;
}

#else

typedef int wf_windows_completion_translation_unit_is_target_guarded;

#endif
