/* The native controls for the dossier's §9.1 initialization-cost row.
 *
 * `drain.wf` reads a file through one caller-owned buffer that the language
 * initializes when it is allocated. The row asks whether paying for that
 * one-time fill is material to steady-state throughput, and says the control
 * must be "an equivalent native read loop over *uninitialized* storage".
 *
 * This file is that control, and also its own initialized twin. One source,
 * one loop, one changed allocation call:
 *
 *     control drain malloc <path>    uninitialized storage — the §9.1 control
 *     control drain calloc <path>    initialized storage — the same-source twin
 *     control fill  malloc <reps>    the allocation alone, no drain
 *     control fill  calloc <reps>    the allocation and its fill, no drain
 *
 * The `drain` modes exist to answer the row across languages. The `fill` modes
 * exist because the cross-language comparison cannot resolve the effect: a
 * one-time fill of one page is far below the noise of any whole-process
 * timing, so measuring it directly is the only way to make the stop condition
 * determinate rather than merely unrefuted.
 *
 * The drain performs exactly the work `drain.wf` performs: acquire the initial
 * directory, open one relative path under it, read into the same buffer until
 * the end, touch one byte of each delivered chunk so the buffer is genuinely
 * read, and report a witness of what it saw.
 */

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define PAGE 4096

static int initialized_choice(const char *name, int *initialized) {
    if (strcmp(name, "calloc") == 0) {
        *initialized = 1;
        return 0;
    }
    if (strcmp(name, "malloc") == 0) {
        *initialized = 0;
        return 0;
    }
    return -1;
}

static unsigned char *acquire(int initialized) {
    if (initialized) {
        return (unsigned char *)calloc(1, PAGE);
    }
    return (unsigned char *)malloc(PAGE);
}

static int drain(int initialized, const char *path) {
    /* The same one-time normalization the Whitefoot command bootstrap
       performs before entry, so the compared work is equal. */
    signal(SIGPIPE, SIG_IGN);

    unsigned char *page = acquire(initialized);
    if (page == NULL) {
        return 2;
    }

    int root = open(".", O_RDONLY);
    if (root < 0) {
        free(page);
        return 2;
    }
    int file = openat(root, path, O_RDONLY);
    if (file < 0) {
        close(root);
        free(page);
        return 2;
    }

    unsigned long total = 0;
    unsigned char witness = 0;
    for (;;) {
        ssize_t transferred = read(file, page, PAGE);
        if (transferred < 0) {
            close(file);
            close(root);
            free(page);
            return 2;
        }
        if (transferred == 0) {
            break;
        }
        total += (unsigned long)transferred;
        witness = (unsigned char)(witness + page[0]);
    }

    close(file);
    close(root);
    free(page);
    if (total == 0) {
        return 1;
    }
    if (witness == 0) {
        return 3;
    }
    return 0;
}

/* The isolated one-time cost: `repetitions` allocations of one buffer, each
 * touched once and released. The touch keeps the allocation observable, so
 * neither arm can be folded away, and it is identical in both arms. */
static int fill(int initialized, unsigned long repetitions) {
    struct timespec start;
    struct timespec stop;
    unsigned long witness = 0;

    if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) {
        return 2;
    }
    for (unsigned long round = 0; round < repetitions; round++) {
        unsigned char *page = acquire(initialized);
        if (page == NULL) {
            return 2;
        }
        page[round % PAGE] = (unsigned char)round;
        witness += page[round % PAGE];
        free(page);
    }
    if (clock_gettime(CLOCK_MONOTONIC, &stop) != 0) {
        return 2;
    }

    double elapsed = (double)(stop.tv_sec - start.tv_sec) * 1e9 +
                     (double)(stop.tv_nsec - start.tv_nsec);
    printf("{\"type\":\"fill\",\"initialized\":%d,\"repetitions\":%lu,"
           "\"elapsed_ns\":%.0f,\"per_allocation_ns\":%.4f,\"witness\":%lu}\n",
           initialized, repetitions, elapsed, elapsed / (double)repetitions,
           witness);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: control <drain|fill> <malloc|calloc> <path|repetitions>\n");
        return 2;
    }
    int initialized = 0;
    if (initialized_choice(argv[2], &initialized) != 0) {
        fprintf(stderr, "control: unknown allocation mode %s\n", argv[2]);
        return 2;
    }
    if (strcmp(argv[1], "drain") == 0) {
        return drain(initialized, argv[3]);
    }
    if (strcmp(argv[1], "fill") == 0) {
        return fill(initialized, strtoul(argv[3], NULL, 10));
    }
    fprintf(stderr, "control: unknown mode %s\n", argv[1]);
    return 2;
}
