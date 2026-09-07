#define _POSIX_C_SOURCE 200809L
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

extern uint64_t wf_dense(uint64_t seed, uint64_t rounds);
extern uint64_t native_value(uint64_t seed, uint64_t rounds);
extern uint64_t native_destination(uint64_t seed, uint64_t rounds);
extern uint64_t native_append_value(uint64_t seed, uint64_t rounds);
#ifdef BOUNDARY_CONTROL
extern uint64_t boundary_value(uint64_t seed, uint64_t rounds);
extern uint64_t boundary_in_place(uint64_t seed, uint64_t rounds);
#endif

typedef uint64_t (*Kernel)(uint64_t, uint64_t);
static const Kernel kernels[] = {wf_dense, native_value, native_destination,
                                native_append_value
#ifdef BOUNDARY_CONTROL
                                , boundary_value, boundary_in_place
#endif
};
static const char *const names[] = {"whitefoot", "c_value", "c_destination",
                                  "c_append_value"
#ifdef BOUNDARY_CONTROL
                                  , "boundary_value", "boundary_in_place"
#endif
};
#define VARIANTS (sizeof(kernels) / sizeof(kernels[0]))
static volatile uint64_t observed;

static uint64_t now_ns(void) {
    struct timespec time;
    if (clock_gettime(CLOCK_MONOTONIC, &time) != 0) {
        perror("clock_gettime");
        exit(2);
    }
    return (uint64_t)time.tv_sec * UINT64_C(1000000000) +
           (uint64_t)time.tv_nsec;
}

static uint64_t run(Kernel kernel, uint64_t rounds, uint64_t iterations,
                    uint64_t seed) {
    uint64_t checksum = 0;
    for (uint64_t at = 0; at < iterations; ++at) {
        checksum += kernel(seed + at, rounds);
    }
    observed = checksum;
    return checksum;
}

static int verify(uint64_t seed) {
    const uint64_t rounds[] = {0, 1, 3, 4, 8};
    for (size_t round = 0; round < sizeof(rounds) / sizeof(rounds[0]); ++round) {
        for (uint64_t at = 0; at < 12; ++at) {
            uint64_t input = seed + at * UINT64_C(11400714819323198485);
            uint64_t expected = native_destination(input, rounds[round]);
            for (size_t variant = 0; variant < VARIANTS; ++variant) {
                uint64_t actual = kernels[variant](input, rounds[round]);
                if (actual != expected) {
                    fprintf(stderr, "mismatch size=%u variant=%s seed=%" PRIu64
                            " rounds=%" PRIu64 " actual=%" PRIu64
                            " expected=%" PRIu64 "\n", SIZE, names[variant],
                            input, rounds[round], actual, expected);
                    return 1;
                }
            }
        }
    }
    printf("verified,%u,%u,60,%zu\n", SIZE, LANES, VARIANTS);
    return 0;
}

static void measure(uint64_t seed) {
    const uint64_t rounds[] = {0, 4};
    for (size_t round = 0; round < 2; ++round) {
        uint64_t iterations[VARIANTS];
        for (size_t variant = 0; variant < VARIANTS; ++variant) {
            uint64_t count = 1;
            for (;;) {
                uint64_t before = now_ns();
                run(kernels[variant], rounds[round], count, seed);
                uint64_t elapsed = now_ns() - before;
                if (elapsed >= UINT64_C(20000000) || count >= UINT64_C(1048576)) {
                    break;
                }
                count *= 2;
            }
            iterations[variant] = count;
        }
        /* Rotate sample order to avoid always measuring one variant first. */
        for (size_t sample = 0; sample < 7; ++sample) {
            for (size_t offset = 0; offset < VARIANTS; ++offset) {
                size_t variant = (sample + offset) % VARIANTS;
                uint64_t before = now_ns();
                uint64_t checksum = run(kernels[variant], rounds[round],
                                        iterations[variant], seed + sample * 97);
                uint64_t elapsed = now_ns() - before;
                printf("sample,%u,%u,%s,%" PRIu64 ",%zu,%" PRIu64 ",%" PRIu64
                       ",%" PRIu64 "\n", SIZE, LANES, names[variant], rounds[round],
                       sample, iterations[variant], elapsed, checksum);
            }
        }
    }
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: %s verify|measure SEED\n", argv[0]);
        return 2;
    }
    char *end = NULL;
    uint64_t seed = strtoull(argv[2], &end, 10);
    if (end == argv[2] || *end != '\0') {
        return 2;
    }
    if (strcmp(argv[1], "verify") == 0) {
        return verify(seed);
    }
    if (strcmp(argv[1], "measure") == 0) {
        measure(seed);
        return 0;
    }
    return 2;
}
