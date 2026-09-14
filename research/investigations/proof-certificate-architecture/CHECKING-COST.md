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

The candidate is selected as an implementation improvement, not a new language
boundary. The residual growing-context cost is still substantial:
128 independent pairs take about 1.15 s even after reuse, and three uses in
that context take about 0.55 s. Larger growing-context cells were not started;
no verdict or practicality claim is inferred for them. The real-program
controls establish no regression on these two programs, not a general
compiler or runtime speedup.

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
