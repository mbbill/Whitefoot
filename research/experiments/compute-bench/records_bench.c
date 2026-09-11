/* Serves compute-bench: the records kernel -- variable-length UTF-8 record
 * batches, irregular per item. It owns the generator and its five shapes, the
 * independent decoder oracle, the scalar state-machine leaf, the dispatch over
 * every form, and the per-call checks.
 *
 * Adapted from the research bundle's records.c (generator, shapes, encode),
 * records_oracle.c (`records_reference`, copied verbatim) and records_native.c
 * (`records_state`, copied verbatim). The two are structurally unlike each
 * other on purpose: the oracle decodes the integer code point and tests the
 * minimum encoding length, the 0x10FFFF bound and the surrogate range, while
 * the leaf is the same automaton as the Whitefoot source -- the
 * remaining/lower/upper triple with the 194/223/239/244 boundaries and the
 * 224 -> lower=160, 237 -> upper=159, 240 -> lower=144, 244 -> upper=143
 * adjustments. Cut: `records_word`, which is a different algorithm and would
 * change what is compared rather than only the speed; the bundle's `wf-runtime`
 * row, which was a C adapter over the frozen research runtime and so was never
 * a Whitefoot number; its stored qualification counts (`calls != 4595603`,
 * `batch_calls != 342`); and its alarm() calls.
 *
 * THE TIMED INTERVAL IS LOAD-BEARING FOR FAIRNESS and is identical for every
 * implementation of this kernel: allocate the output buffer, zero-fill it,
 * compute, join every participant. Release and free are outside it.
 * Allocation and the zero-fill are inside because Whitefoot's
 * buffer_new(count, 0) allocates and zero-fills, so every reference calls
 * malloc plus memset inside the interval. A re-cut that lets the native path
 * allocate uninitialized compares allocators, not schedulers.
 *
 * The UINT64_MAX poison that detects an omitted write is therefore not in the
 * timed path, where it would be a pass over the output that the Whitefoot form
 * does not pay: it runs in `verify`, which is never timed, and it is applied
 * after the zero-fill so that it is the state the callbacks actually see. In
 * the timed path the zero-fill is the omission witness, with one honest limit
 * this kernel has and Mandelbrot does not: an empty record's expected value is
 * 0, so an unwritten element that happens to belong to one of the timed
 * fixture's 474 empty records would read as correct. Every other element of
 * every chunk is nonzero, so no omitted callback, chunk or lane can survive
 * check(); only a single omitted write landing exactly on an empty record
 * could, and `verify` closes even that for every form and every shape. */
#include "backend.h"
#include "harness.h"
#include "records_split.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The timed fixture. `unicode` is the shape that pays for every record:
 * `error-first` exits at the first byte and collapses toward clock
 * resolution, and `skew` collapses the whole batch to about 80 KB. Both stay
 * in the verify set.
 *
 * 131,072 records is the size constant. At the emitted weight 812 the work
 * divisor is ceil(1,200,000 / 812) = 1,478, so the affordable chunk count is
 * 131,072 / 1,478 = 88 and min(16 * lanes, 88) rounds down to 64 chunks at
 * width 4 and at width 8 -- eight chunks per lane at the highest width a table
 * records here. The next doubling would reach 128 chunks at width 8 but costs
 * about 62 ms of wf-seq and leaves the [5 ms, 60 ms] window.
 *
 * RC_MAX_LENGTH is 255 rather than the specification's 256, and this is the
 * one place the difference matters. `summarize_records` carries
 * `requires input_count <= 16777216`, and at 131,072 records seeded 812381 a
 * limit of 256 generates 16,799,739 bytes -- 22,523 over the source's own
 * precondition, which the C caller is the one that has to establish. 255 is
 * the largest value of this constant for which the shipped fixture satisfies
 * it (16,698,303 bytes). Nothing else moves: the size constant, and therefore
 * the chunk count, is untouched. prepare() re-derives the byte total and
 * fails on it rather than trusting this paragraph. */
#define RC_RECORDS ((size_t)131072)
#define RC_MAX_LENGTH ((size_t)255)
#define RC_SHAPE "unicode"
#define RC_SEED ((uint32_t)812381)
/* Callback grain: 16 records, giving 8,192 chunks at the default size against
 * the WF program's 64. It is the research bundle's `group16` setting, whose
 * `group4`/`group16` screen moved the result by about a factor of three in
 * either direction depending on shape, so the number matters; it is not a
 * per-host optimum and no per-host calibration happens anywhere here. */
