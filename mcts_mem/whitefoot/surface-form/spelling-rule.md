- A surface element exists only if it carries a decision the checker cannot uniquely reconstruct from the remaining bytes of the same declaration; a uniquely reconstructible element is deleted.
- Redundancy that restates a derivable fact is retained exactly at trust boundaries — signatures, contracts, effect rows, and cross-declaration names — and deleted inside a body.
- Any relief preserves one program to one byte spelling: parse and reprint are identity and no accepted program has a second spelling.
- The legality of a spelling depends only on its grammar class — the construct kind, the operation identity, declaration versus body — never on use-site context and never on whether inference succeeds at that site; relief is all-or-nothing per class, and a class that cannot be relieved wholly stays uniformly mandatory.
- Every position is mandatory or forbidden; no element is optional.
- Selection among surviving isomorphic candidates uses measurable quantities only — token counts, grammar rule-count delta, lookahead preservation, and the simplicity of the uniqueness argument.

## Facts

- 2026-08-07 rationale: the rule's admissible bases exclude both aesthetics and what a writer model happens to emit; model behaviour is a motivation for looking at a class, never a criterion for deciding it. (sourced)
- 2026-08-07 (817a8a7c) statement: the rule's first application deleted the index element type argument — reconstructible from the base place — while `cvt`, `reinterpret`, `array_new`, `arena_new`, `finf`, and `fnan` keep theirs at every site, because a type-choosing operation cannot derive one; the retained class was later certified total against the complete operation table. (code)
- 2026-08-07 statement: a class deletion is mechanically migratable — the canonical spelling of an existing program is computable from its tree — while any relief requiring a human to choose a new spelling is evidence that the change carries semantics rather than spelling. (sourced)

## Moves

- 2026-08-07 replaced [[positional-relief]]: legality must be decidable from the grammar class alone, because a rule keyed on whether inference succeeds at a site forces the writer to simulate the checker to know what is even writable, and grows the specification by one conditional clause per relieved position (sourced)
