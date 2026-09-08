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

Several historical choices remain deliberately visible: 64 slots per lane
(the current shared runtime has a different capacity), at most 64 lanes,
4096 empty searches before 16 yields, and the old split-budget thresholds.
These are controls to measure and change, not selected optimal settings.
`WF_COMPUTE_STATS=1` retains the shared successful-steal counter and its
observer by default for existing diagnostics. The scalar scheduler timing and
sanitizer objects use `WF_COMPUTE_STATS=0`: both the counter and observer are
absent, checked with `nm`, rather than returning an invented zero count.
Statistics do not participate in task publication, ownership or completion.

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
For the pinned Rayon caller-worker API only, the required lifecycle report is
one 384-byte direct worker allocation and one 1,520-byte indirect queue block,
with the exact semantic allocation stacks checked by
`records-scheduler-memory.awk`. All fourteen Rayon reports in the
[`08e58d63` scheduler run](https://github.com/mbbill/Whitefoot/actions/runs/34184353021)
had that same shape, after successful functional qualification. The checker
allows relocation addresses, source line numbers and compiler hashes to vary;
changed allocation sizes, counts, stack identities, incomplete output, any
additional diagnostic, and any other exit status fail. This documents retained
process-lifetime storage, not a memory-clean teardown. No leak suppression is
installed. Other platforms and non-address sanitizer builds require clean
exit-zero reports; the explicit replay mode only tests archived Linux reports.

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
and held-out confirmation require further work. The Linux scheduler job selects
the explicit C/C++ target `-march=x86-64-v3`; the Rust scheduler library and
oneTBB library use their default host targets. The explicit target avoids
Clang 18's invalid AVX10 combination inferred by `-march=native` on one CI
host; strict warnings and scalar controls remain enabled. New measurements
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
Linux `wf-2` execution remains unqualified; the retained Linux panel above
uses `wf-1` and cannot supply origin counts.

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
