/* Serves compute-bench: the Mandelbrot kernel -- loop with imbalance. It owns
 * the shapes and the generator, the independent oracle, the scalar C kernel
 * body, the dispatch over every form, and the per-call checks.
 *
 * Adapted from the research bundle's mandelbrot.c. `reference()` and
 * `native()` are copied verbatim, including native()'s documented
 * square-reuse loop rotation, which carries xx and yy across iterations and
 * recomputes them at the bottom where the Whitefoot source recomputes them at
 * the top: the same number and order of roundings, checked pointwise. The
 * generator and the seven shapes are copied verbatim. Cut: the bundle's
 * `wf` native selector (a C adapter over the frozen research runtime, which
 * is not a Whitefoot number), its whole-process command panel, its budget and
 * requested-chunk environment policies, and its printed qualification line
 * with stored counts.
 *
 * THE TIMED INTERVAL IS LOAD-BEARING FOR FAIRNESS and is identical for every
 * implementation of this kernel: allocate the output buffer, zero-fill it,
 * compute, join every participant. Release and free are outside it.
 * Allocation and the zero-fill are inside because Whitefoot's
 * buffer_new(count, 0) allocates and zero-fills, so every reference calls
 * malloc plus memset inside the interval. A re-cut that lets the native path
 * allocate uninitialized compares allocators, not schedulers.
 *
 * The UINT64_MAX poison that detects an omitted write is therefore not in the
 * timed path, where it would be a pass over the output that the Whitefoot
 * form does not pay: it runs in `verify`, which is never timed. In the timed
 * path the zero-fill is the omission witness, because every expected value of
 * the timed fixture is nonzero -- an exterior point escapes at iteration 1 and
 * an interior point returns the limit -- so an unwritten element reads 0 and
 * check() fails on it. */
#include "backend.h"
#include "harness.h"
#include "mandelbrot_split.h"

#include <inttypes.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The timed fixture. `trailing` puts the interior quarter last, which is this
 * kernel's stated role: maximum skew. 98,304 points is the middle of the band
 * that makes plain --par emit 16 chunks, and it is the largest size the
 * [5 ms, 60 ms] wf-seq window admits here; the window caps this row's chunk
 * count, not the scheduler. */
#define MB_POINTS ((size_t)98304)
#define MB_LIMIT ((uint64_t)256)
#define MB_SHAPE "trailing"
#define MB_SEED ((uint32_t)828219)
/* Callback grain: 64 points, giving 1,536 chunks at the default size against
 * the WF program's 16. It is the research bundle's fixed-grain panel setting,
 * not a per-host optimum; the bundle's own `trailing` optima were 16 points
 * per callback on Linux x86-64 and 256 on an Apple M1. */
#define MB_GRAIN ((size_t)64)

extern void wf_bench_mandelbrot_par(const double *, const double *, uint64_t, uint64_t,
                                    uint64_t **, uint64_t *);
extern void wf_bench_mandelbrot_par_release(uint64_t *, uint64_t);
extern void wf_bench_mandelbrot_seq(const double *, const double *, uint64_t, uint64_t,
                                    uint64_t **, uint64_t *);
extern void wf_bench_mandelbrot_seq_release(uint64_t *, uint64_t);

/* Volatile temporaries force each specified binary64 rounding in the oracle.
 * This is outside timing. */
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

static uint64_t native(double real, double imaginary, uint64_t limit) {
    double x = 0, y = 0, xx = 0, yy = 0;
    for (uint64_t i = 0; i < limit; ++i) {
        if (xx + yy > 4) return i;
        double xy = x * y;
        y = (xy + xy) + imaginary;
        x = (xx - yy) + real;
        xx = x * x;
        yy = y * y;
    }
    return limit;
}

typedef struct {
    double *x, *y, *held_x, *held_y;
    uint64_t *expected;
    uint64_t *output;
    int output_from_wf;
    size_t n, grain;
    uint64_t limit, iterations;
} Work;

static uint32_t random_next(uint32_t *s) {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    return *s;
}

