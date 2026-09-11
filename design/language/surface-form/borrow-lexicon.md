Decision: Surface names label checked invariants defined in the specification and never borrow backend vocabulary, because a name like noalias names a lowering consequence rather than the invariant the checker enforces, instead of backend terms at the surface.

Decision: The exclusive borrow mode is spelled `&uniq`, because the mode's invariant is uniqueness and not mutation, and `mut` conflates exclusivity with write permission in a way that breaks under future interior-mutability capabilities, instead of Rust's `&mut`.

Rejected:
- `&mut` following the Rust convention: rejected because the exclusive mode's invariant is uniqueness, not mutation, and `mut` conflates exclusivity with write permission and breaks under future interior-mutability capabilities.
- `noalias` as the surface name: rejected because it is backend vocabulary naming a lowering consequence rather than the checked invariant.
