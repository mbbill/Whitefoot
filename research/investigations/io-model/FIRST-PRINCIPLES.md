# Whitefoot I/O from first principles

Status: LIVE DISCUSSION DRAFT. No entry in this file is a language decision
until the owner marks it settled in discussion. This file records the
derivation as it happens; it does not treat the active specification, the
current compiler, conventional compiler design, conformance migration cost,
or implementation effort as authority.

This file belongs beside the I/O investigation because it is the working
derivation for that capability. It is removed after every settled conclusion
has superseded the corresponding part of `DESIGN.md`, or when the investigation
is abandoned.

## Method

Each decision starts from the Whitefoot constitution and the physical problem.
It then compares every known design that could satisfy those premises. A
current implementation can supply a counterexample or measurement, but it
cannot select the design. A familiar design receives no preference for being
familiar, and an unfamiliar design receives no preference for being novel.

Once the owner marks a decision settled, later discussions take its conclusion
and derivation as premises. A settled decision is reopened only by a concrete
counterexample, by refuting one of its load-bearing premises, or by deriving an
alternative which better satisfies the constitution. Later work does not
restart from conventional APIs or from the active implementation merely
because they are familiar.

Before opening each later question, the discussion rereads the settled record
and the corresponding owner conversation. Those conclusions are the starting
state, not optional background. A proposal which repeats a rejected blocking,
writer-scheduled, globally serialized, or implementation-shaped form without
first refuting the recorded reasoning is not a new candidate.

Every design search starts at the highest performance shape the constitution's
safety and proof commitments permit. It first derives the theoretical ceiling,
then asks how to realize it. A conservative fallback, compatibility mode,
existing implementation shortcut, migration convenience, or familiar host API
cannot be the starting candidate and cannot set the architecture. Such a
fallback is considered only after the best design is known, and only as one
target's bounded degradation rather than as language semantics.

### Mandatory two-layer separation

Every later question is classified into exactly one of two layers before it is
reasoned about.

1. **External-to-language mapping.** This layer asks whether an external
   operation has been completely represented as an ordinary Whitefoot API:
   owned values, borrows, lifetimes, state transitions, typed outcomes,
   completion milestones, split or shared authority, and release. A missing
   identity, lifetime, completion state, or authority relation is an API
   mapping defect and is fixed here.
2. **Ordinary API use.** Once the mapping exists, source code sees only normal
   Whitefoot objects and functions. It knows no file, socket, device, host
   thread, syscall, readiness mechanism, or external world. Calls obey exactly
   the same value, control, ownership, lifetime, effect, and overlap rules as
   every other call. A problem at this layer is either an ordinary language
   problem or proof that the API mapping failed to expose a necessary ordinary
   relation.

Arguments may not cross this boundary. In particular, no source-level rule or
special call form is justified by saying that a value "really" names a file or
socket. A function taking two file capabilities is reasoned about exactly as a
function taking two ordinary affine objects. Questions about what those
objects must own, borrow, split, return, or complete belong to the mapping
layer and are settled before ordinary code is considered.

The premises used in the first discussion are:

1. P0: preserve the largest legal optimization space and avoid writer-chosen
   scheduling decisions that a compiler or runtime can make better from proof
   and target state.
2. W1: the accepted default shape must already be the fast shape; a writer
   must not need to discover a hidden scheduling convention or manually tune
   how many requests to keep in flight.
3. W3, T1, and T2: the writer cannot acquire an escape that can lose an
   operation, reuse loaned storage, race a completion, or lie about an effect.
4. External work has its own clock. Performance requires the machine to keep
   independent external operations in flight while useful compute or other
   external work proceeds.

## A false claim never shapes the correct path

Status: SETTLED IN DISCUSSION on 2026-08-26.

A retained `claim` is not input validation, an assertion, an expected failure,
or a writer-requested abort. It is an independently true theorem which the
checker cannot derive and which complete human review has approved as true on
every execution that reaches it. A properly reviewed program therefore cannot
execute a false `claim`. Reaching the trap means that the proof or its review
was defective; it is not another valid outcome of the program.

Whitefoot paid for this premise by making `claim` unusually narrow and by
requiring complete human verification of every retained theorem. The rest of
the system must use the premise. Treating a false `claim` as ordinary control
flow would discard the strongest consequence of that work while retaining all
of its cost.

The first-principles consequence is strict: an execution which cannot occur in
a correct program has no budget on the correct path. No permission,
optimization, source shape, target choice, or fast path may be withheld in
order to stabilize, reproduce, clean up, or otherwise improve the observables
of an execution containing a false `claim`. If concurrency or world operations
make more of that defective execution schedule-dependent, the promise for the
defective execution widens; the permission granted to correct executions does
not narrow.

In particular, the possibility of a false `claim` may not cause any of the
following:

- a claim-free eligibility gate or serialization of code which contains a
  `claim`;
- a trap-latch read or trap-specific submit permit, gate, epoch, shared
  counter, or scheduler poll whose sole purpose is guarding against a false
  claim on a normal operation path;
- an extra copy, allocation, reference count, pin, metadata field, queue hop,
  completion transition, or wake on that path;
- selection of a slower backend or refusal of an otherwise safe optimization;
  or
- an ordering or rollback protocol whose purpose is to make a defective run's
  external effects deterministic.

Work which an operation independently requires for its normal ownership,
lifetime, target safety, or measured fastest implementation is not a trap
cost. The prohibition applies when the work exists only because a retained
theorem might be false.

The false branch is an isolated cold fail-stop path. The failing continuation
never resumes writer code; an operation whose safety proof depends on that
`claim` may not be moved before the check; the runtime performs the required
diagnostic action and enters whole-process termination without language
unwinding, cleanup, or completion draining. Other lanes and target-owned
operations may continue to race until the host actually stops the process.
Every accepted operation still obeys its own ownership and external-effect
contract, but the schedule may select which of those permitted effects became
visible and none is rolled back.

Memory safety remains unconditional. It is preserved by the ownership and
loan rules already required on the normal path, together with qualified host
process teardown which prevents target-held storage from being reused while
the target can still access it. Any additional mitigation needed only for an
abort belongs entirely to the cold trap path or the host trusted base; it may
not surcharge correct execution.

This is an explicit anti-regression rule. Previous designs repeatedly let trap
handling leak into parallel eligibility, scheduling, submission, and resource
lifecycle. A proposal which asks a correct execution to execute, store,
synchronize, copy, wait, or lose optimization because a reviewed theorem might
be false is rejected on that fact alone. Keeping the impossible path isolated
is what makes the compiler and runtime both smaller and stronger; failing to
exploit the impossibility wastes the central benefit of the `claim` discipline.

## Completion is the only language-level I/O model

Status: SETTLED IN DISCUSSION on 2026-08-25.

External interaction has two irreducible directions: the program makes an
operation available to the world, and the world later makes an outcome
available to the program. The language model is therefore submission plus
completion. An unrequested event is represented by a capability-backed event
source whose next outcome enters through the same completion boundary; it does
not require a callback or handler language.

A blocking API binds one outstanding world operation to one occupied execution
thread. That consumes a compute resource while the world, not the program, is
making progress. It also creates a second writer-visible function family and
makes the writer choose whether one operation deserves blocking or
asynchronous treatment. Both costs oppose P0 and W1.

