/* Sparse FIFO oracle, bounded-degree graph fixtures, and dense pull control.
 * The WF sparse algorithm uses intrusive level lists rather than this queue.
 * All fixtures are undirected: outgoing and incoming adjacency are identical. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void (*bfs_entry)(const uint64_t *, uint64_t, uint64_t, uint64_t **, uint64_t *);
typedef void (*bfs_release)(uint64_t *, uint64_t);
static void fail(const char *message) {
    (void)fprintf(stderr, "bfs: %s\n", message); exit(2);
}
static uint64_t *allocate(size_t n) {
    uint64_t *p = malloc((n ? n : 1) * sizeof(*p));
    if (!p) fail("allocation failed");
    return p;
}
static uint64_t *graph(size_t n, unsigned shape) {
    uint64_t *edges = allocate(4 * n);
    for (size_t i = 0; i < 4 * n; ++i) edges[i] = n;
    for (size_t v = 0; v < n; ++v) {
        uint64_t *row = edges + 4 * v;
        if (shape == 1) {
            if (v) row[0] = (v - 1) / 2;
            if (2 * v + 1 < n) row[1] = 2 * v + 1;
            if (2 * v + 2 < n) row[2] = 2 * v + 2;
        } else if (shape == 4) {
            if (v % 31) row[0] = v - 1;
            if (v % 31 < 30 && v + 1 < n) row[1] = v + 1;
            if (v >= 31) row[2] = v - 31;
            if (v + 31 < n) row[3] = v + 31;
        } else {
            if (v && !(shape == 2 && v == n / 2)) row[0] = v - 1;
            if (v + 1 < n && !(shape == 2 && v + 1 == n / 2)) row[1] = v + 1;
            if (shape == 3 && n > 2) {
                if (!v) row[0] = n - 1;
                if (v + 1 == n) row[1] = 0;
            }
            if (shape == 5) { row[2] = row[0]; row[3] = row[1]; }
        }
    }
    return edges;
}
typedef struct { size_t reached, levels, slots; } work_count;
static uint64_t *oracle(const uint64_t *edges, size_t n, work_count *work) {
    uint64_t *distance = allocate(n), *queue = allocate(n);
    for (size_t v = 0; v < n; ++v) distance[v] = n;
    size_t head = 0, tail = 0, levels = 0;
    if (n) { queue[tail++] = 0; distance[0] = 0; }
    while (head < tail) {
        size_t v = queue[head++];
        if (distance[v] + 1 > levels) levels = distance[v] + 1;
        for (size_t slot = 0; slot < 4; ++slot) {
            uint64_t neighbor = edges[4 * v + slot];
            if (neighbor < n && distance[neighbor] == n) {
                distance[neighbor] = distance[v] + 1;
                queue[tail++] = neighbor;
            }
        }
    }
    if (work) *work = (work_count){tail, levels, 4 * tail};
    free(queue); return distance;
}
static size_t compare(const uint64_t *actual, const uint64_t *expected, size_t n) {
    for (size_t v = 0; v < n; ++v) if (actual[v] != expected[v]) fail("distance mismatch");
    return n;
}
static size_t matrix(bfs_entry entry, bfs_release release) {
    static const size_t counts[] = {0, 1, 2, 17, 63, 257, 2049, 65535};
    size_t checked = 0;
    for (size_t i = 0; i < sizeof(counts) / sizeof(counts[0]); ++i) {
        size_t n = counts[i];
        for (unsigned shape = 0; shape < 6; ++shape) {
            if (n == 65535 && shape != 1) continue;
            uint64_t *edges = graph(n, shape), *original = allocate(4 * n);
            memcpy(original, edges, 4 * n * sizeof(*edges));
            uint64_t *expected = oracle(edges, n, NULL);
            for (unsigned pull = 0; pull < 2; ++pull) {
                uint64_t *result = NULL, length = UINT64_MAX;
                entry(edges, 4 * n, pull, &result, &length);
                if (length != n) fail("output length");
                checked += compare(result, expected, n);
                checked += compare(edges, original, 4 * n);
                release(result, length);
            }
            free(expected); free(original); free(edges);
        }
    }
    return checked;
}

#ifdef WFB_BFS_ORACLE
extern void wf_bench_bfs(const uint64_t *, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_bfs_release(uint64_t *, uint64_t);
extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
#endif
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    size_t checked = matrix(wf_bench_bfs, wf_bench_bfs_release);
#ifdef WFB_ORACLE_PARALLEL
    const char *workers = getenv("WF_WORKERS");
    if (workers && atoi(workers) > 1 &&
        (!wf__par_pool_active() || wf__par_grants() == 0)) fail("oracle did not exercise a worker pool");
    if (workers && atoi(workers) == 1 && wf__par_grants() != 0) fail("pool-off oracle handed out work");
#endif
    (void)printf("bfs oracle PASS: 86 configurations, %zu values\n", checked);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
#else
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
#endif
