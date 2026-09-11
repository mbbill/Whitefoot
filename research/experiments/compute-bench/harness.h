/* Serves compute-bench: the interface between the one driver (harness.c) and
 * each kernel's fixtures, oracle and dispatch (<kernel>_bench.c). Every
 * kernel image is harness.c plus exactly one <kernel>_bench.c, so this file
 * is the whole contract between them. */
#ifndef WFB_HARNESS_H
#define WFB_HARNESS_H
#include <stddef.h>
#include <stdint.h>

#if defined(__cplusplus)
#define WFB_NORETURN [[noreturn]]
extern "C" {
#else
#define WFB_NORETURN _Noreturn
#endif

/* Wall time in nanoseconds. CLOCK_MONOTONIC_RAW where the host defines it,
   CLOCK_MONOTONIC otherwise. Never CPU time, never process time. */
uint64_t wfb_now_ns(void);

/* The minimum over 1000 back-to-back empty clock pairs, measured once per
   process. Printed in the header. Never subtracted from anything. */
uint64_t wfb_clock_floor_ns(void);

/* Process CPU time in nanoseconds: every thread of this process summed, user
   plus system. CLOCK_PROCESS_CPUTIME_ID where the host defines it and
   getrusage(RUSAGE_SELF) otherwise. It is the whole process and not one thread
   deliberately: what this bundle wants to know is what a decomposition costs in
   CPU across every lane or worker it started, which a spinning scheduler shows
   in and a wall clock hides. Never wall time, and never a pass/fail input. */
uint64_t wfb_cpu_ns(void);

/* End the run with a message on stderr and a nonzero status. It reports a
   wrong result, a missing form or a bad invocation. It never reports a slow
   one: no elapsed time can reach it. */
WFB_NORETURN void wfb_fail(const char *message);

/* The scheduler core's steal count, from compiler/src/backend/sched/core.c.
   Read around each timed call so a WF row that started no lane is visible in
   the table as `no-lanes` instead of looking merely slow. Referencing this
   symbol is also the link-time proof that the real runtime is present: the
   emitted module carries weak no-op stubs for every other wf__par_* symbol
   but none for this one. */
unsigned long wf__par_grants(void);

/* The linked scheduler's independent-map work unit, from
   compiler/src/backend/sched/entry.c: how much work, in the estimated IR
   instructions per iteration the emitted weight operand counts, one chunk must
   be worth before the splitter can afford another. Read rather than copied, so
   the chunk count in `note` and the two split verify fixtures below can never
   disagree with the runtime the image actually links; it has no weak stub in
   the emitted module either, so referencing it is a second link-time proof
   that the real runtime is present. `main` unsets WF_SPLIT_WORK before
   anything runs, so this always answers the runtime's compiled default. */
uint64_t wf__sched_split_work(void);

/* The log2 of the chunk count the linked scheduler grants a loop of `span`
   iterations whose emitted constant weight is `weight`, from
   compiler/src/backend/sched/core.c: it is the same function the compiled
   program calls at the loop's entry, and zero where that scheduler does not
   split at all. The harness asks it rather than re-deriving its rule, so the
   `note` column and the split verify fixtures describe the split the image
   will really take under its own work unit and its own oversubscription cap.
   Unlike the two symbols above, this one DOES have a weak no-op stub in the
   emitted module, so referencing it proves nothing about the link; what keeps
   the reported count from being a stub's zero is the Makefile's post-link
   assertion that the image carries a strong wf__par_split_budget. */
uint64_t wf__par_split_budget(uint64_t span, uint64_t weight);

/* The independent-map split the linked scheduler computes for a loop of
   `span` iterations whose emitted constant weight is `weight`, at `width`
   lanes: `wf__par_split_budget` above, raised to a chunk count, and 0 where
   that scheduler does not split at all -- at width one, at weight zero, or
   below the admission floor. `wfb_split_floor` returns that floor, the
   smallest span the scheduler will split, 2 * ceil(split_work / weight), so a
   kernel can size a fixture from the module's emitted weight instead of
   storing a point count that a change of weight would silently turn back into
   an unsplit run. Both are reports and neither is an input: nothing in this
   bundle selects, requests or overrides a chunk count at run time, and there
   is no split-budget environment variable on `main`. Both read the linked
   runtime, so a run whose runtime was built at another work unit or another
   cap -- the Makefile's WF_RUNTIME_CONTROL_FLAGS and its A/B twin, which mark
   their own table -- is reported here correctly instead of being described by
   a stale copy of a rule. */
size_t wfb_split_chunks(size_t span, size_t weight, unsigned width);
size_t wfb_split_floor(size_t weight);

typedef struct {
    const char *name;            /* "fir" | "records" | "mandelbrot" | "quadrature" */
    const char *const *forms;    /* NULL-terminated; the table's row names */

    /* Build the inputs and the independent oracle's expected result. Once per
       process, before any call, outside every timed interval. */
    void   (*prepare)(unsigned width);

    /* One complete call of `form` at `width` on the timed fixture, leaving the
       observable result where check() reads it. Returns output elements. This
       is the only thing the clock encloses. */
    size_t (*call)(const char *form, unsigned width);

    /* Compare the last call's result with the oracle, bit for bit, and check
       that the inputs were not modified. Returns elements compared; calls
       wfb_fail on any difference. Never inside a timed interval. */
    size_t (*check)(void);

    /* The kernel's whole fixture suite through `form` at `width`, every
       fixture checked against the oracle. Returns elements compared. Never
       timed. */
    size_t (*verify)(const char *form, unsigned width);

    /* Release this form's pool and the process's storage, after the last
       timed call. Its cost is never inside a timed interval.
       The harness calls the form's wfb_backend `stop()` immediately after
       this returns and reports its value as `explicit_shutdown`, so a kernel
       must release only its own storage here and must not call stop itself:
       stop must run exactly once per process. */
    void   (*finish)(const char *form);

    /* The kernel's size constants as the header's `workload` value, for
       instance "points=98304 limit=256 shape=trailing seed=828219". A row is
       then readable without the Makefile. May be NULL. */
    const char *workload;

    /* The WF program's independent-map split as the linked runtime computes
       it, so the table's `note` column can print the emitted chunk count
       without a reader re-deriving it. `split_span` is the timed fixture's
       loop span; `split_weight` is the constant second operand of the
       `wf__par_split_budget` call in $(BUILD)/<kernel>-par.ll, which the
       Makefile extracts into $(BUILD)/<kernel>_split.h as WFB_SPLIT_WEIGHT.
       Both zero when the --par module has no independent-map split, which is
       the case for a kernel that parallelizes by recursion; the harness then
       prints chunks=na. Nothing here selects or changes what is measured. */
    size_t split_span;
    size_t split_weight;

    /* Forms this kernel lists in `forms` but deliberately does not implement,
       as a NULL-terminated sequence of (form name, reason) pairs. The driver
       reports such a form with class `n/a` and prints the reason where a
       grain would go, and the Makefile's cell list drops every `n/a` row, so
       the form is disclosed in `list` and never run or timed. It exists
       because a form can be absent by construction for one kernel while being
       a real reference for the other three: quadrature decomposes by adaptive
       recursion, which offers no enumerable range for a parallel iterator to
       split, and a flattened frontier would be a different algorithm. This is
       a per-kernel statement about the kernel's own decomposition, never a
       property read from a form's name. May be NULL. */
    const char *const *unsupported;
} wfb_kernel;

extern const wfb_kernel wfb_this_kernel;   /* defined by <kernel>_bench.c */

#if defined(__cplusplus)
}
#endif
#endif
