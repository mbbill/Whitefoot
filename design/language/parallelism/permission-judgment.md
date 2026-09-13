Decision: Sibling-call permission is judged over a window of checked calls together with every statement between them, with data dependencies, effect footprints, loans, and control exits all quantified over the window, because judging only adjacent call pairs made permission turn on statement adjacency rather than semantics, and two programs with byte-identical output differed 1.9 times in wall time, instead of adjacent-pair enumeration.

Decision: The loan judgment reuses the borrow checker's own overlap vocabulary lifted from one call's arguments to the statements of one window, because four alternatives, reliance edges, schedule-parametric ownership, treating exclusive borrows as writes, and weakening the loan rule, each either duplicated the borrow checker or weakened it, instead of a separate parallel aliasing model.

Decision: Unresolved overlap or an unsupported interposed statement form denies permission, and a missing classification never contributes an empty footprint, because a silent empty footprint would grant permission by omission, instead of defaulting unknown statements to no effect.

Decision: A call written as the scrutinee of a `match_stmt`, `value_match`, or `value_if` is a window member on the same terms as a `let`-bound call, and always the last member of its chain because its arm blocks are not statements of the enclosing block, because [PAR-1] judges calls and two spellings of the same operation otherwise got different lowering and a silently incomplete ledger, instead of confining membership to the `let_stmt` form.

Rejected:
- Judging only consecutive let-bound call pairs, with any other statement ending the candidate group: rejected because permission then turned on statement adjacency rather than semantics, measured as a 1.9 times wall-time difference between programs with identical output.
