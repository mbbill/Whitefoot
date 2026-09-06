/*
 * The wake epoch, and nothing else.
 *
 * What this unit was -- a pool of 256 slots, their generations, their
 * publication locks, the claim, the milestone product, the drain and the
 * consume -- is deleted rather than answered, because the record is a block
 * of the submitting frame and the engine that finishes an operation finds it
 * by its address (`research/investigations/io-model/PARK-ON-MISS.md` §7, "The
 * record's pool machinery: deleted, not answered").  A completion is published
 * straight into its record by `wf_completion_record_complete`, so there is no
 * event to raise, no event to scan, and no slot to recycle.
 *
 * What remains is the one thing the deletion does not reach: the epoch a
 * thread with nothing to do sleeps on, and the announcement that lets a
 * publisher know whether anyone is asleep on it (design §6).  It stays a unit
 * of its own rather than folding into the bridge because the Linux ring's own
 * park announces itself against this epoch under this lock
 * (`linux_io_uring.c`, `wf_linux_io_uring_park`), and the native adapter probe
 * links that ring without the bridge.
 *
 * One implementation for every platform.  The lock and the condition this unit
 * announces and sleeps on are the platform's one wait set, behind the six
 * calls `contract.h` declares and `wait_host.c` and `wait_windows.c` answer,
 * so nothing here is `#if`-forked and no host header reaches this file.
 */

#include "contract.h"

#include <errno.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

static void wf_completion_notify_scheduler(wf_completion_runtime *runtime);

int wf_completion_runtime_init(wf_completion_runtime *runtime) {
    int error;

    if (runtime == NULL) {
        return EINVAL;
    }

    memset(runtime, 0, sizeof(*runtime));
    atomic_init(&runtime->wake_epoch, 0);
    atomic_init(&runtime->parked_schedulers, 0);
    atomic_init(&runtime->stat_parks, 0);
    atomic_init(&runtime->stat_wake_signals, 0);
    atomic_init(&runtime->stat_compute_notifications, 0);
    atomic_init(&runtime->stat_target_notifications, 0);
    runtime->wake_callback = NULL;
    runtime->wake_context = NULL;

    error = wf_completion_wait_init(&runtime->wait);
    if (error != 0) {
        return error;
    }
    runtime->initialized = 1;
    return 0;
}

int wf_completion_set_wake_callback(
    wf_completion_runtime *runtime,
    wf_completion_wake_callback wake,
    void *context
) {
    if (runtime == NULL || runtime->initialized == 0 || wake == NULL) {
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
    int first_error;

    if (runtime == NULL || runtime->initialized == 0) {
        return EINVAL;
    }
    if (atomic_load_explicit(&runtime->parked_schedulers, memory_order_acquire)
        != 0) {
        return EBUSY;
    }
    first_error = wf_completion_wait_destroy(&runtime->wait);
    if (first_error == 0) {
        runtime->initialized = 0;
    }
    return first_error;
}

/* Raises the epoch and wakes whoever announced sleep against it.
 *
 * The publisher raises the epoch and then reads the sleeper count; a
 * scheduler about to sleep raises the sleeper count and then reads the epoch.
 * Both cannot read the old value, so either this call sees a sleeper and takes
 * the lock and wakes it, or the scheduler sees the new epoch and does not
 * sleep at all.  Both sides must stay sequentially consistent for that to
 * hold, which is why the two park paths -- this unit's and the Linux target's
 * external wait -- name the order explicitly at their own increment and
 * recheck. */
static void wf_completion_notify_scheduler(wf_completion_runtime *runtime) {
    atomic_fetch_add_explicit(&runtime->wake_epoch, 1, memory_order_seq_cst);
    if (atomic_load_explicit(&runtime->parked_schedulers, memory_order_seq_cst)
        == 0) {
        return;
    }
    wf_completion_wait_lock(&runtime->wait);
    {
        unsigned parked = atomic_load_explicit(
            &runtime->parked_schedulers,
            memory_order_relaxed
        );
        if (parked != 0) {
            /* An active scheduler observes the epoch or its record directly.
             * Only an announced sleeper needs an explicit host wake. External
             * target waits increment parked_schedulers under this same lock,
             * closing their empty-check -> sleep race. */
            if (runtime->wake_callback != NULL) {
                runtime->wake_callback(runtime->wake_context);
            }
            /* One waiter needs one signal. Two or more captured the same old
             * global epoch, so all of them must recheck after this transition. */
            wf_completion_wait_wake(&runtime->wait, parked != 1);
            atomic_fetch_add_explicit(
                &runtime->stat_wake_signals,
                1,
                memory_order_relaxed
            );
        }
    }
    wf_completion_wait_unlock(&runtime->wait);
}

uint64_t wf_completion_wake_epoch(const wf_completion_runtime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    return atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire);
}

void wf_completion_notify_compute(wf_completion_runtime *runtime) {
    if (runtime == NULL || runtime->initialized == 0) {
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
    if (runtime == NULL || runtime->initialized == 0) {
        return;
    }
    atomic_fetch_add_explicit(
        &runtime->stat_target_notifications,
        1,
        memory_order_relaxed
    );
    wf_completion_notify_scheduler(runtime);
}

enum wf_completion_park_result wf_completion_park_if_unchanged(
    wf_completion_runtime *runtime,
    uint64_t observed_epoch,
    uint32_t timeout_milliseconds
) {
    enum wf_completion_wait_result slept = WF_COMPLETION_WAIT_WOKEN;

    if (runtime == NULL || runtime->initialized == 0) {
        return WF_COMPLETION_PARK_FAILED;
    }

    wf_completion_wait_lock(&runtime->wait);
    if (atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire)
        != observed_epoch) {
        wf_completion_wait_unlock(&runtime->wait);
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
        wf_completion_wait_unlock(&runtime->wait);
        return WF_COMPLETION_PARK_EPOCH_CHANGED;
    }

    while (atomic_load_explicit(&runtime->wake_epoch, memory_order_acquire)
           == observed_epoch) {
        slept = wf_completion_wait_sleep(&runtime->wait, timeout_milliseconds);
        if (slept != WF_COMPLETION_WAIT_WOKEN) {
            break;
        }
    }
    atomic_fetch_sub_explicit(
        &runtime->parked_schedulers,
        1,
        memory_order_relaxed
    );
    wf_completion_wait_unlock(&runtime->wait);

    if (slept == WF_COMPLETION_WAIT_TIMED_OUT) {
        return WF_COMPLETION_PARK_TIMED_OUT;
    }
    if (slept != WF_COMPLETION_WAIT_WOKEN) {
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
    return statistics;
}
