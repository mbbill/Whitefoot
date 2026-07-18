# E0.1a Flat-record candidate space

Status: superseded non-normative research proposal retained as historical input.
D11 authorizes no further isolated experiment, production implementation,
specification change, wfc migration, scored timing, or default teaching. A
future dense-family Lock A must explicitly retain, revise, or supersede these
alternatives; this document selects none and does not restart automatically.

The first detached prototype is not a valid production candidate because its record
fill duplicates one affine value. Its useful result is narrower: direct record-field
storage can lower without whole-record copies on the frozen 64-bit targets while
unchanged SoA sources retain raw-IR identity.

## Integration rule: no feature flag

No candidate may be introduced behind a checker, compiler, CLI, teaching, or runtime
flag. A flag would make one compiler maintain two language semantics and violate
Whitefoot's single canonical language rule.

Experiments use two isolated toolchains:

- the baseline toolchain is built from the frozen parent revision;
- one disposable candidate branch/worktree implements one proposed semantics
  unconditionally and is never merged merely to run the experiment.

Both compile the unchanged baseline fixtures for the zero-tax identity gate. An
executable candidate lives only in a disposable worktree; its exact reviewed source
must be archived durably. The first rejected candidate is preserved as the
57,547-byte [`DETACHED_CANDIDATE.patch`](DETACHED_CANDIDATE.patch) at commit
`68a55e4`, with the base and SHA-256 recorded in the review. A later production
change would require explicit owner confirmation and atomic updates to the
specification, checker, stage 0, wfc, conformance, teaching, pattern doctrine, and
code-shape pins. Baseline and candidate semantics never coexist in one tool.

## What the prototype did and did not establish

On x86_64 and AArch64, the prototype derived target row strides of 24 and 56 bytes
for the TokenRow and AstRow fixtures. A field-only access lowered to a bounds check,
row GEP, field GEP, and scalar load/store, without aggregate loads or stores,
`memcpy`, `byval`, or `sret`. Existing primitive-buffer and SoA raw LLVM remained
byte-identical in the frozen fixtures.

It did not establish a sound ownership design, target-generic layout, a production
surface, or a performance benefit. In particular:

- `buffer_new<Record>(count, move seed)` evaluated one affine seed and stored it in
  every row;
- the candidate used 64-bit allocation and layout assumptions that are unsound for
  32-bit targets;
- no scored timing exists.

Hostile review also found pre-existing current-language defects. Commit `38d642e`
fixed the first index-atom liveness/readability reproduction; commit `7438e17` closes
the remaining index, match, payload-borrow, try, and return seams. Commit `50a1ddd`
repairs recursive projection
and strict GRAM-9 in both executable frontends. These fixes are independent of E0.1
and are not premises for limiting or selecting the record-storage design.

## Ownership alternatives are distinct

The prior project record carded a *declarative* `copy struct` tier: the author marks
one struct whose fields satisfy Copy, and the checker verifies the declaration. That
is not automatic structural Copy, which would infer Copy for every eligible record.
The two have different source visibility and migration costs:

| Route | Visibility | Ownership and cost consequence |
|---|---|---|
| Automatic structural Copy | No declaration or call-site marker | Every eligible record changes from affine to Copy; bare assignment, arguments, returns, and matches may copy an arbitrarily large value |
| Declarative `copy struct` | One type-declaration marker | Only opted-in types change class, but every use of such a type still copies bare without a call-site cost marker |
| Derived `Flat(T)` storage eligibility | No declaration marker; `buffer<Record>` selects AoS | Records can remain affine, but a separate sound initialization operation is required |
| Explicit Repeat/Clone | Duplication is visible at the operation | Requires new value semantics, conformance/effect rules, and a decision about fallibility, allocation, and implementation cost |

OWN-1's tag-only-enum amendment is a relevant precedent. Resource-free affinity
bought no safety for Bool and other tag-only states, while the forced integer
workaround caused a measured 1.6-1.8x loss. A narrowly defined Flat record has no
direct borrow or drop obligation, so memory safety alone does not force it to stay
affine.

Nominal uniqueness is a separate correctness and maintenance axis. A record with
Copy leaves may represent one authorization state, protocol token, or
private-constructor invariant. Automatic structural Copy would duplicate that state
even though the bit-copy is memory-safe, and it would prevent future modules from
relying on one-value authority. Declarative Copy leaves the decision with the type
author; affine Flat preserves nominal uniqueness by default. Cost visibility is
independently unresolved: a tag is small, while a record can be 24, 56, or many more
bytes. The tag-only precedent therefore requires explicit engagement; it does not
select automatic Copy, declarative Copy, affine Flat storage, or explicit
duplication.

## Conditional Flat predicate

If the owner selects the affine-storage route, a provisional `Flat(T)` predicate
would mean only that the target can statically determine size and layout, and that
the value contains no region, borrow, or drop obligation. It would not imply:

- implicit Copy, explicit Repeat, or Clone;
- all-bit-pattern or zero validity;
- observable padding or bytewise equality, hashing, or serialization;
- stable field offsets, ABI, or FFI compatibility;
- a necessarily cheap whole-record load or store.

The disposable prototype admitted exactly represented integers, tag-only enums, and
recursively eligible non-empty named records. It excluded floats inside records only
because the stage-0 experiment used an intentionally narrow, hand-maintained layout
model and did not trust that path. This is not evidence that `f32` or `f64` is
semantically non-Flat, and existing primitive float buffers remained unchanged.

