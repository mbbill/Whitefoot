Node: language/checks-and-proofs

Decision: Required partial-operation domains are established only by the specification's deterministic proof system, whose facts come from admitted types and declarations, selected control-flow edges, verified contracts, and checked invariants, and whole-invocation completion requires separate checked progress for loops and callees rather than following from an in-place update callable's ordinary no-failure-exit signature, because the [bounded-execution proposal](../../research/investigations/fixed-resource-execution/DESIGN.md) needs completion evidence without changing the existing sources of arithmetic facts or the atomic update contract, instead of writer assertions or deriving termination from a result type.

Rejected:
- The replaced decision's statement that termination remains unchecked for every function: rejected because optional checked progress is necessary for the fixed-resource invocation objective, while ordinary unannotated functions still receive no termination promise.
