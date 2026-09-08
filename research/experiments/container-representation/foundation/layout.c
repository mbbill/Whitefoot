/*
 * Native representation-cost control for the container foundation study.
 *
 * This program compares concrete C11 layouts. It is not Whitefoot source,
 * does not model the proposed checker, and does not select a language rule.
 * Full, prefix, and ring storage have different semantics and are shown with
 * their minimal metadata. Only the two nullable layouts are rival encodings
 * of the same state.
 */

#include <errno.h>
#include <inttypes.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define CAPACITY 4096u
#define BIT_WORDS ((CAPACITY + 63u) / 64u)
#define DEFAULT_ITERATIONS 1200u
#define DEFAULT_SAMPLES 9u

#if defined(__clang__) || defined(__GNUC__)
#define NOINLINE __attribute__((noinline))
#else
#define NOINLINE
#endif

typedef struct {
    uint64_t word[4];
} Payload;

typedef struct {
    Payload slot[CAPACITY];
} FullStorage;

typedef struct {
    size_t len;
    size_t cap;
    Payload slot[CAPACITY];
} PrefixStorage;

typedef struct {
    size_t head;
    size_t len;
    size_t cap;
    Payload slot[CAPACITY];
} RingStorage;

typedef struct {
    uint64_t valid[BIT_WORDS];
    Payload slot[CAPACITY];
} SeparateNullable;

/* This is an explicit C tag plus payload. It is not Rust Option layout: Rust
 * may use niches or make other target-specific layout choices. */
typedef struct {
    uint8_t tag;
    Payload payload;
} TaggedSlot;

typedef struct {
    TaggedSlot slot[CAPACITY];
} TaggedNullable;

typedef struct {
    uint64_t checksum;
    size_t count;
} CopyResult;

typedef enum {
    PATTERN_DENSE,
    PATTERN_SPARSE,
    PATTERN_CLUSTERED
} Pattern;

static volatile uint64_t published_checksum;

_Static_assert(sizeof(Payload) == 32, "the experiment requires a 32-byte payload");
_Static_assert(CAPACITY % 64u == 0, "the packed validity map must have whole words");

static void fail(const char *message) {
    fprintf(stderr, "layout: %s\n", message);
    exit(1);
}

static void *checked_alloc(size_t bytes) {
    void *result = malloc(bytes);
    if (result == NULL) {
        fail("allocation failed");
    }
    return result;
}

static Payload payload_at(size_t logical_index) {
    uint64_t x = (uint64_t)logical_index + UINT64_C(0x9e3779b97f4a7c15);
    Payload payload;
    for (size_t word = 0; word < 4; ++word) {
        x ^= x >> 30;
        x *= UINT64_C(0xbf58476d1ce4e5b9);
        x ^= x >> 27;
        x *= UINT64_C(0x94d049bb133111eb);
        x ^= x >> 31;
        payload.word[word] = x + (uint64_t)word;
    }
    return payload;
}

static uint64_t include_payload(uint64_t checksum, const Payload *payload) {
    for (size_t word = 0; word < 4; ++word) {
        checksum ^= payload->word[word] + UINT64_C(0x9e3779b97f4a7c15) +
                    (checksum << 6) + (checksum >> 2);
    }
    return checksum;
}

static CopyResult empty_result(void) {
    CopyResult result = {UINT64_C(0xcbf29ce484222325), 0};
    return result;
}

static int payload_equal(const Payload *left, const Payload *right) {
    return memcmp(left, right, sizeof(*left)) == 0;
}

static NOINLINE CopyResult copy_full(
    const Payload *source,
    size_t capacity,
    Payload *destination
) {
    CopyResult result = empty_result();
    for (size_t index = 0; index < capacity; ++index) {
        destination[result.count] = source[index];
        result.checksum = include_payload(result.checksum, &destination[result.count]);
        ++result.count;
    }
    return result;
}

static NOINLINE CopyResult copy_prefix(
    const PrefixStorage *source,
    Payload *destination
) {
    if (source->len > source->cap || source->cap > CAPACITY) {
        fail("invalid prefix state");
    }
    return copy_full(source->slot, source->len, destination);
}

