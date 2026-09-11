/* Serves compute-bench: the ParlayLib native-scheduler reference. It
 * registers two forms that differ only in the struct's `left_offer` field:
 * `parlay` offers the right subtree and `parlay-left` offers the left one,
 * which is the direction generated Whitefoot publishes. Fork direction is a
 * first-order effect on skewed input, so both are reported.
 *
 * Adapted from the research bundle's records_parlay.cpp, with fork2 from its
 * quadrature_native.cpp. Named cuts: that file failed width outside {1, 2, 4}
 * -- the guard here reads WFB_MAX_WIDTH -- and its automatic-granularity
 * variant is not carried, because the bundle's own grain panel records it
 * about 2.9x slower than fixed grain on trailing input.
 *
 * The whole file is inside #ifndef WFB_NO_PARLAY, matching backend.h's two
 * extern declarations and harness.c's two table entries: on a host where the
 * `make deps` probe did not compile ParlayLib, all three disappear together
 * and the table prints the rows as n/a with the recorded reason. */
#ifndef WFB_NO_PARLAY

#include "backend.h"
#include "harness.h"

// Pinned ParlayLib: 51017699dcc421f80479cdb238d3092233ad0d26.
// Keep the native scheduler and its default elastic idle policy. The other
// plugins implement the same API, so accepting them would mislabel this row.
#if defined(PARLAY_CILKPLUS) || defined(PARLAY_OPENCILK) || defined(PARLAY_OPENMP) || \
    defined(PARLAY_TBB) || defined(PARLAY_SEQUENTIAL)
#error "The compute-bench Parlay reference requires the native scheduler"
#endif
#include <parlay/parallel.h>
#ifndef PARLAY_USING_PARLAY_SCHEDULER
#error "The compute-bench Parlay reference did not select the native scheduler"
#endif
static_assert(PARLAY_ELASTIC_PARALLELISM, "Keep Parlay's default elastic policy");
static_assert(PARLAY_ELASTIC_STEAL_TIMEOUT == 10000, "Keep Parlay's default idle timeout");

#include <cstddef>
#include <exception>

namespace {
using Scheduler = parlay::internal::scheduler_type;
Scheduler *pool = nullptr;
unsigned selected_width = 0;
bool stopped = false;

void guard(unsigned width) {
    if (width < 1 || width > WFB_MAX_WIDTH) wfb_fail("parlay: width out of range");
    if (stopped) wfb_fail("parlay: run after shutdown");
    if (selected_width && selected_width != width) wfb_fail("parlay: width changed");
}

// The constructor registers the caller as worker 0 and creates width-1
// helpers. The environment-sized thread-local default scheduler is never used.
void enter(unsigned width) {
    if (!pool) {
        if (Scheduler::get_current_scheduler()) wfb_fail("parlay: caller already owns a scheduler");
        selected_width = width;
        pool = new Scheduler(width);
    }
    if (Scheduler::get_current_scheduler() != pool)
        wfb_fail("parlay: caller is outside the native scheduler");
}

void parlay_map(const wfb_backend *self, unsigned width, std::size_t chunks,
                wfb_chunk fn, void *ctx) {
    (void)self;
    guard(width);
    if (chunks && !fn) wfb_fail("parlay: missing callback");
    try {
        enter(width);
        // Granularity one fixes the host's callback units. Granularity zero
        // would add the upstream timed prefix and automatic subdivision.
        parlay::parallel_for(std::size_t{0}, chunks, [&](std::size_t i) { fn(ctx, i); }, 1,
                             false);
    } catch (const std::exception &error) {
        wfb_fail(error.what());
    } catch (...) {
        wfb_fail("parlay: unexpected scheduler exception");
    }
}

// parlay::par_do runs its FIRST callback locally and publishes the SECOND, so
// passing (right, left) offers the left subtree.
void parlay_fork2(const wfb_backend *self, unsigned width, wfb_task l, void *lc,
                  wfb_task r, void *rc) {
    guard(width);
    if (!l || !r) wfb_fail("parlay: missing task");
    try {
        enter(width);
        if (self->left_offer)
            parlay::par_do([&] { r(rc); }, [&] { l(lc); });
        else
            parlay::par_do([&] { l(lc); }, [&] { r(rc); });
    } catch (const std::exception &error) {
        wfb_fail(error.what());
    } catch (...) {
        wfb_fail("parlay: unexpected scheduler exception");
    }
}

// Upstream destruction signals completion, wakes sleepers, joins every helper
// and restores the caller's previous scheduler registration.
int parlay_stop(void) {
    if (!pool) return stopped ? 1 : 0;
    if (Scheduler::get_current_scheduler() != pool || pool->worker_id() != 0)
        wfb_fail("parlay: shutdown requires the scheduler owner");
    delete pool;
    pool = nullptr;
    stopped = true;
    return 1;
}

}  // namespace

extern "C" {
const wfb_backend wfb_backend_parlay = {
    "parlay", "ParlayLib native scheduler, parallel_for granularity 1, right-offer fork",
    0, 0, parlay_map, parlay_fork2, parlay_stop, NULL,
};
const wfb_backend wfb_backend_parlay_left = {
    "parlay-left", "ParlayLib native scheduler, parallel_for granularity 1, left-offer fork",
    0, 1, parlay_map, parlay_fork2, parlay_stop, NULL,
};
}

#endif /* WFB_NO_PARLAY */
