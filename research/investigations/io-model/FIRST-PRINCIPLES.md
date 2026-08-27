# Whitefoot completion I/O implementation plan

Status: IMPLEMENTATION PLAN WITH OPEN DESIGN GATES, 2026-08-26.

This file supersedes the earlier live discussion draft in place. Git history
retains that discussion. The plan records the conclusions the owner has
accepted, the implementation consequences derived from them, the parts of the
current v0.37 candidate which remain useful, and the questions which must close
before implementation can honestly claim completion.

This file is not the active language specification. The active specification
remains the one named by `docs/roadmap.md`. Nothing here authorizes a merge to
`main`; it defines the technical work to perform on the current work branch.

During the rebuild, `research/investigations/io-model/DESIGN.md`,
`docs/current-plan.md`, the I/O sections of `docs/roadmap.md` and
`compiler/README.md`, the current v0.37 candidate text, and the root README's
candidate identity still describe the superseded shared-Output/root-family
design. They remain evidence for the currently compiling candidate, not design
authority for new work. Before the semantic migration starts they must receive
an explicit superseded boundary or be rewritten together, so an implementer
cannot mistake them for this plan. `RESULTS.md` remains measurement evidence;
its measurements do not select the old semantics.

## 1. Objective

Whitefoot source uses ordinary calls for I/O:

```whitefoot
let left = read_at(
  file: &file_a,
  destination: &uniq left_bytes,
  file_offset: 0_u64,
  start: 0_u64,
  end: left_end
);
let right = read_at(
  file: &file_b,
  destination: &uniq right_bytes,
  file_offset: 0_u64,
  start: 0_u64,
  end: right_end
);
```

The writer supplies values, ownership, borrows, effects, and typed outcomes.
The writer does not choose a blocking API, create a future, install a callback,
select a worker thread, or state queue depth. The compiler may keep independent
operations in flight. The target publishes completion. The existing scheduler
executes writer code only after the required ownership milestone is available.

The target is the fastest sound implementation selected independently on
macOS, Linux, and Windows. Direct or inline execution at depth one is a
specialization of the same completion semantics, not a second source API.

## 2. Mandatory reasoning boundary

Every issue belongs to one of two layers.

### 2.1 External-to-language mapping

This layer turns a file, socket, directory, clock, process facility, device, or
quota into ordinary Whitefoot values:

```text
owned capability
+ shared or exclusive borrow
+ typed outcome
+ explicit lifecycle
+ finite capacity
+ completion milestones
```

If the mapping cannot express the authority, result ownership, failure,
release, or concurrency relation using ordinary language rules, the API is
incomplete.

### 2.2 Ordinary API use

Once mapped, source code sees only ordinary functions and values. Dataflow,
control flow, memory effects, ownership, loans, and place overlap decide which
calls may overlap. No source rule may inspect a native handle, path, syscall,
backend queue, or system-operation spelling to grant permission.

An implementation problem may not be repaired by weakening this boundary.

## 3. Non-negotiable laws

### 3.1 Completion is the only language-level I/O model

The language model is submission plus completion. Readiness, polling, a helper
thread, an interrupt, a native completion queue, and a direct host call are
target mechanisms beneath that model.

### 3.2 No writer callback

Target code may publish typed result bytes and milestone facts and may wake the
scheduler. It never receives or invokes a Whitefoot function pointer. An
unrequested or repeated event is represented by an owned Source or
Subscription, not by executing writer code on a host callback stack.

### 3.3 No correct-path cost for a false claim

A false executed claim is impossible in a correctly reviewed program. No
submission, completion, scheduler, target, release, or wake path reads a trap
latch or carries trap-specific state. The erroneous path is fail-stop and does
not shape normal performance.

### 3.4 No ambient world authority

Except for the explicitly named heap exception and compiler-owned process
entry, termination, and trap boundaries, every writer-visible world authority
originates in an entry parameter or in a factory lineage rooted in such a
parameter.

The following source shapes are therefore not ordinary system APIs:

```whitefoot
now();
random();
connect(address: address);
spawn(image: image);
open(path: path);
```

They require explicit Clock, Entropy, Network, Process, Directory, factory, or
permit capabilities.

### 3.5 No shared world mutation

A world read may use a shared borrow. A world write requires an `own` or
`&uniq` capability. A lock, atomic counter, queue, or target guarantee does
not make shared writer-visible mutation legal.

### 3.6 No second concurrency type system

The language does not use a separate logical-root registry, family-fragment
pair table, or Ordered attribution edge to repair an API which admits
conflicting shared uses. Independent work must be represented by independent
ownership places, disjoint fields, typed facets, ranges, shards, or owned
permits.

## 4. One effect algebra over two state domains

Memory state and world state obey the same frame rule, call substitution,
read/write classification, exactness check, and ownership discipline. They
remain separate source domains because their subjects have different names
and different observable meaning.

The selected source surface is one flat row of self-contained tagged atoms:

```whitefoot
memory reads('f, 'd), memory writes('d), world reads(file)
```

The canonical category order is memory read, memory write, world read, world
write, allocation, and trap. Each category appears at most once. `pure` remains
the unique empty row.

Repeating the domain on each atom is useful redundancy at a trust boundary.
An atom can be copied, inspected, normalized, or diagnosed without depending
on an enclosing block. The grammar accepts only REGIONIDs after `memory` and
only formal authority paths after `world`, so neither of these mistakes forms:

```whitefoot
memory writes(output)
world writes('d)
```

This is one effect algebra whose atoms carry a domain, access mode, and subject
set. It is not two nested effect systems. An `effects { ... }` wrapper was
rejected because it adds a container, grouping and formatting rules without
adding semantics, and makes a `writes(...)` atom depend on distant block
context.

The two subject domains are:

```text
memory effects
    reads and writes of incoming Whitefoot regions

world effects
    reads and writes of incoming capability places
```

Subject lists use commas, as every other Whitefoot list does. World subjects
are rooted only at a formal parameter and select one exact authority leaf:

```text
world_subject    := (IDENT | "deref" "(" world_subject ")")
                    world_projection*
world_projection := "." IDENT | "." TYPEID "." IDENT | "[" const "]"
```

