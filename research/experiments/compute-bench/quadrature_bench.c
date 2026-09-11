/* Serves compute-bench: the quadrature kernel -- recursive fork-join over
 * unequal subtrees. It owns the fixtures, the independent explicit-stack
 * oracle and its analytic anchor, the scalar C recursion, the frontier
 * flattening the static reference needs, the dispatch over every form, and
 * the per-call bitwise checks.
 *
 * Adapted from the research bundle's quadrature.c and quadrature_native.cpp.
 * `density`, `simpson` and `adaptive` are quadrature.c:83-103 verbatim, and
 * the explicit-stack `reference()` is quadrature.c:107-178 with its volatile
 * binary64 temporaries and its traversal unchanged. `forked()` is the
 * budget-cutoff shape of quadrature_native.cpp's `adaptive<Kind>` template,
 * expressed once over backend.h's `fork2` instead of once per library.
 * Named cuts: the bundle's wf-native/wf-value/wf-value-right C adapters over
 * the frozen research runtime, which are not Whitefoot numbers; its
 * environment-driven batch panel and perf control protocol; and the oracle's
 * `refusal_forks` and `node_span` counters, which served the cut refusal form
 * and its unit-node work model and have no reader here.
 *
 * THE TIMED INTERVAL IS LOAD-BEARING FOR FAIRNESS and is identical for every
 * implementation of this kernel: allocate the output buffer, zero-fill it,
 * compute, join every participant. Release and free are outside it.
 * Allocation and the zero-fill are inside because Whitefoot's buffer
 * construction allocates and zero-fills, so every form pays malloc plus
 * memset inside the interval. A re-cut that lets one path allocate
 * uninitialized compares allocators, not schedulers.
 *
 * The zero-fill is also this kernel's omission witness: every expected value
 * of the timed batch is a strictly positive integral, so a sub-interval no
 * form ever wrote reads 0.0 and check() fails on it. That is why no extra
 * poison pass runs in the timed path, where it would be work no other form
 * pays.
 *
 * M, the number of integrations in the batch, is this kernel's one size
 * constant. The batch runs its M integrations IN SEQUENCE, each one
 * internally parallel: one fork-join tree per integration, nothing parallel
 * across the batch. That restriction binds every form here, references
 * included. A form that ran the M integrations in parallel with each other
 * would have replaced this kernel's decomposition with an independent map,
 * and the panel already carries three independent maps. */
#include "backend.h"
#include "harness.h"
#include "quadrature_split.h"

#include <inttypes.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The timed fixture: M independent integrations over M disjoint unit
 * sub-intervals. Sub-interval i is the same left-peak Lorentz profile
 * translated onto its own unit interval, so sub-interval 0 is one plain
 * integration and every other is that integration translated. All six
 * quantities are exact in binary64 at these magnitudes -- i + 0x1p-5 needs
 * eleven significand bits at i = 63, and every Simpson midpoint down to
 * depth 24 needs at most thirty -- so no sub-interval is a different problem
 * in disguise. Left-peak rather than centre-peak because unequal subtrees are
 * the entire reason this kernel is in the panel.
 *
 * M is the knob and the tolerance is not: the recursion saturates at about
 * 149,831 nodes because `combined - whole` underflows to exactly zero at fine
 * subdivision, so no tolerance reaches the [5 ms, 60 ms] wf-seq window, and
 * `depth` cannot be raised past the source's `requires depth <= 24_u64`. */
#define QD_INTEGRATIONS ((size_t)64)
#define QD_TOLERANCE 0x1p-54
#define QD_DEPTH 24u

/* The frontier for every form that has a scheduler: the depth at which the
 * offered recursion stops offering and the static reference stops
 * materializing. One number for all of them, fixed policy, never calibrated
 * per host. */
#define QD_FRONTIER 8u

/* 2^(QD_FRONTIER+1) - 1 nodes and 2^QD_FRONTIER leaves bound the frontier
 * tree exactly, because it is a binary tree of height QD_FRONTIER. */
#define QD_FRONTIER_NODES ((size_t)512)
#define QD_FRONTIER_TASKS ((size_t)256)

extern double wf_bench_quadrature_par(double, double, double, double, double, uint64_t);
extern double wf_bench_quadrature_seq(double, double, double, double, double, uint64_t);

