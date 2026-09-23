Node: language/effects

Decision: Effect rows may name private structural paths from complete interfaces under erased annotation visibility while executable access retains field privacy, and existing exactness, overlap and kill judgments operate directly on resolved paths, because the [private-container witness](../../research/investigations/modular-compilation/LANGUAGE.md#exact-effects-over-private-representation) needs precise public effects and external formal/wrapper rows without granting runtime field access, instead of whole-object effects forced by privacy, named footprint mapping expansion or opaque disjointness axioms.

Decision: Mentioning a private field in an erased proof or effect annotation grants no exhibited runtime access and creates no scheduling edge, because proof support determines fact validity while body operations determine effects, instead of using proof mentions to justify a padded row or treating permission to describe a path as permission to execute it; the existing pure and no-termination-promise decision remains unchanged.

Rejected:
- The earlier named footprint proposal: rejected because annotation visibility now permits the precise structural path itself, removing the need for a second field-label namespace, mapping expansion and its dependency checks.
- Effect rows as upper bounds: rejected because every declared path still requires an ordinary exhibited access and actual structural overlap must decide independence.
