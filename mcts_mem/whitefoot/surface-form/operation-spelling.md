- Arithmetic and integer comparison are spelled infix; every other table operation is spelled as a named call.
- An operation's spelling is fixed by its grammar class and never by its use site.
- The bare infix spellings `+ - * / %` and dotless named spellings `ineg iabs ishl ishr` denote exact mathematical operations whose domain must be proved before lowering; none is a trapping mode.
- Every exact partial family has one matching pure total domain query: `+defined -defined *defined /defined %defined` or the named `.defined` spelling. Branches, requirements, and claims can establish that exact canonical goal.
- Explicit `.wrap`, `.checked`, and `.sat` spellings retain their distinct total or value-returning semantics. The retired `.trap` suffix and hidden trapping aliases do not resolve.
- There is no precedence, associativity, or parenthesization surface, and one expression admits exactly one operation.
- An operator token resolves by its exact spelling and consults no name domain; it is never a declaration, a callee identifier, or an operation name.
- The six integer comparison symbols `== != < <= > >=` are integer-only rows exactly as `+` is; float and tag-only enum comparison keep their prefixed names, and the four ordered symbols are also the proof-domain relations of an invariant or a use step.
- Call-site type application carries the `::` delimiter (`cvt::<u8, u32>(w)`); constructors and type position are unmarked, so `IDENT <` is always a comparison and the expression decision stays two-token.
- A multiplied use relation is parenthesized, `use 3 * (a <= b);`, and a bare one is not; the parentheses delimit the premise the multiplier scales.

## Facts

- 2026-08-19 (55a75434) statement: the ACTIVE claim-only plan selects proof-required exact arithmetic and total domain-query predicates so the same proposition can be established statically or by a named runtime claim without a second operation-specific check channel. (sourced)
- 2026-09-03 rationale: the owner ruled the six comparisons symbolic as one class once the `<` collision was dissolved by the call-site `::` delimiter; strong-LL(4) was rejected because it commits a comparison with a nested right operand to the call arm and turns DIAG-1's two-token GRAM-9 attribution into a four-token case analysis. (sourced)
- 2026-09-03 measurement: parsing is 0.13–2.5% of compile time across `tests/programs` (wfgrep: 8 ms of 6.2 s), so lookahead cost decided nothing. (code)
- 2026-09-03 rationale: `!=` over `<>` and `/=` by LEX-1; `!` enters the alphabet only inside that compound. Deleting the multiplied use relation was rejected as a semantic change disguised as spelling: its rewrite through a named invariant makes a factor-2 block AUTO-redundant. (sourced)

## Moves

- 2026-08-19 (55a75434) replaced [[trapping-mode-axis]]: a bare spelling that means trap gives partial arithmetic an implicit failure edge; exact operations should instead require the matching total `.defined` goal, while explicit wrap, checked, and saturating modes keep their distinct value semantics (sourced)
- 2026-09-03 replaced [[named-comparisons]]: v0.23's whole-class cancellation rested on the `<` collision, which a delimiter on call-site type application dissolves; comparison is the corpus's most frequent operation, its positional form was the last direction-sensitive one, and v0.40 had made the same four names proof-domain relations over infix affine operands (sourced)
