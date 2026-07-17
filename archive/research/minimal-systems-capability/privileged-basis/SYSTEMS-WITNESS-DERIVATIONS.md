# Systems Witness Derivations

Status: constructive D14 paper derivations for a proposed architecture pending
owner review, 2026-07-15. These non-exhaustive traces test the candidate
certified-resource architecture. They are not exact D-2 closure, exact P-1
evidence, a mechanized proof, implementation, syntax selection, performance
result, or production decision.

## 1. Common notation and acceptance rule

```text
ByteCarrier(k,rho,B,A)    one byte-extent root
Place(rho,p,e,T,F)        checked typed logical place over footprint F
Vac(rho,S)                no declared T is live in logical places S
Full(rho,S)               exactly one owned T is live in every place of S
MoveAuth(rho,S)           authority to move or relocate places S
ReleaseAuth(rho)          authority to release the root after all leases end
P * Q                     exact separating composition
OwnerM(state)             opaque ordinary module predicate
Stable(rho,epoch,S)       footprint S cannot relocate before invalidation
Atomic(a,protocol)        atomic location governed by one public protocol
Callable(C)               checked code lease, environment, and invoke authority
```

`Carrier(k,rho,T,C)` below is the proof-erased homogeneous shorthand. Calling it
proof-erased does not grant P-1 credit for a concrete operation. Unless a trace
shows an escrow explicitly, each sealed owner contains partitioned
`MoveAuth` for all of its places and one `ReleaseAuth`. Every `take`/`put`
consumes and reproduces the relevant move authority. Every root release
requires the complete vacant footprint, recombined move authority, release
authority, and quiescence of every borrow, stable-place, external, code, and
reclamation lease.

Each trace passes only when:

1. every owner, root, live cell, vacancy, move/release authority, loan, handle,
   code lease, and external lease has one exact pre- and post-state role;
2. every normal edge reseals a valid owner, transfers it, or executes a total
   disposition;
3. every recoverable failure states its exact commitment and partial progress;
4. no proof introduces an axiom, machine action, memory-model rule, or foreign
   assumption;
5. every irreducible action is an exact entry under the one gate; and
6. the runtime representation contains only state required by the selected
   contract or algorithm.

The earlier `SEQUENTIAL-WITNESS-DERIVATIONS.md` remains evidence for the
bounded nine-role candidate. The traces below test the broader proposed basis,
where arbitrary exact vacancy resources replace the bounded topology grammar.

## 2. Sequential unique-owner storage

### 2.1 Inline-small sequence

```text
Inline(n):
  tag = inline
  * Carrier(inline,rho_i,T,N)
  * Full(rho_i,[0,n))
  * Vac(rho_i,[n,N))

Heap(n,C):
  tag = heap
  * Carrier(heap,rho_h,T,C)
  * Full(rho_h,[0,n))
  * Vac(rho_h,[n,C))
  * inactive inline carrier entirely vacant
```

Inline push performs `put(n,value)` and increments `n`. Pop decrements `n`,
then performs `take(n)`. Spill acquires the heap carrier before focus. Failure
returns the unchanged inline owner and offered value. Success takes every
inline value, puts it into the heap prefix, puts the offered value, proves the
inline carrier vacant, and changes the tag only while resealing the complete
successor invariant.

The discriminant and live-set relation are one predicate, so an early tag
write cannot authorize the wrong disposer. The runtime representation is the
ordinary small-sequence representation: tag, length, optional heap
pointer/capacity, and inline bytes. There is no liveness bitmap or proof field.

### 2.2 Gap buffer

```text
Gap(a,b,C) = Full([0,a)) * Vac([a,b)) * Full([b,C))
```

Move right by one is `take(b); put(a); a += 1; b += 1`. Move left is the
inverse. Insert puts at `a`; deletion takes at `b`. Growth reserves a new
carrier before focus, relocates both ranges, releases the vacant old carrier,
and reseals. The verified plan destroys exactly the two live ranges.

Runtime state is pointer, capacity, and the two endpoints. The proof adds no
tag or scan beyond what the algorithm already needs.

### 2.3 Flat ordered set and binary heap

Both own `Full[0,n) * Vac[n,C)`. Sorted uniqueness and heap order are ordinary
functional properties unless exported as checked facts.

Flat-set insertion starts with vacancy `n`; repeated `take(i-1); put(i)` moves
that vacancy left to the insertion point, where the offered value is put.
Removal performs the inverse and returns the first taken owner.

