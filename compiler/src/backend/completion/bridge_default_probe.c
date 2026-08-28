#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

#include "bridge.h"

#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

/*
 * The bridge on its shipped default helper policy, which nothing else runs.
 *
 * `harness.c`'s main pins WF_IO_HELPERS for any invocation that did not name
 * one, and `completion-test` names 0, 1 and 4, so the policy a delivered
 * program actually gets -- a pool that starts empty and grows only once the
 * adapter has measured its own operations waiting, and a positioned read the
 * bridge declines to submit while there is nothing to overlap it with -- was
 * reachable from no test in the tree.  Both halves of that policy are new in
 * batch 0096 and both choose between two different routes for the same
 * writer-visible operation, which is exactly the decision a test has to watch.
 *
 * So this probe runs with WF_IO_HELPERS unset, on purpose, and refuses to run
 * with it set: an inherited value would silently turn it back into the arm the
 * harness already covers.
 *
 * What it asserts of each read is what a writer can observe and nothing else:
 * every read delivers the byte that is at its offset, with the count and error
 * the operation promises, whichever route the bridge chose for it -- and the
 * run finishes, which is the liveness half, since a declined submission and a
 * grown pool are both decisions about who executes the work.
 *
 * It then asserts that the routes it exists to watch were taken at all, which
 * is the difference between covering a decision and counting it: see the
 * route check at the end of `main`.
 */

#define LANES 4
#define ROUNDS 4000
#define FILE_BYTES 4096
#define WATCHDOG_SECONDS 180

static _Atomic int probe_finished;
static _Atomic unsigned submitted_route;
static _Atomic unsigned declined_route;
static _Atomic unsigned nonpositioned_route;

/* A stuck run is the failure this probe most needs to report, and a test that
 * hangs reports nothing.  The bound is far above any honest run of this size;
 * the tick is short so that joining this thread costs the finished run almost
 * nothing. */
static void *watchdog_main(void *context) {
    unsigned ticks = *(unsigned *)context * 10u;
    unsigned elapsed = 0;
    struct timespec tick = {.tv_sec = 0, .tv_nsec = 100000000};
    while (elapsed < ticks) {
        if (atomic_load_explicit(&probe_finished, memory_order_acquire) != 0) {
            return NULL;
        }
        (void)nanosleep(&tick, NULL);
        elapsed += 1;
    }
    if (atomic_load_explicit(&probe_finished, memory_order_acquire) == 0) {
        fprintf(
            stderr,
            "bridge default probe: STUCK after %u s\n",
            ticks / 10u
        );
        fflush(stderr);
        _exit(9);
    }
    return NULL;
}

typedef struct {
    int descriptor;
    unsigned lane;
    int failed;
} lane_context;

/* The byte the fixture holds at `offset`. */
static unsigned char expected_byte(uint64_t offset) {
    return (unsigned char)(offset % 251u);
}

static void *lane_main(void *context) {
    lane_context *self = context;
    unsigned round;
    for (round = 0; round < ROUNDS; ++round) {
        unsigned char token[16];
        unsigned char byte = 0xff;
        uint64_t offset =
            (uint64_t)((self->lane * 7u + round * 13u) % FILE_BYTES);
        int64_t value = 0;
        int error = 0;
        int accepted;
        memset(token, 0, sizeof(token));
        accepted = wf__completion_file_pread_submit(
            self->descriptor,
            &byte,
            1,
            offset,
            token
        );
        if (accepted == 1) {
            atomic_fetch_add_explicit(
                &submitted_route,
                1,
                memory_order_relaxed
            );
            wf__completion_file_join(token, &value, &error);
        } else {
            /* The route the emitter takes for a refused submission: the same
             * host call, at the statement rather than between submit and
             * join. */
            atomic_fetch_add_explicit(
                &declined_route,
                1,
                memory_order_relaxed
            );
            value = wf__completion_file_pread_direct(
                self->descriptor,
                &byte,
                1,
                (int64_t)offset
            );
            error = value < 0 ? errno : 0;
        }
        if (value != 1 || error != 0 || byte != expected_byte(offset)) {
            fprintf(
                stderr,
                "bridge default probe: lane %u round %u offset %llu: "
                "value %lld error %d byte %u expected %u (route %s)\n",
                self->lane,
                round,
                (unsigned long long)offset,
                (long long)value,
                error,
                (unsigned)byte,
                (unsigned)expected_byte(offset),
                accepted == 1 ? "submitted" : "declined"
            );
            self->failed = 1;
            return NULL;
        }
        /* One non-positioned read every so often.  It is never declined, so it
         * puts work in the queue and takes away the decline's `nothing queued`
         * precondition under the other lanes. */
        if ((round & 0x3fu) == 0) {
            unsigned char other = 0;
            memset(token, 0, sizeof(token));
            if (wf__completion_file_read_submit(
                    self->descriptor,
                    &other,
                    1,
                    token
                ) == 1) {
                atomic_fetch_add_explicit(
                    &nonpositioned_route,
                    1,
                    memory_order_relaxed
                );
                wf__completion_file_join(token, &value, &error);
                if (value < 0) {
                    fprintf(
                        stderr,
                        "bridge default probe: non-positioned read %lld "
                        "error %d\n",
                        (long long)value,
                        error
                    );
                    self->failed = 1;
                    return NULL;
                }
            }
        }
    }
    return NULL;
}

