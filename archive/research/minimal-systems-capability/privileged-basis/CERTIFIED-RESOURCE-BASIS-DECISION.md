# Certified Resource Basis Decision

Status: proposed D14 research architecture pending owner review, 2026-07-15.
This document defines the candidate semantic architecture to test for
derivability. Architecture-level paper witnesses support further study only.
Exact dense D-2 fails closed with 340 unique unresolved obligations across 150
contexts: 168 coarse Convert route-direction, 24 Convert callable-direction,
136 outer-allocator-applicability, six IntoOwner ZST/capacity-reshape, and six
IntoBoxed no-change fullness/ZST obligations. Equivalently, 208 are Convert-
implicated, 136 are allocator-implicated, and 12 concern ZST/fullness, with 16
in both the Convert and allocator sets. Exact P-1 remains pending. This document
authorizes no language syntax,
specification delta, compiler or verifier implementation, standard library,
container implementation, candidate execution, fact channel, or production
adoption.

## 1. Proposed architecture

Propose a two-plane architecture for owner review:

1. one sealed privileged admission verifier per fixed authenticated semantic
   toolchain root, as defined by `GATE-AUTHENTICATION-LOCK.md`; and
2. ordinary opaque proof-carrying resource-protocol modules checked against one
   fixed public reference policy.

The privileged plane admits only irreducible machine, backend, allocator, ABI,
OS, device, target, and foreign edges. The ordinary plane defines containers,
strings, shared owners, synchronization abstractions, state machines, resource
wrappers, and unforeseen data structures. A library proof is not privilege: it
may define predicates and derived lemmas, but it may add no axiom, instruction,
lowering, fact or effect kind, memory-model rule, or foreign assumption.

`LiveShape` is retained as a derived decidable automation fragment for common
full, prefix, interval, ring, gap, and transient-hole states. It is not the
universal semantic boundary. Universal runtime tags are an ordinary per-
representation choice, never a mandatory language tax.

This is the smallest surviving architecture among the evaluated candidates by
semantic authority, not by keyword count. Hiding independent obligations
behind one name would not make the basis smaller. No global minimality theorem
over every possible language architecture is claimed.

## 2. Why the back door is not a source keyword

The privilege gate has no ordinary-source spelling. A keyword, attribute,
module path, compiler flag, package name, or linker symbol that makes an
implementation privileged is forgeable by the writer or its dependencies.

An admitted entry instead contributes a compiler-synthesized checked callable
contract. Ordinary code invokes that contract through the normal call and
ownership rules. The readable operation name is not authority; the compiler
resolves it to the exact authenticated entry identity and applies the one use
predicate. Explicit runtime authority, such as a file handle or device lease,
is an unforgeable value passed through ordinary code.

The candidate is intended to avoid separate equivalents of Rust's `box`, lang
items, intrinsics, private standard-library syntax, and `unsafe`. It proposes:

- one source-inaccessible admission path for irreducible implementations; and
- one safe public resource kernel over which ordinary modules are proved.

This is an architecture hypothesis, not an exact D-2 conclusion. The unresolved
dense obligations must be split and discharged before the research can conclude
that no second privilege path or additional public authority is required.

A future surface form for an opaque resource module would be safe proof syntax,
not a privilege escape. This research does not select that syntax.

## 3. Proposed public semantic basis

The fixed policy has six independent semantic axes plus one uniform ordinary
proof mechanism. These are reference-policy obligations and need not become
six user-visible constructs.

| ID | Semantic authority | What it supplies |
|---|---|---|
| R-1 | Spatial resource | Generative byte carriers, checked structural layout/place witnesses, typed live/dead cells, exact footprints, separation, borrowing, reshape, and exactly-once release. |
| R-2 | Place identity | Logical `(root, epoch, offset)` identity, observable address, relocation and reuse invalidation, stable-place leases, projection, and in-place destruction. |
| R-3 | Control and lifecycle | Exact focus, normal/failure/abort outcomes, commitment, callbacks, abandonment, executable disposition, and explicit `MustClose` obligations. |
| R-4 | Interference and weak memory | Thread resource transfer, atomic events and orderings, data-race exclusion, invariant opening, reclamation, and exact progress assumptions. |
| R-5 | External frame | Exact nondeterministic partial progress, blocking, reentry, callbacks, pointer retention, cancellation, foreign state, effects, and trust attenuation. |
| R-6 | Final-code binding | Canonical checked IR, target and memory profile, verified or validated lowering, linking, relocation, imports, loading, and exact executable identity. |

The uniform proof mechanism supplies ordinary existential/generative
representation sealing, producer-defined predicates, inductive state,
conservative definitions, resource algebras, and derived lemmas checked against
the product reference policy. An admitted checked capability entry is an
ordinary callable instantiating one or more of R-1 through R-5, with R-6
binding its exact implementation and dependency cone. Its contract states
exact ownership, effect, fact, target, frame, failure, and cleanup behavior. It
supplies an irreducible action, not a second proof system.

### 3.1 Fixed reference resources

The underlying carrier is a byte extent, not a homogeneous array. A checked
layout witness creates typed logical places within that extent:

