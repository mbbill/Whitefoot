- A written premise used the multiplication operator for its Farkas multiplicity (`use 3 * p;`).
- The left operand was an integer and the right operand was a relation.
- Whitespace distinguished a term multiplicity from an ordinary affine product.

## Facts

- 2026-09-05 rationale: the spelling asserts something false about the operation. A Farkas coefficient is how many times a premise is added into the certificate sum, not a product of two values, and `*` claims a multiplication whose right operand is a relation — a type the operator has nowhere else. (sourced)
- 2026-09-05 statement: the form is undecidable in the strong-LL(2) grammar the parser is fixed to. After `use`, an integer followed by `*` is the common prefix of an ordinary affine product and a multiplied premise, and the two need different trees, so two tokens of lookahead cannot separate them. (sourced)
- 2026-09-05 pitfall: the rescue proposed for that ambiguity was a whitespace rule, which asks the parser to see a distinction it does not have. Tokens carry no spacing, so the rule could only be enforced by re-reading source text beside the token stream — a second, disagreeing reader of the same bytes. (sourced)

## Moves

- 2026-09-05 replaced by [[operation-spelling]]: naming the multiplicity `times` and delimiting every relation premise removes the category error, the ambiguity, and the whitespace rule together; `times` is evidence-selected rather than preferred, with zero uses as an identifier in the corpus and four of its fifteen doc-string appearances already reaching for the word in exactly this sense (sourced)
- 2026-09-06 (db9892c6) replaced by [[operation-spelling]]: spelling the Farkas multiplicity with `*` claimed it was a multiplication whose right operand is a relation, which made a term multiplicity undecidable in strong-LL(2) and forced a whitespace rule to carry a distinction the parser cannot see; naming it `times` and delimiting every relation premise removes all three, and the corpus's own doc strings already reached for the word (sourced)
