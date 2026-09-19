// Native ownership evidence for the owning-map program, at v0.60.
//
// Retired subject: the allocation-refusal sweep this observer used to drive by
// returning NULL from the replaced `malloc` at each position in turn. [STOR-8]
// makes allocation total in the source -- it never returns a failure and never
// traps -- and the emitted module now answers a null allocation by calling
// `wf_resource_abort` and falling into `unreachable`, so a refusal has no
// source-visible subject and the program has no refusal exit code to check.
// Successor: the identity half below, which is the part an exit code still
// cannot provide -- every owner is a distinct allocation and is released
// exactly once, with nothing left live at exit.
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { max_allocations = 4096 };
struct Cell {
    void *pointer;
    unsigned live;
    unsigned releases;
    uint64_t payload;
};
static struct Cell cells[max_allocations + 1];
static unsigned requests, allocated, released;
static int ledger;

static void fail(const char *reason, unsigned id) {
    fprintf(stderr, "FAIL requests=%u id=%u: %s\n", requests, id, reason);
    exit(100);
}

void *wf_observe_allocate(size_t bytes) {
    unsigned id = ++requests;
    if (id > max_allocations) fail("too many allocation requests", id);
    if (bytes != sizeof(uint64_t)) fail("unexpected Box allocation size", id);
    void *pointer = malloc(bytes);
    if (!pointer) fail("observer host allocation failed", id);
    for (unsigned prior = 1; prior < id; ++prior) {
        if (cells[prior].pointer == pointer) fail("identity reused within a run", id);
    }
    cells[id] = (struct Cell){ .pointer = pointer, .live = 1, .releases = 0, .payload = 0 };
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
    cells[id].payload = value;
    cells[id].live = 0;
    ++cells[id].releases;
    ++released;
    if (ledger) printf("F%u=%" PRIu64 ";", id, value);
    // Quarantine until the run finishes so a stale pointer cannot alias a later
    // owner. This observer changes allocator address reuse, not the fixture's
    // algorithm.
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
    memset(cells, 0, sizeof cells);
    int status = wf_fixture_main(argc, argv);
    if (status != 0) fail("unexpected command exit status", 0);
    if (!requests) fail("the fixture allocated no owner", 0);
    if (allocated != requests || released != requests) fail("incomplete owner cleanup", 0);
    for (unsigned id = 1; id <= requests; ++id) {
        if (!cells[id].pointer || cells[id].live || cells[id].releases != 1) {
            fail("owner was not released exactly once", id);
        }
        free(cells[id].pointer);
    }
    printf("PASS status=%d allocations=%u releases=%u live=0\n", status, allocated, released);
    return 0;
}
