/* Native byte oracle for compiler-generated stream continuations. The parent
 * withholds pipe input until the host reports a suspended root. For TCP it
 * waits for accept suspension before connecting, then receive suspension
 * before sending, so synchronous wrappers cannot qualify those awaits.
 * Owned by io-model/SCHEDULER-EXPERIMENT.md; retain with the continuation
 * backend qualification or replace when the normal host supplies this seam. */
#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <errno.h>
#include <netinet/in.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <unistd.h>

typedef struct producer {
    int descriptor;
    size_t count;
    int socket;
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
    if (request->socket) assert(shutdown(request->descriptor, SHUT_WR) == 0);
    else assert(close(request->descriptor) == 0);
    return NULL;
}

static void expect_line(FILE *diagnostics, char **line, size_t *capacity, const char *expected) {
    ssize_t count = getline(line, capacity, diagnostics);
    if (count < 0 || strcmp(*line, expected) != 0) {
        fprintf(stderr, "continuation stream: expected %sreceived %s\n",
                expected, count < 0 ? "no diagnostic" : *line);
        abort();
    }
}

static void finish(pid_t child, FILE *diagnostics, int expected_status,
                   int required_route, int require_wait, int socket_operations,
                   const char *kind, size_t bytes, unsigned expected_tasks) {
    char *line = NULL;
    size_t capacity = 0;
    unsigned long long registered, dequeued, helper, ring, immediate;
    unsigned owner_progress, progress_batch, pending_buckets;
    ssize_t count = getline(&line, &capacity, diagnostics);
    assert(count > 0);
    /* A refused operation may finish before registration. A pending refusal
     * can instead leave this one initial suspension diagnostic. */
    if (strcmp(line, "WF continuation host: suspended\n") == 0) {
        count = getline(&line, &capacity, diagnostics);
        assert(count > 0);
    }
    int fields = sscanf(line, "WF continuation host: registered=%llu dequeued=%llu helper=%llu uring=%llu inline=%llu owner_progress=%u progress_batch=%u pending_buckets=%u",
                        &registered, &dequeued, &helper, &ring, &immediate, &owner_progress, &progress_batch, &pending_buckets);
    if (fields != 8) {
        fprintf(stderr, "continuation stream: unexpected diagnostic: %s", line);
        abort();
    }
    assert((!require_wait || registered > 0) && registered == dequeued);
    const char *owner = getenv("WF_CONTINUATION_OWNER_PROGRESS");
    assert(owner_progress == (unsigned)(owner && strcmp(owner, "1") == 0));
    const char *batch = getenv("WF_CONTINUATION_PROGRESS_BATCH");
    assert(progress_batch == (batch ? strtoul(batch, NULL, 10) : 1));
    const char *buckets = getenv("WF_CONTINUATION_PENDING_BUCKETS");
    assert(pending_buckets == (buckets ? strtoul(buckets, NULL, 10) : 1));
    if (required_route == 1) assert(ring > 0);
    if (required_route == 2) assert(ring == 0 && helper > 0);
    if (socket_operations >= 0) {
        assert(getline(&line, &capacity, diagnostics) > 0);
        unsigned long long rings[3], helpers[3];
        fields = sscanf(line, "WF continuation host: accept_ring=%llu accept_helper=%llu connect_ring=%llu connect_helper=%llu receive_ring=%llu receive_helper=%llu",
                        &rings[0], &helpers[0], &rings[1], &helpers[1], &rings[2], &helpers[2]);
        assert(fields == 6);
        for (int i = 0; i < 3; ++i) {
            if (!(socket_operations & (1 << i))) continue;
            assert(rings[i] + helpers[i] > 0);
            if (required_route == 1) assert(rings[i] > 0 && helpers[i] == 0);
            if (required_route == 2) assert(rings[i] == 0 && helpers[i] > 0);
        }
        printf("compiled continuation socket routes: %s accept=%llu/%llu connect=%llu/%llu receive=%llu/%llu (ring/helper)\n",
               kind, rings[0], helpers[0], rings[1], helpers[1], rings[2], helpers[2]);
    }
    if (expected_tasks) {
        unsigned long long tasks, completed, retired, peak;
        assert(getline(&line, &capacity, diagnostics) > 0);
        assert(sscanf(line, "WF continuation host: tasks=%llu completed=%llu retired=%llu peak=%llu",
                      &tasks, &completed, &retired, &peak) == 4);
        assert(tasks == expected_tasks && completed == tasks && retired == tasks && peak == 4);
        printf("compiled continuation staged drain: PASS tasks=%llu completed=%llu retired=%llu peak=%llu\n",
               tasks, completed, retired, peak);
    }
    assert(getline(&line, &capacity, diagnostics) == -1 && feof(diagnostics));
    free(line);
    assert(fclose(diagnostics) == 0);
    int status;
    assert(waitpid(child, &status, 0) == child);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != expected_status) {
        fprintf(stderr, "continuation stream: %s expected exit %d, raw wait status %d\n",
                kind, expected_status, status);
    }
    assert(WIFEXITED(status) && WEXITSTATUS(status) == expected_status);
    printf("compiled continuation stream: PASS transport=%s bytes=%zu registered=%llu helper=%llu uring=%llu\n",
           kind, bytes, registered, helper, ring);
}

