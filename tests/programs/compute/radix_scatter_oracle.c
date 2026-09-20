/* Stable binary distribution. The oracle uses two ordered input scans and
 * no chunks. Timed native controls select the same chunk/chain decomposition
 * as WF or a direct count/prefix/scatter algorithm. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { BLOCK = 256 };
typedef void (*scatter_entry)(const uint64_t *, uint64_t, uint32_t, uint64_t **, uint64_t *, void **);
/* The owned result is a Box<Array<T>> cell; the adapter hands that cell back
 * as the retained handle and the release row consumes exactly it. */
typedef void (*scatter_release)(void *);

static _Noreturn void fail(const char *message) {
    (void)fprintf(stderr, "radix_scatter: %s\n", message);
    exit(1);
}
static void *allocate(size_t count, size_t size) {
    void *p = calloc(count ? count : 1, size);
    if (!p) fail("allocation");
    return p;
}
static uint64_t key_at(size_t index, unsigned shape, uint32_t bit) {
    uint64_t value = ((uint64_t)index + 1) * UINT64_C(11400714819323198485);
    value ^= value >> 29;
    uint64_t mask = UINT64_C(1) << bit;
    if (shape == 1) return value & ~mask;
    if (shape == 2) return value | mask;
    if (shape == 3) return index % 97 ? value | mask : value & ~mask;
    return value;
}
static void fill(uint64_t *input, size_t count, unsigned shape, uint32_t bit) {
    for (size_t i = 0; i < count; ++i) input[i] = key_at(i, shape, bit);
}
static uint64_t *oracle(const uint64_t *input, size_t count, uint32_t bit) {
    uint64_t *output = allocate(count, sizeof(*output));
    size_t at = 0;
    for (unsigned digit = 0; digit < 2; ++digit)
        for (size_t i = 0; i < count; ++i)
            if (((input[i] >> bit) & 1) == digit) output[at++] = input[i];
    if (at != count) fail("oracle length");
    return output;
}
static size_t compare(const uint64_t *actual, const uint64_t *expected, size_t count) {
    if (count && !actual) fail("missing output");
    for (size_t i = 0; i < count; ++i) {
        if (actual[i] != expected[i]) {
            (void)fprintf(stderr, "radix_scatter: position=%zu expected=%llu actual=%llu\n",
                          i, (unsigned long long)expected[i], (unsigned long long)actual[i]);
            fail("wrong result or unstable order");
        }
    }
    return count;
}
static size_t configurations;
#ifdef WFB_ORACLE_PARALLEL
static void scatter_input_begin(void);
#endif
static size_t verify_case(scatter_entry entry, scatter_release release, size_t count,
                          unsigned shape, uint32_t bit) {
    uint64_t *input = allocate(count, sizeof(*input));
    fill(input, count, shape, bit);
    uint64_t *expected = oracle(input, count, bit), *actual = NULL, length = UINT64_MAX;
    void *held = NULL;
#ifdef WFB_ORACLE_PARALLEL
    scatter_input_begin();
#endif
    entry(input, count, bit, &actual, &length, &held);
    if (length != count) fail("output length");
    size_t checked = compare(actual, expected, count);
    for (size_t i = 0; i < count; ++i)
        if (input[i] != key_at(i, shape, bit)) fail("input modified");
    release(held);
    free(input);
    free(expected);
    ++configurations;
    return checked;
}
static size_t matrix(scatter_entry entry, scatter_release release) {
    const uint64_t known_input[] = {5, 2, 1, 4, 3, 2};
    const uint64_t known_output[] = {2, 4, 2, 5, 1, 3};
    uint64_t *known = oracle(known_input, 6, 0);
    (void)compare(known, known_output, 6);
    free(known);
    static const size_t sizes[] = {0, 1, 17, 255, 256, 257, 4099, 65537, 131089};
    static const uint32_t bits[] = {0, 7, 63};
    configurations = 0;
    size_t checked = 0;
    for (size_t s = 0; s < sizeof(sizes) / sizeof(sizes[0]); ++s)
        for (unsigned shape = 0; shape < 4; ++shape)
            for (size_t b = 0; b < sizeof(bits) / sizeof(bits[0]); ++b)
                checked += verify_case(entry, release, sizes[s], shape, bits[b]);
    checked += verify_case(entry, release, 1048593, 0, 0);
    return checked;
}

