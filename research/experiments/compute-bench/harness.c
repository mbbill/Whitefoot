/* Serves compute-bench: the one driver linked into every kernel image. It
 * owns the process protocol, the width rule, the clock, the timed loop, the
 * TSV the reducer reads, the backend lookup table, and the `main` that routes
 * through the Whitefoot floor. Nothing here knows what any kernel computes.
 *
 * The timed interval is section 4.5's and is identical for every
 * implementation of a kernel: the clock encloses exactly one `call()` and
 * nothing else. Verification, release, fixture construction and every printf
 * are outside it. No elapsed time reaches a pass/fail decision anywhere in
 * this file: there is no threshold, no alarm, no timeout and no budget. */
/* unsetenv, sysconf and CLOCK_MONOTONIC_RAW are POSIX and Darwin extensions
   that -std=c11 hides unless the feature test macro is set before any
   system header is read. */
#if defined(__APPLE__)
#define _DARWIN_C_SOURCE 1
#else
#define _POSIX_C_SOURCE 200809L
#endif

#include "parlay_status.h"

#include "harness.h"
#include "backend.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

extern int wf__floor_run(int, char **);

/* ---------------------------------------------------------------- clock -- */

#if defined(CLOCK_MONOTONIC_RAW)
#define WFB_CLOCK CLOCK_MONOTONIC_RAW
#define WFB_CLOCK_NAME "CLOCK_MONOTONIC_RAW"
#else
#define WFB_CLOCK CLOCK_MONOTONIC
#define WFB_CLOCK_NAME "CLOCK_MONOTONIC"
#endif

uint64_t wfb_now_ns(void) {
    struct timespec t;
    if (clock_gettime(WFB_CLOCK, &t) != 0) wfb_fail("clock_gettime failed");
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}

/* Process CPU time: every thread of this process, user plus system. The source
   is selected per host, the way the wall clock above is, because no single
   call answers this question everywhere.

   CLOCK_PROCESS_CPUTIME_ID is the POSIX answer and is what Linux gives. It is
   NOT the answer on Darwin: there the clock is served from the task's basic
   info, which carries the time of THREADS THAT HAVE ALREADY EXITED, so a pool
   whose workers are still alive at the read contributes nothing and the column
   read about one lane's worth however many lanes ran. That is a measured
   defect and not a guess -- the M1 Pro tables recorded `tbb` at records W=8
   spending 5,896 us of CPU for a 2,441 us wall on eight threads, `static` at
   quadrature W=4 spending 2,904 us against a 2,866 us wall, and a Whitefoot
   row at quadrature W=4 spending 3,081 us against a 3,033 us wall while
   stealing a thousand chunks, where the same rows on Linux read about wall
   times threads.

   What replaces it on Darwin is the task-level pair, which is the only reading
   here that consults the LIVE threads when it is asked:
   TASK_THREAD_TIMES_INFO sums the user and system time of the threads that
   still exist, and TASK_BASIC_INFO carries the same totals for the ones that
   have exited; a thread's time therefore moves from the first to the second
   when it exits and is counted exactly once either way. Both halves are
   `time_value_t`, seconds plus microseconds, and are converted here. That pair
   is held on a READING and not on its documentation, which is what the rejected
   attempt below cost: the hosted `macos-14` leg of run 34668036736, a
   three-CPU runner, printed `cpu_clock=task_info` on every driver line and
   returned CPU that grows with the lanes and stops where the CPUs do --
   mandelbrot `static` at W=4 read 107,878 us of CPU against a 36,495 us wall,
   2.96 times it, `tbb` at W=4 26,072 against 8,887, 2.93 times it, the W=2
   rows about twice their wall, and the four W=1 serial rows inside half a
   percent of their own wall. A refusal would have been visible rather than
   silent, since the name printed is the source that answered.

   `proc_pid_rusage(RUSAGE_INFO_V0)` was tried first and REJECTED ON EVIDENCE.
   Its `ri_user_time + ri_system_time` are documented as nanoseconds over the
   whole process, but on the hosted `macos-14` runner of run 34667394566 every
   row read 0.02 to 0.07 times its own wall and barely moved with the work:
   mandelbrot `wf` at W=2 read 540 to 570 us of CPU for walls from 11.7 to
   36.2 ms, and `static` at W=4 read 2,363 us against a 35,089 us wall. That is
   not a unit error -- a 24 MHz timebase tick would have put `static` near 1.6
   times its wall -- and a figure that does not scale with the work is not a
   CPU figure. Do not go back to it without a reading that tracks the work.

   getrusage(RUSAGE_SELF) is the fallback for a host that has neither, and for
   a host whose primary source refuses at run time. It is coarser
   (microseconds), which is why it is nobody's first choice, and what it counts
   on Darwin has not been read here, so a fallback taken on that host is a
   figure of unknown standing. That is exactly why the name below reports the
   source that answered rather than the one this file prefers.

   The source is fixed by the first reading of the run and never changes after
   it, so a `before` and an `after` bracketing one call can never come from two
   different sources. wfb_cpu_clock_name() forces that first reading, and
   do_time calls it while printing the header, which is before the first timed
   call. */
