# Minimal static public capability basis

Status: D14 abstract basis recommendation for further research, 2026-07-15.
It assumes, but does not production-select, the hostile-reviewed sealed
compiler-embedded primitive registry. It selects no source spelling,
specification rule, implementation, standard library, container, or
experiment. Exact D-2 derivability and P-1 same-contract performance remain
fail-closed.

## Result

The smallest defensible basis has three different kinds of element. Treating
them as one list would either overprivilege ordinary libraries or hide required
language work.

1. **Three fixed storage leaves** define tag-free places, vacant/live ownership
   transitions, and recoverable physical heap roots.
2. **Six checked ordinary mechanisms** let unprivileged libraries seal and
   prove representation-selected invariants, finish or dispose affine
   protocols, preserve exact borrow provenance, call generic behavior, reason
   about sharing, and seal refinements. These mechanisms do not add machine
   semantics and ordinary formulas are never trusted.
3. **Exact machine/runtime leaves** cover address stability, atomics, threads,
   external events, target/device events, and executable-code activation. Each
   actual external or target event remains a separate fixed registry row; there
   is no generic syscall, opcode, ABI-contract, or device descriptor primitive.

This is one privilege-definition route with multiple independently irreducible
semantic leaves. The basis is smaller than the prior six-axis certified-
resource proposal: it does not require a public general final-code proof
language, arbitrary resource algebra, public external-frame formula, or
ordinary extension capsule. Final-code correctness remains a toolchain
obligation attached to every registry row, not a public capability.

## Fixed storage leaves

### P1. Checked carrier and place formation

Abstract contract:

- create a fresh, generative storage-root identity for inline storage or an
  already-owned physical carrier;
- validate a closed structural layout using checked size, alignment, offset,
  containment, and non-overlap arithmetic;
- form typed places and explicit padding over the exact footprint;
- split and join footprints without moving or reading payload bytes; and
- represent each typed place as `Vacant<T>` or `Live<T>` proof state without a
  mandatory runtime tag.

Ordinary current xlang cannot implement this contract because every array and
buffer slot is fully initialized, affine aggregate buffers are rejected, and
source cannot create uninitialized typed storage or generative place identity.
Inline-small storage, heterogeneous arenas, packed records, type-erased
allocators, header-plus-tail objects, and zero-sized logical places depend on
it.

The operation contains no collection topology, allocator policy, occupancy
bitmap, or user-defined validity rule. Layout input is a fixed compiler-checked
structural algebra, not an opcode or trusted contract. Naturally aligned
places have native layout; packed places permit checked by-value access but
cannot mint a misaligned borrow.

Runtime representation and cost: proof identities, footprint partitions, and
state evidence erase. Inline storage is exactly its selected payload/padding
layout. A physical carrier keeps its native pointer/extent/alignment metadata.
No per-place tag, branch, header, allocation, zero fill, or initialization
traffic is inherent.

Facts and invalidation: formation creates exact root, layout, alignment,
provenance, separation, and vacant-state facts. Reshape or carrier death retires
the place epoch and invalidates all derived places, borrows, and facts.

Removal witness: without P1, inline-small and partially initialized aggregate
storage requires preconstructed `T` values, a universal liveness tag, or an
extra heap object per element; heterogeneous and packed layouts require raw
pointer privilege. Each violates a registered contract or protected cost.

### P2. Exact place-state transition

Abstract contract:

```text
put:     Vacant<T> + own T -> Live<T>
take:    Live<T> -> Vacant<T> + own T
destroy: Live<T> -> Vacant<T>
```

Each transition is permitted only with unique authority to the exact place and
no incompatible borrow or stability lease. `put` makes the place readable only
after the complete value is installed. `take` ends source liveness before
returning the sole owner. `destroy` invokes the statically selected checked
disposition exactly once. Traps abort, but no dead place is read or destroyed
before the trap.

Current xlang kills an entire aggregate after a partial affine move and cannot
reinitialize it. Dense push/pop, sparse insert/remove, gap movement, node
extraction, array mapping, draining, and failure-atomic relocation depend on
P2.

Replacement, swap, range relocation, permutation, grow, shrink, and resize are
derived compositions of `take` and `put`; they are not separate primitives.
An optimizer may lower a proved trivial relocation/copy loop to `memmove` or a
target sequence without changing the public semantics.

Runtime representation and cost: P2 moves or destroys exactly one `T`; it adds
no tag, allocation, zero fill, clone, default construction, or dynamic dispatch.
The ordinary owner chooses compact counters, intervals, bitmaps, or other
metadata only when its representation needs them.

Facts and invalidation: transitions consume one state fact and create the
opposite fact; moves preserve every payload borrow leaf's original provenance.
Overlapping state, access, identity, and refinement facts are invalidated
exactly as declared.

Removal witness: without `put`, an affine value cannot enter spare storage;
without `take`, it cannot be returned without killing the whole owner; without
`destroy`, a dynamic live subset cannot be disposed exactly. Hidden
`Copy`/`Clone`/`Default`, full-capacity initialization, or whole-owner rebuilds
are the only simulations.

### P3. Recoverable physical root acquisition and empty release

