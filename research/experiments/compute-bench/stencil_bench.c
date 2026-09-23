/* Reuse the formally owned result oracle; timing/comparison forms stay here. */
#define WF_ORACLE_NO_MAIN
#include "../../../tests/programs/compute/stencil_oracle.c"

#include "backend.h"
#include "harness.h"

extern void wf_bench_stencil_par(uint64_t, uint64_t, uint64_t, double **, uint64_t *, void **);
extern void wf_bench_stencil_seq(uint64_t, uint64_t, uint64_t, double **, uint64_t *, void **);
extern void wf_bench_stencil_par_release(void *);
extern void wf_bench_stencil_seq_release(void *);

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

static size_t dimension(const char *name, size_t fallback) {
    const char *value = getenv(name);
    if (!value) return fallback;
    char *end;
    unsigned long parsed = strtoul(value, &end, 10);
    if (!*value || *end || parsed < 3 || parsed > 4096) fail("stencil dimension must be 3..4096");
    return (size_t)parsed;
}
static double *expected, *output;
static void *output_held;
static stencil_release release_output;

static void native_release(void *held) {
    free(held);
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
    grid_width = dimension("WFB_STENCIL_WIDTH", grid_width);
    grid_height = dimension("WFB_STENCIL_HEIGHT", grid_height);
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
        entry(grid_width, grid_height, grid_steps, &output, &length, &output_held);
        if (length != cells) fail("generated output length");
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        if (!backend || !backend->map) fail("unknown reference");
        output = native_run(grid_width, grid_height, grid_steps, backend, workers);
        output_held = output;
        release_output = native_release;
    }
    return cells;
}

static size_t check(void) {
    size_t compared = compare(expected, output, grid_width * grid_height);
    release_output(output_held);
    output = NULL;
    output_held = NULL;
    return compared;
}

static const wfb_backend *verify_backend;
static unsigned verify_workers;

static void native_entry(uint64_t width, uint64_t height, uint64_t steps,
                          double **out, uint64_t *length, void **held) {
    *out = native_run((size_t)width, (size_t)height, (size_t)steps, verify_backend, verify_workers);
    *length = width * height;
    *held = *out;
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
