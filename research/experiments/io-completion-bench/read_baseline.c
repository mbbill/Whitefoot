/* glibc hides the POSIX and Linux entry points these files call when the
 * translation unit is compiled as strict C11, so the namespace is asked for
 * before any header. Harmless where it is not needed. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/* The N line of the read-heavy io-completion-bench workload.
 *
 * The many-files N line in baseline.c opens a file per unit of work. This one
 * opens the eight generated files once, before any read, and then performs
 * READS positioned reads at the schedule `workload.h` defines, folding the
 * same position-weighted checksum the Whitefoot programs fold. What separates
 * its modes is only how many of those reads are in flight at once:
 *
 *   direct ROOT READS WINDOW           one thread, blocking pread
 *   pool   ROOT READS WINDOW THREADS   a pthread pool striped over the list
 *   uring  ROOT READS WINDOW DEPTH     Linux only: DEPTH IORING_OP_READ in
 *                                      flight over fixed descriptors
 *
 * WF_IO_NOCACHE=1 applies the same host call to the same descriptors that the
 * Whitefoot runtime's target policy applies, so N and C wait on the same
 * device rather than on two different cache states.
 *
 * Two further modes report no checksum and time nothing but the page cache:
 *
 *   warm           ROOT READS WINDOW           read the whole tree in, so a
 *                                                warm table has the state it
 *                                                says it has
 *   probe-uncached ROOT PROBES WINDOW MIN_US [TOLERANCE_PERCENT]
 *   probe-warm     ROOT PROBES WINDOW MAX_US [TOLERANCE_PERCENT]
 *
 * Each times PROBES positioned reads in each file and fails unless all but
 * TOLERANCE_PERCENT of them fell on the stated side of the threshold. The
 * bundle runs one immediately before and one immediately after every table,
 * so a table's cache-state label is checked rather than assumed. */
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "workload.h"

#if defined(__linux__)
#include <linux/fs.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#define WF_BENCH_HAS_URING 1
#include "uring_baseline.h"
#endif

/* The eight descriptors every mode reads through, opened once. */
struct wf_read_tree {
    int descriptor[WF_BENCH_BIG_FILES];
    uint64_t reads;
    uint64_t window;
    uint64_t blocks;
};

/* How a mode wants its descriptors opened.
 *
 * A measured line takes the target policy the run asked for. A probe takes a
 * descriptor that does not populate the cache, so it can observe residency
 * without creating it. The warm step takes a plain descriptor, because its
 * whole job is to make the tree resident. */
enum wf_read_open_policy {
    WF_READ_OPEN_MEASURED = 0,
    WF_READ_OPEN_PROBE = 1,
    WF_READ_OPEN_WARM = 2
};

static int wf_read_tree_open(struct wf_read_tree *tree, const char *root,
                             enum wf_read_open_policy policy) {
    char path[4096];
    for (unsigned at = 0; at < WF_BENCH_BIG_FILES; at++) {
        tree->descriptor[at] = -1;
    }
    for (unsigned at = 0; at < WF_BENCH_BIG_FILES; at++) {
        int written =
            snprintf(path, sizeof path, "%s/" WF_BENCH_NAME_FORMAT, root, (unsigned long)at);
        if (written < 0 || (size_t)written >= sizeof path) {
            return 1;
        }
        tree->descriptor[at] = open(path, O_RDONLY);
        if (tree->descriptor[at] < 0) {
            perror("read_baseline: open");
            return 1;
        }
        if (policy == WF_READ_OPEN_PROBE) {
            wf_bench_no_populate(tree->descriptor[at]);
        } else if (policy == WF_READ_OPEN_MEASURED) {
            wf_bench_apply_uncached(tree->descriptor[at]);
        }
    }
    return 0;
}

static void wf_read_tree_close(struct wf_read_tree *tree) {
    for (unsigned at = 0; at < WF_BENCH_BIG_FILES; at++) {
        if (tree->descriptor[at] >= 0) {
            (void)close(tree->descriptor[at]);
        }
    }
}

