# Compute runtime recovery control

This experiment isolates current-stack compute join/help/steal from the shared
I/O scheduler. It supplies a research runtime for the current compiler's
ordinary task frames, checks the concurrent protocol, and links the same
unmodified emitted module with this runtime and the compiler's weak sequential
fallback. The normal compiler link driver is unchanged. The FIR calibration
caller below also links the identical optimized WF object with the existing
shared runtime, and compares qualified native output-lane SIMD candidates.
This is cost attribution, not a confirmed performance frontier. The current
[scheduler panel](#scalar-scheduler-comparison) disables SIMD and compares
equal worker budgets with one common scalar compute object. Earlier FIR SIMD
results below retain their original conditions and do not rank these schedulers.

The [investigation](../../investigations/compute-runtime/README.md) owns the
architecture question, [WF workload coverage](../../investigations/compute-runtime/WORKLOADS.md)
and [native references](../../investigations/compute-runtime/BASELINES.md).
`abi.wf` is only a small ABI diagnostic. The variable-input FIR program below
is the first substantive computation; the broader application suite remains to
build. Keep this control while it distinguishes recovery
from the shared runtime; consolidate it into the compiler runtime and its tests,
or remove it, when a qualified replacement makes the duplicate unnecessary.

## Source and execution boundary

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

Several historical choices remain deliberately visible: 64 slots per lane
(the current shared runtime has a different capacity), at most 64 lanes,
4096 empty searches before 16 yields, and the old split-budget thresholds.
Successful steals still increment a shared diagnostic counter. These are
controls to measure and change, not selected optimal settings.

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

The six C probe invocations exercise:

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
control, not a native performance ceiling. Current runtime statistics remain
enabled, including recovery's shared atomic steal counter; differences include
those costs and the different slot/idle policies. Fine-grained scheduler-only
claims need a later instrumented-versus-uninstrumented comparison.

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
and recovered full qualifiers, a C-sanitized qualifier, and eighteen timing-host
smokes. Both timing hosts reuse the exact qualified O3 WF and native objects.
The C-sanitized qualifier also instruments a separate native object; ordinary
WF LLVM does not thereby acquire frontend sanitizer instrumentation. Keep these
record source/adapter/native/host/driver files while this workload needs the
experiment; consolidate them if a shared workload harness supersedes their role.

Run `make -C research/experiments/compute-runtime records-calibrate OUT=/path/to/output
BENCH_ARCH=-march=native` on the selected Linux host (one shell line). The driver
records five rotating/reversing passes across twenty cells and six controls:
native state/word, weak WF, and recovered WF at requested zero/two/four lanes.
The 600 processes cover ASCII, mixed Unicode, early/late invalid bytes and
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
The two native kernels are initial scalar anchors. This end-to-end panel does
not establish a native frontier or full application throughput. SIMD work is
parked. The separate scheduler comparison below supplies matched static/dynamic
controls; held-out inputs and dedicated-core measurements remain open.

## Scalar scheduler comparison

This panel isolates scheduling with four implementations: the recovered WF
runtime, a persistent static busy-spin pool, oneTBB, and Parlay. All four link
the **same separately compiled** `records_state` computation and
`records_scheduler_chunk` callback objects. Every callback processes the same
fixed range of records and writes caller-provided output. The timed path does
not select the ASCII-word candidate or any SIMD library. C and C++ builds use
`-O3 -DNDEBUG -fno-vectorize -fno-slp-vectorize -fno-lto`; oneTBB's library build
also disables automatic vectorization and IPO. The retained assembly permits
inspection of the actual compute/callback functions. These controls concern
compiler-generated code in this experiment and its native library, not the
implementation of operating-system routines.

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

Widths one, two and four include the caller. Each process has one immutable
width and one external caller. The shared workload is flat and its callbacks
perform finite independent CPU work; the static adapter cannot nest. TBB and
Parlay may group chunk indices internally, and the WF and Parlay split orders
differ: equal callback work does not imply an identical internal task graph.
These are initial qualified API choices, not proof of each library's fastest
partitioner or a complete reference frontier. Rayon and OpenCilk still need
executable compute controls.

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
call. Up to three explicit witness waves accommodate lazy startup. A blocking
cross-index barrier was rejected as a capacity probe: schedulers are allowed
to execute independent indices sequentially, and Parlay's cold elastic startup
can delay helper participation. No measured callback waits for another index.

`check-scheduler` runs the complete 4,595,603-input leaf oracle on the exact
scalar timing object. Each backend/width qualifier then checks 780 batches /
270,660 output positions, exactly-once callback indices, joined callback tails,
and immutable inputs across five shapes, uneven sizes and four grains.
Sanitized copies cover the C/C++ host, scalar work, adapters, recovered/static
runtimes and Parlay headers; the linked oneTBB shared library remains an
ordinary scalar Release build. Sixteen sanitized cadence smokes additionally
exercise the timed-host buffer lifecycle across all backends and cadences.
These checks do not substitute for upstream
library suites. The driver validates all raw row identities and retains
source/object/library hashes, flags, qualification logs and host topology.

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
their input seeds. Five passes over 66 cadence cells and twelve backend/width
configurations yield 3,960 processes / 39,240 calls. The gate runs the original
three smoke inputs at all four cadences: 144 processes / 432 calls. Treat these
as same-host screens; worker affinity within the mask, sustained idle costs
and held-out confirmation require further work.

The scheduler header, common callback/host, four adapters and two shell drivers
belong to this mechanism experiment. Retain them while common-code attribution
is needed; consolidate them when a broader maintained compute harness provides
the same boundary. The extracted oracle remains shared with the real WF host.
