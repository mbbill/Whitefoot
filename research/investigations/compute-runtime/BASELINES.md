# Pure-compute reference matrix

The target is a credible measured frontier for CPU computation, including the cost
of expressing parallel work through ordinary, synchronously returning functions.
The reference is the strongest qualified implementation for each named workload
and resource budget. No library is designated the winner in advance.

Primary sources were inspected on 2026-09-07. The first executable references are
the [strict native FIR kernels](../../experiments/compute-runtime/README.md#native-fir-and-first-cost-attribution):
direct and 4/8/16-output SIMD candidates, with full-result qualification and a
same-host calibration caller. They do not yet establish a confirmed frontier.
The [static worker control](../../experiments/compute-runtime/README.md#static-worker-control)
now dispatches the same qualified objects at one/two/four actual lanes; its
condition-variable policy has full-oracle/lifecycle evidence and a dated
[Linux calibration](../../experiments/compute-runtime/README.md#linux-static-control-before-wf-output-groups).
The [scalar scheduler panel](../../experiments/compute-runtime/README.md#scalar-scheduler-comparison)
adds executable oneTBB, Parlay, Rayon join/parallel-iterator and static-spin controls against the recovered
WF runtime. All use the same precompiled scalar work and callback objects,
fixed record chunks and one/two/four participants including the caller.
Automatic SIMD and cross-object LTO are disabled for this scheduling experiment.
Its WF row uses a C adapter to the real runtime, not compiler-generated WF;
end-to-end language results remain separate. No row establishes the frontier.
The [recursive quadrature panel](../../experiments/compute-runtime/README.md#native-recursive-grain-comparison)
adds direct oneTBB `parallel_invoke` and native Parlay `par_do`, sharing a scalar
C++ kernel and five spawn-depth settings against generated WF and C/C++ serial
controls. Diagnostic subtree counts check actual work and foreign-thread branch
execution. The first M1 screen shows strong grain sensitivity and remaining WF
losses. Recursive Rayon, matched-grain native WF, alternative TBB contexts/APIs,
held-out tuning and qualified native-host placement remain missing; the existing
flat callback comparisons do not fill those gaps.
The [record batch panel](../../experiments/compute-runtime/README.md#variable-length-utf-8-record-batches)
also qualifies a native state machine and bounded ASCII-word candidate.
For its validation-plus-scalar-count contract, simdutf **v9.1.1**, commit
`9dd35adc5f2c87a53c5a0e6e5b43af6fffe7187e`, is outside the maintained panel
while SIMD work is parked:
the [pinned API](https://github.com/simdutf/simdutf/blob/v9.1.1/include/simdutf/implementation.h)
requires validation followed by `count_utf8` on valid inputs. A successful
`validate_utf8_with_errors` result counts bytes, not scalars. Compare both
validator forms because early-invalid work differs; a two-pass library API is
not a lower bound for a fused implementation.
Earlier I/O or mixed-panel results do not qualify these comparisons. This matrix belongs to the [compute-runtime
investigation](README.md); update its evidence cells when actual results exist,
and consolidate it if that investigation supersedes this selection.

## Reference selection

“Why it may be fast” below is a mechanism-based hypothesis, not a measured claim.
Initial references cover serial code generation, static scheduling, and three
different mature dynamic parallel systems. Additional candidates enter a bounded
screen where their mechanism is relevant; a full Cartesian product is unnecessary.

| Reference / role | Forms to qualify and potential advantage | Limitation and discriminating measurement |
|---|---|---|
| Optimized native serial; mandatory anchor | C/C++ and Rust ordinary loops/recursion, no scheduler; optimized target ISA, inlining and legal SIMD. Establish useful work cost and compiler quality. | A deliberately scalar loop is only a diagnostic when SIMD is legal. Inspect generated instructions and result checks; compare native serial, each framework at one CPU, and parallel execution separately. |
| Hand-tuned static partition + SIMD; mandatory regular-work anchor | Persistent workers process contiguous or explicitly tiled ranges; tune blocks, unrolling and vector width. Avoid per-element task scheduling and exploit predictable locality. | Static work division cannot repair unknown skew. Charge dispatch/barriers, worker startup in cold runs and any preprocessing. Compare uniform and skewed distributions; retain scalar and vectorized leaf controls. |
| Rayon; mandatory dynamic reference | `par_iter`/parallel slices for data parallelism; `join` and borrowing `scope` for recursive/nested work. Local execution plus work stealing can keep small branches cheap while exposing work to idle workers. | Tune grain/cutoff and pool width; do not substitute global `spawn` plus channels for normal fork/join. Measure one-worker overhead, imbalance, nesting, stack/heap cost and burst restart. Outside-pool `join` and inside-pool execution have different caller participation. |
| OpenCilk; mandatory compiler-assisted reference | `cilk_spawn`, scoped synchronization and `cilk_for`; compiler-visible parallel control and its serialization allow optimization around spawn/join. Relevant to WF's sequential-call design question. | Requires its compiler/runtime, so compiler effects need a matching serial build. Measure cutoffs, local serial overhead, actual steals and separate-object calls. Work/span analysis concerns the chosen computation DAG; it is not a hardware or algorithmic optimum. |
| oneTBB; mandatory dynamic reference | `parallel_for`/`parallel_reduce`, `parallel_invoke`/`task_group`; local depth-first execution and stealing aim to combine locality with load balance. Compare automatic, affinity and static partitioning where appropriate. | Static partitioning omits load balancing; adaptive partitioning and retained affinity have distinct costs. Measure cold/warm reuse, grain sensitivity, nested participation and actual worker budget; do not force one partitioner on all workloads. |
| ParlayLib; additional fine-grain candidate | Native `par_do`/`parallel_for` scheduler, explicit grain control and nested helping; its parallel algorithm library also supplies end-to-end candidates. | Force and record the native scheduler, since the same API can select OpenCilk, OpenMP, TBB or serial backends. Charge scheduler creation and idle policy. Upstream README says support beyond x86-64 is unexplored: ARM is an explicit qualification gap, not an assumed supported platform. |
| Taskflow; additional graph / dynamic-task candidate | Reusable task graph for known dependencies; dynamic subflow and runtime tasking are distinct forms. Graph reuse can amortize construction; cooperative task execution may serve irregular work. | Charge node/edge construction and reclamation unless reuse is part of the workload. Do not rank Taskflow using only recursive subflows: upstream identifies extra graph overhead and offers runtime tasking. Qualify the fastest applicable pinned API before a framework-level conclusion. |
| Go; additional native runtime / end-to-end candidate | Chunked goroutines and persistent workers with explicit `GOMAXPROCS`; compare natural bounded parallel forms and their startup, scheduling and allocation costs. | Qualify native scalar code generation and serial work before attributing a gap to scheduling. Calling a common C leaf through cgo on every small chunk adds a different boundary; that result cannot establish pure scheduler cost. |

LLVM documents both loop and SLP vectorization, including diagnostics and
limitations from calls and floating-point ordering; optimized serial qualification
must inspect those effects. [LLVM vectorizers](https://llvm.org/docs/Vectorizers.html).
Rayon's documented local/remote execution and borrowing forms ground its row.
[Rayon join](https://docs.rs/rayon/1.12.0/rayon/fn.join.html),
[Rayon API](https://docs.rs/rayon/1.12.0/rayon/).
OpenCilk's parallel calls preserve an ordinary result-returning source form, while
`-fopencilk` is required at compilation and link time.
[C++ conversion](https://www.opencilk.org/doc/users-guide/convert-a-c-program/),
[compiler usage](https://www.opencilk.org/doc/users-guide/getting-started/).
oneTBB documents local scheduling and the partitioner tradeoffs separately.
[Scheduler](https://uxlfoundation.github.io/oneTBB/main/tbb_userguide/How_Task_Scheduler_Works.html),
[partitioners](https://uxlfoundation.github.io/oneTBB/main/tbb_userguide/Partitioner_Summary.html).
Parlay's selection macros and lifecycle are visible in its implementation.
[Parallel API](https://github.com/cmuparlay/parlaylib/blob/master/include/parlay/parallel.h),
[scheduler](https://github.com/cmuparlay/parlaylib/blob/master/include/parlay/scheduler.h),
[platform scope](https://github.com/cmuparlay/parlaylib/blob/master/README.md).
Taskflow's documentation explicitly distinguishes subflow overhead from runtime
tasking; its examples' timing tables are not evidence for this investigation.
[Subflows](https://taskflow.github.io/taskflow/SubflowTasking.html),
[recursive runtime tasking](https://taskflow.github.io/taskflow/ExamplesFibonacciNumber.html).

| Source identity actually verified | Experiment pin / evidence still missing |
|---|---|
| Rayon **1.12.0**, rayon-core **1.13.0**, full Cargo lock in the scalar scheduler experiment | Executable recursive join and indexed parallel iterator, caller worker zero, process-lifetime pool, identical scalar C leaf/callback. Artifacts retain Rust compiler, flags, emitted C ABI symbol and archive hashes. Local flat-work/full-output/capacity checks are implemented; native-host performance, nested composition and broader cutoff tuning remain open. |
| OpenCilk **3.0**, tag `opencilk/v3.0`, release short commit `cd8ccfc`; release notes identify LLVM 19.1.7. [Releases](https://github.com/OpenCilk/opencilk-project/releases). | Resolve full compiler/runtime commits and package hashes. The same release page lists 4.0 release candidates separately; do not silently substitute one. |
| oneTBB **v2023.1.0**, full commit `3046c8b0c29df995980003ea24f4d78c80ec0c8d`. [Release](https://github.com/uxlfoundation/oneTBB/releases/tag/v2023.1.0). | Scalar scheduler experiment builds and retains a shared Release library with automatic vectorization/IPO disabled, persistent arena, automatic partitioner and explicit participant budget. Other partitioners and held-out/native-host performance confirmation remain open. |
| Taskflow **v4.1.0**, release short commit `45366fe`. [Release](https://github.com/taskflow/taskflow/releases/tag/v4.1.0). | Live docs list **4.2.0 (Master)** separately. Verify each chosen runtime/subflow API against pinned headers; development documentation is not proof that an API exists in 4.1.0. [Release index](https://taskflow.github.io/taskflow/Releases.html). |
| ParlayLib **`51017699dcc421f80479cdb238d3092233ad0d26`**, native header backend | Scalar scheduler experiment fixes grain one, private caller-owned pool, default elastic policy and 10,000-microsecond steal timeout; checks full output and participant capacity. Alternative idle policies and held-out/native-host performance confirmation remain open. |
| Native C strict FIR direct/output-lane candidates and static workers: checked-in source and executable qualification in the [experiment](../../experiments/compute-runtime/README.md#native-fir-and-first-cost-attribution) | Dated Linux screens cover single-caller kernels and static workers with actual capacity, full-oracle and lifecycle qualification. Each calibration retains source/object hashes, compiler and target flags. Held-out confirmation remains pending; other native languages and other workloads remain unimplemented. |

## Workload matrix

The [WF workload suite](WORKLOADS.md) owns concrete program candidates, capability
gaps, data selection and correctness coverage. FIR and the record validator are
implemented WF programs in that suite; the other application families remain candidates. The matrix
below assigns comparison roles to those families; it does not replace a serious
WF program suite with wrappers around one kernel. `par_layout` is a historical
smoke/regression program, not evidence for future application performance.

Each real program needs an independent result oracle, audited representative
inputs, and both stage-level and complete-computation boundaries where useful.
Include small batches, cache-size tiers, realistic distributions/skew, nested
library composition and held-out input families. Synthetic diagnostic stress
(empty/tiny tasks, controlled recurrences and skewed trees) isolates mechanisms;
it does not establish application quality or substitute for those real programs.

| Family | Useful work and independent axes | Strong comparisons / what it distinguishes |
|---|---|---|
| Regular compute | Independent integer transforms and FP kernels; recurrence within each item versus independent vectorizable items; arithmetic intensity and working set. | Serial and static SIMD versus each dynamic loop form. Distinguish scalar code quality, vectorization and dispatch cost from scaling. |
| Bandwidth | Copy/triad/map/reduction with declared bytes; cache-resident versus substantially larger than LLC; contiguous versus tiled access. | Native tuned streaming loops and static partition versus dynamic loops. Report useful GB/s, memory placement and CPU; a measured bandwidth envelope is not a theoretical ceiling. |
| Irregular recursion / skew | Divide-and-conquer over disjoint outputs; balanced and deliberately skewed trees, deterministic heavy-tailed leaf costs; a real sort or traversal companion. | Rayon join, OpenCilk, TBB tasks and qualified Parlay; Taskflow's applicable dynamic form. Hold tree/cutoff fixed for mechanism checks, permit better algorithms only in end-to-end comparisons. |
| Tiny tasks | Sweep nonzero leaf work, task count and grain; compare flat ranges with nested fork/join producing the same checked output. | Locate crossover against serial and batching. Separately record created tasks, steals, code/stack size and allocation; nanoseconds per empty task is diagnostic only. |
| Nested / bursty | Parallel calls inside parallel calls sharing one budget; repeated short waves with controlled idle gaps; mixed short and long CPU jobs. | Test cooperative joins, oversubscription, idle CPU and restart latency. Reusable Taskflow graphs apply only when dependency reuse is real; dynamic construction remains separately charged. |
| Complete in-memory pipelines | The concrete WF programs in the workload suite, with dependent stages, intermediate ownership and representative small/large batches. | Compare matched stage kernels and the strongest equivalent whole computation separately; include intermediate allocation/copying and small-batch latency. Faster isolated leaves need not make a faster program. |

## Two comparisons, two timing boundaries

The **same-kernel mechanism comparison** fixes input, algorithm, operation
semantics, useful work, task decomposition and leaf cutoff. A separately compiled
C-ABI leaf can give C++, Rust and WF wrappers exactly the same machine kernel;
disable LTO for that diagnostic and retain the object. This measures the imposed
call boundary too. It cannot stand in for an optimizer's natural inline program.

The **best end-to-end comparison** permits each implementation's ordinary strong
form: inlining, SIMD, batching, better decomposition and algorithm/layout changes
that preserve the declared result contract and total resource budget. Retain its
optimized serial counterpart. If a library sort replaces a teaching quicksort,
that is an algorithm-plus-runtime comparison, not isolated scheduler overhead.
Record integer overflow domains and FP order/precision explicitly; a strict FP
result and a reassociated approximation are different contracts.

Measure **cold invocation** including runtime/graph construction, allocation,
required input preparation and teardown, and **warm invocation** with a named
reused pool/graph separately. Cache state is an independent axis: a warm pool does
not imply cached data. All spawned work must finish before the measured call
returns; checks may run afterward but must keep the useful computation observable.
For bursts, include queueing and drain in latency and charge CPU during idle gaps.

## Ordinary calls and separate compilation

The desired qualification is an ordinary call whose result and borrowed storage
are ready for use when it returns, including when its implementation forks work.
The library rows can wrap ordinary leaf functions in tasks without changing the
leaf's signature. Their scheduling APIs still require closures, captures or an
execution context; unchanged signatures do not imply absent metadata or free joins.
OpenCilk changes spawning code's compilation while its source functions retain
ordinary parameters/results; binary interoperability must be tested, not inferred
from that spelling.

Qualify direct and indirect calls to an opaque leaf, a separately compiled
parallel callee called by a serial wrapper, nested parallel callees, and borrowed
stack outputs whose lifetime ends immediately after return. First use no LTO;
then report whole-program optimization separately. Inspect exported symbols,
object calls, stack/frame size and code size. Pin the C++ toolchain ABI; use an
explicit C ABI for cross-language leaves rather than assuming a stable Rust ABI.
These tests evaluate suitability for WF; they do not select a new language ABI.

## Resource, tuning and evidence rules for this comparison

1. Define the budget as actual allowed CPUs, memory and workload concurrency,
   including a caller that computes or spins. Record all runtime threads and
   per-thread affinity, not just a worker-count option. Keep nested calls in the
   same pool. Report oversubscription as a separate configuration.
2. Verify package/core/SMT topology, CPU masks, quotas, NUMA placement and actual
   worker participation. Record architecture, compiler target and available ISA,
   frequency/thermal conditions, and heterogeneous core types. M1, Linux ARM and
   x86 results remain separate populations; VM topology is not dedicated hardware.
3. Report elapsed time, useful work/s, whole-process user/system CPU, CPU/work,
   peak RSS and allocated/live bytes where available. Distinguish reserved stacks
   from resident pages. Retain context switches and separate observed task/steal
   counts. Burst tails need sufficient samples; missing attribution is explicit.
4. Calibrate a bounded grain/partition/idle-policy set on separate inputs and
   repetitions, then confirm on held-out sizes and input families. Freeze source,
   dependencies, tools, binaries, settings and input
   identities before alternating same-host confirmation. Preserve every raw pass,
   failure and adverse tail; never tune on confirmation and then reuse its score.
   Report per-workload results and regressions; an aggregate score cannot conceal
   a loss on a workload family or small-batch latency.
5. Before timing, validate the actual chosen forms on empty/single/uneven inputs,
   skew and nesting; compare full results with an independent oracle. Exercise
   task capture lifetime and joins with appropriate race/memory tooling. Such
   dynamic checks qualify tested executions; they are not WF-style static proof.
6. Admit a candidate to confirmation only after correctness, configuration,
   resource and code-generation evidence is available. Installation failure or an
   unqualified backend is **unmeasured**, not slow. A dominated configuration may
   leave the retained frontier only for its tested workload/budget; retain its
   raw evidence and do not reject a framework from one unfavorable idiom.

Evidence progresses from **source-informed candidate** to **qualified executable**
to **calibrated/frozen** to **independently confirmed on a named host**. Every row
except the qualified FIR native kernels is currently at the first stage. A strongest observed frontier can justify the
next optimization target; neither marketing, elapsed time alone, nor ideal
work/span bounds establish that no better program can exist.
