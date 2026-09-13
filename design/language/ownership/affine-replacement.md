Decision: `replace` is a let-only form in which the replacement value enters and the old value exits into a fresh binding in one atomic operation, so no program point observes a vacant place, because the mandatory old-value binder is forced by no implicit destruction and the no-hole property holds by construction when the replacement is required in the same operation, instead of a take that leaves a hole to be refilled later.

Decision: Element-level vacancy is a value, an Option-shaped element checked by ordinary match, and never a checker state, because per-place flow-sensitive type states are exactly what the simplified ownership calculus excludes and vacancy would otherwise leak into every boundary signature, instead of typed holes tracked across program points.

Decision: Whole-binding replace of slice- and arena-typed places is rejected, because it would break the static origin sets and confinement those types carry, instead of admitting replace on every affine place.

Rejected:
- A bare take that is legal when the checker proves the hole is refilled before scope end: rejected because a hole open across statements needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window, and a meaning for an exclusive borrow over a vacant referent, buying only a use-then-refill window neither consumer needs.
- A two-place `swap`: rejected because it changes a live affine binding's value without death or a new binding, a third mutation path that breaks initialization-keyed facts, loans, and liveness, and everything it expresses is one atomic replace with the moved binding as the replacement.
- Typed holes, flowing a taken slot to an Option-like vacant type state: rejected because per-place flow-sensitive type states are exactly what the simplified calculus excludes, and vacancy would leak into every boundary signature.
