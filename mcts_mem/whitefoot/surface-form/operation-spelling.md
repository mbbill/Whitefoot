- Arithmetic is spelled infix, and every other table operation is spelled as a named call, comparison included.
- An operation's spelling is fixed by its grammar class and never by its use site.
- The bare infix spellings `+ - * / %` and dotless named spellings `ineg iabs ishl ishr` denote exact mathematical operations whose domain must be proved before lowering; none is a trapping mode.
- Every exact partial family has one matching pure total domain query: `+defined -defined *defined /defined %defined` or the named `.defined` spelling. Branches, requirements, and claims can establish that exact canonical goal.
- Explicit `.wrap`, `.checked`, and `.sat` spellings retain their distinct total or value-returning semantics. The retired `.trap` suffix and hidden trapping aliases do not resolve.
- There is no precedence, associativity, or parenthesization surface, and one expression admits exactly one operation.
- An operator token resolves by its exact spelling and consults no name domain; it is never a declaration, a callee identifier, or an operation name.

## Facts

- 2026-08-19 (55a75434) statement: the ACTIVE claim-only plan selects proof-required exact arithmetic and total domain-query predicates so the same proposition can be established statically or by a named runtime claim without a second operation-specific check channel. (sourced)

## Moves

- 2026-08-19 (55a75434) replaced [[trapping-mode-axis]]: a bare spelling that means trap gives partial arithmetic an implicit failure edge; exact operations should instead require the matching total `.defined` goal, while explicit wrap, checked, and saturating modes keep their distinct value semantics (sourced)
