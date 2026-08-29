/* The retirement ledger with a waiter on every route at once.
 *
 * The ledger's give-up decision is process-wide, and its inputs change from
 * threads that are not the waiter's: an operation ending, an operation
 * accepted, a waiter registering, a waiter leaving.  Each of those can turn
 * another waiter's answer from "keep waiting" into "attempt and publish", so
 * each of them owes that waiter a wake — and waiters do not all sleep in the
 * same place.  A refused open on the bounded adapter or the ring sleeps on the
 * ledger's own condition variable; one inside a blocking direct call parks on
 * the runtime endpoint, because it may be the only thread that can reap the
 * completion it is waiting for.  A transition announced on one of them only
 * stops the process: measured before the ledger announced on both, this shape
 * stopped five runs in twenty at one helper and nineteen in twenty at four,
 * with a direct-route waiter parked on an answer a second waiter had already
 * changed, at zero CPU, for as long as it was left alive.  Built against that
 * revision this probe publishes fewer opens than source order owes as well —
 * one run in twelve at one helper, seven in thirty at four — so it answers
 * either way, and the watchdog is for the way that never returns.
 *
 * The shape is the smallest one that puts a waiter in both places with a
 * return to argue over: three held descriptors, three closes the host refuses
 * interleaved with three it performs, four submitted opens and one direct-route
 * open on a thread of the program's own.  Source order frees exactly three
 * descriptors among five opens, so three of them are owed an `Ok`.
 *
 * It is a probe of its own rather than a harness case because it narrows its
 * own `RLIMIT_NOFILE` to make the host refuse an open, which the harness
 * process cannot do without changing every case that runs after it, and
 * because the schedule needs a filesystem whose `IORING_OP_CLOSE` the kernel
 * cannot run inline.  On `ext4` the ring runs a close inline and the window
 * this is about never opens; on `overlayfs`, whose files have a `flush`, the
 * close is punted to a worker and the window is real.  So the probe mounts one
 * where it can, and says plainly when it cannot rather than passing quietly.
 *
 * Two refusals are not the same question, and the probe separates them rather
 * than counting both as a lost `Ok`.  `EMFILE` is the refusal this repetition
 * arranged, out of the three descriptors it holds and frees; anything else —
 * `ENFILE`, the machine's own file table, or `ENOMEM` — refuses an open that
 * those three descriptors would have satisfied, so the repetition never asked
 * the question and is reported as not asked and not counted.  A shortfall
 * whose every refusal is `EMFILE` stays a hard failure, because that is the
 * schedule this probe exists to catch.  The failure line carries what a reader
 * needs to act on it: each open's errno, how many descriptors this runtime
 * gave back during the repetition, how many re-attempts the ledger granted,
 * and whether the host would still hand out a descriptor once every open had
 * published — a shortfall with one left over is a return that this runtime
 * awarded and no open took.  A repetition that meets both at once — a genuine
 * `EMFILE` shortfall and a refusal from outside the narrowed table — is
 * dropped as not asked, because the premise it rests on did not hold; that is
 * the strictest edge of the classification and it is announced per repetition
 * and counted on the passing line as well as the failing one.
 *
 * Beside the count of published opens it checks the ledger's own promise, which
 * is what the count alone cannot see.  One re-attempt per open is the whole of
 * the rule, so five opens can be granted at most five; and every refusal that
 * reached publication was granted one, so more refusals than re-attempts is a
 * refusal published without the second attempt it was owed.  A repetition that
 * ends with a spare descriptor beside a published refusal is a lost `Ok`
 * whatever the count says, and is reported as one.
 *
 * It carries its own watchdog: a stall here is a process that never returns,
 * and a gate that hangs reports nothing.  The watchdog prints what the ledger
 * held and leaves with a status of its own. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>
#if defined(__linux__)
#include <sys/mount.h>
#endif

#include "bridge.h"
#include "contract.h"
#include "file_adapter.h"

/* Repetitions, and the descriptors and opens one repetition uses.  The counts
 * are the ones the schedule needs: three returns to award among five opens on
 * two routes, and enough repetitions that a schedule seen four times in a
 * thousand is not a rarity here. */
#define WF_INTERLEAVE_REPETITIONS 250
#define WF_INTERLEAVE_HELD 3
#define WF_INTERLEAVE_OPENS 4
/* How long one repetition may take before this is a stall rather than slow
 * work.  A repetition is a handful of host calls; a stalled one never ends. */
