- Arithmetic is spelled infix, and every other table operation is spelled as a named call, comparison included.
- An operation's spelling is fixed by its grammar class and never by its use site.
- An operation carries exactly one constant spelling; a mode is an operator suffix where the operation sits on a mode axis, and the bare operator carries the trapping mode.
- There is no precedence, associativity, or parenthesization surface, and one expression admits exactly one operation.
- An operator token resolves by its exact spelling and consults no name domain; it is never a declaration, a callee identifier, or an operation name.

## Facts

- 2026-08-09 (a01bc707) measurement: the expression grammar is left-factored so the decision falls one token after the shared atom prefix; the unfactored three-way choice is not strong-LL(2), because an atom is not one token and five two-token starts begin both an atom expression and an infix one. (code)
- 2026-08-09 (a01bc707) statement: the owner rejected respelling four of the six comparisons while two kept named calls. The asymmetry was forced by a lexical collision — bare `<` and `>` cannot be told from a type-argument list at the two-token horizon — and resolving it by removal rather than completion gives a rule stated over a grammar class instead of over a subset. (sourced)
- 2026-08-09 (a01bc707) measurement: that cancellation made the specification delta smaller rather than larger — two modification sites became byte-identical to the previous version and ceased to be modifications at all, and the reserved-name inventory returned to its previous membership. (code)

## Moves

- 2026-08-09 (a01bc707) replaced [[prefix-table-calls]]: the register had marked prefix arithmetic as minimality-selected rather than evidence-selected, and the sweep supplied the missing evidence; retained three-address form admits exactly one operation per expression, so an infix spelling needs no precedence surface and costs no lookahead (sourced)
