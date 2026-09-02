#include "windows_completion.h"
#include "windows_iocp.h"

#if defined(_WIN32)

#include <errno.h>
#include <limits.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static LONG64 wf_windows_load64(const volatile LONG64 *value) {
    return InterlockedCompareExchange64((volatile LONG64 *)value, 0, 0);
}

static LONG wf_windows_load32(const volatile LONG *value) {
    return InterlockedCompareExchange((volatile LONG *)value, 0, 0);
}

static int wf_windows_completion_close_iocp(
    wf_completion_runtime *runtime,
    HANDLE port,
    ULONG_PTR wake_key
);
static int wf_windows_completion_iocp_wait_finished(
    wf_completion_runtime *runtime,
    HANDLE port,
    ULONG_PTR wake_key,
    unsigned consumed_wake
);

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
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (runtime->wake_port != NULL
        || wf_windows_load32(&runtime->parked_schedulers) != 0
        || wf_windows_load32(&runtime->iocp_waiters) != 0
        || wf_windows_load32(&runtime->iocp_wake_packets) != 0) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EBUSY;
    }
    ReleaseSRWLockExclusive(&runtime->wake_lock);
    if (wf_windows_load64(&runtime->ready_events) != 0) {
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
    return 0;
}

int wf_windows_completion_bind_iocp(
    wf_completion_runtime *runtime,
    struct wf_windows_iocp_adapter *adapter,
    HANDLE port,
    ULONG_PTR wake_key
) {
    int error;
    if (runtime == NULL || runtime->slots == NULL || adapter == NULL
        || port == NULL
        || port == INVALID_HANDLE_VALUE || wake_key == 0) {
        return EINVAL;
    }
    if (wf_windows_iocp_port(adapter) != port
        || wf_windows_iocp_wake_key(adapter) != wake_key) {
        return EINVAL;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (runtime->wake_port != NULL
        && (runtime->wake_port != port || runtime->wake_key != wake_key)) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EBUSY;
    }
    if (wf_windows_load32(&runtime->iocp_waiters) != 0
        || wf_windows_load32(&runtime->iocp_wake_packets) != 0) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EBUSY;
    }
    error = wf_windows_iocp_attach_completion(
        adapter,
        runtime,
        wf_windows_completion_iocp_wait_finished,
        wf_windows_completion_close_iocp
    );
    if (error != 0) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return error;
    }
    runtime->wake_port = port;
    runtime->wake_key = wake_key;
    ReleaseSRWLockExclusive(&runtime->wake_lock);
    return 0;
}

static int wf_windows_completion_close_iocp(
    wf_completion_runtime *runtime,
    HANDLE port,
    ULONG_PTR wake_key
) {
    DWORD error;
    if (runtime == NULL || port == NULL || port == INVALID_HANDLE_VALUE
        || wake_key == 0) {
        return EINVAL;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (runtime->wake_port == NULL) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EPROTO;
    }
    if (runtime->wake_port != port || runtime->wake_key != wake_key) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EBUSY;
    }
    if (wf_windows_load32(&runtime->iocp_waiters) != 0) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EBUSY;
    }
    if (CloseHandle(port) == FALSE) {
        error = GetLastError();
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return (int)error;
    }
    runtime->wake_port = NULL;
    runtime->wake_key = 0;
    InterlockedExchange(&runtime->iocp_wake_packets, 0);
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

