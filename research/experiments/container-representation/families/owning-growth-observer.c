/* The retained native sparse algorithm is the control, not a translation of WF.
 * Allocation quarantine makes premature release and address substitution visible. */
#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void *wf_observe_allocate(uint64_t bytes);
void wf_observe_release(void *pointer);
#define malloc wf_observe_allocate
#define free wf_observe_release
#define main retained_sparse_main
#include "../costs/sparse-owned.c"
#undef main
#undef free
#undef malloc

extern uint64_t wf_map_contract(void *, uint64_t, uint64_t, uint64_t);

typedef struct {
    void *pointer;
    uint64_t bytes, id;
    bool released;
} ObservedAllocation;

typedef struct {
    ObservedAllocation allocations[16];
    size_t requests, allocated, released, fail_at, live, peak;
} Observation;

static Observation observation;
static size_t compared_runs, compared_owners, compared_backings;
static uint64_t expected_requests[7];
static size_t expected_request_count;

static void require(bool condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "owning-growth: %s\n", message);
        exit(1);
    }
}

void *wf_observe_allocate(uint64_t bytes) {
    require(observation.requests < expected_request_count, "unexpected allocation request");
    require(bytes == expected_requests[observation.requests],
            "allocation differs from its exact request-position extent");
    ++observation.requests;
    if (observation.requests == observation.fail_at) return NULL;
    void *pointer = malloc((size_t)bytes);
    require(pointer != NULL, "host allocation failed");
    memset(pointer, 0xcc, (size_t)bytes);
    observation.allocations[observation.allocated++] =
        (ObservedAllocation){pointer, bytes, 0, false};
    observation.live += (size_t)bytes;
    if (observation.live > observation.peak) observation.peak = observation.live;
    return pointer;
}

void wf_observe_release(void *pointer) {
    if (!pointer) return;
    for (size_t i = 0; i < observation.allocated; ++i) {
        ObservedAllocation *entry = &observation.allocations[i];
        if (entry->pointer != pointer) continue;
        require(!entry->released, "owner released twice");
        if (entry->bytes == sizeof(Resource)) {
            Resource resource;
            memcpy(&resource, pointer, sizeof resource);
            entry->id = resource.id;
            for (size_t word = 0; word < 4; ++word)
                require(resource.data[word] == resource.id * 17 + word,
                        "resource identity or contents changed");
        }
        entry->released = true;
        ++observation.released;
        observation.live -= (size_t)entry->bytes;
        memset(pointer, 0xa5, (size_t)entry->bytes);
        return;
    }
    require(false, "release did not return the original allocation address");
}

static void observation_reset(size_t fail_at, uint64_t scenario, bool whitefoot) {
    require(observation.live == 0, "live allocations before reset");
    for (size_t i = 0; i < observation.allocated; ++i)
        free(observation.allocations[i].pointer);
    memset(&observation, 0, sizeof observation);
    observation.fail_at = fail_at;
    /* Source: OP-9 sequences (tag 4, fingerprint 1, key 8, Box ceiling 16)
     * to 32 bytes; emitted heap allocation uses that ceiling. LLVM's actual
     * Slot and sparse-owned.c's ISlot both have a 24-byte element stride.
     * Check each side's exact allocation sequence, not equality of layouts. */
    const uint64_t stride = whitefoot ? 32 : sizeof(ISlot);
    expected_requests[0] = (scenario >= 3 ? 1 : 8) * stride;
    const size_t resources = scenario >= 3 ? 1 : scenario == 0 ? 5 : 3;
    for (size_t i = 1; i <= resources; ++i) expected_requests[i] = sizeof(Resource);
    expected_request_count = resources + 1;
    if (scenario == 4) {
        expected_requests[expected_request_count++] = sizeof(Resource);
    } else {
        expected_requests[expected_request_count++] =
            (scenario == 0 ? 16 : scenario == 3 ? 1 : 8) * stride;
        if (scenario == 3) expected_requests[expected_request_count++] = sizeof(Resource);
    }
    reset_ledger();
    require(backing_live_bytes == 0 && backing_live_allocations == 0,
            "native control retained a backing");
    backing_peak_bytes = backing_peak_allocations = control_bytes_initialized = 0;
}

