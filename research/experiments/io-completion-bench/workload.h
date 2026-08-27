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

#include <stddef.h>
#include <stdint.h>

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

#endif
