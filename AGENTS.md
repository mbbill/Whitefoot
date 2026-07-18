# Whitefoot — agent onboarding

Whitefoot is a systems language for AI-written, human-approved code. Entire bug
classes are unrepresentable (memory corruption, races, silent overflow,
uninitialized reads); there is no unsafe escape. The checker's proofs double
as optimizer facts: safety checks are always on unless a machine-verified
proof discharges them — speed is earned by proof, never by weakening a check.

## Read order (do this before working)

1. `THE-PLAN.md` — the current map: beliefs, evidence ledger, ranked bets.
2. Tail of `optimizer-language-research/implementation/decision-gates.md` —
   the append-only lab log; the last ~10 entries are the live context.
3. `mcts_mem/` — the design-decision tree: before proposing any non-trivial
   design change, read the relevant node and its `.alt/` (what was already
   tried and why it lost).
4. As needed: `CONSTITUTION.md` (law), `PATTERNS.md` (the closed pattern
   doctrine — blessed shapes writers must use), `spec/kernel-spec-v0.6.md`
   (the 90-rule language spec), `optimizer-language-research/notes/user-directives.md`
   (binding owner rulings and amendments).

## Verify (before AND after your work)

- `make check` — root gate: spec CI, rule tests, soundness, perf pins,
  codegen parity + corpus, conformance. Must be green.
- `make -C compiler check` — wfc (the production compiler, written in Whitefoot,
  bootstrapped by `prototype/democ`). Must be green.

## Standing rules (owner-ratified; do not relitigate)

- English-only project: all project code and documentation must be written in
  English. More generally, every new or modified repository artifact must be
  written in English. This includes source identifiers that use natural-language
  words, comments, diagnostics, string literals intended for readers, test names,
  fixtures, documentation, reports, plans, prompts, project-authored datasets,
  and file or directory names. Do not add translations or language-suffixed
  variants. Programming-language tokens, formal mathematical notation, numeric
  data, and external proper names are allowed, but all surrounding prose must be
  English. Before finishing a change, scan changed contents and filenames for
  non-English prose.
- `CLAUDE.md` and `AGENTS.md` are repository-level agent instructions and must
  remain byte-identical. Update both in the same change.
- Durability: commit + one `decision-gates.md` line per completed step.
  Sessions get rewound; git log + gates tail are how work resumes.
- Fact channels (anything that lets the optimizer assume more) get hostile
  adversarial review BEFORE shipping. A green gate is not a review.
- Never trade a check for speed at the source level; proof-elision is the
  only path (PATTERNS P8). `move`/checks/rows have exactly one spelling.
- Report results to the owner in plain language; keep project codenames in
  the repo logs, not in chat.
- Subagent tiering: sonnet only for mechanical work, opus for most tasks,
  top tier for subtle soundness reasoning. Never haiku.
- Framing: performance and correctness language only.

## Layout

- `compiler/` — wfc, the self-hosting compiler (Whitefoot sources, Python-run
  stage-0 tests). The active build track.
- `prototype/` — democ (stage-0 compiler) + checker; reference semantics.
- `spec/` — kernel spec, derivation ledger, FR reconciliation.
- `conformance/`, `codegen-corpus/`, `tools/` — the verification stack.
- `experiments/` — active measured evidence (see its README).
- `optimizer-language-research/` — the lab log (`implementation/decision-gates.md`),
  owner directives (`notes/`), design docs + reviews (`implementation/`).
- `mcts_mem/` — the design-decision tree; read the relevant node + its
  `.alt/` before proposing any non-trivial design change; keep it true per
  the mcts-mem discipline (re-decisions move the old form into `.alt/` with
  paired whys, in the same change as the code).
- `archive/` — superseded plans, the research-era record, shelved harnesses.
  Read-only context; nothing in it gates anything.

## Current focus (2026-07-17)

- The D15 systems-performance-coverage capability research is COMPLETE and
  PARKED at the owner's direction (2026-07-17, D19). The design package is
  finished and budget-verified: a three-tier architecture (narrow language
  core + sealed built-in parts + taught checked-source libraries/cards); ten
  sealed parts; the ratified 15-rule loan/freeze judgment plus six
  adversarially-reviewed kernel deltas (TAG-1, tbl_clone, byte-loads, DOM-1,
  BRAND-1, concurrency/CONC-0..4); and the always-loaded manual fits at
  ~46.4k of the 48k token budget. Owner rulings D16-D19 bind (explicit copy
  struct; catalog minimality = 10 sealed kernels; per-kernel acceptance
  ledgers; trap = process abort; generational pools; four v1 non-goals;
  proof-gated admission per D17; the five concurrency-delta decisions). The
  package lives under
  `optimizer-language-research/implementation/systems-performance-coverage/`
  (start at `DESIGN-DOSSIER.md`); deferred items are tracked in its
  `FOLLOW-UPS.md`. NOTHING is authorized for production: the real-compiler
  landing (loan/freeze rules into the prototype checker, then a sealed part
  end-to-end measured on the deploy target), the per-part five-leg acceptance
  batteries, and production spec drafting are a separate owner-gated phase
  that is NOT yet opened. Do not begin landing work without an explicit owner
  decision.
- The superseded B-Strata / candidate capability-research era is archived at
  `archive/research/minimal-systems-capability/` (historical evidence and
  falsifiers; nothing gates it).
- wfc self-hosting build in `compiler/` (SoA-tape architecture per P2) — a
  separate track: front-end complete over its 477-function unit, LLVM codegen
  at ~15/477 functions, still bootstrapped by `prototype/democ`; unaffected by
  the parked capability research.
- D9a is complete on two independently preregistered shipped-library targets.
  First-green `gpt-5.6-terra`/medium Whitefoot beats `percent-encoding` 2.3.2 by
  1.653x [1.631, 1.667] and one-shot `utf8parse` 0.2.2 by 1.098x
  [1.085, 1.145]. Both retain every bounds site, so this is replicated
  default-shape evidence, not proof-elision evidence. The utf8parse facts
  control is statistically inconclusive, but facts-on/off reports and optimized
  instruction bodies are identical. Do not tune either completed protocol
  from its result.
- Proof tier: PROOF-1/2 shipped and adversarially reviewed; the accounting
  design's approved first slice is in
  `optimizer-language-research/implementation/requires-check-accounting-REVIEW.md`.
