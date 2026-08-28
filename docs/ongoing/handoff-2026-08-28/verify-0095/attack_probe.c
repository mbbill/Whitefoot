/* Adversarial attacks on the Stage A repair (ad04ae2d).  Not repository material. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
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
static const char *stage = "none";
#define CHECK(cond) do { if (!(cond)) { printf("FAIL[%s] %s:%d %s\n", stage, __FILE__, __LINE__, #cond); failures += 1; } } while (0)

static char dir[512];
static char path[512];

static void on_alarm(int signal_number) {
    (void)signal_number;
    /* A hang is a failure with a name. */
    const char *message = "FAIL HANG in stage: ";
    ssize_t ignored = write(2, message, 20);
    ignored = write(2, stage, strlen(stage));
    ignored = write(2, "\n", 1);
    (void)ignored;
    _exit(9);
}

static int hold_the_last_descriptor(const char *file, struct rlimit *saved) {
    struct rlimit narrowed;
    int probe;
    int held;
    if (getrlimit(RLIMIT_NOFILE, saved) != 0) return -1;
    probe = open(file, O_RDONLY);
    if (probe < 0) return -1;
    if (close(probe) != 0) return -1;
    narrowed = *saved;
    narrowed.rlim_cur = (rlim_t)probe + 1;
    if (narrowed.rlim_cur > saved->rlim_max) return -1;
    if (setrlimit(RLIMIT_NOFILE, &narrowed) != 0) return -1;
    held = open(file, O_RDONLY);
    if (held != probe) {
        if (held >= 0) (void)close(held);
        (void)setrlimit(RLIMIT_NOFILE, saved);
        return -1;
    }
    return held;
}

/* Give the runtime every descriptor it lazily creates before the limit
 * narrows: ring, wait set, wake, and any helper. */
static void warm(void) {
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned outcome = 0;
    CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u, 0u,
              WF_FILE_EXPECT_REGULAR, &token) == 1);
    wf__completion_file_open_join(&token, &value, &error_code, &outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
}

/* A1: one close in flight, two opens behind it.  Source order closes, opens
 * once with the freed descriptor and is refused the second time, so exactly
 * one Ok and one Err. */
static void attack_two_opens_behind_one_close(void) {
    struct rlimit saved;
    wf_completion_token close_token;
    wf_completion_token open_tokens[2];
    int64_t value;
    int error_code;
    unsigned outcome;
    int oks = 0;
    int errs = 0;
    int opened = -1;
    int held;
    int index;
    stage = "A1 two opens behind one close";
    warm();
    held = hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    if (held < 0) return;
    CHECK(open(path, O_RDONLY) < 0);
    CHECK(wf__completion_file_close_submit(held, &close_token) == 1);
    for (index = 0; index < 2; ++index) {
        CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u,
                  0u, WF_FILE_EXPECT_REGULAR, &open_tokens[index]) == 1);
    }
    for (index = 0; index < 2; ++index) {
        value = -1; error_code = -1; outcome = 99;
        wf__completion_file_open_join(&open_tokens[index], &value, &error_code,
                                      &outcome);
        if (value >= 0 && error_code == 0
            && outcome == WF_FILE_OPEN_SUCCEEDED) {
            oks += 1;
            opened = (int)value;
        } else {
            errs += 1;
            CHECK(error_code == EMFILE || error_code == ENFILE);
            CHECK(outcome == WF_FILE_OPEN_FAILED);
        }
    }
    value = -1; error_code = -1;
    wf__completion_file_join(&close_token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    printf("A1: oks=%d errs=%d\n", oks, errs);
    CHECK(oks == 1);
    CHECK(errs == 1);
    if (opened >= 0) {
        CHECK(wf__completion_file_close_submit(opened, &close_token) == 1);
        wf__completion_file_join(&close_token, &value, &error_code);
    }
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
}

/* A2: the close itself fails.  Its publication still opens the held open's
 * gate; the re-attempt finds no descriptor and the refusal is published once. */
static void attack_close_that_fails(void) {
    struct rlimit saved;
    wf_completion_token close_token;
    wf_completion_token open_token;
    int64_t value;
    int error_code;
    unsigned outcome;
    uint64_t retries_before;
    int stale;
    int held;
    stage = "A2 close that fails";
    warm();
    held = hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    if (held < 0) return;
    /* A descriptor number the process does not own: closing it fails. */
    stale = held + 1;
    retries_before = wf__completion_open_exhaustion_retries();
    CHECK(wf__completion_file_close_submit(stale, &close_token) == 1);
    CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u, 0u,
              WF_FILE_EXPECT_REGULAR, &open_token) == 1);
    value = -1; error_code = -1; outcome = 99;
    wf__completion_file_open_join(&open_token, &value, &error_code, &outcome);
    printf("A2: open value=%lld error=%d outcome=%u retries=%llu->%llu\n",
           (long long)value, error_code, outcome,
           (unsigned long long)retries_before,
           (unsigned long long)wf__completion_open_exhaustion_retries());
    CHECK(value < 0);
    CHECK(error_code == EMFILE || error_code == ENFILE);
    CHECK(outcome == WF_FILE_OPEN_FAILED);
    value = -1; error_code = 0;
    wf__completion_file_join(&close_token, &value, &error_code);
    printf("A2: close value=%lld error=%d\n", (long long)value, error_code);
    CHECK(error_code == EBADF);
    CHECK(close(held) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
}

