/* Complete-result oracle for the formal records program. Correctness and
 * the separate paired performance runner share inputs and expected results. */
#include "oracle.h"
#include <inttypes.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#pragma STDC FP_CONTRACT OFF

const char *const wf_oracle_name = "records";
const char *const wf_oracle_fixture = "records=131072 max_length=255 shape=unicode seed=812381";

#define RC_SEED UINT32_C(812381)
#define RC_VERIFY_MAX_LENGTH ((size_t)129)
#define RC_RECORD_BOUND UINT64_C(1048576)
#define RC_INPUT_BYTE_BOUND UINT64_C(16777216)
/* The owned result is a Box<Array<T>> cell; the adapter hands that cell back
 * as the retained handle and the release row consumes exactly it. */
extern void wf_bench_records(const uint8_t *, uint64_t, const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *, void **);
extern void wf_bench_records_release(void *);
typedef struct {
    uint8_t *data, *held_data;
    uint64_t *offsets, *held_offsets, *expected, *output;
    void *held;
    size_t n, capacity, count;
} Work;

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

static size_t encode(uint32_t c, uint8_t *s) {
    if (c < 0x80) { s[0] = (uint8_t)c; return 1; }
    if (c < 0x800) { s[0] = (uint8_t)(0xc0 | c >> 6); s[1] = (uint8_t)(0x80 | (c & 63)); return 2; }
    if (c < 0x10000) { s[0] = (uint8_t)(0xe0 | c >> 12); s[1] = (uint8_t)(0x80 | ((c >> 6) & 63)); s[2] = (uint8_t)(0x80 | (c & 63)); return 3; }
    s[0] = (uint8_t)(0xf0 | c >> 18); s[1] = (uint8_t)(0x80 | ((c >> 12) & 63));
    s[2] = (uint8_t)(0x80 | ((c >> 6) & 63)); s[3] = (uint8_t)(0x80 | (c & 63)); return 4;
}

static uint32_t random_next(uint32_t *s) {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    return *s;
}

static uint64_t record_value(const Work *w, size_t record,
                             uint64_t (*decode)(const uint8_t *, size_t)) {
    uint64_t lower = w->offsets[record], upper = w->offsets[record + 1];
    if (lower > upper || upper > (uint64_t)w->n) return UINT64_MAX - 1;
    return decode(w->data + lower, (size_t)(upper - lower));
}

static void oracle(Work *w) {
    for (size_t j = 0; j < w->count; ++j) w->expected[j] = record_value(w, j, records_reference);
    memcpy(w->held_data, w->data, w->n);
    memcpy(w->held_offsets, w->offsets, (w->count + 1) * sizeof(uint64_t));
}

static Work input(size_t count, size_t limit, const char *shape, uint32_t seed) {
    Work w;
    size_t n = 0;
    int kind;
    memset(&w, 0, sizeof w);
    if (!(count <= RC_RECORD_BOUND && limit <= 1048576))
        wf_oracle_fail("records: input domain");
    /* The byte bound is held below, on the total this generator actually
     * produced, not here on the worst case `count * limit`. Record lengths are
     * uniform on [0, limit], so the worst case is twice the mean and would
     * reject the shipped fixture on bytes it never writes. */
    w.count = count;
    w.capacity = count * limit;
    if (w.capacity == 0) w.capacity = 1;
    w.data = malloc(w.capacity);
    w.held_data = malloc(w.capacity);
    w.offsets = malloc((count + 1) * sizeof(uint64_t));
    w.held_offsets = malloc((count + 1) * sizeof(uint64_t));
    w.expected = malloc((count ? count : 1) * sizeof(uint64_t));
    if (!w.data || !w.held_data || !w.offsets || !w.held_offsets || !w.expected)
        wf_oracle_fail("records: input allocation");
    kind = !strcmp(shape, "ascii")         ? 0
           : !strcmp(shape, "unicode")     ? 1
           : !strcmp(shape, "error-first") ? 2
           : !strcmp(shape, "error-last")  ? 3
           : !strcmp(shape, "skew")        ? 4
                                           : -1;
    if (kind < 0) wf_oracle_fail("records: input shape");
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
        wf_oracle_fail("records: the generated batch exceeds the source's input bound");
    oracle(&w);
    return w;
}

