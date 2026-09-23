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

## Sparse destination routing trial (2026-09-21)

This bounded continuation starts at merged `3402048f` and asks whether useful
parallel sparse discovery can proceed independently of stable-scatter tuning.
The research-only [candidate](sparse-frontier.wf) routes the current frontier's
adjacency slots through source-private outboxes into disjoint destination
ranges. Each destination owner handles competing discoveries in source order
and retains an intrusive list for the next level. No loop scatters into shared
visited storage, and no frontier compaction is required.

Before source or performance results select this direction, the criteria are:

- Preserve every FIFO-oracle distance, unreachable sentinel, and input byte in
  sequential and overlapping images. Reuse the formal BFS observer and add
  only research fixtures needed for broad collision-heavy frontiers, vertex-ID
  permutation, self edges, duplicate edges, disconnected vertices, and skew.
- Establish that source routing and destination discovery offer and execute
  nonempty work on helpers. A grant during initialization alone does not
  establish parallel discovery. Permission is not a speedup.
- Count adjacency slots, matrix initialization and visits, list traversals,
  and peak workspace. Charge every allocated or reset cell and repeated scan.
  No full-vertex scan per level or source worker count may be hidden in routing.
- First obtain a correct executable and a small, bounded cost comparison on
  the current compiler. Retain useful sparse FIFO and the unchanged intrusive
  sparse implementation. Test a broad tree and a broad collision-heavy graph,
  with a chain, width-31 grid, and duplicate-discovery control. Do not add
  artificial vertex work to amortize sorting.
- A candidate is promising only if a qualified parallel run beats useful
  sparse FIFO by at least 20 percent on one broad family and improves its own
  sequential image on both broad families. Narrow-frontier work must remain
  proportional to reached adjacency, and its elapsed-time penalty is reported
  explicitly. A result slower than FIFO on both broad families ends this
  algorithm trial; do not tune routing indefinitely or rename it success.
- Separate compiler construction, WF analysis/emission, native construction,
  and execution. Use host-locked bounded runs and unobserved timing images;
  qualify identical-image variation before using a timing difference. Compare
  wall time and process CPU at the same worker widths and retain exact source,
  compiler identity, flags, graph shape, and work counts.

An initial sorting alternative expanded four candidates per frontier vertex,
used the maintained parallel merge sort, split sorted candidates among
destination owners, and copied active output prefixes together. Comparison
sorting costs O(m log m) for m candidate slots in a level; owner routing and
balanced prefix copying can add O(m log V). That alternative never established
a sparse O(V+E) work bound. FN-9 withholds same-component postconditions during
recursive verification, so the diagnostic source capped recursive returned
counts to their candidate and destination spans before copying. The current
[obligation-discharge decision](../../../design/language/checks-and-proofs/obligation-discharge.md)
withholds recursive summaries to prevent unearned circular justification and
teaches a proved invariant in place of an impossible-case branch. This capped
form is a non-adopted diagnostic control, not an implementation proposed to
ship under that decision. No amendment is proposed to bless the control, and
no sorting or proof-system tuning is selected to rescue it. Actual source
acceptance, native correctness, emitted task structure, and cost were not
established for that control.

The first bounded control check used the immutable current-main compiler
SHA-256 `6b87581a30c8a0afcdaf062221a58e21087a408bb7a071bdea6f08ca0d1937e4`.
After separating a correlated scalar update into a normal step helper, the
source still stopped at INV-1 on the leaf's `emitted <= i` backedge invariant
(0.47 seconds for source analysis; no native build or execution). This is an
unresolved source-proof failure, not evidence that the algorithm is impossible
or that the compiler violates the specification. Further repair of the
non-adopted sorting control was not selected while the no-compaction route
below remained more promising.
The rejected source was replaced in place by the private-outbox candidate.
Keep the selected source in research until its distinct permanent-test
obligation is known; remove or extract it when this trial is concluded.

Sparse FIFO remains the useful baseline, including narrow frontiers. A
possible fallback comparator on the existing undirected fixtures is a hybrid
that uses sparse traversal below F < alpha*V and parallel pull above a fixed
positive density threshold alpha. Each full-vertex scan can then be charged
to its frontier: V <= F/alpha. If every newly reached vertex enters exactly
one frontier and degree remains bounded, the complete traversal has linear
work. This argument also requires charging any frontier reconstruction and
buffer copying to those dense rounds. Pull reads the reverse adjacency, so
the undirected fixture premise matters. This alternative only parallelizes
dense frontiers; it does not resolve broad sparse directed work or arbitrary
dependencies. It is recorded for comparison if outbox overhead is material,
not selected as a second implementation before the current trial is measured.

### Private outboxes without frontier compaction

The selected representation retains one intrusive frontier list per
destination owner. In the following level those owners become the source
owners. A source walks only its active list and builds private bucket heads
for the new destination owners. An immutable original adjacency-slot index is
the message node: only its next-link cell is written, without copying the
destination value into a candidate array.
After source routing joins, each destination owner scans its bucket heads,
handles competing discoveries locally, and builds its next intrusive list.
The old links may be reused after that join. No global frontier flattening,
unique-output scatter, or recursive returned-count proof is required.

Let m(t) be four candidate slots per active vertex in level t. Choosing a
destination-owner count proportional to the square root of m(t) makes the next
round's old-source count proportional to the square root of m(t-1). The
rectangular bucket matrix then has O(sqrt(m(t-1) * m(t))) entries, bounded by
O(m(t-1) + m(t)). Charging metadata to adjacent levels gives O(V+E) total work
for the complete traversal, provided all list traversals visit only active
nodes, initialization outside these matrices is paid once, and no later
consumer adds a full-vertex scan. Abrupt frontier shrink does not invalidate
that total-work bound.

The current source reserves V distance cells, V intrusive vertex-link cells,
4V message-link cells, and two V owner-head buffers, all initialized once.
The two head buffers keep the returned policy stride independent of a stored
array-length relation; only the active owner prefixes are read or written.
Persistent storage is therefore 8V u64 cells (64V bytes), plus array headers
and the current C-by-D bucket matrix. Matrix allocation initializes C*D cells,
and receivers read C*D heads. Source routing uses the freshly filled sentinel
heads directly; its redundant second reset was removed before timing.
Both matrix passes are charged explicitly. There is no per-level
vertex-wide clear. The four graph slots per vertex are input, not workspace.

Each receiver maps a message to `(vertex -wrap origin) % owner_width`.
Remainder proves a valid local index for every message without an unavailable
stored per-message range relation or an impossible-case guard. Correct routing
makes this exactly the intended vertex; the independent FIFO comparison must
establish that correspondence. Both variable remainder and source division
are real per-message costs. Source routing rejects out-of-range adjacency
values before building any bucket, preserving absent-edge semantics without
funneling all absent values through the final owner. Discovery counts use
`+wrap` only for zero-termination and choosing the next partition, never as a
storage-bound premise. Actual uniqueness supplies the algorithm's count bound.

The full source admits and emits on the immutable `6fdb6768` compiler,
SHA-256 `d6ba9286f877df7e2a2d9e7d751d415871b2d2d992d558a2d9e37ad14e3a32c5`,
in 0.21 seconds including the guard wrapper. It uses ordinary range ownership
with partial final owners, safe bucket indices, guarded local list traversal,
and proved matrix and message capacities. The public input limit remains
16,777,216 slots. Helpers use conservative erased domain bounds because the
unsigned-division facts supply `vertices <= slots` directly; those helper
contracts do not enlarge accepted inputs or actual allocations.

The source adaptations expose existing proof boundaries rather than change
them. A nonlinear function requirement does not supply the affine
certificate premise over a later product binding, and scaling a premise whose
operand expands from a local sum does not match the separately recorded
product. The source therefore represents the matrix as the complete-row
product plus one tail row, and the receiver advances a proved current index
along its column until the actual matrix end. This preserves C head visits
without an impossible-case branch or recursive count contract. The resulting
data-dependent metadata loop, like the frontier/message walks, needs separate
work-price and helper-participation inspection before any timing conclusion.
The routing phase's span is O(4*max(source frontier size)); discovery span
is O(C + max(incoming bucket messages)). Both can be linear in active work:
concentrated vertex labels or competing destinations can serialize a level.
The current full-range maps and final tail execute in separate source-order
steps, adding at most another largest-owner term to those bounds. Preserve
both the original and permuted broad graph, and explicit skew, before claiming
useful parallel sparse discovery. The source decomposition has no worker count.

The first native image matches the independent formal FIFO oracle in 90
configurations at W1 and W4, checking 1,863,630 distance and unchanged-input
values at each width. This reuses the maintained BFS matrix and adds a
65,535-vertex permuted tree and a directed broad graph with colliding,
duplicate and self-edge discoveries. Construction took 0.68 seconds and the
complete guarded construction/check batch 1.14 seconds. These are correctness
observations, not qualified performance samples. That image's ledger denies
both useful owner loops, so an active worker pool alone establishes no
parallel discovery.

Two ordinary compiler limitations explain those denials. The loop-footprint
walker refuses ordered result-list bindings even when every new binding is
iteration-local; PAR-2 itself imposes no such restriction. Returning the same
two scalars as one `Discovery` record admits the receiver map without another
allocation, traversal, or accumulator. General support for all result-list
bindings remains a compiler opportunity. The record-result image also passes
all 90 oracle configurations at W1 and W4; native construction takes 0.65
seconds and the complete check batch 1.14 seconds. Its source SHA-256 is
`d10f0047164ca614968d55e50549d7efd1a768a5784c901d36dd82f9467f63f2`.
The source-row failure reduces to
`stride = width + padding; cells = rows * stride` followed by disjoint
`[i*stride, i*stride+stride)` writes. A preheader symbolic product creates a
copied-value handle that the range-image classifier formerly kept opaque,
while the added endpoint expanded the same value. Repair `31282f01`, integrated
as `8f6d5131`, expands that recorded transparent image before classifying a
fixed atom. The existing constant-multiplier control remains permitted, and
the unchanged minimal witness, accepted but denied outer-loop permission by
the verified `7895c9d0` compiler, now retains permission and emits its partition
split. The specification, source-acceptance rules and shared proof/ledger
interfaces are unchanged: the existing source-positioned ProofContext still
discharges the formation bounds and both nonnegativity goals. The equivalent
product-endpoint spelling did not pass the existing certificate matcher, so
the admitted source retains the natural start-plus-stride endpoints.

The maintained
[`runtime_preheader_products_expand_transparent_stride_handles` regression](../../../compiler/src/semantic/tests/loop_permission.rs)
covers a runtime row count and a copied stride whose original operands are
overwritten after an explicit stride-bound invariant. It checks the canonical
two-atom stride, both retained sign roots, and the complete derivation ledger;
the copied-value source also changes from a denied to an emitted partition.
All 73 `semantic::tests::loop_permission` cases pass at `31282f01`, including
the existing varying-stride, shifted-range, whole-origin and nested-binder
denials. Final test-executable construction took 80.04 seconds; test execution
took 0.73 seconds (1.35 seconds including its guard). Separate compiler
construction took 43.94 seconds, and the three minimal-source emission
controls took 0.52 seconds together. These are construction and checking costs,
not program performance samples.

On the unchanged record-result [source](sparse-frontier.wf), the isolated
`31282f01` compiler over `c6cd9add`, SHA-256
`86e218b895fa6ee601b49fd731dc843478b3d1a48379896d856c694e3a4c5624`,
retains PAR-2 permission for both useful owner loops and emits in 0.21 seconds.
With `--par --par-ledger --emit-llvm`, routing emits an independent split with
21 captured bindings. Receiver permission also succeeds, but lowering declines
its 352-byte frame over 29 captures because the lane bound is 256 bytes.
The emitted module SHA-256 is
`63f0f31e1481c1e0506a97f0d5edb0e799ef846a8da3096d4e510ba7318d695f`.
The general needed-capture repair now emits both owner maps: routing retains
10 bindings in a 152-byte frame, and discovery retains 11 in a 168-byte
frame. The earlier 90-configuration native oracle describes the `6fdb6768`
baseline; the separate qualification below identifies the capture-repaired
image. Neither source permission nor these bounded native checks replace the
integrated compiler's full gate or establish a performance result.

Those are phase bounds, not the complete implementation's span.
`compiler/src/backend/emitter/buffer.rs::emit_buffer_block` emits a sequential
element-fill loop after each allocation. The fresh matrix consequently adds
O(C*D) serial initialization span per level before routing begins, and the
persistent arrays add O(V) serial fill work once. Independent allocations may
overlap as statements, but each individual fill is serial in this lowering.
This is an implementation cost to attribute in a native result, not a language
ban on parallel initialization or a reason to claim useful speedup already.

### Native phase qualification and prospective FIFO comparison (2026-09-22)

The [dated qualification record](../../experiments/compute-bench/sparse-frontier-2026-09-22.tsv)
retains the artifact hashes, invocation, construction costs, every phase
budget, and the no-offer outcomes. The unchanged source hash remains
`d10f0047164ca614968d55e50549d7efd1a768a5784c901d36dd82f9467f63f2`.
Compiler SHA-256
`480b8b56c8dca5b4469307026a10643d142ee46fff5b937634e3618699e17adf`
emitted module
`8e55e5dd9668df5fdb6bc1ba5bb918a80f0af8c5c6803272f9caba92d050d512`
with `--par --par-scalar-leaf-limit off --emit-llvm --par-ledger`.
That nondefault scalar-leaf setting was part of the initial qualification,
not a selected compiler default. A fresh ordinary `--par` emission completed
in 0.21 seconds, exit 0, with byte-identical LLVM and identical PAR ledger
lines. The same native qualification therefore applies to the default policy
for this source and compiler. Use the separately retained default artifact
path in the prospective timing run; its module hash is unchanged.

Plain and observed native images use ordinary `-std=c11 -pthread -O2
-Wno-override-module` construction and the unchanged 150,000 runtime work
unit. Their construction took 0.69 and 0.72 seconds respectively. Plain W1,
plain W4 and observed W1 each passed 90 configurations and 1,863,630 distance
and unchanged-input comparisons. Observed W4 passed 96 configurations and
70,021,040 comparisons after adding original and permuted 2,097,151-vertex
trees and a 2,621,439-vertex directed collision graph. The entire guarded
construction/check command took 2.69 seconds, exit 0, under a 30-second bound.
These are correctness-batch costs, not performance samples. This initial
qualification checked the large inputs only in the observed image; the
subsequent unobserved preflight below closes that limitation.

