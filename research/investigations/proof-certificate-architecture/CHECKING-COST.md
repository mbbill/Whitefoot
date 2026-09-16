# Source certificate checking cost

## Question and protected behavior

This investigation addresses the large `proof_use` cost in
[TODO](../../../docs/todo.md). The historical 64/128/256-entry timings have no
pinned input bundle; they are a reason to measure, not a current baseline.
The acceptance authority remains [PRF-1 and ENT-6](../../../spec/kernel-spec.md).
The [source-checking assessment](SOURCE-CHECKING.md) separates the chosen
certificate rules from unsupported claims about their cost.

Keep the 4096-entry and affine-formation capacities, complete target AUTO,
independent premise checks against one entering context, duplicate detection,
checked source-order arithmetic, residual DIRECT, diagnostics and erasure.
No time limit, cumulative budget, changed verdict or smaller automatic family
may select a faster result. This work first attributes cost; it does not
presuppose an algorithm change or a language amendment.

## Discriminating experiment

Record the baseline revision, Rust/host versions, build profile, exact source
generator and every raw sample. Use the ordinary compiler through LLVM
emission, without host linking, with the optimized `gate` profile's debug
assertions and overflow checks. Report elapsed time, not runtime performance.

Separate certificate length from the size of the entering proof context:

1. Fixed context: three independent unsigned inequalities, with distinct
   weakened relation-form uses cycling over them. The target is exactly the
   written sum. Vary the number of uses through the 4096-entry ceiling without
   growing the target or the entering variable set.
2. Growing context: N independent unsigned inequalities and an N-premise sum
   target. Keep each source within the separate affine-formation ceiling.
3. Matched context control: the same N inequalities, but only the first three
   occur in the target and its three written uses. This distinguishes context
   construction/query costs from checking N certificate entries.

Start with 16/32/64/128/256 uses or context pairs, then extend affordable runs.
Use three repetitions for exploratory baseline curves. A timed invocation
runs to completion; a deliberately unstarted large cell is recorded as
unmeasured, never as a source rejection. Check successful acceptance and
include invalid-premise and duplicate-use controls before trusting a timing.

Sample the slow ordinary compiler process with native profiling to distinguish
frontend formation, target AUTO, premise admission and sum/residual work.
If those stacks do not isolate the stage, use temporary stage instrumentation
and retain its exact patch with the measurement. Do not ship a general tracing
framework for this experiment.

An implementation candidate is justified only when the profile and matched
controls identify avoidable repeated work under the unchanged rules. Before
comparing it, state its predicted affected cells and protected controls here.
For selection, alternate baseline/candidate order for at least five paired
runs: require at least a 2x median improvement at a reproduced costly cell,
no material regression in the fixed-context and small-certificate controls
(greater than 10% and 1 ms), and unchanged ordinary proof results. Failure to
meet these criteria leaves an attributed cost, not a selected optimization.

## Artifact boundary

The reproducible native Rust harness and raw measurements belong under
`research/experiments/proof-use-cost/`, linked from this investigation and
wired to the research gate for small correctness cases. They serve compiler
checking-cost experiments, not source acceptance or a timing gate. Supersede
the harness when its compiler path is replaced; retain dated results only
while they support a claim. This document owns the experiment and its eventual
conclusions, and is merged or removed if a later study supersedes them.

## Baseline attribution, 2026-09-14

Compiler source is main `2b346cf6`; the unchanged compiler and committed
generator at `50ddd638` produced the [raw baseline](../../experiments/proof-use-cost/baseline-2026-09-14.tsv).
Host: Apple M1 Pro, 8 CPUs, 32 GiB, macOS 26.6.2 (25G83), aarch64 Rust 1.98.1
(48a229cea, LLVM 22.1.8), Cargo `gate` profile. These are generated proof
scaling probes, not representative-program or runtime-performance claims.
The first fixed-16 invocation includes executable first-launch overhead;
it is retained rather than removed. Warm both binaries before paired selection.

| Size | Fixed context, median | Growing context and uses, median | Growing context, three uses, median |
|---|---:|---:|---:|
| 16 | 18.824 ms | 29.948 ms | 18.414 ms |
| 32 | 14.341 ms | 164.483 ms | 40.780 ms |
| 64 | 16.492 ms | 1691.041 ms | 166.322 ms |