#include <sys/resource.h>
#if defined(__APPLE__)
#include <mach/mach.h>
#define WFB_CPU_PRIMARY_NAME "task_info"
static uint64_t wfb_time_value_ns(time_value_t t) {
    return (uint64_t)t.seconds * UINT64_C(1000000000)
         + (uint64_t)t.microseconds * UINT64_C(1000);
}
static int wfb_cpu_primary_ns(uint64_t *out) {
    task_thread_times_info_data_t live;
    task_basic_info_data_t exited;
    mach_msg_type_number_t live_count = TASK_THREAD_TIMES_INFO_COUNT;
    mach_msg_type_number_t exited_count = TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), TASK_THREAD_TIMES_INFO,
                  (task_info_t)&live, &live_count) != KERN_SUCCESS) return 0;
    if (task_info(mach_task_self(), TASK_BASIC_INFO,
                  (task_info_t)&exited, &exited_count) != KERN_SUCCESS) return 0;
    *out = wfb_time_value_ns(live.user_time) + wfb_time_value_ns(live.system_time)
         + wfb_time_value_ns(exited.user_time) + wfb_time_value_ns(exited.system_time);
    return 1;
}
#elif defined(CLOCK_PROCESS_CPUTIME_ID)
#define WFB_CPU_PRIMARY_NAME "CLOCK_PROCESS_CPUTIME_ID"
static int wfb_cpu_primary_ns(uint64_t *out) {
    struct timespec t;
    if (clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &t) != 0) return 0;
    *out = (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
    return 1;
}
#else
#define WFB_CPU_PRIMARY_NAME WFB_CPU_FALLBACK_NAME
static int wfb_cpu_primary_ns(uint64_t *out) { (void)out; return 0; }
#endif

#define WFB_CPU_FALLBACK_NAME "getrusage(RUSAGE_SELF)"

static uint64_t wfb_cpu_fallback_ns(void) {
    struct rusage r;
    uint64_t ns;
    if (getrusage(RUSAGE_SELF, &r) != 0) wfb_fail("getrusage failed");
    ns = (uint64_t)r.ru_utime.tv_sec * UINT64_C(1000000000)
       + (uint64_t)r.ru_utime.tv_usec * UINT64_C(1000);
    ns += (uint64_t)r.ru_stime.tv_sec * UINT64_C(1000000000)
       + (uint64_t)r.ru_stime.tv_usec * UINT64_C(1000);
    return ns;
}

/* -1 before the first reading, 1 once the host's primary source has answered,
   0 once the fallback has been taken. */
static int wfb_cpu_primary_live = -1;

uint64_t wfb_cpu_ns(void) {
    uint64_t ns;
    if (wfb_cpu_primary_live != 0 && wfb_cpu_primary_ns(&ns)) {
        wfb_cpu_primary_live = 1;
        return ns;
    }
    if (wfb_cpu_primary_live > 0) wfb_fail("the process CPU source failed after answering once");
    wfb_cpu_primary_live = 0;
    return wfb_cpu_fallback_ns();
}