typedef struct {
    const char *name;
    double a, b, center, width, tolerance;
    unsigned depth;
    int converged;
} Input;

/* The research bundle's ten, as hex-float literals. None of its pinned node
 * counts is copied into a check: the oracle prints what it computes. */
static const Input cases[] = {
    {"smooth", 0x0p+0, 0x1p+0, 0x1p-1, 0x1p+0, 0x1p-30, 24, 1},
    {"center-peak", 0x0p+0, 0x1p+0, 0x1p-1, 0x1p-6, 0x1p-42, 24, 1},
    {"left-peak", 0x0p+0, 0x1p+0, 0x1p-5, 0x1p-6, 0x1p-42, 24, 1},
    {"right-peak", 0x0p+0, 0x1p+0, 0x1.fp-1, 0x1p-6, 0x1p-42, 24, 1},
    {"outside-peak", 0x0p+0, 0x1p+0, 0x1.4p+0, 0x1p-4, 0x1p-42, 24, 1},
    {"loose", 0x0p+0, 0x1p+0, 0x1p-1, 0x1p-2, 0x1p-8, 24, 1},
    {"depth-zero", 0x0p+0, 0x1p+0, 0x1p-1, 0x1p-6, 0x0p+0, 0, 0},
    {"depth-cap", 0x0p+0, 0x1p+0, 0x1p-5, 0x1p-6, 0x0p+0, 12, 0},
    {"empty", 0x1p-1, 0x1p-1, 0x1p-1, 0x1p-2, 0x1p-30, 24, 1},
    {"reverse", 0x1p+0, 0x0p+0, 0x1p-1, 0x1p-2, 0x1p-30, 24, 1},
};

static uint64_t bits(double x) {
    uint64_t v;
    memcpy(&v, &x, sizeof v);
    return v;
}

/* ------------------------------------------- the scalar C recursion ------ */
/* quadrature.c:83-103 verbatim. It is the `serial` form's whole body -- one
 * thread, no cutoff, no scheduler branch -- and it is also the specialization
 * every offered form drops into below the frontier, so that no node under the
 * frontier pays a scheduler test. No SIMD, no FMA: the Makefile's SCALAR
 * flags reach this translation unit. */
static double density(double x, double center, double width) {
    double z = (x - center) / width;
    return 1 / (1 + z * z);
}

static double simpson(double a, double b, double fa, double fm, double fb) {
    return ((b - a) / 6) * ((fa + 4 * fm) + fb);
}

static double adaptive(double a, double b, double c, double w, double fa, double fm, double fb,
                       double whole, double tolerance, unsigned depth) {
    double m = (a + b) * 0.5, fl = density((a + m) * 0.5, c, w), fr = density((m + b) * 0.5, c, w);
    double left = simpson(a, m, fa, fl, fm), right = simpson(m, b, fm, fr, fb);
    double combined = left + right, delta = combined - whole;
    if (!depth || fabs(delta) <= 15 * tolerance) return combined + delta / 15;
    double l = adaptive(a, m, c, w, fa, fl, fm, left, tolerance * 0.5, depth - 1);
    double r = adaptive(m, b, c, w, fm, fr, fb, right, tolerance * 0.5, depth - 1);
    return l + r;
}

static double native(const Input *p) {
    double fa = density(p->a, p->center, p->width);
    double fm = density((p->a + p->b) * 0.5, p->center, p->width);
    double fb = density(p->b, p->center, p->width);
    double whole = simpson(p->a, p->b, fa, fm, fb);
    return adaptive(p->a, p->b, p->center, p->width, fa, fm, fb, whole, p->tolerance, p->depth);
}

/* --------------------------------------------------------- the oracle ---- */
/* Independent explicit-stack postorder traversal. It preserves the declared
 * left-plus-right sum order while being structurally unlike every recursive
 * implementation here, and volatile binary64 temporaries force every
 * specified rounding. It never runs inside a timed interval. */
static double oracle_density(double x, const Input *p) {
    volatile double d = x - p->center, z = d / p->width, zz = z * z, den = 1 + zz, value = 1 / den;
    return value;
}

static double oracle_simpson(double a, double b, double fa, double fm, double fb) {
    volatile double span = b - a, scale = span / 6, weighted = 4 * fm, partial = fa + weighted,
                    sum = partial + fb, value = scale * sum;
    return value;
}

