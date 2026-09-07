/* Actual completion records borrowed by nested stackless continuations.
 * Built as C for the existing C11 runtime and C++20 for sequential coroutines.
 * The registration list is a test coordinator, not a scheduler candidate.
 * Owned by io-model/SCHEDULER-EXPERIMENT.md; replace with compiler-generated
 * cases when the general continuation lowering can express this protocol. */
#include <stddef.h>
#include <stdint.h>

enum {
    PROBE_RECORD_BYTES = 160, PROBE_RECORD_ALIGN = 8,
    PROBE_ITERATIONS = 128, PROBE_GROUPS = 16, PROBE_WIDTH = 8
};
typedef struct probe_gate { unsigned open; } probe_gate;
typedef struct probe_waiter {
    void *record;
    void *continuation;
    struct probe_waiter *pending_next;
    struct probe_waiter *ready_next;
    unsigned phase;
} probe_waiter;

#ifdef __cplusplus
extern "C" {
#endif
int probe_done(const void *record);
int probe_arm(probe_waiter *waiter, void *record, void *continuation,
              probe_gate *gate, unsigned ordering);
void *probe_take_ready(void);
void probe_open_gate(probe_gate *gate);
void probe_wait_gate(probe_gate *gate);
void probe_start(void);
void probe_stop(void);
void probe_idle(void);
void probe_report(int required_route);
#ifdef __cplusplus
}
#endif

#ifndef __cplusplus
#include "completion/contract.h"
#include "completion/bridge.h"
#include "sched/entry.h"
#include "sched/prim.h"
#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

_Static_assert(PROBE_RECORD_BYTES == WF_COMPLETION_RECORD_BYTES, "record size");
_Static_assert(PROBE_RECORD_ALIGN == WF_COMPLETION_RECORD_ALIGN, "record alignment");

static pthread_mutex_t probe_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t probe_changed = PTHREAD_COND_INITIALIZER;
static probe_waiter *probe_pending;
static probe_waiter *probe_ready_head;
static probe_waiter *probe_ready_tail;
static pthread_t probe_progress_thread;
static _Atomic unsigned probe_stopping;
static uint64_t probe_registered;
static uint64_t probe_dequeued;
static uint64_t probe_before_arm;
static uint64_t probe_during_arm;
static uint64_t probe_routes[5];
#if defined(PROBE_WF_GENERATED)
static uint64_t probe_socket_routes[3][5];
#endif

int wf__sched_host_epoch(uint64_t *epoch);
int wf__sched_host_park(uint64_t observed);
int wf__sched_host_wake(void);
int wf__sched_target_progress(struct wf_sched_core *core);

int probe_done(const void *record) {
    const wf_completion_record *held = record;
    return wf_prim_load_u(&held->sched.state, WF_PRIM_ACQUIRE) == WF_SCHED_DONE;
}

/* Only bridge.c is compiled with its wf_sched_complete reference redirected
 * here. Every publication still invokes the real core publisher exactly once.
 * No runtime record layout or generated-code entry point changes. */
void wf_probe_publish(wf_sched_core *core, wf_sched_record *record) {
    probe_waiter **link;
    probe_waiter *waiter;
    const wf_completion_record *completion = (const wf_completion_record *)record;
    pthread_mutex_lock(&probe_lock);
    assert(completion->route < 5);
    ++probe_routes[completion->route];
#if defined(PROBE_WF_GENERATED)
    if (completion->request.kind == WF_FILE_SOCKET_ACCEPT) ++probe_socket_routes[0][completion->route];
    if (completion->request.kind == WF_FILE_SOCKET_CONNECT) ++probe_socket_routes[1][completion->route];
    if (completion->request.kind == WF_FILE_SOCKET_RECEIVE) ++probe_socket_routes[2][completion->route];
#endif
    for (link = &probe_pending; *link && (*link)->record != record;
         link = &(*link)->pending_next) {}
    waiter = *link;
    if (waiter) {
        assert(waiter->phase == 1 && record->waiter == NULL);
        *link = waiter->pending_next;
        waiter->pending_next = NULL;
    }
    wf_sched_complete(core, record);
    /* Never inspect the completion record after DONE. A registered waiter's
     * frame remains owned by this queue until the sole resumer removes it. */
    if (waiter) {
        waiter->phase = 2;
        waiter->ready_next = NULL;
        if (probe_ready_tail) probe_ready_tail->ready_next = waiter;
        else probe_ready_head = waiter;
        probe_ready_tail = waiter;
    }
    pthread_cond_broadcast(&probe_changed);
    pthread_mutex_unlock(&probe_lock);
    assert(wf__sched_host_wake());
}

