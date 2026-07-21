# Whitefoot

Whitefoot is a systems language for AI-written, human-approved code. It is
designed so that memory corruption, data races, uninitialized reads, and silent
overflow are unrepresentable in accepted source. There is no writer-accessible
unsafe escape. Runtime safety checks remain enabled unless a machine-verified
proof authorizes their removal.

[THE-PLAN.md](THE-PLAN.md) is the sole source for current execution order,
authorization, gates, and stop conditions.

## Authority and handoff state

[Kernel specification v0.8](spec/kernel-spec-v0.8.md), SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`,
remains the immutable active specification and evidence baseline. Its recorded
grammar and semantic contradictions block a production parser. Compiler code
may not silently resolve them.

The audited Rust-foundation handoff and standalone grammar-change verifier are
complete. The verifier is outside the production compiler dependency graph. It
reproduces exact-v0.8 and non-authoritative successor evidence, but it does not
edit a numbered specification, change protected expectations, switch the active
target, or authorize a parser. Phase 3 is the next conditional gate: the exact
candidate bytes, hash, evidence, and impact census require advance owner review
before any guarded installation.

## Production design

Whitefoot exposes ownership, effects, numeric modes, cleanup, checks, and
verified laws as explicit semantic facts. A source-level option may never
weaken a safety check for speed.

The future production path uses one semantic kernel:

```text
exact source transport
  -> lossless lexer and canonical syntax
  -> one semantic kernel
  -> private checked draft
  -> canonical artifact bytes
  -> artifact-only decode and complete replay through the same kernel
  -> accepted compilation
  -> conservative generic lowering
```

The originating invocation must complete that same-kernel replay before it can
construct lowering authority. There is no second production semantic verifier.
Replay checks that the canonical artifact fully and consistently records the
accepted invocation; it is not independent semantic evidence and does not make
the kernel less trusted. Independent models, hostile tests, and differentials
remain mandatory evidence around the production path.

Optional optimizer facts form a later, independently verified overlay. The
empty overlay always works, and an optional fact can improve an already
accepted program but cannot change acceptance, semantic identity, or required
checks.

## What exists now

The safe-Rust foundation contains four narrowly scoped crates:

- `whitefoot-contract` owns exact identities, bounded source transport, spans,
  ceilings, and the version-1 source-binding wire contract.
- `whitefoot-lexer` losslessly partitions exact source bytes into shape-only
  tokens and retained trivia. It does not parse or accept programs.
- `whitefoot-source-audit` checks exact source/specification binding only. It
  is not an artifact verifier or semantic checker.
- `whitefoot-lexical-observer` is a binary-only development adapter for an
  independent byte-level lexical differential. Its output is evidence only.

The compiler-independent `grammar-verifier/` contains two deliberately
independent engines: a safe-Rust strong-LL(2) auditor and a bounded Python
generalized-parser Oracle. Their final common extraction ledger agrees byte for
byte. Both registered dereference cases change from two derivations to one under
the candidate terminal partition, with no introduced derivation or static
intersection/conflict. This is review evidence, not specification authority.

`SourceBundle` ordering is transport order, not normative multi-file or
declaration-order semantics. The workspace has no production terminal
classifier, parser, syntax authority, resolver, semantic kernel, checked
artifact, backend, compiler executable, or release capability.

The compiler-independent conformance corpus, proof/code-shape premise corpus,
focused reference models, and measured experiments remain active evidence.
None is compiler authority, and a corpus count or passing example is not a
completeness claim.

Run the development gates with:

```sh
make -C compiler check
make check
```

A green result states only the capabilities that currently exist. It is not a
compiler-conformance or release claim. Compiler failures and resource outcomes
belong to adapter results, never to normative expected behavior.

## Durable verification

The compiler implementation is replaceable; the evidence is intended to
survive it. The durable system includes specification-versioned conformance,
independent grammar and semantic models, valid and hostile artifact vectors,
property and mutation testing, facts-on/facts-off controls, and full-project
compatibility protocols. Measured experiments under
[experiments/](experiments/README.md) remain mechanism findings with explicit
scope, not general performance or product claims.

## Repository guide

| Purpose | Location |
|---|---|
| Current execution order and authorization | [THE-PLAN.md](THE-PLAN.md) |
| Language specification | [spec/kernel-spec-v0.8.md](spec/kernel-spec-v0.8.md) |
| Specification source index and semantic catalog | [facets/v0.8/](facets/v0.8/README.md) |
| Active Rust compiler workspace | [compiler/](compiler/README.md) |
| Standalone grammar-change evidence | [grammar-verifier/](grammar-verifier/README.md) |
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
