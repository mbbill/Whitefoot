- Centralize all plan-derived execution authority and sequencing in one rolling Current Plan.
- Let task records cite plan or separate owner authority without broadening, selecting, or resequencing work.
- Give every substantial independently integrable task or distinct handoff boundary one numbered task record with its owner, base, goal, direction, method, progress, advisory touch set, dependencies, integration order, validation, and closure; [[task-record-lifecycle]] governs its live and terminal locations.
- Publish a task record before substantial work for discovery from separate workspaces; contributors to one deliverable share a record, and read-only reviewers create none.
- Treat textual overlap as rebase work rather than exclusive ownership, but cross-link semantic or authority overlap and select one premise and integration order before both changes land.
- Make plan replacement fail closed: a plan-derived task loses authority unless a new active plan explicitly carries its exact scope, while separately approved work lasts only to its own stop condition.
- Move a terminal task record to frozen closure history after moving durable facts, measurements, decisions, and status to their canonical owners and repairing live dependent links in the same change.

## Moves

- 2026-08-05 (05d5fe6d) replaced [[sole-current-plan-status]]: the sole-plan status model could not keep several separately bounded ongoing tasks visible without making their records look authorizing; separating execution authority from task status preserves bounded approval while allowing concurrent progress (sourced)
