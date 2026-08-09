# 0042 — ENT-5 re-cut and the archive-integrity gate

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** lead rulings of 2026-08-09 in `governance/APPROVALS.md`
  (the corrected archive gate resolves from stable-file existence and its own
  version token; O11 comes out of this activation; condition 8 and the
  grandfathering of v0.23), over the owner-approved
  `governance/spec-evolution/stable-spec-filename-proposal.md`
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`,
  branch `codex/0042-ent5-gate`
- **Base revision:** `5a3ced2`
- **Dependency:** v0.23 activation (0040) terminal

Registered before substantial work because this slice was briefly claimed by
two agents at once. A task record is the mechanism that prevents that, and its
absence is what allowed it.

## Scope

Steps 1 to 3 only. **The activation is not in this task.**

1. `spec-archive-integrity` learns the stable-file model, on the current
   model, its own commit, green. — **DONE on the task branch** under the
   corrected ruling at `5f729d8`; landing commit pending.
2. Re-cut `governance/spec-evolution/ent5-loop-fix-v024-candidate.md` against
   v0.23 — non-authoritative, its own commit. — **DONE**, landed as `7009434`.
3. Re-verify the anchor after those edits and report the digest. The lead
   recomputes it and takes it to the owner. — **DONE**; digest reported.

### Step 2 / 3 results

- Re-cut digest, to be recomputed independently before it is used:
  `9afd7fd57390b688ba0a2c7d91573d9d2cd3cbb8a8244a440e9120b73f73481e`.
- Anchor, re-checked **after** the edits: **whole-line exact**, exactly one
  match, `spec/kernel-spec-v0.23.md` line 1053. Upgraded from a substring
  test, which would have passed even if the specification's paragraph had
  grown a clause this document does not know about.
- **A figure I reported earlier was wrong and is corrected here.** The anchor
  is **547 bytes**, not 470. The earlier extraction was one wrapped line
  short. The conclusion it supported was unaffected — a substring matching
  once implies the superstring matches at most once, and it does match once —
  but the number was wrong and went into a draft before it was caught.
- Grammar preserved, **measured rather than asserted**: the v0.24 document was
  assembled in scratch (v0.23 with line 1053 replaced) and verified against
  v0.23 by the two-path verifier — exit 0, 69 productions / 84 decisions / 93
  terminal predicates, unchanged. The check was then proved capable of
  failing: a one-token break to `if_stmt`'s production reds it with exit 1.
  The first attempt at that break changed **zero** lines, so its green tested
  nothing; the diff count is what caught it, and is now the guard.

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

## Step 1 ruling and implementation state

The original line-prefix discriminator was unsatisfiable and is retained below
as diagnosis, not as the live direction. The lead corrected the ruling at
`5f729d8`: file existence selects the current layout, and a present stable file
names its own recorded version. The gate implementation and its two-direction
mutation proof were the only remaining work in this task and are now complete
on the task branch.

The implementation stays in the existing `Makefile` target. It strictly parses
and globally deduplicates both ledger record forms before using their tokens,
rejects non-regular specification paths, accepts the current 24-file layout and
the synthetic one-stable-file layout, and checks the stable file's exact title,
version, and digest. In an isolated copy both green baselines exited 0. The
following mutations each exited 2 at the named invariant and were restored
before the next run: recorded file missing; unrecorded versioned file present;
stable file missing; outgoing archive missing (two unarchived versions); wrong
stable version; wrong stable digest; malformed stable title; malformed ledger
record; and a directory at the stable path. The restored synthetic stable
layout exited 0 again.

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

**Resolved by `5f729d8`. No ledger line will be written to make a gate green.**

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
