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

enum { RECORD_WORDS = 32, VARIANT_COUNT = 3 };
extern uint64_t wf_deque_library_word_trace(uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_deque_library_record_trace(uint64_t, uint64_t, uint64_t, uint64_t);
typedef struct { uint64_t words[RECORD_WORDS]; } Record;
typedef union {
    struct { size_t bytes; uint64_t magic; } value;
    max_align_t alignment;
} AllocationHeader;
static size_t requests, requested_bytes, live_bytes, peak_bytes;
static volatile uint64_t observed;

static void require(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "deque library costs: %s\n", message); exit(1); }
}
NOINLINE void *wf_cost_allocate(uint64_t bytes) {
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failure");
    header->value.bytes = (size_t)bytes;
    header->value.magic = UINT64_C(0x64657175656c6962);
    ++requests; requested_bytes += (size_t)bytes; live_bytes += (size_t)bytes;
    if (live_bytes > peak_bytes) peak_bytes = live_bytes;
    return header + 1;
}
NOINLINE void wf_cost_release(void *pointer) {
    if (pointer == NULL) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.magic == UINT64_C(0x64657175656c6962), "allocation identity");
    require(live_bytes >= header->value.bytes, "live-byte accounting");
    live_bytes -= header->value.bytes; header->value.magic = 0; free(header);
}
static void reset_accounting(void) {
    require(live_bytes == 0, "allocation left live between traces");
    requests = requested_bytes = peak_bytes = 0;
}
static HELPER uint64_t make_word(uint64_t seed) { return seed; }
static HELPER Record make_record(uint64_t seed) {
    Record value;
    for (size_t i = 0; i < RECORD_WORDS; ++i) value.words[i] = seed + i;
    return value;
}
static HELPER void accept_word(uint64_t *digest, uint64_t value) {
    *digest = *digest * UINT64_C(131) + value;
}
static HELPER void accept_record(uint64_t *digest, Record value) {
    for (size_t i = 0; i < RECORD_WORDS; ++i)
        *digest = *digest * UINT64_C(131) + value.words[i];
}

