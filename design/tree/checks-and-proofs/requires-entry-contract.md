# Requires and entry contract

Decision: A non-entry function may carry one contract block after its signature, compiled away entirely, in which shared define abbreviations come first and any number of requires and ensures clauses follow, each proved on its own, and a present block holds at least one clause, because the earlier separate requires and ensures blocks were spelled like executable code so a reader could not see they were erased, an abbreviation needed by both had to be written twice, and ensures had no name for the function's result, instead of a single final requirement block ending in one Boolean check.

Decision: A define is a proof-only abbreviation that is substituted textually into every clause using it and never runs, stores a value, or takes a snapshot, because the same abbreviation is read at two different moments, before the call for requires and at the callee's return for ensures, and no single stored value could be right at both, instead of evaluating it once into a hidden slot.

Decision: Every function names its result in the signature so that ensures can refer to it, and a postcondition about a Result's success value writes `Ok(value: name)` to bind the payload, because the result name stands for the whole value while the arithmetic facts the checker tracks live on the integer payload inside it, instead of an unnamed result or a payload-only convention.

Decision: Each requires clause is proved on its own at the call site, in the state just before the call, and none may use another as a premise; two clauses count as the same goal when they name the same operations, types, constants, parameters, and literals after definitions are expanded, whatever the local names or how the expression was shared, because a goal proved once must be recognized as that goal wherever it appears again, instead of matching goals by spelling or by an algebraic normal form.

Decision: A function instance whose requirements contradict each other is legal and simply uncallable: the compiler still checks its structure, ownership, effects, and return shape, lowers an unreachable stub with the right calling convention in place of its body, and publishes no postcondition, because the owner chose a clear meaning for the contract surface over compatibility with the earlier model, instead of rejecting such an instance as a source error.

Rejected:
- A single final requirement block with local bindings and one Boolean check: rejected because it was spelled like executable code so its erased status was invisible, an abbreviation used by both requires and ensures had to be written twice, and the result had no name.
- A single pattern recognizer that matched the base64 capacity shape and elided its checks: rejected because it was right on its one shape but reported only pass or fail, never which fact was missing or which premise failed, so nothing it could not classify failed safely.
