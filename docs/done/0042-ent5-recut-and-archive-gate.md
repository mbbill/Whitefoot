# 0042 — ENT-5 re-cut and the archive-integrity gate

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-09)
- **Authority:** the corrected archive-gate ruling at `5f729d8`, the
  owner-approved stable-specification-filename proposal, and the rulings that
  removed O11 from this activation
- **Outcome:** the ENT-5 delta is re-cut against v0.23 and independently
  anchored; the archive gate now supports both the current versioned layout
  and the approved stable-file layout without using ledger provenance prefixes
  as location markers

## Landed work

- `7009434` re-cut
  `governance/spec-evolution/ent5-loop-fix-v024-candidate.md` against the active
  v0.23 bytes. Its independently recomputed digest is
  `9afd7fd57390b688ba0a2c7d91573d9d2cd3cbb8a8244a440e9120b73f73481e`.
  The exact whole-line anchor occurs once at v0.23 line 1053 and is 547 bytes.
- `8bdc915` recorded the re-cut verification: the assembled scratch v0.24
  preserved 69 productions, 84 decisions, and 93 terminal predicates; a
  deliberate one-token grammar break made the verifier fail.
- `e9e996b` made `spec-archive-integrity` stable-aware. It strictly parses and
  deduplicates both ledger record forms, rejects non-regular paths, and checks
  both directions of the recorded-identity/file mapping. With no stable file,
  every recorded version must have a versioned archive. With a stable file,
  exactly one recorded version lacks an archive and the stable file's exact
  first-line version and digest must identify it.

O11 remains undrafted and unapproved and did not enter this work.

## Validation

- Current layout: `make repository-invariants spec-append-only
  spec-archive-integrity` exited 0; all 24 recorded specifications hashed as
  recorded.
- An isolated stable-layout fixture also exited 0. Nine restored mutations
  exited nonzero at their intended invariant: recorded file missing,
  unrecorded file present, stable file missing, outgoing archive missing,
  stable version mismatch, stable digest mismatch, malformed stable title,
  malformed ledger record, and a directory at the stable path. The restored
  fixture exited 0 again.

## Remaining boundary

This task deliberately excluded activation. The re-cut delta bytes above still
await owner re-approval, and the not-yet-assembled complete v0.24 specification
will require its own exact-byte digest approval. A separately registered task
must then atomically install `spec/kernel-spec.md`, implement continuing-kill
ENT-5 semantics and tests, update every identity and derived pin, and rerun the
frozen acceptance measurement. No `ARCHIVE-SPEC:` line is added for v0.23;
those prefixes record provenance, not location.
