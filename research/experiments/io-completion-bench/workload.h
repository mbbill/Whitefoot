/* The one definition of the generated tree and of the checksum every line of
 * the io-completion-bench must publish.
 *
 * The generator, the native baselines, and the Whitefoot programs all fold the
 * same value: per file, digest = fold(byte) with digest = digest*31 + byte
 * seeded at zero; per tree, sum = sum + digest*(index+1), all in wrapping
 * u64. The weight makes the total independent of the order in which files are
 * folded, so a four-wide lane split and a one-at-a-time loop publish the same
 * bytes. */
#ifndef WF_BENCH_WORKLOAD_H
#define WF_BENCH_WORKLOAD_H

#include <fcntl.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#if defined(_WIN32)
#include <io.h>
#else
#include <unistd.h>
#endif

/* The Whitefoot programs render this name shape from an index with fixed
 * arithmetic, so it is ten bytes wide and never varies. */
#define WF_BENCH_NAME_FORMAT "f%05lu.dat"
#define WF_BENCH_NAME_BYTES 10

/* The read window every line uses. Files never exceed it, so one positioned
 * read transfers a whole file. */
#define WF_BENCH_READ_WINDOW 65536u

static inline uint64_t wf_bench_mix(uint64_t value) {
    value ^= value >> 33;
    value *= 0xff51afd7ed558ccdULL;
    value ^= value >> 33;
    value *= 0xc4ceb9fe1a85ec53ULL;
    value ^= value >> 33;
    return value;
}

/* Mixed sizes: 1 KiB to max_kib KiB, decided by the index alone. */
static inline size_t wf_bench_file_length(uint64_t index, unsigned long max_kib) {
    uint64_t spread = wf_bench_mix(index * 2654435761ULL + 1ULL);
    return (size_t)((spread % max_kib) + 1u) * 1024u;
}

static inline void wf_bench_file_bytes(uint64_t index, unsigned char *out, size_t length) {
    uint64_t state = wf_bench_mix(index + 0x9e3779b97f4a7c15ULL);
    for (size_t at = 0; at < length; at++) {
        state = state * 6364136223846793005ULL + 1442695040888963407ULL;
        out[at] = (unsigned char)(state >> 56);
    }
}

static inline uint64_t wf_bench_digest(const unsigned char *bytes, size_t length) {
    uint64_t digest = 0;
    for (size_t at = 0; at < length; at++) {
        digest = digest * 31ULL + (uint64_t)bytes[at];
    }
    return digest;
}

static inline uint64_t wf_bench_weighted(uint64_t digest, uint64_t index) {
    return digest * (index + 1ULL);
}


/* The read-heavy workload's tree and schedule.
 *
 * The many-files workload above measures a program that opens thousands of
 * small files, which on a host whose openat costs 116 us measures the host's
 * open path and not the completion framework. This one opens a few large
 * files once and then does thousands of positioned reads into them, so the
 * reads are what the time is made of.
 *
 * Read k of a run selects its file and its window-aligned offset from k
 * alone, so the native baselines, the narrow Whitefoot program, and the
 * eight-wide one all traverse exactly the same list and fold exactly the same
 * value. The Whitefoot programs compute the same mix with `ixor`,
 * `ishr.wrap`, and `*wrap`; changing either side without the other changes
 * the published checksum, which is what keeps them honest. */
#define WF_BENCH_BIG_FILES 8u
#define WF_BENCH_BIG_KIB 65536u
#define WF_BENCH_BIG_BYTES ((uint64_t)WF_BENCH_BIG_KIB * 1024u)

/* File index for read k: the lane, so an eight-wide round reads eight
 * different files and a narrow lane stays on one. */
static inline uint64_t wf_bench_read_file(uint64_t position) {
    return position % (uint64_t)WF_BENCH_BIG_FILES;
}

/* How much of each read the checksum folds.
 *
 * Folding every byte would measure the fold and not the framework. The digest
 * is a serial multiply-add chain that runs at about 800 MB/s in both C and
 * Whitefoot, so a full 64 KiB window costs about 80 us of CPU against a 134 us
 * uncached read on the macOS host, and against a 7 us warm one. A program
 * folding every byte makes the warm table pure compute, and adds to the
 * uncached table CPU that a wide program spreads over helpers and a narrow one
 * does not -- a difference that has nothing to do with the I/O being measured. Each line folds the
 * first sixty-fourth of every window instead. That still pins the file, the
 * offset, and the position of every read, and the full transferred byte count
 * is published beside the checksum, so a short read, a dropped read, or a read
 * of the wrong window is still caught. What it does not check is the tail of
 * a window whose first sixty-fourth was right. */