Both useful maps execute nonempty work on helpers on the permuted tree and
collision graph. The respective counts are 511 and 102 nonempty source rows,
1,190,060 and 1,885 nonempty incoming buckets, and 1,546,566 and 1,998,849 new
discoveries on helpers. Nonempty incoming buckets also count useful handling
when every message is already discovered. The original tree records zero
nonempty source rows on helpers despite a nonzero routing budget; its helpers
handle 1,642 nonempty incoming buckets and 1,566,722 discoveries. Preserve
that label-concentration limitation instead of rerunning until routing moves
to a helper. The smaller fixtures receive no routing budget. At actual prices
363 for routing and 2,580 for discovery, the large maps reach a maximum full
owner span of 1,023 and budgets of one and four respectively, corresponding
to two and sixteen chunks. Observed W1 records no grants or helper work.
The observed image's atomics can affect scheduling and are excluded from
every performance interval.

Before recording any timing samples, revision `301fd1b5` fixed the comparison
as follows; the criteria were not changed after seeing the control samples.
The two primary families are the permuted 2,097,151-vertex tree and the
depth-20 collision graph with 2,621,439 vertices. Both have four cells: useful
C FIFO at W1, outbox WF at W1 and W4, and the unchanged intrusive WF at W1 as
context. The original 2,097,151-vertex tree and a 4,097-vertex chain each have
three diagnostic cells: FIFO W1 and outbox W1/W4. Those fourteen cells are
fixed before timing; diagnostic results cannot rescue a primary-family
failure. The existing formal matrix already retains the width-31 grid,
disconnection and duplicate-edge controls; no timing sweep is added for them.
Permutation keeps vertex zero fixed and uses the recorded fixed LCG seed;
every fixture definition and source identity is frozen in the dated record.

Reuse the existing compute-bench harness's clocks, process protocol and
verified-call loop through a scratch kernel adapter. Its wall clock encloses
one complete algorithm call, including all result and work allocation,
required initialization, traversal, routing, matrix processing, joins and
temporary frees. C FIFO calls the formal `oracle(edges, n, NULL)` algorithm:
it allocates both its distance array and queue inside the interval, initializes
all distances to the unreachable sentinel, writes queue cells when enqueued,
and frees the queue before returning the distances. Allocation-failure checks,
absent-edge tests and first-discovery tests remain. That routine performs no
distance comparison or input-preservation scan; those are separate checks.
WF pays all 8V persistent cells and each bucket matrix inside the same
boundary. Returning a result includes its allocation for both
implementations; checking and freeing the returned distance array are outside
for both. Graph construction, expected-distance construction and every full
input comparison are outside timing. Regenerate deterministic input after
each completed call for the input comparison, so a second graph is not held
during WF's temporary workspace. Check the large inputs at W1 and W4 through
the exact unobserved qualification image, then check every timed cell through
the final timing image before sampling.

Compile the useful FIFO C code at `-O3 -fno-fast-math -ffp-contract=off
-fno-lto`, with vectorization permitted and no target override on this arm64
host. WF uses the ordinary `-O2` native path; record its exact source,
emitted module, runtime sources/objects, image, clang identity and flags.
Do not carry the old benchmark's `-fno-vectorize -fno-slp-vectorize` flags into
the FIFO reference. No observer or runtime-work-unit override enters the
timing image, and the candidate's source and algorithm remain unchanged.

Use five fixed paired passes. Each fresh process performs one verified warm-up
and three verified warm calls with `WFB_GAP_US=0`; rotate and reverse cell order
across passes. Preserve all wall-time, process-CPU and steal observations.
Reduce to each process's median of its three warm calls, then compare matched
per-pass medians and report their median ratio, spread and count below one.
First run forty processes for an identical-image A/B control: both primary
graphs at W1 and W4, two identically configured processes per pair, five
passes. Every cell's median B/A warm wall ratio must lie in [0.97, 1.03]; any
failure makes this session inconclusive and stops before the seventy-process
algorithm comparison. No repeated null session selects a favorable result.

The prior selection criterion remains a 20-percent FIFO improvement on at
least one primary family and improvement over WF's own sequential execution
on both. For this bounded comparison, require median outbox-W4/FIFO-W1 wall
ratio at most 0.80 on at least one primary family, median outbox-W4/outbox-W1
ratio at most 0.97 on both, and at least four of five paired wins for every
claimed improvement. Report process CPU separately. A qualified result slower
than FIFO on both primary families ends this algorithm trial; inconclusive
variation authorizes no speedup claim or repeated tuning session. Preserve
the original-tree routing limit, the narrow-chain elapsed penalty, and the
64V-byte persistent workspace plus bucket-matrix cost in the conclusion.

Native construction is expected below two seconds and untimed oracle checks
below five, with a 30-second guarded preflight bound. The null stage is
expected to take 5--15 seconds and the comparison 10--25 seconds, each with
its own 60-second guarded bound. These were prospective costs, not observed
ones. A wrong distance, changed input, artifact-hash drift, competing heavy
load or timeout stops the stage and retains its raw output; a timeout is not
a source rejection or an algorithm-performance verdict. Inspect the shared
guard before each stage. No source, scheduling default or daily CI selection
changes as a result of this protocol alone.

The subsequent preflight used ordinary default-policy LLVM with the same
module bytes. The original exact plain qualification image passed all 96
configurations and 70,021,040 comparisons at both W1 and W4, taking 0.66 and
0.55 seconds respectively. The timing image, SHA-256
`ca2a3826fa6fa35dde913530ab47303317f1d4a2ac9e64c2548e2e8ed3f0a473`,
then passed all fourteen selected cells, each comparing every distance and
input word. Exact generated-input hashes, the adapter, native build recipe,
measurement driver and all artifact hashes are retained in the dated record.
Native construction took 1.63 seconds; the complete untimed oracle stage took
3.47 seconds, including input generation/hashing and process startup. The
guarded preflight took 5.18 seconds, exit 0. No source algorithm, compiler
default, runtime price or specification rule changed.

The identical-image control completed all forty processes and 160 verified
calls: forty warm-up calls and 120 retained warm samples. Its median paired
wall ratios were 1.005629 and 0.999474 on the permuted tree at W1 and W4,
and 1.011637 and 0.953349 on collision at W1 and W4. Collision/W4 is outside
the committed [0.97, 1.03] interval; its five paired wall ratios range from
0.884422 to 1.166223. The session is therefore inconclusive under the recorded
criterion. The seventy-process FIFO comparison was not executed, and no
repeated null session or microtuning was selected. These observations support
neither a BFS advantage nor a BFS loss against useful FIFO. The source of the
identical-image variation is not attributed.

The null stage took 17.59 seconds, exit 3, below its 60-second cap but above
the prospective 5--15-second estimate. Inspection of the retained invocation
costs accounts for 14.184 seconds in permuted-tree processes and 3.261 seconds
in collision processes. Their algorithm-call intervals sum to 10.196 and
2.137 seconds respectively; the remaining approximately 5.11 seconds of
process wall time covers process startup, fixture/oracle work, full checks
and other work outside those intervals. This attributes the stage's cost,
not the cause of its paired variation. All artifact hashes remained unchanged.

The earlier zero-helper-routing observation on the original tree and the
no-routing-budget observations on smaller inputs remain limitations. The
64V-byte persistent workspace, per-level matrix initialization, both metadata
passes and concentrated-label span remain charged algorithm costs. A
narrow-chain elapsed penalty and comparisons against FIFO remain unmeasured
because the required control failed before those cells ran. The bounded BFS
performance action stops here; no broader performance claim follows from
permission, emitted splits, useful helper work or correct output alone.

Design suitability: the shared harness and FIFO oracle cover the intended
useful-work and timing boundaries. The scratch adapter added the frozen
large fixtures and complete between-call input checks; its exact source and
recipe are dated evidence rather than a new maintained runner or a daily
check. Default-policy equivalence and large unobserved correctness are now
established. The failed stability control and original-tree routing limit
remain explicit limitations, with no decomposition, pricing or retry policy
change selected to remove them.

## Reference-model scatter investigation

The reference/effect/storage port in PR #70 changes the source mechanisms
behind the earlier results. This continuation starts at `efd6ebc9`, not at an
uncommitted port workspace. Later upstream revisions are integrated at named
checkpoints; correctness and timing observations identify the exact revision
they describe. The older measurements below remain evidence for their older
sources and compiler, not measurements of the reference model.

The immediate question is whether temporary references remove the owned
chunk transfers from stable scatter without losing its independent result
checks or its useful output parallelism. First reuse the maintained prefix,
histogram, stencil, sort/graph, and scatter observations in
`compiler/src/backend/tests/ranges.rs`. Their formal sources and native
oracles live in `tests/programs/compute/`; this investigation adds no daily
gate dependency on research. Unrelated port defects are recorded with a
reproducer and revision for the port's owner, rather than expanding this
investigation into completion of the entire language migration.

Before selecting an implementation, the discriminating criteria are:

- The unchanged input, stable output, output length, empty/uneven/skewed
  cases, and runtime bit selection satisfy the existing independent oracle
  in sequential and overlapping emissions. The overlapping image must still
  hand out nonempty output work, not just the input partition map.
- First compare the current take/restore source with reads through references
  while keeping the block size, chunk representation, packing chain, output
  allocation, and scheduler fixed. Elimination of aggregate transfers must
  be visible in generated code; a shorter source alone establishes no cost
  reduction. Any later algorithm change is a separate comparison.
- Measure compiler construction, WF analysis/lowering, native construction,
  and native execution separately. Timing uses bounded, host-locked runs on
  an otherwise idle host, retaining wall time, process CPU, worker count,
  input shape, exact sources/build flags, and observer perturbation. Loaded
  or unverified runs do not select an optimization.
- Attribute remaining costs to partitioning, count tally, packing, final
  copies, and allocation/initialization before selecting a scheduler change.
  Reuse the existing native chain and direct-scatter controls with their
  different representations stated explicitly. Keep correctness observers
  separate from timing images.

Reference-based access may remove copies; it does not supply a quantified
relation between input contents and tight scatter offsets. Padded streams,
linear packing depth, and final-copy span remain separate questions. This
continuation does not select a general grain/PGO policy or an I/O mechanism.

#### Post-port attribution boundary (2026-09-21)

The next source comparison uses merged reference-model compiler `36be8784`,
whose implementation matches the final port at `437ae287`. The September 20
measurements below remain observations of `b6aef587`, not a speedup claim on
this new compiler. Keep the reference tally and packing source, independent
stable oracle, and observations of completed helper work in both phases.
Reestablish correctness before measuring the new baseline.

The port now owns source-pair separation questions, flow-positioned proofs,
retained derivations and loop capture identities. It supersedes this
investigation's earlier optional range-proof transport and its blanket
reference-rebinding restriction. Use that shared implementation rather than
reintroducing a second permission proof channel. The port's permission cases
cover dynamic splits, endpoint rebinding, branch-local evidence, complete
multi-target conflicts, nonadjacent pairs and loop-carried generations. The
existing native three-call regression retains the independent scalar prefix,
and the scatter oracle still checks actual input and output work.