static void check(const char *binary, size_t bytes, int required_route, int network) {
    struct sockaddr_in address = {0};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(UINT32_C(0x7f000001));
    char port[16] = {0};
    if (network) {
        int reservation = socket(AF_INET, SOCK_STREAM, 0);
        assert(reservation >= 0);
        assert(bind(reservation, (struct sockaddr *)&address, sizeof(address)) == 0);
        socklen_t length = sizeof(address);
        assert(getsockname(reservation, (struct sockaddr *)&address, &length) == 0);
        assert(close(reservation) == 0);
        assert(snprintf(port, sizeof(port), "%u", (unsigned)ntohs(address.sin_port)) > 0);
    }
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
        if (network) {
            assert(setenv("WF_CONTINUATION_TRACE_SOCKET", "1", 1) == 0);
            assert(setenv("WF_CONTINUATION_REPORT_SOCKET_ROUTES", "1", 1) == 0);
            execl(binary, binary, port, "1", (char *)NULL);
        } else {
            execl(binary, binary, (char *)NULL);
        }
        _exit(127);
    }
    assert(close(input[0]) == 0 && close(output[1]) == 0 && close(errors[1]) == 0);
    FILE *diagnostics = fdopen(errors[0], "r");
    assert(diagnostics != NULL);
    char *line = NULL;
    size_t capacity = 0;
    expect_line(diagnostics, &line, &capacity, "WF continuation host: suspended\n");
    int sender = input[1], receiver = output[0];
    if (network) {
        expect_line(diagnostics, &line, &capacity, "WF continuation host: accept suspended\n");
        int connection = socket(AF_INET, SOCK_STREAM, 0);
        assert(connection >= 0);
        assert(connect(connection, (struct sockaddr *)&address, sizeof(address)) == 0);
        expect_line(diagnostics, &line, &capacity, "WF continuation host: receive suspended\n");
        assert(close(input[1]) == 0);
        sender = receiver = connection;
    }
    producer request = {sender, bytes, network};
    pthread_t writer;
    assert(pthread_create(&writer, NULL, produce, &request) == 0);
    unsigned char buffer[1543];
    size_t received = 0;
    for (;;) {
        ssize_t n = read(receiver, buffer, sizeof(buffer));
        if (n < 0 && errno == EINTR) continue;
        assert(n >= 0);
        if (n == 0) break;
        assert((size_t)n <= bytes - received);
        for (size_t i = 0; i < (size_t)n; ++i) assert(buffer[i] == pattern(received + i));
        received += (size_t)n;
    }
    assert(received == bytes);
    assert(pthread_join(writer, NULL) == 0 && close(receiver) == 0);
    if (network) {
        assert(read(output[0], buffer, sizeof(buffer)) == 0);
        assert(close(output[0]) == 0);
    }
    free(line);
    finish(child, diagnostics, 0, required_route, 1, network ? 5 : -1, network ? "TCP" : "pipe", bytes, 0);
}

/* Existing WF client/refusal programs define this byte/outcome protocol.
 * A refused connect uses a released port, as the ordinary network tests do:
 * macOS times out against a bound-but-not-listening socket. An occupied
 * listen retains the actual native listener for the complete operation. */
