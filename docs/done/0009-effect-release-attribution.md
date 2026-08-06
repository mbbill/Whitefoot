# 0009 — Effect-checking extensions and release attribution

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `2cf0497` (ff), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 3; lead-authorized overlap with 0008 (0008 landed first; this task
  rebased onto it and removed the stops assigned to it)

## Outcome

The complete six-category effect row is checked on the normal path: EFF-1
(`external`/`blocks` payload-free, canonical positions, at-most-once), the
extended EFF-2 exhibited row — syntactic contributions including SYS-2
operation call rows with `operation_region_effects` projected through the
existing borrow-origin machinery, unioned with the release contribution of
every compiler-derived release of a system resource owner on any normal edge
(transitive over owned content; exactly-one-disposition from existing live
tracking; a release-only mismatch reports `ReleaseEffectMismatch` with the
owner and the spec's restructuring text) — and FN-3's six-capability
normalization comparing the two categories by presence. The 14 system
nominals interned into the one checked-nominal path; the SYS-5 release table
joined the catalog with a 167-record round-trip pin. The canonical
`own ReadFile` + `return unit;` case holds exactly in all three directions.
The unsupported boundary moved to lowering
(`LoweringFailure::UnsupportedSystemInterface`); `check_system_surface_support`
is gone. One reviewed method deviation: release contributions are collected
in a post-pass over checked statements rather than at drop sites, keeping the
row comparison pure and giving diagnostics their owner. Five reject cases
flipped pending → runnable; the five accept cases correctly wait for the
lowering chain (0010/0011/0012) with reasons updated. Three latent compiler
defects found and fixed en route (checkpoint pruning, system-callee cycle
edge, a `const`-array-unsound pointer-identity helper).

## Evidence and validation

- Landed commits: `4944f1d` (claim), `1ecff77`/`d1accb4` (implementation),
  `9fd3e71` (conformance flips + pins), `2cf0497` (README/record).
- Gates green on main; 380 lib tests, none regressed; coverage 119/119; no
  expectation byte changed.
- Design memory synced at landing (effects node item + superseding dated
  fact; lint clean).

## Follow-ups

- 0010 consumes `CheckedDrop` records, `CheckedNominalKind::SystemResource`,
  `CheckedExpression::SystemCall`, `system_release_row`,
  `collect_release_sites`, and the lowering stop.
- The positive `arg_get` case remains blocked on the v0.19 rename (task
  0018, at the owner's exact-approval gate).
