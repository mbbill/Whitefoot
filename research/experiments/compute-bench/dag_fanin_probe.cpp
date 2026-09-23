// Runtime-DAG investigation only: complete-output Kahn oracle, passive task
// events and the pinned oneTBB flow-graph reference. No performance intervals.
// The explicit Makefile targets are its only caller; retire with the trial.
#include <oneapi/tbb/flow_graph.h>
#include <oneapi/tbb/global_control.h>
#include <oneapi/tbb/task_arena.h>
#include <oneapi/tbb/version.h>

#include <algorithm>
#include <array>
#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <memory>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

static_assert(TBB_VERSION_MAJOR == 2023 && TBB_VERSION_MINOR == 1 &&
              TBB_VERSION_PATCH == 0, "use the existing pinned oneTBB cache");
struct TaskCell { std::uint64_t value, evaluations; };
static_assert(sizeof(TaskCell) == 16 && offsetof(TaskCell, evaluations) == 8,
              "the LLVM adapter exposes two consecutive u64 fields");
extern "C" {
int wf__floor_run(int, char **);
void dag_probe_spine(std::uint64_t, const std::uint64_t *, std::uint64_t,
                     TaskCell *, std::uint64_t);
void dag_probe_notify(std::uint64_t, const std::uint64_t *, TaskCell *,
                      std::uint64_t *, std::uint64_t);
void dag_probe_n(std::uint64_t, const std::uint64_t *, TaskCell *, std::uint64_t);
unsigned dag_probe_trace_image();
unsigned long wf__par_grants();
int wf__par_pool_active();
}
namespace {
namespace tbb = oneapi::tbb;
constexpr std::uint64_t expensive = 65536;
constexpr std::uint64_t seed_value = UINT64_C(0xffffffffffffffe7);
constexpr std::uint64_t guard_value = UINT64_C(0xbadc0ffee0ffee17);
enum class Family { spine, notify, n };
struct Graph {
    Family family;
    std::string name;
    unsigned mode = 0;
    std::uint64_t argument = 0, seed = seed_value;
    std::vector<std::uint64_t> costs;
    std::vector<std::pair<std::size_t, std::size_t>> edges;
    Graph(Family kind, std::string label) : family(kind), name(std::move(label)) {}
};
using Rows = std::vector<TaskCell>;
using Receipts = std::array<std::uint64_t, 4>;
struct Expected {
    Rows rows;
    Receipts receipts{};
    std::uint64_t work = 0, critical = 0, level = 0;
};
[[noreturn]] void fail(const std::string &message) {
    throw std::runtime_error(message);
}
std::uint64_t rotate(std::uint64_t value, unsigned amount) {
    return (value << amount) | (value >> (64 - amount));
}
// Independent scalar implementation of the written task contract.
std::uint64_t reference_task(std::uint64_t id, std::uint64_t steps,
                             std::uint64_t seed, std::uint64_t incoming) {
    std::uint64_t state = (seed + id) ^ incoming;
    for (std::uint64_t i = 0; i != steps; ++i)
        state = (rotate(state, 13) * UINT64_C(6364136223846793005) +
                 UINT64_C(1442695040888963407)) ^ i;
    return state;
}
std::vector<std::vector<std::size_t>> predecessors(const Graph &graph) {
    std::vector<std::vector<std::size_t>> result(graph.costs.size());
    for (auto edge : graph.edges) {
        if (edge.first >= result.size() || edge.second >= result.size())
            fail("fixture edge outside graph");
        result[edge.second].push_back(edge.first);
    }
    for (auto &row : result) {
        std::sort(row.begin(), row.end());
        if (std::adjacent_find(row.begin(), row.end()) != row.end())
            fail("fixture repeats an edge");
    }
    return result;
}
Expected oracle(const Graph &graph) {
    const std::size_t count = graph.costs.size();
    auto incoming = predecessors(graph);
    std::vector<std::vector<std::size_t>> outgoing(count);
    for (auto edge : graph.edges) outgoing[edge.first].push_back(edge.second);
    std::vector<std::size_t> remaining(count), ready, depth(count);
    std::vector<std::uint64_t> finish(count), level_max(count);
    for (std::size_t i = 0; i != count; ++i) {
        remaining[i] = incoming[i].size();
        if (remaining[i] == 0) ready.push_back(i);
    }
    Expected result;
    result.rows.resize(count);
    for (std::size_t cursor = 0; cursor != ready.size(); ++cursor) {
        const std::size_t id = ready[cursor];
        std::uint64_t mix = 0, earliest = 0;
        for (std::size_t parent : incoming[id]) {
            if (result.rows[parent].evaluations != 1) fail("oracle premature task");
            mix = rotate(mix, 7) ^ result.rows[parent].value;
            earliest = std::max(earliest, finish[parent]);
            depth[id] = std::max(depth[id], depth[parent] + 1);
        }
        result.rows[id] = {reference_task(id, graph.costs[id], graph.seed, mix), 1};
        result.work += graph.costs[id];
        finish[id] = earliest + graph.costs[id];
        result.critical = std::max(result.critical, finish[id]);
        level_max[depth[id]] = std::max(level_max[depth[id]], graph.costs[id]);
        if (graph.family == Family::notify && id >= 2) {
            for (std::size_t parent : incoming[id]) {
                result.receipts[2 * (id - 2)] |= UINT64_C(1) << parent;
                ++result.receipts[2 * (id - 2) + 1];
            }
        }
        for (std::size_t next : outgoing[id]) {
            if (remaining[next] == 0) fail("oracle duplicate retirement");
            if (--remaining[next] == 0) ready.push_back(next);
        }
    }
    if (ready.size() != count) fail("fixture contains a cycle");
    for (std::uint64_t cost : level_max) result.level += cost;
    return result;
}
std::string compare_rows(const Rows &actual, const Rows &expected) {
    if (actual.size() != expected.size()) return "task row count mismatch";
    for (std::size_t i = 0; i != actual.size(); ++i) {
        if (actual[i].value != expected[i].value)
            return "task value mismatch at " + std::to_string(i);
        if (actual[i].evaluations != 1)
            return "task evaluation count mismatch at " + std::to_string(i);
    }
    return {};
}
Graph n_graph(unsigned costs, unsigned mode) {
    Graph graph{Family::n, "n-" + std::to_string(costs) + "-" + std::to_string(mode)};
    graph.mode = mode;
    for (unsigned i = 0; i != 4; ++i)
        graph.costs.push_back((costs & (1U << i)) ? expensive : 1);
    graph.edges = {{0, 2}, {1, 2}, {1, 3}};
    return graph;
}
std::vector<Graph> fixtures(bool native) {
    std::vector<Graph> result;
    for (std::uint64_t length : {0, 1, 2, 4, 7, 8, 9, 16, 32}) {
        for (unsigned profile = 0; profile != 4; ++profile) {
            if (length == 0 && profile >= 2) continue;
            Graph graph{Family::spine, "spine-" + std::to_string(length) + "-" +
                                         std::to_string(profile)};
            graph.argument = length;
            graph.costs.assign(2 * length, 1);
            for (std::size_t i = 0; i != length; ++i) {
                if (profile == 1 || (profile == 2 && i == 0) ||
                    (profile == 3 && i + 1 == length)) graph.costs[2 * i + 1] = expensive;
                graph.edges.emplace_back(2 * i, 2 * i + 1);
                if (i + 1 != length) graph.edges.emplace_back(2 * i, 2 * i + 2);
            }
            result.push_back(std::move(graph));
        }
    }
    for (unsigned fixture = 0; fixture != 17; ++fixture) {
        const unsigned mask = fixture == 16 ? 15 : fixture;
        Graph graph{Family::notify, "notify-" + std::to_string(mask) +
                                          (fixture == 16 ? "-costly" : "-cheap")};
        graph.argument = mask;
        graph.costs.assign(4, fixture == 16 ? expensive : 1);
        for (unsigned source = 0; source != 2; ++source)
            for (unsigned destination = 0; destination != 2; ++destination)
                if (mask & (1U << (2 * source + destination)))
                    graph.edges.emplace_back(source, destination + 2);
        result.push_back(std::move(graph));
    }
    for (unsigned costs = 0; costs != 16; ++costs)
        for (unsigned mode = 0; mode != (native ? 1U : 4U); ++mode)
            result.push_back(n_graph(costs, mode));
    return result;
}
struct Event {
    std::uint64_t id = 0, thread = 0, value = 0, steps = 0;
    bool begin = false;
};
struct Observation {
    std::vector<Event> events;
    std::atomic<std::size_t> next{0};
    std::atomic<bool> overflow{false};
} observation;
std::atomic<std::uint64_t> next_thread{1};
thread_local std::uint64_t thread_id = 0;
void event(std::uint64_t id, bool begin, std::uint64_t value, std::uint64_t steps) {
    if (thread_id == 0) thread_id = next_thread.fetch_add(1, std::memory_order_relaxed);
    const std::size_t at = observation.next.fetch_add(1, std::memory_order_seq_cst);
    if (at >= observation.events.size()) {
        observation.overflow.store(true, std::memory_order_relaxed);
        return;
    }
    observation.events[at] = {id, thread_id, value, steps, begin};
}
struct Trace {
    std::vector<std::size_t> begin, end;
    std::vector<std::uint64_t> thread;
};
std::string check_trace(const Graph &graph, const Expected &expected,
                        const std::vector<Event> &events, Trace &trace) {
    const std::size_t count = graph.costs.size(), absent = events.size();
    if (events.size() != 2 * count) return "trace event count mismatch";
    trace.begin.assign(count, absent);
    trace.end.assign(count, absent);
    trace.thread.assign(count, 0);
    for (std::size_t at = 0; at != events.size(); ++at) {
        const Event &current = events[at];
        if (current.id >= count || current.thread == 0) return "trace invalid task/thread ID";
        const std::size_t id = static_cast<std::size_t>(current.id);
        if (current.begin) {
            if (trace.begin[id] != absent) return "trace duplicate task entry";
            if (current.steps != graph.costs[id] || current.value != graph.seed)
                return "trace task input mismatch";
            trace.begin[id] = at;
            trace.thread[id] = current.thread;
        } else {
            if (trace.begin[id] == absent || trace.end[id] != absent)
                return "trace missing entry or duplicate exit";
            if (trace.thread[id] != current.thread) return "trace recurrence changed thread";
            if (current.value != expected.rows[id].value) return "trace task result mismatch";
            trace.end[id] = at;
        }
    }
    for (std::size_t id = 0; id != count; ++id)
        if (trace.begin[id] == absent || trace.end[id] == absent)
            return "trace missing task event";
    for (auto edge : graph.edges)
        if (trace.end[edge.first] >= trace.begin[edge.second])
            return "trace prerequisite not complete";
    return {};
}
bool overlap(const Trace &trace, std::size_t a, std::size_t b) {
    return trace.thread[a] != trace.thread[b] &&
           std::max(trace.begin[a], trace.begin[b]) < std::min(trace.end[a], trace.end[b]);
}
void print_trace(const Graph &graph, const Trace &trace) {
    std::size_t pairs = 0, progress = 0;
    for (std::size_t a = 0; a != graph.costs.size(); ++a)
        for (std::size_t b = a + 1; b != graph.costs.size(); ++b)
            if (overlap(trace, a, b)) ++pairs;
    if (graph.family == Family::spine)
        for (std::size_t i = 0; i != graph.argument; ++i)
            for (std::size_t j = i + 1; j != graph.argument; ++j) {
                const std::size_t leaf = 2 * i + 1, spine = 2 * j;
                if (trace.thread[leaf] != trace.thread[spine] &&
                    trace.begin[leaf] < trace.begin[spine] &&
                    trace.end[spine] < trace.end[leaf]) ++progress;
            }
    std::printf("overlap\t%s\tpairs=%zu\tspine_progress=%zu", graph.name.c_str(), pairs, progress);
    if (graph.family != Family::spine) {
        const bool d_while_a = trace.thread[3] != trace.thread[0] &&
            trace.begin[0] < trace.begin[3] && trace.begin[3] < trace.end[0];
        const bool c_while_d = trace.thread[2] != trace.thread[3] &&
            trace.begin[3] < trace.begin[2] && trace.begin[2] < trace.end[3];
        std::printf("\tsources=%u\towners=%u\td_while_a=%u\tc_while_d=%u",
                    static_cast<unsigned>(overlap(trace, 0, 1)),
                    static_cast<unsigned>(overlap(trace, 2, 3)),
                    static_cast<unsigned>(d_while_a), static_cast<unsigned>(c_while_d));
    }
    std::putchar('\n');
    for (std::size_t at = 0; at != observation.events.size(); ++at) {
        const auto &row = observation.events[at];
        std::printf("event\t%s\t%zu\t%llu\t%s\tthread=%llu\tvalue=%016llx\tsteps=%llu\n",
                    graph.name.c_str(), at, static_cast<unsigned long long>(row.id),
                    row.begin ? "begin" : "end", static_cast<unsigned long long>(row.thread),
                    static_cast<unsigned long long>(row.value), static_cast<unsigned long long>(row.steps));
    }
}
// Unlike the Kahn oracle, this evaluates only a node made ready by incoming
// oneTBB edge notifications. All nodes/edges exist before any root is seeded.
void native_graph(const Graph &graph, const std::uint64_t *costs,
                   TaskCell *output, std::uint64_t *receipts,
                   tbb::task_arena &arena, bool traced);
void whitefoot_graph(const Graph &graph, const std::uint64_t *costs,
                      TaskCell *output, std::uint64_t *receipts) {
    if (graph.family == Family::spine)
        dag_probe_spine(graph.argument, costs, graph.costs.size(), output, graph.seed);
    else if (graph.family == Family::notify)
        dag_probe_notify(graph.argument, costs, output, receipts, graph.seed);
    else dag_probe_n(graph.mode, costs, output, graph.seed);
}
bool equal(TaskCell a, TaskCell b) {
    return a.value == b.value && a.evaluations == b.evaluations;
}
void oracle_control() {
    Graph graph = n_graph(0, 0);
    graph.costs.assign(4, 0);
    graph.seed = 0;
    auto expected = oracle(graph);
    const Rows known{{0, 1}, {1, 1}, {3, 1}, {2, 1}};
    if (!compare_rows(expected.rows, known).empty() ||
        reference_task(0, 1, 0, 0) != UINT64_C(1442695040888963407))
        fail("oracle known-result control");
}
int run_matrix(bool native, unsigned workers) {
    const bool traced = dag_probe_trace_image() != 0;
    oracle_control();
    // Native mode never enters wf__floor_run, so a second WF pool is absent.
    std::unique_ptr<tbb::global_control> limit;
    std::unique_ptr<tbb::task_arena> arena;
    if (native) {
        limit = std::make_unique<tbb::global_control>(tbb::global_control::max_allowed_parallelism,
                                                       workers);
        arena = std::make_unique<tbb::task_arena>(static_cast<int>(workers), 1);
        arena->initialize();
    }
    std::printf("configuration\tengine=%s\timage=%s\ttotal_participants=%u\trepeats=1\n",
                native ? "tbb" : "wf", traced ? "trace" : "plain", workers);
    std::printf("accounting\ttask_cell_bytes=%zu\tevent_bytes=%zu\tphysical_peak=not-measured\n",
                sizeof(TaskCell), sizeof(Event));
    bool output_control = false, trace_control = false;
    std::size_t cases = 0, tasks = 0;
    for (const Graph &graph : fixtures(native)) {
        const Expected expected = oracle(graph);
        const std::size_t count = graph.costs.size();
        const TaskCell guard{guard_value, ~guard_value};
        Rows storage(count + 2);
        storage.front() = storage.back() = guard;
        std::vector<std::uint64_t> costs(count + 2, guard_value);
        std::copy(graph.costs.begin(), graph.costs.end(), costs.begin() + 1);
        const auto original_costs = costs;
        std::array<std::uint64_t, 6> receipt_storage{};
        receipt_storage.front() = receipt_storage.back() = guard_value;
        observation.events.assign(traced ? 2 * count : 0, Event{});
        observation.next.store(0);
        observation.overflow.store(false);
        const unsigned long grants = native ? 0 : wf__par_grants();
        if (native) native_graph(graph, costs.data() + 1, storage.data() + 1,
                                  receipt_storage.data() + 1,
                                  *arena, traced);
        else whitefoot_graph(graph, costs.data() + 1, storage.data() + 1,
                               receipt_storage.data() + 1);
        Rows actual(storage.begin() + 1, storage.end() - 1);
        const auto mismatch = compare_rows(actual, expected.rows);
        if (!mismatch.empty()) fail(graph.name + ": " + mismatch);
        if (!equal(storage.front(), guard) || !equal(storage.back(), guard) ||
            receipt_storage.front() != guard_value || receipt_storage.back() != guard_value)
            fail(graph.name + ": output boundary changed");
        if (costs != original_costs) fail(graph.name + ": input changed");
        for (std::size_t i = 0; i != 4; ++i)
            if (receipt_storage[i + 1] != expected.receipts[i])
                fail(graph.name + ": notification receipt mismatch");
        const unsigned long used = native ? 0 : wf__par_grants() - grants;
        if (!native && workers == 1 && used != 0) fail("W1 handed out work");
        std::printf("case\t%s\ttasks=%zu\tedges=%zu\twork_rounds=%llu\tcritical_rounds=%llu"
                    "\tlevel_rounds=%llu\toutput_bytes=%zu\tinput_cost_bytes=%zu"
                    "\tsource_notice_init_slots=%u\tsource_notice_inspections=%u"
                    "\tobserver_bytes=%zu\tgrants=%lu\n",
                    graph.name.c_str(), count, graph.edges.size(),
                    static_cast<unsigned long long>(expected.work),
                    static_cast<unsigned long long>(expected.critical),
                    static_cast<unsigned long long>(expected.level), count * sizeof(TaskCell),
                    count * sizeof(std::uint64_t),
                    !native && graph.family == Family::notify ? 4U : 0U,
                    !native && graph.family == Family::notify ? 4U : 0U,
                    observation.events.size() * sizeof(Event), used);
        for (std::size_t id = 0; id != count; ++id)
            std::printf("task\t%s\t%zu\tsteps=%llu\tvalue=%016llx\tevaluations=%llu\n",
                        graph.name.c_str(), id, static_cast<unsigned long long>(graph.costs[id]),
                        static_cast<unsigned long long>(actual[id].value),
                        static_cast<unsigned long long>(actual[id].evaluations));
        if (graph.family == Family::notify)
            std::printf("receipts\t%s\tC_mask=%llu\tC_count=%llu\tD_mask=%llu\tD_count=%llu\n",
                        graph.name.c_str(), static_cast<unsigned long long>(receipt_storage[1]),
                        static_cast<unsigned long long>(receipt_storage[2]),
                        static_cast<unsigned long long>(receipt_storage[3]),
                        static_cast<unsigned long long>(receipt_storage[4]));
        if (!output_control && count) {
            Rows corrupted = actual;
            corrupted[0].value ^= 1;
            const auto rejected = compare_rows(corrupted, expected.rows);
            if (rejected.empty()) fail("output corruption escaped comparator");
            std::printf("control\toutput-corruption\trejected=%s\n", rejected.c_str());
            output_control = true;
        }
        if (traced) {
            if (observation.overflow.load() || observation.next.load() != 2 * count)
                fail(graph.name + ": trace event count/overflow");
            Trace trace;
            const auto error = check_trace(graph, expected, observation.events, trace);
            if (!error.empty()) fail(graph.name + ": " + error);
            if (workers == 1)
                for (std::size_t a = 0; a != count; ++a)
                    for (std::size_t b = a + 1; b != count; ++b)
                        if (overlap(trace, a, b)) fail("W1 recorded two active native threads");
            print_trace(graph, trace);
            if (!trace_control && count) {
                auto corrupted = observation.events;
                corrupted.pop_back();
                Trace discarded;
                const auto rejected = check_trace(graph, expected, corrupted, discarded);
                if (rejected.empty()) fail("trace corruption escaped comparator");
                std::printf("control\ttrace-corruption\trejected=%s\n", rejected.c_str());
                trace_control = true;
            }
        }
        ++cases;
        tasks += count;
    }
    if (!output_control || (traced && !trace_control)) fail("missing corruption control");
    if (!native && workers == 4 && !wf__par_pool_active()) fail("requested WF pool is absent");
    std::printf("PASS\tcases=%zu\ttask_rows=%zu\ttrace=%u\n", cases, tasks,
                static_cast<unsigned>(traced));
    return 0;
}
unsigned selected_workers() {
    const char *value = std::getenv("WF_WORKERS");
    if (value && std::string(value) == "1") return 1;
    if (value && std::string(value) == "4") return 4;
    fail("initial qualification requires WF_WORKERS=1 or 4");
}
} // namespace

