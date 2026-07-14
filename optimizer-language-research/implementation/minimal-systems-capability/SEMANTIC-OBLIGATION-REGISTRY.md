# Minimal Systems Capability Basis: Semantic Obligation Registry

Status: D12-authorized G0-Core research draft, 2026-07-14; not yet
owner-reviewed. This document is non-normative. It freezes no language
mechanism, syntax, standard-library type, compiler path, optimizer fact, family
protocol, benchmark, or production decision. Its purpose is to state the
semantic proof obligations that candidate mechanisms must discharge and to
prevent a convenient representation from being mistaken for a complete
capability basis.

The external source anchor is Rust 1.97.0, release tag `1.97.0`, peeled source
commit `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`. Rust supplies finite caller
needs and implementation witnesses, not default xlang mechanisms. The local
xlang authority remains `CONSTITUTION.md`, D10-D12 in
`optimizer-language-research/notes/user-directives.md`, and the current
specification. A conflict with those sources reopens this registry.

## 1. Registry conclusion

The minimum cannot be stated as one closed list of storage states plus one
closed list of access mechanisms. Those lists answer different questions.

- A storage topology describes which locations currently contain valid values.
- An ownership transition describes how values move between locations.
- A borrow discipline describes who may access those locations concurrently in
  the abstract machine.
- Exit, destruction, allocation failure, identity, refinement, and optimizer
  facts are independent obligations that can coexist with any storage topology.
- Encapsulation, static behavior dispatch, cursors, rebuild transactions, and
  checked storage objects are possible mechanisms. None is itself a proof that
  the obligations above hold.

The current full buffer, a dense initialized prefix, a ring, sparse occupancy,
and nested dependent prefixes are separating witnesses, not a proposed five-tag
runtime union. A general sparse bitmap could encode all of them, but imposing
that encoding on every contract would add metadata and access work where a
length, `head + len`, or node-local length already proves the live set. G0-Core
therefore freezes semantic laws and protected cost boundaries, while each
Family Lock A must select and price exact mechanisms for its family.

## 2. Abstract proof vocabulary

For an owning storage root `S`, the registry uses this abstract model only for
reasoning. It does not prescribe source syntax or runtime layout.

```text
State(S) = {
    allocation and capacity state A,
    live-location predicate Live,
    unique-owner relation Own,
    active borrow/provenance relation Borrow,
    logical identity relation Identity,
    value/refinement predicates Refine,
    optimizer facts Facts
}
```

`Valid(S)` means all of the following hold:

1. every location for which `Live` is true contains one fully valid value of
   its declared type and satisfies every active refinement;
2. every location for which `Live` is false is inaccessible as that type and is
   neither read nor dropped as that type;
3. every affine value has exactly one current owner;
4. all references and cursors satisfy their provenance, lifetime, and
   invalidation rules;
5. allocation, capacity, live-set, identity, and refinement metadata agree;
6. every optimizer fact is true for its recorded owner, version, and scope; and
7. destruction of the owner can identify exactly the values and allocations it
   owns without consulting writer-forgeable authority.

An abstract component may be erased from runtime representation when the
checker proves it from existing state. For example, `Live(i)` may reduce to
`i < len`, a ring interval predicate over `head` and `len`, or a node-local
dependent length. Erasure is earned by proof; it is not permission to omit the
semantic obligation.

## 3. Proof dimensions, separate from mechanisms

These dimensions are orthogonal accounting axes. A candidate may discharge
several with one construct, but it must not omit a row by renaming a mechanism.

| ID | Proof dimension | Question that must be answered |
|---|---|---|
| PD-VALID | Value validity | When do bytes constitute a fully valid `T`, including type invariants and refinements? |
| PD-LIVE | Live-set topology | Which locations are readable and droppable: full, prefix, ring, sparse, nested/multi-range, or a proved transition state? |
| PD-OWN | Ownership transition | How do initialize, move-in, move-out, replace, relocate, clone, swap, delete, and destroy conserve affine ownership? |
| PD-EXIT | Exit and abandonment | What valid state exists after every normal exit, early error, loop exit, and abandonment of an affine protocol value? |
| PD-DROP | Destruction | Which live values are dropped, which moved-from locations are not dropped, and which allocations are released exactly once? |
| PD-BORROW | Borrow and provenance | What root and epoch does an access derive from, which places are disjoint, and which mutation invalidates it? |
| PD-FAIL | Arithmetic, allocation, and failure | Which preparation steps can fail, where commitment occurs, and what ownership/state is returned on failure? |
| PD-IDENT | Logical and physical identity | Are logical identity, pool provenance, temporal freshness, payload lifetime, allocation lifetime, and address stability promised separately? |
| PD-REFINE | Refinement validity | Which checked constructor establishes a predicate such as UTF-8, and which mutations preserve or invalidate it? |
| PD-BEHAVE | Callable behavior | How are equality, hashing, ordering, cloning, and callbacks invoked, and how are broken laws contained to logic errors? |
| PD-FACT | Optimizer fact integrity | Which checked transition proves a proposition, what scopes it, and which event invalidates it? |
| PD-COST | Structural performance | What allocations, initialized bytes, moved bytes, metadata, checks, branches, code size, and protected-baseline tax are intrinsic? |

