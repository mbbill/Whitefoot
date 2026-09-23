/* Research-only source-work oracle. There are no timed intervals here.
 * memchr supplies independent predicate outcomes and logical byte positions;
 * neither those positions nor WF's returned counters measure physical reads.
 */
#include <inttypes.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef WF_FIRST_INDEX_OBSERVED
#define WF_FIRST_INDEX_OBSERVED 0
#endif

enum { MAX_RECORDS = 16, TRACE_SLOTS = 32, EVENT_SLOTS = 8 };
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
static int inject_bad_trace, pair_only, observe_predicates, fixed_controls;
static uint64_t cases, comparisons, trace_fields, input_bytes, native_waves;
static unsigned long plain_steals, trace_steals, loop_steals;
static unsigned actualization_failures;

struct predicate_event {
    const unsigned char *data;
    uint64_t length;
    pthread_t thread;
    unsigned returning;
};
static struct predicate_event events[EVENT_SLOTS];
static _Atomic unsigned event_count;
static _Atomic int observing;

/* A slot has one writer. The caller reads slots only after the WF call has
 * joined every offered scan; reserving an ordinal does not publish its data.
 * The total event order observes overlap without a clock or a wait. */
void wf_probe_predicate_event(const unsigned char *data, uint64_t length,
                              uint64_t returning) {
    if (!atomic_load_explicit(&observing, memory_order_acquire)) return;
    unsigned slot = atomic_fetch_add_explicit(&event_count, 1, memory_order_seq_cst);
    if (slot < EVENT_SLOTS)
        events[slot] = (struct predicate_event){ data, length, pthread_self(),
                                               (unsigned)returning };
}

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
        if (mode == 3 || mode == 4) {
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

static void compare_trace(const char *name, unsigned mode, const char *kind,
                          const struct search_trace *expected,
                          const struct search_trace *actual) {
    for (unsigned slot = 0; slot < TRACE_SLOTS; ++slot) {
        uint64_t want[5], got[5];
        memcpy(want, &expected[slot], sizeof(want));
        memcpy(got, &actual[slot], sizeof(got));
        for (unsigned field = 0; field < 5; ++field) {
            ++trace_fields;
            if (want[field] != got[field])
                failure(name, mode, kind, slot * 5 + field, want[field], got[field]);
        }
    }
}

struct native_block {
    const unsigned char *data;
    const uint64_t *offsets;
    struct search_trace result;
};

static void *native_block_run(void *argument) {
    struct native_block *block = argument;
    for (uint64_t i = block->result.first; i < block->result.end; ++i) {
        struct record_info info = inspect_record(block->data, block->offsets[i],
                                                block->offsets[i + 1]);
        ++block->result.records;
        block->result.bytes += info.bytes;
        if (info.hit) { block->result.index = i; break; }
    }
    return NULL;
}

/* An untimed two-block reference: one newly created helper plus the caller,
 * private results, and a complete join before the next ordered wave. Its
 * thread construction is not a comparison with Whitefoot's worker pool. */
static uint64_t native_pair(const char *name, const unsigned char *data,
                            const uint64_t *offsets, uint64_t count,
                            struct search_trace *trace) {
    memset(trace, 0xff, TRACE_SLOTS * sizeof(*trace));
    for (uint64_t first = 0; first < count; first += 4) {
        uint64_t middle = minimum(count, first + 2), end = minimum(count, first + 4);
        struct native_block blocks[2] = {
            { data, offsets, { first, middle, count, 0, 0 } },
            { data, offsets, { middle, end, count, 0, 0 } }
        };
        pthread_t helper;
        int error = pthread_create(&helper, NULL, native_block_run, &blocks[0]);
        if (error) failure(name, 4, "native-create", first, 0, (uint64_t)error);
        native_block_run(&blocks[1]);
        error = pthread_join(helper, NULL);
        if (error) failure(name, 4, "native-join", first, 0, (uint64_t)error);
        ++native_waves;
        trace[first] = blocks[0].result;
        trace[first + 1] = blocks[1].result;
        uint64_t best = minimum(blocks[0].result.index, blocks[1].result.index);
        if (best < count) return best;
    }
    return count;
}

struct event_report {
    const char *error;
    uint64_t position, expected, actual;
    unsigned record[EVENT_SLOTS], identity[EVENT_SLOTS];
    unsigned completed, caller, helper, overlap;
};

static int event_error(struct event_report *report, const char *kind,
                       uint64_t position, uint64_t expected, uint64_t actual) {
    report->error = kind;
    report->position = position;
    report->expected = expected;
    report->actual = actual;
    return 0;
}

static int validate_events(const struct predicate_event *stream, unsigned total,
                           const unsigned char *data, const uint64_t *offsets,
                           uint64_t count, const struct search_trace *expected,
                           struct event_report *report) {
    unsigned entered[MAX_RECORDS] = { 0 }, returned[MAX_RECORDS] = { 0 };
    unsigned wanted[MAX_RECORDS] = { 0 }, wanted_count = 0;
    pthread_t threads[MAX_RECORDS], identities[EVENT_SLOTS + 1];
    memset(report, 0, sizeof(*report));
    identities[0] = pthread_self();
    unsigned identity_count = 1;
    for (unsigned block = 0; block < 2; ++block) {
        for (uint64_t i = expected[block].first;
             i < expected[block].first + expected[block].records; ++i) {
            wanted[i] = 1;
            ++wanted_count;
        }
    }
    if (count != 4 || total != 2 * wanted_count || total > EVENT_SLOTS)
        return event_error(report, "predicate-event-count", 0, 2 * wanted_count, total);
    for (unsigned ordinal = 0; ordinal < total; ++ordinal) {
        const struct predicate_event *event = &stream[ordinal];
        uint64_t record = count;
        for (uint64_t i = 0; i < count; ++i) {
            if (event->data == data + offsets[i] &&
                event->length == offsets[i + 1] - offsets[i]) record = i;
        }
        if (record == count || !event->length || event->returning > 1)
            return event_error(report, "predicate-range", ordinal, count - 1, record);
        if (!event->returning) {
            if (entered[record])
                return event_error(report, "predicate-duplicate", record, 0, 1);
            entered[record] = ordinal + 1;
            threads[record] = event->thread;
        } else {
            if (!entered[record] || returned[record] ||
                !pthread_equal(threads[record], event->thread))
                return event_error(report, "predicate-return", record, 1, 0);
            returned[record] = ordinal + 1;
        }
        unsigned identity = 0;
        while (identity < identity_count &&
               !pthread_equal(identities[identity], event->thread)) ++identity;
        if (identity == identity_count) identities[identity_count++] = event->thread;
        report->record[ordinal] = (unsigned)record;
        report->identity[ordinal] = identity;
    }
    for (uint64_t i = 0; i < count; ++i) {
        if ((entered[i] != 0) != wanted[i] || (returned[i] != 0) != wanted[i])
            return event_error(report, "predicate-prefix", i, wanted[i], returned[i] != 0);
        if (!wanted[i]) continue;
        if (i % 2 && (!returned[i - 1] || returned[i - 1] >= entered[i] ||
                      !pthread_equal(threads[i - 1], threads[i])))
            return event_error(report, "predicate-source-order", i, 1, 0);
        if (pthread_equal(threads[i], identities[0])) ++report->caller;
        else ++report->helper;
        for (uint64_t j = 0; j < i; ++j) {
            if (wanted[j] && !pthread_equal(threads[i], threads[j]) &&
                entered[i] < returned[j] && entered[j] < returned[i]) ++report->overlap;
        }
    }
    report->completed = wanted_count;
    return 1;
}

static int check_events(const char *name, uint64_t control_size,
                        const unsigned char *data, const uint64_t *offsets,
                        uint64_t count, const struct search_trace *expected) {
    unsigned total = atomic_load_explicit(&event_count, memory_order_relaxed);
    struct event_report report;
    if (!validate_events(events, total, data, offsets, count, expected, &report))
        failure(name, 4, report.error, report.position, report.expected, report.actual);
    for (unsigned ordinal = 0; ordinal < total; ++ordinal) {
        unsigned record = report.record[ordinal];
        printf("predicate-event\t%s\t%s\t%s\tT=%" PRIu64
               "\tordinal=%u\tkind=%s\trecord=%u"
               "\toffset=%" PRIu64 "\tlength=%" PRIu64 "\tthread=%u\n",
               image, workers, name, control_size, ordinal + 1,
               events[ordinal].returning ? "return" : "enter", record,
               offsets[record], events[ordinal].length, report.identity[ordinal]);
    }
    printf("predicate-summary\t%s\t%s\t%s\tT=%" PRIu64
           "\tcompleted=%u\tcaller=%u\thelper=%u\toverlaps=%u\n",
           image, workers, name, control_size, report.completed, report.caller,
           report.helper, report.overlap);
    if (!strcmp(workers, "1") && !strcmp(name, "balanced-absent") && control_size == 1) {
        struct predicate_event copy[EVENT_SLOTS];
        unsigned kept = 0, removed = 0;
        for (unsigned i = 0; i < total; ++i) {
            if (!removed && events[i].returning) removed = i + 1;
            else copy[kept++] = events[i];
        }
        struct event_report rejected;
        if (!removed || validate_events(copy, kept, data, offsets, count, expected, &rejected) ||
            strcmp(rejected.error, "predicate-event-count") || rejected.expected != 8 ||
            rejected.actual != 7)
            failure(name, 4, "missing-completion-control", 0, 1, 0);
        printf("self-control\t%s\t%s\tmissing-completion\tremoved-ordinal=%u"
               "\trejected=%s\texpected=%" PRIu64 "\tactual=%" PRIu64 "\n",
               image, workers, removed, rejected.error, rejected.expected, rejected.actual);
    }
    return report.caller && report.helper && report.overlap;
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
    for (unsigned mode = pair_only ? 4 : 0; mode < (pair_only ? 5 : 4); ++mode) {
        struct search_trace actual[TRACE_SLOTS], expected[TRACE_SLOTS];
        memset(actual, 0xff, sizeof(actual));
        expected_trace(info, count, mode, expected);
        if (observe_predicates) {
            atomic_store_explicit(&event_count, 0, memory_order_relaxed);
            atomic_store_explicit(&observing, 1, memory_order_release);
        }
        unsigned long before = wf__par_grants();
        uint64_t plain = wf_probe_search_plain(data, size, offsets, count + 1, count, mode);
        unsigned long plain_delta = wf__par_grants() - before;
        atomic_store_explicit(&observing, 0, memory_order_release);
        int overlapped = observe_predicates ?
            check_events(name, control_size, data, offsets, count, expected) : 0;
        before = wf__par_grants();
        uint64_t diagnostic = wf_probe_search_trace(data, size, offsets, count + 1,
                                                    count, mode == 4 ? 3 : mode,
                                                    actual, TRACE_SLOTS);
        unsigned long trace_delta = wf__par_grants() - before;
        plain_steals += plain_delta;
        trace_steals += trace_delta;
        comparisons += 2;
        if (plain != native.index) failure(name, mode, "plain-index", 0, native.index, plain);
        if (diagnostic != native.index) failure(name, mode, "trace-index", 0, native.index, diagnostic);
        if (inject_bad_trace) { actual[0].bytes ^= 1; inject_bad_trace = 0; }
        compare_trace(name, mode, "trace-field", expected, actual);
        unsigned long loop_delta = 0;
        if (pair_only) {
            before = wf__par_grants();
            uint64_t loop = wf_probe_search_plain(data, size, offsets, count + 1, count, 3);
            loop_delta = wf__par_grants() - before;
            loop_steals += loop_delta;
            struct search_trace native_trace[TRACE_SLOTS];
            uint64_t paired = native_pair(name, data, offsets, count, native_trace);
            comparisons += 2;
            if (loop != native.index) failure(name, mode, "loop-index", 0, native.index, loop);
            if (paired != native.index)
                failure(name, mode, "native-pair-index", 0, native.index, paired);
            compare_trace(name, mode, "native-pair-prefix", expected, native_trace);
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
                   "\trecords=%" PRIu64 "\tbytes=%" PRIu64 "\tplain-steals=%lu\ttrace-steals=%lu",
                   image, workers, name, control_size, mode, diagnostic, records, bytes,
                   plain_delta, trace_delta);
            if (pair_only)
                printf("\tphase=%s\tloop-steals=%lu\tnative-first-records=%" PRIu64
                       "\tnative-first-bytes=%" PRIu64,
                       fixed_controls ? "fixed" : "matrix", loop_delta,
                       native.records, native.bytes);
            fputs("\tprefixes=", stdout);
            for (unsigned slot = 0; slot < TRACE_SLOTS; ++slot) {
                if (actual[slot].first != UINT64_MAX)
                    printf("[%" PRIu64 ",%" PRIu64 "):%" PRIu64 "/%" PRIu64 ";",
                           actual[slot].first, actual[slot].end,
                           actual[slot].records, actual[slot].bytes);
            }
            putchar('\n');
        }
        if (fixed_controls && control_size == UINT64_C(1048576) &&
            !strcmp(workers, "4") &&
            (!strcmp(name, "balanced-absent") || !strcmp(name, "balanced-late-hit"))) {
            int passed = observe_predicates ? overlapped : plain_delta > 0;
            if (!passed) ++actualization_failures;
            printf("actualization\t%s\t%s\t%s\tT=%" PRIu64
                   "\tevidence=%s\tpass=%s\n", image, workers, name, control_size,
                   observe_predicates ? "predicate-overlap" : "plain-scan-steal",
                   passed ? "yes" : "no");
        }
    }
    free(saved);
    free(data);
}

