Decision: A container receives hash, equality, and comparison as explicit function-kind arguments, with formal and actual groups abbreviating their signatures and arguments hygienically, because the owning-map and priority-queue witnesses need reusable behavior without runtime representation or dispatch cost, instead of function values, dictionaries, implicit type-owned implementations, or repeating every member signature at every library layer.

Decision: A formal member's declared effect row is authoritative, an actual's effects must fit its paths by category and prefix, and ordered contracts must match structurally after ordinary substitution, because a generic caller must retain one written boundary while accepting an implementation that touches less state, instead of deriving the boundary from the selected body or using implication search to match contracts.

Decision: Member calls use the instantiated formal signature, row and contracts while the selected actual is checked independently, because a container's written boundary must not vary with which implementation is selected, instead of importing an actual body's smaller row or stronger inferred facts into its callers.

Decision: Supplied behavior carries no assumed equality, comparison, or hashing laws, because hostile behavior must still preserve ownership and proved bounds even when the container holds unexpected contents, instead of using an unchecked law as a safety premise or retaining source law declarations without a selected consumer.

Decision: A formal group's members quantify their own loan regions while an actual may capture store brands in its type arguments, because borrowing at each call and identifying stored values are different relationships already expressed by ordinary function and nominal parameters, instead of a group-wide loan region or implicit captured environments.

Rejected:
- Runtime dictionaries or callable values for container behavior: rejected because the selected map and queue need direct-call specialization without runtime storage or dispatch.
- Checking actual rows for equality with the formal row: rejected because an actual that accesses only a declared subfield fits the caller's contract without dummy reads or a runtime adapter.
- An owned function-formal result required to be fresh: rejected because it depended on the removed history of owned values and excluded ordinary ownership-returning functions from the common callable boundary.
