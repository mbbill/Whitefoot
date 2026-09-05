/* One park and one publish of the scheduler core, timed.
 *
 * Design `research/investigations/io-model/PARK-ON-MISS.md` §12 item 2 asks
 * for the park cost of the handshake of §6 "against the same 2.2 microsecond
 * park-and-wake figure", and the plan's added choices (docs/current-plan.md,
 * "Decided 2026-09-05: measured before chosen") ask for the claim protocol and
 * the memory orders on the same scale. This is that measurement: the round
 * trip of one park, one publish and one resume through `sched/core.c`, over
 * the real host primitives of `sched/prim_host.c`, reported in nanoseconds the
 * way `completion-core-harness` reports `core_roundtrip_ns`.
 *
 * The shape is `sched/smoke.c`'s, cut down to the one event this measures.
 * Two core threads:
 *
 *   thread 0, the entry thread, runs the loop: initialise a record, hand it to
 *     the publisher, and join it as an I/O record. The join misses at line one,
 *     takes the third line, parks this stack on the record and switches to a
 *     free stack, where the scheduler loop finds nothing, captures the wake
 *     epoch and sleeps on it.
 *   thread 1 publishes: it takes the record, waits for the parker's
 *     registration to appear, and calls `wf_sched_complete`, which claims the
 *     registration, makes the parked stack READY and raises the epoch. Thread 0
 *     wakes on the free stack, finds the READY stack at priority 1, switches
 *     back into the join, and the join returns.
 *
 * So one iteration is exactly one park, one publish, one sleep, one wake and
 * two switches, which is the unit §12 compares against the park-and-wake
 * figure. The publisher's wait for the registration is the one thing here that
 * is not the core's: it makes every iteration a real park rather than a
 * line-one read, and it is the same three instructions in every variant, so it
 * cannot move a comparison between two of them. `parks`, `line_one` and
 * `cancels` are printed beside the number so a run that did not park as
 * intended cannot be read as one that did.
 *
 * The publisher is core thread 1 and never enters the scheduler loop, because
 * a second thread in the loop would race the parker to its own READY stack and
 * the number would be a resume-elsewhere measurement instead. It takes its
 * core identity from `wf_prim_set_thread_index`, which is what the per-thread
 * ready list variant needs to have somewhere to push.
 *
 * Not a gate: it is reached from
 * `research/experiments/park-on-miss-measurements/run.sh` and from the
 * `park-on-miss-measurements` target of `compiler/Makefile`, never from
 * `check`. It is removed with the experiment. */

#include "core.h"
#include "prim.h"

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define STACKS 4u
#define STACK_BYTES (256u * 1024u)

static wf_sched_core core;

/* The record in flight between the two threads, and the sentinel that ends
 * the publisher. A plain atomic word: the handshake this file measures is the
 * core's, and this hand-over must not be one. */
static wf_sched_record *volatile handoff;
static wf_sched_record *const stop_sentinel = (wf_sched_record *)(uintptr_t)1;

static unsigned long long iterations = 20000ull;

/* Which of the two points of the park the publisher waits for; see the wait
 * itself for what each measures. */
static int racing;

static uint64_t monotonic_nanoseconds(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (uint64_t)now.tv_sec * 1000000000ull + (uint64_t)now.tv_nsec;
}

static void *publisher_main(void *argument) {
    (void)argument;
    /* The publisher is a thread the core knows, so that a publish which needs
     * the publishing thread's own state -- the per-thread ready list variant
     * does -- has one. It never calls `wf_sched_run`, so it owns no stack and
     * never runs the scheduler loop. */
    wf_prim_set_thread_index(1u);
    for (;;) {
        wf_sched_record *record;
        for (;;) {
            record = (wf_sched_record *)__atomic_load_n(&handoff, __ATOMIC_ACQUIRE);
            if (record != NULL) {
                break;
            }
        }
        __atomic_store_n(&handoff, NULL, __ATOMIC_RELEASE);
        if (record == stop_sentinel) {
            return NULL;
        }
        /* Wait for the park to reach the point this run measures from.
         *
         * `settled` waits for the parked stack to reach SUSPENDED, so the unit
         * timed is one whole park followed by one whole publish: the publisher
         * cannot be inside the park's own window, and the resume finds the
         * READY stack at the scheduler loop's first priority. That is the
         * protocol's own cost, with no sleep in it.
         *
         * `racing` waits only for the registration, which is section 6's window
         * itself: the publisher acts while the parker is between its
         * registration and its commit, so the run takes the NOTIFIED arm and
         * the cancel arm as the host's timing falls, and the parking thread
         * usually reaches the sleep. It is the more expensive number and the
         * one that exercises the claim. */
        for (;;) {
            wf_sched_stack *parked =
                (wf_sched_stack *)__atomic_load_n(&record->waiter, __ATOMIC_ACQUIRE);
            if (parked == NULL) {
                continue;
            }
            if (racing) {
                break;
            }
            if (__atomic_load_n(&parked->phase, __ATOMIC_ACQUIRE)
                == WF_SCHED_STACK_SUSPENDED) {
                break;
            }
        }
        wf_sched_complete(&core, record);
    }
}

static void main_body(void *argument) {
    unsigned long long round;
    (void)argument;
    for (round = 0; round < iterations; round += 1ull) {
        wf_sched_record record;
        wf_sched_record_init(&record);
        __atomic_store_n(&handoff, &record, __ATOMIC_RELEASE);
        wf_sched_join(&core, &record, 1);
    }
    __atomic_store_n(&handoff, stop_sentinel, __ATOMIC_RELEASE);
    wf_sched_post_status(&core, 0);
}

int main(int argc, char **argv) {
    pthread_t publisher;
    uint64_t started;
    uint64_t elapsed;
    wf_sched_statistics counts;
    if (argc > 3) {
        (void)fprintf(stderr, "usage: %s [ITERATIONS] [settled|racing]\n", argv[0]);
        return 2;
    }
    if (argc >= 2) {
        iterations = strtoull(argv[1], NULL, 10);
        if (iterations == 0ull) {
            (void)fprintf(stderr, "park-publish: ITERATIONS must be positive\n");
            return 2;
        }
    }
    if (argc == 3) {
        if (strcmp(argv[2], "racing") == 0) {
            racing = 1;
        } else if (strcmp(argv[2], "settled") != 0) {
            (void)fprintf(stderr, "park-publish: MODE is settled or racing\n");
            return 2;
        }
    }
    if (wf_sched_init(&core, 2u, STACKS, STACK_BYTES) != 0) {
        (void)fprintf(stderr, "park-publish: core init failed\n");
        return 1;
    }
    if (pthread_create(&publisher, NULL, publisher_main, NULL) != 0) {
        (void)fprintf(stderr, "park-publish: no publisher thread\n");
        return 1;
    }
    started = monotonic_nanoseconds();
    (void)wf_sched_run(&core, 0u, main_body, NULL);
    elapsed = monotonic_nanoseconds() - started;
    pthread_join(publisher, NULL);
    wf_sched_statistics_sum(&core, &counts);
    printf(
        "park-publish: mode=%s roundtrip_ns=%llu iterations=%llu parks=%llu "
        "resumes=%llu cancels=%llu line_one=%llu\n",
        racing ? "racing" : "settled",
        (unsigned long long)(elapsed / iterations),
        iterations,
        counts.parks,
        counts.resumes,
        counts.cancels,
        counts.line_one
    );
    if (counts.parks + counts.line_one < iterations) {
        (void)fprintf(stderr, "park-publish: fewer joins than iterations\n");
        return 1;
    }
    return 0;
}
