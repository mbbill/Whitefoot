# Compute runtime recovery control

This experiment isolates current-stack compute join/help/steal from the shared
I/O scheduler. It supplies a research runtime for the current compiler's
ordinary task frames, checks the concurrent protocol, and links the same
unmodified emitted module with this runtime and the compiler's weak sequential
fallback. The normal compiler link driver is unchanged. There are no performance
results yet.

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

Requires a POSIX LP64 host, Clang with sanitizers, pthreads, `nm`, and the current
offline Cargo dependencies. The root `make check` calls this directory's
`check` target through `research-tests`, using ASan/UBSan by default.

```sh
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

`fir.wf` computes every output of a causal finite impulse response filter with
1 to 64 taps and K-1 initial history samples, oldest first. Tap zero multiplies
the current sample. Each output accumulates in ascending tap order using
distinct `fmul.strict` and `fadd.strict` operations. Samples preceding the block
come from its history; an empty block preserves that history.

The filter recursively divides the output interval and computes independently
owned tiles. Ordinary sibling calls expose the overlap; their joins complete
before the parent returns an owned tree. Source count/get/release functions let
the host check the whole result without adding a serial flattening pass to the
filter. A missing sample is an actual accessor result, checked as such by the
host. The WF history accessor reads the final K-1 samples of the held input
prefix, and its results become the history of subsequent real WF calls.

The original command remains a complete WF program and checks a small impulse.
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
FIR Linux execution and timing remain unqualified in this revision.

### Costs and gaps exposed by this program

The host invokes channels sequentially; WF parallelism currently exists inside
each channel across independent tiles. This does not yet qualify mixed-channel
co-scheduling or a full multistage signal pipeline. Initialized output buffers,
box/tree allocation and recursive dispatch remain in the computation. They
must be charged to a future filter-core timing. Prefix construction, state
queries, result access and destruction must be charged to the corresponding
end-to-end boundary; none has been timed or silently subtracted here. The
scalar oracle is a correctness reference, not a qualified fastest native filter.

The first range contract expressed `end - first <= limit`. Current S4 admits
that as an opaque requirement without projecting its arithmetic into the body
affine state. The accepted program instead states ordinary endpoint bounds,
including the absolute prefix limit, and proves its derived allocation/index
bounds in the body. This is a useful contract-expressiveness limitation to
investigate, not evidence that a required proof can be replaced by a runtime
guard. The active [specification](../../../spec/kernel-spec.md) still selects
those rules; this experiment changes neither acceptance nor the public ABI.
