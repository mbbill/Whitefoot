#include "kernel.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

enum {
    BUFFER_LENGTH = 64 * 1024 * 1024,
    WARMUPS = 3,
};

#if defined(WF_FULL_SCAN)
enum { REPETITIONS = 128 };
static const char *const SHAPE = "full";
#elif defined(WF_EARLY_SCAN)
enum { REPETITIONS = 24 };
static const char *const SHAPE = "early";
#else
#error "define exactly one scan shape"
#endif

static uint64_t next_state(uint64_t state) {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

static void fill_data(uint8_t *data, uint64_t length) {
    uint64_t state = UINT64_C(0x8f3f73b5cf1c9ade);
    for (uint64_t index = 0; index < length; ++index) {
        state = next_state(state);
        data[index] = (uint8_t)(state % 250);
    }
#if defined(WF_EARLY_SCAN)
    if (length >= 4) {
        data[length / 4] = 251;
        data[length / 2] = 252;
        data[length - 1] = 253;
    }
#endif
}

static uint64_t data_hash(const uint8_t *data, uint64_t length) {
    uint64_t hash = UINT64_C(14695981039346656037);
    for (uint64_t index = 0; index < length; ++index) {
        hash ^= data[index];
        hash *= UINT64_C(1099511628211);
    }
    return hash;
}

#if defined(WF_FULL_SCAN)
static uint64_t oracle_scan(const uint8_t *data, uint64_t length) {
    uint64_t lines = 0;
    uint64_t candidates = 0;
    for (uint64_t index = 0; index < length; ++index) {
        if (data[index] == 10) {
            ++lines;
        }
        if (data[index] == 80) {
            ++candidates;
        }
    }
    return lines + candidates * UINT64_C(1000000007);
}
#else
static uint64_t oracle_find(const uint8_t *data, uint64_t length, uint8_t needle) {
    for (uint64_t index = 0; index < length; ++index) {
        if (data[index] == needle) {
            return index;
        }
    }
    return length;
}

static uint64_t oracle_scan(const uint8_t *data, uint64_t length) {
    return oracle_find(data, length, 251) + oracle_find(data, length, 252) +
           oracle_find(data, length, 253) + oracle_find(data, length, 254);
}
#endif

static int verify_case(const uint8_t *data, uint64_t length) {
    const uint64_t expected = oracle_scan(data, length);
    const uint64_t observed = wf_scan((WfBuffer){data, length});
    if (observed != expected) {
        fprintf(stderr,
                "%s correctness failure: length=%" PRIu64
                " expected=%" PRIu64 " observed=%" PRIu64 "\n",
                SHAPE, length, expected, observed);
        return 1;
    }
    return 0;
}

static int run_checks(void) {
    const uint8_t one_line[] = {10};
    const uint8_t one_candidate[] = {80};
    const uint8_t mixed[] = {251, 10, 252, 80, 253, 10, 0};
    uint8_t all_bytes[256];
    uint8_t generated[4096];
    for (uint64_t index = 0; index < 256; ++index) {
        all_bytes[index] = (uint8_t)index;
    }
    fill_data(generated, sizeof(generated));
    const uint8_t empty_storage = 0;
    return verify_case(&empty_storage, 0) ||
           verify_case(one_line, sizeof(one_line)) ||
           verify_case(one_candidate, sizeof(one_candidate)) ||
           verify_case(mixed, sizeof(mixed)) ||
           verify_case(all_bytes, sizeof(all_bytes)) ||
           verify_case(generated, sizeof(generated));
}

static uint64_t monotonic_ns(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC_RAW, &value) != 0) {
        perror("clock_gettime");
        exit(2);
    }
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) +
           (uint64_t)value.tv_nsec;
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--check") == 0) {
        return run_checks();
    }
    if (argc != 1) {
        fprintf(stderr, "usage: scan-floor-binary [--check]\n");
        return 2;
    }

    uint8_t *const data = malloc(BUFFER_LENGTH);
    if (data == NULL) {
        fputs("allocation failed\n", stderr);
        return 2;
    }
    fill_data(data, BUFFER_LENGTH);
    const WfBuffer input = {data, BUFFER_LENGTH};
    const uint64_t expected = oracle_scan(data, BUFFER_LENGTH);
    const uint64_t hash = data_hash(data, BUFFER_LENGTH);

    for (int warmup = 0; warmup < WARMUPS; ++warmup) {
        if (wf_scan(input) != expected) {
            fputs("warmup correctness failure\n", stderr);
            free(data);
            return 1;
        }
    }

    uint64_t aggregate = 0;
    const uint64_t start = monotonic_ns();
    for (int repetition = 0; repetition < REPETITIONS; ++repetition) {
        aggregate += wf_scan(input);
    }
    const uint64_t elapsed = monotonic_ns() - start;
    const uint64_t expected_aggregate = expected * REPETITIONS;
    if (aggregate != expected_aggregate) {
        fputs("timed correctness failure\n", stderr);
        free(data);
        return 1;
    }

    printf("shape=%s checksum=%" PRIu64 " data_hash=%" PRIu64
           " elapsed_ns=%" PRIu64 " repetitions=%d length=%d\n",
           SHAPE, aggregate, hash, elapsed, REPETITIONS, BUFFER_LENGTH);
    free(data);
    return 0;
}