Heap push puts at `n` and repeatedly uses the derived dynamic swap after
comparisons. Pop takes root and final slot, puts the final owner at root, and
sifts through derived swaps. Comparator loans end before each conflicting
transition. No privileged `set`, `heap`, swap, or replace operation exists.

### 2.4 Circular deque and owning array iterator

```text
Ring(head,len,C) =
  Full({(head+j) mod C | 0 <= j < len})
  * Vac(the complement)
```

Push and pop at either end use one put or take and update `head,len`. The
`C=0` branch is resolved before modulo. Growth relocates logical order into a
new prefix. Runtime state matches a conventional deque: pointer, head, length,
and capacity.

After zero-code full adoption, an owning array iterator holds:

```text
Vac([0,front)) * Full([front,back)) * Vac([back,N))
```

`next` takes `front`; `next_back` decrements and takes `back`. Its verified
plan destroys exactly the remaining interval. For `Trivial` payloads the plan
specializes to no loop.

### 2.5 Dense/sparse store

```text
keys:     Full[0,n) * Vac[n,C)
values:   Full[0,n) * Vac[n,C)
position: fully initialized Copy metadata [0,U)
```

Lookup checks `k < U`, reads `p = position[k]`, checks `p < n`, then reads the
live dense key/value and accepts only when `dense_key[p] == k`. The bounds
check and dense-prefix ownership authorize memory access. Position coherence
is ordinary map correctness.

Removal takes key and value at `p`. If `p != n-1`, it takes the final pair,
puts it into the vacancies at `p`, and updates the moved key's position. Both
carriers end with prefix `n-1`. Stale position entries are harmless and need
not be cleared. This route needs no arbitrary-sparse payload primitive.

### 2.6 Copy, zero-code reshape, full adoption, and ZST

`copy_from` and overlapping `copy_within` require `CoreCopy(T)`: exact
layout/value preservation, structurally duplicable leaves, trivial disposition,
and zero protocol action. They borrow the source, use `overwrite_copy`, perform
exactly the specified copy traffic, leave the source live, and perform zero
behavior calls. `init_copy` separately converts each destination `Vac` to
`Full`. A refcounted handle is not `CoreCopy` merely because its handle has no
unique leaf. These routes never use `take`, relocation, or an implicit `Clone`. Explicit
clone, fill, resize-with, and producer members instead invoke their exact
ordinary `Callable` contract with the registered count, order, effects, and
result provenance.

For a dense owner of `[T; N]`, checked multiplication first proves the flat
length and the fixed array-layout theorem proves an ownership/layout bijection:

```text
Full<[T;N]>(rho,[0,len)) * Vac<[T;N]>(rho,[len,cap))
  <->
Full<T>(rho,[0,len*N)) * Vac<T>(rho,[len*N,cap*N))
```

`reshape_partition` retires the old place epoch, maps `MoveAuth`, invalidates
old facts, and mints the flat view while preserving the same carrier and
`ReleaseAuth`. It executes no payload read, write, move, allocation, or
release. Converting an exact-length full heap owner into a growable owner is
the same kind of erased owner repackage. `adopt_full_storage` supplies the
inverse route for the protected two-word fixed-heap baseline.

For zero-sized `T`, the included paper policy uses a virtual root, zero allocated
bytes/calls, target `usize::MAX` logical capacity, and distinct
`(rho,epoch,index)` tokens. A move ends one source token and creates one
destination role while moving zero bytes; disposition visits exactly `len`
owners. Pointer comparison proves nothing. The alternative policy rejects the
instantiation. The paper construction sketches both policies without changing
the candidate basis; exact dense-family direction and D-2 credit remain
unresolved.

### 2.7 Heterogeneous, runtime-tail, and packed storage

A checked runtime layout can partition one carrier into a fixed header, padding
`Space`, and a runtime-count tail of typed places. A heterogeneous arena keeps
ordinary per-object layout/tag/disposition metadata only when its selected
type-erased contract needs them; monomorphic arenas use static place witnesses.
Vacant union reuse first `uncarve`s the old view, retires its epoch, and carves
the new checked view. No live bytes are reinterpreted.

A packed protocol field has an unaligned access profile. By-value load/store or
take/put uses the exact target unaligned lowering or a bytewise sequence and
never creates a misaligned borrow. Atomic or volatile packed fields additionally
use their fixed R-4/R-5 contract. These routes are intended to preserve the
conventional one-allocation layout without imposing a tag or alignment branch
on ordinary aligned storage. Exact same-contract P-1 evidence remains pending.

