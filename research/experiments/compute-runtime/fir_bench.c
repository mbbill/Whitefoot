/* First FIR cost attribution, not a fastest-implementation certification.
 * All calls return complete results. Verification is outside each interval;
 * allocation, representation conversion and destruction are charged below.
 * Compile this host and the native kernel separately, without LTO or fast math.
 */
#define _POSIX_C_SOURCE 200809L
#define _DARWIN_C_SOURCE 1
#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <time.h>
#include "fir_native.h"
#ifdef WF_FILTER_STATIC
#include "fir_static.h"
#endif

#ifndef FIR_RUNTIME
#error FIR_RUNTIME must name the linked runtime control
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
extern unsigned long wf__par_grants(void);
extern int wf__floor_run(int, char **);
#ifdef WF_COMPUTE_CONTROL
#include "runtime.h"
#endif
#ifdef WF_SHARED_CONTROL
extern int wf__sched_report(char *, size_t);
#endif

static uint64_t entered_at;
#ifdef WF_FILTER_STATIC
static int static_enabled;
static unsigned static_requested;
static FirStatic *static_pool;
static FirStaticInfo static_info;
#endif

static void require(int condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "FIR bench: %s\n", message);
        exit(1);
    }
}

static uint64_t now(void) {
    struct timespec t;
    require(clock_gettime(CLOCK_MONOTONIC_RAW, &t) == 0, "clock_gettime failed");
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}

static size_t number(const char *s, size_t maximum) {
    char *end;
    errno = 0;
    unsigned long long n = strtoull(s, &end, 10);
    require(s[0] >= '0' && s[0] <= '9' && *end == '\0' && errno == 0 && n <= maximum,
            "invalid integer argument");
    return (size_t)n;
}

static double *allocate(size_t n) {
    double *p = malloc((n ? n : 1) * sizeof(*p));
    require(p != NULL, "allocation failed");
    return p;
}

static double sample(uint32_t *state, size_t i) {
    *state = *state * UINT32_C(1664525) + UINT32_C(1013904223);
    int32_t v = (int32_t)(*state & UINT32_C(65535)) - 32768;
    double scale = i % 19 == 0 ? 65536.0 : (i % 19 == 1 ? 0x1p-20 : 1.0);
    return ((double)v / 1009.0) * scale;
}

static uint64_t timeval_us(struct timeval t) {
    return (uint64_t)t.tv_sec * UINT64_C(1000000) + (uint64_t)t.tv_usec;
}

typedef struct {
    uint64_t core, cycle;
} Reading;

/* One complete invocation with a fresh prefix and result. Native output is
 * caller-allocated before core; WF core includes its initialized buffers and
 * owned tree. Both materialize a flat result and next history, then release
 * their temporary storage before cycle ends. This accessor-based WF consumer
 * is the current program's form, not a proposed best possible layout. */
static Reading invoke(FilterEntry wf, FirNativeKernel native,
                      const double *input, const double *history, const double *taps,
                      size_t k, size_t n, size_t tile, double *output, double *state) {
    uint64_t start = now();
    size_t h = k - 1;
    double *prefix = allocate(h + n);
    memcpy(prefix, history, h * sizeof(double));
    memcpy(prefix + h, input, n * sizeof(double));
    double *flat = native ? allocate(n) : NULL;
    uint64_t core_start = now();
    void *tree = NULL;
    if (native) {
#ifdef WF_FILTER_STATIC
        if (static_enabled) {
            /* Lazy initialization charges allocation, actual thread startup
             * and readiness to the first core/cycle, including empty calls. */
            if (static_pool == NULL) {
                static_pool = fir_static_create(static_requested, &static_info);
                if (static_pool == NULL || static_info.actual_lanes != static_requested ||
                    static_info.creation_error != 0) {
                    fprintf(stderr, "FIR bench static startup: requested=%u actual=%u creation_error=%d errno=%d\n",
                            static_requested, static_info.actual_lanes,
                            static_info.creation_error, errno);
                    if (static_pool != NULL) {
                        fir_static_destroy(static_pool);
                        static_pool = NULL;
                    }
                    require(0, "static pool did not start exact requested lanes");
                }
            }
            fir_static_run(static_pool, native, prefix, taps, k, n, flat);
        } else
#endif
        native(prefix, taps, k, n, flat);
    } else
        tree = wf(prefix, h + n, taps, k, h, h + n, h, tile);
    uint64_t core_end = now();
    if (native) {
        memcpy(output, flat, n * sizeof(double));
        memcpy(state, prefix + n, h * sizeof(double));
        free(flat);
    } else {
        require(tree != NULL && wf_research_fir_count(tree) == n, "wrong result count");
        for (size_t i = 0; i < n; ++i)
            require(wf_research_fir_get(tree, i, output + i) == 1, "missing result");
        for (size_t i = 0; i < h; ++i)
            require(wf_research_fir_history(prefix, h + n, h, i, state + i) == 1,
                    "missing next history");
        require(wf_research_fir_release(tree) == 0, "release failed");
    }
    free(prefix);
    uint64_t end = now();
    return (Reading){core_end - core_start, end - start};
}