Abstract contract:

```text
acquire(layout) -> Result<fresh empty carrier, AllocationError>
release(empty carrier) -> unit
```

All size and layout arithmetic is checked before the allocator event. Failure
returns no root and consumes no offered payload owner. Success returns one
fresh root whose complete footprint is vacant. Release requires the exact root,
no live place, no borrow or stability lease, and exactly one release authority.
An aborting allocation policy is an ordinary wrapper over the recoverable
contract.

Current `buffer_new` allocates a fully initialized Copy buffer and leaves OOM at
TCB policy; it cannot provide recoverable vacant affine storage. Growable
sequences, maps, pools, arenas, recursive nodes, and ordinary suballocators
depend on P3.

Resize is deliberately absent. Failure-atomic growth derives by acquiring a
new root before commit, moving live owners with P2, then releasing the old empty
root. Ordinary allocators may subpartition a large root and return checked child
owners; size classes, slabs, buddy trees, free lists, quarantine, and growth
policy remain ordinary library invariants.

Runtime representation and cost: one allocator acquisition or release event;
no mandatory zero fill or payload initialization. Allocator rounding and
alignment slack remain explicit structural costs. Zero-sized layouts may use a
fresh logical root with zero allocator calls and zero bytes.

Facts and invalidation: success creates root freshness, extent, alignment, and
vacancy facts. Release consumes them all. No allocator fact authorizes payload
validity.

Removal witness: existing full initialization forces dummy values, zero-fill or
clone traffic, and abort-only growth; per-object boxed simulation adds
allocations and indirections. Those violate dense, sparse, inline-small, and
recoverable-failure contracts.

## Checked ordinary mechanisms

These mechanisms are fixed checker rules available to every ordinary module.
They create no privileged machine action and therefore are not registry rows.
They are nevertheless required for derivability.

### Q1. Opaque generative resource sealing

An ordinary module may hide a runtime representation and a finite exact state
relation over P1/P2 resources. Every exported transition is checked against the
fixed ownership, place, and effect rules. User predicates and proofs can derive
only propositions in that fixed logic; they cannot add a primitive, fact kind,
effect kind, proof rule, lowering, foreign assumption, or runtime body.

This is the no-tax route for dense prefixes, rings, sparse occupancy,
cross-buffer coherence, logical identity, shared counts, and allocator
subdivision. Without Q1, the compiler must recognize every topology or every
place must carry universal runtime metadata. The former violates ordinary-
library generativity and the latter violates `NT-FIXED`, `NT-P2`, and multiple
held-out costs.

### Q2. Exact transition scope and static disposition

A transition scope must close on every normal edge by resealing a valid owner,
transferring all exact resources, or consuming them. A non-abandonable protocol
state is exact-use. An abandonable owner has one statically selected,
checker-verified disposition; no runtime vtable, callback registry, or policy
tag is implied.

This closes holes, drains, partial construction, cursor abandonment, dynamic
live-subset destruction, and recursive cleanup. Without Q2, affinity alone lets
a writer abandon an invalid intermediate state; requiring manual `finish` is
not a proof.

### Q3. Exact borrow-source maps and scoped disjointness

Every returned or stored borrow leaf selects an exact source root, place, and
region. Products and sum branches keep independent tags. Static structure,
cursor progression, or one checked runtime distinctness/partition validation
may create scoped disjoint subplace authority; pointer inequality never does.

This extends current lexical borrowing without runtime metadata on the static
path. It is required by `get`, mutable iteration, entry guards, arenas, graphs,
stored borrow-bearing payloads, and placement under existing borrows.

### Q4. Static generic and stateful callable contracts

Ordinary generic behavior has exact receiver, argument, result-provenance,
effect, failure, call-count, state-ownership, and disposition contracts.
Known implementations monomorphize to direct calls. Dynamic dispatch pays an
explicit code/environment/lease representation only when its contract selects
it.

This is required by equality, hashing, ordering, cloning, iteration adapters,
and cleanup behavior. Broken logical laws may affect results or complexity but
cannot mint storage, ownership, alias, or fact authority.

### Q5. Checked sharing and interference invariants

Thread transfer/shareability, atomic invariant opening, payload versus
allocation lifetime, reclamation, and progress assumptions are checked against
one fixed memory model. Ordinary modules can build reference counts, mutexes,
channels, once cells, and concurrent structures over P5/P6 below, but cannot
declare a race safe or weaken an ordering rule.

Static unique owners pay no count, atomic, borrow-flag, or reclamation tax.

### Q6. Checked refinement sealing

Validation or a proved preserving transition may attach a fixed refinement
predicate to live storage; every incompatible mutation invalidates it. A raw
byte owner pays no refinement metadata or validation. UTF-8 and validated
protocol values depend on Q6; arbitrary asserted refinements are rejected.

## Exact machine and runtime leaves

### P4. Stability and external-access lease

P4 escrows move/release authority for an exact place or root and yields a
scoped stable-place lease. Ordinary code receives no dereferenceable raw
pointer. An exact external or target registry row may consume the lease and its
declared buffer permissions. Ending all such uses returns the escrowed
authority; in-place destruction precedes root release.

