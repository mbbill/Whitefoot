/* glibc hides the POSIX and Linux entry points this file calls when the
 * translation unit is compiled as strict C11, so the namespace is asked for
 * before any header. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/* The io_uring reference of the TCP echo workload: the fastest shape the
 * kernel offers for exactly this protocol, written against the io_uring ABI
 * directly rather than liburing so it builds anywhere the rest of this bundle
 * does.
 *
 *   uring_echo PORT CONNECTIONS [--threads N] [--sqpoll]
 *
 * It listens on 127.0.0.1:PORT, echoes every byte of every connection back to
 * it, and exits 0 once CONNECTIONS connections have been accepted in total and
 * every one of them has closed. `--threads` defaults to the number of online
 * CPUs; `--sqpoll` adds IORING_SETUP_SQPOLL and is off by default.
 *
 * What it uses that a portable server cannot:
 *
 *   multishot accept        one IORING_OP_ACCEPT with IORING_ACCEPT_MULTISHOT
 *                           per listener produces a completion per connection
 *                           with no submission per accept
 *   multishot receive       one IORING_OP_RECV with IORING_RECV_MULTISHOT per
 *                           connection produces a completion per arrival with
 *                           no submission and no buffer chosen per read
 *   a provided buffer ring  IORING_REGISTER_PBUF_RING; the kernel picks the
 *                           destination out of a ring this process refills, so
 *                           the receive path has neither a per-connection
 *                           buffer nor a copy: the echo is sent straight out of
 *                           the buffer the kernel filled
 *   one ring per core       each thread has its own ring, its own buffer ring,
 *                           and its own listening socket under SO_REUSEPORT, so
 *                           a connection is accepted, received and echoed on
 *                           one thread with no shared state on the path
 *
 * Everything is sized from CONNECTIONS before the first accept. The connection
 * table is indexed by descriptor, and each connection's echo queue holds the
 * buffers the kernel has filled but the socket has not yet taken. Nothing is
 * allocated per operation.
 *
 * Ordering: exactly one send is in flight per connection, and the next one is
 * submitted from the completion of the last, so the bytes leave in the order
 * they arrived. A short send re-issues the remainder of the same buffer.
 * Buffer-ring exhaustion (-ENOBUFS) parks the connection until a buffer comes
 * back, and the receive is re-armed then. */
#include <errno.h>
#include <linux/io_uring.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/eventfd.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <unistd.h>

#define OPERATION_ACCEPT 1u
#define OPERATION_RECEIVE 2u
#define OPERATION_SEND 3u
#define OPERATION_WAKE 4u

/* One buffer holds one arrival. Larger than the small message the round-trip
 * lines send and small enough that the large-payload line still spreads one
 * message over several arrivals, which is the case the echo queue exists for. */
#define BUFFER_BYTES 8192u

/* The deepest echo queue one connection may hold. The workload's contract is
 * one outstanding message per connection of at most 64 KiB, which is eight
 * buffers; the ceiling is far above that, and a server that reached it would
 * be holding more unsent bytes than the protocol can produce. */
#define PENDING_MAX 64u

/* Submission entries per ring. Deep enough that a re-armed receive, a send and
 * an accept for every connection one thread holds are never queued behind a
 * full ring. */
#define RING_ENTRIES 4096u

struct ring {
    int descriptor;
    void *shared;
    size_t shared_bytes;
    struct io_uring_sqe *entries;
    size_t entry_bytes;
    unsigned *submission_head;
    unsigned *submission_tail;
    unsigned *submission_mask;
    unsigned *submission_flags;
    unsigned *submission_array;
    unsigned *completion_head;
    unsigned *completion_tail;
    unsigned *completion_mask;
    struct io_uring_cqe *completions;
    /* The tail this thread has filled but not yet published. Entries are
     * written first and the shared tail is stored once, with release order,
     * when the ring is entered: under SQPOLL the kernel thread reads that tail
     * without being asked, so publishing it before the entry is written would
     * hand it an entry that is not there yet. */
    unsigned local_tail;
    unsigned unsubmitted;
    int poll_thread;
};

