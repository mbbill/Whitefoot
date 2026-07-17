# General-Purpose Data-Structure Capability Floor

Status: owner-reviewed non-normative research boundary, 2026-07-13. D11
ratifies eventual general-purpose data-structure capability and staged
family-by-family closure. This report authorizes no candidate or production
implementation, specification change, xlc migration, scored performance run,
default teaching change, or external model disclosure.

## 0. Executive finding

xlang has not yet established the data-structure capability floor required of a
general-purpose systems language. The current language can efficiently express
fixed, fully initialized buffers of Copy elements and the measured append-only
struct-of-arrays pool pattern. It cannot currently express a general growable
sequence of affine elements, an efficient deletion-capable keyed collection, or
a stable-identity collection that reclaims slots. Several standard collection
operations are inexpressible; others can be simulated only by initializing
unused capacity, adding a tag to every payload, allocating every element
separately, rebuilding an entire structure, or retaining storage forever.
Existence of those workarounds is not capability evidence: some lose a required complexity or
ownership property, while others, notably fully initialized tagged slots,
remain unmeasured candidates. Each must pass the project's COMPLETE and
EFFICIENT pattern tests rather than be accepted by construction.

The immediate lesson from the E0.1 discussion is broader than “a builder needs
an uninitialized state.” A vector has one initialized prefix, but a hash table,
recyclable pool, ring buffer, and an in-progress compaction have different live
state topologies. Even a finished prefix sequence normally uses transient
source/destination prefixes or a temporary hole to implement growth, ordered
insert/remove, drain, and stable compaction with optimal element traffic. The
tail or hole must never be a writer-readable `T`, but the implementation must
still represent and prove its state.

Under the standing owner instruction to stop implementation and research first,
the prior paired E0.1 ownership protocol remains paused before Lock A. This
report neither creates nor lifts that pause. D11 makes G0-Core plus the dense
family Lock A necessary before any later owner-authorized reopening; those
gates may instead supersede the old paired protocol.
Automatic structural Copy, declarative `copyable struct`, and an affine fixed
builder answer only a fixed-record initialization question. None provides
unknown-length construction, growth, deletion, move-out, sparse occupancy, or
stable slot reuse. No candidate implementation is authorized by this report.

R3 does not require one universal container. A contiguous sequence, a deque, a
hash map, a sorted map, and a stable-handle pool make incompatible guarantees
and have different lower bounds. R3 should select one canonical mechanism per
frozen semantic and performance contract. The contract identity must include
observable results, ownership and invalidation, failure and drop behavior,
asymptotic bounds, contiguity/stability/order guarantees, and enforceable
allocation or memory ceilings. Private layout, algorithm, and dispatch strategy
remain competing implementations. Calling every
unsupported operation a different “non-goal” would violate R2 and D6 rather
than satisfy R3.

The leading research direction is not public uninitialized memory and not a
large menu of interchangeable containers. It is:

1. freeze the global capability boundary and cross-family laws in a bounded
   G0-Core, without prebuilding every future experiment;
2. preregister exact semantics, candidates, workloads, thresholds, and proofs
   only when each family reaches its own Lock A;
3. identify and select the smallest checked storage-state and ownership
   transitions that close that family without pathological costs;
4. keep the current fixed buffer and append-only SoA path free of growth,
   generation, and sparse-occupancy taxes;
5. permit a closed family to seek separate owner-approved adoption without
   claiming that unrelated families or a shared universal substrate are solved;
   and
6. require hostile soundness and machine-code review before any state relation
   becomes an optimizer fact.

## 1. Why this information surfaced late

This is a process failure, not a newly invented requirement.

- `optimizer-language-research/notes/pre-finalization-open-questions.md`
  already called runtime-sized allocation and dynamic collections the largest
  expressiveness gap, and separately recorded missing graph, replace/swap,
  cursor, and borrowed-aggregate capabilities.
- STOR-1 says growable and keyed collections will be libraries over
  `buffer<T>`, but the specification never established the safe operations a
  library needs to implement partial initialization, relocation, deletion, or
  exact-once destruction.
- `PATTERNS.md` requires the closed catalog to be COMPLETE and EFFICIENT, while
  its known-gaps section already records mutation-during-traversal and stored
  borrows.
- The earlier pool/handle work recorded append-only indices as an efficient
  graph route, but its deletion/reclamation census never ran.
- The E0.1 protocol deliberately narrowed itself to fixed record storage and a
  full-initialization transient builder. Its hostile reviews checked that local
  protocol for ownership, benchmark, and repository consistency; they did not
  first run a general collection-operation census. A locally consistent
  protocol consequently survived while a mandatory upstream completeness
  question remained unanswered.

The repair is procedural as well as technical: every future expressiveness
proposal must pass a capability-floor matrix before R3 narrows the candidate
mechanism. Review scopes must include unsupported neighboring operations and
not merely attacks inside the proposal's chosen API.

## 2. Scope and acceptance vocabulary

This report is about data-structure expressiveness and its direct language
substrate. It does not claim to finish general-purpose I/O, concurrency,
reflection, Unicode, shared ownership, or custom allocator design.

“General-purpose” does not mean that every named data structure is a kernel
type or even a standard-library type. It means that:

- representative standard container families have one taught, efficient route;
- an ordinary library can implement an unseen specialized structure using
  public checked mechanisms, without writer-visible unsafe or per-structure
  compiler magic;
- omission of a redundant surface form is acceptable, while omission of a
  semantic operation or a performance class is a recorded failure;
- an alternative blessed pattern counts only after correctness and performance
  evidence, not because a simulation is theoretically possible.

The audit uses four status classes:

- **E — direct:** expressible by the current language with the required
  semantics and no known structural performance tax.
- **P — proved pattern:** expressible through a blessed alternative with local
  correctness and performance evidence for the stated workload.
- **U — unproved workaround:** plausible, but it adds an unmeasured tax, loses a
  guarantee, or lacks a complete ownership proof.
- **X — gap:** inexpressible, asymptotically worse, or necessarily pathological
  under the current rules.

A family cannot enter production R3 selection while one of its mandatory
operations remains X. A U must be measured into P or the family substrate must
change. Unrelated unresolved families still block the complete-floor claim, not
that local selection.

## 3. Current xlang evidence

### 3.1 Specification boundary

The current v0.6 specification establishes:

- `buffer<T>` is an affine, fixed-length, fully initialized, two-word owner;
- buffer and array elements are Copy-only;
- `buffer_new<T>(n, value)` repeats one Copy value over all `n` slots;
- a partial move kills the whole root binding, and a dead binding cannot be
  reinitialized;
