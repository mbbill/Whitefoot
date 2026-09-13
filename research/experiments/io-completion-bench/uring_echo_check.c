/* Deterministic transport traces through the actual native reference. Kernel
 * setup/enter and synchronous send results are simulated; this is not Linux
 * socket qualification. Owned by uring-check; remove with the reference. */
#define WF_BENCH_URING_TEST 1
#define main uring_echo_program_main
#include "uring_echo.c"
#undef main
#include <fcntl.h>

#define TEST_RING_SIZE 64u
#define TEST_PEERS 2u
#define TEST_BYTES 32u

enum trace_case {
    RETURN_BEFORE, RETURN_AFTER, NO_RETURN, RETRY_CONSUMED,
    TERMINAL_EOF, TERMINAL_CANCEL, SHORT_SEND, TRACE_COUNT
};
static const char *trace_names[] = {
    "return-before-exhaustion", "return-after-exhaustion", "no-return-no-spin",
    "retry-credit-consumed", "terminal-buffer-eof", "terminal-buffer-cancel",
    "short-send-retirement"
};
struct transport_send {
    struct iovec vector[SEND_VECTOR_MAX];
    size_t vectors;
    size_t bytes;
    int pending;
};
static struct {
    enum trace_case scenario;
    unsigned step, peers, receive_arms[TEST_PEERS];
    unsigned submission_head, submission_tail, submission_mask, submission_flags;
    unsigned completion_head, completion_tail, completion_mask;
    unsigned buffer_head;
    int descriptors[TEST_PEERS];
    size_t received[TEST_PEERS], sent[TEST_PEERS];
    struct transport_send sends[TEST_PEERS];
    struct io_uring_sqe entries[TEST_RING_SIZE];
    struct io_uring_cqe *completions;
    unsigned submission_array[TEST_RING_SIZE];
} transport;