// The entry result feeds the recurrence and the exit consumes its result.
// Consequently pure arithmetic cannot move outside these diagnostic markers.
// Their atomics perturb scheduling and are never a performance instrument.
extern "C" [[gnu::noinline]] std::uint64_t dag_trace_begin(
    std::uint64_t id, std::uint64_t steps, std::uint64_t seed) {
    // The WF call crosses translation units. The volatile readback also keeps
    // the same-TU native recurrence dependent on this call's returned value.
    volatile std::uint64_t observed_seed = seed;
    event(id, true, seed, steps);
    return observed_seed;
}
extern "C" [[gnu::noinline]] void dag_trace_end(std::uint64_t id, std::uint64_t value) {
    event(id, false, value, 0);
}
namespace {
void native_graph(const Graph &graph, const std::uint64_t *costs,
                   TaskCell *output, std::uint64_t *receipts,
                   tbb::task_arena &arena, bool traced) {
    const auto incoming = predecessors(graph);
    arena.execute([&] {
        tbb::flow::graph flow;
        using Node = tbb::flow::continue_node<tbb::flow::continue_msg>;
        std::vector<std::unique_ptr<Node>> nodes;
        nodes.reserve(graph.costs.size());
        for (std::size_t id = 0; id != graph.costs.size(); ++id) {
            nodes.push_back(std::make_unique<Node>(flow, [&, id](tbb::flow::continue_msg) {
                std::uint64_t mix = 0;
                for (std::size_t parent : incoming[id]) mix = rotate(mix, 7) ^ output[parent].value;
                const std::uint64_t seed = traced ? dag_trace_begin(id, costs[id], graph.seed)
                                                  : graph.seed;
                std::uint64_t value = (seed + id) ^ mix;
                for (std::uint64_t step = 0; step < costs[id]; ++step) {
                    const std::uint64_t shifted = (value << 13) | (value >> 51);
                    value = shifted * UINT64_C(6364136223846793005);
                    value += UINT64_C(1442695040888963407);
                    value ^= step;
                }
                if (traced) dag_trace_end(id, value);
                output[id].value = value;
                ++output[id].evaluations;
                if (graph.family == Family::notify && id >= 2)
                    for (std::size_t parent : incoming[id]) {
                        receipts[2 * (id - 2)] |= UINT64_C(1) << parent;
                        ++receipts[2 * (id - 2) + 1];
                    }
                return tbb::flow::continue_msg{};
            }));
        }
        // Registration supplies the continue_node predecessor threshold.
        for (auto edge : graph.edges) tbb::flow::make_edge(*nodes[edge.first], *nodes[edge.second]);
        for (std::size_t id = 0; id != graph.costs.size(); ++id)
            if (incoming[id].empty()) nodes[id]->try_put(tbb::flow::continue_msg{});
        flow.wait_for_all();
    });
}
} // namespace
extern "C" int wf__main_body(int, char **) {
    try { return run_matrix(false, selected_workers()); }
    catch (const std::exception &error) {
        std::fprintf(stderr, "dag-fanin: %s\n", error.what());
        return 2;
    }
}
int main(int argc, char **argv) {
    try {
        if (argc != 2) fail("usage: dag_fanin_{plain,trace} wf|tbb");
        if (std::string(argv[1]) == "tbb") return run_matrix(true, selected_workers());
        if (std::string(argv[1]) == "wf") return wf__floor_run(argc, argv);
        fail("engine must be wf or tbb");
    } catch (const std::exception &error) {
        std::fprintf(stderr, "dag-fanin: %s\n", error.what());
        return 2;
    }
}