static NOINLINE CopyResult copy_ring(
    const RingStorage *source,
    Payload *destination
) {
    if (source->len > source->cap || source->cap > CAPACITY ||
        (source->cap != 0 && source->head >= source->cap)) {
        fail("invalid ring state");
    }
    CopyResult result = empty_result();
    for (size_t logical = 0; logical < source->len; ++logical) {
        size_t physical = source->head + logical;
        if (physical >= source->cap) {
            physical -= source->cap;
        }
        destination[result.count] = source->slot[physical];
        result.checksum = include_payload(result.checksum, &destination[result.count]);
        ++result.count;
    }
    return result;
}

static unsigned trailing_zeroes(uint64_t value) {
#if defined(__clang__) || defined(__GNUC__)
    return (unsigned)__builtin_ctzll(value);
#else
    unsigned count = 0;
    while ((value & UINT64_C(1)) == 0) {
        value >>= 1;
        ++count;
    }
    return count;
#endif
}

static NOINLINE CopyResult copy_separate_nullable(
    const SeparateNullable *source,
    size_t capacity,
    Payload *destination
) {
    if (capacity > CAPACITY) {
        fail("invalid nullable capacity");
    }
    CopyResult result = empty_result();
    size_t words = (capacity + 63u) / 64u;
    for (size_t word = 0; word < words; ++word) {
        uint64_t present = source->valid[word];
        if (word + 1 == words && (capacity & 63u) != 0) {
            present &= (UINT64_C(1) << (capacity & 63u)) - 1;
        }
        while (present != 0) {
            unsigned bit = trailing_zeroes(present);
            size_t index = word * 64u + bit;
            destination[result.count] = source->slot[index];
            result.checksum = include_payload(result.checksum, &destination[result.count]);
            ++result.count;
            present &= present - 1;
        }
    }
    return result;
}

static NOINLINE CopyResult copy_tagged_nullable(
    const TaggedNullable *source,
    size_t capacity,
    Payload *destination
) {
    if (capacity > CAPACITY) {
        fail("invalid nullable capacity");
    }
    CopyResult result = empty_result();
    for (size_t index = 0; index < capacity; ++index) {
        if (source->slot[index].tag != 0) {
            destination[result.count] = source->slot[index].payload;
            result.checksum = include_payload(result.checksum, &destination[result.count]);
            ++result.count;
        }
    }
    return result;
}

static int pattern_contains(Pattern pattern, size_t index) {
    switch (pattern) {
        case PATTERN_DENSE:
            return index % 8u != 0;
        case PATTERN_SPARSE:
            return index % 16u == 0;
        case PATTERN_CLUSTERED:
            /* Eight 32-element clusters: the same population as sparse. */
            return index % 512u < 32u;
    }
    fail("unknown nullable pattern");
    return 0;
}

static const char *pattern_name(Pattern pattern) {
    switch (pattern) {
        case PATTERN_DENSE:
            return "dense";
        case PATTERN_SPARSE:
            return "sparse";
        case PATTERN_CLUSTERED:
            return "clustered";
    }
    fail("unknown nullable pattern");
    return "invalid";
}

static size_t initialize_nullable(
    SeparateNullable *separate,
    TaggedNullable *tagged,
    Pattern pattern
) {
    memset(separate->valid, 0, sizeof(separate->valid));
    for (size_t index = 0; index < CAPACITY; ++index) {
        tagged->slot[index].tag = 0;
    }
    size_t present = 0;
    for (size_t index = 0; index < CAPACITY; ++index) {
        if (!pattern_contains(pattern, index)) {
            continue;
        }
        Payload payload = payload_at(index);
        separate->slot[index] = payload;
        separate->valid[index / 64u] |= UINT64_C(1) << (index % 64u);
        tagged->slot[index].payload = payload;
        tagged->slot[index].tag = 1;
        ++present;
    }
    return present;
}

