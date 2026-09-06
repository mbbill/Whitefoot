# 0040 — v0.23 activation

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-09)
- **Authority:** owner exact-byte approval of v0.23 at SHA-256
  `e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5`
- **Outcome:** v0.23 is the active language authority at the deliberately
  versioned path `spec/kernel-spec-v0.23.md`. The specification, compiler
  identity, generated grammar datum, conformance runner, derivation ledger,
  outline, and 15-link activation chain agree on the approved bytes.

## Landed work

- `a01bc70` installed the approved candidate byte-for-byte, recorded
  `ACTIVE-SPEC: v0.23`, regenerated the grammar table, repointed derived
  identities, and retained the stable-filename switchover for the smaller
  ENT-5 activation as approved.
- `dd4e767` restored the OWN-4 witness that the A3 migration had emptied,
  closing the last library-test failure without widening language behavior.
- `c0fa300` corrected the independently pinned activation-chain length from 14
  to 15.
- `201c2d3` completed the verification recovery by deriving canonical-corpus
  exclusions from exact typed manifest expectations rather than filenames or
  counts. It changed no protected corpus material.

## Validation and remaining boundary

The installed specification hashes to the approved digest; coverage is
128/128; the compiler library is 575/575; the focused canonical-corpus test is
3/3; and `make check` exits 0 at `1c3a7dc`. The separately invoked ignored
adapter remains an independent `Pass=389 Fail=1 Skip=13` report rather than a
hidden part of that gate. The next approved file-model change belongs only to
the ENT-5 activation: v0.23 stays immutable at its current path, while v0.24
will become the stable active `spec/kernel-spec.md` after complete-byte owner
approval.