- there is no sanctioned move-out, atomic replace, or swap for general affine
  elements;
- uniq borrows are consumed at calls; reborrow-through-holder and result
  reborrows are recorded but unimplemented;
- contracts exist in the grammar, but a complete callable static member path is
  not yet established;
- there are no modules, private representation, inherent implementation blocks,
  borrowed aggregates, function values, or dynamic dispatch;
- allocation is available for a box, arena value, or fully initialized fixed
  buffer, but not for an initialized prefix or arbitrary occupied slots.

STOR-1's statement that collections are future libraries is therefore a design
direction, not evidence that the libraries are implementable.

### 3.2 Production-compiler workload

The current xlc bootstrap deliberately uses fixed-capacity SoA tapes. That was a
valid bootstrap decision because source length bounds the token and AST counts;
it is not a general-purpose capability result.

The compiler already demonstrates why an exact-fill write-only builder is
insufficient:

- parser and symbol tables append one logical row at a time;
- the final token, AST, symbol, type, and fact counts are data-dependent;
- the parser repeatedly reads earlier rows and backpatches child links and end
  positions;
- retained frontend storage resets logical counts and reuses capacity;
- batch semantic work reserves capacity, writes rows, and commits the new count
  only after the operation succeeds.

For the frozen baseline input, token and AST live prefixes use only about 20.54%
and 10.26% of their full `source_len + 1` capacities. That is static evidence
that initialized-prefix storage is relevant; it is not a timing result and does
not imply that xlc should abandon SoA.

## 4. Representative capability matrix

The table intentionally audits operations, not just type names. Its role column
prevents an existence witness from silently becoming a required standard-library
API or benchmark weight:

- **B — protected baseline:** already selected and must not regress;
- **M — mandatory floor:** every X or U assigned to a family must close before
  that family's R3 selection, and all must close before the complete-floor
  claim;
- **W — substrate/topology witness:** must be implementable efficiently by an
  ordinary library before the implicated family closes, but need not become a
  default container;
- **H — held-out anti-special-casing witness:** frozen before candidate work and
  excluded from training/tuning; each blocks its implicated family and all
  block the complete-floor claim; and
- **O — prevalence-gated optional:** admitted only if a later corpus justifies a
  standard default or if another mandatory contract requires it.

| Workload or operation | Role | Canary | Required efficient property | Status | Current workaround or blocker |
|---|:---:|---|---|:---:|---|
| Fixed buffer of Copy scalars | B | C-FIX | Full initialization, checked index, one allocation | E | Current `buffer<T>` |
| Fixed AoS record buffer | M | C-FIX | Contiguous records and scalar field access | X | Record elements are not admitted |
| Unknown-length append | M | C-SEQ | Initialized prefix, capacity, reserve/grow | X | Prefill worst-case capacity and maintain an unrelated count |
| Append affine value | M | C-SEQ | Move once into tail; rejected push returns ownership | X | Buffers are Copy-only |
| Grow/shrink contiguous sequence | M | C-SEQ | Failure-atomic allocation and direct relocation | X | Cannot replace backing or suppress moved-from drops |
| Pop affine value | M | C-SEQ | Move last value out and shorten live prefix | X | No affine elements or move-out transition |
| Ordered insert/remove and unordered `swap_remove` | M | C-SEQ | Frozen O(n-i) ordered and O(1) unordered contracts | X | No affine move-out; swap simulation can add traffic |
| Swap two dynamic elements | M | C-SEQ | Runtime distinctness plus two exclusive places | X | OWN-7 conservatively overlaps dynamic indexed places |
| Clear/truncate | M | C-SEQ | Drop exactly the removed initialized range | U | Copy tapes can reset a count, but the physical buffer remains fully live |
| Deep clone and bulk move-append | M | C-SEQ | Explicit duplication versus ownership transfer | X | Copy-only loops cover only narrow concrete cases |
| Stable retain and eager drain/splice | M | C-COMPACT | One-pass compaction with exact transient ownership | X | Full rebuild changes allocation and traffic |
| Lazy drain cursor | O | C-LAZY-DRAIN | Partial consumption with sound cursor drop/finish | X | Current no-finalizer rule leaves cleanup unproved |
| Generic unstable and stable sort | M | C-SORT | Static comparator, affine swap, and scratch cleanup | X | No general comparator or affine scratch state |
| Stack adapter | M | C-SEQ | Dense-sequence push/pop with no extra storage tax | X | Fixed Copy-only stack can be hand written |
| FIFO queue/deque | M | C-DEQUE | Amortized O(1) operations at both ends under bounded memory | X | A simple prefix makes front removal O(n) |
| Priority queue | M | C-HEAP | Sequence-backed push/pop/repair under a frozen ordering contract | X | Fixed concrete Copy-only heap is possible |
| Hash map/set | M | C-HASH | Expected O(1) under frozen hash/adversary assumptions | X | Sparse affine occupancy and move-out are absent |
| Ordered map/set | M | C-ORDER | Comparison-model O(log n), ordered range, split/merge | X | No affine node storage or callable comparator |
| Append-only AST/DAG/graph | B | C-P2 | Contiguous SoA columns and non-reused indices | P | P2; measured for the binary-trees/IR-like shape |
| Recyclable stable pool | W | C-POOL | Frozen reuse, identity, exhaustion, and memory contract | X | Census-gated; finite copied identities cannot provide every desired guarantee together |
| Frozen graph | W | C-GRAPH | CSR offsets and edge targets | U | No general library witness exists |
| Dynamic graph with deletion | W | C-GRAPH | Stable identity, reuse, and multi-node mutation | X | Append-only storage never reclaims slots |
| Singly owned recursive list | W | C-RECUR | Finite layout, box extraction, mutable cursor, bounded drop | X | Replace/reborrow and recursive drop rules are absent |
| Doubly linked/cyclic list | W | C-GRAPH | Non-owning stable links and O(1) known-neighbor rewiring | X | Unique recursive ownership cannot encode both directions |
| Homogeneous bump arena/slab | W | C-ARENA | Stable address, bulk reset, and exact payload drop | X | No general multi-value affine storage witness exists |
| Inline-to-heap small sequence | W | C-SMALL | No heap below threshold and sound one-time representation transition | X | No generic affine sequence or transition witness exists |
| Unseen storage-bearing structure | H | H-STORE | Direct use of public storage-state transitions under a frozen dependency budget | X | Must not compose the corresponding finished container |
| LRU cache | H | H-LRU | Hash lookup plus stable O(1) known-node deletion/move | X | Exercises keyed storage, handles, rewiring, and reclamation |
| Indexed priority queue | H | H-IPQ | Map from key to heap position plus affine heap repair | X | Ordinary-library anti-special-casing witness |
| Bytes and UTF-8 text builder | M | C-TEXT | Bulk append, validity sealing, and boundary-safe edits | X | Current byte buffer is fixed and fully initialized |
| Borrowed, uniq, and owning iteration | M | C-ITER | Provenance-preserving borrow or affine cursor | X | Index loops cover only concrete contiguous cases |