typedef struct {
    double a, b, fa, fm, fb, whole, tol, m, fl, fr, left, right, result;
    unsigned depth, state;
    uint64_t subtree_nodes;
} Node;

typedef struct {
    double value;
    uint64_t nodes, leaves, capped, forks;
    uint64_t frontier_blocks, frontier_nodes, largest_subtree;
    unsigned deepest;
} Reference;

static Reference reference(const Input *p) {
    Node stack[25];
    unsigned top = 0;
    Reference out;
    volatile double endpoints = p->a + p->b, m = endpoints * 0.5;
    double fa, fm, fb;
    memset(stack, 0, sizeof stack);
    memset(&out, 0, sizeof out);
    fa = oracle_density(p->a, p);
    fm = oracle_density(m, p);
    fb = oracle_density(p->b, p);
    stack[0].a = p->a;
    stack[0].b = p->b;
    stack[0].fa = fa;
    stack[0].fm = fm;
    stack[0].fb = fb;
    stack[0].whole = oracle_simpson(p->a, p->b, fa, fm, fb);
    stack[0].tol = p->tolerance;
    stack[0].depth = p->depth;
    for (;;) {
        Node *n = &stack[top];
        if (!n->state) {
            n->subtree_nodes = 1;
            ++out.nodes;
            if (top > out.deepest) out.deepest = top;
            {
                volatile double sum = n->a + n->b, mid = sum * 0.5, ls = n->a + mid, lm = ls * 0.5,
                                rs = mid + n->b, rm = rs * 0.5;
                n->m = mid;
                n->fl = oracle_density(lm, p);
                n->fr = oracle_density(rm, p);
            }
            n->left = oracle_simpson(n->a, n->m, n->fa, n->fl, n->fm);
            n->right = oracle_simpson(n->m, n->b, n->fm, n->fr, n->fb);
            {
                volatile double combined = n->left + n->right, delta = combined - n->whole,
                                threshold = 15 * n->tol;
                if (!n->depth || fabs(delta) <= threshold) {
                    volatile double correction = delta / 15, value = combined + correction;
                    n->result = value;
                    n->state = 3;
                    ++out.leaves;
                    if (!n->depth && fabs(delta) > threshold) ++out.capped;
                } else {
                    volatile double tol = n->tol * 0.5;
                    Node child;
                    if (top < QD_FRONTIER) ++out.forks;
                    n->state = 1;
                    if (top >= 24) wfb_fail("quadrature: oracle stack domain");
                    memset(&child, 0, sizeof child);
                    child.a = n->a;
                    child.b = n->m;
                    child.fa = n->fa;
                    child.fm = n->fl;
                    child.fb = n->fm;
                    child.whole = n->left;
                    child.tol = tol;
                    child.depth = n->depth - 1;
                    stack[++top] = child;
                    continue;
                }
            }
        }
        if (n->state == 3) {
            /* The frontier partition, outside timing and outside every
             * implementation: a node at the frontier, and a node above it
             * whose whole subtree is one node, is one block of the partition
             * the static reference materializes. */
            double value;
            uint64_t child_nodes;
            Node *parent;
            if (top == QD_FRONTIER || (top < QD_FRONTIER && n->subtree_nodes == 1)) {
                ++out.frontier_blocks;
                out.frontier_nodes += n->subtree_nodes;
                if (n->subtree_nodes > out.largest_subtree) out.largest_subtree = n->subtree_nodes;
            }
            value = n->result;
            if (!top) {
                out.value = value;
                if (out.frontier_blocks != out.forks + 1 ||
                    out.frontier_nodes + out.forks != out.nodes)
                    wfb_fail("quadrature: oracle frontier partition");
                return out;
            }
            child_nodes = n->subtree_nodes;
            --top;
            parent = &stack[top];
            parent->subtree_nodes += child_nodes;
            if (parent->state == 1) {
                volatile double tol = parent->tol * 0.5;
                Node child;
                parent->result = value;
                parent->state = 2;
                memset(&child, 0, sizeof child);
                child.a = parent->m;
                child.b = parent->b;
                child.fa = parent->fm;
                child.fm = parent->fr;
                child.fb = parent->fb;
                child.whole = parent->right;
                child.tol = tol;
                child.depth = parent->depth - 1;
                stack[++top] = child;
            } else {
                volatile double sum;
                if (parent->state != 2) wfb_fail("quadrature: oracle traversal");
                sum = parent->result + value;
                parent->result = sum;
                parent->state = 3;
            }
        }
    }
}

