/* A host smoke run of the scheduler core: real threads, the real switch, the
 * real park. It is not the gate (the enumerator is, design §11); it is the
 * first thing that has to work before an enumeration is worth running, and
 * it exercises the shapes of §0 the staged pipeline never reaches: a
 * straight-line group of I/O records parked on and resumed on foreign
 * threads, a hand-out whose callee parks on I/O, and the entry stack parked
 * on main's first I/O and run to its return by a worker (S17). */

#include "core.h"

#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define WORKERS 3u
#define STACKS 8u
#define STACK_BYTES (256u * 1024u)
#define DEVICE_QUEUE 64u

static wf_sched_core core;

/* ------------------------------------------------------------- a device */

/* One "device": a thread that completes the records it is handed after a
 * short delay, in the order they arrive. It publishes with the one call the
 * drain uses. */
static pthread_mutex_t device_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t device_signal = PTHREAD_COND_INITIALIZER;
typedef struct device_request {
    wf_sched_record *record;
    int require_suspension;
} device_request;
static device_request device_queue[DEVICE_QUEUE];
static unsigned device_head;
static unsigned device_tail;
static int device_stopping;

static void device_submit(wf_sched_record *record, int require_suspension) {
    pthread_mutex_lock(&device_lock);
    if ((device_tail + 1u) % DEVICE_QUEUE == device_head) {
        abort();
    }
    device_queue[device_tail] = (device_request){record, require_suspension};
    device_tail = (device_tail + 1u) % DEVICE_QUEUE;
    pthread_cond_signal(&device_signal);
    pthread_mutex_unlock(&device_lock);
}

static void *device_main(void *argument) {
    (void)argument;
    for (;;) {
        wf_sched_record *record;
        device_request request;
        struct timespec pause = {0, 200000};
        pthread_mutex_lock(&device_lock);
        while (device_head == device_tail && !device_stopping) {
            pthread_cond_wait(&device_signal, &device_lock);
        }
        if (device_head == device_tail) {
            pthread_mutex_unlock(&device_lock);
            return NULL;
        }
        request = device_queue[device_head];
        record = request.record;
        device_head = (device_head + 1u) % DEVICE_QUEUE;
        pthread_mutex_unlock(&device_lock);
        if (request.require_suspension) {
            for (;;) {
                wf_sched_stack *waiter = __atomic_load_n(&record->waiter, __ATOMIC_ACQUIRE);
                if (waiter != NULL && waiter != WF_SCHED_WAITER_IN_PLACE
                    && __atomic_load_n(&waiter->phase, __ATOMIC_ACQUIRE) == WF_SCHED_STACK_SUSPENDED) {
                    break;
                }
                nanosleep(&pause, NULL);
            }
        }
        nanosleep(&pause, NULL);
        wf_sched_complete(&core, record);
    }
}

/* ------------------------------------------------------------ the program */

static unsigned long long io_rounds;
static unsigned long long compute_sum;

/* Hold the other threads outside the core while the caller joins their
 * oldest task. Only the caller can execute the queued siblings; each must
 * run on its existing stack, not on an EMPTY stack borrowed for the join.
 * The last sibling releases the held workers. This witnesses repeated
 * successful helping at the production setting, beyond the model's one turn. */
#define HELP_SIBLINGS 32u
static pthread_mutex_t help_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t help_signal = PTHREAD_COND_INITIALIZER;
static unsigned help_arrived;
static unsigned help_completed;
static int help_wrong_stack;
static wf_sched_stack *help_stack;
static void owned_target_io_roundtrip(void);

static void held_worker(void *frame) {
    (void)frame;
    pthread_mutex_lock(&help_lock);
    help_arrived += 1u;
    pthread_cond_broadcast(&help_signal);
    while (help_completed < HELP_SIBLINGS) {
        pthread_cond_wait(&help_signal, &help_lock);
    }
    pthread_mutex_unlock(&help_lock);
}

