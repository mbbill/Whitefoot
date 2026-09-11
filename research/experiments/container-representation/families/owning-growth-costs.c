#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
#if defined(__APPLE__)
#define _DARWIN_C_SOURCE
#endif
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void *wf_cost_allocate(uint64_t bytes);
void wf_cost_release(void *pointer);
#define malloc wf_cost_allocate
#define free wf_cost_release
#define main retained_sparse_main
#if defined(RETAIN_HELPERS)
#define SPARSE_BOUNDARY __attribute__((noinline))
#define CONTRACT_NAME "retained"
#elif defined(INLINE_HELPERS)
#define SPARSE_BOUNDARY inline __attribute__((always_inline))
#define CONTRACT_NAME "inlined"
#else
#define CONTRACT_NAME "normal"
#endif
#include "../costs/sparse-owned.c"
#undef main
#undef free
#undef malloc

#define BOUNDARY __attribute__((noinline))
typedef struct {
    uint32_t tag;
    uint8_t fingerprint;
    uint64_t key;
    Resource *payload;
} WFSlot;
typedef struct { WFSlot *pointer; uint64_t capacity, length, head; } WFRun;
_Static_assert(sizeof(WFSlot) == 24 && offsetof(WFSlot, key) == 8 &&
               offsetof(WFSlot, payload) == 16, "LLVM Slot ABI");
_Static_assert(sizeof(WFRun) == 32, "LLVM Vector ABI");
_Static_assert(sizeof(Resource) == 40 && sizeof(ISlot) == 24, "retained native ABI");

extern uint32_t wf_map_put_bridge(void *, uint64_t, Resource *, Resource **);
extern uint64_t wf_map_find_bridge(void *, uint64_t);
extern uint64_t wf_map_grow_bridge(void *, uint64_t, uint8_t *);

typedef union {
    struct { size_t bytes; uint64_t magic; } value;
    max_align_t alignment;
} AllocationHeader;
static size_t allocation_requests, allocation_live, allocation_peak, allocation_fail_at;
static volatile uint64_t observed_checksum;

static void require(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "owning-growth-costs: %s\n", message); exit(1); }
}

void *wf_cost_allocate(uint64_t bytes) {
    ++allocation_requests;
    if (allocation_requests == allocation_fail_at) return NULL;
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failed");
    header->value.bytes = (size_t)bytes;
    header->value.magic = UINT64_C(0x736c6f746f776e72);
    allocation_live += (size_t)bytes;
    if (allocation_live > allocation_peak) allocation_peak = allocation_live;
    return header + 1;
}

void wf_cost_release(void *pointer) {
    if (!pointer) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.magic == UINT64_C(0x736c6f746f776e72), "allocation header identity");
    require(allocation_live >= header->value.bytes, "live byte accounting");
    allocation_live -= header->value.bytes;
    header->value.magic = 0;
    free(header);
}

enum Variant { WF, INTERLEAVED, SPLIT };
typedef struct { WFRun wf; ITable interleaved; STable split; } Tables;

static void *selected_table(Tables *tables, enum Variant variant) {
    if (variant == WF) return &tables->wf;
    if (variant == INTERLEAVED) return &tables->interleaved;
    return &tables->split;
}

static void initialize(Tables *tables, enum Variant variant, size_t capacity) {
    memset(tables, 0, sizeof *tables);
    if (variant == WF) {
        WFSlot *slots = wf_cost_allocate(capacity * 32);
        memset(slots, 0, capacity * sizeof *slots);
        tables->wf = (WFRun){slots, capacity, capacity, 0};
    } else if (variant == INTERLEAVED) {
        require(i_init(&tables->interleaved, capacity, false), "interleaved initialization");
    } else {
        require(s_init(&tables->split, capacity, false), "split initialization");
    }
}

static uint64_t table_digest(Tables *tables, enum Variant variant) {
    if (variant == INTERLEAVED) return i_digest(&tables->interleaved);
    if (variant == SPLIT) return s_digest(&tables->split);
    WFRun *run = &tables->wf;
    require(run->head == 0 && run->length == run->capacity, "returned full, flat map run");
    uint64_t hash = (UINT64_C(1469598103934665603) ^ run->capacity) * UINT64_C(1099511628211);
    for (size_t i = 0; i < run->capacity; ++i) {
        WFSlot *slot = &run->pointer[i];
        require(slot->tag <= 2, "valid WF slot tag");
        uint8_t control = slot->tag == 0 ? EMPTY : slot->tag == 1 ? DELETED : slot->fingerprint;
        hash = (hash ^ control) * UINT64_C(1099511628211);
        if (slot->tag == 2) {
            require(slot->fingerprint == fingerprint(slot->key), "WF fingerprint");
            hash = (hash ^ slot->key) * UINT64_C(1099511628211);
            hash = (hash ^ slot->payload->id) * UINT64_C(1099511628211);
        }
    }
    return hash;
}

