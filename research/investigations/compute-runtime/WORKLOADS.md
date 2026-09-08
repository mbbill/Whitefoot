# Whitefoot compute workload coverage

Whitefoot needs its own substantive performance programs, not just additional
native implementations of `par_layout.wf`. That small tree-layout example
remains a regression sample. Neither a larger tree nor more repetitions turns
it into representative application coverage.

This document owns WF-side workload selection for the [compute runtime
investigation](README.md). The [reference matrix](BASELINES.md) separately owns
the native comparisons. The inventory below was inspected at
`6cc00984415a39c507fa74897c9269b10beebfee`. The
[variable-input FIR experiment](../../experiments/compute-runtime/README.md#variable-input-causal-fir)
supplies an implemented WF computation with output and
parallel-execution qualification plus dated Linux native calibration. Its
original and grouped-output kernels share the recursive program and exact
result contract. This remains one workload family, with broader coverage still
required below. The [UTF-8 record batch](../../experiments/compute-runtime/README.md#variable-length-utf-8-record-batches)
adds a second computation with variable record boundaries, per-record results
and an actualized flat output map; its native frontier and held-out measurement
remain unqualified. [Mandelbrot point rendering](../../experiments/compute-runtime/README.md#mandelbrot-point-rendering)
adds a third computation: strict floating-point recurrence with runtime iteration
limits, data-dependent early exit and clustered/interleaved imbalance. Its
ordinary WF outer map is actualized and compared with six native schedulers
sharing one C callback. This is independent-point work, not nested composition
or irregular graph traversal, and does not replace the missing rows below.
[Adaptive recursive quadrature](../../experiments/compute-runtime/README.md#adaptive-recursive-quadrature)
adds data-dependent subdivision and nested ordinary sibling calls, with an
explicit-stack binary64 oracle, analytic checks for converged fixtures and
an initial scalar C/WF comparison. The original M1/Linux screens observe
substantial parallel-body slowdown. An opt-in scalar-leaf offer control improves
the heavier M1 cases; its later Linux cohort improves relative to original
parallel execution but does not reproduce the sequential speedup. Publication
and generated-code contributions are not separately isolated. Native recursive
oneTBB/Parlay controls now vary spawn depth against a common scalar C++ kernel,
with the same oracle and subtree-local diagnostic counts. The M1 grain screen
exposes remaining WF losses; a matched-grain native WF extension exercises the
same scalar kernel through pointer/by-value frames and forced slot exhaustion.
Reciprocal native fork-direction controls expose mirrored left/right skew
effects while preserving the ordered result and all oracle work counts.
An opt-in generated sequential-refusal control now tests subtree selection on
actual compiler output; fixed queue occupancy remains an unqualified admission
policy, with input-dependent gains and losses. A separate
[sustained batch panel](../../experiments/compute-runtime/README.md#sustained-batches-and-regional-counters)
now measures joined repeated integrations and process CPU/context switches;
the M1 comparison prioritizes excess parallel CPU work over a large scalar
kernel deficit. Linux regional PMU collection is wired but not yet qualified.
Recursive Rayon, general compiler
grain selection and held-out/native-host confirmation
remain missing. The ten-case screen does not
yet qualify larger application composition and does not replace the application
rows below.
Keep the inventory current as
programs become executable; consolidate it when the suite's implemented coverage
supersedes this selection.

## Application coverage to build

Existing program paths are source seeds, not qualified performance workloads.
Each row needs an ordinary WF implementation of the complete named computation,
variable inputs, a correctness oracle, and independent parallelism evidence.
The text, decode, and filter rows offer existing implementation material for
initial coverage; the ordering and traversal rows expose additional capability
requirements. No single row substitutes for the others.

| WF computation | Existing source to examine | Required extension and performance exposure |
| --- | --- | --- |
| In-memory text search and record processing | [wfgrep](../../../tests/programs/wfgrep.wf), [UTF-8 parser](../../../tests/programs/utf8parse.wf), [percent decoder](../../../tests/programs/percent_decode.wf) | Process batches of variable-length records through validation/search and a declared aggregate or ordered result. Preserve record/chunk boundaries and observable malformed-input behavior. Vary match density, record lengths, encoding and output volume. Exercises scanning, branches, variable work, chunk ownership and merge costs. |
| Independent raw-DEFLATE stream decoding | [dynamic decoder](../../../tests/programs/raw_deflate_dynamic_decode.wf), [boundary cases](../../../tests/programs/raw_deflate_boundary.wf), [vectors](../../../tests/programs/raw_deflate_vectors.wf) | Decode complete independent streams, including block transitions, Huffman table construction, literals and matches, and validate all output bytes. Vary stream sizes and literal/match distributions. Parallelism is across independently decodable streams; do not split an arbitrary DEFLATE stream at invalid boundaries. Exercises per-job state, unequal work, memory traffic and larger task frames. |
| Batched signal/image processing | [FIR filter](../../../tests/programs/fir_filter.wf), [grayscale pixels](../../../tests/programs/grayscale_pixels.wf) | Scale to variable arrays/tiles and a complete multi-stage filter or conversion pipeline with defined edge handling. Test independent outputs, input reuse and intermediate buffers; retain the declared floating-point order/precision. Exercises vectorization, regular loops, tiling, bandwidth, fusion and pipeline barriers. |
| Ordering, partitioning and grouped aggregation | No production-sized WF workload selected in this audit | Implement a real sort/partition/group operation with exact ordering, duplicate and stability contracts where applicable. Include sorted, reverse, duplicate-heavy and skewed inputs. Exercises disjoint writable regions, temporary allocation, load balancing and merge/reduction dependencies. Library-native algorithms may compete end to end; scheduler-only controls fix the algorithm. |
| Irregular traversal or spatial-query batches | [recursive tree](../../../tests/programs/recursive_tree.wf) is only an initial representation example | Build a substantive traversal/query workload over variable, independently generated structures, with useful per-node work and a serial reference. Include skew, depth, sparse results and changing query distributions. This must not become another fixed tiny tree fold. Exercises locality, dependent pointer accesses, nested calls, task granularity and imbalance. |

The retained [zlib kernel study](../../experiments/zlib-core-kernels/README.md)
contains useful dated attribution, but explicitly excludes whole-decoder work
and depends on deferred compiler prototypes. It is not a shortcut to a qualified
current-compiler decode benchmark. Likewise, a source file's presence in the
corpus does not demonstrate useful parallelization or production-scale inputs.

## Inputs and composition are part of the program

- Each workload has a bounded small correctness instance and separately sized
  performance inputs: short calls, cache-resident work, and data larger than the
  measured cache where relevant. Record actual bytes and useful operations.
- Use representative data plus deterministic distributions that stress skew,
  duplicates, boundaries, sparse/dense results and expensive outliers. Freeze
  input identities; hold out input families as well as repetitions from tuning.
- Exercise nested library calls and batches containing short and long jobs
  under one total worker budget. Measure one invocation, sustained execution,
  and bursts separately. Reuse is allowed only when the application has it.
- Keep input loading and final checking outside a declared compute-region
  measurement. Charge necessary parsing, task preparation, intermediate
  allocation, merging and completion inside that region. Report an additional
  end-to-end boundary rather than labeling a leaf-only time an application time.
- Preserve the real result: full decoded bytes, ordered records, sample arrays
  or grouped values. Use independent expected results and checked serial
  implementations; WF serial/parallel agreement alone can miss shared bugs.
  A checksum is a reporting convenience, not the sole correctness oracle.

## Diagnostic cases supplement the applications

Small focused programs still serve a distinct purpose: one-worker parallel-path
overhead, unstolen owner-pop, successful and failed steals, tiny nonzero tasks,
deep recursion, unequal branches, nested joins, idle wakeup and memory bandwidth.
They isolate mechanisms that an application timing cannot explain. Keep those
results separate from application coverage and retain `par_layout.wf` here.

For every substantive WF program, inspect the permission ledger and emitted
code: which independent work is proved, which is actualized, where joins occur,
and whether code generation preserves vectorization and locality. An inability
to express or compile an intended workload is a named language/compiler gap.
Unexpected serialization is an outcome to explain, not grounds for removing
the workload, changing its expected answer, or adding a test-specific lowering.

## Evidence needed before a coverage claim

A row advances through **source seed**, **implemented WF program**, **qualified
correctness and actualization**, and **measured application with native
references**. The filter row has FIR correctness, recursive tile actualization
and a [first Linux native comparison](../../experiments/compute-runtime/README.md#first-linux-calibration-before-static-workers)
for independent blocks. Channel co-scheduling, a multistage pipeline and held-out
performance confirmation remain open. The text/record row has a complete UTF-8
validation/count batch with correctness and map actualization; search, richer
record processing and confirmed performance remain open. The other rows are
source seeds or explicit gaps.
New maintained programs must have a real correctness-test caller and a benchmark
caller in the owning experiment; do not leave disconnected fixtures or scripts.

Publish results per program, input family, size, worker budget and host. Report
time, CPU/work, memory, scaling and applicable latency tails, including adverse
cases. A geometric mean or a win count cannot erase a regression. Coverage of
several distinct real computations can support a scoped confidence claim; a
single layout program, even with many configurations, cannot establish that
WF's compute performance generalizes.
