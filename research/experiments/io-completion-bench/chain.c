/* The four-stage chain of design section 0, in C on io_uring, in the four
 * shapes section 12's fourth item names.
 *
 * `research/investigations/io-model/PARK-ON-MISS.md` section 0 walks one
 * thread through independent iterations of `read -> parse -> request -> write`
 * and shows that a wait tied to the thread's one stack collapses the in-flight
 * depth of every stage after the first to the thread count. Section 12 asks
 * for that claim as numbers: 1000 files, 8 threads, four shapes, the dependent
 * stage's in-flight depth and the wall time.
 *
 * The chain here is that chain. Per job j:
 *
 *   1. read      open file j and read one window at offset 0;
 *   2. parse     fold those bytes, and derive the next offset from the fold,
 *                so stage 3 genuinely depends on stage 1's result;
 *   3. request   read one window at that offset -- the dependent operation,
 *                and the one whose in-flight depth is reported;
 *   4. write     write the job's eight result bytes into the output file at
 *                offset 8j, then close the file.
 *
 * The four shapes differ in one thing only: what a worker does when it joins
 * an operation that has not completed.
 *
 *   nested       it runs another whole job nested on its own stack, which is
 *                what today's compute pool does (`wf__par_wait` helping), and
 *                what buries the ready continuation.
 *   compensate   it hands its runnable permit to another OS thread and blocks,
 *                so the machine keeps `threads` jobs advancing and the waiting
 *                continuations live on threads.
 *   switch       it parks the stack it is on and switches to another, through
 *                the scheduler core of `compiler/src/backend/sched/core.c` --
 *                the shipped design, measured as itself rather than as a
 *                model of itself.
 *   pipeline     the staged lowering as it ships: per thread one lane of K
 *                slots, K first-stage reads outstanding, and the loop blocks
 *                on the oldest slot's join and carries that job to its end
 *                before refilling the slot.
 *
 * One thing is deliberately the same in all four: the ring is driven by one
 * reaper thread that waits on the kernel, reaps every completion and publishes
 * it. That is the drain, it is identical in every shape, and keeping it
 * identical is what makes the four numbers a comparison of the join and of
 * nothing else. Submission is made by the worker that reaches the stage, under
 * the ring's lock, exactly as the Whitefoot bridge submits on the submitting
 * thread.
 *
 * Every shape publishes the same fold over the same jobs, so a shape that
 * skipped work cannot report a time.
 *
 * This is a measurement program and not a gate: it is reached from
 * `research/experiments/park-on-miss-measurements/run.sh` and from this
 * bundle's own Makefile, never from `make check`. */

#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#include "uring_baseline.h"

#include "core.h"

/* The window one stage reads. Smaller than the many-files window on purpose:
 * this table is about when a continuation runs, not about bandwidth. */
#define CHAIN_WINDOW 4096u

/* The most operations that may be outstanding at once, over every shape. The
 * nested and compensating shapes bound it by their thread population; the
 * switch shape bounds it by the stack count; the pipeline bounds it by the
 * slot count times the thread count. */
#define CHAIN_MAX_OPS 4096u

#define CHAIN_MAX_THREADS 512u

/* How deep the nested-helping shape may stack jobs on one thread. */
#define CHAIN_NEST_CEILING 24u

/* How many jobs the switch shape keeps offered at once. */
#define CHAIN_SWITCH_WINDOW 64ul

/* ------------------------------------------------------------- the ring */

static struct wf_bench_ring chain_ring;
static pthread_mutex_t chain_ring_lock = PTHREAD_MUTEX_INITIALIZER;

/* A thread's own sleeping place, for the three shapes that do not use the
 * scheduler core's park. */
struct chain_parker {
    pthread_mutex_t lock;
    pthread_cond_t signal;
    int woken;
};

/* One outstanding operation. `sched` is the record the switch shape's join
 * runs the rule over; `waiter` is where the other three shapes register. */
struct chain_op {
    wf_sched_record sched;
    _Atomic int done;
    _Atomic(struct chain_parker *) waiter;
    int result;
    int dependent;
    uint64_t submitted_ns;
};

static struct chain_op chain_ops[CHAIN_MAX_OPS];

/* The free list of operation handles: one lock, because a handle is taken and
 * returned once per stage and never on the timed inner path of a shape. */
static pthread_mutex_t chain_handle_lock = PTHREAD_MUTEX_INITIALIZER;
static unsigned chain_free_handles[CHAIN_MAX_OPS];
static unsigned chain_free_count;

/* ------------------------------------------------------- what is measured */

static _Atomic unsigned long chain_total_inflight;
static _Atomic unsigned long chain_total_peak;
static _Atomic unsigned long chain_dependent_inflight;
static _Atomic unsigned long chain_dependent_peak;
static _Atomic unsigned long long chain_dependent_dwell_ns;
static _Atomic unsigned long long chain_fold;
static _Atomic unsigned long chain_next_job;
static _Atomic int chain_failed;
static _Atomic unsigned long chain_nest_peak;
static _Atomic unsigned long chain_nest_ceiling_hits;

