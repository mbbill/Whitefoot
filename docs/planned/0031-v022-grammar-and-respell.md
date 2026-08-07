# 0031 — v0.22 grammar path and corpus respell (atomic activation prep)

This is a temporary live coordination record, not execution authority.

- **Status:** `PLANNED` (unclaimed)
- **Authority:** `ACTIVE` `docs/current-plan.md` selected slice; owner rulings
  "批" and the N1 version-compat deferral (2026-08-07); the v0.22 delta
  `governance/spec-evolution/index-surface-v022-candidate.md` and assembled
  `kernel-spec-v0.22-candidate.md`
- **Owner / workspace:** unclaimed / filled at claim
- **Base revision:** filled at claim
- **Dependency:** none (candidates landed; step-4 exact-byte approval follows
  this task's evidence, before installation)

## Goal

On one task branch, 0030-style atomic-activation prep for v0.22:
(1) grammar tables — `index` leaves the fixed atoms (IDENT-eligible),
pbase loses the index alternative, psuffix gains `"[" atom "]"`, brackets
join the right-attachment set per O1; (2) delete the index_get catalog row
and reservation; (3) repoint identity pins to the v0.22 candidate bytes
(rule count stays 128); (4) respell the corpus mechanically via reprint —
tests/programs (266 subscript sites + 84 region headers + 31 cvalue
arrays), tests/conformance (138 sites) with verdicts meaning-unchanged per
the derived-material rule, plus the one new O5 conformance case (`index`
as ordinary IDENT); (5) evidence: verifier green on the branch against the
v0.22 candidate (delta §3 expectations), main untouched and green,
`make -C compiler check` green; (6) STOP before merge — report the
candidate SHA-256 from your worktree for the owner's step-4 approval.
Discoveries outside the candidates stop the task with evidence.
