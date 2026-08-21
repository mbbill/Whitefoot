/* Whitefoot parallel runtime.
 *
 * The actualization half of the [PAR-1 candidate] permission judgment. The
 * compiler proves, before anything here runs, that the two statements of an
 * overlapped pair touch disjoint storage, carry no external or blocking row,
 * and reach no claim site; this file only decides whether a lane is free and
 * hands the work to it. It never chooses what may overlap.
 *
 * Contract with the emitted module:
 *
 *   void *wf_par_try_fork(void (*fn)(void *), void *arg)
 *       Hands `fn(arg)` to an idle worker and returns a handle, or returns
 *       NULL when no worker is idle. It never blocks and never runs `fn`
 *       itself: a NULL return means the caller runs `fn(arg)` inline.
 *
 *   void wf_par_join(void *handle)
 *       Blocks until the handed-out task has returned. NULL is a no-op.
 *
 * The pool is process-lifetime trusted computing base, like malloc's
 * internals: no Whitefoot construct names it, no Whitefoot value reaches it,
 * and it writes nothing to any output. WF_WORKERS selects the lane count;
 * unset, unparsable, or below two leaves the pool unstarted, so every
 * try_fork returns NULL and every program runs exactly the sequential
 * schedule it runs today.
 */

#include <pthread.h>
#include <stdlib.h>

/* One lane. Its own mutex and condition variable carry the whole handshake,
 * so two lanes never contend with each other. */
struct wf_par_worker {
    pthread_mutex_t lock;
    pthread_cond_t signal;
    void (*run)(void *);
    void *argument;
    /* 0 idle, 1 task published, 2 task returned. Guarded by `lock`. */
    int state;
    /* Claimed by a forker. Owned atomically so try_fork never blocks. */
    int claimed;
};

/* An upper bound on lanes, so a hostile WF_WORKERS cannot ask for unbounded
 * threads. It is a resource ceiling, not a language constant. */
#define WF_PAR_MAX_WORKERS 64

/* Worker stacks match the platform's usual main-thread stack rather than the
 * much smaller pthread default: a forked task is an ordinary Whitefoot call
 * that may recurse exactly as deep as it would have on the calling thread. */
#define WF_PAR_STACK_BYTES (8u * 1024u * 1024u)

static struct wf_par_worker wf_par_workers[WF_PAR_MAX_WORKERS];
static int wf_par_worker_count;
static pthread_once_t wf_par_started = PTHREAD_ONCE_INIT;

static void *wf_par_worker_main(void *opaque) {
    struct wf_par_worker *worker = (struct wf_par_worker *)opaque;
    for (;;) {
        void (*run)(void *);
        void *argument;
        pthread_mutex_lock(&worker->lock);
        while (worker->state != 1) {
            pthread_cond_wait(&worker->signal, &worker->lock);
        }
        run = worker->run;
        argument = worker->argument;
        pthread_mutex_unlock(&worker->lock);

        run(argument);

        pthread_mutex_lock(&worker->lock);
        worker->state = 2;
        pthread_cond_broadcast(&worker->signal);
        pthread_mutex_unlock(&worker->lock);
    }
    return NULL;
}

static int wf_par_requested_workers(void) {
    const char *setting = getenv("WF_WORKERS");
    char *end = NULL;
    long requested;
    if (setting == NULL || setting[0] == '\0') {
        return 0;
    }
    requested = strtol(setting, &end, 10);
    if (end == setting || *end != '\0' || requested < 2) {
        return 0;
    }
    if (requested > WF_PAR_MAX_WORKERS) {
        requested = WF_PAR_MAX_WORKERS;
    }
    return (int)requested;
}

static void wf_par_start(void) {
    pthread_attr_t attributes;
    int requested = wf_par_requested_workers();
    int index;
    if (requested < 2) {
        return;
    }
    if (pthread_attr_init(&attributes) != 0) {
        return;
    }
    pthread_attr_setstacksize(&attributes, (size_t)WF_PAR_STACK_BYTES);
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    /* One lane fewer than requested: the calling thread is itself a lane, so
     * `WF_WORKERS=2` means two threads of execution in total. */
    for (index = 0; index < requested - 1; index += 1) {
        pthread_t thread;
        struct wf_par_worker *worker = &wf_par_workers[index];
        if (pthread_mutex_init(&worker->lock, NULL) != 0) {
            break;
        }
        if (pthread_cond_init(&worker->signal, NULL) != 0) {
            pthread_mutex_destroy(&worker->lock);
            break;
        }
        worker->state = 0;
        worker->claimed = 0;
        if (pthread_create(&thread, &attributes, wf_par_worker_main, worker) != 0) {
            pthread_cond_destroy(&worker->signal);
            pthread_mutex_destroy(&worker->lock);
            break;
        }
        wf_par_worker_count = index + 1;
    }
    pthread_attr_destroy(&attributes);
}

void *wf_par_try_fork(void (*fn)(void *), void *arg) {
    int index;
    pthread_once(&wf_par_started, wf_par_start);
    for (index = 0; index < wf_par_worker_count; index += 1) {
        struct wf_par_worker *worker = &wf_par_workers[index];
        int expected = 0;
        /* The lane budget: a lane is taken only if it is idle right now.
         * Nothing queues, so a fork never waits and never oversubscribes. */
        if (!__atomic_compare_exchange_n(&worker->claimed, &expected, 1, 0,
                                         __ATOMIC_ACQUIRE, __ATOMIC_RELAXED)) {
            continue;
        }
        pthread_mutex_lock(&worker->lock);
        worker->run = fn;
        worker->argument = arg;
        worker->state = 1;
        pthread_cond_broadcast(&worker->signal);
        pthread_mutex_unlock(&worker->lock);
        return worker;
    }
    return NULL;
}

void wf_par_join(void *handle) {
    struct wf_par_worker *worker = (struct wf_par_worker *)handle;
    if (worker == NULL) {
        return;
    }
    pthread_mutex_lock(&worker->lock);
    while (worker->state != 2) {
        pthread_cond_wait(&worker->signal, &worker->lock);
    }
    worker->state = 0;
    pthread_mutex_unlock(&worker->lock);
    __atomic_store_n(&worker->claimed, 0, __ATOMIC_RELEASE);
}
