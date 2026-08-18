- A bare take is legal when the checker proves the hole is refilled before scope end; the place is vacant at intermediate program points inside one scope.

## Facts

- 2026-08-18 statement: the concrete failure is every scope-leaving edge inside the window — a trap aborts soundly, but a propagate or return edge leaves the scope with the hole open, so the checker must forbid or synthesize repair on return, break, propagate, and every trapping call between take and refill, track per-place hole state across statements, and define what a unique holder means while its referent is vacant; all of it buys only holding old and new values at distinct program points, which neither measured consumer needs since both construct the replacement first. (sourced)

## Moves

- 2026-08-18 (eb8e8634) replaced by [[affine-replacement]]: a hole open across statements needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window, and a meaning for `&uniq` over a vacant referent, buying only a use-then-refill window neither consumer needs (sourced)
