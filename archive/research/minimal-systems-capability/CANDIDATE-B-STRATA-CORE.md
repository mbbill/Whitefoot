# Candidate B-Strata Normalized Core

Date: 2026-07-15

Status: Phase 1 candidate under construction and hostile review.

This document normalizes the eight analytical B-Strata jobs into three
project-independent judgments. It is a research model, not writer syntax or a
production specification change.

## 1. Decisive claim and safety boundary

The working normalization claim is that B-Strata can express the current
reference-pressure corpus with three judgment families, not one form per
container or reclamation policy:

- `K1 ROOTED-PLACE` identifies an exact physical place without granting access;
- `K2 SEALED-STATE` records the closed state and multiplicity of resources over
  those places; and
- `K3 LINEAR-STEP` is the only way accepted code may change authority-bearing
  state.

This Phase 1 core is an upper bound, not yet the final minimum. The exact
Hashbrown, mimalloc, SQLite, and Crossbeam derivations are stress witnesses and
reference baselines. The decisive requirement is to close their frozen demand
contracts and performance bands; a selected substitute may use another data
structure, reclamation strategy, or algorithm. Any rule used only to reproduce
one reference topology must be deleted if a safe, performance-qualified
substitute makes it unnecessary. Accordingly, `PAPER-PASS` in the boundary
ledger means that a reference stress witness is expressible; it does not prove
that every rule in that witness belongs in the final minimal core.

The model must make uninitialized typed reads, overlapping mutable access,
use-after-release, duplicated affine disposition, premature concurrent
release, and authority forged from metadata or an address unrepresentable.
It does not prove B-tree ordering, SQL semantics, database crash consistency,
eventual reclamation, callback termination, allocator policy, or application
progress. Those remain ordinary program properties. An abort has no cleanup
successor, but no invalid read, race, duplicate disposition, or premature
release is permitted before the abort.

## 2. One resource state

The checker state is

```text
Omega = (Roots, Partitions, Obligations, Facts, Events)
```

For each live physical root, `Roots` contains a generative identity `rho`, a
root-layout epoch `lambda`, an exact byte extent and alignment, and one affine
release authority. `Partitions[rho]` is an affine, nonoverlapping, complete
partition of that extent. Each leaf has a local state generation `epsilon` and
is exactly one of:

```text
Vacant(place)
Bytes(place)                 -- every byte in the leaf is initialized
Live<T>(place)               -- exactly one live T occupies the leaf
Transit(place, obligation)   -- an owner is temporarily escrowed in progress
Disposition(place, action)   -- one terminal action remains owed
```

Partial initialization is represented by splitting a place into `Vacant` and
`Bytes` leaves; there is no implicit initialization bitmap. A free-list node
can therefore be `Live<Link>` in its first word and `Vacant` in the remainder.

`lambda` and `epsilon` are distinct. `lambda` changes only when a reunited
whole root is resized, released, or given a new root-wide layout. A local
write, take, or reclassification consumes only the overlapping `epsilon` and
leaves disjoint subowners valid. This distinction is required for mimalloc:
changing one block must not invalidate every other block owner on its page.
Both names are nonwrapping checker identities, normally SSA generations, and
do not require runtime counters.

`Obligations` separately contains unique and shared loans, observation guards,
deferred rights, focus/progress state, and disposition escrows. An obligation
removes the incompatible move, reclassification, or release operation from
the root owner until it ends. Release requires a reunited partition and an
empty obligation set. Released is terminal and has no place or addressable
footprint.

K2 is not a second store of these resources. A K2 value is the unique typed
view of entries already present in `Partitions` and `Obligations`. Sealing,
opening, product, sum, span, classifier, and strict-chain operations only
repackage that one affine multiset. They never leave both the unsealed entries
and a certificate live. Each K3 rule therefore carries a total lineage map from
every affine output to exactly one consumed affine input, except for an output
whose sole initial origin is listed in the authority-origin ledger. The rule
ledger's input/output multiset is normative; the origin DAG alone is not a
sufficient conservation check.

