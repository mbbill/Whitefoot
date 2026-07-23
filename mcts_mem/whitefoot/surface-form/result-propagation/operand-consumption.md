- A direct bare affine own-rooted Result place used by `propagate` consumes its whole storage root exactly once.
- Explicit `move` remains a valid operand spelling.
- Copy Result operands, non-place Result expressions, borrow restrictions, same-E checking, cleanup, effects, and runtime Ok/Err behavior follow their existing rules.

## Facts

- 2026-07-22 owner-approved specification: exact v0.13 selected contextual consumption for the canonical `let value: own T = propagate expression;` form and retained explicit `move`. (sourced)
- 2026-07-22 implementation: one general consuming-expression judgment is shared by match and propagation; a focused regression proves the operand dies and later reuse cites OWN-1. (code)

## Moves

- 2026-07-22 (e7b985ee) replaced [[explicit-move-operand]]: Result forwarding must consume one affine operand, and matching OWN-13 lets the canonical bare propagation form do so without weakening ownership, while requiring `propagate move p` contradicted the approved writer form. (sourced)