static void endpoint(const char *binary, int required_route, int mode) {
    int listener = socket(AF_INET, SOCK_STREAM, 0);
    assert(listener >= 0);
    struct sockaddr_in address = {0};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(UINT32_C(0x7f000001));
    assert(bind(listener, (struct sockaddr *)&address, sizeof(address)) == 0);
    if (mode != 3) assert(listen(listener, 1) == 0);
    socklen_t length = sizeof(address);
    assert(getsockname(listener, (struct sockaddr *)&address, &length) == 0);
    char port[16];
    assert(snprintf(port, sizeof(port), "%u", (unsigned)ntohs(address.sin_port)) > 0);
    if (mode == 3) {
        assert(close(listener) == 0);
        listener = -1;
    }
    int output[2], errors[2];
    assert(pipe(output) == 0 && pipe(errors) == 0);
    pid_t child = fork();
    assert(child >= 0);
    if (child == 0) {
        if (listener >= 0) assert(close(listener) == 0);
        assert(dup2(output[1], STDOUT_FILENO) == STDOUT_FILENO);
        assert(dup2(errors[1], STDERR_FILENO) == STDERR_FILENO);
        assert(close(output[0]) == 0 && close(output[1]) == 0);
        assert(close(errors[0]) == 0 && close(errors[1]) == 0);
        assert(setenv("WF_IO_HELPERS", "4", 1) == 0);
        assert(setenv("WF_CONTINUATION_OBSERVE", "1", 1) == 0);
        assert(setenv("WF_CONTINUATION_REPORT_SOCKET_ROUTES", "1", 1) == 0);
        if (mode == 2) assert(setenv("WF_CONTINUATION_TRACE_SOCKET", "1", 1) == 0);
        execl(binary, binary, port, "1", (char *)NULL);
        _exit(127);
    }
    assert(close(output[1]) == 0 && close(errors[1]) == 0);
    FILE *diagnostics = fdopen(errors[0], "r");
    assert(diagnostics != NULL);
    if (mode == 2) {
        int connection = accept(listener, NULL, NULL);
        assert(connection >= 0);
        unsigned char sent[8];
        size_t received = 0;
        while (received < sizeof(sent)) {
            ssize_t n = read(connection, sent + received, sizeof(sent) - received);
            if (n < 0 && errno == EINTR) continue;
            assert(n > 0);
            received += (size_t)n;
        }
        assert(memcmp(sent, "ABCDEFGH", sizeof(sent)) == 0);
        char *line = NULL;
        size_t capacity = 0;
        expect_line(diagnostics, &line, &capacity, "WF continuation host: suspended\n");
        expect_line(diagnostics, &line, &capacity, "WF continuation host: receive suspended\n");
        free(line);
        for (size_t i = 0; i < sizeof(sent); ++i) {
            unsigned char byte = (unsigned char)('a' + i);
            ssize_t n;
            do { n = write(connection, &byte, 1); } while (n < 0 && errno == EINTR);
            assert(n == 1);
        }
        assert(shutdown(connection, SHUT_WR) == 0 && close(connection) == 0);
    }
    unsigned char published[16];
    size_t count = 0;
    for (;;) {
        assert(count < sizeof(published));
        ssize_t n = read(output[0], published + count, sizeof(published) - count);
        if (n < 0 && errno == EINTR) continue;
        assert(n >= 0);
        if (n == 0) break;
        count += (size_t)n;
    }
    assert(close(output[0]) == 0);
    if (listener >= 0) assert(close(listener) == 0);
    if (mode == 2) assert(count == 8 && memcmp(published, "abcdefgh", 8) == 0);
    else assert(count == 0);
    /* A listen is intentionally a helper operation; only connect/transfer
     * participates in the native-ring requirement. */
    int route = mode == 4 && required_route == 1 ? 2 : required_route;
    finish(child, diagnostics, mode == 4 ? 7 : 0, route, mode == 2, mode == 2 ? 6 : mode == 3 ? 2 : 0,
           mode == 2 ? "TCP-client" : mode == 3 ? "TCP-refused" : "TCP-occupied", count, 0);
}

/* All four peers connect before any sends. The last connected peer must
 * complete first while the earlier three remain silent. A serial loop cannot
 * satisfy this protocol. The repeated form also reuses every issuer slot. */