static Observation completed_observation(void) {
    require(observation.allocated == observation.released && observation.live == 0,
            "not every admitted owner was returned exactly once");
    if (observation.fail_at != 0)
        require(observation.requests == observation.fail_at,
                "execution continued allocating after refusal");
    else
        require(observation.requests == expected_request_count,
                "successful execution omitted a required allocation");
    return observation;
}

static Resource *try_resource(uint64_t id) {
    Resource *resource = wf_observe_allocate(sizeof *resource);
    if (!resource) return NULL;
    resource->id = id;
    for (size_t i = 0; i < 4; ++i) resource->data[i] = id * 17 + i;
    require(id < MAX_RESOURCES && !ledger.created[id], "native owner reused");
    ledger.created[id] = true;
    ledger.live[id] = resource;
    ++ledger.created_count;
    return resource;
}

static void drop_table(ITable *table) {
    i_drop_contents(table);
    i_release_backing(table);
}

static uint64_t native_state_hash(iRehash *state) {
    uint64_t hash = i_digest(&state->source) * 131 + i_digest(&state->target);
    hash = hash * 131 + state->source_next;
    hash = hash * 131 + state->next_probe;
    hash = hash * 131 + state->phase;
    hash *= 131;
    if (state->held_valid) hash += state->held.key;
    hash *= 131;
    if (state->held_valid) hash += state->held.resource->id;
    return hash;
}

static uint64_t native_contract(uint64_t seed, uint64_t scenario, size_t budget) {
    size_t initial = scenario >= 3 ? 1 : 8;
    size_t entries = scenario >= 3 ? 1 : scenario == 0 ? 5 : 3;
    uint64_t base = seed * 100;
    ITable source = {0};
    if (!i_init(&source, initial, false)) return 70;
    for (size_t ordinal = 0; ordinal < entries; ++ordinal) {
        uint64_t offset = ordinal * 8, kind = 1, expected = 0;
        if (scenario == 0) {
            if (ordinal == 2) { offset = 8; kind = 2; expected = base + 2; }
            if (ordinal == 3) {
                Resource *removed = i_remove(&source, seed);
                size_t work;
                require(removed != NULL, "native removal");
                require(removed->id == base + 1, "native removed owner");
                resource_drop(removed);
                Resource *found = i_find(&source, seed + 8, &work);
                require(found && found->id == base + 3 && work == 2,
                        "native lookup past tombstone");
                offset = 8; kind = 2; expected = base + 3;
            }
            if (ordinal == 4) offset = 16;
        }
        Resource *offered = try_resource(base + ordinal + 1);
        if (!offered) { drop_table(&source); return 70; }
        Resource *returned = NULL;
        size_t placed;
        PutResult result = i_put(&source, seed + offset, offered, &returned, &placed);
        require((kind == 1 && result == PUT_INSERTED) ||
                (kind == 2 && result == PUT_REPLACED), "native insertion kind");
        if (kind == 2) {
            require(returned->id == expected, "native displaced owner");
            resource_drop(returned);
        }
    }
    if (scenario == 4) {
        Resource *offered = try_resource(base + 2);
        if (!offered) { drop_table(&source); return 70; }
        Resource *returned = NULL;
        size_t placed;
        PutResult result = i_put(&source, seed + 1, offered, &returned, &placed);
        require(result == PUT_FULL && returned == NULL, "native full return");
        resource_drop(offered);
        uint64_t hash = i_digest(&source);
        drop_table(&source);
        return hash;
    }
    size_t target_capacity = scenario == 0 ? 16 : scenario == 3 ? 1 : 8;
    iRehash migration;
    uint64_t before = i_digest(&source);
    if (!i_begin(&source, target_capacity, false, &migration)) {
        require(i_digest(&source) == before, "native refusal changed source");
        drop_table(&source);
        return 70;
    }
    if (scenario == 3) {
        Resource *offered = try_resource(base + 2);
        if (!offered) { i_cleanup_rehash(&migration); return 70; }
        Resource *returned = NULL;
        size_t placed;
        PutResult result = i_put(&migration.target, seed + 1, offered, &returned, &placed);
        require(result == PUT_INSERTED, "native full target setup");
    }
    size_t total_work = 0, total_moved = 0;
    if (scenario == 2) {
        for (size_t round = 0; round < 14; ++round) {
            size_t work, moved;
            uint64_t before_idle = native_state_hash(&migration);
            i_step(&migration, 0, &work, &moved);
            require(native_state_hash(&migration) == before_idle && !work && !moved,
                    "native zero-budget state");
            i_step(&migration, 1, &work, &moved);
            require(work == 1, "native one-unit progress");
            total_work += work; total_moved += moved;
        }
    } else {
        i_step(&migration, budget, &total_work, &total_moved);
        if (scenario == 3) {
            size_t work, moved;
            uint64_t before_retry = native_state_hash(&migration);
            i_step(&migration, 99, &work, &moved);
            require(native_state_hash(&migration) == before_retry && !work && !moved,
                    "native blocked retry");
        }
    }
    uint64_t result = (native_state_hash(&migration) * 131 + total_work) * 131 + total_moved;
    i_cleanup_rehash(&migration);
    return result;
}

