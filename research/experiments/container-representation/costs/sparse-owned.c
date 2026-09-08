#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
/* Bounded native sparse-layout control. Not Whitefoot syntax or proof evidence. */
#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#define TIMER_NAME "windows-qpc"
#else
#include <time.h>
#define TIMER_NAME "posix-monotonic"
#endif

#ifdef NDEBUG
#error "This checked research control requires assertions; do not define NDEBUG"
#endif

#define MAX_RESOURCES 16384u
#define MAX_ORACLE 4096u
#define EMPTY 0x80u
#define DELETED 0xfeu
#define MAX_FULL 0x7fu

typedef struct Resource {
    uint64_t id;
    uint64_t data[4];
} Resource;

typedef struct Cell {
    uint64_t key;
    Resource *resource;
} Cell;

typedef struct Ledger {
    bool created[MAX_RESOURCES];
    bool dropped[MAX_RESOURCES];
    Resource *live[MAX_RESOURCES];
    size_t created_count;
    size_t dropped_count;
} Ledger;

static Ledger ledger;
static size_t backing_live_bytes;
static size_t backing_peak_bytes;
static size_t backing_live_allocations;
static size_t backing_peak_allocations;
static size_t control_bytes_initialized;
static volatile uint64_t published_witness;

static void reset_ledger(void) {
    memset(&ledger, 0, sizeof(ledger));
}

static Resource *resource_new(uint64_t id) {
    assert(id < MAX_RESOURCES && !ledger.created[id]);
    Resource *resource = malloc(sizeof(*resource));
    assert(resource != NULL);
    resource->id = id;
    for (size_t i = 0; i < 4; ++i) {
        resource->data[i] = id * 17u + i;
    }
    ledger.created[id] = true;
    ledger.live[id] = resource;
    ++ledger.created_count;
    return resource;
}

static void resource_drop(Resource *resource) {
    assert(resource != NULL);
    const uint64_t id = resource->id;
    assert(id < MAX_RESOURCES && ledger.created[id] && !ledger.dropped[id]);
    assert(ledger.live[id] == resource);
    ledger.dropped[id] = true;
    ledger.live[id] = NULL;
    ++ledger.dropped_count;
    free(resource);
}

static void *backing_allocate(size_t bytes, bool force_failure) {
    if (force_failure) {
        return NULL;
    }
    void *memory = malloc(bytes == 0 ? 1 : bytes);
    if (memory != NULL) {
        backing_live_bytes += bytes;
        ++backing_live_allocations;
        if (backing_live_bytes > backing_peak_bytes) {
            backing_peak_bytes = backing_live_bytes;
        }
        if (backing_live_allocations > backing_peak_allocations) {
            backing_peak_allocations = backing_live_allocations;
        }
    }
    return memory;
}

static void backing_release(void *memory, size_t bytes) {
    if (memory == NULL) {
        return;
    }
    assert(backing_live_bytes >= bytes && backing_live_allocations > 0);
    backing_live_bytes -= bytes;
    --backing_live_allocations;
    free(memory);
}

static bool checked_mul(size_t a, size_t b, size_t *out) {
    if (a != 0 && b > SIZE_MAX / a) {
        return false;
    }
    *out = a * b;
    return true;
}

static bool checked_add(size_t a, size_t b, size_t *out) {
    if (b > SIZE_MAX - a) {
        return false;
    }
    *out = a + b;
    return true;
}

static bool checked_align_up(size_t value, size_t alignment, size_t *out) {
    assert(alignment != 0 && (alignment & (alignment - 1)) == 0);
    const size_t mask = alignment - 1;
    size_t sum;
    if (!checked_add(value, mask, &sum)) {
        return false;
    }
    *out = sum & ~mask;
    return true;
}

static uint8_t fingerprint(uint64_t key) {
    return (uint8_t)(key & MAX_FULL);
}

static size_t bucket(uint64_t key, size_t capacity) {
    assert(capacity > 0);
    return (size_t)(key % capacity);
}

static size_t probe_index(size_t start, size_t probe, size_t capacity) {
    assert(start < capacity && probe < capacity);
    return probe >= capacity - start ? probe - (capacity - start) : start + probe;
}

static bool is_full(uint8_t control) {
    return control <= MAX_FULL;
}

static bool is_vacant(uint8_t control) {
    return control == EMPTY || control == DELETED;
}

typedef struct ISlot {
    uint8_t control;
    Cell cell;
} ISlot;

typedef struct ITable {
    size_t capacity;
    size_t bytes;
    ISlot *slots;
} ITable;

typedef struct STable {
    size_t capacity;
    size_t bytes;
    unsigned char *backing;
    uint8_t *controls;
    Cell *cells;
    size_t payload_offset;
} STable;

static bool i_init(ITable *table, size_t capacity, bool force_failure) {
    size_t bytes;
    if (capacity == 0 || !checked_mul(capacity, sizeof(ISlot), &bytes)) {
        return false;
    }
    ISlot *slots = backing_allocate(bytes, force_failure);
    if (slots == NULL) {
        return false;
    }
    for (size_t i = 0; i < capacity; ++i) {
        slots[i].control = EMPTY;
    }
    assert(control_bytes_initialized <= SIZE_MAX - capacity);
    control_bytes_initialized += capacity;
    *table = (ITable){.capacity = capacity, .bytes = bytes, .slots = slots};
    return true;
}

static void i_release_backing(ITable *table) {
    backing_release(table->slots, table->bytes);
    *table = (ITable){0};
}

static uint8_t *i_ctl(ITable *table, size_t index) {
    assert(index < table->capacity);
    return &table->slots[index].control;
}

static Cell *i_cell(ITable *table, size_t index) {
    assert(index < table->capacity);
    return &table->slots[index].cell;
}

