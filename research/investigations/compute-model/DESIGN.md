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

### Data-dependent scatter trial

The next consumer starts from `2b346cf6`, after the nested-affine traversal
repair. It distributes unsigned input values stably by a radix digit, so
equal-digit elements retain their input order. This tests a distinct obligation:
the number of output elements assigned to an input block depends on its data,
rather than a fixed stride or a pivot search in already sorted inputs.

Try ordinary block counts, prefix offsets, and exclusive destination ranges
before selecting a language or compiler change. A binary digit is the smallest
instance; a wider digit is useful only after that instance exposes its proof
and cost obligations. The digit and block dimensions must not encode the
worker count. Any missing capability is classified against the active
specification, with a concrete source witness and the nearest useful
alternative. A failed direct scatter is not a proof of inexpressibility.

The discriminating criteria, recorded before implementation and measurement,
are:

- An independent stable distribution checks every output value and unchanged
  input, including empty input, uneven blocks, repeated digits, skew, and
  runtime dimensions. Correctness precedes timing.
- Useful output production must actually execute in parallel. Permission or
  parallel counting alone does not meet that condition; a mandatory serial
  element-by-element scatter is a remaining limitation.
- Account for counting, offsets, output writes, allocation, peak workspace,
  and span. Repeated full-input scans per input block or worker do not count
  as an efficient parallel representation. State dependence on radix width
  explicitly instead of hiding it in a fixed fixture.
- Compare identical Whitefoot source with and without overlap, a matching
  native decomposition, and a useful native serial distribution. Retain wall
  time, process CPU, worker count, input shape, actual grants, source revisions,
  and build flags. A parallel win alone does not establish competitive cost.

These observations select between using existing proofs, adding a narrowly
motivated shared foundation, and recording a remaining model cost. They do
not select a universal grain policy. Broad scheduling/profile/PGO research,
sparse-graph discovery, and I/O remain separate follow-ups.

The [direct candidate](direct-scatter.wf) reaches OP-4 at
`high < len_of(output)`. Its scalar split bound does not establish the
input-content relation a tight destination requires; the stated contract even
admits an out-of-bounds counterexample. FN-8 excludes subscript expressions
from contracts and FN-9's result relations cannot publish a count of matching
elements in a sequence. The negative witness checks that insufficient scalar
bounds do not authorize the write. It is not a normative rejection of stable
distribution or a proof that no other source formulation works.

The executable alternative in
[`radix_scatter.wf`](../../experiments/compute-bench/programs/radix_scatter.wf)
first partitions each input block into two `FixedVector<u64, 256>` runs.
Their ordinary measures bound each stored count without an array-content
theorem. A scalar prefix phase computes total lengths; a recursive continuation
forms each run's actual destination range and passes the remaining range to
the next block. Each intermediate digit buffer reserves one full block of
capacity per input block, so the continuation's safety follows from each
run's type bound even when the actual counts are skewed. A final copy joins
the populated prefixes into the returned buffer. Input size and selected bit
are runtime values; the local capacity is provisionally 256, independent of
worker count. Wider radix digits and other local capacities are not yet
established by this instance.

This representation has linear element work, but it is not a cost-free
replacement for direct scatter: local runs hold up to two padded inputs,
two intermediate streams hold another two, and the result holds one actual
input. Count extraction currently takes and restores an owned chunk because
the legacy buffer-element borrow path is unsupported. Those transfers,
initialization, and the continuation's linear depth must be included in the
cost rather than dismissed as proof work. The native controls compare both
the same block/chain decomposition and direct count/prefix/scatter; the latter
does not materialize local element streams and is a different algorithmic
representation, not an isolating compiler A/B.

The prototype also exposes an implementation gap under unchanged VIEW-1,
VIEW-2 and SET-2: a direct view of an already represented affine nominal
element stopped as `CompositeValues`, and replacement had no lowering read
for a slice target. The implementation uses the buffer representation's
existing element domain for view formation and captures the displaced element
from the already evaluated slice target after the replacement expression.
Ordinary affine reads/moves, shared-view mutation, live child loans, and
target layout retain their existing checks. This does not add general
structural element views or the still-unsupported borrowed projection path.

