# Whitefoot

Whitefoot is a proof-carrying systems language for AI-written, human-approved
code. Here, proof-carrying means that source and its canonical artifacts carry
the machine-checkable facts and explicit evidence needed to justify every
partial operation and every performance fact the optimizer is allowed to
trust. The normative checker, not the writer, an AI, or an external prover,
decides whether that evidence is sufficient.

The extra evidence is intended to buy safety and speed from the same
mechanism. The target is that an accepted program may contain a logic error,
but cannot execute memory corruption, a data race, an uninitialized read,
silent overflow, or another unproved partial operation. The same checked
ownership, aliasing, effect, bounds, and algebraic facts can remove runtime
checks, authorize optimizations, and prove that parallel tasks are independent
without `unsafe`, speculation, or later rediscovery. Evidence is erased before
execution and creates no runtime branch, lock, dependency, or scheduling edge.

The official compiler does not use SMT to decide acceptance. Its automatic
core runs only specification-fixed, deterministic derivations with a
syntactically computable work bound. Harder reasoning is carried as an explicit
finite certificate: producing that certificate may require AI, an external
tool, or substantial search, but the compiler only checks the committed steps.
No solver seed, heuristic order, timeout, machine load, or success in
rediscovering a proof may change whether the same complete input is accepted.

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

Kernel specification v0.39 remains the active released language authority,
SHA-256
`b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516`, carried by
its immutable archive. It narrows [CLM-1]'s claim-authority control dependence
to the definitions a boundary selector actually chooses, and supersedes v0.38,
whose outgoing bytes are
preserved as [`spec/kernel-spec-v0.38.md`](spec/kernel-spec-v0.38.md). The
merge-time record for that activation is in
[governance/APPROVALS.md](governance/APPROVALS.md), which becomes effective
with the owner's merge approval of the exact revision containing it.

This work branch carries candidate v0.40 at the stable
[specification path](spec/kernel-spec.md), SHA-256
`3c58f67b436c0a066e4c2ea8b523ffbb3e297268fd2cbb158fa50848f899bacc`.
It is the first implementation slice of the proof-carrying transition: direct
`set` operations publish fixed value images, and entailment closes before a
local scope is forgotten. It does not yet remove `claim` or implement the
explicit certificate language, cross-function proof summaries, or the planned
parallel proof composition. Candidate status grants no merge authority.

v0.39 uses ordinary opaque values, `own`, `move`, `&`, and
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
pass count. Canonical `make check` deliberately rejects `CANDIDATE` status at
the archive-identity step: a merge revision must carry the exact ACTIVE
identity and the outgoing archive. A work branch drafting the next version,
including this revision, uses `make spec-candidate-integrity` instead. A green
result states only what the selected gate exercises and is not a completeness
claim.

## License

Whitefoot is available under the [MIT License](LICENSE).