static bool i_same_descriptor(const ITable *left, const ITable *right) {
    return left->capacity == right->capacity && left->bytes == right->bytes &&
           left->slots == right->slots;
}

static bool s_layout(size_t capacity, size_t *offset, size_t *bytes) {
    size_t payload_bytes;
    return capacity > 0 && checked_align_up(capacity, _Alignof(Cell), offset) &&
           checked_mul(capacity, sizeof(Cell), &payload_bytes) &&
           checked_add(*offset, payload_bytes, bytes);
}

static bool s_init(STable *table, size_t capacity, bool force_failure) {
    size_t offset;
    size_t bytes;
    if (!s_layout(capacity, &offset, &bytes)) {
        return false;
    }
    unsigned char *backing = backing_allocate(bytes, force_failure);
    if (backing == NULL) {
        return false;
    }
    uint8_t *controls = backing;
    Cell *cells = (Cell *)(void *)(backing + offset);
    assert(((uintptr_t)cells % _Alignof(Cell)) == 0);
    memset(controls, EMPTY, capacity);
    assert(control_bytes_initialized <= SIZE_MAX - capacity);
    control_bytes_initialized += capacity;
    *table = (STable){
        .capacity = capacity,
        .bytes = bytes,
        .backing = backing,
        .controls = controls,
        .cells = cells,
        .payload_offset = offset,
    };
    return true;
}

static void s_release_backing(STable *table) {
    backing_release(table->backing, table->bytes);
    *table = (STable){0};
}

static uint8_t *s_ctl(STable *table, size_t index) {
    assert(index < table->capacity);
    return &table->controls[index];
}

static Cell *s_cell(STable *table, size_t index) {
    assert(index < table->capacity);
    return &table->cells[index];
}

static bool s_same_descriptor(const STable *left, const STable *right) {
    return left->capacity == right->capacity && left->bytes == right->bytes &&
           left->backing == right->backing && left->controls == right->controls &&
           left->cells == right->cells && left->payload_offset == right->payload_offset;
}

typedef enum PutResult { PUT_INSERTED, PUT_REPLACED, PUT_FULL } PutResult;
typedef enum StepResult { STEP_PAUSED, STEP_DONE, STEP_BLOCKED_FULL } StepResult;
typedef enum Phase { PHASE_SCAN, PHASE_PROBE, PHASE_BLOCKED, PHASE_DONE } Phase;

