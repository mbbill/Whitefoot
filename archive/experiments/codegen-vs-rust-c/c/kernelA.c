#include <stdint.h>
#include <stdio.h>

/* Kernel A: splitmix64-style mixing iterated N times, xor-accumulated.
 * Non-static so clang emits a standalone optimized copy we can inspect. */
uint64_t mix_n(uint64_t seed, uint64_t n) {
    uint64_t z = seed;
    uint64_t acc = 0;
    for (uint64_t i = 0; i < n; i++) {
        z = z + 0x9E3779B97F4A7C15ULL;
        z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
        z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
        z = z ^ (z >> 31);
        acc = acc ^ z;
    }
    return acc;
}

#ifndef SEED
#define SEED 0x0123456789ABCDEFULL
#endif
#ifndef NITER
#define NITER 200000000ULL
#endif

int main(void) {
    uint64_t r = mix_n(SEED, NITER);
    printf("%llu\n", (unsigned long long)r);
    return (int)(r & 0xFF);
}
