#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#else
#include <time.h>
#endif

#define NOINLINE __attribute__((noinline))
#if defined(RETAIN_HELPERS)
#define HELPER NOINLINE
#define CONTRACT "retained"
#else
#define HELPER
#define CONTRACT "normal"
#endif

extern uint64_t wf_vector_library_reserved_trace(void *store, uint64_t rounds,
                                                  uint64_t seed);
extern uint64_t wf_vector_library_growth_trace(void *store, uint64_t rounds,
                                                uint64_t seed);

typedef union {
    struct {
        size_t bytes;
        uint64_t magic;
    } value;
    max_align_t alignment;
} AllocationHeader;

static size_t allocation_requests;
static size_t allocation_live;
static size_t allocation_peak;
static volatile size_t allocation_fail_at;
static volatile uint64_t observed;

static void require(bool condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "vector library costs: %s\n", message);
        exit(1);
    }
}

NOINLINE void *wf_cost_allocate(uint64_t bytes) {
    ++allocation_requests;
    if (allocation_requests == allocation_fail_at) return NULL;
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failure");
    header->value.bytes = (size_t)bytes;
    header->value.magic = UINT64_C(0x766563746f726c69);
    allocation_live += (size_t)bytes;
    if (allocation_live > allocation_peak) allocation_peak = allocation_live;
    return header + 1;
}

NOINLINE void wf_cost_release(void *pointer) {
    if (pointer == NULL) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.magic == UINT64_C(0x766563746f726c69),
            "allocation identity");
    require(allocation_live >= header->value.bytes, "live-byte accounting");
    allocation_live -= header->value.bytes;
    header->value.magic = 0;
    free(header);
}

typedef struct {
    uint64_t *data;
    uint64_t capacity;
    uint64_t length;
    uint64_t head;
} Run;

typedef struct {
    Run storage;
} Vector;

static HELPER bool vector_new(Vector *values) {
    uint64_t *data = wf_cost_allocate(0);
    values->storage = (Run){data, 0, 0, 0};
    return true;
}

static HELPER bool vector_reserve(Vector *values, uint64_t total) {
    Run *run = &values->storage;
    if (run->capacity >= total) return true;
    require(total <= SIZE_MAX / sizeof(uint64_t), "native reserve extent");
    uint64_t *fresh = wf_cost_allocate(total * sizeof(uint64_t));
    if (fresh == NULL) return false;
    for (uint64_t index = 0; index < run->length; ++index)
        fresh[index] = run->data[run->head + index];
    wf_cost_release(run->data);
    *run = (Run){fresh, total, run->length, 0};
    return true;
}

static HELPER bool vector_append(Vector *values, uint64_t value) {
    Run *run = &values->storage;
    if (run->length == run->capacity) {
        uint64_t total = run->capacity == 0 ? 1 : run->capacity * 2;
        if (!vector_reserve(values, total)) return false;
        run = &values->storage;
    }
    run->data[run->head + run->length++] = value;
    return true;
}

static HELPER void vector_exchange_direct(Vector *values, uint64_t left,
                                          uint64_t right) {
    Run *run = &values->storage;
    uint64_t saved = run->data[run->head + left];
    run->data[run->head + left] = run->data[run->head + right];
    run->data[run->head + right] = saved;
}

static HELPER void vector_exchange_matched(Vector *values, uint64_t left,
                                           uint64_t right) {
    if (left == right) return;
    Run *run = &values->storage;
    uint64_t tail = run->data[run->head + --run->length];
    uint64_t end = run->length;
    if (left == end) {
        uint64_t previous = run->data[run->head + right];
        run->data[run->head + right] = tail;
        run->data[run->head + run->length++] = previous;
        return;
    }
    if (right == end) {
        uint64_t previous = run->data[run->head + left];
        run->data[run->head + left] = tail;
        run->data[run->head + run->length++] = previous;
        return;
    }
    uint64_t left_value = run->data[run->head + left];
    run->data[run->head + left] = tail;
    uint64_t right_value = run->data[run->head + right];
    run->data[run->head + right] = left_value;
    uint64_t saved_tail = run->data[run->head + left];
    run->data[run->head + left] = right_value;
    run->data[run->head + run->length++] = saved_tail;
}

static HELPER bool vector_insert(Vector *values, uint64_t index,
                                 uint64_t value, bool direct) {
    uint64_t before = values->storage.length;
    if (!vector_append(values, value)) return false;
    for (uint64_t cursor = before; cursor > index; --cursor) {
        if (direct)
            vector_exchange_direct(values, cursor - 1, cursor);
        else
            vector_exchange_matched(values, cursor - 1, cursor);
    }
    return true;
}

static HELPER uint64_t vector_remove(Vector *values, uint64_t index,
                                     bool direct) {
    Run *run = &values->storage;
    for (uint64_t cursor = index; cursor + 1 < run->length; ++cursor) {
        if (direct)
            vector_exchange_direct(values, cursor, cursor + 1);
        else
            vector_exchange_matched(values, cursor, cursor + 1);
    }
    return run->data[run->head + --run->length];
}

static HELPER void vector_drain(Vector *values, uint64_t *digest) {
    Run *run = &values->storage;
    while (run->length != 0) {
        uint64_t value = run->data[run->head++];
        --run->length;
        *digest = *digest * UINT64_C(131) + value;
    }
}

