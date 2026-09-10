- Represent a completely bounded supplier set with unresolved destination
  placement separately from an unknown source (CheckedStateOrigins).
- Keep these bounds on the ordinary typed route paths. Projection below a bound
  retains the selected source subtree without appending a destination selector
  to its source; exact fresh overrides retain their exclusions.
- Separate complete typed element queries from weak-update prefixes through
  [[typed-element-queries]].
- Instantiate bounds compositionally through selected actual subvalues. A bound
  whose selected suppliers all resolve to fresh state becomes fresh; an unknown
  summary remains unknown independently of its current actuals.
- Distinguish complete call coverage through [[whole-call-coverage]] from exact
  selected routes. A surviving imported bound participates in [[effect-union]]
  without becoming an established effect or a claim about a particular slot.
- Use shared kernel transfer rules over captured operand value images. Owning
  insertion retains complete supplied contents where established; extraction
  and unresolved replacement retain bounds without exact residual membership.

## Facts

- 2026-09-09 (e298ca67) pitfall: An addressed read's expression children describe address calculation, not the consumed stored value. Capturing that value image and the kernel's checked operand images preserves supplier composition through legal read-out/helper boundaries. (code)
- 2026-09-09 (a577b7cb) pitfall: Complete content coverage does not select descriptor-only reads or a type-directed release subset. Treating it as an exact local place demands effects from unobserved element owners. (code)

## Moves

- 2026-09-09 (e298ca67) replaced [[unknown-contained-transfers]]: Representing all unresolved content placement as unknown blocked fresh container helper composition; finite supplier bounds preserve that composition without asserting exact placement. (code)
