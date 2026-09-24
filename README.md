# Whitefoot

Whitefoot is a research systems programming language. A program the compiler
accepts cannot reach undefined behavior, a panic, or a silent integer overflow
at run time, provided the trusted base listed below is correct.

It gets there without runtime checks. Every indexing, arithmetic, conversion,
division and allocation-size operation needs a proof that it is in range. The
compiler finds most proofs itself with a fixed procedure and no SMT solver.
When it cannot, you add a branch or state one more fact, and the compiler
checks that fact too.

```
fn drop_spaces(out: &[u8], src: &[u8]) -> count: u64 reads(src), writes(out) contract {
  requires deref(out).len >= deref(src).len;
} {
  let kept = 0_u64;
  for (
    i in 0_u64..deref(src).len,
    invariant behind: kept <= i
  ) {
    let byte = deref(src)[i];
    if byte != 32_u8 {
      set deref(out)[kept] = byte;
      set kept = kept + 1_u64;
    }
  }
  return kept;
}
```

Without the `invariant` line, the program is rejected:

```text
drop_spaces.wf:8:21: error[OP-4]: UndischargedBoundsObligation
  ...
  source:       set deref(out)[kept] = byte;
  marker:                     ^^^^^^
  residual: kept < deref(out).len
  mechanical_fix: when the relation must hold, establish the residual with a verified requirement, a source invariant, ...
```

With it, the loop has no bounds check. `out.len` arrives in `%rsi` and is
never compared; the register is reused as `i` (x86-64, clang -O2):

```text
.LBB0_9:
	movzbl	(%rdx,%rsi), %r9d       # byte = src[i]
	cmpb	$32, %r9b
	je	.LBB0_11
	movb	%r9b, (%rdi,%rax)       # out[kept] = byte
	incq	%rax                    # kept + 1, proved not to overflow
```

A caller that passes a 5-byte `out` for 6 bytes of input is rejected at the
call, with the callee's requirement it fails:

```text
  requires_clause: drop_spaces.wf:2:3 "requires deref(out).len >= deref(src).len;"
  instantiated_goal: buffer[0..5].len >= text[0..6].len
  disposition: Refuted
```

## What an accepted program cannot do, and what it still can

When the trusted base is correct, an accepted program cannot:

- read or write out of bounds, use freed memory, or read uninitialized memory;
- overflow an integer silently. Each operation states its meaning (`+wrap`,
  `+checked`, `+sat`), and a bare `+` must be proved not to overflow;
- lose a value in a narrowing conversion, or divide by zero;
- panic, abort, throw or unwind. The language has no such construct; expected
  failures are values (`Result`, `Option`) the caller handles;
- behave differently between a debug and a release build. There is one build.

It still can:

- run out of stack. It then stops with the fixed record
  `{"resource":"stack"}`, the same way on every run, and `--stack-ledger`
  reports each function's frame and how many levels each recursive cycle
  fits;
- run out of heap. The allocator stops the program; on Linux with
  overcommit, the kernel's OOM killer may act first;
- loop forever, or compute the wrong answer. Contracts describe what was
  written down, not what was meant;
- be miscompiled. The trusted base is the Whitefoot compiler, LLVM and clang,
  the runtime and allocator, C functions linked in as trusted definitions,
  libc and the operating system ([SCOPE-3](spec/kernel-spec.md)).

## What you write

Contracts on functions (`requires`, `ensures`), `reads`/`writes` effect rows
on signatures, loop invariants, and occasionally an explicit proof step. The
test programs and container library (94 files, 23.6k lines, about 800
functions: a recursive grep, a DEFLATE decoder, a B-tree, a hash map, a
priority queue, a TCP echo server, a directory walker and more) contain 200
contract blocks, 294 invariants and 41 explicit proof steps. The 1,363-line
grep has 3 invariants and no explicit proof step.

The proof procedure is fixed: difference-bound closure, trying zero, one or
two premises per goal ([ENT-1](spec/kernel-spec.md)). There is no timeout and
no work budget, so every machine gives the same verdict. A proof step the
compiler did not need is itself an error, so proofs do not accumulate as
noise.

## Status

Whitefoot is about three months old and is a research compiler, not a
product. One person makes the design rulings; most of the code is written by
AI agents and checked against the specification, the conformance suite and
review. Do not use it for anything that matters.

