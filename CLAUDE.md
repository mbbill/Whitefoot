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

- `docs/roadmap.md` is the sole source for current roadmap, status,
  authorization, and next work.
- The active numbered specification named by `docs/roadmap.md` defines the
  language. Compiler behavior, tests, archived code, and design prose do not.
- `docs/constitution.md` records project law and `docs/patterns.md` records writer
  forms.
- Consult the relevant live `research/design-tree/` node and its rejected alternatives
  before a nontrivial design change.
- Architecture dossiers and `governance/decision-log.md` are historical design and
  decision records. They can explain why something exists, but they cannot add
  current work or override `docs/roadmap.md`.

Read only the material relevant to the current task. Do not turn historical
research into an implied implementation requirement.

## Goal discipline

Before starting or expanding work, answer:

1. What concrete compiler capability or experiment will this unlock?
2. Why is it the next work in `docs/roadmap.md`?
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
  deliberate exception is `spec/`, which is append-only by design.
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
entry. Append-only `spec/` is enforced by a pre-commit hook (installed with
`make install-hooks`); everything else is upheld by discipline.

## Specification and test integrity

- The numbered kernel specification is append-only, enforced by a pre-commit
  hook (`make install-hooks`): a released `spec/kernel-spec-v*.md` is never
  edited, renamed, or deleted. Amending the language is allowed, with care — a
  change batch goes into a new version file. A spec/compiler discrepancy stops
  the affected work for investigation; implementation convenience never selects
  language behavior.
- Before proposing a spec change, verify the new grammar with the grammar
  verifier — a small tool (to be built) that reuses the compiler's own lexer and
  parser to check the grammar constraints. See `docs/roadmap.md`.
- When the spec changes, bring everything derived from it to the newest version
  in the same work: conformance cases and verdicts, the reference model, the
  lexer/parser and generated syntax data, tests, and docs. This consistency is
  your responsibility and is deliberately not machine-enforced.
- Do not silently weaken derived material to make a check pass. Editing a
  conformance verdict, deleting a failing test, or regenerating evidence to go
  green is a governance breach even though no script blocks it. Add tests
  freely; change or remove existing conformance or reference material only with
  owner agreement and a decision-log entry.
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
- Keep commits cohesive. Record material owner decisions and completed roadmap
  transitions briefly in the append-only decision log; do not use agent
  instruction files as a status log.
- Delegate only concrete, independent work. Integrate and review delegated
  results against the same goal and relevance rules.
