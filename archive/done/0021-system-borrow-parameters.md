# 0021 — Borrow-mode parameters for system nominal types

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `fe51150` (cherry-pick), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2

## Outcome

The capability landed in two mechanical sites (parameter admission beside
buffer/slice/struct/box; the OWN-6 child reborrow's SystemResource case) —
effect projection, checked program, and backend already handled borrowed
opaque resources. Every semantics question was settled by v0.19 and verified
by probe (borrowed parameters contribute no release row; owned controls
still do; region attribution both ways). The witness: wfgrep decomposed
from a ~500-line main to 337 lines with `publish_all` (the write loop once,
five call sites) and `report_failure`; all nine oracle cases unchanged. The
cost-shape gates were re-derived, not relaxed, per 0016's recorded caveat:
`publish_all` stays out of line at four sites (cost 245 vs threshold 225 —
deliberately not contorted to squeeze under), so level-2 evidence reads the
program's own code and counts five publication entries (2 stdout, 3
stderr); QUAL-3's wrapper-inlining condition still holds and is still
asserted. Lib tests 439 → 441.

## Finding (routing correction, verified by A/B)

The 35 pending corpus cases attributed to this capability are NOT gated on
it: capability-reverted vs present is byte-identical over the adapter
(Pass=242 Fail=123 both ways on the branch base; per-case diff empty; none
names a system type). They need GENERAL borrow-mode parameters and
let-borrows of scalars (29) and enum nominals (6) — an unsupported
specified capability outside this task's scope and the ACTIVE plan's Work
items: a next-plan candidate, deliberately not absorbed. The 9 unmasked
stragglers from 0019 are the same class. Manifest reasons and the corpus
blocker text were corrected at landing.

## Evidence and validation

- Landed commits: `6d83d93`/`d7088b6`/`ef67636`/`fe51150` + the lead's
  reason corrections. Both gates green by unpiped exit codes; lane
  unchanged at Pass=305 Fail=25 (as the A/B predicted).
