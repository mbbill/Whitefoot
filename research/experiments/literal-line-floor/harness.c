#include "kernel.h"

#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

enum {
    WARMUPS = 3,
    REPETITIONS = 16,
    HOSTILE_LENGTH = 8 * 1024 * 1024,
};

typedef struct {
    uint64_t ordinal;
    uint64_t start;
    uint64_t end;
    uint64_t line_start;
    uint64_t line_end;
    uint64_t line_number;
} Record;

typedef struct {
    Record *data;
    uint64_t length;
    uint64_t capacity;
} Records;

static uint64_t digest_word(uint64_t digest, uint64_t value) {
    return (digest ^ value) * UINT64_C(1099511628211);
}

static uint64_t data_hash(const uint8_t *data, uint64_t length) {
    uint64_t digest = UINT64_C(14695981039346656037);
    for (uint64_t index = 0; index < length; ++index) {
        digest = digest_word(digest, data[index]);
    }
    return digest;
}

static void records_destroy(Records *records) {
    free(records->data);
    records->data = NULL;
    records->length = 0;
    records->capacity = 0;
}

static int records_push(Records *records, Record record) {
    if (records->length == records->capacity) {
        const uint64_t next_capacity = records->capacity == 0 ? 16 : records->capacity * 2;
        if (next_capacity < records->capacity ||
            next_capacity > SIZE_MAX / sizeof(*records->data)) {
            return 1;
        }
        Record *const next = realloc(records->data, (size_t)next_capacity * sizeof(*next));
        if (next == NULL) {
            return 1;
        }
        records->data = next;
        records->capacity = next_capacity;
    }
    records->data[records->length++] = record;
    return 0;
}

static int oracle_records(WfBuffer haystack, WfBuffer needle, Records *records) {
    if (needle.length == 0 || memchr(needle.data, '\n', (size_t)needle.length) != NULL) {
        return 0;
    }
    uint64_t line_number = 1;
    uint64_t line_start = 0;
    while (line_start < haystack.length) {
        uint64_t line_end = line_start;
        while (line_end < haystack.length && haystack.data[line_end] != '\n') {
            ++line_end;
        }
        uint64_t start = line_start;
        while (needle.length <= line_end - start) {
            const uint64_t end = start + needle.length;
            {
                uint64_t matched = 0;
                while (matched < needle.length &&
                       haystack.data[start + matched] == needle.data[matched]) {
                    ++matched;
                }
                if (matched == needle.length) {
                    if (records_push(records,
                                     (Record){records->length, start, end, line_start,
                                              line_end, line_number}) != 0) {
                        return 1;
                    }
                    start = end;
                } else {
                    ++start;
                }
            }
        }
        line_start = line_end < haystack.length ? line_end + 1 : haystack.length;
        ++line_number;
    }
    return 0;
}

static uint64_t digest_records(const Records *records, uint64_t haystack_length,
                               uint64_t needle_length) {
    uint64_t digest = UINT64_C(14695981039346656037);
    for (uint64_t index = 0; index < records->length; ++index) {
        const Record record = records->data[index];
        digest = digest_word(digest, record.ordinal);
        digest = digest_word(digest, record.start);
        digest = digest_word(digest, record.end);
        digest = digest_word(digest, record.line_start);
        digest = digest_word(digest, record.line_end);
        digest = digest_word(digest, record.line_number);
    }
    digest = digest_word(digest, records->length);
    digest = digest_word(digest, haystack_length);
    return digest_word(digest, needle_length);
}

static int verify_case(WfBuffer haystack, WfBuffer needle) {
    Records records = {0};
    if (oracle_records(haystack, needle, &records) != 0) {
        fputs("oracle allocation failed\n", stderr);
        records_destroy(&records);
        return 1;
    }
    const uint64_t expected = digest_records(&records, haystack.length, needle.length);
    const uint64_t observed = wf_literal_line(haystack, needle);
    if (observed != expected) {
        fprintf(stderr,
                "correctness failure: haystack_length=%" PRIu64
                " needle_length=%" PRIu64 " records=%" PRIu64
                " expected=%" PRIu64 " observed=%" PRIu64 "\n",
                haystack.length, needle.length, records.length, expected, observed);
        records_destroy(&records);
        return 1;
    }
    records_destroy(&records);
    return 0;
}

