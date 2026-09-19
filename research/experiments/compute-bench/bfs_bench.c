/* Reuse the formally owned result oracle; timing/comparison forms stay here. */
#define WF_ORACLE_NO_MAIN
#include "../../../tests/programs/compute/bfs_oracle.c"

#include "backend.h"
#include "harness.h"
extern void wf_bench_bfs_par(const uint64_t *, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_bfs_seq(const uint64_t *, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_bfs_par_release(uint64_t *, uint64_t);
extern void wf_bench_bfs_seq_release(uint64_t *, uint64_t);
static const wfb_backend *backend;
static unsigned width;
typedef struct {
    const uint64_t *edges, *previous;
    uint64_t *output;
    size_t count, level, chunks;
    size_t changed[16 * WFB_MAX_WIDTH];
} pull_job;
static void pull_chunk(void *context, size_t chunk) {
    pull_job *job = context;
    size_t start = job->count * chunk / job->chunks;
    size_t end = job->count * (chunk + 1) / job->chunks;
    size_t changed = 0;
    for (size_t v = start; v < end; ++v) {
        uint64_t distance = job->previous[v];
        if (distance == job->count) {
            for (size_t slot = 0; slot < 4; ++slot) {
                uint64_t neighbor = job->edges[4 * v + slot];
                if (neighbor < job->count && job->previous[neighbor] == job->level) {
                    distance = job->level + 1; ++changed; break;
                }
            }
        }
        job->output[v] = distance;
    }
    job->changed[chunk] = changed;
}
static void native_release(uint64_t *p, uint64_t n) { (void)n; free(p); }
static void native_entry(const uint64_t *edges, uint64_t slots, uint64_t pull, uint64_t **out, uint64_t *length) {
    size_t n = slots / 4;
    if (!pull || backend == &wfb_backend_serial) {
        *out = oracle(edges, n, NULL);
    } else {
        uint64_t *previous = allocate(n), *next = allocate(n);
        for (size_t v = 0; v < n; ++v) previous[v] = next[v] = n;
        if (n) previous[0] = 0;
        size_t chunks = 16 * width;
        if (chunks > n) chunks = n;
        for (size_t level = 0; level < n; ++level) {
            pull_job job = {.edges=edges, .previous=previous, .output=next,
                            .count=n, .level=level, .chunks=chunks};
            backend->map(backend, width, chunks, pull_chunk, &job);
            size_t changed = 0;
            for (size_t chunk = 0; chunk < chunks; ++chunk) changed += job.changed[chunk];
            uint64_t *swap = previous; previous = next; next = swap;
            if (!changed) break;
        }
        free(next); *out = previous;
    }
    *length = n;
}
static size_t count = 65535;
static unsigned shape = 1, pull_mode;
static uint64_t *edges, *expected, *output;
static bfs_release release_output;
static char workload[256];
static void select_backend(const char *form, unsigned workers) {
    backend = wfb_backend_named(form); width = workers;
    if (!backend || !backend->map) fail("missing map backend");
}
static void prepare(unsigned workers) {
    (void)workers;
    const char *fixture = getenv("WFB_BFS_GRAPH"), *mode = getenv("WFB_BFS_MODE");
    if (fixture && !strcmp(fixture, "chain")) { count = 4097; shape = 0; }
    else if (fixture && !strcmp(fixture, "grid")) { count = 16384; shape = 4; }
    else if (fixture && !strcmp(fixture, "disconnected")) { count = 4097; shape = 2; }
    else if (fixture && strcmp(fixture, "tree")) fail("WFB_BFS_GRAPH must be tree, chain, grid or disconnected");
    if (mode && !strcmp(mode, "pull")) pull_mode = 1;
    else if (mode && strcmp(mode, "sparse")) fail("WFB_BFS_MODE must be sparse or pull");
    edges = graph(count, shape);
    work_count work;
    expected = oracle(edges, count, &work);
    (void)snprintf(workload, sizeof(workload),
        "vertices=%zu shape=%u mode=%s reached=%zu levels=%zu sparse_edge_slots=%zu pull_vertex_visits=%zu workspace=2n serial=FIFO",
        count, shape, pull_mode ? "pull" : "sparse", work.reached, work.levels,
        work.slots, count * work.levels);
}
static size_t call(const char *form, unsigned workers) {
    uint64_t length = UINT64_MAX;
    if (!strcmp(form, "wf")) {
        wf_bench_bfs_par(edges, 4 * count, pull_mode, &output, &length);
        release_output = wf_bench_bfs_par_release;
    } else if (!strcmp(form, "wf-seq")) {
        wf_bench_bfs_seq(edges, 4 * count, pull_mode, &output, &length);
        release_output = wf_bench_bfs_seq_release;
    } else {
        select_backend(form, workers);
        native_entry(edges, 4 * count, pull_mode, &output, &length);
        release_output = native_release;
    }
    if (length != count) fail("timed output length");
    return count;
}
static size_t check(void) {
    size_t checked = compare(output, expected, count);
    release_output(output, count); output = NULL;
    return checked;
}
static size_t verify(const char *form, unsigned workers) {
    if (!strcmp(form, "wf")) return matrix(wf_bench_bfs_par, wf_bench_bfs_par_release);
    if (!strcmp(form, "wf-seq")) return matrix(wf_bench_bfs_seq, wf_bench_bfs_seq_release);
    select_backend(form, workers); return matrix(native_entry, native_release);
}
static void finish(const char *form) { (void)form; free(edges); free(expected); }
static const char *const forms[] = {"wf", "wf-seq", "serial", "tbb", "parlay", "rayon-join", "rayon-iter", NULL};
const wfb_kernel wfb_this_kernel = {
    "bfs", forms, prepare, call, check, verify, finish, workload, 0, 0, NULL
};
