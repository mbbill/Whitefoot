Node: compiler/checker-facts

Decision: A clause place admits a subscript below its trailing measure and refuses one anywhere else, because x1's [ENT-2] clause (b) and [MSR-1] form an admitted measure place "with any number of field-selection and enum-payload `psuffix`es, `deref` wrappings, and subscripts", which is what makes `deref(rows)[i].len` a term, while a clause still names no element value, instead of refusing every subscript in a clause, which rejected a written requirement the term language admits and reported it as the writer's error.

Decision: The clause goal builder does not yet form the subscripted measure place, so such a clause reports a compiler capability stop rather than a source rejection, because a subscripted goal projection carries a captured offset whose identity the clause template has no occurrence to mint, and [ENT-2] keys a term by spelling while a binding-valued capture is keyed by occurrence, so the two identities have to be reconciled before the projection can be formed, instead of leaving the refusal as [FN-8]'s `InvalidRequires`, which stated that the source was wrong when the specification admits it.

Rejected:
- Minting a fresh capture per clause occurrence and letting two writings of `rows[i].len` be two terms: rejected because [ENT-2] states that two places are the same term exactly when their roots resolve to the same declaration event and their canonical spellings are byte-identical, so a per-occurrence identity would make a clause unable to relate to itself.
- Treating a binding-valued clause offset as the opaque unknown offset: rejected because [MSR-1] admits "a live `own` fragment-integer place" as an offset precisely so that the place's identity is decided over it, and the opaque offset decides nothing.
