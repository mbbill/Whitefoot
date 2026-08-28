/* Adversarial probes for Stage A (0095). Not repository material. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"

static int failures;
#define CHECK(cond) do { if (!(cond)) { printf("FAIL %s:%d %s\n", __FILE__, __LINE__, #cond); failures += 1; } } while (0)

static char dir[512];

static void nap(long ms) {
    struct timespec t;
    t.tv_sec = ms / 1000;
    t.tv_nsec = (ms % 1000) * 1000000L;
    nanosleep(&t, NULL);
}

/* ---- probe 1: a submitted open, then a BLOCKING direct open, no join ---- */

struct watcher {
    char fifo[512];
    uint64_t enters_before;
    uint64_t enters_seen_while_blocked;
    int observed;
};

static void *watcher_main(void *context) {
    struct watcher *w = context;
    int writer;
    nap(400);
    w->enters_seen_while_blocked =
        wf__completion_linux_io_uring_submission_enters();
    w->observed = 1;
    /* Unblock the main thread's blocking O_RDONLY open of the FIFO. */
    writer = open(w->fifo, O_WRONLY);
    if (writer >= 0) {
        (void)close(writer);
    }
    return NULL;
}

static int probe_deferred_doorbell(void) {
    char regular[512];
    struct watcher w;
    pthread_t thread;
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned outcome = 0;
    uint64_t submissions_before;
    int ring_route;
    int fd;
    int direct;

    snprintf(regular, sizeof(regular), "%s/verify-regular", dir);
    snprintf(w.fifo, sizeof(w.fifo), "%s/verify-fifo", dir);
    (void)unlink(regular);
    (void)unlink(w.fifo);
    fd = open(regular, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(fd >= 0);
    CHECK(write(fd, "abcdefgh", 8) == 8);
    CHECK(close(fd) == 0);
    CHECK(mkfifo(w.fifo, 0600) == 0);

    /* Warm the bridge so the ring exists and the route is known. */
    submissions_before = wf__completion_linux_io_uring_submissions();
    CHECK(wf__completion_file_open_at_submit(AT_FDCWD, regular, O_RDONLY, 0u, 0u,
              WF_FILE_EXPECT_REGULAR, &token) == 1);
    wf__completion_file_open_join(&token, &value, &error_code, &outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    ring_route =
        wf__completion_linux_io_uring_submissions() > submissions_before;
    printf("probe1: route=%s\n", ring_route ? "io_uring" : "posix-helpers");

    /* THE SCHEDULE: submit an open, do not join, then make a BLOCKING direct
     * open.  A second thread reads the enter counter while this thread is
     * still stuck inside that blocking call. */
    w.enters_before = wf__completion_linux_io_uring_submission_enters();
    w.enters_seen_while_blocked = 0;
    w.observed = 0;
    CHECK(wf__completion_file_open_at_submit(AT_FDCWD, regular, O_RDONLY, 0u, 0u,
              WF_FILE_EXPECT_REGULAR, &token) == 1);
    if (ring_route) {
        /* Nothing rang yet. */
        CHECK(wf__completion_linux_io_uring_submission_enters()
              == w.enters_before);
    }
    CHECK(pthread_create(&thread, NULL, watcher_main, &w) == 0);
    /* Blocks until the watcher opens the FIFO for writing. */
    {
        int direct_error = 0;
        unsigned direct_outcome = 0;
        direct = wf__completion_file_open_at_direct(AT_FDCWD, w.fifo, O_RDONLY,
                                                   0u, 0u, WF_FILE_EXPECT_ANY,
                                                   &direct_error,
                                                   &direct_outcome);
    }
    CHECK(pthread_join(thread, NULL) == 0);
    CHECK(w.observed == 1);
    printf("probe1: enters before=%llu, seen while blocked=%llu, direct=%d\n",
           (unsigned long long)w.enters_before,
           (unsigned long long)w.enters_seen_while_blocked, direct);
    if (ring_route) {
        CHECK(w.enters_seen_while_blocked > w.enters_before);
    }
    if (direct >= 0) {
        CHECK(wf__completion_file_close_direct(direct) == 0);
    }
    wf__completion_file_open_join(&token, &value, &error_code, &outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    CHECK(unlink(w.fifo) == 0);
    CHECK(unlink(regular) == 0);
    return 0;
}

/* ---- probe 2: retire-and-retry under a narrowed RLIMIT_NOFILE ---- */

static int hold_the_last_descriptor(const char *path, struct rlimit *saved) {
    struct rlimit narrowed;
    int probe;
    int held;
    if (getrlimit(RLIMIT_NOFILE, saved) != 0) return -1;
    probe = open(path, O_RDONLY);
    if (probe < 0) return -1;
    if (close(probe) != 0) return -1;
    narrowed = *saved;
    narrowed.rlim_cur = (rlim_t)probe + 1;
    if (narrowed.rlim_cur > saved->rlim_max) return -1;
    if (setrlimit(RLIMIT_NOFILE, &narrowed) != 0) return -1;
    held = open(path, O_RDONLY);
    if (held != probe) {
        if (held >= 0) (void)close(held);
        (void)setrlimit(RLIMIT_NOFILE, saved);
        return -1;
    }
    return held;
}

static int probe_retire_and_retry(void) {
    char path[512];
    struct rlimit saved;
    wf_completion_token open_token;
    wf_completion_token close_token;
    int64_t value = -1;
    int error_code = -1;
    unsigned outcome = 0;
    uint64_t retries_before;
    int held;
    int fd;

    snprintf(path, sizeof(path), "%s/verify-exhaustion", dir);
    (void)unlink(path);
    fd = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(fd >= 0);
    CHECK(close(fd) == 0);

    /* Warm every lazy descriptor the runtime needs before the limit narrows. */
    CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u, 0u,
              WF_FILE_EXPECT_REGULAR, &open_token) == 1);
    wf__completion_file_open_join(&open_token, &value, &error_code, &outcome);
    CHECK(value >= 0);
    CHECK(wf__completion_file_close_submit((int)value, &open_token) == 1);
    wf__completion_file_join(&open_token, &value, &error_code);
    CHECK(value == 0);

    /* CASE A: source order would succeed.  The one free descriptor is held by
     * a close this runtime already owns, and an open is submitted behind it.
     * A sequential program closes then opens and gets Ok, so the pipeline
     * must too. */
    retries_before = wf__completion_open_exhaustion_retries();
    held = hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    if (held >= 0) {
        CHECK(open(path, O_RDONLY) < 0);
        /* SOURCE ORDER: the close comes first, so a sequential execution
         * frees the descriptor and the open that follows succeeds.  This is
         * exactly LOOP-PIPELINE.md 2.10's case: the pipeline has both in
         * flight, so the open can be attempted while the close is still
         * owed, and only retire-and-retry can keep the program's Ok. */
        CHECK(wf__completion_file_close_submit(held, &close_token) == 1);
        CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u,
                  0u, WF_FILE_EXPECT_REGULAR, &open_token) == 1);
        wf__completion_file_open_join(&open_token, &value, &error_code,
                                      &outcome);
        printf("probe2 caseA: value=%lld error=%d outcome=%u retries=%llu->%llu\n",
               (long long)value, error_code, outcome,
               (unsigned long long)retries_before,
               (unsigned long long)wf__completion_open_exhaustion_retries());
        if (value < 0) {
            /* Diagnosis: retire the close, then ask again by hand.  If this
             * second, later open succeeds, the descriptor was free all along
             * and the refusal the program saw was a timing hole in the
             * re-attempt, not a genuine exhaustion. */
            int64_t v2 = -1; int e2 = -1; unsigned o2 = 0;
            wf_completion_token t2;
            wf__completion_file_join(&close_token, &v2, &e2);
            printf("probe2 caseA diag: close join value=%lld error=%d\n",
                   (long long)v2, e2);
            CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY,
                      0u, 0u, WF_FILE_EXPECT_REGULAR, &t2) == 1);
            wf__completion_file_open_join(&t2, &v2, &e2, &o2);
            printf("probe2 caseA diag: later open value=%lld error=%d outcome=%u\n",
                   (long long)v2, e2, o2);
            if (v2 >= 0) {
                wf__completion_file_close_submit((int)v2, &t2);
                wf__completion_file_join(&t2, &v2, &e2);
            }
            (void)setrlimit(RLIMIT_NOFILE, &saved);
            (void)unlink(path);
            CHECK(0 && "caseA: source order gives Ok, the pipeline gave Err");
            return 0;
        }
        CHECK(value >= 0);
        CHECK(error_code == 0);
        CHECK(outcome == WF_FILE_OPEN_SUCCEEDED);
        if (value >= 0) {
            int reopened = (int)value;
            wf__completion_file_join(&close_token, &value, &error_code);
            CHECK(error_code == 0);
            /* CASE B: genuine exhaustion.  Nothing else is in flight and the
             * one descriptor is the one just opened, so no retirement can
             * help.  The program must see the typed outcome, once. */
            CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY,
                      0u, 0u, WF_FILE_EXPECT_REGULAR, &open_token) == 1);
            wf__completion_file_open_join(&open_token, &value, &error_code,
                                          &outcome);
            printf("probe2 caseB: value=%lld error=%d outcome=%u\n",
                   (long long)value, error_code, outcome);
            CHECK(value < 0);
            CHECK(outcome == WF_FILE_OPEN_FAILED);
            CHECK(error_code == EMFILE || error_code == ENFILE);
            CHECK(wf__completion_file_close_submit(reopened, &close_token) == 1);
            wf__completion_file_join(&close_token, &value, &error_code);
            CHECK(error_code == 0);
        } else {
            wf__completion_file_join(&close_token, &value, &error_code);
        }
        CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    }
    (void)unlink(path);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) { printf("usage: verify_probe DIR\n"); return 2; }
    snprintf(dir, sizeof(dir), "%s", argv[1]);
    probe_deferred_doorbell();
    probe_retire_and_retry();
    printf("verify_probe: %s (failures=%d)\n", failures == 0 ? "PASS" : "FAIL",
           failures);
    return failures == 0 ? 0 : 1;
}
