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
 */

#include <stdio.h>

extern unsigned long wf__par_grants(void);

__attribute__((destructor)) static void wf__par_report(void) {
    (void)fprintf(stderr, "grants=%lu\n", wf__par_grants());
}