static void oracle_matrix(void) {
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
}

static void paired_controls(void) {
    const uint64_t sizes[3] = { 1, 65536, 1048576 };
    fixed_controls = 1;
    for (unsigned i = 0; i < 3; ++i) {
        uint64_t t = sizes[i];
        const uint64_t balanced[4] = { 1, t, 1, t };
        const uint64_t distant[4] = { 1, t, t, 1 };
        check_case("balanced-absent", balanced, 4, 0, 0, t);
        check_case("balanced-late-hit", balanced, 4, 8, 2, t);
        check_case("other-helper", distant, 4, 1, 0, t);
        check_case("local-tails", balanced, 4, 5, 0, t);
    }
}

int wf__main_body(int argc, char **argv) {
    if (argc < 2 || argc > 3) return 3;
    image = argv[1];
    workers = getenv("WF_WORKERS");
    if (!workers) return 3;
    if (argc == 3) {
        if (!strcmp(argv[2], "self-control")) {
            const uint64_t lengths[4] = { 1, 3, 3, 1 };
            inject_bad_trace = 1;
            check_case("intentional-bad-trace", lengths, 4, 1, 0, 0);
            return 4; /* Reaching here means the expected rejection did not occur. */
        }
        pair_only = !strcmp(argv[2], "pair") || !strcmp(argv[2], "pair-observe");
        observe_predicates = !strcmp(argv[2], "pair-observe");
        if (!pair_only || observe_predicates != WF_FIRST_INDEX_OBSERVED ||
            (strcmp(workers, "1") && strcmp(workers, "4"))) return 3;
    }
    if (!observe_predicates) oracle_matrix();
    if (pair_only) paired_controls();
    printf("summary\t%s\t%s\tcases=%" PRIu64 "\tindex-comparisons=%" PRIu64
           "\ttrace-fields=%" PRIu64 "\tinput-bytes=%" PRIu64
           "\tplain-steals=%lu\ttrace-steals=%lu",
           image, workers, cases, comparisons, trace_fields, input_bytes,
           plain_steals, trace_steals);
    if (pair_only)
        printf("\tloop-steals=%lu\tnative-waves=%" PRIu64 "\tobserved=%d"
               "\tactualization-failures=%u", loop_steals, native_waves,
               observe_predicates, actualization_failures);
    putchar('\n');
    return actualization_failures ? 2 : 0;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
