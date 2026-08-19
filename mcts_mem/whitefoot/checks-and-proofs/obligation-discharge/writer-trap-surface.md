- The named, justification-bearing `claim` is the sole writer-reachable language construct that can trap at runtime.
- A false executed claim emits its named accountability record and aborts without unwinding or language cleanup; no contract clause, partial operation, allocation, range operation, or entry wrapper owns another language trap edge.
- Every proof-required hazardous operation must discharge its exact static domain obligation before lowering. A branch, requirement, or retained claim continuation may establish the same goal; refuted and unproved operations reject instead of receiving a runtime fallback.
- Contract definitions, requirements, and postconditions are erased proof metadata. They have no runtime value, storage, ABI component, effect, or failure path.

## Facts

- 2026-08-19 (55a75434) statement: Direction Outline revision 43 and its ACTIVE plan select claim-only runtime traps, static contracts, proof-required exact integer operations, proof-required allocation fit, and proof-required system ranges as one boundary rather than independent compatibility changes. (sourced)

## Moves

- 2026-08-19 (55a75434) replaced [[contract-and-derived-traps]]: any legal non-claim trap gives AI-authored code an unauditable failure edge; contracts are static proof metadata and every partial operation must prove its domain before lowering, so claim must be the sole writer-reachable runtime trap (sourced)
