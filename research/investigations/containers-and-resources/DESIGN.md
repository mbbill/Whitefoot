# Containers and resources: the integrated design

The single design for batch 0116: one set of laws, one set of rules, one
vocabulary, one amendment register. `RESOURCES.md` beside it keeps the writer's-eye
resource migrations and `CONTAINERS.md` the longer library functions of 3.L; neither
carries rule text, and a reader who reads only this file has the whole design.

**Fifth draft, after falsifier round 4 and the owner's minimality ruling of
2026-09-03.** Round 4 confirmed that the fourth draft's root change was right and
total — a store's identity is a region carried in the type of every value it
backs, and no falsifier could find a rule anywhere that admits a store region by
outlives rather than by identity. What round 4 broke was everything the fourth
draft had *not* closed the same way: a store's **activation** (recursion makes two
entries of one region block live at once over one committed extent), a store's
**release** (a partial move abandons a linear leaf; an arena's reset runs no
content release; the disposal walk imports a heap-growing worklist that aborts),
and three type-level notions introduced without the closure the brand got —
`linear` declared beside affine instead of refining it, `loan-bearing` given a
prohibition whose two halves disagree, and a carried formation datum with no
transport across a call.

The owner's ruling reshapes the answer rather than adding to it.

> The kernel specification is the **minimal** set: it admits only what cannot be
> implemented in wf itself. Anything a writer could implement in wf on top of the
> kernel does not enter the spec; it belongs to a standard library — and the owner
> leans toward not having one at all — or to user code. Container capabilities are
> abstracted to the lowest common primitive, and only the truly unimplementable
> part enters the spec. Non-normative content (bound tables, operation
> inventories) never goes in the spec body. Batches are an implementation order
> only, not spec versions; a single implementation is fine if correct.
> Human-factors conveniences are not spec content.

Applying it is not a trim. Section 3 is now two sections that are read
differently. **3.K** is the kernel: forty-eight rules over six nominals and
fourteen operations, every one of which a writer cannot write in wf. **3.L** is
the library, **written out in wf** against 3.K, with each item's proof obligations
walked against the kernel rules and against what v0.40 proves today; its longer
functions live in `CONTAINERS.md` §3 and 3.L.2 is the result. The partition is the
argument: five owners became one primitive in two brandings, thirty-odd operations
became fourteen, three views became two, sixteen added nominals became five, and the
fourth draft's `update` statement, its `AppendView`, its `Pool` store, its three
failure structs and its whole [CNT] and [SEQ] inventory left the kernel and were
rewritten as ordinary wf. **Seven** things the library could not be written without
are now in the kernel, and 3.L.6 names each with the function that demanded it. That test — write it in wf, and only what refuses to be
written enters the spec — is what this draft is for.

Tree read: `batch/0116-containers-and-resources` at `main` a40c7e70,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file, re-derived in this session; every other citation names its file.

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
- `FixedVector<T, n>` holds affine `T` through an initialized-prefix typestate.
- The core is a contiguous initialized-prefix sequence; keyed containers are
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

Five footnotes, because the minimality ruling moves material the settled list
names. Each states what survives and what changed, and each is a decision the
owner has not separately ruled on; 5.0 collects them.

1. **The owner inventory.** The settled list names four owners. `FixedVector<T,
   n>` is unchanged. `HeapVector`, `ArenaVector` and `PoolVector` were three names
   for one shape at three stores; with [PROV-1] putting the store in the brand,
   nothing else distinguished them, so they are one kernel nominal `Vector<'s, T>`
   at three regions, and the three names survive in 3.L as what a writer calls an
   instance.
2. **`FixedRing`.** The fourth draft made it a fifth owner. It is a
   `FixedVector<Option<T>, n>` plus a head and a fill, written in wf in `CONTAINERS.md` §3, so
   it leaves the kernel. The settled list excluded no rotation and names no ring.
3. **`AppendView`.** The settled list names the owner/view split and the
   by-value transformation, both of which survive. `AppendView` and `absorb` were
   the fourth draft's device for one *proof* problem — keeping a caller's length
   alive across an appending callee. [CALL-4]'s exit datum over a `&uniq`
   parameter solves that problem for every borrowed value rather than for one
   type, which is the ruling's own instruction, so the third view and its commit
   event are gone. 5.0 states exactly what is lost.
4. **`update`.** The fourth draft's transformation statement is not a new
   statement here. Its one unwritable half — transforming a place through a call
   with no observable point between the read and the write — becomes an admission
   on the existing `set` [LIV-3]; its other half was sugar and is gone.
5. **Argument order.** The settled append example writes its source argument
   first, while [GRAM-11] fixes argument order from the declaration and every
   helper here declares its destination first.

## Contents

1 [The problem](#1-the-problem) · 2 [The laws](#2-the-laws) ·
3 [The rules](#3-the-rules): [3.K kernel](#3k-kernel-rules),
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
here to remove.

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
readable from the value's own type, so D1's sibling has none either. And make each
of those a property that is **closed** — closed under containment, under every
value-forming step, under every activation of the scope that creates it, and under
every way a value can leave a scope — because round 4's whole finding is that a
property closed in three places and open in the fourth is not a property.

### 1.4 The minimality ruling, and the partition test

The ruling asks one question of every candidate rule: *could a writer implement
this in wf, given the rest of the kernel?* If yes, it is not spec.

Applying that question needs a criterion for the container half, because
containers are where "could a writer write it" is least obvious. The criterion is
storage. A writer can express **values**: construct them, move them, place them
into fields and elements, match on them, and let them go. A writer cannot express
**storage that holds no value**: a slot in `[len, cap)` is typed, addressable and
uninitialized, and wf has no spelling that reaches it, no spelling that declares
it, and no way to make the boundary a checker-maintained fact rather than a
killable data field. `array<T, n>` is the shape a writer *can* have, and it
requires `n` live values, which for affine `T` is exactly what the writer does not
have. So the initialized-prefix run of slots is the lowest common primitive of
every container this design ever proposed, and it is genuinely unimplementable.

Everything above it is arithmetic over that primitive and is written in 3.L: a
ring is a full run of `Option<T>` plus a head and a fill; a pool is a run of runs;
a growable vector is a run plus a growth policy; a keyed table is a full run of
`Option<T>` with element `replace`; middle removal is an exchange and a take;
filled and vacant construction are counted loops. The store half divides the same
way: a **store** — a thing that hands out runs and takes them back — cannot be
written, because it manages storage; a **pool** — a thing that hands out *values*
that happen to be runs, and takes them back — is ordinary data and is written.

Seven things the library could not be written without are in 3.K and nowhere else,
and 3.L.6 names each with the library function that demanded it. That list is the
deliverable of this draft: it is the only honest way to know that the kernel is
neither too big nor too small.

One amendment this design **assumes and does not draft** is stated in 3.K.0, because
without it the container half is not writable. [FORM-1] admits one spelling per
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
in v0.40 or in this design creates, enters, or switches an execution context;
`program_kind := "command"` is the whole production (177) and [FN-7] admits exactly
one entry, so an `interrupt fn` does not parse. Program 4.1 is written accordingly:
it is a cooperative run queue of state machines that advance on one chain, not a
scheduler that switches stacks.

**What the follow-on inherits, and what it must reopen.** Round 4 found the
fourth draft's version of this table self-contradictory: it marked the brand
*inherited* while marking the extent item's per-context identity *owed*, and those
cannot both be true. This draft moves the brand row to **owed with a stated
bound**, which is what [PROV-1]'s new activation invariant makes precise.

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
| envelope accounting is per        | inherited: the per-domain map of 3.3.1 composes per      |
| domain and peak-based [RES-5]     | context; a switch transfers no peak and creates no domain|
| disposal is structural and         | **owed**: [LIV-1] is a per-join check over one function's|
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
traps, aborts, retries, falls back, or promotes a store to a larger one, and no
compiler-derived action does either.*
Because v0.40 claims zero writer-reachable runtime-trap families (spec line 6)
while heap exhaustion still ends a process with no source value: owner ruling R12
(`L5657-5666`), B3, audit answer Q8. The last clause is round 4's: probe `a8`
showed a compiler-derived drop calling `realloc` and `wf_resource_abort`, so a law
stated over operations alone did not reach the code that broke it.

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
extents, per-class slot counts, per-context stacks, lane counts) and never one byte
total. A store the program itself reserves is shaped by the same rule: a reserving
operation that needs an alignment or a separately grantable extent produces its own
`region` item and is not folded into a stack total.*
Because sixteen bytes holding four four-byte objects, the first and third released,
cannot serve an eight-byte request, and a deployment reading one stack number
cannot tell an alignment failure from a size failure: owner ruling R12, B9, B11.

**L7. Lowering before judgment, and a tail call is a dead caller frame.** *Tail
recursion, including mutual tail recursion, is rewritten into one dispatcher
function before any resource judgment runs; an intra-component call edge is a tail
edge exactly when the caller's activation record is dead at the jump, and never
because the call is written in a return statement.*
Because an optimization that may or may not fire cannot be a premise of a guarantee
and a syntactic condition cannot see a live loan into a caller's frame: owner
rulings R3 (`L989`) and R12, B10, probes `f2b` and `f8_tailframe`, accepted today.
"Into one dispatcher function" is round 4's correction: the fourth draft stated an
ABI obligation about a jump its own rewrite removes.

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
"have no flag derived from an actual to be wrong": `EVIDENCE-sweep-D1.md`, probes
`d1` and `x11`, both accepted today.

**L12. The initialized prefix is a stack, and the language says so.** *A run of
slots is exactly `[0, len)` initialized and `[len, cap)` raw; the boundary is
checker-maintained typestate carried by the run's own value, and no per-slot tag,
occupancy bitmap, or runtime discriminant is language state. The kernel admits
exactly append at the end, removal from the end, and exchange of two initialized
positions; every other order is arithmetic a writer performs over those three.*
With no per-slot state the checker never needs a quantified proposition over slots,
and occupancy at a stable index is ordinary data: `FixedVector<Option<T>, n>` full,
with element-position `replace`, is that program, and probe `x7` compiles its shape
today. The last sentence is the minimality ruling applied to L12 itself: the fourth
draft's five removal and construction forms are 3.L.3 to `CONTAINERS.md` §3. Owner's settled
decision; audit answers Q2, Q4, Q10.

**L13. A value's store is a component of its type; acquisition, release and
activation are all closed over it.** *Every store the program can exhaust is named
by one region, minted where the store is reserved or where the runtime hands it in,
and every value that store backs carries that region in its own type. A region
names **at most one live store at any program point**, and a placement whose
storage cannot be per activation is refused wherever more than one activation of
its region block can reach it. A value whose backing is reclaimed per value is
**linear**, which is a property of an affine type and not a third class: it has no
compiler-derived release, and it leaves a scope only by being moved out whole, by
being destructured whole, or by being disposed to the store its type names.
Linearity and disposal are both closed under containment, and a partial move of a
linear value is refused because it is the one way a leaf leaves by none of the
three. No source construct selects, replaces, or observes a release action, and a
store's storage reclamation never stands in for its content's own release.*
Sentence one is round 3's rank-one repair and survived round 4 from every position
attacked. Sentences two, three, four and five are round 4's four: recursion made
one region name two live stores over one committed extent (F1 attack 1, F2 NB2);
`linear` declared beside affine deleted `move` and `dispose` for every value it
classified (F3 defect 3); a partial move abandoned a linear leaf that no rule saw
(F1 attack 2, F2 NB6, F4 blocking 1, probe `x4` accepted today); and an arena's
reset ran no content release, leaking every host handle placed in arena content
(F2 NB5). B2's drop order, audit answer Q10, [STOR-3] 683, [EFF-2] 1421.

**L14 is retired.** It stated that an `AppendView` reaches only what it appended
and never decreases its owner's length. The type is gone (footnote 3) and the
guarantee it bought — a caller keeping a length across an appending callee — is
[CALL-4]'s exit datum, which is stated over every borrowed value. The id is not
reused.

**L15. The descriptor's capacity is a value; the allocator's extent is not.**
*`len(v)`, `cap(v)` and `room(v)` are a run's own logical measures and are readable
as ordinary `u64` values. No operation observes the physical extent the allocator
provided. Every operation that writes a measured place publishes, for each measure
of that place, its exact new value where that measure is exact and a two-sided
bound where it is monotone, including the measures it did not change.*
The first draft forbade reading `cap` and `room` on a rationale that only forbids
reading the allocator's size, so every pop proved and no push did: B3, audit answer
Q9, probes `q24`, `v25`, `v26`. The exact/monotone split is round 4's: an arena's
cursor is monotone and no exact value exists for it, so a law demanding one made
three rows ill-formed and killed `len(arena)` on every refusal edge (F3 defect 6).

**L16. One measure algebra, and one goal disposition.** *`len`, `cap` and `room`
are one-place terms of the term language, defined once with their support, their
kills and their standing identities, over every measured place: runs, views, and
providers alike. Every consumer of a numeric goal asks one question, whose complete
admitted derivation is stated once; no rule grants a proof route to a construct by
name.*
A language in which "can this inequality be derived?" depends on which construct is
asking has several provers and a writer can reason about none of them; probes `v25`
and `v26` are the same proof asked twice with opposite verdicts. [ENT-1] 2645.

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
the same per-edge check makes linear disposal checkable. The whether/which split is
round 4's: `FixedVector<Option<T>, n>` is the design's own promoted idiom and its
per-element release *is* selected by a runtime discriminant (F1 finding 10). Probe
`f3`; [ENT-5]'s own all-predecessor join.

**L18. The kernel admits only what wf cannot express.** *A rule enters the kernel
exactly when no program a writer can write in wf over the remaining kernel has its
effect. A capability a writer can build is not a rule, a convenience is not a rule,
and a table of data is not a rule: the rule is the sentence that says such a table
exists and what it must contain, and the table is generated data beside it.*
The owner's ruling of 2026-09-03, stated as law so that every rule below can be
checked against it and every removal can name it. Its converse is the obligation
3.L discharges: an item moved out of the kernel is written in wf, or the kernel
lacked a primitive and 3.L.6 says which.

---

#### 3.K.0 The region-spelling assumption

This design assumes one amendment it does not draft. **Whether a region is written
at a given position is determined by the program text, and the determined spelling
is the only legal one**: a region name is written exactly where it relates two
positions of one declaration, and elided everywhere else. That is a change to
[FORM-2], [GRAM-2] to [GRAM-5], [FN-2] and the [OWN] borrow forms, it is uniform over
every region position in the language — parameter lists, borrow annotations, region
arguments on types, call-site region arguments, and region blocks — and **it lands
first, as its own separate and mechanical spec amendment**. It is not a rule of this
design, it is not in 3.K's count, and 3.K.11 does not register it.

It is stated here because the container half cannot be written without assuming it.
[FORM-1] admits exactly one spelling per semantic construct. Putting a store's
identity in the type means a region in every type that names a store, unless the
text determines it — in which case *writing* it is a second spelling and the law says
there is only one. So the brand cannot be in the type without that amendment, and
the amendment cannot be brand-specific, because a brand is one more region argument.

**The one property this design needs from it** is narrower than the amendment
itself, and it is the property the owner named: *whether a region or a brand is
written at a given position must be decidable by reading the declaration text alone,
never by waiting for compiler feedback.* Every rule below is written against that
and against nothing else. In particular [PROV-1] supplies the datum a brand position
needs — the enclosing nominal's own region parameters, plus the entry heap's store
region when the entry selects `command.heap` — and that datum is read from one
declaration and one entry input row, never from a callee, a caller, or an
instantiation.

**How the brand is therefore spelled**, which is what section 4 and 3.L are written
in:

- A **heap-derived** type carries no visible brand. The entry heap is unique, it has
  program lifetime, and [OWN-3] 575 makes its region incomparable with every other,
  so `Vector<u8>`, `Bytes` and `Heap` name it and nothing else can be named that way.
  4.2 declares no region parameter anywhere.
- An **arena- or pool-derived** type writes the brand exactly where the same store
  appears at two positions of one declaration:

  ```wf-design
  struct BlockPool['s] { free: FixedVector<Vector<'s, u8>, 8>; }

  fn pool_take['s](pool: &uniq BlockPool<'s>) -> leased: own Option<Vector<'s, u8>>
  ```

  `'s` relates the nominal's parameter to its field, and the parameter's type to the
  result's, so it is written at every one of those positions. The loan region of
  `&uniq` relates nothing and is elided, here and at every provider call in the
  language.
- A helper whose brand relates nothing writes none, and is thereby generic over the
  store it is handed: 3.L.4's `collect(out: &uniq Vector<u8>, source: own Span<u8>)`
  is the whole signature.

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
**forty-eight rules**, six nominals, fourteen operations, two added statement forms,
and 3.K.0's one assumed amendment. Every rule answers L18's question with *no writer
can write this in wf*, and 3.L.6 lists the seven that only the partition test proved.
**3.L is the library**, written in wf against 3.K; it is not part of the language,
it is not blessed, and no rule of 3.K names any of it.

**Every kernel rule states five things: the judgment it creates, the fact it
publishes, what it amends, what it depends on, and its law.** A rule that creates no
judgment writes `*Judgment:* none` and says what it is instead. `*History:*` points
at the round in 6.5-6.8 that produced the rule's current shape and carries nothing
else. Section 3.K.11 is a **collation of the `Amends:` and `Depends:` lines and
carries nothing else**: it is written last, from the rules.

### 3.K Kernel rules

#### 3.K.1 `[MSR]`: measures, and the one goal disposition

This family is first because everything else consumes it. It adds no statement
form and no type; it is a specification amendment.

**[MSR-1] Three measure terms, over one place, for every measured value.**
`len(P)`, `cap(P)` and `room(P)` are terms of the [ENT-2] term language, of
fragment type `u64`, where `P` is an admitted place. They are defined once, here,
for every *measured* type, and which measures a type has, and whether each is
exact or monotone, is table data rather than a rule with an exception. The table
is Appendix A.1; the rule is that it exists, that it gives every measured type a
row, and that every row's cell is one of *exact*, *monotone*, or *absent*.

An admitted place for a measure term is a `place` [GRAM-5] formed with field
selections, `deref` wrappings **and subscripts**, whose final selected type is a
measured type. The subscript admission is the change: `len(table[i])` is a term,
so a run of runs has provable operations.

*Judgment:* the [OP-4] admission above at every subscripted measure place.
*Publishes:* the term. *Amends:* [ENT-2] clause (b) (2675), which today admits
`len(P)` only for `array`, `slice` and `buffer`, and only for subscript-free
places; [OP-4] 909, whose obligation gains the erased-clause attach-site case.
*Law:* L15, L16. *History:* 6.8, F1 attack 8; 6.5, F1-11.

**[MSR-2] Support is descriptor storage, a kill is an ordinary [ENT-5] event, and
a standing fact has empty support.** A measured value's storage is two disjoint
parts, exactly as [STOR-1] and L12 already describe the object: its **descriptor
storage**, the length and capacity words its value carries, and its **element
storage**. The support of a measure term over `P` is:

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

The fourth consequence is why [ENT-5] 2887 clause (a)'s parenthetical carve-out —
"*while an element-position replace, like an element write, kills none*" — is
**removed rather than narrowed**. That sentence is true in v0.40 for a reason
[MSR-1] deletes: `len(P)` is defined only for `array`, `slice` and `buffer` and
admits no subscript, so an element position can never hold a descriptor. Once it
can, the carve-out is a second statement of the granularity, in the wrong place and
now false. The kill becomes the plain overlap test and the four consequences are
derived from it.

At every program point at which `P` is live, these hold implicitly:

```text
Z <= len(P)          Z <= room(P)          len(P) <= cap(P)
```

and the three-term identity `len(P) + room(P) = cap(P)` is appended, as the two
inequalities `len(P) + room(P) - cap(P) <= 0` and `cap(P) - len(P) - room(P) <= 0`,
to [ENT-6] 3001's automatic affine-premise sequence, with the empty support every
standing fact has. That is the shape [ENT-6]'s premises already take, it is usable
by `AUTO`'s families unchanged, and it keeps the identity out of L0, whose
uniqueness argument [ENT-4] 2854 rests on the difference-bound shape.

**A measure whose value is a compile-time constant or a runtime-profile symbol is
a standing fact with empty support.** A formation row that publishes `cap(result) =
n` for a written const `n`, and a runtime store whose `cap` is a profile row,
therefore both give a capacity that no event kills for the life of the term. This
is where the fourth draft put a type-level constant and it generalizes to the one
store whose capacity cannot go in a type: [RES-5] already asserts the sentence for
a profile symbol and [MSR-2] is where the kill lives, so it belongs here.

*Judgment:* none. *Publishes:* the implicit facts, the two automatic premises, and
the standing-fact class. *Amends:* [ENT-2]'s implicit-fact sentence (2722);
[ENT-5]'s support and kill sentences (2857-2890), whose length-term support becomes
the descriptor-storage relation above, whose kill classes (a) through (d) gain the
effect-row statement, and whose clause (a) loses its element-position carve-out;
and [ENT-6] 3001's automatic affine-premise sequence, which gains two
specification-fixed members. *Depends:* [ENT-4] 2854, whose difference-bound
uniqueness argument is why the identity is a premise and not an L0 fact; [ENT-5]
2936-2940, whose "no fact established inside an iteration survives to the next
iteration's head" is what keeps an empty-support fact from crossing a backedge.
*Law:* L15, L16. *History:* 6.8, F1 attack 3 and F2 NB3; 6.7, F1-8.

**[MSR-3] Measure datums, images, and what an atom is keyed by.** A **measure
datum** is a compiler-owned immutable [ENT-2] term of fragment type `u64` with
**empty support**: no [ENT-5] event kills it, no place occurs in it, and no later
write retargets it. It is the device [ENT-2] already has for a `for_stmt` capture
and a [SET-1] commit value, extended to one more producer. There is exactly one
former, keyed on what a datum denotes rather than on where the value came from:

```text
a datum is identified by (program point, admitted place P, measure), is
compiler-owned and immutable, and is established equal to <measure>(P) at that
point
```

Four placements exist, and no fifth:

```text
entry placement       body entry, for each parameter of measured type and each
                        measure it has; the datum denotes that parameter's measure
                        at entry
call placement        one call's pre-transfer point [ENT-5], for each operand
                        place of measured type and each measure it has, reading a
                        borrow operand through its resolved referent and an own
                        operand as its value before transfer
exit placement        one call's post-kill point [ENT-5], for each operand place
                        of measured type whose parameter mode is `&uniq` and each
                        measure it has, read through the resolved referent
construct placement   one `construct` [GRAM-8] or enum-payload construction, for
                        each field or payload operand of measured type and each
                        measure it has, read as that operand's value before
                        transfer
```

The borrow half of the call placement is the split [FN-8] 1269 already makes for a
goal actual, applied to the datum former. The exit placement is [CALL-4]'s, and it
is what makes a helper able to tell its caller what it did to a borrowed run; it
exists only for a `&uniq` operand because only there is the referent exclusive for
the call, so the datum is exact rather than approximate.

Three rules read datums and nothing else does. A [FN-9] or [FN-8] clause operand
naming a parameter's measure denotes that parameter's **entry datum** in a
`requires` and its **exit datum** in an `ensures` [CALL-4], so a consuming use of an
`own` parameter cannot invalidate the first and a callee that changed a borrowed
run can publish the second. A [BLK-0] declared relation naming an operand's measure
denotes that call's **call datum**, so it survives the argument consume that the
same statement performs.

**One sentence fixes what an [INV-1] affine atom over a measured place is keyed
by, and it covers both writing forms.**

> An [INV-1] affine atom over a measured place is keyed by the [ENT-2] term. A
> **reinitializing `set`** [LIV-2] is a declaration event: the old term is retired,
> a new one is introduced, and a header invariant over the new term is
> re-established on the backedge from the operation's declared relation over its
> call datum, which has empty support. An **in-place exchange** [LIV-3] is not a
> declaration event: the root's term survives, the facts over it die by [MSR-2],
> and the same declared relation re-establishes them on the same term. Either way
> the invariant is preservable and the derivation is three steps; a form that is
> neither, and that rewrites a measured place a header invariant names, is a
> diagnostic and not a silence.

*Judgment:* none by itself; a datum is formed, never proved. *Publishes:* the
datum, the image, and the atom-identity rule. *Amends:* [ENT-2]'s term list (a new
clause beside its capture and commit-value clauses); [ENT-5]'s call-boundary
paragraph (2892-2899) and its FN-9 entry-image-stability paragraph (2881-2885),
which are replaced by the datum rather than repaired; [FN-9]'s `M(c,q)` (1339, a
datum operand is always live) and its parameter-entry-image sentences (1310);
[ENT-6]'s image formation, join and loop-header paragraphs (2970-2996); [ENT-3.S5]
2768-2776's copy-equality clause, which gains the construct placement's measured
fields; and [INV-1] 3099's atom identity, which gains the sentence above. *Depends:* [ENT-2]
2687, whose one-static-term-per-statement argument is why a per-point datum is
sound; [ENT-5] 2936-2940, whose head-state construction is why a body-placed datum
does not cross a backedge; [FN-8] 1269, whose borrow-versus-own actual split the
call and exit placements reuse. *Law:* L11, L16. *History:* 6.8, F4 blocking 2 and
F1 attack 10; 6.7, F1-4.

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

