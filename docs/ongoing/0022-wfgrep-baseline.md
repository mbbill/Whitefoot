# 0022 — Zero-change wfgrep baseline

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 1 (PERF-1)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** (executor fills at claim)
- **Frozen program bytes:** `tests/programs/wfgrep.wf` SHA-256
  `d5f94c1a0f9bd3d2e2b014f39f01f19730e85fb626733f8c0780366179732caa` — the zero-change
  baseline measures exactly these bytes regardless of later refactors

## Goal

Preregister and run the first honest performance measurement of the frozen
sequential `wfgrep` against a pinned comparator (`grep -h -F`), then profile
and attribute the first material divergence per the PERF-1 layer chain. The
RG-BASE lesson binds: host cache-position noise defeated a 3% precision gate
once; the protocol must state its noise controls and materiality rules
before any number is read. The scalar newline scan retaining its bounds trap
is the preregistered first attribution suspect — confirmed or refuted by
profile, never assumed.

## Validation, stop, and closure

Protocol committed before measurement; results (win, parity, or attributed
loss — all honest closures) recorded in
`research/experiments/wfgrep-baseline/RESULTS.md`; if attribution cannot
distinguish causes within the preregistered precision, that inability is the
recorded result. Unpiped gates. Close to done.
