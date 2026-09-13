- An ordinary call completes resolution, concrete instantiation, actual-expression obligations, and borrow feasibility before proving the complete substituted requirement in its pre-transfer state, ahead of every consume, borrow commit, and callee-effect kill.
- A refuted or unproved ordinary call is rejected; no fallback runtime check or executable callee prologue exists.
- Each real process entry evaluates the complete requirement exactly once in its compiler-owned wrapper after setup and before owner transfer; failure traps with zero body calls, while success transfers each owner once to one body call.
- A source call to a program-entry function follows ordinary call-site proof and never enters through the process wrapper.

## Moves

- 2026-08-10 (441cd5b8) replaced [[callee-entry-prologue]]: the unconditional ordinary-callee prologue let a helper hide a protected leaf behind a runtime trap; pre-transfer proof exposes the complete atomic requirement without narrowing the admitted predicate surface, while real process entries retain the checked boundary (sourced)
- 2026-08-19 (55a75434) replaced by [[requirement-enforcement]]: with one source-uncallable command entry, no FFI, and contracts forbidden on main, every requirement belongs to an internal ordinary call and can be proved before transfer; retaining an entry-only runtime exception would violate the claim-only trap boundary (sourced)
