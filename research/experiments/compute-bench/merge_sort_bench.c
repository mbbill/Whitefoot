/* Reuse the formally owned result oracle; timing/comparison forms stay here. */
#define WF_ORACLE_NO_MAIN
#include "../../../tests/programs/compute/merge_sort_oracle.c"

#include "backend.h"
#include "harness.h"
static void native_release(void *held) { free(held); }
extern void wf_bench_merge_sort_par(const uint64_t *, uint64_t, uint64_t **, uint64_t *, void **);
extern void wf_bench_merge_sort_seq(const uint64_t *, uint64_t, uint64_t **, uint64_t *, void **);
extern void wf_bench_merge_sort_par_release(void *);
extern void wf_bench_merge_sort_seq_release(void *);
static const wfb_backend *backend;
static unsigned width;
static unsigned frontier;

typedef struct {
    const uint64_t *a, *b;
    uint64_t *out;
    size_t n, m;
    unsigned budget;
} merge_job;
static void merge_task(void *context) {
    merge_job *job = context;
    const uint64_t *a = job->a, *b = job->b;
    uint64_t *out = job->out;
    size_t n = job->n, m = job->m;
    if (!n) { memcpy(out, b, m * sizeof(*b)); return; }
    if (!m) { memcpy(out, a, n * sizeof(*a)); return; }
    if (n + m <= 64) {
        size_t i = 0, j = 0;
        for (size_t at = 0; at < n + m; ++at) {
            if (i < n && (j == m || a[i] <= b[j])) out[at] = a[i++];
            else out[at] = b[j++];
        }
        return;
    }
    if (n < m) {
        merge_job swapped = {b, a, out, m, n, job->budget};
        merge_task(&swapped);
        return;
    }
    size_t middle = (n - 1) / 2, lower = 0, upper = m;
    while (lower < upper) {
        size_t at = lower + (upper - 1 - lower) / 2;
        if (b[at] < a[middle]) lower = at + 1;
        else upper = at;
    }
    unsigned next = job->budget ? job->budget - 1 : 0;
    merge_job left = {a, b, out, middle, lower, next};
    merge_job right = {a + middle, b + lower, out + middle + lower, n - middle, m - lower, next};
    if (job->budget && width > 1) backend->fork2(backend, width, merge_task, &left, merge_task, &right);
    else { merge_task(&left); merge_task(&right); }
}
typedef struct { uint64_t *input, *output; size_t n; unsigned budget; } sort_job;
static void sort_task(void *context) {
    sort_job *job = context;
    if (job->n <= 1) {
        if (job->n) job->output[0] = job->input[0];
        return;
    }
    size_t middle = job->n / 2;
    unsigned next = job->budget ? job->budget - 1 : 0;
    sort_job left = {job->output, job->input, middle, next};
    sort_job right = {job->output + middle, job->input + middle, job->n - middle, next};
    if (job->budget && width > 1) backend->fork2(backend, width, sort_task, &left, sort_task, &right);
    else { sort_task(&left); sort_task(&right); }
    merge_job merge = {job->input, job->input + middle, job->output, middle, job->n - middle,
                       job->budget ? frontier : 0};
    merge_task(&merge);
}
static void native_entry(const uint64_t *input, uint64_t n, uint64_t **out, uint64_t *length, void **held) {
    if (backend == &wfb_backend_serial) {
        *out = oracle(input, n);
    } else {
        uint64_t *scratch = allocate(n), *result = allocate(n);
        memcpy(scratch, input, n * sizeof(*input));
        memcpy(result, input, n * sizeof(*input));
        sort_job job = {scratch, result, n, frontier};
        sort_task(&job);
        free(scratch);
        *out = result;
    }
    *length = n;
    *held = *out;
}
static size_t count = 1048593;
static unsigned distribution;
static uint64_t *input, *expected, *output;
static void *output_held;
static sort_release release_output;
static char workload[180];
static void select_backend(const char *form, unsigned workers) {
    backend = wfb_backend_named(form);
    if (!backend || (backend != &wfb_backend_serial && !backend->fork2)) fail("missing recursive backend");
    width = workers;
    frontier = 0;
    for (unsigned n = 64 * workers; n > 1; n >>= 1) ++frontier;
}
static void prepare(unsigned workers) {
    (void)workers;
    const char *grid = getenv("WFB_SORT_GRID");
    if (grid && !strcmp(grid, "small")) count = 257;
    else if (grid && !strcmp(grid, "skew")) distribution = 4;
    else if (grid && !strcmp(grid, "equal")) distribution = 3;
    else if (grid && strcmp(grid, "large")) fail("WFB_SORT_GRID must be large, small, skew or equal");
    input = allocate(count); fill(input, count, distribution); expected = oracle(input, count);
    (void)snprintf(workload, sizeof(workload), "keys=%zu distribution=%u leaf=64 workspace=2n", count, distribution);
}
static size_t call(const char *form, unsigned workers) {
    uint64_t length = UINT64_MAX;
    if (!strcmp(form, "wf")) {
        wf_bench_merge_sort_par(input, count, &output, &length, &output_held);
        release_output = wf_bench_merge_sort_par_release;
    } else if (!strcmp(form, "wf-seq")) {
        wf_bench_merge_sort_seq(input, count, &output, &length, &output_held);
        release_output = wf_bench_merge_sort_seq_release;
    } else {
        select_backend(form, workers);
        native_entry(input, count, &output, &length, &output_held);
        release_output = native_release;
    }
    if (length != count) fail("timed output length");
    return count;
}
static size_t check(void) {
    size_t checked = compare(output, expected, count);
    release_output(output_held); output = NULL; output_held = NULL;
    return checked;
}
static size_t verify(const char *form, unsigned workers) {
    if (!strcmp(form, "wf")) return matrix(wf_bench_merge_sort_par, wf_bench_merge_sort_par_release);
    if (!strcmp(form, "wf-seq")) return matrix(wf_bench_merge_sort_seq, wf_bench_merge_sort_seq_release);
    select_backend(form, workers);
    return matrix(native_entry, native_release);
}
static void finish(const char *form) { (void)form; free(input); free(expected); input = expected = NULL; }
static const char *const forms[] = {"wf", "wf-seq", "serial", "tbb", "parlay", "rayon-join", NULL};
const wfb_kernel wfb_this_kernel = {
    "merge_sort", forms, prepare, call, check, verify, finish, workload, 0, 0, NULL
};
