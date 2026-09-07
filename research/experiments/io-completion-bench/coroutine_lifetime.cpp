/* Suspended nested-frame destruction for the io-model coroutine experiment.
 * The canonical coroutine-check target runs both allocation forms and the
 * chunk-lease lifetime variant under ASan/UBSan. Retire this probe with that
 * representation comparison. */
#include <fcntl.h>
#if defined(WF_BENCH_CHUNK_LEASE)
#include <errno.h>
#include <signal.h>
#include <sys/socket.h>
#include <unistd.h>
static unsigned lease_short_send, lease_send_again, lease_send_errors;
static unsigned lease_short_recv, lease_recv_errors;
/* Count the real handler's syscall outcomes, not fixture/client calls. These
 * wrappers neither replace results nor change requested byte counts. */
static ssize_t lease_counted_send(int fd, const void *bytes, size_t count, int flags) {
    ssize_t result = send(fd, bytes, count, flags);
    if (result > 0 && (size_t)result < count) lease_short_send++;
    if (result < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) lease_send_again++;
    else if (result < 0) lease_send_errors++;
    return result;
}
static ssize_t lease_counted_recv(int fd, void *bytes, size_t count, int flags) {
    ssize_t result = recv(fd, bytes, count, flags);
    if (result > 0 && (size_t)result < count) lease_short_recv++;
    if (result < 0 && errno != EAGAIN && errno != EWOULDBLOCK && errno != EINTR) lease_recv_errors++;
    return result;
}
#define send lease_counted_send
#define recv lease_counted_recv
#endif
#define main benchmark_main
#include "epoll_echo.c"
#undef main
#if defined(WF_BENCH_CHUNK_LEASE)
#undef send
#undef recv

static void lease_require(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "chunk lifetime: %s\n", message); exit(1); }
}
static void lease_nonblocking(int fd) {
    int flags = fcntl(fd, F_GETFL, 0);
    lease_require(flags >= 0 && fcntl(fd, F_SETFL, flags | O_NONBLOCK) == 0, "nonblocking");
}
static void lease_socket_pair(int descriptors[2]) {
    lease_require(socketpair(AF_UNIX, SOCK_STREAM, 0, descriptors) == 0, "socketpair");
    int bytes = 4096;
    lease_require(setsockopt(descriptors[0], SOL_SOCKET, SO_SNDBUF, &bytes, sizeof bytes) == 0,
                  "send buffer");
    lease_nonblocking(descriptors[0]);
    lease_nonblocking(descriptors[1]);
}
static unsigned char lease_byte(size_t offset, unsigned seed) {
    return (unsigned char)(offset * 29u + seed);
}
static void lease_fill(chunk_lease &chunk, unsigned seed) {
    lease_require(chunk.node != nullptr, "acquire");
    chunk.length = TRANSFER_BYTES;
    for (size_t at = 0; at < chunk.length; at++) chunk.node->bytes[at] = lease_byte(at, seed);
}
static connection_task lease_parent(connection *link, chunk_lease chunk) {
    co_return co_await send_response(link, std::move(chunk));
}
static void lease_resume(connection *link) {
    lease_require(link->resume != nullptr, "missing suspended leaf");
    std::coroutine_handle<>::from_address(link->resume).resume();
}
static void lease_drain(connection *link, connection_task::handle root, int peer, unsigned seed) {
    size_t received = 0;
    unsigned char bytes[8192];
    for (unsigned turn = 0; turn < 10000; turn++) {
        ssize_t taken = recv(peer, bytes, sizeof bytes, 0);
        if (taken > 0) {
            lease_require(received + (size_t)taken <= TRANSFER_BYTES, "extra send bytes");
            for (ssize_t at = 0; at < taken; at++)
                lease_require(bytes[at] == lease_byte(received + (size_t)at, seed), "payload changed");
            received += (size_t)taken;
        } else lease_require(taken < 0 && (errno == EAGAIN || errno == EWOULDBLOCK), "drain outcome");
        if (root.done() && received == TRANSFER_BYTES) return;
        if (!root.done()) lease_resume(link);
    }
    lease_require(false, "drain did not finish");
}

