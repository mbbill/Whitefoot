---
name: agent-reports-are-claims
description: "A subagent's report is a claim, not evidence — require greppable proof before turning any finding into a ruling or a downstream brief"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-08T15:44:53.114Z
---

Learned the hard way 2026-08-07 (Whitefoot FLOOR-5 batch). A drafting agent
reported four specification sites found by a new sweep pattern, with quoted
sentences and rule/line numbers. Three of the four quotes did not exist in
the active spec at all (`grep -rn "there is no coercion" spec/` → 0 hits;
"declared type" → 0 hits in the whole file). I relayed that report verbatim
into a replacement drafter's brief AS A SPECIFICATION; only that drafter's
own verify-before-writing habit kept three fabricated rules out of a
candidate the owner would have been asked to approve byte-exactly.

Same agent, same session, also produced genuinely real finds later (ENT-3
S4, OWN-13) whose structural reasoning was correct while its *numbers* were
inflated (claimed 122 files carrying `requires`; the true count is 22).

**Why:** confabulated specifics read exactly like verified ones in a report,
and a lead who forwards them launders a guess into an authority.

**How to apply:** (1) every agent brief must require the grep/command output
that proves each claimed anchor exists and occurs exactly once; (2) never
forward a peer report as a specification — forward it as an unverified
claim with "verify before writing" attached; (3) for anything that would
reach an owner approval or a spec file, verify at least the load-bearing
anchors personally, with your own command; (4) trust an agent's structural
shapes further than its counts. See [[whitefoot-collaboration-model]].

**Correction 2026-08-07 (owner):** "the lead does it himself" is NOT a
mitigation — the lead is the same model class and confabulated a section
reference in the same session (said §5 where the figure was §3; a peer
agent caught it). What actually separates reliable from unreliable output
here is whether the artifact is machine-checkable: compiler changes,
grammar tables and conformance never produced a hallucination incident
because gates, the table-derivation check and the corpus fail loudly;
candidate PROSE produced several because nothing checks it. Standing rule
adopted: every figure in a specification candidate carries the exact
command that reproduces it, so any reader re-runs it instead of trusting
a report. Process control, not author selection.

