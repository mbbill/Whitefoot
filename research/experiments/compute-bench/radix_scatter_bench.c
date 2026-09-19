/* Reuse the formally owned result oracle; timing/comparison forms stay here. */
#define WF_ORACLE_NO_MAIN
#include "../../../tests/programs/compute/radix_scatter_oracle.c"

#include "backend.h"
#include "harness.h"
extern void wf_bench_radix_scatter_par(const uint64_t *, uint64_t, uint32_t, uint64_t **, uint64_t *);
extern void wf_bench_radix_scatter_seq(const uint64_t *, uint64_t, uint32_t, uint64_t **, uint64_t *);
extern void wf_bench_radix_scatter_par_release(uint64_t *, uint64_t);
extern void wf_bench_radix_scatter_seq_release(uint64_t *, uint64_t);

static const wfb_backend *backend;
static unsigned width, frontier;
static int direct;
typedef struct { uint64_t low[BLOCK], high[BLOCK]; size_t low_count, high_count; } chunk;
typedef struct {
    const uint64_t *input;
    size_t count;
    uint32_t bit;
    chunk *chunks;
    size_t *low_offsets, *high_offsets;
    uint64_t *output;
} scatter_job;
static void partition_chunk(void *context, size_t block) {
    scatter_job *job = context;
    size_t first = block * BLOCK, end = first + BLOCK;
    if (end > job->count) end = job->count;
    chunk *part = &job->chunks[block];
    for (size_t i = first; i < end; ++i) {
        uint64_t value = job->input[i];
        if ((value >> job->bit) & 1) part->high[part->high_count++] = value;
        else part->low[part->low_count++] = value;
    }
}
typedef struct { const uint64_t *source; uint64_t *output; size_t count; } copy_job;
static void copy_task(void *context) {
    copy_job *job = context;
    for (size_t i = 0; i < job->count; ++i) job->output[i] = job->source[i];
}
typedef struct {
    const chunk *chunks;
    size_t remaining;
    uint64_t *low, *high;
    unsigned budget;
} pack_job;
typedef struct { copy_job high; pack_job tail; } continuation_job;
static void pack_task(void *context);
static void continuation_task(void *context) {
    continuation_job *job = context;
    if (job->tail.budget && width > 1)
        backend->fork2(backend, width, copy_task, &job->high, pack_task, &job->tail);
    else { copy_task(&job->high); pack_task(&job->tail); }
}
static void pack_task(void *context) {
    pack_job *job = context;
    if (!job->remaining) return;
    const chunk *part = job->chunks;
    unsigned next = job->budget ? job->budget - 1 : 0;
    copy_job low = {part->low, job->low, part->low_count};
    continuation_job rest = {
        {part->high, job->high, part->high_count},
        {part + 1, job->remaining - 1, job->low + part->low_count,
         job->high + part->high_count, next}
    };
    if (job->budget && width > 1)
        backend->fork2(backend, width, copy_task, &low, continuation_task, &rest);
    else { copy_task(&low); continuation_task(&rest); }
}
static void count_chunk(void *context, size_t block) {
    scatter_job *job = context;
    size_t first = block * BLOCK, end = first + BLOCK, low = 0;
    if (end > job->count) end = job->count;
    for (size_t i = first; i < end; ++i) low += ((job->input[i] >> job->bit) & 1) == 0;
    job->low_offsets[block] = low;
    job->high_offsets[block] = end - first - low;
}
static void scatter_chunk(void *context, size_t block) {
    scatter_job *job = context;
    size_t first = block * BLOCK, end = first + BLOCK;
    if (end > job->count) end = job->count;
    size_t low = job->low_offsets[block], high = job->high_offsets[block];
    for (size_t i = first; i < end; ++i) {
        uint64_t value = job->input[i];
        if ((value >> job->bit) & 1) job->output[high++] = value;
        else job->output[low++] = value;
    }
}
static void native_release(uint64_t *data, uint64_t length) { (void)length; free(data); }
static void native_entry(const uint64_t *input, uint64_t n, uint32_t bit,
                          uint64_t **output, uint64_t *length) {
    if (backend == &wfb_backend_serial) {
        *output = oracle(input, n, bit);
        *length = n;
        return;
    }
    size_t blocks = (size_t)n / BLOCK + 1, capacity = blocks * BLOCK;
    scatter_job job = {input, n, bit, NULL, NULL, NULL, NULL};
    if (direct) {
        job.low_offsets = allocate(blocks, sizeof(size_t));
        job.high_offsets = allocate(blocks, sizeof(size_t));
        backend->map(backend, width, blocks, count_chunk, &job);
        size_t lows = 0, highs = 0;
        for (size_t b = 0; b < blocks; ++b) {
            size_t low = job.low_offsets[b], high = job.high_offsets[b];
            job.low_offsets[b] = lows;
            job.high_offsets[b] = highs;
            lows += low; highs += high;
        }
        for (size_t b = 0; b < blocks; ++b) job.high_offsets[b] += lows;
        job.output = allocate(n, sizeof(uint64_t));
        backend->map(backend, width, blocks, scatter_chunk, &job);
        free(job.low_offsets); free(job.high_offsets);
        *output = job.output;
        *length = lows + highs;
    } else {
        job.chunks = allocate(blocks, sizeof(chunk));
        backend->map(backend, width, blocks, partition_chunk, &job);
        size_t lows = 0, highs = 0;
        for (size_t b = 0; b < blocks; ++b) {
            lows += job.chunks[b].low_count;
            highs += job.chunks[b].high_count;
        }
        uint64_t *low = allocate(capacity, sizeof(uint64_t));
        uint64_t *high = allocate(capacity, sizeof(uint64_t));
        pack_job pack = {job.chunks, blocks, low, high, frontier};
        pack_task(&pack);
        uint64_t *result = allocate(lows + highs, sizeof(uint64_t));
        copy_job low_copy = {low, result, lows}, high_copy = {high, result + lows, highs};
        backend->fork2(backend, width, copy_task, &low_copy, copy_task, &high_copy);
        free(job.chunks); free(low); free(high);
        *output = result;
        *length = lows + highs;
    }
}
static size_t count = 1048593;
static unsigned shape;
static uint32_t selected_bit;
static uint64_t *input, *expected, *output;
static scatter_release release_output;
static char workload[220];
static void select_backend(const char *form, unsigned workers) {
    backend = wfb_backend_named(form);
    if (!backend || (backend != &wfb_backend_serial && (!backend->map || (!direct && !backend->fork2))))
        fail("missing native decomposition backend");
    width = workers;
    frontier = 0;
    for (unsigned n = 64 * workers; n > 1; n >>= 1) ++frontier;
}
static void prepare(unsigned workers) {
    (void)workers;
    const char *grid = getenv("WFB_SCATTER_GRID"), *algorithm = getenv("WFB_SCATTER_NATIVE");
    if (grid && !strcmp(grid, "small")) count = 257;
    else if (grid && !strcmp(grid, "skew")) shape = 3;
    else if (grid && !strcmp(grid, "low")) shape = 1;
    else if (grid && !strcmp(grid, "high")) { shape = 2; selected_bit = 63; }
    else if (grid && strcmp(grid, "large")) fail("WFB_SCATTER_GRID must be large, small, skew, low or high");
    if (algorithm && !strcmp(algorithm, "direct")) direct = 1;
    else if (algorithm && strcmp(algorithm, "chain")) fail("WFB_SCATTER_NATIVE must be chain or direct");
    input = allocate(count, sizeof(uint64_t));
    fill(input, count, shape, selected_bit);
    expected = oracle(input, count, selected_bit);
    (void)snprintf(workload, sizeof(workload),
                   "keys=%zu shape=%u bit=%u block=%u native=%s wf_workspace=4p+n+stack",
                   count, shape, selected_bit, BLOCK, direct ? "direct" : "chain");
}
static size_t call(const char *form, unsigned workers) {
    uint64_t length = UINT64_MAX;
    if (!strcmp(form, "wf")) {
        wf_bench_radix_scatter_par(input, count, selected_bit, &output, &length);
        release_output = wf_bench_radix_scatter_par_release;
    } else if (!strcmp(form, "wf-seq")) {
        wf_bench_radix_scatter_seq(input, count, selected_bit, &output, &length);
        release_output = wf_bench_radix_scatter_seq_release;
    } else {
        select_backend(form, workers);
        native_entry(input, count, selected_bit, &output, &length);
        release_output = native_release;
    }
    if (length != count) fail("timed output length");
    return count;
}
static size_t check(void) {
    size_t checked = compare(output, expected, count);
    for (size_t i = 0; i < count; ++i)
        if (input[i] != key_at(i, shape, selected_bit)) fail("timed input modified");
    release_output(output, count); output = NULL;
    return checked;
}
static size_t verify(const char *form, unsigned workers) {
    if (!strcmp(form, "wf")) return matrix(wf_bench_radix_scatter_par, wf_bench_radix_scatter_par_release);
    if (!strcmp(form, "wf-seq")) return matrix(wf_bench_radix_scatter_seq, wf_bench_radix_scatter_seq_release);
    select_backend(form, workers);
    return matrix(native_entry, native_release);
}
static void finish(const char *form) { (void)form; free(input); free(expected); input = expected = NULL; }
/* The chain needs fork2 as well as map; the static map-only backend cannot
 * execute this decomposition. Keep both native algorithms on the same forms. */
static const char *const forms[] = {"wf", "wf-seq", "serial", "tbb", "parlay", "rayon-join", NULL};
const wfb_kernel wfb_this_kernel = {
    "radix_scatter", forms, prepare, call, check, verify, finish, workload, 0, 0, NULL
};