Every cell has three successful compiler invocations. This does not reproduce
the old unpinned source; it establishes a current input with rapidly growing
cost and shows that certificate length alone does not explain it.

The [temporary instrumentation patch](../../experiments/proof-use-cost/stage-timing.patch)
records cumulative microseconds inside one ordinary proof statement. On the
128-pair / 128-use source its raw marks were:

```text
formed       250
auto     9257076
sum      9257320
premises 21215193
residual 21323159
```

Thus target AUTO took 9.257 s, premise admission 11.958 s, written accumulation
0.244 ms, and residual checking 107.966 ms. These instrumented numbers are for
attribution, not candidate selection. Formation here is proof-flow image
formation; it does not individually time the preceding parser or structural
checker. Full LLVM-emission runs of the same 128 fixture were about 22 s.
The baseline 256 cell has not been started; no verdict is inferred for it.

Native `sample PID 3 1 -mayDie -file OUTPUT` captures identify both costs:
sampling from process launch put 2236 of 2444 driver samples under
`affine_candidate_residual_proof`, with repeated interval-map construction;
a later sample put all 2456 driver samples inside `source_proof_premise_results`,
under the ordinary `prove` call and L0 closure. These are phase-local samples,
not percentages of total compilation time. They agree with the stage marks
and the code: each affine residual rebuilds the same endpoint lookup, and each
written relation source closes the same entering state again.

## Candidate and prediction

The candidate reuses immutable query preparation, not successful proof answers:
retain the entering L0 closure across the relation-premise loop, and retain
each atom's closed/type interval and diagnostic endpoint within one affine
candidate traversal. New term or goal information invalidates a reused L0
view. No cache crosses a source statement or fact-state change, no premise is
published, and candidate order, arithmetic and selected proof parents stay
unchanged. The [query-preparation decision](../../../design/compiler/proof-query-context.md)
records this scope and its refused alternatives.

The alternative of materializing a live fact snapshot is not selected: it
adds snapshot events and independent fact support when only an ephemeral
query view is needed. A cross-flow cache would require kill/join invalidation
across the walker and is not needed to remove the measured repetition.
Pruning AUTO or accepting redundant blocks would change the language and is
outside this implementation experiment.

Prediction, before candidate measurement: closure reuse reduces the premise
stage but not target AUTO; endpoint reuse reduces target AUTO but not repeated
closure. Both should improve growing-64/128 while leaving fixed-context and
small cases within the protected criterion. Compare baseline against closure
reuse alone, then that build against the combined implementation to isolate
the endpoint change with closure reuse held constant. Compare the combination
against baseline for selection; this does not claim the two speedups multiply
independently. Keep prefix and histogram as real-program
controls using their ordinary current source and unchanged LLVM emission.

## Paired selection, 2026-09-14

The [combined comparison](../../experiments/proof-use-cost/paired-combined-2026-09-14.tsv)
uses baseline `50ddd638` and candidate `8881833d`, with the same host, profile
and sources as above. Both binaries were warmed once, order alternated within
each fixture, and all five pairs ran to completion. No compiler build or
profiler ran alongside the comparisons. Times include compiler startup and
LLVM emission, not source generation, host linking or program execution.

| Fixture | Baseline median | Candidate median | Speedup |
|---|---:|---:|---:|
| Fixed context, 16 uses | 12.966 ms | 12.760 ms | 1.02x |
| Growing context, 16 uses | 28.696 ms | 17.282 ms | 1.66x |
| 16-pair context, three uses | 17.640 ms | 15.794 ms | 1.12x |
| Fixed context, 64 uses | 15.749 ms | 15.208 ms | 1.04x |
| Growing context, 64 uses | 1680.280 ms | 170.555 ms | 9.85x |
| 64-pair context, three uses | 163.798 ms | 95.165 ms | 1.72x |
| Fixed context, 128 uses | 19.475 ms | 18.111 ms | 1.08x |
| Growing context, 128 uses | 21480.277 ms | 1148.945 ms | 18.70x |
| 128-pair context, three uses | 1005.767 ms | 548.065 ms | 1.84x |
| Fixed context, 4096 uses | 323.225 ms | 286.128 ms | 1.13x |
| Prefix program | 175.050 ms | 169.701 ms | 1.03x |
| Histogram program | 186.319 ms | 174.980 ms | 1.06x |