int probe_arm(probe_waiter *waiter, void *record, void *continuation,
              probe_gate *gate, unsigned ordering) {
    assert(wf__sched_host_wake());
    pthread_mutex_lock(&probe_lock);
    if (ordering == 1) {
        /* Producer and engine finish while await_suspend has not registered. */
        gate->open = 1;
        pthread_cond_broadcast(&probe_changed);
        while (!probe_done(record)) pthread_cond_wait(&probe_changed, &probe_lock);
        ++probe_before_arm;
    }
    if (probe_done(record)) {
        pthread_mutex_unlock(&probe_lock);
        return 0;
    }
    assert(waiter->phase == 0);
    assert(((wf_completion_record *)record)->sched.waiter == NULL);
    waiter->record = record;
    waiter->continuation = continuation;
    waiter->pending_next = probe_pending;
    waiter->phase = 1;
    probe_pending = waiter;
    ++probe_registered;
    pthread_mutex_unlock(&probe_lock);
    /* A native ring may still have a deferred SQE; wake its progress thread. */
    assert(wf__sched_host_wake());
    if (ordering == 2) {
        pthread_mutex_lock(&probe_lock);
        gate->open = 1;
        pthread_cond_broadcast(&probe_changed);
        while (waiter->phase != 2) pthread_cond_wait(&probe_changed, &probe_lock);
        ++probe_during_arm;
        pthread_mutex_unlock(&probe_lock);
    }
    return 1;
}

void *probe_take_ready(void) {
    probe_waiter *waiter;
    void *continuation;
    pthread_mutex_lock(&probe_lock);
    while (!probe_ready_head) pthread_cond_wait(&probe_changed, &probe_lock);
    waiter = probe_ready_head;
    assert(waiter->phase == 2);
    probe_ready_head = waiter->ready_next;
    if (!probe_ready_head) probe_ready_tail = NULL;
    continuation = waiter->continuation;
    waiter->ready_next = NULL;
    waiter->phase = 3;
    ++probe_dequeued;
    pthread_mutex_unlock(&probe_lock);
    return continuation;
}

void probe_open_gate(probe_gate *gate) {
    pthread_mutex_lock(&probe_lock);
    gate->open = 1;
    pthread_cond_broadcast(&probe_changed);
    pthread_mutex_unlock(&probe_lock);
}

void probe_wait_gate(probe_gate *gate) {
    pthread_mutex_lock(&probe_lock);
    while (!gate->open) pthread_cond_wait(&probe_changed, &probe_lock);
    pthread_mutex_unlock(&probe_lock);
}

static void *probe_progress(void *unused) {
    (void)unused;
    while (!atomic_load_explicit(&probe_stopping, memory_order_acquire)) {
        uint64_t epoch;
        assert(wf__sched_host_epoch(&epoch));
        (void)wf__sched_target_progress(NULL);
        if (atomic_load_explicit(&probe_stopping, memory_order_acquire)) break;
        assert(wf__sched_host_park(epoch));
    }
    return NULL;
}

void probe_start(void) {
    wf_completion_record record;
    unsigned char byte = 0;
    int64_t value;
    int error;
    FILE *input = tmpfile();
    assert(input != NULL && fputc(0x5a, input) == 0x5a && fflush(input) == 0);
    /* Initialize with a valid positioned read, which both routes support.
     * An invalid descriptor would be deliberately refused by the native
     * adapter and would contaminate the no-helper qualification below.
     * The ordinary join itself drives any deferred native submission. */
    wf__completion_file_pread_submit(fileno(input), &byte, 1, 0, &record);
    wf__completion_file_join(&record, &value, &error);
    assert(value == 1 && error == 0 && byte == 0x5a);
    assert(fclose(input) == 0);
    assert(pthread_create(&probe_progress_thread, NULL, probe_progress, NULL) == 0);
}

void probe_idle(void) {
    pthread_mutex_lock(&probe_lock);
    assert(probe_pending == NULL && probe_ready_head == NULL && probe_ready_tail == NULL);
    assert(probe_registered == probe_dequeued);
    pthread_mutex_unlock(&probe_lock);
}