static unsigned long chain_jobs = 1000ul;
static unsigned long chain_threads = 8ul;
static unsigned long chain_slots = 32ul;
static unsigned long chain_stacks = 72ul;
static int chain_directory = -1;
static int chain_output = -1;

/* The shape now running, chosen once on the command line and never at a
 * branch inside a stage. */
enum chain_shape {
    CHAIN_NESTED = 0,
    CHAIN_COMPENSATE = 1,
    CHAIN_SWITCH = 2,
    CHAIN_PIPELINE = 3
};
static enum chain_shape chain_shape;

static wf_sched_core chain_core;

static uint64_t chain_now(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (uint64_t)now.tv_sec * 1000000000ull + (uint64_t)now.tv_nsec;
}

static void chain_fail(const char *what) {
    (void)fprintf(stderr, "chain: %s: %s\n", what, strerror(errno));
    atomic_store(&chain_failed, 1);
}

/* ------------------------------------------------------- handles and ops */

static unsigned chain_take_handle(void) {
    unsigned handle;
    pthread_mutex_lock(&chain_handle_lock);
    if (chain_free_count == 0u) {
        pthread_mutex_unlock(&chain_handle_lock);
        (void)fprintf(stderr, "chain: more operations outstanding than %u\n", CHAIN_MAX_OPS);
        exit(1);
    }
    handle = chain_free_handles[--chain_free_count];
    pthread_mutex_unlock(&chain_handle_lock);
    return handle;
}

static void chain_give_handle(unsigned handle) {
    pthread_mutex_lock(&chain_handle_lock);
    chain_free_handles[chain_free_count++] = handle;
    pthread_mutex_unlock(&chain_handle_lock);
}

/* Submits one read or write and returns its handle. `dependent` marks the
 * request stage, whose in-flight depth is the number this program exists to
 * report. */
static unsigned chain_submit(int operation, int descriptor, uint64_t offset,
                             unsigned length, unsigned char *buffer, int dependent) {
    unsigned handle = chain_take_handle();
    struct chain_op *op = &chain_ops[handle];
    unsigned tail;
    unsigned position;
    struct io_uring_sqe *entry;
    wf_sched_record_init(&op->sched);
    atomic_store_explicit(&op->done, 0, memory_order_relaxed);
    atomic_store_explicit(&op->waiter, NULL, memory_order_relaxed);
    op->result = 0;
    op->dependent = dependent;
    if (dependent) {
        unsigned long now = atomic_fetch_add(&chain_dependent_inflight, 1ul) + 1ul;
        unsigned long peak = atomic_load(&chain_dependent_peak);
        while (now > peak
               && !atomic_compare_exchange_weak(&chain_dependent_peak, &peak, now)) {
        }
    }
    {
        unsigned long now = atomic_fetch_add(&chain_total_inflight, 1ul) + 1ul;
        unsigned long peak = atomic_load(&chain_total_peak);
        while (now > peak && !atomic_compare_exchange_weak(&chain_total_peak, &peak, now)) {
        }
    }
    op->submitted_ns = chain_now();
    pthread_mutex_lock(&chain_ring_lock);
    tail = atomic_load_explicit((_Atomic unsigned *)chain_ring.submission_tail,
                                memory_order_relaxed);
    position = tail & *chain_ring.submission_mask;
    entry = &chain_ring.entries[position];
    memset(entry, 0, sizeof *entry);
    entry->opcode = (uint8_t)operation;
    entry->fd = descriptor;
    entry->off = offset;
    entry->addr = (unsigned long long)(uintptr_t)buffer;
    entry->len = length;
    entry->user_data = handle;
    chain_ring.submission_array[position] = position;
    atomic_store_explicit((_Atomic unsigned *)chain_ring.submission_tail, tail + 1u,
                          memory_order_release);
    if (syscall(__NR_io_uring_enter, chain_ring.descriptor, 1u, 0u, 0u, NULL, 0) < 0) {
        chain_fail("io_uring_enter");
    }
    pthread_mutex_unlock(&chain_ring_lock);
    return handle;
}

/* -------------------------------------------------------- park and wake */

static void chain_parker_init(struct chain_parker *parker) {
    pthread_mutex_init(&parker->lock, NULL);
    pthread_cond_init(&parker->signal, NULL);
    parker->woken = 0;
}

static void chain_parker_wake(struct chain_parker *parker) {
    pthread_mutex_lock(&parker->lock);
    parker->woken = 1;
    pthread_cond_signal(&parker->signal);
    pthread_mutex_unlock(&parker->lock);
}

static void chain_parker_wait(struct chain_parker *parker) {
    pthread_mutex_lock(&parker->lock);
    while (!parker->woken) {
        pthread_cond_wait(&parker->signal, &parker->lock);
    }
    parker->woken = 0;
    pthread_mutex_unlock(&parker->lock);
}

/* The publish, run by the reaper for every completion. The switch shape
 * publishes through the core; the others store DONE and wake a registered
 * parker, claiming the registration first so a parker that gave up and a
 * publisher that found it cannot both act on it. */
