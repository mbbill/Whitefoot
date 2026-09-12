/* Serves compute-bench: the FIR kernel -- a data-parallel loop of small
 * tasks. It owns the LCG generator, the two-algorithm oracle and its two
 * discriminating witnesses, the scalar C kernel body, the dispatch over every
 * form, and the per-call checks.
 *
 * Adapted from the research bundle's fir_native.c, fir_check.c and
 * fir_bench.c. `sample()` is fir_bench.c's generator copied verbatim;
 * `direct_filter`, `DelayLine`, `delay_init`, `delay_filter`,
 * `delay_history`, `check_known` and `check_rounding` are fir_check.c's
 * copied verbatim with their WF_FILTER_HOST and WF_FILTER_NATIVE blocks cut;
 * `fir_native_direct` is fir_native.c's copied verbatim, carrying that file's
 * FIR_TAP_ORDER pragma on the tap loop under its own __clang__ guard.
 *
 * Cut: the bundle's lane-blocked SIMD kernels fir_native_lanes4/8/16. They
 * are bit-identical to the scalar kernel and roughly nine times faster with
 * SLP enabled, and they are not carried because the emitted module's
 * fmul.strict and fadd.strict cannot be reassociated, so a vectorized
 * reference would compare code generation against scalar code generation
 * rather than one decomposition against another. The flag set turns
 * vectorization off for every implementation of every kernel so the asymmetry
 * cannot reappear by accident. Also cut: the bundle's `tile_size` knob and
 * the whole tile-tree WF representation it belongs to -- programs/fir.wf is
 * the flat form, so the WF program's observable work is the same as every
 * reference's -- and the bundle's stored call counts, which are re-derived
 * and printed here instead.
 *
 * THE TIMED INTERVAL IS LOAD-BEARING FOR FAIRNESS and is identical for every
 * implementation of this kernel: allocate the output buffer, zero-fill it,
 * compute, join every participant. Release and free are outside it.
 * Allocation and the zero-fill are inside because Whitefoot's
 * buffer_new(count, 0.0) allocates and zero-fills, so every reference calls
 * malloc plus memset inside the interval. A re-cut that lets the native path
 * allocate uninitialized compares allocators, not schedulers.
 *
 * The output-window canaries that detect an overrun therefore live in
 * `verify`, which is never timed; a canary pass over a 524,288-element output
 * is work the Whitefoot form does not pay. In the timed path the zero-fill is
 * the omission witness: the mixed-magnitude generator makes every expected
 * output of the timed fixture a rounded sum of sixty-four nonzero products,
 * so an element the form never writes reads +0.0 and check() fails on it. */
#include "backend.h"
#include "harness.h"
#include "fir_split.h"

#include <inttypes.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The timed fixture. At the emitted weight 150 the runtime's work divisor is
 * 1,000, so 524,288 outputs afford 524 chunks; min(16 * lanes, 524) is the cap
 * at every width a table records here -- 32 chunks at width 2, 64 at width 4,
 * 128 at width 8, 256 at width 16 and 512 at width 32. The scheduler's cap,
 * not this size, is what sets this row's chunk count. The size stands on the
 * [5 ms, 60 ms] wf-seq window instead, being the largest the window admits
 * here. */
#define FIR_TAPS ((size_t)64)
#define FIR_OUTPUTS ((size_t)524288)
#define FIR_SEED ((uint32_t)92821)
/* Callback grain: 256 outputs, giving 2,048 chunks at the default size
 * against the WF program's 64. Section 5.0 of the specification discloses
 * what this number is: a choice made for this bundle with no bundle
 * measurement behind it, because the bundle's only FIR grain evidence is a WF
 * tile_size sweep belonging to the tile-tree representation this kernel does
 * not carry. It is a fixed policy, never calibrated per host. */
#define FIR_GRAIN ((size_t)256)

enum { MAX_TAPS = 64 };

/* An unlikely bit pattern, from the bundle: it guards the doubles either side
 * of the output window in `verify`. */
