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
