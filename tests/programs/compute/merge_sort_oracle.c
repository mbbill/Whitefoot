/* Independent qsort oracle and binary-split native comparison for merge_sort.
 * The oracle checks every key, including multiplicity, without reproducing
 * the WF decomposition. Define WFB_SORT_ORACLE for the compiler native test. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void (*sort_entry)(const uint64_t *, uint64_t, uint64_t **, uint64_t *, void **);
/* The owned result is a Box<Array<T>> cell; the adapter hands that cell back
 * as the retained handle and the release row consumes exactly it. */
typedef void (*sort_release)(void *);

static void fail(const char *message) {
    (void)fprintf(stderr, "merge_sort: %s\n", message);
    exit(2);
}
static uint64_t *allocate(size_t count) {
    uint64_t *p = malloc((count ? count : 1) * sizeof(*p));
    if (!p) fail("allocation failed");
    return p;
}
static int order(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}
static void fill(uint64_t *p, size_t n, unsigned distribution) {
    uint64_t state = UINT64_C(0x94d049bb133111eb);
    for (size_t i = 0; i < n; ++i) {
        state ^= state << 13; state ^= state >> 7; state ^= state << 17;
        switch (distribution) {
            case 0: p[i] = state; break;
            case 1: p[i] = i; break;
            case 2: p[i] = UINT64_MAX - i; break;
            case 3: p[i] = UINT64_MAX; break;
            case 4: p[i] = i % 97 ? 7 : state; break;
            default: p[i] = i < n / 2 ? i : n - i; break;
        }
    }
}
static uint64_t *oracle(const uint64_t *p, size_t n) {
    uint64_t *result = allocate(n);
    memcpy(result, p, n * sizeof(*p));
    qsort(result, n, sizeof(*result), order);
    return result;
}
static size_t compare(const uint64_t *actual, const uint64_t *expected, size_t n) {
    for (size_t i = 0; i < n; ++i) if (actual[i] != expected[i]) fail("key or multiplicity mismatch");
    return n;
}
static size_t matrix(sort_entry entry, sort_release release) {
    static const size_t counts[] = {0, 1, 2, 17, 63, 64, 65, 257, 4099, 65537};
    size_t checked = 0;
    for (size_t shape = 0; shape < sizeof(counts) / sizeof(counts[0]); ++shape) {
        size_t n = counts[shape];
        for (unsigned distribution = 0; distribution < 6; ++distribution) {
            uint64_t *input = allocate(n), *original = allocate(n), *result = NULL;
            uint64_t length = UINT64_MAX;
            void *held = NULL;
            fill(input, n, distribution);
            memcpy(original, input, n * sizeof(*input));
            uint64_t *expected = oracle(input, n);
            entry(input, n, &result, &length, &held);
            if (length != n) fail("output length");
            checked += compare(result, expected, n);
            checked += compare(input, original, n);
            release(held);
            free(expected); free(original); free(input);
        }
    }
    return checked;
}

#ifndef WF_ORACLE_NO_MAIN
extern void wf_bench_merge_sort(const uint64_t *, uint64_t, uint64_t **, uint64_t *, void **);
extern void wf_bench_merge_sort_release(void *);
extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
#endif
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    size_t checked = matrix(wf_bench_merge_sort, wf_bench_merge_sort_release);
#ifdef WFB_ORACLE_PARALLEL
    const char *workers = getenv("WF_WORKERS");
    if (workers && atoi(workers) > 1 && !wf__par_pool_active()) fail("oracle did not exercise a worker pool");
    if (workers && atoi(workers) > 1 && wf__par_grants() == 0) {
        (void)fprintf(stderr, "merge_sort: oracle observed no steals\n");
        return 2;
    }
    if (workers && atoi(workers) == 1 && wf__par_grants() != 0) fail("pool-off oracle handed out work");
#endif
    (void)printf("merge_sort oracle PASS: 60 configurations, %zu keys\n", checked);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }

#endif
