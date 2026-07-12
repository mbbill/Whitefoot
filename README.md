# xlang

A systems language for AI-written, human-approved code. The checker makes the
memory-corruption, data-race, silent-overflow, and uninitialized-read bug
classes unrepresentable — no unsafe escape exists — and its proofs feed the
optimizer, so checked code runs at C-class speed: safety checks are always on
unless a machine-verified proof discharges them.

Highlights so far (each with a RESULTS.md under `experiments/`):
- base64: byte-identical to system tools, fastest measured implementation on
  this machine after proof-driven check elision (1.66x from proofs alone),
  with the capacity contract enforced at the boundary — even for C callers.
- wc: byte-identical under LC_ALL=C, ~2x GNU coreutils on default invocation.
- The classifier-kernel study: i1-dataflow parity with C and safe Rust.
- Checked algebraic laws: 3.3x on reductions, with FALSE laws refuted at
  compile time — the transform Rust must take on faith.

## Where things are

| I want to... | Go to |
|---|---|
| Understand the project state | `THE-PLAN.md`, then the tail of `optimizer-language-research/implementation/decision-gates.md` |
| Work in this repo as an agent | `CLAUDE.md` |
| Read the law / the doctrine | `CONSTITUTION.md` / `PATTERNS.md` |
| Read the language spec | `spec/kernel-spec-v0.6.md` (+ `spec/derivation-ledger.md` for why each rule exists) |
| The production compiler (xlc, self-hosting) | `compiler/` |
| The stage-0 compiler + checker | `prototype/` |
| Run all verification | `make check` and `make -C compiler check` |
| Measured evidence | `experiments/` (index in its README) |
| Owner rulings | `optimizer-language-research/notes/user-directives.md` |
| Superseded history | `archive/` |

## Verification

Two gates, both required green: `make check` (spec CI, rule tests, soundness
probes, performance pins, codegen parity corpus, conformance suite) and
`make -C compiler check` (the xlc test stack, including the self-parse gate).
Every completed unit of work commits with a one-line entry in the decision
log — the repo is designed so that any fresh session can resume from
`git log` plus the log tail alone.
