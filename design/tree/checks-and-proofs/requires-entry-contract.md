# Requires and entry contract

Scope: the contract block of every non-entry function

Decision: Every non-entry function may carry one erased contract block holding shared define bindings followed by independent requires and ensures clauses, and a present block holds at least one clause, because separate pseudo-runtime blocks duplicated definitions, hid their erased status, and forced an unnamed result convention, instead of a single final requirement block.

Decision: A define is pure, total, non-consuming proof syntax alpha-expanded into each clause with no runtime evaluation, storage, snapshot, or ordering edge, because one definition cannot denote a single runtime value across the caller pre-transfer state and the callee return state, instead of a snapshot slot.

Decision: Every function result is named in its signature as a symbolic whole-result datum, and an `Ok(value: binder)` route selects the success payload when a Result postcondition needs it, because a whole-result name cannot replace the payload route without losing the narrow integer relation carrier, instead of an unnamed result or a payload-only convention.

Decision: Requirements are independent goals judged in the same caller pre-transfer state with none serving as a premise for another, and exact goal identity preserves operation semantics, written type and const arguments, operand order, parameter ordinals and projections, constant identity, typed literals, and expanded definitions while ignoring local spelling and sharing, because a goal must be the same goal wherever it is stated, instead of syntactic or normalized goal identity.

Decision: A contradictory entry state is legal and denotes an uninhabited concrete instance, for which the compiler still performs structural, ownership, effect, route, and return-shape checks, lowers an ABI-shaped unreachable stub, and publishes no summary, because the owner selected semantic clarity over migration compatibility for the contract surface, instead of rejecting the instance as a source error.

Rejected:
- A single final requirement block with copy-only local bindings and one Boolean check: rejected because it duplicated definitions across requires and ensures, hid their erased status, and forced an unnamed result.
- Recognizer-driven elision, one recognizer matching the base64 capacity shape: rejected because correct on its domain but it reported a verdict instead of the first missing fact and first failed premise, and unresolved accounting could not fail closed.
