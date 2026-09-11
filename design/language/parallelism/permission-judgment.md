Decision: Sibling-call permission is judged over a window of checked calls together with every statement between them, with data dependencies, effect footprints, loans, and control exits all quantified over the window, because judging only adjacent call pairs made permission turn on statement adjacency rather than semantics, and two programs with byte-identical output differed 1.9 times in wall time, instead of adjacent-pair enumeration.

Decision: The loan judgment reuses the borrow checker's own overlap vocabulary lifted from one call's arguments to the statements of one window, because four alternatives, reliance edges, schedule-parametric ownership, treating exclusive borrows as writes, and weakening the loan rule, each either duplicated the borrow checker or weakened it, instead of a separate parallel aliasing model.

Decision: Unresolved overlap or an unsupported interposed statement form denies permission, and a missing classification never contributes an empty footprint, because a silent empty footprint would grant permission by omission, instead of defaulting unknown statements to no effect.

Rejected:
- Judging only consecutive let-bound call pairs, with any other statement ending the candidate group: rejected because permission then turned on statement adjacency rather than semantics, measured as a 1.9 times wall-time difference between programs with identical output.