#define WF_INTERLEAVE_WATCHDOG_SECONDS 60

#define WF_INTERLEAVE_SKIP 77

/* Everything below through the overlay helpers exists for the Linux run
 * only: on a host with no kernel completion ring `main` skips before
 * touching any of it, and a definition it cannot reach is an unused-symbol
 * error under the gate's -Werror. */
#if defined(__linux__)
static char wf_interleave_path[384];
static _Atomic unsigned wf_interleave_progress;
static _Atomic int wf_interleave_finished;
static _Atomic int wf_interleave_gate;
static _Atomic int wf_interleave_direct_value;
static _Atomic int wf_interleave_direct_error;

/* What the repetition just run asked of the host and what it got.  A shortfall
 * here is a defect report, and a report that says only how many opens
 * published is not enough to act on: the reader has to be able to tell a
 * schedule that lost a return from a host that never offered three.  Written
 * and read on the loop's own thread, between repetitions. */
static int wf_interleave_open_error[WF_INTERLEAVE_OPENS];
/* The errno of a refusal the narrowed table did not cause, or zero. */
static int wf_interleave_outside_error;
/* Whether the host would still hand this process a descriptor once every open
 * of the repetition had published.  A shortfall with a descriptor left over is
 * a return this runtime awarded and no open took. */
static int wf_interleave_left_over;
/* How many of the five opens published the refusal this repetition arranges.
 * Two of them always do — five opens for three descriptors — and every one of
 * them reached publication through a re-attempt the ledger granted, which is
 * what the accounting below checks the ledger against. */
static int wf_interleave_refused;
static uint64_t wf_interleave_returns_made;
static uint64_t wf_interleave_retries_made;

/* One repetition asked the question only if every open the host refused was
 * refused by this process's own narrowed table.  `ENFILE` is the machine's
 * file table and `ENOMEM` is the machine's memory: both refuse an open that
 * the three descriptors this repetition frees would otherwise have satisfied,
 * so a repetition that meets one is a repetition whose premise did not hold,
 * not a schedule that lost a return.  `EMFILE` is the refusal the probe
 * arranges and the one the runtime's rule is about. */
static int wf_interleave_outside_the_narrowed_table(int error_code) {
    return error_code != 0 && error_code != EMFILE;
}

/* Prints where the ledger stood and leaves.  Nothing here takes a lock the
 * stalled threads could be holding. */
static void *wf_interleave_watchdog(void *context) {
    unsigned seen = 0u;
    unsigned idle_seconds = 0u;
    (void)context;
    for (;;) {
        struct timespec pause;
        unsigned now;
        pause.tv_sec = 1;
        pause.tv_nsec = 0;
        (void)nanosleep(&pause, NULL);
        if (atomic_load(&wf_interleave_finished) != 0) {
            return NULL;
        }
        now = atomic_load(&wf_interleave_progress);
        if (now != seen) {
            seen = now;
            idle_seconds = 0u;
            continue;
        }
        idle_seconds += 1u;
        if (idle_seconds < (unsigned)WF_INTERLEAVE_WATCHDOG_SECONDS) {
            continue;
        }
        printf(
            "completion-retirement-interleave: FAIL stalled at repetition %u"
            " returns=%llu waiters=%llu waits=%llu\n",
            now,
            (unsigned long long)wf_completion_descriptor_returns(),
            (unsigned long long)wf_completion_retirement_waiters(),
            (unsigned long long)wf_completion_retirement_waits()
        );
        (void)fflush(stdout);
        _exit(9);
    }
}

/* One open on the third route, on a thread of the program's own, released
 * together with the submitted work so its refusal lands among the others. */
static void *wf_interleave_direct_open(void *context) {
    int error_code = -1;
    unsigned outcome = 0u;
    int64_t value;
    (void)context;
    while (atomic_load(&wf_interleave_gate) == 0) {
    }
    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        wf_interleave_path,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &outcome
    );
    atomic_store(&wf_interleave_direct_error, error_code);
    atomic_store(
        &wf_interleave_direct_value,
        (value >= 0 && error_code == 0 && outcome == WF_FILE_OPEN_SUCCEEDED)
            ? (int)value
            : -1
    );
    return NULL;
}

/* Narrows this process's descriptor limit until the table is full with `keep`
 * descriptors held, so the next open is one the host refuses for want of one.
 * Everything this runtime needs — the ring, its wake endpoint, any helper —
 * must already exist, which is what the warm-up below is for. */
