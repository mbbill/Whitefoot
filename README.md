# Whitefoot

Whitefoot is a proof-carrying systems language for AI-written, human-approved
code. Here, proof-carrying means that the Whitefoot source itself carries the
machine-checkable statements and proof steps needed to justify every partial
operation and every performance fact the optimizer is allowed to trust. The
compiler checks that source directly. Its syntax tree, fact state, and checked
program are ordinary compiler data; an inconsistency among them is a compiler
defect to repair in code and tests.

The extra evidence is intended to buy safety and speed from the same
mechanism. The target is that an accepted program may contain a logic error,
but cannot execute memory corruption, a data race, an uninitialized read,
silent overflow, or another unproved partial operation. The same checked
ownership, aliasing, effect, bounds, and algebraic facts can remove runtime
checks, authorize optimizations, and prove the ownership, effects, and
independence facts required by `par`, without `unsafe`, speculation, or later
rediscovery. Evidence is erased before execution and creates no runtime branch,
lock, dependency, or scheduling edge.

The official compiler does not use SMT to decide acceptance. Its automatic
core runs only specification-fixed, deterministic, terminating derivations
with a syntactically computable work bound. When those rules are not enough,
the author writes additional finite proof steps in the same source file. AI or an
offline tool may search while writing those steps, but compilation only checks
what the source says. No solver seed, heuristic order, timeout, machine load,
or success in rediscovering a proof may change whether the same complete input
is accepted.

The price is authoring difficulty: programs are more explicit, valid programs
may need proof structure, and safe code without sufficient evidence is
rejected. Whitefoot deliberately spends human ergonomics because its intended
writer is AI. AI may search for programs and proofs and repair checker
failures, but it is never trusted; humans approve the requirements and
resulting changes, and the checker decides what has actually been proved.

## Project goal

The target is a serious research compiler: general enough to implement the
real language, clean enough to evolve, and capable of compiling nontrivial
programs so we can test semantics and performance ideas quickly. It is not an
untrusted-input service or a stable LLVM-scale product.

This is more than a demo compiler: language behavior must come from general
rules, correctness tests stay compiler-independent where useful, and the
compiler must eventually emit and run real programs. Product-scale resource
controls, stable binary-distribution interfaces, and release engineering are
not current goals.

[docs/roadmap.md](docs/roadmap.md) is the living Direction Outline: the current
map of capabilities, open directions, evidence, and candidate projects.
[docs/current-plan.md](docs/current-plan.md) records the latest high-level plan;
neither document grants or withholds permission to work on a branch.
[docs/WORKFLOW.md](docs/WORKFLOW.md) defines the complete four-rule branch and
`main` boundary. [AGENTS.md](AGENTS.md) records the project's technical
priorities and repository discipline.

## Current state

Kernel specification v0.40 is the active language authority, SHA-256
`5079ef2efa7862184f06ccf7dc273ae97eda791679a44f66c86e75afbc46c6e0`; its exact
activation identity is recorded in
[governance/APPROVALS.md](governance/APPROVALS.md). The outgoing v0.39 bytes are
preserved byte-for-byte at
[`spec/kernel-spec-v0.39.md`](spec/kernel-spec-v0.39.md). The source-proof
implementation is complete and activated on this work branch; its conditional
merge-time record becomes effective when the owner approves the exact revision
containing it for merge into `main`.

The work branch checks `requires`, `ensures`, counted-loop `invariant`
statements, and explicit `prove`/`use` steps in the ordinary semantic compiler.
It accepts a supported partial operation only when the current proof context
establishes that operation's exact domain, and it proves selected-target layout
and address arithmetic before emitting the operation. The checked proof
syntax and diagnostic derivations are erased before runtime lowering. Calls,
the optimizer, and `par` consume only the verified semantic consequences fixed
for them; no proof object or checker bookkeeping enters runtime IR. There is no
writer-accessible runtime assertion or hidden fallback check.

`par` consumes this same checked context together with ownership, effect,
iteration-index, layout, target-domain, and bounded queue/completion facts.
Proof checking adds no runtime dependency or scheduling edge. External resource
availability, such as heap exhaustion, stack exhaustion, operating-system quota,
or runtime-start failure, is the only boundary temporarily outside this
implementation cycle; its final source-language failure model remains open.
That scope choice changes neither the project direction nor the required
layout, address, target, parallel-independence, and bounded-completion proofs.

v0.40 retains v0.39's ordinary opaque values, `own`, `move`, `&`, and
`&uniq` for every I/O resource. `reads` and `writes` name formal parameters or
their static struct fields rather than lifetimes. Resource types do not form a
separate language capability category. There is no separate `world`,
`capability-root`, `family-fragment`, or `Ordered` permission system. Completion is
an internal lowering and target contract beneath ordinary calls. File opening
uses ordinary one-shot `FilePermit` owners from an explicit `FileFactory`;
directory selectors remain shared and permit proof data is erased before the
native open ABI.

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
  -> proof-selected compute and completion lowering
  -> conservative LLVM
  -> host executable
```

The detailed implemented surface is maintained in the
[compiler README](compiler/README.md); the Direction Outline summarizes it only
at the level needed to choose projects and research. Valid language that a
growing compiler does not yet implement stops as an explicit unsupported
compiler feature; it is not reported as invalid Whitefoot.

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
| [.github/](.github/workflows/) | Continuous integration on hosts this project does not own: the canonical `make check` and the completion-I/O evidence jobs that need a real Linux kernel or Windows |
| [archive/](archive/) | Retired and superseded material, including the historical [decision log](archive/governance/decision-log.md), Python reference model, and democ-era codegen harness; inert — no active source, build, test, or tool depends on it. Its live disposition map is the [archive promotion audit](research/archive-promotion-audit.md) |

## Verification

```sh
make install-hooks   # once: enable immutable-archive pre-commit protection
make check           # compiler, conformance, and specification identity gate
```

`make check` ends with the wall time of each of its stages, so a gate that
grew names the stage that grew it. The same stages run on every push through
[`.github/workflows/gate.yml`](.github/workflows/gate.yml), on a GitHub-hosted
Linux runner and a GitHub-hosted macOS runner: one job per stage rather than
one job per host, so the wait is the slowest stage instead of the sum, and each
job is capped at eight minutes.
[`.github/workflows/io-hosts.yml`](.github/workflows/io-hosts.yml) carries the
completion-I/O correctness evidence that no machine here can produce on every
push: the Linux io_uring adapter and sanitizers on a real kernel and the Windows
IOCP probes executed natively;
[`.github/workflows/io-bench.yml`](.github/workflows/io-bench.yml) carries the
program-level I/O benches on real
hardware -- including the read-dominated tables on both a Linux and a macOS
runner, the latter being the only macOS host available to this project without
an endpoint-security stack in its I/O path -- and the Windows IOCP probes
executed rather than only cross-linked.

The gate builds and tests the compiler, exercises the native completion
harness, validates conformance structure and rule coverage, runs every
non-pending conformance case through the native compile-run adapter, checks the
maintained research fixtures, and verifies the specification/archive identity
chain. Gate results are revision-specific, so this overview carries no floating
pass count. Canonical `make check` requires the exact ACTIVE identity and the
outgoing archive. A work branch drafting a later version can use
`make spec-candidate-integrity` before its own activation. A green result states
only what the selected gate exercises and does not establish completeness.

## License

Whitefoot is available under the [MIT License](LICENSE).