void probe_stop(void) {
    probe_idle();
    atomic_store_explicit(&probe_stopping, 1, memory_order_release);
    assert(wf__sched_host_wake());
    assert(pthread_join(probe_progress_thread, NULL) == 0);
}

void probe_report(int required_route) {
    assert(probe_before_arm == PROBE_ITERATIONS && probe_during_arm == PROBE_ITERATIONS);
    assert(probe_registered >= PROBE_ITERATIONS * 2 + PROBE_GROUPS * PROBE_WIDTH);
    assert(probe_registered == probe_dequeued);
    const unsigned gated = PROBE_ITERATIONS * 3 + PROBE_GROUPS * PROBE_WIDTH;
    if (required_route == 1 &&
        (probe_routes[WF_COMPLETION_ROUTE_LINUX_IO_URING] < gated ||
         probe_routes[WF_COMPLETION_ROUTE_FILE_ADAPTER] != 0)) {
        fprintf(stderr, "coroutine completion: native io_uring route requirement failed: ring=%llu helper=%llu\n",
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_LINUX_IO_URING],
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_FILE_ADAPTER]);
        abort();
    }
    if (required_route == 2 &&
        (probe_routes[WF_COMPLETION_ROUTE_FILE_ADAPTER] < gated ||
         probe_routes[WF_COMPLETION_ROUTE_LINUX_IO_URING] != 0)) {
        fprintf(stderr, "coroutine completion: helper route requirement failed: ring=%llu helper=%llu\n",
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_LINUX_IO_URING],
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_FILE_ADAPTER]);
        abort();
    }
    printf("completion continuation: PASS registered=%llu dequeued=%llu before=%llu during=%llu helper=%llu uring=%llu inline=%llu\n",
           (unsigned long long)probe_registered, (unsigned long long)probe_dequeued,
           (unsigned long long)probe_before_arm, (unsigned long long)probe_during_arm,
           (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_FILE_ADAPTER],
           (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_LINUX_IO_URING],
           (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_INLINE]);
}

#if defined(PROBE_WF_GENERATED)
/* Experimental LLVM host for compiler-generated bodies. This uses the same
 * publication coordinator as the nested C++ fixture, not a production
 * scheduler. Source code performs its own first system operation; the host
 * does not open a bootstrap file or manufacture an ambient source input. */
_Static_assert(sizeof(probe_waiter) == 40, "continuation waiter size");
_Static_assert(_Alignof(probe_waiter) == 8, "continuation waiter alignment");
void wf__continuation_resume(void *frame);
int wf__continuation_finished(void *frame);

int wf__continuation_record_done(void *record) {
    return probe_done(record);
}

void wf__continuation_prepare(void *storage, void *record) {
    probe_waiter *waiter = storage;
    memset(waiter, 0, sizeof(*waiter));
    waiter->record = record;
}

int wf__continuation_arm(void *storage, void *frame) {
    probe_waiter *waiter = storage;
    return probe_arm(waiter, waiter->record, frame, NULL, 0);
}

/* Observe only after resume returns. Holding the coordinator lock keeps a
 * pending node and its record live; publication removes the node before DONE.
 * The native peer withholds connect/input until these suspension witnesses. */
static unsigned probe_observe_socket_waits(unsigned seen) {
    if (!getenv("WF_CONTINUATION_TRACE_SOCKET")) return seen;
    pthread_mutex_lock(&probe_lock);
    for (probe_waiter *waiter = probe_pending; waiter; waiter = waiter->pending_next) {
        const wf_completion_record *record = waiter->record;
        if (!(seen & 1) && record->request.kind == WF_FILE_SOCKET_ACCEPT) {
            fputs("WF continuation host: accept suspended\n", stderr);
            seen |= 1;
        }
        if (!(seen & 2) && record->request.kind == WF_FILE_SOCKET_RECEIVE) {
            fputs("WF continuation host: receive suspended\n", stderr);
            seen |= 2;
        }
    }
    pthread_mutex_unlock(&probe_lock);
    return seen;
}

