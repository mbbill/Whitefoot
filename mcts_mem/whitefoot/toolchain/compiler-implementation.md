- No source-to-code compiler is active during the foundation phase. The sole moving implementation is a safe-Rust production compiler for the exact normative content of kernel specification v0.8, SHA-256 `d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.
- The Rust implementation starts directly with the permanent checked-artifact architecture; there is no disposable compiler or parallel implementation ladder.
- The production workspace forbids Rust `unsafe`. The declared trusted computing base includes the pinned Rust toolchain and dependencies, LLVM and native tools, the runtime and allocator, the operating system, and foreign call frames.
- Retired wfc and democ snapshots are inert historical evidence. Neither implementation defines language semantics, lowering behavior, build success, or release readiness.
- No active build or source import crosses from the archive. Valuable general regressions are re-derived additively against permanent interfaces rather than transplanted with retired architecture.
- Self-hosting is outside current authorization. One production compiler moves until exact-v0.8 completion and later product qualification justify a new owner decision.

## Facts

- 2026-07-20 (75b768ba) selection: the exact implementation target is `spec/kernel-spec-v0.8.md`, SHA-256 `d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`; its provisional and deferred markings remain unchanged, and any semantic delta requires a new numbered specification. (sourced)
- 2026-07-20 (c5ef95a4) measurement: retired wfc contained 655 functions, with 166 classified clean, 489 unsupported, and 15 emitted; it had not reached compilation, stage 1, or a byte-identical fixpoint. (code)
- 2026-07-20 (c5ef95a4) source-contract finding: the wfc unit omitted 4,568 explicit TYPE-5 region arguments and had no FN-7 entry point; stage 0 enforced only an at-most-one-main condition. (sourced)
- 2026-07-20 specification constraint: kernel specification v0.8 has no growable collection, keyed collection, byte-string, or text facility. (sourced)
- 2026-07-20 implementation observation: archived wfc uses fixed-capacity structure-of-arrays tapes and manual byte handling throughout the compiler. (code)
- 2026-07-20 measurement: a safe-Rust typed-index `Vec` experiment reached the measured AST storage ceiling, while a safe bump representation remained within 21 percent of its time; mature host-language storage can satisfy the production compiler's measured structural needs without an unsafe arena. (sourced)
- 2026-07-20 implementation boundary: the conformance manifest and cases are compiler-independent and can be exercised through named implementation adapters without changing normative expectations. (sourced)
- 2026-07-20 (75b768ba) repository inspection: no Rust compiler implementation existed when the production-Rust direction was selected. (code)

## Moves

- 2026-07-20 (75b768ba) replaced [[self-host-first-wfc-ladder]]: the self-host-first ladder coupled production semantic progress to repairing stage 0 and a nonconforming, incomplete compiler unit under a language without ordinary compiler-building collections or text; one safe-Rust production implementation preserves the specification-derived checked-artifact architecture while freezing both predecessors (sourced)
