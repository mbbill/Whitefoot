#include "kernel.h"

#include <stddef.h>

static uint64_t digest_word(uint64_t digest, uint64_t value) {
    return (digest ^ value) * UINT64_C(1099511628211);
}

static uint64_t finish_digest(uint64_t digest, uint64_t match_count,
                              uint64_t haystack_length, uint64_t needle_length) {
    digest = digest_word(digest, match_count);
    digest = digest_word(digest, haystack_length);
    return digest_word(digest, needle_length);
}

static uint64_t find_scalar(WfBuffer haystack, WfBuffer needle,
                            uint64_t search_start, uint64_t search_end) {
    const uint64_t range_length = search_end - search_start;
    if (range_length < needle.length) {
        return search_end;
    }
    const uint8_t needle_first = needle.data[0];
    const uint64_t last_candidate = search_end - needle.length;
    uint64_t candidate = search_start;
    while (candidate <= last_candidate) {
        if (candidate >= search_end) {
            break;
        }
        int equal = haystack.data[candidate] == needle_first;
        if (equal) {
            uint64_t needle_index = 1;
            while (needle_index < needle.length) {
                const uint64_t haystack_index = candidate + needle_index;
                if (haystack_index >= search_end ||
                    haystack.data[haystack_index] != needle.data[needle_index]) {
                    equal = 0;
                    break;
                }
                ++needle_index;
            }
        }
        if (equal) {
            return candidate;
        }
        ++candidate;
    }
    return search_end;
}

uint64_t wf_literal_line(WfBuffer haystack, WfBuffer needle) {
    uint64_t digest = UINT64_C(14695981039346656037);
    uint64_t match_count = 0;
    uint64_t line_number = 1;
    uint64_t line_start = 0;
    uint64_t line_cursor = 0;
    uint64_t search_cursor = 0;

    int invalid_needle = needle.length == 0;
    for (uint64_t index = 0; index < needle.length; ++index) {
        if (needle.data[index] == '\n') {
            invalid_needle = 1;
            break;
        }
    }
    if (invalid_needle) {
        return finish_digest(digest, match_count, haystack.length, needle.length);
    }

    for (;;) {
        const uint64_t match_start =
            find_scalar(haystack, needle, search_cursor, haystack.length);
        if (match_start == haystack.length) {
            break;
        }
        while (line_cursor < match_start) {
            if (line_cursor >= haystack.length) {
                break;
            }
            if (haystack.data[line_cursor] == '\n') {
                line_start = line_cursor + 1;
                ++line_number;
            }
            ++line_cursor;
        }

        uint64_t line_end = match_start;
        while (line_end < haystack.length && haystack.data[line_end] != '\n') {
            ++line_end;
        }
        uint64_t candidate = line_start;
        for (;;) {
            const uint64_t found = find_scalar(haystack, needle, candidate, line_end);
            if (found == line_end) {
                break;
            }
            const uint64_t match_end = found + needle.length;
            digest = digest_word(digest, match_count);
            digest = digest_word(digest, found);
            digest = digest_word(digest, match_end);
            digest = digest_word(digest, line_start);
            digest = digest_word(digest, line_end);
            digest = digest_word(digest, line_number);
            ++match_count;
            candidate = match_end;
        }

        if (line_end < haystack.length) {
            search_cursor = line_end + 1;
            line_cursor = search_cursor;
            line_start = search_cursor;
            ++line_number;
        } else {
            search_cursor = haystack.length;
            line_cursor = haystack.length;
        }
    }
    return finish_digest(digest, match_count, haystack.length, needle.length);
}
