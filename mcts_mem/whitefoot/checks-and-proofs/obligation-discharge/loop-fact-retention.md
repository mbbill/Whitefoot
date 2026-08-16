- A pre-loop fact is removed at an iteration head only by a continuing kill: a kill event whose structural normal-control successor can reach a later head of the same loop without leaving its body.
- A `return`, propagated-error, or loop-leaving `break` edge inside a loop body is non-continuing and removes no pre-loop fact at that loop's heads.
- Ordinary fallthrough, else-free false edges, and nested-loop continuations classify by the same head-reachability test and keep their kills.

## Moves

- 2026-08-09 (f4c7e60c) replaced [[scope-exit-kill-at-head]]: a scope-leaving edge kills every binding's facts, so one return or propagated error inside a loop body discarded every pre-loop fact at the head — the dominant measured cause of the deflate acceptance divergence; a fact carried into a loop is invalidated only by a kill that can reach a later head of that loop without leaving its body (sourced)