static void cleanup_table(Tables *tables, enum Variant variant) {
    if (variant == WF) {
        for (size_t i = 0; i < tables->wf.capacity; ++i)
            if (tables->wf.pointer[i].tag == 2) resource_drop(tables->wf.pointer[i].payload);
        wf_cost_release(tables->wf.pointer);
    } else if (variant == INTERLEAVED) {
        i_drop_contents(&tables->interleaved);
        i_release_backing(&tables->interleaved);
    } else {
        s_drop_contents(&tables->split);
        s_release_backing(&tables->split);
    }
}

static BOUNDARY uint32_t native_i_put(void *opaque, uint64_t key, Resource *owner, Resource **back) {
    size_t placed;
    PutResult result = i_put(opaque, key, owner, back, &placed);
    if (result == PUT_FULL) *back = owner;
    return (uint32_t)result;
}
static BOUNDARY uint32_t native_s_put(void *opaque, uint64_t key, Resource *owner, Resource **back) {
    size_t placed;
    PutResult result = s_put(opaque, key, owner, back, &placed);
    if (result == PUT_FULL) *back = owner;
    return (uint32_t)result;
}
static BOUNDARY uint64_t native_i_find(void *opaque, uint64_t key) {
    size_t examined;
    Resource *found = i_find(opaque, key, &examined);
    return (found ? found->id : 0) * 131 + examined;
}
static BOUNDARY uint64_t native_s_find(void *opaque, uint64_t key) {
    size_t examined;
    Resource *found = s_find(opaque, key, &examined);
    return (found ? found->id : 0) * 131 + examined;
}

#define GROW_BRIDGE(P, TABLE) \
    static BOUNDARY uint64_t native_##P##_grow(void *opaque, uint64_t count, uint8_t *code) { \
        TABLE *table = opaque; \
        P##Rehash state; \
        if (!P##_begin(table, (size_t)count, false, &state)) { *code = 1; return 0; } \
        size_t examined, moved; \
        P##_step(&state, SIZE_MAX, &examined, &moved); \
        require(state.phase == PHASE_DONE && !state.held_valid, "native complete rehash"); \
        P##_release_backing(&state.source); \
        *table = state.target; \
        *code = 3; \
        return examined * 131 + moved; \
    }
GROW_BRIDGE(i, ITable)
GROW_BRIDGE(s, STable)

typedef uint32_t (*Put)(void *, uint64_t, Resource *, Resource **);
typedef uint64_t (*Find)(void *, uint64_t);
typedef uint64_t (*Grow)(void *, uint64_t, uint8_t *);
static Put insertions[] = {wf_map_put_bridge, native_i_put, native_s_put};
static Find finds[] = {wf_map_find_bridge, native_i_find, native_s_find};
static Grow grows[] = {wf_map_grow_bridge, native_i_grow, native_s_grow};

static uint64_t clock_ns(void) {
#if defined(_WIN32)
    LARGE_INTEGER ticks, frequency;
    require(QueryPerformanceCounter(&ticks) != 0 && QueryPerformanceFrequency(&frequency) != 0, "clock");
    return (uint64_t)((long double)ticks.QuadPart * 1e9L / frequency.QuadPart);
#else
    struct timespec time;
#if defined(__APPLE__)
    require(clock_gettime(CLOCK_MONOTONIC_RAW, &time) == 0, "raw monotonic clock");
#else
    require(clock_gettime(CLOCK_MONOTONIC, &time) == 0, "clock");
#endif
    return (uint64_t)time.tv_sec * UINT64_C(1000000000) + (uint64_t)time.tv_nsec;
#endif
}

typedef struct {
    uint64_t elapsed, checksum, digest, operations, repetitions;
    size_t requests, peak;
} Measurement;

