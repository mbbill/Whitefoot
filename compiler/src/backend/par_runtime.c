/* Whitefoot parallel runtime.
 *
 * The actualization half of the [PAR-1 candidate] permission judgment. The
 * compiler proves, before anything here runs, that the two statements of an
 * overlapped pair touch disjoint storage, carry no external or blocking row,
 * and reach no claim site; this file only decides whether a lane is free and
 * hands the work to it. It never chooses what may overlap.
 *
 * Contract with the emitted module, in the order the module uses it:
 *
 *   void *wf__par_claim(unsigned long bytes)
 *       Takes an idle lane and returns its frame — `bytes` of storage the
 *       caller fills with the handed-out call's arguments and later reads its
 *       result out of — or returns NULL when no lane is idle or no lane's
 *       frame is that large. It never blocks and never runs anything.
 *
 *   void wf__par_publish(void *frame, void (*fn)(void *))
 *       Runs `fn(frame)` on the lane the frame came from. Only a frame a
 *       claim returned reaches it.
 *
 *   void wf__par_join(void *frame)
 *       Blocks until that task has returned, after which the frame holds its
 *       result.
 *
 *   void wf__par_release(void *frame)
 *       Gives the lane back, once the caller has read the result out. The
 *       frame is the caller's until here, which is why the join does not
 *       release: a lane handed on at the join could refill the frame under
 *       the read that follows it.
 *
 * The claim comes first because it is what makes the hand-out cost
 * conditional. The frame lives in the lane rather than in the calling
 * function, so an activation that is never granted a lane never builds one:
 * no stack slot, no argument spills, and the recursion depth of a `--par`
 * build stays the recursion depth of the sequential build.
 *
 * The pool is process-lifetime trusted computing base, like malloc's
 * internals: no Whitefoot construct names it, no Whitefoot value reaches it,
 * and it writes nothing to any output. WF_WORKERS selects the lane count;
 * unset, unparsable, or below two leaves the pool unstarted, so every claim
 * returns NULL and every program runs exactly the sequential schedule it runs
 * today.
 */

#include <pthread.h>
#include <stdlib.h>
#include <sys/resource.h>

/* An upper bound on lanes, so a hostile WF_WORKERS cannot ask for unbounded
 * threads. It is a resource ceiling, not a language constant. */
#define WF_PAR_MAX_WORKERS 64

/* How large a handed-out call's frame a lane can hold: its arguments followed
 * by its result. A call whose frame is larger is simply never granted a lane
 * and runs on the calling thread, which is a schedule the program already has
 * to be correct under. Every argument type the backend emits is a scalar, a
 * pointer, or a small aggregate of those, so this is far above what a real
 * call asks for; it exists so the bound is stated rather than assumed. */
#define WF_PAR_FRAME_BYTES 256

/* One lane. Its own mutex and condition variable carry the whole handshake,
 * so two lanes never contend with each other. */
struct wf__par_worker {
    /* The handed-out call's frame, and the first member: a pointer to the
     * frame is a pointer to the lane, so the emitted module holds one opaque
     * pointer for the whole protocol and never learns this layout. The
     * alignment covers every type the backend puts in a frame, whose widest
     * member is eight bytes. */
    _Alignas(16) unsigned char frame[WF_PAR_FRAME_BYTES];
    pthread_mutex_t lock;
    pthread_cond_t signal;
    void (*run)(void *);
    /* 0 idle, 1 task published, 2 task returned. Guarded by `lock`. */
    int state;
    /* Claimed by a caller. Owned atomically so a claim never blocks. */
    int claimed;
};

/* The lane a frame belongs to. The frame is the first member, so this is the
 * pointer the claim returned, read back as what it always was. */
static struct wf__par_worker *wf__par_lane(void *frame) {
    return (struct wf__par_worker *)frame;
}

/* A worker stack is at least as large as the calling thread's own stack, and
 * never below this floor: a forked task is an ordinary Whitefoot call that may
 * recurse exactly as deep as it would have on the calling thread, so the much
 * smaller pthread default (512 KB on some platforms) would turn a recursion
 * that succeeds sequentially into an overflow on a lane. */
#define WF_PAR_STACK_FLOOR ((size_t)8u * 1024u * 1024u)

/* The calling thread's stack size, or the floor when the platform does not
 * report one. RLIMIT_STACK is the main thread's own limit on every platform
 * this compiler targets; RLIM_INFINITY and absurd values fall back to the
 * floor rather than asking for an unbounded worker stack. */
