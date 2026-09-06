/* Untimed stream/lifetime checks for the epoll representation comparison.
 * Usage: stream_check SERVER echo|compute|truncated WORKERS
 *        stream_check SERVER resident WORKERS CONNECTIONS BYTES PREFIX
 *        stream_check launch SERVER [ARGUMENTS...]
 * Owned by scheduler-stackful/stackful-check and scheduler-pages; retire with
 * those comparisons. Residency captures only byte-checked live connections.
 * The compute answers below are fixed independent protocol vectors. */
#define _POSIX_C_SOURCE 200809L
#include <arpa/inet.h>
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/resource.h>
#include <sys/wait.h>
#if defined(__linux__)
#include <sys/prctl.h>
#endif
#include <time.h>
#include <unistd.h>

static volatile sig_atomic_t server_pid;
static uint16_t server_port;
static const char *mode;

static void stop_server(void) {
    if (server_pid > 0) kill((pid_t)server_pid, SIGKILL);
}

static void timed_out(int signal_number) {
    (void)signal_number;
    stop_server();
    _exit(124);
}

static void require(int condition, const char *reason) {
    if (!condition) {
        fprintf(stderr, "stream_check: %s\n", reason);
        stop_server();
        exit(1);
    }
}

/* Benchmark process policy only. The default caller changes nothing. Both
 * timed launches and untimed residency children use the same inherited flag. */
static void select_page_policy(void) {
    const char *setting = getenv("WF_BENCH_THP_DISABLE");
    if (setting == NULL) return;
    require(strcmp(setting, "0") == 0 || strcmp(setting, "1") == 0, "invalid THP policy");
#if defined(__linux__)
    unsigned long disabled = (unsigned long)(setting[0] == '1');
    require(prctl(PR_SET_THP_DISABLE, disabled, 0UL, 0UL, 0UL) == 0, "THP policy failed");
    require(prctl(PR_GET_THP_DISABLE, 0UL, 0UL, 0UL, 0UL) == (int)disabled, "THP policy did not apply");
#else
    require(0, "process THP policy requires Linux");
#endif
}

static unsigned bounded_number(const char *text, unsigned limit) {
    char *end = NULL;
    unsigned long value = strtoul(text, &end, 10);
    require(end != text && *end == '\0' && value != 0 && value <= limit, "invalid residency size");
    return (unsigned)value;
}

static void snapshot_process(const char *prefix, const char *name) {
    /* A dash permits portable byte/lifetime qualification without /proc.
     * The Linux experiment caller requires both actual snapshot files. */
    if (strcmp(prefix, "-") == 0) return;
    char source[128];
    char destination[4096];
    int written = snprintf(source, sizeof source, "/proc/%ld/%s", (long)server_pid, name);
    require(written > 0 && (size_t)written < sizeof source, "snapshot source path overflow");
    written = snprintf(destination, sizeof destination, "%s.%s", prefix, name);
    require(written > 0 && (size_t)written < sizeof destination, "snapshot output path overflow");
    FILE *input = fopen(source, "r");
    require(input != NULL, "process snapshot unavailable");
    FILE *output = fopen(destination, "w");
    require(output != NULL, "snapshot output unavailable");
    unsigned char bytes[16384];
    size_t count;
    while ((count = fread(bytes, 1u, sizeof bytes, input)) != 0u) {
        require(fwrite(bytes, 1u, count, output) == count, "snapshot write failed");
    }
    require(ferror(input) == 0, "snapshot read failed");
    require(fclose(input) == 0 && fclose(output) == 0, "snapshot close failed");
}

static void delay_ms(unsigned milliseconds) {
    struct timespec delay = {milliseconds / 1000u, (long)(milliseconds % 1000u) * 1000000L};
    while (nanosleep(&delay, &delay) != 0) require(errno == EINTR, "nanosleep failed");
}

static int connect_peer(void) {
    struct sockaddr_in address = {0};
    address.sin_family = AF_INET;
    require(inet_pton(AF_INET, "127.0.0.1", &address.sin_addr) == 1, "loopback address failed");
    address.sin_port = htons(server_port);
    for (unsigned attempt = 0; attempt < 500; attempt++) {
        int descriptor = socket(AF_INET, SOCK_STREAM, 0);
        require(descriptor >= 0, "socket failed");
        if (connect(descriptor, (struct sockaddr *)&address, sizeof address) == 0) return descriptor;
        close(descriptor);
        delay_ms(2);
    }
    require(0, "server did not accept a connection");
    return -1;
}

static void send_all(int descriptor, const unsigned char *bytes, size_t length) {
    while (length != 0) {
        ssize_t moved = send(descriptor, bytes, length, 0);
        if (moved < 0 && errno == EINTR) continue;
        require(moved > 0, "send failed");
        bytes += moved;
        length -= (size_t)moved;
    }
}

