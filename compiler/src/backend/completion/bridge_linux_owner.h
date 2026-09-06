#ifndef WHITEFOOT_BRIDGE_LINUX_OWNER_H
#define WHITEFOOT_BRIDGE_LINUX_OWNER_H

/* Experimental Linux bridge policy. Included inside bridge.c after its
 * shared runtime state; retained only for the owner-local I/O comparison. */

/* Each core worker owns its ring. Host callers outside the core share the
 * final slot and retain the adapter's submission/completion locks. Slots are
 * initialized once, before their first record is offered, on the caller's
 * own thread. A pure-compute worker never creates a ring. */
typedef struct wf_bridge_linux_slot {
    wf_linux_io_uring_adapter adapter;
    unsigned once;
    _Atomic unsigned ready;
    _Atomic unsigned reportable;
} wf_bridge_linux_slot;
static wf_bridge_linux_slot wf_bridge_linux_slots[WF_SCHED_MAX_THREADS + 1u];
static _Atomic unsigned wf_bridge_doorbell_ready;

static wf_bridge_linux_slot *wf_bridge_linux_current(void) {
    unsigned index = wf__sched_current_stack() != NULL
        ? wf_prim_thread_index() : WF_SCHED_MAX_THREADS;
    return &wf_bridge_linux_slots[index];
}

static int wf_bridge_ring_ready(void) {
    return atomic_load_explicit(&wf_bridge_doorbell_ready, memory_order_acquire) != 0u;
}

static void wf_bridge_initialize_current_ring(void) {
    wf_bridge_linux_slot *slot = wf_bridge_linux_current();
    if (wf_linux_io_uring_init(&slot->adapter, &wf_bridge_runtime,
            WF_LINUX_IO_URING_DEPTH, WF_LINUX_IO_URING_COMPLETIONS) == 0) {
        atomic_store_explicit(&slot->reportable, 1u, memory_order_release);
        atomic_store_explicit(&slot->ready, 1u, memory_order_release);
    }
}

/* The existing global epoch still covers every scheduler. Notify each ring
 * that has announced sleepers, under that epoch's wait lock. A ring created
 * concurrently is either published here or has not yet announced sleep. */
static void wf_bridge_linux_notify_all(void *context) {
    (void)context;
    for (unsigned index = 0; index <= WF_SCHED_MAX_THREADS; ++index) {
        wf_bridge_linux_slot *slot = &wf_bridge_linux_slots[index];
        if (atomic_load_explicit(&slot->ready, memory_order_acquire) != 0u
            && slot->adapter.parked_schedulers != 0u) {
            wf_linux_io_uring_notify(&slot->adapter);
        }
    }
}

static int wf_bridge_ring_start(void) {
    wf_bridge_linux_slot *slot = wf_bridge_linux_current();
    if (wf_bridge_native_ring_refused()) return 0;
    wf__sched_once(&slot->once, wf_bridge_initialize_current_ring);
    if (atomic_load_explicit(&slot->ready, memory_order_acquire) == 0u) return 0;
    if (wf_completion_set_wake_callback(&wf_bridge_runtime,
            wf_bridge_linux_notify_all, NULL) != 0) {
        (void)wf_linux_io_uring_destroy(&slot->adapter);
        atomic_store_explicit(&slot->ready, 0u, memory_order_release);
        return 0;
    }
    atomic_store_explicit(&wf_bridge_doorbell_ready, 1u, memory_order_release);
    return 1;
}

static int wf_bridge_ring_offer(wf_completion_record *record) {
    wf_bridge_linux_slot *slot = wf_bridge_linux_current();
    if (!wf_linux_io_uring_carries(record)) return 0;
    if (wf_bridge_ring_ready()) {
        wf__sched_once(&slot->once, wf_bridge_initialize_current_ring);
    }
    if (atomic_load_explicit(&slot->ready, memory_order_acquire) == 0u) {
        return 0;
    }
    if (wf_linux_io_uring_submit(&slot->adapter, record)
        != WF_LINUX_IO_URING_TARGET_OWNS) {
        /* The kind and shape were both answered before the record was
         * offered, so any other answer is a target-runtime failure. */
        wf_bridge_fail(
            "the io_uring target refused a record whose kind and shape it had already accepted"
        );
    }
    {
        int progress_error =
            wf_linux_io_uring_progress_error(&slot->adapter);
        if (progress_error != 0) {
            wf_bridge_fail_with_code(
                "the io_uring target reported a failure while submitting",
                progress_error
            );
        }
    }
    return 1;
}

