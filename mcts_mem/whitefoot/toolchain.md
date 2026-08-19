- Build one serious research compiler in safe Rust, with private interfaces that may evolve as language experiments demand.
- Advance through general end-to-end capabilities: frontend, resolution, checking, simple IR, LLVM, execution, then measured optimization.
- Preserve compiler-independent conformance data as evidence alongside the production compiler.
- Written claims remain executable in facts-off compilation and no optimizer fact may remove them; every other hazardous condition must discharge before lowering.
- Treat artifacts, replay, stable protocols, release machinery, and product-scale resource controls as optional later hardening, not compiler prerequisites.
- Keep runtime performance measurements in research experiments; keep deterministic execution, claim-retention, and static-obligation regressions near the backend.

## Facts

- 2026-07-22 owner correction: the target is a general, evolvable research compiler able to compile nontrivial programs for semantic and performance experiments, not an untrusted-input service or stable LLVM-scale product. (sourced)
- 2026-07-22 implementation: exact v0.10 is active and the lexer, terminal classifier, strong-LL(2) parser, finalized source-bound tree, and FORM-2 audit pass in one safe-Rust crate. (code)
- 2026-07-22 repository finding: compiler-independent conformance expectations and the focused ownership model remain useful, while the dormant codegen runner targets retired compiler interfaces. (code)
- 2026-07-22 roadmap: direct general name resolution is the next capability, followed by the first coherent semantic slice through LLVM. (sourced)
- 2026-07-22 implementation: exact owner-approved v0.11 is active; the frontend, direct resolver, conformance identities and source, and focused reference model use `propagate` for Result forwarding, while `try` is an ordinary IDENT. (code)
- 2026-07-22 (dc7fea6b) implementation: the inactive exact-v0.8/v0.9 static specification catalogs moved under `archive/`; no active compiler, build, test, or tool imports them. (code)
- 2026-07-22 implementation: the first v0.11 scalar capability now runs through semantic checking, one checked authority, target-independent IR, conservative LLVM, and `whitefootc`; independent corpus programs execute and required integer-overflow and explicit-check traps retain exact attribution. (code)
- 2026-07-22 implementation: exact owner-approved v0.12 is active, with compiler and conformance identities bound to its hash. SET-1 copy updates for current local-own places use the same checked-program, typed IR, and LLVM path; unsupported index and borrow targets do not create a second path. (code)
- 2026-07-22 implementation: resolved `loop`/`break` now reuse the compiler's existing checked control flow and typed block parameters; iterative scalar and tag-only-enum programs execute through LLVM while FN-1, OWN-11, and explicit edge cleanup remain enforced. (code)
- 2026-07-22 implementation: exact owner-approved v0.13 is active, with compiler and conformance identities bound to its hash. Result propagation's contextual affine consumption reuses the normal semantic, checked-program, typed IR, and LLVM path; checked division/remainder is the next closed Result-producing family. (code)

- 2026-08-07 (7f68c71f) pitfall: the committed LL(2) select table had carried 70 predicate pairs the grammar cannot derive and 57 rows naming the wrong provenance node since a hand patch a version earlier, and nothing detected it for a version. The detection asymmetry is the lesson: a MISSING row has two detectors (the corpus fails to parse, and the arm-coverage check), while a SPURIOUS row has none — the verifier counts productions, decisions, and terminal predicates and none of them moves when rows are added, and no valid program ever reaches the impossible lookahead a spurious row answers. The tables are now derived from the specification grammar and the derivation is gated. (code)

## Moves

- 2026-07-22 (ed9e3db4) replaced [[product-scale-checked-artifact-toolchain]]: mandatory checked artifacts, replay, capability overlays, release gates, and whole-compiler resource profiles delayed the first executable compiler without serving the current research goal; proportional independent tests and direct ordinary compiler structures retain the useful correctness constraints (sourced)
- 2026-07-22 dropped: the active Python reference-model gate: it consumed a historical toy AST and did not exercise or compare with the Rust compiler, so it did not justify active workflow and maintenance cost (sourced)
