#ifndef WHITEFOOT_COMPLETION_BRIDGE_H
#define WHITEFOOT_COMPLETION_BRIDGE_H

#include <stdint.h>
#include "writer_scheduler.h"

#if defined(__cplusplus)
extern "C" {
#endif

/* Stable integer verdicts for target submit implementations that expose core
 * backpressure.  The Windows positioned-read submit uses all three in this
 * runtime slice.  On that path DIRECT_ONLY means the request cannot use IOCP
 * and leaves the token untouched.  ACCEPTED transfers the request to IOCP and
 * returns with a valid token.  WAIT_CORE_CAPACITY also leaves the token
 * untouched so the source owner can consume one of its older tokens and retry
 * the same request. */
enum wf_completion_submit_verdict {
    WF_COMPLETION_SUBMIT_DIRECT_ONLY = 0,
    WF_COMPLETION_SUBMIT_ACCEPTED = 1,
    WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY = 2
};

/* Called only after WAIT_CORE_CAPACITY when the source owner has no earlier
 * accepted request left to consume. It helps scheduler work or waits for a
 * capacity notification, then returns so the caller can retry the exact
 * submission. */
void wf__completion_wait_core_capacity(void);

/* How many iterations of one loop the runtime will carry in flight at once,
 * asked once per loop entry and never per iteration.  `span` is the loop's
 * statically known trip count, `slot_bytes` the private storage one in-flight
 * iteration owns, and `ceiling` the compiler's own static cap; a zero in any
 * of the three places no bound.  One is always a legal answer and reproduces
 * the sequential program exactly, which is why a link without this unit gets a
 * weak fallback returning one.  Nothing a writer can spell reaches this. */
uint64_t wf__completion_window(
    uint64_t span,
    uint64_t slot_bytes,
    uint64_t ceiling
);

int wf__completion_file_read_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *record
);

int wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *record
);

int wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *record
);

int wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
#if defined(_WIN32)
    unsigned descriptor_class,
#endif
    void *record
);

int wf__completion_file_status_submit(
    int descriptor,
    void *record
);

int wf__completion_file_close_submit(
    int descriptor,
    void *record
);

int wf__completion_directory_next_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position,
    void *record
);

int64_t wf__completion_file_pread_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t file_offset
);

int64_t wf__completion_file_write_direct(
    int descriptor,
    const void *buffer,
    uint64_t count
);

int wf__completion_file_open_at_direct(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
#if defined(_WIN32)
    unsigned descriptor_class,
#endif
    int *error_code,
    unsigned *open_outcome
);

void wf__completion_file_open_join(
    const void *record,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
);

int wf__completion_file_status_direct(
    int descriptor,
    void *status,
    uint64_t status_capacity
);

int wf__completion_file_close_direct(int descriptor);

/* The selected family's qualified directory-enumeration facility through the
 * same target-progress normalization as typed file operations --
 * `__getdirentries64` on Darwin, `getdents64` on Linux.  EINTR and readiness
 * refusal never cross this ABI as writer-visible errors.  `position` is the
 * base-position cell Darwin's facility requires; Linux keeps the whole cursor
 * in the descriptor and leaves the cell untouched.  The native record the
 * batch holds differs by family and is decoded by the emitted shim, not
 * here. */
int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
);

void wf__completion_file_join(
    const void *record,
    int64_t *value,
    int *error_code
);

void wf__completion_file_status_join(
    const void *record,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
);

int wf__completion_file_take(
    const void *record,
    int64_t *value,
    int *error_code
);

int wf__completion_file_take_status(
    const void *record,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
);

void wf__writer_run_root(void *frame);

uint64_t wf__completion_file_submissions(void);
uint64_t wf__completion_file_fallback_submissions(void);
/* Opens no operation record could hold the name of, which their callers
 * performed as direct blocking opens instead.  The outcome of each is the
 * open it asked for; what it lost is the completion path. */
uint64_t wf__completion_file_demoted_opens(void);
uint64_t wf__completion_file_helper_executions(void);
uint64_t wf__completion_target_helper_count(void);
uint64_t wf__completion_target_helper_executions(void);
uint64_t wf__completion_publications(void);
uint64_t wf__completion_linux_io_uring_submissions(void);
/* `io_uring_enter` calls that carried staged submissions to the kernel.  The
 * doorbell is deferred, so this is far below the submission count and the
 * distance between them is what deferring bought. */
uint64_t wf__completion_linux_io_uring_submission_enters(void);

#if defined(__cplusplus)
}
#endif

#endif