static void receive_all(int descriptor, unsigned char *bytes, size_t length) {
    while (length != 0) {
        ssize_t moved = recv(descriptor, bytes, length, 0);
        if (moved < 0 && errno == EINTR) continue;
        require(moved > 0, "response ended early");
        bytes += moved;
        length -= (size_t)moved;
    }
}

struct peer {
    unsigned index;
    int descriptor;
};

static unsigned char echo_byte(unsigned peer, size_t offset) {
    return (unsigned char)((offset * 37u + peer * 53u) ^ (offset >> 11));
}

static void check_residency(unsigned count, unsigned length, const char *prefix) {
    int *descriptors = malloc((size_t)count * sizeof *descriptors);
    unsigned char *bytes = malloc(length);
    require(descriptors != NULL && bytes != NULL, "residency client allocation failed");
    for (unsigned index = 0u; index < count; index++) {
        descriptors[index] = connect_peer();
        for (unsigned at = 0u; at < length; at++) bytes[at] = echo_byte(index, at);
        send_all(descriptors[index], bytes, length);
        memset(bytes, 0, length);
        receive_all(descriptors[index], bytes, length);
        for (unsigned at = 0u; at < length; at++) {
            require(bytes[at] == echo_byte(index, at), "residency echo mismatch");
        }
    }
    /* Every connection has completed a checked exchange and remains open.
     * Taking the snapshot here distinguishes live storage from peak startup
     * RSS or allocator pages retained after connections have closed. */
    snapshot_process(prefix, "smaps");
    snapshot_process(prefix, "status");
    for (unsigned index = 0u; index < count; index++) {
        require(shutdown(descriptors[index], SHUT_WR) == 0, "residency half-close failed");
        unsigned char extra;
        ssize_t taken;
        do { taken = recv(descriptors[index], &extra, 1u, 0); } while (taken < 0 && errno == EINTR);
        require(taken == 0, "residency peer did not finish");
        require(close(descriptors[index]) == 0, "residency close failed");
    }
    free(bytes);
    free(descriptors);
}

#define ECHO_BYTES (2u * 1024u * 1024u)
static void *echo_writer(void *raw) {
    struct peer *peer = raw;
    unsigned char bytes[4096];
    for (size_t offset = 0; offset < ECHO_BYTES; offset += sizeof bytes) {
        for (size_t at = 0; at < sizeof bytes; at++) bytes[at] = echo_byte(peer->index, offset + at);
        send_all(peer->descriptor, bytes, sizeof bytes);
    }
    require(shutdown(peer->descriptor, SHUT_WR) == 0, "half-close failed");
    return NULL;
}

static void check_echo(struct peer *peer) {
    pthread_t writer;
    require(pthread_create(&writer, NULL, echo_writer, peer) == 0, "writer creation failed");
    /* The observed server has a small SO_SNDBUF. Other peers run while this
     * receiver waits, exercising the copy out of the shared receive scratch. */
    delay_ms(100);
    unsigned char bytes[4096];
    for (size_t offset = 0; offset < ECHO_BYTES; offset += sizeof bytes) {
        receive_all(peer->descriptor, bytes, sizeof bytes);
        for (size_t at = 0; at < sizeof bytes; at++) {
            require(bytes[at] == echo_byte(peer->index, offset + at), "echo byte mismatch after back pressure");
        }
    }
    require(pthread_join(writer, NULL) == 0, "writer join failed");
}

static void check_compute(struct peer *peer) {
    static const uint64_t seeds[] = {0, UINT64_C(11400714819323198485), UINT64_MAX};
    static const uint64_t rounds[] = {1, 65536, 4096};
    static const uint64_t answers[] = {
        UINT64_C(1442695040888963407), UINT64_C(14034923464053623880), UINT64_C(16385165208160242592)
    };
    for (unsigned vector = 0; vector < 3; vector++) {
        unsigned char request[64] = {0};
        for (unsigned at = 0; at < 8; at++) {
            request[7u - at] = (unsigned char)(seeds[vector] >> (at * 8u));
            request[15u - at] = (unsigned char)(rounds[vector] >> (at * 8u));
        }
        for (unsigned at = 0; at < sizeof request; at++) {
            send_all(peer->descriptor, request + at, 1);
            delay_ms(1);
        }
        unsigned char response[64];
        receive_all(peer->descriptor, response, sizeof response);
        for (unsigned at = 0; at < sizeof response; at++) {
            require(response[at] == ((answers[vector] >> at) & 1u), "fragmented compute response mismatch");
        }
    }
    require(shutdown(peer->descriptor, SHUT_WR) == 0, "compute half-close failed");
}

