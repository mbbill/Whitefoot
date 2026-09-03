- Arithmetic is spelled infix, and every other table operation is spelled as a named call, comparison included.
- Comparison keeps its named call because bare `<` cannot be told from a type-argument list at the two-token horizon, and a rule over a grammar class beats a four-of-six subset.

## Facts

- 2026-08-09 (a01bc707) statement: the owner rejected respelling four of the six comparisons while two kept named calls; the asymmetry was forced by the `<` collision and resolved by removal so the rule stayed a grammar class, recorded in full under [[trapping-mode-axis]]. (sourced)

## Moves

- 2026-09-03 replaced by [[operation-spelling]]: the collision is dissolved by a delimiter on call-site type application rather than by keeping six names; comparison is the corpus's most frequent operation, its positional form was the last direction-sensitive one, and v0.40 had made the same four names proof-domain relations over infix affine operands (sourced)
