#if !defined(__APPLE__)
#define _POSIX_C_SOURCE 200809L
#endif

#include "runtime.h"

#include <assert.h>
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* The current compiler-owned floor supplies real stack sizing and exhaustion
 * handling. Its weak scheduler-entry seam is deliberately left unoverridden. */
int wf__floor_run(int argc, char **argv);

#if defined(WF_COMPUTE_ABI_OBSERVER)

static void report_abi(void) {
    unsigned workers = wf_compute_worker_count();
    assert(wf__par_pool_active() ? workers >= 2 : workers == 0);
    printf("compute ABI: workers=%u steals=%lu\n", workers, wf__par_grants());
}

__attribute__((constructor)) static void register_report(void) {
    assert(atexit(report_abi) == 0);
}

#else

typedef struct {
    uint64_t seed;
    uint64_t result;
    unsigned depth;
} frame_data;

static atomic_uint blocked;
static atomic_int unblock;
static atomic_int steal_stage;
static atomic_int steal_continue;
static _Thread_local int delayed_thief;
static _Thread_local int retired_executor;
static _Atomic(void *) done_frame;
static atomic_int done_stage;
static atomic_int done_continue;
static pthread_t owner;
static uintptr_t owner_address;
static int owner_only;
static unsigned creation_limit = 64;

int wf_compute_test_allow_worker(unsigned index) {
    return index <= creation_limit;
}

static double now(void) {
    struct timespec value;
    assert(clock_gettime(CLOCK_MONOTONIC, &value) == 0);
    return (double)value.tv_sec + (double)value.tv_nsec * 1e-9;
}

static void await_value(atomic_int *value, int expected) {
    double limit = now() + 10.0;
    while (atomic_load(value) != expected) {
        assert(now() < limit);
        sched_yield();
    }
}

void wf_compute_test_before_steal_read(unsigned victim, unsigned long long top) {
    int expected = 1;
    (void)top;
    if (victim == 0 && atomic_compare_exchange_strong(&steal_stage, &expected, 2)) {
        delayed_thief = 1;
        await_value(&steal_continue, 1);
    }
}

void wf_compute_test_after_steal(int claimed) {
    if (delayed_thief) {
        assert(!claimed);
        delayed_thief = 0;
        atomic_store(&steal_stage, 3);
    }
}

void wf_compute_test_after_steal_read(void *observed) {
    /* Consume the pointer before CAS, preserving the source's read in a
     * deliberately broken plain-cell build even if CAS will fail. */
    assert(observed != NULL);
}

void wf_compute_test_after_done(void *frame) {
    if (frame == atomic_load(&done_frame)) {
        retired_executor = 1;
        atomic_store(&done_stage, 1);
        await_value(&done_continue, 1);
    }
}

void wf_compute_test_after_completion_tail(void) {
    if (retired_executor) {
        retired_executor = 0;
        atomic_store(&done_stage, 2);
    }
}

static frame_data read_frame(void *frame) {
    frame_data data;
    memcpy(&data, frame, sizeof(data));
    return data;
}

static void *acquire(uint64_t seed, unsigned depth) {
    frame_data data = {seed, UINT64_MAX, depth};
    void *frame = wf__par_acquire_lane(sizeof(data));
    assert(frame != NULL);
    assert((uintptr_t)frame % 16 == 0);
    memcpy(frame, &data, sizeof(data));
    return frame;
}

static uint64_t reference(uint64_t seed, unsigned depth) {
    if (depth == 0) {
        return seed * 3 + 5;
    }
    return reference(seed + 1, depth - 1) * 7
        + reference(seed + 2, depth - 1) * 11;
}

static void compute(void *frame) {
    frame_data data = read_frame(frame);
    if (owner_only) {
        uintptr_t here = (uintptr_t)&data;
        uintptr_t distance = here > owner_address ? here - owner_address : owner_address - here;
        assert(pthread_equal(pthread_self(), owner));
        assert(distance < 1024 * 1024);
    }
    if (data.depth == 0) {
        data.result = data.seed * 3 + 5;
    } else {
        void *child = wf__par_acquire_lane(sizeof(frame_data));
        frame_data left = {data.seed + 1, 0, data.depth - 1};
        frame_data right = {data.seed + 2, 0, data.depth - 1};
        if (child != NULL) {
            memcpy(child, &left, sizeof(left));
            wf__par_publish(child, compute);
        } else {
            compute(&left);
        }
        compute(&right);
        if (child != NULL) {
            wf__par_join(child);
            left = read_frame(child);
            wf__par_release(child);
        }
        data.result = left.result * 7 + right.result * 11;
    }
    memcpy(frame, &data, sizeof(data));
}

static void hold_worker(void *frame) {
    frame_data data = read_frame(frame);
    assert(!pthread_equal(pthread_self(), owner));
    atomic_fetch_add(&blocked, 1);
    await_value(&unblock, 1);
    data.result = data.seed * 3 + 5;
    memcpy(frame, &data, sizeof(data));
}

