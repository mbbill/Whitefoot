# Whitefoot production compiler

This directory contains the one permanent safe-Rust implementation of the
Whitefoot compiler. There is no disposable compiler and no parallel
self-hosting track.

The immutable active specification and evidence baseline is
`../spec/kernel-spec-v0.8.md`, SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.
Recorded contradictions in that specification block a production parser until
an exact successor is separately approved and installed. Compiler code may not
invent a resolution. `../THE-PLAN.md` is the sole source for implementation
order and authorization.

## Handoff state

Phase 1's audited foundation handoff is complete. The workspace establishes
source, lexical, identity, resource, and build boundaries only. It is not yet a
source-to-code compiler and makes no conformance or release claim.

Phase 2's standalone grammar-change verifier and evidence package are complete.
That tool stays outside the production compiler dependency graph. It has
prepared exact, non-authoritative successor bytes and impact evidence, but it
does not edit a numbered specification, switch the active target, or create
production parser or artifact structures. Phase 3 remains conditional on exact
owner review and advance approval; no production frontend work is authorized
by the Phase-2 result.

## Current crates

- `whitefoot-contract` owns judgment-free contracts: exact specification and
  nominal catalog identities, bounded ordered source transport, source
  identities and spans, resource ceilings, and the version-1 source-binding
  wire format. Owned construction reports limit and allocation failures.
- `whitefoot-lexer` depends only on `whitefoot-contract`. It losslessly
  partitions exact source bytes into shape-only tokens and retained trivia
  under explicit ceilings. It does not classify grammar terminals, parse,
  resolve, or accept a program.
- `whitefoot-source-audit` depends only on `whitefoot-contract`. It checks that
  a decoded candidate binding names the expected specification and reproduces
  the exact source transport. It does not replace the binding codec's separate
  canonical-byte validation and is not an artifact verifier, semantic checker,
  or independent production authority.
- `whitefoot-lexical-observer` is a binary-only development adapter over the
  contract and lexer. It exposes the lexer's closed outcome families to an
  independent byte-level model. Requests and responses are observation data,
  not tokens with portable identity, parser output, verdicts, receipts, or
  checked artifacts.

The `SourceBundle` sequence is transport order only. It does not define
normative file composition, declaration order, zero-source behavior, or program
root extent. Those meanings require an approved successor specification.

No production terminal classifier, parser, syntax tree authority, resolver,
semantic kernel, semantic record, artifact schema, backend, compiler
executable, conformance adapter, or release capability exists here yet. No
active code imports the retired implementations under `../archive/`.

## Later production authority path

After the specification and phase gates permit it, production acceptance uses
one semantic kernel:

```text
canonical syntax
  -> one semantic kernel
  -> private checked draft
  -> target qualification
  -> canonical artifact projection
  -> artifact-only decode and complete replay through the same kernel
  -> accepted compilation from replay-decoded state
  -> conservative generic lowering
```

Only mandatory replay in the originating invocation may construct lowering
authority. There is no second production semantic certificate verifier.
Same-kernel replay detects incomplete projection, codec, reference, and
corruption defects; it is not independent semantic evidence and does not
reduce the trusted semantic kernel. The current source-audit crate is not a
placeholder for that future replay boundary.

Optional optimizer propositions are a separate later overlay with their own
independent family verifiers. The canonical empty overlay must always lower
correctly, preserving every unproved runtime check.

## Non-authority data

The capability overlay at `../capabilities/whitefoot-rust/v0.8/`, the static
semantic catalog, discrepancy registry, source index, corpora, and experiment
receipts are audit evidence only. Production code must not read them to select
acceptance, semantic handling, or lowering. A digest proves byte identity, not
truth, completeness, or implementation capability.

The version-1 `WFSOURCE` codec remains exactly source transport plus
specification identity. Catalog, tree, proof, semantic, target, and backend
identities do not belong in it. No future artifact envelope should be created
until its canonical schema and real consumer are authorized.

## Gates

Run the compiler gate from the repository root:

```sh
make -C compiler check
```

Run the complete development gate before and after a completed repository
slice:

```sh
make check
```

The compiler gate pins the exact Rust toolchain, v0.8 bytes, static catalog
identity, package graph, dependencies, source policy, formatting, linting,
tests, rustdoc, and cross-path reproducibility. Builds are locked and offline;
the required toolchain must already be installed. The workspace forbids
`unsafe`, build scripts, procedural macros, unapproved dependencies,
source-splicing, and archive imports.

A green gate describes only this incomplete foundation. It does not claim that
Whitefoot source can be parsed, accepted, lowered, or released.
