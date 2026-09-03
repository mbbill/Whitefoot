# Containers and resources: the integrated design

The single design for batch 0116. It merges the two drafts beside it,
`RESOURCES.md` (providers, the envelope `E`, the `resource-closed` judgment) and
`CONTAINERS.md` (owners, views, and the facts that cross a call), into one set of
laws, one set of rules, one vocabulary, and one amendment register. A reader who
has not read either draft can read this file alone. The drafts remain for their
detailed rationale, their rejected alternatives, and their migrations; every rule
they stated normatively lives here.

**Third draft, after falsifier round 2.** Round 2 found as many breaks as round 1,
and most of the new ones were opened by round-1 repairs: a rule added per finding,
each sound in isolation and none checked against the others. Provider identity was
carried by a region that named many stores; a total release rule had rows that
could not name their store; a reborrow amendment admitted the heap and refused
every pool; a divergence amendment was stated over the wrong quantity; and the
amendment register was written by hand beside rules that did not carry the fields
it was supposed to be derived from.

This draft is therefore a consolidation and not a fourth layer. Six concepts do
the work that eighteen round-1 clauses were doing: one origin-set provenance for
views and provider-derived values alike, one immutable measure datum on both sides
of a call boundary, region-local reservation with explicit disposal, one
descriptor-precise support relation for measures, one declaration domain for the
container operations, and one transformation statement. The rule count falls from
sixty-five to fifty-three, every rule states its judgment, what it publishes and
what it amends, and section 3.13 is a collation of those `Amends:` lines rather
than a parallel document.

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

Two footnotes on that list, because falsifier round 2 read it against the rules
and found two seams. [CNT-1] declares a fifth owner, `FixedRing<T, N>`; the
settled list names the four prefix owners and excludes no rotation. And the
settled append example writes its source argument first; [GRAM-11] fixes argument
order from the declaration, and every helper in this file declares its owner
first, so the same call is spelled with the owner leading. Neither is a decision
reopened.

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
delivers the first four and not the fifth: [SCOPE-3] (27-31) leaves heap
exhaustion, stack exhaustion, operating-system quotas and runtime-start resources
outside the source outcome model, so an accepted program may stop at the host
boundary with no Whitefoot value, no status, and no cleanup. A program that can
vanish at three in the morning has not removed the class of failure the writer came
here to remove.

The owner's shape for goal A is a **promise**, not a guarantee about the world.
The compiler computes one finite, shaped envelope `E`; the program promises never
to demand more than `E`; the environment then decides whether it can deliver `E`.
Only the conjunction gives freedom from exhaustion, and a program that reaches the
heap makes no such promise, because total free bytes cannot answer a request for a
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

### 1.3 What the design therefore has to do

Turn every resource a program can exhaust into a value it must hold in order to
consume **and in order to release**, so that "this subtree cannot touch the heap"
is a signature fact and "this program's peak demand is this list of extents and
slot counts" is a compiler judgment. Give the writer one declaration that turns
the second into a compilation requirement. Make every failure to obtain a resource
a typed value that returns the affine inputs it did not consume. Put the runtime
inside the same envelope as the writer's code. And make every fact that survives a
call readable from the callee's declared parameter modes, declared types, and
declared contract, so D1 has no expressible form.

The first draft did all of that and was unusable, because it had no answer to
*what is a length, arithmetically?* The second draft answered that and was unsound,
because it had no answer to *what identifies a store, and what identifies a
measured value across a consume?* Section 3.1 and section 3.2 answer those two
first, and everything else is built on them.

### 1.4 What this design does not decide: execution contexts

A scheduler that switches contexts, an interrupt handler, and a per-task kernel
stack are **out of scope for this batch, by the orchestrator's ruling**, and this
file states the fact rather than filing it as an open question. No source construct
in v0.40 or in this design creates, enters, or switches an execution context;
`program_kind := "command"` is the whole production (177) and [FN-7] admits exactly
one entry, so an `interrupt fn` does not parse and [STK-4]'s round-2 reentrancy
refusal had no expressible instance. Program 4.1 is written accordingly: it is a
cooperative run queue of state machines that advance on one chain, not a scheduler
that switches stacks.

The follow-on design that lands execution contexts inherits a fixed interface from
this one, and nothing here has to be reopened when it does:

```text
| this design fixes                | what a context switch must preserve                     |
|----------------------------------|---------------------------------------------------------|
| E carries one stack item per     | a new context is a new item of E, measured by [STK-3]    |
| execution context [STK-3]        | over its own whole chain; creating one is an acquisition |
|                                  | of that item and is covered [RES-1]                      |
| provenance is a static origin    | a value's provider origin is unchanged by a switch; a    |
| set [PROV-3]                     | context may not resolve an origin its own frames do not   |
|                                  | reach                                                    |
| a reserved extent is region-local| a context switch may not make a confined value outlive    |
| [PROV-5]                         | the region block whose activation holds its extent        |
| envelope accounting is per       | the per-domain map of 3.3.1 composes per context; a       |
| domain and peak-based [RES-6]    | switch transfers no peak and creates no new domain        |
| disposal is explicit [PROV-6]    | a context that dies with a live linear value is the same  |
|                                  | error [LIV-1] reports at a scope exit                     |
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
Because acceptance may not depend on a register allocator or a linked runtime, and
because a bound that names a runtime value is not a bound: owner ruling R13
(`L7036`), B8, [SCOPE-2] 18, [STOR-6] 745.

**L2. No resource is ambient.** *Every covered resource enters the program as a
capability value the runtime hands to `main`, or as a store the program reserves
in an activation it owns, and travels only by ordinary ownership; there is no
ambient allocator, thread source, or stack pool.*
Because an effect row describes what a body did while a held value is an authority
it had, and only the second makes heap-freedom a signature fact: probe `p5_ambient`
allocates while holding nothing and is **accepted today**; [FN-7] 1242's "there is
no ambient system state" loses its last exception.

**L3. Nothing fails silently, and nothing grows behind the writer.** *Every
operation that can fail to obtain a covered resource returns a typed value naming
the failure and handing back every affine input it did not consume; no operation
traps, aborts, retries, falls back, or promotes a store to a larger one.*
Because v0.40 claims zero writer-reachable runtime-trap families (spec line 6)
while heap exhaustion still ends a process with no source value: owner ruling R12
(`L5657-5666`), B3, audit answer Q8.

**L4. No hidden growth.** *No operation both uses existing capacity and acquires
new capacity; every operation that may acquire capacity takes an owner and a
provider, names its allocation effect, and returns a typed failure, while every
operation that only uses existing capacity is total under a proved capacity
requirement and can allocate on no path.*
Because one `push` cannot carry one return type and one effect row across backings:
owner ruling R5 (`L2332`), B2, B3, X1.

**L5. The runtime is inside the envelope.** *The artifact `E` describes is the
writer's code, the compiler-derived cleanup and drop glue, the `par` runtime, and
the target adapter together, from the frame the environment hands the program to
the frame it takes back; a resource any of them needs is an item of `E`, or the
program is not resource-closed on that target.*
Because a guarantee that stops at the edge of generated code is not one, and the
existing `--stack-ledger` reports the entry chain as disjoint roots: owner ruling
R12, B12, the ledger read in 6.1.

**L6. Shape, not bytes.** *`E` is a list of tangible resources (contiguous aligned
extents, per-class slot counts, per-context stacks, lane counts) and never one byte
total. A store the program itself reserves is shaped by the same rule: a reserving
operation that needs an alignment or a separately grantable extent produces its own
`region` item and is not folded into a stack total.*
Because sixteen bytes holding four four-byte objects, with the first and third
released, cannot serve an eight-byte request, and because a deployment reading one
stack number cannot tell an alignment failure from a size failure: owner ruling
R12, B9, B11.

**L7. Lowering before judgment, and a tail call is a dead caller frame.** *Tail
recursion, including mutual tail recursion, is rewritten into loops before any
resource judgment runs; an intra-component call edge is a tail edge exactly when
the caller's activation record is dead at the jump, and never because the call is
written in a return statement.*
Because an optimization that may or may not fire cannot be a premise of a
guarantee, and syntactic conditions cannot see a chain member holding a live loan
into its own frame: owner rulings R3 (`L989`) and R12, B10, probes `f2b` and
`f8_tailframe`, **accepted today**.

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
whole life, a loan of its own strength on every place in its resolved origin set,
beginning at formation and ending when the view value is consumed or released; a
function that changes a view's state consumes it and returns the new one. A loan
covers every binding the address computation of its place reads, for the loan's
whole life.*
The first clause answers write-back without a hidden protocol, the second is what
the first draft asserted and no rule supplied, and the third is what round 2 found
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
With no per-slot state the checker never needs a quantified proposition over slots.
Occupancy a writer wants at a stable index is ordinary data, not typestate:
`FixedVector<Option<T>, N>` with element-position `replace` is that program, and
probe `r2_7` compiles its shape today. Owner's settled decision; audit answers Q2,
Q4, Q10.

**L13. Acquisition and release are symmetric, and both name the store.** *A value
whose backing is reclaimed per value is **linear**: it has no compiler-derived
release, and it leaves a scope only by being moved out or by being consumed by a
disposal operation that takes the same provider its acquisition took, identified by
the value's own provenance rather than by its type or by a parameter. A value whose
backing is reclaimed with a region or with a frame is not linear and keeps its
ordinary compiler-derived release. No source construct selects, replaces, or
observes a release action.*
This replaces the second draft's derived-release-with-a-provider-row, which round 2
broke twice, once because a released value's type names a region and a region may
hold many stores, and once because the bulk-drop rows had no provider formal to
name; and it removes an invisible free, which probe `r2_5` shows the language has
today. B2's drop order, audit answer Q10, [STOR-3] 683, [EFF-2] 1421.

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
publishes the exact new capacity.*
The first draft forbade reading `cap` and `room` on a rationale that only forbids
reading the allocator's size, so every pop proved and no push did: B3 read as
written, audit answer Q9, probes `q24`, `v25`, `v26`.

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
says whether it should run. A linear binding [L13] that is live on an edge leaving
its scope is the error, because no derived release exists to carry it.*
The reinitializing `set` makes liveness non-monotone, and [OWN-11] and today's
`Semantics/Unsupported: OwnershipJoin` are two ways of avoiding the question rather
than answering it; the same per-edge check is what makes linear disposal checkable
rather than hopeful. Probe `f3`; [ENT-5]'s own all-predecessor join.

---

## 3. The rules

Ten families and fifty-three rules. `[MSR]` is the measure terms and the proof
surface, `[PROV]` the capability values, their provenance and their disposal,
`[RES]` the covered set, the envelope and the judgment, `[STK]` the stack, `[RUN]`
the runtime's own closure and the environment's half of the bargain, `[CNT]` the
sequence owners and their typestate, `[VIEW]` the views and the commit event,
`[LIV]` affine liveness and the transformation statement, `[CALL]` what survives a
call, and `[SEQ]` the operation inventory.

**Every rule states four things: the judgment it creates, the fact it publishes,
what it amends, and its law.** A rule that creates no judgment writes
`*Judgment:* none` and says what it is instead. Section 3.13 is a **collation of
those `Amends:` lines and carries nothing else**: it is written last, from the
rules, and a register row with no rule behind it, or a rule whose `Amends:` line
no row carries, is a defect of this file. Round 2's largest finding was that the
second draft's register was neither.

The family is `[PROV]` and not `[CAP]` because [CAP-1] already exists (1962) and
rule ids are never reused. The collision is worth a sentence: [CAP-1] says the
kernel defines *no writer-visible capability category and no system-specific
permission*, and this design does not add one. A provider is an ordinary affine
value, held under `own` or `&uniq`, judged by place overlap and by the ordinary
effect row. "Capability" here means *a value you must hold in order to act*, which
is what `FilePermit` already is.

Two families the first draft had are gone. `[BLD]`, the `par` builder, is deleted
outright; its ids are retired and not reused. The second draft's separate view
provenance rule is gone too, merged into [PROV-3], because provenance is one
mechanism and views were only its first user.

### 3.1 `[MSR]`: measures, and the one goal disposition

This family is first because everything else consumes it. It adds no statement
form and no type; it is a specification amendment.

**[MSR-1] Three measure terms, over one place, for every measured value.**
`len(P)`, `cap(P)` and `room(P)` are terms of the [ENT-2] term language, of
fragment type `u64`, where `P` is an admitted place. They are defined once, here,
for every *measured* type, and which measures a type has is table data rather than
a rule with an exception:

```text
| measured type          | len                  | cap                 | room          |
|------------------------|----------------------|---------------------|---------------|
| array<T, N>            | N                    | N                   | Z             |
| the prefix owners      | initialized elements | slots               | cap - len     |
| FixedRing<T, N>        | queued elements      | N                   | cap - len     |
| Span, MutSpan          | viewed elements      | len                 | Z             |
| AppendView             | appended elements    | the window          | cap - len     |
| Arena<'p>              | cursor bytes         | extent bytes        | cap - len     |
| Pool<'p, T, N>         | live slots           | N                   | cap - len     |
| Heap                   | none                 | none                | none          |
```

`Heap` has no row because L6 says a general store has no measure that means
anything; that is the absence of table data, not an exception clause on a total
definition.

An admitted place for a measure term is a `place` [GRAM-5] formed with field
selections, `deref` wrappings **and subscripts**, whose final selected type is a
measured type. The subscript admission is the change: `len(table[i])` is a term,
so a container of containers has provable operations. A subscripted place's own
[OP-4] obligation is judged independently and is not weakened by occurring under
a measure term.

*Judgment:* none by itself. *Publishes:* the term. *Amends:* [ENT-2] clause (b),
which today admits `len(P)` only for `array`, `slice` and `buffer`, and only for
subscript-free places. *Law:* L16.

**[MSR-2] Support is the descriptor, and a kill is an event that writes it.** The
support of a measure term over `P` is:

- `P`'s **descriptor**, which is the place `P` itself; a place that strictly
  extends `P` through a subscript names `P`'s element storage and is not `P`'s
  descriptor;
- every borrow, `box`, `arena` or `slot` holder any prefix of `P` reads through;
  and
- the support of **every** offset occurring anywhere in `P`, not only the last.

A measure term over `P` dies exactly on an event whose written place overlaps
`P`'s descriptor under [OWN-7], where an event is any [SET-1] commit, [SET-2]
commit, consume, scope exit, or **any action carrying a `writes` occurrence that
projects onto that place under [EFF-2]**, a call and a compiler-derived release
alike. Stating the kill over the effect row rather than over a list of syntactic
forms is what keeps it closed when a later family derives a new action.

Three consequences, and each answers a round-2 finding.

- A write to a **sibling field** does not kill. `len(deref(ring).flags)` has
  descriptor `deref(ring).flags`, and `deref(ring).tail` is neither a prefix of it
  nor extended by it, so `set deref(ring).tail = 1_u64;` kills nothing. Probe
  `r2_4` shows today's compiler kills it and `r2_4b`/`r2_4c` bound the behavior:
  the current implementation is root-granular where [EFF-2] on the same statement
  is field-precise, and this rule makes the measure use the precision [EFF-2]
  already computes. This is the same move [PROV-4] makes for `allocates`.
- An **element write** does not kill, which is [ENT-5]'s existing sentence obtained
  from the descriptor definition rather than asserted as an exception.
- A write to an **offset** does kill, at every level of the projection, so a fact
  over `len(grid[i][j])` dies when `i` is written and not only when `j` is.

At every program point at which `P` is live, these hold implicitly:

```text
Z <= len(P)          Z <= room(P)          len(P) <= cap(P)
len(P) + room(P) = cap(P)
cap(P) = N           for a type whose capacity is the constant N
```

The first three are difference bounds and live in L0. The fourth is a three-term
identity and lives in the affine domain, where [INV-1] already carries relations
of that shape; it is not copied into L0, whose uniqueness and finiteness argument
[ENT-4] 2854 rests on the difference-bound shape.

*Judgment:* none. *Publishes:* the implicit facts. *Amends:* [ENT-2]'s
implicit-fact sentence (2722) and [ENT-5]'s support and kill sentences
(2857-2887), whose length-term support becomes the descriptor relation above and
whose kill classes (a) through (d) gain the effect-row statement. *Law:* L16.

**[MSR-3] Measure datums, and where an image dies.** A **measure datum** is a
compiler-owned immutable [ENT-2] term of fragment type `u64` with **empty
support**: no [ENT-5] event kills it, no place occurs in it, and no later write
retargets it. It is the device [ENT-2] already has for a `for_stmt` capture and a
[SET-1] commit value, extended to one more producer. Exactly two producers exist:

```text
entry datum        for each parameter of measured type and each measure it has,
                     identified by (concrete function instance, parameter ordinal,
                     measure); denotes that parameter's measure at body entry
pre-transfer datum for each own operand of measured type at one call, and each
                     measure it has, identified by (call NodePath, formal ordinal,
                     measure); denotes that operand's measure at the pre-transfer
                     point [ENT-5]
