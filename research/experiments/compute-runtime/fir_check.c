/* Causal FIR oracle for the compute workload. No performance measurements.
 * Every channel has K-1 initial history samples, oldest first. Multiplication
 * and addition are distinct, with ascending tap order and no FP reassociation.
 * Build with -fno-fast-math -ffp-contract=off. The direct oracle and a separate
 * streaming delay-line implementation must agree on every output/state bit.
 */
#include <inttypes.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef WF_COMPUTE_CONTROL
#include "runtime.h"
#endif

#if defined(WF_FILTER_NATIVE) && defined(WF_FILTER_HOST)
#error "Native and WF host qualification are separate build modes"
#endif
#ifdef WF_FILTER_NATIVE
#include "fir_native.h"

static void native_check_call(const double *, size_t, const double *, const double *,
                              size_t, const double *, const double *);
static uint64_t native_calls, native_samples;
#endif

enum { MAX_TAPS = 64 };


typedef struct {
    double delay[MAX_TAPS];
    size_t next;
    size_t taps;
} DelayLine;

static uint32_t next_random(uint32_t *state) {
    *state = *state * UINT32_C(1664525) + UINT32_C(1013904223);
    return *state;
}

static double random_sample(uint32_t *state) {
    int64_t value = (int64_t)(next_random(state) & UINT32_C(65535)) - 32768;
    return (double)value / 1024.0;
}

static void direct_filter(const double *input, size_t count,
                          const double *history, const double *taps,
                          size_t tap_count, double *output, double *next_history) {
    size_t history_count = tap_count - 1;
    for (size_t n = 0; n < count; ++n) {
        double sum = 0.0;
        for (size_t k = 0; k < tap_count; ++k) {
            double sample = n >= k ? input[n - k] : history[history_count + n - k];
            double product = taps[k] * sample;
            sum = sum + product;
        }
        output[n] = sum;
    }
    for (size_t i = 0; i < history_count; ++i) {
        size_t at = count + i;
        next_history[i] = at < history_count ? history[at] : input[at - history_count];
    }
#ifdef WF_FILTER_NATIVE
    native_check_call(input, count, history, taps, tap_count, output, next_history);
#endif
}

static void delay_init(DelayLine *line, const double *history, size_t tap_count) {
    memset(line, 0, sizeof(*line));
    line->taps = tap_count;
    line->next = tap_count - 1;
    memcpy(line->delay, history, (tap_count - 1) * sizeof(double));
}

static void delay_filter(DelayLine *line, const double *input, size_t count,
                         const double *taps, double *output) {
    for (size_t n = 0; n < count; ++n) {
        line->delay[line->next] = input[n];
        size_t read = line->next;
        double sum = 0.0;
        for (size_t k = 0; k < line->taps; ++k) {
            double product = taps[k] * line->delay[read];
            sum = sum + product;
            read = read == 0 ? line->taps - 1 : read - 1;
        }
        output[n] = sum;
        line->next = line->next + 1 == line->taps ? 0 : line->next + 1;
    }
}

static void delay_history(const DelayLine *line, double *history) {
    for (size_t i = 0; i + 1 < line->taps; ++i)
        history[i] = line->delay[(line->next + 1 + i) % line->taps];
}

static void check_samples(const double *expected, const double *actual,
                          size_t count, const char *part, size_t case_id,
                          size_t channel) {
    for (size_t i = 0; i < count; ++i) {
        uint64_t want, got;
        memcpy(&want, expected + i, sizeof(want));
        memcpy(&got, actual + i, sizeof(got));
        if (want != got) {
            fprintf(stderr, "FIR mismatch case=%zu channel=%zu %s[%zu] "
                    "expected=%016" PRIx64 " actual=%016" PRIx64 "\n",
                    case_id, channel, part, i, want, got);
            exit(1);
        }
    }
}

static double *samples_new(size_t count) {
    double *samples = calloc(count ? count : 1, sizeof(*samples));
    if (!samples) {
        perror("FIR allocation");
        exit(1);
    }
    return samples;
}

#ifdef WF_FILTER_HOST
/* This fixture calls the exact compiler-emitted revision through fir_host.ll.
 * It does not define a writer-visible FFI. Every call satisfies the source
 * contract, and borrowed input/taps stay live until all joins have returned. */
typedef void *(*FilterEntry)(const double *, uint64_t, const double *, uint64_t,
                             uint64_t, uint64_t, uint64_t, uint64_t);