static void verify_sequence(const Payload *values, size_t count) {
    for (size_t index = 0; index < count; ++index) {
        Payload expected = payload_at(index);
        if (!payload_equal(&values[index], &expected)) {
            fail("logical sequence differs from its construction witness");
        }
    }
}

static void verify_foundation_cases(void) {
    FullStorage *full = checked_alloc(sizeof(*full));
    PrefixStorage *prefix = checked_alloc(sizeof(*prefix));
    RingStorage *ring = checked_alloc(sizeof(*ring));
    Payload *left = checked_alloc(sizeof(Payload) * CAPACITY);
    Payload *right = checked_alloc(sizeof(Payload) * CAPACITY);

    prefix->len = CAPACITY;
    prefix->cap = CAPACITY;
    ring->head = CAPACITY - 37u;
    ring->len = CAPACITY;
    ring->cap = CAPACITY;
    for (size_t logical = 0; logical < CAPACITY; ++logical) {
        Payload payload = payload_at(logical);
        full->slot[logical] = payload;
        prefix->slot[logical] = payload;
        ring->slot[(ring->head + logical) % CAPACITY] = payload;
    }

    CopyResult full_result = copy_full(full->slot, CAPACITY, left);
    CopyResult prefix_result = copy_prefix(prefix, right);
    if (full_result.count != CAPACITY || prefix_result.count != CAPACITY ||
        full_result.checksum != prefix_result.checksum ||
        memcmp(left, right, sizeof(Payload) * CAPACITY) != 0) {
        fail("full and full-prefix witnesses differ");
    }
    CopyResult ring_result = copy_ring(ring, right);
    if (ring_result.count != CAPACITY ||
        full_result.checksum != ring_result.checksum ||
        memcmp(left, right, sizeof(Payload) * CAPACITY) != 0) {
        fail("full and wrapped-ring witnesses differ");
    }
    verify_sequence(left, CAPACITY);

    /* Zero extent must not dereference any pointer. */
    CopyResult zero = copy_full(NULL, 0, NULL);
    if (zero.count != 0 || zero.checksum != empty_result().checksum) {
        fail("zero-extent full copy failed");
    }

    prefix->len = 0;
    CopyResult empty_prefix = copy_prefix(prefix, left);
    ring->head = CAPACITY - 1u;
    ring->len = 0;
    CopyResult empty_ring = copy_ring(ring, left);
    if (empty_prefix.count != 0 || empty_ring.count != 0) {
        fail("empty prefix or wrapped empty ring copied a payload");
    }

    prefix->len = CAPACITY / 4u;
    CopyResult partial_prefix = copy_prefix(prefix, left);
    if (partial_prefix.count != CAPACITY / 4u) {
        fail("partial prefix count differs");
    }
    verify_sequence(left, partial_prefix.count);

    ring->head = CAPACITY - 5u;
    ring->len = 17u;
    for (size_t logical = 0; logical < ring->len; ++logical) {
        ring->slot[(ring->head + logical) % CAPACITY] = payload_at(logical);
    }
    CopyResult wrapped = copy_ring(ring, left);
    if (wrapped.count != 17u) {
        fail("wrapped ring count differs");
    }
    verify_sequence(left, wrapped.count);

    free(right);
    free(left);
    free(ring);
    free(prefix);
    free(full);
}

