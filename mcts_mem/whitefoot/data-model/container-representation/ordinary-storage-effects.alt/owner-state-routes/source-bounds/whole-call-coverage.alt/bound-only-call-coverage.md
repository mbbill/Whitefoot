- Treat owning insertion and extraction as supplier bounds when their logical
  slot correspondence is not represented.
- Refuse effect projection for every surviving imported bound, including
  operations that transfer all supplied owners into the complete result.
- Resolve bounded transfers to fresh state only when selected actual suppliers
  all resolve to fresh state; wholly unknown sources remain unknown.

## Moves

- 2026-09-09 (a577b7cb) replaced by [[whole-call-coverage]]: Treating intact insertion as a supplier bound blocked whole-value helper effects; complete coverage preserves that composition without claiming selected content origins. (code)