A readiness API reports that a later operation might make progress rather than
reporting the operation's result. Readiness can be a useful target mechanism,
as can a blocking helper thread, polling, an interrupt, a hardware queue, or a
completion port. None becomes a second language model. A target adapter may use
any of them to implement the one submission/completion contract.

When one operation has no independent work with which to overlap, a target may
submit it and immediately wait for the normal call's ownership-complete
requirement. That is one degenerate schedule of the completion model, not a
blocking API exposed beside it.

This selection follows the constitution rather than becoming an axiom. If a
different model is derived or measured to provide higher theoretical and
actual performance while preserving the same safety, proof authority,
AI-writability, target freedom, and single optimal source shape, Whitefoot
replaces completion with that model. Familiarity, compatibility, and
implementation cost neither preserve completion nor justify an alternative.

## Completion-I/O API shape

Status: SETTLED IN DISCUSSION on 2026-08-26. Exact declaration names and
grammar spellings remain future specification work; the semantic surface
selection and its derivation are settled here.

`Source`, `Sink`, `next`, `put`, `finish`, `command`, `receipt`, `ticket`,
`scope`, and related capitalized or code-form names in this section are
analysis roles. They do not select declaration names, type names, operation
names, or grammar spellings.

This section uses **normal call surface** in one narrow sense: the writer uses
the same function-call grammar, ordinary argument evaluation, types,
ownership, lifetimes, effects, and typed outcomes as for any other Whitefoot
function. The phrase does not mean a blocking host call, source-line
serialization, immediate native-ABI return, absence of an internal operation
record, or absence of suspension lowering.

Four questions remain separate at every such call:

```text
source surface       what the writer writes and the checker judges
logical completion   when the result and returned authority become usable
overlap permission   which other statements may progress before that point
target lowering      whether a host thread waits, a queue is used, or the
                     operation completes inline
```

For a finite, one-shot, structurally owned operation whose pending identity
does not become program data, Whitefoot selects the normal call surface. The
result is not an early placeholder: it becomes an ordinary usable value only
at the operation contract's declared ownership-complete milestone. While that
call remains incomplete, the compiler may progress other statements only when
their values, control, loans, authority fragments, and completion obligations
permit the overlap.

For a persistent, unsolicited, unbounded, or multishot world relation, and for
an unknown-size relation whose complete result is delivered as a bounded
sequence, Whitefoot selects an owned family-specific Source or Sink role. A
finite unknown-size one-shot may instead return operation-owned or
pool-selected storage when its contract bounds ownership and completion.
`next`, `put`, and lifecycle operations still use the normal call surface.
Physical requests, buffers, queue depth, prefetch, batching, and rearming
remain target/runtime policy rather than writer scheduling knobs.

Pending identity becomes a writer-visible affine value only when that
identity is itself program semantics: it must be stored or returned, selected
dynamically, individually cancelled or queried, kept beyond its structural
scope, or used to observe independent milestones. Such a value is a
family-specific command, receipt, ticket, operation set, or milestone guard,
not a universal `Pending<T>` wrapped around every I/O result.

A writer-visible scope or transaction likewise exists only for real
collective semantics such as winner/loser selection, atomic commit,
collective cancellation, or error aggregation. A scope whose only purpose is
batching or queue depth is compiler-generated. Writer callbacks are not a
completion surface; a target callback may publish state and wake the scheduler
but never executes writer code.

### Every in-flight operation is a closed affine authority bundle

Status: SETTLED IN DISCUSSION on 2026-08-26. `Operation` in this section is an
analysis name for the logical ownership unit. It does not select a
writer-visible type, syntax, call shape, or runtime allocation.

The target may accept an operation only after one closed affine bundle exists
for everything the target can still access. The bundle logically contains:

```text
one stable operation identity
+ the exact resource authority required by this operation
+ ownership or valid loans for every payload the target may access
+ stable target metadata and any required registration or pin authority
+ the unique inaccessible storage for results not yet produced
```

Every handle, pointer, descriptor, callback context, or DMA mapping reachable
by the target must ultimately reach either state owned by this bundle or state
protected by a live loan held by it. Until the target has published the exact
milestone which ends an access, the corresponding owner, loan, registration,
metadata, and uninitialized result storage may not be destroyed, moved,
reused, read through an uninitialized path, or accessed in conflict.

Moving the whole resource into every operation is not the common rule. It
would serialize two position-explicit reads of one file even when their
destinations are disjoint and the operations share no cursor. Each operation
instead holds the smallest authority its resource contract proves sufficient:
a shared read authority, an exclusive authority, a range or role fragment, an
ordered or keyed reservation, or the whole resource only when the family
proves no finer safe authority relation. Consuming families such as stream
receive, listener accept, datagram receive, and directory iteration can retain
ordered or otherwise family-defined reservations rather than taking the whole
resource. Resource identity alone neither grants overlap nor denies it.

Payload capture likewise has no universal borrow-versus-move answer. A shared
loan permits one immutable buffer to feed several independent zero-copy
operations without copying. An exclusive loan protects caller-owned
destination storage. Moving an owned payload lets its producer disappear and
lets a queue or operation carry the payload without retaining that producer's
owner. A target-selected receive buffer is represented by authority over a
managed pool until a completion transfers one selected initialized buffer.
The operation contract fixes the capture mode; it is not a writer scheduling
choice.

Completion is not one Boolean ownership instant. A result can become available
before the target releases its payload access, and a long-lived operation can
deliver one payload while remaining active for later deliveries. Linux
zero-copy send, for example, can publish the send result before a later
notification permits buffer reuse, while multishot operations and provided
buffer rings can deliver several independently owned buffers before the
operation becomes terminal. The common model therefore distinguishes at least
result availability, each payload's release or delivery, and the terminal
witness that no later runtime publication or target access through that
operation bundle can occur. A simple operation may make these milestones
coincide; an already accepted external effect may still become visible later
when the family contract permits that behavior.

The ownership bundle imposes no common heap allocation, reference count,
global registry, coordinator hop, payload copy, or whole-resource lock. An
inline completion may establish and discharge the same logical bundle entirely
in registers or ordinary local storage, allowing the operation record and
result cell to disappear. Only an operation which remains target-owned after
submission needs stable runtime or target storage, selected by later lowering
and target evidence.

This conclusion fixes the ownership closure which every later surface,
lifecycle, and lowering rule in this section preserves. The following
subsections add authority, attribution, abandonment, and representation rules
without weakening this closure.

### Resource identity and operation concurrency are separate

Status: SETTLED IN DISCUSSION on 2026-08-26.

Every resource instance in a family has one stable logical identity and
lifetime group which owns the common backing object and whole-resource
control. Identity keeps the resource alive and defines where close, release,
fatal failure, and whole reconfiguration act. It is not by itself either a
mutex or permission to overlap operations.

Operation concurrency is determined by family-defined authority fragments.
A long-lived facet such as one direction of a full-duplex connection and a
short-lived reservation held by one in-flight operation are the same
ownership concept at different durations. Neither is a second kind of
lifetime, an ambient runtime permission, or a copy of the native handle.