/* One read of the schedule, folded exactly as every other line folds it. */
static uint64_t fold_one_read(const struct wf_read_tree *tree, uint64_t position,
                              unsigned char *window, uint64_t *bytes) {
    int descriptor = tree->descriptor[wf_bench_read_file(position)];
    uint64_t offset = wf_bench_read_block(position, tree->blocks) * tree->window;
    ssize_t moved = pread(descriptor, window, (size_t)tree->window, (off_t)offset);
    if (moved <= 0) {
        return 0;
    }
    *bytes += (uint64_t)moved;
    return wf_bench_weighted(
        wf_bench_digest(window, wf_bench_check_length(tree->window, (uint64_t)moved)), position);
}

struct lane {
    pthread_t thread;
    const struct wf_read_tree *tree;
    uint64_t start;
    uint64_t stride;
    uint64_t sum;
    uint64_t bytes;
};

static void *lane_main(void *raw) {
    struct lane *lane = raw;
    unsigned char *window = malloc((size_t)lane->tree->window);
    if (window == NULL) {
        return NULL;
    }
    for (uint64_t at = lane->start; at < lane->tree->reads; at += lane->stride) {
        lane->sum += fold_one_read(lane->tree, at, window, &lane->bytes);
    }
    free(window);
    return NULL;
}

static int run_direct(const struct wf_read_tree *tree, uint64_t *sum, uint64_t *bytes) {
    unsigned char *window = malloc((size_t)tree->window);
    if (window == NULL) {
        return 1;
    }
    for (uint64_t at = 0; at < tree->reads; at++) {
        *sum += fold_one_read(tree, at, window, bytes);
    }
    free(window);
    return 0;
}

static int run_pool(const struct wf_read_tree *tree, unsigned long threads, uint64_t *sum,
                    uint64_t *bytes) {
    struct lane *lanes = calloc(threads, sizeof *lanes);
    if (lanes == NULL) {
        return 1;
    }
    for (unsigned long at = 0; at < threads; at++) {
        lanes[at].tree = tree;
        lanes[at].start = at;
        lanes[at].stride = threads;
        if (pthread_create(&lanes[at].thread, NULL, lane_main, &lanes[at]) != 0) {
            free(lanes);
            return 1;
        }
    }
    for (unsigned long at = 0; at < threads; at++) {
        pthread_join(lanes[at].thread, NULL);
        *sum += lanes[at].sum;
        *bytes += lanes[at].bytes;
    }
    free(lanes);
    return 0;
}

/* Reads every block of every file through plain descriptors, so the page cache
 * holds the tree. The warm tables need a state the uncached tables destroy,
 * and 32,768 pseudo-random reads would leave about two per cent of the blocks
 * untouched, so the warm step is a full sequential pass rather than a rerun of
 * the workload. */
static int run_warm(const struct wf_read_tree *tree) {
    unsigned char *window = malloc((size_t)tree->window);
    if (window == NULL) {
        return 1;
    }
    for (unsigned file = 0; file < WF_BENCH_BIG_FILES; file++) {
        for (uint64_t block = 0; block < tree->blocks; block++) {
            ssize_t moved = pread(tree->descriptor[file], window, (size_t)tree->window,
                                  (off_t)(block * tree->window));
            if (moved != (ssize_t)tree->window) {
                free(window);
                return 1;
            }
        }
    }
    free(window);
    return 0;
}

/* The residency probe that gives the words "uncached" and "warm" their
 * meaning.
 *
 * A table may only carry a cache-state label if that state was checked, and
 * the cheapest honest evidence is what one read costs. This mode times PROBES
 * positioned reads in each of the eight files, through descriptors that do
 * not populate the cache, and reports every file's median. `probe-uncached`
 * then demands that the sampled reads be device reads and `probe-warm` that
 * they be cache hits, so whichever label the table carries was checked
 * immediately before and immediately after it ran rather than inferred from
 * the order the tables were taken in.
 *
 * The threshold separates two populations this host keeps far apart: a 64 KiB
 * read served from the unified buffer cache costs 6 to 20 us, and one the
 * device answers costs about 134 us.
 *
 * The rule is a bound on the share of sampled reads that fell on the wrong
 * side, not a per-file all-or-nothing test, and the difference is a measured
 * one. With no benchmark process running at all -- only this probe and a CPU
 * loop between passes -- one of the eight files on this host intermittently
 * becomes half resident for a minute or so and then is reclaimed again, so
 * something outside the benchmark reads the tree. One file half resident is
 * about six per cent of the tree, which shifts every line of a table by the
 * same six per cent and cannot move a ratio; a whole file resident is twelve
 * and a half per cent and is refused. TOLERANCE_PERCENT is where that line is
 * drawn, and the per-file medians are printed either way so a reader can see
 * exactly what the tree looked like rather than take the verdict on trust. */