static void *run_peer(void *raw) {
    struct peer *peer = raw;
    peer->descriptor = connect_peer();
    if (strcmp(mode, "echo") == 0) check_echo(peer);
    else check_compute(peer);
    unsigned char extra;
    ssize_t taken;
    do { taken = recv(peer->descriptor, &extra, 1, 0); } while (taken < 0 && errno == EINTR);
    require(taken == 0, "server did not close after the complete half-closed stream");
    close(peer->descriptor);
    return NULL;
}

int main(int argc, char **argv) {
    if (argc >= 3 && strcmp(argv[1], "launch") == 0) {
        select_page_policy();
        execv(argv[2], argv + 2);
        require(0, "server launch failed");
    }
    require(argc == 4 || argc == 7, "expected SERVER MODE WORKERS [CONNECTIONS BYTES PREFIX]");
    mode = argv[2];
    int resident = strcmp(mode, "resident") == 0;
    require(strcmp(mode, "echo") == 0 || strcmp(mode, "compute") == 0 ||
            strcmp(mode, "truncated") == 0 || resident, "unknown mode");
    require(argc == (resident ? 7 : 4), "incorrect arguments for stream mode");
    unsigned count = resident ? bounded_number(argv[4], 4096u) : 0u;
    unsigned bytes = resident ? bounded_number(argv[5], 65536u) : 0u;
    if (resident) {
        struct rlimit limit;
        require(getrlimit(RLIMIT_NOFILE, &limit) == 0, "descriptor limit unavailable");
        rlim_t wanted = (rlim_t)count + 128u;
        if (limit.rlim_cur < wanted) {
            require(limit.rlim_max >= wanted, "insufficient descriptor ceiling");
            limit.rlim_cur = wanted;
            require(setrlimit(RLIMIT_NOFILE, &limit) == 0, "descriptor limit change failed");
        }
    }
    signal(SIGPIPE, SIG_IGN);
    signal(SIGALRM, timed_out);
    atexit(stop_server);
    alarm(30);

    int probe = socket(AF_INET, SOCK_STREAM, 0);
    require(probe >= 0, "port socket failed");
    struct sockaddr_in address = {0};
    address.sin_family = AF_INET;
    require(inet_pton(AF_INET, "127.0.0.1", &address.sin_addr) == 1, "loopback address failed");
    require(bind(probe, (struct sockaddr *)&address, sizeof address) == 0, "port bind failed");
    socklen_t length = sizeof address;
    require(getsockname(probe, (struct sockaddr *)&address, &length) == 0, "port query failed");
    server_port = ntohs(address.sin_port);
    close(probe);
    char port_text[16];
    snprintf(port_text, sizeof port_text, "%u", server_port);
    int truncated = strcmp(mode, "truncated") == 0;
    pid_t child = fork();
    require(child >= 0, "fork failed");
    if (child == 0) {
        select_page_policy();
        const char *server_cpus = getenv("WF_BENCH_SERVER_CPUS");
        if (server_cpus != NULL) {
            /* The Linux residency caller pins its client separately. taskset
             * execs in place, preserving the PID used for the live snapshot. */
            execlp("taskset", "taskset", "-c", server_cpus, argv[1], port_text,
                   resident ? argv[4] : (truncated ? "1" : "4"), "--threads", argv[3], (char *)NULL);
            _exit(127);
        }
        execl(argv[1], argv[1], port_text, resident ? argv[4] : (truncated ? "1" : "4"), "--threads", argv[3], (char *)NULL);
        _exit(127);
    }
    server_pid = child;
    if (resident) {
        check_residency(count, bytes, argv[6]);
    } else if (truncated) {
        int descriptor = connect_peer();
        unsigned char prefix[17] = {0};
        send_all(descriptor, prefix, sizeof prefix);
        delay_ms(20);
        close(descriptor);
    } else {
        struct peer peers[4];
        pthread_t threads[4];
        for (unsigned at = 0; at < 4; at++) {
            peers[at].index = at;
            require(pthread_create(&threads[at], NULL, run_peer, &peers[at]) == 0, "peer creation failed");
        }
        for (unsigned at = 0; at < 4; at++) require(pthread_join(threads[at], NULL) == 0, "peer join failed");
    }
    int status;
    require(waitpid(child, &status, 0) == child, "server wait failed");
    server_pid = 0;
    require(WIFEXITED(status) && WEXITSTATUS(status) == (truncated ? 1 : 0), "unexpected server exit");
    alarm(0);
    printf("stream_check: PASS mode=%s workers=%s", mode, argv[3]);
    if (resident) printf(" connections=%u bytes=%u", count, bytes);
    putchar('\n');
    return 0;
}
