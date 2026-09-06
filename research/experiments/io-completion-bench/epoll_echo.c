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
 * The default sizes everything from CONNECTIONS before the first accept: the
 * connection table is indexed by descriptor, each connection owns one pending
 * buffer for the remainder of a short write, and each thread owns one receive
 * buffer. The storage experiment can instead receive into private arena slices
 * or allocate a buffer on accept and free it on close. No storage policy
 * allocates buffers per receive or send. The coroutine comparison separately
 * measures heap-allocated versus parent-contained nested call frames. */
#include <errno.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <pthread.h>
#include <stdint.h>
#if defined(__cplusplus)
#include <atomic>
using std::atomic_load_explicit;
using std::atomic_store_explicit;
using std::atomic_fetch_add_explicit;
using std::atomic_exchange_explicit;
using std::memory_order_relaxed;
typedef std::atomic<uint64_t> wf_atomic_u64;
typedef std::atomic<int> wf_atomic_int;
#else
#include <stdatomic.h>
typedef _Atomic uint64_t wf_atomic_u64;
typedef _Atomic int wf_atomic_int;
#endif
#if defined(WF_BENCH_COROUTINE)
#if !defined(__cplusplus) || !defined(WF_BENCH_STACKFUL)
#error "The coroutine comparison requires C++20 and the sequential handler engine"
#endif
#include <coroutine>
#include <utility>
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <unistd.h>

#if defined(WF_BENCH_STACKFUL)
#include <sys/mman.h>
#if !defined(WF_BENCH_COROUTINE)
#include "../../../compiler/src/backend/sched/switch.h"
#endif
#endif

#if defined(WF_BENCH_COMPUTE)
#include "compute_protocol.h"
#endif
#if defined(WF_BENCH_QUANTUM) && !defined(WF_BENCH_COMPUTE)
#error "The compute quantum reference requires the compute protocol"
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

/* Receive storage: shared worker scratch with private short-send spill (0),
 * private slices in the existing arena (1), per-connection malloc (2), or
 * per-connection calloc (3). Only initialized received prefixes are echoed.
 * The latter pair compares the allocator's zeroed-storage path with the same
 * ownership and call lifetime. Owned by the io-model storage experiment. */
#ifndef WF_BENCH_RECEIVE_STORAGE
#define WF_BENCH_RECEIVE_STORAGE 0
#endif
#if WF_BENCH_RECEIVE_STORAGE < 0 || WF_BENCH_RECEIVE_STORAGE > 3
#error "WF_BENCH_RECEIVE_STORAGE must be in 0..3"
#endif
#if WF_BENCH_RECEIVE_STORAGE && defined(WF_BENCH_COMPUTE)
#error "The receive-storage comparison applies to the echo byte stream"
#endif
#if WF_BENCH_RECEIVE_STORAGE
#define WF_BENCH_RECEIVE_BUFFER(worker, link) ((link)->pending)
#else
#define WF_BENCH_RECEIVE_BUFFER(worker, link) ((worker)->scratch)
#endif

struct connection {
    int active;
    unsigned char *pending;
#if defined(WF_BENCH_STACKFUL)
    struct worker *owner;
    void *saved_sp;
#if defined(WF_BENCH_COROUTINE)
    void *resume;
#endif
    int descriptor;
    int done;
#else
    uint32_t offset;
    uint32_t length;
#if defined(WF_BENCH_COMPUTE)
    unsigned received;
#endif
#if defined(WF_BENCH_QUANTUM)
    int queued;
    int computing;
    uint64_t value;
    uint64_t remaining;
#endif
#endif
#if defined(WF_BENCH_STACKFUL) && defined(WF_BENCH_QUANTUM)
    int queued;
#endif
};