## 3. Direct arbitrary sparse occupancy

### 3.1 Robin Hood open-addressed table

Use a fully initialized control-byte carrier and separate key/value carriers.
The opaque predicate is:

```text
Table(ctrl,keys,values,C) =
  for every i in [0,C):
    ctrl[i] is Empty or Tombstone
      => Vac(keys,i) * Vac(values,i)
    ctrl[i] is Occupied(hash_fragment,probe_distance)
      => Full(keys,i) * Full(values,i)
  * ordinary probe-sequence relation
```

Reading a control byte is always safe. Payload access requires the occupied
branch proof for that exact slot. Insertion under focus carries an offered
key/value pair and one logical vacancy. A Robin Hood displacement takes the
resident pair, puts the offered pair, updates the control byte, then continues
with the displaced pair. No pair is duplicated and every normal loop edge has
one candidate or a sealed table.

Backshift deletion takes the removed pair, then repeatedly takes the next
occupied pair and puts it into the current vacancy until an empty slot or
zero-distance entry ends repair. The final control byte becomes empty before
resealing. Rehash acquires the complete destination before opening the source.

The verified plan scans the control bytes, taking and disposing payloads only
on occupied branches, then releases all carriers. The runtime control bytes and
branches are algorithm-required; no second liveness bitmap or privileged hash
table operation appears.

### 3.2 Recyclable stable pool

Each slot has ordinary generation/occupancy metadata and a payload carrier.

```text
Free(i,g)  => Vac(payload,i)
Live(i,g)  => Full(payload,i)
Handle(i,g) validates only against Live(i,g)
```

Removal takes the payload, advances or retires `g`, and links the vacancy into
the free list. Reuse puts a new owner before publishing `Live(i,g')`. A stale
handle fails its generation check and cannot create payload authority. On
generation exhaustion the protocol retires the slot or returns an exact error;
wraparound cannot silently revive a handle.

The generation and free-list state are required by recyclable identity. The
separate append-only pool proof omits both and preserves its one-bounds-check
path.

## 4. Recursive and multi-root ownership

### 4.1 B-tree or rope

An ordinary node representation owns fixed-capacity key/value/child carriers.
Its recursive predicate is conceptually:

```text
Node(rho,n,height) =
  Full(keys,[0,n)) * Vac(keys,[n,K))
  * exact value footprint
  * exact child-owner footprint
  * recursively Node(child_j, ..., height-1)
  * ordinary ordering/balance relation
```

The verifier checks positivity/productivity and recursion decreasing by height
or a well-founded allocation graph. Search borrows one path. Split acquires the
new node before focus, relocates exact owners with take/put, and publishes the
new child only after both nodes satisfy their predicates. Parent split repeats
the same ordinary theorem. A recoverable allocation failure occurs before the
affected level commits or returns the exact prepared partial result named by
the contract.

The paper plan performs a postorder traversal, disposing child owners before
their containing node carrier. It needs no runtime destructor pointer. Its
candidate node layout, branching factor, allocation count, and traversal stack
are chosen to match the conventional implementation; that structural argument
does not itself discharge exact P-1.

### 4.2 Unique linked topology

A singly or doubly linked unique list uses one recursively defined predicate
over node roots. Detach consumes the linking proof and returns one node owner;
splice consumes two list predicates and proves the successor relation. A
doubly linked node carries two ordinary address fields but one root owner; the
predicate proves consistency without treating pointer equality as ownership.

A cycle cannot be formed under direct unique recursion because no finite
well-founded owner can contain itself. Cyclic graphs use a pool/handle protocol
or a shared-ownership protocol. This is an explicit representation choice, not
a failure to express graphs.

### 4.3 Multi-block arena

The arena predicate owns an ordinary dense sequence of block-owner handles,
each block's carrier, and exact payload footprints. Relocating a block handle
does not relocate the block root or retarget payload borrows. New blocks are
acquired before their handle is put into the registry.

Within a block, an ordinary allocator partitions vacant `Space` and delegates a
fresh child root whose disposition returns the complete vacant footprint. The
block escrows overlapping move/release authority until that child returns.
Free-list, slab, size-class, buddy, coalescing, and quarantine policies therefore
need no per-allocation privileged entry after the parent block is acquired.