#define RC_GRAIN ((size_t)16)
/* The verify grid's record length limit, from the research bundle's own
 * fixture grid. Verify fixtures are small enough that no shape can approach
 * the source's input bound. */
#define RC_VERIFY_MAX_LENGTH ((size_t)129)

/* The two preconditions of `summarize_records` that a caller can violate:
 * `requires input_count <= 16777216` on the byte buffer and
 * `requires end <= 1048576` on the record index. They are proof-only source
 * evidence, erased before lowering, so nothing checks them at run time and
 * the caller is what has to hold them. These two constants are where this
 * file holds them. */
#define RC_INPUT_BYTE_BOUND ((uint64_t)16777216)
#define RC_RECORD_BOUND ((uint64_t)1048576)

extern void wf_bench_records_par(const uint8_t *, uint64_t, const uint64_t *, uint64_t,
                                 uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_records_par_release(uint64_t *, uint64_t);
extern void wf_bench_records_seq(const uint8_t *, uint64_t, const uint64_t *, uint64_t,
                                 uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_records_seq_release(uint64_t *, uint64_t);

/* The independent oracle, copied verbatim from the research bundle's
 * records_oracle.c. It is a decoder, not an automaton: it accumulates the
 * scalar value and tests the minimum encoding length, the 0x10FFFF ceiling
 * and the surrogate range. This is outside timing. */
static uint64_t records_reference(const uint8_t *s, size_t n) {
    size_t i = 0;
    uint64_t count = 0;
    while (i < n) {
        uint8_t b = s[i++];
        uint32_t v;
        unsigned more;
        uint32_t minimum;
        if (b < 128) { ++count; continue; }
        if ((b & 0xe0) == 0xc0) { v = b & 0x1f; more = 1; minimum = 0x80; }
        else if ((b & 0xf0) == 0xe0) { v = b & 0x0f; more = 2; minimum = 0x800; }
        else if ((b & 0xf8) == 0xf0) { v = b & 0x07; more = 3; minimum = 0x10000; }
        else return UINT64_MAX;
        if (n - i < more) return UINT64_MAX;
        for (unsigned j = 0; j < more; ++j) {
            b = s[i++];
            if ((b & 0xc0) != 0x80) return UINT64_MAX;
            v = (v << 6) | (b & 0x3f);
        }
        if (v < minimum || v > 0x10ffff || (v >= 0xd800 && v <= 0xdfff)) return UINT64_MAX;
        ++count;
    }
    return count;
}

/* The scalar kernel body, copied verbatim from the research bundle's
 * records_native.c. It is the same automaton as the Whitefoot leaf. */
static uint64_t records_state(const uint8_t *s, size_t n) {
    uint64_t count = 0;
    unsigned remaining = 0, lower = 128, upper = 191;
    for (size_t i = 0; i < n; ++i) {
        unsigned b = s[i];
        if (remaining == 0) {
            ++count;
            if (b < 128) continue;
            if (b < 194) return UINT64_MAX;
            if (b <= 223) remaining = 1;
            else if (b <= 239) {
                remaining = 2;
                if (b == 224) lower = 160;
                if (b == 237) upper = 159;
            } else if (b <= 244) {
                remaining = 3;
                if (b == 240) lower = 144;
                if (b == 244) upper = 143;
            } else return UINT64_MAX;
        } else {
            if (b < lower || b > upper) return UINT64_MAX;
            --remaining;
            lower = 128; upper = 191;
        }
    }
    return remaining ? UINT64_MAX : count;
}

/* Copied verbatim from the research bundle's records.c. */
static size_t encode(uint32_t c, uint8_t *s) {
    if (c < 0x80) { s[0] = (uint8_t)c; return 1; }
    if (c < 0x800) { s[0] = (uint8_t)(0xc0 | c >> 6); s[1] = (uint8_t)(0x80 | (c & 63)); return 2; }
    if (c < 0x10000) { s[0] = (uint8_t)(0xe0 | c >> 12); s[1] = (uint8_t)(0x80 | ((c >> 6) & 63)); s[2] = (uint8_t)(0x80 | (c & 63)); return 3; }
    s[0] = (uint8_t)(0xf0 | c >> 18); s[1] = (uint8_t)(0x80 | ((c >> 12) & 63));
    s[2] = (uint8_t)(0x80 | ((c >> 6) & 63)); s[3] = (uint8_t)(0x80 | (c & 63)); return 4;
}

/* Where the last call's output buffer came from, so that release() hands it
 * back to whoever owns it. The two Whitefoot modules have separate release
 * entry points and a buffer from one must not reach the other. */
#define RC_FROM_MALLOC 0
#define RC_FROM_PAR 1
#define RC_FROM_SEQ 2

typedef struct {
    uint8_t *data, *held_data;
    uint64_t *offsets, *held_offsets;
    uint64_t *expected;
    uint64_t *output;
    int output_source;
    size_t n, capacity, count, grain;
} Work;

static uint32_t random_next(uint32_t *s) {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    return *s;
}

/* One record through the rules the Whitefoot source states: a descending or
 * out-of-input range is UINT64_MAX-1, invalid UTF-8 is UINT64_MAX, and a
 * valid record is its Unicode scalar count. `decode` selects the oracle or
 * the scalar leaf; the range rule is above both and is written once. */
static uint64_t record_value(const Work *w, size_t record,
                             uint64_t (*decode)(const uint8_t *, size_t)) {
    uint64_t lower = w->offsets[record], upper = w->offsets[record + 1];
    if (lower > upper || upper > (uint64_t)w->n) return UINT64_MAX - 1;
    return decode(w->data + lower, (size_t)(upper - lower));
}

/* Recomputes the expected vector and refreshes the immutability copies. It
 * runs after generation and again after any fixture edits an offset, so the
 * oracle is never stale. Never timed. */
static void oracle(Work *w) {
    for (size_t j = 0; j < w->count; ++j) w->expected[j] = record_value(w, j, records_reference);
    memcpy(w->held_data, w->data, w->n);
    memcpy(w->held_offsets, w->offsets, (w->count + 1) * sizeof(uint64_t));
}

/* The generator and its five shapes, copied exactly from the research
 * bundle's records.c: xorshift32 from the seed, per-record length
 * `rand % (limit + 1)`, base bytes `32 + ((i + j) % 95)`, `unicode`
 * overwriting each aligned four-byte group with U+10000 + (j % 0xFFFFF),
 * `error-first`/`error-last` planting one 0xff, and `skew` giving the limit
 * at `j % 1024 == 0` and one byte otherwise. */
static Work input(size_t count, size_t limit, const char *shape, uint32_t seed, size_t grain) {
    Work w;
    size_t n = 0;
    int kind;
    memset(&w, 0, sizeof w);
    if (!(count <= RC_RECORD_BOUND && limit <= 1048576 && grain >= 1 && grain <= 1048576))
        wfb_fail("records: input domain");
    /* The byte bound is held below, on the total this generator actually
     * produced, not here on the worst case `count * limit`. Record lengths are
     * uniform on [0, limit], so the worst case is twice the mean and would
     * reject the shipped fixture on bytes it never writes. */
    w.count = count;
    w.grain = grain;
    w.capacity = count * limit;
    if (w.capacity == 0) w.capacity = 1;
    w.data = malloc(w.capacity);
    w.held_data = malloc(w.capacity);
    w.offsets = malloc((count + 1) * sizeof(uint64_t));
    w.held_offsets = malloc((count + 1) * sizeof(uint64_t));
    w.expected = malloc((count ? count : 1) * sizeof(uint64_t));
    if (!w.data || !w.held_data || !w.offsets || !w.held_offsets || !w.expected)
        wfb_fail("records: input allocation");
    kind = !strcmp(shape, "ascii")         ? 0
           : !strcmp(shape, "unicode")     ? 1
           : !strcmp(shape, "error-first") ? 2
           : !strcmp(shape, "error-last")  ? 3
           : !strcmp(shape, "skew")        ? 4
                                           : -1;
    if (kind < 0) wfb_fail("records: input shape");
    w.offsets[0] = 0;
    for (size_t j = 0; j < count; ++j) {
        size_t length = limit ? random_next(&seed) % (limit + 1) : 0;
        if (kind == 4) length = j % 1024 == 0 ? limit : (limit ? 1 : 0);
        for (size_t i = 0; i < length; ++i) w.data[n + i] = (uint8_t)(32 + ((i + j) % 95));
        if (kind == 1)
            for (size_t i = 0; i + 3 < length; i += 4)
                encode(0x10000 + (uint32_t)(j % 0xfffff), w.data + n + i);
        if (kind == 2 && length) w.data[n] = 0xff;
        if (kind == 3 && length) w.data[n + length - 1] = 0xff;
        n += length;
        w.offsets[j + 1] = n;
    }
    w.n = n;
    /* The caller's half of `requires input_count <= 16777216`, re-derived from
     * the bytes this generator actually produced rather than assumed from the
     * constants above. */
    if ((uint64_t)w.n > RC_INPUT_BYTE_BOUND)
        wfb_fail("records: the generated batch exceeds the source's input bound");
    oracle(&w);
    return w;
}

static void native_chunk(void *opaque, size_t chunk) {
    Work *w = opaque;
    size_t first = chunk * w->grain, end = first + w->grain;
    if (end > w->count) end = w->count;
    for (size_t j = first; j < end; ++j) w->output[j] = record_value(w, j, records_state);
}

static void release(Work *w) {
    if (!w->output) return;
    if (w->output_source == RC_FROM_PAR) wf_bench_records_par_release(w->output, w->count);
    else if (w->output_source == RC_FROM_SEQ) wf_bench_records_seq_release(w->output, w->count);
    else free(w->output);
    w->output = NULL;
}

/* One complete call on `w`. This is the whole timed interval and nothing
 * else. `poison` is never set on the timed path; see the file comment. */
static void run(Work *w, const char *form, unsigned width, int poison) {
    if (!strcmp(form, "wf") || !strcmp(form, "wf-seq")) {
        uint64_t length = UINT64_MAX;
        uint64_t *out = NULL;
        int sequential = !strcmp(form, "wf-seq");
        if (sequential)
            wf_bench_records_seq(w->data, (uint64_t)w->n, w->offsets, (uint64_t)w->count + 1,
                                 0, (uint64_t)w->count, &out, &length);
        else
            wf_bench_records_par(w->data, (uint64_t)w->n, w->offsets, (uint64_t)w->count + 1,
                                 0, (uint64_t)w->count, &out, &length);
        if (length != (uint64_t)w->count) wfb_fail("records: generated length");
        w->output = out;
        w->output_source = sequential ? RC_FROM_SEQ : RC_FROM_PAR;
    } else {
        const wfb_backend *backend = wfb_backend_named(form);
        size_t bytes = (w->count ? w->count : 1) * sizeof(uint64_t);
        if (!backend || !backend->map) wfb_fail("records: no such form");
        w->output = malloc(bytes);
        if (!w->output) wfb_fail("records: output allocation");
        w->output_source = RC_FROM_MALLOC;
        memset(w->output, 0, w->count * sizeof(uint64_t));
        /* Verify only, and after the zero-fill so that it is the state the
         * callbacks actually see. The timed path never reaches this line. */
        if (poison) memset(w->output, 0xff, w->count * sizeof(uint64_t));
        backend->map(backend, width, (w->count + w->grain - 1) / w->grain, native_chunk, w);
    }
}

static size_t compare(Work *w) {
    if (!w->output && w->count) wfb_fail("records: output pointer");
    for (size_t j = 0; j < w->count; ++j)
        if (w->output[j] != w->expected[j]) {
            (void)fprintf(stderr,
                          "records: record=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n", j,
                          w->expected[j], w->output[j]);
            wfb_fail("records: wrong output bits");
        }
    if (memcmp(w->data, w->held_data, w->n) ||
        memcmp(w->offsets, w->held_offsets, (w->count + 1) * sizeof(uint64_t)))
        wfb_fail("records: input immutability");
    return w->count;
}

static void destroy(Work *w) {
    release(w);
    free(w->data);
    free(w->held_data);
    free(w->offsets);
    free(w->held_offsets);
    free(w->expected);
    memset(w, 0, sizeof *w);
}

/* --------------------------------------------------------- the leaf sweep -- */

/* The exhaustive leaf qualification, adapted from the research bundle's
 * check_leaf: every 0x0..0x10FFFF encoding, every nonempty truncation of each,
 * all one- and two-byte combinations with two continuation extensions, and
 * 10,000 xorshift inputs, each inside a 0xff canary ring so that a leaf which
 * read or wrote outside its slice is caught. It runs once per verify target,
 * at form=serial and width=1, because it is the serial form's leaf that it
 * qualifies and because it costs several million calls. Its call count is
 * printed and never asserted: the bundle's `if (calls != 4595603) abort();` is
 * a stored count and is not carried. */
static size_t sweep_calls;

static void sweep_case(const uint8_t *s, size_t n, uint64_t known) {
    uint8_t held[272];
    uint64_t expected, actual;
    if (n > 256) wfb_fail("records: sweep length");
    memset(held, 0xff, sizeof held);
    memcpy(held + 7, s, n);
    expected = records_reference(held + 7, n);
    if (known != UINT64_MAX - 1 && expected != known) wfb_fail("records: sweep oracle");
    actual = records_state(held + 7, n);
    ++sweep_calls;
    if (expected != actual) {
        (void)fprintf(stderr,
                      "records: sweep call=%zu n=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n",
                      sweep_calls, n, expected, actual);
        wfb_fail("records: the leaf disagrees with the oracle");
    }
    for (size_t i = 0; i < sizeof held; ++i)
        if (held[i] != (i >= 7 && i < n + 7 ? s[i - 7] : (uint8_t)0xff))
            wfb_fail("records: sweep canary");
}

static size_t leaf_sweep(void) {
    uint8_t s[256];
    uint32_t rng = 0x19e18ab3;
    memset(s, 0, sizeof s);
    sweep_case(s, 0, 0);
    for (unsigned a = 0; a < 256; ++a) {
        s[0] = (uint8_t)a;
        sweep_case(s, 1, UINT64_MAX - 1);
        for (unsigned b = 0; b < 256; ++b) {
            s[1] = (uint8_t)b;
            sweep_case(s, 2, UINT64_MAX - 1);
            s[2] = 0x80;
            sweep_case(s, 3, UINT64_MAX - 1);
            s[2] = 0xbf;
            s[3] = 0x80;
            sweep_case(s, 4, UINT64_MAX - 1);
        }
    }
    for (uint32_t c = 0; c <= 0x10ffff; ++c) {
        size_t n = encode(c, s);
        sweep_case(s, n, c >= 0xd800 && c <= 0xdfff ? UINT64_MAX : 1);
        for (size_t cut = 1; cut < n; ++cut) sweep_case(s, cut, UINT64_MAX);
    }
    for (unsigned j = 0; j < 10000; ++j) {
        for (size_t i = 0; i < sizeof s; ++i) s[i] = (uint8_t)random_next(&rng);
        sweep_case(s, j % 257, UINT64_MAX - 1);
    }
    memset(s, 0, sizeof s);
    sweep_case(s, sizeof s, sizeof s);
    for (size_t i = 0; i < sizeof s; i += 4) {
        s[i] = 0xf4;
        s[i + 1] = 0x8f;
        s[i + 2] = 0xbf;
        s[i + 3] = 0xbf;
    }
    sweep_case(s, sizeof s, sizeof s / 4);
    return sweep_calls;
}

/* ------------------------------------------------------------ the kernel -- */

static Work timed;
static int prepared;

/* The verify grid's seed is derived from (shape, count, max_length)
 * explicitly. The research bundle's `seed = 812381 + cell * 7919` was an
 * ordinal, so adding or removing one cell silently re-seeded every later one.
 * The grain is deliberately not in it: the four grains must see the same
 * bytes, because what they compare is four decompositions of one input. */
static uint32_t verify_seed(size_t shape_index, size_t count, size_t max_length) {
    uint32_t seed = RC_SEED + 1009u * (uint32_t)shape_index + 7919u * (uint32_t)count +
                    104729u * (uint32_t)max_length;
    return seed ? seed : 1u; /* xorshift32 has no orbit through zero */
}

static void rc_prepare(unsigned width) {
    (void)width;
    if (prepared) return;
    timed = input(RC_RECORDS, RC_MAX_LENGTH, RC_SHAPE, RC_SEED, RC_GRAIN);
    prepared = 1;
}

static size_t rc_call(const char *form, unsigned width) {
    timed.grain = RC_GRAIN;
    run(&timed, form, width, 0);
    return timed.count;
}

/* Outside every timed interval. It compares, then releases the call's buffer,
 * so the next call allocates from the same state this one did. */
static size_t rc_check(void) {
    size_t compared = compare(&timed);
    release(&timed);
    return compared;
}

/* The malformed-range verdict -- UINT64_MAX-1 for a descending range or one
 * that runs past the input -- is reachable from no generated shape, because
 * every generated offset vector ascends and ends at the byte count. These
 * fixtures reach it, and the last two prove that two truncated neighbours do
 * not combine into one valid sequence across a record boundary. They are
 * additional to the specification's grid, which never exercises what the
 * source says about a bad range. */
static size_t verify_ranges(const char *form, unsigned width, int poison, size_t *calls) {
    static const size_t counts[] = {3, 7, 33, 257};
    static const uint8_t euro[] = {0xe2, 0x82, 0xac};
    size_t compared = 0;
    for (size_t c = 0; c < sizeof counts / sizeof counts[0]; ++c) {
        Work w = input(counts[c], RC_VERIFY_MAX_LENGTH, "ascii",
                       verify_seed(0, counts[c], RC_VERIFY_MAX_LENGTH), 4);
        w.offsets[counts[c] / 2] = UINT64_MAX; /* descending, and past the input */
        oracle(&w);
        run(&w, form, width, poison);
        compared += compare(&w);
        ++*calls;
        release(&w);
        w.offsets[counts[c] / 2] = 0; /* descending against its predecessor */
        oracle(&w);
        run(&w, form, width, poison);
        compared += compare(&w);
        ++*calls;
        destroy(&w);
    }
    {
        Work w = input(2, RC_VERIFY_MAX_LENGTH, "ascii",
                       verify_seed(0, 2, RC_VERIFY_MAX_LENGTH), 1);
        memcpy(w.data, euro, sizeof euro);
        w.n = sizeof euro;
        w.offsets[0] = 0;
        w.offsets[1] = 1;
        w.offsets[2] = 3;
        oracle(&w);
        run(&w, form, width, poison);
        compared += compare(&w);
        ++*calls;
        release(&w);
        w.offsets[1] = 3;
        w.offsets[2] = 3;
        oracle(&w);
        run(&w, form, width, poison);
        compared += compare(&w);
        ++*calls;
        destroy(&w);
    }
    return compared;
}

static size_t rc_verify(const char *form, unsigned width) {
    static const char *const shapes[] = {"ascii", "unicode", "error-first", "error-last", "skew"};
    static const size_t counts[] = {0, 1, 2, 3, 7, 15, 16, 17, 31, 32, 33, 257, 4097};
    static const size_t grains[] = {1, 4, 16, 64};
    size_t compared = 0, leaves = 0, calls = 0, sweep = 0;
    int generated = !strcmp(form, "wf") || !strcmp(form, "wf-seq");

    if (!strcmp(form, "serial") && width == 1) sweep = leaf_sweep();

    for (size_t s = 0; s < sizeof shapes / sizeof shapes[0]; ++s)
        for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
            for (size_t g = 0; g < sizeof grains / sizeof grains[0]; ++g) {
                Work w = input(counts[n], RC_VERIFY_MAX_LENGTH, shapes[s],
                               verify_seed(s, counts[n], RC_VERIFY_MAX_LENGTH), grains[g]);
                for (size_t j = 0; j < w.count; ++j) {
                    if (record_value(&w, j, records_state) != w.expected[j])
                        wfb_fail("records: leaf");
                    ++leaves;
                }
                /* The UINT64_MAX poison lives here, where nothing is timed: an
                 * element the form never writes reads UINT64_MAX rather than
                 * inheriting a plausible value, which matters for this kernel
                 * because an empty record's correct value is zero. */
                run(&w, form, width, !generated);
                compared += compare(&w);
                ++calls;
                destroy(&w);
            }

    compared += verify_ranges(form, width, !generated, &calls);

    (void)printf("# records verify: sweep_calls=%zu leaves=%zu calls=%zu compared=%zu\n", sweep,
                 leaves, calls, compared);
    return compared;
}

static void rc_finish(const char *form) {
    (void)form;
    if (prepared) {
        destroy(&timed);
        prepared = 0;
    }
}

static const char *const rc_forms[] = {"wf",     "wf-seq", "serial",     "static",
                                       "tbb",    "parlay", "rayon-join", "rayon-iter",
                                       NULL};

const wfb_kernel wfb_this_kernel = {
    "records",
    rc_forms,
    rc_prepare,
    rc_call,
    rc_check,
    rc_verify,
    rc_finish,
    "records=131072 max_length=255 shape=unicode seed=812381",
    RC_RECORDS,
    WFB_SPLIT_WEIGHT,
    /* This kernel implements every form it lists: nothing is absent by
       construction here. */
    NULL,
};
