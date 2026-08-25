# Whitefoot

Whitefoot is a systems language for AI-written, human-approved code. It is
designed so that memory corruption, data races, uninitialized reads, and silent
overflow are unrepresentable in accepted source. There is no writer-accessible
unsafe escape. Every partial operation is admitted only after machine proof of
its domain; a written claim is the sole writer-reachable runtime trap and is
never removed. A claim is only an independently true theorem that the
normative checker cannot derive and a later admission root genuinely needs;
it is never an assertion, test oracle, intentional abort, or substitute for
ordinary control flow, and its `because` record states the complete derivation.

## Project goal

The target is a serious research compiler: general enough to implement the
real language, clean enough to evolve, and capable of compiling nontrivial
programs so we can test semantics and performance ideas quickly. It is not an
untrusted-input service or a stable LLVM-scale product.

This is more than a demo compiler: language behavior must come from general
rules, correctness tests stay compiler-independent where useful, and the
compiler must eventually emit and run real programs. Product-scale resource
controls, stable artifact protocols, distribution, and release engineering are
not current goals.

[docs/roadmap.md](docs/roadmap.md) is the living Direction Outline: the current
map of capabilities, open directions, evidence, and candidate projects.
[docs/current-plan.md](docs/current-plan.md) records the latest high-level plan;
neither document grants or withholds permission to work on a branch.
[docs/WORKFLOW.md](docs/WORKFLOW.md) defines the complete four-rule branch and
`main` boundary. [AGENTS.md](AGENTS.md) records the project's technical
priorities and repository discipline.

## Current state

Kernel specification v0.37 at the stable
[specification path](spec/kernel-spec.md), SHA-256
`f772f2aec5e0da963c1cb9d8607a9e87cd3ad03cb71f3b6532451404d4d07bb5`,
is active in this exact revision. It supersedes v0.36, whose exact bytes at
SHA-256 `fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`
are preserved as an immutable flat archive. `main` remains on v0.36 until this
exact revision is approved and merged.

The safe-Rust compiler currently implements one ordinary path:

```text
ordered source bundle
  -> lossless lexer
  -> context-free terminal classification
  -> iterative strong-LL(2) parsing
  -> one finalized source-bound syntax tree
  -> exact FORM-2 source validation
  -> CanonicalSyntaxUnit
  -> direct lexical name resolution
  -> ResolvedSyntaxUnit
  -> semantic and ownership checking
  -> private checked program
  -> target-independent typed control-flow IR
  -> selected-host layout and target qualification
  -> conservative LLVM
  -> embedded completion/parallel runtime units when the module names them
  -> host executable
```

The detailed implemented surface is maintained in the
[compiler README](compiler/README.md); the Direction Outline summarizes it only
at the level needed to choose projects and research. Valid language that a
growing compiler does not yet implement stops as an explicit unsupported
capability; it is not reported as invalid Whitefoot.

## Repository layout

The top level is a small, curated set. Each entry has one clear purpose; scripts
live next to what they check.

| Directory | What it is |
|---|---|
| [docs/](docs/) | The living [Direction Outline](docs/roadmap.md), rolling [Current Plan](docs/current-plan.md), project law ([constitution](docs/constitution.md)), seeded writer forms ([patterns](docs/patterns.md)), supporting direction notes ([ideas](docs/ideas.md)), and dated design synthesis ([why-whitefoot](docs/why-whitefoot.md)) |
| [spec/](spec/) | The language: one stable active kernel specification, immutable flat version archives, and the rule-derivation ledger under `spec/derivation/` |
| [compiler/](compiler/README.md) | The safe-Rust compiler: frontend, resolver, first semantic/IR slice, LLVM backend, and `whitefootc` |
| [tests/](tests/) | Test evidence: the active compiler-independent `conformance/` behavior corpus, plus preserved `codegen/` source cases awaiting production-compiler integration |
| [governance/](governance/) | The protected approval ledger, specification-evolution evidence, and the tracked archive-protection hooks |
| [research/](research/) | Active language and compiler experiments |
| [mcts_mem/](mcts_mem/) | The live design tree, consulted and maintained only through the `mcts-mem-use` skill |
| [archive/](archive/) | Retired and superseded material, including the historical [decision log](archive/governance/decision-log.md), Python reference model, and democ-era codegen harness; inert — no active source, build, test, or tool depends on it. Its live disposition map is the [archive promotion audit](research/archive-promotion-audit.md) |

## Verification

```sh
make install-hooks   # once: enable immutable-archive pre-commit protection
make check           # compiler, conformance, and specification identity gate
```

The gate checks repository invariants and the specification digest chain,
builds and tests the compiler, validates conformance structure and coverage,
runs the strict native completion harness, runs the maintained research
oracles, and drives every non-pending conformance case through the ordinary
compiler/link/execute path. The current native tally is `Pass=500 Skip=1`.
A green result states only what these gates exercise and is not a completeness
claim.

## License

Whitefoot is available under the [MIT License](LICENSE).
