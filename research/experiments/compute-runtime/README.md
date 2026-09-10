# Compute runtime recovery control

The original recovery control isolates current-stack compute join/help/steal
from the shared I/O scheduler. Its frozen research runtime serves the compiler's
ordinary task frames, checks the concurrent protocol, and links the same
unmodified emitted module with this runtime and the compiler's weak sequential
fallback. Current implementation work belongs in `compiler/`; the
[ordinary-command panel](#ordinary-cli-mandelbrot) measures its normal link path
without this research runtime. The FIR calibration
caller below also links the identical optimized WF object with the existing
maintained runtime, and compares qualified native output-lane SIMD candidates.
This is cost attribution, not a confirmed performance frontier. The current
[scheduler panel](#scalar-scheduler-comparison) disables SIMD and compares
equal worker budgets with one common scalar compute object. Earlier FIR SIMD
results below retain their original conditions and do not rank these schedulers.

The [investigation](../../investigations/compute-runtime/README.md) owns the
architecture question, [WF workload coverage](../../investigations/compute-runtime/WORKLOADS.md)
and [native references](../../investigations/compute-runtime/BASELINES.md).
`abi.wf` is only a small ABI diagnostic. FIR, UTF-8 record batches and
[Mandelbrot point rendering](#mandelbrot-point-rendering) provide distinct
computations; nested composition and broader application coverage remain open.
Keep this frozen control through qualification of the maintained compute-first
runtime; remove it when the production replacement has passed the comparison.
`make formal-screen OUT=<fresh-absolute-path>` compares one scalar WF object
against repaired historical and recovered cores on POSIX, and the repaired
historical native core on Windows. It also checks the ordinary CLI executable.
Windows additionally compares the prior maintained core at `d39b4836` with
the restored `YieldProcessor` hints. Only the two empty-scan spin sites differ;
the script checks that source delta and shares the WF/host objects and platform
sources. Both raw logs use the same internal runtime label; filenames and
summary rows distinguish the cores. This isolates the net hint effect, including its change to elapsed
spin duration; it does not isolate SMT contention from park frequency.
An identical production image is the noise control. Raw artifacts retain core
and allocation-inclusive durations, process CPU and memory; wall-time summaries
are an initial screen, not whole-workload or all-platform acceptance.

## Historical versus recovered pure-compute comparison

The owner requested a report before selecting a production compute runtime.
`make pure-compare OUT=<fresh-absolute-path>` builds the current compiler and
compares the historical POSIX runtime at `9051576f` with frozen research
`runtime.c` at `d858008f`. Both consume the same scalar WF object per workload.
The [comparison report](../../investigations/compute-runtime/PURE-COMPUTE-COMPARISON.md)
owns the source review, results and recommendation; this is an authorized
historical comparison, not another maintained compiler implementation.

`pure-compare-repairs.patch` applies only to exported scratch copies. It makes
thief index reads sequentially consistent in both, makes old deque pointer
cells atomic, and adds read-only historical observation adapters. The frozen
originals remain in each artifact. These are **repaired variants**, not exact
original-release timings. In particular the original research runtime still
has the index-ordering issue; its earlier timings do not qualify that issue.

The primary pair enables successful-steal counters in both runtimes. A separate
research-off variant disables its counter, and a byte-identical research
replica measures process noise. No capacity-budget overrides, SIMD, FMA or LTO
are enabled. FIR host strings/statistics branches and final executable layout
can differ; only the WF compute object is identical, not the entire executable.

Five process rounds cover FIR at 4,096/65,536 samples and tiles 16/64/1,024,
plus Mandelbrot's seven spatial/skew shapes at both sizes. Widths are 1/2/4
where available; `PURE_WIDTHS` and `PURE_PASSES` select an explicit diagnostic
subset. The saved plan is independently checked against all expected records.
FIR checks every output/history bit against an independently qualified native
kernel. Mandelbrot checks an ordered digest from the native independent-orbit
oracle. Every process verifies actual pool participation; Mandelbrot's 4,096
point default intentionally remains sequential. Its common exit observer's
stderr write is included in whole-command time.

`process.tsv` retains process wall/CPU/RSS/context switches; `summary.tsv`
reports paired ratios without pooling unlike workloads. `raw/` retains FIR's
first/warm core and cycle samples, counts and steals. These warm samples are
not independent process repetitions. Sources, compiler, objects, binaries,
host metadata and SHA-256 manifest accompany the logs. A green CI job means
build/output/matrix checks passed, not performance acceptance. Four POSIX
targets are measured; the unported research implementation cannot supply a
Windows comparison. The report records that gap instead of inventing a port.

## Source and execution boundary

The WF programs use the current `len_of` measure spelling. Their legacy
`buffer`/`box` allocations have no written allocation effect under EFF-1's
ambient-heap paragraph; reads/writes and compiler-derived allocation and
resource-closure checks remain. They still allocate the same representations.
Explicit `Heap`/`Vector` migration would change provider, refusal and storage
semantics and is a separate experiment, not part of this compatibility update.

`runtime.c` recovers the pre-I/O runtime from
[`fee335654d9dea027f4636bbad448d57a4e84d08`](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c).
It renames acquisition to the current `wf__par_acquire_lane` symbol and retains
256-byte frames with 16-byte alignment, ordinary task calls, and result storage
owned by the acquiring lane until release. The current emitter's
[frame layout limit](../../../compiler/src/backend/target.rs) and
[join/read/release lowering](../../../compiler/src/backend/emitter/parallel.rs) are the
compatibility authority. The private observation functions in `runtime.h` are
test instrumentation, not a language FFI or separate-compilation proposal.

An unstolen newest task executes directly in `wf__par_join`. A stolen target
causes local helping and stealing on the current worker stack before the
historical spin/yield/condition-wait policy sleeps. No suspended-stack pool or
completion implementation is linked. The current
[`wf_floor.c`](../../../compiler/src/backend/wf_floor.c) still supplies entry,
worker stack sizing, and exhaustion handling; its weak scheduler-entry seam is
left unoverridden. Generated stack-probe attributes remain in the emitted LLVM.
Nested helping can increase stack depth beyond the sequential call path: this
control does not assert an identical depth bound or remove exhaustion handling.

Two safety repairs distinguish it from the historical bytes:

- Deque pointer cells use atomic relaxed reads/writes. A thief delayed between
  reading indices and reading a cell can outlive a ring wrap. Its later failed
  CAS does not make a plain cell read racing with reuse legal. The existing
  index publication and ownership ordering remain in place.
- The owner's mutex and condition variable are initialized before any workers
  start; a zero-worker startup destroys those resources and declines tasks.

Several historical choices remain deliberately visible: 64 slots per lane,
at most 64 lanes,
4096 empty searches before 16 yields, and the old split-budget thresholds.
These are controls to measure and change, not selected optimal settings.
`WF_COMPUTE_STATS=1` retains the shared successful-steal counter and its
observer by default for existing diagnostics. The scalar scheduler timing and
sanitizer objects use `WF_COMPUTE_STATS=0`: both the counter and observer are
absent, checked with `nm`, rather than returning an invented zero count.
Statistics do not participate in task publication, ownership or completion.

## Owner-local task protocol cost

`protocol_cost.c` measures the unstolen acquire/publish/join/read/release path
against direct calls to the same opaque callback. It uses the ordinary runtime
and floor without scheduling overrides. Each helper first steals one callback
and waits on a condition variable inside it; the owner checks the actual pool
width before waiting for entrants. These callbacks keep helpers unavailable
without spinning. Their frames and owner-stack gate remain alive until every
helper callback has been joined, after all measurements.

The owner processes 262,144 tiny modular-integer callbacks per sample, with
1, 8 or 32 simultaneously published tasks, joined newest first. The direct
control initializes equally sized groups of stack frames and invokes the same
function pointer in the same reverse order. Both write every result to an
output array, checked outside the timed interval. The difference includes
opaque runtime-frame storage, initialization/checking, deque synchronization
and release; it does not isolate atomic instructions from cache traffic or
measure normal parallel speedup, idle-worker wakeup, stealing or exhaustion.
Keep this diagnostic with the runtime experiment; consolidate or remove it
when that protocol is replaced.

`check-protocol-cost` qualifies both ordinary and fully C-sanitized images at
W2/W4 and all three depths through the canonical `check` target. The ordinary
image uses scalar O3 without LTO, statistics enabled and events disabled.
Statistics count successful steals; the timed intervals must observe zero.
Helpers remain inside their held callbacks, so this does not infer the absence
of all deque contention merely from a zero successful-steal count.

```sh
make -C research/experiments/compute-runtime check-protocol-cost OUT=/tmp/wf-protocol
make -C research/experiments/compute-runtime protocol-cost-calibrate \
  OUT=/tmp/wf-protocol RESULTS=/tmp/wf-protocol/protocol-cost/calibration
```

Calibration requires a fresh `RESULTS` directory and the already qualified
ordinary image. It runs 30 processes: W2/W4, depths 1/8/32 and five passes,
reversing width/depth order on odd passes. Each process warms both paths for at
least 100 ms, then alternates direct/task order across nine paired samples.
`first` means the first measured pair after warmup, not cold startup; retain it
separately from the following eight pairs. Raw nanoseconds include wall time
and enclosing owner-thread CPU time; neither includes output verification or
printing. Warmup duration/pair count and all 18 measured rows are retained.
The footer counts 4,718,592 checked measured outputs per process, excluding
warmup. No elapsed-time limit selects a performance pass; a 30-second process
watchdog detects a stalled diagnostic.

The `compute-bench` scheduler job runs this panel sequentially under the same
recorded CPU mask and retains images, flags, source hashes and raw samples in
its `protocol-cost` artifact directory. Helper count is a setup condition;
there is only one executing compute thread during measurement. Thread
placement/frequency remain uncontrolled. Compare paired task-minus-direct
nanoseconds per callback within each process before comparing configurations.

The September 8 M1 screen uses Apple Clang 21.0.0 on MacBookPro18,3
(eight physical/logical CPUs), ordinary image SHA256
`9e2fbf2a2b29c78246782aba82084f5c1b323dc99aeb01a8fbbb5b936d1112ff`.
All 30 processes pass, retaining 540 measured rows, 141,557,760 checked output
positions and zero steals in every measured interval. Values below are
nanoseconds per callback: medians of five process means over eight warm pairs,
with the range of those five means. Direct/task columns are separate medians;
the delta column subtracts within each process before taking its median.

| Pool / pending depth | Direct wall median | Task wall median | Paired extra wall median [min–max] | Paired extra owner CPU median |
| --- | ---: | ---: | ---: | ---: |
| W2 / 1 | 1.437 | 14.613 | 13.176 [13.164–13.683] | 13.171 |
| W2 / 8 | 1.459 | 13.725 | 12.259 [11.917–12.625] | 12.245 |
| W2 / 32 | 1.305 | 14.186 | 12.824 [12.413–13.033] | 12.820 |
| W4 / 1 | 1.455 | 15.016 | 13.514 [13.144–13.852] | 13.515 |
| W4 / 8 | 1.406 | 13.385 | 11.985 [11.899–12.804] | 11.983 |
| W4 / 32 | 1.284 | 13.736 | 12.452 [12.418–13.136] | 12.449 |

Owner CPU and wall deltas are close in this cohort; depth eight is lower than
depth one in all five paired passes at each width. This does not isolate the
last-item CAS: grouping also changes frame reuse, loop work and cache traffic.
The generated Mandelbrot W4 capacity budget is six split levels, at most 64
terminal chunks/63 publications for one map, while this diagnostic uses
16-byte nonrecursive callbacks rather than its 120-byte recursive frames.
Do not multiply the M1 delta into an estimate of Linux map overhead or stolen
completion cost. The next comparison needs matching decomposition and real
generated-frame work as well as the Linux local-protocol measurement.

The Linux protocol panel at exact revision
`e8e5d23a83afa296991faac1c8d7834dc84b34eb` is retained in
[compute run 34224388554](https://github.com/mbbill/Whitefoot/actions/runs/34224388554),
artifact `10055333586`, ZIP SHA256
`d02d28ae66ffbc624b2f86686428f87373a36f47fd52d2e9f552944e9172fd45`.
Ordinary image SHA256 is
`c1f3d765c7b0b8d2940e0933f7aba00400c1cad0b38714ad76c1babf2132b623`.
Independent replay verifies 30 processes/540 rows/141,557,760 measured outputs,
12 ordinary/ASan-UBSan qualifiers, all nine manifest entries and seven source
snapshots against that revision. Every measured steal delta is zero.
The host exposes two EPYC 7763 physical cores/four SMT CPUs under mask0–3;
Clang18.1.3 uses scalar x86-64-v3 without LTO. Individual placement/frequency
and CPU quota remain unqualified. It is a separate cohort from M1.

| Pool / pending depth | Direct wall median | Task wall median | Paired extra wall median [min–max] | Paired extra owner CPU median |
| --- | ---: | ---: | ---: | ---: |
| W2 / 1 | 3.299 | 11.645 | 8.347 [8.264–8.930] | 8.347 |
| W2 / 8 | 2.170 | 9.124 | 6.951 [6.911–7.491] | 6.950 |
| W2 / 32 | 2.190 | 9.149 | 6.956 [6.946–7.183] | 6.956 |
| W4 / 1 | 3.299 | 11.853 | 8.560 [8.518–9.737] | 8.560 |
| W4 / 8 | 2.173 | 9.157 | 6.984 [6.970–7.278] | 6.982 |
| W4 / 32 | 2.193 | 9.207 | 7.003 [6.947–7.168] | 7.003 |

Units and aggregation match the M1 table. Both depths8/32 have lower extra
wall cost than depth1 in all five matched passes at each width; their mutual
comparison is mixed, unlike M1. The direct control itself drops from about
3.3 ns at depth1 to 2.2 ns at depths8/32, reinforcing that grouping changes
more than the runtime's last-item handling. These measurements do not bound
the cost of a contended publication or stolen completion.

## Run and retained outputs

Requires a POSIX LP64 host, Clang/C++17 with sanitizers, CMake, Git, pthreads,
`nm`, the pinned native sources fetched below, and the current offline Cargo
dependencies. The root `make check` calls this directory's
`check` target through `research-tests`, using ASan/UBSan by default.

```sh
make -C research/experiments/compute-runtime scheduler-fetch OUT=/tmp/wf-compute-asan
make -C research/experiments/compute-runtime check OUT=/tmp/wf-compute-asan
make -C research/experiments/compute-runtime check SANITIZER=thread OUT=/tmp/wf-compute-tsan
```

The default `WFC` is built from the current compiler sources in `OUT/compiler`.
A caller may provide `WFC=/path/to/current/whitefootc` after building that exact
tree. `CC` can also be overridden. Binaries are rebuilt when invoking the checks
so changing sanitizer flags cannot reuse an earlier binary. Output stays under
`OUT`: C probe, emitted `abi.ll`, parallel ledger, declaration and symbol lists,
recovered-runtime executable, and weak sequential executable. No timer is used
to select WF source acceptance; bounded waits in the C probe only detect a
hung test.

ASan's use-after-return detection is enabled by default in this target, including
on hosts where the sanitizer would otherwise leave it off. The owner-stack
probe reads the physical call-frame address; an address-taken C local may live
on ASan's fake stack instead. See the
[sanitizer mode](https://clang.llvm.org/docs/AddressSanitizer.html#stack-use-after-return-uar)
and [frame-address builtin](https://gcc.gnu.org/onlinedocs/gcc/Return-Address.html).

The same six C probe invocations run with statistics enabled and disabled:

- Actual foreign pthread execution with two and four lanes, plus held thieves
  that force nested owner execution on the original worker stack.
- Complete recursive results, multi-task joins, oversize-frame refusal,
  all 64 slots exhausted, fallback execution, and repeated slot reuse.
- A delayed thief spanning ring reuse; its stale ownership CAS must fail.
- The owner consuming a completed result and reusing its slot while the old
  executor is held before its final atomic waiter-metadata access.
- Zero and partial worker creation through injected creation refusal, using
  the same runtime cleanup paths as an actual `pthread_create` failure.

The five ABI invocations compile the same two wrapping-arithmetic computations
and check both complete scalar results. They run the recovered runtime with
`WF_WORKERS=0,1,2,4`, then link the same IR against only the floor and weak
sequential fallbacks. The observer requires actual started workers when the
pool is enabled; symbol checks require a strong acquisition implementation and
reject shared scheduler/completion text symbols. Workers zero and one select
the sequential clone; they do not measure unstolen publication/join overhead.

## Qualification and limits

On 2026-09-07, macOS 26.6.2 arm64 with Apple Clang 21.0.0 passed all eleven
invocations under ASan/UBSan and again under ThreadSanitizer, using the unchanged
compiler sources at investigation base `6cc00984415a39c507fa74897c9269b10beebfee`.
A scratch mutation replacing only the three atomic deque-cell accesses with
ordinary accesses failed the delayed-thief case with the expected TSan data
race between publish and steal. The test consumes the observed pointer before
CAS, since an unused load in the broken variant could otherwise be eliminated
on the losing path. These checks establish particular exercised behavior, not
a proof of all concurrent interleavings or a speed comparison.

The first Linux CI run at `6d6f84d9` failed the old local-address stack-distance
assertion. Enabling `detect_stack_use_after_return=1` reproduced that failure on
macOS. Observing the physical frame fixes this test while preserving detection;
the runtime code is unchanged. The
[corrected gate at `a5e0ca4f`](https://github.com/mbbill/Whitefoot/actions/runs/34152751414)
passed all twelve Linux/macOS jobs. Its Linux research job used Clang 18.1.3 on
x86-64 and ran all eleven recovery/ABI invocations under ASan/UBSan. That gate
predates the FIR addition below.

The sanitizers instrument the C runtime, floor and probe. Passing LLVM IR
directly to Clang does not retroactively add full frontend memory-access
instrumentation to generated WF functions. The ABI checks therefore do not
claim every generated WF access is sanitizer-instrumented.

Workers and storage live until process exit, and lane zero is attached to a
single owner thread. Repeated host calls must retain that owner; repeatedly
creating a new floor entry thread is unsupported by this control. There is no
pool shutdown qualification or reusable embedding API. The old environment
parser and split-budget code use requested counts even after partial startup;
tests use explicit valid settings and report actual started counts. Exhaustive
waiter ordering, exhaustion under deep helping, Linux TSan qualification, lifecycle
costs and performance measurement remain open. No timing conclusion can be
drawn from the local sanitizer runs.

## Variable-input causal FIR

`fir.wf` and one selected leaf file compute every output of a causal finite impulse response filter with
1 to 64 taps and K-1 initial history samples, oldest first. Tap zero multiplies
the current sample. Each output accumulates in ascending tap order using
distinct `fmul.strict` and `fadd.strict` operations. Samples preceding the block
come from its history; an empty block preserves that history. `fir_direct.wf`
retains the original one-output recurrence. `fir_lanes.wf` computes sixteen
independent outputs together, followed by a scalar tail. Both define the same
leaf contract and share `fir.wf`'s tree, recursion, accessors and command. The
compiler receives both files in one ordinary compilation; this is not separate
module compilation or a new ABI. Keep the two leaf files while their code
generation is compared; remove the superseded form when that question is settled.

The filter recursively divides the output interval and computes independently
owned tiles. Ordinary sibling calls expose the overlap; their joins complete
before the parent returns an owned tree. Source count/get/release functions let
the host check the whole result without adding a serial flattening pass to the
filter. A missing sample is an actual accessor result, checked as such by the
host. The WF history accessor reads the final K-1 samples of the held input
prefix, and its results become the history of subsequent real WF calls.

The shared command remains a complete WF program and checks a small impulse.
`fir_check.c` supplies the variable input cases and two independent C algorithms:
direct convolution and a stateful circular delay line. Each complete output
and next-history value must agree bit for bit. Hand-calculated nonzero-history,
contraction and tap-order witnesses supplement that agreement. `check-fir`,
also included in `check`, runs the standalone C oracle, the unchanged WF command
with zero/two workers, four recovered-host settings (`WF_WORKERS=0,1,2,4`), and
the weak sequential host: eight invocations in addition to the eleven above.

The host checks eleven configurations and 73 channels: tap counts 1/8/31/64,
lengths from zero through 65,537, empty and nonzero histories, unequal channel
lengths, impulse/step/alternating/deterministic inputs, decimal values and mixed
magnitudes. Every configured channel runs with tile sizes 1/17/257/65,536, and
again through ragged blocks interspersed with empty calls. The source admits a
prefix of at most 16,777,216 samples; the host checks that domain before crossing
its research interface. The 97,085 configured samples become 485,431 checked WF
sample hits, 290,955 history hits and 19,224 checked misses over 6,408 actual WF
calls per host run. Input/tap immutability and miss-result preservation are also
checked.

### Research host boundary

Ordinary emitted functions have internal linkage, so `fir_host.ll` appends tiny
typed wrappers in the same module before optimization. C passes scalar lengths
and pointers; the wrappers explicitly construct LLVM buffer descriptors and
call the emitted functions. Owned result trees stay opaque to C and are freed
by WF's ordinary release path. The host selects the parallel or sequential
clone once through the same `wf__par_pool_active` result used by the generated
command, under one persistent floor owner. Worker count is inspected after
execution because pool creation is lazy.

`OUT/fir.ll` and its ledger retain the original emitted module. `OUT/fir-host.ll`
contains only two command-symbol renames, to let the C host supply its entry,
followed by the checked-in wrappers. The filter code, proof erasure, frame layout
and probe attributes are not rewritten. The Makefile checks expected entry
signatures, actual recursive publication, strong runtime/floor linkage and exclusion
of shared scheduler/completion implementations. It retains `fir-command`,
`fir-oracle`, `fir`, `fir-sequential`, each host run's complete report and the
recovered symbol list. Every host invocation must print the full expected call
and result counts; an exit code alone cannot certify that the host ran. These wrappers
qualify this experiment's emitted revision; they are neither a source-visible
FFI nor evidence of a stable public ABI for separately compiled WF modules.

On the same local macOS/Apple Clang host, all eight FIR invocations pass under
ASan/UBSan and under TSan, including actual two/four-worker execution. Successful
steals were observed and are printed; their count is not a timing-dependent
correctness assertion. Scratch WF variants that fuse multiply/add or reverse
tap order fail the full-output oracle at the intended rounding witnesses.
The [FIR gate at `d13f6971`](https://github.com/mbbill/Whitefoot/actions/runs/34154018796)
passed all twelve Linux/macOS jobs. Its Linux research job used Clang 18.1.3
and ran the complete nineteen recovery/ABI/FIR invocations under ASan/UBSan.
That gate predates the native candidate and calibration caller below.

### Costs and gaps exposed by this program

The host invokes channels sequentially; WF parallelism currently exists inside
each channel across independent tiles. This does not yet qualify mixed-channel
co-scheduling or a full multistage signal pipeline. Initialized output buffers,
box/tree allocation and recursive dispatch remain in the computation. They
are charged to the filter-core interval below. Prefix construction, state
queries, result access and destruction are charged to its complete-cycle
interval. The scalar oracle is a correctness reference, not a qualified fastest
native filter.

The first range contract expressed `end - first <= limit`. Current S4 admits
that as an opaque requirement without projecting its arithmetic into the body
affine state. The accepted program instead states ordinary endpoint bounds,
including the absolute prefix limit, and proves its derived allocation/index
bounds in the body. This is a useful contract-expressiveness limitation to
investigate, not evidence that a required proof can be replaced by a runtime
guard. The active [specification](../../../spec/kernel-spec.md) still selects
those rules; this experiment changes neither acceptance nor the public ABI.

## Native FIR and first cost attribution

`fir_native.c` supplies four ordinary C kernels: a direct loop and 4/8/16
independent output accumulators. Every output retains ascending tap order,
separately rounded multiply/add and the same finite-data contract as WF. The
Clang hint disables only tap-loop vectorization/interleaving, leaving SLP across
independent outputs enabled. Native output is caller allocated; normal double
alignment suffices, tails need no padding, and empty calls still require valid
non-null object pointers. `fir_native.h` owns these preconditions.

`check-native` adds both ASan/UBSan (or the selected sanitizer) and ordinary
optimized-object qualification to the existing oracle. Each checks all four
forms over every K from 1 through 64, lane boundaries, ragged streams, canaries,
input immutability and rounding witnesses: 107,408 calls and 1,294,336 output
samples. The ordinary object is then reused unchanged by the timing binaries.
The older standalone and WF-host checks remain separate and retain their counts.

### WF output groups and exact optimized objects

Three WF configurations retain the same ownership and execution model:

| Kernel identifier | Leaf source | WF object optimization |
|---|---|---|
| `wf` | `fir_direct.wf` | Strict O3, the original control |
| `wf-lanes16` | `fir_lanes.wf` | Strict O3, default LLVM loop and SLP passes |
| `wf-lanes16-slp` | `fir_lanes.wf` | Same strict O3 plus `-fno-vectorize`; SLP stays enabled |

The grouped source writes a finite weighted invariant establishing that all
sixteen outputs lie inside the tile. Additional tail facts establish causal
input bounds. These proofs erase before lowering; there is no executable
overflow/bounds fallback or new arithmetic permission. Each output still
visits every tap in ascending order. The compiler, scheduler, function
contracts and opaque task-frame interface are unchanged. Only this experiment's
WF-object invocation selects the last configuration's flag; production
optimization policy is unchanged. The flag applies to that entire WF module,
including its accessors, and is not a promise to improve other programs.

The first local source screen tried four/eight/sixteen independent accumulators.
LLVM could vectorize tap products with ordered scalar additions before SLP saw
the independent output recurrences. The explicit pass control tests that
mechanism against ordinary O3 in the same matrix. It does not establish that
sixteen is optimal across workloads or targets. A compact local array-loop
variant also compiles, but this compiler revision materializes the full array
at dynamic indexed accesses; its generated lane loop contains repeated whole
array stores/reloads. A dereferenced exclusive legacy-array variant stops with
the explicit `RegionsAndBorrows` capability diagnostic. Neither observation
selects a language rejection or justifies adding a runtime proof guard.

`check-wf-objects` links each actual timed WF object to `fir_wf_check.c` and
the unchanged complete native oracle algorithms. Only the oracle translation
unit's four entry references are redirected to WF adapters with tile sizes
1/17/257/65536. Each run makes 107,408 real WF calls and checks 1,294,336 output
samples, 3,752,300 history values and 322,224 misses, including every K=1..64,
lane boundaries, ragged blocks, canaries and strict rounding witnesses.
For each of the three objects, one ordinary weak host and one recovered
four-worker host with the selected C sanitizer must report the expected world
and complete counts. The generated WF object remains byte-identical to the
timed object; C instrumentation does not add frontend sanitizer checks to WF
LLVM. The original streaming/history test still checks state from WF accessors
across successive calls separately. Keep this adapter while actual optimized
WF objects need the expanded oracle; consolidate it if the broader workload
harness replaces this research boundary.

### Static worker control

`fir_static.c` dispatches those same four kernels over balanced contiguous
partitions, with one persistent owner/caller and up to three helper pthreads.
The timed object is qualified unchanged, with only the oracle translation
unit's callee references renamed to the `fir_static_check.c` adapters. No
kernel or tail implementation is rebuilt or renamed for ordinary qualification.
`check-static`, required by `build-bench`, runs the full native oracle with
one/two/four actual lanes in both the selected sanitizer and the ordinary build,
plus sanitizer cases with four requested lanes and only one/two actually started.
Its hooks verify actual thread identity, partition coverage, waiting for a held
helper before returning borrowed storage, and twelve create/run/destroy cycles.
The fixed test watchdog detects a hung check; it is not a dispatch policy.

`bench-static` accepts `static-direct`, `static-lanes4`, `static-lanes8` and
`static-lanes16`. Requested widths zero/one mean one caller lane; two/four
require exactly that many actual lanes. Failed or partial startup is cleaned up
and rejects the comparison. Lazy creation and helper readiness are inside the
first core and cycle, including the empty diagnostic. Every dispatch waits for
all participating helpers before returning. Destruction runs after the batch
resource snapshot and sample output; its duration and actual capacity appear
in a required final report. The earlier sample PASS alone does not qualify a
static run. Peak RSS and batch CPU therefore exclude shutdown attribution.

Warm dispatch uses one mutex, work/done condition variables and per-helper
pending bits, with no spin or explicit heap allocation in that path. This is
a qualified static contender, not a selected optimal barrier/idle policy.
Small-job wakeup costs and idle CPU tradeoffs still need measurement. The pool
supports one dispatching owner and no concurrent or nested dispatch into the
same pool; it is not a general scheduler or a WF runtime interface. Keep its
source/header/checker while this control distinguishes native partitioning from
WF scheduling; consolidate or remove them when that comparison is superseded.

### Timing boundaries

`fir_bench.c` is an explicit C host for the actual FIR program. Each WF
configuration uses one object unchanged across weak/recovered/shared controls;
all ten executables share the **same `fir-native.o`**. Objects compile at O3
with strict FP and without LTO, with WF pass settings recorded separately in
`fir-wf-flags.txt`. `bench-weak` links the weak sequential path;
`bench-recovered` links the current-stack control; `bench-shared` links main's
compute scheduler units (`core.c`, `prim_host.c`, `entry.c`); `bench-static` adds
the qualified static object to the original weak host. The two grouped forms
use six additional binaries named `bench-<runtime>-<kernel>`; each host accepts
only its compiled WF kernel identifier, preventing a mislabeled WF comparison.
All retain the real
floor and the same research wrappers. The shared runtime is an architectural
control, not a native performance ceiling. This FIR panel's runtime statistics remain
enabled, including recovery's shared atomic steal counter; differences include
those costs and the different slot/idle policies. Fine-grained scheduler-only
claims must distinguish these objects from the statistics-disabled scalar
scheduler panel below.

Every invocation starts from the same supplied history and samples. It returns
all N outputs and K-1 next-history samples, then frees its temporary storage.
This is an independent block API screen, not a timed evolving stream/pipeline.

| Reading | Included work and limits |
|---|---|
| `core_ns` | Filter call through completion. WF includes initialized leaf buffers, owned tree allocation and joins. Native uses a preallocated flat output. These representations differ deliberately; this ratio alone is not scheduler overhead. |
| `cycle_ns` | Fresh prefix allocation/copy, output allocation, core, materialization of all outputs into a caller buffer, next-history copy/query and destruction. WF currently uses one tree accessor per sample; native copies a contiguous result. Includes the two inner clock calls. This is the current complete API form, not the best possible pipeline. |
| First versus warm | Call zero retains lazy runtime startup where needed. Subsequent calls reuse that pool. `entry_ns` measures main-to-floor-body entry and `select_ns` measures world selection separately. No row includes process loading, input generation or pool teardown; this is not total cold-process latency. |
| Batch resources | Wall/user/system CPU and context switches include every first/warm call and full bitwise verification between calls; peak RSS is process lifetime, including oracle/input/result storage. They are not core-only resources. Allocation counts, per-thread attribution and reserved stack bytes remain unmeasured. |

The `CLOCK_MONOTONIC_RAW` timer surrounds individual calls, so very short
`core_ns` values include a material clock/call floor. Every process reports the
clock's stated resolution and minimum/positive minimum of 1,000 empty clock
pairs; these diagnostics are never subtracted. Full output and history
verification runs outside each interval and warms data between calls. No timed
loop prints output. Each call is retained in a TSV; the summary reports warm
per-process means and ranges, not confidence intervals or reliable tail
percentiles. Tiny/empty rows diagnose overhead and do not establish nanosecond
kernel rankings.

`check-bench`, included by canonical `check`, runs 111 processes: three empty,
short and uneven cells across four native forms, weak WF and recovered/shared
WF with requested widths 0/2/4 for all three WF kernels, and all four static forms at those widths.
It checks the exact full report, every result,
and the complete ordered row sequence and configuration keys;
there are no elapsed-time assertions. Alongside the previous nineteen, two
native, eight static and six optimized-WF-object checks, the experiment now runs
146 invocations for the runtime/FIR panel. The record panel below adds 23,
giving 169 invocations in the complete experiment. The host metadata read
may require permission in a local sandbox; native CI has that access.

Run a calibration explicitly, with a target appropriate to the measured host:

```sh
make -C research/experiments/compute-runtime bench-calibrate \
  OUT=/tmp/wf-compute-fir BENCH_ARCH=-march=native
```

On the local M1, use `BENCH_ARCH=-mcpu=apple-m1`. `OUT` must be absolute. The
caller builds and qualifies first, then runs one process at a time. Its default
five passes rotate/reverse configuration order over K={3,15,64},
N={33,4097,262144}, plus an empty diagnostic. WF tiles are 257/4096/65536;
requested WF and static widths are 0 and, when available, 2/4. Static partitioning
does not use the WF tile parameter. On four allowed logical CPUs this gives
79 configurations per cell and 3,950 processes over five passes. The three WF
forms participate in the same rotated/reversed order, rather than separate
form-by-form timing batches. Every configuration and losing
row remains. `ROUNDS` can shorten a smoke/calibration replay, never manufacture
confirmation. Results go to a fresh directory recorded in `OUT/last-calibrate-path.txt`;
an existing directory is not overwritten. Exact objects, assembly, tool flags,
source hashes, host topology and CPU masks accompany the raw data. A missing
cgroup quota or cpuset file is explicitly unqualified, not evidence of no limit.

[compute-bench](../../../.github/workflows/compute-bench.yml) runs this screen
on a GitHub-hosted Linux runner, restricting all children to the same mask of
at most four allowed logical CPUs. VM interference, frequency and per-thread
placement are uncontrolled; this is same-host calibration, not dedicated-host
confirmation. The job artifact preserves the raw calls, failures and objects.
Held-out K={1,7,31,63}, N={1,65,65539} and independent input families are reserved
for a later frozen-candidate confirmation. Cache-cold working sets and dynamic
library references remain separate next experiments.
Keep the native/bench files while this FIR comparison is maintained; consolidate
them into a broader workload harness or remove them when it supersedes this API.

### First Linux calibration, before static workers

Revision `9e66c2557685bff843c1c2c00400440fd58cf04d` ran five passes in
[Linux CI](https://github.com/mbbill/Whitefoot/actions/runs/34157403518), with
Clang 18.1.3, strict O3 `-march=native`, and a four-logical-CPU mask on a VM
reporting AMD EPYC 9V74 and two SMT2 cores. These are reported virtual topology,
not dedicated physical cores. The run did not capture a readable CPU quota.
All 1,250 process summaries are retained in
[`fir-calibration-9e66c255.tsv`](fir-calibration-9e66c255.tsv); 183,500 raw calls,
source/host manifests, exact objects and assembly are in artifact `10031465699`
(`compute-fir-linux`, expires 2026-12-06). The downloaded ZIP's SHA-256 is
`10a7f821ddaa597fbd8dcb86f0c4e534f2e18a61cd8bc34e1999474280f0d6e0`.
The compiler executable's hash was captured, but its bytes are not in that
artifact. Keep this complete summary with the interpretation while the dated
comparison remains useful; remove both if the investigation drops this evidence.

For K=64, N=262,144, and WF tile=4,096, medians of the five per-process warm
means were as follows. Native tile settings are ignored.

| Form | Core, microseconds | Complete cycle, microseconds |
|---|---:|---:|
| C direct, one caller | 10,687.451 | 10,834.823 |
| C 16-output lanes, one caller | 1,142.274 | 1,275.530 |
| WF weak sequential | 10,494.536 | 12,381.825 |
| WF recovered, four requested lanes | 4,160.929 | 6,209.572 |
| WF shared, four requested lanes | 4,725.145 | 7,897.166 |

Across paired passes, direct/native-lanes16 core ratios ranged 8.44–9.41
(median 9.36); recovered-four/native-lanes16 ranged 3.33–3.65 (median 3.59).
The retained Linux assembly uses scalar `vmulsd/vaddsd` in direct C and WF,
and `ymm vmulpd/vaddpd` without FMA in native output-lane forms. Independent
accumulators also reduce dependence-chain pressure; vector width alone does
not explain the whole ratio. Native flat-output and WF allocating-tree core
boundaries differ, and the cycle adds distinct materialization costs. These
results motivate improving WF kernel expression/code generation before drawing
scheduler-ceiling conclusions. They do not measure static workers or establish
held-out confirmation. All twelve Linux/macOS jobs of the matching
[canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34157403358)
passed; that revision ran 54 experiment invocations, before the static addition.

### Linux static control, before WF output groups

Revision `7ad3ec5f4df2e8d0842a6f5a0ff34f954f7c7eaa` passed all twelve jobs of
its [canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34160395286)
and the [five-pass static calibration](https://github.com/mbbill/Whitefoot/actions/runs/34160395321).
The full artifact audit verified all 1,850 processes and 271,580 raw calls,
including exact capacity and completed shutdown in all 600 static processes.
This was another EPYC 9V74 VM reporting two SMT2 cores and mask 0-3, with
Clang 18.1.3 and strict O3 `-march=native`. Quota and individual thread placement
remain unqualified. The run is not pooled with the preceding host population.

At K=64, N=262,144 and WF tile=4,096, median core/cycle microseconds were
1,142.731/1,301.326 for single-caller C lanes16;
1,093.803/1,226.703 for static lanes16 with two actual lanes;
855.686/999.069 for static lanes16 with four;
4,292.095/6,439.568 for recovered WF with four; and
4,691.544/7,638.750 for shared WF with four. Static four lanes beat two in only
three of five paired passes; a lower median does not establish consistent
monotonic scaling. The different allocation/materialization boundaries above
still apply. This result predates the grouped WF configurations.

Artifact `10032428800` (`compute-fir-linux`, expires 2026-12-06) contains every
raw row, source/host manifests and exact objects. Its 6,954,818-byte ZIP has
SHA-256 `ea96e5ad22cc3d04fbe69c021f94f9b06ff1b9397cdaf7acea4cf460f57fb138`.
The independent audit rehashed seventeen sources and eight retained IR/object/
executables and verified all eight static qualification cases and 69 smokes.
Compiler executable bytes are not uploaded; its hash is recorded. Keep this
dated interpretation while the control is useful, and remove it if superseded
evidence makes this comparison unnecessary.

### Linux grouped WF calibration

At `bf985ac42cbced4b424e3c8fec3a2d5c1f24c932`, the
[canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34163120182)
passed all twelve jobs and the
[grouped WF calibration](https://github.com/mbbill/Whitefoot/actions/runs/34163120195)
passed 3,950 processes / 579,860 calls. The independently audited artifact is
`10033306398`, ZIP SHA-256
`7f48e3d24bf681856b406fa653cd9749fab546be0396a41e2eb6465b1690d816`.
It retains all raw rows; twenty frozen-source hashes and seventeen retained
IR/object/executable hashes match. Compiler executable bytes remain unretained.

On that EPYC 9V74 VM (two SMT2 cores, mask 0-3, Clang 18.1.3, strict O3
`-march=native`), K=64/N=262,144/tile=4,096 weak WF core/cycle medians changed
from 10,469.007/12,337.729 to 1,208.903/3,169.144 microseconds with output groups.
Native lanes16 was 1,149.024/1,272.264. Recovered grouped WF with four lanes was
711.882/2,663.706, versus static C's 843.954/982.156: the core won all five
paired passes and the complete cycle lost all five. Batch CPU medians were
28.164 versus 13.879 ms for one first plus four warm calls and checks.
These boundaries do not isolate scheduler performance.

Both Linux grouped WF objects are byte-identical, so their pass-setting timing
differences cannot demonstrate a vectorizer effect. Their leaf uses separate
packed YMM multiply/add and scalar tails. The effect is also workload-dependent:
at K=3/N=262,144/tile=257, weak grouped WF reduced core from 659.797 to 215.532
microseconds while cycle increased from 3,504.390 to 3,759.118. Quota, individual
thread placement and held-out confirmation remain unqualified; this population
is not pooled with the earlier screens.

## Variable-length UTF-8 record batches

`records.wf` validates complete, independently framed UTF-8 records and returns
one Unicode scalar count per record in input order. Its byte grammar follows
[RFC 3629](https://www.rfc-editor.org/rfc/rfc3629.html): empty records, NUL and
noncharacters are valid; overlong encodings, surrogates, values above U+10FFFF
and truncated sequences are invalid. Adjacent offsets describe each record.
Invalid UTF-8 returns `UINT64_MAX`; descending or out-of-input ranges return
`UINT64_MAX-1`. Both differ from every admitted scalar count. A record cannot
borrow continuation bytes from its neighbor.

The ordinary source loop writes independent positions of one initialized
`buffer<u64>`. The compiler stages the map with six captures and a 112-byte
frame, retaining current-stack join/help/steal. A header invariant proves the
leaf's counter bound; every range/index obligation remains static after the
source handles genuinely malformed metadata. The current compiler actualizes
the direct result-index form; an equivalent `record-first` target expression
was declined. This is a compiler actualization limitation, not a language rule.

`records_host.ll` explicitly constructs input descriptors and exports the typed
output buffer's element pointer/count to the C research host. C reads the
initialized scalar elements and releases ownership through the WF release
function. It retains both inputs through the completed call. This is a bounded
experimental adapter, not a production ABI or a separate-compilation design.
No compiler, runtime interface or execution semantics change in this addition.

`records_native.c` supplies a matching state machine and a single-pass native
word candidate: two bounded unaligned eight-byte reads skip sixteen ASCII
bytes at a code-point boundary; non-ASCII sequences follow explicit byte
rules. `records_oracle.c` independently decodes integer code points to check minimum
encoding lengths and scalar ranges. Each qualifier checks 4,595,603 leaf inputs
and 342 batches / 1,821,070 per-record results, including complete scalar
encodings, nonempty truncations, all one/two-byte combinations, selected longer
extensions, invalid metadata, nonzero range starts, empty/uneven batches and
long-record outliers. Canary comparisons detect input writes, not every possible
out-of-slice read; three/four-byte combinations are not exhaustive.

`check-records` runs the original WF command at widths zero/four, ordinary weak
and recovered full qualifiers, a C-sanitized qualifier, and thirty timing-host
smokes. Both timing hosts reuse the exact qualified O3 WF and native objects.
The record builds explicitly disable loop/SLP vectorization and LTO, including
the WF LLVM object. The driver rejects a flags manifest missing those controls;
assembly remains available to inspect the generated bodies. Earlier record
artifacts without these explicit flags are a separate historical cohort.
The C-sanitized qualifier also instruments a separate native object; ordinary
WF LLVM does not thereby acquire frontend sanitizer instrumentation. Keep these
record source/adapter/native/host/driver files while this workload needs the
experiment; consolidate them if a shared workload harness supersedes their role.

Run `make -C research/experiments/compute-runtime records-calibrate OUT=/path/to/output
BENCH_ARCH=-march=native` on the selected Linux host (one shell line). The driver
records five rotating/reversing passes across twenty-five cells and six controls:
native state/word, weak WF, and recovered WF at requested zero/two/four lanes.
The 750 processes cover ASCII, mixed Unicode, early/late invalid bytes and
skewed lengths at one to 65,536 records. Every output is checked after each
timed call. Raw calls, per-process warm means/ranges, code/flags and host
manifests are retained. The CI matrix gives FIR and records separate hosts and
artifacts; compare controls within one workload's host, not between those jobs.

Core includes result allocation and processing: WF initializes its buffer;
native allocates uninitialized output and writes every element. Cycle adds a
complete flat result copy and release. Prepared input bytes/offsets and the
consumer buffer are outside both intervals. The first call includes lazy pool
startup; batch CPU/context switches include first/warm calls, printing and
between-call checks; RSS is process lifetime. Empty/tiny clocks are diagnostics,
not rankings. Offered input bytes are not a claim of bytes actually scanned by
an early-invalid validator.

Parallel-world selection does not imply that a small batch starts workers.
The current estimated map weight is 812 and the recovered split policy admits
no split below 2,956 records. Reports retain actual capacity and steals; a
partial started pool is rejected, and the full qualifier requires four lanes.
The compiler estimates nested loops with a fixed factor of 16 and substitutes
callee estimates to three levels; it does not observe the record's scanned
length or data-dependent early exit.
The 256-record, maximum-length 65,536 cells expose expensive batches below
that threshold; valid Unicode and early-invalid versions also run in the
smoke matrix. At 4,097 records the current budget admits only two chunks, even
when the pool has four lanes. Pool capacity alone is not useful parallel width.
The scheduler panel's manually partitioned C adapter bypasses this estimate,
so it cannot establish that the compiled WF program exploits the same work.
The [`d6bf6c08` Linux record run](https://github.com/mbbill/Whitefoot/actions/runs/34189375566)
confirms this gap across the expanded scalar cohort: 750 processes / 21,750
calls and 52,748,250 checked output positions, plus 30 smoke processes. On its
EPYC 7763 VM (two physical cores, four SMT logical CPUs, mask 0-3), long Unicode
256-record warm core medians were 7,869.056 / 7,910.887 / 7,931.259 microseconds
for recovered WF at requested zero/two/four lanes. Every two/four-lane sample
in that cell reported no started pool and no steals. Native state/word anchors
were 7,205.072 / 5,230.540 microseconds, including their output allocation.
These are end-to-end record-core boundaries, not identical-kernel scheduler
costs. At 4,097 short Unicode records the recovered two/four-lane medians were
330.640 / 330.294 microseconds; the zero-lane process means ranged from 294.563
to 621.010, so its 427.756 median is not a stable serial speedup denominator.
This record runner is separate from the scheduler runner below; do not pool
their timings. Artifact `compute-records-linux`, ID `10041686914`, ZIP SHA-256
`fd3b85ea1aadc4b3b1565cda8c9652cdb434153fe1cb2982f12c69b8062a4f91`, retains
all rows. Independent auditing matched every summary, input byte count, eleven
source hashes and five retained object/executable hashes. Compiler executable
bytes are recorded by hash but unretained. Actual leaf disassembly is scalar;
ABI frame copies may use vector registers. Quota and per-thread placement
remain unqualified, and batch resources include checks and output reporting.
The two native kernels are initial scalar anchors. This end-to-end panel does
not establish a native frontier or full application throughput. SIMD work is
parked. The separate scheduler comparison below supplies matched static/dynamic
controls; held-out inputs and dedicated-core measurements remain open.

### Isolating the static cost budget

`records-budget` compares three runtime budget policies in one executable with
the same emitted WF object. The research host selects `WF_BUDGET_CONTROL=cost`
or `capacity` or `team` before floor entry and before any worker can read the flag.
`cost` preserves estimated affordability; `capacity` bypasses only that test.
`team` also caps requested terminal chunks at the requested lane count.
All three retain the requested lane limit, span cap, queue backpressure, binary
budget-depth descent and acquire-failure fallback. At width four the capacity
policy allows at most 64 terminal chunks; it does not create one job per record.
The ordinary runtime and record hosts do not contain this control unless built
with `WF_COMPUTE_BUDGET_CONTROL`. No compiler rule, source signature or default
scheduling policy changes.

`WF_RECORD_CHECK_CADENCE=call` (the default) checks input immutability after
every call; `batch` checks it at batch end. Both modes check every output after
every call and perform a final input check. This control changes memory traffic,
cache state and worker opportunity together; it does not isolate wakeup cost.
The ordinary qualifiers retain their per-call input checks under both settings.
Batch-only input checks cannot detect transient mutations restored before the
final check and do not replace the ordinary qualifiers.

The budget host records each interval from the previous completed cycle to
the next core start, then prints the gaps after the batch resource snapshot.
Output copying/checking and per-call report printing remain in both modes, so
`batch` is not uninterrupted dispatch. The final input scan is inside batch
wall/CPU accounting and outside the recorded inter-call gaps. Gap bookkeeping
is common to both controls; the budget image is separate from ordinary hosts.

`check-records-budget` retains `check-records`, then runs every policy/cadence
combination through the full leaf/batch qualifier in ordinary and C-sanitized
images: twelve full qualifiers, twelve maximum-repetition boundary smokes, and
ninety policy/cadence/width/input smoke processes. It is included in the experiment's canonical
`check` and in the Linux records CI job. Statistics remain enabled, and the
emitted WF object remains ordinary LLVM rather than frontend-sanitizer code.

```sh
make -C research/experiments/compute-runtime check-records-budget OUT=/tmp/wf-records
OUT=/tmp/wf-records RESULTS=/tmp/wf-records/budget-calibration \
  sh research/experiments/compute-runtime/records-bench.sh budget-calibrate
```

The calibration interleaves all three policies and both input-check cadences
at requested widths zero/two/four
over thirteen cells: five input shapes at 256 long and 4,097 shorter records,
two tiny 33-record inputs and an empty input. Five rotating/reversed passes
with eight warm calls plus a separate first call produce 1,170 processes / 10,530
calls and 9,360 gaps. Policy and cadence are part of each raw report, filename,
configuration and summary. The budget summary has 21 fields: the original
sixteen timing fields, policy, cadence, then mean/minimum/maximum gap in ns.
The driver checks gap order/count, unique metadata, actual worker capacity,
and that total cycles plus gaps fit within batch wall time;
the same executable hash is checked before and after the panel. Width zero
is the neutral control, not a competing parallel runtime. Keep early-invalid
and skewed inputs beside expensive inputs when evaluating the cost of removing
the threshold. Core includes output allocation/initialization and computation;
cycle also copies and releases the result. Batch CPU includes checks and
printing and is not a per-call scheduling-cost measurement. A budget-policy
speedup over WF's own serial execution does not establish a native frontier.

The maintained M1 cadence screen independently verified 1,170 processes /
10,530 calls / 17,683,110 output positions / 9,360 gaps, all 21-field summaries,
execution order and nineteen manifest hashes. The twelve full qualifiers and
twelve maximum-repetition reports pass. Placement is unfixed; statistics remain
enabled. This cohort is separate from the earlier scratch controls and the
two-policy screens below. The following requested-width-four core times are
medians of five process warm means, with eight warm calls per process:

| Input and policy | Per-call input check | Batch-end input check | Paired batch/call ratio [range] |
| --- | ---: | ---: | ---: |
| Early-invalid256, max length 65,536, team | 12.776 us | 2.687 us | 0.210 [0.171--0.348] |
| Early-invalid256, max length 65,536, cost | 5.406 us | 1.917 us | 0.405 [0.154--0.604] |
| Early-invalid4097, max length 128, team | 6.812 us | 11.187 us | 2.190 [1.520--2.748] |
| Skew4097, max length 128, team | 4.885 us | 10.130 us | 2.246 [1.873--3.235] |

Every pair improves in the first two rows and regresses in the other two.
The first row's median process-mean gap falls from 301.792 to 0.640 us.
The second row has zero actual lanes under both cadences, so these changes
cannot be attributed wholly to worker wakeups. Team's long Unicode256 core
is slower than capacity in every pair under both cadences. With batch-end
checks, capacity takes 1.886 ms and team 2.006 ms; their paired team/capacity
ratio is 1.064 [1.032--1.116]. Neither cadence nor chunk cap is a uniform winner.
This is evidence about benchmark conditions and split policy, not a speedup
in the ordinary runtime. The shared executable SHA-256 is
`ee4cd44ecc9d8fa94b8d0498b4c270b3a580f3dfb40ac6b78022eec820b66982`; the WF
object SHA-256 is
`766c426f8e02d8bdfc624eed8d9ce8f4cf4ee7f2a6407c27a7750bbae2983543`.

The [Linux cadence run at `32883518`](https://github.com/mbbill/Whitefoot/actions/runs/34210144231)
independently verifies the same 1,170-process / 10,530-call / 9,360-gap inventory,
all outputs, summaries, order, capacities and twelve qualifier/boundary pairs.
Eighteen source/artifact hashes match the exact revision; the compiler hash is
recorded, but its executable is absent. This EPYC 7763 VM has two physical cores
and four SMT logical CPUs, mask 0--3; CPU quota and per-worker placement remain
unqualified. Requested-width-four team timings differ from M1:

| Input | Per-call input check | Batch-end input check | Paired batch/call ratio [range] |
| --- | ---: | ---: | ---: |
| Early-invalid256, maximum length 65,536 | 11.027 us | 1.297 us | 0.118 [0.081--0.131] |
| Early-invalid4097, maximum length 128 | 15.161 us | 12.357 us | 0.818 [0.528--0.956] |
| Skew4097, maximum length 128 | 10.307 us | 9.006 us | 0.906 [0.452--0.989] |

All five pairs improve in each row. The latter two regress on M1, so that
regression does not generalize to this Linux host. Cost-policy early-invalid256
also improves while reporting zero actual lanes (1.785 to 0.665 us); wakeup
alone still cannot explain the cadence effect. Small width-zero controls vary
substantially even though they take the same serial path: early-invalid4097
capacity/cost spans 0.548--1.649 with per-call checks. Long Unicode width-zero
controls are near neutral. These are separate host cohorts, not a revision
speedup or an operating-system-only causal comparison. Artifact `10049530617`
has ZIP SHA-256
`c25bca1e3202c9b87c805f7ff0b0a76862cd5c561e4be9c2ec598a523dc22eef`.

The earlier two-policy M1 screen, before the cadence extension, independently
checked all 390 processes / 3,510
calls, 5,894,370 output positions and nineteen manifest hash entries. Selected
requested-width-four core timings below are medians of five process warm means;
the paired ratios compare matching passes and need not equal the ratio of the
displayed medians. No scratch-prototype samples are pooled into this screen.

| Input | Cost policy | Capacity policy | Paired capacity/cost ratio [range] |
| --- | ---: | ---: | ---: |
| 256 Unicode records, maximum length 65,536 | 7.077 ms | 1.984 ms | 0.279 [0.272--0.333] |
| 4,097 Unicode records, maximum length 128 | 132.599 us | 70.354 us | 0.530 [0.528--0.548] |
| 256 early-invalid records, maximum length 65,536 | 5.834 us | 11.135 us | 2.642 [1.565--3.878] |
| 256 skewed records, maximum length 65,536 | 22.172 us | 26.047 us | 1.170 [1.142--1.282] |
| 33 early-invalid records, maximum length 64 | 0.130 us | 3.016 us | 22.275 [21.231--39.789] |

Every pair improves for the two Unicode rows and regresses for the other three.
The long 256-record cost runs report zero started lanes despite requesting
two/four; capacity reports two/four. Capacity's paired long-input speedups over
its own width-zero control are 1.914x at two and 3.553x at four. At width zero,
both policies stay sequential and the long-input capacity/cost ratio is 1.000
[0.993--1.008]. Tiny absolute times approach clock resolution; preserve their
regressions without treating their ratios as precise scheduler instruction costs.
The shared executable SHA-256 is
`6d68093b4ca9ecedf135e724cd1a1b99cc6af7164daf709e7c58b52a2f33e80d`; the WF
object SHA-256 is
`766c426f8e02d8bdfc624eed8d9ce8f4cf4ee7f2a6407c27a7750bbae2983543`.
This exposes a cost-selection problem, not a reason to make capacity the
default.

The [two-policy Linux records run at `8a6e5c53`](https://github.com/mbbill/Whitefoot/actions/runs/34206401837)
qualifies that earlier policy tradeoff on an EPYC 7763 VM with two physical cores,
four SMT logical CPUs and process mask 0--3. CPU quota and individual worker
placement remain unqualified. Its 390 processes / 3,510 calls / 5,894,370 output
positions, policy/capacity reports, summaries and execution order were independently
checked. Eighteen source/artifact hashes match; the manifest records the compiler
hash, but its executable is not retained in this artifact. All four full
ordinary/C-sanitized policy qualification reports pass.

| Input, requested width four | Cost policy | Capacity policy | Paired capacity/cost ratio [range] |
| --- | ---: | ---: | ---: |
| 256 Unicode records, maximum length 65,536 | 8.222 ms | 4.917 ms | 0.599 [0.588--0.652] |
| 256 early-invalid records, maximum length 65,536 | 1.026 us | 14.212 us | 13.354 [10.852--16.234] |
| 256 skewed records, maximum length 65,536 | 28.034 us | 55.848 us | 1.978 [1.086--2.595] |

Every pair improves for long Unicode and regresses for the other two rows.
Long Unicode again starts zero lanes under cost and two/four under capacity.
Capacity's paired speedups over its own width-zero control are 1.625x at two
and 1.664x at four; these four logical CPUs are not four physical cores.
The long-input width-zero capacity/cost ratio is 1.001 [0.997--1.004]. Several
short-input width-zero controls vary substantially despite both policies taking
the same serial path: for early-invalid256 the range is 0.665--1.713. Preserve
the repeated regression direction without treating its exact magnitude as an
isolated scheduler cost. Input verification between calls also scans the full
input; this panel measures that call cadence, not uninterrupted dispatch.

Artifact `10047998009` (`compute-records-linux`) has ZIP SHA-256
`3859730b68752c7f2ecb4edd455423e9c033ec6e08732b61f2e4b9998fb5ff33`.
The shared policy executable SHA-256 is
`dd7348dcdbf19ce48c332b18ec335cf2c3274e9a2d872428c0066eab856b0ee0`; its WF
object SHA-256 is
`dd562214489105670e7d38dfe89be1ef16cd47d472196b35c88b9ad55e907043`.
This Linux cohort and the M1 cohort are separate host measurements, not a
revision comparison or evidence for a universal replacement policy.

### Events from compiled WF record calls

`records-events` links the same emitted WF object to the existing `wf-2`
instrumented runtime and the budget/cadence host. `check-records-events` retains
all record and budget checks, then adds twelve ordinary/C-sanitized event-image
qualifiers, twelve maximum-repetition boundary smokes and ninety event smoke
processes. The experiment's canonical `check` and Linux records job call it.
Use `records-bench.sh events-calibrate` with the same `OUT` and a fresh `RESULTS`
directory after that check to collect the same 1,170-process policy/cadence
matrix. This is a separate diagnostic cohort, not production timing.

Event collection requires an explicit `WF_WORKERS` value at most four, including
for standalone host invocations. Four fixed lane banks are sampled before core
start and after core end; the sequential snapshot reads enclose a wider interval
than the core clock. Post-snapshot reads and storage are inside the cycle clock.
Snapshot allocation occurs before batch accounting, and rendering occurs after
the resource snapshot. Instrumentation changes execution even though getter
reads are outside the core clock. Do not price counter overhead or rank runtimes
using this image's elapsed times.

The raw header `# record_events schema=wf-2 banks=4 counters=24` precedes the
ordinary report. After the gap rows, each call has four ordered
`# record_event call=N lane=L` rows with 24 tab-separated values in the order
declared by `runtime_events.h`. Values are per-call differences of cumulative
atomic counters; the banks are not sampled simultaneously. The timing summary
retains 21 columns, with runtime `events` distinguishing it from `budget`.

After every fully joined call, aggregate published jobs must equal local pops
plus successful steals, run starts, run ends and joins. Successful steal origins
partition successful steals per bank. An active parallel cell must publish at
least one job, and inactive banks must remain zero. These are published frame
jobs, not record counts or every sequential terminal chunk: the caller's right
branch is unqueued, and acquire-failure fallback may recurse without publishing.
Search attempts, completion-tail signals and idle events can continue around
the snapshot boundaries. Their deltas are retained without attempt-partition
or park/resume equality requirements and are not OS context-switch counts or
CPU-time shares. The ordinary runtime, emitted WF code and public ABI are
unchanged by this diagnostic.

The maintained M1 diagnostic panel over `32883518` contains 1,170 processes,
10,530 calls, 42,120 bank rows and 1,010,880 counter deltas; all stable job
equalities and twenty manifest hashes passed independent reconstruction.
Its event image SHA256 is
`83dedcb97b50137f99879969085c65ff152a9237331ab30740694112bdaa5701`.
Placement is unfixed; this cohort is separate from the cadence timings without
event instrumentation above. At requested width four and 4,097 records, cost publishes one
job per call, team three and capacity sixty-three. At 256 records, cost
publishes zero while team and capacity retain three and sixty-three. Every
observed slot-refusal delta is zero; affordability refusals are not counted
by that event.

For early-invalid4,097 under team, every warm call steals all three published
jobs under both cadences. Yet batch-end input checking increases idle-origin
steal attempts in all five paired passes: median batch/call ratio 3.493
[3.082, 4.182]. Skew4,097/team also increases in all five pairs, ratio 2.812
[1.622, 4.503]. For early-invalid4,097/capacity, the median of five process
warm means is 55.25 local pops and 7.75 successful steals under either
cadence, despite sixty-three published jobs. Extra queue jobs therefore do
not imply proportionally more work transferred to other workers. These
counts identify search traffic and local execution as measurable distinctions;
they do not establish their CPU cost or explain the opposite M1/Linux timing
directions. No production performance improvement follows from event timings.

The [Linux event run at `63687369`](https://github.com/mbbill/Whitefoot/actions/runs/34213463345)
independently verifies the same 1,170 processes, 10,530 calls, 42,120 bank rows
and 1,010,880 deltas, plus nineteen source/artifact hashes. The compiler binary
is again absent. This runner is an EPYC **9V74**, unlike the prior 7763 cadence
host; it exposes two physical cores/four SMT logical CPUs, mask 0--3, with
quota and per-worker placement unqualified. Artifact `10050888685` has ZIP
SHA256 `d3422dea5f31677bc60f23f4443c010c28472727f994bc08bc72185310cefd10`.
Published-job counts and zero observed slot refusals reproduce the M1 panel.
The search behavior does not: early-invalid4,097/team idle-origin attempts
decrease in all five batch-check pairs, ratio 0.660 [0.093, 0.812]; skew4,097/team
decreases in three of five, ratio 0.549 [0, 1.253]. Early-invalid256 shifts
toward more stolen jobs under batch checks on both hosts. These are separate
instrumented cohorts, not a timing comparison or a universal search rule.

### Idle search spacing control

A bounded M1 control over `63687369` tests whether fewer idle searches improve
the compiled WF record workload. One executable selects zero or sixteen ARM
`__yield()` hints after each failed idle search during the initial 4,096-scan
phase. The selector is read before worker creation and stays immutable.
Join/help, deque ownership, completion, wakeup, split policy and fallback are
unchanged. This is a processor hint, not `sched_yield` or a WF language feature.
The same scan budget remains, so spacing also changes time to park; this is
not an isolated measurement of the cost of a search.

Timing and detailed events use separate images, each with 320 processes and
5,440 calls: team/capacity, call/batch input checks, requested widths zero/four,
zero/sixteen hints, four cells and five rotating passes with sixteen warm
calls plus a first call. Cells are early-invalid256 and Unicode256 with
maximum length 65,536, and early-invalid4,097 and skew4,097 with maximum length
128; seed 828219. Both modes share one image within a panel and the same emitted
WF object. Statistics are enabled; vectorization/LTO are disabled; placement
is unfixed. Forty-eight qualification processes cover protocol interleavings,
ASan/UBSan and TSan full record qualifiers, and repetition boundaries. Sanitizers
cover C components, not the emitted WF object. Exact metadata and inactive
steal validation were strengthened after measurement and replayed on retained
raw data; executable and raw results were unchanged.

With detailed events disabled, long Unicode/capacity/batch core medians improve
from 1.791 to 1.755 ms, paired ratio 0.9798 [0.9674, 0.9996], all five pairs lower.
The other fifteen width-four core comparisons have mixed directions. Under
per-call checks, early-invalid256/team batch CPU increases in every pair:
median 8.499 to 9.484 ms, paired increases 0.641--1.483 ms. Skew4,097/team/call
batch CPU falls 0.308 to 0.249 ms in every pair, but core time worsens in four
of five. Batch CPU includes the first call, checks and printing; it is not
scheduler CPU cost. Width-zero short controls remain noisy despite the selector
having no worker on which to act.

The event panel confirms fewer searches in selected cells: early-invalid4,097/
team/call retains three stolen jobs per warm call while idle-origin attempts
fall in all five pairs, ratio 0.800 [0.554, 0.828]. Its timing panel does not
show a consistent core improvement. Fewer searches alone therefore do not
select this fixed spacing policy; no default change is promoted. The timing
image SHA256 is
`a97bd86631bb2e9d27715993c11ca3e14a9be60ee5a500f41c2af9a5f3855869`;
the event image is
`571f8aaf2e82e5dbfcad38805c2a0236c6d700849063dc903c454b2caeb558b8`.

## Mandelbrot point rendering

`mandelbrot.wf` adds irregular floating-point work to the WF suite. A dependent
complex recurrence remains sequential within each point, while the ordinary
outer loop becomes an independent output map. The result is the first iteration
whose squared magnitude exceeds four, capped by the supplied limit; iteration
zero tests zero. All arithmetic uses separate strict binary64 operations.
Coordinates and iteration limits are runtime inputs. This finite rounded
recurrence is the comparison contract: analytic interior shortcuts, different
rounding, FMA contraction and SIMD are outside this scalar scheduler panel.

### Ordinary CLI Mandelbrot

`make mandelbrot-command-screen OUT=<fresh-absolute-directory>` builds actual
executables with `whitefootc --par --no-vectorize` and
`--no-overlap --no-vectorize`, using the maintained shared runtime and default
publication policy. No emitted symbol is renamed or runtime implementation
substituted. `mandelbrot_command.wf` adds runtime arguments, input generation,
repeated rendering and an ordered 64-bit output digest to the existing kernel.
The no-argument command preserves the original three-point smoke. The command
arguments are `SHAPE COUNT LIMIT REPETITIONS SEED EXPECTED_CHECKSUM`.

`mandelbrot_command.cpp` supplies independent native commands: a strict scalar
serial kernel, a persistent static-partition pool with caller participation,
and a volatile binary64 recurrence oracle checked against known orbits. Oracle
mode also checks the optimized native point kernel pointwise; timed WF and
native commands compare the ordered digest, which is not a collision-free
proof. Input generation, zero-initialized output allocation, digest work,
startup and shutdown are inside the whole-process measurement on both sides.
The one-lane native path renders directly, without per-batch pool atomics;
its one-time team construction/destruction remains charged. The static pool spins and
yields while idle; its CPU cost is charged. It is a regular-work reference,
not a dynamic scheduling ceiling for skew. SIMD, reassociation, contraction,
fast math and LTO are disabled for the comparison.

`command_runner.c` measures process wall time, child user/system CPU and peak
memory on POSIX and Windows, plus POSIX context switches (Windows reports NA).
`mandelbrot-command.sh` owns build, correctness and measurement; its AWK
summarizer consumes the complete process matrix. These experiment-only tools
are used by the Makefile and five-target `ordinary-command` CI jobs; remove
them with this workload or consolidate when another maintained panel replaces
the same measurement. They implement no WF compiler/runtime capability.

The correctness target `check-mandelbrot-command` is part of canonical research
checks: 48 input cases cover seven distributions, zero/odd/larger counts,
iteration limits, repeated calls and seeds including u64 maximum, with WF
sequential/parallel and native serial/static at widths 1/2/4. Parallel WF runs
also cover `WF_SPLIT_WORK=0/60000/240000/1200000`; invalid startup settings must
fail before the command body. Wrong digests and invalid arguments must fail.
Native static also runs under ASan/UBSan and TSan
on POSIX. Windows requires actual execution in CI; POSIX sanitizers are not
evidence about its native build.

The timing matrix uses shapes 0–6 (plane, boundary, interior-first, interleaved,
all-interior, all-exterior, interior-last), counts 4,096/65,536, limit 256 and
32/2 batches respectively: 131,072 points per process. It runs five alternating
passes with matching requested worker counts 1/2/4 where available, plus a
byte-identical WF replica. The `work60000`, `work240000` and `nosplit` forms
execute the **same par image** with `WF_SPLIT_WORK=60000`, `240000` and `0`.
Inherited split tuning is removed from the baseline and native commands.
These values distinguish a zero-level default and budgets for up to two or eight chunks
at 4,096 points/W4, while also exposing over-splitting on cheap exits. They
are diagnostic controls, not selected defaults. A four-worker-capable host
produces 1,405 processes, including five work120000 samples for the
shape4/count4096/W4 four-leaf diagnostic; the initial panel before these
controls had 770.
Raw process samples, oracle inputs/digests, tool flags, source copies, host
metadata and executable/compiler hashes are artifacts. Requested counts do not
prove every worker executed a task; runtime attribution needs separate evidence.

`make mandelbrot-command-diagnose OUT=<fresh-absolute-directory>` runs the same
inputs separately with the normal executable's `WF_SCHED_REPORT=2`. It records
one live scheduler report per process at widths 1/2/4 where available and work
settings 0/60,000/240,000/1,200,000, plus work120000 for
shape4/count4096/W4: 169 reports on a four-participant host, 112
on a two-participant host. The reports and oracle inputs are retained under
`diagnostics/` in each CI artifact. The correctness check requires the normal
CLI to report configured/started workers without a custom observer link.
Timing runs explicitly disable automatic reports. Diagnostic counts are not
timing samples or a simultaneous shutdown snapshot; a started worker need not
have executed a task, and counts alone do not identify time lost waiting.

After the unobserved screen, Linux CI runs the `profile` mode on the unchanged
ordinary executables for shape4/count4096/W4, comparing work60000, work120000
and native static partitions. Software CPU samples use 256 repetitions;
system-wide scheduling traces retain the screen's 32 repetitions. Plain
process envelopes accompany the observed runs. `profile/` retains commands,
availability probes, raw perf data and decoded events. Tool or kernel
restrictions are reported as unavailable. These are attribution observations,
with their own perturbation and duration, not extra performance samples;
system-wide scheduling events must be filtered to the workload's PIDs.

The Linux placement control asks whether runnable delay and uneven CPU
placement explain the short-command gap. It uses `thread-placement.c` through
`LD_PRELOAD` on these same executables, wrapping thread entry without changing
the scheduler. Bound and unbound runs both use the wrapper. Four compute
participants receive distinct CPUs from the original allowed mask when bound;
the WF launcher is excluded, while the native main thread participates.
The wrapper verifies participant count and requested affinity; separate reports
record thread IDs. Five paired passes at 32/256 repetitions retain identical
replicas, wall/CPU/memory/switch counts, and alternate form/binding order.
Each program's bound/unbound pair uses the same wrapper; WF creates four
threads, while native creates three and places its main in initialization.
Binding also changes startup placement and includes affinity-setting costs;
the net change cannot be attributed solely to steady-state scheduling.
Scheduling traces follow the timed pairs and also record new-thread wakeups.
Evidence for placement as a cause requires both reduced runnable delay and a
smaller wall gap, with stable replica measurements. A bound win alone does not
justify production affinity or prove a universally better waiting policy.
The original unwrapped screen remains the performance qualification path;
remove this control when the placement question is settled.

The companion `priority/` panel keeps all four participants bound and varies
only their requested nice value: condition 0 uses 0, condition 1 uses -10.
Both conditions use the same root measurement runner, started through `sudo`
before its timer. The shim sets and reads back priority on each compute
thread; the WF launcher and perf recorder retain their inherited priority.
Permission failure is recorded as unavailable. This is a controlled
background-competition experiment, not ordinary-user performance acceptance.
It retains 120 process samples per panel, separate thread/UID/cgroup/autogroup
reports and post-timing traces. The `condition` column has panel-specific
meaning recorded in `conditions.txt`; old placement artifacts called it `bound`.
Evidence for background interference requires reduced overlap with external
tasks, reduced runnable delay and a smaller paired wall gap together. Trace
order is reversed between the two panels, but traces cannot correct the
separate unobserved samples. A benefit would not select higher production
priority or justify removing slower ordinary-user samples.

The initial screen reports per-cell paired median/min/max wall, CPU and RSS
ratios. A wall/CPU **gap** requires all five ratios above 1.05 and all five WF
wall A/A ratios inside [0.95, 1.05]. Gaps in the default WF/native comparisons
and the original WF A/A comparison fail the screen after all measurements;
tuning comparisons remain diagnostic.
Noisy cells and CPU ratios with a zero accounting sample on either side remain
open. RSS is descriptive. This
5% rule is an initial diagnostic criterion selected after the first local
exploratory cohort and before CI, not a pre-registered claim about that cohort.
Five processes do not establish population tails, and a screen without gaps
does not establish full performance acceptance or the complete project goal.

The first local cohort used the `16dece48` compiler on an eight-core M1 Pro,
with the initial command sources captured in its artifact. At four requested
workers, 4,096-point plane, boundary and interleaved loads had paired WF/static
wall medians **2.782, 3.153 and 3.012**. Their five ratios all exceeded 1.05,
with A/A inside the stated window. The same workloads at 65,536 points were
much closer; this does not excuse the small expensive loops. Emitted IR calls
`wf__par_split_budget(span, 219)`. The maintained threshold is 1,200,000 work
units per chunk: 4,096 elements admit zero split levels, while 65,536 admits
three at W4. This identifies a concrete policy limitation to investigate;
the measurements include input/kernel/command cost and do not isolate the
scheduler's per-task cost. No default policy was changed by this panel.

The [first five-target CI run at `444b89b8`](https://github.com/mbbill/Whitefoot/actions/runs/34397780620)
completed 770 timing processes each on Linux x64, Linux ARM64 and macOS x64,
and 560 on the two-participant macOS ARM64 runner. All four completed the
48-case correctness and native sanitizer checks. Recomputing the summaries
from their raw process matrices reproduces their exits exactly. All-interior,
4,096-point W4 WF/static wall medians are **3.5647** on Linux x64 and
**3.7268** on Linux ARM64, with all five paired ratios above 3.53 and 3.67
respectively. macOS x64 is noisy for this cell. The macOS ARM64 job exits zero,
but **all 140 wall/CPU comparison rows are `noisy-open`**; its all-interior W2
median is 1.9866. None of these is platform performance acceptance. Windows
stops before timing on MSVC's deprecated `getenv` diagnostic in the native
reference. The subsequent reference uses `_dupenv_s`, frees its buffer and
retains strict warnings; the later run below executes it on Windows.

The local `cd059f74` cohort (`split-work-local2`, evidence currently local only)
then completes all 1,400 oracle-checked processes on the eight-core M1 Pro,
after the `444b89b8` canonical gate finishes and without concurrent compilation
or tests. The screen returns failure because default-path gaps remain; no
failed or noisy cell is removed. The normal compiler and all executable/source
hashes are retained. The native serial control now bypasses per-batch pool
atomics; comparisons below are within this cohort. At W4 and 4,096 points,
paired median ratios are:

| Distribution | 60,000/default wall | 60,000/static wall | 60,000/static CPU |
| --- | ---: | ---: | ---: |
| Plane | 0.3996 | 1.1182 | 1.0248 |
| Boundary | 0.3521 | 1.1174 | 1.0095 |
| Interleaved | 0.3661 | 1.1083 | 1.0281 |
| All interior | 0.3011 | 1.0677 | 1.0102 |
| All exterior (noisy) | 1.1550 | 1.2762 | 1.0806 |

Relative to the default's sequential execution at this small size, 60,000 also
raises process CPU: medians are 1.1940/1.1346/1.1466/1.0786 for plane,
boundary, interleaved and all-interior respectively, with all five paired
ratios above one in each cell. The 7.9–19.4% CPU increase accompanies the wall
gain and remains a cost to explain, despite CPU being close to static.

All-interior A/A wall ratios are [0.9978, 1.0032], and its 60,000/default
wall ratios are [0.2973, 0.3028]: the ordinary path regains substantial
parallelism. This is still a 5.4–8.1% wall deficit against static; neither
the gain nor the close CPU ratio establishes top-tier completion. Its peak
RSS ratio to static is 1.9574 (3,014,656 versus 1,540,096 bytes), which remains
an unexplained process-memory gap rather than being hidden by the timing win.
For all-exterior small input,
60,000/default CPU ratios are [1.8342, 2.0303] with median 1.9996, while its
wall A/A fails the noise window. This remains adverse evidence even though it
is not a clean timing qualification. At 65,536 points, all-interior timing is
essentially unchanged by 60,000 (1.0082 versus default), while interior-first
and interior-last improve to 0.6148/0.6155 through finer balancing. Static
partitions are not a strong ceiling for those skewed distributions.

These results support retaining an explicit policy control and investigating
cost estimation and task/wakeup overhead in the maintained runtime. They do
not select a universal lower default: cheap exits, CPU cost, memory, additional
computations and every CI target still require qualification.

The [subsequent five-target run at `ab6cf582`](https://github.com/mbbill/Whitefoot/actions/runs/34400322732)
uses the `cd059f74` implementation. All five ordinary-command correctness
steps pass, including Windows with the corrected native reference. Four hosts
complete 1,400 timing processes each; macOS ARM64 has two participants and
completes 980. All five screens fail on performance gaps. Recomputing each
summary with its captured AWK reproduces its bytes and exit status. The eleven
available executable/source hashes per artifact verify; the compiler executable
itself is not included, so its recorded hash cannot be checked from the ZIP.
For all-interior, 4,096-point input, paired median ratios are:

| CI target | Workers | Default/static wall | 60,000/default wall | 60,000/static wall | 60,000/static CPU | A/A wall range |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Linux x64 | 4 | 3.3877 | 0.4509 | 1.5231 | 0.9899 | 0.9918–1.0161 |
| Linux ARM64 | 4 | 3.7132 | 0.4189 | 1.5550 | 1.0047 | 0.9990–1.0024 |
| macOS x64 (noisy) | 4 | 3.4629 | 0.3125 | 1.1342 | 1.0541 | 0.9251–1.0052 |
| macOS ARM64 (noisy) | 2 | 1.9956 | 0.5178 | 1.0494 | 1.0297 | 0.9644–1.0828 |
| Windows x64 | 4 | 3.1124 | 0.3607 | 1.1275 | 0.8889 | 0.9946–1.0016 |

The Linux 60,000/static wall ranges are 1.4799–1.6235 and 1.4816–1.6244;
the substantial remaining loss is not just the M1's roughly 7% median gap.
Their CPU ratios remain close to one, suggesting parallel utilization/waiting
deserves investigation rather than establishing a cause. Linux x64 exposes
four hardware threads on two EPYC 7763 cores; Linux ARM64 exposes four
Neoverse-N2 cores. Each comparison uses matching participant counts on its own
host. Windows wall ratios are 1.0845–1.3710, while its CPU ratios span
0.6000–1.3333; that coarse/noisy CPU accounting does not establish a CPU win.
For the same Linux x64 cell, the median process CPU/wall ratio is 2.4789 for
60,000 versus 3.8284 for static; summed voluntary/involuntary context switches
have medians 609 versus 17. Linux ARM64 has utilization ratios 2.4553 versus
3.8137 and switch medians 590 versus 14. These five-process observations
motivate collecting task/park reports on the normal path. They do not isolate
which runtime operation causes the lost utilization or prove every context
switch was a scheduler-requested yield.
The macOS cells fail the A/A window and remain unresolved. Across all cells,
wall A/A is noisy in 19/42 Linux x64, 16/42 Linux ARM64, 27/42 macOS x64,
25/28 macOS ARM64 and 24/42 Windows cells. No platform is qualified by this run.

Raw artifacts: [Linux x64](https://github.com/mbbill/Whitefoot/actions/runs/34400322732/artifacts/10123241270),
[Linux ARM64](https://github.com/mbbill/Whitefoot/actions/runs/34400322732/artifacts/10123230529),
[macOS x64](https://github.com/mbbill/Whitefoot/actions/runs/34400322732/artifacts/10123346930),
[macOS ARM64](https://github.com/mbbill/Whitefoot/actions/runs/34400322732/artifacts/10123248547),
[Windows x64](https://github.com/mbbill/Whitefoot/actions/runs/34400322732/artifacts/10123353544).

### Historical manual-link panel

The C host checks every returned count against an explicitly rounded recurrence
and verifies both input arrays after every call. Known fixed/escaping orbits,
NaN/infinities, zero and maximum iteration limits, empty/odd/non-power-of-two
batches and maximum repetition counts qualify the boundary. Each full
qualifier checks 185,155 single points and 252 batches / 184,422 outputs.
`check-mandelbrot` runs all nineteen configurations in ordinary and C-sanitized
images, then ninety-five timing smokes and 150 grain-panel smokes. The split
panel below additionally qualifies eight distinct backend/width/policy forms
in both images and runs 56 smokes. Full qualification uses its own grain16
fixtures, so timing grains sharing one backend/width/policy do not duplicate
that qualification. The allocation panel adds eleven forms in both images and
69 smokes. The
Parlay automatic-grain entry also runs the existing atomic exactly-once,
joined-tail and capacity qualifier at widths one, two and four using the same
adapter object in the shared scheduler image. The existing pinned Rayon Linux
caller-worker lifecycle report is checked with its exact allocation/stack
grammar and actual exit status; other leaks or sanitizer failures reject.
Sanitizers cover this C host, native kernel, WF runtime and floor; emitted WF
code and the reused native adapters/libraries are ordinary objects. Linux
execution must qualify the new shared image's lifecycle report separately.

After `make scheduler-fetch`, run `make check-mandelbrot` with an absolute
`OUT`, then `OUT=... RESULTS=... sh mandelbrot-bench.sh calibrate` and
`OUT=... RESULTS=... sh mandelbrot-bench.sh grain-calibrate`. The targeted split
comparison uses `OUT=... RESULTS=... sh mandelbrot-bench.sh split-calibrate`;
output initialization uses `OUT=... RESULTS=... sh mandelbrot-bench.sh allocation-calibrate`.
Run all commands from this experiment directory; `RESULTS` must not exist. Canonical
`check` includes qualification, and the `mandelbrot-linux` compute CI job builds,
checks and calibrates sequentially on one recorded CPU mask. The native
adapters reuse the scheduler panel's pinned sources and scalar build path.

Nineteen configurations share one executable in the fixed-grain panel:

- generated WF sequential at width one; generated WF automatic at width one
  and four, with cost/capacity/team policies at width four;
- the same native C chunk callback under WF, static pthread, oneTBB, Parlay
  with fixed or automatic scheduler grain,
  Rayon join and Rayon parallel iterator, each at widths one and four.

Native callbacks process sixty-four points each in this panel. Parlay's
automatic scheduler grain groups these callbacks; it does not change how many
points the shared callback receives. With more than one callback, its timed
prefix runs per invocation, inside core time, and is not replayed. Empty and
single-callback ranges bypass calibration. Generated maps report grain
zero, meaning compiler/runtime splitting rather than that native chunk size.
All native schedulers execute the same callback machine code; comparisons
against generated WF also include code-generation and decomposition differences.
All widths include the caller. Width is a requested budget, not evidence that
each worker did useful work. The raw `wf_pool_lanes` field observes only the
WF pool; it does not report native-library occupancy. The validator requires
expected WF capacity, deriving cost's admission threshold from emitted weight
and the runtime threshold instead of treating every width-four label as active.

The fixed-grain calibration has 1,045 processes / 9,405 calls: five
rotating/reflected passes and eight warm calls plus a separate first call.
Seven 4,097-point cells at limit 256 cover a sampled plane, a boundary
neighborhood, clustered/interleaved/trailing equal multisets of interior/exterior
points, all-interior and all-exterior work. The trailing input places the same
heavy quarter at the end, challenging an estimate made from a cheap prefix.
Four 33-point cells at limit sixteen cover plane, boundary, interior and exterior.
Seed 828219 fixes the coordinates. The raw total capped iteration count is
bound across configurations/passes and checked analytically for the fixed-orbit
families. It describes useful recurrence work, not scheduler tasks.

The grain panel retains the five generated-WF controls and varies native
callback size over 1, 16, 64, 256 and 1,024 points for each of seven native
selectors at widths one and four: 75 configurations. Five 4,097-point inputs
(plane, clustered, trailing, interior, exterior) and two 33-point inputs
(plane, exterior) give 2,625 processes / 23,625 calls over five passes. Raw
filenames include grain so no configuration overwrites another. Both panels
retain independent order, manifest and summary files; do not pool their calls
or interpret a lowest median selected from the grain screen as held-out
confirmation. The native callback machine code is common at every grain;
the number of callback invocations changes. With Parlay automatic grain,
callbacks may be grouped further or consumed by the timed prefix, so callback
count is not a runtime-task count. These controls add no timing probes or
adaptive policy to generated WF.

Core time includes output allocation and joined computation. Native output
uses `malloc` because each callback writes its entire assigned range; native
qualification poisons every output first to detect omitted writes. Generated
WF retains whatever initialization its normal `buffer_new` lowering requires.
Cycle time additionally copies the complete output and releases its original
allocation. Input construction and reference computation precede the batch;
full output/input checks and printing occur between calls. Inter-cycle gaps
are retained and emitted after batch resource sampling. Batch wall/user/system
CPU and OS switches therefore include first calls, checks and printing; peak
RSS is process-wide. Pool startup is charged to the first computation;
same-owner native teardown follows the measured batch. Statistics and events
are disabled. These are scalar end-to-end and common-kernel controls, not a
claim of best Mandelbrot algorithms or a tuned native scheduling frontier.

The initial September 8 M1 calibration, before the automatic-grain and trailing
extensions, uses ordinary image SHA256
`1a90946b58ca2c1122fcee199b69988b2eb3fccb44616ab4dd468731d045de36`.
All 850 processes passed; independent replay checked 7,650 calls, 18,906,210
output positions, 6,800 gaps, all summaries and 53 distinct manifest paths.
Worker placement and thermal state are uncontrolled. Tables below give core
microseconds, taking the median of five process means of eight warm calls;
first calls remain separate and no outliers are removed.

| Input / points | WF sequential W1 | WF cost W4 | WF capacity W4 | WF team W4 |
| --- | ---: | ---: | ---: | ---: |
| Plane / 4097 | 656.937 | 658.974 | 177.636 | 186.370 |
| Boundary / 4097 | 1275.547 | 1328.641 | 353.901 | 359.636 |
| Clustered / 4097 | 849.750 | 848.167 | 230.104 | 899.495 |
| Interleaved / 4097 | 874.323 | 882.297 | 228.953 | 228.323 |
| Interior / 4097 | 3462.084 | 3375.911 | 897.865 | 927.667 |
| Exterior / 4097 | 5.193 | 5.573 | 5.526 | 5.120 |
| Plane / 33 | 0.307 | 0.307 | 4.417 | 1.406 |
| Boundary / 33 | 0.787 | 0.813 | 4.058 | 1.776 |
| Interior / 33 | 0.771 | 0.786 | 4.187 | 1.964 |
| Exterior / 33 | 0.089 | 0.094 | 3.833 | 1.411 |

The emitted weight is 219, giving a cost-admission cutoff of 10,960 points
under the current 1,200,000 work threshold. Every generated-WF cost cell has zero
WF pool lanes despite its requested W4 budget. Capacity/cost paired median
ratios are 0.263–0.271 for the five heavy 4,097-point families, faster in every
pair. This is an admission-policy control, not a newly improved default.
Every 33-point capacity comparison is slower in every pair; unconditional
subdivision is not selected either.

Clustered and interleaved inputs have identical long/short point multisets and
265,472 total capped iterations. Changing their placement changes team/capacity
from 3.927 [3.845–3.987], slower in all five pairs, to 0.997 [0.978–1.031],
mixed. This supports investigating subdivision and load imbalance: fixed
team-sized terminal chunks can strand work. It does not measure worker
utilization or actual task counts, which this uninstrumented image does not
record.

The common C callback controls at W4 / grain 64 give:

| Input / 4097 points | WF adapter | Static | oneTBB | Parlay | Rayon join | Rayon iterator |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plane | 176.031 | 176.834 | 180.849 | 200.099 | 200.927 | 198.615 |
| Boundary | 354.120 | 363.703 | 348.823 | 358.724 | 369.042 | 392.625 |
| Clustered | 226.854 | 888.729 | 235.094 | 247.672 | 274.359 | 452.891 |
| Interleaved | 225.563 | 239.787 | 235.250 | 249.193 | 248.000 | 242.239 |
| Interior | 892.209 | 942.271 | 893.219 | 920.224 | 989.875 | 991.896 |
| Exterior | 5.365 | 3.672 | 7.870 | 6.229 | 8.099 | 12.000 |

Generated WF capacity versus each heavy cell's lowest observed native median
has paired median ratios 1.006–1.012, with mixed pair directions. These close
results do not establish a winner. The cheap exterior cell loses to static
in all five pairs, ratio 1.436 [1.325–2.559]. WF sequential versus native
static W1 has heavy-cell paired median ratios 0.986–1.013; that comparison
also includes generated kernel and allocation/representation differences.
At 33 points the native grain creates only one callback, so nominal W4 is
not a four-way parallel control. Native grain tuning, held-out confirmation,
dedicated-core scaling and nested workloads remain open.

The [Linux run at `2bf3c7bd`](https://github.com/mbbill/Whitefoot/actions/runs/34218165282)
also passed Mandelbrot qualification and all 850 calibration processes. Artifact
`compute-mandelbrot-linux`, ID `10052718007`, has ZIP SHA256
`1c2edebfff429822b0c82a3b1330681a229a8cb2d11c5862d042a76fc4a17b0a`.
This is a separate EPYC 7763 VM cohort: two physical cores, four SMT logical
CPUs, inherited mask 0–3, no individual worker pinning, and unqualified CPU
quota. Clang 18.1.3 targets x86-64-v3 with the same scalar/strict-FP/no-LTO
conditions. The compiler executable is not retained in the artifact, so its
recorded hash cannot be independently recomputed. The source revision, emitted
IR, objects, libraries and images remain available. All 34 qualifier/lifecycle
logs passed replay, including the four expected Rayon sanitizer reports with
exactly 1,904 bytes in two allocations; this qualifies the new Linux image's
known lifecycle boundary, without treating it as leak-free.

Core microseconds, using the same five-process warm-mean statistic:

| Input / 4097 points | WF sequential W1 | WF cost W4 | WF capacity W4 | WF team W4 |
| --- | ---: | ---: | ---: | ---: |
| Plane | 667.145 | 672.209 | 247.560 | 345.134 |
| Boundary | 1284.251 | 1286.642 | 489.710 | 702.400 |
| Clustered | 850.053 | 854.533 | 336.437 | 883.483 |
| Interleaved | 849.762 | 852.934 | 326.938 | 450.315 |
| Interior | 3369.780 | 3350.219 | 1276.209 | 1740.493 |
| Exterior | 8.688 | 10.302 | 7.323 | 7.097 |

Generated cost still admits no parallel work in this matrix. Heavy-cell
capacity/cost paired median ratios are 0.368–0.393, faster in every pair.
Clustered team remains slower than cost in all five pairs; capacity recovers
parallel work on both clustered and interleaved inputs. Unlike M1, team is
also consistently slower than capacity on the interleaved input. The logical
CPU budget does not establish equal simultaneous progress on four cores.
For 33 points, capacity loses to cost in all five plane/interior/exterior
pairs; boundary is mixed, with two faster pairs. The small-work conclusion
must retain that host-specific exception.

The common C callback at W4 / grain 64 gives:

| Input / 4097 points | WF adapter | Static | oneTBB | Parlay | Rayon join | Rayon iterator |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plane | 246.576 | 3263.983 | 249.862 | 455.445 | 255.953 | 252.125 |
| Boundary | 626.556 | 2857.543 | 471.461 | 727.039 | 500.172 | 663.296 |
| Clustered | 344.027 | 3827.325 | 336.262 | 450.976 | 344.977 | 459.001 |
| Interleaved | 323.931 | 3582.558 | 324.099 | 528.774 | 333.238 | 327.499 |
| Interior | 1283.185 | 2457.241 | 1234.718 | 1440.170 | 1241.298 | 1267.340 |
| Exterior | 6.655 | 4111.591 | 10.686 | 10.738 | 12.884 | 14.010 |

The static medians contain millisecond excursions accompanied by OS scheduling
activity; they do not establish an intrinsic dispatch cost. Exterior pass zero has eight warm calls
at 3.526–3.957 microseconds and zero batch involuntary switches; passes one
through four each contain millisecond calls and 65–117 involuntary switches.
Several calls cluster near six milliseconds. No observations are discarded.
This is evidence to investigate worker placement and OS scheduling, not proof
of the cause or a general WF advantage over static partitioning. Batch CPU
still includes startup, checks and printing. Native WF boundary is also slower
than oneTBB in this screen. The data do not establish a universal winner.

These two hosts reproduce admission and imbalance losses while disagreeing
on important scheduling details. The later grain panel investigates native
grain sensitivity and an automatic-grain form alongside fixed callback work.
In the pinned Parlay implementation,
automatic grain executes a growing prefix until one sampled block takes at
least 1,000 ns (or the range ends), then partitions the remainder. The current
fixed-grain adapter deliberately supplies grain one over host chunks, bypassing
that calibration; the later `parlay-auto` selector retains it. The former is a
same-chunk scheduler control, not evidence against Parlay's
automatic-grain policy. Any WF sampling experiment must charge its probe,
preserve exactly-once execution and include inputs whose expensive region
appears after the sampled prefix.

The subsequent M1 grain panel uses ordinary image SHA256
`0fcf197c5953a4f68654a4dd620bfbb8f7ef2294d31a6a97ecbf17248b801bd0`.
All 2,625 processes passed, with independent replay of 23,625 calls,
69,359,625 output positions, 21,000 gaps and 52 distinct manifest paths.
This is a fresh same-image cohort; its observations are not pooled with the
earlier fixed-grain M1 or Linux runs. Statistics/events remain disabled,
worker placement is unfixed and first calls remain separate.

Each cell below selects that backend's lowest observed W4 core median from
the five tested grains. Values are microseconds, with points per callback in
parentheses; selection uses the same five process warm means being reported,
so this is a grain screen rather than held-out confirmation.

| Native selector | Plane / 4097 | Clustered / 4097 | Trailing / 4097 | Interior / 4097 | Exterior / 4097 |
| --- | ---: | ---: | ---: | ---: | ---: |
| WF | 175.620 (64) | 225.730 (256) | 227.162 (256) | 876.250 (16) | 3.942 (1024) |
| Static | 175.854 (16) | 887.667 (256) | 643.266 (256) | 905.302 (16) | 3.438 (1024) |
| oneTBB | 179.880 (64) | 230.417 (256) | 230.198 (64) | 875.073 (64) | 6.547 (1024) |
| Parlay fixed | 191.922 (16) | 244.724 (64) | 248.474 (16) | 919.432 (16) | 5.422 (256) |
| Parlay automatic | 197.214 (16) | 261.604 (1) | 502.625 (1) | 938.573 (1) | 5.380 (256) |
| Rayon join | 179.214 (16) | 243.615 (1) | 245.469 (256) | 887.240 (16) | 7.042 (256) |
| Rayon iterator | 187.380 (64) | 448.484 (16) | 248.927 (256) | 946.036 (64) | 11.792 (256) |

Grain choice materially strengthens some references. On plane input, Rayon
join grain 16 versus 64 changes the median from 197.213 to 179.214 us;
the paired ratio is 0.909 [0.851–0.954], faster in all five pairs. On trailing
input, Rayon iterator grain 256 versus 64 changes 334.812 to 248.927 us;
the paired ratio is 0.774 [0.689–1.034], faster in four pairs. Chunk size also
changes static partition boundaries, so its improvements need not represent
lower dispatch overhead. No single grain wins across inputs or backends.

Generated WF capacity core medians are 176.771 / 229.104 / 232.214 / 891.766 us
for plane / clustered / trailing / interior. Comparisons against each cell's
lowest selected native median have mixed pair directions for the first three;
interior loses to oneTBB in all five pairs, ratio 1.018 [1.011–1.021]. Exterior
capacity is 6.328 us versus selected static 3.438 us, ratio 1.838
[1.579–2.427], slower in all five pairs. These generated/native comparisons
include allocation, code generation and decomposition. At 33 points, grains
of 64 or larger expose one native callback, so those rows measure serial-path
overhead under a W4 budget, not four-way useful parallelism. No default WF
policy improvement is claimed by this panel.

Trailing input exposes a specific automatic-grain weakness. At native grain
64, Parlay automatic takes 721.547 us versus fixed 257.734 us, paired ratio
2.865 [2.773–3.071], slower in all five pairs. Automatic also loses all five
pairs at grains 1, 16 and 256; grain 1024 is mixed. Even its selected grain one
remains 502.625 us. The heavy-point count and total useful iterations are the
same as clustered input; ordering changes the observed performance.

A separate one-shot diagnostic copied the pinned Parlay include tree and
inserted only an `fprintf(stderr, ...)` immediately before `start += done` in
`fork_join_scheduler::parfor`, reporting initial range, completed prefix and
selected grain. Only the Parlay adapter object was rebuilt; the other objects
were reused from the ordinary image. Diagnostic image SHA256 is
`94084bb72c3635500b8ba0e0a8883b5fb7ea474dd380bbaf33cdeae468e8cf08`.
Thirty processes cover clustered/trailing/exterior, native grains 1/64, W4,
4,097 points, limit 256, seed 828219 and five passes of nine calls. Every output
is checked; diagnostic timing is not ranked against the ordinary image.
For trailing/grain64, 34 of 45 calls consume 31 callbacks and select scheduler
grain 31; clustered/grain64 selects grain one in all 45 calls. With the
observed grain 31, the pinned split rule partitions the remaining callback
ranges into [31,50) and [50,65). The latter contains 897 of the 1,025 heavy
points. This demonstrates how a cheap prefix can select a coarse terminal
chunk that concentrates most later work. It is consistent with the ordinary
slowdown, but does not establish the ordinary image's per-call chosen grain:
logging, image layout and measurement conditions can change the probe.

The corresponding Linux extension at revision
`1266ef9c14d2a317c15352aa56ae5127cca02211` passed in
[compute run 34221470922](https://github.com/mbbill/Whitefoot/actions/runs/34221470922).
Artifact `10054004011` has ZIP SHA256
`30a7a42e48206ecb9eeda12cc893fc3ff3ee4cfcd5d9a30242f324ee5fe6075f`;
ordinary image SHA256 is
`2eaf839bb3fb69ee81944e085ecf61b2518dd6cc28d206c77282eaee7359e985`.
All 52 manifest paths, including the retained compiler, were verified against
the exact revision or artifact. The grain panel independently replays 2,625
processes, 23,625 calls, 69,359,625 outputs and 21,000 gaps; the separate fixed
panel replays 1,045 processes, 9,405 calls, 24,633,405 outputs and 8,360 gaps.
Neither panel is pooled with another cohort. Shared qualification comprises
38 full qualifiers, 95 ordinary/150 grain smokes and three Parlay-auto
exactly-once/joined-tail/capacity checks. The four Linux sanitized Rayon logs
retain the exact allowed 1,904-byte/two-allocation lifecycle report; they are
not leak-free passes.

The host exposes two AMD EPYC 7763 physical cores/four SMT CPUs, mask 0–3,
with Clang 18.1.3 and scalar x86-64-v3 flags. Individual placement and CPU quota
remain unqualified. Each native cell below selects its lowest observed median
from the five grains, using the same reporting samples, not a held-out run.
Units are microseconds; parentheses give points per callback.

| Native selector | Plane / 4097 | Clustered / 4097 | Trailing / 4097 | Interior / 4097 | Exterior / 4097 |
| --- | ---: | ---: | ---: | ---: | ---: |
| WF | 245.384 (64) | 324.450 (16) | 316.817 (16) | 1249.363 (16) | 5.836 (256) |
| Static | 3688.018 (256) | 2520.220 (1024) | 3553.502 (1) | 2556.248 (256) | 3371.318 (1) |
| oneTBB | 249.179 (64) | 322.213 (16) | 322.019 (16) | 1218.446 (16) | 9.653 (1024) |
| Parlay fixed | 346.249 (1024) | 449.141 (256) | 443.526 (1) | 1366.242 (1) | 8.182 (256) |
| Parlay automatic | 317.357 (16) | 389.083 (1) | 592.923 (1) | 1330.765 (1) | 8.428 (64) |
| Rayon join | 248.141 (64) | 324.911 (16) | 324.883 (16) | 1239.272 (16) | 11.232 (1024) |
| Rayon iterator | 249.501 (16) | 458.357 (1) | 326.266 (16) | 1245.043 (64) | 9.792 (1024) |

Generated WF capacity core medians are 245.934 / 346.685 / 337.791 / 1294.339 us
for plane / clustered / trailing / interior. Against each cell's selected
lowest native median, paired signs are mixed except trailing: capacity/native
WF grain16 is 1.063 [1.022–1.088], slower in all five pairs. These comparisons
include generated representation, allocation and decomposition. Default cost
admission still requires at least 10,960 points for this compiler weight and
does not split any measured input; this panel changes no default policy.

At trailing/grain64, Parlay automatic is 707.577 us versus fixed 500.917 us,
paired ratio 1.376 [1.300–1.549], slower in all five pairs. The independent
fixed panel also loses all five, ratio 1.224 [1.163–1.414]. Automatic loses
all five grain-screen pairs at grains 1/16 too, but grain256 is mixed and
grain1024 has four faster pairs; the M1 grain256 outcome does not transfer.
Equal heavy-point counts and useful iteration totals across clustered and
trailing rule out a change in total useful work, but Linux has no diagnostic
trace of the automatic grain chosen by the ordinary image.

Resource counters expose another question: clustered oneTBB W4/grain1024
records 3,573–3,599 batch involuntary switches across all five processes,
with core median 845.619 us, versus 322.213 us at grain16. Static's multi-ms
excursions also persist across this grain screen. These observations warrant
placement/wait-protocol investigation; they do not identify a cause or prove
intrinsic scheduler costs. Batch CPU/switch counters include checks and host
work outside core timing. No observations were discarded, and a four-thread
budget on this VM does not establish four physical cores of useful progress.

### Matched terminal-range comparison

The split panel asks how much generated/native loss follows from their
different terminal ranges. Under `WF_COMPUTE_BUDGET_CONTROL`, the host may
set `wf_compute_requested_chunks` before floor entry. Zero retains the
lane-derived request; the new `chunks16`/`chunks256` capacity policies request
16/256 terminal chunks at W4. Existing capacity requests 64. Span limits,
queue backpressure, acquisition refusal and sequential fallback still apply;
default cost/team behavior and public function signatures do not change.
An outside-timing check validates the requested budget, including short spans.
It also initializes the lane-count cache before the first measured chunk-policy
call; first-call setup is therefore not identical across policies.

Fourteen configurations share one image: generated sequential W1, generated
W4 cost/capacity/chunks16/chunks256, and native WF/oneTBB/Rayon join at W4 with
16/64/256 points per callback. These native selectors follow the earlier
heavy-cell leaders; the broader seven-selector matrix remains maintained.
Plane/trailing/interior/exterior each run at 4,096 and 4,097 points, limit256,
plus exterior33/limit16. Five rotating/reflected passes with eight warm calls
and one separate first call produce 630 processes/5,670 calls, 20,667,150
checked outputs and 5,040 gaps. Checks and timing scope match the other panels.

At 4,096 points, generated 16/64/256 terminal chunks match the point ranges
of native grain256/64/16. Native WF recursively bisects those callback ranges;
the requested partition geometry aligns, but frame layout, allocation, kernel
code, actual steals and fallback can still differ. At 4,097 points the native
forms expose 17/65/257 fixed-size callbacks, versus 16/64/256 balanced generated
terminal ranges: this is deliberately an uneven-boundary control. Requested
chunks are not measured publications or useful occupancy. At 33 points,
capacity and chunks256 clamp to the same depth; their timing difference is
not evidence of a different intended decomposition.

The September 8 M1 split screen uses ordinary image SHA256
`208a7f139310a106072f723a3da9846b10d9e29ac04f7868d484c92a49302e3d`.
Its generated WF object is byte-identical to the prior grain image, SHA256
`3e7fdf1462fd6158ed548ea9361bfdfad7c6337614eb06fd10d84e0cadadbfd0`.
All 52 manifest paths, 630 processes and raw summaries were independently
replayed. Qualification retains 54 full reports and 301 smokes/28,335 calls.
The M1 placement/frequency controls and scalar flags are unchanged; this is a
new cohort, not pooled with earlier measurements. Core-time medians below use five
process warm means, in microseconds, at 4,096 points.

| Input | WF chunks16 | WF capacity64 | WF chunks256 | Native WF grain64 |
| --- | ---: | ---: | ---: | ---: |
| Plane | 184.323 | 178.906 | 178.302 | 172.203 |
| Trailing | 226.844 | 228.104 | 230.068 | 225.458 |
| Interior | 925.958 | 897.281 | 897.625 | 888.599 |
| Exterior | 3.839 | 5.266 | 6.354 | 4.641 |

At matching 64-terminal geometry, generated/native WF paired core-time ratios are
1.037 [1.005–1.043] on plane, 1.012 [1.011–1.189] on trailing and
1.010 [1.009–1.025] on interior, slower in all five pairs for each. At 256
terminals, generated/native grain16 also loses all five on these heavy cells.
Thus matching terminal geometry does not eliminate the generated/native loss
in this M1 cohort. It does not isolate frame copying from different scalar
kernel code or prove the cause of the earlier Linux loss.

Reducing to 16 chunks improves trailing versus capacity64 in all five pairs
at both sizes: ratios 0.995 [0.841–0.995] at 4,096 and 0.994 [0.976–0.996]
at 4,097. Exterior4096 improves to 3.839 from 5.266 us, paired ratio
0.729 [0.638–0.790], also all five faster. But plane4096 regresses in all five,
ratio 1.034 [1.013–1.061]; interior4096 is mixed. Increasing to 256 chunks
does not consistently improve the heavy cells and worsens exterior at both
large sizes in all five pairs. Neither request is selected as a new default.

The Linux split panel at exact revision
`1b4dd9dc91c68c50abcaa845ada43aa3103430fa` passed in
[compute run 34225829509](https://github.com/mbbill/Whitefoot/actions/runs/34225829509).
Artifact `10055773336` has ZIP SHA256
`1eb5abaca810213c74b303206b0518c3ecf02df26b50f10ad2261fbfbcd82118`;
ordinary image SHA256 is
`aa2a22b37872df2798294ac6fc0ee338cd2f4418d5beb830a262babb157ad938`.
Independent replay verifies all 630 processes/5,670 calls/20,667,150 outputs,
5,040 gaps and 52 manifest paths including the retained compiler. All 54 full
qualifiers pass the strict Linux grammar, including five exact permitted
Rayon lifecycle reports, along with 301 smokes. The EPYC 7763 VM exposes two
physical/four SMT CPUs under mask0–3 with scalar Clang18.1.3/x86-64-v3;
individual placement and quota remain unqualified. Keep this cohort separate.

At trailing4097, generated chunks16 takes 458.790 us versus capacity64
336.241 us, paired core ratio 1.373 [1.294–1.451], slower in all five pairs.
Chunks256 takes 330.123 us, ratio 0.979 [0.971–0.988], faster in all five.
This reverses the M1 direction for the smaller request. Trailing4096 has
large mixed ranges and does not support the same all-five conclusion.
At matched4096 geometry, capacity/native-WF-grain64 has mixed signs on all
three heavy inputs, unlike M1. Interior chunks256/native-WF-grain16 still
loses all five, ratio 1.013 [1.006–1.072]. At 33 points chunks256 is faster
than capacity in all five, but both clamp to the same intended depth; this
does not demonstrate a benefit from more splitting. No request is promoted
to a default from these host-dependent observations.

### Output initialization comparison

The emitted WF wrapper allocates and zeroes its output before calling the
map, whereas the original native path allocates uninitialized output. The
allocation panel adds native policy `zero`, which clears exactly the output
range after the same allocation call and before the unchanged scheduler.
The existing native `cost` label retains uninitialized allocation. This policy
is rejected for generated forms and does not select a different native kernel
or runtime budget. Clearing stays inside core timing. It can change cache
ownership/page effects as well as add writes; the paired difference is an
output-clearing policy observation, not an isolated allocator or memset cost.

Qualification first fills the output with a nonzero sentinel, clears it,
checks through volatile reads, then re-poisons every output before scheduling.
Thus a missing clear fails deterministically and a missing compute write is
still visible. A one-shot build deleting only the clear statement fails with
`mandelbrot: native output clearing`; no altered qualifier is retained.
The ordinary timing path does not execute these qualification-only loops.
M1 assembly retains malloc followed by bzero on the zero policy path, rather
than folding it into calloc. The generated object and native recurrence
remain unchanged.

Twenty-three same-image configurations retain the five generated controls
and add cost/zero pairs for native WF/oneTBB/Rayon join at grains16/64/256.
Plane/trailing/interior/exterior4096 at limit256 and exterior33 at limit16
produce 575 processes/5,175 calls/16,991,595 checked outputs/4,600 gaps across
five passes, with eight warm calls and a separate first call. Qualification
retains 76 full reports and 370 smokes/34,384 calls across all four panels.
The September 8 M1 image SHA256 is
`5ea7bb0a93804b0fe2d194268eb01c86592aceed5f269207c0d220144889121e`;
the generated object remains
`3e7fdf1462fd6158ed548ea9361bfdfad7c6337614eb06fd10d84e0cadadbfd0`.
All 52 manifest paths and raw samples were independently replayed. Scalar
flags and unfixed placement/frequency remain; this is a separate cohort.

The table shows core-time medians over five process warm means at 4,096 points,
in microseconds. Native WF uses grain64 and generated capacity requests64,
matching terminal geometry. Ratios are computed within matched passes first.

| Input | Native WF cost | Native WF zero | Generated WF capacity | Native zero/cost paired ratio [min–max] |
| --- | ---: | ---: | ---: | ---: |
| Plane | 175.010 | 174.615 | 180.823 | 1.004 [0.983–1.014] |
| Trailing | 225.609 | 226.094 | 228.443 | 1.002 [0.991–1.042] |
| Interior | 894.484 | 887.521 | 899.073 | 0.985 [0.967–1.024] |
| Exterior | 4.583 | 4.448 | 4.896 | 1.011 [0.881–1.103] |

None of the 45 native zero/cost groups improves in all five pairs. Three
Rayon join groups regress in all five (trailing4096/grain16,
exterior4096/grain64 and exterior33/grain256); the rest have mixed directions.
On plane4096, generated/native-WF-zero loses all five at each matched request:
chunks16/grain256 is 1.020 [1.015–1.030], capacity64/grain64 is
1.012 [1.007–1.114], and chunks256/grain16 is 1.017 [1.005–1.043].
Adding native output clearing therefore does not remove that observed loss.
The other large-input comparisons against native WF zero remain mixed, and
large excursions are retained:
trailing chunks16 reaches a process mean of 1,503.146 us and a ratio of 6.649
against its matched native-zero control. These observations do not support
an additive universal zeroing cost or assign the remaining loss to a specific
compiler or runtime mechanism.

The [Linux allocation panel at `5ca2e679`](https://github.com/mbbill/Whitefoot/actions/runs/34227268097)
retains artifact `10056367199`, ZIP SHA256
`8d94eb4e6105bd5852af310d07863ef59daecab8e6e04c5365a580c2f8c8a523`.
Independent replay verifies all 575 processes/5,175 calls/16,991,595 outputs,
4,600 gaps and 52 manifest paths, with 76 full qualifiers and 370 smokes.
Seven qualifiers retain the exact permitted Rayon lifecycle report; 69 are
clean. Ordinary image SHA256 is
`a62c1a74b98d04bd96b4ec82b7e44e5090252d884285080a768f25b64db01bc5`.
This host is **EPYC 9V74**, not the preceding 7763 host: two physical/four SMT
CPUs under mask0–3, Clang18.1.3 scalar x86-64-v3, with individual placement
and quota unqualified. Treat it as a separate cohort.

Of 45 native zero/cost groups, 39 have mixed signs, six regress in all five
pairs and none improves in all five. All three plane4096 generated/native-WF-zero
comparisons are mixed; the M1 all-five plane losses do not repeat. Interior4096
chunks16/native-zero grain256 improves in all five, paired ratio
0.973 [0.961–0.995]; exterior4096 chunks256/native-zero grain16 regresses in
all five, 1.030 [1.004–1.354]. All 33-point generated/native-zero comparisons
lose all five, but their terminal geometry differs. Broad ranges remain:
TBB trailing4096/grain256 batch involuntary switches span3–813 without
clearing and131–812 with clearing. Those batch counters do not isolate a
scheduler cause. Neither host selects a default or establishes a universal
additive output-initialization cost.

## Adaptive recursive quadrature

The maintained-runtime path is `quadrature-formal-build` and
`quadrature-formal-calibrate`. It links `compiler/src/backend/sched/` and the
maintained floor directly, reusing the generated WF objects, independent
explicit-stack oracle and native libraries. It does not add a research
runtime. Both images start WF helper threads only when a task is acquired;
native TBB/Parlay/Rayon forms therefore run without a second WF worker pool.
The same native forms run in each image so linked-image/kernel differences
remain visible. The ordinary `whitefootc --par quadrature.wf -o command` path
is also exercised for correctness; its startup and elapsed time are not yet
measured by this panel.

Canonical `check-quadrature` includes 160 maintained/recovered batch processes:
ten inputs, widths one/four, and sequential/default/depth4/depth8 generated
forms. Every call is checked against the oracle. The numerical schema is
shared, but `formal-v2` identifies the maintained ordinary-stack runtime
separately from the recovered `v2` report. Both now measure caller-thread CPU
on the same physical thread across the batch, alongside process CPU and wall
time. Negative reports reject a false runtime identity or a nonnumeric caller
CPU value. Historical `formal-v1` artifacts retained `unavailable` for the
managed-stack implementation; their archived parser remains their reader.

Formal calibration retains 2,400 processes: five forward/reverse passes over
the same ten inputs and widths, four WF forms and eight C++/Rust/native-runtime
reference forms, each in both images. Each process warms up eight times and
checks 256 measured calls; it never pools those calls as independent process
samples. The native forms include serial kernels and TBB, Parlay and Rayon at
two spawn depths. They provide strong algorithm/runtime comparisons, not a
proof that no faster reference exists. SIMD, contraction and LTO remain off.
`parity.tsv` flags each matched generated-form cell when the median paired
maintained/recovered wall or process-CPU ratio exceeds 1.05. Small/noisy cells
are retained for investigation. All native rows remain in `summary.tsv`.
Linux CI runs this screen; macOS local checking is supported. Windows,
additional widths, burst/tail measurements, formal-image sanitizers and normal
CLI timing remain required before broader qualification.

`quadrature.wf` integrates the Lorentz profile
`1 / (1 + ((x - center) / width)^2)` over caller-supplied endpoints using
adaptive Simpson subdivision. Each node evaluates two new points, compares
the refined estimate with its parent estimate, and either returns the
Richardson-corrected estimate or calls its two children and adds their results
in left-plus-right order. Tolerance halves at each child. The explicit depth
argument is an algorithmic subdivision limit: reaching it returns the current
estimate without an accuracy guarantee. It is not compiler proof fuel or a
runtime change. Strict binary64 order is retained; no SIMD, FMA, LTO or
fast-math participates.

This adds data-dependent nested calls and unequal recursive subtrees to the
independent-point workloads. Inputs include broad and narrow centered peaks,
left/right peaks, a peak outside the interval, loose tolerance, reversed/empty
intervals, zero depth and depth-limited subdivision. The ten retained cases
visit1–8,191 nodes and reach depths0–14. This is an initial recursive workload
screen, not coverage of arbitrary integral families, large application batches
or the other missing [application rows](../../investigations/compute-runtime/WORKLOADS.md).
The program, host adapter and C driver belong to this experiment and retire
with it; the adapter qualifies this module's lowering, not a public WF ABI.

The oracle uses an explicit postorder stack with volatile binary64 arithmetic,
independent of the recursive implementations. Every returned value must match
it bitwise. The analytic antiderivative
`width * atan((x - center) / width)` separately checks the non-capped fixtures
using long double with a stated fixture bound of `8*tolerance + 2^-48`.
This fixture check is not a theorem about arbitrary adaptive-estimator error.
The oracle records nodes, leaves, evaluations, deepest level and capped leaves
outside timing. The depth-limited cases keep their actual estimates rather
than being mislabeled converged. A temporary image adding one to the computed
result fails with `quadrature: binary64 result`.

`check-quadrature`, called by the experiment's canonical `check`, runs the same
WF objects in ordinary and ASan-UBSan images, fifty-three forms/grain settings and
worker requests1/4: 212 processes/4,240 checked results, plus twelve forced
owner-slot exhaustion processes/240 results, totaling224 processes/4,480 results.
Host/runtime/floor and
the native C++/Parlay header code are instrumented; generated WF objects and
the shared oneTBB library remain ordinary. The instrumented image
also uses the existing per-lane event counters. At every joined return it
checks publication = local pop + successful steal = run begin = run end = join;
for the original four-worker parallel form it checks publication + slot refusal
equals `3*nodes - leaves + 2`. That count includes two small sibling offers per node,
one recursive offer per internal node and two initial density offers. Startup
must supply all four workers and the full qualification must observe a steal.
The scalar-leaf control below instead checks `nodes - leaves`, retaining only
recursive offers. The [sequential-refusal control](#sequential-subtrees-after-refusal)
checks bounded attempts, with exact counts when no calls are refused and under
full owner-slot exhaustion. Sequential forms must publish nothing. Generated LLVM remains unsanitized;
runtime exhaustion/interleaving qualification stays in `check-runtime`.

The ledger permits density/density, Simpson/Simpson and adaptive/adaptive
siblings. Emitted frames occupy32,48 and88 bytes respectively. On the M1
instrumented centered-peak calls,3,287 nodes produce **8,219 publications**
per call;6,576 of these are small density/Simpson offers including initialization.
The instrumented cohort observes successful steals but does not turn its
sanitized elapsed time into a production cost. These counts identify work
introduced by actualization; they do not by themselves isolate each offer's
contribution to the ordinary elapsed-time loss.

`quadrature-calibrate` runs the ordinary image sequentially across five passes,
two worker requests and fifty-three forms/grain settings: the original `native`,
`wf-seq`, `wf-auto`, `wf-leaf-seq`, `wf-leaf`, the two generated refusal forms,
the four compiler-generated frontier controls at depths4/8,
plus `cpp-seq`, ten
oneTBB/Parlay settings, ten native WF settings and ten reciprocal direction
settings described below, plus Rust sequential and ten reciprocal Rayon
settings. Each
process runs all ten cases, retaining one first and eight warm calls per case:
530 processes/47,700 checked results. Form order reverses on alternate passes.
The AWK reader binds mode, form, requested width and instrumentation to each
invocation, requires the complete ordered input/call inventory and validates
work metadata and event totals. Missing-row, wrong-form and missing-footer
reports are rejected by maintained negative checks; stderr is retained.
The first call of a later case is not cold process startup. Each timer encloses
one complete integration; output checking and printing follow it. Per-call
process CPU and context switches enclose the clocks as well and can include
worker activity; ordinary event fields are zero because instrumentation is
disabled. Tiny calls approach clock resolution and are diagnostic only.

`native` is optimized C recursion for the same arithmetic and subdivision
algorithm, an initial kernel reference rather than a dynamic scheduler ceiling.
`wf-seq` calls the emitted sequential clone. `wf-auto` deliberately calls the
parallel body, including at worker request1 where no pool is started; that
one-worker control exposes unsuccessful offer/call overhead. It is **not**
normal command-entry behavior, which chooses the sequential clone when the
pool is inactive. `wf-leaf-seq` and `wf-leaf` select the sequential/parallel
bodies compiled with the opt-in scalar-leaf control described below. Both
modules share one timing executable, runtime and native kernel; the host chooses
a common indirect generated-call adapter before timing. No default or runtime
interface changes. The oneTBB/Parlay and optional Rayon forms below execute the
recursive algorithm directly, rather than reusing the flat callback adapters.
The configured Rayon matrix passes the Linux compute job; broader tuning and
separate full-gate failures remain open, as recorded below.

Run qualification first, then calibrate alone in a fresh result directory:

```sh
make -C research/experiments/compute-runtime scheduler-fetch OUT=/tmp/wf-quadrature
make -C research/experiments/compute-runtime check-quadrature OUT=/tmp/wf-quadrature
make -C research/experiments/compute-runtime quadrature-calibrate \
  OUT=/tmp/wf-quadrature RESULTS=/tmp/wf-quadrature/calibration
```

The initial three-form September8 M1 screen (MacBookPro18,3, Clang21.0.0, unfixed placement and
frequency) retains ordinary SHA256
`39bb2c2922018bbb083d9815b71638b891f81771276c05fd5a7613bd7eb1f0f7`
and WF object SHA256
`99faabcaf9279d19e41e1423d8f71a8bb7e8e22e769b8dc61e51a8523e1d9bcc`.
Below are microsecond medians of five process warm means. Ratios divide
within each pass before taking the median/range; they need not equal a ratio
of the displayed medians. Native and sequential columns request one worker;
the parallel column requests four on the same host and executes identical
input/arithmetic, not four times the input.

| Input | Native serial | WF sequential | WF parallel W4 | Parallel/sequential paired ratio [min–max] |
| --- | ---: | ---: | ---: | ---: |
| Center peak | 22.203 | 22.432 | 41.609 | 1.833 [1.726–1.981] |
| Left peak | 16.693 | 17.104 | 32.495 | 1.938 [1.820–2.061] |
| Right peak | 16.703 | 17.526 | 31.781 | 1.818 [1.699–2.083] |
| Outside peak | 3.354 | 3.511 | 10.677 | 3.039 [2.964–3.477] |
| Depth cap | 52.839 | 53.286 | 77.114 | 1.468 [1.424–1.504] |

Every listed parallel comparison loses in all five passes. This exposes a
larger issue than the small Mandelbrot representation differences: the scalar
kernel is close to C in these cells, while recursive actualization adds many
fine-grained task operations. Their causal share, a profitable granularity
policy, larger compositions and additional native runtimes
are still unqualified. The new CI row retains the exact compiler, sources,
LLVM, assembly, objects, flags, qualification logs and raw calibration.
The forced parallel-body W1 control also loses all five on centered/left/right
peaks and the depth-cap case, with paired medians1.783/1.809/1.718/1.576
against W1 sequential. No worker pool exists there. Thus unsuccessful offer
checks and altered generated call/code layout warrant investigation alongside
successful publication costs; a large task count alone is not a complete
explanation of the W4 loss.

The [original three-form Linux run at `60fccea3`](https://github.com/mbbill/Whitefoot/actions/runs/34229997227)
retains artifact10057444924, ZIP SHA256
`b3b4748074751220da1b3b0765f1809fb7ddbc4999834f7633772ed2b9c08a87`.
Independent replay verifies30 processes/2,700 calls,12 qualifiers/240 results
and20 retained hashes against that source revision. The host is EPYC9V74,
two physical/four SMT CPUs under mask0–3, scalar Clang18.1.3/x86-64-v3,
with individual placement/frequency and quota unqualified. Ordinary image
SHA256 is `7b5331690134e1ea36a73cd7e9d1d89a819a9e499f8a1cf7ad1990bf888cb6db`.
The original W4 parallel path loses all five paired passes against W1 sequential
on centered/left/right peaks and the depth-cap case, medians
2.587/2.634/2.721/2.545. Centered peak medians are34.372 us sequential and
89.308 us parallel. This confirms a loss on a second host, not the scalar-leaf
control's Linux benefit; the later control cohort is reported below.

### Scalar leaf offer control

The compiler now uses scalar-leaf limit 16 under `--par` by default. This is a
provisional cost choice based on the consistently positive M1/Linux comparisons
below, not a universal optimum or a language rule. Override it with
`--par-scalar-leaf-limit N`, or disable it with `--par-scalar-leaf-limit off`.
Zero still filters zero-operation leaves. Recursive frontier and sequential
refusal remain opt-in. All unfiltered control recipes in this experiment now
write `off` explicitly, preserving their previous lowering and exact work
expectations. Explicit filtered/frontier recipes retain their named thresholds.
The default does not change the runtime linked by normal compiler invocations;
the measurements below used the research compute runtime and do not qualify
shared-runtime performance or broader workload gains.

The dated cohorts below tested this as an opt-in control. After the normal
checks and lowering, it identifies
one-block returning functions with scalar parameters/results, no drops and only
constants or scalar arithmetic/boolean/conversion/reinterpretation operations.
Constants do not count toward the limit. A call, memory operation, control-flow
edge, aggregate or loop excludes the function. This is an IR-operation screen,
not an instruction-count bound, a target timing estimate or a proof budget.
Unknown or more complex callees retain their existing offers.

The pass removes only selected handed-out members from already-permitted
groups, keeps the original source-last join site and drops singleton groups.
Every source call remains at its original position; no worker/result lifetime
or function ABI changes. Compiler tests cover mixed chains with omitted members
at the start and middle, a small final join member, unchanged results, callee
renaming, all-small groups recovering the exact sequential module and the
numeric cutoff boundary. CLI tests require explicit `--par` and reject missing,
repeated, malformed or overflowing limits. The permission judgment is unchanged;
the actualization ledger separately reports omitted offers.

For quadrature this removes the density/Simpson offers, preserving the recursive
pair and its88-byte task frame. Original and filtered modules are compiled from
the **same WF source** and linked into the same executable. The filtered
instrumented centered-peak calls publish1,643 tasks versus8,219 in the original;
the full per-node work and exact binary64 result remain the same. Local M1
assembly allocates160 bytes for the filtered recursive activation versus144
original and128 sequential: the improvement below cannot be explained merely
by a smaller stack frame. Suppression also changes inlining/register allocation,
so the paired benefit is not an isolated atomic/publication cost.

The five-form M1 screen retains ordinary SHA256
`c181ea855e5c0d4a6b71a0c56aaee9cb9fd119b9a776d338293e027707ae2e9f`,
unchanged original WF object
`99faabcaf9279d19e41e1423d8f71a8bb7e8e22e769b8dc61e51a8523e1d9bcc`
and filtered object
`1111aea3e8f58a940238204029a1f4b2bfb442bebcf115f75279278ddc187d7a`.
The host, flags, first/warm boundaries and placement limitations match the M1
screen above, but this is a separate cohort. Values below are microsecond
medians over five process warm means, all with requested width4. Sequential
forms execute one thread regardless of that request.

| Input | WF sequential | Original parallel | Filtered parallel |
| --- | ---: | ---: | ---: |
| Center peak | 21.844 | 39.547 | 16.042 |
| Left peak | 16.380 | 32.469 | 13.359 |
| Right peak | 16.281 | 32.276 | 12.735 |
| Outside peak | 3.375 | 10.792 | 4.875 |
| Depth cap | 52.547 | 77.292 | 26.313 |

Against W1 sequential, filtered W4 paired ratios for centered/left/right peaks
and the depth-cap case are0.734 [0.705–0.750],0.819 [0.780–0.882],
0.785 [0.750–0.811] and0.499 [0.485–0.558], each faster in all five pairs.
Against original W4 they are0.400/0.411/0.382/0.332, also all five faster.
All ten cases improve versus original W4 in all five pairs. However,
smooth/outside/loose/reverse still lose to W1 sequential in all five;
depth-zero/empty are mixed. No adverse or first-call sample is removed.

The remaining small-call losses motivate recursive granularity work. A useful
four-worker gain on some fixtures is not a top-tier parallel-reference result:
larger inputs/compositions and strong native recursive comparisons remain
required. This original screen did not select a default; the later integration
choice above also considers the following Linux comparison and preserves an
explicit unfiltered control.

The [five-form Linux scalar-leaf run at `8f7eed90`](https://github.com/mbbill/Whitefoot/actions/runs/34232662823)
retains artifact10058498265, ZIP SHA256
`f8dd6d24914b590c9f8c8d5d51727f97466d6419322ae4559ab98e8560747626`.
All25 hashes, nine source snapshots,50 processes/4,500 calibration calls and
20 qualifiers/400 results match that exact revision. Ordinary image SHA256 is
`466d7af03aa51dd0d920432c7358ad8c09bd9cf0c329c7bd99dcbccb53cedaf6`.
This is EPYC7763, two physical/four SMT CPUs under mask0–3, Clang18.1.3
scalar x86-64-v3, with placement/frequency/quota unqualified. It differs from
the preceding EPYC9V74 cohort. Leaf W4 improves over original W4 in all five
pairs for every case. However, centered/left/right peaks and depth cap have
leaf-W4/W1-sequential paired medians1.094 [0.972–1.636],1.115 [1.101–1.680],
1.095 [1.003–1.254] and1.115 [0.981–1.188]; only1/0/0/1 pairs improve.
Center medians are32.521 us sequential,81.990 original parallel and35.600
filtered parallel. Depth-cap original warm process means range189.226–641.830
us, all retained. Thus the M1 speedup over sequential does not repeat here;
no OS cause is assigned from aggregate CPU/context-switch observations.
The matching [complete gate](https://github.com/mbbill/Whitefoot/actions/runs/34232663071)
passes, including the deterministic strong-runtime linkage observer.

### Native recursive grain comparison

`quadrature_native.cpp` supplies a scalar C++ sequential specialization and
direct recursive oneTBB `parallel_invoke` / Parlay `par_do` specializations.
All use the same density, Simpson arithmetic, stopping condition and ordered
left-plus-right result. At spawn depths0/2/4/8/24, nodes below that frontier
call the sequential specialization with no scheduler branch. Depth24 offers
every nonterminal pair in these fixtures; depth0 includes native pool entry
but no recursive forks. These are five initial grain settings, not a fully
tuned envelope or a selected WF policy. The existing scalar C recursion and
both WF modules remain in the same executable and use the same oracle.
The C++ entry boundary differs from the generated WF adapter; `cpp-seq` and
depth0 expose kernel/entry overhead before attributing differences to scheduling.

oneTBB uses the pinned v2023.1.0 library, a persistent arena with one reserved
external participant, and `max_allowed_parallelism` equal to the requested
width. Each `parallel_invoke` uses its default bound context; shared contexts
and alternative task APIs are not yet tuned. Parlay uses pinned native
`51017699`, a caller-owned width-sized pool and the default elastic idle policy.
Only the selected runtime starts a pool in each process. Creation is inside
the first timed run; Parlay helper destruction follows all joined measurements.
oneTBB retains its process-lifetime pool. No CPU pinning or idle-policy override
is silently introduced. Builds verify the upstream pins and source cleanliness
through the existing dependency caller; artifacts retain the library, header
hashes, compiler commands, C++ assembly, objects, source snapshots and report.
The native implementation follows the documented
[oneTBB fork/join contract](https://uxlfoundation.github.io/oneTBB/main/specification/source/algorithms/functions/parallel_invoke_func.html)
and the [pinned Parlay implementation](https://github.com/cmuparlay/parlaylib/blob/51017699dcc421f80479cdb238d3092233ad0d26/include/parlay/parallel.h).

The diagnostic build counts executed nodes, application-level fork pairs and
branches beginning on a different thread from their parent. Counters are
subtree-local and combined after joins. Nodes and forks must match the
independent explicit-stack oracle; worker request1 must observe no migrated
branches. These are not upstream internal task counts or OS context switches.
The existing `steals`, `pool_lanes` and publication fields remain WF-specific;
zero there says nothing about native pool participation. Both native runtimes
observe migrated branches in the four-worker positive-depth qualification
cohort, but that does not prove all four participants were active simultaneously.
The timing build omits native counting and thread-ID reads; all event fields
are zero. Per-call process CPU and OS context switches remain raw observations,
not instruction counts or an isolated task-switch cost.

The September8 M1 native screen (MacBookPro18,3, Clang21.0.0, unfixed placement
and frequency) retains ordinary SHA256
`35dcc23fb6575b28a216453235cb3c12f05000597445703ab9d7cc0efccc3bf0`,
native C++ object `9ba3bced70575c8f0cb4b426f3603890bdedae1fcc010d37bfa4abd4965bfff2`
and the unchanged filtered WF object
`1111aea3e8f58a940238204029a1f4b2bfb442bebcf115f75279278ddc187d7a`.
The ordinary executable is a new cohort, not the preceding five-form image.
All sixteen settings, all ten inputs and all first/warm samples are retained.
Selected four-worker medians of five process warm means, in microseconds:

| Input | WF leaf | TBB depth4 | TBB depth24 | Parlay depth8 | Parlay depth24 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak | 15.495 | 14.896 | 164.396 | 10.724 | 16.755 |
| Left peak | 13.250 | 19.073 | 128.427 | 8.703 | 13.588 |
| Right peak | 12.823 | 12.735 | 124.948 | 9.995 | 15.333 |
| Outside peak | 4.760 | 7.396 | 30.187 | 5.453 | 6.094 |
| Depth cap | 26.323 | 21.386 | 381.568 | 18.188 | 31.516 |

For centered/left/right peaks and depth cap, paired WF-leaf/Parlay-depth8
ratios are1.478 [0.864–1.567],1.571 [1.284–1.620],1.324 [0.738–1.515]
and1.526 [1.030–1.600]. Parlay is faster in4/5,5/5,4/5 and5/5 pairs
respectively. Some cases have substantial between-process variation; an
observed median win is not held-out confirmation. TBB depth2 has a depth-cap
median19.161 us and beats WF in5/5 pairs. TBB depth4 beats WF
on center/depth-cap in4/5, while losing on left peak in5/5. Thus neither one
runtime nor one depth wins every input. C/C++ sequential centered-peak medians
are21.651/21.656 us; this screen does not identify a large scalar-kernel deficit.

For center peak, depth8 executes247 fork pairs versus1,643 at depth24;
all versions still visit3,287 nodes. The WF leaf body offers the latter1,643
pairs. The comparison exposes a profitable granularity range beyond small-leaf
suppression, but changes both scheduler and grain: it is not an isolated WF
runtime deficit. Full-depth TBB alone would be a particularly weak reference.
The matched-grain WF controls below address part of this gap. Finer grain
tuning with held-out confirmation, recursive Rayon, larger compositions and
native-host topology/placement qualification remain required. No compiler/runtime/ABI
policy is adopted by these reference controls.

### WF runtime at matched recursive grain

`wf-native` and `wf-value` add the recovered WF runtime to the same C++ kernel
at the same five spawn depths. Both publish the left subtree, compute the right
on the current stack, join before reading the left result, and release the slot.
Refused acquisitions execute both children locally with the remaining spawn
budget; they do not omit work. Depth0 uses the common sequential specialization
without starting a WF pool. Width1 at positive depths still attempts acquisition
with no active pool. Pool configuration is checked before first use. These are
native research adapters, not compiler-generated WF or borrowing/ABI changes.

`wf-native` stores an8-byte pointer to the parent's live closure in the runtime
frame. That closure references parent-stack arguments and the left result;
join protects their lifetime. `wf-value` copies nine doubles, two unsigned
depth fields and a scalar result into an88-byte frame, matching the generated
WF payload size but not its layout or ABI. Its callback copies arguments into
C++ storage, computes, and writes result bytes. The owner reads after join and
before release. `memcpy` accesses raw C-owned storage without assuming a
constructed C++ object's lifetime there. Both use unchanged runtime entry
points and no extra scheduler, I/O or stack switching.

Subtree-local diagnostic migration counts must equal the WF successful-steal
counter. At width4, publication plus slot refusal equals the independent
oracle's exact fork count; publication, local-pop-plus-steal, run and join
totals agree. The diagnostic value frame grows to120 bytes for counters/thread
identity; its timing is not ranked. Four `exhaust` qualifier processes reserve
all64 owner slots without publishing before invoking each native WF form at
depth24. Results still match, with zero publications and one refusal per fork
in the instrumented image. Reservations are released after the joined checks.
Width1 and exhausted-pool paths are covered; general runtime exhaustion and
interleaving tests remain required.

The September8 M1 matched-grain cohort retains ordinary SHA256
`51f111527e44019f1bf5c5600322ab1f0b2f36c4ed7ef64e6260f480a87456cf`,
native C++ object `c3232552a931dc7e35ee23a04b2241147ca7ed1721db6f2ed3c79b4014baf4ff`,
and the unchanged filtered WF object
`1111aea3e8f58a940238204029a1f4b2bfb442bebcf115f75279278ddc187d7a`.
It uses MacBookPro18,3 / Clang21.0.0, scalar flags, unfixed placement/frequency,
five passes and all26 settings. All260 processes/23,400 first/warm results are
retained. The104 normal and four exhaustion qualifiers check2,160 results;
the four maintained malformed reports reject. The preceding native cohort is
separate. Below are microsecond medians of five process warm means at width4.

| Input | Generated WF leaf | WF pointer depth8 | WF value depth8 | WF value depth24 | Parlay depth8 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak | 16.021 | 12.292 | 11.198 | 16.281 | 10.917 |
| Left peak | 13.594 | 10.318 | 9.609 | 14.438 | 8.552 |
| Right peak | 12.672 | 8.672 | 8.297 | 13.537 | 9.922 |
| Outside peak | 4.677 | 5.146 | 4.318 | 4.813 | 5.177 |
| Depth cap | 26.000 | 17.589 | 16.839 | 27.531 | 18.011 |

Paired value-depth8/Parlay-depth8 medians for center/left/right/depth-cap are
1.008 [0.901–1.064],1.124 [1.081–1.165],0.830 [0.796–0.892] and
0.931 [0.740–0.989]. Native WF wins2/0/5/5 of five pairs, not uniformly.
The pinned Parlay `fork_join_scheduler::pardo` publishes its right callback
and executes left locally; these WF adapters publish left and execute right.
Thus that matched-grain cohort does not match fork direction. The reciprocal
controls below test this candidate explanation before assigning left/right
differences to scheduler overhead.
Against generated WF leaf, value-depth8 ratios are0.696/0.705/0.650/0.650,
all five pairs faster. However, full-depth native value ratios against generated
WF are1.043/1.059/1.053/1.074, with only1/0/0/1 faster pairs. These full-depth
cells do not identify a large compiler-code deficit; useful grain selection
remains a concrete gap between generated WF and native controls.

Frame byte count alone does not predict the result. At depth8 the88-byte value
frame beats the8-byte pointer form in5/4/5/4 pairs on those four heavy cases,
paired medians0.903/0.931/0.969/0.959. M1 assembly allocates272 bytes for the
borrowed recursive activation and176 for the value activation, versus160 for
generated WF leaf and128 for the shared C++ sequential specialization. These
are local activation sizes, not peak stack usage. Different closure access,
inlining and register allocation remain confounders; no isolated copying/cache
cost is inferred. Other inputs favor another grain (outside peak value-depth4
is2.657 us versus depth8's4.318 us). No fixed depth becomes a compiler default.
Larger compositions, topology-qualified hosts and an input-adaptive policy
still need evidence.

### Linux matched-grain confirmation

The separate Linux cohort at `e6d4bcbe` is retained in
[run34237663501, artifact10060643475](https://github.com/mbbill/Whitefoot/actions/runs/34237663501/artifacts/10060643475).
Its ZIP SHA256 is
`92122d4b6d1b05584bbc8a8c26f764916206447c79cbdd85e7d737bc94a48340`;
ordinary image SHA256 is
`bc177d8ba6097ce3103d41b50ce180014cc42b8c147b74e9e93855fc9d60d4eb`.
All260 calibration processes/23,400 calls,108 qualifier processes/2,160 results,
46 manifest paths,12 exact-revision sources and213 dependency headers pass the
independent evidence check. The host is an EPYC7763 Microsoft VM with two
physical/four SMT logical CPUs under mask0–3, Clang18.1.3 and scalar
x86-64-v3/no-LTO/strict-FP flags. Individual placement, frequency and quota are
unqualified. Below are width4 microsecond medians of five process warm means.

| Input | Generated WF leaf | WF value depth8 | Parlay depth8 | WF value depth24 |
| --- | ---: | ---: | ---: | ---: |
| Center peak | 34.992 | 24.553 | 30.789 | 43.033 |
| Left peak | 29.621 | 17.488 | 28.967 | 30.060 |
| Right peak | 27.037 | 17.248 | 31.282 | 36.166 |
| Depth cap | 82.379 | 50.980 | 53.249 | 88.597 |

Paired value-depth8/generated-leaf ratios are0.702/0.592/0.634/0.610,
with5/5/4/5 faster pairs. Against Parlay-depth8 they are0.743/0.606/0.551/0.943,
with4/5/4/4 faster pairs. Full-depth value/generated-leaf ratios are
1.209/0.980/1.301/1.079, with0/3/0/0 faster pairs. This supports investigating
useful grain, not treating the native adapter as an already improved compiler.
The native adapters in this artifact still publish opposite sides; it does
not confirm the later reciprocal-direction comparison on Linux.
All outliers remain: one Parlay center process averages140.179 us versus the
30.789 us median. Depth-cap median process CPU time is120.375 us for value-depth8
and68.625 us for Parlay-depth8 despite similar wall times. Aggregate CPU time
cannot assign this difference to a queue operation, runtime spinning or the OS.

### Reciprocal fork direction

`parlay-left` calls pinned Parlay `par_do` with the callbacks exchanged, making
it publish the left subtree and execute right locally. `wf-value-right` sends
the right subtree's arguments through the existing88-byte native WF frame,
executes left locally and copies the joined result into the right result slot.
Both preserve their direction in nested granted forks. The return remains
`left.value + right.value`; arithmetic and stopping conditions are unchanged.
Budget-zero sequential execution and refused-acquisition left-then-right
fallback are unchanged. The new WF direction participates in ordinary and
instrumented full-owner-slot exhaustion checks. No generated WF, runtime or
ABI policy changes.

The September8 M1 direction cohort uses the same host/toolchain/scalar flags
and placement/frequency limitations as the matched-grain screen. Its retained
ordinary SHA256 is
`cde6119402712f53a3fa10b04428880a87192f98cfcf7e0c5d85cb51cdb47658`,
native C++ object `d71396fdc3702783f0867156aa7fc70c1cfbad6935481e2d1070c1fb11dccb36`,
and generated leaf object remains
`1111aea3e8f58a940238204029a1f4b2bfb442bebcf115f75279278ddc187d7a`.
All360 processes/32,400 calls and150 qualifiers/3,000 results are retained;
the prior26 forms/grain settings remain among36. All four maintained malformed
reports reject. This is a new same-image cohort, not a re-use of earlier timings.
Below are microsecond medians of five process warm means at width4 and depth4.
Column directions identify the subtree offered to another worker, not a
guarantee that it was stolen on every call.
The columns use `wf-value`, `wf-value-right`, `parlay-left` and `parlay`,
respectively; the WF columns both use by-value frames.

| Input | WF sends left | WF sends right | Parlay sends left | Parlay sends right |
| --- | ---: | ---: | ---: | ---: |
| Center peak | 10.911 | 10.755 | 13.828 | 18.151 |
| Left peak | 13.104 | 10.787 | 24.724 | 11.141 |
| Right peak | 10.760 | 12.818 | 10.901 | 23.010 |
| Depth cap | 15.703 | 15.682 | 18.245 | 17.151 |

For left/right peaks at depth4, paired WF-right/WF-left medians are
0.823 [0.811–0.843] and1.200 [1.187–1.266]: left improves in all five pairs,
right loses in all five. Parlay-left/Parlay-right ratios are
2.138 [1.958–2.458] and0.470 [0.382–0.630], with the reciprocal all-five
loss/gain. At depth8 these effects are smaller: WF direction ratios are
0.896/1.082 (4/1 faster pairs), Parlay ratios1.155/0.862 (0/5 faster pairs).
Center and depth-cap direction changes remain mixed. The reciprocal skew
response supports fork direction as a material factor in this experiment;
different template code generation and process timing still prevent assigning
the entire change to a single queue operation.

At matched left-publication depth8, WF/Parlay ratios for center/left/right/cap
are1.075 [1.035–1.165],0.982 [0.972–1.034],0.996 [0.895–1.056] and
0.989 [0.961–1.068], with0/4/3/4 faster pairs. The previous opposite-direction
left/right all-five loss/win is not an established cross-runtime gap. Center
still favors Parlay in all five pairs. Matching right-publication likewise
gives left/right medians1.016/0.888 with2/4 faster pairs. These observations
neither establish equivalence nor select a universally faster scheduler.

Do not choose a default source-call order from two mirrored peaks. The caller
does not know the subtree work in advance; direction and useful grain need
input-adaptive evidence. The next optimization should reduce generated
fine-grained offers without changing ordinary function ABI, then measure its
actual work, refusals and scaling on larger compositions and qualified hosts.

### Queue occupancy admission screen

A September8 M1 screen tests whether rejecting acquisition when the owner's
queue already contains enough jobs can improve the existing generated code.
This is a rejected default-policy candidate, not an implemented runtime option.
At `bb80763f`, the generated refusal edge still calls the parallel function;
its descendants continue trying acquisition. It does not select the internal
sequential clone. The control changes neither the generated objects nor the
ordinary frame/join/release ABI.

To reproduce the admission rule in a scratch copy of `runtime.c`, insert the
following immediately before `index = lane->free_head` in
`wf__par_acquire_lane`, after attaching the calling lane. Compile the runtime
with `WF_QUEUE_SCREEN_LIMIT` set to0,1 or4. Zero omits the added loads/test.
Relink each runtime object with identical qualified quadrature host, floor,
native C++, original WF and filtered WF objects, preserving scalar flags and
library dependencies from `quadrature-build`.

```c
#if WF_QUEUE_SCREEN_LIMIT > 0
    unsigned long long queued_bottom = __atomic_load_n(&lane->bottom, __ATOMIC_RELAXED);
    unsigned long long queued_top = __atomic_load_n(&lane->top, __ATOMIC_ACQUIRE);
    if ((long long)(queued_bottom - queued_top) >= WF_QUEUE_SCREEN_LIMIT) return NULL;
#endif
```

The owner reads its bottom and a potentially stale thief-updated top. An early
refusal changes scheduling; no slot has been acquired and no argument payload
has moved. Existing exhaustion handling remains. This heuristic sees queued
job count, not subtree work, idle-worker demand or the dependency critical path.

The screen reuses the retained direction-cohort objects and dependencies on
MacBookPro18,3 / Clang21.0.0, without SIMD, FP contraction, fast-math or LTO.
The new images are separately linked: equal input object bytes do not imply
equal final instruction addresses. Their SHA256 identities for limits0/1/4 are
`576b3089fc4bd7f3aeb2773de59ef27f2814ad8569b9d05ac9dd139db1ad8e22`,
`09dca7f5ed5d92c09509a780e2efe42784d762671365ed8990608b8e78881e32` and
`9f65b0a1c8331dbe5f555d022f3d8f2ee7d39c091201ee14406aa8316cf92660`.
No timing image is rebuilt after measurement. Placement and frequency are not
fixed. This is a local screening result, not a Linux or hardware-limit claim.

Each of the three images runs `wf-leaf`, `wf-auto`, `wf-leaf-seq`, `wf-value 8`
and `parlay-left 8`, at widths1/4, on all ten existing inputs. Thirty ordinary
`check` processes check600 results. Five passes run150 `bench` processes with
13,500 checked calls, retaining first calls and all outliers. The tuple order
is limit, width, form; even passes reverse that complete order. Processes run
alone, with no concurrent builds or diagnostic runs. Existing `quadrature.awk`
checks each ordinary report. Its raw header binds mode, form, width and
instrumentation, but queue-limit identity is bound by the build recipe and
filename rather than a report field. The table reports width4 generated `wf-leaf` warm
means, aggregated to a median over five processes, in microseconds.

| Input | No queue limit | Limit1 | Limit4 | Paired limit1/base | Paired limit4/base |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak | 15.469 | 38.250 | 15.094 | 2.445 | 0.988 |
| Left peak | 13.703 | 13.937 | 13.776 | 0.995 | 0.999 |
| Right peak | 12.542 | 42.068 | 15.787 | 3.349 | 1.184 |
| Depth cap | 26.573 | 36.078 | 24.120 | 1.361 | 0.930 |

Limit1 loses all five center/right/depth-cap pairs. Limit4 gains all five
depth-cap pairs but loses all five right-peak pairs; center and left are mixed.
The sequential-clone controls have mixed directions on these inputs. The
unfiltered `wf-auto` benefits more from limit4, but remains slower than the
filtered generated form. This does not justify restoring small-helper offers.

A separate ASan/UBSan diagnostic uses a new lane-local queue-refusal event,
kept distinct from actual slot exhaustion. It appends `queue_refusals` to the
report and checks publications + slot refusals + queue refusals against the
independent oracle's acquisition opportunities. Joined publication/pop/steal/
run/join conservation, bitwise results and native-node checks remain. Thirty
processes check600 results; generated WF and the oneTBB library remain ordinary
objects as in the existing qualifier. Instrumented timings are not compared
with the ordinary images. One warm `wf-leaf` observation per input gives:

| Input | Base publications | Limit1 publications / queue refusals | Limit4 publications / queue refusals |
| --- | ---: | ---: | ---: |
| Center peak | 1643 | 172 / 1471 | 933 / 710 |
| Left peak | 1236 | 78 / 1158 | 833 / 403 |
| Right peak | 1236 | 246 / 990 | 693 / 543 |
| Depth cap | 4095 | 281 / 3814 | 1477 / 2618 |

Actual slot refusals are zero in these observations. The large decrease in
publications therefore does not mean fewer acquisition attempts or uniformly
better scheduling. These diagnostic calls are separate executions, so their
counts cannot explain each timed call or identify a unique cause of slowdown.
The result rejects simple occupancy1/4 as a broadly useful default for this
corpus. It does not reject all demand policies or the current-stack model.
The next useful distinction is between avoiding task machinery while continuing
parallel recursive calls, and entering an ordinary sequential subtree; that
requires separately justified compiler selection and broader work-distribution
evidence before adopting a policy.

### Sequential subtrees after refusal

`--par-sequential-refusal` is an opt-in compiler experiment. At an ordinary
compute hand-out's refused join edge, a non-suspending callee with an existing
sequential clone calls that clone with the same arguments and `FunctionAbi`.
Successful callbacks and the source-last inline call still use parallel code.
The clone declines descendant compute permissions and returns on the same
stack. No hidden parameter, runtime demand query, public signature effect,
source acceptance rule or default policy changes. Callees without clones,
may-suspend calls and staged completion retain their existing fallback. Retire
the experimental switch when a qualified general actualization policy replaces
it; it is not a standalone claim of an optimal grain policy.

The backend's deterministic recursive fixture uses shared borrowed input and
checks both scalar and destination-passed aggregate results. Its32 leaves each
contribute2, producing64. The original all-refused lowering attempts31 tasks;
the experimental form attempts5 down the inline right spine. Granting exactly
one root task and delaying its callback until join produces31 versus9 attempts,
with exactly one release and unchanged results. Eight executable schedules
check these cases. A leaf without a clone and a staged may-suspend fixture also
retain byte-identical modules. The38-test parallel backend module and the new
CLI composition/invalid-option test pass.

`wf-refusal` and `wf-refusal-seq` use a third generated module compiled with
`--par --par-scalar-leaf-limit 16 --par-sequential-refusal`; they select its
parallel entry and sequential clone respectively. The original and filtered
modules remain byte-identical to the preceding direction-cohort modules.
At that refusal-control revision, `check-quadrature` covered all38 settings at widths1/4 in ordinary/instrumented
images, plus eight full-owner-slot exhaustion processes:160 processes and3,200
checked results. The generated refusal form joins the existing native exhaustion
controls. Independent explicit-stack oracle state identifies nodes on the
all-right path. With all owner slots held, only those internal nodes attempt
acquisition:5/7/6/13/10/1/0/12/0/6 across the ten inputs. Both host and AWK require
those exact counts; a malformed right-spine count is the fifth negative report.

Normal generated-refusal diagnostics permit the number of attempts to lie
between that right-spine count and the full tree's internal-node count, since a
refused subtree stops making attempts. Zero refusals still requires the full
count. Every other form retains its previous exact opportunity equation, and
all forms retain joined publication/pop/steal/run/join conservation and bitwise
results. These bounds do not claim a dynamically observed WF computation-node
count. An earlier unmeasured qualifier directory failed the old `wf-leaf`
actual-steal witness during concurrent compiler tests; that failure is retained
and excluded. The fresh qualification ran without concurrent compilation and
passed, without removing the witness or retrying individual failed cells.

The fresh M1 qualification image SHA256 is
`82a47d20186c2cb0755de474faaf922999e70089760eeb3a44a770e016a39608`,
the generated refusal object is
`0334dfc3571d7ab23dfd1c609e3a8c33af5fbeaf63793b842fc656c54394cd37`,
and the retained compiler is
`8fe40280083a9bc28909a1b1dfdceb42f0c6bf3c7e63f27125b3e1d66fbb24c7`.

A fresh admission screen repeats the preceding scratch queue rule with these
objects. Each of three images contains both generated policies, their sequential
controls, native WF value-depth8 and Parlay-left-depth8. The shared objects are
identical across limits0/1/4; within each image the two compiler policies can be
compared directly. Six forms ×two widths ×three limits ×five alternating-order
passes give180 processes/16,200 calls, plus36 ordinary qualifiers/720 results.
No native timing runs overlap builds or diagnostic execution. MacBookPro18,3,
Clang21.0.0, scalar strict-FP/no-FMA/no-LTO, unfixed placement/frequency and the
previous queue screen's separate-image/recipe-bound-limit caveats apply.
The three ordinary image SHA256 identities in limit0/1/4 order are
`f2ea69daa4cdbc5562531d4f26398907f1f350e2baa06459c49423b95098ccab`,
`35ec3520c06414fa338c0c6395cb84132b1a421e8037de2cbd1b66c22708e9b4` and
`9a9b99806b1856861b75e99716c43991430c27c8d6e2a66313b6539b3f719cf8`.
No measured image is rebuilt. Microsecond medians of five process warm means
at width4 follow; the last column is a median of within-pass paired ratios.

| Input | Existing leaf, no limit | Refusal clone, no limit | Refusal clone, limit1 | Refusal clone, limit4 | Limit4 clone / existing no limit |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak | 15.969 | 15.630 | 11.938 | 14.896 | 0.933 |
| Left peak | 14.036 | 13.849 | 7.729 | 12.833 | 0.906 |
| Right peak | 12.646 | 12.589 | 16.735 | 11.687 | 0.931 |
| Depth cap | 25.734 | 25.276 | 27.505 | 21.542 | 0.835 |

Without a queue limit, new/old compiler-policy ratios are1.023/0.981/1.037/0.990,
with1/4/2/3 faster pairs; no stable no-refusal benefit is established.
At the same limit1, new/old ratios are0.316/0.631/0.392/0.804, all five faster
on these inputs. That recovers much of the failed admission-only performance,
but comparing the combined limit1+clone candidate against the existing
unlimited form gives0.777/0.558/1.317/1.077 with5/3/0/1 faster pairs. It still
hurts right skew and depth cap. Combined limit4+clone against existing unlimited
code gives the table's ratios with4/4/4/4 faster pairs. Every adverse pair and
first call remains; no fixed queue limit is promoted to default.

The new compiler form has not caught the tuned native grain. Within the limit4
image, generated-refusal/Parlay-left-depth8 ratios are1.331/1.287/1.369/1.261;
against native WF value-depth8 they are1.338/1.358/1.333/1.288. All five pairs
lose on all four inputs. These are matched-left publication comparisons through
the same runtime images, with different grain choices, not isolated compiler
instruction costs or proof that either native library is the ceiling.

A separate36-process/720-result ASan/UBSan diagnostic adds a distinct
`queue_refusals` event as in the previous screen. In generated-refusal forms,
slot plus queue refusals select the opportunity bounds; zero combined refusals
still requires the full count. Original forms retain exact accounting with
queue refusals added. One warm center call at limit1 reports112 publications
and1,531 queue refusals for existing generated code, versus7 publications and32
queue refusals with sequential subtree calls. At limit4 those observations are
841+802 versus821+249. Actual slot refusals are zero. This distinguishes fewer
published tasks from fewer acquisition attempts, but the instrumented executions
are separate from timed calls and cannot assign an exact fraction of elapsed
time to those counters. The result supports further compiler grain experiments;
larger compositions, Linux replication and a useful general admission policy
remain unqualified.

### Sustained batches and regional counters

`quadrature-batch-calibrate` adds a separate throughput/cost-attribution panel:

```sh
make -C research/experiments/compute-runtime quadrature-batch-calibrate \
  OUT=/absolute/qualified-output RESULTS=/absolute/fresh-batch-results
```

The existing `ordinary` image also accepts `batch form [spawn-depth]` with
`WF_QUADRATURE_INPUT` selecting one of the ten existing fixtures and
`WF_QUADRATURE_REPEATS` in1..65536. Each process constructs the independent
explicit-stack oracle, checks eight warmup results, then executes the selected
integration repeatedly. The loop accumulates a bitwise mismatch for every
result and checks it after timing. It has one common indirect dispatch per
integration, loaded once through a volatile function pointer before warmup;
the visible C kernel cannot be hoisted out of the loop. The retained M1
assembly has an indirect call on every loop iteration. There is no per-call
clock, printing or oracle construction in this interval. All calls join before
the next call begins; this does not model overlapping requests or nested
application composition.

`getrusage(RUSAGE_SELF)` records process-wide user/system CPU,
voluntary/involuntary context switches and minor/major faults. Batch format v2
also records `CLOCK_PROCESS_CPUTIME_ID` and `CLOCK_THREAD_CPUTIME_ID` deltas in
nanoseconds. The resource interval encloses the process CPU clocks, which
enclose the caller CPU clocks, which enclose the wall clocks. The caller
measurement excludes helper CPU; none of these clocks separates useful work
from spinning. API disagreements are observations, not acceptance thresholds.
Frozen v1 cohorts retain their original host and reader without the new fields.
CPU divided by wall is average process CPU concurrency, including spinning
helpers, not useful-work occupancy. Zero-resolution wall observations remain
valid raw records but cannot supply a ratio denominator. Pool width follows
actual offers: isolated empty/terminal-only filtered inputs need not initialize
the lazy WF pool even with a four-worker request.

On Linux, the collector probes each requested `perf` event on the actual host:
task-clock, context-switches, cpu-migrations, page-faults, cycles, instructions,
branches, branch-misses, cache-references and cache-misses. Unavailable events
retain their probe output and are never replaced with zero. With available
events, a separate observer runs `perf stat --delay=-1 --control=fd:3,4`; the
host enables counters after warmup and waits for acknowledgement, then disables
them after the batch resource snapshot. The inherited process scope includes
worker threads, not other processes on the CPU mask. This follows the
[upstream perf control and inheritance interface](https://raw.githubusercontent.com/torvalds/linux/v6.8/tools/perf/Documentation/perf-stat.txt).
Counts include boundary acknowledgement/resource/clock/disable work. The
enable-ack gap can also change helper parking before the batch, so perf/plain
differences are not a pure counter-overhead subtraction. Raw event runtime and
running percentage are retained; missing, duplicate, not-counted or zero-runtime
selected events fail report qualification. Multiplexed counts are not exact
simultaneous instruction/cycle attribution. Event availability alone does not
qualify PMU accuracy or cache-event semantics on a particular CPU.

The collector retains the selected perf command, its hash, build options and a
verbose task-clock attribute probe before timing. A command may be a wrapper;
the CI job selects the installed tool directly. The Linux compute job records its attempt
to permit process counters on the ephemeral hosted runner. It runs this panel
after the original short-call calibration on the same recorded CPU mask.
Five alternating whole-cell orders cover eighteen forms, worker requests1/4 and
four heavy inputs:720 plain processes, plus720 perf processes if at least one
event is available. Each uses4,096 repetitions by default; `ROUNDS` and
`REPEATS` can select an explicitly recorded different panel. The forms are C
native, generated WF sequential/leaf/refusal and frontier/its sequential clone at depths4/8,
C++ sequential, native WF value
depth8, Parlay-left depth4/8, oneTBB depth8, Rust sequential and both Rayon
directions at depths4/8. This is not a general grain
search. There is no added queue-occupancy cap here; the preceding queue-limit4
candidate is a different experiment. Normal owner-slot capacity still applies.

`check-quadrature` additionally invokes `quadrature-batch.sh check`. Its284
successful batch executions check3,124 outputs:272 ordinary/sanitized cells,
ten input-selector cells and two FIFO protocol encodings. It checks the
documented acknowledgement line and perf versions that append a NUL. Five
negative probes cover wrong repeat identity, unpaired control descriptors,
missing perf events, uncounted events and zero event runtime. The report
validator binds the fixture's expected result as well as its input/work count.
Default check result paths are unique; explicit results and calibration paths
must be fresh. Both new collector files belong to this panel and retire with
it. Existing short-call/exhaustion/task-conservation checks remain in place.

The first M1 batch cohort has360 plain processes,1,474,560 timed calls and2,880
checked warmups. MacBookPro18,3/Clang21.0.0, scalar strict-FP/no-FMA/no-LTO,
unfixed placement/frequency and no overlapping native timing/builds apply.
The ordinary image is
`4da81c9e4a66147230da16e1e7a4797d873b1755b18a1b41d8305218768d93c1`;
the host object is
`2b5395b0277dbed8b0cb7aa8ff027af165f0705674a6d5d228750967fb418d8a`.
The measured collector/validator SHA256 identities are
`a0468a679f0bc7a5bc419b13b1c0332de3aef9ea4fe2c037ed09d729088413de` and
`a8ce99464b3b47107b6018d8793dfad889e33559c6ccbe0dddf3a4f1b8a471f1`.
The build's original140-process batch qualification had two negative probes;
the final collector was then qualified in a fresh140-process replay including
all five negatives before timing. Its manifest is separate from the original
build source snapshot. The existing short-call qualification also passes160
processes/3,200 results and its five malformed-report probes. An earlier
unmeasured batch build exposed a wrong lazy-pool-width expectation; another
exposed shell-function environment persistence in a negative test. Both were
fixed before the retained image and final checker replay. A sandbox-denied CPU
metadata read also stopped before timing; the complete cohort uses a fresh
directory and a successful host record.

Width4 medians of five process observations follow, in microseconds per
integration. CPU is the sum of user and system time over all process threads.

| Input | Generated leaf wall | Generated leaf CPU | Native WF depth8 wall | Native WF depth8 CPU | Parlay-left depth8 wall | Parlay-left depth8 CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Center peak |15.377|59.697|10.722|40.303|10.284|41.137|
| Left peak |13.576|49.471|9.084|32.857|9.374|37.395|
| Right peak |12.889|49.014|8.674|32.926|8.847|34.818|
| Depth cap |25.512|99.096|17.068|64.100|17.745|69.911|

Generated-leaf/Parlay-depth8 paired wall ratios are1.515/1.448/1.488/1.438;
the corresponding CPU ratios are1.489/1.306/1.433/1.417. All five pairs lose
in wall and CPU on every input. Native WF depth8 also beats generated leaf in
both quantities in all five pairs. Generated-leaf CPU/wall medians are3.882/3.842/3.775/3.863;
Parlay depth8 is3.951/3.928/3.941/3.922. Generated WF's sequential clone versus
C++ sequential has paired wall ratios1.001/1.010/1.011/1.008 at worker request4.
This prioritizes excess parallel CPU work, including possible spinning, and
the generated grain/call paths over a large scalar-kernel deficit or mostly
sleeping workers. It does not separate acquisition, bookkeeping, cache misses
and spinning, or establish a general runtime limit.

Scheduling policy changes OS observations too. Parlay-left depth4's center,
left and right inputs have median11,500/27,062/8,395 involuntary switches per
4,096-call batch, with zero median voluntary switches; depth8 has75/72/160
involuntary switches. Left-peak wall medians are26.414 versus9.374 microseconds,
but depth-cap favors depth4 at16.860 versus17.745. These counters do not price
each switch or prove which wait/yield/preemption mechanism caused the loss.
No default grain is selected; this M1 cohort has no perf counter measurements.

The Linux v1 cohort at `4fc495d5` is retained in
[run34248179457, quadrature job102135485972](https://github.com/mbbill/Whitefoot/actions/runs/34248179457/job/102135485972),
artifact10065042253 (ZIP SHA256
`9f1fd01d4fc0ad9f5a403db857feb5ec3b71a271e846c33a40b03c5e91609612`).
Its720 processes check2,949,120 timed calls and5,760 warmups, with360 processes
per observer and1,440 software-event rows. All selected events have positive
runtime and100.00% running. Fifty-seven manifest paths and fourteen source
snapshots match; the ordinary image is
`b30304c0b748510840072b1bf5f5ba4062ba3ed44f182806d9daf67a039c355a`.
The EPYC7763 VM provides two physical cores/four SMT threads, mask0-3,
Linux6.17.0-1022-azure and perf6.17.13. Scalar strict FP/no SIMD/FMA/LTO apply.
Quota and physical isolation are unqualified. Lowering perf permissions from4
to-1 enables software observations but supplies no supported cycles,
instructions, branches, branch-misses, cache-references or cache-misses events.
No IPC, cache or stall attribution follows.

Linux plain-observer width4 medians, microseconds per integration:

| Input | Generated leaf wall / CPU | Native WF depth8 wall / CPU | Parlay-left depth8 wall / CPU |
| --- | ---: | ---: | ---: |
| Center peak |30.175 /109.962|20.381 /71.357|23.306 /78.231|
| Left peak |23.981 /85.728|16.552 /55.308|19.394 /62.523|
| Right peak |23.558 /83.925|15.645 /52.624|18.271 /59.217|
| Depth cap |64.278 /246.099|42.312 /159.443|45.319 /166.393|

Generated-leaf/Parlay-depth8 paired wall ratios are1.290/1.246/1.296/1.431,
with CPU ratios1.408/1.373/1.418/1.483: all five pairs lose on each input.
Native-WF-depth8/Parlay-depth8 wall ratios are0.876/0.860/0.874/0.935 and
CPU ratios0.911/0.881/0.881/0.958, all five pairs winning on each input.
This does not erase the M1 native comparison's mixed center/right wall pairs.
The separate Linux perf observer preserves these directions. Grain and code
generation differ between generated and native paths; this does not identify
a single cost or establish a runtime ceiling.

The Linux task-clock CSV has blank units and integer counts, treated as raw
nanoseconds for this cohort rather than conventional millisecond display.
Task-clock/rusage CPU has median1.000377 across360 processes, but a minimum
of0.727509. Seventeen rows are below0.99, all width4 Parlay-left depth4:
center/left/right/depth-cap median ratios are0.831634/0.731605/0.903807/0.990541.
For example, one left-peak batch reports278,631,449 task-clock ns against
382,994 rusage CPU us and428,631,177 wall ns. Wider positive boundary overhead
alone cannot explain this deficit. The actual packaged perf command and event
attribute dump were not retained in v1; v2 adds them and the two CPU clocks
to investigate the disagreement. The cause remains unresolved. Event runtime
is not batch wall, and perf's printed CPUs-utilized metric uses a different
elapsed interval. Neither that metric nor API agreement is substituted for
the host observations. Hardware counters and broader composition remain open.

The v2 M1 cross-check retains72 processes,294,912 timed calls and576 warmups,
ordinary image
`be35d93513de49523bc0e6dda855332c8eae5a6435a416dbca9ed951cbb663e0`.
Process-clock/rusage CPU ratios have median0.999965196 and range
0.999880355-0.999995302. This one-pass schema/accounting check is not a new
performance ranking or a resolution of the Linux discrepancy. All140 batch
qualifiers and five maintained negative probes pass. The build's collector
snapshot predates only the Linux command-resolution/copy-path adjustment;
the measured collector matches the final script. The following Linux v2 cohort
exercises the new metadata and CPU-clock paths.

The Linux v2 cohort at `1ac92939` is retained in
[run34252119791, quadrature job102148827444](https://github.com/mbbill/Whitefoot/actions/runs/34252119791/job/102148827444),
artifact10066558270, ZIP SHA256
`24d531ab520de55285064a5d31174016faa5820d0e020169b01c75ee597d1703`.
Its720 reports check2,954,880 outputs and1,440 software-event rows;58 manifest
paths and14 source snapshots match. Ordinary image SHA256 is
`56fe2f43928d4ce25162ac604685623789d18f939b074f1eeabc8114163dcaa7`;
the retained perf command is
`5fb08c90293471b24086be829f4da4707ecc83101945868101964be48267ab71`.
Perf6.17.13 build options and the verbose probe confirm inherited software
TASK_CLOCK (type1/config1), a process target and CPU selector-1. These are
probe attributes, not an attribute dump of each later controlled batch.

Across all720 reports, process-clock/rusage CPU has median0.999944763 and
range0.992985091-0.999996171. The minimum, a right-peak oneTBB perf observation,
is retained. In the perf observer, Parlay-left depth4 width4
center/left/right/depth-cap medians are
0.999988/0.999990/0.999973/0.999992. Yet task-clock/process-clock medians for
those same groups remain0.838991/0.733697/0.899252/0.987660. Thus the new API
check reproduces the disagreement between perf and process CPU accounting;
agreement of two APIs does not independently establish which is accurate.
The cause remains unresolved. This cohort is separate from v1 and the M1
frontier experiments; no accounting discrepancy is filtered out or used as a
timing acceptance threshold.

A separate M1 wait-policy screen keeps the pinned Parlay source and replaces
only `steal_job`'s inter-round `sleep_for` with a compiler-only signal fence.
At width4 the source allows801 failed attempts before requesting400ns sleep;
the requested duration is not its measured cost. The site serves both idle
search and join helping. Deques, completion synchronization and the10ms elastic
timeout remain unchanged, but scan density and timeout overshoot can change.
No policy change is adopted. Two separately linked images share the WF host,
runtime, floor and generated objects; recompiled C++ controls have identical
source, not assumed identical machine code. Ordinary sleep/spin image hashes:
`40658e805937b569b2ec8e12f576d5805fc218d125587d14cef78bc2e46d6078` /
`42b38cfa13dbad4c0f602ba9c7d4bf85fee474ba3e26989504aa842eb0a3c9dc`.
Policy identity is bound by build commands and filenames, not the v1 header.
The scratch screen qualifies80 processes/1,600 results before400 timed
processes with1,638,400 calls and3,200 warmups. Five alternating whole-cell orders
cover five forms, widths1/4 and four heavy inputs with both images.

| Parlay-left width4 input | Depth4 spin/sleep wall ratio | Depth4 CPU ratio | Involuntary switches per batch, sleep to spin | Depth8 wall ratio |
| --- | ---: | ---: | ---: | ---: |
| Center peak |0.673250|0.879117|11,506 to44|1.008827|
| Left peak |0.501516|0.687761|27,411 to15|0.973143|
| Right peak |0.976582|1.154839|9,671 to14|0.998156|
| Depth cap |0.992640|0.961308|654 to66|1.000321|

Ratios are medians of five matched pairs. Center/left/right depth4 wall wins
all five; right depth4 CPU loses all five. Every depth8 wall comparison has
mixed directions. Unchanged-source width4 controls have paired median wall
ratios0.980782-1.019537 and CPU ratios0.892159-1.042851. M1 placement/frequency
and separate-image layout remain uncontrolled. The result implicates this
wait policy in part of the depth4 anomaly, without pricing a context switch,
separating idle from join waits, or supporting unconditional busy waiting.

### Private generated recursion frontier

A separate M1 stack-sampling screen compares generated leaf, native WF value
depth8/24 and Parlay-left depth8 in the same v2 ordinary image
`be35d93513de49523bc0e6dda855332c8eae5a6435a416dbca9ed951cbb663e0`.
Three alternating passes cover center-peak/depth-cap with plain and sampled
observers:48 processes,65,536 calls plus eight warmups each,3,146,112 checked
outputs. macOS `sample` requests two seconds at1ms intervals. It samples wall
stacks of every thread, including the floor parent waiting for computation,
and has no handshake with the batch interval. It is not on-CPU profiling.
The parent contributes about20% of the five-thread denominator. The24 reports
contain105,014 thread snapshots; collapsed-top tables omit126 snapshots whose
individual symbols have fewer than five observations. An omitted symbol is
not a measured zero.

Sampling raises paired median wall time by9.7-19.4%, and all24 sampled runs
are slower than their plain pairs. In the plain observer, native WF depth8
has generated-leaf-relative wall ratios0.666/0.659; native WF depth24 has
1.061/1.091, all three wall pairs losing. Generated join and TLS lookup symbols
are frequent in sampled stacks; depth8 native WF spends more snapshots in its
sequential subtree kernel. These observations motivate testing the recursive
frontier, without assigning a nanosecond cost or CPU percentage to a symbol.

The subsequent private LLVM control starts from the actual emitted leaf-host
module (SHA256
`c58f15993dcb0281065a070d67c949674035b81290c9f99d9b768c752433f712`).
It selects the sole self-recursive function containing a compute offer,
then creates internal function/callback layers at limits4/8/12/24. Both direct
recursive calls and the published callback enter the next layer. At the
frontier they enter the existing same-signature sequential clone. Arithmetic,
numerical depth/convergence tests, operands, result order,88-byte frame,
null fallback, join and release are preserved. The limit bounds offered
recursion, not numerical depth. Public shims and ordinary signatures are
unchanged; there is no added runtime word, TLS depth or hidden parameter.
At measurement time this was a private emitted-code experiment, preceding the
compiler control below; it did not qualify mutual recursion, external calls
or arbitrary effects.

Exact reversal checks all48 function/callback layer pairs. The stock regenerated
object and relinked ordinary/sanitized images match the qualified base exactly.
All remaining host/runtime/floor/native objects are shared inputs. The control
combines grain selection with static specialization, inlining and image layout.
Stock/depth8/depth24 Mach-O `__text` sizes are33,156/37,228/46,508 bytes;
these are whole-image instruction-section bytes, not stack or RSS observations.
Depth24 retains every original offer opportunity for these fixtures, whose
numerical depth is at most24, so its changes cannot be credited to fewer offers.

Before timing,170 processes check2,500 outputs across five variants with
ordinary/sanitizer images, widths1/4 and generated/native forms. Seventy full
checks retain the existing oracle;
100 generated sanitizer batches cover all ten inputs, including empty and
depth-zero. Generated LLVM remains ordinary inside the sanitizer image.
These checks do not establish exact transformed publication/refusal counts
or newly forced owner-slot exhaustion. Original full-depth event assertions
remain maintained and are not repurposed as frontier evidence.

The timing screen retains400 processes,1,638,400 timed calls and3,200 warmups:
five limits including stock, four heavy inputs, four forms and five alternating
whole-cell orders at width4. Generated sequential, native WF value depth8 and
Parlay-left depth8 remain controls in each image. Scalar strict FP/no SIMD,
FMA or LTO; MacBookPro18,3/Clang21; unfixed placement/frequency; no overlapping
native timing/builds. Limit identity comes from image paths, recorded commands
and summaries, not a field in the original v2 raw report.

| Input | Stock generated wall us | Frontier8 generated wall us | Frontier8/stock wall ratio | Frontier8/Parlay-left depth8 wall ratio | Faster than Parlay pairs /5 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak |15.750|10.099|0.657|1.009|2|
| Left peak |14.173|8.824|0.633|0.915|5|
| Right peak |12.910|8.205|0.658|0.996|3|
| Depth cap |26.379|16.613|0.640|0.945|4|

Wall values are medians of process batch means; ratios are medians of matched
pass ratios, not ratios of those displayed medians. Frontier8 beats stock in
all five wall and CPU pairs on every input. Rusage CPU ratios versus stock
are0.661/0.607/0.634/0.623. Versus same-image Parlay, CPU ratios are
0.999/0.846/1.006/0.893, with3/4/2/5 lower pairs: wall parity is not a uniform
CPU advantage. Frontier4 favors depth-cap (stock-relative wall0.604 and
Parlay-relative0.907, all five wins), but loses all five Parlay pairs on both
skewed inputs. Frontier12 loses all five Parlay pairs on every input.
Frontier24/stock wall ratios0.951/0.919/0.979/0.928 retain specialization/layout
effects despite unchanged offer opportunities; only depth-cap wins all five.
No fixed depth is selected as a default. This screen alone does not qualify a
general compiler transformation, task/exhaustion behavior, Linux replication
or held-out workloads. The subsequent compiler qualification follows below.

### Compiler-generated recursion frontier

The normal compiler now exposes the opt-in
[`--par --par-recursive-frontier N`](../../../compiler/README.md#parallel-and-completion-lowering)
control. The compiler implementation map owns eligibility and call-level
semantics. This panel builds `wf-frontier` and `wf-frontier-seq` with depths4/8
and scalar-leaf limit16, beside all preceding generated and native forms in
one ordinary image. The fixed depth is an experimental candidate, not a
default or a demonstrated best policy. `spawn_depth=4` or8 selects the matching
generated object in reports; other depths are refused because this image
contains only those two generated variants. No textual recursive-call rewriting is
performed by the harness: its existing shim only exposes generated functions.

`make check-quadrature` includes the new forms and full owner-slot exhaustion.
The independent explicit-stack oracle counts internal tree nodes above the
parallel frontier; the report reader checks the corresponding depth4/8 reference
counts. Every instrumented four-worker parallel run requires publication plus
slot refusal to equal that count, and every published task to join and finish exactly once. Under
full exhaustion it requires zero publications and the exact refusal count.
An added negative report substitutes the full-depth center count1643 for247
and must fail. Original full-depth and sequential-refusal checks stay intact.
Batch calibration includes both new forms using the existing regional clocks
and bitwise output checks; instrumented binaries are never timing references.

Initial M1 qualification of the compiler change on base `68a2be9c` passes
170 full/exhaustion processes with3,400 checked outputs and172 sustained-batch
qualification processes with1,892 outputs. The normal center-peak frontier
run publishes247 tasks; full owner-slot exhaustion reports0 publications and
247 refusals. This closes the private LLVM screen's missing task/exhaustion
qualification for this generated workload. The ordinary image SHA256 is
`f97d20da025a64542a6924dd6b918814b5202c73bbd779eeca904f80d03e2a58`;
compiler binary SHA256 is
`6ed1f21620d6c316e8f16ed3eacb7c801b2c90d1b74246dbe9d199419addfa61`.
The local build reuses the preceding pinned scalar scheduler dependency build;
all harness/generated/runtime objects are rebuilt. ASan/UBSan cover the C/C++
host, runtime, floor and header-based Parlay, while generated LLVM and the
oneTBB library remain ordinary, as before. Compiler cases additionally check
self/mutual recursion, successful and refused callbacks, scalar/destination
results, shared and exclusive borrows, and a suspending cyclic component that
must retain its old emission.

The maintained batch caller subsequently measures440 plain processes across
eleven forms, two worker requests, four inputs and five alternating whole-cell
orders:1,802,240 timed integrations plus3,520 warmups, all checked bitwise.
MacBookPro18,3/Clang21, scalar strict FP with SIMD/FMA/LTO off; no overlapping
native timing, builds or tests. Placement and frequency remain uncontrolled.
A sandbox metadata attempt stopped at `sysctl` before any measured cell;
the complete cohort ran outside that sandbox. `perf` is unavailable on macOS.

| Input | Generated leaf wall us | Compiler frontier8 wall us | Frontier/leaf wall | Frontier/Parlay-left8 wall | Faster than Parlay-left8 pairs /5 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak |15.834|9.924|0.648|0.946|3|
| Left peak |13.977|8.845|0.630|0.937|5|
| Right peak |12.886|8.059|0.639|0.983|4|
| Depth cap |25.669|16.732|0.652|0.963|5|

Wall values are medians of process batch means; ratios are medians of matched
pass ratios, not ratios of the displayed medians. Frontier/leaf wall ratios
give paired median reductions34.8-37.0%, with all five wins for every input.
Rusage CPU ratios are0.589/0.593/0.610/0.643, also all five wins. Relative to
Parlay-left8, CPU ratios are0.847/0.862/0.901/0.922 with5/5/4/5 lower pairs.
This reproduces the private screen's benefit through the normal compiler,
without establishing a uniform advantage over the strongest reference:
frontier/Parlay-left4 depth-cap wall is0.996 with only three lower pairs.
Native WF value8 depth-cap similarly remains mixed (wall0.983, three wins).
Single-worker frontier-sequential/leaf-sequential wall ratios are
0.997/0.993/1.009/1.001 with3/3/2/2 lower pairs; these controls have no uniform
speedup and retain the small right/depth-cap adverse medians.
With a worker request of4, those same sequential forms still start no WF
pool; their paired wall ratios are1.002/1.006/1.013/1.000, only two lower pairs
each. The depth-cap control retains an adverse maximum1.155. Neither control
is normalized away or treated as proof of code-layout neutrality.
Specialization/layout, fixed-grain selection and M1 placement remain material
limits. The Linux replication below tests another platform; held-out
application coverage remains open and no universal ceiling is established.

### Linux compiler-frontier replication

The [5bc311f4 Linux quadrature job](https://github.com/mbbill/Whitefoot/actions/runs/34261295877/job/102179613155)
qualifies the normal compiler frontier before the Rayon extension below.
It retains170 full and172 batch qualifiers, then880 measured processes
(440 plain/440 perf),3,611,520 checked outputs and1,760 software-event rows.
All63 manifest paths and14 harness source snapshots match the exact revision.
Artifact10070215810 has ZIP SHA256
`d3041a3db5faf92995bca07c11855febd7c98843b78e670a13b2dd0fd8cc49ef`;
the ordinary executable is
`8f57b1ea19a952cdcfc784ca25fdfbf72a3652d65ee01a96426e458e21fc31e8`.
Center-peak again has247 frontier publications, or0 publications/247 refusals
with all owner slots held. Original full-depth checks remain intact.

AMD EPYC7763 hosted VM, two cores/four SMT CPUs, mask0-3;
Clang18.1.3/x86-64-v3, strict scalar FP, no SIMD/FMA/LTO. `cpu.max` is
unavailable; full physical capacity/isolation is not qualified. These are
five paired process batch means per input, not latency tails. No M1 or prior
Linux cohort is pooled into the following ratios.

| Input | Plain frontier/leaf wall | Plain frontier/leaf CPU | Plain frontier/Parlay-left8 wall | Plain frontier/Parlay-left8 CPU |
| --- | ---: | ---: | ---: | ---: |
| Center peak |0.659|0.637|0.866|0.891|
| Left peak |0.683|0.646|0.828|0.873|
| Right peak |0.681|0.631|0.887|0.897|
| Depth cap |0.655|0.637|0.938|0.950|

Every table cell improves in all five pairs. Plain wall reductions against
generated leaf are31.7-34.5%. Perf-observer frontier/leaf wall ratios are
0.667/0.670/0.667/0.660 and frontier/Parlay-left8 ratios are
0.875/0.835/0.888/0.937, also five wins each; observers remain separate.
Native WF value8 is still a stronger near-parity comparison: plain right
wall1.006 has only two lower pairs, and depth-cap0.995 has three. Perf center
wall1.001 also has two lower pairs. Against Parlay-left4 on right skew,
frontier consumes more rusage CPU in all five pairs despite lower wall time:
plain1.247/perf1.219. Parlay-left8 has lower observed wall time than depth4 on
every input in this panel. There is no uniform CPU/strongest-reference win.

Plain W4 frontier-sequential/leaf-sequential wall ratios are
1.004/1.007/0.998/1.004 with2/2/3/2 lower pairs; the perf right control retains
an adverse maximum1.127. Neither sequential form starts a WF pool.
Across880 processes, process-clock/rusage median is0.999940 with range
0.990955-0.999996. Perf task-clock/process-clock ranges0.725724-1.015149;
Parlay-left4 medians remain0.833/0.737/0.904/0.990. Agreement between CPU APIs
may share kernel accounting and does not prove which counter is accurate.
All four software events report100% running; six requested hardware events
are unavailable, so IPC/cache/stall attribution remains absent. The retained
verbose probe qualifies the software event setup, not every timed region.

### Recursive Rayon reference

`rayon` and `rayon-left` add reciprocal `rayon::join` recursion at depths
0/2/4/8/24; `rust-seq` uses the same Rust scalar kernel without a pool.
The [pinned Rayon join interface](https://docs.rs/rayon/1.12.0/rayon/fn.join.html)
executes its first closure locally and makes its second available to steal.
Thus `rayon` offers the right branch, while `rayon-left` offers the left and
matches generated WF's direction. Both still sum left plus right. Below the
frontier they call a sequential specialization, matching the native C++
controls' grain rule. Rust preserves the explicit binary64 operation order;
the independent C stack oracle checks all numerical results and work counts.
No per-node C callback substitutes for native Rust recursion.

The existing locked Rayon1.12.0 crate owns this optional `quadrature` module;
its `quadrature-stats` feature adds subtree node/fork/migrated-branch counts.
Default records/event builds do not compile the module. Ordinary timing
has no counters or diagnostic callback calls. The one root C ABI call returns
the value; the instrumented root additionally reports counters through a C
callback. The generated Rust C symbol is resolved from that exact build's IR,
without unsafe Rust or export attributes. All branches join before reporting.
These are application fork/migration counts, not internal Rayon job counts
or OS context switches. Instrumented W4 qualification must observe migration;
an all-zero migration report is a maintained negative case.

The pool includes the caller as worker zero, with three helpers at width4.
It is process-lived, as in the earlier records control. Full and batch
qualification retain stdout and diagnostic stderr separately and validate the
actual exit status. Address-instrumented Linux x86_64 runs use the existing
exact pinned caller-worker lifecycle grammar with the quadrature initializer;
other instrumentation profiles require clean status/stderr. No leak detector
is disabled and no arbitrary allocation is tolerated. Old records grammar
replays remain unchanged. The observed Linux stack change and the subsequent
successful configured-matrix replay are described below.

Rust uses O3 with loop/SLP vectorization and LTO disabled; the Linux v3 panel
also requests Rust x86-64-v3. Rust/Rayon are not ASan/UBSan-instrumented even
in the diagnostic image; C/C++ host/runtime/floor and Parlay headers retain
their existing sanitizer coverage. Rust kernel, closures, result aggregates,
compiler and one root FFI boundary differ from C++ and generated WF, so this
is an end-to-end reference. Rust sequential results must be considered before
attributing a timing difference solely to scheduling.

The initial M1 implementation passed214 full qualifiers/4,280 outputs and
220 batch qualifiers/2,420 outputs, then measured560 plain processes with
2,298,240 checked outputs. Its ordinary image SHA256 is
`4671fd3f5a6e71309f69db32759761f4e22f25996b029501b75d0e7cde5974a3`;
all80 build-manifest paths and20 source snapshots, including nested Rust,
were independently verified. This cohort used an80-byte by-value Rust `Node`
argument, lowered to a memory pointer by Rust1.98.1/LLVM22.1.8. C++ used
scalar arguments with Clang21. Both disabled vectorization and LTO.

That initial reference fails kernel-quality qualification: Rust-sequential /
C++-sequential paired wall medians at requested width1 are
2.863/2.995/2.737/2.769 for center/left/right/cap; requested width4 gives
2.787/3.042/2.725/2.814. Every pair is slower, including rusage CPU. These
sequential forms start no pool. WF-frontier/Rayon8 W4 wall ratios
0.407/0.402/0.392/0.313 and reciprocal Rayon-left8 ratios
0.386/0.345/0.442/0.280 therefore do not establish a scheduling advantage.
No sequential normalization is used to hide the kernel difference. The
current Rust implementation passes the recursive scalar arguments directly;
the operation sequence, fork directions and oracle remain unchanged.

Within that same initial cohort, WF-frontier/Parlay-left8 wall ratios are
0.944/0.911/0.955/0.932, with5/5/4/5 lower pairs. Adverse controls remain:
native WF8 right has frontier wall1.029/CPU1.019 with only two lower pairs,
and Parlay-left4 cap has frontier wall1.016 with two lower pairs. Requested
width4 frontier-sequential/leaf-sequential cap wall1.026/CPU1.023 is slower
in all five pairs. These are process-batch means on an unpinned M1, not
latency tails or evidence of a universal library winner.

The scalar-argument rebuild repeats the same full/batch qualification and
560-process timing protocol, with ordinary SHA256
`602c2dc9909a404d89337a0eb7416cbcf2c678720678ce3aafccbd248b083012`.
All80 manifest paths,20 source snapshots and2,298,240 outputs pass independent
replay. Rust sequential recursion now has scalar LLVM parameters and a128-byte
M1 stack frame instead of192 bytes. The previous large sequential mismatch
is absent: Rust/C++ paired wall medians at width1 are
0.999/0.995/0.999/1.003; requested width4 gives0.986/0.995/0.994/1.017.
This is evidence for changing the argument representation, not a measured
cost assigned to an individual copy or stack instruction. Code layout and
instruction scheduling also change; no PMU attribution is available.

| Input | WF frontier8 wall us | Rayon8 wall us | Rayon-left8 wall us | Paired WF/Rayon8 wall | Paired WF/Rayon-left8 wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| Center peak |10.090|12.575|12.377|0.810|0.821|
| Left peak |8.890|10.483|14.945|0.848|0.580|
| Right peak |7.854|15.727|10.164|0.496|0.773|
| Depth cap |16.467|19.385|19.224|0.875|0.862|

Wall columns are medians of per-process means; ratio columns are medians of
same-pass ratios, not ratios of those medians. WF/Rayon8 wins all five wall
pairs per input; WF/Rayon-left8 wins are5/5/4/5, retaining the right adverse
maximum1.051. Rusage CPU ratios are0.767/0.818/0.574/0.810 against Rayon8
and0.768/0.625/0.744/0.826 against Rayon-left8, all five lower pairs.
The direction-dependent skew response remains visible after fixing the
sequential kernel. This panel fixes grain at8; broader Rayon grain/policy
tuning and held-out confirmation remain open.

The same cohort retains stronger adverse references: WF/Parlay-left4 cap
wall1.028 wins only one pair, with maximum1.112. WF/native-WF8 wall ratios
0.987/0.954/0.930/0.980 win only3/3/4/4 pairs. WF/Parlay-left8 wall ratios
0.924/0.933/0.947/0.932 win3/5/4/5 pairs. Sequential-clone controls remain
mixed, including requested-width4 right wall1.004 with two lower pairs.
Earlier cohorts are not pooled. Kernel timing parity removes the previous
large confound but does not isolate scheduler cost or qualify a globally
strongest reference; closure representation, recursion, pool entry and runtime
policy all remain in the measured end-to-end time.

### Rayon grain replication and Linux lifecycle boundary

The maintained sustained panel adds both Rayon directions at depth4 alongside
depth8, following the short-call grain screen. The same scalar ordinary image
`602c2dc9909a404d89337a0eb7416cbcf2c678720678ce3aafccbd248b083012`
then measures640 plain processes/2,626,560 checked outputs on M1. The new
batch driver passes252 qualifiers/2,772 outputs;214 full qualifiers/4,280
outputs from the unchanged build are retained and replayed. All80 manifest
paths pass. The20 build source snapshots match `dd3f3a83`; the result driver
has SHA256 `5d69313146a5a09b0cecf8a29a214e8b1d7a6fc793bec65d949e1323e664125b`
and records the additional depth4 cells. The retained memory reader precedes
the following Linux fix; that qualification-only reader is not on the plain
timed path. No source snapshot is rewritten to claim an all-current build.

| Input | Rayon4/Rayon8 wall | Rayon4/Rayon8 CPU | Rayon-left4/Rayon-left8 wall | Rayon-left4/Rayon-left8 CPU |
| --- | ---: | ---: | ---: | ---: |
| Center peak |1.122|0.981|1.149|0.988|
| Left peak |1.503|1.182|2.307|1.483|
| Right peak |2.447|1.520|1.458|1.170|
| Depth cap |0.925|0.908|0.847|0.871|

These are W4 same-pass ratio medians, including total process rusage CPU.
Depth4 improves cap in all five wall/CPU pairs for both directions, but loses
every pair on both skewed inputs. Center wall loses four pairs for each
direction despite mixed CPU results. At W1, depth4 improves every input in
all five wall/CPU pairs: wall ratios range0.885-0.970 for right-offer and
0.878-0.966 for left-offer. This supports a grain tradeoff, not a per-fork
cost estimate or a selected optimal policy.

WF-frontier/Rayon-left4 cap wall is0.994 with only three lower pairs and
maximum1.149; CPU0.952 has four. Against Rayon4 cap wall0.931 wins four
pairs, maximum1.131. Parlay-left4 cap still gives adverse WF wall1.014 with
two lower pairs, and native WF8 right wall1.016 has two. Rust/C++ sequential
W1 wall ratios remain0.998/1.000/0.984/1.005; requested-W4 ratios are
1.007/1.004/1.004/0.996. Sequential forms have no pool. M1 placement and
frequency remain uncontrolled; no prior cohort is pooled or normalized away.

The [dd3f3a83 Linux quadrature job](https://github.com/mbbill/Whitefoot/actions/runs/34266355640/job/102196609312)
fails qualification before timing at its first address-instrumented Rayon
report (W1/depth0). Artifact10072050405 retains the original stderr and
functional stdout. The latter passes the20-output oracle; stderr has exactly
the known384-byte worker and1,520-byte queue allocations, seven/five frames,
1,904 bytes total. This build keeps `ThreadPool::build` out of line and
inlines the futex call: initialization frames are build/force/initialize,
where the records image has force/futex/initialize. The old reader rejects
that identity; the corrected quadrature-only path accepts the exact observed
sequence and keeps the original records path. All14 retained records reports
still pass. Wrong identities, allocations, owners, statuses and extra reports
reject in replay; two maintained Linux address-profile negatives mutate the
actual captured pool identity and allocation size. This is no Linux timing
result and does not yet qualify unseen depths, widths or directions.

The subsequent [813c6d44 Linux compute job](https://github.com/mbbill/Whitefoot/actions/runs/34267682585/job/102201077815)
passes the full configured matrix and calibration. Artifact10072723575 has
ZIP SHA256 `37b467cd7ac67ab3dd871d8b218476d9ba2f345ac926f85b529c971d400c9403`;
ordinary image SHA256 is
`5e19d89e2fb0a386ae26e8f4534fe1a3904ee9d02dc9a9866f2057a6765e05be`.
All81 manifest paths and20 source snapshots match the exact revision. Its
214 full and252 batch qualifiers pass;52 actual Rayon stderr reports match
the exact1,904-byte/two-allocation grammar, including both directions, all
five full-report depths and both widths. Other404 separate stderr files are
empty. Required child statuses follow from successful strict caller execution,
not separately retained status rows. This closes the observed lifecycle
failure for this configured compute matrix.

The run retains1,280 process reports (640 plain/640 perf),5,253,120 checked
outputs and2,560 software-event rows. AMD EPYC7763 VM, two cores/four SMT
CPUs, mask0-3, Clang18.1.3/Rust1.98.0, x86-64-v3, scalar/noFMA/noLTO;
physical capacity and isolation remain unqualified.

| Input | Plain WF/Rayon4 wall | Plain WF/Rayon8 wall | Plain WF/Rayon-left4 wall | Plain WF/Rayon-left8 wall |
| --- | ---: | ---: | ---: | ---: |
| Center peak |0.814|0.784|0.803|0.790|
| Left peak |0.752|0.814|0.585|0.714|
| Right peak |0.567|0.671|0.726|0.768|
| Depth cap |0.967|0.896|0.968|0.908|

Every table cell wins all five W4 wall pairs; perf-observer wall comparisons
also win all five. CPU retains adverse pairs: plain WF/Rayon4 cap0.981 has
four lower pairs, and perf WF/Rayon-left4 cap0.971 has four. Native WF8 is
mixed, including plain left CPU1.004 with two lower pairs. Parlay-left4 right
uses less CPU than WF in every pair (WF ratios plain1.208/perf1.184), despite
worse wall time. Rust/C++ sequential plainW1 wall ratios
1.003/1.016/0.997/0.999 retain near-parity kernel timing. Process/rusage
median0.999941 does not resolve task-clock/process range0.728320-1.014730;
six hardware counters remain unavailable. No observer or earlier cohort is
pooled, and no strongest-reference or hardware-limit claim follows.

The separate [813c6d44 full gate](https://github.com/mbbill/Whitefoot/actions/runs/34267682702)
failed both research jobs with child status1 during the instrumented
four-worker Rayon checks. The successful preceding-report counts locate
right-offer depth4 on macOS and left-offer depth4 on Linux, but their retained
job logs contain only the wrapper status, not the underlying diagnostic.
Two hundred local instrumented depth4 processes do not reproduce it. The
wrapper now echoes rejected stderr while preserving rejection and exit status;
the existing unexpected-diagnostic negative also checks that forwarding.
The subsequent exact `a62b98f2` [full gate](https://github.com/mbbill/Whitefoot/actions/runs/34269691183)
passed all12 jobs, and its [compute workflow](https://github.com/mbbill/Whitefoot/actions/runs/34269691191)
passed all five jobs. The previous failure did not recur there. Its cause
remains unverified; forwarding is a diagnostic improvement, not an established
fix for the intermittent failure. No check is relaxed. The timing cohort above
remains bound to `813c6d44`, not silently replaced by the newer successful run.

The independently replayed `a62b98f2` [Linux compute artifact](https://github.com/mbbill/Whitefoot/actions/runs/34269691191/job/102207830095)
contains1,280 sustained reports/5,253,120 checked outputs and2,560 software
event rows. Its81 manifest paths and20 sources match the exact revision;
214 full and252 batch qualifiers pass, including52 pinned lifecycle reports
and404 other empty stderr files. The2,140 input headers also match the
independent work/span reconstruction. Artifact10073487495 has ZIP SHA256
`a7293c0d2159c4d50ecc6e33098d58790fac88439ee6aa834354d2dd229006b9`;
ordinary SHA256 is
`6e3ce9eed1a11b4fc091acfea5633e6c98c163a97debc179cce34346a2033fba`.

This host is an Intel Xeon Platinum8370C VM, two reported cores/four SMT
CPUs under mask0--3, unlike the preceding EPYC cohort. Clang18.1.3 and
Rust1.98.0 use scalar x86-64-v3 with no FMA/LTO. Frequency, physical capacity,
quota and isolation remain unqualified; no cross-cohort change is attributed
to the CPU or source revision. All peaked-input W4 WF/Rayon wall and CPU
comparisons win five pairs in both observers, but balanced cap does not:

| W4 form | Center peak wall us | Left peak wall us | Right peak wall us | Depth cap wall us |
| --- | ---: | ---: | ---: | ---: |
| Generated WF leaf |31.502|25.898|25.258|64.576|
| Generated WF frontier8 |19.527|16.735|15.327|37.681|
| Native WF value8 |20.028|17.319|16.036|38.529|
| Rayon4 |22.479|21.025|30.716|37.648|
| Rayon8 |23.796|18.669|21.690|41.119|
| Rayon-left4 |22.598|30.779|20.718|37.559|
| Rayon-left8 |23.994|21.283|18.822|40.920|
| Parlay-left4 |49.915|79.545|22.625|37.462|
| Parlay-left8 |21.635|18.340|18.441|39.826|
| oneTBB8 |37.524|31.051|28.484|54.600|

Wall values are medians of process batch means, in microseconds per call.
They are not individual-call latency percentiles. The following paired ratios
are not ratios of those independently displayed medians.

| W4 depth-cap comparison | Plain wall ratio [min,max]; lower pairs/5 | Plain CPU ratio; lower pairs/5 |
| --- | ---: | ---: |
| WF frontier8 / Rayon4 |0.999[0.993,1.011];3|1.000;3|
| WF frontier8 / Rayon-left4 |1.012[0.992,1.018];2|1.006;1|
| WF frontier8 / Parlay-left4 |1.006;2|1.040;0|

Ratios are medians of same-pass4096-call process means. Perf-observer cap
wall ratios against the two Rayon4 directions are1.002 and0.997, with only
two and three lower pairs. Parlay4 cap consumes less CPU than WF in all five
pairs under both observers. Native-WF8 right-peak also retains adverse CPU
pairs despite five lower wall pairs. WF frontier8 versus old generated leaf
plain wall ratios0.623/0.644/0.607/0.589 win all five per input; this does not
establish a strongest-reference victory. Rust/C++ sequential W1 ratios
0.959/1.001/1.000/1.000 retain kernel near-parity. Hardware counters remain
unavailable, and task-clock/process ratios range0.813--1.014.

The uniform-tree model and this coarse-reference CPU loss motivate adding
the compiler's existing depth4 option to the executable matrix, including its
sequential control and full owner-slot exhaustion. Depth8 and all native
references remain; the same four sustained inputs retain skewed losses when
coarsening. Both generated depths link into one ordinary image. The build
retains separate generated IR, ledger and assembly, checking four or eight
publication sites respectively. No compiler policy, language ABI or default
grain changes. Timing below must use that new image rather than treating
earlier native or private-LLVM depth4 results as generated-WF measurements.

### Generated depth-four M1 comparison

The expanded matrix on base `d6e19f96` uses ordinary image SHA256
`befd4a630425a5db0cece5cf0ea9390c010c13b6d13d112120508421f8b94686`.
The new depth4 and retained depth8 objects come from the same normal compiler
and WF source. Local qualification passes224 full processes/4,480 outputs,
including both generated depths under full owner-slot exhaustion, and284
batch processes/3,124 outputs. The sustained panel completes720 plain
processes/2,954,880 checked outputs across18 forms, two widths, four heavy
inputs and five alternating passes. Apple Clang21.0.0 and Rust1.98.1 retain
scalar/no-FMA/no-LTO settings; this M1 MacBookPro18,3 is unpinned, with no
frequency or PMU qualification. It is separate from the Intel/EPYC cohorts.

The following four-worker ratios are medians of matched process-pass ratios;
the wall column is the median of process batch means in microseconds.

| Input | Generated WF4 wall us | WF4/WF8 wall | WF4/Parlay-left4 wall | WF4/Rayon-left4 wall |
| --- | ---: | ---: | ---: | ---: |
| Center peak |10.734|1.073|0.654|0.678|
| Left peak |12.961|1.500|0.499|0.443|
| Right peak |10.806|1.355|0.945|0.743|
| Depth cap |15.448|0.944|0.971|0.916|

WF4 loses all five wall pairs to WF8 on all three peaked inputs, and wins
all five on cap. Its corresponding CPU ratios are1.018/1.553/1.301/0.951.
For cap it also wins all five wall pairs against Parlay-left4 and both
Rayon4 directions; CPU ratios are0.937/0.887/0.875 respectively. The cap
CPU comparison against Parlay4 has four lower pairs and one adverse pair
(maximum1.040), despite five lower wall pairs. Adverse
cases remain: WF4 uses more CPU than right-offer Rayon4 on left-peak
(median1.078), and more than Parlay-left4 on right-peak (1.120), despite
lower median wall time. The data supports a grain tradeoff, not a single
selected cutoff. Without a worker pool, W1 WF4/WF8 wall ratios are
1.021/0.997/1.018/1.002, with0/3/0/1 lower pairs; coarsening is not uniformly
cheaper even there. Sequential-clone4/8 W1 ratios are1.002/0.999/1.000/0.998
and retain mixed pairs. These controls are not normalized away or treated as
proof of identical layout. Private layer count, code layout and inlining change along
with scheduling opportunities, so these ratios do not price one task or
isolate scheduler instructions. Linux replication of generated depth4 remains
pending; no new runtime/default policy is promoted by this local screen.

### Recursive frontier work model

The independent explicit-stack oracle now emits `frontier_blocks`,
`frontier_nodes`, `largest_subtree` and `node_span` in each full-report input
header, outside all timing intervals. For the report's `spawn_depth`, a block
is a subtree rooted at the cut, or a leaf reached before it. Above-cut
internal nodes plus block work partition the tree exactly:
`forks + frontier_nodes = nodes` and `frontier_blocks = forks + 1`.
Below the cut, span is the subtree node count; above it, span is one plus
the larger child span. Each visited node has unit cost and result merges are
free in this model. Runtime work, unequal node costs, cache effects and
placement are omitted. `nodes / node_span` is model parallelism, not a
hardware speedup or wall-time guarantee. The configured cut models the
fixed-grain recursive controls; it does not reconstruct a runtime-dependent
refusal tree or the additional offers in original generated WF forms.

| Input | Nodes | Depth4 blocks | Depth4 largest subtree | Depth4 span | Depth8 blocks | Depth8 largest subtree | Depth8 span |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Center peak |3287|16|1051|1055|248|127|135|
| Left peak |2473|16|1479|1483|177|127|135|
| Right peak |2473|16|1479|1483|177|127|135|
| Depth cap |8191|16|511|515|256|31|39|

At depth4, skewed inputs have model parallelism1.668, below four workers;
depth8 raises it to18.319. Balanced cap already has15.905 at depth4, so the
finer cut cannot improve its four-worker work/span lower bound in this model,
while increasing forks from15 to255. This supports the measured coarsening
tradeoff. Center depth4 has model parallelism3.116. Reflection and reversing
fork direction preserve these shape metrics; they cannot explain the observed
direction-dependent timing. Runtime execution order, placement and frame costs
remain candidates for that residual, not conclusions of this model.

The diagnostic rebuild on base `813c6d44` has ordinary SHA256
`156a90c76b0c291e01f50c258d4500adebf245fd22b281b3df10847a310b7fa0`.
Its214 full qualifiers/4,280 outputs and252 batch qualifiers/2,772 outputs
pass. A separate recursive binary64 reconstruction and cut/span pass match
all50 input/cut combinations against2,140 input headers, including values,
partitions and spans. The reader enforces partition conservation, zero/full
cut boundaries and the uniform cap closed form; maintained negative reports
reject a broken partition and an incorrect uniform span. Intermediate skew
values are additionally established by the independent reconstruction, not
solely by the reader's generic inequalities. An initial numeric-string
comparison failure in that reader is retained; explicit numeric conversion
after integer validation fixes it. Timing cohorts above use their original
images and readers; no performance result is assigned to this rebuilt image.

### Rayon worker work attribution

The separate `quadrature-stats` build attributes visited computation nodes to
Rayon worker indices0--3, with the benchmark caller at0. It credits each
above-cut node to its executing worker and credits an entire sequential
subtree once at its boundary. Serial subtrees contain no joins and remain on
one worker. Four128-byte-aligned atomic counters use owner-only relaxed
load/store additions; the single benchmark caller resets them before a root
and reads them after all joins. There is no per-node callback, shared atomic
increment or new task payload. The ordinary timing build excludes these
counters and accesses. Only the private native benchmark observer signature
changes; WF lowering, runtime interfaces and public language ABI do not.

Full diagnostic reports attach a `# worker_nodes` row to every Rust/Rayon
call. The reader requires exact call identity, all four integer counts, sum
equal to the independent oracle's node count, zero work on inactive workers,
and agreement between helper participation and application branch migration.
Single-worker and zero-cut forms credit only worker0. A maintained Rust
broadcast test credits four distinct amounts on four workers, verifies their
index identities and reset, independently of the recursive aggregate count.
Maintained negative reports reject incorrect totals, inactive-worker work,
missing rows and wrong call identities. Per-call migration/work consistency
checks remain; the schedule-independent capability check is described below.

The `05649749` [macOS research job](https://github.com/mbbill/Whitefoot/actions/runs/34274301809/job/102223392979)
exposed `Rayon branch migration witness`: its short calls completed without
any migrated branch. Requiring positive migration in every small benchmark
process was incorrect. [Rayon join](https://docs.rs/rayon/1.12.0/rayon/fn.join.html)
permits local execution when the offered branch is not stolen. The earlier
failures without forwarded stderr remain unclassified; this diagnosis applies
to the failure whose actual diagnostic is retained.

Positive migration is now tested through the kernel's shared `join_results`
helper in an otherwise idle two-worker pool. The first closure waits at a
two-party barrier, so the offered closure must run on the other worker before
the pair can finish. Both publication directions require exactly one migrated
branch and the exact combined result/node/fork counts. Blocking is confined
to this controlled test, not benchmark computation. The separate worker-label
broadcast/reset test remains. Normal reports allow an all-local schedule;
they still require exact results, work/fork counts, worker conservation and
agreement between helper work and migration. A maintained all-local report
checks acceptance, and a report with helper work but zero migration must
reject for that specific inconsistency. This replaces a probabilistic
actualization assertion with a controlled path check; no numerical, memory,
resource-exhaustion or work-conservation check is removed. The extracted
helper may affect code layout, so earlier timing results stay bound to their
original images rather than being assigned to this repair.

These are computation-node counts, not equally expensive instructions, task
durations, upstream internal jobs or OS context switches. The added worker
lookup and counters can change diagnostic scheduling; these reports do not
measure the uninstrumented run's distribution or justify subtracting a fixed
observer overhead. Ordinary sustained timing remains a separate measurement.
This attribution covers the Rust/Rayon controls; generated WF and other native
runtimes do not yet emit equivalent per-worker computation-node counts.

After `check-quadrature`, run `make quadrature-worker-profile OUT="$OUT"
RESULTS="$OUT/quadrature/worker-profile"` from this experiment directory.
The compute CI runs it under the same selected CPU mask before ordinary
calibration, retaining20 diagnostic processes and1,800 checked calls/worker
rows across both directions, depths4/8 and five alternating passes. The first
call for each input is separate from its eight subsequent calls. Per-call
printing and clock/resource observations introduce gaps, so even subsequent
calls are not the uninterrupted sustained-batch protocol. The profile retains
the exact image hash, reader/driver copies, host metadata, raw stdout/stderr
and a manifest. Linux uses the same strict lifecycle diagnostic validator as
qualification, without suppressing unexpected status or stderr.

The maintained target's first M1 profile uses diagnostic image SHA256
`60c83b733d2097c84b12c380197c5578ea2564a0eaec4ade0de9939ee6f6cc17`,
Apple Clang21.0.0 and Rust1.98.1, with scalar flags and no CPU affinity or
frequency control. All20 processes/1,800 calls pass, with3,091,320 credited
computation nodes. Independent replay checks the46 profile manifest entries,
73 build entries, the separate image binding and20 source snapshots. The rebuilt image also passes214
full qualifications/4,280 outputs,252 batch qualifications/2,772 outputs and
the exact one-test worker-index/reset check. Ordinary Rust IR excludes the
counter bank; the diagnostic IR retains the128-byte-aligned bank.

For each call, largest share is `max(worker_nodes) / nodes`. Each process
averages its eight subsequent calls for an input; the following cells are
median[min,max] across the five process means. The200 first-input calls are
retained separately, and no earlier profile or timing cohort is pooled.

| Form | Left-peak largest share | Right-peak largest share |
| --- | ---: | ---: |
| Rayon depth4 |0.628[0.623,0.653]|0.700[0.661,0.866]|
| Rayon-left depth4 |0.732[0.716,0.828]|0.628[0.600,0.675]|
| Rayon depth8 |0.540[0.469,0.619]|0.374[0.288,0.399]|
| Rayon-left depth8 |0.432[0.381,0.474]|0.503[0.362,0.570]|

The coarse skew block alone requires at least1479/2473=0.598 of all nodes
on one worker. Finer splitting removes that specific limit, but the observed
depth8 distribution remains direction-dependent despite equal shape metrics.
This establishes imbalance in the instrumented execution, not its share of
ordinary elapsed time. Task handoff delay and worker startup/wakeup remain
unmeasured explanations to separate from local recursive execution cost;
balanced work alone does not imply low scheduling overhead or prove a causal
explanation for the earlier sustained timing differences.

## Scalar scheduler comparison

This panel investigates scheduling with six forms: the recovered WF runtime,
a persistent static busy-spin pool, oneTBB, Parlay, and Rayon join/parallel
iterator. All six link
the **same separately compiled** `records_state` computation and
`records_scheduler_chunk` callback objects. Every callback processes the same
fixed range of records and writes caller-provided output. The timed path does
not select the ASCII-word candidate or any SIMD library. C and C++ builds use
`-O3 -DNDEBUG -fno-vectorize -fno-slp-vectorize -fno-lto`; oneTBB's library build
also disables automatic vectorization and IPO. Rayon 1.12.0 and its locked
dependency crates use Rust release O3 with loop/SLP vectorization and LTO
disabled. Its precompiled Rust standard library is not rebuilt; the shared
computation and callback are still the identical scalar C objects.
The retained assembly permits
inspection of the actual compute/callback functions. These controls concern
compiler-generated code in this experiment and its native library, not the
implementation of operating-system routines.
Identical object bytes do not guarantee identical linked instruction placement.
The first complete Linux panel below has a substantial width-one difference
that prevents attributing its full parallel timing gap to scheduling.

`records_runtime.c` drives the real `runtime.c` acquisition/publication/join/
release protocol with a 32-byte C frame. It publishes one half of a range, runs
the other half on the current stack, then joins and releases. Acquisition
failure executes that range locally. It bypasses the compiler's static cost
estimate and generated 112-byte record frame. Therefore a row named
`wf-runtime` measures the runtime through this C adapter, **not a compiled WF
program**. There is no new language ABI or lowering path. The end-to-end WF
program above remains necessary to measure code generation and output ownership.

| Backend | Selected form and charged behavior |
|---|---|
| `wf-runtime` | Recovered pool, binary range split, current-stack join/help/steal; process-lifetime helpers. |
| `static-spin` | Width minus one persistent pthread helpers; contiguous partition of chunk indices, release/acquire epoch and padded completion cells. Every dispatch signals all helpers, including empty partitions. Helpers poll while idle. |
| `oneTBB-v2023.1.0-auto-grain1` | oneTBB `3046c8b0c29df995980003ea24f4d78c80ec0c8d`; persistent `global_control` and caller-reserving `task_arena`, `parallel_for` with `auto_partitioner` and range grain one. Library workers have process lifetime. |
| `parlay-native-grain1` | Parlay `51017699dcc421f80479cdb238d3092233ad0d26`; native private pool with caller worker zero, `parallel_for` grain one, default elastic policy and 10,000-microsecond steal timeout; explicit shutdown joins helpers. |
| `rayon-1.12.0-join` | Locked Rayon 1.12.0 / rayon-core 1.13.0; binary recursive `join` down to one common callback per leaf, caller worker zero plus width-minus-one helpers; process-lifetime pool. |
| `rayon-1.12.0-par-iter` | Same Rayon pool and callback; indexed parallel iterator over the chunk indices with its default adaptive splitting. |

Widths one, two and four include the caller. Each process has one immutable
width and one external caller. The shared workload is flat and its callbacks
perform finite independent CPU work; the static adapter cannot nest. TBB,
Parlay and Rayon iterators may group chunk indices internally, and split orders
differ: equal callback work does not imply an identical internal task graph.
These are initial qualified API choices, not proof of each library's fastest
partitioner or a complete reference frontier. OpenCilk still needs an executable
compute control. Go's native runtime/end-to-end comparison remains separate
future work; per-chunk cgo costs must not be attributed solely to scheduling.

Rayon's `use_current_thread` registers the owner as worker zero. Calls use the
inside-pool join/help path directly; there is no outside-pool `install` dispatch
or extra nonparticipating owner counted as a worker. The current-thread
registry cannot be detached by this API, so shutdown reports process lifetime.
The research Rust crate forbids unsafe code. A C adapter binds its ordinary
public `extern "C"` function using the exact build's LLVM symbol, rejecting a
missing or nonunique export. Emitting both rlib and staticlib keeps that symbol
visible without unsafe export attributes. The callback has the original C
pointer/index prototype; Rust transports the opaque address without
dereferencing it, and joined completion bounds the caller-owned storage lifetime.
This build-local binding is not a WF module ABI. The lockfile, compiler identity,
scalar flags, emitted IR, binding and static archive remain in the artifact.

Input generation, independent expected results, output allocation/initialization
and full-result checks are outside the timed call. Every invocation has its own
retained output vector, with the same storage layout in every cadence. The
host admits at most 8,388,608 retained result elements (64 MiB). `call_ns` starts immediately
before scheduler dispatch and ends after all callbacks join. The first call
charges any lazy initialization it triggers; warm calls reuse existing state.
The WF adapter need not start helpers for an empty or one-chunk range. No warmup
precedes the first call. Per-call CPU and context-switch observations enclose the clock
reads too; their resolution and observation overhead matter for tiny work.
RSS is lifetime peak at the observation point. Explicit shutdown is reported
separately; zero means process lifetime, not a free or fully measured teardown.

The final command argument selects `checked`, `dense`, `sleep-100us` or
`sleep-1ms`. Checked calls retain the full input/output comparison between
invocations. Dense calls defer those checks until after the entire batch;
sleep forms also defer them and request the named interval before each call
after the first. All modes then verify **every** retained result, not just the
last invocation. `gap_ns` records the actual end-to-next-start interval,
including observation overhead, checks where selected, sleep and scheduling
delay. It is zero on the first call; a sleep request is not an exact achieved
interval or a performance pass threshold.

The separate batch resource interval encloses the first and all warm calls,
sampling, explicit sleeps, and per-call checks only in `checked` mode. It
ends before final deferred verification, the capacity probe, shutdown and
printing. Thus it charges workers' idle polling during those intervals.
Batch CPU observations also enclose the batch clock reads. The parser checks
that batch wall time encloses all calls plus recorded gaps, and that batch
CPU/switch counts enclose the nested per-call observations. Compare call
latency and batch CPU together: low wakeup latency can consume substantially
more CPU between requests. These fixed finite bursts do not establish energy
efficiency, a production request distribution or an optimal idle policy.
The earlier `ecce5a2d` panel reused and reset one output buffer. Its dated
measurements are not pooled with these rows: the new per-call storage changes
buffer/cache layout even for `checked`, while the within-panel cadence
contrasts keep that layout fixed.

Every process additionally requires a finite-work participation witness with
exactly the requested distinct threads, including the caller, and simultaneous
callback activity at that width. Benchmark witnesses run **after** timing and
are reported separately; they do not prove full utilization on every measured
call. Up to three explicit witness waves use 100,000, 1,000,000 and 10,000,000
dependent arithmetic iterations per callback, retaining 32 callbacks per
requested worker. Longer finite work gives delayed workers an opportunity to
overlap; the same full-width predicate applies in every wave. This addresses
the observed static-pool case where all four threads participated but three
identical short waves never overlapped four callbacks. It neither changes a
measured sample nor retries one. A blocking
cross-index barrier was rejected as a capacity probe: schedulers are allowed
to execute independent indices sequentially, and Parlay's cold elastic startup
can delay helper participation. No measured callback waits for another index.
If all three waves fail, stderr retains each wave's iteration count, distinct-thread count,
peak callback activity and caller participation. Peak activity includes
preempted callbacks; the witness does not establish physical CPU simultaneity.

`check-scheduler` runs the complete 4,595,603-input leaf oracle on the exact
scalar timing object. Each backend/width qualifier then checks 780 batches /
270,660 output positions, exactly-once callback indices, joined callback tails,
and immutable inputs across five shapes, uneven sizes and four grains.
Sanitized copies cover the C/C++ host, scalar work, adapters, recovered/static
runtimes and Parlay headers; the linked oneTBB shared library and Rayon static
archive remain ordinary scalar Release builds. Twenty-four sanitized cadence smokes additionally
exercise the timed-host buffer lifecycle across all backends and cadences.
These checks do not substitute for upstream
library suites. The driver validates all raw row identities and retains
source/object/library hashes, flags, qualification logs and host topology.
Child or row-contract failure prints the raw record and failing path before
exiting, so a failed gate preserves the evidence without rerunning the sample.
Completed reports are flushed before the strong floor joins the entry thread,
including when a later process-exit sanitizer check fails. The sanitizer target
collects every backend/width/cadence report before failing if any case failed;
it does not retry. `records-scheduler-memory.sh` validates the entire functional
prefix, using the timing driver's shared `records-scheduler-record.awk` for
cadence rows. Clean cases require exit zero and no additional output.

Linux x86_64 ASan checks retain LeakSanitizer with an explicit exit code of 23.
For the pinned Rayon caller-worker API only, the recognized lifecycle report
contains one 384-byte direct worker allocation and optionally one 1,520-byte
indirect queue block, with the exact semantic allocation stacks checked by
`records-scheduler-memory.awk`. All fourteen Rayon reports in the
[`08e58d63` scheduler run](https://github.com/mbbill/Whitefoot/actions/runs/34184353021)
had both allocations, after successful functional qualification. The checker
allows relocation addresses, source line numbers and compiler hashes to vary;
changed per-object sizes or counts, stack identities, incomplete output, any
additional diagnostic, and any other exit status fail. This documents retained
process-lifetime storage, not a memory-clean teardown. No leak suppression is
installed. Other platforms and non-address sanitizer builds require clean
exit-zero reports; the explicit replay mode only tests archived Linux reports.

The [dc383eef Linux gate](https://github.com/mbbill/Whitefoot/actions/runs/34385490336/job/102580579719)
instead reports just the same known worker, totaling 384 bytes in one allocation.
The old classifier rejects that report because it requires the queue too.
Rayon's pinned `use_current_thread()` deliberately retains its caller worker;
its FIFO allocates eagerly. The missing queue report is not evidence of lazy
allocation or successful teardown. [LSan's reachability-based operation](https://github.com/google/sanitizers/wiki/AddressSanitizerLeakSanitizerDesignDocument#operation)
reports unreachable blocks, not a complete inventory of retained objects.
The corrected invariant is that every reported allocation has its established
identity, with the summary matching exactly the parsed allocations. The reason
this run does not report the queue remains unknown. A worker must still appear;
unknown allocations, duplicate objects, wrong totals and incomplete reports
still fail. `check-sanitizer-reports`, shared by the scheduler and quadrature
checks, replays both shapes and their malformed variants on every host; it also
requires the normal classifier to reject an injected 257-byte allocation with
either shape. Actual Linux sanitizer execution and its status checks remain.

A separate Linux x86_64 ASan executable injects a 257-byte C allocation after
full qualification. The gate first requires the actual additional allocation
report and then requires the normal checker to reject it. A missing injection,
earlier functional failure or unrelated sanitizer failure cannot satisfy that
negative check. The
[`d6fcc9a5` Linux run](https://github.com/mbbill/Whitefoot/actions/runs/34186079602)
qualified all 42 normal reports, including 14 actual Rayon exit-23 reports.
The additional allocation produced 2,161 bytes in three allocations and the
normal checker rejected it specifically as an extra allocation. That run later
failed the static width-four capacity witness, so it has no accepted calibration.
A failed qualification does not produce an accepted scheduler calibration.

Fetch dependencies explicitly before the offline build, as with Cargo's
recorded dependencies:

```sh
make -C research/experiments/compute-runtime scheduler-fetch OUT=/tmp/wf-scheduler
make -C research/experiments/compute-runtime check-scheduler OUT=/tmp/wf-scheduler
make -C research/experiments/compute-runtime scheduler-calibrate OUT=/tmp/wf-scheduler BENCH_ARCH=-march=native
```

The normal experiment `check` includes these qualifiers. CI fetches the pinned
sources before the research gate and runs calibration in its own job, with
one common recorded CPU mask for all controls. The calibration varies record
count, length, valid/early-invalid/late-invalid/skewed work, and record grain
at the same widths. All thirty original input/grain cells run checked and dense;
three discriminating cells additionally run both sleep intervals, preserving
their input seeds. Five passes over 66 cadence cells and eighteen backend/width
configurations yield 5,940 processes / 58,860 calls. The gate runs the original
three smoke inputs at all four cadences: 216 processes / 648 calls. Treat these
as same-host screens; worker affinity within the mask, sustained idle costs
and held-out confirmation require further work. All five Linux compute jobs select
the explicit C/C++ target `-march=x86-64-v3`; the Rust scheduler library and
oneTBB library use their default host targets. The explicit target avoids
Clang 18's invalid AVX10 combination inferred by `-march=native` on one CI
host, including the records qualification failure in run34239483038 before
calibration began. The remaining FIR/records native targets now use the same
explicit target; this changes target selection, not warning severity or the
existing per-panel SIMD controls. New measurements
are not pooled with the earlier native-target cohort.

### Shared executable layout control

`scheduler-layout` links namespaced copies of all six adapters and two WF
callback-grouping controls into one image.
Its first argument selects exactly one backend before floor entry; each
process retains that selection and width. The leaf, callback and common C
objects are those qualified by the standalone panel. Their addresses within
the image are identical across backend selections, including alignment under
page-aligned ASLR. Symbol checks require one strong leaf and callback definition;
the driver retains the image hash and symbols and rejects an image changed
during calibration. The added `records_selector.c` belongs to this diagnostic
and is retired when the shared-image control is superseded.

All modes pay one common indirect dispatch and load the same linked libraries.
This controls leaf/callback placement within the shared panel, but changes the
executable and initialization environment from the standalone forms. It is not
an isolated alignment-only intervention. Comparisons with the preceding
standalone run also retain run-order effects; neither result should be silently
pooled with the other.

`check-scheduler-layout` retains the complete `check-scheduler` prerequisite,
then runs all twenty-four shared-image backend/width qualifiers and 96 dense smoke
processes / 288 calls. An additional twenty-four ASan/UBSan qualifiers instrument
the new selector and shared host, linking the ordinary qualified common objects
and adapters. The broader standalone sanitizer coverage remains necessary;
six extra grouped-WF qualifiers instrument the adapter, host, common computation
and runtime at widths one, two and four.
Hardlinked aliases preserve the unchanged strict Linux Rayon report grammar;
they contain the same sanitizer image. Missing and invalid backend arguments
must fail with the expected diagnostic before execution.

The canonical experiment `check` includes this target. CI runs the standalone
calibration followed by the shared-image panel under the same CPU mask. The
`layout` driver mode uses four dense input cells: valid Unicode and early-invalid
records at 256 / maximum 65,536 bytes and 4,097 / maximum 128 bytes, grain 16.
Seeds match their standalone counterparts. Five passes over all twenty-four
configurations yield 480 processes / 5,040 calls. Raw grammar, full-output checks
and post-timing capacity requirements are unchanged. Build and run it with:

```sh
make -C research/experiments/compute-runtime scheduler-layout-calibrate OUT=/tmp/wf-scheduler
```

Dependencies must first be fetched as shown above. The Linux qualification
and shared-image results below cover the `d6bf6c08` image; they do not turn the
earlier standalone result into a scheduler-only comparison.

### Callback grouping controls

The shared-image selectors `wf-group4` and `wf-group16` stop binary splitting
when a range contains at most four or sixteen of the original callbacks. They
execute every callback in that range, including the uneven tail, on the current
stack. Callback record grain remains sixteen in the layout panel. Acquisition,
publication, join, release, exhaustion fallback and the 32-byte adapter frame
remain the same. The ordinary `wf` selector retains its original conditional
single-callback terminal; neither the compiler nor the default runtime policy
changes. Distinct backend names retain each grouping choice in raw summaries.

These controls test a decomposition tradeoff, not an improved WF compiler cost
model. At 256 records and callback grain sixteen there are only sixteen
callbacks: group16 executes the entire range on the caller without acquiring
or publishing a task. The WF pool starts only in the subsequent, untimed
capacity probe. Requested width four and successful post-timing capacity do
not establish four participating workers during the measurement.

A preliminary local M1 screen against `52d2435d` motivated the Linux control.
One scratch image contained the original adapter, a separate loop-form cutoff1
control, cutoffs4/16 and all five native references. All used the same scalar
leaf/callback objects with loop/SLP vectorization and LTO disabled. Four inputs, nine selectors,
seven rotating/reversed process passes and 32 warm calls yielded 252 processes,
8,316 calls and 18,099,774 checked outputs. The seed was 828219, requested width
four and cadence dense. Medians of process warm means follow in microseconds:

| Input (records / maximum bytes per record) | Original WF C adapter | Group4 | Group16 |
|---|---:|---:|---:|
| Early invalid 256 / 65,536 | 2.651 | 1.345 | 0.837 |
| Early invalid 4,097 / 128 | 7.379 | 5.999 | 5.704 |
| Unicode 4,097 / 128 | 69.833 | 69.256 | 68.682 |
| Unicode 256 / 65,536 | 1,956.217 | 1,967.574 | 6,684.581 |

Group16 was faster in all seven paired early-invalid 256-record passes and
slower in all seven long-Unicode passes. The paired median time ratios were
0.329 and 3.394 respectively. Pool initialization and work exposure differ;
this does not isolate per-task scheduler cost. Other cells have overlapping
ranges, and one original long-Unicode process mean reached 4.644 milliseconds.
No sample was dropped. The scratch loop-form cutoff1 was not optimized-IR
identical to the original conditional terminal and is not substituted for it.
Local scratch raw files are not a retained CI artifact, and per-worker placement
and thermal state were not controlled. Treat these numbers as exploratory;
the maintained Linux matrix provides the reproducible follow-up, not evidence
that these M1 rankings transfer to another machine.

A separate 84-process M1 screen tested removing the redundant release-time
`FREE` store and caching a slot index in existing padding. It checked 2,772
calls / 6,033,258 outputs, with identical linked leaf/callback bytes and
addresses. Neither candidate established a repeatable gain across these four
inputs; both remain unpromoted. Reduced release instruction counts alone did
not justify a runtime change.

### Common callback diagnostics

`scheduler-trace` is a separate observer image containing the same eight
selectors, ordinary scalar computation/callback objects and runtime adapters.
The original timing image is built without the trace code. This first observer
measures common callback execution, not internal runtime jobs, steals, actual
bytes examined, hardware instructions or operating-system switch events.

Each process selects one of three levels before executing any work:

- `plain` calls the ordinary common callback directly.
- `identity` additionally records native thread identity and claims one
  preallocated cell per callback with a relaxed atomic operation. This detects
  duplicate execution before non-atomic event writes. There is no global event
  counter; cells are aligned to 128 bytes to separate adjacent writers.
- `timeline` adds two `CLOCK_MONOTONIC_RAW` timestamps around the common
  callback. Identity lookup, the claim and event stores lie outside that
  interval but inside the observed scheduler dispatch.

All levels allocate the same event/output footprint before the batch and retain
separate storage for every invocation. The first native-ID lookup on each
helper is charged to dispatch; later lookups use thread-local storage. Callbacks
do not allocate or print events. After all measured calls join, the host checks
every output, input preservation and complete event inventory, then performs
the existing capacity probe and reports data. Lazy pool startup remains charged
to the first call that triggers it. Empty inputs need no observed participants.

Raw records contain call boundaries and CPU/switch/peak-RSS resources, followed
by per-callback index, native TID, begin/end, record range and offered input
bytes. Offered bytes are not bytes actually examined by early-exit validation.
Callback intervals include preemption: their union, overlap and per-thread sums
describe observed wall time, not CPU utilization. Time outside that union may
include runtime work, waiting, preemption or unobserved callback bookkeeping;
it is not directly scheduler CPU cost. Zero-duration clock observations are
allowed and do not prove zero work. Nested runtime-job durations would need
separate accounting to avoid double counting.

The strict AWK collector validates every row, identity, range, byte total,
call/batch resource enclosure and final count before publishing a summary.
Decimal-string arithmetic preserves large timestamps and thread IDs exactly.
Native thread identity is implemented for Linux and macOS only. The canonical
`check-scheduler-trace` retains all prior scheduler checks and adds 72 observer
sanitizer cases (eight selectors, three widths, three levels), 288 ordinary
smoke processes / 864 calls and missing/duplicate/trailing-report negatives.
The new sanitizer image instruments the host, common C computation and runtime;
adapters and external libraries retain their separately qualified ordinary
objects. Existing exact Linux Rayon lifecycle parsing remains required.

The observer calibration deliberately covers only early-invalid and long
Unicode inputs, each 256 records / maximum 65,536 bytes, grain16, seed828219.
Eight selectors, three observation levels, five rotating/reversed passes and
nine calls per process yield 240 processes / 2,160 calls / 23,040 callback
events. Width is four including the caller. Full raw files, source/object
hashes, flags and host metadata are retained in `trace-calibration`; the CI
job runs this after the two existing timing panels on the same CPU mask:

```sh
make -C research/experiments/compute-runtime check-scheduler-trace OUT=/tmp/wf-scheduler
OUT=/tmp/wf-scheduler RESULTS=/tmp/wf-scheduler/trace-calibration \
  sh research/experiments/compute-runtime/records-scheduler-trace.sh calibrate
```

A local M1 implementation screen checked all 240 processes, 552,960 outputs
and 23,040 events independently, including per-chunk source-derived input
bytes and sorted, nonoverlapping intervals for each native thread. Every one
of the forty warm long-Unicode timeline calls per selector observed four TIDs
and overlapping intervals from four callbacks, except group16, which observed
only the caller in all forty. This does not establish simultaneous CPU execution.
For early-invalid timeline calls, Rayon join was caller-only in 32/40 calls and
oneTBB in 29/40; original WF observed four TIDs in 32 calls and two in eight.
These are observations under instrumentation, not recovered plain-run histories.

The short caller-only group16 control exposes observation cost: medians of five
process warm means were 0.818 / 0.854 / 1.114 microseconds for plain / identity /
timeline. All five paired timeline processes were slower than plain, with
paired median time ratio 1.354. Long-Unicode WF plain/timeline medians were
1.985 / 2.006 milliseconds, but their paired median ratio was 1.0004 with range
0.934--1.058. Other short cells vary substantially and can reverse ordering
between levels. Do not subtract a constant timer cost or promote observer
timings into ordinary performance rankings. Local raw files are exploratory,
not a retained CI artifact; the collection preceded the final parser's added
329-byte sanitizer-fixture binding. The measured observer executable and
computation are unchanged by that parser refinement.

The [`0bdcb82f` Linux run](https://github.com/mbbill/Whitefoot/actions/runs/34194826873)
retains the matching trace panel in artifact `compute-scheduler-linux`, ID
`10043653064`, ZIP SHA-256
`b1696e650e56aa443beceb61537d8a27c1e61bde500e97485730181599b13dce`.
Independent reduction checked all 240 processes, 2,160 calls, 552,960 outputs
and 23,040 events against the frozen collector, reconstructed per-chunk bytes,
and matched all 531 recorded source/object/dependency hashes. All 72 trace
sanitizer cases passed their documented contract: 54 clean exits and 18 exact
Rayon caller-registration retention reports. This audit covers the new trace
panel and its qualification, not a fresh audit of the older ordinary panels.

On this EPYC 7763 VM, width four uses a CPU mask of 0--3 exposing two physical
cores and four SMT logical CPUs. Long-Unicode plain observer-image medians of
five process warm means were WF 3.930 ms, Rayon join 3.927 ms and static 3.812 ms;
WF and Rayon ranges overlap. These are observer-image controls, not the
original timing image or a compiled-WF result. Under timeline instrumentation,
every long-Unicode warm call used four callback TIDs and reached four overlapping
intervals for all controls except group16, which remained caller-only. In short
early-invalid calls, WF used three TIDs in all forty calls; Rayon join was
caller-only in 8/40 and oneTBB in 21/40. The corresponding M1 participation
differs, so plain-run histories and cross-platform speedups cannot be inferred.

The static early-invalid timeline panel narrows one loss: the median of five
process ratios of total warm uncovered wall to total warm dispatch wall is
99.9182%, with range 99.8809--99.9666%. Most elapsed time lies outside the union
of common callback intervals. This prioritizes dispatch/waiting/OS scheduling
over optimizing the computation, but does not identify a syscall, spinning,
preemption or quota as the cause. Ratios of sums weight long calls by duration;
an unweighted mean of per-call occupancy ratios answers a different question
and must not be labeled the fraction of total dispatch time.

Linux short group16 plain/timeline process medians were 1.033/2.868 microseconds;
the median paired timeline/plain ratio was 2.789, compared with 1.354 on M1.
Instrumentation can change rankings, especially for short work. Internal
publication/pop/steal/completion counts need separate conservation checks,
including owner-inline join execution; they are not OS context switches.
Aligned OS and hardware counters remain unqualified: this manifest records
`perf_event_paranoid=4` and an unqualified cgroup CPU quota. Availability must
be tested on the actual host, and unavailable events reported as unavailable,
not zero. Whole-process counters include input generation and post-batch
oracles/capacity probes, so they cannot be called dispatch instruction counts.

### Internal WF and Rayon join events

`scheduler-events` extends the callback diagnostic with internal runtime job
counts. It accepts only the original WF and Rayon join selectors. Both use
the ordinary scalar common computation and callback objects. WF compiles
`runtime.c` with `WF_COMPUTE_EVENTS` and legacy statistics disabled; Rayon
builds a private patched `rayon-core` from the locked offline vendored graph.
The normal registry, adapter, dependency archive and timing executable remain
unchanged. The diagnostic patch adds safe counter operations; it does not
change the existing upstream job representation or latch protocol. The safe
Rust adapter's private diagnostic copy exports an ordinary C getter whose
symbol is discovered from that exact build's LLVM IR. This is an experiment
interface, not a WF language ABI or separate-compilation change.

Every event image keeps counters enabled at all three callback trace levels.
The collector interleaves this image with the original counter-free
`scheduler-trace` image, recording the image in an additional summary column.
Comparisons include changed code/data placement and snapshot effects; they
cannot isolate a constant cost per counter increment. The ordinary timing
image and the counter-free trace image contain no new runtime event banks or
hook calls. Matched local optimized LLVM IR is identical with the new event
macros absent, for the runtime, ordinary host and callback trace host.

After the usual trace header, an event report names its schema, bank count and
event count. Each call row is followed by one `runtime` TSV row per bank:
call index, bank index and event deltas in schema order. `wf-2` uses the 24
entries in [runtime_events.h](runtime_events.h). Its final four entries count
join-search attempts, idle-loop search attempts, join-search successes and
idle-loop search successes. The parser also accepts historical `wf-1` reports
containing only the original twenty entries. `rayon-join-1` uses the thirteen
entries listed in the private dependency's retained `metadata.txt`, implemented
by [records-rayon-events.patch](records-rayon-events.patch). Banks are worker
indices, not OS thread IDs; Rayon bank four separately observes unregistered
submitters. Each worker has its own aligned cumulative bank. Counters are read
atomically and never reset while workers run.

The host snapshots immediately outside each call's resource/clock envelope,
then validates the deltas before proceeding to the next call. Snapshot and
validation costs are outside the reported call clock but inside the batch.
They can also change worker state between calls. Every published job has joined
before the ending snapshot. Both the host and report parser independently
check publication = local pop + successful foreign steal = completion. The
WF check includes run entry, join count and both owner-inline execution paths;
the Rayon check includes both StackJob execution and `run_inline`, which
bypasses the executor. Completion increments precede DONE/latch publication
and do not add later accesses to a caller-owned frame. The common callback
output/inventory checks still run separately.

A positive binary join tree with N callbacks publishes N-1 queued jobs in this
qualified cohort. Empty calls publish zero. The original WF width-one adapter
bypasses its runtime and publishes zero for every N, whereas Rayon width one
still publishes N-1. Neither queued-job count equals OS context switches or
the number of common callbacks. Unsupported Rayon injection/broadcast/non-stack
job events and unused/external banks must be zero for this join-only equation.
This does not qualify their nonzero paths or general Rayon APIs.

WF failed-search, wait and signal counts can advance while the caller reads
other banks. They include background activity and are not exact instantaneous
per-call flows. In particular, `JOIN_WAIT` means entry to helping/wait logic,
`JOIN_PARK`/`IDLE_PARK` count condition-wait calls, their resume counterparts
count returns, and `SIGNAL` counts condition-signal calls. None establishes an
actual OS park, wake or context switch. `SLOT_REFUSAL` counts free-list
exhaustion only. This bounded join cohort requires zero such refusals; the
separate runtime protocol retains its exhaustion/fallback qualification.

Search origins name the immediate call site: helping inside a join, or the
worker's outer loop including its final search after publishing its idle bit.
An idle-acquired task's nested join therefore contributes to join-origin work;
the tag does not describe the OS thread state. Each origin attempt increments
before trying one victim, and each origin success increments before executing
the acquired job. The two success counts sum exactly to total successful
steals after all jobs have joined, checked by host and parser. Attempt deltas
are not constrained to sum exactly to aggregate attempts or to exceed success
deltas: snapshot reads are separate, and an attempt may begin before a call
and succeed after that call publishes work. The ordinary macro-off runtime
keeps the same execution and waiting policy.

The canonical check retains existing runtime/scheduler tests and adds an event
runtime protocol variant with deterministic held-thief owner-inline and
idle-origin success counts,
18 C sanitizer event cases and 144 interleaved ordinary smoke processes / 432
calls. The sanitizer event image instruments host/common C computation/runtime;
Rayon and its patched safe counter code retain ordinary Rust objects. Linux's
exact retained-allocation contract remains required. Missing/duplicate bank,
wrong completion/origin count, absent schema and trailing-report negatives check the
collector. Calibration interleaves 180 processes / 1,620 calls: original WF and
Rayon join, three callback levels, two images, five rotating/reversed passes,
the two 256-record trace inputs and a 4,097-record early-invalid input with
maximum length 64. All use grain16 and width four including the caller.

```sh
make -C research/experiments/compute-runtime check-scheduler-events OUT=/tmp/wf-scheduler
OUT=/tmp/wf-scheduler RESULTS=/tmp/wf-scheduler/events-calibration \
  sh research/experiments/compute-runtime/records-scheduler-trace.sh events-calibrate
```

A preliminary M1 screen independently checked all 180 processes, 1,620 calls,
2,488,860 output positions, 104,040 callback rows, 3,645 runtime-bank rows and
1,281 manifest hashes. With callback tracing disabled (`plain`), medians of
five process warm means in the counter-free trace image were:

| Input, grain16, width4 | WF runtime | Rayon join |
| --- | ---: | ---: |
| 256 early-invalid records, maximum length 65,536 | 3.511 us | 1.615 us |
| 256 Unicode records, maximum length 65,536 | 1.964 ms | 1.896 ms |
| 4,097 early-invalid records, maximum length 64 | 7.584 us | 10.375 us |

The WF long-input counter image records exactly fifteen completed queued jobs
per call, but its median process mean is 48,530.75 steal attempts. This makes
idle search/wait behavior a concrete investigation target, not a measured CPU
cost or proof of the best replacement policy. In this image all four WF and
Rayon completion banks participate in all forty warm long-input calls. On the
256-record short input, Rayon completes all queued jobs on the caller in 31/40
warm calls, while WF uses four completion banks in 31/40 and three in 9/40.
These are job-completion banks, not callback identities or ordinary histories.

Perturbation remains material: WF's long-input event/control paired median time
ratio is 0.958 (range 0.943--0.996), despite the extra instructions. Rayon's
4,097-record short-input ratio is 1.342 (range 0.980--8.604), including a
101.953 us event-image process mean. Do not interpret a counter image becoming
faster as negative instrumentation cost or promote its ordering. The local
screen has no fixed worker placement and is exploratory rather than a retained
CI artifact. The counter-free/event image SHA-256 identities are respectively
`a5235c8ff452af0824d1991c85d767c391b3878a22837e6b1242906d5b7c8dbe` and
`5b5ece137c1eba1ca824ed1e21a31ef1ebaf55c6019b72164ca5363688c542b2`.
The matching [`619d33d2` Linux run](https://github.com/mbbill/Whitefoot/actions/runs/34198733090)
completed the event panel. Artifact `compute-scheduler-linux`, ID `10045199631`,
ZIP SHA-256
`4034f6a56de98df56364c3a7d7b5625cab40ba8b45bcc54f0010fedbaf2cd774`, retains
the raw reports and build inputs. Independent reconstruction checked all 180
processes / 1,620 calls, including 810 event calls, summaries and conservation,
and all 304 scoped hash entries (35 source entries across 27 files checked
against the exact revision). All eighteen
actual event-image sanitizer cases passed their contracts: nine WF clean
exits and nine Rayon exit-23 reports with exactly 384 direct plus 1,520 indirect
retained bytes. This scope does not re-audit the older full timing panel.

This host is an Intel Xeon Platinum 8370C VM, two physical cores / four SMT
logical CPUs, process mask 0--3, scalar Clang 18.1.3. Quota and individual worker
placement remain unqualified. It is a different CPU from the earlier EPYC
7763 trace panel; absolute times across those runs are not revision effects.
Counter-free `plain` medians [ranges] of five process warm means, each over
eight calls, are:

| Input, grain16, width4 | WF runtime | Rayon join |
| --- | ---: | ---: |
| 256 early-invalid records, maximum length 65,536 | 2.634 [2.543--4.403] us | 6.688 [2.456--10.353] us |
| 256 Unicode records, maximum length 65,536 | 4.672 [4.587--4.734] ms | 4.665 [4.576--4.779] ms |
| 4,097 early-invalid records, maximum length 64 | 19.798 [18.718--30.200] us | 23.828 [22.304--29.022] us |

Every corresponding range overlaps. In the long-input `plain` event image, both
runtimes complete queued jobs on four banks in all forty warm calls. WF's
median process means are 32,136.125 steal attempts, 32,129.25 empty searches
and 4.375 successful steals; Rayon records 6.25 successful steals. The fixed
fifteen completed queued jobs and these search counts measure different work.
They do not price the searches in CPU cycles or distinguish join-origin from
idle-worker searches. On the 4,097-record input WF completes jobs on three
banks in all forty calls; Rayon uses three in 38/40 and four in 2/40. The
counter-free involuntary-switch process-mean medians are zero for WF and
4.375 for Rayon on that input, a co-observation rather than a causal attribution.

Linux `plain` event/control paired median time ratios [ranges] are 0.989
[0.978--1.007] for WF long input and 1.007 [0.953--1.010] for Rayon long input.
For the 4,097-record short input they are 1.180 [0.760--1.388] and 1.071
[0.909--1.212]. Keep observer-image timing separate from ordinary performance;
these comparisons neither isolate counter cost nor establish a fastest runtime.

A subsequent M1 `wf-2` screen checked 180 processes / 1,620 calls and 1,281
manifest hash entries. All 405 WF event calls conserved the origin successes,
including 1,620 per-bank partitions. At width four / grain16, the long Unicode
`plain` event image records median process means of 28,180.625 join-origin
attempts and 18,573 idle-origin attempts. Those separate medians must not be
added to reconstruct the median total. Per-process join / (join + idle) shares
have median 59.92%, range 55.88--71.12%; the caller contributes a median 40.99%
of join attempts, range 35.65--58.57%. Nested joins on other workers therefore
contribute substantially. This rules out interpreting the aggregate search
count as exclusively idle-loop work; it does not measure either origin's CPU
cost or establish which replacement policy would be faster.

In that same screen, counter-free `plain` long-input medians [ranges] of five
process warm means were WF 1.914 [1.873--1.948] ms and Rayon 1.870
[1.852--2.171] ms. WF's corresponding event/control paired ratio is 0.979
[0.973--1.023]. First calls remain separate, each warm mean contains eight
calls, and these unfixed-placement local results do not establish a speedup.
The counter-free/event image SHA-256 identities are respectively
`79093eb45285b4a95dfd43e6591f4881db58fafe2a4f7457cdd02276152a9dd1` and
`af5c17d4144d1b3a020ae111bd68c5aafd4f1042741fb9fe871f6caf3177ff8c`.
The [`c1da4761` Linux run](https://github.com/mbbill/Whitefoot/actions/runs/34202766630)
subsequently qualified `wf-2`. Artifact `compute-scheduler-linux`, ID
`10046719545`, ZIP SHA-256
`91bee7fa66e701d85b8b411a4bb26a2ca4112ca6d81b03dbdb7227bdbb7ecc56`, retains
180 processes / 1,620 calls. Independent checking matched all summaries and
304 scoped hash entries, including 35 source entries against the exact revision,
and all 1,620 WF per-bank success partitions. All eighteen actual event sanitizer
contracts passed: nine WF clean exits and nine Rayon exit-23 reports with
1,904 retained bytes in two allocations. The earlier `wf-1` panel cannot supply
origin counts and remains separate evidence.

This runner is an EPYC 7763 VM with two physical cores / four SMT logical CPUs,
mask 0--3, with quota and individual worker placement unqualified. Long-input
`plain` join-attempt shares have median 63.10% [54.71--69.79%]; the caller
supplies 40.75% [38.58--43.82%] of join attempts. This again locates substantial
search work in worker-side nested joins, without estimating its CPU cost.
Counter-free long-input medians [ranges] are WF 3.924 [3.881--3.939] ms and
Rayon 3.919 [3.833--3.996] ms, with overlapping ranges. The corresponding
event/control paired ratios are 1.278 [1.269--1.334] and 1.272 [1.238--1.293],
slower in all five pairs for both runtimes. This large shared perturbation
requires separating image and instrumentation effects; it is not an isolated
WF counter-overhead estimate or evidence of a production performance loss.

### First complete Linux scalar panel and attribution limit

The [`f8766994` run](https://github.com/mbbill/Whitefoot/actions/runs/34186817212)
completed all 5,940 processes / 58,860 calls, checking 1,087,845,120 output
positions. Artifact `compute-scheduler-linux`, ID `10040879095`, ZIP SHA-256
`6dc84ee149ccb08d70021c665f56fd3a039576141166a4f5f7a51ca234e28fd5`, retains
all raw rows, qualifiers and build objects. Independent reconstruction matched
every raw identity and all 33 summary fields; all 517 manifest entries and
the dependency hashes match, including previously omitted hidden Cargo files.
All 42 sanitizer cases and the actual extra-allocation negative check passed.
Six timed processes needed the second post-timing capacity wave; none needed
the third. These waves do not change or repeat measured samples.

This host was an AMD EPYC 7763 VM with two physical cores, four logical CPUs
(SMT2), and mask 0-3. CPU quota and individual thread placement are unqualified.
The following are microseconds, median of five process warm means, at width
four, grain 16, dense cadence. Width includes the caller. The 256-record cells
have four warm calls per process; the 4,097-record cells have fifteen.

| Input (records / maximum bytes per record) | WF runtime C adapter | Static spin | oneTBB | Parlay | Rayon join | Rayon iterator |
|---|---:|---:|---:|---:|---:|---:|
| Unicode 256 / 65,536 | 6,221.531 | 3,946.476 | 6,508.005 | 4,142.592 | 4,001.768 | 4,032.950 |
| Unicode 4,097 / 128 | 298.782 | 1,955.208 | 281.870 | 231.500 | 218.178 | 211.964 |
| Early invalid 256 / 65,536 | 1.711 | 3,502.936 | 3.972 | 2.339 | 7.436 | 14.479 |
| Early invalid 4,097 / 128 | 16.548 | 2,269.907 | 19.337 | 16.653 | 26.732 | 26.500 |

The long Unicode difference already exists at width one: WF's executable takes
10.72 ms, oneTBB's 10.68 ms, and the other four about 7.3 ms. WF's width-one
adapter directly calls the common callback without entering the runtime.
All six linked `records_state` bodies have identical 258-byte instruction
sequences (SHA-256
`01177681415f446ee487fc4c41ca50c4b3146d4b8bb421b1a1f4dc9a10e9de9c`).
Their function addresses modulo 64 are 48 for WF and oneTBB, 0 for static
and both Rayon forms, and 16 for Parlay. The comparison at function offset
`0x4e` crosses a 64-byte boundary only in the slower pair. This is a layout
correlation, not a demonstrated hardware cause; it invalidates a scheduler-only
explanation of the full gap. Shared-executable and instruction-placement
controls are needed before selecting a runtime optimization from this result.

Cadence also changes the conclusion. For early-invalid 256 records, WF's
median increases from 1.711 microseconds dense to 12.676 with requested 100-us
gaps and 13.573 with requested 1-ms gaps. Parlay's corresponding values are
2.339, 3.707 and 3.830. The static pool's millisecond short-call costs coincide
with batch CPU/wall ratios near three on this two-core VM; they do not establish
a general WF advantage or a measured syscall cause. Idle CPU and wakeup latency
must be considered together. The 20 or 75 warm observations per cell support
descriptive samples, not production tail estimates or a universal ranking.

### Linux shared-image result

The [`d6bf6c08` scheduler run](https://github.com/mbbill/Whitefoot/actions/runs/34189375566)
completed both panels on one EPYC 7763 VM with two physical cores, four logical
SMT CPUs and mask 0-3. Artifact `compute-scheduler-linux`, ID `10041761880`, ZIP
SHA-256 `3ea155d8dfe4606936e5cdda2494652069f49df5983e530a60dbcab153fccfd0`,
retains 7,325 verified ZIP paths. Auditing reconstructed all 5,940 standalone
and 360 shared-image process summaries, covering 58,860 and 3,780 calls
respectively. All 528 standalone and 531 shared manifest entries match,
including hidden dependency fingerprints. The original 42 sanitizer cases,
extra-allocation negative test and all eighteen added host/selector sanitizer
qualifiers passed on Linux. The six new Rayon cases had actual exit 23 and
exactly the retained caller-registration allocations; other new cases exited
zero. All sanitizer aliases have identical executable bytes.

The shared leaf is at image address `0x1e0c0`, aligned to 64 bytes. Its 258
instruction bytes and the 107-byte callback match all six standalone images.
Standalone long Unicode width-one medians still separate into WF/TBB at
10.684 / 10.756 ms and the other four at 7.264–7.306 ms. In the shared image,
all six width-one medians lie at **7.261–7.300 ms**. The large difference
disappears in this cohort. Shared linking also changes the dispatcher, loaded
libraries and run period; this does not isolate instruction alignment as the
hardware cause.

Shared-image width-four results follow, in microseconds: median of five
process warm means, grain 16, dense cadence. The long cells have four warm
calls per process; short cells have fifteen. This remains the C runtime adapter,
not the real WF record program.

| Input (records / maximum bytes per record) | WF runtime C adapter | Static spin | oneTBB | Parlay | Rayon join | Rayon iterator |
|---|---:|---:|---:|---:|---:|---:|
| Unicode 256 / 65,536 | 4,148.285 | 3,971.289 | 4,291.637 | 4,261.278 | 4,105.678 | 4,179.324 |
| Unicode 4,097 / 128 | 199.610 | 1,955.663 | 204.300 | 217.349 | 213.116 | 207.709 |
| Early invalid 256 / 65,536 | 1.846 | 4,751.730 | 4.095 | 2.465 | 10.600 | 2.635 |
| Early invalid 4,097 / 128 | 36.605 | 2,670.381 | 18.044 | 17.485 | 27.022 | 24.045 |

Long Unicode WF is about 1.0% slower than Rayon join and 4.5% slower than
static partition in this screen, rather than the earlier apparent 50% loss.
Its process means range from 4.024 to 4.549 ms, overlapping Rayon's
4.030–4.198 ms. Median warm CPU sums are 14.864 / 14.810 ms respectively.
This supports competitive current-stack execution for this cell, not a stable
1% ordering or general optimality.

The adverse short result remains: early-invalid 4,097-record WF process means
range from 15.085 to 42.574 microseconds, with median 36.605 versus Parlay's
17.485. The standalone WF median on this same host was 15.424 microseconds at
grain 16 and 12.500 at grain 64. Neither the adverse shared row nor this
cross-panel variation is discarded. Call counts are small, pool capacity is
not useful parallel width, and these finite bursts do not establish steady
state. Kernel scheduling, idle policy, decomposition and code placement still
need separate attribution; short-call rankings and production tails remain
unqualified.

The scheduler header, common callback/host, adapters, locked Rayon crate, shell
drivers and shared report validators
belong to this mechanism experiment. Retain them while common-code attribution
is needed; consolidate them when a broader maintained compute harness provides
the same boundary. The extracted oracle remains shared with the real WF host.
