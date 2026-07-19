# Whitefoot — agent onboarding

Whitefoot is a systems language for AI-written, human-approved code. Entire bug
classes are unrepresentable (memory corruption, races, silent overflow,
uninitialized reads); there is no unsafe escape. The checker's proofs double
as optimizer facts: safety checks are always on unless a machine-verified
proof discharges them — speed is earned by proof, never by weakening a check.

## Read order (do this before working)

1. `THE-PLAN.md` — the sole roadmap: current state, execution order, and gates.
2. Tail of `optimizer-language-research/implementation/decision-gates.md` —
   the append-only lab log; the last ~10 entries are the live context.
3. `mcts_mem/` — the design-decision tree: before proposing any non-trivial
   design change, read the relevant node and its `.alt/` (what was already
   tried and why it lost).
4. As needed: `CONSTITUTION.md` (law), `PATTERNS.md` (the closed pattern
   doctrine — blessed shapes writers must use), `spec/kernel-spec-v0.8.md`
   (the 91-rule language spec), `optimizer-language-research/notes/user-directives.md`
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
- Kernel-spec changes are owner-gated, in advance. Before modifying any numbered
  kernel specification in any way, present the exact proposed delta to the owner
  and get explicit approval; approval of a plan, phase, or checker task is never
  approval to edit the spec. Every approved change bumps the specification
  version and updates the filename, title, and all live references in the same
  change — never revise a numbered specification in place. Record the owner's
  approval in `governance/APPROVALS.md`; that logged approval is the
  authorization to commit.
- Earn a spec change with evidence, never convenience. The accepted specification
  is the authority; the current wfc source is not — it is early code that may be
  wrong. A conflict between wfc and the spec is an investigation, not a license to
  relax the rule: record the alternatives (fix the code to fit the rule vs. change
  the rule), their pros and cons, a soundness argument for any proposed relaxation,
  and measured data (checker feasibility, code shape, performance) before bringing
  the owner-gated change. Never relax a rule on the ground that existing code
  violates it; a spec change must be earned by a soundness or design argument that
  stands on its own, independent of what the current code happens to do.
- The semantics-bearing test surface is owner-gated the same way: conformance
  expected verdicts (`conformance/manifest.jsonl` + `conformance/cases/**`),
  frozen oracle digests, and the reference semantics tests
  (`prototype/checker/test_checker.py`, `prototype/democ/test_codegen.py`). Add
  new tests or conformance cases freely, but modifying, deleting, or weakening an
  existing one — or regenerating a pinned oracle digest — needs the owner's
  logged approval. Never make a failing check pass by changing what it expects
  (W3). After approval, run `make approve-spec REASON="..."`.
- `make check`'s `spec-guard` layer enforces both rules: a guarded change with no
  matching approval in `governance/APPROVALS.md` fails the build. A red
  spec-guard means stop and get the owner's approval — not regenerate the
  baseline to go green.
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

## Current authority and authorization

`THE-PLAN.md` is the sole source for current status, execution order, phase
gates, and next work. Owner directives and the decision log preserve rulings
and evidence. Design dossiers and archived plans do not authorize work.

The owner has authorized phases 1 through 7 in `THE-PLAN.md`, in order, through
the complete acceptance ledger for `seq`. Each phase keeps its written gates
and stop conditions. Concurrency and later catalog work remain outside this
authorization and require a new owner directive.
