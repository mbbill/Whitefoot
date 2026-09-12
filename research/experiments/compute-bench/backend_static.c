/* Serves compute-bench: the static-partition native reference. Persistent
 * pthread helpers, a contiguous equal partition of the chunk-index space, one
 * dispatch per call, no stealing. It is a strong regular-work reference and
 * deliberately not a dynamic-scheduling ceiling for skew, which is why the
 * timed shapes are the skewed ones.
 *
 * Adapted from the research bundle's records_static.c. The named cut: that
 * file declared a fixed `Worker workers[3]` and guarded width against
 * {1, 2, 4}, which would corrupt memory above width 4. Here the worker array
 * and its 128-byte-aligned completion cells are heap-allocated at the first
 * run, sized `width - 1` from the argument, freed in stop(), and the guard
 * reads WFB_MAX_WIDTH from backend.h.
 *
 * Idle lanes busy-poll. That CPU is charged to this reference, not hidden. */
#include "backend.h"
#include "harness.h"

#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>

_Static_assert(ATOMIC_LONG_LOCK_FREE == 2, "static control needs lock-free epochs");

typedef struct {
    _Alignas(128) atomic_ulong done;
    pthread_t thread;
    unsigned index;
} Worker;

static struct {
    _Alignas(128) atomic_ulong epoch;
    _Alignas(128) atomic_uint ready;
    wfb_chunk chunk;
    void *context;
    size_t chunks;
    unsigned width;
    int stopped;
    Worker *workers;
} pool;

static void pause_cpu(void) {
#if defined(__x86_64__)
    __builtin_ia32_pause();
#elif defined(__aarch64__)
    __asm__ volatile("yield");
#else
    atomic_signal_fence(memory_order_seq_cst);
#endif
}

static void run_partition(unsigned index) {
    size_t base = pool.chunks / pool.width;
    size_t extra = pool.chunks % pool.width;
    size_t first = index * base + (index < extra ? index : extra);
    size_t end = first + base + (index < extra ? 1u : 0u);
    for (size_t i = first; i < end; ++i) pool.chunk(pool.context, i);
}

static void *worker_main(void *opaque) {
    Worker *worker = opaque;
    unsigned long seen = 0;
    atomic_fetch_add_explicit(&pool.ready, 1, memory_order_release);
    for (;;) {
        unsigned long current;
        do {
            current = atomic_load_explicit(&pool.epoch, memory_order_acquire);
            if (current == seen) pause_cpu();
        } while (current == seen);
        if (pool.stopped) break;
        run_partition(worker->index);
        atomic_store_explicit(&worker->done, current, memory_order_release);
        seen = current;
    }
    return NULL;
}

static void static_map(const wfb_backend *self, unsigned width, size_t chunks,
                       wfb_chunk fn, void *ctx) {
    unsigned long next;
    (void)self;
    if (width < 1 || width > WFB_MAX_WIDTH) wfb_fail("static: width out of range");
    if (chunks && !fn) wfb_fail("static: missing callback");
    if (pool.stopped) wfb_fail("static: run after shutdown");
    if (pool.width && pool.width != width) wfb_fail("static: width changed");
    if (!pool.width) {
        /* Lazy creation is charged to the first call, like every other
           backend's pool startup. sizeof(Worker) is a multiple of 128 because
           its first member is 128-byte aligned, which is what aligned_alloc
           requires and what gives each completion cell its own line. */
        pool.width = width;
        pool.workers = NULL;
        if (width > 1) {
            pool.workers = aligned_alloc(128, (size_t)(width - 1) * sizeof(Worker));
            if (!pool.workers) wfb_fail("static: worker allocation");
            for (unsigned i = 1; i < width; ++i) {
                Worker *worker = &pool.workers[i - 1];
                atomic_init(&worker->done, 0ul);
                worker->index = i;
                if (pthread_create(&worker->thread, NULL, worker_main, worker))
                    wfb_fail("static: pthread_create");
            }
            while (atomic_load_explicit(&pool.ready, memory_order_acquire) != width - 1)
                pause_cpu();
        }
    }
    pool.chunk = fn;
    pool.context = ctx;
    pool.chunks = chunks;
    next = atomic_load_explicit(&pool.epoch, memory_order_relaxed) + 1;
    if (!next) wfb_fail("static: epoch wrapped");
    atomic_store_explicit(&pool.epoch, next, memory_order_release);
    run_partition(0);
    for (unsigned i = 1; i < width; ++i)
        while (atomic_load_explicit(&pool.workers[i - 1].done, memory_order_acquire) != next)
            pause_cpu();
}

static int static_stop(void) {
    if (pool.stopped) wfb_fail("static: shutdown twice");
    pool.stopped = 1;
    if (pool.workers) {
        atomic_fetch_add_explicit(&pool.epoch, 1, memory_order_release);
        for (unsigned i = 1; i < pool.width; ++i)
            if (pthread_join(pool.workers[i - 1].thread, NULL))
                wfb_fail("static: pthread_join");
        free(pool.workers);
        pool.workers = NULL;
    }
    return 1;
}

/* The grain string names the policy AND its role, because the role is what a
 * reader of the table needs and a truncated policy sentence is what three
 * readers in a row mistook for a broken row: `static` is a strong regular-work
 * reference, not a dynamic-scheduling ceiling for skew, which is exactly why
 * the skewed shape is the timed one. The note carries the other half of the
 * disclosure: in the research bundle this reference's Linux medians carried
 * multi-millisecond excursions with 65-117 involuntary context switches, and
 * excursions are retained here -- there is no outlier removal and no discarded
 * process anywhere in this bundle. */
const wfb_backend wfb_backend_static = {
    "static",
    "equal contiguous partition, persistent helpers, no stealing: a regular-work "
    "reference, not a dynamic-scheduling ceiling for skew",
    0, 0, static_map, NULL, static_stop, "excursions retained",
};