Primary standard-library APIs prove that these operations exist and specify
their contracts; they do not determine prevalence or benchmark weight. G0-Core
keeps frequency-independent W/H topology tests separate from corpus-supported
M/O default-library decisions; each family lock later freezes its exact census
and weights.

## 5. Why one universal representation cannot be optimal

The relevant conflicts are algorithmic, not language fashion.

| Contract A | Contract B | Why one zero-tax representation cannot promise both generally |
|---|---|---|
| Contiguous random access and scan | O(1) splice at already-known positions with stable element address | Contiguous insertion shifts values; stable nodes require indirection or non-moving storage |
| O(1) amortized append at one end | O(1) amortized operations at both ends under arbitrary bounded-memory traces | A simple prefix has no space before index zero; ring, segmented, or centered/sliding candidates price that trace differently |
| Expected O(1) unordered lookup under frozen hash/adversary assumptions | Comparison-model sorted iteration and range query | Hash probing and comparison-tree order have different conditional bounds and layouts |
| Dense iteration | Stable deletion with slot reuse | Dense compaction changes indices; stable reuse needs fresh identity, stable indirection, or static revocation |
| One-word append-only handle | Recyclable identity safe against stale copied handles | Reuse needs fresh identity or proof that every old handle is revoked; no particular representation is implied |
| AoS whole-row locality | SoA single-field locality/vectorization | The same bytes cannot be simultaneously contiguous by row and by every column |
| Deep independent clone | O(1) shared snapshot | O(1) clone requires sharing/RC or persistence; independent ownership must copy reachable state |

Rust's own collection cost table gives the concrete sequence example: `Vec`
has O(n-i) insertion/removal, `VecDeque` reduces that to O(min(i,n-i)), and a
linked list has different cursor/splice behavior and no constant-time indexed
access. Abseil makes the same trade explicit for associative
containers: flat Swiss tables sacrifice pointer stability for locality, while
node tables pay indirection for stable addresses; cache-dense B-trees move
values and invalidate pointers to reduce height and memory.

The R3 consequence is narrow: do not offer multiple spellings or representations
with indistinguishable contracts. Do offer a distinct canonical route when the
observable guarantee or asymptotic class differs. “Performance contract” must
not become an unlimited escape hatch; Section 10 freezes its dimensions before
selection.

## 6. Candidate canonical families, not selected language types

The following is a coverage decomposition for research. Names and kernel/library
placement are deliberately unfrozen.

1. **Fixed initialized buffer.** Keep the present two-word, fixed-length owner.
   It pays no capacity field, growth branch, sparse metadata, or generation
   check.
2. **Dense owning sequence.** Contiguous `[0,len)` values with inaccessible
   spare capacity; append/pop/index/slice, failure-atomic reserve/grow, direct
   relocation, insert/remove, truncate, and explicit clone.
3. **Deque.** The distinct both-ends contract. Ring, segmented, and
   centered/sliding contiguous representations remain candidates. Stack is a
   dense-sequence use; queue is a deque use, not two additional storage types.
4. **Unordered association.** An expected-O(1) map/set contract under frozen
   hash and adversary assumptions, with no pointer-stability promise. A flat
   SIMD-probed table is the leading reference candidate, not a selected
   implementation. Set should reuse the eventual map machinery rather than
   create a second storage substrate.
5. **Ordered association.** A comparison-model ordered-range map/set contract.
   Cache-sized B-tree/B+tree storage is the leading reference candidate, not a
   selected implementation. It is not an alternative spelling of the hash map.
6. **Priority queue.** A sequence-backed priority contract. Binary, d-ary,
   min-max, and other heap organizations remain candidates until the heap-family
   Lock A freezes the required operations; this family needs no new raw storage
   topology.
7. **Append-only indexed pool.** Preserve the P2 phase-scoped, non-recycling
   route with one-word typed handles and no generation tax.
8. **Recyclable stable pool.** A census-gated deletion/reuse admission candidate
   with a contract distinct from append-only identity. Generational or otherwise
   fresh identities require a dynamic check unless all old copies can be
   statically revoked.
9. **Bytes and text.** Reuse dense byte storage, but keep byte/text validity and
   indexing semantics distinct. UTF-8 scalar ordinal access cannot promise O(1)
   without an auxiliary index.

Linked lists, graphs, gap buffers, ropes, LRU caches, indexed heaps, tries, and
specialized trees are initially ordinary-library validation targets rather than
automatic kernel types. A language that can only ship pre-blessed compiler
containers but cannot implement these held-out structures has not met the
general-purpose test.

### 6.1 Linked structures specifically

A unique recursive box chain can model an acyclic singly owned list, but it is
not a complete list substrate. Pop and splice require box extraction and atomic
replace; mutation during traversal requires reborrow/cursor support; every node
is a separate allocation. A doubly linked list cannot give both `next` and
`prev` unique ownership.

One pool owner plus non-owning handles can represent singly linked, doubly
linked, and cyclic structures. A cursor should normally carry a handle rather
than a long-lived element borrow. With neighboring handles already known,
insert/delete/rewire can be O(1). If handles escape and slots recycle, they need
generations or an equivalent fresh identity. If the representation is sealed
and no handle can outlive the relevant exclusive cursor, static revocation may
avoid that check. Both cases require evidence; an append-only pool is not a
deletion-capable answer.

An intrusive `container_of` list should not become writer-visible. A sealed
pool with external link columns provides the safe analogue without pointer
arithmetic or self-referential addresses.

## 7. Required coverage contracts for the safe substrate

Evidence establishes different state topologies and proof obligations. It does
not yet prove how many language primitives or surface types are required. The
terms below are coverage contracts, not selected syntax.

### 7.1 Dense initialized state

The public steady-state invariant is:

```text
[0, len)           initialized, readable, and dropped
[len, capacity)    not a T, inaccessible, and not dropped
```

Required transitions include:

- reserve capacity without constructing payload values;
- append by moving one value exactly once;
- return ownership of a value rejected by a no-grow push;
- pop/truncate/clear with exact-once drop;
- atomically replace one live value and return the old value;
- swap distinct dynamic positions after checked distinctness;
- grow by relocating the live prefix, preserving the original on allocation
  failure, and raw-freeing rather than dropping moved-from slots;