extern void *wf_research_fir_parallel(const double *, uint64_t, const double *,
                                    uint64_t, uint64_t, uint64_t, uint64_t, uint64_t);
extern void *wf_research_fir_sequential(const double *, uint64_t, const double *,
                                      uint64_t, uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_research_fir_count(const void *);
extern uint64_t wf_research_fir_get(const void *, uint64_t, double *);
extern uint64_t wf_research_fir_history(const double *, uint64_t, uint64_t,
                                      uint64_t, double *);
extern uint64_t wf_research_fir_release(void *);
extern int wf__par_pool_active(void);
extern int wf__floor_run(int, char **);
static FilterEntry filter_entry;
static uint64_t wf_calls, wf_samples, wf_states, wf_misses;

static void require(int condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "FIR WF qualification: %s\n", message);
        exit(1);
    }
}

static void wf_check_call(const double *input, size_t count, const double *history,
                          const double *taps, size_t tap_count,
                          const double *expected, const double *expected_history,
                          double *next_history, size_t tile_size,
                          size_t case_id, size_t channel) {
    require(tap_count >= 1 && tap_count <= MAX_TAPS, "invalid tap count");
    require(count <= 16777216 - (tap_count - 1), "input domain exceeded");
    require(tile_size >= 1 && tile_size <= 65536, "invalid tile size");
    size_t h = tap_count - 1;
    size_t prefix_count = h + count;
    double *prefix = samples_new(prefix_count);
    double *unchanged = samples_new(prefix_count);
    double saved_taps[MAX_TAPS];
    memcpy(prefix, history, h * sizeof(double));
    memcpy(prefix + h, input, count * sizeof(double));
    memcpy(unchanged, prefix, prefix_count * sizeof(double));
    memcpy(saved_taps, taps, tap_count * sizeof(double));
    void *tree = filter_entry(prefix, prefix_count, taps, tap_count,
                             h, prefix_count, h, tile_size);
    require(tree != NULL, "missing owned result");
    require(wf_research_fir_count(tree) == count, "wrong complete output count");
    for (size_t i = 0; i < count; ++i) {
        double actual = 0x1.23456789abcdep42;
        require(wf_research_fir_get(tree, i, &actual) == 1, "missing output sample");
        check_samples(expected + i, &actual, 1, "wf-output", case_id, channel);
    }
    const uint64_t absent[] = {count, UINT64_MAX};
    for (size_t i = 0; i < sizeof(absent) / sizeof(absent[0]); ++i) {
        double actual = 0x1.23456789abcdep42;
        double sentinel = actual;
        require(wf_research_fir_get(tree, absent[i], &actual) == 0,
                "out-of-range output reported a hit");
        check_samples(&sentinel, &actual, 1, "wf-miss-preserved", case_id, channel);
        ++wf_misses;
    }
    for (size_t i = 0; i < h; ++i) {
        require(wf_research_fir_history(prefix, prefix_count, h, i,
                                       next_history + i) == 1,
                "missing history sample");
    }
    check_samples(expected_history, next_history, h, "wf-history", case_id, channel);
    double actual = 0x1.23456789abcdep42;
    double sentinel = actual;
    require(wf_research_fir_history(prefix, prefix_count, h, h, &actual) == 0,
            "out-of-range history reported a hit");
    check_samples(&sentinel, &actual, 1, "wf-history-miss", case_id, channel);
    ++wf_misses;
    require(wf_research_fir_release(tree) == 0, "owned result release failed");
    check_samples(unchanged, prefix, prefix_count, "wf-input-immutable", case_id, channel);
    check_samples(saved_taps, taps, tap_count, "wf-taps-immutable", case_id, channel);
    free(unchanged);
    free(prefix);
    ++wf_calls;
    wf_samples += count;
    wf_states += h;
}
#endif

#ifdef WF_FILTER_NATIVE
static const struct { const char *name; FirNativeKernel run; } native_forms[] = {
    {"direct", fir_native_direct}, {"lanes4", fir_native_lanes4},
    {"lanes8", fir_native_lanes8}, {"lanes16", fir_native_lanes16}
};

