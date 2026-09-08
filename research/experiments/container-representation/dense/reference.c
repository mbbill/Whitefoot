#include <stddef.h>
#include <stdint.h>

#ifndef SIZE
#error SIZE must be supplied by the experiment Makefile
#endif
#ifndef LANES
#define LANES 1
#endif

/* Match the emitted FixedVector<u64, SIZE> field order and extent. Neither
 * native implementation is a checked Whitefoot storage implementation. */
typedef struct {
    uint64_t words[LANES];
} Element;
typedef struct {
    Element data[SIZE];
    uint64_t head;
    uint64_t len;
} Dense;
_Static_assert(sizeof(Dense) == SIZE * LANES * sizeof(uint64_t) + 16,
               "native aggregate must match Whitefoot payload plus two measures");
_Static_assert(offsetof(Dense, head) == SIZE * LANES * sizeof(uint64_t),
               "native payload must precede the descriptor measures");

static Dense append_value(Dense values, Element value) {
    values.data[values.len] = value;
    values.len += 1;
    return values;
}

static Dense build_value(uint64_t seed) {
    Dense values = {0};
    for (size_t at = 0; at < SIZE; ++at) {
        Element value;
        for (size_t lane = 0; lane < LANES; ++lane) {
            seed = seed * UINT64_C(6364136223846793005) +
                   UINT64_C(1442695040888963407);
            value.words[lane] = seed;
        }
        values = append_value(values, value);
    }
    return values;
}

static uint64_t update_and_consume(Dense *values, uint64_t seed,
                                   uint64_t rounds) {
    for (uint64_t pass = 0; pass < rounds; ++pass) {
        for (size_t at = 0; at < SIZE; ++at) {
            for (size_t lane = 0; lane < LANES; ++lane) {
                seed = seed * UINT64_C(2862933555777941757) + values->data[at].words[lane];
                values->data[at].words[lane] = seed;
            }
        }
    }
    uint64_t checksum = 0;
    for (size_t at = 0; at < SIZE; ++at) {
        for (size_t lane = 0; lane < LANES; ++lane) {
            checksum = checksum * UINT64_C(1099511628211) + values->data[at].words[lane];
        }
    }
    return checksum;
}

uint64_t native_append_value(uint64_t seed, uint64_t rounds) {
    Dense values = build_value(seed);
    return update_and_consume(&values, seed, rounds);
}

static void build_destination(Dense *values, uint64_t seed) {
    *values = (Dense){0};
    for (size_t at = 0; at < SIZE; ++at) {
        for (size_t lane = 0; lane < LANES; ++lane) {
            seed = seed * UINT64_C(6364136223846793005) +
                   UINT64_C(1442695040888963407);
            values->data[at].words[lane] = seed;
        }
        values->len += 1;
    }
}

static Dense build_return(uint64_t seed) {
    Dense values;
    build_destination(&values, seed);
    return values;
}

uint64_t native_value(uint64_t seed, uint64_t rounds) {
    Dense values = build_return(seed);
    return update_and_consume(&values, seed, rounds);
}

uint64_t native_destination(uint64_t seed, uint64_t rounds) {
    Dense values;
    build_destination(&values, seed);
    return update_and_consume(&values, seed, rounds);
}

#ifdef BOUNDARY_CONTROL
/* Separate checked-by-execution ABI controls; excluded from primary timing.
 * These force the two construction helpers to remain actual call boundaries. */
__attribute__((noinline)) static Dense return_across_boundary(uint64_t seed) {
    Dense values;
    build_destination(&values, seed);
    return values;
}

__attribute__((noinline)) static void construct_across_boundary(Dense *values,
                                                               uint64_t seed) {
    build_destination(values, seed);
}

uint64_t boundary_value(uint64_t seed, uint64_t rounds) {
    Dense values = return_across_boundary(seed);
    return update_and_consume(&values, seed, rounds);
}

uint64_t boundary_in_place(uint64_t seed, uint64_t rounds) {
    Dense values;
    construct_across_boundary(&values, seed);
    return update_and_consume(&values, seed, rounds);
}
#endif
