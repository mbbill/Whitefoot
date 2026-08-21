# Whitefoot workflow

This is the sole operational guide for advancing Whitefoot. Ordinary delivery
runs as lead-orchestrated batches on work branches, with an adversarial audit
at each batch boundary and one owner approval at the merge to `main`.
Specification and protected-compliance changes use the guarded workflows
below; their approval point is that merge.

## Authorities

- `docs/constitution.md` is project law.
- `spec/kernel-spec.md`, as named by `docs/roadmap.md`, is the sole active
  language authority. Flat `spec/kernel-spec-vN.md` files are immutable
  archives.
- `docs/roadmap.md` is the living Direction Outline: it records directions,
  current facts, gaps, and candidate projects, but does not sequence work.
- `docs/current-plan.md` is the sole high-level execution plan, derived from
  one outline revision; it cannot select a direction absent from the outline.
- `docs/ongoing/` holds one numbered record per live batch; `docs/done/`
  holds the same records after integration. Records coordinate and report;
  they never create authority.
- `research/**/RESULTS.md` and equivalent accepted evidence records own
  measurements; `mcts_mem/` owns durable design choices and rejected
  alternatives; `governance/APPROVALS.md` records protected owner approvals.
- `docs/WORKFLOW.md` defines process but selects no work.

Compiler behavior, tests, plans, batch records, design prose, and archives do
not define language semantics. Active source, builds, tests, and tools may not
depend on `archive/`.

## Rule tiers

Every rule in this repository belongs to exactly one tier, and the tier
states how it is enforced. A rule that names no enforcement is guidance.

1. **Machine-enforced.** The activation digest chain, archive immutability,
   candidate lineage, the repository gate (`make check`), generated-identity
   freshness, and prose digest sync. These run on every gate invocation and
   hold without anyone's attention. (`CLAUDE.md`/`AGENTS.md` synchrony is
   audit-enforced guidance: the two carry the same rules but are not
   byte-identical, because `AGENTS.md` is written for Codex.)
2. **Owner boundary.** One decision waits for the owner: merging a branch
   into `main`. The merge packet carries every reviewable class — plan
   creation or material revision, changed `spec/kernel-spec.md` bytes,
   protected-compliance evidence changes, new repository root entries — so
   approval happens once, at the boundary where branch work becomes project
   history. Nothing lands on `main` without it; on a branch, nothing waits
   for it.
3. **Guidance.** Everything else in this document and in project law. It is
   not pretended to bind an executor in flight; it is enforced by the batch
   audit, which hunts violations after the fact and reports them for repair.

## The merge boundary

`main` is the owner's; agents work on branches. A branch is chartered by an
owner direction (a plan item, or a recorded conversational direction quoted
in its batch record) and iterates autonomously: implementation, tests, docs,
plan and roadmap updates, specification candidates, protected-compliance
changes, root entries — nothing on a branch waits for approval. The single
approval is the merge, integrated by rebase onto `main` plus fast-forward
(linear history; no merge commits).

The merge packet presents, in one place, every reviewable class the branch
touched:

1. **Plan.** A new or materially revised `docs/current-plan.md` (objective,
   strategy, semantic direction, principal consumer boundary, acceptance
   criteria, risk posture, stop conditions) rides the branch as `PROPOSED`;
   the approved merge is what makes it `ACTIVE`.
2. **Specification.** Different bytes at `spec/kernel-spec.md` follow the
   specification workflow below: candidate status on the branch; the packet
   carries the complete candidate SHA-256, exact diff, impact inventory,
   verifier results, and accepted-set risk; activation is the merge's
   activation commit.
3. **Protected compliance evidence.** Changes involving conformance case
   source, manifest rows or annotations, declared verdicts, status, rule
   assignment, or coverage — and any change to a canonical compliance
   runner, adapter, oracle, baseline, gate, collection or invocation
   wiring, or gate-integrity test that can alter collection,
   interpretation, verdict, coverage, baseline identity, or whether the
   gate runs — carry the exact before/after audit from the protected
   evidence workflow below. Ordinary compiler tests are not protected.
4. **Repository root.** Any new top-level entry, called out explicitly.

Plus, always: the batch record(s), the branch-tip `make check` result, and
the audit dispositions. Approval covers exactly the presented bytes; a
changed byte or scope re-enters review; the approval is recorded in
`governance/APPROVALS.md` as part of the merge. A rejected or redirected
merge continues on the branch and re-requests.

