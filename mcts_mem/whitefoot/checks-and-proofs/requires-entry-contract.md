- Every non-entry function may carry one optional erased `contract` block containing shared `define` bindings followed by plural independent `requires` and `ensures` clauses. A present block must contain at least one proof clause.
- A definition is pure, total, non-consuming proof syntax and is recursively alpha-expanded into each clause. It creates no runtime evaluation, storage, snapshot, ordering edge, or fact; it cannot refer to the symbolic result.
- Each requirement forms its own finite typed GoalTemplate. All requirements are judged independently in the same caller pre-transfer state, none serves as a premise for another, and all successful goals enter the callee body through S4.
- Every function result is explicitly named in its signature. The name is an FN-9 symbolic whole-result datum, not a storage slot; an `Ok(value: binder)` route selects the success payload when a Result postcondition needs it.
- Each postcondition forms its own RelationTemplate and is proved at every selected return in complete, unasserted, and S4-blinded views. Clauses cannot prove one another; all summaries for a successful concrete SCC publish atomically only after every ordinary, provenance, and strict judgment succeeds.
- Exact goal identity preserves selected operation semantics, concrete written type and const arguments, operand order, formal parameter ordinals and projections, named-constant identity, typed literals, and recursively expanded definitions while ignoring local spelling and sharing.
- A contradictory S4 entry state is legal and denotes an uninhabited concrete function instance. The compiler still performs structural, ownership, effect, route, and return-shape checks, retains the source graph for audit, lowers an ABI-shaped `unreachable` stub instead of the body, and publishes no postcondition summary.
- [[requirement-enforcement]] governs the only execution boundary: internal ordinary calls.

## Facts

- 2026-08-19 (55a75434) statement: the owner selected semantic clarity over migration compatibility — shared proof definitions, one contract block, mandatory named results, plural independent clauses, legal uninhabited instances, and no executable entry requirement are one approved language direction. (sourced)
- 2026-08-19 rationale: one definition cannot mean a single runtime value across caller pre-transfer and callee return-edge states. Erased alpha expansion preserves both existing proof images without inventing a snapshot or a hidden runtime slot. (sourced)
- 2026-08-19 rationale: a whole-result name cannot replace the existing `Result.Ok` payload route without losing the narrow integer relation carrier, so the route remains an explicit per-clause symbolic selector. (sourced)

## Moves

- 2026-08-19 (55a75434) replaced [[single-final-requirement-block]]: separate pseudo-runtime requires and ensures blocks duplicate definitions, hide their erased proof status, and force an unnamed result convention; one contract block with erased definitions, plural independent clauses, and a named symbolic result states the same proof boundary without runtime-looking syntax (sourced)
