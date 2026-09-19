/* Complete-result oracle for the formal fir program. Correctness and
 * the separate paired performance runner share inputs and expected results. */
#include "oracle.h"
#include <inttypes.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#pragma STDC FP_CONTRACT OFF

const char *const wf_oracle_name = "fir";
const char *const wf_oracle_fixture = "taps=64 outputs=524288 seed=92821";

#define FIR_SEED UINT32_C(92821)
enum { MAX_TAPS = 64 };
/* The owned result is a Box<Array<T>> cell; the adapter hands that cell back
 * as the retained handle and the release row consumes exactly it. */
extern void wf_bench_fir(const double *, uint64_t, const double *, uint64_t, uint64_t, uint64_t, uint64_t, double **, uint64_t *, void **);
extern void wf_bench_fir_release(void *);
typedef struct { double delay[MAX_TAPS]; size_t next, taps; } DelayLine;

static double sample(uint32_t *state, size_t i) {
    *state = *state * UINT32_C(1664525) + UINT32_C(1013904223);
    int32_t v = (int32_t)(*state & UINT32_C(65535)) - 32768;
    double scale = i % 19 == 0 ? 65536.0 : (i % 19 == 1 ? 0x1p-20 : 1.0);
    return ((double)v / 1009.0) * scale;
}

static void check_samples(const double *expected, const double *actual, size_t count,
                          const char *part) {
    for (size_t i = 0; i < count; ++i) {
        uint64_t want, got;
        memcpy(&want, expected + i, sizeof want);
        memcpy(&got, actual + i, sizeof got);
        if (want != got) {
            (void)fprintf(stderr,
                          "fir: %s[%zu] expected=%016llx actual=%016llx\n", part, i,
                          (unsigned long long)want, (unsigned long long)got);
            wf_oracle_fail("fir: wrong output bits");
        }
    }
}