The combined implementation meets the recorded 2x criterion at both costly
cells, with no median regression in the protected controls. All 120 timed
invocations accepted. The growing-128 ranges were 21.452–21.502 s before and
1.142–1.166 s after. The full 4096-use case is accepted by both binaries; its
small entering context is essential to interpreting that result. Neither a
universal 4096-use time nor linear total checking cost follows from it.

The first isolation comparison,
[baseline versus closure reuse](../../experiments/proof-use-cost/paired-closure-2026-09-14.tsv),
holds endpoint preparation unchanged at `d465986c`. Growing-64 falls from
1672.267 ms to 875.405 ms (1.91x); fixed-64 and the three-use context control
also do not regress. Closure reuse alone does not meet the 2x selection
criterion: the separately attributed target-AUTO cost remains.

The second isolation,
[closure reuse versus the combined implementation](../../experiments/proof-use-cost/paired-endpoints-2026-09-14.tsv),
holds closure reuse constant and changes only endpoint preparation. Growing-128
falls from 9688.400 ms to 1159.643 ms (8.35x), with fixed-128 essentially
unchanged (19.273 versus 19.204 ms) and the three-use context control falling
from 832.036 to 562.394 ms. Both isolation files contain five successful
alternating pairs per fixture. Together with the stage attribution, these
comparisons distinguish the two sources of repeated preparation without
assuming their effects are independent.

That candidate was selected as an implementation improvement, not a new language
boundary. Its residual growing-context cost was still substantial:
128 independent pairs took about 1.15 s even after reuse, and three uses in
that context took about 0.55 s. Larger growing-context cells were not started in that comparison;
no verdict or practicality claim is inferred for them. The real-program
controls establish no regression on these two programs, not a general
compiler or runtime speedup.

## Entering-context follow-up

The next comparison starts at merged main `277a1844`, after query-preparation
reuse. It asks what accounts for the remaining growing-context cost, not
whether another compiler limit should replace the selected proof rules.
Reuse the same three source families at 16, 64, 128 and 256 pairs/uses;
larger cells are optional measurements, not required acceptance samples.
Keep fixed-context 4096 uses and the ordinary prefix and histogram programs
as protected controls. The new scatter source is an additional real-program
control, not a different proof-checking path. A closure implementation change
also protects `tests/programs/wfgrep.wf`, whose many accepted closure cells
exercise the maintenance cost a generated mostly-redundant graph can hide.

First reproduce the curve with the ordinary gate-profile compiler, then
separate complete L0 closure, affine-index construction, candidate traversal,
and certificate premise queries with native samples and temporary stage
instrumentation. In particular, compare growing-N with the N-pair/three-use
control. Pre-kill materialization is a separate known cost: do not infer that
this experiment measures or repairs it without observing that path.

A candidate must remove attributed implementation work while preserving
complete ENT-4 closure and ENT-6/PRF-1 derivation, entering-state independence,
checked arithmetic, and deterministic candidate/witness selection. Record
the proposed mechanism, alternatives and predicted affected cells before
timing the candidate. Retain the earlier selection rule: five alternating
pairs, at least a 2x median improvement at a reproduced costly cell, and no
protected-control regression greater than both 10 percent and 1 ms. Any
changed witness construction also needs direct equivalence or focused
positive/negative cases; timing alone cannot validate it. No source-language
amendment, skipped family, budget, or cross-flow cache is selected by this
follow-up.

### Context baseline, 2026-09-14

The [new raw baseline](../../experiments/proof-use-cost/context-baseline-2026-09-14.tsv)
uses the unchanged generator and compiler at `277a1844` (the criteria commit
`fb02cb34` changes only this document), on the same M1 Pro, 8-CPU, 32-GiB
macOS 26.6.2 host with Rust 1.98.1 and the gate profile. The seven-accept/two-
PRF-1-negative harness check ran before these three repetitions per cell.

| Size | Fixed context, median | Growing context and uses, median | Growing context, three uses, median |
|---|---:|---:|---:|
| 16 | 13.708 ms | 18.923 ms | 17.787 ms |
| 64 | 16.993 ms | 172.838 ms | 98.753 ms |
| 128 | 19.055 ms | 1159.210 ms | 571.361 ms |
| 256 | 26.863 ms | 9848.927 ms | 4216.063 ms |

