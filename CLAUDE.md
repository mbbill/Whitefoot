# Whitefoot — agent instructions

Whitefoot is a systems language for AI-written, human-approved code. Accepted
programs must make memory corruption, data races, uninitialized reads, and
silent overflow unrepresentable. There is no writer-accessible unsafe escape.
Every partial operation is admitted only after machine proof of its domain; a
written claim is the sole writer-reachable runtime trap and is never removed.

## Project goal

The target is a serious research compiler: general enough to implement the
real language, clean enough to evolve, and capable of compiling nontrivial
programs so we can test semantics and performance ideas quickly. It is not an
untrusted-input service or a stable LLVM-scale product.

“Good enough” means a real compiler rather than a source-shaped demo: one
general implementation path, compiler-independent correctness tests where they
help, useful diagnostics, an executable backend, and real programs that expose
language and compiler weaknesses.

When priorities conflict, use this order:

1. reach the next meaningful end-to-end language or performance experiment;
2. preserve semantic correctness and required safety checks;
3. keep the implementation understandable and easy to change;
4. add only the evidence needed to trust the current result; and
5. defer robustness, infrastructure, and polish that no current experiment
   needs.

If work does not help compile a real program, test a language rule, measure a
compiler idea, or remove the immediate blocker to one of those outcomes, it is
probably not the next work.

## Authority and reading

- `docs/roadmap.md` is the living Direction Outline and sole source for the
  project's current landscape: active specification, direction status, gaps,
  evidence links, and candidate projects. It does not sequence current work.
- `docs/current-plan.md` is the sole current execution proposal or approved
  plan and the sole source of plan-derived authority and sequencing. It must be
  derived from one outline revision and cannot authorize a direction the
  outline has not selected. `PROPOSED` authorizes no execution; the owner
  approves a high-level plan before it becomes `ACTIVE`.
- `docs/ongoing/` contains one numbered coordination record per live batch of
  work or distinct handoff boundary. `docs/done/` retains the same numbered
  record after integration. Both report how authorized work was carried out;
  neither selects, expands, or resequences work by itself, and neither
  replaces the canonical homes for facts, measurements, decisions, or status.
- The active specification at `spec/kernel-spec.md`, named by
  `docs/roadmap.md`, defines the language. Compiler behavior, tests, archived
  code, and design prose do not.
- `docs/constitution.md` records project law and `docs/patterns.md` records writer
  forms.
- Use the installed `mcts-mem-use` skill to consult the relevant live
  `mcts_mem/` node and its rejected alternatives before a nontrivial design
  change. Never edit the tree without first loading and following that skill;
  its formatting, provenance, paired-move, and lint rules are mandatory.
- Architecture dossiers are current or historical design evidence, and
  `archive/governance/decision-log.md` is a historical decision record. They
  can explain why something exists, but they cannot add current work or
  override the outline and current plan.

Read only the material relevant to the current task. Do not turn historical
research into an implied implementation requirement.

## Merge-approval boundary

Agents work on branches; `main` is the owner's. On a work branch every
change proceeds autonomously with no mid-flight approval wait — specification
revisions, conformance and compliance evidence, gate wiring, plan and roadmap
updates, code, tests, and docs. The single approval point is merging to
`main`: nothing reaches `main` except through an owner-approved merge,
integrated by rebase onto `main` plus fast-forward so history stays linear.
A branch may run many batches back to back (an overnight delegation is the
intended shape); direction from the owner, even conversational, is enough to
charter it.

A merge request presents one packet: the batch record(s); the branch-tip
gate result; for changed `spec/kernel-spec.md` bytes the complete candidate
SHA-256, diff, impact inventory, and verifier results; for protected
conformance or compliance changes the exact before/after audit; and any new
repository root entries, called out explicitly. Approval covers exactly the
presented bytes; a changed byte re-enters review. The approval is recorded
in `governance/APPROVALS.md` as part of the merge. A rejected or redirected
merge continues on the branch and re-requests.

Branch autonomy is freedom to iterate, not freedom to conceal: the
integrity rules below still bind on a branch, and silently weakening derived
material remains a governance breach there — the difference is that it is
caught by the batch audit and the merge review rather than by a mid-flight
wait, so the batch record must flag every protected-class change the moment
it lands.

## Goal discipline

Before starting or expanding work, answer:

1. What concrete compiler capability or experiment will this unlock?
2. Why is it authorized by an `ACTIVE` `docs/current-plan.md`, and which plan
   item and outline direction does it advance?
3. What is the smallest correct implementation?
4. Is it exercising a real compiler path or inventing machinery for a
   hypothetical one?
5. Has supporting work become larger or more complicated than the capability
   it supports?