static void native_check_call(const double *input, size_t count, const double *history,
                              const double *taps, size_t k, const double *expected,
                              const double *expected_state) {
    if (k < 1 || k > MAX_TAPS || count > 16777216 - (k - 1)) {
        fputs("Native FIR qualification: invalid fixture dimensions\n", stderr);
        exit(1);
    }
    size_t prefix_count = count + k - 1;
    double *prefix = samples_new(prefix_count);
    double *saved = samples_new(prefix_count);
    memcpy(prefix, history, (k - 1) * sizeof(double));
    memcpy(prefix + k - 1, input, count * sizeof(double));
    memcpy(saved, prefix, prefix_count * sizeof(double));
    double saved_taps[MAX_TAPS];
    memcpy(saved_taps, taps, k * sizeof(double));
    double *backing = samples_new(count + 2);
    const double sentinel = 0x1.23456789abcdep42;
    double state[MAX_TAPS];
    for (size_t form = 0; form < sizeof(native_forms) / sizeof(native_forms[0]); ++form) {
        for (size_t i = 0; i < count + 2; ++i) backing[i] = sentinel;
        native_forms[form].run(prefix, taps, k, count, backing + 1);
        check_samples(expected, backing + 1, count, native_forms[form].name, native_calls, 0);
        check_samples(&sentinel, backing, 1, "before-output", native_calls, form);
        check_samples(&sentinel, backing + count + 1, 1, "after-output", native_calls, form);
        check_samples(saved, prefix, prefix_count, "prefix-immutable", native_calls, form);
        check_samples(saved_taps, taps, k, "taps-immutable", native_calls, form);
        /* Separate host state preparation; not charged to the pure kernel. */
        memcpy(state, prefix + count, (k - 1) * sizeof(double));
        check_samples(expected_state, state, k - 1, "native-next-history", native_calls, form);
        ++native_calls;
        native_samples += count;
    }
    free(backing);
    free(saved);
    free(prefix);
}

static void check_native_boundaries(void) {
    const size_t sizes[] = {0, 1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 257};
    for (size_t k = 1; k <= MAX_TAPS; ++k) {
        for (size_t size = 0; size < sizeof(sizes) / sizeof(sizes[0]); ++size) {
            size_t n = sizes[size];
            double history[MAX_TAPS] = {0}, taps[MAX_TAPS] = {0}, state[MAX_TAPS] = {0};
            double input[257], output[257];
            uint32_t seed = UINT32_C(0x83177731) ^ (uint32_t)k ^ (uint32_t)(n << 16);
            for (size_t i = 0; i < k; ++i) taps[i] = random_sample(&seed) / 7.0;
            for (size_t i = 0; i + 1 < k; ++i) history[i] = random_sample(&seed) / 10.0;
            for (size_t i = 0; i < n; ++i)
                input[i] = random_sample(&seed) / 10.0 * (i % 3 == 0 ? 1e12 : 1e-12);
            direct_filter(input, n, history, taps, k, output, state);
            /* Every ragged window invokes each native form too, including an
             * empty window. The reference state is checked against the full
             * result after processing the entire stream. */
            double carry[MAX_TAPS];
            memcpy(carry, history, (k - 1) * sizeof(double));
            size_t done = 0;
            do {
                double window[257], after[MAX_TAPS] = {0};
                direct_filter(input + done, 0, carry, taps, k, window, after);
                check_samples(carry, after, k - 1, "empty-window-state", k, size);
                if (done == n) break;
                size_t count = 1 + done % 19;
                if (count > n - done) count = n - done;
                direct_filter(input + done, count, carry, taps, k, window, after);
                check_samples(output + done, window, count, "window-output", k, size);
                memcpy(carry, after, (k - 1) * sizeof(double));
                done += count;
            } while (1);
            check_samples(state, carry, k - 1, "whole-stream-state", k, size);
        }
    }
}

#endif

static void check_known(void) {
    const double taps[] = {0.5, -0.25, 0.125};
    const double history[] = {2.0, -4.0};
    const double input[] = {8.0, 0.0, -8.0, 4.0};
    const double expected[] = {5.25, -2.5, -3.0, 4.0};
    const double expected_history[] = {-8.0, 4.0};
    double output[4], state[2];
    direct_filter(input, 4, history, taps, 3, output, state);
    check_samples(expected, output, 4, "known-output", 0, 0);
    check_samples(expected_history, state, 2, "known-history", 0, 0);
#ifdef WF_FILTER_HOST
    double wf_state[MAX_TAPS];
    wf_check_call(input, 4, history, taps, 3, expected, expected_history,
                  wf_state, 2, 0, 0);
#endif

}

