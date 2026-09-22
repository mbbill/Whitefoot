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

enum { RECORD_WORDS = 32, CEILING = 8193 };
extern uint64_t wf_vector_library_word_trace(uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_vector_library_record_trace(uint64_t, uint64_t, uint64_t, uint64_t);

typedef union {
    struct { size_t bytes; uint64_t magic; } value;
    max_align_t alignment;
} AllocationHeader;
static size_t requests, live_bytes, peak_bytes, requested_bytes;
static volatile uint64_t observed;

static void require(bool condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "vector library costs: %s\n", message);
        exit(1);
    }
}

NOINLINE void *wf_cost_allocate(uint64_t bytes) {
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failure");
    header->value.bytes = (size_t)bytes;
    header->value.magic = UINT64_C(0x766563746f726c69);
    ++requests;
    requested_bytes += (size_t)bytes;
    live_bytes += (size_t)bytes;
    if (live_bytes > peak_bytes) peak_bytes = live_bytes;
    return header + 1;
}

NOINLINE void wf_cost_release(void *pointer) {
    if (pointer == NULL) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.magic == UINT64_C(0x766563746f726c69), "allocation identity");
    require(live_bytes >= header->value.bytes, "live-byte accounting");
    live_bytes -= header->value.bytes;
    header->value.magic = 0;
    free(header);
}

static void reset_accounting(void) {
    require(live_bytes == 0, "allocation left live between calls");
    requests = peak_bytes = requested_bytes = 0;
}

typedef struct { uint64_t words[RECORD_WORDS]; } Record;
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

