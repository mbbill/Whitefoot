Node: language/checks-and-proofs/requires-entry-contract

Decision: A requirement's places are formed at the callee's body entry, in the state holding the requirements written before its clause, and a definition's places in the first requirement whose expansion reaches them, so every subscript in such a place owes its bounds obligation there, because the callee establishes the requirement as a fact in its own entry state and a subscripted place is a term only where its subscripts are discharged, instead of forming the place in each caller's instantiated goal or leaving clause subscripts unjudged. The [readonly-field term investigation](../../../research/investigations/readonly-field-terms/DESIGN.md#soundness-every-change-is-an-overlapping-write) records the probe that exposed the gap and the cases that pin the rule.

Rejected:
- Forming a requirement's places in each caller's instantiated goal: rejected because the fact is established in the callee's entry state, where the caller's proof of the subscript is not available, so the callee would hold a fact over a place that is no term in its own state.
- Leaving clause subscripts unjudged: rejected because a requirement could then name an element that does not exist yet, and its fact survived the append that creates that element, which let a loop bounded by the element's field read past another storage's length.
