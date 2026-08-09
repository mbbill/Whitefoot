- A body binder's mode and type are derived from its right-hand side, never written; the derivation is statement-local and reads no later statement, expected type, or use site.
- Every right-hand side is self-typed: operands are typed atoms, calls are typed by their signatures, literals carry mandatory suffixes, and a construction names its nominal.
- A conditional initializer's type is the common type its deliveries agree on; deliveries that disagree are rejected, and an empty delivery set is rejected at the binder.
- Redundant mode and type remain mandatory at every trust boundary — signatures, effect rows, regions, construction field names, match binders, call argument names.

## Facts

- 2026-08-09 (a01bc707) pitfall: the written form had one power the derived form does not, found by a corpus sweep rather than by reading the rule. An annotation could legally name a REGION its right-hand side did not, stating a destination the right-hand side satisfies by outlives rather than equals; a derived type is always the region the right-hand side itself produces. Measured at one site in 1954 annotated bindings. (code)
- 2026-08-09 (a01bc707) statement: whether that removal rejects any program is not established — a borrow at an enclosing region satisfies an inner destination by the same outlives judgment, so the equivalent program may always be writable. It is recorded as an expressible form removed rather than as a narrowing. (sourced)

## Moves

- 2026-08-09 (a01bc707) replaced [[written-mode-and-type]]: a body binder's mode and type are uniquely reconstructed from its right-hand side, so the spelling rule deletes the written form by class; the redundancy that survives is exactly the redundancy at a trust boundary, where a reader cannot reconstruct it from the same declaration (sourced)
