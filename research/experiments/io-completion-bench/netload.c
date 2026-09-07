/* glibc hides the POSIX and Linux entry points this file calls when the
 * translation unit is compiled as strict C11, so the namespace is asked for
 * before any header. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/* The one load generator every server of the TCP echo workload is measured
 * with.
 *
 *   netload PORT CONNECTIONS ROUNDTRIPS BYTES [--threads T]
 *           [--compute ROUNDS] [--heavy-every N] [--admit] [--duration-ms MS]
 *           [--light-per-second RATE]
 *
 * Compute mode uses compute_protocol.h: every Nth connection requests ROUNDS
 * recurrence steps, the others request zero. Expected results are computed
 * before timing. Class tails and spans describe this closed-loop burst,
 * not an open-loop service-level latency guarantee.
 * With --duration-ms, admitted compute connections keep issuing requests until
 * one common deadline, then drain their last request. ROUNDTRIPS is instead a
 * per-connection storage ceiling; reaching it early fails the sample. Results
 * are captured without allocation and checked after exchange timing, so the
 * oracle does not compete with the measured server or require a fixed count.
 * With --light-per-second, each light connection instead has a fixed arrival
 * schedule, staggered across peers. A connection still has one wire request
 * outstanding; later arrivals queue logically in the client and are all
 * drained. Their latency starts at the intended arrival, including that queue.
 * Heavy peers remain closed-loop. Dispatch delay is reported separately.
 *
 * It opens CONNECTIONS connections to 127.0.0.1:PORT, spread over T client
 * threads each holding its own epoll instance, and reports two phases:
 *
 *   the connect phase   from the first connect call until every connection is
 *                       established, which is what "connections accepted per
 *                       second" is measured from
 *   the exchange phase  every connection performing ROUNDTRIPS round trips of
 *                       a BYTES-byte message, all connections active at once,
 *                       which is what round trips per second, bytes per second
 *                       and the latency distribution are measured from
 *
 * Every echoed byte is compared with the byte that was sent. A run that
 * receives anything else, that is refused a connection, or whose peer closes
 * mid-exchange prints one line on stderr and exits nonzero; it never reports a
 * time. That is what makes the three server lines comparable: the generator
 * cannot tell them apart, and none of them can buy a number by publishing the
 * wrong bytes.
 *
 * Allocation is done once, before the first connect: the per-connection send
 * and receive buffers, and one array of CONNECTIONS*ROUNDTRIPS 32-bit
 * microsecond latencies. Nothing is allocated per round trip.
 *
 * One line on stdout, tab separated:
 *
 *   connect_us=  exchange_us=  roundtrips=  rt_per_s=  bytes_per_s=
 *   p50_us=  p90_us=  p99_us=  max_us=
 *
 * bytes_per_s counts the payload once, not once per direction: a round trip of
 * BYTES bytes counts BYTES. */
#include <errno.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <pthread.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <time.h>
#include <unistd.h>

#include "compute_protocol.h"

/* A client-service control, independent of the server's execution budget.
 * Zero retains the original drive-until-EAGAIN client and its data layout. */
#ifndef WF_NETLOAD_SERVICE_ROUNDS
#define WF_NETLOAD_SERVICE_ROUNDS 0u
#endif
#if WF_NETLOAD_SERVICE_ROUNDS < 0 || WF_NETLOAD_SERVICE_ROUNDS > 65536u
#error "The client service budget must be in 0..65536"
#endif

struct connection {
    int descriptor;
    int established;
    int finished;
    unsigned char *outgoing;
    unsigned char *incoming;
    uint64_t sent;
    uint64_t received;
    uint64_t round;
    struct timespec started;
    struct timespec first_started;
    struct timespec last_finished;
    uint64_t index;
    uint64_t light_phase_ns;
    uint64_t planned;
    uint64_t completed_by_deadline;
    int waiting_arrival;
    struct timespec due;
#if WF_NETLOAD_SERVICE_ROUNDS
    struct connection *ready_next;
    int queued;
#endif
};

struct client {
    pthread_t thread;
    int epoll;
    struct connection *first;
    uint64_t count;
    uint64_t outstanding;
    int admitting;
#if WF_NETLOAD_SERVICE_ROUNDS
    struct connection *ready_head;
    struct connection *ready_tail;
    uint64_t ready_count;
    uint64_t service_yields;
#endif
};

static uint64_t option_connections;
static uint64_t option_roundtrips;
static uint64_t option_bytes;
static uint16_t option_port;
static uint32_t *latency_us;
static struct connection *connections;
static struct client *clients;
static pthread_barrier_t gate;
static int option_compute;
static int option_admit;
static uint64_t option_duration_ms;
static uint64_t option_light_per_second;
static struct timespec exchange_origin;
static struct timespec exchange_deadline;
static uint64_t option_compute_rounds;
static uint64_t option_heavy_every = 4;
/* Expected values in fixed-count mode; captured responses in duration mode. */
static uint64_t *compute_values;
static uint32_t *dispatch_us;