These are exploratory observations, not candidate-selection evidence. Other
compiler tests were observed on this interactive workstation during the
investigation; no uncontended-run claim is made for this curve. Fresh paired
runs without competing compiler/test jobs are required for selection.

### Row-summary candidate and prediction

A native sample of the 256-pair baseline, taken after process initialization
for three seconds at one-millisecond intervals, places all 2413 driver-thread
samples in `close_with_excluded_term`, called by function-entry analysis;
2328 terminate in that function's own instructions. This identifies that
phase's transitive loop, not a percentage of whole compilation. Competing
test jobs were active, so elapsed time from this sampled run is not used for
selection.

The [temporary context instrumentation](../../experiments/proof-use-cost/context-stage-timing.patch)
applies to `277a1844`; it is not part of the candidate compiler. Its
[raw stage marks](../../experiments/proof-use-cost/context-stages-2026-09-14.tsv)
retain the entry closure and every mark through the proof's residual check.
`proof-*` marks are cumulative from proof-flow entry; the other marks time
one operation. `items` counts terms for closure/index, target atoms for AUTO,
and uses for proof marks; `entries` counts incoming bounds, indexed L0
images, or listed automatic premises respectively. The four instrumented
invocations completed successfully, but competing builds/tests were active;
these timings attribute work and must not be compared with ordinary runs.

| Fixture | Five L0 closures, total | Two affine indexes, total | Final L0-candidate AUTO traversal | Written accumulation |
|---|---:|---:|---:|---:|
| Growing-128 | 1449.572 ms | 105.236 ms | 1988.556 ms | 0.701 ms |
| Control-128 | 1551.555 ms | 134.131 ms | 125.022 ms | 0.190 ms |
| Growing-256 | 11469.501 ms | 800.640 ms | 14926.492 ms | 1.564 ms |
| Control-256 | 11732.713 ms | 788.255 ms | 310.963 ms | 0.195 ms |

Each source closes once at function entry, twice during complete target AUTO,
once for premise admission, and once for the final residual. The index has
65,793 images at 128 pairs and 262,657 at 256. The matched controls separate
the closure cost from the additional long-target candidate work: both matter,
while source-order accumulation is small here. No non-fast pre-kill
materialization mark was observed in these invocations. This study therefore
does not attribute the separate pre-kill TODO to certificate checking.

The proposed candidate summarizes each dense closure row with its populated
cell count, a lower bound on every cell's numeric bound, an upper bound on
every cell's numeric bound, and an upper bound on its proof depths. A row's
maximum and maximum depth may remain conservatively high after improvement;
every new or stronger cell must maintain all three bounds.

For one fixed `(left, middle)` product, let `a` be its first bound, `m` the
outgoing row's lower bound, `M` the destination row's upper bound, and `D`
the destination row's maximum proof depth. Only a completely populated
destination row is eligible. If `sat_add(a, m) > M`, every candidate is
numerically worse. If they are equal and `sat_add(depth(first), 1) > D`,
every candidate is numerically worse or has a strictly deeper proof. These
are the existing scalar rejection conditions applied conservatively to a
whole product; equal-depth ties still take the original traversal. Incomplete
rows and an inconclusive summary retain the original checks. Read the current
summary for each product so an incident-cell update cannot invalidate it.

This is proposed over scanning every product because the sample identifies
that traversal and the sufficient rejection condition preserves its accepted
update sequence. Extending a statement-local closure cache would not address
the sampled function-entry closure and leaves each closure's growing cost.
A different shortest-path/witness algorithm would require a larger proof-order
comparison and is not justified by this initial attribution.

Prediction, recorded before implementing or timing this candidate: it reduces
closure work in both growing-N and the N-pair/three-use control, but does not
change affine-index construction or the AUTO candidate families. Large
independent integer contexts should benefit most; small and already-closed
states have little to gain and are protected against summary-maintenance
overhead. Verify skipped products against their scalar rejection conditions
and compare complete closed facts and derivation selection before relying on
the paired performance result. The measurements below decide whether this
prediction holds.

### Row-summary selection, 2026-09-15