struct worker {
    pthread_t thread;
    int index;
    int epoll;
    int listener;
    int wake;
    unsigned char *scratch;
#if defined(WF_BENCH_STACKFUL)
    void *saved_sp;
#if defined(WF_BENCH_QUANTUM)
    unsigned replies;
#endif
#if defined(WF_BENCH_OBSERVE)
    uint64_t resumes;
    uint64_t waits;
    uint64_t send_waits;
    uint64_t yields;
#endif
#endif
#if defined(WF_BENCH_QUANTUM)
    int *queue;
    unsigned head;
    unsigned tail;
    unsigned count;
#endif
};

static uint64_t option_connections;
static uint16_t option_port;
static unsigned option_threads;
static unsigned descriptor_capacity;
static struct connection *table;
static struct worker *workers;
static unsigned char *pending_memory;
#if defined(WF_BENCH_STACKFUL) && !defined(WF_BENCH_COROUTINE)
static unsigned char *stack_memory;
static size_t stack_stride;
#endif
static wf_atomic_u64 accepted_total;
static wf_atomic_u64 closed_total;
static wf_atomic_int finished;
static wf_atomic_int failed;
#if defined(WF_BENCH_QUANTUM)
static uint64_t option_quantum = 16384;
#endif

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
#if WF_BENCH_RECEIVE_STORAGE >= 2
    free(table[descriptor].pending);
    table[descriptor].pending = NULL;
#endif
#if !defined(WF_BENCH_STACKFUL)
    table[descriptor].offset = 0;
    table[descriptor].length = 0;
#endif
    close(descriptor);
    atomic_fetch_add_explicit(&closed_total, 1, memory_order_relaxed);
    check_finished();
}

#if defined(WF_BENCH_QUANTUM)
/* One owner and at most one queue entry per connection. No per-step allocation
 * or shared scheduler word: this is the explicit continuation reference for
 * the mixed-load experiment, not a replacement for the language scheduler. */
static void enqueue(struct worker *worker, int descriptor) {
    struct connection *link = &table[descriptor];
    if (link->queued) return;
    if (worker->count == descriptor_capacity) {
        fprintf(stderr, "epoll_compute: duplicate or overflowing ready queue\n");
        mark_failed();
        return;
    }
    link->queued = 1;
    worker->queue[worker->tail] = descriptor;
    worker->tail = worker->tail + 1u == descriptor_capacity ? 0u : worker->tail + 1u;
    worker->count++;
}
#endif

#if defined(WF_BENCH_COROUTINE)
#include "epoll_coroutine.h"
#elif defined(WF_BENCH_STACKFUL)
#include "epoll_stackful.h"
#else
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
#if defined(WF_BENCH_QUANTUM)
    unsigned replies = 0;
#endif
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
#if defined(WF_BENCH_QUANTUM)
        if (link->length != 0) replies++;
#endif
        link->offset = 0;
        link->length = 0;
#if defined(WF_BENCH_QUANTUM)
        if (replies == 8u) {
            enqueue(worker, descriptor);
            return;
        }
#endif
#if defined(WF_BENCH_COMPUTE)
#if defined(WF_BENCH_QUANTUM)
        if (link->computing) {
            uint64_t steps = link->remaining < option_quantum ? link->remaining : option_quantum;
            link->value = compute_churn(link->value, steps);
            link->remaining -= steps;
            if (link->remaining != 0) {
                enqueue(worker, descriptor);
                return;
            }
            link->computing = 0;
            compute_encode(link->pending, link->value);
            link->length = COMPUTE_BYTES;
            continue;
        }
#endif
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
#if defined(WF_BENCH_QUANTUM)
        link->value = compute_decode(link->pending);
        link->remaining = compute_decode(link->pending + 8);
        if (link->remaining > COMPUTE_MAX_ROUNDS) {
            mark_failed();
            close_connection(worker, descriptor);
            return;
        }
        link->received = 0;
        if (link->remaining != 0) {
            link->computing = 1;
            enqueue(worker, descriptor);
            return;
        }
        compute_encode(link->pending, link->value);
#else
        if (!compute_response(link->pending)) {
            mark_failed();
            close_connection(worker, descriptor);
            return;
        }
        link->received = 0;
