# Whitefoot — agent instructions

Whitefoot is a programming language designed as a harness for AI agents.
Its chosen systems-language design carries machine-checked source proofs.
Accepted programs
must make memory corruption, data races, uninitialized
reads, silent overflow, and every other unproved partial operation
unrepresentable. There is no writer-accessible unsafe escape or runtime trap.
Every partial operation is admitted only after machine proof of its domain.
The official compiler uses no SMT for acceptance: automatic derivation is
specification-fixed, deterministic, and terminating. Every admitted automatic
family runs to its specified completion; timeout, machine speed, solver state,
or a cumulative work budget never selects acceptance. Harder proofs arrive as
explicit finite `use` steps inside a local `invariant`, and the compiler checks
those written steps without rediscovering them. Proofs are erased before
lowering and may authorize check removal, optimization, and parallel
independence without adding runtime branches, locks, dependencies, or
scheduling edges.

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

- Work is not planned in a document up front. A selected direction goes to
  `research/investigations/<name>/` for its design, measurements, and rejected
  alternatives, and the decision that survives it is written to the design
  tree under `design/`.
- `docs/roadmap.md` is a reference map of long-range directions and candidate
  projects. It is not part of this loop: nothing waits on it, no step updates
  it, and it grants or withholds nothing. Read it for orientation; do not treat
  a line in it as a statement of what the compiler currently does, which is the
  specification's and the conformance results' to say.
- The active specification at `spec/kernel-spec.md` defines the language.
  The conformance results state what the compiler implements, `docs/todo.md`
  lists its known defects, and `design/compiler` records its decisions.
  Compiler behavior, tests, archived code, and design prose do not define the
  language.
- `docs/constitution.md` owns purpose, chosen objectives, tradeoffs, and
  conditional language-design principles; the design trees hold the concrete
  decisions with their reasons. Concrete choices need their own grounds, not
  just a constitutional ancestor.
  `docs/patterns.md` teaches writer forms without adding acceptance rules;
  `docs/practice.md` explains engineering and evidence techniques without
  adding approval or merge requirements. README is navigation, not a second
  specification or implementation inventory.
- `design/` holds the live design trees: `design/language` for language
  decisions and `design/compiler` for compiler decisions, each decision with
  its reason and its refused alternatives, and `design/log.md` with one entry
  per approved tree change. `design/skill/SKILL.md` owns the procedure: a
  line enters a tree only through the owner's ruling, a decision an agent
  makes on its own is an amendment beside the tree until the owner rules on
  it. DCR runs at the triggers defined by that skill. `mcts_mem/` is a frozen
  historical record that is not written to; its content is being moved into
  the trees and it is deleted when that is complete.
- Architecture dossiers, `archive/done/`, and
  `archive/governance/decision-log.md` preserve historical evidence and
  rationale. `archive/done/` is the retired per-batch record: frozen, not
  written to again, and not cited. A finished task is not evidence — a claim
  that needs support cites the specification, a conformance case, a measured
  result under `research/experiments/`, a design under
  `research/investigations/`, or a design-tree decision where the completion
  checklist's citation boundaries permit it. None of these
  defines live approval or workflow requirements. Any imperative process
  wording retained in those evidence artifacts is historical and superseded by
  the four rules below.
- `archive/APPROVALS.md` is the retired approval ledger, frozen with the rest
  of `archive/`. It recorded merge-time content when approval ran ahead of
  implementation; that loop is gone, and the specification's bytes are now its
  own identity.

Read only the material relevant to the current task. Do not turn historical
research into an implied implementation requirement.