The earlier utilization investigation in [PR #64](https://github.com/mbbill/Whitefoot/pull/64),
at `70c86e5b`, supplies a hypothesis and an observer prototype, not a validated
optimization. Its only coarse phase run encountered severe contention;
ordinary W8 wall time rose from about 4 ms to 96 ms. Those timings cannot
select a change. Its snapshot-copy candidate targeted the retired owning
buffer/slice transfer path. Current stores already use `store_value_at`, and
stored typed-address loads use `load_place_result`; the old fixture and
replacement syntax no longer describe the current path. Any remaining
aggregate expansion must first be reproduced in current generated code,
then compared on identical WF source. Neither shared helper names nor the
old candidate establish that every indexed aggregate path is optimized.

Preserve the useful attribution boundary from that prototype: chunk
allocation/initialization, input partitioning, tally, digit-stream allocation,
packing, result allocation, final copy, and temporary release. Observe wall
time, process CPU and grants at fully joined boundaries, with correctness
observations separate from timing images. The old LLVM-text injector names
retired buffer labels and a two-word return ABI, so do not carry it into the
new build or formal checks unchanged. Add a current diagnostic only for the
next concrete attribution question; it remains an explicit research command.

Before selecting another optimization, compare an unobserved image with its
observed twin to qualify perturbation, then use a fixed-source control whose
predicted phase or work-supply change would distinguish the proposed cause.
Coarse CPU/wall ratios cannot distinguish serial span, limited task supply,
and published work waiting for a worker. Keep the current source shape and
recursive-budget realization fixed for that control, and protect small and
skewed inputs. If the new baseline changes the dominant cost, reconsider the
hypothesis instead of reviving the old backend patch by ancestry.

#### Joined-phase scatter experiment

This experiment starts from merged `d47fb7c7`, with the reference-reading
scatter source and compiler held fixed for the initial attribution. Its first
question is which source phase limits the large mixed-input W8 result, not
whether another scheduler can make a different algorithm faster. Use W1, W2,
W4 and W8, then retain small and skewed inputs as adverse controls for any
selected change. Native chain and direct forms give context; only an
otherwise matched WF control isolates a source or compiler mechanism.

The competing explanations and discriminating observations are:

- Allocation, initialization or tally can dominate serial wall time while
  leaving no work for other lanes. A targeted reduction must lower those
  phases and the unobserved whole-call cost, not merely raise CPU occupancy.
- The packing continuation may expose too little work or have a long source
  dependency chain. Relate its wall/CPU/grant observations to the emitted
  recursion budget and actual continuation shape before changing scheduling.
- Indexed aggregate expansion may retain avoidable element-transfer work.
  Reproduce it in current optimized IR before reviving the earlier backend
  hypothesis; a fixed-source lowering control must remove that work and its
  measured phase cost without changing the algorithm or proof obligations.
- The two final copy loops can bound useful parallel width independently of
  the pool. A change must reduce their span or required copying, rather than
  claim unused workers alone establish a runtime defect.

Use the eight fully joined phase boundaries above. A separate diagnostic
image records monotonic wall time, process CPU and cumulative grant deltas;
it must preserve the ordinary result and source joins. Keep the unobserved
image authoritative for whole-call comparisons. First qualify an
identical-image pair, then compare diagnostic and ordinary images in five
rotating passes, each with one verified warm-up and five verified warm calls.
The median within-pass wall ratio must remain within three percent of one
for quantitative phase attribution at that input/width. Retain inconclusive
controls explicitly; do not subtract the observed overhead from phase costs.

Record compiler and native construction separately from oracle execution and
timing. Reuse checked dependency builds, keep generated artifacts outside the
checkout, and run one guarded command at a time with a short stage-specific
deadline. Qualify the ordinary sequential/parallel WF images and native
controls against the unchanged stable oracle before timing. Preserve every
sample and image identity. Choose the next bounded source or lowering
experiment from this evidence; no new source proof rule, general grain policy,
or production tracing framework is selected by the diagnostic itself.

The explicit `scatter-phases` target in `research/experiments/compute-bench`
builds the diagnostic beside the ordinary `radix_scatter` image. Its Rust
injector recognizes the inspected root in both parallel and sequential worlds,
rejects missing or reordered boundaries, and inserts nine external callbacks
around the eight phases. Partition ends after the map joins; packing ends
after the recursive call joins; final copy ends before any temporary is freed.
The native chain uses the corresponding source boundaries. Callbacks collect
clock and successful-steal counters, and the existing `check` callback prints
them after the whole-call timer stops. The no-overlap and native direct forms
remain uninstrumented controls. No production compiler/runtime hook or daily
gate dependency is added. Retire this shape-specific injector when the phase
experiment no longer uses this source decomposition.

With the usual dependency/compiler overrides and scratch `BUILD`, construct
`images scatter-phases KERNELS=radix_scatter`. Both images accept the existing
`verify wf W` and `time wf W PASS CALLS` commands; select widths with
`WF_WORKERS=W`. The diagnostic adds comment records
`# scatter_phase CALL PHASE WALL_NS CPU_NS STEALS` (tab-separated), leaving
the whole-call rows unchanged. Record input grid, variant and pass with each
invocation. Phase numbers 1 through 8 follow the boundary list above; call 0
is the verified warm-up and is excluded from statistics.

The initial mixed-input run identifies chunk initialization and packing as the
largest W8 phases (about 0.596 and 0.569 ms). The ordinary whole call is 1.930
ms. The WF identical-image and observer controls satisfy the stated band at
all four widths; native-chain observer controls at W1/W2 do not. Retain their
phase data as unqualified. Short-phase process CPU deltas are also unsuitable
for attribution on this host: some exceed wall time times the entire machine's
CPU count, so neither phase utilization nor a worker-idle diagnosis follows
from them. Keep these raw readings, use phase wall time and successful steals,
and do not change the shared clock based on this scatter experiment alone.

A direct replacement of the fixed-count chunk owner with
`Box<Array<Option<Chunk>>>`, filled once with `None`, would avoid repeated
append bookkeeping if construction admitted it. It is rejected before timing:
`Chunk` owns `Slots`, which is `nocopy` under TYPE-9/OWN-1, whereas
`box_array_filled` requires a copy element under OP-13/PRE-1. A syntactically
correct call cannot satisfy that capability requirement, even when the value
is the empty `None` variant. No language rule or oracle is changed to admit
the candidate. A fixed-count owner for affine elements needs a different
construction path; it is a separate language/library question. The current
optimized initializer also retains a 4,112-byte temporary clear and a
4,120-byte copy for each appended `None`. Reducing that work needs a general
construction/transport treatment that respects initialized values, not a
scatter-specific omission of writes.

The bounded scheduling control uses the existing
`--par-recursive-frontier 32` and `--par-recursive-frontier off` options on
identical source. At W8 the default budget of 9 crosses two component entries
per chunk and therefore expands only four packing payloads before falling
back to its sequential world. A budget of 32 exposes sixteen; disabling the
budget offers at every node, still subject to the runtime's finite deque.
Predict increased packing hand-outs; a wall
improvement must survive the ordinary-image comparison on mixed and skewed
inputs without a qualified small/W1 regression beyond three percent. More
hand-outs with unchanged or worse packing/whole-call cost refute increasing
the frontier alone as the next optimization. Keep all outcomes; this control
does not select a universal budget or claim the chain has become a balanced
algorithm. Qualify any new diagnostic image against its own ordinary image.

#### Joined-phase result (2026-09-21)

The [complete samples](../../experiments/compute-bench/radix-scatter-phases-2026-09-21.tsv)
retain all three campaigns, including failed controls: 2,520 whole-call rows
(420 warm-ups and 2,100 warm calls) and 3,840 phase rows. The dataset header
records compiler/source/image hashes, dependency pins and native flags. The
compiler and runtime sources are byte-identical between the previously checked
`b11a313c` build and merged `d47fb7c7`; the existing compiler was reused and
both native images were constructed afresh. Host: Apple M1 Pro, eight CPUs,
32 GiB, arm64 macOS, Apple clang 21.0.0. The diagnostic implementation is
`8d358e53`; the formal source, compiler, runtime, specification and oracle are
unchanged in this experiment.

Each value below is the median of five per-pass medians, each from five
verified warm calls after one verified warm-up. Ratios are instead medians
of the five within-pass ratios; dividing displayed medians is not the
selection rule. The three-percent band is a control criterion, not a claimed
confidence interval. Mixed and skewed inputs have 1,048,593 keys; small has
257. Passes rotate width/variant order and alternate its direction, with no
inter-call gap. Whole-call clocks exclude fixture preparation, result checks
and release of the returned owner; temporary release inside the kernel stays
included. Process CPU includes runtime work and is not a useful-work measure.

| Ordinary mixed-input image | W1 wall / CPU ms | W2 wall / CPU ms | W4 wall / CPU ms | W8 wall / CPU ms |
|---|---:|---:|---:|---:|
| WF, default frontier | 3.376 / 3.371 | 2.565 / 3.446 | 2.113 / 3.868 | 1.930 / 5.257 |
| oneTBB, native chain | 2.117 / 2.086 | 1.538 / 2.292 | 1.501 / 3.396 | 1.477 / 4.992 |
| oneTBB, direct scatter | 1.328 / 1.321 | 0.760 / 1.143 | 0.483 / 1.239 | 0.484 / 1.890 |

The WF null ratios at W1/W2/W4/W8 are 1.0124/0.9961/1.0119/0.9850; observer
ratios are 1.0199/1.0167/1.0109/1.0022, satisfying the prior criterion.
Individual W8 observer pairs range from 0.9915 to 1.0959, so these are coarse
phase estimates, not precise cycle attribution. Native observer ratios fail
at W1 (0.9682) and W2 (1.0616); W4/W8 qualify at 0.9975/1.0217.

| Fully joined phase | WF W1 wall ms | WF W8 wall ms | WF W8 successful steals | Native chain W8 wall ms |
|---|---:|---:|---:|---:|
| Chunk allocation and initialization | 0.594 | 0.596 | 0 | 0.186 |
| Input partition | 1.720 | 0.399 | 27 | 0.266 |
| Tally | 0.021 | 0.024 | 0 | 0.015 |
| Digit-stream allocation and fill | 0.154 | 0.170 | 0 | 0.155 |
| Packing | 0.632 | 0.569 | 8 | 0.516 |
| Result allocation and fill | 0.080 | 0.080 | 0 | 0.085 |
| Final copy | 0.203 | 0.182 | 10 | 0.174 |
| Temporary release | 0.003 | 0.005 | 0 | 0.005 |

The phase medians need not sum to the whole-call median. The diagnostic's W8
whole-call median is 2.049 ms; initialization and packing account for roughly
57 percent of that estimate together. Both final WF copies retain PAR-2
range splitting, so the two source calls do not imply a two-worker ceiling.
The largest observed WF/native phase gap is initialization. Optimized LLVM
retains a per-chunk 4,112-byte clear and 4,120-byte copy in the append loop;
it also retains expanded aggregate transfers in `write_chunk`. Conversely,
the temporary whole-enum reads in tally and borrowed packing are removed by
LLVM. The raw IR alone would have misidentified those reads as a current
large-copy bottleneck.

Short-phase CPU attribution is unqualified independently of observer wall
qualification. For example, native-chain W2 pass 0 warm call 1 reports
304,000 CPU ns over 12,167 wall ns in tally, exceeding even eight CPUs' elapsed
capacity. These `task_info` counter deltas cannot establish CPU occupancy at
these boundaries. The exact accounting cause remains unverified; retain the
raw deltas rather than clamp them or charge delayed CPU to a chosen phase.

The fixed-source frontier experiment gives the following candidate/default
paired wall ratios. Its mixed-input null passes at every width. Skew W8 and
small W2/W4/W8 fail their null controls and have no performance verdict.

| Input / frontier | W1 | W2 | W4 | W8 |
|---|---:|---:|---:|---:|
| Mixed / 32 | 1.0026 | 0.9839 | 0.9778 | 1.0260 |
| Mixed / off | 1.0065 | 1.3001 | 1.3307 | 1.3676 |
| Skew / 32 | 1.0155 | 1.0040 | 0.9934 | inconclusive |
| Skew / off | 0.9964 | 1.2891 | 1.2350 | inconclusive |
| Small / 32 | 1.0123 | inconclusive | inconclusive | inconclusive |
| Small / off | 1.0124 | inconclusive | inconclusive | inconclusive |

Budget 32 supplies no beyond-band whole-call improvement. Budget off makes
every qualified parallel large-input comparison worse. Its W8 diagnostic
qualifies (observer ratio 1.0059): packing takes about 1.273 ms and records 64
successful steals, versus about 0.569 ms and eight in the earlier default
diagnostic. The source chain still holds its pending offers until recursive
return, and the runtime still has a finite deque; removing the compiler cut
does not turn this into balanced independent block work. The 32/W8 diagnostic
(0.9656) and off/W4 diagnostic (1.0310) fail their observer controls; do not
use their phase timing to support the conclusion. The ordinary comparisons
remain usable where their null passed. No frontier change is adopted.

The resulting priority is to investigate aggregate construction/transport on
the unchanged source, with an independent enum/affine correctness boundary,
before expanding this task into a new array constructor or a general grain
policy. Packing needs a control that changes its dependency span or batching,
not merely the number of tiny offers. These remain open costs in
`docs/todo.md`; this result does not implement either optimization or prove
that the runtime has no other scheduling costs. Current proof/permission
architecture and approved frontier defaults are unchanged; no tree amendment
or specification revision is proposed by this measurement.

Construction and execution were separately bounded and timed under the shared
guard. Waiting for other worktrees is excluded from these stage times.

| Stage | What ran | Elapsed |
|---|---|---:|
| WF emission | Existing gate compiler emitted parallel/sequential LLVM | 0.32 s |
| Native construction | Clang/clang++, shared runtime/backends, ordinary and diagnostic images | 3.10 s |
| Initial oracle | 15 invocations of the unchanged 109-configuration stable-result matrix | 1.30 s |
| Identical-image timing | Five rotating passes, W1/W2/W4/W8 | 1.37 s |
| Observer/context timing | WF and native observer pairs plus direct native context | 2.85 s |
| Frontier construction | Two compiler invocations (0.09/0.08 s), native ordinary/diagnostic objects and links | 2.04 s total |
| Frontier oracle | Eight matrix invocations | 1.65 s |
| Frontier timing | Five rotating passes, mixed/skew/small, four widths | 7.89 s |

All 23 oracle invocations passed; every timed call was also checked. The
instrumenter rejected a deliberately renamed packing boundary without
writing an output module; the admitted ordinary source receives exactly nine
callbacks in each root. No Rust compiler/unit-test rebuild was needed for
this research-only change. These measurements do not claim a new full gate.

To reproduce, use the existing benchmark Makefile and dependency pins, an
external scratch `BUILD`, and guarded commands. Build `images scatter-phases
KERNELS=radix_scatter` with the selected `WFC`. The two frontier controls use
that same compiler/source with `--par --par-recursive-frontier 32` or `off`;
bind/prefix the emitted module with the same host adapter and
`module-symbols.awk`, then replace only the parallel module object in the
ordinary image's link recipe. For their diagnostic twins, pass the raw
module through `radix_scatter_phases` first and link the phase callback object.
The sequential module, native backends, runtime objects, flags and link order
are shared. Before timing, run `verify wf W` at W1/W8 for each control and
diagnostic (the default WF image was verified at all four widths). Native
context uses `verify tbb W` and `WFB_SCATTER_NATIVE=chain|direct` at W1/W8;
`verify wf-seq 1` checks the no-overlap control. Each timing invocation is
`WF_WORKERS=W WFB_GAP_US=0 WFB_SCATTER_GRID=GRID IMAGE time FORM W PASS 5`;
native context also selects `WFB_SCATTER_NATIVE`. The dataset records every
invocation and its order. Reduce warm calls to per-pass medians, then pair
matching passes before taking the median ratio. Keep null/observer failures
and phase-CPU limitations separate from the whole-call results.

#### Initial compatibility checkpoint

At `efd6ebc9`, the guarded gate-profile library-test executable construction
took 72.28 s; constructing the compiler executable separately took 40.03 s.
These are construction costs, not native program execution. Directly running
the four existing `backend::tests::ranges` test functions below took 10.66 s
overall with one test thread. Each function includes its WF analysis/lowering
and, when reached, native construction and execution; the table does not
attribute that combined duration to any one of those stages.

| Existing observation | Test-function wall time | Result on the initial checkpoint |
|---|---:|---|
| `blocked_compute_matches_independent_oracles_at_runtime_dimensions` | 5.21 s | Prefix and histogram native oracle checks passed. |
| `stencil_matches_an_independent_dimension_and_step_matrix` | 4.52 s | Native dimension/step oracle checks passed. |
| `stable_scatter_matches_an_independent_oracle_and_hands_out_output_work` | 0.02 s | Semantic analysis stopped at unsupported `CompositeValues`; no native scatter ran. |
| `irregular_compute_matches_independent_sort_and_graph_oracles` | 0.33 s | `merge_sort.wf:112` failed FN-8 at the `merge_values(first: a1, second: b1, output: out1)` call: `deref(a1).len <= deref(out1).len` was unproved. The test did not reach BFS. |

The merge-sort observation is a reproducer for PR #70's owner, not a changed
language expectation or a diagnosis of the underlying proof defect. It is
outside the immediate scatter repair. The scatter stop is also reproduced by
an otherwise empty program whose only helper takes `&[Option<Chunk>]` and
returns `deref(chunks).len`, with `Chunk` owning the same two inline runs. This
isolates an existing represented nominal element from chunk transfer,
parallel permission, or native execution.

The first repair uses the storage element domain already represented by
`buffer_element` for range formation, re-slicing, length reads and indexed
access. The same minimal helper then emits LLVM successfully; this is not
yet evidence of native scatter correctness. The next stop is OP-4 at
`&deref(chunks)[0_u64]`: reference formation treats a range parameter's
element type as the indexable base. PR #70's owner should complete that
general reference path. The consumer comparison first uses an equivalent
storage reference plus an explicit chunk position on both arms, retaining
the block decomposition, packing continuation, capacities and element work.
Only after that shared form passes the independent oracle can its owned
take/restore and reference-read versions select a cost conclusion.

The retained scatter benchmark also needs the formal adapter's current
output-handle ABI. Its entry returns an element pointer and length for
checking plus a separate owning Box handle for release. Both WF forms retain
that handle until the check, while native forms use their allocated pointer
as the handle. Release remains outside the timed call. This is an interface
repair, with no change to the native algorithms or timing boundaries; C
syntax/type checking passed, and full harness execution remains unverified.

The existing `ref4-pos-range-reference-and-reslice` conformance case now also
forms and re-slices a range of affine `Option<Slots<u64, 1>>` elements, replaces
an element through that range, and checks the original slot. Its expectation
is strengthened from acceptance to exit-zero execution, so its existing
scalar length checks also execute. This adds one native construction/run to
the same corpus case, not another Rust test executable or duplicate WF case.
On the repaired compiler, WF analysis plus native construction took 0.57 s
and process launch/execution took 0.36 s, both exit zero. The initial compiler
stops on this extended case as unsupported. Changing the replacement offset
to an unproved `1_u64` still rejects at OP-4; the element-domain repair does
not waive the subscript bound. `make conformance` passed its 28 runner tests
and reported 125/125 rules covered.

#### Reference-source and tally probes

The reference-source probe, initially retained beside this investigation,
keeps the current block decomposition and padded streams. Tally reads a
chunk in place; packing reads the containing Box through a reference and
advances an explicit chunk position. The whole-Box form follows the current
TYPE-9 placement rule. Explicit finite steps carry the same padded-capacity
argument through that position. The formal scatter fixture was unchanged at
this initial checkpoint; the integrated comparison below now uses its
maintained source and removes the temporary research copy.

With compiler sources at `0f9edadd`, the candidate reaches FN-8 at
`copy_run(values: &deref(payload).low, output: first_low)`: the substituted
requirement names `deref(chunks).inner[first].len`, losing the payload and
`low` field. PR #70's in-progress changes already address payload projections
in `goal_referent_image`; they are not independently applied here. The owned
control on the same Box/position signature additionally loses the expected
remaining-range length relation after the output copies. These observations
must be rechecked on a committed upstream checkpoint before timing.

An isolated code-shape probe extracts the `Chunk` and `tally` definitions
from `efd6ebc9:tests/programs/compute/radix_scatter.wf` and from the candidate,
appending the same empty `main` returning exit zero. Both sources emit LLVM
through `whitefootc --emit-llvm SOURCE -o MODULE.ll`. Apple clang 21.0.0 on
arm64 Darwin compiles each with
`clang -O2 -Wno-override-module -S -x ir MODULE.ll -o MODULE.s`. The `_wf_tally`
body, bounded by its label and `-- End function`, has 954 assembly instruction
lines in the owned form and 11 in the reference form. The owned form reserves
4,096 stack bytes plus 64 bytes of saved registers; the reference form has no
stack frame. Its live path reads the tag and two lengths, adds the prior
counts and stores two scalars. Although its unoptimized LLVM contains an
aggregate snapshot, native optimization removes that unused payload copy.
These are isolated emitted-code observations, not elapsed-time results or
an end-to-end scatter speedup; caller placement and full consumer correctness
remain to be checked.

#### Output-overlap checkpoint

A diagnostic compiler snapshot from PR #70, SHA-256
`32c6f0dfda8c301268406a1cff397fbb47d1ea32a79f5891796925b99b1378f7`,
preserves remaining-range lengths across the copy calls. This is an
uncommitted upstream binary, not an integrated compiler revision or a timing
baseline. A helper taking the payload as an ordinary `&Chunk` avoids the
remaining nested-payload contract-image defect. Writing each partition result
through `&Option<Chunk>` also avoids the nominal range-element reference path;
both source forms still need their general compiler repairs. The Box/position
entry needs the explicit finite step `256 times (chunks.inner.len <= blocks)`
to establish the padded capacity requirement. That missing step is a source
proof requirement, not a compiler defect.

With those source changes, the unchanged native oracle passes all 109
configurations and 3,466,725 output values sequentially and in the parallel
image's one-worker fallback. WF emission took 0.07 s, runtime-object construction
0.39 s, sequential native construction 0.52 s, and oracle launch/execution
0.37 s. These are diagnostic verification costs, not kernel measurements.
At two workers the output values still agree, but the required packing observer
reports no steals or nonempty helper output; the parallel correctness check
therefore fails. No performance selection follows from this snapshot.

The permission ledger admits the two adjacent `copy_run` calls but denies the
following recursive pack because `first_high` and `rest_high` are not separated.
`permission::footprint_conflict` currently uses `UnprovedSeparations`, so it
cannot use the ordinary arithmetic proof of those sibling ranges. Separately,
the lowering keeps only the prefix of a permitted run; a preceding non-call
statement hides even the admitted two-call suffix. The first repair retains
each contiguous call subrun, ending it at a non-call, unavailable call result,
block change, or addressed result. The existing native three-call/join-order
case now includes an independent non-call prefix and passes (1.01 s inside the
test function; 1.61 s including the guarded process). Constructing its updated
gate-profile Rust test executable took 68.55 s. No extra WF compilation or
native executable is added to that test's normal schedule.

The next compiler comparison keeps acceptance and the four fixed range
separation queries unchanged. Optional PAR-1 separation evidence must belong
to the argument pair and the state before its earlier statement; a proof
obtained under one branch must not grant overlap after a join or in another
branch. Permission composition must still compare every conflicting access
against the run's accumulated footprint. The discriminating controls are a
permitted split, an overlapping split, endpoint reassignment, and a
branch-local separation that cannot escape its branch, followed by the native
packing observer. Missing evidence remains a sequential permission outcome.
The payload helper also changes recursive call-graph depth, so its temporary
shape is not a valid comparison with the old chain's recursion budget; a
timing comparison must give both arms the same helper boundaries or remove
that helper after the upstream contract-image repair.

#### Integrated reference checkpoint

At this checkpoint the investigation built on PR #70's committed `206c0cc1`. Its broader
reference/storage work subsumes the initial nominal-range repair, and its
overlap lowering includes contiguous call subruns. Those duplicate compiler
changes are absent from this branch's diff; the strengthened conformance case
and non-call-prefix regression remain. Constructing the integrated gate-profile
compiler executable took 42.05 s (41.94 s reported by Cargo).

The new optional permission transport passes a direct ledger probe for a
split whose scalar endpoint is subsequently assigned, refuses an overlapping
split formed after endpoint assignment, and permits a conditional split only
inside its proving branch. These probes exercise the ordinary compiler;
retained-proof replay and the full native scatter observer are still pending
at this checkpoint. No acceptance requirement or source rule is changed.

Two upstream limitations remain reproducible on this committed base. Direct
reference packing stops at the first `copy_run` requirement, now correctly
spelled `deref(chunks).inner[first].Some.value.low.len <=
deref(first_low).len` but unproved. Separately, a loop that starts with
`previous` and `current` both referring to `[0..1]`, then assigns
`previous = current` and `current = &part[i..i+1]`, stops as unsupported
`OwnershipJoin`. That loop was an exploratory negative permission control;
it cannot yet exercise this branch's overlap judgment. Its normative behavior
has not been changed, and completing reference flow through that loop belongs
to #70. The permission test uses an ordinary supported rebinding instead.

The complete source comparison uses the same Box/position packing boundary
and a separate payload helper on both arms. The owned control retains
take/restore tally and consumes each chunk while packing; the reference arm
reads both in place. Both have the same two-function recursive component,
block size, allocations, output copies and scheduling options. This ports the
formal consumer to an executable control before selecting the reference
rewrite; the control is preserved by its git revision for reproduction.

On the integrated compiler, both arms pass the unchanged oracle at one, two
and four workers: 109 configurations and 3,466,725 values per invocation.
The reference observer reports 27,383 and 41,247 nonempty helper output words
at two and four workers; the owned control reports 29,734 and 47,068. These
varying counts establish useful output work, not a performance ranking.
The reference and owned native observer constructions took 0.65 s and 2.47 s;
their three-process verification invocations took 0.53 s and 0.64 s. Runtime
objects were reused after checking that their sources match the integrated
base. Kernel-only elapsed-time comparisons remain separate.

Before the timing comparison, the attribution arms are owned tally/owned
packing (the formal source at `2d8ce0d7`), reference tally only, reference
packing only, and both reference reads. Swap the complete `tally` definition
and its count-loop call/restore section between the two endpoint sources to
construct the intermediate arms. Their helper boundaries and recursive
component remain identical. All arms use the same compiler, native support
objects, harness and link order, and must pass the independent oracle before
timing. An identical-reference-image paired control runs first. If its median
within-pass warm wall-time ratio differs from one by more than three percent
at a selected worker count, that session is inconclusive for selecting a
performance claim there. Record five alternating passes of five verified warm
calls after a verified warm-up, wall and process CPU separately, at 1, 2, 4
and 8 workers. Use the same comparison on mixed, all-low and skewed inputs;
the small-input case is a latency control. No result changes the scheduler
or the language's acceptance rules.

The first source-only comparison at `36acce03` holds the direct chunk-slot
reference workaround fixed on all four arms. On mixed inputs, one-worker
median wall time is 11.63 ms owned, 8.09 ms with reference tally, 7.02 ms with
reference packing, and 3.55 ms with both. All large-input identical-image
controls pass the stated three-percent condition; the four-worker small-input
control does not. These results isolate aggregate-transfer costs in that
intermediate source, not the final parallel program: its partition loop is
denied PAR-2 because the whole nominal-element reference does not retain an
admitted element-map witness. The 8-worker reference image therefore has only
about 1.26 occupied CPUs, despite its useful output offers.

The committed upstream now accepts the original one-element output-range
form, which restores the partition map without a compiler change. It is
restored before the final comparison, identically on all four arms. The
existing native scatter observation now also checks completed nonempty input
partitions on another thread, resetting the scheduling observer only after
those maps join and before output packing begins. At two/four workers it
observes 1,540,096/2,326,528 helper input words and 26,795/37,856 helper output
words, with all 109 configurations still correct. A diagnostic emitted-code
control that returns zero from every independent-range split-budget call
fails specifically with `oracle observed no nonempty helper input partition`.
No extra native image or configuration is added to the maintained test's
schedule; the mutation is a one-shot validation of the stronger observer.

The integrated focused suite passes 74 tests in 12.16 s (12.69 s including
guarded launch): all loop-permission controls, the new source-scoped range
proof/metadata control, the non-call-prefix native group regression, and the
existing prefix/histogram, stencil, sort/BFS and scatter native oracles. The
initial merge-sort failure is fixed by the integrated upstream checkpoint;
both sort and graph now execute. Rebuilding the gate-profile Rust library-test
executable for the final input/output observation took 71.83 s. This is a
construction cost and is separate from the 12.16 s test execution. The broader
permission-only selection still exposes two unchanged upstream tests,
`a_scrutinee_call_forms_no_pair` and
`a_scrutinee_call_written_first_forms_no_pair`, whose expectations predate
#70's call-rooted match support. They are recorded for its owner, not rewritten
as part of scatter.

#### Reference-model scatter result (2026-09-20)

The [retained samples](../../experiments/compute-bench/radix-scatter-reference-2026-09-20.tsv)
identify compiler, source and image hashes, flags, the two source-shape
checkpoints, every sample and the reductions. The final source is
`ebd3e4b7:tests/programs/compute/radix_scatter.wf`. Construct its owned control
from `2d8ce0d7` by restoring the same `write_chunk` range parameter and
`block..after` destination on both arms; swap the complete tally definition
and count-loop section to obtain the two intermediate arms. The compiler
implementation is `b6aef587`, unchanged through these source revisions, on
PR #70 at `206c0cc1`. The host is an unpinned Apple M1 Pro with eight CPUs,
with other guarded builds excluded and ordinary background applications still
present. Apple clang 21 compiles WF and its runtime at `-O2`; the retained
native controls use the bundle's scalar `-O3` flags. Native support, harness
objects and link order are shared within each source comparison.

Every final WF image passes sequential emission and the parallel emission at
1, 2, 4 and 8 workers against 109 configurations / 3,466,725 values. Both
oneTBB controls pass the same matrix at those widths. Those 28 process
invocations take 2.46 s, including launches, and contain no timing protocol.
The reference measurement image's complete construction takes 2.04 s;
emitting the other three arms in both modes takes 0.75 s, and constructing
their native objects/images takes 10.50 s with the common objects reused.

The final identical-image session takes 4.71 s and the six-form measurement
session 19.18 s. These are total session costs, not kernel times. The following
kernel medians exclude input/oracle preparation and result checking/release,
but include the kernel's allocations, initialization, temporary releases and
parallel joins. Each process first executes a verified warm-up and then five
verified warm calls, over five alternating passes with zero call gap.

For 1,048,593 mixed keys at one worker, the independent source changes show
where the removed cost was paid:

| Source arm | Warm wall (ms) | Process CPU (ms) |
|---|---:|---:|
| Owned tally and packing | 11.575 | 11.563 |
| Reference tally only | 8.078 | 8.068 |
| Reference packing only | 7.057 | 7.018 |
| Both reference reads | 3.531 | 3.523 |

The two reductions are nearly additive here: about 3.5 ms from count
extraction and 4.5 ms from packing transfers. This is a controlled attribution
to those source regions, supported by the isolated tally code-shape probe;
it is not a claim that their reference implementations take zero time.

| Workers | Owned wall / CPU (ms) | Reference wall / CPU (ms) | Paired wall speedup |
|---|---:|---:|---:|
| 1 | 11.575 / 11.563 | 3.531 / 3.523 | 3.29x |
| 2 | 10.933 / 11.973 | 2.740 / 3.692 | 3.96x |
| 4 | 10.442 / 12.350 | 2.236 / 4.159 | 4.73x |
| 8 | 10.285 / 13.779 | 2.058 / 5.415 | 4.98x |

Ratios are medians of within-pass ratios, not quotients of the displayed
medians. At eight workers the all-low and skewed inputs improve by 4.79x and
4.61x respectively. Large-input identical-image controls all remain within
the predeclared three-percent wall condition. The 257-key control fails that
condition at four and eight workers, so those latency comparisons remain
inconclusive. This is one host and workload family, not a portable speedup
guarantee or an automatic performance-regression verdict.

The improvement does not finish scatter's cost investigation. At eight
workers the reference WF image takes 2.058 ms wall / 5.415 ms CPU, versus
oneTBB chain's 1.489 / 5.894 and direct scatter's 0.437 / 2.021. The chain
retains the chunk/padded-stream decomposition but has a different callable
representation and recursive-budget realization: WF currently crosses two
functions per recursive chunk, while the native control decrements its
explicit budget once. The direct control also removes local element streams
and the packing chain, changing the algorithm and storage. Neither is a
same-source compiler comparison.

CPU divided by wall is about 2.63 occupied CPUs for WF versus 3.96 for the
native chain, even though WF uses less total CPU. Actual helper input and
output work are now verified, but these totals do not distinguish serial
initialization/counting and the packing continuation's span from runnable
work waiting for a worker. The surviving costs are padded storage and its
construction, payload construction during partitioning, the linear packing
continuation, final copies, and remaining scheduling/placement effects.
Attribute those before choosing a different task shape or scheduler policy.
The native direct result motivates further investigation of a safe tight
destination representation; it does not supply the missing content/count
proof for the current direct WF candidate. General profile/PGO policy and
I/O remain outside this result.

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
[`radix_scatter.wf`](../../../tests/programs/compute/radix_scatter.wf)
first partitions each input block into two `Slots<u64, 256>` runs.
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
input. The reference-model continuation above reads counts and payloads in
place; earlier measurements below include the legacy source's owned
take/restore transfers. Initialization and the continuation's linear depth
remain runtime costs rather than proof work. The native controls compare both
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
with the unresolved cost tracked in [TODO](../../../docs/todo.md). That trial
added no profile collection or adaptive policy.

### Read-only Box-array helper work pricing

The initial check at `3402048f` loses the runtime extent in a helper over
`&Box<Array<u64>>`: a native observer sees an outer iteration price of 123 at
input lengths 0, 1, 17, 4,096 and 65,536. An equivalent range helper retains
prices 10, 15, 95, 20,490 and 327,690, respectively. Both forms return the
independently checked result `14 * length`. Their per-element lowering costs
differ, so equal prices are not the criterion; retaining the helper's extent
is. This is a current representation-dependent summary gap, separate from the
historical grain-policy trial and its continuation-accounting correction.

The repair follows only the checked direct Box-to-runtime-Array
projection. Lowering copies the existing checked absence of writes to an
original reference formal's root. An exact direct chunk capture retains that
fact; rebinding and reconstructed owned captures do not. Work estimation can
then carry a typed Box-array length observation, and the emitter uses the
ordinary Box/header projection. The source call establishes the original
referent's validity. The originating typed length read supplies an exhibited
read effect even in a zero-trip body under EFF-2, and EFF-5 separates it from
reference writes and by-value consumption throughout the call. The no-write
marker alone does not establish the lifetime of an unused formal. A reference
type alone does not establish this lifetime: the former capture-all lowering
could retain an already consumed owner, and pricing must not dereference it.
The accepted extension is recorded in the
[parallel-lowering decision](../../../design/compiler/parallel-lowering.md).

The focused native regression requires increasing prices for original Box,
range and shared read-only alias helpers, unchanged static estimates for
mutable formals, rebound values, local owners and an unrelated `Box<u64>`, and
correct complete results at zero and nonzero inner lengths and outer trip
counts. The existing EFF-5 substituted read/write-alias rejection covers the
checker fact used by lowering. Existing total-estimate and post-loop
continuation-accounting regressions remain protected. Compare emitted modules
for prefix, histogram, BFS and the five formal performance kernels on identical
source; any changed module needs explanation before extending this narrow
comparison to an affected workload.

Before timing the candidate, the performance witness and criterion are fixed:
a batch of 512 independently seeded sequential hashes over 16,384 words,
called through a read-only Box-array helper, with every result and the unchanged
input checked against a separate C calculation. The intended primary W4 result
is actual useful steals where the baseline has none, at least ten percent lower
median paired wall time and a faster candidate in at least four of five pairs.
Report process CPU separately and investigate an increase over fifteen percent.
Protect W1's sequential entry and a 17-word, two-result case without applying a
percentage verdict to the latter's sub-microsecond work; W2 and W8 provide
bounded scaling context. Five rotated passes use one checked warmup and five
warm calls per process, alternating arm order. Fixture construction, oracle
work, checking and release remain outside the interval. An identical-image
control precedes the compiler comparison. Apply the maintained comparison's
three-percent, two-width noise screen at W1/W2/W4 to this control in either
direction: two widths consistently favoring one arm in four of five pairs
make timing inconclusive. The tiny case and W8 remain observations. This
comparison selects no scheduler constant or new grain policy. The cell order
is primary W1/W2/W4/W8, then tiny W1/W4, rotated left by the pass number minus
one. Odd passes run baseline then candidate; even passes reverse those arms.
Reduce each process's five warm calls to medians, then report the median of
the five paired candidate/baseline ratios.

The native witness on `3402048f` with the repair observes Box-helper prices
11, 18, 130, 28,683 and 458,763 at the same five lengths, with unchanged
range-helper prices and independently correct results. A strict increasing-price
assertion now makes the probe fail if only the output is correct. The retained
summary is `7 * length + 5`; the outer iteration's own instructions produce
`7 * length + 11`. The generic `ContainerMeasure(Length)` operation and the
buffer-specific measure both feed the typed Array-length observation. Reading
only the older buffer-specific operation misses the current helper path.
The same two-arm witness passes with independently built `6fdb6768` baseline
and `cabae235` candidate compilers: baseline Box prices remain 123 and candidate
prices grow as above. The exact source and native oracle of the maintained
seven-arm regression also pass in a separate 1.14-second preflight. At lengths
0, 1, 17 and 4,096, mutable, rebound and local-owner estimates stay 123, the
unrelated `Box<u64>` estimate stays 91, and the shared read-only alias estimate
grows from 19 to 57,363. Complete output checks and zero-trip outer calls pass.

The new EFF-5 conformance pair retains a header read inside a zero-trip body
and passes a sibling affine field by value at the same call. Both baseline
and candidate reject the same-owner case, comparing the referenced field with
consumption of the complete owner root, and accept the unrelated-owner
control. This tests the existing EFF-2/OWN-1/EFF-5 safety premise without
changing a source rule. The four source checks take 0.21 seconds together.
After integrating main `c6cd9add`, candidate `7895c9d0` passes the five focused
Rust tests: the seven-arm native regression, total estimates for empty and
inverted ranges, post-loop continuation accounting, helper-summary lowering
and the existing EFF-5 alias rejection. Test-image construction took 82.81 s;
the three native builds within the selected tests took 0.925, 0.791 and
0.827 s. The conformance pair also passes directly with the current candidate.

#### Matched helper result, 2026-09-22 UTC

The retained [WF source](../../experiments/compute-bench/array_reference.wf),
[C oracle](../../experiments/compute-bench/array_reference_bench.c) and
[ordinary ABI adapter](../../experiments/compute-bench/array_reference_host.ll)
have an explicit manual build target in the existing compute-bench Makefile.
Their [raw rows](../../experiments/compute-bench/array-reference-work-2026-09-22.tsv)
retain the identical-image control, matched comparison and separately labelled
tiny-call resolution diagnostic, including warmups. They are research evidence,
with no daily correctness dependency.

The host is MacBookPro18,3 with eight physical/logical CPUs in performance
levels of six and two cores, Apple clang 21.0.0 and Rust 1.98.1. The baseline
compiler is an independently built `6fdb6768` image, SHA-256
`d6ba9286f877df7e2a2d9e7d751d415871b2d2d992d558a2d9e37ad14e3a32c5`.
The candidate used for this comparison is `7895c9d0`, SHA-256
`90711761755287a55b2859c46d03772a862ca0d212287a584992c25c4ef4a563`.
Its experiment module is byte-identical to the independently built `cabae235`
candidate from the earlier main revision. Full emitted modules for prefix,
histogram, BFS, Mandelbrot, records, FIR, quadrature and stencil are identical
across all three compilers, as is the sequential experiment module. Runtime
sources and the formal host adapter are unchanged. No metadata was stripped.
Thus the older baseline remains a matched code comparison after the main
integration; it is not presented as a newly built current-main compiler.

After integrating main `f3cf41d4`, the built `e100682d` compiler, SHA-256
`8a11c347fb84d1f6ae323e605f8ab9c7301d1add7e913e5b2ab943186747b395`,
passes exact correspondence with the retained `7895c9d0` output. All ten
complete LLVM modules are byte-identical: the eight protected parallel kernels
listed above, plus the hash experiment's parallel and sequential modules.
No normalization or metadata removal was used. Runtime sources, WF fixtures
and the formal host adapter are unchanged. The guarded emission-and-comparison
stage passed in 1.07 s, with 1.04 s reported for the command itself; it reused
the existing compiler and performed source emission and byte checks only.
This extends the dated evidence's applicability to `e100682d` while preserving
the recorded timing revisions, image hashes and results.

The only parallel module changes are the split-site header length load and
saturating work arithmetic, plus the two required intrinsic declarations.
The old price is 154; the new price is `9 * length + 10`, or 147,466 at the
primary length. Hash bodies and chunk bodies are unchanged. Native images
use the ordinary `-std=c11 -pthread -O2 -Wno-override-module` flags and runtime,
without LTO or extra scheduling instrumentation. Their SHA-256 identities are
`0cec62229866b74b440de7633a19d1c35b4a2c3db5890d2fe4344aa01030454f`
(baseline) and
`ca604f608f6bef8a8da5b10d6fcfd4e5ea1916da603123956aa7e69f899a309e`
(candidate). Whole-output and unchanged-input checks pass at W1/W4 for both
primary and tiny fixtures. Native construction took 1.25 s and the oracle
preflight took 1.04 s; the fixed null and matched timing stages took 2.09 s and
2.80 s respectively.

The identical-image primary wall ratios at W1/W2/W4 are
0.9946/1.0006/1.0039, passing the prospective noise screen. Matched results:

| Workers | Baseline wall, ms | Candidate wall, ms | Paired wall ratio | Paired CPU ratio | Faster pairs | Median steals, baseline/candidate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 10.618 | 10.541 | 0.9926 | 0.9911 | 4/5 | 0/0 |
| 2 | 10.486 | 5.403 | 0.5159 | 1.0239 | 5/5 | 0/1 |
| 4 | 10.494 | 2.821 | 0.2698 | 1.0522 | 5/5 | 0/5 |
| 8 | 10.499 | 1.742 | 0.1659 | 1.1921 | 5/5 | 0/31 |

The preselected W4 criterion passes: 73.0 percent lower paired wall time,
five of five faster pairs and actual useful steals, with 5.2 percent more
process CPU. W1 selects the identical sequential world and has no offers.
The tiny W1/W4 fixture also has no offers, but its single-call wall readings
quantize to 0 or 1,000 ns and cannot establish an absolute overhead bound;
no tiny ratio is interpreted. The post-hoc resolution diagnostic below retains
these original images and rows.

W8's 19.2 percent CPU increase is a real tradeoff in this comparison. Its
per-pair CPU ratios range from 1.150 to 1.227. The unchanged policy allows
64 leaves at W4 and 128 at W8 for this price; median actual steals rise from
five to 31, and W8 uses the host's two performance levels. Additional
scheduling and slower-core execution are plausible contributors, but this
experiment does not isolate their shares. W8 buys a further roughly
38 percent wall reduction relative to the W4 candidate. Record that cost and
uncertainty without selecting a new grain, queue or worker-width policy.
The bounded pricing repair meets its primary criterion; full gate and
independent completion review remain pending.

#### Tiny-call resolution diagnostic

Before this separate diagnostic ran, its scope was fixed at 4,096 calls of
the unchanged 17-word, two-result WF case per interval, at the protected W1
and W4 widths only. It addresses the unresolved possibility of more than
one microsecond added cost per call, using absolute differences and an
identical-image control rather than a percentage verdict. Each process
contains one warmup interval and five measured intervals. Five paired passes
alternate arm order; odd passes use widths W1 then W4 and baseline then
candidate, while even passes reverse both. The null comparison runs the
candidate image under both arm labels before the matched comparison. No
fixture, work-size, worker-width or policy search follows the observation.

The manual target's `ARRAY_REFERENCE_REPEATS=4096` changes only the C host's
number of calls inside an interval. The same allocations, inputs, WF code,
whole-output and unchanged-input checks surround each interval. Both emitted
WF modules compare byte-for-byte with the corresponding primary modules;
all calls remain externally linked without LTO or floating-point changes.
The diagnostic image SHA-256 identities are
`6ace664888bf39971a28d96a8a3f03f33a57bab9cdc6ed300c4ed616fcc997ff`
(baseline) and
`c6db980e863a7bb92fb7bd8702ea86f8df15444ecbb4979c525c164d32f2ddc0`
(candidate). Original primary image hashes are unchanged. The retained raw
rows are labelled `batched-null-4096` and `batched-matched-4096`; their wall and
CPU columns contain interval totals, divided by 4,096 before the same
within-process median reduction. All output and input checks pass, with zero
actual steals throughout. Image construction took 1.25 s, null execution
0.63 s and matched execution 0.41 s.

The largest absolute null paired wall difference is 4.151 ns per call. The
largest matched candidate increase is 5.616 ns per call; even adding the
observed null variation gives less than 0.01 microseconds, well below the
one-microsecond question. Median paired wall differences are 0 ns at W1 and
+0.244 ns at W4. One W1 baseline pair is much slower and yields a -31.494 ns
candidate difference; it remains in the raw data and is not treated as a
speedup. These are observed differences on this host, not a universal cost
bound or a percentage improvement claim.

CPU resolution is separate: the recorded 1,000 ns interval quantum becomes
0.244 ns per call. The largest null paired CPU difference is 2.930 ns per
call and the largest matched candidate increase is 5.372 ns per call; median
paired CPU differences are 0 ns at W1 and +0.488 ns at W4. The diagnostic
resolves the original tiny-call protection question without replacing the
primary experiment or its clock-unresolved individual-call rows.

## Frozen compute baseline, 2026-09-21

The [retained evidence](../../experiments/compute-bench/compute-baseline-2026-09-21.tsv)
measures source, compiler and native runtime revision
`6fdb6768890b91283fb041dfc7981a2cc2f3e5b8`, with only the research caller's
retained-result-handle adaptation at `f19d53e2` (published as `0cc0e7f4`). It is
a dated baseline, not a measurement of later main revisions. The compiler
image SHA-256 is `d6ba9286f877df7e2a2d9e7d751d415871b2d2d992d558a2d9e37ad14e3a32c5`;
the evidence file is `8fb90790a6fba27a375c4256472d57188204d6b391577d247fa7734a6f210cb6`.
The adaptation separates borrowed output data from its retained owner handle;
checking and owner release remain outside timing, and algorithms and fixtures
are unchanged. Host: unpinned Apple M1 Pro, eight CPUs (six performance/two
efficiency cores), Darwin arm64, Apple clang 21.0.0 and Rust 1.98.1. The shared
guard excludes competing guarded work, not ordinary host activity.

Before measurement, the scope was fixed to six primary fixtures and four
distinct controls below, workers 1/2/4, five rotated/reversed passes, and one
retained first plus five warm calls per process. No baseline ratio selects a
policy. Every call checks the complete independent output oracle; all 20
null/main actions and 4,290 calls pass, with compiler, dependency, source and
image hashes unchanged. The prior 166 calibration calls check cost only and
remain separately labeled. No control was repeated to obtain a favorable
result. Wall and CPU below are medians of process medians, in milliseconds;
steals are the runtime observation, not the static `chunks=na` header.

| Fixture | WF wall W1 / W2 / W4 | WF CPU W1 / W2 / W4 | Steals W1 / W2 / W4 |
|---|---:|---:|---:|
| `prefix-large`: 4,194,321 words, block 4,096 | 2.522 / 1.755 / 1.276 | 2.514 / 3.117 / 3.219 | 0 / 5 / 20 |
| `histogram-large`: same count/block, 256 buckets | 2.999 / 1.696 / 1.030 | 2.963 / 3.025 / 3.192 | 0 / 1 / 10 |
| `stencil-large`: 1,024 by 4,096, 16 steps | 31.141 / 27.205 / 17.133 | 30.635 / 51.866 / 62.084 | 0 / 21 / 163 |
| `chain-pull`: 4,097 vertices | 15.229 / 14.953 / 14.994 | 15.209 / 14.882 / 14.875 | 0 / 0 / 0 |
| `fir`: 524,288 outputs, 64 taps | 13.425 / 6.926 / 3.649 | 13.347 / 13.623 / 13.985 | 0 / 1 / 9 |
| `quadrature`: 64 adaptive integrations | 8.312 / 4.657 / 3.135 | 8.294 / 8.162 / 10.759 | 0 / 445 / 1,013 |
| `prefix-coarse`: block 65,536 | 2.513 / 1.711 / 1.272 | 2.507 / 2.864 / 2.776 | 0 / 4 / 22 |
| `histogram-small`: 17 words, block 3 | 0.00350 / 0.00479 / 0.00492 | 0.002 / 0.004 / 0.004 | 0 / 0 / 0 |
| `stencil-narrow`: 17 by 4,096, 16 steps | 0.490 / 0.430 / 0.264 | 0.489 / 0.429 / 0.269 | 0 / 17 / 51 |
| `chain-sparse`: 4,097 vertices | 0.01521 / 0.01533 / 0.01563 | 0.014 / 0.014 / 0.014 | 0 / 0 / 0 |

Regular kernels perform checked work on additional lanes. Paired W1/W4 wall
speedups are 1.97 for prefix, 2.92 for histogram, 3.68 for FIR and 2.65 for
quadrature. Wide stencil reaches 1.83 while its paired CPU ratio reaches 2.01;
extra offers are useful but their cost remains material. Tiny histogram stays
unsplit yet adds about 1.3--1.4 microseconds at W2/W4. These observations do not
select a work floor, offer policy or adaptive mechanism.

Chain pull's emitted iteration price is 55. At the unchanged 150,000-unit
floor, `4097 / ceil(150000 / 55) = 1`, which affords no split and agrees with
zero steals. Its 16,785,409 full-vertex visits are a materially different
algorithm from the sparse traversal's 16,388 adjacency-slot visits. The useful
FIFO serial reference takes 21.4 microseconds, and WF sparse takes about 15.2
microseconds; a pull/native ratio cannot stand in for useful graph efficiency.
Across the ten fixtures, paired WF W1/`wf-seq` medians range from 0.988 to 1.084
(the upper value is the noisy wide stencil). This finds no broad gross cost
from the two execution worlds in these fixtures. It does not causally measure
the historical continuation correction: no identical-source before/after
compiler control was run, and the earlier failed extent trial remains intact.

The null invokes the **same image path** twice, relabeling the second run
`wf-b`. It qualifies host/cadence only, not separately linked layouts. The
generic reducer's twin-build wording does not change that boundary. All nulls
remain: coarse-prefix W2 has median `b/a=1.0344` with four of five pairs higher;
tiny-histogram W1 has `1.0366`, only 0.125 microseconds, with three higher,
one equal and one lower. Wide-stencil W4 ranges from 0.95 to 1.51 despite a
0.984 median. CPU null ratios include prefix W4 at 0.804 and quadrature W4 at
1.144; tiny CPU readings are quantized to microseconds. Small wall or CPU
claims comparable to these variations are inconclusive, not rescued by a
replacement control or another run.

Native rows cover oneTBB and, for quadrature, Rayon join; the reducer's
"BEST REFERENCE" means only among supplied rows. They deliberately use the
bundle's scalar `-O3`/no-vectorization flags, while WF uses driver `-O2` with
vectorization permitted. Retained object inspection confirms packed stencil
arithmetic and packed FIR multiplication in WF against scalar native kernels.
These are scheduler/decomposition and representation context, **not evidence
of native competitiveness or closure of regular code-generation gaps**.
Strict FP preserves operation order without forbidding independent vector
lanes. Useful serial prefix/histogram also use direct one-pass algorithms;
native FIFO and sparse/pull comparisons explicitly change useful work.

Construction and execution costs are separate. The fresh revision-specific
compiler build took 46.39 s; emission of 18 modules took 0.94 s; construction
of nine native images took 3.75 s with pinned dependencies reused. Six
selected correctness stages took 1.04--1.76 s each, and the three ABI-only
Mandelbrot/records/merge-sort stages took 0.93--1.25 s each. Calibration stages
took 0.83--1.98 s each. The 20 fixed execution actions took 70.80 s across
guarded stages including hashing, setup and checks; the largest action was
12.39 s. The final six fixtures shared a 30.99 s guarded ownership window,
retaining separate action results. These are stage costs, not warm-call times.
Three live-owner refusals ran no benchmark and remain in the record.

To reproduce construction, start from a fresh checkout and target directory;
do not reuse a Cargo target shared by differently edited worktrees. The
published ABI-only patch reproduces the actual `f19d53e2` caller content.
Reuse the cached oneTBB `3046c8b0c29df995980003ea24f4d78c80ec0c8d`, Parlay
`51017699dcc421f80479cdb238d3092233ad0d26` and Rayon 1.12.0 dependencies with
the recorded flags/hashes and successful Parlay probe. If unavailable, the
bundle's `make deps` is a separate guarded dependency-construction stage.

```sh
work=$(mktemp -d /tmp/wf-6fdb.XXXXXX)
git worktree add --detach "$work/source" 6fdb6768890b91283fb041dfc7981a2cc2f3e5b8
git show 0cc0e7f4cdd9048aed124b54d9fe4d93b6873ac4 --format= -- \
  research/experiments/compute-bench > "$work/caller.patch"
git -C "$work/source" apply "$work/caller.patch"
git -C "$work/source" diff --exit-code -- compiler tests/programs/compute
export CARGO_TARGET_DIR="$work/target-6fdb" CARGO_BUILD_JOBS=2 JOBS=2
guard="$work/source/.github/run-check.pl"
WHITEFOOT_CHECK_TIMEOUT=120 perl "$guard" baseline-compiler cargo build \
  --manifest-path "$work/source/compiler/Cargo.toml" --profile gate \
  --bin whitefootc --locked --offline
wfc="$CARGO_TARGET_DIR/gate/whitefootc"
bench="$work/source/research/experiments/compute-bench"
build="$work/build"
deps=/absolute/path/to/the/recorded/pinned/deps
kernels='prefix histogram stencil bfs fir quadrature mandelbrot records merge_sort'
mkdir -p "$build"
printf '/* Generated by `make deps`: the ParlayLib probe compiled. */\n' > "$build/parlay_status.h"
set --
for k in $kernels; do set -- "$@" "$build/$k-par.ll" "$build/$k-seq.ll"; done
WHITEFOOT_CHECK_TIMEOUT=120 perl "$guard" baseline-emission make -C "$bench" -j2 \
  WFC="$wfc" BUILD="$build" DEPS="$deps" KERNELS="$kernels" WF_AB= \
  WF_PAR_CONTROL_FLAGS= WF_RUNTIME_CONTROL_FLAGS= WF_MODULE_CONTROL_FLAGS= "$@"
WHITEFOOT_CHECK_TIMEOUT=120 perl "$guard" baseline-native make -C "$bench" -j2 \
  WFC="$wfc" BUILD="$build" DEPS="$deps" KERNELS="$kernels" WF_AB= \
  WF_PAR_CONTROL_FLAGS= WF_RUNTIME_CONTROL_FLAGS= WF_MODULE_CONTROL_FLAGS= images
```

Run `WF_WORKERS=WIDTH IMAGE verify FORM WIDTH` for WF/TBB at 1/2/4 and
`wf-seq 1`; quadrature also verifies Rayon join at 1/2/4. ABI-only images verify
WF/TBB at 1/4 and `wf-seq 1`. Guard those stages separately. Hash the compiler,
images, emitted modules, runtime/source inputs and pinned libraries before and
after timing, as the retained `context` records do. No compiler build, emission
or native construction belongs inside a measured kernel call.

The retained raw headers specify the exact process order and workload, so a
fixture can be replayed without a new permanent runner. Set `data` to the
absolute retained TSV path, choose one `main-*` or `null-*` group from the
table above, and use a fresh `out`. Repeat once per declared group, retaining
all outputs; the same-image null is not `WF_AB=1` construction.

```sh
data=/absolute/path/to/compute-baseline-2026-09-21.tsv
group=null-prefix-large
out="$work/$group"
WHITEFOOT_CHECK_TIMEOUT=90 JOBS=2 perl "$guard" "$group" \
  sh -eu -s -- "$data" "$group" "$out" "$build" "$bench" <<'SH'
data=$1 group=$2 out=$3 build=$4 bench=$5
test ! -e "$out"; mkdir -p "$out"
unset WFB_BLOCKED_GRID WFB_STENCIL_GRID WFB_STENCIL_WIDTH WFB_STENCIL_HEIGHT
unset WFB_BFS_MODE WFB_BFS_GRAPH WFB_GAP_US
settings=$(awk -F '\t' -v g="$group" '$1==g && $2=="manifest" && $3~/^environment:/ {sub(/^environment: /,"",$3); print $3}' "$data")
if test "$settings" = 'default fixture'; then settings=; fi
awk -F '\t' -v g="$group" '$1==g && $2=="raw" {sub(/^[^\t]*\t[^\t]*\t/,"");print}' "$data" > "$out/recorded.tsv"
awk 'function value(k,s){s=$0;sub(".* " k "=","",s);sub(" .*","",s);return s}
 /^# driver=/ {print value("driver"),value("form"),value("width"),value("pass"),value("calls")}' \
 "$out/recorded.tsv" > "$out/order.txt"
while read -r kernel form width pass calls; do
  real_form=$form; if test "$form" = wf-b; then real_form=wf; fi
  log="$out/$form-w$width-p$pass"
  env $settings WF_WORKERS="$width" "$build/$kernel" time "$real_form" "$width" "$pass" "$calls" > "$log.out" 2> "$log.err"
  if test "$form" = wf-b; then
    awk -F '\t' -v OFS='\t' '/^# driver=/ || /^# batch / {sub(/ form=wf /," form=wf-b ");print;next} /^#/ {print;next} {$2="wf-b";print}' "$log.out" >> "$out/raw.tsv"
  else cat "$log.out" >> "$out/raw.tsv"; fi
done < "$out/order.txt"
awk -f "$bench/reduce.awk" -v passes=5 -v calls=5 "$out/raw.tsv" > "$out/table.txt"
SH
```

### Native kernel vectorization control

The bounded control, selected before measurement at `25263b24`, retains
compiler/runtime `6fdb6768`, harness
`f19d53e2`, fixed wide stencil (1,024 by 4,096, 16 steps) and FIR (524,288
outputs, 64 taps), at W1/W4. It removes the native kernel translation units'
global vectorization bans and FIR's local `FIR_TAP_ORDER` prohibition, retaining
`-O3 -fno-fast-math -ffp-contract=off -fno-lto`, native decomposition and
allocation/release boundaries. These C units also contain driver and oracle
code; the control recompiles that complete scope. WF modules, native runtime,
harness and scheduler/dependency objects remain byte-identical. Cached native
libraries retain their scalar flags, so this is a native-kernel control rather
than a rebuilt-library comparison.

Full existing correctness/oracle matrices and inspection of actual optimized
arithmetic precede timing. Each arm runs five paired passes, each with one
warm-up and five measured calls, at zero call gap. A native wall-time improvement
of at least 10% in at least four of five pairs at either width flags a material
previously hidden cost for subsequent attribution, not an automatic WF compiler
change or a scheduling diagnosis. Retiming identical WF objects in both linked
images and paired identical-image controls qualifies combined layout, wrapper
and host/cadence effects; variation comparable to the proposed benefit leaves
its attribution inconclusive. All outcomes remain reported without rerunning
to obtain a favorable result, and there is no broader sweep. If FIR remains
scalar, this trial establishes no lane-blocked FIR comparison. Construction,
correctness and measurement remain separate guarded stages capped at 90 seconds,
calibrated against the preceding baseline's largest 12.39-second action.

The [2026-09-22 evidence](../../experiments/compute-bench/native-kernel-vectorization-2026-09-22.tsv)
retains all 1,200 checked timing calls, 20 full oracle-matrix outcomes, paired
process medians, unchanged-input hashes, construction commands and optimized
callback excerpts. All oracle matrices passed. Native stencil now uses packed
`fadd.2d`/`fmul.2d`; native FIR computes packed products and adds their extracted
lanes in the original scalar tap order. Neither callback introduces contraction
or a horizontal sum. FIR therefore exercises ordinary optimized tap-loop code,
but the historical lane-blocked source remains unmeasured here. Construction
took 1.15 seconds, verification 2.07/1.65 seconds, and the four timing actions
6.54–16.01 seconds; each selected action ran once, with no timeout or rerun.

| Native cell | Stencil paired B/A wall | Pairs at least 10% faster | FIR paired B/A wall | Pairs at least 10% faster |
| --- | ---: | ---: | ---: | ---: |
| Serial W1 | 0.779 | 5/5 | 0.888 | 4/5 |
| TBB W1 | 0.785 | 5/5 | 0.885 | 5/5 |
| TBB W4 | 0.763 | 4/5 | 0.897 | 4/5 |

Both fixtures meet the prospective hidden-cost criterion. The identical WF
objects retimed in A/B have paired wall medians of 1.010/1.020 for stencil
W1/W4 and 1.007/1.004 for FIR; no such pair moves by 10%. The identical-image
stencil TBB W4 control is noisy, spanning 0.868–1.195, and stencil WF W1 spans
0.959–1.147. FIR TBB W4's null CPU ratio is 0.915 despite a wall ratio of 1.000.
These controls support the substantial native improvement, especially its
consistent W1 result, without assigning an exact causal percentage to
vectorization or treating small CPU differences as settled. The FIR control
also bundles removal of its local pragma with the global flag change.

In the optimized-native image, stencil's WF W1 median is 29.09 ms versus
37.20 ms for native serial, while WF W4 is 16.83 ms versus 14.04 ms for TBB
(paired WF/TBB median 1.188). FIR is close under this contract: WF/native serial
W1 are 13.21/13.26 ms and WF/TBB W4 are 3.58/3.50 ms. The remaining stencil
question is why work and scaling differ across widths, including allocation,
layout, serial phases and runtime costs; these whole-call results select no
scheduler or global code-generation repair. They qualify the frozen `6fdb6768`
regular-kernel baseline, not later-main performance or the remaining compute
families.

For reproduction, the TSV's `context/command-source` rows retain the exact
one-shot scratch recipe; its build action changes only the two C translation
units and links the recorded frozen objects in their original order. Extract
those rows by removing the first two tab-separated fields, inspect the pinned
artifact paths, and invoke its stages through the shared guard as recorded in
`context/manifest`. Empty payloads denote blank lines. Run full verification
and inspect the resulting arithmetic before recording the qualification marker
and starting timing. Each of the eight `*/raw` groups extracts byte-for-byte
to the original stream and feeds the existing `reduce.awk` with five passes and
five calls. The paired summary first takes each process's five-call median,
then forms B/A within the same form, width and pass; its reported ratio is the
median of those five ratios, not a ratio of the arm medians. The retained recipe
is execution evidence, not a new maintained benchmark runner or a default flag
change.

### Zero-budget stencil dispatch control

Static inspection of the frozen wide stencil's parallel world identifies
65,504 row executions per call, each querying the inner split budget with span
1,022 and weight 17.
The retained runtime's 150,000 work floor returns zero affordable chunks; the
harness clears the diagnostic floor override. Nevertheless, each row resolves
thread-local state in the runtime query and enters a recursive splitter whose
frozen machine-code prologue reserves 240 bytes before testing that budget.
The pixel loop already has the same eight-output packed arithmetic shape as
the optimized native callback. This selects an investigation of unused inner
dispatch work, without attributing the whole-call CPU increase to scheduling.

The needed-capture candidate CLI has SHA-256
`480b8b56c8dca5b4469307026a10643d142ee46fff5b937634e3618699e17adf`.
Emitting the unchanged stencil source, SHA-256
`69a94c11eac439e44bd50b81dabf2c1d5da4398878836bfad478a8e46004e352`,
took 0.10 seconds under the shared guard. Pixel, initialization and row captures
fall from 6/6/9 to 4/2/3, and their actual frame sizes from 120/88/112 to
104/56/64 bytes. All eight parallel/sequential chunk bodies retain identical
non-phi operations and the emitted work computations are unchanged. All old
frames already fit the 256-byte limit. Candidate optimized code and stencil
performance remain unqualified at this selection point.

The selected scratch control starts from that ordinary `--par` emission.
Arm A retains it; arm B changes only the pixel-loop site in `stencil_row` to
call its existing chunk directly, removing the inner budget query, its span
computations and recursive splitter entry together. It measures that combined
cost, including any resulting optimization, without separating query cost from
splitter cost. Outer split sites, estimates, allocation, arithmetic, cleanup,
the W1 clone, driver, runtime and dependency objects remain fixed. This is a
diagnostic LLVM variant, not a production lowering rule. Direct use of the
existing chunk preserves the subrange loop and the two-world boundary; a
general zero-budget path would need its own evidence if this control earns it.

Before timing, require the exact IR diff and anchor checks, full existing
stencil oracle matrices for both arms at W1/W4, and inspection of optimized
pixel work and the W1 clone. Then use the existing harness on only
1,024 by 4,096 by 16, five paired W1/W4 passes, one warm-up and five warm calls
per process, zero call gap, with adjacent arm order and width order alternating.
Run matched identical-image A/A pairs and retain all outcomes. A useful lead
requires W4 B/A wall time at most 0.95 in at least four of five pairs and a
median fractional benefit greater than the largest absolute paired null drift
at W4. Report W1, process CPU, first calls and steals as well. A failed criterion
or null control ends this bounded trial as inconclusive, without rerunning to
obtain a favorable result. Construction, full oracles, inspection and each
timing action are separate guarded stages capped at 30 seconds. No core
pinning, runtime constant change, phase instrumentation or wider sweep is part
of this control, and no research artifact enters correctness CI.

The [2026-09-22 record](../../experiments/compute-bench/stencil-zero-budget-2026-09-22.tsv)
retains the exact IR diff, frozen input/image hashes, one-shot recipes and all
outcomes. Construction took 1.35 seconds; all four existing oracle matrices
passed, comparing 3,387,721 values each, in 1.93 seconds. Optimized-code
inspection took 0.89 seconds. The W1 optimized IR matches after expanding
equivalent loop metadata, and its object instructions match after rebasing
addresses. Parallel pixel arithmetic retains its eight-output SIMD shape and
floating-point grouping. B additionally inlines that work into row chunks and
outer split leaves and moves chunk alias checks outside the row loop. Those
effects belong to the combined control; they do not isolate query overhead.

The original null action completed once in 5.65 seconds, retaining all 120
checked calls, but its order generator was defective: `(pass + index) % 2`
combined with reversed widths always ran A/B at W1 and B/A at W4. This violates
the selected within-width alternation, so the session is protocol-invalid and
has no main comparison. Its descriptive paired W4 wall ratios span
0.890044–1.047734; the median is 0.986233 and largest absolute drift 0.109956.
W1 spans 0.899249–1.034519 with median 0.949014. These are preserved outcomes,
not a qualified null or a performance result for B. No image, compiler, runtime
or lowering change follows from this session.

#### Corrected-order session

A separately selected session repairs only that order-generation defect; it
does not rerun or overwrite the original session. It reuses the qualified A/B
images and oracle evidence with hash checks, without rebuilding or repeating
unchanged correctness work. The fixture, widths, arms, five paired passes,
one warm-up plus five warm calls, zero gap, combined attribution scope and
numerical criterion above remain unchanged. Arm order is A/B on even passes
and B/A on odd passes independently of width order, which still reverses each
pass. The retained dry run lists all 20 invocations and mechanically asserts
that each pass contains each arm once at each width, and that each width's arm
order alternates. The new recipe has SHA-256
`05a4cd3ea80ad3fb75ad1aab3f39364c157add395b70230ba59aa29b868d2906`.

After this new criterion is committed and published, run exactly one new
identical-image null action and one main action, each under the shared guard
with a 30-second cap. Preserve all new raw data and use only the new null in
the unchanged numerical criterion; exclude the protocol-invalid session from
that comparison. A failed check or inconclusive criterion ends this corrected
session without retry. No corrected-session timing has occurred at selection,
and no production improvement is selected before its result.

The corrected criterion was published at `45961216` before either action.
Both actions completed once, in 4.67 seconds for the new null and 4.55 seconds
for main; all 240 checked calls, process orders and unchanged-input checks
passed. W4's five paired main wall ratios are 0.773692, 0.856267, 0.859311,
0.820277 and 0.817753. All five clear 0.95, and their median benefit of
17.9723 percent exceeds the new null's largest absolute drift of 4.2470 percent
(null range 0.969571–1.042470, median 0.992097). This meets the selected
useful-lead criterion. The original invalid null remains excluded.

| Width | Main A/B wall ms | Paired wall B/A | Main A/B CPU ms | Paired CPU B/A |
| --- | ---: | ---: | ---: | ---: |
| W1 | 28.887 / 28.880 | 1.010216 | 28.876 / 28.870 | 1.010253 |
| W4 | 16.519 / 13.559 | 0.820277 | 59.823 / 49.964 | 0.826945 |

Arm values are medians of the five process medians; paired ratios are medians
of the five within-pass B/A ratios. W1 main pairs span 0.933252–1.040836,
wider than their 0.976660–1.020736 null despite identical normalized W1 code.
This limits small claims and exact causal percentages. First-call A/B wall
medians are 33.853/33.996 ms at W1 and 21.313/18.795 ms at W4; corresponding
CPU medians are 33.820/33.985 and 67.865/58.255 ms. Median warm successful
steals are 0/0 at W1 and 166/164 at W4, not measurements of task count or
worker idleness. The evidence retains every first call, warm call and counter.

The result supports a general investigation of work that receives a zero
split budget. It establishes a combined dispatch/inlining/alias-check effect
on this fixture, not standalone runtime-query overhead, a new grain policy,
or later-main/native competitiveness. A production mechanism and its separate
validation remain deferred in [the maintained TODO](../../../docs/todo.md).
The scratch change is structurally suitable for attribution because it calls
the existing subrange chunk and preserves arithmetic, cleanup and the
two-world boundary; copying the runtime's current floor into production
lowering would require grounds this experiment does not supply. No compiler,
runtime, specification or correctness-CI change follows from this result.

## Needed loop captures

The sparse-frontier source at `139fc2d1d74480d58ab878c0eb67bca12b5144f1`,
SHA-256 `d10f0047164ca614968d55e50549d7efd1a768a5784c901d36dd82f9467f63f2`,
exposes a task-width obstruction independent of its partition proof. The
emission ledger before needed-capture selection reports 29 captured bindings
for `outbox_level` loop `6.0.27.0` and a 352-byte frame against the 256-byte lane
limit. That lowering captures every surrounding binding, including inputs used
only by an earlier source phase or the function's tail. These bindings reach no
runtime operation in the receiver but are forwarded through every loop block
parameter.

The selected policy first checks the original conservative frame against the
unchanged 256-byte limit. If it fits, retain its complete capture interface,
reconstruction, forwarding, readonly-formal markers and preorder helper
reservation. If it is too wide, build the ordinary chunk once and trace runtime
need backward through its block parameters to try to rescue it. All ordinary instruction operands,
helper arguments, cleanup subjects and returned values seed need. Only the
compiler-generated entry operations that reconstruct captured Box slots are
removable computations; their inputs become needed when the reconstructed
value is needed. Nested candidates complete their own fit-or-rescue choice
before the enclosing chunk reads the nested call. Keep value identities, source operations, ownership,
cleanup order, range proof, source Box representation, and the chunk's loop.
For originally wide frames, capture loads and projections in the parent wait
until the reduced frame fits. This bounded policy supersedes the earlier
all-loop pruning experiment below: the repeated records W4 performance signal
remains unresolved while the candidate changes an already-fitting task's
transport and placement. Removing those captures unlocks no new loop there;
their removal has not been established as the cause of that signal.
The sparse receiver supplies a concrete capability benefit for rescue.
The original refusal path discarded tentative nested synthesis and lowered the
body again. The PR #78 review identified that nested refusals repeat this at
each depth, giving exponential construction despite a small final module.

The selected repair reuses the completed candidate on refusal: splice its
ordinary CFG into the parent and connect its result to the continuation, with
no new runtime call. Map original binding roots back to the parent's values
and remove the generated capture reconstruction, so fallback retains the
original Box slots and cleanup authority. Remap definitions, operands, drops,
source-call result identities, call-site locations and counted-range metadata;
the parent's ordinary overlap collection then resolves the imported call
sites. Original readonly-formal markers stay on the parent's existing formals,
never on new block values. Nested functions and ledger rows survive once.
An originally wide frame reserves its enclosing helper pair only after the
reduced frame fits: its nested helpers consequently precede that rescued
parent in ordinal and emission order. Known-fit frames retain their existing
preorder reservations and names. Retaining preorder names for an undecided
wide candidate would require a second function-ordinal relocation pass with
no semantic consumer. The unchanged frame bound, permissions and ordinary loop operations
continue to select the emitted shape; no timeout or work budget is introduced.

Before implementation, the deterministic criterion is one candidate-body
construction per permitted loop for small increasing depths of genuinely
oversized nested frames. A test-only counter records construction before any
discard, because final function counts cannot expose the old recurrence.
Also retain a fitting inner split beneath a refused outer loop, and validate
source-call/overlap metadata, counted extents, cleanup and native results.
Splicing can revisit a retained IR node once per refusing ancestor, so its
cost is polynomial in the lowered program; source-body construction is not
repeated. Construction and execution remain separately measured.

Before qualification of this narrowed policy, require exactly-fit and
one-scalar-over controls at the same bound, and byte-identical emitted LLVM
against exact main `9450decc6df47ef3ec34e452f98ff7a6282ee72d` under identical
flags for formal kernels whose complete frames all fit, beginning with
records. Stencil and other nested cases need their own correspondence result.
The unchanged sparse receiver must still emit below the bound and pass its
native oracle; the fixed hosted comparison must no longer reproduce the known
adverse records signal. These criteria were fixed before qualification; the
results below distinguish payload correspondence from measured execution.
No local timing run, new threshold, padding or type-specific exception is
selected. Broader pruning of fitting tasks is deferred in the maintained TODO
until a demonstrated benefit and qualified comparison justify that wider
transport change.

An ordinary call to a refused chunk would avoid repeated construction but add
a runtime ABI and outlining cost to fallback. A source-path refusal cache
would need to prove that capture representation and original-formal facts are
invariant between parent and chunk contexts. Extracting every accepted loop
from its parent would replace more of the working outlining path. Reusing the
existing candidate needs only the local CFG transfer and its metadata, so
those broader alternatives are not selected.

The discriminating criterion is that the unchanged sparse receiver actually
emits its split with a frame below the existing limit, while ordinary native
loop tests retain their independently checked results and cleanup. Formal
structural cases must retain call-, cleanup-, return- and nested-helper-only
uses, remove forwarding-only captures, and preserve ordinary lowering when a
still-oversized candidate declines. These are lowering observations, not a
performance verdict. The construction-count regression and ordinary fallback
checks must qualify reuse as well as the reduced capture interface.

Enlarging the lane limit preserves the accidental dependency on lexical scope.
Writer phase helpers would work around it in every consumer. A source-use or
effect walk would duplicate semantic interpretation and risk missing generated
cleanup; a general projection or interprocedural optimizer is unnecessary to
remove this obstruction. The existing exhaustive IR operand walk therefore
becomes the shared owner for capture selection, backend storage and emission.
Read-only formal markers and Box payload reconstruction remain associated with
their original value identities when a capture survives.

Preserving value IDs leaves type metadata for removed aggregate formals and
block parameters. Storage planning previously assigned backing to every typed
aggregate ID, including those with no remaining definition. The selected
companion change seeds storage candidates from the existing flow graph's
entry parameters, block parameters and instruction results. It keeps storage
for every remaining ordinary definition, including unused ones. A tail-only
fixed array in the existing captured-fold native test checks that a rescued
frame leaves no phantom aggregate chunk slots. The aligned Box test retains
its fitting frame's original unused payload transport while checking that
used payload and measure paths retain their cleanup behavior: its changed
capture-count assertion narrows an optimization observation, not the no-read
or single-owner cleanup requirement. Type tombstones and a
whole-chunk renumberer would introduce another representation or rewrite
without serving this consumer.

The earlier all-loop pruning candidate emitted the same source with
`--par --par-scalar-leaf-limit off --emit-llvm --par-ledger` and met the frame
criterion. These dated values do not describe the new rescue-only policy:

| `outbox_level` phase | Before selection | After selection | Dated work price |
| --- | --- | --- | --- |
| Routing, `6.0.16.0` | 21 captures, 256-byte frame | 10 captures, 152-byte frame; split emitted | 363 per iteration, static |
| Receiver, `6.0.27.0` | 29 captures, 352-byte estimate; split declined | 11 captures, 168-byte frame; `+wrap` count reduction emitted | 2,580 per iteration, static |

The emitted frame sizes are the actual `wf__par_acquire_lane` arguments; the
receiver's earlier 352 bytes are its conservative lowering refusal estimate.
The strengthened captured-fold native case retains the removed array's type
metadata, allocates no aggregate chunk slot, and preserves the ordinary
lowering's output at one and four workers and under controlled worker grants.
Its harness construction took 79.33 seconds and its focused execution 2.70
seconds; isolated CLI construction took 45.35 seconds, and source checking plus
LLVM emission took 0.72 seconds. These are construction and qualification
costs, not a program performance comparison. Sparse-frontier native behavior,
worker participation and performance remain separate qualification work.

### Rescue-only payload qualification (2026-09-22)

At `79775234`, the controlled regression restores only the old refusal's
discard-and-retry behavior: replacing CFG reuse with `return Ok(false)`.
The test-only construction counter remains present. Its library-test image
took 80.85 seconds to construct; the sole growth case then failed as intended
at depth two with three constructions instead of two, after depth one passed.
The invocation took 0.72 seconds and stopped before the larger depths. Exact
committed source bytes were restored before constructing the corrected test
image in 81.32 seconds. Its 78 focused tests passed in 14.39 seconds: 32
lowering, 24 storage, 12 loop-split, six tail-call, one cleanup and three
read-only-reference pricing cases. These include depths 1/2/4/6, the fitting
256-byte versus rescued 48-byte frame boundary, and native scalar/Unit fallback
with retained inner splitting and sibling-call metadata. This isolates the
repeated-construction mechanism; it is not a whole-compiler complexity proof
or an execution-performance benchmark. The grouped scratch controller's final
whole-worktree-clean assertion saw the concurrent research-document update and
returned one after the successful tests; all 313 compiler/specification input
hashes and exact restored source were independently verified, without reruns.

The ordinary gate-profile CLI took 43.90 seconds to construct. Five formal
LLVM emissions and comparison took 0.98 seconds on the same Apple M1 Pro
host as the saved `9450decc` baseline: all five raw modules, five bound modules
and five complete host-adapted modules are byte-identical. The 19 formal
fixture/instrument inputs match. No native timing comparison was run locally.

The unchanged sparse source emitted with ordinary `--par --emit-llvm
--par-ledger` in 0.16 seconds. Routing retains 21 captures and its original
256-byte frame; the receiver is rescued from the original 352-byte estimate
to 11 captures and an actual 168-byte lane request. Their static work prices
remain 363 and 2,580. One plain native image took 0.67 seconds to construct;
its W1 and W4 oracle executions took 1.02 and 0.47 seconds, respectively.
Each passed 96 configurations and 70,021,040 distance/input comparisons.
The guarded complete qualification took 2.58 seconds, with all 31 source,
adapter, oracle and runtime inputs checked unchanged before and after.
Only saved scratch checkout paths were adapted to the current source tree.
The image contains no observer symbols. Its module SHA-256 is
`8bb906e243bfaed00117d66cdeaacbb67abcdfbf253a41ffea4ae87cc3edff67` and
native image SHA-256 is
`57c07839dee3872493368d7ca4322b374797122cec2641763541b3f65343be2e`.
These checks qualify current native results, not helper participation or FIFO
performance; the earlier failed FIFO null session remains closed.

The automatic hosted comparisons at implementation `8fe62a10`
([run 35816171952](https://github.com/mbbill/Whitefoot/actions/runs/35816171952))
and its main integration `005000c`
([run 35816497671](https://github.com/mbbill/Whitefoot/actions/runs/35816497671))
both use baseline `e6349b80`. Their synthetic candidates `b48bd277` and
`4ccef357` have the same tree `c049c7af8afbf1d097bbc6d41311d26401c1dc3d`,
also the tree of `005000c`. All five formal kernels' complete LLVM modules,
kernel objects and linked executables are byte-identical between baseline
and candidate and across the two sessions. Ordinary runtime objects and all
33 common recorded source/runtime inputs also match. The only differing
image-directory file is the native configuration's checkout path. This is
direct evidence that the bounded policy preserves these fitting programs'
payloads; it does not identify the cause of the earlier different-payload
records observation.

Both hosts report AMD EPYC 7763, four logical CPUs on two cores, Linux
6.17.0-1022-azure, clang 18.1.3 and rustc 1.98.1, using the ordinary `-O2`
runtime. Each session retains five paired passes and its null control:

| Hosted run | Records W1/W2/W4 baseline/candidate wall | Records W4 CPU | Null suspects | Comparison suspects |
| --- | --- | --- | --- | --- |
| `35816171952` | 1.003118 / 0.992181 / 0.999169 | 0.984852 | Stencil W4: 0.952463, four of five adverse | 0 |
| `35816497671` | 0.997282 / 0.982966 / 0.982410 | 0.965794 | 0; stencil W4 is 0.973062 | 0 |

Both comparisons pass the existing rule. Preserve the first session's null
suspect: these are two automatic revision qualifications, not repeated samples
selected for a favorable result, and neither supplies an optimization speedup
between identical executables. Artifacts `10732050779` and `10731244189`
retain the complete payloads, identities and samples; their downloaded ZIP
SHA-256 values are respectively
`b3f478f693ed6b0e9b282750fb89fb22c9b7d2499a7e6e6c2f460e3ee0af0232` and
`096a6f3b1d8871d45130984af294add11d4c220c218301452fe54d61ca9edaee`.
The following `79775234` change only repairs test access to the public emitter;
it leaves this production implementation unchanged and requires its own
correctness qualification.

The automatic test-repair qualification
([run 35817040045](https://github.com/mbbill/Whitefoot/actions/runs/35817040045))
also compares with `e6349b80`, using synthetic `9aefcf3c`. Records W1/W2/W4
wall ratios are 1.000321/1.015097/1.019881, with W4 CPU 1.035708 and zero
comparison suspects. Its null retains another stencil W4 suspect: wall
0.918926, four of five adverse, CPU 0.904133. This automatic run also passes
the rule; its adverse null result is retained, not used to select another
session or assert a speedup. Its synthetic tree equals `79775234`, and all
five complete LLVM modules, kernel objects and linked executables remain
byte-identical to its baseline and the preceding sessions. Its 33 common
recorded inputs and runtime objects also match, on the same reported host
class. Artifact `10731679336` retains the payloads and samples; its ZIP SHA-256
is `6339dbf0d24cc7b02a53742c27d67504e4588ab30c04cd77ac821facce9f028f`.

### Records W4 hosted comparison remains unresolved

The main-integration revision repeats this signal in
[run 35814248071, job 107032257821](https://github.com/mbbill/Whitefoot/actions/runs/35814248071/job/107032257821):
synthetic candidate `cd9a6329` has tree `75cbce23`, equal to work head
`3eb29b09`, against baseline `9450decc`. On AMD 7763, four logical CPUs/two
cores, clang 18.1.3, the null has zero suspects and records W4 reports
baseline/candidate wall `0.926447`, five of five adverse, and CPU `0.904075`.
The retained artifact is `/private/tmp/whitefoot-pr78-compute-35814248071`.
This is evidence motivating the bounded rescue policy above, not a causal
verdict. The narrowed policy's qualification preserves the fitting payloads;
the cause of this earlier all-loop pruning result remains unresolved.

The formal comparison already reported a `records` W4 suspect at `30198a19`.
[Run 35706215154](https://github.com/mbbill/Whitefoot/actions/runs/35706215154)
retains it at `53c68c29905cb9877e32118ac1be79d972df6c31`, whose compiler,
tests and specification are unchanged from that earlier revision. Its
[artifact 10684228124](https://github.com/mbbill/Whitefoot/actions/runs/35706215154/artifacts/10684228124)
identifies the tested merge as `825714ec66e6839201623ae4eca7efd68c789866`
and baseline as `f3cf41d42cf6324a83c1de0b28e6c0d4e6ff9da8`. The host is
x86-64 AMD EPYC 9V74, four logical CPUs on two cores, using rustc 1.98.1
and Ubuntu clang 18.1.3, ordinary `--par` and `-O2`, without alignment
overrides. Baseline/candidate wall ratios are 1.002281/0.995866/0.898002
at W1/W2/W4; W4 has four adverse pairs out of five and CPU ratio 0.908317.
The identical-image control has a FIR W4 suspect at 0.956881, while records
W4 is 0.990877 and not suspect. Passing the unchanged two-width rule does
not resolve the repeated records observation.

Static inspection used those hosted LLVM modules and ELF objects, without
new emission, construction or timing. The records source, host adapter,
oracle and runtime sources are unchanged from frozen `6fdb6768` through
baseline `f3cf41d4` and this candidate. All 26 function definitions in the
retained frozen `records-par.ll.raw` match the hosted baseline after only
renaming `main` to `wf_fixture_main` and `wf__main_body` to `wf_fixture_body`;
the latter adds two host adapter functions. This establishes records
correspondence, not general compiler or cross-platform object equality.

| Hosted records property | Baseline | Candidate |
| --- | ---: | ---: |
| Captures / actual task frame | 6 / 104 bytes | 4 / 88 bytes |
| Iteration work price | 814 | 814 |
| Optimized splitter / local stack bytes | 679 / 56 | 601 / 56 |
| Optimized parallel chunk bytes | 440 | 440 |
| Linked parallel chunk address | `0x2f20` | `0x2ec0` |

Pruning removes unused `end` and `count` captures. The timed caller retains
pool dispatch, one allocation, one zero-fill, the budget query and split call;
its outgoing stack arguments fall from six to four. Granted and refused
splitter paths retain their predicates and call structure while transporting
fewer values. Both frames fit the same fixed 256-byte runtime slots. All 12
runtime objects, `runner.o` and `records_oracle.o` are byte-identical between
arms. The optimized validator and record-summary functions are byte-identical.
Each parallel/sequential chunk differs at exactly one byte, offset `0x19`:
the output-pointer stack displacement changes from `0x58` to `0x48`.
The remaining instruction bytes and loop alias annotations are unchanged.
Linked placement changes, including runtime addresses, are observed but not
attributed. No added record-processing work or regression fix is established.

The retained ZIP is `/private/tmp/whitefoot-records-35706215154.zip`, SHA-256
`141b4354cf37f754070e94b1fb305ad9ad852f33b5ebb81683b6e8631b9e68ac`;
its extracted directory has the same stem. Within `performance-results`,
`identity.txt`, `comparison/manifest.txt`, `comparison/paired.tsv` and
`null/paired.tsv` retain identity, executable hashes and ratios. The
baseline/candidate `records.o`
hashes are `bd24e30f2a8c2cfe3da120354967b93dc317ee70d21635dc209032d591c6ef02`
and `8e6c5d3657dd23eb2ff5f28777410574801f8269f6653258ba30457859ee44cf`.
From the extracted directory, LLVM 22.1.8 inspection tools reconstruct the
comparison without executing either image:

```sh
diff -u performance-baseline/records.ll performance-candidate/records.ll
for arm in baseline candidate; do
    llvm-nm --print-size --numeric-sort "performance-$arm/records.o"
    llvm-nm --print-size --numeric-sort "performance-$arm/records"
    llvm-objdump -dr --no-show-raw-insn "performance-$arm/records.o"
    llvm-objcopy --dump-section ".text=$arm.text" \
        "performance-$arm/records.o" "$arm.inspect.o"
done
dd if=baseline.text of=baseline.chunk bs=1 skip=2320 count=440
dd if=candidate.text of=candidate.chunk bs=1 skip=2224 count=440
cmp -l baseline.chunk candidate.chunk
```

The byte comparison reports only `26 130 110` (one-based offset, octal
values). Existing raw data has wall time, CPU and result counts, without
scheduling counters. Identical hot work cannot exclude ABI, scheduling or
placement effects; ARM results or emulation cannot clear this Linux signal.
A possible next discriminator, **not selected or run**, is one bounded
records-only W4 paired/null block using the retained Linux images and existing
`WF_SCHED_REPORT=2` counters. A reproducible participation/steal change would
support investigating scheduling; unchanged counters would leave ABI and
placement unresolved, not clear the suspect. Uninformative results would end
that block without a sweep. Attribution and its validation remain deferred in
the [formal-compute TODO](../../../docs/todo.md), because a mechanism change
needs evidence distinguishing these possible causes. No threshold, runtime,
compiler, specification or correctness-CI change follows from this inspection.