void wf__continuation_run(void *frame) {
    static unsigned active;
    unsigned socket_waits = 0;
    assert(!active);
    active = 1;
    atomic_store_explicit(&probe_stopping, 0, memory_order_release);
    wf__continuation_resume(frame);
    if (!wf__continuation_finished(frame)) {
        if (getenv("WF_CONTINUATION_OBSERVE")) {
            fputs("WF continuation host: suspended\n", stderr);
        }
        socket_waits = probe_observe_socket_waits(socket_waits);
        assert(pthread_create(&probe_progress_thread, NULL, probe_progress, NULL) == 0);
        do {
            wf__continuation_resume(probe_take_ready());
            socket_waits = probe_observe_socket_waits(socket_waits);
        } while (!wf__continuation_finished(frame));
        probe_stop();
    }
    probe_idle();
    if (getenv("WF_CONTINUATION_OBSERVE")) {
        fprintf(stderr, "WF continuation host: registered=%llu dequeued=%llu helper=%llu uring=%llu inline=%llu\n",
                (unsigned long long)probe_registered, (unsigned long long)probe_dequeued,
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_FILE_ADAPTER],
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_LINUX_IO_URING],
                (unsigned long long)probe_routes[WF_COMPLETION_ROUTE_INLINE]);
    }
    if (getenv("WF_CONTINUATION_REPORT_SOCKET_ROUTES")) {
        fprintf(stderr, "WF continuation host: accept_ring=%llu accept_helper=%llu connect_ring=%llu connect_helper=%llu receive_ring=%llu receive_helper=%llu\n",
                (unsigned long long)probe_socket_routes[0][WF_COMPLETION_ROUTE_LINUX_IO_URING],
                (unsigned long long)probe_socket_routes[0][WF_COMPLETION_ROUTE_FILE_ADAPTER],
                (unsigned long long)probe_socket_routes[1][WF_COMPLETION_ROUTE_LINUX_IO_URING],
                (unsigned long long)probe_socket_routes[1][WF_COMPLETION_ROUTE_FILE_ADAPTER],
                (unsigned long long)probe_socket_routes[2][WF_COMPLETION_ROUTE_LINUX_IO_URING],
                (unsigned long long)probe_socket_routes[2][WF_COMPLETION_ROUTE_FILE_ADAPTER]);
    }
    active = 0;
}
#endif
#else
#include "completion/bridge.h"
#include <assert.h>
#include <coroutine>
#include <errno.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>
#include <utility>
#include <vector>

static uint64_t frame_allocations;
static uint64_t frame_frees;
static uint64_t frame_bytes;
#if defined(PROBE_ELIDE)
#define PROBE_FRAME_HINT [[clang::coro_await_elidable]]
#else
#define PROBE_FRAME_HINT
#endif

struct PROBE_FRAME_HINT task {
    struct promise_type;
    using handle = std::coroutine_handle<promise_type>;
    struct promise_type {
        std::coroutine_handle<> parent = std::noop_coroutine();
        size_t result = 0;
        static void *operator new(size_t bytes) noexcept {
            void *p = malloc(bytes);
            if (p) { ++frame_allocations; frame_bytes += bytes; }
            return p;
        }
        static void operator delete(void *p, size_t) noexcept {
            ++frame_frees;
            free(p);
        }
        static task get_return_object_on_allocation_failure() noexcept { abort(); }
        task get_return_object() noexcept { return {handle::from_promise(*this)}; }
        std::suspend_always initial_suspend() noexcept { return {}; }
        struct final_transfer {
            bool await_ready() noexcept { return false; }
            std::coroutine_handle<> await_suspend(handle h) noexcept { return h.promise().parent; }
            void await_resume() noexcept {}
        };
        final_transfer final_suspend() noexcept { return {}; }
        void return_value(size_t value) noexcept { result = value; }
        void unhandled_exception() noexcept { abort(); }
    };
    handle frame;
    task(handle h) noexcept : frame(h) {}
    task(const task &) = delete;
    task(task &&other) noexcept : frame(std::exchange(other.frame, {})) {}
    ~task() { if (frame) { assert(frame.done()); frame.destroy(); } }
    bool await_ready() noexcept { return false; }
    std::coroutine_handle<> await_suspend(std::coroutine_handle<> parent) noexcept {
        frame.promise().parent = parent;
        return frame;
    }
    size_t await_resume() noexcept { assert(frame.done()); return frame.promise().result; }
};

