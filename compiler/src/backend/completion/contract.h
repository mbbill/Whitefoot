#ifndef WHITEFOOT_COMPLETION_CONTRACT_H
#define WHITEFOOT_COMPLETION_CONTRACT_H

/*
 * Whitefoot completion contract.
 *
 * A frame is caller-owned storage.  Its mailbox node is part of the frame, so
 * submitting an operation allocates neither a request nor a completion.  The
 * caller must initialize a frame once, may submit it only while FREE/REAPED,
 * and may reuse or release the surrounding storage only after wf_io_wait has
 * observed terminal completion.  The generation carried by each publication
 * makes a late publication distinguishable from the frame's next use.
 *
 * This header is shared by the compiler's completion adapters, the parallel
 * runtime's I/O-frame route, and the standalone adversarial harness.  Backend
 * selection is a build decision: no source value or operation ID reaches it.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdatomic.h>

#if defined(_WIN32)
#define WF_IO_EXPORT __declspec(dllexport)
#else
#define WF_IO_EXPORT
#endif

#define WF_IO_MAX_LANES 128u
#define WF_IO_DISK_THREADS 4u

enum wf_io_frame_state {
    WF_IO_FRAME_FREE = 0,
    WF_IO_FRAME_PREPARING = 1,
    WF_IO_FRAME_SUBMITTED = 2,
    WF_IO_FRAME_EXECUTING = 3,
    WF_IO_FRAME_COMPLETING = 4,
    WF_IO_FRAME_TERMINAL = 5,
    WF_IO_FRAME_REAPED = 6
};

enum wf_io_loan_state {
    WF_IO_LOAN_RETURNED = 0,
    WF_IO_LOAN_IN_FLIGHT = 1,
    WF_IO_LOAN_TERMINAL = 2
};

enum wf_io_backend_feature {
    WF_IO_BACKEND_KQUEUE = 1u << 0,
    WF_IO_BACKEND_WAITER_THREAD = 1u << 1,
    WF_IO_BACKEND_IO_URING = 1u << 2,
    WF_IO_BACKEND_EVENTFD_POLL = 1u << 3,
    WF_IO_BACKEND_POLL_MULTISHOT = 1u << 4,
    WF_IO_BACKEND_POLL_REARM = 1u << 5,
    WF_IO_BACKEND_UNIFIED_PARK = 1u << 6,
    WF_IO_BACKEND_IOCP = 1u << 7,
    WF_IO_BACKEND_FIXED_DISK_POOL = 1u << 8
};

typedef struct wf_io_result {
    intptr_t value;
    int error_code;
} wf_io_result;

typedef wf_io_result (*wf_io_work_fn)(void *context);
typedef int (*wf_io_help_fn)(void *context);

/*
 * The fields are public only so C callers can reserve exact stack/static
 * storage.  They are runtime-owned after wf_io_submit and must be read through
 * the accessors below.  work_next is protected by the disk-queue mutex;
 * mailbox_next is the intrusive release/acquire MPSC node.
 */
typedef struct wf_io_frame {
    _Atomic unsigned state;
    _Atomic unsigned loan;
    _Atomic uint64_t generation;
    _Atomic uint64_t published_generation;
    _Atomic uint64_t observed_generation;
    _Atomic(struct wf_io_frame *) mailbox_next;
    struct wf_io_frame *work_next;
    wf_io_work_fn work;
    void *context;
    intptr_t result;
    int error_code;
    _Atomic unsigned owner_lane;
} wf_io_frame;

typedef struct wf_io_statistics {
    uint64_t submissions;
    uint64_t submission_failures;
    uint64_t executions;
    uint64_t completions;
    uint64_t mailbox_batches;
    uint64_t parks;
    uint64_t wakes;
    uint64_t help_calls;
    uint64_t stale_publications;
    uint64_t duplicate_publications;
    uint64_t backend_poll_arms;
} wf_io_statistics;

enum wf_io_test_publication_result {
    WF_IO_TEST_PUBLICATION_VALID = 0,
    WF_IO_TEST_PUBLICATION_STALE = 1,
    WF_IO_TEST_PUBLICATION_DUPLICATE = 2,
    WF_IO_TEST_PUBLICATION_MALFORMED = 3
};

WF_IO_EXPORT void wf_io_frame_init(wf_io_frame *frame);
WF_IO_EXPORT int wf_io_submit(wf_io_frame *frame, wf_io_work_fn work, void *context);
WF_IO_EXPORT void wf_io_wait(wf_io_frame *frame);

WF_IO_EXPORT unsigned wf_io_frame_state(const wf_io_frame *frame);
WF_IO_EXPORT unsigned wf_io_frame_loan(const wf_io_frame *frame);
WF_IO_EXPORT uint64_t wf_io_frame_generation(const wf_io_frame *frame);
WF_IO_EXPORT intptr_t wf_io_frame_result(const wf_io_frame *frame);
WF_IO_EXPORT int wf_io_frame_error(const wf_io_frame *frame);
WF_IO_EXPORT unsigned wf_io_frame_owner_lane(const wf_io_frame *frame);

/* At most one unrelated compute frame is helped per wait round, and nested
 * completion waits never invoke the hook recursively. */
WF_IO_EXPORT void wf_io_install_help_hook(wf_io_help_fn help, void *context);

WF_IO_EXPORT const char *wf_io_backend_name(void);
WF_IO_EXPORT unsigned wf_io_backend_features(void);
WF_IO_EXPORT wf_io_statistics wf_io_statistics_snapshot(void);
WF_IO_EXPORT int wf_io_is_disk_worker(void);

#if !defined(_WIN32)
/* Native-host adapters used by qualified Unix command targets.  Each adapter
 * captures errno on the disk lane and restores it on the submitting lane. */
WF_IO_EXPORT int wf__io_openat(int directory, const char *path, int flags, ...);
WF_IO_EXPORT intptr_t wf__io_read(int descriptor, void *buffer, uintptr_t count);
WF_IO_EXPORT intptr_t wf__io_write(int descriptor, const void *buffer, uintptr_t count);
WF_IO_EXPORT int wf__io_fstat(int descriptor, void *status);
WF_IO_EXPORT int wf__io_close(int descriptor);
#if defined(__APPLE__)
WF_IO_EXPORT intptr_t wf__io_getdirentries64(
    int descriptor,
    void *buffer,
    uintptr_t count,
    void *position
);
#endif
#endif

#if defined(WF_IO_TESTING)
/* Fault-injection and validator entries are test-only; production adapters do
 * not expose a way to fabricate a completion. */
WF_IO_EXPORT void wf_io_test_fail_next_submission(void);
WF_IO_EXPORT int wf_io_test_validate_publication(wf_io_frame *frame, uint64_t generation);
#endif

#endif
