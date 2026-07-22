# Whitefoot

Whitefoot is a systems language for AI-written, human-approved code. It is
designed so that memory corruption, data races, uninitialized reads, and silent
overflow are unrepresentable in accepted source. There is no writer-accessible
unsafe escape. Runtime safety checks remain enabled unless a machine-verified
proof authorizes their removal.

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

[docs/roadmap.md](docs/roadmap.md) is the sole source for current execution
order and authorization. [WORKFLOW.md](WORKFLOW.md) defines the complete
cross-directory language-change process. [AGENTS.md](AGENTS.md) records the
priority rule and structure discipline future agents must apply.

## Current state

[Kernel specification v0.11](spec/kernel-spec-v0.11.md), SHA-256
`050e110c8c5eb3143c9d3f54968a9df9125f1d4b5991f527b8a15938a4292fbc`,
is the immutable active specification. Exact v0.8 through v0.10 remain
immutable history.

The safe-Rust compiler currently implements one ordinary path:

```text
ordered source bundle
  -> lossless lexer
  -> context-free terminal classification
  -> iterative strong-LL(2) parsing
  -> one finalized source-bound syntax tree
  -> exact FORM-2 source validation
  -> CanonicalSyntaxUnit
  -> direct v0.11 lexical name resolution
  -> ResolvedSyntaxUnit
  -> scalar semantic checking
  -> private checked program
  -> target-independent scalar IR
  -> conservative LLVM
  -> host executable
```

The executable slice currently covers scalar integer/unit values, `Bool`
construction and checks, integer and unit constants, nongeneric own-mode
functions, locals, direct named calls, returns, pure/traps effects, integer
wrap/trap arithmetic, and comparisons. Other valid v0.11 families stop as
explicit unsupported compiler capabilities; they are not reported as invalid
Whitefoot. The next work expands one coherent semantic family end to end.

## Repository layout

The top level is a small, curated set. Each entry has one clear purpose; scripts
live next to what they check.

| Directory | What it is |
|---|---|
| [docs/](docs/) | The plan of record ([roadmap](docs/roadmap.md)), project law ([constitution](docs/constitution.md)), writer forms ([patterns](docs/patterns.md)), and the design rationale ([why-whitefoot](docs/why-whitefoot.md)) |
| [spec/](spec/) | The language: numbered kernel specifications (append-only) and the rule-derivation ledger under `spec/derivation/` |
| [compiler/](compiler/README.md) | The safe-Rust compiler: frontend, resolver, first semantic/IR slice, LLVM backend, and `whitefootc` |
| [tests/](tests/) | Correctness evidence: `conformance/` behavior corpus, `reference/` semantics oracle, `codegen/` optimization-proof corpus (dormant until optimizer work) |
| [governance/](governance/) | The protected approval ledger, exact successor candidates, and the tracked spec-append-only hook |
| [research/](research/) | Active language and compiler experiments |
| [mcts_mem/](mcts_mem/) | The live design tree, consulted and maintained only through the `mcts-mem-use` skill |
| [archive/](archive/) | Retired and superseded material, including the historical [decision log](archive/governance/decision-log.md); inert — no active source, build, test, or tool depends on it |

## Verification

```sh
make install-hooks   # once: enable the spec append-only pre-commit hook
make check           # the gate: compiler, conformance, reference, spec append-only
```

The gate is deliberately small — the compiler builds and passes its tests, the
behavior corpus and reference model agree, and the numbered spec stays
append-only. Everything else (keeping conformance, tests, and other spec-derived
material consistent with the newest spec) is guarded by the guidance in
`AGENTS.md`, not by machinery. A green result states only what it exercises; it
does not claim the language or compiler is complete.

## License

Whitefoot is available under the [MIT License](LICENSE).
