Decision: Ownership is single-owner affine values with explicit ordinary moves, lexical named or unnamed regions, explicit borrow modes at mode-bearing positions, and a checker that rejects when it cannot establish safety, because replicating a Rust-class inferring borrow checker is unacceptable implementation effort, a normal compiler frontend is acceptable and rustc-scale inference is not, so the calculus stands on explicit regions and reject-when-unsure, instead of inferred lifetimes, flow-sensitive borrow liveness, and implicit reborrowing.

Decision: Overlap is judged conservatively over complete resolved paths, with distinct fields and unequal literal indices establishing disjointness, because a conservative judgment the checker can decide is preferred to a precise one it would have to infer, instead of flow-sensitive alias analysis.

Rejected:
- Inferred borrow checking as Rust ships it, with elided lifetimes, non-lexical liveness, and implicit reborrows: rejected because replicating rustc's borrow checker is unacceptable implementation effort, so the ownership design stands only on a simplified explicit-region, reject-when-unsure calculus.