struct read_wait {
    alignas(PROBE_RECORD_ALIGN) unsigned char record[PROBE_RECORD_BYTES];
    probe_waiter waiter{};
    probe_gate *gate;
    unsigned ordering;
    size_t requested;
    read_wait(int fd, unsigned char *buffer, size_t count, probe_gate *g, unsigned order)
        : gate(g), ordering(order), requested(count) {
        wf__completion_socket_receive_submit(fd, buffer, count, record);
    }
    read_wait(const read_wait &) = delete;
    bool await_ready() noexcept { return probe_done(record); }
    bool await_suspend(std::coroutine_handle<> h) noexcept {
        return probe_arm(&waiter, record, h.address(), gate, ordering) != 0;
    }
    size_t await_resume() noexcept {
        int64_t value;
        int error;
        wf__completion_file_join(record, &value, &error);
        assert(error == 0 && value > 0 && (uint64_t)value <= requested);
        return (size_t)value;
    }
};

struct state {
    probe_gate gate{};
    size_t wanted;
    unsigned ordering;
    unsigned seed;
    bool cancel = false;
    unsigned destroyed = 0;
    unsigned char *address = nullptr;
    size_t received = 0;
};
static unsigned char pattern(size_t at, unsigned seed) {
    return (unsigned char)((at * 17 + seed) % 251);
}

static task receive_child(int fd, unsigned char *buffer, state *s) {
    size_t offset = 0;
    while (offset < s->wanted) {
        size_t count = s->wanted - offset;
        if (count > 7) count = 7;
        offset += co_await read_wait(fd, buffer + offset, count, &s->gate,
                                     offset == 0 ? s->ordering : 0);
        if (s->cancel) break;
    }
    co_return offset;
}

/* Preserve an ordinary caller boundary so the lifetime test observes root
 * allocation separately from the optional nested-frame elision. */
[[gnu::noinline]] static task parent(int fd, state *s) {
    unsigned char buffer[128]{};
    struct guard {
        state *s;
        ~guard() { ++s->destroyed; s->address = nullptr; }
    } held{s};
    s->address = buffer;
    size_t count = co_await receive_child(fd, buffer, s);
    assert(count <= s->wanted && s->address == buffer);
    for (size_t at = 0; at < count; ++at) assert(buffer[at] == pattern(at, s->seed));
    if (!s->cancel) assert(count == s->wanted);
    s->received = count;
    co_return count;
}

struct producer_argument { int fd; state *s; };
static void *produce(void *argument) {
    auto *a = (producer_argument *)argument;
    probe_wait_gate(&a->s->gate);
    unsigned char bytes[128];
    for (size_t at = 0; at < a->s->wanted; ++at) bytes[at] = pattern(at, a->s->seed);
    size_t offset = 0;
    while (offset < a->s->wanted) {
        ssize_t n = send(a->fd, bytes + offset, a->s->wanted - offset, 0);
        if (n < 0 && errno == EINTR) continue;
        assert(n > 0);
        offset += (size_t)n;
    }
    return nullptr;
}