With `B = floor(n/256) + 1` and `p = 256B`, the source allocates `4p+n`
words of payload, plus chunk tags/measures and call frames; input and oracle
storage are excluded. Each key is classified once, copied into an intermediate
digit stream once, and copied into the result once, in addition to allocation
initialization and owned aggregate transfers. Metadata tally is O(B). Even
with unlimited offers, packing has an O(B) continuation path and the final
two ordinary copy loops have O(n) span. The existing recursion budget can
further limit offered packing work. Actual steals therefore establish output
parallelism, not scalable span or competitive performance.

This trial restores an existing view capability and retains an experimental
consumer; it does not select a new language rule, shared representation or
scheduling policy. The existing view-loan, ownership, source-proof and parallel
permission decisions still stand. The directly affected catalog entries are
§13's counting/radix case, §15's deferred scatter statement, summary row 13‴,
and transformation T6. The open representation cost is tracked in
`docs/todo.md`; no I/O inference or broader design-tree change follows from
this binary instance.

### Stable scatter result, 2026-09-14

The [retained rows](../../experiments/compute-bench/radix-scatter-2026-09-14.tsv)
measure `e2ced20c` on an Apple M1 Pro, Darwin arm64, eight physical/logical
CPUs. This is an unpinned interactive workstation with background applications
active; compiler jobs were avoided during timing. Apple clang 21.0.0 compiles
Whitefoot and its runtime at the ordinary `-O2` flags. Native references use
the bundle's scalar `-O3`, no-vectorization, no-LTO flags; Rust is 1.98.1 and
the native library pins are retained in each manifest. These are observations
of this workload and build, not an isolating compiler A/B or a portable ratio.

Every form passed the 109-configuration independent matrix at the harness's
applicable widths, including W16 oversubscription. The compiler oracle checks
both emissions at W1/W2/W4. Its correctness-only wrappers require both a
packing-stage steal and a nonempty `copy_run` completed on a thread other
than the packing caller. All earlier maps have joined before that observation
starts, and all packing work joins before it ends. Only a missing scheduling
observation may be resampled; wrong values, changed input, wrong lengths or
missing wrapper entry fail immediately. Timing images contain no wrappers.

Four fixtures use the canonical back-to-back cadence, five rotating/reversing
passes, and five warm calls plus each process's retained first call. The
preceding four exploratory runs use an explicit 500 microsecond gap and are
kept separately as `gap500-*`; they are not the default cadence. Each run's
before/after executable and LLVM hashes match. The main zero-gap medians are:

| Fixture and form | Workers | Wall, ms | Process CPU, ms |
|---|---:|---:|---:|
| 1,048,593 mixed keys, Whitefoot sequential | 1 | 5.366 | 5.359 |
| Same input and Whitefoot source, overlap | 4 | 4.040 | 5.822 |
| Same input and Whitefoot source, overlap | 8 | 3.946 | 7.023 |
| Same block/chain decomposition, oneTBB | 8 | 1.505 | 6.079 |
| Independent two-scan serial distribution | 1 | 0.780 | 0.776 |
| Direct-native fixture, unchanged Whitefoot source | 8 | 4.061 | 7.108 |
| Direct count/prefix/scatter, Parlay | 8 | 0.391 | 1.712 |
| 1,048,593 skewed keys, Whitefoot sequential | 1 | 6.274 | 6.267 |
| Same skewed input and Whitefoot source, overlap | 8 | 4.025 | 7.771 |
| 257 mixed keys, Whitefoot sequential | 1 | 0.0045 | 0.003 |
| Same small input and Whitefoot source, overlap | 8 | 0.0102 | 0.034 |

Large-input overlap is about 1.36 times faster than the same-source sequential
emission by these medians, with about 31 percent more process CPU. It is still
slower than both the native chain and the useful serial algorithm. Against
the direct native control, the table's paired W8 wall ratio is 10.442 and CPU
ratio is 4.119; Whitefoot is lower in zero of five wall pairs. The native chain
borrows flat chunk payloads without Whitefoot's owned take/restore or run-head
handling, so even that comparison does not isolate a lowering defect. Native
direct scatter uses an output plus two scalar offsets per block, instead of
the local element streams and packing chain.