int wf__main_body(int argc, char **argv) {
    uint64_t body_at = now();
#ifdef WF_FILTER_STATIC
    require(argc == 8, "usage: bench wf|direct|lanes4|lanes8|lanes16|static-direct|static-lanes4|static-lanes8|static-lanes16 K N TILE REPS SEED PASS");
#else
    require(argc == 8, "usage: bench wf|direct|lanes4|lanes8|lanes16 K N TILE REPS SEED PASS");
#endif
    size_t k = number(argv[2], 64);
    size_t n = number(argv[3], 16777216);
    size_t tile = number(argv[4], 65536);
    size_t reps = number(argv[5], 4096);
    uint32_t seed = (uint32_t)number(argv[6], UINT32_MAX);
    size_t pass = number(argv[7], 1000);
    require(k > 0 && tile > 0 && reps > 0 && n <= 16777216 - (k - 1), "out of domain");
    FirNativeKernel native = NULL;
    const char *names[] = {"direct", "lanes4", "lanes8", "lanes16"};
    const FirNativeKernel entries[] = {fir_native_direct, fir_native_lanes4,
                                      fir_native_lanes8, fir_native_lanes16};
    for (size_t i = 0; i < 4; ++i)
        if (strcmp(argv[1], names[i]) == 0) native = entries[i];
#ifdef WF_FILTER_STATIC
    const char *static_names[] = {"static-direct", "static-lanes4", "static-lanes8", "static-lanes16"};
    for (size_t i = 0; i < 4; ++i) {
        if (strcmp(argv[1], static_names[i]) == 0) {
            native = entries[i];
            static_enabled = 1;
        }
    }
#endif
    require(native != NULL || strcmp(argv[1], "wf") == 0, "unknown kernel");
    uint64_t select_at = now();
    int parallel = native ? 0 : wf__par_pool_active();
    FilterEntry wf = parallel ? wf_research_fir_parallel : wf_research_fir_sequential;
    uint64_t select_ns = now() - select_at;
    const char *workers = getenv("WF_WORKERS");
    require(workers != NULL, "set WF_WORKERS explicitly");
    (void)number(workers, 64);
#ifdef WF_FILTER_STATIC
    if (static_enabled) {
        size_t lanes = number(workers, 4);
        require(lanes == 0 || lanes == 1 || lanes == 2 || lanes == 4,
                "static mode requires WF_WORKERS=0,1,2,4");
        static_requested = lanes < 2 ? 1 : (unsigned)lanes;
    }
#endif

    size_t h = k - 1;
    double *input = allocate(n), *history = allocate(h), *taps = allocate(k);
    double *expected = allocate(n), *expected_state = allocate(h);
    double *output = allocate(n), *state = allocate(h), *prefix = allocate(n + h);
    uint32_t random = seed;
    for (size_t i = 0; i < n; ++i) input[i] = sample(&random, i);
    for (size_t i = 0; i < h; ++i) history[i] = sample(&random, i + 3);
    for (size_t i = 0; i < k; ++i) taps[i] = sample(&random, i + 7) / 65536.0;
    memcpy(prefix, history, h * sizeof(double));
    memcpy(prefix + h, input, n * sizeof(double));
    /* check-native independently qualifies direct against the delay-line
     * oracle. That qualified kernel supplies this screen's expectations,
     * including the direct configuration; its timed self-comparison alone
     * would not qualify correctness. */
    fir_native_direct(prefix, taps, k, n, expected);
    memcpy(expected_state, prefix + n, h * sizeof(double));
    free(prefix);
    Reading *readings = calloc(reps + 1, sizeof(*readings));
    require(readings != NULL, "reading allocation failed");
    uint64_t clock_min = UINT64_MAX;
    uint64_t clock_positive_min = UINT64_MAX;
    struct timespec resolution;
    require(clock_getres(CLOCK_MONOTONIC_RAW, &resolution) == 0, "clock_getres failed");
    for (unsigned i = 0; i < 1000; ++i) {
        uint64_t a = now(), b = now();
        if (b - a < clock_min) clock_min = b - a;
        if (b > a && b - a < clock_positive_min) clock_positive_min = b - a;
    }
    struct rusage before, after;
    require(getrusage(RUSAGE_SELF, &before) == 0, "getrusage failed");
    uint64_t batch_start = now();
    /* Call zero is retained as first invocation, then REPS warm invocations.
     * Checks warm the output/cache between calls and are never subtracted from
     * batch CPU or wall usage. No log output runs inside the measured batch. */
    for (size_t i = 0; i <= reps; ++i) {
        readings[i] = invoke(wf, native, input, history, taps, k, n, tile, output, state);
        require(memcmp(output, expected, n * sizeof(double)) == 0, "wrong output bits");
        require(memcmp(state, expected_state, h * sizeof(double)) == 0, "wrong history bits");
    }
    uint64_t batch_ns = now() - batch_start;
    require(getrusage(RUSAGE_SELF, &after) == 0, "getrusage failed");
    printf("# runtime=%s kernel=%s workers_requested=%s world=%s entry_ns=%" PRIu64
           " select_ns=%" PRIu64 " clock_pair_min_ns=%" PRIu64 "\n",
           FIR_RUNTIME, argv[1], workers,
#ifdef WF_FILTER_STATIC
           static_enabled ? "static" :
#endif
           parallel ? "parallel" : "sequential",
           body_at - entered_at, select_ns, clock_min);
    printf("# clock=CLOCK_MONOTONIC_RAW reported_resolution_ns=%" PRIu64
           " clock_pair_positive_min_ns=%" PRIu64 "\n",
           (uint64_t)resolution.tv_sec * UINT64_C(1000000000) + (uint64_t)resolution.tv_nsec,
           clock_positive_min == UINT64_MAX ? 0 : clock_positive_min);
    uint64_t rss = (uint64_t)after.ru_maxrss;
#ifndef __APPLE__
    rss *= 1024;
#endif
    printf("# batch_includes_checks=1 batch_ns=%" PRIu64 " user_us=%" PRIu64
           " system_us=%" PRIu64 " maxrss_bytes=%" PRIu64
           " voluntary_switches=%ld involuntary_switches=%ld\n",
           batch_ns, timeval_us(after.ru_utime) - timeval_us(before.ru_utime),
           timeval_us(after.ru_stime) - timeval_us(before.ru_stime), rss,
           after.ru_nvcsw - before.ru_nvcsw, after.ru_nivcsw - before.ru_nivcsw);
#ifdef WF_COMPUTE_CONTROL
    printf("# actual_lanes=%u steals=%lu\n", wf_compute_worker_count(), wf__par_grants());
#endif
#ifdef WF_SHARED_CONTROL
    char report[1024];
    if (wf__sched_report(report, sizeof(report))) printf("# %s\n", report);
    else puts("# shared_pool_report=not_initialized");
#endif
    puts("runtime\tkernel\tworkers\tk\tn\ttile\tseed\tpass\tcall\tphase\tcore_ns\tcycle_ns");
    for (size_t i = 0; i <= reps; ++i)
        printf("%s\t%s\t%s\t%zu\t%zu\t%zu\t%" PRIu32 "\t%zu\t%zu\t%s\t%" PRIu64
               "\t%" PRIu64 "\n", FIR_RUNTIME, argv[1], workers, k, n, tile, seed, pass,
               i, i == 0 ? "first" : "warm", readings[i].core, readings[i].cycle);
    printf("# FIR bench PASS: calls=%zu samples=%zu history=%zu\n",
           reps + 1, (reps + 1) * n, (reps + 1) * h);
#ifdef WF_FILTER_STATIC
    if (static_enabled) {
        /* Pool shutdown is outside the batch snapshot and sample output. It
         * is reported separately; entry/first-call is not total cold process. */
        require(fflush(stdout) == 0, "output flush failed");
        uint64_t shutdown_start = now();
        fir_static_destroy(static_pool);
        uint64_t shutdown_ns = now() - shutdown_start;
        static_pool = NULL;
        printf("# static_requested_lanes=%u static_actual_lanes=%u static_helpers=%u"
               " static_creation_error=%d static_idle=condvar static_spin=0"
               " static_startup_in_first_core=1 static_shutdown_outside_batch=1"
               " static_shutdown_ns=%" PRIu64 "\n", static_info.requested_lanes,
               static_info.actual_lanes, static_info.actual_lanes - 1,
               static_info.creation_error, shutdown_ns);
    }
#endif
    free(readings);
    free(input); free(history); free(taps); free(expected); free(expected_state);
    free(output); free(state);
    return 0;
}

int main(int argc, char **argv) {
    entered_at = now();
    return wf__floor_run(argc, argv);
}
