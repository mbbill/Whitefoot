- Taking an affine value flows the slot to an Option-like vacant type state; the checker tracks per-place type states across program points.
- A boundary crossing while a place is vacant requires the signature to state vacancy, effect-row-like, on every holder type that could reach the place.

## Facts

- 2026-08-18 statement: the concrete failure is that the slot's type must change per program point, so the checker needs flow-sensitive per-place type states, and a function receiving a unique holder could not state whether the referent is whole or vacant without vacancy annotations on every boundary; the cost lands on every writer and every rule while the measured consumers need none of it. (sourced)

## Moves

- 2026-08-18 (eb8e8634) replaced by [[affine-replacement]]: per-place flow-sensitive type states are exactly what the D1a simplification levers exclude, and vacancy would leak into every boundary signature; the Option-shaped value returns legitimately as element vacancy with no checker extension (sourced)
