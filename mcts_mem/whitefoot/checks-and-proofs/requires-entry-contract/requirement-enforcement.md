- All substituted requirements of an ordinary call are judged independently in one shared pre-transfer state.
- Resolution, concrete instantiation, every actual-expression obligation, and borrow feasibility are complete in that state. Consumes, borrow commits, and callee-effect kills have not begun.
- A refuted or unproved requirement rejects at the call. No requirement receives a fallback runtime check and no ordinary callee has an executable contract prologue.
- The sole command entry cannot carry a contract and cannot be called from source. Program start has no requirement judgment, entry-goal metadata, checked wrapper condition, or contract-owned trap path.
- A future foreign or exported boundary must introduce its own separately selected proof adapter before it can call an internal required function; this closed-world rule does not reserve an unchecked entry exception for that future design.

## Facts

- 2026-08-19 (55a75434) statement: the selected single-entry language has one source-uncallable `command fn main`, forbids contracts on it, and defines no FFI, function values, dynamic dispatch, or other external call route. (sourced)

## Moves

- 2026-08-19 (55a75434) replaced [[ordinary-plus-program-entry]]: with one source-uncallable command entry, no FFI, and contracts forbidden on main, every requirement belongs to an internal ordinary call and can be proved before transfer; retaining an entry-only runtime exception would violate the claim-only trap boundary (sourced)