The same-chain W8 comparison also exposes a parallel-utilization gap: dividing
the process-CPU medians by the wall medians gives roughly 1.78 occupied CPUs
for Whitefoot and 4.04 for oneTBB. Whitefoot uses about 16 percent more CPU but
takes 2.62 times the wall time. These ratios include runtime and spinning work;
they neither measure useful computation alone nor isolate scheduler idleness.
Both sources have a packing chain and final two-way copy, so those shared
structures alone do not explain the difference. Serial critical-path costs,
task expansion, work-supply policy and worker execution remain to be separated.

Skew improves the same-source wall ratio to about 1.56 while increasing CPU
about 24 percent. Small input is slower with overlap; its microsecond CPU
values are especially sensitive to clock granularity and scheduling. The
500-microsecond-gap runs retain the same large-input cost conclusion, and the
small W8 median grows to 17.2 microseconds with no median observed steals.
The timing table's steals count covers the complete call, unlike the separate
correctness oracle's packing attribution. Neither cadence selects a grain
policy, and W16 rows are oversubscribed evidence rather than scaling claims.

The payload count at the large fixture is 5,243,921 words, about 40 MiB,
before chunk metadata and stack. Stack use is not inferred from source depth:
`otool -tvV` on the recorded sequential object shows a loop backedge replacing
`pack_chunks`' self-call and a 16,064-byte frame including saved registers.
This is a generated-code observation for that object, not measured aggregate
peak stack across parallel workers. The linear dependence and final-copy span
remain even when tail-call optimization removes recursive frame growth.

The trial meets stable-result, checked-exclusive-range and useful-parallel-work
criteria, but fails to establish competitive representation cost. It does not
justify adopting this chunk chain as the general scatter idiom or changing
the parallel permission rule. The next experiment should first attribute wall
time, CPU time, runnable work and worker activity to block partitioning, count
tally, packing and final copy. Scheduling controls should keep the algorithm
and representation fixed to distinguish insufficient parallel work or a long
serial critical path from available work not reaching workers. That evidence
should select the next optimization. A balanced destination representation
that avoids padded streams and owned count-extraction copies remains a
candidate, checked with the same independent oracle and native controls.
A new content-summary proof mechanism is a candidate only if a concrete
ordinary formulation still cannot express the required bound; neither it nor
a general grain/profile/PGO policy is selected here.

Reproduce from the experiment directory with the existing dependency setup:

```sh
SCATTER_RESULTS="$(mktemp -d)"
make deps
make verify KERNELS=radix_scatter
WFB_SCATTER_NATIVE=direct make verify KERNELS=radix_scatter
WFB_SCATTER_GRID=large WFB_SCATTER_NATIVE=chain make compare KERNELS=radix_scatter PASSES=5 CALLS=5 WFB_GAP_US=0 RESULTS="$SCATTER_RESULTS/steady-chain-large"
```

Use fresh result directories for `small`, `skew`, and `WFB_SCATTER_NATIVE=direct`, and
`WFB_GAP_US=500` for the diagnostic counterparts. The retained runs reused the
already-built compiler and dependencies with `make -o compiler -o deps`;
that does not remove the image rebuild or before/after hash checks.

### Scatter utilization attribution

This follow-up starts from `277a1844`, after the scatter trial and source-proof
checking work merged. The retained W8 chain comparison suggests substantially
lower average occupied CPUs in Whitefoot. It does not identify useful work,
serial critical-path cost, insufficient task supply, or scheduler delay. The
next decision is which concrete mechanism to change, while preserving the
ordinary stable-scatter program, its checked bounds and its representation.

Before measuring, use these discriminating criteria:

- Reestablish the same-chain baseline on the current compiler at W1/W2/W4/W8,
  starting with the retained large mixed fixture. Keep the independent stable
  oracle, useful native serial control and native decomposition; native flags
  and borrowing differences remain comparison limits. Small and skewed inputs
  protect any proposed change rather than supplying a replacement workload.
- Inspect the actual loop and call lowering before adding an instrument.
  Attribute partitioning, count tally, packing and final copy, including the
  allocation and initialization work around them. Stage wall/CPU measurements
  and worker/work-supply events must distinguish long serial work, delayed or
  limited task expansion, and published work not reaching workers. Occupied
  CPU counts alone cannot choose among these explanations.
