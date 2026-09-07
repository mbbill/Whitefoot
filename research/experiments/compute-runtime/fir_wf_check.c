/* Project the unchanged complete native oracle onto four WF tile sizes.
 * Only the oracle translation unit's references are renamed. The exact
 * optimized WF object also used for timing retains all of its own symbols.
 * This checks every tap count and lane boundary through ordinary WF calls;
 * the original streaming/history/miss suite remains a separate caller. */
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#ifdef WF_COMPUTE_CONTROL
#include "runtime.h"
#endif

typedef void *(*FilterEntry)(const double *, uint64_t, const double *, uint64_t,
                             uint64_t, uint64_t, uint64_t, uint64_t);
extern void *wf_research_fir_parallel(const double *, uint64_t, const double *,
                                    uint64_t, uint64_t, uint64_t, uint64_t, uint64_t);
extern void *wf_research_fir_sequential(const double *, uint64_t, const double *,
                                      uint64_t, uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_research_fir_count(const void *);
extern uint64_t wf_research_fir_get(const void *, uint64_t, double *);
extern uint64_t wf_research_fir_history(const double *, uint64_t, uint64_t,
                                      uint64_t, double *);
extern uint64_t wf_research_fir_release(void *);
extern int wf__par_pool_active(void);
extern int wf__floor_run(int, char **);
extern int fir_qualified_oracle_main(void);
static FilterEntry filter;
static uint64_t calls, samples, states, misses;

static void require(int condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "FIR WF object qualification: %s\n", message);
        exit(1);
    }
}

static void checked_filter(const double *prefix, const double *taps, size_t k,
                            size_t n, double *output, uint64_t tile) {
    require(k >= 1 && k <= 64 && n <= 16777216 - (k - 1), "domain");
    size_t h = k - 1;
    void *tree = filter(prefix, n + h, taps, k, h, n + h, h, tile);
    require(tree != NULL && wf_research_fir_count(tree) == n, "result count");
    for (size_t i = 0; i < n; ++i) {
        require(wf_research_fir_get(tree, i, output + i) == 1, "missing sample");
        ++samples;
    }
    for (size_t i = 0; i < h; ++i) {
        double actual = 99.0;
        require(wf_research_fir_history(prefix, n + h, h, i, &actual) == 1,
                "missing history");
        require(memcmp(&actual, prefix + n + i, sizeof(actual)) == 0, "history bits");
        ++states;
    }
    double absent = 99.0;
    require(wf_research_fir_get(tree, n, &absent) == 0 && absent == 99.0, "sample miss");
    require(wf_research_fir_get(tree, UINT64_MAX, &absent) == 0 && absent == 99.0,
            "large sample miss");
    require(wf_research_fir_history(prefix, n + h, h, h, &absent) == 0 && absent == 99.0,
            "history miss");
    misses += 3;
    require(wf_research_fir_release(tree) == 0, "release status");
    ++calls;
}

#define TILE_ADAPTER(tile) \
void fir_wf_tile##tile(const double *prefix, const double *taps, size_t k, \
                       size_t n, double *output) { \
    checked_filter(prefix, taps, k, n, output, tile); \
}
TILE_ADAPTER(1)
TILE_ADAPTER(17)
TILE_ADAPTER(257)
TILE_ADAPTER(65536)
#undef TILE_ADAPTER

int wf__main_body(int argc, char **argv) {
    (void)argc;
    (void)argv;
    alarm(120); /* A hung qualification fails; elapsed time never selects WF acceptance. */
    int parallel = wf__par_pool_active() != 0;
    filter = parallel ? wf_research_fir_parallel : wf_research_fir_sequential;
    require(fir_qualified_oracle_main() == 0, "complete oracle");
    require(calls == 107408 && samples == 1294336 && states == 3752300 && misses == 322224,
            "coverage counts");
#ifdef WF_COMPUTE_CONTROL
    require(parallel ? wf_compute_worker_count() == 4 : wf_compute_worker_count() == 0,
            "actual worker count");
#endif
    printf("FIR WF object qualification PASS: world=%s tiles=4 calls=%" PRIu64
           " samples=%" PRIu64 " history=%" PRIu64 " misses=%" PRIu64 " K=1..64\n",
           parallel ? "parallel" : "sequential", calls, samples, states, misses);
    alarm(0);
    return 0;
}

int main(int argc, char **argv) {
    return wf__floor_run(argc, argv);
}