## The batch loop

Work advances in batches — typically one working session. One lead session
owns each batch end to end:

```text
owner sets direction (a sentence or two; conversation suffices)
        |
        v
lead opens or continues a work branch and decomposes the batch
  - parallel executors in isolated worktrees when scopes are file-disjoint
  - sequential work in the lead's tree when coupled
  - the lead assigns boundaries; no claim files, no reservation protocol
        |
        v
executors return diffs; the diff is the report
lead reviews every diff, integrates on the branch, keeps the gate green
        |
        v
batch end: make check green on the branch
        -> adversarial audit (independent finders + refuters)
        -> batch record finalized
        |
        +--> more to do: next batch on the same branch (no approval wait)
        |
        v
ready to land: merge packet -> owner approves -> rebase + ff to main
rejected or redirected: continue on the branch and re-request
```

Rules of the loop:

- **Executors are tools, not principals.** An executor receives a precise
  brief, implements exactly it, and reports honestly; a blocker or
  out-of-scope discovery is a successful stopped outcome when reported with a
  reproduction — never hacked around, absorbed by weakening a check, or
  quietly narrowed. Executors do not spawn sub-batches or redesign the plan.
- **One live worktree has one writer.** Never commit, reset, rebase, or edit
  inside another live executor's worktree; integrate from a worktree the
  integrator owns.
- **The lead verifies; reports are leads, not evidence.** Load-bearing
  claims from any executor are reproduced by the lead before they reach a
  batch record or an owner packet.
- **External or unsupervised batches** (work delegated to another agent or
  produced outside a lead session) merge only after an entry audit. Trust is
  per batch and established by audit, never assumed.

## Batch records

One numbered record per batch: registered in `docs/ongoing/` as
`NNNN-short-slug.md` when the batch starts, moved unchanged in number to
`docs/done/` in the integration change. Numbers continue the existing
sequence, `max(existing) + 1`, and are never reused. `docs/planned/` is
retired; decomposition lives inside the batch record or the plan.

A batch record opens under an `ACTIVE` `docs/current-plan.md` item, or —
for branch work — under a recorded owner direction that charters the
branch; the record then quotes that direction verbatim as its authority,
and the plan is brought up to date on the branch so the merge packet
presents it. Planning work itself — refreshing the roadmap, drafting or
revising the plan — is not a batch and gets no record; its output is those
documents.

A batch record is a boundary document, never a journal. It states the
authority (the exact `ACTIVE` plan item), scope and exclusions, the
approval classes the batch will touch, and — at closure — the outcome,
landed commits, verification results, and dispositions of audit findings. Progress
narration is forbidden: record updates ride the work commits they describe,
and a docs-only commit is exceptional. Transient state belongs to the
session; evidence belongs in the commit messages of the work itself.

A batch handed to another agent (an overnight delegation, an external tool)
gets its record written before the handoff; that record is the batch
contract the entry audit checks against.

## The batch audit

The audit is the enforcement mechanism for tier-3 guidance and the entry
check for external batches. It is adversarial by construction: independent
finder agents sweep the batch across dimensions (plan-vs-actual, governance,
protected evidence, code quality, design-memory sync), and each major
finding is independently re-verified by a skeptic briefed to refute it.
Findings are repaired or explicitly dispositioned in the batch record; a
refuted finding is recorded as refuted.

The audit also reports the batch's own economics: substance versus
bookkeeping commits, gate wall time, and what landed. A process element that
fails to pay measured rent is a finding.

## Execution discipline

Before changing code, answer:

1. What concrete compiler capability, real program, or experiment does this
   unlock?
2. Which `ACTIVE` plan item or chartering owner direction authorizes it?
3. What is the smallest general implementation?
4. Does it exercise the normal compiler path rather than a project, function,
   source-shape, corpus, or test special case?
5. Has supporting machinery become larger than the capability it serves?

Freeze the real consumer boundary, oracle, and cost obligation before
changing the implementation. A project selects pressure, not language
semantics; make compiler changes general and project-independent. For
performance work, attribute the loss before optimizing, with a same-source
causal comparison and a falsifier.