int main() {
    signal(SIGPIPE, SIG_IGN);
    /* All three possible loans go to worker zero despite four configured
     * workers: a divided per-worker quota would reject this valid skew. */
    option_connections = 3;
    option_threads = 4;
    worker owner{};
    int first[2], second[2];
    lease_socket_pair(first);
    lease_socket_pair(second);
    connection a{}, b{};
    a.owner = b.owner = &owner;
    a.descriptor = first[0]; b.descriptor = second[0];
    chunk_lease ca(&owner); lease_fill(ca, 17);
    chunk_node *node_a = ca.node;
    auto ta = lease_parent(&a, std::move(ca));
    auto ra = connection_task::handle::from_address(ta.release());
    ra.resume();
    lease_require(!ra.done() && owner.chunk_live == 1 && ca.node == nullptr, "first send must hold a moved loan");
    chunk_lease cb(&owner); lease_fill(cb, 91);
    chunk_node *node_b = cb.node;
    auto tb = lease_parent(&b, std::move(cb));
    auto rb = connection_task::handle::from_address(tb.release());
    rb.resume();
    lease_require(!rb.done() && owner.chunk_live == 2 && node_a != node_b, "distinct simultaneous loans");
    {
        chunk_lease scratch(&owner);
        chunk_node *reusable = scratch.node;
        scratch.reset();
        chunk_lease reused(&owner);
        lease_require(reused.node == reusable && reused.node != node_a && reused.node != node_b,
                      "reuse must exclude suspended borrowers");
        lease_fill(reused, 203);
        for (size_t at = 0; at < TRANSFER_BYTES; at++) {
            lease_require(node_a->bytes[at] == lease_byte(at, 17), "first live prefix overwritten");
            lease_require(node_b->bytes[at] == lease_byte(at, 91), "second live prefix overwritten");
        }
    }
    lease_drain(&a, ra, first[1], 17);
    lease_require(ra.promise().result == 1 && owner.chunk_live == 1, "completed send returns its loan");
    ra.destroy();
    {
        chunk_lease after_send(&owner);
        lease_require(after_send.node == node_a && after_send.node != node_b, "reuse after send completion");
    }
    /* The parent destroys its owned suspended send child, which returns B.
     * This is readiness I/O: no outstanding kernel operation owns its bytes. */
    rb.destroy();
    lease_require(owner.chunk_live == 0, "nested send destruction returns its loan");
    for (int fd : first) close(fd);
    for (int fd : second) close(fd);
    {
        chunk_lease x(&owner), y(&owner), z(&owner), excess(&owner);
        lease_require(x.node && y.node && z.node && !excess.node && owner.chunk_count == 3 && failed.load(),
                      "admission bound must fail explicitly without another allocation");
    }
    failed.store(0);

    int idle[2]; lease_socket_pair(idle);
    connection link{}; link.owner = &owner; link.descriptor = idle[0];
    auto idle_task = connection_main(&link);
    auto idle_root = connection_task::handle::from_address(idle_task.release());
    idle_root.resume();
    lease_require(!idle_root.done() && owner.chunk_live == 0, "idle receive must release before waiting");
    const unsigned char request[] = "owned-prefix";
    lease_require(send(idle[1], request, sizeof request, 0) == (ssize_t)sizeof request, "short request");
    lease_resume(&link);
    unsigned char reply[sizeof request];
    lease_require(recv(idle[1], reply, sizeof reply, 0) == (ssize_t)sizeof reply &&
                  memcmp(reply, request, sizeof reply) == 0 && owner.chunk_live == 0, "short initialized prefix");
    lease_require(shutdown(idle[1], SHUT_WR) == 0, "half close");
    lease_resume(&link);
    lease_require(idle_root.done() && owner.chunk_live == 0, "EOF retires empty receive loan");
    idle_root.destroy(); close(idle[0]); close(idle[1]);

    int broken[2]; lease_socket_pair(broken); close(broken[1]);
    connection error_link{}; error_link.owner = &owner; error_link.descriptor = broken[0];
    chunk_lease error_chunk(&owner); lease_fill(error_chunk, 7);
    auto error_task = lease_parent(&error_link, std::move(error_chunk));
    auto error_root = connection_task::handle::from_address(error_task.release());
    error_root.resume();
    lease_require(error_root.done() && error_root.promise().result == 0 && owner.chunk_live == 0 &&
                  lease_send_errors > 0, "send error retires loan");
    error_root.destroy(); close(broken[0]);

    int listener = socket(AF_INET, SOCK_STREAM, 0);
    lease_require(listener >= 0, "reset listener");
    sockaddr_in address{}; address.sin_family = AF_INET; address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    lease_require(bind(listener, (sockaddr *)&address, sizeof address) == 0 && listen(listener, 1) == 0, "reset bind");
    socklen_t extent = sizeof address;
    lease_require(getsockname(listener, (sockaddr *)&address, &extent) == 0, "reset address");
    int peer = socket(AF_INET, SOCK_STREAM, 0);
    lease_require(peer >= 0 && connect(peer, (sockaddr *)&address, sizeof address) == 0, "reset connect");
    int accepted = accept(listener, nullptr, nullptr);
    lease_require(accepted >= 0, "reset accept");
    lease_nonblocking(accepted);
    connection reset_link{}; reset_link.owner = &owner; reset_link.descriptor = accepted;
    auto reset_task = connection_main(&reset_link);
    auto reset_root = connection_task::handle::from_address(reset_task.release());
    reset_root.resume();
    lease_require(!reset_root.done() && owner.chunk_live == 0, "reset starts in receive wait");
    linger abortive{1, 0};
    lease_require(setsockopt(peer, SOL_SOCKET, SO_LINGER, &abortive, sizeof abortive) == 0 && close(peer) == 0, "reset peer");
    for (unsigned turn = 0; turn < 10000 && !reset_root.done(); turn++) lease_resume(&reset_link);
    lease_require(reset_root.done() && lease_recv_errors > 0 && owner.chunk_live == 0, "reset retires receive loan");
    reset_root.destroy(); close(accepted); close(listener);

    lease_require(lease_short_send > 0 && lease_send_again > 0 && lease_short_recv > 0,
                  "actual short send/receive and send EAGAIN required");
    lease_require(owner.chunk_count == 3 && owner.chunk_peak == 3 &&
                  owner.chunk_acquires == owner.chunk_returns && frame_allocations.load() == frame_frees.load(),
                  "final pool/frame conservation");
    printf("chunk lifetime: PASS nodes=3 peak=3 short_send=%u send_eagain=%u short_recv=%u send_errors=%u recv_errors=%u frames=%llu\n",
           lease_short_send, lease_send_again, lease_short_recv, lease_send_errors, lease_recv_errors,
           (unsigned long long)frame_allocations.load());
    chunk_pool_destroy(&owner);
    return 0;
}
#else