static int wf_interleave_hold_the_table(
    struct rlimit *saved,
    int *held,
    int keep
) {
    struct rlimit narrowed;
    int probe;
    int index;
    if (getrlimit(RLIMIT_NOFILE, saved) != 0) {
        return -1;
    }
    probe = open(wf_interleave_path, O_RDONLY);
    if (probe < 0 || close(probe) != 0) {
        return -1;
    }
    narrowed = *saved;
    narrowed.rlim_cur = (rlim_t)probe + (rlim_t)keep;
    if (narrowed.rlim_cur > saved->rlim_max
        || setrlimit(RLIMIT_NOFILE, &narrowed) != 0) {
        return -1;
    }
    for (index = 0; index < keep; ++index) {
        held[index] = open(wf_interleave_path, O_RDONLY);
        if (held[index] < 0) {
            while (--index >= 0) {
                (void)close(held[index]);
            }
            (void)setrlimit(RLIMIT_NOFILE, saved);
            return -1;
        }
    }
    probe = open(wf_interleave_path, O_RDONLY);
    if (probe >= 0) {
        (void)close(probe);
        for (index = 0; index < keep; ++index) {
            (void)close(held[index]);
        }
        (void)setrlimit(RLIMIT_NOFILE, saved);
        return -1;
    }
    return 0;
}

/* One repetition.  Returns the number of opens that succeeded, or -1 where the
 * host would not hold the shape still long enough to ask the question, or -2
 * where it refused an open for a reason this repetition did not arrange. */
static int wf_interleave_once(void) {
    struct rlimit saved;
    int held[WF_INTERLEAVE_HELD];
    wf_completion_token refused[WF_INTERLEAVE_HELD];
    wf_completion_token performed[WF_INTERLEAVE_HELD];
    wf_completion_token opens[WF_INTERLEAVE_OPENS];
    int opened[WF_INTERLEAVE_OPENS];
    pthread_t opener;
    int64_t value;
    int error_code;
    unsigned outcome;
    int index;
    int succeeded = 0;
    uint64_t returns_before = wf_completion_descriptor_returns();
    uint64_t retries_before = wf__completion_open_exhaustion_retries();
    wf_interleave_outside_error = 0;
    wf_interleave_left_over = 0;
    wf_interleave_refused = 0;
    atomic_store(&wf_interleave_direct_error, 0);
    for (index = 0; index < WF_INTERLEAVE_OPENS; ++index) {
        wf_interleave_open_error[index] = 0;
    }
    if (wf_interleave_hold_the_table(&saved, held, WF_INTERLEAVE_HELD) != 0) {
        return -1;
    }
    atomic_store(&wf_interleave_gate, 0);
    atomic_store(&wf_interleave_direct_value, -1);
    if (pthread_create(&opener, NULL, wf_interleave_direct_open, NULL) != 0) {
        for (index = 0; index < WF_INTERLEAVE_HELD; ++index) {
            (void)close(held[index]);
        }
        (void)setrlimit(RLIMIT_NOFILE, &saved);
        return -1;
    }
    /* A close the host refuses beside every close it performs: the first
     * returns nothing and grants no re-attempt, the second returns one. */
    for (index = 0; index < WF_INTERLEAVE_HELD; ++index) {
        (void)wf__completion_file_close_submit(30000, &refused[index]);
        (void)wf__completion_file_close_submit(held[index], &performed[index]);
    }
    for (index = 0; index < WF_INTERLEAVE_OPENS; ++index) {
        (void)wf__completion_file_open_at_submit(
            AT_FDCWD,
            wf_interleave_path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &opens[index]
        );
    }
    atomic_store(&wf_interleave_gate, 1);
    for (index = 0; index < WF_INTERLEAVE_OPENS; ++index) {
        value = -1;
        error_code = -1;
        outcome = 0u;
        wf__completion_file_open_join(&opens[index], &value, &error_code,
                                      &outcome);
        opened[index] = (int)value;
        wf_interleave_open_error[index] = error_code;
        if (value >= 0 && error_code == 0
            && outcome == WF_FILE_OPEN_SUCCEEDED) {
            succeeded += 1;
        }
    }
    for (index = 0; index < WF_INTERLEAVE_HELD; ++index) {
        value = -1;
        error_code = -1;
        wf__completion_file_join(&refused[index], &value, &error_code);
        value = -1;
        error_code = -1;
        wf__completion_file_join(&performed[index], &value, &error_code);
    }
    (void)pthread_join(opener, NULL);
    value = atomic_load(&wf_interleave_direct_value);
    if (value >= 0) {
        succeeded += 1;
    }
    for (index = 0; index < WF_INTERLEAVE_OPENS; ++index) {
        if (wf_interleave_outside_the_narrowed_table(
                wf_interleave_open_error[index]
            ) != 0) {
            wf_interleave_outside_error = wf_interleave_open_error[index];
        }
        if (wf_interleave_open_error[index] == EMFILE) {
            wf_interleave_refused += 1;
        }
    }
    if (wf_interleave_outside_the_narrowed_table(
            atomic_load(&wf_interleave_direct_error)
        ) != 0) {
        wf_interleave_outside_error = atomic_load(&wf_interleave_direct_error);
    }
    if (atomic_load(&wf_interleave_direct_error) == EMFILE) {
        wf_interleave_refused += 1;
    }
    wf_interleave_returns_made =
        wf_completion_descriptor_returns() - returns_before;
    wf_interleave_retries_made =
        wf__completion_open_exhaustion_retries() - retries_before;
    if ((succeeded < WF_INTERLEAVE_HELD || wf_interleave_refused != 0)
        && wf_interleave_outside_error == 0) {
        /* Asked while the table is still narrow, so the answer is about the
         * three descriptors this repetition freed and not about the host's
         * table at large: is one of them still there for the taking after
         * every open of this repetition has published?  If it is, a return
         * this runtime made went to no open at all.
         *
         * Asked of every repetition that published a refusal, not only of one
         * whose count already fell short, because it is the second and
         * independent way this defect shows: a spare descriptor beside a
         * published refusal is a lost `Ok` however the opens are counted. */
        int spare = open(wf_interleave_path, O_RDONLY);
        if (spare >= 0) {
            wf_interleave_left_over = 1;
            (void)close(spare);
        }
    }
    if (value >= 0) {
        (void)close((int)value);
    }
    for (index = 0; index < WF_INTERLEAVE_OPENS; ++index) {
        if (opened[index] >= 0) {
            (void)close(opened[index]);
        }
    }
    (void)setrlimit(RLIMIT_NOFILE, &saved);
    return wf_interleave_outside_error != 0 ? -2 : succeeded;
}

