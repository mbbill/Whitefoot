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
unchanged. This is proposed under `compiler/proof-query-context`; the live
tree is not edited without a ruling.

The alternative of materializing a live fact snapshot is not selected: it
adds snapshot events and independent fact support when only an ephemeral
query view is needed. A cross-flow cache would require kill/join invalidation
across the walker and is not needed to remove the measured repetition.
Pruning AUTO or accepting redundant blocks would change the language and is
outside this implementation experiment.

Prediction, before candidate measurement: closure reuse reduces the premise
stage but not target AUTO; endpoint reuse reduces target AUTO but not repeated
closure. Both should improve growing-64/128 while leaving fixed-context and
small cases within the protected criterion. Compare the two isolated changes
as well as their combination. Keep prefix and histogram as real-program
controls using their ordinary current source and unchanged LLVM emission.
