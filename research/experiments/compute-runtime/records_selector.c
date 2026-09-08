/* Same-image layout diagnostic, owned by the records scheduler experiment.
 * Retire with that diagnostic. Every process selects exactly one adapter;
 * this adds common indirect dispatch and loads every linked runtime library. */
#include "records_scheduler.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define DECLARE_ADAPTER(prefix) \
    void prefix##_run(unsigned, size_t, RecordChunk, void *); \
    const char *prefix##_name(void); \
    int prefix##_stop(void)
DECLARE_ADAPTER(layout_wf);
DECLARE_ADAPTER(layout_wf_group4);
DECLARE_ADAPTER(layout_wf_group16);
DECLARE_ADAPTER(layout_static);
DECLARE_ADAPTER(layout_tbb);
DECLARE_ADAPTER(layout_parlay);
void records_parlay_auto_run(unsigned, size_t, RecordChunk, void *);
const char *records_parlay_auto_name(void);
DECLARE_ADAPTER(layout_rayon_join);
DECLARE_ADAPTER(layout_rayon_iter);

typedef struct {
    const char *key;
    void (*run)(unsigned, size_t, RecordChunk, void *);
    const char *(*name)(void);
    int (*stop)(void);
} Adapter;
#define ADAPTER(key, prefix) {key, prefix##_run, prefix##_name, prefix##_stop}
static const Adapter adapters[] = {
    ADAPTER("wf", layout_wf), ADAPTER("static", layout_static),
    ADAPTER("wf-group4", layout_wf_group4), ADAPTER("wf-group16", layout_wf_group16),
    ADAPTER("tbb", layout_tbb), ADAPTER("parlay", layout_parlay),
    {"parlay-auto", records_parlay_auto_run, records_parlay_auto_name, layout_parlay_stop},
    ADAPTER("rayon-join", layout_rayon_join), ADAPTER("rayon-iter", layout_rayon_iter)
};
static const Adapter *selected;

void records_scheduler_select(const char *name) {
    if (!selected && name) {
        for (size_t i = 0; i < sizeof(adapters) / sizeof(adapters[0]); ++i) {
            if (!strcmp(name, adapters[i].key)) { selected = &adapters[i]; return; }
        }
    }
    fputs("record scheduler: invalid or repeated shared-image selection\n", stderr);
    exit(1);
}

/* Selection precedes floor thread creation; the published pointer is immutable.
 * No extra lock or atomic operation belongs in this timed forwarding path. */
void records_scheduler_run(unsigned width, size_t chunks, RecordChunk chunk, void *context) {
    selected->run(width, chunks, chunk, context);
}
const char *records_scheduler_name(void) { return selected->name(); }
int records_scheduler_stop(void) { return selected->stop(); }
