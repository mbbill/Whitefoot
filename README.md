# Whitefoot

Whitefoot is a programming language designed as a harness for AI agents.
It serves human-directed systems development with implementation delegated to
agents, using constraints, explicit interfaces, and machine-checked proofs to
guide authors toward safe, efficient programs.
The compiler checks required source evidence and erases it before execution;
writers have no unchecked
escape hatch. The [specification](spec/kernel-spec.md) defines the exact
guarantees and trusted boundary.

## Purpose

The target is a serious research compiler that compiles real programs and
lets us test language and performance ideas. Whitefoot uses restrictions,
interfaces, and writer guidance to make ordinary implementations fall into
efficient, verifiable classes and to expose architectural mistakes early.
The goal is useful default performance, not a guarantee that every accepted
program is globally optimal.

Ease of manual source authorship is not an independent goal. Additional source
and proof effort can be worthwhile when they improve performance or correctness,
while usable feedback and enough information to complete the task still matter.
Changing the intended author opens alternatives; experiments must establish
which mechanisms work under their stated conditions.

The [constitution](docs/constitution.md) owns the objectives and
tradeoffs; [Agent instructions](AGENTS.md) own project priorities and workflow.

## Start here

Read the material that owns the question you are working on:

| Question | Source |
|---|---|
| What does the language admit? | [Active kernel specification](spec/kernel-spec.md) |
| What does this compiler implement, and how do I run it? | [Running the compiler](#running-the-compiler) below; the conformance report states the implemented surface |
| What are the project goals and design principles? | [Constitution](docs/constitution.md) |
| How do I work on a branch and prepare a merge? | [AGENTS.md](AGENTS.md); [CLAUDE.md](CLAUDE.md) is the identical alternate entry |
| Which writer forms should I try? | [Patterns](docs/patterns.md) |
| How should I investigate, verify, and maintain documentation? | [Engineering practice](docs/practice.md) |
| Why was a design chosen? | [Design trees](design/), with reasons and refused alternatives |
| Which long-range directions have been considered? | [Reference roadmap](docs/roadmap.md) |

The roadmap is outside the working loop and may be stale. It is not an
implementation-status page or a work queue. Research and dated essays provide
evidence and ideas; they do not add approval requirements. The reading and
authority rules are in [AGENTS.md](AGENTS.md#authority-and-reading).

## Repository

- [compiler/](compiler/): the Rust compiler, LLVM emission, and native
  runtime support.
- [spec/](spec/): the active language, immutable version archives, and rule
  [selection-ground index](spec/derivation/derivation-ledger.md#current-index).
- [tests/](tests/): normative conformance evidence, recorded-verdict snapshots,
  executable programs, and code-generation evidence.
- [docs/](docs/): principles, writer guidance, engineering practice, and
  reference material.
- [research/](research/README.md): investigations and experiments with their
  designs, measurements, and rejected alternatives.
- [mcts_mem/](mcts_mem/): frozen historical decision record, being moved into
  `design/` and deleted when that is complete.
- [design/](design/): live design decisions with their reasons, and the
  procedure that maintains them.
- [governance/](governance/): archive-protection hooks and specification-change
  design evidence. The old approval ledger is retired.
- [.github/](.github/): CI and the pull-request template.
- [archive/](archive/): frozen historical material. Active source, builds,
  tests, and tools do not depend on it.

## Running the compiler

From `compiler/`:

```sh
cargo run --bin whitefootc -- source.wf -o program
cargo run --bin whitefootc -- --emit-llvm source.wf
cargo run --bin whitefootc -- --par source.wf -o program
```

`whitefootc` accepts an ordered bundle of source files. `--no-overlap` selects
the exact sequential reference lowering and cannot be combined with `--par`.
`--par-ledger` and `--stack-ledger` print their reports; name the LLVM output
with `-o` when a report and emitted LLVM would otherwise share stdout.

## Verification

From the repository root:

```sh
make check
make install-hooks   # optional: catch immutable-spec edits earlier
```

`make check` is the canonical complete gate and prints stage timings. Its
stage inventory is defined in the root [Makefile](Makefile) and
[compiler Makefile](compiler/Makefile). For a shorter development feedback
loop:

```sh
make static
make -C compiler format lint
make -C compiler test-unit
cargo test --manifest-path compiler/Cargo.toml --profile gate --locked --offline --lib semantic::tests::source_proofs
```

Use a test filter matching the responsibility changed; `source_proofs` above
is one example. The `gate` profile keeps debug assertions and overflow checks
while optimizing the compiler's analysis work. The complete gate is still
required on the exact revision merged into main.

The [gate workflow](.github/workflows/gate.yml) runs those stages on Linux and
macOS. Additional [I/O host checks](.github/workflows/io-hosts.yml) and
[benchmarks](.github/workflows/io-bench.yml) own their platform-specific
evidence. A green run describes its tested revision and coverage; it is not a
proof of completeness or the absence of known defects. Conformance reports
distinguish passing cases, expected compiler failures, and pending support.

Specification identity is derived from the active file's bytes by
[compiler/build.rs](compiler/build.rs). The work-branch and specification
amendment rules are stated once in [AGENTS.md](AGENTS.md#branch-and-main-boundary).

## License

Whitefoot is available under the [MIT License](LICENSE).
