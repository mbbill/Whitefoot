# Whitefoot

Whitefoot is a research systems programming language. A program the compiler
accepts cannot reach undefined behavior, a panic, or a silent integer overflow
at run time, provided its [trusted base](#what-an-accepted-program-cannot-do-and-what-it-still-can)
is correct. It gets there without runtime checks: every indexing, arithmetic,
conversion, division and allocation-size operation needs a proof that it is in
range, and the compiler finds most of those proofs itself, with a fixed
procedure and no SMT solver. A proved check costs nothing at run time.

## A bounds check that is proved away

This loop keeps the non-space bytes of a buffer in place. In Rust:

```rust
pub fn squeeze(buf: &mut [u8]) -> usize {
    let mut kept = 0;
    for i in 0..buf.len() {
        let b = buf[i];
        if b != b' ' {
            buf[kept] = b;
            kept += 1;
        }
    }
    kept
}
```

`buf[kept]` is always in range, because `kept` never passes `i`. Seeing that
takes induction over the loop, which the optimizer does not do, so the
compiled loop compares `kept` with the length for every byte it keeps. In
Whitefoot the reason is one more line, and the compiler checks it:

```
fn squeeze(buf: &[u8]) -> kept: u64 writes(buf) {
  let kept = 0_u64;
  for (
    i in 0_u64..deref(buf).len,
    invariant behind: kept <= i
  ) {
    let byte = deref(buf)[i];
    if byte != 32_u8 {
      set deref(buf)[kept] = byte;
      set kept = kept + 1_u64;
    }
  }
  return kept;
}
```

The compiler proves that `kept <= i` holds on entry and after every
iteration; with `i < buf.len` that gives `kept < buf.len`, so no check is
left:

| Spelling | Check left in the loop | Needs `unsafe` |
|---|---|---|
| Rust, `buf[kept] = b` | yes | no |
| Rust, [six other safe spellings](research/experiments/bounds-check-spellings/README.md#results) (`get_mut`, `copy_within`, `Cell`, a branchless store, a `while` loop, `assert!(kept <= i)`) | yes | no |
| Rust, `get_unchecked_mut` or `assert_unchecked` | no | yes; the reason is a comment |
| Whitefoot, as above | no | no; the reason is checked |

(rustc 1.98.1, x86-64, `-C opt-level=2` and `3`. Two safe rewrites also leave
no check by changing the algorithm: `retain` on an owned `Vec`, which a slice
borrowed from a caller does not have, and collecting the kept bytes into a new
vector and copying them back, which allocates and reads them twice.)

Drop the invariant, leaving the header `for (i in 0_u64..deref(buf).len) {`,
and the program is rejected, with the fact that is missing:

```text
squeeze.wf:6:21: error[OP-4]: UndischargedBoundsObligation
  source:       set deref(buf)[kept] = byte;
  marker:                     ^^^^^^
  residual: kept < deref(buf).len
  mechanical_fix: …
```

## Write sequential code, get parallel results

Every function states what it reads and writes (`reads(...)`, `writes(...)`),
and the compiler checks that statement against the body. So at any two
statements it knows which memory each one touches. When it can prove the two
touch different memory, it may run them in parallel, and the result is the
one the sequential program computes. Nothing in the source asks for it:

```
fn quicksort(v: &[u64]) -> result: unit writes(v) {
  let n = deref(v).len;
  if n <= 1_u64 {
    return unit;
  }
  let p = partition(v: v);
  let after = p + 1_u64;
  let smaller = &deref(v)[0_u64..p];
  let larger = &deref(v)[after..n];
  quicksort(v: smaller);
  quicksort(v: larger);
  return unit;
}
```

Compiled with `--par`, the two recursive calls run in parallel, because the
compiler proves that `[0, p)` and `[p + 1, n)` do not overlap. The recursion
fans out to a depth the runtime derives from the number of workers and runs
sequentially below it. `--par-ledger` explains every decision:

```text
PAR permitted   quicksort.wf:10  pair(quicksort, quicksort)  eligible
PAR frontier    component(quicksort)  budget-carrying clone family, entered with recursion budget runtime-derived
```

A run that sorts 2 million numbers takes 0.18 s sequentially and 0.07 s on 4
workers ([how it was measured](research/experiments/par-quicksort/README.md)).

## Articles

Short pieces, each on one idea, with programs that compile. The first two
start from the examples above:

1. Prove it or write a branch — bounds and overflow checks that disappear
   because they are proved.
2. Write sequential code, get parallel results — how the compiler finds
   independence in plain code, recursion included, and hands it to the
   workers.
3. Proofs without a solver, by hand — difference bounds, closure and loop
   invariants, worked on paper.
4. What a rejection tells you — diagnostics written for the agent that fixes
   the code.
5. Integers — every operation states its meaning.
6. One build — no panic, no debug/release split, a fixed record on resource
   exhaustion.
7. What a reviewer reads — contracts and effect rows as the review surface.
8. The trusted base — what is trusted, and the plan to shrink it.
9. Where the speed comes from.
10. A layout engine — the first large program.
11. How this project is built with agents.

The articles are being written; each title becomes a link when its article is
published.

## What an accepted program cannot do, and what it still can

When the trusted base is correct, an accepted program cannot:

- read or write out of bounds, use freed memory, or read uninitialized memory;
- overflow an integer silently. Each operation states its meaning (`+wrap`,
  `+checked`, `+sat`), and a bare `+` must be proved not to overflow;
- lose a value in a narrowing conversion, or divide by zero;
- race: parallel execution comes only from proved independence, and its
  result equals the sequential one;
- panic, abort, throw or unwind. The language has no such construct; expected
  failures are values (`Result`, `Option`) the caller handles;
- behave differently between a debug and a release build. There is one build.

It still can:

- run out of stack. It then stops with the fixed record
  `{"resource":"stack"}`, the same way on every run, and `--stack-ledger`
  reports each function's frame and how many levels each recursive cycle fits;
- run out of heap. The allocator stops the program; on Linux with overcommit,
  the kernel's OOM killer may act first;
- loop forever, or compute the wrong answer. Contracts describe what was
  written down, not what was meant;
- be miscompiled. The trusted base is the Whitefoot compiler and its checker,
  LLVM and clang, the runtime and allocator, C functions linked in as trusted
  definitions, libc and the operating system ([SCOPE-3](spec/kernel-spec.md)).

## What you write

Contracts on functions (`requires`, `ensures`), `reads`/`writes` rows on
signatures, loop invariants, and occasionally an explicit proof step. The test
programs and container library contain about 200 contract blocks, 290
invariants and 41 explicit proof steps across about 800 functions. The grep,
about 1,700 lines, needs 2 invariants and no explicit proof step.

The proof procedure is fixed ([ENT-1](spec/kernel-spec.md)): it has no timeout
and no work budget, so every machine gives the same verdict. Proof steps for
an invariant the compiler proves on its own are themselves an error, so proofs
do not accumulate as noise.

## Status

Whitefoot started in July 2026 and is a research compiler, not a product. One
person makes the design rulings; most of the code is written by AI agents and
checked against the specification, the conformance suite and review. Do not
use it for anything that matters.

Not yet available:

- calls to C from source; C enters only as trusted linked definitions;
- high-concurrency I/O for servers, which is being designed;
- explicit threads, async or SIMD. Parallelism comes from `--par` as
  described above.

## Try it

Requires Rust stable (see [Running the compiler](#running-the-compiler)) and
clang.

```sh
git clone https://github.com/mbbill/Whitefoot.git && cd Whitefoot
cargo build --release --manifest-path compiler/Cargo.toml
compiler/target/release/whitefootc tests/programs/wfgrep.wf -o wfgrep
./wfgrep invariant tests/programs
compiler/target/release/whitefootc tests/conformance/cases/op4-neg-index-undischarged.wf
```

Building the compiler takes about a minute; compiling the grep takes about
four seconds. The last command shows a rejection; `--diagnostic-format json`
prints it as JSON.

## Evidence

- [Specification](spec/kernel-spec.md): 130 numbered rules. Every rejection
  cites one rule and one location.
- [Conformance suite](tests/conformance/): about 1,250 cases, more than 600
  of which must be rejected under a named rule (nearly 70 distinct rules).
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
| What happens when in development, where, and who decides? | [Workflow map](docs/workflow.md) |
| How do I work on a branch and prepare a merge? | [AGENTS.md](AGENTS.md) |
| How do I amend the specification, finish a task, or hand work back? | [Agent skills](docs/skills/) |
| Which writer forms should I try? | [Patterns](docs/patterns.md) |
| How should I investigate, verify, and maintain documentation? | [AGENTS.md](AGENTS.md#how-work-proceeds), the [investigation skill](docs/skills/investigation/SKILL.md) and the [document roles](docs/workflow.md#document-roles) |
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
- [docs/](docs/): principles, writer guidance, the workflow map, agent
  skills, and reference material.
- [research/](research/README.md): investigations and experiments with their
  designs, measurements, and rejected alternatives.
- [design/](design/): live design decisions with their reasons, and the
  procedure that maintains them.
- [governance/](governance/): archive-protection hooks and specification-change
  design evidence.
- [.github/](.github/): CI, repository checks and the pull-request template.
- [.agents/skills/](.agents/skills/) and [.claude/skills/](.claude/skills/):
  links through which Codex and Claude Code discover the skills kept in
  `docs/skills/` and `design/skill/`.
- [archive/](archive/): frozen historical material. Active source, builds,
  tests, and tools do not depend on it.

## Running the compiler

Prerequisites: a Rust stable toolchain at least the version in
[compiler/Cargo.toml](compiler/Cargo.toml)'s `rust-version` (`rustup update
stable` on an older installed stable — rustup does not update it on its own),
and clang available at `/usr/bin/clang` on Linux/macOS or as `clang` on PATH
on Windows. A cached build that links ThinLTO fragments (`--cache DIR
--fragments module|function`) also needs LLD on Linux and Windows; the macOS
toolchain's linker does link-time optimization itself. `--full-lto` is
research-only: it builds the comparator that the
[build-cost experiment](research/experiments/modular-build-cost/RESULTS.md)
measures fragment builds against, the program and its runtime optimized as one
region, with the same linker requirement, and is not a build mode for
programs.

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

A rejection prints the location, the cited rule and the kind, the marked
source line, and every payload field under a stable label:

```text
bounds.wf:11:21: error[OP-4]: UndischargedBoundsObligation
  source:       set deref(out)[kept] = byte;
  marker:                     ^^^^^^
  residual: kept < deref(out).len
  mechanical_fix: when the relation must hold, establish the residual with ...
```

`--diagnostic-format json` prints the complete record, category, stage and
byte interval included, as one JSON object per line.
The [readable-diagnostics investigation](research/investigations/readable-diagnostics/DESIGN.md)
describes the record.

## Verification

`make check` also needs `python3` (design lint, repository invariants and the
conformance runner), LLD on Linux (`ld.lld`, Debian/Ubuntu package `lld`) for
the fragment-build test, and the guarded wrapper `.github/run-check.pl`, used
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
macOS, and the
[design-readiness workflow](.github/workflows/design-readiness.yml) rejects
pending design amendments on a pull request that is ready for review.
`make review-scope` lists what a completion review covers. Additional
[I/O host checks](.github/workflows/io-hosts.yml) and
[benchmarks](.github/workflows/io-bench.yml) own their platform-specific
evidence. Automatic CI checks correctness and performance regressions under
the [test rules](AGENTS.md#specification-and-test-integrity): useful
research cases and their dependencies belong in formal tests, while research
runs on explicit request. Full IO matrices and compute scoreboards are
experiments; the separate
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
