# Rust 1.97.0 Concrete Trait-Implementation Crosswalk

Status: generated evidence for G0-Core review. This artifact selects no xlang
trait syntax, dispatch mechanism, storage representation, or implementation
privilege.

## Purpose and boundary

`RUST-1.97.0-API-INVENTORY.tsv` deliberately inventories public declarations,
not concrete trait implementations repeated by rustdoc on implementer pages.
The data-floor `TRAIT-*` rows nevertheless depend on selected direct
implementations for their exact ownership modes, associated types, result
shapes, and implementation counts. `RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv`
closes that evidence boundary without expanding the main public-API census.

The crosswalk is targeted, not an inventory of every implementation in
`core`, `alloc`, and `std`. Its selector follows the named G0 data-floor shapes:
arrays, slices, `str`, the selected owning containers and shared owners, and
the explicit `Ref`/`RefMut` projection supplement. Cross-type conversions and
comparisons are retained only when both sides remain inside that boundary.
Unstable byte-string, C-string, SIMD, networking-conversion, error-object, and
waker implementations rendered on the same rustdoc pages are excluded.

Every row records the exact implementation signature, implementer, canonical
trait path, associated bindings or required-method result shape, stability,
stable-surface reachability, rustdoc section identity, rust-src line identity,
and SHA-256 of the selected source lines. Duplicate rustdoc renderings must
agree before they are canonicalized. Unknown stability, an absent associated
binding, an absent required-method shape, an unresolved source link, a changed
source digest, or an unrecognized duplicate fails generation.

## Frozen selected sets

| Selection family | Owning coverage cluster | Rows | Exact trait split |
|---|---|---:|---|
| `INTO_ITERATOR` | `TRAIT-INTOITER-01` | 26 | `IntoIterator=26` |
| `EXTEND` | `TRAIT-EXTEND-01` | 22 | `Extend=22` |
| `FROM_ITERATOR` | `TRAIT-COLLECT-01` | 21 | `FromIterator=21` |
| `INDEX` | `TRAIT-INDEX-01` | 14 | `Index=8`, `IndexMut=6` |
| `DEREF` | `TRAIT-DEREF-01` | 10 | base owners `Deref=4`, `DerefMut=3`; guards `Deref=2`, `DerefMut=1` |
| `BORROW_PROJECTION` | `TRAIT-BORROW-01` | 24 | `AsRef=9`, `AsMut=6`, `Borrow=5`, `BorrowMut=4` |
| `CONVERSION` | `TRAIT-CONVERT-01` | 40 | `From=31`, `TryFrom=9` |
| `CLONE` | `TRAIT-CLONE-01` | 16 | `Clone=16` |
| `DEFAULT` | `TRAIT-DEFAULT-01` | 54 | `Default=54` |
| `COMPARISON_HASH` | `TRAIT-CMP-01` | 78 | `PartialEq=36`, `Eq=12`, `PartialOrd=10`, `Ord=10`, `Hash=10` |
| `DROP` | `TRAIT-DROP-01` | 7 | `Drop=7` |
| `RANGE_STEP` | three `RANGE-ITER-*` clusters | 22 | `Step=22`; 21 stable-reachable endpoint types |

The generated evidence corrects three unsupported counts in the earlier
manual census prose:

- the named `IntoIterator` entrance set has 26 direct implementations, not 24;
- the selected `FromIterator` set has 21 direct implementations, not 24; and
- `Deref=4,DerefMut=3` describes only `Box`, `Vec`, `String`, and `Rc` base
  owners. Once the row's stated `Ref`/`RefMut` guards are included, the exact
  selected total is `Deref=6,DerefMut=4`.

The previously claimed `Extend=22`, `Index=8,IndexMut=6`, and direct `Drop=7`
counts are confirmed.

## Step implementation set

Rust 1.97 exposes the new `Range::{iter}` family as stable methods while their
bound is the unstable `core::iter::Step` trait. Rustdoc records exactly 22 Step
implementations:

- the twelve signed and unsigned integer types including pointer-sized types;
- `char`;
- `core::ascii::Char`;
- `Ipv4Addr` and `Ipv6Addr`; and
- six unsigned `NonZero` types.

This set is closed to stable user implementations because `Step` is unstable.
The implementation count and the stable-callable endpoint count are different:
`core::ascii::Char` is itself unstable in Rust 1.97, so 22 implementations
produce 21 stable-reachable endpoint types. The crosswalk records that one
negative reachability row explicitly rather than dropping the implementation.
All 22 rows are owned jointly by `RANGE-ITER-HALFOPEN-01`,
`RANGE-ITER-FROM-01`, and `RANGE-ITER-INCLUSIVE-01`.

## Census integration keys

The affected census rows must cite the generated selection family rather than
claiming that concrete implementations exist in the declaration inventory:

| Census row | Required crosswalk source key |
|---|---|
| `TRAIT-INTOITER-01` | `RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv selection_family=INTO_ITERATOR` |
| `TRAIT-EXTEND-01` | `... selection_family=EXTEND` |
| `TRAIT-COLLECT-01` | `... selection_family=FROM_ITERATOR` |
| `TRAIT-INDEX-01` | `... selection_family=INDEX` |
| `TRAIT-DEREF-01` | `... selection_family=DEREF` |
| `TRAIT-BORROW-01` | `... selection_family=BORROW_PROJECTION` |
| `TRAIT-CONVERT-01` | `... selection_family=CONVERSION` |
| `TRAIT-CLONE-01` | `... selection_family=CLONE` |
| `TRAIT-DEFAULT-01` | `... selection_family=DEFAULT` |
| `TRAIT-CMP-01` | `... selection_family=COMPARISON_HASH` |
| `TRAIT-DROP-01` | `... selection_family=DROP` |
| each new `RANGE-ITER-*` row | `... selection_family=RANGE_STEP` |

The crosswalk is cross-cutting evidence and does not add declarations to the
5,555 canonical stable declaration count, the 545 safe seed declaration count,
or the 276 coarse coverage-cluster count. These implementation rows are
evidence owned by those clusters; they are not normalized contracts and may not
be imported as a closed capability basis.

## Reproduction

The builder discovers the exact toolchain with `rustc +1.97.0 --print sysroot`.
An explicit `--sysroot` is also accepted, but its compiler identity must match
release 1.97.0 and commit
`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`. Both the `rust-docs` and
`rust-src` components must be present.

```sh
python3 -B \
  optimizer-language-research/implementation/minimal-systems-capability/tools/build_trait_impl_crosswalk.py \
  --check
python3 -B \
  optimizer-language-research/implementation/minimal-systems-capability/tools/verify_trait_impl_crosswalk.py
```

The verifier pins all 334 implementation identities, every family and trait
count, each family identity-set digest, exact associated bindings, source-line
digests, the 22/21 Step distinction, the base-owner versus guard Deref split,
and coverage of every owning coverage cluster.
