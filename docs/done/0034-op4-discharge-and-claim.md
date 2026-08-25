# 0034 — OP-4 discharge and claim semantics

This is a temporary live coordination record, not execution authority.

- **Status:** `DONE` (2026-08-07: OP-4 discharge-or-reject live, claim semantics live, corpus migrated 71+54+9 claims; ENT-5 evidence packet recorded; wide-scan regression flagged)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0034 / <scratch-root>/wf-0034 (branch task/0034-op4-discharge-and-claim)
- **Base revision:** 8d0e228
- **Dependency:** 0033, 0031

## Goal

Flip the switch per active spec: OP-4 index obligations discharge via the engine or reject with the printed residual (DIAG anchors per the v0.22 psuffix ruling); claim statements execute (CLM-1 semantics, named DIAG-3 record), redundancy advisory and refutation error per CLM-2; index_get path removed with v0.22. Migrate corpus programs per the batch-1 candidate migration note (utf8parse +2 claims, deflate unit ~8 claims + threading requires clauses, sha256 +4 claims per O11) — owner reviews the migration diff per the 2026-08-07 process ruling. Dependency: 0033 and 0031/v0.22 activation terminal.

## Progress

- Completed: CLM-1 claim semantics (checker, S3 facts, check-else-trap
  lowering, DIAG-3 record with the claim name, EFF-2 traps contribution,
  per-function name uniqueness); CLM-2 lifecycle (refutation = hard error
  with predicate and derived negation, redundancy = retained non-rejecting
  advisory list on the checked program, printed to stderr by the driver);
  OP-4 flip (undischarged subscript = compile rejection at the psuffix node
  with the exact ENT-6 residual; discharged subscripts lower with no bounds
  branch — the subscript IR trap plumbing, both bounds-check ops, and both
  guarded index types are deleted); subscripts no longer contribute EFF-2
  `traps` per the v0.22 enumeration; render_expression hardened
  (ProjectValue, BoxDeref). Corpus and derived material migrated in the same
  change: 71 corpus claims + 1 new requires clause, 54 conformance-case
  claims across 12 migrated x-cases, compiler unit/integration tests
  updated. Both gates exit 0.
- Current: reported to lead with the full migration review packet
  (per-file claims, extra-claims-beyond-SIMULATION list, two stopped-out
  conformance cases, wide-scan probe regression).
- Next: lead review; owner review of the migration diff per the 2026-08-07
  process ruling; 0035 acceptance run.

## Stopped-out conformance cases (verdict meaning changes, lead ruling)

- `op4-pos-index` (accept): `return b[i]` with an unproven parameter offset
  now rejects under discharge-or-reject; the case doc asserts the pre-v0.21
  "bounds-checked when unproven" semantics.
- `op4-trap-index-oob` (trap): the runtime OP-4 bounds trap no longer
  exists; the same program is now a compile-time OP-4 rejection.
  Both left byte-identical with their manifest verdicts; they are not
  exercised by the compiler gate.