```text
ByteCarrier(k,rho,B,A)    sole root for B bytes with alignment A
Space(rho,F)              exact footprint F contains no live typed value
Place(rho,p,e,T,F)        checked T-place p, epoch e, exact footprint F
Vac(rho,p,e,T)            place p contains no live T
Full(rho,p,e,v : T)       place p contains exactly the sole owner v
MoveAuth(rho,S)           authority to reconfigure, relocate, or move out S
ReleaseAuth(rho)          authority to end the root after all leases end
P * Q                     exact separating composition
```

`rho` is fresh and existential. It is proof identity, not a writer-chosen name
or runtime field. `F` contains both the byte span and the logical-place
identity. Nonzero byte spans may not overlap in a simultaneous partition;
distinct zero-sized places remain separate even when their spans are empty and
their machine addresses compare equal. A homogeneous
`Carrier(k,rho,T,C)` below is shorthand for a checked array layout over one
`ByteCarrier` and its `C` place witnesses, not the universal representation.

A place witness is accepted only after checked size, storage alignment, access
profile, offset, provenance, containment, and partition arithmetic. Runtime-
length tails, header-plus-tail objects, heterogeneous arenas, tagged unions,
packed fields, and type-erased allocations use the same carrier with
representation-selected layout evidence.
A structural layout partitions every byte into typed places or explicit padding
`Space`; padding is never read as a value. Recomposition returns the same exact
partition authority.
A full place may not be reinterpreted. A vacant footprint may change layout
only through a proved partition bijection that retires the old logical epoch,
mints the new one, and invalidates every old place, borrow, and fact.

```text
carve<T>   : Space(rho,F) * Place(rho,p,e,T,F) -> Vac(rho,p,e,T)
uncarve<T> : Vac(rho,p,e,T) -> Space(rho,F)
```

These are proof-only ownership views. Arbitrary live bytes never become
`Full<T>` through layout evidence: only `put` of an owned valid `T`, checked
aggregate ownership decomposition, or an exact capsule fact proving the fixed
validity relation may establish liveness.

A naturally aligned place supports ordinary borrows. A packed or unaligned
place supports checked by-value load, store, take, and put but cannot mint a
misaligned reference; its exact target lowering may be an unaligned instruction
or a bytewise sequence. Atomic and volatile/MMIO access profiles additionally
require their R-4 or R-5 contract. Thus packed protocols and device layouts do
not require unchecked pointer casts, while aligned code pays no access-mode tag
or branch.

Carrier formation and disposition are conceptually:

```text
try_heap_bytes(B,A)
  -> Result<exists rho.
       ByteCarrier(heap,rho,B,A)
       * Space(rho,extent(B))
       * MoveAuth(rho,extent(B))
       * ReleaseAuth(rho), Error>

try_heap_layout(L)
  -> Result<exists rho.
       ByteCarrier(heap,rho,bytes(L),align(L))
       * Vac(rho,places(L))
       * MoveAuth(rho,places(L))
       * ReleaseAuth(rho), Error>

try_heap_empty<T>(C)
  -> Result<exists rho.
       Carrier(heap,rho,T,C)
       * Vac(rho,[0,C))
       * MoveAuth(rho,[0,C))
       * ReleaseAuth(rho), Error>

inline_empty<T,N>()
  -> exists rho.
       Carrier(inline,rho,T,N)
       * Vac(rho,[0,N))
       * MoveAuth(rho,[0,N))
       * ReleaseAuth(rho)

adopt_full<T,N>(own array<T,N>)
  -> exists rho.
       Carrier(inline,rho,T,N)
       * Full(rho,[0,N))
       * MoveAuth(rho,[0,N))
       * ReleaseAuth(rho)

adopt_full_storage<T,C>(own full_storage<T,C>)
  -> exists k,rho.
       Carrier(k,rho,T,C)
       * Full(rho,[0,C))
       * MoveAuth(rho,[0,C))
       * ReleaseAuth(rho)

export_full
  : Carrier(k,rho,T,C)
    * Full(rho,[0,C))
    * MoveAuth(rho,[0,C))
    * ReleaseAuth(rho)
    -> own full_storage<T,C>

release_empty
  : Carrier(k,rho,T,C)
    * Vac(rho,[0,C))
    * MoveAuth(rho,[0,C))
    * ReleaseAuth(rho)
    -> unit
```

Both adoption operations are erased ownership repackages. They neither read
bytes nor reinitialize or move payloads; the heap form preserves its exact
root, allocation, extent, and disposition. `inline_empty` is distinct because
it creates storage containing no `T`; conflating the two would recreate
`assume_init`.

The irreducible movable-payload transition is one isomorphism with two checked
directions:

```text
Full(rho,i,v)  <->  Vac(rho,i) * own v

take : Full(rho,i,v) * MoveAuth(rho,i)
       -> Vac(rho,i) * MoveAuth(rho,i) * own v
put  : Vac(rho,i) * MoveAuth(rho,i) * own v
       -> Full(rho,i,v) * MoveAuth(rho,i)
```

The operation additionally requires the carrier address authority and absence
of incompatible loans. A borrow escrows the overlapping `MoveAuth`. A stable,
external-retention, or reclamation lease escrows the exact move and/or release
authority its contract must prevent. Neither `take`, full export, nor root
release is possible until those leases return the authority. It never
fabricates, clones, or silently destroys `T`.