You cannot yet write:

- calls to C from source; C enters only as trusted linked definitions;
- modules or separate compilation (being designed);
- servers with connection-level concurrency;
- explicit threads, async or SIMD. Under `--par` the compiler runs statements
  or loop iterations in parallel when its proofs show them independent, the
  result equals the sequential one, and `--par-ledger` explains each loop.

## Try it

Requires Rust stable (see [Running the compiler](#running-the-compiler)) and
clang.

```sh
git clone https://github.com/mbbill/Whitefoot.git && cd Whitefoot
cargo build --release --manifest-path compiler/Cargo.toml
compiler/target/release/whitefootc tests/programs/wfgrep.wf -o wfgrep
./wfgrep invariant tests/programs/wfgrep.wf
compiler/target/release/whitefootc tests/conformance/cases/op4-neg-index-undischarged.wf
```

The last command shows a rejection; `--diagnostic-format json` prints it as
JSON.

## Evidence

- [Specification](spec/kernel-spec.md): 121 numbered rules. Every rejection
  cites one rule and one location.
- [Conformance suite](tests/conformance/): 1,214 cases, 568 of which must be
  rejected under a named rule (61 distinct rules).
- [Programs](tests/programs/) built and run by the test gate.
- [Known defects and follow-up work](docs/todo.md), including compiler bugs.
- [Experiments](research/experiments/README.md), negative results included.

## Related work

| | Borrowed | Different |
|---|---|---|
| Rust | ownership, `Result`, no null | no `unsafe` in source; bounds and overflow are proved, not checked at run time |
| SPARK | proving the absence of runtime errors | no SMT solver; what the fixed procedure cannot prove is written as explicit steps |
| Wuffs | a proof checker instead of a solver | a general-purpose language with heap data and effects |
| Dafny, Verus | contracts and invariants | the goal is runtime safety, not full functional correctness |

## Working on the project

The [constitution](docs/constitution.md) owns the objectives and tradeoffs;
[AGENTS.md](AGENTS.md) owns priorities and workflow, including how agents work
under the owner's rulings. Read the material that owns your question:

| Question | Source |
|---|---|
| What does the language admit? | [Active kernel specification](spec/kernel-spec.md) |
| What does this compiler implement, and how do I run it? | [Running the compiler](#running-the-compiler) below; the conformance report states the implemented surface |
| How do I work on a branch and prepare a merge? | [AGENTS.md](AGENTS.md) |
| Which writer forms should I try? | [Patterns](docs/patterns.md) |
| How should I investigate, verify, and maintain documentation? | [Engineering practice](docs/practice.md) |
| Why was a design chosen? | [Design trees](design/), with reasons and refused alternatives |
| Which research questions and experiments could be useful? | [Ideas](docs/ideas.md) |
| What defects and follow-up work remain? | [Todo](docs/todo.md) |

Research and dated essays provide evidence and ideas; they do not add approval
requirements. The reading and authority rules are in
[AGENTS.md](AGENTS.md#authority-and-reading).

Repository layout:

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
- [design/](design/): live design decisions with their reasons, and the
  procedure that maintains them.
- [governance/](governance/): archive-protection hooks and specification-change
  design evidence.
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

A rejection prints the cited rule, the location with the marked source line,
and every payload field under a stable label:

```text
bounds.wf:11:21: error[OP-4]: UndischargedBoundsObligation
  rule: OP-4
  kind: UndischargedBoundsObligation
  category: Source
  stage: Semantics
  at: bounds.wf:11:21
  bytes: 377..383
  source:       set deref(out)[kept] = byte;
  marker:                     ^^^^^^
  residual: kept < deref(out).len
  mechanical_fix: when the relation must hold, establish the residual with ...
```

`--diagnostic-format json` prints the same fields as one JSON object per line.
The [readable-diagnostics investigation](research/investigations/readable-diagnostics/DESIGN.md)
describes the record.

## Verification

`make check` also needs `python3` (design lint, repository invariants and the
conformance runner), and the guarded wrapper `.github/run-check.pl`, used
below and throughout this section, needs `/usr/bin/time` (Debian/Ubuntu
package `time`).

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