The lowercase step selects a struct field. The `TYPEID.IDENT` step selects one
enum variant payload field. A constant subscript selects one statically known
array leaf. `deref` traverses an owning box or arena; parameter borrow mode is
otherwise transparent.

A finite concrete aggregate must name each affected leaf. The shorter
`world writes(pair)` is not an alias for
`world writes(pair.first, pair.second)`: two spellings for one boundary would
violate canonicality and would hide which authority changed. A subject may end
at an opaque resource leaf, a symbolic generic subtree, or a dynamic/recursive
element-group summary. The last two are single compile-time summaries because
their runtime leaves cannot be enumerated in source. They do not grant element
overlap. Locals and result binders can never be roots because caller-visible
state must project to an incoming formal.

Neither `external` nor `blocks` remains a source effect. The world row
replaces the former's semantic job with explicit subjects. Target completion
contracts replace the latter's implementation-shaped job.

### 4.1 Memory example

```whitefoot
fn fill['d](
  destination: &uniq 'd buffer<u8>
) -> result: own unit
memory writes('d)
```

Mutation of a locally owned buffer does not enter the caller-facing row. Its
state either leaves through an owned result or ends locally.

### 4.2 World read example

```whitefoot
fn read_at['f, 'd](
  file: &'f ReadFile,
  destination: &uniq 'd buffer<u8>,
  file_offset: own u64,
  start: own u64,
  end: own u64
) -> result: own ReadOutcome
memory reads('f, 'd), memory writes('d), world reads(file)
```

`ReadFile` promises position-explicit regular-file content observation with no
caller-visible cursor mutation. Its qualified native representations exclude
devices, proc-like streams, and other files for which positioned read consumes
or commands external state; those require a distinct Device or Source API.
Shared uses may therefore coexist. Destination memory remains protected by its
ordinary exclusive loan.

Hosted `pread` may update access-time metadata or implementation counters. The
`ReadFile` mapping deliberately does not expose or order those incidental host
facts, just as it does not expose another process's cache observation. No
Whitefoot API may later promise ordered observation of them through an
independent capability. If atime or another such fact becomes program
semantics, the file mapping must expose its real authority and reopen the
shared-read proof rather than silently retaining this contract.

### 4.3 World write example

```whitefoot
fn write_once['o, 's](
  output: &uniq 'o Output,
  source: &'s buffer<u8>,
  start: own u64,
  end: own u64
) -> result: own Result<u64, IoError>
memory reads('o, 's), world writes(output)
```

The Output handle's Whitefoot storage is read. The external byte sequence is
written. The `&uniq` loan, not an Ordered family relation, prevents two writes
to one Output from holding authority simultaneously.

### 4.4 Owned terminal example

```whitefoot
fn finish_file(
  output: own FileOutput
) -> result: own Result<unit, FinishError>
world writes(output)
```

An owned capability needs no fictitious borrow lifetime. The world row names
the formal ownership place directly. Compiler-derived release contributes the
same world footprint and is checked against the written row.

### 4.5 Exactness and purity

The checker derives both rows from the complete body, every callee boundary,
and every compiler-derived release on every ordinary control-flow edge, then
checks the written rows in both directions. This includes fallthrough,
explicit return, propagation failure, `give`, `break`, loop backedges, match
residuals, displaced values from replace, and partially constructed aggregate
cleanup. A writer cannot omit or pad a world effect. The impossible false-claim
path remains the fail-stop exception described above; it does not unwind.

`pure` means that memory effects, world effects, allocation effects, and traps
are all empty. A backend or optimizer never invents a hidden world-action tag
to repair a `pure` signature.

The shared algebra does not make a world observation stable like an ordinary
memory read. Another process, device, peer, or clock may change the observed
state without a Whitefoot world write. A world read therefore cannot be
coalesced, duplicated, hoisted from a loop, or replaced by an earlier result
unless its API contract separately proves snapshot stability or another law
which licenses that exact transformation. Shared borrowing may still permit
overlap and arbitrary linearization; it does not turn two observations into
one. Frame, substitution, exactness, and ownership are shared. Value stability
is a property of the mapped API.

### 4.6 World-bearing type shape

Whether a place may appear in a world row is a structural type judgment, never
a type-name or operation-name test. An opaque system resource declaration
explicitly declares each world-bearing leaf. Struct and enum declarations
derive a shape from their fields and payloads. Projection selects a leaf;
borrowing preserves it; construction, move, match, and replacement preserve
the corresponding structural path.

A generic parameter is world-bearing only under an explicit kind constraint or
after closed-world specialization proves its concrete shape. A generic body
which must name a world effect but has neither fact is unsupported rather than
silently pure. Plain data has no world leaf. A factory declares freshness and
publication projection per world-bearing result leaf. The same shape drives
release, row substitution, and ownership lineage, so aggregates do not need an
I/O-specific side table.

## 5. The shared frame theorem

The world row follows the same rule as the memory row:

```text
only state supplied from outside the function appears in its boundary row
```

A function-local fresh capability creates no anonymous `world writes(own)`
effect.

For a fresh capability `F`, returning an owner closes the boundary only while
`F` remains completely framed and unpublished before the return. No namespace,
remote peer, process, device, or other observer may have observed its state.
The owned result then carries all state which survived the call.

If any state is published, made remotely visible, installed in a namespace,
delegated, or otherwise exposed before the return, that transition also
projects to the incoming capability which authorized the exposure. Returning
the resulting owner does not erase that world effect. A connection which has
sent SYN, a process which has started, and a named file which has been
installed therefore carry both a fresh result owner and a write to Network,
ProcessAuthority, or DirectoryWrite respectively.

If an observer exists but neither a returned owner nor an incoming publishing
authority accounts for it, the system API used ambient authority and is
invalid.

Examples:

- a named file create, write, rename, unlink, or publication projects to an
  incoming DirectoryWrite;
- connect, send, FIN, or remote-visible close projects to an incoming Network
  or ConnectPermit lineage;
- process spawn or detach projects to an incoming ProcessAuthority or
  SpawnPermit;