static int wf_bridge_ring_progress(void) {
    wf_bridge_linux_slot *slot = wf_bridge_linux_current();
    size_t published = 0;
    if (atomic_load_explicit(&slot->ready, memory_order_acquire) == 0u) {
        return 0;
    }
    {
        int reap_error = wf_linux_io_uring_progress(
            &slot->adapter,
            WF_BRIDGE_REAP_BUDGET,
            0,
            &published
        );
        if (reap_error != 0) {
            /* Target ownership has already transferred. Falling back now would
             * duplicate an operation, and ignoring the error would strand its
             * owned operation forever. A target-runtime failure is a fail-stop
             * TCB defect, not a writer-visible IoError. */
            wf_bridge_fail_with_code(
                "the io_uring target failed while reaping completions",
                reap_error
            );
        }
    }
    return published != 0;
}

static void wf_bridge_ring_flush(void) {
    wf_bridge_linux_slot *slot = wf_bridge_linux_current();
    if (atomic_load_explicit(&slot->ready, memory_order_acquire) != 0u) {
        (void)wf_linux_io_uring_flush(&slot->adapter);
    }
}

static int wf_bridge_ring_park(uint64_t observed_epoch) {
    wf_bridge_linux_slot *slot = wf_bridge_linux_current();
    if (atomic_load_explicit(&slot->ready, memory_order_acquire) == 0u) {
        return 0;
    }
    {
        int park_error = wf_linux_io_uring_park(
            &slot->adapter,
            observed_epoch,
            UINT32_MAX
        );
        if (park_error != 0) {
            wf_bridge_fail_with_code(
                "the io_uring target failed while parking on the ring",
                park_error
            );
        }
    }
    return 1;
}

static void wf_bridge_ring_shutdown(void) {
    atomic_store_explicit(&wf_bridge_doorbell_ready, 0u, memory_order_release);
    for (unsigned index = 0; index <= WF_SCHED_MAX_THREADS; ++index) {
        wf_bridge_linux_slot *slot = &wf_bridge_linux_slots[index];
        if (atomic_load_explicit(&slot->ready, memory_order_acquire) != 0u) {
            (void)wf_linux_io_uring_destroy(&slot->adapter);
            atomic_store_explicit(&slot->ready, 0u, memory_order_release);
        }
    }
}

static wf_linux_io_uring_statistics wf_bridge_ring_statistics(unsigned *rings) {
    wf_linux_io_uring_statistics sum = {0};
    *rings = 0u;
    for (unsigned index = 0; index <= WF_SCHED_MAX_THREADS; ++index) {
        wf_bridge_linux_slot *slot = &wf_bridge_linux_slots[index];
        if (atomic_load_explicit(&slot->reportable, memory_order_acquire) == 0u) continue;
        wf_linux_io_uring_statistics part = wf_linux_io_uring_statistics_snapshot(&slot->adapter);
        ++*rings;
        sum.submissions += part.submissions;
        sum.submission_enters += part.submission_enters;
        sum.completions += part.completions;
        sum.enter_failures += part.enter_failures;
        sum.kernel_waits += part.kernel_waits;
        sum.kernel_wakes += part.kernel_wakes;
        sum.host_wake_writes += part.host_wake_writes;
        sum.overflow_flushes += part.overflow_flushes;
    }
    return sum;
}

static uint64_t wf_bridge_ring_submissions(void) {
    unsigned rings;
    return wf_bridge_ring_statistics(&rings).submissions;
}

static uint64_t wf_bridge_ring_submission_enters(void) {
    unsigned rings;
    return wf_bridge_ring_statistics(&rings).submission_enters;
}

/* The ring's counters as one line, for the same observer that prints the
 * core's: what the kernel was asked and how often a thread slept for it. */
int wf__bridge_report(char *buffer, size_t capacity) {
    wf_linux_io_uring_statistics ring;
    int written;
    unsigned rings;
    ring = wf_bridge_ring_statistics(&rings);
    if (buffer == NULL || capacity == 0u || rings == 0u) {
        return 0;
    }
    written = snprintf(
        buffer,
        capacity,
        "ring: submissions=%llu submission_enters=%llu completions=%llu "
        "kernel_waits=%llu kernel_wakes=%llu host_wake_writes=%llu "
        "overflow_flushes=%llu runtime_parks=%llu inline=%llu tcp_nodelay=%u owner_rings=%u",
        (unsigned long long)ring.submissions,
        (unsigned long long)ring.submission_enters,
        (unsigned long long)ring.completions,
        (unsigned long long)ring.kernel_waits,
        (unsigned long long)ring.kernel_wakes,
        (unsigned long long)ring.host_wake_writes,
        (unsigned long long)ring.overflow_flushes,
        (unsigned long long)atomic_load_explicit(
            &wf_bridge_runtime.stat_parks,
            memory_order_relaxed
        ),
        (unsigned long long)atomic_load_explicit(
            &wf_bridge_inline_executions,
            memory_order_relaxed
        ),
        (unsigned)WF_TCP_NODELAY, rings
    );
    return written > 0 && (size_t)written < capacity;
}

#endif
