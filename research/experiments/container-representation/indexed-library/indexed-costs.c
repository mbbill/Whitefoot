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
#define PUBLIC NOINLINE
#define CALLBACK NOINLINE
#define CONTRACT "retained"
#else
#define PUBLIC
#define CALLBACK
#define CONTRACT "normal"
#endif
enum { CEILING = 8192, RECORD_WORDS = 32, VARIANT_COUNT = 3 };
enum Status { DONE, ALREADY, MISSING, EXPIRED, WRONG_STORE, CONFLICT, FULL,
              UNSCHEDULED, INVALID_POSITION, BUSY };
typedef struct { uint64_t index, generation; } Handle;
typedef struct { uint64_t store_id; Handle object; } StoreHandle;
typedef struct { uint64_t deadline, id; Handle handle; } Due;
typedef struct { uint64_t words[RECORD_WORDS]; } Wide;
typedef struct { unsigned char unused; } Empty;
typedef struct { uint64_t modulus; } Hash;
static const Empty empty = {0};
typedef struct { uint64_t length, capacity, data[]; } Words;
typedef struct { Words *values; uint64_t digest, used; } Log;
typedef struct { uint64_t id; bool member; uint64_t position; } Info;
// The source backend keeps variant payloads in separate fields. These public
// result extents therefore do not use a compact C union representation.
typedef struct { uint32_t failed; StoreHandle handle; enum Status error; } Lookup;
typedef struct { uint32_t failed; Info value; enum Status error; } InfoResult;
typedef struct { uint32_t failed; uint64_t position; enum Status error; } Position;
typedef struct { uint32_t failed; unsigned char unit; enum Status error; } VisitResult;
typedef struct { uint64_t length, capacity; Due data[]; } Heap;
typedef struct { uint32_t tag; uint64_t key; Handle value; } Bucket;
typedef struct { uint64_t length, capacity; Bucket data[]; } Buckets;
typedef struct { Buckets *cells; uint64_t length; } Map;
typedef struct { uint32_t failed; Handle value; unsigned char unit; } MapLookup;
typedef struct { uint64_t key; Handle value; } MapPair;
typedef struct { uint32_t returned; bool reason; MapPair pair; } MapPut;

static Lookup lookup_ok(StoreHandle handle) { return (Lookup){.failed = 0, .handle = handle}; }
static Lookup lookup_error(enum Status error) { return (Lookup){.failed = 1, .error = error}; }
static enum Status lookup_status(Lookup result) { return result.failed ? result.error : DONE; }
static InfoResult info_ok(Info value) { return (InfoResult){.failed = 0, .value = value}; }
static InfoResult info_error(enum Status error) { return (InfoResult){.failed = 1, .error = error}; }
static enum Status info_status(InfoResult result) { return result.failed ? result.error : DONE; }
static Position position_ok(uint64_t position) { return (Position){.failed = 0, .position = position}; }
static Position position_error(enum Status error) { return (Position){.failed = 1, .error = error}; }
static enum Status position_status(Position result) { return result.failed ? result.error : DONE; }
static unsigned map_put_kind(MapPut result) { return result.returned ? result.reason + 1 : 0; }

#if defined(AUDIT_EVENTS)
static uint64_t comparisons, swaps, heap_assignments, reports, validations,
                store_checks, handle_equalities, probes, payload_assignments;
#define EVENT(counter, amount) ((counter) += (uint64_t)(amount))
#else
#define EVENT(counter, amount) ((void)0)
#endif
typedef union {
    struct { size_t bytes; uint64_t identity, magic; } value;
    max_align_t alignment;
} AllocationHeader;
static size_t requests, releases, requested_bytes, live_bytes, peak_bytes;
static uint64_t allocation_identity, release_identity;
static bool allocation_live[65536];
static volatile uint64_t observed;
static const char *const policies[] = {"weak", "retained"};
static const char *const paths[] = {"reserved-mixed", "growth-cleanup"};
static const char *const variants[] = {"whitefoot", "swap-c", "hole-c"};

