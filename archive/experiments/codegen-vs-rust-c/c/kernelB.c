#include <stdint.h>
#include <stdio.h>

#define K 0x9E3779B97F4A7C15ULL

/* Non-affine mixing recurrence: acc = (acc ^ *addend) * K, iterated n times.
 * Nonlinear so LLVM cannot close-form it -> a real loop survives in every
 * variant; the ONLY difference is whether *addend is reloaded and *acc is
 * round-tripped through memory each iteration. */

/* naive: acc and addend may alias -> compiler must reload *addend and
 * store *acc every iteration. */
void accumulate_naive(uint64_t *acc, const uint64_t *addend, uint64_t n) {
    for (uint64_t i = 0; i < n; i++) {
        *acc = (*acc ^ *addend) * K;
    }
}

/* restrict: promises no alias -> *addend hoisted, *acc kept in a register. */
void accumulate_restrict(uint64_t *restrict acc, const uint64_t *restrict addend, uint64_t n) {
    for (uint64_t i = 0; i < n; i++) {
        *acc = (*acc ^ *addend) * K;
    }
}

#ifndef NITER
#define NITER 1000000000ULL
#endif

int main(void) {
    uint64_t a = 1, b = 3;
    accumulate_naive(&a, &b, NITER);   /* naive == restrict numerically */
    printf("%llu\n", (unsigned long long)a);
    return (int)(a & 0xFF);
}
