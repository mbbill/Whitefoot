/* Sequential C++20 handlers for the owner-local representation comparison.
 * Included by epoll_echo.c only with WF_BENCH_COROUTINE. Retire with the
 * io-model coroutine experiment. The owner alone resumes a leaf; a root owns
 * every nested task and is destroyed only after resumption ends. Elision
 * relies on child destruction before the containing frame is freed, including
 * failure cleanup; it does not promise that every child finishes normally. */
#if defined(WF_BENCH_OBSERVE)
static std::atomic<uint64_t> frame_allocations;
static std::atomic<uint64_t> frame_frees;
static std::atomic<uint64_t> frame_bytes;
#endif

#if defined(WF_BENCH_CORO_ELIDE)
#define WF_CORO_FRAME_HINT [[clang::coro_await_elidable]]
#else
#define WF_CORO_FRAME_HINT
#endif
struct WF_CORO_FRAME_HINT connection_task {
    struct promise_type;
    using handle = std::coroutine_handle<promise_type>;
    struct promise_type {
        std::coroutine_handle<> continuation = std::noop_coroutine();
        int result = 0;
        static void *operator new(size_t bytes) noexcept {
            void *memory = malloc(bytes);
#if defined(WF_BENCH_OBSERVE)
            if (memory != NULL) {
                frame_allocations.fetch_add(1, memory_order_relaxed);
                frame_bytes.fetch_add(bytes, memory_order_relaxed);
            }
#endif
            return memory;
        }
        static void operator delete(void *memory, size_t) noexcept {
#if defined(WF_BENCH_OBSERVE)
            frame_frees.fetch_add(1, memory_order_relaxed);
#endif
            free(memory);
        }
        static connection_task get_return_object_on_allocation_failure() noexcept {
            return {nullptr};
        }
        connection_task get_return_object() noexcept {
            return {handle::from_promise(*this)};
        }
        std::suspend_always initial_suspend() noexcept { return {}; }
        struct final_transfer {
            bool await_ready() noexcept { return false; }
            std::coroutine_handle<> await_suspend(handle self) noexcept {
                return self.promise().continuation;
            }
            void await_resume() noexcept {}
        };
        final_transfer final_suspend() noexcept { return {}; }
        void return_value(int value) noexcept { result = value; }
        void unhandled_exception() noexcept { abort(); }
    };
    handle frame;
    connection_task(handle value) noexcept : frame(value) {}
    connection_task(const connection_task &) = delete;
    connection_task(connection_task &&other) noexcept : frame(std::exchange(other.frame, {})) {}
    ~connection_task() { if (frame) frame.destroy(); }
    void *release() noexcept { return std::exchange(frame, {}).address(); }
    bool await_ready() noexcept { return !frame; }
    std::coroutine_handle<> await_suspend(std::coroutine_handle<> parent) noexcept {
        frame.promise().continuation = parent;
        return frame;
    }
    int await_resume() noexcept {
        if (!frame) { mark_failed(); return 0; }
        return frame.promise().result;
    }
};

struct connection_pause {
    struct connection *link;
    bool queued;
    bool await_ready() noexcept { return false; }
    void await_suspend(std::coroutine_handle<> self) noexcept {
        link->resume = self.address();
#if defined(WF_BENCH_QUANTUM)
        if (queued) enqueue(link->owner, link->descriptor);
#endif
#if defined(WF_BENCH_OBSERVE)
        if (queued) link->owner->yields++;
        else link->owner->waits++;
#endif
    }
    void await_resume() noexcept {}
};

#if defined(WF_BENCH_COMPUTE)
static connection_task receive_request(struct connection *link) {
    unsigned received = 0;
    while (received < COMPUTE_BYTES) {
        ssize_t taken = recv(link->descriptor, link->pending + received,
                             COMPUTE_BYTES - received, 0);
        if (taken > 0) received += (unsigned)taken;
        else if (taken < 0 && errno == EINTR) continue;
        else if (taken < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
            co_await connection_pause{link, false};
        } else {
            if (taken == 0 && received != 0) mark_failed();
            co_return 0;
        }
    }
    co_return 1;
}
#endif

static connection_task send_response(struct connection *link, unsigned char *source, size_t length) {
    size_t offset = 0;
    while (offset < length) {
        ssize_t moved = send(link->descriptor, source + offset, length - offset, MSG_NOSIGNAL);
        if (moved > 0) offset += (size_t)moved;
        else if (moved < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
#if defined(WF_BENCH_OBSERVE)
            link->owner->send_waits++;
#endif
            if (source != link->pending) {
                length -= offset;
                memcpy(link->pending, source + offset, length);
                source = link->pending;
                offset = 0;
            }
            co_await connection_pause{link, false};
        } else co_return 0;
    }
    co_return 1;
}

static connection_task connection_main(struct connection *link) {
    for (;;) {
#if defined(WF_BENCH_COMPUTE)
        if (!(co_await receive_request(link))) break;
#if defined(WF_BENCH_QUANTUM)
        uint64_t value = compute_decode(link->pending);
        uint64_t remaining = compute_decode(link->pending + 8);
        if (remaining > COMPUTE_MAX_ROUNDS) { mark_failed(); break; }
        while (remaining != 0) {
            co_await connection_pause{link, true};
            uint64_t steps = remaining < option_quantum ? remaining : option_quantum;
            value = compute_churn(value, steps);
            remaining -= steps;
        }
        compute_encode(link->pending, value);
#else
        if (!compute_response(link->pending)) { mark_failed(); break; }
#endif
        if (!(co_await send_response(link, link->pending, COMPUTE_BYTES))) break;
#if defined(WF_BENCH_QUANTUM)
        if (++link->owner->replies == 8u) co_await connection_pause{link, true};
#endif
#else
        ssize_t taken = recv(link->descriptor, WF_BENCH_RECEIVE_BUFFER(link->owner, link), TRANSFER_BYTES, 0);
        if (taken > 0) {
            if (!(co_await send_response(link, WF_BENCH_RECEIVE_BUFFER(link->owner, link), (size_t)taken))) break;
        } else if (taken < 0 && errno == EINTR) continue;
        else if (taken < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
            co_await connection_pause{link, false};
        } else break;
#endif
    }
    co_return 1;
}

static void service(struct worker *worker, int descriptor) {
    struct connection *link = &table[descriptor];
#if defined(WF_BENCH_QUANTUM)
    worker->replies = 0;
#endif
#if defined(WF_BENCH_OBSERVE)
    worker->resumes++;
#endif
    std::coroutine_handle<>::from_address(link->resume).resume();
    auto root = std::coroutine_handle<>::from_address(link->saved_sp);
    if (root.done()) {
        root.destroy();
        link->saved_sp = NULL;
        link->resume = NULL;
        close_connection(worker, descriptor);
    }
}
