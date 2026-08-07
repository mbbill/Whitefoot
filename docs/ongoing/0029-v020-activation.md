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

## Progress

- **Completed (executor, branch `task/0029-v020-activation`):** v0.20
  installed as `spec/kernel-spec-v0.20.md`, byte-identical to the approved
  SHA; identity pins repointed (spec constants, whitefoot-spec Ok(120),
  qualification-row version key, standalone runner pin, ledger notes
  relabeled active). OWN-14 landed as an explicit reborrow-position fact on
  the one borrow path (return position admitted; every other non-argument
  form the OWN-14 hard error). OWN-13 landed as root suspension: binder
  creation from a uniq root suspends it for the region remainder, the
  suspended ancestor's loan no longer confiscates its descendants' uses,
  and the ownership join unions suspension. DIAG-1 same-node rank landed
  as `SemanticRule::definition_rank`, machine-checked against the active
  bytes, with the return-position TYPE-7 judgment asked first. Corpus:
  Pass=364 Fail=1 Skip=14 (from 359/6/14); the five settled as approved;
  `own1-neg-match-move-through-borrow` untouched, still OWN-5 vs expected
  OWN-1, pending the owner ruling. Both gates and the §9.1 cost-shape and
  wfgrep suites green by unpiped exit codes. Roadmap authority pointer
  (`docs/roadmap.md` lines 6-8 name v0.19) left to lead integration.

## Validation, stop, and closure

`spec/` append-only (new file only, byte-identical to the approved SHA);
one semantic path, capability by rule; regressions land with each delta;
cost-shape gates, the wfgrep oracle, and both make gates green and
unpiped. A divergence between the approved bytes and implementability
stops the task. Close to done with the final corpus tally.