const char *wfb_cpu_clock_name(void) {
    if (wfb_cpu_primary_live < 0) (void)wfb_cpu_ns();
    return wfb_cpu_primary_live == 1 ? WFB_CPU_PRIMARY_NAME : WFB_CPU_FALLBACK_NAME;
}

uint64_t wfb_clock_floor_ns(void) {
    static uint64_t floor_ns;
    static int measured;
    if (!measured) {
        uint64_t least = UINT64_MAX;
        for (unsigned i = 0; i < 1000; ++i) {
            uint64_t a = wfb_now_ns(), b = wfb_now_ns();
            if (b - a < least) least = b - a;
        }
        floor_ns = least;
        measured = 1;
    }
    return floor_ns;
}

WFB_NORETURN void wfb_fail(const char *message) {
    (void)fprintf(stderr, "compute-bench: %s\n", message ? message : "failure");
    (void)fflush(stderr);
    exit(1);
}

/* -------------------------------------------------------------- backends -- */

static const wfb_backend *const wfb_table[] = {
    &wfb_backend_serial,
    &wfb_backend_static,
    &wfb_backend_tbb,
#ifndef WFB_NO_PARLAY
    &wfb_backend_parlay,
    &wfb_backend_parlay_left,
#endif
    &wfb_backend_rayon_join,
    &wfb_backend_rayon_join_left,
    &wfb_backend_rayon_iter,
};

const wfb_backend *wfb_backend_named(const char *name) {
    if (!name) return NULL;
    for (size_t i = 0; i < sizeof wfb_table / sizeof wfb_table[0]; ++i)
        if (strcmp(wfb_table[i]->name, name) == 0) return wfb_table[i];
    return NULL;
}

/* The reason a named reference is absent from this build. The ParlayLib probe
   in `make deps` is the only thing that removes one today. */
static const char *wfb_absent_reason(void) {
#ifdef WFB_PARLAY_REASON
    return WFB_PARLAY_REASON;
#else
    return "this build carries no backend of that name";
#endif
}

/* ---------------------------------------------------------- forms, width -- */

static int form_is_wf(const char *form) { return strcmp(form, "wf") == 0; }
static int form_is_control(const char *form) { return strcmp(form, "wf-seq") == 0; }

/* The reason this kernel lists a form it does not implement, or NULL when it
   implements every form it lists. The pairs live in the kernel file because
   the fact is about that kernel's decomposition, not about the form's name:
   `rayon-iter` is a real reference for the three independent maps and is
   absent by construction only for the recursive one. */
static const char *form_unsupported(const wfb_kernel *k, const char *form) {
    if (!k->unsupported) return NULL;
    for (const char *const *p = k->unsupported; p[0] && p[1]; p += 2)
        if (strcmp(p[0], form) == 0) return p[1];
    return NULL;
}

static const char *form_grain(const wfb_kernel *k, const char *form) {
    const wfb_backend *b;
    const char *why = form_unsupported(k, form);
    if (form_is_wf(form)) return "compiler-chosen";
    if (form_is_control(form)) return "control";
    if (why) return why;
    b = wfb_backend_named(form);
    return b ? b->grain : wfb_absent_reason();
}

static const char *form_class(const wfb_kernel *k, const char *form) {
    if (form_is_wf(form)) return "wf";
    if (form_is_control(form)) return "control";
    if (form_unsupported(k, form)) return "n/a";
    return wfb_backend_named(form) ? "reference" : "n/a";
}

/* The fixed mark a reference carries into the table's `note` column. It comes
   from the backend's own struct, so a reference states it once beside its
   policy; the driver never derives it from a form's name. The `wf` row's note
   is filled by the reducer from the emitted chunk count and the steal median,
   which are measurements, so it is empty here. */
static const char *form_note(const wfb_kernel *k, const char *form) {
    const wfb_backend *b;
    if (form_is_wf(form) || form_is_control(form)) return "";
    if (form_unsupported(k, form)) return "";
    b = wfb_backend_named(form);
    return (b && b->note) ? b->note : "";
}