#endif
        link->length = COMPUTE_BYTES;
#else
        ssize_t taken = recv(descriptor, WF_BENCH_RECEIVE_BUFFER(worker, link), TRANSFER_BYTES, 0);
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
            ssize_t moved = send(descriptor, WF_BENCH_RECEIVE_BUFFER(worker, link) + handed, (size_t)taken - handed,
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
#if WF_BENCH_RECEIVE_STORAGE
            /* This connection already owns the unsent suffix. */
            link->offset = (uint32_t)handed;
            link->length = (uint32_t)taken;
#else
            memcpy(link->pending, WF_BENCH_RECEIVE_BUFFER(worker, link) + handed, (size_t)taken - handed);
            link->offset = 0;
            link->length = (uint32_t)((size_t)taken - handed);
#endif
            return;
        }
#endif
    }
}
#endif

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
#if defined(WF_BENCH_TCP_VERIFY)
        int enabled = 0;
        socklen_t option_bytes = sizeof(enabled);
#if defined(WF_BENCH_NAGLE)
        const int expected = 0;
#else
        const int expected = 1;
#endif
        if (getsockopt(descriptor, IPPROTO_TCP, TCP_NODELAY, &enabled, &option_bytes) != 0 ||
            (enabled != 0) != expected) {
            fprintf(stderr, "epoll_echo: accepted TCP_NODELAY does not match the requested policy\n");
            close(descriptor);
            mark_failed();
            return;
        }
#endif
        struct connection *link = &table[descriptor];
#if defined(WF_BENCH_SNDBUF)
        int send_bytes = WF_BENCH_SNDBUF;
        if (setsockopt(descriptor, SOL_SOCKET, SO_SNDBUF, &send_bytes, sizeof send_bytes) != 0) {
            report("setsockopt of the test send buffer", errno);
            close(descriptor);
            mark_failed();
            return;
        }
#endif
#if WF_BENCH_RECEIVE_STORAGE >= 2
#if WF_BENCH_RECEIVE_STORAGE == 3
        link->pending = (unsigned char *)calloc(1u, TRANSFER_BYTES);
#else
        link->pending = (unsigned char *)malloc(TRANSFER_BYTES);
#endif
        if (link->pending == NULL) {
            fprintf(stderr, "epoll_echo: out of memory for connection receive storage\n");
            close(descriptor);
            mark_failed();
            return;
        }
#endif
        link->active = 1;
#if defined(WF_BENCH_STACKFUL)
        link->owner = worker;
        link->descriptor = descriptor;
        link->done = 0;
#if defined(WF_BENCH_COROUTINE)
        auto task = connection_main(link);
        link->saved_sp = task.release();
        link->resume = link->saved_sp;
        if (link->saved_sp == NULL) {
            link->active = 0;
#if WF_BENCH_RECEIVE_STORAGE >= 2
            free(link->pending);
            link->pending = NULL;
#endif
            close(descriptor);
            mark_failed();
            return;
        }
#else
        link->saved_sp = wf_switch_prepare(stack_memory + ((size_t)descriptor + 1u) * stack_stride,
                                          connection_main, link);
#endif
#else
        link->offset = 0;
        link->length = 0;
#if defined(WF_BENCH_COMPUTE)
        link->received = 0;
#endif
#if defined(WF_BENCH_QUANTUM)
        link->queued = 0;
        link->computing = 0;
#endif
#endif
#if defined(WF_BENCH_STACKFUL) && defined(WF_BENCH_QUANTUM)
        link->queued = 0;
