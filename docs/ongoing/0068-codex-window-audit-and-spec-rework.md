# 0068 — Codex-window audit repairs and specification representation rework

Owner: lead. Workspace: `spec-rework` branch. Base: `9b6b678` (main, v0.29).
Registered: 2026-08-16, during the owner's overnight delegation of 2026-08-15.

## Authority

Owner conversation 2026-08-15: audit the Codex window `bd6dea4..HEAD` and
repair its findings; research the specification representation problem
(AI-context fit, iteration safety, process cost, conciseness) and implement
the selected design on a branch; direct commits on the branch are authorized,
while landing on main, any activation, and every protected-surface change
remain owner-approval items for the morning review. No `ACTIVE-SPEC:` line,
no archive, and no digest-chain append happens on this branch.

## Scope

1. Repairs for confirmed audit findings (workflow `wf_a7ad69d8-a03`): the
   root-entry law restoration, the two mcts_mem silent divergences, the
   deleted evidence playbook, O11's roadmap entry, the stale provenance
   design-record header, the baked Makefile tally.
2. Specification prose repairs: stale v0.18 self-references, version-tagged
   amendment prose inside rules, Status/Prior changelog eviction.
3. The representation rework selected by the 2026-08-16 research decision
   record (see `research/investigations/spec-representation/`), implemented
   as candidate commits following the freeze-candidate pattern; the spec
   identity gate is expectedly red on this branch until an owner-approved
   activation.
4. A morning review document for the owner enumerating every commit, its
   approval class, and its verification.

## Out of scope

Any semantic language change; any conformance verdict change; activation of
any candidate; merging to main.

## Morning review (written 2026-08-16, end of the overnight run)

Branch `spec-rework`, 18+ commits over main `9b6b678`. Read this section,
then the per-commit messages; every claim below was verified by the lead,
not carried from an agent report.

### Audit verdict on the Codex window

27 raw findings, adversarially verified: **2 confirmed majors, 5 refuted,
20 minors.** The window's governance spine held — digest chain v0.24–v0.29
verified link-by-link against real bytes, archives untouched, every
activation carries a genuine owner approval quote, verdict population
intact. The scariest finding (canonical-corpus exclusion "gutted to all 227
reject cases") was refuted by five independent verifiers and a lead spot
check: once a tree derives, semantic rejects still round-trip, and the
adapter still pins cited rule ids. Confirmed and repaired here:

- `0e34d2c` had weakened the owner's root-entry reservation in CLAUDE.md
  under a delegation that never mentioned it → restored verbatim
  (`4bf6f79`). If the relaxed form is wanted, it is one explicit decision.
- mcts_mem silent divergences: the ENT-5 loop-rule replacement (the
  window's central re-decision) and the v0.25 sole-iteration reversal had
  no move/`.alt` records → both recorded per the skill, lint clean 81
  nodes (`23b61eb`).
- Deleted "failures that look like success" playbook restored to
  `docs/WORKFLOW.md`; O11 restored to the roadmap; stale provenance-record
  header re-headed; baked Makefile tally removed (`894e836`).

Owner-decision leftovers are task #44: the corpus-fitted closed-set spec
residue (S7 ishl-one, S12/S10 asymmetry, one condition in five rules,
PRV-2 witness prose), three plan approvals that exist only as self-written
ledger lines, and the consumerless Stage 9a claim ledger. Task #43 tracks
the gate's ~87s→~1hr wall-time growth.

### The specification representation rework

Decision record: `research/investigations/spec-representation/DOSSIER.md`
(`7ca355b`) — structured markdown profile of the same single file, same
digest/archive/approval model; DSL, sibling formal model, multi-file, and
JSON/YAML containers rejected with reasons. Implemented tonight:

- **Stage 0 tooling** (`dae9f95..196ce0f`, merged `a222d90`): `--index` /
  `--counts` queries; generated `spec_identity.rs` retiring every
  hand-bumped scalar but one deliberate review tripwire; `spec-digest-sync`
  prose gate — which caught the ledger's missed v0.29 update on its first
  run; outline ids prefixed `outline:` ending the rule-id collision;
  **candidate mode** killing the measured 21h36m red window.
- **v0.30 candidate** (`b2b117e`, `a147e1d..0ba366d`, `526225e`): stale
  self-reference and version-tag prose sweeps; header changelog evicted
  (21 Prior paragraphs; history lives in the archives and the chain);
  sentence-per-line (content-preservation proven by a one-shot verifier:
  zero unexplained deltas, then deleted per one-shot law); `[ENT-3.Sk]`
  sub-ids; `wf-` fences with extractors re-keyed; plan vocabulary removed
  from normative text. **431,650 → 384,598 bytes with 126 table rows
  byte-identical and rule count 133 unchanged.**
- **Composition repairs** (`c9bc2eb`, `75899f2`): three new Python tests
  had used the real tree as an ACTIVE fixture — falsified by the candidate
  itself; now synthetic. The runner's sub-id regex had silently widened
  the coverage denominator 133→144; a sub-id is now an anchor folding onto
  its parent, denominator restored, and the gate honestly red-flagged both
  defects before repair.

### Approval matrix for landing on main

- **P1 (protected gate wiring):** `196ce0f`, `526225e`, `75899f2` — exact
  before/after in each message. Nothing binds until your approval.
- **P2 (spec bytes):** the complete v0.30 candidate,
  SHA-256 `db2b4b6906f6309a4fe04568fa5c2beb0fecfae72405591872e6e9c6c70c5ef2`
  at `75899f2`'s tree. No semantic edit rides it; the delta is housing,
  addressing, and prose repair. Activation (chain line + v0.29 archive)
  happens only after your exact-byte approval.
- **Law:** `4bf6f79` restores owner authority (no weakening).
- **Ordinary:** everything else.

### Final gate state

`make check` exit 0 at the branch tip, exit code read directly from the
bare command: candidate v0.30 recognized by the archive gate, 30 archives
hash as recorded, digest sync green, Python suite 27/27, coverage 133/133
with the pre-migration 115-by-case / 30-by-annotation split, compiler
lib 833/833, zero FAILED lines in the log. The gate went honestly red
twice on the way here — fixture tests assuming the tree is ACTIVE, and a
silently widened coverage denominator (133→144) — and both defects were
repaired, not excluded. One process lesson re-learned and recorded: the
first "green" run was an echo swallowing make's real exit 2; exit codes
are now read directly.

### Deferred deliberately

DIAG-1 prose→rows restructure and controlled-vocabulary cells (semantic
risk, daytime work); the conciseness ratchet (≤300 KB target); Stage 2
extraction locks and S-envelope assertions; Stage 3 (executable core)
stays gated default-no.
