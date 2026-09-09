# Historical and recovered pure-compute runtimes

## Scope and provisional conclusion

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
The local paired measurements below show no large old/research separation.
Native CI measurements are being collected; no measured winner is selected here.

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
