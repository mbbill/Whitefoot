/* Serves compute-bench: the one driver linked into every kernel image. It
 * owns the process protocol, the width rule, the clock, the timed loop, the
 * call cadence, the TSV the reducer reads, the backend lookup table, and the
 * `main` that routes through the Whitefoot floor. Nothing here knows what any
 * kernel computes.
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

/* Process CPU time. CLOCK_PROCESS_CPUTIME_ID is POSIX and is what both hosted
   legs have: Linux since 2.6.12 and Darwin since 10.12. getrusage is the
   fallback for a host that defines neither, and it is coarser (microseconds),
   which is why it is not the first choice on any host that has the clock. The
   fallback sums user and system time, as the clock does. */
#if defined(CLOCK_PROCESS_CPUTIME_ID)
#define WFB_CPU_CLOCK_NAME "CLOCK_PROCESS_CPUTIME_ID"
uint64_t wfb_cpu_ns(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &t) != 0) wfb_fail("CPU clock_gettime failed");
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
#else
#define WFB_CPU_CLOCK_NAME "getrusage(RUSAGE_SELF)"
#include <sys/resource.h>
uint64_t wfb_cpu_ns(void) {
    struct rusage r;
    uint64_t ns;
    if (getrusage(RUSAGE_SELF, &r) != 0) wfb_fail("getrusage failed");
    ns = (uint64_t)r.ru_utime.tv_sec * UINT64_C(1000000000)
       + (uint64_t)r.ru_utime.tv_usec * UINT64_C(1000);
    ns += (uint64_t)r.ru_stime.tv_sec * UINT64_C(1000000000)
       + (uint64_t)r.ru_stime.tv_usec * UINT64_C(1000);
    return ns;
}
#endif

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

/* -------------------------------------------------------------- the gap -- */

/* WFB_GAP_US is the gap between consecutive timed calls, in microseconds. With
   WF_WORKERS it is one of the two variables the table sets. Unset, empty or
   zero is the back-to-back cadence of every recorded table and is exactly what
   this file did before the mode existed; a non-zero value asks the other
   question, the one a program with sequential work between its parallel
   regions poses: how much of a runtime's win survives when the next call does
   not arrive while the lanes are still hot.

   IT APPLIES TO EVERY FORM IDENTICALLY, references included. A gap that
   reached only the Whitefoot row would be measuring one scheduler's idle
   policy against another scheduler's warm one, which is not a comparison.

   The ceiling is a fixed structural bound on a run-time setting, so that a
   mistyped value cannot hold a process for an unbounded time. It selects
   nothing, and no elapsed time reaches a verdict here any more than anywhere
   else in this file. */
#define WFB_MAX_GAP_US 1000000uL

/* The wait is a busy-wait on the monotonic clock and deliberately NOT
   nanosleep: the point of the mode is that the calling thread stays running,
   as a program doing its own sequential work between parallel regions does,
   while the runtime's helper lanes go idle and park. A sleeping driver thread
   would hand its CPU back and measure something else.

   It is outside every measured interval. Both clocks are started after this
   returns, so no part of the gap is in a reported wall or CPU figure -- the
   whole of what the gap does to a call is in the call's own numbers. */
