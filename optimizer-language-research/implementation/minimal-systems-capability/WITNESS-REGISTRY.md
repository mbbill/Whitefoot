# Ordinary-Library and Held-Out Witness Registry

Status: G0-Core research registry, 2026-07-14. This file freezes coverage
purposes, observable contracts, family dependencies, held-out budgets, and
anti-special-casing rules. It contains no witness implementation, candidate
mechanism, harness, scored trace, or production authorization.

## 1. Claim being tested

Rust's standard library is a finite demand anchor, not a generativity proof.
Passing only Rust-shaped containers could mean that xlang or a privileged
standard library prebuilt those exact types. The stronger detailed claim is:

> Ordinary no-unsafe xlang libraries can implement the registered sequential,
> unique-owner collection and topology contracts efficiently through the same
> public checked mechanisms available to unrelated libraries.

This is not the complete general-purpose systems-language claim. Concurrency,
shared ownership, resources and FFI, custom allocation, async cancellation,
pinning/address-sensitive values, full text semantics, and target intrinsics
retain separate blocked claims in `SYSTEMS-DOMAIN-LEDGER.md`.

The witness set is deliberately finite. Visible witnesses separate known
topologies and failure obligations. Three training-excluded held-outs test one
direct-storage derivation and two cross-container invariants. A named data
structure need not become a kernel or standard-library type merely because it
is a witness.

## 2. Protected baselines

| ID | Contract | Protection |
|---|---|---|
| B-FIX | Existing fixed, fully initialized Copy buffer | Preserve its two-word owner, one allocation, fixed length, checked indexing, and optimized body. No capacity, occupancy, generation, sharing, or policy field/branch may appear. |
| B-P2 | Existing append-only SoA/index pool | Preserve non-reused indices and the measured append-only access path. No generation, recycling, sparse-state, shared-ownership, or provenance branch may appear. |

These controls run for every family even when that family does not use their
topology. A shared substrate is rejected if its generality becomes a tax on
either protected contract.

## 3. Visible topology witnesses

The mandatory operation canaries in the D11 capability matrix remain in force.
The table below refines the ordinary-library topology witnesses that prevent a
named-container-only result.

| ID | Role | Frozen observable contract | Separating purpose | Capability dependency budget |
|---|:---:|---|---|---|
| W-POOL | W | Insert, get, and remove affine payloads through copyable handles; removed handles cannot revive within the frozen identity horizon; O(1) access/update/removal; exact disposition; steady-live churn has an explicit memory/history bound; identity exhaustion retires or fails rather than wrapping silently. | Distinguishes reusable storage and temporal freshness from an append-only index or a bare-key slab. | `ST-SPARSE`, `OW-INIT`, `OW-MOVEOUT`, `OW-DROP`, `EX-*`, `BR-INVALIDATE`, `FL-*`, `ID-LOGICAL`, `ID-FRESH`, `ID-POOL`, `FT-STATE`, `FT-IDENTITY`, `AB-SEAL`, `AB-GENERIC` |
| W-ARENA | W | Amortized O(1) phase allocation; element identity/address is stable for the phase contract; no individual free; bulk reset/destroy invalidates the phase and disposes every affine payload exactly once, including partial construction and failure. | Bulk phase reclamation differs from per-slot deletion and from Rust bump allocators that intentionally skip payload destruction. | `OW-INIT`, `OW-DROP`, `EX-*`, `BR-PROV`, `FL-*`, `ID-LOGICAL`, `AB-SEAL`, `AB-GENERIC` |
| W-SMALL | W | No heap allocation through inline capacity N; insertion at N+1 performs one sound spill; contiguous slice semantics survive; affine pop/remove/drop work; failed spill preserves the original owner and offered input; automatic spill-back is not required. | Exposes a one-time representation transition and an externally measurable allocation ceiling absent from an ordinary growable sequence contract. | `ST-FULL`, `ST-DENSE`, `ST-HOLE`, `OW-*`, `EX-*`, `BR-*`, `FL-*`, `AB-SEAL`, `AB-GENERIC`, `FT-STATE` |
| W-RECUR | W | Finite layout, unique recursive construction and node extraction, mutable cursor, exact partial-construction cleanup, and bounded-stack destruction for adversarial depth. | Tests unique recursive ownership without importing shared ownership or stable raw addresses. | unique box owner, `OW-MOVEOUT`, `OW-REPLACE`, `OW-DROP`, `EX-*`, `BR-REBORROW`, `BR-RESULT`, `BR-CURSOR`, `FL-*` |
| W-GRAPH | W | Frozen CSR has O(V+E) storage and contiguous edge scans. Dynamic form has stable non-reviving node/edge identities, O(1) lookup, O(local degree) node removal including incident edges, unrelated-handle stability, and O(1) known-neighbor handle-based splice/rewire. | A pool alone does not test referential integrity, cascading mutation, or multi-node repair. Handle-based rewiring is the safe analogue under the current pin/intrusive deferral. | dense sequences, W-POOL, `BR-DISJOINT`, `BR-CURSOR`, `FL-ATOMIC`, `ID-*` except `ID-ADDRESS`, exact cross-container repair |
| W-ECS | W | Two or three fixed archetypes suffice. Entity identity remains stable while adding/removing a component migrates aligned affine columns; swap-removal repairs the displaced entity's reverse location; column scans remain contiguous; no per-entity allocation; failure duplicates or loses no payload. | Existing append-only compiler SoA does not test atomic movement across several aligned buffers plus reverse-index repair. | `ST-DENSE`, `ST-DEPENDENT`, `ST-HOLE`, `OW-SWAP`, `OW-RELOCATE`, `FL-ATOMIC`, `ID-LOGICAL`, `ID-FRESH`, cross-buffer fact invalidation |
| W-GAP | W | Logical sequence content is independent of gap position; insert/delete at the gap are amortized O(1); moving the gap is O(distance); the hole is never readable or droppable as T; growth and recoverable failure preserve the old logical sequence. Use bytes and affine records, not Unicode semantics. | Separates a simultaneous initialized prefix and suffix from a one-prefix owner or arbitrary sparse bitmap, and prices direct bulk movement. | `ST-DENSE`, `ST-HOLE`, `OW-INIT`, `OW-MOVEOUT`, `OW-RELOCATE`, `OW-DROP`, `EX-*`, `FL-*`, `FT-STATE` |

