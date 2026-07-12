# Phase-2 numeric semantics findings

Evidence from workflow `w6jgx7vrd`. This is LLVM-heavy and does not decide source-language numeric defaults.

## Summary

The verified evidence is strongest for LLVM IR’s optimizer-facing numeric and undefinedness contracts, not for final source-language policy choices. LLVM gives optimizers explicit semantic levers: immediate UB, poison, undef, freeze, no-wrap flags, range metadata, inbounds GEPs, and proof/test expectations for canonicalization transforms. These contracts make integer and pointer-index optimizations powerful but conditional: unflagged integer add wraps, flagged overflow becomes poison, division by zero is immediate UB, and some GEP-derived addresses may be computed but not validly dereferenced. The surviving evidence does not justify final language-design decisions for overflow defaults, strict versus fast floating point, FMA/contraction, reproducibility, GPU approximate math, or target-specific math; those areas need a follow-up verification pass.

| ID | Claim summary | Confidence | Source count | Scope limit |
|---|---|---:|---:|---|
| N001 | LLVM IR’s undefinedness model is an explicit optimization contract: values are ordered from immediate UB through poison, undef, freeze(poison), and concrete values, an... | high | 4 | This card supports optimizer-facing LLVM numeric/undefinedness contracts, not a final source-language policy. Strict-vs-fast floating point, FMA/contraction, reproducibility, GPU approximate math, and non-LLVM language defaults need follow-up evidence unless directly named in the claim. |
| N002 | LLVM integer semantics separate default arithmetic results from optimization promises: unflagged integer `add` is modulo bit width, while `nuw` and `nsw` make unsigned... | high | 3 | This card supports optimizer-facing LLVM numeric/undefinedness contracts, not a final source-language policy. Strict-vs-fast floating point, FMA/contraction, reproducibility, GPU approximate math, and non-LLVM language defaults need follow-up evidence unless directly named in the claim. |
| N003 | LLVM treats some numeric precondition failures as immediate UB rather than poison; integer division by zero is the clearest verified example and prevents otherwise tem... | high | 3 | This card supports optimizer-facing LLVM numeric/undefinedness contracts, not a final source-language policy. Strict-vs-fast floating point, FMA/contraction, reproducibility, GPU approximate math, and non-LLVM language defaults need follow-up evidence unless directly named in the claim. |
| N004 | `range` metadata is a numeric fact channel for integer loads and call/invoke returns: values outside the declared union of half-open ranges become poison. | high | 2 | This card supports optimizer-facing LLVM numeric/undefinedness contracts, not a final source-language policy. Strict-vs-fast floating point, FMA/contraction, reproducibility, GPU approximate math, and non-LLVM language defaults need follow-up evidence unless directly named in the claim. |
| N005 | LLVM pointer-index arithmetic also carries optimizer contracts: non-`inbounds` GEP may compute out-of-bounds addresses, but `inbounds` makes out-of-object results pois... | high | 3 | This card supports optimizer-facing LLVM numeric/undefinedness contracts, not a final source-language policy. Strict-vs-fast floating point, FMA/contraction, reproducibility, GPU approximate math, and non-LLVM language defaults need follow-up evidence unless directly named in the claim. |
| N006 | InstCombine operationalizes these contracts as target-independent canonicalization: transforms should not depend on target cost modeling, and flag-sensitive folds are ... | medium | 3 | This card supports optimizer-facing LLVM numeric/undefinedness contracts, not a final source-language policy. Strict-vs-fast floating point, FMA/contraction, reproducibility, GPU approximate math, and non-LLVM language defaults need follow-up evidence unless directly named in the claim. |

## Sources

- https://llvm.org/docs/UndefinedBehavior.html
- https://llvm.org/docs/InstCombineContributorGuide.html
- https://llvm.org/docs/GetElementPtr.html
- https://llvm.org/docs/LangRef.html
- https://llvm.org/doxygen/InstCombineAddSub_8cpp_source.html
- https://llvm.org/doxygen/InstructionSimplify_8cpp_source.html
- https://llvm.org/doxygen/classllvm_1_1ScalarEvolution.html
- https://llvm.org/doxygen/LoopVectorize_8cpp_source.html
- https://llvm.org/docs/Vectorizers.html
- https://eel.is/c++draft/expr.pre
- https://doc.rust-lang.org/reference/expressions/operator-expr.html
- https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html
- https://docs.swift.org/swift-book/documentation/the-swift-programming-language/advancedoperators/#Overflow-Operators
- https://ziglang.org/documentation/master/
- https://www.adaic.org/resources/add_content/standards/12rm/html/RM-4-5.html
- https://standards.ieee.org/ieee/754/6210/
- https://docs.oracle.com/javase/specs/jls/se21/html/jls-4.html#jls-4.2.4
- https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.4
- https://eel.is/c++draft/cfenv
- https://llvm.org/docs/LangRef.html#fast-math-flags
- https://llvm.org/docs/LangRef.html#constrained-floating-point-intrinsics
- https://clang.llvm.org/docs/UsersManual.html#controlling-floating-point-behavior
- https://docs.nvidia.com/cuda/cuda-compiler-driver-nvcc/index.html#use-fast-math-use-fast-math
- https://docs.nvidia.com/cuda/floating-point/index.html
- https://raw.githubusercontent.com/KhronosGroup/OpenCL-Docs/main/api/opencl_runtime_layer.asciidoc

## Open questions / limits

- Source-language overflow defaults remain undecided.
- Strict versus fast floating-point defaults remain undecided.
- FMA/contraction, reproducibility, GPU approximate math, and target-specific math need follow-up verification.
- LLVM contracts require frontend proof or explicit unsafe assumptions before being emitted.