static void chain_publish(struct chain_op *op) {
    if (chain_shape == CHAIN_SWITCH) {
        wf_sched_complete(&chain_core, &op->sched);
        return;
    }
    atomic_store_explicit(&op->done, 1, memory_order_seq_cst);
    {
        struct chain_parker *waiter = atomic_load(&op->waiter);
        if (waiter != NULL
            && atomic_compare_exchange_strong(&op->waiter, &waiter, NULL)) {
            chain_parker_wake(waiter);
        }
    }
}

/* --------------------------------------------------------- the reaper */

static _Atomic int chain_reaper_stop;

static void *chain_reaper_main(void *argument) {
    (void)argument;
    for (;;) {
        unsigned head;
        unsigned tail;
        long entered = syscall(__NR_io_uring_enter, chain_ring.descriptor, 0u, 1u,
                               IORING_ENTER_GETEVENTS, NULL, 0);
        if (entered < 0 && errno != EINTR && errno != ETIME) {
            chain_fail("io_uring_enter reap");
            return NULL;
        }
        head = atomic_load_explicit((_Atomic unsigned *)chain_ring.completion_head,
                                    memory_order_relaxed);
        tail = atomic_load_explicit((_Atomic unsigned *)chain_ring.completion_tail,
                                    memory_order_acquire);
        while (head != tail) {
            struct io_uring_cqe *completion =
                &chain_ring.completions[head & *chain_ring.completion_mask];
            unsigned handle = (unsigned)completion->user_data;
            int result = completion->res;
            head += 1u;
            atomic_store_explicit((_Atomic unsigned *)chain_ring.completion_head, head,
                                  memory_order_release);
            if (handle == (unsigned)-1) {
                return NULL;
            }
            {
                struct chain_op *op = &chain_ops[handle];
                op->result = result;
                atomic_fetch_sub(&chain_total_inflight, 1ul);
                if (op->dependent) {
                    atomic_fetch_add(&chain_dependent_dwell_ns,
                                     chain_now() - op->submitted_ns);
                    atomic_fetch_sub(&chain_dependent_inflight, 1ul);
                }
                chain_publish(op);
            }
            head = atomic_load_explicit((_Atomic unsigned *)chain_ring.completion_head,
                                        memory_order_relaxed);
            tail = atomic_load_explicit((_Atomic unsigned *)chain_ring.completion_tail,
                                        memory_order_acquire);
        }
        if (atomic_load(&chain_reaper_stop) != 0) {
            return NULL;
        }
    }
}

/* The one operation that ends the reaper: a NOP carrying the sentinel handle,
 * submitted after the last job has published its fold. */
static void chain_stop_reaper(void) {
    unsigned tail;
    unsigned position;
    struct io_uring_sqe *entry;
    atomic_store(&chain_reaper_stop, 1);
    pthread_mutex_lock(&chain_ring_lock);
    tail = atomic_load_explicit((_Atomic unsigned *)chain_ring.submission_tail,
                                memory_order_relaxed);
    position = tail & *chain_ring.submission_mask;
    entry = &chain_ring.entries[position];
    memset(entry, 0, sizeof *entry);
    entry->opcode = IORING_OP_NOP;
    entry->user_data = (unsigned long long)(unsigned)-1;
    chain_ring.submission_array[position] = position;
    atomic_store_explicit((_Atomic unsigned *)chain_ring.submission_tail, tail + 1u,
                          memory_order_release);
    (void)syscall(__NR_io_uring_enter, chain_ring.descriptor, 1u, 0u, 0u, NULL, 0);
    pthread_mutex_unlock(&chain_ring_lock);
}

/* ------------------------------------------------------------ the stages */

/* The fold one stage makes of what its read returned. It is over the bytes the
 * kernel actually delivered and not over the whole window, because the tree
 * holds files shorter than a window and a stale window is the one way four
 * shapes reading the same files could publish four different values. */
static uint64_t chain_digest(const unsigned char *bytes, int returned) {
    uint64_t value = 0x9e3779b97f4a7c15ull;
    int at;
    if (returned < 0) {
        returned = 0;
    }
    value = value * 0x100000001b3ull + (uint64_t)(unsigned)returned;
    for (at = 0; at < returned; at += 64) {
        value = value * 0x100000001b3ull + bytes[at];
    }
    return value;
}

/* Every shape's join, apart from the switch shape's, which is the core's.
 *
 * `at_miss` is what the shape does when the record is not done: it returns 1
 * when it found work and ran it, in which case the join re-tests, and 0 when
 * there was nothing to run, in which case this stack has nothing left to do
 * and the join registers and sleeps. A shape with no helping arm passes NULL
 * and always sleeps.
 *
 * The registration is claimed on both sides, so a parker that finds the record
 * done and a publisher that found the registration cannot both act on it. One
 * parker per thread is enough because a helping arm registers nothing: a
 * thread inside `at_miss` has not registered its parker, and the nested join
 * that may register it is the innermost one. */