- stdout and stderr writes project to their entry-supplied Output owners;
- endpoint-local state in a socketpair which never escapes has no additional
  caller footprint, but creating the pair still names its incoming
  SocketFactory, ProcessResources, or semantic quota;
- one returned socketpair endpoint carries its still-unpublished endpoint state
  in the result, while the factory and quota actions remain in the row;
  transferring an endpoint to another process projects to the explicit IPC or
  Process capability used by that transfer.

Factories must declare both facts:

```text
which incoming capability the factory reads or writes
which owned result leaves are fresh
```

Freshness is ordinary ownership construction provenance. It is not an
independent runtime root or permission relation.

Freshness alone does not close a published-child alias. If a factory installs a
named child under a parent and the parent can later reopen that same mutable
state, these operations are not automatically independent:

```text
child = create(parent, "x")
write(child, "A")
other = open(parent, "x")
read(other)
```

The API must select one ownership-complete answer before such a family ships:

- keep the child unpublished until an explicit consuming
  `publish(parent, child)` transition;
- move a keyed entry authority out of the parent while the published child is
  live, so the parent cannot reopen the same entry;
- return independent handles whose contract explicitly promises no
  cross-handle order or coherence, but only when use and release of one handle
  cannot invalidate the safety or lifecycle of another; or
- report a typed busy/conflict result or keep the mapping unsupported.

A published or remote-visible result is not `Fresh` merely because its native
handle is new. Its lineage continues from the DirectoryWrite, ConnectPermit,
ProcessAuthority, or other incoming authority which made it observable. This
lineage projects effects at enclosing function boundaries; it grants no
conflict or ordering permission. Two independent owned handle places may still
overlap under explicitly unordered handle semantics because ordinary ownership,
not lineage equality, decides overlap.

This removes the need for publication/escape ancestry as another identity
system. The API either transfers a structural authority, deliberately exposes
unordered independent handles with safe lifecycles, or refuses the mapping.

## 6. Quota and capacity are real ownership

Program-controlled, recoverable quotas are world state and require explicit
ownership. A Whitefoot budget, registered target slot, reserved port range,
device credit, or runtime pool cannot depend on a hidden process-global owner.

An abstract permit cannot promise that a later hosted allocation will succeed.
Host-wide fd tables, Windows handle capacity, ephemeral ports, kernel memory,
and limits affected by other processes may still report target exhaustion even
after Whitefoot admission. The design must distinguish:

```text
semantic quota
    controlled and exactly reservable by Whitefoot/runtime

host exhaustion
    reported by the target and not made deterministic by a logical permit
```

If a typed host-exhaustion result is required to participate in exact program
ordering, the target must reserve a real transferable resource or the API must
name the real arbiter capability. Otherwise the outcome remains an honest
environmental/resource result rather than a false promise supplied by a
portable permit. Its assignment to one of several otherwise independent
operations is then allowed environmental nondeterminism: if only one host fd
remains, overlapping `open A` and `open B` may let either one receive it. A
program which requires source-order allocation uses a real reserved credit or
one `&uniq` admission authority and thereby expresses the dependency.

The correct pattern separates short admission from long target progress:

```text
short admission
    &uniq Factory / Quota -> owned Permit

long operation
    consume owned Permit + payload -> fresh Resource carrying that credit,
                                      or the same Permit on failure
```

Conceptual source:

```whitefoot
let permits = reserve_connects(
  network: &uniq network,
  quota: &uniq socket_quota,
  count: 2_u64
);

let first = connect(permit: move permits.first, peer: peer_a);
let second = connect(permit: move permits.second, peer: peer_b);
```

The factory and quota loans end after permit creation. The two handshakes may
remain in flight independently.

Each outcome keeps the credit visible:

```whitefoot
match first {
  Connected { connection } => {
    let finished = finish(connection: move connection);
    // Every finished outcome contains the original owned ConnectPermit.
  }
  Failed { error, permit } => {
    // No connection exists; the same permit is immediately reusable.
  }
}
```

The exact result declarations remain API work. The invariant is not optional:
no success, failure, partial, cancellation, or release path loses track of the
credit by mutating an invisible pool.

This is a real ABI boundary. A normal call which keeps either unique loan until
the handshake result has not implemented short admission. The admission call
must finish before the long operation begins, or publish an
`authority_released` milestone which transfers an owned permit while the
remaining target work continues without the factory loan.

Failure returns the unused permit. A successful resource owns its credit until
an explicit close, finish, recycle, or other consuming lifecycle operation
returns the same reusable Permit. The Permit is the authority fragment; it
does not need to be checked against a hidden parent identity. Any restriction
on where it may operate is carried by its declared type and owned payload.
Mixing fungible permits transfers capacity and is legal; a non-fungible credit
must be a different or structurally bound type.

The default API does not consume a Permit into an arbitrary still-live Quota
and then ask the compiler whether their secret ancestors match. A quota may be
split into a remainder plus permits using the same rules as taking owned
elements from an affine container. Long-lived code reuses the returned permits
directly or stores them in an ordinary owned permit container.

Automatic release may close a resource, but it cannot silently return credit
to a quota owner held elsewhere: that would be hidden shared mutation. Its
declared disposition is instead one of:

- consume or burn the embedded credit while closing the native resource;
- require an explicit finish which returns an owned Permit; or
- move the Permit through a producer facet or return lane owned by the resource
  itself; the corresponding collector is another explicit owned value.

This is a type release contract, not runtime convention. Every ordinary
lifecycle edge selects exactly one of `Burn`, `ReturnResult`, or
`SendToOwnedReturnLane`. Pre-accept failure returns the original Permit.
Post-accept success, error, cancellation, and partial-progress races reach one
terminal disposition exactly once. If a resource has neither a legal implicit
`Burn`/owned-lane disposition nor an explicit finish result, the type is
finish-required and implicit release is rejected.

The default resource shape may use the first rule for convenience. A
long-lived program which must reuse a finite logical budget uses the explicit
permit-return path. The cost and lifecycle exist in the API because they exist
in the computation.

