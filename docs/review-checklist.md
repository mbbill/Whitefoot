# Task completion review

Run this checklist when a repository-changing task is ready to finish, before
reporting completion or handing over its result. Use a fast reviewing agent
for the applicable checks. Fix concrete findings, then recheck the affected
items. Review is part of completing the work; approval and merge conditions
remain in [AGENTS.md](../AGENTS.md#branch-and-main-boundary).

## Review input and result

Read the task's requested outcome and constraints, the complete task diff
(including uncommitted and new files), and the actual validation results.
Identify the base and reviewed revision; for uncommitted work, identify the
working-tree changes as well. Read changed sections in context, the relevant
document roles below, and directly affected definitions, callers, or cases.
Do not load the whole repository or require a separate review packet.

Judge the artifacts against the task and current owners, not just the author's
summary. Mechanical checks cover their encoded properties; this review checks
meaning, placement, and omitted dependent updates. Neither reconstructs an
unrecorded reason or certifies the soundness of a design argument. Flag such
uncertainty for the implementing agent rather than inventing missing evidence.

Check the applicable items below. Skip sections whose trigger is absent.
Use `pass`, `finding`, `unverified`, or `not applicable`; missing evidence is
not a pass. When the task changes a review rule or an expected result, compare
its previous form with the requested change rather than judging only against
the newly edited rule.

Put a compact report in the existing PR, or the task reply when there is no PR.
Use the [PR template](../.github/pull_request_template.md)'s three bullets:

- **Scope:** reviewer/model, base and head, checked groups and any skipped
  groups or unreviewed parts. A scoped review does not certify the whole PR.
- **Checks:** commands actually run and their results; distinguish the full
  gate from focused checks and identify the tested revision.
- **Findings:** remaining issues and unverified items, or none within the
  reviewed scope. Each finding names its item ID, file/line, offending text or
  missing evidence, and a short reason; quote both sides of a contradiction.

Summarize clean groups together rather than listing every passed item. Link
detailed evidence when needed instead of copying logs, stage tables or the
checklist. Mark a question needing design judgment `unverified` and return it
to the implementing agent; do not guess or redesign the project. Do not create
a repository audit log. Later edits invalidate review of the affected content,
not unrelated completed checks.

## Document roles

Use the row for the file being edited. A brief summary or relevant technical
explanation is useful; duplicating another document's changing inventory or
mixing in the editing conversation is not. A file needs no new status banner
or self-description merely to satisfy this table.

| Document | Content that serves its reader | Content that does not belong |
|---|---|---|
| Root README | Project introduction, getting started, navigation | Detailed compiler inventory, a second specification, task history |
| `docs/constitution.md` | Complete statements of purpose, chosen objectives, obligations, prohibitions, tradeoffs, and applicable conditions that can guide a choice and test its grounds | Who requested an edit and when, agent conversations, implementation progress, maintenance instructions, abbreviated labels in place of clauses, per-clause usage checklists, a selected mechanism asserted as an inevitable consequence of the purpose |
| `spec/kernel-spec.md` | Normative syntax, semantics, judgments, boundaries and relevant examples | Compiler convenience presented as law, task status, editing history |
| `docs/todo.md` | Known compiler defects and open costs, removed when fixed | Decisions, unsupported capabilities, task progress |
| `docs/patterns.md` | Writer problems, usable forms, examples, applicability and costs | Additional acceptance rules, unsupported universal performance claims, project administration |
| `AGENTS.md` / `CLAUDE.md` | Agent entry, project constraints, authority, workflow and pointers to detailed guidance | Research narration, a second detailed checklist or compiler inventory |
| `docs/practice.md` / this checklist | Engineering methods and decision-update triggers / completion checks and document boundaries | Language semantics, task-specific outcomes, new owner approval requirements |
| `research/`; `governance/spec-evolution/` | Questions, alternatives, designs, change proposals, reproducible experiments, results and limitations; the research README provides navigation | Task completion as technical evidence, a proposal presented as an implemented rule |
| `docs/ideas.md`; `docs/bargain.md`, `docs/why-whitefoot.md` | Candidate mechanisms; explanatory essays and dated rationale respectively | A live work queue, invented present-day measurements, contributor process inserted into an essay |
| `docs/roadmap.md` | Long-range reference directions | Required task sequencing, approval, an authoritative current capability inventory; routine work does not require updating it |
| `docs/ongoing/` | Existing, bounded implementation notes for their named subsystem | A new per-task reporting system, a second project-wide status inventory |
| `design/` | Live design decisions with their reasons and refused alternatives, one log entry per approved tree change, and the procedure that maintains them | Module inventories, implementation transcripts, task progress, history |
| PR description | This change's problem, resulting behavior, selection grounds, validation and limitations | An obsolete description of an earlier diff, a new permanent source of project rules |

Citation boundaries:

- Definitions point to their current owner; technical claims point to the
  specification, source/cases, a relevant design, or reproducible evidence.
  The linked passage must support the claim, not merely discuss the topic.
- The constitution, specification, writer patterns and explanatory essays
  must be usable without consulting the design trees.
  Do not link to `design/` from those documents or use it as their
  authority. State the relevant principle or explanation in the document and
  cite direct technical evidence when needed.
- Maintainer navigation (README, agent instructions, practice, research index)
  may point to the design trees. Research records, derivation evidence,
  and PRs may refer to relevant decisions as historical rationale, not as
  language definitions or proof of an empirical claim. A tree node may cite
  specifications, designs and evidence in its reason. Prefer the directory
  for navigation; node paths can move when decisions are replaced.
- Historical references may name their historical versions and conditions.
  Current guidance uses the active specification's stable path. Frozen
  archives keep their historical content; do not rewrite them to look current.

## A. Scope and repository layout — every change

Source: [repository hygiene](../AGENTS.md#repository-structure-and-hygiene).

- [ ] **A1 — Task fit.** Each changed artifact serves the requested outcome or
  a necessary dependency. Unrelated user work is preserved; the completion
  claim does not silently drop a requirement or leave an advertised fix as a
  stub, TODO, or unconnected implementation.
- [ ] **A2 — New paths.** For each added file or directory, identify its
  consumer, existing home, and removal condition. No unnecessary root entry,
  parallel document, copied implementation, per-task report, or unused helper
  was added. An established directory is not a dumping ground either.
- [ ] **A3 — Connections.** Added scripts and maintained tests have a real
  caller or collection path; documented commands name existing targets.
  Moves/deletions update affected imports, links and gate wiring. Superseded
  live material is updated or removed; frozen history and useful dated
  evidence follow their own retention rules.
- [ ] **A4 — Artifact hygiene.** No scratch output, personal filesystem path,
  credential, or machine-local setup has leaked into the diff. New/modified
  artifacts use English. Tooling uses the project's native path unless its
  different role justifies another tool; the two agent entry files agree.

## D. Documentation — changed prose, comments or examples

- [ ] **D1 — Purpose.** Each added or changed passage serves the containing
  document or code's reader. Check against the role table, including editorial
  history and process instructions inserted into substantive documents. For
  constitutional changes, check that complete clauses state the relevant
  obligations and conditions; a chosen prohibition is not merely a report of
  current implementation behavior.
- [ ] **D2 — References.** Changed references resolve to the intended file,
  heading or symbol, obey the citation boundaries, and support their claim.
  A correct relative path does not make an inappropriate citation acceptable.
- [ ] **D3 — Current meaning.** Changed claims agree with their owning source
  and directly affected guidance. Update an obsolete standing statement, not
  only a later correction beneath it. Distinguish a goal, proposal, specified
  behavior, implemented capability and dated measurement; historical positions
  are not conflicting current instructions merely because they differ.
- [ ] **D4 — Usability.** Instructions name real commands and prerequisites.
  Changed runnable examples have been checked through the ordinary path;
  fragments have enough surrounding context and are not offered as complete
  programs. No duplicated changing status or version identity needs another
  synchronized update. Do not invent new style conventions during review.

## C. Code and cases — implementation or behavior changes

Source: [compiler rules](../AGENTS.md#compiler-rules) and
[test integrity](../AGENTS.md#specification-and-test-integrity).

- [ ] **C1 — Observable case.** A bug fix has a case that distinguishes the
  faulty behavior from the intended result; a new behavior has coverage for
  its normal use and relevant failure/boundary. Identify the case and result.
  Existing coverage is sufficient if it exercises the change. A pure refactor
  can name its existing coverage; prose-only edits need no invented test.
- [ ] **C2 — Independent expectation.** The expected result follows the task
  contract or specification, not the implementation's current output. For a
  regression, show the case fails before and passes after, or explain why the
  old run is unavailable and how the case detects the fault. A checker's
  negative case fails for the intended reason, not an earlier unrelated error.
- [ ] **C3 — General path.** The diff implements a grammar/semantic rule or
  general runtime operation. No function, project, source shape or test name
  selects a special acceptance/lowering path. No duplicated rule, unexplained
  forwarding layer, or fallback conceals an unsupported capability.
- [ ] **C4 — Safety boundary.** Inspect changed acceptance and proof paths for
  added Rust `unsafe`, weakened contracts, runtime substitutes for required
  proof, impossible-case returns, and timeout/fuel/heuristic acceptance limits.
  Changes to optimization facts retain acceptance and behavior with facts off;
  relevant evidence is supplied when those paths change. If a semantic safety
  question cannot be settled by local inspection and cases, flag it for deeper
  review rather than certifying soundness.

## T. Specification and checks — changed language rules, design premises, tests or gate wiring

Source: [specification and test integrity](../AGENTS.md#specification-and-test-integrity).

- [ ] **T1 — Language evidence.** For a specification amendment, the outgoing
  active bytes are archived unchanged, released archives are untouched, and
  the new declaration and title agree. The change declares the [META-5] delta
  (rules, tokens, spellings, exceptions) and evidence/minimality selection
  ground. Affected cases/verdicts, generated syntax, compiler and documentation
  follow the amendment. For changed rules or constitutional premises, apply
  R3–R4 below. For conformance changes,
  the PR explains the normative expectation and how the changed evidence tests
  it. [META-5] is defined in the
  [active specification](../spec/kernel-spec.md#20-spec-meta-rules-ci-checked).
- [ ] **T2 — Preserved checks.** Every removed, skipped, narrowed, regenerated
  or weakened test/check has a technical reason consistent with the requested
  change. An implementation gap, crash, timeout or unsupported feature has not
  been relabeled as normative source rejection. Check assertions and collection
  wiring as well as filenames and pass counts.
- [ ] **T3 — Effective checks.** New cases actually run. New or changed check
  machinery demonstrates that a representative wrong result or missing input
  is detected. Reusing established machinery needs no new mutation campaign.
  Active build/test/tool paths do not depend on `archive/`.

## R. Decisions — changed choices, premises or relevant evidence

Source: [decision practice](practice.md#decision-work). These are checks on
observable artifacts, not a claim to know an agent's internal reasoning or
a second design review. A routine fix under unchanged design can skip this
group; absence of a tree diff does not establish that the group is
inapplicable.

- [ ] **R1 — Stated ground.** A material choice has a retrievable explanation
  of its purpose, required properties, assumptions, alternatives actually
  considered, selection reason, and remaining uncertainty. Relevant prior
  objections are addressed. Distinguish deductions, observations, and
  provisional choices; a constitutional citation alone does not select a
  particular mechanism. Flag a substantive question for the implementing
  agent rather than inventing a rationale. Unresolved proposals and assumptions
  have not become settled decisions through wording alone. Before real project
  adoption, internal adaptation costs have not been used to reject a language
  change, and test/example frequency has not been passed off as real usage.
- [ ] **R2 — Discriminating evidence.** An experiment used to select a design
  states what comparison could distinguish it, the conditions and protected
  requirements, and the actual outcome. A claim of a prediction made before
  measurement has an inspectable prior criterion; otherwise label the finding
  exploratory or the timing unverified. A changed requirement, inconclusive
  result, or trial on one model has not been reported as broader success.
- [ ] **R3 — Consumed changes of reason.** For changed objectives, premises,
  rules, cited sources, or evidence meeting a reopening condition, use the diff
  and direct references to check the named affected set. The explanation says
  which choices still stand, stand on different grounds, or need replacement.
  Their current owners agree with the design tree or the explicitly recorded
  pending amendments that revise it. Remaining questions have a concrete
  source; a log entry does not supersede contradictory standing guidance.
  Do not require an unrelated project-wide sweep.
- [ ] **R4 — Maintained tree.** Added, changed, or retired rules and changed
  grounds have corresponding design-tree updates under M1 to M3, and the
  sources a decision cites resolve and support the stated scope. A log entry
  or a lint success is not proof that a cited argument is true.

## M. Design tree — changed decisions or tree nodes

Use `design/skill/SKILL.md`; this section does not replace it.

- [ ] **M1 — Design, tree, and amendments.** A design-phase delivery includes
  the complete design and its proposed revision to the current tree; phase 1
  is complete only after the owner's confirmation. Implementation carries
  the tree revision the owner ruled, with a concise traceability log, or an
  amendment under `design/amendments/` for every decision the agent made on
  its own. Design changes during implementation follow the same procedure;
  none is only in code.
- [ ] **M2 — Correspondence.** The procedure's correspondence checks ran over
  the tree, the amendments, and the code or specification diff at
  implementation delivery, and every finding is resolved or listed. A
  design-phase review applies the design gate without requiring a finished
  implementation.
- [ ] **M3 — Form.** `make design-lint` passes on the head revision.

## V. Validation and handoff — every change

Source: [evidence practice](practice.md#evidence-guidance) and
[merge boundary](../AGENTS.md#branch-and-main-boundary).

- [ ] **V1 — Actual checks.** Applicable checks ran on the delivered content;
  commands, results and limitations are available. Missing tools, unrun tests,
  skips and expected failures are reported accurately. Focused success is not
  described as a complete gate; review does not rerun unaffected green tests.
- [ ] **V2 — Supported claims.** Counts, paths, revisions and quoted results
  were checked. A performance claim names workload, environment and comparison;
  a causal claim has isolating evidence. A historical result or another agent's
  report is not silently presented as a fresh independent measurement.
- [ ] **V3 — Delivery.** The reply/PR describes the current result and remaining
  limitations. For spec/conformance changes, it explains what changed and its
  selection ground. If merging is requested, verify owner approval and root
  `make check` for the exact merge tree under the existing four rules; neither
  a fast review nor a focused test run substitutes for them.
- [ ] **V4 — Existing PR updated.** Before reporting completion, commit and
  push the reviewed task changes to the existing PR branch without waiting
  for a reminder. Verify its remote head contains the delivered revision and
  its description and validation reflect the current diff. Link the PR in
  the reply. A failed publication is an explicit delivery blocker, not a
  completed update. This check runs after the content review and push.

Use existing checks when applicable: `git diff --check` for patch whitespace;
`make static` for repository invariants, immutable spec archives and live spec
references; `make -C compiler format lint` and the
[focused compiler commands](../README.md#verification) for
code. `make static` does not check document purpose or all links, and compiler
`docs` builds Rust API documentation, not this prose checklist. The root
[Makefile](../Makefile) owns the full gate inventory; the design-tree checks
follow `design/skill/SKILL.md` and are covered by M1-M3.
