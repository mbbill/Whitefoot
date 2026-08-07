- Every unproved bounds obligation carried a runtime check emitted in every build mode; the check was implicit in the indexing operation and the writer stated nothing.
- A retained check's failure trapped with a report, check removal required a deterministic machine-verified proof, and no writer-reachable syntax removed or weakened one.
- The set of retained checks was visible only in the emitted program, never in the source, the signature, or the effect row, which carried one saturating `traps` bit.

## Facts

- 2026-08-06 rationale: the mechanism's wall was that a caller could not tell from a signature or a call site when a callee would trap, and the `traps` bit saturates in any large program, so the trap surface was neither enumerable nor auditable and every unproved obligation stayed invisible until it fired. (sourced)

## Moves

- 2026-08-07 (1032eb63) replaced by [[obligation-discharge]]: an implicit retained check leaves the trap surface unstated and unauditable, so every unproved obligation must now either derive from stated facts, be carried by a named claim, or reject the program (sourced)
