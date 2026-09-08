#ifndef WHITEFOOT_RECORDS_SCHEDULER_H
#define WHITEFOOT_RECORDS_SCHEDULER_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

/* Research-only scheduler boundary. Each invocation calls chunk(context, i)
 * exactly once for every i in [0, chunks), and joins all callbacks before
 * returning. Context and output storage belong to the caller throughout.
 * Width counts the participating caller; it is immutable within a process.
 * Initialization is lazy inside the first run, and is charged to that call.
 * Callbacks do finite CPU work without blocking dependencies on other chunk
 * indices. There is one external caller; static partition does not nest. */
typedef void (*RecordChunk)(void *context, size_t i);
void records_scheduler_run(unsigned width, size_t chunks, RecordChunk chunk, void *context);
const char *records_scheduler_name(void);
/* Release a reusable library/pool after joined work. A process-lifetime
 * runtime returns zero; one indicates that explicit shutdown completed. */
int records_scheduler_stop(void);

/* Common precompiled scalar work, shared unchanged by all schedulers. */
typedef struct {
    const uint8_t *data;
    const uint64_t *offsets;
    uint64_t *output;
    size_t records, grain;
} RecordWork;
void records_scheduler_chunk(void *, size_t);

#ifdef __cplusplus
}
#endif
#endif
