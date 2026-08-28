#ifndef WHITEFOOT_COMPLETION_BRIDGE_H
#define WHITEFOOT_COMPLETION_BRIDGE_H

#include <stdint.h>
#include "writer_scheduler.h"

#if defined(__cplusplus)
extern "C" {
#endif

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
    void *token_storage
);

int wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage
);

int wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
);

int wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    void *token_storage
);

int wf__completion_file_status_submit(
    int descriptor,
    void *token_storage
);

int wf__completion_file_close_submit(
    int descriptor,
    void *token_storage
);

int wf__completion_directory_next_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position,
    void *token_storage
);

/* Stackless writer-continuation variants. `frame` is opaque scheduler state;
 * target code may only make it ready through the completion dependency. */
int wf__completion_file_pread_submit_writer(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage,
    void *frame
);

int wf__completion_file_write_submit_writer(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage,
    void *frame
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
    int *error_code,
    unsigned *open_outcome
);

void wf__completion_file_open_join(
    const void *token_storage,
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

/* Darwin's qualified directory facility through the same target-progress
 * normalization as typed file operations. EINTR and readiness refusal never
 * cross this ABI as writer-visible errors. */
int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
);

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
);

void wf__completion_file_status_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
);

int wf__completion_file_take(
    const void *token_storage,
    int64_t *value,
    int *error_code
);

int wf__completion_file_take_status(
    const void *token_storage,
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

#if defined(__cplusplus)
}
#endif

#endif