static Work input(size_t n, uint64_t limit, const char *shape, uint32_t seed, size_t grain) {
    Work w;
    size_t bytes = (n ? n : 1) * sizeof(double);
    int kind;
    memset(&w, 0, sizeof w);
    if (!(n <= 1048576 && limit <= 65536 && grain >= 1 && grain <= 1048576))
        wfb_fail("mandelbrot: input domain");
    w.n = n;
    w.limit = limit;
    w.grain = grain;
    w.x = malloc(bytes);
    w.y = malloc(bytes);
    w.held_x = malloc(bytes);
    w.held_y = malloc(bytes);
    w.expected = malloc((n ? n : 1) * sizeof(uint64_t));
    if (!w.x || !w.y || !w.held_x || !w.held_y || !w.expected)
        wfb_fail("mandelbrot: input allocation");
    kind = !strcmp(shape, "plane")         ? 0
           : !strcmp(shape, "boundary")    ? 1
           : !strcmp(shape, "clustered")   ? 2
           : !strcmp(shape, "interleaved") ? 3
           : !strcmp(shape, "interior")    ? 4
           : !strcmp(shape, "exterior")    ? 5
           : !strcmp(shape, "trailing")    ? 6
                                           : -1;
    if (kind < 0) wfb_fail("mandelbrot: input shape");
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
        w.iterations += w.expected[i];
    }
    memcpy(w.held_x, w.x, n * sizeof(double));
    memcpy(w.held_y, w.y, n * sizeof(double));
    return w;
}

static void native_chunk(void *opaque, size_t chunk) {
    Work *w = opaque;
    size_t first = chunk * w->grain, end = first + w->grain;
    if (end > w->n) end = w->n;
    for (size_t i = first; i < end; ++i) w->output[i] = native(w->x[i], w->y[i], w->limit);
}

static void release(Work *w) {
    if (!w->output) return;
    if (w->output_from_wf) {
        if (w->grain == 0) wf_bench_mandelbrot_seq_release(w->output, w->n);
        else wf_bench_mandelbrot_par_release(w->output, w->n);
    } else {
        free(w->output);
    }
    w->output = NULL;
}

/* One complete call on `w`. This is the whole timed interval and nothing
 * else. `poison` is never set on the timed path; see the file comment. */
static void run(Work *w, const char *form, unsigned width, int poison) {
    if (!strcmp(form, "wf") || !strcmp(form, "wf-seq")) {
        uint64_t length = UINT64_MAX;
        uint64_t *out = NULL;
        /* The grain field doubles as the release selector for the two
         * modules: 0 marks the --no-overlap module's buffer. */
        w->grain = strcmp(form, "wf-seq") ? MB_GRAIN : 0;
        if (!strcmp(form, "wf-seq"))
            wf_bench_mandelbrot_seq(w->x, w->y, w->n, w->limit, &out, &length);
        else
            wf_bench_mandelbrot_par(w->x, w->y, w->n, w->limit, &out, &length);
        if (length != w->n) wfb_fail("mandelbrot: generated length");
        w->output = out;
        w->output_from_wf = 1;
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        size_t bytes = (w->n ? w->n : 1) * sizeof(uint64_t);
        if (!backend || !backend->map) wfb_fail("mandelbrot: no such form");
        w->output = malloc(bytes);
        if (!w->output) wfb_fail("mandelbrot: output allocation");
        w->output_from_wf = 0;
        memset(w->output, 0, w->n * sizeof(uint64_t));
        /* Verify only, and after the zero-fill so that it is the state the
         * callbacks actually see. The timed path never reaches this line. */
        if (poison) memset(w->output, 0xff, w->n * sizeof(uint64_t));
        backend->map(backend, width, (w->n + w->grain - 1) / w->grain, native_chunk, w);
    }
}

