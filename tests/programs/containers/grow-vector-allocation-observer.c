// The release ledger of the container library's test program, observed by
// interposing every generated allocation and release.
//
// Retired subject: the allocation-refusal sweep. v0.59 returned null at each
// allocation request in turn and read the source-visible fallback the library
// installed -- an `Err(unit)` reserve, an `Err(value)` append, a retained
// owner -- and pinned the exact byte count of every request beside it.
// [STOR-8] makes allocation total in the source: it never returns a failure,
// never traps, and no allocating operation carries a `Result`, and an
// exhausted heap ends the program from the trusted base outside the language
// [SCOPE-3]. There is no refusal for a program to observe and no fallback arm
// to reach, so the sweep and its expected-byte table both go with the rule.
//
// The successor kept here is the identity half, which [STOR-3] and [WIN-3]
// still fix: every allocation the program makes is released exactly once,
// never twice, and none is left behind when the entry returns. That property
// is what catches a missed release walk over an owning element, a double free
// of a superseded backing, and a release of storage the program never owned.
#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern int wf_fixture_main(int argc, char **argv);

enum { MAX_ALLOCATIONS = 64 };

typedef struct {
    void *pointer;
    uint64_t bytes;
    size_t request;
    bool released;
} Allocation;

static Allocation allocations[MAX_ALLOCATIONS];
static size_t allocation_count;
static size_t request_count;
static size_t release_count;

static void require(bool condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "vector allocation observer: %s\n", message);
        exit(1);
    }
}

void *wf_observe_allocate(uint64_t bytes) {
    ++request_count;
    // Allocation is total: the trusted base either hands back storage or ends
    // the process, so this observer never returns null.
    void *pointer = malloc(bytes == 0 ? 1 : (size_t)bytes);
    require(pointer != NULL, "host allocation failed during observation");
    require(allocation_count < MAX_ALLOCATIONS, "allocation ledger overflow");
    memset(pointer, 0xcc, bytes == 0 ? 1 : (size_t)bytes);
    allocations[allocation_count++] =
        (Allocation){pointer, bytes, request_count, false};
    return pointer;
}

void wf_observe_release(void *pointer) {
    if (pointer == NULL) return;
    for (size_t index = 0; index < allocation_count; ++index) {
        Allocation *allocation = &allocations[index];
        if (allocation->pointer != pointer) continue;
        require(!allocation->released, "allocation released twice");
        allocation->released = true;
        ++release_count;
        memset(pointer, 0xa5,
               allocation->bytes == 0 ? 1 : (size_t)allocation->bytes);
        return;
    }
    require(false, "release did not return an allocated address");
}

int main(void) {
    int status = wf_fixture_main(0, NULL);
    if (status != 0) {
        fprintf(stderr,
                "vector allocation observer: the fixture returned status %d, "
                "expected 0\n",
                status);
        exit(1);
    }
    require(request_count > 0, "the fixture must reach the heap at all");
    require(release_count == allocation_count,
            "every allocation is released exactly once");
    for (size_t index = 0; index < allocation_count; ++index) {
        if (!allocations[index].released) {
            fprintf(stderr,
                    "vector allocation observer: request %zu (%" PRIu64
                    " bytes) was never released\n",
                    allocations[index].request, allocations[index].bytes);
            exit(1);
        }
    }
    for (size_t index = 0; index < allocation_count; ++index)
        free(allocations[index].pointer);
    printf("vector allocation observer: %zu allocations, each released exactly once\n",
           allocation_count);
    return 0;
}
