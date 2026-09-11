Decision: Parallel permission derives from checked ownership, effects, dataflow, control flow, and domain proofs, and a failed permission leaves the program sequential rather than rejecting it, because optional facts never gate legality and profitability is decided at runtime and invisible in code, instead of a parallel keyword that changes acceptance.

Decision: Permission and actualization are separate judgments, because a permission is a fact about the program while actualization is a lowering and runtime choice, and conflating them would make acceptance depend on a scheduling decision, instead of one combined judgment.

Decision: Proof-only contracts and invariants introduce no scheduling edge, lock, or runtime branch, because a proof that changes the schedule is not erased, instead of proof statements that participate in scheduling.

Decision: Automatic discovery of parallelism is not a direction, because the language removes only the soundness half of auto-parallelization and none of the decision half, and granularity rather than legality was the binding constraint across decades of prior systems, instead of an auto-parallelizing compiler.

Rejected:
- Automatic parallelization as a compiler direction: rejected because the language settles legality but not the granularity decision that was the binding constraint in prior systems.