#define DEFINE_TABLE(P, TABLE)                                                                 \
    typedef struct P##Rehash {                                                                 \
        TABLE source;                                                                          \
        TABLE target;                                                                          \
        Phase phase;                                                                           \
        size_t source_next;                                                                    \
        Cell held;                                                                             \
        bool held_valid;                                                                       \
        size_t probe_start;                                                                    \
        size_t next_probe;                                                                     \
    } P##Rehash;                                                                               \
                                                                                               \
    static bool P##_same_rehash(const P##Rehash *left, const P##Rehash *right) {               \
        if (!P##_same_descriptor(&left->source, &right->source) ||                             \
            !P##_same_descriptor(&left->target, &right->target) ||                             \
            left->phase != right->phase || left->source_next != right->source_next ||          \
            left->held_valid != right->held_valid ||                                           \
            left->probe_start != right->probe_start || left->next_probe != right->next_probe) { \
            return false;                                                                      \
        }                                                                                      \
        return !left->held_valid ||                                                             \
               (left->held.key == right->held.key &&                                           \
                left->held.resource == right->held.resource);                                  \
    }                                                                                          \
                                                                                               \
    static PutResult P##_put(TABLE *table, uint64_t key, Resource *offered,                    \
                             Resource **old, size_t *placed_index) {                            \
        assert(table->capacity > 0 && offered != NULL);                                         \
        *old = NULL;                                                                            \
        size_t first_deleted = SIZE_MAX;                                                        \
        const size_t start = bucket(key, table->capacity);                                      \
        for (size_t probe = 0; probe < table->capacity; ++probe) {                              \
            const size_t index = probe_index(start, probe, table->capacity);                    \
            const uint8_t control = *P##_ctl(table, index);                                     \
            if (control == DELETED) {                                                           \
                if (first_deleted == SIZE_MAX) {                                                \
                    first_deleted = index;                                                      \
                }                                                                              \
                continue;                                                                      \
            }                                                                                  \
            if (control == EMPTY) {                                                             \
                const size_t destination =                                                     \
                    first_deleted == SIZE_MAX ? index : first_deleted;                          \
                Cell *cell = P##_cell(table, destination);                                      \
                cell->key = key;                                                                \
                cell->resource = offered;                                                       \
                *P##_ctl(table, destination) = fingerprint(key);                                \
                *placed_index = destination;                                                    \
                return PUT_INSERTED;                                                            \
            }                                                                                  \
            assert(is_full(control));                                                           \
            Cell *cell = P##_cell(table, index);                                                \
            if (cell->key == key) {                                                             \
                *old = cell->resource;                                                          \
                cell->resource = offered;                                                       \
                *P##_ctl(table, index) = fingerprint(key);                                      \
                *placed_index = index;                                                          \
                return PUT_REPLACED;                                                            \
            }                                                                                  \
        }                                                                                      \
        if (first_deleted != SIZE_MAX) {                                                        \
            Cell *cell = P##_cell(table, first_deleted);                                        \
            cell->key = key;                                                                    \
            cell->resource = offered;                                                           \
            *P##_ctl(table, first_deleted) = fingerprint(key);                                  \
            *placed_index = first_deleted;                                                      \
            return PUT_INSERTED;                                                                \
        }                                                                                      \
        return PUT_FULL;                                                                        \
    }                                                                                          \
                                                                                               \
    static Resource *P##_find(TABLE *table, uint64_t key, size_t *examined) {                   \
        const size_t start = bucket(key, table->capacity);                                      \
        *examined = 0;                                                                          \
        for (size_t probe = 0; probe < table->capacity; ++probe) {                              \
            const size_t index = probe_index(start, probe, table->capacity);                    \
            const uint8_t control = *P##_ctl(table, index);                                     \
            ++*examined;                                                                        \
            if (control == EMPTY) {                                                             \
                return NULL;                                                                    \
            }                                                                                  \
            if (is_full(control)) {                                                             \
                Cell *cell = P##_cell(table, index);                                            \
                if (control == fingerprint(key) && cell->key == key) {                          \
                    return cell->resource;                                                      \
                }                                                                              \
            }                                                                                  \
        }                                                                                      \
        return NULL;                                                                            \
    }                                                                                          \
                                                                                               \
    static Resource *P##_remove(TABLE *table, uint64_t key) {                                  \
        const size_t start = bucket(key, table->capacity);                                      \
        for (size_t probe = 0; probe < table->capacity; ++probe) {                              \
            const size_t index = probe_index(start, probe, table->capacity);                    \
            const uint8_t control = *P##_ctl(table, index);                                     \
            if (control == EMPTY) {                                                             \
                return NULL;                                                                    \
            }                                                                                  \
            if (is_full(control)) {                                                             \
                Cell *cell = P##_cell(table, index);                                            \
                if (control == fingerprint(key) && cell->key == key) {                          \
                    Resource *resource = cell->resource;                                        \
                    *P##_ctl(table, index) = DELETED;                                           \
                    return resource;                                                            \
                }                                                                              \
            }                                                                                  \
        }                                                                                      \
        return NULL;                                                                            \
    }                                                                                          \
                                                                                               \
    static uint64_t P##_digest(TABLE *table) {                                                  \
        uint64_t digest = UINT64_C(1469598103934665603);                                        \
        digest ^= table->capacity;                                                              \
        digest *= UINT64_C(1099511628211);                                                      \
        for (size_t i = 0; i < table->capacity; ++i) {                                          \
            const uint8_t control = *P##_ctl(table, i);                                         \
            digest ^= control;                                                                  \
            digest *= UINT64_C(1099511628211);                                                  \
            if (is_full(control)) {                                                             \
                Cell *cell = P##_cell(table, i);                                                \
                digest ^= cell->key;                                                            \
                digest *= UINT64_C(1099511628211);                                              \
                digest ^= cell->resource->id;                                                   \
                digest *= UINT64_C(1099511628211);                                              \
            }                                                                                  \
        }                                                                                      \
        return digest;                                                                          \
    }                                                                                          \
                                                                                               \
    static void P##_drop_contents(TABLE *table) {                                               \
        for (size_t i = 0; i < table->capacity; ++i) {                                          \
            if (is_full(*P##_ctl(table, i))) {                                                  \
                resource_drop(P##_cell(table, i)->resource);                                    \
                *P##_ctl(table, i) = EMPTY;                                                     \
            }                                                                                  \
        }                                                                                      \
    }                                                                                          \
                                                                                               \
    static bool P##_begin(TABLE *source, size_t target_capacity, bool force_failure,            \
                          P##Rehash *rehash) {                                                   \
        TABLE target = {0};                                                                     \
        if (!P##_init(&target, target_capacity, force_failure)) {                               \
            return false;                                                                       \
        }                                                                                      \
        *rehash = (P##Rehash){                                                                  \
            .source = *source, .target = target, .phase = PHASE_SCAN, .source_next = 0,          \
        };                                                                                     \
        *source = (TABLE){0};                                                                   \
        return true;                                                                            \
    }                                                                                          \
                                                                                               \
    static void P##_begin_into(TABLE *source, TABLE *target, P##Rehash *rehash) {               \
        assert(source->capacity > 0 && target->capacity > 0);                                   \
        *rehash = (P##Rehash){                                                                  \
            .source = *source, .target = *target, .phase = PHASE_SCAN, .source_next = 0,         \
        };                                                                                     \
        *source = (TABLE){0};                                                                   \
        *target = (TABLE){0};                                                                   \
    }                                                                                          \
                                                                                               \
    static StepResult P##_step(P##Rehash *rehash, size_t budget, size_t *examined,             \
                               size_t *moved) {                                                \
        *examined = 0;                                                                          \
        *moved = 0;                                                                             \
        if (rehash->phase == PHASE_BLOCKED) {                                                   \
            return STEP_BLOCKED_FULL;                                                           \
        }                                                                                      \
        if (rehash->phase == PHASE_DONE) {                                                      \
            return STEP_DONE;                                                                   \
        }                                                                                      \
        while (*examined < budget) {                                                           \
            if (rehash->phase == PHASE_SCAN) {                                                 \
                if (rehash->source_next == rehash->source.capacity) {                           \
                    rehash->phase = PHASE_DONE;                                                 \
                    return STEP_DONE;                                                           \
                }                                                                              \
                const size_t index = rehash->source_next++;                                    \
                const uint8_t control = *P##_ctl(&rehash->source, index);                       \
                ++*examined;                                                                    \
                if (is_full(control)) {                                                         \
                    rehash->held = *P##_cell(&rehash->source, index);                            \
                    rehash->held_valid = true;                                                  \
                    *P##_ctl(&rehash->source, index) = DELETED;                                 \
                    rehash->probe_start = bucket(rehash->held.key, rehash->target.capacity);     \
                    rehash->next_probe = 0;                                                     \
                    rehash->phase = PHASE_PROBE;                                                \
                }                                                                              \
            } else {                                                                           \
                assert(rehash->phase == PHASE_PROBE && rehash->held_valid);                     \
                if (rehash->next_probe == rehash->target.capacity) {                            \
                    rehash->phase = PHASE_BLOCKED;                                              \
                    return STEP_BLOCKED_FULL;                                                   \
                }                                                                              \
                const size_t index = probe_index(                                               \
                    rehash->probe_start, rehash->next_probe, rehash->target.capacity);           \
                ++rehash->next_probe;                                                          \
                const uint8_t control = *P##_ctl(&rehash->target, index);                       \
                ++*examined;                                                                    \
                if (is_vacant(control)) {                                                       \
                    *P##_cell(&rehash->target, index) = rehash->held;                            \
                    *P##_ctl(&rehash->target, index) = fingerprint(rehash->held.key);            \
                    rehash->held_valid = false;                                                 \
                    ++*moved;                                                                    \
                    rehash->phase = PHASE_SCAN;                                                 \
                }                                                                              \
            }                                                                                  \
        }                                                                                      \
        if (rehash->phase == PHASE_SCAN &&                                                      \
            rehash->source_next == rehash->source.capacity) {                                   \
            rehash->phase = PHASE_DONE;                                                         \
            return STEP_DONE;                                                                   \
        }                                                                                      \
        return STEP_PAUSED;                                                                     \
    }                                                                                          \
                                                                                               \
    static void P##_cleanup_rehash(P##Rehash *rehash) {                                        \
        if (rehash->held_valid) {                                                               \
            resource_drop(rehash->held.resource);                                               \
            rehash->held_valid = false;                                                         \
        }                                                                                      \
        P##_drop_contents(&rehash->source);                                                     \
        P##_drop_contents(&rehash->target);                                                     \
        P##_release_backing(&rehash->source);                                                   \
        P##_release_backing(&rehash->target);                                                   \
        rehash->phase = PHASE_DONE;                                                             \
    }

