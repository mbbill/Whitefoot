#include "records_scheduler.h"
#include "rayon-binding.h"
#ifndef RAYON_STRATEGY
#error "Select Rayon join (0) or parallel iterator (1)"
#endif
extern void rayon_run(unsigned, size_t, RecordChunk, void *, unsigned)
    __asm__(RAYON_SYMBOL);
void records_scheduler_run(unsigned width, size_t chunks, RecordChunk chunk, void *context) {
    rayon_run(width, chunks, chunk, context, RAYON_STRATEGY);
}
const char *records_scheduler_name(void) {
    return RAYON_STRATEGY ? "rayon-1.12.0-par-iter" : "rayon-1.12.0-join";
}
/* The caller remains registered with Rayon until process exit. */
int records_scheduler_stop(void) { return 0; }