/* One line on stderr and out, from whichever thread reached the failure
 * first. A run that cannot publish the right bytes must not publish a time,
 * and the other threads have nothing left to do. */
static pthread_mutex_t failure_lock = PTHREAD_MUTEX_INITIALIZER;

static void fail(const char *format, ...) {
    pthread_mutex_lock(&failure_lock);
    va_list arguments;
    va_start(arguments, format);
    fprintf(stderr, "netload: ");
    vfprintf(stderr, format, arguments);
    fprintf(stderr, "\n");
    va_end(arguments);
    fflush(stderr);
    _exit(1);
}

static double seconds_between(struct timespec start, struct timespec end) {
    return (double)(end.tv_sec - start.tv_sec) + (double)(end.tv_nsec - start.tv_nsec) / 1e9;
}

static uint64_t microseconds_between(struct timespec start, struct timespec end) {
    double value = seconds_between(start, end) * 1e6;
    if (value < 0.0) {
        return 0;
    }
    return (uint64_t)value;
}

static int paced_link(const struct client *owner, const struct connection *link) {
    return option_light_per_second && !owner->admitting &&
        link->index % option_heavy_every != 0;
}

static void schedule_arrival(struct connection *link) {
    uint64_t offset = link->light_phase_ns +
        link->round * UINT64_C(1000000000) / option_light_per_second;
    link->due = exchange_origin;
    link->due.tv_sec += (time_t)(offset / UINT64_C(1000000000));
    link->due.tv_nsec += (long)(offset % UINT64_C(1000000000));
    if (link->due.tv_nsec >= 1000000000L) {
        link->due.tv_sec++;
        link->due.tv_nsec -= 1000000000L;
    }
    link->waiting_arrival = 1;
}

static void raise_descriptor_limit(uint64_t wanted) {
    struct rlimit limit;
    if (getrlimit(RLIMIT_NOFILE, &limit) != 0) {
        return;
    }
    if (limit.rlim_cur >= wanted + 64) {
        return;
    }
    rlim_t target = (rlim_t)(wanted + 64);
    if (limit.rlim_max != RLIM_INFINITY && target > limit.rlim_max) {
        target = limit.rlim_max;
    }
    limit.rlim_cur = target;
    setrlimit(RLIMIT_NOFILE, &limit);
}

static int compare_u32(const void *left, const void *right) {
    uint32_t a = *(const uint32_t *)left;
    uint32_t b = *(const uint32_t *)right;
    return (a > b) - (a < b);
}

/* Nearest-rank percentile over the sorted samples. */
static uint32_t percentile(const uint32_t *sorted, uint64_t count, uint64_t share) {
    if (count == 0) {
        return 0;
    }
    uint64_t rank = (count * share + 99) / 100;
    if (rank == 0) {
        rank = 1;
    }
    if (rank > count) {
        rank = count;
    }
    return sorted[rank - 1];
}

/* The message a connection sends on every round. The payload depends on the
 * connection and on the position within the message, so a server that echoed
 * one connection's bytes to another, or that reordered a message, fails the
 * comparison rather than producing a fast time. The first byte carries the
 * round, so a stale echo cannot pass either. */
static void fill_message(struct connection *link) {
    for (uint64_t at = 0; at < option_bytes; at++) {
        link->outgoing[at] = (unsigned char)(link->index * 131u + at * 7u + 11u);
    }
}

static void begin_round(struct connection *link, int admitting) {
    link->sent = 0;
    link->received = 0;
    if (option_compute) {
        uint64_t rounds = !admitting && link->index % option_heavy_every == 0 ? option_compute_rounds : 0;
        compute_request(link->outgoing,
                        compute_seed(link->index, admitting ? UINT64_MAX : link->round), rounds);
    } else {
        link->outgoing[0] = admitting ? 250u : (unsigned char)(link->round * 17u + 3u);
    }
    clock_gettime(CLOCK_MONOTONIC, &link->started);
    if (option_light_per_second && !admitting && link->index % option_heavy_every != 0) {
        uint64_t delay = microseconds_between(link->due, link->started);
        dispatch_us[link->index * option_roundtrips + link->round] =
            delay > UINT32_MAX ? UINT32_MAX : (uint32_t)delay;
        link->started = link->due;
        link->waiting_arrival = 0;
    }
    if (link->round == 0) link->first_started = link->started;
}

#if WF_NETLOAD_SERVICE_ROUNDS
static void enqueue_client(struct client *owner, struct connection *link) {
    if (link->queued) return;
    link->queued = 1;
    link->ready_next = NULL;
    if (owner->ready_tail) owner->ready_tail->ready_next = link;
    else owner->ready_head = link;
    owner->ready_tail = link;
    owner->ready_count++;
    owner->service_yields++;
}
#endif