/* A3: several opens refused together with nothing else in flight.  Each must
 * publish exactly one refusal and none may wait on another. */
static void attack_many_refused_together(int count) {
    struct rlimit saved;
    wf_completion_token tokens[16];
    int64_t value;
    int error_code;
    unsigned outcome;
    uint64_t retries_before;
    int held;
    int index;
    int errs = 0;
    stage = "A3 several refused together";
    warm();
    held = hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    if (held < 0) return;
    retries_before = wf__completion_open_exhaustion_retries();
    for (index = 0; index < count; ++index) {
        CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u,
                  0u, WF_FILE_EXPECT_REGULAR, &tokens[index]) == 1);
    }
    for (index = 0; index < count; ++index) {
        value = -1; error_code = -1; outcome = 99;
        wf__completion_file_open_join(&tokens[index], &value, &error_code,
                                      &outcome);
        if (value < 0) {
            errs += 1;
            CHECK(error_code == EMFILE || error_code == ENFILE);
        }
    }
    printf("A3: count=%d errs=%d retries=%llu->%llu\n", count, errs,
           (unsigned long long)retries_before,
           (unsigned long long)wf__completion_open_exhaustion_retries());
    CHECK(errs == count);
    CHECK(close(held) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
}

/* A4: the refused open's only company is an operation that cannot finish
 * until another thread acts.  The open must wait for it, not publish, and the
 * descriptor that thread gives back must reach the re-attempt. */
struct releaser {
    int held;
    int writer;
    int wrote;
};

static void *releaser_main(void *context) {
    struct releaser *release = context;
    struct timespec pause;
    pause.tv_sec = 0;
    pause.tv_nsec = 200 * 1000 * 1000;
    (void)nanosleep(&pause, NULL);
    (void)close(release->held);
    release->wrote = (int)write(release->writer, "x", 1);
    return NULL;
}

