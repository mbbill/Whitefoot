# v0.19 arg_get parameter-spelling repair

This is a temporary live coordination record, not execution authority. Move
this same numbered record to `docs/done/` at terminal disposition.

- **Status:** `IN PROGRESS`
- **Authority:** owner authorization 2026-08-06 ("授权") for the one-name
  v0.19 amendment batch, following the defect recorded at Outline revision 9
  and `docs/done/0007-system-declaration-domain.md`; the specification-change
  workflow governs; exact byte approval still required before activation
- **Owner / workspace:** lead agent / `main` integration workspace
- **Base revision:** `907076a`

## Goal

Repair the v0.18 internal contradiction: [SYS-2] names `arg_get`'s value
parameter `index`, a fixed [GRAM-5] atom excluded from IDENT by [FORM-3], so
the [GRAM-11]-required call-site spelling is underivable and no legal
`arg_get` call exists. v0.19 renames the parameter to `position` (control
verified: the renamed call compiles through the grammar).

## Method

Copy the active v0.18 as the v0.19 candidate; change exactly the one [SYS-2]
signature line plus the META-5 status header; verify with the native grammar
verifier; present SHA-256 for exact approval; activate atomically with the
derived material: compiler catalog spelling + its byte-exact SYS-2 test,
any conformance case naming the parameter, the derivation-record SYS-2
amendment note, Outline defect-note clearance, and an mcts pitfall fact.

## Validation, stop, and closure

- Verifier green on the candidate; both gates green at activation; sweep
  evidence retained (sole collision).
- Stop: any second byte change beyond the rename and header.
- Close: move to `docs/done/` in the activation change.
