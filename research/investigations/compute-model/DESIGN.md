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
The native test covers 52 prefix configurations and 208 histogram
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

The first five-pass adapter control on this active desktop was inconclusive:
large-fixture process medians had 17--30 percent MAD, and original-fixture
paired ratios ranged from 0.59 to 2.90. Optimized clones contain no remaining
split-budget or chunk calls; their block order differs from `--no-overlap`.
Neither timing nor that inspection establishes a remaining fixed compiler tax.
The adapter is nevertheless corrected because entering the command's world is
part of what this measurement claims to measure, independently of a speed win.

## Binary-split merge pressure

The next consumer uses ordinary sibling recursive calls for both sorting and
merging. A pivot's binary-search rank determines disjoint output views; this
tests the catalog's assertion that a merge must be sequential after private
block sorts. Independent sorted-output and multiset checks precede timing.
The sequential leaf cutoff is 64 elements, shared with the native algorithm;
it is not a worker count or a tuned scheduler setting.

The initial consumer exposes two separate finite proof boundaries. A binary
search's branch join forgets the relation between independently changed bounds.
Returning one search step's two bounds through an ordinary verified helper
preserves their relation using existing multi-result postconditions, without
changing the algorithm or inserting an impossible-case branch. Recursive
postconditions are deliberately withheld inside their own call-graph component,
so moving the search recursion alone does not supply a proof of its result.

More fundamentally, a merge contract can state
`len_of(first) + len_of(second) <= len_of(output)`, but S4 only projects a
comparison with individual L0 operands. The body cannot use its own admitted
sum bound to prove the merged extent. The selected extension captures the
existing affine normalization of ordering leaves at requirement establishment,
including only leaves the existing signed Boolean decomposition establishes.
These are ordinary immutable affine premises, with the requirement and sign
retained as their evidence. They add no nonlinear search, join rule, recursive
summary assumption, or executable guard. Caller obligations remain unchanged.
Replaced scalar or measure values must not inherit their former images.
The integer result of a successful measure observation also keeps that exact
image. Previously two `len_of` reads received fresh scalar atoms related to
their measures only through L0; recovering both equalities and the sum bound
exceeded AUTO's fixed residual combinations. Preserving the values actually
read addresses that lost identity without increasing those combinations.

Keep literal two-term requirements on their existing L0 route: duplicating
every L0 bound in the affine premise list would silently enlarge the number of
ordinary inequalities AUTO can combine. The new family is for the affine
ordering leaves that have no existing L0 projection. Conjunction and negative
disjunction can expose leaves; positive disjunction cannot donate a chosen
child. Negative cases must retain these distinctions and mutation kills.

The source now compiles and an independent `qsort` oracle checks 60 input
configurations, including empty and uneven inputs, reverse order, all-equal
keys, duplicates, and skew. Both modes run at one, two, and four workers.
The loop and recursive kernel oracles enter through `wf__floor_run`, just as
the benchmark does; multiworker tests also require actual task grants. The
earlier small C oracle entry bypassed that bootstrap, although the full native
benchmark did start its pools. Its old width labels did not establish overlap.
The corrected tests include large enough fixtures to exercise the algorithm's
parallel stages, rather than relying on a parallel initialization loop.

Binary search also exposed an independent backend defect: a loop that only
returns, with no break, retains an unreachable structural continuation whose
block parameters have no incoming edges. The emitter now gives those dead
parameters `freeze poison` values rather than rejecting the valid source or
inventing a predecessor. A native regression exercises both compiler modes.

## Sparse frontier versus pull

The graph experiment fixes unsigned distances from vertex zero, with the
vertex count as the unreachable sentinel. Fixtures are undirected and have
four adjacency slots per vertex; an out-of-range slot means no edge. This is
a bounded-degree sparse family, not a claim about arbitrary CSR graphs or
high-degree hubs. The independent oracle uses a FIFO array queue.

The sparse Whitefoot source instead uses one intrusive link slot per vertex
and two frontier heads. A vertex is linked only on its first discovery, so
each reachable adjacency row is visited once and no queue-capacity arithmetic
or per-vertex allocation is required. Source-ordered discovery still writes
data-dependent visited and link slots. The pull control reads a stable distance
buffer and assigns each destination vertex its own output cell, then counts
new discoveries and joins before the next level. Undirected fixtures make
incoming and outgoing adjacency identical; no transpose cost is hidden.
Both algorithms use two arrays of vertex-sized workspace.

Before timing, compare every distance and unchanged input on chains, binary
trees, disconnected components, cycles, grids, and duplicate edges. Time both
modes on a 65,535-vertex tree, a 4,097-vertex chain, and a 16,384-vertex grid;
retain the chain even if the pull loop is eligible. The oracle records reached
vertices and levels: sparse traversal reads four slots per reached vertex,
whereas pull visits every vertex at every level. The native `serial` row
always remains the useful FIFO algorithm; native parallel rows use the same
pull algorithm in pull mode, and FIFO in sparse mode. This distinguishes
algorithmic amplification from lowering and scheduling cost.

## Grain attribution control

The emitted outer-loop weights are 92 and 108 for the prefix maps, 158 for
histogram counting, and 292 for stencil rows. They price a helper's body but
do not multiply its inner work by runtime block width. With the current
150,000 work floor, 1,024 prefix blocks afford no split, 1,024 histogram
blocks afford no split, and 2,046/4,094 stencil rows afford two/four chunks.
These are consequences of the emitted constants and splitter formula, not
timing results.

First compare an explicitly labeled runtime control with a 10,000 work floor
against the unchanged 150,000 default in the same rotated passes. Include
prefix, histogram, stencil, and the existing map and recursive kernels, plus
the original/small stencil and coarse/small blocked fixtures. A default change
requires repeatable wall-time gains on the underpriced work without moving a
material cost into process CPU or adverse fixtures; a noisy win on this active
desktop is insufficient. This control tests a cost-estimation symptom. It
does not establish that one global constant correctly prices runtime helper
work, and a retained default is an acceptable experimental conclusion.
Sweep 1,017/1,030/1,043 rows around the first two-chunk threshold and
2,057/2,058/2,071 around the four-chunk threshold at width 1,024. Also retain
a width-17, height-4,096 adverse control: its outer weight equals the wide
grid's although each row does far less actual work.