The [five-pair comparison](../../experiments/proof-use-cost/paired-rows-2026-09-15.tsv)
uses baseline compiler source `277a1844` and candidate `8ace3242`, on the
same M1 Pro/macOS 26.6.2/Rust 1.98.1 host and gate profile recorded above.
The mechanism and native tests were introduced at `931b2274`; the candidate's
subsequent commits change only a TODO entry and a scratch-index comment.
The source generator remains the one at `8881833d`. The measured binaries'
SHA-256 identities are:

```text
baseline  600455248020312193f5a682886a1f5c2799ee7b1a975202f81ced916f8bf037
candidate 1ea937c8caebbe777cee37ccc04ed81290b407994c60ee0bf1f39b5e870009a7
```

These fresh comparisons ran after the competing compiler/test jobs finished.
Both binaries were warmed once, each fixture alternated baseline/candidate
order for five pairs, and every invocation completed. No build or profiler
ran alongside the comparison; process checks before, during and after it
found no competing compiler/test job. Times include startup and LLVM emission,
not input generation, host linking, program execution or the earlier
instrumentation.

| Fixture | Baseline median | Candidate median | Speedup |
|---|---:|---:|---:|
| Fixed context, 16 uses | 13.412 ms | 13.272 ms | 1.01x |
| Growing context, 16 uses | 17.904 ms | 17.026 ms | 1.05x |
| 16-pair context, three uses | 15.949 ms | 14.827 ms | 1.08x |
| Fixed context, 64 uses | 15.558 ms | 15.394 ms | 1.01x |
| Growing context, 64 uses | 174.417 ms | 119.032 ms | 1.47x |
| 64-pair context, three uses | 96.878 ms | 42.499 ms | 2.28x |
| Fixed context, 128 uses | 19.137 ms | 19.393 ms | 0.99x |
| Growing context, 128 uses | 1150.888 ms | 730.642 ms | 1.58x |
| 128-pair context, three uses | 558.589 ms | 134.278 ms | 4.16x |
| Fixed context, 256 uses | 25.923 ms | 25.439 ms | 1.02x |
| Growing context, 256 uses | 8988.148 ms | 5501.185 ms | 1.63x |
| 256-pair context, three uses | 4155.557 ms | 626.201 ms | 6.64x |
| Fixed context, 4096 uses | 293.719 ms | 294.587 ms | 1.00x |
| Prefix program | 174.620 ms | 176.433 ms | 0.99x |
| Histogram program | 177.264 ms | 179.131 ms | 0.99x |
| Stable scatter program | 980.749 ms | 998.660 ms | 0.98x |
| wfgrep program | 40366.798 ms | 40361.663 ms | 1.00x |

All 170 timed invocations accepted. The candidate meets the recorded 2x
criterion at the 64-, 128- and 256-pair three-use controls. No protected
control regresses by both 10 percent and 1 ms: the largest relative increase
among the real programs is scatter's 1.83 percent, and wfgrep is essentially
unchanged. The fixed 4096-use fixture also remains essentially unchanged.
The 256-pair control ranges are 4.104–4.222 s before and 0.598–0.660 s after;
growing-256 ranges are 8.956–9.077 s before and 5.439–5.537 s after.
These are checking-cost results for these sources, not runtime speedups or
a universal scaling guarantee.

Select the row-summary implementation. It removes provably non-improving
transitive products without changing the accepted candidate sequence, and
the matched-context gains support the attributed closure cost. This is an
addition to query-preparation reuse, not a replacement for it: preparation
reuse still prevents repeated premise closures, while row summaries reduce
work inside each remaining closure. The
[closure-row-dominance decision](../../../design/compiler/closure-row-dominance.md)
records this choice.

The native state tests check the sufficient rejection condition against
individual scalar comparisons at `i128`/depth saturation boundaries, retain
equal-depth ties and incomplete rows, and check summary updates after a
stronger bound acquires a deeper proof. Another test instantiates the same
engine with pruning disabled and compares all closed facts, selected proof
handles and the complete derivation ledger across 256 finite graphs, each
with and without an excluded term. The 512 comparisons include varied
source order, equal paths and disequality strengthening. This is regression
evidence for the optimization, not an independent proof of the whole closure
engine. All 15 state tests and the seven-accept/two-PRF-1-negative probe pass,
including the full 4096-use ceiling. No specification, conformance verdict,
candidate-family, diagnostic-selection or erasure rule changes.

