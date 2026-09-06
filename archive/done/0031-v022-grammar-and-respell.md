# 0031 — v0.22 grammar path and corpus respell (atomic activation prep)

This is a temporary live coordination record, not execution authority.

- **Status:** `DONE` (2026-08-07: v0.22 activated; verifier 65/75/76; adapter 365/1/14; merged)
  branch gates are green; stopped before merge per the card. Awaiting owner
  review of the respell packet and the step-4 exact-byte approval.
- **Authority:** `ACTIVE` `docs/current-plan.md` selected slice; owner rulings
  "批" and the N1 version-compat deferral (2026-08-07); the process ruling
  (reviewable respell, no emitter) and the conformance repurpose ruling
  ("改用途"), both 2026-08-07 in `governance/APPROVALS.md`; the v0.22 delta
  `governance/spec-evolution/index-surface-v022-candidate.md` and assembled
  `kernel-spec-v0.22-candidate.md`.
- **Owner / workspace:** exec-0031 (second executor, took over at the
  handback boundary) / `<scratch-root>/wf-0031-exec2`, branch
  `task/0031-v022-grammar-and-respell`
- **Base revision:** ba6c5cf (branch rebased onto the conformance-ruling
  commit so the cited approval is in-tree)
- **Dependency:** none (step-4 exact-byte approval follows this task's
  evidence, before installation)

## Goal

On one task branch, 0030-style atomic-activation prep for v0.22: grammar
tables, semantic re-anchoring, identity pins, index_get removal, mechanical
corpus respell with owner review packet, the repurposed and additive
conformance cases, and evidence — STOP before merge.

## Progress

Complete on the branch at de9ef81; nothing remaining except lead/owner
review and step-4.

- 706c9fe (first executor): grammar tables, terminal inventory, FORM-2
  right-attachment; a5fc13c: the repurposed element-type case per the
  owner ruling.
- 2fc016b: semantic re-anchoring (subscript as psuffix, suffix-limit
  threading, element type derived from the base place, wrong-base anchored
  per O3, offset stays own u64/Type5, FN-8 psuffix arm, resolution role
  split), plus 84 discovered-missing `]`-closing SELECT_2 follow rows
  (1919 -> 2003; without them `table[ordinary]` failed closed — the first
  generation added the `[`-opening rows only), identity pins to the
  candidate bytes, and the index_get catalog/reservation removal.
- de9ef81: mechanical corpus respell (266 program + 133 conformance sites,
  84 region headers, 31 cvalue arrays, one manifest doc, embedded compiler
  fixtures), and the additive O5 case `form3-pos-index-ordinary-ident`.

## Evidence

- `whitefoot-grammar` on the candidate: 65 productions / 75 decisions /
  76 terminal predicates; candidate SHA-256
  `b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8`,
  128 rules.
- `make -C compiler check` and `make check` green on the branch; main
  untouched at ba6c5cf.
- Conformance verdict comparison, branch vs main baseline (full adapter
  run): main Pass=364 Fail=1 Skip=14; branch Pass=365 Fail=1 Skip=14 —
  the one Fail is the same pre-existing `own1-neg-match-move-through-borrow`
  OWN-5-for-OWN-1 mismatch on both sides, and the +1 Pass is the additive
  case. No verdict changed meaning.
- Scratch parse/run matrix (nested offsets `values[order[j]]`, triple
  chain, deref chain, field-prefixed subscript, set target, literal and
  IDENT offsets) compiles and runs exit 0; an out-of-bounds variant traps
  OP-4 with the trap record at the psuffix node.
- `tests/codegen/` deliberately untouched (preserved holding corpus per
  its README; no harness consumes it); research experiment bundles stay
  frozen evidence.

## Stop condition

Reached: STOP before merge. The review packet (per-file diffstat, ten
representative before/after excerpts, no-verdict-meaning-change statement)
was reported to the lead for the owner. Merge, spec installation, and the
activation change are outside this task.

## Closure

At integration, move this record to `docs/done/` with the landed commits
and the canonical evidence links above.
