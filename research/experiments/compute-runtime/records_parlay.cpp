#include "records_scheduler.h"

#include <cstdio>
#include <cstdlib>
#include <exception>

// Pinned ParlayLib: 51017699dcc421f80479cdb238d3092233ad0d26.
// Keep the native scheduler and its default elastic idle policy. The other
// plugins implement the same API, so accepting them would mislabel this row.
#if defined(PARLAY_CILKPLUS) || defined(PARLAY_OPENCILK) || defined(PARLAY_OPENMP) || \
    defined(PARLAY_TBB) || defined(PARLAY_SEQUENTIAL)
#error "The records Parlay control requires the native scheduler"
#endif
#include <parlay/parallel.h>
#ifndef PARLAY_USING_PARLAY_SCHEDULER
#error "The records Parlay control did not select the native scheduler"
#endif
static_assert(PARLAY_ELASTIC_PARALLELISM, "Keep Parlay's default elastic policy");
static_assert(PARLAY_ELASTIC_STEAL_TIMEOUT == 10000, "Keep Parlay's default idle timeout");

namespace {
using Scheduler = parlay::internal::scheduler_type;
Scheduler *pool = nullptr;
unsigned selected_width = 0;
bool stopped = false;

[[noreturn]] void fail(const char *message) {
    std::fprintf(stderr, "records Parlay: %s\n", message);
    std::abort();
}

}

// One external owner initializes/runs/stops this adapter; external invocations
// must not overlap, and stop follows joined work. Same-width nested run calls
// from its callbacks can reuse Parlay's native scheduler and join/help path.
static void run(unsigned width, size_t chunks, RecordChunk chunk, void *context,
                long granularity) {
    if (width != 1 && width != 2 && width != 4) fail("width must be 1, 2 or 4");
    if (chunks && !chunk) fail("missing chunk callback");
    if (stopped) fail("run after shutdown");
    if (selected_width && selected_width != width) fail("width changed");
    try {
        if (!pool) {
            if (Scheduler::get_current_scheduler()) fail("caller already owns a Parlay scheduler");
            selected_width = width;
            // Constructor registers caller 0 and creates width-1 helpers. Do
            // not use the environment-sized, thread-local default scheduler.
            pool = new Scheduler(width);
        }
        if (Scheduler::get_current_scheduler() != pool) fail("caller is outside the native scheduler");
        // Grain one fixes the host's callback units. Grain zero includes the
        // upstream timed, exactly-once prefix and automatic subdivision.
        parlay::parallel_for(size_t{0}, chunks,
                             [&](size_t i) { chunk(context, i); }, granularity, false);
    } catch (const std::exception &error) {
        fail(error.what());
    } catch (...) {
        fail("unexpected scheduler exception");
    }
}

extern "C" void records_scheduler_run(unsigned width, size_t chunks,
                                      RecordChunk chunk, void *context) {
    run(width, chunks, chunk, context, 1);
}

extern "C" void records_parlay_auto_run(unsigned width, size_t chunks,
                                       RecordChunk chunk, void *context) {
    run(width, chunks, chunk, context, 0);
}

extern "C" const char *records_parlay_auto_name(void) {
    return "parlay-native-auto-grain";
}

extern "C" const char *records_scheduler_name(void) {
    return "parlay-native-grain1";
}

extern "C" int records_scheduler_stop(void) {
    if (!pool) return stopped ? 1 : 0;
    if (Scheduler::get_current_scheduler() != pool || pool->worker_id() != 0)
        fail("shutdown requires the scheduler owner");
    // Upstream destruction signals completion, wakes sleepers, joins every
    // helper and restores the caller's previous scheduler registration.
    delete pool;
    pool = nullptr;
    stopped = true;
    return 1;
}
