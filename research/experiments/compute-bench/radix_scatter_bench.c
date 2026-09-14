/* Stable binary distribution. The oracle uses two ordered input scans and
 * no chunks. Timed native controls select the same chunk/chain decomposition
 * as WF or a direct count/prefix/scatter algorithm. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { BLOCK = 256 };
typedef void (*scatter_entry)(const uint64_t *, uint64_t, uint32_t, uint64_t **, uint64_t *);
typedef void (*scatter_release)(uint64_t *, uint64_t);

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
static size_t verify_case(scatter_entry entry, scatter_release release, size_t count,
                          unsigned shape, uint32_t bit) {
    uint64_t *input = allocate(count, sizeof(*input));
    fill(input, count, shape, bit);
    uint64_t *expected = oracle(input, count, bit), *actual = NULL, length = UINT64_MAX;
    entry(input, count, bit, &actual, &length);
    if (length != count) fail("output length");
    size_t checked = compare(actual, expected, count);
    for (size_t i = 0; i < count; ++i)
        if (input[i] != key_at(i, shape, bit)) fail("input modified");
    release(actual, length);
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

#ifdef WFB_SCATTER_ORACLE
extern void wf_bench_radix_scatter(const uint64_t *, uint64_t, uint32_t, uint64_t **, uint64_t *);
extern void wf_bench_radix_scatter_release(uint64_t *, uint64_t);
extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
#include <stdatomic.h>
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
static unsigned long pack_before, pack_grants;
static size_t pack_calls;
static _Thread_local unsigned char thread_marker;
static const unsigned char *pack_caller;
static _Atomic uint64_t helper_output_words;
/* Only the correctness module wraps the outer pack entry. All earlier maps
 * have joined there, so this delta attributes steals to output packing. No
 * instrumentation is added to the recorded timing image. */
void wf_scatter_pack_begin(void) {
    pack_before = wf__par_grants();
    pack_caller = &thread_marker;
    ++pack_calls;
}
void wf_scatter_copy_done(uint64_t words) {
    if (pack_caller && words && pack_caller != &thread_marker)
        atomic_fetch_add_explicit(&helper_output_words, words, memory_order_relaxed);
}
void wf_scatter_pack_end(void) {
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
        if (pack_grants == 0 || atomic_load(&helper_output_words) == 0) {
            /* The shared sampler's no-observation result. A qualifying
             * observation here needs nonempty helper work as well as a steal. */
            (void)fprintf(stderr, "radix_scatter: oracle observed no steals\n");
            return 2;
        }
    }
    if (workers && atoi(workers) == 1 && wf__par_grants() != 0)
        fail("pool-off oracle handed out work");
    (void)printf("radix_scatter packing: %zu calls, %lu steals, %llu helper output words\n",
                 pack_calls, pack_grants, (unsigned long long)atomic_load(&helper_output_words));
#endif
    (void)printf("radix_scatter oracle PASS: %zu configurations, %zu values\n", configurations, checked);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
#else
#include "backend.h"
#include "harness.h"

#ifdef WFB_SCATTER_PHASES
enum { PHASES = 8, PHASE_CALLS = 256 };
typedef struct { uint64_t start, wall, cpu; unsigned long steals; } phase_sample;
static phase_sample phase_samples[PHASE_CALLS][PHASES];
static const char *const phase_names[PHASES] = {
    "chunk-allocation", "partition", "tally", "digit-allocation",
    "packing", "result-allocation", "final-copy", "release"
};
static const char *phase_form;
static unsigned phase_width;
static size_t phase_calls;
static int phase_next, phase_registered;
static uint64_t phase_wall, phase_cpu;
static unsigned long phase_steals;
static void phase_report(void) {
    if (phase_next != 0) fail("incomplete phase observation");
    (void)fprintf(stderr, "scatter-phases form=%s width=%u calls=%zu cpu_clock=%s\n",
                  phase_form ? phase_form : "none", phase_width, phase_calls,
                  wfb_cpu_clock_name());
    for (size_t call_id = 0; call_id < phase_calls; ++call_id)
        for (size_t phase = 0; phase < PHASES; ++phase) {
            const phase_sample *s = &phase_samples[call_id][phase];
            (void)fprintf(stderr, "scatter-phase\t%s\t%u\t%zu\t%s\t%llu\t%llu\t%llu\t%lu\n",
                          phase_form, phase_width, call_id, phase_names[phase],
                          (unsigned long long)s->start, (unsigned long long)s->wall,
                          (unsigned long long)s->cpu, s->steals);
        }
}
/* Only the diagnostic image imports this callback. Coarse boundaries retain
 * the original joins; no per-element clock or output is on a measured path.
 * CPU includes idle/spinning workers, and allocation page faults are charged
 * to the interval in which the memory is actually touched. */