/* One open and one close through the bridge, so every descriptor this runtime
 * needs exists before the limit narrows. */
static int wf_interleave_warm(void) {
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned outcome = 0u;
    if (wf__completion_file_open_at_submit(AT_FDCWD, wf_interleave_path,
            O_RDONLY, 0u, 0u, WF_FILE_EXPECT_REGULAR, &token) != 1) {
        return -1;
    }
    wf__completion_file_open_join(&token, &value, &error_code, &outcome);
    if (value < 0) {
        return -1;
    }
    if (wf__completion_file_close_submit((int)value, &token) != 1) {
        return -1;
    }
    wf__completion_file_join(&token, &value, &error_code);
    return (int)value;
}

/* Where this probe's file lives, and why it is not simply the scratch
 * directory: the schedule needs a filesystem whose `IORING_OP_CLOSE` the
 * kernel cannot run inline, and `overlayfs` is the one every supported Linux
 * host has.  Mounting it needs a privilege a gate does not always have, so a
 * failure here is a skip with its reason, never a quiet pass.
 *
 * Returns 0 having filled `directory`, or the skip status. */
static char wf_interleave_mount[256];

/* Takes the layer directories back out of the scratch directory, whether they
 * ever carried a mount or not: this runs on every host the gate runs on, and a
 * probe that leaves a directory behind per run is a probe that fills one. */
static void wf_interleave_remove_the_layers(const char *directory) {
    char layer[320];
    (void)rmdir(directory);
    (void)snprintf(layer, sizeof(layer), "%s/lower", wf_interleave_mount);
    (void)rmdir(layer);
    (void)snprintf(layer, sizeof(layer), "%s/upper", wf_interleave_mount);
    (void)rmdir(layer);
    /* An overlay mount keeps its own directory inside the work layer, so the
     * work layer is empty only once that one is gone. */
    (void)snprintf(layer, sizeof(layer), "%s/work/work", wf_interleave_mount);
    (void)rmdir(layer);
    (void)snprintf(layer, sizeof(layer), "%s/work", wf_interleave_mount);
    (void)rmdir(layer);
    (void)rmdir(wf_interleave_mount);
}

