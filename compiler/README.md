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
```

The frontend targets the exact bytes of
`../spec/kernel-spec-v0.10.md`. `cargo run --bin whitefoot-spec` checks that
those bytes are the approved candidate and that the terminal and grammar data
name the same specification identity. The committed grammar tables are
ordinary compiler data; the planned grammar-change verifier must derive and
check them through this compiler before a future spec version is proposed.

The next implementation is direct general name resolution over the canonical
tree. There is deliberately no artifact protocol, catalog interface, replay
layer, resource-profile product, or compatibility boundary in front of it.

Run the compiler gate with:

```sh
make check
```
