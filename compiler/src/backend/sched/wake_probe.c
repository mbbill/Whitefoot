/* Native park-and-wake measurement for this host, through the runtime's own
 * wait station. Include the maintained core so the probe parks and wakes on
 * the delivered `wf__par_signal`, `lane->posted` and `wf_prim_wait_sleep`
 * rather than on a copy of that protocol.
 *
 * It exists to size the spin bound. A lane spins before it parks so that it
 * does not pay for a sleep in order to save less than the sleep costs, and
 * that rule needs two numbers on the host it is applied to: what a park and
 * its wake cost, and what one spin round costs. Both are printed here. This is
 * a measurement and never a timed table row: no threshold is applied to either
 * number, the probe passes on the protocol holding and not on any elapsed
 * time, and nothing in the compiler or the scoreboard reads its output.
 *
 * Keep this probe while a spin bound in the core is a round count justified by
 * a wake cost. */
#if defined(__APPLE__)
#define _DARWIN_C_SOURCE 1
#else
#define _POSIX_C_SOURCE 200809L
#endif

#include "core.c"

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define PROBE_LANES 4
#define PROBE_WARMUP 200u
#define PROBE_ROUNDS 2000u
#define PROBE_SPINS 200000u

static uint64_t probe_now_ns(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t) != 0) {
        (void)fprintf(stderr, "park-and-wake probe: clock_gettime failed\n");
        exit(1);
    }
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
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
}

int main(void) {
    struct wf__par_lane *self = &wf__par_lanes[0];
    unsigned rounds = PROBE_WARMUP + PROBE_ROUNDS;
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

    if (wf_prim_thread_start(&responder, probe_responder, &rounds, 0) != 0) {
        (void)fprintf(stderr, "park-and-wake probe: no responder thread\n");
        return 1;
    }
    while (__atomic_load_n(&probe_ready, __ATOMIC_ACQUIRE) == 0) {
        wf_prim_yield();
    }

    /* One round is: post to the parked responder, park on this station, and be
     * woken by the responder's answer. Both threads park and both are woken
     * once per round, so the round trip is two park-and-wakes and half of it is
     * one. Strict alternation is what makes each side actually sleep; the
     * probe counts the sleeps it really took rather than assuming them. */
    for (round = 0; round < rounds; round += 1) {
        started = probe_now_ns();
        wf__par_signal(&wf__par_lanes[1]);
        wf_prim_wait_lock(&self->wait);
        while (!self->posted) {
            wf_prim_wait_sleep(&self->wait);
        }
        self->posted = 0;
        wf_prim_wait_unlock(&self->wait);
        finished = probe_now_ns();
        if (round >= PROBE_WARMUP) {
            samples[round - PROBE_WARMUP] = finished - started;
        }
    }

    trip = median_ns(samples, PROBE_ROUNDS);
    half = trip / 2;

    /* One spin round, uncontended: the same single-victim probe an idle lane
     * runs, over the prepared empty deques of a four-lane pool with nobody
     * else touching them. It is a floor, not the cost under load -- a real
     * spinner contends with owners pushing and popping the very lines it
     * reads -- so a window derived from it understates the time a round count
     * really buys. Stated as a floor for exactly that reason. */
    spin_total = probe_now_ns();
    for (spin = 0; spin < PROBE_SPINS; spin += 1) {
        if (wf__par_find(self) != NULL) {
            (void)fprintf(stderr, "park-and-wake probe: an empty deque yielded work\n");
            return 1;
        }
    }
    spin_total = probe_now_ns() - spin_total;

    /* A protocol assertion, not a timing one: the station must really have
       parked, or the numbers above are about a signal that never woke anyone.
       How MANY rounds parked is reported and never bounded -- on a loaded host
       the responder can occasionally find its post already waiting, and that is
       a fact about the host, not a failure. */
    parked = __atomic_load_n(&probe_blocked, __ATOMIC_RELAXED);
    if (parked == 0) {
        (void)fprintf(stderr, "park-and-wake probe: no round ever parked\n");
        return 1;
    }

    (void)printf("park-and-wake probe: lanes=%d rounds=%u parked=%u of %u\n",
                 PROBE_LANES, PROBE_ROUNDS, parked, rounds);
    (void)printf("park-and-wake probe: round_trip_median_ns=%llu park_and_wake_ns=%llu\n",
                 (unsigned long long)trip, (unsigned long long)half);
    (void)printf("park-and-wake probe: spin_round_floor_ns=%.1f over %u rounds\n",
                 (double)spin_total / (double)PROBE_SPINS, PROBE_SPINS);
    (void)printf("park-and-wake probe: PASS\n");
    return 0;
}
