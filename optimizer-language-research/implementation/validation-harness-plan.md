# Validation harness plan

Future design candidates should be checked empirically/mechanistically against LLVM/MLIR rather than accepted from literature alone.

Minimum harness:

- Small kernels with expected source facts.
- MLIR output checks with `mlir-opt` / `mlir-translate` where applicable.
- LLVM IR checks with `opt`, `llc`, FileCheck, optimization remarks.
- Negative tests proving invalid contracts are rejected, guarded, trapped, or compiled conservatively.
- Microbenchmarks only after semantic correctness is established.

Do not promote a feature to design_candidate without at least one positive and one negative validation sketch.
