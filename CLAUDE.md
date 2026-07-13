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

## Current focus (2026-07-12)

- xlc self-hosting build in `compiler/` (SoA-tape architecture per P2).
- D9a first target is complete: one exact `gpt-5.6-terra`/medium trajectory's
  first-green `percent_decode` beats shipped `percent-encoding` 2.3.2 by
  1.653x [1.631, 1.667]. Facts-on/off is practical parity with all six checks
  retained, so this is default-shape evidence, not proof-elision evidence.
  The independent second target is now preregistered against shipped
  `utf8parse` 0.2.2: one-shot `Parser`/`Receiver` events, an 84,041-case exact
  and surplus-capacity gate, and a fixed 128 MiB score. It has not yet been
  launched. Do not tune either completed/frozen protocol from model output.
- Proof tier: PROOF-1/2 shipped and adversarially reviewed; the accounting
  design's approved first slice is in
  `optimizer-language-research/implementation/requires-check-accounting-REVIEW.md`.