static void require_trace(int condition, const char *reason) {
    if (!condition) {
        fprintf(stderr, "uring trace: %s inline=%u FAIL: %s\n",
                trace_names[transport.scenario], (unsigned)WF_BENCH_URING_INLINE_SEND, reason);
        exit(2);
    }
}
static unsigned peer_index(int descriptor) {
    for (unsigned at = 0; at < transport.peers; at++)
        if (transport.descriptors[at] == descriptor) return at;
    require_trace(0, "unknown peer descriptor");
    return 0;
}
static unsigned char expected_byte(unsigned peer, size_t offset) {
    return (unsigned char)(17u + peer * 61u + offset * 7u);
}
static void check_transfer(unsigned peer, const struct iovec *vectors, size_t count,
                           size_t bytes) {
    size_t remaining = bytes;
    for (size_t at = 0; at < count && remaining; at++) {
        size_t take = vectors[at].iov_len < remaining ? vectors[at].iov_len : remaining;
        const unsigned char *data = vectors[at].iov_base;
        uintptr_t start = (uintptr_t)workers[0].buffer_memory;
        uintptr_t address = (uintptr_t)data;
        require_trace(address >= start && address - start <= 2u * BUFFER_BYTES &&
                      take <= 2u * BUFFER_BYTES - (address - start),
                      "send vector outside provided storage");
        for (size_t byte = 0; byte < take; byte++)
            require_trace(data[byte] == expected_byte(peer, transport.sent[peer]++),
                          "send reordered or reused a loan before completion");
        remaining -= take;
    }
    require_trace(remaining == 0, "send exceeds submitted vectors");
}
static void take_submissions(struct ring *ring) {
    while (transport.submission_head != ring->local_tail) {
        const struct io_uring_sqe *entry =
            &ring->entries[transport.submission_head++ & transport.submission_mask];
        if (entry->opcode == IORING_OP_RECV) {
            unsigned peer = peer_index(entry->fd);
            require_trace(entry->ioprio == IORING_RECV_MULTISHOT &&
                          entry->flags == IOSQE_BUFFER_SELECT,
                          "receive lost multishot/buffer selection");
            transport.receive_arms[peer]++;
        } else if (entry->opcode == IORING_OP_SENDMSG) {
            unsigned peer = peer_index(entry->fd);
            struct transport_send *send = &transport.sends[peer];
            const struct msghdr *message = (const void *)(uintptr_t)entry->addr;
            require_trace(!send->pending && message->msg_iovlen <= SEND_VECTOR_MAX,
                          "second send or oversized vector submission");
            send->pending = 1;
            send->vectors = message->msg_iovlen;
            send->bytes = 0;
            for (size_t at = 0; at < send->vectors; at++) {
                send->vector[at] = message->msg_iov[at];
                send->bytes += send->vector[at].iov_len;
            }
        } else {
            require_trace(entry->opcode == IORING_OP_ACCEPT || entry->opcode == IORING_OP_READ,
                          "unexpected kernel operation");
        }
    }
    ring->unsubmitted = 0;
}
static void completion(unsigned operation, unsigned peer, int result, unsigned flags) {
    require_trace(transport.completion_tail - transport.completion_head < TEST_RING_SIZE,
                  "completion fixture overflow");
    struct io_uring_cqe *cqe = &transport.completions[
        transport.completion_tail++ & transport.completion_mask];
    cqe->user_data = tag(operation, transport.descriptors[peer]);
    cqe->res = result;
    cqe->flags = flags;
}
static void receive_completion(unsigned peer, int result, unsigned flags) {
    if (flags & IORING_CQE_F_BUFFER) {
        struct worker *worker = &workers[0];
        require_trace((uint16_t)(worker->buffer_tail - transport.buffer_head) > 0,
                      "simulated kernel selected an unpublished buffer");
        unsigned slot = transport.buffer_head++ & worker->buffer_mask;
        uint16_t identifier = worker->buffers->bufs[slot].bid;
        require_trace(identifier < worker->buffer_count, "published invalid buffer ID");
        flags |= (unsigned)identifier << IORING_CQE_BUFFER_SHIFT;
        if (result > 0) {
            unsigned char *bytes = worker->buffer_memory + (size_t)identifier * BUFFER_BYTES;
            for (int at = 0; at < result; at++)
                bytes[at] = expected_byte(peer, transport.received[peer]++);
        }
    }
    completion(OPERATION_RECEIVE, peer, result, flags);
}
static void send_completion(unsigned peer, size_t bytes) {
    struct transport_send *send = &transport.sends[peer];
    require_trace(send->pending && bytes > 0 && bytes <= send->bytes,
                  "completion without a corresponding send");
    const struct msghdr *message = &table[transport.descriptors[peer]].message;
    require_trace(message->msg_iovlen == send->vectors, "in-flight vector count changed");
    for (size_t at = 0; at < send->vectors; at++)
        require_trace(message->msg_iov[at].iov_base == send->vector[at].iov_base &&
                      message->msg_iov[at].iov_len == send->vector[at].iov_len,
                      "in-flight send vector changed");
    check_transfer(peer, send->vector, send->vectors, bytes);
    send->pending = 0;
    completion(OPERATION_SEND, peer, (int)bytes, 0);
}
static int uring_test_ring_setup(struct ring *ring, unsigned entries, int poll_thread) {
    (void)entries; (void)poll_thread;
    transport.completions = calloc(TEST_RING_SIZE, sizeof *transport.completions);
    require_trace(transport.completions != NULL, "completion fixture allocation");
    transport.submission_mask = transport.completion_mask = TEST_RING_SIZE - 1u;
    ring->entries = transport.entries;
    ring->submission_head = &transport.submission_head;
    ring->submission_tail = &transport.submission_tail;
    ring->submission_mask = &transport.submission_mask;
    ring->submission_flags = &transport.submission_flags;
    ring->submission_array = transport.submission_array;
    ring->completion_head = &transport.completion_head;
    ring->completion_tail = &transport.completion_tail;
    ring->completion_mask = &transport.completion_mask;
    ring->completions = transport.completions;
    return 0;
}
static int uring_test_buffers_setup(struct worker *worker) {
    worker->buffer_count = 2u;
    worker->buffer_mask = 1u;
    worker->buffer_ring_bytes = 2u * sizeof(struct io_uring_buf);
    worker->buffers = calloc(1u, worker->buffer_ring_bytes);
    worker->buffer_memory = calloc(2u, BUFFER_BYTES);
    worker->loans = calloc(2u, sizeof *worker->loans);
    require_trace(worker->buffers && worker->buffer_memory && worker->loans,
                  "buffer fixture allocation");
    for (unsigned at = 0; at < 2u; at++) {
        worker->buffers->bufs[at].bid = (uint16_t)at;
        worker->buffers->bufs[at].addr =
            (unsigned long long)(uintptr_t)(worker->buffer_memory + (size_t)at * BUFFER_BYTES);
        worker->buffers->bufs[at].len = BUFFER_BYTES;
    }
    worker->buffer_tail = 2u;
    worker->buffers->tail = worker->buffer_tail;
    return 0;
}
#if WF_BENCH_URING_INLINE_SEND
static ssize_t uring_test_sendmsg(int descriptor, const struct msghdr *message, int flags) {
    (void)flags;
    unsigned peer = peer_index(descriptor);
    if (transport.scenario != RETURN_BEFORE && transport.scenario != SHORT_SEND) {
        errno = EAGAIN;
        return -1;
    }
    size_t bytes = 0;
    for (size_t at = 0; at < message->msg_iovlen; at++) bytes += message->msg_iov[at].iov_len;
    if (transport.scenario == SHORT_SEND && transport.sent[peer] == 0) bytes = 16u;
    check_transfer(peer, message->msg_iov, message->msg_iovlen, bytes);
    return (ssize_t)bytes;
}
#endif
static int uring_test_ring_enter(struct ring *ring, unsigned wait_for) {
    require_trace(wait_for == 1, "unexpected fixture submission pressure");
    take_submissions(ring);
    unsigned step = transport.step++;
    if (step == 0) {
        for (unsigned peer = 0; peer < transport.peers; peer++)
            completion(OPERATION_ACCEPT, peer, transport.descriptors[peer], IORING_CQE_F_MORE);
        return 0;
    }
    unsigned loan = IORING_CQE_F_BUFFER | IORING_CQE_F_MORE;
    switch (transport.scenario) {
    case RETURN_BEFORE:
        if (step == 1) {
            receive_completion(0, TEST_BYTES, loan);
            receive_completion(1, TEST_BYTES, loan);
#if WF_BENCH_URING_INLINE_SEND
            /* Both buffers are exhausted when these terminal CQEs are queued.
             * The real inline path returns both before consuming them. */
            completion(OPERATION_RECEIVE, 0, -ENOBUFS, 0);
            completion(OPERATION_RECEIVE, 1, -ENOBUFS, 0);
#else
        } else if (step == 2) {
            /* Send CQEs precede the exhaustion CQEs; only userspace can
             * republish the associated buffers while consuming this batch. */
            send_completion(0, TEST_BYTES);
            send_completion(1, TEST_BYTES);
            completion(OPERATION_RECEIVE, 0, -ENOBUFS, 0);
            completion(OPERATION_RECEIVE, 1, -ENOBUFS, 0);
#endif
        } else break;
        return 0;
    case RETURN_AFTER:
    case NO_RETURN:
        if (step == 1) {
            receive_completion(0, TEST_BYTES, loan);
            receive_completion(1, TEST_BYTES, loan);
            completion(OPERATION_RECEIVE, 0, -ENOBUFS, 0);
            completion(OPERATION_RECEIVE, 1, -ENOBUFS, 0);
        } else if (transport.scenario == RETURN_AFTER && step == 2) {
            send_completion(0, TEST_BYTES);
            send_completion(1, TEST_BYTES);
        } else if (transport.scenario == NO_RETURN && step < 5) {
            /* A stalled transport produces no new buffer returns. */
        } else break;
        return 0;
    case RETRY_CONSUMED:
        if (step == 1) {
            receive_completion(0, TEST_BYTES, loan);
            receive_completion(1, TEST_BYTES, loan);
        } else if (step == 2) {
            send_completion(0, TEST_BYTES);
            completion(OPERATION_RECEIVE, 0, -ENOBUFS, 0);
        } else if (step == 3) {
            /* Peer 1 takes the returned buffer before peer 0's retry runs. */
            receive_completion(1, TEST_BYTES, loan);
            completion(OPERATION_RECEIVE, 0, -ENOBUFS, 0);
        } else if (step < 6) {
        } else break;
        return 0;
    case TERMINAL_EOF:
        if (step == 1) {
            receive_completion(0, TEST_BYTES, loan);
            receive_completion(0, 0, IORING_CQE_F_BUFFER);
        } else if (step == 2) send_completion(0, TEST_BYTES);
        else break;
        return 0;
    case TERMINAL_CANCEL:
        if (step == 1) receive_completion(0, -ECANCELED, IORING_CQE_F_BUFFER);
        else break;
        return 0;
    case SHORT_SEND:
        if (step == 1) {
            receive_completion(0, TEST_BYTES, loan);
            receive_completion(0, TEST_BYTES, loan);
        } else if (step == 2) send_completion(0, 16u);
#if !WF_BENCH_URING_INLINE_SEND
        else if (step == 3) send_completion(0, 16u);
        else if (step == 4) send_completion(0, TEST_BYTES);
#endif
        else break;
        return 0;
    default: require_trace(0, "unknown trace");
    }
    atomic_store_explicit(&finished, 1, memory_order_relaxed);
    return 0;
}
static void run_trace(enum trace_case scenario) {
    memset(&transport, 0, sizeof transport);
    transport.scenario = scenario;
    transport.peers = scenario <= RETRY_CONSUMED ? 2u : 1u;
    option_threads = 1u; option_connections = transport.peers;
    descriptor_capacity = 128u;
    atomic_store(&accepted_total, 0); atomic_store(&closed_total, 0);
    atomic_store(&finished, 0); atomic_store(&failed, 0);
    table = calloc(descriptor_capacity, sizeof *table);
    workers = calloc(1u, sizeof *workers);
    require_trace(table && workers, "worker fixture allocation");
    workers[0].listener = workers[0].wake = -1;
    workers[0].starved_list = calloc(descriptor_capacity, sizeof(int));
    require_trace(workers[0].starved_list != NULL, "starvation fixture allocation");
    for (unsigned peer = 0; peer < transport.peers; peer++) {
        int descriptor = open("/dev/null", O_RDWR);
        require_trace(descriptor >= 0 && (unsigned)descriptor < descriptor_capacity,
                      "transport descriptor allocation");
        transport.descriptors[peer] = descriptor;
    }
    worker_main(&workers[0]);
    take_submissions(&workers[0].ring);
    unsigned available = (uint16_t)(workers[0].buffer_tail - transport.buffer_head);
    unsigned arms = transport.receive_arms[0] + transport.receive_arms[1];
    printf("uring trace state: %s inline=%u available=%u parked=%u receive_sqes=%u "
           "pending_sends=%u sent=%zu\n", trace_names[scenario],
           (unsigned)WF_BENCH_URING_INLINE_SEND, available, workers[0].starved_count,
           arms, transport.sends[0].pending + transport.sends[1].pending,
           transport.sent[0] + transport.sent[1]);
    fflush(stdout);
    require_trace(!atomic_load(&failed), "actual reference rejected trace");
    unsigned expected_arms = scenario == RETURN_BEFORE || scenario == RETURN_AFTER ? 4u :
        scenario == RETRY_CONSUMED ? 3u : scenario == TERMINAL_CANCEL || scenario == NO_RETURN ? 2u : 1u;
    unsigned expected_parked = scenario == NO_RETURN ? 2u : scenario == RETRY_CONSUMED ? 1u : 0u;
    unsigned expected_available = scenario == NO_RETURN || scenario == RETRY_CONSUMED ? 0u : 2u;
    require_trace(arms == expected_arms, "receive retry missing or spun without a new return");
    require_trace(workers[0].starved_count == expected_parked, "incorrect parked set");
    require_trace(available == expected_available, "incorrect provided-buffer publication");
    if (scenario == TERMINAL_EOF) require_trace(atomic_load(&closed_total) == 1, "EOF did not drain then close");
    unsigned expected_pending = scenario == NO_RETURN ? 2u : scenario == RETRY_CONSUMED ? 1u : 0u;
    size_t expected_sent = scenario == NO_RETURN || scenario == TERMINAL_CANCEL ? 0u :
        scenario == RETRY_CONSUMED || scenario == TERMINAL_EOF ? TEST_BYTES : 2u * TEST_BYTES;
    require_trace((unsigned)(transport.sends[0].pending + transport.sends[1].pending) == expected_pending,
                  "incorrect outstanding send count");
    require_trace(transport.sent[0] + transport.sent[1] == expected_sent,
                  "incorrect echo prefix retirement");
    for (unsigned peer = 0; peer < transport.peers; peer++)
        if (table[transport.descriptors[peer]].active) close(transport.descriptors[peer]);
    free(workers[0].starved_list); free(workers[0].loans);
    free(workers[0].buffer_memory); free(workers[0].buffers);
    free(transport.completions);
    free(workers); workers = NULL; free(table); table = NULL;
    printf("uring trace: %s inline=%u PASS\n", trace_names[scenario],
           (unsigned)WF_BENCH_URING_INLINE_SEND);
}
int main(int argc, char **argv) {
    require_trace(argc <= 2, "usage: uring_echo_check [TRACE]");
    unsigned selected = 0;
    for (unsigned at = 0; at < TRACE_COUNT; at++)
        if (argc == 1 || strcmp(argv[1], trace_names[at]) == 0) {
            run_trace((enum trace_case)at);
            selected++;
        }
    require_trace(selected != 0, "unknown trace name");
    return 0;
}
