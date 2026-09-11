/* Serves compute-bench: the Rayon reference. This is a C shim over the two
 * v0-mangled entry points deps.sh recovers from the crate's emitted LLVM IR
 * into $(DEPS)/rayon/rayon-binding.h. It registers three forms that differ
 * only in their structs' `strategy` and `left_offer` fields.
 *
 * Adapted from the research bundle's records_rayon.c. The named cut: that file
 * was compiled twice with -DRAYON_STRATEGY=0 and =1. That cannot be carried,
 * because this file defines three wfb_backend structs with external linkage
 * and two objects from it would define all three twice. Nothing is lost:
 * adapter.rs already takes both selectors as ordinary run-time arguments. */
#include "backend.h"
#include "harness.h"
#include "rayon-binding.h"

#include <stddef.h>

extern void wfb_rayon_map(unsigned width, size_t chunks, wfb_chunk fn, void *ctx,
                          unsigned strategy) __asm__(RAYON_MAP_SYMBOL);
extern void wfb_rayon_fork2(unsigned width, wfb_task l, void *lc, wfb_task r, void *rc,
                            int left_offer) __asm__(RAYON_FORK2_SYMBOL);

static void rayon_map(const wfb_backend *self, unsigned width, size_t chunks,
                      wfb_chunk fn, void *ctx) {
    if (width < 1 || width > WFB_MAX_WIDTH) wfb_fail("rayon: width out of range");
    if (chunks && !fn) wfb_fail("rayon: missing callback");
    wfb_rayon_map(width, chunks, fn, ctx, self->strategy);
}

static void rayon_fork2(const wfb_backend *self, unsigned width, wfb_task l, void *lc,
                        wfb_task r, void *rc) {
    if (width < 1 || width > WFB_MAX_WIDTH) wfb_fail("rayon: width out of range");
    if (!l || !r) wfb_fail("rayon: missing task");
    wfb_rayon_fork2(width, l, lc, r, rc, self->left_offer);
}

/* The caller remains registered with Rayon until process exit: the
 * use_current_thread registry cannot be detached. */
static int rayon_stop(void) { return 0; }

const wfb_backend wfb_backend_rayon_join = {
    "rayon-join", "rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork",
    0, 0, rayon_map, rayon_fork2, rayon_stop, NULL,
};
const wfb_backend wfb_backend_rayon_join_left = {
    "rayon-join-left", "rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork",
    0, 1, rayon_map, rayon_fork2, rayon_stop, NULL,
};
const wfb_backend wfb_backend_rayon_iter = {
    "rayon-iter", "rayon 1.12.0 parallel iterator, its own adaptive splitting",
    1, 0, rayon_map, NULL, rayon_stop, NULL,
};
