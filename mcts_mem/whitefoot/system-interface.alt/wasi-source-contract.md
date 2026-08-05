- The source-language system contract is literal WASI 0.2/0.3: its worlds, interfaces, resource handles, string-typed paths, and Component Model async are the language's own system semantics.
- Portability and taxonomy come from the existing WASI ecosystem rather than a Whitefoot-owned model.

## Moves

- 2026-08-05 (8f7055fc) replaced by [[system-interface]]: a literal WASI source contract imports Unicode-only paths, no guaranteed caller-buffer or zero-copy route, async tied to Component Model costs, and an incomplete threads and process surface chosen for cross-language components rather than Whitefoot ownership; WASI remains a possible target implementation for operations it can supply (sourced)
