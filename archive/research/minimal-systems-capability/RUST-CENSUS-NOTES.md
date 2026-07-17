# Rust 1.97.0 Census Notes

Status: mechanical evidence inventory, 2026-07-14. This document and its TSV
files are inputs to contract normalization. Item counts are not capability
counts and select no xlang mechanism.

## Provenance

The inventory is generated from the official Rust 1.97.0 rustdoc tree and the
matching `library/` source checkout at commit
`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`. The local compiler fingerprint is
recorded in `RUST-1.97.0-CENSUS-MANIFEST.json` and agrees with that commit.

Reproduction command, with the exact toolchain and source checkout installed:

```sh
python3 -B \
  optimizer-language-research/implementation/minimal-systems-capability/tools/extract_rust_api.py \
  --doc-root "$HOME/.rustup/toolchains/1.97.0-aarch64-apple-darwin/share/doc/rust/html" \
  --source-root /path/to/rust-1.97.0 \
  --output-dir optimizer-language-research/implementation/minimal-systems-capability
```

The checked artifacts are:

- `RUST-1.97.0-API-INVENTORY.tsv` — public items, inherent members, and trait
  declarations;
- `RUST-1.97.0-MODULE-ACCOUNTING.tsv` — every reachable public module, including
  counted and digested collapsed modules; and
- `RUST-1.97.0-CENSUS-MANIFEST.json` — exact version, policies, counts, missing
  pages, external links, and output hashes.

Run the independent gate with:

```sh
python3 -B \
  optimizer-language-research/implementation/minimal-systems-capability/tools/verify_rust_census.py
```

## Inclusion and deduplication rules

The extractor starts at the public `core`, `alloc`, and `std` module indices and
follows their public module tables. For each non-collapsed module it records:

- every rendered public item;
- defining inherent methods and associated functions, including deprecated
  method sections;
- defining inherent associated types and associated constants; and
- required and provided declarations on the defining trait.

Sections labeled as trait implementations are explicitly rejected rather than
bounded by document order. Rustdoc repeats concrete implementations on every
implementer, while the caller protocol is already represented by the trait
declaration. Reexports remain visible as rows, then receive a `duplicate_of`
link when their normalized source declaration is already present.
Canonicalization uses the normalized source anchor, declaration class, and
member name. This handles source ranges rendered differently by `alloc` and
`std`, primitive methods rendered once with `Self` and once with a concrete
type, and methods generated from one source macro.

Stable safe declarations form the caller-contract anchor. Stable unsafe and
unstable declarations remain in the inventory as implementation or future
evidence. A safe signature that exports a raw pointer, `MaybeUninit`, or a leak
obligation is still marked safe at this mechanical stage; the semantic census
must classify its obligation separately.

`core::arch` and `core::intrinsics` are collapsed by module rather than expanded
into tens of thousands of target-specific rows. This is not an exclusion. Every
reachable module has direct stable and unstable item counts plus a digest over
its entries. The 29 collapsed module rows account for 17,424 direct stable and
14,174 direct unstable entries. Their capability disposition is the explicit
target-intrinsic and compiler-runtime family, not the sequential data-structure
floor.

## Exact counts

The detailed inventory contains 17,135 rows:

- 10,267 stable-safe renderings;
- 560 stable-unsafe renderings;
- 6,308 unstable renderings;
- 5,278 canonical stable-safe declarations after reexport/source
  canonicalization; and
- 277 canonical stable-unsafe declarations.

The module inventory has 297 rows, no missing page, and no unresolved external
module link. These counts cover the installed 1.97.0 target's rendered public
surface; platform and architecture families remain explicitly accounted rather
than promoted to portable contracts.

Top-level item tables are consumed one `dt` at a time. Each `dt` must contain
exactly one recognized declaration anchor and may have zero or one adjacent
`dd`; any orphan markup, ambiguous anchor, or unconsumed table content is a
hard extraction failure. The pinned tree has 2,124 entries without a
description: 2,051 in collapsed tables and 73 in detailed tables. This policy
restores five reachable modules that a description-
requiring parser missed: `core::intrinsics::gpu`,
`core::panicking::panic_const`, `std::intrinsics::gpu`,
`std::os::macos::raw`, and `std::os::windows::net`.

Exact canaries cover `std::ascii`, `std::intrinsics`, `SipHasher`, `AsciiExt`,
trait aliases, tuple/unit primitive pages, declaration-less module pages, the
final member section on a page, and the deprecated direct memory intrinsics.
The inventory contains 356 deprecated renderings and 198 canonical stable
deprecated declarations.

An independent seed extractor validates the data-structure-heavy boundary. The
selected array, slice, string, box, sequence, deque, list, heap, ordered and hash
map/set, shared-owner, and dynamic-borrow pages contain 547 page-local
stable-safe and 36 page-local stable-unsafe inherent declarations. Two safe
`Box<[T]>` initialization constructors and one unsafe initialization seal are
rendered on both `Box` and slice pages, leaving exactly 545 canonical safe and
35 canonical unsafe declarations. The verifier pins every per-page count.

## Interpretation limits

This inventory is deliberately broader than the detailed G0 data-structure
closure. It contains numeric convenience methods, OS extensions, I/O, threads,
atomics, async/task, formatting, macros, reflection, and target facilities. The
domain ledger must classify every such family as current detailed scope,
ordinary-library derivation, trusted platform frame, separate later lock,
redundant surface, or owner-ratified non-goal.

Conversely, the inventory is not a completeness oracle. Rust's stable standard
library omits generational pools, arenas, graphs, inline-small sequences,
movable gaps, ECS migration, LRU caches, indexed priority queues, ropes,
intrusive structures, production async runtimes, memory mapping, signals,
terminal control, dynamic loading, and many other systems facilities. Visible
cross-ecosystem witnesses and held-out structures remain mandatory.

The 545 selected safe declarations normalize into a much smaller set of
observable contract clusters. Similar names do not justify merging distinct
ownership, failure, invalidation, order, complexity, address, identity, or
cleanup behavior; many Rust spellings do not justify separate xlang mechanisms.