*Judgment:* the disposition itself. *Publishes:* the disposition of every numeric
goal. *Amends:* [ENT-6] 3034, 3041, 3069 and 3078, the four per-family route and
attach-site grants, which keep their normalization and lose their route grant, and
[FN-9]'s `prove_ordering` route, whose undocumented direct-affine branch becomes
one of the six steps. *Note:* this rule is why the design does not have to be
revisited when the library adds an operation: an operation adds a goal, never a
route. *Law:* L16. *History:* 6.5, F4-3.

**[MSR-5] The contract surface has its own production, over terms.** A `requires`,
`ensures`, `header_invariant`, `invariant_stmt` or `proof_use` operand is a
**term** of the [ENT-2] term language, not an `atom` of [GRAM-5]. The amendment
goes where the refusal is. [GRAM-5] 265's `atom` production has no `call`
alternative, so `ile(len(x), y)` derives nowhere and [GRAM-9] is only [DIAG-1]
1606's attribution of that failure. Probes `w3`, `x5` and `m07` are that rejection,
with the compiler's own mechanical fix naming `define`.

```text
clause_expr    := IDENT "(" clause_operand "," clause_operand ")"
clause_operand := affine_expr
affine_factor  := literal | ent2_place | measure_term | "(" affine_expr ")"
```

`requires_clause` and `ensures_clause` (spec 182-183, [GRAM-2]) take a
`clause_expr` instead of an `expr`; `ent2_place` is [ENT-2] 2675(a)'s place grammar
and `measure_term` is [MSR-1]'s three formers over one admitted place. The relation
is an **IDENT and not a new terminal**, exactly as [INV-1]'s `header_invariant` and
`invariant_stmt` productions already write it (spec 236-238), and it carries
[INV-1] 3099's own sentence in [FN-9]'s wider form: *the IDENT must be exactly
`ieq`, `ine`, `ilt`, `ile`, `igt` or `ige` — [FN-9] 1306's closed root set — and it
selects a proof-domain relation and performs no [OP-1] call, so a `clause_expr` is
not an expression and carries no [OP-5] type judgment.*

*Judgment:* the ordinary [FN-8]/[FN-9]/[INV-1] admission over the widened operand
set. *Publishes:* nothing new. *Amends:* [GRAM-5] 265-266 (a new `clause_expr` and
`clause_operand` production; `atom` and `atom_list` unchanged), [GRAM-2] 182-183's
`requires_clause` and `ensures_clause`, [GRAM-4]'s `affine_factor` production,
[OP-5] 921's contract-predicate scope, [FN-8] 1256-1261, [FN-9]'s operand list
(1306-1308), and [INV-1] 3107's atom sentence; [GRAM-9] is unchanged and needs no
scope sentence. *Depends:* [INV-1] 3099, whose relation-IDENT restriction and
no-call sentence this production reuses verbatim. *Verified today:* probes `w3`,
`x5`, `q1`, `q9`, `r1_lenatom` and `r1_field` are parse rejections, so this is an
amendment and not a compiler defect. *Law:* L16. *History:* 6.8, F3 defect 5;
6.7, F3-6.

#### 3.K.2 `[PROV]`: stores, brand, activation, and release

**[PROV-1] A store's identity is a region, the region is in the type, and a region
names at most one live store at any program point.** This is the rule the design is
built around, and everything else in this family is derived from it.

A **store region** is a region that names one store. A region becomes one by being
named as the store argument of a reserving occurrence [PROV-5], or, for the heap,
by being the entry's own store-region parameter. A region may be named by **at most
one** reserving occurrence; a second occurrence naming a region already used is a
hard error citing PROV-1 at that occurrence's `targ`, with the restructuring `open
one region per store`. [OWN-3] 573 makes region identifiers unique within a
function, and probe `w1` confirms the compiler enforces it.

Every value a store backs carries that store's region in its own type. There are
two stores and one run shape over each, and the table is the whole vocabulary:

```text
| store       | provider                 | one run of slots     | reclamation           |
|-------------|--------------------------|----------------------|-----------------------|
| general     | Heap<'s>                 | Vector<'s, T>        | per run, by dispose   |
| bump extent | Arena<'s, bytes, align>  | Vector<'s, T>        | with 's, by reset     |
| (none)      | (none)                   | FixedVector<T, n>    | with its own scope    |
```

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
the same store exactly when their types name the same region, which [OWN-12] 645 and
[TYPE-5] 374 decide by exact identity. All four round-4 reports attacked this from
every position they could build and none moved it; 6.8 records the routes.

**The brand's spelling is 3.K.0's assumption, and this rule adds nothing to it.** A
store brand is one more region argument, so which spelling of it is canonical is
decided by the separate region-spelling amendment and not by a rule of this family.
Applied here it says: when the entry selects `command.heap` and the enclosing
nominal declares no region parameter, the brand's only possible value is the heap's
store region and it is not written; when the same store appears at two positions of
one declaration, it is written at both.

What this rule owes that amendment is the **candidate set at a stored position**: the
enclosing nominal's own region parameters, plus the entry heap's store region when
the entry selects `command.heap`. That set is read from the nominal's own declaration
and from the entry's one input row, never from a callee, a caller, or an
instantiation, which is what makes the spelling decidable from the declaration text
alone. At a parameter or result position a brand that relates nothing is an implicit
region parameter instead, so a helper handed a run is generic over its store without
saying so.

**The provider parameter itself is never elided.** `heap: &uniq Heap` keeps its
parameter, its mode and its effect row, because that is the signature-visible
allocation fact L2 exists to create; what goes is the region *inside* the type and
the region of the borrow, never the parameter. A signature that allocates still says
so at its parameter list and at its `allocates` row, and [PROV-4]'s reachability
closure reads exactly those. [OWN-3] 575 makes the entry heap's region incomparable
with every other and no second general store can be formed, so the elided brand can
denote nothing else. `struct Bytes { v: Vector<u8>; }` is then an ordinary nominal
with no region parameter, and `CONTAINERS.md` §3 writes `byte_string.wf`'s join with and without
the brand so the difference is visible rather than argued.

`Heap<'s>` is delivered as an `own` entry parameter and lives for the program.
The `command` standard-input table [FN-7] gains ordinal 5:

```text
| ordinal | label        | written mode and type | supplied value                                      |
|---------|--------------|-----------------------|-----------------------------------------------------|
| 5       | command.heap | own Heap              | the one general store the runtime minted before main |
```

and the entry may declare **exactly one region parameter**, admitted only when it
selects that row; program start supplies it and it outlives every region of the
program. Under 3.K.0 the entry writes it only when it also reserves an arena. The
`Heap` `main` receives is dropped on the return edge with the **empty** release row:
the store is the runtime's, the program returns the handle, and no covered
acquisition or release happens there.

*Judgment:* one live store per store region, established by [PROV-5]; provider and
branded types are nominal and closed, and no source declaration introduces another;
the elision resolution above; plus the ordinary [FN-7] label, order, mode and type
checks. *Publishes:* each value's store, as a component of its type; the store's
measures; and the whole-program fact `heap-unreachable` when the entry row is
absent. *Amends:* [TYPE-2] 352, which gains the five branded and
container nominals below and from which `box<T>`, `arena<'r, T>` and `buffer<T>` retire from the
writer surface; [TYPE-7] 471, whose closed deref domain becomes `&'r T` and
`&uniq 'r T` alone, because a single stored value is a run of capacity one and is
reached by subscript; [GRAM-3] 204-207, whose fixed `box`, `arena`, `slice` and
`buffer` type productions retire in favour of ordinary TYPEIDs with `targs`, and
which gains the omitted-store-region form; [OWN-10] 636-640, whose `arena<'r, T>`
content clause becomes a clause over `Vector<'s, T>` content with `'s` in the
subject position; [FN-7]'s table (1221-1227), its "declares no region parameters"
sentence (1212), its canonical five-input byte sequence (1246), and its effect-row
sentence (1214), whose `allocates(heap)` becomes `allocates` over the entry's own
labelled provider input. *Depends:* [OWN-3] 573 and 575, for uniqueness within a
function and incomparability across the boundary, which is also why the elided brand
can denote only one region; [OWN-12] 645 and [TYPE-5] 374,
for exact region identity in type equality, which is the whole of the invariance
argument. *Law:* L2, L13, L16. *History:* 6.8, F1 attack 1 and F4 finding 6; 6.7,
F2-NA1.

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
*Amends:* [OP-1] 793-798, from which `box_new` and `arena_new` retire, and [STOR-2]
680, which defined them; [STOR-5] 718-732, whose enumerated stored-content
positions gain the provider prohibition. *Depends:* [OWN-10] 636, which is why `'s`
and `'b` are always distinct; [OWN-6] 609, which makes an argument borrow a
call-scoped temporary, the fact probe `w8` exercises and the reason store identity
may not rest on what stands at a place between two calls. *Law:* L2, L3, L4, L13,
L16. *History:* 6.8, F1 attack 7.

**[PROV-3] Provenance is for loans, and a loan reaches a range.** [OWN-5]'s finite
origin set, today defined for `slice<'r, T>`, generalizes to the two views and to
nothing else. A **loan-bearing** type is `Span<'r,T>` or `MutSpan<'r,T>`; a value of
one carries a finite set of origins, each an origin place paired with the half-open
index range the value reaches of it, **and, per measure of that place, the
formation datum [MSR-3] minted when the origin entered the set**.

Formation makes a **singleton**: `seq_mut_span(vector: &uniq 'w table[i])` has the
singleton origin `table[i]` with range `[Z, len(table[i]))` and that formation's
own datums. A named const maps to the distinguished `immutable-const` origin.
Binding, moving, passing and returning preserve the set, its ranges and its datums;
a control-flow join takes the union; a parameter of loan-bearing type starts with
the singleton containing its own formal origin, substituted at a call boundary by
exactly the rule [FN-1] 1035-1041 already applies to the origin place. The
**resolved** origin set is the set minus `immutable-const`, which creates no
conflicting access and has no writable storage [OWN-5] 602, [OWN-7] 627.

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
   view descriptor**; [LIV-3] is the rule that governs a write to a loan-bearing
   place. Splitting the two is round 4's repair: the fourth draft wrote one
   sentence whose premise was storage-keyed and whose operative clause was
   type-keyed, and the type reading refused the central statement of both worked
   programs.
4. **Disjointness.** [OWN-7] 625's overlap test extends to ranges: two origins with
   the same resolved place overlap exactly when their ranges intersect, judged by
   the same affine reasoning [PAR-2] 1999 already performs for a single-binder
   element write. This is what makes a `par` fill over one owner expressible.

Use 2 is checkable only because [OWN-7] 625's subscript overlap stays
conservative, and the register's `Depends:` list carries that.

*Judgment:* a loan-bearing value in a prohibited position [BLK-4] is a hard error
there; a write to the storage a live resolved origin describes is the ordinary
[OWN-5] conflict, at the write, naming the loan; and a write to a binding a live
loan's address computation reads is the same conflict. *Publishes:* the origin set,
the resolved origin set, each origin's range, and each origin's carried formation
datums. *Amends:* [OWN-5] 589-607, whose slice-origin paragraphs generalize to
loan-bearing values, whose one access clause becomes the two of use 1 over ranges,
which gains the address-computation, resolved-set and carried-datum sentences, and
whose 603 becomes "a formal view origin has a writable storage path inside its
callee exactly when that view's loan strength on its resolved origin set is
exclusive", the callee-side twin of the [SET-1] change below, its second sentence
unchanged; 596-599's no-slice-valued-join sentence, restated over the loan-bearing
predicate rather than over one retired type name, because the union of two loans is
not a loan any rule can end at one consume; [OWN-7] 625, which gains the range
clause; [SET-1] 483-485, whose "no writable target path may traverse a `slice<'r,
U>` value" is restated as *a target path may traverse a view value exactly when
that view's loan strength on its resolved origin set is exclusive*, which is what
admits the `MutSpan` element write probe `p7` is refused today; [SET-2] 508-513,
whose region-bearing target rejection is replaced by use 3 and [LIV-3]; [EFF-1]
1380, whose "for a direct `slice<'r, T>` parameter, [an effect path] names the
viewed backing state rather than the descriptor" generalizes to a loan-bearing
parameter, which is the declaration-side half [CALL-3] and [VIEW-7] both read; and
[EFF-2] 1400-1404, whose slice-parameter projection generalizes the same way.
*Depends:* [FN-1] 1035-1041, whose call-boundary origin substitution is what carries
a formation datum into a callee and back; [OWN-7] 625, whose conservative subscript
overlap is what makes use 2 checkable. *Law:* L10. *History:* 6.8, F3 defects 1
and 2, F1 attack 9.

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

*Judgment:* [EFF-2]'s both-ways row check, unchanged. *Publishes:* the
provider-reachability closure, and the heap-reaching path, which is the ordered
call chain from `main` to the allocation that [RES-4] prints. *Amends:* [EFF-1]'s
`effect` production (1363-1372), retiring the effect-row atoms `heap` and `arena`;
and [FN-3] 1117-1121, whose conformance effect-row normalization is defined over
"the allocation set whose members are `heap` and each alpha-mapped `arena` region"
and which becomes the set of `allocates` paths under the same parameter-ordinal and
field-ordinal identity 1121 already fixes for `reads` and `writes`, with the region
alpha-mapping applying to modes and types only. *Depends:* [PROG-1] 1486, whose one
closed compilation unit with no function values is why the closure is exact.
*Law:* L2. *History:* 6.7, F3-12.

**[PROV-5] Reservation is an event of the region block, and one live activation is
the condition.** Two reserving operations exist, differing only in placement:

```text
arena_frame<const bytes: u64, const align: u64>['s]()  -> own Arena<'s, bytes, align>
arena_extent<const bytes: u64, const align: u64>['s]() -> own Arena<'s, bytes, align>
```

No operand supplies any of those parameters, so each call writes its complete list
in [GRAM-2] 193-194's declaration order, type and const parameters then region
parameters, with each const parameter a lowercase IDENT as [FORM-3] requires:
`arena_frame<4096, 16, 'a>()`. The written region argument `'s` must be a region
introduced by an enclosing `region_stmt` of the reserving function; a
caller-supplied region parameter is not admitted, and [PROV-1] admits at most one
reserving occurrence per region.

**Each reserves one store per activation of the region block naming `'s`.** The
`frame` form lays the extent out in the reserving activation's frame, so it enters
that context's `stack` item of `E`; the `extent` form produces its own
`region(name, bytes, alignment, contiguous)` item of `E`, whose name is derived
from the reserving occurrence and is not written. **On every edge leaving `'s`'s
block the store's release action resets it to its initial state**: the bump cursor
to zero, and nothing else. That action joins [STOR-3]'s release-action table.

The refusal is therefore stated over the property and not over an enumeration of
the ways to reach it:

> An `arena_extent` occurrence is a hard error at its `targ` when the reserving
> function is a member of a strongly connected component of the call graph, or is
> reachable from more than one execution context. The restructuring is `reserve the
> store in the caller and lend the provider down [PROV-7], or use the frame form`.

*Judgment:* the ordinary region, confinement and [OWN-5] exclusivity judgments,
plus the region-locality check, [PROV-1]'s one-store-per-region check, and the
activation refusal above, each a hard error citing PROV-5 at the `targ` with the
restructuring stated there. *Publishes:* the reserved store's measures, its store
region, its envelope item — one `stack` contribution or one `region` item — and the
one-live-store-per-region invariant [PROV-1] reads. *Amends:* [STOR-3] 683-715,
whose release-action table gains the store reset; nothing else, and the deleted
third clause is why. *Depends:* [ERR-4] 1481, whose
"absence of a complete permission derivation ... never rejects the source" is why
the third clause is gone. *Law:* L2, L5, L6, L13. *History:* 6.8, F1 attack 1, F2
NB2, F3 defect 8.

**[PROV-6] Linearity refines affine; disposal, destructuring and the partial-move
refusal are one closure.** A type is **linear** exactly when it reaches, at any
depth, a value whose backing is reclaimed per value: `Vector<'s, T>` where `'s` is
a `Heap` region, any nominal with a linear field, any enum with a linear payload,
any run whose element type is linear, and any view whose element type is linear.
A type whose backing is reclaimed with a region or with a frame is not linear:
`Vector<'s, T>` at an `Arena` region and `FixedVector<T, n>` over non-linear
elements keep their ordinary compiler-derived release, and a provider is not linear
because [PROV-5] gives its store a reset.

**Linear is a property of an affine type and not a third class.** [OWN-1] 558-559's
classification is unchanged; every linear type is affine; the linear predicate is a
further property of an affine type, defined here, that removes its
compiler-derived release action and fixes its scope-exit disposition. That one
sentence is what lets `move p`, [OWN-13]'s own-place match move, [SET-2]'s affine
target requirement and [ERR-3]'s `propagate` operand reach a linear value at all;
the fourth draft declared the class *beside* copy and affine, and under that
reading no rule admitted `move` or `dispose` of the values the design exists to
manage.

A linear value has **no compiler-derived release**. It leaves a scope by exactly
three routes, and the three are closed under containment together:

```wf-design
let tail = move queue;                        // moved out whole
let Chunk(page: p, used: u) = move chunk;     // destructured whole
dispose table using (heap);                   // disposed to its store
```

**Destructure whole.** `let N(f1: b1, ..., fk: bk) = move v;` is one added
`let_stmt` alternative that consumes a value of nominal type `N` and binds every
field of `N` in declaration order to a fresh IDENT, judged exactly as [CALL-4]'s
multi-result destructuring `let` is: each binder is an independent destination,
each receives its field's declared type and `own` mode, and no residual exists for
any rule to define. It is the inverse of `construct`, and it is what makes
"linearity is closed under containment" true of disassembly as well as of assembly.

**Dispose.** `dispose p using (q1, ..., qk);` consumes the owner place `p`; each
`qi` is a **writable provider place**, reached directly or through a borrow —
`dispose item using (deref(heap));` is the spelling inside a helper — and the
statement takes one statement-scoped exclusive access to each, exactly as a [SET-1]
commit does to its target. That is why no `dispose` needs a region of its own. **Its
judgment is a walk of `p`'s type**, and the walk is stated over the type's variant
structure rather than over a flat leaf set:

```text
for a struct or a run element type: every field in [STOR-3]'s order
for an enum:                        the active variant's payload, selected by the discriminant
for a run:                          every element of [0, len), in ascending order
at a linear leaf:                   release to the store its own type names
at a non-linear leaf:               that leaf's ordinary derived release action
```

For every store region that `p`'s type names at a linear leaf, exactly one named
provider whose type names that region must appear, and no named provider may be
unused. **A container's elements are visited before its backing is released**, so
`dispose` on a full container is legal and needs no emptiness premise.

**The walk's depth is the disposed type's containment height, a compile-time
constant, and the walk therefore uses no auxiliary storage.** That sentence is
round 4's rank-one resource finding and it is a language decision, not a codegen
one: probe `a8` shows today's derived drop of a self-referential owned type
emitting a worklist grown by `realloc` with `wf_resource_abort` on refusal, and
probe `x6` shows that `struct Node { next: Option<Vector<Node>>; value: u64; }` is
**accepted today**, so the shape is reachable. A type whose containment graph has a
cycle has no compile-time height; its release is then an unbounded demand on the
cleanup-scratch domain [RES-5], which is exactly how [RES-3] premise 3 refuses any
other unbounded store, and the diagnostic names the type and the cycle. An unmarked
program keeps today's behaviour and 6.3 records that its abort site survives.

**A partial move of a value of linear type is a hard error.** [OWN-1] 564's "after
any consuming use, the whole binding rooting `p` is dead (partial moves kill the
whole binding)" is the one event that makes a linear binding *not live* without
disposing it, and both [LIV-1]'s check and this rule's own error are stated over
live bindings, so the abandoned sibling leaves its scope by none of the three
routes and no rule sees it. Probe `x4` shows the shape is accepted today, and the
same statement is the mechanical route around the `propagate` refusal below. The
refusal is stated where the death happens, and its mechanical fix names the
destructuring form.

`propagate` and a live linear binding are mutually exclusive, and this rule says
so rather than leaving it to be discovered. A `propagate` error edge leaves every
enclosing scope and offers no statement position on which to dispose, so a
`propagate` in a function holding a live linear binding is a hard error citing
PROV-6 at the `propagate_let_rhs`, with the restructuring `expand the propagate
into a match and dispose on the Err arm`. Probes `w5` and `m03` compile that shape
today, so this is a refusal the design adds and a cost it owes the writer; Q10 asks
whether a release list on the statement should later remove it.

*Judgment:* a linear binding live on any edge leaving its scope, including a
`propagate` error edge and a function-return edge, is a hard error citing PROV-6 at
that edge, naming the binding, its store regions, and the providers a `dispose`
would need; a partial move of a value of linear type is a hard error citing PROV-6
at that `move`, with the restructuring `destructure the whole value with
let N(f: a, ...) = move v;, or dispose it`; a `dispose` whose named providers do
not cover the store regions of `p`'s linear leaves exactly once is a hard error
citing PROV-6 at the statement, rendering the uncovered region and the type path
that reaches it; and a `dispose` of a type whose containment graph has a cycle
denies [RES-3] premise 3 at that statement. *Publishes:* the release events, each
store's post-state measure, and the walk's effect contribution. *Amends:* [STOR-3]
683-715, whose `box<T>` and `buffer<T>` drop rows retire with their types, whose
release-action table gains the statement that a linear type has none and the split
between a store's reset and its content's release, and whose 704-707 system-resource
release contract gains a second subject [RES-9]; [OWN-1] 558-564, whose
classification is unchanged and which gains the linear refinement and the
partial-move refusal; [GRAM-4]'s `stmt` and `let_stmt` productions (one added
statement form and one added `let` alternative) and [FORM-2], which renders each as
one line; [EFF-2] 1421's "each of these memory-reclamation actions carries the
empty effect row", which stays **true** for the four types it names and is joined by
the walk's own contribution; [PAR-1] 1969's footprint, through the ordinary `writes`
row, and [PAR-1] 1990's admitted intervening-statement list, which gains the
`dispose` statement so that a permitted window containing one is not denied.
*Depends:* [STOR-3] 694-700, whose derived-drop order and its affine-element clause
are the walk this rule reuses. *Law:* L3, L5, L13, L17. *History:* 6.8, F3 defect 3,
F1 attack 2, F2 NB1, NB5, NB6, F4 blocking 1.

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
[OWN-6] 611 and [OWN-4] 577. *Verified today:* probes `r1_relend` and `m19` are
`[OWN-6] InvalidChildReborrow`, and `r1_relend_affine` shows the existing
local-region escape cannot carry an affine result out. *Note:* this also unblocks
`docs/patterns.md` P17's threaded-factory shape. *Law:* L2. *History:* 6.8, F4
finding 9; 6.6, F2-N3.

