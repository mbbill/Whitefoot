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
 * reading the tree.
 *
 * A fourth argument "fixed" makes every file exactly MAX_KIB KiB instead of a
 * mixed size decided by the index. That is the read-heavy workload's tree: a
 * few large files, opened once, read at thousands of positioned offsets. */
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <direct.h>
#include <io.h>
#include <share.h>
#include <sys/stat.h>
#else
#include <sys/stat.h>
#include <unistd.h>
#endif

#include "workload.h"

#if defined(_WIN32)
#define WF_BENCH_BINARY_FLAG _O_BINARY
#define WF_BENCH_WRITE_FLAGS (_O_WRONLY | _O_CREAT | _O_TRUNC)
#define wf_bench_close _close
#define wf_bench_mkdir(path) _mkdir(path)
#define wf_bench_write(descriptor, bytes, count) \
    _write((descriptor), (bytes), (unsigned)(count))
typedef int wf_bench_write_result;

static int wf_bench_open(const char *path, int flags, int mode) {
    int descriptor = -1;
    errno_t error;
    (void)mode;
    error = _sopen_s(
        &descriptor,
        path,
        flags | WF_BENCH_BINARY_FLAG,
        _SH_DENYNO,
        _S_IREAD | _S_IWRITE
    );
    if (error != 0) {
        errno = (int)error;
        return -1;
    }
    return descriptor;
}
#else
#define WF_BENCH_BINARY_FLAG 0
#define WF_BENCH_WRITE_FLAGS (O_WRONLY | O_CREAT | O_TRUNC)
#define wf_bench_close close
#define wf_bench_mkdir(path) mkdir((path), 0755)
#define wf_bench_open(path, flags, mode) open((path), (flags), (mode))
#define wf_bench_write(descriptor, bytes, count) \
    write((descriptor), (bytes), (count))
typedef ssize_t wf_bench_write_result;
#endif

int main(int argc, char **argv) {
    if (argc != 4 && argc != 5) {
        fprintf(stderr, "usage: gen ROOT COUNT MAX_KIB [fixed]\n");
        return 2;
    }
    const char *root = argv[1];
    unsigned long count = strtoul(argv[2], NULL, 10);
    unsigned long max_kib = strtoul(argv[3], NULL, 10);
    int fixed = argc == 5 && strcmp(argv[4], "fixed") == 0;
    if (argc == 5 && !fixed) {
        fprintf(stderr, "gen: the fourth argument, when given, must be \"fixed\"\n");
        return 2;
    }
    if (count == 0 || max_kib == 0) {
        fprintf(stderr, "gen: COUNT and MAX_KIB must be positive\n");
        return 2;
    }
    if (wf_bench_mkdir(root) != 0 && errno != EEXIST) {
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
        size_t length = fixed ? (size_t)max_kib * 1024u
                              : wf_bench_file_length(index, max_kib);
        wf_bench_file_bytes(index, bytes, length);
        int written = snprintf(path, sizeof path, "%s/" WF_BENCH_NAME_FORMAT, root, index);
        if (written < 0 || (size_t)written >= sizeof path) {
            fprintf(stderr, "gen: path too long\n");
            free(bytes);
            return 1;
        }
        int descriptor = wf_bench_open(
            path,
            WF_BENCH_WRITE_FLAGS,
            0644
        );
        if (descriptor < 0) {
            perror("gen: open");
            free(bytes);
            return 1;
        }
        /* The generated tree must not be resident in the page cache when it
         * is handed to an uncached table: F_NOCACHE on a reading descriptor
         * does not evict a page a write has just made resident. */
        wf_bench_no_populate(descriptor);
        size_t done = 0;
        while (done < length) {
            wf_bench_write_result moved = wf_bench_write(
                descriptor,
                bytes + done,
                length - done
            );
            if (moved <= 0) {
                perror("gen: write");
                wf_bench_close(descriptor);
                free(bytes);
                return 1;
            }
            done += (size_t)moved;
        }
        if (wf_bench_write_drop_cache(descriptor) != 0) {
            perror("gen: flush");
            wf_bench_close(descriptor);
            free(bytes);
            return 1;
        }
        if (wf_bench_close(descriptor) != 0) {
            perror("gen: close");
            free(bytes);
            return 1;
        }
    }
    free(bytes);
    return 0;
}
