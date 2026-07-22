- The active implementation is one safe-Rust crate under `compiler/`; crate boundaries and public protocols are not design goals.
- Exact v0.10 frontend behavior is preserved while implementation machinery is simplified around the next consumer.
- The next consumer is a direct resolver over the canonical syntax tree, using ordinary owned records and deterministic maps.
- Retired compiler and derivation scripts are inert under `archive/`; active source and gates never import them.
- A small native grammar-change verifier must reuse compiler lexer/parser behavior before a future numbered specification is proposed. It is a spec-development tool, not part of routine compilation.

## Facts

- 2026-07-22 code: six frontend crates, two hash/catalog identities, a source-audit crate, and version-forked Python table scripts were consolidated into one dependency-free Rust crate; the inherited v0.10 frontend suite passes. (code)
- 2026-07-22 code: exact active specification bytes are checked against the approved candidate and terminal/grammar data share the approved SHA-256 identity. (code)
- 2026-07-22 roadmap constraint: the version switch reproduces the frontend without starting semantic architecture; resolution follows as a separate phase. (sourced)

## Moves

- 2026-07-22 replaced [[permanent-artifact-compiler]]: starting with a permanent checked-artifact architecture treated private research-compiler boundaries as product protocols and multiplied crates and gates before a resolver or backend existed; one mutable compiler crate better serves the next end-to-end capability (sourced)
