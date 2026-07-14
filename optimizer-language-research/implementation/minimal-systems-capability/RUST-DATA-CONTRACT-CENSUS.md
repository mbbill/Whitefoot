# Rust 1.97.0 Data-Contract Census

Status: manual semantic normalization evidence, 2026-07-14. This census selects
no xlang syntax, storage representation, privileged transition, standard
container, optimizer fact, or production mechanism.

## Scope and provenance

`RUST-DATA-CONTRACT-CENSUS.tsv` normalizes the selected stable Rust 1.97.0
data-structure surface into caller-observable contracts. The source snapshot is
the official `1.97.0` tag, annotated tag object
`eca4cdea45792600b4275e9d4c64fd827d575a24`, peeled commit
`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`, and tree
`90d170433978b46bb035bb89f8a7bfa41c5d665f`. Exact raw declarations and source
anchors remain in `RUST-1.97.0-API-INVENTORY.tsv`; the generated inventory and
module digests remain pinned by `RUST-1.97.0-CENSUS-MANIFEST.json`.

`RUST-DATA-SURFACE-MAP.tsv` is the exact declaration crosswalk. It contains one
row for each canonical stable-safe seed declaration, keyed by the raw
inventory's canonical declaration identity and source anchor, and names exactly
one primary contract row. Cross-cutting allocation, initialization, helper, and
raw-obligation markers remain annotations rather than competing primary rows.

The seed boundary is arrays, slices, `str`, `Box`, `Vec`, `VecDeque`,
`LinkedList`, `BinaryHeap`, B-tree and hash maps/sets, `String`, `Rc`, `Weak`,
and `RefCell`. Its official rustdoc pages contain 547 stable-safe and 36
stable-unsafe inherent renderings. Reexport/source canonicalization removes two
safe and one unsafe duplicate `Box<[T]>` renderings, leaving exactly **545
stable-safe and 35 stable-unsafe declarations**.

The table also accounts for selected direct trait entrances, `mem::{swap,
replace,take}`, allocation failure, and **118 named one-hop helpers**:

| Helper family | Count | Boundary |
|---|---:|---|
| slice | 26 | disjoint-access error; iterators; windows; chunks; reverse chunks; split families |
| `str` | 24 | byte/scalar iterators; UTF-8 error/chunks; lines; matches; split and escape families |
| array | 1 | owning iterator |
| `Vec`, `VecDeque`, `LinkedList`, `BinaryHeap` | 16 | four helpers each, including owning/lazy iterators and mutable heap peek |
| `BTreeMap`, `BTreeSet` | 22 | ordered iterators/ranges, extraction, set algebra, and map entry guards |
| `HashMap`, `HashSet` | 21 | sparse iterators, drain/extraction, set algebra, and map entry guards |
| `String` | 3 | drain and UTF conversion errors |
| `RefCell` | 4 | shared/unique guards and borrow errors |
| allocation | 1 | `TryReserveError` |

Common wrappers such as `Option`, `Result`, `Range`, `Pin`, `Cow`, and
`RandomState` terminate the one-hop traversal. Iterator `Item` types are not
recursively expanded. A protocol sum such as `Entry` includes its occupied and
vacant guards in the same one-hop family.

## Normalization rules

Each row has the charter form:

```text
pre-state + input ownership + behavior parameters
    -> post-state + result ownership + failure/destruction effects
```

Rust-surface brace notation is exhaustive shorthand: for example,
`Vec::{pop,remove}` expands to both declarations, and
`{btree_map,hash_map}::OccupiedEntry::{get,remove}` expands over both helper
families. Every one of the 545 canonical stable-safe seed declarations appears
in a primary family row or a separately marked raw-obligation row. The 35
stable-unsafe declarations appear only in `RAW-UNSAFE-*` evidence rows.
Cross-cutting `TRAIT-*` and `ALLOC-*` rows summarize protocols already used by
family rows; they do not create a second primary classification.

Run the exact coverage gate with:

```sh
python3 -B \
  optimizer-language-research/implementation/minimal-systems-capability/tools/verify_rust_data_contract_census.py
```

The verifier fails on any omitted or repeated canonical declaration, extra
mapping, changed source identity, unknown contract ID or marker, empty required
field, cross-cutting row used as a primary classification, or unaccounted
stable-unsafe seed declaration.

