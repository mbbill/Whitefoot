# 0024 — General borrow-mode parameters and let-borrows

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 (unsupported
  specified capability, task-0021 finding)
- **Owner / workspace:** executor agent / `worktree-agent-a1d0ac9d018482e6f`,
  lead-reviewed
- **Base revision:** `7240f84`

## Progress

- Claimed at `7240f84`; authorities read (v0.19 OWN-2/4/5/6/10/11/13, TYPE-7,
  GRAM-10, EFF-2 §9.1; task-0021 finding; `docs/WORKFLOW.md` execution loop).
- Baseline before any change: corpus adapter Pass=306 Fail=24 Skip=49. The 44
  waiting cases are the 35 manifest `pending` entries naming this capability
  plus the 9 runnable `RegionsAndBorrows` failures (`reject-own10-dangle`,
  `gram3-pos-modes`, `own4-pos-return-caller-borrow`,
  `own4-neg-return-local-borrow`, `own10-neg-dangle-caller`,
  `own13-pos-borrow-match-live`, `x-borrow-return-uniq-local-region`,
  `type7-neg-match-reference-call`, `type7-neg-return-reference-holder`).

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