static void chain_wait(struct chain_op *op, struct chain_parker *parker,
                       int (*at_miss)(void *), void *at_miss_argument) {
    for (;;) {
        if (atomic_load_explicit(&op->done, memory_order_acquire) != 0) {
            return;
        }
        if (at_miss != NULL && at_miss(at_miss_argument) != 0) {
            continue;
        }
        atomic_store(&op->waiter, parker);
        if (atomic_load_explicit(&op->done, memory_order_seq_cst) != 0) {
            struct chain_parker *registered = parker;
            if (atomic_compare_exchange_strong(&op->waiter, &registered, NULL)) {
                return;
            }
            /* The publisher took the registration and will wake this parker. */
        }
        chain_parker_wait(parker);
        return;
    }
}

struct chain_job {
    unsigned long index;
    int descriptor;
    unsigned char *stage_one;
    unsigned char *stage_three;
    unsigned char result[8];
};

/* Stage 1 and stage 3 read into windows the caller owns, because a shape may
 * have many jobs alive on one thread at once. */
static unsigned chain_start_read(struct chain_job *job, uint64_t offset, int dependent) {
    unsigned char *window = dependent ? job->stage_three : job->stage_one;
    return chain_submit(IORING_OP_READ, job->descriptor, offset, CHAIN_WINDOW, window,
                        dependent);
}

/* Evicts what the tree has cached, so that a read in the timed region is a
 * read the device answers and a join is a join that waits. Every shape runs
 * this same pass immediately before its own timed region, because a shape
 * measured on a warm tree measures the completion path and not the wait. */
static int chain_cool_the_tree(void) {
    unsigned long index;
    for (index = 0ul; index < chain_jobs; index += 1ul) {
        char name[32];
        int descriptor;
        (void)snprintf(name, sizeof name, WF_BENCH_NAME_FORMAT, index);
        descriptor = openat(chain_directory, name, O_RDONLY);
        if (descriptor < 0) {
            return 1;
        }
        (void)posix_fadvise(descriptor, 0, 0, POSIX_FADV_DONTNEED);
        close(descriptor);
    }
    return 0;
}

/* Every file is opened once, before the timed region, and the chain is reads
 * and writes from there on.
 *
 * The reason is the one this bundle's README already gives for the read-heavy
 * workload: an `openat` of a cold inode costs more here than the read that
 * follows it, so a chain that opened per job would report the host's open path
 * in every shape and would never let any shape reach the depth this table is
 * about. Section 0's chain is `read -> parse -> request -> write`; the open is
 * not one of its stages. */
static int *chain_descriptors;
static int chain_direct = 1;

static int chain_open_every_file(void) {
    unsigned long index;
    chain_descriptors = malloc((size_t)chain_jobs * sizeof *chain_descriptors);
    if (chain_descriptors == NULL) {
        return 1;
    }
    for (index = 0ul; index < chain_jobs; index += 1ul) {
        char name[32];
        (void)snprintf(name, sizeof name, WF_BENCH_NAME_FORMAT, index);
        chain_descriptors[index] = openat(chain_directory, name, O_RDONLY | O_DIRECT);
        if (chain_descriptors[index] < 0) {
            /* A filesystem that refuses the unbuffered open is measured
             * buffered, and the printed line says which it was, because on a
             * buffered tree a read does not wait and no shape can reach any
             * depth. */
            chain_direct = 0;
            chain_descriptors[index] = openat(chain_directory, name, O_RDONLY);
            if (chain_descriptors[index] < 0) {
                return 1;
            }
        }
    }
    return 0;
}

static int chain_open(struct chain_job *job) {
    job->descriptor = chain_descriptors[job->index];
    return job->descriptor >= 0 ? 0 : 1;
}

static void chain_finish(struct chain_job *job, uint64_t one, uint64_t three) {
    uint64_t value = one ^ (three * 31ull);
    unsigned at;
    for (at = 0; at < 8u; at += 1u) {
        job->result[at] = (unsigned char)(value >> (8u * at));
    }
    atomic_fetch_add(&chain_fold, value * (unsigned long long)(job->index + 1ul));
}

/* --------------------------------------------- shape 1: nested helping */

struct chain_nested_state {
    struct chain_parker parker;
    unsigned depth;
};

static void chain_run_job(struct chain_nested_state *state, unsigned long index);

/* What a nested-helping worker does at a miss: if there is another job left,
 * run it here, above this join, which is the burial section 0 draws. It
 * answers 0 when there is nothing left to run or the nesting has reached its
 * ceiling, and the join then sleeps like every other shape.
 *
 * The ceiling is this program's and not the design's: a nested job holds two
 * windows and a frame on the helping thread's stack, so an unbounded nesting
 * would end the run on the host stack rather than in a table. It is high
 * enough that no run here reached it, which the printed line reports. */
