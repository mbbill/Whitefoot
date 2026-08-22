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

- `docs/roadmap.md` records the project's current landscape: active
  specification, direction status, gaps, evidence links, and candidate
  projects. `docs/current-plan.md` records the latest high-level plan. Neither
  file grants or withholds permission to work on a branch.
- The active specification at `spec/kernel-spec.md`, named by
  `docs/roadmap.md`, defines the language. Compiler behavior, tests, archived
  code, and design prose do not.
- `docs/constitution.md` records project law and `docs/patterns.md` records writer
  forms.
- `docs/done/`, architecture dossiers, `mcts_mem/`, and
  `archive/governance/decision-log.md` preserve current or historical evidence
  and rationale. They do not define live approval or workflow requirements.
  Any imperative process wording retained in those evidence artifacts is
  historical and superseded by the four rules below.
- `governance/APPROVALS.md` is an append-only historical record and the home
  for the merge-time records required below. Historical entries do not impose
  current process.

Read only the material relevant to the current task. Do not turn historical
research into an implied implementation requirement.

## Branch and main boundary

These are the complete approval and workflow rules:

1. Any change may be made on a work branch without approval, including plans,
   repository layout, specifications, conformance evidence, gate wiring, code,
   tests, and documentation.
2. Every change merged into `main` requires owner approval of the exact
   revision to be merged.
3. The exact revision merged into `main` must pass all repository tests through
   the canonical `make check` entry point before the merge.
4. If the merge changes `spec/kernel-spec.md` or conformance evidence,
   `governance/APPROVALS.md` records the content the owner approved as part of
   the merge. A specification record identifies the exact specification bytes;
   a conformance record identifies the exact added, modified, deleted, or
   renamed conformance content and its before/after boundary.

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
pause on every file. Canonical `make check` enforces append-only versioned
specification archives; the optional hook installed by `make install-hooks`
only reports the same class of mistake earlier.

## Specification and test integrity

- The active kernel specification lives at `spec/kernel-spec.md` and is
  editable on a work branch. Released flat `spec/kernel-spec-vN.md` archives
  are immutable, and the active file's identity is carried by the chained
  digest and archive gates. A spec/compiler discrepancy is a technical defect;
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

## Data safety

Preserve unrelated user changes in a dirty worktree. Never discard, overwrite,
or rewrite work outside the requested change boundary.
