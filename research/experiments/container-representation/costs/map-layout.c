/* Same-algorithm native map representation and validation-cost controls.
 * These C implementations are not checked Whitefoot implementations. */
#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
#include <inttypes.h>
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
#if defined(_MSC_VER)
#define NOINLINE __declspec(noinline)
#else
#define NOINLINE __attribute__((noinline))
#endif

typedef struct { uint64_t key, value[3]; } Entry;
typedef struct { uint8_t state; Entry entry; } Tagged;
typedef struct {
    size_t capacity, payload_capacity;
    uint8_t *control;
    Entry *payload;
    Tagged *tagged;
} Maps;
enum { EMPTY, LIVE, DELETED };
static volatile uint64_t observed;

static void require(int condition, const char *message) {
    if (!condition) { fprintf(stderr, "map-layout: %s\n", message); exit(1); }
}
static void *allocate(size_t bytes) {
    void *p = malloc(bytes);
    require(p != NULL, "allocation failed");
    return p;
}
static uint64_t mix(uint64_t x) {
    x ^= x >> 30; x *= UINT64_C(0xbf58476d1ce4e5b9);
    x ^= x >> 27; x *= UINT64_C(0x94d049bb133111eb);
    return x ^ (x >> 31);
}
static uint64_t key_at(size_t at) { return (uint64_t)at * 2 + 1; }
static uint64_t value_at(uint64_t key) { return mix(key + 91); }
static uint64_t now_ns(void) {
#if defined(_WIN32)
    LARGE_INTEGER count, frequency;
    require(QueryPerformanceCounter(&count) != 0, "clock counter");
    require(QueryPerformanceFrequency(&frequency) != 0, "clock frequency");
    return (uint64_t)((long double)count.QuadPart * 1.0e9L / frequency.QuadPart);
#else
    struct timespec t;
    require(clock_gettime(CLOCK_MONOTONIC, &t) == 0, "clock");
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
#endif
}
static Maps create(size_t capacity) {
    require(capacity && !(capacity & (capacity - 1)), "power-of-two capacity");
    Maps m = {capacity, capacity, allocate(capacity),
              allocate(capacity * sizeof(Entry)), allocate(capacity * sizeof(Tagged))};
    memset(m.control, EMPTY, capacity);
    /* Only tags are initialized; the control must never read an absent payload. */
    for (size_t i = 0; i < capacity; ++i) m.tagged[i].state = EMPTY;
    return m;
}
static void release(Maps *m) { free(m->control); free(m->payload); free(m->tagged); }
static int insert(Maps *m, uint64_t key) {
    size_t at = (size_t)mix(key) & (m->capacity - 1), first_deleted = m->capacity;
    for (size_t visited = 0; visited < m->capacity; ++visited) {
        uint8_t state = m->control[at];
        if (state == LIVE && m->payload[at].key == key) {
            m->payload[at].value[0] = value_at(key);
            m->tagged[at].entry = m->payload[at];
            return 1;
        }
        if (state == DELETED && first_deleted == m->capacity) first_deleted = at;
        if (state == EMPTY) {
            if (first_deleted != m->capacity) at = first_deleted;
            Entry e = {key, {value_at(key), key ^ 33, key ^ 77}};
            m->payload[at] = e; m->tagged[at].entry = e;
            m->control[at] = LIVE; m->tagged[at].state = LIVE;
            return 1;
        }
        at = (at + 1) & (m->capacity - 1);
    }
    if (first_deleted != m->capacity) {
        Entry e = {key, {value_at(key), key ^ 33, key ^ 77}};
        m->payload[first_deleted] = e; m->tagged[first_deleted].entry = e;
        m->control[first_deleted] = LIVE; m->tagged[first_deleted].state = LIVE;
        return 1;
    }
    return 0;
}
static int erase(Maps *m, uint64_t key) {
    size_t at = (size_t)mix(key) & (m->capacity - 1);
    for (size_t visited = 0; visited < m->capacity; ++visited) {
        if (m->control[at] == EMPTY) return 0;
        if (m->control[at] == LIVE && m->payload[at].key == key) {
            m->control[at] = DELETED; m->tagged[at].state = DELETED; return 1;
        }
        at = (at + 1) & (m->capacity - 1);
    }
    return 0;
}

/* -1 is an intended malformed-view result. It is not an impossible-case trap.
 * This validates extents, not arbitrary initializedness or ownership. */
