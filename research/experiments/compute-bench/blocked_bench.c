/* Reuse the formally owned result oracle; timing/comparison forms stay here. */
#define WF_ORACLE_NO_MAIN
#include "../../../tests/programs/compute/blocked_oracle.c"

#include "backend.h"
#include "harness.h"

extern void PAR_ENTRY(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *, void **);
extern void SEQ_ENTRY(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *, void **);
extern void PAR_RELEASE(void *);
extern void SEQ_RELEASE(void *);

typedef struct {
    const uint64_t *input;
    uint64_t *workspace, *output;
    size_t block_size, buckets, rows;
} BlockWork;

static void first_pass(void *context, size_t block) {
    BlockWork *work = context;
    size_t first = block * work->block_size, end = first + work->block_size;
#ifdef WFB_PREFIX
    uint64_t total = 0;
    for (size_t i = first; i < end; ++i) total += work->input[i];
    work->workspace[block] = total;
#else
    uint64_t *counters = work->workspace + block * work->buckets;
    for (size_t i = first; i < end; ++i) ++counters[work->input[i] % work->buckets];
#endif
}

static void last_pass(void *context, size_t index) {
    BlockWork *work = context;
#ifdef WFB_PREFIX
    size_t first = index * work->block_size, end = first + work->block_size;
    uint64_t total = work->workspace[index];
    for (size_t i = first; i < end; ++i) {
        work->output[i] = total;
        total += work->input[i];
    }
#else
    uint64_t total = 0;
    for (size_t row = 0; row < work->rows; ++row)
        total += work->workspace[row * work->buckets + index];
    work->output[index] = total;
#endif
}

static uint64_t *native_run(const uint64_t *input, size_t count, size_t block_size,
                            size_t buckets, const wfb_backend *backend, unsigned workers) {
    /* A useful native sequential reference is the direct one-pass algorithm.
     * Parallel references use the same complete blocks and tail as WF. */
    if (backend == &wfb_backend_serial) return oracle(input, count, buckets);
    size_t blocks = count / block_size, covered = blocks * block_size;
    uint64_t *output = words_new(output_count(count, buckets));
#ifdef WFB_PREFIX
    uint64_t *workspace = words_new(blocks);
#else
    uint64_t *workspace = words_new((blocks + 1) * buckets);
#endif
    BlockWork work = {input, workspace, output, block_size, buckets, blocks + 1};
    backend->map(backend, workers, blocks, first_pass, &work);
#ifdef WFB_PREFIX
    uint64_t total = 0;
    for (size_t block = 0; block < blocks; ++block) {
        uint64_t value = workspace[block];
        workspace[block] = total;
        total += value;
    }
    backend->map(backend, workers, blocks, last_pass, &work);
    for (size_t i = covered; i < count; ++i) {
        output[i] = total;
        total += input[i];
    }
#else
    uint64_t *tail = workspace + blocks * buckets;
    for (size_t i = covered; i < count; ++i) ++tail[input[i] % buckets];
    backend->map(backend, workers, buckets, last_pass, &work);
#endif
    free(workspace);
    return output;
}

static size_t input_count = 4194321, block_size = 4096, buckets = 256;
static unsigned distribution;
static char workload[112];
static uint64_t *input, *expected, *output;
static void *output_held;
static blocked_release release_output;

static void native_release(void *held) {
    free(held);
}

static void prepare(unsigned workers) {
    (void)workers;
    const char *grid = getenv("WFB_BLOCKED_GRID");
    if (grid && !strcmp(grid, "small")) { input_count = 17; block_size = 3; }
    else if (grid && !strcmp(grid, "fine")) block_size = 256;
    else if (grid && !strcmp(grid, "coarse")) block_size = 65536;
    else if (grid && !strcmp(grid, "skew")) distribution = 2;
    else if (grid && strcmp(grid, "large")) fail("WFB_BLOCKED_GRID must be large, small, fine, coarse or skew");
    (void)snprintf(workload, sizeof(workload), "count=%zu block_size=%zu buckets=%zu distribution=%u",
                   input_count, block_size, buckets, distribution);
    input = words_new(input_count);
    for (size_t i = 0; i < input_count; ++i) input[i] = key_at(i, distribution);
    expected = oracle(input, input_count, buckets);
}

static size_t call(const char *form, unsigned workers) {
    size_t count = output_count(input_count, buckets);
    if (!strcmp(form, "wf") || !strcmp(form, "wf-seq")) {
        int sequential = !strcmp(form, "wf-seq");
        blocked_entry entry = sequential ? SEQ_ENTRY : PAR_ENTRY;
        release_output = sequential ? SEQ_RELEASE : PAR_RELEASE;
        uint64_t length = UINT64_MAX;
        entry(input, input_count, block_size, buckets, &output, &length, &output_held);
        if (length != count) fail("generated result length");
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        if (!backend || !backend->map) fail("unknown reference");
        output = native_run(input, input_count, block_size, buckets, backend, workers);
        output_held = output;
        release_output = native_release;
    }
    return count;
}

static size_t check(void) {
    size_t count = output_count(input_count, buckets);
    size_t compared = compare(expected, output, count);
    for (size_t i = 0; i < input_count; ++i)
        if (input[i] != key_at(i, distribution)) fail("input modified");
    release_output(output_held);
    output = NULL;
    output_held = NULL;
    return compared;
}

static const wfb_backend *verify_backend;
static unsigned verify_workers;

static void native_entry(const uint64_t *values, uint64_t count, uint64_t block,
                          uint64_t bins, uint64_t **out, uint64_t *length, void **held) {
    *out = native_run(values, count, block, bins, verify_backend, verify_workers);
    *length = output_count(count, bins);
    *held = *out;
}

static size_t verify(const char *form, unsigned workers) {
    if (!strcmp(form, "wf")) return verify_matrix(PAR_ENTRY, PAR_RELEASE);
    if (!strcmp(form, "wf-seq")) return verify_matrix(SEQ_ENTRY, SEQ_RELEASE);
    verify_backend = wfb_backend_named(form);
    verify_workers = workers;
    if (!verify_backend || !verify_backend->map) fail("unknown verify reference");
    return verify_matrix(native_entry, native_release);
}

static void finish(const char *form) {
    (void)form;
    free(input);
    free(expected);
    input = expected = NULL;
}

static const char *const forms[] = {
    "wf", "wf-seq", "serial", "static", "tbb", "parlay", "rayon-join", "rayon-iter", NULL
};

const wfb_kernel wfb_this_kernel = {
    KERNEL_NAME, forms, prepare, call, check, verify, finish, workload, 0, 0, NULL
};