int wf_completion_has_capacity(const wf_completion_runtime *runtime) {
    size_t index;
    if (runtime == NULL || runtime->slots == NULL
        || runtime->slot_count == 0) {
        return 0;
    }
    for (index = 0; index < runtime->slot_count; ++index) {
        wf_completion_slot *slot = &runtime->slots[index];
        int available;
        AcquireSRWLockShared(&slot->publication_lock);
        available = slot->phase == WF_COMPLETION_FREE
            && slot->generation != UINT64_MAX;
        ReleaseSRWLockShared(&slot->publication_lock);
        if (available) {
            return 1;
        }
    }
    return 0;
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

static void wf_windows_fill_iocp_wakes_locked(
    wf_completion_runtime *runtime
) {
    LONG waiters = wf_windows_load32(&runtime->iocp_waiters);
    LONG packets = wf_windows_load32(&runtime->iocp_wake_packets);
    while (runtime->wake_port != NULL && packets < waiters) {
        if (PostQueuedCompletionStatus(
                runtime->wake_port,
                0,
                runtime->wake_key,
                NULL
            ) == FALSE) {
            /* The caller has already advanced the durable wake epoch, but an
             * IOCP waiter cannot observe it until GQCS returns. There is no
             * recoverable state in which dropping this packet preserves the
             * missed-wake contract. */
            abort();
        }
        packets = InterlockedIncrement(&runtime->iocp_wake_packets);
        InterlockedIncrement64(&runtime->stat_wake_signals);
    }
}

static void wf_windows_notify_scheduler(wf_completion_runtime *runtime) {
    InterlockedIncrement64(&runtime->wake_epoch);
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (wf_windows_load32(&runtime->parked_schedulers) != 0) {
        WakeConditionVariable(&runtime->wake_condition);
        InterlockedIncrement64(&runtime->stat_wake_signals);
    }
    wf_windows_fill_iocp_wakes_locked(runtime);
    ReleaseSRWLockExclusive(&runtime->wake_lock);
}

enum wf_completion_park_result wf_windows_completion_iocp_wait_begin(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch
) {
    if (runtime == NULL) {
        return WF_COMPLETION_PARK_FAILED;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (runtime->wake_port == NULL) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_FAILED;
    }
    if (wf_completion_wake_epoch(runtime) != observed_epoch) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }
    if (wf_windows_load32(&runtime->iocp_waiters) == LONG_MAX) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return WF_COMPLETION_PARK_FAILED;
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
    if (runtime == NULL) {
        return;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (wf_windows_load32(&runtime->iocp_waiters) != 0) {
        (void)InterlockedDecrement(&runtime->iocp_waiters);
    }
    ReleaseSRWLockExclusive(&runtime->wake_lock);
}

/* Withdraw the returned waiter and its optional scheduler packet in one
 * critical section before the adapter can publish a real completion. */
static int wf_windows_completion_iocp_wait_finished(
    wf_completion_runtime *runtime,
    HANDLE port,
    ULONG_PTR wake_key,
    unsigned consumed_wake
) {
    int error = 0;
    if (runtime == NULL || port == NULL || port == INVALID_HANDLE_VALUE
        || wake_key == 0 || consumed_wake > 1u) {
        return EINVAL;
    }
    AcquireSRWLockExclusive(&runtime->wake_lock);
    if (runtime->wake_port != port || runtime->wake_key != wake_key) {
        ReleaseSRWLockExclusive(&runtime->wake_lock);
        return EPROTO;
    }
    if (wf_windows_load32(&runtime->iocp_waiters) == 0) {
        error = EPROTO;
    } else {
        (void)InterlockedDecrement(&runtime->iocp_waiters);
    }
    if (consumed_wake != 0u) {
        if (wf_windows_load32(&runtime->iocp_wake_packets) == 0) {
            error = EPROTO;
        } else {
            (void)InterlockedDecrement(&runtime->iocp_wake_packets);
        }
    }
    ReleaseSRWLockExclusive(&runtime->wake_lock);
    return error;
}

unsigned wf_windows_completion_iocp_waiter_count(
    const wf_completion_runtime *runtime
) {
    return runtime == NULL
        ? 0u
        : (unsigned)wf_windows_load32(&runtime->iocp_waiters);
}

unsigned wf_windows_completion_iocp_wake_packet_count(
    const wf_completion_runtime *runtime
) {
    return runtime == NULL
        ? 0u
        : (unsigned)wf_windows_load32(&runtime->iocp_wake_packets);
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

/* -------------------------------------------------------------------------
 * Process-wide descriptor-retirement ledger.
 *
 * This is the Windows-native port of runtime.c's ledger.  The counters remain
 * lock-free on ordinary submission/publication paths.  The SRW lock protects
 * only waiter order, awards, and the condition-variable decision which makes
 * a refused open's single retry unmissable.
 * ---------------------------------------------------------------------- */

#if !defined(WF_COMPLETION_RETIREMENT_POINT)
#define WF_COMPLETION_RETIREMENT_POINT wf_completion_retirement_point_absent
static void wf_completion_retirement_point_absent(void) {
}
#else
extern void WF_COMPLETION_RETIREMENT_POINT(void);
#endif

static SRWLOCK wf_retirement_lock = SRWLOCK_INIT;
static CONDITION_VARIABLE wf_retirement_signal = CONDITION_VARIABLE_INIT;
static _Atomic uint64_t
    wf_retirement_returns[WF_RETIREMENT_RESOURCE_COUNT];
static _Atomic size_t wf_retirement_in_flight;
static _Atomic size_t wf_retirement_waiter_count;
static _Atomic uint64_t wf_retirement_wait_starts;
static _Atomic(wf_retirement_waiter *) wf_retirement_first;
static wf_retirement_waiter *wf_retirement_last;
static uint64_t wf_retirement_awarded[WF_RETIREMENT_RESOURCE_COUNT];
static _Atomic(wf_completion_runtime *) wf_retirement_endpoint;

uint64_t wf_completion_descriptor_returns(void) {
    return wf_completion_resource_returns(WF_RETIREMENT_CRT_DESCRIPTOR);
}

uint64_t wf_completion_resource_returns(unsigned resource) {
    if (resource >= WF_RETIREMENT_RESOURCE_COUNT) {
        abort();
    }
    return atomic_load_explicit(
        &wf_retirement_returns[resource],
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

static void wf_retirement_announce_on_the_endpoint(void) {
    wf_completion_runtime *endpoint = atomic_load_explicit(
        &wf_retirement_endpoint,
        memory_order_seq_cst
    );
    if (endpoint != NULL) {
        wf_completion_notify_capacity(endpoint);
    }
}

static void wf_retirement_announce(void) {
    AcquireSRWLockExclusive(&wf_retirement_lock);
    WakeAllConditionVariable(&wf_retirement_signal);
    ReleaseSRWLockExclusive(&wf_retirement_lock);
    /* The core wake lock is always acquired after the retirement lock has
     * been released, so publication and park cannot form a lock cycle. */
    wf_retirement_announce_on_the_endpoint();
}

void wf_completion_operation_accepted(void) {
    atomic_fetch_add_explicit(
        &wf_retirement_in_flight,
        1,
        memory_order_seq_cst
    );
    if (atomic_load_explicit(
            &wf_retirement_waiter_count,
            memory_order_seq_cst
        ) != 0) {
        wf_retirement_announce();
    }
}

void wf_completion_resource_returned(unsigned resource) {
    if (resource >= WF_RETIREMENT_RESOURCE_COUNT) {
        abort();
    }
    atomic_fetch_add_explicit(
        &wf_retirement_returns[resource],
        1,
        memory_order_seq_cst
    );
    if (atomic_load_explicit(
            &wf_retirement_waiter_count,
            memory_order_seq_cst
        ) != 0) {
        wf_retirement_announce();
    }
}

void wf_completion_operation_retired_resources(unsigned returned_resources) {
    unsigned resource;
    if ((returned_resources
         & ~((unsigned)WF_RETIREMENT_RETURNED_NATIVE_HANDLE
             | (unsigned)WF_RETIREMENT_RETURNED_CRT_DESCRIPTOR)) != 0) {
        abort();
    }
    for (resource = 0; resource < WF_RETIREMENT_RESOURCE_COUNT; ++resource) {
        if ((returned_resources & (1u << resource)) == 0) {
            continue;
        }
        atomic_fetch_add_explicit(
            &wf_retirement_returns[resource],
            1,
            memory_order_seq_cst
        );
    }
    if (atomic_fetch_sub_explicit(
            &wf_retirement_in_flight,
            1,
            memory_order_seq_cst
        ) == 0) {
        abort();
    }
    if (atomic_load_explicit(
            &wf_retirement_waiter_count,
            memory_order_seq_cst
        ) != 0) {
        wf_retirement_announce();
    }
}

void wf_completion_operation_retired(int returned_a_descriptor) {
    wf_completion_operation_retired_resources(
        returned_a_descriptor != 0
            ? (unsigned)WF_RETIREMENT_RETURNED_NATIVE_HANDLE
                | (unsigned)WF_RETIREMENT_RETURNED_CRT_DESCRIPTOR
            : 0u
    );
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
    wf_completion_retirement_wait_begin_resource(
        waiter,
        seen,
        owed,
        owed_context,
        runs_owed,
        WF_RETIREMENT_CRT_DESCRIPTOR
    );
}

void wf_completion_retirement_wait_begin_resource(
    wf_retirement_waiter *waiter,
    uint64_t seen,
    size_t (*owed)(void *context),
    void *owed_context,
    int runs_owed,
    unsigned resource
) {
    wf_retirement_waiter *same_resource;
    if (waiter == NULL) {
        return;
    }
    if (resource >= WF_RETIREMENT_RESOURCE_COUNT) {
        abort();
    }
    waiter->seen = seen;
    waiter->owed = owed;
    waiter->owed_context = owed_context;
    waiter->resource = resource;
    waiter->runs_owed = runs_owed;
    waiter->awarded = 0;
    waiter->aside = 0;
    waiter->next = NULL;
    AcquireSRWLockExclusive(&wf_retirement_lock);
    same_resource = atomic_load_explicit(
        &wf_retirement_first,
        memory_order_relaxed
    );
    while (same_resource != NULL
           && same_resource->resource != resource) {
        same_resource = same_resource->next;
    }
    if (same_resource == NULL
        && seen > wf_retirement_awarded[resource]) {
        wf_retirement_awarded[resource] = seen;
    }
    if (wf_retirement_last == NULL) {
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
    WakeAllConditionVariable(&wf_retirement_signal);
    ReleaseSRWLockExclusive(&wf_retirement_lock);
    wf_retirement_announce_on_the_endpoint();
}

void wf_completion_retirement_wait_end(wf_retirement_waiter *waiter) {
    wf_retirement_waiter *scan;
    wf_retirement_waiter *previous = NULL;
    if (waiter == NULL) {
        return;
    }
    AcquireSRWLockExclusive(&wf_retirement_lock);
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
    WakeAllConditionVariable(&wf_retirement_signal);
    ReleaseSRWLockExclusive(&wf_retirement_lock);
    wf_retirement_announce_on_the_endpoint();
}

static size_t wf_retirement_owed(const wf_retirement_waiter *waiter) {
    if (waiter == NULL || waiter->owed == NULL) {
        return 0;
    }
    return waiter->owed(waiter->owed_context);
}

static int wf_retirement_must_run_its_own_work(
    const wf_retirement_waiter *waiter
) {
    return waiter != NULL && waiter->runs_owed != 0
        && wf_retirement_owed(waiter) != 0;
}

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

static int wf_retirement_an_earlier_waiter_is_owed(
    const wf_retirement_waiter *waiter,
    uint64_t returns
) {
    const wf_retirement_waiter *scan = atomic_load_explicit(
        &wf_retirement_first,
        memory_order_seq_cst
    );
    while (scan != NULL && scan != waiter) {
        if (scan->resource == waiter->resource && scan->seen != returns) {
            return 1;
        }
        scan = scan->next;
    }
    return 0;
}

static int wf_retirement_award_locked(
    wf_retirement_waiter *waiter,
    uint64_t returns,
    int award
) {
    if (waiter->awarded != 0) {
        return 1;
    }
    if (returns == waiter->seen
        || returns <= wf_retirement_awarded[waiter->resource]
        || wf_retirement_an_earlier_waiter_is_owed(waiter, returns) != 0) {
        return 0;
    }
    if (award != 0) {
        wf_retirement_awarded[waiter->resource] += 1;
        waiter->awarded = 1;
    }
    return 1;
}

void wf_completion_retirement_open_took_a_descriptor(int on_an_award) {
    wf_completion_retirement_open_took_resource(
        WF_RETIREMENT_NATIVE_HANDLE,
        0
    );
    wf_completion_retirement_open_took_resource(
        WF_RETIREMENT_CRT_DESCRIPTOR,
        on_an_award
    );
}

void wf_completion_retirement_open_took_resource(
    unsigned resource,
    int on_an_award
) {
    uint64_t returns;
    if (resource >= WF_RETIREMENT_RESOURCE_COUNT) {
        abort();
    }
    if (on_an_award != 0) {
        return;
    }
    AcquireSRWLockExclusive(&wf_retirement_lock);
    returns = atomic_load_explicit(
        &wf_retirement_returns[resource],
        memory_order_seq_cst
    );
    if (returns > wf_retirement_awarded[resource]) {
        wf_retirement_awarded[resource] += 1;
    }
    ReleaseSRWLockExclusive(&wf_retirement_lock);
}

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
        return WF_RETIREMENT_AWAITED;
    }
    in_flight = atomic_load_explicit(
        &wf_retirement_in_flight,
        memory_order_seq_cst
    );
    idle = atomic_load_explicit(
        &wf_retirement_waiter_count,
        memory_order_seq_cst
    );
    if (waiter->runs_owed == 0) {
        if (idle > SIZE_MAX - owed) {
            abort();
        }
        idle += owed;
    }
    if (in_flight > idle) {
        return WF_RETIREMENT_AWAITED;
    }
    if (wf_retirement_earliest_deciding() != waiter) {
        return WF_RETIREMENT_AWAITED;
    }
    if (wf_retirement_award_locked(
            waiter,
            atomic_load_explicit(
                &wf_retirement_returns[waiter->resource],
                memory_order_seq_cst
            ),
            award
        ) != 0) {
        return WF_RETIREMENT_HAPPENED;
    }
    return WF_RETIREMENT_UNREACHABLE;
}

void wf_completion_retirement_defer_begin(wf_retirement_waiter *waiter) {
    wf_retirement_waiter *promoted;
    int announce = 0;
    if (waiter == NULL) {
        return;
    }
    AcquireSRWLockExclusive(&wf_retirement_lock);
    promoted = wf_retirement_earliest_deciding() == waiter ? waiter : NULL;
    waiter->aside = 1;
    if (promoted != NULL) {
        promoted = wf_retirement_earliest_deciding();
    }
    if (promoted != NULL
        && wf_retirement_state_locked(
               promoted,
               atomic_load_explicit(
                   &wf_retirement_returns[promoted->resource],
                   memory_order_seq_cst
               ),
               0
           ) == WF_RETIREMENT_UNREACHABLE) {
        announce = 1;
        WakeAllConditionVariable(&wf_retirement_signal);
    }
    ReleaseSRWLockExclusive(&wf_retirement_lock);
    if (announce != 0) {
        wf_retirement_announce_on_the_endpoint();
    }
}

void wf_completion_retirement_defer_end(wf_retirement_waiter *waiter) {
    if (waiter == NULL) {
        return;
    }
    AcquireSRWLockExclusive(&wf_retirement_lock);
    waiter->aside = 0;
    ReleaseSRWLockExclusive(&wf_retirement_lock);
}

enum wf_retirement_state wf_completion_retirement_state(
    wf_retirement_waiter *waiter
) {
    enum wf_retirement_state state;
    uint64_t returns_before;
    if (waiter == NULL) {
        return WF_RETIREMENT_UNREACHABLE;
    }
    returns_before = atomic_load_explicit(
        &wf_retirement_returns[waiter->resource],
        memory_order_seq_cst
    );
    WF_COMPLETION_RETIREMENT_POINT();
    AcquireSRWLockExclusive(&wf_retirement_lock);
    state = wf_retirement_state_locked(waiter, returns_before, 1);
    ReleaseSRWLockExclusive(&wf_retirement_lock);
    return state;
}

void wf_completion_retirement_sleep(wf_retirement_waiter *waiter) {
    BOOL slept = TRUE;
    AcquireSRWLockExclusive(&wf_retirement_lock);
    if (wf_retirement_state_locked(
            waiter,
            atomic_load_explicit(
                &wf_retirement_returns[waiter->resource],
                memory_order_seq_cst
            ),
            0
        ) == WF_RETIREMENT_AWAITED
        && !wf_retirement_must_run_its_own_work(waiter)) {
        slept = SleepConditionVariableSRW(
            &wf_retirement_signal,
            &wf_retirement_lock,
            INFINITE,
            0
        );
    }
    ReleaseSRWLockExclusive(&wf_retirement_lock);
    if (slept == FALSE) {
        abort();
    }
}

#else

typedef int wf_windows_completion_translation_unit_is_target_guarded;

#endif