static int verify_expected(WfBuffer haystack, WfBuffer needle,
                           const Record *expected_records, uint64_t expected_length) {
    Records records = {0};
    if (oracle_records(haystack, needle, &records) != 0) {
        records_destroy(&records);
        return 1;
    }
    const int mismatch = records.length != expected_length ||
                         (expected_length != 0 &&
                          memcmp(records.data, expected_records,
                                 (size_t)expected_length * sizeof(*expected_records)) != 0);
    records_destroy(&records);
    if (mismatch) {
        fputs("oracle record-vector failure\n", stderr);
        return 1;
    }
    return verify_case(haystack, needle);
}

static uint64_t integer_power(uint64_t base, uint64_t exponent) {
    uint64_t result = 1;
    for (uint64_t index = 0; index < exponent; ++index) {
        result *= base;
    }
    return result;
}

static void word_for_ordinal(uint8_t *word, uint64_t length,
                             const uint8_t *alphabet, uint64_t alphabet_length,
                             uint64_t ordinal) {
    for (uint64_t index = length; index > 0; --index) {
        word[index - 1] = alphabet[ordinal % alphabet_length];
        ordinal /= alphabet_length;
    }
}

static int run_checks(void) {
    static const uint8_t x[] = {'x'};
    static const uint8_t xx[] = {'x', 'x'};
    static const uint8_t split[] = {'a', 'b', '\n', 'c', 'd'};
    static const uint8_t bc[] = {'b', 'c'};
    static const uint8_t overlap[] = {'a', 'a', 'a', 'a', 'a'};
    static const uint8_t aaa[] = {'a', 'a', 'a'};
    static const uint8_t empty_storage = 0;
    static const uint8_t empty_lines[] = {'a', '\n', '\n', 'b'};
    static const uint8_t b[] = {'b'};
    static const uint8_t aba_twice[] = {'a', 'b', 'a', 'a', 'b', 'a'};
    static const uint8_t aba[] = {'a', 'b', 'a'};
    static const uint8_t newline_needle[] = {'a', '\n', 'b'};
    static const uint8_t binary_line[] = {0, '\r', 255, 'x', '\n'};
    static const uint8_t binary_needle[] = {'\r', 255};
    static const Record xx_records[] = {
        {0, 0, 1, 0, 2, 1},
        {1, 1, 2, 0, 2, 1},
    };
    static const Record empty_line_records[] = {{0, 3, 4, 3, 4, 3}};
    static const Record overlap_records[] = {{0, 0, 3, 0, 5, 1}};
    static const Record twice_records[] = {
        {0, 0, 3, 0, 6, 1},
        {1, 3, 6, 0, 6, 1},
    };
    static const Record binary_records[] = {{0, 1, 3, 0, 4, 1}};
    if (verify_expected((WfBuffer){xx, sizeof(xx)}, (WfBuffer){x, sizeof(x)},
                        xx_records, 2) ||
        verify_expected((WfBuffer){empty_lines, sizeof(empty_lines)},
                        (WfBuffer){b, sizeof(b)}, empty_line_records, 1) ||
        verify_expected((WfBuffer){overlap, sizeof(overlap)},
                        (WfBuffer){aaa, sizeof(aaa)}, overlap_records, 1) ||
        verify_expected((WfBuffer){aba_twice, sizeof(aba_twice)},
                        (WfBuffer){aba, sizeof(aba)}, twice_records, 2) ||
        verify_expected((WfBuffer){binary_line, sizeof(binary_line)},
                        (WfBuffer){binary_needle, sizeof(binary_needle)}, binary_records, 1)) {
        return 1;
    }
    if (verify_case((WfBuffer){&empty_storage, 0}, (WfBuffer){x, sizeof(x)}) ||
        verify_case((WfBuffer){x, sizeof(x)}, (WfBuffer){&empty_storage, 0}) ||
        verify_case((WfBuffer){newline_needle, sizeof(newline_needle)},
                    (WfBuffer){newline_needle, sizeof(newline_needle)}) ||
        verify_case((WfBuffer){x, sizeof(x)}, (WfBuffer){x, sizeof(x)}) ||
        verify_case((WfBuffer){xx, sizeof(xx)}, (WfBuffer){x, sizeof(x)}) ||
        verify_case((WfBuffer){split, sizeof(split)}, (WfBuffer){bc, sizeof(bc)}) ||
        verify_case((WfBuffer){overlap, sizeof(overlap)}, (WfBuffer){aaa, sizeof(aaa)})) {
        return 1;
    }

    static const uint8_t haystack_alphabet[] = {'a', 'b', '\n'};
    static const uint8_t needle_alphabet[] = {'a', 'b'};
    uint8_t haystack[7];
    uint8_t needle[4];
    for (uint64_t haystack_length = 0; haystack_length <= sizeof(haystack); ++haystack_length) {
        const uint64_t haystack_count = integer_power(3, haystack_length);
        for (uint64_t haystack_ordinal = 0; haystack_ordinal < haystack_count;
             ++haystack_ordinal) {
            word_for_ordinal(haystack, haystack_length, haystack_alphabet, 3,
                             haystack_ordinal);
            for (uint64_t needle_length = 1; needle_length <= sizeof(needle);
                 ++needle_length) {
                const uint64_t needle_count = integer_power(2, needle_length);
                for (uint64_t needle_ordinal = 0; needle_ordinal < needle_count;
                     ++needle_ordinal) {
                    word_for_ordinal(needle, needle_length, needle_alphabet, 2,
                                     needle_ordinal);
                    if (verify_case((WfBuffer){haystack, haystack_length},
                                    (WfBuffer){needle, needle_length})) {
                        return 1;
                    }
                }
            }
        }
    }

    uint8_t *const hostile = malloc(HOSTILE_LENGTH);
    if (hostile == NULL) {
        fputs("hostile allocation failed\n", stderr);
        return 1;
    }
    memset(hostile, 'a', HOSTILE_LENGTH);
    for (uint64_t block_end = 4095; block_end < HOSTILE_LENGTH; block_end += 4096) {
        hostile[block_end] = '\n';
    }
    uint8_t hostile_needle[32];
    memset(hostile_needle, 'a', sizeof(hostile_needle));
    hostile_needle[31] = 'b';
    const uint64_t offsets[] = {UINT64_C(1024), UINT64_C(4195328)};
    for (uint64_t index = 0; index < sizeof(offsets) / sizeof(offsets[0]); ++index) {
        hostile[offsets[index] + 31] = 'b';
    }
    static const Record hostile_records[] = {
        {0, 1024, 1056, 0, 4095, 1},
        {1, 4195328, 4195360, 4194304, 4198399, 1025},
    };
    const int failed = verify_expected((WfBuffer){hostile, HOSTILE_LENGTH},
                                       (WfBuffer){hostile_needle, sizeof(hostile_needle)},
                                       hostile_records, 2);
    free(hostile);
    return failed;
}