If the work has drifted, stop. Sunk cost, prior effort, technical interest, and
internal consistency do not justify continuing the wrong task.

Do not build generalized frameworks, exhaustive protocol machinery, portable
identity systems, artifact replay, whole-compiler resource profiles,
transactional publication, release infrastructure, or compatibility machinery
unless a current experiment directly needs them. Use ordinary Rust structures
and private interfaces that can evolve.

Review must challenge relevance, proportionality, and sequencing as well as
technical soundness.

## Batch coordination

Work advances in lead-orchestrated batches, typically one working session
each. The owner sets direction; one lead session decomposes the batch,
dispatches executors (isolated worktrees for file-disjoint parallel scopes,
sequential work when coupled), reviews every returned diff, integrates, and
keeps the gate green. The lead assigns scope boundaries directly; there are
no claim files and no reservation protocol. Executors are tools, not
principals: they implement exactly their brief, report blockers honestly
with a reproduction, and never hack around one, weaken a check, or quietly
narrow a deliverable. One live worktree has one writer.

Each batch has one numbered record: `docs/ongoing/NNNN-short-slug.md` while
live, moved unchanged in number to `docs/done/` in the integration change.
Numbers continue one shared monotonic sequence and are never reused. A
record opens under an `ACTIVE` `docs/current-plan.md` item or under a
recorded owner direction that charters the branch; in the latter case the
record quotes the direction verbatim and the plan is brought up to date on
the branch, for review at merge. Planning work itself (roadmap or plan
revision) is not a batch and gets no record. A record is a boundary
document — the authority (plan item or chartering direction), scope,
approval classes touched, and at closure the outcome, landed commits,
verification, and audit dispositions.
Progress narration is forbidden; record updates ride the work commits they
describe, and a docs-only commit is exceptional. A batch handed to another
agent gets its record written before the handoff as the batch contract.

Every batch ends with the adversarial batch audit — independent finders
plus refuters — which enforces everything the machine-checked gates and the
merge boundary do not; no branch requests merge before its batches are
audited, and an external or unsupervised batch merges only after that
audit. An executor report is a lead, not evidence: the lead reproduces
load-bearing claims before they reach a record or an owner packet. Follow
the complete loop in `docs/WORKFLOW.md`.

## Repository structure and hygiene

The repository root and every established directory are a curated, closed set.
The layout exists so the important things are found first — `docs/roadmap.md`, the
active `spec/`, and the `compiler/` — and so supporting material stays where a
reader expects it. Keeping that legible is a standing obligation, not a
one-time cleanup.

- A new top-level entry — a directory or file at the repository root — is a
  structural decision, not an implementation detail. On a branch it may be
  created when genuinely needed, but it must be called out explicitly in the
  merge packet and survives only with owner approval at merge. Put new
  material in the existing directory that already owns its kind; if none
  fits, prefer asking over inventing a folder.
- Every new file, directory, script, or document earns its place before it is
  created. Be able to state what compiler capability or experiment it serves,
  which existing home it belongs in, and the condition under which it is
  removed. If you cannot name all three, do not create it.
- No bulk dumps. Do not add many scripts or documents in one change and leave
  them unmaintained. A script ships wired to a caller — a gate target or an
  explicit one-shot deleted after use; a document ships into an existing home
  and is kept current or deleted. Material with no owner and no reader is rot
  the moment it lands.
- Prefer native tooling; do not pollute the workspace with Python. The compiler
  is Rust — check it with `cargo test`, `cargo clippy`, and the workspace
  `forbid(unsafe_code)` lint, never a Python script that re-implements what
  cargo or the type system already does, and never a script forked per spec
  version. Python belongs only to genuinely compiler-independent tooling, such
  as the standalone conformance corpus. A new script must justify why the
  native path cannot do the job; if it cannot, it does not ship.
- Supersede in place. When new material replaces old, update, merge, or delete
  the old in the same change. Do not accumulate parallel versions, stale
  dossiers, or abandoned experiments beside their replacements. The single
  deliberate retained-history model is `spec/`: its active file is superseded
  in place, while its flat versioned archives are append-only.
- Keep important folders as clean as the root. The same discipline applies
  inside `spec/`, `compiler/`, `tools/`, `conformance/`, and the research
  directories. An important folder turning into a junk drawer is the same
  defect as a messy root.
- Reorganizing is not the goal; advancing the compiler is. Do not undertake
  large structural churn that no current work needs, and never relocate a
  load-bearing path merely for tidiness. Many paths here are pinned by the
  spec and test guard, reached by oracle scripts, or wired into a gate;
  moving them creates more breakage and rot than it removes. Prefer
  legibility — a clear map, a good name, a stated purpose — over relocation.

