- Every scope-leaving edge is a kill event for each binding whose scope it leaves, and an iteration head removes every pre-loop fact that any kill event in the loop body invalidates.
- A `return` or propagated-error edge inside a loop body leaves every live binding's scope and kills every pre-loop fact at that loop's head.

## Moves

- 2026-08-09 (f4c7e60c) replaced by [[loop-fact-retention]]: a scope-leaving edge kills every binding's facts, so one return or propagated error inside a loop body discarded every pre-loop fact at the head — the dominant measured cause of the deflate acceptance divergence; a fact carried into a loop is invalidated only by a kill that can reach a later head of that loop without leaving its body (sourced)