A resource family can relate simultaneously live fragments in exactly three
ways:

```text
freely coexisting
    both operations may progress without an ordering edge

ordered coexisting
    both operations may be pending, but the resource contract fixes the
    order in which their external actions or consumed inputs are attributed

mutually exclusive
    the later fragment cannot be admitted until the conflicting authority
    has returned
```

A full-duplex connection demonstrates the first two relations. Its receive
and transmit facets freely coexist while sharing one connection lifetime.
Two operations in one transmit direction instead hold ordered reservations:
the target may keep both pending and may publish their completions in either
order, but completion order never selects the byte order. Whole close or reset
requires mutually exclusive authority over the common lifetime group.

A position-explicit file read demonstrates freely coexisting fragments on one
resource identity. Several such reads can share read authority because they do
not use one cursor. A file write contract must additionally account for every
state it can change, including any length, EOF, metadata, allocation, or
durability state relevant to the promised semantics; disjoint byte ranges
alone do not prove that two writes freely coexist.

The common model does not require one generic coordinator object. A fragment
which the compiler and resource contract prove compatible may be a wholly
erased fact. A family which needs dynamic ordering or admission may keep the
minimum state local to that resource or use a qualified target facility.
Unknown or unclassified families conservatively expose only mutually
exclusive whole-resource authority. This preserves one ownership model
without charging every operation for a global registry, lock, reference
count, or generic reservation engine.

The same semantic fragment has four possible representations, selected from
least to most persistent:

```text
static proof
    compiler forms and erases the fragment; no runtime admission cost

small bounded dynamic choice
    compiler emits a local compatibility/versioning guard

dynamic or unbounded pending set
    resource-local state admits, orders, or delays operation fragments

persistent authority delegation
    an ordinary long-lived facet value limits what another subsystem may do
```

Writer-visible splitting therefore serves lasting delegation, not the local
request “make these two calls concurrent.” Known positional operations do not
pay for an interval tree or generic coordinator; only a family whose dynamic
pending set needs such arbitration owns it. Close takes whole-lifetime
authority, and structural operations such as truncate take every content,
length, and metadata fragment their family contract says they can disturb.

The following subsections select when a fragment remains compiler-owned, when
it becomes a long-lived Source/Sink facet or family ticket, and how source
outcomes are attributed. Dynamic admission remains family-specific because no
one interval, queue, or protocol representation covers every resource.

### Event attribution is family semantics, not completion scheduling

Status: SETTLED IN DISCUSSION on 2026-08-26.

An external occurrence, its attribution to one logical reservation, runtime
publication of the resulting completion, and execution of writer code are
separate events:

```text
physical history
    -> source-family linearization and attribution
    -> operation-specific outcome
    -> completion publication
    -> writer continuation execution
```

Only the attribution step decides which ordinary value receives a consuming
outcome. Target completion order, the lane which harvests a completion, and
the lane which next runs writer code do not acquire that authority.

The minimum counterexample uses two receive reservations with different
storage and policy:

```text
reservation a: 4-KiB destination, budget/context A
reservation b: 64-KiB destination, budget/context B
source publishes X = one 32-KiB message, then Y
```

An ordered family maps `X` to `a` and `Y` to `b`, so `a` receives the
family's typed truncation outcome. Completion for `b` may still be published
first. If completion race instead gave `X` to `b`, it would change truncation,
budget use, and ordinary continuation context. The target has already written
a particular buffer, so the runtime cannot repair the mistake by relabeling
two completion records afterward. Attribution is therefore family semantics.

Every event-source family therefore defines its own attribution rule. For an
ordered consuming source, the nth source event after the family's declared
linearization point belongs to the nth eligible logical reservation. Later
reservations may publish completion and resume writer code before earlier
ones; ordered attribution imposes no completion-order or resume-order wait.
Once an event has been attributed, a later notification race cannot transfer
it to another reservation.

The family's linearization point must describe what the target can actually
observe. A listener, for example, does not claim a portable wall-clock order
between simultaneous network arrivals. It may define its event sequence by
the order in which its qualified source engine dequeues and publishes accepted
connections. The target may choose any order which the physical history and
family contract allow before that point; after it has chosen, the scheduler
may not perform a second reassignment.

Logical reservations need not correspond one-for-one with physical target
requests. Anonymous accept slots, provided-buffer pools, multishot requests,
readiness pumps, and target-native queues may remain filled independently of
writer progress. Each complete owned event is moved from that target capacity
to the reservation selected by the family rule. This permits physical
concurrency and zero-copy ownership transfer without making host request or
callback identity part of source semantics.

Ordered consumption is not the universal event model. A family may instead
declare unordered matching, keyed or partitioned consumption, broadcast or
persistent observation, or a coalescing reduction. Such behavior must be an
explicit property of that family, including the allowed attribution, rather
than an accident of backend completion timing. A source whose attribution rule
has not been defined may have at most one outstanding consumer; the system
does not guess either ordered tickets or unordered races for it.

The capacity, cancellation, Source/Sink, and writer-surface consequences of
this attribution law follow below.

### Why one-shot calls and Source/Sink are the minimal surfaces

Status: SETTLED IN DISCUSSION on 2026-08-26.

Consider a finite positioned read followed by independent compute. The
spelling is schematic rather than proposed grammar:

```text
header = read_at(config_file, offset, destination)
digest = hash(default_policy)
policy = combine(header, digest)
```

The read operation may remain target-owned while the compiler runs `hash`.
`header` is not an initialized placeholder during that interval; only the
operation owns its result storage. `combine` cannot run until the read reaches
the milestone which makes `header` usable.

Two independent world operations expose the same rule more directly:

```text
left  = read_at(left_file,  left_offset,  left_buffer)
right = read_at(right_file, right_offset, right_buffer)
value = combine(left, right)
```

After both semantic calls have been reached and their arguments, control,
loans, and authority fragments are available, the compiler may submit both.
Neither `left` nor `right` is a placeholder value; `combine` is the first real
result dependency. Reusing one exclusive capability or overlapping one
destination would instead create an authority or loan edge and prevent the
second handoff until that edge is satisfied.

An explicit generic start/finish surface can force this schedule:

```text
pending = start_read(...)
header = finish(pending)
digest = hash(...)
```

The normal call surface can reproduce the same schedule by waiting at the
same position, but it can also delay the wait until `combine`. The explicit
finish edge cannot in general be removed after the writer states it. It
therefore reduces the legal schedule set without adding authority or result
information for a finite structurally owned operation. This is why generic
writer-placed `await` or `finish` is not the default.

A listener provides the opposite lower bound. It can produce an unbounded
sequence of fresh connections, and the fastest target may represent that
sequence with one multishot request or an anonymous pool of accepts unrelated
to logical `next` calls. No finite one-shot result can own that continuing
relationship. An owned Source therefore holds its source authority, engine,
finite capacity, attribution state, and terminal condition across deliveries.
A Sink symmetrically holds an ordered or otherwise family-defined output
relationship across payloads.

Source/Sink is not imposed on finite random access. Routing one positioned
file read through a persistent writer-visible queue would add lifecycle and
receipt work which its physical operation does not require. The common model
is unified below these surfaces, not by forcing every family through one
surface type.