static void check_rounding(void) {
    /* Ascending separate multiply/add gives +0; contraction gives -2^-54. */
    const double fused_taps[] = {-1.0, 0x1.0000002p0};
    const double fused_history[] = {0x1.ffffffcp-1};
    const double fused_input[] = {1.0};
    const double fused_expected[] = {0.0};
    double output[1], history[2];
    direct_filter(fused_input, 1, fused_history, fused_taps, 2, output, history);
    check_samples(fused_expected, output, 1, "rounding-unfused", 0, 0);
#ifdef WF_FILTER_HOST
    double wf_state[MAX_TAPS];
    wf_check_call(fused_input, 1, fused_history, fused_taps, 2, output, history,
                  wf_state, 1, 0, 0);
#endif
    double contracted = fma(fused_taps[1], fused_history[0], -1.0);
    if (contracted == output[0]) {
        fputs("FIR contraction witness did not discriminate\n", stderr);
        exit(1);
    }
    /* Ascending order gives 1; reverse order loses the small term and gives 0. */
    const double order_taps[] = {1.0, 1.0, 1.0};
    const double order_history[] = {1.0, -1e16};
    const double order_input[] = {1e16};
    const double order_expected[] = {1.0};
    direct_filter(order_input, 1, order_history, order_taps, 3, output, history);
    check_samples(order_expected, output, 1, "rounding-order", 0, 0);
#ifdef WF_FILTER_HOST
    wf_check_call(order_input, 1, order_history, order_taps, 3, output, history,
                  wf_state, 1, 0, 0);
#endif
    double reversed = 0.0;

    for (size_t k = 3; k-- > 0;) {
        double sample = k == 0 ? order_input[0] : order_history[2 - k];
        reversed = reversed + order_taps[k] * sample;
    }
    if (reversed == output[0]) {
        fputs("FIR tap-order witness did not discriminate\n", stderr);
        exit(1);
    }
}

