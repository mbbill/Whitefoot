#define _POSIX_C_SOURCE 200112L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static uintptr_t selected_payload_residue;
static unsigned long allocation_count;
static unsigned long release_count;

__attribute__((constructor)) static void initialize_payload_residue(void) {
    const char *text = getenv("WF_RECORDS_PAYLOAD_RESIDUE");
    char *end = NULL;
    unsigned long parsed;
    if (!text || !*text) abort();
    parsed = strtoul(text, &end, 10);
    if (*end || !(parsed == 16 || parsed == 24 || parsed == 48 || parsed == 56))
        abort();
    selected_payload_residue = (uintptr_t)parsed;
}

/* Called only from the compiler-produced result-cell construction. Parsing and
 * validation happened at process startup, outside every measured interval.
 * Every admitted residue follows this same allocation instruction path. */
void *wf_records_result_allocate(uint64_t bytes) {
    void *raw = NULL;
    uintptr_t aligned;
    uintptr_t cell;
    if (posix_memalign(&raw, 64, (size_t)bytes + 128) != 0) return NULL;
    aligned = (uintptr_t)raw;
    cell = aligned + ((selected_payload_residue + 56) & 63) + 64;
    ((void **)cell)[-1] = raw;
    if (((cell + 8) & 63) != selected_payload_residue) abort();
    allocation_count += 1;
    return (void *)cell;
}

void wf_records_result_release(void *cell) {
    uintptr_t payload;
    if (!cell) abort();
    payload = (uintptr_t)cell + 8;
    if ((payload & 63) != selected_payload_residue) abort();
    release_count += 1;
    free(((void **)cell)[-1]);
}

__attribute__((destructor)) static void report_payload_residue(void) {
    if (allocation_count != release_count) abort();
    (void)fprintf(stderr,
                  "WF_RESULT_PAYLOAD residue=%lu allocations=%lu releases=%lu module_data=%p\n",
                  (unsigned long)selected_payload_residue, allocation_count,
                  release_count, (void *)&selected_payload_residue);
}