- Reuse the existing benchmark twins and lane trace where they cover the
  question. Check instrumentation against an unobserved twin; the existing
  trace's dependent-chain probe and bounded recursive event retention must
  not silently determine a latency or coverage claim. Keep diagnostic images
  separate from the ordinary timing images and preserve the original joins.
- Test a candidate cause by changing one Whitefoot lowering/runtime mechanism
  with source algorithm and representation fixed. The proposed explanation
  must predict which stage or work-supply pattern changes and what contrary
  observation would refute it. Repeat the unobserved comparison, retain every
  wall/CPU pair and image identity, and distinguish a repeatable effect from
  placement or host noise. Native comparison alone is not a causal control.
- A narrow fix must preserve the oracle and required safety checks, explain
  its wall/CPU tradeoff, and be checked against the existing compute workloads.
  A diagnostic result can justify a separately scoped representation change;
  it does not require adopting a new representation in this investigation.

The useful outcome is a located cost and a controlled test of its cause, plus
a bounded repair if supported. No general scheduling/profile/PGO policy,
content-summary proof mechanism, I/O change or live-tree revision is selected
by opening this investigation. Retain evidence here and in the existing
compute-bench bundle; extend that bundle only for observations this consumer
needs, with no separate profiling framework.

The first coarse-observer run encountered severe external contention (system
load above 200, another worktree running tests); even the ordinary W8 image
rose from about 4 ms to 96 ms. It cannot select a performance change. Static
inspection nevertheless exposes a concrete control: buffer/slice element
reads and writes expand an owning aggregate through SSA, while other physical
place transfers already use target-sized `memmove`. The optimized tally and
packing bodies contain hundreds of scalarized fields around each chunk.

Before testing that control, predict that routing indexed aggregate transfers
through the existing snapshot-copy path reduces tally and packing CPU, with
the same source, representation, allocation count, phase joins, loop weight
and recursive frontier. It need not increase worker occupancy: shortening
serial work can lower CPU as well as wall. Compare both unobserved images in
rotated repeated passes, then the coarse observer separately; protect small
and skewed inputs and the existing compute programs. No material phase gain,
an altered source/schedule, or an adverse repeatable compute effect refutes
retaining this as the local repair. Its correctness requirement is the old
element snapshot surviving the replacement, including owning and nested
aggregate contents; memory-copy emission cannot weaken that ordering.

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
The approved automatic-facts decision records this choice. Premise-removal
cases cover changed quotient, divisor and dividend values, signed operations,
unproved division domains, and branch joins.

The specification amendment changes ENT-3.S7 and DIAG-2, with no new numbered
rule, token, production, operation spelling, or exception. It archives the
outgoing v0.56 bytes and declares v0.57 after integration with the ordinary
container and callable model. The original trial used the earlier branch's
v0.56; the retained measurements still identify those original compiler bytes.
These facts erase and change no ABI, runtime operation, release action, or
target-domain obligation. Six new conformance cases cover
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
The native test covers 56 prefix configurations and 224 histogram
configurations in both compiler modes at one, two, and four workers, checking
every output element, the result length, and unchanged input. The
1,048,593-word, 64-word-block case supplies both enough blocks to split and
enough work per block for a worker to take work on a busy host; the earlier
eligible one-word blocks could finish before another lane woke. The pool and
actual-steal assertions remain. A complete checked run can still finish with
an active pool and no steal on a saturated host. Following the existing
counted-program test's `GRANT_OBSERVATION_RUNS` boundary, all five compute
oracles share a runner that samples at most 32 schedules and requires one
with an actual steal at each multiworker width. Only a distinct no-steal
outcome is resampled; a wrong
result, missing output or inactive pool fails immediately, and 32 no-steal
runs also fail. This observes executable overlap, not a per-run scheduling
guarantee. The measured source checked 52/208
configurations before timing, without that additional case. Semantic tests
separately establish that the intended
outer loops are eligible and the inner recurrences are denied; an unrelated
parallel initialization loop is not used as evidence for an algorithm stage.

## Pool-off measurement path