static void attack_open_waits_for_a_parked_read(void) {
    struct rlimit saved;
    struct releaser release;
    pthread_t thread;
    wf_completion_token read_token;
    wf_completion_token open_token;
    unsigned char received = 0;
    int64_t value;
    int error_code;
    unsigned outcome;
    uint64_t retries_before;
    int pipe_pair[2];
    int held;
    stage = "A4 open waits for a parked read";
    warm();
    CHECK(pipe(pipe_pair) == 0);
    held = hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    if (held < 0) return;
    CHECK(open(path, O_RDONLY) < 0);
    retries_before = wf__completion_open_exhaustion_retries();
    {
        uint64_t ring_before = wf__completion_linux_io_uring_submissions();
        uint64_t fallback_before = wf__completion_file_fallback_submissions();
        CHECK(wf__completion_file_read_submit(pipe_pair[0], &received, 1u,
                                              &read_token) == 1);
        printf("A4 route: read ring+%llu fallback+%llu\n",
               (unsigned long long)(wf__completion_linux_io_uring_submissions()
                                    - ring_before),
               (unsigned long long)(wf__completion_file_fallback_submissions()
                                    - fallback_before));
        ring_before = wf__completion_linux_io_uring_submissions();
        fallback_before = wf__completion_file_fallback_submissions();
        CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u,
                  0u, WF_FILE_EXPECT_REGULAR, &open_token) == 1);
        printf("A4 route: open ring+%llu fallback+%llu\n",
               (unsigned long long)(wf__completion_linux_io_uring_submissions()
                                    - ring_before),
               (unsigned long long)(wf__completion_file_fallback_submissions()
                                    - fallback_before));
    }
    release.held = held;
    release.writer = pipe_pair[1];
    release.wrote = -1;
    CHECK(pthread_create(&thread, NULL, releaser_main, &release) == 0);
    value = -1; error_code = -1; outcome = 99;
    wf__completion_file_open_join(&open_token, &value, &error_code, &outcome);
    CHECK(pthread_join(thread, NULL) == 0);
    printf("A4: open value=%lld error=%d outcome=%u wrote=%d retries=%llu->%llu\n",
           (long long)value, error_code, outcome, release.wrote,
           (unsigned long long)retries_before,
           (unsigned long long)wf__completion_open_exhaustion_retries());
    CHECK(release.wrote == 1);
    /* Source order runs the read, then the close inside it, then the open:
     * the open succeeds.  A pipeline that publishes the refusal instead has
     * invented an exhaustion. */
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(outcome == WF_FILE_OPEN_SUCCEEDED);
    if (value >= 0) {
        wf_completion_token close_token;
        int64_t v2 = -1; int e2 = -1;
        CHECK(wf__completion_file_close_submit((int)value, &close_token) == 1);
        wf__completion_file_join(&close_token, &v2, &e2);
    }
    value = -1; error_code = -1;
    wf__completion_file_join(&read_token, &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(received == 'x');
    CHECK(close(pipe_pair[0]) == 0);
    CHECK(close(pipe_pair[1]) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
}

/* A5: capacity pressure while an open is held.  More opens than the runtime's
 * record capacity, all refused, so held entries and capacity waits meet. */
static void attack_capacity_while_held(void) {
    struct rlimit saved;
    wf_completion_token tokens[96];
    int64_t value;
    int error_code;
    unsigned outcome;
    int held;
    int index;
    int submitted = 0;
    int errs = 0;
    stage = "A5 capacity while held";
    warm();
    held = hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    if (held < 0) return;
    for (index = 0; index < 96; ++index) {
        if (wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u,
                0u, WF_FILE_EXPECT_REGULAR, &tokens[index]) != 1) {
            break;
        }
        submitted += 1;
    }
    for (index = 0; index < submitted; ++index) {
        value = -1; error_code = -1; outcome = 99;
        wf__completion_file_open_join(&tokens[index], &value, &error_code,
                                      &outcome);
        if (value < 0) errs += 1;
        else {
            wf_completion_token close_token;
            int64_t v2 = -1; int e2 = -1;
            wf__completion_file_close_submit((int)value, &close_token);
            wf__completion_file_join(&close_token, &v2, &e2);
        }
    }
    printf("A5: submitted=%d errs=%d\n", submitted, errs);
    CHECK(errs == submitted);
    CHECK(submitted > 0);
    CHECK(close(held) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
}

/* A6: the repaired case, repeated, to find a schedule that still loses it. */
static void attack_close_then_open_repeated(int repetitions) {
    struct rlimit saved;
    wf_completion_token close_token;
    wf_completion_token open_token;
    int64_t value;
    int error_code;
    unsigned outcome;
    int held;
    int index;
    int losses = 0;
    stage = "A6 close-then-open repeated";
    warm();
    for (index = 0; index < repetitions; ++index) {
        held = hold_the_last_descriptor(path, &saved);
        if (held < 0) { CHECK(0); return; }
        CHECK(wf__completion_file_close_submit(held, &close_token) == 1);
        CHECK(wf__completion_file_open_at_submit(AT_FDCWD, path, O_RDONLY, 0u,
                  0u, WF_FILE_EXPECT_REGULAR, &open_token) == 1);
        value = -1; error_code = -1; outcome = 99;
        wf__completion_file_open_join(&open_token, &value, &error_code,
                                      &outcome);
        if (value < 0) losses += 1;
        else {
            wf_completion_token t;
            int64_t v2 = -1; int e2 = -1;
            wf__completion_file_close_submit((int)value, &t);
            wf__completion_file_join(&t, &v2, &e2);
        }
        value = -1; error_code = -1;
        wf__completion_file_join(&close_token, &value, &error_code);
        CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    }
    printf("A6: repetitions=%d lost_ok=%d\n", repetitions, losses);
    CHECK(losses == 0);
}

int main(int argc, char **argv) {
    int fd;
    struct sigaction action;
    if (argc < 2) { printf("usage: attack_probe DIR\n"); return 2; }
    snprintf(dir, sizeof(dir), "%s", argv[1]);
    snprintf(path, sizeof(path), "%s/attack-file", dir);
    memset(&action, 0, sizeof(action));
    action.sa_handler = on_alarm;
    CHECK(sigaction(SIGALRM, &action, NULL) == 0);
    alarm(120);
    (void)unlink(path);
    fd = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(fd >= 0);
    CHECK(close(fd) == 0);

    attack_two_opens_behind_one_close();
    attack_close_that_fails();
    attack_many_refused_together(4);
    attack_open_waits_for_a_parked_read();
    attack_capacity_while_held();
    attack_close_then_open_repeated(300);

    (void)unlink(path);
    printf("attack_probe: %s (failures=%d)\n", failures == 0 ? "PASS" : "FAIL",
           failures);
    return failures == 0 ? 0 : 1;
}
