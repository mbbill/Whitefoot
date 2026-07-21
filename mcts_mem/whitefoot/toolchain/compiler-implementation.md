- No source-to-code compiler is active yet. Exact v0.9 is the immutable active target, and its reviewed grammar and frontend-boundary rules authorize the canonical-frontend implementation phase.
- The Rust implementation starts directly with the permanent checked-artifact architecture; there is no disposable compiler or parallel implementation ladder.
- The production workspace forbids Rust `unsafe`. The declared trusted computing base includes the pinned Rust toolchain and dependencies, LLVM and native tools, the runtime and allocator, the operating system, and foreign call frames.
- Retired wfc and democ snapshots are inert historical evidence. Neither implementation defines language semantics, lowering behavior, build success, or release readiness.
- No active build or source import crosses from the archive. Valuable general regressions are re-derived additively against permanent interfaces rather than transplanted with retired architecture.
- Self-hosting is outside current authorization. One production compiler moves until exact-v0.9 completion and later product qualification justify a new owner decision.
- The standalone grammar-change verifier is complete and remains outside the production compiler dependency graph. The active implementation tranche is the exact-v0.9 canonical frontend; verifier reports remain evidence rather than language authority.

## Facts

- 2026-07-20 (75b768ba) selection: the exact implementation target is `spec/kernel-spec-v0.8.md`, SHA-256 `d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`; its provisional and deferred markings remain unchanged, and any semantic delta requires a new numbered specification. (sourced)
- 2026-07-20 (c5ef95a4) measurement: retired wfc contained 655 functions, with 166 classified clean, 489 unsupported, and 15 emitted; it had not reached compilation, stage 1, or a byte-identical fixpoint. (code)
- 2026-07-20 (c5ef95a4) source-contract finding: the wfc unit omitted 4,568 explicit TYPE-5 region arguments and had no FN-7 entry point; stage 0 enforced only an at-most-one-main condition. (sourced)
- 2026-07-20 specification constraint: kernel specification v0.8 has no growable collection, keyed collection, byte-string, or text facility. (sourced)
- 2026-07-20 implementation observation: archived wfc uses fixed-capacity structure-of-arrays tapes and manual byte handling throughout the compiler. (code)
- 2026-07-20 measurement: a safe-Rust typed-index `Vec` experiment reached the measured AST storage ceiling, while a safe bump representation remained within 21 percent of its time; mature host-language storage can satisfy the production compiler's measured structural needs without an unsafe arena. (sourced)
- 2026-07-20 implementation boundary: the conformance manifest and cases are compiler-independent and can be exercised through named implementation adapters without changing normative expectations. (sourced)
- 2026-07-20 (75b768ba) repository inspection: no Rust compiler implementation existed when the production-Rust direction was selected. (code)
- 2026-07-21 (4ecc14d) owner ruling: all top-level function signatures are visible throughout the future closed compilation unit, while locals, regions, labels, and named constants remain declaration-before-use; implementation waits for exact successor-specification encoding. (sourced)
- 2026-07-21 owner-approved target switch: `spec/kernel-spec-v0.9.md`, SHA-256 `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`, is the exact production target. It encodes the closed-unit function-visibility ruling and closes the parser entrance boundary; later semantic questions, profiles, artifacts, backends, and release authority remain separately gated. (sourced)

## Moves

- 2026-07-20 (75b768ba) replaced [[self-host-first-wfc-ladder]]: the self-host-first ladder coupled production semantic progress to repairing stage 0 and a nonconforming, incomplete compiler unit under a language without ordinary compiler-building collections or text; one safe-Rust production implementation preserves the specification-derived checked-artifact architecture while freezing both predecessors (sourced)
