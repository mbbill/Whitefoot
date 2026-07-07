# Numeric semantics and LLVM poison/undef/freeze

Phase-2 numeric workflow `w6jgx7vrd` produced LLVM-heavy evidence cards in `notes/phase2-numeric-findings.jsonl` and `synthesis/phase2-numeric-findings-index.md`.

## Workflow-confirmed LLVM facts

- LLVM has an explicit undefinedness lattice: immediate UB, poison, undef, freeze(poison), concrete value. Valid transforms must move toward more-defined semantics.
- Unflagged integer `add` wraps modulo bit width; `nuw`/`nsw` make overflow produce poison.
- Integer division by zero is immediate UB, so it cannot be speculated before guards when zero is possible.
- `range` metadata is a semantic promise for integer loads/call returns: outside-range values become poison.
- `getelementptr` has pointer-index contracts: non-`inbounds` GEP may compute out-of-bounds addresses without memory access, while `inbounds` out-of-object results become poison, and dereferencing a pointer derived from one object as another separately allocated object is invalid.
- InstCombine treats these contracts as canonicalization constraints and expects flag-sensitive tests/proofs, often via Alive2.

## Still undecided

No source-language default is selected. The following need more primary-source evidence and debate before becoming design candidates:

- Checked vs wrapping vs trapping vs poison-like integer overflow defaults.
- Whether source-level UB is acceptable or should be replaced with proof, debug checks, traps, or unsafe-only assumptions.
- Strict floating-point versus fast-math opt-in defaults.
- Reassociation, FMA/contraction, rounding modes, FP exceptions, NaNs, signed zero, infinities.
- Cross-platform reproducibility versus target-specific performance.
- GPU/approximate math behavior.

## Implementation warning

LLVM numeric attributes/metadata are not comments. They are contracts. A frontend for an optimizer-first language must emit them only when the source semantics, verifier, runtime checks, or unsafe boundary make the promises true.

## Trap-vectorization tiers (2026-07-05, walkthrough refinement of the round-3 relaxed-trap-ordering item)

Owner question resolved into a three-tier analysis; the open spec item should be drafted as a BATCH-REPLAY EQUIVALENCE rule, not weakened trap semantics:

1. Induction arithmetic: 64-bit counters guarded by `i < n` make `iadd.trap` provably non-overflowing; the check eliminates by proof (OP-4) and the proof itself licenses nsw-grade SCEV facts. Trap is free; same codegen as C's UB assumption, opposite trust model.
2. Element-wise data arithmetic: vectorize as compute -> vector overflow mask -> commit-after-check, with scalar replay from batch start on any set lane. Preserves precise trap semantics bit-exactly (stores before the trapping iteration committed, none after). Cost ~one mask op per batch (A005-pattern). SPEC OBLIGATION: trap semantics must explicitly permit batch-deferred stores + scalar replay.
3. Reductions: reassociation changes which intermediate values exist, so precise trap semantics cannot survive vector reduction (integer analog of FP reassociation). Canonical restructurings: widened accumulator (provable no-trap, then checked narrow) or explicit .wrap/.checked. Teaching-pack idioms: counters u64+trap; reductions widen-then-narrow.
