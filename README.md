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
[docs/current-plan.md](docs/current-plan.md) is the single rolling execution
proposal or approved plan derived from that outline. Execution requires either
its `ACTIVE` status or an explicit owner branch direction recorded in the
corresponding batch; candidate merge and activation still require the owner
boundary named there. [docs/WORKFLOW.md](docs/WORKFLOW.md)
defines the project-driven process, parallel research lane, and guarded
language-change branch. [AGENTS.md](AGENTS.md) records the priority rule and
structure discipline future agents must apply.

## Current state

On `main`, Kernel specification v0.33, SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`,
remains active. This work branch carries the v0.34 `CANDIDATE` at the stable
[specification path](spec/kernel-spec.md), SHA-256
`b9284417e12f41a9c4c78728bf684c088aadd2d1fcfd10d305c0ef24c448d27a`;
it has no language authority until the owner-approved atomic merge. Exact v0.8
through v0.32 remain immutable flat archives.

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
  -> conservative LLVM
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

The gate is deliberately small: the compiler builds and passes its tests; the
conformance corpus has valid active-spec identity, structure, rule coverage,
and expectations; and the stable file plus immutable archives match the
recorded digest chain. The native compile-run adapter is invoked separately by
`make conformance-run`; its current result is Pass=432, Fail=1,
Skip=13 and is not silently counted as part of `make check`. A green result
states only what the selected gate exercises and is not a completeness claim.

## License

Whitefoot is available under the [MIT License](LICENSE).
