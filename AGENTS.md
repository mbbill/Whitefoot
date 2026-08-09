# Whitefoot — agent instructions

Whitefoot is a systems language for AI-written, human-approved code. Accepted
programs must make memory corruption, data races, uninitialized reads, and
silent overflow unrepresentable. There is no writer-accessible unsafe escape.
Required runtime checks remain unless a machine-verified proof discharges them.

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
  outline has not selected. `PROPOSED` authorizes no execution; only `ACTIVE`
  or separate owner approval does.
- `docs/ongoing/` contains one numbered temporary coordination record per
  substantial in-flight task or distinct handoff boundary. `docs/done/`
  retains the same numbered record after terminal disposition. Both report how
  authorized work was carried out; neither selects, expands, or resequences
  work by itself, and neither replaces the canonical homes for facts,
  measurements, decisions, or status.
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

## Goal discipline

Before starting or expanding work, answer:

1. What concrete compiler capability or experiment will this unlock?
2. Why is it authorized by an `ACTIVE` `docs/current-plan.md` (or separate
   owner-approved bounded research), and which outline item does it advance?
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

## Parallel task coordination

Every substantial independently integrable task, or task with a distinct
integration or handoff boundary, has one short numbered file under
`docs/ongoing/` while it is live. Agents contributing to one deliverable share
its record; read-only reviewers do not create another. Publish the record to
the integration branch before substantial work so other workspaces can see it.

Task records move through at most three stages: `docs/planned/` holds tasks
decomposed from an `ACTIVE` plan but not yet claimed, `docs/ongoing/` holds
claimed in-flight tasks, and `docs/done/` holds terminal history. Claiming is
one commit that moves the file from `docs/planned/` to `docs/ongoing/`
unchanged in number and fills in owner, workspace, and base revision; the
first claim to land on the integration branch wins. A planned task may be
claimed only when its listed dependencies are terminal. Planned files
authorize nothing; when their plan is replaced, unclaimed tasks are deleted in
the same change unless the new plan carries them.

Names use `NNNN-short-slug.md`, with one zero-padded monotonically increasing
sequence shared by `docs/planned/`, `docs/ongoing/`, and `docs/done/`. After
refreshing the integration branch, allocate `max(existing numbers) + 1` in the
task's first registration commit. The number never changes or returns to the
pool. If two concurrent registrations choose the same number, the later
integration renumbers before landing.

Executor agents implement; they do not research, explore, redesign, or plan.
An executor reads `docs/WORKFLOW.md` and the claimed task's cited authorities,
works in an isolated worktree, executes exactly the written scope, and lands
changes only through lead review. A blocker, plan defect, or discovery outside
the cited authority stops the task and is reported honestly with reproduction
evidence — never hacked around, absorbed by weakening a check or test, or
quietly narrowed. An honest blocked report is a successful executor outcome.
The owner and the lead agent own direction, decomposition, review, and
integration; see the Execution agents section of `docs/WORKFLOW.md`.

At terminal disposition, move the same numbered file to `docs/done/` in the
integration change. Use final status `DONE`, `PARKED`, `REPLACED`, or
`ABANDONED`, and replace live current/next detail with a concise outcome,
landed commits, canonical evidence, validation, and remaining dependency or
follow-up links. A done record is frozen coordination history, not a second
roadmap, results report, decision tree, approval ledger, or source of authority.

Before starting, resuming, rebasing, or integrating a task, refresh the
integration branch and read relevant ongoing records. Their scope is advisory,
not a file lock. Textual overlap is allowed, but semantic or authority overlap
requires cross-linked dependencies and an explicit integration order; it never
resolves by last-writer-wins. A task record cannot authorize or expand work.
Follow the complete schema, refresh, rebase, numbering, and lifecycle rules in
`docs/WORKFLOW.md`.

## Repository structure and hygiene

The repository root and every established directory are a curated, closed set.
The layout exists so the important things are found first — `docs/roadmap.md`, the
active `spec/`, and the `compiler/` — and so supporting material stays where a
reader expects it. Keeping that legible is a standing obligation, not a
one-time cleanup.

- Do not add a new top-level entry — a directory or file at the repository
  root — without owner approval. A new root entry is a structural decision,
  not an implementation detail. Put new material in the existing directory
  that already owns its kind; if none fits, ask rather than invent a folder.
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
pause on every file. The one thing it reserves for the owner is a new top-level
entry. Append-only versioned specification archives are enforced by a
pre-commit hook (installed with `make install-hooks`); everything else is
upheld by discipline.

## Specification and test integrity

- `docs/WORKFLOW.md` is the sole operational workflow guide. It keeps ordinary
  project delivery as the main workflow and defines a separate, conditional
  specification-change workflow for genuine language gaps; implementing
  already-specified behavior and fixing compiler defects do not enter that
  branch. It also defines bounded parallel research. `governance/`, `spec/`,
  and `tests/conformance/` provide records, resources, and tools; none defines
  an independent update lifecycle.
- The active kernel specification lives at `spec/kernel-spec.md` and is
  superseded in place only through the specification activation workflow. At
  activation, its outgoing bytes are archived flat as
  `spec/kernel-spec-vN.md`; every such released archive is absolutely
  immutable and the pre-commit hook (`make install-hooks`) forbids editing,
  renaming, or deleting it. The active file's integrity is carried by the
  chained recorded digest and the landed archive gate. A spec/compiler
  discrepancy stops the affected work for investigation; implementation
  convenience never selects language behavior.
- Before proposing a spec change, verify the new grammar with the native
  grammar verifier that reuses the compiler's own lexer and parser. Follow the
  complete proposal, approval, activation, and closure loop in
  `docs/WORKFLOW.md`.
- When the spec changes, bring everything derived from it to the newest version
  in the same work: conformance cases and verdicts, the lexer/parser and
  generated syntax data, tests, and docs. This consistency is your
  responsibility and is deliberately not machine-enforced.
- Do not silently weaken derived material to make a check pass. Editing a
  conformance verdict, deleting a failing test, or regenerating evidence to go
  green is a governance breach even though no script blocks it. Add tests
  freely; change or remove existing conformance material only with owner
  agreement and an approval-ledger entry.
- Compiler capability, an internal error, a timeout, or an unimplemented
  feature is not a source-language rejection and must not rewrite normative
  expectations.

## Compiler rules

- Use safe Rust; do not introduce `unsafe`.
- Implement language capabilities by grammar and semantic rule, never by
  function name, signature, source shape, project, corpus, or test identity.
- Keep one normal semantic and lowering path. A temporary unsupported
  capability must be explicit rather than misreported as invalid source.
- Never remove a required source or runtime check for speed. Proof is the only
  authority for check elision.
- Keep facts-off compilation correct. An optimizer fact may improve an accepted
  program but may not change acceptance.
- Prefer simple implementations and normal collections. Fix measured
  performance or resource problems instead of designing for imagined scale.
- Keep files cohesive and reviewable. Split by invariant-bearing
  responsibility, not arbitrary line counts or forwarding-only layers.
- No active source, build, test, or tool may depend on `archive/`.
- New and modified repository artifacts, identifiers, comments, diagnostics,
  fixtures, test names, and file names use English.
- `AGENTS.md` and `CLAUDE.md` must remain byte-identical.

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