#endif
        struct epoll_event registration;
        registration.events = EPOLLIN | EPOLLOUT | EPOLLET;
        registration.data.fd = descriptor;
        if (epoll_ctl(worker->epoll, EPOLL_CTL_ADD, descriptor, &registration) != 0) {
            report("epoll_ctl of a connection", errno);
#if defined(WF_BENCH_COROUTINE)
            std::coroutine_handle<>::from_address(link->saved_sp).destroy();
            link->saved_sp = NULL;
            link->resume = NULL;
            link->active = 0;
#endif
#if WF_BENCH_RECEIVE_STORAGE >= 2
            free(link->pending);
            link->pending = NULL;
            link->active = 0;
#endif
            close(descriptor);
            mark_failed();
            return;
        }
        atomic_fetch_add_explicit(&accepted_total, 1, memory_order_relaxed);
        /* The connection may already hold bytes: an edge for them was consumed
         * by the accept, not by a later epoll_wait. */
#if defined(WF_BENCH_QUANTUM)
        enqueue(worker, descriptor);
#else
        service(worker, descriptor);
#endif
    }
}

static void *worker_main(void *raw) {
    struct worker *worker = (struct worker *)raw;
    struct epoll_event events[256];
    while (!atomic_load_explicit(&finished, memory_order_relaxed) &&
           !atomic_load_explicit(&failed, memory_order_relaxed)) {
        int timeout = -1;
#if defined(WF_BENCH_QUANTUM)
        if (worker->count != 0) timeout = 0;
#endif
        int ready = epoll_wait(worker->epoll, events, 256, timeout);
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
#if defined(WF_BENCH_QUANTUM)
            enqueue(worker, descriptor);
#else
            service(worker, descriptor);
#endif
        }
#if defined(WF_BENCH_QUANTUM)
        /* Poll between groups of at most eight continuation turns, with FIFO
         * service inside each owner. Requeued work waits behind older work. */
        unsigned turns = worker->count < 8u ? worker->count : 8u;
        for (unsigned turn = 0; turn < turns; turn++) {
            int descriptor = worker->queue[worker->head];
            worker->head = worker->head + 1u == descriptor_capacity ? 0u : worker->head + 1u;
            worker->count--;
            table[descriptor].queued = 0;
            service(worker, descriptor);
        }
#endif
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
#if !defined(WF_BENCH_NAGLE)
    setsockopt(descriptor, IPPROTO_TCP, TCP_NODELAY, &one, sizeof one);
#endif
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
#if defined(WF_BENCH_QUANTUM)
        if (strcmp(argv[at], "--quantum") == 0 && at + 1 < argc) {
            option_quantum = strtoull(argv[++at], NULL, 10);
            if (option_quantum == 0 || option_quantum > COMPUTE_MAX_ROUNDS) return 2;
            continue;
        }
#endif
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
    table = (struct connection *)calloc(descriptor_capacity, sizeof *table);
    workers = (struct worker *)calloc(option_threads, sizeof *workers);
#if WF_BENCH_RECEIVE_STORAGE < 2
    pending_memory = (unsigned char *)malloc((size_t)descriptor_capacity * TRANSFER_BYTES);
#endif
    if (table == NULL || workers == NULL
#if WF_BENCH_RECEIVE_STORAGE < 2
        || pending_memory == NULL
#endif
    ) {
        fprintf(stderr, "epoll_echo: out of memory\n");
        return 1;
    }
#if WF_BENCH_RECEIVE_STORAGE < 2
    for (unsigned at = 0; at < descriptor_capacity; at++) {
        table[at].pending = pending_memory + (size_t)at * TRANSFER_BYTES;
    }
#endif
#if defined(WF_BENCH_STACKFUL) && !defined(WF_BENCH_COROUTINE)
    /* Reserve before accepting. Only accepted connections touch a context
     * page; each slot has a low guard and 64 KiB of usable stack. Descriptor
     * reuse is safe because close happens after the final switch returns. */
    long page = sysconf(_SC_PAGESIZE);
    if (page <= 0) return 1;
    size_t usable = (65536u + (size_t)page - 1u) / (size_t)page * (size_t)page;
    stack_stride = usable + (size_t)page;
    if (descriptor_capacity > SIZE_MAX / stack_stride) return 1;
    stack_memory = (unsigned char *)mmap(NULL, (size_t)descriptor_capacity * stack_stride, PROT_NONE,
                        MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (stack_memory == MAP_FAILED) return 1;
    for (unsigned at = 0; at < descriptor_capacity; at++) {
        if (mprotect(stack_memory + (size_t)at * stack_stride + (size_t)page,
                     usable, PROT_READ | PROT_WRITE) != 0) return 1;
    }
#endif

    /* Every listener is bound before any thread runs, so a client that finds
     * the port listening finds all of them listening. */
    for (unsigned at = 0; at < option_threads; at++) {
        struct worker *worker = &workers[at];
        worker->index = (int)at;
        worker->scratch = (unsigned char *)malloc(TRANSFER_BYTES);
#if defined(WF_BENCH_QUANTUM)
        worker->queue = (int *)malloc((size_t)descriptor_capacity * sizeof *worker->queue);
        if (worker->queue == NULL) {
            fprintf(stderr, "epoll_compute: out of memory for ready queue\n");
            return 1;
        }
#endif
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
#if defined(WF_BENCH_COROUTINE)
    for (unsigned at = 0; at < descriptor_capacity; at++) {
        if (table[at].saved_sp != NULL) {
            std::coroutine_handle<>::from_address(table[at].saved_sp).destroy();
            table[at].saved_sp = NULL;
        }
    }
#if defined(WF_BENCH_OBSERVE)
    fprintf(stderr, "coroutine: allocations=%llu frees=%llu bytes=%llu\n",
            (unsigned long long)frame_allocations.load(),
            (unsigned long long)frame_frees.load(),
            (unsigned long long)frame_bytes.load());
#endif
#endif
    for (unsigned at = 0; at < option_threads; at++) {
#if defined(WF_BENCH_STACKFUL) && defined(WF_BENCH_OBSERVE)
        fprintf(stderr, "stackful: worker=%u resumes=%llu waits=%llu send_waits=%llu yields=%llu\n", at,
                (unsigned long long)workers[at].resumes,
                (unsigned long long)workers[at].waits,
                (unsigned long long)workers[at].send_waits,
                (unsigned long long)workers[at].yields);
#endif
        free(workers[at].scratch);
#if defined(WF_BENCH_QUANTUM)
        free(workers[at].queue);
#endif
        close(workers[at].wake);
        close(workers[at].listener);
        close(workers[at].epoll);
    }
    int broken = atomic_load_explicit(&failed, memory_order_relaxed);
    uint64_t accepted = atomic_load_explicit(&accepted_total, memory_order_relaxed);
    uint64_t closed = atomic_load_explicit(&closed_total, memory_order_relaxed);
#if defined(WF_BENCH_STORAGE_OBSERVE)
    fprintf(stderr, "storage: policy=%u transfer_bytes=%u accepted=%llu closed=%llu\n",
            (unsigned)WF_BENCH_RECEIVE_STORAGE, (unsigned)TRANSFER_BYTES,
            (unsigned long long)accepted, (unsigned long long)closed);
#endif
#if WF_BENCH_RECEIVE_STORAGE >= 2
    /* A failed run may stop with accepted connections still live. Normal
     * closes have already cleared their pointers, including descriptor reuse. */
    for (unsigned at = 0; at < descriptor_capacity; at++) {
        free(table[at].pending);
    }
#endif
    free(pending_memory);
    free(workers);
    free(table);
#if defined(WF_BENCH_STACKFUL) && !defined(WF_BENCH_COROUTINE)
    munmap(stack_memory, (size_t)descriptor_capacity * stack_stride);
#endif
    if (broken || accepted < option_connections || closed < option_connections) {
        fprintf(stderr, "epoll_echo: accepted %llu and closed %llu of %llu connections\n",
                (unsigned long long)accepted, (unsigned long long)closed,
                (unsigned long long)option_connections);
        return 1;
    }
    return 0;
}