static const double FIR_SENTINEL = 0x1.23456789abcdep42;

extern void wf_bench_fir_par(const double *, uint64_t, const double *, uint64_t, uint64_t,
                             uint64_t, uint64_t, double **, uint64_t *);
extern void wf_bench_fir_par_release(double *, uint64_t);
extern void wf_bench_fir_seq(const double *, uint64_t, const double *, uint64_t, uint64_t,
                             uint64_t, uint64_t, double **, uint64_t *);
extern void wf_bench_fir_seq_release(double *, uint64_t);

/* ------------------------------------------------------------ generator -- */

/* Copied verbatim from the bundle's fir_bench.c. The mixed magnitudes exist
 * to make tap order and rounding observable: without them a reassociated or
 * reversed reduction would agree with the specified one bit for bit. */
static double sample(uint32_t *state, size_t i) {
    *state = *state * UINT32_C(1664525) + UINT32_C(1013904223);
    int32_t v = (int32_t)(*state & UINT32_C(65535)) - 32768;
    double scale = i % 19 == 0 ? 65536.0 : (i % 19 == 1 ? 0x1p-20 : 1.0);
    return ((double)v / 1009.0) * scale;
}

/* --------------------------------------------------------------- oracle -- */

/* Raw bit patterns, never a tolerance. */
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
            wfb_fail("fir: wrong output bits");
        }
    }
}

typedef struct {
    double delay[MAX_TAPS];
    size_t next;
    size_t taps;
} DelayLine;

/* Algorithm one: direct convolution over a separate history array, ascending
 * tap order, multiply and add rounded separately. Copied verbatim. */
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

/* Algorithm two: a stateful circular delay line. Structurally unlike the
 * direct form -- it visits the same products through a rotating cursor and
 * carries its state across calls -- so agreement between the two is evidence
 * about the answer rather than about one implementation. Copied verbatim. */
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
    if (!samples) wfb_fail("fir: oracle allocation");
    return samples;
}

/* The two-algorithm oracle. Both algorithms run and must agree on every
 * output and on the next history, compared as raw bit patterns; only then is
 * `output` the expected result any form is measured against. `blocks` also
 * re-runs the delay line in ragged windows, which exercises its carried state
 * at every phase of the cursor. Never inside a timed interval. */
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

/* ------------------------------------------------------------ witnesses -- */

/* Hand-computed outputs: taps 0.5 / -0.25 / 0.125 over history 2 / -4 and
 * input 8 / 0 / -8 / 4 give 5.25 / -2.5 / -3 / 4, every value exact in
 * binary64. Copied verbatim. */
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
}

/* Two witnesses that the specified decomposition is the one being computed,
 * and -- this is the point -- each ASSERTS THAT IT DISCRIMINATES, so a
 * witness that stopped distinguishing the two answers fails loudly instead of
 * passing vacuously. Copied verbatim. */
static void check_rounding(void) {
    /* Ascending separate multiply/add gives +0; contraction gives -2^-54. */
    const double fused_taps[] = {-1.0, 0x1.0000002p0};
    const double fused_history[] = {0x1.ffffffcp-1};
    const double fused_input[] = {1.0};
    const double fused_expected[] = {0.0};
    double output[1], history[2];
    (void)oracle(fused_input, 1, fused_history, fused_taps, 2, output, history, 1);
    check_samples(fused_expected, output, 1, "rounding-unfused");
    double contracted = fma(fused_taps[1], fused_history[0], -1.0);
    if (contracted == output[0]) wfb_fail("fir: contraction witness did not discriminate");

    /* Ascending order gives 1; reverse order loses the small term and gives 0. */
    const double order_taps[] = {1.0, 1.0, 1.0};
    const double order_history[] = {1.0, -1e16};
    const double order_input[] = {1e16};
    const double order_expected[] = {1.0};
    (void)oracle(order_input, 1, order_history, order_taps, 3, output, history, 1);
    check_samples(order_expected, output, 1, "rounding-order");
    double reversed = 0.0;
    for (size_t k = 3; k-- > 0;) {
        double sample_value = k == 0 ? order_input[0] : order_history[2 - k];
        reversed = reversed + order_taps[k] * sample_value;
    }
    if (reversed == output[0]) wfb_fail("fir: tap-order witness did not discriminate");
}