Merging is allowed only for aliases or convenience spellings with the same
ownership, failure, invalidation, order, complexity, identity, and cleanup
contract. The table therefore keeps separate, among others:

- trapping and recoverable access or reservation;
- shared, unique, and owning traversal;
- ordered removal and unordered swap removal;
- stable sort, unstable sort, and nth selection;
- eager mutation and lazy cursors with abandonment behavior;
- sorted, heap-internal, sequence, and arbitrary hash iteration order;
- deep clone and shared-identity clone; and
- stable contract bounds, conditional expected bounds, and explicitly current
  Rust implementation costs.

`implementation_privilege_evidence` records what Rust uses or exposes; it is
not a proposed xlang mechanism. In particular, safe Rust surfaces may rely on
raw relocation, private uninitialized or sparse state, guard `Drop`, and unsafe
reconstruction. A safe call that exports a pointer, `MaybeUninit`, or a leak
obligation is classified separately from an ordinary safe-library derivation.

## Current-xlang status

The status vocabulary is inherited from the owner-reviewed capability report:

- **E** — directly expressible now with the required contract and no known
  structural tax;
- **P** — established blessed pattern with correctness and performance
  evidence;
- **U** — plausible workaround that still lacks a complete proof or cost
  result; and
- **X** — current gap, asymptotic loss, forbidden privilege, or necessarily
  pathological simulation.

Statuses are deliberately contract-local. Current xlang directly supports
checked metadata/index operations and some in-place operations for fixed,
fully initialized Copy buffers; that does not promote a growable affine
sequence contract. The measured append-only SoA pool remains a protected P
baseline, not evidence for deletion, recycling, shared identity, or keyed
storage. Validated UTF-8 ownership, general affine containers, lazy cleanup
guards, shared ownership, dynamic borrowing, and safe ordinary-library
occupancy transitions remain X or explicitly staged outside the current
language.

## Load-bearing findings

1. Lazy `drain`, `splice`, extraction, mutable heap peek, and dynamic-borrow
   guards are cleanup protocols, not iterator convenience. Rust repairs private
   transient state in `Drop`; xlang cannot rely on a writer remembering a
   `finish` call or on an affine token that may be abandoned.
2. Rust's stable recoverable allocation surface is incomplete. Ten stable
   `try_reserve` methods cover six families, but `TryReserveError::kind` remains
   unstable, most fallible constructors remain unstable, and ordinary
   allocating methods may enter the divergent OOM handler. Capacity overflow,
   allocator failure, OOM, input ownership, and rollback therefore need an
   independent xlang contract.
3. Sparse metadata and dynamic borrow flags are optimizer fact channels. A
   control byte or borrow flag authorizes payload access only while every
   mutation and invalidation rule preserves the relation. Green functional
   tests are not hostile fact review.
4. Whole-place `swap`, `replace`, and `take` are foundational ownership
   transitions. Rust's `replace` implementation explicitly avoids the extra
   movement of a swap-based simulation; these rows must not collapse into Copy
   assignment.
5. Rust API presence is demand evidence, not a completeness or mechanism
   oracle. Rust std omits generational pools, graphs, LRU caches, indexed
   priority queues, inline-small sequences, ropes, and other required
   generativity witnesses, while its safe library implementation has raw and
   destructor privileges unavailable to an ordinary xlang library.

The TSV is a G0-Core accounting input. A contract becomes covered only after a
later derivation ledger supplies an ordinary checked-library witness, normal
exit and abandonment proof, asymptotic and structural cost account, fact and
invalidation ledger, negative soundness canaries, and the applicable family
lock evidence. Nothing in this census authorizes that later work.

## Primary references

- <https://doc.rust-lang.org/1.97.0/std/collections/index.html>
- <https://doc.rust-lang.org/1.97.0/std/primitive.slice.html>
- <https://doc.rust-lang.org/1.97.0/std/primitive.str.html>
- <https://doc.rust-lang.org/1.97.0/std/vec/struct.Vec.html>
- <https://github.com/rust-lang/rust/tree/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library>
- `G0-CORE-CHARTER.md`, `RUST-CENSUS-NOTES.md`, and
  `general-purpose-data-structure-capability-RESEARCH.md`