Follow the four occasions in [decision practice](docs/practice.md#decision-work):

1. **Start or resume:** read the affected current owners; for a material choice,
   also recover the relevant constitutional aims and existing decision grounds.
2. **Choose:** state why a material choice fits its requirements and evidence;
   record a discriminating experiment's criterion before using it to choose.
3. **Update:** when a conclusion or its grounds change, update current guidance
   and material dependents in the same work. Record the design revision as an
   owner-approved tree change or a pending amendment under the design-tree
   procedure.
4. **Finish:** run applicable checks and another agent's
   [completion review](docs/review-checklist.md), handle findings under its
   response rules, and publish the result. This is the single completion
   review checkpoint; no separate review record is required.

Decision practice defines the material-choice boundary, reading and writing
locations, and affected-set procedure. Routine fixes under unchanged design
need no new decision record. Record reasons when choices settle, not by
reconstructing them at task completion.

Use a PR as the owner's ongoing review surface, including before implementation
begins. On an existing PR, commit and push coherent progress to the same branch
and keep its description and actual validation results current. Opening a PR
or publishing intermediate progress does not require a completion review.
Run that review at the checklist's completion triggers, then publish the
reviewed result before reporting completion. Do not wait for another request
to update the PR or leave the reviewable result only in the local worktree.
Verify the remote PR contains the delivered revision and link to it in the
reply. If publication fails, report the blocker and which changes remain
unpublished. Updating a work-branch PR does not authorize a merge into `main`.

## Branch and main boundary

These are the complete approval and merge rules:

1. Work-branch changes need no approval, including plans, repository layout,
   specifications, conformance evidence, gate wiring, code, tests, and
   documentation, except that changes to the live design tree require the
   owner's ruling under `design/skill/SKILL.md`. Unapproved design choices
   remain amendments while implementation continues. DCR findings require
   owner direction before follow-up changes under that skill's response rule.
2. Every change merged into `main` requires owner approval of the exact
   revision to be merged.
3. The exact revision merged into `main` must pass all repository tests through
   the canonical `make check` entry point before the merge.
4. If the merge changes `spec/kernel-spec.md` or conformance evidence, the
   pull request states what changed and its selection ground, answered against
   the exact revision being merged. There is no separate ledger: the
   specification's bytes are its identity, the released archives are
   immutable, and git is the history.

What the four rules mean exactly:

- **Work branch** is any branch other than `main`. Branch implementation
  proceeds without approval, including edits to a specification, conformance
  evidence, or these rules, subject to rule 1's live-tree and DCR response
  boundaries. Keep unapproved tree revisions as amendments while continuing
  authorized work.
- **Exact revision** is the complete tree that will enter `main`. If that tree
  changes after approval or after its successful test run, rules 2 and 3 apply
  to the new revision.
- **All repository tests** is the root `make check` target: the compiler build,
  format and lint, every maintained executable test target in the compiler and
  the active research experiments, the specification checks, conformance
  structure and coverage, and the full native conformance adapter including the
  case ordinary Cargo runs mark ignored. A file retained as a deferred or
  historical artifact that cannot run against the current toolchain is
  evidence, not a test target.
- **Conformance evidence** is `tests/conformance` case source and manifest
  content, its runner and adapter, and any collection or invocation wiring that
  can change which cases run or how their results are read.

A specification amendment lands as one change: the active file retitled and
redeclared vN+1, and the outgoing vN bytes archived as
`spec/kernel-spec-vN.md`. Its identity follows its bytes without being
recorded anywhere — `compiler/build.rs` derives it. The version number is
claimed on the branch and settled at merge: two branches archive the same
outgoing bytes under the same name, so only the new number collides, and the
second to merge retitles two lines and rebuilds. There is no candidate state;
a branch carrying an amendment is merge-ready when its gate is green.

No plan status, branch charter, batch record, worktree arrangement, audit,
packet, rebase method, commit shape, or other workflow step is an additional
approval or merge precondition. The technical rules below define correct
content; they do not create another approval point.

Terms such as *validation*, *ratification*, or *approved implementation* in
language and design artifacts describe technical evidence or trust state. They
do not authorize branch work or add a repository workflow step.

## Repository structure and hygiene

The repository root and every established directory are a curated, closed set.
The layout exists so the important things are found first — `docs/roadmap.md`, the
active `spec/`, and the `compiler/` — and so supporting material stays where a
reader expects it. Keeping that legible is a standing obligation, not a
one-time cleanup.

- A new top-level entry — a directory or file at the repository root — is a
  structural decision, not an implementation detail. Put new material in the
  existing directory that already owns its kind; create a root entry only when
  no existing home fits and it directly serves a current compiler capability
  or experiment.
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
  dossiers, or abandoned experiments beside their replacements. This applies
  to current guidance and replaceable implementation artifacts; frozen
  archives and useful dated evidence retain history under their own rules.
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
- No active source, build, test, or tool may depend on `archive/`.
- New and modified repository artifacts, identifiers, comments, diagnostics,
  fixtures, test names, and file names use English.

Follow this by judgment and keep moving; it is a standing rule, not a reason to
pause on every file. Canonical `make check` enforces append-only versioned
specification archives; the optional hook installed by `make install-hooks`
only reports the same class of mistake earlier.

## Specification and test integrity

- The active kernel specification lives at `spec/kernel-spec.md` and is
  editable on a work branch. Released flat `spec/kernel-spec-vN.md` archives
  are immutable. `compiler/build.rs` derives the active identity from its bytes,
  and `make check` checks identity consistency and archive immutability.
  A spec/compiler discrepancy is a technical defect;
  implementation convenience never selects language behavior.
- When the spec changes, bring everything derived from it to the newest version
  in the same work: conformance cases and verdicts, the lexer/parser and
  generated syntax data, tests, and docs. This consistency is your
  responsibility and is deliberately not machine-enforced.
- Do not silently weaken derived material to make a check pass. Editing a
  conformance verdict, deleting a failing test, or regenerating evidence to go
  green is a governance breach even though no script blocks it. Add ordinary
  compiler tests freely. Conformance cases, manifests, adapters, runners,
  collection wiring, and gate-integrity tests are conformance evidence for
  rule 4 above.
- Never delete, disable, ignore, narrow, or unwire a test or check merely to
  make `make check` green. A deliberately retired test must leave an honest
  technical explanation in the same change.
- Compiler capability, an internal error, a timeout, or an unimplemented
  feature is not a source-language rejection and must not rewrite normative
  expectations.

## Compiler rules

The compiler's implementation rules are its design decisions and live in
`design/compiler`, each with its reason. Before changing the compiler, read
the subtree you are changing and its ancestors; a decision the tree does not
cover is an amendment, never an edit to the tree, as `design/skill/SKILL.md`
prescribes.

## Data safety

Preserve unrelated user changes in a dirty worktree. Never discard, overwrite,
or rewrite work outside the requested change boundary.
