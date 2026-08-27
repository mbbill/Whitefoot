/* glibc hides the POSIX and Linux entry points these files call when the
 * translation unit is compiled as strict C11, so the namespace is asked for
 * before any header. Harmless where it is not needed. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/* The N line of io-completion-bench: the best hand-written native shape for
 * the many-independent-files workload on this platform.
 *
 * Every mode opens each generated file by name, transfers it with one
 * positioned read, folds the shared checksum from `workload.h`, and prints
 * exactly the line the Whitefoot programs print. Modes:
 *
 *   direct N           one thread, blocking openat + pread, no pool at all
 *   pool N THREADS     a pthread pool over a striped index range
 *   uring N DEPTH      Linux only: raw io_uring IORING_OP_READ at DEPTH in
 *                      flight, with openat and close on the submitting thread,
 *                      which is the operation split the Whitefoot Linux
 *                      adapter itself uses
 *
 * The pool is sized by the caller so the harness can report the best fair
 * size rather than a size chosen to flatter either side. */
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "workload.h"

#if defined(__linux__)
#include <linux/fs.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#define WF_BENCH_HAS_URING 1
#include "uring_baseline.h"
#endif

static uint64_t fold_one(int directory, uint64_t index, unsigned char *window) {
    char name[32];
    snprintf(name, sizeof name, WF_BENCH_NAME_FORMAT, (unsigned long)index);
    int descriptor = openat(directory, name, O_RDONLY);
    if (descriptor < 0) {
        return 0;
    }
    ssize_t moved = pread(descriptor, window, WF_BENCH_READ_WINDOW, 0);
    close(descriptor);
    if (moved <= 0) {
        return 0;
    }
    return wf_bench_weighted(wf_bench_digest(window, (size_t)moved), index);
}

struct lane {
    pthread_t thread;
    int directory;
    uint64_t start;
    uint64_t count;
    uint64_t stride;
    uint64_t sum;
    uint64_t bytes;
};

static uint64_t fold_one_counting(int directory, uint64_t index, unsigned char *window,
                                  uint64_t *bytes) {
    char name[32];
    snprintf(name, sizeof name, WF_BENCH_NAME_FORMAT, (unsigned long)index);
    int descriptor = openat(directory, name, O_RDONLY);
    if (descriptor < 0) {
        return 0;
    }
    ssize_t moved = pread(descriptor, window, WF_BENCH_READ_WINDOW, 0);
    close(descriptor);
    if (moved <= 0) {
        return 0;
    }
    *bytes += (uint64_t)moved;
    return wf_bench_weighted(wf_bench_digest(window, (size_t)moved), index);
}

static void *lane_main(void *raw) {
    struct lane *lane = raw;
    unsigned char *window = malloc(WF_BENCH_READ_WINDOW);
    if (window == NULL) {
        return NULL;
    }
    for (uint64_t index = lane->start; index < lane->count; index += lane->stride) {
        lane->sum += fold_one_counting(lane->directory, index, window, &lane->bytes);
    }
    free(window);
    return NULL;
}

static int run_direct(int directory, uint64_t count, uint64_t *sum, uint64_t *bytes) {
    unsigned char *window = malloc(WF_BENCH_READ_WINDOW);
    if (window == NULL) {
        return 1;
    }
    for (uint64_t index = 0; index < count; index++) {
        *sum += fold_one_counting(directory, index, window, bytes);
    }
    free(window);
    (void)fold_one;
    return 0;
}

static int run_pool(int directory, uint64_t count, unsigned long threads, uint64_t *sum,
                    uint64_t *bytes) {
    struct lane *lanes = calloc(threads, sizeof *lanes);
    if (lanes == NULL) {
        return 1;
    }
    for (unsigned long at = 0; at < threads; at++) {
        lanes[at].directory = directory;
        lanes[at].start = at;
        lanes[at].count = count;
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

int main(int argc, char **argv) {
    if (argc < 4) {
        fprintf(stderr, "usage: baseline ROOT MODE COUNT [PARAMETER]\n");
        return 2;
    }
    const char *root = argv[1];
    const char *mode = argv[2];
    uint64_t count = strtoull(argv[3], NULL, 10);
    unsigned long parameter = argc > 4 ? strtoul(argv[4], NULL, 10) : 0;
    int directory = open(root, O_RDONLY | O_DIRECTORY);
    if (directory < 0) {
        perror("baseline: open root");
        return 1;
    }
    uint64_t sum = 0;
    uint64_t bytes = 0;
    int failure;
    if (strcmp(mode, "direct") == 0) {
        failure = run_direct(directory, count, &sum, &bytes);
    } else if (strcmp(mode, "pool") == 0) {
        if (parameter == 0) {
            fprintf(stderr, "baseline: pool needs a positive thread count\n");
            close(directory);
            return 2;
        }
        failure = run_pool(directory, count, parameter, &sum, &bytes);
#if defined(WF_BENCH_HAS_URING)
    } else if (strcmp(mode, "uring") == 0) {
        if (parameter == 0) {
            fprintf(stderr, "baseline: uring needs a positive depth\n");
            close(directory);
            return 2;
        }
        failure = wf_bench_run_uring(directory, count, parameter, &sum, &bytes);
#endif
    } else {
        fprintf(stderr, "baseline: unknown mode %s\n", mode);
        close(directory);
        return 2;
    }
    close(directory);
    if (failure != 0) {
        fprintf(stderr, "baseline: %s failed\n", mode);
        return 1;
    }
    printf("%020llu %020llu\n", (unsigned long long)sum, (unsigned long long)bytes);
    return 0;
}
