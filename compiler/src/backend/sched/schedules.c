/* The schedules of PARK-ON-MISS.md §10, written against the core's ABI the
 * way `smoke.c` drives it, for the enumerator (`enumerate.c`). Each is one
 * `wf_enum_schedule`: an entry body that ends by posting the status, a reset
 * before every execution, an end check, and the arms it exists to reach. */

#include "enumerate.h"

#include <stddef.h>
#include <string.h>

#define STATUS 7

/* ----------------------------------------------------------- shared shape */

/* Every schedule's own state, in one block the enumerator checkpoints with
 * the rest of the model (it is part of what the next steps depend on). */
static struct {
    unsigned long long io_done;
    unsigned long long compute_done;
    unsigned long long s9_refused;
    unsigned s13_resumed_on;
    int s13_cleanup_ran;
} st;

void *const wf_enum_schedule_state = &st;
const size_t wf_enum_schedule_state_bytes = sizeof st;

static void reset_counts(void) {
    memset(&st, 0, sizeof st);
}

static void one_io(void) {
    wf_sched_record record;
    wf_enum_io(&record);
    st.io_done += 1u;
}

static void read_then_add(void *payload) {
    unsigned long long *cell = payload;
    one_io();
    *cell += 1u;
}

static void add_only(void *payload) {
    unsigned long long *cell = payload;
    *cell += 1u;
}

/* A hand-out of `run`, joined and released here; a refused hand-out runs
 * inline on the owner, as the emitter's refused slot does (§2). */
static void hand_out_join(void (*run)(void *)) {
    unsigned long long *cell = wf_enum_hand_out(run, sizeof *cell);
    if (cell == NULL) {
        unsigned long long inline_cell = 0;
        run(&inline_cell);
        st.compute_done += inline_cell;
        return;
    }
    wf_enum_join(cell);
    st.compute_done += *cell;
    wf_enum_release(cell);
}

/* A hand-out of `run` whose join comes later, for a schedule that wants the
 * entry left on the deque: its cell, zeroed, or NULL when the lane refused
 * and the call already ran inline here (§2). */
static unsigned long long *hand_out_only(void (*run)(void *)) {
    unsigned long long *cell = wf_enum_hand_out(run, sizeof *cell);
    if (cell == NULL) {
        unsigned long long inline_cell = 0;
        run(&inline_cell);
        st.compute_done += inline_cell;
        return NULL;
    }
    return cell;
}

/* The join and release of a cell `hand_out_only` granted. A refusal has run
 * and folded its cell in already, so NULL is nothing to join. */
static void join_release(unsigned long long *cell) {
    if (cell == NULL) {
        return;
    }
    wf_enum_join(cell);
    st.compute_done += *cell;
    wf_enum_release(cell);
}