### 3.1 Visible controls and optional compositions

- **O-SLAB:** a bare reusable integer-key slab is a weaker performance/control
  contract. Its stale-key alias risk must be explicit. It cannot discharge
  W-POOL.
- **O-ROPE-UNIQUE:** a uniquely owned rope may be admitted later as composition
  over growable chunks and ordered recursive nodes. O(1) clone or persistent
  snapshots depend on the shared-ownership deferral and are not implied.
- **O-INTRUSIVE:** true pointer-intrusive, self-referential, or multiple-membership
  structures remain scoped to the pin/address family. W-GRAPH's handle-based
  known-neighbor rewiring is required now; it does not claim address pinning.
- **O-LAZY-DRAIN:** the lazy partially consumed form remains optional unless a
  family promotes it. Eager drain/compaction remains mandatory. The lazy form
  cannot inherit correctness from Rust's `Drop` guards without an xlang exit
  proof.

## 4. Exact held-out contracts

The semantic contracts and dependency budgets below are visible and frozen.
Candidate source, tests, traces, and performance observations remain excluded
until Candidate Freeze B. A held-out may not be used for candidate training,
manual tuning, compiler pattern recognition, or threshold selection.

### H-STORE — direct storage substrate: bounded-key sparse set

Caller-visible contract:

- creation fixes a key universe `[0, U)` but constructs no payload value;
- `insert(key, own T)` admits at most one live payload per key and preserves or
  returns the offered owner on duplicate key or recoverable failure;
- `contains`/`get` are O(1); `remove` is O(1) and returns the affine payload;
- live iteration is O(n), visits each current key/value exactly once, and makes
  no stable-order promise across removal;
- there is no per-element heap allocation;
- payload storage is O(capacity), key-position metadata is O(U), and operation
  traces freeze both terms separately; and
- removal, swap-repair, clear, destruction, and every failure path preserve the
  key-to-position/value relation and exact drop.

Allowed dependencies:

`ST-FULL`, `ST-DENSE`, `ST-HOLE`, `OW-INIT`, `OW-MOVEOUT`, `OW-REPLACE`,
`OW-SWAP`, `OW-RELOCATE`, `OW-DROP`, `EX-*`, `BR-PROV`, `BR-DISJOINT`,
`BR-INVALIDATE`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `AB-SEAL`,
`AB-GENERIC`, and `FT-STATE`, plus existing checked scalar/Option operations.

Forbidden dependencies:

- any finished growable sequence, map, set, pool, slab, small-sequence, or ECS
  library;
- a container-specific intrinsic, compiler opcode, recognizable source-name
  rule, or sealed-standard-library raw payload access; and
- shared ownership, pinning, concurrency, custom allocators, or unsafe FFI.

H-STORE therefore tests public checked storage transitions directly. It may use
the project allocator frame but cannot rebuild or expose it.

### H-LRU — composition under two-way coherence

Caller-visible contract:

- fixed positive capacity;
- successful lookup promotes exactly that key to most-recently used;
- insert/update preserves one value per key;
- a full new insertion evicts and returns or destroys exactly the
  least-recently-used value under the frozen API;