```

**At its formation point a datum is established equal to the term it denotes**,
which is the one L0 fact that ties it to the program: `<entry datum of len(p)> = len(p)`
at body entry, and `<pre-transfer datum of len(a)> = len(a)` at the pre-transfer
point. Every later kill removes the term side and never the datum side, which is
what the datum exists for.

Three rules read them and nothing else does. A [FN-9] or [FN-8] clause operand
naming a parameter's measure denotes that parameter's **entry datum**, so a
consuming use of an `own` parameter cannot invalidate it and a helper that writes
`let acc = move out;` can still state `ensures ile(len(written), cap(out))`. A
[SEQ-0] declared relation naming a parameter's measure is established at the caller
over that call's **pre-transfer datum**, so it survives the argument consume that
the same statement performs. And a move of a measured value into a fresh binding,
`let x = move p;`, publishes `len(x) = <pre-transfer datum of len(p)>` and the two
companions, which names no revivable root.

That last clause repairs a soundness break: the second draft published
`len(x) = len(p)` over the dead term `p`, whose spelling its reinitializing `set`
then revived. Under this rule the equality names a datum, and under [LIV-2] the
reinitialized binding is a distinct term.

Measures also carry [ENT-6] affine value images, formed and transferred exactly as
for a live own integer binding: an operation's declared relation over its
pre-transfer datum and its result installs the result's image, a whole-binding
`set` [LIV-2] makes the target denote that image, a join keeps an identical image
or the common nonconstant form plus one fresh delta atom, and a loop's continuing
kill replaces a loop-carried measure by a fresh header atom. **An image dies
exactly where a fact over the same term dies**: same support, same events. That
sentence is the answer round 2 demanded and did not find; probes `g1`, `g1b` and
`g7` show it is what the compiler already does for ordinary bindings.

*Judgment:* none by itself; a datum is formed, never proved. *Publishes:* the
datum, the move equalities, and the image. *Amends:* [ENT-2]'s term list (a new
clause beside its capture and commit-value clauses), [ENT-5]'s call-boundary
paragraph and its FN-9 entry-image-stability paragraph, which are replaced by the
datum rather than repaired, [FN-9]'s `M(c,q)` (a datum operand is always live) and
its parameter-entry-image sentences, and [ENT-6]'s image formation, join and
loop-header paragraphs (2970-2996). *Law:* L11, L16.

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
goal. *Amends:* [ENT-6] 3034 and 3078 (the `SubscriptBounds` and `SystemRange`
grants, which keep their normalization and lose their route grant), and [FN-9]'s
`prove_ordering` route, whose undocumented direct-affine branch becomes one of the
six steps. *Note:* this rule is why the design does not have to be revisited when
[SEQ] adds an operation: an operation adds a goal, never a route. *Law:* L16.

**[MSR-5] The contract clause language is terms, not atoms.** A `requires`,
`ensures`, `header_invariant`, `invariant_stmt` or `proof_use` operand is a
**term** of the [ENT-2] term language, not an `atom` of [GRAM-5]. [GRAM-9]'s
flat-computation rule exists to keep runtime evaluation three-address and does
not apply to erased proof syntax, which evaluates nothing. Therefore
`requires ile(len(source), room(out));` is writable directly, and so is
`invariant fill: ile(r.fill, 8_u64);` over a struct field path, and so is
`invariant order: ile(table[i], n);` over a subscript.

Correspondingly `affine_factor` [GRAM-4] **gains** two alternatives and loses
none:

```text
affine_factor := literal | ent2_place | measure_term | "(" affine_expr ")"
```

where `ent2_place` is [ENT-2] 2675(a)'s place grammar and `measure_term` is
[MSR-1]'s three formers over one admitted place. The second draft wrote that the
production was replaced rather than extended, which would have deleted the literal
and the parenthesized group and unformed every invariant in this file, [PRF-1]'s
own examples included. One consequence is a real capability gain and is stated
rather than left to arrive: [ENT-2] 2675(a) admits a named const as a tracked
place root, so a named const becomes an affine atom, which [INV-1] 3107 forbids
today.

*Judgment:* the ordinary [FN-8]/[FN-9]/[INV-1] admission over the widened operand
set. *Publishes:* nothing new. *Amends:* [GRAM-4]'s `affine_factor` production,
[FN-8]'s clause-expression judgment, [FN-9]'s operand list, and [INV-1]'s atom
sentence (3107); [GRAM-9] is unchanged and gains a stated scope. *Verified today:*
probes `q1`, `q9`, `r1_lenatom` and `r1_field` are [GRAM-4] rejections at parse,
so this is an amendment and not a compiler defect. *Law:* L16.

### 3.2 `[PROV]`: providers, provenance, and disposal

**[PROV-1] Providers, their measures, and the one `Heap`.** A *provider* is a
value of one of the compiler-known opaque nominal types `Heap`, `Arena<'p>`, and
`Pool<'p, T, N>`. A provider is affine [OWN-1], has no writer-visible component,
and is the sole authority for allocating from the store it names: `Heap` names one
general-purpose growable store, `Arena<'p>` names one contiguous extent served by
a bump cursor, and `Pool<'p, T, N>` names `N` interchangeable slots each holding
exactly one `T`. Its measures are [MSR-1]'s table rows.

`Arena<'p>` and `Pool<'p, T, N>` are *confined* types under [CNT-4]. `Heap` is not:
it is delivered as an `own` entry parameter and lives for the program. The
`command` standard-input table [FN-7] therefore gains ordinal 5:

```text
| ordinal | label        | written mode and type | supplied value                                      |
|---------|--------------|-----------------------|-----------------------------------------------------|
| 5       | command.heap | own Heap              | the one general store the runtime minted before main |
```

The row is optional like every other. A `main` that omits it receives no `Heap`
and cannot obtain one [PROV-2]. The `Heap` `main` receives is dropped on the
return edge with the **empty** release row: the store itself is the runtime's, the
program returns the handle, and no covered acquisition or release happens there.

*Judgment:* provider types are nominal and closed, and no source declaration
introduces another; plus the ordinary [FN-7] label, order, mode and type checks.
*Publishes:* the store's measures, and the whole-program fact `heap-unreachable`
when the entry row is absent. *Amends:* [TYPE-2] 352 (three added provider
nominals and `slot<'p, T>`), [TYPE-7] 471 (`slot<'p, T>` joins the closed deref
domain beside `box` and `arena`), [FN-7]'s table (1221-1227), its canonical
five-input byte sequence (1239), and its effect-row sentence (1214), whose
`allocates(heap)` becomes `allocates` over the entry's own labelled provider input.
*Law:* L2, L16.

**[PROV-2] Unforgeable, uncopyable, and taken as a loan.** No source construct
produces a `Heap`; one exists only because the runtime minted exactly one before
`main`. An `Arena<'p>` or `Pool<'p, T, N>` exists only as the result of a
reserving operation [PROV-5]. No operation duplicates, reconstructs, compares,
serializes, or derives a provider from a non-provider value.

An operation that allocates from a store, or releases to it, takes that store's
provider as a written `&uniq 'b` parameter and exhibits it. A provider is never
passed `own`: it is confined, and a moved provider is exactly the shape that
strands a lease with no reachable release target. The one `own` provider in the
language is the `Heap` the entry receives.

