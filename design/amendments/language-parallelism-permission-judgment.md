Node: language/parallelism/permission-judgment

Decision: Overlap permission is judged on a pair of adjacent statements of one block, and a run of statements may all overlap only when every pair of statements the run contains may overlap, because every statement is judged by the paths it actually reads and writes, so a statement standing between two calls is itself a judged member of the pairs around it rather than a form that must be classified or that ends a candidate group, instead of a window of call members with separately classified interposed statements.

Decision: Each statement's writes must be disjoint from the other's reads and writes under the [ordinary path-overlap judgment](https://github.com/mbbill/Whitefoot/blob/main/design/language/ownership.md), including by-value consumption as defined by the [call-site check](https://github.com/mbbill/Whitefoot/blob/main/design/language/effects/call-site-check.md), because intra-call and inter-statement interference need the same interpretation, instead of a parallel-specific alias or loan model.

Decision: Both statements' paths are interpreted in the state before the first, using its ensures to map the second's indices into that state, because an index available only after an append must not appear distinct from the append slot it names, instead of comparing paths in different entry states.

Decision: An overlap that is not proved disjoint denies permission, and a path the implementation cannot resolve overlaps every path, because a silent empty footprint would grant permission by omission, instead of defaulting an unresolved path or an unrecognized access to no effect.

Rejected:
- Judging only consecutive calls: rejected because an intervening statement must be checked by its own access paths rather than ending permission according to its syntax.
- A separate aliasing model for parallel permission, whether by reliance edges, schedule-parametric ownership, treating exclusive borrows as writes, or a weakened overlap rule: rejected because each one either duplicates the ordinary path-overlap judgment or weakens it.
- Defaulting an unclassified statement or an unresolved path to an empty footprint: rejected because permission would then be granted by omission.
