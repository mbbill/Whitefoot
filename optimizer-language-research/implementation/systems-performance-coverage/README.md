# Systems-Performance Coverage — package index

Status: **COMPLETE and PARKED** (2026-07-17, D19). The design is finished and
budget-verified; the production landing is a separate owner-gated phase not yet
opened. This folder is the whole design package for the D15 systems-performance
capability research. Nothing here authorizes a production change.

## Start here

- **`DESIGN-DOSSIER.md`** — the consolidated, owner-readable design document.
  Read this first: goal, the ten sealed parts, the six kernel deltas with their
  review histories, all the evidence, the five ruled decisions, what remains,
  and honest limitations. Everything else is source material for it.
- **`FOLLOW-UPS.md`** — durable register of deferred items (the OWN-11 loop-spawn
  carve-out, the AMD-6/7/8 consolidation, the MM discharge-obligation relocation).

## The design

- `DESIGN-COMPARISON-AND-RECOMMENDATION.md` — the four-design competition + judge
  that selected the three-tier architecture; carries the preregistered M1-M10
  validation ladder (§8).
- `SCENARIO-DEMAND-MAP.md` — the 9-family / 51-scenario coverage target.
- `CATALOG-V1-RECUT.md` — the D16 minimality re-cut to ten sealed parts.
- `MEMBER-AUDIT-THREADS-IO.md` — member-level audit of the threads/par and io pockets.
- `IO-ROW-ENUMERATION.md` — the 17 enumerated io-file rows + the Darwin F_FULLFSYNC pin.

## The kernel-rule layer

- `m1-loan-judgment/` — the ratified 15-rule loan/freeze judgment: `RULES-RATIFIED.md`
  (normative), `M1-PAPER-RESULT.md` (rules + review history), `AMENDMENTS.md`, and the
  machine-checked reference checker (`checker.py`, `programs_ast.py`, `run.py`,
  `mutants.py`) with its 97-program corpus.
- `m2-spec-mass/` — the catalog draft (`optables.md`, `cards.md`, `HANDOUT.md`),
  the six kernel deltas (`KERNEL-DELTAS-DRAFT.md` = the review record with rationale
  and attack walkthroughs), `conc-normative.md` (the always-loaded concurrency
  extract), and the reduced-variant token studies. Always-loaded set: ~46.4k / 48k.

## Evidence

- `evidence/` — raw agent outputs (the four designs, twelve attacks, judge), the
  writer-trial rounds (`m5-*.json`), the review JSONs, the spec-mass counts, and
  the microbench results.
- Dry runs (C/Rust, indicative, Apple M4 — validate shape, not deploy magnitudes):
  `m3a-kernel-dryrun/` (seq/table vs Vec/hashbrown), `m6a-spsc-dryrun/` (SPSC model
  check + zero-RMW), `m8-memchr-dryrun/` (checked==unchecked asm), `m4-arena-dryrun/`
  (check-free deref).

## Superseded predecessor

The earlier B-Strata / candidate capability-research era (the 179MB
`minimal-systems-capability/` folder and its verifier tools) is archived at
`/archive/research/minimal-systems-capability/` as historical evidence. The
capability-floor research that preceded this pass is at
`/archive/research/capability-floor/`.
