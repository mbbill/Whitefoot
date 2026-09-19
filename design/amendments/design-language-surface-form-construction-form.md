Node: language/surface-form/construction-form

Decision: Construction, match binders, and user-function call arguments name every field in declared order, with a missing, extra, repeated, misspelled, or out-of-order name a hard error, and there is no positional form, because positional construction of same-typed fields admits a silent transposition that is an in-bounds wrong value, while names in declared order lift it to a check-time rejection and keep one byte sequence, instead of positional construction.

Decision: Naming applies even to a single-payload variant, because a rule that names fields only when two fields share a type flips when a field is added, and the single-field ceremony is an accepted verbosity cost since irregularity rather than verbosity is what breaks writers, instead of context-dependent naming.

Decision: A destructuring consume of an owner may end its binder list with a rest marker that stands for the fields it does not name, and every field the marker covers must be copy or affine so that the compiler releases it there, a linear field under the marker being a hard error, because a consume names exactly the parts the writer takes out while every other part of that owner is released at the same point, so a binder per released field would be a name that nothing reads, and a marker that binds nothing admits none of the silent transposition that forbids positional binders, instead of requiring every field of a consumed owner to be named.

Rejected:
- Positional construction and positional match binders: rejected because positional construction of same-typed fields admits silent transposition, an in-bounds wrong value on the forbidden silent-corruption rung; named fields in declared order reject it at check time.
- Extending the rest marker to construction and to call arguments: rejected because there is no value for an unnamed field to take there, while in a consume the unnamed fields already have values and a defined fate, which is release.
