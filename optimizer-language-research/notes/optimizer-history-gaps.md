# Optimizer-history gaps

The seed corpus is not yet a historical survey. Before design decisions, gather primary sources and retrospectives for:

- C strict aliasing, provenance, undefined behavior, and frontend metadata mistakes.
- Fortran array and aliasing evolution, including POINTER/TARGET and parallel features.
- Lisp/Self/JVM/JS dynamic optimization, inline caches, speculative guards, and deoptimization.
- ML/Haskell/Clean/Futhark purity, effects, laziness/strictness, uniqueness, and fusion.
- APL/SQL/Halide-style declarative optimization and scheduling separation.
- GPU/SPMD address spaces, vector lanes, divergence, and memory hierarchy.
- LLVM poison/undef/freeze and historical miscompilation lessons.

Each item needs exact sources, version dates, and at least one adversarial verifier before entering design debate as evidence.
