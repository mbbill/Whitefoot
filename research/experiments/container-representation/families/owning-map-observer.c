#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { total_allocations = 192, per_seed = 12 };
struct Cell {
    void *pointer;
    unsigned live;
    unsigned releases;
};
static struct Cell cells[total_allocations + 1];
static unsigned refused, requests, allocated, released, full_returned;
static int ledger;
// Each source exercise supplies these twelve owners. Allocation twelve is
// offered to the genuinely full table and must be returned unchanged.
static const uint64_t payloads[per_seed] = {
    11, 22, 33, 44, 55, 66, 100, 101, 102, 103, 104, 999
};

static void fail(const char *reason, unsigned id) {
    fprintf(stderr, "FAIL refusal=%u requests=%u id=%u: %s\n", refused, requests, id, reason);
    exit(100);
}

void *wf_observe_allocate(size_t bytes) {
    unsigned id = ++requests;
    if (id > total_allocations) fail("too many allocation requests", id);
    if (refused && id > refused) fail("request after first refusal", id);
    if (bytes != sizeof(uint64_t)) fail("unexpected Box allocation size", id);
    if (id == refused) {
        if (ledger) printf("X%u;", id);
        return NULL;
    }
    void *pointer = malloc(bytes);
    if (!pointer) fail("observer host allocation failed", id);
    for (unsigned prior = 1; prior < id; ++prior) {
        if (cells[prior].pointer == pointer) fail("identity reused within a run", id);
    }
    cells[id] = (struct Cell){ .pointer = pointer, .live = 1, .releases = 0 };
    ++allocated;
    if (ledger) printf("A%u;", id);
    return pointer;
}

void wf_observe_release(void *pointer) {
    if (!pointer) fail("null release", 0);
    unsigned id = 0;
    for (unsigned candidate = 1; candidate <= requests; ++candidate) {
        if (cells[candidate].pointer == pointer) {
            id = candidate;
            break;
        }
    }
    if (!id) fail("release does not name a supplied owner", 0);
    if (!cells[id].live || cells[id].releases) fail("duplicate release", id);
    uint64_t value;
    memcpy(&value, pointer, sizeof value);
    if (value != payloads[(id - 1) % per_seed]) fail("owner payload identity changed", id);
    if (id % per_seed == 0) {
        if (requests != id || value != 999) fail("Full did not return current offered owner", id);
        ++full_returned;
    }
    cells[id].live = 0;
    ++cells[id].releases;
    ++released;
    if (ledger) printf("F%u=%" PRIu64 ";", id, value);
    // Quarantine until the run finishes so a stale pointer cannot alias a later owner.
    // This observer changes allocator address reuse, not the fixture's algorithm.
    memset(pointer, 0xa5, sizeof(uint64_t));
}

extern int wf_fixture_main(int argc, char **argv);

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--ledger") == 0) {
        ledger = 1;
    } else if (argc != 1) {
        fprintf(stderr, "usage: owning-map-observer [--ledger]\n");
        return 64;
    }
    unsigned long all_allocated = 0, all_released = 0, all_full = 0;
    for (unsigned failure = 0; failure <= total_allocations; ++failure) {
        memset(cells, 0, sizeof cells);
        refused = failure;
        requests = allocated = released = full_returned = 0;
        if (ledger) printf("run refusal=%u;", refused);
        int status = wf_fixture_main(argc, argv);
        unsigned expected_requests = refused ? refused : total_allocations;
        unsigned expected_owners = refused ? refused - 1 : total_allocations;
        if (status != (refused ? 70 : 0)) fail("unexpected command exit status", 0);
        if (requests != expected_requests) fail("wrong number of requests", 0);
        if (allocated != expected_owners || released != expected_owners) fail("incomplete owner cleanup", 0);
        if (full_returned != expected_owners / per_seed) fail("wrong Full owner count", 0);
        for (unsigned id = 1; id <= requests; ++id) {
            if (id == refused) {
                if (cells[id].pointer || cells[id].live || cells[id].releases) fail("refused allocation became an owner", id);
                continue;
            }
            if (!cells[id].pointer || cells[id].live || cells[id].releases != 1) fail("owner was not released exactly once", id);
            free(cells[id].pointer);
        }
        if (ledger) {
            printf("PASS status=%d requests=%u allocated=%u released=%u full=%u live=0\n",
                status, requests, allocated, released, full_returned);
        }
        all_allocated += allocated;
        all_released += released;
        all_full += full_returned;
    }
    printf("PASS runs=193 refusal_positions=1..192 allocations=%lu releases=%lu full_original_owners=%lu live=0\n",
        all_allocated, all_released, all_full);
    return 0;
}