int main(int argc, char **argv) {
    assert(argc == 1 || (argc == 2 &&
           (strcmp(argv[1], "--require-ring") == 0 || strcmp(argv[1], "--require-helper") == 0)));
    const int required_route = argc == 1 ? 0 : strcmp(argv[1], "--require-ring") == 0 ? 1 : 2;
    assert(setenv("WF_IO_HELPERS", "4", 1) == 0);
    probe_start();
    unsigned cancelled = 0;
    unsigned cases = 0;
    for (unsigned iteration = 0; iteration < PROBE_ITERATIONS; ++iteration) {
        for (unsigned ordering = 0; ordering < 4; ++ordering) {
            int fd[2];
            assert(socketpair(AF_UNIX, SOCK_STREAM, 0, fd) == 0);
            state s{};
            s.wanted = 1 + iteration % 127;
            s.ordering = ordering;
            s.seed = iteration + ordering;
            producer_argument argument{fd[1], &s};
            pthread_t producer{};
            if (ordering == 0) {
                probe_open_gate(&s.gate);
                produce(&argument);
            } else {
                assert(pthread_create(&producer, nullptr, produce, &argument) == 0);
            }
            {
                auto root = parent(fd[0], &s);
                root.frame.resume();
                if (ordering <= 1) assert(root.frame.done());
                if (ordering >= 2) {
                    assert(!root.frame.done() && s.destroyed == 0 && s.address != nullptr);
                    if (iteration % 2) { s.cancel = true; ++cancelled; }
                }
                if (ordering == 3) probe_open_gate(&s.gate);
                while (!root.frame.done()) {
                    assert(s.destroyed == 0 && s.address != nullptr);
                    auto ready = std::coroutine_handle<>::from_address(probe_take_ready());
                    ready.resume();
                }
                assert(s.destroyed == 1 && s.address == nullptr && s.received > 0);
                if (s.cancel) assert(s.received <= 7);
            }
            if (ordering != 0) assert(pthread_join(producer, nullptr) == 0);
            assert(close(fd[0]) == 0 && close(fd[1]) == 0);
            probe_idle();
            assert(frame_allocations == frame_frees);
            ++cases;
        }
    }
    /* Eight roots simultaneously retain distinct parent buffers, exceeding
     * the four helper threads on the fallback route. Every producer remains
     * gated until all roots have suspended. Completion order is unrestricted. */
    for (unsigned group = 0; group < PROBE_GROUPS; ++group) {
        int descriptors[PROBE_WIDTH][2];
        state states[PROBE_WIDTH]{};
        producer_argument arguments[PROBE_WIDTH];
        pthread_t producers[PROBE_WIDTH];
        unsigned char *addresses[PROBE_WIDTH];
        {
            std::vector<task> roots;
            roots.reserve(PROBE_WIDTH);
            for (unsigned i = 0; i < PROBE_WIDTH; ++i) {
                assert(socketpair(AF_UNIX, SOCK_STREAM, 0, descriptors[i]) == 0);
                states[i].wanted = 17 + (group * 7 + i) % 111;
                states[i].ordering = 3;
                states[i].seed = group + i;
                arguments[i] = {descriptors[i][1], &states[i]};
                assert(pthread_create(&producers[i], nullptr, produce, &arguments[i]) == 0);
                roots.push_back(parent(descriptors[i][0], &states[i]));
                roots.back().frame.resume();
                assert(!roots.back().frame.done() && states[i].destroyed == 0);
                addresses[i] = states[i].address;
                assert(addresses[i] != nullptr);
                for (unsigned j = 0; j < i; ++j) assert(addresses[i] != addresses[j]);
                if (i % 2) { states[i].cancel = true; ++cancelled; }
            }
            for (unsigned i = 0; i < PROBE_WIDTH; ++i) probe_open_gate(&states[i].gate);
            for (;;) {
                unsigned finished = 0;
                for (unsigned i = 0; i < PROBE_WIDTH; ++i) {
                    if (roots[i].frame.done()) {
                        assert(states[i].destroyed == 1 && states[i].address == nullptr);
                        ++finished;
                    } else {
                        assert(states[i].destroyed == 0 && states[i].address == addresses[i]);
                    }
                }
                if (finished == PROBE_WIDTH) break;
                std::coroutine_handle<>::from_address(probe_take_ready()).resume();
            }
            for (unsigned i = 0; i < PROBE_WIDTH; ++i) {
                assert(states[i].received > 0);
                if (states[i].cancel) assert(states[i].received <= 7);
                ++cases;
            }
        }
        for (unsigned i = 0; i < PROBE_WIDTH; ++i) {
            assert(pthread_join(producers[i], nullptr) == 0);
            assert(close(descriptors[i][0]) == 0 && close(descriptors[i][1]) == 0);
        }
        probe_idle();
        assert(frame_allocations == frame_frees);
    }
    probe_stop();
    probe_report(required_route);
    assert(cases == PROBE_ITERATIONS * 4 + PROBE_GROUPS * PROBE_WIDTH);
    assert(cancelled == PROBE_ITERATIONS + PROBE_GROUPS * PROBE_WIDTH / 2);
    assert(frame_allocations > 0 && frame_allocations <= cases * 2);
#if defined(PROBE_ELIDE)
    assert(frame_allocations <= cases);
#endif
    printf("nested completion-loan drain: PASS cases=%u cancelled=%u allocations=%llu frees=%llu frame_bytes=%llu\n",
           cases, cancelled, (unsigned long long)frame_allocations,
           (unsigned long long)frame_frees, (unsigned long long)frame_bytes);
}
#endif