DEFINE_TABLE(i, ITable)
DEFINE_TABLE(s, STable)

typedef struct OracleEntry {
    uint64_t key;
    uint64_t resource_id;
} OracleEntry;

typedef struct Oracle {
    OracleEntry entries[MAX_ORACLE];
    size_t length;
} Oracle;

static bool oracle_find(const Oracle *oracle, uint64_t key, uint64_t *resource_id) {
    for (size_t i = 0; i < oracle->length; ++i) {
        if (oracle->entries[i].key == key) {
            *resource_id = oracle->entries[i].resource_id;
            return true;
        }
    }
    return false;
}

static bool oracle_put(Oracle *oracle, uint64_t key, uint64_t id, uint64_t *old_id) {
    for (size_t i = 0; i < oracle->length; ++i) {
        if (oracle->entries[i].key == key) {
            *old_id = oracle->entries[i].resource_id;
            oracle->entries[i].resource_id = id;
            return true;
        }
    }
    assert(oracle->length < MAX_ORACLE);
    oracle->entries[oracle->length++] = (OracleEntry){.key = key, .resource_id = id};
    return false;
}

static bool oracle_remove(Oracle *oracle, uint64_t key, uint64_t *old_id) {
    for (size_t i = 0; i < oracle->length; ++i) {
        if (oracle->entries[i].key == key) {
            *old_id = oracle->entries[i].resource_id;
            oracle->entries[i] = oracle->entries[--oracle->length];
            return true;
        }
    }
    return false;
}

static void count_resource(unsigned counts[MAX_RESOURCES], Resource *resource) {
    assert(resource != NULL && resource->id < MAX_RESOURCES);
    assert(ledger.created[resource->id] && !ledger.dropped[resource->id]);
    assert(ledger.live[resource->id] == resource);
    for (size_t i = 0; i < 4; ++i) {
        assert(resource->data[i] == resource->id * 17u + i);
    }
    ++counts[resource->id];
}

