/* The consumer side of the pipe-relay workload.
 *
 * Runs one producer with both of its outputs on pipes and drains each pipe on
 * its own thread, sleeping DELAY_US after every read so the consumer, not the
 * producer's CPU, sets the pace. That is what makes each write genuinely
 * wait: a chunk is the pipe buffer's size, so a producer cannot get ahead.
 *
 * A producer that can only wait on one output at a time pays both consumers'
 * service times in series. One that keeps two operations in flight pays the
 * larger of the two. The counted bytes are checked on both streams, so a run
 * that published less than it should cannot report a time.
 *
 * Reports: wall milliseconds, then the byte total each stream received. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define PIPE_RELAY_CHUNK 262144u

struct drain {
    pthread_t thread;
    int descriptor;
    unsigned long delay_us;
    unsigned char expected;
    uint64_t received;
    int failure;
};

static void *drain_main(void *raw) {
    struct drain *drain = raw;
    unsigned char *window = malloc(PIPE_RELAY_CHUNK);
    if (window == NULL) {
        drain->failure = 1;
        return NULL;
    }
    for (;;) {
        ssize_t moved = read(drain->descriptor, window, PIPE_RELAY_CHUNK);
        if (moved < 0) {
            drain->failure = 1;
            break;
        }
        if (moved == 0) {
            break;
        }
        for (ssize_t at = 0; at < moved; at++) {
            if (window[at] != drain->expected) {
                drain->failure = 1;
            }
        }
        drain->received += (uint64_t)moved;
        if (drain->delay_us > 0) {
            struct timespec pause;
            pause.tv_sec = (time_t)(drain->delay_us / 1000000ul);
            pause.tv_nsec = (long)((drain->delay_us % 1000000ul) * 1000ul);
            nanosleep(&pause, NULL);
        }
    }
    free(window);
    close(drain->descriptor);
    return NULL;
}

int main(int argc, char **argv) {
    if (argc < 4) {
        fprintf(stderr, "usage: pipe_harness DELAY_US EXPECTED_BYTES COMMAND [ARG...]\n");
        return 2;
    }
    unsigned long delay_us = strtoul(argv[1], NULL, 10);
    uint64_t expected = strtoull(argv[2], NULL, 10);
    int left[2];
    int right[2];
    if (pipe(left) != 0 || pipe(right) != 0) {
        perror("pipe_harness: pipe");
        return 1;
    }
    struct timespec start;
    clock_gettime(CLOCK_MONOTONIC, &start);
    pid_t child = fork();
    if (child < 0) {
        perror("pipe_harness: fork");
        return 1;
    }
    if (child == 0) {
        close(left[0]);
        close(right[0]);
        if (dup2(left[1], 1) < 0 || dup2(right[1], 2) < 0) {
            _exit(120);
        }
        close(left[1]);
        close(right[1]);
        execv(argv[3], &argv[3]);
        _exit(121);
    }
    close(left[1]);
    close(right[1]);
    struct drain drains[2];
    memset(drains, 0, sizeof drains);
    drains[0].descriptor = left[0];
    drains[0].delay_us = delay_us;
    drains[0].expected = 'A';
    drains[1].descriptor = right[0];
    drains[1].delay_us = delay_us;
    drains[1].expected = 'B';
    for (int at = 0; at < 2; at++) {
        if (pthread_create(&drains[at].thread, NULL, drain_main, &drains[at]) != 0) {
            fprintf(stderr, "pipe_harness: cannot start drain\n");
            return 1;
        }
    }
    for (int at = 0; at < 2; at++) {
        pthread_join(drains[at].thread, NULL);
    }
    int status = 0;
    if (waitpid(child, &status, 0) < 0) {
        perror("pipe_harness: waitpid");
        return 1;
    }
    struct timespec end;
    clock_gettime(CLOCK_MONOTONIC, &end);
    double wall_ms = ((double)(end.tv_sec - start.tv_sec) * 1000.0) +
                     ((double)(end.tv_nsec - start.tv_nsec) / 1000000.0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        fprintf(stderr, "pipe_harness: producer exited with %d\n", status);
        return 1;
    }
    if (drains[0].failure || drains[1].failure) {
        fprintf(stderr, "pipe_harness: a stream carried the wrong bytes\n");
        return 1;
    }
    if (expected != 0 && (drains[0].received != expected || drains[1].received != expected)) {
        fprintf(stderr, "pipe_harness: streams carried %llu and %llu, expected %llu each\n",
                (unsigned long long)drains[0].received, (unsigned long long)drains[1].received,
                (unsigned long long)expected);
        return 1;
    }
    printf("%.2f %llu %llu\n", wall_ms, (unsigned long long)drains[0].received,
           (unsigned long long)drains[1].received);
    return 0;
}
