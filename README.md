# Whitefoot

Whitefoot is an experimental systems language for AI-written, human-approved
code. It is designed so that memory corruption, data races, uninitialized
reads, and silent overflow are unrepresentable in accepted source. There is no
writer-accessible unsafe escape. Runtime safety checks remain enabled unless a
machine-verified proof discharges them.

The exact first implementation target is
[kernel specification v0.8](spec/kernel-spec-v0.8.md). The production compiler
is being built from scratch in safe Rust. The former Python demo compiler and
the incomplete Whitefoot self-hosting compiler are preserved under
[archive/toolchains/self-hosting-2026-07-20](archive/toolchains/self-hosting-2026-07-20/README.md)
as historical evidence; they have no active authority.

The current execution order and gates are in [THE-PLAN.md](THE-PLAN.md).

## Design

Whitefoot exposes facts that an ordinary compiler often has to recover
imperfectly:

- ownership and exclusive loans establish aliasing facts;
- exact effect rows state what a function can read, write, allocate, or trap;
- arithmetic modes make overflow behavior local and explicit;
- checked laws can authorize transformations only after verification; and
- canonical source gives one byte-level spelling for a program.

A source-level option may never weaken a safety check for speed. The permanent
pipeline is:

```text
source
  -> Rust frontend and syntax-directed checker
  -> candidate proof-bearing CheckedUnit
  -> independent verifier
  -> VerifiedCheckedUnit
  -> generic LLVM lowering
```

Optimizer facts are a later, independently verified overlay. They can improve
code for an already accepted program but cannot change acceptance.

## Current status

The repository is in the Rust compiler foundation phase.

- v0.8 is frozen as the exact initial implementation target.
- The compiler-independent conformance corpus and codegen premise corpus remain
  active.
- The focused Python reference checker and model checker remain active as
  bounded independent evidence, not as a compiler.
- The active safe-Rust workspace now pins the exact toolchain and specification,
  owns the ordered raw-source contract, and independently verifies exact
  source/spec binding. Its first frontend boundary losslessly partitions exact
  source bytes into shape-only tokens and retained trivia under explicit
  ceilings; this is not yet parsing or language acceptance. It has no compiler
  executable or conformance adapter yet.
- A small byte-exact lexical corpus and independent model exercise that same
  non-authorizing boundary without importing compiler code. They do not yet
  constitute a compiler differential or capability receipt.
- The exact-v0.8 structural source index is generated independently of compiler
  code. Its counts are integrity facts, not an implementation-progress measure;
  authored semantic decomposition and compiler capability remain separate.
- Existing performance experiments are evidence with explicit scope and
  caveats; they are not claims about a finished language or compiler.

Run the currently applicable repository gate with:

```sh
make check
```

The gate reports the exact incomplete development state and runs the Rust
workspace checks; it does not claim compiler conformance. A production release
cannot pass while any normative v0.8 facet is pending, skipped, xfail, or
unsupported.

## Durable verification

The compiler implementation is replaceable. The durable verification system
includes:

- specification-versioned source and expected verdicts;
- a complete spec-derived facet catalog;
- focused independent semantic models;
- hand-authored valid and hostile artifact vectors;
- property, metamorphic, fuzz, and mutation testing;
- facts-on/facts-off controls; and
- full compatibility protocols for real replacement projects.

Compiler capability and implementation failures belong to adapter-owned data,
never to normative expected behavior.

## Evidence boundary

The repository contains measured wins, losses, and failed hypotheses under
[experiments/](experiments/README.md). For example, the base64 experiment showed
that proof-driven check removal can make a direct indexed loop reach the same
performance class as expert safe Rust, while the zlib kernel study showed that
literal lowering is insufficient for important overlapping-copy and decode
shapes. These are mechanism findings, not general performance claims.

Real projects become product evidence only when the complete declared interface
passes upstream compatibility tests, differential fuzzing, downstream
integration, resource and failure testing, reproducibility, and
preregistered target-specific performance gates. Isolated kernels do not count
as production rewrites.

## Repository guide

| Purpose | Location |
|---|---|
| Current execution order and authorization | [THE-PLAN.md](THE-PLAN.md) |
| Language specification | [spec/kernel-spec-v0.8.md](spec/kernel-spec-v0.8.md) |
| Specification source index and facet work | [facets/v0.8/](facets/v0.8/README.md) |
| Active Rust compiler workspace | [compiler/](compiler/README.md) |
| Project law and writer patterns | [CONSTITUTION.md](CONSTITUTION.md), [PATTERNS.md](PATTERNS.md) |
| Compiler-independent behavior corpus | [conformance/](conformance/README.md) |
| Compiler-independent lexical probes | [frontend-corpus/v0.8/](frontend-corpus/v0.8/README.md) |
| Proof/code-shape premise corpus | [codegen-corpus/](codegen-corpus/README.md) |
| Focused reference semantics | [prototype/checker/](prototype/checker/) |
| Measured evidence | [experiments/](experiments/README.md) |
| Design decisions and rejected routes | [mcts_mem/](mcts_mem/) |
| Append-only implementation record | [decision-gates.md](optimizer-language-research/implementation/decision-gates.md) |
| Retired compiler implementations | [archive/toolchains/self-hosting-2026-07-20/](archive/toolchains/self-hosting-2026-07-20/README.md) |
| Repository instructions | [AGENTS.md](AGENTS.md) |

## License

Whitefoot is available under the [MIT License](LICENSE).