#ifndef WF_ORACLE_NO_MAIN
extern void wf_bench_radix_scatter(const uint64_t *, uint64_t, uint32_t, uint64_t **, uint64_t *, void **);
extern void wf_bench_radix_scatter_release(void *);
extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
#include <stdatomic.h>
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
extern void wf_test_worker_schedule_begin(void);
extern void wf_test_worker_schedule_end(void);
static unsigned long pack_before, pack_grants;
static unsigned long input_before, input_grants;
static size_t pack_calls;
static _Thread_local unsigned char thread_marker;
static const unsigned char *input_caller, *pack_caller;
static _Atomic uint64_t helper_input_words, helper_output_words;
static void scatter_input_begin(void) {
    wf_test_worker_schedule_begin();
    input_before = wf__par_grants();
    input_caller = &thread_marker;
}
void wf_scatter_partition_done(uint64_t words) {
    if (input_caller && words && input_caller != &thread_marker)
        atomic_fetch_add_explicit(&helper_input_words, words, memory_order_relaxed);
}
/* Only the correctness module wraps the outer pack entry. All earlier maps
 * have joined there, so resetting the schedule separates input partitioning
 * from output packing. Completed nonempty work is observed in both phases;
 * no instrumentation is added to the recorded timing image. */
void wf_scatter_pack_begin(void) {
    input_grants += wf__par_grants() - input_before;
    input_caller = NULL;
    wf_test_worker_schedule_begin();
    pack_before = wf__par_grants();
    pack_caller = &thread_marker;
    ++pack_calls;
}
void wf_scatter_copy_done(uint64_t words) {
    if (pack_caller && words && pack_caller != &thread_marker)
        atomic_fetch_add_explicit(&helper_output_words, words, memory_order_relaxed);
}
void wf_scatter_pack_end(void) {
    wf_test_worker_schedule_end();
    pack_grants += wf__par_grants() - pack_before;
    pack_caller = NULL;
}
#endif
int wf__main_body(int argc, char **argv) {
    (void)argc;
    (void)argv;
    size_t checked = matrix(wf_bench_radix_scatter, wf_bench_radix_scatter_release);
#ifdef WFB_ORACLE_PARALLEL
    const char *workers = getenv("WF_WORKERS");
    if (workers && atoi(workers) > 1) {
        if (!wf__par_pool_active()) fail("oracle did not exercise a worker pool");
        if (pack_calls != configurations) fail("packing attribution entry was not exercised");
        if (input_grants == 0 || atomic_load(&helper_input_words) == 0)
            fail("oracle observed no nonempty helper input partition");
        if (pack_grants == 0 || atomic_load(&helper_output_words) == 0) {
            /* A qualifying observation needs nonempty helper output work,
             * not merely an earlier count/partition task. */
            (void)fprintf(stderr, "radix_scatter: oracle observed no steals\n");
            return 2;
        }
    }
    if (workers && atoi(workers) == 1 && wf__par_grants() != 0)
        fail("pool-off oracle handed out work");
    (void)printf("radix_scatter packing: %zu calls, %lu steals, %llu helper output words\n",
                 pack_calls, pack_grants, (unsigned long long)atomic_load(&helper_output_words));
    (void)printf("radix_scatter partition: %lu steals, %llu helper input words\n",
                 input_grants, (unsigned long long)atomic_load(&helper_input_words));
#endif
    (void)printf("radix_scatter oracle PASS: %zu configurations, %zu values\n", configurations, checked);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
#endif