static unsigned wfb_cpus(void) {
    long online = sysconf(_SC_NPROCESSORS_ONLN);
    return online > 0 ? (unsigned)online : 1u;
}

/* Section 5.0's width rule, written once. The set is {1, 2, 4, 8, 16, 32};
   every width at most the online CPU count is emitted, plus the smallest
   width in the set above it when there is one. A width above the count is still
   emitted and is marked oversubscribed, because oversubscription rewards
   schedulers that yield and changes which one wins, so such a block carries
   no verdict. */
static const unsigned wfb_width_set[] = {1u, 2u, 4u, 8u, 16u, 32u};

static unsigned wfb_widths(unsigned *out) {
    unsigned cpus = wfb_cpus(), n = 0, i;
    for (i = 0; i < sizeof wfb_width_set / sizeof wfb_width_set[0]; ++i) {
        out[n++] = wfb_width_set[i];
        if (wfb_width_set[i] > cpus) break;
    }
    return n;
}

static unsigned parse_count(const char *text, unsigned long ceiling, const char *what) {
    char *end;
    unsigned long value;
    if (!text || !*text) wfb_fail(what);
    for (const char *p = text; *p; ++p)
        if (*p < '0' || *p > '9') wfb_fail(what);
    value = strtoul(text, &end, 10);
    if (*end || value > ceiling) wfb_fail(what);
    return (unsigned)value;
}

/* WF_WORKERS is the one variable the table sets. The Whitefoot runtime reads
   it at scheduler entry; the references size their pools from the WIDTH
   argument. Disagreement would silently compare two different widths, so the
   two are cross-checked before anything else happens. */
static unsigned parse_width(const char *text) {
    const char *workers = getenv("WF_WORKERS");
    unsigned width = parse_count(text, WFB_MAX_WIDTH, "WIDTH must be 1..WFB_MAX_WIDTH");
    if (width < 1) wfb_fail("WIDTH must be at least 1");
    if (!workers || !*workers) wfb_fail("WF_WORKERS must be set and must equal WIDTH");
    if (parse_count(workers, WFB_MAX_WIDTH, "WF_WORKERS must be 1..WFB_MAX_WIDTH") != width)
        wfb_fail("WF_WORKERS does not equal WIDTH");
    return width;
}

static void require_form(const wfb_kernel *k, const char *form) {
    for (const char *const *f = k->forms; *f; ++f)
        if (strcmp(*f, form) == 0) {
            if (strcmp(form_class(k, form), "n/a") == 0) {
                (void)printf("# %s SKIP: form=%s reason=%s\n", k->name, form,
                             form_unsupported(k, form) ? form_unsupported(k, form)
                                                       : wfb_absent_reason());
                (void)fflush(stdout);
                exit(0);
            }
            return;
        }
    wfb_fail("this kernel has no such form");
}

/* The emitted chunk count. Reported so the table's `note` column prints what
   was actually compared, and used by a kernel to size a verify fixture above
   the admission floor. It is a report, never an input: nothing here selects a
   chunk count, and there is no environment override for the split budget --
   harness.c's main unsets WF_SPLIT_WORK precisely so this default cannot be
   moved under a recorded row.

   IT IS ASKED OF THE LINKED RUNTIME AND NOT RE-DERIVED HERE.
   `wf__par_split_budget` is the same function the compiled program calls at
   the loop's entry, so `note` reports the split the image will really take
   rather than a copy of the rule that was true when this file was last
   edited. A copy would have disagreed with its own image twice over: once
   when the work unit moved, and again when the oversubscription cap did --
   both of which the Makefile's WF_RUNTIME_CONTROL_FLAGS and its A/B twin make
   an ordinary thing to do, and the twin puts two differently built runtimes in
   one table where a single copied rule can only describe one of them.

   The runtime answers for its own lane count rather than for the `width`
   argument, and those are the same number: parse_width above requires
   WF_WORKERS to equal WIDTH, and WF_WORKERS is what the scheduler sizes its
   lanes from. The argument is kept because the width-one answer is this
   function's own -- the runtime has no pool there to ask. It is called with no
   work outstanding on the calling thread, which is the state the budget is a
   statement about; the harness calls it before the first timed call and the
   two split fixtures call it after a completed join.

   The admission floor below is the work term alone -- 2 * ceil(work / weight),
   the smallest span the splitter can afford two chunks of -- so it reads the
   work unit directly. The oversubscription cap cannot lower it: a cap is a
   minimum against the affordable count and never raises the span a split
   needs. */