static int hex_nibble(char byte) {
    if (byte >= '0' && byte <= '9') {
        return byte - '0';
    }
    if (byte >= 'a' && byte <= 'f') {
        return byte - 'a' + 10;
    }
    if (byte >= 'A' && byte <= 'F') {
        return byte - 'A' + 10;
    }
    return -1;
}

static int decode_hex(const char *text, uint8_t **data, uint64_t *length) {
    const size_t text_length = strlen(text);
    if (text_length % 2 != 0) {
        return 1;
    }
    *length = text_length / 2;
    *data = malloc(*length == 0 ? 1 : (size_t)*length);
    if (*data == NULL) {
        return 1;
    }
    for (uint64_t index = 0; index < *length; ++index) {
        const int high = hex_nibble(text[index * 2]);
        const int low = hex_nibble(text[index * 2 + 1]);
        if (high < 0 || low < 0) {
            free(*data);
            *data = NULL;
            return 1;
        }
        (*data)[index] = (uint8_t)((high << 4) | low);
    }
    return 0;
}

static int load_file(const char *path, uint8_t **data, uint64_t *length) {
    FILE *const file = fopen(path, "rb");
    if (file == NULL) {
        fprintf(stderr, "cannot open %s: %s\n", path, strerror(errno));
        return 1;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return 1;
    }
    const long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return 1;
    }
    *length = (uint64_t)size;
    *data = malloc(*length == 0 ? 1 : (size_t)*length);
    if (*data == NULL || fread(*data, 1, (size_t)*length, file) != *length) {
        free(*data);
        *data = NULL;
        fclose(file);
        return 1;
    }
    if (fclose(file) != 0) {
        free(*data);
        *data = NULL;
        return 1;
    }
    return 0;
}

static uint64_t monotonic_ns(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC_RAW, &value) != 0) {
        perror("clock_gettime");
        exit(2);
    }
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}

