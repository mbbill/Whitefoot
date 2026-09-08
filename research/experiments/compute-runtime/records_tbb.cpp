#include "records_scheduler.h"
#include <oneapi/tbb/blocked_range.h>
#include <oneapi/tbb/global_control.h>
#include <oneapi/tbb/parallel_for.h>
#include <oneapi/tbb/partitioner.h>
#include <oneapi/tbb/task_arena.h>
#include <oneapi/tbb/version.h>
#include <cstdio>
#include <cstdlib>
#include <exception>

static_assert(TBB_VERSION_MAJOR == 2023 && TBB_VERSION_MINOR == 1 && TBB_VERSION_PATCH == 0,
              "This reference is qualified against oneTBB v2023.1.0");

namespace {
namespace tbb = oneapi::tbb;
[[noreturn]] void fail(const char *message) {
    std::fprintf(stderr,"records oneTBB: %s\n",message);
    std::abort();
}

// Pinned source: uxlfoundation/oneTBB, 3046c8b0c29df995980003ea24f4d78c80ec0c8d.
// One external caller owns this arena. Nested callbacks may invoke run again:
// execute reuses the same arena and its participant limit, without a new pool.
class Scheduler {
    const unsigned width_;
    tbb::global_control limit_;
    tbb::task_arena arena_;
public:
    explicit Scheduler(unsigned width)
        : width_(width), limit_(tbb::global_control::max_allowed_parallelism,width),
          arena_(static_cast<int>(width),1) {
        arena_.initialize();
    }
    void run(unsigned width, size_t chunks, RecordChunk chunk, void *context) {
        if (width != width_) fail("width changed within one process");
        arena_.execute([=] {
            // The normal adaptive partitioner groups callback indices and
            // subdivides on steals. Grain 1 permits splitting to one chunk;
            // it does not force one task per chunk or match another runtime's DAG.
            tbb::parallel_for(tbb::blocked_range<size_t>(0,chunks,1),
                [=](const tbb::blocked_range<size_t> &range) {
                    for (size_t i=range.begin(); i!=range.end(); ++i) chunk(context,i);
                }, tbb::auto_partitioner{});
        });
        // parallel_for joins callbacks. Its trivially captured body copies may
        // outlive the call; destroying them does not access the borrowed context.
    }
};
}

extern "C" void records_scheduler_run(unsigned width, size_t chunks, RecordChunk chunk, void *context) {
    if (width != 1 && width != 2 && width != 4) fail("width must be 1, 2 or 4");
    if (chunks && !chunk) fail("missing callback");
    try {
        // The first run, including an empty run, pays arena initialization.
        // The global control admits at most width-1 oneTBB workers, alongside
        // the caller's reserved arena slot. No enqueue API is used here.
        static Scheduler scheduler(width);
        scheduler.run(width,chunks,chunk,context);
    } catch (const std::exception &error) {
        fail(error.what());
    } catch (...) {
        fail("scheduler or callback exception");
    }
}

extern "C" const char *records_scheduler_name(void) {
    return "oneTBB-v2023.1.0-auto-grain1";
}

extern "C" int records_scheduler_stop(void) {
    // Every run has joined its callbacks, but this reusable process-lifetime
    // arena/control remains active. Arena destruction alone would not prove
    // that the library's global worker threads have exited.
    return 0;
}
