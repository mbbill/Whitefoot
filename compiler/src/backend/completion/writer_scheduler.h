#ifndef WHITEFOOT_WRITER_SCHEDULER_H
#define WHITEFOOT_WRITER_SCHEDULER_H

#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

typedef void (*wf__writer_resume_fn)(void *frame);

#define WF_WRITER_HEADER_STORAGE_BYTES 64u
#define WF_WRITER_HEADER_STORAGE_ALIGNMENT 8u

void wf__writer_frame_init(void *frame);
void wf__writer_begin_suspend(void *frame, wf__writer_resume_fn resume);
int wf__writer_commit_suspend(void *frame);
void wf__writer_cancel_suspend(void *frame);
void wf__writer_complete(void *frame);

/* Completion draining calls only `ready`: it changes scheduler state and
 * enqueues an opaque frame, but never reads or invokes its resume entry. */
void wf__writer_scheduler_ready(void *frame);

/* Called only by a normal scheduler lane (a par worker/help path or the
 * completion-only root pump). This is the sole ABI which invokes resume. */
int wf__writer_scheduler_help_once(void);
void wf__writer_scheduler_prepare_lanes(void);

int wf__writer_is_done(const void *frame);
uint64_t wf__writer_resume_migrations(void);
uint64_t wf__writer_resume_count(void);

#if defined(__cplusplus)
}
#endif

#endif