static size_t split_divisor(size_t weight) {
    size_t work = (size_t)wf__sched_split_work();
    size_t divisor;
    if (!weight || !work) return 0;
    divisor = (SIZE_MAX / 2 < weight) ? 1 : (work + weight - 1) / weight;
    return divisor ? divisor : 1;
}

size_t wfb_split_floor(size_t weight) {
    size_t divisor = split_divisor(weight);
    return divisor ? 2 * divisor : 0;
}

size_t wfb_split_chunks(size_t span, size_t weight, unsigned width) {
    uint64_t budget;
    if (!span || !weight || width < 2) return 0;
    budget = wf__par_split_budget((uint64_t)span, (uint64_t)weight);
    if (!budget || budget >= (uint64_t)(sizeof(size_t) * 8)) return 0;
    return (size_t)1 << budget;
}

static size_t wfb_chunks(const wfb_kernel *k, unsigned width) {
    return wfb_split_chunks(k->split_span, k->split_weight, width);
}

static void print_chunks(const wfb_kernel *k, const char *form, unsigned width) {
    size_t chunks = wfb_chunks(k, width);
    if (form_is_wf(form) && chunks) (void)printf(" chunks=%zu", chunks);
    else (void)printf(" chunks=na");
}

/* ------------------------------------------------------------- the modes -- */

static int do_list(const wfb_kernel *k) {
    unsigned widths[sizeof wfb_width_set / sizeof wfb_width_set[0]];
    unsigned n = wfb_widths(widths), i;
    (void)printf("# cpus=%u widths=", wfb_cpus());
    for (i = 0; i < n; ++i) (void)printf("%s%u", i ? " " : "", widths[i]);
    (void)printf("\n");
    for (const char *const *f = k->forms; *f; ++f)
        (void)printf("%s\t%s\t%s\n", *f, form_grain(k, *f), form_class(k, *f));
    return fflush(stdout) == 0 ? 0 : 1;
}

static void shut_down(const wfb_kernel *k, const char *form,
                      int *explicit_shutdown, uint64_t *shutdown_ns) {
    const wfb_backend *b = wfb_backend_named(form);
    uint64_t start = wfb_now_ns();
    k->finish(form);
    *explicit_shutdown = (b && b->stop) ? b->stop() : 0;
    *shutdown_ns = wfb_now_ns() - start;
}

static int do_verify(const wfb_kernel *k, const char *form, unsigned width) {
    size_t compared;
    int explicit_shutdown;
    uint64_t shutdown_ns;
    require_form(k, form);
    k->prepare(width);
    compared = k->verify(form, width);
    shut_down(k, form, &explicit_shutdown, &shutdown_ns);
    if (compared == 0) wfb_fail("verify compared nothing");
    (void)printf("# %s VERIFY PASS: form=%s width=%u compared=%zu\n",
                 k->name, form, width, compared);
    return fflush(stdout) == 0 ? 0 : 1;
}

