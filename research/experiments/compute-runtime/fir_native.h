#ifndef WHITEFOOT_FIR_NATIVE_H
#define WHITEFOOT_FIR_NATIVE_H

#include <stddef.h>

/* Single-thread strict FIR kernel API, with caller-validated preconditions:
 * 1<=tap_count<=64; count+tap_count-1<=16777216. prefix holds tap_count-1 old
 * history samples (oldest first), then count new samples; taps holds tap_count
 * coefficients. output has count writable doubles, disjoint from both inputs.
 * All three pointers are valid, non-null object pointers even for empty ranges;
 * backing stays live through the call. No SIMD alignment or padding is needed.
 * Empty calls write nothing. Kernels do not allocate, build prefixes, copy
 * final history, or validate/consume outputs. Compile without reassociation or
 * contraction (-fno-fast-math -ffp-contract=off). This is a C research API. */
typedef void (*FirNativeKernel)(const double *prefix, const double *taps,
                                size_t tap_count, size_t count, double *output);

void fir_native_direct(const double *, const double *, size_t, size_t, double *);
void fir_native_lanes4(const double *, const double *, size_t, size_t, double *);
void fir_native_lanes8(const double *, const double *, size_t, size_t, double *);
void fir_native_lanes16(const double *, const double *, size_t, size_t, double *);

#endif
