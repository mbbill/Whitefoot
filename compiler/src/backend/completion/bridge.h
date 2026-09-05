#ifndef WHITEFOOT_COMPLETION_BRIDGE_H
#define WHITEFOOT_COMPLETION_BRIDGE_H

#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

#if defined(_WIN32)
/* The writer-frame ready queue, which POSIX retires here and Windows has not
 * yet (`research/investigations/io-model/PARK-ON-MISS.md` §7: its handshake
 * moves to the scheduler core's stack park, and both writer schedulers go with
 * both parallel runtimes).  The Windows twins -- `windows_bridge.c` and
 * `writer_scheduler_windows.c` -- are ported as shared code in a later step
 * and still name this ABI, so its declarations stay here, behind the platform
 * test, rather than in a header of their own that no POSIX link would have a
 * use for.  Nothing outside `#if defined(_WIN32)` may name any of it. */
typedef void (*wf__writer_resume_fn)(void *frame);

#define WF_WRITER_HEADER_STORAGE_BYTES 64u
#define WF_WRITER_HEADER_STORAGE_ALIGNMENT 8u
#define WF_COMPLETION_SLOT_CAPACITY 64u
#define WF_WRITER_READY_CAPACITY WF_COMPLETION_SLOT_CAPACITY

void wf__writer_frame_init(void *frame);
void wf__writer_begin_suspend(void *frame, wf__writer_resume_fn resume);
int wf__writer_commit_suspend(void *frame);
void wf__writer_cancel_suspend(void *frame);
void wf__writer_complete(void *frame);
void wf__writer_scheduler_ready(void *frame);
int wf__writer_scheduler_help_once(void);
void wf__writer_scheduler_prepare_lanes(void);
int wf__writer_is_done(const void *frame);
uint64_t wf__writer_resume_migrations(void);
uint64_t wf__writer_resume_count(void);
void wf__writer_run_root(void *frame);
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

/* Every submit fills the record the caller supplies and answers nothing: the
 * runtime either accepted the operation or executed it itself and published
 * its completion into the record, and either way the operation is the
 * runtime's and will be joined
 * (`research/investigations/io-model/PARK-ON-MISS.md` §7, "Every submit path
 * ends in a published record" -- "never with a 0 the caller must interpret").
 * There is no second lowering left for a verdict to select, so a submit that
 * returned one would be a value no caller could act on.  A `NULL` record, or
 * an argument this ABI cannot mean, is a contract violation and terminates;
 * an argument the host itself would refuse is an ordinary failed outcome and
 * is published as one, which is why a `pread` offset above `INT64_MAX`
 * completes with `EINVAL` rather than terminating. */
void wf__completion_file_read_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *record
);

void wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *record
);

void wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *record
);

void wf__completion_file_open_at_submit(
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

/* A submitted status names the storage its bytes go to, because the record
 * carries a size and never the bytes: the destination is the submitter's and
 * the engine writes it there, so a frame that can hold any operation does not
 * pay 192 bytes for a facility one direct call uses (design §7). */
void wf__completion_file_status_submit(
    int descriptor,
    void *status,
    uint64_t status_capacity,
    void *record
);

void wf__completion_file_close_submit(
    int descriptor,
    void *record
);

/* The selected family's qualified directory-enumeration facility through the
 * same target-progress normalization as typed file operations --
 * `__getdirentries64` on Darwin, `getdents64` on Linux.  EINTR and readiness
 * refusal never cross this ABI as writer-visible errors.  `position` is the
 * base-position cell Darwin's facility requires; Linux keeps the whole cursor
 * in the descriptor and leaves the cell untouched.  The native record the
 * batch holds differs by family and is decoded by the emitted shim, not
 * here. */
void wf__completion_directory_next_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position,
    void *record
);

void wf__completion_file_open_join(
    const void *record,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
);

/* Every join runs the scheduler core's one rule over the record (design §2):
 * read it if it is DONE, park this stack and continue on another if a stack is
 * free or READY, and otherwise wait in place with nothing running above the
 * join, making target progress until the record is DONE or there is nothing
 * left to do but sleep (§2's fourth line, I/O arm; §6 step 4).  A thread that
 * is not on a pool stack has no stack to park and takes that last arm
 * directly. */
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

uint64_t wf__completion_file_submissions(void);
uint64_t wf__completion_file_fallback_submissions(void);
uint64_t wf__completion_file_helper_executions(void);
uint64_t wf__completion_target_helper_count(void);
uint64_t wf__completion_target_helper_executions(void);
/* Records completed: one per submission, whichever engine finished it. */
uint64_t wf__completion_publications(void);
/* Operations the engine executed inside the submitting call itself, because
 * the operation had no kernel completion form, no host work at all, or the
 * adapter measured that submitting it could only add a queue crossing to a
 * host call this thread was about to make (design §7.1, primitive 7).  It is
 * a throughput fact and never an outcome: the record is published either way. */
uint64_t wf__completion_inline_executions(void);
uint64_t wf__completion_linux_io_uring_submissions(void);
/* `io_uring_enter` calls that carried staged submissions to the kernel.  The
 * doorbell is deferred, so this is far below the submission count and the
 * distance between them is what deferring bought. */
uint64_t wf__completion_linux_io_uring_submission_enters(void);

#if defined(__cplusplus)
}
#endif

#endif
