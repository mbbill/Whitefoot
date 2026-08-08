# Stable active-specification filename — law amendment proposal

Status: **APPROVED 2026-08-07** (`governance/APPROVALS.md`, "approval (stable
active-specification filename)"), with all eight mandatory amendments below as
conditions of adoption; condition 2's implementation form was subsequently read
down in the same ledger on measured grounds. **Not yet switched over**: by §5 of
this document, v0.23 activates the old way at `spec/kernel-spec-v0.23.md`, and
the switchover rides the first activation with no EBNF change — the approved
ENT-5 loop-rule fix. The line below said "AWAITING OWNER APPROVAL" until
2026-08-08, contradicting the ledger for a day; corrected rather than left,
because a document whose own status disagrees with the record is the defect
this batch spent a day finding elsewhere.

This amends
project law (`CLAUDE.md` / `AGENTS.md`, which must stay byte-identical) and
the specification-change workflow (`docs/WORKFLOW.md`). Nothing in it takes
effect until the owner approves and the switchover commit lands.

Origin: owner proposal 2026-08-07, adversarially reviewed on four lenses
(append-only guarantee, approval binding, tooling and pins, switchover and
history) with the judgment recorded in this session. The judgment was
**adopt with amendments**; every amendment it made mandatory is carried
below. Two of the proposal's supporting claims were refuted by measurement
and are corrected here rather than repeated.

## 1. What changes

The active specification lives at one stable path, `spec/kernel-spec.md`.
A language change edits that file directly on a task branch. At activation,
in one commit, the superseded bytes are archived under their versioned name
`spec/kernel-spec-vN.md` — **flat, beside the twenty-three that already
exist, never in a subdirectory** — and the version line inside the stable
file is bumped. Candidate files cease to exist; review is `git diff`.

## 2. Why

- **The stale path-pin class is structurally eliminated.** A pinned path
  plus a pinned digest stay mutually consistent forever when the target is
  immutable — which is exactly how `tests/conformance/runner.py`'s pin sat
  at v0.20 through two activations while its own digest check passed. Under
  one stable path a stale digest raises instead of agreeing.
- **`git log -L` and `git blame` work across versions for the first time**,
  at paragraph granularity.
- **The hand-assembly failure class disappears** for the 57-anchor
  scratch-script assembly recorded in `docs/ongoing/0036`.

Corrected claims, stated so they are not repeated as justification:
`git diff` does **not** subsume a rule-partitioning review tool — measured,
line-start attribution catches 11 of 20 changed rules and `xfuncname` hunk
headers name the wrong rule 52% of the time, always the preceding one; that
tool must still be built. And "no diff today" is overstated: `git show
--stat -C --find-copies-harder` already renders a version step as a rename
plus diff. The defensible claims are that it is not automatic and that line
history cannot cross a version boundary.

## 3. Mandatory amendments (each is a condition of adoption)

1. **Flat archive.** The hook's pathspec `spec/kernel-spec-v*.md` matches
   only flat files; the obvious widening `spec/*kernel-spec*.md` would
   match the stable file itself and block all specification work. Flat
   requires zero change to the `Makefile` targets, zero change to
   `governance/hooks/pre-commit`, keeps 144 existing citations valid, and
   avoids a migration commit the hook would reject (a `git mv` of a
   released spec stages as `R100`).
2. **Computed digest.** `ACTIVE_KERNEL_SPEC_HASH` becomes a const-fn
   SHA-256 over the embedded bytes rather than a hand-typed array whose
   only test compares it to its own hex literal.
3. **Chained approval record.** Each activation appends one strict line to
   `governance/APPROVALS.md`: `ACTIVE-SPEC: <version> <sha256-new>
   <sha256-previous>`. The compiler asserts the last line's digest equals
   the computed digest of the embedded bytes, its version equals both
   `ACTIVE_KERNEL_SPEC_VERSION` and the version token on the spec's first
   line, and its previous-digest equals the preceding line's digest. This
   replaces both mitigations the proposal offered for the version-label
   risks: those compare labels to labels and pass on any bytes carrying
   the right version string, while a digest chain does not.
4. **Landed-state archive integrity in `make check`.** For every recorded
   `(version, digest)` pair, the archived file must exist and hash to it.
   `pre-commit` is bypassable by `--no-verify`, by merge commits, and by a
   clone whose `core.hooksPath` points elsewhere, so the landed check is
   the real guard.