static const char *check_io(unsigned long long expected) {
    if (st.io_done != expected) {
        return "not every I/O operation completed";
    }
    if (wf_enum_core.status != STATUS) {
        return "the status was not the one posted";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S1 */

/* Independent iterations of the chain, each parking at its first join, and
 * the pool used to the depth this configuration has: the swept
 * configurations park at most two stacks, so three iterations reach every
 * depth and the third is the one that finds no stack (§10 walks four on a
 * pool of five). With two slots on the lane the refused iteration runs
 * inline, and the granted ones are joined newest first. */
#define S1_ITERATIONS 3u

static void s1_main(void *argument) {
    unsigned long long *cells[S1_ITERATIONS];
    unsigned index;
    (void)argument;
    for (index = 0; index < S1_ITERATIONS; index += 1u) {
        cells[index] = hand_out_only(read_then_add);
    }
    for (index = S1_ITERATIONS; index > 0u; index -= 1u) {
        join_release(cells[index - 1u]);
    }
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s1_check(void) {
    if (st.compute_done != S1_ITERATIONS) {
        return "the iterations did not each run once";
    }
    return check_io(S1_ITERATIONS);
}

static const char *s1_covered(const wf_enum_coverage *cov) {
    unsigned long long depth = (unsigned long long)(wf_enum_stacks() - wf_enum_threads());
    if (cov->stats.parks == 0ull) {
        return "no execution parked";
    }
    if (cov->max_parked != depth) {
        return "no execution parked the pool to its depth";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S2 */

/* A publishes J1 and J2 and runs its inline member three frames deep to a
 * read whose join misses. A's stack parks with both entries still on the
 * deque, they run on the new stack, and A resumes to join them done. */
static void s2_read(void) {
    one_io();
}

static void s2_inner(void) {
    s2_read();
}

static void s2_member(void) {
    s2_inner();
}

static void s2_main(void *argument) {
    unsigned long long *first;
    unsigned long long *second;
    (void)argument;
    first = hand_out_only(add_only);
    second = hand_out_only(add_only);
    s2_member();
    join_release(second);
    join_release(first);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s2_check(void) {
    if (st.compute_done != 2u) {
        return "the two hand-outs did not each run once";
    }
    return check_io(1u);
}

static const char *s2_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked at the read three frames down";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S3 */

/* A compute child stolen and unfinished at its join: line three, not a help;
 * the thief's DONE store marks the parent READY. With one thread the child is
 * the newest entry and runs inline (line two). */
static void s3_main(void *argument) {
    (void)argument;
    hand_out_join(add_only);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s3_check(void) {
    if (st.compute_done != 1u) {
        return "the hand-out did not run exactly once";
    }
    return check_io(0u);
}

static const char *s3_covered(const wf_enum_coverage *cov) {
    if (wf_enum_threads() >= 2u && cov->stats.steals == 0ull) {
        return "no execution stole the child";
    }
    if (wf_enum_threads() >= 2u && cov->publish_compute == 0ull) {
        return "no thief published to a parked parent";
    }
    if (cov->stats.inline_runs == 0ull) {
        return "no execution ran the child inline at line two";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S4 */

/* A parked stack resumed on a foreign thread with an unjoined hand-out still
 * on the home deque. The foreign join must not pop it: the home lane or a
 * thief runs it, and that DONE store marks the stack READY. */
static void s4_main(void *argument) {
    unsigned long long *cell;
    (void)argument;
    cell = hand_out_only(add_only);
    one_io();
    join_release(cell);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s4_check(void) {
    if (st.compute_done != 1u) {
        return "the hand-out did not run exactly once";
    }
    return check_io(1u);
}

static const char *s4_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked with the hand-out unjoined";
    }
    if (WF_SCHED_READY_PINNED) {
        if (cov->resume_foreign != 0ull || cov->resume == 0ull) {
            return "pinned resumes were absent or migrated to another worker";
        }
    } else if (wf_enum_threads() >= 2u && cov->resume_foreign == 0ull) {
        return "no execution resumed the stack on a foreign thread";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S5 */

/* A same-group sibling run inline above a join parks on its own I/O: the
 * whole stack parks, and the frame below needs the sibling anyway. */
static void s5_main(void *argument) {
    (void)argument;
    hand_out_join(read_then_add);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s5_check(void) {
    if (st.compute_done != 1u) {
        return "the hand-out did not run exactly once";
    }
    return check_io(1u);
}

static const char *s5_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked inside the sibling";
    }
    /* At two threads the sibling's stack and main's park together, so the
     * pool is used to its depth here (S1 asserts that at one thread). */
    if (wf_enum_threads() >= 2u && cov->max_parked != wf_enum_stacks() - wf_enum_threads()) {
        return "the sibling and its caller did not park to the pool's depth";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S6 */

/* Every thread parked, nothing runnable, operations in flight: every thread
 * sleeps on the one primitive, and one completion wakes at least one thread
 * and never none. S19's two outstanding records carry it once a second thread
 * exists to be the other sleeper. */
static const char *s6_covered(const wf_enum_coverage *cov) {
    if (cov->all_asleep == 0ull) {
        return "no state had every thread asleep with an operation in flight";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S7 */

/* The completion arrives between the begin of the park and its commit: the
 * commit takes NOTIFIED to READY and enqueues the stack exactly once. S20's
 * single I/O carries it once a second thread can drain in that window. */
static const char *s7_covered(const wf_enum_coverage *cov) {
    if (cov->notified == 0ull) {
        return "no completion landed between the begin and the commit";
    }
    if (cov->ready_from_notified == 0ull) {
        return "no commit took NOTIFIED to READY";
    }
    return NULL;
}

/* -------------------------------------------------------------------- S9 */

/* A lane's whole slot set held by outstanding hand-outs: the next hand-out
 * refuses and runs inline, the deque cannot overflow because it is sized by
 * the same constant, and once the joins release the slots a hand-out is
 * granted again. The cap is a resource condition, not a deadlock. */
static void s9_reset(void) {
    reset_counts();
}

static void s9_main(void *argument) {
    unsigned long long *cells[3];
    unsigned long long *last;
    unsigned index;
    (void)argument;
    for (index = 0; index < 3u; index += 1u) {
        cells[index] = hand_out_only(read_then_add);
        if (cells[index] == NULL) {
            st.s9_refused += 1u;
        }
    }
    for (index = 3u; index > 0u; index -= 1u) {
        join_release(cells[index - 1u]);
    }
    last = wf_enum_hand_out(read_then_add, sizeof *last);
    if (last == NULL) {
        wf_enum_fail("a hand-out was refused after the slots were released");
    } else {
        join_release(last);
    }
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s9_check(void) {
    if (st.s9_refused != 1u) {
        return "the lane did not refuse exactly the one hand-out over its cap";
    }
    if (st.compute_done != 4u) {
        return "the four hand-outs did not each run once";
    }
    return check_io(4u);
}

static const char *s9_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked while the slots were held";
    }
    if (cov->stats.inline_runs + st.s9_refused == 0ull) {
        return "no execution ran a member on the owner";
    }
    return NULL;
}

/* ------------------------------------------------------------------ S10a */

/* Pool off: the one thread parks its stack, finds nothing at priorities 1 to
 * 3, and sleeps on the same primitive it sleeps on today, which is an
 * equality with today's behaviour rather than a second mechanism. This is
 * §5's floor at its smallest, the one thread plus one stack. */
static const char *s10a_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "the one thread never parked its stack";
    }
    if (cov->sleeps == 0ull) {
        return "the one thread never slept on the primitive";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S12 */

/* Nested groups: a member of the outer group publishes an inner group of its
 * own. Reverse join order keeps every target newest-or-stolen at each level,
 * and the member's cell carries its own 1 and both of the inner cells. */
static unsigned long long s12_member(unsigned long long **cell) {
    *cell = wf_enum_hand_out(add_only, sizeof **cell);
    if (*cell == NULL) {
        unsigned long long inline_cell = 0;
        add_only(&inline_cell);
        return inline_cell;
    }
    return 0;
}

static void s12_group(void *payload) {
    unsigned long long *cell = payload;
    unsigned long long *first;
    unsigned long long *second;
    unsigned long long total = 1u;
    total += s12_member(&first);
    total += s12_member(&second);
    if (second != NULL) {
        wf_enum_join(second);
        total += *second;
        wf_enum_release(second);
    }
    if (first != NULL) {
        wf_enum_join(first);
        total += *first;
        wf_enum_release(first);
    }
    *cell += total;
}

static void s12_main(void *argument) {
    unsigned long long *cell;
    (void)argument;
    cell = hand_out_only(s12_group);
    join_release(cell);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s12_check(void) {
    if (st.compute_done != 3u) {
        return "the outer member did not carry both inner cells";
    }
    return check_io(0u);
}

static const char *s12_covered(const wf_enum_coverage *cov) {
    if (cov->stats.inline_runs == 0ull) {
        return "no execution ran a member inline at line two";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S13 */

/* An iteration that exits after having parked and resumed: the cleanup on the
 * normal exit edge runs on whatever thread resumed it, which is the thread
 * the exit edge records here. */
static void s13_reset(void) {
    reset_counts();
    st.s13_resumed_on = WF_SCHED_MAX_THREADS;
}

static void s13_main(void *argument) {
    (void)argument;
    one_io();
    st.s13_resumed_on = wf_sched_current_thread(&wf_enum_core)->index;
    st.s13_cleanup_ran = 1;
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s13_check(void) {
    if (!st.s13_cleanup_ran) {
        return "the exit edge never ran";
    }
    if (st.s13_resumed_on >= wf_enum_threads()) {
        return "the exit edge ran on no thread of this configuration";
    }
    return check_io(1u);
}

static const char *s13_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked before the exit edge";
    }
    if (WF_SCHED_READY_PINNED) {
        if (cov->resume_foreign != 0ull || cov->resume == 0ull) {
            return "pinned resumes were absent or migrated to another worker";
        }
    } else if (wf_enum_threads() >= 2u && cov->resume_foreign == 0ull) {
        return "no execution ran the exit edge on a foreign thread";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S14 */

/* One stack parks, resumes, and parks again on a later join. */
static void s14_main(void *argument) {
    (void)argument;
    one_io();
    one_io();
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s14_check(void) {
    return check_io(2u);
}

static const char *s14_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks < 2ull) {
        return "no execution parked twice";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S15 */

/* Two threads reach priority 1 with one READY stack on the list. The core's
 * one mutex serializes the two unlinks: one thread takes the stack, the other
 * finds the list empty and falls to priority 2, and no stack is resumed
 * twice. S20's single park is the one READY stack. */
static const char *s15_covered(const wf_enum_coverage *cov) {
    if (cov->resume == 0ull) {
        return "no execution took a READY stack off the list";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S16 */

/* A stack parks with its own entries on the deque, thieves take some, it
 * resumes elsewhere, and its joins find each target stolen or done. */
static void s16_main(void *argument) {
    unsigned long long *first;
    unsigned long long *second;
    (void)argument;
    first = hand_out_only(add_only);
    second = hand_out_only(add_only);
    one_io();
    join_release(second);
    join_release(first);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s16_check(void) {
    if (st.compute_done != 2u) {
        return "the two hand-outs did not each run once";
    }
    return check_io(1u);
}

static const char *s16_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked with its entries on the deque";
    }
    if (wf_enum_threads() >= 2u && cov->stats.steals == 0ull) {
        return "no thief took an entry off the parked stack's lane";
    }
    if (WF_SCHED_READY_PINNED) {
        if (cov->resume_foreign != 0ull || cov->resume == 0ull) {
            return "pinned resumes were absent or migrated to another worker";
        }
    } else if (wf_enum_threads() >= 2u && cov->resume_foreign == 0ull) {
        return "no execution resumed the stack on a foreign thread";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S17 */

/* The entry stack parks on main's first I/O, a worker resumes it and runs
 * main to its return, and the entry thread is meanwhile in step 4. The post
 * is walked on both sides of that thread's epoch capture, which is the whole
 * of the answer: either the sleeper is woken or the capture sees the post. */
static const char *s17_covered(const wf_enum_coverage *cov) {
    /* Under pinning the entry continuation cannot post from another thread
     * or while its own thread sleeps. Keep all original coverage below for
     * the migrating policy, and assert the opposite affinity invariant here. */
    if (WF_SCHED_READY_PINNED) {
        if (cov->resume_foreign != 0ull || cov->post_by_worker != 0ull
            || cov->post_in_window != 0ull || cov->post_while_asleep != 0ull) {
            return "the pinned entry continuation escaped its running owner";
        }
        if (cov->stats.parks == 0ull || cov->resume == 0ull || cov->post_elsewhere == 0ull) {
            return "no pinned entry parked, resumed and posted its exit";
        }
        return NULL;
    }
    if (cov->post_by_worker == 0ull) {
        return "no execution posted the status from a worker";
    }
    if (cov->post_in_window == 0ull) {
        return "no post landed inside the capture-to-park window";
    }
    if (cov->post_while_asleep == 0ull) {
        return "no post landed with the entry thread asleep";
    }
    if (cov->post_elsewhere == 0ull) {
        return "no post landed outside the entry thread's step 4";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S18 */

/* A group of three reads holds three records in one frame, all outstanding
 * together and each owning its own block, joined where the emitter puts each
 * join. What it walks is that every record's address stays valid to its
 * join. */
static void s18_main(void *argument) {
    wf_sched_record first;
    wf_sched_record second;
    wf_sched_record third;
    (void)argument;
    wf_sched_record_init(&first);
    wf_sched_record_init(&second);
    wf_sched_record_init(&third);
    wf_enum_submit(&first);
    wf_enum_submit(&second);
    wf_enum_submit(&third);
    wf_enum_join_io(&first);
    wf_enum_join_io(&second);
    wf_enum_join_io(&third);
    st.io_done += 3u;
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s18_check(void) {
    return check_io(3u);
}

static const char *s18_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S19 */

/* Two records outstanding in one frame (S18), joined newest first, so their
 * completions arrive in either order and each resumes independently. */
static void s19_main(void *argument) {
    wf_sched_record first;
    wf_sched_record second;
    (void)argument;
    wf_sched_record_init(&first);
    wf_sched_record_init(&second);
    wf_enum_submit(&first);
    wf_enum_submit(&second);
    wf_enum_join_io(&second);
    wf_enum_join_io(&first);
    st.io_done += 2u;
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s19_check(void) {
    return check_io(2u);
}

static const char *s19_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S20 */

/* A may-suspend member that need not miss: one I/O whose completion may be
 * there at the join. The fast path is a take with zero switches. */
static void s20_main(void *argument) {
    (void)argument;
    one_io();
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s20_check(void) {
    return check_io(1u);
}

static const char *s20_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0ull) {
        return "no execution parked";
    }
    if (wf_enum_threads() < 2u) {
        /* One thread drains only in its own idle step, so a completion that
         * arrived before the join still parks: line one and the cancel arms
         * need a second thread's drain. */
        return NULL;
    }
    if (cov->stats.line_one == 0ull) {
        return "no join found its record DONE at line one";
    }
    if (cov->cancel_suspending_io == 0ull) {
        return "no cancel from SUSPENDING (item 6)";
    }
    /* Item 7's arm, a cancel consuming NOTIFIED, is gone: a publisher
     * notifies only a registration it has claimed, and a parker cancels only
     * a registration it took back, so the two never meet on one park. The
     * enumerator fails the execution if the transition ever happens. */
    if (cov->cancel_notified_io != 0ull) {
        return "a cancel consumed NOTIFIED, which the claim protocol forbids";
    }
    if (cov->ready_from_notified == 0ull) {
        return "no commit of NOTIFIED to READY (item 5, S7)";
    }
    if (cov->ready_from_suspended == 0ull) {
        return "no SUSPENDED to READY (item 3)";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S21 */

/* The event lands before the registration, which S7 does not walk: the thief
 * completes and finds no waiter, so it publishes nothing and the parking
 * stack takes its registration back and cancels from SUSPENDING; or the
 * thief claims the registration first, and the parking stack switches away
 * and is resumed through NOTIFIED and the commit. The slot is then released
 * and acquired again, with nothing stale in `waiter`. */
static void s21_main(void *argument) {
    (void)argument;
    hand_out_join(add_only);
    hand_out_join(add_only);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s21_check(void) {
    if (st.compute_done != 2u) {
        return "the two hand-outs did not each run once";
    }
    return check_io(0u);
}

static const char *s21_covered(const wf_enum_coverage *cov) {
    if (cov->cancel_suspending_compute == 0ull) {
        return "no cancel from SUSPENDING on a compute record";
    }
    if (cov->cancel_notified_compute != 0ull) {
        return "a cancel consumed NOTIFIED, which the claim protocol forbids";
    }
    if (cov->ready_from_notified == 0ull) {
        return "no thief claimed a registration inside the parking window";
    }
    if (cov->publish_compute == 0ull) {
        return "no thief published to a parked parent";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S22 */

/* The ingredients of the crossed pair on the exhausted path: a member with a
 * group of its own, which misses on its own read above that group's join,
 * while the frame below misses on a read of its own with the pool spent. The
 * resolution comes from a steal and never from an owner-side pop (I2). */
static void child_with_grandchild(void *payload) {
    unsigned long long *cell = payload;
    hand_out_join(read_then_add);
    one_io();
    *cell += 1u;
}

static void s22_main(void *argument) {
    unsigned long long *cell;
    (void)argument;
    cell = hand_out_only(child_with_grandchild);
    one_io();
    join_release(cell);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s22_check(void) {
    if (st.compute_done != 2u) {
        return "the member and its own member did not each run once";
    }
    return check_io(3u);
}

static const char *s22_covered(const wf_enum_coverage *cov) {
    if (cov->stats.steals == 0ull) {
        return "no execution stole the member";
    }
    if (cov->stats.exhausted_compute_waits == 0ull) {
        return "no execution reached the fourth line's compute arm";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S23 */

/* A nested entry on the compute arm that itself misses on I/O with no stack,
 * and group C item 11's two arms at §5's floor, walked by one body: main
 * hands out J1 (compute) and J2 (a read), reads itself and parks; the loop
 * stack pops J2, whose read misses with no stack and takes the I/O arm with
 * nothing above it (I1); main's read completes, main resumes on the late
 * third line and joins J2 against a newest end that is J1 with no stack to
 * be had, which is the compute arm, reached through the thread's own deque
 * and then a steal (I2). Two iterations of S1's chain, at the floor
 * configurations, where the arms live; the depth at (2,4) is S5's. */
static void s23_main(void *argument) {
    unsigned long long *first;
    unsigned long long *second;
    (void)argument;
    first = hand_out_only(add_only);
    second = hand_out_only(read_then_add);
    one_io();
    join_release(second);
    join_release(first);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s23_check(void) {
    if (st.compute_done != 2u) {
        return "the two hand-outs did not each run once";
    }
    return check_io(2u);
}

static const char *s23_covered(const wf_enum_coverage *cov) {
    int at_floor = wf_enum_stacks() == wf_enum_threads() + 1u;
    if (cov->stats.parks == 0ull) {
        return "no execution parked";
    }
    if (cov->max_parked != wf_enum_stacks() - wf_enum_threads()) {
        return "the chain did not park to the pool's depth";
    }
    if (at_floor && cov->stats.exhausted_io_waits == 0ull) {
        return "no execution reached the fourth line's I/O arm at the floor";
    }
    if (at_floor && cov->stats.exhausted_compute_waits == 0ull) {
        return "no execution reached the fourth line's compute arm at the floor";
    }
    /* A stack turns READY inside the I/O arm only through another thread:
     * with one thread the drain that makes it READY is this thread's own
     * progress, which restarts the rule before the arm's re-check. */
    if (at_floor && wf_enum_threads() >= 2u && cov->stats.late_parks == 0ull) {
        return "no execution found a READY stack inside the I/O arm";
    }
    return NULL;
}

/* ------------------------------------------------------------------- S24 */

/* The child can be parked on its own I/O when the parent resumes. The
 * parent's checkpoint drains that completion and yields to the child, without
 * making the parent's live stack available before its switch has committed. */
static void s24_main(void *argument) {
    unsigned long long *child;
    (void)argument;
    child = hand_out_only(read_then_add);
    one_io();
    wf_sched_checkpoint(&wf_enum_core);
    join_release(child);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s24_check(void) {
    if (st.compute_done != 1u) {
        return "the yielded-to child did not complete exactly once";
    }
    return check_io(2u);
}

static const char *s24_covered(const wf_enum_coverage *cov) {
    if (cov->cooperative == 0ull) {
        return "no checkpoint yielded to a READY stack";
    }
    return NULL;
}

/* Two initial I/O calls exercise owner queues, FIFO links, wake publication,
 * and the no-free-stack compute join. Normal policies keep the same calls on
 * their ordinary deques; round-robin placement has an exact owner count. */
static void s25_main(void *argument) {
    unsigned long long *first;
    unsigned long long *second;
    (void)argument;
    first = wf_enum_hand_out_io(read_then_add, sizeof *first);
    second = wf_enum_hand_out_io(read_then_add, sizeof *second);
    if (first == NULL || second == NULL) {
        wf_enum_fail("two available initial I/O slots were refused");
    }
    join_release(second);
    join_release(first);
    wf_sched_post_status(&wf_enum_core, STATUS);
}

static const char *s25_check(void) {
    if (st.compute_done != 2u) return "an initial I/O call did not return exactly once";
#if WF_SCHED_IO_ROUND_ROBIN
    for (unsigned index = 0; index < wf_enum_core.thread_count; ++index) {
        if (wf_enum_core.threads[index].io_started != 2u / wf_enum_core.thread_count) {
            return "initial I/O calls did not run on their assigned workers";
        }
    }
#endif
    return check_io(2u);
}

static const char *s25_covered(const wf_enum_coverage *cov) {
    if (cov->stats.parks == 0u || cov->resume == 0u) {
        return "no initial I/O task parked and resumed";
    }
    return NULL;
}

#if WF_SCHED_IO_ROUND_ROBIN
/* The startup seam publishes a contiguous prefix after thread creation. An
 * idle configured worker outside that prefix must receive no initial task. */
static void s26_main(void *argument) {
    wf_enum_core.io_dispatch_threads = 1u;
    s25_main(argument);
}

static const char *s26_check(void) {
    if (st.compute_done != 2u) return "a prefix-assigned I/O call was lost";
    for (unsigned index = 0; index < wf_enum_core.thread_count; ++index) {
        unsigned expected = index == 0u ? 2u : 0u;
        if (wf_enum_core.threads[index].io_started != expected) {
            return "an initial I/O task was assigned outside the available prefix";
        }
    }
    return check_io(2u);
}
#endif

/* ------------------------------------------------------------- the table */

/* Three of §10's schedules are not enumerable here and are absent for that
 * reason: S8 is the floor's overflow record, written by the fault handler
 * from a guard hit rather than by any step of this protocol; S10b is Windows,
 * which §10 answers by saying it takes the same one path; and S11 is retired
 * until §5 introduces stack classes. */

const wf_enum_schedule wf_enum_schedules[] = {
    {"S1", 1u, 1u, 2u, 0u, reset_counts, s1_main, s1_check, s1_covered},
    {"S2", 1u, 0u, 2u, 0u, reset_counts, s2_main, s2_check, s2_covered},
    {"S3", 1u, 0u, 2u, 0u, reset_counts, s3_main, s3_check, s3_covered},
    {"S4", 1u, 0u, 2u, 0u, reset_counts, s4_main, s4_check, s4_covered},
    {"S5", 1u, 0u, 2u, 0u, reset_counts, s5_main, s5_check, s5_covered},
    {"S6", 2u, 0u, 2u, 0u, reset_counts, s19_main, s19_check, s6_covered},
    {"S7", 2u, 0u, 2u, 0u, reset_counts, s20_main, s20_check, s7_covered},
    {"S9", 1u, 1u, 2u, 0u, s9_reset, s9_main, s9_check, s9_covered},
    {"S10a", 1u, 1u, 2u, 0u, reset_counts, s20_main, s20_check, s10a_covered},
    {"S12", 1u, 0u, 2u, 0u, reset_counts, s12_main, s12_check, s12_covered},
    {"S13", 1u, 0u, 2u, 0u, s13_reset, s13_main, s13_check, s13_covered},
    {"S14", 1u, 0u, 2u, 0u, reset_counts, s14_main, s14_check, s14_covered},
    {"S15", 2u, 0u, 2u, 0u, reset_counts, s20_main, s20_check, s15_covered},
    {"S16", 1u, 0u, 2u, 0u, reset_counts, s16_main, s16_check, s16_covered},
    {"S17", 2u, 0u, 2u, 0u, reset_counts, s20_main, s20_check, s17_covered},
    {"S18", 1u, 0u, 2u, 0u, reset_counts, s18_main, s18_check, s18_covered},
    {"S19", 1u, 0u, 2u, 0u, reset_counts, s19_main, s19_check, s19_covered},
    {"S20", 1u, 0u, 2u, 0u, reset_counts, s20_main, s20_check, s20_covered},
    {"S21", 2u, 0u, 2u, 0u, reset_counts, s21_main, s21_check, s21_covered},
    {"S22", 2u, 0u, 2u, 0u, reset_counts, s22_main, s22_check, s22_covered},
    {"S23", 1u, 0u, 2u, 3u, reset_counts, s23_main, s23_check, s23_covered},
    {"S24", 1u, 0u, 2u, 0u, reset_counts, s24_main, s24_check, s24_covered},
    {"S25", 1u, 0u, 1u, 0u, reset_counts, s25_main, s25_check, s25_covered},
#if WF_SCHED_IO_ROUND_ROBIN
    {"S26", 1u, 0u, 1u, 0u, reset_counts, s26_main, s26_check, s25_covered},
#endif
};

const unsigned wf_enum_schedule_count = sizeof wf_enum_schedules / sizeof wf_enum_schedules[0];
