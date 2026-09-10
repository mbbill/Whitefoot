- Keep complete supplier coverage distinct from both structural correspondence
  and an incomplete supplier bound (StateOriginPrecision).
- Preserve completeness through transfers retaining every supplied subvalue,
  including whole result components moved through helpers and static fields.
- Use completeness only for declared effects on the complete actual value.
  Local place accesses and type-directed releases do not acquire exact
  selected routes from that coverage.
- Selecting inside unlocated contents or excluding part of them reduces
  complete coverage to a bound. An already bounded or unknown actual never
  acquires completeness through call substitution or wrapping.
- Partitioning an owning run supplies no complete-coverage claim for either
  output without a separate selection or residual-content image.

## Moves

- 2026-09-09 (a577b7cb) replaced [[bound-only-call-coverage]]: Treating intact insertion as a supplier bound blocked whole-value helper effects; complete coverage preserves that composition without claiming selected content origins. (code)