static void staged(const char *binary, int required_route, unsigned batches) {
    struct sockaddr_in address = {0};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(UINT32_C(0x7f000001));
    int reservation = socket(AF_INET, SOCK_STREAM, 0);
    assert(reservation >= 0);
    assert(bind(reservation, (struct sockaddr *)&address, sizeof(address)) == 0);
    socklen_t length = sizeof(address);
    assert(getsockname(reservation, (struct sockaddr *)&address, &length) == 0);
    assert(close(reservation) == 0);
    char port[16], peers[16];
    assert(snprintf(port, sizeof(port), "%u", (unsigned)ntohs(address.sin_port)) > 0);
    assert(snprintf(peers, sizeof(peers), "%u", batches * 4) > 0);
    int errors[2], output[2];
    assert(pipe(errors) == 0 && pipe(output) == 0);
    pid_t child = fork();
    assert(child >= 0);
    if (!child) {
        assert(dup2(errors[1], STDERR_FILENO) == STDERR_FILENO);
        assert(dup2(output[1], STDOUT_FILENO) == STDOUT_FILENO);
        assert(close(errors[0]) == 0 && close(errors[1]) == 0);
        assert(close(output[0]) == 0 && close(output[1]) == 0);
        assert(setenv("WF_IO_HELPERS", "4", 1) == 0);
        assert(setenv("WF_CONTINUATION_WINDOW", "4", 1) == 0);
        assert(setenv("WF_CONTINUATION_OBSERVE", "1", 1) == 0);
        assert(setenv("WF_CONTINUATION_TRACE_SOCKET", "1", 1) == 0);
        assert(setenv("WF_CONTINUATION_REPORT_SOCKET_ROUTES", "1", 1) == 0);
        assert(setenv("WF_CONTINUATION_TRACE_STAGED", "1", 1) == 0);
        execl(binary, binary, port, peers, (char *)NULL);
        _exit(127);
    }
    assert(close(errors[1]) == 0 && close(output[1]) == 0);
    FILE *diagnostics = fdopen(errors[0], "r");
    assert(diagnostics);
    char *line = NULL;
    size_t capacity = 0;
    expect_line(diagnostics, &line, &capacity, "WF continuation host: suspended\n");
    expect_line(diagnostics, &line, &capacity, "WF continuation host: accept suspended\n");
    for (unsigned batch = 0; batch < batches; ++batch) {
        int sockets[4];
        for (unsigned i = 0; i < 4; ++i) {
            sockets[i] = socket(AF_INET, SOCK_STREAM, 0);
            assert(sockets[i] >= 0);
            assert(connect(sockets[i], (struct sockaddr *)&address, sizeof(address)) == 0);
        }
        if (batch == 0) {
            expect_line(diagnostics, &line, &capacity, "WF continuation host: receive suspended\n");
            expect_line(diagnostics, &line, &capacity, "WF continuation host: staged suspended=4\n");
        }
        for (unsigned i = 4; i-- > 0;) {
            unsigned char sent = (unsigned char)(31 + batch * 4 + i), received;
            assert(write(sockets[i], &sent, 1) == 1);
            assert(shutdown(sockets[i], SHUT_WR) == 0);
            assert(read(sockets[i], &received, 1) == 1 && received == sent);
            assert(read(sockets[i], &received, 1) == 0);
            assert(close(sockets[i]) == 0);
        }
    }
    free(line);
    unsigned char unexpected;
    assert(read(output[0], &unexpected, 1) == 0 && close(output[0]) == 0);
    finish(child, diagnostics, 0, required_route, 1, 5, "TCP-staged", batches * 4, batches * 4);
}

int main(int argc, char **argv) {
    /* A missing protocol event fails qualification; it must not leave a CI
     * job parked indefinitely. This watchdog never affects source checking. */
    alarm(120);
    int mode = 0;
    if (argc > 1) {
        if (strcmp(argv[1], "--tcp") == 0) mode = 1;
        else if (strcmp(argv[1], "--client") == 0) mode = 2;
        else if (strcmp(argv[1], "--refused") == 0) mode = 3;
        else if (strcmp(argv[1], "--occupied") == 0) mode = 4;
        else if (strcmp(argv[1], "--fanout") == 0) mode = 5;
        else if (strcmp(argv[1], "--staged") == 0) mode = 6;
    }
    if (mode) { --argc; ++argv; }
    assert(argc == 2 || (argc == 3 &&
           (strcmp(argv[2], "--require-ring") == 0 || strcmp(argv[2], "--require-helper") == 0)));
    int route = argc == 2 ? 0 : strcmp(argv[2], "--require-ring") == 0 ? 1 : 2;
    if (mode >= 5) staged(argv[1], route, mode == 5 ? 1 : 3);
    else if (mode >= 2) endpoint(argv[1], route, mode);
    else {
        check(argv[1], 0, route, mode);
        check(argv[1], 65673, route, mode);
    }
    return 0;
}
