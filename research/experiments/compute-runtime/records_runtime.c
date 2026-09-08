#include "records_scheduler.h"
#include "runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    RecordChunk chunk;
    void *context;
    size_t first, end;
} Frame;

static unsigned configured;
static void run_range(Frame frame);
static void run_frame(void *opaque) {
    Frame frame;
    memcpy(&frame, opaque, sizeof(frame));
    run_range(frame);
}

static void run_range(Frame frame) {
    size_t count = frame.end - frame.first;
    if (count <= 1) {
        if (count) frame.chunk(frame.context, frame.first);
        return;
    }
    void *task = wf__par_acquire_lane(sizeof(frame));
    if (!task) {
        for (size_t i = frame.first; i < frame.end; ++i) frame.chunk(frame.context, i);
        return;
    }
    size_t middle = frame.first + count / 2;
    Frame left = {frame.chunk, frame.context, frame.first, middle};
    memcpy(task, &left, sizeof(left));
    wf__par_publish(task, run_frame);
    frame.first = middle;
    run_range(frame);
    wf__par_join(task);
    wf__par_release(task);
}

void records_scheduler_run(unsigned width, size_t chunks, RecordChunk chunk, void *context) {
    if ((width != 1 && width != 2 && width != 4) || !chunk || (configured && configured != width)) abort();
    if (!configured) {
        /* Configure before the runtime's first cached width read. All later
         * invocations retain this budget; no process-wide nested pool exists. */
        const char *expected = width == 1 ? "1" : width == 2 ? "2" : "4";
        const char *requested = getenv("WF_WORKERS");
        if (!requested || strcmp(requested, expected)) {
            fputs("WF scheduler control requires matching WF_WORKERS\n", stderr);
            abort();
        }
        configured = width;
    }
    if (width == 1) {
        for (size_t i = 0; i < chunks; ++i) chunk(context, i);
    } else {
        run_range((Frame){chunk, context, 0, chunks});
        unsigned actual = wf_compute_worker_count();
        if (actual && actual != width) abort();
    }
}

const char *records_scheduler_name(void) { return "wf-runtime"; }
int records_scheduler_stop(void) { return 0; }
