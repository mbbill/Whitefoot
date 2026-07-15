# xlang — agent onboarding

xlang is a systems language for AI-written, human-approved code. Entire bug
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
- `make -C compiler check` — xlc (the production compiler, written in xlang,
  bootstrapped by `prototype/democ`). Must be green.

## Standing rules (owner-ratified; do not relitigate)

- English-only repository: every new or modified repository artifact must be
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

- `compiler/` — xlc, the self-hosting compiler (xlang sources, Python-run
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

## Current focus (2026-07-15)

- D14 privileged-basis research is the active design track; OD-0 through OD-5
  are not the next action. The gate-authentication layer now has a hostile-
  reviewed conditional result: use stateless signed grants C if an existing
  external-frame template must accept new approved instances without an
  authorization-release update, otherwise use fixed release entries F. Stateful
  snapshots S add only presently unrequired local extension-grant currentness.
  This is not an owner selection. Next freeze the bounded no-formula frame-
  template registry and exact minimal privilege-cut ledger; keep template
  coverage, dense D-2's 340 unresolved obligations across 150 contexts, and
  exact P-1 fail-closed. No language or specification change, compiler or
  verifier implementation, capability entry, standard library, container,
  candidate execution, benchmark, E0.1 restart, migration, or default teaching
  is authorized.
- xlc self-hosting build in `compiler/` (SoA-tape architecture per P2).
- D9a is complete on two independently preregistered shipped-library targets.
  First-green `gpt-5.6-terra`/medium xlang beats `percent-encoding` 2.3.2 by
  1.653x [1.631, 1.667] and one-shot `utf8parse` 0.2.2 by 1.098x
  [1.085, 1.145]. Both retain every bounds site, so this is replicated
  default-shape evidence, not proof-elision evidence. The utf8parse facts
  control is statistically inconclusive, but facts-on/off reports and optimized
  instruction bodies are identical. Do not tune either completed protocol
  from its result.
- Proof tier: PROOF-1/2 shipped and adversarially reviewed; the accounting
  design's approved first slice is in
  `optimizer-language-research/implementation/requires-check-accounting-REVIEW.md`.