void wf_scatter_phase(int phase) {
    if (!phase_registered) {
        if (atexit(phase_report) != 0) fail("phase report registration");
        phase_registered = 1;
    }
    if (phase != phase_next || phase_calls == PHASE_CALLS)
        fail("unexpected or excessive phase observation");
    uint64_t wall = wfb_now_ns(), cpu = wfb_cpu_ns();
    unsigned long steals = wf__par_grants();
    if (phase != 0) {
        if (wall < phase_wall || cpu < phase_cpu || steals < phase_steals)
            fail("phase clocks or counters moved backwards");
        phase_samples[phase_calls][phase - 1] =
            (phase_sample){phase_wall, wall - phase_wall, cpu - phase_cpu,
                           steals - phase_steals};
    }
    phase_wall = wall; phase_cpu = cpu; phase_steals = steals;
    if (phase == PHASES) { ++phase_calls; phase_next = 0; }
    else phase_next = phase + 1;
}
#define SCATTER_PHASE(n) wf_scatter_phase(n)
#else
#define SCATTER_PHASE(n) ((void)0)
#endif

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
        SCATTER_PHASE(0);
        job.chunks = allocate(blocks, sizeof(chunk));
        SCATTER_PHASE(1);
        backend->map(backend, width, blocks, partition_chunk, &job);
        SCATTER_PHASE(2);
        size_t lows = 0, highs = 0;
        for (size_t b = 0; b < blocks; ++b) {
            lows += job.chunks[b].low_count;
            highs += job.chunks[b].high_count;
        }
        SCATTER_PHASE(3);
        uint64_t *low = allocate(capacity, sizeof(uint64_t));
        uint64_t *high = allocate(capacity, sizeof(uint64_t));
        pack_job pack = {job.chunks, blocks, low, high, frontier};
        SCATTER_PHASE(4);
        pack_task(&pack);
        SCATTER_PHASE(5);
        uint64_t *result = allocate(lows + highs, sizeof(uint64_t));
        copy_job low_copy = {low, result, lows}, high_copy = {high, result + lows, highs};
        SCATTER_PHASE(6);
        backend->fork2(backend, width, copy_task, &low_copy, copy_task, &high_copy);
        SCATTER_PHASE(7);
        free(job.chunks); free(low); free(high);
        SCATTER_PHASE(8);
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
#ifdef WFB_SCATTER_PHASES
    phase_form = form; phase_width = workers;
    if (direct) fail("phase image covers the chain representation only");
#endif
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
#ifdef WFB_SCATTER_PHASES
    phase_form = form; phase_width = workers;
    if (direct) fail("phase image covers the chain representation only");
#endif
    if (!strcmp(form, "wf")) return matrix(wf_bench_radix_scatter_par, wf_bench_radix_scatter_par_release);
    if (!strcmp(form, "wf-seq")) return matrix(wf_bench_radix_scatter_seq, wf_bench_radix_scatter_seq_release);
    select_backend(form, workers);
    return matrix(native_entry, native_release);
}
static void finish(const char *form) { (void)form; free(input); free(expected); input = expected = NULL; }
/* The chain needs fork2 as well as map; the static map-only backend cannot
 * execute this decomposition. Keep both native algorithms on the same forms. */
#ifdef WFB_SCATTER_PHASES
static const char *const forms[] = {"wf", "tbb", "parlay", "rayon-join", NULL};
#else
static const char *const forms[] = {"wf", "wf-seq", "serial", "tbb", "parlay", "rayon-join", NULL};
#endif
const wfb_kernel wfb_this_kernel = {
#ifdef WFB_SCATTER_PHASES
    "radix_scatter-phases",
#else
    "radix_scatter",
#endif
    forms, prepare, call, check, verify, finish, workload, 0, 0, NULL
};
#endif
