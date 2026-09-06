/* Suspended nested-frame destruction for the io-model coroutine experiment.
 * The canonical coroutine-check target runs both allocation forms under
 * ASan/UBSan. Retire this probe with that representation comparison. */
#include <fcntl.h>
#define main benchmark_main
#include "epoll_echo.c"
#undef main

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
