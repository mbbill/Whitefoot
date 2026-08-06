# 0024 — General borrow-mode parameters and let-borrows

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 (unsupported
  specified capability, task-0021 finding)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** (executor fills at claim)

## Goal

Implement borrow-mode parameters and let-borrows of scalar and enum types
on the normal path (v0.19 admits the modes; the compiler stops at
`check/types.rs` parse_parameters_with and `check/borrows.rs` check_borrow's
trailing arm — task 0021's reproductions), including match payloads read
through borrows with deref, per the existing buffer/struct/box machinery's
patterns. The 44 waiting conformance cases (35+9, per-case list at task
0021's record) flip runnable and must pass; wfgrep is untouched.

## Validation, stop, and closure

Per-case run evidence for every flip; no existing test weakened; any
semantics question v0.19 does not settle stops the task; unpiped gates.
Close to done with the lane delta.