The following are mechanism categories, not proof dimensions and not selected
designs:

| Mechanism category | What it could contribute | What it does not prove alone |
|---|---|---|
| Opaque representation or private fields | Prevent direct client forgery | Preservation by library methods, exact drop, failure atomicity, or fact validity |
| Checked storage-state object | Centralize live-set transitions | Borrow scheduling, identity policy, behavior laws, or optimal representation for every topology |
| Eager atomic operation | Hide an invalid intermediate state | Generativity for operations outside the fixed operation set |
| Scoped rebuild/relocation transaction | Express source/destination and transient holes | Safe abandonment unless exact-use or cleanup is separately established |
| Reborrow, entry, or cursor | Schedule repeated or multi-place access | Allocation, initialization, relocation, destruction, or stable identity |
| Static behavior contract | Permit direct monomorphized calls | Initialization or metadata/payload coherence; truth of open-world algebraic laws unless checked |
| Generative brand or capability token | Separate authority from data | Individual reclamation, arbitrary multi-place mutation, or abandonment cleanup |
| Generational or provenance-bearing handle | Reject some stale or wrong-owner accesses | Infinite freshness with finite bits and peak-live-only memory |
| Refinement seal | Establish a predicate after validation | Preservation by later mutations or live-slot state |
| Compiler-derived cleanup | Repair or finish a protocol on exit | Permission under current STOR-3, cost, recursive termination, or trap cleanup |

No mechanism category above is authorized by this registry.

## 4. Global semantic laws

These laws apply to every later family whose contract implicates them. Family
Lock A may instantiate or strengthen a law; it may not weaken one silently.

### G-1: Boundary validity

Every owning abstraction has a stated invariant. Construction establishes it.
Every public operation and every non-trap exit leaves every reachable owner in
a valid state. A temporarily invalid state must be wholly contained inside one
checked transition or inside a scoped protocol whose exit theorem establishes
validity on every normal path.

### G-2: Live/read/drop equivalence

A location may be read, borrowed, pattern-matched, compared, cloned, formatted,
or dropped as `T` only when checked state proves that location live and proves
all validity predicates of `T`. A dead, vacant, spare-capacity, moved-from, or
partially initialized location is never exposed as `T`. Writer-visible raw
uninitialized payload access, unchecked `set_len`, unchecked `mark_full`, or an
equivalent split privilege is inadmissible.

### G-3: Ownership conservation

Initializing or moving into a location creates exactly one destination owner.
Moving out or relocating kills exactly one source owner before any source drop
can occur. Replace and swap conserve the complete multiset of affine values.
Clone is a distinct explicit semantic operation and may fail only according to
its frozen contract. No operation acquires a hidden `Copy`, `Clone`, `Default`,
or dummy-value requirement.

### G-4: Normal-exit closure

Fallthrough, `return`, `break`, `give`, `try` propagation, recoverable
allocation/capacity failure, callback return, and permitted protocol-value
abandonment are normal exits. Every one must satisfy G-1 through G-3 and the
family's resource-accounting contract. Proof of the success path alone is not
coverage.

### G-5: Affinity is not completion

An affine value cannot be duplicated, but current xlang affinity permits it to
be abandoned. Therefore an affine cursor, guard, or rebuild token is not a
must-finish proof. A candidate that requires completion must do one of the
following and price that choice explicitly:

1. commit the base owner to a valid state before the token can be abandoned;
2. introduce a statically exact-use/linear obligation on every normal path; or
3. introduce compiler-owned derived cleanup with fully specified exit and drop
   semantics.

Reliance on a writer remembering to call `finish` is rejected.

### G-6: Exact normal destruction

On every normal owner-destruction edge, every live affine payload is dropped
exactly once, every dead or moved-from location is dropped zero times, and every
owned allocation is released exactly once. Partial destinations drop only the
live subset. Destruction authority comes from checked state, not from mutable
client metadata. Recursive destruction must meet the family's termination and
stack/resource bound.

