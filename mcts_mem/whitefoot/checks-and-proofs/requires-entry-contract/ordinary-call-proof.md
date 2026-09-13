- Each substituted requirement of an ordinary call is proved independently in one shared state before ownership transfer, borrow commits and effect kills.
- Resolution, concrete instantiation, actual-expression obligations and borrow feasibility complete before those proofs. A refuted or unproved requirement rejects at the call.
- Source and linked definitions have the same requirement boundary. Neither a native implementation nor process entry introduces a fallback runtime check or callee contract prologue.
- A callable unit may retain a function that no selected build launcher can invoke. Its source acceptance is independent of that build profile.

## Moves

- 2026-09-12 (d695f385) replaced [[requirement-enforcement]]: C2 removes the source-uncallable command form without restoring a runtime contract exception; all calls use ordinary pre-transfer proof, and a build launcher cannot invoke a required signature without an admitted ordinary caller. (sourced)
