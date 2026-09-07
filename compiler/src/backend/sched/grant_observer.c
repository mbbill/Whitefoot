/* The optional observer of the core's steal count.
 *
 * `wf__par_grants` is the core's steal count: hand-outs that ran on a thread
 * other than the one that offered them. This is not an acquisition count:
 * an owner can acquire, publish and execute every task itself with zero steals.
 * The legacy report name stays stable for its measurement and gate readers.
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
 * one, but cannot report a steal when none occurred.
 */

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

extern unsigned long wf__par_grants(void);
extern int wf__sched_report(char *buffer, size_t capacity);
extern int wf__bridge_report(char *buffer, size_t capacity);

#if defined(WF_PAR_TEST_HANDOFF)
/* Only CountedProgram enables this POSIX test interposer. Emitted calls go
 * through these entries; the core and its weak/strong definitions are intact.
 * A requested witness holds the first publisher until another OS thread enters
 * its actual thunk. That thread then executes the original function once. A
 * schedulable-core count or repeated short runs cannot establish this event.
 * The deadline diagnoses a broken test/runtime; it never selects acceptance. */
#include <errno.h>
#include <pthread.h>
#include <time.h>

extern void *wf__par_acquire_lane(unsigned long bytes);
extern void wf__par_publish(void *frame, void (*run)(void *));
extern unsigned wf__sched_pool_running(void);

static unsigned long wf_test_acquired;
static unsigned long wf_test_published;
static unsigned wf_test_claimed;
static unsigned wf_test_entered;
static pthread_t wf_test_owner;
static void *wf_test_frame;
static void (*wf_test_run)(void *);
static pthread_mutex_t wf_test_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf_test_changed = PTHREAD_COND_INITIALIZER;

static void wf_test_fail(const char *message) {
    (void)fprintf(stderr, "handoff witness: %s\n", message);
    _Exit(97);
}

void *wf_test_acquire_lane(unsigned long bytes) {
    void *frame = wf__par_acquire_lane(bytes);
    if (frame != NULL) __atomic_fetch_add(&wf_test_acquired, 1ul, __ATOMIC_RELAXED);
    return frame;
}

static void wf_test_enter(void *frame) {
    if (pthread_mutex_lock(&wf_test_lock)) wf_test_fail("lock failed");
    if (frame != wf_test_frame || pthread_equal(wf_test_owner, pthread_self())) {
        wf_test_fail("the published task did not enter on another thread");
    }
    void (*run)(void *) = wf_test_run;
    wf_test_entered = 1;
    if (pthread_cond_broadcast(&wf_test_changed) || pthread_mutex_unlock(&wf_test_lock)) {
        wf_test_fail("worker notification failed");
    }
    run(frame);
}

void wf_test_publish(void *frame, void (*run)(void *)) {
    __atomic_fetch_add(&wf_test_published, 1ul, __ATOMIC_RELAXED);
    if (!getenv("WF_PAR_TEST_REQUIRE_WORKER") ||
        __atomic_exchange_n(&wf_test_claimed, 1u, __ATOMIC_RELAXED)) {
        wf__par_publish(frame, run);
        return;
    }
    if (!wf__sched_pool_running()) wf_test_fail("no CPU worker started");
    if (pthread_mutex_lock(&wf_test_lock)) wf_test_fail("lock failed");
    wf_test_owner = pthread_self();
    wf_test_frame = frame;
    wf_test_run = run;
    if (pthread_mutex_unlock(&wf_test_lock)) wf_test_fail("unlock failed");
    wf__par_publish(frame, wf_test_enter);
    if (pthread_mutex_lock(&wf_test_lock)) wf_test_fail("lock failed");
    struct timespec deadline;
    if (clock_gettime(CLOCK_REALTIME, &deadline)) wf_test_fail("clock failed");
    deadline.tv_sec += 30;
    while (!wf_test_entered) {
        int status = pthread_cond_timedwait(&wf_test_changed, &wf_test_lock, &deadline);
        if (status == ETIMEDOUT) wf_test_fail("worker did not enter the published task");
        if (status) wf_test_fail("worker wait failed");
    }
    if (pthread_mutex_unlock(&wf_test_lock)) wf_test_fail("unlock failed");
}
#endif

/* The grant line is the one line every judge of this observer reads, and it
 * stays one line. The core's own counters follow it only when the run asked
 * for them with `WF_SCHED_REPORT`, so a case that fails on the grant count can
 * show what the threads did instead of the count alone. */
static void wf__par_report(void) {
    char counters[1024];
#if defined(WF_PAR_TEST_HANDOFF)
    (void)fprintf(stderr, "handouts: acquired=%lu published=%lu worker_entered=%u\n",
                  __atomic_load_n(&wf_test_acquired, __ATOMIC_RELAXED),
                  __atomic_load_n(&wf_test_published, __ATOMIC_RELAXED),
                  wf_test_entered);
#endif
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
