/* Serves compute-bench: the one scheduler boundary every native reference
 * implements, so that a kernel's decomposition is written once and dispatched
 * through oneTBB, ParlayLib, Rayon, a static pthread partition or a serial
 * loop without the kernel knowing which. The contract wording is reused from
 * the research bundle's records_scheduler.h; `fork2`, `grain` and `stop` are
 * added here and its RecordWork struct is cut, because each kernel owns its
 * own context type. */
#ifndef WFB_BACKEND_H
#define WFB_BACKEND_H
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif

/* The width ceiling, written once. It is exactly the width set {1, 2, 4, 8, 16, 32};
 * raising the set means raising this, and every backend's guard reads it.
 * rayon/adapter.rs carries the same number in Rust and says so; the two are
 * changed together or not at all. */
#define WFB_MAX_WIDTH 32

typedef void (*wfb_chunk)(void *context, size_t index);
typedef void (*wfb_task)(void *context);

typedef struct wfb_backend wfb_backend;
struct wfb_backend {
    const char *name;      /* equals the form name in the table */
    const char *grain;     /* one sentence; printed by `make compare` and
                              quoted verbatim in README.md */
    /* Read only by the backend that owns this struct, never by the harness.
       `strategy` selects among one backend's dispatch shapes (in
       backend_rayon.c: 0 = join bisection, 1 = parallel iterator);
       `left_offer` selects which subtree fork2 publishes. They are fields
       rather than a -D on a second compilation of the same source, because a
       second compilation would define every symbol in that file twice and the
       link would fail. The Rust and ParlayLib sides already take both as
       ordinary run-time arguments. */
    unsigned strategy;
    int left_offer;
    /* Calls fn(ctx, i) exactly once for every i in [0, chunks) and joins all
       callbacks before returning. Storage and context belong to the caller
       throughout. `width` counts the participating caller, is immutable
       within a process, and satisfies 1 <= width <= WFB_MAX_WIDTH; every
       backend checks that and calls wfb_fail otherwise. Initialization is
       lazy inside the first run and is charged to that call. NULL when the
       backend has no flat form. `self` is the struct this slot was reached
       through. */
    void (*map)(const wfb_backend *self, unsigned width, size_t chunks,
                wfb_chunk fn, void *ctx);
    /* Runs both tasks and joins both before returning. NULL when absent. */
    void (*fork2)(const wfb_backend *self, unsigned width,
                  wfb_task l, void *lc, wfb_task r, void *rc);
    /* Release a reusable pool after joined work. 0 = process lifetime,
       1 = explicit shutdown completed. Printed in the manifest. */
    int  (*stop)(void);
    /* A short mark for the table's `note` column, or NULL when this reference
       needs none. It is a fixed property of the policy above, printed in the
       process header and copied through by the reducer; it is never derived
       from a form's name and never carries a measurement. */
    const char *note;
};

extern const wfb_backend wfb_backend_serial, wfb_backend_static,
    wfb_backend_tbb, wfb_backend_rayon_join, wfb_backend_rayon_join_left,
    wfb_backend_rayon_iter;
#ifndef WFB_NO_PARLAY
extern const wfb_backend wfb_backend_parlay, wfb_backend_parlay_left;
#endif

/* NULL when this build has no such backend (a host where the ParlayLib probe
   in `deps` did not compile, say). The kernel then prints the row as n/a with
   the reason, which is WFB_PARLAY_REASON when that is why. */
const wfb_backend *wfb_backend_named(const char *name);

#ifdef __cplusplus
}
#endif
#endif
