# Code generation regression corpus

Status: preserved future-facing corpus; not an active gate.

This directory retains Whitefoot source cases, negative controls, and historical
per-family expectation inventories that can become production-compiler codegen
regressions as the relevant language capabilities arrive. The Rust compiler
already has an LLVM backend, but it cannot yet compile these feature families
and no current Rust harness consumes this corpus.

The democ-bound Python runner, its top-level manifest and schema, and its runner
tests are preserved under `archive/tests/codegen/`. They must not be restored as
active tooling. The `cases.json` files beside source families are migration
evidence, not current specification authority or active expected results.

A future integration belongs beside the real backend and must exercise each
selected source through the normal compiler path. It has four layers:

1. executable correctness for lowered programs, explicit claims, and typed
   failure outcomes;
2. acceptance and rejection tests proving that every partial operation has its
   static obligation discharged before lowering;
3. facts-on/facts-off comparisons proving that optimization facts change
   neither acceptance nor the execution of written claims, with near-miss
   programs that must reject; and
4. runtime and code-shape measurements kept under `research/`, because noisy
   timing is experimental evidence rather than an every-commit invariant.

Before promotion, reconcile each selected source with the active numbered
specification, preserve its positive or negative-control purpose, and replace
the historical expectation inventory with a small Rust-owned regression. A
case leaves this holding corpus only after that mapping or after an explicit
decision that its hypothesis is obsolete.

Do not make this corpus an optimizer dispatch table. Production compilation
must implement grammar and semantic rules, never case names, source hashes, or
manifest identities.
