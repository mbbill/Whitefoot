/* Serves compute-bench and its compiler-independent native correctness test.
 * The oracle uses column-major storage; WF and timed references use row-major
 * storage and one ordinary helper/callback per row. Both timed paths allocate
 * and zero two grids, initialize every cell, compute and join all steps, and
 * release the inactive grid. Releasing the returned grid is outside timing.
 * WFB_STENCIL_ORACLE selects the same oracle's dependency-free test entry. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void (*stencil_entry)(uint64_t, uint64_t, uint64_t, double **, uint64_t *);
typedef void (*stencil_release)(double *, uint64_t);

static _Noreturn void fail(const char *message) {
    (void)fprintf(stderr, "stencil: %s\n", message);
    exit(1);
}

static double *grid_new(size_t cells) {
    double *grid = malloc(cells * sizeof(*grid));
    if (!grid) fail("grid allocation");
    memset(grid, 0, cells * sizeof(*grid));
    return grid;
}

static double initial(size_t width, size_t x, size_t y) {
    uint64_t position = (uint64_t)(y * width + x + 1);
    uint64_t encoded = UINT64_C(4607182418800017408) + position * position * UINT64_C(256);
    double value;
    memcpy(&value, &encoded, sizeof(value));
    return value;
}

/* Indexing and traversal differ from the row views under test. Floating
 * operations keep the specified vertical/horizontal grouping and rounding. */
static double *oracle(size_t width, size_t height, size_t steps) {
    size_t cells = width * height;
    double *a = grid_new(cells), *b = grid_new(cells);
    for (size_t x = 0; x < width; ++x)
        for (size_t y = 0; y < height; ++y)
            a[x * height + y] = b[x * height + y] = initial(width, x, y);
    for (size_t t = 0; t < steps; ++t) {
        for (size_t x = 1; x + 1 < width; ++x)
            for (size_t y = 1; y + 1 < height; ++y) {
                size_t at = x * height + y;
                double vertical = a[at - 1] + a[at + 1];
                double horizontal = a[at - height] + a[at + height];
                b[at] = (vertical + horizontal) * 0.25;
            }
        double *swap = a;
        a = b;
        b = swap;
    }
    double *result = grid_new(cells);
    for (size_t y = 0; y < height; ++y)
        for (size_t x = 0; x < width; ++x)
            result[y * width + x] = a[x * height + y];
    free(a);
    free(b);
    return result;
}

static size_t compare(const double *expected, const double *actual, size_t cells) {
    if (!actual) fail("missing output");
    for (size_t i = 0; i < cells; ++i) {
        uint64_t want, got;
        memcpy(&want, expected + i, sizeof(want));
        memcpy(&got, actual + i, sizeof(got));
        if (want != got) {
            (void)fprintf(stderr, "stencil: cell=%zu expected=%.17g actual=%.17g\n",
                          i, expected[i], actual[i]);
            fail("wrong output bits");
        }
    }
    return cells;
}

static size_t verify_matrix(stencil_entry entry, stencil_release release) {
    static const size_t shapes[][2] = {
        {3, 3}, {3, 17}, {19, 3}, {5, 7}, {31, 18}, {64, 65}, {257, 129}
    };
    static const size_t rounds[] = {0, 1, 2, 3, 7, 16};
    size_t compared = 0;
    double *known = oracle(5, 5, 2);
    static const size_t positions[] = {0, 6, 8, 12};
    static const uint64_t expected_bits[] = {
        UINT64_C(4607182418800017664), UINT64_C(4607182418800034944),
        UINT64_C(4607182418800043136), UINT64_C(4607182418800067328)
    };
    for (size_t i = 0; i < sizeof(positions) / sizeof(positions[0]); ++i) {
        uint64_t bits;
        memcpy(&bits, known + positions[i], sizeof(bits));
        if (bits != expected_bits[i]) fail("hand-computed oracle witness");
    }
    free(known);
    for (size_t shape = 0; shape < sizeof(shapes) / sizeof(shapes[0]); ++shape) {
        size_t width = shapes[shape][0], height = shapes[shape][1];
        for (size_t round = 0; round < sizeof(rounds) / sizeof(rounds[0]); ++round) {
            double *expected = oracle(width, height, rounds[round]);
            double *output = NULL;
            uint64_t length = UINT64_MAX;
            entry(width, height, rounds[round], &output, &length);
            if (length != width * height) fail("wrong output length");
            compared += compare(expected, output, (size_t)length);
            release(output, length);
            free(expected);
        }
    }
    return compared;
}

#ifdef WFB_STENCIL_ORACLE
extern void wf_bench_stencil(uint64_t, uint64_t, uint64_t, double **, uint64_t *);
extern void wf_bench_stencil_release(double *, uint64_t);

int main(void) {
    size_t compared = verify_matrix(wf_bench_stencil, wf_bench_stencil_release);
    (void)printf("stencil oracle PASS: compared=%zu\n", compared);
    return 0;
}
#else
#include "backend.h"
#include "harness.h"

