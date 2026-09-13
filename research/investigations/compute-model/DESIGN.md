# Compute expression and cost

Can ordinary functions, proved partitions, and source-ordered phases express
useful parallel algorithms without imposing an inferior algorithm or hiding a
runtime cost? The range-loan consumers establish runtime-width stencil rows
and recursive subdivision, not the broader predictions in the
[concurrency catalog](../io-model/CONCURRENCY-CATALOG.md).

The starting compiler is `d17e7e0d`, after the range-loan and compute measurement
protocol merges. The containers handover added at that revision describes a
separate, unmerged implementation; it is not this experiment's language or
compiler. The active specification and executable cases remain authoritative.

## Consumers and discriminating criteria

These criteria are recorded before the new experiments. All source programs
use the ordinary compiler path. Correctness is established independently of
the parallel implementation, before timing. Input contracts and required
results remain fixed across comparisons.

| Consumer | Required behavior | What the experiment distinguishes |
|---|---|---|
| Prefix sum | Exclusive unsigned 64-bit scan modulo 2^64, including empty input, a one-element input, uneven final blocks, and runtime block sizes. | Ordinary helper calls and exclusive block views must express phase-dependent writeback without a source-expanded worker count. Compare the same source sequentially and in parallel, and against a native scan with the same result. |
| Histogram | Count input keys into a runtime-sized bucket set, including repeated keys, skew, and uneven input partitions. | Data-dependent writes inside each iteration's private range must remain safe without suppressing the enclosing partition's permission. Compare privatized storage and merge costs with a useful native algorithm, not only a serial atomic-free reference. |
| Irregular partition/scatter | Sort or compact actual input with preserved elements and a separately checked result. | Determine whether disjoint output work can be exposed using existing ranges and ordinary helpers, whether an additional finite proof family is needed, or whether phase boundaries force a serial bottleneck. A denied loop alone does not prove the algorithm inexpressible. |
| Sparse graph traversal | Visit the reachable vertices on both narrow-frontier/high-diameter and broad-frontier graphs. | Compare work and memory as well as elapsed time with a sparse native traversal. A dense full-vertex pass per level is not an acceptable replacement for a sparse algorithm merely because its loop parallelizes. |

Add a partitioned-build consumer only if it exercises an obligation the scan,
histogram, and irregular consumers do not. Block sizes are data decomposition,
not a worker count unrolled in source. No writer scheduling API is assumed.

## Runtime cost attribution

The [range-loan measurements](../range-loans/DESIGN.md#corrected-native-measurements-2026-09-13)
report a 15--20 percent single-worker stencil cost and a size-dependent grain
cliff. Reestablish a baseline using the merged
[compute-bench](../../experiments/compute-bench/README.md) protocol before
choosing a change. Keep the original, large, and small fixtures; add a size
sweep around the split threshold rather than replacing an adverse fixture.

Compare identical source under `--no-overlap` and plain `--par`, then isolate
the responsible lowering or runtime mechanism. Retain both favorable and
adverse pairs, actual grants, wall time, process CPU, toolchain, flags, worker
count, and input dimensions. Any A/B control is identified as a control, never
presented as the default program. A proposed general improvement must preserve
outputs, explain the observed cost, and survive the existing compute kernels;
a layout-sensitive difference or a win obtained only by selecting a different
input is not a causal result. Native comparisons retain their disclosed flag
differences and do not establish a universal performance bound.

## Interpretation and scope

Separate compiler defects, specified-but-unimplemented behavior, finite proof
domain gaps, and costs of the source execution model. In particular, a
sequential scatter in one formulation does not establish that all scatter or
all parallel sorting requires a new language mechanism. Try a useful ordinary
representation and retain the concrete failure if it cannot meet the same
requirements. Conversely, extra asymptotic work, a compulsory serial phase,
or excess storage is not dismissed as a writer problem.

Shared foundations are added only for an actual compute consumer's need.
I/O operation values, cancellation, waiting, connection scheduling, and PAR-3
remain outside this investigation. A compute result that calls for a model
revision is recorded with its alternatives and uncertainty as a design
amendment; the existing tree changes only after the owner's ruling.

The source and native comparisons belong in the existing compute-bench
bundle, with its maintained correctness targets. This investigation owns the
interpretation and rejected alternatives; merge or remove replaceable prose
when superseded, retaining useful dated measurements at their evidence source.
