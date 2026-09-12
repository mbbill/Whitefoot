# Sources index for design-decision recovery

Built 2026-09-12. This is a read-only survey of every place on this machine
where a past Whitefoot design decision — a ruling, a measurement, a rejected
alternative, a stated rationale — might still be recorded, for a later pass
to mine into `design/language` and `design/compiler`. Nothing here has been
triaged or judged; it is a map of where to look and how much is there.

## Correction (2026-09-12): the shallow clone, and the real history

The first pass of this index was built from a **shallow clone** — a
`.git/shallow` file recorded seven boundary commits, and the oldest one,
`65b3d24` (2026-08-28), consequently showed as a parentless root importing
3,134 files in one diff. That was purely a shallow-clone artifact, not a
real bulk import: the clone has since been completed with `git fetch
--unshallow origin`, and every claim in the previous pass that rested on
"one root commit at 2026-08-28" or "git only helps for the last two weeks"
is corrected below. The old, wrong framing has been rewritten in place
throughout this file, not kept and struck through, and not merely appended
to — a reader should not need to hold both versions in mind at once.

**The real shape of the history.** `HEAD` on
`claude/code-space-constraints-mdagkl` has **2,376 commits**, with a real,
parentless root at `7c1d7641` (2026-07-07, "Initial commit: xlang — an
optimizer-first language for AI-written code"). **The project was originally
named `xlang`**, with the compiler command `xlc` and source extension `.xl`;
it was renamed **`xlang` → `Whitefoot`** in one deliberate commit,
`601577c0` (2026-07-17, "chore: rename project xlang -> whitefoot"),
immediately followed by `4b7d62af` ("rename compiler command xlc -> wfc and
source extension .xl -> .wf") and `f45d67fc` ("standardize brand name as
Whitefoot"). So the project has three eras by name only, not by substance:
xlang (2026-07-07 – 2026-07-17, 10 days), then Whitefoot throughout. A
second real (non-artifactual) reorganization landed on **2026-07-22**,
commit `51a3e807` ("Restructure: from-scratch 9-folder layout, co-located
gate scripts") — this is the point at which most of the current top-level
layout (`docs/`, `research/`, `governance/`, the `mcts_mem/`-adjacent
structure) took its present shape; material from before that date often
lived under different, now-vanished paths (`optimizer-language-research/`
has 257 historical commits touching it, for one example), so a **current**
path's own `git log <path>` frequently starts at 2026-07-22 even though the
underlying work is older — see the per-source notes below for which paths
this affects. Commit bodies from the whole span, old paths included, are
fully present in the regenerated `commits.md`; only a *path-scoped* history
query undercounts, never the flat commit stream itself.

**This changes the primary conclusion of the first pass.** Commit messages
and bodies are **not** a two-week tail — they are a **complete, continuous,
richly-detailed primary source running the full 2026-07-07 to 2026-09-12
span** (verified by reading the root commit and its immediate successors:
the same dense, rationale-and-evidence commit-body discipline is already
present on day one). `commits.md` should be weighted as a primary source on
a par with `mcts_mem/` for the whole project history, not treated as
secondary to it for anything before an artificial cutoff.

**One real (non-artifactual) divergence worth knowing.** `HEAD` and
`origin/main` have themselves diverged: their common ancestor is `395042f4`
(2026-09-10, "Merge pull request #33 from mbbill/codex/compiler-scheduling-controls").
From there, `origin/main` went on to merge PRs #34 through #47 (90 commits
not present on `HEAD`), while `HEAD` carries 37 commits of its own that
`origin/main` does not have (this branch's own recent work, including this
recovery effort). `commits.md` is `HEAD`'s history only, per the task's own
instruction — for the finer commit-by-commit detail behind PRs #34-47 (as
opposed to their squashed/merged PR-body narrative, which **is** fully
captured in `pull-requests.md` regardless of branch), a reader would need
`git log HEAD..origin/main` separately; that is not reproduced in this
survey.

**`git fetch --unshallow` also surfaced 133 remote branches** (`git branch
-r`), most never seen before. 75 of them are not merged into `origin/main`
— unmerged, and largely un-mined, work. See the new §15 below.

**This branch is live and shared.** A concurrent module-audit effort (see
`design/recall-tmp/audit/`, not this survey's concern) committed twice to
`HEAD` while this correction was being written, which is why the commit
count above is 2,376 rather than the 2,374 first reported by the
coordinator — both are correct, at different, close-together moments.
`commits.md` reflects `HEAD` as of this file's own last regeneration; a
reader working later should expect a small further drift and can always
regenerate it with the one-line command in §1.

GitHub pull requests are unaffected by any of this (they were always fetched
from GitHub's API, never derived from local git depth) — see the
re-verification note in §2.

## At a glance

| source | entries | date range | git-mineable? |
|---|---|---|---|
| `design/recall-tmp/sources/commits.md` | **2,376 commits** | **2026-07-07 .. 2026-09-12** | yes, fully — the whole span |
| `design/recall-tmp/sources/pull-requests.md` | 48 PRs (#1-#48) | created 2026-08-29 .. 2026-09-12 | yes, fully (GitHub-sourced, unaffected by the shallow clone; re-verified, see §2) |
| `archive/governance/decision-log.md` + `decisions/` | 1 index + 15 dated volumes | 2026-07-01 .. 2026-07-22 | only from 2026-07-22 (the volumes' own commit date); pre-07-22 content is a one-time compiled snapshot, but the events it compiles are independently in `commits.md`'s 2026-07-01–07-22 span |
| `archive/done/` | 103 batch records (numbered 0001-0108, 5 gaps) | 2026-08-04 .. 2026-09-06 | yes, richly — 223 commits under the pre-archive path `docs/done/`, 2026-08-05 .. 2026-09-06 |
| `archive/APPROVALS.md` | 125 dated ledger entries | 2026-07-18 .. 2026-09-05 | **yes, richly** — 195 commits under its pre-rename path `governance/APPROVALS.md`, 2026-07-18 .. 2026-09-06 |
| `research/investigations/` | 30 topic directories | mostly 2026-08 .. 2026-09-11 (internal) | yes from 2026-07-22 onward (314 commits); older investigation work lived under other, now-gone paths, findable by full-text search of `commits.md` |
| `research/experiments/` | 25 topic directories | 2026-07-08 .. 2026-09-12 (internal) | yes from 2026-07-22 onward (97 commits); same caveat as investigations |
| `research/notes/` | 9 files | 2026-07-08 .. 2026-08-03+ | mostly no — a one-shot 2026-07-22 snapshot (4 commits total); the originals it compiles are older and live in `commits.md`'s early span instead |
| `governance/spec-evolution/` | 10 candidate documents | 2026-08-06 .. 2026-09-03 | yes from 2026-07-22 onward (88 commits) |
| `mcts_mem/` | 131 node files, 776 dated fact/move lines | 2026-07-02 .. 2026-09-11 | **yes, richly** — 167 commits, from 2026-07-12 |
| `docs/{roadmap,bargain,ideas,why-whitefoot}.md` | 4 reference documents | content back to 2026-07-01 | **yes, richly** — 316/12/11/25 commits respectively, `roadmap.md` from 2026-07-10 |
| `spec/kernel-spec-v0.0.md` .. `v0.52.md` | 53 archived specs | 2026-07-08 .. (v0.52, undated one-liner) | yes — 200 commits under `spec/`, from the true root 2026-07-07; each version's own introduction is its own dated commit |
| `compiler/src/**` doc comments | 10 "ruling", ~183 "owner" (comment lines), ~159 "measured", ~59 "rejected" | scattered, code-attached | yes, via git blame per line |
| **§15 (new): remote branches not merged into `origin/main`** | 75 branches (60 collapse into one archived cluster) | tips 2026-08-25 .. 2026-09-12 | yes, fully (they're commits too) |

Chat logs — the interactive back-and-forth that produced these commits and
PR bodies, as opposed to the resulting written artifact — are **not on this
machine** at all, for the full 2026-07-07 to 2026-09-12 span; see the
closing section.

---

## 1. `design/recall-tmp/sources/commits.md` (regenerated from the unshallowed history)

**Contents.** Every commit reachable from `HEAD` on this branch, oldest
first, one `## <short-hash> <date> <subject>` heading per commit followed by
its full body verbatim (nothing summarized, nothing skipped).

**Entries / date range.** **2,376 commits, 2026-07-07 to 2026-09-12.**
29,295 lines, 1,508,468 bytes (~1.44 MB — much larger than the first pass's
440-commit, 295KB file; this is now the largest single file in the survey).
The first entry is the genuine root, `7c1d7641`, "Initial commit: xlang — an
optimizer-first language for AI-written code" — not a bulk import, an
ordinary first commit with an ordinary body describing what it contains
(the constitution, kernel spec v0.4.1, a checker prototype, the "democ" demo
compiler, and the early research corpus). Every one of the 2,376 commits is
a real, individually-authored decision-sized unit; there is no artificial
boundary anywhere in the file.

**Evidence kind.** This is the single richest *rationale* source in the
entire survey, for the *entire* project history, not just its last two
weeks: commit bodies consistently state the owner ruling being implemented,
the alternative it replaced, the measurement that selected it, and which
spec rule or `mcts_mem` node it touches, from the very first day. The
discipline does not degrade going backward: the root commit and its
immediate successors (e.g. `e687100a`, 2026-07-08, "spec: v0.5 Tier-0
errata + v0.6...", or `6d8c04e0`, 2026-07-08, "conformance: full 89/89 rule
coverage...") are just as dense with rationale, evidence, and named
findings as anything from September. Two eras worth knowing when reading:
before `601577c0` (2026-07-17) the project is called `xlang`, its compiler
`xlc`, its source extension `.xl` — searching for "Whitefoot" or `.wf` will
silently miss the first ten days; and before `51a3e807` (2026-07-22) a good
deal of content lived under paths that no longer exist (e.g.
`optimizer-language-research/`, `docs/done/` before it became
`archive/done/`), so a path-scoped `git log` on a *current* path can
undercount where a full-text search of this file will not.

**Fastest search.** It's flat text with one heading per commit, so plain
`grep -B2 -A15` around a keyword works well:
```
grep -n '^## ' design/recall-tmp/sources/commits.md   # jump table of all 2,376
grep -B1 -A20 -i '<keyword>' design/recall-tmp/sources/commits.md

# to regenerate against a later HEAD on a live branch:
git log --reverse --date=short --format='## %h %ad %s%n%n%b' > design/recall-tmp/sources/commits.md
```
Useful keywords already known to recur: `owner ruling`, `rejected`,
`selection ground`, `measured`, `CANDIDATE`, `ACTIVE v0.`, `superseded`,
and (for the xlang era specifically) `xlang`, `xlc`, `democ`. To jump
straight to the two named eras: `grep -n '^## 601577c0\|^## 51a3e807'`
finds the rename and the reorganization respectively.

---

## 2. `design/recall-tmp/sources/pull-requests.md`

**Contents.** Every pull request on `mbbill/whitefoot` (fetched via the
GitHub MCP `list_pull_requests`/`pull_request_read` tools, `state=all`,
paginated to exhaustion), oldest-created first: `## #<number> <title>`, a
one-line metadata field (state, merge date if merged, head branch, base
branch, created/closed dates), then the full description body verbatim.

**Entries / date range.** **48 PRs, #1-#48** (pagination confirmed no PR
exists past #48 at re-verification time — the task brief's "around 60" was
an overestimate). Created 2026-08-29 to 2026-09-12; 43 are closed (41
merged, 2 closed without merging — #2 and #24, both explicitly shelved
research/refactor branches), **5 still open** at re-verification time: #26,
#28 (both draft/non-draft research on the compute/io runtime), #30
(container-foundation research, draft), #35 (draft — head branch
`claude/code-space-constraints-mdagkl`, **this session's own branch**,
titled "Add the design trees and their procedure, migrate mcts_mem, and
retire the compiler README" — i.e. this very recovery effort already has an
open PR upstream of it, worth reading before publishing new tree changes),
and #48 (amends the #46 compute-regression gate so it can be made a
required check). 2,360 lines, 260,759 bytes.

**Re-verification (requested after the shallow-clone correction, since the
PR set is fetched from GitHub and should be independent of local git
depth).** Confirmed independent, but **not static**: re-fetching found #45
and #47 — both open at the first fetch — had since been merged (2026-09-12,
05:14 and 05:31), and a new PR, **#48**, had been opened in the interim
(05:19). All three entries in this file now carry the re-verified data;
#45's body text itself changed too (it gained a paragraph describing #47's
trace instrument once #47 merged into it — PR bodies are living documents,
not fixed at creation). Every other PR (#1-#44, #46) was re-checked by
number/state and found unchanged since the first fetch. The practical
lesson: **this file is a snapshot as of the stated fetch times, not a
live view** — an open PR's state, and even a merged PR's body, can still
move.

**A data quirk worth knowing before mining this file:** the GitHub list
endpoint's `merged` boolean was **wrong on every single row** (always
`false`), verified against `pull_request_read get` on PR #1, which correctly
reports `merged: true`. This file's `Merged:` field was computed from
`merged_at` instead (cross-checked against `pull_request_read` and found
reliable), so trust this file's field, not a fresh naive re-fetch that reads
`merged` literally.

**Evidence kind.** These are the most polished decision narratives in the
whole repository — most bodies are full batch/PR reports written for owner
review, with an explicit "why", a verification section, measured
performance tables, and (for spec-changing PRs) the exact rule delta and
selection ground. Several PRs are pure design documents with no code change
at all (#2 proof-replaces-claim design corpus, #12/#15/#18 containers-and-
resources design rounds). PR bodies are also where the `governance/APPROVALS.md`
naming (before it was retired and renamed `archive/APPROVALS.md` on
2026-09-06) is most frequently explained.

**Fastest search.**
```
grep -n '^## #' design/recall-tmp/sources/pull-requests.md     # jump table
grep -B1 -A25 -i '<keyword>' design/recall-tmp/sources/pull-requests.md
```
Cross-reference with `commits.md` by branch name (the `Head:` field) or by
session URL (`https://claude.ai/code/session_...`, present in most bodies).

---

## 3. `archive/governance/decision-log.md` and `archive/governance/decisions/`

**Path.** `archive/governance/decision-log.md` (62 lines, 3,310 bytes) is a
short retired index; the real content is the 15 files it points to in the
sibling `archive/governance/decisions/` directory.

**Contents.** The index groups the project's *earliest* history (before the
per-batch `archive/done/` convention existed) into one file per
specification era: `v0.0-v0.4.md` (28KB, "2026-07-01 to 2026-07-07,
foundations"), `v0.5-v0.6-2026-07-08-to-11.md` (56KB), five more `v0.6-*`
day-files (2026-07-12 through 2026-07-18, 12-52KB each — this is the
densest era, "most of the early language and performance research"),
`v0.7.md` (49KB), three `v0.8-*` day-files (2026-07-19/20/21, 13-73KB),
`v0.9.md` (29KB), `v0.10.md` and `v0.11.md` (~4.4KB each, tapering off as
the per-batch record convention took over).

**Entries / date range.** 1 index + 15 dated volumes, spanning 2026-07-01 to
2026-07-22 (the exact day exact-v0.11 became active and this whole log was
archived).

**Evidence kind.** This is the oldest layer of decision evidence in the
repository and predates every other convention (`archive/done/`, `mcts_mem/`,
`APPROVALS.md`'s later form) — a day-by-day session transcript style, dense
with early rulings, rejected mechanisms and the reborrow/enum-equality
investigations' approval trail. **Corrected:** these 15 volumes are
themselves a compiled snapshot, git-touched only on 2026-07-22 (the day of
the reorganization commit `51a3e807`) — but the individual events they
compile were, at the time, committed day by day under other, now-superseded
paths, and are fully present in `commits.md`'s 2026-07-01 – 2026-07-22
span. Read the volumes directly for the compiled narrative; cross-check
`commits.md` for the original, uncompiled commit-by-commit trail of the
same fifteen days.

**Fastest search.** `grep -rn -i '<keyword>' archive/governance/decisions/`
across all 15 files at once; each file is long enough that `grep -B2 -A10`
is worth it. The index table itself (`decision-log.md`) is the fastest way
to pick the right day-file by date before grepping into it.

---

## 4. `archive/done/`

**Contents.** 103 per-batch closure records, `NNNN-slug.md`, one per
completed unit of work in the batch-numbering era, each with a `Status:`
line, an `Authority` citation (which plan/outline item authorized it), an
"Outcome" section, and (for later batches) a landed-commits list and an
"Approval classes" section.

**Entries / date range.** 103 files, numbered 0001 through 0108 with five
gaps (0069, 0088, 0095, 0101, 0104 — 0095 in particular is not missing, it
lives at `docs/ongoing/0095-loop-pipeline.md` because that work was never
finished; the others may be similarly diverted or simply skipped numbers).
Internal dates run 2026-08-04 (batch 0001, system-capability architecture
selection) to 2026-09-06 (batch 0108, streams and TCP, closing out PR #13).
All 103 files were added to `archive/done/` in a single commit on
2026-09-06 (moved verbatim from a prior `docs/done/`, per
`archive/README.md`), so git blame on the *current* path only tells you
when the archival move happened. **Corrected: this is not a dead end.**
The pre-move path, `docs/done/`, has its own rich history independent of
the move — **223 commits, 2026-08-05 to 2026-09-06** — which *is* the
real, incremental, one-batch-closing-at-a-time commit trail behind these
103 files; `git log -- docs/done` (or a full-text search of `commits.md`)
recovers it. Read each file's own `Status:`/date line for a quick date, and
`docs/done`'s git history for the actual sequence of edits behind it.

**Evidence kind.** These are terse, structured **outcome** records rather
than deliberation transcripts: what was decided, which dossier/investigation
it rests on, and a pointer to the fuller evidence (usually under
`research/investigations/`). Good for a fast index of *what* was decided and
*when*, weaker on *why* — follow the "Authority"/"Evidence and validation"
links out to the cited investigation for the actual reasoning and rejected
alternatives.

**Fastest search.** `ls archive/done/` for the numbered index (batch numbers
are cited constantly elsewhere — PR bodies, mcts_mem, commit messages — so
having the number-to-slug mapping open while mining other sources pays off).
`grep -l -i '<keyword>' archive/done/*.md` then read the hit in full (each
file is short, 30-90 lines typically).

---

## 5. `archive/APPROVALS.md`

**Contents.** The retired owner-approval ledger. Was `governance/APPROVALS.md`
until commit `9bec22b` (2026-09-06, "Retire the approval ledger and its
activation chain") renamed it into `archive/`. Append-only: one `## <date> —
<approval kind>` entry per owner approval, each stating the reason and (for
specification changes) the exact byte digest, the delta declaration (rules
added/removed/amended), the conformance-boundary accounting (cases added/
modified/deleted/renamed), and — inconsistently across its life, but often —
the selection ground.

**Entries / date range.** 125 dated `## ` headers, 2026-07-18 to 2026-09-05;
3,141 lines, 339,721 bytes. **Corrected: fully git-mineable.** Under its
pre-rename path, `governance/APPROVALS.md`, this file has **195 commits**
running 2026-07-18 (its very first entry's own commit) to 2026-09-06 (the
retirement/rename commit `9bec22b`) — essentially one commit per entry (or
close to it), so `git log -p -- governance/APPROVALS.md` (the path no
longer exists at HEAD, but the history is real and complete) gives a
near-diff-per-approval view of the ledger actually growing, which the
static file alone does not.

**Evidence kind.** This is probably the single highest-density **ruling +
selection-ground** source in the repository for the v0.6 through v0.49 spec
eras: every specification activation in that window has an entry here
stating exactly what changed, citing the investigation that grounds it, and
(from roughly v0.30 onward, after the "twenty-one accumulated header
paragraphs" were evicted from the spec files themselves — see §12 below)
carrying the *only* surviving prose explanation for that version's change,
since the spec file itself stopped narrating its own history at that point.
Read this file, not the archived spec headers, for versions v0.30 and later.

**Fastest search.**
```
grep -n '^## 2026-' archive/APPROVALS.md          # 125-entry date index
grep -n 'activate v0\.' archive/APPROVALS.md       # every version activation
grep -B1 -A15 -i '<keyword>' archive/APPROVALS.md
```

---

## 6. `research/investigations/` (30 directories)

General shape: a per-question research bundle, usually a `DESIGN.md` or
`DOSSIER.md` (the design/argument), sometimes `SPEC-DELTA.md` (draft rule
text, explicitly marked "not yet a spec change"), `RESULTS.md`/`REPORT.md`
(measured evidence), and probe/evidence subdirectories. This is where
rejected alternatives and adversarial-review findings live in the most
detail anywhere in the repository. **Corrected:** the "git-range" column
below is now recomputed against the complete (unshallowed) history — every
row moved earlier than the first pass reported, several substantially (e.g.
`system-capability-architecture` moved from an artifactual 2026-08-28 to
its real 2026-08-05, matching `archive/done/0001`'s own date exactly). The
one remaining caveat is real, not artifactual: `research/investigations/`
as a *directory name* dates from the 2026-07-22 reorganization
(`51a3e807`), so no row can show an earlier date than that regardless of
when the underlying question was actually first explored — a few rows
(`enum-equality-investigation`, `reborrow-investigation`) show exactly
2026-07-22 for this reason and are older in substance than their range
suggests (`reborrow-investigation` concerns the v0.7-era question, whose
real 2026-07-18 discussion is in `archive/governance/decisions/v0.6-2026-07-18.md`
and the matching span of `commits.md`, not in this directory's own history).

| directory | subject | key documents | git-range |
|---|---|---|---|
| `arith-dissolution` | arithmetic-mode dissolution, v0.31 candidate | `SPEC-DELTA.md` | 2026-08-17 .. 2026-09-04 |
| `attribution-inventory` | OWN-6/OP-5 rule-citation audit after v0.32 | `FINDINGS.md` (batch 0072) | 2026-08-18 |
| `binary-arithmetic` | binary arithmetic in the proof surface | `README.md`, `L2-SPELLING.md`, `PROOF-SURFACE.md` | 2026-09-05 .. 2026-09-06 |
| `check-dissolution` | "check" construct dissolution, v0.32 candidate (PR #47 era) | `SPEC-DELTA.md`, `conformance-inventory.md` | 2026-08-18 |
| `const-eval` | const-evaluation rule deltas for v0.31 | `INITIALIZATION.md`, `SPEC-DELTA.md` | 2026-08-17 .. 2026-09-09 |
| `containers-and-resources` | containers/stores/resource-closed judgment (8 drafts; PR #12/#15/#18) | `DESIGN.md` (6,161 lines), `CONTAINERS.md`, `RESOURCES.md`, `EXTERNAL-WORKLOADS.md`, `REASSESSMENT.md`, two `EVIDENCE-*.md` | 2026-09-03 .. 2026-09-07 |
| `contract-surface` | four competing contract-syntax proposals | `DESIGN-SPACE.md`, `PROPOSAL-{A,B,C,D}-*.md` | 2026-08-18 .. 2026-08-25 |
| `decision-workflow` | a decision workflow for sustained language research (feeds `docs/practice.md`) | `DESIGN.md`, `RULE-GROUNDS.md` | 2026-09-07 .. 2026-09-09 |
| `declaration-provenance` | declaration-site provenance, v0.32-candidate draft | `SPEC-DELTA.md` | 2026-08-18 |
| `division-dissolution` | division dissolution, v0.32 candidate | `GOAL-MATCHING-PROBE.md`, `OPEN-QUESTION.md`, `SPEC-DELTA.md` | 2026-08-18 .. 2026-09-04 |
| `enum-equality-investigation` | enum equality (tag-only eeq/ene), v0.8 era | `DOSSIER.md`, `PACKET.md`, `V0.8-DELTA-DRAFT.md` | 2026-07-22 |
| `exhaustion` | resource-exhaustion synthesis over six dossiers (2026-08-23 night) | `DESIGN.md` | 2026-08-23 |
| `io-model` | the I/O model: completion design, park-on-miss, streams/TCP (PR #13, batches 0106-0108) | `DESIGN.md`, `FIRST-PRINCIPLES.md`, `IMPLEMENTATION-AUDIT.md`, `LOOP-PIPELINE.md`, `NETWORK.md`, `PARK-ON-MISS.md`, `RESULTS.md`, `reviews/` | 2026-08-24 .. 2026-09-10 |
| `linux-enumeration` | Linux directory-enumeration disposition and delta | `SPEC-DELTA.md` | 2026-08-18 .. 2026-08-28 |
| `move-on-copy` | adversarial investigation of move-on-copy semantics | `REPORT.md` | 2026-08-08 .. 2026-08-25 |
| `o11-composition` | O11 signed Boolean-goal composition, candidate | `DESIGN.md`, `SPEC-DELTA.md`, two `probe-*.wf` | 2026-08-17 .. 2026-09-04 |
| `obligation-discharge` | obligation-discharge semantics; trap as checker runtime backstop | `DOSSIER.md`, `ACCEPTANCE.md`, `CANDIDATE-REVIEW.md`, `CLAIM-RESIDUAL-CANONICALITY.md`, `PROBE-{CODEGEN,TAINT,W1}.md`, `SIMULATION.md`, `SYS-POSTCONDITIONS.md` | 2026-08-06 .. 2026-08-25 |
| `proof-certificate-architecture` | certificate-based proof architecture | `PACKET.md`, `SOURCE-CHECKING.md` | 2026-08-10 .. 2026-09-09 |
| `proof-derived-parallelism` | par/proof-derived parallelism v1 (batch 0074) | `DESIGN.md`, `PAL.md`, `RESULTS.md`, `bench/`, `debate/`, `loop/`, `probes/`, `gap-hunt-findings.md` | 2026-08-21 .. 2026-09-11 |
| `reborrow-extension` | reborrow extension, v0.31-candidate draft | `SPEC-DELTA.md`, `chain-evidence.wf` | 2026-08-17 .. 2026-09-03 |
| `reborrow-investigation` | the original no-reborrow investigation (v0.7) | `DOSSIER.md`, `MINIMAL-RULE.md`, `PACKET.md`, `V0.7-DELTA-DRAFT.md`, `modelcheck/` | 2026-07-22 |
| `searching-wfgrep` | file-open-by-name, v0.33-candidate draft | `SPEC-DELTA.md` | 2026-08-18 .. 2026-08-19 |
| `spec-ratchet` | "conciseness ratchet" measurement (batch 0070 W5) | `DELTA-DIAG1.md`, `DELTA-RATCHET.md`, `PASS-EVIDENCE.md` | 2026-08-17 |
| `spec-representation` | how the spec should represent itself (header/version scheme) | `DOSSIER.md` | 2026-08-16 .. 2026-08-25 |
| `spelling-relief` | FLOOR-5 spelling relief sweep (batches, feeds v0.41 comparison symbols) | `SWEEP.md` | 2026-08-06 .. 2026-09-03 |
| `strict-clause-retirement` | strict-in-U clause retirement, v0.33 candidate | `SPEC-DELTA.md`, `probes/` | 2026-08-18 .. 2026-09-04 |
| `system-capability-architecture` | the original system-capability architecture selection (batch 0001) | `DOSSIER.md`, `decisions.json` | 2026-08-05 |
| `take-replace` | take/replace for affine places (batch 0070 W2) | `DESIGN.md` | 2026-08-17 .. 2026-08-18 |
| `test-economy` | cost of duplicated conformance-case execution (batch 0070 W5) | `base64-dedup.md` | 2026-08-17 |
| `wfgrep-traversal` | directory traversal, v0.32-candidate draft | `RECON.md`, `SPEC-DELTA.md` | 2026-08-18 |

**Evidence kind (whole directory).** The richest source of **rejected
alternatives** in the repository — nearly every `DESIGN.md`/`DOSSIER.md`
explicitly lists options considered and not chosen, with the reason. Several
(`containers-and-resources`, `io-model`, `proof-derived-parallelism`) are
multi-round adversarially-reviewed designs whose superseded earlier drafts
are *not* kept in the repo (only the final draft survives per directory,
per the project's "supersede in place" rule) — the round-by-round history of
those, if it matters, lives only in the PR bodies (`pull-requests.md`) and
`archive/done/` batch records that cite them.

**Fastest search.**
```
grep -rl -i '<keyword>' research/investigations/
grep -rn '^Status:' research/investigations/*/*.md    # every doc's own status line
```
Cross-reference by batch number into `archive/done/NNNN-*.md` and by
spec-candidate name into `governance/spec-evolution/`.

---

## 7. `research/experiments/` (25 directories)

Note: the task brief estimated 26; the actual count is 25 directories (plus
a top-level `README.md` file, which is likely what inflated the estimate).
General shape: a `PROTOCOL.md` (preregistered method) and/or `README.md`
paired with a `RESULTS.md`, plus raw driver code, benchmark harnesses, and a
`raw/` directory of captured output. This tree is overwhelmingly
**measurement** evidence, not deliberation — it answers "what did we
measure and under what protocol", with the resulting *decision* usually
made and recorded elsewhere (a PR, `mcts_mem`, or an `archive/done/` batch).
**Corrected:** the git-range column is recomputed against the complete
history (same 2026-07-22 floor caveat as §6 above — several rows, e.g.
`crc32-swap-in`, `data-layout-owning-sequence`, `raw-deflate-default-shape`,
show exactly 2026-07-22 for the same reorganization reason, not because
nothing happened before then).

| directory | subject | key documents | git-range |
|---|---|---|---|
| `auto-parallelism-feasibility` | Study 3: can auto-parallelism be made to work at all | `RESULTS.md`, `SUMMARY.md` | 2026-07-22 .. 2026-08-25 |
| `blind-writer` | can a writer with no compiler feedback still satisfy the checker (dated subdir `2026-08-28/` with programs/probes/ledger/`REPORT.md`) | `2026-08-28/README.md`, `2026-08-28/REPORT.md` | 2026-08-28 .. 2026-09-06 |
| `buffer-initialization-cost` | cost of mandatory buffer initialization | `PROTOCOL.md`, `RESULTS.md` | 2026-08-06 .. 2026-09-05 |
| `checked-law-channel` | Channel 3: checked-law reassociation (FN-4) | `RESULTS.md` | 2026-07-22 .. 2026-08-25 |
| `codegen-vs-rust-c` | Whitefoot vs C/C++/Rust codegen/perf, general | `README.md`, `SUMMARY.md` | 2026-07-22 .. 2026-09-03 |
| `container-representation` | container representation experiments (authority/dense/lifecycle) | `README.md` | 2026-09-06 .. 2026-09-07 |
| `crc32-swap-in` | swapping a checked CRC32 kernel in for a reference one | code + benches only, **no narrative doc** | 2026-07-22 |
| `data-layout-owning-sequence` | E0.1: data layout and owning sequences (heavily hostile-reviewed) | `README.md`, `PROTOCOL.md`, `RESULTS.md`, `RESEARCH.md`/`RESEARCH_REPORT.md`, two `HOSTILE_REVIEW_*.md`, `OWNERSHIP_ROUTE_HOSTILE_REVIEW.md`, `REVIEW_RESPONSE.md`, `BASELINE.md` | 2026-07-22 |
| `default-floor` | default-floor generation/replication protocol | `PROTOCOL.md`, `README.md`, `RESULTS.md` | 2026-07-22 .. 2026-08-25 |
| `differential-fuzz` | differential fuzzing of overlap lowerings | `README.md` | 2026-08-27 .. 2026-09-06 |
| `effect-attrs-channel` | Channel 2: effect rows -> LLVM function attributes (2026-07-09) | `RESULTS.md` | 2026-07-22 .. 2026-08-25 |
| `frequency-study` | one-time Rust-opportunity frequency pilot | `README.md`, `RESULTS.md` | 2026-07-22 .. 2026-09-01 |
| `io-completion-bench` | completion-I/O benchmark harness (linux/windows, uring/epoll refs) | `README.md`, `programs/` | 2026-08-27 .. 2026-09-10 |
| `literal-line-floor` | WF-LITERAL-LINE floor protocol/results | `PROTOCOL.md`, `RESULTS.md`, `CODE_SHAPE.md` | 2026-08-05 .. 2026-09-03 |
| `park-on-miss-measurements` | measurements behind io-model design §12 | `README.md` | 2026-09-05 .. 2026-09-06 |
| `park-on-miss-switch-cost` | cost of one stack switch | `RESULTS.md` | 2026-09-04 |
| `port-study` | porting base64/wc/wc-chunk-summary/binary-trees kernels across languages | per-kernel `RESULTS.md` in `base64/`, `wc/`, `wc-chunk-summary/`, `binary-trees/` | 2026-07-22 .. 2026-09-03 |
| `raw-deflate-default-shape` | raw DEFLATE default-shape experiment (LLM-authored-kernel study) | `README.md`, `PROTOCOL.md`, `task.md`, `teaching-pack.md` | 2026-07-22 |
| `ripgrep` | RG-BASE protocol/results (feeds the ripgrep flagship frame) | `PROTOCOL.md`, `RESULTS.md` | 2026-08-04 .. 2026-08-25 |
| `scoped-alias-channel` | Channel 1: scoped-alias metadata from ownership provenance (F003) | `RESULTS.md` | 2026-07-22 .. 2026-09-03 |
| `wfgrep-baseline` | WFGREP-BASELINE protocol/results | `PROTOCOL.md`, `RESULTS.md` | 2026-08-06 .. 2026-09-03 |
| `wfgrep-double-walk` | WFGREP-DOUBLE-WALK protocol/results | `PROTOCOL.md`, `RESULTS.md` | 2026-08-06 .. 2026-09-06 |
| `wfgrep-scan-floor` | WF-SCAN-FLOOR protocol/results | `PROTOCOL.md`, `RESULTS.md` | 2026-08-05 .. 2026-09-03 |
| `wide-scan-lowering` | WIDE-SCAN-LOWERING protocol/results | `PROTOCOL.md`, `RESULTS.md` | 2026-08-06 .. 2026-09-03 |
| `zlib-core-kernels` | zlib core-kernel proof and lowering study | `README.md`, `RESULTS.md`, `DESIGN-HANDOFF.md`, `GUARDED-COMPILER-RESULTS.md`, `PERIODIC-COMPILER-RESULTS.md` | 2026-07-22 .. 2026-09-03 |

**Evidence kind (whole directory).** Measurements and their protocols
(pre-registration, method, raw numbers) — use these when a decision record
elsewhere cites a number and you need the method behind it, or when you need
to check whether a claimed measurement is actually backed by a committed
protocol. Several `RESULTS.md` are marked with an explicit disclaimer of
what they no longer prove (e.g. the `wfgrep-*` bundles' "Replay status"
sections noting the subject program changed shape since the run) — read
those disclaimers, don't just quote the numbers.

**Fastest search.** Same pattern as investigations:
`grep -rl -i '<keyword>' research/experiments/`; `find research/experiments
-name RESULTS.md | xargs grep -l -i '<keyword>'` to search only the
conclusions and skip raw driver code.

---

## 8. `research/notes/`

**Contents.** Nine standalone files, older and more heterogeneous than the
`investigations`/`experiments` trees (no shared template):

| file | lines | subject |
|---|---|---|
| `batch1-spec-deltas.md` | 750 | proposed spec-fix deltas from a 36-agent propose/critique/synthesize workflow (not yet applied at time of writing) |
| `codegen-vs-rust-c-2026-07-08.md` | 53 | first head-to-head bootstrap-toolchain codegen benchmark ("R0 evidence") |
| `fr-reconciliation-m0.md` | 200 | M0: Featherweight-Rust reconciliation memo for spec §5 (cites Pearce TOPLAS 2021) |
| `headline-artifact-brainstorm.md` | 325 | 2026-07-10 five-lens brainstorm of flagship demo artifacts |
| `headline-artifact-shortlist.md` | 368 | shortlist of external validation candidates (SQLite considered; superseded, per its own status line, by 2026-08-03) |
| `headline-brainstorm-39-ideas.json` | 525 | structured form of the same brainstorm, 39 ideas |
| `missing-research-backlog.jsonl` | 104 | one JSON object per topic still missing a primary source (array layout, numeric semantics, dispatch/generics, ...) |
| `regions-effects-vs-safe-rust-2026-07-08.md` | 42 | adversarial study: "do regions+effects beat best-effort safe Rust? mostly NO" |
| `ripgrep-flagship-frame.md` | 220 | the owner-selected umbrella target framing for the ripgrep experiments |

**Entries / date range.** 9 files; internal dates 2026-07-08 through at least
2026-08-03 (the shortlist's superseded-status date). **Corrected:** this
directory is a one-shot 2026-07-22 compiled snapshot (4 commits touch it
total, all from the reorganization era), not a shallow-clone artifact — the
files it compiles are older than the directory itself, and their original
form (if it was ever committed separately rather than authored directly
into these files) would be findable, if at all, in `commits.md`'s early
span rather than in this path's own history.

**Evidence kind.** Early rationale and rejected-direction material
(especially `regions-effects-vs-safe-rust` and `headline-artifact-shortlist`,
both explicitly "this did NOT pan out" records) plus one live-tracked
backlog (`missing-research-backlog.jsonl`) of research gaps that may or may
not have since been filled — worth checking against the current
`research/investigations/` list for topics that are still actually missing.

**Fastest search.** Only nine files; `grep -l -i '<keyword>' research/notes/*`
then read the hit directly. For the two JSON/JSONL files, treat each line/
object as one record rather than grepping raw text.

---

## 9. `governance/spec-evolution/` (10 candidate documents)

**Contents.** Specification-change candidates, each an argued proposal for
one or more rule edits, generally pre-dating or paralleling the more formal
`research/investigations/*/SPEC-DELTA.md` convention.

| file | status / outcome | internal date |
|---|---|---|
| `comparison-symbols-v041-candidate.md` | activated as v0.41 (PR #9) | 2026-09-03 |
| `ent5-loop-fix-v024-candidate.md` | exact-approved and activated as v0.24 | 2026-08-09 |
| `index-surface-v022-candidate.md` | candidate/draft, owner ruling "批" | 2026-08-07 |
| `nonascii-string-and-escaped-display-deltas.md` | delta text for lead integration, batch 0070 W5 | 2026-08-17 |
| `obligation-discharge-batch1-candidate.md` | candidate, approved at sitting | drafted 2026-08-06 |
| `parked-loan-freeze-candidate.md` | review-candidate v0.18 material (lexical loan-scope closure) | 2026-07-23 |
| `provenance-gate-candidate.md` | design evidence for the gate activated as v0.27 | stage 5a-R |
| `spelling-relief-candidate.md` | candidate/draft (owner overnight standing instruction) | 2026-08-07 |
| `stable-spec-filename-proposal.md` | approved 2026-08-07, adopted with v0.24 | 2026-08-09 |
| `v0.23-review-packet.md` | draft review packet ("what the owner is actually being shown") | pre-v0.23 |

**Entries / date range.** 10 files, internal dates 2026-07-23 to 2026-09-03.
Git-mineable from 2026-07-22 (88 commits touch this directory).

**Evidence kind.** Rulings and selection grounds at the finest grain —
several are literally the packet the owner reviewed before approving a
version, so they show the *proposal as put to the owner*, which is more
complete than the terser `archive/APPROVALS.md` entry recording the
resulting approval. Good paired reading: a `spec-evolution` candidate plus
its corresponding `archive/APPROVALS.md` entry for the same version.

**Fastest search.** Ten files; read each in full (a few hundred lines to a
few thousand). `grep -l -i '<keyword>' governance/spec-evolution/*.md`.

---

## 10. `mcts_mem/` — the frozen memory tree (primary mining target)

**Contents.** `mcts_mem/whitefoot.md` is the root node (project-wide
constitution-derived premises); `mcts_mem/whitefoot/` is a tree of topic
node files (`checks-and-proofs.md`, `data-model.md`, `development-workflow.md`,
`effects.md`, `name-resolution.md`, `ownership.md`, `parallelism.md`,
`pattern-doctrine.md`, `surface-form.md`, `system-interface.md`,
`toolchain.md`, each with further nested nodes) plus a `.alt` file or
directory beside nearly every node holding the rejected alternative(s) for
that decision. 131 `.md` files total (1 root + 130 under `whitefoot/`).

Every node has the same two-part shape: a short **current-state summary**
at the top (a handful of plain bullets, no dates — *this* part is what was
already migrated into `design/language`/`design/compiler`), followed by one
or both of:
- `## Facts` — dated bullets recording rationale, measurements, and
  standing statements (99 such headings across the tree);
- `## Moves` — dated bullets recording what a decision *replaced* and why,
  usually `- YYYY-MM-DD (commit) replaced [[node-name]]: <reason>` (120 such
  headings across the tree).

**Entries / date range.** **776 dated bullet lines** total (verified count,
matching the task brief exactly) across the `## Facts` and `## Moves`
sections combined, spanning 2026-07-02 to 2026-09-11. Git-mineable in its
own right too — 167 commits touch `mcts_mem/`, from 2026-07-12 — so
`git log -p -- mcts_mem` shows each dated bullet being added in something
close to real time, which can help date-disambiguate an entry whose own
text is vague. Of these, 589 end in
the tag `(sourced)`, 155 end in `(code)`, and 32 have neither tag on the
bulleted line itself (typically because the tag sits on a wrapped
continuation line rather than the bullet's own last line — a formatting
variance, not a missing citation). The tags distinguish how the entry was
grounded: **`(sourced)`** marks an entry backed by an external citation —
another document, a measurement bundle, an owner statement or discussion;
**`(code)`** marks an entry grounded directly in what the shipped compiler/
runtime code does at that point in time (i.e., "this is what the
implementation does", not "this is what evidence recommends"). A `(code)`
entry is therefore exactly the kind of statement that can go stale the
moment the implementation changes again — several nodes' `## Moves` sections
explicitly show a later dated entry correcting an earlier `(code)` entry
that the tree had outgrown (e.g. `parallelism.md`'s 2026-09-10 entry
superseding its own 2026-09-06 entries).

**This is the single most important untapped source named in the task
brief:** per the repository's own migration history, only each node's
top-of-file current-state summary and its replacement/rationale entries were
carried into `design/language`/`design/compiler` — **the 776 dated Facts/
Moves lines themselves were explicitly left behind** and are not duplicated
anywhere else in a structured form (individual facts are sometimes echoed in
prose inside a PR body or a batch record, but never as a complete indexed
list). Mining this tree systematically — node by node, dated line by dated
line — is likely the highest-yield single activity for populating the
design trees' still-thin areas.

**Evidence kind.** All four kinds at once, densely: standing rationale,
point measurements (often with exact numbers and sample sizes), explicit
supersession chains (`## Moves`), and every rejected `.alt` node's own
reasoning kept alongside it rather than deleted.

**Fastest search.**
```
find mcts_mem -name '*.md' | sort                       # the whole tree map
grep -rc '^- [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}' mcts_mem/ | awk -F: '{s+=$2} END{print s}'   # 776 total, per-file with plain grep -rc
grep -rn '^- [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}' mcts_mem/ | grep -c '(sourced)$'   # 589
grep -rn '^- [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}' mcts_mem/ | grep -c '(code)$'      # 155
grep -rn -i '<keyword>' mcts_mem/
```
A `.alt` sibling (file or directory) beside any node name is the rejected
branch for that same decision — always check it when mining the node next
to it.

---

## 11. `docs/roadmap.md`, `docs/bargain.md`, `docs/ideas.md`, `docs/why-whitefoot.md`

All four are explicitly marked **non-authoritative reference/synthesis**
(their own `Status:` lines say so) rather than live decision records, but
each compiles or narrates real dated history:

- **`docs/roadmap.md`** (1,342 lines) — "Direction Outline", revision 71.
  Long-range candidate directions with a `Current` line each. Explicitly
  "not part of the working loop" (per `CLAUDE.md`) — read for orientation on
  what was ever considered a direction, never as a statement of current
  behavior.
- **`docs/bargain.md`** (910 lines) — "the bargain ledger", compiled
  2026-07-28 from the founding directives, round-2/3/4 design debates, the
  headline brainstorms (see `research/notes/` above), the research backlog,
  and the capability-research era; §8 (ML direction) and §9 (embedded
  direction) added 2026-07-31 each after an external research pass whose
  findings are cited inline, including where the pass refuted its own
  briefing.
- **`docs/ideas.md`** (213 lines) — supporting mechanism sketches and
  possible experiments not yet promoted to a real investigation.
- **`docs/why-whitefoot.md`** (743 lines) — a dated design synthesis essay
  combining language ideas with measurements from the retired "democ"
  compiler; explicitly warns its historical performance/architecture claims
  are not current.

**Entries / date range.** 4 files; internal content dates back to project
founding (early July 2026, per `bargain.md`'s citation list). **Corrected:**
all four have real, rich, continuous git history, not a one-shot import —
`roadmap.md` 316 commits (from 2026-07-10), `why-whitefoot.md` 25 commits
(from 2026-07-17), `bargain.md` 12 commits (from 2026-07-31, matching its
own stated compilation date), `ideas.md` 11 commits (from 2026-07-17). Of
the four, `roadmap.md` is by far the most git-mineable in its own right —
`git log -p -- docs/roadmap.md` gives 316 dated snapshots of the project's
own sense of its long-range direction, a genuine history in itself.

**Evidence kind.** Compiled rationale and rejected-direction narrative at
the broadest, most synthesized level — useful for recovering *why a whole
direction* was or wasn't pursued, less useful for a specific rule's
wording.

**Also present, lower priority for decision mining:** `docs/todo.md` (37
lines, current defects — its own header states "None of them is a
decision"); `docs/ongoing/0095-loop-pipeline.md` (1,375 lines, the one
`archive/done/`-shaped record for work that was never finished, corresponding
to the 0095 gap noted in §4 above — this is a live-in-progress obligations
list, not yet a closed decision, but records real designed-and-verified
partial results worth not re-doing).

**Fastest search.** `grep -n -i '<keyword>' docs/roadmap.md docs/bargain.md
docs/ideas.md docs/why-whitefoot.md`. `bargain.md` and `roadmap.md` are long
enough to want `grep -n '^#'` first to find the right section.

---

## 12. `spec/kernel-spec-v0.0.md` through `spec/kernel-spec-v0.52.md`

**Contents.** 53 immutable archived specification snapshots (v0.0 through
v0.52; the live `spec/kernel-spec.md` is currently v0.53). Each begins with
a `Status:` line; versions v0.9 through v0.29 also carry a `Prior:` line.

**Which versions carry narrative prose (the task's specific question):**

| range | `Status:` | `Prior:` | shape |
|---|---|---|---|
| v0.0 – v0.8 | yes, substantial (440 -> 5,213 bytes, growing) | no | one paragraph narrating this draft's own purpose and what it revised |
| v0.9 – v0.29 | yes, substantial (peaks at 10,701 bytes at v0.23) | **yes**, substantial | both lines together narrate this version *and* explicitly recap the immediately preceding one — this is the richest per-version prose band in the whole spec archive |
| v0.30 | yes, short (101 bytes) | no | transitional: "structured representation profile; no semantic change from v0.29" |
| v0.31 – v0.32 | yes, short (298 / 525 bytes) | no | still names the batch/rule delta in one line |
| v0.33 – v0.52 | yes, **terse** (21 bytes, literally `Status: ACTIVE v0.NN`) | no | no narrative at all |

The collapse happens at v0.30: `archive/APPROVALS.md`'s v0.30 entry
describes this explicitly as "the v0.30 representation rework, which evicted
twenty-one accumulated header paragraphs... history lives in the archives
and the chain" — i.e. a deliberate decision to stop narrating spec history
inside the spec file itself. From v0.33 onward, **the archived spec file
carries zero explanation of what changed or why**; that explanation lives
only in `archive/APPROVALS.md` (§5 above, through v0.49) and, for versions
after the ledger's 2026-09-06 retirement (v0.50 onward), only in the
corresponding PR body (§2) and `mcts_mem/` entries.

**Evidence kind.** For v0.0-v0.29: self-contained rationale, read the file
directly. For v0.30-v0.52: the file itself is evidence only of *what the
rules said*, not *why* — pair it with the matching `archive/APPROVALS.md`
entry or PR. All of `spec/` is git-mineable in its own right too — 200
commits, from the true root `7c1d7641` (2026-07-07) — and for the terse
v0.33+ versions, the commit that introduces each archived file is itself
worth reading even where the file's own `Status:` line says nothing: it is
frequently the activation commit and carries a real body.

**Fastest search.**
```
grep -A3 '^Status:\|^Prior:' spec/kernel-spec-v0.{0,1,2,3,4,5,6,7,8,9}.md spec/kernel-spec-v0.1?.md spec/kernel-spec-v0.2?.md
```
For v0.30+, search `archive/APPROVALS.md` and `pull-requests.md` instead, by
version number (`grep -n 'v0\.NN' ...`).

---

## 13. `compiler/src/**` doc comments

**Contents.** Rust (`.rs`, 234 files) and C runtime (`.c`/`.h`, 39 files)
source under `compiler/src/`, whose doc comments (`///`, `//!`) and line
comments (`//`) occasionally cite a dated owner ruling or a measurement
directly at the code they justify, rather than only in an external
document.

**Counts** (whole-word, case-insensitive, across all 273 files):

| term | total occurrences | occurrences inside a comment line |
|---|---|---|
| `ruling` | 10 | 10 (all of them) |
| `owner` | 890 | 183 |
| `measured` | 378 | 159 |
| `rejected` | 176 | 59 |

**Caveat:** `owner` is dominated by the unrelated ownership/memory sense
("storage in its owner", "the enclosing owner is consumed") — most of the
890 raw hits and most of the 183 comment hits are about language ownership
semantics, not a human decision-maker. The genuinely decision-flavored hits
are the ones paired with `ruling`, `approved`, `decision`, `directed`, or a
date; combining an explicit date with any of the four terms above finds
exactly **6** comment lines — a small, high-precision set:

```
compiler/src/backend/emitter/cleanup.rs:17     the owner's ruling of 2026-09-04
compiler/src/backend/tests/stack_ledger.rs:285 opposite until 2026-09-04, when the owner...
compiler/src/semantic/tests/generics.rs:302    opposite until the owner's 2026-09-05 ruling
compiler/src/semantic/tests/generics.rs:419    the 2026-08-08 ruling that settled the question
compiler/src/semantic/check/expressions/flat_storage/slices.rs:163  by the 2026-08-08 ruling
compiler/src/syntax/parser/finalize/tests/corpus_shape.rs:44        the 2026-08-08 ruling examined it
```

Other `ruling` hits without an inline date (`driver.rs:2451/2454`,
`backend/tests/completion.rs:561/593`) still cite a ruling but point the
reader at "design section 8" or a manifest row rather than a date —
follow those pointers into the relevant `research/investigations/` design
doc.

**Evidence kind.** Small in volume but high-trust: a comment that survives
next to the code it justifies has, by construction, stayed true through
every later change (or it would have been caught in review) — this is
the least stale evidence source in the repository, at the cost of being
sparse. Good for confirming a decision found elsewhere actually lines up
with what the shipped code does today, and for `git blame` to find the
exact commit (hence PR, hence full rationale) that introduced the comment.

**Fastest search.**
```
grep -rniE 'ruling' compiler/src                                  # all 10
grep -rnE '^\s*(///|//|//!)' compiler/src | grep -inE 'ruling|owner|measured|rejected'
git blame -L<line>,<line> <file>    # to find the introducing commit, then look it up in commits.md
```

---

## 14. Other `archive/` material (not individually named in the task brief)

`archive/README.md` (67 lines) is itself a curated index of most of the
rest of `archive/` and is worth reading before digging further — it
explains provenance for `DECISION_SPRINT.md`, `ROADMAP.md`,
`HANDOVER-2026-07-17.md`, `compiler/PLAN-2026-07-17.md`, `research/`,
`experiments/`, `m3/`, and `toolchains/self-hosting-2026-07-20/` in its own
words, better than a re-description here would. In brief:

- **`archive/current-plan.md`** (1,495 lines) — the last rolling execution
  plan before planning-in-a-document was retired (2026-09-06); its final
  accumulated content is entirely the I/O-model plan (file permit,
  park-on-miss, streams/TCP — the same work as PR #13 and
  `research/investigations/io-model/`), with three inline dated decision
  markers (`### Decided 2026-09-04:`, `### Decided 2026-09-05:`, `###
  Recorded 2026-09-05, deferred...`). **Corrected: this one is richly
  git-mineable under its pre-archive path**, `docs/current-plan.md` — **115
  commits, 2026-08-03 to 2026-09-06** — which is the rolling plan actually
  being edited in place through several complete plan cycles, not just the
  final I/O-model one visible in the frozen text; `git log -p -- docs/current-plan.md`
  recovers the earlier plans that this same file held before being
  overwritten in place for the next cycle (each overwritten version is a
  real decision, generally not recorded anywhere else once superseded).
- **`archive/research/`** (456 files) — the pre-corpus "evidence-first
  research era": multi-agent debates (`debates/`), source papers
  (`sources/`), feature matrices, synthesis notes; also
  `research/minimal-systems-capability/` (2026-07-14/15 capability-research
  era, ~179MB, superseded) and `research/capability-floor/` (2026-07-13).
  Per `archive/README.md`, this produced the corpus `docs/constitution.md`
  and the spec derive from.
- **`archive/experiments/`** (20 files) — corpus-era measurement studies
  (noalias collapse vs Rust/C, region-effect scatter residual, guarded-plan
  parallelism); conclusions already absorbed into corpus notes.
- **`archive/compiler/`** (66 files) — `PLAN-2026-07-17.md` (a retired
  competing compiler roadmap) plus a frozen `frontend-v0.9-2026-07-22/`
  snapshot.
- **`archive/m3/`** (30 files) — the shelved model-tier authorship harness
  (`RESULTS.md`, `IMPLEMENTATION_GATES.md`, `tasks.jsonl`, `prompts/`,
  `submissions/`, `harness/`); shelved per decision D5, kept shelf-ready.
- **`archive/toolchains/self-hosting-2026-07-20/`** (134 files) — the
  retired original Whitefoot `wfc`, the Python "democ" prototype, and the
  tape-era inventory; the active Rust compiler imports nothing from it.
- **`archive/tools/`** (1 file) — an old duplicated-status verifier script,
  replaced by an active `/tools/` equivalent.
- **`archive/retired-gate/`, `archive/premature-capability-audit/`,
  `archive/superseded-lexical-v08/`, `archive/tests/`** — older retired
  Python-era gate/verifier tooling (grammar-verifier, spec-guard,
  lexical/terminal audits, a capability-overlay prototype, frozen
  reference/codegen/conformance snapshots; 263+5+9+11 files respectively).
  Not described in `archive/README.md`; skimmed for this survey but not
  read in depth — code and generated artifacts more than decision prose, so
  lower priority for mining, but flagged here so they aren't mistaken for
  unindexed.
- **`archive/DECISION_SPRINT.md`** (262 lines, "Phase B COMPLETE 2026-07-09")
  and **`archive/ROADMAP.md`** (24 lines, 2026-07-08) — pre-consolidation
  plans, superseded by a since-also-retired `/THE-PLAN.md`.
- **`archive/HANDOVER-2026-07-17.md`** (950 lines) — the last competing
  handover document before single-plan consolidation.

**Corrected:** most of this material is git-mineable, but thinly — these
are largely one-or-few-commit archival snapshots (`archive/research/` 7
commits 2026-07-12..2026-08-25, `archive/experiments/` 4 commits, `archive/compiler/`
2 commits, `archive/m3/` 2 commits, `archive/toolchains/` 1 commit,
`archive/DECISION_SPRINT.md`/`ROADMAP.md` 2 commits each, `archive/HANDOVER-2026-07-17.md`
1 commit, and the four undocumented Python-tooling directories 1-3 commits
each, all from 2026-07-12 through 2026-07-22) — meaning most of these
paths were moved wholesale a small number of times rather than edited
incrementally, which is consistent with them being explicitly retired,
frozen material rather than living documents. `docs/current-plan.md` (the
pre-archive path of `archive/current-plan.md`, above) is the one clear
exception, with real ongoing edit history. Read the files directly, guided
by `archive/README.md`; consult `commits.md`'s full text, not these paths'
own thin git logs, for the reasoning behind their content.

---

## 15. Remote branches not merged into `origin/main` (new source)

`git fetch --unshallow` surfaced 133 remote branches (`git branch -r`) that
the shallow clone never showed at all. **75 of them are not merged into
`origin/main`** (`git branch -r --no-merged origin/main`) — real, unmerged
work, most of it never landed and some of it never even opened as a PR, so
it is not represented in `pull-requests.md` either. This is therefore a
genuinely new source, not a reclassification of an existing one.

**60 of the 75 collapse into one.** Every one of the 60 `codex/io-*`
branches, plus `codex/compute-runtime`, is confirmed (`git merge-base
--is-ancestor`, checked individually for all 60) to be an ancestor of a
single branch, `origin/archive/codex-io-2026-09-11` (tip: "Archive of the
codex/io-\* experiment branches and codex/compute-runtime, 2026-09-11").
Their tips range 2026-08-25 to 2026-09-10. This looks like a deliberate
end-of-life rollup of a large, long-running family of I/O-runtime
experiment branches (io continuation, io-cpu-\*, io-client-\*, io-go-\*, and
more) rather than 61 independent leads — read the archive branch itself
first; the 60 individual branches likely add little beyond what it already
contains, but remain individually inspectable
(`git log origin/codex/io-<name>` or `git diff origin/main...origin/codex/io-<name>`)
if the archive branch's own history compresses something a reader needs at
finer grain.

**The other 14 distinct tips**, sorted by tip date, with a cross-reference
to `pull-requests.md` where one exists:

| branch | tip date | tip subject | already in `pull-requests.md`? |
|---|---|---|---|
| `handoff/2026-08-28-cloud` | 2026-08-28 | handoff: keep the main boundary to its one sentence | no |
| `batch/0104-streaming-paths-design` | 2026-08-28 | Design: streaming chunk loops, and bytes to path | no — and this is the branch behind the `0104` gap noted in §4: no `archive/done/0104-*.md` was ever written for it |
| `batch/0106-claim-model-design` | 2026-08-29 | design: retire the "one flippable decision" count everywhere it was live | no (but subsumed — see next row) |
| `batch/0111-proof-replaces-claim` | 2026-08-29 | design: repair the two split tables... mark the draft superseded | **yes, PR #2** (closed without merging; `0106` above is this branch's own ancestor) |
| `batch/0116-containers-and-resources` | 2026-09-04 | Merge remote-tracking branch 'origin/main' into batch/0116-containers-and-resources | **yes, PR #12** (merged) |
| `batch/0127-b3-parked` | 2026-09-04 | B3: the type-derived call transports, and the ordering they falsify | no |
| `handover/io-model-2026-09-06` | 2026-09-06 | Handover: state the word ban without the words | no |
| `batch/0127-containers` | 2026-09-06 | B7c4b: The fill helper's doc names the reason it is a helper... | no — a **second, independent** `0127` line: confirmed neither ancestor nor descendant of `batch/0127-b3-parked` above, i.e. two different forks reused the same batch number |
| `codex/semantic-proof-probe` | 2026-09-09 | Research contracts for generalized test intent | no |
| `archive/codex-io-2026-09-11` | 2026-09-11 | Archive of the codex/io-\* experiment branches... | no (it's the rollup itself, see above) |
| `codex/container-foundation-research` | 2026-09-11 | fix(backend): omit unreachable loop continuation blocks | **yes, PR #30** (open, draft) |
| `ci/compute-regression` | 2026-09-12 | ci: run the compute regression check on every pull request | **yes, PR #48** (open) |
| `claude/code-space-constraints-mdagkl` | 2026-09-12 | Audit the resolution module against the design tree | **yes, PR #35** (open) — **this is this session's own branch**; fully covered by `commits.md` already, not additional material |
| `compute/asymmetric-window` | 2026-09-12 | Name the revision the asymmetric-core window rule landed at | no |

**Net new mining targets from this section**, after excluding what's
already covered elsewhere (the 60+1 collapsed branches, the 5 rows with an
existing PR, and this session's own branch): **7 branches genuinely
outside every other source in this survey** —
`handoff/2026-08-28-cloud`, `batch/0104-streaming-paths-design`,
`batch/0106-claim-model-design` (ancestor of PR #2's branch, so mostly but
not entirely redundant with it), `batch/0127-b3-parked`,
`handover/io-model-2026-09-06`, `batch/0127-containers`,
`codex/semantic-proof-probe`, and `compute/asymmetric-window` — plus the
`archive/codex-io-2026-09-11` rollup itself as the entry point to the 60
collapsed experiment branches.

**Evidence kind.** Unknown until read — by construction, an unmerged
branch is either abandoned, superseded, or still in flight, and this survey
did not open any of their diffs or commit bodies (that is exactly the next
pass's job). `git log origin/main..<branch>` on each gives its own commits
in the same rich style as `commits.md`; `git diff origin/main...<branch>`
gives the substantive change no PR body ever described.

**Fastest search.**
```
git branch -r --no-merged origin/main             # the 75, recomputed any time
git log origin/main..origin/<branch>              # a branch's own unmerged commits
git diff origin/main...origin/<branch> --stat      # what it actually changes
```

---

## What is not on this machine

**Chat/session logs.** Even with the full 2026-07-07–2026-09-12 commit
history now recovered, what is on this machine is still only the *written
artifact* of each decision — the commit, the PR body, the dated `mcts_mem`
bullet — never the interactive back-and-forth that produced it, which
frequently compresses a long deliberation into one terse sentence ("owner
ruling", "the owner selected"). That reasoning happened in chat sessions
that left no other trace on this filesystem. Several PR bodies and commit
messages reference a `https://claude.ai/code/session_...` URL; those
sessions are not fetchable from here. If the owner can export or share
transcripts, they would be the only remaining source for the deliberation
itself, as opposed to its recorded conclusion, behind any of the fifteen
sources above.

---

## File sizes (this survey's own output, current as of the correction)

| file | lines | bytes |
|---|---:|---:|
| `design/recall-tmp/sources/commits.md` | 29,295 | 1,508,468 |
| `design/recall-tmp/sources/pull-requests.md` | 2,360 | 260,759 |
| `design/recall-tmp/sources.md` (this file) | ~945 | ~62,000 (self-referential; see the task's final report for the exact number) |
