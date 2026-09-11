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

/* Section 5.0's width rule, written once. The set is {1, 2, 4, 8}; every
   width at most the online CPU count is emitted, plus the smallest width in
   the set above it when there is one. A width above the count is still
   emitted and is marked oversubscribed, because oversubscription rewards
   schedulers that yield and changes which one wins, so such a block carries
   no verdict. */
static const unsigned wfb_width_set[] = {1u, 2u, 4u, 8u};

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

/* The emitted chunk count, as compiler/src/backend/sched/core.c computes it:
   chunks = 2^floor(log2(min(16 * lanes, span / ceil(split_work / weight)))).
   Reported so the table's `note` column prints what was actually compared,
   and used by a kernel to size a verify fixture above the admission floor.
   It is a report, never an input: nothing here selects a chunk count, and
   there is no environment override for the split budget -- harness.c's main
   unsets WF_SPLIT_WORK precisely so this default cannot be moved under a
   recorded row. The constant is wf__sched_split_work()'s default in
   compiler/src/backend/sched/entry.c. */
#define WFB_SPLIT_WORK ((size_t)1200000)

static size_t split_divisor(size_t weight) {
    size_t divisor;
    if (!weight) return 0;
    divisor = (SIZE_MAX / 2 < weight) ? 1 : (WFB_SPLIT_WORK + weight - 1) / weight;
    return divisor ? divisor : 1;
}

size_t wfb_split_floor(size_t weight) {
    size_t divisor = split_divisor(weight);
    return divisor ? 2 * divisor : 0;
}

size_t wfb_split_chunks(size_t span, size_t weight, unsigned width) {
    size_t divisor = split_divisor(weight), affordable, want, bound, chunks = 1;
    if (!span || !divisor || width < 2) return 0;
    affordable = span / divisor;
    want = (size_t)16 * width;
    bound = affordable < want ? affordable : want;
    while (chunks * 2 <= bound) chunks *= 2;
    return bound == 0 ? 0 : chunks;
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
                 " calls=%u pass=%u workload=%s clock=%s clock_floor_ns=%llu note=%s",
                 k->name, form, form_grain(k, form), width, cpus,
                 width > cpus ? 1 : 0, calls, pass,
                 k->workload ? k->workload : "unknown", WFB_CLOCK_NAME,
                 (unsigned long long)wfb_clock_floor_ns(), form_note(k, form));
    print_chunks(k, form, width);
    (void)printf("\n");
    (void)printf("# columns=kernel form width pass call phase wall_ns outputs steals\n");

    /* Call zero is the warm-up. It absorbs lazy pool, arena and registry
       startup, is verified like every other call, and enters no statistic.
       Verification between calls warms the data deliberately and is never
       inside the interval. */
    for (unsigned call = 0; call <= calls; ++call) {
        unsigned long before = wf__par_grants();
        uint64_t a = wfb_now_ns();
        size_t produced = k->call(form, width);
        uint64_t b = wfb_now_ns();
        unsigned long after = wf__par_grants();
        compared += k->check();
        outputs += produced;
        (void)printf("%s\t%s\t%u\t%u\t%u\t%s\t%llu\t%zu\t%lu\n", k->name, form,
                     width, pass, call, call ? "warm" : "first",
                     (unsigned long long)(b - a), produced, after - before);
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