- expose shared/uniq slices only over the live prefix;
- invalidate element/slice borrows across structural mutation.

Initialized-prefix steady state is semantically sufficient for append, pop,
clear, clone, eager insert/remove, retain, grow, and swap-only sort. It is not by
itself a constant-optimal implementation mechanism. Adjacent-swap insert/remove
can roughly double compulsory element traffic relative to one-way relocation;
pop-to-new-plus-reverse growth can likewise move elements about twice as much as
direct relocation. Lazy drain also leaves a partially drained owner whose drop
path must restore or finish ownership.

The design must therefore price either:

- a small set of opaque high-level relocation/compaction operations;
- a compiler-checked linear rebuild transaction with source and destination
  ownership; or
- another mechanism that proves transient holes and partial completion without
  making them writer-readable values.

An affine rebuild or cursor token is not sufficient merely because it cannot be
copied. Affine values may be dropped, and current STOR-3 provides no user
finalizers. The baseline acceptance rule is therefore that every non-trap
escape or token drop already leaves a valid owner; a hole-bearing owner that
relies on the writer eventually calling `finish` is rejected. A candidate that
repairs or finishes on token drop must explicitly price a new kernel
derived-cleanup/finalizer rule and its exact exit semantics. Ordinary checked
library code cannot assume that behavior.

### 7.2 Sparse initialized state

A hash table or recyclable pool has an arbitrary live set, not one prefix:

```text
occupied = {1, 4, 9, 17, ...}
```

Deletion, tombstones, free lists, and rehash make this a distinct coverage
contract. A Swiss-style map uses compact control bytes to encode Empty, Deleted,
or Full plus a hash fingerprint, and scans them with SIMD. If a Full control byte
authorizes reading a separate payload slot, their coherence becomes a
load-bearing fact channel. It must remain unforgeable through insert, remove,
retain, rehash, early return, allocation failure, and destruction.
Changing either control or payload state must invalidate every proof derived
from their old relation. SIMD probing must not speculatively read payload lanes
whose control state has not established `Full`, even if masking would discard
the loaded values later.

Possible implementation directions remain open:

- a fully initialized tagged slot, if measurement shows no pathological tag,
  initialization, scan, drop, or affine-move cost;
- one sealed generic abstraction that atomically owns control plus payload
  state;
- a machine-checked relation between metadata and payload initialization;
- a more general generative storage-state mechanism that covers dense and
  sparse cases without charging dense sequences sparse metadata.

The evidence does **not** yet justify two distinct public primitives. It does
justify two mandatory proof and performance contracts. Exposing separately
mutable control bytes and uninitialized payload storage is not acceptable.

### 7.3 Relocation, destruction, and failure

For arbitrary affine and resource-owning `T`:

- relocation transfers one ownership and leaves no value to drop at the source;
- there is no unguarded general `take` that returns a value while leaving a
  writer-readable hole; ordinary ownership leaves storage through a complete
  state transition such as pop, remove, `Full -> Vacant`, or
  `Option.Some -> Option.None`;
- cloning is a separate explicit semantic duplication contract;
- Copy does not imply cheapness, and relocatability does not imply Copy;
- every initialized value is dropped exactly once on each non-trap exit that
  removes its owner: fallthrough, `return`, `break`, `give`, or `try`
  propagation;
- partially built destinations drop only their initialized values;
- destruction decides what to drop only from kernel-owned initialized-prefix or
  `Full` state, never from writer-forgeable length or occupancy metadata;
- capacity arithmetic is checked;
- a recoverable growth path completes capacity arithmetic and allocation before
  moving elements, and preserves the original owner, its elements, and any
  offered affine input on failure;
- early `Result`, callback failure if admitted, and partially consumed cursors
  preserve a valid owner;
- under the current T-A region model, replace and swap experiments initially
  require region-free, borrow-free `T`; admitting borrowed payloads requires a
  separate lifetime proof rather than an implicit generalization;
- recursive indirection must have finite layout and a terminating derived drop
  rule; inline infinite-size cycles must be rejected deterministically.

G0-Core must assign storage and relocation of borrow-bearing payloads either to
an explicit M/W obligation with provenance and invalidation canaries or to a
scoped deferral whose excluded claim is named. Until that disposition closes, a
region-free, borrow-free result cannot support a broader family or complete-floor
claim.

A scoped take guard or linear transaction remains an eligible experiment if it
owns the hole, exposes no uninitialized payload, and every non-trap exit either
restores a value or completes a valid state transition. It is not equivalent to
a standalone writer-visible `take`.

EFF-4 traps abort and run no cleanup. A trap path therefore owes neither drop
execution nor a recoverable post-state; it must still never read uninitialized
memory or perform any other undefined behavior before abort.

Current OP-9 classifies OOM as a TCB-level condition rather than a recoverable
language result. The registry must distinguish checked size/capacity failure,
the current OOM envelope, and any separately proposed recoverable allocator
contract; it may not silently score them as the same behavior.
Any candidate that proposes recoverable allocation must survive allocator fault
injection at every growth, clone, rehash, and node-split allocation boundary.

### 7.4 Access, cursors, and multi-place mutation

Efficient containers need at least one checked route for:

- repeated calls through one uniq owner without consuming it permanently;
- a returned element borrow whose provenance remains tied to the container;
- two or more runtime-distinct mutable elements for swap, graph rewiring, and
  heap repair;
- entry/cursor state that cannot escape its owner or survive invalidating
  mutation;
- mutation while traversing by stable handle where retaining an element borrow
  would be too restrictive.

An element reference or physical cursor carries owner provenance and freezes
reserve, reallocation, deletion of the referenced element, and owner drop for
its lifetime. A region-free logical cursor may instead store a handle or index,
but every access must revalidate the logical identity and occupancy required by
its contract; it does not inherit a permanent physical-address fact.

The existence of reborrow-through-holder (OWN-6 delta) and result reborrows
(OWN-14) is already greenlit in the design tree; this report does not reopen
that decision. The canaries test whether those approved routes, once
implemented, are sufficient for the required access patterns. An additional
affine entry/cursor token is a distinct proof and performance proposal and may
enter comparison only for a measured residue that the approved reborrows do not
cover.

### 7.5 Encapsulation and static behavior

A public `{data, len, capacity}` record is not a safe container if clients can
forge `len`, split the fields, or expose the tail. Modules/private fields and
opaque construction are therefore correctness prerequisites, not merely API
organization.

Generic collections also need a genuinely callable static contract path for
equality, hashing, ordering, cloning, iteration behavior, and possibly
allocation policy. Calls must remain direct after monomorphization. Inherent
type-owned APIs can group and seal operations without introducing receiver
auto-borrow, vtables, or dynamic dispatch. Function values are not established
as a floor requirement if the existing env-struct direction becomes completely
callable and supports stateful repeated use.