// BULK changes only the genuine conversion between different allocations.
// Endpoint helpers and payload ownership/callback order stay identical.
#define DEFINE_DEQUE(P, T, MAKE, ACCEPT, BULK)                                \
typedef struct { uint64_t length, capacity, head; T data[]; } P##_Block;        \
_Static_assert(offsetof(P##_Block, data) == 24, "Ring header extent");          \
static HELPER P##_Block *P##_new(uint64_t capacity) {                          \
    P##_Block *block = wf_cost_allocate(sizeof *block + capacity * sizeof(T)); \
    block->length = block->head = 0; block->capacity = capacity;               \
    return block;                                                            \
}                                                                            \
static HELPER uint64_t P##_push_back(P##_Block **owner, T value) {            \
    P##_Block *block = *owner;                                               \
    uint64_t at = block->head + block->length;                                \
    if (at >= block->capacity) at -= block->capacity;                          \
    block->data[at] = value; return ++block->length;                          \
}                                                                            \
static HELPER uint64_t P##_push_front(P##_Block **owner, T value) {           \
    P##_Block *block = *owner;                                               \
    block->head = block->head ? block->head - 1 : block->capacity - 1;         \
    block->data[block->head] = value; return ++block->length;                 \
}                                                                            \
static HELPER T P##_pop_front(P##_Block **owner) {                            \
    P##_Block *block = *owner;                                               \
    T value = block->data[block->head];                                       \
    if (++block->head == block->capacity) block->head = 0;                     \
    --block->length; return value;                                           \
}                                                                            \
static HELPER T P##_pop_back(P##_Block **owner) {                             \
    P##_Block *block = *owner;                                               \
    uint64_t at = block->head + --block->length;                              \
    if (at >= block->capacity) at -= block->capacity;                          \
    return block->data[at];                                                  \
}                                                                            \
static HELPER P##_Block *P##_rebase(P##_Block *old, uint64_t capacity) {       \
    P##_Block *fresh = wf_cost_allocate(sizeof *fresh + capacity * sizeof(T)); \
    fresh->length = fresh->head = 0; fresh->capacity = capacity;               \
    if (BULK) {                                                              \
        uint64_t first = old->capacity - old->head;                          \
        if (first > old->length) first = old->length;                         \
        if (first) memcpy(fresh->data, old->data + old->head, first * sizeof(T)); \
        uint64_t second = old->length - first;                               \
        if (second) memcpy(fresh->data + first, old->data, second * sizeof(T)); \
        fresh->length = old->length;                                         \
        old->length = 0;                                                     \
    } else {                                                                 \
        while (old->length) {                                                \
            T value = old->data[old->head];                                  \
            if (++old->head == old->capacity) old->head = 0;                  \
            --old->length; fresh->data[fresh->length++] = value;              \
        }                                                                    \
    }                                                                        \
    wf_cost_release(old); return fresh;                                      \
}                                                                            \
static HELPER void P##_drain(P##_Block **owner, uint64_t *digest) {           \
    P##_Block *block = *owner;                                               \
    while (block->length) {                                                  \
        T value = block->data[block->head];                                  \
        if (++block->head == block->capacity) block->head = 0;                \
        --block->length; ACCEPT(digest, value);                               \
    }                                                                        \
}                                                                            \
static HELPER void P##_fill(P##_Block **owner, uint64_t count, uint64_t seed) { \
    for (uint64_t i = 0; i < count; ++i) P##_push_back(owner, MAKE(seed + i)); \
}                                                                            \
static HELPER void P##_free_empty(P##_Block *block) { wf_cost_release(block); } \
static HELPER uint64_t P##_trace(uint64_t count, uint64_t rounds,             \
                                  uint64_t seed, uint64_t path) {            \
    uint64_t digest = seed;                                                  \
    if (path == 2) {                                                         \
        for (uint64_t round = 0; round < rounds; ++round) {                   \
            uint64_t base = seed + round, part = base;                       \
            P##_Block *block = P##_new(count); P##_fill(&block, count, base); \
            for (uint64_t i = 0; i < count / 2; ++i) {                      \
                T value = P##_pop_front(&block); P##_push_back(&block, value); \
            }                                                                \
            block = P##_rebase(block, count * 2 + 1);                        \
            P##_drain(&block, &part); P##_free_empty(block);                 \
            digest = digest * UINT64_C(257) + part;                         \
        }                                                                    \
        return digest;                                                      \
    }                                                                        \
    P##_Block *block = P##_new(count); P##_fill(&block, count, seed);          \
    if (path) {                                                              \
        for (uint64_t round = 0; round < rounds; ++round) {                   \
            uint64_t base = seed + (round + 1) * count;                      \
            for (uint64_t i = 0; i < count; ++i) {                          \
                ACCEPT(&digest, P##_pop_back(&block));                      \
                P##_push_front(&block, MAKE(base + i));                     \
            }                                                                \
        }                                                                    \
    } else {                                                                 \
        for (uint64_t round = 0; round < rounds; ++round) {                   \
            uint64_t base = seed + (round + 1) * count;                      \
            for (uint64_t i = 0; i < count; ++i) {                          \
                ACCEPT(&digest, P##_pop_front(&block));                     \
                P##_push_back(&block, MAKE(base + i));                       \
            }                                                                \
        }                                                                    \
    }                                                                        \
    P##_drain(&block, &digest); P##_free_empty(block); return digest;         \
}                                                                            \
static NOINLINE uint64_t P##_library_trace(uint64_t count, uint64_t rounds,   \
                                            uint64_t seed, uint64_t path) {  \
    return P##_trace(count, rounds, seed, path);                              \
}

DEFINE_DEQUE(word_loop, uint64_t, make_word, accept_word, 0)
DEFINE_DEQUE(word_bulk, uint64_t, make_word, accept_word, 1)
DEFINE_DEQUE(record_loop, Record, make_record, accept_record, 0)
DEFINE_DEQUE(record_bulk, Record, make_record, accept_record, 1)

static NOINLINE uint64_t run(unsigned variant, bool wide, uint64_t count,
                            uint64_t rounds, uint64_t seed, uint64_t path) {
    uint64_t result;
    if (variant == 0)
        result = wide ? wf_deque_library_record_trace(count, rounds, seed, path)
                      : wf_deque_library_word_trace(count, rounds, seed, path);
    else if (variant == 1)
        result = wide ? record_loop_library_trace(count, rounds, seed, path)
                      : word_loop_library_trace(count, rounds, seed, path);
    else
        result = wide ? record_bulk_library_trace(count, rounds, seed, path)
                      : word_bulk_library_trace(count, rounds, seed, path);
    observed = result; return result;
}
static uint64_t oracle_accept(uint64_t digest, uint64_t seed, unsigned words) {
    for (unsigned i = 0; i < words; ++i) digest = digest * UINT64_C(131) + seed + i;
    return digest;
}
static uint64_t oracle(bool wide, uint64_t count, uint64_t rounds,
                       uint64_t seed, uint64_t path) {
    unsigned words = wide ? RECORD_WORDS : 1;
    uint64_t digest = seed;
    if (path == 2) {
        for (uint64_t round = 0; round < rounds; ++round) {
            uint64_t base = seed + round, part = base;
            for (uint64_t i = count / 2; i < count; ++i) part = oracle_accept(part, base + i, words);
            for (uint64_t i = 0; i < count / 2; ++i) part = oracle_accept(part, base + i, words);
            digest = digest * UINT64_C(257) + part;
        }
        return digest;
    }
    for (uint64_t round = 0; round < rounds; ++round)
        for (uint64_t i = 0; i < count; ++i) {
            uint64_t at = path == 1 && round == 0 ? count - 1 - i : i;
            digest = oracle_accept(digest, seed + round * count + at, words);
        }
    for (uint64_t i = 0; i < count; ++i) {
        uint64_t at = path == 1 && rounds ? count - 1 - i : i;
        digest = oracle_accept(digest, seed + rounds * count + at, words);
    }
    return digest;
}
static uint64_t nanos(void) {
#if defined(_WIN32)
    LARGE_INTEGER value, frequency;
    assert(QueryPerformanceCounter(&value) != 0); assert(QueryPerformanceFrequency(&frequency) != 0);
    return (uint64_t)((long double)value.QuadPart * 1.0e9L / frequency.QuadPart);
#else
    struct timespec value; assert(clock_gettime(CLOCK_MONOTONIC, &value) == 0);
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
#endif
}
static void check_accounting(bool wide, uint64_t count, uint64_t rounds, uint64_t path, uint64_t traces) {
    size_t stride = wide ? sizeof(Record) : sizeof(uint64_t);
    size_t expected_requests = 1, expected_bytes = 24 + count * stride, expected_peak = expected_bytes;
    if (path == 2) {
        size_t per_round = 48 + (3 * count + 1) * stride;
        expected_requests = 2 * rounds;
        expected_bytes = rounds * per_round;
        expected_peak = rounds ? per_round : 0;
    }
    expected_requests *= traces; expected_bytes *= traces;
    require(requests == expected_requests && requested_bytes == expected_bytes && peak_bytes == expected_peak,
            "independent allocation count, byte and peak formula");
    require(live_bytes == 0, "complete cleanup");
}
static void check(void) {
    const uint64_t counts[] = {0, 1, 2, 3, 16, 63, 256, 4096};
    const uint64_t rounds[] = {0, 1, 3}, seeds[] = {0, 17, UINT64_MAX};
    size_t configurations = 0;
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned path = 0; path < 3; ++path)
            for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                for (size_t r = 0; r < sizeof rounds / sizeof rounds[0]; ++r)
                    for (size_t s = 0; s < sizeof seeds / sizeof seeds[0]; ++s) {
                        uint64_t expected = oracle(wide != 0, counts[n], rounds[r], seeds[s], path);
                        for (unsigned v = 0; v < VARIANT_COUNT; ++v) {
                            reset_accounting();
                            require(run(v, wide != 0, counts[n], rounds[r], seeds[s], path) == expected,
                                    "independent logical-order checksum");
                            check_accounting(wide != 0, counts[n], rounds[r], path, 1);
                        }
                        ++configurations;
                    }
    printf("deque library costs: %zu configurations, %zu executions passed\n", configurations, configurations * VARIANT_COUNT);
}
static void measure(void) {
    const uint64_t counts[] = {16, 256, 4096};
    const char *paths[] = {"forward-churn", "reverse-churn", "wrapped-rebase", "setup-cleanup"};
    const char *variants[] = {"whitefoot", "loop-c", "bulk-c"};
    puts("contract,cohort,element_bytes,path,count,variant,sample,rounds,traces,elapsed_ns,checksum,requests,requested_bytes,peak_bytes");
    for (unsigned cohort = 0; cohort < 2; ++cohort)
        for (unsigned wide = 0; wide < 2; ++wide)
            for (unsigned path = 0; path < 4; ++path)
                for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                    for (unsigned sample = 0; sample < 11; ++sample) {
                        uint64_t count = counts[n], seed = 101 + sample;
                        uint64_t rounds = path == 3 ? 0 : UINT64_C(32768) / count;
                        uint64_t traces = path == 3 ? UINT64_C(32768) / count : 1;
                        uint64_t source_path = path == 3 ? 0 : path, expected = 0;
                        for (uint64_t trace = 0; trace < traces; ++trace)
                            expected = expected * UINT64_C(257) + oracle(wide != 0, count, rounds, seed + trace, source_path);
                        for (unsigned offset = 0; offset < VARIANT_COUNT; ++offset) {
                            unsigned position = (sample + offset) % VARIANT_COUNT;
                            unsigned v = cohort ? VARIANT_COUNT - 1 - position : position;
                            reset_accounting(); uint64_t before = nanos();
                            uint64_t checksum = 0;
                            for (uint64_t trace = 0; trace < traces; ++trace)
                                checksum = checksum * UINT64_C(257) + run(v, wide != 0, count, rounds, seed + trace, source_path);
                            uint64_t elapsed = nanos() - before;
                            require(checksum == expected && live_bytes == 0, "timed checksum and cleanup");
                            check_accounting(wide != 0, count, rounds, source_path, traces);
                            printf("%s,%u,%zu,%s,%" PRIu64 ",%s,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu,%zu\n",
                                   CONTRACT, cohort, wide ? sizeof(Record) : sizeof(uint64_t), paths[path], count,
                                   variants[v], sample, rounds, traces, elapsed, checksum, requests, requested_bytes, peak_bytes);
                        }
                    }
}
int main(int argc, char **argv) {
    require(argc == 2, "usage: deque-costs check|measure");
    if (strcmp(argv[1], "check") == 0) check();
    else { require(strcmp(argv[1], "measure") == 0, "usage: deque-costs check|measure"); measure(); }
    return 0;
}