/* The analytic antiderivative of the Lorentz profile, in long double, as an
 * independent anchor for a fixture the oracle says converged. It is never an
 * equality test: equality is bitwise against the oracle. */
static void anchor(const Input *p, const Reference *r) {
    long double exact, center = (long double)p->center, w = (long double)p->width;
    if (!p->converged) return;
    exact = w * (atanl(((long double)p->b - center) / w) - atanl(((long double)p->a - center) / w));
    if (r->capped != 0) wfb_fail("quadrature: unexpected depth exhaustion");
    if (!(fabsl((long double)r->value - exact) <= 8 * (long double)p->tolerance + 0x1p-48L))
        wfb_fail("quadrature: analytic integral");
}

/* ------------------------------------- the offered recursion, once ------- */
/* One shape for every library: above the budget each node hands its two
 * subtrees to backend.h's fork2 and joins them; at budget zero the scalar
 * specialization above runs with no scheduler branch anywhere beneath it.
 * `left_offer` -- which subtree the library publishes and which it runs
 * locally -- belongs to the backend struct, not here, so both directions are
 * the same code. */
typedef struct {
    const wfb_backend *backend;
    unsigned lanes;
    double a, b, c, w, fa, fm, fb, whole, tolerance;
    unsigned depth, budget;
    double result;
} Frame;

static double forked(const wfb_backend *backend, unsigned lanes, double a, double b, double c,
                     double w, double fa, double fm, double fb, double whole, double tolerance,
                     unsigned depth, unsigned budget);

static void frame_task(void *opaque) {
    Frame *f = opaque;
    f->result = forked(f->backend, f->lanes, f->a, f->b, f->c, f->w, f->fa, f->fm, f->fb, f->whole,
                       f->tolerance, f->depth, f->budget);
}

static double forked(const wfb_backend *backend, unsigned lanes, double a, double b, double c,
                     double w, double fa, double fm, double fb, double whole, double tolerance,
                     unsigned depth, unsigned budget) {
    if (!budget) return adaptive(a, b, c, w, fa, fm, fb, whole, tolerance, depth);
    {
        double m = (a + b) * 0.5, fl = density((a + m) * 0.5, c, w),
               fr = density((m + b) * 0.5, c, w);
        double left = simpson(a, m, fa, fl, fm), right = simpson(m, b, fm, fr, fb);
        double combined = left + right, delta = combined - whole;
        Frame l, r;
        if (!depth || fabs(delta) <= 15 * tolerance) return combined + delta / 15;
        l.backend = backend; l.lanes = lanes;
        l.a = a; l.b = m; l.c = c; l.w = w;
        l.fa = fa; l.fm = fl; l.fb = fm; l.whole = left;
        l.tolerance = tolerance * 0.5; l.depth = depth - 1; l.budget = budget - 1; l.result = 0;
        r.backend = backend; r.lanes = lanes;
        r.a = m; r.b = b; r.c = c; r.w = w;
        r.fa = fm; r.fm = fr; r.fb = fb; r.whole = right;
        r.tolerance = tolerance * 0.5; r.depth = depth - 1; r.budget = budget - 1; r.result = 0;
        backend->fork2(backend, lanes, frame_task, &l, frame_task, &r);
        return l.result + r.result;
    }
}

/* ----------------------------------- the materialized frontier tree ------ */
/* The static reference has no fork2 -- it is a persistent pool over a flat
 * task list -- so this kernel hands it one. The descent materializes exactly
 * the nodes the real recursion would have: a subtree that converges above the
 * frontier becomes a leaf whose value the descent already settled, and a node
 * at the frontier becomes a task for its whole subtree. Every node's work is
 * done exactly once, in the descent or in its task, never twice.
 *
 * The recombination follows the frontier tree pairwise, in the shape the
 * recursion would have used, because fadd is not associative and a
 * left-to-right fold over the task results would change the bits.
 *
 * ONE DISPATCH PER INTEGRATION, M per timed call, never one dispatch over the
 * union of the M frontiers: a batch-wide dispatch would give this reference a
 * decomposition no fork-join form can have. The M dispatch costs are real and
 * are charged to it, as every other form's M tree startups are charged to
 * them. */