#### 3.K.3 `[BLK]`: the branded run of slots

**[BLK-0] The kernel declaration domain.** The container and store operations are
one compiler-owned **generic** declaration domain, built as [SYS-1] and [SYS-2]
build the system domain and admitted to every compilation unit on the same terms.
Each operation is one complete signature record: named parameters in declared order
[GRAM-11], its type, const and region parameters written as [GRAM-2] 193-194 orders
them, one declared effect row, one declared result mode and type or one ordered
result list, one declared requirement list, and one declared relation list.
**The first declared parameter is the value the operation transforms and returns;
an operation that transforms nothing names its provider first.** The inventory is
Appendix A.2; the rule is that it exists and that every row satisfies the five
sentences below.

**Written arguments.** A row writes its complete type, const and region argument
list in one `targs` list exactly when some parameter of that list is supplied by no
operand; otherwise it writes none. A partial list is not a spelling option. This is
[TYPE-5] 369's own criterion applied to a domain, and a written type argument may
itself be branded.

**The argument form is named.** A kernel-domain call writes its value arguments as
a `fieldinit_list` in declared order, exactly as a user `fn` and a system operation
do. [GRAM-11] 341 admits that form for exactly "a user `fn` or ... an admitted
system operation", 343 forces positional operands for an [OP-1] table operation,
and 345 resolves callee kind by "the same partition that already selects the
callee", which [OP-1] 833 states. A kernel-domain operation is a fourth class in
all four sentences, [OP-1] 833's included, and [TYPE-6] 396's `callee` IDENT
admission gains it too.

**Every row is complete over the measures it writes.** A row carrying `writes(P)`
for a measured `P` publishes, for each measure of `P`, its exact new value where
that measure is exact and a two-sided bound where it is monotone, including the
measures it does not change and **on every exit including a refusal** (L15). Where
two of the three follow from [MSR-2]'s identity the row states one and Appendix A.2
says so. This is the discipline whose absence killed an arena's cursor on its
refusal edge.

**The readers are not in this domain.** `len`, `cap` and `room` are three [OP-1]
table operations taking a bare non-consuming place operand and returning `own u64`,
and they are **`pure`**: the operation reads no state the caller does not already
hold, and [EFF-2] attributes the operand's own read exactly as it does for any
other non-consuming table operand. Probe `r2_10` shows the consequence today. **A
`let` binding one of them establishes an equality**: [ENT-3.S6] 2779's row, today
`let m = len(P);` for a tracked `P` establishes `m = len(P)`, generalizes over
[MSR-1]'s three measures, so `let spare = room(v);` establishes `spare = room(v)`
with the same support [MSR-2] gives the term. Without that one row no `cap` or
`room` value is ever a fact, every branch on capacity is a fresh unrelated atom,
and the whole checked half of 3.L is unwritable — 3.L.6 records it as one of the
six.

