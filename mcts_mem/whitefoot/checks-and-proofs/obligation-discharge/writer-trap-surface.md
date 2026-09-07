- The writer-visible trap surface is retired. The active specification's SCOPE-2, INV-1, and PRF-1 require machine-checked, erased evidence and provide no `claim` statement or runtime proof trap.
- Expected failure belongs to typed outcomes and intended control flow. An always-true relation belongs to verified contracts or invariants, never an impossible-case branch added to satisfy the checker.
- External resource availability follows SCOPE-3; it does not create a source fact or a proof fallback. Historical claim admission, accountability, and abort behavior below describe the replaced design.

## Facts

- 2026-08-19 (55a75434) statement: Direction Outline revision 43 and its ACTIVE plan select claim-only runtime traps, static contracts, proof-required exact integer operations, proof-required allocation fit, and proof-required system ranges as one boundary rather than independent compatibility changes. (sourced)
- 2026-08-21 (77bd9565) statement: residual canonicality makes the trap surface an approval boundary rather than a general assertion surface — source claims that are checker-known, refuted, vacuous, unsupported, overlapping, or not load-bearing never reach checked IR, while admitted claims remain mandatory runtime checks. (code)

## Moves

- 2026-08-19 (55a75434) replaced [[contract-and-derived-traps]]: any legal non-claim trap gives AI-authored code an unauditable failure edge; contracts are static proof metadata and every partial operation must prove its domain before lowering, so claim must be the sole writer-reachable runtime trap (sourced)
- 2026-08-21 (77bd9565) replaced [[assertion-like-claims]]: assertion-like claims turn the sole trap surface into an expected-control and test-oracle escape; residual canonicality admits only independently true checker gaps that are necessary for later source admission (sourced)
