Node: language/name-resolution

Decision: A qualified path names a type, a group, a callee or a constructor, while a place base, a `const` term and a `cvalue` reference stay single identifiers, so another module's constant is named in a value position through a file-local alias, because a qualified place base would split `place` across operand nodes and every rule, proof term and diagnostic that cites `place` would then need a second shape, while an alias names the same identity with the one existing shape, instead of qualified operands factored through the expression grammar; the active specification v0.70 already writes this grammar [GRAM-3, GRAM-5, CONST-1, CONST-2], and the decision replaces none.

Rejected:
- Qualified place bases and constant terms factored through a shared operand prefix, as the research grammar candidate wrote them: rejected because recognizing `m::capacity` and `m::f(...)` with one strong-LL(2) prefix moves the place base out of `place` into an operand node, so [MSR-1] measure places, [ENT-2] tracked places, [SET-1] targets and [EFF-1] paths would each need a second place shape for a spelling an alias already provides.
