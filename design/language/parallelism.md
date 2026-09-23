Decision: Parallel permission derives from checked ownership, effects, dataflow, control flow and domain proofs; a denied permission leaves an accepted program sequential, because overlap is an implementation liberty that must preserve sequential meaning, instead of a source parallel keyword that changes acceptance.

Decision: Permission and actualization are separate judgments, because a permission is a proved program property while scheduling is a cost choice, instead of making acceptance depend on a lowering or runtime decision.

Decision: A counted loop is a parallel-permission site in its own right, because iteration independence must not depend on rewriting the loop as sibling calls, instead of treating loop permission as a special case of the sibling-call rule.

Rejected:
- Source staged-loop permission selected by suspension or native-operation classifications: rejected because a call's declared effect row covers, for the whole of the call, every place the callee may reach through its arguments, and where an operation happens to be implemented cannot authorize overlap that row does not; the loss of pipeline permission is established, but its separate runtime cost has not been measured.
