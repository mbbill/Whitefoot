/* Serves compute-bench: the oneTBB native reference, registering one form,
 * `tbb`. `map` is parallel_for over a blocked_range with the auto
 * partitioner; `fork2` is parallel_invoke. Both run inside one
 * process-lifetime task_arena(width, 1) under a global_control that admits at
 * most width-1 library workers alongside the caller's reserved slot.
 *
 * Adapted from the research bundle's records_tbb.cpp, with fork2 from its
 * quadrature_native.cpp. The named cut: that file failed width outside
 * {1, 2, 4}; the guard here reads WFB_MAX_WIDTH from backend.h. */
#include "backend.h"
#include "harness.h"

#include <oneapi/tbb/blocked_range.h>
#include <oneapi/tbb/global_control.h>
#include <oneapi/tbb/parallel_for.h>
#include <oneapi/tbb/parallel_invoke.h>
#include <oneapi/tbb/partitioner.h>
#include <oneapi/tbb/task_arena.h>
#include <oneapi/tbb/version.h>

#include <cstddef>
#include <exception>

static_assert(TBB_VERSION_MAJOR == 2023 && TBB_VERSION_MINOR == 1 && TBB_VERSION_PATCH == 0,
              "This reference is qualified against oneTBB v2023.1.0");

namespace {
namespace tbb = oneapi::tbb;

// Pinned source: uxlfoundation/oneTBB, 3046c8b0c29df995980003ea24f4d78c80ec0c8d.
// One external caller owns this arena. Nested callbacks may enter again:
// execute reuses the same arena and its participant limit, without a new pool.
class Arena {
    const unsigned width_;
    tbb::global_control limit_;
    tbb::task_arena arena_;

public:
    explicit Arena(unsigned width)
        : width_(width),
          limit_(tbb::global_control::max_allowed_parallelism, width),
          arena_(static_cast<int>(width), 1) {
        arena_.initialize();
    }
    unsigned width() const { return width_; }
    template <typename Body> void execute(Body body) { arena_.execute(body); }
};

Arena &arena_for(unsigned width) {
    // The first call, including an empty one, pays arena initialization.
    static Arena arena(width);
    if (arena.width() != width) wfb_fail("tbb: width changed within one process");
    return arena;
}

void guard(unsigned width) {
    if (width < 1 || width > WFB_MAX_WIDTH) wfb_fail("tbb: width out of range");
}

void tbb_map(const wfb_backend *self, unsigned width, std::size_t chunks,
             wfb_chunk fn, void *ctx) {
    (void)self;
    guard(width);
    if (chunks && !fn) wfb_fail("tbb: missing callback");
    try {
        arena_for(width).execute([=] {
            // The adaptive partitioner groups callback indices and subdivides
            // on steals. Grain 1 permits splitting to one chunk; it does not
            // force one task per chunk or match another runtime's DAG.
            tbb::parallel_for(
                tbb::blocked_range<std::size_t>(0, chunks, 1),
                [=](const tbb::blocked_range<std::size_t> &range) {
                    for (std::size_t i = range.begin(); i != range.end(); ++i) fn(ctx, i);
                },
                tbb::auto_partitioner{});
        });
    } catch (const std::exception &error) {
        wfb_fail(error.what());
    } catch (...) {
        wfb_fail("tbb: scheduler or callback exception");
    }
}

void tbb_fork2(const wfb_backend *self, unsigned width, wfb_task l, void *lc,
               wfb_task r, void *rc) {
    (void)self;
    guard(width);
    if (!l || !r) wfb_fail("tbb: missing task");
    try {
        arena_for(width).execute([=] {
            tbb::parallel_invoke([=] { l(lc); }, [=] { r(rc); });
        });
    } catch (const std::exception &error) {
        wfb_fail(error.what());
    } catch (...) {
        wfb_fail("tbb: scheduler or callback exception");
    }
}

// Every run has joined its callbacks, but this reusable process-lifetime
// arena and control remain active. Arena destruction alone would not prove
// that the library's global worker threads have exited.
int tbb_stop(void) { return 0; }

}  // namespace

extern "C" {
const wfb_backend wfb_backend_tbb = {
    "tbb", "oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1", 0, 0,
    tbb_map, tbb_fork2, tbb_stop, NULL,
};
}
