# ResourceProfile v1 storage and peak model

Status: NON-AUTHORITATIVE REVIEW CANDIDATE. Strides and host values below are
measurement candidates, not selected hard maxima.

## Supported compiler-host class candidate

The first evidence target is 64-bit `aarch64-apple-darwin`, Rust 1.91.1, the
Rust standard library's system allocator path, at least 8 GiB physical memory,
and a supervised compiler-process RSS ceiling of 4 GiB. The measured combined
peak target is at most 3 GiB, leaving 1 GiB for allocator metadata, code,
stacks, dependencies, and measurement uncertainty. The final host-class
manifest must pin OS/toolchain/allocator evidence and its exact SHA-256.

This host class does not imply a Whitefoot target architecture. Other compiler
hosts require a separately measured host-class profile or proof that their
representation and service bounds are no weaker.

## Candidate record charges

Each family has an identity-bound stride and alignment ceiling. Production
compilation statically requires `size_of::<T>() <= stride`,
`align_of::<T>() <= alignment`, and `stride % align_of::<T>() == 0`. The
ledger charges `count * stride`, never the smaller concrete size. Every
product and sum uses checked `u64` before
`usize`, `isize::MAX`, and `Layout::array` validation.

| family | stride bytes | maximum alignment |
|---|---:|---:|
| SourceFile record | 64 | 16 |
| duplicate-path order entry | 8 | 8 |
| Lexeme | 64 | 16 |
| source-boundary offset | 8 | 8 |
| ClassifiedToken | 96 | 16 |
| parser task | 32 | 16 |
| parser frame | 64 | 16 |
| DerivationElement | 64 | 16 |
| finalizer root | 96 | 16 |
| shape task | 16 | 8 |
| NodeRecord | 128 | 16 |
| MixedElement | 16 | 8 |
| TerminalRecord | 32 | 8 |
| BundleSourceExtent | 24 | 8 |
| canonical gap | 32 | 8 |
| NodePath component | 4 | 4 |
| Declaration | 128 | 16 |
| Scope | 64 | 8 |
| DeclarationEvent | 64 | 8 |
| LexicalUse | 64 | 8 |
| DeferredUse | 64 | 8 |
| LookupEntry | 128 | 16 |
| OrderingScratch entry | 128 | 16 |
| CoverageRecord | 32 | 8 |
| DiagnosticIssueElement | 128 | 16 |

Source, logical-path, binding, and spelling byte arrays charge their exact byte
length. Vector headers, explicit padding, profile/capability scalars, and fixed
tables appear separately in the peak ledger's accounted bytes; they are not
smuggled into a record stride. Allocator metadata and slack remain outside the
charged, accounted, and modeled-process byte totals until a finite bound is
established. They are reconciled only through the separately supervised RSS
and service evidence.

## `max_tree_bytes`

Only these five arrays spend the profile's tree-byte field, in order:

```text
DerivationElement
NodeRecord
MixedElement
TerminalRecord
BundleSourceExtent
```

For each family, `family_bytes = actual_count * approved_stride`. The stage
checks the cumulative charged total before host conversion, layout, or
reservation of that family. This is a cumulative allocation charge; dropping
the derivation later does not refund it. Parser tasks/frames, finalizer roots,
shape tasks, gaps, and NodePath are transient and separately bounded/ledgered.
Source, lexical, and classified records have their own profile counts.

The C-01 topology storage order is `Nodes`, `MixedElements`, `Terminals`, then
`SourceExtents`. `MixedElements` occupies the predecessor ChildEdges limit and
storage position. For each nonzero storage the order is checked count product,
tree-byte limit when applicable, `usize`, `Layout::array`, exact fallible
reserve, then writes. Address failure precedes allocation failure. Zero count
does not call the allocator. There is no ChildEdges allocation.

## Required lifetime peaks

The evidence report calculates every phase below and selects the largest; two
end-state totals are insufficient.

1. SourceBundle construction with source record/path/byte copy plus duplicate-
   order scratch during only the lifetime in which that scratch exists;
2. SourceBinding copy and canonical encode/decode candidates, each with its
   exact retained input lifetime and without assuming that duplicate-order
   scratch remains live;
3. lexer count pass fixed state;
4. lexer emission with source plus growing Lexeme and boundary arrays;
5. classifier with source, complete lexical tape, and growing classified
   arrays;
6. parser with those retained inputs, tasks, frames, and growing derivation;
7. finalizer with the complete derivation retained, roots/shape tasks, and
   growing nodes/mixed/terminals/extents;
8. canonical audit with retained frontend state, gaps, and the largest selected
   NodePath;
9. resolution preflight and all exact reservations;
10. each of the four sorts while both LookupEntry and equal-sized
    OrderingScratch arrays are live;
11. successful retained resolution state; and
12. largest selected diagnostic materialization while all tables required by
    issue construction remain live.

Each row includes vector headers and capacity, alignment padding, fixed grammar
tables, profile/capability records, source bytes, path bytes, binding bytes,
spelling storage, and the process baseline measured on the pinned build. Until
a finite allocator-slack bound is pinned, this is only a charged/requested-byte
model and does not prove the 3 GiB actual-RSS target. Actual high-water RSS is
separate supervisor-classified evidence; allocator or process excess is not
misreported as controlled in-process allocation refusal. `try_reserve_exact`
is never treated as a byte-exact allocator promise.

## Narrow identity domains

The ledger separately checks every actual count stored in `u32`, including
SourceId, NodeId, production-child ordinal, mixed start/count, and any future
resolver dense ID selected as `u32`. A `u64` profile value does not prove that
one of these representations fits. The hard-profile validator includes only
downward-closed representation limits; runtime cross-field relations remain
actual-count receipts.

## Measurement protocol

The dependent v0.9 frontend observer supplies only its labeled predecessor
topology cross-check; it is not either independent route and cannot select a
maximum. Both future independent role/count routes report their complete exact
counts without reading `archive/`. A pinned safe-Rust witness asserts record
sizes/alignments and exercises representative combined peaks under the host
supervisor. Generated sources, parameter manifests, reports, executable hash,
toolchain identity, repeated high-water RSS, and timing samples are all
committed. Production implementation must later repeat the measurement and
counter differential; a larger record, peak, or work count stops for review.
