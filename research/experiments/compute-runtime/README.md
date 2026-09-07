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
`abi.wf` is only a small ABI diagnostic, not the substantive application suite
required by that coverage. Keep this control while it distinguishes recovery
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
waiter ordering, exhaustion under deep helping, Linux qualification, lifecycle
costs and performance measurement remain open. No timing conclusion can be
drawn from the local sanitizer runs.