static void verify_nullable_cases(void) {
    SeparateNullable *separate = checked_alloc(sizeof(*separate));
    TaggedNullable *tagged = checked_alloc(sizeof(*tagged));
    Payload *left = checked_alloc(sizeof(Payload) * CAPACITY);
    Payload *right = checked_alloc(sizeof(Payload) * CAPACITY);

    for (Pattern pattern = PATTERN_DENSE; pattern <= PATTERN_CLUSTERED; ++pattern) {
        size_t expected = initialize_nullable(separate, tagged, pattern);
        CopyResult a = copy_separate_nullable(separate, CAPACITY, left);
        CopyResult b = copy_tagged_nullable(tagged, CAPACITY, right);
        if (a.count != expected || b.count != expected || a.checksum != b.checksum ||
            memcmp(left, right, expected * sizeof(Payload)) != 0) {
            fail("nullable encodings disagree");
        }
    }

    memset(separate->valid, 0, sizeof(separate->valid));
    memset(tagged, 0, sizeof(*tagged));
    CopyResult zero_separate = copy_separate_nullable(separate, 0, NULL);
    CopyResult zero_tagged = copy_tagged_nullable(tagged, 0, NULL);
    CopyResult empty_separate = copy_separate_nullable(separate, CAPACITY, left);
    CopyResult empty_tagged = copy_tagged_nullable(tagged, CAPACITY, right);
    if (zero_separate.count != 0 || zero_tagged.count != 0 ||
        empty_separate.count != 0 || empty_tagged.count != 0 ||
        zero_separate.checksum != zero_tagged.checksum ||
        empty_separate.checksum != empty_tagged.checksum) {
        fail("zero-extent or empty nullable witness failed");
    }

    free(right);
    free(left);
    free(tagged);
    free(separate);
}

static double elapsed_ns(struct timespec start, struct timespec stop) {
    return (double)(stop.tv_sec - start.tv_sec) * 1e9 +
           (double)(stop.tv_nsec - start.tv_nsec);
}

typedef CopyResult (*CopyFunction)(const void *, Payload *);

static NOINLINE CopyResult timed_full_adapter(const void *source, Payload *destination) {
    return copy_full(((const FullStorage *)source)->slot, CAPACITY, destination);
}

static NOINLINE CopyResult timed_prefix_adapter(const void *source, Payload *destination) {
    return copy_prefix((const PrefixStorage *)source, destination);
}

static NOINLINE CopyResult timed_ring_adapter(const void *source, Payload *destination) {
    return copy_ring((const RingStorage *)source, destination);
}

static const SeparateNullable *timed_separate_source;
static const TaggedNullable *timed_tagged_source;

static NOINLINE CopyResult timed_separate_adapter(const void *unused, Payload *destination) {
    (void)unused;
    return copy_separate_nullable(timed_separate_source, CAPACITY, destination);
}

static NOINLINE CopyResult timed_tagged_adapter(const void *unused, Payload *destination) {
    (void)unused;
    return copy_tagged_nullable(timed_tagged_source, CAPACITY, destination);
}

static double time_copies(
    CopyFunction function,
    const void *source,
    Payload *destination,
    size_t iterations,
    uint64_t salt
) {
    struct timespec start;
    struct timespec stop;
    uint64_t witness = salt;
    if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) {
        fail("clock_gettime start failed");
    }
    for (size_t iteration = 0; iteration < iterations; ++iteration) {
        CopyResult result = function(source, destination);
        witness ^= result.checksum + (uint64_t)result.count + (uint64_t)iteration;
    }
    if (clock_gettime(CLOCK_MONOTONIC, &stop) != 0) {
        fail("clock_gettime stop failed");
    }
    published_checksum ^= witness;
    return elapsed_ns(start, stop);
}

static void warm_copies(CopyFunction function, const void *source, Payload *destination) {
    uint64_t witness = 0;
    for (size_t iteration = 0; iteration < 16; ++iteration) {
        CopyResult result = function(source, destination);
        witness ^= result.checksum + (uint64_t)result.count + (uint64_t)iteration;
    }
    published_checksum ^= witness;
}

