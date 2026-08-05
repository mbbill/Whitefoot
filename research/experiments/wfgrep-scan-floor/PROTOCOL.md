# WF-SCAN-FLOOR protocol

Status: FROZEN BEFORE COMPARATIVE CURRENT-COMPILER TIMING

The Git commit containing this status and the complete bundle is the freeze
identity. Comparative timing is forbidden before that commit exists.

Authority: owner-approved bounded parallel research on 2026-08-05. This
experiment may inform Direction Outline items `PERF-1`, `FLOOR-1`, and
`FLOOR-2`. It authorizes no numbered-specification, compiler, proof, intrinsic,
runtime, system-capability, matcher, or wfgrep implementation change.

## Question and hypothesis

Can the active v0.17 language and ordinary safe-Rust compiler express two
single-buffer scanner shapes at competitive same-algorithm machine quality?

1. `full`: a complete byte pass that counts newline bytes and one candidate
   byte using the taught P7 Boolean-dataflow shape.
2. `early`: four early-exit byte searches whose matches occur at one quarter,
   one half, the final byte, and nowhere.

The primary hypothesis is that Whitefoot and C compiled by the same Apple
Clang `-O2` backend fall inside a preregistered +/-5% practical-parity band for
the full-pass kernel. The current compiler emits an explicit bounds check at
each source index, but the loop guard should let LLVM remove that redundant
check and vectorize the full pass. The early-exit result is exploratory: the
same scalar algorithm may expose control-flow or check-lowering differences,
but no vector or library-memchr win is assumed.

Safe Rust is a secondary comparator. If C and Rust disagree materially, the
result is attributed to toolchain, source shape, or harness differences before
making a Whitefoot-language claim.

## Frozen semantic work

Both kernels receive one opaque runtime buffer descriptor and return one
wrapping `u64` checksum. They allocate nothing, perform no system operation,
and retain every required Whitefoot check.

`full` scans exactly 67,108,864 bytes per call. It increments two wrapping
counters for bytes 10 and 80 and returns:

    lines + candidates * 1,000,000,007  (mod 2^64)

The timed region executes 128 calls after three correctness-checked warmups,
for exactly 8 GiB of scanned input.

`early` searches in source order for bytes 251, 252, 253, and 254. The input
generator emits only bytes 0 through 249, then places 251 at `length/4`, 252 at
`length/2`, and 253 at `length-1`; 254 is absent. One call therefore performs
`2*length + length/4 + length/2 + 2` byte examinations. The timed region
executes 24 calls after three checked warmups.

The input generator is xorshift64 from seed `0x8f3f73b5cf1c9ade`, applying
left-13, right-7, left-17 XOR shifts and storing `state % 250`. Every executable
reports the same FNV-1a-64 data hash, checksum, repetition count, and length.
Any disagreement invalidates the run.

## Correctness

Before measurement, every binary must pass empty, one-byte, mixed marker,
all-byte, and generated-buffer cases. The C/Whitefoot harness uses a distinct
branching oracle; the safe-Rust harness uses iterator-based `filter` and
`position` oracles. The timed checksum is checked after every process's timed
region. A crash, trap, nonzero exit, stderr, malformed output, work-identity
change, or checksum mismatch invalidates the attempt.

The full-pass result is a scanner-floor proxy, not grep output correctness. It
does not implement substring matches, line offsets, Unicode, regex, traversal,
I/O, formatting, or publication.

## Compiler and harness boundary

The Whitefoot sources pass through the ordinary `whitefootc --emit-llvm` path.
The experiment's `expose.rs` then performs exactly two verified textual
changes:

- `wf_scan` changes from internal to external linkage; and
- the generated but unused host `main` symbol is renamed.

It rejects zero or multiple matches. It changes no target function body,
instruction, check, attribute, type, target triple, or data layout. The
resulting object links behind the same C harness as the C control. This is an
experiment transport boundary, not a compiler capability or proposed ABI.

The primary Whitefoot/C comparison shares:

