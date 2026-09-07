#ifndef WHITEFOOT_FIR_STATIC_H
#define WHITEFOOT_FIR_STATIC_H
#include "fir_native.h"

typedef struct FirStatic FirStatic;
typedef struct {
    unsigned requested_lanes;
    unsigned actual_lanes;  /* Includes the caller; helper threads = lanes-1. */
    int creation_error;     /* pthread error from a partial start, otherwise 0. */
} FirStaticInfo;

/* One owner creates, calls and destroys this pool; concurrent or nested use of
 * the same pool is outside this C research API. Requested lanes: 1, 2 or 4.
 * Startup allocates the pool, starts persistent threads and waits for readiness.
 * A refused helper leaves a usable caller-plus-started-helper pool and is
 * reported. Fatal pool/mutex/condition setup failure returns NULL with errno. */
FirStatic *fir_static_create(unsigned requested_lanes, FirStaticInfo *info);

/* Same input/output contract as fir_native.h. Prefix/coefficient reads are
 * shared; output ranges are contiguous and disjoint. All kernels complete
 * before return. Only nonempty ranges are dispatched; N=0 writes nothing.
 * Pool dispatch does not allocate or create threads; the qualified fir_native
 * kernels also have no heap allocation.
 * Idle helpers park on a condition variable; there is no spin phase. */
void fir_static_run(FirStatic *, FirNativeKernel, const double *prefix,
                    const double *taps, size_t tap_count, size_t count,
                    double *output);

/* Call only after the last run returns; stops and joins every started helper
 * before releasing pool storage. Startup/shutdown are separate from warm run. */
void fir_static_destroy(FirStatic *);
#endif
