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
| How do I work on a branch and prepare a merge? | [AGENTS.md](AGENTS.md) |
| Which writer forms should I try? | [Patterns](docs/patterns.md) |
| How should I investigate, verify, and maintain documentation? | [Engineering practice](docs/practice.md) |
| Why was a design chosen? | [Design trees](design/), with reasons and refused alternatives |
| Which research questions and experiments could be useful? | [Ideas](docs/ideas.md) |
| What defects and follow-up work remain? | [Todo](docs/todo.md) |

Research and dated essays provide evidence and ideas; they do not add approval
requirements. The reading and authority rules are in
[AGENTS.md](AGENTS.md#authority-and-reading).

## Repository

- [compiler/](compiler/): the Rust compiler, LLVM emission, and native
  runtime support.
- [lib/containers/](lib/containers/README.md): reusable Whitefoot container
  source, exercised by callers in the ordinary program corpus.
- [spec/](spec/): the active language and its immutable version archives.
- [tests/](tests/): normative conformance evidence, executable programs,
  code-generation evidence, and the separate performance regression suite.
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

Prerequisites: a Rust stable toolchain at least the version in
[compiler/Cargo.toml](compiler/Cargo.toml)'s `rust-version` (`rustup update
stable` on an older installed stable — rustup does not update it on its own),
and clang available at `/usr/bin/clang` on Linux/macOS or as `clang` on PATH
on Windows.

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

`--par` takes three grain controls. `--par-scalar-leaf-limit N|off` moves or
removes the default threshold that keeps scalar leaves of at most 16
nonconstant operations out of compute offers; `--par-sequential-refusal` runs
a refused offer's callee in its sequential clone; `--par-recursive-frontier
auto|N|off` sets the starting budget of a recursive component's clone family,
where `auto`, the default, asks the runtime, `N` from 1 to 32 pins it at
compile time, and `off` emits no family so every node offers. `whitefootc
--help` prints the full usage. At run time `WF_WORKERS` selects compute
participation; `WF_STACKS` is inert.

## Verification

The guarded wrapper `.github/run-check.pl`, used below and throughout this
section, additionally needs `/usr/bin/time` (Debian/Ubuntu package `time`).

From the repository root:

```sh
make check
make install-hooks   # optional: catch immutable-spec edits earlier
```

`make check` is the canonical complete gate and prints group and phase timings.
The root [Makefile](Makefile) owns its group inventory and recipes; ordinary CI
reads that same inventory with `make check-groups` and invokes the same
`make check-group GROUP=<name>` entry. For a shorter development feedback loop:

```sh
make static
make -C compiler format lint
make -C compiler build        # optimized compiler only
make -C compiler test-build   # construct test executables without running cases
perl .github/run-check.pl source-proofs cargo test --manifest-path compiler/Cargo.toml --profile gate --locked --offline --lib semantic::tests::source_proofs
```

Use a test filter matching the responsibility changed; `source_proofs` above
is one example. The `gate` profile builds the Rust compiler implementation and
test harnesses with optimization, debug assertions and overflow checks. It is
not an optimization switch for WF source. Use a dev build when debugging the
Rust implementation, rather than constructing it for ordinary verification.
Formatting and API documentation have explicit `format` and `docs` commands;
they are not extra correctness-test stages. The complete gate is still
required on the exact revision merged into main.

The root gate, research/benchmark checks and compiler verification targets use
one host-wide owner across worktrees, with two Cargo jobs and two test threads
by default. Wrap other heavy commands as in the filtered example above. The
wrapper prints wall/user/system time and a heartbeat every 30 seconds; a
competing invocation reports the owner and exits. Its 30-minute command limit
terminates the owned process group, including nested commands. Set
`WHITEFOOT_CHECK_TIMEOUT` in seconds for an intentionally longer protocol.
Explicit job/thread settings remain available. After an uncatchable stop,
inspect the recorded PID and command before removing a stale lock.

For a slow compiler test, set `WHITEFOOT_TEST_TIMINGS` to a scratch TSV path.
The shared semantic/backend/program helpers record test name and phase:
Whitefoot compilation, native construction, native execution and semantic
assertions. This is diagnostic coverage of those helpers, not every custom
subprocess. Nested or parallel rows are not additive suite wall time. See the
[measured build/test investigation](research/investigations/test-economy/build-and-test.md).

The [gate workflow](.github/workflows/gate.yml) runs those groups on Linux and
macOS. Additional [I/O host checks](.github/workflows/io-hosts.yml) and
[benchmarks](.github/workflows/io-bench.yml) own their platform-specific
evidence. Automatic CI checks correctness and performance regressions under
the [test boundary](docs/practice.md#test-boundary): useful research cases and
their dependencies belong in formal tests, while research runs on explicit
request. Full IO matrices and compute scoreboards are experiments; the separate
[compute regression check](.github/workflows/compute-regression.yml) supplies
a paired performance verdict using the [formal runner](tests/performance/README.md).
Routine correctness CI and local `make check`
do not build a baseline compiler or run that comparison. A green run describes its tested revision and
coverage; it is not a proof of completeness or the absence of known defects.
Conformance reports distinguish passing cases, expected compiler failures,
and pending support.

Specification identity is derived from the active file's bytes by
[compiler/build.rs](compiler/build.rs). The work-branch and specification
amendment rules are stated once in [AGENTS.md](AGENTS.md#branch-and-main-boundary).

## License

Whitefoot is available under the [MIT License](LICENSE).
