- A named claim admitted any well-typed Boolean predicate and established that predicate on its normal continuation.
- Every admitted claim remained a runtime trap without a later-consumer requirement.
- Checker-proved claims produced an advisory and checker-refuted claims rejected.
- Opaque predicates could serve as assertions, output oracles, or expected aborts.
- The record carried a free-form justification and a name.

## Facts

- 2026-08-21 audit: real programs, compiler fixtures, and protected conformance had accumulated result assertions, impossible-arm tripwires, known-false trap cases, checker-redundant predicates, and claims with no terminal consumer. Mechanical five-field prose could preserve all of them without adding proof authority. (sourced)

## Moves

- 2026-08-21 (77bd9565) replaced by [[writer-trap-surface]]: assertion-like claims turn the sole trap surface into an expected-control and test-oracle escape; residual canonicality admits only independently true checker gaps that are necessary for later source admission (sourced)