static void helped_sibling(void *frame) {
    *(unsigned *)frame = 1u;
    pthread_mutex_lock(&help_lock);
    if (wf_sched_current_stack(&core) != help_stack) {
        help_wrong_stack = 1;
    }
    help_completed += 1u;
    pthread_cond_broadcast(&help_signal);
    pthread_mutex_unlock(&help_lock);
}

static void repeated_current_stack_help(void) {
    void *held[WORKERS];
    void *siblings[HELP_SIBLINGS];
    unsigned index;
    help_stack = wf_sched_current_stack(&core);
    for (index = 0; index < WORKERS; index += 1u) {
        held[index] = wf_sched_acquire(&core, sizeof(unsigned));
        if (held[index] == NULL) abort();
        wf_sched_publish(&core, held[index], held_worker);
    }
    pthread_mutex_lock(&help_lock);
    while (help_arrived < WORKERS) {
        pthread_cond_wait(&help_signal, &help_lock);
    }
    pthread_mutex_unlock(&help_lock);
    owned_target_io_roundtrip();
    for (index = 0; index < HELP_SIBLINGS; index += 1u) {
        siblings[index] = wf_sched_acquire(&core, sizeof(unsigned));
        if (siblings[index] == NULL) abort();
        *(unsigned *)siblings[index] = 0u;
        wf_sched_publish(&core, siblings[index], helped_sibling);
    }
    wf_sched_join_frame(&core, held[0]);
    for (index = 0; index < HELP_SIBLINGS; index += 1u) {
        wf_sched_join_frame(&core, siblings[index]);
        if (*(unsigned *)siblings[index] != 1u) abort();
        wf_sched_release(&core, siblings[index]);
    }
    for (index = 0; index < WORKERS; index += 1u) {
        wf_sched_join_frame(&core, held[index]);
        wf_sched_release(&core, held[index]);
    }
    if (help_wrong_stack || help_completed != HELP_SIBLINGS) {
        (void)fprintf(stderr, "repeated compute help left the current stack\n");
        exit(1);
    }
}

/* One I/O operation: a record in this frame, submitted, then joined. */
static void one_read(int require_suspension) {
    wf_sched_record record;
    wf_sched_record_init(&record);
    device_submit(&record, require_suspension);
    wf_sched_join(&core, &record, 1);
    __atomic_add_fetch(&io_rounds, 1u, __ATOMIC_RELAXED);
}

/* A hand-out whose callee does I/O: it parks on whatever stack it runs on. */
static void read_then_add(void *frame) {
    unsigned long long *cell = frame;
    one_read(0);
    *cell += 1u;
}

static void owned_read_then_add(void *frame) {
    one_read(1);
    *(unsigned long long *)frame += 1u;
}

/* The other workers are held, so this target must take its owner's inline
 * join branch. Its callback still parks on I/O. DONE, result visibility and
 * a repeated join must survive returning through that nested suspension. */
static void owned_target_io_roundtrip(void) {
    wf_sched_statistics before;
    wf_sched_statistics after;
    void *frame = wf_sched_acquire(&core, sizeof(unsigned long long));
    wf_sched_record *record;
    if (frame == NULL) abort();
    record = &wf_sched_slot_of(frame)->record;
    *(unsigned long long *)frame = 0u;
    wf_sched_statistics_sum(&core, &before);
    wf_sched_publish(&core, frame, owned_read_then_add);
    wf_sched_join_frame(&core, frame);
    wf_sched_join_frame(&core, frame);
    wf_sched_statistics_sum(&core, &after);
    if (*(unsigned long long *)frame != 1u || record->state != WF_SCHED_DONE
        || record->waiter != NULL || after.inline_runs != before.inline_runs + 1u
        || after.parks != before.parks + 1u || after.resumes != before.resumes + 1u) {
        (void)fprintf(stderr, "owned inline target lost its result or completion across nested I/O\n");
        exit(1);
    }
    wf_sched_release(&core, frame);
}

