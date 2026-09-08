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
#if defined(RECORD_SCHEDULER_TRACE)
#include <pthread.h>
#if defined(__linux__)
#include <sys/syscall.h>
#endif
#endif
#if defined(RECORD_SCHEDULER_EVENTS)
#include "runtime_events.h"
#include "events-binding.h"
static_assert(WF_EVENT_COUNT == 20 && RAYON_EVENT_COUNT == 13 && RAYON_EVENT_BANKS == 5,
              "runtime event schema changed");
extern "C" uint64_t rayon_event(unsigned, unsigned) __asm__(RAYON_EVENT_SYMBOL);
#endif

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
    unsigned width, iterations;
    std::atomic<unsigned> active{0}, peak{0}, completed{0};
    std::mutex mutex;
    std::set<std::thread::id> threads;
    Capacity(unsigned value, unsigned work) : width(value), iterations(work) {}
};
static void capacity_chunk(void *opaque, size_t i) {
    auto &probe = *static_cast<Capacity *>(opaque);
    require(i < probe.width * 32, "capacity index");
    { std::lock_guard<std::mutex> lock(probe.mutex); probe.threads.insert(std::this_thread::get_id()); }
    unsigned active = probe.active.fetch_add(1) + 1, peak = probe.peak.load();
    while (peak < active && !probe.peak.compare_exchange_weak(peak, active)) {}
    require(active <= probe.width, "callback capacity exceeded");
    volatile uint64_t value = i + 1;
    for (unsigned work = 0; work < probe.iterations; ++work) value = value * UINT64_C(6364136223846793005) + 1;
    probe.active.fetch_sub(1);
    probe.completed.fetch_add(1);
}
static unsigned capacity_check(unsigned width) {
    auto caller = std::this_thread::get_id();
    // Fixed finite waves expose overlap despite delayed worker dispatch.
    // No callback waits for another index or a measured timing threshold.
    const unsigned iterations[] = {100000, 1000000, 10000000};
    struct Observation { size_t threads; unsigned peak; bool caller; } observations[3]{};
    for (unsigned wave = 1; wave <= 3; ++wave) {
        Capacity probe(width, iterations[wave - 1]);
        records_scheduler_run(width, width * 32, capacity_chunk, &probe);
        require(probe.completed.load() == width * 32 && probe.active.load() == 0, "capacity completion");
        require(probe.threads.size() <= width, "excess participant threads");
        auto &observed = observations[wave - 1];
        observed = {probe.threads.size(), probe.peak.load(), probe.threads.count(caller) != 0};
        if (observed.threads == width && observed.caller && observed.peak == width) return wave;
    }
    for (unsigned wave = 1; wave <= 3; ++wave) {
        auto observed = observations[wave - 1];
        std::fprintf(stderr, "record scheduler capacity: backend=%s requested=%u wave=%u iterations=%u threads=%zu peak=%u caller=%u\n",
                     records_scheduler_name(), width, wave, iterations[wave - 1], observed.threads, observed.peak, unsigned(observed.caller));
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
    uint64_t gap;
};
#if defined(RECORD_SCHEDULER_TRACE)
static uint64_t native_thread_id() {
    // Register once per thread. A helper's first lookup is charged to the
    // observed dispatch, but excluded from the common-work timestamp interval.
    thread_local uint64_t cached = [] {
        uint64_t id = 0;
#if defined(__APPLE__)
        require(pthread_threadid_np(nullptr, &id) == 0, "native thread identity");
#elif defined(__linux__)
        long value = syscall(SYS_gettid);
        require(value > 0, "native thread identity");
        id = uint64_t(value);
#else
#error "record trace native thread identity is unqualified on this platform"
#endif
        require(id != 0, "zero thread identity");
        return id;
    }();
    return cached;
}
struct alignas(128) TraceCell {
    // A separate claim per callback detects duplicate execution before any
    // non-atomic trace writes. No shared global counter or event allocation.
    std::atomic<unsigned> claimed{0};
    uint64_t tid = 0, begin = 0, end = 0;
};
struct TraceWork {
    RecordWork work;
    TraceCell *cells;
    size_t chunks;
};
template<bool Timeline> static void traced_chunk(void *opaque, size_t chunk) {
    auto &work = *static_cast<TraceWork *>(opaque);
    require(chunk < work.chunks, "trace chunk range");
    auto &cell = work.cells[chunk];
    require(cell.claimed.fetch_add(1, std::memory_order_relaxed) == 0, "trace duplicate chunk");
    uint64_t tid = native_thread_id();
    uint64_t begin = Timeline ? now() : 0;
    records_scheduler_chunk(&work.work, chunk);
    uint64_t end = Timeline ? now() : 0;
    cell.tid = tid; cell.begin = begin; cell.end = end;
}
#if defined(RECORD_SCHEDULER_EVENTS)
struct RuntimeSnapshot { uint64_t values[5][WF_EVENT_COUNT]{}; };
struct RuntimeEvents {
    bool wf;
    unsigned banks, events;
    explicit RuntimeEvents(unsigned width) : wf(!std::strcmp(records_scheduler_name(), "wf-runtime")),
        banks(wf ? width : 5), events(wf ? unsigned(WF_EVENT_COUNT) : 13) {
        require(wf || !std::strcmp(records_scheduler_name(), "rayon-1.12.0-join"), "runtime events require WF or Rayon join");
    }
    const char *schema() const { return wf ? "wf-1" : "rayon-join-1"; }
    RuntimeSnapshot read() const {
        RuntimeSnapshot result;
        for (unsigned bank = 0; bank < banks; ++bank)
            for (unsigned event = 0; event < events; ++event)
                result.values[bank][event] = wf ? wf_compute_event(bank, event) : rayon_event(bank, event);
        return result;
    }
    RuntimeSnapshot difference(const RuntimeSnapshot &before, const RuntimeSnapshot &after,
                               unsigned width, size_t chunks) const {
        RuntimeSnapshot result;
        uint64_t totals[WF_EVENT_COUNT]{};
        for (unsigned bank = 0; bank < banks; ++bank) {
            for (unsigned event = 0; event < events; ++event) {
                require(after.values[bank][event] >= before.values[bank][event], "runtime counter rollover");
                auto delta = after.values[bank][event] - before.values[bank][event];
                result.values[bank][event] = delta;
                require(delta <= UINT64_MAX - totals[event], "runtime counter sum overflow");
                totals[event] += delta;
                if (!wf && bank >= width) require(delta == 0, "Rayon unexpected worker or external activity");
            }
        }
        uint64_t jobs = chunks && (!wf || width != 1) ? chunks - 1 : 0;
        require(totals[0] == jobs, "runtime publication count");
        if (wf) {
            require(totals[WF_EVENT_LOCAL_POP] <= jobs && totals[WF_EVENT_STEAL_SUCCESS] <= jobs &&
                    totals[WF_EVENT_LOCAL_POP] + totals[WF_EVENT_STEAL_SUCCESS] == jobs, "WF deque conservation");
            require(totals[WF_EVENT_RUN_BEGIN] == jobs && totals[WF_EVENT_RUN_END] == jobs &&
                    totals[WF_EVENT_JOIN] == jobs, "WF completion conservation");
            require(totals[WF_EVENT_INLINE_RUN] <= totals[WF_EVENT_LOCAL_POP] &&
                    totals[WF_EVENT_SLOT_REFUSAL] == 0, "WF inline or slot inventory");
        } else {
            require(totals[1] <= jobs && totals[2] <= jobs && totals[1] + totals[2] == jobs &&
                    totals[3] == jobs && totals[4] <= jobs && totals[5] <= jobs &&
                    totals[4] + totals[5] == jobs, "Rayon join conservation");
            for (unsigned event = 6; event < events; ++event)
                require(totals[event] == 0, "Rayon unsupported job or queue activity");
        }
        return result;
    }
};
#endif
static int trace(int argc, char **argv) {
    require(argc == 11, "usage: scheduler trace WIDTH RECORDS MAX_LENGTH SHAPE GRAIN REPS SEED PASS LEVEL");
    uint64_t width64 = number(argv[2]), count64 = number(argv[3]), limit64 = number(argv[4]);
    uint64_t grain64 = number(argv[6]), reps64 = number(argv[7]), seed64 = number(argv[8]), pass = number(argv[9]);
    const char *level = argv[10];
    bool plain = !std::strcmp(level, "plain"), timeline = !std::strcmp(level, "timeline");
    require(plain || timeline || !std::strcmp(level, "identity"), "trace level");
    require((width64 == 1 || width64 == 2 || width64 == 4) && count64 <= 1048576 && limit64 <= 1048576 &&
            grain64 >= 1 && grain64 <= 1048576 && reps64 >= 1 && reps64 <= 32 && seed64 <= UINT32_MAX, "trace domain");
    unsigned width = unsigned(width64);
    size_t count = size_t(count64), limit = size_t(limit64), grain = size_t(grain64), reps = size_t(reps64);
    size_t chunks = (count + grain - 1) / grain;
    require(!count || reps + 1 <= 8388608 / count, "trace output domain");
    require(!chunks || reps + 1 <= 65536 / chunks, "trace event domain");
    Input input(count, limit, argv[5], uint32_t(seed64));
    // Plain/identity/timeline processes have the same preallocated footprint.
    // Every call retains separate output and event storage until verification.
    std::vector<std::vector<uint64_t>> outputs(reps + 1, std::vector<uint64_t>(count, UINT64_MAX - 1));
    std::unique_ptr<TraceCell[]> cells(new TraceCell[(reps + 1) * chunks]);
    struct Call { uint64_t begin, end; Sample resources; };
    std::vector<Call> calls(reps + 1);
#if defined(RECORD_SCHEDULER_EVENTS)
    RuntimeEvents runtime(width);
    std::vector<RuntimeSnapshot> runtime_calls(reps + 1);
#endif
    uint64_t caller = native_thread_id(), floor = UINT64_MAX;
    for (unsigned i = 0; i < 1000; ++i) { uint64_t start = now(); floor = std::min(floor, now() - start); }
    RecordChunk callback = timeline ? traced_chunk<true> : traced_chunk<false>;
    alarm(90);
    rusage batch_before, batch_after;
    require(getrusage(RUSAGE_SELF, &batch_before) == 0, "initial trace batch resources");
    uint64_t batch_start = now();
    for (size_t call = 0; call <= reps; ++call) {
        TraceWork work{input.work(grain), cells.get() + call * chunks, chunks};
        work.work.output = outputs[call].data();
#if defined(RECORD_SCHEDULER_EVENTS)
        // Snapshots enclose each call's clocks/resources, not just dispatch.
        // Background searches may cross these boundaries. Joined-job counts
        // are stable; search/wait/signal deltas are not exact interval flows.
        RuntimeSnapshot runtime_before = runtime.read();
#endif
        rusage before, after;
        require(getrusage(RUSAGE_SELF, &before) == 0, "initial trace resources");
        uint64_t begin = now();
        if (plain) records_scheduler_run(width, chunks, records_scheduler_chunk, &work.work);
        else records_scheduler_run(width, chunks, callback, &work);
        uint64_t end = now();
        require(getrusage(RUSAGE_SELF, &after) == 0, "final trace resources");
#if defined(RECORD_SCHEDULER_EVENTS)
        runtime_calls[call] = runtime.difference(runtime_before, runtime.read(), width, chunks);
#endif
        uint64_t rss = uint64_t(after.ru_maxrss);
#ifndef __APPLE__
        rss *= 1024;
#endif
        calls[call] = {begin, end, {end - begin, cpu_us(after.ru_utime) - cpu_us(before.ru_utime),
            cpu_us(after.ru_stime) - cpu_us(before.ru_stime), rss,
            after.ru_nvcsw - before.ru_nvcsw, after.ru_nivcsw - before.ru_nivcsw, 0}};
    }
    uint64_t batch_end = now();
    require(getrusage(RUSAGE_SELF, &batch_after) == 0, "final trace batch resources");
    uint64_t batch_rss = uint64_t(batch_after.ru_maxrss);
#ifndef __APPLE__
    batch_rss *= 1024;
#endif
    for (size_t call = 0; call <= reps; ++call) {
        input.output.swap(outputs[call]); input.check(); input.output.swap(outputs[call]);
        std::set<uint64_t> participants;
        for (size_t i = 0; i < chunks; ++i) {
            auto &cell = cells[call * chunks + i];
            require(cell.claimed.load(std::memory_order_relaxed) == unsigned(!plain), "trace callback inventory");
            if (plain) continue;
            require(cell.tid != 0, "trace missing callback tail");
            participants.insert(cell.tid);
            require(timeline ? calls[call].begin <= cell.begin && cell.begin <= cell.end && cell.end <= calls[call].end :
                               cell.begin == 0 && cell.end == 0, "trace callback interval");
        }
        require(participants.size() <= width, "trace excess workers");
    }
    // Capacity and its possible lazy startup stay outside all traced calls.
    unsigned capacity_waves = capacity_check(width);
    uint64_t stop_start = now();
    int shutdown = records_scheduler_stop();
    uint64_t stop_ns = now() - stop_start;
    std::printf("# trace backend=%s width=%u shape=%s records=%zu bytes=%" PRIu64 " max_length=%zu grain=%zu chunks=%zu seed=%" PRIu64 " pass=%" PRIu64 " reps=%zu level=%s caller_tid=%" PRIu64 " clock_pair_min_ns=%" PRIu64 "\n",
        records_scheduler_name(), width, argv[5], count, input.offsets.back(), limit, grain, chunks, seed64, pass, reps, level, caller, floor);
#if defined(RECORD_SCHEDULER_EVENTS)
    std::printf("# runtime_events schema=%s banks=%u events=%u\n", runtime.schema(), runtime.banks, runtime.events);
#endif
    for (size_t call = 0; call <= reps; ++call) {
        auto c = calls[call]; auto s = c.resources;
        std::printf("call\t%zu\t%s\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\n",
            call, call ? "warm" : "first", c.begin, c.end, s.user, s.system, s.rss, s.voluntary, s.involuntary);
#if defined(RECORD_SCHEDULER_EVENTS)
        for (unsigned bank = 0; bank < runtime.banks; ++bank) {
            std::printf("runtime\t%zu\t%u", call, bank);
            for (unsigned event = 0; event < runtime.events; ++event)
                std::printf("\t%" PRIu64, runtime_calls[call].values[bank][event]);
            std::putchar('\n');
        }
#endif
        if (plain) continue;
        for (size_t i = 0; i < chunks; ++i) {
            auto &cell = cells[call * chunks + i];
            size_t first = i * grain, length = std::min(grain, count - first);
            std::printf("chunk\t%zu\t%zu\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%zu\t%zu\t%" PRIu64 "\n",
                call, i, cell.tid, cell.begin, cell.end, first, length, input.offsets[first + length] - input.offsets[first]);
        }
    }
    std::printf("# batch start_ns=%" PRIu64 " end_ns=%" PRIu64 " user_us=%" PRIu64 " system_us=%" PRIu64 " maxrss_bytes=%" PRIu64 " voluntary_switches=%ld involuntary_switches=%ld\n",
        batch_start, batch_end, cpu_us(batch_after.ru_utime) - cpu_us(batch_before.ru_utime),
        cpu_us(batch_after.ru_stime) - cpu_us(batch_before.ru_stime), batch_rss,
        batch_after.ru_nvcsw - batch_before.ru_nvcsw, batch_after.ru_nivcsw - batch_before.ru_nivcsw);
    std::printf("# post_timing_capacity=%u includes_caller=1 capacity_waves=%u explicit_shutdown=%d stop_ns=%" PRIu64 "\n", width, capacity_waves, shutdown, stop_ns);
    std::printf("record scheduler trace PASS: calls=%zu outputs=%zu traced_chunks=%zu\n", reps + 1, (reps + 1) * count, plain ? 0 : (reps + 1) * chunks);
    alarm(0); return 0;
}
#endif
static int benchmark(int argc, char **argv) {
    require(argc == 10, "usage: scheduler WIDTH RECORDS MAX_LENGTH SHAPE GRAIN REPS SEED PASS CADENCE");
    uint64_t width64 = number(argv[1]), count64 = number(argv[2]), limit64 = number(argv[3]);
    uint64_t grain64 = number(argv[5]), reps64 = number(argv[6]), seed64 = number(argv[7]), pass = number(argv[8]);
    require((width64 == 1 || width64 == 2 || width64 == 4) && count64 <= 1048576 && limit64 <= 1048576 &&
            grain64 >= 1 && grain64 <= 1048576 && reps64 >= 1 && reps64 <= 256 && seed64 <= UINT32_MAX, "benchmark domain");
    unsigned width = unsigned(width64);
    size_t count = size_t(count64), limit = size_t(limit64), grain = size_t(grain64), reps = size_t(reps64);
    const char *cadence = argv[9];
    bool checked = !std::strcmp(cadence, "checked");
    uint64_t requested_gap = !std::strcmp(cadence, "sleep-100us") ? 100000 :
                             !std::strcmp(cadence, "sleep-1ms") ? 1000000 : 0;
    require(checked || !std::strcmp(cadence, "dense") || requested_gap, "benchmark cadence");
    require(!count || reps + 1 <= 8388608 / count, "retained output domain");
    Input input(count, limit, argv[4], uint32_t(seed64));
    RecordWork work = input.work(grain);
    size_t chunks = (count + grain - 1) / grain;
    std::vector<Sample> samples(reps + 1);
    // All cadences use identical distinct output storage for each call. This
    // permits complete deferred verification without overwriting earlier results.
    std::vector<std::vector<uint64_t>> outputs(reps + 1, std::vector<uint64_t>(count, UINT64_MAX - 1));
    uint64_t floor = UINT64_MAX;
    for (unsigned i = 0; i < 1000; ++i) { uint64_t start = now(); floor = std::min(floor, now() - start); }
    alarm(90);
    rusage batch_before, batch_after;
    require(getrusage(RUSAGE_SELF, &batch_before) == 0, "initial batch resources");
    uint64_t batch_start = now(), previous_end = 0;
    for (size_t call = 0; call <= reps; ++call) {
        if (call && requested_gap) {
            timespec remaining{0, long(requested_gap)};
            while (nanosleep(&remaining, &remaining) != 0) require(errno == EINTR, "inter-call sleep");
        }
        work.output = outputs[call].data();
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
                         after.ru_nvcsw - before.ru_nvcsw, after.ru_nivcsw - before.ru_nivcsw,
                         call ? start - previous_end : 0};
        previous_end = end;
        if (checked) {
            input.output.swap(outputs[call]);
            input.check();
            input.output.swap(outputs[call]);
        }
    }
    uint64_t batch_ns = now() - batch_start;
    require(getrusage(RUSAGE_SELF, &batch_after) == 0, "final batch resources");
    uint64_t batch_rss = uint64_t(batch_after.ru_maxrss);
#ifndef __APPLE__
    batch_rss *= 1024;
#endif
    // Every timed result is checked in every mode, including all earlier
    // calls in a dense burst. The batch resource interval ends before this.
    for (size_t call = 0; call <= reps; ++call) {
        input.output.swap(outputs[call]);
        input.check();
        input.output.swap(outputs[call]);
    }
    /* Qualify real participation after timing, so the first timed invocation
     * still includes the scheduler's lazy startup and dispatch. */
    unsigned capacity_waves = capacity_check(width);
    uint64_t stop_start = now();
    int shutdown = records_scheduler_stop();
    uint64_t stop_ns = now() - stop_start;
    std::printf("# backend=%s width=%u shape=%s records=%zu bytes=%" PRIu64 " max_length=%zu grain=%zu chunks=%zu seed=%" PRIu64 " pass=%" PRIu64 " reps=%zu clock_pair_min_ns=%" PRIu64 " cadence=%s requested_gap_ns=%" PRIu64 "\n",
                records_scheduler_name(), width, argv[4], count, input.offsets.back(), limit, grain, chunks, seed64, pass, reps, floor, cadence, requested_gap);
    std::puts("backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\tcall\tphase\tcall_ns\tuser_us\tsystem_us\tmaxrss_bytes\tvoluntary_switches\tinvoluntary_switches\tcadence\tgap_ns");
    for (size_t i = 0; i < samples.size(); ++i) {
        auto s = samples[i];
        std::printf("%s\t%u\t%s\t%zu\t%" PRIu64 "\t%zu\t%zu\t%zu\t%" PRIu64 "\t%" PRIu64 "\t%zu\t%s"
                    "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\t%s\t%" PRIu64 "\n",
                    records_scheduler_name(), width, argv[4], count, input.offsets.back(), limit, grain, chunks, seed64, pass,
                    i, i ? "warm" : "first", s.ns, s.user, s.system, s.rss, s.voluntary, s.involuntary, cadence, s.gap);
    }
    std::printf("# batch_includes_first=1 batch_includes_checks=%d batch_ns=%" PRIu64 " user_us=%" PRIu64 " system_us=%" PRIu64 " maxrss_bytes=%" PRIu64 " voluntary_switches=%ld involuntary_switches=%ld\n",
                int(checked), batch_ns, cpu_us(batch_after.ru_utime) - cpu_us(batch_before.ru_utime),
                cpu_us(batch_after.ru_stime) - cpu_us(batch_before.ru_stime), batch_rss,
                batch_after.ru_nvcsw - batch_before.ru_nvcsw, batch_after.ru_nivcsw - batch_before.ru_nivcsw);
    std::printf("# post_timing_capacity=%u includes_caller=1 capacity_waves=%u explicit_shutdown=%d stop_ns=%" PRIu64 "\n", width, capacity_waves, shutdown, stop_ns);
    std::printf("record scheduler benchmark PASS: calls=%zu outputs=%zu\n", reps + 1, (reps + 1) * count);
    alarm(0); return 0;
}
#if defined(RECORD_SCHEDULER_LEAK_PROBE)
extern "C" __attribute__((noinline)) void records_scheduler_leak_probe(void) {
    // Volatile publication prevents allocation elision. The floor joins the
    // entry thread before leak checking, removing its stale stack roots.
    static void *volatile leaked;
    leaked = std::malloc(257);
    require(leaked != nullptr, "leak probe allocation");
    leaked = nullptr;
}
#endif
extern "C" int wf__main_body(int argc, char **argv) {
    int status;
#if defined(RECORD_SCHEDULER_TRACE)
    if (argc > 1 && !std::strcmp(argv[1], "trace")) status = trace(argc, argv);
    else
#endif
    if (argc == 3 && !std::strcmp(argv[1], "check")) {
        uint64_t width = number(argv[2]);
        require(width == 1 || width == 2 || width == 4, "qualification width");
        status = qualify(unsigned(width));
    } else {
        status = benchmark(argc, argv);
    }
#if defined(RECORD_SCHEDULER_LEAK_PROBE)
    // A separate sanitizer-only executable proves that the lifecycle checker
    // rejects an additional leak after a fully successful functional report.
    records_scheduler_leak_probe();
#endif
    // The strong floor joins this entry thread before process-exit sanitizer
    // checks. Preserve its completed report even if LeakSanitizer then exits
    // without flushing C stdio. This observation is outside all timed work.
    require(std::fflush(stdout) == 0, "report flush");
    return status;
}
#if defined(RECORD_SCHEDULER_SELECT)
extern "C" void records_scheduler_select(const char *name);
int main(int argc, char **argv) {
    // Select once before floor entry and every measured interval. The common
    // host, callback and leaf then have one image layout for every backend.
    require(argc >= 2, "shared-image backend argument");
    records_scheduler_select(argv[1]);
    return wf__floor_run(argc - 1, argv + 1);
}
#else
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
#endif
