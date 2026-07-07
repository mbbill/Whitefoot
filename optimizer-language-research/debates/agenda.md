# Debate agenda

Use confirmed findings as exhibits, not conclusions. Every debate item should cite evidence cards and gap records, or conclude `research_needed`.

## Agenda

- Evidence standard first: what qualifies as enough support for a design implication, and how should the corpus weigh specification text, implementation docs, source code, academic papers, benchmark data, production reports, and historical retrospectives?
- Breadth check before feature ranking: is the corpus broad enough to compare optimizer enablers, or must debates pause until arrays/layout/vectorization, dispatch/generics, numeric semantics, dynamic typing/JIT, memory models, concurrency, effects, and ABI have workflow-confirmed evidence?
- Layering check: for each claim under discussion, is it a source-language semantic rule, verifier/proof obligation, compiler analysis, IR annotation, runtime/JIT mechanism, ABI representation, backend pass fact, or empirical performance result?
- Aliasing as one option among several: should non-aliasing be modeled as a default semantic rule, opt-in contract, proof obligation, inferred property, dynamic guard, unsafe promise, or conservative optimization opportunity? Compare Rust, SPARK, Fortran, C restrict, Clean uniqueness, Pony capabilities, and JIT guard/deopt systems without selecting a winner.
- IR preservation question in neutral form: what evidence exists that preserving each source-level fact into MLIR or LLVM enables specific optimizations, which passes consume it, where can it be invalidated or dropped, and what costs or hazards follow?
- Managed/runtime question: when are escape analysis, scalar replacement, inline caches, guards, tiering, and deoptimization sufficient, and when would a language-level allocation, lifetime, ownership, or effect guarantee be needed?
- Exception/error/effect question: compare result types, return codes, zero-cost exceptions, funclets, checked exceptions, algebraic effects, condition systems, panic/abort, setjmp/longjmp, coroutines, and async cancellation while separating source ergonomics from ABI/backend representation.
- Array/layout/vectorization question: should array shape, layout, iteration spaces, contiguity, bounds, and scheduling be privileged as much as aliasing? Debate Fortran/APL/HPF/SISAL/NESL/SAC/Futhark/Halide/ISPC/MLIR evidence needs against the current alias-first seed corpus.
- Numeric semantics question: how much freedom should optimizers get around overflow, poison/undef/freeze, fast-math, NaNs, signed zero, rounding, reproducibility, and target-specific approximations? What user failure modes are acceptable?
- Concurrency and memory model question: how should data-race rules, atomics, actors, tasks, channels, Send/Sync-like capabilities, STM, GPU address spaces, and foreign threads bound alias/lifetime/value assumptions?
- Implementation gate question: what minimal semantic contract, LLVM/MLIR mapping, verifier check, threat model, and FileCheck or benchmark validation must a feature pass before moving from research evidence to design_candidate?
- Historical failure question: which failures must any future design explicitly avoid, including strict-aliasing surprises, UB/provenance traps, poison/undef miscompilations, checked-exception ergonomics, dynamic JIT warmup/deopt cliffs, Fortran aliasing exceptions, GPU portability traps, and unsound frontend metadata emission?

## Allowed outcomes

- research_needed
- prototype_needed
- evidence_conflict
- design_candidate_later
- no_decision

## Phase-2 evidence status (2026-07-01)

All five phase-2 tracks now have evidence cards:

- Numeric semantics: N001-N006 (workflow-confirmed)
- Arrays/layout/vectorization: A001-A010 (direct extraction)
- Dispatch/generics/runtime: D001-D010 (direct extraction)
- Concurrency/memory models: C001-C006 (direct extraction)
- Dynamic JIT/deoptimization: J001-J005 (direct extraction)
- Seed corpus: F001-F009 (workflow-confirmed)

Direct-extraction cards have NOT been adversarially verified; debates using them must either treat them as provisional or trigger verification first.

The corpus is now broad enough to begin the debate phase, with the caveat that C++/Java memory-model claims rest partly on secondary sources, and HotSpot/Swift/C++-standard primary texts are still missing.