/* Drives one connection as far as it will go without blocking: send what is
 * left of the current message, receive what is left of its echo, verify it,
 * record the round trip, and start the next one. Returns when the kernel has
 * no more room to send or no more bytes to give, or when the optional client
 * service budget puts the next round on the owner's ready FIFO. */
static void pump(struct client *owner, struct connection *link) {
#if WF_NETLOAD_SERVICE_ROUNDS
    unsigned turns = 0;
#endif
    for (;;) {
        while (link->sent < option_bytes) {
            ssize_t moved = send(link->descriptor, link->outgoing + link->sent,
                                 (size_t)(option_bytes - link->sent), MSG_NOSIGNAL);
            if (moved > 0) {
                link->sent += (uint64_t)moved;
                continue;
            }
            if (moved < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
                break;
            }
            fail("connection %llu: send failed after %llu of %llu bytes: %s",
                 (unsigned long long)link->index, (unsigned long long)link->sent,
                 (unsigned long long)option_bytes, strerror(errno));
        }
        while (link->received < option_bytes) {
            ssize_t moved = recv(link->descriptor, link->incoming + link->received,
                                 (size_t)(option_bytes - link->received), 0);
            if (moved > 0) {
                link->received += (uint64_t)moved;
                continue;
            }
            if (moved == 0) {
                fail("connection %llu: the peer closed after echoing %llu of %llu bytes of round %llu",
                     (unsigned long long)link->index, (unsigned long long)link->received,
                     (unsigned long long)option_bytes, (unsigned long long)link->round);
            }
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
                return;
            }
            fail("connection %llu: receive failed: %s", (unsigned long long)link->index,
                 strerror(errno));
        }
        unsigned char computed[COMPUTE_BYTES];
        const unsigned char *expected = link->outgoing;
        if (option_compute && option_duration_ms && !owner->admitting) {
            uint64_t value = 0;
            for (unsigned at = 0; at < COMPUTE_BYTES; at++) {
                if (link->incoming[at] > 1u)
                    fail("connection %llu round %llu: non-bit compute response",
                         (unsigned long long)link->index, (unsigned long long)link->round);
                value |= (uint64_t)link->incoming[at] << at;
            }
            compute_values[link->index * option_roundtrips + link->round] = value;
            expected = link->incoming; /* Full recurrence verification follows timing. */
        } else if (option_compute) {
            uint64_t value = owner->admitting ? compute_seed(link->index, UINT64_MAX)
                : compute_values[link->index * option_roundtrips + link->round];
            compute_encode(computed, value);
            expected = computed;
        }
        if (memcmp(link->incoming, expected, (size_t)option_bytes) != 0) {
            uint64_t at = 0;
            while (at < option_bytes && link->incoming[at] == expected[at]) {
                at++;
            }
            fail("connection %llu round %llu: byte %llu is 0x%02x, expected 0x%02x",
                 (unsigned long long)link->index, (unsigned long long)link->round,
                 (unsigned long long)at, link->incoming[at], expected[at]);
        }
        struct timespec now;
        clock_gettime(CLOCK_MONOTONIC, &now);
        uint64_t elapsed = microseconds_between(link->started, now);
        if (!owner->admitting) {
            latency_us[link->index * option_roundtrips + link->round] =
                elapsed > UINT32_MAX ? UINT32_MAX : (uint32_t)elapsed;
            if (option_duration_ms && seconds_between(now, exchange_deadline) >= 0.0)
                link->completed_by_deadline++;
        }
        link->round++;
        int done = link->round == (owner->admitting ? 1u : option_roundtrips);
        if (paced_link(owner, link)) {
            done = link->round == link->planned;
        } else if (option_duration_ms && !owner->admitting) {
            done = seconds_between(exchange_deadline, now) >= 0.0;
            if (!done && link->round == option_roundtrips)
                fail("connection %llu exhausted its duration sample capacity",
                     (unsigned long long)link->index);
        }
        if (done) {
            link->last_finished = now;
            link->finished = 1;
            owner->outstanding--;
            return;
        }
        if (paced_link(owner, link)) {
            schedule_arrival(link);
            return;
        }
        begin_round(link, owner->admitting);
#if WF_NETLOAD_SERVICE_ROUNDS
        if (++turns == WF_NETLOAD_SERVICE_ROUNDS) {
            /* The next request is ready even if no fresh kernel edge arrives.
             * Keep it in an owner-local FIFO before returning to arrivals. */
            enqueue_client(owner, link);
            return;
        }
#endif
    }
}

