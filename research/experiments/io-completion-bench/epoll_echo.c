/* glibc hides the POSIX and Linux entry points this file calls when the
 * translation unit is compiled as strict C11, so the namespace is asked for
 * before any header. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/* The epoll reference of the TCP echo workload: the same contract as
 * `uring_echo`, in the shape most deployed servers still have.
 *
 *   epoll_echo PORT CONNECTIONS [--threads N]
 *
 * It listens on 127.0.0.1:PORT, echoes every byte of every connection back to
 * it, and exits 0 once CONNECTIONS connections have been accepted in total and
 * every one of them has closed. `--threads` defaults to the number of online
 * CPUs.
 *
 * One epoll instance and one SO_REUSEPORT listening socket per thread, so a
 * connection is accepted, read and echoed on one thread with no shared state
 * on the path and no thundering herd on a shared listener. Every descriptor is
 * non-blocking and registered edge triggered, so a wakeup means "there is
 * something new" and the loop drains the socket until EAGAIN rather than
 * returning to epoll_wait after every buffer.
 *
 * The cost this pays and `uring_echo` does not is one syscall per readiness
 * edge and one per transfer: epoll_wait, then recv, then send, per connection
 * per exchange, against a completion the ring hands over with no submission at
 * all. It has no multishot accept, no multishot receive, and no provided
 * buffer ring; a read needs a buffer chosen by this process before the kernel
 * has said there is anything to put in it.
 *
 * Everything is sized from CONNECTIONS before the first accept: the connection
 * table is indexed by descriptor, each connection owns one pending buffer for
 * the remainder of a short write, and each thread owns one receive buffer.
 * Nothing is allocated per operation. */
#include <errno.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <unistd.h>

#if defined(WF_BENCH_COMPUTE)
#include "compute_protocol.h"
#endif

/* One receive per thread and one pending buffer per connection, both this
 * size. The loop reads only when the pending buffer is empty, so what a short
 * write leaves behind is at most one receive and the pending buffer can never
 * overflow. */
#if defined(WF_BENCH_COMPUTE)
#define TRANSFER_BYTES COMPUTE_BYTES
#else
#define TRANSFER_BYTES 65536u
#endif

struct connection {
    int active;
    unsigned char *pending;
    uint32_t offset;
    uint32_t length;
#if defined(WF_BENCH_COMPUTE)
    unsigned received;
#endif
};

struct worker {
    pthread_t thread;
    int index;
    int epoll;
    int listener;
    int wake;
    unsigned char *scratch;
};

static uint64_t option_connections;
static uint16_t option_port;
static unsigned option_threads;
static unsigned descriptor_capacity;
static struct connection *table;
static struct worker *workers;
static unsigned char *pending_memory;
static _Atomic uint64_t accepted_total;
static _Atomic uint64_t closed_total;
static _Atomic int finished;
static _Atomic int failed;

static void report(const char *what, int error) {
    fprintf(stderr, "epoll_echo: %s: %s\n", what, strerror(error));
    fflush(stderr);
}

static void wake_everyone(void) {
    if (workers == NULL) {
        return;
    }
    uint64_t one = 1;
    for (unsigned at = 0; at < option_threads; at++) {
        if (workers[at].wake >= 0) {
            ssize_t written = write(workers[at].wake, &one, sizeof one);
            (void)written;
        }
    }
}

/* A failure ends the run for every thread, not only the one that saw it: the
 * others are blocked in epoll_wait with nothing left to report. */