static HELPER void vector_drop(Vector *values) {
    require(values->storage.length == 0, "native final length");
    wf_cost_release(values->storage.data);
}

static NOINLINE uint64_t native_round(uint64_t seed, bool reserve_first,
                                      bool direct) {
    Vector values;
    require(vector_new(&values), "native construction");
    if (reserve_first) require(vector_reserve(&values, 32), "native reserve");
    for (uint64_t index = 0; index < 32; ++index)
        require(vector_append(&values, seed + index), "native append");
    require(vector_insert(&values, 16, seed, direct), "native insert");
    uint64_t digest = vector_remove(&values, 16, direct);
    vector_drain(&values, &digest);
    vector_drop(&values);
    return digest;
}

static NOINLINE uint64_t native_trace(uint64_t rounds, uint64_t seed,
                                      bool reserve_first, bool direct) {
    uint64_t digest = seed;
    for (uint64_t round = 0; round < rounds; ++round) {
        uint64_t observed_round =
            native_round(seed + round, reserve_first, direct);
        digest = digest * UINT64_C(257) + observed_round;
    }
    return digest;
}

static uint64_t nanos(void) {
#if defined(_WIN32)
    LARGE_INTEGER value, frequency;
    assert(QueryPerformanceCounter(&value) != 0);
    assert(QueryPerformanceFrequency(&frequency) != 0);
    return (uint64_t)((long double)value.QuadPart * 1.0e9L /
                      frequency.QuadPart);
#else
    struct timespec value;
    assert(clock_gettime(CLOCK_MONOTONIC, &value) == 0);
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) +
           (uint64_t)value.tv_nsec;
#endif
}

enum Variant { WHITEFOOT, MATCHED_C, DIRECT_C };

static NOINLINE uint64_t batch(enum Variant variant, bool reserve_first,
                               uint64_t seed, uint64_t rounds,
                               unsigned repetitions) {
    uint8_t heap = 0;
    uint64_t checksum = 0;
    for (unsigned repetition = 0; repetition < repetitions; ++repetition) {
        uint64_t next = seed + repetition;
        if (variant == WHITEFOOT) {
            checksum += reserve_first
                            ? wf_vector_library_reserved_trace(&heap, rounds,
                                                               next)
                            : wf_vector_library_growth_trace(&heap, rounds,
                                                             next);
        } else {
            checksum += native_trace(rounds, next, reserve_first,
                                     variant == DIRECT_C);
        }
    }
    observed = checksum;
    return checksum;
}

static void reset_accounting(void) {
    require(allocation_live == 0, "allocation left live between samples");
    allocation_requests = 0;
    allocation_peak = 0;
}

static void check(void) {
    const uint64_t rounds[] = {0, 1, 3, 17};
    for (unsigned path = 0; path < 2; ++path) {
        bool reserve_first = path == 0;
        for (unsigned sample = 0; sample < 32; ++sample) {
            for (unsigned count = 0; count < sizeof rounds / sizeof rounds[0];
                 ++count) {
                uint64_t seed = UINT64_C(17) + sample;
                reset_accounting();
                uint64_t wf = batch(WHITEFOOT, reserve_first, seed,
                                    rounds[count], 1);
                size_t wf_requests = allocation_requests;
                reset_accounting();
                uint64_t matched = batch(MATCHED_C, reserve_first, seed,
                                         rounds[count], 1);
                require(wf == matched, "matched-control checksum");
                require(allocation_requests == wf_requests,
                        "matched-control allocation count");
                reset_accounting();
                uint64_t direct = batch(DIRECT_C, reserve_first, seed,
                                        rounds[count], 1);
                require(wf == direct, "direct-control checksum");
                require(allocation_requests == wf_requests,
                        "direct-control allocation count");
            }
        }
    }
    puts("vector library costs: 384 matched traces passed");
}

static void measure(void) {
    const uint64_t rounds = 512;
    const unsigned repetitions = 64;
    puts("contract,path,variant,sample,calls,rounds,elapsed_ns,checksum,requests,peak_bytes");
    for (unsigned path = 0; path < 2; ++path) {
        bool reserve_first = path == 0;
        const char *path_name = reserve_first ? "reserved" : "growth";
        for (unsigned sample = 0; sample < 15; ++sample) {
            for (unsigned offset = 0; offset < 3; ++offset) {
                enum Variant variant = (enum Variant)((sample + offset) % 3);
                reset_accounting();
                uint64_t before = nanos();
                uint64_t checksum = batch(variant, reserve_first,
                                          UINT64_C(101) + sample, rounds,
                                          repetitions);
                uint64_t elapsed = nanos() - before;
                require(allocation_live == 0, "allocation left after sample");
                printf("%s,%s,%s,%u,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64
                       ",%zu,%zu\n",
                       CONTRACT, path_name,
                       variant == WHITEFOOT
                           ? "whitefoot"
                           : variant == MATCHED_C ? "matched-c" : "direct-c",
                       sample,
                       repetitions, rounds, elapsed, checksum,
                       allocation_requests, allocation_peak);
            }
        }
    }
}

int main(int argc, char **argv) {
    require(argc == 2, "usage: vector-costs check|measure");
    if (strcmp(argv[1], "check") == 0) {
        check();
        return 0;
    }
    require(strcmp(argv[1], "measure") == 0,
            "usage: vector-costs check|measure");
    measure();
    return 0;
}