## 8. Kernel, sealed standard library, and ordinary libraries

These layers answer different questions.

### 8.1 Language substrate

The kernel specifies the checked state transitions and the proofs they produce:
initialization, ownership transfer, exact destruction, borrowing/invalidation,
allocation/failure, and any metadata-to-payload fact. It should be generative,
not a list of `hashmap_insert` special cases.

No writer-callable operation may hand out raw uninitialized payload access,
unchecked `set_len`/`mark_full`, or an equivalent split privilege. Every
callable transition must preserve a valid owner on its own; safety may not
depend on the standard library eventually making a second call. If an
implementation needs transient invalid state, the kernel must contain and
complete that state machine atomically or prove a scoped transaction whose every
non-trap exit leaves a valid owner. Repair-on-drop is a separate finalizer
proposal under Section 7.1, not a current library capability.

### 8.2 Sealed-library witness

Each candidate must demonstrate that an opaque library can preserve length,
heap-order, probe-metadata, tree-balance, or handle-representation invariants as
applicable. This is an experimental witness, not a commitment to ship every
family in the standard library. Only families that later pass their complete
evidence gate, R1, and a separate owner adoption decision receive a canonical
standard-library API; other topology witnesses may remain ordinary libraries.

This opacity does not reject same-typed wrong-pool handles; their narrower
standing contract is preserved in Section 9. “Sealed” must not mean unchecked:
the witness remains checked xlang over the kernel transitions above. It does not
receive a private raw-memory, uninitialized-tail, `set_len`, or
unchecked-payload privilege that an ordinary checked abstraction could never
obtain.

### 8.3 Ordinary no-unsafe libraries

An ordinary library need not implement a raw allocator or ABI. It must be able
to instantiate every checked ownership transition needed by an unseen
storage-bearing structure without inaccessible per-structure intrinsics. Using
public `HashMap` and `sequence` to compose an LRU proves composition; it does not
alone prove that a novel sparse store can be implemented. The acceptance suite
therefore needs both composition witnesses and at least one held-out structure
that exercises the public storage-state substrate itself.

The precise boundary is an owner decision after experiments. Public
writer-visible raw memory or `MaybeUninit<T>` is rejected by T1/W3; a standard
library that alone receives arbitrary unsafe privilege is insufficient for a
general-purpose language; requiring ordinary libraries to rebuild allocators is
unnecessary.

G0-Core designates H-STORE, freezes its purpose and dependency budget, and
excludes it from candidate tuning. The implicated Family Lock A instantiates an
exact dependency list only within that frozen budget. H-STORE directly
instantiates the public dense or sparse storage-state transitions and may not
call the corresponding finished sequence, map, pool, or another wrapper that
hides those transitions. H-LRU and H-IPQ remain composition witnesses; they
cannot substitute for H-STORE.

## 9. Handle and pool correctness

Append-only and recyclable stable pools have different observable contracts.

An append-only typed handle can be one index. It remains valid until the whole
pool dies; access needs only a bounds check. This is ideal for ASTs, pass-local
IR, phase graphs, and other monotonic structures.
The append-only contract therefore has no per-element delete, clear, truncate,
index reuse, or conversion into a recyclable pool while issued handles remain;
reclaiming all storage is owner destruction.

A recyclable pool aims to reuse payload storage rather than retain one payload
slot per historical insertion. That does not by itself prove memory proportional
only to peak live population: finite-generation retirement and identity history
can grow under adversarial churn. The stable-identity Family Lock A must freeze
and measure live payload, high-water capacity, retired slots, and retained
history separately.

If a copied handle contains only slot `i`, reusing `i` gives two different
objects identical handle bits. No lookup can distinguish them. The choices are:

1. never reuse the index;
2. add fresh identity metadata and check it;
3. statically revoke every old handle before reuse.

Freely Copy graph handles make option 3 unavailable in the general case.
Generational handles are therefore a natural candidate, but their full contract
must define:

- pool provenance and cross-pool misuse;
- stale-handle result (recoverable absence or trap);
- deletion while borrowed;
- relocation behavior;
- generation exhaustion without silent wrap/identity resurrection;
- exact handle width and per-access checks;
- iteration over fragmented occupancy.

Retiring an exhausted slot or trapping before reuse is safer than allowing a
bounded generation to wrap. A recyclable design must never resurrect an issued
identity: a maximum-generation slot is retired, clear/reset invalidates every
issued handle without discarding the history needed to prove that fact, and
generation metadata cannot be shrunk away and later reused for the same index.

There is a finite-identity impossibility boundary: freely copied finite-width
handles cannot simultaneously promise indefinite reuse, memory bounded only by
peak live population, and permanent rejection of every stale handle. G0-Core
records this conflict as a mandatory contract dimension; the stable-identity
Family Lock A freezes which guarantee is relinquished. Candidate contracts
include eventual exhaustion/trap, slot retirement with an explicit
high-water/history term, or a static revocation discipline that does not permit
freely copied escaping handles. C-POOL includes steady-live adversarial churn with deliberately tiny
generations so this choice cannot hide behind a large production width.

The recorded owner ruling classifies a same-typed handle from another pool that
selects an in-bounds object as a memory-safe logic bug. This research does not
promote pool provenance into a safety theorem or a free global-identity fact.
If a selected stable-identity API promises cross-pool rejection, that promise
must be priced explicitly; otherwise its documentation and held-out tests must
preserve the narrower ruling.

Append-only and recyclable contracts may share implementation machinery, but
they must be statically distinguishable so the append-only path remains
generation-free. Nominal types, a compile-time policy, and other zero-cost
encodings remain candidates; a runtime flag that charges every append-only
access for recycling semantics is rejected.

## 10. Staged completeness and performance gate

Completeness is frozen globally and closed family by family. G0-Core prevents a
difficult capability from disappearing or changing contract after results; it
does not preregister every future experiment. Exact selection data belongs to
the relevant Family Lock A immediately before that family's candidate work.

### 10.1 G0-Core: global boundary freeze

G0-Core is timeboxed to one owner-review cycle. It contains no candidate code,
harness, reference implementation, scored trace, or family-specific numeric
threshold. It freezes:

- the complete Section 4 registry, including role, current status, canary,
  dependency family, one-paragraph caller-observable contract, and required
  asymptotic class;
- the allowed R3 contract dimensions: result/order, ownership/invalidation,
  error/trap/drop, complexity, contiguity, stable identity/address, range and
  iteration guarantees, enforceable allocation or memory ceilings, and any
  genuinely open-world behavior requirement;