Inspection before the original timing runs found that the compute-bench
adapters called `wf_<kernel>` directly in a parallel module. The then-emitted
command entry instead called the sequential clone when
`wf__par_pool_active()` was false. A one-worker table row therefore measured a
different entry path from the compiled program: the loop splitter returned a
zero budget, but the adapter still entered the outlined parallel lowering.

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

The ordinary-function launcher now owns executable entry selection. The
adapter retains the same pool-dependent world choice and is composed with
`module-symbols.awk` before linking paired modules, preserving the ordinary
definitions' linkage and attributes. The compute sources and test fixtures use
ordinary `fn main` declarations. This integration changes neither the retained
historical rows nor their attribution to the original compiler revisions.

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
The corrected tests include large fixtures and separately assert that both
the recursive sort and recursive merge bodies emit publish sites. Their
cumulative grant counter proves that the worker pool participates; it alone
does not identify which algorithm stage supplied a grant.

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

## Measurements and assessment, 2026-09-13

The [retained stream](../../experiments/compute-bench/compute-model-2026-09-13.tsv)
contains the raw rows, manifests, and reducer tables for the nine-kernel main
run, 20 additional final fixture groups, and three initial pool-off controls.
The compiled source for the final matrix is `2f5617a9` on an
eight-CPU Apple M1 Pro, Darwin arm64, Apple Clang 21 and Rust 1.98.1, using the
bundle's pinned oneTBB, ParlayLib and Rayon versions. Whitefoot uses the
driver's `-O2`; native references use `-O3` with the documented scalar flags.
No affinity is available and this is an active desktop, not an isolated host.
Every process makes one first call and five warm calls; the reported median
is the median of five per-process warm medians. Paired ratios compare the
same pass, so they need not equal the ratio of the two displayed medians.
The first-call rows, process CPU, actual grants, all reference forms and
oversubscribed widths are retained. No compiler or test ran during timing.

The main run uses canonical `make compare` with the default runtime as `wf`
and the explicitly labeled 10,000-work-unit runtime as `wf-b`. Additional
fixtures reuse those verified images, checking their hashes before and after,
at widths 1, 4 and 8, with `wf-seq`/FIFO or one-pass serial controls at one
worker and oneTBB at four/eight. Their exact commands and rotated/reversed
orders are in the stream. They are a selected diagnostic matrix, not a full
native scoreboard. Hostname and scratch paths are normalized in metadata;
numeric rows are unchanged. Tiny microsecond fixtures and unstable cells
cannot support fine percentage or CPU-efficiency claims.

### Pool-off and the stencil cliff

With the corrected entry, the large stencil reads 37.634 ms for `wf` at one
worker against 37.512 ms for `wf-seq`, a 0.3 percent difference between medians,
with 0.5/0.4 percent MAD. The narrow-row control is 0.492/0.491 ms, and the
small fixture about 2.8/2.9 microseconds. The original grid is noisier,
16.529/16.075 ms with 10.3/3.1 percent MAD. The former 15--20 percent number
does not survive as an established tax on an ordinary pool-off program.
The old adapter measured the wrong entry path; the initial isolating control
was noisy, so this establishes a corrected measurement and absence of that
large tax in the new run, not a precise causal speedup from the adapter alone.

The size sweep confirms the grain cliff. At width 1,024 and eight workers,
height 1,017 costs 5.865 ms, height 1,030 costs 3.640 ms, and height 1,043
costs 3.598 ms. Crossing height 2,057 to 2,058 moves 9.090 ms to 6.123 ms;
height 2,071 is 5.978 ms. Those are the predicted one-to-two and two-to-four
row-chunk transitions. Initialization also grants work, so a nonzero total
grant count below a row threshold does not imply the time-step rows split.

### Block work and the rejected global constant

At the default 4,194,321-word input with 4,096-word blocks, prefix and histogram
emit zero runtime grants under the current work floor at all measured widths.
Their algorithmic outer loops are permitted; the cost estimate prevents offers.
The 10,000-unit control exposes the missing parallel work, but its benefit is
not uniform:

