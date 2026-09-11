Decision: There is no writer-visible trap surface, no claim statement and no runtime proof trap, because any legal writer-reachable trap gives AI-authored code an unauditable failure edge, instead of a named claim as the sole trap.

Decision: An expected failure is a typed outcome or intended control flow, and an always-true relation is a verified contract or invariant, because a branch whose false edge is not intended program behavior is a proof written as executable code, instead of an impossible-case branch added to satisfy the checker.

Decision: External resource availability follows the specification's external-availability boundary and creates neither a source fact nor a proof fallback, because host quota and exhaustion are outside the source-outcome model, instead of a hidden availability trap.

Rejected:
- A single named, justified claim as the sole writer-reachable runtime trap: rejected because the active specification requires machine-checked, erased evidence with no runtime proof surface.
- Assertion-like claims admitting any Boolean predicate: rejected because they turned the trap surface into an expected-control and test-oracle escape.
- An anonymous body check beside the named claim: rejected because a check is a claim minus its name, justification, accountability, and refutation, so keeping both left a weaker duplicate through which a writer could assert without accountability.
- Contract-carried and derived trap forms beside the claim: rejected because any legal non-claim trap gave generated code an unauditable failure edge.