/* The admission handshake and the timed exchange use the same byte verifier. */
static void exchange(struct client *owner) {
    struct epoll_event events[256];
    owner->outstanding = owner->count;
#if WF_NETLOAD_SERVICE_ROUNDS
    owner->ready_head = NULL;
    owner->ready_tail = NULL;
    owner->ready_count = 0;
    owner->service_yields = 0;
#endif
    for (uint64_t at = 0; at < owner->count; at++) {
        struct connection *link = &owner->first[at];
        link->round = 0;
        link->finished = 0;
        link->completed_by_deadline = 0;
        link->waiting_arrival = 0;
#if WF_NETLOAD_SERVICE_ROUNDS
        link->ready_next = NULL;
        link->queued = 0;
#endif
        if (paced_link(owner, link)) {
            if (!link->planned) {
                link->finished = 1;
                owner->outstanding--;
            } else {
                schedule_arrival(link);
            }
            continue;
        }
        begin_round(link, owner->admitting);
        pump(owner, link);
    }
    while (owner->outstanding > 0) {
        struct timespec delay;
        struct timespec *timeout = NULL;
        if (option_light_per_second && !owner->admitting) {
            struct timespec now;
            clock_gettime(CLOCK_MONOTONIC, &now);
            for (uint64_t at = 0; at < owner->count; at++) {
                struct connection *link = &owner->first[at];
                if (!link->waiting_arrival || link->finished) continue;
                if (seconds_between(link->due, now) >= 0.0) {
                    begin_round(link, 0);
                    pump(owner, link);
                    clock_gettime(CLOCK_MONOTONIC, &now);
                }
            }
            if (!owner->outstanding) break;
            for (uint64_t at = 0; at < owner->count; at++) {
                struct connection *link = &owner->first[at];
                if (!link->waiting_arrival || link->finished) continue;
                struct timespec remaining = {link->due.tv_sec - now.tv_sec,
                                             link->due.tv_nsec - now.tv_nsec};
                if (remaining.tv_nsec < 0) {
                    remaining.tv_sec--;
                    remaining.tv_nsec += 1000000000L;
                }
                if (remaining.tv_sec < 0) remaining = (struct timespec){0, 0};
                if (!timeout || seconds_between(remaining, delay) > 0.0) {
                    delay = remaining;
                    timeout = &delay;
                }
            }
        }
#if WF_NETLOAD_SERVICE_ROUNDS
        if (owner->ready_head) {
            delay = (struct timespec){0, 0};
            timeout = &delay;
        }
#endif
        int ready = option_light_per_second && !owner->admitting
            ? epoll_pwait2(owner->epoll, events, 256, timeout, NULL)
            : epoll_wait(owner->epoll, events, 256,
#if WF_NETLOAD_SERVICE_ROUNDS
                         owner->ready_head ? 0 : -1
#else
                         -1
#endif
                         );
        if (ready < 0) {
            if (errno == EINTR) {
                continue;
            }
            fail("epoll_wait on exchange: %s", strerror(errno));
        }
        for (int at = 0; at < ready; at++) {
            struct connection *link = events[at].data.ptr;
            if (link->finished || link->waiting_arrival) {
                continue;
            }
#if WF_NETLOAD_SERVICE_ROUNDS
            if (link->queued) continue;
#endif
            pump(owner, link);
        }
#if WF_NETLOAD_SERVICE_ROUNDS
        /* A finite group prevents a perpetually ready heavy peer from hiding
         * due light arrivals. Poll readiness again between groups of eight. */
        uint64_t turns = owner->ready_count < 8u ? owner->ready_count : 8u;
        for (uint64_t turn = 0; turn < turns; turn++) {
            struct connection *link = owner->ready_head;
            owner->ready_head = link->ready_next;
            if (!owner->ready_head) owner->ready_tail = NULL;
            owner->ready_count--;
            link->ready_next = NULL;
            link->queued = 0;
            pump(owner, link);
        }
#endif
    }
}