struct connection {
    int active;
    int closing;
    int sending;
    int starved;
    unsigned head;
    unsigned count;
    uint16_t queue_buffer[PENDING_MAX];
    uint32_t queue_offset[PENDING_MAX];
    uint32_t queue_length[PENDING_MAX];
    /* The whole queue leaves in one operation. The vector is rebuilt only
     * when a send is armed, so a receive that lands while one is in flight
     * appends to the queue and waits for the next. */
    struct msghdr message;
    struct iovec vector[PENDING_MAX];
};

struct worker {
    pthread_t thread;
    int index;
    int listener;
    int wake;
    uint64_t wake_storage;
    struct ring ring;
    struct io_uring_buf_ring *buffers;
    size_t buffer_ring_bytes;
    unsigned char *buffer_memory;
    unsigned buffer_count;
    unsigned buffer_mask;
    uint16_t buffer_tail;
    int *starved_list;
    unsigned starved_count;
};

static uint64_t option_connections;
static uint16_t option_port;
static unsigned option_threads;
static int option_sqpoll;
static unsigned descriptor_capacity;
static struct connection *table;
static struct worker *workers;
static _Atomic uint64_t accepted_total;
static _Atomic uint64_t closed_total;
static _Atomic int finished;
static _Atomic int failed;

static void wake_everyone(void);

/* A failure ends the run for every thread, not only the one that saw it: the
 * others are blocked in io_uring_enter with nothing left to complete. */
static void mark_failed(void) {
    atomic_store_explicit(&failed, 1, memory_order_relaxed);
    wake_everyone();
}

static void report(const char *what, int error) {
    fprintf(stderr, "uring_echo: %s: %s\n", what, strerror(error));
    fflush(stderr);
}

