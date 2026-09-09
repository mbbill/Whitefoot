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
static wf_sched_core core;
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

static void execute(wf_sched_slot *slot) {
    /* This harness gives the executor sole responsibility for retirement;
     * no external joiner holds the frame after publication. */
    wf_sched_execute(&core, slot);
    wf_sched_release(&core, slot->frame);
    (void)__atomic_fetch_add(&completed, 1u, __ATOMIC_RELEASE);
}

static void await_start(void) {
    (void)__atomic_fetch_add(&ready, 1u, __ATOMIC_RELEASE);
    while (!__atomic_load_n(&started, __ATOMIC_ACQUIRE)) {
        wf_prim_pause();
    }
}

static void worker(unsigned index) {
    wf_prim_set_thread_index(index);
    await_start();
    while (!__atomic_load_n(&stopped, __ATOMIC_ACQUIRE)) {
        wf_sched_slot *slot = wf_sched_find(&core, &core.threads[index]);
        if (slot != NULL) {
            execute(slot);
        } else {
            wf_prim_pause();
        }
    }
}

static void observer(void) {
    unsigned long long previous = 0;
    await_start();
    do {
        wf_sched_statistics counts;
        wf_sched_statistics_sum(&core, &counts);
        check(counts.steals >= previous, "live steal count moved backward");
        previous = counts.steals;
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
    wf_sched_statistics counts;
    unsigned returned[WF_SCHED_LANE_SLOTS] = {0};
    unsigned id;
    unsigned index;
    unsigned free_count = 0;
    check(wf_sched_init(&core, PROBE_WORKERS, PROBE_WORKERS + 1u, 65536u) == 0,
          "initialization failed");
    wf_prim_set_thread_index(0);
    for (index = 1; index <= PROBE_WORKERS; index += 1u) {
        threads[index - 1u] = start_thread(index);
    }
    /* Fill once before starting thieves, so remote execution is required. */
    for (id = 0; id < WF_SCHED_LANE_SLOTS; id += 1u) {
        unsigned *frame = wf_sched_acquire(&core, sizeof(*frame));
        check(frame != NULL, "initial lane capacity missing");
        *frame = id;
        wf_sched_publish(&core, frame, task);
    }
    check(wf_sched_acquire(&core, sizeof(unsigned)) == NULL, "full lane did not refuse acquisition");
    while (__atomic_load_n(&ready, __ATOMIC_ACQUIRE) != PROBE_WORKERS) wf_prim_pause();
    __atomic_store_n(&started, 1u, __ATOMIC_RELEASE);
    while (__atomic_load_n(&completed, __ATOMIC_ACQUIRE) == 0u) wf_prim_pause();
    for (; id < PROBE_TASKS; id += 1u) {
        unsigned *frame;
        while ((frame = wf_sched_acquire(&core, sizeof(*frame))) == NULL) {
            wf_sched_slot *slot = wf_sched_pop(&core.lanes[0]);
            if (slot != NULL) execute(slot); else wf_prim_pause();
        }
        *frame = id;
        wf_sched_publish(&core, frame, task);
        if ((id & 3u) == 0u) {
            wf_sched_slot *slot = wf_sched_pop(&core.lanes[0]);
            if (slot != NULL) execute(slot);
        }
    }
    while (__atomic_load_n(&completed, __ATOMIC_ACQUIRE) < PROBE_TASKS) {
        wf_sched_slot *slot = wf_sched_pop(&core.lanes[0]);
        if (slot != NULL) execute(slot); else wf_prim_pause();
    }
    __atomic_store_n(&stopped, 1u, __ATOMIC_RELEASE);
    for (index = 0; index < PROBE_WORKERS; index += 1u) join_thread(threads[index]);
    for (id = 0; id < PROBE_TASKS; id += 1u) check(seen[id] == 1u, "missing task");
    check(completed == PROBE_TASKS, "wrong completion count");
    check(core.lanes[0].top == core.lanes[0].bottom, "deque not empty");
    for (unsigned list = 0; list < 2u; list += 1u) {
        index = list == 0u ? core.lanes[0].free_head : core.lanes[0].local_free_head;
        for (; index != WF_SCHED_NO_SLOT; index = core.lanes[0].slots[index].next_free) {
            check(index < WF_SCHED_LANE_SLOTS, "invalid free-list index");
            check(returned[index]++ == 0u, "free-list cycle or duplicate membership");
            free_count += 1u;
        }
    }
    check(free_count == WF_SCHED_LANE_SLOTS, "slot not returned");
    wf_sched_statistics_sum(&core, &counts);
    check(counts.steals > 0 && counts.steals <= PROBE_TASKS, "invalid final steal count");
    check(observations > 0, "observer never ran");
    (void)printf("sched deque probe: PASS tasks=%u steals=%llu observations=%llu slots=%u\n",
                 completed, counts.steals, observations, free_count);
    return 0;
}
