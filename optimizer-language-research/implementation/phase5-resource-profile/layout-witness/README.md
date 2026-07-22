# ResourceProfile v1 layout witness

Status: NON-AUTHORITATIVE EVIDENCE FOUNDATION. This crate selects no numerical
maximum, installs no specification, and grants no compiler authority.

This standalone safe-Rust crate implements the host-layout and lifetime-peak
obligations in [`../STORAGE-MODEL.md`](../STORAGE-MODEL.md). It has no registry
dependency. Its three path dependencies let it name public records from the
active v0.9 frontend; they do not make this witness semantic evidence.

## What it establishes

- `charges.rs` copies the exact 25 candidate record charges from the storage
  model and checks every count with checked `u64` multiplication, `usize`,
  `isize::MAX`, and an exact aligned `Layout::array` witness. Zero capacity is
  represented without authorizing an allocator call.
- `layouts.rs` uses compile-time assertions and repeated runtime observations
  for the active public `SourceFile`, duplicate-path order entry, `Lexeme`,
  source-boundary offset, `ClassifiedToken`, `BundleSourceExtent`, and `NodePath` component
  representations. Each check requires actual size at most stride, actual
  alignment at most the ceiling, and stride divisible by actual alignment.
- `host.rs` pins the first candidate compiler-host class to
  `aarch64-apple-darwin`, Rust 1.91.1, the standard-library system-allocator
  path assumption, at least 8 GiB physical memory, a 4 GiB supervised process
  RSS ceiling, and a 3 GiB modeled-process target. The last four facts are
  measurement assumptions, not approved profile values.
- `ledger.rs` defines one canonical binary ledger. It has the twelve required
  lifetime categories and seventeen rows: peak 2 has separate source-binding
  copy, canonical-encode, and canonical-decode rows, while peak 10 has four
  separately mandatory rows for `SameScopeKey`, `RegionOwnerKey`,
  `ArmBinderKey`, and `LookupKey`.
  Every row stores all 25 record capacities and all eight exact byte charges in
  fixed order. The decoder rejects a changed version, identity, row count, row
  order, sort order, malformed RSS tag, truncation, trailing bytes, impossible
  layout, and arithmetic overflow.
- The ledger has a closed `u32` identity-domain receipt for actual sources,
  production nodes, maximum direct production children, mixed elements,
  maximum mixed start/count, tree depth, and format depth. Every value is
  accepted at exactly `u32::MAX` and rejected one above. No resolver dense ID
  representation exists yet; selecting one as `u32` requires a new ledger
  codec version and an added closed domain rather than silent omission.

Run the pinned checks without network access:

```sh
cargo fmt --check
cargo check --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
cargo run --locked --offline
```

The binary exits unsuccessfully if it was not built for the candidate target
with Rust 1.91.1. Its output is a layout observation, not a peak ledger or an
approval packet.

## Byte meanings

For each row, `requested_bytes` is the sum of `capacity * candidate stride`
for all record families plus exact logical-path, source, binding, and spelling
byte-array capacities. `accounted_bytes` adds vector headers, explicit padding,
fixed grammar tables, and profile/capability control records.
`modeled_process_bytes` then adds the separately measured empty-process
baseline.

None of these numbers is allocator consumption or RSS. `try_reserve_exact`
does not promise a byte-exact allocator request, allocator metadata and slack
are not bounded yet, and an in-process reading cannot establish its own
supervised high-water RSS. The ledger therefore carries RSS as a separate
external observation with an explicit `Unmeasured` state. A ledger with
`Unmeasured` RSS remains useful charged-byte evidence but cannot support the
3 GiB or 4 GiB process claims.

The ledger accepts only proposal
`7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d`,
candidate specification
`71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9`,
and storage-model
`ee6e8cd0dd70d81eaa0ca11db4614e3877afce1241e9413a7dd9863aeb4f3139`
SHA-256 identities. It also binds the pre-execution witness executable SHA-256.
Digest mutations, an all-zero executable digest, and empty,
oversized, or non-graphic-ASCII host/toolchain/allocator/supervisor identities
are rejected. The canonical bytes record measured inputs and observations;
they never encode an `approved` flag. Only the owner-gated ResourceProfile
process can approve values derived from evidence.

## Evidence still missing

This external crate cannot name eight private active frontend types. Parser
`Task`, parser `Frame`, `DerivationElement`, finalizer `Completed`, `ShapeTask`,
`NodeRecord`, `TerminalRecord`, and `GapStyle` still require compile-time
assertions inside their owning compiler crate before numerical approval. The
witness reports each as `pending-in-crate-assertion`; it does not substitute a
mirror type or claim its layout.

The C-01 `MixedElement` and all nine resolution families (`Declaration`,
`Scope`, `DeclarationEvent`, `LexicalUse`, `DeferredUse`, `LookupEntry`,
`OrderingScratch`, `CoverageRecord`, and `DiagnosticIssueElement`) do not exist
yet. Their table entries are conservative charge ceilings only. Actual
production types must fit those ceilings before implementation can use the
profile; a mismatch stops for redesign or reapproval.

A complete measurement also still needs:

1. the exact operating-system build, physical memory, allocator identity and
   configuration, toolchain, storage-model, and executable hashes in a
   committed supervisor manifest;
2. real count and capacity rows from both non-sharing source-to-role routes;
3. representative combined allocations for all twelve categories and all
   four sort rows;
4. repeated externally supervised empty-process baseline and high-water RSS
   readings on the pinned host; and
5. a finite allocator-slack argument or measured bound that reconciles charged
   bytes with observed RSS.

Until those exist, the host class, charge table, 3 GiB modeled target, 4 GiB
RSS ceiling, and every eventual ResourceProfile field remain candidates only.