- the exact C harness and `CLOCK_MONOTONIC_RAW` timer;
- one opaque cross-object call per kernel invocation;
- Apple Clang 21.0.0 at `-O2`, without LTO or target-specific CPU flags; and
- the same runtime buffer bytes and oracle.

The C control spells the same index loops and wrapping unsigned arithmetic.
It is not allowed to call `memchr` or use intrinsics.

Safe Rust remains free of `unsafe`. Its kernel is a separate rlib and its
harness uses `Instant`, preventing whole-program specialization without adding
an unsafe C-ABI slice conversion. It uses rustc 1.97.1, LLVM 22.1.6,
`opt-level=2`, `panic=abort`, one codegen unit, no LTO, and no target CPU flag.
The timer difference is a disclosed secondary-comparator limitation.

The target is the same Mac16,12 Apple M4 host recorded by RG-BASE: macOS 26.5.2
build 25F84, 16 GiB RAM. Measurement requires AC Power and Low Power Mode off
before and after the run. This is a target-specific result.

## Code-shape evidence before timing

The committed build always retains:

- raw Whitefoot LLVM before the linkage rewrite;
- optimized Whitefoot LLVM;
- Whitefoot, C, and Rust assembly; and
- the six executable controls under the uncommitted work root.

Review records, for each shape and comparator:

- target-loop loads and calls;
- raw and optimized trap/check edges;
- vector loop presence and lane width;
- scalar early-exit control shape;
- unexpected allocations, copies, or helper/library calls; and
- whether the final target body still implements the frozen work.

A measured ratio receives no mechanism attribution unless the expected final
code difference is present. Dead-code elimination, constant folding of the
runtime buffer, library-call substitution, changed work, or setup work inside
the timer invalidates the comparison.

## Measurement schedule and statistics

The fixed run id is `wf-scan-floor-1`; its create-once directory must not exist.
Each binary performs allocation, input generation, oracle construction, and
three warmups before starting its internal kernel timer. Process startup,
allocation, generation, oracle, and output are excluded from the kernel result.

There are 30 paired rounds. Each of all six Whitefoot/C/Rust execution orders
appears exactly five times. Full and early shapes alternate first position by
round. Samples are never deleted or extended after observing a result.

For each round and control:

    ratio = control elapsed / Whitefoot elapsed

Thus 1.0 is parity, below 1.0 means Whitefoot is slower, and above 1.0 means
Whitefoot is faster. The point statistic is the median of 30 within-round
ratios. A deterministic 10,000-resample bootstrap over complete rounds uses
seed 20260805 and reports a descriptive 95% percentile interval.

The interpretation bands are fixed:

- practical parity: the complete interval lies within `[0.95, 1.05]`;
- material Whitefoot loss: the complete interval lies below `0.90`;
- material Whitefoot win: the complete interval lies above `1.10`;
- directional only: the interval excludes 1.0 but does not clear a material
  threshold; and
- otherwise inconclusive.

Relative interval half-width above 5% is reported as precision-inconclusive
even if a point estimate crosses a band. The C comparison is primary. A Rust
comparison may corroborate, narrow, or challenge attribution but cannot
override unexplained disagreement with the same-Clang C control.

## Decision routing and stop

The experiment stops after both shapes are classified and their first material
cause is attributed among source shape, required check, emitted lowering, LLVM
recovery, final binary, toolchain/harness, and noise.

- Parity validates only these current ordinary scanner shapes.
- A slower-but-accepted source alternative is a `FLOOR-1/FLOOR-2` finding.
- A Whitefoot-vs-equivalent-C final-code gap is a `PERF-1` finding.
- A material retained-check gap may motivate a later `PROOF-1` proposal only
  after its exact proposition and consequence are separately authorized.
- A gap to a different memchr/SIMD/substring algorithm is algorithmic ceiling
  evidence, not proof of a language feature.

No result in this bundle authorizes a compiler or specification change. Do not
add substring search, regex, Unicode, filesystem work, output, threading, or a
favorable extra workload after seeing results. Record positive, negative, or
inconclusive evidence, update only MCTS facts that pass their admission test,
and stop.