*Judgment:* row resolution by name, receiver type and written arguments; the
per-row requirement discharge under [MSR-4]; and the [GRAM-11] named-argument
check. A diagnostic for an operation cites **[BLK-0]** and names the operation in
its payload, exactly as an [OP-1] diagnostic cites [OP-1]; [DIAG-1] 1535 admits one
numbered language rule and the inventory rows are table data, not rules.
*Publishes:* every declared relation of every row. *Amends:* [SYS-1] 2130 (a fourth
admitted declaration source), [SYS-3] 2303 (admitted to every unit), [TYPE-6]
391-403 (the operation spellings enter the lexical IDENT domain, the nominals the
TYPEID domain, and 396's `callee` IDENT admission gains the fourth class),
[DIAG-1] 1687-1712 (collision rank 5, and a `container_declaration_ordinal` beside
the system one), [ENT-3] 2724 (one added enumerated source S13, plus the arm route
above) and [ENT-3.S6] 2779 (the equality row generalizes over the three measures),
[OP-1] 766-845 (`len` gains `cap` and `room`, their domain extends to runs, views
and providers, and `slice_of`, `buffer_new`, `buffer_vacant`, `box_new` and
`arena_new` retire; `ReservedLowerNames` gains `cap` and `room`; 833's callee
partition gains the fourth class), [TYPE-5] 369 (the written-argument criterion
covers a fourth callee class), [GRAM-11] 341-345, and [FN-2] 1087 (its
explicit-argument rule covers this domain). *Law:* L11, L15, L16. *History:* 6.8,
F3 defects 6 and 7; 6.7, F3-1.

**[BLK-1] Two runs, one shape, and what a slot may hold.** Exactly two container
nominals, differing in two properties for two reasons:

```text
| type                | capacity            | storage              | linear        |
|---------------------|---------------------|----------------------|---------------|
| FixedVector<T, n>   | the type constant n | inline in its owner  | never by      |
|                     |                     | or the stack frame   | itself        |
| Vector<'s, T>       | a measure, fixed at | one run taken from   | when 's       |
|                     | the take            | the store 's names   | reclaims per  |
|                     |                     |                      | run           |
```

Each is a run of slots whose **initialized storage is exactly `[0, len)`** and whose
`[len, cap)` is raw. `len`, `cap` and `room` are [MSR-1]'s terms with [MSR-2]'s facts
and [BLK-0]'s readers. A run carries no other state: no per-slot tag, no occupancy
bitmap, no head offset, no runtime discriminant (L12). A `Vector<'s, T>` of capacity
one is a single stored value, so the language needs no box nominal and [TYPE-7]'s
deref domain loses its three. `array<T, n>` is retained exactly as it is, as the
`len = cap = n` case with no typestate and a copy-only element domain, so
`tests/programs/fir_filter.wf` is untouched.

No operation, no subscript, and no borrow yields a place outside a run's
initialized region. A subscript on a run or a view carries the ordinary [OP-4]
obligation `ilt(index, len(base))`, against `len` and never against `cap`. There is
no uninitialized read to reject, because there is no spelling that reaches one.

`T` may be copy, affine, or linear. The initialized region is what makes an affine
element sound: an element enters and leaves only through an operation that moves the
boundary or exchanges two initialized positions, so no slot is read before it is
written or after it is taken. A run over a linear `T` is itself linear [PROV-6], and
`dispose` walks it.

*Judgment:* the ordinary nominal-resolution and construction judgments; a
`construct` naming a container nominal is a hard error citing BLK-1; [OP-4] at
every subscript, against `len`. *Publishes:* the two types, their measure rows and
their typestate. *Amends:* [TYPE-2] 352, two added composite types, and its
flat-element restriction, which the runs do not inherit; [OP-4] 909, whose
indexable bases extend to the two runs, `Span` and `MutSpan`, and whose obligation
is against `len`. *Verified today:* `array_new<box<u64>, 4>` is [OP-1]
`InvalidOperation` (probe `p9`), so an affine element is new capability. *Law:*
L12, L13. *History:* 6.8, the minimality ruling; 6.5, F1-10.

**[BLK-2] Formation, one row per placement and one per store.** Five rows, and no
sixth:

```text
seq_fixed<T, const n: u64>()                       -> own FixedVector<T, n>          pure
seq_frame<T, const n: u64>['s]()                   -> own Vector<'s, T>              pure
seq_arena<T>['s](arena: &uniq Arena<'s,bytes,align>, count: own u64)
                                                   -> own Option<Vector<'s, T>>
seq_arena_proved<T>['s](arena: ..., count: own u64) -> own Vector<'s, T>
seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)
                                                   -> own Option<Vector<'s, T>>
```

**Every failure is an `Option` and the kernel declares no failure nominal.** L3
requires a failure to hand back every affine input it did not consume, and no
kernel acquisition takes one: a count is copy and a provider is borrowed. So the
three failure structs and the fourth draft's `NoRecord` all leave the kernel, and a
library that wants to return an owner inside a refusal declares its own struct over
its own type — `CONTAINERS.md` §3 writes one. The `Heap` has **no proved form**, because no
honest domain predicate exists for a general store (L6); the arena has one, whose
requirement [MSR-4] discharges and whose failure is a static rejection with no
fallback, exactly as an unproved subscript is.

*Judgment:* [BLK-0] row resolution, [MSR-4] discharge at the proved spelling, and
[PROV-5]'s judgments at `seq_frame`. *Publishes:* each run's measures, and each
store's post-state measures and refusal relation. *Amends:* [OP-1] 793-798
(`buffer_new`, `buffer_vacant`, `box_new` and `arena_new` retire); [TYPE-2] 352.
*Law:* L3, L4, L6, L8, L18. *History:* 6.8, F3 defect 4 and the minimality ruling.

**[BLK-3] Three operations move the boundary, and nothing else does.** `V` is
either run type.

```text
seq_place(vector: own V, value: own T)  -> own V
    requires igt(room(vector), Z)
seq_take(vector: own V)                 -> (rest: own V, value: own T)
    requires igt(len(vector), Z)
seq_exchange(vector: own V, first: own u64, second: own u64) -> own V
    requires ilt(first, len(vector)), ilt(second, len(vector))
```

Element access is the ordinary v0.40 surface over the initialized prefix: `v[i]`
reads, `set v[i] = e;` writes a copy element [SET-1], and `let old = replace v[i] =
e;` exchanges an affine one [SET-2]. That surface is what a ring and a keyed table
are built out of, and probe `x7` compiles its shape today.

Each takes the run **by value** and returns it, carries `reads(vector),
writes(vector)`, and publishes its complete measure row.

There is **no removal from the middle, no removal from the front, no clear, no
truncate, no growth, no filled construction and no vacant construction** in the
kernel. Each is written in wf in 3.L, and 3.L.6 records that none of them needed a
primitive the three rows above do not have.

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge of each requirement.
*Publishes:* each row's declared relations. *Amends:* nothing beyond [BLK-0]'s.
*Verified today:* probe `c8` shows a function writing one position of an
`own buffer<u8>` parameter and returning it must exhibit `writes(vector)`, so these
rows are not `pure`. *Law:* L4, L9, L12, L15, L18.

**[BLK-4] Confinement, and the one position closure.** A type is **confined** when
its complete type after substitution names a region. The confinement of a value is
the **set** of regions its complete type names, and it may be moved, returned, or
bound to a destination that **every** member outlives-or-equals [OWN-3]. That
quantifier is the whole rule: a value of type
`Result<Vector<'s, Page>, Shortfall<Vector<'q, u64>>>` names two regions, which
[OWN-3] 575 makes incomparable, and fail-closed is the right answer.

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

and is used as `Chunk<'s>`, an ordinary TYPEID with `targs`. Under [PROV-1]'s
elision a nominal over the entry heap declares no region parameter at all and is
written `Chunk`; the parametric form is what a program with two stores writes. Two
instances of one such nominal have the same type only when their region arguments
are identical: region parameters on a nominal are **invariant**, which is [OWN-12]
645 and [TYPE-5] 374 applied where they already apply, and which is why this
feature needs no variance design.

*Judgment:* a loan-bearing or provider type in a prohibited position, or a confined
type in a position whose owner does not name its region, is a hard error citing
BLK-4 at the complete contained `type`, with the restructuring `keep the view as a
direct local, parameter, or result` for the first, `lend the provider to the
operation that needs it` for the second, and `give this nominal a region parameter
and confine the field to it` for the third; and a confined value bound to a
destination some member of its region set does not outlive is a hard error citing
BLK-4 at the binding, rendering every member. *Publishes:* the confinement set.
*Amends:* [STOR-4] 716, whose "may not be returned" becomes the ordinary outlives
relation over the set; [STOR-5] 718-732, whose enumerated position list is replaced
by the intensional split above and whose deferral of per-leaf provenance inside
stored values is **withdrawn as unnecessary** rather than discharged, because a
store brand is a type parameter and needs no per-leaf record; [FN-2] 1087, whose
blanket rejection of a region-bearing generic argument narrows to loan-bearing and
provider arguments and whose "instantiation arguments are always explicit" now
covers region arguments on nominals; and [GRAM-2]'s `struct_decl` and `enum_decl`,
which gain `region_params?` after `generics?`. *Depends:* [OWN-3] 575, whose
fail-closed incomparability is the invariance argument. *Verified today:* probe
`f7_regionresult` is [FN-2] `RegionBearingGenericArgument` and probes `r2_6` and
`m05` are [GRAM-2] parse errors at `struct Wrap['p]`, so both halves are new.
*Law:* L10, L13. *History:* 6.7, F1-6.

*[CNT-1] through [CNT-7] and [SEQ-0] are deleted.* Five owners, a per-owner release
table, a `&uniq`-container prohibition, a growth rule and an operation-domain rule
are [BLK-0] through [BLK-4] plus 3.L. [CNT-7] is worth its own sentence, because it
is the one whose deletion adds capability rather than removing text: it refused a
`&uniq` parameter whose direct type is a container, and round 4 showed the refusal
is nullified by a one-field wrapper struct while costing every writer their
container helpers. The shape it was protecting is refused where it should be, by
[CALL-5]'s conservative kill: a projected callee write through a `&uniq Vector`
kills the caller's measures over that origin, which is exactly the sweep's sound
repair and is why probe `x11`'s accepted program becomes a rejection. The ids are
retired and not reused.

#### 3.K.4 `[VIEW]`: views and loans

**[VIEW-1] The two views.**

```text
| type            | reads | writes elements | changes length     | loan      | affine |
|-----------------|-------|-----------------|--------------------|-----------|--------|
| Span<'r, T>     | yes   | no              | no                 | shared    | yes    |
| MutSpan<'r, T>  | yes   | yes             | no, fixed by type  | exclusive | yes    |
```

Each is an `own` affine value carrying a region `'r`, exactly as `slice<'r, T>`
does today, and each is loan-bearing [PROV-3]. `Span<'r, T>` **is** today's
`slice<'r, T>` renamed; the rename is the whole of the change to it. Its measures
are [MSR-1]'s rows.

There is no third view. The fourth draft's `AppendView` presented a run's spare
window so that a caller's length could survive an appending callee; [CALL-4]'s exit
datum publishes that for every borrowed value, so the type, its commit event, its
carried formation datum and L14 are all gone (footnote 3). What a writer loses is
the *guarantee* that a callee cannot shrink what it was handed: an appending helper
now takes `&uniq 'a Vector<'s, T>` and could take elements out of it. That is a
contract question, not a memory-safety one, and a `requires`/`ensures` pair states
it; 5.0 records the trade as a decision the owner has not ruled on.

*Judgment:* none by itself. *Publishes:* the two types and their loan strengths.
*Amends:* [TYPE-2] 352 (one added view type, `slice` renamed `Span`), [OWN-1] 558
(both are affine), and [CONST-2] 547-551, [OP-7] 935 and [OP-1]'s `slice_of` row,
which name the retired spellings. *Law:* L10. *History:* 6.8, footnote 3.

**[VIEW-2] Formation, and the loan the view value holds.** A view is formed from a
borrow of the run:

```text
seq_span['r](vector: &'r v)          -> own Span<'r, T>
seq_mut_span['r](vector: &uniq 'r v) -> own MutSpan<'r, T>
```

and **the view value, not the argument borrow, holds the loan**. For its whole
life, a view value holds a loan of its own strength on the range it reaches of
every place in its resolved origin set [PROV-3]. The loan begins at formation and
ends when the view value is consumed or released. The argument borrow is a
call-scoped temporary, which probes `f2b`, `r1_twouniq` and `w8` confirm by
accepting two of them on one place in one region with an ordinary write between; it
could not be the freeze.

*Judgment:* [OWN-5] at the formation borrow, and the ordinary [BLK-0] relation
establishment. *Publishes:* the loan, the two formation relations, and the origin
record's carried datums [PROV-3]. *Amends:* nothing beyond [PROV-3]'s amendment of
[OWN-5]. *Depends:* [OWN-5] 601, the conflict sentence that refuses a second
exclusive view, and [OWN-6] 609, which makes the argument borrow call-scoped.
*Law:* L10, L15. *History:* 6.8, F1 attack 20.

*[VIEW-3] and [VIEW-5] are deleted.* [VIEW-3] was `absorb`, the append window's
commit event, and [VIEW-5] the disposition of an abandoned window. Both retire with
`AppendView`; their ids are not reused.

**[VIEW-4] A view descriptor's length cannot be changed through a borrow.** No
operation takes a `MutSpan` or a `&uniq` to one and produces a different length,
and none changes its owner's length. The ground is two properties of a **borrowed**
view descriptor, and both survive [LIV-2] and [LIV-3]:

- [LIV-2] admits a reinitializing `set` only for a bare binding **declared in the
  current function**, and a `&uniq 'b MutSpan<'r, T>` holder in a callee is a
  borrow of a descriptor the caller declared, so no callee can reinitialize it.
- [LIV-3] admits an in-place exchange at a loan-bearing place only when the
  displaced value is consumed at that commit point; `deref(handle)` in a callee
  holds a view the callee did not form and does not consume, so the premise fails
  there and `replace deref(handle) = ...` is refused with it.

Therefore `MutSpan<'r,T>` and `&uniq 'b MutSpan<'r,T>` are both length-fixed for
[CALL-3].

*Judgment:* none by itself; it is the premise of [CALL-3]. *Publishes:* the
length-fixed class. *Amends:* nothing beyond [PROV-3]'s amendments of [SET-1] and
[SET-2]. *Depends:* [LIV-2]'s bare-binding-declared-in-this-function premise and
[LIV-3]'s consume premise, the two properties above. *Law:* L11. *History:* 6.8,
F3 defect 1 and I19; 6.7, F3-2.

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
version**. That is why 3.L's appending helpers take `&uniq 'a Vector<'s, T>` and
not a view: the `&uniq` container parameter [CNT-7] used to forbid is what a helper
can actually hold. Disposal is not confined this way, because [PROV-6]'s walk
compares types and not places.

*Judgment:* [FN-1]'s ceiling containment at every `return_stmt`, plus the
same-region result rejection. *Publishes:* the result's origin set. *Amends:*
[FN-1] 1017-1030, by generalizing "slice" to "view" and by adding the same-region
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
run rather than the run's own `len`; Q7 records the fix. Its second cost under the fourth draft — two writer-visible regions per I/O site,
because the view's region and the call's borrow region are two regions and [OWN-10]
requires each to be opened after its subject — is **gone**: both relate nothing, so
both are elided under 3.K.0, and Q11 is answered rather than deferred.

*Judgment:* [SYS-8]'s two range obligations, restated over `len` of the borrowed
view. *Publishes:* the endpoint facts [ENT-3.S10] already enumerates, now over a
view. *Amends:* [SYS-8] 2482-2521, [SYS-2] 2158-2301's declaration records and its
normative counts, and the prose of [SYS-9], [SYS-11], [SYS-12] and [SYS-14], which
name `buffer<u8>`. *Depends:* [EFF-1] 1380 as [PROV-3] amends it, which is what
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

*Judgment:* a per-join and per-scope-exit structural check over the ownership state
the checker already computes; no search. *Publishes:* the unconditional release set
of every edge. *Amends:* [OWN-1] 558 and [OWN-11] 641. *Law:* L17. *History:* 6.5,
F1-1, F1-2.

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
a *distinct term* from the consumed one, exactly as [ENT-2] 2677 already rules for
"a fresh binding legally reusing an expired spelling", so a fact stated over the
old value never reaches the new one, and a header invariant over the new term is
re-established on the backedge from the operation's declared relation over its call
datum.

*Judgment:* the deadness premise plus the ordinary [TYPE-5] exact-type check.
*Publishes:* the new binding's term identity and its measure images. *Amends:*
[ENT-2] 2677's term-identity paragraph (one added declaration event), [OWN-1] 564's
"reinitialization requires a new `let`", [STOR-1] 674 and [SET-1] 482-500, whose
affine-target rejection, dead-root sentence and post-right-hand-side revalidation
together carry the old premise. *Verified today:* probe `p10` is [STOR-1]
`AffineSetTarget` for a live target and probe `w6` is [OWN-1] `UseAfterMove` for a
dead one, the two halves this rule replaces. *Law:* L10, L16, L17. *History:* 6.7,
F3-8.

**[LIV-3] The in-place exchange, which is an admission on `set` and not a new
statement.** `set p = f(q: move p, args);` is additionally admitted when `p` is a
writable place of affine type, `f` is any call — a user `fn`, a kernel-domain row,
or a system operation — whose first result has exactly `p`'s type, and `move p`
occurs **exactly once** in `f`'s argument list. Its two-result form is

```wf-design
set p = seq_place(vector: move p, value: byte);
set (p, taken) = seq_take(vector: move p);
```

where the first target is the exchanged place and every later target is a fresh
IDENT bound to the result at that ordinal.

**Its judgment is [SET-2]'s, not [SET-1]'s, and that is what makes it not sugar.**
The previous value is read out of `resolved(p)` into the operation's named
parameter, the operation runs, its first result is written back into
`resolved(p)`, and each later result initializes its binder. There is no
writer-observable program point between the read and the write (spec 515), so there
is no partial move, no dead root and no uninitialized hole, and the root binding
stays live (spec 516).

**This is the one form the partition test could not write in wf**, and the reason
is worth stating because it decides the whole convenience question. At a bare
binding the writer could rebind: `let next = f(q: move p, ...); set p = move next;`
is two statements and [LIV-2] admits the second. At every other place they cannot.
`move p[i]` and `move p.f` are partial moves that kill the root [OWN-1] 564, and
`move deref(h)` is a move through a borrow, which [OWN-5] 614 forbids outright with
[SET-2]'s exchange as the sole exception. So the only route is a placeholder —
`let old = replace p[i] = <something>;` — and a placeholder must be a value of the
displaced type, which for a `Vector<'s, T>` is a run that owns storage and is
itself linear, so every transformation costs an allocation and a disposal on a
provably dead arm, and for a type with no cheap empty value there is no route at
all. Probes `x2` and `x3` are the two rejections today, and `x2`'s own mechanical
fix names the field-by-field fold that is exactly the ceremony this removes.

An exchange is **not** a declaration event [MSR-3]: the root's term survives, the
facts over it die by [MSR-2], and the call's declared relations re-establish them on
the same term through **one added [ENT-3.S12] destination clause**, without which the
row's relations have no destination at the one form that is the only spelling for a
whole class of places:

> The written-back place of an in-place exchange is the S12 destination for every
  > published relation naming the call's first result, and each later target is the
  > destination for every relation naming the result at its ordinal, each
  > established after the statement's own [SET-2] read-out, write-in and kills, in
  > [ENT-5] 2892-2899's existing order, with `M(c,q)` requiring every other
  > referenced support to be live at establishment.

*Judgment:* the single-occurrence check on `move p`, the result-count and type
checks, then [SET-2]'s exchange judgment. *Publishes:* the call's declared
relations, on the written-back place and on each later target. *Amends:* [SET-1]
476-500 (one added admission), [SET-2] 508-524, which gains a compiler-derived
exchange whose replacement value is derived from the read-out rather than written
by the writer, and whose target may be linear or region-bearing because nothing is
rebound; [GRAM-4]'s `set_stmt` production (a target list); [ENT-3.S12] 2827's
destination list (one added clause); and [FORM-2], which renders the form on one
line. *Verified today:* probes `x2` and `x3` are [STOR-1] `AffineSetTarget`, so this
is new capability and not a compiler defect. *Law:* L10, L18. *History:* 6.8, F1
attack 4, F4 finding 5 and blocking 2, and the minimality ruling.

#### 3.K.6 `[CALL]`: what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the
callee's declared parameter modes and types and its declared contract.** These are
the owner's three call rules of 2026-09-03.

**[CALL-1] Through a shared borrow, every fact survives.** For an argument whose
parameter mode is `&'r`, of any type, run and view included, the call is not a kill
event for any fact supported by the actual's resolved place. Ground: [OWN-5] admits
no write through a shared holder, so [EFF-2] can project no `writes` occurrence
onto that place, so [MSR-2]'s kill does not fire.
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
statement performs cannot kill it.
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
storage** and kills no measure term over that origin. For every other parameter
type the projected write kills measures as an ordinary descriptor-storage-overlapping
event [MSR-2]. That "every other" now includes `&uniq 'b Vector<'s, T>`, which
[CNT-7] used to forbid outright: a callee holding one can call `seq_place` and
`seq_take` through it, so the caller's `len` must die, and probe `x11` is the
program whose accept becomes a rejection.
*Judgment:* the kill classification per parameter type. *Publishes:* the surviving
measures. *Amends:* nothing beyond [MSR-2]'s. *Depends:* [VIEW-4], the
length-fixedness this classification reads; [EFF-1] 1380 as [PROV-3] amends it,
without which a view parameter's projected write reaches the descriptor and not the
element storage this rule names. *Law:* L11. *History:* 6.8, F1 attack 9, F4
finding 4.

**[CALL-4] Contract vocabulary, the ordered result list, the exit datum, and where
the relations land.** [FN-9]'s clause operands are terms [MSR-5], so `len(P)`,
`cap(P)` and `room(P)` over an admitted formal place are operands with no
per-family admission. `len(result)`, `cap(result)` and `room(result)` are operands
when the written result type is measured, which today's result-datum restriction to
fragment integers forbids.

```wf-design
fn collect['s, 'd, 'v](out: &uniq 'd Vector<'s, u8>, source: own Span<'v, u8>) -> written: own u64
    reads(source), writes(out) contract {
  requires ile(len(source), room(out));
  ensures ieq(len(out), written);
} { ... }
```

The clause is still **single-state**: no clause names two states of one term, there
is no `old()`, and there is no frame rule. A writer who wants the entry value names
it in a `requires`, and a writer who wants the exit value names it in an `ensures`;
the two never appear in one clause. Two-state `ensures` is rejected by the owner and
is not proposed here.

The exit datum is one of the six things the partition test found the kernel needed
(3.L.6). Without it a helper that changes a borrowed run can tell its caller
nothing, so every capacity proof collapses into the function that owns the run,
every helper boundary costs a re-read and a real branch, and `collect` — the one
program every draft has carried — is unwritable in wf. It is sound because the
`&uniq` is exclusive for the call and [ENT-5]'s order puts the callee's kills before
the establishment; probe `x10` shows the *syntax* already compiles today, read as
the entry image, so this is a semantic addition and not a grammar one.

**Which state a parameter's measure denotes is fixed by the clause, not by the
parameter.** In a `requires` it denotes that parameter's **entry datum**, so a
consuming use inside the body cannot take it away. In an `ensures` it denotes that
parameter's **exit datum** [MSR-3] when the parameter's mode is `&uniq`, and its
entry datum otherwise. The clause is still **single-state**: no clause names two
states of one term, there is no `old()`, and there is no frame rule.

A function may declare an ordered result tuple, and **each result binding is a
datum of every clause of that function**, so one clause may name more than one:

```wf-design
fn render['s](block: own Vector<'s, u8>, task: own Task) -> (rest: own Vector<'s, u8>, written: own u64) ... contract {
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

The same clause serves [PROV-6]'s destructuring consume, whose binders are the
nominal's fields rather than a call's results, and [LIV-3]'s later targets.

*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set and
the widened result shape. *Publishes:* the clause relations, on every result
ordinal and on every `&uniq` parameter's exit datum. *Amends:* [FN-9] 1295-1360
(measured results, multi-datum clauses, the entry-datum and exit-datum operands),
[ENT-3.S12] 2827's destination list (one added clause, serving three forms),
[GRAM-2]'s `fn_decl` result shape, [GRAM-4]'s `let_stmt` and `return_stmt`,
[FORM-2] 52-76's rendering, and [FN-1] 999-1013's result shape. *Verified today:*
probe `x10` compiles a single-state `ensures` anchored on a `&uniq` parameter's
`len` through a `define`, probe `p2` shows `len(result)` does not parse, and probes
`p8`/`k09` show the multi-return signature does not parse. *Law:* L10, L11, L16.
*History:* 6.8, F4 finding 3; 6.7, F3-3.

**[CALL-5] No transport reads the actual's spelling.** The three transports above
are selected by the callee's declared parameter mode and type and by its declared
contract. No rule of this design consults the argument expression's shape, the
callee's body, its name, or any per-parameter summary derived from its body. A
parameter type for which no transport is selected kills conservatively.*Judgment:* the conservative default for every unselected parameter type.
*Publishes:* the absence of a call-site-derived fact. *Amends:* [ENT-5] 2870's
clause (b), whose projected-callee-write kill is now classified by [CALL-1..3] and
by nothing else. *Law:* L11.

#### 3.K.7 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

```text
| class              | members                                                                        |
|--------------------|--------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context, including every       |
|                    | frame-placed arena [PROV-5]; every extent-placed arena and every seq_frame run; |
|                    | every worker-lane stack; allocator and runtime metadata; compiler-derived       |
|                    | cleanup and release-walk scratch; the adapter's persistent buffers              |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots;  |
|                    | the runtime's fixed handle table; every other runtime-owned store               |
```

*Judgment:* none; it fixes the domains [RES-3] quantifies over. *Publishes:* the
covered set. *Amends:* nothing. *Law:* L1, L5.

**[RES-2] The envelope `E`, over the target's profile table.** `E = E(P, T, B)` is,
for one program `P`, one selected target and ABI `T` [STOR-6], **and one build
`B`**, a finite table with one row for each lane count `W` the target's runtime
supports. Each row is a finite list of shaped items:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count)                           interchangeable fixed-size records
stack(context, bytes)                        one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
```

**`E` is a function of three things and not two**, because [STK-3] makes it an output
of code generation: two builds of one accepted program at two optimization levels
publish different rows, and a deployment sizes against the row it was given.

**Which items carry a source-stage figure is stated rather than quantified over.** A
`region` item's bytes and alignment, and a `slots` item's count, are [RES-5]'s
target-independent arithmetic and are read by acceptance; each additionally carries
the target-stage exact figure. A `stack` item has **no source-stage figure at all**
([STOR-6] 759, [STK-3]), so stage one's entire stack content is premise 2 of [RES-3],
acyclicity.
*Judgment:* `E` is well-formed only if every item's arithmetic was performed in
the unbounded mathematical domain and is representable on `T`, the same standard
[STOR-6] already applies. *Publishes:* `E` itself, as a compilation artifact.
*Amends:* nothing. *Law:* L1, L6. *History:* 6.8, F2 NB15.

**[RES-3] The judgment, in two stages.** For a program `P`,
`source-resource-closed(P)` holds exactly when, on the rewritten call graph
[STK-1], every premise below is established from program text alone:

```text
1  no reachable store is a Heap                                    [PROV-4, RES-4]
2  the call graph is acyclic                                       [STK-2]
3  every covered store's demand is bounded, per domain, by the
     symbolic composition of 3.K.7.1                               [RES-5]
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
`E`. *Amends:* [STOR-6] 733-765, whose "the language defines no numeric
per-function frame ceiling" sentence keeps its scope for the *language* and is
joined, for a resource-closed build, by a computed per-context envelope, and whose
target-stage obligations gain `E`-materialization. *Law:* L1, L8, L9. *History:*
6.8, F2 NB17.

**[RES-4] The entry requirement, the heap, and the deferrals it moves.** The entry
may carry the marker `resource_closed` before its `command` program-kind marker:

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

A program whose call graph reaches a `Heap<'s>` is not resource-closed, and a
`main` selecting `command.heap` is by itself the rejection. A bounded general store
is still a general store: an envelope item can promise bytes, and cannot promise
that the next contiguous aligned request has a home.
*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage one
is a hard error naming its own cause: the heap-reaching path, rendered from `main`
to the allocation and located at the offending `input_label` or the deepest `call`;
the call-graph cycle [STK-2]; or the unbounded store [RES-5]. *Publishes:* the
property as a compilation fact, and the scope of [SCOPE-3]'s deferrals. *Amends:*
[FN-7] 1211, which fixes main's marker set; [GRAM-2]'s `program_kind` production;
and [SCOPE-3] 27-31. *Law:* L1, L6.

**[RES-5] Store domains and their algebras, in target-independent arithmetic.**
Every covered store presents its state through [MSR-1]'s measures, and exactly
five domains are defined. Nothing else is admitted, and a store outside this list
contributes no envelope item and denies [RES-3].

```text
| domain                     | state         | acquire            | release        | serviceable when |
|----------------------------|---------------|--------------------|----------------|------------------|
| uniform slots              | len, cap      | len + 1            | len - 1, on    | room >= 1        |
|  (lane, task, queue,       |               |                    | the store's    |                  |
|   completion and handle    |               |                    | own release    |                  |
|   records of the runtime)  |               |                    | event [RES-9]  |                  |
| bump extent                | len monotone, | len + advance<T>   | nothing; the   | room >= advance  |
|  (Arena<'s, bytes, align>) |  in bytes,    |                    | store resets   |                  |
|                            |  cap = bytes  |                    | with 's        |                  |
| general heap (Heap<'s>)    | -             | -                  | per run, by    | undecidable      |
|                            |               |                    | dispose        | from E [RES-4]   |
| static and frame placement | fixed offsets | none at run time   | none           | decided at       |
|                            |               |                    |                | compile time     |
| cleanup scratch            | depth         | +1 per containment | -1 per level   | depth <= the     |
|                            |               |  level entered     |  left          | type's height    |
```

The **cleanup scratch** domain is where round 4's rank-one resource finding lands. A
release walk's depth is the disposed type's containment height, a compile-time
constant [PROV-6], so a non-recursive type's walk needs no runtime structure at all
(probes `a5`, `a6`). A type whose containment graph has a cycle has no height, its
demand is not a closed expression, and premise 3 fails at the release with the type
and the cycle named — probe `x6` shows such a type is accepted today and probe `a8`
shows its drop emitting a `realloc`'d worklist with `wf_resource_abort`.

`advance<T>` is the arena's per-take quantity and is **exact when the store is at
least as aligned as `T`**, which its own type says: for `align >= align_ceiling(T)`
it is `round_up(len, align_ceiling(T)) - len + size_ceiling(T)`, a closed
expression in exactly the constants [RES-3] admits, and `arena_take` requires
`align >= align_ceiling(T)` as a compile-time comparison of two constants.
Otherwise it falls back to the ceiling `align_ceiling(T) - 1 + size_ceiling(T)`.
The ceiling alone costs a proved arena `(align - 1)/(size + align - 1)` of its
extent per take — half of it for a 16-aligned 16-byte record — and putting the
alignment in the type is what makes the exact form available.

*Judgment:* the composition of 3.K.7.1 per domain. *Publishes:* per program point,
per domain, the store's `len` bound. *Amends:* [OP-9] 968-996, whose `buffer_fits`
stays a representability predicate, whose ceiling table gains Appendix A.1's derived
rows, whose region-bearing exclusion is lifted, and which additionally fixes
`advance<T>`. *Law:* L3, L6, L16. *History:* 6.8, F2 NB1, NB11, NB13.

**[RES-6] Typed failure, and the two spellings.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume. **The kernel declares no failure
nominal**, because no kernel acquisition takes an affine input: a count is copy and
a provider is borrowed, so `Option<Vector<'s, T>>` carries everything a refusal has
to carry. A library operation that consumes an owner and may refuse declares its
own one-field struct over its own type; `CONTAINERS.md` §3 writes one and 3.L.2 explains why the
kernel does not.

Each covered-store acquisition with a measure comes in exactly two spellings, on the
model of `+` and `+checked`: a proved form admitted only when [MSR-4] discharges its
goal, and a checked form that is total. **The `Heap` has no proved form** (L6). A
store with measures publishes more: a refused `seq_arena` establishes
`ilt(room(arena), advance<T>)`, which is L8's second half.

The runtime's handle table is a covered store, and its refusal joins the **existing**
`IoError` channel: `reserve_file` keeps `own Result<FilePermit, IoError>` and its
`Err` edge establishes `ieq(room(factory), Z)` when the class is
`ResourceExhausted`. [SYS-7]'s "the class is the sole portable semantic
discriminator" is the reason, and the cost of a second non-unifiable error type is
measured — five broken `propagate` chains in `wfgrep.wf` — for a distinction the
class set already carries. This rule needs the *relation*, not a nominal.

No covered-resource failure is a trap, an abort, a process exit, a retry, or a
promotion to a larger store, in the writer's code or in the runtime. The batch-0079
floor's `wf_resource_abort` site loses its **allocation-refusal** caller once
allocation returns a value and **keeps two others** — the release walk's exhaustion
arm and its doubling-overflow arm — until [PROV-6]'s bounded walk replaces them. The
fourth draft claimed it lost its last caller, which probe `a8` falsifies.

*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result` or an `Option`,
plus [MSR-4] discharge at the proved spelling. *Publishes:* the returned owner's
identity on the refusal edge, and the store's own refusal relation where the store
has measures. *Amends:* [SYS-2] 2255 and 2451, `reserve_file`'s outcome row, which
gains a recoverable failure outcome; [SYS-7] 2467-2481's closed class set, which is
**unchanged** and is the reason no nominal is added; the batch 0079 exhaustion floor
as stated above; and [SCOPE-3] 29, whose "heap exhaustion ... may stop execution at
the host boundary without a Whitefoot value" ceases to be true. *Law:* L3, L6, L8,
L16. *History:* 6.8, F3 defect 4, F4 finding 7, F2 NB1.

**[RES-7] What bare resource-closedness does not cover, and the one exclusion
test.** Disk space, the successful acquisition of a file, socket or other host
object not exclusively reserved before start, network reachability and throughput,
CPU time, deadlines, scheduler fairness, power, device health, host termination,
and OS quota revocation are outside [RES-1] and outside every judgment in this
file. They remain typed system outcomes where the operation defines one, and
environment conditions where it does not.

Which **operations** a marked program may not call is decided by a property, never
by a written list, and **the property is a source-stage record**:

> The [SYS-2] declaration record gains one target-independent column, *acquires
> from*, whose value is a covered store or `none`, written once per operation
> beside its effect row. A system operation is unavailable in a resource-closed
> program exactly when that column names a store that is not an item of `E`.

Applied to v0.40 **that set is presently empty** and the column reads `none` for all
sixteen operations: [SYS-2] 2264 says no system operation allocates, [SYS-9] 2525 and
2543 say `arg_get` and `relative_path` allocate and copy nothing, and the current
backend agrees. Excluding them would have silently deleted every `HostString`, every
`RelativePath` and `open_read` from every marked program.

The column is why this is a **source** judgment. Reading [QUAL-1]'s semantic-ID
record instead makes acceptance a function of the linked implementation, which L1 and
[SCOPE-2] forbid; a qualified implementation needing an undeclared store fails
[QUAL-2] **qualification**, citing no language rule, which is where every other
implementation property lives.
*Judgment:* a call to an operation the column excludes, from a marked program's call
graph, is a hard error citing RES-7 at the `call`. *Publishes:* the boundary.
*Amends:* [SYS-2] 2158-2301's declaration records, which gain the column; [QUAL-2]
2363, whose qualification obligations gain an undeclared-store failure; [ERR-4]
1478, whose "unavailable external resources remain outside the source outcome
model" gains the two families [RES-6] and [RES-4] move inside. *Depends:* [SYS-2]
2264, which is why every column value is `none` in this version. *Law:* L1.
*History:* 6.8, F2 NB7.

**[RES-8] The per-function summary is part of the callable boundary, in three
pieces.** Each function's boundary [FN-1] gains three derived components:

- a **source-stage per-domain map** over that function's formal provider and
  measure terms, substitutable at a call site;
- a **source-stage per-domain saturation flag**: whether every acquisition the
  function performs on that domain, transitively, is one that cannot succeed when
  the store is full; and
- a **target-stage own-storage figure** covering every store it reserves [PROV-5]
  and its own frame.

The flag is what makes 3.K.7.1's second loop discharge compose across a call. It is
derived from the callee's declared rows — a checked spelling has it, a proved
spelling has it when its goal comes from a header invariant — and never from its
body, so [CALL-5] is respected. Without it a caller's loop can never evaluate the
full-store discharge, and the retaining shape 4.1 is written around is refused the
moment its acquisition is one function down.

The three are separate because they belong to different stages, and splitting them
keeps [PROV-4]'s framing honest: a self-reserved store contributes to the third, so
3.K.7.1's call rule never meets a callee demand with no actual to substitute. The map
composes across the one closed compilation unit [PROG-1] and no further.
*Judgment:* none; a boundary statement. *Publishes:* all three components.
*Amends:* [FN-1] 999-1006's boundary list. *Depends:* [PROG-1] 1486, the one closed
unit the composition claim is scoped to. *Law:* L1, L5. *History:* 6.8, F2 NB10.

**[RES-9] The runtime's own stores, and the handle table's five parts.** A covered
store needs five things written in one place: a **capacity**, an **acquire event**,
a **release event**, a **refusal relation**, and a **multiplicity**. The program's
own stores have all five from [PROV-5], [BLK-2] and [MSR-2]. The runtime's have
them from the profile row and the operations that touch them, and the one a marked
program can actually reach — the handle table — needs three amendments that no
earlier draft made, because [SYS-2] and [SYS-10] together deny it.

[SYS-10] 2548-2552 **is amended.** Its sentence "Reserving it promises no native
descriptor, **handle-table entry**, kernel memory, or host quota" is replaced by:
*reserving a `FilePermit` consumes one record of a runtime store whose capacity the
target's profile publishes; the record returns when the permit, or the `ReadFile`,
`DirectoryRead` or `DirectorySource` it became, is released; host exhaustion at the
open is a different condition and remains the ordinary `ResourceExhausted` member
of the open operation's typed `IoError` result, outside `E`.* And its "This first
slice never returns or recycles the permit" is replaced by the release event below,
because a store whose records never come back is a consumable budget and not
reusable capacity, and 3.K.7.1's row would then be wrong about which question to
ask.

[SYS-2] 2295's closed proposition set is **amended too.** It says today that the only
system-result propositions available to source invariants are [SYS-9]'s enumerated
relations and the facts of selecting one typed outcome, and that no unlisted
component establishes any relation. The measure relations of a covered system store
join that enumeration as a named source; without it `cap(files)` dies at the first
`reserve_file` and no marked program can open a file in a loop.

**The release event is the third amendment, and it goes where every other release
in this design is made visible: in the release action's own effect row.** [STOR-3]
704-707 already gives a system resource type "one ordinary state-effect row" for its
release action and already substitutes a formal path for its table-local `owner`
subject. What it lacks is a second subject. A type whose backing is a covered store
names that store in its release row, so `ReadFile`'s release exhibits `writes(owner)`
**and** the runtime handle-table path. The path needs a spelling reachable from a
function that holds no `FileFactory`, and the honest one is the device [RES-5]
already uses for a capacity: the runtime's stores are named by the profile, not by a
formal, and a release row may name a runtime store directly.

Two things then fall out with no further rule: [MSR-2]'s kill fires on
`len(factory)` at every close, and 3.K.7.1's `release one` transfer has a program
event, so a marked program that opens a file in a loop composes a zero backedge
delta. Reclassifying `ReadFile` as linear was considered and refused: it would put a
`dispose` on every close site in the corpus and retire the release-completeness
[SYS-5] grants.
*Judgment:* none by itself; it supplies the fact sources [RES-5] and 3.K.7.1 read,
and its failure is a runtime's [QUAL-2] qualification failure. *Publishes:* each
runtime store's capacity, acquire event, release event, refusal relation and
multiplicity. *Amends:* [SYS-10] 2548-2552 (the reservation's promise and the
permit's recycling), [SYS-2] 2295 (the closed proposition set), [STOR-3] 704-707
(the release contract's second subject), and [SYS-5]'s release-completeness, which
is **kept**. *Depends:* [QUAL-2] 2363, which is where a runtime that cannot publish
a capacity fails. *Law:* L1, L3, L5. *History:* 6.8, F2 NB3, NB4, F1 attack 5.

##### 3.K.7.1 How `E` is composed

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

A statement's summary is **one map from exit label to `(peak, delta)`**. The exit
labels of a statement are its fallthrough, each variant of a result it produces,
each `break` label it may take, and `propagate`. A statement with no fallthrough
label, which after [STK-4] includes a `loop_stmt` no `break` resolves to, carries no
fallthrough entry, and the sequence rule is written so that this is a defined case.

Per resource kind `r`, the primitive transfers are fixed:

```text
acquire one       (peak 1, delta +1)     on the success exit; (0, 0) on a refusal exit
release one       (peak 0, delta -1)     at a dispose, or at a store's own release event
derived release   (peak 0, delta -1)     contributed by a scope-exit edge, per released value
move an owner     (peak 0, delta  0)     moving into a run acquires nothing
borrow an owner   (peak 0, delta  0)
```

A delta may be an integer or an interval `[min, max]`. **An interval enters the
peak equation as its `max` and the delta equation as an interval, and every test
below reads its `max`.** The compositions are:

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
           formal measure and provider terms replaced by the actual ones, and read its
           per-domain saturation flag for the loop rule below

loop       let d be the backedge delta and p one iteration's peak.
             max(d) <= 0  -> peak(loop) = p; delta(loop) = d; no iteration bound is needed
             max(d) >  0  -> the loop is bounded on a domain exactly when the composed
               peak is a closed expression [RES-3], which it becomes exactly through one
               of three discharges, and the loop's own map is stated per discharge:
                 (i)  a compile-time constant trip count T:
                        peak(loop) = p + (T - 1) * max(d);  delta(loop) = T * d
                 (ii) a store whose cap is a standing fact [MSR-2] and whose every
                        acquisition of that domain on the loop's paths, transitively
                        [RES-8], cannot succeed when the store is full:
                        peak(loop) = cap(store);  delta(loop) = [0, cap(store)]
                 (iii) a writer [INV-1] invariant over the measure terms:
                        peak(loop) = the invariant's own target;  delta(loop) likewise
               Otherwise there is no finite E and premise 3 fails here.
           each exit label of the loop carries the loop's own map, not the map of the
           edge that reaches it

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration peak p
                                        and retained d, and K the profile's window
```

##### 3.K.7.2 Which stage decides what

```text
 1  tail-SCC rewrite, source premise [STK-1]        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-4]    source stage   compiler
 4  per-function source-stage demand map and
      saturation flag [RES-8]                       source stage   compiler
 5  loop and branch composition (3.K.7.1)           source stage   compiler
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

**There is no separate target obligation.** An activation record and a frame are
target-stage objects ([STOR-6] 741, [STK-3]), and the fourth draft's obligation was
attached to a transfer its own rewrite removes. Naming the lowering is the repair:
one dispatcher, one frame, no transfer, no ABI condition, and R3's second fallback
stays real for a program carrying a run of any size.

One cost of the first clause is recorded rather than discovered: a component member
that opens a region for an `arena_frame` or a `seq_frame` has a live region at the
jump, so its edge is not a tail edge and [STK-2] refuses the component. Tail
recursion and region-scoped scratch are mutually exclusive, and a writer who needs
both writes the loop.
*Judgment:* per edge, from the ownership and loan state the checker already has;
no proof search. *Publishes:* an acyclic call graph, or a component that is still
cyclic, and the strongly connected components [PROV-5]'s activation refusal reads.
*Amends:* nothing; this is a lowering and not an admission rule, so recursion stays
permitted. *Verified today:* probes `f2b` and `f8_tailframe` are mutual tail
recursions carrying a live borrow of a caller local and are accepted, so the premise
refuses a shape the syntactic list admitted. *Law:* L7. *History:* 6.8, F2 NB14.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After
[STK-1], a program whose call graph still contains a cycle has no finite stack
envelope and is not resource-closed. A `requires` bound on a recursion parameter,
a proof that a recursion argument decreases, and every other depth certificate
are **not** admitted as a substitute.
*Judgment:* under [RES-4], a hard error citing STK-2 that renders the complete
cycle in call order and the restructuring `rewrite the recursion as a loop over
an explicit FixedVector work list, or make every recursive call a tail call whose
caller frame is dead at the jump`. *Publishes:* nothing. *Amends:* nothing.
*Depends:* [FN-6] 1205, whose permission of recursion is why a recursive program is
excluded from [RES-4] rather than rejected. *Law:* L7.

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

The entry context's **initial** stack is part of the deployment grant, and [RUN-4]
checks it like every other item rather than assuming it. A **worker lane's** chain
has no defined root; that question does not arise, because [RUN-2] fixes `W = 1` for
every resource-closed build.

`E` is an **output** of code generation, recomputed after every optimization, which
is why [RES-2] makes it a function of the build.
*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic discipline.
*Publishes:* one `stack(context, bytes)` item per context per profile row.
*Amends:* [STOR-6] 757-761. *Law:* L5, L6. *History:* 6.8, F2 NB15.

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
entries at all. Its one silence is stated: a scope whose exit edge is unreachable
carries no compiler-derived release and no [LIV-1] check, so a **linear** binding
live on a path reaching only such a loop is not an error. It is a **retained item of
the enclosing scope's map** — attributed to the region block or function scope that
would have released it, since the loop has no exit label of its own — so the leak is
visible in `E` rather than invisible in the fact state. No reset runs on that absent
edge either, so nothing observes the retained store.
*Judgment:* [FN-1]'s existing reachability and fallthrough judgment over the
corrected edge set. *Publishes:* the graph, hence 3.K.7.1's exit labels, and the
retained item's attribution. *Amends:* [FN-1] 1070. *Verified today:* probes
`n2_idle` and `f3_forever` are `[FN-1] FunctionFallthrough`. *Law:* L1, L9.
*History:* 6.8, F2 NB9.

#### 3.K.9 `[RUN]`: runtime closure and admission

**[RUN-1] The artifact, runtime closure, and the no-permission obligation.** For
every judgment in this file the artifact is the writer's code, the compiler-derived
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

**And it takes no [PAR-1], [PAR-2] or [PAR-3] permission for any statement or loop
of a marked program.** That obligation lives here, whose subject is already an
implementation, and not in a rule: [PAR-1] 2102 says whether an overlap was performed
at all is not observable and that **no rule of this specification is stated in terms
of it**. The obligation is soundness-critical and the hazard is executed: the current
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

*Judgment:* a target-qualification obligation, auditable from the emitted code and
the runtime's own translation units; its failure is a [QUAL-2] qualification
failure, not a source rejection, and no source construct can weaken or waive it.
*Publishes:* the runtime's own items and capacities. *Amends:* [SYS-2] 2264's "no
system operation allocates", which is kept and given its companion: an adapter
record and a handle-table record are runtime-owned stores of [RES-1] with published
capacities; [RES-1]'s covered set, which the obligation quantifies over.
*Depends:* [PAR-1] 2102, whose unobservability sentence is why the no-permission
obligation is here and not in a rule. *Law:* L3, L5. *History:* 6.8, F2 NB12.

**[RUN-2] `par` enters `E` as a profile, and a marked build publishes `lanes(1)`.**
For each supported lane count `W`, the runtime publishes one finite profile row: `W`
lanes, `W - 1` worker stacks, a task-record capacity `K(W, d)` where `d` is the
program's maximum nested `par` depth, fixed queue capacities, a fixed
completion-record capacity, and the handle-table capacity. The number of iterations
of a `par`-permitted loop never appears in `E`.

What this rule keeps is exactly what **is** a function of program text: **the
profile row a marked build publishes is the `W = 1` row.** The obligation not to
take permission is [RUN-1]'s. Two consequences follow for free: [PAR-3]'s
replicated places, which are execution memory no envelope item counts, cannot occur
in a marked build; and [STK-3]'s undefined worker-lane chain does not have to be
defined in this version.
*Judgment:* a fixed-arithmetic composition (3.K.7.1's `par` rule) against each
profile row for an unmarked program, plus the published-row rule on a marked one;
the compiler emits no per-`W` clone. *Publishes:* the `lanes` and `slots` items of
each row. *Amends:* the sentence common to [PAR-1] 1989, [PAR-2] 2024 and [PAR-3]
2049, "exhaustion of the execution resources an implementation spends on
overlapping is a resource condition under [SCOPE-3] and is not an observable of
this rule": for a program resource-closed on this target that exhaustion is
unreachable, because [RUN-1] takes no permission. *Law:* L5, L9. *History:* 6.8,
F2 NB12.

**[RUN-3] The parallel footprint of an allocation is its provider place, and of a
view its origin range.** In [PAR-1]'s written-footprint clause, "the caller region
each `allocates(arena 'r)` entry names after region substitution" is replaced by
"the places each `allocates` path reaches under the [EFF-2] call-boundary
projection", the same projection the rule already applies to `reads` and `writes`.
Two statements that allocate from one provider therefore conflict, and two that
allocate from distinct providers do not. With [PROV-6] the same is true of two
statements that only dispose.

[PAR-2]'s permission for a fill through a `MutSpan` needs two amendments. The
**loan** condition is stated over **iteration-formed** loans: every exclusive loan
formed by a statement of `B` is rooted in a binding `B` introduces, and a loan formed
before `L` on a root every footprint of `B` reaches only through 1999-2002's refined
single-element ranges does not deny. And the **write footprint** of `set m[at] = v;`
contains its origin at range `[a*at+b, a*at+b+1)` rather than at whole place
([PROV-3] use 1), which is what [PAR-2]'s standing condition needs.

[PAR-1] 1990's admitted intervening-statement list **gains the `dispose`
statement**, whose footprint is one write of each named provider place. Without it
every window containing a disposal is denied, and since the `set` exchange [LIV-3]
is an ordinary `set_stmt` it is already admitted, so `dispose` is the only addition
the two new forms need.
*Judgment:* the existing [PAR-1] and [PAR-2] permission judgments, with one fewer
special case, one added loan clause, ranged origins, and one added intervening
form. *Publishes:* permission. *Amends:* [PAR-1] 1969 and 1990, [PAR-2] 1994-2028,
and [PAR-3] 2029-2056 through their "forms every footprint exactly as [PAR-1] forms
one" clauses. *Depends:* [PAR-2] 1999's single-binder affine element-write
refinement, which is the disjointness argument the range clause composes with.
*Law:* L2, L5, L10. *History:* 6.8, F2 NB16.

**[RUN-4] The startup protocol.** Program start has four points, and the covered
guarantee spans the last three:

```text
PreStart
    select a row of E from the target's profile table, largest supported W first
    read the granted extent of the entry context's stack and compare it with the
        selected row's stack(entry) figure
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

*Judgment:* a target obligation, not a source judgment. *Publishes:* the selected
row. *Amends:* [PROG-3] 1499-1509, whose start-time obligation gains the
materialization of `E` and the entry-stack comparison, and whose `ProgramFinished`
boundary is now named. *Law:* L1, L5. *History:* 6.8, F2 NB15.

**[RUN-5] Admission, and the theorem.** `Admitted(H, row)` holds when an
environment `H` has actually established a grant implementing every item of the
selected row before the barrier, the entry context's initial stack included, and,
for the duration of the run, does not revoke it and permits no unmodelled
competitor to consume from it. Then:

```text
source-resource-closed(P)  and  E-materializes(P, T, B)  and  Admitted(H, row)
*Judgment:* none by the compiler. *Publishes:* the deployment contract, which is
the selected row. *Amends:* nothing. *Law:* L1.

------------------------------------------------------------------------------
no covered-resource exhaustion in run(H, T(P))
```

#### 3.K.10 One name per concept

```text
| concept                    | chosen                | why                                                     |
|----------------------------|-----------------------|---------------------------------------------------------|
| a run of slots, frame-      | FixedVector<T, n>     | the settled name; its capacity is in its type because   |
|   resident                 |                       | layout needs it before the run exists                   |
| a run of slots, store-      | Vector<'s, T>         | one type at three regions; its capacity is a measure    |
|   resident                 |                       | because a growth policy must change it                  |
| the store's handle          | Heap<'s>, Arena<..>   | a value you must hold in order to act; the parameter    |
|                            |                       | is never elided, because it is the allocation fact      |
| the brand's spelling        | written iff it relates| 3.K.0's assumption: decidable from the declaration text |
|                            | two positions         | alone. The entry heap is never written                  |
| build an empty run          | seq_fixed, seq_frame, | the placement is in the name, because it decides which  |
|                            | seq_arena, seq_heap   | item of E the run becomes (L6)                          |
| reserve a bump store        | arena_frame,          | as above; nothing else reserves                         |
|                            | arena_extent          |                                                         |
| append one element          | seq_place             | one name, whatever the backing                          |
| remove one element          | seq_take              | from the end; every other order is 3.L                  |
| swap two positions          | seq_exchange          | the row that makes relocation writable                  |
| read a measure              | len, cap, room        | one quantity, one name, term and reader alike           |
| a read-only view            | Span<'r, T>           | the rename is the whole change to slice<'r, T>          |
| a writable view             | MutSpan<'r, T>        | element writes only; its length is fixed by its type    |
| destroy a linear value      | dispose p using (..); | one statement, closed under containment as linearity is |
| take a linear value apart   | let N(f: a) = move v; | the inverse of construct, so the closure covers         |
|                            |                       | disassembly too                                         |
| transform a place through   | set p = f(q: move p); | one admission on the one assignment form; the only form |
|   a call                   |                       | the partition test could not write in wf                |
| rebind a consumed binding   | set p = e;            | the premise is deadness; the language gains no second   |
|                            |                       | assignment form                                         |
| a refusal                   | Option<T>             | the kernel consumes no affine input, so it declares no  |
|                            |                       | failure nominal; a library one declares its own struct  |
| the property                | resource-closed       | the long spelling is the one in use                     |
| the failure variant field   | Err(error: e)         | [PRE-1] declares Err(error: E)                          |
```

`FixedRing`, `PoolVector`, `HeapVector`, `ArenaVector`, `AppendView`, `absorb`,
`update`, `Full<T>`, `TooSmall`, `OutOfMemory`, `PoolExhausted`, `NeedCapacity` and
`NoRecord` are **not** in the kernel vocabulary. The first four are library names
for kernel types or library nominals (3.L.1, `CONTAINERS.md` §3, `CONTAINERS.md` §3); the next three are
retired with their rules; the last six are library nominals a writer declares over
their own type, and `CONTAINERS.md` §3 declares one.

#### 3.K.11 Amendment register

**This register is a collation of the `Amends:` and `Depends:` lines of every rule
in 3.K, and it carries nothing else.** It was written last, from the rules. It
covers 3.K only: 3.L amends nothing, because it is ordinary wf.

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
   satisfying conditions 1 to 4 while leaving [ENT-5] 2887's element-position
   carve-out — a sentence inside a cited range, made false by the amending rule's
   own body — unmentioned; and
6. every `*Publishes:* X on Y` names the [ENT-3] destination clause that puts X on
   Y. A `Publishes:` line with no destination is the same defect as an `Amends:`
   line with no row. Three forms publish onto a destination in this design and each
   names its clause: [CALL-4]'s destructuring `let`, [LIV-3]'s exchange targets, and
   [PROV-6]'s destructuring consume, all three served by the one added S12 clause.

**Changed.** Line numbers are `spec/kernel-spec.md` at a40c7e70, re-derived in this
session; five of the fourth draft's were wrong and four of its ranges overshot into a
blank line or a section heading. Each row's `by` column names the rules whose
`Amends:` lines reach it, and those lines carry the detail; a row that also records a
surviving depended sentence marks it **bold** (condition 4).

```text
| rule            | line      | change                                                          | by                          |
|-----------------|-----------|-----------------------------------------------------------------|-----------------------------|
| [SCOPE-3]       | 27-31     | heap exhaustion leaves the deferred set; stack and covered-store | [RES-4], [RES-6], [RUN-2]   |
|                 |           | exhaustion leave it for marked programs                          |                             |
| [FORM-2]        | 52-76     | +4 renderings: result list, destructuring let and consume, set   | [CALL-4], [LIV-3], [PROV-6] |
|                 |           | target list, dispose                                             |                             |
| [GRAM-2]        | 165-200   | result list; resource_closed; region_params on nominals;         | [CALL-4], [RES-4], [BLK-4], |
|                 |           | requires/ensures (182-183) take a clause_expr                    | [MSR-5]                     |
| [GRAM-3]        | 204-207   | slice/buffer/box/arena productions retire; runs and views are    | [PROV-1]                    |
|                 |           | ordinary TYPEIDs with targs                                      |                             |
| [GRAM-4]        | 214-254   | destructuring let and consume; comma return; set target list;    | [CALL-4], [LIV-3], [MSR-5], |
|                 |           | affine_factor GAINS terms; stmt gains dispose                    | [PROV-6]                    |
| [GRAM-5]        | 265-266   | +clause_expr, +clause_operand; atom untouched                    | [MSR-5]                     |
| [GRAM-9]        | 324       | unchanged; named because [MSR-5] moves the amendment away        | [MSR-5]                     |
| [GRAM-11]       | 341-345   | a fourth callee class in all three sentences                     | [BLK-0]                     |
| [TYPE-2]        | 352       | +5 nominals, slice renamed Span, box/arena/buffer retire; the    | [PROV-1], [BLK-1], [BLK-2], |
|                 |           | flat-element restriction is not inherited                        | [VIEW-1]                    |
| [TYPE-5]        | 369       | a fourth callee class for the written-argument criterion         | [BLK-0]                     |
| [TYPE-6]        | 391-403   | the domain's spellings, nominals and region parameters; 396's    | [BLK-0]                     |
|                 |           | callee IDENT admission                                           |                             |
| [TYPE-7]        | 471       | the deref domain becomes the two borrow modes alone              | [PROV-1]                    |
| [SET-1]         | 476-500   | loan-strength target traversal; one deadness-at-commit premise;  | [PROV-3], [LIV-2], [LIV-3], |
|                 |           | one added admission, the in-place exchange                       | [VIEW-4]                    |
| [SET-2]         | 508-524   | region-bearing rejection replaced by [PROV-3] use 3 and [LIV-3]; | [PROV-3], [LIV-3], [VIEW-4] |
|                 |           | a compiler-derived exchange whose replacement is the read-out    |                             |
| [CONST-2]       | 547-551   | its naming of buffer, slice and slice_of follows the retirements | [VIEW-1]                    |
| [OWN-1]         | 558-567   | 558-559 UNCHANGED; linear refines affine; 564 gains the          | [PROV-6], [VIEW-1],         |
|                 |           | partial-move refusal and one reinitialization route              | [LIV-1], [LIV-2]            |
| [OWN-4]         | 577       | the lent-onward child's loan ends at its receiving statement     | [PROV-7]                    |
| [OWN-5]         | 589-607   | origins generalize to loan-bearing values and carry a range and  | [PROV-3], [VIEW-2]          |
|                 |           | formation datums; two ranged access clauses; the address-        |                             |
|                 |           | computation freeze; 596 and 603 restated. **601 survives and     |                             |
|                 |           | [VIEW-2] depends on it**                                         |                             |
| [OWN-6]         | 611       | a child reborrow may name a caller-supplied region under the     | [PROV-7]                    |
|                 |           | result-type condition, for every reborrow                        |                             |
| [OWN-7]         | 624-625   | overlap extends to ranges. **625's subscript conservatism        | [PROV-3]                    |
|                 |           | survives and [PROV-3] use 2 depends on it**                      |                             |
| [OWN-10]        | 636-640   | 638's arena content clause becomes one over Vector content.      | [PROV-1]                    |
|                 |           | **636 survives and [PROV-2] depends on it** (4a and 4b)          |                             |
| [OWN-11]        | 641       | the move prohibition is replaced by [LIV-1]'s join agreement     | [LIV-1]                     |
| [STOR-1]        | 670-677   | the runs join the storage table; 677's growable paragraph is     | [LIV-2]                     |
|                 |           | superseded by the library; 674 narrows to a live target          |                             |
| [STOR-2]        | 680       | box_new and arena_new retire; a store take is a kernel row       | [PROV-2]                    |
| [STOR-3]        | 683-715   | a linear type has no derived release; box and buffer rows        | [PROV-5], [PROV-6], [RES-9] |
|                 |           | retire; the store reset joins the table and is split from        |                             |
|                 |           | content release; 704-707 gains a second subject. **694-700's     |                             |
|                 |           | drop order survives and [PROV-6] reuses it**                     |                             |
| [STOR-4]        | 716       | confinement becomes the outlives relation over the region set    | [BLK-4]                     |
| [STOR-5]        | 718-732   | the position list becomes the three-way intensional split; the   | [BLK-4], [PROV-2]           |
|                 |           | per-leaf-provenance deferral is withdrawn as unnecessary         |                             |
| [STOR-6]        | 733-765   | E-materialization joins the target-stage obligations             | [RES-3], [STK-3]            |
| [OP-1]          | 766-845   | +cap and +room, pure, over runs, views and providers; five       | [PROV-2], [BLK-0], [BLK-2], |
|                 |           | constructors retire; ReservedLowerNames +2; 833 gains the class  | [VIEW-1]                    |
| [OP-4]          | 909-915   | indexable bases extend; the obligation is against len; a         | [BLK-1], [MSR-1]            |
|                 |           | subscripted measure place in an erased clause discharges at its  |                             |
|                 |           | own attach site                                                  |                             |
| [OP-5]          | 921       | "and contract predicate" narrows to a source condition           | [MSR-5]                     |
| [OP-7]          | 935       | slice_of retires; cap and room join the structural operations    | [VIEW-1]                    |
| [OP-9]          | 968-996   | the ceiling table gains Appendix A.1's rows, the region-bearing  | [RES-5]                     |
|                 |           | exclusion is lifted, and advance<T> is fixed here                 |                             |
| [FN-1]          | 999-1070  | the view ceiling and its duplicate-result rejection; an ordered  | [VIEW-6], [CALL-4],         |
|                 |           | result list; three boundary components; a loop_stmt's normal-    | [RES-8], [STK-4]            |
|                 |           | successor edge. **1035-1041 survives and [PROV-3] depends on it**|                             |
| [FN-2]          | 1087      | the rejection narrows to loan-bearing and provider arguments;    | [BLK-4], [BLK-0]            |
|                 |           | explicit instantiation covers nominals and the kernel domain     |                             |
| [FN-3]          | 1117-1121 | the allocation component becomes the set of allocates paths      | [PROV-4]                    |
| [FN-7]          | 1211-1246 | command.heap; resource_closed; exactly one region parameter;     | [PROV-1], [RES-4]           |
|                 |           | allocates over a labelled input; the byte sequence gains the row |                             |
| [FN-8]          | 1256-1261 | clause operands are a clause_expr; 1261 becomes a GoalTemplate-  | [MSR-5]                     |
|                 |           | formation sentence. **1269 survives and [MSR-3] depends on it**  |                             |
| [FN-9]          | 1295-1360 | terms as operands; measured results; multi-datum clauses; entry  | [MSR-3], [MSR-4], [MSR-5],  |
|                 |           | and exit datums replace 1310; 1339's M(c,q) admits a datum;      | [CALL-4]                    |
|                 |           | 1306's closed root set is what [MSR-5] reuses                    |                             |
| [EFF-1]         | 1363-1380 | allocates takes formal-rooted paths; heap and arena retire;      | [PROV-4], [PROV-3]          |
|                 |           | 1380 generalizes to a loan-bearing parameter, which [CALL-3]     |                             |
|                 |           | and [VIEW-7] depend on (4a)                                      |                             |
| [EFF-2]         | 1400-1421 | the slice projection generalizes; 1421 stays TRUE for its four   | [PROV-3], [PROV-6]          |
|                 |           | types and is joined by the disposal walk's contribution          |                             |
| [ERR-4]         | 1478      | the deferral gains the two families that move inside             | [RES-7]                     |
| [PROG-3]        | 1499-1509 | PreStart materializes E and compares the granted entry stack;    | [RUN-4]                     |
|                 |           | ProgramFinished is named                                         |                             |
| [DIAG-1]        | 1687-1712 | rank 5 covers the kernel domain; +container_declaration_ordinal  | [BLK-0]                     |
| [PAR-1]         | 1969,1989,| the provider-place projection; dispose enters a footprint and    | [RUN-3], [RUN-2], [PROV-6]  |
|                 | 1990      | joins 1990's intervening list; exhaustion unreachable when       |                             |
|                 |           | marked. **2102 survives and [RUN-1] depends on it**              |                             |
| [PAR-2]         | 1994-2028 | iteration-formed loans; a view's ranged write footprint; the     | [RUN-3], [RUN-2]            |
|                 |           | element-write form. **1999 survives and [RUN-3] depends on it**  |                             |
| [PAR-3]         | 2029-2056 | the exhaustion sentence; replicated places cannot occur marked   | [RUN-3], [RUN-2]            |
| [SYS-1]         | 2130      | a fourth admitted declaration source                             | [BLK-0]                     |
| [SYS-2]         | 2158-2301 | views at the range-bearing operations; an "acquires from" column | [VIEW-7], [RUN-1], [RES-6], |
|                 |           | reading none for all sixteen; reserve_file gains a recoverable   | [RES-7], [RES-9]            |
|                 |           | outcome; 2295's proposition set gains covered-store measures.    |                             |
|                 |           | **2264 is kept and [RES-7]'s column values depend on it**        |                             |
| [SYS-3]         | 2303      | the kernel domain is admitted to every unit                      | [BLK-0]                     |
| [SYS-5]         | 2560,2575 | release-completeness is KEPT; the release action gains the       | [RES-9]                     |
|                 |           | handle-table subject                                             |                             |
| [SYS-7]         | 2467-2481 | the class set is UNCHANGED, which is why no nominal is added     | [RES-6]                     |
| [SYS-8]         | 2482-2521 | the seven range-bearing operations take MutSpan and Span         | [VIEW-7]                    |
| [SYS-9,11,12,14]| 2522-2639 | their prose naming buffer<u8> is restated over views             | [VIEW-7]                    |
| [SYS-10]        | 2548-2552 | a reservation consumes a runtime record with a published         | [RES-9]                     |
|                 |           | capacity, and the record returns on release                      |                             |
| [QUAL-2]        | 2363      | +one failure: an implementation needing an undeclared store.     | [RES-7]                     |
|                 |           | **2363's own sentence survives and [RES-9] depends on it**       |                             |
| [ENT-2]         | 2675,2677,| measure terms over a subscriptable place; +the measure datum;    | [MSR-1], [MSR-3], [LIV-2],  |
|                 | 2722      | a reinitializing set is a declaration event; +standing facts.    | [MSR-2]                     |
|                 |           | **2687 survives and [MSR-3] depends on it**                      |                             |
| [ENT-3]         | 2724,2768,| +S13 and its arm route; S5 gains the construct placement; S6     | [BLK-0], [MSR-3], [CALL-4], |
|                 | 2778,2827 | generalizes over three measures; S12 gains one clause serving    | [LIV-3]                     |
|                 |           | three forms                                                      |                             |
| [ENT-5]         | 2857-2899 | descriptor-storage support; the effect-row kill; 2887(a) LOSES   | [MSR-2], [MSR-3], [CALL-5]  |
|                 |           | its element-position carve-out; the datum replaces the call-     |                             |
|                 |           | boundary and 2881-2885 paragraphs; clause (b) is classified by   |                             |
|                 |           | [CALL-1..3]. **2936-2940 survives and [MSR-2] and [MSR-3]        |                             |
|                 |           | depend on it**                                                   |                             |
| [ENT-6]         | 2970-3092 | one goal disposition; measures carry images; 3001 gains          | [MSR-3], [MSR-4], [MSR-2]   |
|                 |           | len + room = cap as two members                                  |                             |
| [INV-1]         | 3099-3107 | 3099's restriction is reused by [MSR-5] and gains [MSR-3]'s      | [MSR-3], [MSR-5]            |
|                 |           | atom-identity sentence; 3107 admits terms and named consts.      |                             |
|                 |           | **3099 survives and [MSR-5] depends on it**                      |                             |
| batch 0079      | docs/done/| the abort site loses its allocation caller and keeps two until   | [RES-6], [PROV-6]           |
| exhaustion floor| 0079-...  | [PROV-6]'s bounded walk replaces them                            |                             |
```

**Depended on and unchanged.** Each row is the collation of one or more `Depends:`
lines, and each names the rule that depends on it. A later batch changing one of
these sentences changes a rule of this design without touching it. Dependencies that
fall inside changed text, or that name a retired subject, are on the changed rows
above instead (condition 4).

```text
| rule       | line | the sentence, and who depends on it                                       |
|------------|------|---------------------------------------------------------------------------|
| OWN-3      | 573  | region identifiers are unique within a function: [PROV-1], which is why a  |
|            |      | store region's spelling denotes one store                                  |
| OWN-3      | 575  | distinct caller-supplied regions are incomparable and every ordering rule  |
|            |      | fails closed: [PROV-1] and [BLK-4], the whole invariance argument          |
| OWN-6      | 609  | a borrow not bound by let is a call-scoped temporary: [PROV-2] and         |
|            |      | [VIEW-2], which is why the argument borrow is not the freeze               |
| OWN-12     | 645  | region substitution controls type equality: [PROV-1], which is why two     |
|            |      | stores are distinguished by their types                                    |
| TYPE-5     | 374  | argument types match declared parameter types exactly: [PROV-1], the other |
|            |      | half of the invariance argument                                            |
| ERR-4      | 1481 | parallel permissions never reject the source: [PROV-5], which is why its   |
|            |      | multiplicity refusal reads the call graph and not a permission             |
| FN-6       | 1205 | recursion is permitted: [STK-2], which excludes a program from [RES-4]     |
|            |      | rather than rejecting it                                                   |
| PROG-1     | 1486 | one closed compilation unit with no function values: [PROV-4]'s exact      |
|            |      | reachability closure and [RES-8]'s composition claim                       |
| ENT-4      | 2854 | L0's uniqueness and finiteness rests on the difference-bound shape:        |
|            |      | [MSR-2], which is why len + room = cap is an affine premise and not an L0  |
|            |      | fact                                                                       |
```

**META-5 delta**, declared here because the register is its natural home. Numbered
language rules: 131 today, plus the 48 of 3.K, none reusing a live or retired id;
the region-spelling amendment (3.K.0) is counted with its own batch and not here.
Unique fixed lowercase grammar atoms: minus 5 for the retired `heap` and `arena`
effect atoms and the retired `slice`, `buffer` and `box` type productions (`arena`
is one atom serving both a production and an effect entry, and retires once), plus 3
for `resource_closed`, `dispose` and `using`; net minus 2. The fourth draft's
`update`, `by` and `into` are not added, because [LIV-3] is an admission on `set`.
Grammar productions: plus 2, being `clause_expr` and `clause_operand` and the
`dispose_stmt`, less the retired `slice_of`-bearing forms — net plus 3 counting the
statement; changed, 9, being `let_stmt`, `return_stmt`, `set_stmt`,
`result_binding`, `program_kind`, `struct_decl`, `enum_decl`, `effect`,
`affine_factor`, with `requires_clause`/`ensures_clause` counted once as a pair.
`ReservedLowerNames`: plus 2, `cap` and `room`. Nominal types: plus 5, being 2
providers, 2 runs and 1 view, and one renamed, `slice` to `Span`. Declaration
domains: plus 1, with one `container_declaration_ordinal`. Entry input rows: plus 1.
[SYS-2]'s normative inventory counts change with [VIEW-7], [RES-6], [RES-7] and
[RES-9] and are recomputed when those rules are written into the spec, not asserted
here.

**Retired outright, with no successor.** The fourth draft's five owner types
([BLK-1]); its `AppendView`, `absorb` and the abandoned-window disposition; its
`update` statement and its three atoms; its `Pool` store, `PoolSlot`, `PoolVector`,
`seq_lease`, `pool_frame`, `pool_extent`, `pool_take`, `pool_release` and the pool
seam; its `FixedRing` and four ring rows; its `HeapBox` and `ArenaBox`, which are
runs of capacity one; its three failure structs and its `NoRecord`; its `seq_filled`,
`seq_vacant`, `seq_take_at`, `seq_clear`, `seq_truncate`, `seq_reserve_heap`,
`seq_reserve_arena`, `seq_shrink`, `seq_heap_filled`, `seq_push`, `seq_try_push`,
`seq_pop` and every `try` row; the `&uniq buffer<T>` and `&uniq Container`
prohibition ([CNT-7], deleted as capability rather than as text); the effect-row
atoms `heap` and `arena`; `slice_of`, `box_new` and `arena_new`; the first draft's
`Builder<'r, T>` and `[BLD]`; the second draft's `[STK-4]` reentrancy premise;
`[CNT-5]`; and L14. Every id is retired and none is reused.

**Writer doctrine this design invalidates**, which `docs/patterns.md` must carry in
the same batch. **P16** ("One length fact above the writes") rests on the
element-write exception surviving a callee boundary through a `&uniq buffer` actual;
[CALL-5] makes that kill conservative, so P16's shape is invalid from B1 until B5
restores it over `MutSpan`, and P16 gains a second correction from [MSR-2] — a length
fact survives a write to a **sibling field**, which probe `r2_4` shows today's
compiler killing. **P17**'s field-by-field fold is **narrowed** to non-linear
aggregates, because [PROV-6] refuses a partial move of a linear one, and its
`replace` note gains [LIV-2]'s dead-target `set` and [LIV-3]'s in-place exchange.
**P19** is unchanged and gains a case: a measure term joins by the same delta-atom
rule. **P15** is unchanged and both worked programs follow it. **P8** should gain
what probes `q5'`, `m10` and `x1b` bought: an exact `-` or `+` carries an ordering
into a backedge where the wrapping form gives the checker a fresh atom. Four new
patterns are owed: structural disposal, the linear destructuring consume, the
`propagate`-free allocating helper, and 3.L.3's two-invariant construction loop.

---

### 3.L The library, written in wf

#### 3.L.0 How to read this section

Everything below is **ordinary wf**, written against 3.K and against the unchanged
v0.40 rules. It defines no rule, amends no rule, and is named by no rule. It exists
to discharge L18's obligation: an item the kernel no longer carries is written out
here, or the kernel lacked a primitive and 3.L.6 says which.

Each item states its **proof route** — which kernel rule discharges each obligation,
and which of those v0.40 already proves today, naming the probe where one exists. The
code is design text; the standard it is held to is that every statement is accepted by
a compiler implementing 3.K and the unchanged v0.40 rules.

One genericity limit is stated once here rather than repeated. A **writer's**
generic over an element type cannot serve a copy and an affine instantiation from
one body — probes `m12` and `m14` show one accepted at `box<u64>` and rejected at
`u64` — and a const generic parameter is not readable as a value (`m16`). The
kernel's own domain is generic because it is compiler-owned and monomorphized
[BLK-0]; a writer's library is generic only where its body never needs the
distinction, and is otherwise written per element type. That is an [OWN-1] question
(Q8) and a compiler-capability question, not a partition finding, and where it
bites below the code is written at a concrete element type and says so.

#### 3.L.1 The owner names

`FixedVector<T, n>` is the kernel type and needs no library. `HeapVector<T>`,
`ArenaVector<'a, T>` and `PoolVector` are what a writer *calls* a
`Vector<'s, T>` whose store is the heap, a named arena, and a library pool
respectively; they are one kernel type at three regions and the library adds
nothing to them (footnote 1). Under 3.K.0 a heap run in a stored position is written
`Vector<u8>` and an arena run `Vector<'a, u8>`, which is the whole visible
difference between them.

#### 3.L.2 The partition, item by item

Every item is written in wf in `CONTAINERS.md` §3 against 3.K and against the
unchanged v0.40 rules, with its proof obligations walked there. This table is the
result; two items are written out below because they are the two that earned kernel
additions, and one more is the elision demonstration.

```text
| item                          | written as                          | route, and what discharges it       |
|-------------------------------|-------------------------------------|-------------------------------------|
| FixedVector<T, n>             | the kernel type itself              | nothing to write                    |
| HeapVector, ArenaVector,      | Vector<'s, T> at three regions      | nothing to write                    |
|   PoolVector                  |                                     |                                     |
| vacant<T, n>                  | a counted loop of seq_place over    | two header invariants; the exit     |
|                               | None<T>(), 3.L.3 below              | ordering, not an equality; x1c, x1d |
| filled_bytes<n>               | the same, reusing one copy value    | as above; per element type (Q8)     |
| take_at                       | seq_exchange then seq_take          | the requires plus Z <= index        |
| clear, truncate               | a counted drain, two invariants     | as vacant; a linear T disposes each |
|                               |                                     | and the signature says so [PROV-6]  |
| FixedRing<T, n>               | struct Ring { slots: FixedVector<   | [OP-4] from a requires the caller   |
|                               | Option<T>, n>; head; fill; } with   | discharges from ring_new's ensures  |
|                               | element replace                     | through the construct placement     |
| growth policy, HeapVector     | seq_heap, drain-in-reverse,         | four invariants; seq_exchange is    |
|                               | seq_exchange to restore order,      | what makes order preservation       |
|                               | replace, dispose                    | writable at all                     |
| block pool                    | struct BlockPool { free:            | a branch on len and on room, which  |
|                               | FixedVector<Vector<'s,u8>, m>; }    | needs [ENT-3.S6] over three         |
|                               | with seq_take and seq_place         | measures                            |
| collect and the appenders     | a counted loop over &uniq, `CONTAINERS.md` §3    | the exchange, and the exit datum    |
|                               | below                               |                                     |
| keyed families                | vacant plus element replace         | [OP-4] from the requires; x7        |
| try_place, try_take, try_push | a branch on room or len and two     | [ENT-3.S6] again                    |
|                               | returns                             |                                     |
| update p by op(...)           | set p = op(receiver: move p, ...)   | [LIV-3]                             |
| update p by op(...) into x    | set (p, x) = op(receiver: move p,   | [LIV-3]'s multi-target form         |
|                               | ...)                                |                                     |
| OutOfMemory<T> and its family | an ordinary one-field struct over   | [BLK-4] admits it; the kernel needs |
|                               | the writer's own type               | none                                |
```

Every one of them is writable. Nothing in the fourth draft's container inventory
turned out to need a kernel primitive that the three per-slot rows, the two views and
the four formations do not already have — except the seven of 3.L.6, each of which a
named function below or in `CONTAINERS.md` demanded.

#### 3.L.3 Filled and vacant construction, written out

The two constructions the fourth draft made inventory rows. `vacant` is the more
interesting because round 3 concluded no loop could publish `len = n`; it is right
that no loop publishes the *equality*, and wrong that the equality is what a
subscript needs.

```wf-design
fn vacant<T, const n: u64>() -> result: own FixedVector<Option<T>, n> pure contract {
  ensures ige(len(result), n);
} {
  doc "Builds a run of n slots, every one holding None.";
  let built = seq_fixed<Option<T>, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: ige(len(built), at),
    invariant spare: ige(room(built) + at, n)
  ) {
    let empty = None<T>();
    set built = seq_place(vector: move built, value: move empty);
  }
  return move built;
}
```

**Proof route.** `seq_fixed` publishes `len(built) = Z` and `cap(built) = n`
[BLK-2], and [MSR-2]'s identity gives `room(built) = n`. `grown`'s base is
`Z >= Z`; `spare`'s base is `n + Z >= n`. `seq_place`'s own requirement
`igt(room(built), Z)` discharges from `spare` and the counted loop's `at < n`
([ENT-3.S11]) by [MSR-4] step 5's unordered-pair family. On the backedge
`seq_place` declares `len(result) = len(vector) + 1` and
`cap(result) = cap(vector)` over that call's own datum, which has empty support
[MSR-3], so `room` falls by one and `at` rises by one and both invariants are
preserved; the `set` is an in-place exchange, so it is **not** a declaration event
and the two atoms survive on the same term [MSR-3]. At the exit `at = n`, so
`ige(len(built), n)` holds and the `ensures` discharges.

`vacant` is generic over `T` with no copy bound, because `None<T>()` is built fresh
each iteration. `filled` is not, because it reuses one `value`, so it is written per
copy element type:

```wf-design
fn filled_bytes<const n: u64>(value: own u8) -> result: own FixedVector<u8, n> pure contract {
  ensures ige(len(result), n);
} {
  doc "Builds a run of n byte slots, every one holding value.";
  let built = seq_fixed<u8, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: ige(len(built), at),
    invariant spare: ige(room(built) + at, n)
  ) {
    set built = seq_place(vector: move built, value: value);
  }
  return move built;
}
```

Same route. This is the function [VIEW-7] needs for an addressable I/O destination,
and it is the one `wfgrep.wf`'s migration calls twice.

#### 3.L.4 `collect`, written out

The one program every draft has carried, and the item that demanded two of the seven.

```wf-design
fn collect(out: &uniq Vector<u8>, source: own Span<u8>)
    -> written: own u64
    reads(source), writes(out) contract {
  requires ile(len(source), room(out));
  ensures ieq(len(out), written);
} {
  doc "Appends every byte of source into the destination's spare room.";
  let count = len(source);
  let before = len(deref(out));
  for @copy (
    at in 0_u64..count,
    invariant spare: ige(room(deref(out)) + at, count),
    invariant grown: ige(len(deref(out)), before + at)
  ) {
    let byte = source[at];
    set deref(out) = seq_place(vector: move deref(out), value: byte);
  }
  return before +wrap count;
}
```

`collect` writes no region at all: `out`'s brand relates nothing, so it is an
implicit region parameter and the function is generic over the store it is handed,
and `source`'s loan region relates nothing either. Under the fourth draft the same
function carried three.

`collect` writes no region at all: `out`'s brand relates nothing, so it is an
implicit region parameter and the function is generic over the store it is handed,
and `source`'s loan region relates nothing either. Under the fourth draft the same
function carried three.

**Proof route.** `spare`'s base is the `requires`, whose `room(out)` denotes the
parameter's **entry datum** [CALL-4]. `seq_place`'s `igt(room, Z)` discharges from
`spare` and `at < count` by [MSR-4] step 5; probes `k21` and `k21b` are that
arithmetic at v0.40 scale, accepted and then rejected when the invariant is deleted.
The backedge is [MSR-3]'s three steps: the exchange is not a declaration event, the
facts over `room(deref(out))` die by [MSR-2], and `seq_place`'s declared relation
over its call datum — which has empty support and therefore survives the exchange's
own kill — re-establishes them on the same term. The `ensures` names `len(out)` in
an `ensures`, so it denotes the parameter's **exit datum** [CALL-4].

**Two of the seven are here.** Without [LIV-3]'s exchange there is no way to write
the loop body at all: `move deref(out)` is a move through a borrow, which [OWN-5]
614 forbids with [SET-2] as the sole exception, and the `replace` route needs a
placeholder `Vector<'s, u8>`, which is a run that owns storage and is itself linear,
so every iteration would allocate and dispose. Probe `x3` is that rejection today.
Without [CALL-4]'s exit datum the `ensures` says nothing a caller can use, every
capacity proof collapses into the function that owns the run, and every helper
boundary costs a re-read and a real branch — which is the branch L4 promises
`seq_place` does not have.

That pair is also the whole of what `AppendView` was for. The fourth draft's
`collect` took `own AppendView<'o, u8>` and returned it, and `absorb` published the
owner's new length from a datum the view had carried since formation. This one
publishes the same fact about the same run, over a borrow, with no third view type,
no commit event, no carried datum, and no rule that has to transport one across a
call — which is round 4's F3 defect 2 removed rather than repaired.

#### 3.L.5 The store region, before and after

**The store region is elided.** `byte_string.wf` has exactly one store, so under
[PROV-1] nothing in it names a region. The difference is worth writing out, because
it is F4's largest single finding and it is invisible in prose. Its join — the
program spells it `bs_concat` — reads, under the fourth draft's brand:

```wf-design
struct Bytes['h] {
  v: HeapVector<'h, u8>;
}

fn bs_concat['h, 'd, 's, 'b](destination: &uniq 'd Bytes<'h>, source: &'s Bytes<'h>,
                             heap: &uniq 'b Heap<'h>) -> done: own Bool
    reads(destination.v, source.v, heap), writes(destination.v, heap), allocates(heap) { ... }

    // at each of its four call sites:
    let joined = bs_concat<'h, 'd, 's, 'b>(destination: ..., source: ..., heap: ...);
```

and under this draft, where the region-spelling amendment (3.K.0) takes the loan
regions as well as the brand, because none of them relates two positions:

```wf-design
struct Bytes {
  v: Vector<u8>;
}

fn bs_concat(destination: &uniq Bytes, source: &Bytes, heap: &uniq Heap) -> done: own Bool
    reads(destination.v, source.v, heap), writes(destination.v, heap), allocates(heap) { ... }

    let joined = bs_concat(destination: ..., source: ..., heap: ...);
```

The whole region parameter list leaves the struct and the signature, four brand
occurrences leave the written types, three borrow annotations lose their names, and
the call site loses its `targs` and its three borrow names. Across the eleven
functions of `byte_string.wf` that is ten `['h]`, fifteen brand occurrences and
twelve call-site brand arguments from the brand alone, and every region parameter
list, borrow name and call-site region argument from 3.K.0. The five provider
parameters and the seven disposals stay, and they are what buys something. Nothing
about the brand's soundness changes: `Vector<u8>` still names a store in its type,
and a nominal meant to hold an arena's run still writes its region at both
positions.

#### 3.L.6 What the partition test found the kernel lacked

Seven, each named with the library function that demanded it and the probe that
shows it is new capability rather than a compiler defect. Numbering: this is the
list 3.L.2's last paragraph points at.

```text
| # | kernel addition                      | demanded by                       | today                 |
|---|--------------------------------------|-----------------------------------|-----------------------|
| 1 | the in-place exchange admission of   | collect, bs_reserve, pool_take,   | x2, x3 REJECTED       |
|   | `set` [LIV-3]                        | pool_release, vacant, filled,     | [STOR-1]              |
|   |                                      | clear, try_place — every library  | AffineSetTarget       |
|   |                                      | function that transforms a place  |                       |
|   |                                      | it does not own outright          |                       |
| 2 | its multi-target form,               | pool_take, bs_reserve's drain,    | new grammar           |
|   | `set (p, x) = f(...)` [LIV-3]        | clear — every two-result row at a |                       |
|   |                                      | field or deref place              |                       |
| 3 | the exit datum over a `&uniq`        | collect, bs_reserve — every       | x10 ACCEPTED and      |
|   | parameter [CALL-4]                   | helper that changes a borrowed    | read as the ENTRY     |
|   |                                      | run and must tell its caller      | image                 |
| 4 | [ENT-3.S6] over the three measures   | every try_ form, pool_take,       | S6 2779 covers len    |
|   | [BLK-0]                              | pool_release — every branch on a  | alone                 |
|   |                                      | capacity                          |                       |
| 5 | `seq_exchange` [BLK-3]               | take_at, bs_reserve's             | no analogue           |
|   |                                      | order-preserving relocation       |                       |
| 6 | the `&uniq` run parameter, i.e.      | collect, bs_reserve, pool_take,   | x11 ACCEPTED, and     |
|   | [CNT-7]'s deletion                   | pool_release, and every helper    | unsound without       |
|   |                                      | that is not a constructor         | [CALL-5]              |
| 7 | the construct placement of the       | Ring, BlockPool, Bytes — every    | construct kills the   |
|   | measure datum [MSR-3]                | library nominal wrapping a run    | operand's measures    |
```

And the list that matters as much: **what the partition did *not* need.** A ring
needed no kernel rotation, a pool needed no kernel store, a keyed table needed no
kernel occupancy, a growth policy needed no kernel growth row, middle removal needed
no kernel row, filled and vacant construction needed no kernel row, and the `try`
family needed nothing at all. Five owner types became two, thirty-odd operations
became fourteen, three views became two, sixteen added nominals became five, and one
statement form became an admission on an existing one — and every capability the
fourth draft claimed is still written somewhere in this section.

Two items were **not** resolved by writing them, and they are the honest residue of
the test. An arena-backed run is not linear, so a leased run that a writer drops
leaks a free-list slot with no diagnostic (`CONTAINERS.md` §3). And a writer's generic cannot
serve a copy and an affine element type from one body, so `filled` is written per
element type (3.L.0). Neither is a missing primitive; the first is a consequence of
tying linearity to the store's reclamation discipline and the second is Q8.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. The
standard they are held to is that every statement is accepted by a compiler
implementing 3.K's rules, **the library functions of 3.L**, and the unchanged v0.40
rules; both were walked statement by statement against all three before this draft
was finished. Round 4 walked the fourth draft's pair and found three classes of
refusal — an `update` whose relations reach no destination, a `move` and a `dispose`
of a value no rule classified as affine, and an `absorb` naming a datum no rule
transports across a call. All three are gone, and each is gone because a rule
changed rather than because the program was rewritten around it.

Byte figures are symbolic. No implementation computed any of them, and where a
figure depends on code generation the table says so instead of inventing a number.

### 4.1 A cooperative run queue with the heap absent

A fixed run queue of tasks, a 256-byte transmit ring, and an eight-block pool with
typed exhaustion. Each task is a state machine that advances one step per turn and
re-queues itself while it wants another. No heap, no recursion, an acyclic call
graph, and a queue loop whose resource state is restored on every backedge. It is
**not** a context-switching scheduler, and 1.5 says why. It uses `try_place`,
`try_take` (3.L.2), `ring_new`, `ring_place` (`CONTAINERS.md` §3), `pool_new`, `pool_take` and
`pool_release` (`CONTAINERS.md` §3) from the library, and nothing else the kernel does not
declare.

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

fn render(block: &uniq Vector<u8>, task: &Task) -> written: own u64
    reads(task.state), writes(block) contract {
  requires ige(room(block), 8_u64);
  ensures ile(written, len(block));
} {
  doc "Writes one eight-byte record for a task into the block and reports the count.";
  let narrowed = cvt<u32, u8>(deref(task).state);
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
    invariant spare: ige(room(deref(block)) + at, 8_u64),
    invariant grown: ige(len(deref(block)), at)
  ) {
    set deref(block) = seq_place(vector: move deref(block), value: mark);
  }
  return 8_u64;
}

fn drain(ring: &uniq Ring<u8, 256>, block: &Vector<u8>, count: own u64)
    -> sent: own u64
    reads(block, ring.head, ring.fill), writes(ring.slots, ring.fill) contract {
  requires ile(count, len(block));
  requires ige(len(ring.slots), 256_u64);
} {
  doc "Copies one prefix of the block into the transmit ring and reports how many bytes it placed.";
  let placed = 0_u64;
  for @copy (
    at in 0_u64..count,
    invariant slots: ige(len(deref(ring).slots), 256_u64)
  ) {
    let byte = deref(block)[at];
    set (deref(ring), unplaced) = ring_place<u8, 256>(ring: move deref(ring), value: byte);
    match unplaced {
      None() => {
        set placed = placed +wrap 1_u64;
      }
      Some(value: dropped) => {
      }
    }
  }
  return placed;
}

resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Runs a cooperative queue of state machines over a pooled block store and a transmit ring.";
  let ring = ring_new<u8, 256>();
  let pending = seq_fixed<Task, 32>();
  let first = Task(state: 0_u32, arg: 65_u64);
  set (pending, unplaced) = try_place<Task, 32>(vector: move pending, value: move first);
  match unplaced {
    None() => {
    }
    Some(value: rejected) => {
      return exit_status(code: 1_u8);
    }
  }
  region 'a {
    let scratch = arena_frame<65536, 16, 'a>();
    let code = 0_u8;
    region {
      let made = pool_new<'a>(arena: &uniq scratch);
      match made {
        None() => {
          set code = 1_u8;
        }
        Some(value: pool) => {
          let blocks = move pool;
          loop @queue {
            set (pending, next) = try_take<Task, 32>(vector: move pending);
            match next {
              None() => {
                break @queue;
              }
              Some(value: task) => {
                region {
                  let leased = pool_take<'a>(pool: &uniq blocks);
                  match leased {
                    None() => {
                    }
                    Some(value: block) => {
                      let held = move block;
                      region {
                        let written = render(block: &uniq held, task: &task);
                        region {
                          let sent = drain(ring: &uniq ring, block: &held, count: written);
                        }
                      }
                      region {
                        let back = pool_release<'a>(pool: &uniq blocks, run: move held);
                      }
                    }
                  }
                }
                let stepped = advance(task: move task);
                match stepped {
                  None() => {
                  }
                  Some(value: again) => {
                    set (pending, refused) = try_place<Task, 32>(vector: move pending, value: move again);
                    match refused {
                      None() => {
                      }
                      Some(value: lost) => {
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
}
```

#### The envelope the compiler publishes

```text
E(queue.wf, <embedded target>, <this build>) row W = 1

  region  static.image        bytes  <target>        align  <target>  contiguous
  stack   entry               bytes  <post-codegen>  align  <ABI>     contiguous
  lanes                       count  1
  slots   task.records        count  0
  slots   completion.records  count  0
  slots   handle.table        count  0
```

`static.image` is the const items and the static parts of the emitted module
[STOR-6]. `stack.entry` is `main`'s frame — the `Ring` (256 `Option<u8>` slots plus
two words), the `FixedVector<Task, 32>` (32 strides plus one word), the `BlockPool`'s
`FixedVector<Vector<'a,u8>, 8>` (8 descriptors plus one word) and the one
`arena_frame` occurrence's 65536-byte extent — plus `render`, `drain`, `advance` and
the library, plus the runtime frames beneath `main` and its bounded teardown, plus
the cleanup-scratch domain, whose depth is the height of `Task`, `Ring` and
`BlockPool` and is therefore a constant; measured post-codegen over the whole chain
[STK-3], [PROV-5], [RES-5]. `lanes` is 1 because no permission is taken [RUN-1] and
[RUN-2] publishes the `W = 1` row, and every `slots` row is zero because there is no
`par` permission, no may-suspend operation and no system handle.

#### Why it is source-resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | main declares pure, selects no command.heap, and arena_frame is pure         |
|                       | [BLK-2], so [PROV-4]'s closure is empty and [RES-4] does not fire            |
| acyclic call graph    | main -> {render, drain, advance, the library, the kernel domain}. No cycle,  |
|                       | so [STK-1] rewrites nothing and [STK-2] passes; and because there is no      |
|                       | cycle, [PROV-5]'s activation refusal does not fire at arena_frame either     |
| arena demand bounded  | pool_new takes eight 256-byte runs once, before the queue loop; the loop's   |
|                       | backedge delta on the bump domain is 0, so 3.K.7.1's loop rule needs no      |
|                       | iteration bound. The lease and its release are on the same path, so the      |
|                       | free list's own delta is 0 too                                              |
| queue and ring        | FixedVector<Task, 32> and Ring<u8, 256> are frame placement, whose [RES-5]   |
|                       | row is decided at compile time and contributes no demand at all              |
| cleanup scratch       | every type reachable from main has an acyclic containment graph, so every    |
|                       | release walk's depth is a constant [PROV-6]                                  |
| L9's displacement     | ring_place refuses at capacity and returns the byte, and drain reports what  |
|                       | it placed, so nothing is displaced silently                                  |
| stack bounded         | one context, one chain, measured after code generation [STK-3]              |
| runtime closed        | W = 1, no task or completion records; every runtime store's peak is zero     |
```

#### The writer's-eye walkthrough

`let written = render(block: &uniq held, task: &task);` is the statement the fourth
draft could not write. `render` takes the run through a
**`&uniq` container parameter**, which [CNT-7] refused outright; its deletion is safe
because [CALL-5] kills the caller's measures at the call, and the caller does not
need them — `render`'s own `ensures ile(written, len(block))` publishes what it
needs, over the parameter's **exit datum** [CALL-4]. The fourth draft wrote this line as
`render<'a, 'w, 'w>(block: &uniq 'w held, task: &'w task)`, three region arguments
and two borrow names for a call that relates nothing.

Inside `render`, whose two borrows and one brand all relate nothing and are all
elided (3.K.0):

```wf-design
  for @fill (
    at in 0_u64..8_u64,
    invariant spare: ige(room(deref(block)) + at, 8_u64),
    invariant grown: ige(len(deref(block)), at)
  ) {
    set deref(block) = seq_place(vector: move deref(block), value: mark);
  }
```

The **backedge** is the derivation the whole container surface rests on, and this
draft has one answer for it rather than two. The `set` is an **in-place exchange**
[LIV-3], not a reinitializing `set`, so by [MSR-3]'s atom-identity sentence the
root's [ENT-2] term survives, the facts over `room(deref(block))` and
`len(deref(block))` die by [MSR-2], and `seq_place`'s declared
`len(result) = len(vector) + 1` and `cap(result) = cap(vector)` re-establish them on
the same term through [LIV-3]'s added [ENT-3.S12] destination clause. The datum
those relations name has empty support and therefore survives the exchange's own
kill. `at` grows by exactly one on the same edge, so `room + at` is preserved and
`len >= at` is preserved. Three steps, once per iteration, per invariant — and the
fourth draft's own walkthrough cited [LIV-2] here for a statement its [LIV-3] said
was not a `set`, which is the contradiction this form removes by being a `set`.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one library function with a typed
failure, disposal is one statement, the append helper takes the destination by
`&uniq` and publishes its exit measure, and **not one region names the store**,
because the program has exactly one.

```wf-design
const ceiling: u64 = 4096_u64;

struct Bytes {
  v: Vector<u8>;
}

fn bs_new(heap: &uniq Heap) -> made: own Option<Bytes>
    reads(heap), writes(heap), allocates(heap) {
  doc "Builds one empty byte string over a zero-length backing run.";
  let taken = seq_heap<u8>(heap: &uniq deref(heap), count: 0_u64);
  match taken {
    Some(value: run) => {
      let holder = Bytes(v: move run);
      return Some<Bytes>(value: move holder);
    }
    None() => {
      return None<Bytes>();
    }
  }
}

command fn main(command.stdout as sink: own Output, command.heap as heap: own Heap)
    -> status: own ExitStatus
    reads(sink, heap), writes(sink, heap), allocates(heap) {
  doc "Collects one fixed input run into a heap-backed run and writes it out, reporting a refusal instead of dying.";
  let input = filled_bytes<4096>(value: 65_u8);
  let code = 0_u8;
  region {
    let made = bs_new(heap: &uniq heap);
    match made {
      Some(value: holder) => {
        let s = move holder;
        let total = 0_u64;
        region {
          let grew = bs_reserve(s: &uniq s, heap: &uniq heap, additional: ceiling);
          match grew {
            True() => {
              region {
                let line = seq_span(vector: &input);
                set total = collect(out: &uniq s.v, source: move line);
              }
              region {
                let body = seq_span(vector: &s.v);
                region {
                  let outcome = write_once(output: &uniq sink, source: &body, start: 0_u64, end: total);
                  match outcome {
                    Ok(value: next) => {
                    }
                    Err(error: problem) => {
                      set code = 74_u8;
                    }
                  }
                }
              }
            }
            False() => {
              set code = 70_u8;
            }
          }
        }
        dispose s using (heap);
      }
      None() => {
        set code = 70_u8;
      }
    }
  }
  return exit_status(code: code);
}
```

#### The writer's-eye walkthrough

`write_once<'c, 'c, 'w>(...)` is [VIEW-7] over a view. Its obligations are
`ile(0_u64, total)`, implicit, and `ile(total, len(deref(body)))`, which discharges
from [VIEW-2]'s `len(body) = <formation datum of len(s.v)>` and `collect`'s exit
datum `len(s.v) = total`. This is the statement that makes goal A's container half
real. Its three regions all relate nothing, so all three are elided (3.K.0); the inner block
still exists because [OWN-10] needs `body` bound before the borrow, and it has no
name. Under the fourth draft this one statement carried three region arguments and
two borrow names, and Q11 asked for the formation half back.

`dispose s using (heap);` is [PROV-6], once, on the arm that has a value to destroy.
`Bytes` is a nominal with a linear field, so it is linear, so the enclosing region
block cannot be left with one alive; the walk drops each `u8` element, which derives nothing, and
then releases the backing to the store `Vector<u8>`'s type names. `heap` is the
entry's own `own Heap` binding and needs no region, because `using` names a place.
The walk's depth is `Bytes`'s containment height, a constant [PROV-6], so it needs no
auxiliary storage and no `wf_resource_abort` is reachable from it — which is round
4's rank-one resource finding, and probe `a8` is the shape that has one today.
**There is no path on which the process disappears**, which is the whole of goal B.

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

Four of the diagnostics the design owes a writer, each citing a rule that exists in
3.K; `DisposeProviderMissing` and `LinearValueAcrossPropagate` are stated inside
[PROV-6].
Three more — [PROV-1]'s `SecondStoreInOneRegion`, [PROV-1]'s `AmbiguousStoreRegion`
and [BLK-4]'s `ConfinedFieldWithoutRegion` — are stated inside their rules and are
not repeated here.

```text
Semantics/Source [BLK-0]: UndischargedOperationDomain
  operation: seq_place
  residual:  "Z < room(deref(block))"
  mechanical_fix: state a header invariant over room(deref(block)) [INV-1, MSR-5],
    dominate the place with a branch on room(deref(block)), take a larger run
    before the loop, or use the library's try_place

Semantics/Source [PROV-6]: LinearValueNotDisposed
  binding "s" of type Bytes is live on the edge leaving 'g
  its linear leaf Bytes.v of type Vector<u8> names the store region of command.heap
  mechanical_fix: move the value out of this scope, or write dispose s using (heap);
    a store-backed run has no compiler-derived release, so nothing else can free it

Semantics/Source [PROV-6]: LinearValuePartiallyMoved
  "move chunk.page" takes one leaf out of "chunk" of type Chunk, which is linear
  the residual leaf Chunk.spare of type Vector<u8> would then leave this scope by
    neither a move, a destructuring consume, nor a dispose
  mechanical_fix: write let Chunk(page: p, spare: q) = move chunk; and handle both
    leaves, or dispose the whole value

Semantics/Source [PROV-5]: ExtentReservedOnACallCycle
  arena_extent<65536, 16, 'p> is reached from a strongly connected component of the
    call graph: descend -> descend
  one committed extent would be held by every live activation at once
  mechanical_fix: reserve the store in the caller and lend the provider down
    [PROV-7], or use arena_frame, whose extent is per activation
```

The last is the one round 4 asked for by name, and probe `x8` is the program that
gets it.

---

## 5. Open questions

Everything the owner's rulings settle is dropped and not restated. So is everything
the earlier drafts asked and this one answers: the length-class terms and the goal
disposition are [MSR-1] and [MSR-4]; the arithmetic residual is [MSR-3]'s datums and
images; the coverage certificate died with `Builder`; the arena's reclamation is
[RES-5]'s cursor domain; the optimizer-versus-envelope question is [STK-3] and
[RES-2]; the profile table is [RES-2]. Seven questions earlier drafts filed are
**answered here rather than asked**, on the merits: a store is identified by its
region *per live activation of its region block* [PROV-1], [PROV-5]; a store-owned
value is destroyed by one structural statement whose walk is bounded by the type's
containment height [PROV-6]; a linear value is taken apart by
`let N(f: a, ...) = move v;` and a partial move of one is refused; disposal needs no
effect category of its own; region-parametric nominals belong in this version
[BLK-4]; the value-in / value-out spelling gets an admission on `set` rather than a
statement of its own [LIV-3]; and control entering the call graph from outside it is
1.5's, with the row the brand itself now owes.

### 5.0 The decisions the ruling forced, and the ruling's own question

These are the decisions the minimality ruling forced and the owner has not
separately ruled on. Each states what was traded and what the alternative costs.

**Q0a. `AppendView` and `absorb` are gone** (footnote 3). What is lost is a
*guarantee*: an `AppendView` could not reach below its owner's length, so a callee
handed one could not shrink what it was given. A callee handed `&uniq 'a Vector<'s, T>`
can. What is gained is that [CALL-4]'s exit datum publishes a borrowed value's
post-state for **every** type rather than for one, which is what makes `collect`
writable in wf, so the kernel keeps one mechanism instead of two.
*Recommend the trade*: the lost guarantee is a contract property, not a
memory-safety one, and keeping `AppendView` means keeping a third view, a commit
event, and a carried datum needing its own transport across a call.

**Q0c. A pool is library, and a leased run is not linear** (`CONTAINERS.md` §3). Under the fourth
draft a lease was linear and `dispose` was owed, so a dropped lease was a compile
error. Here it leaks a free-list slot until the region ends, with no diagnostic.
*This is the one place the trade is genuinely uncomfortable.* The leak is bounded by
the pool's own capacity and visible in `E` as a retained item, and a writer who wants
the check backs the free list from the heap instead of the arena; the alternative is
a way to declare an ordinary nominal linear, which is a feature of its own size.
*Recommend leaving it, and recording the loss.*

**Q0b, Q0d and Q0e together, and *recommend all three*.** Five owners became two and
`FixedRing` became a library nominal (footnotes 1, 2): a ring costs one `Option` word
per slot and one `match` per removal, and buys a readable head, correct disposal over
a linear element with no rule of its own, and one fewer nominal with four fewer rows.
`update` became an admission on `set` (footnote 4), losing nothing and costing three
fewer grammar atoms. And the kernel declares no failure nominal ([BLK-2], [RES-6]),
losing one shared vocabulary and gaining not having three compiler-owned nominals
whose only job is to be a struct a writer could have written.

**Q0g is decided and is recorded rather than asked.** The region-spelling amendment
lands first, separately and mechanically, and is not this design's (3.K.0). What
this design owes it is one property — the spelling is decidable from the declaration
text alone — and what it gets back is measured in 3.K.0 and answers Q11.

**Q0f, the ruling's own question: should any of 3.L ship?** The owner leans toward no
standard library at all, and 3.L proves the partition whether or not a line of it is
committed. Three items are load-bearing for this design's evidence — `filled` for
[VIEW-7]'s addressable destinations, `collect` for the append story, and the pool for
4.1. *Recommend: no `std`; those three land as test programs under
`tests/programs/`, where a rot check already reaches them.*

### 5.1 The questions this design genuinely does not decide

**Q1. May a marked program handle a typed refusal, or must it prove every
acquisition?** **Permissive**: both spellings are admitted, since neither can ask for
more than `E`, and L8 plus [RES-6] make it real — a refusal edge carries the store's
own `room(store) = Z`, and 3.K.7.1's loop rule names the checked spelling as one of
the three things that bounds a retaining loop.

**Q2. Where does a hosted marked program's large memory come from?** **Frame and
extent placement only**, as [PROV-5] and [BLK-2] provide; an entry row delivering a
committed region becomes right the day a program needs a store whose *size* is a
deployment decision rather than a source constant.

**Q3. Does the range relation need `seq_split_at`?** Not in this version. The
relation it needs already exists in [PROV-3]; what is missing is only the row.

**Q4. How does a marked program reach a device?** `main`'s effect row names only its
own labelled inputs and the `command` table is closed, so 4.1 has a transmit ring and
no way to flush it. **A second program kind** under [FN-7]'s existing closed-table
discipline, arriving with the execution-context design of 1.5: an interrupt vector
and an MMIO window are one batch, and a sixth hosted row would put a device on every
hosted program's entry.

**Q5. When does `par` become usable inside a marked program?** [RUN-1] denies
permission and [RUN-2] publishes `lanes(1)`, because the current runtime's wait path
runs a stolen task on the waiting lane's own stack. The answer is the
compiler-managed work-first continuation representation, then lifting the denial and
defining a worker lane's chain [STK-3]. This is the largest engineering item the
design implies, and [RUN-1] is what makes it a scheduling item rather than a
soundness risk.

**Q6. Does this version want a keyed or sparse container family?** Not yet.
`CONTAINERS.md` §3.5 writes stable-identity storage as a vacant run plus
element-position `replace`, which is sound, L12-clean, and compiles in shape today
(probe `x7`). A `FixedTable<T, n>` whose typestate is an occupancy set is the next
candidate, and under L18 it has to justify itself against §3.5, which works.

**Q7. Should a system operation be able to append?** **Yes, in the batch that lands
[CALL-4]'s exit datum in the [SYS-2] declaration domain, and not here.** Then the
bytes the host wrote become the run's own `len` and the caller reads it from the
operation's `ensures`, instead of [VIEW-7]'s addressable destination and a `u64`
beside the run.

**Q8. Is `copy` structural over aggregates?** [OWN-1] makes every owned composite
affine regardless of its field types, which is why 3.L.3's `filled` is written per
element type and why probes `m12` and `m14` disagree. **A `struct` or `enum` all of
whose field types are copy should be copy** — and it is not this design's to land. It
is the reason a writer's generic library is thinner than the kernel's own domain.

**Q9. Is `E` part of program identity?** **An emitted machine-readable table beside
the object, and explicitly not part of [PROG-2] compilation-unit identity**, which
[RES-2]'s three-argument form already says it is not.

**Q10. Should a `propagate` carry a disposal?** [PROV-6] refuses a `propagate` while
a linear binding is live, and probes `w5` and `m03` show the language admits that
shape today. **Leave the refusal now**; a release list on the statement, checked by
exactly [LIV-1], should be paid for by a program whose rewrite was actually painful.

**Q11 is answered and is retained only as a record.** It asked whether a
view-forming borrow needs its own written region. It does not: the region relates
nothing, so the region-spelling amendment elides it and the enclosing block keeps its
braces and loses its name. [VIEW-2] had already made the argument — if the argument
borrow is not the freeze, it does not need a writer-named region — and the amendment
is that argument generalized to every position.

## 6. Verified versus reasoned

**Verified** means a compiler executed it, against the gate-profile `whitefootc`
built from this tree, in this session or in one of the sixteen falsifier sessions
whose probe names are quoted. No timing figure appears anywhere in this file, and the
known wrong acceptance of a `replace` at an arena descriptor is a compiler defect
counted as a design finding nowhere.

### 6.1 What the current compiler does

Fourteen runs over eleven probe programs were made in the session that wrote this
draft. The table describes each closely enough to rewrite it; the sources are session
scratch files and are not in the repository.

```text
| probe            | program                                                        | verdict                                   |
|------------------|----------------------------------------------------------------|-------------------------------------------|
| x1_fillorder     | counted loop, one `ige(fill, at)` header, exact `+` in the body | REJECTED [OP-2], residual                 |
|                  |                                                                | "fill +defined 1_u64"                     |
| x1b_fillorder    | the same with `+wrap`                                          | REJECTED [INV-1] Backedge, required       |
|                  |                                                                | relation "ile((at + 1_u64), fill)"        |
| x1c_fillorder    | the same with BOTH `ige(fill, at)` and `ile(fill, at)`, then    | **ACCEPTED**, exit 0                      |
|                  | `invariant done: ige(fill, 8_u64);` after the loop              |                                           |
| x1d_fillsub      | the same, and the loop-exit ordering discharges `data[3]`       | **ACCEPTED**, exit 0                      |
|                  | under a dominating `ilt(3_u64, fill)` branch                    |                                           |
| x2_exchange_set  | `set c = bump(cell: move c);` at a struct-typed local           | REJECTED [STOR-1] AffineSetTarget,        |
|                  |                                                                | fix: "combine it with the old value       |
|                  |                                                                | field by field"                           |
| x3_movederef     | `set deref(h) = bump(cell: move deref(h));` through `&uniq`     | REJECTED [STOR-1] AffineSetTarget         |
| x4_partial       | `struct Two { a: box<u64>; b: box<u64>; }`, `move pair.a`, the  | **ACCEPTED**, exit 0                      |
|                  | residual never touched                                         |                                           |
| x5_uniqensures   | `ensures ile(written, len(deref(destination)));` written direct | REJECTED [GRAM-9] at parse                |
| x6_rectype       | `struct Node { next: Option<box<Node>>; value: u64; }`          | **ACCEPTED**, exit 0                      |
| x7_optreplace    | `buffer_vacant<u64>(4)`, two element-position `replace`s, then  | **ACCEPTED**, exit 0                      |
|                  | `len(table)`                                                   |                                           |
| x8_screc         | recursive `descend` opening `region 's` and calling             | **ACCEPTED**, exit 0                      |
|                  | `arena_new<'s, u64>` on every activation                       |                                           |
| x9_capzero       | `buffer_new(0_u64, 0_u8)` and `len` of it                      | **ACCEPTED**, exit 0                      |
| x10_uniqdefine   | the same `ensures` as x5 through a `define`, `pure` body        | **ACCEPTED**, exit 0                      |
| x11_uniqkill     | callee writes one element through `&uniq buffer<u8>`; caller    | **ACCEPTED**, exit 0                      |
|                  | keeps `len(out) = 4` into a subscript                          |                                           |
```

What each establishes, and which rule it decided rather than confirmed.

Inherited verdicts this draft still rests on, from the sixteen falsifier sessions,
by what each group establishes. D1 reproduces at this tip (`d1`). [CALL-1], [CALL-2]
and [CALL-5] already behave and the struct-field route already kills correctly (`p1`,
`p6`, `f7`, `m04`, `s7`). `MutSpan` writes, affine elements, `len(result)` and
multi-return are new capability rather than compiler defects (`p7`, `p9`, `k12`,
`p2`, `p8`, `k09`, `r1_multi`). Allocation while holding nothing, and a free inside a
`pure` callee, are accepted (`p5_ambient`, `n4`, `r1_ambient`, `r2_5`, `q9`, `w7`,
`m02`) — L2's and L13's evidence. A view value, not its argument borrow, must hold
the loan, and a borrow region must open after its binding (`f1c`, `f1d`, `f2b`,
`r1_twouniq`, `r2_1`, `r2_2`, `c4`, `w8`). [LIV-1] replaces three avoidances (`f3`,
`f5`, `f6`, `r1_own11`, `s5`, `s6`) and [LIV-2] has two halves (`p10`, `w6`). The
syntactic tail conditions are refuted (`f2b_tail`, `f8_tailframe`, `p3_rec`) and the
idle and driver loops are `FunctionFallthrough` (`n2_idle`, `f3_forever`, `k30`,
`n3_propagate_loop`). [BLK-4] and [MSR-5] are new syntax (`f7_regionresult`, `k05`,
`r2_6`, `m05`, `r1_lenatom`, `r1_field`, `c1`, `c2`, `w3`, `m07`). The measure kill
is root-granular today (`r2_4`, `r2_4b`, `r2_4c`); element-position replace keeps a
`len` (`r2_7`, `k24`, `n13`); the arm-fact route and the readers' purity hold
(`r2_9`, `r2_10`); a box field and a replace of one must keep being admitted (`q1`,
`q2`); a partial move kills the root (`q3`, `q7`); no loop publishes `len = N` as an
equality (`n14`, `n15`, `n19`); a by-value transformation is not `pure` (`c8`);
[PROV-7] has a reason and a general reading has a price (`r1_relend`,
`r1_relend_affine`, `m19`); the fill loop's arithmetic and its conditional and
three-invariant shapes are accepted (`k21`, `k21b`, `k08`, `k31`, `b4b`, `m11`,
`m17`, `m10`); the arena-content stop, the partial-move drop, the recursive region
and the release walk's worklist are all executed (`a1`, `a2`, `a3`, `a5`, `a6`,
`a8`); and `par` eligibility plus three disjoint chain roots are the ledger read
(`n7_par`, `--stack-ledger`).

### 6.2 The proof surface, isolated

```text
| probe                      | shape                                                    | verdict                          |
|----------------------------|----------------------------------------------------------|----------------------------------|
| v23_param_anchored         | counted loop, header invariant, ensures over a parameter  | ACCEPTED                         |
| v24_len_anchored           | identical, ensures over len(deref(destination))           | REJECTED [FN-9]                  |
| x10_uniqdefine             | the same through a `define`, `pure` body                  | ACCEPTED, read as the ENTRY image|
| v25_subscript_consumer     | identical loop, consumer is a subscript                   | ACCEPTED under [OP-4]            |
| v26_ensures_consumer       | identical loop, consumer is an ensures                    | REJECTED [FN-9]                  |
| q2b / q3b                  | one file differing in one token                           | ACCEPTED then REJECTED, in one   |
|                            |                                                          | compilation                      |
| k22                        | ensures over a hoisted len after a proved loop            | REJECTED [FN-9], residual        |
| v22_loop_then_inv_stmt     | the [INV-1] conclusion proves and does not reach [FN-9]   | REJECTED [FN-9]                  |
| q5 / q5' / q5''            | one-line invariant header; -wrap on the backedge; exact - | [FORM-2]; [INV-1] Backedge; OK   |
| x1 / x1b / x1c             | the same three ways, on an increasing counter             | [OP-2]; [INV-1] Backedge; OK     |
```

`q2b`/`q3b` and `k22` are why [MSR-4] is a law-level change rather than a repair.
`x10` against `v24` is why [CALL-4]'s exit datum is a semantic addition and not a
grammar one.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in 3.K.** None is implemented, and no compiler has seen any of the
  new types, operations, terms, statements or markers.
- **Every function in 3.L.** They are written against 3.K and against the unchanged
  v0.40 rules and walked against both, and their proof routes are stated per
  function; none was compiled, because 3.K is not implemented.
- **Every program in section 4** and every diagnostic quoted there.
- **Every figure in 4.1's envelope**, which is why every one is written as a
  composition or as `<post-codegen>` rather than as a number.
- **[PROV-1]'s brand.** That a region can carry a store's identity as well as its
  extent, and that invariance follows from [OWN-12] 645 and [TYPE-5] 374 with no
  variance design, is argued from rule text — and all four round-4 reports attacked
  it from every position they could build and none of them moved it, which is the
  strongest evidence any part of this design has.
- **[PROV-1]'s elision.** That the rule is a function of program text, that it is
  local to one declaration, and that resolving it before [TYPE-5]'s check changes no
  judgment are argued and not executed. The claim most worth attacking is locality:
  if any occurrence's candidate set can be widened by something outside the
  enclosing declaration, two programs that differ nowhere visible would elide
  differently.
- **[PROV-5]'s activation refusal.** That call-graph SCC membership, one execution
  context, and the ordinary [PAR-1] footprint rules together cover every way two
  activations of one reserving occurrence can be live at once is argued from four
  rules and not executed.
- **[PROV-6]'s bounded walk.** That the containment height is the right bound, that
  the walk's order matches [STOR-3]'s leaf for leaf, and that the enum case's
  discriminant selection is ordinary drop glue are argued and not executed. Probe
  `a8` is the mechanism it replaces and probes `a5`/`a6` are the shape it keeps.
- **The compiler defect at `[SET-2]`'s arena half**, found in round 3 and confirmed
  in round 4: [SET-2] 512 makes a region-bearing `replace` target a hard error for
  `slice<'r, U>` **and** `arena<'r, U>`, and `check_mutation_target_class`
  (`compiler/src/semantic/check/expressions.rs:310-326`) tests only the slice
  variant. It is benign at this tip and load-bearing for the batch that implements
  [PROV-3] use 3, which must be a relation over loan-bearing types and not a
  re-wording over one `CheckedType` variant.
- **The composition algebra of 3.K.7.1.** Its sequence and branch rules over an
  exit-label map are standard, the no-fallthrough case is defined, the interval
  arithmetic is stated, and the loop's own map is now stated per discharge. Its `par`
  rule depends on a runtime profile that does not exist, and its derived-release
  transfer has never been composed against a program with a nonzero `slots` row.
- **[MSR-3]'s four placements**, checked by enumeration and not by execution, the
  construct placement being the newest and least attacked; **the current runtime's
  closure**, which no existing target can be certified to meet and whose ledger read
  shows three disjoint chain roots; and **the claim that `wfgrep` becomes
  heap-free**, whose substitution was never compiled and which moves bytes out of the
  heap into frames, a [STK-3] question rather than a free win.

### 6.4 Falsifiers this design asks for next

1. **Write 3.L against 3.K by hand, one function at a time, and find the eighth
   kernel addition.** The partition test is this draft's central claim and one
   falsifier round is not enough of it; the most valuable single result would be a
   library function that cannot be written and whose missing primitive is not one of
   3.L.6's seven.
2. Attack **[PROV-5]'s activation refusal** with a way to make two activations of one
   `arena_extent` occurrence live at once that is neither an SCC edge nor a second
   execution context — through a `par`-permitted window, a rewritten tail component,
   or a library function the compiler inlines.
3. Attack **[PROV-6]'s bounded walk** with a linear value inside an `Option` a join
   left in different variants, a type whose walk order differs from [STOR-3]'s, and
   the destructuring consume applied to a nominal one of whose fields is itself
   linear and partially moved in the same statement.
4. Attack **[CALL-4]'s exit datum** with a callee that consumes a `&uniq` parameter's
   referent through [SET-2], one that reborrows it into a further call, and a caller
   holding a second live borrow of an overlapping place.
5. Attack **[LIV-3]'s exchange** where the call's own arguments read the target's
   offset, where the call diverges or propagates, and where the first result's type
   equals the target's only after region substitution.
6. Attack **3.K.0's assumption**: find a position whose spelling is not decidable
   from the declaration text alone, or two programs that differ only in a region name
   and would be spelled differently.
7. Hand-execute 3.K.7.1 on 4.1, on `CONTAINERS.md` §3.4's pool, and on a hosted
   program with ten exits, checking the loop's own map and the derived-release
   transfer against all three.
8. Rewrite `wfgrep` and `byte_string` by hand against [VIEW-7], [PROV-6], 3.K.0 and
   Q10's refusal, and count what remains.

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
| F2-NB11 GAP K<T> is a ceiling where an exact advance exists     | [RES-5]: advance<T> is exact when align >= align_ceiling(T),|
|                                                                 | required at the take as a comparison of two constants       |
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
| F3 I3 five wrong line numbers                                   | corrected: [TYPE-5] 374, [ENT-2] 2677, [FN-9] 1339,         |
|                                                                 | [ENT-5] 2881-2885, [OWN-7] 625                              |
| F3 I4 four ranges overshoot a blank line or a heading           | corrected: [SCOPE-3] 27-31, [OP-9] 968-996, [FN-9]          |
|                                                                 | 1295-1360, [PAR-3] 2029-2056                                |
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
| F3 I18 [CALL-4]'s examples disagree with section 4's            | the examples are rewritten from section 4                   |
| F3 I19 [VIEW-4]'s ground covers only a borrowed descriptor      | the bare `own MutSpan` case is the second bullet's          |
| F3 N1..N9 nine notes                                            | N1 moot (absorb deleted); N2 the shared-loan freeze is      |
|                                                                 | [OWN-5] 583 and is cited; N3 [PROV-6]'s propagate refusal   |
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

---

## 7. Implementation order

**This is an implementation order and nothing else.** The owner's ruling of
2026-09-03 says so in terms: batches are an order of work, not spec versions, and a
single implementation is fine if it is correct. Nothing below is an approval, a
schedule, or a licence to trade a rule away for a cheaper batch; one batch that lands
all forty-eight rules correctly is the better outcome. The order is *for* naming, at
each step, a test writable before the next step exists.

**B0 is not one of these batches.** The region-spelling amendment (3.K.0) lands
first, separately and mechanically, and is not this design's work; every batch below
assumes it and none of them implements it.

**B1. Type-derived call transports.** Rules: [CALL-1], [CALL-2], [CALL-3],
[CALL-5]. First of the container work because it is the live defect and needs none
of the new types: today's `&uniq buffer<T>` keeps its spelling and gets [CALL-5]'s
type-derived classification, `element = false`, which is exactly the sweep's minimal
sound repair. Test: **`ent5-neg-callee-uniq-buffer-replace-kills-length.wf` turns
XPASS**, rejecting at [OP-4] with residual `9_u64 < len(line)`; plus probe `x11`'s
program, whose accept becomes the same rejection; plus one positive case pinning
[CALL-1]. `docs/patterns.md` P16 is corrected in the same change.

**B2. The proof surface.** Rules: [MSR-1], [MSR-2], [MSR-4], [MSR-5]. Second because
every later batch's contracts and invariants are unwritable without it, and because
it is a specification amendment with no new construct. Tests: a conformance pair
mirroring `v23`/`v24`, both accepted after the amendment; one mirroring `v25`/`v26`
so two consumers of one exported invariant agree; one mirroring probes `w3` and
`x5`, a clause whose operands are two `len` terms, accepted where it is a [GRAM-9]
parse failure today; one pinning that a literal and a parenthesized group are still
affine factors; one discharging a goal from `len + room = cap` as an affine premise;
one pinning that an element-position `replace` of a *descriptor* kills its measures
and of a *scalar* kills nothing, which is the carve-out's removal under test; **and
`r2_4`'s program accepted**, because [MSR-2]'s descriptor-precise support is a repair
of a live over-kill and not only a new rule.

**B3. Multi-return, the exchange, and join-checked liveness.** Rules: [CALL-4],
[LIV-1], [LIV-2], [LIV-3]. Third because B5 and B6 are written in this syntax.
Tests, all writable in today's vocabulary plus B1 and B2: probe `p8`'s signature
parses and binds, and a two-result `ensures` reaches both binders of a destructuring
`let`; **probe `x2`'s and probe `x3`'s programs are accepted**, which is the exchange
under test at a bare binding and at a `deref`; an exchange at a `buffer` element
place is accepted where probe `q7`'s `set` spelling is [OWN-1] today; a two-target
exchange binds its second result; probe `p10`'s program and probe `w6`'s are both
accepted after [LIV-2]; probe `f3`'s program is a [LIV-1] error naming both
predecessors instead of `SemanticUnsupported`; a loop moving and restoring an outer
binding is accepted where probe `f5` is [OWN-11] today; and **probe `x10`'s `ensures`
read as an exit datum**, with a caller that discharges from it, which is [CALL-4]'s
addition under test.

**B4. Measure datums, images, and atom identity.** Rules: [MSR-3]. Separated from B2
because it touches [ENT-2]'s term list, [ENT-5]'s call boundary and [ENT-6]'s
transfer machinery, and because it needs [LIV-2] and [LIV-3] from B3. Tests: a
`buffer` helper whose `ensures` names `len` of a parameter it consumed is accepted,
and its caller establishes the relation where `M(c,q)` refuses it today; a relation
over a **borrowed** owner's measure establishes at the caller; a reinitialized
binding does not inherit a fact stated over its predecessor; **a header invariant
over a binding an exchange rewrites is preserved on the backedge**, with the [LIV-2]
variant rejected so the two forms are pinned apart; and a `construct` carrying a
measured operand publishes the field's measure, which is `CONTAINERS.md` §3's `Ring`
under test.

**B5. The brand, the runs, typestate, confinement, and the declaration domain.**
Rules: [PROV-1], [BLK-0], [BLK-1], [BLK-2], [BLK-3], [BLK-4]. Retires `buffer<T>`,
`box<T>` and `arena<'r, T>` from the writer surface. Carries monomorphization for a
compiler-owned generic domain. Tests: a `FixedVector<Handle, 64>` object table with
affine elements, filled by 3.L.3's `vacant` and compacted by `CONTAINERS.md` §3's `take_at`,
accepted, where probe `p9` is [OP-1] today; a `vacant` result whose `ige(len, n)`
discharges a subscript with no equality anywhere, which is probes `x1c`/`x1d` under
test at full scale; `struct Chunk['s]` accepted where probes `r2_6` and `m05` are
parse errors today, with two instances at different regions rejected as distinct
types; a stored brand elided in a heap-only program and written beside an arena,
which is 3.K.0's assumption under test; and **two reserving occurrences naming one
region rejected at the second**. This batch supersedes B1's conformance case, whose
program no longer typechecks; that disposition is conformance evidence and is
recorded in `governance/APPROVALS.md` with the merge.

**B6. Views, loans, ranges.** Rules: [VIEW-1], [VIEW-2], [VIEW-4], [VIEW-6],
[PROV-3], and the view rows. [PROV-3] lands here because views are its only user and
because [SET-1] and [SET-2] must change in the same batch that admits the `MutSpan`
write. Tests: an element write through a `MutSpan` is accepted where probe `p7` is
[SET-1] today; **a `replace` through `&uniq MutSpan` is rejected**, and so is a
`replace` of a `Vector` place under a live origin set, which probe `w2` shows the
compiler accepts today for the arena spelling, so use 3 must be a relation over
resolved origins and not one `CheckedType` test; two `MutSpan`s on one run are
rejected at the second formation citing [OWN-5]; a write to `k` while a view formed
at `table[k]` is live is rejected citing the view's loan; and a two-result signature
with two same-region view results is rejected at [VIEW-6].

**B7. Stores, the heap as a value, and structural release.** Rules: [PROV-2],
[PROV-4], [PROV-5], [PROV-6], [PROV-7], [RES-6]. Tests: probe `p5_ambient`'s program
is **rejected**; a `main` that omits `command.heap` cannot reach any allocation;
probes `r2_5`, `w7` and `m02` are rejected with [PROV-6]'s diagnostic and their
repairs compile; **probe `x4`'s program is rejected with `LinearValuePartiallyMoved`**
and its destructuring-consume repair compiles; a run released to a store of a
different region fails to typecheck with the two types rendered; `dispose` of a
`FixedVector<Chunk<'a>, 8>` compiles and frees every leaf, and the same statement
with a missing provider is `DisposeProviderMissing`; probes `w5` and `m03` are
rejected with `LinearValueAcrossPropagate`; a region block entered twice by a loop
republishes `len(store) = Z` truthfully; **probe `x8`'s program is rejected with
`ExtentReservedOnACallCycle` under `arena_extent` and accepted under `arena_frame`**;
**probe `x6`'s self-referential type is rejected at its `dispose`**, naming the
cycle, and its `a5`/`a6` non-recursive sibling still compiles to a straight-line
walk; an arena-backed run of `ReadFile` closes every handle at its scope exit, which
is the reset/content split under test; a helper lending a provider onward compiles,
where `r1_relend` and `m19` are [OWN-6] today; and two overlapped disposals from one
store are denied [PAR-1] permission while a window containing one is not.

**B8. System I/O over views, and the handle table.** Rules: [VIEW-7], [RES-9].
Tests: `tests/programs/wfgrep.wf` migrated to 3.L.3's `filled` and `MutSpan`,
compiling with no `allocates` entry anywhere on its call graph — the first program
that demonstrates goal A's container half end to end; **and a marked `main` selecting
`command.files` and `command.cwd` that opens one file in a loop, reads it into a
`filled` destination over a `MutSpan`, and publishes a handle row of one**, which is
the witness goal A's I/O half has never had and which is the single test that would
have caught F2's NB3, NB4, NB5 and NB8 at once.

**B9. The stack judgment.** Rules: [STK-1], [STK-2], [STK-3]. Tests: probes
`f2b_tail` and `f8_tailframe` are **not** rewritten by [STK-1]'s premise and are
rejected by [STK-2] under the marker; their borrow-free variants are rewritten into
one dispatcher with one frame; a member holding a live confined value across the jump
is not rewritten, nor is one that opens a region for an `arena_frame`; probe `p3_rec`
stays accepted without the marker; and a `--stack-ledger` run reports one chain per
context rather than disjoint roots.

**B10. The divergent entry.** Rules: [STK-4]. Tests: probe `f3_forever`'s idle loop
is accepted; **probe `n3_propagate_loop`'s driver loop is accepted**; a loop with a
reachable `break` still requires a return; and a linear binding live on a path that
reaches only a divergent loop is accepted and appears as a retained item of the
enclosing scope's published map.

**B11. The envelope and the judgment.** Rules: [RES-1] to [RES-5], [RES-7], [RES-8],
[RUN-1], [RUN-4], [RUN-5]. Tests: 4.1 is source-resource-closed and its `E` matches a
pinned symbolic expectation; 4.2 is reported not resource-closed with the
heap-reaching path rendered; a retaining loop whose trip count is a runtime value is
rejected at that loop with the value named; one whose checked refusal rejoins the
backedge is **accepted**; one of four iterations followed by one more acquisition
publishes a peak of five and not two (NB9's repair); the same loop with its
acquisition one function down is accepted through [RES-8]'s saturation flag (NB10's);
a loop whose only discharge is the standing `len <= cap` is rejected; B8's marked
file program composes its handle demand and is rejected when it exceeds the profile
cap; and a program whose demand exceeds every profile row fails **target
qualification** citing no language rule.

**B12. `par` and the envelope.** Rules: [RUN-2], [RUN-3]. Tests: a `filled` plus
`MutSpan` plus counted subscript fill receives [PAR-2] permission in an unmarked
program, which needs the ranged origin and which no earlier draft could pass; the
same loop inside a `resource_closed` entry receives no permission and the published
row reads `lanes(1)`; two overlapped statements allocating from distinct providers
are permitted and two from one provider are not; a permitted window containing a
`dispose` is not denied; and 3.K.7.1's `par` rule composes against a pinned profile
row.

**3.L is not a batch.** It is written against the rules, not implemented beside
them; where its functions are useful as evidence — `filled` in B8, `collect` and
`vacant` in B5, the pool in B11 — they land as test programs under
`tests/programs/`, which is where 5.0 recommends they stay.

---

## Appendix A: generated data

Two tables the rule text refers to and does not contain. **Neither is a rule.**
[BLK-0] says that an operation inventory exists and what every row of it must
satisfy; [MSR-1] and [RES-5] say that a measure table and a ceiling table exist and
what every row of them must contain. The tables themselves are **generated data**,
carried the way [SYS-2]'s declaration records are carried, and a diagnostic cites the
rule and names the row in its payload rather than citing the row.

### A.1 Measures and ceilings

Derived from [BLK-1]'s storage column rather than written per nominal: a value whose
backing is a run of its own is a descriptor, and a value whose backing is inline
carries its elements.

```text
| measured type            | len                  | cap            | room      | len is   |
|--------------------------|----------------------|----------------|-----------|----------|
| array<T, n>              | n                    | n              | Z         | exact    |
| FixedVector<T, n>        | initialized elements | n              | cap - len | exact    |
| Vector<'s, T>            | initialized elements | slots taken    | cap - len | exact    |
| Span, MutSpan            | viewed elements      | len            | Z         | exact    |
| Arena<'s, bytes, align>  | cursor bytes         | bytes          | cap - len | monotone |
| FileFactory              | live handle records  | the profile's  | cap - len | exact    |
|                          |                      | handle-table   |           |          |
|                          |                      | row [RES-9]    |           |          |
| Heap<'s>                 | none                 | none           | none      | -        |
```

`Heap<'s>` has no row because L6 says a general store has no measure that means
anything; that is the absence of table data, not an exception clause on a total
definition. `Arena`'s `len` is the one monotone measure in the design, because the
store's own alignment padding is a target-stage quantity, and L15's second half is
what a monotone row publishes instead of an exact value.

```text
| nominal                     | (size_ceiling, align_ceiling)                          |
|-----------------------------|--------------------------------------------------------|
| Heap<'s>, Arena<..>         | (32, 16)   proof-only representation, one word         |
| Vector<'s, T>               | (32, 16)   a descriptor: one pointer, one cap, one len |
| FixedVector<T, n>           | T's pair repeated n times, plus (8, 8) for the length  |
| Span<'r,T>, MutSpan<'r,T>   | (32, 16)                                               |
```

`advance<T>` for the bump domain is `round_up(len, align_ceiling(T)) - len +
size_ceiling(T)` when the store's own `align` is at least `align_ceiling(T)`, which
`arena_take` requires as a compile-time comparison of two constants, and the ceiling
`align_ceiling(T) - 1 + size_ceiling(T)` otherwise.

### A.2 The kernel operation inventory

Fourteen rows. `V` is either run type. Every row is complete over the measures it
writes, on every exit, as [BLK-0] requires; where two of three follow from [MSR-2]'s
identity the row states one and this table says so.

```text
Formation
  seq_fixed<T, const n: u64>()                       -> own FixedVector<T, n>          pure
      len(result) = Z, cap(result) = n
  seq_frame<T, const n: u64>['s]()                   -> own Vector<'s, T>              pure
      len(result) = Z, cap(result) = n            its own region item of E [PROV-5]
  arena_frame<const bytes: u64, const align: u64>['s]()  -> own Arena<'s, bytes, align>   pure
      len(result) = Z, cap(result) = bytes
  arena_extent<const bytes: u64, const align: u64>['s]() -> own Arena<'s, bytes, align>   pure
      len(result) = Z, cap(result) = bytes
  seq_arena<T>['s](arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Option<Vector<'s, T>>                   allocates(arena), writes(arena)
      Some(value: r): len(r) = Z, cap(r) = count,
                      <datum of len(arena)> <= len(arena) <= <datum> + advance<T> * count
      None:           len(arena) = <datum of len(arena)>, room(arena) < advance<T> * count
      both:           cap(arena) = <datum of cap(arena)>
  seq_arena_proved<T>['s](arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Vector<'s, T>                           allocates(arena), writes(arena)
      requires ige(room(arena), advance<T> * count)
      as the Some row above
  seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)
      -> own Option<Vector<'s, T>>                   allocates(heap), writes(heap)
      Some(value: r): len(r) = Z, cap(r) = count
      None:           nothing; a general store publishes no measure (L6)

Per slot
  seq_place(vector: own V, value: own T)             -> own V     reads(vector), writes(vector)
      requires igt(room(vector), Z)
      len(result) = len(vector) + 1, cap(result) = cap(vector)
  seq_take(vector: own V)          -> (rest: own V, value: own T) reads(vector), writes(vector)
      requires igt(len(vector), Z)
      len(rest) = len(vector) - 1, cap(rest) = cap(vector)
  seq_exchange(vector: own V, first: own u64, second: own u64)
                                                     -> own V     reads(vector), writes(vector)
      requires ilt(first, len(vector)), ilt(second, len(vector))
      len(result) = len(vector), cap(result) = cap(vector)
      the elements formerly at first and second are at second and first

Readers                       ([OP-1] table rows, not this domain)
  len(p) / cap(p) / room(p)                          -> own u64   pure

Views
  seq_span['r](vector: &'r v)                        -> own Span<'r, T>      reads(vector)
      len(result) = <datum of len(v)>, cap(result) = <datum of len(v)>
  seq_mut_span['r](vector: &uniq 'r v)               -> own MutSpan<'r, T>   reads(vector)
      len(result) = <datum of len(v)>, cap(result) = <datum of len(v)>
```

Two statements are not rows and are stated in [PROV-6]: `dispose p using (q1, ...);`
and the destructuring consume `let N(f1: b1, ...) = move v;`.

Notes on the inventory. **`seq_place` is the operation the whole design exists
for**: total under its requirement, allocation-free on every backing, one store plus
one length increment. **`seq_exchange` earns its row from `CONTAINERS.md` §3.3**,
the only place order-preserving relocation is written. **Nothing here is total at a
capacity boundary**, because an overwriting form would need L9's published
displacement. And **nothing here removes from the middle, clears, truncates, grows,
or constructs a filled or vacant run** — each is 3.L, and 3.L.6 records that none
needed a row the three per-slot operations do not have.
