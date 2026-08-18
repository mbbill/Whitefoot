- `replace` is a let-RHS-only form: the replacement value enters and the old value exits into the fresh binding in one atomic operation; no program point observes a vacant place (SET-2).
- The old value's sole owner is the new binding; its disposition is the ordinary binding lifecycle — moved onward or released by the ordinary scope-exit drop. No implicit destruction exists; the binder is grammatically mandatory and no bare `replace p = e;` statement form exists.
- The target is a writable region-free affine final place; target formation, writability, loan judgment, and revalidation reuse the copy-replacement judgments, and the binder is an ordinary binding. Replacement through a `&uniq` holder is the sole admitted move of content reached through a borrow.
- Element-level vacancy is a value, not a checker state: an Option-shaped element written by the vacancy operation is an ordinary enum checked by ordinary match, with no per-place flow analysis.
- Whole-binding replace of slice- and arena-typed places is rejected to protect static origin sets and confinement; a future need for slice rebinding is a separate origin-set-join design, not a relaxation.

## Facts

- 2026-08-18 rationale: the let-RHS shape is derived from constraints, not taste — the mandatory old-value binder is forced by no-implicit-destruction, and the no-hole property holds by construction because the replacement value is required in the same operation. (sourced)
- 2026-08-18 (eb8e8634) statement: the three rejected shapes were weighed in the take/replace investigation and were never live in the code; their failure modes are recorded verbatim in `research/investigations/take-replace/DESIGN.md` §2. (sourced)

## Moves

- 2026-08-18 (eb8e8634) replaced [[typed-hole]]: per-place flow-sensitive type states are exactly what the D1a simplification levers exclude, and vacancy would leak into every boundary signature; the Option-shaped value returns legitimately as element vacancy with no checker extension (sourced)
- 2026-08-18 (eb8e8634) replaced [[swap-only]]: swap changes a live affine binding's value without death or a new let — a third mutation path breaking initialization-keyed facts, loans, and liveness — and everything it expresses is one atomic replace with the moved binding as replacement (sourced)
- 2026-08-18 (eb8e8634) replaced [[closed-scope-hole]]: a hole open across statements needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window, and a meaning for `&uniq` over a vacant referent, buying only a use-then-refill window neither consumer needs (sourced)
