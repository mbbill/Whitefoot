/* Post-hoc attribution control for the many-files divergence (disclosed in
 * RESULTS.md as beyond the four preregistered instruments; it classifies
 * nothing and exists to distinguish the claimed cause of the per-open cost).
 *
 * `openat` mode reproduces wfgrep's exact per-file syscall shape: one
 * O_DIRECTORY descriptor for the working directory, then per file
 * openat(dirfd, path, O_RDONLY), reads into one reused 4096-byte buffer,
 * close. `open` mode reproduces the comparator's plain open(path) shape.
 * The binary shares wfgrep's provenance (local unsigned clang -O2 output),
 * so comparing it with wfgrep and with the platform-signed system grep
 * separates process-identity cost in the host's security layer from the
 * program's own syscall shape.
 */
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: open-control openat|open FILE...\n");
        return 2;
    }
    int use_openat = strcmp(argv[1], "openat") == 0;
    int dirfd = -1;
    if (use_openat) {
        dirfd = open(".", O_DIRECTORY);
        if (dirfd < 0) {
            perror("open .");
            return 2;
        }
    }
    static unsigned char buffer[4096];
    unsigned long long witness = 0;
    for (int i = 2; i < argc; i++) {
        int fd = use_openat ? openat(dirfd, argv[i], O_RDONLY)
                            : open(argv[i], O_RDONLY);
        if (fd < 0) {
            perror(argv[i]);
            return 2;
        }
        ssize_t got;
        while ((got = read(fd, buffer, sizeof buffer)) > 0) {
            witness += buffer[0] + (unsigned long long)got;
        }
        close(fd);
        if (got < 0) {
            perror("read");
            return 2;
        }
    }
    printf("%llu\n", witness);
    return 0;
}