Disposal takes every payload before releasing its block, then disposes the
block-owner registry. Dedicated-block accounting uses actual acquired bytes,
alignment, allocator rounding, and unused tail. No fixed maximum number of
blocks or privileged arena container is needed.

## 5. Traversal, repair, and abandonment

### 5.1 Draining cursor

Construction invalidates prior element and structural facts and produces an
opaque cursor owning:

```text
source allocation
* already-yielded region vacant
* remaining drain region full
* retained suffix full
* exact final repair obligation
```

Each `next` takes one owner from the remaining drain region. Terminal `None`
does not by itself discharge suffix repair or allocation ownership. Normal
cursor abandonment invokes the static verified plan: destroy the unyielded
drain remainder, relocate or rejoin the retained suffix according to the
member contract, reseal the source, and return the parent authority. A cursor
whose repair can fail or suspend is `MustClose` instead.

No writer discipline or generic terminal-`None` rule is trusted.

### 5.2 Composed and non-lending cursors

A composed cursor predicate carries a finite tagged map from each child to its
source roots and footprints. Owned yields leave the cursor entirely. Borrowed
yields retain their declared external source lifetime; destroying the adapter
does not shorten a non-receiver-bounded result. Repeated unique yields coexist
only when cursor progression or a checked partition proves disjoint
footprints.

The candidate erases proof metadata and retains the selected adapter fields as
runtime state. Exact generated-state and operation parity remain P-1 evidence
obligations.

## 6. Shared ownership and dynamic borrowing

### 6.1 Strong and weak owner

An ordinary allocation uses the states:

```text
Live(s,w)       s > 0; payload live; implicit weak sentinel exists
Dropping(w)     one thread owns the last-strong disposal transition
PayloadDead(w)  payload gone; explicit weak handles may remain
Freed           no handles; allocation released
```

Strong clone increments `s` with checked overflow. Downgrade increments `w`.
Weak upgrade uses compare-exchange and succeeds only while `s > 0`; zero is
permanent. The last strong release transfers exclusive authority to
`Dropping`, takes and disposes the payload once, then removes the sentinel. The
shared protocol escrows root `ReleaseAuth` while any strong or weak handle
exists; the last weak transition recovers it, proves quiescence and vacancy,
and frees the carrier.

In the concurrent form, increments, decrements, upgrade, and last-owner
synchronization use the fixed atomic profile. The count protocol and disposer
are ordinary proofs; allocation and atomic instructions are gate entries.
Strong cycles are either an advertised leak contract, broken with weak edges,
or avoided through pools/handles.

### 6.2 Dynamic shared/unique borrow

The ordinary runtime state is:

```text
Idle | Readers(n) | Writer
```

Borrow creation validates and changes this state, then returns a guard owning
the exact permission to restore it. Reader split duplicates only the allowed
observation resource; mutable guard split requires exact disjoint footprints.
The guard's verified plan restores state on every normal abandonment edge.
Concurrent forms use atomics or a lock protocol under R-4.

This runtime state is the requested dynamic-borrow contract. Static borrowing
uses no state word or branch.

### 6.3 Stored borrows and runtime-selected behavior

An owner retaining `T` with borrow leaves is indexed by the borrowed roots and
cannot outlive their leases. Moving `T` into a carrier preserves each leaf's
root and permission; moving it out returns the same leaves. A verified plan
ends or returns every remaining leaf exactly once. Callable environments use
the same rule, so a producer or comparator may retain declared borrow state
without deriving provenance from its receiver, call frame, or container
storage. Purely static lifetimes erase.

A known behavior with no environment specializes to a direct call. Runtime
selection seals:

```text
exists E,p. CodeLease(p,C,E) * Environment(E) * InvokeAuth(p,E,C)
```

The contract `C` accounts for receiver mode, arguments, result owners, normal
and abort outcomes, reentry, effects, environment disposition, and code
lifetime. A closure collection or driver registry stores the code selector and
environment chosen by its representation. Unload is rejected while any code
lease exists. Dynamically loaded certified code passes the same final-image
predicate; an opaque plugin remains an exact `TRUSTED_FOREIGN` capsule. Static
users pay no dynamic state, while selected dynamic users pay their code/env
representation and indirect branch.

## 7. Weak-memory concurrency

### 7.1 Release/acquire publication

Let `P` own one non-atomic payload and let an atomic flag have protocol states
`Empty` and `Published(P)`.

