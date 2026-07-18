- Construction writes every declared field exactly once as name-colon-value, names equal to the declared field names in declared order; a missing, extra, repeated, misspelled, or out-of-order name is a hard error (GRAM-8).
- Reading is symmetric: match binders are written field-colon-freshBinder in declared order, with the binder a fresh name distinct from the field name.
- User-function call arguments are named in declared order too; table-operation calls stay positional, their operand order being intrinsic.
- There is no positional construction form; a nullary constructor is written with empty parentheses.

## Facts

- 2026-07-08 rationale: uniform naming applies even to single-payload variants because name-only-when-two-same-typed-fields and positional-for-single-field are both context-dependent spellings that flip when a field is added; the single-field ceremony is an accepted verbosity cost — verbosity is free, irregularity is the enemy. (sourced)
- 2026-07-08 statement: the honest delta over Rust is on cheat-proofness and check-time rejection, not performance — Rust leaves tuple structs and all enum payloads positional (admitting exactly the silent same-typed transposition this form closes) and permits free field order with shorthand and update syntax; Whitefoot names every product in one byte sequence. (sourced)
- 2026-07-08 statement: the weak-writer net sign is experiment-gated — the adoption record pre-registers transposition-rate reduction versus the new wrong-field-name error class as the decision metric, with direction already forced and only magnitude open. (sourced)

## Moves

- 2026-07-08 (e687100a) replaced [[positional-construction]]: positional construction of same-typed fields admits silent transposition — an in-bounds wrong value on the forbidden silent-corruption rung; named-in-declared-order fields lift it to a check-time reject while declared order keeps one byte sequence (sourced)
