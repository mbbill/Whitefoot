# Whitefoot compiler

This directory is one safe-Rust crate containing the active compiler. It is an
implementation, not a collection of stable libraries: module boundaries are
private design choices and should change when the next compiler capability
needs them.

The implemented path is currently:

```text
ordered source bundle
  -> lossless lexer
  -> terminal classification
  -> strong-LL(2) parser
  -> finalized source-bound syntax tree
  -> exact FORM-2 validation
  -> direct v0.14 lexical name resolution
  -> semantic and ownership checking
  -> private checked program
  -> target-independent typed control-flow IR
  -> conservative textual LLVM
  -> host executable
```

The frontend targets the exact bytes of
`../spec/kernel-spec-v0.14.md`. `cargo run --bin whitefoot-spec` checks that
those bytes are the approved candidate and that the terminal and grammar data
name the same specification identity. The committed grammar tables are
ordinary compiler data. For a specification proposal, run the native verifier
through this compiler:

```sh
cargo run --bin whitefoot-grammar -- \
  ../governance/spec-evolution/kernel-spec-vN-candidate.md
```

It compares the proposal's complete canonical-format, lexer, and grammar
contract with the active contract, checks the compiler's terminal inventory
and every strong-LL(2) decision, and runs the real lexer and parser. It fails
closed when a proposal changes that contract; a structural change must first
extend this same native path rather than reviving an independent grammar
engine.

The resolver covers every v0.14 declaration, lexical-use, and deferred
owner/member role through one grammar-driven path, including exact scopes,
visibility, reservations, collisions, and deterministic diagnostics.

The implemented semantic families support exact scalar integers, unit,
`Bool`, integer and unit constants, nongeneric own-mode functions, locals,
direct calls, returns, pure/traps effects, wrapping and trapping
add/subtract/multiply, checked add/subtract/multiply/divide/remainder, integer
absolute value and negation in all three modes, integer comparisons, Boolean
operations, and nominal tag equality. Checked division and remainder guard
divisor zero and signed minimum/-1 before the partial LLVM instruction and
produce the exact `Result<T, DivError>` variant. Absolute value uses
defined-edge `llvm.abs` for every signed width: wrapping retains the minimum
value, trapping emits OP-2, and checked returns `Err(Overflow())`. Negation uses
modular `sub 0, x` for wrapping and signed-subtraction overflow detection for
trapping and checked modes, with no `nsw`/`nuw` promises. Nongeneric acyclic
structs and enums flow through the same path,
including construction, nested projection, statement/value matching, `give`,
per-site exhaustiveness checking, whole-binding affine moves, and explicit
reverse-order cleanup edges. Consuming field projections also retain the
untouched affine sibling subtrees that must be dropped. SET-1 supports live
own-mode copy locals and nested copy fields, rejects affine replacement under
STOR-1, and rechecks target liveness after the right-hand side. Semantic
success produces the only lowering authority. The IR retains required checks,
source trap sites, checked set paths, and cleanup;
the backend uses conservative LLVM without unearned overflow flags or check
elision. Unimplemented v0.14 families stop as explicit unsupported compiler
capabilities rather than source-language rejections. Whole-unit ERR-2
variant-addition edit-list enumeration and the full conformance adapter remain
future work. Index and borrow-backed SET-1 targets remain explicit unsupported
capabilities until those place families exist; none of these gaps is implied
complete by the current gate.

Compile a source file through the normal path with:

```sh
cargo run --bin whitefootc -- source.wf -o program
cargo run --bin whitefootc -- --emit-llvm source.wf
```

There is deliberately no artifact protocol, replay layer, resource-profile
product, or compatibility boundary in front of this path.

Run the compiler gate with:

```sh
make check
```
