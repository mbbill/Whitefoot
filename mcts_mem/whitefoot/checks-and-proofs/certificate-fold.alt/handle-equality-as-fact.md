- The opaque handle a binding contributes was to be published as an ordinary fact stating its defining equality, so that everything provable about `width + padding` stayed provable about `stride`.

## Facts

- 2026-09-05 measurement: built and run. The published equality is invisible to the residual, because the residual is discharged by the direct L0 route by rule, and that route does not consult a published affine fact. The sum folded correctly and the residual then came out as exactly that equality instead of zero.
- 2026-09-05 pitfall: the failure is not that the fact was wrong or missing. It was present, true, and unreachable from the route that had to use it. A fact published into a layer the consumer does not read is indistinguishable from no fact at all, and the symptom — a residual equal to the equality — reads like an arithmetic bug rather than a routing one.

## Moves

- 2026-09-05 replaced by [[certificate-fold]]: the handle needs to exist between the fold and the residual and be substituted away before proving, not to be carried into the proof as a premise (sourced)
