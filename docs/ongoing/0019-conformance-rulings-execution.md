# 0019 — Conformance rulings execution

This is a temporary live coordination record, not execution authority. Move
this same numbered record to `docs/done/` at terminal disposition.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 4 and the owner's
  2026-08-06 protected-surface approval (`governance/APPROVALS.md`)
- **Owner / workspace:** executor agent / worktree
  `.claude/worktrees/agent-a873aed21e5e3e431`, branch
  `worktree-agent-a873aed21e5e3e431`, lead-reviewed
- **Base revision:** `d84643b`

## Goal

Execute the two approved rulings and classify the two divergences: append
the minimal admissible entry to the 41 incomplete-unit case sources (every
other byte unchanged; each case then reproduces its declared verdict);
correct the 35 overclaiming `runnable` statuses to `pending` with reasons
naming the borrow-parameter capability (task 0021); investigate
`gram5-pos-recursive-place-projection` (expects run 0, gets TYPE-5) and
`type7-neg-propagate-box-holder` (expects TYPE-7, gets ERR-3) to a
classification per the blocker routing — report, do not fix here.

## Progress

- Claimed at `d84643b`; baseline corpus lane re-run started for the
  per-case bucket enumeration the approval's boundary names.

## Validation, stop, and closure

Per-case verdict evidence after each source completion; the corpus lane
tally must improve exactly by the predicted buckets; any case whose verdict
still diverges after completion is a finding, not an edit target. Unpiped
gates. Close to done with the lane tally before/after.