Pinned disposal does not use `take`: consuming the sealed pinned owner yields a
restricted in-place destruction authority, not relocation authority. The
statically selected disposition executes at the same place, consumes `Full`,
produces `Vac`, and ends every stability and external-access lease. Only then
are the escrowed `MoveAuth` and `ReleaseAuth` returned. Thus a pinned value is
never moved merely to destroy it, and its root cannot be released while any
address-bearing lease survives.

Range split/join divide and recombine both footprint state and `MoveAuth` as
proof equivalences. Existing checked borrowing applies to `Full`. Swap,
replacement, permutation, and relocation derive from `take` and `put`; they are
not independent primitives for movable places.

Structural reshape is a second proof equivalence, not a payload operation:

```text
reshape_partition(old_places, new_places, layout_bijection)
  : states(old_places) * MoveAuth(old_places)
    -> states(new_places) * MoveAuth(new_places)
```

The compiler checks that both partitions cover exactly the same carrier
footprint, preserve alignment and every live owner's type/layout and sole
ownership, and provide an exact old-to-new ownership mapping. The transition
retires the old logical epoch and invalidates old facts. It moves zero bytes.
It derives aggregate field opening/resealing, union view changes while vacant,
and `[T; N]`-sequence to flat-`T` conversion when checked multiplication and
the language's array-layout theorem establish the bijection. `N = 0` and
zero-sized payloads use logical-place counts rather than pointer arithmetic.

### 3.2 Opaque ordinary module

An ordinary module may define:

```text
opaque Owner<S, T>

hidden:
  runtime representation R
  generative roots rho...
  resource algebra G
  invariant Inv(S, R, G)
  disposition D
  proofs of every exported transition
```

Each exported operation supplies:

```text
resource precondition
normal postcondition
each recoverable-failure postcondition
pre-abort safety invariant
declared effects and fact changes
implementation
proof against the fixed reference policy
```

The producer may define pure relations, inductive predicates, guarded recursive
worlds, ghost resources, state machines, and proof automation. Definitions are
transparent to the verifier and opaque to clients. Recursive definitions must
pass positivity, productivity, and guardedness rules. A resource algebra must
prove its validity and composition laws. No producer rule enters the TCB: a
derived rule is accepted only with a proof that it implies the reference
policy.

The client sees only the runtime representation manifest, exported checked
contract, effects, disposition, and cost-relevant facts. It cannot construct a
root, alter hidden metadata, replay a transition, or widen a footprint.

### 3.3 Exact focus

Opening an opaque resource protocol is an exact control-flow judgment:

```text
focus Owner<S,T> {
  ...
}
```

The notation is conceptual, not selected syntax. Every normal edge, including
fallthrough, return, break, propagation, callback stop, and helper return, must
do exactly one of the following:

- re-establish a declared opaque owner postcondition;
- return or transfer every named resource in a declared result; or
- consume the complete owner through its disposition.

Every vacancy, live value, loan, root, offered owner, and temporary protocol
authority is accounted exactly once. Trap aborts without unwind, but no invalid
read, duplicate destruction, stale fact use, or release may occur before the
trap.

The exact outer judgment is necessary because an affine focus token may simply
be abandoned. A callback or helper cannot capture the open protocol; its exact
resource postcondition must rejoin the caller.

### 3.4 Disposition

Disposition is a static type property:

```text
Disposition(T) = Trivial | Plan<E, delta> | MustClose
```

- `Trivial` means normal abandonment has no operational work.
- `Plan<E,delta>` names statically bound executable ordinary code with a proof
  that it disposes every valid state on every non-aborting execution with exact
  effects `E` and an exact pre-abort prefix invariant.
- `MustClose` forbids normal abandonment. An exact explicit transition,
  transfer, or close must discharge the owner.

A verified plan may inspect module-private invariant-covered metadata, take
each live payload once, recursively invoke approved plans, and call exact total
non-suspending admitted actions such as atomic decrement or root release after
their preconditions hold. It must structurally decrease and terminate. It may
not invoke an arbitrary callback, allocate, suspend, report recoverable
failure, publish a value, read a vacant slot, or trust unchecked metadata.

A nested payload disposition may trap under its declared abort contract. At
that point the plan has destroyed exactly one proved prefix or selected set,
every remaining owner is still live in its recorded place, and no released or
dead place has been accessed again. There is no rollback or unwind after the
trap. Thus dense drop and truncate can preserve exact destructor-prefix abort
semantics without pretending that a possibly trapping destructor is a
recoverable failure.

The plan has no runtime pointer, vtable, tag, or registry. It is statically
selected, monomorphized, and inlineable. A container of `MustClose` values is
also `MustClose` unless an exact admitted action supplies a total terminal
abandonment transition.

This is narrower than arbitrary `Drop`: it is a proved weakening map
`T -> unit`, not a general user callback. It closes the otherwise fatal gap
between a certificate that knows what is live and executable code that actually
destroys it.

### 3.5 Static and runtime-selected callables

Static generic behavior and runtime dispatch use one checked callable model:

```text
Callable<C> = exists E,p.
  CodeLease(p,C,E) * Environment<E> * InvokeAuth(p,E,C)
```

`C` fixes receiver mode, arguments, results, every normal/failure/abort branch,
effects, borrow provenance, environment disposition, ABI, reentry,
concurrency, and code lifetime. Invocation returns, transforms, transfers, or
consumes every callable and environment authority exactly as `C` declares.

A known internal function with no environment specializes to a direct call and
needs no runtime callable object. A closure, heterogeneous behavior collection,
callback, driver registry, or open-world dispatch point may instead choose a
code pointer or selector, environment, lifetime state, and indirect branch.
Those costs belong only to the selected dynamic contract. Closed tagged unions
and ordinary vtables are library representations of the same protocol, not
separate language privilege.

The fixed machine semantics and admitted lowering supply the irreducible
checked indirect-call edge. Ordinary verified code can construct a `Callable`
only for a checked implementation and exact contract. Dynamic loading or
symbol resolution yields a `CodeLease` only after the same R-5/R-6 admission
and final-image predicate; unloading consumes every outstanding lease. An
opaque native plugin remains `TRUSTED_FOREIGN`. A certified dynamically loaded
artifact passes the same predicate as statically linked certified code. No
module name, function pointer, or plugin registration creates authority.

### 3.6 Source-preserving copy and stored borrows

Source-preserving copy is not relocation and not an implicit `Clone` call. A
fixed core predicate `CoreCopy(T)` proves the exact theorem: the representation
and validity relation are preserved by the declared machine/field copy; every
logical leaf is structurally duplicable without protocol action;
`Disposition(T) = Trivial`; and the operation invokes no behavior or effect.
Merely containing no unique leaf is insufficient: a shared-owner handle whose
count must be incremented is not `CoreCopy`.

The core exposes two separate checked judgments. `init_copy` reads a live source
through a compatible loan and changes a vacant destination to full.
`overwrite_copy` changes an already-full destination to the copied value and is
sound only because the displaced value has trivial disposition. Overlapping
bulk overwrite follows fixed memmove ordering semantics. Both leave the source
live and perform zero behavior calls. Explicit `Clone`, producers, hashers,
comparators, and callbacks are ordinary checked `Callable` contracts with their
own exact call count, order, effects, and result provenance. A coarse demand
label never silently adds either premise.

An opaque owner may be indexed by external roots and retain values containing
borrow leaves. Moving such a value preserves every leaf's root and permission;
it does not require the payload to be region-free. The container owner cannot
outlive the retained root, and its disposition returns or ends every leaf once.
Reborrows and yielded results receive exact subleases. A callable environment
may retain the same kind of leaves under its `Environment` contract. Purely
static lifetimes erase; a contract selecting runtime borrow enforcement pays
only its selected state. This closes stored-borrow expressibility without
granting a producer new provenance facts.

## 4. Derived storage operations

### 4.1 Replacement

```text
Full(i,old) * own new
  -> Vac(i) * own old * own new
  -> Full(i,new) * own old
```

### 4.2 Dynamic swap

After proving distinct logical footprints and ending overlapping loans:

```text
Full(i,x) * Full(j,y)
  -> Vac(i) * Vac(j) * own x * own y
  -> Full(i,y) * Full(j,x)
```

Pointer inequality is never the disjointness proof, including for zero-sized
values.

### 4.3 Arbitrary relocation

```text
Full(src,x) * Vac(dst)
  -> Vac(src) * own x * Vac(dst)
  -> Vac(src) * Full(dst,x)
```

A loop carries a fixed number of exact vacancy resources. It does not require a
persistent bitmap. Overlapping ranges choose a safe direction. A proved bulk
lowering to `memmove`-class code is an optimization of the same semantics, not
a new public authority.

The bounded `LiveShape` candidate needed primitive swap and replace because
two interior vacancies exceeded its interval grammar. General exact resources
remove that artificial boundary.

## 5. Allocation and failure

Physical allocation and release are admitted allocator actions. The ordinary
carrier constructor exposes exact outcomes:

```text
Err(e)            no root and no consumed offered owner
Ok(empty_owner)   one fresh root and every slot vacant
```

Aborting shortage is an ordinary wrapper over the recoverable entry. This
preserves one underlying mechanism while keeping the two observable contracts
distinct.

The efficient default for failure-atomic growth is reserve-first:

1. check every size, layout, and capacity calculation;
2. acquire every required replacement carrier;
3. on failure, dispose any wholly empty new carrier and return the unchanged
   old owner plus every offered affine value;
4. open exact focus only after the last recoverable preparation;
5. perform infallible `take`/`put` relocation;
6. establish the new invariant; and
7. release the empty old carrier.

Failure atomicity is a protocol theorem, not one universal runtime transaction.
An unknown-length producer that can fail after consuming input must expose
partial progress, reserve a proven bound, buffer externally, or choose an
aborting policy. The basis does not promise impossible rollback.

### 5.1 Zero-sized payloads

