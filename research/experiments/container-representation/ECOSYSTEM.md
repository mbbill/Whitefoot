# Rust and C++ container comparison

## Question and scope

How does the current Whitefoot library perform against ordinary production
Rust and C++ containers when they complete the same application task, and
which observed differences deserve the next investigation?

This experiment starts from main `0f22b026b`, after the complete container
library delivery. It does not select a language amendment or change a library
algorithm. Existing C controls remain attribution tools, not an asserted
performance ceiling. The sources, commands, toolchain identities, raw samples,
and qualifications below must travel with any reported ratio.

| Whitefoot family | Rust baseline | C++ baselines |
|---|---|---|
| GrowVector | `Vec` | `std::vector` |
| Deque | `VecDeque` | `std::deque` |
| HashMap | `std::collections::HashMap` | `std::unordered_map`, `absl::flat_hash_map` |
| PriorityQueue | `BinaryHeap` | `std::priority_queue` and standard heap algorithms where its adapter lacks the operation |
| OrderedMap | `BTreeMap` | `std::map`, `absl::btree_map` |

The first comparison reuses the five existing family drivers and their
independent behavior oracles. Slab, independently retained membership, and
the indexed composite require different comparison APIs and are outside this
first matrix. These synthetic traces provide discriminating costs, not a
real-application workload-frequency distribution.

## Comparison contract

Practical comparisons preserve the requested application outcomes while
allowing each library its ordinary algorithms, representations, capacities,
growth policies, and optimized APIs. Do not implement the Whitefoot algorithm
inside a Rust or C++ wrapper and call it a standard-library comparison. Record
reference stability, iteration order, ownership of replaced/removed values,
logical capacity limits, and cleanup obligations for each trace. An adapter
needed to deliver the requested outcome belongs in its cost; an outcome that
the application does not require must not be imposed just to mimic Whitefoot.

Small scalars and wide inline values are separate cases. Wide bytes alone do
not establish nested-owner performance. Do not add a per-element allocation
to a competitor merely because its value is large. All consumed results must
contribute to the independent oracle, and every allocation must be reclaimed.

Normal optimized builds provide the practical ranking. Retained public-helper
builds, optimized IR, and existing C variants are attribution evidence, with
their changed visibility and ABI conditions stated. Do not infer a causal
percentage by subtracting unrelated whole-trace timings.

Hash maps need two separately labelled questions: the ordinary default hasher
and an aligned hash calculation for attribution. Neither a cheap integer hash
nor a randomized default should silently stand in for the other. Different
table layouts and load policies remain visible in both series.

## Measurement criteria recorded before running

- Rebuild Whitefoot, native C controls, Rust, and C++ against one recorded
  source revision. Pin external-library versions; record compiler versions,
  flags, target, and allocator conditions. Historical samples are not the
  denominator for this run.
- Start with small, medium, and large populations already in each family
  driver, with scalar and wide values. Keep its complete trace, fixed input
  generation, correctness oracle, and consumption of results. Record any
  necessary trace change before using its measurements.
- Separate build time, correctness execution, allocation accounting, and
  timing. Practical timed builds use ordinary allocation without live
  accounting counters; separately instrumented executions report allocation
  counts, requested bytes, and peak live requested bytes. A requested-byte
  peak is not process RSS or allocator-resident memory.
- Use warmup and repeated paired samples with rotating implementation order,
  including a reverse-order cohort. Preserve all samples. A short or unstable
  cell is inconclusive until a longer bounded run resolves it.
- Report each workload and payload independently. Use whole-trace elapsed
  time and per-operation normalization only where the denominator is defined;
  do not manufacture isolated lookup or growth latency by subtracting setup.
  Growth-focused traces locate a follow-up question, not a measured p99 pause.
- A repeated gap above 10% in both order cohorts is a triage signal, not a
  correctness gate or universal performance requirement. Smaller differences
  remain descriptive. Large memory differences and missing efficient APIs
  can justify investigation even where elapsed time is close.
- Attribute an observed gap only as far as evidence permits: application
  contract, hashing, algorithm, representation, allocation, or emitted code.
  A plausible explanation without a controlled discriminator stays a
  hypothesis. Record actionable unresolved questions in `docs/todo.md`.

## Reproduction and results

Implementation and measurements are pending. Explicit experiment targets will
be added to the existing container-representation Makefile; none will become
a dependency of canonical correctness CI. The final record will link the
family sources and raw samples, state validated coverage and limitations,
and rank follow-up investigations without silently selecting optimizations.
