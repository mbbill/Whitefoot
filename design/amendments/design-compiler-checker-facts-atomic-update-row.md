Node: compiler/checker-facts

Decision: [OP-12]'s refusal of an atomic update whose callee row reaches a prefix of the updated place is judged at the call, from the substituted row [EFF-5] already builds and from the enclosing `set`'s own target, because the rule's two halves live in two places -- only the statement knows which place is being updated and whether the called row returns that place's type, and only the call knows whether its first argument is that very place and what its row reaches -- and both are already held while the call is checked, instead of a second pass over the statement after the call, which would have to rebuild the substitution the call just discarded.

Decision: The judgment runs before [EFF-5]'s pairwise disjointness at the same call, because the target argument's own by-value contribution to that comparison overlaps exactly the row entry [OP-12] refuses, so [EFF-5] otherwise reports the update's own transfer as an unproved overlap and the writer is told to pass one of the two positions when the actual defect is the callee's declared row, instead of letting definition order pick the citation.

Decision: A row entry rooted at the target argument itself is exempt, because that argument is the old value's transfer into `f`, which [OP-12] requires rather than refuses, instead of comparing every entry uniformly, which would make the rule refuse its own admitted form.

Rejected:
- Judging the update at the `set` statement after the right-hand side is checked: rejected because [EFF-5] has already refused the statement by then, so the OP-12 citation the rule names would never be reached.
- Recognizing the update from the statement's syntax alone, without the callee's declared result: rejected because [OP-12] says a call that fails the result condition "is not an atomic update and is judged as an ordinary `set`", so a row returning something else must fall through to [SET-1] untouched.