Record durable design choices and rejected alternatives in `mcts_mem/`,
following the complete rules in the installed `mcts-mem-use` skill
(Claude Code sessions invoke it as a skill; other agents read the skill
document directly). A re-decision must be recorded within its batch; the
audit checks for silent divergence between the tree and the landed code.

## Blocker routing

- **Compiler defect:** implemented behavior contradicts the active spec. Add
  the smallest regression and fix the normal path without changing normative
  expectations.
- **Unsupported specified capability:** implement it only when the active
  plan needs it; never report it as invalid source.
- **Protected-evidence issue:** reproduce it, keep the active spec
  authoritative, and enter the protected evidence workflow before landing.
- **Research or performance question:** run the cheapest bounded probe with
  a hypothesis, observable, and stop condition.
- **Language gap:** record the minimal witness; if neither the active plan
  nor the branch's chartering direction contains it, update the plan on the
  branch and present it at merge. Exact spec bytes always ride the
  specification workflow into the merge packet.
- **Project-local issue:** adapt the project when the frozen contract is
  preserved, rather than generalizing the language or compiler.

A soundness defect may preempt other work; fix it with hostile regression
evidence, then return to the batch.

## Evidence discipline

- State exact commands, inputs, outputs, counts, and exit codes. Read an exit
  code directly, not through a pipe.
- Prefer differential reproduction on the same source before and after the
  change. Use a detached worktree for the other side rather than moving a live
  working tree.
- Resolve every commit id, digest, path, and count with the relevant tool before
  writing it. Do not copy old measurements forward.
- When adding tests, verify the collected count increased as expected and that
  a deliberate negative control makes the check fail.
- If diagnostic ordering, precedence, or rule citation may move, compare every
  affected case's result and cited rule across both binaries; an unchanged
  failure set is insufficient.
- A peer report is a lead, not evidence. Reproduce it or label the unverified
  part. If a probe did not isolate the claim, write `not measured`.
- Every new check states what a green run does and does not establish.
- A test earns its runtime with its purpose, never with its duration: a slow
  test is not thereby a thorough test. State what each test proves and prove
  it in the minimum time that does. Unjustified repetition — xN loops with no
  stated reason, exhaustive sweeps where a boundary sample proves the same
  property — is forbidden. Setup identical across scenarios is built once and
  shared, unless the setup's isolation or repetition is itself the property
  under test (a determinism assertion legitimately compiles twice). Measured
  exhibit: ten scenario tests at 136s each collapsed to 136s total once the
  shared artifact stopped being rebuilt per test, with zero assertions lost.

### The failures that look like success

Most defects announce themselves. A handful do not, and every one of them was
found here by a deliberate question rather than by a gate, because **their
failure mode is success**: a conformance case that passes while testing
nothing, a check that cannot fail, a transform verified against its own
output, an operation performed against a baseline that no longer describes
reality. Nothing that watches for failure sees any of them.

The one habit that reaches all of them is to **prefer the observation that
separates two hypotheses over one consistent with the hypothesis you already
hold**. Before running a check, ask what result would make you believe the
other thing; if no result would, the check is decorative. Worked instances:

- A clean working tree *and* HEAD containing the fix — either alone is equally
  consistent with the fix having been destroyed.
- A test that MOVED to a different error versus one that STAYED PUT: moving
  means the fix worked and a second cause is underneath; staying means it did
  not work. The pass count is identical either way.
- Breaking a check in each direction it can fail, not once. A wrong value and
  a missing entry should fail differently; proving both is what separates a
  real check from a decorative one.

Three corollaries: run a transform against the input it should have handled,
never against its own output — a migrator or renderer checked on what it
produced is a fixed point and always agrees with itself. A mask's fix is
itself a probe — read the run immediately after removing one instead of
treating it as confirmation; a mask means the number of hidden problems is
unknown, never one. When a migrated case behaves oddly, read the migration
diff before the compiler — the program may have stopped being the program the
case was written about.

When writing rules like these, state the **property** that produces the
failure, not the causes you happen to have met; a cause list is wrong in both
directions at once.

## Specification-change workflow

Use this workflow for a real language gap named by the `ACTIVE` plan or by
the branch's chartering direction. Candidate preparation on the branch is
non-authoritative and needs no approval; owner approval happens at merge,
and the merge's activation commit is what activates.