static unsigned round_up_power_of_two(unsigned value) {
    unsigned result = 1;
    while (result < value) {
        result *= 2;
    }
    return result;
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

/* --- the ring ---------------------------------------------------------- */

static int ring_setup(struct ring *ring, unsigned entries, int poll_thread) {
    struct io_uring_params parameters;
    memset(&parameters, 0, sizeof parameters);
    if (poll_thread) {
        parameters.flags |= IORING_SETUP_SQPOLL;
        parameters.sq_thread_idle = 2000;
    } else {
        /* One thread owns this ring: it is the only one that submits and the
         * only one that waits. Saying so lets the kernel defer the completion
         * work to the moment this thread asks for it instead of interrupting
         * it, which both removes the interrupt and lets several arrivals for
         * one connection be reaped together. The pair is worth about a
         * quarter of the small-message rate here. SQPOLL cannot be combined
         * with it, because there the submitting task is the kernel's own. */
        parameters.flags |= IORING_SETUP_SINGLE_ISSUER | IORING_SETUP_DEFER_TASKRUN;
    }
    long created = syscall(__NR_io_uring_setup, entries, &parameters);
    if (created < 0) {
        report("io_uring_setup", errno);
        return 1;
    }
    ring->descriptor = (int)created;
    ring->poll_thread = poll_thread;
    if ((parameters.features & IORING_FEAT_SINGLE_MMAP) == 0) {
        fprintf(stderr, "uring_echo: this kernel has no single-mmap rings\n");
        close(ring->descriptor);
        return 1;
    }
    if (poll_thread && (parameters.features & IORING_FEAT_SQPOLL_NONFIXED) == 0) {
        fprintf(stderr, "uring_echo: this kernel's SQPOLL needs registered files\n");
        close(ring->descriptor);
        return 1;
    }
    size_t shared_bytes = parameters.sq_off.array + parameters.sq_entries * sizeof(unsigned);
    size_t completion_bytes =
        parameters.cq_off.cqes + parameters.cq_entries * sizeof(struct io_uring_cqe);
    if (completion_bytes > shared_bytes) {
        shared_bytes = completion_bytes;
    }
    void *map = mmap(NULL, shared_bytes, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE,
                     ring->descriptor, IORING_OFF_SQ_RING);
    if (map == MAP_FAILED) {
        report("mmap of the ring", errno);
        close(ring->descriptor);
        return 1;
    }
    ring->shared = map;
    ring->shared_bytes = shared_bytes;
    ring->entry_bytes = parameters.sq_entries * sizeof(struct io_uring_sqe);
    ring->entries = mmap(NULL, ring->entry_bytes, PROT_READ | PROT_WRITE,
                         MAP_SHARED | MAP_POPULATE, ring->descriptor, IORING_OFF_SQES);
    if (ring->entries == MAP_FAILED) {
        report("mmap of the submission entries", errno);
        munmap(map, shared_bytes);
        ring->shared = NULL;
        close(ring->descriptor);
        return 1;
    }
    unsigned char *base = map;
    ring->submission_head = (unsigned *)(base + parameters.sq_off.head);
    ring->submission_tail = (unsigned *)(base + parameters.sq_off.tail);
    ring->submission_mask = (unsigned *)(base + parameters.sq_off.ring_mask);
    ring->submission_flags = (unsigned *)(base + parameters.sq_off.flags);
    ring->submission_array = (unsigned *)(base + parameters.sq_off.array);
    ring->completion_head = (unsigned *)(base + parameters.cq_off.head);
    ring->completion_tail = (unsigned *)(base + parameters.cq_off.tail);
    ring->completion_mask = (unsigned *)(base + parameters.cq_off.ring_mask);
    ring->completions = (struct io_uring_cqe *)(base + parameters.cq_off.cqes);
    ring->unsubmitted = 0;
    ring->local_tail = 0;
    /* The submission array never changes: entry k of the ring is always
     * submission slot k, so the array is filled once here instead of on every
     * submission. */
    for (unsigned at = 0; at <= *ring->submission_mask; at++) {
        ring->submission_array[at] = at;
    }
    return 0;
}

static void ring_teardown(struct ring *ring) {
    munmap(ring->entries, ring->entry_bytes);
    munmap(ring->shared, ring->shared_bytes);
    close(ring->descriptor);
}

static int ring_enter(struct ring *ring, unsigned wait_for);

/* Hands back the next free submission entry, submitting what is already
 * queued if the ring is full. */
static struct io_uring_sqe *ring_next(struct ring *ring) {
    for (;;) {
        unsigned head =
            atomic_load_explicit((_Atomic unsigned *)ring->submission_head, memory_order_acquire);
        unsigned mask = *ring->submission_mask;
        if ((ring->local_tail - head) <= mask) {
            struct io_uring_sqe *entry = &ring->entries[ring->local_tail & mask];
            memset(entry, 0, sizeof *entry);
            ring->local_tail++;
            ring->unsubmitted++;
            return entry;
        }
        if (ring_enter(ring, 0) != 0) {
            return NULL;
        }
    }
}

static int ring_enter(struct ring *ring, unsigned wait_for) {
    unsigned flags = 0;
    unsigned to_submit = ring->unsubmitted;
    if (to_submit > 0) {
        atomic_store_explicit((_Atomic unsigned *)ring->submission_tail, ring->local_tail,
                              memory_order_release);
    }
    if (ring->poll_thread) {
        unsigned state = atomic_load_explicit((_Atomic unsigned *)ring->submission_flags,
                                              memory_order_acquire);
        /* The poll thread reads the tail itself, so the count is ignored; the
         * only reason to enter is to wake it or to wait for completions. */
        to_submit = 0;
        if ((state & IORING_SQ_NEED_WAKEUP) != 0) {
            flags |= IORING_ENTER_SQ_WAKEUP;
        } else if (wait_for == 0) {
            ring->unsubmitted = 0;
            return 0;
        }
    }
    if (wait_for > 0) {
        flags |= IORING_ENTER_GETEVENTS;
    }
    if (to_submit == 0 && flags == 0) {
        ring->unsubmitted = 0;
        return 0;
    }
    long entered = syscall(__NR_io_uring_enter, ring->descriptor, to_submit, wait_for, flags,
                           NULL, 0);
    if (entered < 0 && errno != EINTR && errno != EBUSY) {
        report("io_uring_enter", errno);
        return 1;
    }
    ring->unsubmitted = 0;
    return 0;
}

/* --- the provided buffer ring ------------------------------------------- */

static int buffers_setup(struct worker *worker) {
    worker->buffer_ring_bytes = (size_t)worker->buffer_count * sizeof(struct io_uring_buf);
    void *map = mmap(NULL, worker->buffer_ring_bytes, PROT_READ | PROT_WRITE,
                     MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
    if (map == MAP_FAILED) {
        report("mmap of the buffer ring", errno);
        return 1;
    }
    worker->buffers = map;
    memset(worker->buffers, 0, worker->buffer_ring_bytes);
    struct io_uring_buf_reg registration;
    memset(&registration, 0, sizeof registration);
    registration.ring_addr = (unsigned long long)(uintptr_t)map;
    registration.ring_entries = worker->buffer_count;
    registration.bgid = (unsigned short)worker->index;
    if (syscall(__NR_io_uring_register, worker->ring.descriptor, IORING_REGISTER_PBUF_RING,
                &registration, 1)
        < 0) {
        report("io_uring_register of the buffer ring", errno);
        munmap(map, worker->buffer_ring_bytes);
        return 1;
    }
    worker->buffer_memory = malloc((size_t)worker->buffer_count * BUFFER_BYTES);
    if (worker->buffer_memory == NULL) {
        fprintf(stderr, "uring_echo: out of memory for the buffers\n");
        return 1;
    }
    for (unsigned at = 0; at < worker->buffer_count; at++) {
        struct io_uring_buf *slot = &worker->buffers->bufs[at];
        slot->addr = (unsigned long long)(uintptr_t)(worker->buffer_memory + (size_t)at * BUFFER_BYTES);
        slot->len = BUFFER_BYTES;
        slot->bid = (unsigned short)at;
    }
    worker->buffer_tail = (uint16_t)worker->buffer_count;
    atomic_store_explicit((_Atomic uint16_t *)&worker->buffers->tail, worker->buffer_tail,
                          memory_order_release);
    return 0;
}

static void arm_receive(struct worker *worker, int descriptor);

/* Returns one buffer to the ring and wakes whatever connection was parked
 * waiting for one. Only the owning thread publishes into its own ring, so the
 * tail needs no lock. */
static void buffer_return(struct worker *worker, uint16_t identifier) {
    struct io_uring_buf *slot = &worker->buffers->bufs[worker->buffer_tail & worker->buffer_mask];
    slot->addr =
        (unsigned long long)(uintptr_t)(worker->buffer_memory + (size_t)identifier * BUFFER_BYTES);
    slot->len = BUFFER_BYTES;
    slot->bid = identifier;
    worker->buffer_tail++;
    atomic_store_explicit((_Atomic uint16_t *)&worker->buffers->tail, worker->buffer_tail,
                          memory_order_release);
    while (worker->starved_count > 0) {
        int descriptor = worker->starved_list[--worker->starved_count];
        table[descriptor].starved = 0;
        if (table[descriptor].active && !table[descriptor].closing) {
            arm_receive(worker, descriptor);
        }
    }
}

/* --- the operations ------------------------------------------------------ */

static uint64_t tag(unsigned operation, int descriptor) {
    return ((uint64_t)operation << 32) | (uint64_t)(uint32_t)descriptor;
}

static void arm_accept(struct worker *worker) {
    struct io_uring_sqe *entry = ring_next(&worker->ring);
    if (entry == NULL) {
        mark_failed();
        return;
    }
    entry->opcode = IORING_OP_ACCEPT;
    entry->fd = worker->listener;
    entry->ioprio = IORING_ACCEPT_MULTISHOT;
    entry->user_data = tag(OPERATION_ACCEPT, worker->listener);
}

static void arm_receive(struct worker *worker, int descriptor) {
    struct io_uring_sqe *entry = ring_next(&worker->ring);
    if (entry == NULL) {
        mark_failed();
        return;
    }
    entry->opcode = IORING_OP_RECV;
    entry->fd = descriptor;
    entry->ioprio = IORING_RECV_MULTISHOT;
    entry->flags = IOSQE_BUFFER_SELECT;
    entry->buf_group = (unsigned short)worker->index;
    entry->user_data = tag(OPERATION_RECEIVE, descriptor);
}

static void arm_wake(struct worker *worker) {
    struct io_uring_sqe *entry = ring_next(&worker->ring);
    if (entry == NULL) {
        mark_failed();
        return;
    }
    entry->opcode = IORING_OP_READ;
    entry->fd = worker->wake;
    entry->addr = (unsigned long long)(uintptr_t)&worker->wake_storage;
    entry->len = sizeof worker->wake_storage;
    entry->user_data = tag(OPERATION_WAKE, worker->wake);
}

/* The echo queue, sent straight out of the buffers the kernel filled: no copy
 * on either side of the exchange. Exactly one send is in flight per
 * connection, so the bytes leave in the order they arrived, and everything the
 * connection is holding leaves in that one send rather than one operation per
 * arrival -- which is what keeps a large message from costing a completion
 * round trip per buffer. */
static void arm_send(struct worker *worker, int descriptor) {
    struct connection *link = &table[descriptor];
    struct io_uring_sqe *entry = ring_next(&worker->ring);
    if (entry == NULL) {
        mark_failed();
        return;
    }
    for (unsigned at = 0; at < link->count; at++) {
        unsigned slot = (link->head + at) % PENDING_MAX;
        link->vector[at].iov_base = worker->buffer_memory +
                                    (size_t)link->queue_buffer[slot] * BUFFER_BYTES +
                                    link->queue_offset[slot];
        link->vector[at].iov_len = link->queue_length[slot] - link->queue_offset[slot];
    }
    memset(&link->message, 0, sizeof link->message);
    link->message.msg_iov = link->vector;
    link->message.msg_iovlen = link->count;
    entry->opcode = IORING_OP_SENDMSG;
    entry->fd = descriptor;
    entry->addr = (unsigned long long)(uintptr_t)&link->message;
    entry->len = 1;
    entry->msg_flags = MSG_NOSIGNAL;
    entry->user_data = tag(OPERATION_SEND, descriptor);
    link->sending = 1;
}

static void close_connection(struct worker *worker, int descriptor) {
    struct connection *link = &table[descriptor];
    /* A parked connection leaves the starved list here. The kernel hands the
     * same descriptor number to the next accept, and a re-arm left over from
     * the closed connection would put a second multishot receive on the new
     * one. */
    if (link->starved) {
        for (unsigned at = 0; at < worker->starved_count; at++) {
            if (worker->starved_list[at] == descriptor) {
                worker->starved_list[at] = worker->starved_list[--worker->starved_count];
                break;
            }
        }
        link->starved = 0;
    }
    while (link->count > 0) {
        buffer_return(worker, link->queue_buffer[link->head]);
        link->head = (link->head + 1) % PENDING_MAX;
        link->count--;
    }
    link->active = 0;
    link->sending = 0;
    link->closing = 0;
    close(descriptor);
    atomic_fetch_add_explicit(&closed_total, 1, memory_order_relaxed);
}

/* The exit condition, checked by whichever thread completes the last close:
 * CONNECTIONS accepted and all of them closed. The thread that sees it wakes
 * every other ring, since the others are blocked in io_uring_enter with
 * nothing left to complete. */
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

static void *worker_main(void *raw) {
    struct worker *worker = raw;
    /* The ring is set up here rather than in main because IORING_SETUP_
     * SINGLE_ISSUER binds the ring to the task that created it, and this is
     * the task that will submit and wait on it. The listeners are already
     * bound, so a client that finds the port listening finds every listener
     * listening whatever these threads are still doing. */
    if (ring_setup(&worker->ring, RING_ENTRIES, option_sqpoll) != 0 ||
        buffers_setup(worker) != 0) {
        mark_failed();
        return NULL;
    }
    arm_accept(worker);
    arm_wake(worker);
    while (!atomic_load_explicit(&finished, memory_order_relaxed) &&
           !atomic_load_explicit(&failed, memory_order_relaxed)) {
        if (ring_enter(&worker->ring, 1) != 0) {
            mark_failed();
            break;
        }
        unsigned head = atomic_load_explicit((_Atomic unsigned *)worker->ring.completion_head,
                                             memory_order_relaxed);
        unsigned tail = atomic_load_explicit((_Atomic unsigned *)worker->ring.completion_tail,
                                             memory_order_acquire);
        while (head != tail) {
            struct io_uring_cqe *completion =
                &worker->ring.completions[head & *worker->ring.completion_mask];
            unsigned operation = (unsigned)(completion->user_data >> 32);
            int descriptor = (int)(uint32_t)completion->user_data;
            int result = completion->res;
            unsigned flags = completion->flags;
            head++;
            /* The head is published before the work below so a multishot
             * operation the kernel is still filling never sees a full ring. */
            atomic_store_explicit((_Atomic unsigned *)worker->ring.completion_head, head,
                                  memory_order_release);
            if (operation == OPERATION_ACCEPT) {
                if (result >= 0) {
#if defined(WF_BENCH_TCP_VERIFY)
                    int enabled = 0;
                    socklen_t option_bytes = sizeof(enabled);
#if defined(WF_BENCH_NAGLE)
                    const int expected = 0;
#else
                    const int expected = 1;
#endif
                    if (getsockopt(result, IPPROTO_TCP, TCP_NODELAY, &enabled, &option_bytes) != 0 ||
                        (enabled != 0) != expected) {
                        fprintf(stderr, "uring_echo: accepted TCP_NODELAY does not match the requested policy\n");
                        close(result);
                        mark_failed();
                        break;
                    }
#endif
                    if ((unsigned)result >= descriptor_capacity) {
                        fprintf(stderr, "uring_echo: descriptor %d is past the table\n", result);
                        mark_failed();
                        break;
                    }
                    struct connection *link = &table[result];
                    memset(link, 0, sizeof *link);
                    link->active = 1;
                    atomic_fetch_add_explicit(&accepted_total, 1, memory_order_relaxed);
                    arm_receive(worker, result);
                } else if (result != -ECANCELED) {
                    report("accept", -result);
                    mark_failed();
                    break;
                }
                if ((flags & IORING_CQE_F_MORE) == 0 &&
                    atomic_load_explicit(&accepted_total, memory_order_relaxed) <
                        option_connections) {
                    arm_accept(worker);
                }
            } else if (operation == OPERATION_RECEIVE) {
                struct connection *link = &table[descriptor];
                if (!link->active) {
                    continue;
                }
                if (result > 0) {
                    if ((flags & IORING_CQE_F_BUFFER) == 0) {
                        fprintf(stderr, "uring_echo: a receive completed with no buffer\n");
                        mark_failed();
                        break;
                    }
                    uint16_t identifier = (uint16_t)(flags >> IORING_CQE_BUFFER_SHIFT);
                    if (link->count == PENDING_MAX) {
                        fprintf(stderr, "uring_echo: connection %d holds %u unsent buffers\n",
                                descriptor, PENDING_MAX);
                        mark_failed();
                        break;
                    }
                    unsigned slot = (link->head + link->count) % PENDING_MAX;
                    link->queue_buffer[slot] = identifier;
                    link->queue_offset[slot] = 0;
                    link->queue_length[slot] = (uint32_t)result;
                    link->count++;
                    if (!link->sending) {
                        arm_send(worker, descriptor);
                    }
                } else if (result == 0) {
                    /* The peer has stopped sending. Everything it did send is
                     * echoed first; the close follows the last send. */
                    link->closing = 1;
                    if (!link->sending) {
                        close_connection(worker, descriptor);
                        check_finished();
                        continue;
                    }
                } else if (result == -ENOBUFS) {
                    /* The buffer ring ran dry. The connection waits for a
                     * buffer to come back rather than spinning on a re-arm. */
                    if (!link->starved) {
                        link->starved = 1;
                        worker->starved_list[worker->starved_count++] = descriptor;
                    }
                    continue;
                } else if (result != -ECANCELED) {
                    link->closing = 1;
                    if (!link->sending) {
                        close_connection(worker, descriptor);
                        check_finished();
                        continue;
                    }
                }
                if ((flags & IORING_CQE_F_MORE) == 0 && link->active && !link->closing &&
                    !link->starved) {
                    arm_receive(worker, descriptor);
                }
            } else if (operation == OPERATION_SEND) {
                struct connection *link = &table[descriptor];
                if (!link->active) {
                    continue;
                }
                link->sending = 0;
                if (result > 0) {
                    /* A short send leaves the rest of the queue where it is,
                     * with the partly sent buffer's offset advanced; the next
                     * send re-issues from there. */
                    uint32_t left = (uint32_t)result;
                    while (left > 0 && link->count > 0) {
                        unsigned slot = link->head;
                        uint32_t remaining = link->queue_length[slot] - link->queue_offset[slot];
                        if (left < remaining) {
                            link->queue_offset[slot] += left;
                            break;
                        }
                        left -= remaining;
                        uint16_t identifier = link->queue_buffer[slot];
                        link->head = (link->head + 1) % PENDING_MAX;
                        link->count--;
                        buffer_return(worker, identifier);
                    }
                    if (link->count > 0) {
                        arm_send(worker, descriptor);
                        continue;
                    }
                    if (link->closing) {
                        close_connection(worker, descriptor);
                        check_finished();
                    }
                    continue;
                }
                if (result != -ECANCELED) {
                    report("send", result < 0 ? -result : 0);
                }
                close_connection(worker, descriptor);
                check_finished();
            } else if (operation == OPERATION_WAKE) {
                if (!atomic_load_explicit(&finished, memory_order_relaxed)) {
                    arm_wake(worker);
                }
            }
        }
        if (atomic_load_explicit(&failed, memory_order_relaxed)) {
            break;
        }
        check_finished();
    }
    return NULL;
}

static int listener_for(uint16_t port) {
    int descriptor = socket(AF_INET, SOCK_STREAM, 0);
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
        fprintf(stderr, "usage: uring_echo PORT CONNECTIONS [--threads N] [--sqpoll]\n");
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
        if (strcmp(argv[at], "--sqpoll") == 0) {
            option_sqpoll = 1;
            continue;
        }
        fprintf(stderr, "uring_echo: unknown argument %s\n", argv[at]);
        return 2;
    }
    if (port == 0 || port > 65535 || option_connections == 0 || option_threads == 0) {
        fprintf(stderr, "uring_echo: PORT, CONNECTIONS and N must be positive\n");
        return 2;
    }
    option_port = (uint16_t)port;
    raise_descriptor_limit(option_connections + 8 * option_threads);

    descriptor_capacity = (unsigned)(option_connections + 8 * option_threads + 64);
    table = calloc(descriptor_capacity, sizeof *table);
    workers = calloc(option_threads, sizeof *workers);
    if (table == NULL || workers == NULL) {
        fprintf(stderr, "uring_echo: out of memory\n");
        return 1;
    }
    /* Every buffer ring, every echo queue and the connection table are sized
     * here, from CONNECTIONS, and nothing is sized again once a connection
     * arrives. */
    unsigned wanted = (unsigned)(option_connections * 4 / option_threads);
    if (wanted < 256) {
        wanted = 256;
    }
    if (wanted > 2048) {
        wanted = 2048;
    }
    unsigned buffers = round_up_power_of_two(wanted);

    /* Every listener is bound before any thread runs, so a client that finds
     * the port listening finds all of them listening. */
    for (unsigned at = 0; at < option_threads; at++) {
        struct worker *worker = &workers[at];
        worker->index = (int)at;
        worker->buffer_count = buffers;
        worker->buffer_mask = buffers - 1;
        worker->listener = listener_for(option_port);
        if (worker->listener < 0) {
            return 1;
        }
        worker->wake = eventfd(0, 0);
        if (worker->wake < 0) {
            report("eventfd", errno);
            return 1;
        }
        worker->starved_list = calloc(descriptor_capacity, sizeof *worker->starved_list);
        if (worker->starved_list == NULL) {
            fprintf(stderr, "uring_echo: out of memory\n");
            return 1;
        }
    }

    for (unsigned at = 0; at < option_threads; at++) {
        if (pthread_create(&workers[at].thread, NULL, worker_main, &workers[at]) != 0) {
            fprintf(stderr, "uring_echo: pthread_create failed\n");
            return 1;
        }
    }
    for (unsigned at = 0; at < option_threads; at++) {
        pthread_join(workers[at].thread, NULL);
    }
    for (unsigned at = 0; at < option_threads; at++) {
        struct worker *worker = &workers[at];
        free(worker->starved_list);
        free(worker->buffer_memory);
        if (worker->buffers != NULL) {
            munmap(worker->buffers, worker->buffer_ring_bytes);
        }
        if (worker->ring.shared != NULL) {
            ring_teardown(&worker->ring);
        }
        close(worker->wake);
        close(worker->listener);
    }
    int broken = atomic_load_explicit(&failed, memory_order_relaxed);
    uint64_t accepted = atomic_load_explicit(&accepted_total, memory_order_relaxed);
    uint64_t closed = atomic_load_explicit(&closed_total, memory_order_relaxed);
    free(workers);
    free(table);
    if (broken || accepted < option_connections || closed < option_connections) {
        fprintf(stderr, "uring_echo: accepted %llu and closed %llu of %llu connections\n",
                (unsigned long long)accepted, (unsigned long long)closed,
                (unsigned long long)option_connections);
        return 1;
    }
    return 0;
}