### G-7: Trap boundary

Under current EFF-4, a trap aborts and runs no cleanup. A trap path owes no
recoverable post-state and no drop execution, but it must not read an invalid or
uninitialized value, double-drop, use a dangling reference, or otherwise enter
undefined behavior before abort. No-unwind removes unwind edges; it does not
remove the normal exits listed in G-4.

### G-8: Recoverable failure preservation

Checked capacity arithmetic and every recoverable allocation occur before a
destructive commitment, unless rollback is proved. On failure, the original
owner, its values, and any offered affine input retain the exact ownership and
observable state promised by the operation. After commitment, remaining steps
must be infallible or carry a proved rollback state machine. Current OOM is a
TCB-level condition under OP-9; a recoverable allocator is a separate contract
and cannot be assumed by a family without its own lock.

### G-9: Borrow, provenance, and invalidation

Every physical reference identifies an owning root, place, region, and any
state/version required to prove continued validity. A live incompatible borrow
blocks relocation, deletion, owner destruction, and any mutation that can
invalidate its place. Multiple exclusive positions require a checked
disjointness proof. A logical handle or index is not a physical reference; it
must revalidate the identity and occupancy promised by its contract at access.

### G-10: Identity dimensions remain distinct

Logical object identity, physical address stability, pool provenance, temporal
freshness, payload lifetime, allocation lifetime, and pre-invalidation
notification are separate contract dimensions. A mechanism solving one does
not inherit the others. Append-only and recyclable handles remain distinct
contracts, and the protected append-only path pays no generation or recycling
tax.

### G-11: Finite copied-handle boundary

Freely copied finite-width handles cannot simultaneously promise all three of:

1. indefinite slot reuse;
2. memory bounded only by peak live population; and
3. permanent rejection of every stale handle.

After enough allocate/delete cycles, a finite handle representation repeats;
the lookup cannot distinguish the new object from an old handle with identical
bits. A recyclable family must freeze which guarantee is relinquished through
exhaustion/trap, retirement/history growth, unbounded identity, or static
revocation. Silent wrap and identity resurrection are forbidden.

### G-12: Refinements are state

Initialization alone does not prove semantic validity such as UTF-8, sortedness,
heap order, or tree balance. A checked producer establishes each refinement,
and every mutating transition must preserve it or invalidate the corresponding
capability/fact. An invalid behavior law may cause a contained data-structure
logic error, but it must not authorize an invalid payload read or memory
corruption.

### G-13: Fact-channel integrity

Only a checked transition or machine-verified proof may mint a fact that lets
the optimizer assume more. Facts are tied to an owner, proposition, provenance,
and state version; every relevant mutation invalidates them. Metadata cannot be
forged independently of the payload state it authorizes. The facts-off program
must preserve identical language semantics, differing only in optimization.
Every new fact channel receives hostile review before shipping; green tests are
not a soundness review.

### G-14: Ordinary-library generativity

A capability counts only when an ordinary no-unsafe library can derive an
assigned held-out storage-bearing structure from public checked mechanisms.
Standard-library-only raw access, container-specific compiler recognition, or
calling the corresponding completed container fails this test. A sealed
standard-library implementation remains a useful witness, not an extra
privilege tier.

### G-15: Protected no-tax paths

The existing fixed, fully initialized `buffer<Copy T>` path and the protected
append-only SoA/index path must not acquire spare-capacity initialization state,
sparse occupancy, generation fields, reference counts, identity checks,
recycling branches, or unrelated metadata. A family may pay state and checks
required by its stronger contract; it may not charge them to a weaker protected
contract. Proof state should erase when it is derivable from existing fields.

### G-16: Scope honesty

A closure claim names the payload, lifetime, ownership, failure, concurrency,
address-stability, and platform envelope it actually proves. An excluded
dimension is a recorded scoped deferral with a blocked claim and reopening
trigger, not an implicit success.

## 5. Separating Rust 1.97.0 witnesses

These source witnesses establish independent obligations. They do not select
Rust's unsafe implementation techniques for xlang.