- global ownership, exact-destruction, recoverable-failure, borrow/relocation,
  affine-abandonment, and optimizer-fact-channel rules;
- forbidden pathological simulations and every protected B no-regression or
  no-tax rule;
- family order and dependencies, O-promotion policy, claim vocabulary, and the
  reopening rules in Section 10.7;
- H-STORE, H-LRU, and H-IPQ purposes, visible contracts, dependency budgets,
  exclusion rules, and holdout-rotation policy;
- the cumulative META-5 accounting schema and the benchmark, soundness, lock,
  hostile-review, and durability templates used by later family gates.

Private representation, node size, probe layout, algorithm family,
monomorphization, and internal static/dynamic dispatch never create a new R3
contract. One writer-facing route may specialize internally by `T`, size,
target, or a preregistered crossover.

G0-Core does not freeze future families' exact algorithms, payload matrices,
traces, allocator, numeric margins, complete soundness fixtures, or populated
derivability ledgers. Scope that does not fit the timebox moves to the relevant
Family Lock A; it does not enlarge G0-Core. Closing G0-Core only makes a later
request to draft the dense-family Lock A eligible. It does not itself authorize
that draft or any candidate implementation.

### 10.2 Role-mapped canaries

The following inventory is the global coverage map. G0-Core freezes each
canary's identity, role, coarse observable contract, dependency family, and
asymptotic requirement. A Family Lock A freezes exact programs and measurements
only for the canaries relevant to that family. Relevant M, W, and H obligations
block that family's closure; every M, W, and H obligation blocks the complete
capability-floor claim. B rows are always protected no-regression controls. O
rows block only if G0-Core or a later owner ruling promotes them, or if a
mandatory contract depends on them.

- **C-FIX:** unchanged scalar-buffer source/IR plus fixed AoS record indexing;
- **C-SEQ:** unknown-length affine construction, reserve/grow/shrink, bulk
  extend/move-append, split, ordered insert/remove, O(1) `swap_remove`, pop,
  dynamic swap, truncate/clear, explicit deep clone, and destruction;
- **C-COMPACT:** stable retain and eager drain/splice with direct element-traffic
  accounting;
- **C-LAZY-DRAIN:** optional partial-consumption cursor, including early drop or
  explicit finish under the selected finalizer policy;
- **C-SORT:** unstable and stable sort of affine payloads with a static
  comparator and partial-scratch cleanup;
- **C-DEQUE:** wrap/rebalance, `make_contiguous`-like access, and adversarial
  alternating-end churn under a frozen memory bound;
- **C-HASH:** accumulator/Entry, reserve/shrink, remove/retain/rehash, adversarial
  collisions, and steady-live delete/insert churn;
- **C-ORDER:** comparison-tree node split/merge and ordered range mutation;
- **C-HEAP:** priority push/pop/repair for the frozen priority contract;
- **C-P2:** unchanged append-only AST/DAG access and machine-code accounting;
- **C-POOL:** recyclable deletion/reuse, stale access, and the finite-identity
  steady-live churn boundary from Section 9;
- **C-GRAPH:** frozen CSR plus dynamic graph deletion and known-neighbor
  singly/doubly linked rewiring;
- **C-RECUR:** recursive box construction, extraction, mutation, inline-cycle
  rejection, and very deep destruction with bounded stack use;
- **C-ARENA:** homogeneous stable-address allocation, partial construction,
  exact element drops, and bulk reset;
- **C-SMALL:** inline-to-heap affine-sequence transition with exact ownership,
  drop, and below-threshold allocation accounting;
- **C-TEXT:** unknown-length byte output, bulk append, UTF-8 validation/sealing,
  and boundary-preserving edits;
- **C-ITER:** shared, uniq, and owning iteration with escape/invalidation
  negatives; and
- **C-FAIL:** every owning canary repeated through early `Result`, checked
  capacity failure, partial construction, and each recoverable allocation point.

H-STORE, H-LRU, and H-IPQ are frozen, training-excluded anti-special-casing
witnesses, not substitutes for this registry. H-STORE's dependency budget
forbids the corresponding finished container. A candidate sees their contracts
but not their implementation or performance traces until the training
candidates and scoring rules are frozen.

### 10.3 Family Lock A and Candidate Freeze B

Before any candidate implementation for family F, Family Lock A(F) must freeze:

- exact operation results, ordering, ownership and invalidation, failure and
  drop behavior, asymptotic guarantees, and enforceable allocation or memory
  ceilings for every implicated contract;
- the candidate mechanisms and reference algorithms to be compared;
- the candidate-construction and tuning protocol: author, model, tool, and
  candidate assignment; implementation order or randomization; time, token,
  repair, and tuning budgets; allowed shared versus candidate-specific changes;
  and correction, disqualification, and failure rules;
- the applicable B controls and M, W, and H canaries;
- payload classes, sizes, capacities, load factors, operation distributions,
  realistic and adversarial traces, targets, `DataLayout`, allocator, and
  recoverable-allocation policy;
- structural counters, primary and secondary endpoints, non-inferiority
  margins, lexicographic tie breaks, crossover rules, and the unique-selection
  or no-selection rule;
- the exact soundness and failure corpus applicable to the family;
- the incremental META-5 ledger: every public spelling, kernel transition,
  trusted fact or code path, normative rule, and grammar, type, ownership,
  effect, drop, diagnostic, and lowering obligation, with a derivability matrix
  from candidate transitions to required operations;
- training/held-out separation, witness custody and disclosure points, and the
  rule for rotating a compromised holdout; and
- artifact hashes, independent review scopes, durability requirements, and the
  later owner decision required before production adoption.

“Fastest” is undefined without these workload and target choices. Performance
crossovers by `T`, size, or target may be handled behind one canonical writer
route by a preregistered static specialization or dispatch rule. They do not
create another R3 contract. Non-inferiority alone cannot select a winner. If the
preregistered rule yields no unique survivor, the result is “no selection,” not
permission to ship several equivalent writer-facing routes.

After implementations exist but before any scored or held-out run, Candidate
Freeze B records exact source hashes, build commands, toolchain and environment,
training results, compliance with the frozen construction/tuning protocol, only
the corrections already authorized by Lock A, and the immutable scoring inputs.
A deviation reopens Lock A. Before implementing or exposing a public transition,
state topology, trusted fact, or trusted code path whose admissible semantics
span families, Lock A must have closed for every implicated family. The same rule
applies to any cross-family contract or performance claim.

### 10.4 Family structural and measured performance gate

Before wall-clock timing, reject a route that has any of these contract-level
failures:

- an unsupported mandatory operation or ownership/failure path;
- an asymptotic regression against the frozen contract;
- per-element allocation under a contiguous-storage contract;
- full-capacity construction of generic payload values for spare-capacity
  storage;
- a hidden whole-record Copy/Clone requirement;
- tombstone, retired-slot, or historical-allocation growth beyond the frozen
  family contract;
- writer-visible raw initialization/occupancy state; or
- a generation check, generation field, or recycling branch on the protected
  append-only path.

The following are measured costs, not automatic rejections: fused tag or bitmap
bytes, element traffic relative to a payload-specific direct-relocation byte
model, allocator calls, drop scans, bounds/alias branches, static versus
indirect behavior calls, and SIMD versus scalar probing. Family Lock A freezes
thresholds per payload, scale, trace, and target; monomorphization is charged
for code and instruction-cache growth as well as credited for direct calls.

Record allocations, initialized/touched/moved bytes, peak and transient bytes,
metadata bytes per live element, drops, checks, branches, vector width,
instructions, code size, load factor, probe count, cache/TLB misses,
fragmentation, and p50/p95/p99 latency. Layout, capacity arithmetic, handle
width, and thresholds use the target `DataLayout`. Every “pathological” claim
must reduce to a frozen structural rule or a 99% confidence margin before
scoring.

### 10.5 Family soundness gate

G0-Core freezes the following attack schema. Family Lock A must instantiate all
applicable attacks as exact fixtures and justify every inapplicable item:

- underfill, overfill, and repeated finish;
- rejected push losing its affine value;
- move-after-pop/remove, nested affine payloads, exact drop counters, and double
  drop;
- growth, deletion, clear, and owner drop under live element/slice/entry borrow;
- allocation or checked-capacity failure at each preparation step, including
  preservation of the offered affine input on a failed push;
- partial clone/drain/retain destruction and early drop of rebuild/cursor tokens;
- stale, cross-pool, relocated, cleared, shrink/regrow ABA, and deliberately
  small-width exhausted-generation handles;
- forged Full metadata, payload/control disagreement, proof invalidation after
  every metadata mutation, and inability to call unchecked payload access;
- duplicate or overlapping mutable positions;
- cursor escape and mutation after invalidation;
- recursive drop and inline size cycles;
- comparator/hash inconsistency containment;
- 32-bit and 64-bit size, capacity, and layout boundaries;
- MemorySanitizer-style checks that SIMD probing never reads non-`Full` payload
  lanes;
- fact-on/fact-off semantic and code-shape identity where facts are not earned.

Every initialization or occupancy relation that permits check elimination is a
fact channel and receives hostile review before shipping. Green tests alone do
not approve it.

### 10.6 Family closure, adoption, and the complete-floor claim

Each family candidate must provide:

- a kernel state-machine witness;
- a sealed standard-library implementation;
- an ordinary-library held-out witness;
- independent ownership/state/failure review;
- independent complexity/machine-code attribution review;
- independent census/R3/repository-consistency review.

Family F closes only when:

- every M obligation assigned to F is established or proven derivable;
- every protected B control remains non-regressed and pays no forbidden tax;
- every W obligation assigned to or implicated by F under the G0-Core dependency
  map is implemented by an ordinary no-unsafe library through public checked
  mechanisms, completely and efficiently;
- every H witness assigned to F passes without candidate-specific compiler or
  standard-library privilege;
- fact channels, ownership/state/failure behavior, machine code, R3 scope, and
  repository consistency have passed independent hostile review; and
- the preregistered rule selects a unique held-out survivor.

A closed family may seek a separate owner decision for production adoption. An
approved adoption updates the specification, derivation ledger, design tree,
pattern catalog, tests, and implementation in the same durable change, as
applicable. It may expose no public transition, state topology, trusted fact, or
trusted code path outside the adopted family locks. It does not establish
another family's capability, approve a shared universal substrate, authorize
xlc migration or default teaching, or permit a general-purpose capability-floor
claim.

The complete floor may be claimed only after every B baseline remains
non-regressed and every M, W, and H obligation is closed. An O obligation blocks
only if it was promoted or is required by a mandatory contract. Every other O
deferral must retain its corpus decision and dependency result rather than
silently disappearing.

### 10.7 Reopening rules

A closed global or family lock reopens before further candidate work when:

- an observable result, ownership/invalidation, failure/drop, complexity,
  contiguity/stability/order, or enforceable resource contract changes;
- G0-Core's registry role, family assignment, dependency map, held-out
  dependency budget, or scoped deferral changes;
- a new candidate mechanism or crossover rule could change selection;
- a scored implementation, workload, target, allocator, threshold, endpoint,
  gating or scored soundness fixture, or holdout becomes visible, corrected, or
  compromised outside the frozen correction rule;
- a new fact channel, trusted path, public spelling, or normative obligation is
  introduced;
- a supposedly family-local candidate depends on an unlocked family; or
- evidence invalidates a protected baseline, soundness claim, asymptotic bound,
  or ordinary-library derivation.

Any gating or scored soundness-fixture change after Lock A reopens that family;
after Candidate Freeze B it also invalidates the freeze and requires symmetric
reruns. An explicitly non-gating diagnostic fixture already covered by the
frozen attack schema may be appended without reopening. So may an independently
recoverable, non-semantic clerical correction made before Candidate Freeze B
with an append-only explanation, or a non-semantic toolchain fingerprint. The
lock record must state which rule applies; ambiguity reopens it.

## 11. Recommended research order

The previous five E0 topics remain useful, but their ordering is not a capability
argument. The work is serial within a family and staged across families. No item
below starts without separate owner authorization.

1. **G0-Core — bounded global boundary.** In one owner-review cycle, freeze only
   the Section 10.1 registry, laws, schemas, dependencies, claim vocabulary, and
   reopening rules. Reconcile them against the five E0 topics and existing MCTS
   alternatives. Produce no syntax, candidate implementation, harness, or
   scored result.
2. **G1 — dense-family Lock A and experiment.** Freeze exact dense sequence
   semantics and the complete family protocol. It must explicitly retain,
   revise, or supersede every arm and measurement promised by the paused E0.1
   fixed-record protocol. Only a later owner approval may start candidates.
   Compare the smallest safe high-level operation set with a linear
   rebuild/relocation transaction; cover affine growth, ordered removal, stable
   retain, sorting, failure, and any xlc-shaped witnesses justified by the
   registry. Keep fixed buffers unchanged.
3. **G2 — sparse-family Lock A and experiment.** Compare fully initialized
   tagged slots against compact split metadata under one frozen hash-table
   algorithm, price the payload/metadata fact channel, and prove exact drops
   through rehash. A dense candidate may not implement or expose a public
   transition or trusted fact spanning sparse state before this lock exists;
   sparse selection may still require later algorithm comparison under its own
   contract.