static Measurement run_case(enum Variant variant, unsigned operation, size_t capacity,
                            uint64_t seed, bool refuse_growth) {
    require(allocation_live == 0 && backing_live_bytes == 0, "clean experiment boundary");
    allocation_requests = allocation_peak = allocation_fail_at = 0;
    reset_ledger();
    Tables tables;
    initialize(&tables, variant, capacity);
    Resource **owners = malloc((capacity / 2) * sizeof *owners);
    require(owners != NULL, "owner preparation");
    for (size_t i = 0; i < capacity / 2; ++i) owners[i] = resource_new(i + 1);
    void *table = selected_table(&tables, variant);
    Put insert = insertions[variant];
    if (operation != 1) {
        for (size_t i = 0; i < capacity / 2; ++i) {
            Resource *back = NULL;
            require(insert(table, i * 17 + seed, owners[i], &back) == 0 && back == NULL,
                    "untimed source construction");
        }
    }
    uint64_t before = table_digest(&tables, variant);
    if (refuse_growth) allocation_fail_at = allocation_requests + 1;
    uint64_t checksum = 0, operations = operation == 0 ? capacity * 16 : capacity / 2;
    const uint64_t start = clock_ns();
    if (operation == 0) {
        Find lookup = finds[variant];
        for (uint64_t i = 0; i < operations; ++i)
            checksum += lookup(table, (i % capacity) * 17 + seed);
    } else if (operation == 1) {
        for (size_t i = 0; i < capacity / 2; ++i) {
            Resource *back = NULL;
            require(insert(table, i * 17 + seed, owners[i], &back) == 0 && back == NULL,
                    "timed insertion ownership result");
        }
    } else {
        uint8_t code = 0;
        checksum = grows[variant](table, capacity * 2, &code);
        require(code == (refuse_growth ? 1 : 3), "growth outcome");
        operations = 1;
    }
    const uint64_t elapsed = clock_ns() - start;
    const uint64_t digest = table_digest(&tables, variant);
    if (refuse_growth) require(digest == before, "growth refusal returned original state");
    Measurement result = {elapsed, checksum, digest, operations, 1, allocation_requests, allocation_peak};
    cleanup_table(&tables, variant);
    free(owners);
    require(allocation_live == 0 && ledger.created_count == ledger.dropped_count,
            "every prepared owner released exactly once");
    observed_checksum = checksum + digest;
    return result;
}

static void compare_results(Measurement expected, Measurement actual) {
    require(expected.checksum == actual.checksum && expected.digest == actual.digest &&
            expected.operations == actual.operations && expected.requests == actual.requests,
            "matched operation contract differs");
}

static Measurement run_sample(enum Variant variant, unsigned operation, size_t capacity,
                              uint64_t seed, bool measuring) {
    const size_t repeats = measuring ? (capacity == 256 ? 64 : 4) : 1;
    Measurement total = {0};
    for (size_t repeat = 0; repeat < repeats; ++repeat) {
        Measurement one = run_case(variant, operation, capacity, seed * 37 + repeat, false);
        total.elapsed += one.elapsed;
        total.operations += one.operations;
        total.repetitions += one.repetitions;
        total.requests += one.requests;
        total.checksum = total.checksum * 131 + one.checksum;
        total.digest = total.digest * 131 + one.digest;
        if (one.peak > total.peak) total.peak = one.peak;
    }
    return total;
}

int main(int argc, char **argv) {
    require(argc == 2 && (strcmp(argv[1], "check") == 0 || strcmp(argv[1], "measure") == 0),
            "usage: owning-growth-costs check|measure");
    const bool measuring = strcmp(argv[1], "measure") == 0;
    const size_t capacities[] = {256, 4096, 16384};
    const char *names[] = {"whitefoot", "interleaved", "split"};
    const char *operations[] = {"lookup", "insert", "rehash"};
    if (measuring)
        puts("contract,variant,operation,capacity,sample,repetitions,operations,elapsed_ns,checksum,digest,requests,peak_bytes");
    for (size_t c = 0; c < sizeof capacities / sizeof capacities[0]; ++c) {
        size_t capacity = capacities[c];
        for (unsigned operation = 0; operation < 3; ++operation) {
            unsigned samples = measuring ? 18 : 4;
            for (unsigned sample = 0; sample < samples; ++sample) {
                Measurement measured[3];
                for (unsigned offset = 0; offset < 3; ++offset) {
                    unsigned variant = (sample + offset) % 3;
                    measured[variant] = run_sample((enum Variant)variant, operation, capacity,
                                                  sample, measuring);
                }
                compare_results(measured[INTERLEAVED], measured[WF]);
                compare_results(measured[INTERLEAVED], measured[SPLIT]);
                if (measuring && sample >= 4) {
                    for (unsigned variant = 0; variant < 3; ++variant) {
                        Measurement m = measured[variant];
                        printf("%s,%s,%s,%zu,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu\n",
                               CONTRACT_NAME, names[variant], operations[operation], capacity,
                               sample - 4, m.repetitions, m.operations, m.elapsed, m.checksum, m.digest, m.requests, m.peak);
                    }
                }
            }
        }
        Measurement reference = run_case(INTERLEAVED, 2, capacity, 19, true);
        compare_results(reference, run_case(WF, 2, capacity, 19, true));
        compare_results(reference, run_case(SPLIT, 2, capacity, 19, true));
    }
    if (!measuring) printf("owning-growth-costs %s: matched lookup, insert, rehash and refusal\n", CONTRACT_NAME);
    return 0;
}