Dynamic counts use affine containers of permits. Fixed products preserve exact
lineage per leaf. A dynamic affine container records occupancy with the
language's ordinary element-ownership rules and carries a finite origin set for
its possible elements. Taking an element transfers one owned value and its
origin set; replace returns the displaced value; partitioning transfers
disjoint element ranges; loops and recursion compute a conservative fixed
point. A constant index or proven partition may retain a narrower summary.
Mixing permits from different formals widens the element origin set rather than
inventing freshness. An unknown origin remains unsupported for a boundary
which requires exact projection.

This summary does not grant or deny parallelism. Extracted permits in distinct
owned places may coexist because ordinary ownership proves the places
disjoint. No runtime lineage identity is added. Static permits and proof-only
authority may erase completely. Runtime capacities which are not part of
program semantics remain target backpressure and do not masquerade as
writer-visible quota.

## 7. The heap exception

The implicit process heap violates the general capability-closure law:

```text
functions may allocate without receiving an explicit Heap capability
```

Whitefoot tolerates this exception for code-generation convenience and records
it explicitly as `allocates(heap)`. Current OOM remains a TCB/resource failure,
not a recoverable state threaded through ordinary program semantics.

Fallible allocation, custom allocators, request budgets, revocable memory
capacity, and allocator selection should not inherit this exception. They
belong to a later explicit Allocator, Arena, MemoryBudget, and Permit design.
Unifying heap, arena, and allocation authority is a candidate project after
the I/O work closes; it is not part of this implementation.

## 8. Ownership is the only overlap authority

World effects declare actual external use. They do not grant overlap.

The overlap checker uses the existing judgments:

```text
data and control dependencies
resolved ownership places
ordinary memory footprints
shared and exclusive loans
own consumption
exit paths
```

The additional world checks are mode and completeness checks:

- a world write must name an `own` or `&uniq` capability formal;
- a world read may name a shared, unique, or owned capability formal;
- a system operation may use only authority supplied through its typed
  arguments;
- the body and release footprint must equal the written world row.

Taking a shared capability borrow is itself the API's visible promise that
concurrent observations may be linearized in any allowed order without
changing the program contract. Merely being non-consuming and
non-cursor-mutating is insufficient: two clock, metadata, or sensor reads can
observe externally changing state in an order the caller cares about. Such an
API takes `&uniq` unless arbitrary observation order is part of its specified
meaning, even when its effect remains `world reads(...)`.

A cursor, entropy sequence, accept backlog, send position, receive position,
output sequence, or lifecycle transition is a world write and therefore also
uses `&uniq` or `own`. This adds no commutativity tag: the borrow mode is the
language-level rule, just as it is for ordinary memory.

Different owned places are independent in the language proof. The source
semantics therefore gives no cross-capability ordering guarantee. If stdout
and stderr are redirected to one pipe, or two paths are hard links, writes
through their two independent owners may appear in any allowed overlap order.
That environment alias does not merge the owners after entry. A program which
requires one order must carry one common `&uniq` sequencing or admission
authority in its API.

This rule does not let the Whitefoot runtime mint false independence. When a
factory, split, reopen operation, or mapping implemented inside the trusted
system creates related handles, its contract must either provide genuinely
independent authority, expose the common sequencing authority structurally, or
withhold overlap. The language ignores aliases introduced outside the mapped
program; the mapping remains responsible for aliases it creates.

## 9. Structural facets, not relation tables

Independent sub-authorities are represented as ordinary structure:

```whitefoot
struct TcpParts {
  receive: TcpReceive;
  send: TcpSend;
}
```

Receive and send operations borrow disjoint fields. Same-direction operations
borrow one field uniquely. Whole-resource control consumes or uniquely borrows
the aggregate after all field loans end.

The API must not mint independently escaping aliases which later require a
hidden generative brand to prove they came from the same parent. Keep related
facets in one owned aggregate with scoped field borrows, or explicitly design a
future branded capability system. A one-way split is valid only when the
target contract proves that each child's operation, terminal transition, and
release cannot alter the other child. Shared native lifecycle, reset, shutdown,
or close authority fails that proof.

Long-lived send and receive work may need to move into separate suspended
frames and later participate in one whole-resource close. Scoped field borrows
alone do not solve that lifecycle. Such an API must keep the aggregate in a
structured parent frame, return an affine join token with both children, or
define an explicit consuming reunite operation which cannot mix children from
different parents. This is an open API gate, not grounds for an implicit root.

Disjoint ranges, protocol streams, queue lanes, and resource shards follow the
same rule. If two authorities are placed in independent ownership places, the
system contract must prove the mapping itself did not create an undisclosed
conflict. Environment aliases outside that mapping retain the cross-capability
nondeterminism above.

Independent facets and communicating endpoints are different API shapes.
Independent facets promise that their observable operations and terminal
transitions commute. A linear channel pair, cancellation pair, request/response
pair, or Permit return lane intentionally communicates: a transition through
one endpoint changes what the other endpoint can later observe.

Communication remains ordinary ownership rather than shared mutation. Each
endpoint is an owned value; send/request operations use `own` or `&uniq` on the
producer endpoint, receive/next operations use `own` or `&uniq` on the consumer
endpoint, and the channel's capacity, ordering, overflow, close, and terminal
rules are part of the type contract. Their world rows name the endpoint places.
One producer naturally preserves its unique program order. Multiple producer
permits are legal only under an explicit arbitrary-merge contract or structural
lanes. These protocol relations do not claim the endpoints are independently
reorderable and do not create a general family-pair permission table.

## 10. Ownership lineage

The compiler must complete general structural ownership lineage rather than
maintain an I/O-specific root system:

```text
Formal(place)
Fresh
Absent
Union(...)
```

Lineage is stored per affine resource leaf:

```text
Pair
  first  -> lineage A
  second -> lineage B
```

Move transfers lineage. Borrow refers to the current place. Construction stores
lineage by field or variant payload. Projection selects the corresponding
leaf. Return transfers lineage to the result. A system factory declares fresh
result leaves or a consuming continuation of an input owner.

Enum lineage remains correlated with its discriminator. If `Choice.Left`
carries the left formal and `Choice.Right` carries the right formal, a later
match recovers the exact origin in each arm; it does not receive
`Union(left, right)` in both. `Impossible` variants are also distinct from a
fixed-point `Bottom` which has not yet found a producing route. Only a real
join of the same result leaf forms an origin union. This same structure lets a
consuming split return several disjoint result fields without flattening them
back into one root.