static int compare_double_probe(const void *left, const void *right) {
    double a = *(const double *)left;
    double b = *(const double *)right;
    return (a > b) - (a < b);
}

static int run_probe(const struct wf_read_tree *tree, double threshold_us, int want_uncached,
                     double tolerance_percent) {
    unsigned long probes = (unsigned long)tree->reads;
    double *samples = malloc(probes * sizeof *samples);
    unsigned char *window = malloc((size_t)tree->window);
    /* Fresh offsets on every invocation. A probe that always read the same
     * blocks would, over the eight probes a run needs, become a measurement
     * of the blocks the previous probe touched rather than of the tree. */
    uint64_t salt = wf_bench_mix((uint64_t)getpid() * 2654435761ULL +
                                 (uint64_t)time(NULL) * 40503ULL + 1ULL);
    unsigned long wrong_side = 0;
    unsigned long total = 0;
    double slowest = 0.0;
    double fastest = 0.0;
    if (samples == NULL || window == NULL) {
        free(samples);
        free(window);
        fprintf(stderr, "read_baseline: probe out of memory\n");
        return 1;
    }
    for (unsigned file = 0; file < WF_BENCH_BIG_FILES; file++) {
        for (unsigned long at = 0; at < probes; at++) {
            uint64_t seed = salt + (uint64_t)file * 1000003ULL + (uint64_t)at + 1ULL;
            uint64_t offset = wf_bench_read_block(seed, tree->blocks) * tree->window;
            struct timespec start;
            struct timespec end;
            clock_gettime(CLOCK_MONOTONIC, &start);
            ssize_t moved = pread(tree->descriptor[file], window, (size_t)tree->window,
                                  (off_t)offset);
            clock_gettime(CLOCK_MONOTONIC, &end);
            if (moved != (ssize_t)tree->window) {
                fprintf(stderr, "read_baseline: probe read of file %u was short\n", file);
                free(samples);
                free(window);
                return 1;
            }
            samples[at] = ((double)(end.tv_sec - start.tv_sec) * 1000000.0) +
                          ((double)(end.tv_nsec - start.tv_nsec) / 1000.0);
            total++;
            if (want_uncached ? samples[at] <= threshold_us : samples[at] >= threshold_us) {
                wrong_side++;
            }
        }
        qsort(samples, probes, sizeof *samples, compare_double_probe);
        double middle = probes % 2 == 1
                            ? samples[probes / 2]
                            : (samples[probes / 2 - 1] + samples[probes / 2]) / 2.0;
        printf("probe " WF_BENCH_NAME_FORMAT " median %8.1f us over %lu reads of %llu B\n",
               (unsigned long)file, middle, probes, (unsigned long long)tree->window);
        if (file == 0 || middle > slowest) {
            slowest = middle;
        }
        if (file == 0 || middle < fastest) {
            fastest = middle;
        }
    }
    free(samples);
    free(window);
    double share = total == 0 ? 100.0 : (double)wrong_side * 100.0 / (double)total;
    const char *label = want_uncached ? "uncached" : "warm";
    const char *side = want_uncached ? "at or below" : "at or above";
    if (share > tolerance_percent) {
        fprintf(stderr,
                "read_baseline: refusing the %s label -- %lu of %lu sampled reads (%.1f%%) were "
                "%s %.1f us, past the stated %.1f%% bound; per-file medians %.1f..%.1f us\n",
                label, wrong_side, total, share, side, threshold_us, tolerance_percent, fastest,
                slowest);
        return 1;
    }
    printf("probe: %s confirmed -- %lu of %lu sampled reads (%.1f%%) were %s %.1f us, within the "
           "stated %.1f%% bound; per-file medians %.1f..%.1f us\n",
           label, wrong_side, total, share, side, threshold_us, tolerance_percent, fastest,
           slowest);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 5) {
        fprintf(stderr, "usage: read_baseline ROOT MODE READS WINDOW [PARAMETER]\n  MODE is direct, pool, uring, probe-uncached, or probe-warm\n");
        return 2;
    }
    const char *root = argv[1];
    const char *mode = argv[2];
    struct wf_read_tree tree;
    tree.reads = strtoull(argv[3], NULL, 10);
    tree.window = strtoull(argv[4], NULL, 10);
    unsigned long parameter = argc > 5 ? strtoul(argv[5], NULL, 10) : 0;
    if (tree.reads == 0 || tree.window == 0 || WF_BENCH_BIG_BYTES % tree.window != 0) {
        fprintf(stderr, "read_baseline: READS must be positive and WINDOW must divide the file\n");
        return 2;
    }
    tree.blocks = WF_BENCH_BIG_BYTES / tree.window;
    int probing = strncmp(mode, "probe", 5) == 0;
    int warming = strcmp(mode, "warm") == 0;
    enum wf_read_open_policy policy = probing ? WF_READ_OPEN_PROBE
                                    : warming ? WF_READ_OPEN_WARM
                                              : WF_READ_OPEN_MEASURED;
    if (wf_read_tree_open(&tree, root, policy) != 0) {
        wf_read_tree_close(&tree);
        return 1;
    }
    if (warming) {
        int failure = run_warm(&tree);
        wf_read_tree_close(&tree);
        if (failure != 0) {
            fprintf(stderr, "read_baseline: warm failed\n");
            return 1;
        }
        printf("warm: the whole tree has been read through plain descriptors\n");
        return 0;
    }
    if (probing) {
        int want_uncached = strcmp(mode, "probe-uncached") == 0;
        if (!want_uncached && strcmp(mode, "probe-warm") != 0) {
            fprintf(stderr, "read_baseline: unknown mode %s\n", mode);
            wf_read_tree_close(&tree);
            return 2;
        }
        if (parameter == 0) {
            fprintf(stderr, "read_baseline: a probe needs a positive threshold in us\n");
            wf_read_tree_close(&tree);
            return 2;
        }
        double tolerance = argc > 6 ? strtod(argv[6], NULL) : 10.0;
        int refused = run_probe(&tree, (double)parameter, want_uncached, tolerance);
        wf_read_tree_close(&tree);
        return refused == 0 ? 0 : 1;
    }
    uint64_t sum = 0;
    uint64_t bytes = 0;
    int failure;
    if (strcmp(mode, "direct") == 0) {
        failure = run_direct(&tree, &sum, &bytes);
    } else if (strcmp(mode, "pool") == 0) {
        if (parameter == 0) {
            fprintf(stderr, "read_baseline: pool needs a positive thread count\n");
            wf_read_tree_close(&tree);
            return 2;
        }
        failure = run_pool(&tree, parameter, &sum, &bytes);
#if defined(WF_BENCH_HAS_URING)
    } else if (strcmp(mode, "uring") == 0) {
        if (parameter == 0) {
            fprintf(stderr, "read_baseline: uring needs a positive depth\n");
            wf_read_tree_close(&tree);
            return 2;
        }
        failure = wf_bench_run_read_uring(tree.descriptor, tree.reads, tree.window, tree.blocks,
                                          parameter, &sum, &bytes);
#endif
    } else {
        fprintf(stderr, "read_baseline: unknown mode %s\n", mode);
        wf_read_tree_close(&tree);
        return 2;
    }
    wf_read_tree_close(&tree);
    if (failure != 0) {
        fprintf(stderr, "read_baseline: %s failed\n", mode);
        return 1;
    }
    printf("%020llu %020llu\n", (unsigned long long)sum, (unsigned long long)bytes);
    return 0;
}
