// Ordinary-command native controls. No WF ABI or runtime is used here.
#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <thread>
#include <vector>

static void pause(unsigned &rounds) {
    if (++rounds == 4096) {
        rounds = 0;
        std::this_thread::yield();
        return;
    }
#if defined(__x86_64__) || defined(_M_X64)
    __builtin_ia32_pause();
#elif defined(__aarch64__)
    __asm__ __volatile__("yield");
#else
    std::atomic_signal_fence(std::memory_order_seq_cst);
#endif
}

static bool number(const char *text, std::uint64_t &value) {
    const std::size_t length = std::strlen(text);
    if (length == 0 || length > 20) return false;
    value = 0;
    for (std::size_t i = 0; i < length; ++i) {
        if (text[i] < '0' || text[i] > '9') return false;
        const unsigned digit = static_cast<unsigned>(text[i] - '0');
        if (value > (UINT64_MAX - digit) / 10) return false;
        value = value * 10 + digit;
    }
    return true;
}

// Independently rounded recurrence; used only to generate/check expectations.
static std::uint64_t oracle(double real, double imaginary, std::uint64_t limit) {
    volatile double x = 0, y = 0;
    for (std::uint64_t i = 0; i < limit; ++i) {
        volatile double xx = x * x, yy = y * y, magnitude = xx + yy;
        if (magnitude > 4) return i;
        volatile double product = x * y, twice = product + product;
        volatile double difference = xx - yy;
        y = twice + imaginary;
        x = difference + real;
    }
    return limit;
}

// Reuse the previous iteration's squares, preserving strict operation order.
static std::uint64_t escape(double real, double imaginary, std::uint64_t limit) {
    double x = 0, y = 0, xx = 0, yy = 0;
    for (std::uint64_t i = 0; i < limit; ++i) {
        if (xx + yy > 4) return i;
        const double xy = x * y;
        y = (xy + xy) + imaginary;
        x = (xx - yy) + real;
        xx = x * x;
        yy = y * y;
    }
    return limit;
}

struct Points {
    const double *real;
    const double *imaginary;
    std::uint64_t *output;
    std::size_t count;
    std::uint64_t limit;
};

static void render(const Points &points, unsigned lane, unsigned lanes) {
    const std::size_t first = points.count * lane / lanes;
    const std::size_t end = points.count * (lane + 1) / lanes;
    for (std::size_t i = first; i < end; ++i)
        points.output[i] = escape(points.real[i], points.imaginary[i], points.limit);
}

// Persistent static partitions: one release/acquire generation per batch,
// with the caller participating. Startup, idle CPU and shutdown are charged.
// Spin/yield is deliberately visible in CPU results, not hidden as free work.
class StaticTeam {
    unsigned lanes_;
    std::vector<std::thread> threads_;
    std::atomic<unsigned> ready_{0}, remaining_{0};
    std::atomic<std::uint64_t> generation_{0};
    std::atomic<bool> stop_{false};
    Points points_{};

public:
    explicit StaticTeam(unsigned lanes) : lanes_(lanes) {
        for (unsigned lane = 1; lane < lanes_; ++lane) {
            threads_.emplace_back([this, lane] {
                std::uint64_t seen = 0;
                unsigned rounds = 0;
                ready_.fetch_add(1, std::memory_order_release);
                for (;;) {
                    const auto next = generation_.load(std::memory_order_acquire);
                    if (next == seen) {
                        pause(rounds);
                        continue;
                    }
                    if (stop_.load(std::memory_order_relaxed)) return;
                    seen = next;
                    rounds = 0;
                    render(points_, lane, lanes_);
                    // No access to points_ after completion publication.
                    remaining_.fetch_sub(1, std::memory_order_release);
                }
            });
        }
        unsigned rounds = 0;
        while (ready_.load(std::memory_order_acquire) != lanes_ - 1) pause(rounds);
    }

    void run(const Points &points) {
        if (lanes_ == 1) {
            render(points, 0, 1);
            return;
        }
        points_ = points;
        remaining_.store(lanes_ - 1, std::memory_order_relaxed);
        generation_.fetch_add(1, std::memory_order_release);
        render(points_, 0, lanes_);
        unsigned rounds = 0;
        while (remaining_.load(std::memory_order_acquire) != 0) pause(rounds);
    }