Branches and recursion form structural finite unions and fixed points. Unknown
never becomes Fresh or empty; it causes an explicit unsupported boundary or a
conservative refusal. A preliminary recursive call is never guessed Fresh.

For dynamic affine containers, lineage is an element-origin set rather than a
fictional statically enumerated leaf. The container's existing occupancy proof
ensures that take, replace, partition, iteration, and release move each element
exactly once. Constant indices and proven disjoint ranges may refine the set;
dynamic extraction conservatively inherits it. This affects boundary effect
projection, not the ownership proof that two extracted values occupy different
places.

This is a language-defined, artifact-surfaced ownership judgment. It carries no
runtime identity, family relation, or ordering authority. It is the same
information required to release affine aggregate leaves exactly once.

The current compiler's flat zero-or-one capability origin record must be
replaced by this per-leaf representation. `Pair { first: ReadFile, second:
ReadFile }` is a required positive case, not a permanent language limit.

## 11. Completion ownership

Before target acceptance, the compiler-owned call frame owns one prepared
affine bundle:

```text
typed operation identity
resource owner or live resource loan
payload owners or live payload loans
stable target metadata
inaccessible result storage
quota / target-capacity permit where semantic
```

Admission is an internal ownership protocol, not a typed I/O outcome:

```text
PREPARED
    frame owns the complete bundle; target owns nothing

RECORD_RESERVED / WAIT_CAPACITY
    one generation-safe record is reserved; frame still owns the bundle;
    a target-capacity notification makes the exact submission retryable

ACCEPTED
    target atomically accepts the complete bundle; frame can no longer use it
```

Transient record, SQE, queue, or helper-credit exhaustion suspends and retries
the compiler-owned admission path. It neither calls the host operation nor
returns `WouldBlock` or another writer-visible I/O failure. The acquisition
order and rollback are fixed by the runtime: a failed pre-accept attempt
transfers nothing; abandoning an unaccepted record restores it and leaves every
owner and loan in the frame. Capacity release publishes the wake epoch used by
the waiting admission. A batch or dependency handoff must reserve all resources
whose partial ownership could form hold-and-wait, or reserve none.

Target acceptance transfers the complete bundle from the frame to the target.
The target then publishes monotonic milestones:

```text
accepted
result_ready
payload_released
authority_released
order_committed(subject)
terminal
```

`accepted` precedes every other milestone. `result_ready` means the complete
typed outcome bytes are initialized and immutable. `payload_released` means
the target can no longer read or write any payload loan. `authority_released`
means the target can no longer use the resource owner or loan. The payload and
authority milestones are independent when the target can release them at
different times. `terminal` implies `result_ready`, `payload_released`,
`authority_released`, and every required `order_committed` fact, and promises
that the target will publish nothing more.
Even cancellation and target failure publish a typed outcome before terminal.

`authority_released` says only that the target has stopped accessing the owner
or loan. It does not by itself prove that the world action reached a
linearization point which later same-subject work cannot overtake.
`order_committed(subject)` supplies that separate fact. A stream write may
publish it when the request enters an ordered target queue; an unordered target
or durability barrier may need native linking or terminal completion. A
same-subject dependency waits for both the required authority release and the
operation contract's ordering milestone. Waiting for terminal is always a
correct fallback, but not automatically the fastest one.

Each fact changes from false to true at most once; a target may publish several
new facts atomically in one event. Publication is release and runtime drain is
acquire. Duplicate, regressing, post-terminal, result-before-initialization, or
release-before-last-access publication fails target qualification and is a
runtime contract fault if reached.

These facts do not create authority. They return parts of the affine operation
bundle which acceptance transferred to the target. They are nevertheless
permission-critical: a result consumer requires `result_ready`, payload memory
access requires `payload_released`, and a new borrow or consume of the resource
requires `authority_released`.

Callable and operation summaries map milestones to exact formal or result
leaves, not just to one undifferentiated payload and authority bit:

```text
result leaf              <- result_ready(component)
payload formal/path A    <- payload_released(A)
payload formal/path B    <- payload_released(B)
authority formal/path C  <- authority_released(C)
world formal/path C      <- order_committed(C)
```

The first positioned-read slice has one destination payload component and one
file-authority component. A wrapper borrowing several leaves may publish them
at different times, so a dependent on A need not wait for unrelated B. The
checked callable contract carries this transformer through direct and indirect
calls. The runtime representation may coalesce facts proven simultaneous, but
cannot erase semantic distinctions needed by a caller.

A completion record is reusable only after terminal has been drained, the
typed result has moved to stable frame storage, and no authorized holder
remains: no target bundle, pending event, dependent-frame registration,
consume-wait registration, or live result-owner token. Old copied token bits
may still physically exist; the next generation must reject them before they
can observe or modify result storage. Generation validation is therefore part
of ordinary stale-reference safety, not proof that old bits vanished.

A normal call exposes its result only when the ownership required by its source
semantics is complete. A family-specific receipt or Source exists only when
split milestones or persistent identity are program semantics.

Compiler-derived release follows the same completion protocol. An enclosing
call does not return past a close, detach, abandon, or credit disposition until
that release reaches the completion milestone fixed by the resource type.
Unrelated releases may overlap. A detached abandon is legal only when its type
contract proves that no later caller-visible ordering, error result, owner, or
Permit depends on it. A meaningful close, durability, or quota result therefore
uses an explicit consuming finish operation rather than an implicit drop.

Target code never resumes writer code. It publishes bytes and milestones. The
completion runtime drains the event before a compiler-owned writer frame can
become runnable.

## 12. API surface patterns

### 12.1 Finite one-shot

Positioned file reads, finite datagram sends, one-shot timers, and finite
operations with fully owned inputs use ordinary calls. The compiler may overlap
them when existing data, place, loan, world-effect, and exit rules permit.

### 12.2 Stateful sequential resource

Output, a directory cursor, a same-direction stream state, an RNG sequence, and
similar state use `&uniq` one-shot operations or one explicit batch operation
under a single unique loan.

