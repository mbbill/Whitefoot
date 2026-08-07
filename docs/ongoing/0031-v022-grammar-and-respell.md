# 0031 — v0.22 grammar path and corpus respell (atomic activation prep)

This is a temporary live coordination record, not execution authority.

- **Status:** `BLOCKED` before any compiler edit — item (4)'s mechanical
  reprint names a tool the repository does not have, and the reprint as
  specified cannot be performed by one canonical printer (see Blocker)
- **Authority:** `ACTIVE` `docs/current-plan.md` selected slice; owner rulings
  "批" and the N1 version-compat deferral (2026-08-07); the v0.22 delta
  `governance/spec-evolution/index-surface-v022-candidate.md` and assembled
  `kernel-spec-v0.22-candidate.md`
- **Owner / workspace:** exec-0031 /
  `/Users/bytedance/do_not_scan/wf-0031-worktree`, branch
  `task/0031-v022-grammar-and-respell`
- **Base revision:** 0615f49
- **Dependency:** none (candidates landed; step-4 exact-byte approval follows
  this task's evidence, before installation)

## Goal

On one task branch, 0030-style atomic-activation prep for v0.22:
(1) grammar tables — `index` leaves the fixed atoms (IDENT-eligible),
pbase loses the index alternative, psuffix gains `"[" atom "]"`, brackets
join the right-attachment set per O1; (2) delete the index_get catalog row
and reservation; (3) repoint identity pins to the v0.22 candidate bytes
(rule count stays 128); (4) respell the corpus mechanically via reprint —
tests/programs (266 subscript sites + 84 region headers + 31 cvalue
arrays), tests/conformance (138 sites) with verdicts meaning-unchanged per
the derived-material rule, plus the one new O5 conformance case (`index`
as ordinary IDENT); (5) evidence: verifier green on the branch against the
v0.22 candidate (delta §3 expectations), main untouched and green,
`make -C compiler check` green; (6) STOP before merge — report the
candidate SHA-256 from your worktree for the owner's step-4 approval.
Discoveries outside the candidates stop the task with evidence.

## Progress

Orientation only; stopped before any compiler or corpus edit. No file
outside this record is touched on the branch. Items (1)-(3) and (5)-(6) look
executable as written; item (4) does not, and it sits on the critical path
for (5)'s `make -C compiler check` evidence, so the task stops here rather
than building tables whose corpus evidence cannot be produced.

## Blocker

**The canonical printer item (4) directs me to use does not exist.** The
compiler's canonical machinery is an auditor, not an emitter:
`compiler/src/syntax/parser/finalize/canonical.rs` exposes exactly one
public entry, `audit_canonical` (line 447), which computes expected FORM-2
gap styles and *compares* them against the source bytes through
`bytes_match` and `gap_matches` (`canonical/format.rs`). Nothing in the
crate turns a finalized tree back into source text, `whitefootc`'s complete
option surface is `[--emit-llvm] [-o OUTPUT] SOURCE...` with no format or
reprint mode, and a repository-wide search finds no formatter in
`compiler/`, `tools/` (which does not exist), or the standalone Python
conformance tooling.

**Even given a printer, the reprint as specified is cross-grammar.** Delta
§6 says "the canonical printer computes the new spelling from the old
tree". A canonical printer emits under the grammar its tables carry. After
item (1) lands, `pbase` no longer has the `index` alternative, so a v0.22
compiler cannot obtain a v0.21 tree from the existing corpus; before item
(1) lands, no printer can emit `p[i]` because the subscript `psuffix` does
not exist. Reading old bytes and writing new spelling needs three pieces —
a v0.21 parse, a tree transform from the prefix call form to the subscript
suffix, and a v0.22 emitter — and the delta budgets for one. (Inference,
not measured: post-change `index<u8>(a, i)` in expression position may
still parse as an ordinary call to a function named `index`, since the
respelling makes `index` IDENT-eligible, and would then fail resolution
rather than parsing; in place position, such as `set index<u8>(a, i) = x;`,
it cannot parse at all. Either way no subscript-bearing tree results.)

**Footprint, measured on the branch at 0615f49** — hand-editing is both
forbidden by the card and impractical: 266 `index<` sites in
`tests/programs/*.wf` and 135 in `tests/conformance/` across 50 files, plus
84 region-parameter headers and 31 `= [` cvalue arrays. The conformance
count is 135 here against the delta's 138; the three-site difference is
unexplained and worth reconciling before any migration runs.

**A fourth item the card's six do not mention.** The semantic checker reads
indexing as a prefix on the base — `has_fixed(pbase, FixedTerminal::Index)`
— at five sites across `semantic/check/expressions.rs`,
`expressions/places.rs`, `expressions/flat_storage.rs`, and
`check/requires.rs`. Moving the form to a `psuffix` alternative re-shapes
every one of them, and the per-subscript judgments the delta re-anchors
(O3) land on those same paths. That is real semantic work beyond "grammar
tables", and it must land in the same branch or the reprinted corpus cannot
check.

## Stop condition

Reached. The task resumes when the reprint mechanism is decided: write and
land a migration path (v0.21 parse + transform + v0.22 emit, whose home,
lifetime, and deletion condition are a repository-hygiene decision), rule
that a different mechanism is acceptable, or resequence so the corpus
respell is its own task behind a printer. Whichever is chosen, the card
should also state whether the semantic re-anchoring is in this task's
scope.
