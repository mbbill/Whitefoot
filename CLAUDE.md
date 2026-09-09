# Whitefoot — agent instructions

Whitefoot is a systems language designed as a harness for AI agents, with
machine-checked source proofs. Build a real, general compiler and executable
programs that expose language and performance weaknesses. Prioritize useful
end-to-end capability, required correctness and safety, clear implementation,
and sufficient evidence; defer infrastructure and polish without a current use.

## Authority and reading

Read only the current owners relevant to the task:

- `spec/kernel-spec.md`: language authority. Code, tests and design prose do not
  override it. `compiler/README.md`: implementation map and known gaps.
- `docs/constitution.md`: objectives and tradeoffs.
  `spec/derivation/derivation-ledger.md`: current rule-to-ground index.
  Concrete choices need their own technical grounds.
- `docs/patterns.md`: writer guidance. `docs/practice.md`: engineering,
  evidence and decision methods. `docs/review-checklist.md`: completion checks.
- `research/investigations/`: designs, measurements and rejected alternatives.
  `mcts_mem/`: settled decisions; maintain with the current `mcts-mem-use` skill
  and its verification instructions. Do not duplicate the skill's tool setup.
- `docs/roadmap.md` is reference only: no task sequencing, approval or routine
  updates depend on it. Archives are frozen historical evidence, not current
  instructions; do not write to or cite retired `archive/done/` task records.

Follow [decision practice](docs/practice.md#decision-work): read current grounds
at start, state reasons and experimental criteria before choosing, and update
material dependents when a choice or its grounds change. Routine fixes need no
new decision record. Keep facts in their owning document, not parallel summaries.

Before completion, run applicable checks and another agent's
[completion review](docs/review-checklist.md), including actual user-path delivery
and work missing from the diff. Fix findings. For an existing PR, commit and push
each completed round to that same branch, refresh its description and validation,
and verify its remote revision before reporting completion. Link the PR; report
any publication blocker and unpublished work. PR updates do not authorize merging.

## Branch and main boundary

1. Work branches (anything except `main`) need no approval for ordinary changes,
   including specifications, conformance evidence, code and repository rules.
   This does not waive the explicit research exception requirement below.
2. Every merge into `main` requires owner approval of the **exact revision**.
3. That exact revision must pass the root **`make check`**, covering all maintained
   compiler/research tests, static checks and the full native conformance adapter,
   including its ordinarily ignored case. Focused tests are not a substitute.
4. If the merge changes the active specification or conformance evidence, the PR
   states what changed and its selection ground for that exact revision.

The exact revision means the complete tree entering `main`; any change requires
fresh approval and a successful full gate for the new tree. Conformance evidence
includes cases, manifests, runners, adapters, and wiring affecting collection or
interpretation. There is no approval ledger or additional plan, audit, worktree,
commit-shape or historical "ratification" requirement.

## Repository structure and hygiene

**Research is not an alternative implementation home.** `research/` is for
exploration and validation: experiments, benchmarks, independent oracles, native
references, measurements and design evidence. Substantive Whitefoot compiler
and runtime components **MUST** live in the maintained compiler implementation
and use its normal integration path. Agents **MUST NOT** create or maintain a
separate research implementation without explicit owner permission for that
specific exception. "Prototype", "control", "temporary", or general instructions
to research, optimize or work autonomously **DO NOT** grant permission.

Before such an exception, explain its scope, why the maintained implementation
cannot serve the experiment, and its integration or retirement path; obtain and
record explicit permission in the existing PR or investigation. Even authorized
research code is not delivery of a requested compiler capability until the
normal path uses it.

- Use existing homes. New files need a concrete consumer, purpose and removal
  condition; new root entries require a structural reason. No bulk dumps, unused
  scripts, duplicated implementations or unmaintained documents.
- Supersede in place; update callers, links and tests together. Preserve frozen
  archives and useful dated evidence. Do not relocate load-bearing paths or
  undertake structural churn merely for tidiness.
- Prefer native tooling. Do not reimplement compiler checks in Python; reserve
  it for genuinely independent tooling such as conformance. Wire scripts to a
  maintained caller or remove them after their explicit one-shot use.
- No active source, build, test or tool may depend on `archive/`.
- New/modified artifacts, identifiers, comments, diagnostics and filenames use
  English. Keep AGENTS.md and CLAUDE.md identical.

## Specification and test integrity

- The active specification is editable on a work branch; released
  `spec/kernel-spec-vN.md` archives are immutable. An amendment archives the
  outgoing bytes unchanged and retitles/redeclares the active file vN+1 in the
  same change. `compiler/build.rs` derives identity; the gate checks consistency.
- Update affected compiler behavior, conformance, syntax data, tests, docs and
  rule grounds together. Implementation convenience never selects semantics.
- **Never weaken a verdict, delete/disable/ignore/narrow a check, or regenerate
  evidence just to pass.** Deliberate retirement needs an honest technical reason.
  Unsupported capability, internal failure or timeout is not source rejection.

## Compiler rules

- Use safe Rust; do not introduce `unsafe`. Keep one general semantic/lowering
  path. Never special-case a function name, signature, source shape, project or
  test identity. Report unsupported capabilities explicitly.
- Accepted programs require proof of every partial operation's domain. No
  writer-accessible unsafe escape or runtime trap may replace proof. Proofs are
  erased and introduce no runtime branches, locks, dependencies or task edges.
- Acceptance uses no SMT. Automatic derivation is specification-fixed,
  deterministic and terminating; admitted families run to completion. No timeout,
  fuel, cumulative work budget, heuristic failure or hash order may decide
  acceptance. Harder proofs use explicit finite `use` steps, checked as written.
- Express required always-true relations as proof-only contracts/invariants.
  A branch may guard an operation only when its false edge is intended behavior;
  never add an impossible-case return merely to satisfy the checker.
- Keep facts-off compilation correct: optimizer facts may improve code, never
  change acceptance or semantics.
- Prefer simple implementations and normal collections. Optimize measured costs;
  keep files cohesive by responsibility, without forwarding-only layers.

## Data safety

Preserve unrelated changes in a dirty worktree. Never discard, overwrite or
rewrite work outside the requested scope.