R-1 has an exact zero-sized route. A dense owner may use a virtual generative
root, zero allocator calls and bytes, target `usize::MAX` logical capacity, and
one distinct `(rho,epoch,index)` owner token per live element. Length arithmetic
still checks before ownership transfer or a behavior call. Pointer equality,
inequality, and ordering grant no identity, alias, or disjointness fact. A move
transfers one logical token and ends the source role while moving zero bytes;
disposition destroys exactly `len` logical owners and performs no payload
release. Borrow footprints are logical index sets, including empty byte spans.

A contract may instead reject zero-sized instantiations. The paper construction
sketches both policy variants over the same candidate basis. This does not close
their exact dense-family direction or earn D-2 credit; that choice remains
unresolved here.

### 5.2 Ordinary custom allocators

Only acquisition or release of a machine/OS allocation root is irreducible. An
ordinary allocator may own a large carrier, partition vacant `Space`, and
delegate a footprint as a fresh existential child root. The child disposition
returns the complete vacant footprint to the parent protocol; while delegated,
the parent escrows overlapping `MoveAuth` and its root-release authority. Split,
coalesce, size classes, slabs, arenas, buddy trees, free lists, quarantine, and
allocation policy are ordinary invariants. Delegation and return are proof-only
root/footprint repackages and perform no OS allocation, payload copy, or hidden
per-object action beyond metadata selected by the allocator.

## 6. Shared ownership and dynamic borrowing

Shared and weak owners are ordinary protocols over module-chosen runtime
metadata plus erased authoritative ghost state:

```text
State = Alive(strong > 0, weak >= 0) | PayloadGone(weak > 0) | Dead
```

The module proves:

- clone increments the appropriate count without overflow;
- the last strong transition takes and disposes the payload once;
- weak upgrade succeeds only while a strong owner exists;
- the final weak transition releases the allocation once;
- cycles follow the advertised leak or cycle-breaking contract; and
- concurrent count operations satisfy the selected atomic profile.

Dynamic borrowing is another ordinary protocol. Its runtime state is required
by the contract, such as one writer state or a reader count. A returned guard
owns the exact permission to restore that state; guard abandonment uses a
static verified plan. Split guards require a checked footprint partition.

No reference count or runtime borrow word appears in a statically unique owner.

## 7. Concurrency and weak memory

A proof module cannot invent the meaning of atomics. R-4 fixes the source
interference model; R-6 proves or validates preservation by every lowering.
The source profile fixes:

- atomic load, store, exchange, compare-exchange, read-modify-write, and fence;
- `Relaxed`, `Acquire`, `Release`, `AcqRel`, and `SeqCst` orderings, with no
  consume ordering;
- per-location modification order and synchronization/happens-before;
- thread spawn/join resource transfer;
- non-atomic conflict and data-race rejection;
- invariant opening around atomic events;
- reclamation obligations; and
- separate safety and progress assumptions.