static void mark_failed(void) {
    atomic_store_explicit(&failed, 1, memory_order_relaxed);
    wake_everyone();
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

/* The exit condition, checked by whichever thread completes the last close:
 * CONNECTIONS accepted and all of them closed. The thread that sees it wakes
 * every other loop, since the others are blocked in epoll_wait with nothing
 * left to report. */
static void check_finished(void) {
    if (atomic_load_explicit(&accepted_total, memory_order_relaxed) < option_connections) {
        return;
    }
    if (atomic_load_explicit(&closed_total, memory_order_relaxed) < option_connections) {
        return;
    }
    if (atomic_exchange_explicit(&finished, 1, memory_order_relaxed)) {
        return;
    }
    wake_everyone();
}

static void close_connection(struct worker *worker, int descriptor) {
    epoll_ctl(worker->epoll, EPOLL_CTL_DEL, descriptor, NULL);
    table[descriptor].active = 0;
    table[descriptor].offset = 0;
    table[descriptor].length = 0;
    close(descriptor);
    atomic_fetch_add_explicit(&closed_total, 1, memory_order_relaxed);
    check_finished();
}

/* Drives one connection as far as it will go without blocking: flush what a
 * short write left behind, then read until EAGAIN, echoing each arrival as it
 * comes. A read happens only with the pending buffer empty, which is what
 * bounds it, and every byte read is echoed before the next read.
 *
 * Returning with the pending buffer full is this connection's back pressure:
 * the next EPOLLOUT edge resumes it, and the read that follows the flush picks
 * up whatever arrived meanwhile, which an edge-triggered loop would otherwise
 * never be told about. */
static void service(struct worker *worker, int descriptor) {
    struct connection *link = &table[descriptor];
    for (;;) {
        while (link->length > link->offset) {
            ssize_t moved = send(descriptor, link->pending + link->offset,
                                 (size_t)(link->length - link->offset), MSG_NOSIGNAL);
            if (moved > 0) {
                link->offset += (uint32_t)moved;
                continue;
            }
            if (moved < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
                return;
            }
            close_connection(worker, descriptor);
            return;
        }
        link->offset = 0;
        link->length = 0;
#if defined(WF_BENCH_COMPUTE)
        while (link->received < COMPUTE_BYTES) {
            ssize_t taken = recv(descriptor, link->pending + link->received,
                                 COMPUTE_BYTES - link->received, 0);
            if (taken > 0) {
                link->received += (unsigned)taken;
                continue;
            }
            if (taken < 0 && errno == EINTR) continue;
            if (taken < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) return;
            if (taken == 0 && link->received != 0) mark_failed();
            close_connection(worker, descriptor);
            return;
        }
        if (!compute_response(link->pending)) {
            mark_failed();
            close_connection(worker, descriptor);
            return;
        }
        link->received = 0;
        link->length = COMPUTE_BYTES;
#else
        ssize_t taken = recv(descriptor, worker->scratch, TRANSFER_BYTES, 0);
        if (taken == 0) {
            close_connection(worker, descriptor);
            return;
        }
        if (taken < 0) {
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
                return;
            }
            if (errno == EINTR) {
                continue;
            }
            close_connection(worker, descriptor);
            return;
        }
        size_t handed = 0;
        while (handed < (size_t)taken) {
            ssize_t moved = send(descriptor, worker->scratch + handed, (size_t)taken - handed,
                                 MSG_NOSIGNAL);
            if (moved > 0) {
                handed += (size_t)moved;
                continue;
            }
            if (moved < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
                break;
            }
            close_connection(worker, descriptor);
            return;
        }
        if (handed < (size_t)taken) {
            memcpy(link->pending, worker->scratch + handed, (size_t)taken - handed);
            link->offset = 0;
            link->length = (uint32_t)((size_t)taken - handed);
            return;
        }
#endif
    }
}

static void accept_ready(struct worker *worker) {
    for (;;) {
        int descriptor = accept4(worker->listener, NULL, NULL, SOCK_NONBLOCK);
        if (descriptor < 0) {
            if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
                return;
            }
            if (errno == ECONNABORTED) {
                continue;
            }
            report("accept4", errno);
            mark_failed();
            return;
        }
        if ((unsigned)descriptor >= descriptor_capacity) {
            fprintf(stderr, "epoll_echo: descriptor %d is past the table\n", descriptor);
            close(descriptor);
            mark_failed();
            return;
        }
        struct connection *link = &table[descriptor];
        link->active = 1;
        link->offset = 0;
        link->length = 0;
#if defined(WF_BENCH_COMPUTE)
        link->received = 0;
#endif
        struct epoll_event registration;
        registration.events = EPOLLIN | EPOLLOUT | EPOLLET;
        registration.data.fd = descriptor;
        if (epoll_ctl(worker->epoll, EPOLL_CTL_ADD, descriptor, &registration) != 0) {
            report("epoll_ctl of a connection", errno);
            close(descriptor);
            mark_failed();
            return;
        }
        atomic_fetch_add_explicit(&accepted_total, 1, memory_order_relaxed);
        /* The connection may already hold bytes: an edge for them was consumed
         * by the accept, not by a later epoll_wait. */
        service(worker, descriptor);
    }
}

static void *worker_main(void *raw) {
    struct worker *worker = raw;
    struct epoll_event events[256];
    while (!atomic_load_explicit(&finished, memory_order_relaxed) &&
           !atomic_load_explicit(&failed, memory_order_relaxed)) {
        int ready = epoll_wait(worker->epoll, events, 256, -1);
        if (ready < 0) {
            if (errno == EINTR) {
                continue;
            }
            report("epoll_wait", errno);
            mark_failed();
            break;
        }
        for (int at = 0; at < ready; at++) {
            int descriptor = events[at].data.fd;
            if (descriptor == worker->listener) {
                accept_ready(worker);
                continue;
            }
            if (descriptor == worker->wake) {
                uint64_t ticket = 0;
                ssize_t taken = read(worker->wake, &ticket, sizeof ticket);
                (void)taken;
                continue;
            }
            if (!table[descriptor].active) {
                continue;
            }
            service(worker, descriptor);
        }
    }
    return NULL;
}