static void print_layouts(void) {
    size_t tagged_payload_offset = offsetof(TaggedSlot, payload);
    size_t tagged_padding = sizeof(TaggedNullable) -
                            CAPACITY * (sizeof(Payload) + sizeof(uint8_t));
    printf("layout,full,layout_bytes=%zu,peak_backing_bytes=%zu,payload_capacity_bytes=%zu,metadata_bytes=0,allocations=1,allocator_overhead=unmeasured\n",
           sizeof(FullStorage), sizeof(FullStorage), CAPACITY * sizeof(Payload));
    printf("layout,prefix,layout_bytes=%zu,peak_backing_bytes=%zu,payload_capacity_bytes=%zu,metadata_bytes=%zu,allocations=1,allocator_overhead=unmeasured\n",
           sizeof(PrefixStorage), sizeof(PrefixStorage), CAPACITY * sizeof(Payload),
           offsetof(PrefixStorage, slot));
    printf("layout,ring,layout_bytes=%zu,peak_backing_bytes=%zu,payload_capacity_bytes=%zu,metadata_bytes=%zu,allocations=1,allocator_overhead=unmeasured\n",
           sizeof(RingStorage), sizeof(RingStorage), CAPACITY * sizeof(Payload),
           offsetof(RingStorage, slot));
    printf("layout,nullable_separate,layout_bytes=%zu,peak_backing_bytes=%zu,payload_capacity_bytes=%zu,validity_bytes=%zu,padding_bytes=0,allocations=1,allocator_overhead=unmeasured\n",
           sizeof(SeparateNullable), sizeof(SeparateNullable), CAPACITY * sizeof(Payload),
           sizeof(((SeparateNullable *)0)->valid));
    printf("layout,nullable_c_tagged,layout_bytes=%zu,peak_backing_bytes=%zu,payload_capacity_bytes=%zu,validity_bytes=%zu,padding_bytes=%zu,tagged_slot_bytes=%zu,tag_to_payload_offset=%zu,allocations=1,allocator_overhead=unmeasured\n",
           sizeof(TaggedNullable), sizeof(TaggedNullable), CAPACITY * sizeof(Payload),
           CAPACITY * sizeof(uint8_t), tagged_padding, sizeof(TaggedSlot),
           tagged_payload_offset);
}

static void print_structural_cases(void) {
    printf("case,full,live=%u,initialized_payload_bytes=%zu,validity_initialization_bytes=0,payload_bytes_copied=%zu\n",
           CAPACITY, CAPACITY * sizeof(Payload), CAPACITY * sizeof(Payload));
    printf("case,prefix_full,live=%u,initialized_payload_bytes=%zu,metadata_initialization_bytes=%zu,payload_bytes_copied=%zu\n",
           CAPACITY, CAPACITY * sizeof(Payload), offsetof(PrefixStorage, slot),
           CAPACITY * sizeof(Payload));
    printf("case,prefix_quarter,live=%u,initialized_payload_bytes=%zu,metadata_initialization_bytes=%zu,payload_bytes_copied=%zu\n",
           CAPACITY / 4u, (CAPACITY / 4u) * sizeof(Payload),
           offsetof(PrefixStorage, slot), (CAPACITY / 4u) * sizeof(Payload));
    printf("case,ring_full_wrapped,live=%u,initialized_payload_bytes=%zu,metadata_initialization_bytes=%zu,payload_bytes_copied=%zu\n",
           CAPACITY, CAPACITY * sizeof(Payload), offsetof(RingStorage, slot),
           CAPACITY * sizeof(Payload));
    printf("case,ring_wrapped_17,live=17,initialized_payload_bytes=%zu,metadata_initialization_bytes=%zu,payload_bytes_copied=%zu\n",
           17u * sizeof(Payload), offsetof(RingStorage, slot), 17u * sizeof(Payload));
    for (Pattern pattern = PATTERN_DENSE; pattern <= PATTERN_CLUSTERED; ++pattern) {
        size_t present = 0;
        for (size_t index = 0; index < CAPACITY; ++index) {
            present += (size_t)pattern_contains(pattern, index);
        }
        printf("case,nullable_%s,live=%zu,initialized_payload_bytes=%zu,separate_validity_initialization_bytes=%zu,c_tagged_validity_initialization_bytes=%u,payload_bytes_copied=%zu\n",
               pattern_name(pattern), present, present * sizeof(Payload),
               sizeof(((SeparateNullable *)0)->valid), CAPACITY,
               present * sizeof(Payload));
    }
}