Payload shape is selected independently of surface shape:

```text
caller retains storage identity   -> operation holds a shared or exclusive loan
caller relinquishes storage       -> operation or Sink owns the moved payload
target selects storage            -> Source owns a pool/quota permit and returns
                                     the selected initialized buffer
unbounded result                  -> Source yields bounded owned chunks
```

These modes state real future use of storage, not scheduling preference. A
shared source loan allows one immutable buffer to feed several zero-copy
operations; an owned transfer lets the producer disappear; a target-selected
pool avoids guessing receive size.

Normal-call dataflow is not sufficient when pending identity itself affects
program control. Wait-any, race, deadline, dynamic collection, and individual
cancellation require a structured completion owner or a family-specific
ticket. For example, a choice scope owns both an I/O bundle and a timer bundle,
selects one winner exactly once, and retains every loser until its own
cancel/drain protocol reaches terminal. A function over two already completed
values cannot implement wait-any, because evaluating its arguments would have
waited for both.

A deadline winning likewise does not prove payload release. An uncancellable
family must either wait for target quiescence before returning its timeout
outcome or return a family token which continues to own the active bundle. An
uncompleted result also cannot enter an ordinary `container<T>` as though `T`
were initialized; the container must own family tickets or be a specialized
operation set/collector. Across a function boundary, every pending borrow and
milestone obligation travels through the hidden completion ABI or through a
real Source/Sink, guard, scope, or ticket owner.

### Finite capacity, backpressure, and unknown-size results

Status: SETTLED IN DISCUSSION on 2026-08-26.

Every operation record, target queue, completion queue, buffer pool, device
tag set, listener backlog, and ready-event queue is finite. The common
operation lifecycle therefore includes admission before target ownership:

```text
READY
    the closed bundle exists; target has no access

WAIT_CAPACITY
    a slot, credit, or buffer is unavailable; runtime still owns the bundle

SUBMITTING
    one lane is attempting the target handoff

IN_FLIGHT
    target may access the declared payload and metadata

QUIESCING
    stop/cancel/close was requested; target access may continue

TERMINAL
    no later publication or target access is possible

CONSUMED
    outcomes and authority have been transferred or released; slot may reuse
```

A host request may finish after partial progress while the semantic operation
continues. Once the target has released that request's descriptor and declared
the exact progress cursor, the bundle returns from `IN_FLIGHT` to `READY` or
`WAIT_CAPACITY` and later submits only the proven remainder. This edge is what
lets an adapter absorb short host operations without duplicating an external
prefix.

Temporary target or runtime fullness is normally `WAIT_CAPACITY`, not a
writer-visible I/O failure. The scheduler runs other eligible work and retries
admission when capacity returns. This prevents a writer retry loop from
becoming a second scheduling interface. A family exposes a capacity outcome
only when bounded credit, drop, timeout, or non-waiting admission is itself
part of that resource's semantics.