### 12.3 Persistent source

Listeners, periodic events, file monitoring, signals, hotplug, multishot
receive, and unknown-size streams use an owned Source or Subscription. The
source owns its persistent target operation, pool, event capacity, overflow
policy, and terminal lifecycle. `next` uniquely borrows the source and
returns owned shots or batches.

If a returned shot owns a pool slot or RecyclePermit, the Source no longer owns
the complete pool while that shot is live. `finish(source)` and implicit drop
cannot free or reuse the outstanding storage. The API must require every shot
to recycle before finish, consume an explicit container of every outstanding
permit at finish, or select an explicit abandon/owned-return-lane disposition.
A hidden reference count or shared mutable pool cannot repair a missing owner.

### 12.4 Request and response

A host callback which requires a writer decision becomes an owned Request plus
ResponsePermit delivered through a Source. Writer code runs on the Whitefoot
scheduler. A host protocol which requires immediate reentrant execution on the
original callback stack and cannot be adapted is unsupported.

### 12.5 Finish and recycle

Finish, close, abandon, recycle, quota handback, durable commit, peer
acknowledgement, and detach are distinct consuming operations whenever their
outcomes differ. Compiler-derived release cannot silently perform a stronger
successful lifecycle transition.

### 12.6 Cancellation

The default finite ordinary call has no writer-visible pending identity and is
not independently cancellable. Cancellation is available only when an API
explicitly maps it into owned authority, for example a structural pair of one
CancelRequest facet and one operation registration moved into the target
bundle. There is no ambient cancellation table or magic future handle.

The target contract defines one linearization between the cancel request and
ordinary terminal completion. If terminal wins, cancellation reports `TooLate`
and the ordinary outcome owns every value. If cancellation wins, the operation
still reaches terminal with a typed `Cancelled` or `Partial` outcome. That
outcome accounts for committed external progress and returns or burns every
payload owner, resource authority, and Permit exactly once. A cancellation
acknowledgement which precedes last target access is not terminal and cannot
release a loan.

The CancelRequest and operation registration are ordinary owned facets designed
to communicate through the target arbiter; their API specifies that race. They
do not authorize unrelated shared mutation or a generic relation table. Exact
signatures, reusable cancellation scopes, deadlines, and whether a selected API
offers cancellation at all remain part of its concrete design.

### 12.7 Foreign and callback mappings

A foreign boundary receives an explicit owned or borrowed mapping context for
every foreign global, registration table, signal disposition, callback queue,
and retained allocation it may observe or change. No foreign or target callback
executes Whitefoot writer code. During a synchronous foreign call, a
compiler-owned adapter may use an explicit call-scoped child loan and
qualification must prove that the foreign side retains nothing after return.
An asynchronous callback instead receives a closed owned bundle containing the
retained payload, target operation, event capacity, and terminal/unregister
protocol; it publishes into a Source.

Unregister is complete only at a target-defined terminal milestone after which
the foreign side cannot call again or retain the pointer. Signal and event
overflow policy is owned by the Source rather than hidden in a process global.
If a foreign library cannot expose or satisfy these facts, the mapping fails
qualification. A process-isolated adapter may turn it into explicit IPC, but
the compiler does not accept an unverifiable in-process escape.

## 13. Compiler implementation

### 13.1 Language and checking

1. Implement the selected flat tagged atoms and formal authority-path grammar.
2. Require every constructible effect to have a writable name and keep the
   callable boundary complete.
3. Check memory and world rows in both directions, including compiler-derived
   release.
4. Enforce world-write mode: `own` or `&uniq`, never shared.
5. Reject ambient authority and any factory whose observable quota,
   publication, or parent effect cannot be projected to a formal.
6. Replace capability-root cardinality with per-leaf ownership lineage.
7. Keep target action and completion milestones in system contracts and in
   artifact-surfaced callable summaries; these facts return transferred
   authority at proven milestones rather than minting new authority.
8. A first-class or indirect callable type must carry its complete memory and
   world rows, formal-subject substitution, result-lineage transformer, and
   completion contract. Until that callable boundary exists, an indirect call
   with world-bearing arguments or results is explicitly unsupported; it never
   receives an empty or global fallback effect.

### 13.2 Permission

Reuse the existing dataflow, memory footprint, operand-read, loan, consumed
place, and exit judgments. Remove family-fragment pair relations and Ordered
edges. Direct system calls enter the same ordinary call analysis.

The current whole-run rule is too coarse for:

```text
A writes out
B writes unrelated err
C later writes out
```

`A` and `C` cannot overlap, but `C` should become eligible as soon as
`A` releases `out`, even if `B` remains incomplete. Build a generic
dependency DAG from ordinary data, control, memory, and loan-release edges
instead of using an Output-specific Ordered batch.

### 13.3 Lowering

1. Keep completion enabled independently of compute `--par`.
2. Preserve pure-compute zero-link and zero-loop-tax boundaries.
3. Generalize selective stackless lowering from one single-block suspension to
   branches, loops, multiple suspension sites, indirect calls, and non-tail
   suspended children.
4. Allow a completed operation to activate compiler-owned dependent target work
   directly when no writer frame must run first, but only after closing the
   bounded-record handoff below.
5. Fuse static same-authority sequences under one unique operation when a
   qualified target offers `writev`, request linking, or a finite batch
   without changing partial/error semantics.
6. Keep direct and inline depth-one specializations of the same source call.
7. Treat multishot as persistent Source lowering with a target-owned pool, not
   as fusion of a finite list of ordinary one-shot calls.
8. Give each dynamic function activation at most one reusable suspension frame
   or an equivalent proven bound. A call which does not suspend must erase that
   frame and retain the direct ABI.
9. Do not materialize proof-only authority. Do not pass a native handle twice
   as both handle and capability. Closed-world wrappers must remain
   specializable; opaque ABI pointer and register-spill cost receives its own
   bound and benchmark.

## 14. Runtime implementation

The new completion runtime remains separate from the pre-existing compute-par
runtime. In prose and diagnostics, call `wf_completion_slot` an I/O completion
record; it is not `wf__par_slot`.

Retain and harden:

- typed target requests with no writer function pointer;
- generation validation before result bytes change;
- one terminal publisher;
- separate result, payload-release, authority-release, and terminal facts;
- drain-before-writer-ready;
- exact-token consume-to-park handshake;
- one wake epoch and correct multi-waiter publication;
- target callbacks which never execute writer code;
- zero-helper progress for qualified nonblocking, native-completion, inline,
  or otherwise bounded direct-progress operations;
- bounded queues and operation storage.

Rework:

- remove the special Ordered Output root, batch, and 16-member arrays;
- remove language meaning from the current 64-record bridge capacity;
- derive or configure bounded capacity from target/runtime policy and measure it;
- compare whole-batch all-or-none admission against streaming admission and
  dependency-driven submission;
- close the bounded dependency handoff before enabling successor activation:
  either reserve successor capacity, move predecessor results into stable frame
  storage and consume their records during drain, reuse one record along a
  dependency chain, or prove another no-hold-and-wait protocol;
- distinguish semantic quota permits from runtime target backpressure;
- let permit pools use batch refill, per-lane caches, and explicit recycle
  without writer-visible shared mutation;
- define any runtime-owned atomic sequencer used by dynamic producer permits as
  one typed trusted primitive; it may implement native admission but grants no
  shared writer-visible mutation. Such permits are valid only when each owns a
  real reservation and the API explicitly permits the target to choose their
  merge order. If consumer-visible order must follow program order, use one
  `&uniq` sequencer authority or structurally separate lanes instead. Moving an
  old Ordered relation table into the runtime is not an implementation.

## 15. Target implementation

### Linux

Keep real io_uring operation submission, CQ completion, and the combined native
wait set. Requalify every API after the ownership rewrite. Do not replace
current-position Output semantics with positioned write semantics. Measure
linked requests, batching, registered resources, and multishot against the best
direct path.

### macOS

Keep typed target fallback for facilities without a qualified native completion
path. Zero-helper mode remains required only where the chosen operation cannot
block the sole scheduler beside work needed to unblock it. A potentially
blocking fallback with independent runnable work must first obtain helper
credit or use another target facility. Helper count is target policy, not a
language mode. Compare bounded helper, dispatch, readiness, direct, and
completion paths per operation.

### Windows

Keep the IOCP/OVERLAPPED implementation fail-closed until it executes on a
Windows runner and closes the persistent multi-waiter wake proof. Cross-linking
alone is not execution evidence.

## 16. Verification plan

### 16.1 API correctness

Every system API receives a contract matrix covering:

```text
success
empty input
partial progress
EOF / terminal state
typed resource failure
invalid target representation
owner and permit disposition on every result
ordinary memory changes and untouched tails
world effects and publication
release, finish, abandon, and recycle
```

### 16.2 Concurrency and runtime robustness

Use deterministic hostile schedules and long randomized stress:

```text
completion before wait
stale generation
duplicate terminal publication
drain / consume / park races
multi-waiter wake
capacity exhaustion and refill
permit split, return, loss, and duplicate prevention
partial I/O and cancellation races
single-thread and multiple-thread progress
dependency activation without unrelated group joins
```

Run ASan, UBSan, TSan where supported, strict C warnings, and native target
tests. Stress complements deterministic race construction; it does not replace
it.

### 16.3 Performance

Compare against the best native and Rust shape, not a naive blocking baseline:

```text
depth 1 through target saturation
static and dynamic keys
payload sizes and batch sizes
single and multiple producers
whole-capability uniq versus short admission plus permits
central pool versus per-lane credits and batch refill
direct, inline, linked, batch, multishot, and helper paths
CPU cycles, atomics, cache misses, context switches
p50 and p99 latency
live pinned bytes and buffer-byte-time
extra ABI arguments and spills
```

The two first decisive experiments are parallel open/connect admission and
dynamic multi-producer delivery. Failure to match the native ceiling reopens
the responsible capability granularity or API.

## 17. Current candidate disposition

### Retain

- completion-only source model;
- ordinary call surface;
- no writer callback;
- no correct-path false-claim cost;
- typed outcomes and empty-I/O no-host behavior;
- completion record generation and publication protocol;
- drain-before-resume and lost-wake fixes;
- pure-compute completion-runtime link boundary;
- Linux io_uring work, macOS typed fallback, and Windows IOCP foundation;
- measurements which compare direct, completion, helper, and wake paths.

### Delete or replace

- mixed REGIONID/IDENT operands inside one memory effect row;
- shared Output and shared DirectorySource mutation;
- Ordered Output batches and OutputBytes edges;
- family Free/Ordered/Exclusive relation as a second concurrency system;
- logical-root identity used independently of ordinary ownership places;
- the flat zero-or-one capability origin record;
- the 16-member ordered limit as a design concept;
- the 64-member free batch limit as a language-like claim;
- completion grouping which waits for unrelated members before releasing a
  dependent operation.

### Generalize

- ownership lineage to every affine resource leaf;
- world effects to aggregate-safe formal subjects;
- compiler stackless state machines;
- dependency-driven completion scheduling;
- quota, permit, pool, lease, Source, and recycle lifecycles;
- API and runtime tests across every result and platform.

## 18. Implementation sequence

1. **Safe core isolation.** Before the language gates close, isolate and qualify
   the generic completion state machine plus positioned `read_at`. Its gate has
   an explicit link-unit list containing the generic runtime, file adapter, and
   dedicated core/read probe but not the legacy bridge, Output code, or logical
   root implementation. A dependency/link map is the primary isolation proof;
   absence of legacy symbols in the final binary is an additional guard, not
   the proof by itself.

   The gate must pass generation/stale publication, exactly-one terminal,
   result-before-publication visibility, drain-before-consume, exact-token
   consume-to-park, capacity exhaustion/retry/wake, single- and multi-waiter
   schedules, and positioned-read empty/full/short/EOF/error/untouched-tail
   cases. Stress plus ASan/UBSan and TSan supplement the deterministic cases.
   This slice may not add dependent-successor, quota, facet, or new stackless
   semantics.
2. **Language gate.** Install the selected effect-domain spelling and world
   subject paths; close exact generic release transformation,
   capability-closure law, and quota semantics. Write positive and negative
   examples before each affected compiler slice.