- expected O(1) get/insert/remove under the frozen hash/adversary assumptions;
- no per-operation scan of all entries; and
- hash membership, stable identity, linked order, payload ownership, and
  failure behavior remain coherent.

Dependency budget: the selected finished unordered map plus selected recyclable
stable pool/handle-linked ordering and their public APIs. H-LRU may not receive
new raw storage privilege or a compiler-recognized LRU path. It is a composition
witness and cannot substitute for H-STORE.

### H-IPQ — composition under heap/reverse-index coherence

Caller-visible contract:

- keyed items are unique;
- peek is O(1);
- push, pop, keyed removal, and priority change are O(log n);
- every heap exchange atomically repairs the keyed reverse position;
- ownership and failure behavior are exact for affine item/priority payloads;
  and
- no operation repairs coherence by an O(n) search.

Dependency budget: the selected finished unordered map, dense affine sequence,
and priority-queue/heap behavior through public APIs. H-IPQ receives no bespoke
compiler path and cannot substitute for H-STORE.

## 5. Ordinary-library generativity gate

Every W and H witness must satisfy all of the following:

1. Compile as an ordinary external xlang library with the same compiler
   artifact and public capability set used for unrelated programs.
2. Import only its frozen dependency allowlist. No privileged module, hidden
   intrinsic, container-specific opcode, unchecked payload access, or
   standard-library-only transition is legal.
3. Pass negative canaries for inaccessible-state reads, duplicate/drop loss,
   interrupted transitions, stale identity, overlapping mutable places,
   partial construction, and all applicable failure points.
4. Meet frozen asymptotics before timing, then report allocation count,
   initialized/touched/moved bytes, live/high-water/transient memory, metadata,
   checks, branches, code size, and failure injection where applicable.
5. Demonstrate that facts-off changes only retained checks/performance, never
   acceptance or semantics.
6. Reject candidate-specific compiler recognition even if the result is fast.

A standard-library implementation may serve as a reviewed witness of API
sealing, but it cannot alone close W or H. This gate is the difference between
"xlang ships useful containers" and "ordinary libraries can build an unseen
efficient structure."

## 6. Holdout custody and rotation

- G0-Core freezes the contracts, budgets, and exclusion rules above.
- Family Lock A freezes exact test-oracle schemas, payload classes, targets,
  trace families, endpoints, and the custodian responsible for source/tests.
- Before candidate construction, the custodian records hashes of the hidden
  source, tests, trace generator, and sealed inputs outside candidate-visible
  material. Candidate agents receive only the visible contract and allowlist.
- Candidate Freeze B records candidate hashes first; only then may the held-out
  implementation and immutable scoring inputs be disclosed to the scoring
  process.
- Any leak, candidate-specific correction, compiler recognition, or post-freeze
  semantic/gating change compromises the held-out. The family lock reopens and
  the custodian rotates to a new implementation within the same frozen contract
  and dependency budget. The compromised result is diagnostic only.
- H-STORE, H-LRU, and H-IPQ are logically independent. One passing held-out
  cannot compensate for another failure.

No hidden artifact is created by G0-Core; custody begins only under a separately
authorized Family Lock A. This avoids false claims that a plaintext repository
fixture is training-excluded.

## 7. Primary evidence

- Rust's intentionally bounded standard collection set:
  <https://doc.rust-lang.org/1.97.0/std/collections/>
- Generational identity and exhaustion pressure:
  <https://docs.rs/generational-arena/latest/generational_arena/>
- Bare slab key reuse:
  <https://docs.rs/slab/latest/slab/>
- Phase/bump arena behavior:
  <https://docs.rs/bumpalo/latest/bumpalo/>
- Inline-to-heap sequence transition:
  <https://docs.rs/smallvec/latest/smallvec/>
- Stable graph mutation:
  <https://docs.rs/petgraph/latest/petgraph/stable_graph/struct.StableGraph.html>
- ECS table/archetype storage:
  <https://docs.rs/bevy_ecs/latest/bevy_ecs/storage/> and
  <https://docs.rs/bevy/latest/bevy/ecs/archetype/>
- Movable gap behavior:
  <https://www.gnu.org/software/emacs/manual/html_node/elisp/Buffer-Gap.html>
- LRU contract:
  <https://docs.rs/lru/latest/lru/struct.LruCache.html>
- Indexed priority queue contract and costs:
  <https://docs.rs/priority-queue/latest/priority_queue/priority_queue/struct.PriorityQueue.html>
- Address-sensitive/intrusive motivation:
  <https://doc.rust-lang.org/1.97.0/std/pin/> and
  <https://docs.rs/intrusive-collections/latest/intrusive_collections/>
- Rope composition and sharing distinction:
  <https://docs.rs/ropey/latest/ropey/struct.Rope.html> and
  <https://research.google/pubs/ropes-an-alternative-to-strings/>