static void require(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "indexed library costs: %s\n", message); exit(1); }
}
NOINLINE void *wf_cost_allocate(uint64_t bytes) {
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failure");
    header->value.bytes = (size_t)bytes;
    header->value.identity = ++allocation_identity;
    require(allocation_identity < sizeof allocation_live / sizeof allocation_live[0], "allocation ledger extent");
    require(!allocation_live[allocation_identity], "fresh allocation identity");
    allocation_live[allocation_identity] = true;
    header->value.magic = UINT64_C(0x696e646578636f73);
    ++requests; requested_bytes += (size_t)bytes; live_bytes += (size_t)bytes;
    if (live_bytes > peak_bytes) peak_bytes = live_bytes;
    return header + 1;
}
NOINLINE void wf_cost_release(void *pointer) {
    if (pointer == NULL) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.magic == UINT64_C(0x696e646578636f73), "allocation identity");
    require(header->value.identity <= allocation_identity && allocation_live[header->value.identity], "one release per allocation identity");
    allocation_live[header->value.identity] = false;
    require(live_bytes >= header->value.bytes, "live-byte accounting");
    release_identity ^= header->value.identity;
    ++releases; live_bytes -= header->value.bytes;
    header->value.magic = 0; free(header);
}
static uint64_t xor_prefix(uint64_t n) {
    switch (n % 4) { case 0: return n; case 1: return 1; case 2: return n + 1; default: return 0; }
}
static void reset_accounting(void) {
    require(live_bytes == 0, "allocation left live between traces");
    requests = releases = requested_bytes = peak_bytes = 0;
    allocation_identity = release_identity = 0;
}
static void check_accounting(void) {
    require(live_bytes == 0 && requests == releases, "complete backing cleanup");
    require(release_identity == xor_prefix(allocation_identity), "allocation/release identities");
}
static void word(Log *log, uint64_t value) {
    log->digest = log->digest * UINT64_C(131) + value; ++log->used;
    if (log->values->length < log->values->capacity)
        log->values->data[log->values->length++] = value;
}
static void outcome(Log *log, uint64_t operation, uint64_t id, enum Status status) {
    word(log, 1); word(log, operation); word(log, id); word(log, (uint64_t)status);
}
static void log_handle(Log *log, StoreHandle handle) {
    word(log, 2); word(log, handle.store_id); word(log, handle.object.index); word(log, handle.object.generation);
}
static void offer(Log *log, uint64_t identity) { word(log, 3); word(log, identity); }
static bool handle_equal(Handle a, Handle b) {
    EVENT(handle_equalities, 1);
    return a.index == b.index && a.generation == b.generation;
}
static uint64_t record_id(uint64_t i) { return 2 * i + (i % 8 == 1 ? 8191 : 1); }
static uint64_t next_state(uint64_t state) { return state * UINT64_C(6364136223846793005) + UINT64_C(1442695040888963407); }
static CALLBACK uint64_t small_make(uint64_t identity, uint64_t seed) { (void)seed; return identity; }
static CALLBACK Wide wide_make(uint64_t identity, uint64_t seed) {
    Wide value; value.words[0] = identity;
    for (size_t i = 1; i < RECORD_WORDS; ++i)
        value.words[i] = identity * UINT64_C(11400714819323198485) + seed + i;
    return value;
}
static CALLBACK void small_observe(Log *log, const uint64_t *value) { word(log, *value); }
static CALLBACK void small_consume(Log *log, uint64_t value) { word(log, value); }
static CALLBACK void wide_observe(Log *log, const Wide *value) {
    for (size_t i = 0; i < RECORD_WORDS; ++i) word(log, value->words[i]);
}
static CALLBACK void wide_consume(Log *log, Wide value) {
    for (size_t i = 0; i < RECORD_WORDS; ++i) word(log, value.words[i]);
}
static CALLBACK int due_compare(const Empty *env, const Due *a, const Due *b) {
    (void)env;
    EVENT(comparisons, 1);
    if (a->deadline != b->deadline) return (a->deadline > b->deadline) - (a->deadline < b->deadline);
    if (a->id != b->id) return (a->id > b->id) - (a->id < b->id);
    if (a->handle.index != b->handle.index) return (a->handle.index > b->handle.index) - (a->handle.index < b->handle.index);
    return (a->handle.generation > b->handle.generation) - (a->handle.generation < b->handle.generation);
}
static CALLBACK uint64_t id_hash(const Hash *env, const uint64_t *key) { return env->modulus ? *key % env->modulus : 0; }
static CALLBACK bool id_equal(const Hash *env, const uint64_t *a, const uint64_t *b) { (void)env; return *a == *b; }
static CALLBACK Handle read_handle(const Empty *env, const uint64_t *key, const Handle *value) { (void)env; (void)key; return *value; }
static CALLBACK Due read_due(const Empty *env, const Due *value) { (void)env; return *value; }
static CALLBACK void discard_id(const Empty *env, uint64_t id, Handle handle) { (void)env; (void)id; (void)handle; }
static CALLBACK void discard_due(const Empty *env, Due value) { (void)env; (void)value; }
static Map map_new(uint64_t capacity) {
    Buckets *cells = wf_cost_allocate(16 + capacity * sizeof(Bucket));
    cells->length = cells->capacity = capacity;
    for (uint64_t i = 0; i < capacity; ++i) cells->data[i].tag = 0;
    return (Map){cells, 0};
}
static uint64_t map_probe(uint64_t home, uint64_t step, uint64_t count) {
    uint64_t remaining = count - home;
    return step >= remaining ? step - remaining : home + step;
}
static uint64_t map_find(const Map *map, uint64_t id, const Hash *env) {
    uint64_t hash = id_hash(env, &id), count = map->cells->length;
    for (uint64_t step = 0; step < count; ++step) {
        uint64_t index = map_probe(hash % count, step, count); EVENT(probes, 1);
        const Bucket *bucket = &map->cells->data[index];
        if (bucket->tag == 0) break;
        if (bucket->tag == 2 && id_equal(env, &bucket->key, &id)) return index;
    }
    return count;
}
static MapLookup map_lookup(const Map *map, uint64_t id, const Hash *env) {
    uint64_t at = map_find(map, id, env);
    if (at < map->cells->length) return (MapLookup){0, read_handle(&empty, &map->cells->data[at].key, &map->cells->data[at].value), 0};
    return (MapLookup){1, {0, 0}, 0};
}
static MapPut map_try_put(Map *map, uint64_t id, Handle value, const Hash *env) {
    uint64_t hash = id_hash(env, &id), count = map->cells->length, available = count;
    for (uint64_t step = 0; step < count; ++step) {
        uint64_t at = map_probe(hash % count, step, count); EVENT(probes, 1);
        Bucket *bucket = &map->cells->data[at];
        if (bucket->tag == 2) {
            if (id_equal(env, &bucket->key, &id)) {
                MapPut result = {1, false, {bucket->key, bucket->value}};
                *bucket = (Bucket){2, id, value}; return result;
            }
        } else if (bucket->tag == 0) { if (available == count) available = at; break; }
        else if (available == count) available = at;
    }
    if (available < count && map->length < CEILING) {
        ++map->length; map->cells->data[available] = (Bucket){2, id, value};
        return (MapPut){0, false, {0, {0, 0}}};
    }
    return (MapPut){1, true, {id, value}};
}
static void map_reserve(Map *map, uint64_t capacity, const Hash *env) {
    if (capacity <= map->cells->length) return;
    Map next = map_new(capacity); next.length = map->length;
    Buckets *old = map->cells;
    while (old->length) {
        Bucket bucket = old->data[--old->length];
        if (bucket.tag != 2) continue;
        uint64_t hash = id_hash(env, &bucket.key);
        for (uint64_t step = 0; step < capacity; ++step) {
            uint64_t at = map_probe(hash % capacity, step, capacity); EVENT(probes, 1);
            if (next.cells->data[at].tag != 2) { next.cells->data[at] = bucket; break; }
        }
    }
    map->cells = next.cells; wf_cost_release(old);
}
static MapPut map_put(Map *map, uint64_t id, Handle value, const Hash *env) {
    MapPut tried = map_try_put(map, id, value, env);
    if (map_put_kind(tried) != 2 || map->cells->length == CEILING) return tried;
    uint64_t capacity = map->cells->length ? 2 * map->cells->length : 1;
    if (capacity > CEILING) capacity = CEILING;
    map_reserve(map, capacity, env);
    return map_try_put(map, tried.pair.key, tried.pair.value, env);
}
static bool map_remove(Map *map, uint64_t id, const Hash *env) {
    if (map->length == 0) return false;
    uint64_t at = map_find(map, id, env);
    if (at == map->cells->length) return false;
    map->cells->data[at].tag = 1; --map->length; return true;
}
static void map_free(Map map) {
    while (map.cells->length) {
        Bucket bucket = map.cells->data[--map.cells->length];
        if (bucket.tag == 2) discard_id(&empty, bucket.key, bucket.value);
    }
    wf_cost_release(map.cells);
}

