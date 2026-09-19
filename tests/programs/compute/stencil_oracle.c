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

typedef void (*stencil_entry)(uint64_t, uint64_t, uint64_t, double **, uint64_t *, void **);
/* The owned result is a Box<Array<T>> cell; the adapter hands that cell back
 * as the retained handle and the release row consumes exactly it. */
typedef void (*stencil_release)(void *);

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
        {3, 3}, {3, 17}, {19, 3}, {5, 7}, {31, 18}, {64, 65}, {257, 129}, {257, 2049}
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
    double *known_output = NULL;
    uint64_t known_length = UINT64_MAX;
    void *known_held = NULL;
    entry(5, 5, 2, &known_output, &known_length, &known_held);
    if (known_length != 25) fail("known output length");
    compared += compare(known, known_output, 25);
    release(known_held);
    free(known);
    for (size_t shape = 0; shape < sizeof(shapes) / sizeof(shapes[0]); ++shape) {
        size_t width = shapes[shape][0], height = shapes[shape][1];
        for (size_t round = 0; round < sizeof(rounds) / sizeof(rounds[0]); ++round) {
            double *expected = oracle(width, height, rounds[round]);
            double *output = NULL;
            uint64_t length = UINT64_MAX;
            void *held = NULL;
            entry(width, height, rounds[round], &output, &length, &held);
            if (length != width * height) fail("wrong output length");
            compared += compare(expected, output, (size_t)length);
            release(held);
            free(expected);
        }
    }
    return compared;
}

#if !defined(WF_ORACLE_NO_MAIN) || defined(WF_ORACLE_PERFORMANCE)
extern void wf_bench_stencil(uint64_t, uint64_t, uint64_t, double **, uint64_t *, void **);
extern void wf_bench_stencil_release(void *);
#endif

#ifndef WF_ORACLE_NO_MAIN
extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
#endif


int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    size_t compared = verify_matrix(wf_bench_stencil, wf_bench_stencil_release);
#ifdef WFB_ORACLE_PARALLEL
    const char *workers = getenv("WF_WORKERS");
    if (workers && atoi(workers) > 1 && !wf__par_pool_active()) fail("oracle did not exercise a worker pool");
    if (workers && atoi(workers) > 1 && wf__par_grants() == 0) {
        (void)fprintf(stderr, "stencil: oracle observed no steals\n");
        return 2;
    }
    if (workers && atoi(workers) == 1 && wf__par_grants() != 0) fail("pool-off oracle handed out work");
#endif
    (void)printf("stencil oracle PASS: compared=%zu\n", compared);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }

#endif

#ifdef WF_ORACLE_PERFORMANCE
/* The separate paired workflow owns timing. Ordinary verification does not
 * allocate this 4M-cell fixture or compute its sixteen-step reference. */
const char *const wf_oracle_name = "stencil";
const char *const wf_oracle_fixture = "width=1024 height=4096 steps=16 initial=squared-position";
static double *timed_expected, *timed_output;
static void *timed_held;
size_t wf_oracle_verify(void) {
    return verify_matrix(wf_bench_stencil, wf_bench_stencil_release);
}
void wf_oracle_prepare(void) { timed_expected = oracle(1024, 4096, 16); }
size_t wf_oracle_call(void) {
    uint64_t length = UINT64_MAX;
    wf_bench_stencil(1024, 4096, 16, &timed_output, &length, &timed_held);
    if (length != 1024 * 4096) fail("timed output extent");
    return (size_t)length;
}
size_t wf_oracle_check(void) {
    size_t compared = compare(timed_expected, timed_output, 1024 * 4096);
    wf_bench_stencil_release(timed_held);
    timed_output = NULL;
    timed_held = NULL;
    return compared;
}
void wf_oracle_finish(void) { free(timed_expected); timed_expected = NULL; }
#endif
