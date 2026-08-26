#ifndef WHITEFOOT_COMPLETION_BRIDGE_H
#define WHITEFOOT_COMPLETION_BRIDGE_H

#include <stdint.h>
#include "writer_scheduler.h"

#define WF_ORDERED_OUTPUT_ROOT_CAPACITY 16u
#define WF_ORDERED_OUTPUT_MEMBER_CAPACITY 16u
#define WF_FILE_BATCH_MEMBER_CAPACITY 64u

#if defined(__cplusplus)
extern "C" {
#endif

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

/* Reserves one independent file group all-or-none before any member owns a
 * completion slot. `token_storage` is a contiguous token array. */
int wf__completion_file_batch_claim(
    void *token_storage,
    uint32_t count,
    uint32_t requires_fallback
);
void wf__completion_file_pread_submit_reserved(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage
);
void wf__completion_file_write_submit_reserved(
    int descriptor,
    const void *buffer,
    uint64_t count,
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

/* Darwin's qualified directory facility through the same target-progress
 * normalization as typed file operations. EINTR and readiness refusal never
 * cross this ABI as writer-visible errors. */
int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
);

uint64_t wf__completion_output_batch_begin(
    uint64_t logical_root,
    uint32_t expected
);
void wf__completion_output_batch_submit(
    uint64_t root_key,
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
);
void wf__completion_output_batch_commit(uint64_t root_key);

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
);

int wf__completion_file_take(
    const void *token_storage,
    int64_t *value,
    int *error_code
);

void wf__writer_run_root(void *frame);

uint64_t wf__completion_file_submissions(void);
uint64_t wf__completion_file_fallback_submissions(void);
uint64_t wf__completion_file_helper_executions(void);
uint64_t wf__completion_target_helper_count(void);
uint64_t wf__completion_target_helper_executions(void);
uint64_t wf__completion_publications(void);
uint64_t wf__completion_linux_io_uring_submissions(void);

#if defined(__cplusplus)
}
#endif

#endif