static int chain_nested_miss(void *opaque) {
    struct chain_nested_state *state = opaque;
    unsigned long index;
    if (state->depth >= CHAIN_NEST_CEILING) {
        atomic_fetch_add(&chain_nest_ceiling_hits, 1ul);
        return 0;
    }
    index = atomic_fetch_add(&chain_next_job, 1ul);
    if (index >= chain_jobs) {
        return 0;
    }
    state->depth += 1u;
    {
        unsigned long peak = atomic_load(&chain_nest_peak);
        while ((unsigned long)state->depth > peak
               && !atomic_compare_exchange_weak(&chain_nest_peak, &peak,
                                                (unsigned long)state->depth)) {
        }
    }
    chain_run_job(state, index);
    state->depth -= 1u;
    return 1;
}

static void chain_run_job(struct chain_nested_state *state, unsigned long index) {
    struct chain_job job;
    _Alignas(4096) unsigned char windows[2u * CHAIN_WINDOW];
    unsigned handle;
    uint64_t one;
    uint64_t three;
    uint64_t offset;
    job.index = index;
    job.stage_one = windows;
    job.stage_three = windows + CHAIN_WINDOW;
    if (chain_open(&job) != 0) {
        chain_finish(&job, 0, 0);
        return;
    }
    handle = chain_start_read(&job, 0, 0);
    chain_wait(&chain_ops[handle], &state->parker, chain_nested_miss, state);
    one = chain_digest(job.stage_one, chain_ops[handle].result);
    chain_give_handle(handle);
    offset = (one & 1ull) != 0ull ? (uint64_t)CHAIN_WINDOW : 0ull;
    handle = chain_start_read(&job, offset, 1);
    chain_wait(&chain_ops[handle], &state->parker, chain_nested_miss, state);
    three = chain_digest(job.stage_three, chain_ops[handle].result);
    chain_give_handle(handle);
    chain_finish(&job, one, three);
    handle = chain_submit(IORING_OP_WRITE, chain_output, (uint64_t)index * 8ull, 8u,
                          job.result, 0);
    chain_wait(&chain_ops[handle], &state->parker, chain_nested_miss, state);
    chain_give_handle(handle);
}

static void *chain_nested_thread(void *argument) {
    struct chain_nested_state state;
    (void)argument;
    chain_parker_init(&state.parker);
    state.depth = 0;
    for (;;) {
        unsigned long index = atomic_fetch_add(&chain_next_job, 1ul);
        if (index >= chain_jobs) {
            return NULL;
        }
        chain_run_job(&state, index);
    }
}

/* ---------------------------------------- shape 2: thread compensation */

/* `chain_threads` runnable permits. A worker about to block gives its permit
 * back, so another OS thread starts a job and the machine keeps that many jobs
 * advancing; on its wake it takes a permit again. The waiting continuations
 * live on threads, which is the cost this shape pays and the depth it buys. */
static pthread_mutex_t chain_permit_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t chain_permit_signal = PTHREAD_COND_INITIALIZER;
static unsigned long chain_permits;

static void chain_permit_take(void) {
    pthread_mutex_lock(&chain_permit_lock);
    while (chain_permits == 0ul) {
        pthread_cond_wait(&chain_permit_signal, &chain_permit_lock);
    }
    chain_permits -= 1ul;
    pthread_mutex_unlock(&chain_permit_lock);
}

static void chain_permit_give(void) {
    pthread_mutex_lock(&chain_permit_lock);
    chain_permits += 1ul;
    pthread_cond_signal(&chain_permit_signal);
    pthread_mutex_unlock(&chain_permit_lock);
}

static void chain_compensating_wait(struct chain_op *op, struct chain_parker *parker) {
    if (atomic_load_explicit(&op->done, memory_order_acquire) != 0) {
        return;
    }
    chain_permit_give();
    chain_wait(op, parker, NULL, NULL);
    chain_permit_take();
}

static void *chain_compensate_thread(void *argument) {
    struct chain_parker parker;
    (void)argument;
    chain_parker_init(&parker);
    chain_permit_take();
    for (;;) {
        struct chain_job job;
        _Alignas(4096) unsigned char windows[2u * CHAIN_WINDOW];
        unsigned handle;
        uint64_t one;
        uint64_t three;
        uint64_t offset;
        unsigned long index = atomic_fetch_add(&chain_next_job, 1ul);
        if (index >= chain_jobs) {
            chain_permit_give();
            return NULL;
        }
        job.index = index;
        job.stage_one = windows;
        job.stage_three = windows + CHAIN_WINDOW;
        if (chain_open(&job) != 0) {
            chain_finish(&job, 0, 0);
            continue;
        }
        handle = chain_start_read(&job, 0, 0);
        chain_compensating_wait(&chain_ops[handle], &parker);
        one = chain_digest(job.stage_one, chain_ops[handle].result);
        chain_give_handle(handle);
        offset = (one & 1ull) != 0ull ? (uint64_t)CHAIN_WINDOW : 0ull;
        handle = chain_start_read(&job, offset, 1);
        chain_compensating_wait(&chain_ops[handle], &parker);
        three = chain_digest(job.stage_three, chain_ops[handle].result);
        chain_give_handle(handle);
        chain_finish(&job, one, three);
        handle = chain_submit(IORING_OP_WRITE, chain_output, (uint64_t)index * 8ull, 8u,
                              job.result, 0);
        chain_compensating_wait(&chain_ops[handle], &parker);
        chain_give_handle(handle);
    }
}