| Fixture and workers | Default median | 10,000 control median | Paired control/default wall | Paired CPU |
|---|---:|---:|---:|---:|
| Prefix, W4 | 2.564 ms | 1.453 ms | 0.559, 5/5 lower | 1.677 |
| Histogram, W4 | 2.927 ms | 0.958 ms | 0.329, 5/5 lower | 1.064 |
| Large stencil, W8 | 13.622 ms | 11.108 ms | 0.828, 5/5 lower | 1.228 |
| Narrow stencil, W4 | 0.217 ms | 0.305 ms | 1.358, 0/5 lower | 1.358 |
| Coarse prefix, W8 | 2.643 ms | 2.987 ms | 1.134, 0/5 lower | 1.352 |
| Chain pull, W4 | 15.429 ms | 30.417 ms | 1.900, 0/5 lower | 1.907 |

The small histogram also starts a task under the lower floor and becomes
slower (4.7 to 7.3 microseconds at W4); its scale warrants caution about exact
percentages, not deleting the adverse case. Coarse histogram stays unsplit
even under the control. Fine blocks make prefix more competitive but enlarge
histogram's workspace: the fine histogram is 12.107 ms at W4 under the default,
versus 2.927 ms with default-size blocks and 1.336 ms for its one-pass serial
reference. A source block size is a real memory/algorithm choice, not a free
scheduler tuning knob.

Keep the 150,000 unit and 16-per-lane cap. The approved decision revises their
grounds and removes the unsupported universal-plateau claim. This constant
control does not establish a dynamic estimator; the subsequent runtime-extent
trial retains the cheap-row, tiny, coarse and graph controls. Broader grain
policy remains an open cost in `docs/todo.md` rather than being hidden by a
favorable constant.

### Irregular algorithms

The random 1,048,593-key comparison sort reads 84.355 ms at W1 and 22.639 ms
at W4. The same-algorithm native references at W4 are 22.550 ms for Rayon,
22.644 ms for ParlayLib and 25.202 ms for oneTBB. This is near parity for
that actual parallel merge algorithm. The source does not require a forced
serial merge tree. Its adverse cases matter: 257 keys take 6.2 microseconds
under `--no-overlap` but 24.3 microseconds at W4; all-equal large input takes
4.279 ms at W4 while native serial `qsort` takes 1.883 ms. Mostly repeated
keys likewise favor `qsort` (2.919 ms versus Whitefoot W4 4.390 ms).
Parallel merge is expressible; this algorithm is not universally the best sort.

The graph experiment gives a sharper rejection of unconditional pull:

| Undirected fixture | Levels | Sparse adjacency slots | Pull vertex visits | WF sparse W1 | WF pull W4 | Native FIFO W1 |
|---|---:|---:|---:|---:|---:|---:|
| Binary tree, 65,535 vertices | 16 | 262,140 | 1,048,560 | 0.145 ms | 0.523 ms | 0.131--0.134 ms |
| Chain, 4,097 vertices | 4,097 | 16,388 | 16,785,409 | 0.0125 ms | 15.429 ms | 0.021 ms |
| Width-31 grid, 16,384 vertices | 558 | 65,536 | 9,142,272 | 0.0447 ms | 5.196 ms | 0.0435--0.0444 ms |

Work counts are exact consequences of the independently checked reached set
and distance levels for these fixtures; they count different operations and
are not a hardware-instruction ratio. The sparse and pull timings come from
separate named fixture runs, so their wall ratio is descriptive, not a paired
isolating control. Even the broad tree does not repay a full-vertex round here.
On the chain, Whitefoot pull at W4 is faster than native oneTBB pull (28.711 ms)
and still loses by orders of magnitude to the useful sparse algorithm.
Scheduler parity on an inferior algorithm would not satisfy the requirement.

### What the three stages establish

Ordinary functions and proved range partitions now express runtime blocked
scan, privatized scatter and parallel binary-split merge. Two finite proof
foundations were necessary: runtime quotient/product images, and preservation
of affine requirements/observed values. No general nonlinear solver, source
scheduler API or additional overlap rule was needed. A partitioned-build
consumer was not added: these programs already exercise private mutable
workspaces, runtime partition bounds and data-dependent recursive destinations;
no distinct outstanding obligation justified another kernel in this scope.

