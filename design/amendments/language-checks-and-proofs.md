Node: language/checks-and-proofs

Decision: Automatic derivation uses fixed terminating families that run to completion, and a harder proof is a finite explicit certificate checked without rediscovery, because acceptance must not depend on solver state, time, machine speed or a work budget, instead of SMT-backed acceptance.

Decision: Required partial-operation domains are established by the specification's deterministic proof system from admitted types and declarations, selected control-flow edges, verified contracts and checked invariants, because an unproved domain cannot authorize a partial operation, instead of writer assertions or runtime traps.

Decision: A nonempty explicit certificate is rejected when automatic derivation already proves its target, because the boundary between automatic and written proof must be decidable from the language rules, instead of tolerating redundant certificates.

Decision: A proof_use certificate holds at most 4096 entries and one invariant's affine formation at most 4096 scheduled expression nodes, input terms and result terms, because the specification must fix these structural source ceilings independently of machine capacity; the chosen number is arbitrary, instead of a time or work budget or no structural ceiling.

Decision: The fact language carries no quantified storage-element or per-slot occupancy facts, including at control-flow joins, because data-dependent occupancy can be stored as ordinary tags or options and checked through a bounded index, instead of element invariants that must be instantiated at each read and re-established at each write.

Decision: A place reached through one or more subscripts is a term of the fact language exactly when its last step selects a readonly field, so the length of one storage held inside another is a term while an ordinary integer field of an element is not, because the readonly fields are the quantities the storage operations publish and that every bounds proof is stated over, and admitting them below a subscript is what makes a storage whose elements are themselves storages provable at all, while admitting every integer place below a subscript would widen the set of terms the fixed families close over before anything has measured what that closure then costs, instead of admitting every subscripted integer place as a term, or keeping subscripts out of terms entirely and leaving a nested storage unprovable.

Decision: Required index and range separation is discharged by the ordinary fixed automatic families and written proof steps, with an undischarged pair remaining overlapping under the [shared path judgment](https://github.com/mbbill/Whitefoot/blob/main/design/language/ownership.md), because access and parallel permission must consume the same proved distinctness, instead of a separate alias solver or checker-dependent separation.

Rejected:
- Tolerating an unused explicit certificate as advice rather than an error: rejected because the redundancy rule ties the writer's choice to the exact language version instead of to compiler behavior.
- Limiting the explicit certificate language to what the automatic families can already prove: rejected because a certificate that can only restate an automatic proof adds no proof, so the certificate language must reach beyond the automatic families.
- Quantified invariants over the elements of a storage: rejected because every write into the storage would owe the quantified fact again, and establishing it would require a derivation over elements that no terminating fixed family performs.