/* ------------------------------------------------------- the scalar kernel */

/* Clang's ordered tap-reduction vectorization can consume the loop before
 * output recurrences reach SLP. Disable just that loop transformation; this
 * leaves output-lane packing enabled and changes no arithmetic permission.
 * Other compilers may choose different code and require fresh qualification.
 * Carried from fir_native.c, where it guards the same reduction. */
#if defined(__clang__)
#define FIR_TAP_ORDER _Pragma("clang loop vectorize(disable) interleave(disable)")
#else
#define FIR_TAP_ORDER
#endif

/* Copied verbatim from fir_native.c. Contract: 1<=tap_count<=64,
 * count+tap_count-1<=16777216; prefix contains tap_count-1 old history
 * samples (oldest first), then count new samples. Each output starts at +0
 * and visits all taps in ascending order, rounding multiply and add
 * separately -- the same decomposition programs/fir.wf writes with
 * fmul.strict and fadd.strict. */
static void fir_native_direct(const double *restrict prefix, const double *restrict taps,
                              size_t tap_count, size_t count, double *restrict output) {
    size_t history = tap_count - 1;
    for (size_t n = 0; n < count; ++n) {
        double sum = 0.0;
        FIR_TAP_ORDER
        for (size_t k = 0; k < tap_count; ++k) {
            double product = taps[k] * prefix[history + n - k];
            sum = sum + product;
        }
        output[n] = sum;
    }
}

/* ------------------------------------------------------------- fixtures -- */

typedef struct {
    double *prefix, *held_prefix; /* history-prefixed input: h + n doubles */
    double *taps, *held_taps;
    double *expected;
    double *expected_history;
    double *backing; /* the native output allocation; NULL for a WF result */
    double *output;  /* the n output doubles, wherever they live */
    int output_from_wf, output_from_seq, guarded;
    size_t n, k, h, grain;
} Work;

static double *allocate(size_t n) {
    double *p = malloc((n ? n : 1) * sizeof *p);
    if (!p) wfb_fail("fir: fixture allocation");
    return p;
}

/* Seeds are derived from (K, N) explicitly, never from a cell ordinal: a
 * seed that counts fixtures silently re-seeds every later fixture when the
 * grid changes. */
static uint32_t fixture_seed(size_t k, size_t n) {
    return (uint32_t)(FIR_SEED + (uint32_t)k * UINT32_C(2654435761) +
                      (uint32_t)n * UINT32_C(40503));
}