static int check_suite(void) {

    static const struct {
        size_t taps, channels, count;
        int skew;
    } cases[] = {
        {1, 1, 0, 0}, {1, 3, 1, 0}, {8, 3, 1, 1},
        {8, 17, 7, 1}, {31, 3, 31, 0}, {31, 17, 65, 1},
        {64, 3, 257, 1}, {8, 17, 4099, 1}, {64, 3, 65537, 1},
        {31, 3, 97, 0}, {64, 3, 513, 1}
    };
    check_known();
    check_rounding();
    uint64_t total = 0;
    size_t checked_channels = 0;
    for (size_t c = 0; c < sizeof(cases) / sizeof(cases[0]); ++c) {
        size_t k = cases[c].taps;
        for (size_t channel = 0; channel < cases[c].channels; ++channel) {
            size_t count = cases[c].count;
            if (cases[c].skew && channel % 3 != 0)
                count = channel % 3 == 1 ? count / 17 : 0;
            double *input = samples_new(count);
            double *expected = samples_new(count);
            double *streamed = samples_new(count);
            double history[MAX_TAPS] = {0}, taps[MAX_TAPS] = {0};
            double next_history[MAX_TAPS] = {0}, streamed_history[MAX_TAPS] = {0};
            uint32_t seed = UINT32_C(0x91e10da5) ^ (uint32_t)c ^ ((uint32_t)channel << 12);
            for (size_t i = 0; i < k; ++i)
                taps[i] = random_sample(&seed) / 64.0;
            for (size_t i = 0; i + 1 < k; ++i)
                history[i] = random_sample(&seed);
            for (size_t i = 0; i < count; ++i) {
                if (c % 4 == 0) input[i] = i == count / 2 ? 8.0 : 0.0;
                else if (c % 4 == 1) input[i] = 1.0;
                else if (c % 4 == 2) input[i] = i % 2 ? -3.0 : 3.0;
                else input[i] = random_sample(&seed);
            }
            if (c == 9 || c == 10) {
                for (size_t i = 0; i < k; ++i)
                    taps[i] = random_sample(&seed) / 7.0;
                for (size_t i = 0; i + 1 < k; ++i)
                    history[i] = random_sample(&seed) / 10.0;
                for (size_t i = 0; i < count; ++i) {
                    double value = random_sample(&seed) / 10.0;
                    if (c == 10)
                        value *= i % 3 == 0 ? 1e12 : (i % 3 == 1 ? 1e-12 : 1.0);
                    input[i] = value;
                }
            }
            direct_filter(input, count, history, taps, k, expected, next_history);

            DelayLine line;
            delay_init(&line, history, k);
            delay_filter(&line, input, 0, taps, streamed);
            size_t done = 0;
            while (done < count) {
                size_t block = 1 + done % 67;
                if (block > count - done) block = count - done;
#ifdef WF_FILTER_NATIVE
                double before[MAX_TAPS], after[MAX_TAPS];
                delay_history(&line, before);
#endif
                delay_filter(&line, input + done, 0, taps, streamed + done);
#ifdef WF_FILTER_NATIVE
                native_check_call(input + done, 0, before, taps, k, expected + done, before);
#endif

                delay_filter(&line, input + done, block, taps, streamed + done);
#ifdef WF_FILTER_NATIVE
                delay_history(&line, after);
                native_check_call(input + done, block, before, taps, k, expected + done, after);
#endif

                done += block;
            }
            delay_history(&line, streamed_history);
            check_samples(expected, streamed, count, "stream-output", c, channel);
            check_samples(next_history, streamed_history, k - 1, "stream-history", c, channel);
#ifdef WF_FILTER_HOST
            const size_t tile_sizes[] = {1, 17, 257, 65536};
            double wf_history[MAX_TAPS] = {0};
            for (size_t tile = 0; tile < sizeof(tile_sizes) / sizeof(tile_sizes[0]); ++tile)
                wf_check_call(input, count, history, taps, k, expected, next_history,
                              wf_history, tile_sizes[tile], c, channel);
            /* Host streams carry state obtained from ordinary WF history_get.
             * Each window is fully checked against its own direct oracle, then
             * against the one-call result; an empty call executes even at N=0. */
            memcpy(wf_history, history, (k - 1) * sizeof(double));
            double *block_expected = samples_new(count);
            double block_state[MAX_TAPS] = {0}, next_wf_state[MAX_TAPS] = {0};
            done = 0;
            do {
                wf_check_call(input + done, 0, wf_history, taps, k,
                              expected + done, wf_history, next_wf_state, 17, c, channel);
                memcpy(wf_history, next_wf_state, (k - 1) * sizeof(double));
                if (done == count) break;
                size_t block = 1 + done % 67;
                if (block > count - done) block = count - done;
                direct_filter(input + done, block, wf_history, taps, k,
                              block_expected, block_state);
                check_samples(expected + done, block_expected, block,
                              "window-oracle", c, channel);
                wf_check_call(input + done, block, wf_history, taps, k,
                              block_expected, block_state, next_wf_state, 17, c, channel);
                memcpy(wf_history, next_wf_state, (k - 1) * sizeof(double));
                done += block;
            } while (1);
            check_samples(next_history, wf_history, k - 1, "wf-stream-history", c, channel);
            free(block_expected);
#endif
            total += count;

            ++checked_channels;
            free(streamed);
            free(expected);
            free(input);
        }
    }
    printf("FIR scalar oracle PASS: cases=%zu channels=%zu samples=%" PRIu64
#ifdef WF_FILTER_HOST
           "; WF complete output/history included\n",
#else
           "; WF execution not included\n",
#endif
           sizeof(cases) / sizeof(cases[0]),
           checked_channels, total);
    return 0;
}

#ifdef WF_FILTER_HOST
int wf__main_body(int argc, char **argv) {
    (void)argc;
    (void)argv;
    int active = wf__par_pool_active() != 0;
    filter_entry = active ? wf_research_fir_parallel : wf_research_fir_sequential;
    int status = check_suite();
#ifdef WF_COMPUTE_CONTROL
    unsigned workers = wf_compute_worker_count();
    require(active ? workers >= 2 : workers == 0, "pool selection/worker count mismatch");
#endif

    printf("FIR WF qualification PASS: world=%s calls=%" PRIu64
           " samples=%" PRIu64 " history=%" PRIu64 " misses=%" PRIu64 "\n",
           active ? "parallel" : "sequential-clone", wf_calls, wf_samples, wf_states, wf_misses);
#ifdef WF_COMPUTE_CONTROL
    printf("FIR recovered control: workers=%u grants=%lu\n", workers, wf__par_grants());
#endif

    return status;
}

int main(int argc, char **argv) {
    return wf__floor_run(argc, argv);
}
#else
int main(void) {
#ifdef WF_FILTER_NATIVE
    int status = check_suite();
    check_native_boundaries();
    printf("Native FIR qualification PASS: forms=%zu calls=%" PRIu64
           " samples=%" PRIu64 " K=1..64\n",
           sizeof(native_forms) / sizeof(native_forms[0]), native_calls, native_samples);
    return status;
#else
    return check_suite();
#endif
}

#endif