`Facts` are read-only projections from accepted judgments. A fact never
appears in the premises of a rule that creates ownership, liveness,
observation, retirement, release, or another authority. `Events` are exact
machine or external event witnesses; an event can advance an already-owned
sealed state only through a fixed K3 rule.

## 3. K1 ROOTED-PLACE

The judgment is:

```text
Omega |- p : Place(rho, lambda, lo, hi, align)
```

It has only three constructor classes:

1. `root`: derive `[0, extent)` from an existing root owner;
2. `project`: derive a declared field, checked index, or checked byte range;
3. `split/join`: split at a checked boundary or reunite exact siblings.

All offset, length, stride, and end arithmetic is checked for overflow and
containment. Dynamic disjointness is obtained by a successful split or exact
range check, not by a writer proposition or a general arithmetic solver.
Fields and literal unequal indices normalize syntactically; other ranges use
the exact checked boundaries that created them.

A K1 place is copyable descriptive data in the checker. It creates no root,
owner, live value, borrow, child edge, or release right. An address, page
number, bucket index, handle, metadata object, or wrapper identity cannot be
substituted for `rho`.

## 4. K2 SEALED-STATE

The judgment is:

```text
Omega |- c : Sealed<m>(rho, lambda, state)
```

Every constructor fixes `m` as `affine`, `shared`, or `scoped(region)`. The
closed constructor grammar is:

```text
State ::=
    leaf(Vacant | Bytes | Live<T> | Transit | Disposition)       [affine]
  | product(State...) | sum(State...) | span(index, count, State)[affine]
  | classified(metadata, decoder, place_map, role_table, State)  [affine]
  | strict(Nil | Node(place, child_state))                       [affine]
  | loan(shared | unique, place, epsilon, region)                [scoped]
  | control(Stable | Focus | Progress | Repair | Poison, State)  [affine]
  | callable(Reusable | Once, effects, environment, gate)        [shared/affine]
  | observer(Domain | IngressSet | PublishedCustody |
             Observation | Retired | Quiescent, State)           [affine/scoped/shared]
  | fact(fact_kind, rho, lambda, epsilon, invalidators)           [shared]
```

The multiplicities are not parameters chosen by a library. `Once`, `Retired`,
all owner-bearing states, and the domain controller are affine. A unique loan
is affine within its region. An observation is an affine guard owner that may
yield shared scoped payload loans. `Quiescent(domain, cutoff)` and facts are
shared because they own no target; each target still has one affine retirement
and disposition right. A reusable callable is shared only when its captures
and effect row permit shared invocation.

`IngressSet` and `PublishedCustody` are affine. An ingress set is a closed
product, sum, span, or strict structural source whose leaves are exactly
`Open(slot, target)` or `Closed(slot, target)`. Publication consumes the target
owner into `PublishedCustody` and opens the exact ingress leaves named by the
sealed structure. A successful atomic event may close only the ingress leaf
bound to that event's slot, root, generation, old value, and new value. It does
not by itself establish that every ingress is closed. `ClosedIngress` is the
all-closed elimination of the complete set. Retirement consumes both the
published custody and `ClosedIngress`. This explicitly rejects the historical
failure mode in which a head is unlinked while another tail or auxiliary edge
can still admit a new observer.

### 4.1 Structural composition

Products require pairwise disjoint leaves and sums select one already-existing
closed state. A sum does not allocate a tag; it may be indexed by runtime state
already chosen by the representation. Spans are symbolic in their checked
count and do not cause the checker or generated code to unroll capacity.

`strict` is a finite inductive ownership form, not arbitrary reachability:

```text
Chain = Nil | Node(disjoint_live_place, Chain)
```

Push consumes one disjoint node and one chain. Pop consumes `Node` and returns
that exact node plus the tail. The stored next address selects the maintained
child certificate but creates no child authority. Affinity rules out cycles
and duplicate nodes. A closed fuel, live span, active prefix, or strict child
provides every disposition measure; a writer reachability predicate is not
accepted.