The producer writes while owning `P`, then a release store transfers `P` into
the flag protocol. A consumer acquire load that reads from that release
receives the synchronized view and may recover `P`. A relaxed load does not
recover `P`; a subsequent payload read is rejected. Therefore sequential
ownership alone cannot silently authorize weak-memory publication.

The protocol and ghost transfer are ordinary. Atomic semantics and lowering
are fixed-root/gate responsibilities.

### 7.2 Lock and channel

A lock's atomic word owns an invariant containing the protected resource when
unlocked. Successful acquire compare-exchange opens that invariant and returns
the resource; release returns it before the release store. Failed retries
receive no protected authority. Parking and wakeup are exact admitted effects;
queue policy is ordinary.

A channel uses an ordinary queue invariant plus endpoint protocol. Send moves
the message resource into the queue; receive takes it. Disconnect and endpoint
disposition account every buffered message and wake lease. Session-style
protocols are derived predicates, not proof-kernel axioms.

### 7.3 Lock-free stack with safe reclamation

Nodes are stable carrier roots. Compare-exchange updates an atomic head under a
logical stack invariant. A successful push transfers the node owner into the
invariant; a successful pop transfers logical removal authority but cannot
immediately release the node while another thread may retain a read lease.

An ordinary hazard or epoch protocol records active guards and retired nodes.
Each active guard escrows the node's release/reuse authority through the
protocol. Reclamation obtains exclusive retired-node move/disposition authority
and `ReleaseAuth` only after a proof or checked scan shows no guard can access
the root/epoch. Then its verified plan destroys payload in place or takes it
when movable and releases the carrier. The hazard records, epochs, scans,
retries, and fences are algorithm-required. The candidate proof introduces no
additional runtime registry in this paper trace; exact P-1 parity remains
pending.

Lock freedom is separate from memory safety and requires an exact progress and
scheduler assumption. The proof cannot claim it merely from successful safety
checking.

## 8. Place identity, pinning, and self-reference

Create a stable heap, static, mapped, or lexically fixed carrier and initialize
every field that does not depend on the final address. Then consume relocation
authority and seal:

```text
Pinned(rho,epoch,S) =
  Stable(rho,epoch,S)
  * Full(rho,S)
  * escrow(MoveAuth(rho,S), ReleaseAuth(rho))
```

Internal references store the same root/epoch and exact offsets. Dereference
requires the live stability lease. Structural projection is a proved theorem
only when field offset and in-place destruction are preserved. Disposal
consumes the sealed pinned owner, receives restricted in-place destruction
authority rather than `take`, invalidates registrations and internal loans,
destroys fields at the same places, and retires the epoch. Only after every
lease ends does it recover `MoveAuth` and `ReleaseAuth` and release storage.

Moving a pointer-sized owner handle does not move its heap root. Moving inline
payload storage is rejected while pinned. An intrusive list owns pinned node
roots and ordinary link fields; membership is a protocol predicate and does
not mint node ownership.

This paper route uses no pin tag or mandatory extra allocation. A representation
choosing heap roots is intended to retain only the allocation and indirection it
selected for stability; exact P-1 parity remains pending. Stack/inline pinning
requires a lexical non-relocation proof.

## 9. External resources, partial I/O, and callbacks

### 9.1 Read into dead byte storage

An exact I/O entry may expose:

```text
read(fd, Vac(bytes,[0,m))) ->
    Progress(n,fd',Full([0,n)) * Vac([n,m)))   where 0 < n <= m
  | End(fd',Vac([0,m)))
  | Interrupted(fd',Vac([0,m)))
  | Error(fd',exact partial-progress state)
```

An ordinary `read_exact` loop carries `Full[0,k) * Vac[k,m)`, retries only the
declared interruption state, and returns either full ownership or the exact
partial prefix on failure. This avoids mandatory zero initialization while
never allowing an arbitrary safe callable to read dead input bytes.

A write borrows a live prefix. Partial write changes only the exact external
stream/handle state and returns the unconsumed suffix borrow/position.

### 9.2 Reentrant callback and retained pointer

The capsule contract states whether callbacks occur before return, after
return, or concurrently; the invoking thread; pointer escape; reachable
memory; partial effects; abort behavior; and fact attenuation. Ordinary code
must close its invariant before a reentrant call. A pointer retained after
return transfers a stable-place lease that escrows the exact move/release
authority to an external owner. Callback and retention completion return or
terminate that lease before in-place disposal. If the provider can retain the
pointer indefinitely or completion can fail, the wrapper remains `MustClose`;
ordinary abandonment is not a terminal outcome.

