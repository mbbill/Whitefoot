/* Native controls for one explicit byte-run reserve/fill/consume operation.
 * The WF module's allocator symbols are instrumented only by the test adapter. */
#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
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
#if defined(_MSC_VER)
#define NOINLINE __declspec(noinline)
#else
#define NOINLINE __attribute__((noinline))
#endif

typedef struct { void *pointer; size_t size; } Allocation;
typedef struct {
    Allocation owned[2];
    size_t requests, mallocs, reallocs, fail_at, live, peak, inplace;
} Ledger;
static Ledger ledger;
static volatile uint64_t observed;
static void require(int ok, const char *why) {
    if (!ok) { fprintf(stderr, "growth: %s\n", why); exit(1); }
}
static void reset(size_t fail_at) {
    require(ledger.live == 0, "unreleased backing before reset");
    memset(&ledger, 0, sizeof ledger);
    ledger.fail_at = fail_at;
}
static size_t owned_at(void *p) {
    for (size_t i = 0; i < 2; ++i) if (ledger.owned[i].pointer == p) return i;
    require(0, "free or resize of unowned backing");
    return 0;
}
NOINLINE void *growth_malloc(uint64_t amount) {
    ++ledger.mallocs;
    if (ledger.requests++ == ledger.fail_at) return NULL;
    require(amount > 0 && amount <= 1048576, "allocation extent");
    void *p = malloc((size_t)amount);
    if (!p) return NULL;
    size_t at = ledger.owned[0].pointer ? 1 : 0;
    require(!ledger.owned[at].pointer, "too many live backings");
    ledger.owned[at] = (Allocation){p, (size_t)amount};
    ledger.live += (size_t)amount;
    if (ledger.live > ledger.peak) ledger.peak = ledger.live;
    return p;
}
NOINLINE void growth_free(void *p) {
    if (!p) return;
    size_t at = owned_at(p);
    ledger.live -= ledger.owned[at].size;
    ledger.owned[at] = (Allocation){0};
    free(p);
}
static NOINLINE void *resize(void *p, size_t amount) {
    ++ledger.reallocs;
    if (ledger.requests++ == ledger.fail_at) return NULL;
    size_t at = owned_at(p), previous_size = ledger.owned[at].size;
    uintptr_t previous_address = (uintptr_t)p;
    void *grown = realloc(p, amount);
    if (!grown) return NULL;
    if ((uintptr_t)grown == previous_address) ++ledger.inplace;
    ledger.owned[at] = (Allocation){grown, amount};
    ledger.live = ledger.live - previous_size + amount;
    if (ledger.live > ledger.peak) ledger.peak = ledger.live;
    return grown;
}
static uint64_t clock_ns(void) {
#if defined(_WIN32)
    LARGE_INTEGER ticks, frequency;
    require(QueryPerformanceCounter(&ticks) != 0, "clock counter");
    require(QueryPerformanceFrequency(&frequency) != 0, "clock frequency");
    return (uint64_t)((long double)ticks.QuadPart * 1e9L / frequency.QuadPart);
#else
    struct timespec t;
    require(clock_gettime(CLOCK_MONOTONIC, &t) == 0, "clock");
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
#endif
}
static uint64_t digest(const uint8_t *data, size_t length, uint64_t status) {
    for (size_t i = 0; i < length; ++i) status = status * 131 + data[i];
    return status;
}
static uint8_t fill(uint8_t *data, size_t begin, size_t end, uint8_t state) {
    for (size_t i = begin; i < end; ++i) {
        state = (uint8_t)(state * 73u + 41u);
        data[i] = state;
    }
    return state;
}
enum { WF, C_LOOP, C_BULK, C_REALLOC, VARIANTS };
static const char *names[] = {"whitefoot", "c_loop", "c_bulk", "c_realloc"};
extern uint64_t wf_growth_trace(void *, uint8_t, uint64_t, uint64_t, uint64_t);

