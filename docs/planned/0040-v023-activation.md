# 0040 — v0.23 activation

This is a temporary live coordination record, not execution authority.

- **Status:** `PLANNED` (unclaimed)
- **Authority:** the owner's byte-exact approval of the v0.23 candidate,
  which does not yet exist and is a precondition of this task
- **Owner / workspace:** unclaimed / filled at claim
- **Base revision:** filled at claim
- **Dependency:** FLOOR-5's M3b, M3c and M4 terminal; owner approval

## Goal

Install the approved v0.23 bytes and bring every derived artifact to them,
in one commit, gates green at the end.

## The landmine, found before it fired

`compiler/src/backend/qualification.rs` carries three guards written as
`ACTIVE_KERNEL_SPEC_VERSION != "v0.22"`. At v0.23 all three fire and
**fail closed to `MissingMapping`**, so every system operation and the
command entry silently lose qualification. They have moved at every bump
since v0.19. Repoint them; do not assume the version-labelled pins are
only the obvious ones.

## Steps

1. Owner exact-byte approval of the candidate digest, recorded in
   `governance/APPROVALS.md` as its own entry.
2. Install the candidate bytes as `spec/kernel-spec-v0.23.md`, verified
   byte-identical with `cmp`.
3. Repoint every pin: `compiler/src/spec.rs` (path, `include_str!`,
   version, and the recorded hash byte array at ~line 60),
   `compiler/src/bin/spec.rs`, `tests/conformance/runner.py`,
   `spec/derivation/derivation-ledger.md`, `docs/roadmap.md`'s authority
   line and revision, and the three `qualification.rs` guards above.
4. **Regenerate the grammar tables — do not copy the file.** The
   generated header embeds the source specification path, and that line is
   inside the derivation check, so a copy-only activation reds the gate.
5. Append the chained `ACTIVE-SPEC: v0.23 <new> <previous>` line to
   `governance/APPROVALS.md`; `whitefoot-spec` asserts it matches the
   computed digest of the embedded bytes and chains to its predecessor.
6. Delete the SUPERSEDED candidate (v0.22's) once the installed bytes are
   verified identical; retain the newly active version's candidate, which
   the `APPROVED_CANDIDATE` comparison consumes.
7. Both gates exit 0, read directly. The three activation-gated checks
   (`path_and_version_label_agree`,
   `computed_identity_is_the_approved_digest`, and
   `recorded_chain_ends_at_the_embedded_specification`) close here and
   nowhere else — they are red on the branch by design.
8. Move `docs/ongoing/0036` and `docs/ongoing/0038` to `docs/done/`.

## Not in this task

The stable-filename switchover (`spec/kernel-spec.md`) is approved but
rides the NEXT small activation — the ENT-5 loop-rule fix — never this
one, which is a large EBNF-changing batch. See `governance/APPROVALS.md`
2026-08-07 and `governance/spec-evolution/stable-spec-filename-proposal.md`.

## Ready state (lead, 2026-08-09) — everything below is verified, not assumed

**The approved bytes.** The candidate is final and its digest was recomputed by
the lead on `main` rather than copied from any report:

```
shasum -a 256 governance/spec-evolution/kernel-spec-v0.23-candidate.md
e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5
```

Three pins already agree with it — `compiler/src/spec.rs` (as a byte array,
which a hex grep does not see; decode it to compare), `spec/derivation/
derivation-ledger.md`, and `tests/conformance/runner.py`. Four independent
derivations, one value.

**The landmine is defused for this activation and WILL RECUR at the next one.**
The three guards at `compiler/src/backend/qualification.rs:762,804,862` now read
`ACTIVE_KERNEL_SPEC_VERSION != "v0.23"`, so they pass here — but they hard-code
the version string, so **every future activation must update them or they fail
closed silently**. They are safe rather than unsound when stale (the guarded
path bails), which is exactly why nobody notices. Treat this as a standing
activation checklist item, not a v0.23 finding: the ENT-5 activation will meet
it again at v0.24.

**Gate state entering activation**, measured on `main`: lib 572 passed / 3
failed, of which **two are the activation-gated `spec::tests` that this commit
closes**; the third is `semantic::tests::borrows::general_borrows_…`, a
pre-existing `RegionsAndBorrows` capability gap. Conformance adapter 389 / 1 /
13, the single failure being `own3-pos-outlives-store` — the A3 counterexample
the approved bytes now name as a removed expressible form. Coverage 128/128.

**What the activation commit does**, from the approved stable-filename
proposal's §5, which sequences this activation the OLD way deliberately: install
the approved bytes at `spec/kernel-spec-v0.23.md` (versioned, because a 34-rule
EBNF-changing corpus-migrating activation must not be paired with a file-model
change); append the exact-byte approval entry and the `ACTIVE-SPEC:` chained
line; repoint the roadmap's authority line and revision; regenerate the grammar
tables, whose header embeds the source basename. **The `ACTIVE-SPEC:` line is an
owner approval record — writing one to make a gate green is forbidden.**

The stable-filename switchover does NOT ride this activation. It rides the first
activation with no EBNF change, which is the approved ENT-5 loop-rule fix.
