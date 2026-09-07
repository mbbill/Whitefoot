/* Native byte oracle for compiler-generated stream continuations. The parent
 * withholds all input until the experimental host reports a suspended root.
 * Owned by io-model/SCHEDULER-EXPERIMENT.md; retain with the continuation
 * backend qualification or replace when the normal host supplies this seam. */
#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

typedef struct producer {
    int descriptor;
    size_t count;
} producer;

static unsigned char pattern(size_t offset) {
    return (unsigned char)((offset * 17 + 3) % 251);
}

static void *produce(void *opaque) {
    const producer *request = opaque;
    unsigned char bytes[997];
    size_t offset = 0;
    while (offset < request->count) {
        size_t count = request->count - offset;
        if (count > sizeof(bytes)) count = sizeof(bytes);
        for (size_t i = 0; i < count; ++i) bytes[i] = pattern(offset + i);
        size_t written = 0;
        while (written < count) {
            ssize_t n = write(request->descriptor, bytes + written, count - written);
            if (n < 0 && errno == EINTR) continue;
            assert(n > 0);
            written += (size_t)n;
        }
        offset += count;
    }
    assert(close(request->descriptor) == 0);
    return NULL;
}

static void check(const char *binary, size_t bytes, int required_route) {
    int input[2], output[2], errors[2];
    assert(pipe(input) == 0 && pipe(output) == 0 && pipe(errors) == 0);
    pid_t child = fork();
    assert(child >= 0);
    if (child == 0) {
        assert(dup2(input[0], STDIN_FILENO) == STDIN_FILENO);
        assert(dup2(output[1], STDOUT_FILENO) == STDOUT_FILENO);
        assert(dup2(errors[1], STDERR_FILENO) == STDERR_FILENO);
        assert(close(input[0]) == 0 && close(input[1]) == 0);
        assert(close(output[0]) == 0 && close(output[1]) == 0);
        assert(close(errors[0]) == 0 && close(errors[1]) == 0);
        assert(setenv("WF_IO_HELPERS", "4", 1) == 0);
        assert(setenv("WF_CONTINUATION_OBSERVE", "1", 1) == 0);
        execl(binary, binary, (char *)NULL);
        _exit(127);
    }
    assert(close(input[0]) == 0 && close(output[1]) == 0 && close(errors[1]) == 0);
    FILE *diagnostics = fdopen(errors[0], "r");
    assert(diagnostics != NULL);
    char *line = NULL;
    size_t capacity = 0;
    ssize_t count = getline(&line, &capacity, diagnostics);
    if (count < 0 || strcmp(line, "WF continuation host: suspended\n") != 0) {
        fprintf(stderr, "continuation stream: root did not suspend before input: %s\n",
                count < 0 ? "no diagnostic" : line);
        abort();
    }
    producer request = {input[1], bytes};
    pthread_t writer;
    assert(pthread_create(&writer, NULL, produce, &request) == 0);
    unsigned char buffer[1543];
    size_t received = 0;
    for (;;) {
        ssize_t n = read(output[0], buffer, sizeof(buffer));
        if (n < 0 && errno == EINTR) continue;
        assert(n >= 0);
        if (n == 0) break;
        assert((size_t)n <= bytes - received);
        for (size_t i = 0; i < (size_t)n; ++i) assert(buffer[i] == pattern(received + i));
        received += (size_t)n;
    }
    assert(received == bytes);
    assert(close(output[0]) == 0 && pthread_join(writer, NULL) == 0);
    unsigned long long registered, dequeued, helper, ring, immediate;
    count = getline(&line, &capacity, diagnostics);
    assert(count > 0);
    int fields = sscanf(line, "WF continuation host: registered=%llu dequeued=%llu helper=%llu uring=%llu inline=%llu",
                        &registered, &dequeued, &helper, &ring, &immediate);
    if (fields != 5) {
        fprintf(stderr, "continuation stream: unexpected diagnostic: %s", line);
        abort();
    }
    assert(registered > 0 && registered == dequeued);
    if (required_route == 1) assert(ring > 0);
    if (required_route == 2) assert(ring == 0 && helper > 0);
    assert(getline(&line, &capacity, diagnostics) == -1 && feof(diagnostics));
    free(line);
    assert(fclose(diagnostics) == 0);
    int status;
    assert(waitpid(child, &status, 0) == child);
    assert(WIFEXITED(status) && WEXITSTATUS(status) == 0);
    printf("compiled continuation stream: PASS bytes=%zu registered=%llu helper=%llu uring=%llu\n",
           bytes, registered, helper, ring);
}

int main(int argc, char **argv) {
    assert(argc == 2 || (argc == 3 &&
           (strcmp(argv[2], "--require-ring") == 0 || strcmp(argv[2], "--require-helper") == 0)));
    int route = argc == 2 ? 0 : strcmp(argv[2], "--require-ring") == 0 ? 1 : 2;
    check(argv[1], 0, route);
    check(argv[1], 65673, route);
    return 0;
}
