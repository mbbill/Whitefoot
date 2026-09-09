/* Native ring reuse and live-counter probe, run by sched-deque-test and CI.
 * Include the maintained core to reach its deque operations without replacing
 * them. Ordinary host stacks let ThreadSanitizer observe real concurrent
 * accesses without the scheduler's custom stack switching. Keep this test
 * while the shared core owns this deque and its slot lifetime protocol. */
#include "core.c"

#include <stdio.h>
#include <stdlib.h>
#if defined(_WIN32)
#include <windows.h>
#else
#include <pthread.h>
#endif

#define PROBE_WORKERS 4u
#define PROBE_TASKS 200000u

static unsigned seen[PROBE_TASKS];
static unsigned completed;
static unsigned ready;
static unsigned started;
static unsigned stopped;
static unsigned long long observations;

static void check(int condition, const char *message) {
    if (!condition) {
        (void)fprintf(stderr, "sched deque probe: %s\n", message);
        abort();
    }
}

static void task(void *frame) {
    unsigned id = *(unsigned *)frame;
    check(id < PROBE_TASKS, "invalid task payload");
    check(__atomic_fetch_add(&seen[id], 1u, __ATOMIC_RELAXED) == 0u,
          "task executed twice");
}

static void execute(struct wf__par_slot *slot) {
    /* Publish completion; the owning lane joins and releases the slot. */
    wf__par_execute(slot);

    (void)__atomic_fetch_add(&completed, 1u, __ATOMIC_RELEASE);
}

static void await_start(void) {
    (void)__atomic_fetch_add(&ready, 1u, __ATOMIC_RELEASE);
    while (!__atomic_load_n(&started, __ATOMIC_ACQUIRE)) {
        wf_prim_yield();
    }
}

static void worker(unsigned index) {
    wf__par_self = &wf__par_lanes[index];
    await_start();
    while (!__atomic_load_n(&stopped, __ATOMIC_ACQUIRE)) {
        struct wf__par_slot *slot = wf__par_find(wf__par_self);
        if (slot != NULL) {
            execute(slot);
        } else {
            wf_prim_yield();
        }
    }
}

static void observer(void) {
    unsigned long long previous = 0;
    await_start();
    do {
        unsigned long long steals = wf__par_grants();
        check(steals >= previous, "live steal count moved backward");
        previous = steals;
        observations += 1u;
    } while (!__atomic_load_n(&stopped, __ATOMIC_ACQUIRE));
}

#if defined(_WIN32)
typedef HANDLE probe_thread;
static DWORD WINAPI thread_main(LPVOID argument) {
    unsigned index = (unsigned)(uintptr_t)argument;
    if (index == PROBE_WORKERS) observer(); else worker(index);
    return 0;
}
static probe_thread start_thread(unsigned index) {
    HANDLE thread = CreateThread(NULL, 0, thread_main, (void *)(uintptr_t)index, 0, NULL);
    check(thread != NULL, "CreateThread failed");
    return thread;
}
static void join_thread(probe_thread thread) {
    check(WaitForSingleObject(thread, INFINITE) == WAIT_OBJECT_0, "thread wait failed");
    check(CloseHandle(thread) != 0, "CloseHandle failed");
}
#else
typedef pthread_t probe_thread;
static void *thread_main(void *argument) {
    unsigned index = (unsigned)(uintptr_t)argument;
    if (index == PROBE_WORKERS) observer(); else worker(index);
    return NULL;
}
static probe_thread start_thread(unsigned index) {
    pthread_t thread;
    check(pthread_create(&thread, NULL, thread_main, (void *)(uintptr_t)index) == 0,
          "pthread_create failed");
    return thread;
}
static void join_thread(probe_thread thread) {
    check(pthread_join(thread, NULL) == 0, "pthread_join failed");
}
#endif

int main(void) {
    probe_thread threads[PROBE_WORKERS];
    unsigned returned[WF_PAR_LANE_SLOTS] = {0};
    void *frames[WF_PAR_LANE_SLOTS];
    unsigned free_count = 0;
    for (unsigned i = 0; i < PROBE_WORKERS; ++i) {
        wf__par_prepare(&wf__par_lanes[i], (int)i);
        check(wf_prim_wait_init(&wf__par_lanes[i].wait) == 0, "wait initialization failed");
    }
    wf__par_self = &wf__par_lanes[0];
    __atomic_store_n(&wf__par_lane_count, PROBE_WORKERS, __ATOMIC_RELAXED);
    for (unsigned i = 1; i <= PROBE_WORKERS; ++i) threads[i-1] = start_thread(i);
    unsigned id = 0;
    while (id < PROBE_TASKS) {
        unsigned batch = PROBE_TASKS - id;
        if (batch > WF_PAR_LANE_SLOTS) batch = WF_PAR_LANE_SLOTS;
        for (unsigned i = 0; i < batch; ++i) {
            unsigned *frame = wf__par_acquire_lane(sizeof(*frame));
            check(frame != NULL, "lane capacity missing");
            *frame = id++;
            frames[i] = frame;
            wf__par_publish(frame, task);
        }
        if (batch == WF_PAR_LANE_SLOTS)
            check(wf__par_acquire_lane(sizeof(unsigned)) == NULL, "full lane did not refuse");
        if (!__atomic_load_n(&started, __ATOMIC_RELAXED)) {
            while (__atomic_load_n(&ready, __ATOMIC_ACQUIRE) != PROBE_WORKERS) wf_prim_yield();
            __atomic_store_n(&started, 1u, __ATOMIC_RELEASE);
            /* Require real remote execution before the owner starts helping. */
            while (__atomic_load_n(&completed, __ATOMIC_ACQUIRE) == 0) wf_prim_yield();
        }
        for (unsigned i = batch; i > 0; --i) {
            wf__par_join(frames[i-1]);
            wf__par_release(frames[i-1]);
        }
    }
    __atomic_store_n(&stopped, 1u, __ATOMIC_RELEASE);
    for (unsigned i = 0; i < PROBE_WORKERS; ++i) join_thread(threads[i]);
    for (id = 0; id < PROBE_TASKS; ++id) check(seen[id] == 1, "missing task");
    struct wf__par_lane *lane = &wf__par_lanes[0];
    check(lane->top == lane->bottom, "deque not empty");
    for (int i = lane->free_head; i >= 0; i = lane->slots[i].next_free) {
        check((unsigned)i < WF_PAR_LANE_SLOTS, "invalid free-list index");
        check(returned[i]++ == 0, "free-list cycle");
        ++free_count;
    }
    check(free_count == WF_PAR_LANE_SLOTS, "slot not returned");
    unsigned long long steals = wf__par_grants();
    check(WF_SCHED_STATS ? steals > 0 && steals <= PROBE_TASKS : steals == 0,
          "invalid steal count");
    check(observations > 0, "observer never ran");
    printf("sched deque probe: PASS tasks=%u steals=%llu observations=%llu slots=%u\n",
           PROBE_TASKS, steals, observations, free_count);
    return 0;
}
