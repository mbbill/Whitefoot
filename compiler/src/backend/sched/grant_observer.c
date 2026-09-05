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
 * Where this runs, and what that costs it
 * ---------------------------------------
 * A destructor runs from `exit`, and by then this process is in an unusual
 * state that no shipped program is ever in: the entry thread has returned its
 * status, and the scheduler pool -- three detached threads on this job's
 * `WF_WORKERS=4` -- is still in its loop, because a detached worker has no stop
 * protocol and the runtime deliberately leaves it to the kernel
 * (`completion/bridge.c`, `wf_bridge_shutdown`). So this line is written while
 * other threads run, and it is the only thing in any Windows link of this
 * project that calls the C runtime from an exit handler.
 *
 * That is why the count is taken first and the number is spelled here rather
 * than by `fprintf`: what reaches the C runtime at exit is one `fputs` of a
 * finished string, with no format parsing, no varargs and no conversion state
 * behind it. It does not make the exit path safe by itself -- one `fputs` is
 * still stdio -- and it is not offered as a diagnosis of anything; it is the
 * smallest surface this observation can be made through, which is what a
 * test-only unit that runs in a state nothing else runs in should ask for.
 *
 * The read itself is a benign race by construction: the core keeps its
 * counters per thread so that no two threads ever write one word, the counters
 * are static storage that outlives every thread, and a worker still granting
 * lanes while this sums them can only make the total it reports smaller than
 * the run's true one -- which cannot turn a pool that granted nothing into one
 * that did.
 */

#include <stddef.h>
#include <stdio.h>

extern unsigned long wf__par_grants(void);

/* "grants=" plus the widest 64-bit decimal, a newline and a terminator. */
#define WF_PAR_REPORT_BYTES 40u

__attribute__((destructor)) static void wf__par_report(void) {
    unsigned long granted = wf__par_grants();
    char line[WF_PAR_REPORT_BYTES];
    char digits[24];
    size_t digit_count = 0;
    size_t length = 0;
    const char prefix[] = "grants=";
    size_t index;

    for (index = 0; index + 1u < sizeof(prefix); index += 1u) {
        line[length] = prefix[index];
        length += 1u;
    }
    do {
        digits[digit_count] = (char)('0' + (int)(granted % 10ul));
        digit_count += 1u;
        granted /= 10ul;
    } while (granted != 0ul && digit_count < sizeof(digits));
    while (digit_count != 0) {
        digit_count -= 1u;
        line[length] = digits[digit_count];
        length += 1u;
    }
    line[length] = '\n';
    length += 1u;
    line[length] = '\0';
    (void)fputs(line, stderr);
}