| Witness | Exact source evidence | Separating result |
|---|---|---|
| `MaybeUninit<T>` and partial arrays | [`maybe_uninit.rs` validity invariant](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/mem/maybe_uninit.rs#L7-L16), [invalid-byte rules](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/mem/maybe_uninit.rs#L42-L93), [partial-array accounting](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/mem/maybe_uninit.rs#L119-L164), and [`write_iter` guard](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/mem/maybe_uninit.rs#L1385-L1405) | Invalid bytes are not `T`; partial construction needs a live-count proof and exact partial drop. `MaybeUninit` is implementation evidence, not an admissible xlang writer surface. |
| `Vec<T>` | [`vec/mod.rs` representation guarantees](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs#L303-L410), [`push`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs#L1035-L1049), [`set_len`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs#L2138-L2222), and [`pop`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs#L2843-L2852) | Optimal growable affine sequence requires spare capacity that is not `T`, ordered state updates, move-out, relocation, and failure preservation. A fully initialized buffer cannot derive this contract for arbitrary `T` without dummy construction or hidden requirements. |
| `Vec::Drain` | [`Drain` creation commits a shortened base vector](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs#L2942-L3003) and [`Drain::drop` repairs the tail](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/drain.rs#L173-L220) | Partial consumption exposes the difference between an always-valid base owner, cleanup/liveness restoration, and a must-finish protocol. |
| `VecDeque<T>` | [`VecDeque` ring representation](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/collections/vec_deque/mod.rs#L75-L116) | A wrapped live set of at most two intervals is neither one prefix nor optimally represented by arbitrary per-slot occupancy. |
| `BTreeMap<K,V>` nodes | [`node.rs` invariants and layout](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/collections/btree/node.rs#L1-L93), [`split`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/collections/btree/node.rs#L1221-L1307), and [`merge`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/collections/btree/node.rs#L1392-L1454) | One node has dependent prefixes of `n` keys, `n` values, and `n+1` edges; split and merge require source/destination transition proofs across allocations. |
| `hashbrown` raw table used by `std::collections::HashMap` | Rust 1.97 pins `hashbrown` 0.17.1; exact source commit [`c62a63a...`, table and control state](https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs#L557-L580), [`prepare_insert_index`](https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs#L1907-L1919), and [`set_ctrl`](https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs#L2564-L2595) | Sparse control state authorizes payload access. Split control/payload mutation is a temporal unsafe obligation in Rust and must become one checked transition or an unforgeable relation in xlang. |
| `String` | [`String` is a `Vec<u8>` plus an always-valid UTF-8 invariant](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/string.rs#L114-L117), [`from_utf8`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/string.rs#L515-L573), and unchecked-construction consequences at [lines 993-1005](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/string.rs#L993-L1005) | Live-byte topology does not imply a semantic refinement. Validation/sealing and boundary-preserving mutation are separate obligations. |
| `Rc<T>` and `Weak<T>` | [`RcInner`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/rc.rs#L285-L289), [`new_cyclic_in`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/rc.rs#L870-L910), [`try_unwrap`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/rc.rs#L1054-L1067), and [`Weak::upgrade`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/rc.rs#L3545-L3558) | Payload-live and allocation-live are distinct states; weak identity can remain after the payload is dead. This is not solved by unique-owner stable handles. |
| `Arc<T>` and atomic weak ownership | [`ArcInner`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/sync.rs#L388-L397), [`Arc::clone`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/sync.rs#L2399-L2432), [`Arc` destruction](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/sync.rs#L2827-L2875), and [`Weak::upgrade`](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/sync.rs#L3276-L3304) | Shared lifecycle becomes a synchronization proof: atomic memory order must publish initialization, prevent resurrection from zero, and order last-owner destruction. Unique-owner results cannot be generalized to concurrency. |
| `Pin<T>` and intrusive structures | [`pin.rs` address-stability contract](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/pin.rs#L1-L28) and [drop guarantee](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/pin.rs#L513-L586) | Logical identity across relocation is weaker than a stable address plus notification before storage reuse. |
| `RefCell<T>` guards | [`RefCell` state](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cell.rs#L844-L858), guard release at [lines 1552-1603](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cell.rs#L1552-L1603), and leaked-guard behavior at [lines 1814-1821](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cell.rs#L1814-L1821) | Memory safety can survive abandonment while usability or capacity is permanently lost. Safety, exact cleanup, and liveness are separate contract choices. |
| `mem::forget` and `ManuallyDrop<T>` | [`mem::forget` safety rationale](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/mem/mod.rs#L73-L170) and [`ManuallyDrop` validity and derived-behavior hazards](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/mem/manually_drop.rs#L8-L149) | Rust unsafe code may not rely on destructor execution, and derived behaviors must not inspect dead fields. xlang may deliberately require stronger exact normal-exit cleanup, but that is an explicit xlang law, not an inference from Rust. |
| `LinkedList<T>` | [`LinkedList` node topology](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/collections/linked_list.rs#L50-L65) and [cursor operations](https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/collections/linked_list.rs#L573-L610) | Doubly linked mutation combines stable nodes, non-owning links, multi-place access, deallocation, and cursor validity; unique recursive ownership alone does not cover it. |

These witnesses establish at least four independent lower bounds:

1. arbitrary affine spare capacity requires a state in which storage exists but
   no `T` exists there;
2. optimal topology-specific access cannot generally be reduced to one sparse
   per-slot representation without tax;
3. operation correctness depends on transition and exit states, not only
   steady states; and
4. identity, refinements, shared lifecycle, and address stability are
   independent of live-slot topology.

## 6. Current xlang evidence and gaps

This table records current evidence, not intended future semantics.

| Area | Current evidence | Current gap or limit |
|---|---|---|
| Fixed storage | TYPE-2 defines affine, fixed-length, fully initialized `buffer<T>` with Copy-only elements; OP-9 checks allocation-size overflow. | No spare-capacity state, affine element storage, move-out, replace, relocation, or growth. |
| Destruction | STOR-3 surfaces compiler-derived drops/releases on normal region exits and forbids user finalizers and reference counting. | Current buffer destruction frees one allocation; it has no checked dynamic live-set from which to drop an affine prefix or sparse set. |
| Trap behavior | EFF-4 aborts without unwind or cleanup. | No-unwind does not solve early normal exits or abandoned affine protocol values. |
| Ownership | OWN-1 provides explicit affine moves, whole-binding death after partial move, and no reinitialization of a dead binding. | The current rules cannot express a temporary hole, partial destination, or subsequent restoration of the same owner binding. |
| Disjoint access | OWN-5/6 track resolved-place exclusivity. | OWN-7 proves dynamic indexed positions disjoint only when both are unequal literals; general swap, heap repair, and graph rewiring remain blocked. |
| Borrow reuse | Reborrow-through-holder and result-reborrow directions are recorded. | They are not implemented, and no complete entry/cursor escape and invalidation proof exists. |
| Encapsulation and behavior | Contracts and monomorphization exist in the grammar; the intended env-struct route promises direct calls. | Modules, private representation, inherent implementation blocks, complete callable static members, and dynamic/open behavior are not established. |
| Collection direction | STOR-1 says growable/keyed collections are future libraries over buffers and structs. | This is a direction, not a derivation; required checked storage transitions do not exist. |
| Protected performance | Fixed Copy buffers are a selected baseline; the P2 append-only SoA/index pattern has measured evidence. | A universal occupancy/generation substrate would risk charging both baselines for stronger contracts they do not promise. |
| Production workload | xlc uses fixed-capacity SoA tapes; baseline token and AST live prefixes occupy about 20.54% and 10.26% of allocated capacity. | This establishes demand for unknown final lengths, not a selected growable representation or timing result. |
| Fact channels | Checked algebraic laws, effect rows, borrow-derived alias facts, and checked `requires` facts already demonstrate proof-to-optimizer flow. | No initialization, occupancy, refinement, handle-freshness, or metadata/payload fact channel is approved. |
| Failure | Checked size/capacity arithmetic can trap; current OOM is TCB-level. | Recoverable allocation, failure-atomic growth, partial clone, rehash, node split, and offered-affine-input preservation remain unproved. |
| Payload scope | Current buffers admit Copy, region-free payloads only. | Borrow-bearing affine payload storage and relocation require a separate lifetime and invalidation proof. |

The current evidence supports the protected full-buffer and append-only paths.
It does not establish dense affine sequence, ring, sparse occupancy, tree-node,
recyclable identity, text refinement, or shared-lifecycle capability.

## 7. Normal exits, no unwind, and affine abandonment

The following classification is global and must be instantiated by every
owning family:

| Event | Classification | Required result |
|---|---|---|
| Fallthrough, `return`, `break`, `give`, `try` propagation | Normal exit | All reachable owners valid; exact normal destruction for owners leaving scope |
| Recoverable capacity/allocation failure | Normal result path | Frozen failure state and ownership, normally preservation before commitment |
| Callback returns an error or stops iteration | Normal result path | Valid owner and exact accounting for consumed/unconsumed values |
| Affine token or cursor is abandoned | Normal path under current affinity | Base owner already valid, or an approved exact-use/derived-cleanup rule proves repair |
| Owner leaves scope | Normal destruction | Drop exactly live payloads and free each allocation once |
| Trap | Abort, no unwind, no cleanup | No recoverable post-state; no invalid read, dangling access, or double-drop before abort |
| Current OOM | TCB-level condition | Not silently reclassified as recoverable allocation |

Three policies must not be conflated:

- **memory safety:** abandonment cannot expose invalid memory;
- **resource exactness:** normal exits release or transfer every owned resource
  according to the frozen xlang contract; and
- **continued usability:** abandonment may or may not restore capacity, unlock a
  logical borrow, or preserve every element.

Rust permits safe resource leaks and permanently leaked `RefCell` borrow state;
xlang's current STOR-3 instead intends compiler-derived exact normal-exit
destruction. A family must state which usability guarantees accompany that
stronger resource rule. It may not derive them merely from the word `affine`.

## 8. Fact-channel registry schema

Every proposed initialization, occupancy, refinement, identity, or
metadata-to-payload relation that could remove a check or permit a load must
receive a row with all fields below before implementation. Blank or
"implementation-defined" fields fail the schema.

| Field | Required content |
|---|---|
| Fact ID | Stable identifier and schema version |
| Proposition | Exact machine-checkable statement, including quantified indices and value-validity predicate |
| Root | Owning allocation/abstraction to which the fact belongs |
| Producer | Checked transition or verified proof that establishes the proposition |
| Preconditions | Prior state, ownership, borrow, arithmetic, and refinement requirements |
| Scope | Region/control-flow dominance plus owner provenance and state version/epoch |
| Consumers | Checker and optimizer operations permitted by the fact, including whether it authorizes a payload load, check elision, alias metadata, or direct call |
| Invalidators | Every mutation, move, relocation, deletion, rehash, clear, reset, shrink/regrow, behavior call, or owner transition that can falsify it |
| Transfer rule | Whether and how the fact moves across borrow, move, return, call, branch join, and monomorphization |
| Speculation rule | Whether the backend may speculate the authorized operation; payload reads require validity before the load, not merely before use of the loaded result |
| Facts-off semantics | Same source-level result, trap behavior, ownership, and valid-memory accesses with the optimization fact disabled |
| Artifact evidence | Surfaced producer, proof/dependency identity, consumer sites, and invalidation accounting |
| Negative canaries | Forged metadata, stale version, wrong owner, mutation-after-proof, payload/control disagreement, non-live SIMD lane, and branch-join mismatch as applicable |
| Hostile review | Independent review scope, exact artifact hash, findings, and disposition before shipping |

Example proposition, not an approved fact:

```text
Full(S, i, version_v)
    => allocated(S, i)
    && live(S, i)
    && valid_T(payload(S, i))
    && metadata_version(S) == version_v
```

A control byte and payload write cannot be two independently callable public
steps if the intermediate `Full && !valid_T` state is observable or optimizer-
readable. Likewise, an optimizer may not load every candidate payload lane and
mask non-`Full` results afterward: the validity proof must dominate each load.

The fact schema follows the project's existing rule that safety proofs and
optimizer facts share one base. Formal and empirical precedent reinforces the
need for exact scope and invalidation: [Stacked Borrows](https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf)
models alias provenance needed by optimizations, while
[Alive2](https://web.ist.utl.pt/nuno.lopes/pubs.php?id=alive2-pldi21) found
optimizer errors through formal refinement checking. Neither source authorizes
a particular xlang model.

## 9. Candidate proof and derivability criteria

A Family Lock A must turn applicable criteria into exact fixtures and proofs.
The criterion names are global; the concrete state machine is family-local.

1. **Initialization theorem:** every constructor establishes `Valid(S)`.
2. **Transition preservation:** for every checked transition `op`, its stated
   precondition plus `Valid(S)` implies valid returned owners and post-state on
   every normal result.
3. **Slot ledger:** every physical slot has a unique abstract live/dead state;
   reads and drops occur if and only if live; ownership is neither duplicated
   nor lost.
4. **Exit closure:** every non-trap control-flow exit, including early errors
   and affine-token abandonment, is enumerated and proves G-4 through G-6.
5. **Failure refinement:** each recoverable failure point refines the frozen
   failure contract and accounts for the original owner and every offered
   affine value.
6. **Borrow theorem:** every reference or physical cursor has a root,
   provenance, and validity interval; invalidating transitions reject while it
   is live; multi-place unique access proves disjointness.
7. **Identity theorem:** stale, cross-pool, relocated, cleared, reused, and
   exhausted identities behave according to the exact contract; no identity
   silently resurrects.
8. **Refinement theorem:** every refinement producer validates its predicate;
   every admitted mutation proves preservation or invalidates the seal.
9. **Behavior containment:** a comparator, hash, equality, clone, or callback
   cannot forge storage validity or turn a logic-law violation into memory
   corruption.
10. **Fact theorem:** every fact has a producer, proposition, scope,
    invalidators, and consumer; facts-on and facts-off are semantically
    equivalent.
11. **Destruction theorem:** normal destruction drops exactly the live payloads,
    skips dead/moved locations, frees each allocation once, and meets recursive
    destruction bounds.
12. **Ordinary-library derivation:** H-STORE implements the implicated storage
    topology using only its frozen public dependency budget, with no completed
    container or privileged raw path.
13. **Structural-cost derivation:** the implementation meets the frozen
    asymptotic contract and accounts allocations, initialized/touched/moved
    bytes, metadata, checks, branches, drops, and code size before timing.
14. **Protected-path theorem:** B-FIX and B-P2 acquire no state, metadata,
    branch, or code-shape change belonging only to stronger contracts.

[RustBelt](https://plv.mpi-sws.org/rustbelt/popl18/paper.pdf) provides the
general precedent that a safe library surface requires semantic verification
conditions for its privileged substrate. [Oxide](https://arxiv.org/abs/1903.00982)
shows that initialized, dead, and maybe-dead states can be represented in a
formally safe core. [Vault](https://www.microsoft.com/en-us/research/wp-content/uploads/2001/05/pldi01.pdf)
shows why exact-use keys and typestate are relevant when affinity does not
guarantee protocol completion. These are proof techniques and mechanism
evidence, not xlang selections.

[GhostCell](https://plv.mpi-sws.org/rustbelt/ghostcell/paper.pdf) demonstrates
zero-runtime-cost separation of data and permission through generative brands
for arena graphs, while also documenting limits on simultaneous interior
access and topology proofs. It therefore remains a useful generativity witness,
not evidence that one branded token solves storage, reclamation, and arbitrary
graph mutation. [CETS](https://llvm.org/pubs/2010-06-ISMM-CETS.html) demonstrates
that temporal identity can be enforced dynamically, but its reported overhead
also confirms that identity checks are a priced contract rather than a free
property of every pointer.

A 2026 primary preprint on verifying the Rust standard library reports that
properties such as complete `MaybeUninit` initialization and pointer provenance
require reasoning at internal program points rather than boundary contracts
alone. That supports an explicit transition proof, not any Rust representation.
[Verifying the Rust Standard Library](https://arxiv.org/abs/2606.17374)

## 10. Scoped deferrals and blocked claims

The detailed first closure is the sequential, unique-owner data-structure
floor. The following domains remain accounted but are not silently inherited by
that closure.

| Domain | G0 disposition | Claim blocked until a later lock closes | Reopening trigger |
|---|---|---|---|
| Borrow-bearing stored payloads | Explicit scope decision required for each family; initial experiments may use region-free, borrow-free `T` only | General storage/relocation for values containing borrows | A candidate stores, moves, returns, or destroys a borrow-bearing payload |
| Shared ownership and weak identity | Deferred separate lifecycle family | `Rc`/`Weak`-class payload/allocation lifetime, cycle policy, and O(1) share/clone | Any candidate relies on shared ownership, weak references, or payload-dead/allocation-live state |
| Concurrency and atomic sharing | Deferred systems family | Data-race freedom and `Arc`/atomic memory-order contracts | Cross-thread sharing, atomic reference counts, concurrent collection access, or parallel reclamation |
| Pinning and address-sensitive values | Deferred address-stability family | `Pin`-class immobility, intrusive links, and notification before invalidation/reuse | A contract promises stable physical address or contains self/address-sensitive links |
| User-defined finalization | Not present under STOR-3; derived cleanup remains a priced candidate only where locked | Arbitrary RAII/finalizer protocols | A candidate requires writer-defined cleanup, repair-on-drop, or observable destructor behavior |
| Recoverable custom allocation | Current OOM remains TCB-level; later allocator family required | Allocator-parameterized containers and recoverable OOM | A family exposes allocator behavior, failure injection, placement, or deallocation policy |
| Resources and FFI | Deferred boundary family | Files, sockets, foreign ownership, callbacks, and ABI lifetime guarantees | A candidate owns a non-memory resource or crosses a foreign boundary |
| Async and cancellation | Deferred control/lifecycle family | Cancellation-safe partial operations and task-local destruction | Suspension, cancellation, or resumable protocol state enters a contract |
| Complete text and Unicode | Byte builder and UTF-8 sealing may close separately; full text remains deferred | Unicode segmentation, normalization, locale, and OS-string contracts | A family claims more than byte storage plus explicitly frozen UTF-8 operations |
| Target intrinsics and SIMD surface | Accounted as compressed later target families | General architecture-intrinsic capability | A selected family requires writer-visible target intrinsics rather than ordinary lowering |
| Dynamic dispatch and open-world behavior | Not established as first-floor requirement | Trait-object/vtable-class open behavior | A required contract cannot be derived through closed/static behavior and monomorphization |
| Cyclic tracing or garbage collection | Not selected; separate ownership family | General tracing/cycle collection | A mandatory topology cannot be expressed under unique, arena, or explicit shared ownership |

Closing a row requires its own contract and evidence gate. Using a deferred
domain inside an earlier candidate reopens every implicated family lock before
implementation or exposure of the shared mechanism.

## 11. What G0-Core and Family Lock A may freeze

| Subject | G0-Core may freeze | Family Lock A must freeze |
|---|---|---|
| Proof obligations | Global laws G-1 through G-16 and criterion schemas | Exact invariant, state machine, pre/postconditions, and proofs for family `F` |
| Coverage | B/M/W/H/O roles, coarse caller contracts, family dependencies, holdout identities and budgets | Exact operations, canaries, payload scope, and implicated witnesses for `F` |
| Mechanisms | Categories and cost axes only; no selection or privileged path | Exact candidate mechanisms and reference algorithms to compare |
| Language surface | META-5 accounting fields only | Every proposed spelling, grammar/type/ownership/effect/drop rule, diagnostic, and lowering obligation |
| Failure and exit | Global normal-exit, trap, abandonment, and recoverable-failure laws | Exact allocation policy, commitment points, rollback/return ownership, callback behavior, and cleanup policy |
| Facts | Fact-channel schema and hostile-review requirement | Exact fact proposition, producer, scope, invalidators, consumers, artifact form, and negative fixtures |
| Performance | Protected no-tax laws, asymptotic/coarse structural contracts, same-shape/end-to-end control schemas | Algorithms, capacity policy, allocator, payloads, traces, targets, structural thresholds, timing margins, endpoints, and selection rule |
| Soundness | Global attack and proof schemas | Exact executable fixtures and justification for every inapplicable attack |
| Deferrals | Domain identity, blocked claim, and reopening trigger | Any family-local narrowing, with precise excluded payload/contract |
| Adoption | Nothing beyond eligibility for owner discussion | Nothing automatically; closure permits a separate owner adoption request |

G0-Core must not freeze a storage spelling, builder token, finalizer rule,
uninitialized surface, standard container, private compiler opcode, candidate
algorithm, numeric threshold, or scored workload. Family Lock A must not claim
that a family-local mechanism solves an unlocked cross-family dimension.

Candidate Freeze B remains later still: it pins exact implementation hashes,
builds, environment, allowed corrections, and immutable scored inputs after
candidate construction and before scoring. Neither this registry nor G0-Core
authorizes Family Lock A, candidate construction, Candidate Freeze B, E0.1
restart, specification change, compiler implementation, xlc migration, scored
execution, production adoption, or default teaching.

## 12. Registry reopening conditions

This artifact reopens before further candidate work if evidence establishes any
of the following:

- a required proof dimension is absent or two recorded dimensions are not
  actually orthogonal;
- a new stable caller contract or held-out topology separates the current laws;
- a current xlang rule changes ownership, drop, trap, failure, borrow, or fact
  behavior relied on here;
- a proposed mechanism exposes state or privilege spanning unlocked families;
- a deferred domain becomes a dependency of an earlier family;
- a new fact channel or optimizer consumer appears;
- a protected baseline would pay a cost not recorded by G-15; or
- primary evidence invalidates a source claim or exact version pin.

Clerical source-link repair does not alter the registry semantics. Changing a
law, proof dimension, scope boundary, witness conclusion, or freeze boundary
does.

## 13. Claims not made

This registry does not establish:

- that the proof dimensions require the same number of language primitives;
- that dense, ring, sparse, and nested live sets require distinct public types;
- that Rust's `MaybeUninit`, destructors, traits, pointers, or unsafe internals
  are acceptable xlang mechanisms;
- that structural typestate, a checked storage object, a linear transaction,
  generative brands, or high-level atomic operations will win Family Lock A;
- that generational handles are the only recyclable identity design;
- that a finalizer or repair-on-drop rule is required;
- that the first unique-owner closure covers shared ownership, concurrency,
  pinning, resources, async cancellation, complete text, allocators, or target
  intrinsics;
- that xlc should replace its fixed SoA tapes;
- that any family is closed, implementable in current xlang, performance-
  optimal, or production-authorized; or
- that G0-Core completion authorizes anything beyond the next owner review.
