# Compute performance in the shared runtime

## Current delivery and acceptance scope (2026-09-09)

The owner requires delivery through ordinary `whitefootc --par source.wf -o
program`, using the maintained shared runtime in `compiler/src/backend/sched/`.
Refactor that implementation where its design or code quality obstructs compute
performance. Do not choose a different runtime because a module contains I/O.
The existing research runtime is a frozen comparison input during this work,
not the implementation home; retire the duplicate once its comparison purpose
is served. I/O optimization is deferred, but completion, mixed compute/I/O
progress, frame lifetimes and exhaustion handling must remain correct. Changes
to source execution semantics or public ABI require discussion with the owner.

The target set is the compiler's closed set in
[`target.rs`](../../../compiler/src/backend/target.rs), not the smaller set
currently covered by compute measurements:

| Target | Required evidence |
| --- | --- |
| `aarch64-apple-darwin` | Native CI correctness and performance |
| `x86_64-apple-darwin` | Native CI correctness and performance |
| `aarch64-unknown-linux-gnu` | Native CI correctness and performance |
| `x86_64-unknown-linux-gnu` | Native CI correctness and performance |
| `x86_64-pc-windows-msvc` | Native CI correctness and performance |

Completion requires measured acceptance on **every row**. Cross-compilation,
emulation, local M1 measurements, missing runners or correctness-only jobs do
not substitute for a row. Keep an unavailable or noisy row unresolved.

Before selecting runtime changes, use these experimental criteria:

- Compare candidate and baseline on the same CI machine, alternating order,
  with identical inputs, worker budgets, arithmetic, vectorization and build
  options. Scheduling comparisons disable SIMD and LTO on both sides. Retain
  exact revisions, sources, commands, binaries and raw samples.
- First isolate runtime cost with the identical generated WF object, then
  measure normal CLI executables with pure compute and compute before/after
  light I/O. An experimental linker comparison alone does not deliver the goal.
- Cover the distinct computations in [WORKLOADS.md](WORKLOADS.md), including
  fine/coarse work, skew, nested composition and repeated/bursty work. Check
  outputs with independent oracles. Retain startup, wall time, CPU cost,
  memory and scaling separately; instrumented runs explain ordinary timings.
- Start with five alternating process pairs per cell, keeping warm invocations
  inside a process distinct from independent process samples. Investigate a
  repeatable candidate/baseline wall-time ratio above 1.05 in any cell. Use
  further independent cohorts to resolve noise rather than discard outliers
  or average a loss away. An unresolved cell cannot pass acceptance.
- Matching the research baseline requires each cell to be within 5% on wall
  time, with repeatable CPU or memory increases above 5% separately explained
  and resolved; a wall-time win cannot hide excessive spinning. This 5% band
  is an initial engineering equivalence criterion, not a universal noise
  estimate. Native references from [BASELINES.md](BASELINES.md) still test
  whether the baseline itself leaves avoidable scheduling cost.
- The old research runtime is POSIX-only. Windows must use its existing formal
  runtime as the before-control and matched native references to qualify the
  resulting scheduler; an untested port of the old research code would not be
  a qualified baseline. Report this difference explicitly.

The first bounded candidate gave current-stack compute helping priority over
taking another stack at a compute join. A local M1 FIR screen with the same
scalar WF object found no consistent improvement; at four workers the 64-output
tile mean went from 40.9 to 47.4 microseconds, versus 15.1 for the recovered
control. That change was reverted. This screen used five processes per cell,
256 warm calls per process, 4,096 outputs and 16 taps; it is not CI acceptance.

The current candidate avoids locking the ready list merely to observe that it
is empty. List mutation stays locked, the head is read and written atomically,
and the wake-epoch protocol is unchanged. The native smoke and all four
scheduler enumeration configurations pass. Initial local timings are mixed,
so performance selection remains open: do not describe this as an accepted
optimization. A sampled long FIR run found substantial condition-variable
waiting, yielding and wake-mutex contention; ready-list locking alone does not
explain the gap. Sampling includes output verification and idle threads and
does not measure compute-region CPU fractions.

`make -C research/experiments/compute-runtime formal-screen OUT=<fresh-path>`
reproduces the initial POSIX screen through the current compiler, with fixed
formal-before revision `188088d41552d0d3bccf8368798dcc44702bf75c`, a checked
unchanged recovered baseline, and the candidate formal runtime. CI covers the
four POSIX targets; the existing Windows native mixed-runtime protocol remains
in `io-bench.yml`. This screen is FIR-only, its normal CLI execution is a
correctness check, and its threshold evaluates warm core wall time only. It
cannot complete the broader workload, CPU, CLI timing or five-target goal.
Linked-image layout can also change despite using identical WF object bytes;
small differences require independent confirmation and attribution.

