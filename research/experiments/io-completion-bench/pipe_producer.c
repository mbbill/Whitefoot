/* The N line of the pipe-relay workload: the best hand-written native shapes
 * for pushing two independent byte streams at two independent consumers.
 *
 *   seq       one thread, write(1) then write(2) each round. This is what a
 *             blocking-call language gives a writer who states the two
 *             operations in order, and it is the shape Whitefoot's sequential
 *             reference build reaches.
 *   threads   one thread per output. Nothing on either stream ever waits for
 *             the other, so this is the ceiling the completion path is trying
 *             to reach without asking the writer for a thread.
 *
 * Both modes publish the same bytes: ROUNDS chunks of 'A' on fd 1 and ROUNDS
 * chunks of 'B' on fd 2. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define PIPE_RELAY_CHUNK 262144u

struct stream {
    pthread_t thread;
    int descriptor;
    unsigned char *bytes;
    unsigned long rounds;
    int failure;
};

static int push(int descriptor, const unsigned char *bytes, unsigned long rounds) {
    for (unsigned long round = 0; round < rounds; round++) {
        size_t done = 0;
        while (done < PIPE_RELAY_CHUNK) {
            ssize_t moved = write(descriptor, bytes + done, PIPE_RELAY_CHUNK - done);
            if (moved <= 0) {
                return 1;
            }
            done += (size_t)moved;
        }
    }
    return 0;
}

static void *stream_main(void *raw) {
    struct stream *stream = raw;
    stream->failure = push(stream->descriptor, stream->bytes, stream->rounds);
    return NULL;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: pipe_producer MODE ROUNDS\n");
        return 2;
    }
    const char *mode = argv[1];
    unsigned long rounds = strtoul(argv[2], NULL, 10);
    unsigned char *left = malloc(PIPE_RELAY_CHUNK);
    unsigned char *right = malloc(PIPE_RELAY_CHUNK);
    if (left == NULL || right == NULL) {
        free(left);
        free(right);
        return 1;
    }
    memset(left, 'A', PIPE_RELAY_CHUNK);
    memset(right, 'B', PIPE_RELAY_CHUNK);
    int failure = 0;
    if (strcmp(mode, "seq") == 0) {
        for (unsigned long round = 0; round < rounds && failure == 0; round++) {
            failure |= push(1, left, 1);
            failure |= push(2, right, 1);
        }
    } else if (strcmp(mode, "threads") == 0) {
        struct stream streams[2];
        memset(streams, 0, sizeof streams);
        streams[0].descriptor = 1;
        streams[0].bytes = left;
        streams[0].rounds = rounds;
        streams[1].descriptor = 2;
        streams[1].bytes = right;
        streams[1].rounds = rounds;
        for (int at = 0; at < 2; at++) {
            if (pthread_create(&streams[at].thread, NULL, stream_main, &streams[at]) != 0) {
                failure = 1;
            }
        }
        for (int at = 0; at < 2; at++) {
            pthread_join(streams[at].thread, NULL);
            failure |= streams[at].failure;
        }
    } else {
        fprintf(stderr, "pipe_producer: unknown mode %s\n", mode);
        failure = 2;
    }
    free(left);
    free(right);
    return failure;
}
