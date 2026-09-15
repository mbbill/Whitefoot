#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern int wf_fixture_main(int argc, char **argv);

enum { MAX_ALLOCATIONS = 16 };

typedef struct {
    void *pointer;
    uint64_t bytes;
    size_t request;
    bool released;
} Allocation;

static const uint64_t expected_bytes[] = {
    0, 8, 16, 32, 0, 8, 16, 8, 32, 8, 64,
};

static Allocation allocations[MAX_ALLOCATIONS];
static size_t allocation_count;
static size_t request_count;
static size_t release_order[MAX_ALLOCATIONS];
static size_t release_count;
static size_t fail_at;

static void require(bool condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "vector allocation observer: %s\n", message);
        exit(1);
    }
}

void *wf_observe_allocate(uint64_t bytes) {
    require(request_count < sizeof expected_bytes / sizeof expected_bytes[0],
            "unexpected allocation request");
    ++request_count;
    if (bytes != expected_bytes[request_count - 1]) {
        fprintf(stderr,
                "vector allocation observer: request %zu has %" PRIu64
                " bytes, expected %" PRIu64 "\n",
                request_count, bytes, expected_bytes[request_count - 1]);
        exit(1);
    }
    if (request_count == fail_at) return NULL;

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
        require(release_count < MAX_ALLOCATIONS, "release ledger overflow");
        release_order[release_count++] = allocation->request;
        memset(pointer, 0xa5,
               allocation->bytes == 0 ? 1 : (size_t)allocation->bytes);
        return;
    }
    require(false, "release did not return an allocated address");
}

static void reset(size_t refusal) {
    for (size_t index = 0; index < allocation_count; ++index)
        free(allocations[index].pointer);
    memset(allocations, 0, sizeof allocations);
    memset(release_order, 0, sizeof release_order);
    allocation_count = request_count = release_count = 0;
    fail_at = refusal;
}

static void run_case(size_t refusal, int expected_status,
                     const size_t *expected_releases,
                     size_t expected_release_count) {
    reset(refusal);
    int status = wf_fixture_main(0, NULL);
    if (status != expected_status) {
        fprintf(stderr,
                "vector allocation observer: refusal %zu returned status %d, expected %d\n",
                refusal, status, expected_status);
        exit(1);
    }
    require(release_count == expected_release_count,
            "release count differs from the exact ledger");
    for (size_t index = 0; index < release_count; ++index) {
        if (release_order[index] != expected_releases[index]) {
            fprintf(stderr,
                    "vector allocation observer: release %zu returned request "
                    "%zu, expected %zu\n",
                    index + 1, release_order[index], expected_releases[index]);
            exit(1);
        }
    }
    for (size_t index = 0; index < allocation_count; ++index)
        require(allocations[index].released,
                "an admitted allocation was not released exactly once");
    if (refusal == 0 || refusal == 1 || refusal == 5) {
        require(request_count == sizeof expected_bytes / sizeof expected_bytes[0],
                "successful or zero-byte-refused chain omitted an allocation request");
    } else {
        require(request_count == refusal,
                "execution continued allocating after refusal");
    }
}

int main(void) {
    static const size_t success[] = {1, 2, 3, 4, 5, 7, 9, 10, 6, 8, 11};
    static const size_t zero_1[] = {2, 3, 4, 5, 7, 9, 10, 6, 8, 11};
    static const size_t zero_5[] = {1, 2, 3, 4, 7, 9, 10, 6, 8, 11};
    static const size_t fail_2[] = {1};
    static const size_t fail_3[] = {1, 2};
    static const size_t fail_4[] = {1, 2, 3};
    static const size_t fail_6[] = {1, 2, 3, 4, 5};
    static const size_t fail_7[] = {1, 2, 3, 4, 6, 5};
    static const size_t fail_8[] = {1, 2, 3, 4, 5, 6, 7};
    static const size_t fail_9[] = {1, 2, 3, 4, 5, 8, 6, 7};
    static const size_t fail_10[] = {1, 2, 3, 4, 5, 7, 6, 8, 9};
    static const size_t fail_11[] = {1, 2, 3, 4, 5, 7, 10, 6, 8, 9};

    run_case(0, 0, success, sizeof success / sizeof success[0]);
    run_case(1, 0, zero_1, sizeof zero_1 / sizeof zero_1[0]);
    run_case(2, 70, fail_2, sizeof fail_2 / sizeof fail_2[0]);
    run_case(3, 70, fail_3, sizeof fail_3 / sizeof fail_3[0]);
    run_case(4, 70, fail_4, sizeof fail_4 / sizeof fail_4[0]);
    run_case(5, 0, zero_5, sizeof zero_5 / sizeof zero_5[0]);
    run_case(6, 70, fail_6, sizeof fail_6 / sizeof fail_6[0]);
    run_case(7, 70, fail_7, sizeof fail_7 / sizeof fail_7[0]);
    run_case(8, 70, fail_8, sizeof fail_8 / sizeof fail_8[0]);
    run_case(9, 70, fail_9, sizeof fail_9 / sizeof fail_9[0]);
    run_case(10, 70, fail_10, sizeof fail_10 / sizeof fail_10[0]);
    run_case(11, 70, fail_11, sizeof fail_11 / sizeof fail_11[0]);
    reset(0);
    puts("vector allocation observer: every refusal and release ledger passed");
    return 0;
}