/* A group of N hand-outs published together and joined newest first (§4). */
static void hand_out_group(unsigned count) {
    void *frames[8];
    unsigned index;
    if (count > 8u) {
        abort();
    }
    for (index = 0; index < count; index += 1u) {
        frames[index] = wf_sched_acquire(&core, sizeof(unsigned long long));
        if (frames[index] == NULL) {
            /* Refused: the same call runs inline on the owner (§2). */
            unsigned long long cell = 0;
            read_then_add(&cell);
            __atomic_add_fetch(&compute_sum, cell, __ATOMIC_RELAXED);
            continue;
        }
        *(unsigned long long *)frames[index] = 0;
        wf_sched_publish(&core, frames[index], read_then_add);
    }
    while (index > 0u) {
        index -= 1u;
        if (frames[index] == NULL) {
            continue;
        }
        wf_sched_join_frame(&core, frames[index]);
        __atomic_add_fetch(&compute_sum, *(unsigned long long *)frames[index], __ATOMIC_RELAXED);
        wf_sched_release(&core, frames[index]);
    }
}

/* Two I/O records outstanding together in one frame (S18), then a group. */
static void main_body(void *argument) {
    unsigned round;
    (void)argument;
    repeated_current_stack_help();
    for (round = 0; round < 40u; round += 1u) {
        wf_sched_record first;
        wf_sched_record second;
        wf_sched_record_init(&first);
        wf_sched_record_init(&second);
        device_submit(&first, 0);
        device_submit(&second, 0);
        wf_sched_join(&core, &second, 1);
        wf_sched_join(&core, &first, 1);
        __atomic_add_fetch(&io_rounds, 2u, __ATOMIC_RELAXED);
        hand_out_group(4u);
    }
    wf_sched_post_status(&core, 7);
}

static void *worker_main(void *argument) {
    unsigned index = (unsigned)(uintptr_t)argument;
    (void)wf_sched_run(&core, index, NULL, NULL);
    return NULL;
}

int main(void) {
    pthread_t device;
    pthread_t workers[WORKERS];
    unsigned index;
    int status;
    wf_sched_statistics counts;
    /* Initialization must reset live metadata even when its caller supplies
     * reused, nonzero storage rather than the process's pristine BSS. */
    memset(&core, 0xa5, sizeof(core));
    if (wf_sched_init(&core, WORKERS + 1u, STACKS, STACK_BYTES) != 0) {
        (void)fprintf(stderr, "core init failed\n");
        return 1;
    }
    if (pthread_create(&device, NULL, device_main, NULL) != 0) {
        return 1;
    }
    for (index = 0; index < WORKERS; index += 1u) {
        pthread_attr_t attributes;
        if (wf_sched_start_thread(&core, index + 1u) != 0) {
            (void)fprintf(stderr, "no stack for worker %u\n", index + 1u);
            return 1;
        }
        pthread_attr_init(&attributes);
        pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
        if (pthread_create(&workers[index], &attributes, worker_main, (void *)(uintptr_t)(index + 1u)) != 0) {
            return 1;
        }
        pthread_attr_destroy(&attributes);
    }
    status = wf_sched_run(&core, 0, main_body, NULL);
    wf_sched_statistics_sum(&core, &counts);
    pthread_mutex_lock(&device_lock);
    device_stopping = 1;
    pthread_cond_signal(&device_signal);
    pthread_mutex_unlock(&device_lock);
    pthread_join(device, NULL);
    printf(
        "sched smoke: status=%d io_rounds=%llu compute_sum=%llu parks=%llu resumes=%llu "
        "cancels=%llu steals=%llu inline=%llu exhausted_io=%llu exhausted_compute=%llu\n",
        status,
        io_rounds,
        compute_sum,
        counts.parks,
        counts.resumes,
        counts.cancels,
        counts.steals,
        counts.inline_runs,
        counts.exhausted_io_waits,
        counts.exhausted_compute_waits
    );
    if (status != 7 || io_rounds != 1u + 40u * 2u + 40u * 4u || compute_sum != 40u * 4u
        || counts.parks != counts.resumes) {
        (void)fprintf(stderr, "sched smoke: FAIL\n");
        return 1;
    }
    printf("sched smoke: PASS\n");
    return 0;
}
