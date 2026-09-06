/* The mixed-load experiment's fixed request/response protocol. Owned by
 * io-model/SCHEDULER-EXPERIMENT.md; retain while the mixed-load comparison is
 * active, and remove with that experiment if it is superseded.
 *
 * Request: big-endian u64 seed, big-endian u64 round count, 48 reserved bytes.
 * Response: 64 bytes, each 0 or 1, encoding the recurrence's result low bit
 * first. Framing is independent of TCP segmentation. */
#ifndef WHITEFOOT_BENCH_COMPUTE_PROTOCOL_H
#define WHITEFOOT_BENCH_COMPUTE_PROTOCOL_H

#include <stdint.h>

#define COMPUTE_BYTES 64u
#define COMPUTE_MAX_ROUNDS 16777216u

static inline uint64_t compute_churn(uint64_t seed, uint64_t rounds) {
    uint64_t value = seed;
    for (uint64_t round = 0; round < rounds; round++) {
        uint64_t rotated = (value << 17) | (value >> 47);
        value = (value ^ rotated) * UINT64_C(6364136223846793005)
            + UINT64_C(1442695040888963407);
    }
    return value;
}

static inline uint64_t compute_seed(uint64_t connection, uint64_t round) {
    return UINT64_C(0x9e3779b97f4a7c15)
        ^ (connection * UINT64_C(0xd6e8feb86659fd93))
        ^ (round * UINT64_C(0xa0761d6478bd642f));
}

static inline uint64_t compute_decode(const unsigned char *bytes) {
    uint64_t value = 0;
    for (unsigned at = 0; at < 8; at++) value = (value << 8) | bytes[at];
    return value;
}

static inline void compute_request(unsigned char *bytes, uint64_t seed, uint64_t rounds) {
    for (unsigned at = 0; at < 8; at++) {
        bytes[7 - at] = (unsigned char)(seed >> (at * 8));
        bytes[15 - at] = (unsigned char)(rounds >> (at * 8));
    }
    for (unsigned at = 16; at < COMPUTE_BYTES; at++) bytes[at] = 0;
}

static inline void compute_encode(unsigned char *bytes, uint64_t value) {
    for (unsigned at = 0; at < COMPUTE_BYTES; at++) bytes[at] = (unsigned char)((value >> at) & 1u);
}

static inline int compute_response(unsigned char *bytes) {
    uint64_t seed = compute_decode(bytes);
    uint64_t rounds = compute_decode(bytes + 8);
    if (rounds > COMPUTE_MAX_ROUNDS) return 0;
    compute_encode(bytes, compute_churn(seed, rounds));
    return 1;
}

#endif
