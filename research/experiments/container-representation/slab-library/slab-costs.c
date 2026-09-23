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
extern uint64_t wf_slab_library_word_trace(uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_slab_library_record_trace(uint64_t, uint64_t, uint64_t, uint64_t);
typedef struct { uint64_t words[RECORD_WORDS]; } Record;
typedef struct { uint64_t index, generation; } Handle;
typedef union {
    struct { size_t bytes; uint64_t magic; } value;
    max_align_t alignment;
} AllocationHeader;
static size_t requests, requested_bytes, live_bytes, peak_bytes;
static volatile uint64_t observed;
static void require(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "slab library costs: %s\n", message); exit(1); }
}
NOINLINE void *wf_cost_allocate(uint64_t bytes) {
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failure");
    header->value.bytes = (size_t)bytes; header->value.magic = UINT64_C(0x736c61626c696272);
    ++requests; requested_bytes += (size_t)bytes; live_bytes += (size_t)bytes;
    if (live_bytes > peak_bytes) peak_bytes = live_bytes;
    return header + 1;
}
NOINLINE void wf_cost_release(void *pointer) {
    if (pointer == NULL) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.magic == UINT64_C(0x736c61626c696272), "allocation identity");
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
static HELPER void visit_word(uint64_t *digest, const uint64_t *value) { accept_word(digest, *value); }
static HELPER void visit_record(uint64_t *digest, const Record *value) {
    for (size_t i = 0; i < RECORD_WORDS; ++i)
        *digest = *digest * UINT64_C(131) + value->words[i];
}

// The tagged control overlays vacant next with the live payload. Both controls
// keep the full generation and validate occupancy separately; neither boxes T.
typedef struct { uint64_t length, value, generation, next; } WordWindowCell;
typedef struct { uint64_t length; Record value; uint64_t generation, next; } RecordWindowCell;
typedef struct { uint64_t tag, generation; union { uint64_t value, next; } payload; } WordTaggedCell;
typedef struct { uint64_t tag, generation; union { Record value; uint64_t next; } payload; } RecordTaggedCell;
_Static_assert(sizeof(WordWindowCell) == 32 && sizeof(WordTaggedCell) == 24, "scalar slab strides");
_Static_assert(sizeof(RecordWindowCell) == 280 && sizeof(RecordTaggedCell) == 272, "record slab strides");
#define WINDOW_OCC(c) ((c)->length)
#define WINDOW_VALUE(c) ((c)->value)
#define WINDOW_NEXT(c) ((c)->next)
#define TAGGED_OCC(c) ((c)->tag)
#define TAGGED_VALUE(c) ((c)->payload.value)
#define TAGGED_NEXT(c) ((c)->payload.next)

