# Source-to-IR lowering sketches

No syntax decisions yet. Each future sketch should include source-like pseudocode, semantic facts, candidate MLIR, candidate LLVM IR, expected surviving facts, and facts that must not be emitted without proof.

## Sketch backlog

- noalias slices/views
- ownership transfer and escape
- array bounds and range metadata
- loops/reductions/vectorization
- closures and capture allocation
- dispatch and generics
- exceptions/results/effects
- atomics/concurrency
- allocation/runtime/FFI boundary
