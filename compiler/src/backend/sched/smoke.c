/* A host smoke run of the scheduler core: real threads, the real switch, the
 * real park. It is not the gate (the enumerator is, design §11); it is the
 * first thing that has to work before an enumeration is worth running, and
 * it exercises the shapes of §0 the staged pipeline never reaches: a
 * straight-line group of I/O records parked on and resumed on foreign
 * threads, a hand-out whose callee parks on I/O, and the entry stack parked
 * on main's first I/O and run to its return by a worker (S17). */

#include "core.h"
#include "prim.h"

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
static wf_sched_record *device_queue[DEVICE_QUEUE];
static unsigned device_head;
static unsigned device_tail;
static int device_stopping;

static void device_submit(wf_sched_record *record) {
    pthread_mutex_lock(&device_lock);
    if ((device_tail + 1u) % DEVICE_QUEUE == device_head) {
        abort();
    }
    device_queue[device_tail] = record;
    device_tail = (device_tail + 1u) % DEVICE_QUEUE;
    pthread_cond_signal(&device_signal);
    pthread_mutex_unlock(&device_lock);
}

static void *device_main(void *argument) {
    (void)argument;
#if WF_SCHED_LOCAL_WAKE
    /* A helper's default ordinal is zero, but it is not the entry worker. */
    if (wf_prim_thread_index() != 0u || wf_prim_is_core_thread(&core, 0u)) {
        fputs("an unattached helper was mistaken for the entry worker\n", stderr);
        abort();
    }
#endif
    for (;;) {
        wf_sched_record *record;
        struct timespec pause = {0, 200000};
        pthread_mutex_lock(&device_lock);
        while (device_head == device_tail && !device_stopping) {
            pthread_cond_wait(&device_signal, &device_lock);
        }
        if (device_head == device_tail) {
            pthread_mutex_unlock(&device_lock);
            return NULL;
        }
        record = device_queue[device_head];
        device_head = (device_head + 1u) % DEVICE_QUEUE;
        pthread_mutex_unlock(&device_lock);
        nanosleep(&pause, NULL);
        wf_sched_complete(&core, record);
    }
}

/* ------------------------------------------------------------ the program */

static unsigned long long io_rounds;
static unsigned long long compute_sum;

/* One I/O operation: a record in this frame, submitted, then joined. */
static void one_read(void) {
    wf_sched_record record;
#if WF_SCHED_LOCAL_WAKE
    if (!wf_prim_is_core_thread(&core, wf_prim_thread_index())
        || wf_prim_is_core_thread(NULL, wf_prim_thread_index())) {
        fputs("a running scheduler lost its core identity\n", stderr);
        abort();
    }
#endif
    wf_sched_record_init(&record);
    device_submit(&record);
    wf_sched_join(&core, &record, 1);
    __atomic_add_fetch(&io_rounds, 1u, __ATOMIC_RELAXED);
}

/* A hand-out whose callee does I/O: it parks on whatever stack it runs on. */
static void read_then_add(void *frame) {
    unsigned long long *cell = frame;
    one_read();
    *cell += 1u;
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
        wf_sched_publish_staged(&core, frames[index], read_then_add);
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

static void check_prepared_context(const wf_sched_stack *stack) {
    long host_page = sysconf(_SC_PAGESIZE);
    size_t page = host_page > 0 ? (size_t)host_page : 4096u;
    const unsigned char *top = stack->high - sizeof(*stack);
    if ((const unsigned char *)stack->saved_sp < stack->low + page
        || (const unsigned char *)stack->saved_sp >= top
        || (uintptr_t)stack->saved_sp % sizeof(void *) != 0u) {
        fputs("first-use context escaped its guarded stack\n", stderr);
        abort();
    }
}

/* Two I/O records outstanding together in one frame (S18), then a group. */
static void main_body(void *argument) {
    unsigned round;
    (void)argument;
    check_prepared_context(wf_sched_current_stack(&core));
    for (round = 0; round < 40u; round += 1u) {
        wf_sched_record first;
        wf_sched_record second;
        wf_sched_record_init(&first);
        wf_sched_record_init(&second);
        device_submit(&first);
        device_submit(&second);
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
    /* Initialization must establish every live field itself. In the used-lane
     * footprint experiment, inactive lanes retain this poison and must never
     * be consulted by an active worker or a resumed stack. */
    memset(&core, 0xa5, sizeof(core));
    if (wf_sched_init(&core, WORKERS + 1u, STACKS, STACK_BYTES) != 0) {
        (void)fprintf(stderr, "core init failed\n");
        return 1;
    }
    for (index = 0; index < STACKS; index += 1u) {
        const wf_sched_stack *stack = core.stacks[index];
        long host_page = sysconf(_SC_PAGESIZE);
        size_t page = host_page > 0 ? (size_t)host_page : 4096u;
        if ((size_t)(stack->high - stack->low) < STACK_BYTES + page
#if WF_SCHED_COMPACT_STACKS
            || stack != &core.stack_cells[index].value
            || stack->saved_sp != NULL
            || (uintptr_t)stack % 128u != 0u
#else
            || (unsigned char *)stack + sizeof(*stack) != stack->high
            || (unsigned char *)stack->saved_sp < stack->low + page
            || (unsigned char *)stack->saved_sp >= (unsigned char *)stack
            || (uintptr_t)stack % 16u != 0u
#endif
        ) {
            (void)fprintf(stderr, "stack layout lost usable depth or frame bounds\n");
            return 1;
        }
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
        check_prepared_context(core.threads[index + 1u].stack);
        pthread_attr_init(&attributes);
        pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
        if (pthread_create(&workers[index], &attributes, worker_main, (void *)(uintptr_t)(index + 1u)) != 0) {
            return 1;
        }
        pthread_attr_destroy(&attributes);
    }
    status = wf_sched_run(&core, 0, main_body, NULL);
#if WF_SCHED_LOCAL_WAKE
    if (wf_prim_is_core_thread(&core, 0u)) {
        fputs("a returned host thread retained scheduler ownership\n", stderr);
        abort();
    }
#endif
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
    if (status != 7 || io_rounds != 40u * 2u + 40u * 4u || compute_sum != 40u * 4u
        || counts.parks != counts.resumes) {
        (void)fprintf(stderr, "sched smoke: FAIL\n");
        return 1;
    }
#if WF_SCHED_IO_ROUND_ROBIN
    for (index = 0; index < WORKERS + 1u; ++index) {
        if (__atomic_load_n(&core.threads[index].io_started, __ATOMIC_RELAXED) != 40u) {
            fputs("initial I/O hand-outs were not distributed evenly\n", stderr);
            return 1;
        }
    }
#endif
    printf("sched smoke: PASS\n");
    return 0;
}
