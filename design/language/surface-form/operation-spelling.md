Decision: Arithmetic and integer comparison are spelled infix and every other table operation is a named call, with an operation's spelling fixed by its grammar class and never by its use site, because three-address form admits exactly one operation per expression so an infix spelling needs no precedence surface and costs no lookahead, instead of mode-suffixed prefix calls for all computation.

Decision: The bare infix spellings and the dotless named spellings denote exact mathematical operations whose domain must be proved before lowering, and none is a trapping mode, because a bare spelling that means trap gives partial arithmetic an implicit failure edge, while explicit wrap, checked, and saturating modes keep their distinct value semantics, instead of a bare operator that traps.

Decision: The six integer comparison symbols are integer-only rows exactly as `+` is, and call-site type application carries the `::` delimiter, because the delimiter dissolves the collision between `<` as comparison and `<` as type application that had forced comparisons into named calls, and comparison is the corpus's most frequent operation whose positional form was the last direction-sensitive one, instead of named comparison calls.

Decision: A `use` cites one premise with its multiplicity written as `N times` before it, a relation premise always parenthesized and a named premise never, because spelling the multiplicity with `*` claimed a multiplication whose right operand is a relation, which was undecidable with two tokens of lookahead and forced a whitespace rule to carry a distinction the parser cannot see, instead of `use 3 * p`.

Rejected:
- Strong four-token lookahead to keep `<` overloaded: rejected because it commits a comparison with a nested right operand to the call arm and turns two-token diagnostic attribution into a four-token case analysis.
