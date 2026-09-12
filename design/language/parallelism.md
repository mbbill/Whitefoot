Decision: Parallel permission derives from checked ownership, effects, dataflow, control flow, and domain proofs, and a failed permission leaves the program sequential rather than rejecting it, because parallelism is a property the checker derives from what it already proves, while whether to use it is a cost choice that must not change which programs are accepted, instead of a parallel keyword that changes acceptance.

Decision: Permission and actualization are separate judgments, because a permission is a fact about the program while actualization is a lowering and runtime choice, and conflating them would make acceptance depend on a scheduling decision, instead of one combined judgment.

Decision: A counted loop is a parallel-permission site in its own right, because its permission must not depend on rewriting the loop as a pair of sibling calls, instead of loop permission as an amendment to the sibling-call rule.