static void *client_main(void *raw) {
    struct client *owner = raw;
    struct epoll_event events[256];

    if (option_duration_ms) {
        /* Fault in every capture page before release. A faster server must
         * not pay extra client first-touch faults during its timed sample. */
        volatile uint32_t *latencies = latency_us;
        volatile uint64_t *values = compute_values;
        for (uint64_t at = 0; at < owner->count; at++) {
            uint64_t start = owner->first[at].index * option_roundtrips;
            for (uint64_t round = 0; round < option_roundtrips; round += 512u) {
                latencies[start + round] = 0;
                values[start + round] = 0;
                if (dispatch_us) ((volatile uint32_t *)dispatch_us)[start + round] = 0;
            }
        }
    }

    /* The checksum oracle runs before connect timing, once per sample. It
     * never competes with the server during the measured exchange. Seeds
     * differ for every request; no shortened recurrence is substituted. */
    if (option_compute && !option_duration_ms) {
        for (uint64_t at = 0; at < owner->count; at++) {
            struct connection *link = &owner->first[at];
            uint64_t rounds = link->index % option_heavy_every == 0 ? option_compute_rounds : 0;
            for (uint64_t round = 0; round < option_roundtrips; round++) {
                compute_values[link->index * option_roundtrips + round] =
                    compute_churn(compute_seed(link->index, round), rounds);
            }
        }
    }

    /* Phase one: every socket is created before any connect call, so the
     * connect phase measures connects and not socket creation. */
    for (uint64_t at = 0; at < owner->count; at++) {
        struct connection *link = &owner->first[at];
        link->descriptor = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0);
        if (link->descriptor < 0) {
            fail("socket: %s", strerror(errno));
        }
        int one = 1;
        setsockopt(link->descriptor, IPPROTO_TCP, TCP_NODELAY, &one, sizeof one);
    }

    pthread_barrier_wait(&gate);

    pthread_barrier_wait(&gate); /* release after the connect start timestamp */

    struct sockaddr_in address;
    memset(&address, 0, sizeof address);
    address.sin_family = AF_INET;
    address.sin_port = htons(option_port);
    address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    uint64_t pending = 0;
    for (uint64_t at = 0; at < owner->count; at++) {
        struct connection *link = &owner->first[at];
        int outcome = connect(link->descriptor, (struct sockaddr *)&address, sizeof address);
        if (outcome == 0) {
            link->established = 1;
            continue;
        }
        if (errno != EINPROGRESS) {
            fail("connection %llu: connect refused: %s", (unsigned long long)link->index,
                 strerror(errno));
        }
        struct epoll_event registration;
        registration.events = EPOLLOUT;
        registration.data.ptr = link;
        if (epoll_ctl(owner->epoll, EPOLL_CTL_ADD, link->descriptor, &registration) != 0) {
            fail("epoll_ctl on connect: %s", strerror(errno));
        }
        pending++;
    }
    while (pending > 0) {
        int ready = epoll_wait(owner->epoll, events, 256, -1);
        if (ready < 0) {
            if (errno == EINTR) {
                continue;
            }
            fail("epoll_wait on connect: %s", strerror(errno));
        }
        for (int at = 0; at < ready; at++) {
            struct connection *link = events[at].data.ptr;
            int problem = 0;
            socklen_t size = sizeof problem;
            if (getsockopt(link->descriptor, SOL_SOCKET, SO_ERROR, &problem, &size) != 0) {
                fail("connection %llu: getsockopt: %s", (unsigned long long)link->index,
                     strerror(errno));
            }
            if (problem != 0) {
                fail("connection %llu: connect refused: %s", (unsigned long long)link->index,
                     strerror(problem));
            }
            link->established = 1;
            epoll_ctl(owner->epoll, EPOLL_CTL_DEL, link->descriptor, NULL);
            pending--;
        }
    }

    pthread_barrier_wait(&gate);

    if (!option_admit) pthread_barrier_wait(&gate); /* timed release without handshake */

    /* Phase two: every connection is armed edge triggered for both directions
     * and starts its first round, so all of them are in flight at once. */
    for (uint64_t at = 0; at < owner->count; at++) {
        struct connection *link = &owner->first[at];
        struct epoll_event registration;
        registration.events = EPOLLIN | EPOLLOUT | EPOLLET;
        registration.data.ptr = link;
        if (epoll_ctl(owner->epoll, EPOLL_CTL_ADD, link->descriptor, &registration) != 0) {
            fail("epoll_ctl on exchange: %s", strerror(errno));
        }
    }
    if (option_admit) {
        owner->admitting = 1;
        exchange(owner);
        pthread_barrier_wait(&gate); /* every peer has answered the handshake */
        pthread_barrier_wait(&gate); /* release after the exchange snapshot */
        owner->admitting = 0;
    }
    exchange(owner);

    pthread_barrier_wait(&gate);

    pthread_barrier_wait(&gate); /* cleanup follows the final CPU snapshot */
    for (uint64_t at = 0; at < owner->count; at++) {
        close(owner->first[at].descriptor);
    }
    if (option_duration_ms) {
        /* Every measured byte was retained as a canonical 64-bit response.
         * Verify every request before main can publish a successful sample. */
        for (uint64_t at = 0; at < owner->count; at++) {
            struct connection *link = &owner->first[at];
            uint64_t rounds = link->index % option_heavy_every == 0 ? option_compute_rounds : 0;
            for (uint64_t round = 0; round < link->round; round++) {
                uint64_t expected = compute_churn(compute_seed(link->index, round), rounds);
                if (compute_values[link->index * option_roundtrips + round] != expected)
                    fail("connection %llu round %llu: captured compute result mismatch",
                         (unsigned long long)link->index, (unsigned long long)round);
            }
        }
    }
    return NULL;
}

