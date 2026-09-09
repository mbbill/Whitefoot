/* Native current-stack execution, refusal, startup failure and completion-tail
 * lifetime tests. Includes the delivered core; hooks only hold native threads
 * at otherwise unobservable race windows, never replace the deque operations. */
#include "entry.h"
#include "prim.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static unsigned worker_limit = WF_SCHED_MAX_THREADS;
static int owner_allowed = 1;
static unsigned hold_tail;
static unsigned tail_entered;
static void *tail_frame;
static unsigned held_thief;
static unsigned release_thief;
static unsigned arm_thief;
static unsigned thief_returned;
static _Thread_local int delayed_thief;
static int wf_sched_test_allow_owner_wait(void) { return owner_allowed; }
static int wf_sched_test_allow_worker(unsigned index) { return index < worker_limit; }
static void wf_sched_test_before_steal_read(void) {
    unsigned expected = 1;
    if (__atomic_compare_exchange_n(&arm_thief, &expected, 0, 0,
            __ATOMIC_ACQ_REL, __ATOMIC_RELAXED)) {
        __atomic_store_n(&held_thief, 1, __ATOMIC_RELEASE);
        while (!__atomic_load_n(&release_thief, __ATOMIC_ACQUIRE)) wf_prim_yield();
        delayed_thief = 1;
    }
}
static void wf_sched_test_after_steal(int claimed) {
    if (delayed_thief) {
        if (claimed) abort();
        delayed_thief = 0;
        __atomic_store_n(&thief_returned, 1, __ATOMIC_RELEASE);
    }
}
static void wf_sched_test_after_done(void *frame) {
    if (frame == __atomic_load_n(&tail_frame, __ATOMIC_ACQUIRE)
        && __atomic_load_n(&hold_tail, __ATOMIC_ACQUIRE)) {
        __atomic_store_n(&tail_entered, 1, __ATOMIC_RELEASE);
        while (__atomic_load_n(&hold_tail, __ATOMIC_ACQUIRE)) wf_prim_yield();
    }
}
#include "core.c"

static void check(int value, const char *why) {
    if (!value) { fprintf(stderr, "compute smoke: %s\n", why); abort(); }
}
static uint64_t sum(unsigned depth);
typedef struct { unsigned depth; uint64_t result; } task_frame;
static void task(void *opaque) {
    task_frame *frame = opaque;
    frame->result = sum(frame->depth);
}
static uint64_t sum(unsigned depth) {
    if (!depth) return 1;
    task_frame *frame = wf__par_acquire_lane(sizeof(*frame));
    if (!frame) return sum(depth - 1) + sum(depth - 1);
    frame->depth = depth - 1;
    wf__par_publish(frame, task);
    uint64_t right = sum(depth - 1);
    wf__par_join(frame);
    uint64_t left = frame->result;
    wf__par_release(frame);
    return left + right;
}
static void stamp(void *opaque) { *(uint64_t *)opaque = 42; }

int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "owner-fail") == 0) owner_allowed = 0;
    if (argc > 1 && strcmp(argv[1], "worker-fail") == 0) worker_limit = 1;
    if (argc > 1 && strcmp(argv[1], "partial") == 0) worker_limit = 2;
    wf__runtime_start();
    for (unsigned round = 0; round < 8; ++round)
        check(sum(12) == 4096, "nested tasks lost or duplicated results");
    unsigned workers = wf__sched_pool_running();
    if (!owner_allowed || worker_limit == 1 || !wf__par_pool_active()) {
        check(workers == 0, "failed startup left an active pool");
        check(wf__par_acquire_lane(8) == NULL, "failed startup granted a lane");
        puts("compute smoke: PASS sequential/refused startup");
        return 0;
    }
    check(workers == (worker_limit == 2 ? 1u : 3u), "unexpected actual pool width");
    check(wf__par_acquire_lane(UINT64_MAX) == NULL, "oversized frame was truncated");
    check(wf__par_split_budget(UINT64_MAX, UINT64_MAX) <= 10, "u64 split ABI failed");
    void *frames[WF_SCHED_LANE_SLOTS];
    for (unsigned i = 0; i < WF_SCHED_LANE_SLOTS; ++i) {
        frames[i] = wf__par_acquire_lane(8);
        check(frames[i] != NULL, "slot capacity lost");
        wf__par_publish(frames[i], stamp);
    }
    check(wf__par_acquire_lane(8) == NULL, "full lane did not refuse");
    for (unsigned i = WF_SCHED_LANE_SLOTS; i > 0; --i) {
        wf__par_join(frames[i-1]);
        check(*(uint64_t *)frames[i-1] == 42, "slot result corrupted");
        wf__par_release(frames[i-1]);
    }
    /* Hold a publisher after DONE, then reuse the same frame while its tail
     * is still outstanding. Only permanent atomic metadata may be read. */
    uint64_t *frame = wf__par_acquire_lane(8);
    __atomic_store_n(&tail_frame, frame, __ATOMIC_RELEASE);
    __atomic_store_n(&hold_tail, 1, __ATOMIC_RELEASE);
    wf__par_publish(frame, stamp);
    while (!__atomic_load_n(&tail_entered, __ATOMIC_ACQUIRE)) wf_prim_yield();
    wf__par_join(frame);
    check(*frame == 42, "DONE did not publish payload");
    wf__par_release(frame);
    uint64_t *reused = wf__par_acquire_lane(8);
    check(reused == frame, "test did not reuse retired frame");
    *reused = 99;
    __atomic_store_n(&hold_tail, 0, __ATOMIC_RELEASE);
    wf__par_publish(reused, stamp);
    wf__par_join(reused);
    check(*reused == 42, "completion tail corrupted next generation");
    wf__par_release(reused);
    /* Delay a thief before its ring-cell read across several complete wraps.
     * Its stale CAS must lose, and reading the reused cell must stay atomic. */
    __atomic_store_n(&arm_thief, 1, __ATOMIC_RELEASE);
    frame = wf__par_acquire_lane(8);
    wf__par_publish(frame, stamp);
    while (!__atomic_load_n(&held_thief, __ATOMIC_ACQUIRE)) wf_prim_yield();
    wf__par_join(frame);
    wf__par_release(frame);
    for (unsigned i = 0; i < WF_SCHED_LANE_SLOTS * 8; ++i) {
        frame = wf__par_acquire_lane(8);
        wf__par_publish(frame, stamp);
        wf__par_join(frame);
        check(*frame == 42, "ring-wrap result corrupted");
        wf__par_release(frame);
    }
    __atomic_store_n(&release_thief, 1, __ATOMIC_RELEASE);
    while (!__atomic_load_n(&thief_returned, __ATOMIC_ACQUIRE)) wf_prim_yield();
    check(sum(12) == 4096, "post-wrap nested work failed");
    printf("compute smoke: PASS workers=%u slots=%u grants=%lu\n", workers, WF_SCHED_LANE_SLOTS, wf__par_grants());
    return 0;
}
