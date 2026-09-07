#include "fir_static.h"
#include <errno.h>
#include <pthread.h>
#include <stdlib.h>

typedef struct {
    FirNativeKernel kernel;
    const double *prefix;
    const double *taps;
    double *output;
    size_t tap_count, count;
    unsigned active;
} Job;

typedef struct {
    struct FirStatic *pool;
    pthread_t thread;
    unsigned index;
    int pending;
} Worker;

struct FirStatic {
    pthread_mutex_t lock;
    pthread_cond_t work, done;
    Worker workers[3];
    Job job;
    unsigned lanes, ready, remaining;
    int stop;
};

#ifdef FIR_STATIC_TEST
extern int fir_static_test_create(unsigned, pthread_t *, void *(*)(void *), void *);
extern void fir_static_test_before_kernel(unsigned, size_t);
extern void fir_static_test_after_kernel(unsigned, size_t);
extern int fir_static_test_wait(pthread_cond_t *, pthread_mutex_t *);
#endif

static void checked(int result) {
    if (result != 0) abort();
}

static void run_range(const Job *job, unsigned index) {
    size_t base = job->count / job->active;
    size_t extra = job->count % job->active;
    size_t first = (size_t)index * base + (index < extra ? index : extra);
    size_t count = base + (index < extra);
#ifdef FIR_STATIC_TEST
    fir_static_test_before_kernel(index, count);
#endif
    /* Rebase both prefix and output, retaining the complete K-1 history for
     * the first sample in each partition. No samples are copied here. */
    job->kernel(job->prefix + first, job->taps, job->tap_count,
                count, job->output + first);
#ifdef FIR_STATIC_TEST
    fir_static_test_after_kernel(index, count);
#endif
}

static void *worker_main(void *argument) {
    Worker *worker = argument;
    FirStatic *pool = worker->pool;
    checked(pthread_mutex_lock(&pool->lock));
    ++pool->ready;
    checked(pthread_cond_signal(&pool->done));
    for (;;) {
        while (!worker->pending && !pool->stop)
            checked(pthread_cond_wait(&pool->work, &pool->lock));
        if (pool->stop) break;
        Job job = pool->job;
        worker->pending = 0;
        checked(pthread_mutex_unlock(&pool->lock));
        run_range(&job, worker->index);
        checked(pthread_mutex_lock(&pool->lock));
        /* All accesses to borrowed input/output are over before retirement.
         * The owner acquires this mutex before returning or reusing job data. */
        --pool->remaining;
        if (pool->remaining == 0) checked(pthread_cond_signal(&pool->done));
    }
    checked(pthread_mutex_unlock(&pool->lock));
    return NULL;
}

FirStatic *fir_static_create(unsigned requested, FirStaticInfo *info) {
    if ((requested != 1 && requested != 2 && requested != 4) || !info) {
        errno = EINVAL;
        return NULL;
    }
    *info = (FirStaticInfo){requested, 0, 0};
    FirStatic *pool = calloc(1, sizeof(*pool));
    if (!pool) return NULL;
    int error = pthread_mutex_init(&pool->lock, NULL);
    if (error) goto failed_mutex;
    error = pthread_cond_init(&pool->work, NULL);
    if (error) goto failed_work;
    error = pthread_cond_init(&pool->done, NULL);
    if (error) goto failed_done;
    pool->lanes = 1;
    checked(pthread_mutex_lock(&pool->lock));
    for (unsigned index = 1; index < requested; ++index) {
        Worker *worker = &pool->workers[index - 1];
        worker->pool = pool;
        worker->index = index;
#ifdef FIR_STATIC_TEST
        error = fir_static_test_create(index, &worker->thread, worker_main, worker);
#else
        error = pthread_create(&worker->thread, NULL, worker_main, worker);
#endif
        if (error) {
            info->creation_error = error;
            break;
        }
        ++pool->lanes;
    }
    while (pool->ready != pool->lanes - 1)
        checked(pthread_cond_wait(&pool->done, &pool->lock));
    checked(pthread_mutex_unlock(&pool->lock));
    info->actual_lanes = pool->lanes;
    return pool;

failed_done:
    checked(pthread_cond_destroy(&pool->work));
failed_work:
    checked(pthread_mutex_destroy(&pool->lock));
failed_mutex:
    free(pool);
    errno = error;
    return NULL;
}

void fir_static_run(FirStatic *pool, FirNativeKernel kernel,
                    const double *prefix, const double *taps,
                    size_t tap_count, size_t count, double *output) {
    if (count == 0) return;
    unsigned active = count < pool->lanes ? (unsigned)count : pool->lanes;
    Job job = {kernel, prefix, taps, output, tap_count, count, active};
    if (active == 1) {
        run_range(&job, 0);
        return;
    }
    checked(pthread_mutex_lock(&pool->lock));
    pool->job = job;
    pool->remaining = active - 1;
    for (unsigned index = 1; index < active; ++index)
        pool->workers[index - 1].pending = 1;
    checked(pthread_cond_broadcast(&pool->work));
    checked(pthread_mutex_unlock(&pool->lock));
    run_range(&job, 0);
    checked(pthread_mutex_lock(&pool->lock));
    while (pool->remaining != 0)
#ifdef FIR_STATIC_TEST
        checked(fir_static_test_wait(&pool->done, &pool->lock));
#else
        checked(pthread_cond_wait(&pool->done, &pool->lock));
#endif
    checked(pthread_mutex_unlock(&pool->lock));
}

void fir_static_destroy(FirStatic *pool) {
    checked(pthread_mutex_lock(&pool->lock));
    pool->stop = 1;
    checked(pthread_cond_broadcast(&pool->work));
    checked(pthread_mutex_unlock(&pool->lock));
    for (unsigned index = 1; index < pool->lanes; ++index)
        checked(pthread_join(pool->workers[index - 1].thread, NULL));
    checked(pthread_cond_destroy(&pool->done));
    checked(pthread_cond_destroy(&pool->work));
    checked(pthread_mutex_destroy(&pool->lock));
    free(pool);
}