int main() {
    for (unsigned iteration = 0; iteration < 1024; iteration++) {
        int descriptors[2];
        if (socketpair(AF_UNIX, SOCK_STREAM, 0, descriptors) != 0) return 1;
        int flags = fcntl(descriptors[0], F_GETFL, 0);
        if (flags < 0 || fcntl(descriptors[0], F_SETFL, flags | O_NONBLOCK) != 0) return 2;
        worker owner{};
        connection link{};
        unsigned char bytes[COMPUTE_BYTES]{};
        link.descriptor = descriptors[0];
        link.owner = &owner;
        link.pending = bytes;
        auto task = connection_main(&link);
        auto root = std::coroutine_handle<>::from_address(task.release());
        root.resume();
        if (root.done() || link.resume == nullptr || owner.waits != 1) return 3;
        root.destroy();
        close(descriptors[0]);
        close(descriptors[1]);
    }
    uint64_t expected = 2048;
#if defined(WF_BENCH_CORO_ELIDE)
    expected = 1024;
#endif
    if (frame_allocations.load() != expected || frame_frees.load() != expected) return 4;
    printf("destroy nested suspended frame: PASS allocations=%llu frees=%llu bytes=%llu\n",
           (unsigned long long)frame_allocations.load(),
           (unsigned long long)frame_frees.load(),
           (unsigned long long)frame_bytes.load());
    return 0;
}
#endif
