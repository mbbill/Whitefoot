Node: language/ownership/affine-replacement

Decision: Exchanging the value held in a place is an ordinary operation that takes a reference to the place and the replacement value and returns the previous value as an owned result, so no program point observes a vacant place, because the replacement is supplied by the same operation that removes the old value, instead of a let-only statement form of its own or a take that leaves a hole to be refilled later.

Decision: A move out of a field or out of the content of a heap cell consumes the whole owner: the owner ceases to exist, its other affine parts are released, and a remaining linear part refuses the move unless it is taken in the same destructuring, because a partially moved owner is a hole under another name, and a whole-owner consume keeps every place a program can name either wholly present or gone, instead of a partial move that leaves the owner alive with one component missing.

Decision: A move out of a slot of a window or an element of an array is refused, the ways out being the removing operations, the place exchange, and the two-place exchange, because those slots are covered by a length or by an always-filled invariant and nothing records that one of them is empty, instead of an element take carrying a later refill obligation.

Decision: Assigning to a place the result of a total function applied to that place's own value is one atomic update with no program point between the read and the commit, admitted when that function neither writes, moves out of, nor releases any prefix of the place, because rebuilding a node in place is then written directly and a failure is expressed as an enum value stored back into the place, instead of moving the value out into a temporary and assigning it back across an observable gap.

Decision: Element-level vacancy is a value, an Option-shaped element checked by an ordinary match, and never a checker state, because per-place flow-sensitive type states are exactly what this ownership model excludes and vacancy would otherwise leak into every boundary signature, instead of typed holes tracked across program points.

Rejected:
- A bare take that is legal when the checker proves the hole is refilled before scope end: rejected because a hole open across statements needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window, and a meaning for a reference whose path reaches a vacant place, buying only a use-then-refill window no consumer needs.
- Typed holes, flowing a taken slot to an Option-like vacant type state: rejected because per-place flow-sensitive type states are exactly what this model excludes, and vacancy would leak into every boundary signature.
- Refusing whole-binding exchange on slice-typed and arena-typed places: rejected because neither is a type any more: a range is a reference formed from indexable storage and an arena is ordinary storage used with an append as allocation, so no value carries the static origin set or the confinement the refusal protected.