#define DEFINE_SLAB(P, T, CELL, OCC, VALUE, NEXT, MAKE, VISIT, ACCEPT)          \
typedef struct { uint64_t length, capacity; CELL data[]; } P##_Block;          \
typedef struct { P##_Block *cells; uint64_t free_head; } P##_Slab;            \
typedef struct { uint64_t failed; union { Handle handle; T value; } data; } P##_Insert; \
typedef struct { uint64_t present; T value; } P##_Remove;                     \
_Static_assert(offsetof(P##_Block, data) == 16, "Slots header extent");        \
static HELPER P##_Slab P##_new(uint64_t capacity) {                          \
    P##_Block *block = wf_cost_allocate(sizeof *block + capacity * sizeof(CELL)); \
    block->length = 0; block->capacity = capacity;                            \
    return (P##_Slab){block, capacity};                                      \
}                                                                            \
static HELPER P##_Insert P##_insert(P##_Slab *slab, T value) {               \
    P##_Block *block = slab->cells;                                          \
    uint64_t index = slab->free_head;                                        \
    P##_Insert result;                                                       \
    if (index < block->length) {                                             \
        CELL *cell = &block->data[index];                                    \
        if (OCC(cell) == 0) {                                                \
            uint64_t following = NEXT(cell);                                \
            VALUE(cell) = value; OCC(cell) = 1; slab->free_head = following; \
            result.failed = 0; result.data.handle = (Handle){index, cell->generation}; \
            return result;                                                   \
        }                                                                    \
    } else if (block->length < block->capacity) {                            \
        index = block->length++;                                             \
        CELL *cell = &block->data[index];                                    \
        cell->generation = 0; NEXT(cell) = slab->free_head;                   \
        VALUE(cell) = value; OCC(cell) = 1;                                  \
        result.failed = 0; result.data.handle = (Handle){index, 0};           \
        return result;                                                       \
    }                                                                        \
    result.failed = 1; result.data.value = value; return result;              \
}                                                                            \
static HELPER bool P##_visit(P##_Slab *slab, Handle handle, uint64_t *digest) { \
    P##_Block *block = slab->cells;                                          \
    if (handle.index < block->length) {                                      \
        CELL *cell = &block->data[handle.index];                             \
        if (cell->generation == handle.generation && OCC(cell) > 0) {        \
            VISIT(digest, &VALUE(cell)); return true;                        \
        }                                                                    \
    }                                                                        \
    return false;                                                            \
}                                                                            \
static HELPER P##_Remove P##_remove(P##_Slab *slab, Handle handle) {         \
    P##_Block *block = slab->cells; P##_Remove result; result.present = 0;   \
    if (handle.index < block->length) {                                      \
        CELL *cell = &block->data[handle.index];                             \
        if (cell->generation == handle.generation && OCC(cell) > 0) {        \
            result.value = VALUE(cell); OCC(cell) = 0; result.present = 1;  \
            if (cell->generation < UINT64_MAX) {                            \
                ++cell->generation; NEXT(cell) = slab->free_head;            \
                slab->free_head = handle.index;                             \
            }                                                                \
        }                                                                    \
    }                                                                        \
    return result;                                                           \
}                                                                            \
static HELPER void P##_free(P##_Slab slab, uint64_t *digest) {               \
    while (slab.cells->length) {                                             \
        CELL cell = slab.cells->data[--slab.cells->length];                   \
        if (OCC(&cell) > 0) ACCEPT(digest, VALUE(&cell));                     \
    }                                                                        \
    wf_cost_release(slab.cells);                                             \
}                                                                            \
static HELPER uint64_t P##_trace(uint64_t count, uint64_t rounds,             \
                                  uint64_t seed, uint64_t path) {            \
    P##_Slab slab = P##_new(count); uint64_t digest = seed;                  \
    for (uint64_t i = 0; i < count; ++i) {                                  \
        P##_Insert made = P##_insert(&slab, MAKE(seed + i));                 \
        if (!made.failed) { accept_word(&digest, made.data.handle.index); accept_word(&digest, made.data.handle.generation); } \
        else { ACCEPT(&digest, made.data.value); accept_word(&digest, UINT64_MAX); } \
    }                                                                        \
    P##_Insert full = P##_insert(&slab, MAKE(seed ^ UINT64_C(11400714819323198485))); \
    if (full.failed) ACCEPT(&digest, full.data.value); else accept_word(&digest, UINT64_MAX); \
    for (uint64_t round = 0; round < rounds; ++round)                        \
        for (uint64_t i = 0; i < count; ++i) {                              \
            Handle handle = {i, path ? round : 0};                          \
            if (path == 0) {                                                \
                if (!P##_visit(&slab, handle, &digest)) accept_word(&digest, UINT64_MAX); \
            } else {                                                         \
                P##_Remove taken = P##_remove(&slab, handle);               \
                if (taken.present) ACCEPT(&digest, taken.value); else accept_word(&digest, UINT64_MAX); \
                accept_word(&digest, P##_visit(&slab, handle, &digest) ? UINT64_MAX : 0); \
                P##_Insert made = P##_insert(&slab, MAKE(seed + (round + 1) * count + i)); \
                if (!made.failed) { accept_word(&digest, made.data.handle.index); accept_word(&digest, made.data.handle.generation); } \
                else { ACCEPT(&digest, made.data.value); accept_word(&digest, UINT64_MAX); } \
            }                                                                \
        }                                                                    \
    P##_free(slab, &digest); return digest;                                  \
}                                                                            \
static NOINLINE uint64_t P##_library_trace(uint64_t count, uint64_t rounds,   \
                                            uint64_t seed, uint64_t path) {  \
    return P##_trace(count, rounds, seed, path);                              \
}

DEFINE_SLAB(word_window, uint64_t, WordWindowCell, WINDOW_OCC, WINDOW_VALUE, WINDOW_NEXT, make_word, visit_word, accept_word)
DEFINE_SLAB(word_tagged, uint64_t, WordTaggedCell, TAGGED_OCC, TAGGED_VALUE, TAGGED_NEXT, make_word, visit_word, accept_word)
DEFINE_SLAB(record_window, Record, RecordWindowCell, WINDOW_OCC, WINDOW_VALUE, WINDOW_NEXT, make_record, visit_record, accept_record)
DEFINE_SLAB(record_tagged, Record, RecordTaggedCell, TAGGED_OCC, TAGGED_VALUE, TAGGED_NEXT, make_record, visit_record, accept_record)

static NOINLINE uint64_t run(unsigned variant, bool wide, uint64_t count,
                            uint64_t rounds, uint64_t seed, uint64_t path) {
    uint64_t result;
    if (variant == 0)
        result = wide ? wf_slab_library_record_trace(count, rounds, seed, path)
                      : wf_slab_library_word_trace(count, rounds, seed, path);
    else if (variant == 1)
        result = wide ? record_window_library_trace(count, rounds, seed, path)
                      : word_window_library_trace(count, rounds, seed, path);
    else
        result = wide ? record_tagged_library_trace(count, rounds, seed, path)
                      : word_tagged_library_trace(count, rounds, seed, path);
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
    for (uint64_t i = 0; i < count; ++i) {
        digest = oracle_accept(digest, i, 1); digest = oracle_accept(digest, 0, 1);
    }
    digest = oracle_accept(digest, seed ^ UINT64_C(11400714819323198485), words);
    for (uint64_t round = 0; round < rounds; ++round)
        for (uint64_t i = 0; i < count; ++i) {
            digest = oracle_accept(digest, seed + (path ? round * count : 0) + i, words);
            if (path) {
                digest = oracle_accept(digest, 0, 1);
                digest = oracle_accept(digest, i, 1);
                digest = oracle_accept(digest, round + 1, 1);
            }
        }
    for (uint64_t i = count; i != 0; --i)
        digest = oracle_accept(digest, seed + (path ? rounds * count : 0) + i - 1, words);
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
static size_t expected_bytes(unsigned variant, bool wide, uint64_t count) {
    size_t stride = wide ? sizeof(RecordWindowCell) : sizeof(WordWindowCell);
    if (variant == 2) stride = wide ? sizeof(RecordTaggedCell) : sizeof(WordTaggedCell);
    return 16 + count * stride;
}
static void check(void) {
    const uint64_t counts[] = {0, 1, 2, 3, 16, 63, 256, 4096};
    const uint64_t rounds[] = {0, 1, 3}, seeds[] = {0, 17, UINT64_MAX};
    size_t configurations = 0;
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned path = 0; path < 2; ++path)
            for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                for (size_t r = 0; r < sizeof rounds / sizeof rounds[0]; ++r)
                    for (size_t s = 0; s < sizeof seeds / sizeof seeds[0]; ++s) {
                        uint64_t expected = oracle(wide != 0, counts[n], rounds[r], seeds[s], path);
                        for (unsigned v = 0; v < VARIANT_COUNT; ++v) {
                            reset_accounting();
                            require(run(v, wide != 0, counts[n], rounds[r], seeds[s], path) == expected,
                                    "independent value/outcome checksum");
                            require(live_bytes == 0, "complete cleanup");
                            require(requests == 1 && requested_bytes == expected_bytes(v, wide != 0, counts[n]) && peak_bytes == requested_bytes,
                                    "exact per-layout backing allocation");
                        }
                        ++configurations;
                    }
    printf("slab library costs: %zu configurations, %zu executions passed\n", configurations, configurations * VARIANT_COUNT);
}
static void measure(void) {
    const uint64_t counts[] = {16, 256, 4096};
    const char *paths[] = {"prefilled-lookup", "reuse-churn", "setup-cleanup"};
    const char *variants[] = {"whitefoot", "window-c", "tagged-c"};
    puts("contract,cohort,element_bytes,path,count,variant,sample,rounds,traces,elapsed_ns,checksum,requests,requested_bytes,peak_bytes");
    for (unsigned cohort = 0; cohort < 2; ++cohort)
        for (unsigned wide = 0; wide < 2; ++wide)
            for (unsigned path = 0; path < 3; ++path)
                for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                    for (unsigned sample = 0; sample < 11; ++sample) {
                        uint64_t count = counts[n], seed = 101 + sample;
                        uint64_t rounds = path == 2 ? 0 : UINT64_C(32768) / count;
                        uint64_t traces = path == 2 ? UINT64_C(32768) / count : 1;
                        uint64_t source_path = path == 2 ? 0 : path, expected = 0;
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
                            size_t bytes = expected_bytes(v, wide != 0, count);
                            require(requests == traces && requested_bytes == traces * bytes && peak_bytes == bytes,
                                    "timed allocation count, bytes and peak");
                            printf("%s,%u,%zu,%s,%" PRIu64 ",%s,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu,%zu\n",
                                   CONTRACT, cohort, wide ? sizeof(Record) : sizeof(uint64_t), paths[path], count,
                                   variants[v], sample, rounds, traces, elapsed, checksum, requests, requested_bytes, peak_bytes);
                        }
                    }
}
int main(int argc, char **argv) {
    require(argc == 2, "usage: slab-costs check|measure");
    if (strcmp(argv[1], "check") == 0) check();
    else { require(strcmp(argv[1], "measure") == 0, "usage: slab-costs check|measure"); measure(); }
    return 0;
}