static int listener_for(uint16_t port) {
    int descriptor = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0);
    if (descriptor < 0) {
        report("socket", errno);
        return -1;
    }
    int one = 1;
    if (setsockopt(descriptor, SOL_SOCKET, SO_REUSEADDR, &one, sizeof one) != 0 ||
        setsockopt(descriptor, SOL_SOCKET, SO_REUSEPORT, &one, sizeof one) != 0) {
        report("setsockopt", errno);
        close(descriptor);
        return -1;
    }
    /* Set on the listener, where an accepted connection inherits it, rather
     * than once per accepted descriptor: the connect rate is one of the
     * measures, and a per-connection setsockopt would be charged to it. */
    setsockopt(descriptor, IPPROTO_TCP, TCP_NODELAY, &one, sizeof one);
    struct sockaddr_in address;
    memset(&address, 0, sizeof address);
    address.sin_family = AF_INET;
    address.sin_port = htons(port);
    address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    if (bind(descriptor, (struct sockaddr *)&address, sizeof address) != 0) {
        report("bind", errno);
        close(descriptor);
        return -1;
    }
    if (listen(descriptor, 4096) != 0) {
        report("listen", errno);
        close(descriptor);
        return -1;
    }
    return descriptor;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: epoll_echo PORT CONNECTIONS [--threads N]\n");
        return 2;
    }
    unsigned long port = strtoul(argv[1], NULL, 10);
    option_connections = strtoull(argv[2], NULL, 10);
    long online = sysconf(_SC_NPROCESSORS_ONLN);
    option_threads = online > 0 ? (unsigned)online : 1u;
    for (int at = 3; at < argc; at++) {
        if (strcmp(argv[at], "--threads") == 0 && at + 1 < argc) {
            option_threads = (unsigned)strtoul(argv[++at], NULL, 10);
            continue;
        }
        fprintf(stderr, "epoll_echo: unknown argument %s\n", argv[at]);
        return 2;
    }
    if (port == 0 || port > 65535 || option_connections == 0 || option_threads == 0) {
        fprintf(stderr, "epoll_echo: PORT, CONNECTIONS and N must be positive\n");
        return 2;
    }
    option_port = (uint16_t)port;
    raise_descriptor_limit(option_connections + 8 * option_threads);

    descriptor_capacity = (unsigned)(option_connections + 8 * option_threads + 64);
    table = calloc(descriptor_capacity, sizeof *table);
    workers = calloc(option_threads, sizeof *workers);
    pending_memory = malloc((size_t)descriptor_capacity * TRANSFER_BYTES);
    if (table == NULL || workers == NULL || pending_memory == NULL) {
        fprintf(stderr, "epoll_echo: out of memory\n");
        return 1;
    }
    for (unsigned at = 0; at < descriptor_capacity; at++) {
        table[at].pending = pending_memory + (size_t)at * TRANSFER_BYTES;
    }

    /* Every listener is bound before any thread runs, so a client that finds
     * the port listening finds all of them listening. */
    for (unsigned at = 0; at < option_threads; at++) {
        struct worker *worker = &workers[at];
        worker->index = (int)at;
        worker->scratch = malloc(TRANSFER_BYTES);
        worker->epoll = epoll_create1(0);
        worker->listener = listener_for(option_port);
        worker->wake = eventfd(0, EFD_NONBLOCK);
        if (worker->scratch == NULL || worker->epoll < 0 || worker->listener < 0 ||
            worker->wake < 0) {
            report("worker setup", errno);
            return 1;
        }
        struct epoll_event registration;
        registration.events = EPOLLIN | EPOLLET;
        registration.data.fd = worker->listener;
        if (epoll_ctl(worker->epoll, EPOLL_CTL_ADD, worker->listener, &registration) != 0) {
            report("epoll_ctl of the listener", errno);
            return 1;
        }
        registration.events = EPOLLIN;
        registration.data.fd = worker->wake;
        if (epoll_ctl(worker->epoll, EPOLL_CTL_ADD, worker->wake, &registration) != 0) {
            report("epoll_ctl of the wake channel", errno);
            return 1;
        }
    }

    for (unsigned at = 0; at < option_threads; at++) {
        if (pthread_create(&workers[at].thread, NULL, worker_main, &workers[at]) != 0) {
            fprintf(stderr, "epoll_echo: pthread_create failed\n");
            return 1;
        }
    }
    for (unsigned at = 0; at < option_threads; at++) {
        pthread_join(workers[at].thread, NULL);
    }
    for (unsigned at = 0; at < option_threads; at++) {
        free(workers[at].scratch);
        close(workers[at].wake);
        close(workers[at].listener);
        close(workers[at].epoll);
    }
    int broken = atomic_load_explicit(&failed, memory_order_relaxed);
    uint64_t accepted = atomic_load_explicit(&accepted_total, memory_order_relaxed);
    uint64_t closed = atomic_load_explicit(&closed_total, memory_order_relaxed);
    free(pending_memory);
    free(workers);
    free(table);
    if (broken || accepted < option_connections || closed < option_connections) {
        fprintf(stderr, "epoll_echo: accepted %llu and closed %llu of %llu connections\n",
                (unsigned long long)accepted, (unsigned long long)closed,
                (unsigned long long)option_connections);
        return 1;
    }
    return 0;
}