static int do_time(const wfb_kernel *k, const char *form, unsigned width,
                   unsigned pass, unsigned calls) {
    unsigned cpus = wfb_cpus();
    size_t outputs = 0, compared = 0;
    int explicit_shutdown;
    uint64_t shutdown_ns;
    require_form(k, form);
    if (calls < 1) wfb_fail("CALLS must be at least 1");

    k->prepare(width);
    (void)printf("# driver=%s form=%s grain=%s width=%u cpus=%u oversubscribed=%d"
                 " calls=%u pass=%u workload=%s clock=%s clock_floor_ns=%llu"
                 " cpu_clock=%s note=%s",
                 k->name, form, form_grain(k, form), width, cpus,
                 width > cpus ? 1 : 0, calls, pass,
                 k->workload ? k->workload : "unknown", WFB_CLOCK_NAME,
                 (unsigned long long)wfb_clock_floor_ns(), wfb_cpu_clock_name(),
                 form_note(k, form));
    print_chunks(k, form, width);
    (void)printf("\n");
    (void)printf("# columns=kernel form width pass call phase wall_ns cpu_ns outputs steals\n");

    /* Call zero is the warm-up. It absorbs lazy pool, arena and registry
       startup, is verified like every other call, and enters no statistic.
       Verification between calls warms the data deliberately and is never
       inside the interval.

       The wall clock stays the outermost pair and the CPU clock is nested
       inside it, so the wall interval still encloses exactly one `call()` plus
       two CPU clock reads and nothing else. The nesting is this way round on
       purpose: wall is the primary measurement and keeps the widest bracket, so
       no CPU the call spends can fall outside the wall interval, and the two
       nested reads cost 757 ns as a pair where that was timed, on the Linux
       host of the 2026-09-11 agreement record, against a per-call interval of
       milliseconds -- orders below this bundle's own MAD, and the agreement of a
       before/after `compare` on one tree is what checks that rather than the
       arithmetic. The CPU figure is a process figure, so it counts every
       worker or lane thread the form started, spinning ones included; that is
       the point of the column. */
    for (unsigned call = 0; call <= calls; ++call) {
        unsigned long before = wf__par_grants();
        uint64_t a = wfb_now_ns();
        uint64_t ca = wfb_cpu_ns();
        size_t produced = k->call(form, width);
        uint64_t cb = wfb_cpu_ns();
        uint64_t b = wfb_now_ns();
        unsigned long after = wf__par_grants();
        compared += k->check();
        outputs += produced;
        (void)printf("%s\t%s\t%u\t%u\t%u\t%s\t%llu\t%llu\t%zu\t%lu\n", k->name, form,
                     width, pass, call, call ? "warm" : "first",
                     (unsigned long long)(b - a), (unsigned long long)(cb - ca),
                     produced, after - before);
    }

    shut_down(k, form, &explicit_shutdown, &shutdown_ns);
    (void)printf("# batch form=%s width=%u calls=%u outputs=%zu compared=%zu"
                 " explicit_shutdown=%d shutdown_ns=%llu\n",
                 form, width, calls, outputs, compared, explicit_shutdown,
                 (unsigned long long)shutdown_ns);
    return fflush(stdout) == 0 ? 0 : 1;
}

int wf__main_body(int argc, char **argv) {
    const wfb_kernel *k = &wfb_this_kernel;
    if (argc == 2 && strcmp(argv[1], "list") == 0) return do_list(k);
    if (argc == 4 && strcmp(argv[1], "verify") == 0)
        return do_verify(k, argv[2], parse_width(argv[3]));
    if (argc == 6 && strcmp(argv[1], "time") == 0)
        return do_time(k, argv[2], parse_width(argv[3]),
                       parse_count(argv[4], 1024, "PASS"),
                       parse_count(argv[5], 1024, "CALLS"));
    wfb_fail("usage: <image> list | verify FORM WIDTH | time FORM WIDTH PASS CALLS");
}

/* The four unsets happen here, before wf__floor_run, and nowhere else: the
   scheduler core reads its settings at entry, which is before wf__main_body
   runs, and ends the process there on a value outside a setting's ceiling.
   Unsetting them from the harness body would be too late. WF_SPLIT_WORK is
   unset for the same reason even where it does nothing: a split-budget
   override must never silently change a recorded row. WF_WORKERS is never
   unset -- it is the one variable the table sets, and parse_width above
   cross-checks it against the WIDTH argument. */
int main(int argc, char **argv) {
    if (unsetenv("WF_SCHED_REPORT") || unsetenv("WF_STACKS") ||
        unsetenv("WF_IO_HELPERS") || unsetenv("WF_SPLIT_WORK"))
        wfb_fail("cannot clear the scheduler environment");
    return wf__floor_run(argc, argv);
}