static size_t compare(Work *w) {
    if (!w->output && w->n) wfb_fail("mandelbrot: output pointer");
    for (size_t i = 0; i < w->n; ++i)
        if (w->output[i] != w->expected[i]) {
            (void)fprintf(stderr,
                          "mandelbrot: point=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n", i,
                          w->expected[i], w->output[i]);
            wfb_fail("mandelbrot: wrong output bits");
        }
    if (memcmp(w->x, w->held_x, w->n * sizeof(double)) ||
        memcmp(w->y, w->held_y, w->n * sizeof(double)))
        wfb_fail("mandelbrot: input immutability");
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

/* ------------------------------------------------------------ the kernel -- */

static Work timed;
static int prepared;

static void mb_prepare(unsigned width) {
    (void)width;
    if (prepared) return;
    timed = input(MB_POINTS, MB_LIMIT, MB_SHAPE, MB_SEED, MB_GRAIN);
    prepared = 1;
}

static size_t mb_call(const char *form, unsigned width) {
    timed.grain = MB_GRAIN;
    run(&timed, form, width, 0);
    return timed.n;
}

/* Outside every timed interval. It compares, then releases the call's buffer,
 * so the next call allocates from the same state this one did. */
static size_t mb_check(void) {
    size_t compared = compare(&timed);
    release(&timed);
    return compared;
}

static size_t mb_verify(const char *form, unsigned width) {
    static const char *const shapes[] = {"plane",    "boundary", "clustered", "interleaved",
                                         "interior", "exterior", "trailing"};
    static const size_t counts[] = {0, 1, 3, 7, 33, 257, 4097};
    static const uint64_t limits[] = {0, 1, 16, 256};
    static const double special[] = {-INFINITY, -3, -2, -1, -0.0, 0, 1, 2, 3, INFINITY, NAN};
    size_t compared = 0, leaves = 0, calls = 0;
    int generated = !strcmp(form, "wf") || !strcmp(form, "wf-seq");

    /* The special-point grid: the oracle against the scalar kernel body over
     * +/-INF, NaN, +/-0 and small integers, at several limits. */
    for (size_t i = 0; i < sizeof special / sizeof special[0]; ++i)
        for (size_t j = 0; j < sizeof special / sizeof special[0]; ++j)
            for (size_t k = 0; k < sizeof limits / sizeof limits[0]; ++k) {
                uint64_t want = reference(special[i], special[j], limits[k]);
                if (native(special[i], special[j], limits[k]) != want)
                    wfb_fail("mandelbrot: special point");
                ++leaves;
            }

    for (size_t s = 0; s < sizeof shapes / sizeof shapes[0]; ++s)
        for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
            for (size_t k = 0; k < sizeof limits / sizeof limits[0]; ++k) {
                Work w = input(counts[n], limits[k], shapes[s], MB_SEED, MB_GRAIN);
                for (size_t i = 0; i < w.n; ++i) {
                    if (native(w.x[i], w.y[i], w.limit) != w.expected[i])
                        wfb_fail("mandelbrot: leaf");
                    ++leaves;
                }
                /* The UINT64_MAX poison lives here, where nothing is timed:
                 * an element the form never writes reads UINT64_MAX rather
                 * than inheriting a plausible value. */
                run(&w, form, width, !generated);
                compared += compare(&w);
                ++calls;
                destroy(&w);
            }

    /* One fixture above the split admission floor. Every fixture above is a
     * few thousand points at most, and the linked scheduler splits a loop
     * only from 2 * ceil(split_work / weight) iterations upward, so without
     * this the --par module takes its unsplit path at every width in verify
     * and the chunked path the table times is never the path the oracle
     * checks. The size is derived here from the weight the Makefile read out
     * of this kernel's own emitted module, rounded up to a whole number of
     * callback chunks so every reference form sees full chunks too; no count
     * is stored, and a change of weight moves the fixture rather than
     * silently returning it below the floor. The width the harness passes in
     * is the only thing that decides whether the run splits, and the chunk
     * count it produces is printed below, never chosen. */
    {
        size_t admission = wfb_split_floor(WFB_SPLIT_WEIGHT);
        size_t points = admission ? ((admission + MB_GRAIN - 1) / MB_GRAIN) * MB_GRAIN : 0;
        if (points) {
            Work w = input(points, MB_LIMIT, MB_SHAPE, MB_SEED, MB_GRAIN);
            for (size_t i = 0; i < w.n; ++i) {
                if (native(w.x[i], w.y[i], w.limit) != w.expected[i])
                    wfb_fail("mandelbrot: leaf");
                ++leaves;
            }
            run(&w, form, width, !generated);
            compared += compare(&w);
            ++calls;
            destroy(&w);
        }
        (void)printf("# mandelbrot split fixture: weight=%zu floor=%zu points=%zu chunks=%zu\n",
                     (size_t)WFB_SPLIT_WEIGHT, admission, points,
                     wfb_split_chunks(points, WFB_SPLIT_WEIGHT, width));
    }

    (void)printf("# mandelbrot verify: leaves=%zu calls=%zu compared=%zu\n", leaves, calls,
                 compared);
    return compared;
}

static void mb_finish(const char *form) {
    (void)form;
    if (prepared) {
        destroy(&timed);
        prepared = 0;
    }
}

static const char *const mb_forms[] = {"wf",     "wf-seq", "serial",     "static",
                                       "tbb",    "parlay", "rayon-join", "rayon-iter",
                                       NULL};

const wfb_kernel wfb_this_kernel = {
    "mandelbrot",
    mb_forms,
    mb_prepare,
    mb_call,
    mb_check,
    mb_verify,
    mb_finish,
    "points=98304 limit=256 shape=trailing seed=828219",
    MB_POINTS,
    WFB_SPLIT_WEIGHT,
    /* This kernel implements every form it lists: nothing is absent by
       construction here. */
    NULL,
};
