Node: compiler/generic-validation-scope

Decision: Symbolic generic validation analyzes entailment only for the bodies of the canonical generic instances and of every function reachable through their calls, leaving the other bodies of its scratch inventory unanalyzed, because only those instances are judged and a judged body reads another function's analysis solely through the postcondition summaries of its callees, while the [flow-analysis follow-up](../../research/investigations/proof-certificate-architecture/CHECKING-COST.md#flow-analysis-closure-follow-up) finds the validation repeating the complete analysis of nongeneric bodies that the concrete phase then performs again, instead of analyzing the whole scratch inventory.

Rejected:
- Reusing the validation's analysis of a nongeneric body in the concrete phase: rejected because the scratch inventory's function and nominal identities are discarded at its checkpoint, so its analysis is not the concrete program's.
- Excluding every nongeneric function from validation: rejected because a judged generic body can discharge an obligation only through a reachable callee's published postcondition.