static void release(Work *w) {
    if (w->held) wf_bench_records_release(w->held);
    w->output = NULL;
    w->held = NULL;
}
static void run(Work *w) {
    uint64_t length = UINT64_MAX;
    wf_bench_records(w->data, w->n, w->offsets, w->count + 1, 0, w->count, &w->output, &length, &w->held);
    if (length != w->count) wf_oracle_fail("records: output extent");
}

static size_t compare(Work *w) {
    if (!w->output && w->count) wf_oracle_fail("records: output pointer");
    for (size_t j = 0; j < w->count; ++j)
        if (w->output[j] != w->expected[j]) {
            (void)fprintf(stderr,
                          "records: record=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n", j,
                          w->expected[j], w->output[j]);
            wf_oracle_fail("records: wrong output bits");
        }
    if (memcmp(w->data, w->held_data, w->n) ||
        memcmp(w->offsets, w->held_offsets, (w->count + 1) * sizeof(uint64_t)))
        wf_oracle_fail("records: input immutability");
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

static uint32_t verify_seed(size_t shape_index, size_t count, size_t max_length) {
    uint32_t seed = RC_SEED + 1009u * (uint32_t)shape_index + 7919u * (uint32_t)count +
                    104729u * (uint32_t)max_length;
    return seed ? seed : 1u; /* xorshift32 has no orbit through zero */
}

static size_t verify_ranges(void) {
    static const size_t counts[] = {3, 7, 33, 257};
    static const uint8_t euro[] = {0xe2, 0x82, 0xac};
    size_t compared = 0;
    for (size_t c = 0; c < sizeof counts / sizeof counts[0]; ++c) {
        Work w = input(counts[c], RC_VERIFY_MAX_LENGTH, "ascii",
                       verify_seed(0, counts[c], RC_VERIFY_MAX_LENGTH));
        w.offsets[counts[c] / 2] = UINT64_MAX; /* descending, and past the input */
        oracle(&w);
        run(&w);
        compared += compare(&w);
        release(&w);
        w.offsets[counts[c] / 2] = 0; /* descending against its predecessor */
        oracle(&w);
        run(&w);
        compared += compare(&w);
        destroy(&w);
    }
    {
        Work w = input(2, RC_VERIFY_MAX_LENGTH, "ascii",
                       verify_seed(0, 2, RC_VERIFY_MAX_LENGTH));
        memcpy(w.data, euro, sizeof euro);
        w.n = sizeof euro;
        w.offsets[0] = 0;
        w.offsets[1] = 1;
        w.offsets[2] = 3;
        oracle(&w);
        run(&w);
        compared += compare(&w);
        release(&w);
        w.offsets[1] = 3;
        w.offsets[2] = 3;
        oracle(&w);
        run(&w);
        compared += compare(&w);
        destroy(&w);
    }
    return compared;
}

size_t wf_oracle_verify(void) {
    static const char *const shapes[] = {"ascii", "unicode", "error-first", "error-last", "skew"};
    static const size_t counts[] = {0, 1, 2, 3, 7, 15, 16, 17, 31, 32, 33, 257, 4097};
    size_t compared = verify_ranges();
    for (size_t s = 0; s < sizeof shapes / sizeof shapes[0]; ++s)
        for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n) {
            Work w = input(counts[n], RC_VERIFY_MAX_LENGTH, shapes[s], verify_seed(s, counts[n], RC_VERIFY_MAX_LENGTH));
            run(&w);
            compared += compare(&w);
            destroy(&w);
        }
    return compared;
}

#ifdef WF_ORACLE_PERFORMANCE
static Work timed;
void wf_oracle_prepare(void) { timed = input(131072, 255, "unicode", RC_SEED); }
size_t wf_oracle_call(void) { run(&timed); return timed.count; }
size_t wf_oracle_check(void) {
    size_t compared = compare(&timed);
    release(&timed);
    return compared;
}
void wf_oracle_finish(void) { destroy(&timed); }
#endif