static void compare(uint64_t seed, uint64_t scenario, size_t budget, size_t fail_at) {
    observation_reset(fail_at, scenario, false);
    uint64_t expected = native_contract(seed, scenario, budget);
    Observation native = completed_observation();
    observation_reset(fail_at, scenario, true);
    uint64_t actual = wf_map_contract(NULL, seed, scenario, budget);
    if (actual != expected) {
        fprintf(stderr, "seed=%" PRIu64 " scenario=%" PRIu64 " budget=%zu refusal=%zu"
                        " expected=%" PRIu64 " actual=%" PRIu64 "\n",
                seed, scenario, budget, fail_at, expected, actual);
        exit(1);
    }
    Observation wf = completed_observation();
    require(wf.requests == native.requests && wf.allocated == native.allocated,
            "allocation ledger differs from native control");
    for (size_t i = 0; i < wf.allocated; ++i) {
        require(wf.allocations[i].id == native.allocations[i].id,
                "exact owner release ledger differs from native control");
        if (native.allocations[i].bytes == sizeof(Resource)) {
            require(wf.allocations[i].bytes == sizeof(Resource), "resource extent changed");
            ++compared_owners;
        } else {
            require(wf.allocations[i].bytes / 32 == native.allocations[i].bytes / sizeof(ISlot),
                    "released backing capacity differs from native control");
            ++compared_backings;
        }
    }
    ++compared_runs;
}

int main(void) {
    for (uint64_t seed = 0; seed < 16; ++seed) {
        for (uint64_t scenario = 0; scenario < 5; ++scenario) {
            size_t stops = scenario == 1 ? 15 : 1;
            for (size_t stop = 0; stop < stops; ++stop) {
                size_t budget = scenario == 0 ? 100 : scenario == 3 ? 3 : stop;
                compare(seed, scenario, budget, 0);
                size_t requests = observation.requests;
                for (size_t fail_at = 1; fail_at <= requests; ++fail_at)
                    compare(seed, scenario, budget, fail_at);
            }
        }
    }
    observation_reset(0, 0, false);
    printf("owning-growth: %zu matched executions; %zu resource releases; %zu backing releases\n",
           compared_runs, compared_owners, compared_backings);
    return 0;
}