#define DEFINE_VERIFY(P, TABLE)                                                                \
    static void P##_count_table(TABLE *table, unsigned counts[MAX_RESOURCES]) {                 \
        for (size_t i = 0; i < table->capacity; ++i) {                                          \
            const uint8_t control = *P##_ctl(table, i);                                         \
            assert(is_full(control) || control == EMPTY || control == DELETED);                 \
            if (is_full(control)) {                                                             \
                Cell *cell = P##_cell(table, i);                                                \
                assert(control == fingerprint(cell->key));                                      \
                count_resource(counts, cell->resource);                                         \
            }                                                                                  \
        }                                                                                      \
    }                                                                                          \
                                                                                               \
    static void P##_verify_table(TABLE *table, const Oracle *oracle) {                          \
        size_t actual = 0;                                                                      \
        for (size_t i = 0; i < table->capacity; ++i) {                                          \
            if (is_full(*P##_ctl(table, i))) {                                                  \
                Cell *cell = P##_cell(table, i);                                                \
                uint64_t expected;                                                              \
                assert(oracle_find(oracle, cell->key, &expected));                              \
                assert(expected == cell->resource->id);                                         \
                ++actual;                                                                       \
            }                                                                                  \
        }                                                                                      \
        assert(actual == oracle->length);                                                       \
    }                                                                                          \
                                                                                               \
    static void P##_verify_conservation(TABLE *table, Resource **callers,                      \
                                        size_t caller_count) {                                 \
        unsigned counts[MAX_RESOURCES] = {0};                                                   \
        for (size_t id = 0; id < MAX_RESOURCES; ++id) {                                         \
            if (ledger.dropped[id]) {                                                          \
                ++counts[id];                                                                   \
            }                                                                                  \
        }                                                                                      \
        P##_count_table(table, counts);                                                         \
        for (size_t i = 0; i < caller_count; ++i) {                                             \
            count_resource(counts, callers[i]);                                                 \
        }                                                                                      \
        for (size_t id = 0; id < MAX_RESOURCES; ++id) {                                         \
            assert(counts[id] == (ledger.created[id] ? 1u : 0u));                              \
        }                                                                                      \
    }                                                                                          \
                                                                                               \
    static void P##_verify_rehash(P##Rehash *rehash, const Oracle *oracle,                     \
                                  Resource **callers, size_t caller_count) {                    \
        unsigned counts[MAX_RESOURCES] = {0};                                                   \
        for (size_t id = 0; id < MAX_RESOURCES; ++id) {                                         \
            if (ledger.dropped[id]) {                                                          \
                ++counts[id];                                                                   \
            }                                                                                  \
        }                                                                                      \
        P##_count_table(&rehash->source, counts);                                               \
        P##_count_table(&rehash->target, counts);                                               \
        if (rehash->held_valid) {                                                               \
            count_resource(counts, rehash->held.resource);                                      \
        }                                                                                      \
        for (size_t i = 0; i < caller_count; ++i) {                                             \
            count_resource(counts, callers[i]);                                                 \
        }                                                                                      \
        for (size_t id = 0; id < MAX_RESOURCES; ++id) {                                         \
            if (ledger.created[id]) {                                                          \
                assert(counts[id] == 1);                                                        \
            } else {                                                                           \
                assert(counts[id] == 0);                                                        \
            }                                                                                  \
        }                                                                                      \
        size_t actual = 0;                                                                      \
        for (size_t which = 0; which < 2; ++which) {                                            \
            TABLE *table = which == 0 ? &rehash->source : &rehash->target;                     \
            for (size_t i = 0; i < table->capacity; ++i) {                                     \
                if (is_full(*P##_ctl(table, i))) {                                              \
                    Cell *cell = P##_cell(table, i);                                            \
                    uint64_t expected;                                                          \
                    assert(oracle_find(oracle, cell->key, &expected));                          \
                    assert(expected == cell->resource->id);                                     \
                    ++actual;                                                                   \
                }                                                                              \
            }                                                                                  \
        }                                                                                      \
        if (rehash->held_valid) {                                                               \
            uint64_t expected;                                                                  \
            assert(oracle_find(oracle, rehash->held.key, &expected));                           \
            assert(expected == rehash->held.resource->id);                                      \
            ++actual;                                                                           \
        }                                                                                      \
        assert(actual == oracle->length);                                                       \
    }

DEFINE_VERIFY(i, ITable)
DEFINE_VERIFY(s, STable)

#define CHECK_LAYOUT(P, TABLE, BASE_ID)                                                         \
    static void check_##P(void) {                                                               \
        reset_ledger();                                                                         \
        TABLE failed = {0};                                                                     \
        assert(!P##_init(&failed, 8, true));                                                     \
        assert(failed.capacity == 0);                                                           \
                                                                                               \
        TABLE table = {0};                                                                      \
        assert(P##_init(&table, 8, false));                                                      \
        Oracle oracle = {0};                                                                    \
        Resource *a = resource_new((BASE_ID) + 1);                                              \
        Resource *b = resource_new((BASE_ID) + 2);                                              \
        Resource *c = resource_new((BASE_ID) + 3);                                              \
        Resource *d = resource_new((BASE_ID) + 4);                                              \
        Resource *e = resource_new((BASE_ID) + 5);                                              \
        Resource *old = NULL;                                                                   \
        size_t index = SIZE_MAX;                                                                \
        uint64_t old_id = 0;                                                                    \
        assert(P##_put(&table, 0, a, &old, &index) == PUT_INSERTED && index == 0);              \
        assert(!oracle_put(&oracle, 0, a->id, &old_id));                                        \
        assert(P##_put(&table, 8, b, &old, &index) == PUT_INSERTED && index == 1);              \
        assert(!oracle_put(&oracle, 8, b->id, &old_id));                                        \
        size_t examined = 0;                                                                    \
        assert(P##_find(&table, 8, &examined) == b && examined == 2);                            \
        assert(P##_put(&table, 8, c, &old, &index) == PUT_REPLACED && old == b);                \
        assert(oracle_put(&oracle, 8, c->id, &old_id) && old_id == b->id);                      \
        resource_drop(old);                                                                     \
        old = P##_remove(&table, 0);                                                            \
        assert(old == a && oracle_remove(&oracle, 0, &old_id) && old_id == a->id);              \
        resource_drop(old);                                                                     \
        assert(P##_find(&table, 8, &examined) == c && examined == 2);                            \
        assert(P##_put(&table, 8, e, &old, &index) == PUT_REPLACED && index == 1 && old == c); \
        assert(oracle_put(&oracle, 8, e->id, &old_id) && old_id == c->id);                      \
        resource_drop(old);                                                                     \
        assert(P##_put(&table, 16, d, &old, &index) == PUT_INSERTED && index == 0);             \
        assert(!oracle_put(&oracle, 16, d->id, &old_id));                                       \
        P##_verify_table(&table, &oracle);                                                      \
        P##_verify_conservation(&table, NULL, 0);                                               \
                                                                                               \
        const uint64_t before = P##_digest(&table);                                             \
        const TABLE descriptor_before = table;                                                  \
        P##Rehash refused = {0};                                                                \
        assert(!P##_begin(&table, 16, true, &refused));                                         \
        assert(P##_digest(&table) == before);                                                   \
        assert(P##_same_descriptor(&table, &descriptor_before));                                \
        P##_verify_table(&table, &oracle);                                                      \
        P##_verify_conservation(&table, NULL, 0);                                               \
                                                                                              \
        P##Rehash growth = {0};                                                                 \
        assert(P##_begin(&table, 16, false, &growth));                                          \
        size_t growth_work = 0;                                                                 \
        size_t growth_moved = 0;                                                                \
        assert(P##_step(&growth, SIZE_MAX, &growth_work, &growth_moved) == STEP_DONE);         \
        assert(growth.target.capacity == 16 && growth_moved == oracle.length);                  \
        P##_verify_rehash(&growth, &oracle, NULL, 0);                                           \
        P##_cleanup_rehash(&growth);                                                            \
        assert(ledger.dropped_count == ledger.created_count);                                   \
                                                                                               \
        reset_ledger();                                                                         \
        TABLE full = {0};                                                                       \
        assert(P##_init(&full, 1, false));                                                      \
        Resource *inside = resource_new((BASE_ID) + 10);                                        \
        Resource *offered = resource_new((BASE_ID) + 11);                                       \
        assert(P##_put(&full, 1, inside, &old, &index) == PUT_INSERTED);                        \
        assert(P##_put(&full, 2, offered, &old, &index) == PUT_FULL);                           \
        assert(offered->id == (BASE_ID) + 11);                                                   \
        Resource *caller_owned[] = {offered};                                                   \
        P##_verify_conservation(&full, caller_owned, 1);                                        \
        resource_drop(offered);                                                                 \
        P##_drop_contents(&full);                                                               \
        P##_release_backing(&full);                                                             \
        assert(ledger.dropped_count == ledger.created_count);                                   \
                                                                                               \
        for (size_t stop = 0; stop <= 14; ++stop) {                                             \
            reset_ledger();                                                                     \
            TABLE source = {0};                                                                 \
            assert(P##_init(&source, 8, false));                                                \
            Oracle expected = {0};                                                              \
            for (size_t k = 0; k < 3; ++k) {                                                   \
                Resource *resource = resource_new((BASE_ID) + 100 + stop * 3 + k);              \
                const uint64_t key = k * 8;                                                      \
                assert(P##_put(&source, key, resource, &old, &index) == PUT_INSERTED);          \
                assert(!oracle_put(&expected, key, resource->id, &old_id));                     \
            }                                                                                  \
            P##Rehash migration = {0};                                                          \
            assert(P##_begin(&source, 8, false, &migration));                                   \
            const P##Rehash unchanged = migration;                                              \
            size_t work = SIZE_MAX;                                                             \
            size_t moved = SIZE_MAX;                                                            \
            assert(P##_step(&migration, 0, &work, &moved) == STEP_PAUSED && work == 0 &&        \
                   moved == 0);                                                                 \
            assert(P##_same_rehash(&migration, &unchanged));                                   \
            StepResult result = P##_step(&migration, stop, &work, &moved);                      \
            assert(work <= stop);                                                               \
            P##_verify_rehash(&migration, &expected, NULL, 0);                                  \
            if (stop < 14) {                                                                    \
                assert(result == STEP_PAUSED);                                                   \
            } else {                                                                            \
                assert(result == STEP_DONE);                                                     \
            }                                                                                  \
            P##_cleanup_rehash(&migration);                                                     \
            assert(ledger.dropped_count == ledger.created_count);                               \
        }                                                                                      \
                                                                                               \
        reset_ledger();                                                                         \
        TABLE resumed_source = {0};                                                             \
        assert(P##_init(&resumed_source, 8, false));                                            \
        Oracle resumed_oracle = {0};                                                            \
        for (size_t k = 0; k < 3; ++k) {                                                       \
            Resource *resource = resource_new((BASE_ID) + 200 + k);                            \
            const uint64_t key = k * 8;                                                         \
            assert(P##_put(&resumed_source, key, resource, &old, &index) == PUT_INSERTED);     \
            assert(!oracle_put(&resumed_oracle, key, resource->id, &old_id));                  \
        }                                                                                      \
        P##Rehash resumed = {0};                                                                \
        assert(P##_begin(&resumed_source, 8, false, &resumed));                                 \
        StepResult resumed_result = STEP_PAUSED;                                                \
        size_t resume_calls = 0;                                                                \
        while (resumed_result == STEP_PAUSED) {                                                 \
            size_t work = 0;                                                                    \
            size_t moved = 0;                                                                   \
            resumed_result = P##_step(&resumed, 1, &work, &moved);                             \
            assert(work == 1 && moved <= 1);                                                    \
            P##_verify_rehash(&resumed, &resumed_oracle, NULL, 0);                             \
            assert(++resume_calls <= 14);                                                       \
            if (resumed_result == STEP_PAUSED) {                                                \
                const P##Rehash paused = resumed;                                               \
                assert(P##_step(&resumed, 0, &work, &moved) == STEP_PAUSED);                   \
                assert(work == 0 && moved == 0 && P##_same_rehash(&resumed, &paused));         \
            }                                                                                  \
        }                                                                                      \
        assert(resumed_result == STEP_DONE && resume_calls == 14);                              \
        P##_cleanup_rehash(&resumed);                                                           \
        assert(ledger.dropped_count == ledger.created_count);                                   \
                                                                                               \
        reset_ledger();                                                                         \
        TABLE source = {0};                                                                     \
        TABLE target = {0};                                                                     \
        assert(P##_init(&source, 1, false) && P##_init(&target, 1, false));                     \
        Oracle blocked_oracle = {0};                                                            \
        Resource *moving = resource_new((BASE_ID) + 300);                                       \
        Resource *occupant = resource_new((BASE_ID) + 301);                                     \
        assert(P##_put(&source, 0, moving, &old, &index) == PUT_INSERTED);                      \
        assert(!oracle_put(&blocked_oracle, 0, moving->id, &old_id));                           \
        assert(P##_put(&target, 1, occupant, &old, &index) == PUT_INSERTED);                    \
        assert(!oracle_put(&blocked_oracle, 1, occupant->id, &old_id));                         \
        P##Rehash blocked = {0};                                                                \
        P##_begin_into(&source, &target, &blocked);                                             \
        size_t work = 0;                                                                        \
        size_t moved = SIZE_MAX;                                                                \
        assert(P##_step(&blocked, 3, &work, &moved) == STEP_BLOCKED_FULL);                      \
        assert(work == 2 && moved == 0 && blocked.held_valid && blocked.next_probe == 1);       \
        P##_verify_rehash(&blocked, &blocked_oracle, NULL, 0);                                  \
        const P##Rehash same = blocked;                                                         \
        assert(P##_step(&blocked, 99, &work, &moved) == STEP_BLOCKED_FULL && work == 0 &&       \
               moved == 0);                                                                     \
        assert(P##_same_rehash(&blocked, &same));                                               \
        P##_cleanup_rehash(&blocked);                                                           \
        assert(ledger.dropped_count == ledger.created_count);                                   \
    }

CHECK_LAYOUT(i, ITable, 0)
CHECK_LAYOUT(s, STable, 1000)

typedef struct StructuralStats {
    size_t control_examinations;
    size_t cell_moves;
    size_t cell_move_bytes;
    size_t initialized_control_bytes;
    size_t one_backing_bytes;
    size_t old_new_peak_bytes;
    size_t old_new_peak_allocations;
} StructuralStats;

static void reset_backing_stats(void) {
    assert(backing_live_bytes == 0 && backing_live_allocations == 0);
    backing_peak_bytes = 0;
    backing_peak_allocations = 0;
    control_bytes_initialized = 0;
}

#define DEFINE_STRUCTURAL(P, TABLE)                                                            \
    static StructuralStats structural_##P(void) {                                               \
        reset_ledger();                                                                         \
        reset_backing_stats();                                                                  \
        TABLE source = {0};                                                                     \
        const size_t capacity = 4096;                                                           \
        assert(P##_init(&source, capacity, false));                                             \
        Oracle oracle = {0};                                                                    \
        Resource *old = NULL;                                                                   \
        size_t index = 0;                                                                       \
        uint64_t old_id = 0;                                                                    \
        for (size_t i = 0; i < capacity / 2; ++i) {                                            \
            Resource *resource = resource_new(5000 + i);                                       \
            assert(P##_put(&source, i * 2, resource, &old, &index) == PUT_INSERTED);           \
            assert(!oracle_put(&oracle, i * 2, resource->id, &old_id));                        \
        }                                                                                       \
        const size_t one_backing_bytes = source.bytes;                                          \
        P##Rehash rehash = {0};                                                                 \
        assert(P##_begin(&source, capacity, false, &rehash));                                   \
        size_t examined = 0;                                                                    \
        size_t moved = 0;                                                                       \
        assert(P##_step(&rehash, SIZE_MAX, &examined, &moved) == STEP_DONE);                   \
        assert(examined == capacity + capacity / 2);                                            \
        assert(moved == capacity / 2);                                                         \
        assert(control_bytes_initialized == capacity * 2);                                      \
        P##_verify_rehash(&rehash, &oracle, NULL, 0);                                           \
        StructuralStats stats = {                                                               \
            .control_examinations = examined,                                                   \
            .cell_moves = moved,                                                                \
            .cell_move_bytes = moved * sizeof(Cell),                                            \
            .initialized_control_bytes = control_bytes_initialized,                             \
            .one_backing_bytes = one_backing_bytes,                                             \
            .old_new_peak_bytes = backing_peak_bytes,                                           \
            .old_new_peak_allocations = backing_peak_allocations,                               \
        };                                                                                      \
        P##_cleanup_rehash(&rehash);                                                            \
        assert(ledger.dropped_count == ledger.created_count);                                   \
        assert(backing_live_bytes == 0 && backing_live_allocations == 0);                       \
        return stats;                                                                           \
    }

DEFINE_STRUCTURAL(i, ITable)
DEFINE_STRUCTURAL(s, STable)

static double now_seconds(void) {
#if defined(_WIN32)
    LARGE_INTEGER count;
    LARGE_INTEGER frequency;
    assert(QueryPerformanceCounter(&count) != 0);
    assert(QueryPerformanceFrequency(&frequency) != 0);
    return (double)count.QuadPart / (double)frequency.QuadPart;
#else
    struct timespec time;
    assert(clock_gettime(CLOCK_MONOTONIC, &time) == 0);
    return (double)time.tv_sec + (double)time.tv_nsec / 1000000000.0;
#endif
}

#define DEFINE_MEASURE(P, TABLE)                                                               \
    static void migrate_once_##P(TABLE *table, size_t capacity, uint64_t *witness,              \
                                 size_t *total_examined, size_t *total_moved) {                 \
        P##Rehash rehash = {0};                                                                 \
        assert(P##_begin(table, capacity, false, &rehash));                                     \
        size_t examined = 0;                                                                    \
        size_t moved = 0;                                                                       \
        assert(P##_step(&rehash, SIZE_MAX, &examined, &moved) == STEP_DONE);                   \
        assert(examined == capacity + capacity / 2 && moved == capacity / 2);                  \
        P##_release_backing(&rehash.source);                                                    \
        *table = rehash.target;                                                                 \
        rehash.target = (TABLE){0};                                                             \
        *witness += examined + P##_digest(table);                                               \
        *total_examined += examined;                                                            \
        *total_moved += moved;                                                                  \
    }                                                                                           \
                                                                                               \
    static double measure_migration_##P(size_t capacity, size_t rounds, uint64_t *out_witness, \
                                        size_t *out_examined, size_t *out_moved) {              \
        reset_ledger();                                                                         \
        reset_backing_stats();                                                                  \
        TABLE table = {0};                                                                      \
        assert(P##_init(&table, capacity, false));                                              \
        Resource *old = NULL;                                                                   \
        size_t index = 0;                                                                       \
        for (size_t i = 0; i < capacity / 2; ++i) {                                            \
            Resource *resource = resource_new(5000 + i);                                       \
            assert(P##_put(&table, i * 2, resource, &old, &index) == PUT_INSERTED);            \
        }                                                                                       \
        uint64_t witness = 0;                                                                   \
        size_t total_examined = 0;                                                              \
        size_t total_moved = 0;                                                                 \
        for (size_t warm = 0; warm < 3; ++warm) {                                              \
            migrate_once_##P(&table, capacity, &witness, &total_examined, &total_moved);       \
        }                                                                                       \
        const double start = now_seconds();                                                     \
        for (size_t round = 0; round < rounds; ++round) {                                      \
            migrate_once_##P(&table, capacity, &witness, &total_examined, &total_moved);       \
        }                                                                                       \
        const double elapsed = now_seconds() - start;                                           \
        published_witness += witness;                                                           \
        P##_drop_contents(&table);                                                              \
        P##_release_backing(&table);                                                            \
        assert(ledger.dropped_count == ledger.created_count);                                   \
        assert(backing_live_bytes == 0 && backing_live_allocations == 0);                       \
        *out_witness = witness;                                                                 \
        *out_examined = total_examined;                                                         \
        *out_moved = total_moved;                                                               \
        return elapsed;                                                                         \
    }

DEFINE_MEASURE(i, ITable)
DEFINE_MEASURE(s, STable)

static void check(void) {
    backing_live_bytes = 0;
    backing_peak_bytes = 0;
    backing_live_allocations = 0;
    backing_peak_allocations = 0;
    check_i();
    check_s();
    assert(backing_live_bytes == 0 && backing_live_allocations == 0);
    const StructuralStats interleaved = structural_i();
    const StructuralStats split = structural_s();
    assert(interleaved.control_examinations == split.control_examinations);
    assert(interleaved.cell_moves == split.cell_moves);
    assert(interleaved.cell_move_bytes == split.cell_move_bytes);
    assert(interleaved.initialized_control_bytes == split.initialized_control_bytes);
    size_t split_offset;
    size_t split_bytes;
    assert(s_layout(4096, &split_offset, &split_bytes));
    size_t interleaved_bytes;
    assert(checked_mul(4096, sizeof(ISlot), &interleaved_bytes));
    printf("check=ok variants=2 backing_allocations_each=1 resource_bytes=%zu cell_bytes=%zu\n",
           sizeof(Resource), sizeof(Cell));
    printf("layout capacity=4096 interleaved_bytes=%zu split_offset=%zu split_bytes=%zu\n",
           interleaved_bytes, split_offset, split_bytes);
    printf("migration capacity=4096 entries=2048 examinations=%zu logical_cell_moves=%zu "
           "logical_cell_bytes=%zu initialized_control_bytes=%zu\n",
           interleaved.control_examinations, interleaved.cell_moves,
           interleaved.cell_move_bytes, interleaved.initialized_control_bytes);
    printf("migration_backing interleaved_one=%zu interleaved_peak=%zu "
           "split_one=%zu split_peak=%zu peak_allocations_each=%zu\n",
           interleaved.one_backing_bytes, interleaved.old_new_peak_bytes,
           split.one_backing_bytes, split.old_new_peak_bytes,
           interleaved.old_new_peak_allocations);
    printf("cases=initial-refusal,collision,replace,delete,tomb-reuse,full-insert,rehash-refusal,"
           "growth,budget-stops,resume-budget-one,blocked-full,cleanup\n");
}

static void measure(void) {
    const size_t capacity = 4096;
    const size_t rounds = 100;
    printf("scope=rehash+digest timer=" TIMER_NAME " warmup_rounds=3 timed_rounds=%zu "
           "includes=target-malloc+control-init+scan+probe+source-free+digest "
           "resources=preallocated load=direct-hit-half-full\n",
           rounds);
    for (size_t sample = 0; sample < 5; ++sample) {
        double im;
        double sm;
        uint64_t iw = 0;
        uint64_t sw = 0;
        size_t ie = 0;
        size_t se = 0;
        size_t ic = 0;
        size_t sc = 0;
        if ((sample & 1u) == 0) {
            im = measure_migration_i(capacity, rounds, &iw, &ie, &ic);
            sm = measure_migration_s(capacity, rounds, &sw, &se, &sc);
        } else {
            sm = measure_migration_s(capacity, rounds, &sw, &se, &sc);
            im = measure_migration_i(capacity, rounds, &iw, &ie, &ic);
        }
        assert(iw == sw && ie == se && ic == sc);
        printf("sample=%zu migration_i_ns=%.3f migration_s_ns=%.3f "
               "witness=%" PRIu64 " examinations=%zu cell_moves=%zu\n",
               sample, im * 1e9 / (double)rounds, sm * 1e9 / (double)rounds,
               iw, ie, ic);
    }
    printf("witness=%" PRIu64 "\n", published_witness);
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s check|measure\n", argv[0]);
        return 2;
    }
    if (strcmp(argv[1], "check") == 0) {
        check();
        return 0;
    }
    if (strcmp(argv[1], "measure") == 0) {
        measure();
        return 0;
    }
    fprintf(stderr, "unknown command: %s\n", argv[1]);
    return 2;
}
