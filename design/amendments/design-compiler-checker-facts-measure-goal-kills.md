Node: compiler/checker-facts

Decision: A goal whose support names a measured place's descriptor word is killed by the same source event and resolved-word overlap as the corresponding [ENT-2] term, with a whole-value write covering its descriptor words and an element-position write reaching only the selected element's measures, because both proof levels read the same [MSR-2] support and the pending compiler/checker-facts measure-word-kills decision, so retaining an old length goal beside a verified changed length would create a contradiction that discharges every later obligation, instead of using whole-place containment for goals or killing the outer container's measures on every element write.

Rejected:
- Leaving the goal level alone because the term level already kills: rejected because the observed effect is an unsound accept, not a lost proof: a callee's declared exit relation and the caller's stale reading of the same measure were both live, and the [FN-9] postcondition of the enclosing function was then discharged by contradiction whatever it claimed.