#define WF_BENCH_CHECK_SHIFT 6u

static inline size_t wf_bench_check_length(uint64_t window, uint64_t moved) {
    uint64_t wanted = window >> WF_BENCH_CHECK_SHIFT;
    return (size_t)(moved < wanted ? moved : wanted);
}

/* Window index for read k, over a file of `blocks` windows. */
static inline uint64_t wf_bench_read_block(uint64_t position, uint64_t blocks) {
    return wf_bench_mix(position * 11400714819323198485ULL) % blocks;
}

/* Darwin's F_NOCACHE, applied where a descriptor's own traffic must not
 * populate the page cache: the generator's writes, and the residency probe's
 * reads. It is a mode of the descriptor, so it holds for every transfer
 * through it and not only the first, and it never evicts a page that is
 * already resident -- which is what lets the probe observe residency without
 * changing it.
 *
 * Linux has no such per-descriptor mode, so here it is a no-op: its only
 * lever, POSIX_FADV_DONTNEED, evicts, which is what an uncached open wants
 * and the opposite of what a probe wants. A Linux probe therefore accepts
 * that its own few megabytes become resident. */
static inline void wf_bench_no_populate(int descriptor) {
    if (descriptor < 0) {
        return;
    }
#if defined(__APPLE__)
    (void)fcntl(descriptor, F_NOCACHE, 1);
#else
    (void)descriptor;
#endif
}

/* The host call that takes a descriptor's reads past the page cache: do not
 * populate, and on Linux also drop what is resident now, since that is the
 * only lever the kernel offers. Neither half evicts a page another descriptor
 * is holding resident, which is why the tree is generated so that nothing is
 * resident to begin with and why the probe checks that claim before a table
 * may call itself uncached. */
static inline void wf_bench_uncached_host(int descriptor) {
    wf_bench_no_populate(descriptor);
#if defined(__linux__)
    if (descriptor >= 0) {
        (void)posix_fadvise(descriptor, 0, 0, POSIX_FADV_DONTNEED);
    }
#endif
}

/* Whether the run was asked for uncached reads.
 *
 * This mirrors, for the native baselines, the WF_IO_NOCACHE target policy the
 * Whitefoot runtime reads in
 * compiler/src/backend/completion/file_adapter.h. Both sides make the same
 * host call on the same descriptors so that N and C wait on the same device
 * rather than on two different cache states. */
static inline int wf_bench_uncached_requested(void) {
#if defined(_WIN32)
    char text[2] = {0};
    size_t required = 0;
    errno_t error = getenv_s(
        &required,
        text,
        sizeof(text),
        "WF_IO_NOCACHE"
    );
    return error == 0 && required == 2u && text[0] == '1' && text[1] == 0;
#else
    const char *text = getenv("WF_IO_NOCACHE");
    return text != NULL && text[0] == '1' && text[1] == 0;
#endif
}

static inline void wf_bench_apply_uncached(int descriptor) {
    if (!wf_bench_uncached_requested()) {
        return;
    }
    wf_bench_uncached_host(descriptor);
}

/* The write side of the same discipline: get the bytes onto the device and
 * leave nothing of them in the cache. Darwin needs only the flush, because
 * the writing descriptor above never populated; Linux drops the pages the
 * write path kept. Without this the first "uncached" table after a generation
 * is served from memory, which is the defect the batch-0092 record
 * describes. */
static inline int wf_bench_write_drop_cache(int descriptor) {
    if (descriptor < 0) {
        return -1;
    }
#if defined(_WIN32)
    if (_commit(descriptor) != 0) {
        return -1;
    }
#else
    if (fsync(descriptor) != 0) {
        return -1;
    }
#if defined(__linux__)
    (void)posix_fadvise(descriptor, 0, 0, POSIX_FADV_DONTNEED);
#endif
#endif
    return 0;
}

#endif
