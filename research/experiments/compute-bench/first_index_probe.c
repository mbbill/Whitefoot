/* Research-only source-work oracle. There are no timed intervals here.
 * memchr supplies independent predicate outcomes and logical byte positions;
 * neither those positions nor WF's returned counters measure physical reads.
 */
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { MAX_RECORDS = 16, TRACE_SLOTS = 32 };
struct search_trace {
    uint64_t first, end, index, records, bytes;
};
struct record_info { uint64_t hit, bytes; };
_Static_assert(sizeof(struct search_trace) == 5 * sizeof(uint64_t),
               "the host adapter passes five consecutive u64 fields");

extern int wf__floor_run(int, char **);
extern unsigned long wf__par_grants(void);
extern uint64_t wf_probe_search_plain(const unsigned char *, uint64_t,
                                     const uint64_t *, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_probe_search_trace(const unsigned char *, uint64_t,
                                     const uint64_t *, uint64_t, uint64_t, uint64_t,
                                     struct search_trace *, uint64_t);

static const char *image, *workers;
static int inject_bad_trace;
static uint64_t cases, trace_fields, input_bytes;
static unsigned long plain_steals, trace_steals;

static uint64_t minimum(uint64_t a, uint64_t b) { return a < b ? a : b; }

static struct record_info inspect_record(const unsigned char *data,
                                        uint64_t first, uint64_t end) {
    const unsigned char *found = memchr(data + first, 0, (size_t)(end - first));
    struct record_info info = {
        found != NULL, found ? (uint64_t)(found - (data + first)) + 1 : end - first
    };
    return info;
}

/* A useful native sequential first-index search, not a native wave emulator. */
static struct search_trace native_first(const unsigned char *data,
                                       const uint64_t *offsets, uint64_t count) {
    struct search_trace result = { 0, count, count, 0, 0 };
    for (uint64_t i = 0; i < count; ++i) {
        struct record_info info = inspect_record(data, offsets[i], offsets[i + 1]);
        ++result.records;
        result.bytes += info.bytes;
        if (info.hit) { result.index = i; return result; }
    }
    return result;
}

static struct search_trace expected_prefix(const struct record_info *info,
                                           uint64_t first, uint64_t end,
                                           uint64_t count) {
    struct search_trace result = { first, end, count, 0, 0 };
    for (uint64_t i = first; i < end; ++i) {
        ++result.records;
        result.bytes += info[i].bytes;
        if (info[i].hit) { result.index = i; break; }
    }
    return result;
}

/* The trace expectation comes from independent per-record outcomes, with the
 * proposed partition defining which prefixes every completed wave must own. */
static void expected_trace(const struct record_info *info, uint64_t count,
                           unsigned mode, struct search_trace *trace) {
    memset(trace, 0xff, TRACE_SLOTS * sizeof(*trace));
    if (mode == 0) { trace[0] = expected_prefix(info, 0, count, count); return; }
    uint64_t width = mode == 2 ? 1 : 4;
    for (uint64_t first = 0; first < count;) {
        uint64_t end = minimum(count, first + width), best = count;
        if (mode == 3) {
            for (uint64_t block = 0; block < 2; ++block) {
                uint64_t lower = minimum(count, first + 2 * block);
                uint64_t upper = minimum(count, lower + 2);
                trace[first + block] = expected_prefix(info, lower, upper, count);
                best = minimum(best, trace[first + block].index);
            }
        } else {
            for (uint64_t i = first; i < end; ++i) {
                trace[i] = expected_prefix(info, i, i + 1, count);
                best = minimum(best, trace[i].index);
            }
        }
        if (best < count) return;
        first = end;
        if (mode == 2) width = minimum(16, width * 2);
    }
}

static void failure(const char *name, unsigned mode, const char *kind,
                    uint64_t position, uint64_t expected, uint64_t actual) {
    fprintf(stderr, "FAIL\t%s\t%s\tcase=%" PRIu64 "\t%s\tmode=%u\t%s\t"
                    "position=%" PRIu64 "\texpected=%" PRIu64 "\tactual=%" PRIu64 "\n",
            image, workers, cases, name, mode, kind, position, expected, actual);
    exit(2);
}

static void check_case(const char *name, const uint64_t *lengths, uint64_t count,
                       uint64_t mask, unsigned marker, uint64_t control_size) {
    uint64_t offsets[MAX_RECORDS + 1] = { 0 }, saved_offsets[MAX_RECORDS + 1];
    struct record_info info[MAX_RECORDS];
    if (count > MAX_RECORDS) exit(3);
    for (uint64_t i = 0; i < count; ++i) offsets[i + 1] = offsets[i] + lengths[i];
    uint64_t size = offsets[count];
    if (size > UINT64_C(16777216)) exit(3);
    unsigned char *data = malloc(size ? (size_t)size : 1);
    unsigned char *saved = malloc(size ? (size_t)size : 1);
    if (!data || !saved) exit(3);
    memset(data, 0x7f, (size_t)size);
    for (uint64_t i = 0; i < count; ++i) {
        if ((mask >> i) & 1) {
            if (lengths[i] == 0) exit(3);
            uint64_t at = marker == 0 ? 0 : marker == 1 ? lengths[i] / 2 : lengths[i] - 1;
            data[offsets[i] + at] = 0;
        }
        info[i] = inspect_record(data, offsets[i], offsets[i + 1]);
    }
    memcpy(saved, data, (size_t)size);
    memcpy(saved_offsets, offsets, sizeof(offsets));
    struct search_trace native = native_first(data, offsets, count);
    ++cases;
    for (unsigned mode = 0; mode < 4; ++mode) {
        struct search_trace actual[TRACE_SLOTS], expected[TRACE_SLOTS];
        memset(actual, 0xff, sizeof(actual));
        expected_trace(info, count, mode, expected);
        unsigned long before = wf__par_grants();
        uint64_t plain = wf_probe_search_plain(data, size, offsets, count + 1, count, mode);
        unsigned long plain_delta = wf__par_grants() - before;
        before = wf__par_grants();
        uint64_t diagnostic = wf_probe_search_trace(data, size, offsets, count + 1,
                                                    count, mode, actual, TRACE_SLOTS);
        unsigned long trace_delta = wf__par_grants() - before;
        plain_steals += plain_delta;
        trace_steals += trace_delta;
        if (plain != native.index) failure(name, mode, "plain-index", 0, native.index, plain);
        if (diagnostic != native.index) failure(name, mode, "trace-index", 0, native.index, diagnostic);
        if (inject_bad_trace) { actual[0].bytes ^= 1; inject_bad_trace = 0; }
        for (uint64_t slot = 0; slot < TRACE_SLOTS; ++slot) {
            uint64_t want[5], got[5];
            memcpy(want, &expected[slot], sizeof(want));
            memcpy(got, &actual[slot], sizeof(got));
            for (unsigned field = 0; field < 5; ++field) {
                ++trace_fields;
                if (want[field] != got[field])
                    failure(name, mode, "trace-field", slot * 5 + field, want[field], got[field]);
            }
        }
        if (memcmp(data, saved, (size_t)size) || memcmp(offsets, saved_offsets, sizeof(offsets)))
            failure(name, mode, "input-changed", 0, 0, 1);
        input_bytes += size + sizeof(offsets);
        if (control_size) {
            uint64_t records = 0, bytes = 0;
            for (unsigned slot = 0; slot < TRACE_SLOTS; ++slot) {
                if (actual[slot].first == UINT64_MAX) continue;
                records += actual[slot].records;
                bytes += actual[slot].bytes;
            }
            printf("control\t%s\t%s\t%s\tT=%" PRIu64 "\tmode=%u\tindex=%" PRIu64
                   "\trecords=%" PRIu64 "\tbytes=%" PRIu64 "\tplain-steals=%lu\ttrace-steals=%lu\tprefixes=",
                   image, workers, name, control_size, mode, diagnostic, records, bytes,
                   plain_delta, trace_delta);
            for (unsigned slot = 0; slot < TRACE_SLOTS; ++slot) {
                if (actual[slot].first != UINT64_MAX)
                    printf("[%" PRIu64 ",%" PRIu64 "):%" PRIu64 "/%" PRIu64 ";",
                           actual[slot].first, actual[slot].end,
                           actual[slot].records, actual[slot].bytes);
            }
            putchar('\n');
        }
    }
    free(saved);
    free(data);
}

int wf__main_body(int argc, char **argv) {
    if (argc < 2 || argc > 3) return 3;
    image = argv[1];
    workers = getenv("WF_WORKERS");
    if (!workers) return 3;
    if (argc == 3) {
        if (strcmp(argv[2], "self-control")) return 3;
        const uint64_t lengths[4] = { 1, 3, 3, 1 };
        inject_bad_trace = 1;
        check_case("intentional-bad-trace", lengths, 4, 1, 0, 0);
        return 4; /* Reaching here means the expected rejection did not occur. */
    }
    for (uint64_t count = 0; count <= MAX_RECORDS; ++count) {
        for (uint64_t hit = 0; hit <= count; ++hit) {
            for (unsigned family = 0; family < 4; ++family) {
                uint64_t lengths[MAX_RECORDS] = { 0 };
                for (uint64_t i = 0; i < count; ++i) {
                    lengths[i] = family == 0 ? 1 : family == 1 ? (i % 4 == 0 ? 0 : i % 7 + 1)
                                                         : family == 2 ? (i % 2 ? 17 : 3) : 65;
                    if (i == hit && lengths[i] == 0) lengths[i] = 1;
                }
                uint64_t mask = hit < count ? UINT64_C(1) << hit : 0;
                check_case("single-or-absent", lengths, count, mask, family % 3, 0);
            }
        }
    }
    for (uint64_t count = 0; count <= 8; ++count) {
        for (uint64_t mask = 0; mask < (UINT64_C(1) << count); ++mask) {
            uint64_t lengths[MAX_RECORDS] = { 0 };
            for (uint64_t i = 0; i < count; ++i) lengths[i] = 3 + i % 5;
            check_case("all-hit-masks", lengths, count, mask, (unsigned)(mask % 3), 0);
        }
    }
    const uint64_t sizes[2] = { 1, 65536 };
    for (unsigned i = 0; i < 2; ++i) {
        uint64_t t = sizes[i];
        const uint64_t other_helper[4] = { 1, t, t, 1 };
        const uint64_t local_tails[4] = { 1, t, 1, t };
        const uint64_t geometric[3] = { 1, 1, t };
        check_case("other-helper", other_helper, 4, 1, 0, t);
        check_case("local-tails", local_tails, 4, 5, 0, t);
        check_case("geometric-final-wave", geometric, 3, 2, 0, t);
    }
    printf("summary\t%s\t%s\tcases=%" PRIu64 "\tindex-comparisons=%" PRIu64
           "\ttrace-fields=%" PRIu64 "\tinput-bytes=%" PRIu64
           "\tplain-steals=%lu\ttrace-steals=%lu\n",
           image, workers, cases, cases * 8, trace_fields, input_bytes,
           plain_steals, trace_steals);
    return 0;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
