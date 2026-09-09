# Historical and recovered pure-compute runtimes

## Scope and conclusion

This report compares the last main POSIX compute runtime before shared
compute/I/O scheduling with the recovered research runtime. The owner requested
the comparison on 2026-09-09 and will choose a direction after reading it.
Production-runtime integration is paused. The current unified implementation
is retained in the [restorable checkpoint](../io-model/UNIFIED-RUNTIME-CHECKPOINT.md).

The two compute candidates share the same central scheduler. Research is a
recovery with safety fixes and experimental instrumentation, not a newly
invented faster scheduling architecture. A sensible future base would preserve
their common current-stack design, bring forward the necessary correctness
repairs, and remove experimental controls from the production core. This is a
recommendation for owner discussion, not authorization to start integration.
Native CI does not establish one version as the overall winner. Most coarse
compute cells are close, but historical code has a meaningful fine-grained FIR
advantage on Linux x86-64. macOS noise prevents firm ranking there. It would be
incorrect to assume the research file is the faster version simply because it
was used in earlier first-tier experiments.

## Exact comparison inputs

| Input | Frozen source | Role |
| --- | --- | --- |
| Historical main POSIX | `9051576f6a4d723b4eb072850f49859853decae7:compiler/src/backend/par_runtime.c` | Last version before `92b19e1` replaced it with the shared scheduler |
| Historical main Windows | Same revision, `compiler/src/backend/par_runtime_windows.c` | Portability/code review; not interchangeable with the POSIX file |
| Recovered research | `d858008f560b25da896af2a17f8b1d07ac49fd6e:research/experiments/compute-runtime/runtime.c` | Existing POSIX-only recovery control |

The recovery cites `fee335654d9dea027f4636bbad448d57a4e84d08`. That commit is
not an ancestor of `9051576f`; do not invent a linear commit ancestry. Its
runtime blob is `75b1cb5206c0ef67e0f3804967e6c3eefd0a81b5`, also found at
`ea40acd1^:compiler/src/backend/par_runtime.c`. Compare the actual file bytes.

## Architecture and quality

| Aspect | Historical POSIX | Recovered research | Assessment |
| --- | --- | --- | --- |
| Task representation | 256-byte frames with 16-byte alignment, 64 owner-local slots/lane | Same | Small fixed footprint; refusal must retain required source semantics |
| Queue and local path | Owner deque; own newest task called directly by join | Same | No stack migration or per-task allocation on this path |
| Stolen-task join | Run local work, then steal on the current stack | Same | Preserves nested helping; stack depth can exceed sequential execution |
| Idle behavior | 4,096 searches, then 16 yields, then condition wait | Same | Cheap wake response costs idle CPU; neither is proven optimal for all workloads |
| Loop budget | Work floor 1,200,000; up to 16 chunks/lane | Same defaults | Compiler publication policy and runtime algorithm must be assessed separately |
| Ring pointer cells | Plain pointer accesses | Relaxed atomic accesses | Research repairs a delayed-thief/ring-reuse data race |
| Thief index ordering | Acquire loads | Acquire loads | Both frozen versions need an ordering repair before adoption |
| Startup | Owner wait station initialized after workers | Initialized before workers; failure cleanup | Research's sequencing is easier to justify |
| Statistics | Shared successful-steal counter always enabled | Optional counter, event hooks and budget experiments | Useful evidence controls; experimental policy branches do not belong in a clean production core |
| Completion integration | Additional help-once entry in this historical revision | Pure compute only | Neither file is the current unified scheduler |
| Lifecycle | Process-lifetime pool, fixed owner attachment | Same | Neither is a general restartable embedded executor |
| Source organization | 947 lines, substantial invariant/rationale comments | 659 lines, fewer comments and extra test conditionals | Shorter is not a simpler algorithm; retain useful invariant explanations and remove instrumentation clutter |

Their ordinary acquire/publish/join/read/release protocol is compatible with
the current emitted compute module used by this experiment. Historical
`wf__par_grants` is a data symbol whereas the research observer is a function;
the comparison explicitly adapts that observation interface. This does not
establish a public language ABI or qualify all separately compiled programs.
The current floor still supplies stack sizing/exhaustion handling to both.

The local path avoids a mutex when no worker is parked. Publishing still has
atomic ordering and an idle-set check; stealing uses a CAS, successful-steal
statistics can contend, and waking a parked lane takes its wait machinery.
"Mostly local" is justified; "no synchronization" or "zero overhead" is not.

## Correctness and measurement boundary

