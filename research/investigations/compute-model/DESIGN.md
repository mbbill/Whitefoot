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

## Runtime division images

The initial prefix consumer on `bf660d89` stops at `blocks * block_size`
with OP-2 after computing `blocks = count / block_size`. The specification's
S7 quotient facts require a written literal divisor. Runtime division therefore
provides neither the bound needed to establish the product's ordinary domain
from the consumer's bounded inputs nor the relation between the covered blocks
and the input extent. This is a language proof-domain gap, before permission
or scheduling is considered.

The selected extension is a fixed value-image family. A discharged unsigned
exact division of admitted terms or constants publishes `quotient <= dividend`
for runtime divisors as well as literals. Its captured immutable value images
may also justify `product <= dividend` when a later independently admitted
exact multiplication uses that quotient and the same divisor values, in either
operand order. Literal-divisor scaled affine images retain their existing
behavior. The product's domain must still discharge normally before its value
gets this consequence; this extension is not a complete integer arithmetic
solver and does not add a new product-domain route.

The reason is the division identity `dividend = quotient * divisor + remainder`
with a positive unsigned divisor and a nonnegative remainder. The checker
matches the exact captured images, not current binding spellings. Copies may
retain a value, but replacing an operand cannot retarget the old theorem.
The retained product consequence cites both the division and the checked
multiplication. No nonlinear proposition is published or searched.

The alternatives are keeping a literal block size, requiring callers to
provide a precomputed partition with additional contracts, changing the scan
algorithm to avoid division, or adding runtime tests for true arithmetic
relations. They do not address the general runtime partition calculation;
the last also conflicts with the source-proof boundary. A wider theorem or
general nonlinear certificate language has no necessity established here.
The proposed automatic-facts amendment records this choice pending owner
ruling. Premise-removal cases will cover changed quotient, divisor and
dividend values, signed operations, unproved division domains, and branch joins.

The specification amendment changes ENT-3.S7 and DIAG-2, with no new numbered
rule, token, production, operation spelling, or exception. It archives the
outgoing v0.55 bytes and declares v0.56. The target-mapping review carries its
rows forward because these facts erase and change no ABI, runtime operation,
release action, or target-domain obligation. Six new conformance cases cover
direct and committed products, surviving aliases, three invalid retargetings,
and the requirement that a product discharge its own domain first; no old
normative expectation is weakened.

## Initial blocked consumers

The prefix program uses two independent maps separated by a sequential scan of
the block totals. Each output helper computes the ordinary sequential
recurrence within its assigned range. The histogram's outer map hands one
counter range to each input block; data-dependent writes remain sequential
inside that helper, and the enclosing block loop is independently permitted.
Both use runtime dimensions and process a final partial block.

The histogram records the complete blocks' counter extent and then adds the
tail row's width. Reusing that actual product also makes its workspace-bound
certificate fold through the existing product identities. No new range or
certificate rule is needed for either consumer. The scan helper requires the
input view to fit in its output view; every call supplies equally sized views,
and the returned allocation has exactly the input length.

Independent C oracles use one direct pass, without the block decomposition.
The native test covers 48 prefix configurations and 192 histogram
configurations in both compiler modes at one, two, and four workers, checking
every output element, the result length, and unchanged input. These checks
passed before timing. Semantic tests separately establish that the intended
outer loops are eligible and the inner recurrences are denied; an unrelated
parallel initialization loop is not used as evidence for an algorithm stage.

## Pool-off measurement path

Inspection before the new timing runs found that the compute-bench adapters
call `wf_<kernel>` directly in a parallel module. The emitted command entry
instead calls the sequential clone when `wf__par_pool_active()` is false.
Thus a one-worker table row currently measures a different entry path from
an ordinary compiled command. The loop splitter returns a zero budget there,
but the adapter still enters the outlined parallel lowering.

The discriminating control is the same compiled module, runtime, input, and
comparison process, with the adapter calling the already-emitted sequential
clone at one worker. Compare both paths with `--no-overlap`; inspect the
optimized bodies as well as elapsed time. A difference removed by selecting
the command's world is a measurement-path cost. Any remaining difference
requires separate lowering attribution. Neither conclusion is established by
the source inspection alone. Before adopting a correction, check that the
adapter selects the same world as command entry at multiple workers and that
every existing kernel still computes its independently fixed result.
