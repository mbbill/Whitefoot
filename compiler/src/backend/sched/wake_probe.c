/* Posted-before-wait, real wait/wake and empty-deque correctness share the
 * scheduler policy image. The optional "measure" argument retains the
 * calibration samples for manual research; no automatic verdict reads time. */
#if !defined(WF_SCHED_POLICY_COLLECTION)
#if defined(__APPLE__)
#define _DARWIN_C_SOURCE 1
#else
#define _POSIX_C_SOURCE 200809L
#endif
#include "core.c"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>

#define PROBE_LANES 4
#define PROBE_WARMUP 200u
#define PROBE_ROUNDS 2000u
#define PROBE_SPINS 200000u

static uint64_t probe_now_ns(void) {
#if defined(_WIN32)
    LARGE_INTEGER value, frequency;
    if (!QueryPerformanceCounter(&value) || !QueryPerformanceFrequency(&frequency)) abort();
    return (uint64_t)((long double)value.QuadPart * 1000000000 / frequency.QuadPart);
#else
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t) != 0) {
        (void)fprintf(stderr, "park-and-wake probe: clock_gettime failed\n");
        exit(1);
    }
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
#endif
}

static int compare_ns(const void *left, const void *right) {
    uint64_t a = *(const uint64_t *)left;
    uint64_t b = *(const uint64_t *)right;
    return a < b ? -1 : (a > b ? 1 : 0);
}

static uint64_t median_ns(uint64_t *values, unsigned count) {
    qsort(values, count, sizeof values[0], compare_ns);
    return count % 2 ? values[count / 2]
                     : (values[count / 2 - 1] + values[count / 2]) / 2;
}

static uint64_t samples[PROBE_ROUNDS];
static unsigned probe_ready;
static unsigned probe_blocked;
static unsigned probe_preposted;
static unsigned probe_finished;

/* The idle worker's park, verbatim: take the station's lock, sleep while no
 * post has arrived, clear the post, release. Then answer, through the same
 * signal a finishing thief uses. */
static void probe_responder(void *opaque) {
    struct wf__par_lane *self = &wf__par_lanes[1];
    unsigned rounds = *(unsigned *)opaque;
    unsigned round;
    __atomic_store_n(&probe_ready, 1u, __ATOMIC_RELEASE);
    for (round = 0; round < rounds; round += 1) {
        wf_prim_wait_lock(&self->wait);
        if (round == 0 && self->posted) __atomic_store_n(&probe_preposted, 1u, __ATOMIC_RELEASE);
        if (!self->posted) {
            __atomic_fetch_add(&probe_blocked, 1u, __ATOMIC_RELAXED);
        }
        while (!self->posted) {
            wf_prim_wait_sleep(&self->wait);
        }
        self->posted = 0;
        wf_prim_wait_unlock(&self->wait);
        wf__par_signal(&wf__par_lanes[0]);
    }
    __atomic_store_n(&probe_finished, 1u, __ATOMIC_RELEASE);
}

static int wf_probe_wake(int argc, char **argv) {
    int measure = argc == 2 && !strcmp(argv[1], "measure");
    struct wf__par_lane *self = &wf__par_lanes[0];
    unsigned rounds = measure ? PROBE_WARMUP + PROBE_ROUNDS : 2u;
    wf_prim_thread responder;
    unsigned round;
    unsigned spin;
    uint64_t started;
    uint64_t finished;
    uint64_t trip;
    uint64_t half;
    uint64_t spin_total;
    unsigned parked;
    int index;

    for (index = 0; index < PROBE_LANES; index += 1) {
        wf__par_prepare(&wf__par_lanes[index], index);
        if (wf_prim_wait_init(&wf__par_lanes[index].wait) != 0) {
            (void)fprintf(stderr, "park-and-wake probe: no wait station\n");
            return 1;
        }
    }
    __atomic_store_n(&wf__par_lane_count, PROBE_LANES, __ATOMIC_RELAXED);
    wf__par_self = self;

    /* First consume a post already waiting; the second round observes the
     * responder inside its no-post wait before sending the wake. */
    wf__par_signal(&wf__par_lanes[1]);
    if (wf_prim_thread_start(&responder, probe_responder, &rounds, 0) != 0) {
        (void)fprintf(stderr, "park-and-wake probe: no responder thread\n");
        return 1;
    }
    while (__atomic_load_n(&probe_ready, __ATOMIC_ACQUIRE) == 0) {
        wf_prim_yield();
    }

    /* The marker is published while holding the wait mutex; the signal can
     * acquire that mutex only after the responder sleeps. This establishes
     * the second round's schedule, rather than relying on aggregate samples. */
    for (round = 0; round < rounds; round += 1) {
        if (round == 1) {
            while (!__atomic_load_n(&probe_blocked, __ATOMIC_ACQUIRE)) wf_prim_yield();
        }
        started = measure ? probe_now_ns() : 0;
        if (round != 0) wf__par_signal(&wf__par_lanes[1]);
        wf_prim_wait_lock(&self->wait);
        while (!self->posted) {
            wf_prim_wait_sleep(&self->wait);
        }
        self->posted = 0;
        wf_prim_wait_unlock(&self->wait);
        finished = measure ? probe_now_ns() : 0;
        if (measure && round >= PROBE_WARMUP) {
            samples[round - PROBE_WARMUP] = finished - started;
        }
    }

    while (!__atomic_load_n(&probe_finished, __ATOMIC_ACQUIRE)) wf_prim_yield();
    trip = measure ? median_ns(samples, PROBE_ROUNDS) : 0;
    half = trip / 2;

    /* One spin round, uncontended: the same single-victim probe an idle lane
     * runs, over the prepared empty deques of a four-lane pool with nobody
     * else touching them. It is a floor, not the cost under load -- a real
     * spinner contends with owners pushing and popping the very lines it
     * reads -- so a window derived from it understates the time a round count
     * really buys. Stated as a floor for exactly that reason. */
    spin_total = measure ? probe_now_ns() : 0;
    for (spin = 0; spin < (measure ? PROBE_SPINS : PROBE_LANES); spin += 1) {
        if (wf__par_find(self) != NULL) {
            (void)fprintf(stderr, "park-and-wake probe: an empty deque yielded work\n");
            return 1;
        }
    }
    spin_total = measure ? probe_now_ns() - spin_total : 0;

    parked = __atomic_load_n(&probe_blocked, __ATOMIC_RELAXED);
    if (parked == 0 || !__atomic_load_n(&probe_preposted, __ATOMIC_ACQUIRE)) {
        (void)fprintf(stderr, "park-and-wake probe: posted/waiting premise was not observed\n");
        return 1;
    }

    if (measure) {
        (void)printf("park-and-wake probe: lanes=%d rounds=%u parked=%u of %u\n",
                     PROBE_LANES, PROBE_ROUNDS, parked, rounds);
        /* Half a round trip is a proxy, not an isolated host park latency. */
        (void)printf("park-and-wake probe: round_trip_median_ns=%llu half_round_trip_ns=%llu\n",
                     (unsigned long long)trip, (unsigned long long)half);
        (void)printf("park-and-wake probe: spin_round_floor_ns=%.1f over %u rounds\n",
                     (double)spin_total / (double)PROBE_SPINS, PROBE_SPINS);
    }
    (void)printf("park-and-wake probe: PASS\n");
    return 0;
}

#if !defined(WF_SCHED_POLICY_COLLECTION)
int main(int argc, char **argv) { return wf_probe_wake(argc, argv); }
#endif