5. **Two-path grammar verifier.** `whitefoot-grammar` currently compares a
   candidate against baked-in active bytes; once the candidate *is* the
   active file that comparison is `X != X` and the mandatory verifier
   passes on every input. It must take a baseline path and a candidate
   path, both read at runtime.
6. **Linear activation only.** The activation commit must be a linear step
   from the previously approved bytes, which the digest chain enforces.
   `-X ours` / `-X theirs` on any change touching the stable file is
   forbidden: it was measured silently dropping an owner-approved rule
   change with both proposed gates green. Concurrent drafting stays free;
   a rebase changes the digest and therefore requires re-approval, which
   is correct behaviour rather than friction.
7. **Archive-creates-or-fails.** The archive step must create
   `spec/kernel-spec-vN.md` and fail if that path already exists,
   preserving the free path-occupancy collision detector that a content
   merge cannot replace.
8. **Status word inside the approved bytes.** The spec's status line
   becomes part of the approved bytes, so it must read `Status: ACTIVE vN`
   before approval, and the file is never edited after approval.

## 4. Law amendments required

- `CLAUDE.md` and `AGENTS.md` (byte-identical): the sentence naming `spec/`
  as the append-only exception, and the specification-integrity bullet
  stating that a released `spec/kernel-spec-v*.md` is never edited,
  renamed, or deleted, are amended to distinguish the mutable active file
  from the immutable archive: **the archived versioned specifications
  remain absolutely immutable and the hook continues to enforce exactly
  that; the active file at the stable path is mutable by design, and its
  integrity is enforced by the computed-digest chain and the landed-state
  archive gate instead of by the filename.**
- `docs/WORKFLOW.md` step 2 (currently "Copy the active spec to
  `governance/spec-evolution/kernel-spec-vN-candidate.md` and apply the
  smallest complete change") becomes: edit `spec/kernel-spec.md` on the
  task branch; the review artifact is `git diff`; the approval artifact is
  the digest of that file at the approving commit. Step 3's grammar
  verification command takes the two explicit paths.
- `spec/derivation/derivation-ledger.md`'s binding language names the
  stable path plus the version, not a versioned filename.

## 5. Switchover procedure

Prerequisites, landed and green first, each valuable on its own merits and
independent of this proposal — registered as task 0039: repair the three
existing tautologies; computed digest; chained approval record; archive
integrity gate plus `pre-merge-commit`; two-path grammar verifier.

Then:

- **FLOOR-5 / v0.23 activates the old way, unchanged.** A 24-rule,
  EBNF-changing, corpus-migrating activation must not be paired with a
  file-model change.
- **The switchover rides the first small activation with no EBNF change** —
  concretely the approved ENT-5 loop-rule fix. It must ride an activation:
  switching between activations would put one version's bytes at two paths
  at HEAD, which is the parallel-versions defect the hygiene rule forbids.
  Riding an activation, the previous version is already the flat archive
  and the new stable file duplicates nothing.
- That single commit adds `spec/kernel-spec.md` with the approved bytes;
  repoints the compiler's path, text, and version constants; deletes the
  now-unnecessary approved-candidate comparison; repoints the conformance
  runner and rewrites (never deletes) the runner tests keyed to the old
  shape; regenerates the grammar tables, whose header embeds the source
  basename; updates the roadmap authority line and revision, and the
  ledger binding; appends both the exact-byte approval entry and the
  `ACTIVE-SPEC:` line; and amends the law files above.
- Every activation thereafter is one commit: archive the superseded bytes
  under their versioned name, edit the stable file, bump the version line,
  append the chained approval line.

## 6. Residue, stated plainly

The active specification becomes mutable by design, so "alter approved
bytes" and "activate a new version" become the same filesystem operation,
and the filename discriminator that separated them with zero semantics is
gone. Protection moves into digest bookkeeping — the class this project has
failed at repeatedly. Amendments 2, 3, and 4 are what compensate; nothing
else does. The newest bytes live for one version's lifetime in a mutable
file with no second copy, so a bad rebase or a `--no-verify` push on the
integration branch during that window loses them. Concurrency moves from
zero-conflict independent files to rebase-and-re-approve, with a guaranteed
conflict on the version line. And the gate proves the file matches the
digest in the approval record, never that the owner read those bytes —
equally true today, and closed only by the owner recomputing the digest of
what was shown.

## 7. Not adopted

`spec/released/` as an archive directory (measured: breaks the hook's
pathspec or blocks all specification work, invalidates 144 citations, and
requires a migration commit the hook rejects); any scheme that switches
between activations rather than riding one; and treating the digest chain
as optional.