**Second correction 2026-08-08 (measured, lead's own error):** the lead
wrote a fabricated SHA-256 and a nonexistent commit id into a task card —
`git log --all -S<digest>` traced both to the lead's own commit — while
simultaneously instructing agents to report only recomputed values, and
then wrongly accused an executor of the fabrication. The executor verified
the accusation instead of accepting it, which is what surfaced the truth.
Generalized lesson: **the artifact that orients the next agent (a task
card, a hand-back brief) is the highest-leverage place for a fabricated
fact**, because it is read as description rather than as claim and no
machine check covers it — every gate in this repo watches the spec and the
compiler, none watches docs/. Practice: any digest, commit id, or count
written into a task card carries the command that reproduces it and an
instruction to recompute; and an accusation of fabrication is itself a
claim requiring the same proof as any other.

**Third instance 2026-08-08, and the rule it finally produced:** a hand-back
brief carried tip `a4d3d33` (`git rev-parse` → fatal, malformed object name),
the third fabricated commit id from the lead on one task. The executor
checked before working and found the branch also held by a live worktree
with the previous round's work still uncommitted, so the brief's "M1 is
complete" was false too. Standing practice adopted: **a brief names the
BRANCH, never a commit id — the executor resolves and verifies the tip
itself.** An id in a brief is a claim with no checker; a branch name
resolves. Note the asymmetry that keeps recurring: the SHA-256 digest in the
same brief was correct, while the human-ish counts were wrong twice in
opposite directions (87 too low, then 142, against a measured 105 Bool
matches / 210 arms / 13 files). Copied digests survive; remembered counts and
ids do not. The cross-check that made the count trustworthy was structural —
`True()` and `False()` arm counts equal in every file — so prefer a figure
that carries an internal consistency check over one that does not.

**Fourth instance 2026-08-08, new artifact class:** writing durable design
facts into `mcts_mem/`, the lead wrote `5188548e` for a commit whose real
8-char form is `5188548f` — one character, invented by autocompleting a hash
it had just seen as `5188548`. The design tree is the worst place for this:
it is permanent, it is read as settled history, and `npx mcts-mem lint`
checks FORM (links resolve, paired moves agree, entries are atomized — it did
catch a separate defect, a fact chaining two claims) but never TRUTH. Nothing
in the repo would ever have flagged it. Practice: **run `git rev-parse
--short=8 <hash>` on every id you write, in your own prose too, before the
commit** — the same rule already applied to briefs extends to the tree, the
dossiers, and APPROVALS. A truncated hash is not shortenable by guessing the
next digit; resolve it. Generalized: the lead's error rate on invented short
identifiers is now 4 for 4 unverified, so treat any id typed rather than
pasted from command output as wrong until a command says otherwise.

**The sharpest discriminator found so far (2026-08-08), and it works WITHIN a
single report.** An executor reported closing a soundness hole where "a
trapping operator or a `move`, borrow, or subscripted operand escaped" a
requires check position. It later retracted three of the four itself: only the
subscripted operand was real. The claims sorted perfectly by their backing —
the subscript claim carried a differential reproduction (same source, exit 0 at
the parent commit, rejects at the fix, both sides rebuilt) and was true; the
other three carried prose and were false (the trapping operator is caught
earlier by a different rule; the `move` probe rejected for an unrelated reason;
the borrow probe never parsed). Same agent, same report, same paragraph.

**How to apply:** ask of each claim "what did it RUN on both sides?", not "does
this agent seem careful?" — carefulness is not the variable, and a good agent's
prose claims fail alongside its reproduced ones. And relaying matters: I copied
the inflated sentence into `governance/APPROVALS.md`, converting a claim into
an authority, and only the executor's own retraction pulled it back. Extract
the reproduced half, cite the command, and mark everything else "not measured"
— which is also the phrase to require from agents, since this one wrote exactly
that for the borrow probe and it was the most useful line in the report.

**Over-retraction is the same error wearing humility (2026-08-08, the same
executor's own diagnosis).** Having just retracted three unreproduced claims, it
retracted a fourth that *did* have a reproduction available — calling a
load-bearing guard "prospective". Removing the guard and rebuilding showed a
trapping row riding straight through it. Its words: "over-retraction reads as
rigour but is the same error of not measuring." Both directions are writing a
claim without running the thing, and the humble-sounding one is harder to catch.
So when an agent walks a claim back, ask what it ran to justify the
*retraction*, not only the original. It reported that being pushed on the
over-retraction was more useful than being let off for the overstatement.

**Two refinements from running four parallel executors (2026-08-08).** First,
"a brief names the branch, never a commit id" has no caveat version: I wrote
"main at `acf85cd`" and later "`main`, now at `8df0e29`", each stale before it
arrived, even though both messages also said "resolve the tip yourself". An id
in a brief invites reliance regardless of what sits beside it — so do not write
one at all. Second, and more useful: **with parallel agents, messages cross
routinely**, so a ruling delivered only by message may never be read. Three
times an executor reported "still waiting on your call" for something already
ruled. Put rulings in the durable file (here `governance/APPROVALS.md`) and
tell agents to read them there; message ordering is not a channel you control,
and a committed file is.

**Differential measurement — the rule and its escape.** Measuring a before/after
requires moving the working tree (a file checkout, a rebuild at another
revision), which is how an executor destroyed an uncommitted fix it had already
verified. Two parts, the second from that agent: prefer a throwaway `git
worktree add --detach` for the other side — one extra compile, touches nothing —
and when taking the cheaper in-place path, commit the after state BEFORE
checking out the before. Naming the escape matters as much as the guard, or the
rule degrades into "commit more often" and gets ignored.
