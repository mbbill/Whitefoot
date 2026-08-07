# 0029 — v0.20 activation

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 3 and the owner's
  2026-08-07 exact-byte approval (`governance/APPROVALS.md`)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `bbfb353926dcb3794d5fcea3c5e5976e79c58a02`
- **Dependency:** task 0028 (terminal: candidate approved)

## Goal

Install the approved v0.20 bytes as the active specification and bring the
compiler and derived material to it: spec constants and integrity pins,
the three semantic deltas (OWN-14 returned reborrows; OWN-13 arm-scoped
child reborrows with region-remainder suspension; DIAG-1 same-node
citation rank), non-argument reborrow citation recategorization, ledger
note labels, regressions, and the conformance re-verification. Expected
settlement: five of the six red cases reach their approved verdicts;
`own1-neg-match-move-through-borrow` stays untouched pending a separate
owner ruling (compiler cites OWN-5, expectation says OWN-1; the spec text
places move-through-borrow in OWN-5).

## Validation, stop, and closure

`spec/` append-only (new file only, byte-identical to the approved SHA);
one semantic path, capability by rule; regressions land with each delta;
cost-shape gates, the wfgrep oracle, and both make gates green and
unpiped. A divergence between the approved bytes and implementability
stops the task. Close to done with the final corpus tally.
