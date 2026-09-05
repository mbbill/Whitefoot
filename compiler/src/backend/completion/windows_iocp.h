#ifndef WHITEFOOT_WINDOWS_IOCP_H
#define WHITEFOOT_WINDOWS_IOCP_H

#if defined(_WIN32)

/*
 * The Windows kernel completion ring, in the same shape as `linux_io_uring.h`.
 *
 * It finds the record by address: the `OVERLAPPED` a request is issued with is
 * embedded in the frame's own completion record, so the completion packet's
 * `OVERLAPPED` pointer *is* the record's identity, exactly as a CQE's
 * `user_data` is on Linux (design
 * `research/investigations/io-model/PARK-ON-MISS.md` §7, "What survives, and
 * the one change inside it").  There is no entry pool, no token, no
 * generation, and therefore no capacity this ring can refuse for.
 *
 * Its wait is the one park the bridge supplies through the
 * `wf__sched_host_epoch` / `_park` / `_wake` seam on Windows (§7, platform item
 * 2): the completion port itself, with a wake delivered as a
 * `PostQueuedCompletionStatus` packet, so a ready stack and an I/O completion
 * arrive on one queue and there is no second wake path to keep consistent.
 */

#include "contract.h"

#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <windows.h>

#if defined(__cplusplus)
extern "C" {
#endif

enum wf_windows_iocp_submit_result {
    WF_WINDOWS_IOCP_TARGET_OWNS = 0,
    WF_WINDOWS_IOCP_SUBMIT_INVALID = 1,
    WF_WINDOWS_IOCP_UNAVAILABLE = 2
};

typedef struct wf_windows_iocp_statistics {
    uint64_t submissions;
    /* Requests the kernel answered synchronously, published on the submitting
     * thread because the association asked for no packet in that case. */
    uint64_t inline_completions;
    uint64_t dequeued_completions;
    uint64_t completions;
    uint64_t kernel_waits;
    uint64_t host_wake_posts;
} wf_windows_iocp_statistics;

/* Target-private protocol state.  The completion port is not Whitefoot memory
 * and never crosses the generated-code ABI.  A request buffer crosses this
 * boundary only inside a `wf_completion_record`, which is a block of the frame
 * that submitted the operation and outlives it. */
typedef struct wf_windows_iocp_adapter {
    wf_completion_runtime *runtime;
    HANDLE port;
    ULONG_PTR wake_key;
    ULONG_PTR file_key;
    _Atomic size_t in_flight;
    _Atomic int progress_error;
    unsigned initialized;

    _Atomic uint64_t stat_submissions;
    _Atomic uint64_t stat_inline_completions;
    _Atomic uint64_t stat_dequeued_completions;
    _Atomic uint64_t stat_completions;
    _Atomic uint64_t stat_kernel_waits;
    _Atomic uint64_t stat_host_wake_posts;
} wf_windows_iocp_adapter;

/* Creates the one completion port this process's ring uses.  `concurrency` is
 * the port's own concurrency hint; zero asks the kernel for its default. */
int wf_windows_iocp_init(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    DWORD concurrency
);

/* Associates one file handle with the port, once, and asks the kernel not to
 * queue a packet for a request it answers synchronously -- that request is
 * already complete, so the submitting thread publishes it instead of paying
 * for a port round trip. */
int wf_windows_iocp_associate(
    wf_windows_iocp_adapter *adapter,
    HANDLE handle
);

/* Whether this ring has a form for one record's request kind.  Asked by the
 * bridge before the record is handed over, so a kind the ring does not carry
 * reaches the bounded adapter or the inline executor instead of being refused
 * after the operation was already the ring's. */
int wf_windows_iocp_carries(const wf_completion_record *record);

/* Hands one record to the ring, on a handle the port has taken.  It cannot
 * answer capacity: the record is the frame's and the port holds no pool. */
enum wf_windows_iocp_submit_result wf_windows_iocp_submit(
    wf_windows_iocp_adapter *adapter,
    wf_completion_record *record,
    HANDLE handle
);

/* Dequeues at most `budget` ready packets without waiting and publishes the
 * terminal completions among them.  A wake packet found here belongs to a
 * sleeper: it is put back when one is announced and dropped when none is,
 * because the epoch and not the packet is the durable wake fact. */
int wf_windows_iocp_progress(
    wf_windows_iocp_adapter *adapter,
    size_t budget,
    size_t *published
);

/* Announces any completion-core epoch change to the port.  This is the
 * callback installed with `wf_completion_set_wake_callback`; it posts one
 * packet per announced sleeper and never runs a continuation. */
void wf_windows_iocp_notify(void *context);

/* Parks one thread on the port.  `observed_epoch` is rechecked under the
 * runtime's own wait lock before the kernel wait, closing
 * completion-before-park without a polling timeout.  UINT32_MAX means no
 * deadline. */
int wf_windows_iocp_park(
    wf_windows_iocp_adapter *adapter,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
);

/* A nonzero value is a sticky target-owned progress failure.  Once an
 * operation has been accepted, the bridge must fail-stop on this condition. */
int wf_windows_iocp_progress_error(const wf_windows_iocp_adapter *adapter);

size_t wf_windows_iocp_in_flight(const wf_windows_iocp_adapter *adapter);

/* Refuses with EBUSY until every accepted operation has produced and published
 * its packet. */
int wf_windows_iocp_destroy(wf_windows_iocp_adapter *adapter);

wf_windows_iocp_statistics wf_windows_iocp_statistics_snapshot(
    const wf_windows_iocp_adapter *adapter
);

#if defined(__cplusplus)
}
#endif

#endif

#endif
