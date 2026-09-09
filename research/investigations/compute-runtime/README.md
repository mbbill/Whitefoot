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
four POSIX targets and Windows. The Windows job invokes the same script with
the native MSVC-target compiler and existing Windows runtime leaves; it compares
the fixed formal-before scheduler/floor, candidate, frozen previous revision
and same-source longer-idle-window control. A byte-identical candidate replica
measures process/host variability. Previous keeps its own frozen Windows
host/completion sources; candidate and the oldest before control link current
host/completion sources beside matching private headers.
This overlay is needed by generated host diagnostics; "before"
does not mean an entirely historical Windows runtime. No research runtime
is ported. Its benchmark uses QueryPerformanceCounter for elapsed time,
GetProcessTimes for whole-process CPU time, and peak working set for memory;
context-switch counts are explicitly unavailable. CPU times' 100-ns units do
not imply that small batches have 100-ns accounting resolution. Strong native
parallel references and CPU acceptance remain required on Windows. The existing
mixed-runtime protocol remains in `io-bench.yml`. This screen is FIR-only, its normal CLI execution is a
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

The [native CI screen at `aeb35be5`](https://github.com/mbbill/Whitefoot/actions/runs/34346052104)
still fails the performance band on all four POSIX targets. Selected median
paired candidate/recovered wall ratios are below; widths are configured pool
participants, not proof of useful work on every participant.

| Target | Width | 4,096 / tile 64 | 65,536 / tile 64 | 65,536 / tile 1,024 |
| --- | ---: | ---: | ---: | ---: |
| Linux x86-64 | 4 | 1.857 | 1.412 | 1.353 |
| Linux AArch64 | 4 | 2.230 | 1.516 | 1.037 |
| macOS x86-64 | 4 | 2.818 | 1.965 | 0.940 |
| macOS AArch64 | 2 | 1.729 | 0.985 | 1.011 |

Linux x86-64 at 65,536 / tile 256 reaches 0.990, while smaller tasks still
lose substantially. Some paired ranges are wide: Linux AArch64 at 4,096 /
tile 64 spans 1.303–5.332. A favorable median in such a cell does not establish
stable equivalence. Canonical local `make check` passes at `aeb35be5`, including
the full native conformance and snapshot adapters; performance acceptance is
separate and remains open.

Native correctness review found two defects in the shared deque: plain cell
accesses race with stale thieves during ring reuse, and acquire-only thief
index loads lack the ordering needed against the owner's claim. The maintained
core now uses relaxed atomic cell loads/stores and sequentially consistent
thief top/bottom loads. For the duplicate-claim history, the SC order is owner
bottom decrement, owner top check, first thief CAS, second thief top read,
second thief bottom read. The last read cannot select the older bottom. This
argument assumes no complete 64-bit counter rollover; it is not a complete
weak-memory proof. The ordering issue and atomic-cell requirement agree with
the analysis in [Lê et al., PPoPP 2013](https://fzn.fr/readings/ppopp13.pdf).
Apple ARM code generation changes from `ldapr` to `ldar` for the index loads;
the repair's performance cost must be measured, not assumed zero.

Live diagnostic counters now use relaxed atomic reads and single-writer
load/store increments. The writer is the physical thread, reloaded after a
possible migration. These observations add no synchronization or scheduling
edge and are not one simultaneous snapshot. The enumerator does not branch on
counter accesses; it now does branch on ring-cell accesses and hashes the full
ring, retaining stale-reader and unpublished-push states. All four reduced
configurations pass after that change, with 44,819,639 states for two threads
and four stacks. This remains SC interleaving evidence, not weak-memory proof.

The compiler-owned `sched-deque-test` exercises the actual core on ordinary
host stacks with eight slots, 200,000 tasks, three thieves, owner pops and a
live observer. It requires exactly-once execution, complete slot return and
monotone observed steal counts. Both native M1 and ThreadSanitizer runs pass.
The same probe reports a counter race with the old core; after applying only
the counter repair to that old core, it reports the separate ring-cell race.
No race suppression is used. POSIX canonical checks and Windows native CI run
the probe; Linux CI additionally runs it under ThreadSanitizer. At `0f1603b2`,
the four POSIX native probes and both [host correctness jobs](https://github.com/mbbill/Whitefoot/actions/runs/34350981830)
pass, including Windows's native probe and Linux's ThreadSanitizer run.
The [partitioned repository CI](https://github.com/mbbill/Whitefoot/actions/runs/34350982089)
also passes on that revision. The canonical local `make check` for `0f1603b2`
completed successfully, including the full native conformance adapter.

The repair's local M1 cost comparison uses byte-identical WF object files,
alternating repaired/`aeb35be5` binaries, widths one/four, inputs 4,096/65,536,
tiles 64/1,024 and five processes with 256 warm calls each. All eight median
wall ratios are within 5% (0.933–1.034); whole-batch CPU median ratios span
1.011–1.050. The four-participant small-input/tile-64 wall ratios span
0.817–1.152, so this does not establish a speedup or stable equivalence there.
The broader 64-call FIR screen still fails. Correctness selects this repair;
native CI and further controlled comparisons must establish its cost.

The recovered runtime remains frozen, including its acquire-only thief index
reads (its ring cells already use atomics);
its timing is historical comparison evidence, not a correctness-qualified
implementation to restore. The shared runtime still updates counters while
the recovered timing build disables them. This accounting asymmetry and
broader fair native references remain unresolved before final qualification.

A local maintained-core layout experiment separated owner-written deque bottom
from the thieves' top and isolated each physical thread's observed counters
on 128-byte boundaries. The current layout packs 136-byte thread records,
allowing independent counter writers to share a line. Two alternating M1
cohorts used identical scalar WF objects, widths one/four, inputs 4,096/65,536,
tiles 64/1,024, five process pairs and 1,024/256 warm calls for small/large
inputs. At four participants, 65,536 / tile 64, candidate/original wall ratios
were 0.958 and 0.922, with CPU ratios 0.974 and 0.958. Small-input/tile-64 wall
ratios were 0.980 and 1.020. One single-participant large-input cell had an
unresolved 1.114 RSS ratio in the second cohort, with roughly 0.56 MB variation
inside both sets. The layout is not selected or retained in the implementation;
resolve the Windows policy regression and memory observation before reopening
this candidate. Native smoke and the 200,000-task deque probe passed; no claim
of cross-platform layout qualification follows.

The [first complete Windows FIR screen at `b4a3283d`](https://github.com/mbbill/Whitefoot/actions/runs/34349696349)
passes oracle, normal CLI and pool-width checks, but fails three of 24 wall
cells against its historical scheduler/floor control. At four participants,
4,096 / tile 1,024 has median paired ratio 2.781, range 2.712–2.981. Its five
process means are 60.94–63.04 us versus 21.12–22.95 us; pooled warm-call p50 is
60.9 versus 15.9 us and p95 is 76.5 versus 69.1 us. These are different sample
levels, not interchangeable confidence estimates. The host is Windows Server
2025, EPYC 7763, two cores/four logical CPUs, high-performance power plan,
Clang 20.1.8. QPC frequency is 10 MHz. Whole-process CPU readings for these
short batches jump in 15.625-ms multiples and include zeros; they cannot
establish CPU efficiency. Do not attribute the regression to helping alone:
the two controls contain several scheduler changes. The next causal control
holds current sources fixed and sets only compute helping to zero, with
separate longer diagnostic batches for current-runtime counters and CPU.

The first local M1 same-source help comparison completed all 480 process
samples and 48 separate diagnostic processes. At four participants and 65,536
outputs, candidate/help0 median wall ratios are 0.878 at tile 64 and 0.909 at
tile 1,024. Small-input ratios are less stable; this does not answer the
Windows regression. One 4,097-call diagnostic process at 4,096 / tile 1,024
records 677 parks for the candidate versus 5,841 for help0, with whole-batch
CPU 418.6 versus 522.3 ms. The report and exact lane/warm-call counts are
validated per diagnostic process. One process is explanatory evidence, not
CPU-performance qualification. The wall verdict is saved before diagnostics
so a later diagnostic failure cannot hide completed measurements.

The [five-target help control at `0f1603b2`](https://github.com/mbbill/Whitefoot/actions/runs/34350982086)
completed all formal screens and their diagnostics, but every platform still
has failing wall cells. At Windows four participants, 4,096 / tile 1,024,
candidate/before is 2.773 (paired range 1.507–3.130); candidate/help0 is 1.128
(0.565–1.788). The zero-help process means are 33.63–55.37 us, still well above
before's 19.22–24.38 us. Thus the helping budget alone does not account for the
regression. Windows artifact `10103808989` has ZIP SHA-256
`7cc55bf141adc868b9e8499d5cd08bea374365b5ff2fa9ba394d4001ba181bea`.

For that cell, separate 4,097-call diagnostic processes record candidate/help0
batch wall 355.4/340.1 ms and CPU 937.5/984.4 ms. Their live process-total
scheduler reports show stack parks 48/7,562, including startup/selection rather
than exactly the batch interval. Whole-batch costs include verification;
one diagnostic process per mode is not
independent CPU qualification. The enormous reduction in stack parks does not
produce a corresponding wall reduction. These are stack switches, not host
sleeps. The next measurement reads the existing current bridge's atomic wait
announcements and host wake signals at batch boundaries, including for the
historical scheduler overlay without reading its unsafe scheduler counters.
The getters live in the maintained compiler, do not initialize the bridge,
and add no updates to hot paths. An announcement may be cancelled before
sleeping, and a signal does not count awakened threads.

The next question is whether faster empty-ready checks shorten the fixed-round
idle window enough to put workers to sleep between bursts, making the next
call pay a host wake. This is a hypothesis, not an attribution. A large increase
in wait announcements/signals accompanying the regression would support a
focused idle-window control; comparable counts would send the investigation
back to other costs. No runtime policy was changed for this measurement.
POSIX attribution images still omit the completion bridge while Windows images
include it; full-link POSIX timing and ordinary CLI timing remain required.
The new getters and FIR instrumentation pass strict C syntax checks and
full-link M1 correctness smokes at one/four participants; these smokes ran
during the canonical check and are not performance measurements. Independent
review found no implementation defect in this diagnostic delta; its metadata
and process-total versus batch-boundary clarifications are incorporated.

The [Windows wait observations at `076a476b`](https://github.com/mbbill/Whitefoot/actions/runs/34352414576)
were recorded on an EPYC 9V74, two cores/four logical CPUs, Windows Server 2025,
Clang 20.1.8, high-performance power plan. This is a different processor from
the earlier EPYC 7763 runs; compare controls within this run. At four
participants, 4,096 / tile 1,024, the five candidate/before paired wall ratios
have median 2.146 and range 1.263–2.837. Each short process has 64 warm calls
plus one retained first call. Its observations are:

| Mode | Five warm process means (us) | Batch wait announcements | Batch wake signals |
| --- | --- | --- | --- |
| Before | 20.66, 16.10, 16.14, 16.59, 20.43 | 33, 29, 24, 26, 35 | 45, 48, 35, 43, 54 |
| Candidate | 44.34, 45.68, 43.92, 20.95, 32.58 | 198, 193, 196, 11, 111 | 194, 199, 196, 10, 108 |
| Zero help | 33.95, 36.34, 34.15, 32.32, 26.45 | 129, 114, 127, 129, 75 | 176, 154, 188, 183, 115 |

The candidate's slower processes coincide with many more wait announcements
and wake signals. The separate long diagnostic process instead has 81
announcements over 4,097 calls and mean warm time 18.16 us, compared with
before's 244 announcements and 19.23 us. It does not reproduce the short-run
loss, so neither ignoring the short samples nor attributing the difference to
call count alone is justified. Long-batch candidate/before CPU is 468.75/437.5
ms and wall is 179.74/177.85 ms; these include verification and remain one
process per mode. Artifact `10104466331` has ZIP SHA-256
`e820fa5a9d45ebc9db8490b939aec00c3f67294df7528229480948ae284fe9e5`.

This co-observation motivated the idle-window control. `idle4096` uses
the identical current runtime and WF object with only the existing
`WF_SCHED_IDLE_SPIN_ROUNDS` set to 4,096 instead of 256. The default is unchanged.
The predicted result is fewer host waits together with removal of the short
Windows loss. If waits fall without wall improvement, that hypothesis is
insufficient. Even a wall improvement cannot select the policy if longer
spinning creates an unresolved CPU regression. The same five-target screen
retains all other cells, CPU observations and existing controls. CI now also
triggers on completion and Windows host changes, whose code is part of the
Windows timing images.
The full-link M1 smoke passes with the longer window and reports 4,096 rounds;
diagnostic validation rejects that report when 256 rounds are expected.
Independent review confirmed the control's scope, wiring and recorded artifact
figures.

The [five-target screen at `6060cc67`](https://github.com/mbbill/Whitefoot/actions/runs/34353534080)
completed, with performance failures on every target. Its separate
[12-job gate](https://github.com/mbbill/Whitefoot/actions/runs/34353534106) and
[Linux/Windows I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34353534059)
passed. On Windows, the EPYC 7763 host had two cores/four logical CPUs,
Windows Server 2025, Clang 20.1.8 and the high-performance power plan. At four
participants, 4,096 / tile 1,024, the five paired candidate/idle4096 wall
ratios have median 3.7455, range 1.6301–4.4359; idle4096/before has median
0.803. The separate 4,097-call diagnostic processes recorded:

| Mode | Warm mean (us) | p50 / p95 (us) | Batch wall (ms) | Batch CPU (ms) | Wait announcements / signals |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before | 17.326 | 15.1 / 27.5 | 186.052 | 734.375 | 258 / 362 |
| Candidate, idle 256 | 53.329 | 59.5 / 68.0 | 353.217 | 765.625 | 9,825 / 9,888 |
| Zero join help | 41.493 | 46.8 / 66.2 | 299.462 | 796.875 | 6,577 / 9,486 |
| Idle 4,096 | 13.999 | 13.0 / 17.7 | 189.939 | 750.000 | 0 / 0 |

This supports idle/wake overhead as a major loss in this short cell. It does
not select 4,096 as the default: at four participants and 65,536 / tile 64,
idle4096/candidate wall is 0.980 while diagnostic batch CPU is 1.261;
tile 256 is 0.985 wall and 1.203 CPU; tile 1,024 is 0.994 wall and 1.729 CPU.
Those cells still enter host waits after the longer spin. Increasing a fixed
window buys burst latency but spends CPU before long idle gaps. CPU comes
from one diagnostic process per mode, includes verification, and is quantized
in 15.625 ms increments; it is a reason to investigate, not repeated CPU
qualification. The default remains 256. Artifact `10105001028` has ZIP SHA-256
`59dd8807b83927faa6cd4a5a60c031744077258b1535ee0bc5cfa9aeeabd53c5`.

The four POSIX artifacts from the same run reinforce that the longer fixed
window is not a portable default. These are idle4096/candidate ratios; wall is
the median paired warm-core ratio, while CPU is the separate longer diagnostic
batch (one process per mode, including verification):

| Target | Participants | 4,096 / tile 1,024 wall / CPU | 65,536 / tile 64 wall / CPU |
| --- | ---: | ---: | ---: |
| Linux x64 | 4 | 0.635 / 0.941 | 1.186 / 1.415 |
| Linux ARM64 | 4 | 0.772 / 0.948 | 1.145 / 1.237 |
| macOS x64 | 4 | 0.746 / 1.594 | 1.019 / 1.117 |
| macOS ARM64 | 2 | 0.621 / 1.599 | 1.059 / 1.163 |

The small Linux cell's voluntary context switches fall from 8,769 to 27 on
x64 and 9,369 to 22 on ARM64. In the large tile-64 cell they remain similar
(1,606/1,588 and 2,119/2,192), despite longer spinning. These process-wide
counts support the idle-gap explanation without equating a context switch to
a particular runtime park. Darwin reports zero voluntary switches in these
samples, which is not evidence that no wait occurred. Other tiles also retain
CPU increases. Every extracted file matched its artifact manifest; ZIP hashes
are:

| Artifact | ZIP SHA-256 |
| --- | --- |
| `10104829158` (Linux x64) | `e309f48d11a2dc5bddea69eca754d0ba3ec66ee51c91941b6e40a84743eb8f2f` |
| `10104872709` (Linux ARM64) | `c6f3b1c811b5d37c003d178db649f81b6299203625652298b67291784d619b6a` |
| `10104910285` (macOS x64) | `a5501261e72fb2b20921f57c17cc8ffcbfbe37217d2dbea214cc640822b1935f` |
| `10105005239` (macOS ARM64) | `0a3b14bddb39736a3105c921ad31b4e32823bde4165fa6e497437aa63c80105c` |

A local four-participant M1 full-link/core-only comparison used the same scalar
WF object, inputs 4,096/65,536, tiles 64/1,024, and five alternating process
pairs with 1,024/256 warm calls. Median wall ratios were 0.833, 1.224, 1.006
and 0.963 respectively, with broad paired ranges (0.336–1.363 in the first
cell). This is unresolved local noise, not a selection ground for removing
the completion bridge or a substitute for normal CLI and mixed-program timing.

## Completion ordering candidate

The maintained core is testing release publication of DONE while retaining
the SC COMPLETING store, waiter observation/claim, and park registration and
recheck. Both the core's in-place idle recheck and the bridge's host-stack
fallback recheck now use SC. Their acquire-only forms did
not establish the following SC-order argument. No state, ABI, waiter ownership,
stack switch or I/O progress mechanism changes.

If the publisher misses a still-live registration, its SC waiter observation
precedes that registration in the SC order. The preceding COMPLETING store therefore
precedes the parker's subsequent SC state recheck, which cannot still observe
the old PENDING initialization. It sees COMPLETING or the later DONE and
avoids an unnotified sleep. If the publisher claims the registration, the
existing phase handshake owns its wake. A cancellation or replacement registration
does not inherit an old check: every new registration has its own SC recheck,
including after a failed publisher claim. DONE remains the final record access;
its release store and the joiner's acquiring read publish the result before
the joiner may release the frame. This uses the mixed SC/non-SC load rules in
[C11 draft N1570, 7.17.3 paragraph 6](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).
Deleting COMPLETING instead is unsound: a publisher could see NULL, a parker
could register and read PENDING, and only then would DONE be stored with no
claimed waiter to wake.

The experimental criteria are unchanged protocol-enumerator and native
concurrency checks, code generation on all supported targets, and measured
task/FIR costs against the same current source with SC DONE. Enumeration
checks SC histories, not weak-memory executions; the ordering argument and
independent review remain necessary. A removed x86 locked instruction is a
cost hypothesis, not a measured end-to-end improvement. This candidate is
separate from the idle-window CI control and is not performance-qualified.

Focused M1 checks passed the native smoke, 200,000-task deque probe, its
ThreadSanitizer build, and all four SC enumeration configurations (the largest
visited 44,819,639 states). The completion default-route probe and its
ThreadSanitizer build also passed with the full bridge and localhost TCP.
The existing protocol-cost benchmark now also
links the maintained shared scheduler directly, using the same held-helper
setup, output oracle and zero-additional-steals assertion as its recovered
baseline. `check-protocol-cost` passed all three images at widths two/four and
depths one/eight/32. Only the recovered image is sanitizer-instrumented in that
target; the shared deque has its own ThreadSanitizer check. The existing
`protocol-cost-calibrate` target still calibrates the recovered runtime only.

A separate M1 owner-local comparison used five alternating process pairs,
four participants with three helpers held, 100 ms warmup, and eight warm
samples of 262,144 tasks each. Candidate/SC-before median task-time ratios
were 0.9994, 1.0020 and 0.9986 at depths one/eight/32, with all paired ratios
between 0.9930 and 1.0219. There is no demonstrated M1 gain. Cross-target
compilation of the actual core shows identical completion-function assembly
on both ARM targets, and final DONE publication changes from `xchg` to `mov`
on all three x64 targets. The Apple ARM in-place idle recheck changes from
`ldapr` to `ldar`. Cross-compilation is instruction inspection, not native
platform qualification.

The next five-target screen replaces the zero-help attribution control with
`previous`, frozen maintained scheduler/floor sources at `6060cc67`. Zero-help
already isolated the join-help question and does not isolate this memory-order
change. Both the original before and recovered controls remain, as does
idle4096; no performance threshold is relaxed. Every image uses the same WF
object. On Windows every historical scheduler is compiled with its matching
headers and an identical current host/completion overlay, including the SC
bridge recheck. Thus previous isolates the core change, not a difference in
Windows bridge source. All source snapshots, flags and binary hashes travel
with the artifact. Native results are recorded below; performance remains
unqualified.

The local M1 script run completed all oracle, normal-CLI, width and diagnostic
checks and returned the performance-failure status. Against previous, 22 of
24 median ratios were within the screen band; two participants at 65,536 /
tile 16 measured 1.0979 (range 1.0790–1.1183), and four participants at 4,096 /
tile 256 measured 1.0964 (0.9806–1.1570). These losses remain unresolved;
neither the protocol microbenchmark nor cross-target instruction inspection
overrides actual workload measurements. This is local evidence, not CI
acceptance.

Independent review of this delta covered the completion ordering, bridge
fallback, maintained-runtime protocol host, comparison provenance and recorded
measurements. It found no remaining blocking issue within that scope; the
diagnostic metadata now explicitly distinguishes before (reports disabled)
from previous/current controls (race-free reports enabled). This scoped review
does not certify the whole PR or the outstanding performance goal.

### Same-image variability control

The [five native screens at `c8384799`](https://github.com/mbbill/Whitefoot/actions/runs/34355813654)
completed their oracle, CLI, width and diagnostic checks; all five retain
performance failures. Its [12-job gate](https://github.com/mbbill/Whitefoot/actions/runs/34355813469)
and [Linux/Windows I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34355813476)
passed. The Windows EPYC 7763 screen has candidate/previous paired wall median
0.9250 (range 0.8780–0.9739) at four participants, 4,096 / tile 256, but its
separate long diagnostic instead records 51.185/37.521 us warm means. At two
participants, 4,096 / tile 1,024, the paired median is 1.2808
(0.7627–1.6418); its long diagnostic is also slower, 32.616/22.920 us.
These opposing observations do not qualify an overall speedup. Windows
artifact `10105858169` has ZIP SHA-256
`da4fe392b6c5c5e00033a3eb9273c9bd3a8e063cce3753ab7fec42360d8ab780`;
all extracted file hashes matched its manifest.

The Linux and Windows results retain large paired ranges,
including in one-participant cells. Reducing a locked instruction has not yet
established an end-to-end improvement. The next screen adds `replica`, a
byte-for-byte copy of the candidate executable invoked independently with the
same input, width and sample count. The script verifies binary identity; its
raw runtime label remains candidate. Candidate and replica are adjacent in
each pass and reverse order together. Their median wall ratio must lie within
`[1/1.05, 1.05]`; both faster and slower discrepancies flag investigation.
Existing before/recovered/previous/idle comparisons and thresholds remain.

This A/A control measures variability without a source or code-generation
difference. An adverse A/A result cannot excuse a candidate loss or pass a
platform: it says the measurement conditions need further work before a small
effect can be attributed. The same raw first/warm samples are retained. This
addition does not yet provide repeated CPU diagnostics or normal CLI timing.
Shell syntax and whitespace checks pass. The actual extracted summary program
accepts A/A medians 1.00, 0.98 and 1.03, rejects 1.08 and 0.94, and rejects a
missing replica. Independent review found no issue in the binary-copy order,
invocation labels, symmetric band or artifact coverage.

The [five-target A/A run at `5e3cbc24`](https://github.com/mbbill/Whitefoot/actions/runs/34357325383)
completed and failed every platform's performance screen. Replica comparisons
outside the symmetric band were Windows 2/24, Linux x86-64 11/24, Linux
AArch64 6/24, macOS x86-64 4/24 and macOS AArch64 3/16. Linux x86-64's
one-participant 4,096 / tile64 median was 1.3323 (range 0.7952–1.3897),
despite identical executable bytes. These short cohorts cannot reliably select
small runtime effects. They do not invalidate the recorded losses or grant
acceptance. That revision passed its local canonical `make check`, its
[12-job gate](https://github.com/mbbill/Whitefoot/actions/runs/34357325212), and
[Linux/Windows I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34357325252).

The next screen lengthens each independent process to 4,096 warm calls for
4,096 outputs and 512 calls for 65,536 outputs. Both the full batch and its
first64 prefix retain the same verdict criteria; either can fail. The prefix
is an overlapping view, not extra independent samples or an exact repeat of
the former 64-call process cohort. Raw first calls remain available. Five
processes remain the independent samples; the longer batch does not create
thousands of independent observations. CPU diagnostics remain separate and
normal CLI timing remains open.

### Coalescing condition-wait notifications

The maintained completion runtime and fallback host primitives now re-arm a
wake-needed flag when a waiter announces. Every publication still advances the
SC epoch. On the condition-variable-only path, the first notification for the
announced set takes the wait lock and signals; later notifications can skip
that lock until a new announcement. The flag/epoch SC pair closes the no-lock
missed-wake race. External callbacks retain the original per-publication
behavior: the flag stays set while an external waiter remains. This is one
shared runtime and its existing routes, not link-time runtime selection.

The candidate targets a measured cost: the Windows `c8384799` long
4-participant 65,536 / tile16 diagnostic counted 102,543 notification signals
for 1,551 announcements. That compute-only bridge uses condition variables;
signals are requests, not awakened threads or IOCP posts. Performance selection
requires lower notification/CPU cost without stable wall-time regressions;
results remain pending on all five targets. The previous control is frozen at
`5e3cbc24`, including its Windows bridge.

Review prevented extending coalescing to external I/O waits. A new IOCP park
can consume an older waiter's packet. Retaining and re-posting that packet
until all announcements withdraw would preserve the token but could keep the
queue permanently nonempty at the port's concurrency limit, preventing older
waiters from running. This follows the documented [IOCP concurrency behavior](https://learn.microsoft.com/en-us/windows/win32/fileio/i-o-completion-ports).
That proposed repair was removed before publication. Existing IOCP token
ownership/progress remains an unresolved correctness question; passing prior
I/O tests does not settle it. This round preserves the old external notifier
and token-consumption behavior instead of introducing an unqualified I/O change.

The harness checks coalescing, rearming, cancelled announcements, and unchanged
external notification counts. Existing real-thread wake tests now synchronize
past the final epoch recheck before requiring a wake result. The final narrowed
macOS harness passes at helper counts 0/1/4 and in no-cache mode; its bridge
ThreadSanitizer check also passes. Scheduler/deque tests passed before the
external-callback narrowing, which does not change those primitives. Final
native CI, canonical and all-platform performance acceptance remain required.

### Wake-coalescing measurements at 536abedd

The [five native screens](https://github.com/mbbill/Whitefoot/actions/runs/34360325388)
completed and all retain performance failures. Linux/Windows I/O checks passed;
the gate completed eleven jobs successfully but Linux unit was cancelled,
without a test failure reported in its log. Exact-revision local root
`make check` passed, including compiler, research, conformance and snapshot
checks. No platform is performance-qualified by these results.

For long batches at four participants and 4,096 outputs, candidate/previous
paired wall medians for tile16/64/256/1024 were respectively
0.8518/0.8355/0.9093/1.1362 on Linux x86-64 and
0.9106/0.9371/0.9174/0.8739 on Linux AArch64. The x86-64 coarse cell was slower
in all five pairs (range 1.1165–1.2346). At two participants, that same coarse
cell regressed on both Linux targets: medians 1.2407 and 1.2107, with minima
1.1450 and 1.1912. These losses prevent selecting coalescing as a portable win.

The Linux x86-64 diagnostic at two participants / 4,096 / tile1024 recorded
1,085 voluntary switches for the candidate versus 16 for previous; batch
user+system CPU was 313,576 versus 303,222 us. At four participants those
switch counts were 7,066 versus 3,332, CPU 693,577 versus 629,609 us. These
are separate diagnostic batches, not causal estimates or five extra samples.
Core stack-park counts differ from host waits and must not be substituted for
them. Artifact `10107962650` has ZIP SHA-256
`bc651e07c570e71fbdcd44f61af6ccbc983d514d143edf0d0bdf0153d2176e51`;
all 899 extracted manifest entries matched. The next attribution question is
why fewer notification opportunities coincide with more host waiting in these
coarse cells, while fine-grained cells improve.

A local M1 run of the same runtime source bytes (captured as a dirty tree over
5e3cbc24 before publication) also retained losses: four-participant small-cell
candidate/previous medians were 0.7852/0.7286/0.7381/0.8207, but corresponding
candidate/recovered medians remained 2.0369/1.4395/1.8415/1.6178. This is local
exploration, not another CI platform pass.

### Maintained-runtime quadrature coverage

The existing quadrature program/oracle now also executes on the maintained
scheduler, with the comparison protocol owned by the
[experiment](../../experiments/compute-runtime/README.md#adaptive-recursive-quadrature).
This adds recursive/skewed/depth-limited computations to the formal-runtime
comparison. Same-object attribution remains distinct from normal CLI delivery:
the ordinary CLI command is checked, but its timing remains unfinished.

Local full quadrature rebuilding succeeded. Its checks passed after granting
native CPU-topology access: 160 new formal/recovered processes plus all the
existing quadrature, sanitizer, exhaustion and batch-protocol checks. Review
caught and corrected nested source snapshots on repeated builds and invalid
caller-thread CPU subtraction across formal stack migration. The first
[native CI attempt at d5cd68b8](https://github.com/mbbill/Whitefoot/actions/runs/34363309499/job/102505558200)
stopped before calibration: the existing sanitizer validator rejected a
384-byte Rayon pool-build allocation report. That check remains enforced;
the run supplies no formal quadrature performance result. Windows, stronger
scaling and end-to-end delivery qualification remain open.

### Eliding obsolete wait announcements

The next maintained-runtime candidate checks the wake epoch before taking the
host wait lock. Fallback POSIX and Windows primitives also recheck under the
lock before announcing. Completion already had that locked check. A changed
epoch returns to the scheduler's work scan; it neither consumes a notification
nor clears the wake-needed flag. The existing post-announcement SC checks
remain the lost-wake protection. Ordinary calls, current-stack joins, public
ABI and I/O routing are unchanged. The frozen previous control advances to
`d5cd68b8` to isolate these early returns from the preceding coalescing change.

This targets avoidable mutex and announcement traffic, not the cost of a
necessary kernel sleep. An ephemeral local M1 million-call stale-epoch probe
measured 9.06–16.27 ns/call before and 2.17–5.79 ns/call after across five
sequential pairs. Order/frequency effects are visible; this is path-cost
evidence only, not an application speedup or cross-platform qualification.
Selection still requires the same application-level wall and CPU criteria,
including the coarse Linux regressions and unchanged I/O progress checks.

Scoped independent review found no blocking defect in the early-return
handshake. The modified tree passed the maintained completion harness with
0/1/4 helpers and no-cache mode, the default-route ThreadSanitizer probe,
scheduler smoke and Windows GNU cross-compilation. The published `f6e71c6a`
also passed exact-revision root `make check`. These correctness checks do not
qualify a platform's performance.

The local M1 screen completed all 720 process means and 72 diagnostic batches
but failed its performance criteria. At 4,096 outputs / tile1024, two/four
participant candidate/previous paired medians were 0.9309/0.9268, with every
pair below one. At 65,536 / tile16 they were 1.0580/1.3244; the four-participant
range was 0.9762–1.5283. Short-prefix views also retained failures. These are
the source bytes later published as f6e71c6a, captured while dirty over
d5cd68b8, not a clean-revision timing claim.

The [f6e71c6a Linux x86-64 screen](https://github.com/mbbill/Whitefoot/actions/runs/34364977223/job/102511242442)
did not reproduce that large/fine local regression: its four-participant
candidate/previous median was 0.9919 (0.9848–1.0121). However, small/coarse
candidate/recovered medians remained 1.4821 and 1.8206 at two/four participants.
The four-participant small/coarse A/A median was itself 1.0510. No platform is
accepted on the basis of this mixed evidence.

A local layout control has been built separately. The wait edit moved
entry/WF/native functions by 24 bytes in the original pair, while the core
functions stayed in place. Linking the changed primitive last with a common
Darwin order file holds every other text-symbol address fixed across that
pair; writable-data addresses also match, but a floor constant still moves.
Common host and computation objects are reused. This diagnoses a possible
layout confound; it neither changes the production linker nor erases the
original regressions.

### Equal-epoch notifications must rearm the next wait

The layout attribution did not finish: the run completed 67 process rows before
a f6e71c6a four-participant 4,096 / tile1024 process stalled. A native
sample showed all four threads in the scheduler loop's condition wait;
105 seconds of elapsed time had consumed only 0.33 seconds of CPU. Its partial
timings are not a completed layout experiment. The process was sampled and
then terminated; debugger attachment did not complete.

Independent review confirmed a legal missed-wake execution in notification
coalescing, predating the early-return optimization. A publisher can advance
the epoch before a new waiter captures it, but acquire the wait lock only
after that waiter sleeps. Its delayed broadcast clears wake-needed and wakes
the new waiter with an unchanged epoch. The old loop sleeps again without
rearming; the next publication can then skip its required signal.

Both POSIX/Windows fallback primitives and the shared completion condition
loop now rearm with SC ordering, then recheck the epoch with SC ordering,
before every repeated wait. Registration counts still describe park calls,
not each sleep attempt; timeout/error handling and external I/O routes remain.
A harness-only observer on return from the real condition wait lets the
regression deliver the delayed notifier's locked reset/broadcast tail, wait
for the actual return/re-sleep, then send a real notification. Resetting the
observer under the wait lock also excludes earlier spurious returns.

The final regression fails against the frozen f6e71c6a completion unit and
passes against the repair with otherwise matching test objects. The combined
working tree passes helper counts 0/1/4 and no-cache mode, and the completion
TSan harness passes helper counts 0/1/4. Both repaired Windows units compile
with the Windows GNU cross-toolchain; native MSVC execution remains required.
Separately, the
original frozen f6 compute/host objects with only the POSIX primitive repaired
completed 100 fresh processes of the stalled case, each verifying 4,097 calls.
All other text-symbol addresses match the stalled image. This supplies native
weak-primitive progress evidence as well as the deterministic completion test;
it is not performance qualification or conclusive attribution of every hang.

The [f6 quadrature job](https://github.com/mbbill/Whitefoot/actions/runs/34364977223/job/102511242571)
also stopped during formal calibration, at pass4 / four participants /
center-peak / wf-leaf, after its earlier correctness checks passed. Its last
summary row was written at 14:47 UTC; cancellation was at 14:59 UTC with a
formal process still present. Artifact `10110358723` ZIP SHA-256 is
`cb08a0b039e0ea9c3031d9d1f3d2ea45cc49fdb7889e5dcb3d8676be38148961`.
Linux AArch64's formal screen was also cancelled while comparing runtimes.
Those are incomplete measurements, not performance passes.

The timing reference returns to pre-coalescing `5e3cbc24`. Keeping the known
stalling f6/d5 implementations in every timing loop would prevent completing
the matrix and would not establish a qualified reference. Their evidence is
retained, and the old f6 completion unit now fails the new regression; no
acceptance threshold or workload was relaxed. Exact f2d9d0fa subsequently passes
local canonical `make check`: compiler 581 seconds, research 210, conformance
99 and snapshot 21, ending `WHITEFOOT ALL TESTS GREEN`. Linux and Windows native
I/O host checks pass. The Linux research CI gate still fails on the previously
seen Rayon caller-worker 384-byte LSan report; its validator expects the
associated indirect queue as well. The direct allocation matches the known
`use_current_thread` lifecycle, but why that queue is absent from this report
has not been established. No sanitizer criterion is relaxed.

The [f2d9d0fa compute run](https://github.com/mbbill/Whitefoot/actions/runs/34369584492)
completes all five native FIR screens, with performance
failures rather than stalled timing loops. Long-batch paired medians below
are candidate/reference; `previous` is pre-coalescing 5e3cbc24. Each ratio
uses five fresh-process pairs on its own host, not cross-host timings.

| Host | Workers | N / tile | Previous | Recovered | Identical replica |
|---|---:|---|---:|---:|---:|
| Linux x64 | 4 | 4096 / 1024 | 0.9039 | 1.4356 | 0.9925 |
| Linux x64 | 4 | 65536 / 16 | 0.9763 | 1.1428 | 0.9846 |
| Linux ARM64 | 4 | 4096 / 1024 | 0.8251 | 1.6742 | 1.0095 |
| Linux ARM64 | 4 | 65536 / 16 | 1.0023 | 1.2143 | 1.0212 |
| macOS ARM64 | 2 | 4096 / 1024 | 0.9274 | 1.6086 | 0.9944 |

All five coarse-cell pairs lose to recovered on each listed host. Long-batch
A/A has two cells outside its band on Linux x64, zero on Linux ARM64, and
five of sixteen on macOS ARM64; the latter does not qualify small deltas.
The same run's quadrature panel completes all 2,400 processes with 256 repeats
and its data verifier passes. Performance does not: center-peak / wf-leaf /
four workers has formal/recovered wall 1.2726 and CPU 1.4620. Successful data
collection after the wait repair is not performance acceptance.

An independent delivery review also confirms that these attribution images
do not close ordinary CLI timing. FIR and quadrature commands are tiny smoke
cases; records' command is manually linked and Mandelbrot lacks that normal
CLI panel. Substantial input generation and repetition live in C drivers.
Normal linking uses O2; attribution uses O3 with auto-vectorization disabled.
A general scalar control must also suppress the maintained lowering's explicit
wide byte probes; Clang vectorizer flags alone cannot remove those vectors.
These remain delivery gaps, not grounds for applying research timings to the
ordinary executable.

Windows also completes with a failed performance screen. Its four-worker
4,096 / tile1024 candidate/previous median is 0.8898, but candidate/identical
replica is 0.7542 (range 0.6036-1.3159), so the apparent improvement is not
qualified. Four of 24 long-batch A/A cells are outside the band. The research
recovered control still has no qualified Windows port; these ratios are
against the maintained-runtime control, not evidence of parity with recovered.
macOS x64 is also noisy: nine of 24 long-batch A/A cells exceed the band.
At four workers, 4,096 / tile1024 has candidate/previous 1.0174,
candidate/recovered 1.3533 and candidate/replica 1.3368; this does not qualify
a precise implementation delta. No native platform meets full acceptance.

### Completing an owned join target locally

The M1 large/fine diagnostic executed approximately 2.09 million tasks in
the owner's inline-join branch versus about 12,000 steals. Every inline task
still paid the generic completion handshake. The next maintained-core
candidate removes that handshake only after the owner successfully pops its
own join target. It calls the body and release-publishes DONE directly.
Generic helper/steal execution and I/O completion keep the full handshake.
The local comparison uses f2d9d0fa as its reference and the same repaired
wait primitive in both images; f6's defective wait is not a timing control.

The premise is the existing unique live joining continuation: the compiler
keeps the handle private and emits join, result read, then release. That
continuation is the direct caller, so it cannot simultaneously be parked on
this record. Nested calls or I/O may migrate the whole stack; their waits
name their own records. This does not admit concurrent joins on a shared
future, change public ABI, change callback execution or select a runtime.

The enumerator permits the direct PENDING-to-DONE edge only for the active
same-stack compute join, after its callback-return witness, with no waiter.
Both witnesses are checkpointed with the logical stack. It still requires
COMPLETING for I/O and generic completions. All four full sweeps pass;
the two-thread/four-stack S5 sweep observes 119 owner-DONE transitions after
migration, not 119 independent tasks. A negative case rejects DONE before
callback return. Native smoke holds thieves, defers its device completion
until the inner stack is SUSPENDED, then verifies one inline execution, one
park/resume, the result and a repeated join. It passes, as do the six Rust
scheduler tests, bridge ThreadSanitizer and Windows GNU core cross-compilation.
These focused checks preceded the waiter repair; the combined-tree completion
checks are recorded above. Exact 98c283cb subsequently passes local canonical
`make check` and all twelve gate CI jobs; its native performance results below
do not qualify the candidate. Selection requires application and CPU improvement without
losing these ownership and progress properties, not merely fewer instructions.

Two local FIR cohorts each complete 90 fresh processes: one/two/four workers,
4,096 / tile1024 and 65,536 / tile16, five alternating passes, fixed/inline/
identical-replica images. Every output is checked. They run after the exact
f2 canonical process exits, with common computation/host/primitive objects.
In the ordinary link layout, long compute ratios are near parity (two-worker
fine 0.9777; four-worker fine 0.9971), but whole-batch CPU rises 9-15%, even in
the sequential world that cannot execute the changed join branch. The one-worker
large case's core remains about 535-538 us while its full cycle grows from
about 1,598 to 1,818 us. The CPU observation includes verification; it cannot
be attributed solely to the scheduler.

The join edit shrinks text by 28 bytes and shifts later functions. A separate
Darwin order-file control places join last and holds every other text-symbol
address fixed. Under that layout, the extra batch CPU cost disappears. For
two-worker fine work, all five core and CPU pairs improve: medians 0.9662 and
0.9809; identical-image core ratio is 1.0113. Four-worker fine medians are
0.9937 core and 0.9974 CPU. Short coarse work still loses: two-worker first64
core ratio 1.0843, range 1.0570-1.1886. These are overlapping short views of
the same processes, not independent cohorts. The local layout experiment
diagnoses the confound; it neither changes the production linker nor qualifies
the candidate's overall application performance.

Two quadrature cohorts then complete 240 processes each, with 4,096 checked
repeats plus eight warmups: fixed/inline/replica, four inputs, one/four workers,
leaf and sequential-kernel forms, five alternating passes. Both use common
f2 gate computation/host/primitive objects; the second holds all non-join
text-symbol addresses fixed. Its candidate/fixed ratios are:

| Input | One-worker leaf wall / CPU | Four-worker leaf wall / CPU |
|---|---|---|
| Center peak | 0.9410 / 0.9412 | 0.9577 / 0.9489 |
| Left peak | 0.9421 / 0.9421 | 0.9694 / 0.9704 |
| Right peak | 0.9335 / 0.9335 | 0.9885 / 0.9941 |
| Depth cap | 0.9938 / 0.9934 | 0.9341 / 0.9279 |

All five one-worker peaked-input pairs improve in both layouts; their fixed
layout A/A wall medians are 0.9992-1.0033. This one-worker leaf form explicitly
executes the outlined task path on the core without helper threads; it is not
the ordinary CLI's automatic sequential-world selection. Separate sequential
kernel controls remain near parity. Four-worker medians also improve, but
individual wall/CPU pairs lose and A/A ranges are wide; no per-cell pass is
claimed from those medians. In the first layout, four-worker center-peak wall
is 0.9283 and depth-cap is 0.9462, showing why layout-conditioned results must
remain separate rather than pooled.

The repeated local-task benefit justifies native CI evaluation of this small
maintained-core shortcut; it does not resolve the FIR short-batch losses or
the much larger recovered-runtime gaps. The next five-target screen pins
`previous` to repaired f2d9d0fa to isolate the shortcut. Workloads, thresholds
and the recovered/identical-image controls remain unchanged. Full application
qualification and ordinary CLI timing remain open.

The combined-tree `whitefootc` binary is rebuilt through Cargo's gate profile.
Its normal `--par ... -o ...` FIR and quadrature executables pass at one/four
workers. This confirms current-source CLI integration and correctness, not
ordinary CLI performance qualification.

### Native result for 98c283cb: not qualified

The [five-target run](https://github.com/mbbill/Whitefoot/actions/runs/34373190141)
completes every native FIR screen and the 2,400-process quadrature panel.
All five FIR screens fail performance acceptance. Ordinary CLI FIR correctness,
native deque checks, the [gate](https://github.com/mbbill/Whitefoot/actions/runs/34373190119),
[I/O host checks](https://github.com/mbbill/Whitefoot/actions/runs/34373190112)
and [I/O benchmark checks](https://github.com/mbbill/Whitefoot/actions/runs/34373190123)
pass. The prior intermittent Rayon sanitizer report does not recur in this gate;
that is not a demonstrated lifecycle fix.

Selected long-batch candidate / f2d9d0fa paired wall medians follow. Ratios below
one favor the candidate. Each cell retains five independent processes per image;
the two workloads are coarse 4,096 / tile1024 and fine 65,536 / tile16.

| Native host | Coarse, 2 workers | Coarse, 4 workers | Fine, 2 workers | Fine, 4 workers | Long A/A cells outside band |
|---|---:|---:|---:|---:|---:|
| Linux x64 | 1.1646 | 1.2683 | 0.9562 | 0.9848 | 3 / 24 |
| Linux ARM64 | 0.9997 | 0.9968 | 0.9783 | 0.9770 | 0 / 24 |
| macOS ARM64 | 0.9292 | Not run: two CPUs | 1.0867 | Not run: two CPUs | 7 / 16 |
| macOS x64 | 0.9360 | 0.9909 | 0.9870 | 0.9501 | 4 / 24 |
| Windows x64 MSVC | 0.9646 | 1.1174 | 0.9632 | 1.0206 | 0 / 24 |

Linux x64 coarse work regresses in every pair: two-worker range
1.1516-1.2769 and four-worker 1.1624-1.2994, with corresponding A/A medians
0.9867 and 1.0021. Windows four-worker coarse work also needs investigation
(range 0.8992-1.1197). Mac gains or losses cannot be selected through the
substantial identical-image variation. Linux ARM64 avoids the new coarse
regression but remains 1.6038 times the recovered runtime at four workers;
Linux x64 is 1.8132 times that control. Matching the preceding maintained
revision is not matching the recovered baseline.

Quadrature still loses against the recovered runtime. Center-peak leaf/four
wall and CPU medians are 1.3692 and 1.5054; depth-cap leaf/four is 1.3993 and
1.4460. These ratios are from this host's own controls, not a cross-run comparison
of absolute times against f2's different CI host.

The Linux x64 [artifact](https://github.com/mbbill/Whitefoot/actions/runs/34373190141/artifacts/10113055288)
has ZIP SHA256 `3450c6d70a9462f99a9fd1306bed5117f305fdd10556ce3dec3b8fbf53038848`.
Its disassembly keeps the join prefix and stack-frame size unchanged, removes
the owner's generic completion tail call and shrinks join by 17 bytes. Later
text, including both FIR computation functions, moves by 16 bytes. The separate
four-worker coarse diagnostic reports candidate/previous inline counts
3,976/2,201, steals 8,315/10,090 and parks 15/28; both execute 12,291 tasks.
These instrumented, single-process counts are not paired timing evidence and
do not establish why the ordinary five-process cohort regresses. Fewer parks
alone does not explain or excuse the loss.

The next Linux-only diagnostic places join after the other executable sections
through a declaration and linker script, compiling the unchanged maintained
sources. Before timing, it requires identical addresses for every other text
symbol and a byte-identical candidate replica. Its separate 90-process cohort
covers one/two/four workers, the coarse/fine cells above and five alternating
passes. Symbol sizes, ELF maps and disassembly preserve possible instruction or
data-placement differences: matching function starts alone does not eliminate
all binary-layout effects. Ordinary placement remains measured with unchanged
acceptance criteria. This control tests the following-function address
explanation; it does not change the production
linker or repair the regression. Remove it once that causal question is resolved.

### Scalar builds through the ordinary compiler

`whitefootc --no-vectorize` now supplies the missing general scalar build option
in the maintained compiler. It suppresses explicit WF byte probes after semantic
checking; normal native linking and stack-ledger generation also pass Clang's
loop/SLP disabling flags. LLVM-only consumers must pass those host flags when
they subsequently compile the IR. Default optimization remains `-O2`, and the
default vectorization setting remains enabled. This is not a promise that
platform library internals contain no SIMD.

The option passes all-target Cargo checking, Clippy with warnings denied, and
four focused tests: scalar lowering retains the ordinary loop, invalid proofs
retain their diagnostics, CLI mode/ledger selection is independent, and the
actual native CLI executes a boundary-sensitive byte walk with both ledgers
enabled. Scoped independent review found one misleading LLVM-only documentation
sentence, now corrected; no remaining compiler-option finding. New exact-tree
canonical and cross-platform execution remain pending. FIR's five native CI
commands now request this option; their host-driven attribution objects stay
at the separately recorded `-O3` setting. Full ordinary CLI timing and broader
native workload coverage remain open.

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
