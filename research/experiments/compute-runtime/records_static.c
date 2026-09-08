#include "records_scheduler.h"
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>

/* Persistent contiguous partition with polling helpers. This is an active
 * CPU-budget reference: idle polling is not free and must be charged in burst
 * or energy comparisons. Each completion cell owns a cache line. */
_Static_assert(ATOMIC_LONG_LOCK_FREE == 2, "static control needs lock-free epochs");
typedef struct {
    _Alignas(128) atomic_ulong done;
    pthread_t thread;
    unsigned index;
} Worker;
static struct {
    _Alignas(128) atomic_ulong epoch;
    _Alignas(128) atomic_uint ready;
    RecordChunk chunk;
    void *context;
    size_t chunks;
    unsigned width;
    int stop;
    Worker workers[3];
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
    size_t end = first + base + (index < extra);
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
        if (pool.stop) break;
        run_partition(worker->index);
        atomic_store_explicit(&worker->done, current, memory_order_release);
        seen = current;
    }
    return NULL;
}

void records_scheduler_run(unsigned width, size_t chunks, RecordChunk chunk, void *context) {
    if ((width != 1 && width != 2 && width != 4) || !chunk || pool.stop || (pool.width && pool.width != width)) abort();
    if (!pool.width) {
        pool.width = width;
        for (unsigned i = 1; i < width; ++i) {
            Worker *worker = &pool.workers[i - 1];
            worker->index = i;
            if (pthread_create(&worker->thread, NULL, worker_main, worker)) abort();
        }
        while (atomic_load_explicit(&pool.ready, memory_order_acquire) != width - 1) pause_cpu();
    }
    pool.chunk = chunk;
    pool.context = context;
    pool.chunks = chunks;
    unsigned long next = atomic_load_explicit(&pool.epoch, memory_order_relaxed) + 1;
    if (!next) abort();
    atomic_store_explicit(&pool.epoch, next, memory_order_release);
    run_partition(0);
    for (unsigned i = 1; i < width; ++i)
        while (atomic_load_explicit(&pool.workers[i - 1].done, memory_order_acquire) != next) pause_cpu();
}

const char *records_scheduler_name(void) { return "static-spin"; }
int records_scheduler_stop(void) {
    if (pool.stop) abort();
    pool.stop = 1;
    atomic_fetch_add_explicit(&pool.epoch, 1, memory_order_release);
    for (unsigned i = 1; i < pool.width; ++i)
        if (pthread_join(pool.workers[i - 1].thread, NULL)) abort();
    return 1;
}