typedef struct {
    int left, right; /* child node indices; left < 0 marks a task leaf */
    double a, b, c, w, fa, fm, fb, whole, tolerance;
    unsigned depth;
    int settled; /* 1 when the descent already computed `value` */
    double value;
} Block;

typedef struct {
    Block nodes[QD_FRONTIER_NODES];
    size_t count;
    int leaves[QD_FRONTIER_TASKS];
    int order[QD_FRONTIER_TASKS];
    size_t tasks;
} Frontier;

/* One integration at a time, in the caller's thread: the batch is sequential
 * by construction, so one tree serves the whole call and costs no allocation
 * inside the timed interval beyond the output buffer every form allocates. */
static Frontier frontier;

static int frontier_build(Frontier *f, double a, double b, double c, double w, double fa, double fm,
                          double fb, double whole, double tolerance, unsigned depth,
                          unsigned level) {
    int index;
    Block *n;
    if (f->count >= QD_FRONTIER_NODES) wfb_fail("quadrature: frontier node capacity");
    index = (int)f->count++;
    n = &f->nodes[index];
    n->left = -1;
    n->right = -1;
    n->settled = 0;
    n->value = 0;
    n->a = a; n->b = b; n->c = c; n->w = w;
    n->fa = fa; n->fm = fm; n->fb = fb; n->whole = whole;
    n->tolerance = tolerance;
    n->depth = depth;
    if (level == QD_FRONTIER) return index;
    {
        double m = (a + b) * 0.5, fl = density((a + m) * 0.5, c, w),
               fr = density((m + b) * 0.5, c, w);
        double left = simpson(a, m, fa, fl, fm), right = simpson(m, b, fm, fr, fb);
        double combined = left + right, delta = combined - whole;
        int li, ri;
        if (!depth || fabs(delta) <= 15 * tolerance) {
            n->settled = 1;
            n->value = combined + delta / 15;
            return index;
        }
        li = frontier_build(f, a, m, c, w, fa, fl, fm, left, tolerance * 0.5, depth - 1, level + 1);
        ri = frontier_build(f, m, b, c, w, fm, fr, fb, right, tolerance * 0.5, depth - 1,
                            level + 1);
        f->nodes[index].left = li;
        f->nodes[index].right = ri;
    }
    return index;
}

static void frontier_collect(Frontier *f, int index) {
    Block *n = &f->nodes[index];
    if (n->left < 0) {
        if (f->tasks >= QD_FRONTIER_TASKS) wfb_fail("quadrature: frontier task capacity");
        f->leaves[f->tasks++] = index;
        return;
    }
    frontier_collect(f, n->left);
    frontier_collect(f, n->right);
}

/* Round-robin hand-out with no knowledge of task cost. backend_static
 * partitions the chunk-index space into equal contiguous blocks, so laying
 * the leaves out lane by lane -- lane j receiving leaves j, j+lanes,
 * j+2*lanes, ... -- makes that partition exactly round-robin. The two agree
 * by construction: lane j's contiguous block holds `base + (j < extra)`
 * entries and its round-robin stride gives it ceil((tasks - j) / lanes),
 * which is the same number. If the partition were ever to change this stays a
 * cost-blind distribution and stops being exactly round-robin; it never
 * becomes cost-aware, and it never changes a result. */
static void frontier_order(Frontier *f, unsigned lanes) {
    size_t at = 0;
    unsigned j;
    for (j = 0; j < lanes; ++j) {
        size_t i;
        for (i = j; i < f->tasks; i += lanes) f->order[at++] = f->leaves[i];
    }
    if (at != f->tasks) wfb_fail("quadrature: frontier ordering");
}

static void frontier_task(void *opaque, size_t index) {
    Frontier *f = opaque;
    Block *n = &f->nodes[f->order[index]];
    if (n->settled) return;
    n->value = adaptive(n->a, n->b, n->c, n->w, n->fa, n->fm, n->fb, n->whole, n->tolerance,
                        n->depth);
}

static double frontier_value(const Frontier *f, int index) {
    const Block *n = &f->nodes[index];
    if (n->left < 0) return n->value;
    return frontier_value(f, n->left) + frontier_value(f, n->right);
}