The first [four-target native CI screen at `708e3c4d`](https://github.com/mbbill/Whitefoot/actions/runs/34343071425)
completed its oracle, normal-CLI correctness and actual-pool-width checks, but
all four targets failed its performance band. Representative median paired
candidate/recovered ratios (16 taps, tile 64, five processes per cell) are:

| Native target | Participants | 4,096 outputs | 65,536 outputs |
| --- | ---: | ---: | ---: |
| Linux x86-64 | 4 | 1.741 | 1.390 |
| Linux AArch64 | 4 | 2.088 | 1.512 |
| macOS x86-64 | 4 | 5.444 | 1.206 |
| macOS AArch64 | 2 | 1.777 | 1.062 |

The macOS AArch64 runner exposes three CPUs, so the screen measures widths one
and two there. Linux x86-64 exposes four logical CPUs on two SMT cores; these
rows are within-host comparisons, not comparable four-physical-core machines.
The run's `formal-runtime-<target>` artifacts retain raw samples, sources,
binaries, build options and host identity. In one Linux four-participant small
batch, candidate voluntary/involuntary switches total 1,023 versus 50 for the
control; startup is 2.151 ms versus 0.210 ms and peak RSS 25.16 MB versus 2.33 MB.
These process measurements include verification and are not core-only CPU
profiles. The [Windows mixed timing run](https://github.com/mbbill/Whitefoot/actions/runs/34343071339)
failed because `io-warm` remained unstable across two complete cohorts.
[Linux and Windows completion correctness](https://github.com/mbbill/Whitefoot/actions/runs/34343071312)
passed; that does not make Windows performance qualified.

The next bounded candidate initializes only the configured lane prefix and the
trailing status/idle metadata. The original full-capacity clear touches about
20 MiB even with one participant. The prediction is lower startup time and RSS
without a warm regression. It preserves the original hot-data layout and all
public-call/task-frame ABIs. A first variant moved metadata before the lanes;
two independent local M1 cohorts found a repeatable roughly 6% small-input
loss, so that layout change was removed. Clearing the two live regions instead
retains the startup saving without needing an enumerator layout change. The
poisoned-storage smoke checks that live fields do not depend on pristine BSS.
Local FIR measurements use the same WF object, widths one/four, 16 taps,
4,096/65,536 outputs, tiles 64/1,024 and five alternating processes of 256 warm
calls per cell. Against the published shared-runtime control at `708e3c4d`, the
unchanged-layout candidate is within the wall-time band in seven cells; the
remaining small-input median ratio is 1.060 with a wide 0.775–1.116 paired range.
This is not parity with the recovered runtime. Warm acceptance remains open;
startup savings alone do not select it.

The compute-join hypothesis is that short stolen work finishes sooner than
the shared park/resume round trip. Give a compute join a bounded interval of
current-stack pop/steal/help and processor pauses before taking an EMPTY stack.
READY stacks remain immediately eligible, and every empty-handed turn still
drains I/O progress; after the interval the existing park/exhaustion protocol
applies. This changes internal scheduling policy, not calls or source semantics.
Compare zero, 16, 64 and 256 turns using identical WF object bytes; reject a
wall-time gain bought with a repeatable CPU-cost increase above the stated band.
The earlier rejected helper-first change had no bounded wait when no work was
available, so it did not test this short-completion hypothesis. Qualify mixed
progress and all enumerator configurations before publishing any selection.

The branch candidate uses 256 outer turns following the local zero/16/64/256
screen. This bounds neither callback duration nor recursive stack depth. The
final local `formal-screen` (five processes, 64 warm calls, widths one/two/four)
passes all 24 wall comparisons against the fixed formal-before control, but
fails 12 of 24 against the recovered runtime. Selected four-participant results
are below; CPU is the whole measured batch including checks, not core-only.

| FIR input / tile | Before core mean | Candidate core mean | Recovered core mean | Before / candidate batch CPU |
| --- | ---: | ---: | ---: | ---: |
| 4,096 / 64 | 36.59 us | 28.14 us | 14.53 us | 15.72 / 10.91 ms |
| 65,536 / 1,024 | 149.09 us | 135.75 us | 133.80 us | 72.78 / 59.78 ms |

For the first row, startup falls from 1.471 to 0.176 ms and peak RSS from
22.51 to 3.36 MB. Mean process context switches fall from 647 to 340; for the
second row they fall from 1,135 to 268. These are local candidate results, not
cross-platform acceptance. The screen still does not time the normal CLI path,
and its pool-width check is not a witness of useful work on every worker.

All four reduced-bound enumeration configurations pass with one help turn;
the (two-thread, four-stack) sweep explores 44,732,346 states. This checks the
protocol at that bound, not all production histories. The native smoke runs
32 successive sibling tasks on the caller's existing stack while the other
workers hold its join target; every result is checked and joined before frame
release. The default passes, while zero help turns fail its current-stack
assertion. Its mixed I/O cases and poisoned initialization also pass. Independent
scoped review found no remaining issue in these changes after correcting the
model-coverage claim.

Native code-quality work remains: deque cells are still plain accesses despite
possible stale thieves racing ring reuse, and diagnostic counters can be read
while workers update them. The screen avoids the concurrent counter snapshot,
but the shared runtime still updates counters while the recovered timing build
disables them. These are unresolved correctness/accounting issues, not justified
by a successful enumeration or a faster timing. They must be addressed before
final qualification.

## Earlier investigation and evidence

The selected question is whether Whitefoot's proof-derived compute parallelism
can approach the fastest equivalent native implementations while tasks execute
on each worker's current call stack. Recover the local join/help/steal path,
remove completion scheduling from the compute execution path, and measure the
remaining costs separately from generated kernel code. Performance takes
precedence over sharing an execution mechanism with I/O.

This investigation starts at main revision
`6cc00984415a39c507fa74897c9269b10beebfee`. The source audit below identifies a
recovery control, now implemented and locally qualified in the
[compute runtime experiment](../../experiments/compute-runtime/README.md).
The normal compiler link path is unchanged. The experiment retains qualified
workloads, native references and dated performance results. The
[WF workload coverage](WORKLOADS.md) and [reference matrix](BASELINES.md)
distinguish that evidence from broader comparisons still missing.
Further compiler scheduling-policy development is deferred. The existing
scalar-leaf filter becomes the `--par` default at 16 nonconstant IR operations,
with an explicit `off` override. Recursive frontier and sequential refusal
remain opt-in; no universal recursive grain is selected. The
[compiler guide](../../../compiler/README.md#parallel-and-completion-lowering)
owns these current defaults and their evidence limits.
The immediate integration scope is the measured compute foundation and its
test/reference tools, before returning to I/O execution design.
The separate
[I/O investigation, PR #26](https://github.com/mbbill/Whitefoot/pull/26), is
paused and retains its implementation and measurements. Its stackless path is
not the base of this work.

This directory owns the compute runtime experiment's rationale and reference
selection. Keep it current while that experiment is active; consolidate or
remove superseded material when another investigation takes over the question.
The active [specification](../../../spec/kernel-spec.md) defines acceptance and
the [compiler guide](../../../compiler/README.md) describes implemented paths.
The recovery experiment adds executable checks under the root `make check`;
it changes no language rule, public ABI, or conformance expectation.

## Recovering the actual compute path

The relevant revisions are different controls, not interchangeable historical
performance baselines:

| Revision | Source evidence and role |
| --- | --- |
| `408dd34ebe4a385486002b32dbc3ecaf661b2aaa` | [Introduces per-thread work-stealing deques](https://github.com/mbbill/Whitefoot/commit/408dd34ebe4a385486002b32dbc3ecaf661b2aaa). It identifies the original mechanism; its dated performance claims do not qualify today's compiler or references. |
| `fee335654d9dea027f4636bbad448d57a4e84d08` | Last repository revision before the `17ec9458` I/O integration. Its [parallel runtime](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c) is the recovery reference for a compute worker pool without completion bridge integration. |
| `9051576f6a4d723b4eb072850f49859853decae7` | Parent of the POSIX replacement. Its [parallel runtime](https://github.com/mbbill/Whitefoot/blob/9051576f6a4d723b4eb072850f49859853decae7/compiler/src/backend/par_runtime.c) still helps on the current stack, but already exposes a help seam for the completion bridge. It is not the clean pre-I/O control. |
| `92b19e1a703159d461932792bc334c10d2e89b89` | [Replaces the POSIX compute runtime](https://github.com/mbbill/Whitefoot/commit/92b19e1a703159d461932792bc334c10d2e89b89) with `sched/core.c` and `sched/entry.c`, deletes `par_runtime.c`, and puts the program entry on a scheduler pool stack. |
| `6cc00984415a39c507fa74897c9269b10beebfee` | Current investigation base. Its [compute join](../../../compiler/src/backend/sched/core.c) retains the shared stack scheduler. This is the current-runtime control, not a restored compute runtime. |

At `fee33565`, `wf__par_join` owner-pops its task and calls it directly if it
has not been stolen. Otherwise it executes other local tasks and enters
`wf__par_wait`, which checks the target, owner-pops, steals, and runs available
work nested on the current stack. Only after repeated empty searches does it
yield and eventually wait on a condition variable. No suspended-stack pool or
I/O completion drain appears in that path. See the historical
[join](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c#L757)
and [wait](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c#L457).

The old implementation is not cost-free. Owner-pop has strong atomic ordering
and races with the thief on the last entry; publication checks idle workers
and may wake one; task completion has a waiter handshake; idle workers can
enter the kernel. Reclaiming one's own newest task avoids the shared completion
publication, but that does not make all offers or all joins free. Those costs
must be measured rather than inferred from the older comments.

At the current base, `wf_sched_join` checks DONE and attempts its own newest
child first. It then calls `wf_sched_take_target` and parks if another stack is
available. Nested owner-pop/steal execution is the no-target compute fallback.
This reversal of priority is visible in
[core.c](../../../compiler/src/backend/sched/core.c). Changing `WF_STACKS`
does not restore the old policy: the pool has a supported minimum and still
supplies switch targets. The earlier
[park-on-miss design, section 12](../io-model/PARK-ON-MISS.md#12-measurements-before-a-line-of-the-compiler-changes)
already identifies nested helping as the compute comparison.

## Where I/O currently enters a compute measurement

The link driver uses the union of parallel and completion requirements to
include the shared scheduler. `wf__floor_run` then delegates the program entry
to that scheduler when it is linked. The coupling therefore includes startup,
stack reservations, task completion, joining, idle progress, and shutdown; it
is larger than the stack-switch instruction sequence.

There is a second coupling: `par_layout.wf` publishes its checksum with
`write_once`, and the driver test explicitly requires that program to link
completion support. A computation with no I/O in its hot loop can therefore
still carry the whole shared scheduler. Inspect
[the driver and its runtime-selection test](../../../compiler/src/bin/whitefootc.rs)
and [the entry floor](../../../compiler/src/backend/wf_floor.c).

The restored path must be distinguished by its actual linked and executed
runtime, not by an I/O-free workload label. Its kernel should return a value
or fill caller-owned output that a host oracle checks outside the timed
compute region. A separately reported end-to-end measurement must include
equivalent setup and output costs. Do not replace a real computation with an
unchecked return code or silently subtract those costs from an end-to-end row.

Removing I/O costs from compute does not authorize weakening stack-exhaustion
handling, memory safety, or task-result lifetime rules. Recovery needs the
current compiler's ordinary task entry contract and result ownership; simply
linking the old runtime against new frames without checking layout and calling
conventions would not establish compatibility. Keep existing I/O tests intact
as coverage of that capability while its execution mechanism is separated.

## What the first measurements must distinguish

1. **Kernel quality:** current WF sequential code against optimized equivalent
   native sequential code, with identical arithmetic and output semantics.
2. **Scheduling policy:** the same emitted compute kernel and task boundaries
   with the current shared runtime and the restored compute runtime. Attribute
   any compiler or layout differences explicitly if identical code is not
   achievable.
3. **Unstolen cost:** both sequential-clone execution and actual parallel-path
   execution without successful steals. A one-worker setting that selects the
   sequential clone cannot measure task publication and local join costs.
4. **Resource cost:** worker counts, startup and shutdown, allocations, stack
   reservations versus resident memory, task publication, steals, kernel
   switches, and work performed while another task is joined. Timing and
   diagnostic observation use separate builds or runs.
5. **Competitive performance:** the fresh [reference matrix](BASELINES.md),
   including static partitioning where the workload permits it. A win against
   one dynamic scheduler is not an upper bound on native performance.

Use current accepted programs and equivalent native kernels. Historical
[proof-derived parallelism results](../proof-derived-parallelism/RESULTS.md)
and its [dated reference rotation](../proof-derived-parallelism/bench/baseline-20260823/README.md)
identify useful workloads and earlier issues. They do not replace a new
same-host comparison: compiler semantics, code generation, thread budgets,
toolchains, machines, and statistical treatment have changed.

`par_layout.wf` remains a historical regression sample. The new investigation
needs multiple substantive WF programs with different algorithms, layouts,
inputs, and parallel structures, as defined in [WORKLOADS.md](WORKLOADS.md).
Adding native references or increasing the layout repetition count does not
provide that coverage. The WF program suite and its native references are
separate implementation deliverables.

## Execution contract to preserve

Compute tasks remain ordinary calls on worker stacks. At a join, execute
available permitted compute work before sleeping; an I/O completion cannot be
a hidden dependency of that helping path. The existing ownership and proof
judgments remain the authority for legal overlap. Public signatures and
contracts must suffice for separate compilation; no hidden coroutine calling
convention is selected by inspecting a separately compiled function body.

Whether regular loops should use static partitioning, how coarse tasks should
be, and which idle policy minimizes measured cost remain experiment questions.
The target is explainable proximity to each workload's hardware and dependency
limits, with losses exposed. No universal fastest-runtime claim follows from
the historical audit or the amount of time spent on the preceding I/O work.
