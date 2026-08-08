# 0039 — spec identity integrity: computed digest, chained approval, archive gate

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS` — claimed 2026-08-07.
- **Authority:** owner instruction 2026-08-07 ("开始吧") on the adversarial
  judgment of the stable-filename proposal; the judgment's amendments M2–M6
  and switchover steps C1–C4
- **Owner / workspace:** exec-0039 / `/Users/bytedance/do_not_scan/wf-0039`
  on branch `task/0039-spec-identity-integrity`
- **Base revision:** a375dba
- **Dependency:** none. Deliberately independent of the stable-filename
  switchover, which rides a later small activation (C6); every item here
  repairs a real defect in the CURRENT versioned scheme and is worth
  landing on its own merits.

## Goal

Four steps, in this order, each its own cohesive commit, gates green
(exit codes read directly, never through a pipe) before and after:

1. **C1a — repair three tautologies (M6).** `compiler/src/bin/spec.rs`
   around lines 112-113 compares `X != X || Y != Y` (both operands are
   const aliases of the same value, at `compiler/src/syntax/grammar.rs`
   and `compiler/src/syntax/terminal.rs`);
   `compiler/src/syntax/grammar/tests.rs` asserts the same tautology;
   `compiler/src/bin/spec.rs` compares `ACTIVE_KERNEL_SPEC_HASH` to its
   own hex literal. Verify each is a tautology before touching it, then
   make each compare two independently-derived quantities or delete it
   with the reason recorded. Do not land the later steps beside checks of
   the shape they replace.
2. **C1b — computed digest (M2).** Replace the hand-typed
   `ACTIVE_KERNEL_SPEC_HASH` with a const-fn SHA-256 over
   `ACTIVE_KERNEL_SPEC_BYTES`, in safe Rust, no dependency (the crate has
   none and keeps none). The implementation is checked against the digest
   the owner's `shasum -a 256` produced and recorded in
   `governance/APPROVALS.md`, so a wrong implementation fails loudly
   rather than agreeing with itself.
3. **C2 — chained approval record (M3).** Append one strict machine-
   readable line per activation to `governance/APPROVALS.md`:
   `ACTIVE-SPEC: <version> <sha256-new> <sha256-previous>`. Backfill the
   line for the active version. `compiler/src/bin/spec.rs` (which already
   `include_str!`s the derivation ledger) parses these and asserts: the
   last line's digest equals the computed digest of the embedded spec
   bytes; its version equals both `ACTIVE_KERNEL_SPEC_VERSION` and the
   version token on line 1 of the spec; and its previous-digest equals
   the digest on the preceding line. Fully testable under the current
   versioned filenames.
4. **C3 — landed-state archive integrity (M4).** New `make check` target
   `spec-archive-integrity`: for every recorded `(version, digest)` pair,
   assert `spec/kernel-spec-vN.md` exists and hashes to it. Backfill the
   missing digests in the same change — 14 of 23 are in APPROVALS, 10 are
   recoverable from the spec's own `Prior:` chain, and v0.0–v0.8 have
   none; record those **as-found and labelled as such**, never as
   re-approvals. Add `governance/hooks/pre-merge-commit` running the same
   staged check. Rationale: `pre-commit` is bypassable by `--no-verify`,
   by merge commits, and by a clone whose `core.hooksPath` points
   elsewhere, so the landed-state check in `make check` is the real guard.
5. **C4 — two-path grammar verifier (M5).** `compiler/src/bin/grammar.rs`
   compares a candidate's frontend contract against the baked-in active
   bytes; once the candidate becomes the active file that is `X != X` and
   the mandatory verifier passes on every input. Change `verify_candidate`
   to take a baseline path and a candidate path, both read at runtime, and
   update the command in `docs/WORKFLOW.md` step 3.

## Progress

- Claimed at a375dba; all five defects confirmed by reading the const
  definitions before any change (see the per-step commits).

## Scope and expected touch set

`compiler/src/spec.rs` (+ a new `compiler/src/spec/sha256.rs`),
`compiler/src/bin/spec.rs`, `compiler/src/bin/grammar.rs`,
`compiler/src/syntax/grammar/tests.rs`, `governance/APPROVALS.md`,
`governance/hooks/pre-merge-commit`, `Makefile`, `docs/WORKFLOW.md`.
No `spec/` filename, no `docs/roadmap.md` authority line, no FLOOR-5
candidate.

## Dependencies and integration order

None. Semantic overlap risk: task 0036 (FLOOR-5) will later activate a new
specification version and must then append one `ACTIVE-SPEC:` line rather
than hand-editing an identity constant.

## Validation

`make -C compiler check` and `make check`, exit codes read directly, before
and after each step. The computed digest is cross-checked against
`shasum -a 256 spec/kernel-spec-v0.22.md`.

## Stop condition

A discovery outside this scope stops the task with reproduction evidence.

## Closure

Landed through lead review; no merge commits on the task branch.
