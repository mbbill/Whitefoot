#include "records_scheduler.h"
#include "records_native.h"
#include <algorithm>
#include <atomic>
#include <cerrno>
#include <cinttypes>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <mutex>
#include <set>
#include <thread>
#include <vector>
#include <sys/resource.h>
#include <time.h>
#include <unistd.h>

extern "C" int wf__floor_run(int, char **);
static void require(bool ok, const char *why) {
    if (!ok) { std::fprintf(stderr, "record scheduler: %s\n", why); std::exit(1); }
}
static uint64_t number(const char *text) {
    require(text && *text, "missing integer");
    for (const char *p = text; *p; ++p) require(*p >= '0' && *p <= '9', "invalid integer");
    errno = 0;
    char *end;
    uint64_t value = std::strtoull(text, &end, 10);
    require(!errno && !*end, "integer overflow");
    return value;
}
static uint64_t now() {
    timespec time;
    require(clock_gettime(CLOCK_MONOTONIC_RAW, &time) == 0, "clock");
    return uint64_t(time.tv_sec) * 1000000000 + uint64_t(time.tv_nsec);
}
static uint64_t cpu_us(timeval v) { return uint64_t(v.tv_sec) * 1000000 + uint64_t(v.tv_usec); }
static uint32_t next_random(uint32_t &state) {
    state ^= state << 13; state ^= state >> 17; state ^= state << 5; return state;
}
struct Input {
    std::vector<uint8_t> bytes, saved;
    std::vector<uint64_t> offsets, saved_offsets, expected, output;
    Input(size_t count, size_t limit, const char *shape, uint32_t seed) {
        require(count <= 1048576 && limit <= 1048576 && (!count || limit <= 16777216 / count), "input domain");
        int kind = !std::strcmp(shape, "ascii") ? 0 : !std::strcmp(shape, "unicode") ? 1 :
                   !std::strcmp(shape, "error-first") ? 2 : !std::strcmp(shape, "error-last") ? 3 :
                   !std::strcmp(shape, "skew") ? 4 : -1;
        require(kind >= 0, "input shape");
        bytes.reserve(std::max(size_t(1), count * limit));
        offsets.push_back(0);
        for (size_t j = 0; j < count; ++j) {
            size_t length = limit ? next_random(seed) % (limit + 1) : 0;
            if (kind == 4) length = j % 1024 == 0 ? limit : (limit ? 1 : 0);
            size_t first = bytes.size();
            for (size_t i = 0; i < length; ++i) bytes.push_back(uint8_t(32 + (i + j) % 95));
            if (kind == 1) for (size_t i = 0; i + 3 < length; i += 4) {
                uint32_t c = uint32_t(0x10000 + j % 0xfffff);
                bytes[first + i] = uint8_t(0xf0 | c >> 18);
                bytes[first + i + 1] = uint8_t(0x80 | ((c >> 12) & 63));
                bytes[first + i + 2] = uint8_t(0x80 | ((c >> 6) & 63));
                bytes[first + i + 3] = uint8_t(0x80 | (c & 63));
            }
            if (kind == 2 && length) bytes[first] = 0xff;
            if (kind == 3 && length) bytes[first + length - 1] = 0xff;
            offsets.push_back(bytes.size());
        }
        size_t offered = bytes.size();
        if (bytes.empty()) bytes.push_back(0);
        for (size_t j = 0; j < count; ++j)
            expected.push_back(records_reference(bytes.data() + offsets[j], offsets[j + 1] - offsets[j]));
        require(offsets.back() == offered, "input size");
        saved = bytes; saved_offsets = offsets; output.resize(count);
    }
    RecordWork work(size_t grain) { return {bytes.data(), offsets.data(), output.data(), output.size(), grain}; }
    void reset() { std::fill(output.begin(), output.end(), UINT64_MAX - 1); }
    void check() const {
        require(output == expected, "complete output");
        require(bytes == saved && offsets == saved_offsets, "input immutability");
    }
};
struct Capacity {
    unsigned width;
    std::atomic<unsigned> active{0}, peak{0}, completed{0};
    std::mutex mutex;
    std::set<std::thread::id> threads;
    explicit Capacity(unsigned value) : width(value) {}
};
static void capacity_chunk(void *opaque, size_t i) {
    auto &probe = *static_cast<Capacity *>(opaque);
    require(i < probe.width * 32, "capacity index");
    { std::lock_guard<std::mutex> lock(probe.mutex); probe.threads.insert(std::this_thread::get_id()); }
    unsigned active = probe.active.fetch_add(1) + 1, peak = probe.peak.load();
    while (peak < active && !probe.peak.compare_exchange_weak(peak, active)) {}
    require(active <= probe.width, "callback capacity exceeded");
    volatile uint64_t value = i + 1;
    for (unsigned work = 0; work < 100000; ++work) value = value * UINT64_C(6364136223846793005) + 1;
    probe.active.fetch_sub(1);
    probe.completed.fetch_add(1);
}
static unsigned capacity_check(unsigned width) {
    auto caller = std::this_thread::get_id();
    for (unsigned wave = 1; wave <= 3; ++wave) {
        Capacity probe(width);
        records_scheduler_run(width, width * 32, capacity_chunk, &probe);
        require(probe.completed.load() == width * 32 && probe.active.load() == 0, "capacity completion");
        require(probe.threads.size() <= width, "excess participant threads");
        if (probe.threads.size() == width && probe.threads.count(caller) && probe.peak.load() == width) return wave;
    }
    require(false, "full capacity was not observed with finite CPU callbacks");
    return 0;
}
struct CheckedWork {
    RecordWork work;
    size_t chunks;
    std::unique_ptr<std::atomic<unsigned>[]> seen;
    std::atomic<size_t> completed{0};
    CheckedWork(RecordWork work_, size_t chunks_) : work(work_), chunks(chunks_), seen(new std::atomic<unsigned>[chunks_]) {
        for (size_t i = 0; i < chunks; ++i) seen[i].store(0);
    }
};
static void checked_chunk(void *opaque, size_t chunk) {
    auto &work = *static_cast<CheckedWork *>(opaque);
    require(chunk < work.chunks && work.seen[chunk].fetch_add(1) == 0, "exactly-once chunk");
    records_scheduler_chunk(&work.work, chunk);
    work.completed.fetch_add(1);
}
static int qualify(unsigned width) {
    alarm(90);
    unsigned capacity_waves = capacity_check(width);
    size_t batches = 0, outputs = 0;
    const size_t counts[] = {0,1,2,3,7,15,16,17,31,32,33,257,4097};
    for (const char *shape : {"ascii", "unicode", "error-first", "error-last", "skew"}) {
        for (size_t count : counts) {
            Input input(count, 129, shape, 17819);
            for (size_t grain : {size_t(1), size_t(4), size_t(16), size_t(64)}) {
                for (unsigned pass = 0; pass < 3; ++pass) {
                    input.reset();
                    size_t chunks = (count + grain - 1) / grain;
                    CheckedWork work(input.work(grain), chunks);
                    records_scheduler_run(width, chunks, checked_chunk, &work);
                    require(work.completed.load() == chunks, "joined callback tails");
                    for (size_t i = 0; i < chunks; ++i) require(work.seen[i].load() == 1, "missing chunk");
                    input.check(); ++batches; outputs += count;
                }
            }
        }
    }
    int shutdown = records_scheduler_stop();
    std::printf("record scheduler qualification PASS: backend=%s width=%u actual=%u capacity_waves=%u batches=%zu outputs=%zu explicit_shutdown=%d\n",
                records_scheduler_name(), width, width, capacity_waves, batches, outputs, shutdown);
    alarm(0); return 0;
}
struct Sample {
    uint64_t ns, user, system, rss;
    long voluntary, involuntary;
};
static int benchmark(int argc, char **argv) {
    require(argc == 9, "usage: scheduler WIDTH RECORDS MAX_LENGTH SHAPE GRAIN REPS SEED PASS");
    uint64_t width64 = number(argv[1]), count64 = number(argv[2]), limit64 = number(argv[3]);
    uint64_t grain64 = number(argv[5]), reps64 = number(argv[6]), seed64 = number(argv[7]), pass = number(argv[8]);
    require((width64 == 1 || width64 == 2 || width64 == 4) && count64 <= 1048576 && limit64 <= 1048576 &&
            grain64 >= 1 && grain64 <= 1048576 && reps64 >= 1 && reps64 <= 256 && seed64 <= UINT32_MAX, "benchmark domain");
    unsigned width = unsigned(width64);
    size_t count = size_t(count64), limit = size_t(limit64), grain = size_t(grain64), reps = size_t(reps64);
    Input input(count, limit, argv[4], uint32_t(seed64));
    RecordWork work = input.work(grain);
    size_t chunks = (count + grain - 1) / grain;
    std::vector<Sample> samples(reps + 1);
    uint64_t floor = UINT64_MAX;
    for (unsigned i = 0; i < 1000; ++i) { uint64_t start = now(); floor = std::min(floor, now() - start); }
    alarm(90);
    for (size_t call = 0; call <= reps; ++call) {
        input.reset();
        rusage before, after;
        require(getrusage(RUSAGE_SELF, &before) == 0, "initial resources");
        uint64_t start = now();
        records_scheduler_run(width, chunks, records_scheduler_chunk, &work);
        uint64_t end = now();
        require(getrusage(RUSAGE_SELF, &after) == 0, "final resources");
        uint64_t rss = uint64_t(after.ru_maxrss);
#ifndef __APPLE__
        rss *= 1024;
#endif
        samples[call] = {end - start, cpu_us(after.ru_utime) - cpu_us(before.ru_utime),
                         cpu_us(after.ru_stime) - cpu_us(before.ru_stime), rss,
                         after.ru_nvcsw - before.ru_nvcsw, after.ru_nivcsw - before.ru_nivcsw};
        input.check();
    }
    /* Qualify real participation after timing, so the first timed invocation
     * still includes the scheduler's lazy startup and dispatch. */
    unsigned capacity_waves = capacity_check(width);
    uint64_t stop_start = now();
    int shutdown = records_scheduler_stop();
    uint64_t stop_ns = now() - stop_start;
    std::printf("# backend=%s width=%u shape=%s records=%zu bytes=%" PRIu64 " max_length=%zu grain=%zu chunks=%zu seed=%" PRIu64 " pass=%" PRIu64 " reps=%zu clock_pair_min_ns=%" PRIu64 "\n",
                records_scheduler_name(), width, argv[4], count, input.offsets.back(), limit, grain, chunks, seed64, pass, reps, floor);
    std::puts("backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\tcall\tphase\tcall_ns\tuser_us\tsystem_us\tmaxrss_bytes\tvoluntary_switches\tinvoluntary_switches");
    for (size_t i = 0; i < samples.size(); ++i) {
        auto s = samples[i];
        std::printf("%s\t%u\t%s\t%zu\t%" PRIu64 "\t%zu\t%zu\t%zu\t%" PRIu64 "\t%" PRIu64 "\t%zu\t%s\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\n",
                    records_scheduler_name(), width, argv[4], count, input.offsets.back(), limit, grain, chunks, seed64, pass,
                    i, i ? "warm" : "first", s.ns, s.user, s.system, s.rss, s.voluntary, s.involuntary);
    }
    std::printf("# post_timing_capacity=%u includes_caller=1 capacity_waves=%u explicit_shutdown=%d stop_ns=%" PRIu64 "\n", width, capacity_waves, shutdown, stop_ns);
    std::printf("record scheduler benchmark PASS: calls=%zu outputs=%zu\n", reps + 1, (reps + 1) * count);
    alarm(0); return 0;
}
extern "C" int wf__main_body(int argc, char **argv) {
    if (argc == 3 && !std::strcmp(argv[1], "check")) {
        uint64_t width = number(argv[2]);
        require(width == 1 || width == 2 || width == 4, "qualification width");
        return qualify(unsigned(width));
    }
    return benchmark(argc, argv);
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
