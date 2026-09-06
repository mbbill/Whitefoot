# 0063 — Stage 8b real-consumer migration and behavior oracles

- **Status:** `DONE` (2026-08-15)
- **Authority:** the owner-approved ACTIVE Current Plan, Workstream 8b under
  Direction Outline item `PROOF-8`
- **Reviewed H5:**
  `5fd017b46973e5cbf990fe3fc92a2cc20a76f91c`

## Outcome

The held stack executed this handoff before its planned coordination record was
claimed; activation records the actual result without fabricating claim
history. H5 migrated exactly the four raw-DEFLATE sources and wfgrep. Fourteen
`read_bits` calls use the verified selected-payload receiver, twenty calls use
two distinct verified `append_slice` declarations, and only wfgrep A10 uses
the bounded `value_if` repair. Output, error, cleanup, effect, status,
required-check, and invalid-domain behavior remain unchanged.

The exact 14/20/A10 owning test passed, both clause-stripped invalid-domain
controls passed, and formatting, all-target checks, warnings-denied lint, and
independent final review were green. The installed five source identities are
owned by the Current Plan and acceptance record. The atomic v0.28 activation
containing this record installs H5.

Stage 9a is the next execution item. This file is frozen coordination history,
not current authority.