// Concrete controls keep element size and the consumption algorithm
// static, as in WF instantiation; no function pointer or width dispatch enters
// an element operation. CONSUME differs only inside consuming truncation:
// 0 reverses then consumes, 1 consumes directly, 2 interleaves swap then take,
// 3 interleaves take then swap through a private local.
#define DEFINE_VECTOR(P, T, MAKE, ACCEPT, CONSUME)                             \
typedef struct { uint64_t length, capacity; T data[]; } P##_Block;              \
_Static_assert(offsetof(P##_Block, data) == 16, "Slots header extent");          \
static HELPER P##_Block *P##_new(void) {                                       \
    P##_Block *block = wf_cost_allocate(sizeof *block);                        \
    block->length = block->capacity = 0;                                      \
    return block;                                                            \
}                                                                            \
static HELPER uint64_t P##_reserve(P##_Block **owner, uint64_t total) {         \
    P##_Block *old = *owner;                                                  \
    if (old->capacity >= total) return old->capacity;                          \
    P##_Block *fresh = wf_cost_allocate(sizeof *fresh + total * sizeof(T));    \
    fresh->length = old->length; fresh->capacity = total;                      \
    memmove(fresh->data, old->data, old->length * sizeof(T));                   \
    wf_cost_release(old); *owner = fresh;                                    \
    return total;                                                            \
}                                                                            \
static HELPER uint64_t P##_append(P##_Block **owner, T value) {                \
    if ((*owner)->length == (*owner)->capacity) {                             \
        uint64_t total = (*owner)->capacity ? (*owner)->capacity * 2 : 1;     \
        if (total > CEILING) total = CEILING;                                \
        P##_reserve(owner, total);                                           \
    }                                                                        \
    P##_Block *block = *owner;                                                \
    block->data[block->length++] = value;                                    \
    return block->length;                                                    \
}                                                                            \
static HELPER uint64_t P##_insert(P##_Block **owner, uint64_t at, T value) {    \
    if ((*owner)->length == (*owner)->capacity) {                             \
        uint64_t total = (*owner)->capacity ? (*owner)->capacity * 2 : 1;     \
        if (total > CEILING) total = CEILING;                                \
        P##_reserve(owner, total);                                           \
    }                                                                        \
    P##_Block *block = *owner;                                                \
    memmove(block->data + at + 1, block->data + at,                            \
            (block->length - at) * sizeof(T));                               \
    block->data[at] = value;                                                 \
    return ++block->length;                                                  \
}                                                                            \
static HELPER T P##_remove(P##_Block **owner, uint64_t at) {                   \
    P##_Block *block = *owner;                                                \
    T value = block->data[at];                                               \
    --block->length;                                                         \
    memmove(block->data + at, block->data + at + 1,                            \
            (block->length - at) * sizeof(T));                               \
    return value;                                                            \
}                                                                            \
static HELPER T P##_swap_remove(P##_Block **owner, uint64_t at) {              \
    P##_Block *block = *owner;                                                \
    uint64_t last = block->length - 1;                                       \
    T saved = block->data[at];                                               \
    block->data[at] = block->data[last];                                      \
    block->data[last] = saved;                                               \
    T result = block->data[--block->length];                                  \
    return result;                                                           \
}                                                                            \
static HELPER void P##_truncate(P##_Block **owner, uint64_t retained,          \
                                uint64_t *digest) {                          \
    P##_Block *block = *owner;                                                \
    uint64_t count = block->length;                                          \
    if (CONSUME == 1) {                                                      \
        for (uint64_t i = retained; i < count; ++i) ACCEPT(digest, block->data[i]); \
        block->length = retained;                                           \
    } else if (CONSUME == 3) {                                               \
        uint64_t half = (count - retained) / 2;                               \
        for (uint64_t i = 0; i < half; ++i) {                                \
            uint64_t left = retained + i;                                   \
            T taken = block->data[--block->length];                           \
            T saved = block->data[left];                                     \
            block->data[left] = taken;                                       \
            taken = saved;                                                  \
            ACCEPT(digest, taken);                                          \
        }                                                                    \
        while (block->length != retained) ACCEPT(digest, block->data[--block->length]); \
    } else {                                                                 \
        uint64_t half = (count - retained) / 2;                               \
        for (uint64_t i = 0; i < half; ++i) {                                \
            uint64_t left = retained + i;                                   \
            uint64_t right = CONSUME == 2 ? block->length - 1 : count - 1 - i; \
            T saved = block->data[left];                                    \
            block->data[left] = block->data[right];                          \
            block->data[right] = saved;                                      \
            if (CONSUME == 2) ACCEPT(digest, block->data[--block->length]);   \
        }                                                                    \
        while (block->length != retained) ACCEPT(digest, block->data[--block->length]); \
    }                                                                        \
}                                                                            \
static HELPER void P##_drain(P##_Block **owner, uint64_t *digest) {            \
    P##_truncate(owner, 0, digest);                                           \
}                                                                            \
static HELPER void P##_free_empty(P##_Block *owner) { wf_cost_release(owner); } \
static HELPER void P##_work(P##_Block **owner, uint64_t count, uint64_t seed,  \
                            uint64_t *digest) {                              \
    for (uint64_t i = 0; i < count; ++i) P##_append(owner, MAKE(seed + i));    \
    uint64_t middle = count / 2;                                             \
    P##_insert(owner, middle, MAKE(seed ^ UINT64_C(11400714819323198485)));    \
    ACCEPT(digest, P##_remove(owner, middle));                                \
    if (count) ACCEPT(digest, P##_swap_remove(owner, 0));                      \
    P##_truncate(owner, (*owner)->length / 2, digest);                         \
    P##_drain(owner, digest);                                                \
}                                                                            \
static HELPER uint64_t P##_round(uint64_t count, uint64_t seed, bool reserve) { \
    P##_Block *owner = P##_new();                                             \
    if (reserve) P##_reserve(&owner, count + 1);                              \
    uint64_t digest = seed;                                                  \
    P##_work(&owner, count, seed, &digest);                                   \
    P##_free_empty(owner);                                                    \
    return digest;                                                           \
}                                                                            \
static HELPER void P##_tail_work(P##_Block **owner, uint64_t retained,        \
                                 uint64_t removed, uint64_t seed,             \
                                 uint64_t *digest) {                          \
    for (uint64_t i = 0; i < removed; ++i)                                   \
        P##_append(owner, MAKE(seed + retained + i));                        \
    P##_truncate(owner, retained, digest);                                   \
}                                                                            \
static NOINLINE uint64_t P##_trace(uint64_t count, uint64_t rounds,            \
                                  uint64_t seed, uint64_t path) {             \
    uint64_t checksum = seed;                                                \
    if (path >= 3) {                                                         \
        uint64_t removed = path - 3;                                         \
        if (removed > count) removed = count;                               \
        uint64_t retained = count - removed;                                \
        P##_Block *owner = P##_new();                                         \
        P##_reserve(&owner, count + 1);                                      \
        for (uint64_t i = 0; i < retained; ++i)                             \
            P##_append(&owner, MAKE(seed + i));                              \
        for (uint64_t r = 0; r < rounds; ++r) {                              \
            uint64_t digest = seed + r;                                     \
            P##_tail_work(&owner, retained, removed, seed + r, &digest);     \
            checksum = checksum * UINT64_C(257) + digest;                    \
        }                                                                    \
        uint64_t digest = seed;                                              \
        P##_drain(&owner, &digest);                                          \
        P##_free_empty(owner);                                                \
        return checksum * UINT64_C(257) + digest;                            \
    }                                                                        \
    if (path == 2) {                                                         \
        P##_Block *owner = P##_new();                                         \
        P##_reserve(&owner, count + 1);                                      \
        for (uint64_t r = 0; r < rounds; ++r) {                              \
            uint64_t digest = seed + r;                                     \
            P##_work(&owner, count, seed + r, &digest);                      \
            checksum = checksum * UINT64_C(257) + digest;                    \
        }                                                                    \
        P##_free_empty(owner);                                                \
    } else {                                                                 \
        for (uint64_t r = 0; r < rounds; ++r)                                \
            checksum = checksum * UINT64_C(257) + P##_round(count, seed + r, path == 0); \
    }                                                                        \
    return checksum;                                                         \
}