static void direct_filter(const double *input, size_t count, const double *history,
                          const double *taps, size_t tap_count, double *output,
                          double *next_history) {
    size_t history_count = tap_count - 1;
    for (size_t n = 0; n < count; ++n) {
        double sum = 0.0;
        for (size_t k = 0; k < tap_count; ++k) {
            double sample_value = n >= k ? input[n - k] : history[history_count + n - k];
            double product = taps[k] * sample_value;
            sum = sum + product;
        }
        output[n] = sum;
    }
    for (size_t i = 0; i < history_count; ++i) {
        size_t at = count + i;
        next_history[i] = at < history_count ? history[at] : input[at - history_count];
    }
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

static double *samples_new(size_t count) {
    double *samples = calloc(count ? count : 1, sizeof(*samples));
    if (!samples) wf_oracle_fail("fir: oracle allocation");
    return samples;
}

static size_t oracle(const double *input, size_t count, const double *history,
                     const double *taps, size_t tap_count, double *output,
                     double *next_history, int blocks) {
    double streamed_history[MAX_TAPS];
    double *streamed = samples_new(count);
    DelayLine line;
    size_t agreed = 0;

    direct_filter(input, count, history, taps, tap_count, output, next_history);

    delay_init(&line, history, tap_count);
    delay_filter(&line, input, count, taps, streamed);
    delay_history(&line, streamed_history);
    check_samples(output, streamed, count, "oracle-delay-line");
    check_samples(next_history, streamed_history, tap_count - 1, "oracle-delay-history");
    agreed += count + tap_count - 1;

    if (blocks) {
        size_t done = 0;
        memset(streamed, 0, (count ? count : 1) * sizeof(double));
        delay_init(&line, history, tap_count);
        while (done < count) {
            size_t block = 1 + done % 67;
            if (block > count - done) block = count - done;
            delay_filter(&line, input + done, block, taps, streamed + done);
            done += block;
        }
        delay_history(&line, streamed_history);
        check_samples(output, streamed, count, "oracle-ragged-output");
        check_samples(next_history, streamed_history, tap_count - 1, "oracle-ragged-history");
        agreed += count + tap_count - 1;
    }

    free(streamed);
    return agreed;
}

/* These witnesses reach WF too; agreement between two C oracles alone would
 * not guard generated arithmetic against contraction or reordered taps. */
static void check_witness(const double *input, size_t count, const double *history,
                          const double *taps, size_t tap_count, const double *expected) {
    double prefix[6], *output = NULL;
    uint64_t length = UINT64_MAX;
    void *held = NULL;
    size_t h = tap_count - 1;
    if (h + count > 6) wf_oracle_fail("fir: witness extent");
    memcpy(prefix, history, h * sizeof(double));
    memcpy(prefix + h, input, count * sizeof(double));
    wf_bench_fir(prefix, h + count, taps, tap_count, h, h + count, h, &output, &length, &held);
    if (length != count) wf_oracle_fail("fir: witness output extent");
    check_samples(expected, output, count, "WF-witness");
    wf_bench_fir_release(held);
}

static void check_known(void) {
    const double taps[] = {0.5, -0.25, 0.125};
    const double history[] = {2.0, -4.0};
    const double input[] = {8.0, 0.0, -8.0, 4.0};
    const double expected[] = {5.25, -2.5, -3.0, 4.0};
    const double expected_history[] = {-8.0, 4.0};
    double output[4], state[2];
    (void)oracle(input, 4, history, taps, 3, output, state, 1);
    check_samples(expected, output, 4, "known-output");
    check_samples(expected_history, state, 2, "known-history");
    check_witness(input, 4, history, taps, 3, expected);
}

static void check_rounding(void) {
    /* Ascending separate multiply/add gives +0; contraction gives -2^-54. */
    const double fused_taps[] = {-1.0, 0x1.0000002p0};
    const double fused_history[] = {0x1.ffffffcp-1};
    const double fused_input[] = {1.0};
    const double fused_expected[] = {0.0};
    double output[1], history[2];
    (void)oracle(fused_input, 1, fused_history, fused_taps, 2, output, history, 1);
    check_samples(fused_expected, output, 1, "rounding-unfused");
    check_witness(fused_input, 1, fused_history, fused_taps, 2, fused_expected);
    double contracted = fma(fused_taps[1], fused_history[0], -1.0);
    if (contracted == output[0]) wf_oracle_fail("fir: contraction witness did not discriminate");

    /* Ascending order gives 1; reverse order loses the small term and gives 0. */
    const double order_taps[] = {1.0, 1.0, 1.0};
    const double order_history[] = {1.0, -1e16};
    const double order_input[] = {1e16};
    const double order_expected[] = {1.0};
    (void)oracle(order_input, 1, order_history, order_taps, 3, output, history, 1);
    check_samples(order_expected, output, 1, "rounding-order");
    check_witness(order_input, 1, order_history, order_taps, 3, order_expected);
    double reversed = 0.0;
    for (size_t k = 3; k-- > 0;) {
        double sample_value = k == 0 ? order_input[0] : order_history[2 - k];
        reversed = reversed + order_taps[k] * sample_value;
    }
    if (reversed == output[0]) wf_oracle_fail("fir: tap-order witness did not discriminate");
}

typedef struct {
    double *prefix, *held_prefix, *taps, *held_taps;
    double *expected, *expected_history, *output;
    void *held;
    size_t n, k, h;
} Work;

static double *allocate(size_t n) {
    double *p = malloc((n ? n : 1) * sizeof *p);
    if (!p) wf_oracle_fail("fir: fixture allocation");
    return p;
}

static uint32_t fixture_seed(size_t k, size_t n) {
    return (uint32_t)(FIR_SEED + (uint32_t)k * UINT32_C(2654435761) +
                      (uint32_t)n * UINT32_C(40503));
}

static size_t input(Work *w, size_t k, size_t n, uint32_t seed, int blocks) {
    double *history, *stream;
    uint32_t random = seed;
    size_t agreed;
    memset(w, 0, sizeof *w);
    if (!(k >= 1 && k <= MAX_TAPS && n <= 16777216 - (k - 1)))
        wf_oracle_fail("fir: input domain");
    w->k = k;
    w->n = n;
    w->h = k - 1;
    w->prefix = allocate(w->h + n);
    w->held_prefix = allocate(w->h + n);
    w->taps = allocate(k);
    w->held_taps = allocate(k);
    w->expected = allocate(n);
    w->expected_history = allocate(w->h);
    history = allocate(w->h);
    stream = allocate(n);
    /* The generator and its call order are the bundle's. */
    for (size_t i = 0; i < n; ++i) stream[i] = sample(&random, i);
    for (size_t i = 0; i < w->h; ++i) history[i] = sample(&random, i + 3);
    for (size_t i = 0; i < k; ++i) w->taps[i] = sample(&random, i + 7) / 65536.0;
    memcpy(w->prefix, history, w->h * sizeof(double));
    memcpy(w->prefix + w->h, stream, n * sizeof(double));
    agreed = oracle(stream, n, history, w->taps, k, w->expected, w->expected_history, blocks);
    memcpy(w->held_prefix, w->prefix, (w->h + n) * sizeof(double));
    memcpy(w->held_taps, w->taps, k * sizeof(double));
    free(history);
    free(stream);
    return agreed;
}

static void release(Work *w) {
    if (w->held) wf_bench_fir_release(w->held);
    w->output = NULL;
    w->held = NULL;
}
static void run(Work *w) {
    uint64_t length = UINT64_MAX;
    wf_bench_fir(w->prefix, w->h + w->n, w->taps, w->k, w->h, w->h + w->n, w->h, &w->output, &length, &w->held);
    if (length != w->n) wf_oracle_fail("fir: output extent");
}

static size_t compare(Work *w) {
    if (!w->output && w->n) wf_oracle_fail("fir: output pointer");
    check_samples(w->expected, w->output, w->n, "output");
    if (memcmp(w->prefix, w->held_prefix, (w->h + w->n) * sizeof(double)))
        wf_oracle_fail("fir: input immutability");
    if (memcmp(w->taps, w->held_taps, w->k * sizeof(double)))
        wf_oracle_fail("fir: taps immutability");
    return w->n;
}

static void destroy(Work *w) {
    release(w);
    free(w->prefix);
    free(w->held_prefix);
    free(w->taps);
    free(w->held_taps);
    free(w->expected);
    free(w->expected_history);
    memset(w, 0, sizeof *w);
}

size_t wf_oracle_verify(void) {
    static const size_t taps[] = {1, 2, 3, 7, 15, 16, 17, 31, 32, 63, 64};
    static const size_t counts[] = {0, 1, 2, 17, 33, 255, 256, 257, 4097};
    size_t compared = 6;
    check_known();
    check_rounding();
    for (size_t k = 0; k < sizeof taps / sizeof taps[0]; ++k)
        for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n) {
            Work w;
            (void)input(&w, taps[k], counts[n], fixture_seed(taps[k], counts[n]), 1);
            run(&w);
            compared += compare(&w);
            destroy(&w);
        }
    return compared;
}

#ifdef WF_ORACLE_PERFORMANCE
static Work timed;
void wf_oracle_prepare(void) { (void)input(&timed, 64, 524288, FIR_SEED, 0); }
size_t wf_oracle_call(void) { run(&timed); return timed.n; }
size_t wf_oracle_check(void) {
    size_t compared = compare(&timed);
    release(&timed);
    return compared;
}
void wf_oracle_finish(void) { destroy(&timed); }
#endif
