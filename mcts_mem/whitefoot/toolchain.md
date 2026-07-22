- Build one serious research compiler in safe Rust, with private interfaces that may evolve as language experiments demand.
- Advance through general end-to-end capabilities: frontend, resolution, checking, simple IR, LLVM, execution, then measured optimization.
- Preserve compiler-independent conformance data and focused reference models where they catch a distinct class of error; do not construct a second production compiler.
- Required runtime checks remain in facts-off compilation. An optional fact may remove a check only after focused proof and negative-canary testing.
- Treat artifacts, replay, stable protocols, release machinery, and product-scale resource controls as optional later hardening, not compiler prerequisites.
- Keep runtime performance measurements in research experiments; keep deterministic execution, check-retention, and proof-elision regressions near the backend.

## Facts

- 2026-07-22 owner correction: the target is a general, evolvable research compiler able to compile nontrivial programs for semantic and performance experiments, not an untrusted-input service or stable LLVM-scale product. (sourced)
- 2026-07-22 implementation: exact v0.10 is active and the lexer, terminal classifier, strong-LL(2) parser, finalized source-bound tree, and FORM-2 audit pass in one safe-Rust crate. (code)
- 2026-07-22 repository finding: compiler-independent conformance expectations and the focused ownership model remain useful, while the dormant codegen runner targets retired compiler interfaces. (code)
- 2026-07-22 roadmap: direct general name resolution is the next capability, followed by the first coherent semantic slice through LLVM. (sourced)

## Moves

- 2026-07-22 replaced [[product-scale-checked-artifact-toolchain]]: mandatory checked artifacts, replay, capability overlays, release gates, and whole-compiler resource profiles delayed the first executable compiler without serving the current research goal; proportional independent tests and direct ordinary compiler structures retain the useful correctness constraints (sourced)