static int run_input(const char *path, const char *needle_hex, int timed) {
    uint8_t *haystack_data = NULL;
    uint64_t haystack_length = 0;
    uint8_t *needle_data = NULL;
    uint64_t needle_length = 0;
    if (load_file(path, &haystack_data, &haystack_length) ||
        decode_hex(needle_hex, &needle_data, &needle_length)) {
        free(haystack_data);
        free(needle_data);
        fputs("cannot load runtime input\n", stderr);
        return 2;
    }
    if (needle_length == 0 || memchr(needle_data, '\n', (size_t)needle_length) != NULL) {
        free(haystack_data);
        free(needle_data);
        fputs("runtime needle must be non-empty and contain no LF\n", stderr);
        return 2;
    }
    const WfBuffer haystack = {haystack_data, haystack_length};
    const WfBuffer needle = {needle_data, needle_length};
    Records records = {0};
    if (oracle_records(haystack, needle, &records) != 0) {
        free(haystack_data);
        free(needle_data);
        return 2;
    }
    const uint64_t expected = digest_records(&records, haystack.length, needle.length);
    if (haystack.length == UINT64_C(67108864) && needle.length == 23) {
        uint64_t lf_count = 0;
        uint64_t first_count = 0;
        for (uint64_t index = 0; index < haystack.length; ++index) {
            lf_count += haystack.data[index] == '\n';
            first_count += haystack.data[index] == needle.data[0];
        }
        const uint64_t last_start = records.length == 0 ? UINT64_MAX
                                                        : records.data[records.length - 1].start;
        if (records.length != 74 || lf_count != UINT64_C(1324231) ||
            first_count != UINT64_C(20172286) || last_start != UINT64_C(63538171)) {
            fprintf(stderr,
                    "frozen real-input invariants failed: records=%" PRIu64
                    " lf=%" PRIu64 " first=%" PRIu64 " last=%" PRIu64 "\n",
                    records.length, lf_count, first_count, last_start);
            records_destroy(&records);
            free(haystack_data);
            free(needle_data);
            return 1;
        }
    }
    if (wf_literal_line(haystack, needle) != expected) {
        records_destroy(&records);
        free(haystack_data);
        free(needle_data);
        fputs("input correctness failure\n", stderr);
        return 1;
    }

    uint64_t elapsed = 0;
    if (timed) {
        for (int warmup = 0; warmup < WARMUPS; ++warmup) {
            if (wf_literal_line(haystack, needle) != expected) {
                fputs("warmup correctness failure\n", stderr);
                return 1;
            }
        }
        uint64_t aggregate = UINT64_C(14695981039346656037);
        const uint64_t start = monotonic_ns();
        for (int repetition = 0; repetition < REPETITIONS; ++repetition) {
            const uint64_t invocation = wf_literal_line(haystack, needle);
            aggregate = digest_word(aggregate, (uint64_t)repetition);
            aggregate = digest_word(aggregate, invocation);
        }
        elapsed = monotonic_ns() - start;
        uint64_t expected_aggregate = UINT64_C(14695981039346656037);
        for (int repetition = 0; repetition < REPETITIONS; ++repetition) {
            expected_aggregate = digest_word(expected_aggregate, (uint64_t)repetition);
            expected_aggregate = digest_word(expected_aggregate, expected);
        }
        if (aggregate != expected_aggregate) {
            fputs("timed correctness failure\n", stderr);
            return 1;
        }
    }

    printf("digest=%" PRIu64 " records=%" PRIu64 " input_hash=%" PRIu64
           " needle_hash=%" PRIu64 " elapsed_ns=%" PRIu64
           " repetitions=%d length=%" PRIu64 "\n",
           expected, records.length, data_hash(haystack.data, haystack.length),
           data_hash(needle.data, needle.length), elapsed, timed ? REPETITIONS : 0,
           haystack.length);
    records_destroy(&records);
    free(haystack_data);
    free(needle_data);
    return 0;
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--check") == 0) {
        return run_checks();
    }
    if (argc == 4 && strcmp(argv[1], "--verify-input") == 0) {
        return run_input(argv[2], argv[3], 0);
    }
    if (argc == 4 && strcmp(argv[1], "--bench-input") == 0) {
        return run_input(argv[2], argv[3], 1);
    }
    fputs("usage: binary --check | --verify-input PATH NEEDLE_HEX | --bench-input PATH NEEDLE_HEX\n",
          stderr);
    return 2;
}