static void finish(void *frame, uint64_t seed, unsigned depth) {
    frame_data data;
    wf__par_join(frame);
    data = read_frame(frame);
    assert(data.seed == seed);
    assert(data.depth == depth);
    assert(data.result == reference(seed, depth));
    wf__par_release(frame);
}

static void protocol(void) {
    void *holders[3];
    unsigned workers;
    unsigned index;
    double limit;
    holders[0] = acquire(90, 0);
    workers = wf_compute_worker_count();
    assert(workers >= 2 && workers <= 4);
    wf__par_publish(holders[0], hold_worker);
    for (index = 1; index < workers - 1; ++index) {
        holders[index] = acquire(90 + index, 0);
        wf__par_publish(holders[index], hold_worker);
    }
    limit = now() + 10.0;
    while (atomic_load(&blocked) != workers - 1) {
        assert(now() < limit);
        sched_yield();
    }
    /* Every thief is held, so these are genuine owner-pop and nested calls. */
    owner_only = 1;
    for (index = 0; index < 16; ++index) {
        void *task = acquire(index, 7);
        wf__par_publish(task, compute);
        finish(task, index, 7);
    }
    owner_only = 0;
    atomic_store(&unblock, 1);
    for (index = 0; index < workers - 1; ++index) {
        finish(holders[index], 90 + index, 0);
    }
    assert(wf__par_grants() >= workers - 1);
    assert(wf__par_acquire_lane(257) == NULL);
    for (unsigned round = 0; round < 16; ++round) {
        void *frames[64];
        unsigned capacity = wf_compute_slot_capacity();
        assert(capacity == 64);
        for (index = 0; index < capacity; ++index) {
            frames[index] = acquire((uint64_t)round * 64 + index, 3);
        }
        assert(wf__par_acquire_lane(sizeof(frame_data)) == NULL);
        for (index = 0; index < capacity; ++index) {
            wf__par_publish(frames[index], compute);
        }
        for (index = capacity; index != 0; --index) {
            finish(frames[index - 1], (uint64_t)round * 64 + index - 1, 3);
        }
    }
    puts("compute protocol: owner-pop, foreign workers, nesting, refusal and reuse pass");
}

static void delayed_read(void) {
    void *first = acquire(1, 0);
    assert(wf_compute_worker_count() == 2);
    atomic_store(&steal_stage, 1);
    wf__par_publish(first, compute);
    await_value(&steal_stage, 2);
    finish(first, 1, 0);
    for (unsigned index = 1; index < wf_compute_slot_capacity(); ++index) {
        void *next = acquire(index + 1, 0);
        wf__par_publish(next, compute);
        finish(next, index + 1, 0);
    }
    /* Release precedes the overwrite: no harness happens-before edge orders
     * this reuse against the stale thief's pointer read. Its old CAS must lose. */
    atomic_store(&steal_continue, 1);
    first = acquire(991, 0);
    wf__par_publish(first, compute);
    finish(first, 991, 0);
    await_value(&steal_stage, 3);
    puts("compute deque: delayed thief across ring reuse pass");
}

static void retired_frame(void) {
    void *first = acquire(5, 0);
    void *reused;
    assert(wf_compute_worker_count() == 2);
    atomic_store(&done_frame, first);
    wf__par_publish(first, compute);
    await_value(&done_stage, 1);
    finish(first, 5, 0);
    reused = acquire(123, 0);
    assert(reused == first);
    wf__par_publish(reused, compute);
    finish(reused, 123, 0);
    atomic_store(&done_frame, NULL);
    atomic_store(&done_continue, 1);
    await_value(&done_stage, 2);
    puts("compute completion: result consumed and frame reused before tail pass");
}

int wf__main_body(int argc, char **argv) {
    char marker;
    owner = pthread_self();
    owner_address = (uintptr_t)&marker;
    assert(argc == 2);
    assert(wf__par_pool_active());
    if (strcmp(argv[1], "zero-start") == 0) {
        frame_data data = {13, 0, 7};
        creation_limit = 0;
        assert(wf__par_acquire_lane(sizeof(data)) == NULL);
        assert(wf_compute_worker_count() == 0);
        compute(&data);
        assert(data.result == reference(13, 7));
        puts("compute startup: zero workers takes sequential fallback");
    } else if (strcmp(argv[1], "partial-start") == 0) {
        creation_limit = 1;
        protocol();
        assert(wf_compute_worker_count() == 2);
        puts("compute startup: partial worker creation remains usable");
    } else if (strcmp(argv[1], "protocol") == 0) {
        protocol();
    } else if (strcmp(argv[1], "delayed-read") == 0) {
        delayed_read();
    } else {
        assert(strcmp(argv[1], "retired-frame") == 0);
        retired_frame();
    }
    return 0;
}

int main(int argc, char **argv) {
    return wf__floor_run(argc, argv);
}

#endif