For this paper derivability argument only, the exact witness is the
[PS2.1 Coq artifact](https://plax-lab.github.io/publications/promisingcomp/promisingcomp-coq.rar)
published with *Verifying Optimizations of Concurrent Programs in the
Promising Semantics*, retrieved 2026-07-15: 757,427 bytes, SHA-256
`e409056a3305fdd89b3d7012cf83b3d43f904f3311d1c20293b7f1a1785dac7a`.
Its `ProgramEvent` read, write, update, and fence semantics and its `relaxed`,
`strong_relaxed`, `acqrel`, and `seqcst` ordering lattice are the witness, not a
loose "Promising-family" reference. Xlang's research mapping uses fixed typed
atomic locations; acquire loads and release stores are the direction-specific
views of `acqrel`, an RMW carries separate read/write orderings, compare-
exchange failure is a read branch, weak spurious failure is an explicit
nondeterministic no-write branch, and consume is absent. Mixed-size or
overlapping atomic/non-atomic places are rejected by R-1; unsupported target
widths require an exact admitted helper contract and cannot claim lock freedom.

This frozen artifact is a feasibility witness, not production memory-model
adoption. A production semantic root must separately freeze one exact formal
profile covering the same questions, or every R-4-dependent production claim
remains unavailable. The gate may instantiate only the profile already fixed
in that root.

The concrete target atomic instruction, thread creation, park, wake, and clock
actions are exact admitted entries. An ordinary higher-order concurrent
separation proof may define invariants, authoritative ghost state, protocols,
locks, channels, reference counts, epochs, and hazard ownership. Proof state
erases. Runtime words, fences, retries, and reclamation records remain only
when the chosen algorithm requires them.

Safety does not imply lock freedom, termination, fairness, or eventual wakeup.
Any such claim carries its exact scheduler/environment assumption in the
contract and, when external, in the admitted frame.

## 8. Address stability, pinning, and self-reference

Pinning does not require a privileged named `Pin` container. A carrier already
has a generative root and exact origin. The fixed policy states which roots have
stable physical placement and which transitions relocate a payload.

An ordinary protocol may seal:

```text
Pinned(rho, S) = stable_place(rho,S) * Full(rho,S) * no_relocation_lease
```

Creation initializes a stable heap, static, mapped, or lexically fixed carrier
before publishing internal references. Projection preserves the lease only for
fields proved structurally pinned. Destruction occurs in place, ends every
internal borrow, then releases the carrier. Moving the small owner handle is
allowed when it does not move the root; moving inline payload bytes is rejected
while the lease exists.

The paper route is intended to let self-referential owners and intrusive nodes
use the allocation count and pointer layout of conventional pinned
implementations; exact P-1 parity remains pending. Stack/inline pinning
additionally requires a lexical non-relocation proof; it is not silently
inferred from pointer equality.

## 9. Async and cancellation without an async primitive

The minimal basis does not require language-level suspension or a privileged
`Future` type. An asynchronous computation can be an ordinary opaque state
machine:

```text
poll : Owner<State> * WakeLease
  -> Ready(result, TerminalState, ReturnedOrConsumedWakeLease)
   | Pending(Owner<RegisteredStateContainingWakeLease>)

cancel : Owner<State>
  -> Cancelled(TerminalState)
   | HandedOff(ExternalContinuationOwner, named_acceptor)
   | MustClose(CancelState)
```

OS registration, readiness, completion, cancellation observation, clocks,
thread parking, and wake syscalls are exact admitted effects. Executor queues,
scheduling policy, task storage, waker ownership, and parking policy are
ordinary modules. The ordinary task module proves ownership of its state frame,
buffers, wake token, child tasks, and every poll transition. A pending external
operation retains the exact buffer or lease named by its capsule contract.
Every branch accounts for wake registration, child owners, buffers, and
external completion authority. `MustClose` is not terminal: only explicit
closure or irreversible transfer to the named acceptor discharges it.

If cancellation is a total synchronous admitted transition, a verified plan
may use it. If cancellation can block, suspend, fail while preserving an
obligation, or require acknowledgement, the state owner is `MustClose`; the
executor must drive the explicit cancellation protocol to a terminal state.
Dropping a task is never assumed to cancel it safely.

This route can express executors, futures, structured tasks, and cancellation
without adding hidden stack unwinding or mandatory heap allocation. A future
surface `async` lowering would be ordinary compiler sugar only after it proves
the same state-machine contract.

## 10. External resources and FFI

An external capsule supplies only the irreducible edge:

```text
exact ABI, target, frame, and provider disposition
offered and returned resource owners
success, partial-progress, interruption, and failure states
Terminal
| Transferred(to named owner, exact residual obligations)
| MustClose(next state)
callback, reentry, and foreign-thread behavior
fact preservation and attenuation
```

Abandonment is admitted only when `Trivial` or a verified total `Plan` proves
the exact disposition. Merely dropping an affine capsule does not discharge an
external obligation.

Ordinary resource modules supply buffering, retry policy, typestate, partial-
I/O loops, path and protocol policy, checked wrappers, and composite cleanup.
They prove correct use of the capsule contract; they do not prove an opaque OS,
device, loader, or foreign implementation correct.

`CERTIFIED` requires exact implementation bytes and a complete dependency
cone. An opaque dynamic provider is necessarily `TRUSTED_FOREIGN`; it binds a
root-defined validation policy and names the residual provider assumption.

## 11. Fixed checked-resource IR and final-code binding

The proposed ordinary route targets one canonical checked-resource IR in the
semantic root. Its artifact contains:

- canonical executable IR;
- exported checked contracts and hidden representation manifest;
- proof definitions and certificate;
- disposition and effect/fact provenance; and
- exact dependencies on admitted entries.

The fixed verifier checks the artifact. Proof search, AI generation, solvers,
and derived front-end rules are untrusted. Proofs and ghost resources erase
before code generation.

Final-code safety is discharged only by either an end-to-end refinement theorem
or a machine-checked per-artifact refinement certificate from canonical IR to
the exact post-link, post-relocation loaded image and resolved helper/import
cone. The chain covers optimization, code generation, object emission or
assembly, static and dynamic linking, relocation, loader mapping, the target
memory profile, executable-code immutability, and the load event that fixes the
actual image identity. A validation route therefore binds code/data bytes,
imports, relocations, provider identities, the complete dependency cone, and a
checked load receipt. Determinism or byte identity alone is insufficient.

Every uncovered stage is an explicit R-6 TCB assumption and the resulting
route is not `CERTIFIED`.

An unverified source proof followed by unconstrained optimization is not
sufficient. Arbitrary proof-carrying native objects remain outside this
selection because they would substantially enlarge the decoder, machine model,
and verifier. They may be researched later without changing the ordinary
resource-module principle.

## 12. Structural performance hypothesis

The candidate is designed to avoid a forced language tax. The table records the
architecture-level cost hypothesis to test; it is not an exact P-1 PASS or a
throughput-parity result. Exact same-contract schedule and cost evidence remains
pending, including Rotate dispatch and stable and cached-key stable sort.

| Cost dimension | Candidate rule and remaining obligation |
|---|---|
| Payload and metadata layout | Chosen by the ordinary module; no universal header, topology tag, bitmap, generation, reference count, or destructor pointer. Static layout evidence erases. Runtime-erased layout or ownership uses only its selected descriptor, disposition, or closed-set tag. |
| Allocation and indirection | The candidate adds no universal allocation or indirection. Inline and full adoption are intended to be proof-only ownership repackages when their exact preconditions hold; per-contract parity remains an evidence obligation. |
| Ownership traffic | `take`/`put` correspond to required moves; reshape and owner repackaging move zero bytes. Source-preserving `Copy` has its declared copy traffic and never masquerades as relocation or `Clone`. |
| Calls and dispatch | Static generic calls are intended to specialize directly, with possible code-size growth. A selected dynamic callable retains its code pointer or selector, environment/lifetime state, and indirect branch. Exact best-dispatch parity remains pending. |
| Cleanup | `Trivial` emits nothing. A static plan is inlineable but may branch over selected metadata, traverse the structure, or need bounded traversal state. Exact generated cost remains a measurement obligation. |
| Proof state | Roots, permissions, protocol predicates, ghost state, and certificates erase. |
| Dynamic checks | Retained when required by the public contract or selected runtime validation. Elision requires a machine proof and exact provenance. |
| Concurrency | Unique sequential code pays no atomic or sharing cost. Concurrent algorithms pay their selected atomics, retries, and reclamation state. An unsupported width may require a helper or lock table and cannot claim lock-free cost. |
| Pinning | No runtime pin tag is mandatory, but a selected stable-place contract may retain backing storage and inhibit relocation, scalar replacement, register promotion, or stack-slot reuse until leases close. |
| Async | No mandatory coroutine runtime or unwind machinery exists. A selected task may require a discriminant, live state fields, registration/queue state, atomics or reference counts, frame indirection, and buffers retained through cancellation. |
| Must-close lifecycle | No runtime word is universal. A selected `MustClose` protocol may impose retention, backpressure, and unbounded shutdown latency until explicit progress or transfer. |

The paper witnesses preserve candidate routes to the representation, allocation
count, indirection count, operation sequence, and asymptotic algorithm of a
best-known unsafe or privileged implementation. They do not establish this for
every exact protected contract, so P-1 remains pending. They also do not prove
that a future xlang compiler will generate equal code. Code size, throughput,
proof production, checking time, diagnostics, and weak-writer success require
separately authorized measurements.

## 13. Pareto and lower-bound argument

| Architecture | Expressiveness | Mandatory runtime tax | Ordinary-library generativity | TCB |
|---|---|---:|---|---|
| Privileged container catalog | Bounded by approved names | Per-container choice | Fails | Every implementation or unsafe substrate |
| Closed `LiveShape` catalog | Bounded by topology grammar | None inside catalog | Fails for arbitrary sparse/recursive protocols | Fixed catalog and cleanup machinery |
| Universal runtime tags | Broad dynamic topology | Required tag/check/scan state | Strong | Runtime validator plus metadata coherence |
| Writer-visible unsafe | Operationally universal | Per implementation | Strong | Every unsafe implementation and reviewer judgment |
| Ordinary certified protocols | Open-ended over fixed semantics; enumerated witnesses require certificates | None beyond selected contract | Strongest surviving evaluated candidate | Fixed reference policy, verifier, backend binding, exact foreign assumptions |

Each retained authority has a removal witness:

1. Without empty carriers, an affine non-`Default`, non-`Copy`, non-cloneable
   value cannot inhabit spare capacity.
2. Without `take`, `pop` cannot return sole ownership without cloning or
   leaving a second droppable owner.
3. Without `put`, `push` cannot establish one live value in vacant storage.
4. Without checked byte extents and typed-place witnesses, header-plus-tail,
   heterogeneous arena, and vacant union reuse require serialization, extra
   allocations, or unchecked reification.
5. Without proof-only partition reshape, flattening `[T; N]` storage requires
   forbidden payload traffic despite an exact layout bijection.
6. Without full adoption, converting an already-full inline or heap owner requires
   payload traffic or a second privileged path.
7. Without generative opacity, a client can forge length, substitute a root,
   or replay stale authority.
8. Without partitionable `MoveAuth` and linear `ReleaseAuth`, a root can be
   moved, destroyed, reused, or released while a borrow, pin, external pointer,
   or reclamation guard survives.
9. Without exact focus, an affine temporary protocol can be abandoned while
   the steady-state owner is invalid.
10. Without executable disposition, a certificate cannot perform required
   payload destruction and release.
11. Without `MustClose`, a fallible or suspending terminal protocol is falsely
   treated as abandonable.
12. Without user-defined predicates and conservative recursion, an arbitrary
   sparse or recursive live set exceeds every fixed topology catalog.
13. Without exact core source-preserving copy, a `Copy` contract must either
    destroy the source, invoke forbidden behavior, or lose its declared traffic
    and call-count bound.
14. Without checked existential callables and code leases, runtime-selected
    closures, callbacks, registries, and unload-safe plugins are not derivable.
15. Without resource algebra and interference semantics, unique ownership does
    not derive shared observations, weak lifecycle, or concurrent invariants.
16. Without fixed atomic semantics, a proof can invent publication or ordering
    facts.
17. Without exact foreign contracts, ordinary code can only guess what an OS,
    device, or callback does.
18. Without final loaded-image binding, an accepted proof can be invalidated by
    optimization, assembly, linking, relocation, import substitution, or loading.

`swap`, `replace`, range split/join, bounded topology names, async suspension,
named containers, and runtime destructor registries fail this test: each is
derived or dominated rather than independently necessary.

## 14. Evidence base

The sources establish the feasibility of components, not the correctness of an
unimplemented xlang design:

- [Foundational Proof-Carrying Code](https://www.cs.princeton.edu/~appel/papers/fpcc.pdf)
  reduces trusted checking to logic plus machine semantics and a small checker.
- [PCC with Untrusted Proof Rules](https://people.eecs.berkeley.edu/~necula/ISSS02/isss02.pdf)
  shows that producer-defined high-level rules can be proved to imply a fixed
  low-level safety policy instead of entering the TCB.
- [Typed Assembly Language](https://www.cs.cornell.edu/talc/papers/tal-toplas.pdf)
  provides precedent for typed code pointers, existential packages, and checked
  low-level control transfer rather than untyped function-pointer trust.
- [ATS stateful views](https://open.bu.edu/bitstreams/2c20177c-44d5-498e-a7e0-3ac321b4f65f/download),
  [Mezzo permissions](https://gallium.inria.fr/~fpottier/publis/pottier-protzenko-mezzo.pdf),
  [Vault adoption and focus](https://www.microsoft.com/en-us/research/wp-content/uploads/2001/05/pldi01.pdf),
  and [GhostCell](https://plv.mpi-sws.org/rustbelt/ghostcell/paper.pdf)
  provide precedents for erased typed initialization state, separation,
  generative roots, adoption, and focus.
- [Iris from the Ground Up](https://iris-project.org/pdfs/2018-jfp-iris-from-the-ground-up-final.pdf)
  supplies a modular higher-order concurrent separation foundation with user-
  defined resource algebras and invariants.
- [SteelCore](https://mtzguido.github.io/pubs/steelcore.pdf) derives an
  extensible concurrent separation logic over effectful semantics and verifies
  locks and protocol-indexed channels.
- [Relaxed Separation Logic](https://people.mpi-sws.org/~viktor/papers/oopsla2013-rsl.pdf)
  and [RustBelt Meets Relaxed Memory](https://plv.mpi-sws.org/rustbelt/rbrlx/paper.pdf)
  show why weak-memory ownership transfer and reclamation require ordering-
  specific reasoning rather than sequential permissions.
- [Promising Semantics](https://sf.snu.ac.kr/promise-concurrency/),
  [Promising 2.0](https://sf.snu.ac.kr/promising2.0/), and the exact
  [PS2.1 mechanization](https://plax-lab.github.io/publications/promisingcomp/coqdoc/index.html)
  provide a type-safe weak-
  memory witness that avoids bad out-of-thin-air behavior while validating
  compiler and hardware optimizations; [IMM](https://plv.mpi-sws.org/imm/paper.pdf)
  supplies mechanized weak-memory compilation evidence.
- [Actris](https://iris-project.org/actris/) demonstrates ordinary protocol
  reasoning for asynchronous message passing over a concurrent separation
  foundation.
- [RefinedC](https://plv.mpi-sws.org/refinedc/paper.pdf) demonstrates
  foundational proof production with a restricted, predictable automation
  fragment rather than unrestricted backtracking.
- [StarMalloc](https://arxiv.org/abs/2403.09435) demonstrates a competitive
  verified concurrent allocator and reusable verified data structures over
  Steel, while not proving xlang's default-writer or compiler-cost claims.

No cited system simultaneously establishes xlang's complete combination of no
writer-visible unsafe, no garbage collector, no unwind, ordinary unprivileged
library generativity, predictable AI proof production, and protected default-
shape performance. Those remain project-specific obligations.

## 15. Proposed claim and limits

The architecture hypothesis submitted for owner review is:

> Relative to one fixed authenticated semantic toolchain root, one sealed gate
> admits every irreducible semantic edge. Ordinary opaque modules can carry
> executable resource protocols and producer-supplied proofs checked against
> one fixed reference policy. Generative byte carriers, checked typed places
> and partition reshape, one live/vacant movable-place isomorphism, core
> source-preserving copy, exact focus, static verified disposition, checked
> callables/code leases, and fixed interference semantics are candidate
> ingredients for the enumerated container and systems protocol layer without a
> privileged named container or mandatory universal runtime metadata.

The paper witnesses have not closed that hypothesis. Exact dense D-2 currently
fails closed on 340 unique unresolved obligations across 150 contexts, and exact
P-1 remains pending.
Those results can still falsify the claim that this candidate needs no additional
public authority or privilege path.

The claim does not establish:

- production soundness of a not-yet-formalized checker or IR;
- automatic or reliable proof generation by the default writer;
- useful proof diagnostics;
- bounded proof size or checking time;
- final loaded-image correctness without end-to-end refinement or checked
  post-link/post-relocation validation and a load receipt;
- measured throughput, code size, or compile time;
- external-provider truth beyond explicit trusted assumptions;
- progress without declared fairness assumptions;
- real-time, side-channel, or fault-tolerance guarantees; or
- a production language or syntax decision.

Those are later implementation and measurement gates. The candidate is designed
to address them without a second privilege path or a privileged standard
library, but that conclusion remains pending exact D-2, exact P-1, and owner
review.