Every provider operation declares two regions: `'p`, the store's own confinement
region, which appears in the provider's type and in the type of anything it
produces, and `'b`, the region of the loan the call holds. They are always
distinct, and [OWN-10] is the general reason: a borrow of a local names a region
introduced **inside that binding's own scope**, and `'p` is introduced before the
provider binding exists. Probe `r2_2` is that rejection and probe `r2_1` is the
admitted shape.

```text
| op                | signature                                                                                        | effects                       |
|-------------------|--------------------------------------------------------------------------------------------------|-------------------------------|
| box_new           | ['b](heap: &uniq 'b Heap, value: own T) -> own Result<box<T>, OutOfMemory<T>>                      | allocates(heap), writes(heap) |
| box_free          | ['b](heap: &uniq 'b Heap, item: own box<T>) -> own T                                               | writes(heap)                  |
| arena_new         | ['p, 'b](arena: &uniq 'b Arena<'p>, value: own T) -> own arena<'p, T>                              | allocates(arena), writes(arena) |
| arena_new_checked | ['p, 'b](arena: &uniq 'b Arena<'p>, value: own T) -> own Result<arena<'p, T>, NeedCapacity<T>>     | allocates(arena), writes(arena) |
| pool_take         | ['p, 'b](pool: &uniq 'b Pool<'p, T, N>, value: own T) -> own slot<'p, T>                           | allocates(pool), writes(pool) |
| pool_take_checked | ['p, 'b](pool: &uniq 'b Pool<'p, T, N>, value: own T) -> own Result<slot<'p,T>, PoolExhausted<T>>  | allocates(pool), writes(pool) |
| pool_release      | ['p, 'b](pool: &uniq 'b Pool<'p, T, N>, item: own slot<'p, T>) -> own T                            | writes(pool)                  |
```

The sequence rows that consume a provider are [SEQ]'s, not this table's.
`buffer_new` and `buffer_vacant` do not appear, because [CNT-1] retires
`buffer<T>` from the writer surface entirely.

*Judgment:* a `construct` [GRAM-8] naming a provider nominal, and every other
source route to one, is a hard error citing PROV-2 at the complete `construct`,
with the restructuring `receive the provider as a parameter, or reserve one with
pool_frame or arena_frame`; and an allocation or release call whose provider
argument is missing, is not a provider place, or is not writable is a hard error
citing PROV-2 at the `call`. *Publishes:* uniqueness of the `Heap`; the provider
place each operation reaches; and the store's post-state measures, which are
[SEQ-0] declared relations over the pre-transfer datum: `len(pool)' = len(pool) + 1`
at a take, `len(pool)' = len(pool) - 1` at a release, and
`len(arena)' <= len(arena) + K<T>` at an arena allocation. *Amends:* the
`box_new`, `arena_new`, `buffer_new` and `buffer_vacant` rows of [OP-1] (793-798)
and [STOR-2] 680, which is the rule that defines `box_new` and `arena_new` and
which now gives both a provider parameter, a `Result` result and a `writes` row.
*Law:* L2, L3, L4, L16.

**[PROV-3] Provenance: one origin set for views and for provider-derived values.**
[OWN-5]'s finite origin set, today defined for `slice<'r, T>`, becomes one
mechanism with two users, which are the only *provenance-bearing* kinds:

```text
| kind                    | members                                                    | an origin is          |
|-------------------------|------------------------------------------------------------|-----------------------|
| view                    | Span<'r,T>, MutSpan<'r,T>, AppendView<'r,T>                | a viewed storage place |
| provider-derived value  | box<T>, arena<'p,T>, slot<'p,T>, and the four owners whose | the provider place     |
|                         | backing a provider serves [CNT-1]                          | it was acquired from   |
```

Formation, preservation and access are stated once. A formation makes a
**singleton**: `seq_mut_span(vector: &uniq 'w table[i])` has the singleton origin
`table[i]`, and `pool_take(pool: &uniq 'b blocks, ...)` gives its result the
singleton origin `blocks`. A named const maps to the distinguished
`immutable-const` origin. Binding, moving, passing and returning preserve the set;
a control-flow join takes the union; a parameter of provenance-bearing type starts
with the singleton containing its own formal origin, substituted at a call
boundary. The **resolved** origin set of a value is its origin set minus
`immutable-const`, which creates no conflicting access and has no writable storage
[OWN-5] 602, [OWN-7] 627; every rule that needs a singleton needs a singleton
*resolved* set.

Four uses, and no fifth:

1. **Access strength.** An access through a value of shared loan strength is one
   shared access through every resolved origin; an access through a value of
   exclusive loan strength is one exclusive access through every resolved origin.
   [VIEW-1] fixes each view's strength. A provider-derived value holds no loan at
   all: its origin records where it came from, not a borrow of it.
2. **A loan covers its address computation.** While a loan on a resolved place is
   live, every binding that place's address computation reads is frozen: a write to
   it conflicts under [OWN-5], at the write, naming the loan. Forming a view at
   `table[k]` therefore freezes `k` exactly as it freezes `table`. Without this
   sentence a view can be committed against a window it never held.
3. **A live origin set fixes its storage.** No statement may rebind the storage a
   live origin set describes: a `set` target, a `replace` target, and every future
   exchange form whose target type is provenance-bearing is a hard error, wherever
   the target is reached from. This is the property [SET-2] 508 is protecting,
   stated as a property instead of as a list of two type names, so that adding a
   view type never reopens it.
4. **Disposal targeting** [PROV-6].

Use 2 is checkable only because [OWN-7] 624's subscript overlap stays
conservative, and the register's third list carries that dependency.

*Judgment:* a provenance-bearing value in a prohibited position [CNT-4] is a hard
error there; a rebinding of storage under a live origin set is a hard error citing
PROV-3 at the complete target `place`, with the restructuring `a view's origin set
and a provider-derived value's provenance are fixed at initialization; bind a new
one under a new let`; and a write to a binding a live loan's address computation
reads is the ordinary [OWN-5] conflict. *Publishes:* the origin set, and the
resolved origin set. *Amends:* [OWN-5]'s slice-origin paragraph (580-598), whose
"`slice<'r, T>` value" generalizes to "provenance-bearing value", whose one
access clause becomes the two of use 1, and which gains the address-computation
and resolved-set sentences; [SET-1] 483-485, whose "no writable target path may
traverse a `slice<'r, U>` value" is restated as *a target path may traverse a view
value exactly when that view's loan strength on its resolved origin set is
exclusive*, which is what admits the `MutSpan` element write probe `p7` is refused
today; [SET-2] 508-513, whose region-bearing target rejection is replaced by use 3;
and [EFF-2] 1400-1404, whose slice-parameter projection generalizes to a
provenance-bearing parameter. *Law:* L10, L13.

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
`effect` production (1363-1372); *retires* the effect-row atoms `heap` and `arena`
(META-5: unique fixed lowercase grammar atoms minus 2). *Law:* L2.

**[PROV-5] Reservation is region-local, and its placement is written.** Four
reserving operations exist:

```text
pool_frame<T, const N: u64>['p]()                      -> own Pool<'p, T, N>
pool_extent<T, const N: u64>['p]()                     -> own Pool<'p, T, N>
arena_frame<const BYTES: u64, const ALIGN: u64>['p]()  -> own Arena<'p>
arena_extent<const BYTES: u64, const ALIGN: u64>['p]() -> own Arena<'p>
```

No operand supplies any of those parameters, so each call writes its complete list:
`pool_frame<FixedVector<u8, 256>, 8, 'p>()` [SEQ-0].

Each reserves one store **per activation of the reserving function**. The `frame`
forms lay the extent out in that activation's frame, so it enters the reserving
context's `stack` item of `E`; the `extent` forms produce their own
`region(name, bytes, alignment, contiguous)` item of `E`, whose name is derived
from the reserving occurrence and is not written. Frame placement is the default
for scratch. Extent placement is what a page table, an MMIO window and a DMA
descriptor ring need, and L6 is the reason the choice exists: a deployment reading
one stack total cannot tell a 4096-alignment failure from a size failure, and
cannot grant the page table separately from the stack.

**The written region argument `'p` must be a region introduced by an enclosing
`region_stmt` of the reserving function.** A caller-supplied region parameter is
not admitted. That one sentence ties the extent's storage lifetime to its
confinement region: a value confined to `'p` cannot outlive `'p`, and `'p` cannot
outlive the activation holding the extent. Without it a helper reserves in its own
frame, returns a value confined to its caller's region, and the caller reads a dead
frame. A program that wants a store to outlive a helper reserves it in the caller
and lends the provider down [PROV-7].

*Judgment:* the ordinary region, confinement and [OWN-5] exclusivity judgments,
plus the region-locality check above, whose failure is a hard error citing PROV-5
at the `targ`, with the restructuring `reserve the store in a region this function
opens, and lend the provider to the helper that needs it`. *Publishes:* the
reserved store's measures, its singleton provenance origin [PROV-3], and its
envelope item, one `stack` contribution or one `region` item. *Amends:* [TYPE-5]'s
retained-argument discipline gains no exception, because these are container-domain
rows [SEQ-0]; nothing else. *Law:* L2, L5, L6.

**[PROV-6] Linear disposal.** A type is **linear** exactly when its backing store
reclaims per value: `box<T>`, `slot<'p, T>`, `HeapVector<T>`, `PoolVector<'p,T,N>`,
any nominal with a linear field, any enum with a linear payload, and any container
whose element type is linear. A type whose backing is reclaimed with a region or
with a frame is **not** linear: `arena<'p, T>`, `ArenaVector<'r, T>`,
`FixedVector<T, N>` and `FixedRing<T, N>` over non-linear elements keep their
ordinary compiler-derived release.

A linear value has **no compiler-derived release**. It leaves a scope only by
being moved out or by being consumed by a disposal operation, and every disposal
operation takes the provider its acquisition took: `box_free`, `pool_release`,
`seq_release_heap`, `seq_release_pool`. The disposal operation's judgment is one
sentence: **the resolved provenance origin set [PROV-3] of the disposed value must
be the singleton whose member is the resolved place of the provider argument.**
Not the same region, not the same type: the same place. A lease taken from `alpha`
and released into `beta` is refused even when both are `Pool<'p, u64, 8>` in one
region, and a value whose origin set two branches made a pair is refused because
it is not a singleton.

Three things follow and are stated rather than discovered.

- **Virality is real and is now visible.** A function that takes ownership of a
  linear value on any path and does not return it must hold the provider to dispose
  it, so it names that provider in its signature, transitively up to the holder.
  That is the honest signature fact, and it is the discipline `FilePermit` already
  imposes. Today the opposite is true and invisible: probe `r2_5` compiles
  `fn swallow(item: own box<u64>) -> result: own u64 pure`, whose heap free appears
  in no row and which no [PAR-1] footprint can see.
- **Two frees conflict**, because a disposal is a statement whose `writes` row
  names the provider.
- **A bulk drop of linear elements does not exist.** `seq_clear` and `seq_truncate`
  are inapplicable when the element type is linear; the writer drains with
  `seq_try_take` while holding the provider.

*Judgment:* a linear binding live on an edge leaving its scope is a hard error
citing PROV-6 at that scope exit, naming the binding, its provider type and the
disposal operation, with the restructuring `move the value out, or release it with
<op> while the provider is still live`; and a disposal whose resolved provenance is
not the singleton of the provider argument's resolved place is a hard error citing
PROV-6 at the `call`, rendering both origins. *Publishes:* the release event and
the store's post-state measure. *Amends:* [STOR-3] 683-705, whose `box<T>` drop and
`buffer<T>` drop rows retire and whose release-action table gains the statement
that a linear type has none; [OWN-1] 558, which gains the linear class beside
copy and affine; [EFF-2] 1421's "each of these memory-reclamation actions carries
the empty effect row", which stays **true** and stays unchanged, because after this
rule no memory reclamation is a derived action; [PAR-1] 1969's footprint, through
the ordinary `writes` row rather than a special case. *Law:* L3, L5, L13, L17.

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
draft required a region-free result, which admits `box_new` and `seq_reserve_heap`
and refuses `pool_take`, `arena_new` and `seq_lease`, whose results name the
*store's* region `'p` and never the loan's `'b`: it delivered the capability story
for goal B and left goal A with no `alloc_page` helper, no slab front end, no
layered allocator, and 3.11's pool seam unobtainable. Under the corrected condition
every provider-consuming row is lendable, and the original justification, that
nothing derived from the child outlives the statement, holds verbatim. The second
sentence is the one the second draft assumed and did not write: [OWN-4] makes a
borrow live to the end of its named region's block, so without it a helper may lend
a provider at most once.

*Judgment:* [OWN-6]'s admission, with one more admitted region source under the
stated result-type condition. *Amends:* [OWN-6] 611 and [OWN-4] 570, for this one
form. *Verified today:* probe `r1_relend` is `[OWN-6] InvalidChildReborrow`, and
`r1_relend_affine` shows the existing local-region escape cannot carry an affine
result out. *Note:* this also unblocks `docs/patterns.md` P17's threaded-factory
shape. *Law:* L2.

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

Every member presents its state as one of [RES-6]'s domains; a runtime-owned store
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
obligation, so `W = 1` must always be legal. Every item carries **two** byte
figures: the source-stage ceiling, computed by [RES-6]'s target-independent
arithmetic, and the target-stage exact figure. Acceptance reads the first;
materialization checks the second; a deployment sizes against whichever its
tooling has.
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
     symbolic composition of 3.3.1                                 [RES-6]
```

**A bound is a closed expression in compile-time constants, type-level constants
and runtime-profile symbols. A per-domain figure that names a runtime value is not
a bound**, and premise 3 fails at the loop or call that introduced it, with that
value named. That sentence is what makes stage two a substitution rather than a
discovery, and its absence is why the second draft's loop rule and its arena
arithmetic were both unsound.

The second draft carried a fourth premise, "no execution context reachable from
source can be re-entered from outside the call graph". It is deleted, not weakened:
v0.40 has no source form that declares a reentrant entry point, so the premise
refused nothing. Reentrancy arrives with the execution-context design [1.4].

For a selected target `T` and its runtime, `E-materializes(P, T)` holds when every
symbolic figure of stage one has a concrete value on `T` (frame sizes measured
after code generation [STK-3], strides and alignments [STOR-6], the runtime's own
profile rows [RUN-3]) and every row of the resulting table is representable and is
one the runtime's published profile can carry [RUN-2]. Failure here is a
**target-qualification failure** under [STOR-6] and [QUAL-2]: it stops compilation,
cites no language rule, and is not a source rejection.
*Judgment:* stage one, per domain, over the checked program; deterministic,
terminating, and free of search, budget or timeout. *Publishes:* the property, and
`E`. *Amends:* [STOR-6] 733-761, whose "the language defines no numeric
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

A program whose call graph reaches a `Heap` is not resource-closed, and a `main`
selecting `command.heap` is by itself the rejection. A bounded general store is
still a general store: an envelope item can promise bytes, and cannot promise that
the next contiguous aligned request has a home.
*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage one
is a hard error naming its own cause: the heap-reaching path, rendered from `main`
to the allocation and located at the offending `input_label` or the deepest `call`;
the call-graph cycle [STK-2]; or the unbounded store [RES-6]. *Publishes:* the
property as a compilation fact. *Amends:* [FN-7], which fixes main's marker set,
and [GRAM-2]'s `program_kind` production (META-5: fixed atoms plus 1). *Law:* L1,
L6.

**[RES-5] Store domains and their algebras, in target-independent arithmetic.**
Every covered store presents its state through [MSR-1]'s measures, and exactly
three domains are defined. Nothing else is admitted, and a store outside this list
contributes no envelope item and denies [RES-3].

```text
| domain                     | state         | acquire            | release        | serviceable when |
|----------------------------|---------------|--------------------|----------------|------------------|
| uniform slots              | len, cap = N  | len + 1            | len - 1        | room >= 1        |
|  (Pool; lane, task, queue, |               |                    | [PROV-6]       |                  |
|   completion and handle    |               |                    |                |                  |
|   records of the runtime)  |               |                    |                |                  |
| bump extent (Arena<'p>)    | len, cap      | len + K<T>         | nothing; the   | room >= K<T>     |
|                            |  in bytes     |                    | extent returns |                  |
|                            |               |                    | with 'p        |                  |
| static and frame placement | fixed offsets | none at run time   | none           | decided at       |
|                            |               |                    |                | compile time     |
| general heap (Heap)        | -             | -                  | -              | undecidable      |
|                            |               |                    |                | from E [RES-4]   |
```

`K<T>` is the compile-time constant `align_ceiling(T) - 1 + size_ceiling(T)`,
computed by [OP-9]'s existing ceiling arithmetic. It is **target-independent**, and
it is the only arena advance quantity in this design: the second draft wrote the
exact `round_up(len, align(T)) - len + size(T)` inside the stage-one premise and
the ceiling inside the requirement, with no sentence saying which was normative.
Here stage one is the ceiling and [RES-2]'s second figure carries the exact
composition at target stage.

The runtime's own tables are uniform-slot stores of this list, with their `cap`
published by the profile row [RUN-3] and their `len` composed from the program by
the algebra of 3.3.1.
*Judgment:* the composition of 3.3.1 per domain. *Publishes:* per program point,
per domain, the store's `len` bound. *Amends:* [OP-9] 968, whose `buffer_fits`
stays a representability predicate and which additionally fixes `K<T>`. *Law:* L6,
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
statement instead of a nested `match` on every refusal edge.

Each covered-store acquisition comes in exactly two spellings, on the model of `+`
and `+checked`:

```text
pool_take(pool: p, value: v)          requires igt(room(p), Z)        -> own slot<'p, T>
pool_take_checked(pool: p, value: v)  total                           -> own Result<slot<'p,T>, PoolExhausted<T>>
arena_new(arena: a, value: v)         requires ige(room(a), K<T>)     -> own arena<'p, T>
arena_new_checked(arena: a, value: v) total                           -> own Result<arena<'p,T>, NeedCapacity<T>>
```

The proved form is admitted only when [MSR-4]'s disposition discharges its goal; an
unproved goal is a static rejection with no fallback, exactly as an unproved
subscript is. **The `Heap` has no proved form**: no honest domain predicate exists
for a general store (L6), so every heap acquisition is total and returns `Result`
unconditionally, and its `Err` edge publishes only the returned owner. A store with
measures publishes more: a refused `pool_take_checked` establishes
`ieq(room(pool), Z)` and a refused `arena_new_checked` establishes
`ilt(room(arena), K<T>)`, which is L8's second half and is what makes a checked
acquisition change a loop's summary.

No covered-resource failure is a trap, an abort, a process exit, a retry, or a
promotion to a larger store, in the writer's code or in the runtime.
*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result`, plus [MSR-4]
discharge at the proved spelling. *Publishes:* the returned owner's identity on the
`Err` edge, and the store's own refusal relation where the store has measures.
*Amends:* [TYPE-2] (three added nominals); the batch 0079 exhaustion floor, whose
`wf_resource_abort` site for allocation refusal loses its last reachable caller once
allocation returns a value; and [SCOPE-3] 29, whose
"heap exhaustion ... may stop execution at the host boundary without a Whitefoot
value" ceases to be true. *Law:* L3, L6, L8, L16.

**[RES-7] What bare resource-closedness does not cover.** Disk space, the
successful acquisition of a file, socket or other host object not exclusively
reserved before start, network reachability and throughput, CPU time, deadlines,
scheduler fairness, power, device health, host termination, and OS quota revocation
are outside [RES-1] and outside every judgment in this file. They remain typed
system outcomes where the operation defines one, and environment conditions where
it does not.

One consequence is written rather than promised. A [SYS-2] operation that
materializes a runtime-length host object into runtime-owned storage,
namely `arg_get`, `relative_path`, `host_copy_bytes` and `host_copy_utf8` in their
host-string forms, either names an adapter store whose capacity the profile
publishes [RUN-2], or is **unavailable in a resource-closed program**. This version
takes the second option for `arg_get` and `relative_path` and the first for the
copies, whose destination is a caller-owned view. The second draft promised an
exclusion list by name and wrote none.
*Judgment:* a call to an excluded operation from a marked program's call graph is a
hard error citing RES-7 at the `call`. *Publishes:* the boundary. *Amends:*
[ERR-4] 1478, whose "unavailable external resources remain outside the source
outcome model" gains the two families [RES-6] and [STK-4] move inside. *Law:* L1.

**[RES-8] The per-function summary is part of the callable boundary, in two
pieces.** Each function's boundary [FN-1] gains two derived components, and they
are separate because they belong to different stages:

- a **source-stage per-domain map** over that function's formal provider and
  measure terms, substitutable at a call site, composable across compilation
  units; and
- a **target-stage own-storage figure** covering every store it reserves [PROV-5]
  and its own frame.

The second draft published one component that was half a source-stage map over
formals and half a target-stage byte count over storage the signature does not
mention, and 3.3.2 put those two on opposite sides of the split. Splitting them is
also what keeps [PROV-4]'s framing honest: a self-reserved store contributes to the
second component, which is where the frame item already lives, so 3.3.1's call rule
never meets a callee demand with no actual to substitute.
*Judgment:* none; a boundary statement. *Publishes:* both components. *Amends:*
[FN-1] 999-1006's boundary list. *Law:* L1, L5.

#### 3.3.1 How `E` is composed

Every covered resource is one of three kinds, and conflating them is the single
most common way to get a wrong answer (L9).

```text
| kind                 | question                          | examples                              | bound         |
|----------------------|-----------------------------------|---------------------------------------|---------------|
| reusable capacity    | how many are held at once?        | pool slots, task and completion       | peak len      |
|                      |                                   | records, lanes, queue slots           |               |
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

and the compositions are:

```text
sequence   when A has a fallthrough exit, for each exit label L of B:
             peak(A;B)[L]  = max( peak(A)[fallthrough], delta(A)[fallthrough] + peak(B)[L] )
             delta(A;B)[L] = delta(A)[fallthrough] + delta(B)[L]
           for each non-fallthrough exit label L of A, A;B carries A's own (peak, delta)[L]
           when A has no fallthrough exit, A;B is exactly A's map and B contributes nothing

branch     the union of the arms' maps, keyed by exit label; two arms reaching one
           label contribute the componentwise max of peak and, when their deltas
           differ, the interval [min, max] of delta

call       substitute the callee's source-stage map [RES-8] at the call site, with its
           formal measure and provider terms replaced by the actual ones

loop       for the backedge label, let d be the backedge delta, an integer or an interval:
             max(d) <= 0  -> peak is one iteration's peak; no iteration bound is needed
             max(d) >  0  -> the loop is bounded only when the composed peak is a closed
               expression [RES-3], which it becomes exactly through: a trip count that is
               a compile-time constant; or a store whose cap is a type-level constant and
               whose every acquisition on the loop's paths is the checked spelling, whose
               refusal exit is a real exit with delta 0, or a proved spelling whose goal is
               discharged from a header invariant; or a writer [INV-1] invariant over the
               measure terms. Otherwise there is no finite E and premise 3 fails here.
           each exit label of the loop carries the map of the edge that reaches it

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration peak p
                                        and retained d, and K the profile's window
```

The loop rule is the round-2 repair. The second draft offered "a counted range, a
structural capacity cutoff (`len <= cap`), or a writer-supplied resource
invariant": `len <= cap` is a **standing identity** of [MSR-2] that holds of every
measured place at every point, so that alternative discharged every loop vacuously,
and a counted range bounds nothing when its endpoint is a runtime value. What the
second alternative was reaching for is a condition on the **acquisitions**.

**What needs no writer annotation:** straight-line acquire, move, borrow and
release; lexical scopes and cleanup edges; branch joins; per-variant retention a
`Result` or `Option` already distinguishes; `FixedVector`'s `len <= cap` and its
initialized prefix; moving an owner into or out of a container; a loop whose
backedge restores the state; a constant-trip counted loop whose per-iteration delta
is a fixed affine expression; a non-recursive call with a computed map; and a `par`
loop composed by the formula above.

**What needs one:** a loop that may retain with no structural cutoff; a relation
across two containers (`len(active) + len(waiting) <= cap(pool)`); a resource
returned only at a later milestone; an acquisition whose size is a computed value;
a `par` window the profile does not fix; and any place where the writer wants a
tighter answer than the per-branch maximum. These are ordinary [INV-1] invariants
over the measure terms, which are affine atoms by [MSR-5]. The checker never
searches for an invariant: it does not enumerate paths, guess loop invariants,
choose allocator placements, or divide a store between claimants.

#### 3.3.2 Which stage decides what

```text
 1  tail-SCC rewrite [STK-1]                        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-4]    source stage   compiler
 4  per-function source-stage demand map [RES-8]    source stage   compiler
 5  loop and branch composition (3.3.1)             source stage   compiler
 6  concrete sizes, strides, static image           target stage   compiler
 7  per-context frame envelope [STK-3]              target stage   compiler, post-codegen
 8  runtime profile row for each supported W        target stage   runtime data
 9  par composition against the profile             target stage   compiler
10  assembling E and emitting it as an artifact     target stage   compiler
11  selecting W for this run                        PreStart       launcher
12  committing every region and stack item          PreStart       launcher
13  creating lanes and reaching the ready barrier   PreStart       runtime
14  initializing every adapter record and queue     PreStart       runtime
15  crossing SourceStart and invoking main          PreStart -> Running  runtime
```

Steps 1 to 5 decide whether the program is source-resource-closed, and are the
only steps a source rejection may cite. Steps 6 to 10 decide whether this build
qualifies. Steps 11 to 15 decide whether this run is admitted.

### 3.4 `[STK]`: the stack

**[STK-1] A tail edge is one whose caller frame is dead.** For each strongly
connected component of the call graph in which every intra-component call edge is
a tail edge, the compiler rewrites the component into one dispatcher loop before
frames are measured. **An intra-component edge is a tail edge exactly when, at
that edge, the caller's activation record is dead**: no loan, borrow, view, region
or reborrow the caller introduced is live; no compiler-derived drop remains to run
after the call; no linear binding of the caller is still live [PROV-6]; no `par`
join is outstanding; and no place the caller's frame holds is reachable from any
argument of the call or from any value live across it.

That one premise replaces the first draft's five syntactic conditions. Being
written as the complete `expr` of a `return_stmt` is a consequence of the premise,
not a condition beside it. A confined value cannot defeat it, because [PROV-5]
makes a reserved extent region-local: a live `arena<'p, T>` argument implies `'p`'s
block is open, which implies the reserving activation is live, and that activation
is not the caller being rewritten unless the caller itself introduced `'p`, in
which case the first clause already fires.

A component in which some edge is not a tail edge is **not rewritten**, and is
then refused by [STK-2]. It is never rewritten with a smaller frame.
*Judgment:* per edge, from the ownership and loan state the checker already has;
no proof search. *Publishes:* an acyclic call graph, or a component that is still
cyclic. *Amends:* nothing; this is a lowering and not an admission rule, so
recursion stays permitted. *Verified today:* probes `f2b` and
`f8_tailframe` are mutual tail recursions carrying a live borrow of a caller local
and are accepted, so the premise refuses a shape the syntactic list admitted.
*Law:* L7.

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
receive; without that sentence `PreStart` commits the stack its own frames run on.
And a **worker lane's** chain has no defined root, because a lane executes whatever
the runtime handed it; that question does not arise here, because [RUN-2] fixes
`W = 1` for every resource-closed build, and it is one of the two things the `par`
continuation work must answer before `W > 1` is admitted.

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
entries at all. Two silences it creates are recorded rather than discovered:
compiler-derived releases on an enclosing scope's exit edge never run on a path
that reaches only such a loop, and [LIV-1] has no join to check there. Neither is
exploitable, and both mean those two rules are simply quiet on a divergent path.
*Judgment:* [FN-1]'s existing reachability and fallthrough judgment over the
corrected edge set. *Publishes:* the graph, and hence 3.3.1's exit labels.
*Amends:* [FN-1] 1070. *Verified today:* probes `n2_idle` and `f3_forever` are
`[FN-1] FunctionFallthrough`. *Law:* L1.

**[STK-5] Stack exhaustion moves inside the model, for these programs only.** For
a program that is resource-closed on its target, stack exhaustion is not a
deferred external resource condition: [STK-2] and [STK-3] make the maximum chain a
computed item of `E`, and under an admitted run [RUN-5] it is unreachable. For
every other program, [SCOPE-3]'s deferral stands unchanged, and so does the
guard-page floor that reports it, whose own alternate stack is, for a
resource-closed build, an item of `E`.
*Judgment:* none; a scope statement. *Publishes:* the scope. *Amends:* [SCOPE-3]
29-31. *Law:* L1.

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

**Acquisition and admission control are different obligations, and the second
draft deleted the wrong one.** A qualified runtime must additionally have, for every
one of its stores, a **bounded admission discipline** whose bound is that store's
published capacity: it declines to start work for which no record is available and
resumes when one is, without acquiring anything. Lazily chunking a `par` index range
is exactly such a discipline, and it is what makes [RUN-2]'s "a billion iterations
hold no more task records than a thousand" true. Saturation of a **program-owned**
store is unreachable, because the program's peak was composed against the published
capacity; saturation of the runtime's own scheduling is admission control and must
exist. What stays forbidden is **inline execution**, which nests a task's chain
inside a lane's current activation and which no term of [STK-3] counts, and
**unbounded waiting** on a store no other frame will release. A runtime that cannot
publish a bounded capacity for one of its stores does not support the marker.
*Judgment:* a target-qualification obligation, auditable from the emitted code and
the runtime's own translation units; its failure is a [QUAL-2] qualification
failure, not a source rejection, and no source construct can weaken or waive it.
*Publishes:* the runtime's own items and capacities. *Amends:* [SYS-2] 2264's "no
system operation allocates", which is kept and given its companion: an adapter
record, a host-string lease's backing, and a path buffer are runtime-owned stores
of [RES-1] with published capacities, or the operations that need them are excluded
by [RES-7]. *Law:* L3, L5.

**[RUN-2] `par` enters `E` as a profile, and a resource-closed build runs it
sequentially.** For each supported lane count `W`, the runtime publishes one finite
profile row: `W` lanes, `W - 1` worker stacks, a task-record capacity `K(W, d)`
where `d` is the program's maximum nested `par` depth, fixed queue capacities, a
fixed completion-record capacity, and the handle-table capacity. The number of
iterations of a `par`-permitted loop never appears in `E`.

**Until a compiler-managed continuation representation exists, a `par` statement in
a resource-closed program is executed sequentially and the profile row for such a
build publishes `lanes(1)`.** This is a rule and not a recommendation, because the
alternative is unsound and no compiler check catches it: the current runtime's wait
path executes a stolen task on the waiting lane's own stack, so `stack(lane_i)` as
[STK-3] computes it is wrong by a factor bounded only by the outstanding-task
count, and [RUN-5]'s theorem is then false on an admitted environment. Round 2
asked whether the continuation redesign is needed for soundness or only for
liveness; it is needed for soundness, and this rule is what makes that answer
load-bearing rather than an open question. Two consequences follow for free:
[PAR-3]'s replicated places, which are execution memory no envelope item counts,
cannot occur in a resource-closed build; and [STK-3]'s undefined worker-lane chain
does not have to be defined in this version.
*Judgment:* a fixed-arithmetic composition (3.3.1's `par` rule) against each
profile row, plus the sequential-execution requirement on a marked program; the
compiler emits no per-`W` clone. *Publishes:* the `lanes` and `slots` items of each
row. *Amends:* the sentence common to [PAR-1] 1989, [PAR-2] 2024 and [PAR-3] 2049,
"exhaustion of the execution resources an implementation spends on overlapping is a
resource condition under [SCOPE-3] and is not an observable of this rule": for a
program resource-closed on this target that exhaustion is unreachable. *Law:* L5,
L9.

**[RUN-3] The parallel footprint of an allocation is its provider place.** In
[PAR-1]'s written-footprint clause, "the caller region each `allocates(arena 'r)`
entry names after region substitution" is replaced by "the places each `allocates`
path reaches under the [EFF-2] call-boundary projection", the same projection the
rule already applies to `reads` and `writes`. Two statements that allocate from one
provider therefore conflict, and two that allocate from distinct providers do not.
With [PROV-6] the same is now true of two statements that only *free*, because a
free is a call with a `writes` row on the provider.

[PAR-2]'s permission for a fill through a `MutSpan` needs one further amendment,
and it is a genuine refinement rather than the "one word" the second draft claimed.
[PAR-2] 2006 requires every place a footprint of `B` holds an exclusive loan on to
be rooted in a binding `B` introduces, and 2004 denies on any loan overlapping the
resolved root. A `MutSpan` formed **once, outside** the loop holds one loan for all
iterations, which is not the hazard either condition names. The amendment states
the condition over **iteration-formed** loans: every exclusive loan *formed by a
statement of B* is rooted in a binding `B` introduces, and a loan formed before `L`
on a root every footprint of `B` reaches only through the refined single-element
ranges of 1999-2002 does not deny, which is the argument 2001-2002 already makes.
Its element-write form additionally reads "a direct subscript of an array, a prefix
owner, or a `MutSpan`", never a `FixedRing` subscript, whose logical-to-physical
mapping is not affine in the binder.
*Judgment:* the existing [PAR-1] and [PAR-2] permission judgments, with one fewer
special case and one added loan clause. *Publishes:* permission. *Amends:*
[PAR-1] 1969, [PAR-2] 1994-2006, and [PAR-2]/[PAR-3] through their "forms every
footprint exactly as [PAR-1] forms one" clauses. *Law:* L2, L5.

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
| HeapVector<T>        | prefix | one heap allocation, | Heap      | runtime    | yes    | seq_reserve_heap |
|                      |        | none while empty     |           |            |        |             |
| ArenaVector<'r, T>   | prefix | one arena block in 'r| the arena | runtime    | no     | seq_reserve_arena |
| PoolVector<'p, T, N> | prefix | one pool lease of a  | the pool  | N, from    | yes    | never       |
|                      |        | FixedVector<T,N> slot|           | the slot   |        |             |
| FixedRing<T, N>      | ring   | inline, N slots      | none      | N          | no     | never       |
```

The `linear` column is [PROV-6]'s classification and follows from the backing:
`HeapVector` and `PoolVector` reclaim per value and are linear; `ArenaVector`'s
block returns with its region and `FixedVector` and `FixedRing` are frame-resident,
so none of the three is. An owner over a linear element type is linear whatever its
own backing.

A prefix owner's initialized storage is exactly `[0, len)`. A ring carries one
further piece of typestate, a head offset, and its initialized storage is
`[head, head + len)` modulo `N`; that is still one scalar relation and no per-slot
state, so L12 holds unchanged. A ring's element access is by logical index
`0 <= i < len(ring)`, written as an ordinary subscript, and a ring yields **no
view**, because its initialized region is not contiguous. The second draft gave a
ring only a copy-returning reader, which made a ring of records write-only; the
subscript removes that restriction rather than adding a second operation.

A container type is a compiler-owned nominal: no writer-visible field, constructed
only by the [SEQ] operations, no source construction form. An ordinary struct whose
invariants are reproved at every use is refused, because `len <= cap` would then be
a fact with support the writer can kill.
*Judgment:* the ordinary nominal-resolution and construction judgments; a
`construct` naming an owner nominal is a hard error citing CNT-1. *Publishes:* the
five types and their measure rows. *Amends:* [TYPE-2] 352 (five added composite
types) and [GRAM-3] 204-207, whose fixed `buffer` production retires with the
writer-facing type (META-5: unique fixed lowercase grammar atoms minus 1). *Law:*
L4, L12.

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
second family: `FixedVector<Option<T>, N>`, filled once, with
`replace v[i] = Some(...)` to occupy a slot and `replace v[i] = None()` to vacate
it. The prefix is full at `N` and never moves, so no index is renumbered, and the
occupancy is the writer's own `Option` discriminant, which is data and not
typestate, so L12 is untouched. Probe `r2_7` compiles that shape today, including a
`len` read that survives an element-position replace. Its cost is one construction
loop and one `match` per read, recorded in Q6; it replaces `buffer_vacant`, which
is a heap operation and retires with `buffer<T>`.
*Judgment:* [OP-4] at every subscript, against `len`. *Publishes:* the typestate.
*Amends:* [OP-4] 909, whose indexable bases extend to the prefix owners, the ring,
`Span` and `MutSpan`, and whose obligation is against `len`. *Law:* L11, L12, L15,
L16.

**[CNT-3] Affine and linear elements, and `array<T, N>` unchanged.** `T` may be
affine in every owner, and may be linear. The initialized region is what makes this
sound: an element enters and leaves only through an operation that moves the
boundary or exchanges two initialized positions, so no slot is read before it is
written or after it is taken. An owner over a linear `T` is itself linear
[PROV-6], so it cannot be dropped and its elements cannot be bulk-dropped; the
writer drains it while holding the provider.

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

**[CNT-4] Confinement.** A type is **confined** when its complete type after
substitution names a region that is not an origin set: `arena<'r, T>`,
`slot<'p, T>`, `Arena<'p>`, `Pool<'p, T, N>`, `ArenaVector<'r, T>`,
`PoolVector<'p, T, N>`, and every generic instance one of them is an argument of.
A confined value may occupy any position whose owning value's own complete type
names the same region, so that the position is itself confined and [STOR-4] governs
it. A **provenance-bearing** type [PROV-3] may occupy no position from which a
value could outlive or hide its origin set: no field, no enum payload, no element
of any owner, no `box`, `arena` or `slot` content, no generic type argument, and no
result outside [VIEW-6]'s ceiling.

The confinement of a value is the **set** of regions its complete type names, and
it may be moved, returned, or bound to a destination that **every** member
outlives-or-equals [OWN-3]. Quantifying is the whole repair: the second draft said
"its own region", and a value of type
`Result<slot<'p, Page>, NeedCapacity<arena<'q, u64>>>` has two, which [OWN-3] makes
incomparable, so an implementation choosing either one lets the other escape.
With the quantifier, [OWN-3]'s fail-closed rule gives the right answer for free.

**A source nominal may declare region parameters**, written exactly as a function
declares them, and is confined by them:

```wf-design
struct Chunk['p] {
  page: PoolVector<'p, u8, 4096>;
  used: u64;
}
```

Two instances of one such nominal have the same type only when their region
arguments are identical: region parameters on a nominal are **invariant**, which is
[OWN-3]'s "distinct caller-supplied regions are incomparable, and any rule
requiring an order between them fails closed" applied where it already applies, and
which is why this feature needs no variance design.

This is decided on the merits rather than deferred. [CNT-4] already admits a
confined value into a container element, because the container's own type names the
region; forbidding the same value in a record field therefore buys **no soundness at
all**, and it forces every kernel structure into parallel columns whose index
correspondence nothing checks, which is the defect L11 exists to remove. The
precondition the deferral was really protecting is [PROV-5]'s region-local
reservation, which this draft states.
*Judgment:* a provenance-bearing type in a prohibited position, or a confined type
in a position whose owner does not name its region, is a hard error citing CNT-4 at
the complete contained `type`, with the restructuring `keep the view as a direct
local, parameter, or result` for the first and `give this nominal a region
parameter and confine the field to it` for the second; and a confined value bound
to a destination some member of its region set does not outlive is a hard error
citing CNT-4 at the binding, rendering every member. *Publishes:* the confinement
set. *Amends:* [STOR-4] 716, whose "may not be returned" becomes the ordinary
outlives relation over the set; [STOR-5] 718, whose enumerated position list is
replaced by the intensional split above; [FN-2] 1087, whose blanket rejection of a
region-bearing generic argument narrows to provenance-bearing arguments and whose
"instantiation arguments are always explicit" now covers region arguments on
nominals; and [GRAM-2]'s `struct_decl` and `enum_decl`, which gain `region_params?`
after `generics?`. *Verified today:* probe `f7_regionresult` is [FN-2]
`RegionBearingGenericArgument` and probe `r2_6` is a [GRAM-2] parse error at
`struct Wrap['p]`, so both halves are new. *Law:* L10, L13.

**[CNT-5] Release and disposal of owners.** The disposition of every owner:

```text
| owner        | at a scope exit                                                          |
|--------------|--------------------------------------------------------------------------|
| FixedVector  | compiler-derived: drop each initialized element in ascending index order  |
| FixedRing    | compiler-derived: drop each initialized element in ascending logical index|
| ArenaVector  | compiler-derived element drops; the block goes with 'r [STOR-4]          |
| HeapVector   | none; linear [PROV-6]. seq_release_heap(vector, heap) disposes it        |
| PoolVector   | none; linear [PROV-6]. seq_release_pool(pool, vector) disposes it        |
```

Both disposal rows carry `requires ieq(len(vector), Z)`. That is uniform for every
element type, it makes the drop-order question vanish, and it is the only shape
under which a container of **linear** elements can be disposed at all, since those
elements have no derived drop; a writer drains with `seq_try_take` or, for a
non-linear element type, with one `seq_clear`.
*Judgment:* [LIV-1] at every scope exit, plus [PROV-6]'s provenance match at each
disposal. *Publishes:* the release event and the store's post-state measure.
*Amends:* [STOR-3] 683's `buffer<T>` drop sentences, which are superseded.
*Law:* L13, L17.

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
type is a hard error citing CNT-7 at the complete `param`, with the restructuring
`pass a MutSpan or AppendView for element and append work, or take the owner by
value and return it`. A shared `&'r` container parameter remains legal: it can
observe measures and read elements and can change nothing.

This is the rule that retires D1's shape. `&uniq` survives everywhere its
referent's measures are type facts rather than state: a `&uniq` to a struct holding
`array<T, N>` fields, or to a `MutSpan`, is legal because no operation on either can
change a length. It does **not** survive on a `&uniq slot<'p, Container>`: the
second draft blessed that case with the sentence "no operation on any of them can
change a length", which is false, because [CNT-7] bites on the direct type,
`deref(s)` selects the container, and a `replace` through the holder is D1 one
indirection over. That shape is refused, but by [CALL-5]'s conservative default and
not by this rule, and the difference matters because someone will one day extend
[CALL-3]'s class using the wrong sentence as the criterion.
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
does today, and each is provenance-bearing [PROV-3]. `Span<'r, T>` **is** today's
`slice<'r, T>` renamed; the rename is the whole of the change to it. Its measures
are [MSR-1]'s rows.
*Judgment:* none by itself. *Publishes:* the three types and their loan strengths.
*Amends:* [TYPE-2] 352 (two added view types, `slice` renamed `Span`), [GRAM-3]
204-207, whose fixed `slice` production retires in favour of an ordinary TYPEID
with `targs` (META-5: unique fixed lowercase grammar atoms minus 1), [OWN-1] 558
(all three are affine), and [CONST-2] 547-551, [OP-7] 935 and [OP-1]'s `slice_of`
row, which name the retired spellings. *Law:* L10.

**[VIEW-2] Formation, and the loan the view value holds.** A view is formed from a
borrow of the owner:

```text
seq_span['r](vector: &'r v)          -> own Span<'r, T>
seq_mut_span['r](vector: &uniq 'r v) -> own MutSpan<'r, T>
seq_append_view['r](vector: &uniq 'r v) -> own AppendView<'r, T>
```

and **the view value, not the argument borrow, holds the loan**. For its whole
life, a view value holds a loan of its own strength on every place in its resolved
origin set [PROV-3]. The loan begins at formation and ends when the view value is
consumed or released. The argument borrow is a call-scoped temporary, which probes
`f2b` and `r1_twouniq` confirm by accepting two of them on one place in one region;
it could not be the freeze.

Exclusivity then refuses a second `AppendView` on one owner at its formation, and
the sentence that does that work is [OWN-5]'s "a write, move, or unique borrow of an
ordinary place conflicts when that place overlaps any such origin", applied to the
second formation's argument borrow; the second draft credited a different sentence,
and this one is preserved by [PROV-3] rather than by luck. It deliberately admits a
*shared* borrow and a direct `let n = len(buf);` while an exclusive view is live,
which is sound: a `Span` sees the committed prefix and an `AppendView` publishes
nothing until `absorb`. Ending the loan at the consume rather than at the end of
`'r` is what makes append-commit-read need no nested region per phase.

Formation publishes:

```text
seq_span         len(s) = len(v),  cap(s) = len(v)
seq_mut_span     len(m) = len(v),  cap(m) = len(v)
seq_append_view  len(a) = Z,       cap(a) = room(v)
```

Each is a two-term relation over the pre-transfer datum of the borrowed owner;
`room(a) = room(v)` follows from [MSR-2]'s identity and is not separately
published.
*Judgment:* [OWN-5] at the formation borrow, and the ordinary [SEQ-0] relation
establishment. *Publishes:* the loan, and the three formation relations.
*Amends:* nothing beyond [PROV-3]'s amendment of [OWN-5]. *Law:* L10, L14, L15.

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
3. every fact whose support overlaps `P`'s descriptor dies [MSR-2]; and
4. `len(P)' = <pre-transfer datum of len(P)> + w` and
   `room(P)' = <pre-transfer datum of room(P)> - w` are established.

Step 4 is the round-2 repair. The second draft transferred an *image* across the
kill step 3 performs, and no rule said whether an image survives a kill; either
answer was wrong, one making `absorb` publish nothing and the other leaving a stale
image behind every projected callee write in the language. A datum has empty
support, so step 4 never depended on surviving step 3. Requiring a singleton
**resolved** set is what makes the rule satisfiable at all: [FN-1] 1036 includes
`immutable-const` in every call-site origin set, so a view that crossed a call never
has a singleton origin set.
*Judgment:* the four steps above; a non-singleton resolved origin, or one that is a
formal-view origin, is a hard error citing VIEW-3 at the operand `atom`, with the
restructuring `return the view to the function that formed it and absorb it there`.
*Publishes:* the commit value and the owner's new measures. *Amends:* [ENT-3.S5]'s
commit-value clause, which gains `absorb`'s. *Law:* L10, L14, L16.

**[VIEW-4] A view descriptor's length cannot be changed through a borrow.** No
operation takes a `MutSpan` or a `&uniq` to one and produces a different length,
and none changes its owner's length. The ground is stated once, as a property
rather than per type: a view is affine, so [SET-1] refuses a `set` of it, and its
origin set is live, so [PROV-3] use 3 refuses a `replace` of it wherever the target
is reached from. Therefore `MutSpan<'r,T>`, `&uniq 'b MutSpan<'r,T>` and
`&uniq 'b AppendView<'r,T>` are all length-fixed for [CALL-3].

**This dependency is load-bearing and the register carries it as a row of its
own.** The second draft recorded it in prose and listed [SET-2] as deliberately
unchanged, while replacing the relation [SET-2] defers to; the result was D1
verbatim on `&uniq MutSpan`, admitted by every rule the register left standing. A
rule whose premise moved is a changed rule.
*Judgment:* none by itself; it is the premise of [CALL-3]. *Publishes:* the
length-fixed class. *Amends:* nothing beyond [PROV-3]'s amendments of [SET-1] and
[SET-2]. *Law:* L11.

**[VIEW-5] An abandoned `AppendView` drops what it appended.** Its
compiler-derived release drops the elements of `[base, base + len(a))` in ascending
order, then nothing. The owner's `len` is unchanged, so the abandoned elements are
neither leaked nor double-dropped, and no fact about `len(P)` was ever published.
Not absorbing is a well-defined, safe program that discards work, which is what
makes `absorb` an ordinary operation rather than a must-use obligation. When `T` is
linear the view is linear too and abandoning it is [LIV-1]'s error, because there
is no derived drop to run.
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
passing it down, and no helper library over views can exist in this version.
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
than a register row because it is goal A's container half. It is also the rule that
carries a cost this design does not hide: a destination must be **addressable**
before the host writes into it, so it is built with `seq_filled` and the count the
host produced is an ordinary `u64` beside the container rather than the container's
own `len`. The write-back machinery of [VIEW-3] does not reach the one boundary
where lengths genuinely come from outside. Section 5 records that as an open
question with a recommendation.
*Judgment:* [SYS-8]'s two range obligations, restated over `len` of the borrowed
view. *Publishes:* the endpoint facts [ENT-3.S10] already enumerates, now over a
view. *Amends:* [SYS-8] 2482-2485, [SYS-2] 2158-2301's declaration records and its
normative counts, and the prose of [SYS-9], [SYS-11], [SYS-12] and [SYS-14], which
name `buffer<u8>`. *Law:* L11, L14.

### 3.8 `[LIV]`: liveness, reinitialization, and transformation

**[LIV-1] Liveness is join-checked, and that is what makes release
unconditional.** A binding's live-or-dead status is a property of a program point,
not of a path: at every join of the conservative structural graph [FN-1], and at
every loop head, every predecessor must agree on the status of every binding in
scope. A disagreement is a hard error citing LIV-1 at the join, naming the two
predecessors and the binding. On every edge leaving a scope, every **linear**
binding of that scope must be dead [PROV-6], because no derived release exists to
carry it.

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
current function, a `let` binding or a parameter, **whose current value has already
been consumed**, and `e` produces exactly `p`'s type.

```wf-design
set pending = move rest;
```

Its judgment: evaluate `e` under ordinary rules, including any consume of `p` inside
it; every fact whose support contains `p`'s root dies at that consume; then the
binding is reinitialized with `e`'s value, live and usable, with no observable
program point between. It derives no drop and no release, because the target holds
no value.

**A reinitializing `set` is a declaration event for [ENT-2] term identity.** The
reinitialized binding is a *distinct term* from the consumed one, exactly as
[ENT-2] already rules for "a fresh binding legally reusing an expired spelling".
That one sentence is the second half of round 2's rank-one soundness repair: without
it, a fact stated over the old value reaches the new one, and the language admits an
eight-element measure on an empty container. Its measure images transfer by
[MSR-3]: `e`'s declared relations over its pre-transfer datums install `p`'s new
images, which is the mechanism that carries a length or a spare across a loop
backedge, and images are keyed by binding rather than by term, so loop-carried
reasoning is unaffected.

The premise is one fact the checker already tracks: **the target is dead**.
[STOR-1]'s existing rejection of a `set` on a *live* affine place keeps its exact
wording and its `replace` mechanical fix; only the dead case is added.
*Judgment:* the deadness premise plus the ordinary [TYPE-5] exact-type check.
*Publishes:* the new binding's term identity and its measure images. *Amends:*
[ENT-2] 2678's term-identity paragraph (one added declaration event), [OWN-1] 558,
[STOR-1] 670 and [SET-1] 493, whose affine-target rejections narrow to a live
target. *Verified today:* probe `p10` is [STOR-1] `AffineSetTarget` and probe `p11`
is [OWN-1] `UseAfterMove`, the two halves of the premise. *Law:* L10, L16, L17.

**[LIV-3] The transformation statement.** One statement form is added, and it is
the only spelling of the receiver-threading shape:

```wf-design
update view by seq_push(value: byte);
update work by seq_place(value: left);
update buckets[slot] by seq_clear();
```

`update p by op(args);` is admitted when `op` is a container-domain operation
[SEQ-0] with exactly one result whose type is the type of `p`, and `p` is a
writable owner place. It means exactly
`set p = op(<op's first declared parameter>: move p, args);` with the receiver
supplied from `p`, and it carries that statement's complete judgment, [LIV-2]
included.

It is not sugar, on two counts, both round 2's findings. It reaches places `set`
cannot: `set` requires a bare binding, so a container nested at `table[i]` has no
operation but view formation, and draining one costs a `replace` with a freshly
constructed empty container. And it removes the `move` a writer must spell on a
value the statement hands straight back, which is the friction in nine of the
fourteen programs round 2 wrote and the only one that scales with the number of
operations rather than the number of loops.

Because it is the only spelling, `set p = op(receiver: move p, ...)` for a
single-result container-domain `op` is a [FORM-1] rejection whose fix is
`update p by op(...)`. It is the only spelling **for that domain**: a user helper
that threads its owner keeps `set buf = collect(out: move buf, source: move line);`,
because [SEQ-0] fixes a receiver-first convention that [FN-1] does not. A multi-result row keeps
`let (rest, x) = op(vector: move p, ...);` then `set p = move rest;`. This design
adds no receiver-position call form: `view.seq_push(value: byte)` would be a second
call syntax whose resolution [GRAM-5] does not have, while `update`'s resolution is
a table lookup.
*Judgment:* row selection, the single-result and type checks, then [LIV-2]'s.
*Publishes:* nothing beyond [LIV-2]. *Amends:* [GRAM-4]'s `stmt` production (one
added statement form) and [FORM-2], which renders it as one line
`update <place> by <call>;` (META-5: fixed atoms plus 2, `update` and `by`).
*Verified today:* probe `r2_8` is a [FORM-1] parse rejection, so this is new
syntax. *Law:* L10.

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
consumed parameter's measure, which denotes that call's **pre-transfer datum**
[MSR-3]: `len(result) = len(view) + 1` means what it reads as, and it is
establishable at the caller precisely because a datum has empty support and the
consume the same statement performs cannot kill it. That is the repair of the
finding that this transport, on which the entire surface rests, published nothing
at all: [FN-9]'s `M(c,q)` requires every referenced formal to substitute to a
**live** term, and the actual of a value-in/value-out row is dead by the time the
relation is established.
*Judgment:* the ordinary [ENT-3.S13] establishment, subject to `M(c,q)` as [MSR-3]
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
ordinary descriptor-overlapping event [MSR-2].
*Judgment:* the kill classification per parameter type. *Publishes:* the surviving
measures. *Amends:* nothing beyond [MSR-2]'s. *Law:* L11, L14.

**[CALL-4] Contract vocabulary, and the ordered result list.** [FN-9]'s clause
operands are terms [MSR-5], so `len(P)`, `cap(P)` and `room(P)` over an admitted
formal place are operands with no per-family admission; a parameter's measure
denotes its **entry datum** [MSR-3], so a consuming use inside the body does not
take it away. `len(result)`, `cap(result)` and `room(result)` are operands when the
written result type is measured, which today's result-datum restriction to fragment
integers forbids. So the canonical append contract is writable:

```wf-design
fn append_span['o, 'i](out: own AppendView<'o, u8>, source: own Span<'i, u8>) -> written: own AppendView<'o, u8> reads(out, source), writes(out) contract {
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
fn render['p](block: own PoolVector<'p, u8, 256>, task: own Task) -> (rest: own PoolVector<'p, u8, 256>, written: own u64) ... contract {
  ensures ile(written, len(rest));
}

let (rest, task) = seq_take(vector: move pending);
```

The result is not a value: there is no tuple type, no tuple place, and no way to
store or pass one. It is a return-and-bind form only, which keeps [CNT-4] and
[TYPE-2] untouched. Multi-return is load-bearing, not a convenience: `seq_take`
must return an owner and an element, and no single value can carry both, since an
enum payload holding a confined or provenance-bearing value is refused by [CNT-4].
Three productions change together: `result_binding` may be one binding or a
parenthesized list, `let_stmt` may bind one IDENT or a parenthesized list, and
`return_stmt` may carry one `expr` or a comma-separated list whose length equals the
function's. Every element is judged independently by the ordinary [FN-1] return
rule. Its canonical rendering is stated rather than left to [FORM-2]'s attachment
sets: a result list renders as `-> (` then its comma-separated bindings then `)`,
a destructuring `let` as `let (` then its binders then `) = `, and a multi-value
`return` renders its expressions comma-separated on one line.
*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set and
the widened result shape. *Publishes:* the clause relations. *Amends:* [FN-9]
1295-1305 (measured results, multi-datum clauses, the entry-datum operand),
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
indirection over, through `&uniq slot<'p, Container>`, which [CNT-7] does not
reach.
*Judgment:* the conservative default for every unselected parameter type.
*Publishes:* the absence of a call-site-derived fact. *Amends:* [ENT-5] 2870's
clause (b), whose projected-callee-write kill is now classified by [CALL-1..3] and
by nothing else. *Law:* L11.

### 3.10 `[SEQ]`: the operation inventory

**[SEQ-0] The container declaration domain.** The container and provider
operations are one compiler-owned **generic** declaration domain, built as [SYS-1]
and [SYS-2] build the system domain and admitted to every compilation unit on the
same terms. Each operation is one complete signature record: named parameters in
declared order [GRAM-11], its type, const and region parameters written as
[GRAM-2] orders them, one declared effect row, one declared result mode and type or
one ordered result list, one declared requirement list, and one declared relation
list. **The first declared parameter is the value the operation transforms and
returns; an operation that transforms nothing names its provider first**, which is
`box_free`'s and `pool_release`'s existing shape and is what [LIV-3] reads.

Five sentences fix everything the second draft left to a table cell.

**Written arguments.** A row writes its complete type, const and region argument
list in one `targs` list, ordered as [GRAM-2] orders a declaration (type and const
parameters, then region parameters), exactly when some parameter of that list is
supplied by no operand; otherwise it writes none. A partial list is not a spelling
option, because a partial list is the transposition hazard redundant-explicit facts
exist to remove. So `seq_fixed<Task, 32>()`, `seq_filled<u8, 4096>(value: 65_u8)`
and `pool_frame<FixedVector<u8, 256>, 8, 'p>()` write their lists, while
`seq_lease(pool: &uniq 'b blocks)`, `seq_span(vector: &'s input)` and
`seq_push(view: move acc, value: byte)` write none: their operands' types supply
every type and const parameter and the borrow's own written region supplies the
loan region. This is [TYPE-5]'s own criterion, "no operand can supply them",
applied to a domain rather than enumerated per row.

**Where the relations come from.** An operation's declared relations are
established on its results exactly as [ENT-3.S12] establishes a verified user
summary, with parameter operands denoting that call's pre-transfer datums [MSR-3].
A row with **per-variant** relations, every `try` row, names one *designated
outcome result*, and its per-variant relations are established at entry to the arm
of the first `match` whose scrutinee is a bare live binding of that result, under
the same no-kill, no-`set` path discipline [ENT-3.S7] already uses for a `+checked`
arm fact. Probe `r2_9` shows that discipline tolerates an intervening statement
today; what it does not tolerate is a statement that consumes the result the
relations name, which is why a program that wants both arms' relations writes the
`match` before the rebind. The second draft asserted the route was free because
"[ERR-3]/[OWN-13] already dispatch on the variant"; they dispatch, and [ENT-3.S12]
has four destinations and none of them is a variant of one member of an ordered
result list.

**One row per operation.** Each operation carries its own requirement and relation
cells, written over its own formals. The second draft keyed those cells by row id
and let three operations share two, which left `seq_take_front` with a requirement
naming a formal it does not have, so a take from an empty ring was admitted and
published nothing.

**The readers are not in this domain.** `len`, `cap` and `room` are three [OP-1]
table operations taking a bare non-consuming place operand and returning `own u64`,
and they are **`pure`**: the operation reads no state the caller does not already
hold, and [EFF-2] attributes the operand's own read exactly as it does for any
other non-consuming table operand. Probe `r2_10` shows the consequence today: a
`define capacity = len(deref(destination));` over a shared-borrow parameter compiles
in a `pure` function, and declaring `reads(destination)` for it is an [EFF-2]
rejection. A `reads` row on the readers would remove `len` from every
`contract_define` and `requires`, which is `wfgrep`'s accepted shape and P16's.

*Judgment:* row resolution by name, receiver type and written arguments; the
per-row requirement discharge under [MSR-4]; and the [GRAM-11] named-argument
check. A diagnostic for an operation cites **[SEQ-0]** and names the operation in
its payload, exactly as an [OP-1] diagnostic cites [OP-1]; [DIAG-1] 1535 admits one
numbered language rule and the inventory rows below are table data, not rules.
*Publishes:* every declared relation of every row. *Amends:* [SYS-1] 2130 (a fourth
admitted declaration source), [SYS-3] 2303 (admitted to every unit), [TYPE-6]
391-403 (the operation spellings enter the lexical IDENT domain and the nominals
the TYPEID domain), [DIAG-1] 1687-1712 (collision rank 5, and a
`container_declaration_ordinal` beside the system one), [ENT-3] 2724 (one added
enumerated source S13, plus the arm route above), [OP-1] 793-828 (`len` gains `cap`
and `room`, their domain extends to owners, views and providers, and `slice_of`,
`buffer_new` and `buffer_vacant` retire; `ReservedLowerNames` gains `cap` and
`room`), [TYPE-5] 367-374 (the written-argument criterion above covers a fourth
callee class), and [FN-2] 1087 (its explicit-argument rule covers this domain).
*Law:* L11, L16.

#### The inventory

`V` ranges over the four prefix owners. Every row's first parameter is the value it
transforms, except a disposal, which names its provider first. A row's type, const
and region parameters are exactly those its signature names, declared in [GRAM-2]'s
order and elided below where the signature shows them; what a **call** writes is
[SEQ-0]'s written-argument rule. Each provider is written out per row rather than
abbreviated, because a `HeapVector` and an `ArenaVector` have different providers,
different effect rows and different failure types, and one row varying all three by
receiver is the effect polymorphism this design rejects.

**Construction.**

```text
seq_fixed<T, N>()                  -> own FixedVector<T, N>     pure
    declares len(result) = Z, cap(result) = N
seq_ring<T, N>()                   -> own FixedRing<T, N>       pure
    declares len(result) = Z, cap(result) = N
seq_heap<T>()                      -> own HeapVector<T>         pure
    declares len(result) = Z, cap(result) = Z
seq_arena<T>['r]()                 -> own ArenaVector<'r, T>    pure
    declares len(result) = Z, cap(result) = Z
seq_filled<T, N>(value: own T)     -> own FixedVector<T, N>     pure          T copy
    declares len(result) = N, cap(result) = N
seq_heap_filled<T>['b](heap: &uniq 'b Heap, count: own u64, value: own T)
    -> own Result<HeapVector<T>, OutOfMemory<unit>>             allocates(heap), writes(heap)   T copy
    requires buffer_fits<T>(count)
    declares Ok(value: r): len(r) = count, cap(r) = count
seq_lease(pool: &uniq 'b Pool<'p, FixedVector<T, N>, K>)
    -> own Result<PoolVector<'p, T, N>, PoolExhausted<unit>>    allocates(pool), writes(pool)
    declares Ok(value: r): len(r) = Z, cap(r) = N, len(pool)' = len(pool) + 1
             Err: room(pool) = Z
seq_lease_proved(pool: &uniq 'b Pool<'p, FixedVector<T, N>, K>)
    -> own PoolVector<'p, T, N>                                 allocates(pool), writes(pool)
    requires igt(room(pool), Z)
    declares len(result) = Z, cap(result) = N, len(pool)' = len(pool) + 1
```

**Readers and element access.**

```text
len(p) / cap(p) / room(p)          -> own u64                   pure          [OP-1] rows
p[i]                               element place                              prefix owner, ring, Span, MutSpan
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
seq_exchange(vector, first: own u64, second: own u64) -> own V<T>             pure
    requires ilt(first, len(vector)), ilt(second, len(vector))
    declares len(result) = len(vector), cap(result) = cap(vector)
seq_clear(vector)                          -> own V<T>                        reads(vector), writes(vector)   T non-linear
    declares len(result) = Z, cap(result) = cap(vector)
seq_reserve_heap['b](vector: own HeapVector<T>, heap: &uniq 'b Heap, additional: own u64)
    -> own Result<HeapVector<T>, OutOfMemory<HeapVector<T>>>    reads(vector, heap), writes(vector, heap), allocates(heap)
    declares Ok(value: r): len(r) = len(vector), cap(r) = cap(vector) + additional
             Err: the vector returns unchanged in error.rejected
seq_reserve_arena['p, 'b](vector: own ArenaVector<'p, T>, arena: &uniq 'b Arena<'p>, additional: own u64)
    -> own Result<ArenaVector<'p, T>, NeedCapacity<ArenaVector<'p, T>>>
                                                               reads(vector, arena), writes(vector, arena), allocates(arena)
    declares Ok(value: r): len(r) = len(vector), cap(r) = cap(vector) + additional
             Err: the vector returns unchanged in error.rejected
seq_shrink['b](vector: own HeapVector<T>, heap: &uniq 'b Heap)
    -> own Result<HeapVector<T>, OutOfMemory<HeapVector<T>>>    reads(vector, heap), writes(vector, heap), allocates(heap)
    declares Ok(value: r): len(r) = len(vector), cap(r) = len(vector)
             Err: the vector returns unchanged in error.rejected
seq_release_heap['b](heap: &uniq 'b Heap, vector: own HeapVector<T>) -> own unit    writes(heap)
    requires ieq(len(vector), Z)
    declares nothing
seq_release_pool(pool: &uniq 'b Pool<'p, FixedVector<T,N>, K>, vector: own PoolVector<'p, T, N>) -> own unit    writes(pool)
    requires ieq(len(vector), Z)
    declares len(pool)' = len(pool) - 1
```

**Ring operations.** A ring is a distinct receiver with distinct ends, so it has
its own names rather than sharing a row.

```text
ring_place(ring: own FixedRing<T,N>, value: own T) -> own FixedRing<T,N>      reads(ring), writes(ring)
    requires igt(room(ring), Z)
    declares len(result) = len(ring) + 1, cap(result) = N          appended at the tail
ring_try_place(ring, value: own T) -> (rest: own FixedRing<T,N>, unplaced: own Option<T>)   reads(ring), writes(ring)
    designated outcome: unplaced
    declares None: len(rest) = len(ring) + 1;  Some: len(rest) = len(ring), room(rest) = Z
ring_take(ring: own FixedRing<T,N>) -> (rest: own FixedRing<T,N>, value: own T)             reads(ring), writes(ring)
    requires igt(len(ring), Z)
    declares len(rest) = len(ring) - 1, cap(rest) = N              removed from the head
ring_try_take(ring) -> (rest: own FixedRing<T,N>, value: own Option<T>)                     reads(ring), writes(ring)
    designated outcome: value
    declares Some: len(rest) = len(ring) - 1;  None: len(rest) = Z, len(ring) = Z
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
    declares None: len(rest) = len(view) + 1, room(rest) = room(view) - 1
             Some: len(rest) = len(view), room(rest) = Z
seq_pop(view) -> (rest: own AppendView<'r,T>, value: own T)                  reads(view), writes(view)
    requires igt(len(view), Z)
    declares len(rest) = len(view) - 1, room(rest) = room(view) + 1
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
  linear element has no drop [PROV-6].
- **A `par` fill needs no new type**: `seq_filled`, `seq_mut_span`, and a counted
  loop of `set m[i] = ...;` under [RUN-3]'s two amendments, executed sequentially
  in a resource-closed build [RUN-2].
- **Nothing in the inventory is total at a capacity boundary.** An overwriting ring
  would need L9's published-displacement relation, and no program here needs it.

### 3.11 The pool seam, resolved

`Pool<'p, T, N>` names `N` interchangeable single-`T` slots, and a `PoolVector`
needs one **contiguous run** of them. A pool that serves *runs* of `k` slots is not
a uniform-slot domain: whether a run of 3 is serviceable is not decided by `len`,
and L6's fragmentation counterexample reappears at slot granularity.

The shape that keeps the algebra is to lease **one slot whose content is the run**:

```wf-design
region 'p {
  let blocks = pool_frame<FixedVector<Record, 256>, 8>['p]();
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
a slot. A `FixedVector` is frame-resident and region-free, so it is a legal slot
content type. Two consequences, both recorded: the capacity is fixed at reservation,
so `PoolVector` carries `N` in its type and `seq_lease` takes no runtime capacity
argument; and a program wanting two block sizes reserves two pools in two nested
regions, so `E` names both.

The inner `region 'b` is not decoration: [OWN-10] requires a borrow of a local to
name a region introduced **inside that binding's own scope**, and `'p` is introduced
before `blocks` exists. Probe `r2_2` is that rejection, probe `r2_1` is the admitted
shape, and [PROV-2] states the general rule so no example can get it wrong.

This section is a program only because [CNT-4] admits a confined generic argument
and quantifies confinement over the region set, and because [PROV-7] lets a helper
lend the pool onward.

### 3.12 One name per concept

```text
| concept                    | chosen                | why                                                     |
|----------------------------|-----------------------|---------------------------------------------------------|
| construct an empty owner   | seq_fixed<T,N> etc.   | one prefix names one family; a row is selected by name,  |
|                            |                       | receiver type and written arguments                      |
| construct a filled owner   | seq_filled<T,N>       | what array_new and buffer_new already mean               |
| append one element         | seq_push (view),      | the backing is in the receiver type, not the name        |
|                            | seq_place (owner),    |                                                          |
|                            | ring_place (ring)     |                                                          |
| remove one element         | seq_pop, seq_take,    | a view cannot remove what another view appended (L14);   |
|                            | seq_take_at,          | a ring removes from the head and says so in its name     |
|                            | ring_take             |                                                          |
| read-only view             | Span<'r, T>           | the rename is the whole change to slice<'r, T>           |
| the three measures         | len, cap, room        | one quantity, one name, term and reader alike            |
| reserve a store            | pool_frame,           | the placement is in the name, because it decides which   |
|                            | pool_extent,          | item of E the store becomes (L6)                         |
|                            | arena_frame,          |                                                          |
|                            | arena_extent          |                                                          |
| lease a pool block         | seq_lease             | capacity comes from the pool's slot type (3.11)          |
| dispose a linear value     | box_free,             | acquisition and release are symmetric (L13); each names  |
|                            | pool_release,         | the provider its acquisition named                       |
|                            | seq_release_heap,     |                                                          |
|                            | seq_release_pool      |                                                          |
| growth failure             | OutOfMemory<T>,       | L3 requires the failure to hand back the affine input;   |
|                            | PoolExhausted<T>,     | each is a struct with one field, rejected                |
|                            | NeedCapacity<T>       |                                                          |
| transform an owner         | update p by op(...);  | one spelling for the one shape value-in/value-out forces |
| rebind a consumed binding  | set p = e;            | the premise is deadness; the language gains no second    |
|                            |                       | assignment form                                          |
| the property               | resource-closed       | the long spelling is the one in use                      |
| the failure variant field  | Err(error: e)         | [PRE-1] declares Err(error: E)                           |
```

`Full<T>` and `TooSmall` are **not** in the vocabulary: no row produces either,
because the `try` forms return `Option<T>` instead.

### 3.13 Amendment register

**This register is a collation of the `Amends:` line of every rule in section 3,
and it carries nothing else.** It was written last, from the rules. A row with no
rule in its `by` column is a defect of this file, and so is a rule whose `Amends:`
line no row carries. The second draft's register was neither derived nor complete,
and every consistency defect round 2 found in it was downstream of that: a change
described in a row with no rule behind it, or effected by a rule with no row.

**Changed.** Line numbers are `spec/kernel-spec.md` at a40c7e70.

```text
| rule            | line      | change                                                                    | by                  |
|-----------------|-----------|---------------------------------------------------------------------------|---------------------|
| [SCOPE-3]       | 27-31     | heap exhaustion leaves the deferred set; stack and covered-store          | [RES-6], [STK-5]    |
|                 |           | exhaustion leave it for resource-closed programs                          |                     |
| [FORM-2]        | 52-76     | +3 rendering sentences: the result list, the destructuring let, and the    | [CALL-4], [LIV-3]   |
|                 |           | one-line update statement, on the for_stmt precedent                      |                     |
| [GRAM-2]        | 176-197   | fn_decl admits an ordered result list; program_kind admits resource_closed | [CALL-4], [RES-4],  |
|                 |           | (+1 fixed atom); struct_decl and enum_decl gain region_params?             | [CNT-4]             |
| [GRAM-3]        | 204-207   | the fixed `slice` and `buffer` productions retire; the views and owners    | [VIEW-1], [CNT-1]   |
|                 |           | are ordinary TYPEIDs with targs (META-5: fixed lowercase atoms -2)         |                     |
| [GRAM-4]        | 220-243   | let_stmt admits a destructuring binder list and return_stmt a comma-       | [CALL-4], [MSR-5],  |
|                 |           | separated list; affine_factor GAINS the [ENT-2] place grammar and the      | [LIV-3]             |
|                 |           | three measure terms and loses nothing; stmt gains update (+2 fixed atoms)  |                     |
| [GRAM-9]        | 323-327   | unchanged in force, given a stated scope: it governs runtime evaluation    | [MSR-5]             |
|                 |           | and not erased proof syntax                                               |                     |
| [TYPE-2]        | 352       | +3 provider nominals, +1 slot, +5 owners, +2 views, +3 failure structs,    | [PROV-1], [CNT-1],  |
|                 |           | slice renamed Span, buffer<T> retires from the writer surface; the         | [CNT-3], [VIEW-1],  |
|                 |           | flat-element restriction is not inherited by the owners                    | [RES-6]             |
| [TYPE-5]        | 367-374   | a fourth callee class: a container-domain row writes its complete type,    | [SEQ-0]             |
|                 |           | const and region argument list exactly when an operand supplies none       |                     |
| [TYPE-6]        | 391-403   | the container-domain operation spellings enter the lexical IDENT domain    | [SEQ-0]             |
|                 |           | and its nominals the TYPEID domain                                        |                     |
| [TYPE-7]        | 471       | slot<'p, T> joins the closed deref domain beside box and arena            | [PROV-1]            |
| [CONST-2]       | 547-551   | its naming of buffer, slice and slice_of follows the retirements           | [VIEW-1]            |
| [SET-1]         | 483-494   | "no writable target path may traverse a slice value" is restated over      | [PROV-3], [LIV-2]   |
|                 |           | loan strength, which admits the MutSpan element write; the affine-target   |                     |
|                 |           | rejection narrows to a live target                                        |                     |
| [SET-2]         | 508-513   | the region-bearing target rejection is replaced by the property it was     | [PROV-3]            |
|                 |           | protecting: no statement rebinds storage a live origin set describes       |                     |
| [OWN-1]         | 558       | providers, slots, owners and views are affine; a linear class joins copy   | [PROV-6], [VIEW-1], |
|                 |           | and affine; one reinitialization route that is not a new let; liveness     | [LIV-1], [LIV-2]    |
|                 |           | must agree at every join                                                  |                     |
| [OWN-4]         | 570       | for a lent-onward child reborrow only, the child's loan ends at the end    | [PROV-7]            |
|                 |           | of its receiving statement and the parent resumes there                    |                     |
| [OWN-5]         | 580-598   | the slice-origin paragraph generalizes to provenance-bearing values; its   | [PROV-3]            |
|                 |           | one access clause becomes two, shared-strength and exclusive-strength; a   |                     |
|                 |           | loan covers its place's address computation; the resolved origin set is    |                     |
|                 |           | defined as the set minus immutable-const                                  |                     |
| [OWN-6]         | 611       | a child reborrow may name a caller-supplied region the parent's region     | [PROV-7]            |
|                 |           | outlives-or-equals when the receiving call's result type does not name     |                     |
|                 |           | the loan region                                                           |                     |
| [OWN-11]        | 641       | the move prohibition is replaced by [LIV-1]'s join agreement; the borrow   | [LIV-1]             |
|                 |           | half is unchanged                                                         |                     |
| [STOR-1]        | 670       | the owners join the storage-class table; buffer<T>'s sentence and the      | [LIV-2]             |
|                 |           | growable-collection paragraph are superseded; the affine-set rejection     |                     |
|                 |           | narrows to a live target                                                  |                     |
| [STOR-2]        | 680       | box_new and arena_new take a provider, return Result, and carry a writes   | [PROV-2]            |
|                 |           | row                                                                       |                     |
| [STOR-3]        | 683-705   | a linear type has no compiler-derived release action; the box<T> and       | [PROV-6], [CNT-5],  |
|                 |           | buffer<T> drop rows retire; the owner and AppendView release actions join  | [VIEW-5]            |
|                 |           | the table                                                                 |                     |
| [STOR-4]        | 716       | confinement becomes the ordinary outlives relation, quantified over every  | [CNT-4]             |
|                 |           | region the value's type names                                             |                     |
| [STOR-5]        | 718       | the enumerated position list is replaced by the intensional split of       | [CNT-4]             |
|                 |           | provenance-bearing and confined types                                     |                     |
| [STOR-6]        | 733-761   | the "no numeric frame ceiling" sentence keeps its scope for the language;  | [RES-3], [STK-3]    |
|                 |           | E-materialization joins the target-stage obligations and its failure is a  |                     |
|                 |           | qualification failure citing no language rule                             |                     |
| [OP-1]          | 793-828   | +cap and +room rows beside len, whose domain extends to owners, views and  | [PROV-2], [SEQ-0],  |
|                 |           | providers and which stay pure; box_new and arena_new take a provider;      | [VIEW-1]            |
|                 |           | buffer_new, buffer_vacant and slice_of retire; ReservedLowerNames +2       |                     |
| [OP-4]          | 909       | indexable bases extend to the prefix owners, FixedRing, Span and MutSpan;  | [CNT-2]             |
|                 |           | the obligation is against len, never cap                                  |                     |
| [OP-7]          | 935       | slice_of retires; cap and room join the structural operations             | [VIEW-1]            |
| [OP-9]          | 968       | buffer_fits stays a representability predicate and additionally fixes the  | [RES-5]             |
|                 |           | target-independent constant K<T> the arena algebra and requirement use     |                     |
| [FN-1]          | 999-1070  | the slice-return ceiling generalizes to views and gains the same-region    | [VIEW-6], [CALL-4], |
|                 |           | duplicate-result rejection; the result shape admits an ordered list; the   | [RES-8], [STK-4]    |
|                 |           | boundary publishes a source-stage demand map and a target-stage own-       |                     |
|                 |           | storage figure; a loop_stmt has an edge to its normal successor if and     |                     |
|                 |           | only if some break resolves to it                                         |                     |
| [FN-2]          | 1087      | the region-bearing generic-argument rejection narrows to provenance-       | [CNT-4], [SEQ-0]    |
|                 |           | bearing arguments; explicit instantiation covers nominal region arguments  |                     |
|                 |           | and the container domain                                                  |                     |
| [FN-7]          | 1210-1253 | one new input row command.heap; one new entry marker resource_closed;      | [PROV-1], [RES-4]   |
|                 |           | main's effect row admits allocates over its own labelled provider          |                     |
| [FN-8]          | 1256      | clause operands are terms [MSR-5], not atoms                               | [MSR-5]             |
| [FN-9]          | 1295-1305 | clause operands are terms; a measured result admits len/cap/room; an       | [MSR-3], [MSR-4],   |
|                 |           | ordered result list gives one clause more than one result datum; a         | [MSR-5], [CALL-4]   |
|                 |           | parameter's measure operand denotes an entry datum, so the entry-image     |                     |
|                 |           | stability paragraph is replaced rather than repaired; M(c,q) admits a      |                     |
|                 |           | measure datum, which is always live; the direct-affine route is one step   |                     |
|                 |           | of [MSR-4]                                                                |                     |
| [ERR-4]         | 1478      | "unavailable external resources remain outside the source outcome model"   | [RES-7]             |
|                 |           | gains the two families that move inside                                    |                     |
| [PROG-3]        | 1499-1509 | the start-time obligation includes materializing the selected row of E;    | [RUN-4]             |
|                 |           | ProgramFinished is named; PreStart may descend the profile table and does  |                     |
|                 |           | not commit the entry stack it received                                    |                     |
| [EFF-1]         | 1363-1378 | allocates takes formal-rooted effect paths; the atoms heap and arena       | [PROV-4]            |
|                 |           | retire (META-5: fixed lowercase atoms -2)                                  |                     |
| [EFF-2]         | 1400-1404 | "slice parameter names the backing" generalizes to a provenance-bearing    | [PROV-3]            |
|                 |           | parameter. 1421's empty-release-row sentence is UNCHANGED and stays true,  |                     |
|                 |           | because after [PROV-6] no memory reclamation is a derived action           |                     |
| [DIAG-1]        | 1687-1712 | collision rank 5 covers the container domain; a                            | [SEQ-0]             |
|                 |           | container_declaration_ordinal joins the system one                         |                     |
| [PAR-1]         | 1969,1989 | the allocates(arena 'r) region clause becomes the ordinary provider-place  | [RUN-3], [RUN-2]    |
|                 |           | projection; execution-resource exhaustion is unreachable for a resource-   |                     |
|                 |           | closed program                                                            |                     |
| [PAR-2]         | 1994-2024 | the exclusive-loan condition is stated over loans formed by a statement of | [RUN-3], [RUN-2]    |
|                 |           | the body, so a view formed once outside the loop does not deny; the        |                     |
|                 |           | element-write form reads "a direct subscript of an array, a prefix owner,  |                     |
|                 |           | or a MutSpan"; the exhaustion sentence as above                            |                     |
| [PAR-3]         | 2049      | the exhaustion sentence as above; its replicated places cannot occur in a  | [RUN-2]             |
|                 |           | resource-closed build, which executes par sequentially                     |                     |
| [SYS-1]         | 2130      | a fourth admitted declaration source, on [SYS-1]'s own terms               | [SEQ-0]             |
| [SYS-2]         | 2158-2301 | the range-bearing operations' buffer parameters become MutSpan or Span,    | [VIEW-7], [RUN-1]   |
|                 |           | changing the inventory's normative counts; "no system operation allocates" |                     |
|                 |           | gains its companion, that every adapter-owned store is an item of E or the |                     |
|                 |           | operation is excluded by [RES-7]                                           |                     |
| [SYS-3]         | 2303      | the container domain is admitted to every compilation unit                 | [SEQ-0]             |
| [SYS-8]         | 2482-2485 | read_at, write_once, directory_next, host_copy_bytes, host_copy_utf8,      | [VIEW-7]            |
|                 |           | open_directory and open_file take &uniq 'd MutSpan<'r,u8> for a            |                     |
|                 |           | destination and &'s Span<'r,u8> for a source; the two range obligations    |                     |
|                 |           | keep their form and order with len of the borrowed view                    |                     |
| [SYS-9,11,12,14]| 2523-2620 | their normative prose naming buffer<u8> is restated over views             | [VIEW-7]            |
| [ENT-2]         | 2671-2722 | the three measure terms are one-place terms over an admitted place that    | [MSR-1], [MSR-2],   |
|                 |           | may end in a subscript; the term list gains the measure datum beside the   | [MSR-3], [LIV-2]    |
|                 |           | capture and commit-value clauses; the implicit-fact sentence gains the     |                     |
|                 |           | four standing facts; a reinitializing set is a declaration event, so the   |                     |
|                 |           | reinitialized binding is a distinct term                                   |                     |
| [ENT-3]         | 2724      | +1 enumerated source S13, the declared relations of a container-domain     | [SEQ-0], [VIEW-3]   |
|                 |           | operation, established as S12 is, with a per-variant arm route on the      |                     |
|                 |           | S7 path discipline; S5 gains absorb's commit value                         |                     |
| [ENT-5]         | 2857-2887 | a measure's support is its descriptor, its holders and every offset's      | [MSR-2], [MSR-3],   |
|                 |           | support, and a kill is any event carrying a writes occurrence that         | [CALL-5]            |
|                 |           | projects onto it; the call-boundary paragraph and the entry-image          |                     |
|                 |           | stability paragraph are replaced by the measure datum; clause (b)'s        |                     |
|                 |           | projected-callee-write kill is classified by [CALL-1..3] and nothing else  |                     |
| [ENT-6]         | 2963-3092 | one numeric goal disposition replaces the per-family route lists;          | [MSR-3], [MSR-4]    |
|                 |           | measures carry affine value images, and an image dies exactly where a      |                     |
|                 |           | fact over the same term dies                                              |                     |
| [INV-1]         | 3095-3107 | affine atoms are the [ENT-2] place grammar, the measure terms, and named   | [MSR-5]             |
|                 |           | consts, which 3107 forbids today                                          |                     |
| batch 0079      | docs/done/| the heap-refusal abort site loses its last reachable caller; the           | [RES-6]             |
| exhaustion floor| 0079-...  | guard-page record survives, and for a resource-closed build its alternate  |                     |
|                 |           | stack is an item of E [STK-5]                                             |                     |
```

**META-5 delta**, declared here because the register is its natural home. Numbered
language rules: 131 today, plus the 53 of section 3, none reusing a live or retired
id. Unique fixed lowercase grammar atoms: minus 2 for the retired `heap` and
`arena` effect atoms, minus 2 for the retired `slice` and `buffer` type
productions, plus 1 for `resource_closed`, plus 2 for `update` and `by`; net minus
1. `ReservedLowerNames`: plus 2, `cap` and `room`. Nominal types: plus 14, being 3
providers, 1 slot, 5 owners, 2 views and 3 failure structs, and one renamed,
`slice` to `Span`. Declaration domains: plus 1. [SYS-2]'s normative inventory
counts change with [VIEW-7] and are recomputed when that rule is written into the
spec, not asserted here.

**Retired outright, with no successor.** The writer-facing `&uniq buffer<T>` and
`&uniq Container` state-borrow forms ([CNT-7]); `buffer_vacant`'s `Option`-element
construction, which [CNT-2] serves over `FixedVector<Option<T>, N>`; the effect-row
atoms `heap` and `arena` ([PROV-4]); `slice_of` in favour of `seq_span`; the first
draft's `Builder<'r, T>` type and its `[BLD]` family; and the second draft's
`[STK-4]` reentrancy premise, which had no expressible instance and which the
execution-context design [1.4] owes a rule for.

**Deliberately unchanged, and why.** Each row states the reason, because a rule
this design *depends* on and does not change is exactly the rule a later batch will
change without seeing the dependency, which is how D1 arrived on `&uniq MutSpan` in
the second draft.

```text
| rule       | line | why it is unchanged, and what depends on that                             |
|------------|------|---------------------------------------------------------------------------|
| [CAP-1]    | 1962 | providers add no capability category, no permission kind, and no second    |
|            |      | interference vocabulary; [PROV-2] is judged by place overlap and the       |
|            |      | ordinary effect row, which is [CAP-1]'s own vocabulary                     |
| [OWN-5]    | 585  | "a write, move, or unique borrow of an ordinary place conflicts when that  |
|  conflict  |      | place overlaps any such origin" is the sentence that refuses a second      |
|  sentence  |      | AppendView on one owner. [VIEW-2] rests on it and amends it nowhere        |
| [OWN-7]    | 624  | subscript overlap stays conservative, which is what makes a view formed on |
|            |      | table[i] sound and what makes [PROV-3]'s address-computation freeze        |
|            |      | checkable                                                                  |
| [OWN-9]    | 634  | the optimizer consequence is unchanged; a loan held by a value is still    |
|            |      | one usable mutable path per place                                          |
| [OWN-10]   | 636  | borrow-storage duration is unchanged and is load-bearing twice: it is why  |
|            |      | a provider's confinement region can never be the loan region of a borrow   |
|            |      | of it [PROV-2], and it is what 3.11 and both worked programs obey by       |
|            |      | opening each borrow region after the binding it borrows                     |
| [EFF-2]    | 1421 | "each of these memory-reclamation actions carries the empty effect row"    |
|            |      | stays true, because [PROV-6] leaves no derived reclamation of provider-    |
|            |      | owned storage to carry a row                                              |
| [FN-6]     | 1205 | recursion stays permitted; it merely excludes a program from [RES-4]       |
| [GRAM-11]  | 340  | container-domain operations are named-argument operations of a declaration|
|            |      | domain, exactly like [SYS-2]'s, with the receiver first                    |
| [PAR-2]    | 1999 | the single-binder affine element-write refinement itself is unchanged; it  |
|  refinement|      | is the disjointness argument [RUN-3]'s loan clause relies on               |
```

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
A new pattern is owed for linear disposal, because [PROV-6] changes the shape of
every hosted helper that takes ownership of heap-backed storage.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. The
standard they are held to is that every statement is accepted by a compiler
implementing section 3's rules **and the unchanged v0.40 rules**, and both were
walked statement by statement against both before this draft was finished. Round 2
walked the second draft's pair and found five classes of refusal the design never
claimed to amend: [OWN-10] borrow-storage duration on two borrows, missing or
misplaced type and const arguments on container rows, an unformable
`seq_reserve` signature, an `absorb` whose operand's origin set could never be a
singleton, and per-arm relations consumed before their `match`. All five are fixed
here, and each fix is named where it appears.

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

fn render['p](block: own PoolVector<'p, u8, 256>, task: own Task) -> (rest: own PoolVector<'p, u8, 256>, back: own Task, written: own u64) reads(block, task.state), writes(block) contract {
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

fn drain['p, 'b](block: &'b PoolVector<'p, u8, 256>, ring: own FixedRing<u8, 256>, count: own u64) -> (rest: own FixedRing<u8, 256>, sent: own u64) reads(block, ring), writes(ring) contract {
  requires ile(count, len(deref(block)));
} {
  doc "Copies one prefix of the block into the transmit ring and reports how many bytes it placed.";
  let placed = 0_u64;
  for @copy (at in 0_u64..count) {
    let byte = deref(block)[at];
    let (next, unplaced) = ring_try_place(ring: move ring, value: byte);
    set ring = move next;
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
  let (queued, unplaced) = seq_try_place(vector: move pending, value: move first);
  match unplaced {
    None() => {
      set pending = move queued;
    }
    Some(value: rejected) => {
      set pending = move queued;
      return exit_status(code: 1_u8);
    }
  }
  region 'p {
    let blocks = pool_frame<FixedVector<u8, 256>, 8, 'p>();
    loop @queue {
      let (rest, next) = seq_try_take(vector: move pending);
      match next {
        None() => {
          set pending = move rest;
          break @queue;
        }
        Some(value: task) => {
          set pending = move rest;
          region 'b {
            let leased = seq_lease(pool: &uniq 'b blocks);
            match leased {
              Ok(value: block) => {
                let (filled, back, written) = render<'p>(block: move block, task: move task);
                region 'd {
                  let (fed, sent) = drain<'p, 'd>(block: &'d filled, ring: move ring, count: written);
                  set ring = move fed;
                }
                let emptied = seq_clear(vector: move filled);
                seq_release_pool(pool: &uniq 'b blocks, vector: move emptied);
                let stepped = advance(task: move back);
                match stepped {
                  None() => {
                  }
                  Some(value: again) => {
                    let (requeued, refused) = seq_try_place(vector: move pending, value: move again);
                    set pending = move requeued;
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
                let (fed, gone) = ring_try_place(ring: move ring, value: 33_u8);
                set ring = move fed;
                match gone {
                  None() => {
                  }
                  Some(value: shed) => {
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
| lanes = 1           | no par in the program; and [RUN-2] fixes W = 1 for a resource-     | [RUN-2]         |
|                     | closed build in any case                                           |                 |
| every slots row = 0 | no par statement, no may-suspend operation, no system handle       | [RUN-2],        |
|                     |                                                                    | [RES-5]         |
```

The layout arithmetic is `CONTAINERS.md` G7's, which is why the composition is
written and no total is. The pool is a frame item because the program wrote
`pool_frame`; `pool_extent` would have produced its own `region` item instead, which
is what a page table or a DMA ring uses and which the second draft could not
express at all.

#### Why it is source-resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | main declares pure and selects no command.heap, so [PROV-4]'s closure over   |
|                       | the call graph is empty and [RES-4] does not fire                           |
| acyclic call graph    | main -> {render, drain, advance, the container domain}; render and drain     |
|                       | -> the container domain. No cycle, so [STK-1] rewrites nothing and [STK-2]   |
|                       | passes                                                                      |
| pool demand bounded   | the lease and its release are on the same path, so the Ok arm's map is       |
|                       | (peak 1, delta 0) on the pool and the Err exit is (0, 0). The queue loop's   |
|                       | backedge delta is 0, so 3.3.1's loop rule needs no iteration bound and       |
|                       | len(blocks) <= 1 throughout                                                 |
| queue and ring        | FixedVector<Task, 32> and FixedRing<u8, 256> are frame placement, whose      |
|                       | [RES-5] row is decided at compile time and contributes no demand at all;     |
|                       | their capacities are type-level constants                                    |
| L9's displacement     | ring_try_place refuses at capacity and returns the byte, and drain reports   |
|                       | what it placed, so nothing is displaced silently                            |
| stack bounded         | one context, one chain, measured after code generation [STK-3]              |
| runtime closed        | W = 1, no task or completion records; every runtime store's peak is zero     |
```

#### The writer's-eye walkthrough

`let blocks = pool_frame<FixedVector<u8, 256>, 8, 'p>();` writes its complete
argument list because no operand supplies any of it [SEQ-0], and `'p` is a region
this function opens, which [PROV-5] requires. That requirement makes the extent's
storage lifetime and its confinement region the same thing, and it repairs a
use-after-return the second draft admitted.

`region 'b { let leased = seq_lease(pool: &uniq 'b blocks); ... }` opens a region
**after** the binding it borrows, which [OWN-10] requires and which the second
draft's pool seam and worked program both violated; probe `r2_2` is that rejection
and probe `r2_1` is the admitted shape. `'b` is also introduced inside the loop
body, for [OWN-11]'s unchanged borrow half. The call writes no arguments:
`blocks`'s type supplies the element type and both constants, and the borrow
supplies the loan region.

`match next { ... set pending = move rest; ... }` puts the `match` **before** the
rebind, on both arms. That is [SEQ-0]'s per-variant route: `seq_try_take` names
`next` as its designated outcome and its relations are established at arm entry
over `rest`, which a rebind written before the `match` would consume. This program
does not need the relations and is written correctly anyway.

`let (filled, back, written) = render<'p>(...)` is **[CALL-4]**'s ordered result
list, and `ensures ile(written, len(rest))` names two of its results, which
[CALL-4] admits and [FN-9] alone does not. Inside `render`:

```wf-design
    for @fill (
      at in 0_u64..8_u64,
      invariant spare: ige(room(view) + at, 8_u64)
    ) {
      update view by seq_push(value: mark);
    }
```

Its base holds because [VIEW-2] publishes `cap(view) = room(block)` over `block`'s
pre-transfer datum, [MSR-2]'s identity gives `room(view) = cap(view) - len(view)`,
and the `requires` gives `room(block) >= 8`. Its backedge holds because [SEQ-0]'s
declared `room(result) = room(view) - 1` transfers `room(view)`'s image while the
binder's grows by one. And `seq_push`'s `igt(room(view), Z)` follows from the header
target and S11's `at < 8` by [MSR-4]'s unordered-pair family. Probes `k21` and
`k21b` are that arithmetic at v0.40 scale, accepted and then rejected at [FN-8] when
the invariant is deleted.

`update view by seq_push(value: mark);` is [LIV-3] and is the whole loop body; the
second draft's `set view = seq_push(view: move view, value: mark);` named the target
three times and is now a [FORM-1] rejection whose fix is this line.

`set total = absorb(view: move view);` is the commit. `view`'s resolved origin set
is the singleton `{block}`, a resolved place of this function, so [VIEW-3] admits
it, and the commit publishes `len(block)' = <entry datum of len(block)> + w` over a
datum with empty support, which is exact and does not depend on surviving the kill
the same rule performs. `ensures ile(written, len(rest))` discharges from it and the
standing `Z <= len(P)`.

`seq_clear` then `seq_release_pool(pool: &uniq 'b blocks, vector: move emptied);`
is [PROV-6], written as an expression statement because a disposal returns `unit`. The lease is linear, so leaving it live at the end of `region 'b` is
[LIV-1]'s error, and disposing it names the pool the lease came from, checked by
**provenance** rather than by type: `filled`'s resolved origin set is the singleton
`{blocks}`, and a release into a second pool of the identical type in the same
region is refused with both origins rendered.

`drain<'p, 'd>(block: &'d filled, ...)` is **[CALL-1]**: a shared borrow is a kill
event for nothing, so `len(filled)` survives and discharges `drain`'s `requires`
from `render`'s `ensures`. `region 'd` is opened after `filled` is bound, for
[OWN-10]'s reason; probe `b1_own10` is the second draft's version being refused.

`loop @queue` moves `pending` and `ring` from inside the loop body, which [OWN-11]
forbade outright; **[LIV-1]** replaces that prohibition with the condition that
matters, and both are restored on every backedge and live on the `break` edge. The
loop has a resolved `break`, so [STK-4] gives it a normal successor and `main`'s
`return` is reachable.

**One deferral, stated rather than hidden.** The ring is a transmit buffer and this
program has no way to reach a device: `main`'s effect row may name only its own
labelled inputs [FN-7], and the `command` table has no device row. That is open
question Q6, and it is why 4.1 is a queue rather than a driver.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one named operation with a typed
failure, release is another, the append helper takes the view by value and returns
it, and `OutOfMemory` is a value on an ordinary edge.

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

fn grow['h](buf: own HeapVector<u8>, heap: &uniq 'h Heap, additional: own u64) -> outcome: own Result<HeapVector<u8>, OutOfMemory<HeapVector<u8>>> reads(buf, heap), writes(buf, heap), allocates(heap) {
  doc "Reserves spare capacity, handing the vector back unchanged when the store refuses.";
  return seq_reserve_heap(vector: move buf, heap: &uniq 'h deref(heap), additional: additional);
}

command fn main(command.stdout as sink: own Output, command.heap as heap: own Heap) -> status: own ExitStatus reads(sink, heap), writes(sink, heap), allocates(heap) {
  doc "Collects one fixed input buffer into a heap vector and writes it out, reporting a refusal instead of dying.";
  let input = seq_filled<u8, 4096>(value: 65_u8);
  let empty = seq_heap<u8>();
  let total = 0_u64;
  let code = 0_u8;
  region 'h {
    let reserved = grow<'h>(buf: move empty, heap: &uniq 'h heap, additional: ceiling);
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
            let outcome = write_once<'h, 'c, 'w>(output: &uniq 'h sink, source: &'c body, start: 0_u64, end: total);
            match outcome {
              Ok(value: next) => {
              }
              Err(error: problem) => {
                set code = 74_u8;
              }
            }
          }
        }
        let cleared = seq_clear(vector: move ready);
        seq_release_heap(heap: &uniq 'h heap, vector: move cleared);
      }
      Err(error: refused) => {
        let recovered = move refused.rejected;
        let emptied = seq_clear(vector: move recovered);
        seq_release_heap(heap: &uniq 'h heap, vector: move emptied);
        set code = 70_u8;
      }
    }
  }
  return exit_status(code: code);
}
```

#### The writer's-eye walkthrough

`let input = seq_filled<u8, 4096>(value: 65_u8);` is the row whose absence made the
first draft's `wfgrep` migration unreachable: `seq_fixed` gives `len = Z`, and under
[CNT-2] a zero-length container is unreadable and unwritable until elements have
been placed one at a time, so a `MutSpan` formed on it names no bytes. That
addressability requirement is also the cost [VIEW-7] records.

`let empty = seq_heap<u8>();` publishes `len = Z`, `cap = Z`, `room = Z` and
**allocates nothing**: an empty growable sequence owns no backing. That is L4 at the
constructor, and it is why `empty` is safely linear from its first statement.

`grow<'h>(buf: move empty, heap: &uniq 'h heap, additional: ceiling)` is
**[CALL-2]** on `buf` and the single acquisition point of the program. It is also
why [PROV-7] exists: `grow` lends `&uniq 'h Heap` onward to `seq_reserve_heap`,
whose result type **does not name the loan region `'h`**, which is the admitted
condition and is equally true of `pool_take`, `arena_new` and `seq_lease`. The
second draft's region-free condition admitted this call and refused all three.

On the `Ok` arm, [SEQ-0]'s relations arrive over `grow`'s pre-transfer datums:
`cap(ready) = cap(empty) + ceiling` and `len(ready) = len(empty)`. The capacity is
an **equality, not a lower bound**, which is what keeps L15 honest.

`let view = seq_append_view(vector: &uniq 'fill ready);` writes no arguments and
publishes `len(view) = Z`, `cap(view) = room(ready)`. **The view value holds the
loan** [VIEW-2], exclusively, so a second `AppendView` on `ready` is refused at its
own formation by [OWN-5]'s origin-conflict sentence, which [PROV-3] preserves.

`set total = absorb(view: move done);` is the statement the second draft could not
write: `done` is a call result, so its origin set is `{ready, immutable-const}` by
[FN-1] 1036 and is never a singleton, while [VIEW-3] requires a singleton
**resolved** set. The commit ends the loan at the consume rather than at the end of
`'fill`, which is what lets `region 'w` read `ready` immediately afterwards.

`write_once<'h, 'c, 'w>(...)` is [VIEW-7] over a view, with three regions because
they are three things: the output's loan, the descriptor's loan, and the viewed
data. `region 'c` exists for [OWN-10]. Its obligations are `ile(0_u64, total)`,
implicit, and `ile(total, len(deref(body)))`, which discharges from [VIEW-2]'s
`len(body) = len(ready)` and [VIEW-3]'s image `Z + total`. This is the statement
that makes goal A's container half real.

`seq_clear` then `seq_release_heap` is [PROV-6], on both arms. A `HeapVector` is
linear, so `region 'h` cannot be left with one alive and the disposal names the
`Heap` it came from; probe `r2_5` is the language it replaces. On the `Err` arm
`refused.rejected` is the original owner handed back unchanged (L3). **There is no
path on which the process disappears**, which is the whole of goal B.

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
section 3. The first is what a push without a capacity proof reports:

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
  its provider is Pool<'p, FixedVector<u8, 256>, 8>, held at "blocks"
  mechanical_fix: move the value out of this scope, or release it here with
    seq_release_pool(pool: ..., vector: move block); a pool-backed value has no
    compiler-derived release, so nothing else can free it

Semantics/Source [PROV-6]: ProviderProvenanceMismatch
  released value "block" came from "alpha"
  provider argument resolves to "beta"
  mechanical_fix: release a leased value to the pool it was leased from; if the
    value's origin is a set rather than one place, bind it separately on each
    branch that leases it

Semantics/Source [CNT-4]: ConfinedFieldWithoutRegion
  field "page" of struct Chunk has type PoolVector<'p, u8, 4096>, and Chunk
    declares no region parameter
  mechanical_fix: write `struct Chunk['p] { ... }`; a nominal with region
    parameters is confined by them and its instances are compared by exact region
    identity

Semantics/Source [VIEW-6]: SameRegionViewResults
  results "data_out" and "bad_out" are both AppendView<'o, u8>
  each therefore aliases every AppendView<'o, u8> parameter of this function
  mechanical_fix: give each result its own formal region
```

The last is a declaration error rather than a caller's discovery, which is the same
defect class as D1 seen from the callee's side.

---

## 5. Open questions

Everything the owner's rulings settle is dropped and not restated. So is
everything the first two drafts asked and this one answers: the length-class terms
and the goal disposition are [MSR-1] and [MSR-4]; the arithmetic residual is
[MSR-3]'s datums and images; the `absorb` commit is [VIEW-3]; the coverage
certificate died with `Builder`; the arena's reclamation is [RES-5]'s cursor
domain; the optimizer-versus-envelope question is [STK-3]; the profile table is
[RES-2]; three-term relations route through the affine domain [MSR-4]; a view's
measures and an owner's are distinct terms plus the [VIEW-2] equalities. Four
questions the second draft filed are **answered here rather than asked**, and the
answers are on the merits:

- *What disposes a provider-owned value?* A linear obligation, [PROV-6]. The second
  draft recommended the derived release and recorded the linear form as a cleaner
  endpoint; round 2 showed the derived release cannot state its own subject, cannot
  be written for a bulk drop, and hides a free the parallel judgment must see.
- *Do region-parametric nominals belong in this version?* Yes, [CNT-4]. The
  deferral bought no soundness once confined values were admitted into container
  elements, and its cost was every kernel structure written as parallel columns.
- *Should the value-in / value-out spelling get sugar?* It gets a statement form,
  [LIV-3], which is not sugar because it reaches places `set` cannot.
- *What about control entering the call graph from outside it?* Out of scope for
  this batch, and section 1.4 states the interface the execution-context design
  inherits instead of filing a question the language cannot yet pose.

What remains is what this design genuinely does not decide.

**Q1. May a resource-closed program handle a typed refusal, or must it prove every
acquisition?** *(a)* Strict: every covered acquisition uses the proved spelling.
*(b)* Permissive: both spellings are admitted, since neither can ask for more than
`E`.
**Recommend (b), and L8 plus [RES-6] make it real.** A refusal edge now carries the
store's own `room(store) = Z`, and 3.3.1's loop rule names the checked spelling as
one of the three things that bounds a retaining loop, so the permissive form
changes a summary rather than being ignored by it.

**Q2. Where does a hosted resource-closed program's large memory come from?**
*(a)* Frame and extent placement only, as [PROV-5] provides. *(b)* One more entry
row delivering a committed region, `command.region as store: own Arena<'store>`.
**Recommend (a).** `pool_extent` and `arena_extent` already produce a `region` item
of `E` that a deployment grants separately, which is what the second draft's
frame-only placement could not express. (b) becomes right the day a program needs a
store whose *size* is a deployment decision rather than a source constant, and it
puts a deployment-shaped input on every hosted program's entry, so it should wait
for a program that needs it.

**Q3. What relation admits a `par` fill over disjoint ranges of one view?**
[PROV-3] says two accesses through one origin conflict, which is what makes views
sound; a `seq_split_at` and divide-and-conquer over a span need a *second*
relation, disjointness of ranges over one origin.
*(a)* Give each origin the half-open index range its value reaches, maintained by
every rule that forms, moves, passes, returns and reborrows a provenance-bearing
value, with [OWN-7]'s overlap test extended to ranges. *(b)* Do not; a `par` fill
goes through [SEQ-0]'s filled construction plus a `MutSpan` plus direct subscript
writes, under [RUN-3]'s loan amendment.
**Recommend (b) now and (a) as the successor.** The second draft recommended (b)
and did not deliver it, because [PAR-2] denies permission on a loan formed outside
the loop; [RUN-3] amends the condition that is actually wrong, which is one
sentence and is the same argument [PAR-2]'s own element-write refinement already
makes. (a) is the general answer, it is what a splitting API needs, and it should
be written properly in one place rather than approximated by refining one loan.

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
executes `par` sequentially there and publishes `lanes(1)`, because the current
runtime's wait path executes a stolen task on the waiting lane's own stack and no
term of [STK-3] counts that. *(a)* Restrict `par` shapes to those whose lowering
cannot nest a stolen task, and execute the rest sequentially. *(b)* Build the
compiler-managed work-first continuation representation, then lift the restriction
and define a worker lane's chain [STK-3].
**Recommend (b), and note that [RUN-2] is what makes it a scheduling item rather
than a soundness risk.** This is the largest engineering item the design implies.
Until it lands, a resource-closed program that wants overlap gets none, and says so
in its published row.

**Q6. Does this version want a keyed or sparse container family?** [CNT-2] writes
stable-identity storage as `FixedVector<Option<T>, N>` with element-position
`replace`, which is sound, is L12-clean, and compiles in shape today (probe
`r2_7`). Its cost is one construction loop of `N` places, one `Option` word per
slot, and one `match` per read. *(a)* Leave it there. *(b)* Add a `FixedTable<T, N>`
whose typestate is an occupancy set, whose whole operation surface is index-local
so no quantified proposition arises, and whose occupancy word is representation
rather than language state.
**Recommend (a) for this version and (b) as the next container family.** (b) is
what a kernel object table, a page cache and a slab front end actually want, and it
is exactly the "keyed containers are fixed families over the core, later" the owner
settled; landing it here would make this batch two designs.

**Q7. Should a system operation be able to append?** [VIEW-7] gives a destination
`&uniq 'd MutSpan<'r, u8>`, so an I/O buffer is addressable before the host writes
into it and the byte count comes back as an ordinary `u64` beside the container.
*(a)* Leave it: one memset per buffer, and the length typestate does not reach the
boundary where lengths come from outside. *(b)* Give the producing operations
`own AppendView<'r, u8>` and an ordered result list, so the bytes the host wrote
become the view's `len` and `absorb` publishes the owner's new length.
**Recommend (b), in the batch that lands multi-return in the [SYS-2] declaration
domain, and not here.** (b) is the right answer and it is the one place where the
whole write-back protocol pays for itself twice; it also requires the system domain
to gain a result-list shape, which is a change to [SYS-2]'s records and counts and
belongs beside them.

**Q8. Is `copy` structural over aggregates?** [OWN-1] makes every owned composite
affine regardless of its field types, which is why `seq_filled`, `array_new` and
`seq_heap_filled` admit only primitives and why P17 exists. *(a)* Leave it.
*(b)* A `struct` or `enum` all of whose field types are copy is itself copy.
**Recommend (b), and note that it is not this design's to land.** Three rows of
[SEQ-0] are restricted by it, and under (b) `seq_filled<Descriptor, 64>` and
`seq_filled<Option<u64>, 64>` become constructions instead of loops. It is an
[OWN-1] question with its own consequences across the language, and this design
names it because it is the reason three of its own rows read `T copy`.

**Q9. Is `E` part of program identity?** *(a)* Diagnostic output only. *(b)* An
emitted machine-readable table beside the object.
**Recommend (b), and explicitly not part of [PROG-2] compilation-unit identity.**
The envelope is useless if the deployment cannot read it, and keeping it out of
unit identity keeps it a derived fact about one build, which [STK-3] and [RUN-5]
both say it is.

---

## 6. Verified versus reasoned

**Verified** means a compiler executed it. The binary is the gate-profile
`whitefootc` built from this tree; every probe below was run against it, either in
the session that wrote this file or in one of the eight falsifier sessions whose
verdicts are quoted with their probe names. No timing figure from any machine
appears anywhere in this file.

### 6.1 What the current compiler does

Eleven probes were run in the session that wrote this draft, to check what this draft
newly rests on rather than to re-inherit earlier verdicts. The table below describes each probe program closely enough to rewrite it; the sources were session scratch files and are not in the repository.


```text
| probe               | program                                                      | verdict                                   |
|---------------------|--------------------------------------------------------------|-------------------------------------------|
| r2_1_region_after   | two &uniq argument borrows of one outer binding in one region,| ACCEPTED, exit 0                          |
|                     | plus a nested region opened after an inner binding            |                                           |
| r2_2_region_before  | a shared borrow of a binding declared inside the borrow region| REJECTED [OWN-10] InvalidBorrowLifetime   |
| r2_3_targ_order     | fn peek<T>['r](...) called as peek<Cell, 'r>(...)             | Semantics/Unsupported: Generics           |
| r2_3b_targ_order    | the same, called as peek<'r, Cell>(...)                       | Semantics/Unsupported: Generics           |
| r2_4_rootkill       | hoisted len(deref(ring).flags), then set deref(ring).tail     | REJECTED [OP-4], residual                 |
|                     |                                                              | "at < len(deref(ring).flags)"             |
| r2_4b_control       | the same with the sibling-field write deleted                 | ACCEPTED, exit 0                          |
| r2_4c_elemwrite     | the same with an element write into flags instead             | ACCEPTED, exit 0                          |
| r2_5_boxdrop        | fn swallow(item: own box<u64>) -> own u64 pure, dropping it   | ACCEPTED, exit 0                          |
| r2_6_structregion   | struct Wrap['p] { page: arena<'p, u64>; }                     | REJECTED [GRAM-2] at parse, expected {, < |
| r2_7_optionslots    | replace slots[i] = Some(...) and = None(), then len(slots)     | ACCEPTED, exit 0                          |
|                     | reaching a guard                                              |                                           |
| r2_8_update         | update line by seq_push(value: 7_u8);                         | REJECTED [FORM-1] at parse                |
| r2_9_armgap         | one statement between a +checked call and its match, then a    | ACCEPTED, exit 0                          |
|                     | second checked operation inside the Ok arm                     |                                           |
| r2_10_definelen     | define capacity = len(deref(destination)) on a &'d parameter,  | ACCEPTED as pure; REJECTED [EFF-2]        |
|                     | used in a requires                                            | EffectMismatch when reads(destination)    |
|                     |                                                              | is declared                               |
```

What each establishes, and which rule it changed rather than confirmed.

- `r2_1` and `r2_2` are [OWN-10] and they changed two programs and one rule: every
  borrow of a local must name a region opened **after** the binding, and [PROV-2]
  now states the general reason so no example can get it wrong. `r2_1` also
  re-confirms that two `&uniq` argument borrows of one place in one region coexist
  as call-scoped temporaries, which is why [VIEW-2]'s loan belongs to the view value.
- `r2_3` and `r2_3b` reached no verdict: user generics are `Semantics/Unsupported`
  today, so no target order for mixed arguments can be measured and [GRAM-2]'s
  declaration order is the only authority. Section 7 carries the consequence, that
  the container domain needs monomorphization the compiler does not have.
- `r2_4`, `r2_4b` and `r2_4c` bound the measure kill exactly: a sibling-field write
  kills today, and deleting it or making it an element write does not. [MSR-2] makes
  the support the descriptor rather than the root, so the current implementation
  becomes a defect to repair rather than the rule.
- `r2_5` is L13's evidence: a heap free happens inside a callee that declares
  `pure`, so no signature mentions it and no [PAR-1] footprint can see it.
- `r2_6` and `r2_8` show that region-parametric nominals ([CNT-4]) and `update`
  ([LIV-3]) are new syntax and not compiler defects.
- `r2_7` is [CNT-2]'s stable-identity claim at v0.40 scale, and `r2_9` bounds
  [SEQ-0]'s arm route: an intervening statement does not break an arm-fact route,
  while a statement that consumes the result the relations name would.
- `r2_10` is [SEQ-0]'s purity claim from both sides. A `reads` row on the readers
  would remove `len` from every `contract_define`, which is `wfgrep`'s accepted
  shape and P16's.

Inherited verdicts this draft still rests on, from the eight falsifier sessions:

```text
| probe                       | verdict                                                                   |
|-----------------------------|---------------------------------------------------------------------------|
| conformance case, d1        | ACCEPTED, exit 0; D1 reproduces at this tip                               |
| p1                          | REJECTED [OP-4], residual "9_u64 < len(b)": [CALL-2] already behaves      |
| p6                          | ACCEPTED: [CALL-1] already holds                                          |
| p7                          | REJECTED [SET-1], root_class "slice view": MutSpan is new capability      |
| p2 / p4                     | REJECTED [GRAM-9] / ACCEPTED: [CALL-4]'s two halves                       |
| p8, k09, r1_multi           | REJECTED [GRAM-2]: multi-return is new syntax                             |
| p9, k12                     | REJECTED [OP-1]: affine elements have no construction route today         |
| p10 / p11                   | REJECTED [STOR-1] / [OWN-1]: the two halves of [LIV-2]'s premise          |
| p5_ambient, n4, r1_ambient  | ACCEPTED: L2's evidence                                                   |
| f1c, f1d, f2b, r1_twouniq   | ACCEPTED / [OWN-5] / ACCEPTED / ACCEPTED: why a view value holds the loan |
| f3, f5, f6, r1_own11        | Unsupported OwnershipJoin; [OWN-11] twice: the three avoidances [LIV-1]    |
|                             | replaces                                                                  |
| f7                          | REJECTED [OP-4]: D1 is narrow, so [CALL-5]'s default is right elsewhere   |
| f2b_tail, f8_tailframe      | ACCEPTED: the witnesses that refute the syntactic tail conditions         |
| n2_idle, f3_forever, k30    | REJECTED [FN-1] FunctionFallthrough: the idle loop                        |
| n3_propagate_loop           | REJECTED [FN-1]: the driver loop the second draft's amendment still refused|
| f7_regionresult, k05        | REJECTED [FN-2]: why [CNT-4]'s generic-argument half exists               |
| k04                         | REJECTED [OWN-3] UnresolvedUse, TypeRegion: a struct has no region binder |
| f5b                         | ACCEPTED, both rows silent: why [PROV-4] reads the leaf's type            |
| r1_relend, r1_relend_affine | REJECTED [OWN-6]: why [PROV-7] exists                                     |
| r1_lenatom, r1_field, q1, q9| REJECTED [GRAM-4] at parse: [MSR-5] is an amendment, not a defect         |
| r1_const                    | REJECTED [FORM-3]: a named const is lowercase                             |
| k21 / k21b                  | ACCEPTED / REJECTED [FN-8]: the fill loop's arithmetic works today        |
| k08, k31, b4b_loopregion    | ACCEPTED: the guard route, and a region opened inside a loop body         |
| k15, k07                    | ACCEPTED: a cursor advanced on one arm; a modulus-derived bucket write    |
| k18, k23, k24               | ACCEPTED: a heap slab; buffer_vacant plus element-position replace        |
| k19, k25, k16b, k26         | REJECTED [OWN-1] / [OP-1]: why three rows read "T copy" (Q8)             |
| q6, q13, q14, q24           | ACCEPTED: sort, open-addressed table, demux, len chain                    |
| b1_own10                    | REJECTED [OWN-10]: the second draft's own drain borrow                    |
| b2_resultname, b3_forhdr    | ACCEPTED: a body let reusing the result binder; a two-line for header     |
| b5_define_len               | ACCEPTED: a contract define over len                                      |
| n6_regionidentity           | ACCEPTED: two arena values of one complete type in one region             |
| n5_boxfield                 | ACCEPTED: a box field dropped in a callee naming no heap                  |
| n7_par with --par-ledger    | PAR permitted, eligible                                                   |
| --stack-ledger              | one flat number for every context; three disjoint chain roots             |
| tests/programs (six)        | ALL ACCEPTED, exit 0                                                      |
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
  against both; that is a stronger claim than the second draft's and it is still a
  claim, not a verdict.
- **Every figure in 4.1's envelope**, which is why every one of them is written as
  a composition or as `<post-codegen>` rather than as a number.
- **The composition algebra of 3.3.1.** Its sequence and branch rules over an
  exit-label map are standard, and the no-fallthrough case is now defined. Its `par`
  rule depends on a runtime profile that does not exist. Its loop rule's third
  discharge, a writer invariant, has never been exercised against a program with two
  stores.
- **[MSR-3]'s measure datum.** It is [ENT-2]'s own device for a capture term and a
  commit value, extended to two more producers; that it composes correctly with
  [ENT-5]'s pre-kill closure at a call boundary and at a reinitializing `set` is
  argued and not executed. This is the rule the whole surface now rests on and it
  should be attacked first.
- **[PROV-6]'s linearity.** That no program needs a linear value to reach a scope
  exit alive, that `requires ieq(len(vector), Z)` on the two release rows is a cost
  a writer can always pay, and that the virality is bounded in practice are all
  claims about programs nobody has written.
- **[PROV-3]'s provenance over provider-derived values.** The mechanism is
  [OWN-5]'s, and [OWN-5]'s own soundness argument is about *loans*, which a
  provider-derived value does not hold. Whether the origin set is preserved
  correctly through a container element, a `Result` payload and a multi-return is
  argued from the four preservation sentences and not checked.
- **[STK-1]'s deadness premise.** It refuses the two witnesses the syntactic list
  admitted, and [PROV-5] closes the confined-value route round 2 found. Whether it
  admits every component a correct rewrite could take is not proved.
- **Everything about the current runtime's closure.** [RUN-1] is written as a
  qualification obligation precisely because no existing target can be certified to
  meet it, and the `--stack-ledger` read above shows the entry chain is presently
  three disjoint roots.
- **The claim that `wfgrep` becomes heap-free.** Its eleven `buffer_new` calls
  reach three declared rows, all of which [SEQ-0] and [VIEW-7] replace. The
  substitution was not performed and compiled, and it moves bytes out of the heap
  and into frames, which is a [STK-3] question rather than a free win.

### 6.4 Falsifiers this design asks for next

1. Attack [MSR-3]'s measure datum with a `propagate` edge, a `value_if` delivery,
   and a datum whose call is inside a loop body that the loop-header kill rewrites.
2. Attack [PROV-6] with a linear value that reaches a scope exit only on a
   `propagate` path, and with a linear element inside a confined container inside a
   region-parametric nominal.
3. Attack [PROV-3]'s preservation with a provider-derived value that crosses a
   multi-return, a container element and a `Result` payload in one program, and
   check that the resolved origin set is still a singleton where a disposal needs
   one.
4. Hand-execute 3.3.1 on 4.1 and on a breadth-first walk, and check that the
   repaired loop rule distinguishes them.
5. Attack [MSR-2]'s descriptor-precise kill with a `deref` chain, a subscripted
   descriptor, and a callee whose [EFF-2] path is the enclosing aggregate because
   [EFF-1] admits no dynamic selector.
6. Attack [CNT-4]'s invariance with a region-parametric nominal used at two
   different regions in one function, and with one nested inside another.
7. Rewrite `wfgrep` by hand against [VIEW-7] and [PROV-6] and count what the
   `MutSpan` destinations and the explicit releases cost at every call site.
8. Attack [LIV-3] with an `update` whose target is a subscripted place whose offset
   the operation's own arguments read.

### 6.5 Falsifier round 1: what each finding hit, and what refuses it now

Every BREAKS, DEFECT and BLOCKING finding of the first four reports, one line each,
with the rule that refuses it in **this** draft. Where round 2 reopened a finding,
the row says so and 6.6 carries the repair. The reports are superseded.

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
each. Round 2's diagnosis was that round-1 repairs were added piecemeal; the right
column is therefore mostly the same six concepts, which is the point.

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
part round 2 praised, which is why this draft drafts five instead of two.

---

## 7. Implementation order

Twelve batches, re-derived from the rules this draft states. Each names the rules
it implements and the test it adds, and **every rule of section 3 appears in
exactly one batch**; round 2 found two rules unbatched and one batch whose test
needed three later batches. This is an ordering, not a design choice; nothing here
may be read as trading a rule away for a cheaper batch, and nothing here is an
approval or a schedule.

Three hard constraints the ordering obeys. The operation inventory is written in
the syntax B3 introduces, so multi-return and the transformation statement come
before any operation that returns two results. The container domain is **generic**,
and probe `r2_3` shows user generics are `Semantics/Unsupported` today, so B5 is
the batch that first needs monomorphization for a compiler-owned domain and it must
carry that work. And no batch's test may need a later batch's rules.

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
`v25`/`v26` so two consumers of one exported invariant agree; one mirroring `q9`, a
struct field path as an affine atom; one pinning that a literal and a parenthesized
group are still affine factors; **and `r2_4`'s program accepted**, because
[MSR-2]'s descriptor-precise support is a repair of a live over-kill and not only a
new rule.

**B3. Multi-return, the destructuring `let`, join-checked liveness, and the
transformation statement.** Rules: [CALL-4], [LIV-1], [LIV-2], [LIV-3]. Third
because B5 and B7 are written in this syntax. Tests: probe `p8`'s signature parses
and binds; probe `p10`'s program is accepted and probe `p11`'s repair is
unnecessary; probe `f3`'s program is a [LIV-1] error naming both predecessors
instead of `SemanticUnsupported`; a loop moving and restoring an outer binding is
accepted where probe `f5` is [OWN-11] today; probe `r2_8`'s `update` parses and
lowers to the same store as its `set` spelling; and the `set` spelling of a
single-result container-domain call is a [FORM-1] rejection naming `update`.

**B4. Measure datums and images.** Rules: [MSR-3]. Separated from B2 because it
touches [ENT-2]'s term list, [ENT-5]'s call boundary and [ENT-6]'s transfer
machinery rather than route lists, and because it needs [LIV-2] from B3. Tests, all
writable in **today's** vocabulary plus B2 and B3, which is the constraint round 2
found the second draft's B4 breaking: a v0.40 `buffer` helper whose `ensures` names
`len` of a parameter it consumed is accepted; the same helper's caller establishes
the declared relation on the result where `M(c,q)` refuses it today; a
reinitialized binding does not inherit a fact stated over its predecessor, which is
round 2's rank-one program written as a negative case; and an image is unavailable
after a projected callee write, pinning `g1` against `g1b`.

**B5. Owners, typestate, confinement, and the declaration domain.** Rules:
[CNT-1], [CNT-2], [CNT-3], [CNT-4], [CNT-5], [CNT-6], [SEQ-0] and the constructor,
place, take, exchange, clear and ring rows. Retires `buffer<T>` from the writer
surface. Carries monomorphization for a compiler-owned generic domain. Tests: a
`FixedVector<Handle, 64>` object table with affine elements, filled by
`seq_filled`, compacted by `seq_take_at` and reordered by `seq_exchange`, accepted,
where probe `p9` is [OP-1] today; a `FixedRing<Descriptor, 64>` whose elements are
read and written by subscript; `Result<PoolVector<'p, T, N>, E>` accepted where
`f7_regionresult` is [FN-2] today; `struct Chunk['p]` accepted where probe `r2_6`
is a parse error today, with two instances at different regions rejected as
distinct types; and the `FixedVector<Option<T>, N>` stable-identity shape of
[CNT-2], which is probe `r2_7` in the new vocabulary. This batch supersedes B1's
conformance case, whose program no longer typechecks; that disposition is
conformance evidence and is recorded in `governance/APPROVALS.md` with the merge.

**B6. Views, loans, and the commit event.** Rules: [VIEW-1] to [VIEW-6], [PROV-3],
and the view rows of [SEQ-0]. [PROV-3] lands here rather than in B7 because views
are its first user and because [SET-1] and [SET-2] must change in the same batch
that admits the `MutSpan` write. Tests: an element write through a `MutSpan` is
accepted where probe `p7` is [SET-1] today; **a `replace` through
`&uniq MutSpan<'r,u8>` is rejected**, which is round 2's D1-one-type-over written
as a negative case; two `AppendView`s formed on one owner are rejected at the
second formation citing [OWN-5]; a write to `k` while a view formed at `table[k]`
is live is rejected citing the view's loan; an owner is readable immediately after
`absorb` with no enclosing region, and `absorb` is admitted on a view that crossed
a call; and a two-result signature with two same-region view results is rejected at
[VIEW-6].

**B7. Providers, the heap as a value, and linear disposal.** Rules: [PROV-1],
[PROV-2], [PROV-4], [PROV-5], [PROV-6], [PROV-7], [RES-6], and the provider-bearing
[SEQ-0] rows. Tests: probe `p5_ambient`'s program is **rejected**; a `main` that
omits `command.heap` cannot reach any allocation; probe `r2_5`'s program is
rejected with [PROV-6]'s diagnostic and its repair compiles; a lease released into a
second pool of the identical type in the same region is rejected with both origins
rendered; a reserving operation naming a caller-supplied region is rejected at the
`targ`; a helper lending a provider onward to `pool_take` compiles, where
`r1_relend` is [OWN-6] today; and two overlapped statements that only free are
denied [PAR-1] permission.

**B8. System I/O over views.** Rules: [VIEW-7]. Test: `tests/programs/wfgrep.wf`
migrated to `seq_filled` and `MutSpan`, compiling with no `allocates` entry
anywhere on its call graph. It is the first program that demonstrates goal A's
container half end to end, and the migration is also the measurement Q7 needs.

**B9. The stack judgment.** Rules: [STK-1], [STK-2], [STK-3], [STK-5]. Tests:
probes `f2b_tail` and `f8_tailframe` are **not** rewritten by [STK-1] and are
rejected by [STK-2] under the marker; their borrow-free variants are rewritten and
accepted; a member holding a live confined value across the jump is likewise not
rewritten; probe `p3_rec` stays accepted without the marker; and a `--stack-ledger`
run reports one chain per context rather than disjoint roots.

**B10. The divergent entry.** Rules: [STK-4]. Small and separable, and it is the
batch a kernel writer notices first. Tests: probe `f3_forever`'s idle loop is
accepted; **probe `n3_propagate_loop`'s driver loop is accepted**, which the second
draft's amendment would still have refused; and a loop with a reachable `break`
still requires a return.

**B11. The envelope and the judgment.** Rules: [RES-1], [RES-2], [RES-3], [RES-4],
[RES-5], [RES-7], [RES-8], [RUN-1], [RUN-4], [RUN-5]. Tests: section 4.1's program
is source-resource-closed and its `E` table matches a pinned symbolic expectation;
section 4.2's is reported not resource-closed with the heap-reaching path rendered;
a retaining loop whose trip count is a runtime value is rejected at that loop with
the value named, which is the repaired loop rule under test; a loop whose only
discharge is the standing `len <= cap` is likewise rejected; and a program whose
runtime demand exceeds every profile row fails **target qualification** citing no
language rule, which is the two-stage split under test.

**B12. `par` and the envelope.** Rules: [RUN-2], [RUN-3]. Tests: a `seq_filled`
plus `MutSpan` plus counted subscript fill receives [PAR-2] permission, which the
second draft's own claim could not; the same loop inside a `resource_closed` entry
is executed sequentially and the published row reads `lanes(1)`; and the `par` rule
of 3.3.1 composes against a pinned profile row for an unmarked program.

Two items sit across the batches. **Monomorphization** for a compiler-owned generic
domain is B5's, and nothing before B5 needs it. **Q5's continuation redesign** is
the largest engineering item any of this implies; [RUN-2] lets B11 and B12 ship
without it at the cost of `lanes(1)`, and lifting that restriction is a batch of its
own after B12.
