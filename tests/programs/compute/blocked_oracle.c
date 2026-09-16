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
        {257, 1}, {257, 16}, {4099, 64}, {8193, 1}, {65537, 1024}, {131089, 65536},
        /* Enough blocks to split and enough work per block to exercise steals
         * on a busy host; the one-word blocks can finish before a lane wakes. */
        {1048593, 64}
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

#ifndef WF_ORACLE_NO_MAIN
extern void ENTRY(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void RELEASE(uint64_t *, uint64_t);

extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
#endif
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    size_t compared = verify_matrix(ENTRY, RELEASE);
#ifdef WFB_ORACLE_PARALLEL
    const char *workers = getenv("WF_WORKERS");
    int pool_active = wf__par_pool_active();
    unsigned long grants = wf__par_grants();
    if (workers && atoi(workers) > 1 && !pool_active) {
        (void)fprintf(stderr, "%s: workers=%s pool_active=%d grants=%lu\n",
                      KERNEL_NAME, workers, pool_active, grants);
        fail("oracle did not exercise a worker pool");
    }
    if (workers && atoi(workers) > 1 && grants == 0) {
        /* Results passed, but this schedule supplied no parallel observation.
         * The compiler test resamples only this distinct outcome. */
        (void)fprintf(stderr, "%s: oracle observed no steals\n", KERNEL_NAME);
        return 2;
    }
    if (workers && atoi(workers) == 1 && grants != 0) fail("pool-off oracle handed out work");
#endif
    (void)printf("%s oracle PASS: compared=%zu\n", KERNEL_NAME, compared);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }

#endif