### 4.2 Classified state

A classifier contains only fixed-width metadata loads, equality, masks,
ranges, null tests, checked integer arithmetic, checked affine place mapping,
and a finite role table. It contains no source call, loop, recursion,
quantifier, proof term, validator, cleanup program, or project identity.

Classification is elimination of an existing affine certificate. It never
mints a leaf. The certificate is first sealed only from:

- fresh vacant creation plus checked initialization of its metadata;
- a fixed generic adoption rule over initialized bytes that cannot create an
  affine external resource, pointer provenance, or destructor authority; or
- a valid predecessor K3 transition.

While sealed, metadata that participates in the classifier is writable only
by consuming the classifier into an exclusive focus and closing a fixed K3
transition. A raw metadata store, an ordinary validator, or a successful
decoder cannot produce `Live`, `Vacant`, a strict child, or ownership.

The certificate stores a symbolic functional state map. Updating dynamic slot
`i` creates a nominal persistent-DAG node `(predecessor, i, successor-role)`
without materializing one ghost bit per slot. A CFG join accepts identical
nominal map identities; a different merge is legal only through a fixed K3
merge operation that consumes every branch input. A loop backedge must match
one explicit closed `Progress` schema. The checker never asks whether two
writer expressions are extensionally equal and never expands an unbounded
conditional-expression tree.

### 4.3 Closed validity, facts, gates, and profiles

Plain-data adoption is not a trait or writer-supplied validator. Its grammar is
closed:

```text
PlainData ::=
    all-bits-valid integer or IEEE scalar
  | checked-range scalar
  | finite discriminant with PlainData arms
  | fixed array(PlainData)
  | declared product(PlainData...)
```

The checker derives every range, discriminant, size, alignment, field, and
padding check. Adopted values are drop-free and contain no pointer provenance,
reference, affine owner, external handle, callable, or resource obligation.
Adoption consumes `Bytes` and preserves its root and footprint; ordinary
construction or a separate owner transition is required for all other types.

The fact grammar is exactly `Bounds`, `Alignment`, `LiveRole`, `SameRoot`,
`Disjoint`, `CurrentGeneration`, `ProtocolState`, `Publication`, and
`NonInterference`. Each kind has one fixed producer row. The checker computes
the complete invalidator set from the producer's dependency IDs; source code
cannot supply or omit invalidators.

A callable gate is exactly `Ready`, `Current(certificate, epsilon)`,
`Scoped(region)`, or `Quiescent(domain, cutoff)`, or a finite product of those
existing K2 certificates. It is not a predicate, callback, proof term, or fact.

An external operation names one sealed platform-profile row. The row fixes its
ABI, frame layout, argument and result ownership, byte footprints, ordering,
normal and every failure/partial outcome, cancellation behavior, facts, and
runtime event. Ordinary source cannot define or widen a row. The finite Phase 1
leaf kinds are allocation/acquisition, release, handle open/close, access
check, file-size query, exact range read/write, resize/truncate, sync, lock
transition, delete, atomic/target event, and separately admitted closed ABI
call. There is no generic provider-control or FFI catch-all.

### 4.4 Control and repair

`Stable` exposes the declared ordinary operations. `Focus` is exclusive and
lexical. `Progress` carries a structural cursor and the exact remaining
owners. `Repair` exposes only sealed repair entrypoints and disposition.
`Poison` exposes only close, discard-and-reacquire, or an exact externally
sourced recovery entrypoint. None of these states proves application-level
correctness.

A normal outcome can reconstruct `Stable` only when the complete root
partition is valid and every resource obligation for that state is closed.
An external event witness alone cannot produce `Stable`, `RepairComplete`, or
`Poison`; a fixed K3 outcome step consumes the entire preceding control state.

## 5. K3 LINEAR-STEP

The judgment is:

```text
Omega ; inputs --op / exact-events--> Omega' ; outputs
```

`op` is a closed table:

- owner traffic: `initialize`, `take`, `replace-return-old`, `swap`,
  overlap-checked `relocate`, `destroy`, byte overwrite, and `reclassify`;
- borrowing: begin/end shared or unique loan;
- control: open focus, advance one structural progress item, select a complete
  success/recoverable/repair/poison/abort outcome, and close stable state;
- disposition: remove one obligation before destroy, exact release, stable
  transfer, or one-shot invocation;
- callable: pack a checked reusable or affine once environment, transfer it
  when its static thread rule permits, and consume-before-invoke;
- concurrency: exact atomic custody transfer, begin/end observation, retire
  only after a sealed complete ingress set has become all-closed, validate a
  complete observer domain, and combine retirement with quiescence for
  disposition;
- external: consume an exact platform-profile event result, including partial
  completion, without granting a high-level state; and
- facts and release: project a non-authorizing fact, invalidate overlapping
  facts, and release only a reunited obligation-free root.

No operation accepts a writer state relation, invariant, cleanup body,
termination argument, quiescence predicate, or project name. Ordinary control
flow composes the fixed steps; it does not extend their authority.

Owner-traffic variants stay distinct because lowering one through another
would add a move, copy, temporary owner, destructor call, allocation, or
overlap restriction in at least one frozen operation. Atomic compare-exchange
transfers custody only on the declared successful event; failure and retry
retain the offered owner.

An event that changes authority is not a copyable fact. It is either consumed
inside the same K3 transition as the native event or stored as one affine event
entry inside `Transit`. Replaying or projecting such an entry cannot repeat an
owner, ingress, retirement, or state transition. Only a non-authorizing
read-only projection may be copied as a fact.

## 6. Authority origins and no-cycle rule

The companion authority ledger records one origin class for every authority.
The controlling rules are:

- allocation or acquisition alone creates a generative root, release
  authority, and vacant bytes;
- checked writes and exact external reads alone initialize bytes;
- initialization, a closed plain-data adoption, or a valid predecessor step
  alone establishes typed state;
- classifier elimination, addresses, metadata, facts, and validators create no
  authority;
- loans originate only from an existing live or initialized leaf;
- observation originates only from the fixed begin-observe event sequence;
- retirement originates only by consuming published custody plus a complete
  all-closed ingress set produced by checked structural transitions;
- quiescence originates only from complete fixed domain validation; and
- release consumes rather than produces authority.

State transitions may move an existing authority through a cycle such as
`Stable -> Focus -> Stable`; this is not an origin cycle because every edge
consumes the predecessor. The origin graph excludes transfer edges and must be
acyclic. In particular, facts never feed authority, quiescence never assumes
itself, and metadata never feeds the liveness it is used to decode.

## 7. Verdict-forcing boundary 1: nonforgeable liveness

Fresh Hashbrown storage begins as initialized `EMPTY` control bytes plus vacant
payload leaves. Sealing the classifier encapsulates those already-existing
states. A `FULL` observation can eliminate the certificate to a scoped loan of
the corresponding already-live payload only after a K3 insert transition has
consumed the vacant leaf, initialized the payload, written the control role,
and resealed the classifier. Writing `FULL` through an ordinary byte store is
rejected because the sealed classifier must first be consumed and no admitted
transition can close without the payload owner.

Fresh mimalloc page extension splits blocks, initializes only each intrusive
link word, and folds the resulting free nodes into a strict chain. Allocation
pops one maintained node and reclassifies its exact block to caller-owned
vacant bytes; it does not zero the remainder. Free consumes the caller owner,
initializes the link word, and pushes the node. A list head or copied next
address without the chain certificate cannot read a link or yield a block.

SQLite external input creates initialized page bytes only. Checked parsing may
derive byte subranges and non-authorizing facts, but a page number, cell offset,
or successful format validator cannot mint a root, affine child, or typed
owner. Crossbeam allocation and publication establish the pointee; a copied
tagged address is non-authorizing, and payload access additionally requires a
same-domain observation, publication ordering, and noninterference.

