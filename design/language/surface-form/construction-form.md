Decision: Construction, match binders, and user-function call arguments name every field in declared order, with a missing, extra, repeated, misspelled, or out-of-order name a hard error, and there is no positional form, because positional construction of same-typed fields admits a silent transposition that is an in-bounds wrong value, while names in declared order lift it to a check-time rejection and keep one byte sequence, instead of positional construction.

Decision: Naming applies even to a single-payload variant, because a rule that names fields only when two fields share a type flips when a field is added, and the single-field ceremony is an accepted verbosity cost since irregularity rather than verbosity is what breaks writers, instead of context-dependent naming.

Rejected:
- Positional construction and positional match binders: rejected because positional construction of same-typed fields admits silent transposition, an in-bounds wrong value on the forbidden silent-corruption rung; named fields in declared order reject it at check time.