static int wf_interleave_mount_overlay(const char *scratch, char *directory,
                                       size_t capacity) {
    char lower[320];
    char upper[320];
    char work[320];
    char options[1024];
    (void)snprintf(wf_interleave_mount, sizeof(wf_interleave_mount),
                   "%s/wf-retirement-interleave-%ld", scratch, (long)getpid());
    (void)snprintf(lower, sizeof(lower), "%s/lower", wf_interleave_mount);
    (void)snprintf(upper, sizeof(upper), "%s/upper", wf_interleave_mount);
    (void)snprintf(work, sizeof(work), "%s/work", wf_interleave_mount);
    (void)snprintf(directory, capacity, "%s/merged", wf_interleave_mount);
    if (mkdir(wf_interleave_mount, 0700) != 0 && errno != EEXIST) {
        printf("completion-retirement-interleave: SKIP no scratch directory"
               " (errno=%d)\n", errno);
        return WF_INTERLEAVE_SKIP;
    }
    if ((mkdir(lower, 0700) != 0 && errno != EEXIST)
        || (mkdir(upper, 0700) != 0 && errno != EEXIST)
        || (mkdir(work, 0700) != 0 && errno != EEXIST)
        || (mkdir(directory, 0700) != 0 && errno != EEXIST)) {
        printf("completion-retirement-interleave: SKIP no overlay layers"
               " (errno=%d)\n", errno);
        return WF_INTERLEAVE_SKIP;
    }
    (void)snprintf(options, sizeof(options),
                   "lowerdir=%s,upperdir=%s,workdir=%s", lower, upper, work);
    if (mount("overlay", directory, "overlay", 0uL, options) != 0) {
        int reason = errno;
        wf_interleave_remove_the_layers(directory);
        printf("completion-retirement-interleave: SKIP overlayfs is not"
               " mountable here (errno=%d); the ring runs a close on this"
               " host's own filesystem inline, so the schedule this probe is"
               " about does not exist to be tested\n", reason);
        return WF_INTERLEAVE_SKIP;
    }
    return 0;
}

static void wf_interleave_unmount_overlay(const char *directory) {
    (void)umount(directory);
    wf_interleave_remove_the_layers(directory);
}
#endif