    ~StaticTeam() {
        stop_.store(true, std::memory_order_relaxed);
        generation_.fetch_add(1, std::memory_order_release);
        for (auto &thread : threads_) thread.join();
    }
};

static double fraction(std::uint64_t &seed) {
    seed = seed * UINT64_C(6364136223846793005) + UINT64_C(1442695040888963407);
    const std::uint64_t bits = UINT64_C(4607182418800017408) + (seed & UINT64_C(4503599627370495));
    double encoded;
    std::memcpy(&encoded, &bits, sizeof(encoded));
    return encoded - 1;
}

static std::uint64_t checksum(bool reference, unsigned lanes, std::uint64_t shape,
                              std::size_t count, std::uint64_t limit,
                              std::uint64_t repetitions, std::uint64_t seed) {
    StaticTeam team(lanes);
    std::vector<double> real(count, 0), imaginary(count, 0);
    std::uint64_t digest = UINT64_C(14695981039346656037);
    const std::size_t quarter = count / 4;
    for (std::uint64_t batch = 0; batch < repetitions; ++batch) {
        for (std::size_t i = 0; i < count; ++i) {
            const double a = fraction(seed), b = fraction(seed);
            double x = 3, y = 0;
            if (shape == 0) { x = -2 + 3 * a; y = -1.5 + 3 * b; }
            if (shape == 1) { x = -0.75 + (a - 0.5) * 0.03125; y = 0.125 + (b - 0.5) * 0.03125; }
            if (shape == 4 || (shape == 2 && i < quarter) ||
                (shape == 3 && i % 4 == 0) || (shape == 6 && i >= count - quarter)) x = 0;
            real[i] = x;
            imaginary[i] = y;
        }
        std::vector<std::uint64_t> output(count, 0);
        if (reference) {
            for (std::size_t i = 0; i < count; ++i) {
                output[i] = oracle(real[i], imaginary[i], limit);
                if (escape(real[i], imaginary[i], limit) != output[i]) {
                    std::fprintf(stderr, "native Mandelbrot disagrees with rounded oracle\n");
                    std::exit(1);
                }
            }
        } else {
            team.run({real.data(), imaginary.data(), output.data(), count, limit});
        }
        for (const auto value : output)
            digest = digest * UINT64_C(1099511628211) + value;
    }
    return digest;
}

int main(int argc, char **argv) {
    if (argc == 1)
        return oracle(0, 0, 16) == 16 && oracle(2, 0, 16) == 2 &&
               oracle(3, 0, 16) == 1 && oracle(3, 0, 0) == 0 ? 0 : 1;
    const bool reference = std::strcmp(argv[1], "oracle") == 0;
    const bool serial = std::strcmp(argv[1], "serial") == 0;
    if (!reference && !serial && std::strcmp(argv[1], "static") != 0) return 2;
    if (argc != (reference ? 7 : 8)) return 2;
    std::uint64_t args[6]{};
    for (int i = 2; i < argc; ++i) if (!number(argv[i], args[i - 2])) return 2;
    if (args[0] > 6 || args[1] > 1048576 || args[2] > 65536 || args[3] > 4096) return 2;
    std::uint64_t lanes = 1;
    if (!reference && !serial) {
#ifdef _WIN32
        char *setting = nullptr;
        std::size_t length = 0;
        if (_dupenv_s(&setting, &length, "WF_WORKERS") != 0) return 2;
#else
        const char *setting = std::getenv("WF_WORKERS");
#endif
        const bool valid = !setting || (number(setting, lanes) && lanes != 0 && lanes <= 64);
#ifdef _WIN32
        std::free(setting);
#endif
        if (!valid) return 2;
    }
    const auto actual = checksum(reference, static_cast<unsigned>(lanes), args[0],
                                 static_cast<std::size_t>(args[1]), args[2], args[3], args[4]);
    if (reference) {
        std::printf("%llu\n", static_cast<unsigned long long>(actual));
        return 0;
    }
    return actual == args[5] ? 0 : 1;
}