3. **Ownership lineage.** Implement per-leaf lineage through construction,
   move, projection, match, replace, result substitution, recursion, and
   release. Make `Pair<ReadFile, ReadFile>` execute.
4. **Effect checker.** Add the world domain, release contribution, call
   substitution, contract equality, entry support, diagnostics, and ledgers.
5. **API correction.** Convert stateful world operations to `own`/`&uniq`;
   add typed facets, factories, quotas, permits, finish, abandon, and recycle
   contracts for the selected file slice.
6. **Permission and IR.** Remove family relation permission and build ordinary
   dependency edges and milestone requirements.
7. **Lowering.** Generalize stackless state machines and implement dependent
   target activation, batching, and direct specialization.
8. **Runtime cleanup.** Remove Ordered Output machinery, revise bounded
   admission, and keep the proven completion/wake core.
9. **Targets.** Requalify macOS and Linux; execute and finish Windows.
10. **Evidence.** Complete API matrices, hostile race tests, stress, sanitizer,
   conformance, maintained-program, link-boundary, and performance gates.
11. **Activation.** Only after exact owner review, convert the candidate
    specification to ACTIVE, archive the previous bytes, record the protected
    boundary, and run canonical `make check` on the exact revision.

## 19. Open design gates

The effect-domain spelling and formal authority-path surface are closed above.
The following questions are not silently decided by this plan:

1. exact factory, quota, permit-return, pool, lease, and revocation APIs;
2. whether a mutable child which can be reopened through a parent uses a keyed
   lease, explicitly unordered independent-handle semantics, or an unsupported
   mapping boundary;
3. the boundary between exactly reservable semantic quota and nondeterministic
   hosted target exhaustion;
4. bounded completion-record handoff across dependency activation;
5. typed Source, Subscription, Request/ResponsePermit, and ExternalMapping
   families, including outstanding-shot pool ownership and finish;
6. the static and dynamic authority-splitting rules needed to match native
   parallel open/connect and multi-producer ceilings;
7. whether unordered dynamic producer permits are needed at all, and, if native
   MPSC measurements require them, their real reservation, consumer-visible
   merge semantics, world subject, and typed runtime sequencer;
8. affine facet reunion or structured-parent ownership for long-lived
    bidirectional work;
9. full stackless ABI, frame-placement, reuse, and direct-path erasure;
10. pre-accept ownership, acquisition order, rollback, capacity derivation,
    and streaming versus all-or-none admission;
11. Windows native execution and multi-waiter wake qualification;
12. shared memory, MMIO, DMA, and GPU representation proof, which ordinary
    Whitefoot borrows cannot solve when an external writer remains live;
13. dynamic affine-container occupancy and effect-projection rules for mixed
    and recursively produced capability origins;
14. complete foreign mapping contracts for hidden globals, synchronous and
    asynchronous callbacks, retained pointers, unregister, signals, and
    process isolation;
15. cancellation and deadline API signatures, reusable authority, race
    outcomes, partial progress, and per-result owner disposition;
16. a symbolic generic release-effect transformer and instance-level exactness
    for types whose world-bearing leaves or release rows depend on type
    arguments;
17. indirect-callable world rows, subject substitution, result lineage, and
    completion contracts;
18. the exact cleanup contribution on every ordinary control-flow edge;
19. the scalable runtime encoding of per-formal/per-leaf result, payload, and
    authority release milestones, plus per-subject `order_committed`;
20. protocol rules for owned communicating endpoint pairs, including capacity,
    merge order, close, and terminal behavior; and
21. target qualification boundaries for regular-file positioned reads versus
    device, virtual-file, and metadata-observing APIs.

An open gate may block the affected API or optimization. It does not authorize
a hidden tag, ambient capability, shared mutation, global serialization, or
unsafe fallback.

## 20. Completion criteria

The I/O work is complete only when:

- every externally observable ordinary world action names explicit incoming
  authority; a returned fresh owner is sufficient only for state which stayed
  unpublished and completely framed until return;
- every program-controlled recoverable quota has an explicit owner and
  handback path;
- automatic release has an explicit credit disposition and never mutates a
  separately held quota owner;
- a Source cannot finish or release storage still owned by an outstanding shot,
  slot, or recycle permit;
- no logical permit claims to reserve a host resource which the target did not
  actually reserve;
- any unreserved host exhaustion is specified as environmental nondeterminism,
  while code requiring deterministic allocation carries a real arbiter;
- no shared world write is accepted;
- memory and world effects are distinct in source and identical in frame,
  substitution, exactness, and read/write logic;
- no optimization treats a world observation as stable merely because
  Whitefoot code exhibits no world write;
- permission relies on ordinary ownership places and loans rather than a
  family relation system;
- fixed multi-root aggregates retain exact structural lineage, and dynamic
  affine containers preserve occupancy plus conservative element origins;
- completion preserves every owner and loan through its declared milestones;
- pre-accept capacity waits leave the complete bundle outside target ownership,
  transfer nothing on retry, and have a proven wake and rollback path;
- callable milestones release exact formal/result leaves rather than one
  undifferentiated bundle;
- result, payload, authority, terminal, cancellation, and record-reuse
  transitions form one qualified monotonic ownership state machine;
- same-subject successors wait for the qualified ordering milestone rather
  than mistaking early authority release for world linearization;
- dependency activation cannot deadlock while completed predecessors occupy
  bounded completion records;
- no target executes writer code;
- pure computation links and executes no completion machinery;
- all three hosted targets have honest qualification status;
- API correctness, hostile concurrency, stress, sanitizer, conformance, and
  maintained-program gates pass;
- measured fast paths match the best relevant native/Rust shape or the
  responsible design is reopened; and
- the exact revision receives owner approval before merge to `main`.

## 21. Deferred successor: allocation unification

Heap allocation is the deliberate ambient-authority exception in the current
language, while arena and future quota/permit designs expose more ownership.
After I/O closes, investigate one unified allocation model covering heap,
arena, allocator selection, fallible capacity, budgets, recycle, and target
memory pools. That project starts from the same capability-closure and frame
principles, but it does not delay this I/O implementation.