4. **G3 — stable-identity-family Lock A and experiment.** Compare append-only
   typed pools, recyclable generational pools, recursive ownership, and cursor
   routes on the assigned graph, list, arena, and LRU witnesses. Preserve the
   protected append-only path without generation or recycling tax.
5. **Later family locks and experiments.** Deque, heap, ordered map/tree, and
   bytes/text each receive their own exact Lock A, applicable ordinary-library
   and held-out witnesses, Candidate Freeze B, measurement, and hostile review.
   One family's closure does not wait for unrelated optional families, and any
   per-structure compiler exception reopens the implicated decision.
6. **Integration and default teaching.** Claim the complete floor only under
   Section 10.6. Capability adoption, xlc migration, default layout teaching,
   and a benchmark-blind low-tier writer panel retain separate owner gates.
   Expert implementation evidence alone does not establish the default.

AoS versus SoA remains an independent representation question inside these
workloads. The language must not make one inexpressible. Capability adoption,
xlc migration, and default layout teaching retain separate gates.

## 12. Owner rulings recorded

D11 records the following boundary:

1. General-purpose systems-language capability is the eventual target. For data
   structures, representative standard contracts need one taught efficient
   route, and ordinary no-unsafe libraries must be able to implement
   representative held-out structures through public checked mechanisms.
2. G0 is split into a bounded G0-Core and an exact Family Lock A immediately
   before each family's candidate work. G0-Core does not prebuild all future
   protocols.
3. A family may seek separate owner-approved production adoption after its own
   complete gate. The complete capability-floor claim waits for all protected
   B baselines and mandatory M, W, and H obligations.
4. R3 selects one canonical writer-facing mechanism per frozen observable
   semantic and performance contract, not one universal representation and not
   one global algorithm for every unforeseen structure.
5. Append-only and recyclable stable identity are separate contracts. Shared
   machinery remains experimental, and the append-only path pays no generation
   or recycling tax.
6. G0-Core plus dense Family Lock A are necessary but not sufficient to lift
   the E0.1 pause. The old fixed-record paired protocol does not restart
   automatically and may be superseded by the dense result.

This ruling authorizes only correction of the research boundary. It does not
authorize G0-Core work, a family lock, candidate or production implementation,
specification changes, xlc migration, a scored run, or default teaching. The
next step remains for a separate owner discussion.

## 13. Primary sources and local evidence

Primary external sources:

- Rust standard collection families, selection guidance, and operation costs:
  <https://doc.rust-lang.org/std/collections/>
- Rust `Vec` representation, initialized prefix, capacity, and operations:
  <https://doc.rust-lang.org/std/vec/struct.Vec.html>
- Rust `VecDeque` growable ring and segmented representation:
  <https://doc.rust-lang.org/std/collections/struct.VecDeque.html>
- Rust `LinkedList` API and implementation source:
  <https://doc.rust-lang.org/std/collections/struct.LinkedList.html> and
  <https://doc.rust-lang.org/src/alloc/collections/linked_list.rs.html>
- Rust `HashMap`, `BTreeMap`, and `BinaryHeap`:
  <https://doc.rust-lang.org/std/collections/struct.HashMap.html>,
  <https://doc.rust-lang.org/std/collections/struct.BTreeMap.html>, and
  <https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html>
- Rust `mem::replace`, `mem::swap`, slices, and iterators:
  <https://doc.rust-lang.org/std/mem/fn.replace.html>,
  <https://doc.rust-lang.org/std/mem/fn.swap.html>,
  <https://doc.rust-lang.org/std/primitive.slice.html>, and
  <https://doc.rust-lang.org/std/iter/>
- Abseil Swiss-table control metadata and SIMD probing:
  <https://abseil.io/about/design/swisstables>
- Abseil cache-dense B-tree design and pointer-stability trade-off:
  <https://abseil.io/about/design/btree>
- Rust SlotMap identity, slot reuse, ABA caveat, and measured representation
  trade-offs: <https://docs.rs/slotmap/latest/slotmap/>
- petgraph stable graph deletion/index behavior:
  <https://docs.rs/petgraph/latest/petgraph/stable_graph/struct.StableGraph.html>
- rustc typed indexed vectors:
  <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_index/vec/struct.IndexVec.html>
- Swift Collections package's deque and broader collection inventory:
  <https://github.com/apple/swift-collections>
- Go heap and list implementations as independent operation witnesses:
  <https://go.dev/src/container/heap/heap.go> and
  <https://go.dev/src/container/list/list.go>
- Linux intrusive lists and red-black trees:
  <https://docs.kernel.org/core-api/list.html> and
  <https://docs.kernel.org/core-api/rbtree.html>
- Ropes and gap buffers as specialist edit structures:
  <https://doi.org/10.1002/spe.4380251203> and
  <https://www.gnu.org/software/emacs/manual/html_node/elisp/Buffer-Gap.html>

Local evidence:

- `CONSTITUTION.md` — R2/R3 and the P0/W1/W3 ordering.
- `PATTERNS.md` and `mcts_mem/xlang/pattern-doctrine.md` — COMPLETE and EFFICIENT
  catalog tests.
- `spec/kernel-spec-v0.6.md` — TYPE-2, OWN-1/5/6/7/11, STOR-1/3, FN-2/3/5.
- `mcts_mem/xlang/data-model.md` — fixed buffer and append-only pool/handle
  direction.
- `mcts_mem/xlang/ownership/no-reborrow.md` — current restriction and recorded
  relief directions.
- `mcts_mem/xlang/toolchain.md` — fixed-capacity xlc as a bootstrap decision.
- `experiments/data-layout-owning-sequence/BASELINE.md` — xlc capacity and live
  prefix accounting.
- `experiments/data-layout-owning-sequence/RESEARCH_REPORT.md` — AoS, Copy,
  initialized-prefix, and STOR-1 separation.
- `experiments/port-study/binary-trees/RESULTS.md` — measured SoA/indexed tree
  evidence and its caveats.

## 14. Claims not made

This report does not establish:

- exact ecosystem prevalence for every operation;
- a selected surface spelling or kernel type;
- that dense and sparse state require distinct language primitives;
- that a generational pool is the only possible recycling design;
- that a linked list should be a default standard container;
- that xlc should migrate from its current fixed SoA layout;
- that AoS or SoA is globally faster;
- that function values, reference counting, GC, or dynamic dispatch are
  required;
- that any proposed route is sound, production-ready, or performance-optimal;
- authorization to implement, score, teach, or externally disclose a candidate.