static void benchmark(size_t iterations, size_t samples) {
    FullStorage *full = checked_alloc(sizeof(*full));
    PrefixStorage *prefix = checked_alloc(sizeof(*prefix));
    RingStorage *ring = checked_alloc(sizeof(*ring));
    SeparateNullable *separate = checked_alloc(sizeof(*separate));
    TaggedNullable *tagged = checked_alloc(sizeof(*tagged));
    Payload *destination = checked_alloc(sizeof(Payload) * CAPACITY);

    prefix->len = CAPACITY;
    prefix->cap = CAPACITY;
    ring->head = CAPACITY - 37u;
    ring->len = CAPACITY;
    ring->cap = CAPACITY;
    for (size_t logical = 0; logical < CAPACITY; ++logical) {
        Payload payload = payload_at(logical);
        full->slot[logical] = payload;
        prefix->slot[logical] = payload;
        ring->slot[(ring->head + logical) % CAPACITY] = payload;
    }

    CopyFunction functions[3] = {
        timed_full_adapter, timed_prefix_adapter, timed_ring_adapter
    };
    const void *sources[3] = {full, prefix, ring};
    const char *names[3] = {"full", "prefix_full", "ring_full_wrapped"};
    for (size_t which = 0; which < 3; ++which) {
        warm_copies(functions[which], sources[which], destination);
    }
    for (size_t sample = 0; sample < samples; ++sample) {
        for (size_t turn = 0; turn < 3; ++turn) {
            size_t which = (sample + turn) % 3;
            double ns = time_copies(
                functions[which], sources[which], destination, iterations,
                ((uint64_t)sample << 32) ^ (uint64_t)which
            );
            printf("timing,%s,sample=%zu,iterations=%zu,elapsed_ns=%.0f,ns_per_copy=%.3f\n",
                   names[which], sample, iterations, ns, ns / (double)iterations);
        }
    }

    for (Pattern pattern = PATTERN_DENSE; pattern <= PATTERN_CLUSTERED; ++pattern) {
        initialize_nullable(separate, tagged, pattern);
        timed_separate_source = separate;
        timed_tagged_source = tagged;
        warm_copies(timed_separate_adapter, NULL, destination);
        warm_copies(timed_tagged_adapter, NULL, destination);
        for (size_t sample = 0; sample < samples; ++sample) {
            CopyFunction pair[2] = {timed_separate_adapter, timed_tagged_adapter};
            const char *pair_names[2] = {"separate", "c_tagged"};
            for (size_t turn = 0; turn < 2; ++turn) {
                size_t which = (sample + turn) % 2;
                double ns = time_copies(
                    pair[which], NULL, destination, iterations,
                    ((uint64_t)pattern << 48) ^ ((uint64_t)sample << 32) ^ which
                );
                printf("timing,nullable_%s_%s,sample=%zu,iterations=%zu,elapsed_ns=%.0f,ns_per_copy=%.3f\n",
                       pattern_name(pattern), pair_names[which], sample,
                       iterations, ns, ns / (double)iterations);
            }
        }
    }

    printf("witness,published_checksum=%" PRIu64 "\n", published_checksum);
    free(destination);
    free(tagged);
    free(separate);
    free(ring);
    free(prefix);
    free(full);
}

static size_t parse_count(const char *text, const char *name) {
    errno = 0;
    char *end = NULL;
    unsigned long long value = strtoull(text, &end, 10);
    if (errno != 0 || end == text || *end != '\0' || value == 0 || value > SIZE_MAX) {
        fprintf(stderr, "layout: invalid %s: %s\n", name, text);
        exit(2);
    }
    return (size_t)value;
}

int main(int argc, char **argv) {
    if (argc > 3) {
        fprintf(stderr, "usage: layout [iterations [samples]]\n");
        return 2;
    }
    size_t iterations = argc >= 2 ? parse_count(argv[1], "iterations") : DEFAULT_ITERATIONS;
    size_t samples = argc >= 3 ? parse_count(argv[2], "samples") : DEFAULT_SAMPLES;

    verify_foundation_cases();
    verify_nullable_cases();
    print_layouts();
    print_structural_cases();
    benchmark(iterations, samples);
    return 0;
}
