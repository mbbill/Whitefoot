#include "kernels.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* splitmix64 — same PRNG discipline as the M3/M6/M8 dry runs. */
static uint64_t sm_state;
static uint64_t sm_next(void) {
    sm_state += 0x9E3779B97F4A7C15ULL;
    uint64_t z = sm_state;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

/* Sattolo: a single-cycle permutation of [0,n). next[i] = perm[i] then visits
   all n nodes in one Hamiltonian cycle (the standard pointer-chase structure). */
static void sattolo(uint32_t *perm, size_t n) {
    for (size_t i = 0; i < n; i++) perm[i] = (uint32_t)i;
    for (size_t i = n - 1; i >= 1; i--) {
        size_t j = (size_t)(sm_next() % i);   /* 0 <= j < i */
        uint32_t t = perm[i]; perm[i] = perm[j]; perm[j] = t;
    }
}

static double now_ns(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e9 + (double)ts.tv_nsec;
}

static int cmp_d(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    return (x > y) - (x < y);
}
static double median(double *v, int n) { qsort(v, n, sizeof(double), cmp_d); return v[n/2]; }

static volatile uint64_t g_sink;

int main(void) {
    printf("# M4 arena branded-id deref dry run\n");
    printf("# Apple M4 / macOS arm64; system cc; indicative (deploy target = Linux x86-64)\n");
    printf("# Node = 16 bytes; random single-cycle (Sattolo) chain; steps = node count.\n");
    printf("# M4 cache: L1d 64KB, L2 4MB. 4K nodes=64KB (L1), 64K=1MB (L2), 1M=16MB (DRAM).\n\n");

    size_t sizes[] = { 4096, 65536, 1000000, 4000000 };
    const char *labels[] = { "4K   (64KB, L1)", "64K  (1MB, L2)",
                             "1M   (16MB, L2/DRAM)", "4M   (64MB, DRAM)" };

    printf("%-20s  %10s %10s %10s   %8s %8s %8s\n",
           "residency", "(a)free", "(b)chk", "(c)ptr", "a/c", "b/c", "b/a");

    for (int s = 0; s < 4; s++) {
        size_t n = sizes[s];
        int runs = 21;
        /* Loop the cycle so each timed run does ~4M node-visits: steady-state
           cache-resident (small n) or DRAM (large n) latency, timer jitter tiny. */
        size_t repeat = (4000000u / n); if (repeat < 1) repeat = 1;
        size_t steps = n * repeat;

        Node *slab = malloc(n * sizeof(Node));
        PNode *pnodes = malloc(n * sizeof(PNode));
        uint32_t *perm = malloc(n * sizeof(uint32_t));
        if (!slab || !pnodes || !perm) { fprintf(stderr, "OOM\n"); return 1; }

        sm_state = 0xA4E4A0000000001ULL + s;   /* distinct but deterministic per size */
        sattolo(perm, n);
        for (size_t i = 0; i < n; i++) {
            uint64_t v = sm_next();
            slab[i].val = v; slab[i].next_idx = perm[i]; slab[i]._pad = 0;
            pnodes[i].val = v; pnodes[i].next = &pnodes[perm[i]];
        }

        /* differential: (a), (b), (c) must sum identically over one full cycle */
        uint64_t sa = walk_checkfree(slab, 0, n);
        uint64_t sb = walk_checked(slab, n, 0, n);
        uint64_t sc = walk_pointer(&pnodes[0], n);
        if (!(sa == sb && sb == sc)) {
            printf("DIFFERENTIAL FAIL at %s: a=%llu b=%llu c=%llu\n", labels[s],
                   (unsigned long long)sa, (unsigned long long)sb, (unsigned long long)sc);
            return 2;
        }

        double ta[32], tb[32], tc[32];
        /* Measure each variant's runs CONSECUTIVELY (not interleaved): interleaving
           a/b/c contaminates the memory-latency signal via shared-cycle residency. */
        for (int w = 0; w < 3; w++) g_sink += walk_checkfree(slab, 0, steps);
        for (int r = 0; r < runs; r++) {
            double t0 = now_ns(); g_sink += walk_checkfree(slab, 0, steps); ta[r] = (now_ns()-t0)/(double)steps;
        }
        for (int w = 0; w < 3; w++) g_sink += walk_checked(slab, n, 0, steps);
        for (int r = 0; r < runs; r++) {
            double t0 = now_ns(); g_sink += walk_checked(slab, n, 0, steps); tb[r] = (now_ns()-t0)/(double)steps;
        }
        for (int w = 0; w < 3; w++) g_sink += walk_pointer(&pnodes[0], steps);
        for (int r = 0; r < runs; r++) {
            double t0 = now_ns(); g_sink += walk_pointer(&pnodes[0], steps); tc[r] = (now_ns()-t0)/(double)steps;
        }
        double ma = median(ta, runs), mb = median(tb, runs), mc = median(tc, runs);
        printf("%-20s  %9.3fns %9.3fns %9.3fns   %7.3fx %7.3fx %7.3fx\n",
               labels[s], ma, mb, mc, ma/mc, mb/mc, mb/ma);

        free(slab); free(pnodes); free(perm);
    }

    printf("\n# Bands: (a)/(c) in [0.9,1.1]; (b)/(c) <= 1.25x on the L1/cache-hot case.\n");
    printf("# (b)/(a) is the check cost; DIFFERENTIAL OK (all sums equal at every size).\n");
    (void)g_sink;
    return 0;
}