Two limits remain concrete work: runtime helper pricing, and useful
parallel sparse discovery without replacing O(V+E) traversal with dense rounds.
The intrusive sparse representation shows that queue-capacity proofs are not
the blocker for this bounded-degree family. Ownership of competing discoveries
and sparse work creation is the unresolved parallel question; high-degree CSR,
stable parent choices, destination compaction and alternative sorting families
still need their own discriminating consumers. The catalog's corresponding
predictions are replaced in place. I/O-specific design and implementation
remain deferred while these compute questions are considered.

## Runtime extent estimate

The first extent trial asked whether attribution could yield a sufficient
grain correction. Its control compares the unchanged programs and runtime
constants at `0d571cac` against a
compiler estimate that substitutes available runtime counted-loop extents for
the fixed nesting factor. This criterion precedes implementation and timing.
The same emitted operation count still supplies the price of an iteration;
the experiment changes how many inner iterations a helper contributes.

Preserve the counted loop's captured endpoints in lowering and form a bounded
call-summary estimate from ordinary integer values and descriptor lengths.
Translate a helper's summary through its actual arguments, so the length of
an iteration's subrange can reduce to the captured stride without evaluating
the loop body. Unavailable or data-dependent extents keep the static estimate.
Estimate arithmetic must be total and must read no array elements, execute no
user calls, and introduce no acceptance or proof dependency. Evaluate the
estimate once at a parallel loop entry; the sequential world retains its
ordinary chunk call. A timing-adaptive estimator, a new writer annotation,
and another global work-floor reduction do not address this comparison.

The primary cases are the default prefix, histogram and wide stencil, with
W1/4/8, the existing five rotated passes and retained first-call, warm-wall,
CPU and grant rows. Each must preserve its independently checked result;
blocked loops must actually offer useful work at their runtime dimensions.
For each primary kernel require at least one W4/W8 paired warm-wall improvement
of ten percent or more, with at least four of five pairs lower. Retain the
stencil threshold sweep to check that the old underpriced two/four-chunk
ceiling no longer determines its lane use.

The protected adverse cases are the width-17 stencil, coarse prefix, tiny
histogram and high-diameter pull traversal, alongside the other maintained
compute kernels. A candidate does not select a new default if a stable adverse
cell adds more than ten percent warm wall or fifteen percent process CPU.
For a sub-ten-microsecond cell, also require more than one microsecond of
absolute wall increase before interpreting a percentage as material. Preserve
all cells, including failures; a noisy or inconclusive comparison calls for
isolation, not a favorable replacement fixture. The constants remain 150,000
and 16 throughout this control. Its result determines whether this estimate
is sufficient or a different mechanism is still needed.

The implementation retains counted-loop endpoint captures and excludes the
post-loop continuation from its iteration price. Three call-summary rounds
translate scalar values and descriptor lengths through actual arguments;
unchanged values forwarded around a loop keep their identities, while an
updated or data-dependent value that cannot reach a formal capture keeps the
static price. The bounded analysis controls optimization precision only.
Emitted sums, differences and products saturate, and divisions use positive
compiler constants. No estimate can justify a source operation or omit its
work. The native probes check actual prefix output while varying block sizes,
empty and inverted ranges, and an unexecuted inner range with maximal endpoints;
the latter still prices safely. The first trial at `f851c65c` passed independent
native comparisons for the nine maintained consumers and baseline twin at
W1/2/4/8/16, but failed the timing trial's protected-case criterion below.
The owner approved retaining the extent mechanism provisionally
and deferred the broader policy study, including runtime profiles and PGO.
This changes the delivery scope, not the trial's criterion or failed result.

### Runtime extent trial result

The first trial is **not selected**. Candidate `f851c65c` and compiler baseline
`0d571cac` use identical program and runtime sources, including the 150,000
work unit and 16-chunks-per-lane cap. The
[retained stream](../../experiments/compute-bench/runtime-extent-2026-09-13.tsv)
contains twelve fixture groups with manifests, original raw numerical rows
and rendered tables; its SHA-256 is
`a1b289fbda35ebbe5ba9c78ae40df4557643627c89a9852f533b94bf5cedc5a8`.
The host and scalar toolchain are the same eight-CPU Apple M1 Pro described
above. All groups use five rotated passes and one first plus five warm calls
per process. `wf` is the candidate and `wf-b` the old compiler. Ratios below
are medians of paired process medians; displayed times are separate medians.