An unbounded hidden queue is not an alternative. Finite memory, an
unthrottled producer, and lossless retention cannot all hold forever. A
persistent Source must declare one honest behavior when its capacity is full:
producer backpressure, a finite host backlog followed by typed overflow,
coalescing, dropping with a gap/count outcome, latest-value sampling, or
terminal failure. [Apple Dispatch
Sources](https://developer.apple.com/library/archive/documentation/General/Conceptual/ConcurrencyProgrammingGuide/GCDWorkQueues/GCDWorkQueues.html),
for example, explicitly coalesce pending events rather than retaining an
unbounded event list.

Unknown-size results have four honest storage shapes:

```text
bounded caller destination  -> typed truncation or required-size outcome
owned segmented storage     -> grow without moving target-accessible segments
target-selected pool slot   -> completion transfers one initialized owner
bounded chunk sequence      -> Source yields chunks until EOF or failure
```

A preliminary size query followed by allocation is valid only when the family
proves that size remains stable between the two operations. Files,
directories, datagrams, and live devices do not supply that proof in general.

For destination storage of capacity `n`, a delivery of `k` bytes proves newly
delivered contents only for `[0, k)`. An initially uninitialized destination
keeps its tail unreadable; a previously initialized caller buffer retains its
prior tail value after the target releases the loan. For a pool slot, result
publication does not by itself permit recycling: the target must have released
the slot and the last result owner must have finished with it. Each reused slot
carries a generation or equivalent proof so a late completion cannot write a
later logical object.

Queue depth, buffer size classes, batch size, and watermarks are target/runtime
policy within an owned memory budget. A hosted target may grow nonmoving slabs;
an embedded target may use statically sized frames, descriptors, and pools.
The writer states a semantic maximum only when message size, memory quota, or
loss policy is part of the program's requirement.

A persistent source failure is a typed lifecycle transition rather than a
trap or an excuse to release loans early:

```text
ACTIVE_SOURCE -> FAILING -> FAILED or REPLACED
```

The family declares what happens to already attributed events, produced but
unconsumed shots, pending reservations, the source root, and every pool token.
It may deliver attributed outcomes before a terminal failure, fail remaining
reservations, poison the root, or return a replacement capability. In every
case target access ends before the corresponding payload or pool authority is
returned.

### Completion milestones and typed partial outcomes

Status: SETTLED IN DISCUSSION on 2026-08-26.

A **milestone** is one exact fact which a resource contract publishes. The
common vocabulary distinguishes at least:

```text
accepted             target has progress responsibility and effects may begin
result_ready         a specified typed result is initialized
payload_released     target no longer accesses the named payload range
authority_released   a conflicting resource fragment may be granted again
terminal             no later publication or target access can occur
visible(domain)      the effect is visible in the named observer domain
durable(model)       the effect survives the named failure model
acknowledged(peer)   the named peer/protocol layer has confirmed it
```

These are not synonyms. [Linux zero-copy
send](https://man7.org/linux/man-pages/man2/io_uring_enter.2.html) can publish
the send result before a later notification permits buffer reuse; that
notification does not prove remote receipt. A normal file write completion
does not prove durability; a separate
[sync/commit](https://pubs.opengroup.org/onlinepubs/009695399/functions/fsync.html)
family establishes that stronger fact.

The normal surface for a finite one-shot operation completes at its declared
**ownership-complete milestone**: its writer result is ready, every borrowed
payload which the caller regains is released, and the required operation
authority is returned. This keeps normal call lifetime semantics simple. If a
family must expose an earlier result while retaining payload or authority, it
returns or associates a family-specific affine milestone guard/receipt. That
guard becomes visible only because the split milestone is program semantics.

Partial progress cannot collapse to a bare error. If a one-megabyte write has
made a 128-KiB prefix externally visible before `NoSpace`, the terminal outcome
must identify at least:

```text
the completed prefix and milestones it reached
the still-owned remainder
the resource's next state
the retry cursor, if any retry is valid
the typed error
```

The same rule protects input: a read error may coexist with an initialized
prefix, and a directory error may coexist with entries already consumed from
the source. Some family outcomes therefore contain both a value and an error.

An adapter may retry only after proof that the earlier attempt was not
accepted, that the family is idempotent under a stable operation identity, or
that retry continues from an exact progress cursor without repeating the
prefix. A zero-progress nonterminal attempt returns to target wait or
backpressure; it never busy-loops.

One-shot and multishot cardinality share the lifecycle but not result storage:

```text
one-shot
    zero or more internal progress events -> one terminal outcome

multishot
    one persistent subscription bundle
        -> many independently owned shot bundles
        -> one subscription terminal outcome
```

Each shot owns its payload/pool token. A subscription result cell is never
overwritten for the next shot. The family declares maximum unconsumed shots,
pool exhaustion, attribution, coalescing/drop behavior, source failure, and
the fate of already attributed shots during finish or cancellation.

### Cancellation, abandonment, finish, release, and close

Status: SETTLED IN DISCUSSION on 2026-08-26.

A cancel request moves an active operation toward `QUIESCING`; it is not a
terminal witness. Normal completion may win the race, cancellation may win,
or an uncancellable operation may continue. Exactly one terminal transition
publishes a typed outcome such as completed, cancelled, partial-then-cancelled,
late error, or too late. Payload and metadata remain owned until target
quiescence, whatever the cancel-request API returned.

An unused result does not make an effectful operation removable or
cancellable. A write still carries its external commitment, a receive may
have consumed input, and an accept may own a fresh connection. The runtime
drains the operation to its required terminal state and then releases an
unobserved result, unless a separately proved elimination rule applies.

An active bundle can never be dropped. Its ownership transfers to one of:

```text
the enclosing structured operation scope
a resource-local pending set which continues to drain
a family cancel-and-drain obligation
a finish obligation
whole-process teardown after the impossible false-claim path
```

Every normal, typed-error, `return`, branch, and loop exit from a latent
operation scope carries one of the family-declared drain, cancel-and-drain,
transfer-to-a-named-owner, finish, or abort/discard dispositions. Transfer is
not writer-visible fire-and-forget: the receiving resource-local or enclosing
owner retains the complete bundle through its required terminal. The compiler
does not infer a cancel merely because a sibling failed or a result became
dead, and it never submits across an unresolved control dependency in order to
create overlap.

`finish` is a world state transition, not a generic wait spelling. It can mean
graceful EOF, flush, durable commit, source shutdown with final statistics,
device-queue drain, or transaction publication. A family is either:

```text
release-complete
    compiler-derived release can safely complete its declared lifecycle

finish-required
    normal success must consume the owner through finish, return the unfinished
    owner, or take an explicitly different abort/discard transition
```

Compiler-derived release cannot silently upgrade itself into successful
finish, because the writer has not authorized that stronger world transition
and cannot handle its typed failure. Close is a whole-lifetime-group consuming
transition: it stops new reservations, applies the family drain/cancel policy,
waits for every conflicting authority-release, target-access, and
registration-retirement milestone required by that family, and only then
reclaims root storage. An independent durability or acknowledgement witness
may remain live after it no longer owns the root. Close never rolls back
effects already performed.

The trap path remains outside these rules. Correct execution never reaches it,
and no normal operation, Source/Sink, scope, slot, or target path pays for it.

### Every resource family closes the same contract

Status: SETTLED IN DISCUSSION on 2026-08-26.

The common API is complete only when each family declares all of the following
or explicitly marks a category inapplicable:

1. root lifecycle, parameter-to-authority projection, fragment coexistence,
   and how every fragment is returned or retired;
2. capacity owner and bound, admission order, target-acceptance linearization,
   transient-full behavior, and proof that a retry cannot duplicate effects;
3. every payload region the target may access, its borrow/move/pool mode,
   stable-address or registration needs, and its exact release milestones;
4. result-size class, storage owner, initialization proof, truncation or chunk
   behavior, and buffer recycling;
5. progress unit, partial external effect, retry cursor, zero-progress rule,
   and any independently releasable prefix;
6. one-shot or multishot cardinality, shot ownership, maximum unconsumed
   shots, and subscription terminal condition;
7. ordered, unordered, keyed, broadcast, or coalescing attribution, including
   cancellation and source-failure races;
8. the milestone relation among acceptance, result, payload release,
   authority release, terminal, visibility, durability, and acknowledgement;
9. every normal, partial, late-error, cancelled, and source-failure outcome,
   including exact resource and payload next-state;
10. cancel support and quiescence proof; backpressure, loss, or overflow;
    abandon, finish, implicit release, close, and root-reclamation behavior;
11. target qualification evidence that each native path preserves the same
    authority, attribution, milestone, memory-access, and progress contract.

These declarations belong to trusted or machine-verified system-family data.
The writer cannot weaken them, fill a missing category with a claim, or select
a less safe backend behavior.

### Family surface selection

Status: SETTLED IN DISCUSSION on 2026-08-26.

| Family or operation | Selected surface | Load-bearing reason |
|---|---|---|
| Positioned file read/write | finite one-shot normal call surface | independent offsets and finite outcomes should not pay persistent-queue cost |
| Sequential file view | owned Source/Sink | cursor, read-ahead, batching, EOF, and finalization persist across chunks |
| Stream receive direction | owned Source; fixed-buffer finite read may be one-shot | unknown byte arrival, ordered attribution, pool/backpressure, EOF |
| Stream send direction | owned Sink | ordered payload queue, batching, backpressure, half-close, late errors |
| Finite datagram send | finite one-shot normal call surface | one bounded message and one typed send outcome |
| Repeated datagram receive or persistent batched send | message Source/Sink role | message boundaries, target-selected buffers, truncation/drop policy |
| Listener | Source of owned connections | persistent accept capacity and multishot/preposted target operations |
| One-shot timer | finite one-shot; a family may declare broadcast observation | one bounded expiry/cancel outcome rather than a persistent sequence |
| Periodic/coalescing event | Source | repeated outcomes and family delivery algebra |
| Directory iteration | Source of owned batches | advancing cursor, unknown record sizes, partial batch plus error |
| Device | one-shot, Source/Sink, or family Command | target queue/tag/DMA algebra and semantic command identity vary by family |
| Multi-resource finite operation | composite one-shot normal call surface | all authority fragments are acquired before any target handoff |
| Real collective transaction | family Transaction/Scope | atomic commit, collective abort, or error aggregation is program semantics |

No family selects a writer callback. A UI or event loop consumes a Source and
the Whitefoot scheduler executes the resulting writer code.

### Compiler, runtime, and code generation required by the API

Status: SETTLED IN DISCUSSION on 2026-08-26.

The checked compiler graph carries:

```text
value and argument dependencies
control dependencies
authority-fragment relations
payload loans and capacity reservations
event attribution
result, payload-release, authority-release, and terminal milestones
scope, finish, close, and release obligations
```

It may submit a node only after its semantic call has been reached and all
submission inputs and authority are available. It may progress another node
before the first completes only when this graph gives the necessary free or
ordered coexistence permission.

Every concrete function receives a compiler-derived internal classification:

```text
NeverSuspends
MaySuspend
```

This is target/lowering metadata, not a writer effect or claim. A
`NeverSuspends` direct function retains the normal machine ABI, receives no
frame pointer or scheduler check, returns no pending tag, and initializes no
completion backend. A direct call chain uses a hidden suspend-capable ABI only
where a world operation can really suspend. An indirect call whose target set
is unknown pays for a suspend-capable thunk at that call site; the cost does
not spread to known pure calls.

The default representation is selective stackless lowering. A compute segment
runs on the normal machine stack. At a real suspension, the compiler stores
only values, loans, operation slots, child counts, and the program position
which remain live across that suspension. A resume entry can therefore run on
any lane permitted by the scheduler rather than remaining attached to one
native stack. This choice supersedes the earlier provisional fixed-calling-lane
assumption.

Operation metadata must have stable storage before the first target handoff,
because a completion may race the return from submission. A target adapter
distinguishes exactly:

```text
TARGET_OWNS
INLINE_TERMINAL_WITH_NO_FUTURE_EVENT
REJECTED_BEFORE_OWNERSHIP
```

The selected target adapter owns exactly one terminalization path. Either the
submission classification proves a true inline terminal with no future event,
or a later asynchronous publisher performs the terminal transition, never
both. On a platform where an apparently inline success still produces a later
completion packet, the submitter does not also publish or release. A callback,
CQE harvester, helper, or interrupt writes results, changes state, and wakes
the scheduler; it never invokes writer continuations.

Storage selection proceeds from least to most dynamic:

```text
statically bounded operation slots embedded in the parent frame
compiler-generated window/arena with slot-lifetime reuse
persistent Source/Sink control and buffer pools
nonmoving resource/lane slabs for dynamic bounded fan-out
backpressure before any unbounded growth
```

Inline completion can discharge the logical bundle locally and permit scalar
replacement of the result cell and operation record. When no independent work
exists and measurement shows a direct host call wins, a target may collapse
submission plus immediate wait into that direct call. This is a degenerate
lowering of the same completion contract, not a second writer-visible blocking
API.

The machine shape is known to be implementable without adopting another
language's surface: [LLVM coroutine
lowering](https://llvm.org/docs/Coroutines.html) splits a function into
initial, resume, and destroy paths and can place a nonescaping frame in caller
storage; [embedded executors](https://github.com/embassy-rs/embassy)
demonstrate statically allocated state machines awakened by interrupts on one
stack. Whitefoot must nevertheless qualify its own emitted shape and may
replace the mechanism if measurements find a faster sound one.

### Evidence which can still falsify the selected shape

Status: OPEN EXPERIMENT. The owner settled the derived surface and lowering
direction, not unmeasured performance claims.

The semantic surface is settled, but its P0 claims remain experimentally
falsifiable. The cheapest required probes are:

1. Pure-compute machine code and startup resources are identical with and
   without unused completion capability.
2. An inline-heavy one-shot path adds no heap allocation, completion enqueue,
   or wake compared with the best direct target call.
3. Two non-inlined, separately compiled compatible wrappers overlap without
   writer-visible pending values or an LTO-only trick.
4. Runtime-selected queue depth approaches the best measured hand-tuned
   pending queue and batch window across payload sizes and loads.
5. Source/Sink reaches raw multishot, preposted-accept, buffer-pool, and target
   pump ceilings without an extra payload copy or one writer call per physical
   event.
6. Cancellation/attribution races never duplicate an event, return a loan
   early, or accumulate unbounded abandoned state.
7. Opposite-order multi-resource admission cannot deadlock or half-submit and
   does not require a universal global lock.

Failure reopens the relevant surface or lowering premise from the
constitution. It is not dismissed because the abstract model admits more
schedules.

## Capability values and ordinary regions are the language-level identity

Status: SETTLED IN DISCUSSION on 2026-08-25.

The rejected candidate used a second type-level identity:

```wf
&uniq 'loan Output<'world>
```

`Output<'world>` would be new syntax and was introduced before necessity was
proved. It is not part of the selected model. A lifetime or Whitefoot region
states how long a borrow is live; it is not an object identifier. Two factory
results may live in the same lexical region, and one value may receive several
successive borrow lifetimes.

The settled identity rule uses the existing ownership and effect vocabulary.
The spelling below illustrates possession versus use only; it does not select
the final Output/Sink declaration or call shape:

```wf
fn emit['out, 's](
  output: &uniq 'out Output,
  source: &'s buffer<u8>,
) -> result: own unit reads('s), writes('out);
```

In this one-at-a-time schematic fallback, `&uniq 'out Output` states
possession: the logical call holds the only usable root path through its
ownership-complete milestone. A returned milestone guard would instead carry
any authority which survives an earlier result. `writes('out)` states use: the
function actually performs an observable state transition through the logical
resource. These facts remain independent. Holding a key does not prove that
the function used it, and using it does not make the key own the persistent
external result.

At a call boundary the formal row is projected to the actual resolved place.
Two values `left` and `right` therefore produce `writes(place(left))` and
`writes(place(right))`, even when both borrows use the same caller lexical
region. At the unrefined root level, two uses of one value conservatively
conflict. A checked family authority rule may refine them into freely
coexisting or ordered fragments, as an actual Send Sink does when it mints
several ordered reservations. Distinct lifetime spellings are neither
necessary nor sufficient for that refinement.

A capability value locates a language-level identity/lifetime group and its
whole-resource control; family authority and attribution contracts determine
the actual interaction and ordering fragments. It is not a claim that the host
backing is globally unique. The external effect may outlive the capability
value; the checked `reads`/`writes` fact records the operation, not the
lifetime of the physical object.

The API section above now fixes borrow, move, pool, finish, and release
semantics at the common level. Exact declarations, container projection, and
generated-capability lineage remain specification work and may require
internal metadata, but they do not restore a second writer-visible lifetime or
`Output<'world>`.

## Environment aliasing is outside the language alias proof

Status: SETTLED IN DISCUSSION on 2026-08-25.

Two successful `open` operations produce two logical capability instances.
Hard links, symbolic links, mount or rename behavior, redirection of stdout and
stderr to one sink, another process, a common remote endpoint, and other
environment choices may cause two instances to reach the same physical state.
Whitefoot does not recover or track that physical identity.

Consequently, two distinct file capabilities may be used concurrently even if
the environment maps them to one inode. Their file bytes may reflect any
interleaving the qualified operation and host permit. This is an external
logical race, like another process modifying the file; it is not a Whitefoot
memory data race and may never become native undefined behavior, memory
corruption, an invalid buffer access, or a broken loan. Target qualification
must serialize or reject a native facility whose concurrent use cannot meet
that safety boundary.

The exact `reads`/`writes` footprint therefore names logical capabilities and
coordination domains, not final host objects. Different capability roots have
no implicit cross-root external order. A program that needs such order supplies
a value, control, loan, or coordinator dependency as the completion-dependency
rule requires.

This boundary is what makes the ordinary ownership model both general and
fast: it does not impose a filesystem identity registry or a global I/O order
on every program merely to stabilize an environment-created alias.

## Relationships created by the runtime must close under ownership

Status: SETTLED IN DISCUSSION on 2026-08-25.

Environment-created aliasing is outside the proof, but the system API must be
honest about mutable protocol state which it creates and whose contract the
program relies on. It may not manufacture one non-commuting runtime state and
present it as two unrelated owners.

Every system-created relationship remains inside one resource lifetime group.
The family can project long-lived role or range facets, mint short-lived
operation reservations, move/consume whole authority, or retain the minimum
resource-local coordination state required by noncommuting operations. These
are durations and refinements of one authority model, not a closed list of
unrelated API tricks.

A listener Source retains common accept authority and owns anonymous physical
accept capacity; each logical accept reservation receives one fresh connection
owner under the family's attribution rule. A stream may expose one long-lived
send facet and one receive facet. It does not mint two unordered independent
send owners, but the one send facet may create many ordered short-lived send
reservations. A native `dup` which shares a cursor, queue, reservation, or
other noncommuting state retains the same lineage/lifetime group rather than
pretending to create independent roots.

Each in-flight operation owns its closed bundle through all declared
milestones and the unique final terminal witness. Result publication,
payload release, authority release, cancellation, and terminal may be separate
transitions; no one-bit completion shorthand may release the whole bundle
early.

No ambient writer authority is allowed. Clock samples, randomness, input
consumption, receive, accept, resource factories, and every other ordered world
interaction enter through an explicit capability or compiler-owned operation
contract. This is the completeness condition which lets ownership and exact
`reads`/`writes` replace a coarse global marker.

## `external` and `blocks` are deleted as language effects

Status: SETTLED IN DISCUSSION on 2026-08-25.

`external` records a non-empty world interaction as one payload-free bit. It
does not say which logical capability was used, whether the operation read or
wrote it, or which other operation conflicts. Keeping it would either serialize
all I/O or duplicate facts already present in capability-specific
`reads`/`writes`.

The valid jobs once attributed to `external` have precise owners:

- exact `reads`/`writes` distinguish possession from actual use and propagate
  the fact through user calls;
- an empty complete effect row defines purity;
- compiler-derived release contributes the resource contract's own footprint;
- trap and teardown rules speak directly about submitted operations and world
  writes;
- an unknown trusted boundary must expose a conservative capability footprint
  or be rejected; and
- cross-capability ordering is a real value, control, loan, or coordinator
  dependency, not a global source-order promise.

An implementation may cache a derived `touches_world` bit, but that cache has
no language authority and the writer never declares it.

`blocks` describes whether one target implementation occupies a host thread.
That property changes when the same semantic operation is served by a hardware
queue, an interrupt, a helper thread, or another backend. It is therefore not a
portable language effect. The operation's completion and progress contract
states its terminal outcomes, possible non-completion, cancellation rules, and
loan-return point. Target metadata states whether an adapter consumes a helper
thread or must avoid a required compute lane. User-function summaries are
derived from calls and releases rather than written as `blocks`.

The final language removes both atoms from writer effect rows and does not keep
their bare spellings reserved merely for their retired meanings. Compiler
internals may retain derived summaries for performance, diagnostics, or
lowering, but those summaries are not independent semantic facts.

The deletion itself does not select those later layers. The preceding API
section and the runtime sections below now select their common semantic shape
and lowering direction. Exact factory declarations, target ABI bytes, and
measured emitted layouts remain specification and implementation work rather
than reasons to restore either effect atom.

## The completion backend does not require one dedicated I/O thread

Status: SETTLED IN DISCUSSION on 2026-08-25.

The hosted target set for this investigation is exactly macOS, Linux, and
Windows. Other hosted systems and embedded targets do not shape this runtime
decision.

Whenever a program contains world operations, completion is their sole backend
model. A pure program may dead-strip or lazily avoid initializing every
completion-runtime facility, preserving the selected pure-compute zero-tax
boundary. For a program which uses the world, this statement still does not
require one dedicated I/O thread. It requires only that operations can be
submitted and that some target facility can publish their milestones. An
existing compute lane may reap or wait for completions whenever it has no ready
compute work.

On Linux, ordinary `io_uring` exposes shared submission and completion queues;
Whitefoot needs no dedicated userspace I/O thread. Submission-queue polling is
an optional mode which creates a kernel polling thread, and the kernel may use
internal asynchronous workers for operations which need them. Those are Linux
backend choices, not common-runtime requirements. See
[`io_uring_setup(2)`](https://man7.org/linux/man-pages/man2/io_uring_setup.2.html).

On Windows, an I/O completion port queues completion packets. The main thread
or any existing thread may call `GetQueuedCompletionStatus` or its batched
variant, so a Whitefoot compute lane may service the port when idle. At least
one thread must eventually dequeue packets, but it need not be a dedicated I/O
thread. See
[I/O Completion Ports](https://learn.microsoft.com/en-us/windows/win32/fileio/i-o-completion-ports)
and
[`GetQueuedCompletionStatus`](https://learn.microsoft.com/en-us/windows/win32/api/ioapiset/nf-ioapiset-getqueuedcompletionstatus).

On macOS, Dispatch I/O schedules asynchronous file operations and delivers
handlers on dispatch queues. A Whitefoot-specific dedicated I/O thread is not
required, but the backend may use libdispatch-managed execution resources and
may incur a thread hop. Event-oriented facilities may instead be waited on by
an idle compute lane. Which macOS arrangement is fastest remains an experiment,
not a common architectural rule. See
[Dispatch I/O](https://developer.apple.com/documentation/dispatch/dispatch-i-o)
and
[`dispatch_io_read`](https://developer.apple.com/documentation/dispatch/dispatch_io_read).

Compute lowering is independent of this question. The number of lanes which
execute Whitefoot code neither enables nor disables the completion backend,
and the choice of a target-managed helper or polling thread does not select a
single-threaded or multi-threaded compute lowering.

## One scheduler consumes compute work and world completions

Status: SETTLED IN DISCUSSION on 2026-08-25.

Whitefoot has one scheduler with two specialized progress sources. Runnable
compute work remains in work-stealing deques: its owning lane may run it and
another lane may steal it. A pending world operation has no writer code which
a lane can execute; only the target can publish its completion. It therefore
uses a completion queue or mailbox rather than a compute deque.

Completion publication performs only target-to-runtime transitions: initialize
the value carried by a declared milestone, release-publish that milestone or
the final terminal state, and make every newly satisfied frame runnable. The
scheduler then wakes only the parked lanes needed for that new runnable set. A
target callback, waiter, libdispatch worker, kernel worker, or other adapter
thread never runs a writer continuation, steals compute work, or decides which
calls may overlap. Such a thread remains part of the target adapter and does
not form a second Whitefoot executor.

The scheduler repeatedly observes both sources. It first makes arrived
milestones visible to their owners, immediately continues when a result or
authority requirement it needs is satisfied, otherwise runs local compute work
or steals ready compute work. Only when neither source can make progress does
it park at one target-specific wait point which can be awakened by either new
compute work or a world completion.

Two sleep rules are mandatory. After processing any completion, wake hint, or
compute frame, the lane returns to the top and rechecks all sources; it never
parks immediately after progress. Before sleeping, the lane first announces
its intent to sleep and then rechecks both compute and completion sources. A
publisher which arrives before the recheck is observed by the lane; a publisher
which arrives after the announcement is responsible for the wake. These rules
prevent a lane from sleeping beside already completed work or losing a wake in
the gap between an empty check and the host wait.

The common architecture does not require compute work and completions to share
one queue, one producer protocol, or one target facility. It requires one
scheduling authority and one sleeping decision, so a completion does not cross
an I/O executor, a second ready queue, and then the compute scheduler before
writer code can continue.

## World means target ownership, not a thread

Status: SETTLED IN DISCUSSION on 2026-08-26.

`World` names the period in which an operation has left the Whitefoot
scheduler and the selected target owns progress under the family contract. It
publishes each milestone the contract guarantees; if and when the family
reaches its terminal condition, it publishes the unique terminal witness.
Possible non-completion and persistent Sources therefore do not become false
liveness promises. World is neither a Whitefoot lane nor necessarily one
kernel thread. The target may represent this responsibility with kernel
operation state, a protocol stack, a device or DMA queue, a kernel worker, an
optional polling thread, or libdispatch-managed work on macOS.

The world does not inspect a compute deque and steal an I/O task. The lane
which creates the operation executes the short, nonblocking target submission
path itself. Once the target accepts responsibility, ownership changes as
follows:

```text
READY / WAIT_CAPACITY: runtime owns the complete operation bundle
    -> SUBMITTING: one lane is publishing the target descriptor
    -> IN_FLIGHT: target owns progress and may publish declared milestones
    -> QUIESCING: stop was requested but target access may continue
    -> TERMINAL: target access and later publication have ended
    -> CONSUMED: outcomes and returned authority have their final owners
```

Submission may complete inline or be rejected with a terminal error. If a
target queue is temporarily full, the operation remains runtime-owned and
ready to submit; any lane may perform that short runtime action. It does not
become world-owned until target acceptance. Bounded batching may combine
submissions, but the maximum delay before a target kick is runtime policy and
must be measured.

On ordinary Linux `io_uring`, the submitting lane publishes the SQE and kicks
the ring; SQPOLL optionally replaces that kick with a kernel polling thread.
On Windows the submitting lane issues the overlapped operation directly. On
macOS the lane hands the operation to Dispatch I/O or another selected adapter;
libdispatch-managed work may occur before the kernel sees the actual file
operation. A centralized Whitefoot submitter would add a queue handoff and is
therefore not the common default.

## World operations use the same internal join shape as parallel compute

Status: SETTLED IN DISCUSSION on 2026-08-26. This is internal runtime
structure beneath the selected API surfaces, not a writer-visible join form.

Both compute and world work have a runtime handle whose result or ownership
milestone is eventually needed. Their internal joins differ in one fallback:

```text
compute join:
    if another lane ran the frame, take its result
    otherwise the joining lane may run the frame itself

world join:
    if the required milestone is published, take the result or authority
    otherwise run other eligible compute work or wait for target completion
    the joining lane can never execute the world operation itself
```

With one Whitefoot lane, a frame submits the operation and continues until it
first needs a declared result, payload release, authority release, or scope
terminal. If that milestone is already published, the frame continues without
suspending. Otherwise it registers the exact dependency, rechecks to close the
publication race, saves only its live state, and returns control to the
scheduler. The lane then runs another eligible frame; only the scheduler, when
no frame can progress, enters the target wait.

With two or more lanes, the same suspended-frame protocol applies. Submission
does not give one lane ownership of the later join. Publication records the
declared milestone and makes every newly satisfied frame runnable; it performs
no wake when no registered dependency became ready. A nonterminal result
milestone does not falsely retire the operation or release a payload whose
later milestone is still outstanding.

The selected compiler representation is a stackless frame containing only
state live across suspension. The frame may therefore resume on any eligible
lane which atomically claims it; the originating lane remains an affinity hint,
not an ownership requirement. A target callback still never executes the
continuation directly. A target or fallback which retains a native stack may
pin that particular continuation to its lane, but that is a measured target
degradation rather than the common semantic rule.

The operation state is conceptually a product rather than one completion bit:

```text
lifecycle phase
+ published milestone facts
+ per-payload release facts
+ live multishot deliveries
+ unique final terminal witness
+ registered frame requirements
```

The race-free conceptual protocol is:

```text
depend_on_world(frame, operation, requirement):
    if acquire(operation.milestones) satisfies requirement:
        continue frame

    save frame state and mark SUSPENDING
    register (frame, requirement)

    if acquire(operation.milestones) satisfies requirement
       or frame was notified while SUSPENDING:
        unregister dependency
        mark frame RUNNING
        continue frame

    atomically commit frame to SUSPENDED
    return control to scheduler
```

```text
publish_milestone(operation, milestone, value):
    write value if this milestone carries one
    release_publish(operation.milestones, milestone)

    for each registered requirement newly satisfied:
        if its frame is SUSPENDING:
            mark that frame notified
        else if atomically changing SUSPENDED to READY succeeds:
            enqueue that frame once

    wake the number of parked lanes scheduler policy now requires
```

Immediate milestone publication does not mean unconditional wake. A
continuation which has not registered a requirement needs no wake; it will
observe the published state later. A one-owner one-shot fast path may store one
dependency inline; broadcast, multishot, and shared milestones use a
family-owned dependency set. Runnable publication precedes any wake, and the
scheduler avoids both missed wakes and broadcast thundering herds. Nor does a
wake guarantee immediate writer execution: a lane already running
nonpreemptive compute reaches another frame only at its next scheduler
boundary.

A dedicated target reaper may reduce the delay between a host event and its
declared milestone publication, drain queue pressure, and wake parked lanes as
scheduler policy requires. It cannot execute writer code or eliminate the
delay caused by all compute lanes running long nonpreemptive work. Fixed or
adaptive reapers, Linux SQPOLL, macOS worker arrangements, batching limits,
and completion-to-resume latency therefore remain target experiments rather
than common language or scheduler rules.

## Open experiment: completion overhead against direct blocking host calls

Status: OPEN EXPERIMENT. No performance dominance result has been established.

The earlier paper argument that completion admits every blocking schedule does
not prove equal or better realized performance. A completion runtime may pay
three additional groups of cost: request encoding and publication; suspension
state and waiting bookkeeping; and completion publication, result transfer,
wakeup, and resumption. A direct blocking call may remain on one stack and
return through one host call. The difference can be material at concurrency
one even when completion provides more schedules at higher concurrency.

This experiment isolates the finite one-shot direct-host floor. The broader
Source/Sink, pure-compute, cross-function, cancellation, and multi-resource
falsifiers are listed in the API section's open-evidence subsection and remain
separate measurements.

Build semantically matched blocking and completion implementations on all
three targets:

- Linux: direct `read`/`write` against `io_uring`;
- Windows: synchronous I/O against overlapped I/O plus an I/O completion port;
  and
- macOS: direct `read`/`write` against Dispatch I/O and any competing event
  arrangement which the target supports.

Sweep concurrency depth, payload size, isolated I/O versus mixed compute and
I/O, single operations, bursts, and sustained saturation. Measure median and
tail latency, throughput, CPU cycles, system calls, context switches, thread
hops, cache misses, bytes of live operation state, and the effect of batched
submission and completion. Test allocation-free slots, inline completion, and
same-thread resumption rather than assuming them free.

The result selects target implementation strategy and identifies the depth at
which completion repays its machinery. It does not introduce a second
writer-visible blocking API. If completion cannot recover the relevant
performance on one target, the investigation must locate the irreducible cost
and revisit the common model or target implementation from the constitution;
it may not dismiss the measurement because completion has a larger abstract
schedule set.
