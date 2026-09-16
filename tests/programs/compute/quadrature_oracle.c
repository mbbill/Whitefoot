/* Complete-result oracle for the formal quadrature program. Correctness and
 * the separate paired performance runner share inputs and expected results. */
#include "oracle.h"
#include <inttypes.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#pragma STDC FP_CONTRACT OFF

const char *const wf_oracle_name = "quadrature";
const char *const wf_oracle_fixture = "integrations=64 tolerance=0x1p-54 depth=24";

#define QD_INTEGRATIONS ((size_t)64)
#define QD_TOLERANCE 0x1p-54
#define QD_DEPTH 24u
extern double wf_bench_quadrature(double, double, double, double, double, uint64_t);

typedef struct {
    const char *name;
    double a, b, center, width, tolerance;
    unsigned depth;
    int converged;
} Input;

/* Converged peaks, depth exhaustion and empty/reversed intervals. Hex-float
 * inputs pin the actual binary64 values rather than decimal approximations. */
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
} Node;
typedef struct { double value; uint64_t capped; } Reference;

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
                    if (!n->depth && fabs(delta) > threshold) ++out.capped;
                } else {
                    volatile double tol = n->tol * 0.5;
                    Node child;
                    n->state = 1;
                    if (top >= 24) wf_oracle_fail("quadrature: oracle stack domain");
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
            double value;
            Node *parent;
            value = n->result;
            if (!top) {
                out.value = value;
                return out;
            }
            --top;
            parent = &stack[top];
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
                if (parent->state != 2) wf_oracle_fail("quadrature: oracle traversal");
                sum = parent->result + value;
                parent->result = sum;
                parent->state = 3;
            }
        }
    }
}

static void anchor(const Input *p, const Reference *r) {
    long double exact, center = (long double)p->center, w = (long double)p->width;
    if (!p->converged) return;
    exact = w * (atanl(((long double)p->b - center) / w) - atanl(((long double)p->a - center) / w));
    if (r->capped != 0) wf_oracle_fail("quadrature: unexpected depth exhaustion");
    if (!(fabsl((long double)r->value - exact) <= 8 * (long double)p->tolerance + 0x1p-48L))
        wf_oracle_fail("quadrature: analytic integral");
}

static double integration(const Input *p) {
    return wf_bench_quadrature(p->a, p->b, p->center, p->width, p->tolerance, p->depth);
}
static Input batch[QD_INTEGRATIONS], held[QD_INTEGRATIONS];
static double expected[QD_INTEGRATIONS];
static double *output;
static int prepared;

static void release(void) {
    free(output);
    output = NULL;
}

static void prepare(void) {
    size_t i;
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
        Reference r = reference(&batch[i]);
        anchor(&batch[i], &r);
        expected[i] = r.value;
    }
    memcpy(held, batch, sizeof batch);
    prepared = 1;
}

static size_t call(void) {
    size_t bytes = QD_INTEGRATIONS * sizeof(double), i;
    output = malloc(bytes);
    if (!output) wf_oracle_fail("quadrature: output allocation");
    memset(output, 0, bytes);
    for (i = 0; i < QD_INTEGRATIONS; ++i) output[i] = integration(&batch[i]);
    return QD_INTEGRATIONS;
}

static size_t check(void) {
    size_t i;
    if (!output) wf_oracle_fail("quadrature: output pointer");
    for (i = 0; i < QD_INTEGRATIONS; ++i)
        if (bits(output[i]) != bits(expected[i])) {
            (void)fprintf(stderr,
                          "quadrature: interval=%zu expected=%a actual=%a expected_bits=%" PRIx64
                          " actual_bits=%" PRIx64 "\n",
                          i, expected[i], output[i], bits(expected[i]), bits(output[i]));
            wf_oracle_fail("quadrature: wrong output bits");
        }
    if (memcmp(batch, held, sizeof batch)) wf_oracle_fail("quadrature: input immutability");
    release();
    return QD_INTEGRATIONS;
}

size_t wf_oracle_verify(void) {
    size_t compared = 0;
    for (size_t i = 0; i < sizeof cases / sizeof cases[0]; ++i) {
        Reference r = reference(&cases[i]);
        anchor(&cases[i], &r);
        double actual = integration(&cases[i]);
        if (bits(actual) != bits(r.value)) {
            fprintf(stderr, "quadrature: %s expected=%a actual=%a\n", cases[i].name, r.value, actual);
            wf_oracle_fail("quadrature: fixture result");
        }
        ++compared;
    }
    prepare();
    (void)call();
    compared += check();
    prepared = 0;
    return compared;
}
#ifdef WF_ORACLE_PERFORMANCE
void wf_oracle_prepare(void) { prepare(); }
size_t wf_oracle_call(void) { return call(); }
size_t wf_oracle_check(void) { return check(); }
void wf_oracle_finish(void) { release(); prepared = 0; }
#endif