// Each expansion has the source's concrete payload/result layout. HOLE changes
// only heap movement: validation, staged memberships and owner outcomes match.
#define DEFINE_STORE(P, T, MAKE, OBSERVE, CONSUME, HOLE)                      \
typedef struct { uint64_t id; T payload; bool member; uint64_t position; } P##_Record; \
typedef struct { uint64_t occupied; P##_Record value; uint64_t generation, next; } P##_Cell; \
typedef struct { uint64_t length, capacity; P##_Cell data[]; } P##_Cells;    \
typedef struct { P##_Cells *cells; uint64_t free_head; } P##_Slab; \
typedef struct { uint64_t store_id; bool retained; P##_Slab objects; Map by_id; Heap *due; Hash id_hash; } P##_Store; \
typedef struct { uint32_t failed; StoreHandle handle; T payload; } P##_Insert; \
typedef struct { uint32_t failed; P##_Record value; enum Status error; } P##_Delete; \
typedef struct { enum Status status; T payload; } P##_Replacement;         \
static enum Status P##_delete_status(P##_Delete result) { return result.failed ? result.error : DONE; } \
static P##_Record *P##_get(P##_Slab *slab, Handle handle) {                 \
    EVENT(validations, 1);                                                  \
    if (handle.index < slab->cells->length) {                               \
        P##_Cell *cell = &slab->cells->data[handle.index];                    \
        if (cell->generation == handle.generation && cell->occupied) return &cell->value; \
    }                                                                      \
    return NULL;                                                           \
}                                                                          \
static CALLBACK Info P##_read_info(const Empty *env, const P##_Record *value) { \
    (void)env;                                                             \
    return (Info){value->id, value->member, value->position};                \
}                                                                          \
static CALLBACK void P##_set_position(const uint64_t *position, P##_Record *value) { value->position = *position; } \
static CALLBACK void P##_set_member(const bool *member, P##_Record *value) { value->member = *member; } \
static CALLBACK void P##_swap_payload(T *offered, P##_Record *value) {      \
    T previous = value->payload; value->payload = *offered; *offered = previous; EVENT(payload_assignments, 3); \
}                                                                          \
static CALLBACK void P##_report(P##_Slab *slab, const Due *value, uint64_t position) { \
    EVENT(reports, 1); P##_Record *record = P##_get(slab, value->handle);     \
    if (record != NULL) P##_set_position(&position, record);               \
}                                                                          \
static CALLBACK void P##_consume_record(Log *log, P##_Record value) {      \
    word(log, 4); word(log, value.id); CONSUME(log, value.payload);         \
}                                                                          \
static CALLBACK void P##_observe_record(Log *log, const P##_Record *value) { \
    word(log, 5); word(log, value->id); OBSERVE(log, &value->payload);       \
}                                                                          \
static void P##_heap_reserve(P##_Store *store, uint64_t capacity) {         \
    Heap *old = store->due;                                                 \
    if (old->capacity >= capacity) return;                                  \
    Heap *next = wf_cost_allocate(16 + capacity * sizeof(Due));             \
    next->length = old->length; next->capacity = capacity;                  \
    memcpy(next->data, old->data, old->length * sizeof(Due)); EVENT(heap_assignments, old->length); \
    store->due = next; wf_cost_release(old);                                \
}                                                                          \
static void P##_rise(P##_Store *store, uint64_t at, Due pending) {          \
    Heap *heap = store->due;                                                \
    while (at != 0) {                                                      \
        uint64_t parent = (at - 1) / 2;                                    \
        const Due *value = HOLE ? &pending : &heap->data[at];               \
        if (due_compare(&empty, &heap->data[parent], value) <= 0) break;           \
        if (HOLE) {                                                        \
            heap->data[at] = heap->data[parent]; EVENT(heap_assignments, 1); \
            P##_report(&store->objects, &heap->data[at], at);                         \
        } else {                                                           \
            Due temporary = heap->data[parent]; heap->data[parent] = heap->data[at]; heap->data[at] = temporary; \
            EVENT(swaps, 1); EVENT(heap_assignments, 3);                    \
            P##_report(&store->objects, &heap->data[parent], parent); P##_report(&store->objects, &heap->data[at], at); \
        }                                                                  \
        at = parent;                                                       \
    }                                                                      \
    if (HOLE) { heap->data[at] = pending; EVENT(heap_assignments, 1); P##_report(&store->objects, &heap->data[at], at); } \
}                                                                          \
static void P##_sink(P##_Store *store, uint64_t at, Due pending) {          \
    Heap *heap = store->due; uint64_t count = heap->length;                 \
    while (at < count / 2) {                                               \
        uint64_t child = at * 2 + 1, right = child + 1;                     \
        if (right < count && due_compare(&empty, &heap->data[right], &heap->data[child]) < 0) child = right; \
        const Due *value = HOLE ? &pending : &heap->data[at];               \
        if (due_compare(&empty, value, &heap->data[child]) <= 0) break;       \
        if (HOLE) {                                                        \
            heap->data[at] = heap->data[child]; EVENT(heap_assignments, 1);  \
            P##_report(&store->objects, &heap->data[at], at);                         \
        } else {                                                           \
            Due temporary = heap->data[at]; heap->data[at] = heap->data[child]; heap->data[child] = temporary; \
            EVENT(swaps, 1); EVENT(heap_assignments, 3);                    \
            P##_report(&store->objects, &heap->data[at], at); P##_report(&store->objects, &heap->data[child], child); \
        }                                                                  \
        at = child;                                                        \
    }                                                                      \
    if (HOLE) { heap->data[at] = pending; EVENT(heap_assignments, 1); P##_report(&store->objects, &heap->data[at], at); } \
}                                                                          \
static void P##_repair(P##_Store *store, uint64_t at, Due pending) {        \
    if (at != 0) {                                                         \
        uint64_t parent = (at - 1) / 2;                                    \
        const Due *value = HOLE ? &pending : &store->due->data[at];         \
        if (due_compare(&empty, &store->due->data[parent], value) > 0) { P##_rise(store, at, pending); return; } \
    }                                                                      \
    P##_sink(store, at, pending);                                           \
}                                                                          \
static bool P##_push_due(P##_Store *store, Due value) {                    \
    uint64_t entering = store->due->length;                                \
    if (entering == CEILING) return false;                                 \
    if (entering == store->due->capacity) {                                \
        uint64_t capacity = entering ? entering * 2 : 1;                   \
        if (capacity > CEILING) capacity = CEILING;                        \
        P##_heap_reserve(store, capacity);                                \
    }                                                                      \
    ++store->due->length;                                                  \
    if (!HOLE) { store->due->data[entering] = value; EVENT(heap_assignments, 1); P##_report(&store->objects, &store->due->data[entering], entering); } \
    P##_rise(store, entering, value); return true;                         \
}                                                                          \
static Due P##_remove_due(P##_Store *store, uint64_t at) {                 \
    Due removed = store->due->data[--store->due->length]; EVENT(heap_assignments, 1); \
    if (at < store->due->length) {                                         \
        Due previous = store->due->data[at];                              \
        if (HOLE) EVENT(heap_assignments, 2);                              \
        if (!HOLE) { store->due->data[at] = removed; EVENT(heap_assignments, 3); P##_report(&store->objects, &store->due->data[at], at); } \
        P##_repair(store, at, removed); removed = previous;                \
    }                                                                      \
    return removed;                                                        \
}                                                                          \
static Due P##_replace_due(P##_Store *store, uint64_t at, Due value) {     \
    Due previous = store->due->data[at];                                   \
    if (HOLE) EVENT(heap_assignments, 1);                                   \
    if (!HOLE) { store->due->data[at] = value; EVENT(heap_assignments, 3); P##_report(&store->objects, &store->due->data[at], at); } \
    P##_repair(store, at, value); return previous;                         \
}                                                                          \
static PUBLIC P##_Store P##_new(uint64_t store_id, bool retained, uint64_t capacity, uint64_t index_capacity, uint64_t heap_capacity, uint64_t modulus) { \
    P##_Cells *objects = wf_cost_allocate(16 + capacity * sizeof(P##_Cell)); \
    objects->length = 0; objects->capacity = capacity;                      \
    Map by_id = map_new(index_capacity);                                    \
    Heap *due = wf_cost_allocate(16 + heap_capacity * sizeof(Due));         \
    due->length = 0; due->capacity = heap_capacity;                          \
    return (P##_Store){store_id, retained, {objects, capacity}, by_id, due, {modulus}}; \
}                                                                          \
static PUBLIC P##_Insert P##_insert(P##_Store *store, uint64_t id, T payload) { \
    P##_Record value = {id, payload, false, CEILING}; EVENT(payload_assignments, 1); \
    uint64_t at = store->objects.free_head; P##_Insert result;                      \
    if (at < store->objects.cells->length) {                                     \
        P##_Cell *cell = &store->objects.cells->data[at];                         \
        if (!cell->occupied) {                                             \
            cell->value = value; cell->occupied = 1; store->objects.free_head = cell->next; EVENT(payload_assignments, 1); \
            result.failed = false; result.handle = (StoreHandle){store->store_id, {at, cell->generation}}; return result; \
        }                                                                  \
    } else if (store->objects.cells->length < store->objects.cells->capacity) {        \
        at = store->objects.cells->length++;                                     \
        store->objects.cells->data[at] = (P##_Cell){1, value, 0, store->objects.free_head}; EVENT(payload_assignments, 1); \
        result.failed = false; result.handle = (StoreHandle){store->store_id, {at, 0}}; return result; \
    }                                                                      \
    result.failed = true; result.payload = payload; EVENT(payload_assignments, 1); return result; \
}                                                                          \
static PUBLIC InfoResult P##_info(P##_Store *store, StoreHandle handle) {  \
    EVENT(store_checks, 1);                                                \
    if (handle.store_id != store->store_id) return info_error(WRONG_STORE);  \
    P##_Record *value = P##_get(&store->objects, handle.object);                      \
    if (value == NULL) return info_error(EXPIRED);                         \
    return info_ok(P##_read_info(&empty, value));                           \
}                                                                          \
static PUBLIC VisitResult P##_visit(P##_Store *store, StoreHandle handle, Log *log) { \
    EVENT(store_checks, 1);                                                \
    if (handle.store_id != store->store_id) return (VisitResult){1, 0, WRONG_STORE}; \
    P##_Record *value = P##_get(&store->objects, handle.object);                      \
    if (value == NULL) return (VisitResult){1, 0, EXPIRED};                 \
    P##_observe_record(log, value); return (VisitResult){0, 0, DONE};       \
}                                                                          \
static PUBLIC Lookup P##_lookup(P##_Store *store, uint64_t id) {          \
    MapLookup found = map_lookup(&store->by_id, id, &store->id_hash);        \
    if (found.failed) return lookup_error(MISSING);                        \
    StoreHandle handle = {store->store_id, found.value};                   \
    InfoResult checked = P##_info(store, handle);                          \
    if (info_status(checked) != DONE) return lookup_error(info_status(checked)); \
    if (checked.value.id != id) return lookup_error(CONFLICT);             \
    return lookup_ok(handle);                                             \
}                                                                          \
static PUBLIC enum Status P##_attach(P##_Store *store, StoreHandle handle) { \
    InfoResult checked = P##_info(store, handle);                          \
    if (info_status(checked) != DONE) return info_status(checked);                     \
    if (checked.value.member) return ALREADY;                             \
    MapLookup old = map_lookup(&store->by_id, checked.value.id, &store->id_hash); \
    if (!old.failed) {                                                     \
        if (handle_equal(old.value, handle.object)) return ALREADY;        \
        if (P##_get(&store->objects, old.value) != NULL || store->retained) return CONFLICT; \
    }                                                                      \
    MapPut inserted = map_put(&store->by_id, checked.value.id, handle.object, &store->id_hash); \
    if (map_put_kind(inserted) == 2) return FULL;                            \
    P##_Record *value = P##_get(&store->objects, handle.object); bool member = true; \
    if (value == NULL) return EXPIRED;                                     \
    P##_set_member(&member, value); return DONE;                           \
}                                                                          \
static PUBLIC enum Status P##_detach(P##_Store *store, uint64_t id, StoreHandle handle) { \
    EVENT(store_checks, 1);                                                \
    if (handle.store_id != store->store_id) return WRONG_STORE;             \
    MapLookup found = map_lookup(&store->by_id, id, &store->id_hash);        \
    if (found.failed) return MISSING;                                     \
    if (!handle_equal(found.value, handle.object)) return CONFLICT;        \
    if (!map_remove(&store->by_id, id, &store->id_hash)) return MISSING;    \
    P##_Record *value = P##_get(&store->objects, handle.object); bool member = false; \
    if (value != NULL) P##_set_member(&member, value);                     \
    return DONE;                                                          \
}                                                                          \
static PUBLIC Position P##_position(P##_Store *store, StoreHandle handle) { \
    InfoResult checked = P##_info(store, handle);                          \
    if (info_status(checked) != DONE) return position_error(info_status(checked)); \
    if (checked.value.position == CEILING) return position_error(UNSCHEDULED); \
    if (checked.value.position < store->due->length) {                     \
        Due entry = read_due(&empty, &store->due->data[checked.value.position]);   \
        if (handle_equal(entry.handle, handle.object) && entry.id == checked.value.id) return position_ok(checked.value.position); \
    }                                                                      \
    return position_error(INVALID_POSITION);                               \
}                                                                          \
static PUBLIC enum Status P##_schedule(P##_Store *store, StoreHandle handle, uint64_t deadline) { \
    InfoResult checked = P##_info(store, handle);                          \
    if (info_status(checked) != DONE) return info_status(checked);                     \
    if (checked.value.position != CEILING) {                              \
        Position position = P##_position(store, handle);                  \
        return position_status(position) == DONE ? ALREADY : position_status(position);       \
    }                                                                      \
    return P##_push_due(store, (Due){deadline, checked.value.id, handle.object}) ? DONE : FULL; \
}                                                                          \
static PUBLIC enum Status P##_reschedule(P##_Store *store, StoreHandle handle, uint64_t deadline) { \
    Position position = P##_position(store, handle);                      \
    if (position_status(position) != DONE) return position_status(position);                  \
    Due entry = read_due(&empty, &store->due->data[position.position]);            \
    (void)P##_replace_due(store, position.position, (Due){deadline, entry.id, entry.handle}); return DONE; \
}                                                                          \
static PUBLIC enum Status P##_cancel(P##_Store *store, StoreHandle handle) { \
    Position position = P##_position(store, handle);                      \
    if (position_status(position) != DONE) return position_status(position);                  \
    (void)P##_remove_due(store, position.position);                        \
    P##_Record *value = P##_get(&store->objects, handle.object); uint64_t absent = CEILING; \
    if (value == NULL) return EXPIRED;                                     \
    P##_set_position(&absent, value); return DONE;                         \
}                                                                          \
static PUBLIC P##_Replacement P##_replace(P##_Store *store, uint64_t id, T payload) { \
    Lookup found = P##_lookup(store, id);                                  \
    if (lookup_status(found) != DONE) return (P##_Replacement){lookup_status(found), payload}; \
    P##_Record *value = P##_get(&store->objects, found.handle.object);               \
    if (value == NULL) return (P##_Replacement){EXPIRED, payload};         \
    P##_swap_payload(&payload, value); return (P##_Replacement){DONE, payload}; \
}                                                                          \
static PUBLIC P##_Delete P##_delete(P##_Store *store, StoreHandle handle) { \
    InfoResult checked = P##_info(store, handle); P##_Delete result;      \
    result.failed = 1; result.error = info_status(checked);                 \
    if (info_status(checked) != DONE) return result;                        \
    if (store->retained && (checked.value.member || checked.value.position != CEILING)) { result.error = BUSY; return result; } \
    P##_Record *value = P##_get(&store->objects, handle.object);                     \
    if (value == NULL) { result.error = EXPIRED; return result; }           \
    result.failed = 0; result.value = *value; EVENT(payload_assignments, 1); \
    P##_Cell *cell = &store->objects.cells->data[handle.object.index];            \
    cell->occupied = 0;                                                    \
    if (cell->generation < UINT64_MAX) { ++cell->generation; cell->next = store->objects.free_head; store->objects.free_head = handle.object.index; } \
    return result;                                                        \
}                                                                          \
static PUBLIC uint64_t P##_expire(P##_Store *store, uint64_t now, Log *log) { \
    uint64_t consumed = 0;                                                 \
    while (store->due->length) {                                          \
        Due first = read_due(&empty, &store->due->data[0]);                        \
        if (first.deadline > now) break;                                  \
        Due removed = P##_remove_due(store, 0);                            \
        P##_Record *value = P##_get(&store->objects, removed.handle); uint64_t absent = CEILING; \
        if (value != NULL) P##_set_position(&absent, value);               \
        StoreHandle handle = {store->store_id, removed.handle};            \
        (void)P##_detach(store, removed.id, handle);                       \
        P##_Delete deleted = P##_delete(store, handle);                    \
        if (P##_delete_status(deleted) == DONE) { P##_consume_record(log, deleted.value); ++consumed; } \
    }                                                                      \
    return consumed;                                                      \
}                                                                          \
static PUBLIC void P##_free(P##_Store store, Log *log) {                   \
    map_free(store.by_id);                                                 \
    while (store.due->length) discard_due(&empty, store.due->data[--store.due->length]); \
    wf_cost_release(store.due);                                            \
    while (store.objects.cells->length) {                                       \
        P##_Cell cell = store.objects.cells->data[--store.objects.cells->length];       \
        if (cell.occupied) P##_consume_record(log, cell.value);             \
    }                                                                      \
    wf_cost_release(store.objects.cells);                                       \
}                                                                          \
static void P##_trace_insert(P##_Store *store, Log *log, uint64_t id, uint64_t identity, uint64_t seed, uint64_t deadline) { \
    offer(log, identity); P##_Insert made = P##_insert(store, id, MAKE(identity, seed)); \
    outcome(log, 1, id, made.failed ? FULL : DONE);                        \
    if (made.failed) { word(log, 4); word(log, id); CONSUME(log, made.payload); return; } \
    log_handle(log, made.handle);                                          \
    outcome(log, 2, id, P##_attach(store, made.handle));                    \
    outcome(log, 4, id, P##_schedule(store, made.handle, deadline));        \
}                                                                          \
static Lookup P##_trace_lookup(P##_Store *store, Log *log, uint64_t id) {  \
    Lookup found = P##_lookup(store, id); outcome(log, 3, id, lookup_status(found)); \
    if (lookup_status(found) == DONE) log_handle(log, found.handle);               \
    return found;                                                         \
}                                                                          \
static void P##_trace_delete(P##_Store *store, Log *log, uint64_t id, StoreHandle handle) { \
    P##_Delete deleted = P##_delete(store, handle); outcome(log, 9, id, P##_delete_status(deleted)); \
    if (P##_delete_status(deleted) == DONE) P##_consume_record(log, deleted.value); \
}                                                                          \
static NOINLINE uint64_t P##_trace(uint64_t count, uint64_t seed, uint64_t retained, unsigned path, Log *log) { \
    uint64_t index_capacity = path == 0 ? count * 2 : 0;                   \
    P##_Store store = P##_new(17, retained != 0, count, index_capacity, index_capacity, CEILING); \
    uint64_t state = seed;                                                 \
    for (uint64_t i = 0; i < count; ++i) {                                 \
        state = next_state(state);                                        \
        P##_trace_insert(&store, log, record_id(i), i + 1, seed, 4 * count + ((state >> 32) % (4 * count))); \
    }                                                                      \
    P##_trace_insert(&store, log, UINT64_MAX, count + 1, seed, 0);          \
    if (path == 0) {                                                       \
        for (uint64_t i = 0; i < count; ++i) {                             \
            uint64_t id = record_id(i); Lookup found = P##_trace_lookup(&store, log, id); \
            if (lookup_status(found) != DONE) continue;                           \
            VisitResult visited = P##_visit(&store, found.handle, log); outcome(log, 12, id, visited.failed ? visited.error : DONE); \
            offer(log, count + 2 + i);                                     \
            P##_Replacement old = P##_replace(&store, id, MAKE(count + 2 + i, seed)); \
            outcome(log, 5, id, old.status); word(log, 4); word(log, id); CONSUME(log, old.payload); \
            outcome(log, 6, id, P##_reschedule(&store, found.handle, i % 2 ? 16 * count + i : i)); \
        }                                                                  \
        for (uint64_t i = 0; i < count; i += 4) {                          \
            uint64_t id = record_id(i); Lookup found = P##_trace_lookup(&store, log, id); \
            if (lookup_status(found) != DONE) continue;                           \
            outcome(log, 7, id, P##_cancel(&store, found.handle));         \
            outcome(log, 4, id, P##_schedule(&store, found.handle, 8 * count + i)); \
        }                                                                  \
        for (uint64_t i = 0; i < count; i += 8) {                          \
            uint64_t id = record_id(i); Lookup found = P##_trace_lookup(&store, log, id); \
            if (lookup_status(found) != DONE) continue;                           \
            P##_trace_delete(&store, log, id, found.handle);               \
            if (retained) {                                               \
                if (i % 16 == 0) {                                       \
                    outcome(log, 8, id, P##_detach(&store, id, found.handle)); \
                    P##_trace_delete(&store, log, id, found.handle);       \
                    outcome(log, 7, id, P##_cancel(&store, found.handle)); \
                } else {                                                   \
                    outcome(log, 7, id, P##_cancel(&store, found.handle)); \
                    P##_trace_delete(&store, log, id, found.handle);       \
                    outcome(log, 8, id, P##_detach(&store, id, found.handle)); \
                }                                                          \
                P##_trace_delete(&store, log, id, found.handle);           \
            } else {                                                       \
                (void)P##_trace_lookup(&store, log, id);                    \
                outcome(log, 7, id, P##_cancel(&store, found.handle));     \
            }                                                              \
            P##_trace_insert(&store, log, id, 2 * count + 2 + i, seed, 8 * count + i); \
        }                                                                  \
        uint64_t expired = P##_expire(&store, 4 * count - 1, log);         \
        word(log, 6); word(log, expired);                                  \
        for (uint64_t i = 2; i < count; i += 4) {                          \
            uint64_t id = record_id(i); (void)P##_trace_lookup(&store, log, id); \
            P##_trace_insert(&store, log, id, 3 * count + 2 + i, seed, 8 * count + i); \
        }                                                                  \
        uint64_t retired = P##_expire(&store, 9 * count - 1, log);         \
        word(log, 6); word(log, retired);                                  \
    } else {                                                               \
        uint64_t expired = P##_expire(&store, UINT64_MAX, log);            \
        word(log, 6); word(log, expired);                                  \
    }                                                                      \
    P##_free(store, log); return log->digest;                              \
}

DEFINE_STORE(small_swap, uint64_t, small_make, small_observe, small_consume, 0)
DEFINE_STORE(small_hole, uint64_t, small_make, small_observe, small_consume, 1)
DEFINE_STORE(wide_swap, Wide, wide_make, wide_observe, wide_consume, 0)
DEFINE_STORE(wide_hole, Wide, wide_make, wide_observe, wide_consume, 1)
_Static_assert(sizeof(Due) == 32 && sizeof(Bucket) == 32, "fixed index strides");
_Static_assert(sizeof(small_swap_Record) == 32 && sizeof(wide_swap_Record) == 280, "record strides");
_Static_assert(sizeof(small_swap_Cell) == 56 && sizeof(wide_swap_Cell) == 304, "Slab cell strides");
_Static_assert(sizeof(MapPut) == 32 && sizeof(MapLookup) == 32, "tagged map results");
_Static_assert(sizeof(Lookup) == 40 && sizeof(InfoResult) == 40, "tagged composite lookup results");
_Static_assert(sizeof(Position) == 24 && sizeof(VisitResult) == 12, "scalar and unit tagged results");
_Static_assert(sizeof(small_swap_Insert) == 40 && sizeof(wide_swap_Insert) == 288, "offered-owner result extents");
_Static_assert(sizeof(small_swap_Delete) == 48 && sizeof(wide_swap_Delete) == 296, "returned-record result extents");
#if defined(WITH_WF)
extern uint64_t wf_indexed_cost_small_trace(uint64_t, uint64_t, uint64_t, uint64_t, Log *);
extern uint64_t wf_indexed_cost_wide_trace(uint64_t, uint64_t, uint64_t, uint64_t, Log *);
#endif
static uint64_t run(unsigned variant, bool wide, uint64_t count, uint64_t seed,
                    unsigned policy, unsigned path, Log *log) {
#if defined(WITH_WF)
    if (variant == 0) return wide ? wf_indexed_cost_wide_trace(count, seed, policy, path, log)
                                 : wf_indexed_cost_small_trace(count, seed, policy, path, log);
#endif
    require(variant == 1 || variant == 2, "available implementation");
    if (wide) return variant == 1 ? wide_swap_trace(count, seed, policy != 0, path, log)
                                 : wide_hole_trace(count, seed, policy != 0, path, log);
    return variant == 1 ? small_swap_trace(count, seed, policy != 0, path, log)
                        : small_hole_trace(count, seed, policy != 0, path, log);
}

// This model has no reverse positions and no heap. It keeps an ordered vector
// of due entries and a sorted ID association table, independently of hashing.
typedef struct {
    uint64_t id, owner, generation;
    bool live, member, scheduled;
} ModelRecord;
typedef struct { uint64_t id; bool present; Handle handle; } ModelId;
typedef struct {
    ModelRecord records[CEILING];
    ModelId ids[CEILING];
    Due sorted[CEILING];
    uint64_t vacant[CEILING], vacancies, materialized, capacity, due_count;
    bool retained, wide;
    uint64_t seed;
} Model;
static int model_id_order(const void *a, const void *b) {
    uint64_t left = ((const ModelId *)a)->id, right = ((const ModelId *)b)->id;
    return (left > right) - (left < right);
}
static int model_due_order(Due a, Due b) {
    const uint64_t left[] = {a.deadline, a.id, a.handle.index, a.handle.generation};
    const uint64_t right[] = {b.deadline, b.id, b.handle.index, b.handle.generation};
    for (size_t i = 0; i < 4; ++i)
        if (left[i] != right[i]) return left[i] < right[i] ? -1 : 1;
    return 0;
}
static ModelId *model_id(Model *model, uint64_t id) {
    uint64_t first = 0, last = model->capacity;
    while (first < last) {
        uint64_t middle = first + (last - first) / 2;
        if (model->ids[middle].id < id) first = middle + 1; else last = middle;
    }
    require(first < model->capacity && model->ids[first].id == id, "oracle ID domain");
    return &model->ids[first];
}
static ModelRecord *model_record(Model *model, Handle handle) {
    if (handle.index >= model->materialized) return NULL;
    ModelRecord *record = &model->records[handle.index];
    return record->live && record->generation == handle.generation ? record : NULL;
}
static void model_payload(Log *log, Model *model, uint64_t marker, uint64_t id, uint64_t owner) {
    word(log, marker); word(log, id); word(log, owner);
    if (model->wide)
        for (uint64_t i = 1; i < RECORD_WORDS; ++i)
            word(log, owner * UINT64_C(11400714819323198485) + model->seed + i);
}
static void model_place(Model *model, Due entry) {
    require(model->due_count < CEILING, "oracle heap capacity");
    uint64_t first = 0, last = model->due_count;
    while (first < last) {
        uint64_t middle = first + (last - first) / 2;
        if (model_due_order(model->sorted[middle], entry) < 0) first = middle + 1; else last = middle;
    }
    memmove(model->sorted + first + 1, model->sorted + first,
            (size_t)(model->due_count - first) * sizeof(Due));
    model->sorted[first] = entry; ++model->due_count;
    ModelRecord *record = model_record(model, entry.handle);
    require(record != NULL && !record->scheduled, "oracle single live membership");
    record->scheduled = true;
}
static Due model_unlink(Model *model, Handle handle) {
    uint64_t at = 0;
    while (at < model->due_count && !handle_equal(model->sorted[at].handle, handle)) ++at;
    require(at < model->due_count, "oracle indexed membership");
    Due removed = model->sorted[at]; --model->due_count;
    memmove(model->sorted + at, model->sorted + at + 1,
            (size_t)(model->due_count - at) * sizeof(Due));
    ModelRecord *record = model_record(model, handle);
    if (record != NULL) record->scheduled = false;
    return removed;
}
static Lookup model_lookup(Model *model, Log *log, uint64_t id) {
    ModelId *association = model_id(model, id);
    enum Status status = MISSING;
    StoreHandle handle = {17, association->handle};
    if (association->present) status = model_record(model, association->handle) == NULL ? EXPIRED : DONE;
    outcome(log, 3, id, status);
    if (status == DONE) log_handle(log, handle);
    return status == DONE ? lookup_ok(handle) : lookup_error(status);
}
static void model_insert(Model *model, Log *log, uint64_t id, uint64_t owner, uint64_t deadline) {
    offer(log, owner);
    if (model->vacancies == 0 && model->materialized == model->capacity) {
        outcome(log, 1, id, FULL); model_payload(log, model, 4, id, owner); return;
    }
    uint64_t slot = model->vacancies ? model->vacant[--model->vacancies] : model->materialized++;
    ModelRecord *record = &model->records[slot];
    record->id = id; record->owner = owner; record->live = true;
    record->member = record->scheduled = false;
    Handle handle = {slot, record->generation};
    outcome(log, 1, id, DONE); log_handle(log, (StoreHandle){17, handle});
    ModelId *association = model_id(model, id);
    require(!association->present || model_record(model, association->handle) == NULL, "oracle attach domain");
    association->present = true; association->handle = handle; record->member = true;
    outcome(log, 2, id, DONE);
    model_place(model, (Due){deadline, id, handle}); outcome(log, 4, id, DONE);
}
static void model_cancel(Model *model, Log *log, uint64_t id, Handle handle) {
    ModelRecord *record = model_record(model, handle);
    enum Status status = record == NULL ? EXPIRED : record->scheduled ? DONE : UNSCHEDULED;
    if (status == DONE) (void)model_unlink(model, handle);
    outcome(log, 7, id, status);
}
static void model_detach(Model *model, Log *log, uint64_t id, Handle handle) {
    ModelId *association = model_id(model, id);
    enum Status status = !association->present ? MISSING : handle_equal(association->handle, handle) ? DONE : CONFLICT;
    if (status == DONE) {
        association->present = false;
        ModelRecord *record = model_record(model, handle);
        if (record != NULL) record->member = false;
    }
    if (log != NULL) outcome(log, 8, id, status);
}
static bool model_delete(Model *model, Log *log, uint64_t id, Handle handle, bool report_status) {
    ModelRecord *record = model_record(model, handle);
    enum Status status = record == NULL ? EXPIRED : model->retained && (record->member || record->scheduled) ? BUSY : DONE;
    if (report_status) outcome(log, 9, id, status);
    if (status != DONE) return false;
    model_payload(log, model, 4, record->id, record->owner); record->live = false;
    if (record->generation < UINT64_MAX) {
        ++record->generation; model->vacant[model->vacancies++] = handle.index;
    }
    return true;
}
static void model_expire(Model *model, Log *log, uint64_t now) {
    uint64_t consumed = 0;
    while (model->due_count && model->sorted[0].deadline <= now) {
        Due entry = model_unlink(model, model->sorted[0].handle);
        model_detach(model, NULL, entry.id, entry.handle);
        if (model_delete(model, log, entry.id, entry.handle, false)) ++consumed;
    }
    word(log, 6); word(log, consumed);
}
static void oracle(bool wide, uint64_t count, uint64_t seed, unsigned policy, unsigned path, Log *log) {
    Model *model = calloc(1, sizeof *model); require(model != NULL, "oracle storage");
    model->capacity = count; model->wide = wide; model->retained = policy != 0; model->seed = seed;
    for (uint64_t i = 0; i < count; ++i) model->ids[i].id = record_id(i);
    qsort(model->ids, (size_t)count, sizeof model->ids[0], model_id_order);
    uint64_t state = seed;
    for (uint64_t i = 0; i < count; ++i) {
        state = next_state(state);
        model_insert(model, log, record_id(i), i + 1, 4 * count + ((state >> 32) % (4 * count)));
    }
    model_insert(model, log, UINT64_MAX, count + 1, 0);
    if (path == 0) {
        for (uint64_t i = 0; i < count; ++i) {
            uint64_t id = record_id(i); Lookup found = model_lookup(model, log, id);
            require(lookup_status(found) == DONE, "oracle lookup");
            ModelRecord *record = model_record(model, found.handle.object);
            model_payload(log, model, 5, id, record->owner); outcome(log, 12, id, DONE);
            offer(log, count + 2 + i); outcome(log, 5, id, DONE);
            model_payload(log, model, 4, id, record->owner); record->owner = count + 2 + i;
            (void)model_unlink(model, found.handle.object);
            model_place(model, (Due){i % 2 ? 16 * count + i : i, id, found.handle.object});
            outcome(log, 6, id, DONE);
        }
        for (uint64_t i = 0; i < count; i += 4) {
            uint64_t id = record_id(i); Lookup found = model_lookup(model, log, id);
            model_cancel(model, log, id, found.handle.object);
            model_place(model, (Due){8 * count + i, id, found.handle.object}); outcome(log, 4, id, DONE);
        }
        for (uint64_t i = 0; i < count; i += 8) {
            uint64_t id = record_id(i); Lookup found = model_lookup(model, log, id);
            (void)model_delete(model, log, id, found.handle.object, true);
            if (policy) {
                if (i % 16 == 0) {
                    model_detach(model, log, id, found.handle.object);
                    (void)model_delete(model, log, id, found.handle.object, true);
                    model_cancel(model, log, id, found.handle.object);
                } else {
                    model_cancel(model, log, id, found.handle.object);
                    (void)model_delete(model, log, id, found.handle.object, true);
                    model_detach(model, log, id, found.handle.object);
                }
                (void)model_delete(model, log, id, found.handle.object, true);
            } else {
                (void)model_lookup(model, log, id);
                model_cancel(model, log, id, found.handle.object);
            }
            model_insert(model, log, id, 2 * count + 2 + i, 8 * count + i);
        }
        model_expire(model, log, 4 * count - 1);
        for (uint64_t i = 2; i < count; i += 4) {
            uint64_t id = record_id(i); (void)model_lookup(model, log, id);
            model_insert(model, log, id, 3 * count + 2 + i, 8 * count + i);
        }
        model_expire(model, log, 9 * count - 1);
    } else model_expire(model, log, UINT64_MAX);
    for (uint64_t i = model->materialized; i != 0; --i) {
        ModelRecord *record = &model->records[i - 1];
        if (record->live) model_payload(log, model, 4, record->id, record->owner);
    }
    free(model);
}
static Words *new_log(uint64_t capacity) {
    Words *values = malloc(sizeof *values + (size_t)capacity * sizeof(uint64_t));
    require(values != NULL, "outcome log allocation");
    values->length = 0; values->capacity = capacity; return values;
}
static Log log_start(Words *values, uint64_t seed) {
    values->length = 0; return (Log){values, seed, 0};
}
static void check_owners(const Log *log, bool wide, uint64_t count) {
    unsigned char *offered = calloc((size_t)4 * count + 2, 1);
    unsigned char *consumed = calloc((size_t)4 * count + 2, 1);
    require(offered != NULL && consumed != NULL, "owner ledger allocation");
    uint64_t at = 0, words = wide ? RECORD_WORDS : 1;
    while (at < log->values->length) {
        uint64_t marker = log->values->data[at++];
        if (marker == 1 || marker == 2) { require(at + 3 <= log->values->length, "fixed event extent"); at += 3; }
        else if (marker == 3) {
            require(at < log->values->length, "offer event extent");
            uint64_t owner = log->values->data[at++];
            require(owner > 0 && owner < 4 * count + 2, "offered owner domain");
            require(++offered[owner] == 1, "each owner offered exactly once");
        } else if (marker == 4 || marker == 5) {
            require(at + 1 + words <= log->values->length, "payload event extent");
            ++at; uint64_t owner = log->values->data[at];
            require(owner > 0 && owner < 4 * count + 2, "returned owner domain");
            require(offered[owner] == 1, "returned or observed owner was offered");
            if (marker == 4) require(++consumed[owner] == 1, "each owner consumed exactly once");
            else require(consumed[owner] == 0, "only live owner can be observed");
            at += words;
        } else if (marker == 6) { require(at < log->values->length, "expiry event extent"); ++at; }
        else require(false, "known outcome event");
    }
    for (uint64_t owner = 1; owner < 4 * count + 2; ++owner)
        require(offered[owner] == consumed[owner], "all offered owners consumed once");
    free(consumed); free(offered);
}
static void equal_logs(const Log *actual, const Log *expected, unsigned variant,
                       bool wide, uint64_t count, uint64_t seed, unsigned policy, unsigned path) {
    require(actual->used == actual->values->length && expected->used == expected->values->length,
            "complete transcript fits its backing");
    if (actual->used != expected->used || actual->digest != expected->digest ||
        memcmp(actual->values->data, expected->values->data, (size_t)expected->used * sizeof(uint64_t)) != 0) {
        uint64_t at = 0, bound = actual->used < expected->used ? actual->used : expected->used;
        while (at < bound && actual->values->data[at] == expected->values->data[at]) ++at;
        fprintf(stderr, "transcript mismatch: %s %s %s bytes=%u n=%" PRIu64 " seed=%" PRIu64 " word=%" PRIu64 " actual=%" PRIu64 " expected=%" PRIu64 " lengths=%" PRIu64 "/%" PRIu64 "\n",
                variants[variant], policies[policy], paths[path], wide ? 256 : 8, count, seed, at,
                at < actual->used ? actual->values->data[at] : UINT64_MAX,
                at < expected->used ? expected->values->data[at] : UINT64_MAX, actual->used, expected->used);
        exit(1);
    }
    check_owners(actual, wide, count);
}
static uint64_t repetitions(bool wide, uint64_t count) {
    uint64_t repeats = (wide ? 512 : 2048) / count; return repeats ? repeats : 1;
}
static void check_extent_formula(bool wide, uint64_t count, unsigned path, uint64_t traces) {
    uint64_t stride = wide ? 304 : 56;
    uint64_t expected_requests = 3;
    uint64_t expected_bytes = 48 + count * (stride + 128);
    uint64_t expected_peak = expected_bytes;
    if (path != 0) {
        expected_bytes = 48 + count * stride;
        for (uint64_t capacity = 1; capacity <= count; capacity *= 2) {
            expected_requests += 2;
            expected_bytes += 2 * (16 + capacity * 32);
        }
        expected_peak = 64 + count * (stride + 80);
    }
    require(requests == expected_requests * traces && requested_bytes == expected_bytes * traces && peak_bytes == expected_peak,
            "independent backing count/extent and growth-overlap formula");
}
static void check(void) {
    const uint64_t counts[] = {16, 256, 4096};
    size_t configurations = 0, executions = 0;
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned policy = 0; policy < 2; ++policy)
            for (unsigned path = 0; path < 2; ++path)
                for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n) {
                    uint64_t count = counts[n], seeds = 6 + repetitions(wide != 0, count);
                    Words *expected_words = new_log(count * 256 + 64), *actual_words = new_log(count * 256 + 64);
                    for (uint64_t s = 0; s < seeds; ++s) {
                        uint64_t seed = 101 + s;
                        Log expected = log_start(expected_words, seed);
                        oracle(wide != 0, count, seed, policy, path, &expected);
                        check_owners(&expected, wide != 0, count);
                        size_t matched_requests = 0, matched_bytes = 0, matched_peak = 0;
#if defined(WITH_WF)
                        const unsigned first_variant = 0;
#else
                        const unsigned first_variant = 1;
#endif
                        for (unsigned v = first_variant; v < VARIANT_COUNT; ++v) {
                            Log actual = log_start(actual_words, seed); reset_accounting();
                            require(run(v, wide != 0, count, seed, policy, path, &actual) == actual.digest, "trace digest result");
                            check_accounting(); check_extent_formula(wide != 0, count, path, 1);
                            equal_logs(&actual, &expected, v, wide != 0, count, seed, policy, path);
                            if (v == first_variant) { matched_requests = requests; matched_bytes = requested_bytes; matched_peak = peak_bytes; }
                            else require(requests == matched_requests && requested_bytes == matched_bytes && peak_bytes == matched_peak,
                                         "matched complete backing requests and growth overlap");
                            ++executions;
                        }
                        ++configurations;
                    }
                    free(actual_words); free(expected_words);
                }
    printf("indexed costs: %zu unique complete matrix inputs, %zu full transcript/owner-ledger executions passed\n", configurations, executions);
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
static void quantum(void) {
    uint64_t minimum = UINT64_MAX, maximum = 0, previous = nanos(), transitions = 0;
    for (unsigned i = 0; i < 100000; ++i) {
        uint64_t current = nanos(), step = current - previous;
        if (step) { if (step < minimum) minimum = step; if (step > maximum) maximum = step; ++transitions; }
        previous = current;
    }
    printf("clock_source,min_nonzero_ns,max_step_ns,transitions\nmonotonic,%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n", minimum, maximum, transitions);
}
static void measure(unsigned cohort) {
    const uint64_t counts[] = {16, 256, 4096};
    Words *empty = new_log(0);
    puts("contract,cohort,payload_bytes,policy,path,count,variant,sample,traces,elapsed_ns,checksum,requests,releases,requested_bytes,peak_bytes,transcript_words");
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned policy = 0; policy < 2; ++policy)
            for (unsigned path = 0; path < 2; ++path)
                for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                    for (unsigned sample = 0; sample < 7; ++sample) {
                        uint64_t count = counts[n], seed = 101 + sample, traces = repetitions(wide != 0, count);
                        uint64_t expected = 0, expected_words = 0;
                        for (uint64_t t = 0; t < traces; ++t) {
                            Log log = log_start(empty, seed + t); oracle(wide != 0, count, seed + t, policy, path, &log);
                            expected = expected * UINT64_C(257) + log.digest; expected_words += log.used;
                        }
                        for (unsigned offset = 0; offset < VARIANT_COUNT; ++offset) {
                            unsigned position = (sample + offset) % VARIANT_COUNT;
                            unsigned variant = cohort ? VARIANT_COUNT - 1 - position : position;
                            reset_accounting(); uint64_t checksum = 0, used = 0, before = nanos();
                            for (uint64_t t = 0; t < traces; ++t) {
                                Log log = log_start(empty, seed + t);
                                checksum = checksum * UINT64_C(257) + run(variant, wide != 0, count, seed + t, policy, path, &log);
                                used += log.used;
                            }
                            uint64_t elapsed = nanos() - before; observed = checksum;
                            require(checksum == expected && used == expected_words, "timed transcript digest and extent"); check_accounting();
                            check_extent_formula(wide != 0, count, path, traces);
                            printf("%s,%u,%u,%s,%s,%" PRIu64 ",%s,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu,%zu,%zu,%" PRIu64 "\n",
                                   CONTRACT, cohort, wide ? 256 : 8, policies[policy], paths[path], count, variants[variant], sample,
                                   traces, elapsed, checksum, requests, releases, requested_bytes, peak_bytes, used);
                        }
                    }
    free(empty);
}
#if defined(AUDIT_EVENTS)
static void events(void) {
    const uint64_t counts[] = {16, 256, 4096}; Words *empty = new_log(0);
    puts("payload_bytes,policy,path,count,variant,comparisons,swaps,heap_assignments,position_reports,slab_validation_calls,store_identity_checks,handle_identity_comparisons,hash_probes,payload_assignments,heap_assigned_bytes,payload_assigned_bytes,requests,releases,requested_bytes,peak_bytes");
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned policy = 0; policy < 2; ++policy)
            for (unsigned path = 0; path < 2; ++path)
                for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                    for (unsigned variant = 1; variant < VARIANT_COUNT; ++variant) {
                        Log expected = log_start(empty, 101); oracle(wide != 0, counts[n], 101, policy, path, &expected);
                        Log actual = log_start(empty, 101); reset_accounting();
                        comparisons = swaps = heap_assignments = reports = validations = probes = payload_assignments = 0;
                        store_checks = handle_equalities = 0;
                        require(run(variant, wide != 0, counts[n], 101, policy, path, &actual) == expected.digest, "counted transcript digest");
                        check_accounting();
                        check_extent_formula(wide != 0, counts[n], path, 1);
                        printf("%u,%s,%s,%" PRIu64 ",%s,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu,%zu,%zu\n",
                               wide ? 256 : 8, policies[policy], paths[path], counts[n], variants[variant], comparisons, swaps, heap_assignments,
                               reports, validations, store_checks, handle_equalities, probes, payload_assignments, heap_assignments * sizeof(Due), payload_assignments * (wide ? 256 : 8),
                               requests, releases, requested_bytes, peak_bytes);
                    }
    free(empty);
}
#endif
int main(int argc, char **argv) {
    require(argc >= 2, "usage: indexed-costs check|measure COHORT|quantum|events");
    if (strcmp(argv[1], "check") == 0) { require(argc == 2, "check arguments"); check(); }
    else if (strcmp(argv[1], "quantum") == 0) { require(argc == 2, "quantum arguments"); quantum(); }
#if defined(AUDIT_EVENTS)
    else if (strcmp(argv[1], "events") == 0) { require(argc == 2, "events arguments"); events(); }
#endif
    else {
        require(argc == 3 && strcmp(argv[1], "measure") == 0 && (strcmp(argv[2], "0") == 0 || strcmp(argv[2], "1") == 0), "measure cohort 0 or 1");
        measure((unsigned)(argv[2][0] - '0'));
    }
    return 0;
}