static inline __attribute__((always_inline)) uint64_t native_trace(uint8_t seed, size_t initial, size_t count,
                                    size_t limit, unsigned variant) {
    uint8_t *data = growth_malloc(initial);
    if (!data) return 0;
    uint8_t state = fill(data, 0, initial, seed);
    uint64_t status = 1;
    size_t length = initial;
    if (count != initial) {
        if (count > limit) status = 2;
        else {
            uint8_t *grown = variant == C_REALLOC ? resize(data, count) : growth_malloc(count);
            if (!grown) status = 3;
            else {
                if (variant != C_REALLOC) {
                    if (variant == C_BULK) memcpy(grown, data, initial);
                    else for (size_t i = 0; i < initial; ++i) grown[i] = data[i];
                    growth_free(data);
                }
                data = grown;
                (void)fill(data, initial, count, state);
                length = count;
            }
        }
    }
    uint64_t result = digest(data, length, status);
    growth_free(data);
    return result;
}
static NOINLINE uint64_t loop_trace(uint8_t seed, size_t initial, size_t count, size_t limit) {
    return native_trace(seed, initial, count, limit, C_LOOP);
}
static NOINLINE uint64_t bulk_trace(uint8_t seed, size_t initial, size_t count, size_t limit) {
    return native_trace(seed, initial, count, limit, C_BULK);
}
static NOINLINE uint64_t realloc_trace(uint8_t seed, size_t initial, size_t count, size_t limit) {
    return native_trace(seed, initial, count, limit, C_REALLOC);
}
static uint64_t oracle(uint8_t seed, size_t initial, size_t count, size_t limit,
                       size_t fail_at) {
    if (fail_at == 0) return 0;
    size_t length = initial;
    uint64_t hash = 1;
    if (count != initial) {
        if (count > limit) hash = 2;
        else if (fail_at == 1) hash = 3;
        else length = count;
    }
    /* Generate the final logical sequence directly; no allocation or movement. */
    unsigned value = seed;
    for (size_t i = 0; i < length; ++i) {
        value = (73u * value + 41u) % 256u;
        hash = hash * 131 + value;
    }
    return hash;
}
static uint64_t invoke(unsigned variant, uint8_t seed, size_t initial,
                       size_t count, size_t limit) {
    /* Heap is proof-only here: the current WF function forwards this pointer
     * but never dereferences it. This is an experiment ABI, not a public FFI. */
    uint8_t heap_placeholder = 0;
    switch (variant) {
    case WF: return wf_growth_trace(&heap_placeholder, seed, initial, count, limit);
    case C_LOOP: return loop_trace(seed, initial, count, limit);
    case C_BULK: return bulk_trace(seed, initial, count, limit);
    default: return realloc_trace(seed, initial, count, limit);
    }
}
static void check(void) {
    const size_t sizes[] = {1, 16, 257, 4096};
    const size_t failures[] = {SIZE_MAX, 0, 1};
    size_t cases = 0;
    for (unsigned seed = 0; seed < 16; ++seed)
        for (size_t n = 0; n < 4; ++n)
            for (size_t grow = 1; grow <= 2; ++grow)
                for (size_t bound = 0; bound < 3; ++bound)
                    for (size_t failure = 0; failure < 3; ++failure) {
                        size_t initial = sizes[n], count = initial * grow;
                        size_t limit = bound == 0 ? 0 : bound == 1 ? initial : count;
                        uint64_t expected = oracle((uint8_t)seed, initial, count, limit, failures[failure]);
                        for (unsigned v = 0; v < VARIANTS; ++v) {
                            reset(failures[failure]);
                            uint64_t actual = invoke(v, (uint8_t)seed, initial, count, limit);
                            require(actual == expected, "logical result differs from oracle");
                            require(ledger.live == 0, "backing leaked on result edge");
                            size_t requests = failures[failure] == 0 || count == initial || count > limit ? 1 : 2;
                            require(ledger.requests == requests, "wrong allocation-request count");
                            size_t peak = failures[failure] == 0 ? 0 : initial;
                            if (requests == 2 && failures[failure] != 1)
                                peak = v == C_REALLOC ? count : initial + count;
                            require(ledger.peak == peak, "wrong tracked live extent");
                            ++cases;
                        }
                    }
    printf("growth: %zu variant/input checks passed\n", cases);
}
static uint64_t batch(unsigned variant, size_t initial, size_t count, size_t iterations,
                      size_t *inplace) {
    uint64_t sum = 0;
    *inplace = 0;
    uint8_t heap_placeholder = 0;
#define RUN_BATCH(call) do { \
    for (size_t i = 0; i < iterations; ++i) { \
        reset(SIZE_MAX); \
        sum += (call); \
        *inplace += ledger.inplace; \
    } \
} while (0)
    switch (variant) {
    case WF: RUN_BATCH(wf_growth_trace(&heap_placeholder, (uint8_t)(19 + i), initial, count, count)); break;
    case C_LOOP: RUN_BATCH(loop_trace((uint8_t)(19 + i), initial, count, count)); break;
    case C_BULK: RUN_BATCH(bulk_trace((uint8_t)(19 + i), initial, count, count)); break;
    case C_REALLOC: RUN_BATCH(realloc_trace((uint8_t)(19 + i), initial, count, count)); break;
    default: require(0, "unknown variant");
    }
#undef RUN_BATCH
    observed = sum;
    require(ledger.live == 0, "timed backing leaked");
    return sum;
}
static void measure(void) {
    const size_t initial[] = {16, 4096, 524288};
    puts("variant,initial,count,sample,iterations,elapsed_ns,checksum,inplace_reallocs");
    for (size_t n = 0; n < 3; ++n) {
        size_t count = 2 * initial[n];
        for (unsigned sample = 0; sample < 7; ++sample)
            for (unsigned order = 0; order < VARIANTS; ++order) {
                unsigned v = (sample + order) % VARIANTS;
                size_t iterations = n == 0 ? 65536 : n == 1 ? 1024 : 8, inplace;
                (void)batch(v, initial[n], count, n == 2 ? 2 : 32, &inplace);
                uint64_t started = clock_ns();
                uint64_t sum = batch(v, initial[n], count, iterations, &inplace);
                uint64_t elapsed = clock_ns() - started;
                uint64_t expected = 0;
                for (size_t i = 0; i < iterations; ++i)
                    expected += oracle((uint8_t)(19 + i), initial[n], count, count, SIZE_MAX);
                require(sum == expected, "timed result differs from oracle");
                printf("%s,%zu,%zu,%u,%zu,%" PRIu64 ",%" PRIu64 ",%zu\n",
                       names[v], initial[n], count, sample, iterations, elapsed, sum, inplace);
            }
    }
}
int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "measure") == 0) measure();
    else { require(argc == 2 && strcmp(argv[1], "check") == 0, "usage: check|measure"); check(); }
    return 0;
}
