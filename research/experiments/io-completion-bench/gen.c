/* glibc hides the POSIX and Linux entry points these files call when the
 * translation unit is compiled as strict C11, so the namespace is asked for
 * before any header. Harmless where it is not needed. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/* Deterministic generated file set for the io-completion-bench workloads.
 *
 * Writes COUNT files named f00000.dat .. into one directory. Every file's
 * length and bytes come from its index alone, so the folded checksum every
 * benchmark line publishes is reproducible from the index range without
 * reading the tree. */
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include "workload.h"

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: gen ROOT COUNT MAX_KIB\n");
        return 2;
    }
    const char *root = argv[1];
    unsigned long count = strtoul(argv[2], NULL, 10);
    unsigned long max_kib = strtoul(argv[3], NULL, 10);
    if (count == 0 || max_kib == 0) {
        fprintf(stderr, "gen: COUNT and MAX_KIB must be positive\n");
        return 2;
    }
    if (mkdir(root, 0755) != 0 && errno != EEXIST) {
        perror("gen: mkdir");
        return 1;
    }
    unsigned char *bytes = malloc(max_kib * 1024u);
    if (bytes == NULL) {
        fprintf(stderr, "gen: out of memory\n");
        return 1;
    }
    char path[4096];
    for (unsigned long index = 0; index < count; index++) {
        size_t length = wf_bench_file_length(index, max_kib);
        wf_bench_file_bytes(index, bytes, length);
        int written = snprintf(path, sizeof path, "%s/" WF_BENCH_NAME_FORMAT, root, index);
        if (written < 0 || (size_t)written >= sizeof path) {
            fprintf(stderr, "gen: path too long\n");
            free(bytes);
            return 1;
        }
        int descriptor = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (descriptor < 0) {
            perror("gen: open");
            free(bytes);
            return 1;
        }
        size_t done = 0;
        while (done < length) {
            ssize_t moved = write(descriptor, bytes + done, length - done);
            if (moved <= 0) {
                perror("gen: write");
                close(descriptor);
                free(bytes);
                return 1;
            }
            done += (size_t)moved;
        }
        if (close(descriptor) != 0) {
            perror("gen: close");
            free(bytes);
            return 1;
        }
    }
    free(bytes);
    return 0;
}