The wrapper proof is ordinary. The foreign behavior remains exactly the named
`TRUSTED_FOREIGN` assumption unless fully certified.

## 10. Async state machine and cancellation

An ordinary task owner uses states such as:

```text
Idle
Polling(exclusive_poll_lease)
Pending(resources,wake_registration,optional_external_operation)
Ready(result)
Cancelling(exact outstanding leases)
Terminal
```

The exact interface is:

```text
poll(Owner<State>, WakeLease)
  -> Ready(result, TerminalState, ReturnedOrConsumedWakeLease)
   | Pending(Owner<RegisteredStateContainingWakeLease>)

cancel(Owner<State>)
  -> Cancelled(TerminalState)
   | HandedOff(ExternalContinuationOwner, named_acceptor)
   | MustClose(CancelState)
```

`poll` consumes an exclusive poll lease. Every `Pending` return reseals all
resources and stores/replaces the wake registration exactly once. `Ready`
transfers the output once and returns or consumes the wake lease. Cooperative
cancellation begins only from a sealed state, never at an arbitrary instruction
inside open focus. `MustClose` is an obligation, not a terminal branch; only
closure or irreversible named transfer discharges it.

For an outstanding kernel operation, submission transfers a pinned buffer
lease, including the necessary move/release escrow, to the reactor/operation
owner. Cancellation has an exact capsule outcome:

```text
CancelledBeforeCommit
Completed(n)
PartiallyCommitted(n)
StillOwnedByExternalOperation
```

If cancellation synchronously returns every lease, the task may have a total
verified plan. Otherwise it is `MustClose`; the reactor retains the buffer and
task obligation until completion or acknowledged cancellation. Dropping an
owner never pretends that external use stopped.

Scheduler queues, futures, structured child ownership, wake policy, and the
task state machine are ordinary code. OS submission/cancellation, readiness,
park/wake, and clocks are exact gate entries. The route needs no language-level
suspension primitive or mandatory coroutine heap allocation.

## 11. Final-code closure

Each proof-bearing artifact binds the canonical checked-resource IR, contracts,
proof, disposition, target and memory profile, and every admitted dependency.
Closure requires an end-to-end refinement theorem or a machine-checked
per-artifact refinement certificate through optimization, object emission or
assembly, static/dynamic linking, relocations, import resolution, loader
mapping, executable immutability, and the exact post-load code/data image. The
validation route also binds provider identities, the complete dependency cone,
and a checked load receipt. Determinism or pre-link bytes alone do not suffice.

Dynamic libraries are checked compositionally at the load boundary. Generated
code must pass the same predicate before becoming executable. A valid source
proof followed by an unverified substituted backend is not a derivation.

## 12. Paper-witness result

The traces construct candidate routes for:

- full, dense-prefix, ring, gap, transient-hole, and arbitrary-sparse storage;
- heterogeneous/dynamic layout, zero-code partition reshape, full-owner
  adoption, core Copy, ZST, and type-preserving emplacement;
- dynamic growth, direct relocation, replacement, swap, and failure atomicity;
- recursive and multi-root ownership;
- recyclable stable identity;
- borrowing, cursors, drain repair, and exact abandonment;
- shared/weak lifecycle, stored/dynamic borrowing, and static/dynamic callables;
- release/acquire publication, locks, channels, and lock-free reclamation;
- stable address, self-reference, and intrusive membership;
- partial I/O, callbacks, pointer retention, async cancellation, and external
  resources; and
- proof-to-final-code binding.

No listed trace needs a privileged named container, writer-visible unchecked
assertion, hidden `Copy`/`Clone`/`Default` premise, universal runtime liveness
state, or a second admission path. Any use of core Copy is an explicit
`CoreCopy(T)` precondition; explicit Clone and producer calls retain their exact
contracts. This bounded witness set does not establish exhaustive derivability:
exact dense D-2 fails closed with 340 unique unresolved obligations across 150
contexts: 208 are Convert-implicated, 136 are allocator-implicated, and 12
concern ZST/fullness, with 16 in both the Convert and allocator sets. Exact P-1
remains pending. No listed trace has produced an architecture contradiction, but
exact enumeration can still falsify the candidate. Production
soundness, proof-generation practicality, code generation, and measured cost
also remain unproved and unauthorized.