1. **Bound the delta.** Consult the relevant live MCTS node and rejected
   alternatives with the `mcts-mem-use` skill. Inventory grammar, names,
   types, ownership, effects, runtime behavior, checks, diagnostics, ABI,
   conformance, compiler, examples, and documentation as changed, unchanged,
   or not applicable.
2. **Draft the candidate in place.** Edit `spec/kernel-spec.md` on the
   branch and declare it: the status line becomes
   `Status: CANDIDATE vN+1 supersedes vN <sha256-of-vN>`, where the digest is
   the activation-chain tail. Under candidate mode the full gate runs green
   on the branch throughout drafting; the candidate's own digest is not
   recorded until activation. Never resolve the stable spec with `ours` or
   `theirs`; a rebase onto a moved chain tail recomputes the declaration.
3. **Prepare evidence.** Derive positive, negative, and near-miss
   expectations. For grammar changes, run the two-path production verifier
   against the predecessor bytes (from the activation commit or the
   versioned archive once it exists) and the candidate. Assemble the exact
   diff, impact inventory (the `whitefoot-spec --index` reference graph is
   its mechanical source), protected changes, verifier output, accepted-set
   risk, and the complete candidate SHA-256.
4. **Present at merge.** The candidate rides the branch in CANDIDATE status
   through as many revisions as the work needs, each keeping the branch
   gate green. When the branch requests merge, the packet carries the
   owner-facing explanation and the exact digest of the final candidate.
   Any byte or scope change after presentation, including a rebase onto a
   moved chain tail, re-enters review. The owner's merge approval is
   recorded in `governance/APPROVALS.md` as part of the merge.
5. **Activate atomically at merge.** After approval, one activation commit
   concludes the rebased branch: archive `main`'s outgoing bytes as
   `spec/kernel-spec-vN.md`, failing if that path exists; flip the status
   line to `Status: ACTIVE vN+1`; append the chained record
   `ACTIVE-SPEC: vN+1 <new-sha256> <previous-sha256>`; regenerate the
   identity module (`whitefoot-spec --emit-identity`) and grammar tables;
   land conformance changes and derived material together; then
   fast-forward `main`. Valid but unsupported behavior remains unsupported,
   never rejection.
6. **Verify and close.** The gate must be green on both sides of the
   activation commit. Recompute the installed digest independently, inspect
   the impact rows, rerun the frozen real consumer, and record any durable
   decision in MCTS.

Released versioned specification archives are append-only and are never
edited, renamed, or deleted.

## Protected compliance workflow

This branch covers every protected compliance addition or change defined in
the approval boundary, even when the active specification does not change.

1. Reproduce the need against the active spec and distinguish a bad expectation
   from a compiler defect, unsupported capability, or tool failure.
2. Prepare the smallest exact candidate and a before/after audit listing every
   path or case id, source or manifest change, verdict, status, rule, coverage,
   actual behavior, collection count, and active-spec basis.
3. Land the change on the branch without waiting, flag it in the batch
   record the moment it lands, and carry the explanation — its compliance
   and accepted-set implications and the exact boundary — into the merge
   packet. Owner approval happens at merge and is recorded in
   `governance/APPROVALS.md` as part of the merge; a changed byte or scope
   re-enters review.
4. On the branch, run the per-case differential, adapter or canonical
   runner, coverage checks, and the complete repository gate; the packet
   reports all before/after totals.

When a protected evidence change follows a spec change, combine it with that
specification approval packet so the owner sees the language and corpus impact
together.

Never change a verdict, status, rule citation, coverage row, or baseline merely
to make a gate green. A compiler limitation, internal error, timeout, or
unsupported feature cannot rewrite normative expectations.

## Verification and closure

Use `/Users/bytedance/do_not_scan` for scratch files and test artifacts.

Run `make -C compiler check` before and after compiler work, and `make
check` before committing a completed slice. Read exit codes directly, never
through a pipe. A green gate states only what it exercises. Keep every written
claim executable, and lower a partial operation only after machine proof of its
domain; optional facts may not change acceptance, cleanup, output, or claim
behavior.

At closure, put semantics in the spec, implementation facts in code/tests
and the compiler README, measurements in the canonical results record,
durable decisions in MCTS, protected approvals in the ledger, and the batch
outcome in the batch record. Update the Direction Outline and the Current
Plan once, rather than copying status into every document.