DEFINE_VECTOR(word_reverse, uint64_t, make_word, accept_word, 0)
DEFINE_VECTOR(word_direct, uint64_t, make_word, accept_word, 1)
DEFINE_VECTOR(word_interleaved, uint64_t, make_word, accept_word, 2)
DEFINE_VECTOR(word_take_swap, uint64_t, make_word, accept_word, 3)
DEFINE_VECTOR(record_reverse, Record, make_record, accept_record, 0)
DEFINE_VECTOR(record_direct, Record, make_record, accept_record, 1)
DEFINE_VECTOR(record_interleaved, Record, make_record, accept_record, 2)
DEFINE_VECTOR(record_take_swap, Record, make_record, accept_record, 3)

enum Variant { WHITEFOOT, REVERSE_C, DIRECT_C, INTERLEAVED_C, TAKE_SWAP_C, VARIANT_COUNT };
static NOINLINE uint64_t run(enum Variant variant, bool wide, uint64_t count,
                            uint64_t rounds, uint64_t seed, uint64_t path) {
    uint64_t result;
    if (variant == WHITEFOOT)
        result = wide ? wf_vector_library_record_trace(count, rounds, seed, path)
                      : wf_vector_library_word_trace(count, rounds, seed, path);
    else if (variant == REVERSE_C)
        result = wide ? record_reverse_trace(count, rounds, seed, path)
                      : word_reverse_trace(count, rounds, seed, path);
    else if (variant == DIRECT_C)
        result = wide ? record_direct_trace(count, rounds, seed, path)
                      : word_direct_trace(count, rounds, seed, path);
    else if (variant == INTERLEAVED_C)
        result = wide ? record_interleaved_trace(count, rounds, seed, path)
                      : word_interleaved_trace(count, rounds, seed, path);
    else
        result = wide ? record_take_swap_trace(count, rounds, seed, path)
                      : word_take_swap_trace(count, rounds, seed, path);
    observed = result;
    return result;
}

