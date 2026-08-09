# 0042 — ENT-5 re-cut and the archive-integrity gate

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** lead rulings of 2026-08-09 in `governance/APPROVALS.md`
  (the archive gate resolves by line type; O11 comes out of this activation;
  condition 8 and the grandfathering of v0.23), over the owner-approved
  `governance/spec-evolution/stable-spec-filename-proposal.md`
- **Owner / workspace:** exec-uninfix / `/Users/bytedance/do_not_scan/wf-uninfix`,
  branch `task/0043-ent5-recut`
- **Base revision:** `bfc78ec`
- **Dependency:** v0.23 activation (0040) terminal

Registered before substantial work because this slice was briefly claimed by
two agents at once. A task record is the mechanism that prevents that, and its
absence is what allowed it.

## Scope

Steps 1 to 3 only. **The activation is not in this task.**

1. `spec-archive-integrity` learns the stable-file model, on the current
   model, its own commit, green.
2. Re-cut `governance/spec-evolution/ent5-loop-fix-v024-candidate.md` against
   v0.23 — non-authoritative, its own commit.
3. Re-verify the anchor after those edits and report the digest. The lead
   recomputes it and takes it to the owner.

## Not in this task

The activation commit, and everything the proposal's §5 assigns to it:
installing `spec/kernel-spec.md`, deleting `bin/spec.rs`'s
`APPROVED_CANDIDATE` comparison, and repointing
`compiler/src/backend/qualification.rs`'s three version guards to `v0.24`.
Deleting the candidate comparison before the stable file exists would leave
nothing checking the candidate at all.

O11 does not ride this activation (ruled at `cea70f2`). It is unapproved,
flips a declared conformance verdict, and touches CLM-2 as well as ENT-3;
the findings are recorded for whoever drafts it.

## Step 1 is blocked, and the blocker is a governance collision

**The ruled discriminator assumes a ledger shape the ledger does not have.**
Measured on `bfc78ec`: `governance/APPROVALS.md` carries **15
`ACTIVE-SPEC:` lines (v0.9–v0.23) and 9 `ARCHIVE-SPEC:` lines (v0.0–v0.8)**,
over **disjoint** version sets.

- `ACTIVE-SPEC:` is the **approved activation chain** — one line per
  activation, retained forever, each carrying (new digest, previous digest).
  It means "this version was activated", never "this version is active now".
- `ARCHIVE-SPEC:` is an **after-the-fact measurement** of the pre-chain
  specifications that never had exact-byte approval. The ledger says so in
  terms, and says they "carry a different prefix from the approved chain
  above" precisely because of that difference in provenance.

Two consequences:

1. **The assertion is false today by 14.** Fifteen versions lack an
   `ARCHIVE-SPEC:` line, not one, and the resolution rule would send all
   fifteen to `spec/kernel-spec.md`, which does not exist.
2. **It cannot be repaired by back-filling.** Back-fill v0.9–v0.22 and
   exactly one version (v0.23) lacks an ARCHIVE line — but then the gate
   resolves v0.23 to `spec/kernel-spec.md`, which does not exist today, so it
   reds. Back-fill v0.23 as well and **zero** versions lack one, so the
   assertion reds instead. **The ruled shape is unsatisfiable on today's
   tree in either direction**, because today no version is at the stable
   path while the assertion requires exactly one always to be.

Separately, re-purposing `ARCHIVE-SPEC:` as a location discriminator would
make one prefix mean two unrelated things, and would give future
activation-written lines — for owner-approved bytes — the prefix whose stated
meaning is "not an approval, measured after the fact".

**Recommended shape, preserving the ruling's intent and its rejection of
order-dependence:** make the assertion conditional on the stable file's
existence, and take the discriminator from the stable file's own version
token rather than from a line prefix.

- `spec/kernel-spec.md` absent: every recorded version must have
  `spec/kernel-spec-<version>.md`. Today that is 24 versions and it is
  exactly the check that already runs — correct and non-vacuous now.
- `spec/kernel-spec.md` present: **exactly one** recorded version must lack a
  versioned file, the stable file's first-line version token must name that
  version, and the stable file must hash to that version's recorded digest.

Zero still means the activation forgot to install the stable file and two
still means it forgot to archive the outgoing one, which is the whole reason
the assertion exists. Nothing depends on which line is last.

**Awaiting the lead's ruling. No ledger line will be written to make a gate
green.**

## Step 2 note — one judgment call outside the listed changes

Condition 8 puts `Status: ACTIVE vN` inside the approved bytes. Every
released specification's status paragraph ends "These bytes are
non-authoritative until the grammar check, derived-material review,
full-document hash, exact owner approval, and active-target installation
complete" — a sentence that is false the moment the bytes are active, and
contradicts `Status: ACTIVE` standing beside it. Restated as a condition on
installation rather than deleted, so the closure record survives. Flagged
because it changes a sentence every prior version carried.

Also noted: the ruling cites the candidate document's own `Status: CANDIDATE,
OWNER-APPROVED` header as condition 8's offender. The line condition 8
actually governs is §2's proposed **specification** version-header paragraph,
which reads `Status: REVIEW CANDIDATE vNEXT`. Both are corrected, so the
conflation changes no action.