These four routes use the same rule: metadata selects authority already
maintained by an affine sealed state. It never creates that authority.

## 8. Verdict-forcing boundary 2: observer closure

Ingress closure, retirement, and observer closure are two separate affine
steps followed by one shared proof:

```text
RETIRE
  PublishedCustody(domain, target)
  + ClosedIngress(domain, target) from the complete sealed IngressSet
  + exact bag-seal cutoff event, or Pending before that event
  ------------------------------------------------------------
  Retired(domain, target, cutoff | Pending)
```

Bag sealing consumes every pending retired right in the bag into the same
exact cutoff without copying any target right. The one observer theorem schema
is then:

```text
OBS-CLOSE
  complete fixed registry coverage
  + every payload loan nested in a registered Observation
  + exact ordering that closes begin/scan races
  + validation that every pre-cutoff Observation ended
  ------------------------------------------------------------
  Quiescent(domain, cutoff)
```

The premises are outputs of fixed K3 events and structural folds, not writer
propositions. The registry is a K2 span or strict set whose slots have only
`Inactive` or `Active(generation)` roles. A validation fold has three
fail-closed outcomes: an inactive slot is discharged; an active pre-cutoff
slot blocks; and a concurrent structural change returns retry/stalled without
credit. `Quiescent` is shared because it owns no target; any number of distinct
affine `Retired(domain, target, cutoff)` rights may use the same validated grace
period once each. Generation advancement is permitted only after a complete
scan proves that every active slot is in the current generation. This makes
cyclic machine epoch encodings safe without treating a wrapped integer as a
fresh authority.

`ClosedIngress` and observer validation are deliberately separate. Waiting for
one, two, or any fixed number of generations cannot repair an ingress that is
still open. A successful CAS produces `ClosedIngress` only for a sealed
single-ingress target, or as one step in a complete structural ingress fold.
A copied address, a generic "successful unlink" label, an unsafe defer promise,
or a writer unreachability assertion produces nothing.

For mimalloc, the registry instance has the abandoned-map bit for one page as
its gate. A reader atomically clears the set bit before inspecting the page. If
the concurrent ownership claim loses, it restores the bit before ending its
observation. A freeing owner closes admission, and `clear_once_set` waits for a
temporarily cleared bit to be restored before atomically clearing it. The
successful clear is both complete one-slot validation and exclusion of new
readers. If the reader stalls, release stalls; it does not become legal.

For Crossbeam, the registry is the maintained participant list. First pin
publishes `Active(global_epoch)` and executes the native barrier before any
protected load. Each successful structural unlink closes only its bound
ingress leaf; eliminating the complete all-closed set creates one pending
retired right. A complete participant scan may advance the generation only if
every pinned participant is in the scanned current epoch; stalled iteration
does not advance. Two such native advances after bag sealing imply that every
observation capable of seeing a pre-cutoff target has ended. The existing
participant scan, fences, epoch store, and queue eligibility comparison are
the validation events; protected loads retain exactly one caller-selected
atomic load and no per-load domain event. The exact target row must separately
prove that the existing x86 locked read-modify-write has the required compiler
and hardware barrier semantics; a fact assertion cannot fill that obligation.

In both instances, the target root cannot be released or reused until its
affine retired right is combined with `Quiescent`. Reacquisition creates a new
generative root, so an old address or wrapped machine label cannot revive the
old authority.

## 9. Verdict-forcing boundary 3: erased affine one-shot disposition

`callable(Once, effects, environment, gate)` is the following affine package
over an existing environment; it creates no captured authority:

```text
Once<W, A, Sigma>[Phi, Obligations, Gate, Effect]
```

`W` and `A` are the native inline word and alignment limits. `Phi` maps every
capture field to its exact root, generation, range, and provenance.
`Obligations` is the captured affine multiset. `Gate` is one of the closed K2
certificate gates from Section 4.3. `Effect` is one exact effect row.
`Sigma` is a finite canonical set of
`capture-schema x gate-schema x effect-row x normal-post` signatures for the
monomorphized sink, and every pack site must match one member. Draining the
heterogeneous sink has `join(Sigma)`, just like a finite sum elimination;
source code cannot insert an unproduced wildcard signature.