static void wfb_wait_gap(unsigned gap_us) {
    uint64_t deadline;
    if (!gap_us) return;
    deadline = wfb_now_ns() + (uint64_t)gap_us * UINT64_C(1000);
    while (wfb_now_ns() < deadline) { }
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

/* Read once per process, in every mode, so a malformed value is refused by the
   cheap `verify` sweep as well as by a long `compare` run instead of only by
   the one that waits on it. Absent and empty both mean zero: the Makefile hands
   the variable down only when it is set, and a reader running an image by hand
   gets the back-to-back cadence without setting anything. */
static unsigned wfb_gap_us(void) {
    const char *text = getenv("WFB_GAP_US");
    if (!text || !*text) return 0;
    return parse_count(text, WFB_MAX_GAP_US, "WFB_GAP_US must be 0..WFB_MAX_GAP_US");
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

/* The gap is READ AND REPORTED HERE AND NOT WAITED. `verify` drives no timed
   call loop: the whole fixture sweep is one call into the kernel's own
   `verify`, which owns its grid and is never measured, so the only place a
   gap could go is inside each kernel's fixture loop -- between calls that no
   clock brackets and no table reports. A wait there would lengthen `verify`
   and check nothing, so this mode makes `verify` slower by nothing at all.
   What it does do is refuse a malformed WFB_GAP_US in the cheap sweep that
   runs before a long `compare`, which is why the Makefile hands the variable
   to both. */
static int do_verify(const wfb_kernel *k, const char *form, unsigned width) {
    size_t compared;
    int explicit_shutdown;
    uint64_t shutdown_ns;
    unsigned gap_us = wfb_gap_us();
    require_form(k, form);
    k->prepare(width);
    compared = k->verify(form, width);
    shut_down(k, form, &explicit_shutdown, &shutdown_ns);
    if (compared == 0) wfb_fail("verify compared nothing");
    (void)printf("# %s VERIFY PASS: form=%s width=%u gap_us=%u compared=%zu\n",
                 k->name, form, width, gap_us, compared);
    return fflush(stdout) == 0 ? 0 : 1;
}

static int do_time(const wfb_kernel *k, const char *form, unsigned width,
                   unsigned pass, unsigned calls) {
    unsigned cpus = wfb_cpus();
    size_t outputs = 0, compared = 0;
    int explicit_shutdown;
    uint64_t shutdown_ns;
    unsigned gap_us = wfb_gap_us();
    require_form(k, form);
    if (calls < 1) wfb_fail("CALLS must be at least 1");

    k->prepare(width);
    /* `gap_us` goes in the header because the gap is part of what this process
       measured: a row taken at one cadence and a row taken at another are two
       measurements, and the reducer refuses to put them in one table. It sits
       ahead of `workload` and `note`, each of which the reducer reads as
       everything up to the next key. */
    (void)printf("# driver=%s form=%s grain=%s width=%u cpus=%u oversubscribed=%d"
                 " gap_us=%u calls=%u pass=%u workload=%s clock=%s"
                 " clock_floor_ns=%llu cpu_clock=%s note=%s",
                 k->name, form, form_grain(k, form), width, cpus,
                 width > cpus ? 1 : 0, gap_us, calls, pass,
                 k->workload ? k->workload : "unknown", WFB_CLOCK_NAME,
                 (unsigned long long)wfb_clock_floor_ns(), WFB_CPU_CLOCK_NAME,
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
       nested reads cost tens of nanoseconds against a per-call interval of
       milliseconds -- three to four orders below this bundle's own MAD, and the
       agreement of a before/after `compare` on one tree is what checks that
       rather than the arithmetic. The CPU figure is a process figure, so it
       counts every worker or lane thread the form started, spinning ones
       included; that is the point of the column. */
    for (unsigned call = 0; call <= calls; ++call) {
        unsigned long before;
        uint64_t a, ca;
        size_t produced;
        /* The gap, between consecutive calls and nowhere else: nothing precedes
           the warm-up call, and by the time the wait returns the previous
           call's verification and its printf are long done, so the wait is the
           last thing that happens before the clock starts. That is the state
           the mode is about -- the lanes have had the whole gap with no work
           to find. */
        if (call) wfb_wait_gap(gap_us);
        before = wf__par_grants();
        a = wfb_now_ns();
        ca = wfb_cpu_ns();
        produced = k->call(form, width);
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
    (void)printf("# batch form=%s width=%u calls=%u gap_us=%u outputs=%zu"
                 " compared=%zu explicit_shutdown=%d shutdown_ns=%llu\n",
                 form, width, calls, gap_us, outputs, compared, explicit_shutdown,
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
   override must never silently change a recorded row. WF_WORKERS and
   WFB_GAP_US are never unset -- they are the two variables the table sets,
   neither is read by the scheduler, and each is reported by the process that
   read it: parse_width above cross-checks WF_WORKERS against the WIDTH
   argument, and the gap goes into the header and the trailer of every batch it
   shaped. */
int main(int argc, char **argv) {
    if (unsetenv("WF_SCHED_REPORT") || unsetenv("WF_STACKS") ||
        unsetenv("WF_IO_HELPERS") || unsetenv("WF_SPLIT_WORK"))
        wfb_fail("cannot clear the scheduler environment");
    return wf__floor_run(argc, argv);
}
