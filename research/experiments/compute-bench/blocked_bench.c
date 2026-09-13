/* The prefix and histogram consumers share input generation, shape checks,
 * and the native oracle entry. Each image selects one kernel at compile time;
 * WFB_BLOCKED_ORACLE builds the same oracle without scheduler dependencies.
 * Independent one-pass oracles do not reproduce the blocked WF algorithm. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(WFB_PREFIX)
#define KERNEL_NAME "prefix"
#define ENTRY wf_bench_prefix
#define RELEASE wf_bench_prefix_release
#define PAR_ENTRY wf_bench_prefix_par
#define PAR_RELEASE wf_bench_prefix_par_release
#define SEQ_ENTRY wf_bench_prefix_seq
#define SEQ_RELEASE wf_bench_prefix_seq_release
#elif defined(WFB_HISTOGRAM)
#define KERNEL_NAME "histogram"
#define ENTRY wf_bench_histogram
#define RELEASE wf_bench_histogram_release
#define PAR_ENTRY wf_bench_histogram_par
#define PAR_RELEASE wf_bench_histogram_par_release
#define SEQ_ENTRY wf_bench_histogram_seq
#define SEQ_RELEASE wf_bench_histogram_seq_release
#else
#error "Select WFB_PREFIX or WFB_HISTOGRAM"
#endif

typedef void (*blocked_entry)(const uint64_t *, uint64_t, uint64_t, uint64_t,
                              uint64_t **, uint64_t *);
typedef void (*blocked_release)(uint64_t *, uint64_t);

static _Noreturn void fail(const char *message) {
    (void)fprintf(stderr, "%s: %s\n", KERNEL_NAME, message);
    exit(1);
}

static uint64_t *words_new(size_t count) {
    uint64_t *words = calloc(count ? count : 1, sizeof(*words));
    if (!words) fail("allocation");
    return words;
}

static uint64_t key_at(size_t index, unsigned shape) {
    uint64_t i = (uint64_t)index;
    if (shape == 1) return UINT64_MAX;
    if (shape == 2) return index % 97 ? 7 : i;
    if (shape == 3) return UINT64_MAX - i;
    uint64_t mixed = (i + 1) * UINT64_C(11400714819323198485);
    return mixed ^ (mixed >> 29);
}

static size_t output_count(size_t count, size_t buckets) {
#ifdef WFB_PREFIX
    (void)buckets;
    return count;
#else
    (void)count;
    return buckets;
#endif
}

static uint64_t *oracle(const uint64_t *input, size_t count, size_t buckets) {
    uint64_t *output = words_new(output_count(count, buckets));
#ifdef WFB_PREFIX
    uint64_t total = 0;
    for (size_t i = 0; i < count; ++i) {
        output[i] = total;
        total += input[i];
    }
#else
    for (size_t i = 0; i < count; ++i) ++output[input[i] % buckets];
#endif
    return output;
}

static size_t compare(const uint64_t *expected, const uint64_t *actual, size_t count) {
    if (count && !actual) fail("missing output");
    for (size_t i = 0; i < count; ++i) {
        if (expected[i] != actual[i]) {
            (void)fprintf(stderr, "%s: position=%zu expected=%llu actual=%llu\n",
                          KERNEL_NAME, i, (unsigned long long)expected[i],
                          (unsigned long long)actual[i]);
            fail("wrong result");
        }
    }
    return count;
}

static size_t verify_matrix(blocked_entry entry, blocked_release release) {
    static const size_t shapes[][2] = {
        {0, 1}, {0, 64}, {1, 1}, {1, 3}, {2, 1}, {17, 3}, {17, 64},
        {257, 1}, {257, 16}, {4099, 64}, {65537, 1024}, {131089, 65536}
    };
#ifdef WFB_PREFIX
    static const size_t bucket_counts[] = {1};
#else
    static const size_t bucket_counts[] = {1, 3, 17, 256};
#endif
    const uint64_t known_input[] = {1, 2, 1, 5};
#ifdef WFB_PREFIX
    const uint64_t known_output[] = {0, 1, 3, 4};
    const size_t known_length = 4;
#else
    const uint64_t known_output[] = {0, 2, 2};
    const size_t known_length = 3;
#endif
    uint64_t *known = oracle(known_input, 4, 3);
    (void)compare(known_output, known, known_length);
    free(known);
    size_t compared = 0;
    for (size_t s = 0; s < sizeof(shapes) / sizeof(shapes[0]); ++s) {
        size_t count = shapes[s][0], block_size = shapes[s][1];
        uint64_t *input = words_new(count);
        for (unsigned distribution = 0; distribution < 4; ++distribution) {
            for (size_t i = 0; i < count; ++i) input[i] = key_at(i, distribution);
            for (size_t b = 0; b < sizeof(bucket_counts) / sizeof(bucket_counts[0]); ++b) {
                size_t buckets = bucket_counts[b], expected_length = output_count(count, buckets);
                uint64_t *expected = oracle(input, count, buckets), *actual = NULL;
                uint64_t length = UINT64_MAX;
                entry(input, count, block_size, buckets, &actual, &length);
                if (length != expected_length) fail("wrong result length");
                compared += compare(expected, actual, expected_length);
                for (size_t i = 0; i < count; ++i)
                    if (input[i] != key_at(i, distribution)) fail("input modified");
                release(actual, length);
                free(expected);
            }
        }
        free(input);
    }
    return compared;
}

#ifdef WFB_BLOCKED_ORACLE
extern void ENTRY(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void RELEASE(uint64_t *, uint64_t);

int main(void) {
    size_t compared = verify_matrix(ENTRY, RELEASE);
    (void)printf("%s oracle PASS: compared=%zu\n", KERNEL_NAME, compared);
    return 0;
}
#else
#include "backend.h"
#include "harness.h"

extern void PAR_ENTRY(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void SEQ_ENTRY(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void PAR_RELEASE(uint64_t *, uint64_t);
extern void SEQ_RELEASE(uint64_t *, uint64_t);

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
static blocked_release release_output;

static void native_release(uint64_t *words, uint64_t count) {
    (void)count;
    free(words);
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
        entry(input, input_count, block_size, buckets, &output, &length);
        if (length != count) fail("generated result length");
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        if (!backend || !backend->map) fail("unknown reference");
        output = native_run(input, input_count, block_size, buckets, backend, workers);
        release_output = native_release;
    }
    return count;
}

static size_t check(void) {
    size_t count = output_count(input_count, buckets);
    size_t compared = compare(expected, output, count);
    for (size_t i = 0; i < input_count; ++i)
        if (input[i] != key_at(i, distribution)) fail("input modified");
    release_output(output, count);
    output = NULL;
    return compared;
}

static const wfb_backend *verify_backend;
static unsigned verify_workers;

static void native_entry(const uint64_t *values, uint64_t count, uint64_t block,
                          uint64_t bins, uint64_t **out, uint64_t *length) {
    *out = native_run(values, count, block, bins, verify_backend, verify_workers);
    *length = output_count(count, bins);
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
#endif
