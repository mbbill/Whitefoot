# Array, layout, and iteration dossier

Phase-2 direct source extraction produced evidence cards in `notes/phase2-arrays-findings.jsonl` and an index in `synthesis/phase2-arrays-findings-index.md`.

## Evidence themes

- MLIR MemRef: shape/layout/stride/memory-space metadata, views, distinct_objects noalias contract, allocation/deallocation, lowering caveats.
- MLIR Linalg: structured ops, indexing maps, iterator kinds, tiling/fusion/promotion/vectorization/lowering.
- MLIR Affine: restricted affine loops/accesses for dependence analysis and transformations.
- MLIR Vector: n-D virtual vectors, transfer ops, progressive lowering to target/LLVM forms.
- LLVM Vectorizers: legality/profitability, runtime pointer checks, reductions/inductions, math flags, cost model.
- Halide: algorithm/schedule separation, tiling, vectorization, unrolling, parallelization.
- ISPC: SPMD gang/lane model, uniform/varying, masks/divergence, target-dependent SIMD width.
- Futhark: regular arrays, size types, SOACs, uniqueness/consumption for copy avoidance.
- Chapel: domains as first-class index sets, named/const domains, compatible-shape array operations.
- Fortran DO CONCURRENT: arbitrary-order iteration, locality clauses, restrictions that help but do not guarantee parallel lowering.

## Open design-neutral questions

- Which facts should live in source syntax, type system, contracts, libraries, schedule DSL, or intermediate IR?
- How should invalid facts fail: static rejection, runtime check, trap, unsafe-only promise, conservative fallback, or IR-level UB?
- How much of the future language should privilege affine/regular arrays versus irregular pointer-heavy data structures?
- How to represent views/slices so aliasing remains explicit and vectorization can still happen?
- How to validate that MLIR/LLVM actually consume the preserved facts?