/* ------------------------------------------------- shape 3: stack switch */

/* One job as a compute hand-out of the core: the entry thread offers jobs, any
 * thread runs one, and every join inside it runs the core's rule -- so a miss
 * parks the stack and the thread switches to a free one and takes the next
 * job. In-flight depth is bounded by the stack count and by nothing else,
 * which is the design's claim. */
struct chain_switch_frame {
    unsigned long index;
};

static void chain_switch_job(void *frame) {
    struct chain_switch_frame *own = frame;
    struct chain_job job;
    _Alignas(4096) unsigned char windows[2u * CHAIN_WINDOW];
    unsigned handle;
    uint64_t one;
    uint64_t three;
    uint64_t offset;
    job.index = own->index;
    job.stage_one = windows;
    job.stage_three = windows + CHAIN_WINDOW;
    if (chain_open(&job) != 0) {
        chain_finish(&job, 0, 0);
        return;
    }
    handle = chain_start_read(&job, 0, 0);
    wf_sched_join(&chain_core, &chain_ops[handle].sched, 1);
    one = chain_digest(job.stage_one, chain_ops[handle].result);
    chain_give_handle(handle);
    offset = (one & 1ull) != 0ull ? (uint64_t)CHAIN_WINDOW : 0ull;
    handle = chain_start_read(&job, offset, 1);
    wf_sched_join(&chain_core, &chain_ops[handle].sched, 1);
    three = chain_digest(job.stage_three, chain_ops[handle].result);
    chain_give_handle(handle);
    chain_finish(&job, one, three);
    handle = chain_submit(IORING_OP_WRITE, chain_output, (uint64_t)job.index * 8ull, 8u,
                          job.result, 0);
    wf_sched_join(&chain_core, &chain_ops[handle].sched, 1);
    chain_give_handle(handle);
}

/* The entry thread's body: offer jobs, and join the newest outstanding one
 * whenever the lane refuses a slot (section 4's join order). */
static void chain_switch_body(void *argument) {
    void *offered[CHAIN_SWITCH_WINDOW];
    unsigned long outstanding = 0ul;
    unsigned long index;
    (void)argument;
    for (index = 0ul; index < chain_jobs; index += 1ul) {
        void *frame;
        /* The window this body keeps offered. It is not the lane's slot count:
         * this body is a stack that parks, so after a resume it continues on
         * whichever thread took it and offers into that thread's lane, and the
         * offered set is therefore spread over lanes rather than bounded by
         * one of them. */
        while (outstanding == CHAIN_SWITCH_WINDOW) {
            outstanding -= 1ul;
            wf_sched_join_frame(&chain_core, offered[outstanding]);
            wf_sched_release(&chain_core, offered[outstanding]);
        }
        frame = wf_sched_acquire(&chain_core, sizeof(struct chain_switch_frame));
        while (frame == NULL) {
            if (outstanding == 0ul) {
                struct chain_switch_frame own;
                own.index = index;
                chain_switch_job(&own);
                break;
            }
            outstanding -= 1ul;
            wf_sched_join_frame(&chain_core, offered[outstanding]);
            wf_sched_release(&chain_core, offered[outstanding]);
            frame = wf_sched_acquire(&chain_core, sizeof(struct chain_switch_frame));
        }
        if (frame == NULL) {
            continue;
        }
        ((struct chain_switch_frame *)frame)->index = index;
        offered[outstanding] = frame;
        outstanding += 1ul;
        wf_sched_publish(&chain_core, frame, chain_switch_job);
    }
    while (outstanding > 0ul) {
        outstanding -= 1ul;
        wf_sched_join_frame(&chain_core, offered[outstanding]);
        wf_sched_release(&chain_core, offered[outstanding]);
    }
    wf_sched_post_status(&chain_core, 0);
}

static void *chain_switch_worker(void *argument) {
    (void)wf_sched_run(&chain_core, (unsigned)(uintptr_t)argument, NULL, NULL);
    return NULL;
}

/* --------------------------------------------- shape 4: staged pipeline */

/* One lane of K slots per thread, K first-stage reads outstanding, and the
 * loop blocking on the oldest slot's join, which is the lowering the tree
 * ships for a permitted I/O loop. The dependent stages then run one at a time
 * per thread, which is section 0's observation stated as a program. */
struct chain_pipeline_slot {
    struct chain_job job;
    unsigned handle;
    int live;
};

