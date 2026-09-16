/* Complete-result oracle for the formal mandelbrot program. Correctness and
 * the separate paired performance runner share inputs and expected results. */
#include "oracle.h"
#include <inttypes.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#pragma STDC FP_CONTRACT OFF

const char *const wf_oracle_name = "mandelbrot";
const char *const wf_oracle_fixture = "points=98304 limit=256 shape=trailing seed=828219";

extern void wf_bench_mandelbrot(const double *, const double *, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_mandelbrot_release(uint64_t *, uint64_t);
typedef struct {
    double *x, *y, *held_x, *held_y;
    uint64_t *expected, *output;
    size_t n;
    uint64_t limit;
} Work;

static uint64_t reference(double real, double imaginary, uint64_t limit) {
    volatile double x = 0, y = 0;
    for (uint64_t i = 0; i < limit; ++i) {
        volatile double xx = x * x, yy = y * y, magnitude = xx + yy;
        if (magnitude > 4) return i;
        volatile double xy = x * y, twice = xy + xy, difference = xx - yy;
        y = twice + imaginary;
        x = difference + real;
    }
    return limit;
}

static uint32_t random_next(uint32_t *s) {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    return *s;
}

static Work input(size_t n, uint64_t limit, const char *shape, uint32_t seed) {
    Work w;
    size_t bytes = (n ? n : 1) * sizeof(double);
    int kind;
    memset(&w, 0, sizeof w);
    if (!(n <= 1048576 && limit <= 65536))
        wf_oracle_fail("mandelbrot: input domain");
    w.n = n;
    w.limit = limit;
    w.x = malloc(bytes);
    w.y = malloc(bytes);
    w.held_x = malloc(bytes);
    w.held_y = malloc(bytes);
    w.expected = malloc((n ? n : 1) * sizeof(uint64_t));
    if (!w.x || !w.y || !w.held_x || !w.held_y || !w.expected)
        wf_oracle_fail("mandelbrot: input allocation");
    kind = !strcmp(shape, "plane")         ? 0
           : !strcmp(shape, "boundary")    ? 1
           : !strcmp(shape, "clustered")   ? 2
           : !strcmp(shape, "interleaved") ? 3
           : !strcmp(shape, "interior")    ? 4
           : !strcmp(shape, "exterior")    ? 5
           : !strcmp(shape, "trailing")    ? 6
                                           : -1;
    if (kind < 0) wf_oracle_fail("mandelbrot: input shape");
    for (size_t i = 0; i < n; ++i) {
        if (kind < 2) {
            double a = (double)(random_next(&seed) & 65535) / 65536;
            double b = (double)(random_next(&seed) & 65535) / 65536;
            w.x[i] = kind == 0 ? -2 + 3 * a : -0.75 + (a - 0.5) / 32;
            w.y[i] = kind == 0 ? -1.5 + 3 * b : 0.125 + (b - 0.5) / 32;
        } else {
            int inside = kind == 4 || (kind == 2 && i < (n + 3) / 4) ||
                         (kind == 3 && i % 4 == 0) || (kind == 6 && i >= n - (n + 3) / 4);
            w.x[i] = inside ? 0 : 3;
            w.y[i] = 0;
        }
        w.expected[i] = reference(w.x[i], w.y[i], limit);
    }
    memcpy(w.held_x, w.x, n * sizeof(double));
    memcpy(w.held_y, w.y, n * sizeof(double));
    return w;
}

static void release(Work *w) {
    if (w->output) wf_bench_mandelbrot_release(w->output, w->n);
    w->output = NULL;
}
static void run(Work *w) {
    uint64_t length = UINT64_MAX;
    wf_bench_mandelbrot(w->x, w->y, w->n, w->limit, &w->output, &length);
    if (length != w->n) wf_oracle_fail("mandelbrot: output extent");
}

static size_t compare(Work *w) {
    if (!w->output && w->n) wf_oracle_fail("mandelbrot: output pointer");
    for (size_t i = 0; i < w->n; ++i)
        if (w->output[i] != w->expected[i]) {
            (void)fprintf(stderr,
                          "mandelbrot: point=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n", i,
                          w->expected[i], w->output[i]);
            wf_oracle_fail("mandelbrot: wrong output bits");
        }
    if (memcmp(w->x, w->held_x, w->n * sizeof(double)) ||
        memcmp(w->y, w->held_y, w->n * sizeof(double)))
        wf_oracle_fail("mandelbrot: input immutability");
    return w->n;
}

static void destroy(Work *w) {
    release(w);
    free(w->x);
    free(w->y);
    free(w->held_x);
    free(w->held_y);
    free(w->expected);
    memset(w, 0, sizeof *w);
}

size_t wf_oracle_verify(void) {
    static const char *const shapes[] = {"plane", "boundary", "clustered", "interleaved", "interior", "exterior", "trailing"};
    static const size_t counts[] = {0, 1, 3, 7, 33, 257, 4097};
    static const uint64_t limits[] = {0, 1, 16, 256};
    size_t compared = 0;
    for (size_t s = 0; s < sizeof shapes / sizeof shapes[0]; ++s)
        for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
            for (size_t k = 0; k < sizeof limits / sizeof limits[0]; ++k) {
                Work w = input(counts[n], limits[k], shapes[s], 828219);
                run(&w);
                compared += compare(&w);
                destroy(&w);
            }
    return compared;
}

#ifdef WF_ORACLE_PERFORMANCE
static Work timed;
void wf_oracle_prepare(void) { timed = input(98304, 256, "trailing", 828219); }
size_t wf_oracle_call(void) { run(&timed); return timed.n; }
size_t wf_oracle_check(void) {
    size_t compared = compare(&timed);
    release(&timed);
    return compared;
}
void wf_oracle_finish(void) { destroy(&timed); }
#endif