static size_t input(Work *w, size_t k, size_t n, uint32_t seed, size_t grain, int blocks) {
    double *history, *stream;
    uint32_t random = seed;
    size_t agreed;
    memset(w, 0, sizeof *w);
    if (!(k >= 1 && k <= MAX_TAPS && n <= 16777216 - (k - 1) && grain >= 1 &&
          grain <= 1048576))
        wfb_fail("fir: input domain");
    w->k = k;
    w->n = n;
    w->h = k - 1;
    w->grain = grain;
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

static void native_chunk(void *opaque, size_t chunk) {
    Work *w = opaque;
    size_t first = chunk * w->grain, end = first + w->grain;
    if (end > w->n) end = w->n;
    /* The flat native kernel indexes the history-prefixed input directly, so
     * a partition boundary copies no samples and re-bases no history. */
    fir_native_direct(w->prefix + first, w->taps, w->k, end - first, w->output + first);
}

static void release(Work *w) {
    if (!w->output) return;
    if (w->output_from_wf) {
        if (w->output_from_seq) wf_bench_fir_seq_release(w->output, w->n);
        else wf_bench_fir_par_release(w->output, w->n);
    } else {
        free(w->backing);
    }
    w->backing = NULL;
    w->output = NULL;
    w->output_from_wf = 0;
    w->output_from_seq = 0;
    w->guarded = 0;
}

/* One complete call on `w`. This is the whole timed interval and nothing
 * else. `guard` is never set on the timed path; see the file comment. */
static void run(Work *w, const char *form, unsigned width, int guard) {
    if (!strcmp(form, "wf") || !strcmp(form, "wf-seq")) {
        uint64_t length = UINT64_MAX;
        double *out = NULL;
        int seq = !strcmp(form, "wf-seq");
        if (seq)
            wf_bench_fir_seq(w->prefix, (uint64_t)(w->h + w->n), w->taps, (uint64_t)w->k,
                             (uint64_t)w->h, (uint64_t)(w->h + w->n), (uint64_t)w->h, &out,
                             &length);
        else
            wf_bench_fir_par(w->prefix, (uint64_t)(w->h + w->n), w->taps, (uint64_t)w->k,
                             (uint64_t)w->h, (uint64_t)(w->h + w->n), (uint64_t)w->h, &out,
                             &length);
        if (length != (uint64_t)w->n) wfb_fail("fir: generated length");
        w->output = out;
        w->backing = NULL;
        w->output_from_wf = 1;
        w->output_from_seq = seq;
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        size_t slots = (guard ? w->n + 2 : w->n);
        if (!backend || !backend->map) wfb_fail("fir: no such form");
        w->backing = malloc((slots ? slots : 1) * sizeof(double));
        if (!w->backing) wfb_fail("fir: output allocation");
        w->output_from_wf = 0;
        w->guarded = guard;
        if (guard) {
            /* Never timed: one sentinel double either side of the output
             * window, so a form that runs off the end is caught rather than
             * silently overwriting a neighbouring allocation. */
            for (size_t i = 0; i < slots; ++i) w->backing[i] = FIR_SENTINEL;
            w->output = w->backing + 1;
        } else {
            w->output = w->backing;
        }
        memset(w->output, 0, w->n * sizeof(double));
        backend->map(backend, width, (w->n + w->grain - 1) / w->grain, native_chunk, w);
    }
}

static size_t compare(Work *w) {
    if (!w->output && w->n) wfb_fail("fir: output pointer");
    check_samples(w->expected, w->output, w->n, "output");
    if (w->guarded) {
        check_samples(&FIR_SENTINEL, w->backing, 1, "before-output");
        check_samples(&FIR_SENTINEL, w->backing + w->n + 1, 1, "after-output");
    }
    if (memcmp(w->prefix, w->held_prefix, (w->h + w->n) * sizeof(double)))
        wfb_fail("fir: input immutability");
    if (memcmp(w->taps, w->held_taps, w->k * sizeof(double)))
        wfb_fail("fir: taps immutability");
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

/* ------------------------------------------------------------ the kernel -- */

static Work timed;
static int prepared;

static void fir_prepare(unsigned width) {
    (void)width;
    if (prepared) return;
    (void)input(&timed, FIR_TAPS, FIR_OUTPUTS, FIR_SEED, FIR_GRAIN, 0);
    prepared = 1;
}

static size_t fir_call(const char *form, unsigned width) {
    timed.grain = FIR_GRAIN;
    run(&timed, form, width, 0);
    return timed.n;
}

/* Outside every timed interval. It compares, then releases the call's buffer,
 * so the next call allocates from the same state this one did. */
static size_t fir_check(void) {
    size_t compared = compare(&timed);
    release(&timed);
    return compared;
}

static size_t fir_verify(const char *form, unsigned width) {
    static const size_t taps[] = {1, 2, 3, 7, 15, 16, 17, 31, 32, 63, 64};
    static const size_t counts[] = {0, 1, 2, 17, 33, 255, 256, 257, 4097};
    static const size_t grains[] = {1, 17, 256};
    size_t compared = 0, calls = 0, fixtures = 0, agreed = 0, leaves = 0;
    int generated = !strcmp(form, "wf") || !strcmp(form, "wf-seq");

    check_known();
    check_rounding();

    for (size_t t = 0; t < sizeof taps / sizeof taps[0]; ++t)
        for (size_t c = 0; c < sizeof counts / sizeof counts[0]; ++c) {
            Work w;
            agreed += input(&w, taps[t], counts[c], fixture_seed(taps[t], counts[c]),
                            grains[0], 1);
            ++fixtures;
            /* The scalar C kernel body against the same oracle, once per
             * fixture, so a form that dispatches it correctly is not the only
             * thing being checked. */
            {
                double *leaf = samples_new(w.n);
                fir_native_direct(w.prefix, w.taps, w.k, w.n, leaf);
                check_samples(w.expected, leaf, w.n, "leaf");
                leaves += w.n;
                free(leaf);
            }
            for (size_t g = 0; g < sizeof grains / sizeof grains[0]; ++g) {
                w.grain = grains[g];
                /* The sentinel canaries live here, where nothing is timed. A
                 * generated result owns its own buffer and cannot carry
                 * them. */
                run(&w, form, width, !generated);
                compared += compare(&w);
                ++calls;
                release(&w);
            }
            destroy(&w);
        }

    /* One fixture above the split admission floor. Every fixture above is
     * 4,097 outputs at most, and the linked scheduler splits a loop only from
     * 2 * ceil(split_work / weight) iterations upward, so without this the
     * --par module takes its unsplit path at every width in verify and the
     * chunked path the table times is never the path the oracle checks. The
     * size is derived here from the weight the Makefile read out of this
     * kernel's own emitted module, rounded up to a whole number of callback
     * chunks so every reference form sees full chunks too; no count is
     * stored, and a change of weight moves the fixture rather than silently
     * returning it below the floor. One grain, not the three the grid above
     * sweeps: grain 1 over a fixture this size is tens of thousands of
     * callbacks and this fixture is here for the split, not for the grain.
     * The width the harness passes in is the only thing that decides whether
     * the run splits, and the chunk count it produces is printed below,
     * never chosen. */
    {
        size_t admission = wfb_split_floor(WFB_SPLIT_WEIGHT);
        size_t outputs = admission ? ((admission + FIR_GRAIN - 1) / FIR_GRAIN) * FIR_GRAIN : 0;
        if (outputs) {
            Work w;
            agreed += input(&w, FIR_TAPS, outputs, fixture_seed(FIR_TAPS, outputs), FIR_GRAIN, 1);
            ++fixtures;
            {
                double *leaf = samples_new(w.n);
                fir_native_direct(w.prefix, w.taps, w.k, w.n, leaf);
                check_samples(w.expected, leaf, w.n, "leaf");
                leaves += w.n;
                free(leaf);
            }
            run(&w, form, width, !generated);
            compared += compare(&w);
            ++calls;
            destroy(&w);
        }
        (void)printf("# fir split fixture: weight=%zu floor=%zu outputs=%zu chunks=%zu\n",
                     (size_t)WFB_SPLIT_WEIGHT, admission, outputs,
                     wfb_split_chunks(outputs, WFB_SPLIT_WEIGHT, width));
    }

    (void)printf("# fir verify: fixtures=%zu calls=%zu compared=%zu leaves=%zu "
                 "oracle_agreed=%zu\n",
                 fixtures, calls, compared, leaves, agreed);
    return compared;
}

static void fir_finish(const char *form) {
    (void)form;
    if (prepared) {
        destroy(&timed);
        prepared = 0;
    }
}

static const char *const fir_forms[] = {"wf",     "wf-seq", "serial",     "static",
                                        "tbb",    "parlay", "rayon-join", "rayon-iter",
                                        NULL};

const wfb_kernel wfb_this_kernel = {
    "fir",
    fir_forms,
    fir_prepare,
    fir_call,
    fir_check,
    fir_verify,
    fir_finish,
    "taps=64 outputs=524288 seed=92821",
    FIR_OUTPUTS,
    WFB_SPLIT_WEIGHT,
    /* This kernel implements every form it lists: nothing is absent by
       construction here. */
    NULL,
};
