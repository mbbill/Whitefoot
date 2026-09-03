# Containers and resources: the integrated design

The single design for batch 0116. It merges the two drafts beside it,
`RESOURCES.md` (providers, the envelope `E`, the `resource-closed` judgment) and
`CONTAINERS.md` (owners, views, and the facts that cross a call), into one set of
laws, one set of rules, one vocabulary, and one amendment register. A reader who
has not read either draft can read this file alone. The drafts remain for their
detailed rationale, their rejected alternatives, and their migrations; every rule
they stated normatively lives here.

**Fourth draft, after falsifier round 3.** Round 3 found one cause under three of
its four reports. A value's relationship to its backing store was carried by
something other than its type: by a provider's *place*, by an origin *set* the
preservation rules did not traverse, by a per-activation *extent* whose region
block could be entered twice. Store identity therefore fell to a move plus a
reinitializing `set`, to a runtime offset, and to every field, element and payload
position the same draft had just opened. Disposal was a leaf-only operation on a
class closed under containment, so a container of leases was undestroyable and a
helper could release nothing. And an arena's capacity was an ordinary killable
fact, so no arena loop had a bound.

This draft makes one change at the root. **A store's identity is a region, and the
region is in the type.** A region names at most one store, a reserving occurrence
mints its store at the region it names, and every value that store backs carries
that region in its own type: `Pool<'s, T, N>`, `PoolSlot<'s, T>`,
`PoolVector<'s, T, N>`. Store identity is then preserved by type formation itself,
through construct, field projection, container elements, enum payloads,
multi-return, joins and calls, because no value-forming step changes a value's
type. Disposal becomes structural, closed under containment exactly as linearity
is, and checkable inside a helper by type equality. Provenance keeps its [OWN-5]
shape for the one kind that needs it, the views, which carry a loan and not a
store. The rule count falls from fifty-three to fifty-two, and one preservation
closure replaces four preservation sentences and a deferred per-leaf-provenance
obligation, one structural disposal replaces a four-operation list and a per-owner
release table, and one type parameter replaces a confinement region beside a store
identity.

Tree read: `batch/0116-containers-and-resources` at `main` a40c7e70,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file; every other citation names its file.

**Nothing here is implemented.** No compiler code was written for it. Section 3
is draft rule text for a work branch, not an amendment. Every program in
section 4 is design text and compiles nowhere. Section 6 separates what a
compiler executed in this session from what is argued on paper.

Settled by the owner, and not reopened anywhere below:

- The heap is an explicit capability **value** handed to `main`, so heap-freedom
  is a signature fact.
- `resource-closed` is a derived, writer-requirable property over an envelope `E`
  of tangible resources; a general heap, including a bounded general heap, is
  never part of `E`.
- No frame-accumulating recursion in v1; tail recursion is lowered.
- `FixedVector<T, N>` holds affine `T` through an initialized-prefix typestate.
- The core is a contiguous initialized-prefix sequence; keyed containers are
  fixed families over it, later.
- Owners (`HeapVector`, `FixedVector<T, N>`, `ArenaVector`, `PoolVector`) versus
  affine views (`Span`, `MutSpan`, `AppendView`), transformed by value, with
  single-state `ensures` under [FN-9]. Two-state `ensures` is rejected.
- Append helpers take the owner by value and return it. Pass-by-pointer is only
  an ABI.
- Three call rules: through a shared borrow all facts survive; through a value
  passed and returned only contract facts survive; an element write through a
  length-fixed view never touches length.
- Mutation of container state through `&uniq` is retired.
- Multi-return `-> (a: own T, b: own U)` with `let (a, b) = f(...)`.
- System I/O goes over views.
- Every rule is a deterministic function of program text and compiler version,
  never of time or of a work budget.

Three footnotes, because three rounds have read that list against the rules.
[CNT-1] declares a fifth owner, `FixedRing<T, N>`, and the settled list names the
four prefix owners and excludes no rotation. The settled append example writes its
source argument first, while [GRAM-11] fixes argument order from the declaration
and every helper here declares its owner first. And the settled owner names are
unchanged: what this draft adds to `HeapVector<T>` is the store region, giving
`HeapVector<'s, T>`, the same type with its store written down.

## Contents

