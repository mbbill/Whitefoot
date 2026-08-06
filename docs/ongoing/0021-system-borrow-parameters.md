# 0021 — Borrow-mode parameters for system nominal types

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 (unsupported
  specified capability, task-0015 finding)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** (executor fills at claim)

## Goal

Implement `&`/`&uniq` parameters of system nominal types on the normal path
(v0.19 restricts nothing here: SYS-4 kinds take shared/unique borrows;
`semantic/check/types.rs` currently admits non-own parameters only for
buffer/slice/struct/box), then decompose `tests/programs/wfgrep.wf`'s
~500-line `main` into helpers as the composition witness: same oracle, same
§9.1 cost gates, behavior byte-identical. The 35 task-0019 `pending` cases
this capability gates flip back runnable and must pass.

## Validation, stop, and closure

Oracle and cost_shape gates green on the refactored program; the 35 cases
runnable and passing; unpiped gates. Any semantics question the spec does
not settle stops the task. Close to done.