static double flattened(const wfb_backend *backend, unsigned lanes, double a, double b, double c,
                        double w, double fa, double fm, double fb, double whole, double tolerance,
                        unsigned depth) {
    int root;
    frontier.count = 0;
    frontier.tasks = 0;
    root = frontier_build(&frontier, a, b, c, w, fa, fm, fb, whole, tolerance, depth, 0);
    frontier_collect(&frontier, root);
    frontier_order(&frontier, lanes);
    backend->map(backend, lanes, frontier.tasks, frontier_task, &frontier);
    return frontier_value(&frontier, root);
}

/* -------------------------------------------------------- the dispatch --- */

static double integration(const char *form, unsigned width, const Input *p) {
    if (!strcmp(form, "wf"))
        return wf_bench_quadrature_par(p->a, p->b, p->center, p->width, p->tolerance,
                                       (uint64_t)p->depth);
    if (!strcmp(form, "wf-seq"))
        return wf_bench_quadrature_seq(p->a, p->b, p->center, p->width, p->tolerance,
                                       (uint64_t)p->depth);
    if (!strcmp(form, "serial")) return native(p);
    {
        const wfb_backend *backend = wfb_backend_named(form);
        double fa, fm, fb, whole;
        if (!backend) wfb_fail("quadrature: no such form");
        fa = density(p->a, p->center, p->width);
        fm = density((p->a + p->b) * 0.5, p->center, p->width);
        fb = density(p->b, p->center, p->width);
        whole = simpson(p->a, p->b, fa, fm, fb);
        if (backend->fork2)
            return forked(backend, width, p->a, p->b, p->center, p->width, fa, fm, fb, whole,
                          p->tolerance, p->depth, QD_FRONTIER);
        if (backend->map)
            return flattened(backend, width, p->a, p->b, p->center, p->width, fa, fm, fb, whole,
                             p->tolerance, p->depth);
        wfb_fail("quadrature: this form has no dispatch");
    }
}

/* ------------------------------------------------------------ the kernel -- */

static Input batch[QD_INTEGRATIONS];
static Input held[QD_INTEGRATIONS];
static double expected[QD_INTEGRATIONS];
static Reference oracle[QD_INTEGRATIONS];
static double *output;
static int prepared;

static void release(void) {
    free(output);
    output = NULL;
}

static void qd_prepare(unsigned width) {
    size_t i;
    (void)width;
    if (prepared) return;
    for (i = 0; i < QD_INTEGRATIONS; ++i) {
        batch[i].name = "batch";
        batch[i].a = (double)i;
        batch[i].b = (double)i + 0x1p+0;
        batch[i].center = (double)i + 0x1p-5;
        batch[i].width = 0x1p-6;
        batch[i].tolerance = QD_TOLERANCE;
        batch[i].depth = QD_DEPTH;
        batch[i].converged = 1;
        oracle[i] = reference(&batch[i]);
        anchor(&batch[i], &oracle[i]);
        expected[i] = oracle[i].value;
    }
    memcpy(held, batch, sizeof batch);
    prepared = 1;
}

/* One complete call on the timed batch. This is the whole timed interval and
 * nothing else: allocate, zero-fill, run the M integrations in sequence, and
 * join every participant of each one before the next begins. */
static size_t qd_call(const char *form, unsigned width) {
    size_t bytes = QD_INTEGRATIONS * sizeof(double), i;
    output = malloc(bytes);
    if (!output) wfb_fail("quadrature: output allocation");
    memset(output, 0, bytes);
    for (i = 0; i < QD_INTEGRATIONS; ++i) output[i] = integration(form, width, &batch[i]);
    return QD_INTEGRATIONS;
}

/* Outside every timed interval. The whole M-element result vector is compared
 * element by element against the oracle, so an implementation that got one
 * sub-interval wrong cannot hide inside a sum. */
static size_t qd_check(void) {
    size_t i;
    if (!output) wfb_fail("quadrature: output pointer");
    for (i = 0; i < QD_INTEGRATIONS; ++i)
        if (bits(output[i]) != bits(expected[i])) {
            (void)fprintf(stderr,
                          "quadrature: interval=%zu expected=%a actual=%a expected_bits=%" PRIx64
                          " actual_bits=%" PRIx64 "\n",
                          i, expected[i], output[i], bits(expected[i]), bits(output[i]));
            wfb_fail("quadrature: wrong output bits");
        }
    if (memcmp(batch, held, sizeof batch)) wfb_fail("quadrature: input immutability");
    release();
    return QD_INTEGRATIONS;
}