// Independent logical-order oracle: after swap-removing the first item, the
// last original item occupies index zero. Truncation visits the suffix before
// drain visits the retained prefix. It never allocates or mutates a vector.
static uint64_t oracle_accept(uint64_t digest, uint64_t seed, unsigned words) {
    for (unsigned word = 0; word < words; ++word)
        digest = digest * UINT64_C(131) + seed + word;
    return digest;
}
static uint64_t oracle(uint64_t count, uint64_t rounds, uint64_t seed, bool wide,
                       uint64_t path) {
    unsigned words = wide ? RECORD_WORDS : 1;
    uint64_t checksum = seed;
    if (path >= 3) {
        uint64_t removed = path - 3;
        if (removed > count) removed = count;
        uint64_t retained = count - removed;
        for (uint64_t round = 0; round < rounds; ++round) {
            uint64_t base = seed + round, digest = base;
            for (uint64_t at = retained; at < count; ++at)
                digest = oracle_accept(digest, base + at, words);
            checksum = checksum * UINT64_C(257) + digest;
        }
        uint64_t digest = seed;
        for (uint64_t at = 0; at < retained; ++at)
            digest = oracle_accept(digest, seed + at, words);
        return checksum * UINT64_C(257) + digest;
    }
    for (uint64_t round = 0; round < rounds; ++round) {
        uint64_t base = seed + round;
        uint64_t digest = oracle_accept(base, base ^ UINT64_C(11400714819323198485), words);
        if (count) digest = oracle_accept(digest, base, words);
        uint64_t live = count ? count - 1 : 0;
        uint64_t retained = live / 2;
        for (unsigned part = 0; part < 2; ++part) {
            uint64_t start = part ? 0 : retained, end = part ? retained : live;
            for (uint64_t at = start; at < end; ++at)
                digest = oracle_accept(digest, base + (at ? at : count - 1), words);
        }
        checksum = checksum * UINT64_C(257) + digest;
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

static void check(void) {
    const uint64_t counts[] = {0, 1, 2, 3, 8, 16, 63, 256, 4096, 8192};
    const uint64_t rounds[] = {0, 1, 3};
    const uint64_t seeds[] = {0, 17, UINT64_MAX};
    size_t configurations = 0;
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned path = 0; path < 7; ++path)
            for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                for (size_t r = 0; r < sizeof rounds / sizeof rounds[0]; ++r)
                    for (size_t s = 0; s < sizeof seeds / sizeof seeds[0]; ++s) {
                        uint64_t expected = oracle(counts[n], rounds[r], seeds[s], wide != 0, path);
                        size_t wf_requests = 0, wf_peak = 0, wf_bytes = 0;
                        for (unsigned v = 0; v < VARIANT_COUNT; ++v) {
                            reset_accounting();
                            uint64_t actual = run((enum Variant)v, wide != 0, counts[n], rounds[r], seeds[s], path);
                            require(actual == expected, "independent logical-order checksum");
                            require(live_bytes == 0, "complete cleanup");
                            if (v == WHITEFOOT) {
                                wf_requests = requests; wf_peak = peak_bytes; wf_bytes = requested_bytes;
                            } else {
                                require(requests == wf_requests, "matched allocation count");
                                require(peak_bytes == wf_peak, "matched peak backing bytes");
                                require(requested_bytes == wf_bytes, "matched requested bytes");
                            }
                        }
                        ++configurations;
                    }
    require(configurations == 1260, "configuration matrix");
    printf("vector library costs: %zu five-way configurations, %zu executions passed\n",
           configurations, configurations * VARIANT_COUNT);
}

static void measure(void) {
    const uint64_t counts[] = {16, 256, 4096};
    const char *paths[] = {"reserved", "growth", "reuse", "suffix-0", "suffix-1", "suffix-2", "suffix-3"};
    const char *variants[] = {"whitefoot", "reverse-c", "direct-c", "swap-take-c", "take-swap-c"};
    puts("contract,cohort,element_bytes,path,count,variant,sample,rounds,elapsed_ns,checksum,requests,requested_bytes,peak_bytes");
    for (unsigned cohort = 0; cohort < 2; ++cohort)
        for (unsigned wide = 0; wide < 2; ++wide)
            for (unsigned path = 0; path < 7; ++path)
                for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                    for (unsigned sample = 0; sample < 11; ++sample) {
                        if (path == 3 || path == 5 || (path >= 3 && counts[n] == 256)) continue;
                        uint64_t count = counts[n];
                        uint64_t rounds = path >= 3 ? UINT64_C(65536) / (path - 3)
                                                   : UINT64_C(16384) / count;
                        uint64_t seed = UINT64_C(101) + sample;
                        uint64_t expected = oracle(count, rounds, seed, wide != 0, path);
                        for (unsigned offset = 0; offset < VARIANT_COUNT; ++offset) {
                            unsigned position = (sample + offset) % VARIANT_COUNT;
                            unsigned v = cohort ? VARIANT_COUNT - 1 - position : position;
                            reset_accounting();
                            uint64_t before = nanos();
                            uint64_t checksum = run((enum Variant)v, wide != 0, count, rounds, seed, path);
                            uint64_t elapsed = nanos() - before;
                            require(checksum == expected, "timed checksum");
                            require(live_bytes == 0, "timed cleanup");
                            printf("%s,%u,%zu,%s,%" PRIu64 ",%s,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu,%zu\n",
                                   CONTRACT, cohort, wide ? sizeof(Record) : sizeof(uint64_t),
                                   paths[path], count, variants[v], sample, rounds, elapsed,
                                   checksum, requests, requested_bytes, peak_bytes);
                        }
                    }
}

int main(int argc, char **argv) {
    require(argc == 2, "usage: vector-costs check|measure");
    if (strcmp(argv[1], "check") == 0) check();
    else {
        require(strcmp(argv[1], "measure") == 0, "usage: vector-costs check|measure");
        measure();
    }
    return 0;
}