Address-sensitive values, retained outbound buffers, self-reference, true
intrusive links, asynchronous I/O, and device access depend on P4. Without it,
they require permanent extra indirection, copying, or unchecked lifetime
claims. Contracts that need only logical identity do not pay stability cost.

### P5. Typed atomic events

P5 is the closed family of typed atomic loads, stores, exchanges,
compare-exchanges, read-modify-write events, fences, and wait/notify events with
fixed value domains and fixed memory-order meanings. Each operation is total or
reports its exact failure; no pointer or memory-order descriptor installs new
semantics.

Atomic cells carry only the representation selected by their atomic contract.
Unique non-atomic values pay no field or branch. Shared ownership,
synchronization, channels, concurrent reclamation, and device atomics depend on
P5. Removing it forces a privileged named lock/container or admits data races.

### P6. Thread runtime events

P6 contains separately specified spawn, join, park, unpark, thread-identity,
and thread-local lifetime events. Q5 proves transfer, sharing, and cleanup;
P6 supplies only the irreducible scheduler/runtime actions. Thread creation and
parking costs occur only on selected paths.

Removing P6 makes actual parallel execution and blocking synchronization
impossible even if atomics remain. Async state machines and channels remain
ordinary libraries over P5/P6 and exact I/O/timer leaves; executor policy is
not privileged.

### P7. Exact external resource events

P7 is a rule for a set of rows, not one generic operation. Every file, socket,
process, clock, timer, OS-handle, or outbound-ABI event that crosses the wall is
one exact registry row with fixed:

- resource types and ownership/lifetime transitions;
- buffer place, stability, and direction permissions;
- partial-progress, interruption, blocking, cancellation, and reentry behavior;
- normal, recoverable-failure, and abort outcomes;
- cleanup and close obligations;
- concurrency and foreign-retention scope; and
- conservative fact attenuation.

Ordinary wrappers derive buffering, parsing, retry, path policy, command
construction, DNS policy, formatting, and higher-level resource APIs. There is
no public syscall number, ABI signature, callback formula, memory contract, or
generic foreign-call descriptor. Removing an actual demanded row makes that
external event unavailable; merging rows with different effects or ownership
would be a false minimum.

The finite ledger proves category need for allocation, trap reporting, byte
I/O, filesystem, process, ABI, network, clock, thread/sync, async integration,
and build/target frames. It does not yet enumerate every target's exact P7 row,
so whole-envelope closure remains open.

### P8. Exact target and device events

Like P7, P8 denotes independently reviewed rows rather than a generic opcode.
Each SIMD, target-feature, virtual-memory, cache-control, MMIO, DMA, or device
operation fixes permitted widths, alignment, event count, atomicity/tearing,
ordering, speculation/duplication/elision rules, mapping lifetime, fault
behavior, and fact attenuation. Ordinary values cannot select a new instruction
or effect.

Portable scalar algorithms remain ordinary and pay no target tag or dispatch.
Removing a demanded row loses that target/device contract; a raw opcode or
address primitive would fail W3.

### P9. Executable-code activation and code lease

P9 validates and activates an exact immutable code image under one target and
ABI contract, returns an affine code lease, and permits an indirect call only
through the checked callable contract tied to that lease. Unload consumes all
leases. Writable and executable states never overlap unless a separately fixed
platform contract proves an equivalent safe transition.

JIT and loader held-outs depend on P9. Ordinary static calls and static generic
dispatch pay no lease, indirection, or activation cost. Removing P9 makes
dynamic code impossible; treating an arbitrary pointer as callable would fail
W3 and final-image integrity.

## Subsumption and rejected additions

- `replace`, `swap`, `relocate`, `resize`, drain repair, and permutation derive
  from P1/P2 plus Q1/Q2/Q3; they are not basis rows.
- `copy` and `clone` are not storage transitions. Bitwise copy requires a fixed
  compiler proof that the type is trivially duplicable; semantic clone is Q4
  behavior. Neither is implicit.
- Dense, ring, sparse, tree, map, heap, pool, graph, arena, string, iterator,
  allocator policy, mutex, channel, executor, buffered I/O, and retry policy are
  ordinary libraries, never registry rows.
- A universal runtime bitmap, place tag, refcount, stable address, dynamic
  dispatch object, or proof object is rejected because protected weaker shapes
  would pay for unselected contracts.
- A generic syscall, foreign signature, opcode, state machine, semantic
  descriptor, pre/postcondition, or proof rule is rejected because it lets
  ordinary data define privileged semantics.

## Minimality and remaining uncertainty

Every P row has a removal witness above. Q1 through Q6 also have removal
witnesses, but they remain checker/language research rather than privileged
machine operations. P7 and P8 intentionally refuse a misleading fixed count:
the authority count is the number of exact demanded external and target events,
not two generic back doors.

The basis is an abstract recommendation for derivability work, not a complete
design. Exact proof syntax and automation, P7/P8 target inventories, Q5 memory
model, P9 final-image validation, all 340 unresolved dense D-2 obligations, and
same-contract P-1 evidence remain open. No positive whole-systems or production
claim follows.
