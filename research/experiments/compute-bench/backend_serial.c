/* Serves compute-bench: the serial native reference. `map` is a loop and
 * `fork2` is left then right, both on the calling thread, so a kernel's
 * decomposition runs with no scheduler at all. This is the width-one row the
 * table reports beside `wf-seq`; it never enters the ratio column. */
#include "backend.h"
#include "harness.h"

static void serial_map(const wfb_backend *self, unsigned width, size_t chunks,
                       wfb_chunk fn, void *ctx) {
    (void)self;
    if (width != 1) wfb_fail("serial: width must be 1");
    if (chunks && !fn) wfb_fail("serial: missing callback");
    for (size_t i = 0; i < chunks; ++i) fn(ctx, i);
}

static void serial_fork2(const wfb_backend *self, unsigned width, wfb_task l,
                         void *lc, wfb_task r, void *rc) {
    (void)self;
    if (width != 1) wfb_fail("serial: width must be 1");
    if (!l || !r) wfb_fail("serial: missing task");
    l(lc);
    r(rc);
}

/* Nothing to release: no pool was ever created. */
static int serial_stop(void) { return 0; }

const wfb_backend wfb_backend_serial = {
    "serial", "none: one thread, a loop over all callbacks", 0, 0,
    serial_map, serial_fork2, serial_stop, NULL,
};