Packing chooses a representation at monomorphization time. For the frozen
Crossbeam route, an environment of at most three words and alignment no greater
than `usize` is stored inline beside one call pointer. A larger or over-aligned
environment uses an explicit K3 allocation and the same existing boxed
fallback as the reference; allocation failure returns the original
environment. The protected inline route cannot silently select that fallback.
There is no runtime variant tag, second owner box, reference count, vtable, or
second drop-glue pointer.

Normal control flow may only transfer, enqueue, or invoke the seal. A full sink
returns the same package unchanged or transfers the complete sink before
retry. Invocation first replaces the active slot with the native vacant/NO_OP
value, consumes the seal into `InFlight`, moves its environment out, and only
then performs the one existing indirect call. Normal return must discharge or
transfer every declared environment owner. Divergence has no successor and
abort has no cleanup edge. A normal scope exit with an outstanding once seal is
rejected, so cancellation cannot require an extra representation-level
destructor pointer.

Cross-thread transfer is admitted only for a statically sendable environment
or a sealed retired-root disposition whose only available terminal action is
valid in the destination execution domain after its quiescence gate. There is
no analogue of Crossbeam's arbitrary unsafe `defer_unchecked` promise. The
frozen unlink-and-destroy route uses the sealed retired-root case. A copied
callable, a second invocation, an invocation before its gate, or a transfer of
thread-affine destruction is rejected.

mimalloc's registered deferred-free callback is deliberately not forced into
this form: it is a reusable shared callable with two acquire loads and one
native indirect call. The one-shot rule is justified by the Crossbeam
representation and is a K2 seal plus fixed K3 callable step, not a fourth
project-specific primitive.

The default Crossbeam bag capacity is 64; sanitizer and Miri capacity is 4.
The later two-element model bound is only a proof bound and cannot be reported
as native representation parity.

## 10. Verdict-forcing boundary 4: exact external repair

Exact leaves are limited to the sealed rows described in Section 4.3,
including access/open, file-size, handle, range I/O, resize/truncate, sync,
lock, close/delete, allocation/release, and the particular admitted ABI calls.
Each result records the exact buffer footprint, possible indeterminate side
effect when the provider exposes no partial count, handle state, ordering,
error, and invalidators. A leaf may return initialized bytes or an affine event
entry; it may not return `Stable`, `RepairComplete`, a page role, a child edge,
or a database fact.

Only a successful full read or a sealed profile's guaranteed fully initialized
short-read outcome produces `Bytes` for the complete requested buffer. Any
other read error produces no readable authority except for an exact initialized
prefix explicitly reported by that row. A failed write, truncate, sync, delete,
or unlock with no exact completion report produces its closed conservative
`Indeterminate` external state rather than an invented unchanged or successful
state.

The SQLite rollback-journal route is an ordinary K3 repair fold:

1. a sealed `Repair` owner carries the pager and cache roots, journal/WAL and
   database handles, page-size-specific scratch root, cursor, page obligations,
   cache pins and dirty state, WAL and file locks, optional super-journal name,
   bit-vector and temporary owners, and backup source/destination chains;
2. exact reads initialize the scratch bytes, and checked ordinary parsing
   validates the page number, checksum, record width, and bounds; a page-size
   change explicitly replaces the scratch root through an exact allocation
   outcome;
3. checked lookup selects an existing cache root; page number alone grants no
   root or borrow;
4. an exact write event and byte overwrite consume the old cache generation,
   create a new initialized-byte generation, and invalidate every derived page
   fact;
5. the existing reinitializer is ordinary checked reusable code, not a leaf.
   It first invalidates auxiliary page roles. Its ignored parse failure leaves
   initialized bytes plus `AuxInvalid` and no derived page facts; this safe
   cache state is not pager poison and lazy parsing may be retried later;