The remaining cost is material: growing-256 still takes 5.50 s, and even
three uses in its context take 0.626 s. Complete matrix/index construction
and the unchanged long-target AUTO traversal remain work; this candidate
does not make them linear or universally cheap. Larger cells were not
started. The generated sources did not exercise non-fast pre-kill
materialization, so that separate TODO is neither measured nor claimed fixed.

To reproduce this follow-up, build the two pinned ordinary compilers before
starting the comparison:

```sh
context_cost_root=$(mktemp -d)
context_cost_checkout=$PWD
git worktree add --detach "$context_cost_root/baseline" 277a1844
git worktree add --detach "$context_cost_root/candidate" 8ace3242
cargo build --manifest-path "$context_cost_root/baseline/compiler/Cargo.toml" \
  --profile gate --bin whitefootc --locked --offline
cargo build --manifest-path "$context_cost_root/candidate/compiler/Cargo.toml" \
  --profile gate --bin whitefootc --locked --offline
make -C research/experiments/proof-use-cost compare \
  WORK_ROOT="$context_cost_root/probe" \
  BASELINE="$context_cost_root/baseline/compiler/target/gate/whitefootc" \
  CANDIDATE="$context_cost_root/candidate/compiler/target/gate/whitefootc" \
  SIZES=16,64,128,256,4096 \
  REAL_SOURCES="$context_cost_checkout/research/experiments/compute-bench/programs/prefix.wf $context_cost_checkout/research/experiments/compute-bench/programs/histogram.wf $context_cost_checkout/research/experiments/compute-bench/programs/radix_scatter.wf $context_cost_checkout/tests/programs/wfgrep.wf"
```

## Reproduction and correctness boundary

The native driver is
[`runner.rs`](../../experiments/proof-use-cost/runner.rs), whose generator and
comparison mode are pinned at `8881833d`. Create disposable detached worktrees
and build the three ordinary compilers before timing:

```sh
cost_root=$(mktemp -d)
cost_checkout=$PWD
for cost_pair in baseline:50ddd638 closure:d465986c combined:8881833d; do
  cost_label=${cost_pair%%:*}
  cost_revision=${cost_pair#*:}
  git worktree add --detach "$cost_root/$cost_label" "$cost_revision"
  cargo build --manifest-path "$cost_root/$cost_label/compiler/Cargo.toml" \
    --profile gate --bin whitefootc --locked --offline
done
make -C research/experiments/proof-use-cost compare \
  WORK_ROOT="$cost_root/combined-probe" \
  BASELINE="$cost_root/baseline/compiler/target/gate/whitefootc" \
  CANDIDATE="$cost_root/combined/compiler/target/gate/whitefootc" \
  SIZES=16,64,128,4096 \
  REAL_SOURCES="$cost_checkout/research/experiments/compute-bench/programs/prefix.wf $cost_checkout/research/experiments/compute-bench/programs/histogram.wf"
```

For the first isolation, select baseline and closure binaries with `SIZES=64`;
for the second, select closure and combined binaries with `SIZES=128`.
Leave `REAL_SOURCES` empty for those comparisons. The comparison target never
rebuilds either compiler, warms both once, and requires five alternating pairs.
It emits one TSV row per invocation; retain compiler labels with each result.
The ordinary `bench` mode can explore other sizes without treating its times
as paired selection evidence.

Root `make check` calls the experiment's `check` target: seven accepted
fixtures, including 4096 uses at fixed context, and invalid-premise and
duplicate-use controls requiring PRF-1 rejection. No timing threshold enters
the gate. The driver is format-checked and built with safe Rust and warnings
denied. Substituting `/usr/bin/true` fails on the invalid-premise control;
substituting a nonexistent compiler fails to start. Neither wrong success
nor a missing executable can silently produce a green experiment check.

The ordinary source-proof suite covers independent premise admission, exact
diagnostic locations, duplicate identity, signed tightening, overflow order,
source formation and capacity. The query-view tests additionally cover reuse
and invalidation by new terms, an existing measure's changed bound, and an
existing goal's newly supplied projection. The 1043-test semantic suite and
all-target Clippy check passed on the candidate. These are correctness
controls for changed query preparation, not a proof that the compiler is sound.