The same prototype also excluded zero-sized records, payload enums, arrays, buffers,
boxes, arenas, slices and other borrow-bearing fields, cells, sequences,
erased/generic/unknown types, finalizers, and by-value cycles. Those are experiment
limits, not a selected production boundary.

## Fixed-buffer initialization is unresolved

The existing `buffer_new<T>(count, fill)` is sound because v0 requires Copy T and
evaluates `fill` once before repeating it. Merely deriving `Flat(Record)` cannot
relax that precondition. A moved fill contracts one affine value into many; a fresh
outer constructor containing a nested move has the same problem. Explicit `move`
does not grant duplication.

The previously suggested recursively fresh recipe also conflicts with GRAM-9.
Ordinary construct fields are atoms, so a nested construction must be bound by a
preceding `let`. That binding denotes one affine nested record, which cannot then be
repeated into every outer row. A recursive per-slot recipe therefore needs a second
recursive construction spelling and its own evaluation semantics; it cannot be
described as ordinary construction reused for free.

The candidate routes and their preliminary lower-bound costs are below. No row is a
complete META-5 price:

| Route | Benefit | Required price or restriction |
|---|---|---|
| Dedicated recursive recipe | Constructs each nested Flat row independently while records remain affine | Adds recursive recipe grammar, field ordering, evaluation count, effects, parser/AST/checker/diagnostics, and a FORM-1/GRAM-9/META-5 justification |
| Single-level dedicated initializer | Keeps field operands as GRAM-9 atoms and has a smaller surface | Excludes nested record fields or requires source flattening; still adds a dedicated initializer spelling and arity rules |
| Per-slot builder or initialized-prefix owner | Uses ordinary ANF construction and moves each fresh row once | Requires partial-initialization state, inaccessible tail, failure atomicity, and borrow/drop rules, coupling E0.1a to E0.1b |
| Explicit Repeat/Clone | Makes duplication an explicit operation and can reuse the existing fill shape | Broadens semantics beyond storage and must specify whether duplication is bitwise or semantic, fallible, allocating, or effectful |
| Declarative `copy struct` | Reuses existing `buffer_new<T>` and GRAM-9 unchanged | Adds declaration grammar, all-Copy eligibility checking, TYPE-2 aggregate layout/lowering, and ownership-context conformance for assignment/call/return/match; later copies are declaration-visible but not call-site-visible |

Automatic structural Copy has the same initialization convenience as declarative
Copy while removing the declaration-level signal and changing all structurally
eligible records. No route is selected here. Before another isolated prototype, the
owner must freeze one route, its canonical spelling, evaluation count, effects,
eligibility boundary, and META-5 delta.

## Conditional fixed-buffer operations

If an affine Flat-storage route is selected and safe full initialization is solved,
the fixed buffer could remain fully initialized and fixed-length: no capacity field,
growth branch, slot tag, drop bitmap, or hidden header change. A narrow experiment
could then test scalar field projection/update, `len`, existing conservative borrows,
and complete-row replacement from one fresh construction.

A bare whole-record read or `move index<Record>(...)` would remain outside that
narrow experiment because moving an element leaves a hole and treating the read as a
copy changes OWN-1. Padding would have no semantic value. These are conditional
experiment limits, not a final language ruling.

## Required lowering for any record-storage experiment

Target DataLayout must determine size, alignment, stride, and field offsets.
Allocation must check multiplication, alignment rounding, pointer-index bounds, the
allocator's size type, `isize::MAX`, and failure. An expression such as
`index<Row>(rows, i).field` must lower to a bounds check, row GEP, field GEP, and
scalar load/store. Field-only paths may not materialize the record or emit `memcpy`,
`byval`, or `sret`.

Existing primitive-buffer and SoA fixtures must reproduce raw LLVM and native
machine code exactly under the candidate toolchain. This is the zero-tax gate; no
flag is involved.

## Storage and pattern doctrine gates

A builder or opaque initialized-prefix owner cannot be treated as a small follow-on
without confronting STOR-1. STOR-1 currently says growable collections are future
library structures over `buffer<T>` and are not kernel constructs. A kernel
`sequence<T>` would reverse that direction and owes a full META-5 delta covering
tokens, rules, spellings, exceptions, storage, ownership, borrowing, growth,
failure, destruction, diagnostics, teaching, and selection ground. Preserving STOR-1
instead requires identifying the minimal checked operations a library owner needs.
This file selects neither route.

PATTERNS.md is a production and default-teaching gate. P2 currently blesses the
append-only, index-linked SoA pool. Before any AoS record capability enters
production, the closed catalog must give evidence-backed, canonical guidance for
when AoS is blessed and when P2 remains the required shape. Default teaching also
requires the separately authorized benchmark-blind writer panel. Prototype
feasibility alone changes neither P2 nor the teaching pack.

## Decision status

The archived detached implementation is rejected. The field-lowering feasibility
result is retained. D11 supersedes the former direct owner-selection step. The
next state is a separate owner discussion about whether to authorize bounded
G0-Core work; G0-Core and a later dense-family lock do not automatically restart
this proposal. No experiment, production work, specification delta, scored
timing, wfc migration, pattern change, or external model run is authorized by
this document.

The design tree remains unchanged because no production route has been selected.
A later production route that supersedes the recorded declarative-copy path must
move the old path to the appropriate `.alt/` branch with paired reasons in the
same decision; selecting declarative `copy struct` must instead update its card
with the deciding evidence.