The [explicit comparison repair](../../experiments/compute-runtime/pure-compare-repairs.patch)
changes only scratch copies. Both receive SC thief top/bottom loads; the old
copy also receives atomic ring cells and read-only observer adapters. No
formal compiler or frozen research runtime is edited. Thus the timings compare
**old with repairs** and **research with repairs**, not unsafe original bytes.

The index-ordering concern is independent of pointer-cell atomicity. With only
acquire reads, a later thief can observe a new top with an older bottom after
an owner pop and another thief advance top. The owner and that later thief can
both believe they own the same task. The comparison uses the conservative SC
form already used by the maintained core; stress-test success alone is not a
C11 ownership proof. Startup-failure behavior still differs and is not covered
by these timing runs. This is not a complete new proof of either runtime.

Earlier first-tier results included compiler leaf/refusal/frontier decisions,
the choice of task size and sometimes disabled diagnostic counters. They do
not show that replacing the old runtime file alone achieves those gains.
The current experiment holds the emitted object and scalar arithmetic fixed
and compares counters-on first, then reports counters-off separately.

## Performance evidence

The [reproduction entry](../../experiments/compute-runtime/README.md#historical-versus-recovered-pure-compute-comparison)
defines the bounded FIR/Mandelbrot matrix, common correctness repairs, build
flags, worker validation and raw artifacts. Previous unified/recovered ratios
are not substituted for this two-pure-runtime question.

### Four-target native CI

At `4ff01e889f0b18a13adc03d1568c47206edfcbea`, all four
[pure-comparison jobs](https://github.com/mbbill/Whitefoot/actions/runs/34407502237)
passed their build, output, participation and complete-matrix checks: 4,400
processes in total. This is a result of these four jobs, not a claim that the
whole workflow or canonical gate passed. Every artifact ZIP hash matched GitHub's
digest, all 43 source/object/compiler/binary hashes per artifact matched, and
recomputing each full summary from raw process data reproduced it exactly.

| Target | Exposed CPU / compiler | Participants | Processes | Complete artifact |
| --- | --- | --- | ---: | --- |
| Linux x86-64 | EPYC 7763; 2 cores / 4 SMT threads; Clang 18.1.3 | 1, 2, 4 | 1,200 | [raw/source/binaries](https://github.com/mbbill/Whitefoot/actions/runs/34407502237/artifacts/10126018792) |
| Linux AArch64 | Neoverse-N2; 4 cores; Clang 18.1.3 | 1, 2, 4 | 1,200 | [raw/source/binaries](https://github.com/mbbill/Whitefoot/actions/runs/34407502237/artifacts/10125985573) |
| macOS AArch64 | Virtual M1; 3 CPUs; Clang 15.0.0 | 1, 2 | 800 | [raw/source/binaries](https://github.com/mbbill/Whitefoot/actions/runs/34407502237/artifacts/10126016588) |
| macOS x86-64 | i7-8700B; 4 exposed CPUs; Clang 17.0.0 | 1, 2, 4 | 1,200 | [raw/source/binaries](https://github.com/mbbill/Whitefoot/actions/runs/34407502237/artifacts/10126140243) |

The [complete 660-row process comparison table](pure-comparison-4ff01e88.tsv)
retains every case, width and contrast, including losses. It reproduces the four
artifact summaries with a target column added. Retain this measured result
while the report cites it; it is evidence, not an implementation or acceptance
threshold. Machines differ, so compare old/research within a row, not absolute
speed between architectures.

FIR warm core: medians of five paired process medians; 16 taps, tile 64.
Above one favors research. Bounds are observed pair ranges, not confidence
intervals. A/A is the byte-identical research replica divided by research.

| Target; participants | Outputs | Old/research | Pair range | A/A range |
| --- | ---: | ---: | --- | --- |
| Linux x86-64; 4 | 4,096 | 1.0066 | 0.9848–1.0234 | 0.9983–1.0218 |
| Linux x86-64; 4 | 65,536 | 0.9821 | 0.9485–1.0052 | 0.9913–1.0011 |
| Linux AArch64; 4 | 4,096 | 0.9035 | 0.8911–1.1240 | 0.9444–1.1510 |
| Linux AArch64; 4 | 65,536 | 0.9963 | 0.9513–1.0102 | 0.9575–1.0159 |
| macOS AArch64; 2 | 4,096 | 0.9926 | 0.8088–1.1566 | 0.8359–1.2063 |
| macOS AArch64; 2 | 65,536 | 1.0014 | 0.9632–1.5594 | 0.9848–1.2287 |
| macOS x86-64; 4 | 4,096 | 0.8901 | 0.5211–1.2726 | 0.8669–1.8624 |
| macOS x86-64; 4 | 65,536 | 1.1357 | 1.0002–2.0671 | 0.9988–1.5062 |

There is a real exception to a blanket "same performance" conclusion. Linux
x86-64 at 4,096 outputs, **tile 16**, four participants gives old/research core
0.7928 [0.6749, 0.8575], process wall 0.8125 [0.7746, 0.8775], CPU 0.8113,
RSS 0.9876, and 1,645 fewer process context switches (median paired delta).
All five pairs favor old. A/A core ranges 0.8526–1.1190 and process wall
0.9454–1.0558: the host is not perfectly stable, but the old advantage is large
enough to preserve as an unresolved performance difference. No causal runtime
change was selected from this single cohort.

The same Linux machine's 4,096/tile-64 case has nearly equal core time but
old/research process wall 0.9142 [0.8793, 0.9479] and CPU 0.8539. The batch
interval, which excludes process launch, also favors old (0.9108); the warm
full-call cycle ratio is 0.8714. Startup alone therefore does not explain it.
Do not erase this result by reporting only the core interval. Other intervals
include preparation, allocations/copies, cleanup and checking, and interact
with background workers and executable layout.

Read-only object inspection adds a useful constraint: in the Linux x86-64
artifact, unrelocated instruction bytes for `wf__par_publish`, `wf__par_join`,
`wf__par_release`, `wf__par_worker_main` and `wf__par_split_budget` match between
old and research. Their positions and relocations in the final executables
are not identical, and startup differs. Thus the FIR difference is not evidence
that research introduced a different join/steal algorithm. Layout, startup
state and interaction with the host remain possible causes, not proven ones.

Mandelbrot whole-process examples at 65,536 points; 16 repetitions:

| Target; participants | Shape | Old/research wall median [range] | CPU ratio | RSS ratio | Context-switch delta |
| --- | --- | --- | ---: | ---: | ---: |
| Linux x86-64; 4 | Plane | 1.0064 [0.9899, 1.0490] | 0.9978 | 1.0141 | +23 |
| Linux x86-64; 4 | Interior-first | 1.0012 [0.9914, 1.0285] | 0.9991 | 0.9916 | +20 |
| Linux AArch64; 4 | Plane | 0.9778 [0.9340, 1.0287] | 0.9988 | 0.9820 | -31 |
| Linux AArch64; 4 | Interior-first | 1.0012 [0.9973, 1.0039] | 0.9990 | 1.0064 | +125 |
| macOS AArch64; 2 | Plane | 1.0025 [0.6018, 1.3959] | 1.0012 | 1.0000 | +15 |
| macOS x86-64; 4 | Plane | 1.0703 [0.8642, 1.2846] | 1.0088 | 1.0000 | -25 |

Neither tails nor counter removal establish a consistent winner. For example,
Linux x86-64 tile-64 core p95 old/research medians are 1.1242 and 1.0551 at the
two sizes, despite core medians near one; Linux ARM gives 0.9952 and 0.9576.
These are ratios of each process's nearest-rank warm p95, not population p95.
The large case has only 64 warm calls/process, so its tail estimate is coarse.
Counter-off core ratios on Linux x86-64 are 1.0148 and 0.9965. On macOS ARM,
small FIR counter-off is 0.8457 [0.6430, 0.8935], but its A/A range is wide;
that deserves confirmation before selecting a counter policy.

The report stops at this bounded comparison. Resolving every noisy cell or
explaining the Linux fine-grain exception is further qualification work, not
permission to resume production integration before the owner reads this report.

### Local Apple Silicon diagnostic cohort

The 2026-09-09 local cohort used Apple Clang 21.0.0, Darwin 25.6.0 arm64,
the `d858008f` compiler and the uncommitted comparison harness whose exact
sources/hashes are retained in `/private/tmp/whitefoot-pure-compare-local/pure-compare/`.
The sandbox denied detailed sysctl CPU metadata. The local interactive machine
is diagnostic evidence, not a substitute for controlled native CI.
All 1,200 processes (20 cases, three widths, five rounds, four images) passed
output and actual-pool checks, and the complete matrix reader returned zero.

FIR figures below use the median warm core time within each process, then the
median of five paired old/research ratios. Bounds are the five ratios' minimum
and maximum, not confidence intervals. Ratios above one favor research.

| Participants | Outputs; tile | Old/research warm core ratio | Pair range | Identical-image replica/research range |
| --- | --- | ---: | --- | --- |
| 1 | 4,096; 64 | 0.9984 | 0.9968–1.0016 | 0.9968–1.0000 |
| 1 | 65,536; 64 | 1.0027 | 0.9729–1.0099 | 0.9663–1.0071 |
| 2 | 4,096; 64 | 1.0513 | 0.9383–1.2225 | 0.8107–1.1106 |
| 2 | 65,536; 64 | 0.9919 | 0.9481–1.0010 | 0.9487–0.9939 |
| 4 | 4,096; 64 | 0.9930 | 0.9462–1.0426 | 0.9537–1.0429 |
| 4 | 65,536; 64 | 1.0023 | 0.9689–1.0450 | 0.9454–1.0300 |

For four participants, selected whole-process ratios include startup, checks,
and shutdown; they are not core-only costs:

| Case | Old/research wall median [range] | CPU ratio | RSS ratio | Context-switch delta |
| --- | --- | ---: | ---: | ---: |
| FIR 4,096; tile 64 | 1.0099 [0.9669, 1.1356] | 1.0135 | 1.0083 | -6 |
| FIR 65,536; tile 64 | 1.0529 [0.9800, 1.0686] | 1.0311 | 1.0065 | -11 |
| Mandelbrot plane; 65,536 | 1.0046 [0.9946, 1.0229] | 1.0043 | 1.0045 | -20 |
| Mandelbrot interior-first; 65,536 | 0.9965 [0.9959, 1.0033] | 0.9982 | 1.0045 | +11 |

Disabling the research steal counter gives four-participant FIR core ratios
of 0.9764 [0.9229, 1.0447] and 0.9728 [0.9290, 1.0638] at those two sizes.
These noisy ranges do not establish a repeatable counter benefit. The small
two-participant FIR cell is especially noisy even for identical executable
bytes. No row supports claiming a new scheduler architecture is much faster.
Keep full per-cell and tail data in the artifacts; these representative rows
are not a universal equivalence claim.

Both candidates share a visible policy limit: every 4,096-point Mandelbrot
case leaves the pool unstarted even when four participants are requested.
For the all-interior shape, the research whole-command median is 58.273 ms at
one participant and 58.066 ms at four. At 65,536 points the same shape does
start the pool and scales from 885.273 ms to 239.308 ms. These commands perform
16 repetitions. This is evidence to revisit the shared split-cost estimate
after choosing the runtime base; it is not evidence that one candidate has a
better deque. Similar old/research performance does not mean scheduling is
already optimal.

This panel can distinguish a large runtime regression or a counter cost. It
cannot rank all dynamic runtimes, prove top-1 performance, or cover interactive
bursts, irregular recursive application graphs, NUMA and larger machines.
The existing [workloads](WORKLOADS.md) and [native references](BASELINES.md)
remain necessary for later production qualification. No SIMD conclusions are
drawn; vectorization and contraction are disabled throughout this comparison.

## Portability and likely consolidation work

The historical implementation supplies POSIX and Windows bodies; research
supplies POSIX/LP64 only. Historical Windows uses address waits and different
startup/configuration refusal behavior. Its Clang atomics path also requires
the thief-ordering audit. Keeping that file verbatim would not establish the
same correctness and performance across the five supported targets.

Windows has useful implementation choices that are absent from both POSIX
files: a per-lane wake generation and address comparison before sleeping,
and per-lane identity counters compiled only for probes. Its production steal
path has no global successful-steal increment. Conversely, it requires 2–64
participants and treats partial worker startup and an oversized task frame as
fatal host failures; POSIX can decline the pool or frame. These are substantive
platform differences to reconcile, not merely pthread-to-Windows spelling.
The wake-generation mechanism is a candidate to retain, not a measured claim
that it beats condition variables on another OS.

After an owner choice, the bounded engineering work is to maintain one common
compute protocol and queue implementation with narrow platform wait/thread
primitives; retain ordinary calls/current-stack helping and required exhaustion
handling; select a default counter policy; and remove research-only controls.
Port-specific failure semantics and wake lifetimes need deliberate tests.
This does not require choosing a runtime at link time from a function's I/O
needs, introducing stackless calls or changing public function ABI.

I/O must remain functional during a later compute transition, but integrating
parallel I/O into this compute executor is a separate design question. The
checkpoint preserves the unified experiment and its limitations. All five
platforms still require native correctness/performance CI before a future
formal-runtime delivery can be called complete. This report does not satisfy
that production acceptance condition or authorize a merge.