1. [The problem](#1-the-problem)
2. [The laws](#2-the-laws)
3. [The rules](#3-the-rules)
4. [Two worked programs](#4-two-worked-programs)
5. [Open questions](#5-open-questions)
6. [Verified versus reasoned](#6-verified-versus-reasoned)
7. [Implementation order](#7-implementation-order)

---

## 1. The problem

### 1.1 Two goals, one language

**Goal A: the heap is off, and only logic errors remain.** A writer building an
OS kernel, a bootloader, a flight controller, or a device driver wants a program
that cannot corrupt memory, cannot race, cannot read an uninitialized byte, cannot
silently overflow, and also cannot die because a store ran out. Today the language
delivers the first four and not the fifth: [SCOPE-3] (27-34) leaves heap
exhaustion, stack exhaustion, operating-system quotas and runtime-start resources
outside the source outcome model, so an accepted program may stop at the host
boundary with no Whitefoot value, no status, and no cleanup. A program that can
vanish at three in the morning has not removed the class of failure the writer came
here to remove.

The owner's shape for goal A is a **promise**, not a guarantee about the world.
The compiler computes one finite, shaped envelope `E`; the program promises never
to demand more than `E`; the environment decides whether it can deliver `E`. Only
the conjunction gives freedom from exhaustion, and a program that reaches the heap
makes no such promise, because total free bytes cannot answer a request for a
contiguous aligned extent.

**Goal B: with a heap, be honest.** A hosted program wants the heap and should
have it. What it must not have is a hidden trap. Today it has two. Allocation is
ambient: any function may allocate while holding nothing, and refusal ends the
process. And release is invisible: probe `r2_5` compiles a function that takes
`own box<u64>`, never returns it, and declares `pure`, so a heap free happens
inside a callee whose signature does not mention a heap and whose caller cannot
order anything against it. Goal B asks for both halves to be values: allocation is
an operation on a provider the caller holds, refusal is an ordinary typed outcome
that hands back every affine input it did not consume, and release is a statement
that names the same provider.

Both goals are one language. There is no subset mode, no second prelude, and no
dialect: the same rules judge every program, and one entry marker turns the
failure to establish the property into a compile error instead of a note.

### 1.2 The concrete failure: D1

The sweep of 2026-09-03 found an unsound accept that is exactly the defect this
design has to make unrepresentable. The program is recorded as
`tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`,
manifest line 165, status `xfail`. **Re-run in this session against the gate
binary: accepted, exit 0.**

```wf
fn shrink['a](handle: &uniq 'a buffer<u8>) -> discarded: own buffer<u8>
    reads(handle), writes(handle), allocates(heap) {
  let smaller = buffer_new(2_u64, 0_u8);
  let old = replace deref(handle) = move smaller;
  return move old;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let line = buffer_new(10_u64, 0_u8);
  region 'r {
    let dropped = shrink<'r>(handle: &uniq 'r line);
  }
  let tail = line[9_u64];
  return exit_status(code: 0_u8);
}
```

`buffer_new(10_u64, ...)` establishes `len(line) = 10`. The callee replaces the
whole referent with a two-byte allocation. The caller keeps the stale length and
uses it to discharge offset 9 of what is now a two-byte object: an accepted
out-of-bounds heap read, and (with `set line[9_u64] = 7_u8;` instead) an accepted
out-of-bounds heap write.

The located mechanism is the point. `argument_referent`
(`compiler/src/semantic/places.rs:349-355`) classifies a projected callee write as
an *element* write for every `&uniq buffer<T>` actual, from the **actual's
syntactic shape**; `event_kills_term`
(`compiler/src/semantic/entailment/flow.rs:2927`) then honours [ENT-5]'s true
sentence "an element write never kills a length fact" and keeps the stale length.
The specification is right and the compiler is wrong, but repairing the flag
repairs one sentence. The defect class is that a caller inferred a fact-preserving
property of a callee from information about the **call site**, and nothing in
[EFF-2]'s row distinguishes an element write from a whole-referent replace.

That class is what a container design multiplies. A caller wants to keep across a
call a container's length, its capacity, its initialized prefix, the spare of an
append window, the disjointness of a range handed to a lane; every one of those,
justified by anything other than the callee's declared types and contract, is
another D1 waiting for the next sweep.

D1 has a sibling, and this draft is the first to name it. A caller also wants to
know **which store** a value belongs to, and every answer derived from anything
but the value's own type is defeated the same way. Round 3's rank-one break was
three statements: lease from the pool bound at `a`, move `a` out, rebind `a` to a
second pool, release the lease into the store now standing at that place.

### 1.3 What the design therefore has to do

Turn every resource a program can exhaust into a value it must hold in order to
consume **and in order to release**, so that "this subtree cannot touch the heap"
is a signature fact and "this program's peak demand is this list of extents and
slot counts" is a compiler judgment. Give the writer one declaration that turns
the second into a compilation requirement. Make every failure to obtain a resource
a typed value that returns the affine inputs it did not consume. Put the runtime
inside the same envelope as the writer's code. Make every fact that survives a
call readable from the callee's declared parameter modes, declared types, and
declared contract, so D1 has no expressible form. And make every value's store
readable from the value's own type, so D1's sibling has none either.

The first draft did all of that and was unusable, because it had no answer to
*what is a length, arithmetically?* The second was unsound because it had no answer
to *what identifies a store, and what identifies a measured value across a
consume?* The third answered the first with a datum keyed on a parameter position
and the second with a place, and both answers were too narrow for their consumers.
Sections 3.1 and 3.2 answer all three, and everything else is built on them.

### 1.4 What this design does not decide: execution contexts

A scheduler that switches contexts, an interrupt handler, and a per-task kernel
stack are **out of scope for this batch, by the orchestrator's ruling**, and this
file states the fact rather than filing it as an open question. No source construct
in v0.40 or in this design creates, enters, or switches an execution context;
`program_kind := "command"` is the whole production (177) and [FN-7] admits exactly
one entry, so an `interrupt fn` does not parse. Program 4.1 is written accordingly:
it is a cooperative run queue of state machines that advance on one chain, not a
scheduler that switches stacks.

**What the follow-on inherits, and what it must reopen.** The third draft claimed
nothing here would have to be reopened. That claim was false in two places and is
withdrawn; the table below states the interface and marks each row with whether it
is inherited or owed.

```text
| this design fixes                | what a context switch must do with it                   |
|----------------------------------|---------------------------------------------------------|
| E carries one stack item per     | inherited: a new context is a new item of E, measured    |
| execution context [STK-3]        | by [STK-3] over its own whole chain, and creating one    |
|                                  | is an acquisition covered by [RES-1]                     |
| a store's identity is a region    | inherited: a switch cannot change a value's type, so it  |
| in its type [PROV-1]             | cannot change which store a value belongs to             |
| an extent item is named by its    | **owed**: an item is per (occurrence, context), so the   |
| reserving occurrence [PROV-5]    | context count enters [RES-2]'s item vocabulary and       |
|                                  | [RES-3]'s closed-expression sentence. [PROV-5] refuses   |
|                                  | the multiplicity it cannot count until then              |
| envelope accounting is per       | inherited: the per-domain map of 3.3.1 composes per      |
| domain and peak-based [RES-5]    | context; a switch transfers no peak and creates no domain|
| disposal is structural and        | **owed**: [LIV-1] is a per-join check over one function's|
| explicit [PROV-6]                | [FN-1] graph. A context that dies is not an edge of any  |
|                                  | such graph, and the successor owes a rule that a context |
|                                  | may not be abandoned holding a linear value              |
| a loan is held by a value for its| **owed**: [OWN-5]'s exclusivity argument is over one     |
| whole life [L10, VIEW-2]         | thread of control, so a suspension point needs the same  |
|                                  | no-live-loan premise [STK-1] gives a tail edge           |
```

The interference question that follow-on has to answer, and that a stack item does
not answer, is stated once here so it is not lost: a preemptible context is a
second thread of control, and [CAP-1] 1962 makes `own`, `&`, `&uniq` and place
overlap the complete interference vocabulary. A handler that preempts a foreground
activation holding a `&uniq` on a place the handler touches makes two usable
mutable paths to one place live at once, which [OWN-9] promises does not happen.
The successor is a masking discipline the checker can hold plus proved disjointness
between a handler's reachable place set and every place a preemptible statement can
hold a loan on, judged by the machinery [PAR-1] already has. That is a language
feature of the size of `par`, and it is not this batch.

**What a cooperative run queue can and cannot do in this version.** Stated
plainly, because 4.1 is the only witness this design has for goal A's control
shape and a reader should not over-read it.

- A task **can** own a resource. `struct Task['s] { block: PoolVector<'s, u8, 256>; state: u32; }`
  is linear [PROV-6], a `FixedVector<Task<'s>, 32>` of them is linear, and
  `dispose queue using (blocks);` destroys the whole tree. This is new in this
  draft; under the third draft the same program was undestroyable.
- A task **cannot** keep a view across a turn. A view is loan-bearing, and
  [CNT-4] admits no loan-bearing value into a field, an element, or a payload.
  The task holds the owner and forms the view inside the turn that uses it.
- A task **cannot** have an outstanding I/O operation. A may-suspend system
  operation retains its argument loans until its `loan-released` milestone
  ([FN-1]'s target summary), so the loan is held inside the call and the call
  completes within one turn.

That is acceptable for this design, and the reason is the same for both
prohibitions: a suspended operation and a stored loan are one request, to hold an
exclusive claim across a point where another thread of control may run, and that
claim is what the follow-on's suspension premise has to define. Building either
half now would fix the answer before the rule that judges it exists. What a writer
loses is real: a driver that wants to overlap two reads writes two turns and one
state field, and the compiler checks none of that correspondence.

---

## 2. The laws

Seventeen laws. Every rule in section 3 is an instance of one of them, and **a
rule that cannot name its law is not admitted.** L1 through L9 are the resource
laws, L10 through L15 the container laws, L16 and L17 the two the first falsifier
round added. One law is restated in this draft, L13, and that restatement is the
largest change in the file. Each law states its rationale and the owner ruling or
evidence it rests on in one sentence; ruling ids cite
`EVIDENCE-owner-discussion-2026-08-31.md`.

**L1. The envelope is the program's promise, and the promise is made in two
stages.** *A resource-closed program declares one finite, shaped envelope `E` and
promises that on every legal execution, and on every finite prefix of an infinite
one, its demand for covered resources never exceeds `E`. The judgment that a
program makes that promise is a source judgment, a deterministic function of
program text and compiler version alone, and every per-domain bound it establishes
is a closed expression in compile-time constants, type-level constants and
runtime-profile symbols. Computing `E`'s concrete figures, and checking that a
selected target and runtime can carry them, is a target-stage qualification
obligation whose failure cites no language rule.*
Because acceptance may not depend on a register allocator or a linked runtime:
owner ruling R13 (`L7036`), B8, [SCOPE-2] 18, [STOR-6] 745.

**L2. No resource is ambient.** *Every covered resource enters the program as a
capability value the runtime hands to `main`, or as a store the program reserves
in a region block it owns, and travels only by ordinary ownership; there is no
ambient allocator, thread source, or stack pool.*
Because only a held value makes heap-freedom a signature fact: probe `p5_ambient`
allocates while holding nothing and is **accepted today**, and [FN-7] 1242's "there
is no ambient system state" loses its last exception.

**L3. Nothing fails silently, and nothing grows behind the writer.** *Every
operation that can fail to obtain a covered resource returns a typed value naming
the failure and handing back every affine input it did not consume; no operation
traps, aborts, retries, falls back, or promotes a store to a larger one.*
Because v0.40 claims zero writer-reachable runtime-trap families (spec line 6)
while heap exhaustion still ends a process with no source value: owner ruling R12
(`L5657-5666`), B3, audit answer Q8.

**L4. No hidden growth.** *No operation both uses existing capacity and acquires
new capacity; every operation that may acquire capacity takes a provider, names
its allocation effect, and returns a typed failure, while every operation that
only uses existing capacity is total under a proved capacity requirement and can
allocate on no path.*
Because one `push` cannot carry one return type and one effect row across backings:
owner ruling R5 (`L2332`), B2, B3, X1. The third draft wrote "takes an owner and a
provider", which excluded this law's own constructors.

**L5. The runtime is inside the envelope.** *The artifact `E` describes is the
writer's code, the compiler-derived cleanup and drop glue, the `par` runtime, and
the target adapter together, from the frame the environment hands the program to
the frame it takes back; a resource any of them needs is an item of `E`, or the
program is not resource-closed on that target.*
Because a guarantee that stops at the edge of generated code is not one: owner
ruling R12, B12, the ledger read in 6.1.

**L6. Shape, not bytes.** *`E` is a list of tangible resources (contiguous aligned
extents, per-class slot counts, per-context stacks, lane counts) and never one byte
total. A store the program itself reserves is shaped by the same rule: a reserving
operation that needs an alignment or a separately grantable extent produces its own
`region` item and is not folded into a stack total.*
Because sixteen bytes holding four four-byte objects, the first and third released,
cannot serve an eight-byte request, and a deployment reading one stack number
cannot tell an alignment failure from a size failure: owner ruling R12, B9, B11.

**L7. Lowering before judgment, and a tail call is a dead caller frame.** *Tail
recursion, including mutual tail recursion, is rewritten into loops before any
resource judgment runs; an intra-component call edge is a tail edge exactly when
the caller's activation record is dead at the jump, and never because the call is
written in a return statement.*
Because an optimization that may or may not fire cannot be a premise of a guarantee
and a syntactic condition cannot see a live loan into a caller's frame: owner
rulings R3 (`L989`) and R12, B10, probes `f2b` and `f8_tailframe`, accepted today.

**L8. Demand is computed as if every acquisition succeeds; a store's own refusal
is an ordinary fact.** *The judgment replays each execution assuming every covered
acquire succeeds, and may never conclude that demand is small because a failed
acquisition would have ended the program. It does read the store's own post-state
relation on a refusal edge, because `room(store) = Z` is a fact about the store.*
The first half removes the circularity; without the second the checked spelling
changes a loop's summary by nothing: B8, owner ruling R12.

**L9. Stock, not flow, and a total operation at a capacity boundary must say what
it dropped.** *Resource-closedness bounds what is held at once and what is consumed
irreversibly; it never bounds how many times a program acts. An operation may be
total at a capacity boundary only when the value it displaces is copy and its
displacement is a published relation the caller can read; a silent drop of an
affine value is a refusal wearing a disguise and is inadmissible under L3.*
The first half is why a service loop runs forever with one live slot; the second is
why "overwriting is this ring's semantics" cannot be written on every bounded
store: B8, owner ruling R12.

**L10. A view is a value, and it holds its own loan.** *A view is an affine value
with a static type, not a reference the callee writes through; it holds, for its
whole life, a loan of its own strength on the range it reaches of every place in
its resolved origin set, beginning at formation and ending when the view value is
consumed or released; a function that changes a view's state consumes it and
returns the new one. A loan covers every binding the address computation of its
place reads, for the loan's whole life.*
The first clause answers write-back without a hidden protocol, the range is what
makes a `par` fill over one owner expressible, and the last is what round 2 found
missing when a view formed at `table[k]` left the offset writable: owner's settled
decision of 2026-09-03, B6, probes `f1c`, `f1d`, `f2b`, `r1_twouniq`.

**L11. Length is a type fact or a contract fact, never a guess.** *At every program
point the checker's knowledge of a sequence's measures comes from exactly one of:
the type, an established fact with live support, a compiler-owned measure datum, or
a verified contract relation; no rule infers a measure from the shape of an
argument, the name of a callee, the absence of a write, or what a body was seen to
do.*
This is D1 stated as a law, and it is why the repair is not "fix the flag" but
"have no flag derived from an actual to be wrong": `EVIDENCE-sweep-D1.md`, probe
`d1`, accepted today.

**L12. The initialized prefix is a stack, and the language says so.** *A prefix
sequence's storage is exactly `[0, len)` initialized and `[len, cap)` raw; the
boundary is checker-maintained typestate carried by the owner's static type, and no
per-slot tag, occupancy bitmap, or runtime discriminant is language state. A prefix
admits append at the end, removal from the end, removal from the middle by exchange
with the end, and exchange of two positions; it does not admit removal from the
front, and a kernel that needs a queue gets a second owner whose initialized region
is a rotation of the prefix.*
With no per-slot state the checker never needs a quantified proposition over slots,
and occupancy at a stable index is ordinary data: `FixedVector<Option<T>, N>` with
element-position `replace` is that program, and probe `r2_7` compiles its shape
today. Owner's settled decision; audit answers Q2, Q4, Q10.

**L13. A value's store is a component of its type, and acquisition and release are
symmetric over it.** *Every store the program can exhaust is named by one region,
minted where the store is reserved or where the runtime hands it in, and every
value that store backs carries that region in its own type. A value whose backing
is reclaimed per value is **linear**: it has no compiler-derived release, and it
leaves a scope only by being moved out or by being disposed to the store its type
names. Linearity and disposal are both closed under containment: a nominal, an
enum, or a container reaching a linear value is linear, and disposing it disposes
every linear value it reaches. No source construct selects, replaces, or observes a
release action.*
The first sentence is round 3's rank-one repair: the third draft identified a store
by the place its provider stood in, so a `move` plus a reinitializing `set` handed
a lease to the wrong store while every rule agreed. A type travels with a value and
a place does not. The rest is round 3's rank-two repair: the third draft closed the
linear class under containment and left disposal a list of four leaf operations, so
a container of leases could be neither dropped nor destroyed. Together they remove
an invisible free, which probe `r2_5` shows the language has today. B2's drop order,
audit answer Q10, [STOR-3] 683, [EFF-2] 1421.

**L14. An `AppendView` reaches only what it appended.** *An `AppendView` presents
the spare window `[base, cap)` of its owner, where `base` is the owner's length at
formation; its own `len` counts what was appended through it and starts at zero, no
operation on it reaches an index below `base`, and no operation on it decreases the
owner's length.*
This is what lets a caller's length fact stay alive across a callee that appends:
B6, the owner's third call rule of 2026-09-03.

**L15. The descriptor's capacity is a value; the allocator's extent is not.**
*`len(v)`, `cap(v)` and `room(v)` are the descriptor's own logical measures and are
readable as ordinary `u64` values. No operation observes the physical extent the
allocator provided, and every operation that changes a descriptor's capacity
publishes the exact new capacity. Every operation that writes a measured place
publishes the new value of each of that place's measures, including the ones it
did not change.*
The first draft forbade reading `cap` and `room` on a rationale that only forbids
reading the allocator's size, so every pop proved and no push did: B3, audit answer
Q9, probes `q24`, `v25`, `v26`. The last sentence is round 3's arena finding: a row
that published its cursor and not its extent killed the extent at the first
allocation, and no arena loop had a bound.

**L16. One measure algebra, and one goal disposition.** *`len`, `cap` and `room`
are one-place terms of the term language, defined once with their support, their
kills and their standing identities, over every measured place: sequence owners,
views, and providers alike. Every consumer of a numeric goal asks one question,
whose complete admitted derivation is stated once; no rule grants a proof route to
a construct by name.*
A language in which "can this inequality be derived?" depends on which construct is
asking has several provers and a writer can reason about none of them; probes `v25`
and `v26` are the same proof asked twice with opposite verdicts. [ENT-1] 2645.

**L17. Affine liveness agrees at every join, and a linear value never reaches a
scope exit alive.** *A binding's live-or-dead status must be the same on every
predecessor of every join and at every loop head; a disagreement is a hard error at
the join. Consequently a compiler-derived release on a scope-exit edge is
unconditional, exactly as [STOR-3] requires, and there is no runtime state that
says whether it should run. A linear binding [L13] that is live on any edge leaving
its scope, a `propagate` error edge included, is the error, because no derived
release exists to carry it.*
The reinitializing `set` makes liveness non-monotone, and [OWN-11] and today's
`Semantics/Unsupported: OwnershipJoin` avoid the question rather than answering it;
the same per-edge check makes linear disposal checkable. Probe `f3`; [ENT-5]'s own
all-predecessor join.

---

## 3. The rules

Ten families and fifty-two rules. `[MSR]` is the measure terms and the proof
surface, `[PROV]` the stores, their providers and their disposal, `[RES]` the
covered set, the envelope and the judgment, `[STK]` the stack, `[RUN]` the
runtime's own closure and the environment's half of the bargain, `[CNT]` the
sequence owners and their typestate, `[VIEW]` the views and the commit event,
`[LIV]` affine liveness and the transformation statement, `[CALL]` what survives a
call, and `[SEQ]` the operation inventory.

**Every rule states five things: the judgment it creates, the fact it publishes,
what it amends, what it depends on, and its law.** A rule that creates no judgment
writes `*Judgment:* none` and says what it is instead; a rule whose soundness
argument cites no unchanged rule omits `*Depends:*`. Section 3.13 is a **collation
of those `Amends:` and `Depends:` lines and carries nothing else**: it is written
last, from the rules. A register row with no rule behind it, an `Amends:` or
`Depends:` line no row carries, a rule appearing in both lists for one line range,
and a rule a `Depends:` line cites whose subject any `Amends:` line in this file
renames, retires or redefines are each a defect of this file. That last condition
is mechanical, and it is the one that would have caught round 2's [SET-2] finding
and round 3's [OWN-5] finding without anyone remembering to look.

The family is `[PROV]` and not `[CAP]` because [CAP-1] already exists (1962) and
rule ids are never reused. The collision is worth a sentence: [CAP-1] says the
kernel defines *no writer-visible capability category and no system-specific
permission*, and this design does not add one. A provider is an ordinary affine
value, held under `own` or `&uniq`, judged by place overlap and by the ordinary
effect row. "Capability" here means *a value you must hold in order to act*, which
is what `FilePermit` already is.

Three families or rules the earlier drafts had are gone. `[BLD]`, the `par`
builder, is deleted outright; its ids are retired and not reused. The second
draft's separate view provenance rule was merged into [PROV-3], and this draft
narrows [PROV-3] again, to views alone, because a store's identity is now in the
type and needs no origin set. `[CNT-5]`, the per-owner release table, is deleted;
[PROV-6] states its content in one sentence for every type, including the ones
[CNT-5] could not name.

### 3.1 `[MSR]`: measures, and the one goal disposition

This family is first because everything else consumes it. It adds no statement
form and no type; it is a specification amendment.

**[MSR-1] Three measure terms, over one place, for every measured value.**
`len(P)`, `cap(P)` and `room(P)` are terms of the [ENT-2] term language, of
fragment type `u64`, where `P` is an admitted place. They are defined once, here,
for every *measured* type, and which measures a type has is table data rather than
a rule with an exception:

```text
| measured type            | len                  | cap                 | room          |
|--------------------------|----------------------|---------------------|---------------|
| array<T, N>              | N                    | N                   | Z             |
| the prefix owners        | initialized elements | slots               | cap - len     |
| FixedRing<T, N>          | queued elements      | N                   | cap - len     |
| Span, MutSpan            | viewed elements      | len                 | Z             |
| AppendView               | appended elements    | the window          | cap - len     |
| Arena<'s, BYTES, ALIGN>  | cursor bytes         | BYTES               | cap - len     |
| Pool<'s, T, N>           | live slots           | N                   | cap - len     |
| FileFactory              | live handle records  | the profile's       | cap - len     |
|                          |                      | handle-table row    |               |
| Heap<'s>                 | none                 | none                | none          |
```

`Heap<'s>` has no row because L6 says a general store has no measure that means
anything; that is the absence of table data, not an exception clause on a total
definition. `FileFactory` has one because the runtime's handle table is a covered
store [RES-1] and a covered store whose measures no place names cannot appear in a
writer's invariant, which is what round 3 found missing.

An admitted place for a measure term is a `place` [GRAM-5] formed with field
selections, `deref` wrappings **and subscripts**, whose final selected type is a
measured type. The subscript admission is the change: `len(table[i])` is a term,
so a container of containers has provable operations. A subscripted place's own
[OP-4] obligation is judged independently and is not weakened by occurring under
a measure term.

*Judgment:* none by itself. *Publishes:* the term. *Amends:* [ENT-2] clause (b)
(2675), which today admits `len(P)` only for `array`, `slice` and `buffer`, and
only for subscript-free places. *Law:* L16.

**[MSR-2] Support is descriptor storage, and a kill is an ordinary [ENT-5]
event.** A measured value's storage is two disjoint parts, exactly as [STOR-1] and
L12 already describe the object: its **descriptor storage**, the length, capacity
and head words its type carries, and its **element storage**. The support of a
measure term over `P` is:

- `P`'s descriptor storage;
- every borrow or content holder any prefix of `P` reads through; and
- the support of **every** offset occurring anywhere in `P`, not only the last.

The kill is then [ENT-5]'s own rule with no new overlap notion: a measure term
dies exactly on an [ENT-5] event whose written place overlaps its support, where an
event is any [SET-1] commit, [SET-2] commit, consume, scope exit, or **any action
carrying a `writes` occurrence that projects onto that storage under [EFF-2]**, a
call and a compiler-derived release alike. Stating the kill over the effect row
rather than over a list of syntactic forms is what keeps it closed when a later
family derives a new action.

Four consequences follow from that one definition, and none is an exception
clause.

- An **element write** does not kill, because element storage is not descriptor
  storage. This is [ENT-5]'s existing sentence obtained rather than asserted.
  Probe `w4` is that program today, accepted.
- A write to a **sibling field** does not kill: `len(deref(ring).flags)` has
  descriptor storage inside `deref(ring).flags`, which `deref(ring).tail` does not
  overlap. Probe `r2_4` shows today's compiler kills it and `r2_4b`/`r2_4c` bound
  the behavior: the current implementation is root-granular where [EFF-2] on the
  same statement is field-precise, and this rule makes the measure use the
  precision [EFF-2] already computes. This is the same move [PROV-4] makes for
  `allocates`.
- A write to an **offset** does kill, at every level of the projection, so a fact
  over `len(grid[i][j])` dies when `i` is written and not only when `j` is.
- The nested case comes out right without a second sentence: `set grid[i][j] = x;`
  writes element storage of `grid[i]`, so `len(grid[i])` survives, while
  `replace grid[i] = w;` writes `grid[i]`'s descriptor storage, so it does not.

The third draft stated the kill as [OWN-7] overlap with the descriptor *place*,
which is prefix-based, so it killed every measure on every element write. Moving
the granularity into the support, where [ENT-5] already puts it, is the repair.

At every program point at which `P` is live, these hold implicitly:

```text
Z <= len(P)          Z <= room(P)          len(P) <= cap(P)
cap(P) = N           for a type whose capacity is the constant N
```

and the three-term identity `len(P) + room(P) = cap(P)` is appended, as the two
inequalities `len(P) + room(P) - cap(P) <= 0` and `cap(P) - len(P) - room(P) <= 0`,
to [ENT-6] 3001's automatic affine-premise sequence, with the empty support every
standing fact has. That is the shape [ENT-6]'s premises already take, it is usable
by `AUTO`'s families unchanged, and it keeps the identity out of L0, whose
uniqueness argument [ENT-4] 2854 rests on the difference-bound shape. The third
draft gave the identity to "the affine domain, where [INV-1] already carries
relations of that shape", which is not a home: [INV-1] 3099 admits four relation
roots and carries no standing premise, and every discharge in this file uses it.

*Judgment:* none. *Publishes:* the implicit facts and the two automatic premises.
*Amends:* [ENT-2]'s implicit-fact sentence (2722), [ENT-5]'s support and kill
sentences (2857-2887), whose length-term support becomes the descriptor-storage
relation above and whose kill classes (a) through (d) gain the effect-row
statement, and [ENT-6] 3001's automatic affine-premise sequence, which gains one
specification-fixed member. *Depends:* [ENT-4] 2854, whose difference-bound
uniqueness argument is why the identity is a premise and not an L0 fact. *Law:*
L16.

**[MSR-3] Measure datums, and where an image dies.** A **measure datum** is a
compiler-owned immutable [ENT-2] term of fragment type `u64` with **empty
support**: no [ENT-5] event kills it, no place occurs in it, and no later write
retargets it. It is the device [ENT-2] already has for a `for_stmt` capture and a
[SET-1] commit value, extended to one more producer. There is exactly one former,
and it is keyed on what a datum denotes rather than on where the value came from:

```text
a datum is identified by (program point, admitted place P, measure), is
compiler-owned and immutable, and is established equal to <measure>(P) at that
point
```

Two placements exist, and no third:

```text
entry placement       body entry, for each parameter of measured type and each
                        measure it has; the datum denotes that parameter's measure
                        at entry
call placement        one call's pre-transfer point [ENT-5], for each operand
                        place of measured type and each measure it has, reading a
                        borrow operand through its resolved referent and an own
                        operand as its value before transfer
```

The borrow half is the split [FN-8] 1269 already makes for a goal actual, applied
to the datum former. The third draft minted a datum only for an `own` operand,
which left four of its five consumers naming a datum nothing produced.

A **view carries the call datums of its own formation call**, one per measure of
its origin place, for its whole life. That is what `absorb` names [VIEW-3], and it
is exact rather than approximate because the view holds an exclusive loan on that
origin from formation to consume [L10], so no event between the two can change the
owner's measures except through the view itself.

One static datum per program point is enough, inside a loop body included, for
[ENT-2] 2687's own reason: forward flow visits every statement once, so every fact
derived about a per-point datum holds of each dynamic evaluation separately. That
is [ENT-2]'s argument for a commit value and a capture, stated here because a datum
is the third such term.

Three rules read datums and nothing else does. A [FN-9] or [FN-8] clause operand
naming a parameter's measure denotes that parameter's **entry datum**, so a
consuming use of an `own` parameter cannot invalidate it and a helper that writes
`let acc = move out;` can still state `ensures ile(len(written), cap(out))`. A
[SEQ-0] declared relation naming an operand's measure denotes that call's **call
datum**, so it survives the argument consume that the same statement performs.
And [VIEW-3] step 4 names the view's carried formation datums. A fourth reader the
third draft carried, `let x = move p;`, is deleted: [ENT-3.S5]'s copy equality plus
[ENT-5] 2888's pre-kill closure already carry the measure across the consume, and
[LIV-2]'s distinct-term rule stops the revival it was written to prevent.

Measures also carry [ENT-6] affine value images, formed and transferred exactly as
for a live own integer binding: an operation's declared relation over its call
datum and its result installs the result's image, a whole-binding `set` [LIV-2]
makes the target denote that image, a join keeps an identical image or the common
nonconstant form plus one fresh delta atom, and a loop's continuing kill replaces a
loop-carried measure by a fresh header atom. **An image dies exactly where a fact
over the same term dies**: same support, same events. And an [INV-1] affine atom
over a measured place is keyed by the [ENT-2] **term**, so a reinitializing `set`
retires the old atom and introduces a new one; a header invariant over an updated
owner is re-established on the backedge from the operation's declared relation over
its call datum, which has empty support. That last sentence is the derivation every
`update` inside a loop rests on, and 4.1's walkthrough writes it out.

*Judgment:* none by itself; a datum is formed, never proved. *Publishes:* the
datum and the image. *Amends:* [ENT-2]'s term list (a new clause beside its capture
and commit-value clauses), [ENT-5]'s call-boundary paragraph (2892-2899) and its
FN-9 entry-image-stability paragraph (2879-2884), which are replaced by the datum
rather than repaired, [FN-9]'s `M(c,q)` (1345, a datum operand is always live) and
its parameter-entry-image sentences (1310, 1320-1322), and [ENT-6]'s image
formation, join and loop-header paragraphs (2970-2996). *Depends:* [ENT-2] 2687,
whose one-static-term-per-statement argument is why a per-point datum is sound
inside a loop; [FN-8] 1269, whose borrow-versus-own actual split the call placement
reuses. *Law:* L11, L16.

**[MSR-4] One numeric goal disposition, shared by every consumer.** [ENT-6]
states once the complete ordered derivation of a numeric goal, and every consumer
submits its goal to it:

```text
1  contradiction in the current combined state             [ENT-4]
2  the exact signed fact, when the goal has one
3  the closed L0 state
4  DIRECT over the affine domain
5  AUTO over the affine domain, exactly the zero-, one-, unordered-pair
     and final-L0-image families with their two integer tightenings
6  the affine-left / L0-right bridge, for every live measure term, every
     measure datum, and every live own integer binding having a current image
```

The consumers are exactly: [OP-4] subscript bounds, [SYS-8] system range, [OP-2]
integer domain, [OP-9] allocation fit, [FN-8] requirements, [FN-9] normal-result
relations, [INV-1] invariant targets, and the operation-domain obligations of
[SEQ-0]. **The per-family route lists retire.**

*Judgment:* the disposition itself. *Publishes:* the disposition of every numeric
goal. *Amends:* [ENT-6] 3034, 3041, 3069 and 3078, the four per-family route and
attach-site grants, which keep their normalization and lose their route grant, and
[FN-9]'s `prove_ordering` route, whose undocumented direct-affine branch becomes
one of the six steps. *Note:* this rule is why the design does not have to be
revisited when [SEQ] adds an operation: an operation adds a goal, never a route.
*Law:* L16.

**[MSR-5] The contract surface has its own production, over terms.** A `requires`,
`ensures`, `header_invariant`, `invariant_stmt` or `proof_use` operand is a
**term** of the [ENT-2] term language, not an `atom` of [GRAM-5]. The amendment
goes where the refusal is. [GRAM-5] 265's `atom` production has no `call`
alternative, so `ile(len(x), y)` derives nowhere and [GRAM-9] is only [DIAG-1]
1606's attribution of that failure; the third draft scoped [GRAM-9] and left the
production that actually refuses the form unregistered and unchanged. Probe `w3` is
that rejection, with the compiler's own mechanical fix naming `define`.

The contract surface therefore gains its own productions and `atom` is untouched:

```text
clause_expr    := relation_op "(" clause_operand "," clause_operand ")"
clause_operand := affine_expr
affine_factor  := literal | ent2_place | measure_term | "(" affine_expr ")"
```

`requires_clause` and `ensures_clause` take a `clause_expr` instead of an `expr`;
`ent2_place` is [ENT-2] 2675(a)'s place grammar and `measure_term` is [MSR-1]'s
three formers over one admitted place. Widening `atom` instead would reopen nested
runtime calls at every argument, subscript offset and `for` endpoint, which is
[GRAM-9]'s actual purpose. So `requires ile(len(source), room(out));` is writable
directly, and so is `invariant fill: ile(r.fill, 8_u64);` over a struct field path.

`affine_factor` **gains** two alternatives and loses none; the second draft wrote
that the production was replaced, which would have deleted the literal and the
parenthesized group and unformed every invariant in this file. One consequence is a
real capability gain and is stated rather than left to arrive: [ENT-2] 2675(a)
admits a named const as a tracked place root, so a named const becomes an affine
atom, which [INV-1] 3107 forbids today.

One shape is **not** admitted, and the third draft's own example was ahead of its
production: `invariant order: ile(table[i], n);` does not derive, because
`ent2_place` admits no subscript. A measure term over a subscripted place is
admitted and an ordinary value at one is not, and the reason is not an accident of
the grammar: `len(table[i])` is a property of a descriptor and carries no [OP-4]
obligation, while `table[i]` in an erased clause would carry one, and a proof
obligation inside erased proof syntax has no program point at which to discharge.

*Judgment:* the ordinary [FN-8]/[FN-9]/[INV-1] admission over the widened operand
set. *Publishes:* nothing new. *Amends:* [GRAM-5] 265-266 (a new `clause_expr` and
`clause_operand` production; `atom` and `atom_list` unchanged), [GRAM-4]'s
`requires_clause`, `ensures_clause` and `affine_factor` productions, [FN-8]'s
clause-expression judgment (1256-1257), [FN-9]'s operand list (1306-1308), and
[INV-1]'s atom sentence (3107); [GRAM-9] is unchanged and needs no scope sentence.
*Verified today:* probes `w3`, `q1`, `q9`, `r1_lenatom` and `r1_field` are parse
rejections, so this is an amendment and not a compiler defect. *Law:* L16.

### 3.2 `[PROV]`: stores, providers, provenance, and disposal

**[PROV-1] A store's identity is a region, and the region is in the type.** This
is the rule the fourth draft is written around, and everything else in this family
is derived from it.

A **store region** is a region that names one store. A region becomes one by being
named as the store argument of a reserving occurrence [PROV-5], or, for the heap,
by being the entry's own store-region parameter. A region may be named by **at most
one** reserving occurrence; a second occurrence naming a region already used is a
hard error citing PROV-1 at that occurrence's `targ`, with the restructuring `open
one region per store`. Because [OWN-3] 573 makes region identifiers unique within a
function, and probe `w1` confirms the compiler enforces it, a store region's
spelling denotes exactly one store per entry of its block.

Every value a store backs carries that store's region in its own type. There are
three stores and three shapes over each, and the table is the whole vocabulary:

```text
| store       | provider                | one value                 | one sequence          |
|-------------|-------------------------|---------------------------|-----------------------|
| general     | Heap<'s>                | HeapBox<'s, T>            | HeapVector<'s, T>     |
| bump extent | Arena<'s, BYTES, ALIGN> | ArenaBox<'s, T>           | ArenaVector<'s, T>    |
| slot pool   | Pool<'s, T, N>          | PoolSlot<'s, T>           | PoolVector<'s, T, N>  |
```

A confinement region and a store identity merge into one parameter here, because a
store's lifetime and a store's identity are one fact about one reservation.
`Arena<'s, BYTES, ALIGN>` additionally carries its extent, so `cap` is a standing
type fact [MSR-2] that no event kills, which is what a pool already had and an
arena did not.

**Preservation is a closure property and needs no clause of its own.** A value's
store is a component of its type; no value-forming step in the language changes a
value's type; therefore no value-forming step changes a value's store. That covers
`construct`, field projection, container placement and removal, enum payload
construction and `match` binding, multi-return, a control-flow join, a
value-in / value-out row's result, an argument transfer and a return, in one
sentence, and it covers every future step for the same reason. Two values have the
same store exactly when their types name the same region, which [OWN-12] 645 and
[TYPE-5] 372 already decide by exact identity: region substitution controls type
equality, argument types match declared parameter types exactly, and v0.40 has no
subtyping, so a branded type is invariant in its store region without a variance
design.

The third draft carried this in an [OWN-5] origin set, which its four preservation
sentences did not carry through a field, an element, a payload or a
value-in / value-out row, and which a `move` of the provider plus a reinitializing
`set` could point at a different store. Both defeats are type errors here.

`Heap<'s>` is delivered as an `own` entry parameter and lives for the program.
The `command` standard-input table [FN-7] gains ordinal 5:

```text
| ordinal | label        | written mode and type | supplied value                                      |
|---------|--------------|-----------------------|-----------------------------------------------------|
| 5       | command.heap | own Heap<'s>          | the one general store the runtime minted before main |
```

and the entry may declare **exactly one region parameter**, admitted only when it
selects that row, which names the heap's store region; program start supplies it
and it outlives every region of the program. [OWN-3] 575's incomparability of
distinct caller-supplied regions is what makes it invariant. The row is optional
like every other: a `main` that omits it receives no `Heap` and cannot obtain one
[PROV-2]. The `Heap` `main` receives is dropped on the return edge with the
**empty** release row: the store itself is the runtime's, the program returns the
handle, and no covered acquisition or release happens there.

*Judgment:* one store per store region, checked at each reserving occurrence;
provider and branded types are nominal and closed, and no source declaration
introduces another; plus the ordinary [FN-7] label, order, mode and type checks.
*Publishes:* each value's store, as a component of its type; the store's measures;
and the whole-program fact `heap-unreachable` when the entry row is absent.
*Amends:* [TYPE-2] 352, which gains the nine branded nominals above and from which
`box<T>`, `arena<'r, T>` and `buffer<T>` retire from the writer surface; [TYPE-7]
471, whose closed deref domain becomes `&'r T`, `&uniq 'r T`, `HeapBox<'s, T>`,
`ArenaBox<'s, T>` and `PoolSlot<'s, T>`; [GRAM-3] 204-207, whose fixed `box`,
`arena`, `slice` and `buffer` type productions retire in favour of ordinary TYPEIDs
with `targs`; [FN-7]'s table (1221-1227), its "declares no region parameters"
sentence (1212), its canonical five-input byte sequence (1246), and its effect-row
sentence (1214), whose `allocates(heap)` becomes `allocates` over the entry's own
labelled provider input. *Depends:* [OWN-3] 573 and 575, for uniqueness within a
function and incomparability across the boundary; [OWN-12] 645 and [TYPE-5] 372,
for exact region identity in type equality, which is the whole of the invariance
argument. *Law:* L2, L13, L16.

**[PROV-2] Unforgeable, uncopyable, and taken as a loan.** No source construct
produces a provider; a `Heap<'s>` exists only because the runtime minted exactly
one before `main`, and an `Arena` or `Pool` only as the result of a reserving
operation [PROV-5]. No operation duplicates, reconstructs, compares, serializes, or
derives a provider from a non-provider value.

An operation that allocates from a store, or releases to it, takes that store's
provider as a written `&uniq 'b` parameter and exhibits it. A provider is never
passed `own`: it is confined to its own store region, and a moved provider strands
its own store. The one `own` provider in the language is the `Heap` the entry
receives.

Every provider operation declares two regions: `'s`, the store's region, which
appears in the provider's type and in the type of everything it produces, and `'b`,
the region of the loan the call holds. They are always distinct, and [OWN-10] 636
is the general reason: a borrow of a local names a region introduced **inside that
binding's own scope**, and `'s` is introduced before the provider binding exists.
Probe `r2_2` is that rejection and probe `r2_1` is the admitted shape.

```text
| op                 | signature                                                                                          | effects                         |
|--------------------|----------------------------------------------------------------------------------------------------|---------------------------------|
| heap_take          | ['s,'b](heap: &uniq 'b Heap<'s>, value: own T) -> own Result<HeapBox<'s,T>, OutOfMemory<T>>          | allocates(heap), writes(heap)   |
| heap_release       | ['s,'b](heap: &uniq 'b Heap<'s>, item: own HeapBox<'s,T>) -> own T                                   | writes(heap)                    |
| arena_take         | ['s,'b](arena: &uniq 'b Arena<'s,BYTES,ALIGN>, value: own T) -> own ArenaBox<'s,T>                   | allocates(arena), writes(arena) |
| arena_take_checked | ['s,'b](arena: &uniq 'b Arena<'s,BYTES,ALIGN>, value: own T) -> own Result<ArenaBox<'s,T>, NeedCapacity<T>> | allocates(arena), writes(arena) |
| pool_take          | ['s,'b](pool: &uniq 'b Pool<'s,T,N>, value: own T) -> own PoolSlot<'s,T>                             | allocates(pool), writes(pool)   |
| pool_take_checked  | ['s,'b](pool: &uniq 'b Pool<'s,T,N>, value: own T) -> own Result<PoolSlot<'s,T>, PoolExhausted<T>>   | allocates(pool), writes(pool)   |
| pool_release       | ['s,'b](pool: &uniq 'b Pool<'s,T,N>, item: own PoolSlot<'s,T>) -> own T                              | writes(pool)                    |
```

`heap_release` and `pool_release` hand the content back; they are the inverses of
their acquisitions and they destroy nothing. Destroying a value is `dispose`
[PROV-6], which is a different construct. There is no `arena_release`, because
arena content is reclaimed with the store and not per value.

These are container-domain rows [SEQ-0], not [OP-1] table rows. The third draft
declared them in [SEQ-0] and amended them as [OP-1] rows at once, leaving them in
two domains with two argument forms and two diagnostic rules. `box_new` and
`arena_new` therefore **retire** from [OP-1] rather than being amended there,
exactly as `buffer_new`, `buffer_vacant` and `slice_of` do.

*Judgment:* a `construct` [GRAM-8] naming a provider or branded nominal, and every
other source route to one, is a hard error citing PROV-2 at the complete
`construct`, with the restructuring `receive the provider as a parameter, or
reserve one with pool_frame or arena_frame`; and an allocation or release call
whose provider argument is missing, is not a provider place, or is not writable is
a hard error citing PROV-2 at the `call`. *Publishes:* uniqueness of the `Heap`;
and the store's post-state measures, which are [SEQ-0] declared relations over the
call's own datums [MSR-3], stated single-state:
`len(pool) = <call datum of len(pool)> + 1` at a take,
`len(pool) = <call datum of len(pool)> - 1` at a release,
`len(arena) <= <call datum of len(arena)> + K<T>` at an arena allocation, and
`cap(P) = <call datum of cap(P)>` at every one of them. The third draft wrote these
with a primed post-state term, which the settled list rejects, over a datum its own
producer could not mint, on a result binding that does not exist. Single-state over
a live term and an immutable datum is [ENT-3.S5]'s own post-write shape. *Amends:* [OP-1]
793-798, from which `box_new` and `arena_new` retire, and [STOR-2] 680, which
defined them. *Depends:* [OWN-10] 636, which is why `'s` and `'b` are always
distinct; [OWN-6] 609, which makes an argument borrow a call-scoped temporary, the
fact probe `w8` exercises and the reason store identity may not rest on what stands
at a place between two calls. *Law:* L2, L3, L4, L13, L16.

**[PROV-3] Provenance is for loans, and a loan reaches a range.** [OWN-5]'s finite
origin set, today defined for `slice<'r, T>`, generalizes to the three views and to
nothing else. A **loan-bearing** type is `Span<'r,T>`, `MutSpan<'r,T>` or
`AppendView<'r,T>`; a value of one carries a finite set of origins, each an origin
place paired with the half-open index range the value reaches of it.

Formation makes a **singleton**: `seq_mut_span(vector: &uniq 'w table[i])` has the
singleton origin `table[i]` with range `[Z, len(table[i]))`, and
`seq_append_view(vector: &uniq 'w v)` has the singleton origin `v` with range
`[len(v), cap(v))`. A named const maps to the distinguished `immutable-const`
origin. Binding, moving, passing and returning preserve the set; a control-flow join
takes the union; a parameter of loan-bearing type starts with the singleton
containing its own formal origin, substituted at a call boundary. The **resolved**
origin set is the set minus `immutable-const`, which creates no conflicting access
and has no writable storage [OWN-5] 602, [OWN-7] 627; every rule needing a singleton
needs a singleton *resolved* set.

Four uses, and no fifth:

1. **Access strength, over the range.** An access through a value of shared loan
   strength is one shared access to the range of every resolved origin; an access
   through a value of exclusive loan strength is one exclusive access to the range
   of every resolved origin. [VIEW-1] fixes each view's strength.
2. **A loan covers its address computation.** While a loan on a resolved place is
   live, every binding that place's address computation reads is frozen: a write to
   it conflicts under [OWN-5], at the write, naming the loan. Forming a view at
   `table[k]` therefore freezes `k` exactly as it freezes `table`.
3. **A live origin set fixes its storage.** No statement may rebind the storage a
   live origin set describes: a `set` target, a `replace` target, and every future
   exchange form whose target type is loan-bearing is a hard error, wherever the
   target is reached from.
4. **Disjointness.** [OWN-7] 624's overlap test extends to ranges: two origins with
   the same resolved place overlap exactly when their ranges intersect, judged by
   the same affine reasoning [PAR-2] 1999 already performs for a single-binder
   element write. This is what makes a `par` fill over one owner expressible, and
   it is the relation a later `seq_split_at` needs.

Use 3 is stated over **loan-bearing** targets only. The third draft quantified it
over provider-derived values too, which refused `replace deref(h).b = move fresh;`
at a heap-backed field, a program probe `q2` shows the compiler accepts today and
[STOR-1] 677 calls the language's own growable-collection idiom. A store-branded
value holds no loan and aliases nothing, so exchanging one for another of the same
type retargets no loan and hides no store.

Use 2 is checkable only because [OWN-7] 624's subscript overlap stays
conservative, and the register's `Depends:` list carries that.

*Judgment:* a loan-bearing value in a prohibited position [CNT-4] is a hard error
there; a rebinding of storage under a live origin set is a hard error citing PROV-3
at the complete target `place`, with the restructuring `a view's origin set is
fixed at formation; bind a new view under a new let`; and a write to a binding a
live loan's address computation reads is the ordinary [OWN-5] conflict.
*Publishes:* the origin set, the resolved origin set, and each origin's range.
*Amends:* [OWN-5] 589-607, whose slice-origin paragraphs generalize to loan-bearing
values, whose one access clause becomes the two of use 1 over ranges, which gains
the address-computation and resolved-set sentences, and whose 603 becomes "a formal
view origin has a writable storage path inside its callee exactly when that view's
loan strength on its resolved origin set is exclusive", the callee-side twin of the
[SET-1] change below, its second sentence unchanged; 596-599's no-slice-valued-join
sentence, restated over the loan-bearing predicate rather than over one retired type
name, because the union of two loans is not a loan any rule can end at one consume;
[OWN-7] 624, which gains the range clause; [SET-1] 483-485, whose "no writable
target path may traverse a `slice<'r, U>` value" is restated as *a target path may
traverse a view value exactly when that view's loan strength on its resolved origin
set is exclusive*, which is what admits the `MutSpan` element write probe `p7` is
refused today; [SET-2] 508-513, whose region-bearing target rejection is replaced by
use 3; and [EFF-2] 1400-1404, whose slice-parameter projection generalizes to a
loan-bearing parameter. *Law:* L10.

**[PROV-4] `allocates` names a provider path, and reachability reads the leaf.**
The effect grammar's `allocates` entry takes formal-rooted [EFF-1] paths naming
provider state, in canonical order, replacing the fixed atoms:

```text
effect := "reads" "(" effect_path ("," effect_path)* ")"
        | "writes" "(" effect_path ("," effect_path)* ")"
        | "allocates" "(" effect_path ("," effect_path)* ")"
```

An `allocates(p)` entry is exhibited exactly when the body reaches an allocation
whose provider argument projects to `p` under [EFF-2]'s call-boundary projection.
A function *reaches a store* when its own row carries an `allocates` or `writes`
entry whose path's **selected type at the leaf** is that store's provider type, or
when it calls a function that does; the leaf's selected type is what [EFF-2]
already computes, so `allocates(env.heap)` on an `Env`-typed formal is a
heap-reaching row and the closure stays exact for aggregates. Because the
compilation unit is closed [PROG-1], there are no function values, and there is no
ambient store, the transitive closure of that relation over the call graph is
exact and is computed from signatures alone.

A body that allocates only from a provider it reserved itself frames out of its
own signature exactly as any other fresh-local state does, and [PROV-5] makes the
reserved extent an ordinary place of that activation.

*Judgment:* [EFF-2]'s both-ways row check, unchanged. *Publishes:* the
provider-reachability closure, and the heap-reaching path, which is the ordered
call chain from `main` to the allocation that [RES-4] prints. *Amends:* [EFF-1]'s
`effect` production (1363-1372), retiring the effect-row atoms `heap` and `arena`;
and [FN-3] 1117-1121, whose conformance effect-row normalization is defined over
"the allocation set whose members are `heap` and each alpha-mapped `arena` region"
and which becomes the set of `allocates` paths under the same parameter-ordinal and
field-ordinal identity 1121 already fixes for `reads` and `writes`, with the region
alpha-mapping applying to modes and types only. Without that second row a
conformance's signature equality is stated in a vocabulary this rule deletes.
*Law:* L2.

**[PROV-5] Reservation is an event of the region block, and its placement is
written.** Four reserving operations exist:

```text
pool_frame<T, const N: u64>['s]()                      -> own Pool<'s, T, N>
pool_extent<T, const N: u64>['s]()                     -> own Pool<'s, T, N>
arena_frame<const BYTES: u64, const ALIGN: u64>['s]()  -> own Arena<'s, BYTES, ALIGN>
arena_extent<const BYTES: u64, const ALIGN: u64>['s]() -> own Arena<'s, BYTES, ALIGN>
```

No operand supplies any of those parameters, so each call writes its complete list
in [GRAM-2]'s declaration order, type and const parameters then region parameters:
`pool_frame<FixedVector<u8, 256>, 8, 'p>()` [SEQ-0]. The written region argument
`'s` must be a region introduced by an enclosing `region_stmt` of the reserving
function; a caller-supplied region parameter is not admitted, and [PROV-1] admits
at most one reserving occurrence per region.

**Each reserves one store per entry of the region block naming `'s`.** The `frame`
forms lay the extent out in the reserving activation's frame, so it enters that
context's `stack` item of `E`; the `extent` forms produce their own
`region(name, bytes, alignment, contiguous)` item of `E`, whose name is derived
from the reserving occurrence and is not written. Storage is per occurrence in both
cases and is reused across entries. **On every edge leaving `'s`'s block the store's
release action resets it to its initial state**: a bump cursor to zero, a slot pool
to zero live slots. That action joins [STOR-3]'s release-action table beside the
owners' and the `AppendView`'s.

The reset is what makes the freshly published `len(store) = Z` true on a second
entry, and it is sound because every value the store served names `'s`, [CNT-4]
confines it to `'s`, and [LIV-1] makes it dead on that edge. Without the reset a
`region` block inside a loop republishes `len = Z` over a cursor the previous
iteration advanced, which round 3 turned into an out-of-extent write in eleven
statements. `E` is unaffected: peak demand over a resetting store is one entry's.

Frame placement is the default for scratch. Extent placement is what a page table,
an MMIO window and a DMA descriptor ring need, and L6 is the reason the choice
exists: a deployment reading one stack total cannot tell a 4096-alignment failure
from a size failure, and cannot grant the page table separately from the stack.

One multiplicity this version cannot count, it refuses. An `extent`-form occurrence
reachable from more than one execution context, or from a statement an
implementation may execute with overlapping execution under [PAR-1], [PAR-2] or
[PAR-3], is a hard error at the `targ`, with the restructuring `reserve the store
in the caller and lend the provider down, or use the frame form`. An item named by
an occurrence is one item however many activations reach it, so two simultaneous
activations would hold one committed extent and two providers each believing it
owns the whole. Section 1.4 records that lifting this is the follow-on's.

*Judgment:* the ordinary region, confinement and [OWN-5] exclusivity judgments,
plus the region-locality check, [PROV-1]'s one-store-per-region check, and the
multiplicity refusal above, each a hard error citing PROV-5 at the `targ` with the
restructuring stated there. *Publishes:* the reserved store's measures, its store
region, and its envelope item, one `stack` contribution or one `region` item.
*Amends:* [STOR-3] 683-715, whose release-action table gains the store reset;
nothing else. *Law:* L2, L5, L6, L13.

**[PROV-6] Linearity and disposal are both closed under containment.** A type is
**linear** exactly when it reaches, at any depth, a value whose backing is
reclaimed per value: `HeapBox<'s,T>`, `PoolSlot<'s,T>`, `HeapVector<'s,T>`,
`PoolVector<'s,T,N>`, any nominal with a linear field, any enum with a linear
payload, and any container whose element type is linear. A type whose backing is
reclaimed with a region or with a frame is not linear: `ArenaBox<'s,T>`,
`ArenaVector<'s,T>`, `FixedVector<T,N>` and `FixedRing<T,N>` over non-linear
elements keep their ordinary compiler-derived release, and a provider is not linear
because [PROV-5] gives its store a reset.

A linear value has **no compiler-derived release**. It leaves a scope only by being
moved out, or by being consumed by one statement:

```wf-design
dispose queue using (blocks);
dispose table using (heap, blocks);
```

`dispose p using (q1, ..., qk);` consumes the owner place `p`; each `qi` is a
**writable provider place**, not a written borrow, and the statement takes one
statement-scoped exclusive access to each, exactly as a [SET-1] commit does to its
target. That is why no `dispose` needs a region of its own. **Its judgment is a
walk of `p`'s type.** For every store region `'s` that `p`'s type names at a linear
leaf, exactly one named provider whose type names `'s` must appear, and no named
provider may be unused; the walk then visits every leaf in the order [STOR-3]
already fixes for a derived drop, releasing each linear leaf to the provider its own
type names and running each non-linear leaf's ordinary derived release. A
container's elements are visited before its backing is released, so `dispose` on a
full container is legal and the emptiness premise the third draft wrote on its two
release rows disappears.

Four things follow and are stated rather than discovered.

- **Disposal introduces no new reclamation action.** It is [STOR-3]'s own derived
  drop with the store's release substituted at each linear leaf, so a reader who
  knows how a `buffer<T>` of affine elements is dropped today knows this walk.
- **Every free is visible in a signature and in a footprint.** The statement's
  effect contribution is one write of each named provider place, which [EFF-2]
  projects to the enclosing row exactly as it projects a `set` commit. Two disposals
  of values from one store therefore conflict under [PAR-1], and a callee that
  disposes exhibits the row at its caller. Today the opposite is true and invisible:
  probe `r2_5` compiles `fn swallow(item: own box<u64>) -> result: own u64 pure`,
  and probe `w7` does the same one field deeper.
- **Disposal is modular, and no new effect category is needed to make it so.** A
  helper writes
  `fn retire['s,'b](pool: &uniq 'b Pool<'s,u64,4>, item: own PoolSlot<'s,u64>) -> done: own unit writes(pool)`
  and disposes inside it, because both formals name `'s` and the walk's requirement
  is type equality. The third draft could not: its check compared a formal origin
  token with a formal borrow's resolved place, two objects never equal, so its own
  virality paragraph described a program its judgment refused. A `disposes(item ->
  pool)` effect category was the obvious repair and is **not** adopted: with the
  brand it would restate what the two formals' types already say, and the footprint
  it would supply is the `writes` row the statement already carries.
- **Virality is real and is visible.** A function that takes ownership of a linear
  value on any path and does not return it must hold a provider for every store the
  value's type names, so it names those providers in its signature, transitively up
  to the holder. That is the honest signature fact, and it is the discipline
  `FilePermit` already imposes.

`propagate` and a live linear binding are mutually exclusive, and this rule says
so rather than leaving it to be discovered. A `propagate` error edge leaves every
enclosing scope and offers no statement position on which to dispose, so a
`propagate` in a function holding a live linear binding is a hard error citing
PROV-6 at the `propagate_let_rhs`, with the restructuring `expand the propagate
into a match and dispose on the Err arm`. Probe `w5` compiles that shape today, so
this is a refusal the design adds and a cost it owes the writer; 6.7 records it and
Q10 asks whether a release list on the statement should later remove it.

*Judgment:* a linear binding live on any edge leaving its scope, including a
`propagate` error edge and a function-return edge, is a hard error citing PROV-6 at
that edge, naming the binding, its store regions, and the providers a `dispose`
would need, with the restructuring `move the value out, or dispose it here while
the providers are live`; a `dispose` whose named providers do not cover the store
regions of `p`'s linear leaves exactly once is a hard error citing PROV-6 at the
statement, rendering the uncovered region and the type path that reaches it.
*Publishes:* the release events and each store's post-state measure. *Amends:*
[STOR-3] 683-715, whose `box<T>` and `buffer<T>` drop rows retire with their types
and whose release-action table gains the statement that a linear type has none;
[OWN-1] 558, which gains the linear class beside copy and affine; [GRAM-4]'s `stmt`
production (one added statement form) and [FORM-2], which renders it as one line;
[EFF-2] 1421's "each of these memory-reclamation actions carries the empty effect
row", which stays **true** and stays unchanged, because after this rule no memory
reclamation of store-owned storage is a derived action; [PAR-1] 1969's footprint,
through the ordinary `writes` row rather than a special case. *Depends:* [STOR-3]
694-700, whose derived-drop order and its affine-element clause are the walk this
rule reuses. *Law:* L3, L5, L13, L17.

**[PROV-7] A provider can be lent onward.** A helper that receives a provider as
`&uniq 'b P` must be able to hand it to the operation that allocates. Today it
cannot: [OWN-6]'s child reborrow admits only a locally-introduced region whose
block does not extend beyond the enclosing statement, so a reborrow into `'b` is
inadmissible and a reborrow into a fresh local region cannot carry an affine result
out. The amendment is [OWN-6]'s own reasoning applied one position further, and it
is stated over the right quantity:

> A child reborrow may name a caller-supplied region `'b` that resolved(`h`)'s
> region outlives-or-equals **when the receiving call's result type does not name
> `'b`**. That child's loan ends at the end of its receiving statement, and the
> parent resumes there.

The condition is on the **loan** region and not on region-freedom. The second
draft required a region-free result, which admits `heap_take` and refuses
`pool_take`, `arena_take` and `seq_lease`, whose results name the store's `'s` and
never the loan's `'b`; it left goal A with no `alloc_page` helper and no layered
allocator. Under the corrected condition every provider-consuming row is lendable.
One correction to the original justification matters for the next batch: nothing
derived from the child outlives the statement in the sense [OWN-6] means, a loan,
while the result's store region does travel out in its type, which is the point of
the brand rather than a leak.

*Judgment:* [OWN-6]'s admission, with one more admitted region source under the
stated result-type condition. *Publishes:* the child loan's extent. *Amends:*
[OWN-6] 611 and [OWN-4] 577, for this one form. *Verified today:* probe `r1_relend`
is `[OWN-6] InvalidChildReborrow`, and `r1_relend_affine` shows the existing
local-region escape cannot carry an affine result out. *Note:* this also unblocks
`docs/patterns.md` P17's threaded-factory shape. *Law:* L2.

### 3.3 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

```text
| class              | members                                                                        |
|--------------------|--------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context, including every       |
|                    | frame-placed provider extent [PROV-5]; every extent-placed provider store;      |
|                    | every worker-lane stack; allocator and runtime metadata; compiler-derived       |
|                    | cleanup scratch; the adapter's persistent buffers                               |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots;  |
|                    | the runtime's fixed handle table; every other runtime-owned store               |
```

Every member presents its state as one of [RES-5]'s domains; a runtime-owned store
that does not is a qualification failure of that runtime [RUN-2], not a source
condition. An extension is written and never implied:
`resource_closed(core + file_handles)` is a different, stronger declaration, and no
such extension is defined in this version.
*Judgment:* none; it fixes the domains [RES-3] quantifies over. *Publishes:* the
covered set. *Amends:* nothing. *Law:* L1, L5.

**[RES-2] The envelope `E`, over the target's profile table.** `E = E(P, T)` is,
for one program `P` and one selected target and ABI `T` [STOR-6], a finite table
with one row for each lane count `W` the target's runtime supports. Each row is a
finite list of shaped items:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count)                           interchangeable fixed-size records
stack(context, bytes)                        one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
```

No item is a bare byte total, and no two items are summed into one. Items are not
fungible: two `region` items are two extents. The table, not a row, is the
program's promise, because [PAR-1] 1988 makes parallel permission never an
obligation, so `W = 1` must always be legal.

**Which items carry a source-stage figure, and which do not, is stated rather than
quantified over.** A `region` item's bytes and alignment, and a `slots` item's
count, are computed by [RES-5]'s target-independent arithmetic and are read by
acceptance; each additionally carries the target-stage exact figure that
materialization checks. A `stack` item has **no source-stage figure at all**, and
this draft says so instead of promising one: [STOR-6] 759 defines no numeric
per-function frame ceiling, [STK-3] measures `frame(f)` after code generation, and
[RES-5]'s frame-placement domain contributes no stage-one demand. Stage one's
entire stack content is therefore premise 2 of [RES-3], acyclicity; the bytes
arrive at target stage. The third draft wrote that every item carries two figures,
which is false for the item the owner's list names first, and the false universal
hid the one domain where the marker's promise is made after the promise is checked.
*Judgment:* `E` is well-formed only if every item's arithmetic was performed in
the unbounded mathematical domain and is representable on `T`, the same standard
[STOR-6] already applies. *Publishes:* `E` itself, as a compilation artifact.
*Amends:* nothing. *Law:* L1, L6.

**[RES-3] The judgment, in two stages.** For a program `P`,
`source-resource-closed(P)` holds exactly when, on the rewritten call graph
[STK-1], every premise below is established from program text alone:

```text
1  no reachable store is a Heap                                    [PROV-4, RES-4]
2  the call graph is acyclic                                       [STK-2]
3  every covered store's demand is bounded, per domain, by the
     symbolic composition of 3.3.1                                 [RES-5]
```

**A bound is a closed expression in compile-time constants, type-level constants
and runtime-profile symbols. A per-domain figure that names a runtime value is not
a bound**, and premise 3 fails at the loop or call that introduced it, with that
value named. That sentence is what makes stage two a substitution rather than a
discovery.

The second draft carried a fourth premise, "no execution context reachable from
source can be re-entered from outside the call graph". It is deleted, not weakened:
v0.40 has no source form that declares a reentrant entry point, so the premise
refused nothing. Reentrancy arrives with the execution-context design [1.4].

For a selected target `T` and its runtime, `E-materializes(P, T)` holds when every
symbolic figure of stage one has a concrete value on `T` (frame sizes measured
after code generation [STK-3], strides and alignments [STOR-6], the runtime's own
profile rows [RUN-3]), every row of the resulting table is representable and is one
the runtime's published profile can carry [RUN-2], and the selected ABI satisfies
[STK-1]'s target-stage tail obligation. Failure here is a **target-qualification
failure** under [STOR-6] and [QUAL-2]: it stops compilation, cites no language
rule, and is not a source rejection.
*Judgment:* stage one, per domain, over the checked program; deterministic,
terminating, and free of search, budget or timeout. *Publishes:* the property, and
`E`. *Amends:* [STOR-6] 733-765, whose "the language defines no numeric
per-function frame ceiling" sentence keeps its scope for the *language* and is
joined, for a resource-closed build, by a computed per-context envelope, and whose
target-stage obligations gain `E`-materialization. *Law:* L1, L8, L9.

**[RES-4] The entry requirement, and the heap.** The entry may carry the marker
`resource_closed` before its `command` program-kind marker:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
```

The marker changes no acceptance judgment: every program is judged by exactly the
same rules. It changes two things. It makes the failure of [RES-3] stage one a
hard error rather than a reported property. And it selects which [SCOPE-3]
deferrals apply: for a marked program, stack exhaustion and covered-store
exhaustion are inside the model, and for every other program they stay deferred.

A program whose call graph reaches a `Heap<'s>` is not resource-closed, and a
`main` selecting `command.heap` is by itself the rejection. A bounded general store
is still a general store: an envelope item can promise bytes, and cannot promise
that the next contiguous aligned request has a home.
*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage one
is a hard error naming its own cause: the heap-reaching path, rendered from `main`
to the allocation and located at the offending `input_label` or the deepest `call`;
the call-graph cycle [STK-2]; or the unbounded store [RES-5]. *Publishes:* the
property as a compilation fact. *Amends:* [FN-7] 1211, which fixes main's marker
set, and [GRAM-2]'s `program_kind` production. *Law:* L1, L6.

**[RES-5] Store domains and their algebras, in target-independent arithmetic.**
Every covered store presents its state through [MSR-1]'s measures, and exactly
four domains are defined. Nothing else is admitted, and a store outside this list
contributes no envelope item and denies [RES-3].

```text
| domain                     | state         | acquire            | release        | serviceable when |
|----------------------------|---------------|--------------------|----------------|------------------|
| uniform slots              | len, cap = N  | len + 1            | len - 1        | room >= 1        |
|  (Pool; lane, task, queue, |               |                    | [PROV-6]       |                  |
|   completion and handle    |               |                    |                |                  |
|   records of the runtime)  |               |                    |                |                  |
| bump extent                | len, cap      | len + K<T>         | nothing; the   | room >= K<T>     |
|  (Arena<'s, BYTES, ALIGN>) |  in bytes,    |                    | store resets   |                  |
|                            |  cap = BYTES  |                    | with 's        |                  |
| static and frame placement | fixed offsets | none at run time   | none           | decided at       |
|                            |               |                    |                | compile time     |
| general heap (Heap<'s>)    | -             | -                  | -              | undecidable      |
|                            |               |                    |                | from E [RES-4]   |
```

`K<T>` is the compile-time constant `align_ceiling(T) - 1 + size_ceiling(T)`,
computed by [OP-9]'s existing ceiling arithmetic. It is **target-independent**, and
it is the only arena advance quantity in this design: stage one is the ceiling and
[RES-2]'s second figure carries the exact composition at target stage.

Two data additions are needed and are stated here rather than assumed. [OP-9] 983's
`(size_ceiling, align_ceiling)` table gains a pair for each nominal this design
adds, on the same aggregate composition rule it already uses, and its sentence
excluding region-bearing types from `buffer_fits`'s domain is **lifted**, because
it exists for values that could not be stored and [CNT-4] now stores them:

```text
| nominal                              | (size_ceiling, align_ceiling)                    |
|--------------------------------------|--------------------------------------------------|
| Heap<'s>, Arena<..>, Pool<..>        | (32, 16)   proof-only representation, one word   |
| HeapBox<'s,T>, PoolSlot<'s,T>        | (16, 16)   as box<T> today                       |
| ArenaBox<'s,T>                       | (16, 16)                                         |
| HeapVector<'s,T>, ArenaVector<'s,T>  | (32, 16)   as buffer<T> today, plus one length   |
| PoolVector<'s,T,N>                   | the pair of FixedVector<T,N>, plus (8,8)         |
| FixedVector<T,N>, FixedRing<T,N>     | T's pair repeated N times, plus (8,8) per word   |
| Span, MutSpan, AppendView            | (32, 16)                                         |
| OutOfMemory<T>, PoolExhausted<T>,    | T's own pair                                     |
| NeedCapacity<T>                      |                                                  |
```

Without those rows `K<T>` is undefined for every content type this design admits,
[RES-5]'s bump algebra has no acquire term, and [STOR-6]'s "actual does not exceed
the ceiling" check has nothing to check against.

The runtime's own tables are uniform-slot stores of this list, with their `cap`
published by the profile row [RUN-3] and their `len` composed from the program by
the algebra of 3.3.1. A profile symbol is a standing fact with empty support, so a
store whose `cap` is one satisfies 3.3.1's loop rule exactly as a type-level
constant does.
*Judgment:* the composition of 3.3.1 per domain. *Publishes:* per program point,
per domain, the store's `len` bound. *Amends:* [OP-9] 968-998, whose `buffer_fits`
stays a representability predicate, whose ceiling table gains the rows above, whose
region-bearing exclusion is lifted, and which additionally fixes `K<T>`. *Law:* L6,
L16.

**[RES-6] Typed failure, and the two spellings.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume. The failure family is three
compiler-owned generic **structs**, each with exactly one field:

```text
struct OutOfMemory<T>   { rejected: T; }
struct PoolExhausted<T> { rejected: T; }
struct NeedCapacity<T>  { rejected: T; }
```

`T` is `unit` where nothing is handed back. They are structs and not enums because
L3 requires the failure to *carry* the unconsumed input rather than to choose among
variants, and because `let recovered = move refused.rejected;` is then one
statement instead of a nested `match` on every refusal edge. They are the design's
only compiler-owned nominals with a writer-visible field; [SEQ-0] registers their
field table so [DIAG-1] 1768's deferred-use carrier has one to check.

Each covered-store acquisition comes in exactly two spellings, on the model of `+`
and `+checked`:

```text
pool_take(pool: p, value: v)          requires igt(room(p), Z)        -> own PoolSlot<'s, T>
pool_take_checked(pool: p, value: v)  total                           -> own Result<PoolSlot<'s,T>, PoolExhausted<T>>
arena_take(arena: a, value: v)        requires ige(room(a), K<T>)     -> own ArenaBox<'s, T>
arena_take_checked(arena: a, value: v) total                          -> own Result<ArenaBox<'s,T>, NeedCapacity<T>>
```

The proved form is admitted only when [MSR-4]'s disposition discharges its goal; an
unproved goal is a static rejection with no fallback, exactly as an unproved
subscript is. **The `Heap` has no proved form**: no honest domain predicate exists
for a general store (L6), so every heap acquisition is total and returns `Result`
unconditionally, and its `Err` edge publishes only the returned owner. A store with
measures publishes more: a refused `pool_take_checked` establishes
`ieq(room(pool), Z)` and a refused `arena_take_checked` establishes
`ilt(room(arena), K<T>)`, which is L8's second half and is what makes a checked
acquisition change a loop's summary.

The runtime's handle table is a covered store, so `reserve_file` joins this rule:
its outcome becomes `own Result<FilePermit, NoRecord<unit>>` and its `Err` edge
establishes `ieq(room(factory), Z)`. Round 3's finding is that a covered store
whose exhaustion is indistinguishable from a host refusal has no refusal relation,
so no marked program that opens anything can establish premise 3.

No covered-resource failure is a trap, an abort, a process exit, a retry, or a
promotion to a larger store, in the writer's code or in the runtime.
*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result`, plus [MSR-4]
discharge at the proved spelling. *Publishes:* the returned owner's identity on the
`Err` edge, and the store's own refusal relation where the store has measures.
*Amends:* [TYPE-2] 352 (three failure structs and `NoRecord`); [SYS-2] 2255 and
2451, `reserve_file`'s signature and outcome row; [SYS-7] 2467's closed class set,
which gains no `IoError` class because the exhaustion outcome is a separate type;
the batch 0079 exhaustion floor, whose `wf_resource_abort` site for allocation
refusal loses its last reachable caller once allocation returns a value; and
[SCOPE-3] 29, whose "heap exhaustion ... may stop execution at the host boundary
without a Whitefoot value" ceases to be true. *Law:* L3, L6, L8, L16.

**[RES-7] What bare resource-closedness does not cover, and the one exclusion
test.** Disk space, the successful acquisition of a file, socket or other host
object not exclusively reserved before start, network reachability and throughput,
CPU time, deadlines, scheduler fairness, power, device health, host termination,
and OS quota revocation are outside [RES-1] and outside every judgment in this
file. They remain typed system outcomes where the operation defines one, and
environment conditions where it does not.

Which **operations** a marked program may not call is decided by a property and
never by a written list:

> A system operation is unavailable in a resource-closed program exactly when its
> [QUAL-1] semantic-ID record names a store that is not an item of `E`.

Applied to v0.40, **that set is presently empty**, and the third draft's list of
four excluded operations was written from a premise the specification denies.
[SYS-2] 2264 says "No system operation allocates." [SYS-9] 2525 says `arg_get`
returns "one inline opaque `HostString` lease with no allocation and no byte copy".
[SYS-9] 2543 says neither outcome of `relative_path` "allocates or copies a byte",
and [QUAL-2] establishes the argument backing before entry, which is where
[RUN-4] puts every other item of `E`. So `arg_get`, `relative_path`,
`host_copy_bytes` and `host_copy_utf8` are all available, and the exclusions the
third draft wrote had a second cost it did not see: `arg_get` is the only producer
of a `HostString` and `relative_path` the only producer of a `RelativePath`
([SYS-14] 2626), so excluding them silently deleted the copies and `open_read` from
every marked program, and no marked program could read its command line.

What is genuinely covered, and what the third draft left with no vocabulary, is the
runtime's own handle table. [MSR-1] gives it a measure over the `FileFactory` the
entry holds, [RES-5] gives it the uniform-slot domain, [RUN-3] publishes its `cap`
as a profile symbol, and [RES-6] gives `reserve_file` a typed exhaustion outcome
with a refusal relation. With those four, a marked program that opens files
composes its handle demand exactly as it composes pool demand, and 3.3.1's loop
rule bounds it.
*Judgment:* a call to an operation the test excludes, from a marked program's call
graph, is a hard error citing RES-7 at the `call`. *Publishes:* the boundary.
*Amends:* [ERR-4] 1478, whose "unavailable external resources remain outside the
source outcome model" gains the two families [RES-6] and [STK-5] move inside.
*Depends:* [SYS-2] 2264 and [QUAL-2] 2363, which are why the excluded set is empty
in this version and would not be under a runtime that allocated. *Law:* L1.

**[RES-8] The per-function summary is part of the callable boundary, in two
pieces.** Each function's boundary [FN-1] gains two derived components, and they
are separate because they belong to different stages:

- a **source-stage per-domain map** over that function's formal provider and
  measure terms, substitutable at a call site; and
- a **target-stage own-storage figure** covering every store it reserves [PROV-5]
  and its own frame.

The third draft published one component that was half a source-stage map over
formals and half a target-stage byte count over storage the signature does not
mention. Splitting them is also what keeps [PROV-4]'s framing honest: a
self-reserved store contributes to the second component, which is where the frame
item already lives, so 3.3.1's call rule never meets a callee demand with no actual
to substitute. The map composes across the one closed compilation unit [PROG-1]
and this version claims no more than that; the third draft claimed composition
across units, which [PROG-1] 1486 does not have.
*Judgment:* none; a boundary statement. *Publishes:* both components. *Amends:*
[FN-1] 999-1006's boundary list. *Law:* L1, L5.

#### 3.3.1 How `E` is composed

Every covered resource is one of three kinds, and conflating them is the single
most common way to get a wrong answer (L9).

```text
| kind                 | question                          | examples                              | bound         |
|----------------------|-----------------------------------|---------------------------------------|---------------|
| reusable capacity    | how many are held at once?        | pool slots, task and completion       | peak len      |
|                      |                                   | records, lanes, queue slots, handles  |               |
| consumable budget    | how much is spent and not         | arena cursor bytes, a fixed           | net consumed  |
|                      | returned in this epoch?           | append-only log's records             |               |
| external effect flow | how many times did it happen?     | opens, writes, submissions            | not bounded,  |
|                      |                                   |                                       | not in E      |
```

A statement's summary is **one map from exit label to `(peak, delta)`**. The exit
labels of a statement are its fallthrough, each variant of a result it produces,
each `break` label it may take, and `propagate`. A statement with no fallthrough
label, which after [STK-4] includes a `loop_stmt` no `break` resolves to, simply
carries no fallthrough entry, and the sequence rule below is written so that this
is a defined case and not a hole.

Per resource kind `r`, the primitive transfers are fixed:

```text
acquire one       (peak 1, delta +1)     on the success exit; (0, 0) on a refusal exit
release one       (peak 0, delta -1)
move an owner     (peak 0, delta  0)     moving into a container acquires nothing
borrow an owner   (peak 0, delta  0)
```

A delta may be an integer or an interval `[min, max]`. **An interval enters the
peak equation as its `max` and the delta equation as an interval, and every test
below reads its `max`**; the third draft stated that only in the loop rule, where
the intent shows, and left the sequence rule, where the interval is consumed,
without it. The compositions are:

```text
sequence   when A has a fallthrough exit, for each exit label L of B:
             peak(A;B)[L]  = max( peak(A)[fallthrough], max(delta(A)[fallthrough]) + peak(B)[L] )
             delta(A;B)[L] = delta(A)[fallthrough] + delta(B)[L]     (interval sum)
           for each non-fallthrough exit label L of A, A;B carries A's own (peak, delta)[L]
           when A has no fallthrough exit, A;B is exactly A's map and B contributes nothing

branch     the union of the arms' maps, keyed by exit label; two arms reaching one
           label contribute the componentwise max of peak and, when their deltas
           differ, the interval [min, max] of delta

call       substitute the callee's source-stage map [RES-8] at the call site, with its
           formal measure and provider terms replaced by the actual ones

loop       for the backedge label, let d be the backedge delta:
             max(d) <= 0  -> peak is one iteration's peak; no iteration bound is needed
             max(d) >  0  -> the loop is bounded on a domain exactly when the composed
               peak is a closed expression [RES-3], which it becomes exactly through: a
               trip count that is a compile-time constant; or a store whose cap is a
               standing fact (a type-level constant or a profile symbol) and whose every
               acquisition of that domain on the loop's paths is one that cannot succeed
               when the store is full, which is the checked spelling anywhere on any path
               or a proved spelling whose goal is discharged from a header invariant; or a
               writer [INV-1] invariant over the measure terms. Otherwise there is no
               finite E and premise 3 fails here.
           each exit label of the loop carries the map of the edge that reaches it

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration peak p
                                        and retained d, and K the profile's window
```

The loop rule's second discharge is stated over the **acquisition** and not over
the refusal edge's position in the graph. The third draft required the refusal exit
to be "a real exit with delta 0", which refuses the design's own idiom: 4.1's `Err`
arm rejoins the backedge rather than leaving the loop, and a retaining variant of
it is bounded by `cap` whether the refusal leaves the loop or not, because the
checked spelling cannot succeed on a full store. What the condition needs to say is
that no path can overdraw, which is a property of the acquisition.

Which shapes need a writer annotation and which do not is worked through in
`RESOURCES.md`. The rule here is only that an annotation, where one is needed, is
an ordinary [INV-1] invariant over the measure terms, which are affine atoms by
[MSR-5]. The checker never searches for one: it does not enumerate paths, guess
loop invariants, choose allocator placements, or divide a store between claimants.

#### 3.3.2 Which stage decides what

```text
 1  tail-SCC rewrite, source premise [STK-1]        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-4]    source stage   compiler
 4  per-function source-stage demand map [RES-8]    source stage   compiler
 5  loop and branch composition (3.3.1)             source stage   compiler
 6  concrete sizes, strides, static image           target stage   compiler
 7  the ABI tail obligation [STK-1]                 target stage   compiler
 8  per-context frame envelope [STK-3]              target stage   compiler, post-codegen
 9  runtime profile row for each supported W        target stage   runtime data
10  par composition against the profile             target stage   compiler
11  assembling E and emitting it as an artifact     target stage   compiler
12  selecting W for this run                        PreStart       launcher
13  committing every region and stack item          PreStart       launcher
14  creating lanes and reaching the ready barrier   PreStart       runtime
15  initializing every adapter record and queue     PreStart       runtime
16  crossing SourceStart and invoking main          PreStart -> Running  runtime
```

Steps 1 to 5 decide whether the program is source-resource-closed, and are the
only steps a source rejection may cite. Steps 6 to 11 decide whether this build
qualifies. Steps 12 to 16 decide whether this run is admitted.

### 3.4 `[STK]`: the stack

**[STK-1] A tail edge is one whose caller frame is dead, and the premise is split
across the two stages.** For each strongly connected component of the call graph in
which every intra-component call edge is a tail edge, the compiler rewrites the
component into one dispatcher loop before frames are measured.

The **source premise**, which is what [RES-3] premise 2 reads, is a fact about
ownership and loans and nothing else. An intra-component edge is a source tail edge
exactly when, at that edge: no loan, borrow, view, region or reborrow the caller
introduced is live; no compiler-derived drop remains to run after the call; no
linear binding of the caller is still live [PROV-6]; no `par` join is outstanding;
and no place the caller declared is read by any value live across the call.

The **target obligation**, which is [RES-3] stage two's and which cites no language
rule, is that the selected ABI does not keep the caller's frame live across the
jump for any argument of any rewritten edge.

The split is the repair of a category error. An activation record and a frame are
target-stage objects ([STOR-6] 741, [STK-3]), and the third draft's last clause,
"no place the caller's frame holds is reachable from any argument of the call",
asks the by-value-versus-by-pointer question the settled list puts outside source.
Under it one source program is resource-closed on one ABI and rejected by [STK-2]
on another, with a hard error citing a numbered language rule, which is the defect
[SCOPE-2] and [RES-3]'s own two-stage split exist to prevent.

That one premise still replaces the first draft's five syntactic conditions. Being
written as the complete `expr` of a `return_stmt` is a consequence of the premise,
not a condition beside it. A confined value cannot defeat it, because [PROV-5]
makes a store region local to the reserving function: a live `ArenaBox<'s, T>`
argument implies `'s`'s block is open, which implies the reserving activation is
live, and that activation is not the caller being rewritten unless the caller
introduced `'s`, in which case the first clause already fires.

One cost of the first clause is recorded rather than discovered: a component member
that opens a region for a `pool_frame` or `arena_frame` has a live region at the
jump, so its edge is not a tail edge and [STK-2] refuses the component. Tail
recursion and region-scoped scratch are mutually exclusive, and a writer who needs
both writes the loop.
*Judgment:* per edge, from the ownership and loan state the checker already has;
no proof search. *Publishes:* an acyclic call graph, or a component that is still
cyclic. *Amends:* nothing; this is a lowering and not an admission rule, so
recursion stays permitted. *Verified today:* probes `f2b` and `f8_tailframe` are
mutual tail recursions carrying a live borrow of a caller local and are accepted,
so the premise refuses a shape the syntactic list admitted. *Law:* L7.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After
[STK-1], a program whose call graph still contains a cycle has no finite stack
envelope and is not resource-closed. A `requires` bound on a recursion parameter,
a proof that a recursion argument decreases, and every other depth certificate
are **not** admitted as a substitute.
*Judgment:* under [RES-4], a hard error citing STK-2 that renders the complete
cycle in call order and the restructuring `rewrite the recursion as a loop over
an explicit FixedVector work list, or make every recursive call a tail call whose
caller frame is dead at the jump`. *Publishes:* nothing. *Amends:* nothing.
*Law:* L7.

**[STK-3] The frame envelope, over the whole chain.** For each execution context,
the `stack` item of `E` is measured over the context's **whole chain**, from the
point at which the environment hands that context a stack to the point at which it
takes it back: process entry through `ProgramFinished` for the entry context.
`main`'s own chain is one segment of it, and the runtime's start-up trampoline, its
teardown, its drop glue, and the exhaustion floor's own frames are other segments.
Within one segment,

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

over the acyclic graph of [STK-2], where the possibly-active callees include those
reached on error and propagation edges, compiler-derived drop glue, the target
adapter's helpers, and the ABI save area. `frame(f)` is measured **after final code
generation**, which is why this is a target-stage figure.

Two things the second draft left undefined are settled here. The entry context's
**initial** stack is part of the deployment grant that `Admitted` [RUN-5] covers
and that `PreStart` does not create, so the protocol commits only stacks it did not
receive. And a **worker lane's** chain has no defined root, because a lane executes
whatever the runtime handed it; that question does not arise here, because [RUN-2]
fixes `W = 1` for every resource-closed build, and it is one of the two things the
`par` continuation work must answer before `W > 1` is admitted.

`E` is an **output** of code generation and never an input to it, so the compiler
recomputes `E` after every optimization and publishes the figure the emitted code
needs; two builds of one accepted program may therefore publish different rows.
*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic discipline.
*Publishes:* one `stack(context, bytes)` item per context per profile row.
*Amends:* [STOR-6] 757-761. *Law:* L5, L6.

**[STK-4] A loop with no resolved break has no normal successor.** [FN-1]'s
conservative structural graph gains exactly one sentence, and it replaces one:

> A `loop_stmt` has an edge to `normal_successor(loop_stmt)` **if and only if some
> `break_stmt` resolves to it.**

No second clause. A `return`, a `propagate` `Err` edge, and a `give` delivering
outside the loop are edges to the function-return sink or to an enclosing
construct, and were never edges to the loop's normal successor. The second draft
wrote "no resolved break **and no other exit**", which reintroduced them by hand
and left every driver loop that reports an error outward exactly as refused as
before; probe `n3_propagate_loop` is that program and is `[FN-1]
FunctionFallthrough` today.

This is the rule that lets a kernel's idle loop and a driver's service loop be
entries at all. It creates one silence, and this draft states its disposition
rather than calling it unexploitable: a scope whose exit edge is unreachable
carries no compiler-derived release and no [LIV-1] check, so a **linear** binding
live on a path that reaches only such a loop is not an error, and **is** a retained
item of 3.3.1's map for its store. The leak is then visible in `E` rather than
invisible in the fact state, which is L9's stock-not-flow applied to the one
control shape this rule newly admits.
*Judgment:* [FN-1]'s existing reachability and fallthrough judgment over the
corrected edge set. *Publishes:* the graph, and hence 3.3.1's exit labels.
*Amends:* [FN-1] 1070. *Verified today:* probes `n2_idle` and `f3_forever` are
`[FN-1] FunctionFallthrough`. *Law:* L1, L9.

**[STK-5] Stack exhaustion moves inside the model, for these programs only.** For
a program that is resource-closed on its target, stack exhaustion is not a
deferred external resource condition: [STK-2] and [STK-3] make the maximum chain a
computed item of `E`, and under an admitted run [RUN-5] it is unreachable. For
every other program, [SCOPE-3]'s deferral stands unchanged, and so does the
guard-page floor that reports it, whose own alternate stack is, for a
resource-closed build, an item of `E`.
*Judgment:* none; a scope statement. *Publishes:* the scope. *Amends:* [SCOPE-3]
27-34. *Law:* L1.

### 3.5 `[RUN]`: runtime closure and admission

**[RUN-1] The artifact, and runtime closure as one obligation.** For every
judgment in this file the artifact is the writer's code, the compiler-derived
cleanup and drop glue, the monomorphized instances, the `par` runtime, the
allocator and its metadata, and the qualified target adapter: everything the
process runs between process entry and `ProgramFinished`.

A runtime qualified for resource-closed programs performs, after the `SourceStart`
barrier and until `ProgramFinished`, **no covered acquisition whatsoever**: no
allocator call for runtime-owned storage, no thread or helper creation, no stack,
queue, table or worklist growth, no lazy TLS or adapter initialization, no
first-use mapping, and no first-error formatting buffer. Every runtime record is
established before the barrier or is carved from a fixed store that is already an
item of `E`.

**Acquisition and admission control are different obligations.** A qualified
runtime must additionally have, for every one of its stores, a **bounded admission
discipline** whose bound is that store's published capacity: it declines to start
work for which no record is available and resumes when one is, without acquiring
anything. Lazily chunking a `par` index range is exactly such a discipline.
Saturation of a **program-owned** store is unreachable, because the program's peak
was composed against the published capacity; saturation of the runtime's own
scheduling is admission control and must exist. What stays forbidden is **inline
execution**, which nests a task's chain inside a lane's current activation and
which no term of [STK-3] counts, and **unbounded waiting** on a store no other
frame will release. A runtime that cannot publish a bounded capacity for one of its
stores does not support the marker.

The scope of this obligation is stated plainly, because the third draft answered
every parallel resource question with "cannot occur in a marked build" and left the
build where parallelism actually happens with no answer. This rule is what a
runtime must meet **to support the marker**. For an unmarked build the worker
lane's chain, [PAR-3]'s replicated places, and the stolen-task nesting remain
[SCOPE-3]-deferred exactly as they are today, and this design neither improves nor
worsens them.
*Judgment:* a target-qualification obligation, auditable from the emitted code and
the runtime's own translation units; its failure is a [QUAL-2] qualification
failure, not a source rejection, and no source construct can weaken or waive it.
*Publishes:* the runtime's own items and capacities. *Amends:* [SYS-2] 2264's "no
system operation allocates", which is kept and given its companion: an adapter
record and a handle-table record are runtime-owned stores of [RES-1] with published
capacities. *Law:* L3, L5.

**[RUN-2] `par` enters `E` as a profile, and a marked program takes no `par`
permission.** For each supported lane count `W`, the runtime publishes one finite
profile row: `W` lanes, `W - 1` worker stacks, a task-record capacity `K(W, d)`
where `d` is the program's maximum nested `par` depth, fixed queue capacities, a
fixed completion-record capacity, and the handle-table capacity. The number of
iterations of a `par`-permitted loop never appears in `E`.

**Until a compiler-managed continuation representation exists, an implementation
may not take [PAR-1], [PAR-2] or [PAR-3] permission for a statement or loop of a
program whose entry carries the marker, and the profile row for such a build
publishes `lanes(1)`.** The subject is permission, because v0.40 has no `par`
statement: [PAR-1] 1988 makes overlapping execution a permission over ordinary
statements and never an obligation, and the third draft's "a `par` statement in a
resource-closed program is executed sequentially" named a construct that does not
exist.

This is a rule and not a recommendation, because the alternative is unsound and no
compiler check catches it: the current runtime's wait path executes a stolen task
on the waiting lane's own stack, so `stack(lane_i)` as [STK-3] computes it is wrong
by a factor bounded only by the outstanding-task count, and [RUN-5]'s theorem is
then false on an admitted environment. Two consequences follow for free: [PAR-3]'s
replicated places, which are execution memory no envelope item counts, cannot occur
in a marked build; and [STK-3]'s undefined worker-lane chain does not have to be
defined in this version.
*Judgment:* a fixed-arithmetic composition (3.3.1's `par` rule) against each
profile row for an unmarked program, plus the no-permission rule on a marked one;
the compiler emits no per-`W` clone. *Publishes:* the `lanes` and `slots` items of
each row. *Amends:* the sentence common to [PAR-1] 1989, [PAR-2] 2024 and [PAR-3]
2049, "exhaustion of the execution resources an implementation spends on
overlapping is a resource condition under [SCOPE-3] and is not an observable of
this rule": for a program resource-closed on this target that exhaustion is
unreachable, because no permission is taken. *Law:* L5, L9.

**[RUN-3] The parallel footprint of an allocation is its provider place, and of a
view its origin range.** In [PAR-1]'s written-footprint clause, "the caller region
each `allocates(arena 'r)` entry names after region substitution" is replaced by
"the places each `allocates` path reaches under the [EFF-2] call-boundary
projection", the same projection the rule already applies to `reads` and `writes`.
Two statements that allocate from one provider therefore conflict, and two that
allocate from distinct providers do not. With [PROV-6] the same is true of two
statements that only dispose, because a disposal is a statement with a `writes` row
on each provider it names.

[PAR-2]'s permission for a fill through a `MutSpan` needs two amendments, and the
third draft supplied only the first.

- The **loan** condition. [PAR-2] 2006 requires every place a footprint of `B`
  holds an exclusive loan on to be rooted in a binding `B` introduces, and 2004
  denies on any loan overlapping the resolved root. A `MutSpan` formed **once,
  outside** the loop holds one loan for all iterations, which is not the hazard
  either condition names. The amendment states the condition over
  **iteration-formed** loans: every exclusive loan *formed by a statement of B* is
  rooted in a binding `B` introduces, and a loan formed before `L` on a root every
  footprint of `B` reaches only through the refined single-element ranges of
  1999-2002 does not deny.
- The **write footprint**. [PROV-3] use 1 makes an access through a view one access
  to the range of every resolved origin, so the footprint of `set m[at] = value;`
  contains the origin `target` at range `[a*at+b, a*at+b+1)` rather than at whole
  place. That is what [PAR-2]'s standing condition needs, because the origin is
  rooted in a binding declared outside `L` and nothing else refines it. Without the
  range the third draft's own pinned test could not pass: [PAR-2] denies on a
  whole-root write and denies on an unresolved footprint element alike.

Its element-write form additionally reads "a direct subscript of an array, a prefix
owner, or a `MutSpan`", never a `FixedRing` subscript, whose logical-to-physical
mapping is not affine in the binder.
*Judgment:* the existing [PAR-1] and [PAR-2] permission judgments, with one fewer
special case, one added loan clause, and ranged origins. *Publishes:* permission.
*Amends:* [PAR-1] 1969, [PAR-2] 1994-2028, and [PAR-3] 2029-2058 through their
"forms every footprint exactly as [PAR-1] forms one" clauses. *Depends:* [PAR-2]
1999's single-binder affine element-write refinement, which is the disjointness
argument the range clause composes with. *Law:* L2, L5, L10.

**[RUN-4] The startup protocol.** Program start has four points, and the covered
guarantee spans the last three:

```text
PreStart
    select a row of E from the target's profile table, largest supported W first
    materialize every item of that row the deployment grant did not already supply:
        commit each region (committed backing, not a reserved address range)
        commit each stack other than the entry context's own
        create W-1 lanes and park them at the ready barrier
        establish every queue, task, completion and wait record
        initialize every adapter record, TLS block and runtime table
    a step that fails -> select the next smaller row and start over; when no row
        remains, report StartFailed(item); nothing below happens

SourceStart  (the barrier)
    every item of the selected row is established; no covered acquisition remains
    the runtime enters its closed mode [RUN-1]

Running
    main executes; source and runtime draw only on the selected row

ProgramFinished
    main returns an ExitStatus, or the program is one that does not return [STK-4]
    every compiler-derived release on the return edge has run
    every outstanding par task and completion record has drained
    the runtime's bounded teardown is complete
```

Descending the table is not a retry of a failed acquisition and does not violate
L3: it is the selection step being made with better information, and [PAR-1] 1988
already guarantees `W = 1` is legal for every program. A `PreStart` failure at
`W = 1` is reported as `StartFailed(item)` on an implementation-defined channel
using fixed, preallocated storage; no source statement executes, no owner comes
into existence, no language cleanup runs, and no `ExitStatus` is produced.
*Judgment:* a target obligation, not a source judgment. *Publishes:* the selected
row. *Amends:* [PROG-3] 1499-1509, whose start-time obligation gains the
materialization of `E` and whose `ProgramFinished` boundary is now named. *Law:*
L1, L5.

**[RUN-5] Admission, and the theorem.** `Admitted(H, row)` holds when an
environment `H` has actually established a grant implementing every item of the
selected row before the barrier, the entry context's initial stack included, and,
for the duration of the run, does not revoke it and permits no unmodelled
competitor to consume from it. Then:

```text
source-resource-closed(P)  and  E-materializes(P, T)  and  Admitted(H, row)
--------------------------------------------------------------------------
no covered-resource exhaustion in run(H, T(P))
```

An environment that later revokes the grant, kills the process, or violates the
target profile has falsified `Admitted`; it has not falsified the program's
property. The row a deployment reads is a fact about **one build**: [STK-3] makes
`E` an output of code generation, so a rebuild at another optimization level
publishes its own row and a deployment sizes against the row it was given.
*Judgment:* none by the compiler. *Publishes:* the deployment contract, which is
the selected row. *Amends:* nothing. *Law:* L1.

### 3.6 `[CNT]`: owners, typestate, and confinement

**[CNT-1] The owner inventory.** Exactly five sequence owners, each with a static
backing fixed by its type. Four are prefix owners (L12); the fifth is the rotation
a queue needs and a prefix cannot express.

```text
| type                 | shape  | backing              | provider  | cap        | linear | growth      |
|----------------------|--------|----------------------|-----------|------------|--------|-------------|
| FixedVector<T, N>    | prefix | inline, N slots      | none      | N          | no     | never       |
| HeapVector<'s, T>    | prefix | one heap allocation, | Heap<'s>  | runtime    | yes    | seq_reserve_heap |
|                      |        | none while empty     |           |            |        |             |
| ArenaVector<'s, T>   | prefix | one arena block      | Arena<'s> | runtime    | no     | seq_reserve_arena |
| PoolVector<'s, T, N> | prefix | one pool lease of a  | Pool<'s>  | N, from    | yes    | never       |
|                      |        | FixedVector<T,N> slot|           | the slot   |        |             |
| FixedRing<T, N>      | ring   | inline, N slots      | none      | N          | no     | never       |
```

The `linear` column is [PROV-6]'s classification and follows from the backing:
`HeapVector` and `PoolVector` reclaim per value and are linear; `ArenaVector`'s
block resets with its store region and `FixedVector` and `FixedRing` are
frame-resident, so none of the three is. An owner over a linear element type is
linear whatever its own backing, and `dispose` destroys it either way.

A prefix owner's initialized storage is exactly `[0, len)`. A ring carries one
further piece of typestate, a head offset, and its initialized storage is
`[head, head + len)` modulo `N`; that is still one scalar relation and no per-slot
state, so L12 holds unchanged. A ring's element access is by logical index
`0 <= i < len(ring)`, written as an ordinary subscript, and a ring yields **no
view**, because its initialized region is not contiguous.

A container type is a compiler-owned nominal: no writer-visible field, constructed
only by the [SEQ] operations, no source construction form. An ordinary struct whose
invariants are reproved at every use is refused, because `len <= cap` would then be
a fact with support the writer can kill.
*Judgment:* the ordinary nominal-resolution and construction judgments; a
`construct` naming an owner nominal is a hard error citing CNT-1. *Publishes:* the
five types and their measure rows. *Amends:* [TYPE-2] 352, five added composite
types. *Law:* L4, L12, L13.

**[CNT-2] Container state is typestate, raw slots are unreachable, and stable
identity is data.** Each owner carries `len` and, where it is not a constant,
`cap`; a ring carries a head the writer cannot name. `len(v)`, `cap(v)` and
`room(v)` are [MSR-1]'s terms with [MSR-2]'s facts and [SEQ-0]'s readers. There is
no second definition of a length here.

No [SEQ] operation, no subscript, and no borrow yields a place outside a
container's initialized region. A subscript on an owner or view carries the
ordinary [OP-4] obligation `ilt(index, len(base))`, against `len` and never against
`cap`. There is no uninitialized read to reject, because there is no spelling that
reaches one.

Storage whose **slot identity is stable across removals** is written and needs no
second family: `seq_vacant<T, N>()` constructs `FixedVector<Option<T>, N>` full of
`None`, with `replace v[i] = Some(...)` to occupy a slot and
`replace v[i] = None()` to vacate it. The prefix is full at `N` and never moves, so
no index is renumbered, and the occupancy is the writer's own `Option`
discriminant, which is data and not typestate, so L12 is untouched. Probe `r2_7`
compiles that shape today, including a `len` read that survives an element-position
replace, and probe `w4` confirms the surviving length reaches a subscript.

`seq_vacant` is new in this draft and it exists for a measured reason. The third
draft said the table is "filled once" and left the filling to a `seq_place` loop,
because `seq_filled` requires `T copy` and `Option<T>` is affine. Round 3 showed no
loop can publish the equality: an equality is not an [INV-1] invariant root (probe
`n14`), and the two-sided `ige`/`ile` pair reaches an ordering at the loop exit and
not the equality (probes `n15`, `n19`). One inventory row publishes `len = N`
directly, needs no `copy`, needs no construction loop, and needs no invariant. It
is `buffer_vacant` made frame-resident and non-allocating.
*Judgment:* [OP-4] at every subscript, against `len`. *Publishes:* the typestate.
*Amends:* [OP-4] 909, whose indexable bases extend to the prefix owners, the ring,
`Span` and `MutSpan`, and whose obligation is against `len`. *Law:* L11, L12, L15,
L16.

**[CNT-3] Affine and linear elements, and `array<T, N>` unchanged.** `T` may be
affine in every owner, and may be linear. The initialized region is what makes this
sound: an element enters and leaves only through an operation that moves the
boundary or exchanges two initialized positions, so no slot is read before it is
written or after it is taken. An owner over a linear `T` is itself linear
[PROV-6], and `dispose` walks it.

`array<T, N>` is retained exactly as it is, as the `len = cap = N` case. It keeps
its copy-only element domain, because `array` carries no length separate from `N`,
so every slot is live at once and there is no boundary to make an affine element's
entry and exit unambiguous. A program that needs no length carries no length, and
`tests/programs/fir_filter.wf` is untouched by this design.
*Judgment:* none by itself; the element-type domain of each [SEQ] row.
*Publishes:* the affine-element capability. *Amends:* [TYPE-2]'s flat-element
restriction, by not inheriting it for the owners. *Verified today:*
`array_new<box<u64>, 4>` is [OP-1] `InvalidOperation` (probe `p9`), so this is new
capability. *Law:* L12.

**[CNT-4] Confinement, and the one position closure.** A type is **confined** when
its complete type after substitution names a region. The confinement of a value is
the **set** of regions its complete type names, and it may be moved, returned, or
bound to a destination that **every** member outlives-or-equals [OWN-3]. That
quantifier is the whole rule: a value of type
`Result<PoolSlot<'s, Page>, NeedCapacity<ArenaBox<'q, u64>>>` names two regions,
which [OWN-3] 575 makes incomparable, and fail-closed is the right answer.

A confined value may occupy any position whose owning value's own complete type
names the same region, so that the position is itself confined and [STOR-4] governs
it. That is what admits a store-branded value into a field, a container element and
an enum payload, which is the capability goal A needs, and it is safe because the
store's identity travels in the type into the position and back out of it
[PROV-1].

A **loan-bearing** type [PROV-3] may occupy no position from which a value could
outlive or hide its origin set: no field, no enum payload, no element of any owner,
no content of a branded single value, no generic type argument, and no result
outside [VIEW-6]'s ceiling. This is the narrowing round 3 forced. The third draft
applied that prohibition to every provenance-bearing type, which then included
`box`, `slot` and every provider-backed owner, and it therefore refused, by name,
`Result<HeapVector<T>, OutOfMemory<HeapVector<T>>>` and every other row of its own
inventory, `struct Chunk['p]`, its own flagship example, and `struct Rec { b: box<u64>; }`,
which probe `q1` shows the language accepts today. A loan may not be stored because
a stored loan outlives the exclusivity argument [OWN-5] makes for it; a store brand
may be stored because it is a type parameter and aliases nothing.

**A source nominal may declare region parameters**, written exactly as a function
declares them, and is confined by them:

```wf-design
struct Chunk['s] {
  page: PoolVector<'s, u8, 4096>;
  used: u64;
}
```

and is used as `Chunk<'s>`, an ordinary TYPEID with `targs`, exactly as a generic
function's instance is written. Two instances of one such nominal have the same
type only when their region arguments are identical: region parameters on a nominal
are **invariant**, which is [OWN-12] 645 and [TYPE-5] 372 applied where they
already apply, and which is why this feature needs no variance design.

This is decided on the merits rather than deferred. Forbidding a confined value in
a record field buys **no soundness at all** once the same value is admitted into a
container element, and it forces every kernel structure into parallel columns whose
index correspondence nothing checks, which is the defect L11 exists to remove.
*Judgment:* a loan-bearing type in a prohibited position, or a confined type in a
position whose owner does not name its region, is a hard error citing CNT-4 at the
complete contained `type`, with the restructuring `keep the view as a direct local,
parameter, or result` for the first and `give this nominal a region parameter and
confine the field to it` for the second; and a confined value bound to a
destination some member of its region set does not outlive is a hard error citing
CNT-4 at the binding, rendering every member. *Publishes:* the confinement set.
*Amends:* [STOR-4] 716, whose "may not be returned" becomes the ordinary outlives
relation over the set; [STOR-5] 718-732, whose enumerated position list is replaced
by the intensional split above and whose deferral of per-leaf provenance inside
stored values is **withdrawn as unnecessary** rather than discharged, because a
store brand is a type parameter and needs no per-leaf record; [FN-2] 1087, whose
blanket rejection of a region-bearing generic argument narrows to loan-bearing
arguments and whose "instantiation arguments are always explicit" now covers region
arguments on nominals; and [GRAM-2]'s `struct_decl` and `enum_decl`, which gain
`region_params?` after `generics?`. *Depends:* [OWN-3] 575, whose fail-closed
incomparability is the invariance argument. *Verified today:* probe
`f7_regionresult` is [FN-2] `RegionBearingGenericArgument` and probe `r2_6` is a
[GRAM-2] parse error at `struct Wrap['p]`, so both halves are new. *Law:* L10, L13.

*[CNT-5] is deleted.* It was a per-owner table of scope-exit dispositions, and it
disagreed with [PROV-6] for every owner over a linear element type: it promised a
`FixedVector` a derived element drop that [PROV-6] had abolished, so one type had
two dispositions and both were wrong, one an invisible free of every element and
the other a value no program could let go of. [PROV-6] now states the disposition
once, for every type: a non-linear type keeps its ordinary compiler-derived
release, a linear type has none and leaves by move or by `dispose`, and a
provider's release is [PROV-5]'s store reset. The id is retired and not reused.

**[CNT-6] Acquiring capacity is owner-level and provider-bearing.** Every
operation that may change `cap(v)` takes the owner **by value**, takes the
provider, names its allocation effect, and returns `Result`, handing the untouched
owner back inside the error. There is no capacity-changing operation on a borrow
and none on a view, and there is none that keeps a larger backing on failure: a
fallback is what L3 forbids.
*Judgment:* [SEQ-0] row selection; a growth operation on a view receiver is a hard
error citing SEQ-0 `InvalidReceiver`. *Publishes:* nothing beyond the rows.
*Amends:* nothing. *Law:* L3, L4.

**[CNT-7] A container type never appears behind `&uniq`.** A `param`, `rtype`, or
`let`-bound holder whose mode is `&uniq 'r` and whose direct type is a container
type is a hard error citing CNT-7 at the complete `param`. A shared `&'r` container
parameter remains legal: it can observe measures and read elements and can change
nothing.

Its restructuring branches on the owner, because one string is false for two of the
five: for a prefix owner, `pass a MutSpan or AppendView for element and append
work, or take the owner by value and return it`; for a `FixedRing`, `take the ring
by value and return it, or transform it in place with update`, because a ring
yields no view and the third draft's single string sent a ring writer to one.

This is the rule that retires D1's shape. `&uniq` survives everywhere its
referent's measures are type facts rather than state: a `&uniq` to a struct holding
`array<T, N>` fields, or to a `MutSpan`, is legal because no operation on either can
change a length. It does **not** survive on a `&uniq PoolSlot<'s, Container>`:
[CNT-7] bites on the direct type, `deref(s)` selects the container, and a `replace`
through the holder is D1 one indirection over. That shape is refused, but by
[CALL-5]'s conservative default and not by this rule, and the difference matters
because someone will one day extend [CALL-3]'s class using the wrong sentence as
the criterion.
*Judgment:* the parameter and holder check above. *Publishes:* the absence of a
`&uniq` container transport. *Retires:* the writer-facing `&uniq buffer<T>` and
`&uniq Container` state-borrow forms. *Amends:* nothing beyond retiring those
forms. *Law:* L11.

### 3.7 `[VIEW]`: views, loans, and write-back

**[VIEW-1] The three views.**

```text
| type              | reads             | writes elements   | changes length      | loan on its origins | affine |
|-------------------|-------------------|-------------------|---------------------|---------------------|--------|
| Span<'r, T>       | yes               | no                | no                  | shared              | yes    |
| MutSpan<'r, T>    | yes               | yes               | no, fixed by type   | exclusive           | yes    |
| AppendView<'r, T> | the window it     | the window it     | grows the window    | exclusive           | yes    |
|                   | appended          | appended          | only                |                     |        |
```

Each is an `own` affine value carrying a region `'r`, exactly as `slice<'r, T>`
does today, and each is loan-bearing [PROV-3]. `Span<'r, T>` **is** today's
`slice<'r, T>` renamed; the rename is the whole of the change to it. Its measures
are [MSR-1]'s rows.
*Judgment:* none by itself. *Publishes:* the three types and their loan strengths.
*Amends:* [TYPE-2] 352 (two added view types, `slice` renamed `Span`), [OWN-1] 558
(all three are affine), and [CONST-2] 547-551, [OP-7] 935 and [OP-1]'s `slice_of`
row, which name the retired spellings. *Law:* L10.

**[VIEW-2] Formation, and the loan the view value holds.** A view is formed from a
borrow of the owner:

```text
seq_span['r](vector: &'r v)             -> own Span<'r, T>
seq_mut_span['r](vector: &uniq 'r v)    -> own MutSpan<'r, T>
seq_append_view['r](vector: &uniq 'r v) -> own AppendView<'r, T>
```

and **the view value, not the argument borrow, holds the loan**. For its whole
life, a view value holds a loan of its own strength on the range it reaches of
every place in its resolved origin set [PROV-3]. The loan begins at formation and
ends when the view value is consumed or released. The argument borrow is a
call-scoped temporary, which probes `f2b`, `r1_twouniq` and `w8` confirm by
accepting two of them on one place in one region with an ordinary write between; it
could not be the freeze.

Exclusivity then refuses a second `AppendView` on one owner at its formation, and
the sentence that does that work is [OWN-5] 601's "a write, move, or unique borrow
of an ordinary place conflicts when that place overlaps any such origin", applied
to the second formation's argument borrow.

The rule deliberately admits a *shared* borrow and a direct `let n = len(buf);`
while an exclusive view is live, and this draft states the justification that
actually covers both view kinds rather than the one that covers `AppendView` alone.
For an `AppendView` the reason is publication: it reaches only `[base, cap)` and
publishes nothing until `absorb`, so a concurrent reader of `[0, base)` and a
concurrent reader of `len` both see committed state. For a `MutSpan` the reason is
the range: the shared reader's access is to the origin at the range the *reader*
reaches, and [PROV-3] use 4 judges the two accesses by range overlap, so a shared
read of a `len` word or of a disjoint element does not conflict and a shared read
of an element the `MutSpan` covers does. The third draft asserted the admission and
justified it only for `AppendView`.

Formation publishes:

```text
seq_span         len(s) = <call datum of len(v)>,  cap(s) = <call datum of len(v)>
seq_mut_span     len(m) = <call datum of len(v)>,  cap(m) = <call datum of len(v)>
seq_append_view  len(a) = Z,                       cap(a) = <call datum of room(v)>
```

Each is a two-term relation over that formation call's own datums [MSR-3], which
exist for a borrow operand because the call placement reads a borrow through its
resolved referent. `room(a) = room(v)` follows from [MSR-2]'s identity and is not
separately published.
*Judgment:* [OWN-5] at the formation borrow, and the ordinary [SEQ-0] relation
establishment. *Publishes:* the loan, the three formation relations, and the
carried formation datums [MSR-3]. *Amends:* nothing beyond [PROV-3]'s amendment of
[OWN-5]. *Depends:* [OWN-5] 601, the conflict sentence that refuses a second
exclusive view, and [OWN-6] 609, which makes the argument borrow call-scoped.
*Law:* L10, L14, L15.

**[VIEW-3] The spare window, and `absorb` as the commit.** An `AppendView`'s
`base` is the owner's length at formation and is not a source-visible value.
`len(a)` counts what this view appended and `cap(a)` is the window. Every [SEQ]
operation on an `AppendView` acts on `[base + i]` for `0 <= i < len(a)`;
`seq_truncate` may reduce `len(a)` to zero and no further. A callee that receives an
`AppendView` therefore cannot reduce its caller's `len(v)`.

```wf-design
let written = absorb(view: move a);
```

`absorb` consumes the view, ends its loan, and returns `own u64`. Its judgment, in
this order:

1. the view's **resolved** origin set [PROV-3] must be a singleton, and its member
   must be a resolved place of the current function; a formal-view origin is not
   one, so a callee cannot commit its caller's length behind the caller's back;
2. the result is bound to the commit value `w`, with `w = len(a)` established at it;
3. every fact whose support overlaps `P`'s descriptor storage dies [MSR-2]; and
4. `len(P) = <the view's carried formation datum of len(P)> + w` and
   `cap(P) = <the view's carried formation datum of cap(P)>` are established, and
   `room(P)` follows from [MSR-2]'s identity.

Step 4 names the datum the view has carried since its formation [MSR-3], and the
value is exact because the view held an exclusive loan on `P` from formation to
this consume, so nothing outside the view could change `P`'s measures in between.
The third draft's rule named a pre-transfer datum of a place that is not an operand
of the `absorb` call, which no producer could mint, and its only worked instance
silently substituted the *entry* datum, which differs from the formation datum by
every operation the owner underwent between entry and formation. That substitution
publishes a length one too large for a program that removes an element before
forming the view, and the extra index is a raw slot in `[len, cap)`.

Requiring a singleton **resolved** set is what makes step 1 satisfiable at all:
[FN-1] 1036 includes `immutable-const` in every call-site origin set, so a view that
crossed a call never has a singleton origin set.
*Judgment:* the four steps above; a non-singleton resolved origin, or one that is a
formal-view origin, is a hard error citing VIEW-3 at the operand `atom`, with the
restructuring `return the view to the function that formed it and absorb it there`.
*Publishes:* the commit value and the owner's new measures. *Amends:* [ENT-3.S5]'s
commit-value clause, which gains `absorb`'s. *Depends:* [FN-1] 1036, whose
call-site origin set always contains `immutable-const`, which is why step 1 is over
the resolved set. *Law:* L10, L14, L16.

**[VIEW-4] A view descriptor's length cannot be changed through a borrow.** No
operation takes a `MutSpan` or a `&uniq` to one and produces a different length,
and none changes its owner's length. The ground is stated once, as two properties
of a **borrowed** view descriptor, and both survive [LIV-2]:

- [LIV-2] admits a reinitializing `set` only for a bare binding **declared in the
  current function**. A `&uniq 'b MutSpan<'r, T>` holder in a callee is a borrow of
  a descriptor the caller declared, so no callee can reinitialize it, and
  `deref(handle)` is not a bare binding in any case.
- `replace deref(handle) = ...` is refused by [PROV-3] use 3, because the view's
  origin set is live at every program point of the callee: the view value is not
  consumed there.

Therefore `MutSpan<'r,T>`, `&uniq 'b MutSpan<'r,T>` and `&uniq 'b AppendView<'r,T>`
are all length-fixed for [CALL-3].

The third draft's first ground was "a view is affine, so [SET-1] refuses a `set` of
it", which is the sentence [LIV-2] deletes and [LIV-3] then makes the central
statement of both worked programs. A rule whose premise the same document removes
is not a premise. **This dependency is load-bearing and the register carries it as
a `Depends:` row.**
*Judgment:* none by itself; it is the premise of [CALL-3]. *Publishes:* the
length-fixed class. *Amends:* nothing beyond [PROV-3]'s amendments of [SET-1] and
[SET-2]. *Depends:* [LIV-2]'s bare-binding-declared-in-this-function premise and
[PROV-3] use 3, the two properties above. *Law:* L11.

**[VIEW-5] An abandoned `AppendView` drops what it appended.** Its
compiler-derived release drops the elements of `[base, base + len(a))` in ascending
order, then nothing. The owner's `len` is unchanged, so the abandoned elements are
neither leaked nor double-dropped, and no fact about `len(P)` was ever published.
Not absorbing is a well-defined, safe program that discards work, which is what
makes `absorb` an ordinary operation rather than a must-use obligation. When `T` is
linear the view is linear too, so it has no derived release and abandoning it is
[LIV-1]'s error; the writer disposes it, and the walk visits exactly
`[base, base + len(a))`.
*Judgment:* [LIV-1] at the scope exit. *Publishes:* the release. *Amends:*
[STOR-3]'s release-action table, one added row. *Law:* L13, L14.

**[VIEW-6] Views are never stored, and a view result declares its origin.** A view
is never stored [CNT-4] and never returned except under this rule. [FN-1]'s
slice-result ceiling applies unchanged to each view type: a function whose written
result is `own Span<'r, T>` (respectively `MutSpan`, `AppendView`) has the ceiling
containing `immutable-const` and the formal-view origin of every parameter whose
written mode and type are exactly that same view type with the same formal region
and element type.

The consequence a caller cannot see is made a declaration error rather than a
discovery: **an ordered result list containing two results of the same view type
and the same formal region is a hard error citing VIEW-6 at the `result_binding` of
the second**, with the restructuring `give each result its own formal region`.
Without this a three-output demux written with one region `'o` returns three views
each aliasing all three inputs, and every later judgment about them is conservative
for a reason nothing in the signature shows.

One consequence is recorded because the second draft did not: [FN-1]'s containment
check forbids a helper from manufacturing a view of storage it reaches through a
borrow, [CNT-7] forbids a `&uniq` container parameter, and [VIEW-3] confines
`absorb` to the formation function. Together, `seq_span`, `seq_mut_span` and
`seq_append_view` are usable only in the function that directly owns the container.
Both worked programs satisfy that by forming every view in the owning function and
passing it down, and no helper library over views can exist in this version. Note
what changed beside it: **disposal is not confined this way**, because [PROV-6]'s
walk compares types and not places, so a helper may release what it is handed even
though it may not view what it reaches.
*Judgment:* [FN-1]'s ceiling containment at every `return_stmt`, plus the
same-region result rejection. *Publishes:* the result's origin set. *Amends:*
[FN-1] 1017-1030, by generalizing "slice" to "view" and by adding the same-region
rejection. *Law:* L10, L11.

**[VIEW-7] System operations over views.** The seven range-bearing operations
[SYS-8] take views instead of `buffer<u8>`, and their modes are fixed rather than
left to a table cell:

```text
a destination the operation writes  ->  &uniq 'd MutSpan<'r, u8>
a source the operation reads        ->  &'s Span<'r, u8>
```

so `read_at['f, 'd, 'r](file: &'f ReadFile, destination: &uniq 'd MutSpan<'r, u8>,
file_offset: own u64, start: own u64, end: own u64) -> result: own ReadOutcome`.
Both are borrows of the **descriptor**, so the view survives the call and a
destination can be filled by a loop of reads, which an `own` destination could not.
Both are length-fixed [VIEW-4], so [CALL-3] gives the caller its measures back. The
two obligations keep their form and their order with `len(deref(buffer))` reading
`len(deref(destination))`.

This is the change that lets a heap-free program do I/O, and it is a rule rather
than a register row because it is goal A's container half. It carries two costs
this design does not hide. A destination must be **addressable** before the host
writes into it, so it is built with `seq_filled` and the count the host produced is
an ordinary `u64` beside the container rather than the container's own `len`; the
write-back machinery of [VIEW-3] does not reach the one boundary where lengths
genuinely come from outside, and Q7 records the fix. And every I/O site costs two
writer-visible items rather than one, because the view's region and the call's
borrow region are two regions and [OWN-10] requires each to be opened after its
subject; Q11 records the relief.
*Judgment:* [SYS-8]'s two range obligations, restated over `len` of the borrowed
view. *Publishes:* the endpoint facts [ENT-3.S10] already enumerates, now over a
view. *Amends:* [SYS-8] 2482-2522, [SYS-2] 2158-2302's declaration records and its
normative counts, and the prose of [SYS-9], [SYS-11], [SYS-12] and [SYS-14], which
name `buffer<u8>`. *Law:* L11, L14.

### 3.8 `[LIV]`: liveness, reinitialization, and transformation

**[LIV-1] Liveness is join-checked, and that is what makes release
unconditional.** A binding's live-or-dead status is a property of a program point,
not of a path: at every join of the conservative structural graph [FN-1], and at
every loop head, every predecessor must agree on the status of every binding in
scope. A disagreement is a hard error citing LIV-1 at the join, naming the two
predecessors and the binding. On every edge leaving a scope, a `propagate` error
edge and the function-return edge included, every **linear** binding of that scope
must be dead [PROV-6], because no derived release exists to carry it.

Today the compiler answers the first class with `Semantics/Unsupported:
OwnershipJoin` (probe `f3`) and [OWN-11] avoids it a second way, by forbidding an
outer binding to be moved inside a loop body. Both are avoidance. Once [LIV-2]
admits reinitialization, liveness stops being monotone and the question has to be
answered, because [STOR-3] carries releases on edges and "no release action is
conditional": one static edge with two runtime dispositions is a double free on one
path or a leak on the other, and L12 forbids the runtime discriminant that would
tell them apart.

With LIV-1 in hand, [OWN-11]'s move prohibition is **replaced** rather than lifted:
a binding declared outside a loop body may be moved inside it exactly when the loop
head and every exit edge see it with one status. [OWN-11]'s borrow half is
unchanged: a `borrow_expr` inside a loop body still names only regions introduced
inside that body, which the programs of section 4 satisfy by opening a region
inside the loop, exactly as `docs/patterns.md` P15 prescribes and as probe `k31`
accepts today.
*Judgment:* a per-join and per-scope-exit structural check over the ownership state
the checker already computes; no search. *Publishes:* the unconditional release set
of every edge. *Amends:* [OWN-1] 558 and [OWN-11] 641. *Law:* L17.

**[LIV-2] Reinitializing `set`, and a new declaration event.** `set p = e;` is
additionally admitted when `p` is a bare binding of affine type declared in the
current function, a `let` binding or a parameter, `e` produces exactly `p`'s type,
and **`p` is dead at the commit point**.

```wf-design
set pending = move rest;
```

One premise, stated once: dead at the commit point, whether `p` died before the
statement or inside `e`. The third draft wrote the admission over "whose current
value has already been consumed" and then judged "evaluate `e` under ordinary
rules, **including any consume of `p` inside it**", which are two different
premises, and every `update` in both worked programs is the second shape. The
premise is checkable from the state [SET-1] 496-500 already computes after the
right-hand side, which is where it re-establishes root liveness today.

Its judgment: evaluate `e` under ordinary rules, including any consume of `p`
inside it; every fact whose support contains `p`'s root dies at that consume; then
the binding is reinitialized with `e`'s value, live and usable, with no observable
program point between. It derives no drop and no release, because the target holds
no value at the commit.

**A reinitializing `set` is a declaration event for [ENT-2] term identity.** The
reinitialized binding is a *distinct term* from the consumed one, exactly as
[ENT-2] 2678 already rules for "a fresh binding legally reusing an expired
spelling". Without it a fact stated over the old value reaches the new one, and the
language admits an eight-element measure on an empty container. Its measure images
transfer by [MSR-3], and an [INV-1] affine atom over it is keyed by the term, so a
header invariant is re-established on the backedge from the operation's declared
relation over its call datum.
*Judgment:* the deadness premise plus the ordinary [TYPE-5] exact-type check.
*Publishes:* the new binding's term identity and its measure images. *Amends:*
[ENT-2] 2678's term-identity paragraph (one added declaration event), [OWN-1] 564's
"reinitialization requires a new `let`", [STOR-1] 674 and [SET-1] 482-500, whose
affine-target rejection, dead-root sentence and post-right-hand-side revalidation
together carry the old premise. *Verified today:* probe `p10` is [STOR-1]
`AffineSetTarget` for a live target and probe `w6` is [OWN-1] `UseAfterMove` for a
dead one, the two halves this rule replaces. *Law:* L10, L16, L17.

**[LIV-3] The transformation statement.** One statement form is added, in two
shapes, and it is the only spelling of the receiver-threading shape:

```wf-design
update view by seq_push(value: byte);
update buckets[slot] by seq_clear();
update work by seq_try_take() into taken;
update ring by ring_try_place(value: byte) into shed;
```

`update p by op(args);` is admitted when `op` is a container-domain operation
[SEQ-0] whose first result has the type of `p`, and `p` is a writable owner place.
The `into x` shape is admitted when `op` has exactly two results, the second
binding a fresh IDENT `x`; the plain shape when `op` has exactly one.

**Its judgment is [SET-2]'s, not [SET-1]'s, and that is what makes it not sugar.**
The previous owner is read out of `resolved(p)` into the operation's first declared
parameter, the operation runs, its first result is written back into
`resolved(p)`, and, for the `into` shape, its second result initializes `x`. There
is no writer-observable program point between the read and the write (spec 515),
so there is no partial move, no dead root and no uninitialized hole, and the root
binding stays live (spec 516). A bare binding is the special case of that general
form, not a separate rule.

The third draft defined it as "exactly `set p = op(<first parameter>: move p,
args);` ... carrying that statement's complete judgment, [LIV-2] included", and
then justified its existence by the case that expansion refuses. `move buckets[slot]`
is a partial move, which [OWN-1] 564 makes kill the whole `buckets` binding, so the
write-back lands on a dead root; probe `q7` is that rejection today, at the `set`,
citing [OWN-1]. Worse, the expansion killed `len(buckets)` at the root and no rule
re-established it, so the first `update` at a subscript destroyed the container's
length for the rest of the function. Under the [SET-2] reading nothing consumes the
root, so `len(buckets)` never dies and only `len(buckets[slot])` moves, which is
what [MSR-2]'s support relation says should happen.

Two further consequences are stated rather than left. An `update` at an element
place of a container of **loan-bearing** elements is refused by [PROV-3] use 3, and
[CNT-4] already forbids such a container, so the case is closed twice. And the
`into` shape is what removes the last rebind: every `try` row and every take is
multi-result, which is where L3 and [RES-6] send an honest writer, and the third
draft's own 4.1 carried one `update` against eight two-statement rebinds.

Because it is the only spelling, `set p = op(receiver: move p, ...)` for a
container-domain `op` is a [FORM-1] rejection whose fix is `update p by op(...)`.
It is the only spelling **for that domain**: a user helper that threads its owner
keeps `set buf = collect(out: move buf, source: move line);`, because [SEQ-0] fixes
a receiver-first convention that [FN-1] does not. This design adds no
receiver-position call form: `view.seq_push(value: byte)` would be a second call
syntax whose resolution [GRAM-5] does not have, while `update`'s resolution is a
table lookup.
*Judgment:* row selection, the result-count and type checks, then [SET-2]'s
exchange judgment. *Publishes:* the operation's declared relations, on the
written-back place and on `x`. *Amends:* [SET-2] 508-524, which gains a
compiler-owned exchange whose replacement value is derived from the read-out rather
than written by the writer, and whose target may be linear or region-bearing
because nothing is rebound; [GRAM-4]'s `stmt` production (one added statement
form); and [FORM-2], which renders it as one line `update <place> by <call>;` or
`update <place> by <call> into <IDENT>;`. *Verified today:* probe `r2_8` is a
[FORM-1] parse rejection, so this is new syntax. *Law:* L10.

### 3.9 `[CALL]`: what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the
callee's declared parameter modes and types and its declared contract.** These are
the owner's three call rules of 2026-09-03.

**[CALL-1] Through a shared borrow, every fact survives.** For an argument whose
parameter mode is `&'r`, of any type, container and view included, the call is not
a kill event for any fact supported by the actual's resolved place. Ground:
[OWN-5] admits no write through a shared holder, so [EFF-2] can project no
`writes` occurrence onto that place, so [MSR-2]'s kill does not fire.
*Judgment:* none; the absence of a kill. *Publishes:* the survival of every such
fact. *Amends:* nothing. *Verified today* for `&'a buffer<u8>`: probe `p6` keeps
`len(line) = 10` across the call and the subsequent `line[9_u64]` is accepted.
*Law:* L11.

**[CALL-2] Through a value passed and returned, only the contract's facts exist on
the result.** An `own` argument is a consuming use, so every fact whose support
contains that binding's root dies. The result is a fresh binding carrying exactly
the callee's verified relations, and nothing else. Those relations may name the
consumed parameter's measure, which denotes that call's **call datum** [MSR-3]:
`len(result) = len(view) + 1` means what it reads as, and it is establishable at
the caller precisely because a datum has empty support and the consume the same
statement performs cannot kill it. That is the repair of the finding that this
transport, on which the entire surface rests, published nothing at all: [FN-9]'s
`M(c,q)` requires every referenced formal to substitute to a **live** term, and the
actual of a value-in/value-out row is dead by the time the relation is established.
*Judgment:* the ordinary [ENT-3.S12] establishment, subject to `M(c,q)` as [MSR-3]
amends it. *Publishes:* the callee's declared relations on the result. *Amends:*
nothing beyond [MSR-3]'s. *Verified today:* probe `p1`, `passthrough(out: move a)`
returning the same buffer, then `b[9_u64]`, is **rejected** with residual
`9_u64 < len(b)`; the transport already behaves correctly and what was missing is
the vocabulary to publish across it. *Law:* L11.

**[CALL-3] An element write through a length-fixed view never touches length
facts.** For an argument whose parameter's declared type is `MutSpan<'r, T>` or
`&uniq 'b MutSpan<'r, T>`, which [VIEW-4] fixes a length for, a projected callee
`writes` occurrence kills every fact whose support overlaps the viewed **element
storage** and kills no measure term over that origin. For a parameter of type
`AppendView<'r, T>` or `&uniq 'b AppendView<'r, T>` the same holds, and in
addition the callee can neither decrease the owner's length (L14) nor increase it,
because only `absorb` publishes an increase and [VIEW-3] denies `absorb` to a
callee. For every other parameter type the projected write kills measures as an
ordinary descriptor-storage-overlapping event [MSR-2].
*Judgment:* the kill classification per parameter type. *Publishes:* the surviving
measures. *Amends:* nothing beyond [MSR-2]'s. *Depends:* [VIEW-4], the
length-fixedness this classification reads. *Law:* L11, L14.

**[CALL-4] Contract vocabulary, the ordered result list, and where its relations
land.** [FN-9]'s clause operands are terms [MSR-5], so `len(P)`, `cap(P)` and
`room(P)` over an admitted formal place are operands with no per-family admission;
a parameter's measure denotes its **entry datum** [MSR-3], so a consuming use
inside the body does not take it away. `len(result)`, `cap(result)` and
`room(result)` are operands when the written result type is measured, which today's
result-datum restriction to fragment integers forbids. So the canonical append
contract is writable:

```wf-design
fn collect['o, 's](out: own AppendView<'o, u8>, source: own Span<'s, u8>) -> written: own AppendView<'o, u8> reads(out, source), writes(out) contract {
  requires ile(len(source), room(out));
  ensures ile(len(written), cap(out));
} { ... }
```

The clause is single-state: `out` denotes the entry datum and `written` the result,
with no second state, no `old()`, and no frame rule. Two-state `ensures` is
rejected by the owner and is not proposed anywhere in this design.

A function may declare an ordered result tuple, and **each result binding is a
datum of every clause of that function**, so one clause may name more than one:

```wf-design
fn render['s](block: own PoolVector<'s, u8, 256>, task: own Task) -> (rest: own PoolVector<'s, u8, 256>, written: own u64) ... contract {
  ensures ile(written, len(rest));
}

let (rest, task) = seq_take(vector: move pending);
```

**Those relations reach the caller through one added [ENT-3.S12] destination
clause**, and without it a multi-result contract publishes nothing: 2827 fixes a
closed destination list of four, and a destructuring `let` is none of them. The
clause is the single-binder route quantified over ordinals:

> Each binder of a destructuring `let` is the S12 destination for every published
> relation naming the result at that ordinal, established after the call's ordinary
> transfer, consumes, borrow commits and kills in [ENT-5] 2892-2899's existing
> order, with `M(c,q)` requiring every other referenced support to be live at
> establishment.

It needs no new fact source, and it makes the writer's multi-return and [SEQ-0]'s
own row route the same shape rather than two. The third draft amended [FN-9]'s
admission paragraphs and left the establishment paragraphs untouched, so its own
4.1 asserted a discharge that could not happen.

The result is not a value: there is no tuple type, no tuple place, and no way to
store or pass one. It is a return-and-bind form only, which keeps [CNT-4] and
[TYPE-2] untouched. Multi-return is load-bearing, not a convenience: `seq_take`
must return an owner and an element, and no single value can carry both, since an
enum payload holding a loan-bearing value is refused by [CNT-4]. Three productions
change together: `result_binding` may be one binding or a parenthesized list,
`let_stmt` may bind one IDENT or a parenthesized list, and `return_stmt` may carry
one `expr` or a comma-separated list whose length equals the function's. Every
element is judged independently by the ordinary [FN-1] return rule. Its canonical
rendering is stated rather than left to [FORM-2]'s attachment sets: a result list
renders as `-> (` then its comma-separated bindings then `)`, a destructuring `let`
as `let (` then its binders then `) = `, and a multi-value `return` renders its
expressions comma-separated on one line.
*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set and
the widened result shape. *Publishes:* the clause relations, on every result
ordinal. *Amends:* [FN-9] 1295-1362 (measured results, multi-datum clauses, the
entry-datum operand), [ENT-3.S12] 2827's destination list (one added clause),
[GRAM-2]'s `fn_decl` result shape, [GRAM-4]'s `let_stmt` and `return_stmt`,
[FORM-2] 52-76's rendering, and [FN-1] 999-1013's result shape. *Verified today:*
probe `p4` compiles a single-state `ensures` anchored on `len(deref(destination))`
with a fragment result, probe `p2` shows `len(result)` does not parse, and probes
`p8`/`k09` show the multi-return signature does not parse. *Law:* L10, L11, L16.

**[CALL-5] No transport reads the actual's spelling.** The three transports above
are selected by the callee's declared parameter mode and type and by its declared
contract. No rule of this design consults the argument expression's shape, the
callee's body, its name, or any per-parameter summary derived from its body. A
parameter type for which no transport is selected kills conservatively.

*This is D1 stated as a rule.* The located mechanism of D1, `argument_referent`
returning `element = true` for every `&uniq buffer<T>` actual, is a fact derived
from the actual's shape, and under CALL-5 no such fact exists to be derived. The
precision it was buying is bought instead by the type: a `MutSpan` argument is
element-only **because its type admits nothing else**. Applying CALL-5 to the
residual `&uniq buffer<T>` spelling yields `element = false`, which is exactly the
sweep's minimal sound repair, and is why the D1 conformance case turns XPASS at
[OP-4] in the batch that lands this family. It is also what refuses D1 one
indirection over, through `&uniq PoolSlot<'s, Container>`, which [CNT-7] does not
reach.
*Judgment:* the conservative default for every unselected parameter type.
*Publishes:* the absence of a call-site-derived fact. *Amends:* [ENT-5] 2870's
clause (b), whose projected-callee-write kill is now classified by [CALL-1..3] and
by nothing else. *Law:* L11.

### 3.10 `[SEQ]`: the operation inventory

**[SEQ-0] The container declaration domain.** The container, store and provider
operations are one compiler-owned **generic** declaration domain, built as [SYS-1]
and [SYS-2] build the system domain and admitted to every compilation unit on the
same terms. Each operation is one complete signature record: named parameters in
declared order [GRAM-11], its type, const and region parameters written as
[GRAM-2] orders them, one declared effect row, one declared result mode and type or
one ordered result list, one declared requirement list, and one declared relation
list. **The first declared parameter is the value the operation transforms and
returns; an operation that transforms nothing names its provider first**, which is
`pool_release`'s existing shape and is what [LIV-3] reads.

Six sentences fix everything a table cell would otherwise decide.

**Written arguments.** A row writes its complete type, const and region argument
list in one `targs` list, ordered as [GRAM-2] orders a declaration (type and const
parameters, then region parameters), exactly when some parameter of that list is
supplied by no operand; otherwise it writes none. A partial list is not a spelling
option, because a partial list is the transposition hazard redundant-explicit facts
exist to remove. So `seq_fixed<Task, 32>()`, `seq_vacant<Slot, 64>()` and
`pool_frame<FixedVector<u8, 256>, 8, 'p>()` write their lists, while
`seq_lease(pool: &uniq 'b blocks)`, `seq_span(vector: &'s input)` and
`seq_push(view: move acc, value: byte)` write none: their operands' types supply
every type, const and store-region parameter and the borrow's own written region
supplies the loan region. This is [TYPE-5] 369's own criterion, "no operand can
supply them", applied to a domain rather than enumerated per row.

**The argument form is named, and [GRAM-11] must say so.** A container-domain call
writes its value arguments as a `fieldinit_list` in declared order, exactly as a
user `fn` and a system operation do. [GRAM-11] 341 admits that form for exactly
"a user `fn` or ... an admitted system operation", 343 forces positional operands
for an [OP-1] table operation, and 345 resolves callee kind by "the same partition
that already selects the callee", which [OP-1] 833 states. A container-domain
operation is a fourth class in all three sentences; the third draft registered the
exactly analogous [TYPE-5] enumeration and put [GRAM-11] in its unchanged list,
where "unchanged" was false and every call in the file derived under no admitted
argument form.

**Where the relations come from.** An operation's declared relations are
established on its results exactly as [ENT-3.S12] establishes a verified user
summary, with operand measures denoting that call's datums [MSR-3] and multi-result
rows landing through [CALL-4]'s destination clause. A row with **per-variant**
relations names one *designated outcome result*, and its per-variant relations are
established at entry to the arm of the first `match` whose scrutinee is a bare live
binding of that result, under the same no-kill, no-`set` path discipline
[ENT-3.S7] already uses for a `+checked` arm fact. A row whose single result is a
`Result` or an `Option` designates that result and writes no separate line. Probe
`r2_9` shows that discipline tolerates an intervening statement today; what it does
not tolerate is a statement that consumes the result the relations name, which is
why a program that wants both arms' relations writes the `match` before the rebind.

**Every row is complete over the measures it writes.** A row carrying `writes(P)`
for a measured `P` states the new value of each of `len(P)`, `cap(P)` and `room(P)`,
including the ones it does not change, or it is not a well-formed row (L15). Where
two of the three follow from [MSR-2]'s identity the row states one and the
inventory says so. This is the discipline whose absence made an arena's extent die
at its first allocation.

**One row per operation.** Each operation carries its own requirement and relation
cells, written over its own formals.

**The readers are not in this domain.** `len`, `cap` and `room` are three [OP-1]
table operations taking a bare non-consuming place operand and returning `own u64`,
and they are **`pure`**: the operation reads no state the caller does not already
hold, and [EFF-2] attributes the operand's own read exactly as it does for any
other non-consuming table operand. Probe `r2_10` shows the consequence today: a
`define capacity = len(deref(destination));` over a shared-borrow parameter compiles
in a `pure` function, and declaring `reads(destination)` for it is an [EFF-2]
rejection.

*Judgment:* row resolution by name, receiver type and written arguments; the
per-row requirement discharge under [MSR-4]; and the [GRAM-11] named-argument
check. A diagnostic for an operation cites **[SEQ-0]** and names the operation in
its payload, exactly as an [OP-1] diagnostic cites [OP-1]; [DIAG-1] 1535 admits one
numbered language rule and the inventory rows below are table data, not rules.
*Publishes:* every declared relation of every row. *Amends:* [SYS-1] 2130 (a fourth
admitted declaration source), [SYS-3] 2303 (admitted to every unit), [TYPE-6]
391-403 (the operation spellings enter the lexical IDENT domain, the nominals the
TYPEID domain, and a nominal's region parameters the REGIONID domain), [DIAG-1]
1687-1712 (collision rank 5, a `container_declaration_ordinal` beside the system
one, and the field table [RES-6]'s three failure structs need for 1768's
deferred-use carrier), [ENT-3] 2724 (one added enumerated source S13, plus the arm
route above), [OP-1] 766-845 (`len` gains `cap` and `room`, their domain extends to
owners, views and providers, and `slice_of`, `buffer_new`, `buffer_vacant`,
`box_new` and `arena_new` retire; `ReservedLowerNames` gains `cap` and `room`),
[TYPE-5] 369 (the written-argument criterion above covers a fourth callee class and
`arena_new` leaves the retained-argument enumeration with its retirement),
[GRAM-11] 341-345 (the fourth callee class, in all three sentences), and [FN-2]
1087 (its explicit-argument rule covers this domain). *Law:* L11, L16.

#### The inventory

`V` ranges over the four prefix owners; a row naming a store writes its store
region `'s`. Every row's first parameter is the value it transforms, except a
reservation and a release, which name their provider first. A row's type, const and
region parameters are exactly those its signature names, declared in [GRAM-2]'s
order and elided below where the signature shows them; what a **call** writes is
[SEQ-0]'s written-argument rule. Each provider is written out per row rather than
abbreviated, because a `HeapVector` and an `ArenaVector` have different providers,
different effect rows and different failure types, and one row varying all three by
receiver is the effect polymorphism this design rejects.

**Reservation.** A reservation is not an `allocates` site: `allocates` names a
store a body *draws from* [PROV-4], and a reservation *creates* one whose storage
is an ordinary place of the reserving activation. So these rows are `pure`, which
is what lets 4.1's entry be `pure`.

```text
pool_frame<T, const N: u64>['s]()                      -> own Pool<'s, T, N>            pure
    declares len(result) = Z, cap(result) = N
pool_extent<T, const N: u64>['s]()                     -> own Pool<'s, T, N>            pure
    declares len(result) = Z, cap(result) = N
arena_frame<const BYTES: u64, const ALIGN: u64>['s]()  -> own Arena<'s, BYTES, ALIGN>   pure
    declares len(result) = Z, cap(result) = BYTES
arena_extent<const BYTES: u64, const ALIGN: u64>['s]() -> own Arena<'s, BYTES, ALIGN>   pure
    declares len(result) = Z, cap(result) = BYTES
```

**Construction.**

```text
seq_fixed<T, N>()                  -> own FixedVector<T, N>       pure
    declares len(result) = Z, cap(result) = N
seq_ring<T, N>()                   -> own FixedRing<T, N>         pure
    declares len(result) = Z, cap(result) = N
seq_filled<T, N>(value: own T)     -> own FixedVector<T, N>       pure          T copy
    declares len(result) = N, cap(result) = N
seq_vacant<T, N>()                 -> own FixedVector<Option<T>, N>   pure
    declares len(result) = N, cap(result) = N
seq_heap<T, 's>()                  -> own HeapVector<'s, T>       pure
    declares len(result) = Z, cap(result) = Z
seq_arena<T, 's>()                 -> own ArenaVector<'s, T>      pure
    declares len(result) = Z, cap(result) = Z
seq_heap_filled<T>['s, 'b](heap: &uniq 'b Heap<'s>, count: own u64, value: own T)
    -> own Result<HeapVector<'s, T>, OutOfMemory<unit>>            allocates(heap), writes(heap)   T copy
    requires buffer_fits<T>(count)
    declares Ok(value: r): len(r) = count, cap(r) = count
seq_lease<T, N, K>['s, 'b](pool: &uniq 'b Pool<'s, FixedVector<T, N>, K>)
    -> own Result<PoolVector<'s, T, N>, PoolExhausted<unit>>       allocates(pool), writes(pool)
    declares Ok(value: r): len(r) = Z, cap(r) = N,
                           len(pool) = <call datum of len(pool)> + 1
             Err:          room(pool) = Z
             both:         cap(pool) = <call datum of cap(pool)>
seq_lease_proved<T, N, K>['s, 'b](pool: &uniq 'b Pool<'s, FixedVector<T, N>, K>)
    -> own PoolVector<'s, T, N>                                    allocates(pool), writes(pool)
    requires igt(room(pool), Z)
    declares len(result) = Z, cap(result) = N,
             len(pool) = <call datum of len(pool)> + 1, cap(pool) = <call datum of cap(pool)>
```

**Readers and element access.**

```text
len(p) / cap(p) / room(p)          -> own u64                     pure          [OP-1] rows
p[i]                               element place                                prefix owner, ring, Span, MutSpan
    requires ilt(i, len(p))        [OP-4]
```

**Prefix-owner operations.** Each takes `vector: own V<T>` first.

```text
seq_place(vector, value: own T)            -> own V<T>                        reads(vector), writes(vector)
    requires igt(room(vector), Z)
    declares len(result) = len(vector) + 1, cap(result) = cap(vector)
seq_try_place(vector, value: own T)        -> (rest: own V<T>, unplaced: own Option<T>)   reads(vector), writes(vector)
    designated outcome: unplaced
    declares None: len(rest) = len(vector) + 1, cap(rest) = cap(vector)
             Some: len(rest) = len(vector), cap(rest) = cap(vector), room(rest) = Z
seq_take(vector)                           -> (rest: own V<T>, value: own T)  reads(vector), writes(vector)
    requires igt(len(vector), Z)
    declares len(rest) = len(vector) - 1, cap(rest) = cap(vector)
seq_take_at(vector, index: own u64)        -> (rest: own V<T>, value: own T)  reads(vector), writes(vector)
    requires ilt(index, len(vector))
    declares len(rest) = len(vector) - 1, cap(rest) = cap(vector)
    and the permutation: the element formerly at len(vector) - 1 is at index in rest,
    every other index below len(rest) is unchanged, and no other index moves
seq_try_take(vector)                       -> (rest: own V<T>, value: own Option<T>)      reads(vector), writes(vector)
    designated outcome: value
    declares Some: len(rest) = len(vector) - 1, cap(rest) = cap(vector)
             None: len(rest) = Z, cap(rest) = cap(vector), len(vector) = Z
seq_exchange(vector, first: own u64, second: own u64) -> own V<T>             reads(vector), writes(vector)
    requires ilt(first, len(vector)), ilt(second, len(vector))
    declares len(result) = len(vector), cap(result) = cap(vector)
seq_clear(vector)                          -> own V<T>                        reads(vector), writes(vector)   T non-linear
    declares len(result) = Z, cap(result) = cap(vector)
seq_reserve_heap<T>['s, 'b](vector: own HeapVector<'s, T>, heap: &uniq 'b Heap<'s>, additional: own u64)
    -> own Result<HeapVector<'s, T>, OutOfMemory<HeapVector<'s, T>>>    reads(vector, heap), writes(vector, heap), allocates(heap)
    declares Ok(value: r): len(r) = len(vector), cap(r) = cap(vector) + additional
             Err: the vector returns unchanged in error.rejected
seq_reserve_arena<T>['s, 'b](vector: own ArenaVector<'s, T>, arena: &uniq 'b Arena<'s, BYTES, ALIGN>, additional: own u64)
    -> own Result<ArenaVector<'s, T>, NeedCapacity<ArenaVector<'s, T>>>
                                                               reads(vector, arena), writes(vector, arena), allocates(arena)
    declares Ok(value: r): len(r) = len(vector), cap(r) = cap(vector) + additional,
                           len(arena) <= <call datum of len(arena)> + K<T> * additional,
                           cap(arena) = <call datum of cap(arena)>
             Err: the vector returns unchanged in error.rejected
seq_shrink<T>['s, 'b](vector: own HeapVector<'s, T>, heap: &uniq 'b Heap<'s>)
    -> own Result<HeapVector<'s, T>, OutOfMemory<HeapVector<'s, T>>>    reads(vector, heap), writes(vector, heap), allocates(heap)
    declares Ok(value: r): len(r) = len(vector), cap(r) = len(vector)
             Err: the vector returns unchanged in error.rejected
```

`seq_exchange` carries `reads(vector), writes(vector)` like every other by-value
transformation of an owner. The third draft declared it `pure`, which is an
ill-formed row: [EFF-2] checks a row in both directions, and probe `c8` shows that
a function writing one position of an `own buffer<u8>` parameter and returning it
must exhibit `writes(vector)`.

There is **no release row**. `dispose` [PROV-6] is the one spelling, it is a
statement rather than an operation, and it needs no `requires ieq(len(v), Z)`
because its walk drains what it finds. The third draft's `seq_release_heap` and
`seq_release_pool` retire with that requirement.

**Ring operations.** A ring is a distinct receiver with distinct ends, so it has
its own names rather than sharing a row.

```text
ring_place(ring: own FixedRing<T,N>, value: own T) -> own FixedRing<T,N>      reads(ring), writes(ring)
    requires igt(room(ring), Z)
    declares len(result) = len(ring) + 1, cap(result) = N          appended at the tail
ring_try_place(ring, value: own T) -> (rest: own FixedRing<T,N>, unplaced: own Option<T>)   reads(ring), writes(ring)
    designated outcome: unplaced
    declares None: len(rest) = len(ring) + 1, cap(rest) = N
             Some: len(rest) = len(ring), cap(rest) = N, room(rest) = Z
ring_take(ring: own FixedRing<T,N>) -> (rest: own FixedRing<T,N>, value: own T)             reads(ring), writes(ring)
    requires igt(len(ring), Z)
    declares len(rest) = len(ring) - 1, cap(rest) = N              removed from the head
ring_try_take(ring) -> (rest: own FixedRing<T,N>, value: own Option<T>)                     reads(ring), writes(ring)
    designated outcome: value
    declares Some: len(rest) = len(ring) - 1, cap(rest) = N
             None: len(rest) = Z, cap(rest) = N, len(ring) = Z
```

**View operations.** Each takes its view first.

```text
seq_span['r](vector: &'r V<T>)      -> own Span<'r, T>            reads(vector)     [VIEW-2]
seq_mut_span['r](vector: &uniq 'r V<T>) -> own MutSpan<'r, T>     reads(vector)     [VIEW-2]
seq_append_view['r](vector: &uniq 'r V<T>) -> own AppendView<'r, T>   reads(vector) [VIEW-2]
seq_push(view: own AppendView<'r,T>, value: own T) -> own AppendView<'r,T>    reads(view), writes(view)
    requires igt(room(view), Z)
    declares len(result) = len(view) + 1, room(result) = room(view) - 1, cap(result) = cap(view)
seq_try_push(view, value: own T) -> (rest: own AppendView<'r,T>, unplaced: own Option<T>)   reads(view), writes(view)
    designated outcome: unplaced
    declares None: len(rest) = len(view) + 1, room(rest) = room(view) - 1, cap(rest) = cap(view)
             Some: len(rest) = len(view), room(rest) = Z, cap(rest) = cap(view)
seq_pop(view) -> (rest: own AppendView<'r,T>, value: own T)                  reads(view), writes(view)
    requires igt(len(view), Z)
    declares len(rest) = len(view) - 1, room(rest) = room(view) + 1, cap(rest) = cap(view)
seq_truncate(view, keep: own u64) -> own AppendView<'r,T>                    reads(view), writes(view)   T non-linear
    requires ile(keep, len(view))
    declares len(result) = keep, cap(result) = cap(view)
absorb(view: own AppendView<'r,T>) -> own u64                                reads(view), writes(view)   [VIEW-3]
```

Notes on the inventory.

- **`seq_push` is the operation the whole design exists for.** It is total,
  allocation-free on every backing, and lowers to a store plus a length increment
  with no capacity branch. With `room` readable, [MSR-2]'s identity, and [MSR-3]'s
  datums and images, it is discharged in a loop by a header invariant, by a
  dominating branch on `room`, or by a `requires`; probes `k21`/`k21b` are that
  arithmetic at v0.40 scale.
- **Every `try` row publishes per-arm relations** and names the result they are
  keyed on, which is what [SEQ-0]'s arm route needs.
- **There is no growing `push` anywhere.** Push-with-growth is the shell: reserve,
  form the view, push, absorb (L4).
- **`seq_take_at` publishes its permutation**, because a writer who cannot see
  which element moved cannot use the object table [CNT-3] advertises.
- **`seq_clear` and `seq_truncate` require a non-linear element type**, because a
  linear element must go to a store and these rows name none. The writer drains
  with `seq_try_take` while holding the provider, or disposes the whole container.
- **A `par` fill needs no new type**: `seq_filled`, `seq_mut_span`, and a counted
  loop of `set m[i] = ...;` under [RUN-3]'s amendments, in an unmarked program
  [RUN-2].
- **Nothing in the inventory is total at a capacity boundary.** An overwriting ring
  would need L9's published-displacement relation, and no program here needs it.

### 3.11 The pool seam, resolved

`Pool<'s, T, N>` names `N` interchangeable single-`T` slots, and a `PoolVector`
needs one **contiguous run** of them. A pool that serves *runs* of `k` slots is not
a uniform-slot domain: whether a run of 3 is serviceable is not decided by `len`,
and L6's fragmentation counterexample reappears at slot granularity.

The shape that keeps the algebra is to lease **one slot whose content is the run**:

```wf-design
region 'p {
  let blocks = pool_frame<FixedVector<Record, 256>, 8, 'p>();
  region 'b {
    let leased = seq_lease(pool: &uniq 'b blocks);
    match leased {
      Ok(value: block) => { ... }
      Err(error: refused) => { ... }
    }
  }
}
```

The pool still holds eight interchangeable slots of one type, `room >= 1` still
decides serviceability, and `PoolVector<'p, Record, 256>` is exactly a lease of such
a slot. A `FixedVector` is frame-resident and store-free, so it is a legal slot
content type. Three consequences, all recorded: the capacity is fixed at
reservation, so `PoolVector` carries `N` in its type and `seq_lease` takes no
runtime capacity argument; a program wanting two block sizes reserves two pools in
two nested regions, so `E` names both and their leases have distinct types; and the
reservation writes its complete argument list in one `targs`, with the region last,
because [GRAM-5] 265's `call := callee targs? "(" ... ")"` has no second argument
position and the third draft's `pool_frame<..., 8>['p]()` does not parse.

The inner `region 'b` is not decoration: [OWN-10] requires a borrow of a local to
name a region introduced **inside that binding's own scope**, and `'p` is introduced
before `blocks` exists. Probe `r2_2` is that rejection, probe `r2_1` is the admitted
shape, and [PROV-2] states the general rule so no example can get it wrong.

### 3.12 One name per concept

```text
| concept                    | chosen                | why                                                     |
|----------------------------|-----------------------|---------------------------------------------------------|
| construct an empty owner   | seq_fixed<T,N> etc.   | one prefix names one family; a row is selected by name,  |
|                            |                       | receiver type and written arguments                      |
| construct a filled owner   | seq_filled<T,N>       | what array_new already means                             |
| construct a slot table     | seq_vacant<T,N>       | publishes len = N, which no construction loop can        |
| append one element         | seq_push (view),      | the backing is in the receiver type, not the name        |
|                            | seq_place (owner),    |                                                          |
|                            | ring_place (ring)     |                                                          |
| remove one element         | seq_pop, seq_take,    | a view cannot remove what another view appended (L14);   |
|                            | seq_take_at,          | a ring removes from the head and says so in its name     |
|                            | ring_take             |                                                          |
| commit an append window    | absorb                | one word for the one event that publishes a new length   |
| read-only view             | Span<'r, T>           | the rename is the whole change to slice<'r, T>           |
| the three measures         | len, cap, room        | one quantity, one name, term and reader alike            |
| reserve a store            | pool_frame,           | the placement is in the name, because it decides which   |
|                            | pool_extent,          | item of E the store becomes (L6)                         |
|                            | arena_frame,          |                                                          |
|                            | arena_extent          |                                                          |
| take from a store          | heap_take, arena_take,| one verb for acquisition; the store is in the name and   |
|                            | pool_take, seq_lease  | in the result's type                                     |
| hand content back          | heap_release,         | the inverse of the take; it destroys nothing             |
|                            | pool_release          |                                                          |
| destroy a linear value     | dispose p using (..); | one statement, closed under containment exactly as       |
|                            |                       | linearity is (L13)                                       |
| growth failure             | OutOfMemory<T>,       | L3 requires the failure to hand back the affine input;   |
|                            | PoolExhausted<T>,     | each is a struct with one field, rejected                |
|                            | NeedCapacity<T>       |                                                          |
| transform an owner         | update p by op(...);  | one spelling for the one shape value-in/value-out forces |
| transform and take         | update p by op(...)   | the same statement where the row returns two results     |
|                            |   into x;             |                                                          |
| rebind a consumed binding  | set p = e;            | the premise is deadness; the language gains no second    |
|                            |                       | assignment form                                          |
| the property               | resource-closed       | the long spelling is the one in use                      |
| the failure variant field  | Err(error: e)         | [PRE-1] declares Err(error: E)                           |
```

`Full<T>` and `TooSmall` are **not** in the vocabulary: no row produces either,
because the `try` forms return `Option<T>` instead.

### 3.13 Amendment register

**This register is a collation of the `Amends:` and `Depends:` lines of every rule
in section 3, and it carries nothing else.** It was written last, from the rules.
Four conditions make it checkable rather than remembered, and each is a defect of
this file when it fails:

1. a changed row whose `by` column names no rule whose `Amends:` line reaches it;
2. an `Amends:` line no changed row carries;
3. a `Depends:` line no third-list row carries, or a third-list row no
   `Depends:` line produces; and
4. **a `Depends:` citation whose sentence lies inside a range some `Amends:` line
   changes.** That is the mechanical form of the failure round 2 found twice and
   round 3 found once: a rule whose premise another rule moved. When a dependency
   really does fall inside changed text, it is recorded **on the changed row**,
   which states that the depended sentence survives and who depends on it. Five
   such dependencies exist and each is written into its row: [OWN-5] 601 under
   [VIEW-2], [OWN-7] 624's conservatism under [PROV-3], [FN-1] 1036 under [VIEW-3],
   [PAR-2] 1999 under [RUN-3], and [STOR-3] 694-700 under [PROV-6].

**Changed.** Line numbers are `spec/kernel-spec.md` at a40c7e70, re-derived in this
session; four of the third draft's were wrong.

```text
| rule            | line      | change                                                                    | by                  |
|-----------------|-----------|---------------------------------------------------------------------------|---------------------|
| [SCOPE-3]       | 27-34     | heap exhaustion leaves the deferred set; stack and covered-store          | [RES-6], [STK-5]    |
|                 |           | exhaustion leave it for resource-closed programs                          |                     |
| [FORM-2]        | 52-76     | +4 rendering sentences: the result list, the destructuring let, the        | [CALL-4], [LIV-3],  |
|                 |           | one-line update statement in both shapes, and the dispose statement        | [PROV-6]            |
| [GRAM-2]        | 165-200   | fn_decl admits an ordered result list; program_kind admits resource_closed | [CALL-4], [RES-4],  |
|                 |           | struct_decl and enum_decl gain region_params?                              | [CNT-4]             |
| [GRAM-3]        | 204-207   | the fixed slice, buffer, box and arena type productions retire; the views, | [PROV-1], [VIEW-1]  |
|                 |           | owners and branded singles are ordinary TYPEIDs with targs                 |                     |
| [GRAM-4]        | 214-254   | let_stmt admits a destructuring binder list and return_stmt a comma-       | [CALL-4], [MSR-5],  |
|                 |           | separated list; affine_factor GAINS the [ENT-2] place grammar and the      | [LIV-3], [PROV-6]   |
|                 |           | three measure terms and loses nothing; requires_clause and ensures_clause  |                     |
|                 |           | take a clause_expr; stmt gains update and dispose                          |                     |
| [GRAM-5]        | 265-266   | +2 productions, clause_expr and clause_operand, for the contract surface;  | [MSR-5]             |
|                 |           | atom and atom_list are unchanged, which is why [GRAM-9] needs no scope     |                     |
| [GRAM-11]       | 341-345   | the callee enumeration, the positional-operand sentence and the partition  | [SEQ-0]             |
|                 |           | sentence gain the container declaration domain as a fourth class           |                     |
| [TYPE-2]        | 352       | +16 nominals (3 providers, 3 branded singles, 5 owners, 2 views, 3 failure | [PROV-1], [CNT-1],  |
|                 |           | structs), slice renamed Span, box/arena/buffer retire from the writer      | [CNT-3], [VIEW-1],  |
|                 |           | surface; the flat-element restriction is not inherited by the owners       | [RES-6]             |
| [TYPE-5]        | 369       | a fourth callee class: a container-domain row writes its complete type,    | [SEQ-0]             |
|                 |           | const and region argument list exactly when an operand supplies none;      |                     |
|                 |           | arena_new leaves the retained-argument enumeration with its retirement     |                     |
| [TYPE-6]        | 391-403   | the container-domain spellings enter the lexical IDENT domain, its         | [SEQ-0]             |
|                 |           | nominals the TYPEID domain, and a nominal's region parameters the REGIONID |                     |
| [TYPE-7]        | 471       | the closed deref domain becomes the two borrow modes plus HeapBox,         | [PROV-1]            |
|                 |           | ArenaBox and PoolSlot                                                      |                     |
| [SET-1]         | 482-500   | "no writable target path may traverse a slice value" is restated over      | [PROV-3], [LIV-2]   |
|                 |           | loan strength, which admits the MutSpan element write; the affine-target   |                     |
|                 |           | rejection, the dead-root sentence and the post-right-hand-side             |                     |
|                 |           | revalidation together become the one deadness-at-commit premise            |                     |
| [SET-2]         | 508-524   | the region-bearing target rejection is replaced by [PROV-3] use 3 over     | [PROV-3], [LIV-3]   |
|                 |           | loan-bearing targets only; the exchange gains a compiler-owned form whose  |                     |
|                 |           | replacement is derived from the read-out value                             |                     |
| [CONST-2]       | 547-551   | its naming of buffer, slice and slice_of follows the retirements           | [VIEW-1]            |
| [OWN-1]         | 558-567   | providers, branded values, owners and views are affine; a linear class     | [PROV-6], [VIEW-1], |
|                 |           | joins copy and affine; 564's "reinitialization requires a new let" gains   | [LIV-1], [LIV-2]    |
|                 |           | one route; liveness must agree at every join                               |                     |
| [OWN-4]         | 577       | for a lent-onward child reborrow only, the child's loan ends at the end    | [PROV-7]            |
|                 |           | of its receiving statement and the parent resumes there                    |                     |
| [OWN-5]         | 589-607   | the slice-origin paragraphs generalize to loan-bearing values; each origin | [PROV-3]            |
|                 |           | carries the half-open range its value reaches; the one access clause       |                     |
|                 |           | becomes two, over ranges; a loan covers its place's address computation;   |                     |
|                 |           | the resolved origin set is the set minus immutable-const; 596's no-join    |                     |
|                 |           | sentence is restated over the loan-bearing predicate; 603 gains the        |                     |
|                 |           | callee-side twin of the [SET-1] change. **601's conflict sentence survives |                     |
|                 |           | verbatim and its quantifier widens; [VIEW-2] depends on it**               |                     |
| [OWN-6]         | 611       | a child reborrow may name a caller-supplied region the parent's region     | [PROV-7]            |
|                 |           | outlives-or-equals when the receiving call's result type does not name     |                     |
|                 |           | the loan region                                                            |                     |
| [OWN-7]         | 624       | overlap extends to ranges: two origins with one resolved place overlap     | [PROV-3]            |
|                 |           | exactly when their ranges intersect. **Subscript overlap stays             |                     |
|                 |           | conservative and [PROV-3] use 2 depends on that**                          |                     |
| [OWN-11]        | 641       | the move prohibition is replaced by [LIV-1]'s join agreement; the borrow   | [LIV-1]             |
|                 |           | half is unchanged                                                          |                     |
| [STOR-1]        | 674       | the owners join the storage-class table; buffer<T>'s sentence and the      | [LIV-2]             |
|                 |           | growable-collection paragraph are superseded; the affine-set rejection     |                     |
|                 |           | narrows to a live target                                                   |                     |
| [STOR-2]        | 680       | box_new and arena_new retire; heap_take and arena_take are container-      | [PROV-2]            |
|                 |           | domain rows taking a provider and returning a branded value                |                     |
| [STOR-3]        | 683-715   | a linear type has no compiler-derived release action; the box<T> and       | [PROV-5], [PROV-6], |
|                 |           | buffer<T> drop rows retire with their types; the table gains the store     | [VIEW-5]            |
|                 |           | reset and the AppendView release. **694-700's derived-drop order and its   |                     |
|                 |           | affine-element clause survive and are the walk [PROV-6] reuses**           |                     |
| [STOR-4]        | 716       | confinement becomes the ordinary outlives relation, quantified over every  | [CNT-4]             |
|                 |           | region the value's type names                                              |                     |
| [STOR-5]        | 718-732   | the enumerated position list is replaced by the intensional split of       | [CNT-4]             |
|                 |           | loan-bearing and confined types; the per-leaf-provenance deferral is       |                     |
|                 |           | withdrawn as unnecessary, a store brand being a type parameter             |                     |
| [STOR-6]        | 733-765   | the "no numeric frame ceiling" sentence keeps its scope for the language;  | [RES-3], [STK-3]    |
|                 |           | E-materialization and [STK-1]'s ABI obligation join the target-stage       |                     |
|                 |           | obligations and their failure cites no language rule                       |                     |
| [OP-1]          | 766-845   | +cap and +room rows beside len, whose domain extends to owners, views and  | [PROV-2], [SEQ-0],  |
|                 |           | providers and which stay pure; box_new, arena_new, buffer_new,             | [VIEW-1]            |
|                 |           | buffer_vacant and slice_of retire; ReservedLowerNames +2                   |                     |
| [OP-4]          | 909       | indexable bases extend to the prefix owners, FixedRing, Span and MutSpan;  | [CNT-2]             |
|                 |           | the obligation is against len, never cap                                   |                     |
| [OP-7]          | 935       | slice_of retires; cap and room join the structural operations              | [VIEW-1]            |
| [OP-9]          | 968-998   | buffer_fits stays a representability predicate, the ceiling table gains a  | [RES-5]             |
|                 |           | pair for each added nominal, the region-bearing exclusion is lifted, and   |                     |
|                 |           | the target-independent constant K<T> is fixed here                         |                     |
| [FN-1]          | 999-1070  | the slice-return ceiling generalizes to views and gains the same-region    | [VIEW-6], [CALL-4], |
|                 |           | duplicate-result rejection; the result shape admits an ordered list; the   | [RES-8], [STK-4]    |
|                 |           | boundary publishes a source-stage demand map and a target-stage own-       |                     |
|                 |           | storage figure; a loop_stmt has an edge to its normal successor if and     |                     |
|                 |           | only if some break resolves to it. **1036's call-site origin set still     |                     |
|                 |           | contains immutable-const and [VIEW-3] depends on that**                    |                     |
| [FN-2]          | 1087      | the region-bearing generic-argument rejection narrows to loan-bearing      | [CNT-4], [SEQ-0]    |
|                 |           | arguments; explicit instantiation covers nominal region arguments and the  |                     |
|                 |           | container domain                                                           |                     |
| [FN-3]          | 1117-1121 | effect-row normalization's allocation component becomes the set of         | [PROV-4]            |
|                 |           | allocates paths under 1121's own ordinal identity; the region alpha-map    |                     |
|                 |           | applies to modes and types only                                            |                     |
| [FN-7]          | 1211-1246 | one new input row command.heap; one new entry marker resource_closed; the  | [PROV-1], [RES-4]   |
|                 |           | no-region-parameters sentence admits exactly one, naming the heap store,   |                     |
|                 |           | supplied by program start; main's effect row admits allocates over its own |                     |
|                 |           | labelled provider input; the canonical byte sequence gains the row         |                     |
| [FN-8]          | 1256-1257 | clause operands are a clause_expr over terms [MSR-5], not an expr          | [MSR-5]             |
| [FN-9]          | 1295-1362 | clause operands are terms; a measured result admits len/cap/room; an       | [MSR-3], [MSR-4],   |
|                 |           | ordered result list gives one clause more than one result datum; a         | [MSR-5], [CALL-4]   |
|                 |           | parameter's measure operand denotes an entry datum, so 1310 and 1320-1322  |                     |
|                 |           | are replaced by the datum; 1345's M(c,q) admits a measure datum, which is  |                     |
|                 |           | always live; the direct-affine route is one step of [MSR-4]                |                     |
| [EFF-1]         | 1363-1372 | allocates takes formal-rooted effect paths; the atoms heap and arena       | [PROV-4]            |
|                 |           | retire                                                                     |                     |
| [EFF-2]         | 1400-1404 | "slice parameter names the backing" generalizes to a loan-bearing          | [PROV-3]            |
|                 |           | parameter. 1421's empty-release-row sentence is UNCHANGED and stays true,  |                     |
|                 |           | because after [PROV-6] no reclamation of store-owned storage is derived    |                     |
| [ERR-4]         | 1478      | "unavailable external resources remain outside the source outcome model"   | [RES-7]             |
|                 |           | gains the two families that move inside                                    |                     |
| [PROG-3]        | 1499-1509 | the start-time obligation includes materializing the selected row of E;    | [RUN-4]             |
|                 |           | ProgramFinished is named; PreStart may descend the profile table and does  |                     |
|                 |           | not commit the entry stack it received                                     |                     |
| [DIAG-1]        | 1687-1712 | collision rank 5 covers the container domain; a                            | [SEQ-0]             |
|                 |           | container_declaration_ordinal joins the system one; 1768's deferred-use    |                     |
|                 |           | carrier gains the failure structs' field table                             |                     |
| [PAR-1]         | 1969,1989 | the allocates(arena 'r) region clause becomes the ordinary provider-place  | [RUN-3], [RUN-2],   |
|                 |           | projection, and a dispose statement enters a footprint through its writes  | [PROV-6]            |
|                 |           | row; execution-resource exhaustion is unreachable for a marked program     |                     |
| [PAR-2]         | 1994-2028 | the exclusive-loan condition is stated over loans formed by a statement of | [RUN-3], [RUN-2]    |
|                 |           | the body; a view's write footprint is its origin at the range it reaches;  |                     |
|                 |           | the element-write form reads "a direct subscript of an array, a prefix     |                     |
|                 |           | owner, or a MutSpan"; the exhaustion sentence as above. **1999's           |                     |
|                 |           | single-binder refinement survives and [RUN-3] depends on it**              |                     |
| [PAR-3]         | 2029-2058 | the exhaustion sentence as above; its replicated places cannot occur in a  | [RUN-3], [RUN-2]    |
|                 |           | marked program, which takes no permission                                  |                     |
| [SYS-1]         | 2130      | a fourth admitted declaration source, on [SYS-1]'s own terms               | [SEQ-0]             |
| [SYS-2]         | 2158-2302 | the range-bearing operations' buffer parameters become MutSpan or Span,    | [VIEW-7], [RUN-1],  |
|                 |           | changing the inventory's normative counts; reserve_file returns a Result   | [RES-6]             |
|                 |           | with a typed exhaustion outcome. **2264's "no system operation allocates"  |                     |
|                 |           | is kept and gains its companion, and [RES-7]'s exclusion test depends on   |                     |
|                 |           | it: because it is true, that test excludes nothing in this version**       |                     |
| [SYS-3]         | 2303      | the container domain is admitted to every compilation unit                 | [SEQ-0]             |
| [SYS-7]         | 2467-2481 | the IoError class set is unchanged; handle-table exhaustion is a separate  | [RES-6]             |
|                 |           | outcome type rather than a new class, so no portable class is added        |                     |
| [SYS-8]         | 2482-2522 | read_at, write_once, directory_next, host_copy_bytes, host_copy_utf8,      | [VIEW-7]            |
|                 |           | open_directory and open_file take &uniq 'd MutSpan<'r,u8> for a            |                     |
|                 |           | destination and &'s Span<'r,u8> for a source; the two range obligations    |                     |
|                 |           | keep their form and order with len of the borrowed view                    |                     |
| [SYS-9,11,12,14]| 2523-2641 | their normative prose naming buffer<u8> is restated over views             | [VIEW-7]            |
| [ENT-2]         | 2675,2678,| the three measure terms are one-place terms over an admitted place that    | [MSR-1], [MSR-3],   |
|                 | 2722      | may end in a subscript; the term list gains the measure datum beside the   | [LIV-2], [MSR-2]    |
|                 |           | capture and commit-value clauses; a reinitializing set is a declaration    |                     |
|                 |           | event, so the reinitialized binding is a distinct term; the implicit-fact  |                     |
|                 |           | sentence gains the four standing facts                                     |                     |
| [ENT-3]         | 2724,     | +1 enumerated source S13, the declared relations of a container-domain     | [SEQ-0], [VIEW-3],  |
|                 | 2827      | operation, established as S12 is, with a per-variant arm route on the S7   | [CALL-4]            |
|                 |           | path discipline; S5 gains absorb's commit value; S12's destination list    |                     |
|                 |           | gains each binder of a destructuring let at its own ordinal                |                     |
| [ENT-5]         | 2857-2899 | a measure's support is its descriptor storage, its holders and every       | [MSR-2], [MSR-3],   |
|                 |           | offset's support, and a kill is any event carrying a writes occurrence     | [CALL-5]            |
|                 |           | that projects onto it; the call-boundary paragraph and the entry-image     |                     |
|                 |           | stability paragraph are replaced by the measure datum; clause (b)'s        |                     |
|                 |           | projected-callee-write kill is classified by [CALL-1..3] and nothing else  |                     |
| [ENT-6]         | 2970-3092 | one numeric goal disposition replaces the four per-family route and        | [MSR-3], [MSR-4],   |
|                 |           | attach-site grants; measures carry affine value images, and an image dies  | [MSR-2]             |
|                 |           | exactly where a fact over the same term dies; 3001's automatic affine-     |                     |
|                 |           | premise sequence gains len + room = cap as two specification-fixed members |                     |
| [INV-1]         | 3107      | affine atoms are the [ENT-2] place grammar, the measure terms, and named   | [MSR-5]             |
|                 |           | consts, which 3107 forbids today                                           |                     |
| batch 0079      | docs/done/| the heap-refusal abort site loses its last reachable caller; the           | [RES-6]             |
| exhaustion floor| 0079-...  | guard-page record survives, and for a resource-closed build its alternate  |                     |
|                 |           | stack is an item of E [STK-5]                                              |                     |
```

**Depended on and unchanged.** Each row is the collation of one or more `Depends:`
lines, and each names the rule that depends on it. A later batch changing one of
these sentences changes a rule of this design without touching it.

```text
| rule       | line | the sentence, and who depends on it                                       |
|------------|------|---------------------------------------------------------------------------|
| [CONST-2]  | 541  | named const storage is permanently read-only, so immutable-const creates   |
|            |      | no conflicting access: [PROV-3]'s resolved-set definition                  |
| [OWN-3]    | 573  | region identifiers are unique within a function: [PROV-1], which is why a  |
|            |      | store region's spelling denotes one store                                  |
| [OWN-3]    | 575  | distinct caller-supplied regions are incomparable and every ordering rule  |
|            |      | fails closed: [PROV-1] and [CNT-4], the whole invariance argument          |
| [OWN-6]    | 609  | a borrow not bound by let is a call-scoped temporary: [PROV-2] and         |
|            |      | [VIEW-2], which is why the argument borrow is not the freeze               |
| [OWN-9]    | 633  | one usable mutable path per place: [PROV-3] use 1, whose ranged access is  |
|            |      | the form that claim now takes over a view                                  |
| [OWN-10]   | 636  | a borrow of a local names a region introduced inside that binding's scope: |
|            |      | [PROV-2], which is why a store region is never a loan region               |
| [OWN-12]   | 645  | region substitution controls type equality: [PROV-1], which is why two     |
|            |      | stores are distinguished by their types                                    |
| [TYPE-5]   | 374  | argument types match declared parameter types exactly: [PROV-1], the other |
|            |      | half of the invariance argument                                            |
| [CAP-1]    | 1962 | own, &, &uniq and place overlap are the complete interference vocabulary:  |
|            |      | [PROV-2], which adds no capability category                                |
| [FN-6]     | 1205 | recursion is permitted: [STK-2], which excludes a program from [RES-4]     |
|            |      | rather than rejecting it                                                   |
| [FN-8]     | 1269 | a borrow formal uses its resolved referent and an own actual its value     |
|            |      | before transfer: [MSR-3]'s call placement, which reuses that split         |
| [PROG-1]   | 1486 | one closed compilation unit with no function values: [PROV-4]'s exact      |
|            |      | reachability closure and [RES-8]'s composition claim                       |
| [ENT-2]    | 2687 | one static term per statement, because forward flow visits each statement  |
|            |      | once: [MSR-3]'s per-program-point datum inside a loop                      |
| [ENT-4]    | 2854 | L0's uniqueness and finiteness rests on the difference-bound shape:        |
|            |      | [MSR-2], which is why len + room = cap is an affine premise and not an L0  |
|            |      | fact                                                                       |
| [QUAL-2]   | 2363 | a target supplies every guarantee a semantic ID requires before the        |
|            |      | operation is admitted: [RES-7], whose exclusion test reads that record     |
```

**META-5 delta**, declared here because the register is its natural home. Numbered
language rules: 131 today, plus the 52 of section 3, none reusing a live or retired
id. Unique fixed lowercase grammar atoms: minus 5 for the retired `heap` and
`arena` effect atoms and the retired `slice`, `buffer` and `box` type productions
(`arena` is one atom serving both a production and an effect entry, and retires
once), plus 6 for `resource_closed`, `update`, `by`, `into`, `dispose` and `using`;
net plus 1. Grammar productions: plus 4, being `clause_expr`, `clause_operand`, the
`update_stmt` and the `dispose_stmt`; changed, 9, being `let_stmt`, `return_stmt`,
`result_binding`, `program_kind`, `struct_decl`, `enum_decl`, `effect`,
`affine_factor`, and the two clause productions counted once as the pair
`requires_clause`/`ensures_clause`. `ReservedLowerNames`: plus 2, `cap` and `room`.
Nominal types: plus 16, being 3 providers, 3 branded singles, 5 owners, 2 views and
3 failure structs, and one renamed, `slice` to `Span`. Declaration domains: plus 1,
with one `container_declaration_ordinal`. Entry input rows: plus 1. [SYS-2]'s
normative inventory counts change with [VIEW-7] and [RES-6] and are recomputed when
those rules are written into the spec, not asserted here.

**Retired outright, with no successor.** The writer-facing `&uniq buffer<T>` and
`&uniq Container` state-borrow forms ([CNT-7]); `buffer_vacant`'s `Option`-element
construction, which `seq_vacant` serves frame-resident; the effect-row atoms `heap`
and `arena` ([PROV-4]); `slice_of` in favour of `seq_span`; `box_new` and
`arena_new` in favour of `heap_take` and `arena_take`; the third draft's
`seq_release_heap` and `seq_release_pool` in favour of `dispose`; the first draft's
`Builder<'r, T>` type and its `[BLD]` family; the second draft's `[STK-4]`
reentrancy premise, which had no expressible instance; and `[CNT-5]`, whose content
[PROV-6] states in one sentence.

**Writer doctrine this design invalidates**, which `docs/patterns.md` must carry in
the same batch. **P16** ("One length fact above the writes") states that the
compiler honours the element-write exception across a callee boundary through a
`&uniq buffer` actual; [CALL-5] makes that kill conservative for exactly that
spelling, so P16's shape is invalid from B1 until B6 restores it over `MutSpan`,
where it is sound by type. P16 gains a second correction from [MSR-2]: a length
fact survives a write to a **sibling field** of the same record, which is precisely
what probe `r2_4` shows today's compiler killing. **P17**'s advice to fold a
returned record field by field remains right, and its `replace` note gains
[LIV-2]'s dead-target `set` and [LIV-3]'s `update` as the third and fourth commit
forms. **P19**'s join rule is unchanged and gains a case: a measure term joins by
the same delta-atom rule. **P15** is unchanged and is the pattern both worked
programs follow. **P8** should gain the sentence probe `q5'` bought: an exact `-`
carries an ordering into a backedge where `-wrap` gives the checker a fresh atom.
Two new patterns are owed, and neither is optional: one for structural disposal,
because [PROV-6] changes the shape of every hosted helper that takes ownership of
store-backed storage, and one for the `propagate`-free allocating helper, because
[PROV-6] refuses a shape `growable_vec.wf` and `byte_string.wf` both use today.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. The
standard they are held to is that every statement is accepted by a compiler
implementing section 3's rules **and the unchanged v0.40 rules**, and both were
walked statement by statement against both before this draft was finished. Round 3
walked the third draft's pair and found six classes of refusal: a `requires` whose
operands are calls, a `set` of a live target, an `absorb` naming a datum no
producer mints, a multi-return whose relations reach no destination, a reserving
row with no declared relations, and a disposal whose provenance nothing preserved.
All six are fixed here, and each fix is named where it appears.

Byte figures are symbolic. No implementation computed any of them, and where a
figure depends on code generation the table says so instead of inventing a number.

### 4.1 A cooperative run queue with the heap absent

A fixed run queue of tasks, a 256-byte transmit ring, and an eight-block pool with
typed exhaustion. Each task is a state machine that advances one step per turn and
re-queues itself while it wants another. No heap, no recursion, an acyclic call
graph, and a queue loop whose resource state is restored on every backedge. It is
**not** a context-switching scheduler, and section 1.4 says why.

```wf-design
struct Task {
  state: u32;
  arg: u64;
}

fn advance(task: own Task) -> next: own Option<Task> reads(task.state, task.arg) {
  doc "Advances one state machine and returns it again while it wants another turn.";
  let step = task.state +wrap 1_u32;
  let more = ilt(step, 3_u32);
  if more {
    let ready = Task(state: step, arg: task.arg);
    return Some<Task>(value: move ready);
  }
  return None<Task>();
}

fn render['s](block: own PoolVector<'s, u8, 256>, task: own Task) -> (rest: own PoolVector<'s, u8, 256>, back: own Task, written: own u64) reads(block, task.state), writes(block) contract {
  requires ige(room(block), 8_u64);
  ensures ile(written, len(rest));
} {
  doc "Writes one eight-byte record for a task into the block and reports the count.";
  let narrowed = cvt<u32, u8>(task.state);
  let mark = 63_u8;
  match narrowed {
    Ok(value: byte) => {
      set mark = byte;
    }
    Err(error: narrowing) => {
    }
  }
  let total = 0_u64;
  region 'f {
    let view = seq_append_view(vector: &uniq 'f block);
    for @fill (
      at in 0_u64..8_u64,
      invariant spare: ige(room(view) + at, 8_u64)
    ) {
      update view by seq_push(value: mark);
    }
    set total = absorb(view: move view);
  }
  return move block, move task, total;
}

fn drain['s, 'b](ring: own FixedRing<u8, 256>, block: &'b PoolVector<'s, u8, 256>, count: own u64) -> (rest: own FixedRing<u8, 256>, sent: own u64) reads(ring, block), writes(ring) contract {
  requires ile(count, len(deref(block)));
} {
  doc "Copies one prefix of the block into the transmit ring and reports how many bytes it placed.";
  let placed = 0_u64;
  for @copy (at in 0_u64..count) {
    let byte = deref(block)[at];
    update ring by ring_try_place(value: byte) into unplaced;
    match unplaced {
      None() => {
        set placed = placed +wrap 1_u64;
      }
      Some(value: dropped) => {
      }
    }
  }
  return move ring, placed;
}

resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Runs a cooperative queue of state machines over a leased block pool and a transmit ring.";
  let ring = seq_ring<u8, 256>();
  let pending = seq_fixed<Task, 32>();
  let first = Task(state: 0_u32, arg: 65_u64);
  update pending by seq_try_place(value: move first) into unplaced;
  match unplaced {
    None() => {
    }
    Some(value: rejected) => {
      return exit_status(code: 1_u8);
    }
  }
  region 'p {
    let blocks = pool_frame<FixedVector<u8, 256>, 8, 'p>();
    loop @queue {
      update pending by seq_try_take() into next;
      match next {
        None() => {
          break @queue;
        }
        Some(value: task) => {
          region 'b {
            let leased = seq_lease(pool: &uniq 'b blocks);
            match leased {
              Ok(value: block) => {
                let (filled, back, written) = render<'p>(block: move block, task: move task);
                region 'd {
                  let (fed, sent) = drain<'p, 'd>(ring: move ring, block: &'d filled, count: written);
                  set ring = move fed;
                }
                dispose filled using (blocks);
                let stepped = advance(task: move back);
                match stepped {
                  None() => {
                  }
                  Some(value: again) => {
                    update pending by seq_try_place(value: move again) into refused;
                    match refused {
                      None() => {
                      }
                      Some(value: lost) => {
                      }
                    }
                  }
                }
              }
              Err(error: exhausted) => {
                update ring by ring_try_place(value: 33_u8) into shed;
                match shed {
                  None() => {
                  }
                  Some(value: dropped) => {
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
```

#### The envelope the compiler publishes

```text
E(queue.wf, <embedded target>) row W = 1

  region  static.image        bytes  <target>        align  <target>  contiguous
  stack   entry               bytes  <post-codegen>  align  <ABI>     contiguous
  lanes                       count  1
  slots   task.records        count  0
  slots   completion.records  count  0
  slots   handle.table        count  0
```

```text
| item                | where it comes from                                                | rule            |
|---------------------|--------------------------------------------------------------------|-----------------|
| static.image        | the const items and the static parts of the emitted module         | [STOR-6]        |
| stack.entry         | main's frame, holding the ring (256 element bytes plus a head and  | [STK-3],        |
|                     | a length word), the FixedVector<Task, 32> (32 strides plus a       | [PROV-5]        |
|                     | length word) and the one pool_frame occurrence's extent            |                 |
|                     | (8 * (256 + one length word), plus the pool's own free-list word); |                 |
|                     | plus render, drain and advance, plus the runtime frames beneath    |                 |
|                     | main and its bounded teardown; measured post-codegen over the      |                 |
|                     | whole chain                                                        |                 |
| lanes = 1           | no permission is taken; [RUN-2] fixes W = 1 for a marked build     | [RUN-2]         |
| every slots row = 0 | no par permission, no may-suspend operation, no system handle      | [RUN-2],        |
|                     |                                                                    | [RES-5]         |
```

The layout arithmetic is `CONTAINERS.md` G7's, which is why the composition is
written and no total is. The pool is a frame item because the program wrote
`pool_frame`; `pool_extent` would have produced its own `region` item instead.

#### Why it is source-resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | main declares pure, selects no command.heap, and every reserving row is      |
|                       | pure [SEQ-0], so [PROV-4]'s closure is empty and [RES-4] does not fire       |
| acyclic call graph    | main -> {render, drain, advance, the container domain}; render and drain     |
|                       | -> the container domain. No cycle, so [STK-1] rewrites nothing and [STK-2]   |
|                       | passes                                                                      |
| pool demand bounded   | the lease and its disposal are on the same path, so the Ok arm's map is      |
|                       | (peak 1, delta 0) on the pool and the Err exit is (0, 0). The queue loop's   |
|                       | backedge delta is 0, so 3.3.1's loop rule needs no iteration bound and       |
|                       | len(blocks) <= 1 throughout                                                 |
| queue and ring        | FixedVector<Task, 32> and FixedRing<u8, 256> are frame placement, whose      |
|                       | [RES-5] row is decided at compile time and contributes no demand at all;     |
|                       | their capacities are type-level constants                                   |
| L9's displacement     | ring_try_place refuses at capacity and returns the byte, and drain reports   |
|                       | what it placed, so nothing is displaced silently                            |
| stack bounded         | one context, one chain, measured after code generation [STK-3]              |
| runtime closed        | W = 1, no task or completion records; every runtime store's peak is zero     |
```

#### The writer's-eye walkthrough

`let blocks = pool_frame<FixedVector<u8, 256>, 8, 'p>();` writes its complete
argument list because no operand supplies any of it [SEQ-0], with the region last
in [GRAM-2]'s order and inside the one `targs` list [GRAM-5] admits. `'p` is a
region this function opens [PROV-5] and is named by no second reserving occurrence
[PROV-1], so from here on it is this store's name: `blocks` has type
`Pool<'p, FixedVector<u8, 256>, 8>` and every lease `PoolVector<'p, u8, 256>`.

`update pending by seq_try_place(value: move first) into unplaced;` is [LIV-3] in
its two-result shape, and it is one [SET-2] exchange: `pending` is read out,
supplied as the row's first parameter, `rest` is written back, and `unplaced` binds
the second. Nothing consumes `pending`'s root, so `len(pending)` never dies and the
`match` may follow the statement rather than having to precede a rebind. The third
draft wrote this as two statements and carried eight such pairs against one
`update`.

`region 'b { let leased = seq_lease(pool: &uniq 'b blocks); ... }` opens a region
**after** the binding it borrows [OWN-10] and inside the loop body [OWN-11]; probes
`r2_2` and `r2_1` are the two halves. The call writes no arguments: `blocks`'s type
supplies the element type, both constants and the store region, and the borrow
supplies the loan region.

`let (filled, back, written) = render<'p>(...)` is **[CALL-4]**'s ordered result
list, and its `ensures ile(written, len(rest))` names two results, which [CALL-4]
admits and [FN-9] alone does not. It reaches this caller through [CALL-4]'s added
[ENT-3.S12] destination clause, each binder receiving the relations that name its
own ordinal; without that clause the third draft's identical line published
nothing, and `drain`'s `requires` was undischarged. Inside `render`:

```wf-design
    for @fill (
      at in 0_u64..8_u64,
      invariant spare: ige(room(view) + at, 8_u64)
    ) {
      update view by seq_push(value: mark);
    }
```

The invariant's **base** holds because [VIEW-2] publishes `len(view) = Z` and
`cap(view) = <call datum of room(block)>` at the formation, [MSR-2]'s identity
gives `room(view) = cap(view) - len(view)`, and the `requires` gives
`room(block) >= 8` at an entry no event separates from the formation.

The invariant's **backedge** is the derivation the whole container surface rests
on, and it is written out here because no earlier draft wrote it. Each `update` is
a [SET-2] exchange whose replacement is `seq_push`'s result, so [LIV-2] makes the
written-back `view` a distinct [ENT-2] term and [MSR-3] retires the old affine atom
and introduces a new one. The new atom is not orphaned: `seq_push` declares
`room(result) = room(view) - 1` over that call's own datum, the datum has empty
support and therefore survives the exchange's kill, and `at` grows by exactly one
on the same edge, so `room(view) + at` is preserved. Three steps, once per
iteration, per invariant.

The **consumer**, `seq_push`'s `igt(room(view), Z)`, follows from the header target
and S11's `at < 8` by [MSR-4]'s unordered-pair family. Probes `k21` and `k21b` are
that arithmetic at v0.40 scale, accepted and then rejected at [FN-8] when the
invariant is deleted.

`set total = absorb(view: move view);` is the commit. `view`'s resolved origin set
is the singleton `{block}`, a resolved place of this function, so [VIEW-3] admits
it, and step 4 publishes `len(block) = <the view's carried formation datum> + w`
over a datum with empty support. The formation datum is the right one and the entry
datum is not: they agree here only because nothing touches `block` in between, and
under the third draft's reading a caller who removed an element first published a
length one too large. `ensures ile(written, len(rest))` discharges from step 4 and
the standing `Z <= len(P)`.

`dispose filled using (blocks);` is [PROV-6], one statement where the third draft
needed two. No `seq_clear` first, because the walk drains what it finds; no region
around it, because `using` names a place; and the store match is `filled`'s type
naming `'p` against `blocks`'s. A lease from a second pool in a second region would
not typecheck here, which is round 3's rank-one break made unrepresentable rather
than checked.

`drain<'p, 'd>(block: &'d filled, ...)` is **[CALL-1]**: a shared borrow is a kill
event for nothing, so `len(filled)` survives and discharges `drain`'s `requires`
from `render`'s `ensures`. It costs its own `region 'd`, because `'b` was opened
before `filled` was bound and [OWN-10] requires a borrow of a local to name a
region opened inside that binding's scope. That is the second region this one call
pays for, and Q11 is where the relief is recorded.

`loop @queue` moves `pending` and `ring` from inside the loop body, which [OWN-11]
forbade outright; **[LIV-1]** replaces that prohibition with the condition that
matters, and both are restored on every backedge and live on the `break` edge. The
loop has a resolved `break`, so [STK-4] gives it a normal successor and `main`'s
`return` is reachable.

**One thing checked rather than assumed.** The `match` after the first
`update pending ... into unplaced;` has a `Some` arm that returns from `main`. That
edge leaves `main` with `pending`, `ring` and `first` all non-linear and the pool
not yet reserved, so [LIV-1] owes no disposal there. An earlier reservation would
have made that arm an error, which is the check [PROV-6] performs on every exit
edge and not only on the last.

**One deferral, stated rather than hidden.** The ring is a transmit buffer and this
program has no way to reach a device: `main`'s effect row may name only its own
labelled inputs [FN-7], and the `command` table has no device row. That is open
question Q4, and it is why 4.1 is a queue rather than a driver.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one named operation with a typed
failure, disposal is one statement, the append helper takes the view by value and
returns it, and `OutOfMemory` is a value on an ordinary edge.

```wf-design
const ceiling: u64 = 4096_u64;

fn collect['o, 's](out: own AppendView<'o, u8>, source: own Span<'s, u8>) -> filled: own AppendView<'o, u8> reads(out, source), writes(out) contract {
  requires ile(len(source), room(out));
  ensures ile(len(filled), cap(out));
} {
  doc "Appends every byte of source into the view's spare window.";
  let count = len(source);
  for @copy (
    at in 0_u64..count,
    invariant spare: ige(room(out) + at, count)
  ) {
    let byte = source[at];
    update out by seq_push(value: byte);
  }
  return move out;
}

fn grow['h, 'b](buf: own HeapVector<'h, u8>, heap: &uniq 'b Heap<'h>, additional: own u64) -> outcome: own Result<HeapVector<'h, u8>, OutOfMemory<HeapVector<'h, u8>>> reads(buf, heap), writes(buf, heap), allocates(heap) {
  doc "Reserves spare capacity, handing the vector back unchanged when the store refuses.";
  return seq_reserve_heap(vector: move buf, heap: &uniq 'b deref(heap), additional: additional);
}

command fn main['h](command.stdout as sink: own Output, command.heap as heap: own Heap<'h>) -> status: own ExitStatus reads(sink, heap), writes(sink, heap), allocates(heap) {
  doc "Collects one fixed input buffer into a heap vector and writes it out, reporting a refusal instead of dying.";
  let input = seq_filled<u8, 4096>(value: 65_u8);
  let empty = seq_heap<u8, 'h>();
  let total = 0_u64;
  let code = 0_u8;
  region 'g {
    let reserved = grow<'h, 'g>(buf: move empty, heap: &uniq 'g heap, additional: ceiling);
    match reserved {
      Ok(value: ready) => {
        region 'fill {
          let view = seq_append_view(vector: &uniq 'fill ready);
          region 's {
            let line = seq_span(vector: &'s input);
            let done = collect<'fill, 's>(out: move view, source: move line);
            set total = absorb(view: move done);
          }
        }
        region 'w {
          let body = seq_span(vector: &'w ready);
          region 'c {
            let outcome = write_once<'c, 'c, 'w>(output: &uniq 'c sink, source: &'c body, start: 0_u64, end: total);
            match outcome {
              Ok(value: next) => {
              }
              Err(error: problem) => {
                set code = 74_u8;
              }
            }
          }
        }
        dispose ready using (heap);
      }
      Err(error: refused) => {
        let recovered = move refused.rejected;
        dispose recovered using (heap);
        set code = 70_u8;
      }
    }
  }
  return exit_status(code: code);
}
```

#### The writer's-eye walkthrough

`command fn main['h](...)` declares one region parameter, which [FN-7] admits only
because the entry selects `command.heap` [PROV-1]. `'h` is the heap store's name
for the whole program, and it appears in `Heap<'h>`, in `HeapVector<'h, u8>` and in
`OutOfMemory<HeapVector<'h, u8>>`. Nothing else can name it, because [OWN-3] makes
a caller-supplied region incomparable with every other.

`let input = seq_filled<u8, 4096>(value: 65_u8);` is the row whose absence made the
first draft's `wfgrep` migration unreachable: `seq_fixed` gives `len = Z`, and under
[CNT-2] a zero-length container is unreadable and unwritable until elements have
been placed one at a time, so a `MutSpan` formed on it names no bytes. That
addressability requirement is also the cost [VIEW-7] records.

`let empty = seq_heap<u8, 'h>();` writes its store region because no operand
supplies it, publishes `len = Z`, `cap = Z`, `room = Z` and **allocates nothing**:
an empty growable sequence owns no backing. That is L4 at the constructor, and it
is why `empty` is safely linear from its first statement.

`grow<'h, 'g>(buf: move empty, heap: &uniq 'g heap, additional: ceiling)` is
**[CALL-2]** on `buf` and the single acquisition point of the program. It is also
why [PROV-7] exists: `grow` lends `&uniq 'g Heap<'h>` onward to `seq_reserve_heap`,
whose result type names `'h` and not the loan region `'g`, which is the admitted
condition and is equally true of `pool_take`, `arena_take` and `seq_lease`. The
second draft's region-free condition admitted this call and refused all three.

On the `Ok` arm, [SEQ-0]'s relations arrive over `grow`'s call datums:
`cap(ready) = cap(empty) + ceiling` and `len(ready) = len(empty)`. The capacity is
an **equality, not a lower bound**, which is what keeps L15 honest.

`let view = seq_append_view(vector: &uniq 'fill ready);` writes no arguments and
publishes `len(view) = Z`, `cap(view) = <formation datum of room(ready)>`. **The
view value holds the loan** [VIEW-2], exclusively, so a second `AppendView` on
`ready` is refused at its own formation by [OWN-5] 601's origin-conflict sentence,
which [PROV-3] preserves and widens.

`set total = absorb(view: move done);` is the statement the second draft could not
write: `done` is a call result, so its origin set is `{ready, immutable-const}` by
[FN-1] 1036 and is never a singleton, while [VIEW-3] requires a singleton
**resolved** set. The commit ends the loan at the consume rather than at the end of
`'fill`, which is what lets `region 'w` read `ready` immediately afterwards, and it
names the datum the view has carried since its formation.

`write_once<'c, 'c, 'w>(...)` is [VIEW-7] over a view. Its obligations are
`ile(0_u64, total)`, implicit, and `ile(total, len(deref(body)))`, which discharges
from [VIEW-2]'s `len(body) = len(ready)` and [VIEW-3]'s published
`len(ready) = Z + total`. This is the statement that makes goal A's container half
real. Its two regions are the output's loan and the descriptor's loan, and `'w` is
the viewed data's; `region 'c` exists for [OWN-10] and is opened after `body` is
bound.

`dispose ready using (heap);` is [PROV-6], on both arms and in one statement each.
A `HeapVector<'h, u8>` is linear, so `region 'g` cannot be left with one alive; the
walk drops each `u8` element, which derives nothing, and then releases the backing
to the store `'h` names. `heap` is the entry's own `own Heap<'h>` binding and needs
no region, because `using` names a place. On the `Err` arm `refused.rejected` is
the original owner handed back unchanged (L3), reached by an ordinary field move
that [CNT-4] admits because the field's type names `'h` exactly as the struct's
instance does. **There is no path on which the process disappears**, which is the
whole of goal B.

#### What the compiler reports

```text
note: queue.wf is source-resource-closed; envelope written to queue.E
note: collector.wf is not source-resource-closed
  [RES-4] main selects command.heap
    heap-reaching path:  main -> grow -> seq_reserve_heap
  a general store cannot appear in an envelope [L6], so no envelope is computed
  still true of this program:
    no covered-resource failure is a trap [RES-6]; seq_reserve_heap returns a value
    the heap is reachable only through the parameter above [PROV-4]
    every release of heap-owned storage is a statement that names the heap [PROV-6]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

Five diagnostics the design owes a writer, each citing a rule that exists in
section 3. Two more, [PROV-1]'s `SecondStoreInOneRegion` and [CNT-4]'s
`ConfinedFieldWithoutRegion`, are stated inside their rules and are not repeated
here. The first is what a push without a capacity proof reports:

```text
Semantics/Source [SEQ-0]: UndischargedOperationDomain
  operation: seq_push
  residual:  "Z < room(view)"
  mechanical_fix: state a header invariant over room(view) [INV-1, MSR-5],
    dominate the push with a branch on room(view), reserve on the owner before
    forming the view, or use seq_try_push
```

It cites `[SEQ-0]` and names the operation in its payload, because [DIAG-1] 1535
admits exactly one numbered language rule and an inventory row is table data.

```text
Semantics/Source [PROV-6]: LinearValueNotDisposed
  binding "block" of type PoolVector<'p, u8, 256> is live on the edge leaving 'b
  its store region 'p is reserved at "blocks" of type Pool<'p, FixedVector<u8, 256>, 8>
  mechanical_fix: move the value out of this scope, or write
    dispose block using (blocks); a store-backed value has no compiler-derived
    release, so nothing else can free it

Semantics/Source [PROV-6]: DisposeProviderMissing
  disposing "table" of type FixedVector<Chunk<'p>, 8> reaches a linear leaf
    Chunk<'p>.page of type PoolVector<'p, u8, 4096>
  no named provider has a type naming 'p
  mechanical_fix: name the provider of 'p in the using list; a dispose names one
    provider for every store its value's type reaches

Semantics/Source [PROV-6]: LinearValueAcrossPropagate
  binding "held" of type HeapBox<'h, u64> is live on this propagate's error edge
  mechanical_fix: expand the propagate into a match and dispose "held" on the Err
    arm; a propagate edge leaves every enclosing scope and has no statement
    position on which a disposal could run

Semantics/Source [SEQ-0]: AffineFillElementType
  operation: seq_filled
  element type Option<Task> is affine, and this row requires a copy element type
  mechanical_fix: use seq_vacant<Task, 32>() for a table of vacant slots, which
    publishes len = N and needs no copy; or place elements one at a time
```

The last is a selection error rather than a caller's later discovery, and it is the
message round 3 asked for by name: a writer building a task table types
`seq_filled<Option<Task>, 32>` first.

---

## 5. Open questions

Everything the owner's rulings settle is dropped and not restated. So is
everything the earlier drafts asked and this one answers: the length-class terms
and the goal disposition are [MSR-1] and [MSR-4]; the arithmetic residual is
[MSR-3]'s datums and images; the `absorb` commit is [VIEW-3]; the coverage
certificate died with `Builder`; the arena's reclamation is [RES-5]'s cursor
domain; the optimizer-versus-envelope question is [STK-3]; the profile table is
[RES-2]. Six questions earlier drafts filed are **answered here rather than asked**,
and the answers are on the merits:

- *What identifies a store?* Its region, carried in the type of every value it
  backs, [PROV-1]. A place is defeated by a move plus a rebind and by a runtime
  offset; a type is defeated by neither.
- *What disposes a store-owned value?* One structural statement, [PROV-6], closed
  under containment exactly as linearity is. A four-operation list cannot name a
  container of leases, and a derived release cannot state its own subject.
- *Does disposal need its own effect category?* No. A `disposes(item -> pool)`
  entry would restate the two formals' types, and the footprint it would supply is
  the `writes` row the statement already carries.
- *Do region-parametric nominals belong in this version?* Yes, [CNT-4]. The
  deferral bought no soundness once confined values were admitted into container
  elements, and its cost was every kernel structure written as parallel columns.
- *Should the value-in / value-out spelling get sugar?* It gets a statement form in
  two shapes, [LIV-3], which is not sugar because it is a [SET-2] exchange and
  reaches places `set` cannot.
- *What about control entering the call graph from outside it?* Out of scope for
  this batch, and section 1.4 states the interface the execution-context design
  inherits, including the two rows it must reopen.

What remains is what this design genuinely does not decide.

**Q1. May a resource-closed program handle a typed refusal, or must it prove every
acquisition?** *(a)* Strict: every covered acquisition uses the proved spelling.
*(b)* Permissive: both spellings are admitted, since neither can ask for more than
`E`.
**Recommend (b), and L8 plus [RES-6] make it real.** A refusal edge carries the
store's own `room(store) = Z`, and 3.3.1's loop rule names the checked spelling as
one of the three things that bounds a retaining loop, on any path rather than only
on a loop exit.

**Q2. Where does a hosted resource-closed program's large memory come from?**
*(a)* Frame and extent placement only, as [PROV-5] provides. *(b)* One more entry
row delivering a committed region, `command.region as store: own Arena<'store, B, A>`.
**Recommend (a).** `pool_extent` and `arena_extent` already produce a `region` item
of `E` that a deployment grants separately. (b) becomes right the day a program
needs a store whose *size* is a deployment decision rather than a source constant,
and it puts a deployment-shaped input on every hosted program's entry, so it should
wait for a program that needs it.

**Q3. Does the range relation need a splitting operation?** [PROV-3] gives each
origin the half-open range its value reaches, and [RUN-3] uses it, so a counted
`MutSpan` fill has [PAR-2] permission in this version. What it does not yet have is
`seq_split_at`, which hands a writer two views of disjoint ranges of one owner as
two values. *(a)* Add it now. *(b)* Leave it; the counted fill covers the case a
program has today.
**Recommend (b) for this version and (a) as the next view operation.** The relation
the split needs already exists, which is the point of having written the range into
[PROV-3] rather than refining one loan; what is missing is only the row and its
declared relations, and no program in this batch needs it.

**Q4. How does a resource-closed program reach a device?** `main`'s effect row
names only its own labelled inputs and the `command` table is closed, so 4.1 has a
transmit ring and no way to flush it. *(a)* A sixth row on the hosted table.
*(b)* A second program kind under [FN-7]'s existing closed-table discipline, with
its own standard-input table naming memory-mapped regions.
**Recommend (b), as a named deferral, arriving with the execution-context design
of 1.4.** An interrupt vector and an MMIO window are one batch: a handler with no
device to service and a device with no handler are each half a driver. (a) would
put a device on every hosted program's entry.

**Q5. When does `par` become usable inside a resource-closed program?** [RUN-2]
denies [PAR-1] to [PAR-3] permission there and publishes `lanes(1)`, because the
current runtime's wait path executes a stolen task on the waiting lane's own stack
and no term of [STK-3] counts that. *(a)* Restrict the shapes whose lowering can
nest a stolen task, and deny permission for the rest. *(b)* Build the
compiler-managed work-first continuation representation, then lift the denial and
define a worker lane's chain [STK-3].
**Recommend (b), and note that [RUN-2] is what makes it a scheduling item rather
than a soundness risk.** This is the largest engineering item the design implies.

**Q6. Does this version want a keyed or sparse container family?** [CNT-2] writes
stable-identity storage as `seq_vacant<T, N>` over `FixedVector<Option<T>, N>` with
element-position `replace`, which is sound, is L12-clean, and compiles in shape
today (probe `r2_7`). Its remaining cost is one `Option` word per slot and one
`match` per read; the construction loop and its two header invariants are gone.
*(a)* Leave it there. *(b)* Add a `FixedTable<T, N>` whose typestate is an
occupancy set, whose whole operation surface is index-local so no quantified
proposition arises, and whose occupancy word is representation rather than language
state.
**Recommend (a) for this version and (b) as the next container family.** (b) is
what a kernel object table, a page cache and a slab front end actually want, and it
is exactly the "keyed containers are fixed families over the core, later" the owner
settled.

**Q7. Should a system operation be able to append?** [VIEW-7] gives a destination
`&uniq 'd MutSpan<'r, u8>`, so an I/O buffer is addressable before the host writes
into it and the byte count comes back as an ordinary `u64` beside the container.
*(a)* Leave it: one fill per buffer, and the length typestate does not reach the
boundary where lengths come from outside. *(b)* Give the producing operations
`own AppendView<'r, u8>` and an ordered result list, so the bytes the host wrote
become the view's `len` and `absorb` publishes the owner's new length.
**Recommend (b), in the batch that lands multi-return in the [SYS-2] declaration
domain, and not here.** (b) is the right answer and it is the one place where the
whole write-back protocol pays for itself twice; it also requires the system domain
to gain a result-list shape, which is a change to [SYS-2]'s records and counts and
belongs beside them.

**Q8. Is `copy` structural over aggregates?** [OWN-1] makes every owned composite
affine regardless of its field types, which is why `seq_filled` and
`seq_heap_filled` admit only primitives and why P17 exists. *(a)* Leave it.
*(b)* A `struct` or `enum` all of whose field types are copy is itself copy.
**Recommend (b), and note that it is not this design's to land.** Under (b)
`seq_filled<Descriptor, 64>` becomes a construction instead of a loop. It is an
[OWN-1] question with its own consequences across the language, and this design
names it because it is the reason two of its own rows read `T copy`.

**Q9. Is `E` part of program identity?** *(a)* Diagnostic output only. *(b)* An
emitted machine-readable table beside the object.
**Recommend (b), and explicitly not part of [PROG-2] compilation-unit identity.**
The envelope is useless if the deployment cannot read it, and keeping it out of
unit identity keeps it a derived fact about one build, which [STK-3] and [RUN-5]
both say it is.

**Q10. Should a `propagate` be able to carry a disposal?** [PROV-6] refuses a
`propagate` in a function holding a live linear binding, because the error edge has
no statement position on which a disposal could run, and probe `w5` shows the
language admits that shape today. The cost is a five-line `match` in every
allocating helper that propagates, which `growable_vec.wf` and `byte_string.wf`
both are. *(a)* Leave the refusal. *(b)* Admit a release list on the statement,
`let x = propagate f(...) disposing (held using (heap));`, checked by exactly
[LIV-1].
**Recommend (a) now and (b) as a measured follow-on.** (a) is one sentence and one
diagnostic, and it is honest: the third draft refused the same shape and did not
know it. (b) is a new production carrying a statement list in an initializer, and
it should be paid for by a program whose rewrite under (a) was actually painful,
which the migration of the two corpus programs will show.

**Q11. Should a view-forming borrow need its own written region?** [OWN-10] forces
`region 'dest { let window = seq_mut_span(vector: &uniq 'dest input); region 'call { ... } }`,
two regions and one formation at every I/O site, which is [VIEW-7]'s recorded cost
and which `wfgrep` pays seven times. *(a)* Leave it. *(b)* Admit a **formation
borrow** as a call-scoped temporary whose region is introduced by the formation
statement itself, so `let window = seq_mut_span(vector: &uniq input);` writes no
region.
**Recommend (b), and note that [VIEW-2] has already made the argument for it.** If
the argument borrow is not the freeze, and [VIEW-2] says in terms that it is not
because the view value holds the loan, then it does not need a writer-named region.
It is deferred here only because it is an [OWN-10] change with reach beyond this
design's own operations, and because no program in section 4 is blocked by it.

---

## 6. Verified versus reasoned

**Verified** means a compiler executed it. The binary is the gate-profile
`whitefootc` built from this tree; every probe below was run against it, either in
the session that wrote this file or in one of the twelve falsifier sessions whose
verdicts are quoted with their probe names. No timing figure from any machine
appears anywhere in this file.

### 6.1 What the current compiler does

Eight probes were run in the session that wrote this draft, to check what this
draft newly rests on rather than to re-inherit earlier verdicts. The table
describes each probe program closely enough to rewrite it; the sources were session
scratch files and are not in the repository.

```text
| probe            | program                                                        | verdict                                   |
|------------------|----------------------------------------------------------------|-------------------------------------------|
| w1_regionname    | two `region 'r` blocks in one function                         | REJECTED [OWN-3] RepeatedRegion           |
| w2_arenareplace  | `let old = replace first = move second;` at arena<'r, u64>     | ACCEPTED, exit 0                          |
| w3_clausecall    | `requires ile(len(source), len(target));`                      | REJECTED [GRAM-9] at parse, expected      |
|                  |                                                                | [":", ")", ",", "[", "."]                 |
| w4_elemwrite     | hoisted `len`, `set slots[0] = ...`, then a guarded subscript   | ACCEPTED, exit 0                          |
|                  | against the hoisted length                                     |                                           |
| w5_propagatebox  | a live `box<u64>` across a `propagate` edge, derived drop after| ACCEPTED, exit 0                          |
| w6_setaffine     | `set held = box_new(6_u64);` after `let taken = move held;`    | REJECTED [OWN-1] UseAfterMove at the set  |
| w7_boxfield      | a `box` field in a struct, one field moved out, the residual   | ACCEPTED, exit 0                          |
|                  | record consumed by a callee declaring only reads(c.tag)        |                                           |
| w8_twouniq       | two `&uniq 'b x` argument borrows of one place in one region,  | ACCEPTED, exit 0                          |
|                  | with `set x = 7_u64;` between them                             |                                           |
```

What each establishes, and which rule it changed rather than confirmed.

- `w1` is [PROV-1]'s enabling fact and the reason a region can be a store's name:
  [OWN-3] 573's uniqueness is enforced, so one spelling denotes one occurrence.
  Without it the brand would need a new binder kind.
- `w2` is a **live compiler defect found by round 3 and re-confirmed here**.
  [SET-2] 512 makes a region-bearing replace target a hard error for
  `slice<'r, U>` **and** `arena<'r, U>`; `check_mutation_target_class`
  (`compiler/src/semantic/check/expressions.rs:310-326`) tests only the slice
  variant. It is benign at this tip and load-bearing for B6, which must implement
  [PROV-3] use 3 as a relation over loan-bearing types rather than re-wording
  [SET-2] over one `CheckedType` variant.
- `w3` is [MSR-5]'s amendment, and it is why the amendment goes to [GRAM-5] rather
  than to [GRAM-9]: the compiler's own mechanical fix names `define`, because
  `atom` has no `call` alternative and [GRAM-9] is only the attribution.
- `w4` is [MSR-2]'s second consequence at v0.40 scale: an element write does not
  kill a length, and the surviving length reaches an [OP-4] goal.
- `w5` is the program [PROV-6] now refuses. A heap value is live across a
  `propagate` edge and the compiler derives its release there; under [PROV-6] there
  is no derived release and no statement position, so the design owes the writer a
  `match`. This is a capability the language has today and this design removes.
- `w6` is [LIV-2]'s premise from the dead side: a `set` of a consumed affine
  binding is [OWN-1] `UseAfterMove` **at the set**, and probe `p10` is [STOR-1]
  `AffineSetTarget` from the live side. The two together are exactly the one
  premise [LIV-2] states.
- `w7` is [PROV-6]'s virality target one level deeper than `r2_5`: a struct with a
  heap-backed field is consumed by a callee whose row names only a copy field, and
  the free is invisible. It is also the shape [CNT-4] must keep admitting, which is
  why the position prohibition narrowed to loan-bearing types.
- `w8` is why store identity may not rest on a place: two argument borrows of one
  place coexist as call-scoped temporaries with an ordinary write between them, so
  nothing a provider operation touched stays frozen after its own statement.

Inherited verdicts this draft still rests on, from the twelve falsifier sessions,
grouped by what each group establishes:

```text
| probes                             | what they establish                                    |
|------------------------------------|--------------------------------------------------------|
| d1 conformance case                | D1 reproduces at this tip, ACCEPTED exit 0             |
| p1, p6, f7                         | [CALL-1] and [CALL-2] already behave; D1 is narrow     |
| p7, p9, k12, p2, p8, k09, r1_multi | MutSpan writes, affine elements, len(result) and       |
|                                    | multi-return are new capability, not compiler defects  |
| p5_ambient, n4, r1_ambient, r2_5,  | ACCEPTED: allocation while holding nothing, and a free |
| q9                                 | inside a `pure` callee; L2's and L13's evidence        |
| f1c, f1d, f2b, r1_twouniq, r2_1,   | why a view value holds the loan, and why a borrow      |
| r2_2, c4                           | region must open after its binding                     |
| f3, f5, f6, r1_own11               | the three avoidances [LIV-1] replaces                  |
| p10                                | [STOR-1] AffineSetTarget, the live half of [LIV-2]     |
| f2b_tail, f8_tailframe, p3_rec     | the witnesses refuting the syntactic tail conditions   |
| n2_idle, f3_forever, k30,          | [FN-1] FunctionFallthrough on the idle and driver      |
| n3_propagate_loop                  | loops [STK-4] admits                                   |
| f7_regionresult, k05, r2_6, r2_8,  | [CNT-4], `update` and [MSR-5] are new syntax           |
| r1_lenatom, r1_field, c1 / c2      |                                                        |
| r2_4, r2_4b, r2_4c                 | the measure kill is root-granular today                |
| r2_7, k24, n13                     | element-position replace with a surviving len          |
| r2_9, r2_10                        | the arm-fact route, and the readers' purity            |
| q1, q2                             | a box field and a replace of one, which [CNT-4] and    |
|                                    | [PROV-3] must keep admitting                           |
| q3, q7                             | a partial move kills the root and the later `set` is   |
|                                    | refused, which is why [LIV-3] is a [SET-2] exchange    |
| n14, n15, n19                      | no loop publishes len = N, which is why seq_vacant is  |
| c8                                 | why seq_exchange is not pure                           |
| r1_relend, r1_relend_affine        | why [PROV-7] exists                                    |
| k21 / k21b, k08, k31, b4b          | the fill loop's arithmetic and the guard route         |
| n7_par, --stack-ledger, six programs| par eligibility, three disjoint chain roots, all pass  |
```

### 6.2 The proof surface, isolated

```text
| probe                      | shape                                                    | verdict                          |
|----------------------------|----------------------------------------------------------|----------------------------------|
| v23_param_anchored         | counted loop, header invariant, ensures over a parameter  | ACCEPTED                         |
| v24_len_anchored           | identical, ensures over len(deref(destination))           | REJECTED [FN-9]                  |
| v25_subscript_consumer     | identical loop, consumer is a subscript                   | ACCEPTED under [OP-4]            |
| v26_ensures_consumer       | identical loop, consumer is an ensures                    | REJECTED [FN-9]                  |
| q2b / q3b                  | one file differing in one token                           | ACCEPTED then REJECTED, in one   |
|                            |                                                          | compilation                      |
| k22                        | ensures over a hoisted len after a proved loop            | REJECTED [FN-9], residual        |
| v22_loop_then_inv_stmt     | the [INV-1] conclusion proves and does not reach [FN-9]   | REJECTED [FN-9]                  |
| q5 / q5' / q5''            | one-line invariant header; -wrap on the backedge; exact - | [FORM-2]; [INV-1] Backedge; OK   |
```

`q2b`/`q3b` and `k22` are why [MSR-4] is a law-level change rather than a repair:
the same proof, asked by two consumers, inside one accepted-then-rejected
compilation.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in section 3.** None is implemented, and no compiler has seen any of
  the new types, operations, terms, statements or markers.
- **Every program in section 4** and every diagnostic quoted there. They are
  written against the unchanged v0.40 rules as well as this design's and were walked
  against both, and the walkthrough records two statements that would be rejected as
  printed rather than repairing them silently.
- **Every figure in 4.1's envelope**, which is why every one of them is written as
  a composition or as `<post-codegen>` rather than as a number.
- **[PROV-1]'s brand, which the whole draft now rests on.** That a region can carry
  a store's identity as well as its extent, that one-store-per-region is a
  restriction no program needs to violate, and that invariance follows from
  [OWN-12] and [TYPE-5] with no variance design are argued from rule text. The
  claim most worth attacking is the last: if any rule anywhere admits a region
  argument by outlives rather than by identity, two stores become one type.
- **[PROV-6]'s structural walk.** That it terminates on every type the language can
  form, that its order matches [STOR-3]'s derived-drop order leaf for leaf, and
  that `dispose` on a partially moved aggregate is either well defined or refused
  are argued and not executed. The partial-move case is the one round 3 raised and
  this draft does not answer: probe `w7` shows a partially moved record is legal
  today, and `dispose` of one is not stated.
- **The composition algebra of 3.3.1.** Its sequence and branch rules over an
  exit-label map are standard, the no-fallthrough case is defined, and the interval
  arithmetic is now stated. Its `par` rule depends on a runtime profile that does
  not exist. Its loop rule's third discharge, a writer invariant, has never been
  exercised against a program with two stores.
- **[MSR-3]'s measure datum, re-keyed.** That one former on (point, place, measure)
  covers all its consumers is checked by enumeration here and not by execution, and
  the borrow placement's soundness rests on [FN-8] 1269's own split.
- **[PROV-5]'s store reset.** That no value can observe the reset, because every
  one names `'s` and [LIV-1] kills it on that edge, is argued from two rules and
  not executed.
- **Everything about the current runtime's closure.** [RUN-1] is written as a
  qualification obligation precisely because no existing target can be certified to
  meet it, and the `--stack-ledger` read above shows the entry chain is presently
  three disjoint roots.
- **The claim that `wfgrep` becomes heap-free.** Its eleven `buffer_new` calls
  reach three declared rows, all of which [SEQ-0] and [VIEW-7] replace. The
  substitution was not performed and compiled, and it moves bytes out of the heap
  and into frames, which is a [STK-3] question rather than a free win.

### 6.4 Falsifiers this design asks for next

1. Attack [PROV-1]'s invariance: find any rule that admits a region argument by
   outlives rather than by identity, at a nominal, at a container element type, at
   a `Result` payload, or through [FN-2]'s substitution, and make two stores share
   one type.
2. Attack [PROV-6] with a partially moved linear aggregate, with a linear value
   inside an `Option` disposed on one arm of a join, and with a type whose walk
   order differs from [STOR-3]'s.
3. Attack [PROV-5]'s reset with a value that escapes `'s` through a result, through
   a `give`, or through an enum payload the confinement check reads differently.
4. Hand-execute 3.3.1 on 4.1 and on a two-store program, and check the interval
   arithmetic and the repaired loop route against both.
5. Attack [MSR-3]'s datum with a `propagate` edge, a `value_if` delivery, and a
   datum whose call is inside a loop body that the loop-header kill rewrites.
6. Attack [LIV-3]'s exchange judgment where the operation's own arguments read the
   target's offset, and where the operation diverges.
7. Rewrite `wfgrep` and `byte_string` by hand against [VIEW-7], [PROV-6] and Q10's
   refusal, and count what the `MutSpan` destinations, the disposals and the
   propagate rewrites cost at every site.
8. Attack [RES-7]'s exclusion test against a runtime that does allocate in one
   operation, and check that the test excludes exactly that operation.

### 6.5 Falsifier round 1: what each finding hit, and what refuses it now

Every BREAKS, DEFECT and BLOCKING finding of the first four reports, one line each,
with the rule that refused it in the **third** draft. Where round 2 reopened a
finding, the row says so and 6.6 carries the repair; where round 3 reopened one,
6.7 carries it. The reports are superseded.

```text
| finding                                                       | disposition                                                 |
|---------------------------------------------------------------|-------------------------------------------------------------|
| F1-1 [OWN-11] refuses every value-in/value-out loop            | [LIV-1] replaces the move prohibition                       |
| F1-2 reinitializing set makes liveness path-dependent          | [LIV-1] join agreement; release unconditional               |
| F1-3 [SEQ] publishes terms the operation killed                | reopened in round 2; [MSR-3]'s pre-transfer datum           |
| F1-4 views have no loan strength; two AppendViews              | [VIEW-2] the view value holds its own loan                  |
| F1-5/6/7 [BLD] certifies a range, cannot release, is denied    | [BLD] deleted; [SEQ-0] plus [RUN-3]                         |
| F1-8 a heap free exhibits nothing, so two frees race           | reopened in round 2; [PROV-6] linear disposal               |
| F1-9 the Heap may die before its allocations                   | [PROV-6]'s provenance and [LIV-1]'s scope exit              |
| F1-10 [STOR-5]'s position list omits container elements        | [CNT-4] intensional prohibition                             |
| F1-11 len(P) forbids a subscript                               | [MSR-1] admits a subscripted place                          |
| F1-12/13/14 no FIFO, no exchange, no runtime-chosen target     | [CNT-1] FixedRing with a subscript; seq_take_at,            |
|                                                               | seq_exchange; [PROV-3] formation on a subscripted place     |
| F1-16 the try rows publish nothing arm-specific                | [SEQ-0] per-arm relations and the arm route                 |
| F2-A1 every checked acquisition is untypeable                  | [CNT-4] confined generic arguments                          |
| F2-A2 tail lowering rewrites a live-frame component            | [STK-1] deadness; [PROV-5] closes the confined route        |
| F2-A3 static providers frame out; par over one extent          | [PROV-5] one store per activation                           |
| F2-A4 L8 kills the checked-acquisition escape hatch            | L8 split; [RES-6] publishes the refusal                     |
| F2-A5 [RUN-2] licenses inline execution and waiting            | [RUN-1] forbids both; [RUN-2] answers the stack half        |
| F2-A6 E's stack item starts at main                            | [STK-3] the whole chain, both directions                    |
| F2-A7 a lease outlives a moved provider                        | [PROV-2] &uniq only; [PROV-6] disposal                      |
| F2-A8 [RES-1] and [RES-5] disagree about runtime stores        | [RES-1] one domain each; [RES-7] writes the exclusions      |
| F2-A9 a resource-closed program cannot leave main              | [STK-4], stated over the right quantity                     |
| F2-A10 confined containers neither storable nor returnable     | [CNT-4], both halves                                        |
| F2-A11 L9's defined overwrite makes the judgment vacuous       | L9's second clause                                          |
| F2-A12 live/capacity/remaining are not terms                   | [MSR-1] retires them into len/cap/room                      |
| F2-A13 the composition algebra is not a function               | 3.3.1's exit-label map, no-fallthrough case defined         |
| F2-A14 reachability is rooted in the formal's type             | [PROV-4] roots it in the leaf's type                        |
| F2-A15 acceptance depends on target and runtime                | [RES-3] two stages; [RES-5]'s ceiling arithmetic            |
| F2-A16 the profile table leaves the promise unquantified       | [RES-2] the table is the promise                            |
| F2-A17 E does not compose across units                         | [RES-8], split by stage                                     |
| F2-A18 [RES-4]'s example entry does not typecheck              | 4.1's entry is pure; Q4 states the device route             |
| F3-R1 [OWN-11] unregistered                                    | registered; [LIV-1]                                         |
| F3-R2 named versus positional arguments                        | [SEQ-0]; its type and const half reopened in round 2        |
| F3-R3 len/cap/room cannot appear in a clause                   | [MSR-5] clause operands are terms                           |
| F3-R4 [GRAM-2]/[GRAM-4]/[FORM-2] unregistered                  | registered; [CALL-4] states the rendering                   |
| F3-R5 the publishes column has no fact source                  | [SEQ-0] and [ENT-3] source S13                              |
| F3-R6/R7 [SYS-2] and [TYPE-7] unregistered                     | registered                                                  |
| F3-2.2 unchanged rules listed as amended                       | the register is three lists and is derived                  |
| F3-4.1 seq_shrink contradicts L3                               | [SEQ-0]'s row returns Result                                |
| F3-4.2 [RES-8]'s [SYS-9] analogy is backwards                  | deleted; [SEQ-0] is the fact source                         |
| F3-4.3 reserve<T>() is undefined                               | [RES-5]'s constant K<T> from [OP-9]                         |
| F3-4.4 cap(a) + len(v) = cap(v) is not an L0 fact              | [VIEW-2] publishes cap(a) = room(v)                         |
| F3-4.12/4.13 the builder rules do not close                    | [BLD] deleted; [VIEW-4] states the ground                   |
| F3-4.16 the implementation order is unsatisfiable              | section 7 re-derived again                                  |
| F3-4.17 B1 silently invalidates patterns.md P16                | named in the register, with a second [MSR-2] correction     |
| F3-D1..D6 the judgment depends on target and codegen           | [RES-3] two stages; [STK-3]; [RUN-1]                        |
| F3-3 about 25 statements of section 4 are refused              | both programs rewritten, and walked against v0.40 too       |
| F4-1 room has no reader and no relation                        | L15 restated; [MSR-2] identity; [SEQ-0]'s readers           |
| F4-2 no filled construction, no middle removal                 | seq_filled, seq_take_at, seq_exchange                       |
| F4-3 proof routes are granted per consumer family              | [MSR-4]                                                     |
| F4-4 [INV-1] atoms are identifiers                             | [MSR-5]                                                     |
| F4-5 conditional append has no join-preserving image           | [MSR-3]'s delta-atom join                                   |
| F4-7 same-region view results alias silently                   | [VIEW-6] is a declaration error                             |
| F4-8 diagnostics cite wrong or nonexistent rules               | section 4's five diagnostics cite section 3                 |
| F4-9 both worked programs are untested transcriptions          | both rewritten and walked                                   |
```

### 6.6 Falsifier round 2: what each finding hit, and what refuses it now

Every BREAKS, DEFECT and BLOCKING finding of the four round-2 reports, one line
each, with the rule that refused it in the **third** draft. Round 2's diagnosis was
that round-1 repairs were added piecemeal; the right column is therefore mostly the
same six concepts, which is the point. Three of its rows name mechanisms this draft
replaces, and 6.7 says what replaced them: [CNT-5] is deleted, place-based
provenance is a type parameter, and the parameter-keyed datum is keyed on a place
and a point.

```text
| finding                                                        | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| F1-a1 BREAKS a move equality names a dead root and [LIV-2]      | [MSR-3]'s datum publishes the equality; [LIV-2] makes the  |
|   revives it under one term identity                            | rebound binding a distinct term                            |
| F1-a2 BREAKS D1 on &uniq MutSpan: [SET-2]'s relation was        | [PROV-3] use 3 states the property [SET-2] protects; the   |
|   replaced out from under [VIEW-4]                              | register carries [SET-2] as changed                        |
| F1-a3 BREAKS a view at table[k] freezes the collection, not     | [PROV-3] use 2: a loan covers every binding its place's    |
|   the offset, so absorb credits the wrong element               | address computation reads                                  |
| F1-a4 BREAKS seq_clear and seq_truncate free with no provider   | [PROV-6]; both rows require a non-linear element type      |
| F1-a5 BREAKS the release target is not a function of the type   | [PROV-6] selects by provenance and requires a singleton    |
| F1-a6 GAP M(c,q) blocks every value-in/value-out relation       | [MSR-3]'s pre-transfer datum, always live, empty support   |
| F1-a7 GAP no rule says where an image dies                      | [MSR-3]'s last sentence; [VIEW-3] reads a datum instead    |
| F1-a8 GAP the draft's PROV-9 admits no arena or pool operation | [PROV-7], over the loan region, with the loan extent       |
| F1-a9 GAP [PAR-2] denies the MutSpan fill                       | [RUN-3]'s iteration-formed-loan clause                     |
| F1-a10 BREAKS seq_take_front has no domain and no relations     | [SEQ-0] one row per operation; ring_take carries both      |
| F1-a11 GAP a value confined to two regions has no judgment      | [CNT-4] quantifies over every region the type names        |
| F1-a12 GAP measure support stops at the last subscript          | [MSR-2] is recursive over the whole chain                  |
| F1-a13 the draft's CNT-9 blesses &uniq slot on a false ground  | [CNT-7] corrects it; [CALL-5]'s default refuses it         |
| F1-a14 [VIEW-2]'s exclusivity rests on an unregistered sentence | the register's third list carries [OWN-5] 585              |
| F1-a16 views are usable only in the owning function             | recorded in [VIEW-6]                                       |
| F2-N1 BREAKS two pools of one type in one region; a lease       | [PROV-3] provenance plus [PROV-6]'s singleton check        |
|   released into the wrong store aliases two owners of one slot  |                                                            |
| F2-N2 BREAKS a frame extent confined to a caller's region       | [PROV-5] region-local reservation; [STK-1] follows         |
| F2-N3 BREAKS the region-free condition unblocks only the heap   | [PROV-7], stated over the loan region                      |
| F2-N4 BREAKS the release row has no subject, and is viral       | [PROV-6]; virality is real, visible, and named             |
| F2-N5 BREAKS the loop rule is discharged by a standing identity | 3.3.1's condition on the acquisitions; [RES-3]'s closed-   |
|   and admits a runtime trip count                               | expression sentence                                        |
| F2-N6 BREAKS [RUN-2] deletes the backpressure [RUN-3] needs,    | [RUN-1] separates acquisition from admission control;      |
|   and Q11 is required for soundness                             | [RUN-2] makes sequential par and lanes(1) a rule           |
| F2-N7 BREAKS the divergence amendment is over the wrong         | [STK-4]: an edge iff some break resolves to the loop;      |
|   quantity, and undefines the sequence rule                     | 3.3.1 defines the no-fallthrough case                      |
| F2-N8 BREAKS stage one reads target size and alignment          | [RES-5] uses K<T> only; [RES-2] carries both figures       |
| F2-N9 BREAKS every extent folds into one stack item             | [PROV-5]'s pool_extent and arena_extent; L6's second half  |
| F2-N10 GAP a lane's chain is undefined; PreStart commits its    | [STK-3] settles the entry stack; [RUN-2]'s W = 1 removes   |
|   own stack                                                     | the lane question for now                                  |
| F2-N11 GAP [PAR-3]'s replicated places are unmodelled           | [RUN-2]: they cannot occur in such a build                 |
| F2-N12 GAP the boundary summary mixes stages                    | [RES-8] publishes two components                           |
| F2-N13 GAP the kill list omits compiler-derived releases        | [MSR-2] states the kill over the effect row                |
| F2-N14 GAP no system operation appends                          | [VIEW-7] records the cost; Q7 states the fix               |
| F2-N15 GAP [STK-4]'s reentrancy ground is wrong                 | the premise is deleted; 1.4 states the real problem        |
| F2-N16 GAP Q4 is under-specified and unsound without N2         | [CNT-4] lands it, with invariance and [PROV-5]             |
| F2-A8-residual the promised exclusion list does not exist       | [RES-7] writes it                                          |
| F3-1 DEFECT container rows write arguments no rule admits       | [SEQ-0]'s written-argument rule, on [TYPE-5]'s criterion   |
| F3-2 DEFECT per-arm relations have no establishment route       | [SEQ-0]'s designated outcome and arm route                 |
| F3-3 DEFECT five rows declare an unrootable effect path         | deleted; those rows read reads/writes(vector)              |
| F3-4 DEFECT the absorb singleton is unsatisfiable after a call  | [VIEW-3] requires a singleton resolved set                 |
| F3-5 DEFECT [OWN-10] refuses two of the design's own borrows    | [PROV-2]'s two-region sentence; 3.11 and both programs     |
|                                                                | rewritten; probes r2_1 and r2_2                            |
| F3-6 DEFECT [PAR-2] denies the fill that replaced [BLD]         | [RUN-3], registered as a real refinement                   |
| F3-7 DEFECT affine_factor loses literals and grouping           | [MSR-5] gains alternatives and loses none                  |
| F3-8 DEFECT three inventory rows are not well-formed            | [SEQ-0]'s inventory; seq_reserve is split                  |
| F3-9 DEFECT the failure nominals are declared nowhere           | [RES-6] declares them as structs                           |
| F3-10 DEFECT [SYS-8]'s parameter modes are unspecified          | [VIEW-7] fixes both modes                                  |
| F3-11 DEFECT the readers carry a reads row                      | [SEQ-0]: they are pure; probe r2_10                        |
| F3-12/13 DEFECT [SET-1] and [SET-2] are unregistered            | [PROV-3] amends both, and both are registered              |
| F3-14 DEFECT [GRAM-3], [STOR-2] and the domain machinery        | [CNT-1], [VIEW-1], [PROV-2] and [SEQ-0] amend them, and    |
|   are unregistered                                              | the register carries every row                             |
| F3-15 DEFECT [SYS-8] and the divergence change have no rule     | [VIEW-7] and [STK-4] are rules                             |
| F3-5a premise 3 quantifies over target-stage data               | [RES-3]: a profile symbol is closed, a runtime value is not|
| F3-5b the Heap's refusal relation is unstatable                 | [RES-6]: the Heap publishes only the returned owner        |
| F3-5c the settled list is reopened three times                  | section 1's two footnotes; one canonical spelling          |
| F3-5d [MSR-1] and [PROV-1] disagree about the Heap              | [MSR-1] is a table; the Heap has no row                    |
| F3-5e section 7 leaves rules unbatched, B4's test unreachable   | section 7 re-derived, every rule batched                   |
| F3-5f a third of the rules omit the register's fields           | every rule states four fields                              |
| F3-5g [STK-4]'s source check has no source to check             | the premise is deleted                                     |
| F3-5i bulk drops silently destroy affine values                 | [SEQ-0]'s non-linear condition, and L9's note              |
| F3-5j the envelope figures contradict CONTAINERS G7             | 4.1's envelope is symbolic                                 |
| F3-5k the Heap's own release action is undefined                | [PROV-1]: the empty release row                            |
| F3-2a a diagnostic cites an inventory row as a rule             | [SEQ-0] with the operation in the payload                  |
| F3-3a two "unchanged" rows in the changed table                 | removed; the third list carries them                       |
| F3-3b three wrong line numbers                                  | corrected                                                  |
| F3-3c no META-5 delta is declared                               | declared in 3.13                                           |
| F3-1.3 rows carry more than one operation                       | one row per operation                                      |
| F4-1 BLOCKING no stable-identity element storage                | [CNT-2]'s FixedVector<Option<T>, N>; probe r2_7; Q6        |
| F4-2 BLOCKING FixedRing has no element access for non-copy T    | [CNT-1] gives a ring an ordinary subscript                 |
| F4-3 BLOCKING no source construct creates an execution context  | out of scope by ruling; 1.4 fixes the interface; 4.1       |
|                                                                | no longer pretends                                         |
| F4-4 BLOCKING Q4 buys no soundness and forces parallel columns  | [CNT-4] lands region-parametric nominals                   |
| F4-5 BLOCKING the interrupt gap is at [GRAM-2]/[FN-7]           | 1.4 says so; the [STK-4] premise is deleted                |
| F4-6 FRICTION the rebind tax dominates                          | [LIV-3]'s update statement                                 |
| F4-7 FRICTION the measure kill is root-granular                 | [MSR-2] is descriptor-precise; probes r2_4                 |
| F4-8 FRICTION seq_filled requires T copy                        | Q8 names structural copy and says it is not this design's  |
| F4-9 FRICTION a nested container has no operation               | [LIV-3]'s update reaches a subscripted place               |
| F4-10 FRICTION three drafted-diagnostic holes                   | section 4 drafts five, including all three                 |
```

Findings the reports rated HOLDS or CLEAN, preserved and not weakened: [CALL-1]
and [CALL-2] survive every shape attacked, in both rounds. [LIV-1] is complete for
[FN-1]'s conservative graph, and round 2 tried six routes around it. [CNT-4]'s
position closure really does close what [STOR-5]'s enumeration left open.
[MSR-4]'s claim that one disposition suffices survived fourteen written programs.
`seq_take_at` and `seq_exchange` create no vacancy state, `FixedRing`'s wraparound
is one scalar relation, a race in a `par` fill is not expressible, `array<T, N>` is
untouched, and `fir_filter` pays nothing. The second draft's diagnostics were the
part round 2 praised, which is why the third draft drafted five and this one seven.

### 6.7 Falsifier round 3: what each finding hit, and what refuses it now

Every BREAKS, DEFECT, GAP and BLOCKING finding of the four round-3 reports, one
line each. Round 3's diagnosis was that one cause sat under three of the four
reports: a value's relationship to its backing store was carried by something other
than its type. The right column is therefore mostly one concept, which is the point.

```text
| finding                                                        | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| F1-1 BREAKS a provider-derived value in a field has no origin,  | [PROV-1]: the store is in the type, so a field carries it;  |
|   and per-leaf provenance is deferred                           | [STOR-5]'s deferral is withdrawn as unnecessary             |
| F1-2 BREAKS [CNT-5] and [PROV-6] disagree; one answer is an     | [CNT-5] deleted; [PROV-6] states one disposition for every  |
|   invisible free of every element                               | type and disposes structurally                             |
| F1-3 BREAKS one store per activation, one lifetime per region   | [PROV-5]: reservation is an event of the region block, with |
|   block, and a loop enters the block twice                      | a reset release action on every edge leaving it            |
| F1-4 BREAKS [VIEW-3] step 4 names a datum no producer mints,    | [MSR-3]: one former keyed on (point, place, measure); the   |
|   and the worked reading publishes a length one too large       | view carries its formation call's datums, which [VIEW-3]    |
|                                                                 | names                                                       |
| F1-5 GAP the singleton check is unobtainable in a helper and    | [PROV-6]: the check is type equality on the store region,   |
|   not preserved through a value-in / value-out row              | so a helper discharges it and no set is preserved           |
| F1-6 BREAKS [CNT-4] refuses the inventory's own result types    | [CNT-4]: the position prohibition narrows to loan-bearing   |
| F1-7 GAP use 3 refuses a replace of a heap-backed field         | [PROV-3] use 3 narrows to loan-bearing targets; probe q2    |
| F1-8 GAP the kill relation kills on every element write         | [MSR-2]: support is descriptor storage, and the kill is     |
|                                                                 | [ENT-5]'s own overlap with no new notion; probe w4          |
| F1-9 GAP [LIV-3] is refused at the only position that           | [LIV-3] is one [SET-2] exchange; nothing consumes the root, |
|   justifies it, and its expansion kills the container's length  | so the container's own length survives; probes q3, q7       |
| F1-10 GAP [PAR-2] still denies the MutSpan fill, on the write   | [PROV-3] use 1 and use 4: an origin carries the range its   |
|   footprint rather than on the loan                             | value reaches, and [RUN-3] projects that range              |
| F1-11 GAP every provider post-state relation is two-state over  | [PROV-2] and [SEQ-0]: single-state over the call datum and  |
|   a datum that cannot be minted                                 | the live term, which is [ENT-3.S5]'s own shape             |
| F1-12 GAP [OWN-5]'s no-join sentence lost its subject and is    | registered; restated over the loan-bearing predicate, and   |
|   in neither register list                                      | the register's fourth defect condition now catches the class|
| F1-13 GAP two silences: a divergent loop's linear binding, and  | [STK-4] states the disposition (no error, a retained item   |
|   `seq_exchange` declaring pure                                 | of E); [SEQ-0]'s row reads reads/writes; probe c8          |
| F2-NA1 BREAKS a move plus a reinitializing set hands a lease to | [PROV-1]: two stores are two types, so the set is a type    |
|   the wrong store                                               | error; probe w8 is why a place could not carry it          |
| F2-NA2 BREAKS a runtime offset selects the store                | [PROV-1]: a homogeneous container of two stores has no      |
|                                                                 | element type                                                |
| F2-NA3 BREAKS linearity is viral upward and disposal is         | [PROV-6]: disposal is closed under containment exactly as   |
|   leaf-only, so a container of leases cannot leave a scope       | linearity is, and drains what it finds                      |
| F2-NA4 BREAKS the virality clause is unsatisfiable in a helper  | [PROV-6]: type equality, discharged inside the callee; no   |
|                                                                 | `disposes` effect category is needed and none is added      |
| F2-NA5 BREAKS an arena's cap dies at its first allocation       | [PROV-1] puts BYTES in the type and L15 requires cap' = cap |
|                                                                 | on every writing row                                        |
| F2-NA6 GAP no ceiling data for any added nominal                | [RES-5]: [OP-9]'s table gains a pair per nominal and its    |
|                                                                 | region-bearing exclusion is lifted                          |
| F2-NA7 BREAKS [RES-7]'s premise is denied by [SYS-2]; the list  | [RES-7]: exclusion by property, which excludes nothing in   |
|   is not closed; the covered store has no vocabulary            | this version; the handle table gains a measure, a domain,   |
|                                                                 | a profile cap and a typed refusal                           |
| F2-NA8 GAP [STK-1]'s premise is over a target-stage object      | [STK-1] is split into a source premise and an ABI           |
|                                                                 | obligation, landing at [RES-3] stage two                    |
| F2-NA9 BREAKS a per-occurrence extent item cannot be per        | [PROV-5] refuses the multiplicity it cannot count; 1.4      |
|   activation                                                    | marks the row as owed rather than inherited                 |
| F2-NA10 BREAKS 1.4's "nothing has to be reopened" is false      | 1.4 rewritten: three rows inherited, three owed, and the    |
|                                                                 | cooperative queue's three limits stated plainly             |
| F2-NA11 GAP [RES-2]'s "two figures per item" is false for the   | [RES-2] says which items carry a source-stage figure; the   |
|   stack item                                                    | stack item does not, and stage one's stack content is        |
|                                                                 | premise 2                                                   |
| F2-NA12 GAP the par apparatus is checked where it cannot run;   | [RUN-2] is restated over permission, which is what v0.40    |
|   [RUN-2]'s subject does not exist                              | has; [RUN-1]'s scope is stated for the unmarked build       |
| F2-NA13 GAP the loop route over-refuses and the interval        | 3.3.1: route (ii) is over the acquisition, and the interval |
|   arithmetic is unstated                                        | enters the peak as its max on every rule                    |
| F2-13 compiler defect: [SET-2]'s arena half is unenforced       | recorded, probe w2; B6 must implement [PROV-3] use 3 as a   |
|                                                                 | relation, not as a CheckedType test                        |
| F3-1 DEFECT the datum has one producer and five consumers       | [MSR-3], as F1-4                                            |
| F3-2 DEFECT [VIEW-4]'s ground is the sentence [LIV-2] deletes   | [VIEW-4] restated over [LIV-2]'s own premise and use 3, and |
|                                                                 | carried as a `Depends:` row                                 |
| F3-3 DEFECT a user multi-return publishes nothing at the caller | [CALL-4]: one added [ENT-3.S12] destination clause, each    |
|                                                                 | binder at its own ordinal                                   |
| F3-4 DEFECT [GRAM-11] is in the unchanged list and must change  | registered as a fourth callee class, with [OP-1] 833        |
| F3-5 DEFECT the provider operations are in two domains          | box_new and arena_new retire from [OP-1]; heap_take and     |
|                                                                 | arena_take are container-domain rows                        |
| F3-6 DEFECT every clause in the file is in a shape [GRAM-5]     | [MSR-5]: a clause_expr production, `atom` untouched;        |
|   does not derive, and [GRAM-5] is unregistered                 | probe w3                                                   |
| F3-7 DEFECT len + room = cap has no home                        | [MSR-2]: two specification-fixed members of [ENT-6] 3001's  |
|                                                                 | automatic affine-premise sequence                          |
| F3-8 DEFECT [LIV-2]'s admission and judgment disagree           | one premise: dead at the commit point; probes p10 and w6    |
| F3-9 DEFECT [LIV-3] contradicts itself about subscripts         | [SET-2] exchange, as F1-9                                   |
| F3-10 DEFECT [OWN-5] 603 is unregistered and [CALL-3]           | registered: 603 gains the callee-side twin of the [SET-1]   |
|   contradicts it                                                | change, its second sentence unchanged                      |
| F3-11 DEFECT `slot<'p, T>` is unspellable                       | `PoolSlot<'s, T>`, an ordinary TYPEID, with `HeapBox` and   |
|                                                                 | `ArenaBox` beside it                                        |
| F3-12 DEFECT [FN-3] is unregistered and [PROV-4] deletes its    | registered: the allocation component becomes the set of     |
|   vocabulary                                                    | allocates paths under 1121's own identity                  |
| F3-13 DEFECT `seq_exchange` declares pure and writes            | reads(vector), writes(vector); probe c8                     |
| F3-14 DEFECT the four reserving rows carry no effect row and no | written into the inventory as pure rows with declared       |
|   relations                                                     | relations, which is what makes 4.1's entry pure             |
| F3-I1 four wrong register line numbers                          | re-derived this session: [OWN-4] 577, [OWN-5] 601, [OWN-9]  |
|                                                                 | 633, [FN-7] 1211-1246                                       |
| F3-I2 six stale [RES-5]/[RES-6] citations                       | corrected in 1.4, [RES-2], [RES-3], [RES-4] and both        |
|                                                                 | companions                                                  |
| F3-I3 [PROV-7] states no Publishes                              | it states one                                               |
| F3-I4 two "unchanged" rows sit in the changed table             | the register's fourth condition replaces them: a depended   |
|                                                                 | sentence inside changed text is recorded on the changed row |
| F3-I5 four rows carry content no Amends line supplies           | re-derived from the rules                                   |
| F3-I6 META-5 double-counts `arena` and omits four counts        | recomputed, with production, entry-row and domain counts    |
| F3-I12 the Result rows name no designated outcome               | [SEQ-0]: a single Result or Option result designates itself |
| F3-I13 Pool and Arena have no release action                    | [PROV-5]'s store reset, in [STOR-3]'s table                 |
| F3-I9 [RES-8] claims cross-unit composition                     | dropped to the one closed unit [PROG-1] has                 |
| F3-I11 append_span versus collect                               | one name, `collect`, here and in both companions            |
| F3-I15, I16 two stale companion residues                        | fixed in `RESOURCES.md`                                     |
| F3-I18, I19 section 7's batching                                | re-derived; every rule in exactly one batch and no test      |
|                                                                 | needing a later batch                                       |
| F3-N1, N2, N3 probe count, Q8 count, absorb's vocabulary row    | corrected                                                   |
| F4-1 BLOCKING a linear container has no disposal                | [PROV-6], as F2-NA3                                         |
| F4-2 BLOCKING no preservation clause for a field, an element    | [PROV-1]: preservation is a closure property of type        |
|   or a payload                                                  | formation, so no clause is needed                          |
| F4-3 BLOCKING disposal is non-modular                           | [PROV-6], as F2-NA4                                         |
| F4-4 BLOCKING propagate and linearity are mutually exclusive    | [PROV-6] says so, with a diagnostic and a mechanical fix;   |
|   and no rule says so                                           | probe w5; Q10 asks whether to relieve it                    |
| F4-5 BLOCKING no sentence says what an invariant atom over an   | [MSR-3]: keyed by the [ENT-2] term, re-established on the   |
|   updated binding is keyed by                                   | backedge from the call datum; 4.1 writes the derivation     |
| F4-6 FRICTION [LIV-3]'s admission contradicts its meaning       | [SET-2] exchange, as F1-9                                   |
| F4-7 FRICTION update reaches only single-result rows            | `update p by op(...) into x;`, which covers every try row   |
| F4-8 FRICTION no helper may hold a container                    | not changed, and the reason is stated: admitting `&uniq V`  |
|                                                                 | with a restricted operation set reopens D1's shape at the   |
|                                                                 | one place [CNT-7] closes it. [CNT-7]'s restructuring text   |
|                                                                 | now branches per owner, which was the false half            |
| F4-9 FRICTION no construction loop publishes len = N            | `seq_vacant<T, N>`; probes n14, n15, n19                    |
| F4-10 FRICTION [VIEW-7] costs two items per site, and arg_get   | Q11 states the relief and its argument; [RES-7] restores    |
|   is wfgrep's unremovable blocker                               | arg_get, so walk's recursion is the only blocker            |
| F4-11 FRICTION three text defects                               | 3.11's call spelling fixed; [MSR-5]'s subscript example     |
|                                                                 | dropped with its reason; [VIEW-2]'s admission justified for |
|                                                                 | both view kinds                                             |
| F4-12 FRICTION the replace-swap dummy, and no message for       | `update ... into` drains a nested container; the            |
|   seq_filled with an affine T                                   | AffineFillElementType diagnostic points at seq_vacant       |
```

Findings the reports rated HOLDS or CLEAN, preserved and not weakened: [CALL-1],
[CALL-2], [CALL-3] and [CALL-5] survive every shape attacked, in all three rounds,
now including the region-parametric-nominal route. [LIV-1] is complete for [FN-1]'s
conservative graph and round 3 tried six more routes. [CNT-4]'s quantified
confinement and its nominal invariance are total and fail closed. [VIEW-3]'s
singleton-**resolved**-set requirement is the right repair of round 2's F3-4 and
round 3 could not manufacture a second resolved origin. [MSR-4]'s one disposition
survived a fourth set of programs. [PROV-7]'s loan-region condition admits every
provider-consuming row. [MSR-2]'s descriptor definition gives the right answer for
a `deref` chain, a subscripted descriptor and a nested `absorb`. `seq_take_at`,
`seq_exchange` and `FixedRing`'s rotation create no vacancy state, and `array<T, N>`
and `fir_filter` pay nothing.

---

## 7. Implementation order

Twelve batches, re-derived from the rules this draft states. Each names the rules
it implements and the test it adds, **every rule of section 3 appears in exactly
one batch**, and **no batch's test needs a later batch's rules**; round 3 found two
tests of B3 that needed B5. This is an ordering, not a design choice; nothing here
may be read as trading a rule away for a cheaper batch, and nothing here is an
approval or a schedule.

Three hard constraints the ordering obeys. The operation inventory is written in
the syntax B3 introduces, so multi-return and the transformation statement come
before any operation that returns two results. The container domain is **generic**,
and probe `r2_3` shows user generics are `Semantics/Unsupported` today, so B5 is
the batch that first needs monomorphization for a compiler-owned domain and it must
carry that work. And the store brand is a region in a type, so every batch that
introduces a branded type is after the batch that introduces the brand.

**B1. Type-derived call transports, and the retirement of container state mutation
through `&uniq`.** Rules: [CALL-1], [CALL-2], [CALL-3], [CALL-5], [CNT-7]. First
because it is the live defect and because it needs none of the new types: today's
`&uniq buffer<T>` keeps its spelling and gets [CALL-5]'s type-derived
classification, `element = false`, which is exactly the sweep's minimal sound
repair. Test: **`ent5-neg-callee-uniq-buffer-replace-kills-length.wf` turns XPASS**,
rejecting at [OP-4] with residual `9_u64 < len(line)`; plus one positive case
pinning [CALL-1]. `docs/patterns.md` P16 is corrected in the same change.

**B2. The proof surface.** Rules: [MSR-1], [MSR-2], [MSR-4], [MSR-5]. Second
because every later batch's contracts and invariants are unwritable without it, and
because it is a specification amendment with no new construct. Tests: a conformance
pair mirroring `v23`/`v24` (both accepted after the amendment); one mirroring
`v25`/`v26` so two consumers of one exported invariant agree; one mirroring probe
`w3`, a clause whose operands are two `len` terms, accepted where it is a [GRAM-9]
parse failure today; one pinning that a literal and a parenthesized group are still
affine factors; one discharging a goal from `len + room = cap` as an affine
premise; **and `r2_4`'s program accepted**, because [MSR-2]'s descriptor-precise
support is a repair of a live over-kill and not only a new rule.

**B3. Multi-return, the destructuring `let`, join-checked liveness, and the
transformation statement.** Rules: [CALL-4], [LIV-1], [LIV-2], [LIV-3]. Third
because B5 and B7 are written in this syntax. Tests, all writable in today's
vocabulary plus B1 and B2: probe `p8`'s signature parses and binds, and a `v0.40`
helper's two-result `ensures` reaches both binders of a destructuring `let`, which
is [CALL-4]'s added S12 clause under test; probe `p10`'s program and probe `w6`'s
are both accepted after [LIV-2]; probe `f3`'s program is a [LIV-1] error naming both
predecessors instead of `SemanticUnsupported`; a loop moving and restoring an outer
binding is accepted where probe `f5` is [OWN-11] today; probe `r2_8`'s `update`
parses, and an `update` at a `buffer` element place is accepted where probe `q7`'s
`set` spelling is [OWN-1] today, which is the exchange judgment under test; and the
`set` spelling of a container-domain call is a [FORM-1] rejection naming `update`.

**B4. Measure datums and images.** Rules: [MSR-3]. Separated from B2 because it
touches [ENT-2]'s term list, [ENT-5]'s call boundary and [ENT-6]'s transfer
machinery rather than route lists, and because it needs [LIV-2] from B3. Tests, all
writable in today's vocabulary plus B2 and B3: a `buffer` helper whose `ensures`
names `len` of a parameter it consumed is accepted; the same helper's caller
establishes the declared relation on the result where `M(c,q)` refuses it today; a
relation over a **borrowed** owner's measure establishes at the caller, which is the
call placement's borrow half under test; a reinitialized binding does not inherit a
fact stated over its predecessor; an image is unavailable after a projected callee
write, pinning `g1` against `g1b`; and a header invariant over a binding an `update`
rewrites is preserved on the backedge, which is 4.1's three-step derivation under
test.

**B5. The store brand, owners, typestate, confinement, and the declaration
domain.** Rules: [PROV-1], [CNT-1], [CNT-2], [CNT-3], [CNT-4], [CNT-6], [SEQ-0] and
the constructor, place, take, exchange, clear and ring rows. Retires `buffer<T>`
from the writer surface. Carries monomorphization for a compiler-owned generic
domain. [PROV-1] lands here rather than in B7 because the owners are the first
branded types and because [CNT-4] and [PROV-1] are one confinement story. Tests: a
`FixedVector<Handle, 64>` object table with affine elements, filled by `seq_vacant`
and compacted by `seq_take_at`, accepted, where probe `p9` is [OP-1] today; a
`seq_vacant` result whose `len = N` discharges a subscript with no invariant, where
probes `n14`/`n15` show no loop can; a `FixedRing<Descriptor, 64>` read and written
by subscript; `struct Chunk['s]` accepted where probe `r2_6` is a parse error today,
with two instances at different regions rejected as distinct types; and **two
reserving occurrences naming one region rejected at the second**, which is
[PROV-1]'s own check. This batch supersedes B1's conformance case, whose program no
longer typechecks; that disposition is conformance evidence and is recorded in
`governance/APPROVALS.md` with the merge.

**B6. Views, loans, ranges, and the commit event.** Rules: [VIEW-1] to [VIEW-6],
[PROV-3], and the view rows of [SEQ-0]. [PROV-3] lands here because views are its
only user and because [SET-1] and [SET-2] must change in the same batch that admits
the `MutSpan` write. Tests: an element write through a `MutSpan` is accepted where
probe `p7` is [SET-1] today; **a `replace` through `&uniq MutSpan<'r,u8>` is
rejected**, and so is a `replace` of an `ArenaBox` place, which probe `w2` shows the
compiler accepts today, so use 3 must be implemented as a relation over loan-bearing
types and not as one `CheckedType` test; two `AppendView`s on one owner are rejected
at the second formation citing [OWN-5]; a write to `k` while a view formed at
`table[k]` is live is rejected citing the view's loan; an owner is readable
immediately after `absorb` with no enclosing region, and an `absorb` after an
operation that shortened the owner publishes the **formation** length plus the
commit; and a two-result signature with two same-region view results is rejected at
[VIEW-6].

**B7. Stores, the heap as a value, and structural disposal.** Rules: [PROV-2],
[PROV-4], [PROV-5], [PROV-6], [PROV-7], [RES-6], and the provider-bearing [SEQ-0]
rows. Tests: probe `p5_ambient`'s program is **rejected**; a `main` that omits
`command.heap` cannot reach any allocation; probe `r2_5`'s and probe `w7`'s programs
are rejected with [PROV-6]'s diagnostic and their repairs compile; a lease released
to a pool of a different store region fails to typecheck, with the two types
rendered; `dispose` of a `FixedVector<Chunk<'s>, 8>` compiles and frees every leaf,
and the same statement with a missing provider is `DisposeProviderMissing`; probe
`w5`'s program is rejected with `LinearValueAcrossPropagate` and its `match` repair
compiles; a region block entered twice by a loop republishes `len(store) = Z`
truthfully, which is [PROV-5]'s reset under test; a helper lending a provider onward
to `pool_take` compiles, where `r1_relend` is [OWN-6] today; and two overlapped
disposals from one store are denied [PAR-1] permission.

**B8. System I/O over views.** Rules: [VIEW-7]. Test: `tests/programs/wfgrep.wf`
migrated to `seq_filled` and `MutSpan`, compiling with no `allocates` entry
anywhere on its call graph. It is the first program that demonstrates goal A's
container half end to end, and the migration is also the measurement Q7 and Q11
need.

**B9. The stack judgment.** Rules: [STK-1], [STK-2], [STK-3], [STK-5]. Tests:
probes `f2b_tail` and `f8_tailframe` are **not** rewritten by [STK-1]'s source
premise and are rejected by [STK-2] under the marker; their borrow-free variants
are rewritten and accepted; a member holding a live confined value across the jump
is likewise not rewritten; a member that opens a region for a `pool_frame` is not
rewritten, which is the recorded cost under test; probe `p3_rec` stays accepted
without the marker; and a `--stack-ledger` run reports one chain per context rather
than disjoint roots.

**B10. The divergent entry.** Rules: [STK-4]. Small and separable, and it is the
batch a kernel writer notices first. Tests: probe `f3_forever`'s idle loop is
accepted; **probe `n3_propagate_loop`'s driver loop is accepted**; a loop with a
reachable `break` still requires a return; and a linear binding live on a path that
reaches only a divergent loop is accepted and appears as a retained item of the
published map, which is [STK-4]'s stated disposition under test.

**B11. The envelope and the judgment.** Rules: [RES-1], [RES-2], [RES-3], [RES-4],
[RES-5], [RES-7], [RES-8], [RUN-1], [RUN-4], [RUN-5]. Tests: section 4.1's program
is source-resource-closed and its `E` table matches a pinned symbolic expectation;
section 4.2's is reported not resource-closed with the heap-reaching path rendered;
a retaining loop whose trip count is a runtime value is rejected at that loop with
the value named; a retaining loop whose checked refusal rejoins the backedge is
**accepted**, which is the repaired route (ii) under test; a loop whose only
discharge is the standing `len <= cap` is rejected; a marked program that opens a
file composes its handle demand and is rejected when it exceeds the profile cap; and
a program whose runtime demand exceeds every profile row fails **target
qualification** citing no language rule.

**B12. `par` and the envelope.** Rules: [RUN-2], [RUN-3]. Tests: a `seq_filled`
plus `MutSpan` plus counted subscript fill receives [PAR-2] permission in an
unmarked program, which needs the ranged origin and which neither earlier draft
could pass; the same loop inside a `resource_closed` entry receives no permission
and the published row reads `lanes(1)`; two overlapped statements allocating from
distinct providers are permitted and two from one provider are not; and the `par`
rule of 3.3.1 composes against a pinned profile row for an unmarked program.

Two items sit across the batches. **Monomorphization** for a compiler-owned generic
domain is B5's, and nothing before B5 needs it. **Q5's continuation redesign** is
the largest engineering item any of this implies; [RUN-2] lets B11 and B12 ship
without it at the cost of `lanes(1)`, and lifting that restriction is a batch of its
own after B12.
