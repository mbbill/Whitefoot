- Represent a completely bounded supplier set with unresolved destination
  placement separately from an unknown source (CheckedStateOrigins).
- Keep these bounds on the ordinary typed route paths. Projection below a bound
  retains the selected source subtree without appending a destination selector
  to its source; exact fresh overrides retain their exclusions.
- Instantiate bounds compositionally through selected actual subvalues. A bound
  whose selected suppliers all resolve to fresh state becomes fresh; an unknown
  summary remains unknown independently of its current actuals.
- Require exact selected routes at effect consumers. A surviving imported bound
  is an explicit capability limit, not an exhibited exact effect set or a claim
  about which owner occupies a particular slot.
- Use shared kernel transfer rules over captured operand value images. Owning
  insertion, extraction and unresolved replacement retain their possible
  suppliers without claiming exact placement or residual membership.

## Facts

- 2026-09-09 (e298ca67) pitfall: An addressed read's expression children describe address calculation, not the consumed stored value. Capturing that value image and the kernel's checked operand images preserves supplier composition through legal read-out/helper boundaries. (code)

## Moves

- 2026-09-09 (e298ca67) replaced [[unknown-contained-transfers]]: Representing all unresolved content placement as unknown blocked fresh container helper composition; finite supplier bounds preserve that composition without asserting exact placement. (code)
