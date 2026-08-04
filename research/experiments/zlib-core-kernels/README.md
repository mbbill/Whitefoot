# zlib core-kernel proof and lowering study

Status: **deferred research record**. This directory contains reproducible
isolated-kernel evidence and disposable stage-0 compiler prototypes. It does
not change the language specification, `docs/patterns.md`, production compiler,
Direction Outline, or Current Plan.

The study asks a narrower question than “can Whitefoot implement zlib?”:

> Can the obvious checked scalar source for two important RFC 1951 inflate
> kernels reach optimized-library performance when compiler proofs select a
> different machine strategy?

The answer on the measured Apple M4 host is **yes for these two isolated
kernels, with important limits**. Literal lowering of the source is not
competitive in every case. Removing bounds checks alone is also insufficient.
Two closed, strategy-changing compiler prototypes recover the missing
performance without changing the Whitefoot sources.

## Read this first

1. `RESULTS.md` records the corrected ordinary-lowering measurements and why
   the default machine shape loses.
2. `PERIODIC-COMPILER-RESULTS.md` records the periodic-copy compiler result.
3. `GUARDED-COMPILER-RESULTS.md` records the guarded Huffman bit-window result.
4. `DESIGN-HANDOFF.md` explains the proofs, their incompleteness, the proposed
   writer patterns, production-compiler prerequisites, and pickup gates.

The central conclusion is:

```text
canonical checked source
        + finite machine-verified theorem schema
        + strategy-selecting lowering
        = competitive isolated-kernel machine code
```

The temporary recognizers are not complete production proofs. The periodic
prototype calls hand-written ARM NEON C after exact structural and contract
matching. The guarded Huffman prototype accepts one alpha-normalized AST
digest, certifies its constant table, and emits specialized LLVM directly.
Both mechanisms deliberately fail closed, but neither supplies the production
proof objects, independent body-derived obligations, per-site provenance, or
verified lowering refinements required before check elision can ship.

## What is included

- `match_copy.wf` and `huffman_literals.wf`: unchanged canonical scalar
  Whitefoot sources.
- `bench.c`, `huffman_bench.c`, `zng_kernel.c`, and
  `zng_huffman_kernel.c`: correctness and pinned zlib-ng comparison harnesses.
- `results.json` and `huffman-results.json`: corrected ordinary-lowering raw
  measurements.
- `periodic_copy_helper.c`, `run_periodic_compiler.py`, and
  `periodic-compiler-results.json`: periodic compiler experiment.
- `run_huffman_guarded.py`, `test_guarded_bit_window.py`, and
  `huffman-guarded-results.json`: guarded bit-window compiler experiment.
- `bounds-elision-ceiling/`: evidence that proving away implicit bounds checks
  does not by itself invent the required algorithms.
- `manual-lowering-ceiling/`: the hand-written C strategy probes that separated
  attainable machine performance from compiler-proof feasibility.
- `compiler-prototypes/`: the two independent stage-0 compiler patches and
  their baseline/optimized LLVM snapshots. They are archived for inspection,
  not applied to the current compiler.
- `assembly/`: textual ARM64 disassembly snapshots generated from the measured
  objects and executables.
- `SHA256SUMS`: integrity manifest for the archived experiment.

No executables, object files, zlib build trees, corpora, or Python caches are
tracked. They are regenerable or machine-local.

Verify the archived text and raw measurements without rebuilding native code:

```sh
python3 -B experiments/zlib-core-kernels/verify_archive.py
```

## Ordinary-lowering reproduction

From the repository root:

```sh
python3 experiments/zlib-core-kernels/run.py
python3 experiments/zlib-core-kernels/run_huffman.py
```

The runners expect a clean zlib-ng 2.3.3 checkout at commit
`12731092979c6d07f42da27da673a9f6c7b13586` and its native build directory.
Set `WHITEFOOT_ZNG_ROOT` when it is not at the recorded local default.

The compiler-prototype runners require the corresponding archived `democ.patch`
applied to the recorded historical base. They are intentionally not runnable
against current production `wfc`; see `compiler-prototypes/README.md`.

## Scope boundary

These functions exclude block transitions, dynamic Huffman table construction,
length/distance dispatch around the kernels, checksum, stream state, allocation,
and I/O. The Huffman comparison is an all-literal projection. The match-copy
benchmark repeats identical matches and therefore exposes amortization that a
mixed token stream may not preserve.

The earlier complete raw-DEFLATE default-shape harness remains in
`../raw-deflate-default-shape/`. No correctness-green model candidate or score
was produced before the work was redirected to these core kernels. A mixed
literal/match replay on frozen real distributions remains the next performance
experiment before any whole-decoder claim.
