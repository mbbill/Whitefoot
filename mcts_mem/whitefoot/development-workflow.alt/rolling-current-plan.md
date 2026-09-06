- Keep one rolling plan document, `docs/current-plan.md`, as the sole home for execution sequencing, carrying a `Current` field with the live state.
- Derive each milestone in it backward from candidate-project pressure, and revise it in place as work lands rather than writing a new plan per batch.

## Facts

- 2026-08-03 (36273e48) implementation: `docs/current-plan.md` became the sole status-bearing plan and remained `PROPOSED` from then on. (code)
- 2026-09-06 measurement: the workflow that actually emerged runs research to implementation to specification and never wrote to the plan before doing the work, so the document had no step that consumed it.
- 2026-09-06 pitfall: a finished plan that nobody replaces does not become inert, it becomes a changelog. The plan delivered its content at v0.40 and was never retired, so nine later versions were appended to it one clause at a time; the same five paragraphs were simultaneously copied into `docs/roadmap.md`, the specification's status header, and the approval record, and the `Current` field that was supposed to carry the live state went stale underneath them. The tell is a document whose newest content is dated and whose oldest content is a status claim.

## Moves

- 2026-09-06 replaced by [[development-workflow]]: the investigation directory now carries a selected slice's design and measurements, `mcts_mem/` carries what it settled, and the plan retires to `archive/` beside the per-batch `docs/done/` record, where the citations that still name them resolve (sourced)
