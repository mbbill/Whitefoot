# Whitefoot research compiler

This directory contains the continuing safe-Rust compiler implementation. It
is intended to become a real, general compiler for language and performance
experiments, not a throwaway demo and not an LLVM-scale product.

The immutable active target is `../spec/kernel-spec-v0.9.md`, SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
The exact approved v0.10 candidate is not active until the guarded switch and
frontend reproduction complete. `../THE-PLAN.md` is the sole source for work
order.

## Current capability

The compiler currently ends at `CanonicalSyntaxUnit`:

```text
SourceBundle
  -> lossless shape lexer
  -> terminal classifier
  -> iterative strong-LL(2) parser
  -> one private derivation
  -> topology and source finalizer
  -> streaming FORM-2 validation
  -> CanonicalSyntaxUnit
```

That value proves syntax and exact source binding only. There is no resolver,
semantic checker, IR, LLVM backend, CLI, or executable-program capability yet.

## Active crates

- `whitefoot-contract` owns shared source, identity, span, and frontend
  contracts.
- `whitefoot-language-data` owns specification-derived terminal predicates.
- `whitefoot-lexer` partitions exact bytes into shape-only tokens and trivia.
- `whitefoot-syntax-data` owns generated grammar and strong-LL(2) data.
- `whitefoot-syntax` classifies, parses, finalizes, and validates exact FORM-2
  bytes before publishing canonical syntax.
- `whitefoot-source-audit` checks exact source/specification binding.
  byte-level development model.

These boundaries are private compiler implementation details and may evolve as
real downstream consumers appear. Do not add stable schemas, artifact replay,
resource-profile systems, or placeholder crates for imagined future stages.

## Next capability

After the exact v0.10 switch, implement one direct general resolver over
`CanonicalSyntaxUnit`. It must classify and resolve every grammar-defined name
role without function, project, source-shape, or corpus special cases. Use
ordinary safe-Rust data structures and expose only the records needed by the
type checker.

After resolution, build the smallest coherent semantic family end to end
through a simple target-independent IR and LLVM. Other valid language features
may remain explicitly not implemented while the compiler grows; they must not
be mislabeled as invalid source.

## Engineering standard

- Follow the active numbered specification.
- Keep one general compiler path.
- Preserve all required safety checks on the facts-off path.
- Prefer simple normal Rust over preemptive infrastructure.
- Keep internal APIs easy to change.
- Add independent evidence where it catches plausible semantic mistakes.
- Use dogfood programs to choose capability order, never to select semantics.
- Keep files cohesive by responsibility.

Run the workspace gate with:

```sh
make -C compiler check
```

The root `make check` includes this gate.