static void *chain_pipeline_thread(void *argument) {
    struct chain_parker parker;
    struct chain_pipeline_slot *slots;
    unsigned char *windows;
    unsigned long oldest = 0ul;
    unsigned long at;
    (void)argument;
    chain_parker_init(&parker);
    slots = calloc(chain_slots, sizeof *slots);
    if (posix_memalign((void **)&windows, 4096u, (size_t)chain_slots * 2u * CHAIN_WINDOW) != 0) {
        windows = NULL;
    }
    if (slots == NULL || windows == NULL) {
        free(slots);
        free(windows);
        (void)fprintf(stderr, "chain: out of memory\n");
        exit(1);
    }
    for (at = 0ul; at < chain_slots; at += 1ul) {
        slots[at].job.stage_one = windows + at * 2u * CHAIN_WINDOW;
        slots[at].job.stage_three = slots[at].job.stage_one + CHAIN_WINDOW;
        slots[at].live = 0;
    }
    for (;;) {
        int any = 0;
        /* Fill every free slot with a first-stage read. */
        for (at = 0ul; at < chain_slots; at += 1ul) {
            unsigned long index;
            if (slots[at].live) {
                any = 1;
                continue;
            }
            index = atomic_fetch_add(&chain_next_job, 1ul);
            if (index >= chain_jobs) {
                continue;
            }
            slots[at].job.index = index;
            if (chain_open(&slots[at].job) != 0) {
                chain_finish(&slots[at].job, 0, 0);
                continue;
            }
            slots[at].handle = chain_start_read(&slots[at].job, 0, 0);
            slots[at].live = 1;
            any = 1;
        }
        if (!any) {
            break;
        }
        /* Block on the oldest live slot and carry that job to its end. */
        for (at = 0ul; at < chain_slots; at += 1ul) {
            unsigned long which = (oldest + at) % chain_slots;
            struct chain_pipeline_slot *slot = &slots[which];
            uint64_t one;
            uint64_t three;
            uint64_t offset;
            unsigned handle;
            if (!slot->live) {
                continue;
            }
            chain_wait(&chain_ops[slot->handle], &parker, NULL, NULL);
            one = chain_digest(slot->job.stage_one, chain_ops[slot->handle].result);
            chain_give_handle(slot->handle);
            offset = (one & 1ull) != 0ull ? (uint64_t)CHAIN_WINDOW : 0ull;
            handle = chain_start_read(&slot->job, offset, 1);
            chain_wait(&chain_ops[handle], &parker, NULL, NULL);
            three = chain_digest(slot->job.stage_three, chain_ops[handle].result);
            chain_give_handle(handle);
            chain_finish(&slot->job, one, three);
            handle = chain_submit(IORING_OP_WRITE, chain_output,
                                  (uint64_t)slot->job.index * 8ull, 8u, slot->job.result, 0);
            chain_wait(&chain_ops[handle], &parker, NULL, NULL);
            chain_give_handle(handle);

            slot->live = 0;
            oldest = (which + 1ul) % chain_slots;
            break;
        }
    }
    free(slots);
    free(windows);
    return NULL;
}

/* ---------------------------------------------------------------- main */

static int chain_parse_shape(const char *name) {
    if (strcmp(name, "nested") == 0) {
        chain_shape = CHAIN_NESTED;
        return 0;
    }
    if (strcmp(name, "compensate") == 0) {
        chain_shape = CHAIN_COMPENSATE;
        return 0;
    }
    if (strcmp(name, "switch") == 0) {
        chain_shape = CHAIN_SWITCH;
        return 0;
    }
    if (strcmp(name, "pipeline") == 0) {
        chain_shape = CHAIN_PIPELINE;
        return 0;
    }
    return 1;
}

