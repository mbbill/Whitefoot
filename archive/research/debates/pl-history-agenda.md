# Programming-language history debate agenda

Frame debates as historical alternatives, not winner selection. Compare:

- C/Fortran-style semantic promises.
- ML/Haskell-style purity, strictness/laziness, effects, and fusion.
- Self/JVM/JS-style speculative optimization, guards, and deoptimization.
- APL/SQL/Halide/Futhark-style declarative array/query optimization.
- Rust/SPARK/Pony-style ownership, proof, and capability systems.
- GPU/SPMD/data-parallel address-space and scheduling models.

For each proposal identify whether it is a source rule, library convention, verifier obligation, IR annotation, runtime/JIT mechanism, ABI representation, or empirical performance claim.

Force comparison against historical failures: strict-aliasing surprises, UB/provenance traps, LLVM poison/undef issues, Java checked-exception ergonomics, JIT warmup/deopt cliffs, Fortran pointer/TARGET complications, GPU portability traps, and unsound frontend lowering to optimizer metadata.