int main(int argc, char **argv) {
#if !defined(__linux__)
    (void)argc;
    (void)argv;
    printf("completion-retirement-interleave: SKIP this target has no kernel"
           " completion ring, so a close cannot be asynchronous here\n");
    return WF_INTERLEAVE_SKIP;
#else
    pthread_t watchdog;
    char directory[320];
    int repetition;
    int attempted = 0;
    int lost = 0;
    int miscounted = 0;
    int skipped = 0;
    int created;
    if (argc < 2) {
        printf("usage: retirement-interleave-probe SCRATCH_DIRECTORY\n");
        return 2;
    }
    {
        int mounted = wf_interleave_mount_overlay(argv[1], directory,
                                                  sizeof(directory));
        if (mounted != 0) {
            return mounted;
        }
    }
    (void)snprintf(wf_interleave_path, sizeof(wf_interleave_path),
                   "%s/file", directory);
    created = open(wf_interleave_path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (created < 0 || close(created) != 0) {
        printf("completion-retirement-interleave: FAIL no file to open"
               " (errno=%d)\n", errno);
        wf_interleave_unmount_overlay(directory);
        return 1;
    }
    if (pthread_create(&watchdog, NULL, wf_interleave_watchdog, NULL) != 0) {
        printf("completion-retirement-interleave: FAIL no watchdog\n");
        wf_interleave_unmount_overlay(directory);
        return 1;
    }
    if (wf_interleave_warm() != 0) {
        printf("completion-retirement-interleave: FAIL the runtime would not"
               " open and close this file\n");
        atomic_store(&wf_interleave_finished, 1);
        (void)pthread_join(watchdog, NULL);
        wf_interleave_unmount_overlay(directory);
        return 1;
    }
    for (repetition = 0; repetition < WF_INTERLEAVE_REPETITIONS;
         ++repetition) {
        int succeeded = wf_interleave_once();
        atomic_store(&wf_interleave_progress, (unsigned)repetition + 1u);
        if (succeeded == -2) {
            /* The premise did not hold: an open was refused by something
             * other than the three descriptors this repetition governs, so
             * this repetition never asked the question.  Said out loud, and
             * not counted either way — a host that refuses every repetition
             * this way reaches the skip below rather than a quiet pass. */
            skipped += 1;
            printf("completion-retirement-interleave: repetition %d not asked:"
                   " an open was refused with errno=%d, which is not this"
                   " process's narrowed descriptor table\n",
                   repetition, wf_interleave_outside_error);
            continue;
        }
        if (succeeded < 0) {
            continue;
        }
        attempted += 1;
        if (succeeded < WF_INTERLEAVE_HELD) {
            lost += 1;
            printf("completion-retirement-interleave: repetition %d published"
                   " %d of %d owed opens; errno=%d,%d,%d,%d direct=%d"
                   " returned=%llu re-attempts=%llu descriptor-left-over=%d\n",
                   repetition, succeeded, WF_INTERLEAVE_HELD,
                   wf_interleave_open_error[0], wf_interleave_open_error[1],
                   wf_interleave_open_error[2], wf_interleave_open_error[3],
                   (int)atomic_load(&wf_interleave_direct_error),
                   (unsigned long long)wf_interleave_returns_made,
                   (unsigned long long)wf_interleave_retries_made,
                   wf_interleave_left_over);
        } else if (wf_interleave_left_over != 0
                   && wf_interleave_refused != 0) {
            /* The count says every owed `Ok` was published and the table says
             * a descriptor this repetition freed went to no open, with a
             * refusal published against it.  The same loss, seen the other
             * way. */
            lost += 1;
            printf("completion-retirement-interleave: repetition %d published"
                   " %d of %d owed opens and left a descriptor over with %d"
                   " refusals published; returned=%llu re-attempts=%llu\n",
                   repetition, succeeded, WF_INTERLEAVE_HELD,
                   wf_interleave_refused,
                   (unsigned long long)wf_interleave_returns_made,
                   (unsigned long long)wf_interleave_retries_made);
        }
        /* What the ledger promised, checked from outside it.  One re-attempt
         * per open is the whole of the rule, so the repetition's five opens
         * can be granted at most five; and every refusal that reached
         * publication was granted one, so a repetition that published more
         * refusals than the ledger granted re-attempts has published one
         * without the second attempt it is owed.  Both held at every one of
         * the 25 losses this probe's diagnosis was built from, so a run where
         * one of them breaks is a different defect and says so. */
        if (wf_interleave_retries_made
                > (uint64_t)(WF_INTERLEAVE_OPENS + 1)
            || (uint64_t)wf_interleave_refused
                > wf_interleave_retries_made) {
            miscounted += 1;
            printf("completion-retirement-interleave: repetition %d granted"
                   " %llu re-attempts for %d opens, %d of them refused\n",
                   repetition,
                   (unsigned long long)wf_interleave_retries_made,
                   WF_INTERLEAVE_OPENS + 1, wf_interleave_refused);
        }
    }
    atomic_store(&wf_interleave_finished, 1);
    (void)pthread_join(watchdog, NULL);
    (void)unlink(wf_interleave_path);
    wf_interleave_unmount_overlay(directory);
    if (attempted == 0) {
        if (skipped != 0) {
            printf("completion-retirement-interleave: SKIP every one of %d"
                   " repetitions met a refusal this process's narrowed table"
                   " did not cause, so none of them asked the question\n",
                   skipped);
        } else {
            printf("completion-retirement-interleave: SKIP this process cannot"
                   " narrow its own descriptor table, so no open here is one"
                   " the host refuses for want of a descriptor\n");
        }
        return WF_INTERLEAVE_SKIP;
    }
    /* `not-asked` is on the passing line as well as the failing one: a
     * repetition that met a refusal this process's narrowed table did not
     * cause is excluded from both counts, and a reader has to be able to see
     * how much of the run that was — including the case the exclusion is
     * strictest about, a genuine `EMFILE` shortfall in a repetition that also
     * met one of those refusals, which is dropped as not asked rather than
     * counted as a loss. */
    printf("completion-retirement-interleave: %s repetitions=%d lost=%d"
           " miscounted=%d not-asked=%d helpers=%llu\n",
           lost == 0 && miscounted == 0 ? "PASS" : "FAIL",
           attempted, lost, miscounted, skipped,
           (unsigned long long)wf__completion_target_helper_count());
    return lost == 0 && miscounted == 0 ? 0 : 1;
#endif
}
