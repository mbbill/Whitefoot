// Runtime-DAG investigation only: complete-output Kahn oracle, passive task
// events, the pinned oneTBB flow-graph reference and a selected WF cost mode.
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
#include <time.h>
#include <utility>
#include <vector>
#ifdef __APPLE__
#include <mach/mach.h>
#endif

static_assert(TBB_VERSION_MAJOR == 2023 && TBB_VERSION_MINOR == 1 &&
              TBB_VERSION_PATCH == 0, "use the existing pinned oneTBB cache");
struct TaskCell { std::uint64_t value, evaluations; };
static_assert(sizeof(TaskCell) == 16 && offsetof(TaskCell, evaluations) == 8,
              "the LLVM adapter exposes two consecutive u64 fields");
struct RuntimeResult { std::uint64_t status, rounds, notices; };
static_assert(sizeof(RuntimeResult) == 24 && offsetof(RuntimeResult, rounds) == 8 &&
              offsetof(RuntimeResult, notices) == 16,
              "the runtime adjacency result exposes three consecutive u64 fields");
extern "C" {
int wf__floor_run(int, char **);
void dag_probe_spine(std::uint64_t, const std::uint64_t *, std::uint64_t,
                     TaskCell *, std::uint64_t);
void dag_probe_spine_phased(std::uint64_t, const std::uint64_t *, std::uint64_t,
                            TaskCell *, std::uint64_t);
void dag_probe_spine_phased_scalar(std::uint64_t, std::uint64_t, std::uint64_t,
                                   TaskCell *, std::uint64_t);
void dag_probe_notify(std::uint64_t, const std::uint64_t *, TaskCell *,
                      std::uint64_t *, std::uint64_t);
void dag_probe_n(std::uint64_t, const std::uint64_t *, TaskCell *, std::uint64_t);
void dag_probe_wide_four(const std::uint64_t *, TaskCell *, std::uint64_t);
void dag_probe_runtime(std::uint64_t, std::uint64_t, std::uint64_t,
                       const std::uint64_t *, std::uint64_t, const std::uint64_t *,
                       std::uint64_t, TaskCell *, std::uint64_t, RuntimeResult *);
unsigned dag_probe_trace_image();
unsigned long wf__par_grants();
int wf__par_pool_active();
}
namespace {
namespace tbb = oneapi::tbb;
constexpr std::uint64_t expensive = 65536;
constexpr std::uint64_t seed_value = UINT64_C(0xffffffffffffffe7);
constexpr std::uint64_t guard_value = UINT64_C(0xbadc0ffee0ffee17);
struct GuardedRuntimeResult {
    std::uint64_t before = guard_value;
    RuntimeResult value{guard_value, guard_value, guard_value};
    std::uint64_t after = guard_value;
    bool intact() const { return before == guard_value && after == guard_value; }
};
enum class Family { spine, notify, n, runtime, wide };
enum class Engine { whitefoot, tbb, phased, scalar, runtime_loop, runtime_tree, runtime_tbb };
struct Graph {
    Family family;
    std::string name;
    unsigned mode = 0;
    std::uint64_t argument = 0, seed = seed_value;
    std::uint64_t scalar_leaf_steps = 1;
    std::vector<std::uint64_t> costs;
    std::vector<std::uint64_t> successors;
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
bool native_engine(Engine engine) {
    return engine == Engine::tbb || engine == Engine::runtime_tbb;
}
bool runtime_engine(Engine engine) {
    return engine == Engine::runtime_loop || engine == Engine::runtime_tree ||
           engine == Engine::runtime_tbb;
}
const char *engine_name(Engine engine) {
    switch (engine) {
    case Engine::whitefoot: return "wf";
    case Engine::tbb: return "tbb";
    case Engine::phased: return "wf-phased";
    case Engine::scalar: return "wf-scalar";
    case Engine::runtime_loop: return "wf-runtime-loop";
    case Engine::runtime_tree: return "wf-runtime-tree";
    case Engine::runtime_tbb: return "tbb-runtime";
    }
    fail("unknown engine");
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
std::vector<Graph> runtime_fixtures(Engine engine) {
    std::vector<Graph> result;
    auto append = [&](Graph graph) {
        const std::size_t count = graph.costs.size();
        graph.successors.assign(2 * count, count);
        std::vector<std::size_t> used(count);
        for (auto edge : graph.edges) {
            if (edge.first >= edge.second || edge.second >= count || used[edge.first] == 2)
                fail("invalid runtime fixture edge");
            graph.successors[2 * edge.first + used[edge.first]++] = edge.second;
        }
        if (engine == Engine::runtime_tbb) {
            result.push_back(std::move(graph));
            return;
        }
        if (count == 0) {
            graph.name += "-c0";
            result.push_back(std::move(graph));
            return;
        }
        for (unsigned owners : {1, 2, 4}) {
            if (owners > count) continue;
            Graph owned = graph;
            owned.argument = owners;
            owned.name += "-c" + std::to_string(owners);
            result.push_back(std::move(owned));
        }
    };
    for (std::size_t count = 0; count <= 5; ++count) {
        std::vector<std::pair<std::size_t, std::size_t>> possible;
        for (std::size_t source = 0; source != count; ++source)
            for (std::size_t destination = source + 1; destination != count; ++destination)
                possible.emplace_back(source, destination);
        const unsigned combinations = 1U << possible.size();
        for (unsigned mask = 0; mask != combinations; ++mask) {
            Graph graph{Family::runtime, "runtime-" + std::to_string(count) + "-" +
                                            std::to_string(mask)};
            graph.costs.assign(count, 1);
            std::vector<unsigned> in(count), out(count);
            bool valid = true;
            for (std::size_t bit = 0; bit != possible.size(); ++bit) {
                if (!(mask & (1U << bit))) continue;
                const auto edge = possible[bit];
                if (++out[edge.first] > 2 || ++in[edge.second] > 2) {
                    valid = false;
                    break;
                }
                graph.edges.push_back(edge);
            }
            if (!valid) continue;
            if (count == 0) graph.name = "runtime-empty";
            if (count == 1) graph.name = "runtime-singleton";
            if (count == 3 && mask == 7) graph.name = "runtime-triangle";
            if (count == 4 && mask == 33) graph.name = "runtime-disconnected";
            append(std::move(graph));
        }
    }
    Graph reverse{Family::runtime, "runtime-reverse-arrival"};
    reverse.costs.assign(8, 1);
    reverse.edges = {{0, 2}, {2, 6}, {4, 6}};
    append(std::move(reverse));
    for (bool costly : {false, true}) {
        Graph progress{Family::runtime, costly ? "runtime-progress-costly" : "runtime-progress-unit"};
        progress.costs.assign(6, 1);
        if (costly) progress.costs[0] = progress.costs[3] = expensive;
        progress.edges = {{0, 2}, {1, 3}, {1, 4}, {2, 4}, {3, 5}, {4, 5}};
        append(std::move(progress));
    }
    return result;
}
std::vector<Graph> fixtures(Engine engine) {
    if (runtime_engine(engine)) return runtime_fixtures(engine);
    std::vector<Graph> result;
    for (std::uint64_t length : {0, 1, 2, 4, 7, 8, 9, 16, 32}) {
        for (unsigned profile = 0; profile != 4; ++profile) {
            if (length == 0 && profile >= 2) continue;
            if (engine == Engine::scalar &&
                ((length != 0 && length != 1 && length != 32) || profile >= 2)) continue;
            Graph graph{Family::spine, "spine-" + std::to_string(length) + "-" +
                                         std::to_string(profile)};
            graph.argument = length;
            graph.scalar_leaf_steps = profile == 1 ? expensive : 1;
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
    if (engine == Engine::phased || engine == Engine::scalar) return result;
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
        for (unsigned mode = 0; mode != (engine == Engine::tbb ? 1U : 4U); ++mode)
            result.push_back(n_graph(costs, mode));
    for (bool costly : {false, true}) {
        Graph graph{Family::wide, costly ? "wide-costly" : "wide-cheap"};
        graph.costs.assign(4, costly ? expensive : 1);
        result.push_back(std::move(graph));
    }
    return result;
}
RuntimeResult routing_oracle(const Graph &graph) {
    const std::size_t count = graph.costs.size();
    if (count == 0) return {0, 0, 0};
    const std::size_t owners = graph.argument;
    if (owners == 0 || owners > count) fail("routing oracle owner count outside domain");
    const std::size_t stride = count / owners;
    auto owner = [&](std::size_t id) { return std::min(id / stride, owners - 1); };
    std::vector<std::uint64_t> crossing_depth(count);
    const auto incoming = predecessors(graph);
    RuntimeResult result{0, 1, 0};
    for (std::size_t id = 0; id != count; ++id) {
        for (std::size_t parent : incoming[id]) {
            if (parent >= id) fail("routing oracle requires topological IDs");
            const std::uint64_t crossing = owner(parent) != owner(id);
            result.notices += crossing;
            crossing_depth[id] = std::max(crossing_depth[id], crossing_depth[parent] + crossing);
        }
        result.rounds = std::max(result.rounds, crossing_depth[id] + 1);
    }
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
    if (graph.family == Family::notify || graph.family == Family::n) {
        const bool d_while_a = trace.thread[3] != trace.thread[0] &&
            trace.begin[0] < trace.begin[3] && trace.begin[3] < trace.end[0];
        const bool c_while_d = trace.thread[2] != trace.thread[3] &&
            trace.begin[3] < trace.begin[2] && trace.begin[2] < trace.end[3];
        std::printf("\tsources=%u\towners=%u\td_while_a=%u\tc_while_d=%u",
                    static_cast<unsigned>(overlap(trace, 0, 1)),
                    static_cast<unsigned>(overlap(trace, 2, 3)),
                    static_cast<unsigned>(d_while_a), static_cast<unsigned>(c_while_d));
    }
    if (graph.name.rfind("runtime-progress-", 0) == 0) {
        const bool early = trace.thread[3] != trace.thread[0] &&
            trace.begin[0] < trace.begin[3] && trace.begin[3] < trace.end[0];
        std::printf("\ttask0_task3=%u\ttask3_while_task0=%u",
                    static_cast<unsigned>(overlap(trace, 0, 3)), static_cast<unsigned>(early));
    }
    if (graph.name.rfind("runtime-reverse-arrival", 0) == 0)
        std::printf("\ttask4_before_task2=%u",
                    static_cast<unsigned>(trace.end[4] < trace.begin[2]));
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
void whitefoot_graph(Engine engine, const Graph &graph, const std::uint64_t *costs,
                      const std::uint64_t *successors, TaskCell *output,
                      std::uint64_t *receipts, RuntimeResult &report) {
    if (graph.family == Family::runtime) {
        dag_probe_runtime(engine == Engine::runtime_tree ? 1 : 0, graph.argument,
                           graph.costs.size(), costs, graph.successors.size(), successors,
                           graph.costs.size() + 1, output, graph.seed, &report);
    } else if (graph.family == Family::spine) {
        if (engine == Engine::scalar) {
            dag_probe_spine_phased_scalar(graph.argument, graph.scalar_leaf_steps,
                                          graph.costs.size(), output, graph.seed);
        } else {
            const auto entry = engine == Engine::phased ? dag_probe_spine_phased : dag_probe_spine;
            entry(graph.argument, costs, graph.costs.size(), output, graph.seed);
        }
    }
    else if (graph.family == Family::notify)
        dag_probe_notify(graph.argument, costs, output, receipts, graph.seed);
    else if (graph.family == Family::wide)
        dag_probe_wide_four(costs, output, graph.seed);
    else dag_probe_n(graph.mode, costs, output, graph.seed);
}
bool equal(TaskCell a, TaskCell b) {
    return a.value == b.value && a.evaluations == b.evaluations;
}
std::size_t malformed_runtime_controls(Engine engine) {
    const std::uint64_t form = engine == Engine::runtime_tree ? 1 : 0;
    struct Invalid {
        const char *name;
        std::uint64_t form, owners;
        std::vector<std::uint64_t> costs, successors;
    };
    const std::vector<Invalid> invalid{
        {"short-successors", form, 1, {1, 1}, {2, 2, 2}},
        {"long-successors", form, 1, {1, 1}, {2, 2, 2, 2, 2}},
        {"out-of-range", form, 1, {1, 1}, {3, 2, 2, 2}},
        {"self-edge", form, 1, {1, 1}, {0, 2, 2, 2}},
        {"backward-edge", form, 1, {1, 1}, {2, 2, 0, 2}},
        {"duplicate-edge", form, 1, {1, 1}, {1, 1, 2, 2}},
        {"excess-indegree", form, 2, {1, 1, 1, 1}, {3, 4, 3, 4, 3, 4, 4, 4}},
        {"invalid-form", 2, 1, {1, 1}, {2, 2, 2, 2}},
        {"zero-owners", form, 0, {1, 1}, {2, 2, 2, 2}},
        {"excess-owners", form, 3, {1, 1}, {2, 2, 2, 2}},
        {"unrepresentable-owner-matrix", form, ~UINT64_C(0), {1, 1}, {2, 2, 2, 2}},
        {"empty-with-owner", form, 1, {}, {}},
        {"empty-with-successor", form, 0, {}, {0}}
    };
    for (const auto &control : invalid) {
        std::vector<std::uint64_t> costs(control.costs.size() + 2, guard_value);
        std::copy(control.costs.begin(), control.costs.end(), costs.begin() + 1);
        const auto original_costs = costs;
        std::vector<std::uint64_t> successors(control.successors.size() + 2, guard_value);
        std::copy(control.successors.begin(), control.successors.end(), successors.begin() + 1);
        const auto original_successors = successors;
        Rows output(control.costs.size() + 3, TaskCell{guard_value, ~guard_value});
        const Rows original_output = output;
        GuardedRuntimeResult returned;
        RuntimeResult &report = returned.value;
        observation.events.clear();
        observation.next.store(0);
        observation.overflow.store(false);
        dag_probe_runtime(control.form, control.owners, control.costs.size(), costs.data() + 1,
                           control.successors.size(), successors.data() + 1,
                           control.costs.size() + 1, output.data() + 1, seed_value, &report);
        if (!returned.intact() || report.status != 1 || report.rounds != 0 || report.notices != 0)
            fail(std::string(control.name) + ": wrong validation result");
        if (costs != original_costs || successors != original_successors ||
            !std::equal(output.begin(), output.end(), original_output.begin(), equal))
            fail(std::string(control.name) + ": validation error changed input/output");
        if (observation.next.load() != 0 || observation.overflow.load())
            fail(std::string(control.name) + ": validation error executed a task");
        std::printf("invalid\t%s\tstatus=1\trounds=0\tnotices=0\toutput_unchanged=1"
                    "\tinputs_unchanged=1\tevents=0\n", control.name);
    }
    return invalid.size();
}
void print_routing(const Graph &graph, const RuntimeResult &report) {
    const std::uint64_t count = graph.costs.size(), owners = graph.argument;
    const std::uint64_t stride = count == 0 ? 0 : count / owners;
    const std::uint64_t cells = owners * owners, rounds = report.rounds;
    std::printf("routing\t%s\towners=%llu\tstride=%llu\tlast_owner_size=%llu"
                "\tstatus=%llu\trounds=%llu\tnotices=%llu\taux_words=%llu"
                "\tseparate_allocations=%u\tresult_bytes=%zu\n", graph.name.c_str(),
                static_cast<unsigned long long>(owners), static_cast<unsigned long long>(stride),
                static_cast<unsigned long long>(count == 0 ? 0 : count - (owners - 1) * stride),
                static_cast<unsigned long long>(report.status),
                static_cast<unsigned long long>(rounds), static_cast<unsigned long long>(report.notices),
                static_cast<unsigned long long>(12 * count + 2 * cells + 2 * owners),
                count == 0 ? 0U : 7U, sizeof(RuntimeResult));
    // Counts of written source operations, not physical allocation high-water marks
    // or optimized native load/store counts. The returned notices/rounds are checked.
    std::printf("routing-model\t%s\tstate_rows_initialized=%llu\tnotice_rows_initialized=%llu"
                "\thead_cells_initialized=%llu\tready_heads_initialized=%llu"
                "\treports_initialized=%llu\toutput_cells_initialized=%llu"
                "\tvalidation_successor_reads=%llu\ttask_successor_reads=%llu"
                "\tnotice_successor_reads=%llu\troot_state_visits=%llu\ttask_ready_visits=%llu"
                "\tcost_reads=%llu\tdrain_calls=%llu"
                "\thead_resets=%llu\thead_inspections=%llu\tnotice_writes=%llu"
                "\tnotice_visits=%llu\treport_writes=%llu\treport_reads=%llu\n",
                graph.name.c_str(), static_cast<unsigned long long>(count),
                static_cast<unsigned long long>(4 * count), static_cast<unsigned long long>(2 * cells),
                static_cast<unsigned long long>(owners), static_cast<unsigned long long>(owners),
                static_cast<unsigned long long>(count), static_cast<unsigned long long>(4 * count),
                static_cast<unsigned long long>(2 * count), static_cast<unsigned long long>(report.notices),
                static_cast<unsigned long long>(count), static_cast<unsigned long long>(count),
                static_cast<unsigned long long>(count), static_cast<unsigned long long>(owners * rounds + report.notices),
                static_cast<unsigned long long>(cells * rounds),
                static_cast<unsigned long long>(cells * rounds), static_cast<unsigned long long>(report.notices),
                static_cast<unsigned long long>(report.notices), static_cast<unsigned long long>(owners * rounds),
                static_cast<unsigned long long>(owners * rounds));
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
int run_matrix(Engine engine, unsigned workers) {
    const bool native = native_engine(engine);
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
                engine_name(engine), traced ? "trace" : "plain", workers);
    std::printf("accounting\ttask_cell_bytes=%zu\tevent_bytes=%zu\tphysical_peak=not-measured\n",
                sizeof(TaskCell), sizeof(Event));
    bool output_control = false, trace_control = false;
    std::size_t cases = 0, tasks = 0;
    for (const Graph &graph : fixtures(engine)) {
        const Expected expected = oracle(graph);
        const std::size_t count = graph.costs.size();
        const bool runtime = graph.family == Family::runtime;
        const TaskCell guard{guard_value, ~guard_value};
        Rows storage(count + (runtime ? 3 : 2));
        storage.front() = storage.back() = guard;
        if (runtime) storage[count + 1] = guard;
        if (runtime && !native)
            std::fill(storage.begin() + 1, storage.begin() + 1 + count, guard);
        std::vector<std::uint64_t> costs(count + 2, guard_value);
        std::copy(graph.costs.begin(), graph.costs.end(), costs.begin() + 1);
        const auto original_costs = costs;
        std::vector<std::uint64_t> successors(graph.successors.size() + 2, guard_value);
        std::copy(graph.successors.begin(), graph.successors.end(), successors.begin() + 1);
        const auto original_successors = successors;
        GuardedRuntimeResult returned;
        RuntimeResult &report = returned.value;
        std::array<std::uint64_t, 6> receipt_storage{};
        receipt_storage.front() = receipt_storage.back() = guard_value;
        observation.events.assign(traced ? 2 * count : 0, Event{});
        observation.next.store(0);
        observation.overflow.store(false);
        // The runtime's legacy API name returns successful lane steals.
        const unsigned long steals_before = native ? 0 : wf__par_grants();
        if (native) native_graph(graph, costs.data() + 1, storage.data() + 1,
                                  receipt_storage.data() + 1,
                                  *arena, traced);
        else whitefoot_graph(engine, graph, costs.data() + 1, successors.data() + 1,
                               storage.data() + 1, receipt_storage.data() + 1, report);
        if (runtime && !native) {
            const auto routing = routing_oracle(graph);
            if (!returned.intact() || report.status != 0 || report.rounds != routing.rounds ||
                report.notices != routing.notices || report.rounds > graph.argument)
                fail(graph.name + ": routing status/round/notice mismatch");
        }
        Rows actual(storage.begin() + 1, storage.begin() + 1 + count);
        const auto mismatch = compare_rows(actual, expected.rows);
        if (!mismatch.empty()) fail(graph.name + ": " + mismatch);
        if (!equal(storage.front(), guard) || !equal(storage.back(), guard) ||
            (runtime && !equal(storage[count + 1], guard)) ||
            receipt_storage.front() != guard_value || receipt_storage.back() != guard_value)
            fail(graph.name + ": output boundary changed");
        if (costs != original_costs)
            fail(graph.name + (engine == Engine::scalar ? ": oracle cost metadata changed"
                                                       : ": input changed"));
        if (successors != original_successors) fail(graph.name + ": successor input changed");
        for (std::size_t i = 0; i != 4; ++i)
            if (receipt_storage[i + 1] != expected.receipts[i])
                fail(graph.name + ": notification receipt mismatch");
        const unsigned long steals = native ? 0 : wf__par_grants() - steals_before;
        if (!native && workers == 1 && steals != 0) fail("W1 recorded stolen work");
        std::printf("case\t%s\ttasks=%zu\tedges=%zu\twork_rounds=%llu\tcritical_rounds=%llu"
                    "\tlevel_rounds=%llu\toutput_bytes=%zu\tinput_cost_bytes=%zu"
                    "\tsource_notice_init_slots=%u\tsource_notice_inspections=%u"
                    "\tobserver_bytes=%zu\tactual_steals=%lu\n",
                    graph.name.c_str(), count, graph.edges.size(),
                    static_cast<unsigned long long>(expected.work),
                    static_cast<unsigned long long>(expected.critical),
                    static_cast<unsigned long long>(expected.level), count * sizeof(TaskCell),
                    (engine == Engine::scalar ? 1 : count) * sizeof(std::uint64_t),
                    !native && graph.family == Family::notify ? 4U : 0U,
                    !native && graph.family == Family::notify ? 4U : 0U,
                    observation.events.size() * sizeof(Event), steals);
        if (runtime) {
            std::printf("adjacency\t%s\tsuccessor_slots=%zu\tinput_successor_bytes=%zu"
                        "\tinput_numbering=topological\n", graph.name.c_str(),
                        graph.successors.size(), graph.successors.size() * sizeof(std::uint64_t));
            if (!native) print_routing(graph, report);
        }
        if (engine == Engine::scalar)
            std::printf("input\t%s\trepresentation=scalar\tleaf_steps=%llu"
                        "\tsource_cost_bytes=%zu\toracle_cost_bytes=%zu\n",
                        graph.name.c_str(), static_cast<unsigned long long>(graph.scalar_leaf_steps),
                        sizeof(std::uint64_t), count * sizeof(std::uint64_t));
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
    if (runtime_engine(engine) &&
        (cases != (native ? 495U : 1471U) || tasks != (native ? 2399U : 7167U)))
        fail("runtime adjacency matrix cardinality mismatch");
    const std::size_t invalid = runtime_engine(engine) && !native ? malformed_runtime_controls(engine) : 0;
    std::printf("PASS\tcases=%zu\ttask_rows=%zu\ttrace=%u", cases, tasks, static_cast<unsigned>(traced));
    if (runtime_engine(engine)) std::printf("\tinvalid_cases=%zu\n", invalid);
    else std::putchar('\n');
    return 0;
}
unsigned selected_workers() {
    const char *value = std::getenv("WF_WORKERS");
    if (value && std::string(value) == "1") return 1;
    if (value && std::string(value) == "4") return 4;
    fail("initial qualification requires WF_WORKERS=1 or 4");
}
std::uint64_t wall_ns() {
    timespec value{};
    if (clock_gettime(CLOCK_MONOTONIC, &value) || value.tv_sec < 0)
        fail("monotonic clock failed");
    return static_cast<std::uint64_t>(value.tv_sec) * UINT64_C(1000000000) +
           static_cast<std::uint64_t>(value.tv_nsec);
}
#ifdef __APPLE__
constexpr const char *cpu_clock_name = "task_info_live_plus_exited";
std::uint64_t time_ns(time_value_t value) {
    if (value.seconds < 0 || value.microseconds < 0) fail("negative task CPU time");
    return static_cast<std::uint64_t>(value.seconds) * UINT64_C(1000000000) +
           static_cast<std::uint64_t>(value.microseconds) * 1000;
}
std::uint64_t cpu_ns() {
    task_thread_times_info_data_t live{};
    task_basic_info_data_t exited{};
    mach_msg_type_number_t live_count = TASK_THREAD_TIMES_INFO_COUNT;
    mach_msg_type_number_t exited_count = TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), TASK_THREAD_TIMES_INFO,
                  reinterpret_cast<task_info_t>(&live), &live_count) != KERN_SUCCESS ||
        task_info(mach_task_self(), TASK_BASIC_INFO,
                  reinterpret_cast<task_info_t>(&exited), &exited_count) != KERN_SUCCESS)
        fail("all-thread task CPU clock failed");
    return time_ns(live.user_time) + time_ns(live.system_time) +
           time_ns(exited.user_time) + time_ns(exited.system_time);
}
#else
constexpr const char *cpu_clock_name = "CLOCK_PROCESS_CPUTIME_ID";
std::uint64_t cpu_ns() {
    timespec value{};
    if (clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &value) || value.tv_sec < 0)
        fail("process CPU clock failed");
    return static_cast<std::uint64_t>(value.tv_sec) * UINT64_C(1000000000) +
           static_cast<std::uint64_t>(value.tv_nsec);
}
#endif
int run_measurement(int argc, char **argv) {
    if ((argc != 6 && argc != 7) ||
        (std::string(argv[2]) != "baseline" && std::string(argv[2]) != "candidate") ||
        (argc == 7 && std::string(argv[6]) != "slow") ||
        std::string(argv[5]).size() != 1 || argv[5][0] < '0' || argv[5][0] > '4')
        fail("usage: image measure baseline|candidate FIXTURE WIDTH PASS [slow]");
    const unsigned workers = selected_workers();
    if (std::string(argv[4]) != std::to_string(workers)) fail("timing width mismatch");
    if (dag_probe_trace_image() != 0) fail("timing requires the plain image");
    if (workers == 4 && !wf__par_pool_active()) fail("timing WF pool is absent");
    const auto all = fixtures(Engine::whitefoot);
    const auto selected = std::find_if(all.begin(), all.end(), [&](const Graph &graph) {
        const bool admitted = (graph.family == Family::n && graph.mode == 3) ||
            graph.family == Family::wide || graph.name == "spine-8-0" ||
            graph.name == "spine-8-1";
        return admitted && graph.name == argv[3];
    });
    if (selected == all.end()) fail("fixture outside the selected cost matrix");
    const Graph &graph = *selected;
    const Expected expected = oracle(graph);
    const bool cheap = std::all_of(graph.costs.begin(), graph.costs.end(),
                                   [](std::uint64_t cost) { return cost == 1; });
    const unsigned repeats = cheap ? (workers == 1 ? 1048576U : 16384U) : 32U;
    const unsigned actual_repeats = repeats * (argc == 7 ? 2U : 1U);
    const std::size_t count = graph.costs.size();
    const TaskCell guard{guard_value, ~guard_value};
    Rows storage(count + 2, guard);
    std::vector<std::uint64_t> costs(count + 2, guard_value);
    std::copy(graph.costs.begin(), graph.costs.end(), costs.begin() + 1);
    const auto original_costs = costs;
    const auto *input = costs.data() + 1;
    auto *output = storage.data() + 1;
    std::printf("# measure\twall_clock=CLOCK_MONOTONIC\tcpu_clock=%s\tplain=1\n",
                cpu_clock_name);
    for (unsigned sample = 0; sample != 6; ++sample) {
        std::fill(storage.begin() + 1, storage.end() - 1, TaskCell{0, 0});
        const std::uint64_t wall_start = wall_ns();
        const std::uint64_t cpu_start = cpu_ns();
        if (graph.family == Family::n) {
            for (unsigned call = 0; call != actual_repeats; ++call)
                dag_probe_n(3, input, output, graph.seed);
        } else if (graph.family == Family::wide) {
            for (unsigned call = 0; call != actual_repeats; ++call)
                dag_probe_wide_four(input, output, graph.seed);
        } else {
            for (unsigned call = 0; call != actual_repeats; ++call)
                dag_probe_spine(graph.argument, input, count, output, graph.seed);
        }
        const std::uint64_t cpu_end = cpu_ns();
        const std::uint64_t wall_end = wall_ns();
        if (wall_end <= wall_start || cpu_end <= cpu_start) fail("nonpositive clock interval");
        const std::uint64_t wall = wall_end - wall_start, cpu = cpu_end - cpu_start;
        for (std::size_t task = 0; task != count; ++task) {
            if (output[task].value != expected.rows[task].value ||
                output[task].evaluations != actual_repeats)
                fail(graph.name + ": measured batch value/count mismatch");
        }
        if (!equal(storage.front(), guard) || !equal(storage.back(), guard) ||
            costs != original_costs) fail(graph.name + ": measured batch boundary/input changed");
        std::printf("measure\t%s\t%s\t%u\t%s\t%u\t%llu\t%llu\t%u\t%u\t%zu\n",
                    graph.name.c_str(), argv[2], workers, argv[5], sample,
                    static_cast<unsigned long long>(wall),
                    static_cast<unsigned long long>(cpu), repeats, actual_repeats, count);
        std::fflush(stdout);
        if (sample != 0 && (wall < 1000000 || cpu < 1000000))
            fail(graph.name + ": measured interval below the selected 1 ms floor");
    }
    return 0;
}
Engine selected_engine(const std::string &name) {
    for (Engine engine : {Engine::whitefoot, Engine::tbb, Engine::phased, Engine::scalar,
                          Engine::runtime_loop, Engine::runtime_tree, Engine::runtime_tbb})
        if (name == engine_name(engine)) return engine;
    fail("unknown DAG qualification engine: " + name);
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
extern "C" int wf__main_body(int argc, char **argv) {
    try {
        if (std::string(argv[1]) == "measure") return run_measurement(argc, argv);
        return run_matrix(selected_engine(argv[1]), selected_workers());
    }
    catch (const std::exception &error) {
        std::fprintf(stderr, "dag-fanin: %s\n", error.what());
        return 2;
    }
}
int main(int argc, char **argv) {
    try {
        if (argc >= 2 && std::string(argv[1]) == "measure")
            return wf__floor_run(argc, argv);
        if (argc != 2)
            fail("usage: dag_fanin_{plain,trace} wf|tbb|wf-phased|wf-scalar|wf-runtime-loop|wf-runtime-tree|tbb-runtime");
        const Engine engine = selected_engine(argv[1]);
        if (native_engine(engine)) return run_matrix(engine, selected_workers());
        return wf__floor_run(argc, argv);
    } catch (const std::exception &error) {
        std::fprintf(stderr, "dag-fanin: %s\n", error.what());
        return 2;
    }
}
