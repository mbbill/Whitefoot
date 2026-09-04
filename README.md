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
core runs only specification-fixed, deterministic, terminating derivations.
Each admitted rule family is run to its specified completion; there is no
timeout, cumulative proof-work budget, solver seed, or heuristic stopping
condition that can turn the same source into a different verdict. When those
rules are not enough, the author writes finite proof steps in the same source
file. AI or an offline tool may search while writing them, but compilation only
checks the written steps.

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

Kernel specification v0.43 is the active language authority, SHA-256
`037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`, carried by
the stable [specification path](spec/kernel-spec.md). It carries two
amendments: every loop body is now a region block, so a borrow written in the
body takes that body's own region bare and dies with the iteration; and
[ENT-6]'s control-flow join is repaired to be associative, so nested joins
reach the image one flat join over the same branches reaches. It supersedes
v0.42, whose outgoing bytes are preserved byte-for-byte at
[`spec/kernel-spec-v0.42.md`](spec/kernel-spec-v0.42.md). The merge-time record
for that activation is in
[governance/APPROVALS.md](governance/APPROVALS.md), which becomes effective
with the owner's merge approval of the exact revision containing it.

v0.40 checks `requires`, `ensures`, loop-header `invariant` relations,
and local `invariant` statements in the ordinary semantic compiler. A local
invariant may carry an explicit `use` block when the fixed automatic rules are
insufficient. It accepts a supported partial operation only when the current
proof context establishes that operation's exact domain, and it proves
selected-target layout and address arithmetic before emitting the operation.
The checked proof syntax and diagnostic derivations are erased before runtime
lowering. Calls, the optimizer, and `par` consume only verified semantic
consequences; no proof object or checker bookkeeping enters runtime IR. There
is no writer-accessible runtime assertion or hidden fallback check.
This implementation cycle does not introduce a `.wfproof` artifact,
cross-module proof cache, incremental-proof protocol, or compiler
self-verification layer. Those are possible future build concerns, not part of
making source proof correct now.

The automatic affine boundary is part of the language, not an implementation
guess. For each goal, AUTO checks the zero-premise direct route, every available
coefficient-one single premise, every unordered coefficient-one premise pair
including a premise paired with itself, and the final fixed L0-image route.
Those finite families are exhausted in specification order when the goal is
not proved. A relation that needs three or more published affine premises
outside the final fixed L0-image route, a special elimination route, or a
future named nonlinear rule must carry explicit `use` steps. A
nonempty `use` block is rejected as redundant when AUTO already proves its
target under the same specification version.

The canonical loop surface makes induction visible at the loop header:

```wf
for (
  i in 0_u64..count,
  invariant per_byte: sum <= 255_u32 * i
) {
  let w = deref(weights)[i];
  let wide = cvt::<u8, u32>(w);
  set sum = sum + wide;
}
```

The first `for` header item is the binding and every later item is an
`invariant`; the final item has no trailing comma. `loop` uses the same optional
parenthesized invariant list but has no binding item. Header invariants cannot
have `use` blocks, and their names exist only in the loop body. Local
invariants are checked once at their program point; every `use` is proved from
the same entering snapshot, only the outer conclusion is published, factor one
is omitted, and repeating the same normalized premise is invalid. In this
example AUTO subtracts the one published affine premise `per_byte`; DIRECT then
proves the residual from the `u8` type interval of `wide`. Adding a `use` block
would therefore be redundant and invalid.

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
push: the Linux io_uring adapter and sanitizers on a real kernel, plus the
native Windows compiler bootstrap, direct and bounded blocking adapters, IOCP
correctness and capacity recovery, mandatory `--par` compute pool, and zero
eligible-fallback checks;
[`.github/workflows/io-bench.yml`](.github/workflows/io-bench.yml) carries the
program-level I/O benches on real hardware, including the read-dominated
tables on both a Linux and a macOS runner, and the fixed-host paired Windows
compute, IOCP, and mixed qualification. The macOS runner is the only macOS
host available to this project without an endpoint-security stack in its I/O
path.

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