6. the cursor advances monotonically and removes exactly one replay
   obligation; and
7. the main recovery fold closes only after an empty replay source and the
   required successful sync, journal finalization, and lock transition. A
   later auxiliary step, such as super-journal-name allocation, may then return
   `Stable(READER) + Error` because the pager is already safely closed.

Before that main close, an I/O or finalization failure retains `Repair`, or
produces `Indeterminate`/`Poison` when the sealed provider row cannot report
whether an external side effect occurred. Ordinary allocation failure follows
its exact commit position and may either preserve `Repair` or return the
already-closed stable outcome above. `pager_end_transaction`,
`sqlite3WalUndo`, the reinitializer, and backup update are ordinary checked
folds/calls, not leaves. WAL cache reload is an exact read followed by
overwrite, invalidation, and checked reinitialization. WAL, rollback journal,
VFS policy, and B-tree semantics do not enter the language grammar.

A backup update has its own destination owner and outcome. Its failure may move
that backup to `BackupError` while the source pager remains in its exact repair
or stable lineage; it cannot silently poison or validate the source pager.

The Phase 2 leaf ledger must enumerate every concrete platform/provider row.
The Phase 1 reference boundary is falsified immediately if its complete route
needs a high-level pager, WAL, B-tree, or reinitializer leaf rather than this
finite low-level interface. That rejects the reference route, not the frozen
SQLite demand, which may still be closed by an admissible substitute.

## 11. Deterministic checking and erasure

Let `P` be checked program and CFG size including nominal state-map DAG nodes,
`M` the number of monomorphized instances, `A` the maximum sealed-state and
effect-row arity, `S` the largest finite callable signature set, and `G=(V,E)`
the authority-origin graph. Place normalization and state checking are single
bottom-up passes. Exact interval sets use balanced trees and signature
membership uses canonical finite sets, so the worst-case bound is

```text
time   O(P * M * (A log A + log S) + S * A + V + E)
space  O(P * M * A + S * A + V + E)
```

Closed role tables, nominal map nodes, and callable effect rows are part of
`P`. Runtime chain
length, allocation capacity, participant count, journal length, and queue size
are not unrolled by the checker. Structural drain and validation combinators
check one local step and a fixed decreasing measure. The checker performs no
solver search, heuristic, timeout, backtracking, or writer-proof checking.
Authority-origin ranks and named-rule membership are checked by a linear-time
DAG pass after all rule rows are loaded. Complete affine input/output lineage
checking remains a Phase 1 gate requirement, not an achieved result.

### 11.1 Open lineage-conservation blocker

The current verifier checks that every declared output authority name appears
once in a lineage map, but it does not yet prove that each lineage source is an
authority consumed by that rule or an allowed true origin. It also does not
parse the normal and failure equations as branch-local affine multisets.
Consequently, hostile mutations such as changing `K2-CLASSIFY` to map
`A-ROLEVIEW<=A-FACT`, or changing `A-LIVE`'s `origin_rule` to `K2-FACT`, can
survive the implemented ledger checks when the final status assertion is
bypassed. This is a verifier soundness blocker independent of source-topology
fidelity. Phase 1 remains fail-closed until the checker rejects both mutations
and exhaustively accounts for every branch's affine inputs, outputs, true
origins, sinks, escrows, and restoration equation.

Root identities, `lambda`, `epsilon`, partitions, classified certificates,
control states, obligations, facts, and cutoff proofs erase. Only state already
selected by the program representation remains: control bytes, lengths,
links, pager fields, participant epochs, callback words, and exact event
state. Generic lowering dispatches on the verified K3 opcode, never on a
project, operation, or whole-graph pattern. Static erasure must be visible
before fixture-specific backend dead-code elimination.

## 12. Phase 1 disposition

The disposition remains unset until the open lineage-conservation blocker is
repaired, the rule, normalization, authority-origin, interaction, and
boundary-proof ledgers pass their verifier, and independent hostile review is
complete. The only permitted results are `STRATA-CORE-PASS` or `STRATA-NO`.