int main(int argc, char **argv) {
    char path[512];
    unsigned char fixture[FILE_BYTES];
    int descriptor;
    pthread_t watchdog;
    pthread_t threads[LANES];
    lane_context lanes[LANES];
    unsigned seconds = WATCHDOG_SECONDS;
    size_t index;
    int failed = 0;
    uint64_t ring_submissions;
    uint64_t adapter_submissions;
    unsigned submitted;
    unsigned declined;

    if (argc != 2) {
        fprintf(stderr, "usage: %s SCRATCH_DIRECTORY\n", argv[0]);
        return 2;
    }
    if (getenv("WF_IO_HELPERS") != NULL) {
        fprintf(
            stderr,
            "bridge default probe: WF_IO_HELPERS is set; this probe is about "
            "the route taken when it is not\n"
        );
        return 2;
    }
    for (index = 0; index < FILE_BYTES; ++index) {
        fixture[index] = expected_byte((uint64_t)index);
    }
    if (snprintf(
            path,
            sizeof(path),
            "%s/wf-bridge-default-%ld",
            argv[1],
            (long)getpid()
        ) <= 0) {
        return 2;
    }
    descriptor = open(path, O_RDWR | O_CREAT | O_TRUNC, 0600);
    if (descriptor < 0
        || write(descriptor, fixture, FILE_BYTES) != (ssize_t)FILE_BYTES) {
        fprintf(stderr, "bridge default probe: fixture failed\n");
        return 2;
    }
    if (pthread_create(&watchdog, NULL, watchdog_main, &seconds) != 0) {
        return 2;
    }
    for (index = 0; index < LANES; ++index) {
        lanes[index].descriptor = descriptor;
        lanes[index].lane = (unsigned)index;
        lanes[index].failed = 0;
        if (pthread_create(&threads[index], NULL, lane_main, &lanes[index])
            != 0) {
            return 2;
        }
    }
    for (index = 0; index < LANES; ++index) {
        (void)pthread_join(threads[index], NULL);
    }
    atomic_store_explicit(&probe_finished, 1, memory_order_release);
    (void)pthread_join(watchdog, NULL);
    for (index = 0; index < LANES; ++index) {
        failed |= lanes[index].failed;
    }
    (void)close(descriptor);
    (void)unlink(path);

    /* Which route this host's bridge actually took, and whether the counters
     * agree that both of this probe's reasons for existing were exercised.
     *
     * Counting is not covering.  Without these the probe reports its route
     * split and asserts nothing about it, so a bridge that stopped declining
     * altogether -- or one whose native ring silently stopped accepting --
     * would still print a PASS line, and only a reader comparing two runs by
     * eye would notice.
     *
     * The route is read from the counters rather than from the host name,
     * because a Linux kernel with io_uring disabled runs the POSIX adapter
     * and must be held to the POSIX adapter's promise.  Ring submissions
     * above zero is the native ring; anything else is the adapter, on either
     * host.
     *
     * Both routes owe a submitted positioned read: the first read of the run
     * is measured by nothing, so the decline's WAIT_SHORT precondition cannot
     * hold for it, and the ring accepts unconditionally.  Only the adapter
     * route owes a declined one -- on the ring a positioned read never
     * reaches the decline at all, which is why `declined` is expected to be
     * zero there and is not asserted.
     *
     * `helpers` is reported and not asserted, and that is a statement about
     * the evidence rather than an omission: the pool has grown in no run of
     * this probe on either host, because the queue never outruns it -- four
     * lanes each hold one read at a time.  What the demand-driven pool does
     * when the queue does outrun it is decided by
     * `test_pool_grows_when_operations_wait` and
     * `test_helper_growth_stops_at_the_helper_storage`, which script the
     * clock and block the queue, and measured by the bench runners' cold
     * tables.  This probe covers the decline half and the liveness of both. */
    ring_submissions = wf__completion_linux_io_uring_submissions();
    adapter_submissions = wf__completion_file_fallback_submissions();
    submitted = atomic_load_explicit(&submitted_route, memory_order_relaxed);
    declined = atomic_load_explicit(&declined_route, memory_order_relaxed);
    if (submitted == 0) {
        fprintf(
            stderr,
            "bridge default probe: no positioned read was submitted; the "
            "bridge declined every one of them\n"
        );
        failed = 1;
    }
    if (ring_submissions == 0 && declined == 0) {
        fprintf(
            stderr,
            "bridge default probe: the POSIX adapter route declined no "
            "positioned read; the decline policy this probe exists for did "
            "not fire\n"
        );
        failed = 1;
    }

    if (failed != 0) {
        fprintf(
            stderr,
            "bridge default probe: FAIL submitted=%u declined=%u "
            "non-positioned=%u helpers=%llu submissions=%llu "
            "ring=%llu adapter=%llu\n",
            submitted,
            declined,
            atomic_load_explicit(&nonpositioned_route, memory_order_relaxed),
            (unsigned long long)wf__completion_target_helper_count(),
            (unsigned long long)wf__completion_file_submissions(),
            (unsigned long long)ring_submissions,
            (unsigned long long)adapter_submissions
        );
        return 1;
    }
    fprintf(
        stderr,
        "bridge default probe: PASS submitted=%u declined=%u "
        "non-positioned=%u helpers=%llu submissions=%llu "
        "ring=%llu adapter=%llu route=%s\n",
        submitted,
        declined,
        atomic_load_explicit(&nonpositioned_route, memory_order_relaxed),
        (unsigned long long)wf__completion_target_helper_count(),
        (unsigned long long)wf__completion_file_submissions(),
        (unsigned long long)ring_submissions,
        (unsigned long long)adapter_submissions,
        ring_submissions != 0 ? "native-ring" : "posix-adapter"
    );
    return 0;
}