| Main fixture, W8 | Old compiler | Candidate | Candidate / old wall | Candidate / old CPU | Candidate faster |
|---|---:|---:|---:|---:|---:|
| Prefix, 4,096-word blocks | 3.619 ms | 1.173 ms | 0.324 | 0.572 | 5/5 |
| Histogram, 4,096-word blocks | 4.174 ms | 1.149 ms | 0.275 | 1.420 | 5/5 |
| Stencil, 1,024 by 4,096 | 15.700 ms | 12.085 ms | 0.754 | 1.078 | 5/5 |

All three clear the primary wall criterion. Their W8 median steals are 54,
28 and 419 respectively, against 0, 0 and 75 for the baseline. The extra
offers compute independently checked results, rather than only appearing in
emitted IR. Dynamic estimates have no literal weight for the harness's static
`chunks` extraction, so that field honestly reads `na`; the grant observation
is still actual runtime data.

The stencil sweep also removes the old boundary cliffs. At heights
1,017/1,030/1,043 the candidate W8 medians are 2.112/2.070/2.135 ms, against
5.848/3.597/3.627 ms. At 2,057/2,058/2,071 they are 4.263/4.280/4.393 ms,
against 7.206/5.635/6.194 ms. Every paired comparison favors the candidate,
but this is not a free gain: at height 1,043 the paired process CPU ratio is
about 2.075, and at 2,057 it is 1.678. The larger input's work is spread more
widely; wall improvement alone does not satisfy the protected-cost criterion.

The decisive protected failure is chain pull at W2: candidate/old wall is
1.511, with all five pairs slower. The generated call's constant price falls
from 168 to 60. For 4,097 vertices, the unchanged runtime then computes
`affordable = 4097 / ceil(150000 / 60) = 1`, instead of four affordable
chunks under the old price. The candidate observes zero steals. This is a
loss of useful splitting, even though process CPU falls to about 0.776 of
the baseline. W4 is 8.6 percent slower with much less CPU; W8 is faster.
The useful sparse algorithm remains much cheaper than this pull control;
that does not excuse a failed protected scheduling case.

Coarse prefix improves at W4/W8 with paired wall ratios 0.496/0.424 and no
CPU increase. Small prefix and histogram remain within one microsecond of
their baselines. Narrow stencil has W4 wall/CPU ratios 1.047/1.136; its
oversubscribed W16 CPU ratio is about 1.211. Coarse histogram also trades
large wall gains for more CPU, about 1.647 at W8. These cells are retained,
not removed from the comparison. The other main kernels have no stable
material wall regression in this run.

The main histogram's W1 result was inconclusive, so the isolated group
repeats the same input and unchanged images. It reads 2.942 ms candidate,
2.946 ms baseline and 2.952 ms sequential, with paired candidate/old wall
1.011. It does not reproduce a material one-worker regression. The W8 CPU
increase does reproduce, at about 1.422, alongside a 0.296 wall ratio.

The result supports captured extents as useful information but does not
establish their unconditional substitution as a broadly suitable policy.
Preserving the old price as a lower bound is one possible comparison, not a
selected correction. The protected wall and CPU criteria and all failed rows
remain part of this first trial's evidence.

Correspondence review found that omitting a counted loop's continuation from
its known extent left that loop's factor of 16 in the static fallback depth.
The corrected accounting removes that spurious level before the depth cap and
extent substitution. A native regression moves identical arithmetic from
before to after an inner counted loop: results and scheduling prices must
agree, both for captured extents and for a data-dependent static fallback,
while preserving the enclosing loop's multiplier. The timings above are for
`f851c65c`, before this correction; they do not measure the corrected compiler.

The owner chose to retain the mechanism provisionally and study broader
parallel strategy separately. The open question is whether a common policy
can serve varied workloads, shapes and machines, or whether selection needs
more context; this trial does not prove that no broadly useful policy exists.
Runtime profile-guided compilation and online adaptation are candidate
directions in [the research ideas](../../../docs/ideas.md#parallel-grain-policies-and-runtime-profiles),
with the unresolved cost tracked in [TODO](../../../docs/todo.md). No profile
collection, adaptive policy or further timing trial is implemented here.