Follow this by judgment and keep moving; it is a standing rule, not a reason to
pause on every file. The one structural decision the owner always reviews at
merge is a new top-level entry. Append-only versioned specification archives
are enforced by a pre-commit hook (installed with `make install-hooks`);
everything else is upheld by discipline.

## Specification and test integrity

- `docs/WORKFLOW.md` is the sole operational workflow guide. It keeps ordinary
  project delivery as the main workflow and defines a separate, conditional
  specification-change workflow for genuine language gaps; implementing
  already-specified behavior and fixing compiler defects do not enter that
  branch. It also defines bounded parallel research. `governance/`, `spec/`,
  and `tests/conformance/` provide records, resources, and tools; none defines
  an independent update lifecycle.
- The active kernel specification lives at `spec/kernel-spec.md` and is
  superseded in place only through the specification activation workflow,
  whose approval and activation point is the merge to `main`. On a work
  branch the file carries CANDIDATE status and may be revised freely; the
  approved merge's activation commit archives `main`'s outgoing bytes flat
  as `spec/kernel-spec-vN.md`. Every such released archive is absolutely
  immutable and the pre-commit hook (`make install-hooks`) forbids editing,
  renaming, or deleting it. The active file's integrity is carried by the
  chained recorded digest and the landed archive gate. A spec/compiler
  discrepancy stops the affected work for investigation; implementation
  convenience never selects language behavior.
- Before requesting merge of a spec change, verify the new grammar with the
  native grammar verifier that reuses the compiler's own lexer and parser.
  Follow the candidate, evidence, merge-approval, and activation loop in
  `docs/WORKFLOW.md`.
- When the spec changes, bring everything derived from it to the newest version
  in the same work: conformance cases and verdicts, the lexer/parser and
  generated syntax data, tests, and docs. This consistency is your
  responsibility and is deliberately not machine-enforced.
- Do not silently weaken derived material to make a check pass. Editing a
  conformance verdict, deleting a failing test, or regenerating evidence to go
  green is a governance breach even though no script blocks it. Add ordinary
  compiler tests freely. Any addition, modification, deletion, or rename
  involving protected conformance or equivalent compliance evidence proceeds
  on the branch without waiting, but requires an exact before/after audit
  presented in the merge packet, owner approval at merge, and an
  approval-ledger entry recorded with the merge; the batch record flags each
  such change the moment it lands so the packet cannot omit it. This
  includes any change to canonical compliance gates, their collection or
  invocation wiring, or gate-integrity tests that can alter collection,
  interpretation, verdict, coverage, baseline identity, or whether the gate
  runs. If the correction requires different language semantics, present it
  in the same merge packet as the specification change.
- Compiler capability, an internal error, a timeout, or an unimplemented
  feature is not a source-language rejection and must not rewrite normative
  expectations.

## Compiler rules

- Use safe Rust; do not introduce `unsafe`.
- Implement language capabilities by grammar and semantic rule, never by
  function name, signature, source shape, project, corpus, or test identity.
- Keep one normal semantic and lowering path. A temporary unsupported
  capability must be explicit rather than misreported as invalid source.
- Never remove or weaken a written claim for speed. Required static proof is
  the only authority for admitting a partial operation.
- Keep facts-off compilation correct. An optimizer fact may improve an accepted
  program but may not change acceptance or claim execution.
- Prefer simple implementations and normal collections. Fix measured
  performance or resource problems instead of designing for imagined scale.
- Keep files cohesive and reviewable. Split by invariant-bearing
  responsibility, not arbitrary line counts or forwarding-only layers.
- No active source, build, test, or tool may depend on `archive/`.
- New and modified repository artifacts, identifiers, comments, diagnostics,
  fixtures, test names, and file names use English.
- `CLAUDE.md` (for Claude Code) and `AGENTS.md` (for Codex) carry the same
  project rules; `AGENTS.md` replaces Claude-Code-only tooling references
  (skills) with their in-repo equivalents and contains nothing
  Claude-Code-specific. Update both in the same change; the batch audit
  checks they have not drifted.

## Working practice

- Preserve unrelated user changes in a dirty worktree.
- Add the smallest practical regression before fixing a reproducible defect.
- Run `make -C compiler check` before and after compiler work.
- Run `make check` before committing a completed repository slice.
- A green gate states only the capabilities it exercises; it is not a
  completeness claim.
- Keep commits cohesive. Record the current landscape in `docs/roadmap.md`,
  current sequencing in `docs/current-plan.md`, durable design choices and
  rejected alternatives through the `mcts-mem-use` skill, and protected owner
  approvals in `governance/APPROVALS.md`; do not use agent instruction files as
  a status log.
- Delegate only concrete, independent work. Integrate and review delegated
  results against the same goal and relevance rules.
