/* The core's counters, read at exit, for the measurements this experiment
 * makes over whole compiled programs.
 *
 * Design section 12's fifth item asks for "the smallest stack count at which
 * the pool stops refusing for the corpus programs, and what a refusal costs
 * below it". A refusal is the fourth line of the rule (section 2) taken
 * because the pool had no stack, and the core already counts it, per thread,
 * as `exhausted_io_waits` and `exhausted_compute_waits`. Nothing a program can
 * write reads those counters, and nothing the driver stages carries this file,
 * so no shipped program has it: it is linked beside an emitted module by
 * `run.sh` and by nothing else, exactly as `sched/grant_observer.c` is linked
 * by the tests that read the grant count.
 *
 * It registers its report through `atexit` from a constructor, for the reason
 * `grant_observer.c` gives at length: on the MSVC target a destructor
 * attribute is a C-runtime terminator that runs after the streams are torn
 * down. This file is POSIX-only in practice, and follows the same form anyway
 * because there is no reason for two spellings of one thing.
 *
 * The read is the same benign race `grant_observer.c` documents: the counters
 * are per thread, they are static storage that outlives every thread, and a
 * worker still running while they are summed can only make a total smaller
 * than the run's true one -- which cannot turn a pool that refused into one
 * that did not.
 *
 * The line goes to the diagnostic channel, so a program's own bytes are
 * untouched and the benchmark runner's checksum comparison still holds. */

#include <stdio.h>
#include <stdlib.h>

#include "core.h"
#include "entry.h"

static void wf__sched_statistics_report(void) {
    wf_sched_statistics counts;
    wf_sched_statistics_sum(&wf__sched_core, &counts);
    (void)fprintf(
        stderr,
        "sched-statistics parks=%llu resumes=%llu cancels=%llu steals=%llu "
        "inline=%llu late_parks=%llu line_one=%llu exhausted_io=%llu "
        "exhausted_compute=%llu\n",
        counts.parks,
        counts.resumes,
        counts.cancels,
        counts.steals,
        counts.inline_runs,
        counts.late_parks,
        counts.line_one,
        counts.exhausted_io_waits,
        counts.exhausted_compute_waits
    );
}

__attribute__((constructor)) static void wf__sched_statistics_register(void) {
    (void)atexit(wf__sched_statistics_report);
}
