/* Native single-thread FIR candidates; research qualification, no scheduler.
 * Contract: 1<=tap_count<=64, count+tap_count-1<=16777216; prefix contains
 * tap_count-1 old history samples (oldest first), then count new samples.
 * Input/coefficient backing stays live and is disjoint from writable output.
 * count output doubles are caller-owned and may start uninitialized. Empty
 * calls write nothing. All pointers remain valid and non-null even when empty. No prefix/state construction or allocation is hidden.
 *
 * Compile -fno-fast-math -ffp-contract=off. Each output starts at +0 and visits
 * all taps in ascending order, rounding multiply and add separately. SIMD
 * packs independent outputs; it never forms a reassociated tap reduction.
 */
#include "fir_native.h"

void fir_native_direct(const double *restrict prefix,
                       const double *restrict taps, size_t tap_count,
                       size_t count, double *restrict output) {
    size_t history = tap_count - 1;
    for (size_t n = 0; n < count; ++n) {
        double sum = 0.0;
        for (size_t k = 0; k < tap_count; ++k) {
            double product = taps[k] * prefix[history + n - k];
            sum = sum + product;
        }
        output[n] = sum;
    }
}

/* Clang's ordered tap-reduction vectorization can consume the loop before
 * output recurrences reach SLP. Disable just that loop transformation; this
 * leaves output-lane packing enabled and changes no arithmetic permission.
 * Other compilers may choose different code and require fresh qualification. */
#if defined(__clang__)
#define FIR_TAP_ORDER _Pragma("clang loop vectorize(disable) interleave(disable)")
#else
#define FIR_TAP_ORDER
#endif

/* Fixed output tiles expose independent recurrences to SLP/vectorization.
 * The compiler still chooses its ISA/vector width; no alignment or padded
 * tail is assumed. The remainder uses the identical direct recurrence. */
#define FIR_LANES(LANES) \
void fir_native_lanes##LANES(const double *restrict prefix, \
                             const double *restrict taps, size_t tap_count, \
                             size_t count, double *restrict output) { \
    size_t history = tap_count - 1; \
    size_t n = 0; \
    while (count - n >= LANES) { \
        double sums[LANES] = {0}; \
        FIR_TAP_ORDER \
        for (size_t k = 0; k < tap_count; ++k) { \
            double coefficient = taps[k]; \
            for (size_t lane = 0; lane < LANES; ++lane) { \
                double product = coefficient * prefix[history + n + lane - k]; \
                sums[lane] = sums[lane] + product; \
            } \
        } \
        for (size_t lane = 0; lane < LANES; ++lane) \
            output[n + lane] = sums[lane]; \
        n += LANES; \
    } \
    fir_native_direct(prefix + n, taps, tap_count, count - n, output + n); \
}

FIR_LANES(4)
FIR_LANES(8)
FIR_LANES(16)
#undef FIR_LANES
#undef FIR_TAP_ORDER
