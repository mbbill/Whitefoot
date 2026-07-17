# Rust 1.97 behavior canaries

These source fixtures preserve the safe-Rust counterexamples used by the G0-Core
ownership and behavior analysis. They are evidence about caller-visible Rust
contracts, not proposed xlang mechanisms. Binaries are intentionally excluded.

The pinned compiler is Rust 1.97.0, commit
`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`.

| Source | Purpose | SHA-256 |
|---|---|---|
| `xlang_buildhasher_root_swap.rs` | A shared `BuildHasher` receiver can replace an internal borrow root while producing equivalent hashes. | `76588ebc4bf1cc9c191e4b08f3cee00dffd96fa39e315ab3bd5a057bc7aa9a09` |
| `xlang_buildhasher_transfer.rs` | `build_hasher` can move a unique borrow leaf from stored builder state into the generated hasher without simultaneous liveness. | `0a366e560f3eaf10b85e8bee963a0204d218e3df4766e758e718087c12e9d962` |
| `xlang_clone_source_effects.rs` | `Clone` and `clone_from` can evolve shared source state, and slice helpers propagate those effects. | `456a9ffe70b4df2b90c9d4eb0edf353aeb41f3b135b06e8b31da931293730642` |
| `xlang_clone_helper_source_effects.rs` | Default `clone_from`, array, `Box`, and cloning `Rc` fallbacks can transitively invoke a source-changing payload `Clone`. | `6120847f27ff814f60f0f92a96ff016970ca64cb32aa3718cee6f96607be1284` |
| `xlang_behavior_receiver_effects.rs` | Safe behavior receivers can end, move, or replace borrow leaves through interior mutability across comparison, hashing, bounds, cursor, view, and callback paths. | `ac42b0d6cd70a8ee3b04528184335768f725ee7f925c9d7253897cb8523fdef8` |
| `xlang_repeat_n_source_effects.rs` | `RepeatN` drops a zero-count seed, clones the first `n - 1` results, and moves the evolved seed on the final result. | `ca5cdad81136a15d5b5291a670963d8c4c9a97436bdfd85fb39ed646fe9f3ff5` |
| `xlang_range_step_stable_entrances.rs` | All 21 stable-reachable Rust 1.97 Step endpoint types compile for `Range`, `RangeFrom`, and `RangeInclusive` `iter` entrances with the exact yielded type. | `99a7243215ed384b2ba7fa0890bddbf3c05463e2c2b293552bda2f637da9208d` |
| `xlang_range_step_ascii_char_rejected.rs` | The twenty-second Step implementer, unstable `core::ascii::Char`, is rejected by stable Rust with three exact `ascii_char` E0658 diagnostics. | `a98c1a8c266b173c219127931f44f7c170076127252310886e04440d8465437b` |
| `xlang_range_step_downstream_impl_rejected.rs` | A downstream custom Step implementation is rejected by stable Rust with four exact `step_trait` E0658 diagnostics. | `aa9f46fa6f2aea5a850cf569f7f87867cca4d197c9f221ea083e4830b06171ce` |

Run the complete pinned check from the repository root:

```sh
python3 -B optimizer-language-research/implementation/minimal-systems-capability/tools/verify_behavior_canaries.py
python3 -B optimizer-language-research/implementation/minimal-systems-capability/tools/verify_trait_impl_canaries.py
```

The verifier checks the exact compiler commit and source hashes, then applies
this command shape to each fixture in an isolated temporary directory:

```sh
rustc +1.97.0 --edition=2024 -C opt-level=0 -C debuginfo=0 SOURCE -o OUTPUT
OUTPUT
```

The trait-implementation verifier separately compiles and executes the one
positive Range/Step fixture. It then requires the two negative fixtures to fail
under `--error-format=short`, pins the full diagnostic SHA-256 values, and
checks the exact E0658 feature predicate and occurrence count. This separates
the 22 actual Step implementations from the 21 endpoint types reachable by
stable Rust callers.