static void print_oracle(const Input *p, const Reference *r, size_t index) {
    (void)printf("# quadrature oracle input=%s index=%zu nodes=%" PRIu64 " leaves=%" PRIu64
                 " capped=%" PRIu64 " deepest=%u evaluations=%" PRIu64 " forks=%" PRIu64
                 " frontier_blocks=%" PRIu64 " frontier_nodes=%" PRIu64 " largest_subtree=%" PRIu64
                 " value=%a\n",
                 p->name, index, r->nodes, r->leaves, r->capped, r->deepest, 3 + 2 * r->nodes,
                 r->forks, r->frontier_blocks, r->frontier_nodes, r->largest_subtree, r->value);
}

/* The whole fixture suite through one form at one width: the ten single
 * fixtures and all M sub-intervals of the timed batch, each one checked
 * against the oracle separately. Never timed. No expected count, node total
 * or PASS string is stored here; every number printed is re-derived. */
static size_t qd_verify(const char *form, unsigned width) {
    size_t compared = 0, i;
    uint64_t batch_nodes = 0;
    for (i = 0; i < sizeof cases / sizeof cases[0]; ++i) {
        Reference r = reference(&cases[i]);
        double value;
        anchor(&cases[i], &r);
        print_oracle(&cases[i], &r, i);
        value = integration(form, width, &cases[i]);
        if (bits(value) != bits(r.value)) {
            (void)fprintf(stderr, "# quadrature fixture=%s expected=%a actual=%a\n", cases[i].name,
                          r.value, value);
            wfb_fail("quadrature: wrong fixture bits");
        }
        ++compared;
    }
    for (i = 0; i < QD_INTEGRATIONS; ++i) {
        double value = integration(form, width, &batch[i]);
        if (bits(value) != bits(expected[i])) {
            (void)fprintf(stderr, "# quadrature batch interval=%zu expected=%a actual=%a\n", i,
                          expected[i], value);
            wfb_fail("quadrature: wrong batch bits");
        }
        batch_nodes += oracle[i].nodes;
        ++compared;
    }
    if (memcmp(batch, held, sizeof batch)) wfb_fail("quadrature: input immutability");
    /* The timed batch's node total is a report, never an assertion. */
    (void)printf("# quadrature verify: fixtures=%zu integrations=%zu compared=%zu batch_nodes=%" PRIu64
                 " frontier=%u\n",
                 sizeof cases / sizeof cases[0], QD_INTEGRATIONS, compared, batch_nodes,
                 QD_FRONTIER);
    return compared;
}

static void qd_finish(const char *form) {
    (void)form;
    release();
    prepared = 0;
}

static const char *const qd_forms[] = {"wf",     "wf-seq",     "serial",          "static",
                                       "tbb",    "parlay",     "parlay-left",     "rayon-join",
                                       "rayon-join-left", "rayon-iter", NULL};

/* `rayon-iter` is listed and never run. A parallel iterator needs an
 * enumerable range; an adaptive recursion has none until it is flattened, and
 * a flattened-frontier iterator is a different algorithm from recursive
 * fork-join. Listing it with its reason is what section 5.1 asks for: a reader
 * of the form list sees why the row is missing instead of wondering whether it
 * was dropped. It is listed as unsupported for this kernel only -- the same
 * backend is a real reference for the three independent maps -- and the fact
 * is stated here, about this kernel's decomposition, rather than being read
 * off the form's name anywhere in the driver. */
static const char *const qd_unsupported[] = {
    "rayon-iter",
    "absent by construction: a parallel iterator needs an enumerable range and an "
    "adaptive recursion has none until it is flattened, and a flattened frontier is a "
    "different algorithm from recursive fork-join",
    NULL};

const wfb_kernel wfb_this_kernel = {
    "quadrature",
    qd_forms,
    qd_prepare,
    qd_call,
    qd_check,
    qd_verify,
    qd_finish,
    "integrations=64 tolerance=0x1p-54 depth=24",
    /* This kernel parallelizes by recursion structure, so its emitted module
     * has no independent-map split and no admission threshold at all: the
     * note column prints chunks=na. WFB_SPLIT_WEIGHT is read from the
     * generated header so that a codegen change which did introduce a split
     * would show up here rather than being silently ignored. */
    0,
    WFB_SPLIT_WEIGHT,
    qd_unsupported,
};
