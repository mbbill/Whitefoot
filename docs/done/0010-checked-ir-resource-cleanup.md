# 0010 — Checked-IR resource identities and cleanup

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main (cherry-pick onto post-v0.19 tip),
  2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 4

## Outcome

The checked program and typed IR carry the system facts lowering and
auditing need: QUAL-1 semantic IDs on system calls (the SYS-2 ordinal, never
a source name), per-type `SystemResourceContract` (action, row, backing —
including HOST-3's lease retention) with `system_release_row` derived from
it, explicit release records on every drop (`CheckedDrop`/`IrDrop::release`)
as the single source of truth that 0009's EFF-2 attribution now reads, the
SYS-12 stdout/stderr may-alias retained on `IrEntry::Command`, and EFF-5
order preserved structurally. One reviewed design ruling: identity is
type-level plus affine move tracking — the record's own Direction makes a
per-value identity graph exactly the alias machinery the first slice
forbids; the may-alias record is the one retained non-derivable fact. The
unsupported stop moved to the backend
(`BackendFailure::UnsupportedSystemInterface`, raised before layout); 0011
removes it as the qualification table lands.

## Evidence and validation

- Landed commits: `5f6cb1f` (claim), `1ccec9c` (implementation),
  cherry-picked onto the v0.19 tip with the `position` spelling verified and
  both gates green by unpiped exit codes; lib tests 380 → 391, program
  witnesses 18/18, coverage 119/119.
- The five accept-expectation conformance cases correctly remain pending on
  0011/0012 (only their reason prose moved; verified programmatically).

## Follow-ups

- 0011 consumes the contract/ordinal surface and removes the backend stop
  for qualified operations; 0012 completes native emission; whoever closes
  0012 re-reads the five pending accept-case reasons.
