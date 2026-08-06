# 0021 — Borrow-mode parameters for system nominal types

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 (unsupported
  specified capability, task-0015 finding)
- **Owner / workspace:** executor agent / `worktree-agent-a69e30a5a0e00b887`,
  lead-reviewed
- **Base revision:** `d84643b`

## Goal

Implement `&`/`&uniq` parameters of system nominal types on the normal path
(v0.19 restricts nothing here: SYS-4 kinds take shared/unique borrows;
`semantic/check/types.rs` currently admits non-own parameters only for
buffer/slice/struct/box), then decompose `tests/programs/wfgrep.wf`'s
~500-line `main` into helpers as the composition witness: same oracle, same
§9.1 cost gates, behavior byte-identical. The 35 task-0019 `pending` cases
this capability gates flip back runnable and must pass.

## Progress

- Claimed at `d84643b`; authorities read (v0.19 SYS-2/SYS-4/OWN/FN-1/EFF-2,
  task-0015 finding 1, the §9.1 cost-shape gates).
- Next: admit the parameter mode on the normal path, then the `wfgrep`
  decomposition.

## Validation, stop, and closure

Oracle and cost_shape gates green on the refactored program; the 35 cases
runnable and passing; unpiped gates. Any semantics question the spec does
not settle stops the task. Close to done.