static NOINLINE int lookup_checked(const Maps *m, uint64_t key, uint64_t *value) {
    size_t at = (size_t)mix(key) & (m->capacity - 1);
    for (size_t visited = 0; visited < m->capacity; ++visited) {
        if (m->control[at] == EMPTY) return 0;
        if (m->control[at] == LIVE) {
            if (at >= m->payload_capacity) return -1;
            if (m->payload[at].key == key) { *value = m->payload[at].value[0]; return 1; }
        }
        at = (at + 1) & (m->capacity - 1);
    }
    return 0;
}
static NOINLINE int lookup_split(const Maps *m, uint64_t key, uint64_t *value) {
    size_t at = (size_t)mix(key) & (m->capacity - 1);
    for (size_t visited = 0; visited < m->capacity; ++visited) {
        if (m->control[at] == EMPTY) return 0;
        if (m->control[at] == LIVE && m->payload[at].key == key) {
            *value = m->payload[at].value[0]; return 1;
        }
        at = (at + 1) & (m->capacity - 1);
    }
    return 0;
}
static NOINLINE int lookup_tagged(const Maps *m, uint64_t key, uint64_t *value) {
    size_t at = (size_t)mix(key) & (m->capacity - 1);
    for (size_t visited = 0; visited < m->capacity; ++visited) {
        if (m->tagged[at].state == EMPTY) return 0;
        if (m->tagged[at].state == LIVE && m->tagged[at].entry.key == key) {
            *value = m->tagged[at].entry.value[0]; return 1;
        }
        at = (at + 1) & (m->capacity - 1);
    }
    return 0;
}
typedef int (*Lookup)(const Maps *, uint64_t, uint64_t *);
static const Lookup lookups[] = {lookup_tagged, lookup_split, lookup_checked, lookup_split};
static const char *names[] = {"tagged", "split", "split_check_each", "split_validate_batch"};
static NOINLINE uint64_t batch(const Maps *m, const uint64_t *queries, size_t count, size_t variant) {
    /* A volatile observation preserves repeated calls across the timed boundary. */
    (void)observed;
    if (variant == 3 && m->payload_capacity < m->capacity) return UINT64_MAX;
    uint64_t checksum = 0;
    /* Match Rust's direct noinline lookup boundary, dispatching once per batch. */
#define RUN_LOOKUP(function) \
    for (size_t i = 0; i < count; ++i) { \
        uint64_t value = 0; \
        int found = function(m, queries[i], &value); \
        if (found < 0) return UINT64_MAX; \
        checksum += found ? value : UINT64_C(0x9e3779b97f4a7c15); \
    }
    switch (variant) {
        case 0: RUN_LOOKUP(lookup_tagged); break;
        case 1: case 3: RUN_LOOKUP(lookup_split); break;
        case 2: RUN_LOOKUP(lookup_checked); break;
        default: require(0, "invalid variant");
    }
#undef RUN_LOOKUP
    return checksum;
}
static void verify(void) {
    for (size_t capacity = 1; capacity <= 64; capacity *= 2) {
        Maps m = create(capacity);
        unsigned char present[128] = {0};
        for (size_t i = 0; i < capacity; ++i) { require(insert(&m, key_at(i)), "fill"); present[i] = 1; }
        require(!insert(&m, key_at(capacity)), "full refusal");
        require(insert(&m, key_at(0)), "replace in full table");
        for (size_t i = 0; i < capacity; i += 2) { require(erase(&m, key_at(i)), "remove"); present[i] = 0; }
        for (size_t i = capacity; i < capacity + (capacity + 1) / 2; ++i) {
            require(insert(&m, key_at(i)), "reuse tombstone"); present[i] = 1;
        }
        for (size_t i = 0; i < capacity * 2; ++i) {
            for (size_t v = 0; v < 3; ++v) {
                uint64_t value = 0;
                int found = lookups[v](&m, key_at(i), &value);
                require(found == present[i], "independent membership oracle");
                require(!found || value == value_at(key_at(i)), "independent value oracle");
            }
        }
        uint64_t queries[] = {key_at(0), key_at(capacity), key_at(capacity * 3)};
        uint64_t expected = batch(&m, queries, 3, 0);
        for (size_t v = 1; v < 4; ++v) require(batch(&m, queries, 3, v) == expected, "batch equality");
        m.payload_capacity = 0;
        require(batch(&m, queries, 3, 3) == UINT64_MAX, "reject malformed batch extent");
        uint64_t value = 0;
        require(lookup_checked(&m, key_at(capacity), &value) == -1, "reject malformed live extent");
        release(&m);
    }
    printf("verified,full,replace,remove,reuse,missing,malformed,7_capacities\n");
}
static void measure(void) {
    const size_t capacities[] = {256, 65536};
    const size_t eighths[] = {2, 7};
    const size_t count = 32768, repeats = 12, samples = 7;
    printf("kind,capacity,load_eighths,variant,sample,queries,nanoseconds,checksum,backing_bytes\n");
    for (size_t c = 0; c < 2; ++c) for (size_t l = 0; l < 2; ++l) {
        Maps m = create(capacities[c]);
        size_t live = m.capacity * eighths[l] / 8;
        for (size_t i = 0; i < live; ++i) require(insert(&m, key_at(i)), "measurement fill");
        uint64_t *queries = allocate(count * sizeof(uint64_t));
        uint64_t state = 7640891576956012809ULL, expected = 0;
        for (size_t i = 0; i < count; ++i) {
            state = mix(state + i);
            size_t chosen = (size_t)(state % (live * 2));
            queries[i] = key_at(chosen);
            expected += chosen < live ? value_at(queries[i]) : UINT64_C(0x9e3779b97f4a7c15);
        }
        for (size_t v = 0; v < 4; ++v) require(batch(&m, queries, count, v) == expected, "query oracle");
        for (size_t s = 0; s < samples; ++s) for (size_t position = 0; position < 4; ++position) {
            size_t v = (position + s) % 4;
            observed = batch(&m, queries, count, v);
            uint64_t sum = 0, start = now_ns();
            for (size_t repeat = 0; repeat < repeats; ++repeat) sum += batch(&m, queries, count, v);
            uint64_t elapsed = now_ns() - start;
            observed = sum;
            require(sum == expected * repeats, "timed checksum");
            size_t bytes = m.capacity * (v == 0 ? sizeof(Tagged) : sizeof(Entry) + 1);
            printf("sample,%zu,%zu,%s,%zu,%zu,%" PRIu64 ",%" PRIu64 ",%zu\n",
                   m.capacity, eighths[l], names[v], s, count * repeats, elapsed, sum, bytes);
        }
        free(queries); release(&m);
    }
}
int main(int argc, char **argv) {
    require(argc == 2, "expected check or measure");
    if (!strcmp(argv[1], "check")) verify();
    else if (!strcmp(argv[1], "measure")) { measure(); }
    else require(0, "unknown command");
    return 0;
}
