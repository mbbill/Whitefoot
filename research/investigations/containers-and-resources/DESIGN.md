# Containers and resources: the integrated design

The single design for batch 0116: one set of laws, one set of rules, one
vocabulary, one amendment register. `RESOURCES.md` beside it keeps the writer's-eye
resource migrations and `CONTAINERS.md` the longer library functions of 3.L; neither
carries rule text, and a reader who reads only this file has the whole design.

**Sixth draft, after falsifier round 5 and the owner's rulings of 2026-09-03
evening.** Round 5 confirmed the fifth draft's two structural moves — the partition
test and the region-spelling assumption — and then found the same defect in four
independent lenses: *a notion was introduced without the closure the brand got*.
Store identity closed in round 3 and has not moved since; activation and release
closed in round 4 and held; what round 5 broke was **accounting** (eight BREAKS
against one composition algebra that has no transfer for the release its own
canonical idiom performs, no map for the one loop shape goal A exists for, and no
definition of the term it is indexed by), **measure data** (a three-term algebra
whose third term is derivable only outside `AUTO`'s declared budget, so every
appending loop in the file is refused), **linearity** (keyed on the store's
reclamation discipline, so a library that recycles values cannot say that its
values must come back), and the **elision assumption** (a criterion that gets its
two flagship cases backwards and produces an empty candidate set at exactly the
position both worked programs need).

The owner's two rulings reshape the answer rather than adding to it.

> **R1. Parameters are inputs, results are outputs.** The fifth draft's `&uniq`
> block and container parameters for helpers are withdrawn, and with them
> [CALL-4]'s exit datum: F1's first attack showed the exit datum is a caller-side
> object the callee cannot own, so a callee proved the entry fact and the caller
> read it as the exit fact — D1 restored through the rule written to close D1.
> Helpers are value-in, value-out, with multi-return and the exchange admission of
> `set`; a contract then relates inputs and outputs with no entry/exit convention.
> In-place mutation exists only through length-fixed views.
>
> **R2. Linear is the reclamation half of affine, and the modifier is for logical
> obligations only.** `affine` today does two jobs — ownership and lifetime, and
> store reclamation — and the second is linear's. The criterion is stated once and
> everything is derived from it: *a value whose release action requires a
> capability is linear; a value whose release requires nothing is affine.* With the
> heap an explicit capability value, an implicit scope-exit free would have to
> smuggle the capability, which is exactly what L2 forbids. Linearity is closed
> under containment, so **no writer ever marks a store-derived type**: it is read
> off the type. A `linear` modifier on a user nominal therefore exists only for
> **logical** obligations — a transaction that must commit or roll back, a request
> that must be answered, a lease that must go back to a specific pool when the pool
> is library code holding an affine run. A linear value must be moved,
> destructured, or disposed before every leaving edge.

R1 is the deeper of the two. It removes a rule rather than adding one, it restores
[CNT-7]'s effect without [CNT-7]'s text, and it makes the non-shrink guarantee L14
was retired for **statable again**: `ensures len(rest) >= len(out)` relates a
result to an input, is single-state in every sense the owner's ruling used, and
needs no `old()`, no frame rule and no third view. Q0a is answered rather than
traded.

R2 is the smaller change and the more honest one, and its cost should be read
before its benefit. **Every heap-derived value in a hosted program is now disposed
explicitly, with the `Heap` in hand.** `byte_string.wf` under this draft carries
seven visible frees where today it carries none, and 3.L.5 counts them. That is the
price of L2: once the store is a value you must hold in order to allocate, it is a
value you must hold in order to free, and a compiler-derived free at a scope exit
would have to reach a capability the scope may not hold. The way to write less is
not to make the free implicit but to **use a region block or an arena**, whose
reclamation needs no capability and whose values are therefore ordinary affine
values with an ordinary derived release. That is the trade goal A already wanted a
writer to make, stated where the writer meets it.

Tree read: `batch/0116-containers-and-resources` at `main` 30602914,
`spec/kernel-spec.md` **v0.41 ACTIVE**. Bare three- and four-digit line numbers
are that file at 30602914, re-derived in this session against v0.41; every other
citation names its file. v0.41 respelled the six integer comparisons as infix
`== != < <= > >=`, delimited call-site type and region application with `::`, and
put the four ordered symbols in proof position; every clause, invariant and call
below is written in that surface, and 6.9 records that the change is a respelling
with no effect on any finding of round 5.

**Nothing here is implemented.** No compiler code was written for it. Section 3.K
is draft rule text for a work branch, not an amendment; section 3.L is design text
for programs that compile nowhere. Section 6 separates what a compiler executed in
this session from what is argued on paper.

Settled by the owner, and not reopened anywhere below:

- The heap is an explicit capability **value** handed to `main`, so heap-freedom
  is a signature fact.
- `resource-closed` is a derived, writer-requirable property over an envelope `E`
  of tangible resources; a general heap, including a bounded general heap, is
  never part of `E`.
- No frame-accumulating recursion in v1; tail recursion is lowered.
- `FixedVector<T, n>` holds affine `T` through a checker-maintained typestate.
- The core is a contiguous run of initialized slots; keyed containers are
  fixed families over it, later.
- Owners versus affine views, transformed by value, with single-state `ensures`
  under [FN-9]. Two-state `ensures` is rejected.
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

Six footnotes, because the minimality ruling and R1 move material the settled list
names. Each states what survives and what changed, and each is a decision the
owner has not separately ruled on; 5.0 collects them.

1. **The owner inventory.** The settled list names four owners. `FixedVector<T,
   n>` is unchanged. `HeapVector`, `ArenaVector` and `PoolVector` were three names
   for one shape at three stores; with [PROV-1] putting the store in the brand,
   nothing else distinguished them, so they are one kernel nominal `Vector<'s, T>`
   at two regions, and the three names survive in 3.L as what a writer calls an
   instance.
2. **`FixedRing`.** The fourth draft made it a fifth owner; the fifth draft made it
   a library `FixedVector<Option<T>, n>` plus a head and a fill, which round 5
   measured at seven times the memory of a hand-written byte ring and which deleted
   in-place slot mutation. This draft makes it **neither**: [BLK-1]'s typestate is a
   window rather than a prefix, so a ring *is* a run, with no `Option`, no tag, and
   ordinary element access. 3.K.3 states what the window costs.
3. **`AppendView`.** The settled list names the owner/view split and the by-value
   transformation, both of which survive. `AppendView` and `absorb` were the fourth
   draft's device for keeping a caller's length alive across an appending callee;
   the fifth draft replaced them with [CALL-4]'s exit datum, which R1 withdraws.
   Under R1 an appending helper takes the run **by value and returns it**, so the
   caller's length is the *result's* length and no device is needed at all. 5.0
   states what is lost, which this time is nothing.
4. **`update`.** The fourth draft's transformation statement is not a new
   statement here. Its one unwritable half — transforming a place through a call
   with no observable point between the read and the write — is an admission on the
   existing `set` [LIV-3]; its other half was sugar and is gone.
5. **Argument order.** The settled append example writes its source argument
   first, while [GRAM-11] fixes argument order from the declaration and every
   helper here declares its destination first.
6. **`seq_exchange`.** The fifth draft made it a kernel row and named it the fifth
   of seven additions. Round 5 wrote it in wf in three statements over rows the
   kernel already has, so L18 removes it; 3.L.2 carries the three statements and
   states the one real cost of writing it that way.

## Contents

1 [The problem](#1-the-problem) · 2 [The laws](#2-the-laws) and
[the eight notions](#21-the-eight-notions-and-their-closures) ·
3 [The rules](#3-the-rules): [3.K kernel](#3k-kernel-rules),
[3.S surface proposals](#3s-surface-change-proposals),
[3.L library](#3l-the-library-written-in-wf) ·
4 [Two worked programs](#4-two-worked-programs) · 5 [Open questions](#5-open-questions) ·
6 [Verified versus reasoned](#6-verified-versus-reasoned) ·
7 [Implementation order](#7-implementation-order) ·
[Appendix A](#appendix-a-generated-data)

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
here to remove. Neither has one that silently stops making progress at three in the
morning because it lost the last block of a pool it owns, which is what round 5
found in this design's own flagship program and which R2 closes.

**Goal B: with a heap, be honest.** A hosted program wants the heap and should have
it. What it must not have is a hidden trap, and it has two today. Allocation is
ambient: any function may allocate while holding nothing, and refusal ends the
process. And release is invisible: probe `r2_5` compiles a function that takes
`own box<u64>`, never returns it, and declares `pure`. Goal B asks for both halves to
be values: allocation is an operation on a provider the caller holds, refusal is an
ordinary typed outcome that hands back every affine input it did not consume, and
release is a statement that names the same provider.

Both goals are one language. There is no subset mode, no second prelude, and no
dialect: the same rules judge every program, and one entry marker turns the
failure to establish the property into a compile error instead of a note.

### 1.2 The concrete failure: D1

The sweep of 2026-09-03 found an unsound accept that is exactly the defect this
design has to make unrepresentable. The program is recorded as
`tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`,
manifest line 165, status `xfail`. **Re-run in this session against the gate
binary as probe `t9`: accepted, exit 0.**

```wf
fn shrink['a](handle: &uniq 'a buffer<u8>) -> discarded: own buffer<u8> reads(handle), writes(handle), allocates(heap) {
  let smaller = buffer_new(2_u64, 0_u8);
  let old = replace deref(handle) = move smaller;
  return move old;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let line = buffer_new(10_u64, 0_u8);
  region 'r {
    let dropped = shrink::<'r>(handle: &uniq 'r line);
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

**Under R1 the program has no shape.** `shrink`'s parameter is a run behind a
`&uniq`, which is the one thing R1 withdraws: a helper that transforms a run takes
it by value and returns it, and a helper that only writes elements takes a
length-fixed view. Neither can change a caller's length behind the caller's back,
because in the first case the caller's length is the *result's* length and in the
second the length cannot change at all. The fifth draft closed D1 by classifying
the kill at a parameter type it still admitted; this draft closes it by not
admitting the parameter.

### 1.3 What the design therefore has to do

Turn every resource a program can exhaust into a value it must hold in order to
consume **and in order to release**, so that "this subtree cannot touch the heap"
is a signature fact and "this program's peak demand is this list of extents and
slot counts" is a compiler judgment. Give the writer one declaration that turns
the second into a compilation requirement. Make every failure to obtain a resource
a typed value that returns the affine inputs it did not consume. Put the runtime
inside the same envelope as the writer's code. Make every fact that survives a
call readable from the callee's declared parameter modes, declared types, and
declared contract, so D1 has no expressible form. Make every value's store
readable from the value's own type, so D1's sibling has none either.

And make each of those a property that is **closed**. Rounds 3, 4 and 5 all made
the same finding one notion further out, and §2.1 is this draft's answer to the
pattern rather than to the three instances: the design names its notions, states
one closure sentence for each, and checks every rule against those sentences.

### 1.4 The minimality ruling, and the partition test

The ruling asks one question of every candidate rule: *could a writer implement
this in wf, given the rest of the kernel?* If yes, it is not spec.

> The kernel specification is the **minimal** set: it admits only what cannot be
> implemented in wf itself. Anything a writer could implement in wf on top of the
> kernel does not enter the spec; it belongs to a standard library — and the owner
> leans toward not having one at all — or to user code. Container capabilities are
> abstracted to the lowest common primitive, and only the truly unimplementable
> part enters the spec. Non-normative content (bound tables, operation
> inventories) never goes in the spec body. Batches are an implementation order
> only, not spec versions; a single implementation is fine if correct.
> Human-factors conveniences are not spec content.

Applying that question needs a criterion for the container half, because
containers are where "could a writer write it" is least obvious. The criterion is
storage. A writer can express **values**: construct them, move them, place them
into fields and elements, match on them, and let them go. A writer cannot express
**storage that holds no value**: a slot outside the initialized set is typed,
addressable and uninitialized, and wf has no spelling that reaches it, no spelling
that declares it, and no way to make the boundary a checker-maintained fact rather
than a killable data field. `array<T, n>` is the shape a writer *can* have, and it
requires `n` live values, which for affine `T` is exactly what the writer does not
have. So the run of initialized slots is the lowest common primitive of every
container this design ever proposed, and it is genuinely unimplementable.

Everything above it is arithmetic over that primitive and is written in 3.L: a
pool is a run of runs, a growable vector is a run plus a growth policy, a keyed
table is a full run of `Option<T>` with element `replace`, middle removal is a take
and an element `replace`, filled and vacant construction are counted loops. The
store half divides the same way: a **store** — a thing that hands out runs and
takes them back — cannot be written, because it manages storage; a **pool** — a
thing that hands out *values* that happen to be runs, and takes them back — is
ordinary data and is written.

Round 5 applied the test in the other direction, which no earlier round had done,
and that is where this draft's largest corrections come from. Two rows the fifth
draft called unimplementable are writable (`seq_exchange`, footnote 6) or are
duplicates of a row it already had (`seq_frame`, 3.K.3). Three things the library
needs are unwritable and were in no register: a const generic as a value, a
relation published per enum variant, and an obligation to hand a value back.
3.L.6 is the list that results, and it is eight rather than seven with two
removed and three added.

One amendment this design **assumes and does not draft** is stated in 3.K.0, because
without it the container half is not writable. [FORM-1] 35 admits one spelling per
semantic construct, so a store's identity cannot be in the type unless the text
determines when the region is written; the owner has ruled that the determination
rule lands first, separately, and uniformly over every region position in the
language. The one property this design needs from it is that the spelling be
decidable by reading the declaration text alone, never by waiting for compiler
feedback, and every program below is written under exactly that.

Two consequences hold once and for all. **The library is not part of the language**:
no rule of 3.K names a library function and a program that never reads 3.L is a
complete program. **And it is not blessed**: whether any of it ships is 5.0's
question.

### 1.5 What this design does not decide: execution contexts

A scheduler that switches contexts, an interrupt handler, and a per-task kernel
stack are **out of scope for this batch, by the orchestrator's ruling**, and this
file states the fact rather than filing it as an open question. No source construct
in v0.41 or in this design creates, enters, or switches an execution context;
`program_kind := "command"` is the whole production (181) and [FN-7] 1216 admits
exactly one entry, so an `interrupt fn` does not parse. Program 4.1 is written
accordingly: it is a cooperative run queue of state machines that advance on one
chain, not a scheduler that switches stacks.

**One thing 1.5 previously left ambiguous is decided here**, because [PROV-5]'s
activation refusal reads it and round 5 found the file answering both ways. **A
worker lane is an execution context**: [RES-1] counts its stack, [STK-3] gives it
an item, and [RUN-4] creates it. What is true of source is narrower than the fifth
draft's sentence: *no source construct creates a context whose chain the program
controls.* A `par`-permitted window may therefore put two activations of one
reserving occurrence in two contexts, which is precisely why [PROV-5]'s refusal
names that source.

```text
| this design fixes                | what a context switch must do with it                    |
|----------------------------------|----------------------------------------------------------|
| E carries one stack item per      | inherited: a new context is a new item of E, measured    |
| execution context [STK-3]         | by [STK-3] over its own whole chain, and creating one    |
|                                   | is an acquisition covered by [RES-1]                     |
| a store's identity is a region in  | **owed, and bounded**: [PROV-1] identifies a store per   |
| its type [PROV-1]                 | *live activation of its region block*, and [PROV-5]      |
|                                   | refuses every placement whose storage cannot be per      |
|                                   | activation wherever more than one activation can reach   |
|                                   | it. A switch multiplies activations, so the successor    |
|                                   | either extends that refusal or gives an extent item a    |
|                                   | per-context identity. The type is untouched either way   |
| envelope accounting is per        | **owed, and now stateable**: a domain is a store         |
| store and peak-based [RES-5]      | [RES-5], and the map carries a retention entry [RES-10], |
|                                   | so a context's steady state is an ordinary entry of its  |
|                                   | own map rather than a shape with no home                 |
| release is structural and         | **owed**: [LIV-1] is a per-join check over one function's|
| explicit [PROV-6]                 | [FN-1] graph. A context that dies is not an edge of any  |
|                                   | such graph, and the successor owes a rule that a context |
|                                   | may not be abandoned holding a linear value              |
| a loan is held by a value for its  | **owed**: [PROV-3]'s exclusivity argument is over one    |
| whole life [L10, VIEW-2]          | thread of control, so a suspension point needs the same  |
|                                   | no-live-loan premise [STK-1] gives a tail edge           |
```

---
## 2. The laws

Seventeen live laws. Every rule in 3.K is an instance of one of them, and **a rule
that cannot name its law is not admitted.** L1 through L9 are the resource laws,
L10 through L15 the container laws, L16 and L17 the two the first falsifier round
added, and L18 is the minimality ruling stated as law. **L14 is retired**, with its
id never reused: it stated that an `AppendView` reaches only what it appended, and
the type it quantified over is gone (footnote 3). Each law states its rationale and
the owner ruling or evidence it rests on in one sentence; ruling ids cite
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
owner ruling R13 (`L7036`), B8, [SCOPE-2] 18, [STOR-6] 738.

**L2. No resource is ambient.** *Every covered resource enters the program as a
capability value the runtime hands to `main`, or as a store the program reserves
in a region block it owns, and travels only by ordinary ownership; there is no
ambient allocator, thread source, or stack pool.*
Because only a held value makes heap-freedom a signature fact: probe `p5_ambient`
allocates while holding nothing and is **accepted today**, and [FN-7] 1248's "there
is no ambient system state" loses its last exception.

**L3. Nothing fails silently, and nothing grows behind the writer.** *Every
operation that can fail to obtain a covered resource returns a typed value naming
the failure and handing back every affine input it did not consume; no operation
traps, aborts, retries, falls back, or promotes a store to a larger one, and no
compiler-derived action does either.*
Because v0.41 claims zero writer-reachable runtime-trap families (spec line 6)
while heap exhaustion still ends a process with no source value: owner ruling R12
(`L5657-5666`), B3, audit answer Q8. The last clause is round 4's, and round 5
showed it was still aspirational: a cyclic containment graph kept the aborting
release walk in every *unmarked* program, so [PROV-6] now refuses that type in
every program rather than denying a premise only a marked entry checks.

**L4. No hidden growth.** *No operation both uses existing capacity and acquires
new capacity; every operation that may acquire capacity takes a provider, names
its allocation effect, and returns a typed failure, while every operation that
only uses existing capacity is total under a proved capacity requirement and can
allocate on no path.*
Because one `push` cannot carry one return type and one effect row across backings:
owner ruling R5 (`L2332`), B2, B3, X1.

**L5. The runtime is inside the envelope.** *The artifact `E` describes is the
writer's code, the compiler-derived cleanup and drop glue, the `par` runtime, the
allocator and the target adapter together, from the frame the environment hands the
program to the frame it takes back; a resource any of them needs is an item of `E`,
or the program is not resource-closed on that target.*
Because a guarantee that stops at the edge of generated code is not one: owner ruling
R12, B12, the ledger read in 6.1.

**L6. Shape, not bytes.** *`E` is a list of tangible resources (contiguous aligned
extents, per-class slot counts, per-context stacks, lane counts, host handles) and
never one byte total. A store the program itself reserves is shaped by the same
rule: a reserving operation that needs an alignment or a separately grantable
extent produces its own `region` item and is not folded into a stack total.*
Because sixteen bytes holding four four-byte objects, the first and third released,
cannot serve an eight-byte request, and a deployment reading one stack number
cannot tell an alignment failure from a size failure: owner ruling R12, B9, B11.
The `handle` shape is round 5's: the runtime holds host objects that are neither
bytes nor interchangeable records, and a profile row that cannot name them makes
`E` incomplete by construction for every hosted marked program.

**L7. Lowering before judgment, and a tail call is a dead caller frame.** *Tail
recursion, including mutual tail recursion, is rewritten into one dispatcher
function before any resource judgment runs; an intra-component call edge is a tail
edge exactly when the caller's activation record is dead at the jump, and never
because the call is written in a return statement.*
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
do. A relation about a value a callee received names that value; no relation
describes a state of a caller's object at a point the callee cannot name.*
This is D1 stated as a law. The second sentence is R1 stated as law, and it is the
sentence the fifth draft's exit datum broke: a clause over a `&uniq` parameter's
post-state is a claim about the caller's object made in the callee's vocabulary,
and F1's first attack is the program that follows. `EVIDENCE-sweep-D1.md`, probes
`t9`, `d1` and `x11`, all accepted today.

**L12. The initialized region is a window, and the language says so.** *A run of
slots is exactly the `len` slots beginning at `head` modulo `cap`, initialized, with
the rest raw; the boundary is checker-maintained typestate carried by the run's own
value, and no per-slot tag, occupancy bitmap, or runtime discriminant is language
state. The kernel admits exactly append and removal at each end; every other order
is arithmetic a writer performs over those four.*
With no per-slot state the checker never needs a quantified proposition over slots,
and occupancy at a stable index is ordinary data. The window is round 5's
correction and it is the largest change in this draft: under the fifth draft's
*prefix*, "every other order is arithmetic a writer performs" was **false for a
queue**, and the price of its being false was a library ring over `Option<T>` that
F4 measured at seven times a hand-written byte ring and that deleted in-place slot
mutation, which K2's DMA descriptor ring needs. A prefix is the `head = Z` case, so
nothing a program writes today changes. Owner's settled decision; audit answers Q2,
Q4, Q10.

**L13. A value's store is a component of its type; acquisition, release and
activation are all closed over it.** *Every store the program can exhaust is named
by one region, minted where the store is reserved or where the runtime hands it in,
and every value that store backs carries that region in its own type. A region
names **at most one live store at any program point**, and a placement whose
storage cannot be per activation is refused wherever more than one activation of
its region block can reach it. A value whose backing is reclaimed per value is
**linear**: `affine` carries ownership and lifetime, `linear` carries reclamation,
and the criterion that separates them is stated once — **a value whose release
action requires a capability is linear, and a value whose release requires nothing
is affine.** A value whose declaration carries the `linear` modifier is linear too,
for a logical obligation rather than a storage one. Linearity is a property of an
affine type and not a third class, it is closed under containment, and a linear
value leaves a scope only by being moved out whole, by being destructured whole, or
by being disposed to the store its type names. A partial consume of a linear value
is refused because it is the one way a leaf leaves by none of the three. No source
construct selects, replaces, or observes a release action, and a store's storage
reclamation never stands in for its content's own release.*
Sentence one is round 3's rank-one repair and has survived every position attacked
since. The capability criterion is R2 and it is what makes the rest derived rather
than declared: with the heap an explicit value [L2], an implicit scope-exit free
would have to smuggle the capability, so a heap-backed run is linear by
construction and no writer marks it, while an arena-backed run and a frame-resident
run need nothing to reclaim them and stay ordinary affine values. The modifier is
reserved for what the criterion cannot see — a value whose silent drop is a
*logical* bug — and round 5 is the evidence that such values exist and are
unwritable without it: 4.1 lost a pool block on an ignored refusal, with no
diagnostic and no envelope movement. B2's drop order, audit answer Q10, [STOR-3]
688, [EFF-2] 1427.

**L14 is retired.** It stated that an `AppendView` reaches only what it appended
and never decreases its owner's length. The type is gone (footnote 3). Under R1 the
guarantee it bought is an ordinary clause relating a result to an input —
`ensures len(rest) >= len(out)` — so nothing replaces it and nothing is lost.
The id is not reused.

**L15. The descriptor's measures are values; the allocator's extent is not.**
*`len(v)`, `cap(v)`, `room(v)` and `head(v)` are a run's own logical measures and
are readable as ordinary `u64` values. No operation observes the physical extent
the allocator provided. Every operation that writes a measured place publishes, for
each measure of that place, its exact new value where that measure is exact and a
two-sided bound where it is not, including the measures it did not change, on every
exit including a refusal. A row never leaves a measure to be reconstructed from the
standing identity.*
The first draft forbade reading `cap` and `room` on a rationale that only forbids
reading the allocator's size, so every pop proved and no push did: B3, audit answer
Q9, probes `q24`, `v25`, `v26`. The exact/bounded split is round 4's, over an
arena's monotone cursor and now also over a window's head. The last sentence is
round 5's rank-three finding: publishing two of three and leaving `room` to the
identity puts every appending loop's backedge outside `AUTO`'s two-premise budget,
so every such loop in the fifth draft was refused. Probes `g3` and `g4` are the
same shape with and without the published relation.

**L16. One measure algebra, and one goal disposition.** *`len`, `cap`, `room` and
`head` are one-place terms of the term language, defined once with their support,
their kills and their standing identities, over every measured place: runs, views,
and providers alike. Every consumer of a numeric goal asks one question, whose
complete admitted derivation is stated once; no rule grants a proof route to a
construct by name.*
A language in which "can this inequality be derived?" depends on which construct is
asking has several provers and a writer can reason about none of them; probes `v25`
and `v26` are the same proof asked twice with opposite verdicts. [ENT-1] 2648.

**L17. Affine liveness agrees at every join, and a linear value never reaches a
scope exit alive.** *A binding's live-or-dead status must be the same on every
predecessor of every join and at every loop head; a disagreement is a hard error at
the join. Consequently **whether** a compiler-derived release runs on a scope-exit
edge is not runtime state and the edge's disposition is unconditional; **which**
release runs inside a value may be, exactly as an enum's derived drop selects on its
discriminant today. A linear binding [L13] that is live on any edge leaving its
scope, a `propagate` error edge included, is the error, because no derived release
exists to carry it.*
The reinitializing `set` makes liveness non-monotone, and [OWN-11] and today's
`Semantics/Unsupported: OwnershipJoin` avoid the question rather than answering it;
the same per-edge check makes linear disposal checkable. Probe `f3`; [ENT-5]'s own
all-predecessor join.

**L18. The kernel admits only what wf cannot express.** *A rule enters the kernel
exactly when no program a writer can write in wf over the remaining kernel has its
effect. A capability a writer can build is not a rule, a convenience is not a rule,
and a table of data is not a rule: the rule is the sentence that says such a table
exists and what it must contain, and the table is generated data beside it.*
The owner's ruling of 2026-09-03, stated as law so that every rule below can be
checked against it and every removal can name it. Its converse is the obligation
3.L discharges, and round 5 showed the test has to run in **both** directions:
`seq_exchange` failed it (footnote 6) and three things the library needs failed it
the other way (3.L.6).

### 2.1 The eight notions and their closures

Rounds 3, 4 and 5 produced one finding each and it was the same finding: a notion
was introduced, used by several rules, and closed by none of them. This subsection
names every notion the design has, states its closure property in one sentence, and
is the checklist every rule below is written against. **A rule that mentions a
notion without respecting its sentence is a defect of this file**, and 3.K.11's
conditions are the mechanical half of the same check.

```text
| notion       | closure property, in one sentence                                          |
|--------------|-----------------------------------------------------------------------------|
| identity     | a value's store is a component of its type, so every value-forming and       |
|              | value-transporting step preserves it, and no rule anywhere admits a store    |
|              | region by outlives rather than by exact identity                            |
| activation   | a region names at most one live store at any program point, and every        |
|              | placement whose storage cannot be per activation is refused wherever more    |
|              | than one activation of its region block can be live at once                 |
| release      | every value has exactly one disposition on every edge leaving its scope —    |
|              | moved, destructured, disposed, or one compiler-derived release — and a       |
|              | store's storage reclamation never stands in for its content's release       |
| accounting   | every covered store is one domain of the map, every edge of the graph        |
|              | carries an entry of that map including the retention entry of an edge that   |
|              | never runs, and every acquisition and every release the program performs is  |
|              | one of the map's primitive transfers                                        |
| linearity    | a value is linear exactly when its release action requires a capability, or  |
|              | when its declaration says so; the predicate is closed under containment and  |
|              | is discharged only by a move, a destructuring, or a disposal of the whole    |
|              | value                                                                       |
| loan-bearing | a loan-bearing value holds, for its whole life, a loan of its own strength   |
|              | on the range it reaches of every place in its resolved origin set, and may   |
|              | occupy no position from which it could outlive or hide that set             |
| measure data | every measure a program can name is a term with descriptor-storage support,  |
|              | published exactly and completely by the row that wrote its place, and killed |
|              | exactly by an event that writes that storage                                |
| elision      | whether a region is written at a position is decided by the declaration      |
|              | text alone: written where it is minted or otherwise underdetermined by that  |
|              | declaration's own operands, elided where they determine it                  |
```

Where each is carried, and which round-5 finding showed it open:

- **identity** — [PROV-1], and preservation is a consequence of type formation
  rather than a clause. Attacked from every position in four rounds and not moved.
- **activation** — [PROV-1]'s invariant, [PROV-5]'s refusal. Round 5 found the
  refusal had lost a disjunct while gaining one (F1 finding 12) and that `seq_frame`
  minted a store no reserving occurrence named (F1 attack 5, F2 F5-1, F3 defect 13).
  [PROV-5] now states the property and names three sources; `seq_frame` is deleted.
- **release** — [PROV-6], [LIV-1], [STOR-3]'s table. Round 5 found `dispose` had no
  condition on its operand and contributed no write of what it destroyed, so a
  callee freed a caller's run through a shared borrow that [CALL-1] guarantees kills
  nothing (F1 attack 2), and no closure over a sub-place (F1 attack 3). `dispose` is
  now a **consume and a write**, which closes both with rules the language has.
- **accounting** — [RES-5], [RES-8], [RES-10] and 3.K.7.1. This is round 5's open
  notion and all eight of F2's BREAKS are in it. A domain is now a **store**, the map
  carries a **retention** entry and a **reset** transfer, and every column and flag
  is derived from data a signature or a contract already carries.
- **linearity** — [PROV-6]. Round 5 showed the predicate could not express "hand it
  back", so 4.1 leaked a pool block with no diagnostic and no envelope movement (F2
  F5-8, F4 §3.3). R2 closes it in two halves: the storage half is *derived* from
  the capability criterion, so a store-derived type is never marked, and the
  logical half is the modifier, whose whole writer test is **would silently
  dropping this value be a bug?** 3.L.7 is the guideline.
- **loan-bearing** — [PROV-3], [BLK-4], [VIEW-4]. Round 5 found `replace` through a
  `&uniq MutSpan` refused by no rule (F3 defect 4) and a carried formation datum
  with no reader (F3 I26). [VIEW-4] now states the commit rule; the datum is deleted.
- **measure data** — [MSR-1] to [MSR-3], [BLK-0]. Round 5 found rows publishing two
  of three measures and leaving the third to a standing identity `AUTO` cannot
  combine (F1 attack 4). [BLK-0] now requires every measure on every exit.
- **elision** — 3.K.0, [PROV-1]. Round 5 found the criterion backwards at the
  store-minting position and empty at a parameter position in a heap-free program
  (F1 attack 6, F3 defect 2), and a linearity obligation that changed per
  instantiation (F2 F5-13). 3.K.0 restates the criterion as *derivation*; [PROV-6]
  makes an elided brand linear at the declaration, so one declaration has one
  verdict.

---

#### 3.K.0 The region-spelling assumption

This design assumes one amendment it does not draft. **Whether a region is written
at a given position is determined by the program text, and the determined spelling
is the only legal one.** That is a change to [FORM-2], [GRAM-2] to [GRAM-5], [FN-2]
and the [OWN] borrow forms, it is uniform over every region position in the
language — parameter lists, borrow annotations, region arguments on types,
call-site region arguments, and region blocks — and **it lands first, as its own
separate and mechanical spec amendment**. It is not a rule of this design, it is not
in 3.K's count, and 3.K.11 does not register it.

It is stated here because the container half cannot be written without assuming it.
[FORM-1] 35 admits exactly one spelling per semantic construct. Putting a store's
identity in the type means a region in every type that names a store, unless the
text determines it — in which case *writing* it is a second spelling and the law
says there is only one. So the brand cannot be in the type without that amendment,
and the amendment cannot be brand-specific, because a brand is one more region
argument.

**The criterion is derivation, not repetition.** The fifth draft wrote it as
"written exactly where it relates two positions of one declaration", and round 5
showed that criterion getting both of its flagship cases backwards: it elides
`arena_frame`'s `'s`, which is the sole mint of store identity and which [PROV-5]
requires written, and it writes `seq_heap`'s and `seq_span`'s, which both worked
programs omit. The criterion this design needs, and the only one consistent with
[BLK-0]'s written-argument rule, is:

> A region, type or const argument is **written** at a position exactly when the
> declaration's own operands do not determine it, and **elided** exactly when they
> do. Written and elided are decided per argument, not per list.

Applied to every spelling in this file, the criterion and the text now agree:

```text
| occurrence                                                   | determined by an operand?      | spelling            |
|--------------------------------------------------------------|--------------------------------|---------------------|
| arena_frame<const bytes, const align>['s]()                  | no operands exist              | all three written   |
| seq_fixed<T, const n>()                                      | no operands exist              | both written        |
| seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)        | 's from heap; T from nothing   | seq_heap::<u8>(...) |
| seq_arena<T>['s](arena: &uniq Arena<'s, ...>, count)         | 's from arena; T from nothing  | seq_arena::<u8>(...)|
| seq_span['r, T](vector: &'r v)                               | both from the borrow operand   | seq_span(...)       |
| a user fn's own region parameter list                        | supplied by the actuals        | elided at the call  |
| struct BlockPool['s] { free: FixedVector<Lease<'s>, 8>; }    | a declaration mints its own    | 's written at both  |
```

The last row is the general shape of a **declaration**: a nominal's or a function's
own region parameter list is where a region name is bound, and a bound name is
written at its binder and at every position of that declaration whose type names
it. Nothing outside the declaration is consulted, which is the property this design
needs.

**Two positions, two candidate sets, and neither is ever empty.** Round 5 found the
fifth draft's candidate set empty at a parameter position in a heap-free program,
which made 4.1's central statement undeclarable. The repair is to say which
position is which:

- At a **stored** position — a field, an enum payload, a run element, a written
  type argument — an elided brand denotes the enclosing nominal's sole region
  parameter when it has exactly one, and otherwise the entry heap's store region.
  When the nominal declares a region parameter it is written, so the elided form
  arises only in a program whose values all come from the entry heap. When the
  nominal has no region parameter and the entry selects no heap, the position is a
  [BLK-4] hard error naming the nominal, not an empty resolution.
- At a **parameter or result** position an elided brand is always an **implicit
  region parameter**, one per occurrence, and never the entry heap. A signature
  that must name the entry heap writes it, and the entry heap therefore has a
  spelling: the entry's own declared store-region parameter, in scope for the whole
  program. That is what makes a helper declarable in a program with no heap at all:
  4.1's `render['s]` and `drain['s]` bind their own region name and never mention
  the entry heap, where under the fifth draft's candidate set they had no admissible
  brand and did not typecheck.

**And an elided brand is linear at the declaration** [PROV-6]. Round 5's F2 F5-13
showed that a function generic over its store has two verdicts — its parameter is
linear at a `Heap` instantiation and affine at an `Arena` one — chosen by a caller
who writes nothing that distinguishes them. Under R1 the repair is exact and costs
nothing: a helper that is generic over the store is value-in, value-out, so it
releases nothing, so its obligation is the same at every instantiation. Stating it
as a rule rather than as a consequence is what keeps it true when a later helper
tries to release: *a value whose store region is an implicit parameter is linear at
the declaration and must leave by a result*. A helper that must release names its
store and its provider, which makes the region written.

**How the brand is therefore spelled**, which is what section 4 and 3.L are written
in:

- A **heap-derived** type in a stored position carries no visible brand. The entry
  heap is unique and has program lifetime, so `Vector<u8>`, `Bytes` and `Heap` name
  it. 4.2 declares no region parameter anywhere.
- An **arena-derived** type writes the brand at the declaration that binds it and
  at every position of that declaration whose type names it:

  ```wf-design
  linear struct Lease['s] {
    run: Vector<'s, u8>;
  }

  fn pool_take['s](pool: own BlockPool<'s>) -> (rest: own BlockPool<'s>, leased: own Option<Lease<'s>>)
  ```

  The modifier is [S18], the region parameter list on a nominal is [S20], and the run
  is [S1]. `'s` relates the nominal's parameter to its field, and the function's
  parameter to its results, so it is written at every one of those positions and
  supplied by the actual at the call.
- A helper that **hands a run back** relates its parameter's store to its result's,
  so it binds one region name and writes it at its binder and at both positions:
  `fn collect['s](out: own Vector<'s, u8>, source: own Span<u8>) -> (rest: own
  Vector<'s, u8>, written: own u64)`. That is one identifier per helper, written
  once; the **call site** elides it, because the `out` operand determines it, and
  writes `collect(out: move buf, source: move line)`. This is R1's whole spelling
  cost and 4.1 pays it twice.
- A helper whose brand relates nothing writes none and is generic over the store it
  is handed — and by the sentence below it may therefore not release what it was
  given, which is what keeps one declaration to one verdict.

Measured on this worktree, `tests/programs` is 28 files and 131 top-level function
declarations, of which **67 carry a region parameter list**, and across all 67 **no
region name is used at two positions outside its own parameter list**. The corpus
also writes 484 named borrow annotations, 251 call-site region arguments and 232
region-block names. Under the amendment essentially all of them become [FORM-1]
rejections and disappear, no program's meaning changes, and one mechanical migration
pass over the corpus, the conformance cases and the snapshot cases converts them —
by a tool that ships nowhere, because [FORM-1] 36 says the toolchain never
auto-formats. Two costs this design would otherwise carry go with them: [VIEW-7]'s
two writer-visible regions per I/O site, and the forty-four reborrow regions
[PROV-7] was written to buy back.

## 3. The rules

Section 3 is two sections, read differently. **3.K is the kernel**: nine families,
**fifty rules**, six nominals, twelve declaration-domain operations plus four
readers, two added statement forms, one
added declaration modifier, and 3.K.0's one assumed amendment. Every rule answers
L18's question with *no writer can write this in wf*, and 3.L.6 lists the eight that
only the partition test proved. **3.L is the library**, written in wf against 3.K;
it is not part of the language, it is not blessed, and no rule of 3.K names any of
it.

The count moved from forty-eight by exactly two, and both are content the fifth
draft already had without a rule id. [MSR-6] is the const-generic admission round 5
found five library functions depending on and no register carrying. [RES-10] is the
composition algebra, which was normative content in an unnumbered subsection and
which round 5 broke in eight places; giving it an id puts it under the same
`Judgment`/`Publishes`/`Amends`/`Law` discipline and the same §2.1 check as every
other rule, which is how the eight would have been caught.

**Every kernel rule states four things — the judgment it creates, the fact it
publishes, what it amends, and its law — plus a `Depends:` line exactly when it
rests on a v0.41 sentence no `Amends:` line in this file changes.** A rule that
creates no judgment writes `*Judgment:* none` and says what it is instead.
`*History:*` points at the round in 6.5-6.9 that produced the rule's current shape
and carries nothing else. Section 3.K.11 is a **collation of the `Amends:` and
`Depends:` lines and carries nothing else**: it is written last, from the rules.

### 3.K Kernel rules

#### 3.K.1 `[MSR]`: measures, terms, and the one goal disposition

This family is first because everything else consumes it. It adds no statement
form and no type; it is a specification amendment.

**[MSR-1] Four measure terms, over one place, for every measured value.**
`len(P)`, `cap(P)`, `room(P)` and `head(P)` **[S11]** are terms of the [ENT-2] term language,
of fragment type `u64`, where `P` is an admitted place. They are defined once, here,
for every *measured* type, and which measures a type has, and whether each is
**exact** or **bounded**, is table data rather than a rule with an exception. The
table is Appendix A.1; the rule is that it exists, that it gives every measured type
a row, and that every row's cell is one of *exact*, *bounded*, or *absent*.

An **exact** measure is one for which every writing row publishes a value; a
**bounded** measure is one for which some writing row can publish only a two-sided
range, because no exact value exists in the source domain. Exactly two measures are
bounded anywhere: an `Arena`'s `len`, whose alignment padding is a target-stage
quantity, and a run's `head` after a front operation, whose new value is a modular
expression the affine domain does not carry. Both are stated once in A.1 and
nowhere else.

An admitted place for a measure term is a `place` [GRAM-5] formed with field
selections, `deref` wrappings **and subscripts**, whose final selected type is a
measured type. The subscript admission is the change: `len(table[i])` is a term,
so a run of runs has provable operations.

*Judgment:* the [OP-4] admission above at every subscripted measure place.
*Publishes:* the terms. *Amends:* [ENT-2] clause (b) (2681), which today admits
`len(P)` only for `array`, `slice` and `buffer`, and only for subscript-free
places; [OP-4] 914, whose obligation gains the erased-clause attach-site case.
*Law:* L15, L16. *History:* 6.9, F1 attack 4 and the window.

**[MSR-2] Support is descriptor storage, a kill is an ordinary [ENT-5] event, and
a standing fact has empty support.** A measured value's storage is two disjoint
parts, exactly as [STOR-1] and L12 already describe the object: its **descriptor
storage**, the measure words its value carries, and its **element storage**. The
support of a measure term over `P` is:

- `P`'s descriptor storage;
- every borrow or content holder any prefix of `P` reads through; and
- the support of **every** offset occurring anywhere in `P`, not only the last.

The kill is then [ENT-5]'s own rule with no new overlap notion: a measure term
dies exactly on an [ENT-5] event whose written place overlaps its support, where an
event is any [SET-1] commit, [SET-2] commit, consume, scope exit, or **any action
carrying a `writes` occurrence that projects onto that storage under [EFF-2]**, a
call, a `dispose` [PROV-6] and a compiler-derived release alike. Stating the kill
over the effect row rather than over a list of syntactic forms is what keeps it
closed when a later family derives a new action — and it is why `dispose` had to
become a write, since round 5's second attack carried a release **past** a kill
stated over writes.

Four consequences follow and none is an exception clause. An **element write** does
not kill, because element storage is not descriptor storage (probe `w4`). A write to
a **sibling field** does not kill, because `deref(r).flags`'s descriptor storage and
`deref(r).tail` do not overlap — probes `r2_4`, `r2_4b` and `r2_4c` show today's
compiler is root-granular where [EFF-2] on the same statement is field-precise, and
this rule uses the precision [EFF-2] already computes. A write to an **offset** does
kill, at every level, so a fact over `len(grid[i][j])` dies when `i` is written. And
an element-position **replace of a descriptor** kills that descriptor's measures
while one of a scalar kills nothing: `set grid[i][j] = x;` writes element storage of
`grid[i]`, `replace grid[i] = w;` writes its descriptor storage.

The fourth consequence is why [ENT-5] 2893 clause (a)'s parenthetical carve-out —
"*while an element-position replace, like an element write, kills none*" — is
**removed rather than narrowed**. That sentence is true in v0.41 for a reason
[MSR-1] deletes: `len(P)` is defined only for `array`, `slice` and `buffer` and
admits no subscript, so an element position can never hold a descriptor. Once it
can, the carve-out is a second statement of the granularity, in the wrong place and
now false. The kill becomes the plain overlap test and the four consequences are
derived from it.

At every program point at which `P` is live, these hold implicitly:

```text
Z <= len(P)     Z <= room(P)     Z <= head(P)     len(P) <= cap(P)     head(P) <= cap(P)
```

and the three-term identity `len(P) + room(P) = cap(P)` is appended, as the two
inequalities `len(P) + room(P) - cap(P) <= 0` and `cap(P) - len(P) - room(P) <= 0`,
to [ENT-6] 3007's automatic affine-premise sequence, with the empty support every
standing fact has. That is the shape [ENT-6]'s premises already take, it is usable
by `AUTO`'s families unchanged, and it keeps the identity out of L0, whose
uniqueness argument [ENT-4] 2860 rests on the difference-bound shape. **The identity
is a convenience for the writer and never a route by which a row's own post-state is
derived**; [BLK-0] requires every row to publish every measure it writes, which is
what puts every backedge inside `AUTO`'s one-premise family.

**A measure whose value is a compile-time constant or a runtime-profile symbol is
a standing fact with empty support.** A formation row that publishes `cap(result) =
n` for a written const `n`, and a runtime store whose `cap` is a profile row,
therefore both give a capacity that no event kills for the life of the term. This
is where the fourth draft put a type-level constant and it generalizes to the one
store whose capacity cannot go in a type: [RES-9] already asserts the sentence for
a profile symbol and [MSR-2] is where the kill lives, so it belongs here.

*Judgment:* none. *Publishes:* the implicit facts, the two automatic premises, and
the standing-fact class. *Amends:* [ENT-2]'s implicit-fact sentence (2728);
[ENT-5]'s support and kill sentences (2863-2896), whose length-term support becomes
the descriptor-storage relation above, whose kill classes (a) through (d) gain the
effect-row statement, and whose clause (a) loses its element-position carve-out;
and [ENT-6] 3007's automatic affine-premise sequence, which gains two
specification-fixed members. *Depends:* [ENT-4] 2860, whose difference-bound
uniqueness argument is why the identity is a premise and not an L0 fact; [ENT-5]
2942-2946, whose "no fact established inside an iteration survives to the next
iteration's head" is what keeps an empty-support fact from crossing a backedge.
*Law:* L15, L16. *History:* 6.8, F1 attack 3 and F2 NB3; 6.9, F1 attack 2.

**[MSR-3] Measure datums, and what an atom is keyed by.** A **measure datum** is a
compiler-owned immutable [ENT-2] term of fragment type `u64` with **empty support**:
no [ENT-5] event kills it, no place occurs in it, and no later write retargets it.
It is the device [ENT-2] already has for a `for_stmt` capture and a [SET-1] commit
value, extended to one more producer. There is exactly one former, keyed on what a
datum denotes rather than on where the value came from:

```text
a datum is identified by (program point, admitted place P, measure), is
compiler-owned and immutable, and is established equal to <measure>(P) at that
point
```

**Three placements exist, and no fourth:**

```text
entry placement       body entry, for each parameter of measured type and each
                        measure it has; the datum denotes that parameter's measure
                        at entry
call placement        one call's pre-transfer point [ENT-5], for each operand
                        place of measured type and each measure it has, reading a
                        borrow operand through its resolved referent and an own
                        operand as its value before transfer
construct placement   one `construct` [GRAM-8] or enum-payload construction, for
                        each field or payload operand of measured type and each
                        measure it has, read as that operand's value before
                        transfer
```

The borrow half of the call placement is the split [FN-8] 1275 already makes for a
goal actual, applied to the datum former.

**The fifth draft's fourth placement is withdrawn.** It minted an *exit* datum at a
call's post-kill point for each `&uniq` operand, so that a callee could publish what
it had done to a borrowed run. Round 5's first attack is the program that follows:
the exit datum is a caller-side object with no callee-side placement, so the callee
discharged its `ensures` against the only datum it had — the entry datum — and the
caller established it as the exit fact. That is D1 with the run two elements long
and `len >= 10` in the caller's state. R1 removes the shape rather than adding a
fifth placement: a helper that transforms a run takes it **by value** and returns
it, so what the caller reads is a relation on a *result*, which [CALL-2] already
transports and which no callee can be wrong about. The remaining three placements
share one property that the exit placement did not have and that is now the closure
sentence of this rule: **every placement is a point at which the function forming
the datum can itself read the value.**

Three rules read datums and nothing else does. A [FN-9] or [FN-8] clause operand
naming a parameter's measure denotes that parameter's **entry datum**, in a
`requires` and in an `ensures` alike, so a consuming use of an `own` parameter
cannot invalidate it and no clause needs to say which state it means: a parameter is
an input and has one. A [BLK-0] declared relation naming an operand's measure
denotes that call's **call datum**, so it survives the argument consume that the
same statement performs. A [CALL-4] clause naming a result's measure denotes the
result itself.

**One sentence fixes what an [INV-1] affine atom over a measured place is keyed
by, and it covers every writing form.**

> An [INV-1] affine atom over a measured place is keyed by the [ENT-2] term. A
> **reinitializing `set`** [LIV-2] is a declaration event: the old term is retired,
> a new one is introduced, and a header invariant over the new term is
> re-established on the backedge from the operation's declared relation over its
> call datum, which has empty support. An **in-place exchange** [LIV-3] is not a
> declaration event: the root's term survives, the facts over it die by [MSR-2],
> and the same declared relation re-establishes them on the same term. **Each
> later target of a multi-target `set` [LIV-3] is a declaration event**, exactly as
> its `let` counterpart is. A form that is none of the three, and that rewrites a
> measured place a header invariant names, is a diagnostic and not a silence; so is
> a reinitializing `set` of a binding a live header invariant names, whose atom the
> statement retires and whose invariant would otherwise be silently orphaned.

*Judgment:* the orphaned-invariant diagnostic above; a datum is formed, never
proved. *Publishes:* the datum and the atom-identity rule. *Amends:* [ENT-2]'s term
list (a new clause beside its capture and commit-value clauses); [ENT-5]'s
call-boundary paragraph (2898-2905) and its FN-9 entry-image-stability paragraph
(2887-2891), which are replaced by the datum rather than repaired; [FN-9]'s
`M(c,q)` (1345, a datum operand is always live) and its parameter-entry-image
sentences (1316); [ENT-6]'s image formation, join and loop-header paragraphs
(2976-3002); [ENT-3.S5] 2774-2782's copy-equality clause, which gains the construct
placement's measured fields; and [INV-1] 3109-3113's atom resolution, which gains
the sentence above. *Depends:* [ENT-2] 2693, whose one-static-term-per-statement
argument is why a per-point datum is sound; [ENT-5] 2942-2946, whose head-state
construction is why a body-placed datum does not cross a backedge; [FN-8] 1275,
whose borrow-versus-own actual split the call placement reuses. *Law:* L11, L16.
*History:* 6.9, F1 attack 1 and attack 10; 6.8, F4 blocking 2.

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
[BLK-0]. **The per-family route lists retire.**

**This rule is not widened, and [BLK-0] is where round 5's arithmetic finding is
repaired.** F1's fourth attack showed the design's own `spare` backedge needing
three published affine premises where step 5 admits two. Widening `AUTO` to three
would change the prover's complexity class and destroy [ENT-6] 3026's promise that
"an author can determine from this rule alone whether a target is automatic", which
is worth more than the convenience. The defect was in what the rows publish, not in
what the prover accepts, and probes `g3` and `g4` locate it exactly: the identical
three-term header invariant is provable when the body publishes one exact relation
and is not when it publishes none.

*Judgment:* the disposition itself. *Publishes:* the disposition of every numeric
goal. *Amends:* [ENT-6] 3040, 3047, 3075 and 3084, the four per-family route and
attach-site grants, which keep their normalization and lose their route grant, and
[FN-9]'s `prove_ordering` route, whose undocumented direct-affine branch becomes
one of the six steps. *Note:* this rule is why the design does not have to be
revisited when the library adds an operation: an operation adds a goal, never a
route. *Law:* L16. *History:* 6.9, F1 attack 4; 6.5, F4-3.

**[MSR-5] The contract clause is the relation an invariant already is, over a wider
operand set.** **[S17]** A `requires`, `ensures`, `header_invariant`,
`invariant_stmt` or `proof_use` operand is a **term** of the [ENT-2] term language,
not an `atom` of [GRAM-5].

**v0.41 does half of this rule's work and it is smaller for it.** A contract clause's
root is already one `compare_op` over two `expr`s ([FN-9] 1312), and a
`header_invariant` is already `affine_expr compare_op affine_expr` ([GRAM-4] 239).
What is left is the **operand set**: [GRAM-5] 269's `atom` has no `call`
alternative, so `len(source) <= room(out)` derives nowhere, and probe `t5` is that
rejection at [GRAM-5] with the compiler's own mechanical fix naming `define`. The
amendment goes where the refusal is, and it makes a clause exactly the shape an
invariant already has:

```text
clause_expr    := affine_expr compare_op affine_expr
affine_factor  := literal | ent2_place | measure_term | "(" affine_expr ")"
```

`requires_clause` and `ensures_clause` (spec 185-186, [GRAM-2]) take a `clause_expr`
instead of an `expr`; `ent2_place` is [ENT-2] 2681(a)'s place grammar and
`measure_term` is [MSR-1]'s four formers over one admitted place. The `compare_op`
is the grammar's own token and not a new terminal, and it carries [INV-1] 3105's own
sentence in [FN-9]'s wider form: *the operator selects a proof-domain relation and
performs no [OP-1] operation, so a `clause_expr` is not an expression and carries no
[OP-5] type judgment*. Where [INV-1] 3105 admits the four ordered symbols alone, a
contract clause admits all six, exactly as [FN-9] 1312 already does.

`ent2_place` admits an IDENT and its projections; it does **not** admit `Z`, which
is [ENT-2] clause (g)'s compiler-owned zero and has no source spelling. Probe `t11`
is that rejection at [GRAM-5], and 3.L writes `0_u64`.

*Judgment:* the ordinary [FN-8]/[FN-9]/[INV-1] admission over the widened operand
set. *Publishes:* nothing new. *Amends:* [GRAM-5] 269-270 (a new `clause_expr`
production; `atom` and `atom_list` unchanged), [GRAM-2] 185-186's `requires_clause`
and `ensures_clause`, [GRAM-4]'s `affine_factor` production, [OP-5] 926's
contract-predicate scope, [FN-8] 1262-1267, [FN-9]'s operand list (1312-1314), and
[INV-1] 3109-3113's atom sentence; [GRAM-9] is unchanged. *Depends:* [INV-1] 3105,
whose relation restriction and no-operation sentence this production reuses
verbatim. *Verified today:* probes `t5`, `t11`, `w3`, `x5`, `q1`, `q9`, `r1_lenatom`
and `r1_field` are parse rejections, so this is an amendment and not a compiler
defect. *Law:* L16. *History:* 6.9, F3 defect 8 and the v0.41 respelling; 6.8, F3
defect 5.

**[MSR-6] A const generic is a value wherever a named const is.** **[S21]** [TYPE-6] 401's
`pbase` admission — "an in-scope runtime value binding, contract definition,
admitted symbolic result datum, or named const" — gains **an in-scope const
generic**, and [ENT-2]'s `for_stmt` endpoint admission and [MSR-5]'s
`clause_operand` gain it with it. It is a monomorphization-time constant, so
[ENT-2] clause (c) already makes it a symbolic constant *term*; this rule is the
one sentence that lets a program **name** it.

Round 5 found the omission by writing the library rather than by reading the rules,
and it is the clearest single result the partition test produced in either
direction. Probes `t1`, `t2` and `t3` are the three positions, all
`[TYPE-5] UnresolvedUse { spelling: "n", role: PlaceBase, admissible: [NamedConst,
Value], available: [ConstGeneric] }`; probe `t4` is the same program with a named
const in all three positions and is **accepted**. Every capacity-parametric function
in 3.L and `CONTAINERS.md` reads its bound in one of them, including the two 3.L.3
writes out in full, so without this rule the library is not merely inconvenient but
unparseable — and 3.L.6 records it as one of the eight.

Nothing about term identity, support or kills changes: a const generic's term has
empty support and no event kills it, exactly as a named const's does, so the
diagnostic domain gains one member and the proof domain gains nothing.

*Judgment:* the ordinary [TYPE-6] resolution over the widened admission, and the
ordinary [TYPE-5] type check at each use. *Publishes:* the const generic as a
`pbase`. *Amends:* [TYPE-6] 401 (`pbase`'s admitted declaration classes), [ENT-2]
2685-2687's endpoint admission, and [MSR-5]'s `clause_operand` through
`ent2_place`. *Depends:* [ENT-2] 2681 clause (c), which already makes an
integer-typed const generic a symbolic constant term, and which is why this rule
adds a spelling and not a fact source. *Verified today:* `t1`, `t2` and `t3`
rejected, `t4` accepted. *Law:* L16, L18. *History:* 6.9, F4 blocking 1.

#### 3.K.2 `[PROV]`: stores, brand, activation, and release

**[PROV-1] A store's identity is a region, the region is in the type, and a region
names at most one live store at any program point.** This is the rule the design is
built around, and everything else in this family is derived from it.

A **store region** is a region that names one store. A region becomes one by being
named as the store argument of a reserving occurrence [PROV-5], or, for the heap,
by being the entry's own store-region parameter. **There is no third way**, which
is now a checkable sentence rather than an assumption: the fifth draft's
`seq_frame` produced a `Vector<'s, T>` whose `'s` no reserving occurrence named, so
this rule's invariant had nothing to quantify over, [PROV-6]'s predicate classified
it by no clause, and [RES-1] and Appendix A.2 gave it two different envelope items.
The row is deleted (3.K.3) and the sentence is true again.

A region may be named by **at most one** reserving occurrence; a second occurrence
naming a region already used is a hard error citing PROV-1 at that occurrence's
`targ`, with the restructuring `open one region per store`. [OWN-3] 578 makes
region identifiers unique within a function, and probe `w1` confirms the compiler
enforces it.

Every value a store backs carries that store's region in its own type. There are
two stores and one run shape over each, and the table is the whole vocabulary:

```text
| store       | provider [S3, S4]        | one run [S1, S2]     | release needs         | class   |
|-------------|--------------------------|----------------------|-----------------------|---------|
| general     | Heap<'s>                 | Vector<'s, T>        | the Heap capability   | linear  |
| bump extent | Arena<'s, bytes, align>  | Vector<'s, T>        | nothing; 's resets it | affine  |
| (none)      | (none)                   | FixedVector<T, n>    | nothing; the frame    | affine  |
```

The last column is not a fourth fact: it is [PROV-6]'s criterion read off the third,
and it is the whole of what R2 makes derived rather than declared.

`FixedVector<T, n>` has no store region because it has no store: its run is inline
in its owner or the stack frame, nothing is ever released to it, and its
confinement is ordinary [OWN-1] liveness. Its capacity is in its type because a
frame-resident run must have a size before layout runs; a store-resident run's
capacity is a measure fixed at the take, because a growth policy that could not
change it would not be a growth policy. Those are the only two differences and
each has one reason.

**Preservation is a closure property and needs no clause of its own.** A value's
store is a component of its type; no value-forming step changes a value's type;
therefore none changes a value's store. That covers `construct`, field projection,
element placement and removal, enum payload construction and `match` binding,
multi-return, a join, a value-in / value-out row's result, an argument transfer and a
return, in one sentence, and every future step for the same reason. Two values have
the same store exactly when their types name the same region, which [OWN-12] 650 and
[TYPE-5] 379 decide by exact identity. All five falsifier rounds attacked this from
every position they could build and none moved it; 6.8 and 6.9 record the routes.

**The brand's spelling is 3.K.0's assumption, and this rule adds nothing to it.**
What this rule owes that amendment is the **candidate set at each position**, which
3.K.0 states, and the sentence that makes it non-empty by construction: at a stored
position the set is the enclosing nominal's own region parameters plus the entry
heap's store region when the entry selects one, and at a parameter or result
position an elided brand is an implicit region parameter. That is read from the
nominal's own declaration and from the entry's one input row, never from a callee,
a caller, or an instantiation, which is what makes the spelling decidable from the
declaration text alone.

**The provider parameter itself is never elided.** `heap: &uniq Heap` keeps its
parameter, its mode and its effect row, because that is the signature-visible
allocation fact L2 exists to create; what goes is the region *inside* the type and
the region of the borrow, never the parameter. A signature that allocates still says
so at its parameter list and at its `allocates` row, and [PROV-4]'s reachability
closure reads exactly those. `struct Bytes { v: Vector<u8>; }` is then an ordinary
nominal with no region parameter, and 3.L.5 writes `byte_string.wf`'s join with and
without the brand so the difference is visible rather than argued.

`Heap<'s>` is delivered as an `own` entry parameter and lives for the program.
The `command` standard-input table [FN-7] gains ordinal 5:

```text
| ordinal | label             | written mode and type | supplied value                                 |
|---------|-------------------|-----------------------|------------------------------------------------|
| 5       | command.heap [S22]| own Heap              | the one general store the runtime minted first |
```

and the entry may declare **exactly one region parameter**, admitted only when it
selects that row or reserves an arena; program start supplies it and it outlives
every region of the program. Under 3.K.0 the entry writes it only when a signature
must name the heap or when it also reserves an arena. The `Heap` `main` receives is
dropped on the return edge with the **empty** release row: the store is the
runtime's, the program returns the handle, and no covered acquisition or release
happens there.

*Judgment:* one live store per store region, established by [PROV-5]; provider and
branded types are nominal and closed, and no source declaration introduces another;
plus the ordinary [FN-7] label, order, mode and type checks. A second reserving
occurrence naming one region is the hard error stated above, `SecondStoreInOneRegion`.
*Publishes:* each value's store, as a component of its type; the store's measures;
and the whole-program fact `heap-unreachable` when the entry row is absent.
*Amends:* [TYPE-2] 357, which gains the five branded and container nominals below
and from which `box<T>`, `arena<'r, T>` and `buffer<T>` retire from the writer
surface; [TYPE-7] 476, whose closed deref domain becomes `&'r T` and `&uniq 'r T`
alone, because a single stored value is a run of capacity one and is reached by
subscript; [GRAM-3] 207-210, whose fixed `box`, `arena`, `slice` and `buffer` type
productions retire in favour of ordinary TYPEIDs with `targs`, and which gains the
omitted-store-region form; [OWN-10] 641-645, whose `arena<'r, T>` content clause
becomes a clause over `Vector<'s, T>` content with `'s` in the subject position;
[FN-7]'s table (1227-1233), its "declares no region parameters" sentence (1218), its
canonical five-input byte sequence (1245-1246), and its effect-row sentence (1220),
whose `allocates(heap)` becomes `allocates` over the entry's own labelled provider
input. *Depends:* [OWN-3] 578 and 580, for uniqueness within a function and
incomparability across the boundary; [OWN-12] 650 and [TYPE-5] 379, for exact region
identity in type equality, which is the whole of the invariance argument. *Law:* L2,
L13, L16. *History:* 6.9, F1 attack 5 and F3 defect 13; 6.8, F1 attack 1.

**[PROV-2] Unforgeable, uncopyable, taken as a loan, and never stored.** No source
construct produces a provider; a `Heap<'s>` exists only because the runtime minted
exactly one before `main`, and an `Arena` only as the result of a reserving
operation [PROV-5]. No operation duplicates, reconstructs, compares, serializes, or
derives a provider from a non-provider value.

An operation that allocates from a store, or releases to it, takes that store's
provider as a written `&uniq 'b` parameter and exhibits it. A provider is never
passed `own`: it is confined to its own store region, and a moved provider strands
its own store. The one `own` provider in the language is the `Heap` the entry
receives.

**A provider parameter is the one borrow R1 does not withdraw**, and the reason is
that it is not a container: no operation changes a provider's *identity*, only its
measures, so a caller keeping a fact about a provider across a call is keeping a
fact the callee's own declared relations publish. R1's subject is a value whose
length a callee could change out from under a caller; a provider has no length a
caller reasons about except through the row that changed it.

*Judgment:* a `construct` [GRAM-8] naming a provider or container nominal, and
every other source route to one, is a hard error citing PROV-2 at the complete
`construct`, with the restructuring `receive the provider as a parameter, or
reserve one with arena_frame`; a provider type in a stored position is a hard error
citing PROV-2 at the complete contained `type`, with the restructuring `lend the
provider to the operation that needs it; a provider is never stored`; and an
allocation or release call whose provider argument is missing, is not a provider
place, or is not writable is a hard error citing PROV-2 at the `call`. *Publishes:*
uniqueness of the `Heap`; and the store's post-state measures, which are [BLK-0]
declared relations over the call's own datums [MSR-3], stated single-state.
*Amends:* [OP-1] 798-803, from which `box_new` and `arena_new` retire, and [STOR-2]
685, which defined them; [STOR-5] 723-737, whose enumerated stored-content
positions gain the provider prohibition. *Depends:* [OWN-10] 641, which is why `'s`
and `'b` are always distinct; [OWN-6] 614, which makes an argument borrow a
call-scoped temporary, the fact probe `w8` exercises and the reason store identity
may not rest on what stands at a place between two calls. *Law:* L2, L3, L4, L13,
L16. *History:* 6.8, F1 attack 7.

**[PROV-3] Provenance is for loans, and a loan reaches a range.** [OWN-5]'s finite
origin set, today defined for `slice<'r, T>`, generalizes to the two views and to
nothing else. A **loan-bearing** type is `Span<'r,T>` or `MutSpan<'r,T>`; a value of
one carries a finite set of origins, each an origin place paired with the half-open
index range the value reaches of it.

**The fifth draft's carried formation datum is deleted.** It existed for
`AppendView`'s commit event, `AppendView` is gone, and round 5 found the machinery
surviving its only consumer — which is the same defect L18 forbids one level down.
A view's measures come from the ordinary declared relations of its formation row
over that call's own datum [BLK-0], like every other row's.

Formation makes a **singleton**: `seq_mut_span(vector: &uniq table[i])` has the
singleton origin `table[i]` with range `[Z, len(table[i]))`. A named const maps to
the distinguished `immutable-const` origin. Binding, moving, passing and returning
preserve the set and its ranges; a control-flow join takes the union; a parameter of
loan-bearing type starts with the singleton containing its own formal origin,
substituted at a call boundary by exactly the rule [FN-1] 1041-1047 already applies
to the origin place. The **resolved** origin set is the set minus `immutable-const`,
which creates no conflicting access and has no writable storage [OWN-5] 607,
[OWN-7] 632.

Four uses, and no fifth:

1. **Access strength, over the range.** An access through a value of shared loan
   strength is one shared access to the range of every resolved origin; an access
   through a value of exclusive loan strength is one exclusive access to the range
   of every resolved origin. [VIEW-1] fixes each view's strength. An ordinary
   access to a place that is a resolved origin is judged at the range that access
   reaches, which is the whole place for a whole-place access and the single
   element for a subscript.
2. **A loan covers its address computation.** While a loan on a resolved place is
   live, every binding that place's address computation reads is frozen: a write to
   it conflicts under [OWN-5], at the write, naming the loan. Forming a view at
   `table[k]` therefore freezes `k` exactly as it freezes `table`.
3. **A live origin set fixes its storage.** While a value's origin set is live, no
   statement may write, replace, or exchange the storage any resolved origin of
   that set describes. This clause is **storage-keyed and says nothing about the
   view descriptor**; [VIEW-4] is the rule that governs a commit at a loan-bearing
   place.
4. **Disjointness.** [OWN-7] 630's overlap test extends to ranges: two origins with
   the same resolved place overlap exactly when their ranges intersect, judged by
   the same affine reasoning [PAR-2] 2005 already performs for a single-binder
   element write. This is what makes a `par` fill over one owner expressible.

Use 2 is checkable only because [OWN-7] 630's subscript overlap stays
conservative, and the register's `Depends:` list carries that.

*Judgment:* a loan-bearing value in a prohibited position [BLK-4] is a hard error
there; a write to the storage a live resolved origin describes is the ordinary
[OWN-5] conflict, at the write, naming the loan; and a write to a binding a live
loan's address computation reads is the same conflict. *Publishes:* the origin set,
the resolved origin set, and each origin's range. *Amends:* [OWN-5] 594-611, whose
slice-origin paragraphs generalize to loan-bearing values, whose one access clause
becomes the two of use 1 over ranges, which gains the address-computation and
resolved-set sentences, and whose 608 becomes "a formal view origin has a writable
storage path inside its callee exactly when that view's loan strength on its
resolved origin set is exclusive", the callee-side twin of the [SET-1] change below,
its second sentence unchanged; 601-604's no-slice-valued-join sentence, restated
over the loan-bearing predicate rather than over one retired type name, because the
union of two loans is not a loan any rule can end at one consume; [OWN-7] 630, which
gains the range clause; [SET-1] 488-490, whose "no writable target path may traverse
a `slice<'r, U>` value" is restated as *a target path may traverse a view value
exactly when that view's loan strength on its resolved origin set is exclusive*,
which is what admits the `MutSpan` element write probe `p7` is refused today;
[SET-2] 513-529, whose region-bearing target rejection is replaced by use 3 and
[VIEW-4], and whose "it establishes no fact" sentence becomes false for the
[LIV-3] exchange, whose declared relations land through the added S12 clause;
[EFF-1] 1386, whose "for a direct `slice<'r, T>` parameter, [an effect path] names
the viewed backing state rather than the descriptor" generalizes to a loan-bearing
parameter, which is the declaration-side half [CALL-3] and [VIEW-7] both read; and
[EFF-2] 1406-1410, whose slice-parameter projection generalizes the same way.
*Depends:* [FN-1] 1041-1047, whose call-boundary origin substitution is what carries
an origin into a callee and back; [OWN-7] 630, whose conservative subscript overlap
is what makes use 2 checkable. *Law:* L10. *History:* 6.9, F3 defect 4 and I26;
6.8, F3 defects 1 and 2.

**[PROV-4] `allocates` names a provider path, and reachability reads the leaf.**
The effect grammar's `allocates` entry takes formal-rooted [EFF-1] paths naming
provider state, in canonical order, replacing the fixed atoms:

```text
effect := "reads" "(" effect_path ("," effect_path)* ")"      // [S23]
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

**An allocating row is `reads`, `allocates` and `writes` of the same provider
path**, and A.2 writes all three on both allocating rows. An allocator observes its
prior state while changing it, which is exactly the both-categories case [EFF-1]
1389 already states; the fifth draft's rows omitted `reads`, which made every
library signature that declared it declared-but-unexhibited and every one that
omitted it exhibited-but-undeclared. Probe `t10` is [EFF-2]'s both-ways check firing
on the smaller half of the same mistake.

*Judgment:* [EFF-2]'s both-ways row check, unchanged. *Publishes:* the
provider-reachability closure, and the heap-reaching path, which is the ordered
call chain from `main` to the allocation that [RES-4] prints. *Amends:* [EFF-1]'s
`effect` production (1369-1378), retiring the effect-row atoms `heap` and `arena`;
and [FN-3] 1123-1127, whose conformance effect-row normalization is defined over
"the allocation set whose members are `heap` and each alpha-mapped `arena` region"
and which becomes the set of `allocates` paths under the same parameter-ordinal and
field-ordinal identity 1127 already fixes for `reads` and `writes`, with the region
alpha-mapping applying to modes and types only. *Depends:* [PROG-1] 1492, whose one
closed compilation unit with no function values is why the closure is exact.
*Law:* L2. *History:* 6.9, F3 defect 6; 6.7, F3-12.

**[PROV-5] Reservation is an event of the region block, and one live activation is
the condition.** Two reserving operations exist, differing only in placement:

```text
arena_frame<const bytes: u64, const align: u64>['s]()  -> own Arena<'s, bytes, align>   [S9, S4]
arena_extent<const bytes: u64, const align: u64>['s]() -> own Arena<'s, bytes, align>   [S9, S4]
```

No operand supplies any of those parameters, so each call writes its complete list
in [GRAM-2] 196-198's declaration order, type and const parameters then region
parameters, with each const parameter a lowercase IDENT as [FORM-3] requires:
`arena_frame::<4096, 16, 'a>()`. That is 3.K.0's criterion, not an exception to it.
The written region argument `'s` must be a region introduced by an enclosing
`region_stmt` of the reserving function; a caller-supplied region parameter is not
admitted, and [PROV-1] admits at most one reserving occurrence per region.

**Each reserves one store per activation of the region block naming `'s`.** The
`frame` form lays the extent out in the reserving activation's frame, so it enters
that context's `stack` item of `E`; the `extent` form produces its own
`region(name, bytes, alignment, contiguous)` item of `E`, whose name is derived
from the reserving occurrence and is not written. **On every edge leaving `'s`'s
block the store's release action resets it to its initial state**: the bump cursor
to zero, and nothing else. That action joins [STOR-3]'s release-action table, and
[RES-10]'s `reset` transfer is its arithmetic.

**The refusal is stated over the property, and three sources are named because each
is decidable from declared data.** The fourth draft named `par`; the fifth draft
deleted that disjunct while adding another, which is how a closure loses a member
between drafts:

> An `arena_extent` occurrence is a hard error at its `targ` when **more than one
> activation of its region block can be live at one program point**. Three sources
> are refused by name: membership of a strongly connected component of the call
> graph, read **after** [STK-1]'s rewrite [STK-2]; reachability from more than one
> execution context, a worker lane included (1.5); and reachability from a statement
> an implementation may execute with overlapping execution under [PAR-1], [PAR-2] or
> [PAR-3]. The restructuring is `reserve the store in the caller and lend the
> provider down [PROV-7], or use the frame form`.

The property is the rule and the three sources are the diagnostic. Reading the call
graph after the tail rewrite is stated because [STK-1] turns a tail-recursive
component into one dispatcher with one frame, which really does leave one
activation; reading it before would refuse a program the lowering makes safe.

*Judgment:* the ordinary region, confinement and [OWN-5] exclusivity judgments,
plus the region-locality check, [PROV-1]'s one-store-per-region check, and the
activation refusal above, each a hard error citing PROV-5 at the `targ` with the
restructuring stated there. *Publishes:* the reserved store's measures, its store
region, its envelope item — one `stack` contribution or one `region` item — and the
one-live-store-per-region invariant [PROV-1] reads. *Amends:* [STOR-3] 688-720,
whose release-action table gains the store reset. *Depends:* [ERR-4] 1487, whose
"absence of a complete permission derivation ... never rejects the source" is why
the `par` source is read as *may execute with overlapping execution* rather than as
a permission that was taken. *Law:* L2, L5, L6, L13. *History:* 6.9, F1 finding 12
and F2 F5-10; 6.8, F1 attack 1 and F2 NB2.

**[PROV-6] Linearity is the reclamation half of affine, and disposal,
destructuring and the partial-consume refusal are one closure.**

**The criterion, stated once.** A type is **linear** exactly when

```text
its release action requires a capability, at any depth,
  or its declaration carries the `linear` modifier,
  or it contains, at any depth, a type that is linear by either clause
```

and it is **affine** otherwise. That is the whole definition, and everything the
fifth draft enumerated is derived from it. `Vector<'s, T>` at a `Heap` region is
linear because freeing its run needs the `Heap`, which is a value [L2] and which an
implicit scope-exit release would have to smuggle. `Vector<'s, T>` at an `Arena`
region is affine because its region's reset needs nothing. `FixedVector<T, n>` is
affine because a frame needs nothing. A compiler-owned system resource is affine
because its release is the runtime's, and [RES-9] states why reclassifying
`ReadFile` was considered and refused. Any nominal, enum or run reaching a linear
value is linear by the third clause.

**Linear is a property of an affine type and not a third class.** [OWN-1] 563-564's
classification is unchanged; every linear type is affine; the linear predicate is a
further property of an affine type, defined here, that removes its
compiler-derived release action and fixes its scope-exit disposition. That one
sentence is what lets `move p`, [OWN-13] 654's own-place match move, [SET-2] 516's
affine target requirement and [ERR-3]'s `propagate` operand reach a linear value at
all.

**The `linear` modifier, and what it is for.** `linear struct N { ... }` and
`linear enum N { ... }` **[S18]** are one added modifier on [GRAM-2]'s
`struct_decl` and `enum_decl`. It exists **only for a logical obligation**, because the storage
obligation is already derived: no writer marks a store-derived type, and a writer
who marks one has written a modifier the criterion already implied. What the
modifier says is *silently dropping this value is a bug*, and the values that pass
that test are the ones the criterion cannot see — a transaction that must commit or
roll back, a request that must be answered, a lease that must go back to a specific
pool when the pool is library code holding an affine run. 3.L.7 is the writer
guideline and `CONTAINERS.md` §3.4 is the lease.

This is the one thing in this family that no wf program can have. A writer can write
a pool; a writer cannot write *an obligation to give something back*, because that
is a property of a type and every wf mechanism for it is a runtime field a program
can forget to read. Round 5 measured the gap in the design's own flagship program:
4.1 bound `pool_release`'s refusal and never matched it, losing a block for the life
of the program with no diagnostic, no effect and no envelope movement.

A linear value has **no compiler-derived release**. It leaves a scope by exactly
three routes, and the three are closed under containment together:

```wf-design
let tail = move queue;
let Chunk(page: p, used: u) = move chunk;
dispose table using (heap);
```

— moved out whole, destructured whole [S13], and disposed to its store [S12].

An own-place `match` [OWN-13] is a destructuring: it consumes the scrutinee and
binds each payload as `own`, so the obligation passes to the binders exactly as the
`let` form's does. That is why an `Option<Lease>` cannot be dropped and must be
matched, which is the whole of Q0c's repair.

**Destructure whole.** `let N(f1: b1, ..., fk: bk) = move v;` **[S13]** is one added
`let_stmt` alternative that consumes a value of nominal type `N` and binds every
field of `N` in declaration order to a fresh IDENT, judged exactly as [CALL-4]'s
multi-result destructuring `let` is: each binder is an independent destination,
each receives its field's declared type and `own` mode, and no residual exists for
any rule to define. It is the inverse of `construct`, and it is what makes
"linearity is closed under containment" true of disassembly as well as of assembly.

**Dispose is a consume and a write.** `dispose p using (q1, ..., qk);` **[S12]** is admitted
exactly when `p`'s type reaches at least one leaf whose release requires a
capability. Each `qi` is a **writable provider place**, reached directly or through
a borrow — `dispose old using (deref(heap));` is the spelling inside a helper — and
the statement takes one statement-scoped exclusive access to each, exactly as a
[SET-1] commit does to its target. That is why no `dispose` needs a region of its
own. The statement is, of `p`:

- **one consuming use** [OWN-1] of `p`'s root, so `p` must be a place rooted in a
  live own-mode binding *of this function*, the whole binding is dead afterwards,
  and a `dispose` of a proper sub-place is a partial consume; and
- **one write of `p`'s ultimate storage origin**, exhibited in the statement's
  effect row beside one write of each named provider place, so [EFF-2] projects it,
  [MSR-2] kills over it, [CALL-1] to [CALL-3] classify it, and [PAR-1] 1975's
  footprint contains it.

Both halves are round 5's, and both are repairs of a *release that was not a
write*. The fifth draft demanded writability of the providers and said nothing about
`p`, so `fn peek(v: &Vector<'s,u8>, heap: &uniq Heap<'s>) { dispose deref(v) using
(deref(heap)); }` freed a caller's run through a shared borrow — and [CALL-1]
guarantees in terms that a shared-borrow call kills nothing, because its ground is
that no `writes` occurrence can project onto that place. The consume half refuses
the program at [OWN-1] (content reached through a borrow may never be consumed,
[OWN-5] 591); the write half makes the honest case — a caller disposing what it owns
— kill the caller's measures the way every other write does.

**Its judgment is a walk of `p`'s type**, stated over the type's variant structure
rather than over a flat leaf set:

```text
for a struct or a run element type: every field in [STOR-3]'s order
for an enum:                        the active variant's payload, selected by the discriminant
for a run:                          every element of the initialized window, in ascending logical order
at a capability-released leaf:      release to the store its own type names
at every other leaf:                that leaf's ordinary derived release action
```

For every store region that `p`'s type names at a capability-released leaf, exactly
one named provider whose type names that region must appear, and no named provider
may be unused. **A container's elements are visited before its backing is
released**, so `dispose` on a full container is legal and needs no emptiness premise.

**The walk's depth is the disposed type's containment height, a compile-time
constant, and the walk therefore uses no auxiliary storage.** A type whose
containment graph has a **cycle** has no compile-time height, and this draft refuses
it **at the type, in every program**, rather than denying a resource premise only a
marked entry checks. The fifth draft's disposition left L3's own no-abort clause
aspirational: premise 3 is a hard error only under [RES-4]'s marker, so every
unmarked program — which is every hosted program, 4.2 included — kept the aborting
release walk that probe `a8` shows emitting a `realloc`'d worklist and
`wf_resource_abort`, and probe `x6` shows the type is accepted today. A rule that is
a hard error under a marker and a process abort without one is not one rule.

> A type whose containment graph has a cycle is a hard error citing PROV-6 at its
> `struct_decl` or `enum_decl`, naming the cycle, with the restructuring `hold the
> cells in a run and link by index`.

**A partial consume of a value of linear type is a hard error.** [OWN-1] 569's
"after any consuming use, the whole binding rooting `p` is dead (partial moves kill
the whole binding)" is the one event that makes a linear binding *not live* without
discharging it, and both [LIV-1]'s check and this rule's own error are stated over
live bindings, so the abandoned sibling leaves its scope by none of the three routes
and no rule sees it. The refusal is stated over the **consume** and not over `move`,
which is why it now reaches `dispose c.page` as well as `move c.page`; probes `x4`,
`g7` and `p6_partial` show the `move` shape accepted today and the last shows the
residual being freed by a derived drop. The refusal is stated where the death happens, and its
mechanical fix names the destructuring form.

`propagate` and a live linear binding are mutually exclusive, and this rule says
so rather than leaving it to be discovered. A `propagate` error edge leaves every
enclosing scope and offers no statement position on which to discharge, so a
`propagate` in a function holding a live linear binding is a hard error citing
PROV-6 at the `propagate_let_rhs`, with the restructuring `expand the propagate
into a match and dispose on the Err arm`. Probes `w5` and `m03` compile that shape
today, so this is a refusal the design adds and a cost it owes the writer; Q10 asks
whether a release list on the statement should later remove it.

**One consequence of the criterion, stated because a writer meets it on day one.**
Every heap-derived value in a hosted program is disposed explicitly, with the `Heap`
in hand. 3.L.5 counts seven such statements in `byte_string.wf`. The way to write
fewer is a region block or an arena, whose values are affine; the way to write none
is goal A.

*Judgment:* a linear binding live on any edge leaving its scope, including a
`propagate` error edge and a function-return edge, is a hard error citing PROV-6 at
that edge, naming the binding, its store regions, and the providers a `dispose`
would need; a partial consume of a value of linear type is a hard error citing
PROV-6 at that consume, with the restructuring `destructure the whole value with
let N(f: a, ...) = move v;, or dispose it whole`; a `dispose` whose operand is not
rooted in a live own-mode binding of this function, or whose named providers do
not cover the store regions of `p`'s capability-released leaves exactly once, or
whose operand's type reaches no such leaf, is a hard error citing PROV-6 at the
statement, rendering the uncovered region and the type path that reaches it; and a
type whose containment graph has a cycle is the declaration error above.
*Publishes:* the release events, each store's post-state measure, the statement's
write of `p` and of each provider, and the walk's effect contribution. *Amends:*
[STOR-3] 688-720, whose `box<T>` and `buffer<T>` **heap rows retire with their
types**, so that its derived release covers exactly region-end reclamation, frame
reclamation and the compiler-owned system-resource release; whose table gains the
store reset [PROV-5] and the sentence that a linear type has no derived release; and
whose 709-712 system-resource release contract gains a second subject [RES-9];
[OWN-1] 563-571, whose classification is unchanged and which gains the linear
refinement, the partial-consume refusal, and `dispose` in its consuming-use list;
[GRAM-2]'s `struct_decl` and `enum_decl` (one added modifier), [GRAM-4]'s `stmt` and
`let_stmt` productions (one added statement form and one added `let` alternative)
and [FORM-2], which renders each on one line; [EFF-2] 1427's "each of these
memory-reclamation actions carries the empty effect row", which stays **true** for
the actions that survive and is joined by the walk's own contribution; [PAR-1]
1975's footprint, through the ordinary `writes` row; and [ERR-3] 1472's retained
judgments, which gain the live-linear-binding refusal. *Depends:* [STOR-3] 699-705,
whose derived-drop order and its affine-element clause are the walk this rule
reuses; [OWN-5] 591, whose "content reached through any borrow may never be moved"
is what the consume half of `dispose` inherits; [OWN-13] 654, whose own-place match
move is why a `match` is a destructuring. *Law:* L3, L5, L13, L17. *History:* 6.9,
F1 attacks 2 and 3, F2 F5-7 and F5-8, F4 finding 3; 6.8, F3 defect 3.

**[PROV-7] A provider can be lent onward, generally.** A helper that receives a
provider as `&uniq 'b P` must be able to hand it to the operation that allocates.
Today it cannot: [OWN-6]'s child reborrow admits only a locally-introduced region
whose block does not extend beyond the enclosing statement, so a reborrow into `'b`
is inadmissible and a reborrow into a fresh local region cannot carry an affine
result out. The amendment is [OWN-6]'s own reasoning applied one position further,
and it is stated **generally, over every child reborrow and not only over a
provider**:

> A child reborrow may name a caller-supplied region `'b` that resolved(`h`)'s
> region outlives-or-equals **when the receiving call's result type does not name
> `'b`**. That child's loan ends at the end of its receiving statement, and the
> parent resumes there.

*Judgment:* [OWN-6]'s admission, with one more admitted region source under the
stated result-type condition. *Publishes:* the child loan's extent. *Amends:*
[OWN-6] 616 and [OWN-4] 582. *Verified today:* probes `r1_relend` and `m19` are
`[OWN-6] InvalidChildReborrow`, and `r1_relend_affine` shows the existing
local-region escape cannot carry an affine result out. *Note:* this also unblocks
`docs/patterns.md` P17's threaded-factory shape. *Law:* L2. *History:* 6.8, F4
finding 9; 6.6, F2-N3.

#### 3.K.3 `[BLK]`: the branded run of slots

**[BLK-0] The kernel declaration domain.** The container and store operations are
one compiler-owned **generic** declaration domain, built as [SYS-1] and [SYS-2]
build the system domain and admitted to every compilation unit on the same terms.
Each operation is one complete signature record: named parameters in declared order
[GRAM-11], its type, const and region parameters written as [GRAM-2] 196-198 orders
them, one declared effect row, one declared result mode and type or one ordered
result list, one declared requirement list, and one declared relation list.
**The first declared parameter is the value the operation transforms and returns;
an operation that transforms nothing names its provider first.** The inventory is
Appendix A.2; the rule is that it exists and that every row satisfies the five
sentences below.

**Written arguments, per argument.** A row writes each type, const or region
argument exactly when no operand of that row determines it, and elides it exactly
when some operand does. That is 3.K.0's criterion applied to a domain, and it
replaces the fifth draft's all-or-nothing list, which made `seq_heap::<u8>(heap: ..., count: ...)` a forbidden partial spelling under one rule while both worked programs
and the library wrote it under another. A written type argument may itself be
branded.

**The argument form is named.** A kernel-domain call writes its value arguments as
a `fieldinit_list` in declared order, exactly as a user `fn` and a system operation
do. [GRAM-11] 346 admits that form for exactly "a user `fn` or ... an admitted
system operation", 348 forces positional operands for an [OP-1] table operation,
and 350 resolves callee kind by "the same partition that already selects the
callee", which [OP-1] 838 states. A kernel-domain operation is a fourth class in
all four sentences, [OP-1] 838's included, and [TYPE-6] 401's `callee` IDENT
admission gains it too.

**Every row is complete over every measure it writes, on every exit.** A row
carrying `writes(P)` for a measured `P` publishes, for **each** measure of `P`, its
exact new value where that measure is exact and a two-sided bound where it is
bounded, including the measures it did not change and **on every exit including a
refusal** (L15).

This is round 5's rank-three finding and it is the sentence the fifth draft got
wrong in one clause. It licensed a row to publish two of three measures "where two
of the three follow from [MSR-2]'s identity". They do follow — but only through the
identity, which lives in the affine premise list, so reconstructing `room` from
`len` and `cap` costs two premises before the goal is reached, and the design's own
`spare` invariant then needs three where [ENT-6] 3019 admits exactly two. The
result was that **every appending and draining loop in the fifth draft was
refused**, including both worked programs and every function of 3.L. Probe `g4`
accepts the identical three-term header when one exact relation is published and
probe `g3` rejects it when none is. The identity stays as a convenience for the
writer; it is never the route by which a row's own post-state is derived.

**The readers are not in this domain.** `len`, `cap`, `room` and `head` are four
[OP-1] table operations taking a bare non-consuming place operand and returning
`own u64`, and they are **`pure`**: the operation reads no state the caller does not
already hold, and [EFF-2] attributes the operand's own read exactly as it does for
any other non-consuming table operand — so a **caller** that reads a measure of a
borrowed place exhibits `reads` of it and must declare it. Probes `r2_10` and `t10`
are the two halves. **A `let` binding one of them establishes an equality**:
[ENT-3.S6] 2785's row, today `let m = len(P);` for a tracked `P` establishes
`m = len(P)`, generalizes over [MSR-1]'s four measures, so `let spare = room(v);`
establishes `spare = room(v)` with the same support [MSR-2] gives the term. Without
that one row no `cap`, `room` or `head` value is ever a fact, every branch on
capacity is a fresh unrelated atom, and the whole checked half of 3.L is
unwritable — 3.L.6 records it as one of the eight.

*Judgment:* row resolution by name, receiver type and written arguments; the
per-row requirement discharge under [MSR-4]; and the [GRAM-11] named-argument
check. A diagnostic for an operation cites **[BLK-0]** and names the operation in
its payload, exactly as an [OP-1] diagnostic cites [OP-1]; [DIAG-1] 1541 admits one
numbered language rule and the inventory rows are table data, not rules.
*Publishes:* every declared relation of every row. *Amends:* [SYS-1] 2136 (a fourth
admitted declaration source), [SYS-3] 2309 (admitted to every unit), [TYPE-6]
396-407 (the operation spellings enter the lexical IDENT domain, the nominals the
TYPEID domain, and 401's `callee` IDENT admission gains the fourth class),
[DIAG-1] 1693-1718 (collision rank 5, and a `container_declaration_ordinal` beside
the system one), [ENT-3] 2730 (one added enumerated source S13, plus the arm route
above) and [ENT-3.S6] 2785 (the equality row generalizes over the four measures),
[OP-1] 771-850 (`len` gains `cap`, `room` and `head`, their domain extends to runs,
views and providers, and `slice_of`, `buffer_new`, `buffer_vacant`, `box_new` and
`arena_new` retire; `ReservedLowerNames` gains `cap`, `room` and `head`; 838's
callee partition gains the fourth class), [TYPE-5] 374 (the written-argument
criterion covers a fourth callee class and becomes per-argument), [GRAM-11] 346-350,
and [FN-2] 1093 (its explicit-argument rule covers this domain). *Law:* L11, L15,
L16. *History:* 6.9, F1 attack 4, F3 defects 2 and 6; 6.8, F3 defects 6 and 7.

**[BLK-1] Two runs, one shape, one window, and what a slot may hold.** Exactly two
container nominals, differing in two properties for two reasons:

```text
| type [S1, S2]       | capacity            | storage              | release needs  |
|---------------------|---------------------|----------------------|----------------|
| FixedVector<T, n>   | the type constant n | inline in its owner  | nothing        |
|                     |                     | or the stack frame   |                |
| Vector<'s, T>       | a measure, fixed at | one run taken from   | what 's needs  |
|                     | the take            | the store 's names   |                |
```

**Each is a run of slots whose initialized storage is a window.** The initialized
set is exactly the `len` slots beginning at `head`, taken modulo `cap`; the rest is
raw. `len`, `cap`, `room` and `head` are [MSR-1]'s terms with [MSR-2]'s facts and
[BLK-0]'s readers. A run carries no other state: no per-slot tag, no occupancy
bitmap, no runtime discriminant (L12). A subscript `v[i]` selects the element at
**logical** offset `i`, which is the slot at physical offset `(head + i) mod cap`,
and carries the ordinary [OP-4] obligation `i < len(v)` — against `len`, never
against `cap`, and never against `head`. A `Vector<'s, T>` of capacity one is a
single stored value, so the language needs no box nominal and [TYPE-7]'s deref
domain loses its three. `array<T, n>` is retained exactly as it is, as the
`len = cap = n`, `head = Z` case with no typestate and a copy-only element domain,
so `tests/programs/fir_filter.wf` is untouched.

**Why a window and not a prefix, and what it costs.** The fifth draft's prefix made
L12's own last clause false: a queue is not arithmetic a writer performs over append
and remove-at-the-end, and the price of pretending otherwise was a library ring over
`Option<T>`, which round 5 measured at **2072 bytes against a hand-written 280** for
a 256-byte ring under [OP-9] 992's own ceiling, and which deletes in-place slot
mutation because no place reaches inside an enum payload. The window makes a ring a
**run**: no `Option`, no tag, ordinary element access, exact `len`. It also deletes
`seq_exchange`'s last demander, because a growth policy drains from the front and
appends at the back, in order, with no reversal to undo.

Adding a fifth kernel row for removal-at-the-front and shifting the survivors would
have been the smaller-looking change and is the wrong one twice over: it is O(n) per
removal in the one loop a driver cares about, and a shifting `take_front` is
**writable in wf** over the four rows a prefix already has, so L18 would keep it out
of the kernel anyway. Only the head-carrying form is unwritable, because only it
makes the boundary checker-maintained rather than a data field. L18 selects the
window.

Its cost is four things, and no fifth:

1. one word per descriptor, which A.1 carries;
2. one more measure term, `head`, in [MSR-1]'s table and in every row that writes a
   run — four columns where the fifth draft had three;
3. one standing fact, `head(P) <= cap(P)`, beside the three the identity already
   gives; and
4. one requirement on view formation, `head(vector) <= 0_u64` [VIEW-2], because a
   `Span` is contiguous and a wrapped window is not. Every formation row publishes
   `head(result) = Z` and every back operation preserves it, so a program that never
   removes from the front discharges it by a chain of exact equalities and states
   nothing.

Lowering pays one add and one conditional subtract per subscript. That is a runtime
cost and not a proof cost, and an optimizer that proves `head` identically zero for
a given run emits the ordinary `base + i * stride` — an optimizer fact that improves
an accepted program and changes no acceptance, which is the only kind this language
admits.

`T` may be copy, affine, or linear; the trichotomy is [OWN-1] 564's two classes plus
[PROV-6]'s refinement, and this sentence names it rather than restating it as three.
The window is what makes an affine element sound: an element enters and leaves only
through an operation that moves a boundary, so no slot is read before it is written
or after it is taken. A run over a linear `T` is itself linear [PROV-6], and
`dispose` walks its window.

*Judgment:* the ordinary nominal-resolution and construction judgments; a
`construct` naming a container nominal is a hard error citing BLK-1; [OP-4] at
every subscript, against `len`. *Publishes:* the two types, their measure rows and
their window typestate. *Amends:* [TYPE-2] 357, two added composite types, and its
flat-element restriction, which the runs do not inherit; [OP-4] 914, whose
indexable bases extend to the two runs, `Span` and `MutSpan`, and whose obligation
is against `len`. *Verified today:* `array_new::<box<u64>, 4>` is [OP-1]
`InvalidOperation` (probe `p9`), so an affine element is new capability. *Law:*
L12, L13. *History:* 6.9, F4 blocking 4; 6.8, the minimality ruling.

**[BLK-2] Formation, one row per placement and one per store.** Four rows, and no
fifth:

```text
seq_fixed<T, const n: u64>()                        -> own FixedVector<T, n>   pure  // [S7]
seq_arena<T>['s](arena: &uniq Arena<'s,bytes,align>, count: own u64)
                                                    -> own Option<Vector<'s, T>>
seq_arena_proved<T>['s](arena: ..., count: own u64) -> own Vector<'s, T>
seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)
                                                    -> own Option<Vector<'s, T>>
```

**The fifth draft's `seq_frame` is deleted**, and it is worth one paragraph because
it is the clearest instance of the pattern §2.1 exists to catch. It produced a
`Vector<'s, T>`, so [PROV-1] put a store region in its type; but no reserving
occurrence named that region, so [PROV-1]'s one-live-store invariant had nothing to
quantify over, [PROV-5]'s activation refusal did not reach it, [PROV-6]'s predicate
classified it by no clause, [STOR-3] gave it no release action, and [RES-1] and
Appendix A.2 disagreed about whether its envelope item was a `stack` contribution or
a `region`. Every one of those is a notion whose closure sentence the row violated.
And its capacity is a written const, which is exactly what [PROV-1]'s two-differences
argument says makes a run `FixedVector<T, n>`, so it was also a duplicate. A
frame-placed *arena* run remains available and is `arena_frame` plus `seq_arena`.

**Every failure is an `Option` and the kernel declares no failure nominal.** L3
requires a failure to hand back every affine input it did not consume, and no
kernel acquisition takes one: a count is copy and a provider is borrowed. So the
three failure structs and the fourth draft's `NoRecord` all leave the kernel, and a
library that wants to return an owner inside a refusal declares its own nominal over
its own type — `CONTAINERS.md` §3.3 writes one. The `Heap` has **no proved form**,
because no honest domain predicate exists for a general store (L6); the arena has
one, whose requirement [MSR-4] discharges and whose failure is a static rejection
with no fallback, exactly as an unproved subscript is.

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge at the proved spelling.
*Publishes:* each run's measures, and each store's post-state measures and refusal
relation. *Amends:* [OP-1] 798-803 (`buffer_new`, `buffer_vacant`, `box_new` and
`arena_new` retire); [TYPE-2] 357. *Law:* L3, L4, L6, L8, L18. *History:* 6.9, F1
attack 5 and F3 defect 13; 6.8, the minimality ruling.

**[BLK-3] Four operations move a boundary, and nothing else does.** `V` is either
run type.

```text
seq_place(vector: own V, value: own T)        -> own V   // [S8]  requires room(vector) > Z
seq_place_front(vector: own V, value: own T)  -> own V           requires room(vector) > Z
seq_take(vector: own V)                       -> (rest: own V, value: own T)
                                                                 requires len(vector) > Z
seq_take_front(vector: own V)                 -> (rest: own V, value: own T)
                                                                 requires len(vector) > Z
```

Element access is the ordinary v0.41 surface over the initialized window: `v[i]`
reads, `set v[i] = e;` writes a copy element [SET-1], and `let old = replace v[i] =
e;` exchanges an affine one [SET-2]. That surface is what a keyed table is built out
of, and probe `x7` compiles its shape today.

Each takes the run **by value** and returns it, carries `reads(vector),
writes(vector)`, and publishes its complete measure row on every exit.

**`seq_exchange` is not a row**, and footnote 6 is why: round 5 wrote it in wf in
three statements over rows this table already has —

```wf-design
let (rest, endv) = seq_take(vector: move v);
let old = replace rest[i] = move endv;
let back = seq_place(vector: move rest, value: move old);
```

— which is the transposition of `i` with the last position, for copy, affine and
linear element types alike, and transpositions with one fixed position generate
every transposition. L18 therefore removes it, and 3.L.2 writes the swap. **What it
costs to write it that way is real and is stated rather than hidden**: the three
statements kill and re-establish `len` twice where one row would have published
`len(result) = len(vector)` once, so a loop that swaps under a header invariant
carries the measure through three steps instead of one. That is a proof-surface cost
a writer pays for a capability the kernel does not owe them, which is exactly the
trade L18 asks for.

There is **no removal from the middle, no clear, no truncate, no growth, no filled
construction and no vacant construction** in the kernel. Each is written in wf in
3.L, and 3.L.6 records that none of them needed a primitive the four rows above do
not have.

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge of each requirement.
*Publishes:* each row's declared relations. *Amends:* nothing beyond [BLK-0]'s.
*Verified today:* probe `c8` shows a function writing one position of an
`own buffer<u8>` parameter and returning it must exhibit `writes(vector)`, so these
rows are not `pure`. *Law:* L4, L9, L12, L15, L18. *History:* 6.9, F3 defect 16 and
F4 blocking 4.

**[BLK-4] Confinement, and the one position closure.** A type is **confined** when
its complete type after substitution names a region. The confinement of a value is
the **set** of regions its complete type names, and it may be moved, returned, or
bound to a destination that **every** member outlives-or-equals [OWN-3]. That
quantifier is the whole rule: a value of type
`Result<Vector<'s, Page>, Shortfall<Vector<'q, u64>>>` names two regions, which
[OWN-3] 580 makes incomparable, and fail-closed is the right answer.

A confined value may occupy any position whose owning value's own complete type
names the same region, so the position is itself confined and [STOR-4] governs it.
That is what admits a store-branded value into a field, a run element and an enum
payload, and it is safe because the store's identity travels in the type into the
position and back out of it [PROV-1].

A **loan-bearing** type [PROV-3] may occupy no position from which a value could
outlive or hide its origin set: no field, no enum payload, no element of any run,
no generic type argument, and no result outside [VIEW-6]'s ceiling. A **provider**
type may occupy none of the same positions, for [PROV-2]'s different reason. A
store-branded run may occupy any of them, because it is a type parameter and
aliases nothing. Three clauses, three reasons, one closure.

**A source nominal may declare region parameters**, written exactly as a function
declares them, and is confined by them:

```wf-design
struct Chunk['s] {
  page: Vector<'s, u8>;
  used: u64;
}
```

— region parameters on a nominal, [S20] — and it is used as `Chunk<'s>`, an ordinary
TYPEID with `targs`. Under 3.K.0 a nominal
over the entry heap declares no region parameter at all and is written `Chunk`; the
parametric form is what a program with two stores writes. Two instances of one such
nominal have the same type only when their region arguments are identical: region
parameters on a nominal are **invariant**, which is [OWN-12] 650 and [TYPE-5] 379
applied where they already apply, and which is why this feature needs no variance
design.

**A stored position with no admissible brand is this rule's error, not a
resolution failure.** When a nominal declares no region parameter, the entry selects
no `command.heap`, and a field's type needs a store brand, the position is a hard
error citing BLK-4 at the complete contained `type`, with the restructuring `give
this nominal a region parameter and confine the field to it`. That is 3.K.0's
non-emptiness sentence given a home.

*Judgment:* a loan-bearing or provider type in a prohibited position, or a confined
type in a position whose owner does not name its region, is a hard error citing
BLK-4 at the complete contained `type`, with the restructuring `keep the view as a
direct local, parameter, or result` for the first, `lend the provider to the
operation that needs it` for the second, and the sentence above for the third; and a
confined value bound to a destination some member of its region set does not outlive
is a hard error citing BLK-4 at the binding, rendering every member. *Publishes:*
the confinement set. *Amends:* [STOR-4] 721, whose "may not be returned" becomes the
ordinary outlives relation over the set; [STOR-5] 723-737, whose enumerated position
list is replaced by the intensional split above and whose deferral of per-leaf
provenance inside stored values is **withdrawn as unnecessary** rather than
discharged, because a store brand is a type parameter and needs no per-leaf record;
[FN-2] 1093, whose blanket rejection of a region-bearing generic argument narrows to
loan-bearing and provider arguments and whose "instantiation arguments are always
explicit" now covers region arguments on nominals; and [GRAM-2]'s `struct_decl` and
`enum_decl`, which gain `region_params?` after `generics?`. *Depends:* [OWN-3] 580,
whose fail-closed incomparability is the invariance argument. *Verified today:*
probe `f7_regionresult` is [FN-2] `RegionBearingGenericArgument` and probes `r2_6`
and `m05` are [GRAM-2] parse errors at `struct Wrap['p]`, so both halves are new.
*Law:* L10, L13. *History:* 6.9, F1 attack 6; 6.7, F1-6.

*[CNT-1] through [CNT-7] and [SEQ-0] are deleted.* Five owners, a per-owner release
table, a `&uniq`-container prohibition, a growth rule and an operation-domain rule
are [BLK-0] through [BLK-4] plus 3.L. [CNT-7] is worth its own sentence, because it
is the one whose disposition changed twice. It refused a `&uniq` parameter whose
direct type is a container; round 4 showed the refusal nullified by a one-field
wrapper struct; the fifth draft deleted it and let [CALL-5]'s conservative kill
carry the shape; round 5 showed that kill defeated by a fact published *after* it.
**R1 restores its effect without its text**: a helper takes a run by value and
returns it, so there is no borrowed container to protect, no wrapper to nullify the
refusal with, and no channel that can carry a fact past a kill. The ids are retired
and not reused.

#### 3.K.4 `[VIEW]`: views and loans

**[VIEW-1] The two views.**

```text
| type [S5, S6]   | reads | writes elements | changes length     | loan      | affine |
|-----------------|-------|-----------------|--------------------|-----------|--------|
| Span<'r, T>     | yes   | no              | no                 | shared    | yes    |
| MutSpan<'r, T>  | yes   | yes             | no, fixed by type  | exclusive | yes    |
```

Each is an `own` affine value carrying a region `'r`, exactly as `slice<'r, T>`
does today, and each is loan-bearing [PROV-3]. `Span<'r, T>` **is** today's
`slice<'r, T>` renamed; the rename is the whole of the change to it. Its measures
are [MSR-1]'s rows, with `head` exact at `Z` because a view is formed only over an
unwrapped window.

There is no third view, and under R1 there is no work for one. The fourth draft's
`AppendView` presented a run's spare window so that a caller's length could survive
an appending callee; the fifth draft replaced it with an exit datum over a `&uniq`
parameter, which round 5 broke. Under R1 an appending helper takes the run **by
value and returns it**, so the caller's length is the result's length, published by
an ordinary `ensures` over an ordinary result. **What a writer gains back is the
guarantee L14 was retired for**: `ensures len(rest) >= len(out)` says that the
helper did not shorten what it was handed, relates a result to an input, names one
state of each, and needs no `old()`, no frame rule and no third type.

*Judgment:* none by itself. *Publishes:* the two types and their loan strengths.
*Amends:* [TYPE-2] 357 (one added view type, `slice` renamed `Span`), [OWN-1] 563
(both are affine), and [CONST-2] 552-556, [OP-7] 940 and [OP-1]'s `slice_of` row,
which name the retired spellings. *Law:* L10. *History:* 6.9, R1; 6.8, footnote 3.

**[VIEW-2] Formation, the loan the view value holds, and the unwrapped premise.** A
view is formed from a borrow of the run:

```text
seq_span['r, T](vector: &'r v)          -> own Span<'r, T>      reads(vector)   // [S10]
    requires head(vector) <= Z
seq_mut_span['r, T](vector: &uniq 'r v) -> own MutSpan<'r, T>   reads(vector)
    requires head(vector) <= Z
```

and **the view value, not the argument borrow, holds the loan**. For its whole
life, a view value holds a loan of its own strength on the range it reaches of
every place in its resolved origin set [PROV-3]. The loan begins at formation and
ends when the view value is consumed or released. The argument borrow is a
call-scoped temporary, which probes `f2b`, `r1_twouniq` and `w8` confirm by
accepting two of them on one place in one region with an ordinary write between; it
could not be the freeze.

The `requires` is the window's one visible cost [BLK-1]. A `Span` is a contiguous
range and a wrapped window is two, so formation is admitted only where the window
begins at the run's own origin. With the standing `Z <= head` the requirement is
`head = Z`, every formation row publishes it, and every back operation preserves it,
so a program that never removes from the front discharges it by a chain of exact
equalities and writes nothing. A program that does — a ring — either forms no view
or drains into a run that has one; `CONTAINERS.md` §3.2 writes the drain.

*Judgment:* [OWN-5] at the formation borrow, [MSR-4] discharge of the unwrapped
requirement, and the ordinary [BLK-0] relation establishment. *Publishes:* the loan
and the two formation rows' relations. *Amends:* nothing beyond [PROV-3]'s
amendment of [OWN-5]. *Depends:* [OWN-5] 606, the conflict sentence that refuses a
second exclusive view, and [OWN-6] 614, which makes the argument borrow call-scoped.
*Law:* L10, L15. *History:* 6.9, F4 blocking 4; 6.8, F1 attack 20.

*[VIEW-3] and [VIEW-5] are deleted.* [VIEW-3] was `absorb`, the append window's
commit event, and [VIEW-5] the disposition of an abandoned window. Both retire with
`AppendView`; their ids are not reused.

**[VIEW-4] A view descriptor's length cannot be changed through a borrow, and a
commit at a loan-bearing place is admitted only when the displaced value is
consumed.** No operation takes a `MutSpan` or a `&uniq` to one and produces a
different length, and none changes its owner's length.

The first ground is [LIV-2]: it admits a reinitializing `set` only for a bare
binding **declared in the current function**, and a `&uniq 'b MutSpan<'r, T>` holder
in a callee is a borrow of a descriptor the caller declared, so no callee can
reinitialize it.

The second ground was wrong in the fifth draft and is restated here, because round 5
found the one shape it left admitted. The fifth draft attributed the refusal of
`replace deref(handle) = fresh;` to "[LIV-3]'s consume premise" — but [LIV-3]
governs `set p = f(q: move p, ...)` and reaches no `replace` at all, [SET-2] 518's
region-bearing rejection had been replaced out from under it, and [PROV-3] use 3 is
storage-keyed and says so. So the statement was admitted by every rule that could
have refused it, and B6's own test list demanded a rejection whose rule did not
exist. The repair is to state the condition over the **commit** rather than over one
statement form, and to state it here, where both forms meet:

> A commit that displaces a value of loan-bearing type is admitted exactly when the
> displaced value is consumed by that same statement's right-hand side. [SET-2]'s
> exchange under [LIV-3] satisfies it, because the displaced value is the `move p`
> operand; a `replace` at a loan-bearing place does not, because its replacement is
> written by the writer and the displaced value survives as the fresh binding, so
> the loan the displaced view held would outlive its own descriptor.

That refuses `replace deref(handle) = ...` at exactly the right point, admits every
`set p = f(move p, ...)`, and leaves `replace` at a non-loan-bearing place — a
keyed table's `Option<T>` slot, a run element — untouched.

Therefore `MutSpan<'r,T>` and `&uniq 'b MutSpan<'r,T>` are both length-fixed for
[CALL-3].

*Judgment:* the commit admission above, a hard error citing VIEW-4 at the complete
target `place` with the restructuring `consume the displaced view in the same
statement, or bind a new view under a new let`. *Publishes:* the length-fixed class.
*Amends:* [SET-2] 513-529's admitted commits, beyond [PROV-3]'s amendment of it.
*Depends:* [LIV-2]'s bare-binding-declared-in-this-function premise. *Law:* L11.
*History:* 6.9, F3 defect 4; 6.8, F3 defect 1 and I19.

**[VIEW-6] Views are never stored, and a view result declares its origin.** A view
is never stored [BLK-4] and never returned except under this rule. [FN-1]'s
slice-result ceiling applies unchanged to each view type: a function whose written
result is `own Span<'r, T>` (respectively `MutSpan`) has the ceiling containing
`immutable-const` and the formal-view origin of every parameter whose written mode
and type are exactly that same view type with the same formal region and element
type.

**An ordered result list containing two results of the same view type and the same
formal region is a hard error citing VIEW-6 at the `result_binding` of the second**,
with the restructuring `give each result its own formal region`. Without it a
three-output demux written with one region returns three views each aliasing all
three inputs.

One consequence is recorded because it is a real restriction: [FN-1]'s containment
check forbids a helper from manufacturing a view of storage it reaches through a
borrow, so `seq_span` and `seq_mut_span` are usable only in the function that
directly owns the run, and **no helper library over views can exist in this
version**. Under R1 that costs less than it did under the fifth draft, because a
helper that transforms a run takes the run itself; what it still costs is that a
helper which only *writes elements* must be handed a view the caller formed.
Disposal is not confined this way, because [PROV-6]'s walk compares types and not
places.

*Judgment:* [FN-1]'s ceiling containment at every `return_stmt`, plus the
same-region result rejection. *Publishes:* the result's origin set. *Amends:*
[FN-1] 1023-1036, by generalizing "slice" to "view" and by adding the same-region
rejection. *Law:* L10, L11. *History:* 6.5, F4-7.

**[VIEW-7] System operations over views.** The seven range-bearing operations
[SYS-8] take views instead of `buffer<u8>`, and their modes are fixed rather than
left to a table cell:

```text
a destination the operation writes  ->  &uniq 'd MutSpan<'r, u8>
a source the operation reads        ->  &'s Span<'r, u8>
```

so `read_at(file: &ReadFile, destination: &uniq MutSpan<u8>, file_offset: own u64,
start: own u64, end: own u64) -> result: own ReadOutcome`, whose three regions
relate nothing and are therefore all elided (3.K.0).
Both are borrows of the **descriptor**, so the view survives the call and a
destination can be filled by a loop of reads, which an `own` destination could not.
Both are length-fixed [VIEW-4], so [CALL-3] gives the caller its measures back. The
two obligations keep their form and their order with `len(deref(buffer))` reading
`len(deref(destination))`.

This is the change that lets a heap-free program do I/O, and it is a rule rather
than a register row because it is goal A's container half. Its cost is that a
destination must be **addressable** before the host writes into it, so it is built
by 3.L.3's `filled` and the count the host produced is an ordinary `u64` beside the
run rather than the run's own `len`; Q7 records the fix. Its second cost under the
fourth draft — two writer-visible regions per I/O site — is **gone**: both relate
nothing, so both are elided under 3.K.0, and Q11 is answered rather than deferred.

*Judgment:* [SYS-8]'s two range obligations, restated over `len` of the borrowed
view. *Publishes:* the endpoint facts [ENT-3.S10] already enumerates, now over a
view. *Amends:* [SYS-8] 2488-2527, [SYS-2] 2164-2307's declaration records and its
normative counts, and the prose of [SYS-9], [SYS-11], [SYS-12] and [SYS-14], which
name `buffer<u8>`. *Depends:* [EFF-1] 1386 as [PROV-3] amends it, which is what
makes a view parameter's effect path name the viewed backing rather than the
descriptor. *Law:* L11. *History:* 6.8, F1 attack 9; 6.7, F3-10.

#### 3.K.5 `[LIV]`: liveness, reinitialization, and the in-place exchange

**[LIV-1] Liveness is join-checked, and that is what makes release
unconditional.** A binding's live-or-dead status is a property of a program point,
not of a path: at every join of the conservative structural graph [FN-1], and at
every loop head, every predecessor must agree on the status of every binding in
scope. A disagreement is a hard error citing LIV-1 at the join, naming the two
predecessors and the binding. On every edge leaving a scope, a `propagate` error
edge and the function-return edge included, every **linear** binding of that scope
must be dead [PROV-6], because no derived release exists to carry it.

**This rule states its own two amendments rather than leaving them to the
register.** [OWN-11] 646's "a binding declared outside that body may not be moved
inside it" is **replaced** by the join agreement above: the prohibition exists
because a moved-and-not-restored binding makes the loop head disagree with the
preheader, and the join check decides exactly that question, so a loop that moves an
outer binding and restores it before the backedge is admitted and one that does not
is a LIV-1 error naming the head. Round 5 found the fifth draft asserting that
replacement only in its register table, with the rule's own body silent — and both
worked programs depending on it. And [OWN-1] 566-567's "SET-1 and SET-2 recheck the
live-root premise after their right-hand sides and never revive a dead binding" is
**kept**, because [LIV-2]'s reinitialization is a new admission at a dead target and
not a revival of a dead root inside one statement.

*Judgment:* a per-join and per-scope-exit structural check over the ownership state
the checker already computes; no search. *Publishes:* the unconditional release set
of every edge. *Amends:* [OWN-1] 563 and [OWN-11] 646, as stated above. *Law:* L17.
*History:* 6.9, F3 I9; 6.5, F1-1, F1-2.

**[LIV-2] Reinitializing `set`, and a new declaration event.** `set p = e;` is
additionally admitted when `p` is a bare binding of affine type declared in the
current function, a `let` binding or a parameter, `e` produces exactly `p`'s type,
and **`p` is dead at the commit point**, whether `p` died before the statement or
inside `e`.

Its judgment: evaluate `e` under ordinary rules; every fact whose support contains
`p`'s root dies at the consume that killed it; then the binding is reinitialized with
`e`'s value, live and usable, with no observable program point between. It derives no
drop and no release, because the target holds no value at the commit.

**A reinitializing `set` is a declaration event for [ENT-2] term identity**, and
[MSR-3]'s atom-identity sentence states what follows: the reinitialized binding is
a *distinct term* from the consumed one, exactly as [ENT-2] 2683 already rules for
"a fresh binding legally reusing an expired spelling", so a fact stated over the
old value never reaches the new one. [MSR-3] also makes the case where that silently
costs the writer a fact — a reinitializing `set` of a binding a live header
invariant names — a diagnostic rather than a silence.

*Judgment:* the deadness premise plus the ordinary [TYPE-5] exact-type check.
*Publishes:* the new binding's term identity and its measure images. *Amends:*
[ENT-2] 2683's term-identity paragraph (one added declaration event), [OWN-1] 569's
"reinitialization requires a new `let`", [STOR-1] 679 through [LIV-3]'s restated
partition, and [SET-1] 481-505, whose affine-target rejection, dead-root sentence and
post-right-hand-side revalidation together carry the old premise. *Verified today:*
probe `p10` is [STOR-1] `AffineSetTarget` for a live target and probe `w6` is
[OWN-1] `UseAfterMove` for a dead one, the two halves this rule replaces. *Law:*
L10, L16, L17. *History:* 6.7, F3-8.

**[LIV-3] The in-place exchange, which is an admission on `set` and not a new
statement.** `set p = f(q: move p, args);` **[S15]** is additionally admitted when
`p` is a writable place of affine type, `f` is any call — a user `fn`, a
kernel-domain row, or a system operation — whose first result has exactly `p`'s
type, and `move p` occurs **exactly once** in `f`'s argument list. Its multi-target
form **[S14]** is

```wf-design
set p = seq_place(vector: move p, value: byte);
set (p, taken) = seq_take(vector: move p);
```

where the first target is the exchanged place and every later target introduces a
fresh binding.

**The amendment this form needs is [STOR-1] 679's, and that sentence refused every
statement of the fifth draft.** 679 reads "Setting an affine-typed final place with `set` is a
hard error citing STOR-1 at the complete target `place` ... the restructuring `use
replace`", and 678 partitions writable final places by [OWN-1] class with one
spelling each. The fifth draft admitted this form over a **live affine** place and
amended [SET-1], [SET-2], [GRAM-4], [ENT-3.S12] and [FORM-2] — but not [STOR-1], and
its only amendment of 679 narrowed it "to a live target", which is exactly this
target. Every function of 3.L, both worked programs and every companion snippet died
at their first exchange; probes `t8` and `x2`/`x3` are that rejection today. The
repair is not an exemption for one form but the partition restated once, because the
language now has three writing forms where v0.41 has two:

> [SET-1] may overwrite only a copy-typed final place; [SET-2] may replace an affine
> one; and **a `set` whose right-hand side consumes the target's own previous value
> — a reinitializing `set` [LIV-2] at a dead target, or an in-place exchange
> [LIV-3] at a live one — is the third form, judged by its owning rule.** 679's
> diagnostic keeps the case it was written for: a live affine target with an
> unrelated right-hand side.

**Its judgment is [SET-2]'s, not [SET-1]'s, and that is what makes it not sugar.**
The previous value is read out of `resolved(p)` into the operation's named
parameter, the operation runs, its first result is written back into `resolved(p)`,
and each later target initializes its binder. There is no writer-observable program
point between the read and the write (spec 520), so there is no partial move, no
dead root and no uninitialized hole, and the root binding stays live (spec 521).

**Its effect footprint is [SET-2]'s.** An in-place exchange exhibits one read and
one write of the target's ultimate storage origin, and the call's own projected row
in addition. The fifth draft stated what the form proved and what it published and
nothing about what it wrote, which left three other rules reading a footprint no
sentence supplied: [MSR-2]'s kill classification at the caller, [PAR-1] 1975's
overlap test for a window containing an exchange, and [CALL-3]'s classification when
the exchanged place is itself an argument's referent. Deriving a field-precise
footprint from the callee's row would be wrong, because the value written back is a
whole new value of the target's type and the callee's row describes what it read and
wrote *inside* that value.

**Its later targets are ordinary `let` bindings introduced at the statement.** Each
receives its ordinal's declared type and `own` mode, its scope is the enclosing
block exactly as a `let`'s is, it is a declaration event for [ENT-2] term identity
[MSR-3], and [LIV-1]'s join agreement and [PROV-6]'s scope-exit check quantify over
it like any other binding. That is the sentence [PROV-6] already writes for the
destructuring consume's binders, quantified over ordinals instead of fields; without
it a linear later target has no scope any rule can check, and the failure mode is a
silent leak per iteration rather than a diagnostic.

**This is the one form the partition test could not write in wf**, and the reason
is worth stating because it decides the whole convenience question. At a bare
binding the writer could rebind: `let next = f(q: move p, ...); set p = move next;`
is two statements and [LIV-2] admits the second. At every other place they cannot.
`move p[i]` and `move p.f` are partial moves that kill the root [OWN-1] 569, and
`move deref(h)` is a move through a borrow, which [OWN-5] 591 forbids outright with
[SET-2]'s exchange as the sole exception. So the only route is a placeholder —
`let old = replace p[i] = <something>;` — and a placeholder must be a value of the
displaced type, which for a `Vector<'s, T>` is a run that owns storage and is
itself linear, so every transformation costs an allocation and a disposal on a
provably dead arm, and for a type with no cheap empty value there is no route at
all. Probes `t8`, `x2` and `x3` are the rejections today, and `x2`'s own mechanical
fix names the field-by-field fold that is exactly the ceremony this removes.

An exchange is **not** a declaration event [MSR-3] at its first target: the root's
term survives, the facts over it die by [MSR-2], and the call's declared relations
re-establish them on the same term through **one added [ENT-3.S12] destination
clause**, stated in [CALL-4] and serving four forms.

*Judgment:* the single-occurrence check on `move p`, the result-count and type
checks, then [SET-2]'s exchange judgment. *Publishes:* the call's declared
relations, on the written-back place and on each later target, and the statement's
read and write of the target's ultimate storage origin. *Amends:* [STOR-1] 678-679
(the writable-place partition, restated above), [SET-1] 481-505 (one added
admission), [SET-2] 513-529, which gains a compiler-derived exchange whose
replacement value is derived from the read-out rather than written by the writer,
whose target may be linear or region-bearing because nothing is rebound, and whose
"it establishes no fact" sentence becomes false for this form; [GRAM-4]'s `set_stmt`
production (a target list); [ENT-3.S12] 2833's destination list, through [CALL-4];
and [FORM-2], which renders the form on one line. *Verified today:* probes `t8`,
`x2` and `x3` are [STOR-1] `AffineSetTarget`, so this is new capability and not a
compiler defect. *Law:* L10, L18. *History:* 6.9, F3 defects 1 and 3, F1 attacks 9
and 10; 6.8, F1 attack 4.

#### 3.K.6 `[CALL]`: what survives a call

This is D1's section, and under R1 it is shorter than it was. Exactly three
transports exist, and **each reads only the callee's declared parameter modes and
types and its declared contract.** These are the owner's three call rules of
2026-09-03.

**[CALL-1] Through a shared borrow, every fact survives.** For an argument whose
parameter mode is `&'r`, of any type, run and view included, the call is not a kill
event for any fact supported by the actual's resolved place. Ground: [OWN-5] admits
no write through a shared holder, so [EFF-2] can project no `writes` occurrence
onto that place, so [MSR-2]'s kill does not fire.

**That ground is exactly as strong as the set of actions classified as writes**,
which is why [PROV-6] had to make `dispose` one. Round 5's second attack is a callee
that takes `&Vector<'s, u8>` and disposes its referent: every clause of this rule was
true, its ground held in both halves, and its conclusion was false, because a release
was not a write. The repair is in [PROV-6] and not here, and this sentence records
the dependency so that the next action added to the language is checked against it.

*Judgment:* none; the absence of a kill. *Publishes:* the survival of every such
fact. *Amends:* nothing. *Depends:* [OWN-5] 585-606's shared-holder prohibition,
which is the whole ground. *Verified today* for `&'a buffer<u8>`: probe `p6` keeps
`len(line) = 10` across the call and the subsequent `line[9_u64]` is accepted.
*Law:* L11. *History:* 6.9, F1 attack 2.

**[CALL-2] Through a value passed and returned, only the contract's facts exist on
the result.** An `own` argument is a consuming use, so every fact whose support
contains that binding's root dies. The result is a fresh binding carrying exactly
the callee's verified relations, and nothing else. Those relations may name the
consumed parameter's measure, which denotes that call's **call datum** [MSR-3]:
`len(rest) = len(out) + 1` means what it reads as, and it is establishable at the
caller precisely because a datum has empty support and the consume the same
statement performs cannot kill it.

**Under R1 this is the transport a container helper uses**, and it is the reason R1
costs the writer nothing: a helper's `ensures` names its own result and its own
inputs, both of which the caller can see, so there is nothing for a callee to be
wrong about and nothing for a caller to read in a vocabulary the callee did not
have. The fifth draft's exit datum was an attempt to give a *borrowed* parameter the
same expressiveness, and it failed for a reason that is now stated as law: a
relation about a caller's object, made at a point the callee cannot name, is not a
relation the callee can prove (L11).

*Judgment:* the ordinary [ENT-3.S12] establishment, subject to `M(c,q)` as [MSR-3]
amends it. *Publishes:* the callee's declared relations on the result. *Amends:*
nothing beyond [MSR-3]'s. *Verified today:* probe `p1`, `passthrough(out: move a)`
returning the same buffer, then `b[9_u64]`, is **rejected** with residual
`9_u64 < len(b)`; the transport already behaves correctly and what was missing is
the vocabulary to publish across it. *Law:* L11. *History:* 6.9, R1.

**[CALL-3] An element write through a length-fixed view never touches length
facts.** For an argument whose parameter's declared type is `MutSpan<'r, T>` or
`&uniq 'b MutSpan<'r, T>`, which [VIEW-4] fixes a length for, a projected callee
`writes` occurrence kills every fact whose support overlaps the viewed **element
storage** and kills no measure term over that origin. For every other parameter
type the projected write kills measures as an ordinary descriptor-storage-overlapping
event [MSR-2].

**Under R1 "every other" no longer includes a borrowed run**, because R1 withdraws
the parameter: a helper that transforms a run takes it by value [CALL-2], and a
helper that writes elements takes a view. The conservative kill therefore remains as
the default for every parameter type this design does not classify, and it is no
longer load-bearing for D1 — which is the point, because round 5 showed a kill can
be defeated by a fact published after it and by an action that is not a write, and a
design whose central defect is closed by a kill has one door per channel.

*Judgment:* the kill classification per parameter type. *Publishes:* the surviving
measures. *Amends:* nothing beyond [MSR-2]'s. *Depends:* [VIEW-4], the
length-fixedness this classification reads; [EFF-1] 1386 as [PROV-3] amends it,
without which a view parameter's projected write reaches the descriptor and not the
element storage this rule names. *Law:* L11. *History:* 6.9, R1; 6.8, F1 attack 9.

**[CALL-4] Contract vocabulary, the ordered result list, the per-variant route, and
where the relations land.** [FN-9]'s clause operands are terms [MSR-5], so `len(P)`,
`cap(P)`, `room(P)` and `head(P)` over an admitted formal place are operands with no
per-family admission. The same terms over an admitted **result** place are operands
too, which today's result-datum restriction to fragment integers forbids: probe `t5`
is the parse rejection and probe `t14` is the resolution rejection, so this is a
semantic addition at both levels.

```wf-design
fn collect['s](out: own Vector<'s, u8>, source: own Span<u8>)
    -> (rest: own Vector<'s, u8>, written: own u64)
    reads(out, source), writes(out) contract {
  requires len(source) <= room(out);
  ensures len(rest) >= len(out);
  ensures written <= len(rest);
} { ... }
```

The ordered result list is [S16] and the clause operands are [S17]. **No clause names
two states of one term, and under R1 no clause needs to.** A
parameter is an input and has exactly one state; a result is an output and has
exactly one state; a relation between them is single-state in both operands. There
is no `old()`, no frame rule, and no entry/exit convention to remember — the fifth
draft needed one because its helper mutated a caller's object, and R1 removed the
mutation rather than the convention. **This is also where L14's retired guarantee
comes back**: `ensures len(rest) >= len(out)` says the helper did not shorten what it
was handed, and it is an ordinary clause with no special machinery anywhere.

**A function may declare an ordered result tuple [S16]**, and each result binding is
a datum of every clause of that function, so one clause may name more than one:

```wf-design
fn render['s](block: own Vector<'s, u8>, task: &Task)
    -> (rest: own Vector<'s, u8>, written: own u64) ... contract {
  ensures written <= len(rest);
}

let (rest, task) = seq_take(vector: move pending);
```

**A relation is published per enum variant and per result ordinal, and a result
datum admits field projection [S24].** [FN-9] 1307 admits exactly one routed shape,
`when Ok(value: r):` for `own Result<T, E>` with `T` a fragment integer, and 1314
excludes a nested result projection outright. That is the narrowest useful surface a
contract could have, and round 5 measured what it costs: **no library constructor
can publish a fact about the run it built and no fallible helper can publish that it
succeeded.** `ring_new`'s `len(result.slots) >= n`, `pool_new`'s, `pool_take`'s
capacity, `bs_new`'s and `bs_reserve`'s are all unstatable, so 4.1's `render` and
`drain` requirements are undischarged at their call sites, 4.2's `collect` call is
undischarged, and every hosted append re-reads a measure and takes a runtime branch
that is statically true — which is the branch L4 promises `seq_place` does not have.
Probe `t6` is the rejection for an `Option` result and probe `t14` for a projected
result datum. The generalization is three sentences:

> A routed clause is admitted as `when V(f: r):` for **any variant `V` of any
> returned enum**, with `r` that clause's fresh symbolic payload datum. An unrouted
> clause is admitted for a written result of any **measured** type as well as any
> fragment integer. For a function with an ordered result list, a clause is routed
> to one **ordinal** by naming that ordinal's binder, and every ordinal is a datum
> of every clause. `len(P)`, `cap(P)`, `room(P)` and `head(P)` are operands for an
> admitted place `P` formed from a result datum with field and `deref` projections,
> on exactly the terms [FN-9] 1313 already grants a parameter datum, whenever `P`'s
> final selected type is measured.

The relation stays a comparison over two `u64` terms in every case; what widens is
which datums may occur in it, not what a relation is. That is what makes the
addition a domain widening rather than a second proof surface.

**Those relations reach the caller through one added [ENT-3.S12] destination
clause**, and without it a multi-result contract publishes nothing: 2833 fixes a
closed destination list of four, and a destructuring `let`, a `set` target list and a
plain `set` receiver are none of them. Round 5 showed the last of those breaking the
fifth draft's own program 4.2, where `set total = collect(...)` published nothing at
all and the statement the design calls "goal A's container half" was undischarged.
The clause is the single-binder route quantified over ordinals **and stated over
both writing forms**:

> Each binder of a destructuring `let`, and **each target of a `set` target list,
> including the single-target form**, is the S12 destination for every published
> relation naming the result at that ordinal, established after the call's ordinary
> transfer, consumes, borrow commits, target commit and kills in [ENT-5] 2898-2905's
> existing order, with `M(c,q)` requiring every other referenced support to be live
> at establishment.

The same clause serves [PROV-6]'s destructuring consume, whose binders are the
nominal's fields rather than a call's results, and [LIV-3]'s exchanged first target,
whose relation lands on the place rather than on a binder. Four forms, one clause,
which is what the register's sixth condition asks for. [FN-9] 1357's narrow
direct-set route is subsumed rather than joined: its extra premises exist only
because that route substitutes a receiver that is *also* an argument, and this clause
covers the case where it is not.

*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set, the
widened result shape and the widened route set. *Publishes:* the clause relations,
on every result ordinal and on every admitted variant route. *Amends:* [FN-9]
1301-1365 (measured and multi-ordinal results, per-variant routes, result field
projection, multi-datum clauses, and 1357's narrow route subsumed), [ENT-3.S12]
2833's destination list (one added clause, serving four forms), [GRAM-2]'s `fn_decl`
result shape, [GRAM-4]'s `let_stmt`, `set_stmt` and `return_stmt`, [FORM-2] 52-78's
rendering, and [FN-1] 1005-1019's result shape. *Verified today:* probes `t5` and
`t14` show a measured or projected result operand does not parse and does not
resolve, probe `t6` shows a variant route on an `Option` is `[FN-9]
InvalidPostconditionSelector`, and probe `t7` shows the multi-return signature does
not parse. *Law:* L10, L11, L16. *History:* 6.9, R1, F4 blocking 2, F3 defects 3
and 9.

**[CALL-5] No transport reads the actual's spelling.** The three transports above
are selected by the callee's declared parameter mode and type and by its declared
contract. No rule of this design consults the argument expression's shape, the
callee's body, its name, or any per-parameter summary derived from its body. A
parameter type for which no transport is selected kills conservatively.

**One rule of this design tested that sentence and now satisfies it.** [RES-8]'s
saturation flag was defined in the fifth draft as a property of "every acquisition
the function performs, transitively" and asserted in the same sentence that
[CALL-5] was respected. It was not: that quantifier ranges over the statements of a
body. [RES-8] now reads a **declared** row, which is what makes this sentence true
of the whole design rather than of its call rules alone.

*Judgment:* the conservative default for every unselected parameter type.
*Publishes:* the absence of a call-site-derived fact. *Amends:* [ENT-5] 2876's
clause (b), whose projected-callee-write kill is now classified by [CALL-1..3] and
by nothing else. *Law:* L11. *History:* 6.9, F2 F5-9.

#### 3.K.7 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

```text
| class              | members                                                                        |
|--------------------|--------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context, including every       |
|                    | frame-placed arena [PROV-5]; every extent-placed arena; every worker-lane       |
|                    | stack; allocator and runtime metadata; the release walk's frame-resident        |
|                    | scratch [RES-5]; the adapter's persistent buffers and mappings                  |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots;  |
|                    | the runtime's fixed handle table; every other runtime-owned store               |
| host objects       | every host object a qualified runtime holds for the program's duration: the     |
|                    | completion ring's own descriptor, an adapter's persistent mappings              |
```

The third class is round 5's. [RUN-2]'s profile row enumerated lanes, stacks, task
records, queue capacities, completion records and the handle table, and the shipping
adapter holds three `mmap`s and a file descriptor that are none of those, so `E` was
incomplete by construction for every hosted marked program. L6's answer is a shape,
not a byte total, so the covered set gains a class and [RES-2] gains an item.

*Judgment:* none; it fixes the domains [RES-3] quantifies over. *Publishes:* the
covered set. *Amends:* nothing. *Law:* L1, L5. *History:* 6.9, F2 F5-12.

**[RES-2] The envelope `E`, over the target's profile table.** `E = E(P, T, B)` is,
for one program `P`, one selected target and ABI `T` [STOR-6], **and one build
`B`**, a finite table with one row for each lane count `W` the target's runtime
supports. Each row is a finite list of shaped items:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count)                           interchangeable fixed-size records
stack(context, bytes)                        one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
handle(kind, count)                          host objects the runtime holds for the run
```

**`E` is a function of three things and not two**, because [STK-3] makes it an output
of code generation: two builds of one accepted program at two optimization levels
publish different rows, and a deployment sizes against the row it was given.

**And `E` carries the content digest of the artifact it describes.** Making `E` a
function of the build creates an obligation the fifth draft did not state: the row a
launcher commits must be the row of the binary it launches. Q9 puts `E` outside
[PROG-2] compilation-unit identity, which is right, and a table from an earlier
build therefore satisfies every check [RUN-4] performs while describing a different
artifact — an overflow in `Running`, which is 1.1's program that vanishes at three
in the morning arriving through the repair meant to remove it. `PreStart` refuses a
row whose digest does not match the module it is starting.

**Which items carry a source-stage figure is stated rather than quantified over.** A
`region` item's bytes and alignment, a `slots` item's count and a `handle` item's
count are [RES-5]'s target-independent arithmetic and are read by acceptance; each
additionally carries the target-stage exact figure. A `stack` item has **no
source-stage figure at all** ([STOR-6] 764, [STK-3]), so stage one's entire stack
content is premise 2 of [RES-3], acyclicity.

*Judgment:* `E` is well-formed only if every item's arithmetic was performed in
the unbounded mathematical domain and is representable on `T`, the same standard
[STOR-6] already applies. *Publishes:* `E` itself, as a compilation artifact, with
its digest. *Amends:* nothing. *Law:* L1, L6. *History:* 6.9, F2 F5-11 and F5-12;
6.8, F2 NB15.

**[RES-3] The judgment, in two stages.** For a program `P`,
`source-resource-closed(P)` holds exactly when, on the rewritten call graph
[STK-1], every premise below is established from program text alone:

```text
1  no reachable store is a Heap                                    [PROV-4, RES-4]
2  the call graph is acyclic                                       [STK-2]
3  every covered store's demand is bounded, per domain, by the
     symbolic composition of [RES-10]                              [RES-5, RES-10]
```

**A bound is a closed expression in compile-time constants, type-level constants and
runtime-profile symbols. A per-domain figure that names a runtime value is not a
bound**, and premise 3 fails at the loop or call that introduced it, with that value
named. That sentence is what makes stage two a substitution rather than a discovery.

**What premise 3 is for is stated.** It is a boundedness filter over the published
envelope: it decides whether a finite `E` exists and what its figures are. It is
*not* what stops an acquisition from over-drawing a store; that is the
per-acquisition obligation — a checked spelling refuses with a value [RES-6], a
proved spelling discharges under [MSR-4] — and it holds with or without the marker.
Both halves are needed and neither substitutes for the other.

For a selected target `T` and its runtime, `E-materializes(P, T, B)` holds when
every symbolic figure of stage one has a concrete value (frame sizes measured after
code generation [STK-3], strides and alignments [STOR-6], the runtime's own
profile rows [RUN-3]), every row of the resulting table is representable and is one
the runtime's published profile can carry [RUN-2]. Failure here is a
**target-qualification failure** under [STOR-6] and [QUAL-2]: it stops compilation,
cites no language rule, and is not a source rejection.

*Judgment:* stage one, per domain, over the checked program; deterministic,
terminating, and free of search, budget or timeout. *Publishes:* the property, and
`E`. *Amends:* [STOR-6] 738-770, whose "the language defines no numeric
per-function frame ceiling" sentence keeps its scope for the *language* and is
joined, for a resource-closed build, by a computed per-context envelope, and whose
target-stage obligations gain `E`-materialization. *Law:* L1, L8, L9. *History:*
6.8, F2 NB17.

**[RES-4] The entry requirement, the heap, and the deferrals it moves.** The entry
may carry the marker `resource_closed` **[S19]** before its `command` program-kind
marker:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
```

The marker changes no acceptance judgment: every program is judged by exactly the
same rules. It changes two things. It makes the failure of [RES-3] stage one a
hard error rather than a reported property. And it selects which [SCOPE-3]
deferrals apply: for a marked program, **stack exhaustion and covered-store
exhaustion are inside the model** — [STK-2] and [STK-3] make the maximum chain a
computed item of `E` and under an admitted run [RUN-5] it is unreachable — and for
every other program they stay deferred, as does the guard-page floor that reports
them, whose own alternate stack is, for a marked build, an item of `E`.

**One thing the marker no longer selects is whether a program may abort.** The fifth
draft made a cyclic containment graph a premise-3 denial, which is a hard error only
under this marker, so every unmarked program kept the release walk that aborts.
[PROV-6] now refuses that type in every program, and L3's last clause is true rather
than aspirational.

A program whose call graph reaches a `Heap<'s>` is not resource-closed, and a
`main` selecting `command.heap` is by itself the rejection. A bounded general store
is still a general store: an envelope item can promise bytes, and cannot promise
that the next contiguous aligned request has a home.

*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage one
is a hard error naming its own cause: the heap-reaching path, rendered from `main`
to the allocation and located at the offending `input_label` or the deepest `call`;
the call-graph cycle [STK-2]; or the unbounded store [RES-5]. *Publishes:* the
property as a compilation fact, and the scope of [SCOPE-3]'s deferrals. *Amends:*
[FN-7] 1217, which fixes main's marker set; [GRAM-2]'s `program_kind` production;
and [SCOPE-3] 27-31. *Law:* L1, L6. *History:* 6.9, F2 F5-7.

**[RES-5] Five algebras, and a domain is a store.** Every covered store presents
its state through [MSR-1]'s measures. Exactly five **algebras** are defined, and a
**domain** is one pair (algebra, store identity), where a store's identity is
[PROV-1]'s region for a program store and the profile's row name for a runtime
store. Nothing else is admitted, and a store outside this list contributes no
envelope item and denies [RES-3].

```text
| algebra                    | state         | acquire            | release        | serviceable when |
|----------------------------|---------------|--------------------|----------------|------------------|
| uniform slots              | len, cap      | len + 1            | len - 1, on    | room >= 1        |
|  (lane, task, queue,       |               |                    | the store's    |                  |
|   completion and handle    |               |                    | own release    |                  |
|   records of the runtime)  |               |                    | event [RES-9]  |                  |
| bump extent                | len bounded,  | len + advance<T>   | nothing; the   | room >= advance  |
|  (Arena<'s, bytes, align>) |  in bytes,    |                    | store resets   |                  |
|                            |  cap = bytes  |                    | with 's        |                  |
| general heap (Heap<'s>)    | -             | -                  | per run, by    | undecidable      |
|                            |               |                    | dispose        | from E [RES-4]   |
| static and frame placement | fixed offsets | none at run time   | none           | decided at       |
|                            |               |                    |                | compile time     |
| cleanup scratch            | depth         | +1 per containment | -1 per level   | depth <= the     |
|                            |               |  level entered     |  left          | type's height    |
```

**Domain is a store, not a kind**, and that is round 5's third BREAK. If a domain
were a kind, two arenas in one program would share one domain, their peaks would
add, one arena's reset would be invisible to the other's accounting, and
[RES-10]'s route (ii) — `peak(loop) = cap(store)` — would have no referent. If it is
a store, a store minted inside a loop body has a domain whose life is one iteration,
which is exactly what makes its reset a zero rather than a mystery. [RES-8]'s map
and its saturation flag are keyed by the same pair.

**The cleanup-scratch domain is frame-resident in the releasing context**, so its
contribution is a term of `Stack(f)` under [STK-3] and its source-stage `depth` is a
multiplier on a target-stage per-level figure derived from [BLK-1]'s storage column.
The fifth draft gave the domain a `depth` and [RES-2] no item shape that carries a
depth, and 4.1 folded it into `stack.entry` in prose that no rule authorized. Frame
residency is the only placement that keeps L6 true, because a scratch store with its
own extent would be a `region` item no source construct reserved.

**`advance<T>` is a closed expression, and the store's own alignment is what makes
it one.** The fifth draft wrote it as `round_up(len, align_ceiling(T)) - len +
size_ceiling(T)` and called that closed; it names `len`, the arena's runtime cursor,
which [RES-3] says is not a bound, so the exact form denied premise 3 at every take
and the domain L6 exists for had no bound under its own rule. The repair is to make
the cursor's invariant do the work instead of the formula:

> Every take advances the cursor by `round_up(size_ceiling(T) * count, align)`,
> where `align` is the **store's** own type constant, and both allocating rows
> require `align >= align_ceiling(T)` as a compile-time comparison of two constants.

The cursor is then a multiple of `align` at every program point, the padding at a
take is zero, the advance is a closed expression in exactly two type-level constants
and one written const, and the run's padding is charged **once** rather than per
element — which is the other half of the same finding, since a 16-aligned 16-byte
record charged per element cost a proved arena nearly half its extent. There is no
"otherwise" clause: a requirement and a fallback cannot both govern one premise.

*Judgment:* the composition of [RES-10] per domain. *Publishes:* per program point,
per domain, the store's `len` bound. *Amends:* [OP-9] 974-1001, whose `buffer_fits`
stays a representability predicate, whose ceiling table gains Appendix A.1's derived
rows, whose region-bearing exclusion is lifted, and which additionally fixes
`advance<T>`. *Law:* L3, L6, L16. *History:* 6.9, F2 F5-3, F5-6 and F5-16.

**[RES-6] Typed failure, and the two spellings.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume. **The kernel declares no failure
nominal**, because no kernel acquisition takes an affine input: a count is copy and
a provider is borrowed, so `Option<Vector<'s, T>>` carries everything a refusal has
to carry. A library operation that consumes an owner and may refuse declares its
own nominal over its own type; `CONTAINERS.md` §3.3 writes one and 3.L.2 explains
why the kernel does not.

Each covered-store acquisition with a measure comes in exactly two spellings, on the
model of `+` and `+checked`: a proved form admitted only when [MSR-4] discharges its
goal, and a checked form that is total. **The `Heap` has no proved form** (L6). A
store with measures publishes more: a refused `seq_arena` establishes
`room(arena) < advance<T>`, which is L8's second half and which A.2 states in the
same units [RES-5] does.

The runtime's handle table is a covered store, and its refusal joins the **existing**
`IoError` channel: `reserve_file` **gains** `own Result<FilePermit, IoError>` in
place of the total `own FilePermit` [SYS-2] 2261 declares today, and its `Err` edge
establishes `room(factory) == Z` when the class is `ResourceExhausted`. [SYS-7]'s
"the class is the sole portable semantic discriminator" is the reason no second
nominal is added.

**The cost of that change is measured on the right alternative** [S25]. The fifth
draft wrote "keeps", which is false, and then measured the cost of a *second error
nominal* — five broken `propagate` chains in `wfgrep.wf` — which is the cost of the
alternative it did not take. The cost of the change it did take is a `match` or a
`propagate` at every reservation: **eleven sites across five corpus programs**
(`wfgrep.wf` ×5, `dir_walk.wf`, `raw_deflate_boundary.wf`,
`completion_read_boundary.wf`, `completion_windows_capacity.wf`), none of them
inside a `propagate` chain today. The honest alternative is a **total**
`reserve_file` over a store whose capacity is proved — the proved spelling every
other covered store has — which costs nothing at the call sites and costs one header
invariant over `room(factory)` in a loop; 5.0 records it as unruled.

No covered-resource failure is a trap, an abort, a process exit, a retry, or a
promotion to a larger store, in the writer's code or in the runtime. The batch-0079
floor's `wf_resource_abort` site loses its **allocation-refusal** caller once
allocation returns a value, and loses its **release-walk** callers once [PROV-6]
refuses a cyclic containment graph outright; the doubling-overflow arm goes with the
worklist that needed it.

*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result` or an `Option`,
plus [MSR-4] discharge at the proved spelling. *Publishes:* the returned owner's
identity on the refusal edge, and the store's own refusal relation where the store
has measures. *Amends:* [SYS-2] 2261 and 2457, `reserve_file`'s outcome row, which
gains a recoverable failure outcome; [SYS-7] 2473-2487's closed class set, which is
**unchanged** and is the reason no nominal is added; the batch 0079 exhaustion floor
as stated above; and [SCOPE-3] 29, whose "heap exhaustion ... may stop execution at
the host boundary without a Whitefoot value" ceases to be true. *Law:* L3, L6, L8,
L16. *History:* 6.9, F2 F5-15; 6.8, F3 defect 4.

**[RES-7] What bare resource-closedness does not cover, and the one exclusion
test.** Disk space, the successful acquisition of a file, socket or other host
object not exclusively reserved before start, network reachability and throughput,
CPU time, deadlines, scheduler fairness, power, device health, host termination,
and OS quota revocation are outside [RES-1] and outside every judgment in this
file. They remain typed system outcomes where the operation defines one, and
environment conditions where it does not.

Which **operations** a marked program may not call is decided by a property, never
by a written list, and **that property is derived from data the [SYS-2] record
already carries**:

> Each [SYS-2] declaration record's *acquires from* column is derived, not written:
> **an operation acquires one submission record and one completion record exactly
> when its target contract is `may-suspend`**, because [SYS-2]'s own contract says a
> may-suspend operation has a logical record that exists before target handoff and a
> `wait-capacity` submission outcome that retains a bundle in the runtime; and
> `reserve_file` acquires one handle record [RES-9]. A system operation is
> unavailable in a resource-closed program exactly when a store it acquires from has
> **count zero in the selected row** of `E`.

Both halves are round 5's, and each replaces a sentence that was wrong. The column
was *written* and read `none` for all sixteen operations, on the premise that
[SYS-2] 2270 says "no system operation allocates" — a non sequitur, since the column's
subject is acquisition from a pre-established store and not allocation, and one the
same draft contradicted eight paragraphs later by giving `reserve_file` a record. It
is verifiably wrong: `compiler/src/backend/completion/linux_io_uring.c:425-450,
587-640` reserves an entry from a fixed `entry_capacity` table on **every**
submission, and `compiler/src/backend/completion/bridge.c:900-1240` routes `read_at`,
`write_once`, `open_file`, `open_read`, `open_directory`, `open_directory_source` and
`directory_next` through that path — seven operations plus `reserve_file`, against a
column that read `none` for all of them. And the test compared *presence*, which
[RUN-2]'s own construction makes universally true, so it could never fire; 4.1's
published envelope reads `slots completion.records count 0` while a marked program
calling `read_at` would have passed.

The column is why this is a **source** judgment. Reading [QUAL-1]'s semantic-ID
record instead makes acceptance a function of the linked implementation, which L1 and
[SCOPE-2] forbid; a qualified implementation needing an undeclared store fails
[QUAL-2] **qualification**, citing no language rule.

*Judgment:* a call to an operation the test excludes, from a marked program's call
graph, is a hard error citing RES-7 at the `call`, naming the store and its
zero-count row. *Publishes:* the boundary. *Amends:* [SYS-2] 2164-2307's declaration
records, which gain the derived column; [QUAL-2] 2369, whose qualification
obligations gain an undeclared-store failure; [ERR-4] 1484, whose "unavailable
external resources remain outside the source outcome model" gains the two families
[RES-6] and [RES-4] move inside. *Depends:* [SYS-2]'s may-suspend target contract,
which is the data the column is derived from. *Law:* L1. *History:* 6.9, F2 F5-5.

**[RES-8] The per-function summary is part of the callable boundary, in three
pieces, and every piece is declared.** Each function's boundary [FN-1] gains three
derived components:

- a **source-stage per-domain map** over that function's formal provider and
  measure terms, substitutable at a call site, keyed by (algebra, store) [RES-5];
- a **declared saturation fact per provider parameter**, written
  `saturating(p)` **[S26]** in the function's contract; and
- a **target-stage own-storage figure** covering every store it reserves [PROV-5]
  and its own frame.

**The saturation fact is declared and not derived, which is the whole repair.** The
fifth draft defined it as "whether every acquisition the function performs on that
domain, **transitively**, is one that cannot succeed when the store is full", and
asserted in the same sentence that it was "derived from the callee's declared rows
... and never from its body, so [CALL-5] is respected". That quantifier ranges over
the statements of a body and of every callee's body; no declared row says which
spelling an operation used; and [ENT-1] 2661 additionally forbids reading which
premise discharged a goal, which the fifth draft's clause "a proved spelling has it
when its goal comes from a header invariant" does directly. `saturating(p)` is an
ordinary contract fact checked against the body by exactly the both-ways discipline
[EFF-2] already applies to `allocates` — *this function performs no acquisition on
`p`'s store that could succeed when that store is full* — so a caller reads a
declaration, [CALL-5] is true in fact rather than by assertion, and the fact is keyed
by a provider place and hence by a store, which is what [RES-10]'s route (ii) needs.

A kernel row's own saturation is table data on the row: a **checked** acquisition
spelling is saturating and a **proved** one is not, because a proved acquisition is
one the caller has already bounded by its own [MSR-4] discharge.

The three components are separate because they belong to different stages, and
splitting them keeps [PROV-4]'s framing honest: a self-reserved store contributes to
the third, so [RES-10]'s call rule never meets a callee demand with no actual to
substitute. The map composes across the one closed compilation unit [PROG-1] and no
further.

*Judgment:* the both-ways check of each declared `saturating(p)` against the body,
citing RES-8 at the `contract_block`. *Publishes:* all three components. *Amends:*
[FN-1] 1005-1012's boundary list; [GRAM-2]'s `contract_block` (one added clause
form). *Depends:* [PROG-1] 1492, the one closed unit the composition claim is scoped
to; [ENT-1] 2661, whose "a retained witness changes diagnostic parent choice only,
never the derivable set or acceptance" is why proof provenance may not be read.
*Law:* L1, L5. *History:* 6.9, F2 F5-9 and F3 defect 14; 6.8, F2 NB10.

**[RES-9] The runtime's own stores, and a release event stated over the record.** A
covered store needs five things written in one place: a **capacity**, an **acquire
event**, a **release event**, a **refusal relation**, and a **multiplicity**. The
program's own stores have all five from [PROV-5], [BLK-2] and [MSR-2]. The runtime's
have them from the profile row and the operations that touch them, and the one a
marked program can actually reach — the handle table — needs three amendments that
no earlier draft made, because [SYS-2] and [SYS-10] together deny it.

[SYS-10] 2554-2558 **is amended.** Its sentence "Reserving it promises no native
descriptor, **handle-table entry**, kernel memory, or host quota" is replaced by:
*reserving a `FilePermit` consumes one record of a runtime store whose capacity the
target's profile publishes; host exhaustion at the open is a different condition and
remains the ordinary `ResourceExhausted` member of the open operation's typed
`IoError` result, outside `E`.* And its "This first slice never returns or recycles
the permit" is replaced by the release event below.

**The release event is stated over the record, not over three type names.** The
fifth draft wrote that the record returns "when the permit, or the `ReadFile`,
`DirectoryRead` or `DirectorySource` it became, is released" — an enumeration, and
round 5 found the arm it omits. [SYS-10] consumes a permit "on every success **or
recoverable-failure** outcome", so a driver polling for a device node acquires a
record per iteration, the open fails, the permit is consumed by the operation rather
than released, it became none of the three named holders, and the record never comes
back. After `cap(factory)` iterations the program can open nothing, with no
diagnostic and a published envelope that is correct about its own arithmetic. The
sentence that closes it is a property:

> A handle record returns when the value holding it is released, when it is consumed
> by an operation that produces no successor holder, or when the operation it
> authorized returns any outcome that produces no holder. For each covered runtime
> store, the set of acquire sites and the set of release sites must together cover
> every path of every operation that touches it, and a target that cannot exhibit
> that coverage fails [QUAL-2].

Stating it as a **closure obligation on the store** rather than as a checklist is
what stops the next open-like operation from extending the enumeration and the one
after that from being forgotten.

[SYS-2] 2301's closed proposition set is **amended too.** It says today that the only
system-result propositions available to source invariants are [SYS-9]'s enumerated
relations and the facts of selecting one typed outcome. The measure relations of a
covered system store join that enumeration as a named source; without it
`cap(factory)` dies at the first `reserve_file` and no marked program can open a file
in a loop. **And the release action publishes a relation, not only an effect row**:
[RES-10]'s `release one` transfer has a program event, so the fact state and the
arithmetic agree about one store instead of disagreeing about it.

**The multiplicity is one table per process.** The fifth draft listed multiplicity as
one of five parts and supplied four; under `W = 1` the omission is moot and under
Q5's lifted denial it is not, so it is stated: the handle table is per process, not
per context or per lane, and a successor that gives a lane its own table gives it its
own store identity [RES-5] with it.

**The release row's second subject** goes where every other release in this design is
made visible: in the release action's own effect row. [STOR-3] 709-712 already gives
a system resource type "one ordinary state-effect row" for its release action and
already substitutes a formal path for its table-local `owner` subject. A type whose
backing is a covered store names that store in its release row, so `ReadFile`'s
release exhibits `writes(owner)` **and** the runtime handle-table path, named by the
profile rather than by a formal.

Reclassifying `ReadFile` as linear was considered and refused: its release needs no
capability [PROV-6], so the criterion does not reach it, and marking it would put a
`dispose` on every close site in the corpus and retire the release-completeness
[SYS-5] 2397-2400 grants.

*Judgment:* none by itself; it supplies the fact sources [RES-5] and [RES-10] read,
and its failure is a runtime's [QUAL-2] qualification failure. *Publishes:* each
runtime store's capacity, acquire event, release event, refusal relation and
multiplicity. *Amends:* [SYS-10] 2554-2558 (the reservation's promise and the
permit's recycling), [SYS-2] 2301 (the closed proposition set), [STOR-3] 709-712 (the
release contract's second subject), and [SYS-5] 2397-2400's release-completeness,
which is **kept**. *Depends:* [QUAL-2] 2369, which is where a runtime that cannot
publish a capacity fails. *Law:* L1, L3, L5. *History:* 6.9, F2 F5-4 and F5-15;
6.8, F2 NB3, NB4.

**[RES-10] How `E` is composed.** This is the arithmetic every promise about `E` is
computed by. The fifth draft carried it as an unnumbered subsection, which is why it
gained a derived-release transfer and a per-discharge loop map without ever being
checked against §2.1's accounting sentence; round 5 broke it in three structural
places and five smaller ones. It is a rule here so that it states a judgment,
publishes a fact, names its law, and is read by the same conditions as everything
else.

Every covered resource is one of three kinds, and conflating them is the single
most common way to get a wrong answer (L9).

```text
| kind                 | question                          | examples                              | bound         |
|----------------------|-----------------------------------|---------------------------------------|---------------|
| reusable capacity    | how many are held at once?        | task and completion records, lanes,   | peak len      |
|                      |                                   | queue slots, handle records           |               |
| consumable budget    | how much is spent and not         | arena cursor bytes, a fixed           | net consumed  |
|                      | returned in this epoch?           | append-only log's records             |               |
| external effect flow | how many times did it happen?     | opens, writes, submissions            | not bounded,  |
|                      |                                   |                                       | not in E      |
```

**A statement's summary is one map from label to `(peak, delta)`, and the label set
has a member no edge carries.** The labels of a statement are its fallthrough, each
variant of a result it produces, each `break` label it may take, `propagate`, and
**`retained`**.

`retained` is round 5's rank-two repair. [STK-4] admits a loop no `break` resolves
to, which is what makes a kernel's idle loop and a driver's service loop entries at
all, and it promises that what such a loop holds is "visible in `E` rather than
invisible in the fact state". Under the fifth draft's label set that loop's exit-label
set is **empty**, its map has no entries, the sequence rule makes every enclosing
statement's map empty too, and every acquisition the loop performs — every arena
take, every lease, every handle record — reaches `E` as nothing at all. The
attribution [STK-4] offered recursed into a statement whose own fallthrough is
unreachable and terminated nowhere. The label is the fix, and it is not a patch for
one loop shape: it is the entry a steady state has, which is what a service loop and
a switched context both are.

```text
retained   what the statement holds that no edge of it will release: for a loop with
             no fallthrough, the body's own peak composed with its backedge delta
             discharged by the three routes below; for every other statement, the
             componentwise max of its children's retained entries
```

Per domain `r` [RES-5], the primitive transfers are fixed:

```text
acquire one       (peak 1, delta +1)     on the success exit; (0, 0) on a refusal exit
release one       (peak 0, delta -1)     at a dispose, or at a store's own release event
derived release   (peak 0, delta -1)     contributed by a scope-exit edge, per released value
reset a store     (peak 0, delta -len(store))
                                         contributed by the release action of a store whose
                                         [RES-5] algebra reclaims with its region [PROV-5]
move an owner     (peak 0, delta  0)     moving into a run acquires nothing
borrow an owner   (peak 0, delta  0)
```

**The reset transfer is the third structural repair.** A region block re-entered by a
loop is the design's own recommended idiom for per-iteration scratch, and under the
fifth draft's five transfers its exit edge fitted only `release one`, whose delta is
`-1`. A block that took 256 bytes therefore left `+255` on the backedge, `max(d) > 0`,
route (i) had no trip count, route (ii) asked whether a *bump* acquisition can succeed
on a full store, and route (iii) could not name a store minted inside the body — so
the design's canonical program was refused, or, with the checked spelling, was
bounded at a per-iteration peak mistaken for a lifetime peak. The reset's delta is not
a constant and does not have to be: it is the exact inverse of everything the block's
own map accumulated, so `delta(region_block) = 0` **falls out of the arithmetic**
instead of being asserted in prose, which is what NB18's answer needed and never had.

A delta may be an integer or an interval `[min, max]`. **An interval enters the
peak equation as its `max` and the delta equation as an interval, and every test
below reads its `max`.** The compositions are:

```text
sequence   when A has a fallthrough exit, for each label L of B:
             peak(A;B)[L]  = max( peak(A)[fallthrough], max(delta(A)[fallthrough]) + peak(B)[L] )
             delta(A;B)[L] = delta(A)[fallthrough] + delta(B)[L]     (interval sum)
           for each non-fallthrough label L of A, A;B carries A's own (peak, delta)[L]
           when A has no fallthrough exit, A;B is exactly A's map and B contributes nothing
           retained composes as the componentwise max of the two retained entries

branch     the union of the arms' maps, keyed by label; two arms reaching one
           label contribute the componentwise max of peak and, when their deltas
           differ, the interval [min, max] of delta; retained is the max

call       substitute the callee's source-stage map [RES-8] at the call site, with its
           formal measure and provider terms replaced by the actual ones, and read its
           declared saturation fact for the loop rule below; the callee's retained
           entry is carried unchanged

loop       let d be the backedge delta and p one iteration's peak.
             max(d) <= 0  -> peak(loop) = p; delta(loop) = d; no iteration bound is needed
             max(d) >  0  -> the loop is bounded on a domain exactly when the composed
               peak is a closed expression [RES-3], which it becomes exactly through one
               of three discharges, and the loop's own map is stated per discharge:
                 (i)  a compile-time constant trip count T:
                        peak(loop) = p + (T - 1) * max(d);  delta(loop) = T * d
                 (ii) a store whose cap is a standing fact [MSR-2] and for whose domain
                        every acquisition on the loop's paths is saturating, read from
                        the row and from each callee's declared saturating(p) [RES-8]:
                        peak(loop) = cap(store);  delta(loop) = [0, cap(store)]
                 (iii) a writer [INV-1] invariant over the measure terms:
                        peak(loop) = the invariant's own target;  delta(loop) likewise
               Otherwise there is no finite E and premise 3 fails here.
           a loop with no fallthrough carries no fallthrough entry and its retained entry
             is p composed with d discharged by the same three routes
           each other label of the loop carries the loop's own map, not the map of the
             edge that reaches it

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration peak p
                                        and retained d, and K the profile's window
```

*Judgment:* the composition itself, per domain, over the checked program;
deterministic and free of search. *Publishes:* per statement, per domain, one map
from label to `(peak, delta)`, `retained` included, which is what [RES-3] premise 3
quantifies over. *Amends:* nothing in v0.41; this is new machinery over [FN-1]'s
existing graph. *Depends:* [FN-1] 1076's conservative structural graph as [STK-4]
corrects it, which is where the label set comes from. *Law:* L1, L8, L9. *History:*
6.9, F2 F5-2, F5-3 and F5-8; 6.8, F2 NB8, NB9.

##### 3.K.7.1 Which stage decides what

```text
 1  tail-SCC rewrite, source premise [STK-1]        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-4]    source stage   compiler
 4  per-function source-stage demand map and
      declared saturation facts [RES-8]             source stage   compiler
 5  loop and branch composition [RES-10]            source stage   compiler
 6  concrete sizes, strides, static image           target stage   compiler
 7  per-context frame envelope [STK-3]              target stage   compiler, post-codegen
 8  runtime profile row for each supported W        target stage   runtime data
 9  par composition against the profile             target stage   compiler
10  assembling E, its digest, and emitting it       target stage   compiler
11  selecting W for this run                        PreStart       launcher
12  matching E's digest against the module          PreStart       launcher
13  committing every region, stack and handle item  PreStart       launcher
14  creating lanes and reaching the ready barrier   PreStart       runtime
15  initializing every adapter record and queue     PreStart       runtime
16  crossing SourceStart and invoking main          PreStart -> Running  runtime
```

Steps 1 to 5 decide whether the program is source-resource-closed, and are the
only steps a source rejection may cite. Steps 6 to 10 decide whether this build
qualifies. Steps 11 to 16 decide whether this run is admitted.

#### 3.K.8 `[STK]`: the stack

**[STK-1] A tail edge is one whose caller frame is dead, and the rewrite removes
the transfer.** For each strongly connected component of the call graph in which
every intra-component call edge is a tail edge, the compiler rewrites the component
into **one dispatcher function with one frame** before frames are measured. The
intra-component edges are then not calls at all: the dispatcher owns the frame, the
loop body assigns the next state, and there is no ABI transfer left to ask about.

The premise is a fact about ownership and loans and nothing else. An
intra-component edge is a tail edge exactly when, at that edge: no loan, borrow,
view, region or reborrow the caller introduced is live; no compiler-derived drop
remains to run after the call; no linear binding of the caller is still live
[PROV-6]; no `par` join is outstanding; and no place the caller declared is read by
any value live across the call.

**There is no separate target obligation**: an activation record and a frame are
target-stage objects ([STOR-6] 746, [STK-3]), and the fourth draft's obligation was
attached to a transfer its own rewrite removes.

Two costs are recorded rather than discovered. A component member that opens a
region for an `arena_frame` has a live region at the jump, so its edge is not a tail
edge and [STK-2] refuses the component: tail recursion and region-scoped scratch are
mutually exclusive, and a writer who needs both writes the loop. And under R2 the
drop clause bites harder in a hosted program than the fifth draft implied, because a
linear binding live at the jump is now common: R3's "two fallbacks" is closer to one
there, and the way back to two is a region block.

*Judgment:* per edge, from the ownership and loan state the checker already has;
no proof search. *Publishes:* an acyclic call graph, or a component that is still
cyclic, and the strongly connected components [PROV-5]'s activation refusal reads
**after** this rewrite. *Amends:* nothing; this is a lowering and not an admission
rule, so recursion stays permitted. *Verified today:* probes `f2b` and `f8_tailframe`
are mutual tail recursions carrying a live borrow of a caller local and are accepted,
so the premise refuses a shape the syntactic list admitted. *Law:* L7. *History:*
6.8, F2 NB14.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After
[STK-1], a program whose call graph still contains a cycle has no finite stack
envelope and is not resource-closed. A `requires` bound on a recursion parameter,
a proof that a recursion argument decreases, and every other depth certificate
are **not** admitted as a substitute.
*Judgment:* under [RES-4], a hard error citing STK-2 that renders the complete
cycle in call order and the restructuring `rewrite the recursion as a loop over
an explicit FixedVector work list, or make every recursive call a tail call whose
caller frame is dead at the jump`. *Publishes:* nothing. *Amends:* nothing.
*Depends:* [FN-6] 1211, whose permission of recursion is why a recursive program is
excluded from [RES-4] rather than rejected. *Law:* L7.

**[STK-3] The frame envelope, over the whole chain.** For each execution context,
the `stack` item of `E` is measured over the context's **whole chain**, from the
point at which the environment hands that context a stack to the point at which it
takes it back: process entry through `ProgramFinished` for the entry context.
`main`'s own chain is one segment of it, and the runtime's start-up trampoline, its
teardown, its drop glue, the release walk's frame-resident scratch [RES-5] and the
exhaustion floor's own frames are other segments. Within one segment,

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

**The entry context's stack is materialized, not read.** The fifth draft made it
"part of the deployment grant" that [RUN-4] compares; `compiler/src/backend/wf_floor.c:303-329`
shows the shipping floor *creating* it with `pthread_attr_setstacksize` at
`WF_FLOOR_STACK_BYTES` and silently falling back to the host thread on failure — so
the one item [RUN-5]'s theorem is most conditioned on was being downgraded without a
report. [RUN-4] creates it at the figure the row names and reports `StartFailed`
when it cannot. A **worker lane's** chain is measured the same way; [RUN-2] fixes
`W = 1` for every resource-closed build, so in this version there is exactly one.

`E` is an **output** of code generation, recomputed after every optimization, which
is why [RES-2] makes it a function of the build and carries its digest.
*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic discipline.
*Publishes:* one `stack(context, bytes)` item per context per profile row.
*Amends:* [STOR-6] 762-766. *Law:* L5, L6. *History:* 6.9, F2 F5-11; 6.8, F2 NB15.

**[STK-4] A loop with no resolved break has no normal successor.** [FN-1]'s
conservative structural graph gains exactly one sentence, and it replaces one:

> A `loop_stmt` has an edge to `normal_successor(loop_stmt)` **if and only if some
> `break_stmt` resolves to it.**

No second clause. A `return`, a `propagate` `Err` edge, and a `give` delivering
outside the loop are edges to the function-return sink or to an enclosing
construct, and were never edges to the loop's normal successor. Probe
`n3_propagate_loop` is the driver loop this admits and is `[FN-1]
FunctionFallthrough` today.

This is the rule that lets a kernel's idle loop and a driver's service loop be
entries at all, and **[RES-10]'s `retained` label is what makes their demand
visible.** The fifth draft promised that a linear binding live on a path reaching
only such a loop is "a retained item of the enclosing scope's map"; round 5 showed
that map had no entry that could hold it and that the attribution recursed
nowhere. What is true now is stated in one place: a scope whose exit edge is
unreachable carries no compiler-derived release and no [LIV-1] check, so such a
binding is not an error, and what the loop holds — that binding and every store
record its body acquired and did not release — is the `retained` entry of its own
map, which composes outward and reaches `E`. No reset runs on that absent edge
either, so nothing observes the retained store.
*Judgment:* [FN-1]'s existing reachability and fallthrough judgment over the
corrected edge set. *Publishes:* the graph, hence [RES-10]'s label set.
*Amends:* [FN-1] 1076. *Verified today:* probes `n2_idle` and `f3_forever` are
`[FN-1] FunctionFallthrough`. *Law:* L1, L9. *History:* 6.9, F2 F5-2; 6.8, F2 NB9.

#### 3.K.9 `[RUN]`: runtime closure and admission

**[RUN-1] The artifact, runtime closure, and where the no-permission obligation
lives.** For every judgment in this file the artifact is the writer's code, the
compiler-derived cleanup and drop glue, the monomorphized instances, the `par`
runtime, the allocator and its metadata, and the qualified target adapter:
everything the process runs between process entry and `ProgramFinished`.

A runtime qualified for resource-closed programs performs, after the `SourceStart`
barrier and until `ProgramFinished`, **no covered acquisition whatsoever**: no
allocator call for runtime-owned storage, no thread or helper creation, no stack,
queue, table or worklist growth, no lazy TLS or adapter initialization, no
first-use mapping, and no first-error formatting buffer. Every runtime record is
established before the barrier or is carved from a fixed store that is already an
item of `E`. Today's adapter does not meet that: `bridge.c:670` initializes under
`pthread_once` **inside the submit path**, which is a first-use mapping by name.
That is an honest [QUAL-2] failure of one implementation and not a defect of this
rule.

**The no-permission obligation is a build obligation, not a runtime one.** The
fourth draft put it in a rule, which [PAR-1] 1987 forbids; the fifth draft moved it
into this rule's qualification obligation, which filed it against a party that
cannot discharge it — [PAR-1] permission is a *compile-time* grant to overlap, and a
runtime's translation units contain no record of a permission decision to audit. It
is stated here in its correct form:

> The emitted module of a marked program contains no `par` construct. That is a
> property of `T(P)` under [QUAL-2]'s qualification of the toolchain, checkable in
> the artifact by the party that produced it.

The obligation is soundness-critical and the hazard is executed: the current
runtime's wait path runs a stolen task on the waiting lane's own stack, so
`stack(lane_i)` as [STK-3] computes it is wrong by a factor bounded only by the
outstanding-task count.

**Acquisition and admission control are different obligations.** A qualified runtime
must additionally have, per store, a **bounded admission discipline** whose bound is
that store's published capacity: it declines work for which no record is available
and resumes when one is, acquiring nothing. What stays forbidden is **inline
execution**, which nests a task's chain inside a lane's current activation and which
no term of [STK-3] counts, and **unbounded waiting** on a store no other frame will
release. A runtime that cannot publish a bounded capacity does not support the
marker.

*Judgment:* a target- and build-qualification obligation, auditable from the emitted
module and the runtime's own translation units; its failure is a [QUAL-2]
qualification failure, not a source rejection, and no source construct can weaken or
waive it. *Publishes:* the runtime's own items and capacities. *Amends:* [SYS-2]
2270's "no system operation allocates", which is kept and given its companion: an
adapter record and a handle-table record are runtime-owned stores of [RES-1] with
published capacities; [QUAL-2] 2369, which gains the emitted-module property.
*Depends:* [PAR-1] 1987, whose unobservability sentence is why the no-permission
obligation is not a rule. *Law:* L3, L5. *History:* 6.9, F2 F5-12; 6.8, F2 NB12.

**[RUN-2] `par` enters `E` as an open profile, and a marked build publishes
`lanes(1)`.** For each supported lane count `W`, the runtime publishes one finite
profile row. **The row is open, not enumerated**: it publishes one figure per item of
[RES-1] that the runtime owns, enumerated by the runtime and not by this rule, so an
adapter mapping, an alternate stack and a completion ring's descriptor each get an
item shape [RES-2]. A rule that enumerated what a profile contains would be wrong at
the next adapter, and round 5 showed the fifth draft's enumeration already wrong at
this one. The number of iterations of a `par`-permitted loop never appears in `E`.

What this rule keeps is exactly what **is** a function of program text: **the
profile row a marked build publishes is the `W = 1` row.** Two consequences follow
for free: [PAR-3]'s replicated places, which are execution memory no envelope item
counts, cannot occur in a marked build; and [STK-3]'s worker-lane chain, though now
defined (1.5), has exactly one instance to measure.
*Judgment:* a fixed-arithmetic composition ([RES-10]'s `par` rule) against each
profile row for an unmarked program, plus the published-row rule on a marked one;
the compiler emits no per-`W` clone. *Publishes:* the `lanes`, `slots` and `handle`
items of each row. *Amends:* the sentence common to [PAR-1] 1995, [PAR-2] 2030 and
[PAR-3] 2055, "exhaustion of the execution resources an implementation spends on
overlapping is a resource condition under [SCOPE-3] and is not an observable of this
rule": for a program resource-closed on this target that exhaustion is unreachable,
because no `par` construct is emitted [RUN-1]. *Law:* L5, L9. *History:* 6.9, F2
F5-12.

**[RUN-3] The parallel footprint of an allocation is its provider place, of a view
its origin range, and 1975's intervening list is a property.** In [PAR-1]'s
written-footprint clause, "the caller region each `allocates(arena 'r)` entry names
after region substitution" is replaced by "the places each `allocates` path reaches
under the [EFF-2] call-boundary projection", the same projection the rule already
applies to `reads` and `writes`. Two statements that allocate from one provider
therefore conflict, and two that allocate from distinct providers do not. With
[PROV-6] the same is true of two statements that only dispose.

[PAR-2]'s permission for a fill through a `MutSpan` needs two amendments. The
**loan** condition is stated over **iteration-formed** loans: every exclusive loan
formed by a statement of `B` is rooted in a binding `B` introduces, and a loan formed
before `L` on a root every footprint of `B` reaches only through 2005-2008's refined
single-element ranges does not deny. And the **write footprint** of `set m[at] = v;`
contains its origin at range `[a*at+b, a*at+b+1)` rather than at whole place
([PROV-3] use 1), which is what [PAR-2]'s standing condition needs.

**The admitted intervening-statement list of [PAR-1] 1975 becomes the property it is
reaching for.** The fifth draft added `dispose` to the enumeration and asserted that
`dispose` was "the only addition the two new forms need"; this draft adds three
forms, not two — `dispose` [S12], the destructuring consume [S13] and the
destructuring `let` of a multi-result call [S16] — and the last of those is 4.1's own
canonical statement, so a writer's choice between `set (p, x) = f(...)` and
`let (p, x) = f(...)` would decide whether the surrounding loop can be overlapped.
An enumeration that needs an amendment per grammar addition is the wrong shape:

> An intervening statement denies permission exactly when its footprint conflicts
> with a member's under the conditions this rule already states, and not otherwise.

Every new statement form then arrives permitted or denied by its own footprint.
*Judgment:* the existing [PAR-1] and [PAR-2] permission judgments, with one fewer
special case, one added loan clause, ranged origins, and an intervening rule stated
over footprints. *Publishes:* permission. *Amends:* [PAR-1] 1975 and 1981, [PAR-2]
2000-2034, and [PAR-3] 2035-2061 through their "forms every footprint exactly as
[PAR-1] forms one" clauses. *Depends:* [PAR-2] 2005's single-binder affine
element-write refinement, which is the disjointness argument the range clause
composes with. *Law:* L2, L5, L10. *History:* 6.9, F2 F5-14; 6.8, F2 NB16.

**[RUN-4] The startup protocol.** Program start has four points, and the covered
guarantee spans the last three:

```text
PreStart
    select a row of E from the target's profile table, largest supported W first
    refuse a row whose digest does not match the module being started [RES-2]
    materialize every item of that row:
        commit each region (committed backing, not a reserved address range)
        create each stack, the entry context's own included, at the row's figure
        commit each handle item
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

**The descent is honest about the marked case.** A marked build publishes exactly
one row [RUN-2], so there is no next smaller row and the failure of any item is
`StartFailed` on the first attempt. That is the right behaviour and the fifth draft's
protocol described a resilience it does not have there; the descent is real for an
unmarked program, which carries no `E` promise anyway.

*Judgment:* a target obligation, not a source judgment. *Publishes:* the selected
row. *Amends:* [PROG-3] 1505-1515, whose start-time obligation gains the
materialization of `E`, the digest match and the entry-stack creation, and whose
`ProgramFinished` boundary is now named. *Law:* L1, L5. *History:* 6.9, F2 F5-11.

**[RUN-5] Admission, and the theorem.** `Admitted(H, row)` holds when an
environment `H` has actually established a grant implementing every item of the
selected row before the barrier, the entry context's stack included, and, for the
duration of the run, does not revoke it and permits no unmodelled competitor to
consume from it. Then:

```text
source-resource-closed(P)  and  E-materializes(P, T, B)  and  Admitted(H, row)
------------------------------------------------------------------------------
no covered-resource exhaustion in run(H, T(P))
```

*Judgment:* none by the compiler. *Publishes:* the deployment contract, which is
the selected row. *Amends:* nothing. *Law:* L1. *History:* 6.9, F3 I22, the corrupt
fence.

#### 3.K.10 One name per concept

Every spelling in this table is a **proposal** [3.S], not a decision.

```text
| concept                    | proposed              | why                                                     |
|----------------------------|-----------------------|---------------------------------------------------------|
| a run of slots, frame-      | FixedVector<T, n>     | the settled name; its capacity is in its type because   |
|   resident [S2]            |                       | layout needs it before the run exists                   |
| a run of slots, store-      | Vector<'s, T>         | one type at two regions; its capacity is a measure      |
|   resident [S1]            |                       | because a growth policy must change it                  |
| the store's handle [S3, S4]| Heap<'s>, Arena<..>   | a value you must hold in order to act; the parameter    |
|                            |                       | is never elided, because it is the allocation fact      |
| the brand's spelling       | written iff the       | 3.K.0's assumption: decidable from the declaration text |
|                            | operands do not       | alone. The entry heap is written only where a signature |
|                            | determine it          | must name it                                            |
| build an empty run [S7]    | seq_fixed, seq_arena, | the placement is in the name, because it decides which  |
|                            | seq_heap              | item of E the run becomes (L6)                          |
| reserve a bump store [S9]  | arena_frame,          | as above; nothing else reserves                         |
|                            | arena_extent          |                                                         |
| append at either end [S8]  | seq_place,            | one name per end, whatever the backing                  |
|                            | seq_place_front       |                                                         |
| remove at either end [S8]  | seq_take,             | the window is two-sided, so L12's last clause is true   |
|                            | seq_take_front        |                                                         |
| read a measure [S11]       | len, cap, room, head  | one quantity, one name, term and reader alike           |
| a read-only view [S5]      | Span<'r, T>           | the rename is the whole change to slice<'r, T>          |
| a writable view [S6]       | MutSpan<'r, T>        | element writes only; its length is fixed by its type    |
| destroy a store-backed     | dispose p using (..); | one statement, closed under containment as linearity is |
|   value [S12]              |                       |                                                         |
| take a value apart [S13]   | let N(f: a) = move v; | the inverse of construct, so the closure covers         |
|                            |                       | disassembly too                                         |
| oblige a value to be       | linear struct N {..}  | for a logical obligation only; the storage obligation   |
|   consumed [S18]           |                       | is derived from the type                                |
| transform a place through   | set p = f(q: move p); | one admission on the one assignment form; the only form |
|   a call [S15]             |                       | the partition test could not write in wf                |
| rebind a consumed binding   | set p = e;            | the premise is deadness; the language gains no second   |
|                            |                       | assignment form                                         |
| a refusal                   | Option<T>             | the kernel consumes no affine input, so it declares no  |
|                            |                       | failure nominal; a library one declares its own         |
| the property [S19]          | resource_closed       | the long spelling is the one in use                     |
| the failure variant field   | Err(error: e)         | [PRE-1] declares Err(error: E)                          |
```

`FixedRing`, `PoolVector`, `HeapVector`, `ArenaVector`, `AppendView`, `absorb`,
`update`, `seq_frame`, `seq_exchange`, `HeapBox`, `ArenaBox`, `PoolSlot`,
`heap_take`, `arena_take`, `pool_take` as a kernel row, `Full<T>`, `TooSmall`,
`OutOfMemory`, `PoolExhausted`, `NeedCapacity` and `NoRecord` are **not** proposed
and are not in the kernel vocabulary. The first four are library names for kernel
types (3.L.1); `update` is [LIV-3]; `seq_frame` and `seq_exchange` are this draft's
two removals (3.K.3); the three box and slot names are runs of capacity one or
library nominals; the `*_take` operation names belong to the library's own functions;
and the last six are library nominals a writer declares over their own type.

#### 3.K.11 Amendment register

**This register is a collation of the `Amends:` and `Depends:` lines of every rule
in 3.K, and it carries nothing else.** It was written last, from the rules. It
covers 3.K only: 3.L amends nothing, because it is ordinary wf, and 3.S proposes
spellings rather than amending rules.

Six conditions make it checkable rather than remembered, and each is a defect of
this file when it fails:

1. a changed row whose `by` column names no rule whose `Amends:` line reaches it;
2. an `Amends:` line no changed row carries;
3. a `Depends:` line no third-list row carries, or a third-list row no `Depends:`
   line produces;
4. **(a)** a `Depends:` citation whose sentence lies inside a range some `Amends:`
   line changes; and **(b)** a `Depends:` citation, or any sentence in the depended
   rule, whose subject type, operation spelling, or effect atom any `Amends:` line
   in this file renames, retires or redefines. When a dependency really does fall
   inside changed text, or names a retired subject, it is recorded **on the changed
   row**, which states that the depended sentence survives and who depends on it;
5. **an `Amends:` line must state a change for every sentence in its cited range
   that the amending rule's own body contradicts.** Round 4 found the fourth draft
   leaving [ENT-5] 2893's element-position carve-out unmentioned; round 5 found the
   fifth draft doing the same to [SET-2]'s "it establishes no fact", at the rule the
   previous round's finding had created. Both are now stated on their rows; and
6. every `*Publishes:* X on Y` names the [ENT-3] destination clause that puts X on
   Y. A `Publishes:` line with no destination is the same defect as an `Amends:`
   line with no row. Four forms publish onto a destination in this design and one
   clause serves all four: [CALL-4]'s destructuring `let`, its `set` target list
   including the single-target form, [LIV-3]'s exchanged first target, and
   [PROV-6]'s destructuring consume.

**Changed.** Line numbers are `spec/kernel-spec.md` **v0.41** at 30602914, derived
in this session by remapping every v0.40 citation through the version diff and then
re-reading each cited sentence. Round 5 found eight wrong numbers and seven ranges
overshooting into a blank line or a heading in the fifth draft; every range below
ends on the rule's last nonblank line, and the eight are corrected in place —
[SYS-2]'s closed proposition set is **2283-2285** and not 2295+6, [PAR-1]'s
intervening list is **1975** and not 1990+6, [PAR-1]'s unobservability sentence is
**1987** and not 2102+6 (which is inside [PRE-1]), [SYS-5]'s two subjects are
**2397-2400** and **2407-2428** and not 2560/2575+6, [FN-7]'s canonical byte
sequence is **1245-1246** and not 1246+6, [OWN-5]'s move-through-borrow prohibition
is **591** and not 614+6 (which is inside [OWN-6]), and [INV-1]'s atom admission is
**3109-3113** while 3105 is the relation restriction. Each row's `by` column names
the rules whose `Amends:` lines reach it; a row that also records a surviving
depended sentence marks it **bold** (condition 4).

```text
| rule            | line      | change                                                          | by                          |
|-----------------|-----------|-----------------------------------------------------------------|-----------------------------|
| [SCOPE-3]       | 27-31     | heap exhaustion leaves the deferred set; stack and covered-store | [RES-4], [RES-6], [RUN-2]   |
|                 |           | exhaustion leave it for marked programs                          |                             |
| [FORM-2]        | 52-78     | +5 renderings: result list, destructuring let and consume, set   | [CALL-4], [LIV-3], [PROV-6] |
|                 |           | target list, dispose, the linear modifier                        |                             |
| [GRAM-2]        | 168-203   | result list; resource_closed; region_params on nominals; the     | [CALL-4], [RES-4], [BLK-4], |
|                 |           | linear modifier; a saturating clause; requires/ensures (185-186) | [MSR-5], [PROV-6], [RES-8]  |
|                 |           | take a clause_expr                                               |                             |
| [GRAM-3]        | 207-210   | slice/buffer/box/arena productions retire; runs and views are    | [PROV-1]                    |
|                 |           | ordinary TYPEIDs with targs                                      |                             |
| [GRAM-4]        | 217-256   | destructuring let and consume; comma return; set target list;    | [CALL-4], [LIV-3], [MSR-5], |
|                 |           | affine_factor GAINS terms; stmt gains dispose                    | [PROV-6]                    |
| [GRAM-5]        | 269-270   | +clause_expr; atom and atom_list untouched                       | [MSR-5]                     |
| [GRAM-9]        | 329-333   | unchanged; named because [MSR-5] moves the amendment away        | [MSR-5]                     |
| [GRAM-11]       | 346-350   | a fourth callee class in all three sentences                     | [BLK-0]                     |
| [TYPE-2]        | 357       | +6 nominals, slice renamed Span, box/arena/buffer retire; the    | [PROV-1], [BLK-1], [BLK-2], |
|                 |           | flat-element restriction is not inherited                        | [VIEW-1]                    |
| [TYPE-5]        | 374       | the written-argument criterion covers a fourth callee class and  | [BLK-0]                     |
|                 |           | becomes per-argument. **379 survives and [PROV-1] depends on it**|                             |
| [TYPE-6]        | 396-407   | the domain's spellings, nominals and region parameters; 401's    | [BLK-0], [MSR-6]            |
|                 |           | callee IDENT admission; 401's pbase gains a const generic        |                             |
| [TYPE-7]        | 476       | the deref domain becomes the two borrow modes alone              | [PROV-1]                    |
| [SET-1]         | 481-505   | loan-strength target traversal; one deadness-at-commit premise;  | [PROV-3], [LIV-2], [LIV-3]  |
|                 |           | one added admission, the in-place exchange                       |                             |
| [SET-2]         | 513-529   | region-bearing rejection replaced by [PROV-3] use 3 and          | [PROV-3], [LIV-3], [VIEW-4] |
|                 |           | [VIEW-4]; a compiler-derived exchange whose replacement is the   |                             |
|                 |           | read-out; **529's "it establishes no fact" becomes false for     |                             |
|                 |           | that exchange, whose relations land through the S12 clause**     |                             |
| [CONST-2]       | 552-556   | its naming of buffer, slice and slice_of follows the retirements | [VIEW-1]                    |
| [OWN-1]         | 563-571   | 563-564 UNCHANGED; linear refines affine; 569 gains the          | [PROV-6], [VIEW-1],         |
|                 |           | partial-consume refusal and dispose as a consuming use; 566-567  | [LIV-1], [LIV-2]            |
|                 |           | is KEPT                                                          |                             |
| [OWN-4]         | 582       | the lent-onward child's loan ends at its receiving statement     | [PROV-7]                    |
| [OWN-5]         | 594-611   | origins generalize to loan-bearing values and carry a range; two | [PROV-3], [VIEW-2]          |
|                 |           | ranged access clauses; the address-computation freeze; 601 and   |                             |
|                 |           | 608 restated. **606 survives and [VIEW-2] depends on it; 591 is  |                             |
|                 |           | outside this range and [LIV-3] and [PROV-6] depend on it**       |                             |
| [OWN-6]         | 616       | a child reborrow may name a caller-supplied region under the     | [PROV-7]                    |
|                 |           | result-type condition, for every reborrow. **614 survives and    |                             |
|                 |           | [PROV-2] and [VIEW-2] depend on it**                             |                             |
| [OWN-7]         | 630       | overlap extends to ranges. **630's subscript conservatism        | [PROV-3]                    |
|                 |           | survives and [PROV-3] use 2 depends on it** (4a)                 |                             |
| [OWN-10]        | 641-645   | 643's arena content clause becomes one over Vector content.      | [PROV-1]                    |
|                 |           | **641 survives and [PROV-2] depends on it** (4a and 4b)          |                             |
| [OWN-11]        | 646       | the move prohibition is replaced by [LIV-1]'s join agreement,    | [LIV-1]                     |
|                 |           | stated in [LIV-1]'s own body                                     |                             |
| [STOR-1]        | 675-682   | the runs join the storage table; the writable-place partition    | [LIV-2], [LIV-3], [PROV-1]  |
|                 |           | (678-679) gains the consuming-right-hand-side form; 681's        |                             |
|                 |           | growable paragraph and 682's arena-index-pool and keyed-         |                             |
|                 |           | collection rejections are superseded by the library, which       |                             |
|                 |           | recycles values without recycling slots                          |                             |
| [STOR-2]        | 685       | box_new and arena_new retire; a store take is a kernel row       | [PROV-2]                    |
| [STOR-3]        | 688-720   | a linear type has no derived release; the box and buffer HEAP    | [PROV-5], [PROV-6], [RES-9] |
|                 |           | rows retire, so derived release covers exactly region-end and    |                             |
|                 |           | frame reclamation and the system-resource release; the store     |                             |
|                 |           | reset joins the table; 709-712 gains a second subject.           |                             |
|                 |           | **699-705's drop order survives and [PROV-6] reuses it** (4a)    |                             |
| [STOR-4]        | 721       | confinement becomes the outlives relation over the region set    | [BLK-4]                     |
| [STOR-5]        | 723-737   | the position list becomes the three-way intensional split; the   | [BLK-4], [PROV-2]           |
|                 |           | per-leaf-provenance deferral is withdrawn as unnecessary         |                             |
| [STOR-6]        | 738-770   | E-materialization joins the target-stage obligations; 762-766's  | [RES-3], [STK-3]            |
|                 |           | frame sentences gain the per-context envelope                    |                             |
| [OP-1]          | 771-850   | +cap, +room, +head, pure, over runs, views and providers; five   | [PROV-2], [BLK-0], [BLK-2], |
|                 |           | constructors retire; ReservedLowerNames +3; 838 gains the class  | [VIEW-1]                    |
| [OP-4]          | 914-920   | indexable bases extend; the obligation is against len; a         | [BLK-1], [MSR-1]            |
|                 |           | subscripted measure place in an erased clause discharges at its  |                             |
|                 |           | own attach site                                                  |                             |
| [OP-5]          | 926       | "and contract predicate" narrows to a source condition           | [MSR-5]                     |
| [OP-7]          | 940       | slice_of retires; cap, room and head join the structural         | [VIEW-1]                    |
|                 |           | operations                                                       |                             |
| [OP-9]          | 974-1001  | the ceiling table gains Appendix A.1's derived rows, the region- | [RES-5]                     |
|                 |           | bearing exclusion is lifted, and advance<T> is fixed here        |                             |
| [FN-1]          | 1005-1076 | the view ceiling and its duplicate-result rejection; an ordered  | [VIEW-6], [CALL-4],         |
|                 |           | result list; three boundary components; a loop_stmt's normal-    | [RES-8], [STK-4]            |
|                 |           | successor edge (1076). **1041-1047 survives and [PROV-3] depends |                             |
|                 |           | on it; 1076 as corrected is what [RES-10]'s label set reads**    |                             |
| [FN-2]          | 1093      | the rejection narrows to loan-bearing and provider arguments;    | [BLK-4], [BLK-0]            |
|                 |           | explicit instantiation covers nominals and the kernel domain     |                             |
| [FN-3]          | 1123-1127 | the allocation component becomes the set of allocates paths      | [PROV-4]                    |
| [FN-7]          | 1216-1252 | command.heap; resource_closed; exactly one region parameter;     | [PROV-1], [RES-4]           |
|                 |           | allocates over a labelled input; 1245-1246's byte sequence       |                             |
|                 |           | gains the row                                                    |                             |
| [FN-8]          | 1262-1267 | clause operands are a clause_expr; 1267 becomes a GoalTemplate-  | [MSR-5]                     |
|                 |           | formation sentence. **1275 survives and [MSR-3] depends on it**  |                             |
| [FN-9]          | 1301-1365 | terms as operands; measured and multi-ordinal results; per-      | [MSR-3], [MSR-4], [MSR-5],  |
|                 |           | variant routes; result field projection; multi-datum clauses;    | [CALL-4]                    |
|                 |           | the entry datum replaces 1316; 1345's M(c,q) admits a datum;     |                             |
|                 |           | 1357's narrow direct-set route is subsumed by the S12 clause;    |                             |
|                 |           | 1312's closed compare_op set is what [MSR-5] reuses              |                             |
| [EFF-1]         | 1369-1389 | allocates takes formal-rooted paths; heap and arena retire;      | [PROV-4], [PROV-3]          |
|                 |           | 1386 generalizes to a loan-bearing parameter, which [CALL-3]     |                             |
|                 |           | and [VIEW-7] depend on (4a); **1389's both-categories sentence   |                             |
|                 |           | survives and [PROV-4]'s allocating row reads it**                |                             |
| [EFF-2]         | 1392-1440 | the slice projection generalizes; 1427 stays TRUE for the        | [PROV-3], [PROV-6]          |
|                 |           | actions that survive and is joined by the disposal walk's        |                             |
|                 |           | contribution and the exchange's read and write                   |                             |
| [ERR-3]         | 1472-1483 | the retained judgments gain the live-linear-binding refusal      | [PROV-6]                    |
| [ERR-4]         | 1484      | the deferral gains the two families that move inside. **1487     | [RES-7]                     |
|                 |           | survives and [PROV-5] depends on it**                            |                             |
| [PROG-3]        | 1505-1515 | PreStart materializes E, matches its digest and creates the      | [RUN-4]                     |
|                 |           | entry stack; ProgramFinished is named                            |                             |
| [DIAG-1]        | 1693-1718 | rank 5 covers the kernel domain; +container_declaration_ordinal  | [BLK-0]                     |
| [PAR-1]         | 1975,1981,| the provider-place projection; the intervening-statement list    | [RUN-3], [RUN-2], [PROV-6]  |
|                 | 1995      | (1975) becomes a footprint property; dispose enters a footprint; |                             |
|                 |           | 1995's exhaustion sentence is unreachable when marked. **1987    |                             |
|                 |           | survives and [RUN-1] depends on it**                             |                             |
| [PAR-2]         | 2000-2034 | iteration-formed loans; a view's ranged write footprint; the     | [RUN-3], [RUN-2]            |
|                 |           | element-write form. **2005 survives and [RUN-3] depends on it**  |                             |
| [PAR-3]         | 2035-2061 | the exhaustion sentence; replicated places cannot occur marked   | [RUN-3], [RUN-2]            |
| [SYS-1]         | 2136      | a fourth admitted declaration source                             | [BLK-0]                     |
| [SYS-2]         | 2164-2307 | views at the range-bearing operations; a derived "acquires from" | [VIEW-7], [RUN-1], [RES-6], |
|                 |           | column; 2261's reserve_file gains a recoverable outcome; 2283-   | [RES-7], [RES-9]            |
|                 |           | 2285's proposition set gains covered-store measures. **2270 is   |                             |
|                 |           | kept and [RUN-1] reads it; the may-suspend target contract       |                             |
|                 |           | survives and [RES-7]'s column is derived from it**               |                             |
| [SYS-3]         | 2309      | the kernel domain is admitted to every unit                      | [BLK-0]                     |
| [SYS-5]         | 2397-2400,| release-completeness is KEPT; the release action (2407-2428)     | [RES-9]                     |
|                 | 2407-2428 | gains the handle-table subject                                   |                             |
| [SYS-7]         | 2473-2487 | the class set is UNCHANGED, which is why no nominal is added     | [RES-6]                     |
| [SYS-8]         | 2488-2527 | the seven range-bearing operations take MutSpan and Span         | [VIEW-7]                    |
| [SYS-9,11,12,14]| 2529-2638 | their prose naming buffer<u8> is restated over views             | [VIEW-7]                    |
| [SYS-10]        | 2554-2558 | a reservation consumes a runtime record with a published         | [RES-9]                     |
|                 |           | capacity, and the record returns by a stated closure             |                             |
| [QUAL-2]        | 2369      | +two failures: an implementation needing an undeclared store,    | [RES-7], [RUN-1]            |
|                 |           | and an emitted module containing par in a marked program.        |                             |
|                 |           | **2369's own sentence survives and [RES-9] depends on it** (4a)  |                             |
| [ENT-2]         | 2681,2683,| measure terms over a subscriptable place; +the measure datum;    | [MSR-1], [MSR-3], [LIV-2],  |
|                 | 2685-2687,| a reinitializing set is a declaration event; a const generic is  | [MSR-2], [MSR-6]            |
|                 | 2728      | admitted at an endpoint; +standing facts. **2681 clause (c) and  |                             |
|                 |           | 2693 survive and [MSR-6] and [MSR-3] depend on them**            |                             |
| [ENT-3]         | 2730,2774,| +S13 and its arm route; S5 gains the construct placement; S6     | [BLK-0], [MSR-3], [CALL-4], |
|                 | 2785,2833 | generalizes over four measures; S12 gains one clause serving     | [LIV-3]                     |
|                 |           | four forms                                                       |                             |
| [ENT-5]         | 2863-2905 | descriptor-storage support; the effect-row kill; 2893(a) LOSES   | [MSR-2], [MSR-3], [CALL-5]  |
|                 |           | its element-position carve-out; the datum replaces the call-     |                             |
|                 |           | boundary and 2887-2891 paragraphs; clause (b) is classified by   |                             |
|                 |           | [CALL-1..3]. **2942-2946 survives and [MSR-2] and [MSR-3]        |                             |
|                 |           | depend on it**                                                   |                             |
| [ENT-6]         | 2976-3098 | one goal disposition; measures carry images; 3007 gains          | [MSR-3], [MSR-4], [MSR-2]   |
|                 |           | len + room = cap as two members; 3040/3047/3075/3084 keep their  |                             |
|                 |           | normalization and lose their route grant. **3019 and 3026        |                             |
|                 |           | survive UNWIDENED, which is why [BLK-0] and not [MSR-4] carries  |                             |
|                 |           | round 5's arithmetic repair**                                    |                             |
| [INV-1]         | 3101-3157 | 3105's relation restriction is reused by [MSR-5]; 3109-3113's    | [MSR-3], [MSR-5], [MSR-6]   |
|                 |           | atom admission gains terms, named consts and const generics,     |                             |
|                 |           | and [MSR-3]'s atom-identity sentence. **3105 survives and        |                             |
|                 |           | [MSR-5] depends on it**                                          |                             |
| [ENT-1]         | 2648-2676 | UNCHANGED, and named because [RES-8] depends on 2661: a          | [RES-8]                     |
|                 |           | retained witness never enters acceptance, which is why the       |                             |
|                 |           | saturation fact may not read proof provenance                    |                             |
| batch 0079      | docs/done/| the abort site loses its allocation caller and its release-walk  | [RES-6], [PROV-6]           |
| exhaustion floor| 0079-...  | callers, and the doubling-overflow arm with them                 |                             |
```

**Depended on and unchanged.** Each row is the collation of one or more `Depends:`
lines, and each names the rule that depends on it. A later batch changing one of
these sentences changes a rule of this design without touching it. Dependencies that
fall inside changed text, or that name a retired subject, are on the changed rows
above instead (condition 4).

```text
| rule       | line | the sentence, and who depends on it                                       |
|------------|------|---------------------------------------------------------------------------|
| OWN-3      | 578  | region identifiers are unique within a function: [PROV-1], which is why a  |
|            |      | store region's spelling denotes one store                                 |
| OWN-3      | 580  | distinct caller-supplied regions are incomparable and every ordering rule  |
|            |      | fails closed: [PROV-1] and [BLK-4], the whole invariance argument          |
| OWN-5      | 591  | content reached through any borrow may never be moved, with [SET-2]'s      |
|            |      | exchange the sole exception: [LIV-3], for why the exchange is the only     |
|            |      | route, and [PROV-6], for why dispose's consume half needs no clause        |
| OWN-6      | 614  | a borrow not bound by let is a call-scoped temporary: [PROV-2] and         |
|            |      | [VIEW-2], which is why the argument borrow is not the freeze               |
| OWN-12     | 650  | region substitution controls type equality: [PROV-1], which is why two     |
|            |      | stores are distinguished by their types                                    |
| OWN-13     | 654  | an own-place match moves the scrutinee and binds payloads own: [PROV-6],   |
|            |      | which is why a match is a destructuring and Option<Lease> cannot be dropped|
| TYPE-5     | 379  | argument types match declared parameter types exactly: [PROV-1], the other |
|            |      | half of the invariance argument                                           |
| ERR-4      | 1487 | parallel permissions never reject the source: [PROV-5], which is why its   |
|            |      | par source reads "may execute with overlapping execution"                  |
| FN-1       | 1041 | the call-boundary origin substitution: [PROV-3], which is how an origin    |
|            | -1047| crosses a call and comes back                                             |
| FN-6       | 1211 | recursion is permitted: [STK-2], which excludes a program from [RES-4]     |
|            |      | rather than rejecting it                                                  |
| FN-8       | 1275 | a borrow goal actual uses its resolved referent and an own actual its      |
|            |      | pre-transfer value: [MSR-3], whose call placement reuses the split         |
| PAR-1      | 1987 | whether an overlap was performed is not observable and no rule is stated   |
|            |      | in terms of it: [RUN-1], which is why the no-permission obligation is a    |
|            |      | build obligation and not a rule                                           |
| PROG-1     | 1492 | one closed compilation unit with no function values: [PROV-4]'s exact      |
|            |      | reachability closure and [RES-8]'s composition claim                       |
| ENT-1      | 2661 | a retained witness changes diagnostic parent choice only, never the        |
|            |      | derivable set or acceptance: [RES-8], which is why saturation is declared  |
| ENT-2      | 2681 | an integer-typed const generic is a symbolic constant term: [MSR-6], which |
|            |      | is why admitting its spelling adds no fact source                          |
| ENT-2      | 2693 | one static term per statement: [MSR-3], why a per-point datum is sound     |
| ENT-4      | 2860 | L0's uniqueness and finiteness rests on the difference-bound shape:        |
|            |      | [MSR-2], which is why len + room = cap is an affine premise and not an L0  |
|            |      | fact                                                                      |
| ENT-5      | 2942 | no fact established inside an iteration survives to the next iteration's   |
|            | -2946| head: [MSR-2] and [MSR-3], why an empty-support fact does not cross a      |
|            |      | backedge                                                                  |
| INV-1      | 3105 | the relation is one ordered compare_op and performs no operation:          |
|            |      | [MSR-5], whose clause_expr reuses the restriction verbatim                 |
| STOR-3     | 699  | the derived-drop order and its affine-element clause: [PROV-6], the walk   |
|            | -705 | this rule reuses                                                          |
| QUAL-2     | 2369 | qualification stops compilation and cites no language rule: [RES-9], where |
|            |      | a runtime that cannot publish a capacity fails                            |
```

**META-5 delta**, declared here because the register is its natural home. Numbered
language rules: 131 today, plus the 50 of 3.K, none reusing a live or retired id;
the region-spelling amendment (3.K.0) is counted with its own batch and not here.
Unique fixed lowercase grammar atoms: minus 5 for the retired `heap` and `arena`
effect atoms and the retired `slice`, `buffer` and `box` type productions (`arena`
is one atom serving both a production and an effect entry, and retires once), plus 5
for `resource_closed`, `dispose`, `using`, `linear` and `saturating`; net zero.
Grammar productions: plus 2, being `clause_expr` and `dispose_stmt`, less the retired
`slice_of`-bearing forms; changed, 10, being `let_stmt`,
`return_stmt`, `set_stmt`, `result_binding`, `program_kind`, `struct_decl`,
`enum_decl`, `contract_block`, `effect`, `affine_factor`, with
`requires_clause`/`ensures_clause` counted once as a pair. `ReservedLowerNames`:
plus 3, `cap`, `room` and `head`. Nominal types: plus 6, being 2 providers, 2 runs
and 2 views, one of which (`Span`) is `slice` renamed. Declaration domains: plus 1,
with one `container_declaration_ordinal`. Entry input rows: plus 1. Compound
punctuation tokens: unchanged; this design adds none, and v0.41's five are already
counted there. [SYS-2]'s normative inventory counts change with [VIEW-7], [RES-6],
[RES-7] and [RES-9] and are recomputed when those rules are written into the spec,
not asserted here.

**Retired outright, with no successor.** The fourth draft's five owner types
([BLK-1]); its `AppendView`, `absorb` and the abandoned-window disposition; its
`update` statement and its three atoms; its `Pool` store, `PoolSlot`, `PoolVector`,
`seq_lease`, `pool_frame`, `pool_extent`, `pool_take`, `pool_release` and the pool
seam; its `FixedRing` and four ring rows; its `HeapBox` and `ArenaBox`; its three
failure structs and its `NoRecord`; its `seq_filled`, `seq_vacant`, `seq_take_at`,
`seq_clear`, `seq_truncate`, `seq_reserve_heap`, `seq_reserve_arena`, `seq_shrink`,
`seq_heap_filled`, `seq_push`, `seq_try_push`, `seq_pop` and every `try` row; the
`&uniq buffer<T>` and `&uniq Container` prohibition ([CNT-7], whose *effect* R1
restores without its text); the effect-row atoms `heap` and `arena`; `slice_of`,
`box_new` and `arena_new`; the first draft's `Builder<'r, T>` and `[BLD]`; the
second draft's `[STK-4]` reentrancy premise; `[CNT-5]`; L14; and **this draft's own
two**, the fifth draft's `seq_frame` row ([BLK-2]) and its `seq_exchange` row
([BLK-3]), together with `[CALL-4]`'s exit datum and `[MSR-3]`'s exit placement,
which R1 withdraws. Every id is retired and none is reused.

**Writer doctrine this design invalidates**, which `docs/patterns.md` must carry in
the same batch. **P16** ("One length fact above the writes") rests on hoisting a
length above a sequence of `&uniq` callee writes; under R1 the parameter it hoists
across does not exist, so the pattern is rewritten over `&uniq MutSpan<u8>`, where
[CALL-3] keeps the fact for the reason P16 states, and over the value-in / value-out
form, where the fact is the result's. P16 gains a second correction from [MSR-2] — a
length fact survives a write to a **sibling field**, which probe `r2_4` shows today's
compiler killing. **P17**'s field-by-field fold is **narrowed** to non-linear
aggregates, because [PROV-6] refuses a partial consume of a linear one, and its
`replace` note gains [LIV-2]'s dead-target `set` and [LIV-3]'s in-place exchange.
**P19** is unchanged and gains a case: a measure term joins by the same delta-atom
rule. **P15** is unchanged and both worked programs follow it. **P8** should gain
what probes `q5'`, `m10` and `x1b` bought: an exact `-` or `+` carries an ordering
into a backedge where the wrapping form gives the checker a fresh atom. Five new
patterns are owed: structural disposal with the provider in hand, the linear
destructuring consume, the `propagate`-free allocating helper, 3.L.3's
two-invariant construction loop, and the value-in / value-out helper whose contract
relates a result to an input.

---

### 3.S Surface change proposals

**Every language-surface addition this design relies on is listed here, and every
one of them is the owner's decision and not the design's.** A new keyword, statement
form, grammar production, operator, modifier, compiler-owned type name or operation
name is proposed, never adopted. The rules of 3.K use these spellings because a rule
needs a spelling to be precise, and every use is marked with its proposal id so the
dependency is visible; **nothing in 3.K should be read as decided.** If the owner
changes a spelling, the rule that uses it is unchanged except in its bytes; if the
owner refuses an addition outright, the "do not add it" alternative below states what
the design loses.

Each entry states the exact spelling, why the kernel needs it and why no wf program
has its effect, at least two alternatives with their costs, and its status.

```text
| id  | spelling                                    | kind                    | status   |
|-----|---------------------------------------------|-------------------------|----------|
| S1  | Vector<'s, T>                               | compiler-owned nominal  | PROPOSED |
| S2  | FixedVector<T, n>                           | compiler-owned nominal  | PROPOSED |
| S3  | Heap<'s>                                    | compiler-owned nominal  | PROPOSED |
| S4  | Arena<'s, bytes, align>                     | compiler-owned nominal  | PROPOSED |
| S5  | Span<'r, T>                                 | rename of slice<'r, T>  | PROPOSED |
| S6  | MutSpan<'r, T>                              | compiler-owned nominal  | PROPOSED |
| S7  | seq_fixed, seq_arena, seq_arena_proved,     | operation names         | PROPOSED |
|     |   seq_heap                                  |                         |          |
| S8  | seq_place, seq_place_front, seq_take,       | operation names         | PROPOSED |
|     |   seq_take_front                            |                         |          |
| S9  | arena_frame, arena_extent                   | operation names         | PROPOSED |
| S10 | seq_span, seq_mut_span                      | operation names         | PROPOSED |
| S11 | cap, room, head                             | operation names         | PROPOSED |
| S12 | dispose p using (q1, ..., qk);              | statement form          | PROPOSED |
| S13 | let N(f1: b1, ..., fk: bk) = move v;        | let alternative         | PROPOSED |
| S14 | set (p, x) = f(...);                        | set target list         | PROPOSED |
| S15 | set p = f(q: move p, ...);                  | set admission           | PROPOSED |
| S16 | -> (a: own T, b: own U), let (a, b) = ...,  | result list and its     | PROPOSED |
|     |   return e1, e2;                            |   binding and return    |          |
| S17 | clause_expr over measure terms              | grammar production      | PROPOSED |
| S18 | linear struct N { ... }                     | declaration modifier    | PROPOSED |
| S19 | resource_closed command fn main             | entry marker            | PROPOSED |
| S20 | struct N['s] { ... }                        | region params on a      | PROPOSED |
|     |                                             |   nominal               |          |
| S21 | a const generic as a value, an endpoint     | resolution admission    | PROPOSED |
|     |   and a clause operand                      |                         |          |
| S22 | command.heap as h: own Heap, and main's     | entry input row and     | PROPOSED |
|     |   one region parameter                      |   region parameter      |          |
| S23 | allocates(path)                             | effect production       | PROPOSED |
| S24 | ensures when V(f: r): ... over any variant  | contract routes         | PROPOSED |
|     |   and any result ordinal                    |                         |          |
| S25 | reserve_file -> own Result<FilePermit, ..>  | system-row change       | PROPOSED |
| S26 | saturating(p)                               | contract clause         | PROPOSED |
```

**S1-S2, the two run nominals.** `Vector<'s, T>` and `FixedVector<T, n>`.
*Needed because* a run of initialized slots with a checker-maintained boundary is the
one thing 1.4's criterion says a writer cannot express: `array<T, n>` requires `n`
live values, which for affine `T` is exactly what the writer does not have, and every
data structure in 3.L is arithmetic over these two. *Alternatives:* (a) **do not add
them** — the language keeps `buffer<T>`, which is heap-only, has no affine element
domain (probe `p9`), and cannot carry a store brand, so goal A has no container at
all and D1's repair has nothing to be stated over; (b) **one nominal instead of two**,
with capacity always a measure — costs `FixedVector`'s layout-before-existence
property, so no run is frame-resident and goal A's stack-only programs disappear; (c)
**keep `buffer<T>`'s spelling for the store-resident one** — costs a rename in the
corpus but saves a nominal, and loses nothing else; the owner may prefer it.

**S3-S4, the two provider nominals.** `Heap<'s>` and `Arena<'s, bytes, align>`.
*Needed because* L2 makes a store a value a program must hold, and no wf declaration
can produce an unforgeable one: a writer's `struct Heap {}` is constructible.
*Alternatives:* (a) **do not add them** — allocation stays ambient, probe
`p5_ambient` stays accepted, heap-freedom is not a signature fact, and goals A and B
both fail at their first sentence; (b) **one provider nominal with a kind field** —
costs the type-level distinction that makes `allocates(env.heap)` a heap-reaching row
and makes an arena's `cap` a type constant, and buys one fewer name.

**S5-S6, the two views.** `Span<'r, T>` (today's `slice<'r, T>`, renamed) and
`MutSpan<'r, T>`. *Needed because* [SET-1] 488-490 makes every slice-rooted target
unwritable, so no writable view exists and a system operation cannot fill a caller's
run without taking the run itself; probe `p7` is the refusal. *Alternatives:* (a)
**do not add `MutSpan`** — every element-writing helper takes the run by value and
returns it, which is correct but forces a copy-out/copy-in discipline on I/O and
deletes [CALL-3], the third of the owner's three call rules; (b) **keep the name
`slice` and add `mut_slice`** — costs nothing technical, and the rename to `Span` is
a readability judgment the owner should make on its own merits, not one this design
needs.

**S7-S10, the operation names.** `seq_fixed`, `seq_arena`, `seq_arena_proved`,
`seq_heap`; `seq_place`, `seq_place_front`, `seq_take`, `seq_take_front`;
`arena_frame`, `arena_extent`; `seq_span`, `seq_mut_span`. *Needed because* each
moves a checker-maintained boundary or mints a store, and neither is expressible;
[BLK-3]'s own text shows the one row that *was* expressible (`seq_exchange`) leaving
under L18. *Alternatives:* (a) **do not add them** — as S1-S4; (b) **a different
naming scheme**, for instance `run_*` or a `Vector::` associated-name form — costs
nothing this design depends on, since no rule reads a name; the `seq_` prefix is
chosen only because it groups in `ReservedLowerNames` and reads as one family; (c)
**fold the front operations into the back ones with a direction argument** — costs a
runtime branch on a compile-time constant in the one loop a driver cares about, and
buys two fewer names.

**S11, the three measure readers.** `cap(p)`, `room(p)`, `head(p)` beside `len(p)`.
*Needed because* [ENT-3.S6] 2785 makes `let m = len(P);` a fact and there is no
second such row, so every branch on capacity binds an unrelated atom and the whole
checked half of 3.L is unwritable; `CONTAINERS.md` §3.4 is the demonstration.
*Alternatives:* (a) **do not add them** — every capacity test is a runtime branch the
checker cannot read, `try_place` is unwritable, and [BLK-0]'s own flagship diagnostic
loses its "dominate the place with a branch on `room`" repair; (b) **`room` only**,
deriving `cap` from `len + room` — costs one of the two premises `AUTO` may combine,
which is exactly the arithmetic failure round 5 found; (c) **`head` only as a
typestate with no reader** — costs [VIEW-2]'s unwrapped premise a spelling, so view
formation over a run becomes a flow-sensitive property no `requires` can restore.

**S12, `dispose p using (q1, ..., qk);`.** *Needed because* a store-backed value's
release requires a capability [PROV-6], and no wf statement both consumes a value and
lends a provider to the operation that reclaims it; a writer's
`heap_release(heap: &uniq h, run: move p)` call would be an ordinary call whose
result is `unit`, which is expressible, so this entry's whole justification rests on
the **walk**: the statement's judgment is a structural walk of `p`'s type releasing
every capability-released leaf to the provider its own type names, and a writer
cannot write a walk over a type they did not declare. *Alternatives:* (a) **do not
add it, and reuse `move` into a compiler-owned release operation**, `let done =
heap_release(heap: &uniq h, run: move p);` — no new statement, no new atoms, and
[PROV-6]'s consume and write halves are the ordinary call rules; the cost is that
the walk must then be per-type, so a writer disposing a `Bytes` calls one operation
per leaf and a nested aggregate is a hand-written traversal, which is exactly the
ceremony the walk removes and which gets it wrong silently when a field is added;
(b) **region-end only**, with no per-value release at all — the heap then has no
release operation, every heap-derived value lives until its region ends, and goal B
becomes a program that cannot free; (c) **`drop p using (h);`** or another verb —
costs nothing.

**S13, the destructuring consume `let N(f1: b1, ..., fk: bk) = move v;`.** *Needed
because* linearity is closed under containment, so a linear aggregate must be
takeable apart in one statement that leaves no residual; without it [PROV-6]'s
partial-consume refusal has no mechanical fix and a slab free list has no spelling
(round 4's blocking finding 1). *Alternatives:* (a) **do not add it** — a linear
aggregate can only be moved whole or disposed whole, so a writer who needs one field
out of a linear record cannot get it, and the refusal becomes a wall rather than a
redirection; (b) **`let N { a, b } = move v;`**, brace-form rather than the
`construct` mirror — costs nothing technical and is a readability judgment; this
design writes the paren form only because it is `construct`'s exact inverse and
[GRAM-8]'s field-name discipline then carries over unchanged; (c) **an own-place
`match` with one arm** — already legal for an enum ([OWN-13] 654) and this design
reads it as a destructuring, so for enums the alternative *is* the answer and only
structs need the form.

**S14, the multi-target `set (p, x) = f(...);`.** *Needed because* a two-result
operation at a place that is not a bare binding — a field, a `deref`, a subscript —
has no other spelling: the `let` form cannot write back into the place, and [LIV-3]'s
single-target form cannot bind the second result. *Alternatives:* (a) **do not add
it** — every two-result operation is usable only at a bare binding, so `seq_take` at
`s.buf` or at `wheel[slot]` is unwritable and the library's drain loops disappear;
(b) **two statements with a fresh tuple binding**, `let (next, taken) = f(q: move p);
set p = move next;` — needs no new `set` form and is one statement longer, but it is
*not* equivalent at a non-bare place, because `move p` at a field is a partial move
that kills the root ([OWN-1] 569) and `set p = move next;` at a live affine field is
[STOR-1] 679; so the alternative works only where the single-target form already
does; (c) **spell the introduced binders**, `set (p, let x) = f(...);` — one token
distinguishes the exchanged place from the new name, which round 5 asked for on
readability grounds; this design instead states the binder rule in [LIV-3] and the
owner may prefer the token.

**S15, the exchange admission `set p = f(q: move p, ...);`.** *Needed because* it is
the one form the partition test could not write in wf, and [LIV-3] states the whole
argument: at a bare binding a writer rebinds in two statements, and at every other
place `move p[i]` and `move p.f` are partial moves and `move deref(h)` is a move
through a borrow ([OWN-5] 591), so the only route is a placeholder of the displaced
type, which for a store-backed run costs an allocation and a disposal on a provably
dead arm. *Alternatives:* (a) **do not add it** — every container operation is usable
only at a bare local, which deletes nested containers, fields of records, and every
`deref` through a `&uniq`; probes `t8`, `x2` and `x3` are the rejections; (b) **a
`swap` operation** taking two places — expressible only for two places of one type
and does not reach a call at all, so it solves the placeholder problem and not the
transformation problem; (c) **extend `replace`** so its right-hand side may consume
the target, `let old = replace p = f(q: move p);` — the same semantics with a
binding nobody wants, and [SET-2]'s "the previous value's sole owner is x" would
become false.

**S16, the ordered result list.** `-> (a: own T, b: own U)` on a declaration,
`let (a, b) = f(...);` at a call, `return e1, e2;` in a body. *Needed because*
value-in / value-out (R1) makes every transforming operation return the value it was
handed plus what it computed, and a language with one result can express that only by
declaring a nominal per operation. *Alternatives:* (a) **do not add it** — every
kernel row and every library helper declares a two-field struct, so `seq_take` needs
one nominal, `try_place` another, and the kernel grows a nominal per row, which is
what L18 exists to prevent; (b) **return a prelude `Pair<A, B>`** — one nominal
instead of many, at the cost of a `match` or two field reads at every call site and
of losing the per-ordinal contract route S24 needs.

**S17, `clause_expr` over measure terms.** *Needed because* [GRAM-5] 269's `atom` has
no `call` alternative, so `len(source) <= room(out)` derives nowhere; probe `t5` is
the parse rejection. Under v0.41 the relation itself is already an infix
`compare_op`, so what remains is the *operand set*. *Alternatives:* (a) **do not add
it, and keep hoisting into `define`** — legal today (probe `x10`'s shape) and it
works for a *parameter*, but a `define` is erased into the clause by alpha-expansion
and its right-hand side is an ordinary expression, so it cannot name a **result**'s
measure at all (probe `t14`), which is exactly what R1's contracts need; (b) **extend
`atom` with a call alternative** — reaches the same clauses and much more, since
`atom` occurs in argument lists, subscripts and infix operands, so it would admit
nested calls everywhere and [GRAM-9]'s three-address discipline would go with it.

**S18, the `linear` modifier.** `linear struct N { ... }`, `linear enum N { ... }`.
*Needed because* the capability criterion [PROV-6] sees storage obligations and not
logical ones, and a writer cannot write *an obligation to give a value back*: every
wf mechanism for it is a runtime field a program can forget to read. Round 5 measured
the gap in this design's own program 4.1. *Alternatives:* (a) **do not add it, and
derive linearity entirely from types** — the storage half still works, and the cost is
Q0c exactly as the fifth draft recorded it: a library pool's lease is affine, a
dropped lease loses a block for the life of the program, and nothing reports it in
the fact state or in `E`; (b) **`must_consume` as an attribute-like spelling** — the
same semantics under a name that says what it does rather than what class it joins,
and it avoids overloading a word this design also uses for the derived predicate;
that is a real argument for it and the owner may prefer it; (c) **make the pool a
kernel store**, so its lease is linear by the criterion — restores the fourth draft's
`Pool`, `PoolSlot` and six operation rows, which the minimality ruling removed.

**S19, `resource_closed`.** *Needed because* the whole design's judgment must be a
compile error for the program that asks for it and a note for every other, and there
is no wf declaration that changes the severity of a compiler judgment.
*Alternatives:* (a) **do not add it** — `E` is computed and reported for every
program and never enforced, so a writer learns at deployment rather than at compile
time; goal A's "one entry marker turns the failure into a compile error" is the
sentence lost; (b) **a compiler flag rather than a source marker** — makes acceptance
a function of the invocation, which [SCOPE-2] 18 and L1 forbid.

**S20, region parameters on a nominal.** `struct N['s] { ... }`, used as `N<'s>`.
*Needed because* a store's identity is in the type [PROV-1] and a nominal that holds
a store-backed value must therefore name that store; probes `r2_6` and `m05` are the
parse errors today. *Alternatives:* (a) **do not add it** — no nominal may hold a
store-backed value, so `Bytes`, `BlockPool`, `Chunk` and every library structure are
unwritable and the design has no data structures at all; (b) **infer the nominal's
region from its fields** — an inference [TYPE-5]'s statement-local discipline
forbids, and it makes two instantiations of one nominal silently different types.

**S21, a const generic as a value.** *Needed because* every capacity-parametric
function reads its bound as a value, a loop endpoint or a clause operand; probes
`t1`, `t2` and `t3` are the three rejections and probe `t4` shows a named const
already works in all three. *Alternatives:* (a) **do not add it, and use named consts
only** — every capacity is a top-level `const` and every parametric function is
written once per capacity, which round 5 counted at about forty-three bodies for
fourteen library algorithms; (b) **admit it as a clause operand only** — closes the
contract half and leaves the loop and the value halves, so `vacant<T, n>` still
cannot count to `n`.

**S22, `command.heap` and main's region parameter.** *Needed because* the heap must
enter as a value [L2] and the entry table [FN-7] 1227 is closed, and because a
signature that must name the entry heap needs a region to name (3.K.0).
*Alternatives:* (a) **do not add the row** — there is no heap value, so goal B has no
honest allocation story and every hosted program keeps the ambient allocator; (b)
**add the row and forbid main a region parameter** — then no signature can name the
entry heap, so a helper that releases heap-backed storage cannot be written and every
`dispose` must occur in `main`.

**S23, `allocates(path)`.** *Needed because* [EFF-1] 1369's fixed atoms `heap` and
`arena 'r` cannot name a provider a function received as a field of an aggregate, so
[PROV-4]'s reachability closure is inexact exactly where a program threads an
environment struct. *Alternatives:* (a) **do not change it** — heap-reachability is
computed per region name rather than per provider place, which is sound but refuses a
program that holds two arenas in one record; (b) **keep the atoms and forbid a
provider in an aggregate** — enforceable, and it costs the `Env`-struct pattern
`docs/patterns.md` P5 already teaches.

**S24, per-variant and per-ordinal contract routes.** `ensures when V(f: r): ...`
for any variant of any returned enum, an unrouted clause for any measured result, one
route per result ordinal, and field projection on a result datum. *Needed because*
[FN-9] 1307 admits exactly `when Ok(value: r):` over `Result<int, E>` and 1314
excludes a nested result projection, so no library constructor can publish a fact
about what it built and no fallible helper can publish that it succeeded; probes `t6`
and `t14` are the rejections and round 5 traced three undischarged obligations in
program 4.1 to them. *Alternatives:* (a) **do not add them** — every capacity proof
collapses into the function that owns the run, every helper boundary costs a re-read
and a statically-true runtime branch, and the whole point of moving containers into a
library is lost at the first function boundary; (b) **add the variant route and not
the projection** — reaches `pool_take` and `try_place`, and leaves `ring_new`'s
`len(result.slots) >= n` unstatable, so a constructor still cannot publish; (c) **a
witness integer beside every result**, which is legal today — costs one result per
fact and is what a writer does now.

**S25, `reserve_file` becomes fallible.** `-> result: own Result<FilePermit,
IoError>` in place of [SYS-2] 2261's total `own FilePermit`. *Needed because* the
handle table is a covered store with a finite capacity [RES-9], and L3 requires its
refusal to be a value. *Alternatives:* (a) **do not change it** — the store has no
refusal edge, so a marked program that opens files in a loop either cannot be
accepted or is accepted with a promise the runtime cannot keep; (b) **a total
`reserve_file` over a proved capacity**, the proved spelling every other covered
store has — costs nothing at the eleven corpus call sites and costs one header
invariant over `room(factory)` in a loop, which is strictly less than the `match`
this proposal adds; 5.0 records it as the alternative the owner should weigh.

**S26, `saturating(p)`.** A contract clause naming a provider parameter. *Needed
because* [RES-10]'s route (ii) must compose across a call, and the fact it needs —
*this function performs no acquisition on `p`'s store that could succeed when that
store is full* — is a property of a body, which [CALL-5] forbids a caller to derive.
*Alternatives:* (a) **do not add it** — route (ii) does not compose, so a retaining
loop is refused the moment its acquisition is one function down, which is the shape
program 4.1 is written around; (b) **derive it from the body anyway**, as the fifth
draft did — the option round 5 refused, because it makes [CALL-5] false and opens the
door to the next body-derived summary, which is what D1's flag was; (c) **infer it
from the callee's declared `allocates` row** — an `allocates` path says which
provider is reached, never which spelling was used, so the inference is not available.

**Names this design does *not* propose**, listed because a reader of an earlier draft
will look for them: `HeapBox`, `ArenaBox`, `PoolSlot`, `Pool`, `PoolVector`,
`FixedRing`, `AppendView`, `heap_take`, `arena_take`, `pool_take`, `pool_release`,
`pool_new`, `seq_frame`, `seq_exchange`, `absorb`, `update`, `by`, `into`,
`seq_filled`, `seq_vacant`, `seq_clear`, `seq_truncate`, `seq_take_at`, `seq_push`,
`seq_pop`, and the failure nominals `Full<T>`, `TooSmall`, `OutOfMemory`,
`PoolExhausted`, `NeedCapacity` and `NoRecord`. The box and slot names are runs of
capacity one; the pool and ring names are library nominals and library functions a
writer declares in wf (`CONTAINERS.md` §3); `seq_frame` and `seq_exchange` are this
draft's two removals; and the failure nominals are ordinary user structs, because no
kernel acquisition consumes an affine input.

---

### 3.L The library, written in wf

#### 3.L.0 How to read this section

Everything below is **ordinary wf**, written against 3.K and against the unchanged
v0.41 rules. It defines no rule, amends no rule, and is named by no rule. It exists
to discharge L18's obligation: an item the kernel no longer carries is written out
here, or the kernel lacked a primitive and 3.L.6 says which.

Each item states its **proof route** — which kernel rule discharges each obligation,
and which of those v0.41 already proves today, naming the probe where one exists. The
code is design text; the standard it is held to is that every statement is accepted by
a compiler implementing 3.K and the unchanged v0.41 rules.

Four discipline sentences are stated once here rather than repeated, and each is a
round-5 finding about this section rather than about the rules:

- **Every body is three-address.** `let mirror = count -wrap 1_u64 -wrap at;` is two
  operations in one expression and is a [GRAM-4] parse error (probe `t13`); [GRAM-6]
  282 says composition is by `let`.
- **`Z` is the term language's zero and appears only in inventory rows.** wf source
  writes `0_u64`; probe `t11` is the [GRAM-5] rejection of the other spelling. The
  distinction is exactly L18's line between generated data and source.
- **A measure read is `pure` at the operation and an ordinary `reads` at the
  caller**, so a helper that reads `len` of a borrowed run names it in its row (probe
  `t10`), and a helper that declares a row its body does not exhibit is refused the
  same way. [EFF-2] 1432 admits no wider and no narrower declaration.
- **A writer's generic over an element type cannot serve a copy and an affine
  instantiation from one body** — probes `m12` and `m14` show one accepted at
  `box<u64>` and rejected at `u64` — so a function that *reuses* a value is written
  per element class and says so. That is Q8, not a partition finding. Capacity
  genericity, by contrast, is available: [MSR-6] makes a const generic a value, and
  without it every function below would be written once per capacity.

#### 3.L.1 The owner names

`FixedVector<T, n>` is the kernel type and needs no library. `HeapVector<T>` and
`ArenaVector<'a, T>` are what a writer *calls* a `Vector<'s, T>` whose store is the
heap and a named arena respectively; they are one kernel type at two regions and the
library adds nothing to them (footnote 1). Under 3.K.0 a heap run in a stored
position is written `Vector<u8>` and an arena run `Vector<'a, u8>`, which is the whole
visible difference between them. **A ring is not a library type at all**: under
[BLK-1]'s window a ring is a `FixedVector<T, n>` used from both ends, so `FixedRing`
has no successor rather than a library one (footnote 2).

#### 3.L.2 The partition, item by item

Every item is written in wf in `CONTAINERS.md` §3 against 3.K and against the
unchanged v0.41 rules, with its proof obligations walked there. This table is the
result; three items are written out below because they are the ones that earned or
lost a kernel row.

```text
| item                          | written as                          | route, and what discharges it       |
|-------------------------------|-------------------------------------|-------------------------------------|
| FixedVector<T, n>             | the kernel type itself              | nothing to write                    |
| HeapVector, ArenaVector       | Vector<'s, T> at two regions        | nothing to write                    |
| a ring, a queue, a deque      | a run used from both ends [BLK-1]   | nothing to write; no Option, no tag |
| vacant<T, const n>            | a counted loop of seq_place over    | two header invariants; the exit     |
|                               | None<T>(), 3.L.3 below              | ordering, not an equality; x1c, x1d |
| filled<T, const n>            | the same, reusing one copy value    | as above; per element class (Q8)    |
| swap                          | seq_take, one element replace,      | three statements; 3.L.2 below       |
|                               | seq_place                           |                                     |
| take_at                       | swap with the last, then seq_take   | the requires plus 0 <= index        |
| clear, truncate               | a counted drain, two invariants     | as vacant; a linear T disposes each |
|                               |                                     | and the signature says so [PROV-6]  |
| growth policy, HeapVector     | seq_heap, drain from the front,     | four invariants; the window is what |
|                               | append at the back, replace,        | makes order preservation free       |
|                               | dispose                             |                                     |
| block pool with a lease       | linear struct Lease['s] plus a      | a branch on len and on room, which  |
|                               | FixedVector<Lease<'s>, m> free list | needs [ENT-3.S6] over four measures |
| collect and the appenders     | a counted loop, value in and value  | the exchange, and a contract that   |
|                               | out, 3.L.4 below                    | relates a result to an input        |
| keyed families                | vacant plus element replace         | [OP-4] from the requires; x7        |
| try_place, try_take, try_push | a branch on room or len and two     | [ENT-3.S6] again                    |
|                               | returns                             |                                     |
| update p by op(...)           | set p = op(receiver: move p, ...)   | [LIV-3]                             |
| update p by op(...) into x    | set (p, x) = op(receiver: move p,   | [LIV-3]'s multi-target form         |
|                               | ...)                                |                                     |
| OutOfMemory<T> and its family | an ordinary one-field struct over   | [BLK-4] admits it; the kernel needs |
|                               | the writer's own type               | none                                |
```

**The swap, written out, because it is this draft's removal.** The fifth draft made
`seq_exchange` a kernel row and named it the fifth of seven additions. It is three
statements over rows the kernel already has:

```wf-design
fn swap_with_last<T, const n: u64>(vector: own FixedVector<T, n>, at: own u64)
    -> result: own FixedVector<T, n>
    reads(vector), writes(vector) contract {
  requires at + 1_u64 <= len(vector);
  ensures len(result) >= len(vector);
} {
  doc "Exchanges the element at at with the last element.";
  let (rest, endv) = seq_take(vector: move vector);
  let old = replace rest[at] = move endv;
  let back = seq_place(vector: move rest, value: move old);
  return move back;
}
```

It is the transposition of `at` with the last position, for a copy, affine or linear
element type alike — the placeholder the fifth draft said an element `replace` needs
is the element `seq_take` just handed back — and transpositions with one fixed
position generate every transposition. L18 therefore removes the row.

**What writing it this way costs, stated rather than hidden.** The three statements
kill and re-establish `len` twice where one row published `len(result) = len(vector)`
once, so the `requires` above is `at + 1 <= len(vector)` rather than
`at < len(vector)` and the caller carries the measure through three steps instead of
one. That is a real proof-surface cost, it is a cost a writer pays for a capability
the kernel does not owe them, and it is the trade L18 asks for. If the owner judges
the cost too high the row comes back, and 3.S has no entry for it because this draft
does not propose it.

#### 3.L.3 Filled and vacant construction, written out

`vacant` is the more interesting because round 3 concluded no loop could publish
`len = n`; it is right that no loop publishes the *equality*, and wrong that the
equality is what a subscript needs.

```wf-design
fn vacant<T, const n: u64>() -> result: own FixedVector<Option<T>, n> pure contract {
  ensures len(result) >= n;
} {
  doc "Builds a run of n slots, every one holding None.";
  let built = seq_fixed::<Option<T>, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: len(built) >= at,
    invariant spare: room(built) + at >= n
  ) {
    let empty = None<T>();
    set built = seq_place(vector: move built, value: move empty);
  }
  return move built;
}
```

**Proof route.** `seq_fixed` publishes `len(built) = 0`, `cap(built) = n` and
`room(built) = n` — all three exactly, which is [BLK-0]'s completeness sentence doing
the work round 5 found missing. `grown`'s base is `0 >= 0`; `spare`'s base is
`n + 0 >= n`, and it needs no appeal to the standing identity because the row
published `room` itself. `seq_place`'s own requirement `room(built) > 0` discharges
from `spare` and the counted loop's `at < n` ([ENT-3.S11]) by [MSR-4] step 5. On the
backedge `seq_place` declares `len(result) = len(vector) + 1`,
`room(result) = room(vector) - 1`, `cap(result) = cap(vector)` and
`head(result) = head(vector)`, over that call's own datum, which has empty support
[MSR-3]; `room` falls by one and `at` rises by one, so each invariant is preserved by
**one** published premise, which is what puts the derivation inside [ENT-6] 3019's
two-premise budget. Probe `g4` is that shape accepted at v0.41 scale and probe `g3`
is the same shape rejected when the relation is missing. The `set` is an in-place
exchange, so it is **not** a declaration event and the two atoms survive on the same
term [MSR-3]. At the exit `at = n`, so `len(built) >= n` holds and the `ensures`
discharges.

`n` is read as a loop endpoint, which is [MSR-6] and probe `t2`'s rejection today.
`vacant` is generic over `T` with no copy bound, because `None<T>()` is built fresh
each iteration. `filled` is not, because it reuses one `value`:

```wf-design
fn filled<T, const n: u64>(value: own T) -> result: own FixedVector<T, n> pure contract {
  ensures len(result) >= n;
} {
  doc "Builds a run of n slots, every one holding a copy of value.";
  let built = seq_fixed::<T, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: len(built) >= at,
    invariant spare: room(built) + at >= n
  ) {
    set built = seq_place(vector: move built, value: value);
  }
  return move built;
}
```

Same route, and it is written for a **copy** `T` only: the bare `value` use is
[OWN-1] 564's copy-on-use, and at an affine instantiation the same body needs `move`
and would consume it on the first iteration. That is Q8 and 3.L.0 states it once.
This is the function [VIEW-7] needs for an addressable I/O destination, and it is the
one `wfgrep.wf`'s migration calls twice — **once**, under [MSR-6], where the fifth
draft called two hand-written copies.

#### 3.L.4 `collect`, written out

The one program every draft has carried, and the item R1 changes most.

```wf-design
fn collect['s](out: own Vector<'s, u8>, source: own Span<u8>)
    -> (rest: own Vector<'s, u8>, written: own u64)
    reads(out, source), writes(out) contract {
  requires len(source) <= room(out);
  ensures len(rest) >= len(out);
  ensures written <= len(rest);
} {
  doc "Appends every byte of source into the destination's spare room.";
  let count = len(source);
  let before = len(out);
  for @copy (
    at in 0_u64..count,
    invariant spare: room(out) + at >= count,
    invariant grown: len(out) >= before + at
  ) {
    let byte = source[at];
    set out = seq_place(vector: move out, value: byte);
  }
  invariant done: len(out) >= before + count;
  let total = before + count;
  return move out, total;
}
```

`collect` writes **one** region name, `'s`, at its binder and at the two positions
whose store must be the same one; `source`'s loan region relates nothing and is
elided, and so is `'s` at every call site, because the `out` operand determines it.
Under the fourth draft the same function carried three region arguments at every
call; under the fifth it carried none and took its destination by `&uniq`, which is
the parameter R1 withdraws. One written identifier per hand-back helper is R1's
spelling cost and it is the whole of it.

**Proof route.** `let count = len(source);` and `let before = len(out);` are
[ENT-3.S6] equalities over the live terms, and at that point the live term of
`room(out)` and `len(out)` each equals its **entry datum** [MSR-3], so the `requires`
transports into the loop's base: `spare` at `at = 0` is `room(out) >= count`, which is
the clause. `grown`'s base is `len(out) >= before`. `seq_place`'s `room > 0`
discharges from `spare` and `at < count` by [MSR-4] step 5; probes `k21` and `k21b`
are that arithmetic at v0.41 scale, accepted and then rejected when the invariant is
deleted. The backedge is [MSR-3]'s three steps over **one** published relation per
invariant, as in 3.L.3. `done` is the [INV-1] exact-exhaustion conclusion at the
continuation — probes `x1c` and `x1d` are that shape accepted today — and it is what
makes the exact `before + count` discharge its [OP-2] domain: `done` plus the standing
`len(out) <= cap(out)` and the implicit `cap(out) <= max(u64)` is a one-premise
`AUTO`. The two `ensures` then read off `done` and off `total`'s own equality.

**What R1 changed here, and what it cost.** The fifth draft's `collect` took
`out: &uniq Vector<u8>` and returned only `written`, and its `ensures len(out) =
written` denoted a caller-side *exit datum* that the callee had no placement for —
round 5's first attack. This version takes the run by value and hands it back, so
every clause names something the callee can see and the caller receives: `len(out)`
is the input's length and `len(rest)` is the output's. **The non-shrink guarantee is
back and is one clause**: `ensures len(rest) >= len(out)` is exactly what L14
promised and `AppendView` was built for, with no third view, no commit event, no
carried datum and no exit placement. The cost is one result binder at every call site
and one `set (buf, count) = collect(...)` where the fifth draft wrote
`set count = collect(...)` — and that `set` was itself undischarged, because a plain
`set` receiver was not an [ENT-3.S12] destination.

**Two of the eight are here.** Without [LIV-3]'s exchange the loop body has no
spelling at a non-bare place, and without the S12 destination clause over a `set`
target list the caller's `set (buf, count) = collect(out: move buf, source: move
line);` publishes nothing.

#### 3.L.5 The store region and the disposals, before and after

**The store region is elided.** `byte_string.wf` has exactly one store, so under
[PROV-1] nothing in it names a region. Its join — the program spells it `bs_concat`
— reads, under the fourth draft's brand:

```wf-design
struct Bytes['h] {
  v: HeapVector<'h, u8>;
}

fn bs_concat['h, 'd, 's, 'b](destination: &uniq 'd Bytes<'h>, source: &'s Bytes<'h>,
                             heap: &uniq 'b Heap<'h>) -> done: own Bool
    reads(destination.v, source.v, heap), writes(destination.v, heap), allocates(heap) { ... }

    let joined = bs_concat::<'h, 'd, 's, 'b>(destination: ..., source: ..., heap: ...);
```

and under this draft, where R1 also takes the destination by value:

```wf-design
struct Bytes {
  v: Vector<u8>;
}

fn bs_concat(destination: own Bytes, source: &Bytes, heap: &uniq Heap)
    -> (joined: own Bytes, grew: own Bool)
    reads(destination.v, source.v, heap), writes(destination.v, heap), allocates(heap) { ... }

    set (left, grew) = bs_concat(destination: move left, source: &right, heap: &uniq heap);
```

The whole region parameter list leaves the struct and the signature, four brand
occurrences leave the written types, three borrow annotations lose their names, and
the call site loses its `targs` and its three borrow names. Across the eleven
functions of `byte_string.wf` that is ten `['h]`, fifteen brand occurrences and
twelve call-site brand arguments from the brand alone, and every region parameter
list, borrow name and call-site region argument from 3.K.0.

**And seven disposals arrive.** That is R2's cost and it is counted rather than
described: `Bytes` is linear because `Vector<u8>`'s release needs the `Heap`, so
every one of `byte_string.wf`'s seven points at which a `Bytes` value stops being
used is a `dispose s using (heap);` — five in `main` and two inside `bs_reserve`.
None of them existed before, because today the compiler frees the `buffer<u8>` at a
scope exit under no effect row at all (probe `r2_5`). Seventeen of the roughly
twenty-nine writer-visible items the program then carries buy something a systems
programmer wants: five provider parameters, seven disposals, five `match`es on a
typed refusal. The way to carry fewer is an arena, whose values are affine.

#### 3.L.6 What the partition test found the kernel lacked

Eight, each named with the library function that demanded it and the probe that
shows it is new capability rather than a compiler defect. Two of the fifth draft's
seven are gone and three are added, which is what running the test in both
directions produced.

```text
| # | kernel addition                      | demanded by                       | today                 |
|---|--------------------------------------|-----------------------------------|-----------------------|
| 1 | the in-place exchange admission of   | collect, bs_reserve, pool_take,   | s8, x2, x3 REJECTED   |
|   | `set` [LIV-3, S15]                   | vacant, filled, clear, try_place  | [STOR-1]              |
|   |                                      | — every library function that     | AffineSetTarget       |
|   |                                      | transforms a place it does not    |                       |
|   |                                      | own outright                      |                       |
| 2 | its multi-target form [LIV-3, S14]   | pool_take, bs_reserve's drain,    | new grammar           |
|   | and the ordered result list [S16]    | clear, collect's caller — every   |                       |
|   |                                      | two-result row at any place       |                       |
| 3 | [ENT-3.S6] over the four measures    | every try_ form, pool_take,       | S6 2785 covers len    |
|   | [BLK-0]                              | pool_release — every branch on a  | alone                 |
|   |                                      | capacity                          |                       |
| 4 | the construct placement of the       | Bytes, BlockPool — every library  | construct kills the   |
|   | measure datum [MSR-3]                | nominal wrapping a run            | operand's measures    |
| 5 | a const generic as a value, an       | vacant, filled, try_place, and    | t1, t2, t3 REJECTED   |
|   | endpoint and a clause operand        | every capacity-parametric         | [TYPE-5]              |
|   | [MSR-6, S21]                         | function; without it the corpus   | UnresolvedUse; t4     |
|   |                                      | needs ~43 bodies for 14 algorithms| ACCEPTED              |
| 6 | a relation published per enum        | pool_take, try_place, bs_reserve, | t6 [FN-9] Invalid-    |
|   | variant and per result ordinal, with | and every library constructor;    | PostconditionSelector;|
|   | field projection on a result datum   | without it no constructor can     | t14 [TYPE-5] on       |
|   | [CALL-4, S24]                        | publish and no fallible helper    | len(result)           |
|   |                                      | can say it succeeded              |                       |
| 7 | the window's front operations        | every queue, ring, deque and FIFO | no analogue; a        |
|   | [BLK-1, BLK-3, S8]                   | — and the growth policy, whose    | shifting take_front   |
|   |                                      | order preservation is free under  | IS writable, so only  |
|   |                                      | a window                          | the head-carrying     |
|   |                                      |                                   | form enters           |
| 8 | linearity by declaration             | the pool's Lease, and every       | 4.1 leaks a block     |
|   | [PROV-6, S18]                        | library that recycles values      | today with no         |
|   |                                      |                                   | diagnostic            |
```

**What left the list, and why.** The fifth draft's item 3 was [CALL-4]'s exit datum,
which R1 withdraws: a helper takes the run by value, so the fact it publishes is a
relation on its own result and needs no placement at a caller's point. Its item 5 was
`seq_exchange`, which 3.L.2 writes in three statements. And its item 6 was "the
`&uniq` run parameter, i.e. [CNT-7]'s deletion", which R1 also withdraws — the
parameter is gone and the capability it was said to buy is the by-value form.

And the list that matters as much: **what the partition did *not* need.** A queue
needed no kernel ring, a pool needed no kernel store, a keyed table needed no kernel
occupancy, a growth policy needed no kernel growth row, middle removal needed no
kernel row, filled and vacant construction needed no kernel row, and the `try` family
needed nothing at all. Five owner types became two, thirty-odd operations became
twelve, three views became two, sixteen added nominals became six, and one
statement form became an admission on an existing one.

One item was **not** resolved by writing it, and it is the honest residue: a writer's
generic cannot serve a copy and an affine element type from one body, so `filled` is
written per element class (3.L.0). That is Q8 and not a missing primitive. The fifth
draft's other residue — an arena-backed lease that a writer drops leaks a free-list
slot with no diagnostic — is closed by item 8.

#### 3.L.7 When to write `linear`, and when not to

The storage obligation is derived and a writer never marks it: a heap-backed run is
linear because its release needs the `Heap`, an arena-backed run and a frame-resident
run are affine because their reclamation needs nothing, and any nominal reaching a
linear value is linear by containment [PROV-6]. **Marking a store-derived type is
always redundant and is a sign the writer has misread the criterion.**

The modifier is for a **logical** obligation, and the whole test is one question:

> **Would silently dropping this value be a bug?**

If the answer is yes for a reason that is not about storage, the type is `linear`.
The shapes that pass are recognizable:

```text
| shape                          | what the drop would silently do                            |
|--------------------------------|------------------------------------------------------------|
| a lease from a pool the library| consume a free-list slot for the life of the region, so the |
|   owns                         | pool empties with no diagnostic and no envelope movement    |
| a transaction that must commit | leave the journal open, so a later reader sees a partial    |
|   or roll back                 | write that no path ever completed                           |
| a request that must be answered| drop a reply the peer is waiting for                        |
| a permit, ticket or token that | leak an admission slot the issuer counts                    |
|   an issuer counts             |                                                            |
| a builder that must be finished| discard everything appended to it                           |
```

And the shapes that do **not**: a value whose only cost of being dropped is memory
the language already reclaims; a value the writer merely wants to remember to use, for
which the modifier is a type-level answer to a review question; and a value whose
obligation is conditional, since the modifier is unconditional and a writer who marks
one will meet [LIV-1] on the arm where the obligation does not apply.

The cost of a wrong `linear` is paid at every scope exit of every value of that type,
including in code the writer does not own, and the diagnostic names a binding rather
than the obligation the marker meant. The cost of a missing one is what 4.1 had. When
in doubt, the shapes above are the guide and the question is the test.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. The
standard they are held to is that every statement is accepted by a compiler
implementing 3.K's rules, **the library functions of 3.L**, and the unchanged v0.41
rules; both were walked statement by statement against all three before this draft
was finished, and this time the walk was held to the standard round 5 asked for:
**for every loop, the facts live at its head and the rule that keeps them there.**
Round 5 found the fifth draft's pair failing at three obligations each, and all six
were of one kind — a fact established before a loop that the loop's own body kills,
with no header invariant to carry it. Every loop below states its own.

Byte figures are symbolic. No implementation computed any of them, and where a
figure depends on code generation the table says so instead of inventing a number.

### 4.1 A cooperative run queue with the heap absent

A fixed run queue of tasks, a 256-byte transmit ring, and an eight-block pool with
typed exhaustion and a **linear lease**. Each task is a state machine that advances
one step per turn and re-queues itself while it wants another. No heap, no
recursion, an acyclic call graph, and a queue loop whose resource state is restored
on every backedge. It is **not** a context-switching scheduler, and 1.5 says why. It
uses `try_place`, `try_take`, `pool_new`, `pool_take` and `pool_release`
(`CONTAINERS.md` §3.4) from the library, and nothing else the kernel does not
declare.

```wf-design
struct Task {
  state: u32;
  arg: u64;
}

linear struct Lease['s] {
  run: Vector<'s, u8>;
}

struct BlockPool['s] {
  free: FixedVector<Vector<'s, u8>, 8>;
}

fn advance(task: own Task) -> next: own Option<Task> reads(task.state, task.arg) {
  doc "Advances one state machine and returns it again while it wants another turn.";
  let step = task.state +wrap 1_u32;
  let more = step < 3_u32;
  if more {
    let ready = Task(state: step, arg: task.arg);
    return Some<Task>(value: move ready);
  }
  return None<Task>();
}

fn render['s](block: own Lease<'s>, task: &Task)
    -> (rest: own Lease<'s>, written: own u64)
    reads(block.run, task.state), writes(block.run) contract {
  requires room(block.run) >= 8_u64;
  ensures written <= len(rest.run);
} {
  doc "Writes one eight-byte record for a task into the leased block.";
  let narrowed = cvt::<u32, u8>(deref(task).state);
  let mark = 63_u8;
  match narrowed {
    Ok(value: byte) => {
      set mark = byte;
    }
    Err(error: narrowing) => {
    }
  }
  for @fill (
    at in 0_u64..8_u64,
    invariant spare: room(block.run) + at >= 8_u64,
    invariant grown: len(block.run) >= at
  ) {
    set block.run = seq_place(vector: move block.run, value: mark);
  }
  invariant done: len(block.run) >= 8_u64;
  return move block, 8_u64;
}

fn drain['s](ring: own FixedVector<u8, 256>, block: &Lease<'s>, count: own u64)
    -> (rest: own FixedVector<u8, 256>, sent: own u64)
    reads(ring, block.run), writes(ring) contract {
  requires count <= room(ring);
  requires count <= len(block.run);
  ensures sent <= len(rest);
} {
  doc "Copies one prefix of the leased block into the transmit ring.";
  for @copy (
    at in 0_u64..count,
    invariant spare: room(ring) + at >= count,
    invariant grown: len(ring) >= at
  ) {
    let byte = deref(block).run[at];
    set ring = seq_place(vector: move ring, value: byte);
  }
  invariant done: len(ring) >= count;
  return move ring, count;
}

resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Runs a cooperative queue of state machines over a pooled block store and a transmit ring.";
  let ring = seq_fixed::<u8, 256>();
  let pending = seq_fixed::<Task, 32>();
  let first = Task(state: 0_u32, arg: 65_u64);
  set (pending, unplaced) = try_place::<Task, 32>(vector: move pending, value: move first);
  match unplaced {
    None() => {
    }
    Some(value: rejected) => {
      return exit_status(code: 1_u8);
    }
  }
  let code = 0_u8;
  region 'a {
    let scratch = arena_frame::<65536, 16, 'a>();
    let made = pool_new::<'a>(arena: &uniq scratch);
    match made {
      None() => {
        set code = 1_u8;
      }
      Some(value: pool) => {
        loop @queue (
          invariant slots: len(ring) + 8_u64 <= 256_u64
        ) {
          set (pending, next) = try_take::<Task, 32>(vector: move pending);
          match next {
            None() => {
              break @queue;
            }
            Some(value: task) => {
              set (pool, leased) = pool_take::<'a>(pool: move pool);
              match leased {
                None() => {
                }
                Some(value: held) => {
                  let spare = room(held.run);
                  let big = spare >= 8_u64;
                  if big {
                    set (held, written) = render::<'a>(block: move held, task: &task);
                    set (ring, sent) = drain::<'a>(ring: move ring, block: &held, count: written);
                  }
                  set (pool, unreturned) = pool_release::<'a>(pool: move pool, lease: move held);
                  match unreturned {
                    None() => {
                    }
                    Some(value: lost) => {
                      let Lease(run: orphan) = move lost;
                    }
                  }
                }
              }
              let stepped = advance(task: move task);
              match stepped {
                None() => {
                }
                Some(value: again) => {
                  set (pending, refused) = try_place::<Task, 32>(vector: move pending, value: move again);
                  match refused {
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
  }
  return exit_status(code: code);
}
```

#### The envelope the compiler publishes

```text
E(queue.wf, <embedded target>, <this build>) row W = 1, digest <module>

  region  static.image        bytes  <target>        align  <target>  contiguous
  stack   entry               bytes  <post-codegen>  align  <ABI>     contiguous
  lanes                       count  1
  slots   task.records        count  0
  slots   completion.records  count  0
  slots   handle.table        count  0
  handle  adapter.mappings    count  0
```

`static.image` is the const items and the static parts of the emitted module
[STOR-6]. `stack.entry` is `main`'s frame — the `FixedVector<u8, 256>` ring (256
slots plus three measure words), the `FixedVector<Task, 32>` (32 strides plus three
words), the `BlockPool`'s `FixedVector<Lease<'a>, 8>` (8 descriptors plus three
words) and the one `arena_frame` occurrence's 65536-byte extent — plus `render`,
`drain`, `advance` and the library, plus the runtime frames beneath `main` and its
bounded teardown, plus the cleanup-scratch domain, which [RES-5] places in this same
frame and whose depth is the height of `Task`, `Lease` and `BlockPool` and is
therefore a constant; measured post-codegen over the whole chain [STK-3], [PROV-5],
[RES-5]. `lanes` is 1 because no `par` construct is emitted [RUN-1] and [RUN-2]
publishes the `W = 1` row; every `slots` and `handle` row is zero because there is no
`par` permission, no may-suspend operation and no system handle. **A `handle` row of
zero is what makes [RES-7]'s test meaningful**: a marked program calling `read_at`
would name `completion.records`, whose count in this row is zero, and be refused at
the call.

#### Why it is source-resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | main declares pure, selects no command.heap, and arena_frame is pure         |
|                       | [BLK-2], so [PROV-4]'s closure is empty and [RES-4] does not fire            |
| acyclic call graph    | main -> {render, drain, advance, the library, the kernel domain}. No cycle,  |
|                       | so [STK-1] rewrites nothing and [STK-2] passes; [PROV-5]'s activation        |
|                       | refusal reads the same post-rewrite graph and does not fire, and the frame   |
|                       | form is per activation in any case                                           |
| arena demand bounded  | pool_new takes eight 256-byte runs once, before the queue loop; the loop's   |
|                       | backedge delta on the bump domain is 0, so [RES-10]'s loop rule needs no     |
|                       | iteration bound. The lease and its release are on the same path, and R2 is   |
|                       | what makes that a checked fact rather than an assumption: `Lease` is linear, |
|                       | `Option<Lease>` is linear by containment, so the refusal arm cannot be       |
|                       | dropped and the free list's delta is 0 on every path                         |
| the free list         | a FixedVector in a frame; frame placement's [RES-5] row is decided at        |
|                       | compile time and contributes no (peak, delta). What keeps it full is R2 and  |
|                       | not the envelope, which is the honest division of labour                     |
| queue and ring        | FixedVector<Task, 32> and FixedVector<u8, 256> are frame placement           |
| cleanup scratch       | every type reachable from main has an acyclic containment graph, so every    |
|                       | release walk's depth is a constant [PROV-6] and its storage is a term of     |
|                       | stack.entry [RES-5]                                                         |
| L9's displacement     | try_place and pool_release both hand the value back, and both refusals are   |
|                       | matched, so nothing is displaced silently                                    |
| stack bounded         | one context, one chain, measured after code generation [STK-3]              |
| runtime closed        | W = 1, no task or completion records; every runtime store's peak is zero     |
| retained              | the queue loop has a break, so it has a fallthrough entry and its retained   |
|                       | entry is empty; a variant with no break would publish its steady state       |
|                       | there rather than nowhere [RES-10], [STK-4]                                 |
```

#### The writer's-eye walkthrough

**`set (held, written) = render::<'a>(block: move held, task: &task);`** is the
statement three drafts could not write. Under the fourth draft `render` took a
`&uniq` container and [CNT-7] refused it. Under the fifth it took one and published
its post-state through an exit datum, which is the shape round 5 turned back into D1.
Here it takes the lease **by value and hands it back**, so what the caller learns is
`written <= len(rest.run)` about the value it now holds, and there is nothing a
callee could be wrong about. The `set` is [LIV-3]'s multi-target form at an arm
binder; `held` stays live through the commit, so [LIV-1]'s join agreement is met on
both arms of the enclosing `match`.

**`requires room(block.run) >= 8_u64;` is discharged by a dominating branch**, and
that branch is the honest price of the pool being library data rather than a kernel
store. A `Vector<'s, u8>` carries its capacity as a *measure* and not in its type
[BLK-1], so putting one into a `FixedVector` element and taking it out again loses
the figure `pool_new` established: the element type is `Vector<'s, u8>` for a run of
any capacity, and no clause `pool_take` could write would recover it. `let spare =
room(held.run); let big = spare >= 8_u64; if big { ... }` is one runtime branch per
lease, its first statement is a fact only because [ENT-3.S6] generalizes over the
four measures [BLK-0], and the branch is [BLK-0]'s own first mechanical fix. Round 5
found this obligation undischarged and this is what discharging it costs; 5.1's Q6
records that a container whose element capacity is in its type is the next candidate
and has to justify itself against this branch.

**`invariant slots: len(ring) + 8_u64 <= 256_u64` on the queue loop** is the second
of round 5's two findings against this program, and it is a header invariant because
it has to be. `drain`'s `requires count <= room(ring)` needs a fact about `ring` at a
point inside a loop whose body writes `ring`; [ENT-5] 2942-2946 removes every such
fact at the head, so a fact established before the loop is gone. The invariant is
proved in the state before the loop from `seq_fixed`'s `len(ring) = 0` and preserved
on the backedge from `drain`'s `ensures sent <= len(rest)` together with the
standing `len(ring) <= cap(ring)`. Writing it is the writer's job and the fifth
draft's program did not.

**Inside `render`**, whose two borrows and one brand are the only regions it names:

```wf-design
  for @fill (
    at in 0_u64..8_u64,
    invariant spare: room(block.run) + at >= 8_u64,
    invariant grown: len(block.run) >= at
  ) {
    set block.run = seq_place(vector: move block.run, value: mark);
  }
```

The **backedge** is the derivation the whole container surface rests on. The `set` is
an **in-place exchange** [LIV-3] at a **field of a linear value**, so by [MSR-3]'s
atom-identity sentence the root's [ENT-2] term survives, the facts over
`room(block.run)` and `len(block.run)` die by [MSR-2], and `seq_place`'s declared
`len(result) = len(vector) + 1` **and `room(result) = room(vector) - 1`** re-establish
them on the same term through [CALL-4]'s S12 destination clause. Each invariant is
preserved by **one** published premise; under the fifth draft, which published two of
three measures and left `room` to the standing identity, this derivation needed three
and [ENT-6] 3019 admits two, so this loop and every loop like it was refused.

**`let Lease(run: orphan) = move lost;`** is the destructuring consume [S13] on the
arm where the pool refused a lease back. It is one statement and it is *visible*,
which is the whole of R2: under the fifth draft `pool_release`'s refusal was an
`Option` a writer could bind and never match — 4.1 did exactly that — and the block
was gone for the life of the program with no diagnostic, no effect and no envelope
item. `Lease` is linear, so `Option<Lease>` is linear, so the arm must exist.

**The pool itself is affine and needs no dissolve**, and getting that right is a
modelling decision R2 forces rather than a detail. The obligation belongs on the
value that is *handed out*, so `Lease` is the linear nominal and `BlockPool`'s free
list holds bare `Vector<'s, u8>` runs, which are arena-backed and therefore affine.
`pool_take` constructs a lease around a run it takes out and `pool_release`
destructures one and puts the run back. Had the free list held leases, the pool would
have been linear by containment and an *empty* one would still have had no route out
— a run of a declaration-linear element type is linear whatever its length, and
neither `dispose` (no capability leaf) nor a destructuring consume (a run is not a
nominal) reaches it. 5.1's Q13 records that shape as the one R2 does not handle.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one library function with a typed
failure, disposal is one statement, the append helper takes the destination by value
and hands it back, and **not one region names the store**, because the program has
exactly one.

```wf-design
const ceiling: u64 = 4096_u64;

struct Bytes {
  v: Vector<u8>;
}

enum Grown {
  Grew(value: Bytes, room: u64);
  Refused(value: Bytes);
}

fn bs_new(heap: &uniq Heap) -> made: own Option<Bytes>
    reads(heap), writes(heap), allocates(heap) {
  doc "Builds one empty byte string over a zero-length backing run.";
  let taken = seq_heap::<u8>(heap: &uniq deref(heap), count: 0_u64);
  match taken {
    None() => {
      return None<Bytes>();
    }
    Some(value: run) => {
      let holder = Bytes(v: move run);
      return Some<Bytes>(value: move holder);
    }
  }
}

command fn main(command.stdout as sink: own Output, command.heap as heap: own Heap)
    -> status: own ExitStatus
    reads(sink, heap), writes(sink, heap), allocates(heap) {
  doc "Collects one fixed input run into a heap-backed run and writes it out, reporting a refusal instead of dying.";
  let input = filled::<u8, 4096>(value: 65_u8);
  let code = 0_u8;
  let made = bs_new(heap: &uniq heap);
  match made {
    None() => {
      set code = 70_u8;
    }
    Some(value: holder) => {
      let grown = bs_reserve(s: move holder, heap: &uniq heap, additional: ceiling);
      match grown {
        Grew(value: ready, room: spare) => {
          let total = 0_u64;
          let kept = move ready;
          region {
            let line = seq_span(vector: &input);
            set (kept.v, total) = collect(out: move kept.v, source: move line);
          }
          region {
            let body = seq_span(vector: &kept.v);
            let outcome = write_once(output: &uniq sink, source: &body, start: 0_u64, end: total);
            match outcome {
              Ok(value: next) => {
              }
              Err(error: problem) => {
                set code = 74_u8;
              }
            }
          }
          dispose kept using (heap);
        }
        Refused(value: back) => {
          set code = 70_u8;
          dispose back using (heap);
        }
      }
    }
  }
  return exit_status(code: code);
}
```

#### The writer's-eye walkthrough

**`bs_reserve` returns an enum and not a `Bool`.** The fifth draft's version returned
`own Bool` and 4.2 matched on it, which [GRAM-6] 285 makes a hard error outright
(probe `t12`) — and the deeper defect was that a `Bool` return from a fallible growth
tells the caller that the store refused and hands back nothing, which is the refusal
wearing a disguise L3 and L9 forbid. `Grown` carries the `Bytes` on both arms and the
new spare room on the success arm, and `bs_reserve`'s per-variant clause
`ensures when Grew(value: ready, room: spare): spare >= additional;` [S24] is what
makes the next line's `requires` dischargeable. `Grown` has a linear field, so it is
linear by containment, so neither arm can be dropped — which is why both arms end in
a `dispose`.

**`set (kept.v, total) = collect(out: move kept.v, source: move line);`** is R1's
central statement, at a **field** place. The relations reach `total` and `kept.v`
through [CALL-4]'s S12 clause over a `set` target list; under the fifth draft the
same line was `set total = collect(out: &uniq s.v, source: move line);` and published
**nothing at all**, because a plain `set` receiver is not one of [ENT-3.S12] 2833's
four destinations and [FN-9] 1357's narrow route requires the receiver to be an
argument. Round 5 verified both halves against the gate. `collect`'s
`requires len(source) <= room(out)` discharges from `bs_reserve`'s published `spare`
and `seq_span`'s `len(result) = <datum of len(input)>`, both of which name terms the
caller holds.

**`write_once(output: &uniq sink, source: &body, start: 0_u64, end: total)`** is
[VIEW-7] over a view. Its obligations are `0_u64 <= total`, implicit, and
`total <= len(deref(body))`, which discharges from [VIEW-2]'s
`len(body) = <datum of len(kept.v)>` and `collect`'s `ensures written <= len(rest)`
read at the ordinal `total` was bound to. This is the statement that makes goal A's
container half real. Its three regions all relate nothing, so all three are elided
(3.K.0); the inner blocks still exist because [OWN-10] 641 needs `body` bound before
the borrow, and they have no names.

**`dispose kept using (heap);` and `dispose back using (heap);`** are [PROV-6] [S12],
once per arm that holds a value. `Bytes` is a nominal with a field whose release
needs the `Heap`, so it is linear, so the `match` cannot be left with one alive on
either arm. The walk drops each `u8` element, which derives nothing, and then
releases the backing to the store `Vector<u8>`'s type names. `heap` is the entry's
own `own Heap` binding and needs no region, because `using` names a place. The walk's
depth is `Bytes`'s containment height, a constant, so no `wf_resource_abort` is
reachable from it. **There is no path on which the process disappears**, which is the
whole of goal B — and R2's cost is the two statements themselves, which is what 3.L.5
counts at seven for `byte_string.wf`.

#### What the compiler reports

```text
note: queue.wf is source-resource-closed; envelope written to queue.E
note: collector.wf is not source-resource-closed
  [RES-4] main selects command.heap
    heap-reaching path:  main -> bs_new -> seq_heap
  a general store cannot appear in an envelope [L6], so no envelope is computed
  still true of this program:
    no covered-resource failure is a trap [RES-6]; seq_heap returns a value
    the heap is reachable only through the parameter above [PROV-4]
    every release of heap-owned storage is a statement that names the heap [PROV-6]
    every release walk's depth is a compile-time constant [PROV-6]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

Five of the diagnostics the design owes a writer, each citing a rule that exists in
3.K. `SecondStoreInOneRegion` and `ConfinedFieldWithoutRegion` are stated inside
[PROV-1] and [BLK-4] and are not repeated here.

```text
Semantics/Source [BLK-0]: UndischargedOperationDomain
  operation: seq_place
  residual:  "0_u64 < room(block.run)"
  mechanical_fix: state a header invariant over room(block.run) [INV-1, MSR-5],
    dominate the place with a branch on room(block.run), take a larger run
    before the loop, or use the library's try_place

Semantics/Source [PROV-6]: LinearValueNotDisposed
  binding "kept" of type Bytes is live on the edge leaving this match arm
  its capability-released leaf Bytes.v of type Vector<u8> names the store region of
    command.heap
  mechanical_fix: move the value out of this scope, destructure it, or write
    dispose kept using (heap); a store-backed run has no compiler-derived release,
    so nothing else can free it

Semantics/Source [PROV-6]: LinearValueNotConsumed
  binding "lost" of type Option<Lease<'a>> is live on the edge leaving this match arm
  Lease is declared linear, so Option<Lease<'a>> is linear by containment
  no leaf of it requires a capability, so it cannot be disposed
  mechanical_fix: match it and, on the Some arm, return the lease to the pool or
    write let Lease(run: r) = move lost;

Semantics/Source [PROV-6]: LinearValuePartiallyConsumed
  "move chunk.page" takes one leaf out of "chunk" of type Chunk, which is linear
  the residual leaf Chunk.spare of type Vector<u8> would then leave this scope by
    neither a move, a destructuring consume, nor a dispose
  mechanical_fix: write let Chunk(page: p, spare: q) = move chunk; and handle both
    leaves, or dispose the whole value

Semantics/Source [PROV-5]: ExtentReservedOnACallCycle
  arena_extent::<65536, 16, 'p> is reached from a strongly connected component of
    the call graph, read after the tail rewrite [STK-1]: descend -> descend
  one committed extent would be held by every live activation at once
  mechanical_fix: reserve the store in the caller and lend the provider down
    [PROV-7], or use arena_frame, whose extent is per activation
```

The third is new in this draft and is the diagnostic Q0c asked for by name; probe
`x8` is the program that gets the last one.

---

## 5. Open questions

Everything the owner's rulings settle is dropped and not restated. So is everything
the earlier drafts asked and this one answers: the length-class terms and the goal
disposition are [MSR-1] and [MSR-4]; the arithmetic residual is [MSR-3]'s datums and
images; the coverage certificate died with `Builder`; the arena's reclamation is
[RES-5]'s cursor domain; the optimizer-versus-envelope question is [STK-3] and
[RES-2]; the profile table is [RES-2]. Eight questions earlier drafts filed are
**answered here rather than asked**, on the merits: a store is identified by its
region *per live activation of its region block* [PROV-1], [PROV-5]; a store-owned
value is destroyed by one structural statement whose walk is bounded by the type's
containment height [PROV-6]; a linear value is taken apart by `let N(f: a, ...) =
move v;` and a partial consume of one is refused; disposal needs no effect category
of its own; region-parametric nominals belong in this version [BLK-4]; the value-in /
value-out spelling gets an admission on `set` rather than a statement of its own
[LIV-3]; a ring needs no `Option` and no kernel rotation, because the run's own
typestate is a window [BLK-1]; and control entering the call graph from outside it is
1.5's, with the row the brand itself now owes.

### 5.0 The decisions the rulings forced, and the ruling's own question

These are the decisions the minimality ruling and R1 and R2 forced and the owner has
not separately ruled on. Each states what was traded and what the alternative costs.
**Every spelling any of them names is a proposal (3.S) and not a decision.**

**Q0a. `AppendView` and `absorb` are gone, and under R1 nothing is lost**
(footnote 3). The fifth draft recorded this as a trade: an `AppendView` could not
reach below its owner's length, so a callee handed one could not shrink what it was
given, and a callee handed `&uniq Vector<'s, T>` could. R1 removes the second half of
that sentence — there is no borrowed container parameter — and the guarantee itself
becomes an ordinary clause, `ensures len(rest) >= len(out)`, relating a result to an
input with no `old()`, no frame rule and no third view. *Recommend the trade, and
note that it is no longer a trade.*

**Q0b. A pool's lease is `linear` by declaration, and 4.1's leak is a compile
error.** Under the fifth draft an arena-backed lease was affine, a dropped lease
leaked a free-list slot until the region ended, and the design recorded the leak as
uncomfortable and bounded and visible in `E`. Round 5 showed it is none of those:
`E`'s items are [RES-2]'s shapes over [RES-5]'s stores and a library free list is
neither, so the loss is invisible in the fact state **and** invisible in `E`, and
"bounded by the pool's capacity" means the loss is total after that many iterations.
R2 closes it, and the cost is stated in 3.L.7: one modifier, one `match` on every
refusal, and a destructuring on every deliberate discard. *Recommend R2, with the
`must_consume` spelling of 3.S [S18] as the alternative the owner may prefer.*

**Q0c. Every heap-derived value in a hosted program is now disposed explicitly.**
That is R2's criterion applied honestly and 3.L.5 counts it at seven statements for
`byte_string.wf`. The alternative is an implicit scope-exit free, which would have to
reach a `Heap` the scope may not hold, so it is not available while L2 stands.
*Recommend it, and recommend that the doctrine say plainly that a region block or an
arena is how a writer writes fewer.*

**Q0d and Q0e together, and *recommend both*.** Five owners became two and
`FixedRing` became nothing at all (footnotes 1, 2): under [BLK-1]'s window a ring is
a run, which removes an `Option` word per slot, restores in-place element mutation,
and deletes eleven library items. `update` became an admission on `set` (footnote 4),
losing nothing and costing three fewer grammar atoms. And the kernel declares no
failure nominal ([BLK-2], [RES-6]), losing one shared vocabulary and gaining not
having compiler-owned nominals whose only job is to be a struct a writer could have
written.

**Q0f, the window's own trade.** [BLK-1] states the four costs and they are small,
but one of them is a *language* cost and should be weighed as one: `head` is a fourth
measure in every table and every row, and every subscript lowers to an add and a
conditional subtract unless an optimizer proves the head zero. The alternative is a
prefix plus a shifting `seq_take_front` written in the library, which costs O(n) per
removal and which L18 would keep out of the kernel anyway. *Recommend the window, and
record that a writer who never removes from the front pays one descriptor word and
nothing else.*

**Q0g is decided and is recorded rather than asked.** The region-spelling amendment
lands first, separately and mechanically, and is not this design's (3.K.0). What
this design owes it is one property — the spelling is decidable from the declaration
text alone — and what it gets back is measured in 3.K.0 and answers Q11. R1 adds one
identifier per hand-back helper, which 3.L.4 counts.

**Q0h, the ruling's own question: should any of 3.L ship?** The owner leans toward no
standard library at all, and 3.L proves the partition whether or not a line of it is
committed. Four items are load-bearing for this design's evidence — `filled` for
[VIEW-7]'s addressable destinations, `collect` for the append story, `vacant` for the
keyed families, and the pool for 4.1. *Recommend: no `std`; those four land as test
programs under `tests/programs/`, where a rot check already reaches them.*

### 5.1 The questions this design genuinely does not decide

**Q1. May a marked program handle a typed refusal, or must it prove every
acquisition?** **Permissive**: both spellings are admitted, since neither can ask for
more than `E`, and L8 plus [RES-6] make it real — a refusal edge carries the store's
own `room(store) == 0_u64`, and [RES-10]'s loop rule names the checked spelling as one
of the three things that bounds a retaining loop.

**Q2. Where does a hosted marked program's large memory come from?** **Frame and
extent placement only**, as [PROV-5] and [BLK-2] provide; an entry row delivering a
committed region becomes right the day a program needs a store whose *size* is a
deployment decision rather than a source constant.

**Q3. Does the range relation need `seq_split_at`?** Not in this version. The
relation it needs already exists in [PROV-3]; what is missing is only the row.

**Q4. How does a marked program reach a device?** `main`'s effect row names only its
own labelled inputs and the `command` table is closed, so 4.1 has a transmit ring and
no way to flush it. **A second program kind** under [FN-7]'s existing closed-table
discipline, arriving with the execution-context design of 1.5.

**Q5. When does `par` become usable inside a marked program?** [RUN-1] forbids the
emitted module a `par` construct and [RUN-2] publishes `lanes(1)`, because the
current runtime's wait path runs a stolen task on the waiting lane's own stack. The
answer is the compiler-managed work-first continuation representation, then lifting
the prohibition. **[PROV-5]'s activation refusal is written for that day**: its third
source names `par`, so lifting the prohibition does not reopen the extent
multiplicity round 4 found.

**Q6. Does this version want a keyed or sparse container family?** Not yet.
`CONTAINERS.md` §3.5 writes stable-identity storage as a vacant run plus
element-position `replace`, which is sound, L12-clean, and compiles in shape today
(probe `x7`). A `FixedTable<T, n>` whose typestate is an occupancy set is the next
candidate, and under L18 it has to justify itself against §3.5, which works.

**Q7. Should a system operation be able to append?** **Yes, in the batch that lands
[CALL-4]'s widened result vocabulary in the [SYS-2] declaration domain, and not
here.** Then the bytes the host wrote become the run's own `len` and the caller reads
it from the operation's `ensures`, instead of [VIEW-7]'s addressable destination and a
`u64` beside the run.

**Q8. Is `copy` structural over aggregates?** [OWN-1] 564 makes every owned composite
affine regardless of its field types, which is why 3.L.3's `filled` is written per
element class and why probes `m12` and `m14` disagree. **A `struct` or `enum` all of
whose field types are copy should be copy** — and the half that matters more here is
the second: **a generic body's `move` of a type parameter should be admitted at a
copy instantiation, where it is a no-op.** Without that half the first half does not
remove the wall, because the *template* is checked as if `T` were affine. Neither is
this design's to land, and together they are the difference between fourteen library
bodies and about forty.

**Q9. Is `E` part of program identity?** **An emitted machine-readable table beside
the object, carrying the module's content digest and explicitly not part of [PROG-2]
compilation-unit identity**, which [RES-2]'s three-argument form already says it is
not. The digest is what makes the table a promise rather than a document.

**Q10. Should a `propagate` carry a disposal?** [PROV-6] refuses a `propagate` while
a linear binding is live, and probes `w5` and `m03` show the language admits that
shape today. Under R2 the refusal is more common than it was, because more values are
linear. **Leave the refusal now**; a release list on the statement, checked by
exactly [LIV-1], should be paid for by a program whose rewrite was actually painful.

**Q13 is new, and is the one shape R2 does not handle.** A run whose element type is
linear *by declaration* is linear whatever its length, and it has no route out: it is
not a nominal, so the destructuring consume does not reach it, and it has no
capability-released leaf, so `dispose` does not either. A writer meets it the moment
they put a lease, a ticket or a transaction into a `FixedVector`. This design avoids
the shape by putting the obligation on the value that is handed out and not on the
container of spares (`CONTAINERS.md` §3.4), which is the right modelling and is not a
rule. **The principled fix is a fourth route — a *drained* consume, `let () = drain
v;`, admitted when the run's length is provably zero — or a rule that a run of length
zero holds no obligation.** Both are language additions and neither is proposed here;
until one exists the doctrine is the modelling rule above.

**Q11 is answered and is retained only as a record.** It asked whether a
view-forming borrow needs its own written region. It does not: the region relates
nothing, so the region-spelling amendment elides it and the enclosing block keeps its
braces and loses its name.

**Q12 is new.** [RES-6] states two ways to make `reserve_file` honest — a fallible
outcome at eleven corpus call sites, or a total operation over a proved capacity with
one header invariant per loop — and this design proposes the first [S25] while
recording that the second costs less at the call sites. **The owner should choose**,
because the choice is the same one every covered store's two spellings present and it
is the first time the language would apply it to a system operation.

## 6. Verified versus reasoned

**Verified** means a compiler executed it, against a gate-profile `whitefootc` built
from this tree, in this session or in one of the twenty falsifier sessions whose
probe names are quoted. Probes named `t1`-`t14` were run in this session against the
**v0.41** gate binary; probes named `t1`-`t14` are the same programs run against the
v0.40 binary earlier in this session, before the base changed, and every one of them
reproduced under v0.41 with the same rule and the same kind. No timing figure appears
anywhere in this file, and the known wrong acceptance of a `replace` at an arena
descriptor is a compiler defect counted as a design finding nowhere.

### 6.1 What the current compiler does

Fourteen programs were compiled in this session, each twice — once at v0.40 and once
respelled at v0.41 — and the verdicts agree. The table describes each closely enough
to rewrite it; the sources are session scratch files and are not in the repository.

```text
| probe            | program                                                        | verdict                                   |
|------------------|----------------------------------------------------------------|-------------------------------------------|
| t1               | `fn fill<const n: u64>()` reading `n` as a value in             | REJECTED [TYPE-5] UnresolvedUse           |
|                  | `buffer_new(n, 0_u8)`                                          | available: [ConstGeneric]                 |
| t2               | the same const generic as a `for` endpoint, `0_u64..n`         | REJECTED [TYPE-5] UnresolvedUse, same     |
| t3               | the same const generic as a clause operand,                    | REJECTED [TYPE-5] UnresolvedUse, same     |
|                  | `requires index < n;`                                          |                                           |
| t4               | a named const in all three of those positions                  | **ACCEPTED**, exit 0                      |
| t5               | `ensures len(result) >= len(out);` written directly            | REJECTED [GRAM-5] at parse                |
| t14              | `define kept = len(result);` on a measured result              | REJECTED [TYPE-5] UnresolvedUse,          |
|                  |                                                                | role PlaceBase, available: []             |
| t6               | `ensures when Some(value: got): got >= value;` on an Option     | REJECTED [FN-9]                           |
|                  |                                                                | InvalidPostconditionSelector              |
| t7               | `fn split(...) -> (low: own u64, high: own u64)`               | REJECTED [GRAM-2] at parse                |
| t8               | `set c = bump(cell: move c);` at a live affine local           | REJECTED [STOR-1] AffineSetTarget         |
| t9               | D1 verbatim: `replace deref(handle)` in a callee through        | **ACCEPTED**, exit 0                      |
|                  | `&uniq 'a buffer<u8>`, caller subscripts offset 9              |                                           |
| t10              | a callee reading `len(deref(h))` through `&uniq` and            | REJECTED [EFF-2] EffectMismatch,          |
|                  | declaring `pure`                                               | missing: ["reads(handle)"]                |
| t11              | `Z` as a written contract operand                              | REJECTED [GRAM-5] (v0.41) / [FORM-3]      |
|                  |                                                                | (v0.40) at parse                          |
| t12              | `match g {` on an `own Bool`                                    | REJECTED [GRAM-6] InvalidConditionalForm  |
| t13              | `let mirror = count -wrap 1_u64 -wrap at;`                     | REJECTED [GRAM-4] at parse, expected ";"  |
```

What each establishes, and which rule it decided rather than confirmed. **t1, t2, t3
against t4** are [MSR-6] and are the eighth item of 3.L.6: a const generic is
admitted where a const *argument* is expected and nowhere else, while a named const
works in all three positions, so the addition is a domain-table row and not new
machinery. **t5 and t14** are [MSR-5] and [CALL-4]'s result operands, at the parse
level and at the resolution level respectively; together they show that a `define`
cannot buy what a result operand needs. **t6** is [CALL-4]'s per-variant route.
**t7** is the ordered result list. **t8** is [LIV-3] and, with the [STOR-1] 679
reading, the rule this draft repairs. **t9** is D1 at this tip, still accepted.
**t10** is [EFF-2]'s both-ways check, which is why every signature in 3.L and section
4 carries the row its body exhibits. **t11**, **t12** and **t13** are three of round
5's text defects in the fifth draft's own library, each executed.

Inherited verdicts this draft still rests on, from the twenty falsifier sessions, by
what each group establishes. [CALL-1], [CALL-2] and [CALL-5] already behave and the
struct-field route already kills correctly (`p1`, `p6`, `f7`, `m04`, `s7`). `MutSpan`
writes, affine elements and multi-return are new capability rather than compiler
defects (`p7`, `p9`, `k12`, `p2`, `p8`, `k09`, `r1_multi`). Allocation while holding
nothing, and a free inside a `pure` callee, are accepted (`p5_ambient`, `n4`,
`r1_ambient`, `r2_5`, `q9`, `w7`, `m02`) — L2's and L13's evidence. A view value, not
its argument borrow, must hold the loan (`f1c`, `f1d`, `f2b`, `r1_twouniq`, `r2_1`,
`r2_2`, `c4`, `w8`). [LIV-1] replaces three avoidances (`f3`, `f5`, `f6`, `r1_own11`,
`s5`, `s6`) and [LIV-2] has two halves (`p10`, `w6`). The syntactic tail conditions
are refuted (`f2b_tail`, `f8_tailframe`, `p3_rec`) and the idle and driver loops are
`FunctionFallthrough` (`n2_idle`, `f3_forever`, `k30`, `n3_propagate_loop`). [BLK-4]
is new syntax (`f7_regionresult`, `r2_6`, `m05`). The measure kill is root-granular
today (`r2_4`, `r2_4b`, `r2_4c`); element-position replace keeps a `len` (`r2_7`,
`k24`, `n13`); a partial move kills the root and its residual is freed (`q3`, `q7`,
`x4`, `g7`, `p6_partial`); no loop publishes `len = N` as an equality (`n14`, `n15`,
`n19`); a by-value transformation is not `pure` (`c8`); [PROV-7] has a reason
(`r1_relend`, `r1_relend_affine`, `m19`); the fill loop's arithmetic and its
two-invariant and three-invariant shapes are accepted (`k21`, `k21b`, `k08`, `k31`,
`x1c`, `x1d`, `g4`) and the three-term header without a published relation is not
(`g3`); `+checked` publishes only for a constant addend (`g1`, `g2`); the
arena-content stop, the recursive region and the release walk's `realloc`'d worklist
with its `wf_resource_abort` are all executed (`a1`, `a5`, `a6`, `a8`, `x6`, `x8`,
`p2_recarena`, `p3_rectype`); `reserve_file` lowers to `ret i1 true` and the io_uring
adapter reserves an entry on every submission (`p1_reclose`, and the three source
reads of 6.2); and `par` eligibility plus three disjoint chain roots are the ledger
read (`n7_par`, `--stack-ledger`).

### 6.2 The runtime sources this design reads

Three reads, because [RES-7] and [RES-9] are stated over them and round 5 showed the
fifth draft's column contradicting all three.

```text
| source                                                   | what it shows                                   |
|----------------------------------------------------------|-------------------------------------------------|
| completion/linux_io_uring.c:425-450, 587-640             | every submission calls wf_linux_reserve_entry   |
|                                                          | on a fixed entry_capacity table and waits when  |
|                                                          | it is full                                      |
| completion/bridge.c:660-720, 900-1240, 1504              | read_at, write_once, open_file, open_read,      |
|                                                          | open_directory, open_directory_source and       |
|                                                          | directory_next all take that path; close is     |
|                                                          | direct; the adapter initializes under           |
|                                                          | pthread_once inside the submit path             |
| emitter/system.rs:2892-2901, backend/wf_floor.c:238-329  | reserve_file lowers to `ret i1 true`; the floor |
|                                                          | creates the entry stack and falls back silently |
```

The first two are why [RES-7]'s column is derived from the `may-suspend` contract and
reads a store for eight operations rather than `none` for sixteen. The third is why
[RES-9]'s store is a design addition rather than a compiler defect, and why [STK-3]
materializes the entry stack instead of reading it.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in 3.K.** None is implemented, and no compiler has seen any of the
  new types, operations, terms, statements, modifiers or markers.
- **Every proposal in 3.S.** No spelling in this file has been ruled on.
- **Every function in 3.L** and **every program in section 4**, written against 3.K
  and the unchanged v0.41 rules and walked against both, with each loop's head state
  stated; none was compiled, because 3.K is not implemented.
- **Every figure in 4.1's envelope**, which is why every one is written as a
  composition or as `<post-codegen>`.
- **[PROV-1]'s brand.** Argued from rule text — and all five falsifier rounds
  attacked it from every position they could build and none moved it, which is the
  strongest evidence any part of this design has.
- **3.K.0's two criteria.** That the declaration criterion and the call-site
  criterion together decide every position, that both are local to one declaration,
  and that resolving them before [TYPE-5]'s check changes no judgment are argued and
  not executed. The claim most worth attacking is that the two never disagree at one
  position.
- **[PROV-5]'s activation refusal.** That SCC membership read after [STK-1],
  execution contexts including lanes, and `par` reachability together cover every way
  two activations can be live at once is argued from four rules and not executed.
- **[PROV-6]'s criterion and its walk.** That "release requires a capability"
  partitions the same set the fifth draft enumerated, that the containment height is
  the right bound, and that the cyclic refusal at the declaration costs no program a
  writer can otherwise write are argued. Probe `a8` is the mechanism the walk
  replaces and probes `a5`/`a6` are the shape it keeps.
- **[BLK-1]'s window.** That `head` costs exactly the four things [BLK-1] lists, that
  no rule outside [VIEW-2] needs it, and that a wrapped window is the only thing view
  formation must refuse are argued and not executed. This is the newest structure in
  the draft and 6.4 asks for it first.
- **[RES-10]'s algebra.** Its sequence, branch and call rules over a label map are
  standard, the no-fallthrough case is defined, the interval arithmetic is stated,
  the loop's own map is stated per discharge, and `retained` and `reset` are new. Its
  `par` rule depends on a runtime profile that does not exist, and neither `retained`
  nor `reset` has been composed against a program by hand.
- **The compiler defect at `[SET-2]`'s arena half**, found in round 3 and confirmed
  since: [SET-2] 517 makes a region-bearing `replace` target a hard error for
  `slice<'r, U>` **and** `arena<'r, U>`, and `check_mutation_target_class`
  (`compiler/src/semantic/check/expressions.rs:310-326`) tests only the slice
  variant. It is benign at this tip and load-bearing for the batch that implements
  [PROV-3] use 3 and [VIEW-4], which must be relations over loan-bearing types and
  not a re-wording over one `CheckedType` variant.
- **[MSR-3]'s three placements**, checked by enumeration and not by execution; **the
  current runtime's closure**, which no existing target can be certified to meet; and
  **the claim that `wfgrep` becomes heap-free**, whose substitution was never
  compiled and which moves bytes out of the heap into frames, a [STK-3] question
  rather than a free win.

### 6.4 Falsifiers this design asks for next

1. **Attack [BLK-1]'s window**, which is the newest structure here and the one that
   changes the most: find a rule outside [VIEW-2] that needs `head`, a program in
   which the four costs are not the whole bill, a wrapped run reaching a system
   operation, or a subscript whose lowering the `head` term makes unprovable.
2. **Write 3.L against 3.K by hand, one function at a time, and find the ninth
   kernel addition.** Round 5 found three by writing four functions carefully; the
   yield per function is high and the design has not exhausted it.
3. **Attack R1 itself**: find a capability a `&uniq` container parameter had that
   value-in / value-out does not, or a program in which the one written region name
   per hand-back helper compounds.
4. **Attack [PROV-6]'s criterion** with a value whose release requires a capability
   the criterion does not see — a system resource whose close needs a factory, a
   value whose release is conditional — and with a `dispose` of a
   `FixedVector<Option<T>, n>` whose slots a join left in different variants.
5. **Hand-execute [RES-10]** on 4.1, on `CONTAINERS.md` §3.4's pool, and on a
   divergent service loop, checking `retained`, `reset` and the loop's own map
   against all three.
6. **Attack the `linear` modifier** with a type that is linear by declaration and
   affine by criterion inside a generic, with a linear value in a `par` footprint,
   and with a library whose linear nominal crosses a boundary a writer does not own.
7. **Rewrite `wfgrep` and `byte_string` by hand** against [VIEW-7], [PROV-6], R1,
   3.K.0 and Q10's refusal, and count what remains.
8. **Attack 3.K.0's two criteria** at a position where a declaration relates two
   positions *and* an operand determines the same argument, and at a nominal whose
   region parameter and the entry heap are both candidates.
### 6.5 Falsifier round 1: what each finding hit, and what refuses it now

**6.5 to 6.8 are carried unchanged from the fifth draft except where round 5 showed a
disposition false**, and they are written in the vocabulary of the draft that made
them: a right-hand column may name a rule this draft has since deleted ([SEQ-0],
[CNT-*], [VIEW-3]), a subsection that has been renumbered (3.3.1 is now [RES-10]), or
a spelling v0.41 has respelled. That is history and is left alone; where a
disposition is *false about its own draft*, 6.8 says so in the row.

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
| F2-NA10 BREAKS 1.4's "nothing has to be reopened" is false      | 1.4 rewritten — and the count in the fifth draft's own      |
|                                                                 | disposition was wrong: its table had two rows inherited and |
|                                                                 | three owed, not three and three. 1.5 now has two and three, |
|                                                                 | one of which the accounting repair makes stateable          |
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

### 6.8 Falsifier round 4: what each finding hit, and what refuses it now

Every BREAKS, GAP, DEFECT, BLOCKING and FRICTION finding of the four round-4
reports, one line each. Round 4's diagnosis was that the fourth draft closed store
*identity* under every value-forming step and left three other notions open — a
store's **activation**, a store's **release**, and the three type-level predicates it
introduced without the same closure. The owner's minimality ruling arrived in the
same week and dissolved a second class of finding outright, by moving its subject
out of the kernel; those rows say `moved to 3.L` and name the function that now
carries it.

```text
| finding                                                        | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| F1-a1 BREAKS recursion makes two entries of one region block    | [PROV-1] states the invariant as one live store per region  |
|   live at once over one extent-form store                       | per program point; [PROV-5] refuses an extent occurrence on |
|                                                                 | a call-graph SCC, and says why the frame form is safe       |
| F1-a2 BREAKS a partial move abandons a linear leaf that neither | [PROV-6]: a partial move of a linear value is a hard error, |
|   [LIV-1] nor [PROV-6] sees                                     | and `let N(f: a) = move v;` is the route out; probe x4      |
| F1-a3 GAP [ENT-5] 2887(a)'s element-position carve-out is not   | [MSR-2]: the carve-out is REMOVED, not narrowed; the kill   |
|   narrowed, and [MSR-1] made element positions hold descriptors | is the plain overlap test and the four consequences derive  |
| F1-a4 GAP an `update`'s relations have no [ENT-3.S12]           | moved: `update` is gone, and [LIV-3]'s exchange states its  |
|   destination, and 4.1 patches it by citing [LIV-2]             | own S12 clause; [MSR-3]'s one atom-identity sentence covers |
|                                                                 | both writing forms, so 4.1's derivation is true as written  |
| F1-a5 GAP the handle table has an acquire event and no release  | [RES-9]: [SYS-10] and [SYS-2] 2295 are amended and the      |
|   event                                                         | release row gains the covered store as a second subject     |
| F1-a6 GAP a non-linear leaf's derived release is invisible in   | [PROV-6]'s effect contribution is the union: each named     |
|   the operation that runs it                                    | provider plus the release row of every non-linear leaf.     |
|                                                                 | `seq_clear` and `seq_truncate` are moved to `CONTAINERS.md` §3, where a  |
|                                                                 | user function's release contribution is [EFF-2]'s ordinary  |
|                                                                 | one                                                         |
| F1-a7 GAP nothing forbids a provider in stored content, and     | [PROV-2]: a provider type is inadmissible in every stored   |
|   `dispose p using (p.q);` is unjudged                          | position, on [BLK-4]'s third clause, for a disposal-order   |
|                                                                 | reason stated there                                         |
| F1-a8 GAP [MSR-1] and [MSR-5] give opposite reasons for the     | [MSR-1] decides it once, by position: a program position    |
|   [OP-4] obligation on a subscripted measure place              | discharges at its program point, an erased clause at its    |
|                                                                 | own attach site                                             |
| F1-a9 DEFECT [EFF-1] 1380 is unregistered, so a view            | registered on the [EFF-1] row under [PROV-3], with          |
|   parameter's effect path names the descriptor                  | [CALL-3] and [VIEW-7] carrying `Depends:` lines             |
| F1-a10 DEFECT the register's condition 4 cannot catch a         | condition 4 becomes 4a and 4b, and condition 5 is added;    |
|   `Depends:` whose subject type was retired                     | [OWN-10] 638 and [ENT-5] 2936-2940 are now carried          |
| F1-a11..a20 HOLDS (the brand from every type-level route, the   | preserved and not weakened; the brand's invariance argument |
|   reset, `dispose` six ways, hiding a linear value, the datum   | is unchanged and 6.3 still names it as the claim most worth |
|   across a join, `update ... into`, a clause over a killed      | attacking                                                   |
|   measure, `seq_vacant`, a box behind `&uniq`, `par` over a     |                                                            |
|   `MutSpan`)                                                    |                                                            |
| F1 ranked 10 DEFECT L17's ground and [PROV-6]'s walk exclude    | L17 gains the whether/which split; [PROV-6]'s walk is       |
|   enums                                                         | stated over the variant structure                           |
| F2-NB1 BREAKS the disposal walk's scratch is a heap store that  | [PROV-6]: the walk's depth is the type's containment        |
|   aborts                                                        | height, a constant, so it needs no auxiliary storage;       |
|                                                                 | [RES-5] gains the cleanup-scratch domain for the cyclic     |
|                                                                 | case; [RES-6] corrects the false claim about the abort      |
|                                                                 | site's last caller; probes x6 and a8                        |
| F2-NB2 BREAKS recursion re-enters an extent occurrence          | as F1-a1                                                    |
| F2-NB3 BREAKS the handle table has no admissible fact source    | [RES-9] amends [SYS-2] 2295; [MSR-2] makes a profile        |
|   and its cap dies at the first call                            | symbol a standing fact with empty support                   |
| F2-NB4 BREAKS [SYS-10] denies the store and is unregistered     | [RES-9] amends [SYS-10] 2548-2552 and registers it          |
| F2-NB5 BREAKS an arena's reset runs no content release          | [PROV-6]: a store's storage reclamation never stands in for |
|                                                                 | its content's release; the two actions are split; probe a1  |
| F2-NB6 BREAKS the residual of a partial move                    | as F1-a2                                                    |
| F2-NB7 BREAKS [RES-7]'s test is a source rejection over         | [RES-7]: a target-independent "acquires from" column of the |
|   [QUAL-1] target data                                          | [SYS-2] record, with [QUAL-2] carrying the target half      |
| F2-NB8 GAP no `(peak, delta)` for a compiler-derived release    | 3.K.7.1 gains the derived-release primitive transfer        |
| F2-NB9 GAP the loop's own map is never stated for max(d) > 0    | 3.K.7.1 states peak(loop) and delta(loop) per discharge     |
| F2-NB10 GAP route (ii) does not compose across a call           | [RES-8] gains a per-domain saturation flag, derived from    |
|                                                                 | declared rows and not from a body, so [CALL-5] is respected |
| F2-NB11 GAP K<T> is a ceiling where an exact advance exists     | **FALSE as stated**: the fifth draft's exact form named     |
|                                                                 | len(arena), a runtime cursor, which [RES-3] forbids in a    |
|                                                                 | bound, and attached its requirement to `arena_take`, an     |
|                                                                 | operation in no inventory. [RES-5] now rounds the cursor to |
|                                                                 | the store's own align at every take, once per run           |
| F2-NB12 GAP [RUN-2] is stated over an implementation's choice   | the no-permission sentence moves into [RUN-1]'s             |
|                                                                 | qualification obligation; [RUN-2] keeps the published row   |
| F2-NB13 GAP the [OP-9] rows disagree about the lease            | moved to Appendix A.1, derived from [BLK-1]'s storage       |
|                                                                 | column rather than written per nominal; the lease is gone   |
| F2-NB14 GAP [STK-1]'s target obligation has no subject          | [STK-1] names the lowering: one dispatcher, one frame, no   |
|                                                                 | transfer, and the ABI obligation is deleted                 |
| F2-NB15 GAP the entry stack is never checked; E is not a        | [RUN-4] gains the entry-stack comparison; [RES-2] makes E a |
|   function of (P, T)                                            | function of program, target and build                       |
| F2-NB16 GAP `update` and `dispose` deny every [PAR-1] window    | `update` is gone and the exchange is an ordinary `set`;     |
|                                                                 | [PAR-1] 1990 gains `dispose` [RUN-3]                        |
| F2-NB17 GAP premise 3 never compares peak to cap                | [RES-3] states what premise 3 is for and where the          |
|                                                                 | no-overdraw guarantee actually comes from                   |
| F2-NB18 the three re-entry answers                              | stated in [PROV-5] and [STK-4]                              |
| F3 defect 1 [PROV-3] use 3's two conditions disagree            | split: use 3 is storage-keyed and says nothing about the    |
|                                                                 | descriptor; [LIV-3] is the consume-keyed rule; [VIEW-4]'s   |
|                                                                 | bare-`MutSpan` case is the second bullet's                  |
| F3 defect 2 the carried formation datum has no transport        | [PROV-3]: the datum rides the origin record, which [FN-1]   |
|                                                                 | 1035-1041 already substitutes at a call boundary — and      |
|                                                                 | then `AppendView` is deleted, so nothing needs it           |
| F3 defect 3 `linear` is a third class beside affine             | [PROV-6]: linear REFINES affine; [OWN-1] 558-559 unchanged  |
| F3 defect 4 `NoRecord<unit>` is undeclared and uncounted        | deleted: the kernel declares no failure nominal at all      |
|                                                                 | [BLK-2], and reserve_file's refusal is [SYS-7]'s existing   |
|                                                                 | ResourceExhausted with a published relation [RES-6]         |
| F3 defect 5 `relation_op` is undefined, and [OP-5]/[FN-8]       | [MSR-5]: the relation is an IDENT on [INV-1] 3099's model,  |
|   demand a type the production has not                          | restricted to [FN-9] 1306's closed root set, with [OP-5]    |
|                                                                 | 921 and [FN-8] 1261 amended and registered                  |
| F3 defect 6 three rows violate L15 on a refusal edge            | L15 gains the exact/monotone split; [MSR-1] gains the       |
|                                                                 | column; [BLK-0] requires completeness on every exit         |
| F3 defect 7 `cap` and `room` reads never become facts           | [BLK-0]: [ENT-3.S6] 2779 generalizes over the three         |
|                                                                 | measures — and 3.L.6 records it as one of the seven the    |
|                                                                 | library could not be written without                        |
| F3 defect 8 [PROV-5]'s refusal reads [PAR] permission           | the [PAR] clause is DELETED; the SCC clause replaces it and |
|                                                                 | the ordinary footprint rules cover the parallel case        |
| F3 I1 five third-list rows have no `Depends:` line              | every third-list row is now produced by one                 |
| F3 I2 requires_clause/ensures_clause are [GRAM-2] 182-183       | registered on the [GRAM-2] row                              |
| F3 I3 five wrong line numbers                                   | corrected in the fifth draft — and round 5 found **eight    |
|                                                                 | more** wrong and seven ranges still overshooting, three of  |
|                                                                 | them ranges this row claimed corrected. 3.K.11 re-derives   |
|                                                                 | every citation against v0.41 and states the eight           |
| F3 I4 four ranges overshoot a blank line or a heading           | **FALSE as stated**: only [SCOPE-3] was right; [OP-9],      |
|                                                                 | [FN-9] and [PAR-3] each still ended one line past their     |
|                                                                 | rule, and four new ranges overshot the same way. Every      |
|                                                                 | range in 3.K.11 now ends on its rule's last nonblank line   |
| F3 I5 [OP-1] 833 and [TYPE-6] 396 are not registered            | both are on their rows under [BLK-0]                        |
| F3 I6, I7 [SEQ-0] is in three batches; two B3 tests need B5     | [SEQ-0] is deleted; section 7 is re-derived and every rule  |
|                                                                 | is in exactly one batch                                     |
| F3 I8 [MSR-1]'s table has no buffer<T> row, so len(buffer)      | moot: [MSR-1]'s table is Appendix A.1 and `buffer<T>`       |
|   stops being a term three batches before buffer retires        | retires in the same batch that introduces the runs          |
| F3 I9 the [GRAM-3] row's `by` names a rule that does not amend  | the row is by [PROV-1] alone, whose Amends line reaches it  |
| F3 I10 "five such dependencies" is one short                    | the count is dropped; each condition-4 row states itself    |
| F3 I11 the [STOR-1] row cites 674 alone                         | the row is 670-677                                          |
| F3 I12 the provider-first exception is misstated                | [BLK-0]'s sentence is over an operation that transforms     |
|                                                                 | nothing, and a reservation takes no parameters at all       |
| F3 I13 C:39 cites [VIEW-6] for the absorb commit                | moot; `absorb` is deleted, and CONTAINERS.md is rewritten   |
| F3 I14 [OWN-9] 633 is in the unchanged list and describes its   | [OWN-9] is not in either list; it is non-normative and no   |
|   own change                                                    | rule depends on it                                          |
| F3 I15 two bare "unchanged" claims inside changed rows          | each is now a condition-4a citation with its dependant      |
| F3 I16 C:269-271's invariant base is derived from the caller    | rewritten in CONTAINERS.md against 3.L.4                    |
| F3 I17 3.3.1's route (ii) reads which premise discharged a goal | 3.K.7.1's route (ii) reads a property of the acquisition    |
|                                                                 | and [RES-8]'s saturation flag, both declared data           |
| F3 I18 [CALL-4]'s examples disagree with section 4's            | **FALSE as stated**: the fifth draft's [CALL-4] still wrote |
|                                                                 | `collect` with three regions and a `render` of a different  |
|                                                                 | mode, arity and result shape from 4.1's. R1 rewrites both,  |
|                                                                 | and [CALL-4]'s examples are now byte-identical to 4.1's     |
| F3 I19 [VIEW-4]'s ground covers only a borrowed descriptor      | the bare `own MutSpan` case is the second bullet's          |
| F3 N1..N9 nine notes                                            | N1 moot (absorb deleted); N2 **FALSE as stated** — no rule  |
|                                                                 | of the fifth draft cited [OWN-5] 583 anywhere; [CALL-1] now |
|                                                                 | cites [OWN-5] 585-606 as the ground it depends on; N3       |
|                                                                 | is on the [ERR-3] path and stated in the rule; N4, N5       |
|                                                                 | fixed in CONTAINERS.md; N6 moot; N7 [CALL-4] states the     |
|                                                                 | entry/exit split per clause; N8 result_binding is on the    |
|                                                                 | [GRAM-2] row; N9 [QUAL-2] is a changed row under [RES-7]    |
| F4-1 BLOCKING [PROV-6] is undefined on a partially moved        | as F1-a2: the destructuring consume plus the refusal        |
|   linear aggregate, and a slab free list has no spelling        |                                                            |
| F4-2 BLOCKING round 3's finding 5 is answered twice             | [MSR-3]'s one atom-identity sentence, and `update`'s        |
|                                                                 | removal leaves one writing form per case                    |
| F4-3 FRICTION no fact about a borrowed run's post-state crosses | [CALL-4]'s exit datum over a `&uniq` parameter, which is    |
|   a call                                                        | also what deletes `AppendView`; probe x10                   |
| F4-4 FRICTION [CNT-7] is nullified by a one-field wrapper       | [CNT-7] is DELETED; `&uniq Vector` is admitted and          |
|                                                                 | [CALL-5] kills; probes m04, x11                             |
| F4-5 FRICTION the fallible rows are unreachable from `update`   | moved to 3.L: growth is a library function, and [LIV-3]'s   |
|                                                                 | multi-target form reaches every two-result row at every     |
|                                                                 | place                                                       |
| F4-6 FRICTION the brand's hosted tax: 37 items in an            | the region-spelling amendment, which lands first and         |
|   11-function program                                           | separately (3.K.0); measured again on this worktree there   |
| F4-7 FRICTION reserve_file's second failure channel             | as F3 defect 4                                              |
| F4-8 FRICTION [VIEW-7]'s two regions per I/O site               | both are elided under 3.K.0; Q11 is answered                |
| F4-9 FRICTION five text and decision gaps                       | `construct OutOfMemory<T>` is moot (no kernel failure       |
|                                                                 | nominal; a library one is an ordinary struct);              |
|                                                                 | `dispose item using (deref(heap));` is written in [PROV-6]  |
|                                                                 | and in `CONTAINERS.md` §3; [PROV-7] is stated generally; const gparams   |
|                                                                 | are lowercase IDENTs per [FORM-3]; a branded type inside a  |
|                                                                 | written type argument is admitted by [BLK-0]                |
| F4-10 FRICTION two prices that should be stated                 | both are stated in [PROV-6] and in `CONTAINERS.md` §3                    |
| F4-11 CLEAN [PROV-1]'s totality, [PROV-6]'s walk, seq_vacant,   | preserved and not weakened; the brand survives a fourth     |
|   [MSR-2], [RES-7], [STK-4], [CALL-1/2/3/5], [MSR-4]            | round and is the one thing no report could move             |
```

**Where a fourth-draft rule's content went**, for a reader holding 6.5 to 6.7:

```text
| fourth-draft rule            | now                                                     |
|------------------------------|---------------------------------------------------------|
| [CNT-1] the owner inventory   | [BLK-1] (two runs) and 3.L.1, `CONTAINERS.md` §3                     |
| [CNT-2] typestate, seq_vacant | [BLK-1] and 3.L.3, `CONTAINERS.md` §3                               |
| [CNT-3] affine elements       | [BLK-1]                                                 |
| [CNT-4] confinement           | [BLK-4]                                                 |
| [CNT-6] growth is owner-level | `CONTAINERS.md` §3, and L4 unchanged                                 |
| [CNT-7] no &uniq container    | deleted; [CALL-5] refuses the shape it was protecting   |
| [SEQ-0] the declaration domain| [BLK-0], with the inventory in Appendix A.2             |
| [VIEW-3] absorb               | deleted; [CALL-4]'s exit datum                          |
| [VIEW-5] the abandoned window | deleted with AppendView                                 |
| [LIV-3] update                | [LIV-3], as an admission on `set`                       |
| [PROV-1]'s elision paragraph  | 3.K.0, and the separate region-spelling amendment       |
| [STK-5] stack exhaustion      | [RES-4]                                                 |
| L14                           | retired                                                 |
```

### 6.9 Falsifier round 5: what each finding hit, and what refuses it now

Every BREAKS, GAP, DEFECT, BLOCKING, FRICTION and INCONSISTENCY finding of the four
round-5 reports, one line each. Round 5's diagnosis was one sentence in four voices:
*a notion was introduced without the closure the brand got*, and the open notion this
time was **accounting**, with **measure data**, **linearity** and the **elision
assumption** beside it. §2.1 is this draft's answer to the pattern; the rows below
are its answer to the instances. The reports are superseded.

**The base moved under this draft and it changed nothing here.** v0.41 respelled the
six integer comparisons as infix, delimited call-site type application with `::`, and
put the four ordered symbols in proof position. Every round-5 finding was re-checked
against the respelling and every one survives: the fourteen probes were re-run
against the v0.41 binary with the same rule and the same kind (6.1), and the only
finding whose *text* changes is F3 defect 8, whose `Z` rejection moves from [FORM-3]
to [GRAM-5]. The design's own surface is written in v0.41 throughout, and [MSR-5] is
smaller for it: the clause relation is now the same infix form an invariant already
uses, so what the rule adds is the operand set alone.

```text
| F1 (memory and fact soundness)                                 | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| 1 BREAKS the exit datum is a caller-side object with no callee- | **R1**: withdrawn. [MSR-3] has three placements and every   |
|   side placement, so the callee proves the entry fact and the   | one is a point the forming function can read; a helper      |
|   caller reads it as the exit fact — D1 restored                | takes the run by value and publishes on its result [CALL-2] |
| 2 BREAKS dispose has no operand condition and contributes no    | [PROV-6]: dispose is **a consume and a write**. The consume |
|   write, so a callee frees a caller's run through a shared      | half needs an own-mode root of this function [OWN-1], the   |
|   borrow that [CALL-1] guarantees kills nothing                 | write half makes [MSR-2] and [CALL-1..3] see it            |
| 3 BREAKS a partial dispose abandons a leaf, and the refusal is  | [PROV-6]: the refusal is stated over the **consume**, and   |
|   stated over `move`                                            | [OWN-1] 569 names dispose as a consuming use                |
| 4 GAP the three-measure backedge needs three affine premises    | [BLK-0]: every row publishes every measure it writes,       |
|   and AUTO admits two, so every appending loop is refused       | exactly, on every exit. [MSR-4] is not widened; probes g3   |
|                                                                 | and g4 locate the fault in the rows, not in the prover      |
| 5 GAP seq_frame reserves a store [PROV-5] does not name         | the row is **deleted** ([BLK-2]); it was a duplicate of     |
|                                                                 | seq_fixed and it violated four closure sentences            |
| 6 GAP the elision candidate set is empty at a parameter         | 3.K.0: two criteria, one per position kind, and the         |
|   position in a heap-free program                               | parameter set is an implicit region parameter, never empty  |
| 7 GAP bs_reserve is refused twice and its route is false        | `CONTAINERS.md` §3.3 rewritten: the window drains from the  |
|                                                                 | front, so the @flip loop is gone; the +checked route is     |
|                                                                 | replaced by cap(built), which seq_heap publishes            |
| 8 GAP 4.1 has two undischargeable requires                      | 4.1 rewritten: pool_take publishes room per variant [S24],  |
|                                                                 | and the queue loop carries a header invariant over the ring |
| 9 DEFECT [LIV-3] states no effect footprint                     | [LIV-3] states it: [SET-2]'s, one read and one write of the |
|                                                                 | target's ultimate storage origin, plus the call's own row   |
| 10 GAP a multi-target set's later targets have no scope, mode   | [LIV-3]: each is an ordinary let binding introduced at the  |
|   or release rule                                               | statement, and a declaration event [MSR-3]                  |
| 11 GAP seq_exchange's permutation is unstatable                 | moot: the row is **deleted** (3.L.2 writes the swap), so    |
|                                                                 | the prose that could not be a declared relation is gone     |
| 12 GAP Q0c is larger than 5.0 records                           | **R2**: linear by declaration; Q0b restates the loss and    |
|                                                                 | 4.1's leak is a compile error                               |
| 13, 14, 15 HOLDS ([LIV-3]'s routes, the &uniq door, the         | preserved and not weakened; 15's two unstated premises —    |
|   construct placement)                                          | a result binding is a place, and S12 substitutes inside a   |
|                                                                 | term — are stated in [CALL-4] [S24]                        |
| 16 DEFECT register condition 5 is violated by [LIV-3] itself    | the [SET-2] row states it: "establishes no fact" becomes    |
|                                                                 | false for the exchange                                      |
| part 2 finding 1 GAP [PROV-5] lost its par disjunct while       | [PROV-5] states the **property** and names three sources,   |
|   gaining another                                               | including par, and reads the graph after [STK-1]            |
```

```text
| F2 (resource-closedness)                                       | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| F5-1 BREAKS seq_frame's region belongs to no closure            | deleted ([BLK-2]); as F1-5                                  |
| F5-2 BREAKS a divergent loop's map is empty, so a service       | [RES-10]: the label set gains **retained**, and [STK-4]'s   |
|   loop's whole demand is absent from E                          | promise is true by construction                             |
| F5-3 BREAKS no reset transfer, and "domain" is undefined        | [RES-10] gains the **reset** transfer, whose delta is       |
|                                                                 | -len(store); [RES-5] makes a domain a (algebra, store) pair |
| F5-4 BREAKS the handle record consumed by a failed open never   | [RES-9]: the release event is a **closure obligation over   |
|   returns                                                       | the record**, not an enumeration of three holder types      |
| F5-5 BREAKS the acquires-from column is wrong for eight         | [RES-7]: the column is **derived** from the may-suspend     |
|   operations and its test cannot fire                           | contract, and the test reads the selected row's count       |
| F5-6 BREAKS advance<T> names a runtime cursor                   | [RES-5]: every take rounds the cursor to the store's own    |
|                                                                 | align, so the advance is closed and is charged once per run |
| F5-7 BREAKS a cyclic containment graph reached by a derived     | [PROV-6]: refused **at the type, in every program**, so     |
|   release is refused by nothing                                 | L3's no-abort clause stops being aspirational               |
| F5-8 BREAKS Q0c's mitigation is false; a lost lease is in no    | **R2**; Q0b says so and deletes the mitigation sentence     |
|   domain and in no E                                            |                                                            |
| F5-9 GAP the saturation flag is a body summary [CALL-5] forbids | [RES-8]: `saturating(p)` is a **declared** contract clause  |
|                                                                 | checked both ways like `allocates`, keyed by a provider     |
| F5-10 GAP the extent refusal's scope rests on a contradiction   | 1.5 decides it: **a worker lane is an execution context**,  |
|   about lanes                                                   | and [PROV-5] names par as its third source                  |
| F5-11 GAP the entry stack is created, not granted; E is not     | [STK-3] materializes it and [RUN-4] reports StartFailed;    |
|   bound to its build                                            | [RES-2] carries the module's **digest**                     |
| F5-12 GAP [RUN-1]'s subject is not the runtime; the profile row | [RUN-1] makes it a **build** obligation over the emitted    |
|   cannot carry what [RES-1] covers                              | module; [RUN-2]'s row is **open**; [RES-1] and [RES-2] gain |
|                                                                 | a host-object class and a `handle` item shape               |
| F5-13 GAP linearity is per instantiation at an elided brand     | 3.K.0: an elided brand is **linear at the declaration**, so |
|                                                                 | one declaration has one verdict; under R1 it costs nothing  |
| F5-14 GAP [PAR-1] 1975 gains dispose and not the two let forms  | [RUN-3]: 1975's enumeration becomes the **footprint         |
|                                                                 | property** it is reaching for                               |
| F5-15 GAP "keeps" is false and the cost is measured on the      | [RES-6]: "gains", eleven sites across five programs, and    |
|   wrong alternative; the fifth part is unsupplied               | the total-with-proved-capacity alternative is Q12; [RES-9]  |
|                                                                 | supplies the multiplicity                                   |
| F5-16 GAP the cleanup-scratch domain has no item shape          | [RES-5]: it is **frame-resident in the releasing context**, |
|                                                                 | so it is a term of Stack(f) and L6 stays true               |
| NB1..NB18 re-verified                                           | NB2, NB3, NB4, NB5, NB6, NB9, NB13, NB14, NB17 HOLD and     |
|                                                                 | are not weakened; NB1, NB7, NB8, NB10, NB11, NB12, NB15,    |
|                                                                 | NB16, NB18 are the residues F5-1..F5-16 close               |
```

```text
| F3 (consistency)                                               | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| 1 DEFECT [LIV-3] is refused by [STOR-1] 674, unamended          | [LIV-3] restates [STOR-1]'s **partition** over three        |
|                                                                 | writing forms rather than exempting one; probe t8           |
| 2 DEFECT 3.K.0's criterion gets both flagship cases backwards   | 3.K.0: the criterion is **derivation**, per argument, with  |
|                                                                 | one sentence for a declaration and one for a call site      |
| 3 DEFECT `set total = collect(...)` reaches no S12 destination  | [CALL-4]'s clause covers **a `set` target list, the         |
|                                                                 | single-target form included**; four forms, one clause       |
| 4 DEFECT `replace` through &uniq MutSpan is refused by no rule  | [VIEW-4] states the rule over the **commit**: a displaced   |
|                                                                 | loan-bearing value must be consumed by the same statement   |
| 5 DEFECT every const-generic library function reads n as a      | [MSR-6] [S21]; probes t1, t2, t3 against t4                 |
|   value                                                         |                                                            |
| 6 DEFECT every signature that touches a measure or a provider   | 3.L.0 states the discipline, [BLK-0] states the reader      |
|   has the wrong effect row                                      | half, [PROV-4] fixes the allocating rows at reads+allocates |
|                                                                 | +writes, and every signature in 3.L and §4 is corrected     |
| 7 DEFECT 4.2 matches on a Bool                                  | `bs_reserve` returns an enum carrying the value on both     |
|                                                                 | arms, which is also what L3 and L9 require; probe t12       |
| 8 DEFECT 3.L writes `Z` as a source operand                     | 3.L.0: source writes `0_u64`; probe t11                     |
| 9 DEFECT three contracts use a nested result projection         | [CALL-4] [S24] admits it on a result datum on the same      |
|                                                                 | terms [FN-9] 1313 grants a parameter datum                  |
| 10 DEFECT the +checked route claims a fact [ENT-3.S7] does not  | rewritten over cap(built), which seq_heap publishes; no     |
|   establish                                                     | widening of S7 is proposed                                  |
| 11 DEFECT advance<T> is not closed                              | as F2 F5-6                                                  |
| 12 DEFECT L3's no-abort clause is undone by the unmarked case   | as F2 F5-7                                                  |
| 13 DEFECT seq_frame has no store, no judgment and no user       | as F1-5                                                     |
| 14 DEFECT the saturation flag reads proof provenance            | as F2 F5-9; [ENT-1] 2661 is on the third list under [RES-8] |
| 15 DEFECT two infix operations in one expression                | 3.L.0: every body is three-address; probe t13               |
| 16 DEFECT seq_exchange is writable in wf, so it fails L18       | the row is **deleted**; 3.L.2 writes the three statements   |
|                                                                 | and states what writing it that way costs                   |
| I1-I7 the eight wrong line numbers and seven overshooting       | every citation re-derived against v0.41 in 3.K.11, which    |
|   ranges                                                        | names the eight; every range ends on its last nonblank line |
| I8, I9 the two condition-5 failures                             | the [STOR-1] row states the partition and 682's disposition;|
|                                                                 | [LIV-1]'s own body states the [OWN-11] replacement          |
| I10 three `by` columns name a rule whose Amends is "nothing     | each such rule now states its own Amends target directly    |
|   beyond X's"                                                   |                                                            |
| I11, I12, I13, I14 four false 6.8 dispositions                  | each row in 6.8 is corrected and marked FALSE as stated     |
| I15 "keeps" versus "gains"                                      | as F2 F5-15                                                 |
| I16 the arena refusal relation disagrees in three places        | [RES-6], [RES-5] and A.2 all write room(arena) < advance<T> |
|                                                                 | for one run, in the units [RES-5] fixes                     |
| I17 A.1 does not carry the cell [MSR-1] requires                | A.1 carries an exact/bounded/absent cell per measure per row|
| I18, I19 A.2's free identifiers and its value-named view rows   | both corrected in A.2                                       |
| I20 `arena_take` is an operation in no inventory                | the name is gone; the rows are seq_arena and                |
|                                                                 | seq_arena_proved                                            |
| I21 3.L.4's duplicated paragraph                                | removed                                                     |
| I22 [RUN-5]'s fields are spliced inside its theorem fence       | the fence carries the theorem alone and the four fields     |
|                                                                 | follow it                                                   |
| I23 28 of 48 rules state no Depends                             | §3 states four required fields plus a Depends line exactly  |
|                                                                 | when the rule rests on an unchanged sentence                |
| I24 [BLK-1] restates a trichotomy L13 deleted                   | [BLK-1] names [OWN-1] 564's two classes plus [PROV-6]'s     |
|                                                                 | refinement                                                  |
| I25 L14's retirement sentence claims what the exit datum is not | under R1 the guarantee is an ordinary clause, so L14's      |
|                                                                 | retirement note says nothing is lost                        |
| I26 the carried formation datum has no reader                   | [PROV-3]: **deleted**                                       |
| I27 `AmbiguousStoreRegion` is claimed and not stated            | the claim is gone; [PROV-1] states SecondStoreInOneRegion   |
|                                                                 | and [BLK-4] states ConfinedFieldWithoutRegion               |
| I28 the [GRAM-9] row's change is "unchanged"                    | the row says why it is listed: [MSR-5] moves the amendment  |
|                                                                 | away from it                                                 |
| I29 the two programs disagree about arm order                   | every match in this file writes its variants in the         |
|                                                                 | declaration order [PRE-1] 2096 fixes: None before Some, Ok  |
|                                                                 | before Err                                                  |
| I30 §3.K.10 names FixedRing against the library's Ring          | neither name exists: a ring is a run [BLK-1]                |
| I31 META-5's production delta is incoherent                     | recomputed in 3.K.11                                        |
| I32 Q0f's three items against §7's four                         | four in both                                                |
| I33 no rule states the derived release of an arena-backed run   | [PROV-6]'s criterion states it: its release requires no     |
|                                                                 | capability, so it is affine and [STOR-3]'s region-end       |
|                                                                 | reclamation covers it                                       |
| N1..N7 seven notes                                              | N1 the `//` comments are removed from every wf-design block;|
|                                                                 | N2 A.2's unexercised rows are exercised by 4.1 or named in  |
|                                                                 | 6.4; N3 try_take is written in `CONTAINERS.md` §3.6; N4     |
|                                                                 | [PROV-5] reads the graph after [STK-1] and says so; N5 §7   |
|                                                                 | records the B1 conformance disposition; N6 [ERR-3] 1472 is  |
|                                                                 | a changed row under [PROV-6]; N7 [VIEW-4] is restated over  |
|                                                                 | the commit and covers both cases                            |
```

```text
| F4 (writer)                                                    | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| 1 BLOCKING a const generic is admissible only as a const        | [MSR-6] [S21], the fifth of 3.L.6's eight; probes t1-t4     |
|   argument                                                      |                                                            |
| 2 BLOCKING no published relation on a fallible or aggregate     | [CALL-4] [S24], the sixth; probes t6 and t14                |
|   result                                                        |                                                            |
| 3 BLOCKING the Q0c leak is real and 4.1 contains one            | **R2** [S18], the eighth; 4.1's arm is now mandatory        |
| 4 BLOCKING FixedRing over Option costs about 7x and deletes     | [BLK-1]'s **window**, the seventh: a ring is a run, with no |
|   in-place slot mutation                                        | Option, no tag and ordinary element access                  |
| 5 FRICTION a generic that moves a T cannot be instantiated at   | Q8, restated in both halves: structural copy **and** a      |
|   a copy element type                                           | copy-instantiation `move` that is a no-op. Not this         |
|                                                                 | design's, and 3.L.0 says where it bites                     |
| 6 FRICTION Q0a's lost guarantee has no spelling                 | under **R1** it does: `ensures len(rest) >= len(out)`. The  |
|                                                                 | entry(...) operand F4 proposed is not needed                |
| 7 FRICTION three text defects in the printed library and        | all three corrected: the effect rows (F3-6), collect's      |
|   programs                                                      | ensures (now derivable through the `done` conclusion), and  |
|                                                                 | the @flip loop (gone with the window)                       |
| 8 FRICTION a multi-target set is a declaration spelled `set`,   | [LIV-3] states the binder rule and [MSR-3] makes the        |
|   and a [LIV-2] set silently orphans an invariant               | orphaned-invariant case a diagnostic. The `let` token 3.S   |
|                                                                 | [S14] records as an alternative the owner may prefer        |
| 9 FRICTION [CNT-7]'s deletion silently reprices P16             | **R1** withdraws the parameter, so the repricing does not   |
|                                                                 | arise; 3.K.11 rewrites P16 over MutSpan and the by-value    |
|                                                                 | form                                                        |
| 10 FRICTION six diagnostic holes                                | (a), (c) and (e) are answered by [MSR-6], [CALL-4] and Q8;  |
|                                                                 | (b) by [CALL-4]'s widened routes; (d) is now an error under |
|                                                                 | R2 and §4 drafts it; (f) is [MSR-3]'s diagnostic            |
| 11 CLEAN 3.K.0's elision, [CNT-7]'s deletion, [PROV-6]'s        | preserved and not weakened                                  |
|   destructuring consume, [MSR-3]'s atom identity, [LIV-3] on    |                                                            |
|   `set`, [PROV-1]'s brand, [MSR-4], [CALL-1/2/3/5], [RES-6],    |                                                            |
|   [PROV-7]                                                      |                                                            |
```

**Where a fifth-draft rule's content went**, for a reader holding 6.5 to 6.8:

```text
| fifth-draft rule or row      | now                                                     |
|------------------------------|---------------------------------------------------------|
| [CALL-4]'s exit datum         | withdrawn (R1); a helper publishes on its own result    |
| [MSR-3]'s exit placement      | withdrawn (R1); three placements remain                 |
| [MSR-1]'s three measures      | four, with `head` [BLK-1]                               |
| [BLK-2]'s seq_frame row       | deleted; it is seq_fixed                                |
| [BLK-3]'s seq_exchange row    | deleted; 3.L.2 writes it in three statements            |
| [BLK-3]'s three per-slot rows | four, with the front operations [BLK-1]                 |
| [PROV-3]'s carried datum      | deleted; it had no reader                               |
| [PROV-6]'s enumerated linear  | one criterion — release requires a capability — plus    |
|   predicate                   | the `linear` modifier for a logical obligation [S18]    |
| 3.K.7.1 the composition       | [RES-10], a rule, with `retained` and `reset`           |
| [RES-8]'s derived flag        | a declared `saturating(p)` clause [S26]                 |
| [RES-7]'s written column      | derived from the may-suspend contract                   |
| the ceremony of §2.1          | new: eight notions, eight closure sentences             |
| the spellings of 3.K          | unchanged in the rules, and all of them PROPOSED in 3.S |
```

---

## 7. Implementation order

**This is an implementation order and nothing else.** The owner's ruling of
2026-09-03 says so in terms: batches are an order of work, not spec versions, and a
single implementation is fine if it is correct. Nothing below is an approval, a
schedule, or a licence to trade a rule away for a cheaper batch; one batch that lands
all fifty rules correctly is the better outcome. The order is *for* naming, at each
step, a test writable before the next step exists. **And no batch below may begin
before the owner has ruled on the 3.S proposals its rules use**, because every one of
them needs a spelling.

**B0 is not one of these batches.** The region-spelling amendment (3.K.0) lands
first, separately and mechanically, and is not this design's work; every batch below
assumes it and none of them implements it.

**B1. The proof surface.** Rules: [MSR-1], [MSR-2], [MSR-4], [MSR-5], [MSR-6].
First because every later batch's contracts and invariants are unwritable without it,
and because it is a specification amendment with no new construct. Tests: probes
`t1`, `t2` and `t3` accepted after [MSR-6] and `t4` still accepted; a clause whose
operands are two `len` terms, accepted where probe `t5` is a [GRAM-5] parse failure
today; a literal and a parenthesized group still affine factors; a goal discharged
from `len + room = cap` as an affine premise; an element-position `replace` of a
*descriptor* killing its measures and of a *scalar* killing nothing, which is the
carve-out's removal under test; **and `r2_4`'s program accepted**, because [MSR-2]'s
descriptor-precise support is a repair of a live over-kill and not only a new rule.

**B2. Type-derived call transports.** Rules: [CALL-1], [CALL-2], [CALL-3],
[CALL-5]. Second because it is the live defect and needs none of the new types:
today's `&uniq buffer<T>` keeps its spelling and gets [CALL-5]'s type-derived
classification. Test: **`ent5-neg-callee-uniq-buffer-replace-kills-length.wf` turns
XPASS**, rejecting at [OP-4] with residual `9_u64 < len(line)`; plus probe `t9`'s
program, whose accept becomes the same rejection; plus one positive case pinning
[CALL-1]. `docs/patterns.md` P16 is corrected in the same change. **This batch flips
a conformance case from `xfail`, which is conformance evidence; the disposition is
recorded in `governance/APPROVALS.md` with the merge**, as B6's supersession is.

**B3. Multi-return, the exchange, and join-checked liveness.** Rules: [CALL-4],
[LIV-1], [LIV-2], [LIV-3]. Third because B6 and B7 are written in this syntax.
Tests: probe `t7`'s signature parses and binds, and a two-result `ensures` reaches
both binders of a destructuring `let` and both targets of a `set` target list; **probe
`t8`'s program is accepted**, which is the exchange under test at a bare binding, and
the same at a `deref` and at a field; an exchange at a `buffer` element place accepted
where probe `q7`'s `set` spelling is [OWN-1] today; probes `p10` and `w6` both
accepted after [LIV-2]; probe `f3`'s program a [LIV-1] error naming both predecessors
instead of `SemanticUnsupported`; a loop moving and restoring an outer binding
accepted where probe `f5` is [OWN-11] today; probe `t6`'s per-variant `ensures`
accepted and read at the caller's arm; and **a plain `set` receiver publishing a
relation**, which is the destination round 5 found missing.

**B4. Measure datums, images, and atom identity.** Rules: [MSR-3]. Separated from
B1 because it touches [ENT-2]'s term list, [ENT-5]'s call boundary and [ENT-6]'s
transfer machinery, and because it needs [LIV-2] and [LIV-3] from B3. Tests: a
`buffer` helper whose `ensures` names `len` of a parameter it consumed is accepted,
and its caller establishes the relation where `M(c,q)` refuses it today; a
reinitialized binding does not inherit a fact stated over its predecessor; **a header
invariant over a binding an exchange rewrites is preserved on the backedge**, with the
[LIV-2] variant rejected so the two forms are pinned apart, and with the
orphaned-invariant diagnostic under test; and a `construct` carrying a measured
operand publishes the field's measure.

**B5. Linearity, structural release, and the destructuring forms.** Rules:
[PROV-6], [LIV-1]'s scope-exit half. Moved ahead of the container batch because R2's
criterion is stated over release actions the language already has and because every
later test needs the diagnostics. Tests: probes `r2_5`, `w7` and `m02` rejected with
`LinearValueNotDisposed` and their repairs compiling; **probe `x4`'s program rejected
with `LinearValuePartiallyConsumed`** and its destructuring-consume repair compiling;
a `dispose` through a shared borrow rejected at [OWN-1], which is round 5's second
attack under test; a `dispose` of a proper sub-place rejected; a `linear struct`
whose value is dropped rejected and whose value is destructured accepted; probes `w5`
and `m03` rejected with `LinearValueAcrossPropagate`; **probe `x6`'s self-referential
type rejected at its declaration** in a program with no marker at all, naming the
cycle, with its `a5`/`a6` non-recursive sibling still compiling to a straight-line
walk.

**B6. The brand, the runs, the window, confinement, and the declaration domain.**
Rules: [PROV-1], [BLK-0], [BLK-1], [BLK-2], [BLK-3], [BLK-4]. Retires `buffer<T>`,
`box<T>` and `arena<'r, T>` from the writer surface. Carries monomorphization for a
compiler-owned generic domain. Tests: a `FixedVector<Handle, 64>` object table with
affine elements, filled by 3.L.3's `vacant`, accepted, where probe `p9` is [OP-1]
today; a `vacant` result whose `len >= n` discharges a subscript with no equality
anywhere, which is probes `x1c`/`x1d` under test at full scale; **a queue built from
`seq_place` and `seq_take_front` with no `Option` anywhere**, whose `len` is exact and
whose elements are mutated in place, which is the window's whole justification; a
`Span` formed over a run that has had a front removal **rejected** at [VIEW-2]'s
premise and accepted after a drain; `struct Chunk['s]` accepted where probes `r2_6`
and `m05` are parse errors today, with two instances at different regions rejected as
distinct types; a stored brand elided in a heap-only program and written beside an
arena; and **two reserving occurrences naming one region rejected at the second**.
This batch supersedes B2's conformance case, whose program no longer typechecks; that
disposition is conformance evidence and is recorded in `governance/APPROVALS.md`.

**B7. Views, loans, ranges.** Rules: [VIEW-1], [VIEW-2], [VIEW-4], [VIEW-6],
[PROV-3]. [PROV-3] lands here because views are its only user and because [SET-1] and
[SET-2] must change in the same batch that admits the `MutSpan` write. Tests: an
element write through a `MutSpan` accepted where probe `p7` is [SET-1] today; **a
`replace` through `&uniq MutSpan` rejected by [VIEW-4]'s commit rule**, and so is a
`replace` of a `Vector` place under a live origin set, which probe `w2` shows the
compiler accepts today for the arena spelling; two `MutSpan`s on one run rejected at
the second formation citing [OWN-5]; a write to `k` while a view formed at `table[k]`
is live rejected citing the view's loan; and a two-result signature with two
same-region view results rejected at [VIEW-6].

**B8. Stores, the heap as a value, and reservation.** Rules: [PROV-2], [PROV-4],
[PROV-5], [PROV-7], [RES-6]. Tests: probe `p5_ambient`'s program **rejected**; a
`main` that omits `command.heap` cannot reach any allocation; a run released to a
store of a different region failing to typecheck with the two types rendered; a
region block entered twice by a loop republishing `len(store) = 0_u64` truthfully;
**probe `x8`'s program rejected with `ExtentReservedOnACallCycle` under
`arena_extent` and accepted under `arena_frame`**, with the graph read after [STK-1];
an arena-backed run of `ReadFile` closing every handle at its scope exit, which is
the reset/content split under test; a helper lending a provider onward compiling,
where `r1_relend` and `m19` are [OWN-6] today; and two overlapped disposals from one
store denied [PAR-1] permission while a window containing one is not.

**B9. System I/O over views, and the handle table.** Rules: [VIEW-7], [RES-9].
Tests: `tests/programs/wfgrep.wf` migrated to 3.L.3's `filled` and `MutSpan`,
compiling with no `allocates` entry anywhere on its call graph — the first program
that demonstrates goal A's container half end to end; **a marked `main` selecting
`command.files` and `command.cwd` that opens one file in a loop, reads it into a
`filled` destination over a `MutSpan`, and publishes a handle row of one**; and **an
open that fails on every attempt, whose handle records all come back**, which is
round 5's F5-4 under test and which no earlier batch list would have caught.

**B10. The stack judgment and the divergent entry.** Rules: [STK-1], [STK-2],
[STK-3], [STK-4]. Tests: probes `f2b_tail` and `f8_tailframe` **not** rewritten by
[STK-1]'s premise and rejected by [STK-2] under the marker; their borrow-free
variants rewritten into one dispatcher with one frame; a member holding a live linear
binding across the jump not rewritten, nor one that opens a region for an
`arena_frame`; probe `p3_rec` still accepted without the marker; a `--stack-ledger`
run reporting one chain per context rather than disjoint roots; probe `f3_forever`'s
idle loop accepted; **probe `n3_propagate_loop`'s driver loop accepted**; and a loop
with a reachable `break` still requiring a return.

**B11. The envelope and the judgment.** Rules: [RES-1] to [RES-5], [RES-7],
[RES-8], [RES-10], [RUN-1], [RUN-4], [RUN-5]. Tests: 4.1 source-resource-closed and
its `E` matching a pinned symbolic expectation; 4.2 reported not resource-closed with
the heap-reaching path rendered; a retaining loop whose trip count is a runtime value
rejected at that loop with the value named; one whose checked refusal rejoins the
backedge **accepted**; one of four iterations followed by one more acquisition
publishing a peak of five and not two; the same loop with its acquisition one function
down accepted through a declared `saturating(p)`; **a region block inside a loop whose
reset composes to a zero backedge delta**, which is F5-3 under test; **a service loop
with no `break` whose acquisitions appear in its `retained` entry**, which is F5-2;
B9's marked file program composing its handle demand and rejected when it exceeds the
profile cap; a marked program calling a may-suspend operation whose store has count
zero rejected at [RES-7]; and a program whose demand exceeds every profile row failing
**target qualification** citing no language rule.

**B12. `par` and the envelope.** Rules: [RUN-2], [RUN-3]. Tests: a `filled` plus
`MutSpan` plus counted subscript fill receiving [PAR-2] permission in an unmarked
program, which needs the ranged origin; the same loop inside a `resource_closed`
entry emitting no `par` construct and publishing `lanes(1)`; two overlapped statements
allocating from distinct providers permitted and two from one provider not; a window
containing a `dispose`, a destructuring consume and a multi-result `let` **each judged
by its own footprint** rather than by an enumeration; and [RES-10]'s `par` rule
composing against a pinned profile row.

**3.L is not a batch.** It is written against the rules, not implemented beside
them; where its functions are useful as evidence — `filled` in B9, `collect` and
`vacant` in B6, the pool in B11 — they land as test programs under
`tests/programs/`, which is where 5.0 recommends they stay.

---

## Appendix A: generated data

Two tables the rule text refers to and does not contain. **Neither is a rule.**
[BLK-0] says that an operation inventory exists and what every row of it must
satisfy; [MSR-1] and [RES-5] say that a measure table and a ceiling table exist and
what every row of them must contain. The tables themselves are **generated data**,
carried the way [SYS-2]'s declaration records are carried, and a diagnostic cites the
rule and names the row in its payload rather than citing the row. Every spelling in
them is a 3.S proposal.

### A.1 Measures and ceilings

Derived from [BLK-1]'s storage column rather than written per nominal: a value whose
backing is a run of its own is a descriptor, and a value whose backing is inline
carries its elements. **Every cell is one of `exact`, `bounded` or `absent`**, which
is what [MSR-1] requires and what the fifth draft's table carried for `len` alone.

```text
| measured type            | len                | cap             | room      | head       |
|--------------------------|--------------------|-----------------|-----------|------------|
| array<T, n>              | n, exact           | n, exact        | 0, exact  | 0, exact   |
| FixedVector<T, n>        | initialized slots, | n, exact        | cap - len,| window     |
|                          |   exact            |                 |   exact   |   origin,  |
|                          |                    |                 |           |   bounded  |
| Vector<'s, T>            | initialized slots, | slots taken,    | cap - len,| as above   |
|                          |   exact            |   exact         |   exact   |            |
| Span, MutSpan            | viewed elements,   | len, exact      | 0, exact  | 0, exact   |
|                          |   exact            |                 |           |            |
| Arena<'s, bytes, align>  | cursor bytes,      | bytes, exact    | cap - len,| absent     |
|                          |   bounded          |                 |   bounded |            |
| FileFactory              | live handle        | the profile's   | cap - len,| absent     |
|                          |   records, exact   |   handle-table  |   exact   |            |
|                          |                    |   row, exact    |           |            |
| Heap<'s>                 | absent             | absent          | absent    | absent     |
```

`Heap<'s>` has no measure because L6 says a general store has no measure that means
anything; that is the absence of table data, not an exception clause. **Exactly two
cells are `bounded` and each has one reason.** An `Arena`'s `len` is bounded because
its alignment padding is a target-stage quantity. A run's `head` is bounded because a
front operation moves it modulo `cap` and a modular expression is not a difference
bound; every formation row publishes `head = 0_u64` exactly, every back operation
publishes `head(result) = head(vector)` exactly, and only `seq_place_front` and
`seq_take_front` publish the two-sided `0_u64 <= head(result)`,
`head(result) <= cap(result)`. A program that never removes from the front therefore
never meets a bounded cell.

```text
| nominal                     | (size_ceiling, align_ceiling)                          |
|-----------------------------|--------------------------------------------------------|
| Heap<'s>, Arena<..>         | (32, 16)   proof-only representation, one word         |
| Vector<'s, T>               | (32, 16)   a descriptor: pointer, cap, len, head       |
| FixedVector<T, n>           | T's pair repeated n times, plus (24, 8) for len, cap   |
|                             |   and head, with aggregate alignment max(align(T), 8)  |
| Span<'r,T>, MutSpan<'r,T>   | (32, 16)                                               |
| array<T, n>                 | T's pair repeated n times, as [OP-9] 992 already fixes |
```

`advance<T>` for the bump domain is `round_up(size_ceiling(T) * count, align)`, where
`align` is the store's own type constant and both allocating rows require
`align >= align_ceiling(T)` as a compile-time comparison of two constants. There is
no fallback: the requirement refuses the other case rather than charging a ceiling
for it.

### A.2 The kernel operation inventory

Twelve rows, plus the four readers, which are [OP-1] table rows and not this
domain. `V` is either run type. Every row is complete over **every** measure
it writes, on every exit, as [BLK-0] requires; the fifth draft's licence to state two
of three and leave the rest to the standing identity is gone, and this table is where
the difference is visible.

```text
Formation                                                                          [S7]
  seq_fixed<T, const n: u64>()                       -> own FixedVector<T, n>       pure
      len(result) = 0, cap(result) = n, room(result) = n, head(result) = 0
  seq_arena<T>['s](arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Option<Vector<'s, T>>       reads(arena), allocates(arena), writes(arena)
      requires align >= align_ceiling(T)
      Some(value: r): len(r) = 0, cap(r) = count, room(r) = count, head(r) = 0,
                      <datum of len(arena)> <= len(arena)
                        <= <datum> + round_up(size_ceiling(T) * count, align)
      None:           len(arena) = <datum of len(arena)>,
                      room(arena) < round_up(size_ceiling(T) * count, align)
      both:           cap(arena) = <datum of cap(arena)>
  seq_arena_proved<T>['s](arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Vector<'s, T>               reads(arena), allocates(arena), writes(arena)
      requires align >= align_ceiling(T)
      requires room(arena) >= round_up(size_ceiling(T) * count, align)
      as the Some row above
  seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)
      -> own Option<Vector<'s, T>>       reads(heap), allocates(heap), writes(heap)
      Some(value: r): len(r) = 0, cap(r) = count, room(r) = count, head(r) = 0
      None:           nothing; a general store publishes no measure (L6)

Reservation                                                                        [S9]
  arena_frame<const bytes: u64, const align: u64>['s]()
      -> own Arena<'s, bytes, align>                                                pure
      len(result) = 0, cap(result) = bytes, room(result) = bytes
                                              its contribution to stack(context) [PROV-5]
  arena_extent<const bytes: u64, const align: u64>['s]()
      -> own Arena<'s, bytes, align>                                                pure
      len(result) = 0, cap(result) = bytes, room(result) = bytes
                                              its own region item of E [PROV-5]

Per slot                                                                           [S8]
  seq_place(vector: own V, value: own T)  -> own V     reads(vector), writes(vector)
      requires room(vector) > 0
      len(result) = len(vector) + 1, room(result) = room(vector) - 1,
      cap(result) = cap(vector),     head(result) = head(vector)
  seq_place_front(vector: own V, value: own T)
                                          -> own V     reads(vector), writes(vector)
      requires room(vector) > 0
      len(result) = len(vector) + 1, room(result) = room(vector) - 1,
      cap(result) = cap(vector),     0 <= head(result), head(result) <= cap(result)
  seq_take(vector: own V)   -> (rest: own V, value: own T)
                                                       reads(vector), writes(vector)
      requires len(vector) > 0
      len(rest) = len(vector) - 1,   room(rest) = room(vector) + 1,
      cap(rest) = cap(vector),       head(rest) = head(vector)
  seq_take_front(vector: own V) -> (rest: own V, value: own T)
                                                       reads(vector), writes(vector)
      requires len(vector) > 0
      len(rest) = len(vector) - 1,   room(rest) = room(vector) + 1,
      cap(rest) = cap(vector),       0 <= head(rest), head(rest) <= cap(rest)

Readers                       ([OP-1] table rows, not this domain)                 [S11]
  len(p) / cap(p) / room(p) / head(p)                -> own u64                     pure

Views                                                                             [S10]
  seq_span['r, T](vector: &'r V)          -> own Span<'r, T>          reads(vector)
      requires head(vector) <= 0_u64
      len(result) = <datum of len(vector)>, cap(result) = <datum of len(vector)>,
      room(result) = 0, head(result) = 0
  seq_mut_span['r, T](vector: &uniq 'r V) -> own MutSpan<'r, T>       reads(vector)
      requires head(vector) <= 0_u64
      as the row above
```

Two statements are not rows and are stated in [PROV-6]: `dispose p using (q1, ...);`
[S12] and the destructuring consume `let N(f1: b1, ...) = move v;` [S13].

Notes on the inventory. **`seq_place` is the operation the whole design exists for**:
total under its requirement, allocation-free on every backing, one store plus one
length increment. **The four per-slot rows are two-sided because L12 is**, and the
front pair is what makes a queue a run rather than a run of `Option`. **Nothing here
is total at a capacity boundary**, because an overwriting form would need L9's
published displacement. **Nothing here removes from the middle, clears, truncates,
grows, exchanges, or constructs a filled or vacant run** — each is 3.L, and 3.L.6
records that none needed a row the four per-slot operations do not have.
