# Phase-2 arrays, layout, and vectorization findings

Direct source-specific extraction after the broad workflow stalled. These are evidence cards, not design decisions. They need later adversarial debate and deeper source notes.

| ID | Theme | Claim summary | Sources | Scope limit |
|---|---|---|---|---|
| A001 | structured memory representation | MLIR MemRef exposes array memory metadata—shape, strides, offsets, memory spaces, allocation, and view-like transformations—in optimizer-visible IR, while also providing explicit no-alias... | https://mlir.llvm.org/docs/Dialects/MemRef/ | Does not decide source syntax or whether source-level UB is acceptable. Metadata/view operations can alias original storage and need verifier/runtime obligations. |
| A002 | structured loop/data semantics | MLIR Linalg keeps structured array computation as operations whose operands define iteration spaces, whose indexing maps relate loops to data, and whose iterator kinds expose parallel/red... | https://mlir.llvm.org/docs/Dialects/Linalg/ | Docs are design/implementation rationale, not proof of profitability or final language design. |
| A003 | affine restrictions and dependence analysis | MLIR Affine restricts loop bounds and memory accesses to affine forms so dependence analysis and loop transformations can be efficient and reliable; affine.parallel and reduction forms en... | https://mlir.llvm.org/docs/Dialects/Affine/ | Only applies where programs fit affine restrictions; semi-affine or arbitrary indexing weakens applicability. |
| A004 | vector-level IR | MLIR Vector represents n-dimensional virtual vectors as structured SSA values with progressive lowering and transfer operations, preserving vector/memory movement semantics above scalar l... | https://mlir.llvm.org/docs/Dialects/Vector/ | Target legality, register pressure, LLVM 1-D vector limits, scalable-vector constraints, and hardware shapes remain constraints. |
| A005 | LLVM vectorization legality/profitability | LLVM Loop and SLP vectorizers depend on legality and profitability information: alias/dependence facts, reductions/inductions, trip counts, FP math flags, control-flow shape, runtime poin... | https://llvm.org/docs/Vectorizers.html | Vectorization is a conditional optimization, not a guarantee; pragmas are hints and runtime checks/fallbacks may be needed. |
| A006 | schedule as explicit optimization layer | Halide demonstrates an explicit schedule layer where loop order, split/fuse, tiling, vectorization, unrolling, and parallelization are separated from the pure algorithm, exposing optimiza... | https://halide-lang.org/tutorials/tutorial_lesson_05_scheduling_1.html | Halide is a DSL with purity/update constraints; benefits are workload-dependent and not automatically transferable to a general language. |
| A007 | SPMD and lane-aware semantics | ISPC exposes SIMD execution through an SPMD gang model, uniform/varying distinctions, foreach lane-wise iteration, execution masks for divergence, and target-dependent gang/mask sizes, ma... | https://ispc.github.io/ispc.html | Performance depends on target, data coherence, memory layout, gather/scatter costs, and guide details need deeper verification. |
| A008 | regular arrays / uniqueness for parallel arrays | Futhark requires regular arrays and tracks sizes in types; its consuming in-place updates and alias rules allow semantically pure array updates to avoid copying when old arrays and aliase... | https://futhark.readthedocs.io/en/stable/language-reference.html | Parallel execution is not guaranteed; exact layout/codegen and fusion behavior require further implementation/paper evidence. |
| A009 | domain-based array semantics | Chapel arrays are defined over domains, first-class index sets; const/named domains and compatible-shape whole-array operations expose shape/index-set information that documentation says ... | https://chapel-lang.org/docs/primers/arrays.html | Primer does not specify exact optimizations, aliasing, or distribution implementation details. |
| A010 | parallel-loop independence with caveats | Fortran DO CONCURRENT expresses arbitrary-order iteration and imposes restrictions on cross-iteration dependences/effects; locality specifiers can communicate variable-level intent, but r... | https://flang.llvm.org/docs/DoConcurrent.html | Not guaranteed to run concurrently; compilers may still need dependence analysis, runtime checks, or serial lowering. |

## Main patterns observed

- Optimizer-friendly array systems preserve more than element type: shape, bounds, strides, indexing maps, iterator roles, memory spaces, layout transforms, and independence/reduction facts.
- Several systems separate algorithm from schedule or preserve high-level loop/data structure for staged lowering.
- Many optimizer-facing facts carry obligations: noalias/distinctness, in-bounds assumptions, regularity, affine restrictions, locality clauses, purity/side-effect constraints, and uniqueness/consumption.
- None of this decides whether a future language should be a DSL, MLIR-first source, Fortran-like array language, SPMD language, or general-purpose language with opt-in kernels.

## Follow-up gaps

- Deeper primary sources/papers for Halide, Futhark fusion, ISPC layout rules, Chapel distributions/locales, and Fortran standard text.
- Empirical/prototype checks: whether facts survive lowering and are consumed by MLIR/LLVM passes.
- Aliasing among slices/views and source-level semantics for invalid contracts.
