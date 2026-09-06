/* The test-only observer of the core's grant count.
 *
 * `wf__par_grants` is the core's steal count: hand-outs that ran on a thread
 * other than the one that offered them. No Whitefoot construct can name it and
 * no program reads it; it exists exactly so that a pool which never grants a
 * lane cannot pass for one that does (`sched/entry.h`).
 *
 * This unit is linked only by a build that wants to read that count: the POSIX
 * `backend/tests/parallel.rs` cases, which include these bytes and link them
 * beside an emitted module, and the `completion-windows` job's "Require native
 * workers for a real --par program" step, which links the same bytes on the
 * other platform. Nothing the driver stages carries it, so no shipped program
 * has it.
 *
 * One file rather than two spellings of it, for the same reason the runtime it
 * observes is one implementation: two platforms proving one property should be
 * proving it with one thing.
 *
 * Where this runs, and why it registers rather than declaring itself a
 * destructor
 * ---------------------------------------------------------------------------
 * The report used to carry `__attribute__((destructor))`, and on the MSVC
 * target that is not the same thing it is on ELF. Clang lowers it there into a
 * C-runtime *terminator* -- an entry in the `.CRT$XT` table that `_initterm`
 * walks from inside `common_exit` -- and that table is walked after the UCRT
 * has released its stream locks. A `fputs` from there enters a critical
 * section that no longer exists: the `completion-windows` job's observed
 * `--par` executable died in `RtlpWaitOnCriticalSection` writing to address
 * 0x24, under `fputs` under `wf__par_report` under `_initterm` under
 * `common_exit`, with the program's own bytes already on standard output and
 * the three workers parked exactly where they belong. On ELF the same
 * attribute is a `.fini_array` entry that runs while stdio is still alive,
 * which is why POSIX never showed it, and no amount of care in the write saves
 * a write made after the streams are gone.
 *
 * So the report is registered rather than declared. `constructor` lowers to a
 * `.CRT$XCU` initializer on MSVC and to `.init_array` on ELF, both of which
 * run before `main`; `atexit` handlers run first in the exit sequence on both
 * platforms, before any terminator and with the streams intact. The LIFO order
 * against the bridge's own `atexit(wf_bridge_shutdown)` matters for a run with
 * no additional workers: that handler tears down the ring before this report.
 * The bridge therefore retains reportability of its static atomic counters
 * after teardown. With workers still running it leaves the engine intact.
 * Both cases find live stdio and counters whose storage outlives teardown.
 *
 * The core's counter writes and this snapshot's reads are relaxed atomic.
 * Each counter has one writer, and its static storage outlives every thread.
 * The snapshot is not simultaneous across workers: a worker still granting
 * lanes while this sums them can make the total smaller than the run's final
 * one, but cannot turn a pool that granted nothing into one that did.
 */

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

extern unsigned long wf__par_grants(void);
extern int wf__sched_report(char *buffer, size_t capacity);
extern int wf__bridge_report(char *buffer, size_t capacity);

/* The grant line is the one line every judge of this observer reads, and it
 * stays one line. The core's own counters follow it only when the run asked
 * for them with `WF_SCHED_REPORT`, so a case that fails on the grant count can
 * show what the threads did instead of the count alone. */
static void wf__par_report(void) {
    char counters[1024];
    (void)fprintf(stderr, "grants=%lu\n", wf__par_grants());
    if (wf__sched_report(counters, sizeof(counters))) {
        (void)fprintf(stderr, "%s\n", counters);
        if (wf__bridge_report(counters, sizeof(counters))) {
            (void)fprintf(stderr, "%s\n", counters);
        }
    }
}

__attribute__((constructor)) static void wf__par_register_report(void) {
    (void)atexit(wf__par_report);
}