static size_t wf__par_stack_bytes(void) {
    struct rlimit limit;
    if (getrlimit(RLIMIT_STACK, &limit) == 0 && limit.rlim_cur != RLIM_INFINITY
        && limit.rlim_cur > (rlim_t)WF_PAR_STACK_FLOOR
        && limit.rlim_cur <= (rlim_t)((size_t)512u * 1024u * 1024u)) {
        return (size_t)limit.rlim_cur;
    }
    return WF_PAR_STACK_FLOOR;
}

static struct wf__par_worker wf__par_workers[WF_PAR_MAX_WORKERS];
static int wf__par_worker_count;
static pthread_once_t wf__par_started = PTHREAD_ONCE_INIT;

/* Lanes granted since process start.
 *
 * No Whitefoot construct can name it and no program reads it; it exists so
 * that a measurement, or a gate, can tell a pool that grants lanes from one
 * that silently never does. Without it an overlap test passes just as well
 * against a runtime that refuses everything, which is the one way this could
 * be broken without any test noticing. */
unsigned long wf__par_grants;

static void *wf__par_worker_main(void *opaque) {
    struct wf__par_worker *worker = (struct wf__par_worker *)opaque;
    for (;;) {
        void (*run)(void *);
        pthread_mutex_lock(&worker->lock);
        while (worker->state != 1) {
            pthread_cond_wait(&worker->signal, &worker->lock);
        }
        run = worker->run;
        pthread_mutex_unlock(&worker->lock);

        run(worker->frame);

        pthread_mutex_lock(&worker->lock);
        worker->state = 2;
        pthread_cond_broadcast(&worker->signal);
        pthread_mutex_unlock(&worker->lock);
    }
    return NULL;
}

static int wf__par_requested_workers(void) {
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

static void wf__par_start(void) {
    pthread_attr_t attributes;
    int requested = wf__par_requested_workers();
    int index;
    if (requested < 2) {
        return;
    }
    if (pthread_attr_init(&attributes) != 0) {
        return;
    }
    /* A refused stack request is a silent downgrade to the pthread default,
     * which is exactly the hazard this call exists to close, so it stops the
     * pool instead: no lane is better than a lane that overflows. */
    if (pthread_attr_setstacksize(&attributes, wf__par_stack_bytes()) != 0) {
        pthread_attr_destroy(&attributes);
        return;
    }
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    /* One lane fewer than requested: the calling thread is itself a lane, so
     * `WF_WORKERS=2` means two threads of execution in total. */
    for (index = 0; index < requested - 1; index += 1) {
        pthread_t thread;
        struct wf__par_worker *worker = &wf__par_workers[index];
        if (pthread_mutex_init(&worker->lock, NULL) != 0) {
            break;
        }
        if (pthread_cond_init(&worker->signal, NULL) != 0) {
            pthread_mutex_destroy(&worker->lock);
            break;
        }
        worker->state = 0;
        worker->claimed = 0;
        if (pthread_create(&thread, &attributes, wf__par_worker_main, worker) != 0) {
            pthread_cond_destroy(&worker->signal);
            pthread_mutex_destroy(&worker->lock);
            break;
        }
        wf__par_worker_count = index + 1;
    }
    pthread_attr_destroy(&attributes);
}

void *wf__par_claim(unsigned long bytes) {
    int index;
    pthread_once(&wf__par_started, wf__par_start);
    if (bytes > (unsigned long)WF_PAR_FRAME_BYTES) {
        return NULL;
    }
    for (index = 0; index < wf__par_worker_count; index += 1) {
        struct wf__par_worker *worker = &wf__par_workers[index];
        int expected = 0;
        /* The lane budget: a lane is taken only if it is idle right now.
         * Nothing queues, so a claim never waits and never oversubscribes. */
        if (!__atomic_compare_exchange_n(&worker->claimed, &expected, 1, 0,
                                         __ATOMIC_ACQUIRE, __ATOMIC_RELAXED)) {
            continue;
        }
        return worker->frame;
    }
    return NULL;
}

void wf__par_publish(void *frame, void (*fn)(void *)) {
    struct wf__par_worker *worker = wf__par_lane(frame);
    pthread_mutex_lock(&worker->lock);
    worker->run = fn;
    worker->state = 1;
    pthread_cond_broadcast(&worker->signal);
    pthread_mutex_unlock(&worker->lock);
    __atomic_add_fetch(&wf__par_grants, 1, __ATOMIC_RELAXED);
}

void wf__par_join(void *frame) {
    struct wf__par_worker *worker = wf__par_lane(frame);
    pthread_mutex_lock(&worker->lock);
    while (worker->state != 2) {
        pthread_cond_wait(&worker->signal, &worker->lock);
    }
    worker->state = 0;
    pthread_mutex_unlock(&worker->lock);
}

void wf__par_release(void *frame) {
    __atomic_store_n(&wf__par_lane(frame)->claimed, 0, __ATOMIC_RELEASE);
}