int main(int argc, char **argv) {
    if (argc < 5) {
        fprintf(stderr, "usage: netload PORT CONNECTIONS ROUNDTRIPS BYTES [--threads T] [--compute ROUNDS] [--heavy-every N] [--admit] [--duration-ms MS] [--light-per-second RATE]\n");
        return 2;
    }
    unsigned long port = strtoul(argv[1], NULL, 10);
    option_connections = strtoull(argv[2], NULL, 10);
    option_roundtrips = strtoull(argv[3], NULL, 10);
    option_bytes = strtoull(argv[4], NULL, 10);
    long online = sysconf(_SC_NPROCESSORS_ONLN);
    uint64_t threads = online > 0 ? (uint64_t)online : 1;
    for (int at = 5; at < argc; at++) {
        if (strcmp(argv[at], "--threads") == 0 && at + 1 < argc) {
            threads = strtoull(argv[++at], NULL, 10);
            continue;
        }
        if (strcmp(argv[at], "--compute") == 0 && at + 1 < argc) {
            option_compute = 1;
            option_compute_rounds = strtoull(argv[++at], NULL, 10);
            continue;
        }
        if (strcmp(argv[at], "--heavy-every") == 0 && at + 1 < argc) {
            option_heavy_every = strtoull(argv[++at], NULL, 10);
            continue;
        }
        if (strcmp(argv[at], "--admit") == 0) {
            option_admit = 1;
            continue;
        }
        if (strcmp(argv[at], "--duration-ms") == 0 && at + 1 < argc) {
            option_duration_ms = strtoull(argv[++at], NULL, 10);
            if (!option_duration_ms || option_duration_ms > 60000) {
                fprintf(stderr, "netload: duration must be 1..60000 milliseconds\n");
                return 2;
            }
            continue;
        }
        if (strcmp(argv[at], "--light-per-second") == 0 && at + 1 < argc) {
            option_light_per_second = strtoull(argv[++at], NULL, 10);
            if (!option_light_per_second || option_light_per_second > 1000000) {
                fprintf(stderr, "netload: light rate must be 1..1000000 requests/s per peer\n");
                return 2;
            }
            continue;
        }
        fprintf(stderr, "netload: unknown argument %s\n", argv[at]);
        return 2;
    }
    if (port == 0 || port > 65535 || option_connections == 0 || option_roundtrips == 0 ||
        option_bytes == 0 || threads == 0) {
        fprintf(stderr, "netload: PORT, CONNECTIONS, ROUNDTRIPS, BYTES and T must be positive\n");
        return 2;
    }
    option_port = (uint16_t)port;
    if (option_compute && (option_bytes != COMPUTE_BYTES ||
        option_compute_rounds > COMPUTE_MAX_ROUNDS || option_heavy_every == 0)) {
        fprintf(stderr, "netload: compute mode requires 64 bytes, at most 16777216 rounds and a positive heavy interval\n");
        return 2;
    }
    if (option_duration_ms && (!option_compute || !option_admit)) {
        fprintf(stderr, "netload: duration mode requires compute and admission\n");
        return 2;
    }
    if (option_light_per_second && (!option_duration_ms || option_heavy_every < 2 ||
        option_connections < 2 || option_roundtrips <
            (option_light_per_second * option_duration_ms + 999u) / 1000u)) {
        fprintf(stderr, "netload: paced light requests require duration, both classes and capacity for every planned arrival\n");
        return 2;
    }
    if (threads > option_connections) {
        threads = option_connections;
    }
    raise_descriptor_limit(option_connections);

    connections = calloc((size_t)option_connections, sizeof *connections);
    clients = calloc((size_t)threads, sizeof *clients);
    latency_us = calloc((size_t)(option_connections * option_roundtrips), sizeof *latency_us);
    unsigned char *payloads = malloc((size_t)option_connections * (size_t)option_bytes * 2);
    if (option_compute) compute_values = calloc((size_t)(option_connections * option_roundtrips), sizeof *compute_values);
    if (option_light_per_second) dispatch_us = calloc((size_t)(option_connections * option_roundtrips), sizeof *dispatch_us);
    if (connections == NULL || clients == NULL || latency_us == NULL || payloads == NULL ||
        (option_compute && compute_values == NULL) || (option_light_per_second && dispatch_us == NULL)) {
        fprintf(stderr, "netload: out of memory\n");
        return 1;
    }
    for (uint64_t at = 0; at < option_connections; at++) {
        struct connection *link = &connections[at];
        link->index = at;
        link->descriptor = -1;
        link->outgoing = payloads + (size_t)(at * 2) * (size_t)option_bytes;
        link->incoming = link->outgoing + option_bytes;
        fill_message(link);
        if (option_light_per_second && at % option_heavy_every != 0) {
            uint64_t light_count = option_connections -
                (option_connections + option_heavy_every - 1u) / option_heavy_every;
            uint64_t ordinal = at - at / option_heavy_every - 1u;
            link->light_phase_ns = ordinal * UINT64_C(1000000000) /
                (option_light_per_second * light_count);
            uint64_t duration_ns = option_duration_ms * UINT64_C(1000000);
            link->planned = duration_ns > link->light_phase_ns
                ? ((duration_ns - link->light_phase_ns) * option_light_per_second +
                   UINT64_C(999999999)) / UINT64_C(1000000000) : 0;
        }
    }

    /* The connections are spread over the client threads in equal blocks; the
     * first `remainder` threads carry one more. */
    uint64_t block = option_connections / threads;
    uint64_t remainder = option_connections % threads;
    uint64_t handed = 0;
    for (uint64_t at = 0; at < threads; at++) {
        struct client *owner = &clients[at];
        owner->count = block + (at < remainder ? 1 : 0);
        owner->first = &connections[handed];
        owner->outstanding = owner->count;
        handed += owner->count;
        owner->epoll = epoll_create1(0);
        if (owner->epoll < 0) {
            fprintf(stderr, "netload: epoll_create1: %s\n", strerror(errno));
            return 1;
        }
    }

    pthread_barrier_init(&gate, NULL, (unsigned)threads + 1);
    for (uint64_t at = 0; at < threads; at++) {
        if (pthread_create(&clients[at].thread, NULL, client_main, &clients[at]) != 0) {
            fprintf(stderr, "netload: pthread_create failed\n");
            return 1;
        }
    }

    struct timespec connect_start;
    struct timespec connect_end;
    struct timespec exchange_start;
    struct timespec exchange_end;
    struct rusage exchange_cpu_start;
    struct rusage exchange_cpu_end;
    pthread_barrier_wait(&gate);
    clock_gettime(CLOCK_MONOTONIC, &connect_start);
    pthread_barrier_wait(&gate);
    pthread_barrier_wait(&gate);
    clock_gettime(CLOCK_MONOTONIC, &connect_end);
    if (option_admit) pthread_barrier_wait(&gate);
    getrusage(RUSAGE_SELF, &exchange_cpu_start);
    clock_gettime(CLOCK_MONOTONIC, &exchange_start);
    exchange_origin = exchange_start;
    exchange_deadline = exchange_start;
    exchange_deadline.tv_sec += (time_t)(option_duration_ms / 1000u);
    exchange_deadline.tv_nsec += (long)(option_duration_ms % 1000u) * 1000000L;
    if (exchange_deadline.tv_nsec >= 1000000000L) {
        exchange_deadline.tv_sec++;
        exchange_deadline.tv_nsec -= 1000000000L;
    }
    pthread_barrier_wait(&gate);
    pthread_barrier_wait(&gate);
    clock_gettime(CLOCK_MONOTONIC, &exchange_end);
    getrusage(RUSAGE_SELF, &exchange_cpu_end);
    pthread_barrier_wait(&gate);
    for (uint64_t at = 0; at < threads; at++) {
        pthread_join(clients[at].thread, NULL);
    }
    pthread_barrier_destroy(&gate);

    uint64_t total = 0;
    for (uint64_t at = 0; at < option_connections; at++) total += connections[at].round;
    if (option_light_per_second) {
        uint32_t *light_dispatch = malloc((size_t)total * sizeof *light_dispatch);
        uint32_t *light_service = malloc((size_t)total * sizeof *light_service);
        if (!light_dispatch || !light_service) fail("out of memory for paced latency summaries");
        uint64_t count = 0;
        uint64_t planned = 0;
        for (uint64_t at = 0; at < option_connections; at++) {
            struct connection *link = &connections[at];
            if (at % option_heavy_every == 0) continue;
            if (link->round != link->planned) fail("paced peer did not drain every planned request");
            planned += link->planned;
            for (uint64_t round = 0; round < link->round; round++) {
                uint64_t index = at * option_roundtrips + round;
                uint32_t dispatch = dispatch_us[index];
                uint32_t elapsed = latency_us[index];
                light_dispatch[count] = dispatch;
                light_service[count] = elapsed >= dispatch ? elapsed - dispatch : 0;
                count++;
            }
        }
        qsort(light_dispatch, (size_t)count, sizeof *light_dispatch, compare_u32);
        qsort(light_service, (size_t)count, sizeof *light_service, compare_u32);
        printf("light_per_second=%llu\tlight_planned=%llu\tlight_dispatch_p99_us=%u\tlight_service_p99_us=%u\t",
               (unsigned long long)option_light_per_second, (unsigned long long)planned,
               percentile(light_dispatch, count, 99), percentile(light_service, count, 99));
        free(light_dispatch);
        free(light_service);
    }
    if (option_compute) {
        /* Preserve the per-class tails before sorting the aggregate. A class
         * owns disjoint connection blocks in latency_us; the temporary array
         * is allocated only after all client threads have joined. */
        uint32_t *class_latency = malloc((size_t)total * sizeof *class_latency);
        if (class_latency == NULL) fail("out of memory for class latencies");
        for (unsigned heavy = 0; heavy < 2; heavy++) {
            uint64_t count = 0;
            uint64_t minimum_count = UINT64_MAX;
            uint64_t completed_by_deadline = 0;
            uint32_t worst_peer_p99 = 0;
            struct timespec first = exchange_end;
            struct timespec last = connect_end;
            for (uint64_t connection = 0; connection < option_connections; connection++) {
                if ((connection % option_heavy_every == 0) != (heavy != 0)) continue;
                if (connections[connection].round && seconds_between(connections[connection].first_started, first) > 0)
                    first = connections[connection].first_started;
                if (connections[connection].round && seconds_between(last, connections[connection].last_finished) > 0)
                    last = connections[connection].last_finished;
                uint64_t peer_count = connections[connection].round;
                completed_by_deadline += connections[connection].completed_by_deadline;
                uint32_t *peer_latency = latency_us + connection * option_roundtrips;
                qsort(peer_latency, (size_t)peer_count, sizeof *peer_latency, compare_u32);
                uint32_t peer_p99 = percentile(peer_latency, peer_count, 99);
                if (peer_count < minimum_count) minimum_count = peer_count;
                if (peer_p99 > worst_peer_p99) worst_peer_p99 = peer_p99;
                memcpy(class_latency + count, latency_us + connection * option_roundtrips,
                       (size_t)connections[connection].round * sizeof *class_latency);
                count += connections[connection].round;
            }
            qsort(class_latency, (size_t)count, sizeof *class_latency, compare_u32);
            printf("%s_count=%llu\t%s_p50_us=%u\t%s_p99_us=%u\t%s_max_us=%u\t%s_span_us=%llu\t",
                   heavy ? "heavy" : "light", (unsigned long long)count,
                   heavy ? "heavy" : "light", percentile(class_latency, count, 50),
                   heavy ? "heavy" : "light", percentile(class_latency, count, 99),
                   heavy ? "heavy" : "light", count ? class_latency[count - 1] : 0,
                   heavy ? "heavy" : "light", (unsigned long long)(count ? microseconds_between(first, last) : 0));
            printf("%s_min_count=%llu\t%s_worst_peer_p99_us=%u\t",
                   heavy ? "heavy" : "light", (unsigned long long)(count ? minimum_count : 0),
                   heavy ? "heavy" : "light", worst_peer_p99);
            printf("%s_completed_by_deadline=%llu\t", heavy ? "heavy" : "light",
                   (unsigned long long)completed_by_deadline);
        }
        free(class_latency);
    }
    uint64_t packed = 0;
    for (uint64_t at = 0; at < option_connections; at++) {
        memmove(latency_us + packed, latency_us + at * option_roundtrips,
                (size_t)connections[at].round * sizeof *latency_us);
        packed += connections[at].round;
    }
    qsort(latency_us, (size_t)total, sizeof *latency_us, compare_u32);
    uint64_t connect_us = microseconds_between(connect_start, connect_end);
    double exchange_seconds = seconds_between(exchange_start, exchange_end);
    uint64_t exchange_us = microseconds_between(exchange_start, exchange_end);
    double per_second = exchange_seconds > 0.0 ? (double)total / exchange_seconds : 0.0;
    double bytes_per_second =
        exchange_seconds > 0.0 ? (double)total * (double)option_bytes / exchange_seconds : 0.0;
#if WF_NETLOAD_SERVICE_ROUNDS
    uint64_t service_yields = 0;
    for (uint64_t at = 0; at < threads; at++) service_yields += clients[at].service_yields;
    printf("client_service_rounds=%u\tclient_service_yields=%llu\t",
           (unsigned)WF_NETLOAD_SERVICE_ROUNDS, (unsigned long long)service_yields);
#endif
    printf("admitted=%d\tadmission_us=%llu\t", option_admit, (unsigned long long)microseconds_between(connect_end, exchange_start));
    printf("duration_ms=%llu\tdrain_us=%llu\t", (unsigned long long)option_duration_ms,
           (unsigned long long)(option_duration_ms ? microseconds_between(exchange_deadline, exchange_end) : 0));
    printf("connect_us=%llu\texchange_us=%llu\troundtrips=%llu\trt_per_s=%.1f"
           "\tbytes_per_s=%.1f\tp50_us=%u\tp90_us=%u\tp99_us=%u\tmax_us=%u"
           "\tclient_exchange_user_us=%lld\tclient_exchange_system_us=%lld\n",
           (unsigned long long)connect_us, (unsigned long long)exchange_us,
           (unsigned long long)total, per_second, bytes_per_second,
           percentile(latency_us, total, 50), percentile(latency_us, total, 90),
           percentile(latency_us, total, 99), total > 0 ? latency_us[total - 1] : 0,
           (long long)(exchange_cpu_end.ru_utime.tv_sec - exchange_cpu_start.ru_utime.tv_sec) * 1000000
               + exchange_cpu_end.ru_utime.tv_usec - exchange_cpu_start.ru_utime.tv_usec,
           (long long)(exchange_cpu_end.ru_stime.tv_sec - exchange_cpu_start.ru_stime.tv_sec) * 1000000
               + exchange_cpu_end.ru_stime.tv_usec - exchange_cpu_start.ru_stime.tv_usec);
    fflush(stdout);
    free(payloads);
    free(compute_values);
    free(dispatch_us);
    free(latency_us);
    free(clients);
    free(connections);
    return 0;
}
