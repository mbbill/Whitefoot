#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
#include <assert.h>
#include <inttypes.h>
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
#define SIZE 16
#ifdef RETAIN_HELPERS
#define HELPER NOINLINE
#define NATIVE_NAME "c_heap_boundary"
#else
#define HELPER
#define NATIVE_NAME "c_heap"
#endif

extern uint64_t wf_priority_trace(uint64_t seed, uint64_t rounds);
static volatile uint64_t observed;

typedef struct {
    uint64_t values[SIZE];
    uint64_t count;
    uint64_t head;
} Heap;

static uint64_t next(uint64_t state) {
    return state * UINT64_C(6364136223846793005) + UINT64_C(1442695040888963407);
}

static uint64_t mix(uint64_t checksum, uint64_t value) {
    return checksum * UINT64_C(1099511628211) + value;
}

static HELPER void push(Heap *heap, uint64_t value) {
    uint64_t at = heap->count++;
    heap->values[at] = value;
    while (at != 0) {
        uint64_t parent = (at - 1) / 2;
        uint64_t lower = heap->values[at];
        uint64_t upper = heap->values[parent];
        if (upper <= lower) break;
        heap->values[parent] = lower;
        heap->values[at] = upper;
        at = parent;
    }
}

static HELPER uint64_t pop(Heap *heap) {
    uint64_t result = heap->values[0];
    uint64_t last = heap->values[--heap->count];
    if (heap->count == 0) return result;
    heap->values[0] = last;
    uint64_t at = 0;
    for (;;) {
        uint64_t left = 2 * at + 1;
        if (left >= heap->count) break;
        uint64_t best = left;
        uint64_t right = left + 1;
        if (right < heap->count && heap->values[right] < heap->values[left]) best = right;
        uint64_t lower = heap->values[best];
        uint64_t upper = heap->values[at];
        if (upper <= lower) break;
        heap->values[at] = lower;
        heap->values[best] = upper;
        at = best;
    }
    return result;
}

static NOINLINE uint64_t native_trace(uint64_t seed, uint64_t rounds) {
    uint64_t checksum = 0;
    uint64_t state = seed;
    for (uint64_t round = 0; round < rounds; ++round) {
        Heap heap = { .count = 0, .head = 0 };
        for (unsigned i = 0; i < SIZE; ++i) {
            state = next(state);
            push(&heap, state % 97);
        }
        for (unsigned i = 0; i < SIZE; ++i) checksum = mix(checksum, pop(&heap));
    }
    return checksum;
}

static int compare(const void *left, const void *right) {
    uint64_t a = *(const uint64_t *)left;
    uint64_t b = *(const uint64_t *)right;
    return (a > b) - (a < b);
}

/* Sorting is an independent behavioral oracle, not a timed heap competitor. */
static uint64_t oracle(uint64_t seed, uint64_t rounds) {
    uint64_t checksum = 0;
    uint64_t state = seed;
    for (uint64_t round = 0; round < rounds; ++round) {
        uint64_t values[SIZE];
        for (unsigned i = 0; i < SIZE; ++i) {
            state = next(state);
            values[i] = state % 97;
        }
        qsort(values, SIZE, sizeof values[0], compare);
        for (unsigned i = 0; i < SIZE; ++i) checksum = mix(checksum, values[i]);
    }
    return checksum;
}

static uint64_t nanos(void) {
#if defined(_WIN32)
    LARGE_INTEGER value, frequency;
    assert(QueryPerformanceCounter(&value) != 0);
    assert(QueryPerformanceFrequency(&frequency) != 0);
    return (uint64_t)((long double)value.QuadPart * 1.0e9L / frequency.QuadPart);
#else
    struct timespec value;
    assert(clock_gettime(CLOCK_MONOTONIC, &value) == 0);
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
#endif
}

static NOINLINE uint64_t batch(unsigned variant, uint64_t seed, uint64_t rounds, unsigned count) {
    uint64_t checksum = 0;
    if (variant == 0) {
        for (unsigned i = 0; i < count; ++i) {
            checksum += wf_priority_trace(seed + i, rounds);
        }
    } else {
        for (unsigned i = 0; i < count; ++i) {
            checksum += native_trace(seed + i, rounds);
        }
    }
    observed = checksum;
    return checksum;
}

int main(int argc, char **argv) {
    assert(argc == 2 || argc == 3);
    uint64_t seed = argc == 3 ? strtoull(argv[2], NULL, 10) : 19;
    const uint64_t rounds[] = {0, 1, 2, 7, 16};
    for (unsigned i = 0; i < 64; ++i) {
        for (unsigned r = 0; r < sizeof rounds / sizeof rounds[0]; ++r) {
            uint64_t expected = oracle(seed + i, rounds[r]);
            assert(wf_priority_trace(seed + i, rounds[r]) == expected);
            assert(native_trace(seed + i, rounds[r]) == expected);
        }
    }
    if (strcmp(argv[1], "check") == 0) return 0;
    assert(strcmp(argv[1], "measure") == 0);
    puts("variant,rounds,sample,calls,elapsed_ns,checksum");
    for (uint64_t rounds_count = 1; rounds_count <= 16; rounds_count *= 16) {
        for (unsigned sample = 0; sample < 7; ++sample) {
            uint64_t previous = 0;
            for (unsigned order = 0; order < 2; ++order) {
                unsigned variant = (sample + order) % 2;
                (void)batch(variant, seed, rounds_count, 256);
                uint64_t start = nanos();
                uint64_t checksum = batch(variant, seed, rounds_count, 4096);
                uint64_t finish = nanos();
                assert(finish >= start);
                if (order != 0) assert(checksum == previous);
                previous = checksum;
                printf("%s,%" PRIu64 ",%u,4096,%" PRIu64 ",%" PRIu64 "\n",
                       variant == 0 ? "whitefoot" : NATIVE_NAME, rounds_count, sample,
                       finish - start, checksum);
            }
        }
    }
    return 0;
}