int main(int argc, char **argv) {
    pthread_t reaper;
    pthread_t workers[CHAIN_MAX_THREADS];
    unsigned long at;
    uint64_t started;
    uint64_t elapsed;
    unsigned entries;
    if (argc < 4 || argc > 7) {
        (void)fprintf(stderr,
                      "usage: %s TREE SHAPE JOBS [THREADS] [SLOTS] [STACKS]\n"
                      "  SHAPE: nested | compensate | switch | pipeline\n",
                      argv[0]);
        return 2;
    }
    if (chain_parse_shape(argv[2]) != 0) {
        (void)fprintf(stderr, "chain: unknown shape %s\n", argv[2]);
        return 2;
    }
    chain_jobs = strtoul(argv[3], NULL, 10);
    if (argc >= 5) {
        chain_threads = strtoul(argv[4], NULL, 10);
    }
    if (argc >= 6) {
        chain_slots = strtoul(argv[5], NULL, 10);
    }
    if (argc >= 7) {
        chain_stacks = strtoul(argv[6], NULL, 10);
    }
    if (chain_jobs == 0ul || chain_threads == 0ul || chain_threads > CHAIN_MAX_THREADS
        || chain_slots == 0ul || chain_slots > CHAIN_MAX_OPS / 4u || chain_stacks == 0ul
        || chain_stacks > 1024ul) {
        (void)fprintf(stderr, "chain: JOBS, THREADS, SLOTS or STACKS is out of range\n");
        return 2;
    }
    chain_directory = open(argv[1], O_RDONLY | O_DIRECTORY);
    if (chain_directory < 0) {
        chain_fail("open tree");
        return 1;
    }
    {
        char output[4096];
        (void)snprintf(output, sizeof output, "%s/chain.out", argv[1]);
        chain_output = open(output, O_RDWR | O_CREAT | O_TRUNC, 0644);
        if (chain_output < 0) {
            chain_fail("open output");
            return 1;
        }
    }
    entries = 8u;
    while (entries < CHAIN_MAX_OPS) {
        entries *= 2u;
    }
    if (wf_bench_ring_setup(&chain_ring, entries) != 0) {
        (void)fprintf(stderr, "chain: io_uring_setup failed: %s\n", strerror(errno));
        return 1;
    }
    for (at = 0ul; at < CHAIN_MAX_OPS; at += 1ul) {
        chain_free_handles[at] = (unsigned)(CHAIN_MAX_OPS - 1ul - at);
    }
    chain_free_count = CHAIN_MAX_OPS;
    if (pthread_create(&reaper, NULL, chain_reaper_main, NULL) != 0) {
        chain_fail("reaper thread");
        return 1;
    }
    if (chain_open_every_file() != 0) {
        chain_fail("open the tree");
        return 1;
    }
    if (chain_cool_the_tree() != 0) {
        chain_fail("cool the tree");
        return 1;
    }
    started = chain_now();
    if (chain_shape == CHAIN_SWITCH) {
        if (wf_sched_init(&chain_core, (unsigned)chain_threads, (unsigned)chain_stacks,
                          1024u * 1024u)
            != 0) {
            (void)fprintf(stderr, "chain: core init failed\n");
            return 1;
        }
        for (at = 1ul; at < chain_threads; at += 1ul) {
            pthread_attr_t attributes;
            if (wf_sched_start_thread(&chain_core, (unsigned)at) != 0) {
                (void)fprintf(stderr, "chain: no stack for worker %lu\n", at);
                return 1;
            }
            pthread_attr_init(&attributes);
            pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
            if (pthread_create(&workers[at], &attributes, chain_switch_worker,
                               (void *)(uintptr_t)at)
                != 0) {
                chain_fail("worker thread");
                return 1;
            }
            pthread_attr_destroy(&attributes);
        }
        (void)wf_sched_run(&chain_core, 0u, chain_switch_body, NULL);
    } else {
        void *(*body)(void *) = chain_nested_thread;
        unsigned long population = chain_threads;
        if (chain_shape == CHAIN_COMPENSATE) {
            body = chain_compensate_thread;
            chain_permits = chain_threads;
            /* The compensating shape needs threads to compensate with. Its
             * population is the depth it can reach; the permit count is what
             * keeps only `chain_threads` of them runnable. */
            population = chain_threads * 8ul;
            if (population > CHAIN_MAX_THREADS) {
                population = CHAIN_MAX_THREADS;
            }
        } else if (chain_shape == CHAIN_PIPELINE) {
            body = chain_pipeline_thread;
        }
        for (at = 0ul; at < population; at += 1ul) {
            if (pthread_create(&workers[at], NULL, body, (void *)(uintptr_t)at) != 0) {
                chain_fail("worker thread");
                return 1;
            }
        }
        for (at = 0ul; at < population; at += 1ul) {
            pthread_join(workers[at], NULL);
        }
    }
    elapsed = chain_now() - started;
    chain_stop_reaper();
    pthread_join(reaper, NULL);
    if (atomic_load(&chain_failed) != 0) {
        return 1;
    }
    if (chain_shape == CHAIN_SWITCH) {
        wf_sched_statistics counts;
        wf_sched_statistics_sum(&chain_core, &counts);
        printf("chain: core parks=%llu resumes=%llu cancels=%llu line_one=%llu "
               "steals=%llu inline=%llu late_parks=%llu exhausted_io=%llu "
               "exhausted_compute=%llu\n",
               counts.parks, counts.resumes, counts.cancels, counts.line_one,
               counts.steals, counts.inline_runs, counts.late_parks,
               counts.exhausted_io_waits, counts.exhausted_compute_waits);
    }
    printf("chain: shape=%s jobs=%lu threads=%lu slots=%lu stacks=%lu direct=%d wall_ms=%.2f "
           "outstanding_peak=%lu dependent_peak=%lu dependent_mean=%.2f "
           "nest_peak=%lu nest_ceiling=%lu "
           "fold=%020llu\n",
           argv[2], chain_jobs, chain_threads,
           chain_shape == CHAIN_PIPELINE ? chain_slots : 0ul,
           chain_shape == CHAIN_SWITCH ? chain_stacks : 0ul, chain_direct,
           (double)elapsed / 1000000.0, atomic_load(&chain_total_peak),
           atomic_load(&chain_dependent_peak),
           (double)atomic_load(&chain_dependent_dwell_ns) / (double)elapsed,
           atomic_load(&chain_nest_peak), atomic_load(&chain_nest_ceiling_hits),
           (unsigned long long)atomic_load(&chain_fold));
    wf_bench_ring_teardown(&chain_ring);
    close(chain_output);
    close(chain_directory);
    return 0;
}
