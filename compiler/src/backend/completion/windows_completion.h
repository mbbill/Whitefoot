#ifndef WHITEFOOT_WINDOWS_COMPLETION_H
#define WHITEFOOT_WINDOWS_COMPLETION_H

#if defined(_WIN32)

#include "native_completion_api.h"

#include <stddef.h>
#include <stdint.h>
#include <windows.h>

#if defined(__cplusplus)
extern "C" {
#endif

struct wf_windows_iocp_adapter;

enum wf_completion_phase {
    WF_COMPLETION_FREE = 0,
    WF_COMPLETION_READY = 1,
    WF_COMPLETION_WAIT_CAPACITY = 2,
    WF_COMPLETION_SUBMITTING = 3,
    WF_COMPLETION_IN_FLIGHT = 4,
    WF_COMPLETION_TERMINAL_PHASE = 5,
    WF_COMPLETION_RETIRED = 6
};

enum wf_completion_claim_result {
    WF_COMPLETION_CLAIMED = 0,
    WF_COMPLETION_CLAIM_WAIT_CAPACITY = 1,
    WF_COMPLETION_CLAIM_INVALID = 2
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

typedef void (*wf_completion_ready_callback)(void *frame);

enum wf_completion_park_result {
    WF_COMPLETION_PARK_WOKEN = 0,
    WF_COMPLETION_PARK_EPOCH_CHANGED = 1,
    WF_COMPLETION_PARK_TIMED_OUT = 2,
    WF_COMPLETION_PARK_FAILED = 3
};

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

typedef struct wf_completion_slot {
    SRWLOCK publication_lock;
    uint64_t generation;
    unsigned phase;
    uint32_t milestones;
    volatile LONG event_pending;
    unsigned event_drained;
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
    uint64_t capacity_notifications;
} wf_completion_statistics;

struct wf_completion_runtime {
    wf_completion_slot *slots;
    size_t slot_count;
    volatile LONG64 claim_cursor;
    volatile LONG64 drain_cursor;
    volatile LONG64 ready_events;

    SRWLOCK claim_lock;
    SRWLOCK wake_lock;
    CONDITION_VARIABLE wake_condition;
    volatile LONG64 wake_epoch;
    volatile LONG parked_schedulers;
    volatile LONG iocp_waiters;
    volatile LONG iocp_wake_packets;

    /* When bound, compute and completion publication keep enough persistent
     * wake packets for every announced IOCP waiter. `iocp_wake_packets` is the
     * exact count successfully posted but not yet reported consumed by the
     * adapter; repeated notifications only fill a deficit and therefore
     * coalesce. A real completion may retire its waiter before an older wake
     * packet is dequeued; that bounded surplus remains reusable. */
    HANDLE wake_port;
    ULONG_PTR wake_key;

    volatile LONG64 stat_claims;
    volatile LONG64 stat_claim_capacity_waits;
    volatile LONG64 stat_target_capacity_waits;
    volatile LONG64 stat_publications;
    volatile LONG64 stat_stale_publications;
    volatile LONG64 stat_duplicate_terminals;
    volatile LONG64 stat_drained_events;
    volatile LONG64 stat_consumptions;
    volatile LONG64 stat_parks;
    volatile LONG64 stat_wake_signals;
    volatile LONG64 stat_compute_notifications;
    volatile LONG64 stat_capacity_notifications;
    wf_completion_ready_callback ready_frame;
};

int wf_completion_runtime_init(
    wf_completion_runtime *runtime,
    wf_completion_slot *slots,
    size_t slot_count
);
int wf_completion_runtime_destroy(wf_completion_runtime *runtime);
int wf_completion_set_ready_callback(
    wf_completion_runtime *runtime,
    wf_completion_ready_callback ready
);

/* Binding never transfers ownership of `port`. `wake_key` must be the value
 * returned by wf_windows_iocp_wake_key for the adapter which owns that port.
 * A null-OVERLAPPED packet is a scheduler wake, never an I/O result. */
int wf_windows_completion_bind_iocp(
    wf_completion_runtime *runtime,
    struct wf_windows_iocp_adapter *adapter,
    HANDLE port,
    ULONG_PTR wake_key
);

/* Announce/recheck for a scheduler about to wait in
 * GetQueuedCompletionStatus. A notification fills the outstanding packet
 * count to the announced waiter count while holding the same lock used here,
 * closing the final race without accumulating one packet per notification.
 * A bound adapter's progress call withdraws the announcement automatically on
 * its first dequeue result. `wait_end` is only for cancelling a successful
 * announcement before entering progress. */
enum wf_completion_park_result wf_windows_completion_iocp_wait_begin(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch
);
void wf_windows_completion_iocp_wait_end(wf_completion_runtime *runtime);

unsigned wf_windows_completion_iocp_waiter_count(
    const wf_completion_runtime *runtime
);
unsigned wf_windows_completion_iocp_wake_packet_count(
    const wf_completion_runtime *runtime
);

enum wf_completion_claim_result wf_completion_claim(
    wf_completion_runtime *runtime,
    wf_completion_token *token
);

size_t wf_completion_drain(
    wf_completion_runtime *runtime,
    wf_completion_event *events,
    size_t event_capacity,
    size_t scan_budget
);
size_t wf_completion_ready_event_count(const wf_completion_runtime *runtime);

enum wf_completion_transition_result wf_completion_observe(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t *milestones,
    unsigned *phase
);

enum wf_completion_depend_result wf_completion_depend(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t requirement,
    void *frame
);

enum wf_completion_consume_result wf_completion_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    void *result,
    size_t result_capacity,
    wf_completion_outcome *outcome
);
enum wf_completion_consume_wait_result wf_completion_wait_to_consume(
    wf_completion_runtime *runtime,
    wf_completion_token token
);

uint64_t wf_completion_wake_epoch(const wf_completion_runtime *runtime);
void wf_completion_notify_compute(wf_completion_runtime *runtime);
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
_Static_assert(
    _Alignof(wf_completion_token) >= _Alignof(uint64_t),
    "completion token alignment"
);

#if defined(__cplusplus)
}
#endif

#endif

#endif
