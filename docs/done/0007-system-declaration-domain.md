# 0007 — System declaration domain

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `c1178e8`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 2; lead-authorized overlap with 0006 (0006 landed first; this task
  rebased and adopted its accessor)

## Outcome

The Route C system declaration domain is implemented: the SYS-2 inventory
(14 nominals, 39 constructors with typed fields, 11 operation signatures)
lives as a structured compiler catalog (`compiler/src/resolution/catalog.rs`)
with a test requiring byte-exact equality against the active spec's SYS-2
blocks; kind-declaring units ([SYS-3] via `syntax::unit_program_kind`, read
after FN-8 admission per DIAG-1's stage order) admit all 167 records with
exact `system_declaration_ordinal`s into the three TYPE-6 domains; unadmitted
units see ordinary undeclared names and lookalike declarations resolve to
source. DIAG-1 rank-5 collisions reject deterministically in both directions
with the `(System, ordinal)` origin, no shadowing either way. The
activation-era resolution unsupported gate is removed; a resolved system use
now stops at semantic checking as explicit unsupported
(`SystemDeclarationUse`), and a kind-declaring unit naming no system
declaration stops at its `program_kind` node — the boundary moved to the
right stage, it did not disappear. Surface for 0008/0009:
`SystemDeclarationId`/`SystemDeclarationRecord`, `ResolvedTarget::System`,
`DeclarationOrigin::System`, the catalog tables, and
`operation_region_effects`.

## Evidence and validation

- Landed commits: `07f9733` (claim), `c1178e8` (implementation), `3c9191e`
  (record update). Both gates green on main; 381 tests, 0 failed; no
  conformance verdict touched.

## Findings and follow-ups

- **Specification defect (blocks part of 0008; v0.19 requested):** v0.18 is
  internally contradictory — SYS-2 names `arg_get`'s second parameter
  `index`; GRAM-11+SYS-2 require that exact spelling as a call-site
  `fieldinit` IDENT; but `index` is a fixed GRAM-5 atom excluded from IDENT
  by FORM-3, so no complete legal `arg_get` call exists. Reproduction and a
  rename control live under `/Users/bytedance/do_not_scan/wf-0007/`. A
  systematic sweep of every SYS-2 parameter name, field name, operation
  name, label tail, and canonical binder against the 67 fixed terminals
  found exactly this one collision. The catalog keeps the normative
  spelling; nothing was worked around.
- Pre-existing v0.17-era discrepancy (untouched): failed `fn_bind`
  right-IDENT lookups cite FN-4 where DIAG-1's role table says FN-3.
- Tasks 0008/0009 consume the resolver surface; 0008 also owns the
  semantic-gate ordering flagged at 0006 closure.
