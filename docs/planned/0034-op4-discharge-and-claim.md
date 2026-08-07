# 0034 — OP-4 discharge and claim semantics

This is a temporary live coordination record, not execution authority.

- **Status:** `PLANNED` (unclaimed)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** unclaimed / filled at claim
- **Base revision:** filled at claim
- **Dependency:** 0033, 0031

## Goal

Flip the switch per active spec: OP-4 index obligations discharge via the engine or reject with the printed residual (DIAG anchors per the v0.22 psuffix ruling); claim statements execute (CLM-1 semantics, named DIAG-3 record), redundancy advisory and refutation error per CLM-2; index_get path removed with v0.22. Migrate corpus programs per the batch-1 candidate migration note (utf8parse +2 claims, deflate unit ~8 claims + threading requires clauses, sha256 +4 claims per O11) — owner reviews the migration diff per the 2026-08-07 process ruling. Dependency: 0033 and 0031/v0.22 activation terminal.