extern void wf_bench_stencil_par(uint64_t, uint64_t, uint64_t, double **, uint64_t *);
extern void wf_bench_stencil_seq(uint64_t, uint64_t, uint64_t, double **, uint64_t *);
extern void wf_bench_stencil_par_release(double *, uint64_t);
extern void wf_bench_stencil_seq_release(double *, uint64_t);

typedef struct {
    size_t width, height;
    const double *input;
    double *output;
} RowWork;

static void native_row(void *context, size_t row) {
    RowWork *work = context;
    size_t start = (row + 1) * work->width;
    for (size_t x = 1; x + 1 < work->width; ++x) {
        size_t at = start + x;
        double vertical = work->input[at - work->width] + work->input[at + work->width];
        double horizontal = work->input[at - 1] + work->input[at + 1];
        work->output[at] = (vertical + horizontal) * 0.25;
    }
}

/* Native references use one row per scheduler callback, fixed before
 * measurement. Backend scheduling policies are the bundle's existing ones. */
static double *native_run(size_t width, size_t height, size_t steps,
                          const wfb_backend *backend, unsigned workers) {
    size_t cells = width * height;
    double *a = grid_new(cells), *b = grid_new(cells);
    for (size_t y = 0; y < height; ++y)
        for (size_t x = 0; x < width; ++x)
            a[y * width + x] = b[y * width + x] = initial(width, x, y);
    for (size_t t = 0; t < steps; ++t) {
        RowWork work = {width, height, a, b};
        backend->map(backend, workers, height - 2, native_row, &work);
        double *swap = a;
        a = b;
        b = swap;
    }
    free(b);
    return a;
}

static size_t grid_width = 1024, grid_height = 4096, grid_steps = 16;
static char workload[96] = "width=1024 height=4096 steps=16 initial=squared-position";
static double *expected, *output;
static stencil_release release_output;

static void native_release(double *grid, uint64_t length) {
    (void)length;
    free(grid);
}

static void prepare(unsigned workers) {
    (void)workers;
    const char *grid = getenv("WFB_STENCIL_GRID");
    if (grid && !strcmp(grid, "small")) {
        grid_width = 17;
        grid_height = 13;
        grid_steps = 3;
    } else if (grid && !strcmp(grid, "original")) {
        grid_height = 2048;
    } else if (grid && strcmp(grid, "large")) {
        fail("WFB_STENCIL_GRID must be large, original or small");
    }
    (void)snprintf(workload, sizeof(workload), "width=%zu height=%zu steps=%zu initial=squared-position",
                   grid_width, grid_height, grid_steps);
    expected = oracle(grid_width, grid_height, grid_steps);
}

static size_t call(const char *form, unsigned workers) {
    size_t cells = grid_width * grid_height;
    if (!strcmp(form, "wf") || !strcmp(form, "wf-seq")) {
        uint64_t length = UINT64_MAX;
        int sequential = !strcmp(form, "wf-seq");
        stencil_entry entry = sequential ? wf_bench_stencil_seq : wf_bench_stencil_par;
        release_output = sequential ? wf_bench_stencil_seq_release : wf_bench_stencil_par_release;
        entry(grid_width, grid_height, grid_steps, &output, &length);
        if (length != cells) fail("generated output length");
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        if (!backend || !backend->map) fail("unknown reference");
        output = native_run(grid_width, grid_height, grid_steps, backend, workers);
        release_output = native_release;
    }
    return cells;
}

static size_t check(void) {
    size_t compared = compare(expected, output, grid_width * grid_height);
    release_output(output, (uint64_t)compared);
    output = NULL;
    return compared;
}

static const wfb_backend *verify_backend;
static unsigned verify_workers;

static void native_entry(uint64_t width, uint64_t height, uint64_t steps,
                          double **out, uint64_t *length) {
    *out = native_run((size_t)width, (size_t)height, (size_t)steps, verify_backend, verify_workers);
    *length = width * height;
}

static size_t verify(const char *form, unsigned workers) {
    if (!strcmp(form, "wf")) return verify_matrix(wf_bench_stencil_par, wf_bench_stencil_par_release);
    if (!strcmp(form, "wf-seq")) return verify_matrix(wf_bench_stencil_seq, wf_bench_stencil_seq_release);
    verify_backend = wfb_backend_named(form);
    verify_workers = workers;
    if (!verify_backend || !verify_backend->map) fail("unknown verify reference");
    return verify_matrix(native_entry, native_release);
}

static void finish(const char *form) {
    (void)form;
    free(expected);
    expected = NULL;
}

static const char *const forms[] = {
    "wf", "wf-seq", "serial", "static", "tbb", "parlay", "rayon-join", "rayon-iter", NULL
};

const wfb_kernel wfb_this_kernel = {
    "stencil", forms, prepare, call, check, verify, finish, workload,
    /* Initialization and nested row loops have different runtime spans.
     * There is no single chunk count to report; grants remain measured. */
    0, 0, NULL
};
#endif
