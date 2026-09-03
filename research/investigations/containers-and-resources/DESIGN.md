# Containers and resources: the integrated design

The single design for batch 0116: one set of laws, one set of rules, one
vocabulary, one amendment register. `RESOURCES.md` beside it keeps the writer's-eye
resource migrations and `CONTAINERS.md` the longer library functions of 3.L; neither
carries rule text, and a reader who reads only this file has the whole design.

**Seventh draft, after falsifier round 6 and the owner's decisions of 2026-09-03.**
Round 6 confirmed the sixth draft's two rulings — value-in / value-out, and linearity
derived from one criterion — and then found the same defect in three independent
lenses, one level up from round 5's. Round 5's summary was *a channel that carries an
effect past a kill*; round 6's is **a claim that carries a premise past the judgment
that was supposed to establish it**. `[VIEW-4]` claimed no operation produces a view
of a different length and judged only a commit, and `[CALL-3]` read the claim and
handed a caller a stale length into an out-of-bounds write. `[PROV-6]` claimed
`dispose`'s consume half means the disposer owns the storage and judged only that the
root is own-mode, and a view is an own-mode value that owns nothing. `[PROV-6]`
claimed linearity is closed under containment and judged a walk with no action for a
declaration-linear leaf. `[MSR-3]` claimed "a parameter is an input and has one
state" and judged every parameter by it, including the `&uniq` provider the same
draft keeps.

That pattern has a mechanical answer and 3.K.11 now carries it as its **seventh
condition**: *every fact a rule states must appear in that rule's `Judgment:` line,
and every rule that reads such a fact must name the judgment it comes from.* Every
rule below has been rewritten against it, and the four findings above are what it
catches.

The owner decided two things this round, and both are written below as **decided,
not proposed**.

> **D1 (owner-decided 2026-09-03). The `linear` modifier is adopted, together with
> derived linearity.** Both halves are kernel. The criterion — *a value whose release
> action requires a capability is linear; a value whose release requires nothing is
> affine* — stays exactly as R2 stated it, and the modifier stays what R2 reserved it
> for: a **logical** obligation the criterion cannot see. `[S18]` is ADOPTED in 3.S,
> which this draft rewrites as a decision record.
>
> **D2 (owner-decided 2026-09-03). `set` has one commit rule.** The right-hand side
> is fully evaluated first, during which **every target is dead**; then **all targets
> are reinitialised at the commit**. Multi-target `set (a, b, ...) = ...` is the n-ary
> case and `n` is unbounded. The right-hand side may be a multi-return call or a value
> list of matching arity, evaluated left to right, so `set (p, x) = move x, move p;`
> is a swap and three targets rotate. Targets must be pairwise **non-overlapping
> places**: a place and its sub-place, or two subscripts of one run, are refused. The
> single-target `set p = f(p: move p, ...)` is the same rule. **There is no
> "exchange admission" as a separate clause, and no swap or exchange operation exists
> anywhere, kernel or library.** This amends `[STOR-1]` and `[SET-1]` — the owner's
> citation of 674 is v0.40's number; the sentence is v0.41's **679** and the partition
> it restates is **678**. `[S14]` and `[S15]` leave 3.S as retired ids and `[LIV-3]` is
> retired into `[LIV-2]`.

D2 is the larger of the two and it is a simplification rather than an addition. The
sixth draft had three writing forms with three rules — `[SET-1]`'s copy overwrite,
`[LIV-2]`'s reinitialization at a dead binding, and `[LIV-3]`'s in-place exchange —
each with its own admission, its own atom-identity ruling and its own effect
footprint. One rule replaces all three, the swap and the rotation fall out of the
n-ary form instead of being asked for, `[MSR-3]`'s orphaned-invariant diagnostic
disappears because a target that names a binding in scope keeps its term, and round
6's finding that a later target silently redeclares a name disappears with it.

Two other structural moves are the orchestrator's and are stated where they land.
**R1 becomes a rule**: `[BLK-4]` refuses a container nominal or a loan-bearing type as
the direct or indirect referent of a `&uniq` parameter of a source-declared `fn`,
which is the sentence six drafts have asserted as doctrine and none has judged, and
which closes round 6's first two BREAKS at their declarations. And **linearity closes
under ownership rather than under type syntax**: a loan-bearing value owns nothing,
so a view of runs is not linear and `dispose` cannot reach through it.

Tree read: `batch/0116-containers-and-resources` at `main` 30602914,
`spec/kernel-spec.md` **v0.41 ACTIVE**. Bare three- and four-digit line numbers are
that file at 30602914; every other citation names its file. v0.41 respelled the six
integer comparisons as infix `== != < <= > >=`, delimited call-site type and region
application with `::`, and put the four ordered symbols in proof position; every
clause, invariant and call below is written in that surface.

**Nothing here is implemented.** No compiler code was written for it. Section 3.K is
draft rule text for a work branch, not an amendment; section 3.L is design text for
programs that compile nowhere. Section 6 separates what a compiler executed in this
session from what is argued on paper.

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
- Mutation of container state through `&uniq` is retired — and, from this draft,
  refused by a rule rather than by doctrine [BLK-4].
- Multi-return `-> (a: own T, b: own U)` with `let (a, b) = f(...)`.
- System I/O goes over views.
- Every rule is a deterministic function of program text and compiler version,
  never of time or of a work budget.
- **Linearity is derived from one criterion, and the `linear` modifier exists for a
  logical obligation** (D1).
- **`set` has one commit rule, n-ary, with every target dead through the right-hand
  side and every target reinitialised at the commit** (D2).

Five footnotes, because the minimality ruling and R1 move material the settled list
names. Each states what survives and what changed; 5.0 collects them.

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
   ordinary element access. 3.K.3 states what the window costs and [BLK-3] carries the
   one row round 6 showed a window needs.
3. **`AppendView`.** The settled list names the owner/view split and the by-value
   transformation, both of which survive. `AppendView` and `absorb` were the fourth
   draft's device for keeping a caller's length alive across an appending callee;
   the fifth draft replaced them with [CALL-4]'s exit datum, which R1 withdrew.
   Under R1 an appending helper takes the run **by value and returns it**, so the
   caller's length is the *result's* length and no device is needed at all.
4. **`update`, `swap` and `seq_exchange`.** The fourth draft's transformation
   statement, the fifth draft's exchange row and every swap spelling any draft
   proposed are gone. Under D2 there is one assignment statement; a transformation is
   `set p = f(p: move p, ...)`, a swap of two whole places is
   `set (p, q) = move q, move p;`, and a swap of two elements of one run is refused
   by D2's non-overlap condition and written in three statements over rows the kernel
   already has (3.L.2).
5. **Argument order.** The settled append example writes its source argument
   first, while [GRAM-11] fixes argument order from the declaration and every
   helper here declares its destination first.

## Contents

1 [The problem](#1-the-problem) · 2 [The laws](#2-the-laws) and
[the ten notions](#21-the-ten-notions-and-their-closures) ·
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
morning because it lost the last block of a store it owns — which is what round 5
found in this design's own flagship program, and which round 6 found again one store
over, in the **arena**. 1.1's promise is not kept by a value that is reclaimed
eventually; it is kept by a program whose demand on every covered store is a bound,
and [RES-10] is where this draft makes that true for a bump extent as well as for a
heap.

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
manifest line 165, status `xfail`. **Re-run in this session against the v0.41 gate
binary as probe `r1`: accepted, exit 0 — and re-run as probe `r5`, with every region
name elided, against a build that implements 3.K.0's amendment: accepted there too.**

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

**Under this draft the program has no shape, and a rule says so.** `shrink`'s
parameter is a run behind a `&uniq`, and [BLK-4]'s fourth clause refuses a container
nominal as the direct or indirect referent of a `&uniq` parameter of a
source-declared `fn`. A helper that transforms a run takes it by value and returns
it; a helper that only writes elements takes a length-fixed view. Neither can change
a caller's length behind the caller's back, because in the first case the caller's
length is the *result's* length and in the second the view reaches element storage
only.

**This is the fourth disposition D1 has had and the first that is a judgment.** The
fourth draft refused the parameter by its direct type ([CNT-7]), which a one-field
wrapper struct nullified. The fifth draft deleted the refusal and relied on a
conservative kill, which round 5 defeated with a fact published *after* the kill. The
sixth draft withdrew the parameter by doctrine and retired [CNT-7]'s text, and round
6 observed that doctrine refuses no declaration and wrote the program again. The
clause below is stated over the reachability closure [PROV-4] already computes, so
the wrapper defeat is closed by construction, and it is a rule, so it refuses.

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

And make each of those a property that is **closed** and that some rule
**judges**. Rounds 3, 4 and 5 each found one notion introduced without its closure;
§2.1 is this draft's answer to that pattern. Round 6 found a different pattern — a
notion with a closure sentence but no judgment — and 3.K.11's seventh condition is
this draft's answer to that one. The two checks are mechanical and are run over every
rule below.

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

Round 5 applied the test in the other direction, which no earlier round had done, and
round 6 sharpened the standard it must be applied at. **An L18 removal must be priced
against a walked program.** The sixth draft removed `seq_exchange` on the strength of
a three-statement replacement whose `requires` was one unit short of the obligation
its own body carries, so the trade the owner was asked to accept was priced with the
wrong number. 3.L.2 walks the replacement and states the real price, and 3.K.11's
seventh condition is what makes the same mistake visible in a rule.

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

**A worker lane is an execution context**: [RES-1] counts its stack, [STK-3] gives it
an item, and [RUN-4] creates it. What is true of source is narrower: *no source
construct creates a context whose chain the program controls.* A `par`-permitted
window may therefore put two activations of one reserving occurrence in two contexts,
which is precisely why [PROV-5]'s refusal names that source.

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
owner ruling R13 (`L7036`), B8, [SCOPE-2] 18, [STOR-6] 738. Round 6 found the sixth
draft breaking the first sentence in one place — [RES-7]'s exclusion test read a
figure the *runtime* publishes and issued a source rejection from it — and [RES-7]
now splits at the stage boundary this law draws.

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
(`L5657-5666`), B3, audit answer Q8. The last clause is round 4's; round 5 showed it
was still aspirational for a cyclic containment graph, and round 6 showed the repair
over-refusing, so [PROV-6] now refuses exactly the cycle its own release walk would
follow.

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
never one byte total. Every item carries the shape its members have: an extent
carries bytes and an alignment, a countable object carries a count, and a store the
program itself reserves is shaped by the same rule.*
Because sixteen bytes holding four four-byte objects, the first and third released,
cannot serve an eight-byte request, and a deployment reading one stack number
cannot tell an alignment failure from a size failure: owner ruling R12, B9, B11.
Round 6 found the sixth draft breaking this law twice inside its own item list, so
[RES-2]'s shapes are corrected and every runtime-owned extent is a `region` item.

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
changes a loop's summary by nothing: B8, owner ruling R12. Round 6 showed the second
half being read too far: a store's refusal bounds what is **held**, so it bounds a
reusable-capacity domain and says nothing about a consumable budget, where refusal is
exhaustion rather than boundedness. [RES-10] states the split.

**L9. Stock, not flow, and a total operation at a capacity boundary must say what
it dropped.** *Resource-closedness bounds what is held at once and what is consumed
irreversibly; it never bounds how many times a program acts. An operation may be
total at a capacity boundary only when the value it displaces is copy and its
displacement is a published relation the caller can read; a silent drop of an
affine value is a refusal wearing a disguise and is inadmissible under L3.*
The first half is why a service loop runs forever with one live slot; the second is
why "overwriting is this ring's semantics" cannot be written on every bounded
store: B8, owner ruling R12.

**L10. A view is a value, it holds its own loan, and it owns nothing.** *A view is
an affine value with a static type, not a reference the callee writes through; it
holds, for its whole life, a loan of its own strength on the range it reaches of
every place in its resolved origin set, beginning at formation and ending when the
view value is consumed or released; a function that changes a view's state consumes
it and returns the new one. A loan covers every binding the address computation of
its place reads, for the loan's whole life. **A loan-bearing value owns nothing**:
what it reaches belongs to its origin, so no obligation and no release action of what
it reaches is ever a property of the view.*
The first clause answers write-back without a hidden protocol, the range is what
makes a `par` fill over one owner expressible, and the address-computation clause is
what round 2 found missing when a view formed at `table[k]` left the offset writable:
owner's settled decision of 2026-09-03, B6, probes `f1c`, `f1d`, `f2b`, `r1_twouniq`.
The last sentence is round 6's: without it `[PROV-6]`'s containment closure made
`slice<'r, Vector<u8>>` linear, `dispose` walked it, and eight runs were freed through
a shared loan.

**L11. Length is a type fact or a contract fact, never a guess.** *At every program
point the checker's knowledge of a sequence's measures comes from exactly one of:
the type, an established fact with live support, a compiler-owned measure datum, or
a verified contract relation; no rule infers a measure from the shape of an
argument, the name of a callee, the absence of a write, or what a body was seen to
do. A relation about a value a callee received names that value; no relation
describes a state of a caller's object at a point the callee cannot name. **A
container nominal and a loan-bearing type are therefore not reachable through a
`&uniq` parameter of a source-declared function at all**, because such a parameter is
the one position from which a callee can leave a caller holding a measure of a value
the callee replaced.*
This is D1 stated as a law. The second sentence is R1 stated as law and it is the
sentence the fifth draft's exit datum broke; the third is R1 stated as a *rule*'s
premise, which round 6 showed the sixth draft still leaving to doctrine.
`EVIDENCE-sweep-D1.md`, probes `w3`, `d1` and `x11`, all accepted today.

**L12. The initialized region is a window, and the language says so.** *A run of
slots is exactly the `len` slots beginning at `head` modulo `cap`, initialized, with
the rest raw; the boundary is checker-maintained typestate carried by the run's own
value, and no per-slot tag, occupancy bitmap, or runtime discriminant is language
state. The kernel admits exactly append and removal at each end, plus the one
operation that returns a wrapped window to its origin; every other order is
arithmetic a writer performs over those five. A logical offset is the coordinate every
rule of this design speaks in, and the map from it to storage is stated once.*
With no per-slot state the checker never needs a quantified proposition over slots,
and occupancy at a stable index is ordinary data. Under a *prefix*, "every other order
is arithmetic a writer performs" was **false for a queue**, and the price was a library
ring over `Option<T>` measured at seven times a hand-written byte ring, with in-place
slot mutation deleted. Round 6 added the law's last two clauses: `head` was an
**absorbing** state with no row returning it to zero, so a wrapped ring could never be
viewed again; and the logical/physical coordinate question was left to be inferred by
[PROV-3], [OWN-7] and [PAR-2] separately. Owner's settled decision; audit answers Q2,
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
affine type and not a third class, **it is closed under ownership** — a type is
linear when it *owns*, at any depth, a linear value, and a loan-bearing type owns
nothing — and a linear value leaves a scope only by being moved out whole, by being
destructured whole, or by being disposed to the store its type names. A partial
consume of a linear value is refused because it is the one way a leaf leaves by none
of the three. No source construct selects, replaces, or observes a release action,
and a store's storage reclamation never stands in for its content's own release.*
Sentence one is round 3's rank-one repair and has survived every position attacked
since. The capability criterion is R2 and it is what makes the rest derived rather
than declared: with the heap an explicit value [L2], an implicit scope-exit free
would have to smuggle the capability, so a heap-backed run is linear by construction
and no writer marks it, while an arena-backed run and a frame-resident run need
nothing to reclaim them and stay ordinary affine values. The modifier is D1 and is
reserved for what the criterion cannot see — a value whose silent drop is a *logical*
bug. **Closure under ownership rather than under containment is round 6's**, and it
is the difference between a rule that frees a caller's runs through a shared view and
one that does not.

**L14 is retired.** It stated that an `AppendView` reaches only what it appended
and never decreases its owner's length. The type is gone (footnote 3). Under R1 the
guarantee it bought is an ordinary clause relating a result to an input —
`ensures len(rest) >= len(out)` — so nothing replaces it and nothing is lost.
The id is not reused.

**L15. The descriptor's measures are values; the allocator's extent is not; and a
measure a caller needs is published by whoever wrote it.** *`len(v)`, `cap(v)`,
`room(v)` and `head(v)` are a run's own logical measures and are readable as ordinary
`u64` values. No operation observes the physical extent the allocator provided. Every
operation that writes a measured place publishes, for each measure of that place, its
exact new value where that measure is exact and a two-sided bound where it is not,
including the measures it did not change, on every exit including a refusal. **That
obligation is on every operation, and a function that hands a measured value back is
an operation.** A row never leaves a measure to be reconstructed from the standing
identity.*
The first draft forbade reading `cap` and `room` on a rationale that only forbids
reading the allocator's size, so every pop proved and no push did: B3, audit answer
Q9, probes `q24`, `v25`, `v26`. The exact/bounded split is round 4's, over an arena's
monotone cursor and now also over a window's head. The completeness sentence is round
5's; **its quantifier is round 6's**, which showed it binding the twelve kernel rows
and not the wf functions written over them, so `filled` published no `head` and no
constructed run could ever be viewed, and `collect` published no `room` and no run
could be appended to twice. [CALL-7] is where the wider quantifier lands.

**L16. One measure algebra, one goal disposition, and one denotation per position.**
*`len`, `cap`, `room` and `head` are one-place terms of the term language, defined
once with their support, their kills and their standing identities, over every
measured place: runs, views, and providers alike. Every consumer of a numeric goal
asks one question, whose complete admitted derivation is stated once; no rule grants
a proof route to a construct by name. **And one spelling has one denotation at each
position at which it can occur**, stated once in a table rather than distributed over
three rules.*
A language in which "can this inequality be derived?" depends on which construct is
asking has several provers and a writer can reason about none of them; probes `v25`
and `v26` are the same proof asked twice with opposite verdicts. [ENT-1] 2648. The
last sentence is round 6's: `len(arena)` denoted the post-state in an inventory row,
the entry datum in a user `ensures`, and the call datum at a caller, distinguished
only by an angle-bracket convention that lived in an appendix.

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

**L18. The kernel admits only what wf cannot express, and a removal is priced
against a walked program.** *A rule enters the kernel exactly when no program a
writer can write in wf over the remaining kernel has its effect. A capability a
writer can build is not a rule, a convenience is not a rule, and a table of data is
not a rule: the rule is the sentence that says such a table exists and what it must
contain, and the table is generated data beside it. **A row removed under this law
carries, beside it, the replacement program walked to the standard 3.L.0 states — its
obligations, the rule that discharges each, and the probe where one exists — and the
cost the replacement carries.***
The owner's ruling of 2026-09-03, stated as law so that every rule below can be
checked against it and every removal can name it. Its converse is the obligation 3.L
discharges, and round 5 showed the test has to run in **both** directions. The last
sentence is round 6's, which found `seq_exchange`'s three-statement replacement
carrying a `requires` one unit short of its own body's obligation, so a removal was
argued from a program that does not compile.

### 2.1 The ten notions and their closures

Rounds 3, 4 and 5 produced one finding each and it was the same finding: a notion
was introduced, used by several rules, and closed by none of them. This subsection
names every notion the design has and states its closure property in one sentence.
Round 6 produced two more of the same shape one level out. Three lenses found a notion
with a closure sentence and **no judgment**, and 3.K.11's seventh condition is the
mechanical half of that check. The fourth found something §2.1 itself had not been
built to catch: **the draft checked every new rule against the notions it *uses* and
the v0.41 rules it *amends*, and not against the v0.41 machinery that would have to
carry what it *publishes*.** **publication** is the tenth row and [CALL-7] is the rule
that carries it. **A rule that mentions a notion without respecting its sentence, that
states a fact its own `Judgment:` line does not produce, or that publishes a fact with
no source and no destination, is a defect of this file.**

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
|              | moved, destructured, disposed, or one compiler-derived release; a store's    |
|              | storage reclamation never stands in for its content's release; and the       |
|              | capability a disposal spends is **determined** by the brand rather than      |
|              | written, on the same principle as region elision. **OPEN at one shape**: a   |
|              | run whose element type is linear by declaration is linear, has no capability |
|              | leaf, and is not a nominal, so none of the three routes reaches it (Q13)     |
| accounting   | every covered store is one domain of the map, every edge of the graph        |
|              | carries an entry of that map including the retention entry of an edge that   |
|              | never runs, every entry composes by the same arithmetic at every position,   |
|              | and every acquisition and every release the program performs — a             |
|              | compiler-derived action and a may-suspend release action included — is one   |
|              | of the map's primitive transfers                                            |
| linearity    | a value is linear exactly when it **owns**, at any depth, a value whose      |
|              | release action requires a capability or whose declaration says so; a         |
|              | loan-bearing value owns nothing; and the predicate is discharged only by a   |
|              | move, a destructuring, or a disposal of the whole value                     |
| loan-bearing | a loan-bearing value holds, for its whole life, a loan of its own strength   |
|              | on the range it reaches of every place in its resolved origin set, may       |
|              | occupy no position from which it could outlive or hide that set, and may be  |
|              | the referent of no `&uniq` parameter of a source-declared function          |
| measure data | every measure a program can name is a term with descriptor-storage support,  |
|              | published exactly and completely by every operation that writes its place,   |
|              | killed exactly by an event that writes that storage, and given a datum at    |
|              | **every** event by which a measured value acquires a name                    |
| publication  | every fact a rule publishes names the [ENT-3] source that establishes it,    |
|              | the substitution that instantiates it, the destination it lands on, and the  |
|              | support that keeps it alive; a `Publishes:` line with no source or no        |
|              | destination is the same defect as an `Amends:` line with no row             |
| set commit   | one statement writes places: its right-hand side is evaluated with every     |
|              | target dead, every target is reinitialised at one commit, the targets are    |
|              | pairwise non-overlapping, and a target that names a binding in scope keeps   |
|              | that binding's term                                                         |
| elision      | whether a region is written at a position is decided by the declaration      |
|              | text alone: written where it is minted or otherwise underdetermined by that  |
|              | declaration's own operands, elided where they determine it                   |
```

Where each is carried, and which round-6 finding showed it open:

- **identity** — [PROV-1], and preservation is a consequence of type formation
  rather than a clause. Attacked from every position in six rounds and not moved.
- **activation** — [PROV-1]'s invariant, [PROV-5]'s refusal. Round 6 attacked the
  restated refusal from all three of its named sources and did not move it.
- **release** — [PROV-6], [LIV-1], [STOR-3]'s table. Round 6 found `dispose`'s
  consume half assuming that own-mode implies own-storage, which is false for a view
  (F1 attack 3), and its walk giving a declaration-linear leaf "its ordinary derived
  release action", which a linear type has none of (F1 attack 4). Both are operand
  conditions on `dispose` now, and both are in its `Judgment:` line.
- **accounting** — [RES-5], [RES-8], [RES-10] and 3.K.7.1. Round 5 opened this
  notion and round 6 found it still open in seven places: `retained` composed by a
  clause that contradicted the general one, the loop routes named a level where the
  test is over the backedge delta, the formation rows dropped [OP-9]'s obligation, the
  derived *acquires from* column read one of the two tables that declare a target
  contract, its exclusion test read a runtime-published figure, [RUN-3] deleted two
  denials that are not footprint conflicts, and the cleanup-scratch domain had no
  source site. Each is repaired at its rule and 6.10 lists them.
- **linearity** — [PROV-6]. Round 6's F2 argued the criterion answers the wrong
  question and should be re-keyed to *returns its backing before the store's lifetime
  ends*; **the owner refused that**, and the arena case F2 built it from is
  **accounting** and is repaired in [RES-10]. What round 6 did move is the closure:
  containment reached through a view, so the predicate closes under **ownership**.
- **loan-bearing** — [PROV-3], [BLK-4], [VIEW-4]. Round 6 broke `[VIEW-4]`'s
  length-fixedness (F1 attack 1): it was a claim its own judgment did not make, and
  `[CALL-3]` read it. [VIEW-4] is now the commit refusal alone, [CALL-3] reads what a
  view can **write** rather than what its length is, and [BLK-4] refuses the parameter
  position the attack needed.
- **measure data** — [MSR-1] to [MSR-3], [BLK-0], [CALL-7]. Round 6 found placements
  for three of the six events by which a measured value acquires a name (F4 finding
  2): a move-rebind, an enum payload binder and a destructuring consume's field
  binders each destroyed every measure fact with no source spelling to restore it.
  [MSR-3] has six placements.
- **set commit** — [LIV-2]. New this draft, and it is D2. Three rules with three
  admissions became one; the notion exists so that the next writing form is checked
  against a sentence rather than added beside two others.
- **publication** — [CALL-7], and [BLK-0], [BLK-3], [PROV-2], [PROV-6] and [RES-6]
  read it. This is round 6's open notion and it is the one §2.1 itself did not catch:
  the sixth draft's `Publishes:` lines were **asserted and never constructed**.
  [BLK-0] named an [ENT-3] source `S13` that no rule of the file stated and an "arm
  route above" that referred to nothing, so no kernel row's declared relation ever
  became a fact and every proof route in 3.L and in both worked programs began with a
  step the language did not have; and a **provider's** post-state relation — which
  [RES-6]'s refusal relation, L8's second half and [RES-10] route (ii) all read — was
  not an admissible [FN-9] clause at all and had no destination. [CALL-7] states S13
  once, with its four parts, and every other rule cites it.
- **elision** — 3.K.0, [PROV-1]. Round 6 found the "an elided brand is linear at the
  declaration" sentence scoped to *implicit* parameters, so a nominal's **written**
  region parameter gave one declaration two linearity verdicts (F1 attack 10).
  [PROV-6] states the obligation over the whole region parameter list and 3.K.0's
  sentence is its instance.

---
#### 3.K.0 The region-spelling assumption, and the determination principle

This design rests on one amendment it does not draft — **and which has landed**: a
build in this session rejects a written region name at a position no other position of
its declaration names, citing `[FORM-8] RegionSpelling`, and accepts the fully elided
spelling of the same program (6.1, probes `r1` and `r5`). **Whether a region is written
at a given position is determined by the program text, and the determined spelling
is the only legal one.** That is a change to [FORM-2], [GRAM-2] to [GRAM-5], [FN-2]
and the [OWN] borrow forms, it is uniform over every region position in the
language, and over the type and const argument positions beside them — parameter
lists, borrow annotations, region arguments on types, call-site region, type and const
arguments, and region blocks — and **it lands first, as its own separate and
mechanical spec amendment**. **The scope is stated once, here, and every later
sentence of this file uses it**: round 6 found the sixth draft carrying three
different scopes for one criterion, one in §2.1's notion table, one in this opening
and one in the criterion itself. It is not a rule of this design, it is not
in 3.K's count, and 3.K.11 does not register it. **What those probes also show is that
it changes nothing about D1**: the amended build accepts D1 in its elided spelling, so
R1 had to become a rule [BLK-4] and could not stay a spelling convention.

It is stated here because the container half cannot be written without assuming it.
[FORM-1] 35 admits exactly one spelling per semantic construct. Putting a store's
identity in the type means a region in every type that names a store, unless the
text determines it — in which case *writing* it is a second spelling and the law
says there is only one. So the brand cannot be in the type without that amendment,
and the amendment cannot be brand-specific, because a brand is one more region
argument.

**The criterion is derivation, not repetition.**

> A region, type or const argument is **written** at a position exactly when the
> declaration's own operands do not determine it, and **elided** exactly when they
> do. Written and elided are decided per argument, not per list.
>
> **Two positions are outside the criterion and are always written.** A `construct`'s
> arguments, because a `construct` consults no expected type [TYPE-5] 383-386 and has
> no operands to determine them; and a **declaration's own parameter binders**,
> because a binder is where a name comes into existence and there is nothing for it to
> be determined by. A `region_stmt`'s binder is written exactly when some position of
> its block names it, and elided otherwise, which is why 4.1 writes `region 'a {` and
> 4.2 writes `region {`.

Applied to every spelling in this file, the criterion and the text agree:

```text
| occurrence                                                   | determined by an operand?      | spelling            |
|--------------------------------------------------------------|--------------------------------|---------------------|
| arena_frame<const bytes, const align>['s]()                  | no operands exist              | all three written   |
| seq_fixed<T, const n>()                                      | no operands exist              | both written        |
| seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)        | 's from heap; T from nothing   | seq_heap::<u8>(...) |
| seq_arena<T>['s](arena: &uniq Arena<'s, ...>, count)         | 's from arena; T from nothing  | seq_arena::<u8>(...)|
| seq_slice['r, T](vector: &'r v)                               | both from the borrow operand   | seq_slice(...)       |
| a user fn's own region parameter list                        | supplied by the actuals        | elided at the call  |
| render::<'a>(block: move held, task: &task)                  | 's from the block operand      | render(...)         |
| Some<Task>(value: move ready)                                | a construct: outside the rule  | T written           |
| region 'a { ... arena_frame::<..., 'a>() ... }               | the block's own binder, named  | 'a written          |
| region { let body = seq_slice(vector: &kept.v); ... }        | the block's binder, unnamed    | no name written     |
| struct BlockPool['s] { free: FixedVector<Lease<'s>, 8>; }    | a declaration mints its own    | 's written at both  |
```

The last row is the general shape of a **declaration**: a nominal's or a function's
own region parameter list is where a region name is bound, and a bound name is
written at its binder and at every position of that declaration whose type names
it. Nothing outside the declaration is consulted, which is the property this design
needs.

**This is a principle about determination and not a rule about regions, and one
other position obeys it.** A `dispose` statement spends the capability of every store
its operand's type names at a capability-released leaf, and **that capability is
determined by the brand**: a store region names at most one live store [PROV-1], a
store has exactly one provider [PROV-2], and at any program point at most one live
binding can lend `&uniq` to that provider [OWN-5]. There is nothing for the writer to
choose, so under [FORM-1] there is nothing for the writer to write, and the statement
is `dispose p;`. [PROV-6] states the resolution and its error. **Allocation is the
opposite case and keeps its written provider**: there the writer *chooses* which store
backs the new value, and the brand of the result is created by that choice, so the
operand is the choice and is written. Determination decides both, in opposite
directions, from one sentence.

**Two positions, two candidate sets, and neither is ever empty.**

- At a **stored** position — a field, an enum payload, a run element, a written
  type argument — an elided brand denotes the enclosing nominal's sole region
  parameter when it has exactly one, and otherwise the entry heap's store region.
  When the nominal declares a region parameter it is written, so the elided form
  arises only in a program whose values all come from the entry heap. When the
  nominal has no region parameter and the entry selects no heap, the position is a
  [BLK-4] hard error naming the nominal, not an empty resolution.
- At a **parameter or result** position an elided brand is always an **implicit
  region parameter**, one per occurrence, and never the entry heap. **The entry heap
  has no source spelling at all** — `main` declares no region parameter
  (owner-decided 2026-09-03, [S22]), so its region is the elided default at every
  stored position and is never named — and a signature that must relate two of its
  own positions to one store binds **its own** region name for them, exactly as any
  other helper does. That is what makes a helper declarable in a program with a heap
  and in one with none, by the same sentence: 4.2's `bs_reserve(s: own Bytes, heap:
  &uniq Heap, ...)` writes no region and 4.1's `render['s]` writes one, and neither
  mentions an entry parameter that does not exist.

**A declaration generic over a store has one linearity verdict**, and that sentence
is [PROV-6]'s rather than this section's. The fifth draft's defect was per-instantiation
linearity at an elided brand; the sixth draft's repair was scoped to *implicit*
parameters and round 6 found a nominal's **written** region parameter outside it.
[PROV-6] states the obligation over the whole region parameter list, of which the
elided case is one instance.

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

  The modifier is D1's, the region parameter list on a nominal is [S20], and the run
  is [S1]. `'s` relates the nominal's parameter to its field, and the function's
  parameter to its results, so it is written at every one of those positions and
  supplied by the actual at the call.
- A helper that **hands a run back** relates its parameter's store to its result's,
  so it binds one region name and writes it at its binder and at both positions:
  `fn collect['s](out: own Vector<'s, u8>, source: own slice<u8>) -> (rest: own
  Vector<'s, u8>, written: own u64)`. That is one identifier per helper, written
  once; the **call site** elides it, because the `out` operand determines it, and
  writes `collect(out: move buf, source: move line)`. This is R1's whole spelling
  cost and 4.1 pays it twice.
- A helper whose brand relates nothing writes none and is generic over the store it
  is handed — and by [PROV-6] it may therefore not let such a value reach a scope
  exit, which is what keeps one declaration to one verdict.

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
**fifty-one rules**, five added nominals, thirteen declaration-domain operations plus
four readers, one added statement form and one added `let` alternative, one added
declaration modifier, and 3.K.0's one separate amendment. Every rule answers L18's
question with *no writer can write this in wf* — except one half of [CALL-4], whose
status 3.S records honestly — and 3.L.6 lists the nine that only the partition test
proved. **3.L is the
library**, written in wf against 3.K; it is not part of the language, it is not
blessed, and no rule of 3.K names any of it.

The count moved from fifty by one, and three rules moved. `[LIV-3]` is **retired into
`[LIV-2]`** under D2: one commit rule replaces three writing rules, and the id is not
reused. `[CALL-6]` is **added** and is the rule round 6 found missing: it states the
[ENT-3] source by which a declared relation becomes a fact, which the sixth draft named
in an `Amends:` line and stated nowhere, so no kernel row's relation and no provider's
post-state had a source or a destination. `[CALL-7]` is **added**: it is L15's
completeness obligation quantified over every function that hands a measured value
back, which the sixth draft stated for the thirteen kernel rows alone while every
program in the file depended on it holding for a wf function.

**Every kernel rule states four things — the judgment it creates, the fact it
publishes, what it amends, and its law — plus a `Depends:` line exactly when it
rests on a v0.41 sentence no `Amends:` line in this file changes.** A rule that
creates no judgment writes `*Judgment:* none` and says what it is instead. **Every
fact a rule states appears in its `Judgment:` or `Publishes:` line, and every rule
that reads a fact another rule states names the judgment it comes from** (3.K.11
condition 7). `*History:*` points at the round in 6.5-6.10 that produced the rule's
current shape and carries nothing else. Section 3.K.11 is a **collation of the
`Amends:` and `Depends:` lines and carries nothing else**: it is written last, from
the rules.

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

An **exact** measure is one for which every writing operation publishes a value; a
**bounded** measure is one for which some writing operation can publish only a
two-sided range, because no exact value exists in the source domain. Exactly two
measures are bounded anywhere: an `Arena`'s `len`, whose alignment padding is a
target-stage quantity, and a run's `head` after a front operation, whose new value is
a modular expression the affine domain does not carry. Both are stated once in A.1
and nowhere else.

**A measure is a logical quantity and `head` is the origin of the logical
coordinate system, stated once here because four other rules read it.** A run's
initialized set is the `len` slots beginning at `head` taken modulo `cap` [BLK-1]. A
**logical offset** `i` names the slot at physical offset `(head + i) mod cap`. Every
measure term, every [OP-4] obligation, every [PROV-3] range, every [PAR-2]
disjointness argument and every [RUN-3] footprint is stated in **logical**
coordinates, and the one sentence that carries a logical conclusion to a storage
conclusion is stated here:

> `i |-> (head + i) mod cap` is injective on `[Z, len)` because `len <= cap`, so two
> disjoint logical ranges of one run describe disjoint storage.

Round 6 found the sixth draft leaving the coordinate question to be inferred
separately by [PROV-3] use 3 (storage-keyed), [PROV-3] use 4 (range-keyed) and
[PAR-2] 2005's `a*i + b` refinement, which is three rules reading one unstated
convention.

An admitted place for a measure term is a `place` [GRAM-5] formed with field
selections, `deref` wrappings **and subscripts**, whose final selected type is a
measured type. The subscript admission is the change: `len(table[i])` is a term,
so a run of runs has provable operations.

*Judgment:* the [OP-4] admission above at every subscripted measure place; the
injectivity sentence is a definition and is proved by `len <= cap`, which [MSR-2]
publishes as a standing fact. *Publishes:* the four terms, the logical coordinate
system, and the injectivity sentence [PROV-3] use 4 and [RUN-3] read. *Amends:*
[ENT-2] clause (b) (2681), which today admits `len(P)` only for `array`, `slice` and
`buffer`, and only for subscript-free places; [OP-4] 914, whose obligation gains the
erased-clause attach-site case. *Law:* L12, L15, L16. *History:* 6.10, F2 F6-14;
6.9, F1 attack 4 and the window.

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
event is any [LIV-2] commit, [SET-2] commit, consume, scope exit, or **any action
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
is a convenience for the writer and never a route by which an operation's own
post-state is derived**; [BLK-0] and [CALL-7] require every writer of a measured
place to publish every measure it wrote, which is what puts every backedge inside
`AUTO`'s one-premise family.

**A measure whose value is a compile-time constant or a runtime-profile symbol is
a standing fact with empty support.** A formation operation that publishes
`cap(result) = n` for a written const `n`, and a runtime store whose `cap` is a
profile row, therefore both give a capacity that no event kills for the life of the
term. This is where the fourth draft put a type-level constant and it generalizes to
the one store whose capacity cannot go in a type: [RES-9] already asserts the
sentence for a profile symbol and [MSR-2] is where the kill lives, so it belongs here.

*Judgment:* the kill classification at every [ENT-5] event, which is the judgment
[CALL-1], [CALL-3] and [MSR-3] read. *Publishes:* the implicit facts above, the two
automatic premises, and the standing-fact class. *Amends:* [ENT-2]'s implicit-fact
sentence (2728); [ENT-5]'s support and kill sentences (2863-2896), whose length-term
support becomes the descriptor-storage relation above, whose kill classes (a) through
(d) gain the effect-row statement, and whose clause (a) loses its element-position
carve-out; and [ENT-6] 3007's automatic affine-premise sequence, which gains two
specification-fixed members. *Depends:* [ENT-4] 2860, whose difference-bound
uniqueness argument is why the identity is a premise and not an L0 fact; [ENT-5]
2942-2946, whose "no fact established inside an iteration survives to the next
iteration's head" is what keeps an empty-support fact from crossing a backedge.
*Law:* L15, L16. *History:* 6.8, F1 attack 3 and F2 NB3; 6.9, F1 attack 2.

**[MSR-3] Measure datums, what an atom is keyed by, and one denotation per
position.** A **measure datum** is a compiler-owned immutable [ENT-2] term of
fragment type `u64` with **empty support**: no [ENT-5] event kills it, no place
occurs in it, and no later write retargets it. It is the device [ENT-2] already has
for a `for_stmt` capture and a commit value, extended to more producers. There is
exactly one former, keyed on what a datum denotes rather than on where the value came
from:

```text
a datum is identified by (program point, admitted place P, measure), is
compiler-owned and immutable, and is established equal to <measure>(P) at that
point
```

**Six placements exist, and no seventh. The closure sentence is that a measured
value acquires a name at exactly six kinds of event, and every one of them is a point
at which the function forming the datum can itself read the value.**

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
rebind placement      one `let` or one [LIV-2] `set` whose right-hand side at an
                        ordinal is `move P` for a measured place `P`, read before
                        transfer; the destination at that ordinal is established
                        equal to it
payload placement     an own-place `match`'s arm binders [OWN-13] of measured
                        type, read from the scrutinee's payload before transfer
field placement       a destructuring consume's field binders [S13] of measured
                        type, read from the consumed value before transfer
```

**The last three are round 6's**, and they are the finding with no writer-side
workaround anywhere. `let b = move a;` destroyed every measure fact about a run
(probe `x11` at v0.41 scale), an enum payload binder did the same (probe `x12`), and a
destructuring consume's binders are the same event — so `[S13]`, the statement this
design added so a linear aggregate could be taken apart, handed the writer a run they
could not subscript, and 4.2 threw away everything `bs_reserve` published one line
before the statement this design calls goal A's container half. There is no source
spelling that restates `len(p) = len(chunk.page)` after a destructuring; the datum is
the only channel.

**The fifth draft's exit placement stays withdrawn.** It minted a datum at a call's
post-kill point for each `&uniq` operand, so that a callee could publish what it had
done to a borrowed run; round 5 showed the callee discharging its `ensures` against
the entry datum and the caller establishing it as the exit fact, which is D1. Under
R1 a helper that transforms a run takes it **by value** and returns it, so what the
caller reads is a relation on a *result*, which [CALL-2] transports.

**One denotation per position, stated once (L16).** Round 6 found `len(arena)`
meaning three different things in three rules, distinguished by an angle-bracket
convention that lived in Appendix A.2. The table is the rule:

```text
| the operand occurs in                          | it denotes                                  |
|------------------------------------------------|---------------------------------------------|
| a [BLK-0] or [SYS-2] declared relation, naming  | the operation's own POST-state              |
|   a place the row's `writes` occurrence covers  |                                             |
| a [BLK-0] or [SYS-2] declared relation, naming  | that call's CALL datum                      |
|   any other operand                             |                                             |
| a [FN-8] `requires`, naming a parameter         | that parameter's ENTRY datum                |
| a [FN-9] `ensures`, naming an `own` or shared-  | that parameter's ENTRY datum                |
|   borrow parameter                              |                                             |
| a [FN-9] `ensures`, naming a `&uniq` parameter  | **inadmissible**                            |
| a [FN-9] clause, naming a result binder         | the result itself                           |
| any of the above, read at the CALLER after      | that call's CALL datum for a parameter       |
|   substitution                                  |   operand, and the result for a result       |
|   operand                                       |                                             |
```

**The `&uniq` row is the repair, and it is the one that closed round 6's second
BREAK.** The sixth draft gave a clause operand naming a parameter's measure exactly
one denotation — the entry datum — "in a `requires` and in an `ensures` alike", on
the ground that "a parameter is an input and has one state". That ground is true of an
`own` parameter and false of a `&uniq` one, which the callee mutates by construction;
combined with the datum's defining property that no event kills it, an `ensures`
became a channel that republished an entry fact at a caller's post-kill point. Probe
`e2` is the callee at v0.41, **rejected** by [FN-9]'s entry-image-stability paragraph
(2887-2891), and probe `e3` is the same program with the mutation deleted, accepted;
this rule's `Amends:` line replaces that paragraph, so without the `&uniq` row the
repair would delete the language's own guard. **The caller row is the other half**: a
parameter operand of an established relation substitutes to that call's call datum
and never to a live term, so no relation the caller establishes describes a state the
callee produced at a point the callee cannot name (L11).

**What the `&uniq` row costs is stated rather than discovered.** After [BLK-4]'s
fourth clause the only `&uniq` parameter that reaches a measured place is a
**provider**, so the cost is exactly this: *a user `fn` that lends a provider onward
can publish nothing about that store's post-state*, and its caller's `room(scratch)`
fact dies at the call with nothing to replace it, so every subsequent proved
acquisition in the caller is undischargeable and the caller uses the checked
spelling. [PROV-2]'s own justifying sentence is corrected to say so. The kernel and
system rows are unaffected, because they are declaration records whose relations are
complete over what they write [BLK-0] and are read at the first row of the table.

**One sentence fixes what an [INV-1] affine atom over a measured place is keyed
by, and under D2 it is one sentence rather than three.**

> An [INV-1] affine atom over a measured place is keyed by the [ENT-2] term. **A
> [LIV-2] `set` target that names a binding in scope keeps that binding's term**: the
> statement is a write of the place, the facts over it die by [MSR-2], and the
> right-hand side's declared relations re-establish them on the same term through
> [CALL-4]'s destination clause. A target that resolves to no binding in scope
> introduces one and is a declaration event, exactly as a `let` is, and no invariant
> can name it beforehand.

The sixth draft needed three rulings here — an exchange that is not a declaration
event, a reinitializing `set` that is, and a multi-target `set` whose later targets
are — plus a diagnostic for the invariant the middle case silently orphaned. D2
collapses them: a name in scope is a place, a place keeps its term, and the orphan
case does not arise, so **the orphaned-invariant diagnostic is deleted** and an
invariant that the commit's relations fail to re-establish is the ordinary [INV-1]
failure at the loop head.

*Judgment:* the atom-identity resolution above, at every [INV-1] atom over a
measured place, and the inadmissibility of a `&uniq` parameter's measure in an
`ensures`, a hard error citing MSR-3 at the clause with the restructuring `take the
value by value and relate the result, or state the fact as a requires`. A datum is
formed, never proved. *Publishes:* the datum at each of the six placements, the
denotation table, and the atom-identity rule. *Amends:* [ENT-2]'s term list (a new
clause beside its capture and commit-value clauses); [ENT-5]'s call-boundary
paragraph (2898-2905) and its FN-9 entry-image-stability paragraph (2887-2891), which
are replaced by the datum and the denotation table rather than repaired; [FN-9]'s
`M(c,q)` (1345, a datum operand is always live), its parameter-entry-image sentences
(1316) and its operand admission, which loses a `&uniq` parameter's measure in an
`ensures`; [ENT-6]'s image formation, join and loop-header paragraphs (2976-3002);
[ENT-3.S5] 2774-2782's copy-equality clause, which gains the construct, rebind,
payload and field placements' measured operands; and [INV-1] 3109-3113's atom
resolution, which gains the sentence above. *Depends:* [ENT-2] 2693, whose
one-static-term-per-statement argument is why a per-point datum is sound; [ENT-5]
2942-2946, whose head-state construction is why a body-placed datum does not cross a
backedge; [FN-8] 1275, whose borrow-versus-own actual split the call placement
reuses; [OWN-13] 654, whose own-place match move is the event the payload placement
attaches to. *Law:* L11, L16. *History:* 6.10, F1 attack 2 and attack 8, F4 findings
2 and 5; 6.9, F1 attack 1.

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
relations, [INV-1] invariant targets, the operation-domain obligations of [BLK-0],
and [RES-10]'s backedge-delta test. **The per-family route lists retire.**

**This rule is not widened.** F1's fourth attack in round 5 showed the design's own
`spare` backedge needing three published affine premises where step 5 admits two.
Widening `AUTO` to three would change the prover's complexity class and destroy
[ENT-6] 3026's promise that "an author can determine from this rule alone whether a
target is automatic". The defect was in what the writers of a measured place publish,
not in what the prover accepts, and probes `g3` and `g4` locate it exactly: the
identical three-term header invariant is provable when the body publishes one exact
relation and is not when it publishes none. [BLK-0] and [CALL-7] are where that is
repaired.

*Judgment:* the disposition itself, at every goal every consumer above submits.
*Publishes:* the disposition of every numeric goal. *Amends:* [ENT-6] 3040, 3047,
3075 and 3084, the four per-family route and attach-site grants, which keep their
normalization and lose their route grant, and [FN-9]'s `prove_ordering` route, whose
undocumented direct-affine branch becomes one of the six steps. *Note:* this rule is
why the design does not have to be revisited when the library adds an operation: an
operation adds a goal, never a route. *Law:* L16. *History:* 6.9, F1 attack 4; 6.5,
F4-3.

**[MSR-5] The contract clause is the relation an invariant already is, over a wider
operand set.** **[S17]** A `requires`, `ensures`, `header_invariant`,
`invariant_stmt` or `proof_use` operand is a **term** of the [ENT-2] term language,
not an `atom` of [GRAM-5].

**v0.41 does half of this rule's work and it is smaller for it.** A contract clause's
root is already one `compare_op` over two `expr`s ([FN-9] 1312), and a
`header_invariant` is already `affine_expr compare_op affine_expr` ([GRAM-4] 239).
What is left is the **operand set**: [GRAM-5] 269's `atom` has no `call`
alternative, so `len(source) <= room(out)` derives nowhere, and probe `e1` is that
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
contract clause admits all six, exactly as [FN-9] 1312 already does — which is what
lets [CALL-7]'s completeness clauses state an exact relation in one clause where a
header invariant costs two (Q14).

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
verbatim. *Verified today:* probes `e1`, `t11`, `w3`, `x5`, `q1`, `q9`, `r1_lenatom`
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
in 3.L and `CONTAINERS.md` reads its bound in one of them, so without this rule the
library is not merely inconvenient but unparseable — and 3.L.6 records it as one of
the eight.

Nothing about term identity, support or kills changes: a const generic's term has
empty support and no event kills it, exactly as a named const's does, so the
diagnostic domain gains one member and the proof domain gains nothing. **And it
introduces no shadowing hazard**, which round 6 checked and could not break: probe
`c3` shows a const generic is already a member of the `LexicalIdentifier` declaration
domain with class `ConstGeneric`, so `let n = 3_u64;` inside `fn build<T, const n:
u64>` is a `[TYPE-6] DeclarationCollision` today and no program has one spelling
denoting both.

*Judgment:* the ordinary [TYPE-6] resolution over the widened admission, and the
ordinary [TYPE-5] type check at each use. *Publishes:* the const generic as a
`pbase`. *Amends:* [TYPE-6] 401 (`pbase`'s admitted declaration classes), [ENT-2]
2685-2687's endpoint admission, and [MSR-5]'s `clause_operand` through
`ent2_place`. *Depends:* [ENT-2] 2681 clause (c), which already makes an
integer-typed const generic a symbolic constant term, and which is why this rule
adds a spelling and not a fact source. *Verified today:* `t1`, `t2` and `t3`
rejected, `t4` accepted; `c3` is the collision that makes shadowing impossible.
*Law:* L16, L18. *History:* 6.10, F1 attack 14 HOLDS; 6.9, F4 blocking 1.

#### 3.K.2 `[PROV]`: stores, brand, activation, and release

**[PROV-1] A store's identity is a region, the region is in the type, and a region
names at most one live store at any program point.** This is the rule the design is
built around, and everything else in this family is derived from it.

A **store region** is a region that names one store. A region becomes one by being
named as the store argument of a reserving occurrence [PROV-5], or, for the heap,
by being minted for the entry heap before `main`. **There is no third way**, which
is a checkable sentence rather than an assumption: the fifth draft's `seq_frame`
produced a `Vector<'s, T>` whose `'s` no reserving occurrence named, so this rule's
invariant had nothing to quantify over, [PROV-6]'s predicate classified it by no
clause, and [RES-1] and Appendix A.2 gave it two different envelope items. The row is
deleted (3.K.3) and the sentence is true again.

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
and it is the whole of what R2 makes derived rather than declared. **The third column
is also what determines the capability a `dispose` spends**, because one store has
one provider [PROV-2] and a value's brand names the store: the writer chooses a store
when the value is *made* and has no choice when it is *released*, which is why
`seq_heap` writes its provider and `dispose` does not (3.K.0).

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
multi-return, a join, a value-in / value-out result, an argument transfer and a
return, in one sentence, and every future step for the same reason. Two values have
the same store exactly when their types name the same region, which [OWN-12] 650 and
[TYPE-5] 379 decide by exact identity. All six falsifier rounds attacked this from
every position they could build and none moved it; 6.8 through 6.10 record the routes.

**The brand's spelling is 3.K.0's assumption, and this rule adds nothing to it.**
What this rule owes that amendment is the **candidate set at each position**, which
3.K.0 states, and the sentence that makes it non-empty by construction: at a stored
position the set is the enclosing nominal's own region parameters plus the entry
heap's store region when the entry selects one, and at a parameter or result
position an elided brand is an implicit region parameter. That is read from the
nominal's own declaration and from the entry's one input row, never from a callee,
a caller, or an instantiation, which is what makes the spelling decidable from the
declaration text alone.

**The provider parameter of an allocating operation is never elided.** `heap: &uniq
Heap` keeps its parameter, its mode and its effect row, because that is the
signature-visible allocation fact L2 exists to create; what goes is the region
*inside* the type and the region of the borrow, never the parameter. A signature that
allocates still says so at its parameter list and at its `allocates` row, and
[PROV-4]'s reachability closure reads exactly those. `struct Bytes { v: Vector<u8>; }`
is then an ordinary nominal with no region parameter, and 3.L.5 writes
`byte_string.wf`'s join with and without the brand so the difference is visible rather
than argued.

`Heap<'s>` is delivered as an `own` entry parameter and lives for the program.
The `command` standard-input table [FN-7] gains ordinal 5:

```text
| ordinal | label             | written mode and type | supplied value                                 |
|---------|-------------------|-----------------------|------------------------------------------------|
| 5       | command.heap [S22]| own Heap              | the one general store the runtime minted first |
```

and **`main` declares no region parameter** (owner-decided 2026-09-03): [FN-7] 1218's
existing sentence is kept, the entry heap's region is the elided default at every
stored position (3.K.0), and no signature ever names it. Program start supplies the
store and it outlives every region of the program. The `Heap` `main` receives is
dropped on the return edge with the **empty** release row: the store is the
runtime's, the program returns the handle, and no covered acquisition or release
happens there.

*Judgment:* one live store per store region, established by [PROV-5]; the
`SecondStoreInOneRegion` hard error above at a second reserving occurrence naming one
region; the ordinary [FN-7] label, order, mode and type checks; and the exact-identity
type equality [OWN-12] 650 and [TYPE-5] 379 already perform, which is the judgment
[PROV-6]'s provider resolution, [BLK-4]'s confinement check and [RES-5]'s domain
identity all read. *Publishes:* each value's store, as a component of its type; the
store's measures; the store-to-provider map [PROV-6] resolves against; and the
whole-program fact `heap-unreachable` when the entry row is absent. *Amends:* [TYPE-2]
357, which gains the five branded and container nominals below and from which
`box<T>`, `arena<'r, T>` and `buffer<T>` retire from the writer surface; [TYPE-7] 476,
whose closed deref domain becomes `&'r T` and `&uniq 'r T` alone, because a single
stored value is a run of capacity one and is reached by subscript; [GRAM-3] 207-210,
whose fixed `box`, `arena` and `buffer` type productions retire in favour of ordinary
TYPEIDs with `targs`, whose `slice` production is joined by `mut_slice` [VIEW-1], and
which gains the omitted-store-region form; [OWN-10]
641-645, whose `arena<'r, T>` content clause becomes a clause over `Vector<'s, T>`
content with `'s` in the subject position; [FN-7]'s table (1227-1233), whose "declares
no region parameters" sentence (1218) is **kept**, its canonical five-input byte sequence
(1245-1246), and its effect-row sentence (1220), whose `allocates(heap)` becomes
`allocates` over the entry's own labelled provider input. *Depends:* [OWN-3] 578 and
580, for uniqueness within a function and incomparability across the boundary.
*Law:* L2, L13, L16. *History:* 6.9, F1 attack 5 and F3 defect 13; 6.8, F1 attack 1.

**[PROV-2] Unforgeable, uncopyable, taken as a loan, and never stored.** No source
construct produces a provider; a `Heap<'s>` exists only because the runtime minted
exactly one before `main`, and an `Arena` only as the result of a reserving
operation [PROV-5]. No operation duplicates, reconstructs, compares, serializes, or
derives a provider from a non-provider value.

An operation that **allocates** from a store takes that store's provider as a written
`&uniq 'b` parameter and exhibits it. A provider is never passed `own`: it is confined
to its own store region, and a moved provider strands its own store. The one `own`
provider in the language is the `Heap` the entry receives. **A release does not take a
provider parameter and could not choose one**: the store is determined by the
released value's brand and the provider by the store, so `dispose` resolves rather
than takes (3.K.0, [PROV-6]).

**A provider parameter is the one `&uniq` [BLK-4] does not refuse**, and the reason
is that it is not a container: no operation changes a provider's *identity*, only its
measures. **What a caller may therefore keep across such a call is exactly what the
callee's declaration publishes, and for a user `fn` that is nothing** — [MSR-3]'s
denotation table makes a `&uniq` parameter's measure inadmissible in an `ensures`, so
a helper that lends a provider onward publishes no post-state for it. The sixth draft
wrote the opposite ("a caller keeping a fact about a provider across a call is keeping
a fact the callee's own declared relations publish"), and round 6 found there are no
such relations for a user `fn`. The corrected sentence is: **a caller's fact about a
lent provider dies at the call, and the caller uses the checked spelling afterwards**;
only the compiler-owned rows of [BLK-0] and [SYS-2], whose relations are complete over
what they write, hand a provider's post-state back. 5.1's Q17 records the cost.

*Judgment:* a `construct` [GRAM-8] naming a provider or container nominal, and
every other source route to one, is a hard error citing PROV-2 at the complete
`construct`, with the restructuring `receive the provider as a parameter, or
reserve one with arena_frame`; a provider type in a stored position is a hard error
citing PROV-2 at the complete contained `type`, with the restructuring `lend the
provider to the operation that needs it; a provider is never stored`; and an
allocation call whose provider argument is missing, is not a provider place, or is
not writable is a hard error citing PROV-2 at the `call`. *Publishes:* uniqueness of
the `Heap`; the one-provider-per-store map [PROV-6]'s resolution reads; and the
store's post-state measures, which are [BLK-0] declared relations over the call's own
datums [MSR-3], stated single-state. *Amends:* [OP-1] 798-803, from which `box_new`
and `arena_new` retire, and [STOR-2] 685, which defined them; [STOR-5] 723-737, whose
enumerated stored-content positions gain the provider prohibition. *Depends:*
[OWN-10] 641, which is why `'s` and `'b` are always distinct; [OWN-6] 614, which
makes an argument borrow a call-scoped temporary, the fact probe `w8` exercises and
the reason store identity may not rest on what stands at a place between two calls.
*Law:* L2, L3, L4, L13, L16. *History:* 6.10, F1 attack 8; 6.8, F1 attack 7.

**[PROV-3] Provenance is for loans, a loan reaches a logical range, and a
loan-bearing value owns nothing.** [OWN-5]'s finite origin set, today defined for
`slice<'r, T>`, generalizes to the two views and to nothing else. A **loan-bearing**
type is `slice<'r,T>` or `mut_slice<'r,T>`; a value of one carries a finite set of
origins, each an origin place paired with the half-open **logical** index range the
value reaches of it [MSR-1].

Formation makes a **singleton**: `seq_mut_slice(vector: &uniq table[i])` has the
singleton origin `table[i]` with range `[Z, len(table[i]))`. A named const maps to
the distinguished `immutable-const` origin. Binding, moving, **copying**, passing and
returning preserve the set and its ranges; a control-flow join takes the union; a parameter of
loan-bearing type starts with the singleton containing its own formal origin,
substituted at a call boundary by exactly the rule [FN-1] 1041-1047 already applies
to the origin place. The **resolved** origin set is the set minus `immutable-const`,
which creates no conflicting access and has no writable storage [OWN-5] 607,
[OWN-7] 632.

**A loan-bearing value owns nothing** (L10). What it reaches belongs to its origin,
so no obligation of what it reaches — a release action, a linear obligation, a
disposal — is ever a property of the view. [PROV-6] reads that sentence twice, and it
is also why a `slice<'r, T>` can be **copy** (owner-decided 2026-09-03, [VIEW-1]): a
value that owns nothing has nothing a second copy could double-free.

**A copy view's loan is per value and shared loans do not conflict.** Each copy of a
`slice<'r, T>` carries the same origin set and holds **its own** loan of shared
strength on the same ranges; [OWN-5] admits any number of shared accesses to one
range, so two copies are two shared loans and neither denies the other. The loan of a
copy view begins where that value is formed or copied and ends where that value is
dead, so the origin is frozen for exactly as long as some copy of the view is live —
which is the same freeze one affine view gave, with the re-formation removed.
A `mut_slice<'r, T>` stays **affine**, because two exclusive loans on one range are
exactly what [OWN-5] 606 refuses, so its loan can only be moved and never duplicated.

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
   statement may write, replace, exchange, **or consume** the storage any resolved
   origin of that set describes. This clause is **storage-keyed and says nothing
   about the view descriptor**; [VIEW-4] is the rule that governs a commit at a
   loan-bearing place. The consume is round 6's wording repair: the route was already
   closed by use 1 and [OWN-5], and naming it here closes the notion where §2.1 says
   the notion is closed rather than one rule away.
4. **Disjointness.** [OWN-7] 630's overlap test extends to logical ranges: two
   origins with the same resolved place overlap exactly when their ranges intersect,
   judged by the same affine reasoning [PAR-2] 2005 already performs for a
   single-binder element write, and carried to a storage conclusion by [MSR-1]'s
   injectivity sentence.

*Judgment:* a loan-bearing value in a prohibited position [BLK-4] is a hard error
there; a write to, or consume of, the storage a live resolved origin describes is the
ordinary [OWN-5] conflict, at the write, naming the loan; a write to a binding a live
loan's address computation reads is the same conflict; and the range-overlap test of
use 4, which is the judgment [RUN-3] and [PAR-2] read. *Publishes:* the origin set,
the resolved origin set, each origin's logical range, and the sentence that a
loan-bearing value owns nothing, which [PROV-6]'s closure and [PROV-6]'s `dispose`
domain both read. *Amends:* [OWN-5] 594-611, whose slice-origin paragraphs generalize
to loan-bearing values and gain the copy clause above, whose one access clause becomes the two of use 1 over ranges,
which gains the address-computation and resolved-set sentences, and whose 608 becomes
"a formal view origin has a writable storage path inside its callee exactly when that
view's loan strength on its resolved origin set is exclusive"; 601-604's
no-slice-valued-join sentence, restated over the loan-bearing predicate rather than
over one retired type name, because the union of two loans is not a loan any rule can
end at one consume; [OWN-7] 630, which gains the range clause; [SET-1] 488-490, whose
"no writable target path may traverse a `slice<'r, U>` value" is restated as *a target
path may traverse a view value exactly when that view's loan strength on its resolved
origin set is exclusive*, which is what admits the `mut_slice` element write probe `p7`
is refused today; [SET-2] 513-529, whose region-bearing target rejection is replaced
by use 3 and [VIEW-4]; [EFF-1] 1386, whose "for a direct `slice<'r, T>` parameter, [an
effect path] names the viewed backing state rather than the descriptor" generalizes to
a loan-bearing parameter, which is the declaration-side half [CALL-3] and [VIEW-7]
both read; and [EFF-2] 1406-1410, whose slice-parameter projection generalizes the
same way. *Depends:* [FN-1] 1041-1047, whose call-boundary origin substitution is what
carries an origin into a callee and back; [OWN-7] 630, whose conservative subscript
overlap is what makes use 2 checkable. *Law:* L10, L12. *History:* 6.10, F2 F6-14 and
F1 attack 13; 6.9, F3 defect 4 and I26.

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

**The same closure computes reachability through a type**, and [BLK-4]'s fourth
clause is its second consumer: *does this type reach, through fields, enum payloads,
run elements and written type arguments, a value of a given class?* One closure, two
readers, and the one-field-wrapper defeat that killed [CNT-7] in round 4 is closed by
construction.

**An allocating row names the same provider path in all three categories, written in
[EFF-1] 1369's canonical order `reads(p), writes(p), allocates(p)`**, and A.2 writes
all three on every acquiring row in that order. An allocator observes its prior state
while changing it, which is exactly the both-categories case [EFF-1] 1389 already
states. The order is [FORM-1]'s one legal byte sequence and not a preference; the
sixth draft wrote `reads, allocates, writes` in its own prose and in six rows of A.2
and `CONTAINERS.md` while writing the canonical order in §4, so one of the two
spellings was a hard error, and 3.L.0 now carries the order beside its other
discipline sentences. **A `dispose` exhibits `writes` of the resolved provider place
and no `allocates`**, because it spends the store's capability without acquiring from
it; [PROV-6] states the row and [RUN-3] reads it. Probe `t10` is [EFF-2]'s both-ways
check firing on the smaller half of the same mistake.

*Judgment:* [EFF-2]'s both-ways row check, unchanged, which is the judgment [PROV-6],
[RES-4] and [RUN-3] read. *Publishes:* the provider-reachability closure; the
type-reachability closure [BLK-4] reads; and the heap-reaching path, which is the
ordered call chain from `main` to the allocation that [RES-4] prints. *Amends:*
[EFF-1]'s `effect` production (1369-1378), retiring the effect-row atoms `heap` and
`arena`; and [FN-3] 1123-1127, whose conformance effect-row normalization is defined
over "the allocation set whose members are `heap` and each alpha-mapped `arena`
region" and which becomes the set of `allocates` paths under the same
parameter-ordinal and field-ordinal identity 1127 already fixes for `reads` and
`writes`, with the region alpha-mapping applying to modes and types only. *Depends:*
[PROG-1] 1492, whose one closed compilation unit with no function values is why the
closure is exact. *Law:* L2. *History:* 6.10, [BLK-4]'s second reader; 6.9, F3 defect
6; 6.7, F3-12.

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
that context's `stack` item of `E` — **and the item carries the alignment the
occurrence wrote** [RES-2], which is round 6's repair of a place where L6's own
sentence and this rule disagreed. The `extent` form produces its own
`region(name, bytes, alignment, contiguous)` item of `E`, whose name is derived from
the reserving occurrence and is not written. **On every edge leaving `'s`'s block the
store's release action resets it to its initial state**: the bump cursor to zero, and
nothing else. That action joins [STOR-3]'s release-action table, and [RES-10]'s
`reset` transfer is its arithmetic.

**The refusal is stated over the property, and three sources are named because each
is decidable from declared data.**

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
activation; reading it before would refuse a program the lowering makes safe. Round 6
attacked all three sources and moved none.

*Judgment:* the ordinary region, confinement and [OWN-5] exclusivity judgments,
plus the region-locality check, [PROV-1]'s one-store-per-region check, and the
activation refusal above, each a hard error citing PROV-5 at the `targ` with the
restructuring stated there. *Publishes:* the reserved store's measures, its store
region, its envelope item — one `stack` contribution carrying the written alignment,
or one `region` item — and the one-live-store-per-region invariant [PROV-1] reads.
*Amends:* [STOR-3] 688-720, whose release-action table gains the store reset.
*Depends:* [ERR-4] 1487, whose "absence of a complete permission derivation ... never
rejects the source" is why the `par` source is read as *may execute with overlapping
execution* rather than as a permission that was taken. *Law:* L2, L5, L6, L13.
*History:* 6.10, F2 F6-9; 6.9, F1 finding 12 and F2 F5-10.

**[PROV-6] Linearity is the reclamation half of affine, closed under ownership, and
disposal, destructuring and the partial-consume refusal are one closure.**
**Owner-decided 2026-09-03**: the criterion and the `linear` modifier are both
kernel, and D1 settles them together.

**The criterion, stated once.** A type is **linear** exactly when

```text
its own release action requires a capability,
  or its declaration carries the `linear` modifier,
  or it OWNS, at any depth, a type that is linear by either clause
```

and it is **affine** otherwise. A type **owns** its fields, its enum payloads, the
elements of a run it is, and the values a written type argument of it stands for. **A
loan-bearing type owns nothing** [PROV-3, L10]: what a view reaches belongs to its
origin, so no obligation of what it reaches is a property of the view.

That last sentence is round 6's third BREAK. Under closure by *containment* over type
syntax, `slice<'r, Vector<u8>>` is linear because it mentions a linear type; `dispose`
is then admitted on it, its consume half is satisfied because a view is an `own`
binding, and its walk frees eight of a caller's runs through a **shared** loan. Closure
under *ownership* is the exact repair: a view of runs is affine with an empty derived
release, and `[BLK-1]`'s "a run over a linear `T` is itself linear" stays true because a
run does own its elements. The other reading is no better: read containment as *not*
reaching through a view and nothing says so, and the dual failure arrives with no
program at all — a linear view has no route out, so every view over a run of heap-backed
runs would be undeclarable.

`Vector<'s, T>` at a `Heap` region is linear because freeing its run needs the
`Heap`, which is a value [L2] and which an implicit scope-exit release would have to
smuggle. `Vector<'s, T>` at an `Arena` region is affine because its region's reset
needs nothing. `FixedVector<T, n>` is affine because a frame needs nothing. A
compiler-owned system resource is affine because its release is the runtime's, and
[RES-9] states why reclassifying `ReadFile` was considered and refused.

**The criterion answers the question the owner asked it.** Round 6 proposed re-keying
it to *does releasing this value return its backing before its store's lifetime ends*,
on the strength of an arena take in a service loop that never gets its bytes back.
**The owner refused the re-keying**: the criterion is what a release *needs*, and
frame- and region-reclaimed values are affine. That program is real and it is an
**accounting** failure — [RES-10] charges such a take trips × size rather than
certifying it bounded at the store's capacity.

**Linear is a property of an affine type and not a third class.** [OWN-1] 563-564's
classification is unchanged; every linear type is affine; the linear predicate is a
further property of an affine type, defined here, that removes its
compiler-derived release action and fixes its scope-exit disposition. That one
sentence is what lets `move p`, [OWN-13] 654's own-place match move, [SET-2] 516's
affine target requirement and [ERR-3]'s `propagate` operand reach a linear value at
all.

**The `linear` modifier, what it is for, and what it buys.** `linear struct N { ... }`
and `linear enum N { ... }` are one added modifier on [GRAM-2]'s `struct_decl` and
`enum_decl`. It exists **only for a logical obligation**, because the storage
obligation is already derived: no writer marks a store-derived type, and a writer who
marks one has written a modifier the criterion already implied. What the modifier says
is *silently dropping this value is a bug*, and the values that pass that test are the
ones the criterion cannot see — a transaction that must commit or roll back, a request
that must be answered, a builder that must be finished. 3.L.7 is the writer guideline.

**What it buys is must-consume, visibly — and not must-return**, and round 6 is right
that the sixth draft implied the second. A linear value's three routes include
`destructure whole`, which for a nominal whose fields are affine legally throws the
contents away in one visible statement. That is the correct semantics for a
transaction and for a request, whose honour path *is* a destructuring inside the code
that owns them. It is not, by itself, the property a pool's lease needs, and the honest
statement is that **a directional obligation is bought by proving the return, not by
marking the type**: a library release whose refusal a caller can discard is the
checked spelling, and the proved spelling — a total release under a `requires` the
caller discharges — leaves the value exactly one route on every path. 3.L.7 states the
rule and 4.1 is written on the proved spelling for exactly this reason.

This is the one thing in this family that no wf program can have. A writer can write a
pool; a writer cannot write *a type whose silent drop is refused*, because every wf
mechanism for it is a runtime field a program can forget to read.

A linear value has **no compiler-derived release**. It leaves a scope by exactly
three routes, and the three are closed under ownership together:

```wf-design
let tail = move queue;
let Chunk(page: page, used: used) = move chunk;
dispose table;
```

— moved out whole, destructured whole [S13], and disposed to the store its type
names [S12].

An own-place `match` [OWN-13] is a destructuring: it consumes the scrutinee and
binds each payload as `own`, so the obligation passes to the binders exactly as the
`let` form's does. That is why an `Option<Lease>` cannot be dropped and must be
matched.

**Destructure whole.** `let N(f1: b1, ..., fk: bk) = move v;` **[S13]** is one added
`let_stmt` alternative that consumes a value of nominal type `N` and binds every
field of `N` in declaration order to a fresh IDENT, judged exactly as [CALL-4]'s
multi-result destructuring `let` is: each binder is an independent destination,
each receives its field's declared type and `own` mode, each measured binder receives
a **field placement** datum [MSR-3], and no residual exists for any rule to define. It
is the inverse of `construct`, and it is what makes "linearity is closed under
ownership" true of disassembly as well as of assembly.

**Dispose is a consume and a write, and it names no capability.** `dispose p;`
**[S12]** is one added statement form. **The capability it spends is determined and is
therefore not written** (3.K.0): a value's brand names its store [PROV-1], a store has
exactly one provider [PROV-2], and at any program point at most one live binding can
lend `&uniq` to that provider [OWN-5], so there is nothing for the writer to choose
and under [FORM-1] nothing to write. Identity and permission are different things and
the brand supplies both: it says *which* store, and the store says *which* capability.

> **Resolution.** For each capability-released leaf reached by the walk of `p`'s type,
> let `'s` be the store region that leaf's type names and `P('s)` the provider type of
> `'s`'s store. The statement resolves the **innermost live binding of this function
> whose type is `P('s)`**, reached directly or through a borrow, and **writes** it. It
> is a hard error citing PROV-6 at the statement when no such binding is in scope, or
> when the only one in scope is reached through a **shared** borrow, with the
> restructuring `take the store's provider as a &uniq parameter of this function` and
> the missing parameter rendered.

The statement's effect row therefore carries `writes` of each resolved provider place
beside `writes` of `p`'s own ultimate storage origin, so a release is still a
signature-visible spend of a held capability: `allocates(heap)` and `writes(heap)`
appear on exactly the functions that reach a store, [PROV-4]'s closure is unchanged,
and [RUN-3] sees two overlapped disposals from one store conflict. **Nothing about L2
is weakened. What is removed is a redundant spelling, not a permission.**

**Allocation keeps its written provider and this is not an inconsistency.** At an
allocation the writer *chooses* which store backs the new value and the result's brand
is created by that choice, so the provider is the choice and is written:
`seq_heap::<u8>(heap: &uniq heap, count: 0_u64)`. At a release there is no choice, so
there is nothing to write. One determination principle, two opposite answers, and
3.K.0 states it once.

The statement is, of `p`:

- **one consuming use** [OWN-1] of `p`'s root, so `p` must be a place rooted in a live
  own-mode binding *of this function* **whose type is not loan-bearing**, and whose
  walk **traverses no loan-bearing value**; the whole binding is dead afterwards, and
  a `dispose` of a proper sub-place is a partial consume; and
- **one write of `p`'s ultimate storage origin**, exhibited in the statement's
  effect row beside the write of each resolved provider place, so [EFF-2] projects it,
  [MSR-2] kills over it, [CALL-1] to [CALL-3] classify it, and [PAR-1] 1975's
  footprint contains it.

**The two loan-bearing conditions are round 6's, and they are not redundant.** The
first is the linearity closure read at the operand: an own-mode binding of
loan-bearing type is a value that *owns nothing*, so it has nothing to dispose. The
second is `dispose`'s own operand domain: a value may own a view, and the walk must
not follow it into storage the disposer does not own. Round 5 showed what happens when
one rule is asked to carry two notions, and these are two.

**And a `dispose` may not discharge a declaration-linear obligation.** The walk has no
action for a leaf that is linear by the modifier — that is the whole content of *a
linear value has no compiler-derived release* — so the sixth draft's walk silently did
nothing for it, which is verbatim the failure the modifier exists to prevent. The
admission condition is stated where the admission is:

> `dispose p;` is admitted only when `p`'s type reaches at least one leaf whose
> release requires a capability, and **no leaf of `p`'s type is linear by the modifier
> or owns one**. A type reaching both is disposed only after the declaration-linear
> part is taken out by a destructuring consume, which is the statement's mechanical
> fix.

Adding a walk row that "runs the modifier's obligation" would be the wrong repair:
there is no such action to run, and inventing one would make the modifier a destructor
hook, which this rule's own closure sentence forbids.

**Its judgment is a walk of `p`'s type**, stated over the type's variant structure
rather than over a flat leaf set:

```text
for a struct or a run element type: every field in [STOR-3]'s order
for an enum:                        the active variant's payload, selected by the discriminant
for a run:                          every element of the initialized window, in ascending logical order
at a capability-released leaf:      release to the store its own type names, spending the
                                      resolved provider of that store
at every other leaf:                that leaf's ordinary derived release action
at a loan-bearing value:            unreachable; the operand condition above refuses it
```

**A container's elements are visited before its backing is released**, so `dispose` on
a full container is legal and needs no emptiness premise — and a `bs_shrink` that
disposes a run still holding elements is correct for the same reason, which
`CONTAINERS.md` §3.3 now states because a writer reading "drain then dispose" will
assume otherwise.

**The walk's depth is the disposed type's containment height, a compile-time
constant, and the walk therefore uses no auxiliary storage.** A type whose graph has a
**cycle** through that walk has no compile-time height, and this draft refuses it **at
the type, in every program**, rather than denying a resource premise only a marked
entry checks. The refusal is stated over **the graph the walk follows**, which is
round 6's correction:

> A type is a hard error citing PROV-6 at its `struct_decl` or `enum_decl` when the
> sub-graph of its ownership graph reached **through leaves whose release requires a
> capability** has a cycle. The diagnostic names the cycle, with the restructuring
> `hold the cells in a run and link by index`.

The sixth draft stated it over **containment**, which refuses every recursive
structure in every program — including an arena- or frame-backed one whose walk is
empty, because an arena-backed run has no per-value release action at all — and
`tests/programs/recursive_tree.wf` is in the corpus today. A heap-backed cycle is
refused in every program, which is what L3's no-abort clause needs: premise 3 of
[RES-3] is a hard error only under [RES-4]'s marker, so a rule that is a hard error
under a marker and a process abort without one is not one rule. Probe `a8` shows the
aborting walk with its `realloc`'d worklist today and probe `x6` shows the type
accepted.

**A partial consume of a value of linear type is a hard error.** [OWN-1] 569's
"after any consuming use, the whole binding rooting `p` is dead (partial moves kill
the whole binding)" is the one event that makes a linear binding *not live* without
discharging it, and both [LIV-1]'s check and this rule's own error are stated over
live bindings, so the abandoned sibling leaves its scope by none of the three routes
and no rule sees it. The refusal is stated over the **consume** and not over `move`,
which is why it reaches `dispose chunk.page` as well as `move chunk.page`; probes
`x4`, `g7` and `p6_partial` show the `move` shape accepted today and the last shows
the residual being freed by a derived drop.

**A declaration generic over a store has one linearity verdict, and the obligation is
stated at the declaration over the whole region parameter list.** Round 6 found the
sixth draft's sentence scoped to *implicit* region parameters, so a nominal's
**written** one escaped it: `struct Holder['s] { run: Vector<'s, u8>; }` is linear at
a heap `'s` and affine at an arena `'s`, and `fn discard['s](h: own Holder<'s>) ->
done: own u64` is accepted at one instantiation and refused at the other, chosen by a
caller who writes nothing that distinguishes them.

> A function that declares a region parameter `'s` — written or implicit — may not
> let a value whose type names `'s` reach a scope exit. It must move it out by a
> result, destructure it, or resolve a provider for it, which requires a parameter of
> that store's provider type and therefore writes `'s`. The check is at the
> declaration, over the body, once; a hard error citing PROV-6 at the `fn_decl` names
> the region, the binding and the missing provider parameter.

That costs nothing at an arena instantiation, because moving out by a result is what
such a helper does anyway, and it makes 3.K.0's elided-brand sentence one instance of
a rule rather than a patch for one axis.

`propagate` and a live linear binding are mutually exclusive, and this rule says
so rather than leaving it to be discovered. A `propagate` error edge leaves every
enclosing scope and offers no statement position on which to discharge, so a
`propagate` in a function holding a live linear binding is a hard error citing
PROV-6 at the `propagate_let_rhs`, with the restructuring `expand the propagate
into a match and dispose on the Err arm`. Probes `w6` and `w7` measure what that
restructuring costs — eleven lines at indent two become forty-one at indent
twenty-two for five error exits, forty-six with the disposals — and `a2` compiles the
shape the refusal removes. **That cost is larger than the sixth draft recorded**, it
lands on `tests/programs/raw_deflate_dynamic_decode.wf`'s seven-site 214-line
`decode_dynamic` with three live heap runs, and 3.S [S28] proposes the relief with
5.1's Q10 putting the choice to the owner.

**One consequence of the criterion, stated because a writer meets it on day one.**
Every heap-derived value in a hosted program is disposed explicitly. 3.L.5 counts
seven such statements in `byte_string.wf`. The way to write fewer is a region block or
an arena, whose values are affine; the way to write none is goal A.

*Judgment:* the linearity predicate itself, computed from the type by the criterion
above, which is the judgment [LIV-1], [BLK-1], [STK-1] and [RES-10] read; a linear
binding live on any edge leaving its scope, including a `propagate` error edge and a
function-return edge, is a hard error citing PROV-6 at that edge, naming the binding,
its store regions, and the provider a `dispose` would resolve; a partial consume of a
value of linear type is a hard error citing PROV-6 at that consume, with the
restructuring `destructure the whole value with let N(f: a, ...) = move v;, or dispose
it whole`; a `dispose` whose operand is not rooted in a live own-mode binding of this
function, whose root type is loan-bearing, whose walk traverses a loan-bearing value,
whose type reaches no capability-released leaf, or whose type reaches a
declaration-linear leaf, is a hard error citing PROV-6 at the statement; the provider
resolution above and its two failures; the declaration-site region-parameter
obligation above; and the cyclic-graph declaration error above. *Publishes:* the
linear predicate; the release events; each store's post-state measure; the statement's
write of `p` and of each resolved provider; and the walk's effect contribution, which
[RES-10] charges and [RUN-3] reads. *Amends:* [STOR-3] 688-720, whose `box<T>` and
`buffer<T>` **heap rows retire with their types**, so that its derived release covers
exactly region-end reclamation, frame reclamation and the compiler-owned
system-resource release; whose table gains the store reset [PROV-5] and the sentence
that a linear type has no derived release; and whose 709-712 system-resource release
contract gains a second subject [RES-9]; [OWN-1] 563-571, whose classification is
unchanged and which gains the linear refinement, the partial-consume refusal, and
`dispose` in its consuming-use list; [GRAM-2]'s `struct_decl` and `enum_decl` (one
added modifier), [GRAM-4]'s `stmt` and `let_stmt` productions (one added statement
form and one added `let` alternative) and [FORM-2], which renders each on one line;
[EFF-2] 1427's "each of these memory-reclamation actions carries the empty effect
row", which stays **true** for the actions that survive and is joined by the walk's
own contribution; [PAR-1] 1975's footprint, through the ordinary `writes` row; and
[ERR-3] 1472's retained judgments, which gain the live-linear-binding refusal.
*Depends:* [STOR-3] 699-705, whose derived-drop order and its affine-element clause
are the walk this rule reuses; [OWN-5] 591, whose "content reached through any borrow
may never be moved" is what the consume half of `dispose` inherits, and 606, whose
exclusivity is why at most one live binding lends `&uniq` to a provider; [OWN-13] 654,
whose own-place match move is why a `match` is a destructuring. *Law:* L3, L5, L13,
L17. *History:* 6.10, F1 attacks 3, 4 and 10, F2 F6-1, F6-2, F6-13 and F6-16, and the
owner's D1 and the `using` removal; 6.9, F1 attacks 2 and 3, F2 F5-7 and F5-8.

**[PROV-7] A provider can be lent onward, generally.** A helper that receives a
provider as `&uniq 'b P` must be able to hand it to the operation that allocates.
Today it cannot: [OWN-6]'s child reborrow admits only a locally-introduced region
whose block does not extend beyond the enclosing statement, so a reborrow into `'b`
is inadmissible and a reborrow into a fresh local region cannot carry an affine
result out. The amendment is [OWN-6]'s own reasoning applied one position further,
and it is stated **generally, over every child reborrow and not only over a
provider**:

> A child reborrow may name a caller-supplied region `'b` that resolved(`holder`)'s
> region outlives-or-equals **when the receiving call's result type does not name
> `'b`**. That child's loan ends at the end of its receiving statement, and the
> parent resumes there.

*Judgment:* [OWN-6]'s admission, with one more admitted region source under the
stated result-type condition, which is the judgment [PROV-6]'s provider resolution
reads when the resolved binding is itself a borrow. *Publishes:* the child loan's
extent. *Amends:* [OWN-6] 616 and [OWN-4] 582. *Verified today:* probes `r1_relend`
and `m19` are `[OWN-6] InvalidChildReborrow`, and `r1_relend_affine` shows the
existing local-region escape cannot carry an affine result out. *Note:* this also
unblocks `docs/patterns.md` P17's threaded-factory shape. *Law:* L2. *History:* 6.8,
F4 finding 9; 6.6, F2-N3.

#### 3.K.3 `[BLK]`: the branded run of slots

**[BLK-0] The kernel declaration domain.** The container and store operations are
one compiler-owned **generic** declaration domain, built as [SYS-1] and [SYS-2]
build the system domain and admitted to every compilation unit on the same terms.
Each operation is one complete signature record: named parameters in declared order
[GRAM-11], its type, const and region parameters written as [GRAM-2] 196-198 orders
them, one declared effect row, one declared result mode and type or one ordered
result list, one declared requirement list, and one declared relation list.
**The first declared parameter is the value the operation transforms and returns; an
operation that transforms nothing names its provider first; and an operation that
neither transforms nor provides names the value it observes first**, which is the case
the two view formers need and which the sixth draft's two-case sentence had no room
for. The inventory is
Appendix A.2; the rule is that it exists and that every row satisfies the five
sentences below.

**Written arguments, per argument.** A row writes each type, const or region
argument exactly when no operand of that row determines it, and elides it exactly
when some operand does. That is 3.K.0's criterion applied to a domain, and it
replaces the fifth draft's all-or-nothing list, which made
`seq_heap::<u8>(heap: ..., count: ...)` a forbidden partial spelling under one rule
while both worked programs and the library wrote it under another. A written type
argument may itself be branded.

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

This is round 5's rank-three finding, and **round 6's finding is its quantifier**:
the sentence binds the thirteen rows of this domain and nothing else, while every
program in this file needs it of a wf function too. [CALL-7] is where the wider
quantifier lands, and the two sentences are deliberately separate, because this one is
a property a compiler-owned table must satisfy and that one is a declaration-site
obligation on a writer.

The arithmetic the completeness sentence buys is worth restating, because it is why
the sentence exists. The fifth draft licensed a row to publish two of three measures
"where two of the three follow from [MSR-2]'s identity". They do follow — but only
through the identity, which lives in the affine premise list, so reconstructing `room`
from `len` and `cap` costs two premises before the goal is reached, and the design's
own `spare` invariant then needs three where [ENT-6] 3019 admits exactly two. The
result was that **every appending and draining loop was refused**. Probe `g4` accepts
the identical three-term header when one exact relation is published and probe `g3`
rejects it when none is. The identity stays as a convenience for the writer; it is
never the route by which an operation's own post-state is derived.

**Every acquiring row carries [OP-9]'s allocation-fit obligation.** `buffer_new`
carries it today — probe `a4` is a run whose element count comes from the environment
and is `[OP-9] UndischargedAllocationFitObligation` — and the four rows that replace
it carried none, so a retirement silently deleted the one judgment that keeps a
source length representable on a target. [OP-9] and [STOR-6] are a matched pair: the
accepted [OP-9] judgment retains a numeric upper bound that target qualification
multiplies by the actual stride, so with no judgment the target stage has nothing to
multiply. Each acquiring row of A.2 therefore carries `requires fits::<T>(count)`,
[OP-9]'s own predicate under the name its `buffer` retirement gives it, discharged
under [MSR-4] like every other goal. `seq_fixed` carries none: `n` is a type constant
and [STOR-6]'s ordinary layout judgment covers it.

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
per-row requirement discharge under [MSR-4], the allocation-fit obligation included;
the [GRAM-11] named-argument check; and the completeness check over each row's
published relation set, which is the judgment [MSR-4], [CALL-2] and [RES-10] read
when they consume a row's post-state. A diagnostic for an operation cites **[BLK-0]**
and names the operation in its payload, exactly as an [OP-1] diagnostic cites [OP-1];
[DIAG-1] 1541 admits one numbered language rule and the inventory rows are table
data, not rules. *Publishes:* every declared relation of every row, at the denotation
[MSR-3]'s table gives it. *Amends:* [SYS-1] 2136 (a fourth admitted declaration
source), [SYS-3] 2309 (admitted to every unit), [TYPE-6] 396-407 (the operation
spellings enter the lexical IDENT domain, the nominals the TYPEID domain, and 401's
`callee` IDENT admission gains the fourth class), [DIAG-1] 1693-1718 (collision rank
5, and a `container_declaration_ordinal` beside the system one), [ENT-3] 2730 (one
added enumerated source S13, plus the arm route above) and [ENT-3.S6] 2785 (the
equality row generalizes over the four measures), [OP-1] 771-850 (`len` gains `cap`,
`room` and `head`, their domain extends to runs, views and providers, and `slice_of`,
`buffer_new`, `buffer_vacant`, `box_new` and `arena_new` retire; `ReservedLowerNames`
gains `cap`, `room` and `head`; 838's callee partition gains the fourth class),
[OP-9] 974-1001 (its predicate is carried by the acquiring rows of this domain),
[TYPE-5] 374 (the written-argument criterion covers a fourth callee class and becomes
per-argument), [GRAM-11] 346-350, and [FN-2] 1093 (its explicit-argument rule covers
this domain). *Law:* L11, L15, L16. *History:* 6.10, F2 F6-6; 6.9, F1 attack 4, F3
defects 2 and 6.

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

**Each is a run of slots whose initialized storage is a window** (L12,
owner-decided). The initialized set is exactly the `len` slots beginning at `head`,
taken modulo `cap`; the rest is raw. `len`, `cap`, `room` and `head` are [MSR-1]'s
terms with [MSR-2]'s facts and [BLK-0]'s readers. A run carries no other state: no
per-slot tag, no occupancy bitmap, no runtime discriminant. A subscript `v[i]`
selects the element at **logical** offset `i`, which is the slot at physical offset
`(head + i) mod cap` [MSR-1], and carries the ordinary [OP-4] obligation
`i < len(v)` — against `len`, never against `cap`, and never against `head`. A
`Vector<'s, T>` of capacity one is a single stored value, so the language needs no
box nominal and [TYPE-7]'s deref domain loses its three. `array<T, n>` is retained
exactly as it is, as the `len = cap = n`, `head = Z` case with no typestate and a
copy-only element domain, so `tests/programs/fir_filter.wf` is untouched.

**Why a window and not a prefix, and what it costs.** The fifth draft's prefix made
L12's own last clause false: a queue is not arithmetic a writer performs over append
and remove-at-the-end, and the price of pretending otherwise was a library ring over
`Option<T>`, which round 5 measured at **2072 bytes against a hand-written 280** for
a 256-byte ring under [OP-9] 992's own ceiling, and which deletes in-place slot
mutation because no place reaches inside an enum payload. The window makes a ring a
**run**: no `Option`, no tag, ordinary element access, exact `len`. Round 6 rebuilt
the comparison and could not reopen it, and it is the change every writer lens rated
the best any draft has made.

Its cost is five things, and no sixth:

1. one word per descriptor, which A.1 carries;
2. one more measure term, `head`, in [MSR-1]'s table and in every row that writes a
   run — four columns where the fifth draft had three;
3. one standing fact, `head(P) <= cap(P)`, beside the three the identity already
   gives;
4. one requirement on view formation, `head(vector) + len(vector) <= cap(vector)`
   [VIEW-2] — *the window does not wrap*, which is what a contiguous view actually
   needs; and
5. **one more boundary row**, `seq_rebase` [BLK-3], because without it `head` is an
   **absorbing state**.

**Cost 5 is round 6's and the sixth draft did not have it.** A.1 makes `head` the one
bounded cell and, in the sixth draft, **no row republished `head = Z`**: after one
`seq_take_front` a run's `head` was known only as `Z <= head <= cap` and every later
operation propagated that bound unchanged, including draining to empty. So a ring
could never be viewed again — not after a drain, not after a refill — and every
transmit path over a ring owed a permanent second run of full capacity plus an O(n)
copy per flush. `seq_rebase` is the priced escape and it is unwritable for exactly the
reason the other four rows are: it moves a checker-maintained boundary.

Cost 4 is also round 6's correction. The sixth draft wrote the premise as
`head(vector) <= Z`, which is stronger than a contiguous view needs and which a
drained run cannot satisfy; the non-wrap form is what the storage argument actually
requires, every back operation preserves it exactly at `head = Z`, an empty run
satisfies it from the standing facts, and `seq_rebase` re-establishes it in one
operation for a run that has wrapped.

Lowering pays one add and one conditional subtract per subscript. That is a runtime
cost and not a proof cost, and an optimizer that proves `head` identically zero for
a given run emits the ordinary `base + i * stride` — an optimizer fact that improves
an accepted program and changes no acceptance, which is the only kind this language
admits. **In a ring `head` is genuinely nonzero and no optimizer removes it**, so a
completion handler touching six fields of a descriptor pays it six times; the
language's own repair is to borrow the element once and write fields through
`deref(slot)`, probe `x10` shows that shape is `Semantics/Unsupported:
RegionsAndBorrows` today, and `docs/patterns.md` owes the pattern.

`T` may be copy, affine, or linear; the trichotomy is [OWN-1] 564's two classes plus
[PROV-6]'s refinement, and this sentence names it rather than restating it as three.
The window is what makes an affine element sound: an element enters and leaves only
through an operation that moves a boundary, so no slot is read before it is written
or after it is taken. A run over a linear `T` **owns** its elements, so it is itself
linear [PROV-6], and `dispose` walks its window.

*Judgment:* the ordinary nominal-resolution and construction judgments; a
`construct` naming a container nominal is a hard error citing BLK-1; [OP-4] at
every subscript, against `len`, which is the judgment [PROV-3] use 4 and [RUN-3]
read after [MSR-1]'s injectivity sentence carries it to storage. *Publishes:* the two
types, their measure rows and their window typestate. *Amends:* [TYPE-2] 357, two
added composite types, and its flat-element restriction, which the runs do not
inherit; [OP-4] 914, whose indexable bases extend to the two runs and the two views,
and whose obligation is against `len`. *Verified today:* `array_new::<box<u64>, 4>` is
[OP-1] `InvalidOperation` (probe `p9`), so an affine element is new capability.
*Law:* L12, L13. *History:* 6.10, F2 F6-15 and F4 finding 7; 6.9, F4 blocking 4.

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

Each acquiring row carries [OP-9]'s allocation-fit obligation over `(T, count)`
[BLK-0], which is the judgment that keeps a run's element count representable on the
selected target and which the sixth draft's retirement of `buffer_new` dropped in
silence.

**The fifth draft's `seq_frame` stays deleted**, and it is worth one paragraph
because it is the clearest instance of the pattern §2.1 exists to catch. It produced a
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

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge at the proved spelling and
at every allocation-fit obligation. *Publishes:* each run's measures, and each store's
post-state measures and refusal relation. *Amends:* [OP-1] 798-803 (`buffer_new`,
`buffer_vacant`, `box_new` and `arena_new` retire); [TYPE-2] 357. *Law:* L3, L4, L6,
L8, L18. *History:* 6.10, F2 F6-6; 6.9, F1 attack 5 and F3 defect 13.

**[BLK-3] Five operations move a boundary, and nothing else does.** `V` is either
run type.

```text
seq_place(vector: own V, value: own T)        -> own V   // [S8]  requires room(vector) > Z
seq_place_front(vector: own V, value: own T)  -> own V           requires room(vector) > Z
seq_take(vector: own V)                       -> (rest: own V, value: own T)
                                                                 requires len(vector) > Z
seq_take_front(vector: own V)                 -> (rest: own V, value: own T)
                                                                 requires len(vector) > Z
seq_rebase(vector: own V)                     -> own V           (no requirement)
```

Element access is the ordinary v0.41 surface over the initialized window: `v[i]`
reads, `set v[i] = e;` writes a copy element [LIV-2], and `let old = replace v[i] =
e;` exchanges an affine one [SET-2]. That surface is what a keyed table is built out
of, and probe `x7` compiles its shape today.

Each takes the run **by value** and returns it, carries `reads(vector),
writes(vector)`, and publishes its complete measure row on every exit.

**`seq_rebase` is the row round 6 showed a window needs.** It publishes
`head(result) = Z` with `len`, `cap` and `room` unchanged; its lowering is a rotate in
place, whose cost is O(len) and is the cost every real ring driver already pays before
a bulk transfer. Without it `head` is absorbing (see [BLK-1] cost 5), a wrapped run is
unviewable for the life of the value, and `CONTAINERS.md` §3.2's staging run is a
permanent duplicate of every ring a program ever views rather than a copy a writer
chooses. It passes L18's test for the same reason the other four do — it moves a
checker-maintained boundary and no wf program can — and it is the only addition this
draft makes to the inventory.

**There is no swap and no exchange operation, anywhere** (owner-decided 2026-09-03).
A swap of two whole non-overlapping places is `set (p, q) = move q, move p;` under
[LIV-2] and needs no operation; a swap of two elements of **one** run is refused by
[LIV-2]'s non-overlap condition and is three statements over the rows above:

```wf-design
let (rest, endv) = seq_take(vector: move vector);
let old = replace rest[at] = move endv;
let back = seq_place(vector: move rest, value: move old);
```

— the transposition of `at` with the last position, for copy, affine and linear
element types alike, and transpositions with one fixed position generate every
transposition. **What it costs is stated, and this draft states it correctly**: the
three statements kill and re-establish `len` twice where one row would have published
`len(result) = len(vector)` once, and the obligation the middle statement carries is
`at < len(rest)` where `len(rest) = len(vector) - 1`, so a caller must prove
`at + 2_u64 <= len(vector)` and the last position — where the transposition is the
identity — needs a dominating branch of its own. The sixth draft priced this removal
at `at + 1_u64 <= len(vector)`, which is one unit short of its own body's obligation
and which over `u64` is the same proposition as `at < len(vector)`, so the trade the
owner was asked to accept was priced against a program that does not compile. L18's
last sentence is the general repair and 3.L.2 walks the program.

There is **no removal from the middle, no clear, no truncate, no growth, no filled
construction and no vacant construction** in the kernel. Each is written in wf in
3.L, and 3.L.6 records that none of them needed a primitive the five rows above do
not have.

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge of each requirement.
*Publishes:* each row's declared relations, `seq_rebase`'s `head(result) = Z`
included, which is what [VIEW-2]'s premise reads. *Amends:* nothing beyond [BLK-0]'s.
*Verified today:* probe `c8` shows a function writing one position of an
`own buffer<u8>` parameter and returning it must exhibit `writes(vector)`, so these
rows are not `pure`. *Law:* L4, L9, L12, L15, L18. *History:* 6.10, F2 F6-15, F1
attack 7, and the owner's D2; 6.9, F3 defect 16 and F4 blocking 4.

**[BLK-4] Confinement, the one position closure, and the `&uniq` parameter
refusal.** A type is **confined** when its complete type after substitution names a
region. The confinement of a value is the **set** of regions its complete type names,
and it may be moved, returned, or bound to a destination that **every** member
outlives-or-equals [OWN-3]. That quantifier is the whole rule: a value of type
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
aliases nothing.

**And no container nominal and no loan-bearing type may be the referent of a
`&uniq` parameter of a source-declared `fn`.** This is R1 stated as a rule, which the
owner's ruling made it and which six drafts had asserted as doctrine:

> In the parameter list of a source-declared `fn`, a parameter of mode `&uniq` is a
> hard error citing BLK-4 at the complete `param` when its referent type **is, or
> reaches at any depth, a container nominal or a loan-bearing type**. Depth is the
> reachability closure [PROV-4] computes over fields, enum payloads, run elements and
> written type arguments. The restructuring is `take the run by value and return it,
> or take a length-fixed view of it`.

Three things about it are deliberate. **The closure is what closes the round-4
defeat**: [CNT-7] refused a parameter whose *direct* type is a container and a
one-field wrapper struct nullified it, and "reaches at any depth" is the same
computation [PROV-4] already performs for providers. **A provider is admitted**,
because it is neither a container nominal nor loan-bearing and because [PROV-2] states
why no callee can change its identity. And the clause quantifies over a
**source-declared** `fn` and not over the compiler-owned domains: a [BLK-0] or
[SYS-2] row is a declaration record whose relations are complete over everything it
writes ([BLK-0]) and whose behaviour no body can vary, so there is no unnamed point at
which a caller's object could change — which is precisely the ground L11's second
sentence gives. `seq_mut_slice(vector: &uniq 'r v)` and `read_at(destination: &uniq
mut_slice<u8>, ...)` are therefore unaffected, and they are the two shapes goal A's
I/O loop is written over.

Round 6 wrote two programs against the missing rule and this clause refuses both at
their declarations: a helper taking `handle: &uniq Vector<u8>` that replaces the
referent (D1 again, one type over), and a helper taking `destination: &uniq
mut_slice<'r, u8>` that installs a shorter view into it, which reached an
out-of-bounds element write in a `pure`, heap-free, `resource_closed`-eligible
program. Both are declaration errors now, and neither needs a kill to be classified.

**A source nominal may declare region parameters**, written exactly as a function
declares them, and is confined by them:

```wf-design
struct Chunk['s] {
  page: Vector<'s, u8>;
  used: u64;
}
```

— region parameters on a nominal, [S20] — and it is used as `Chunk<'s>`, an ordinary
TYPEID with `targs`. Under 3.K.0 a nominal over the entry heap declares no region
parameter at all and is written `Chunk`; the parametric form is what a program with
two stores writes. Two instances of one such nominal have the same type only when
their region arguments are identical: region parameters on a nominal are
**invariant**, which is [OWN-12] 650 and [TYPE-5] 379 applied where they already
apply, and which is why this feature needs no variance design.

**A stored position with no admissible brand is this rule's error, not a
resolution failure.** When a nominal declares no region parameter, the entry selects
no `command.heap`, and a field's type needs a store brand, the position is a hard
error citing BLK-4 at the complete contained `type`, with the restructuring `give
this nominal a region parameter and confine the field to it`. That is 3.K.0's
non-emptiness sentence given a home.

*Judgment:* the `&uniq` parameter refusal above, over [PROV-4]'s type-reachability
closure, which is the judgment [CALL-3] and [MSR-3]'s `&uniq` row both read; a
loan-bearing or provider type in a prohibited position, or a confined type in a
position whose owner does not name its region, is a hard error citing BLK-4 at the
complete contained `type`, with the restructuring `keep the view as a direct local,
parameter, or result` for the first, `lend the provider to the operation that needs
it` for the second, and the sentence above for the third; and a confined value bound
to a destination some member of its region set does not outlive is a hard error citing
BLK-4 at the binding, rendering every member. *Publishes:* the confinement set, and
the fact that no source-declared `&uniq` parameter reaches a container nominal or a
loan-bearing type, which is what [CALL-3]'s narrowed default and [MSR-3]'s denotation
table rest on. *Amends:* [STOR-4] 721, whose "may not be returned" becomes the
ordinary outlives relation over the set; [STOR-5] 723-737, whose enumerated position
list is replaced by the intensional split above and whose deferral of per-leaf
provenance inside stored values is **withdrawn as unnecessary** rather than
discharged, because a store brand is a type parameter and needs no per-leaf record;
[FN-2] 1093, whose blanket rejection of a region-bearing generic argument narrows to
loan-bearing and provider arguments and whose "instantiation arguments are always
explicit" now covers region arguments on nominals; [GRAM-2]'s `struct_decl` and
`enum_decl`, which gain `region_params?` after `generics?`; and [FN-1] 1005-1012's
parameter-list admission, which gains the `&uniq` referent refusal. *Depends:*
[OWN-3] 580, whose fail-closed incomparability is the invariance argument.
*Verified today:* probe `f7_regionresult` is [FN-2] `RegionBearingGenericArgument`,
probes `r2_6` and `m05` are [GRAM-2] parse errors at `struct Wrap['p]`, and probe `d1`
compiles the `&uniq` container parameter this clause refuses, so all three halves are
new. *Law:* L10, L11, L13. *History:* 6.10, F1 attacks 1 and 2, and the orchestrator's
R1-as-a-rule ruling; 6.9, F1 attack 6.

*[CNT-1] through [CNT-7] and [SEQ-0] are deleted.* Five owners, a per-owner release
table, a `&uniq`-container prohibition, a growth rule and an operation-domain rule
are [BLK-0] through [BLK-4] plus 3.L. **[CNT-7]'s effect is restored by [BLK-4]'s
fourth clause, over the reachability closure that defeats its wrapper problem**; its
id stays retired and is not reused.

#### 3.K.4 `[VIEW]`: views and loans

**[VIEW-1] The two views.**

```text
| type              | reads | writes elements | changes length     | loan      | class  |
|-------------------|-------|-----------------|--------------------|-----------|--------|
| slice<'r, T>      | yes   | no              | no                 | shared    | copy   |
| mut_slice<'r, T>  | yes   | yes             | no                 | exclusive | affine |
```

`slice<'r, T>` is v0.41's own type under v0.41's own name and **this design does not
rename it** (owner-decided 2026-09-03): the Rust precedent is exact, the semantics do
not differ, and a rename buys a reader nothing. `mut_slice<'r, T>` **[S6]** is the one
added view, and it is added because [SET-1] 488-490 makes every slice-rooted target
unwritable, so no writable view exists today and probe `p7` is the refusal.

Each is an `own` value carrying a region `'r`, and each is loan-bearing [PROV-3]. Its
measures are [MSR-1]'s rows, with `head` exact at `Z` because a view is formed only
over an unwrapped window [VIEW-2].

**The shared view is `copy` and the writable one is affine** (owner-decided
2026-09-03, [S27]). [OWN-1] 564 makes `slice` affine today, and affinity there buys no
safety: duplicating a shared view is only a second **shared** loan on the same range,
which [OWN-5] admits without limit, and a value that owns nothing [PROV-3] has nothing
a second copy could double-free. What affinity costs is a re-formation at every second
use, which is a `seq_slice` call and a fresh borrow in the middle of a loop that had
one. `mut_slice` stays affine because two exclusive loans on one range are exactly
what [OWN-5] 606 refuses, so an exclusive view must be moved rather than copied and
the single-writer argument is unchanged. The Rust precedent is exact in both
directions: `&[T]` is `Copy` and `&mut [T]` is not.

Three consequences are stated because rules read them. A `slice` operand is used
**without `move`** [OWN-1] 564, so `collect(out: move buf, source: line)` is the call
spelling and a `move` of one is `[OWN-1] MoveOfCopy` (probe `x14` is that diagnostic at
a copy element type today). A `slice` target of a [LIV-2] `set` falls under that rule's
copy case and needs no consume. And a `slice` is never linear, never disposed and never
destructured, which was already true of every loan-bearing type [PROV-6].

There is no third view. The fourth draft's `AppendView` presented a run's spare
window so that a caller's length could survive an appending callee; the fifth draft
replaced it with an exit datum over a `&uniq` parameter, which round 5 broke. Under R1
an appending helper takes the run **by value and returns it**, so the caller's length
is the result's length, published by an ordinary `ensures` over an ordinary result and
required to be complete by [CALL-7]. **What a writer gains back is the guarantee L14
was retired for**: `ensures len(rest) >= len(out)` says that the helper did not
shorten what it was handed, relates a result to an input, names one state of each, and
needs no `old()`, no frame rule and no third type.

*Judgment:* none by itself; it fixes the two types and their loan strengths, which
[PROV-3] use 1 judges and [CALL-3] and [VIEW-4] read. *Publishes:* the two types,
their loan strengths, and the loan-bearing predicate. *Amends:* [TYPE-2] 357 (one
added view type; `slice`'s spelling is unchanged), [OWN-1] 563-564, whose
classification gains `mut_slice` as affine and **moves `slice` to copy**,
and [CONST-2] 552-556, [OP-7] 940 and [OP-1]'s `slice_of` row, which name the retired
constructor. *Law:* L10. *History:* 6.10, the owner's naming decision and [S27];
6.9, R1.

**[VIEW-2] Formation, the loan the view value holds, and the non-wrap premise.** A
view is formed from a borrow of the run:

```text
seq_slice['r, T](vector: &'r v)          -> own slice<'r, T>      reads(vector)   // [S10]
    requires head(vector) + len(vector) <= cap(vector)
seq_mut_slice['r, T](vector: &uniq 'r v) -> own mut_slice<'r, T>  reads(vector)
    requires head(vector) + len(vector) <= cap(vector)
```

and **the view value, not the argument borrow, holds the loan**. For its whole
life, a view value holds a loan of its own strength on the logical range it reaches
of every place in its resolved origin set [PROV-3]. The loan begins at formation and
ends when the view value is consumed or released. The argument borrow is a
call-scoped temporary, which probes `f2b`, `r1_twouniq` and `w8` confirm by
accepting two of them on one place in one region with an ordinary write between; it
could not be the freeze.

**The `requires` is the window's one visible cost, and it is stated over the property
a contiguous view needs** [BLK-1] cost 4. A view is one contiguous range and a
wrapped window is two, so formation is admitted exactly where the window does not
wrap. Three things then hold that did not hold under the sixth draft's stronger
`head(vector) <= Z`: every formation row publishes `head = Z` and every back
operation preserves it, so a program that never removes from the front discharges the
premise by a chain of exact equalities and states nothing; **an empty run satisfies it
from the standing facts alone**, `head <= cap`, so a drained ring is viewable; and a
run that has wrapped is returned to the premise by one `seq_rebase` [BLK-3] rather
than being unviewable for the life of the value.

**And the premise crosses a contract**, which is the other half of the same round-6
finding. `[BLK-0]`'s completeness sentence binds the kernel rows, so the chain of
exact equalities is exact **inside one function** and a loop backedge removes it
([ENT-5] 2942-2946); a caller of `filled::<u8, 4096>()` therefore knew nothing about
`head(input)` and every `seq_slice` in every program in the sixth draft was
undischarged, including both worked programs and the one statement 4.2's walkthrough
calls goal A's container half. [CALL-7] is the repair: a wf function that hands a
measured value back publishes every measure of it, so `filled`'s contract carries
`head(result) = 0_u64` and the premise discharges at the caller.

*Judgment:* [OWN-5] at the formation borrow, [MSR-4] discharge of the non-wrap
requirement, and the ordinary [BLK-0] relation establishment. *Publishes:* the loan,
and the two formation rows' relations. *Amends:* nothing beyond [PROV-3]'s amendment
of [OWN-5]. *Depends:* [OWN-5] 606, the conflict sentence that refuses a second
exclusive view, and [OWN-6] 614, which makes the argument borrow call-scoped.
*Law:* L10, L15. *History:* 6.10, F1 attack 5, F2 F6-15, F4 finding 1; 6.9, F4
blocking 4.

*[VIEW-3] and [VIEW-5] are deleted.* [VIEW-3] was `absorb`, the append window's
commit event, and [VIEW-5] the disposition of an abandoned window. Both retire with
`AppendView`; their ids are not reused.

**[VIEW-4] A `replace` may not displace a live loan.** A commit that displaces a
value of loan-bearing type is admitted exactly when the displaced value is **consumed
by that same statement's right-hand side**. Under [LIV-2] every `set` satisfies that
by construction — the rule requires every target to be dead at the commit, which for
an affine target means its previous value was consumed by the right-hand side — so
the clause bites on exactly one form:

> `let old = replace p = e;` where `p`'s type is loan-bearing is a hard error citing
> VIEW-4 at the complete target `place`, with the restructuring `consume the displaced
> view in the same statement with set, or bind a new view under a new let`. Its
> replacement is written by the writer and the displaced view survives as `old`, so
> the loan the displaced view held would outlive the descriptor whose place it was
> held from.

`replace` at a non-loan-bearing place — a keyed table's `Option<T>` slot, a run
element — is untouched.

**The sixth draft's headline sentence is deleted rather than repaired**, and that is
round 6's first BREAK. It read *"No operation takes a `mut_slice` or a `&uniq` to one
and produces a different length, and none changes its owner's length"* — a **claim**,
whose `Judgment:` line stated only the commit admission. [VIEW-6]'s ceiling admits a
function returning either of two same-region view parameters, so `pick(a, b) -> b`
produces a different length; the commit admission then let it through a `&uniq
mut_slice` parameter; and [CALL-3], which read the claim as a premise, handed the
caller a stale length into an out-of-bounds element write. Two rules close it and
neither is a patch to this one. **[BLK-4]** refuses the `&uniq mut_slice` parameter at
its declaration, so no callee installs a view behind a caller's back. And **[CALL-3]**
is restated over what a view can *write* — element storage — which [PROV-3] and
[EFF-1] 1386 already judge, rather than over what its length is, which nothing judged.
A view a function hands back is an ordinary result whose measures come from
[CALL-2] and [CALL-7] like every other result's, so `pick` is sound and useless
rather than sound-looking and dangerous.

*Judgment:* the `replace` refusal above, which is the only fact this rule states.
*Publishes:* nothing; the length-fixed class the sixth draft published here is gone,
and [CALL-3] reads [PROV-3]'s write classification instead. *Amends:* [SET-2]
513-529's admitted commits, beyond [PROV-3]'s amendment of it. *Depends:* [PROV-3]
use 3, whose storage-keyed sentence is why this rule is about the descriptor and not
about the storage. *Law:* L10, L11. *History:* 6.10, F1 attack 1; 6.9, F3 defect 4.

**[VIEW-6] Views are never stored, and a view result declares its origin.** A view
is never stored [BLK-4] and never returned except under this rule. [FN-1]'s
slice-result ceiling applies unchanged to each view type: a function whose written
result is `own slice<'r, T>` (respectively `mut_slice`) has the ceiling containing
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
borrow, so `seq_slice` and `seq_mut_slice` are usable only in the function that
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
a destination the operation writes  ->  &uniq 'd mut_slice<'r, u8>
a source the operation reads        ->  &'s slice<'r, u8>
```

so `read_at(file: &ReadFile, destination: &uniq mut_slice<u8>, file_offset: own u64,
start: own u64, end: own u64) -> result: own ReadOutcome`, whose three regions
relate nothing and are therefore all elided (3.K.0).
Both are borrows of the **descriptor**, so the view survives the call and a
destination can be filled by a loop of reads, which an `own` destination could not.
Both write element storage only, so [CALL-3] gives the caller its measures back.
The two obligations keep their form and their order with `len(deref(buffer))` reading
`len(deref(destination))`.

**The fourth clause of [BLK-4] does not reach these**, and the reason is the clause's own
scope rather than an exception: a [SYS-2] declaration record's behaviour is fixed by
its record, its relations are complete over what it writes, and it has no body in
which an unnamed point could exist. That is the same ground L11's second sentence
gives, and it is why the clause quantifies over a source-declared `fn`.

This is the change that lets a heap-free program do I/O, and it is a rule rather
than a register row because it is goal A's container half. Its cost is that a
destination must be **addressable** before the host writes into it, so it is built
by 3.L.3's `filled` and the count the host produced is an ordinary `u64` beside the
run rather than the run's own `len`; Q7 records the fix. Its second cost under the
fourth draft — two writer-visible regions per I/O site — is **gone**: both relate
nothing, so both are elided under 3.K.0.

*Judgment:* [SYS-8]'s two range obligations, restated over `len` of the borrowed
view. *Publishes:* the endpoint facts [ENT-3.S10] already enumerates, now over a
view. *Amends:* [SYS-8] 2488-2527, [SYS-2] 2164-2307's declaration records and its
normative counts, and the prose of [SYS-9], [SYS-11], [SYS-12] and [SYS-14], which
name `buffer<u8>`. *Depends:* [EFF-1] 1386 as [PROV-3] amends it, which is what
makes a view parameter's effect path name the viewed backing rather than the
descriptor, and which is the judgment [CALL-3] reads. *Law:* L11. *History:* 6.8,
F1 attack 9; 6.7, F3-10.

#### 3.K.5 `[LIV]`: liveness and the one commit rule

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
is a LIV-1 error naming the head. And [OWN-1] 566-567's "SET-1 and SET-2 recheck the
live-root premise after their right-hand sides and never revive a dead binding" is
**replaced** by [LIV-2]'s own commit premise, which is stated over the state at the
commit rather than over the state before the statement — a difference the sixth draft
kept as an exception and D2 removes.

*Judgment:* a per-join and per-scope-exit structural check over the ownership state
the checker already computes; no search. This is the judgment [PROV-6]'s scope-exit
refusal, [STK-1]'s tail premise and [STK-4]'s unreachable-exit sentence all read.
*Publishes:* the unconditional release set of every edge. *Amends:* [OWN-1] 563 and
566-567, and [OWN-11] 646, as stated above. *Law:* L17. *History:* 6.9, F3 I9; 6.5,
F1-1, F1-2.

**[LIV-2] One `set` commit rule.** **Owner-decided 2026-09-03 (D2).** One statement
writes places, and it replaces three: [SET-1]'s copy overwrite, the sixth draft's
reinitializing `set` at a dead binding, and its in-place exchange. `[LIV-3]` is
retired into this rule and its id is not reused.

```wf-design
set p = e;
set p = f(vector: move p, value: byte);
set (p, taken) = seq_take(vector: move p);
set (p, q) = move q, move p;
set (a, b, c) = move c, move a, move b;
```

**The form.** `set (p1, ..., pn) = rhs;` for `n >= 1`, with the parentheses omitted
at `n = 1`. The right-hand side is either **one call with `n` results** or a **value
list of `n` expressions**, evaluated left to right. Each target is a `place`
[GRAM-5] — a bare binding, a field selection, a `deref`, or a subscript — or an
identifier that resolves to no binding in scope, which introduces one exactly as a
`let` does.

**The commit.**

> The right-hand side is evaluated in full first, and **through that evaluation every
> target is dead**. Then **all targets are reinitialised at one commit**, in declared
> order, each from its own ordinal's value. There is no writer-observable program
> point between the read-out and the commit (spec 520), so there is no partial move,
> no dead root and no uninitialized hole, and every target is live afterwards.

**The three admission conditions, and no fourth.**

1. **Every target is dead at the commit.** A target is dead there when the
   right-hand side consumed its previous value (`move p` occurring in it), when the
   target was already dead before the statement, or when the target's type is copy,
   for which there is nothing to consume. A **live affine target whose previous value
   the right-hand side does not consume** is [STOR-1] 679's error, kept for exactly the
   case it was written for, with the restructuring `use replace`.
2. **The targets are pairwise non-overlapping places.** A place and its sub-place
   (`p` and `p.f`, `v` and `v[i]`), and two subscripts of one run (`v[i]` and `v[j]`),
   are refused, because the commit order would decide the result. The refusal is a hard
   error citing LIV-2 at the second target, with the restructuring `write the two
   commits as two statements, or take the element out and put it back` — which for two
   elements of one run is [BLK-3]'s three statements.
3. **Arity and type.** The right-hand side supplies exactly `n` values and each
   ordinal's type is exactly its target's type [TYPE-5].

**What falls out, rather than being asked for.** `set p = f(vector: move p, ...)` is
the transformation the sixth draft called an in-place exchange, at a bare binding, a
field, a `deref` or a subscript alike. `set p = e;` at a dead `p` is the
reinitialization. `set v[i] = 7_u8;` at a copy element is v0.41's own `set`.
`set (p, q) = move q, move p;` is a **swap**, and three targets rotate — which is why
this design has no swap operation anywhere (footnote 4). `set (pending, unplaced) =
try_place(...)` writes one place and introduces one binding, which is what a
two-result library row needs at a place that is not a bare binding.

**Its judgment is [SET-2]'s, not [SET-1]'s, and that is what makes it not sugar.**
For each ordinal the previous value is read out of `resolved(p)`, the right-hand side
runs, and the value is written back into `resolved(p)`. At a bare binding a writer
could sometimes rebind in two statements — `let next = f(vector: move p, ...); set p =
move next;` — and at every other place they cannot: `move p[i]` and `move p.f` are
partial moves that kill the root [OWN-1] 569, and `move deref(handle)` is a move
through a borrow, which [OWN-5] 591 forbids outright with [SET-2]'s exchange as the
sole exception. So the only alternative route is a placeholder of the displaced type,
which for a `Vector<'s, T>` is a run that owns storage and is itself linear, so every
transformation costs an allocation and a disposal on a provably dead arm, and for a
type with no cheap empty value there is no route at all. **And the two-statement form
is not equivalent even at a bare binding**, which round 6 established and which is a
better argument than the one about places: `let next = ...; set p = move next;` is a
move-rebind, and without [MSR-3]'s rebind placement it destroys every measure fact
about the run (probe `x11`).

**Its effect footprint.** The statement exhibits one read and one write of each
target's ultimate storage origin, and the right-hand side's own projected row in
addition. Deriving a field-precise footprint from a callee's row would be wrong,
because the value written back is a whole new value of the target's type and the
callee's row describes what it read and wrote *inside* that value. [MSR-2] kills over
that write, [PAR-1] 1975's overlap test reads it, and [CALL-3] classifies it when a
target is itself an argument's referent.

**Term identity, in one sentence** [MSR-3]. A target that names a binding in scope
**keeps that binding's [ENT-2] term**: the statement is a write of the place, the
facts over it die by [MSR-2], and the right-hand side's declared relations
re-establish them on the same term through [CALL-4]'s destination clause — which is
what makes a header invariant over an appending loop's run survive its own backedge.
A target that resolves to no binding in scope introduces one and is a declaration
event. The sixth draft needed three separate rulings here and a diagnostic for the
invariant one of them silently orphaned; **that diagnostic is deleted**, because a
name in scope is a place and a place keeps its term, so an invariant the commit's
relations fail to re-establish is the ordinary [INV-1] failure at the loop head.

*Judgment:* the three admission conditions above, each a hard error citing LIV-2 at
the target or statement named there, plus the commit itself; this is the judgment
[MSR-2]'s kill, [MSR-3]'s atom identity, [PAR-1]'s footprint and [CALL-4]'s
destination clause all read. *Publishes:* the right-hand side's declared relations on
each target, the statement's read and write of each target's ultimate storage origin,
and the term-identity rule above. *Amends:* [STOR-1] 674 and 678-679 (the
writable-place partition: [SET-1] and this rule write places, [SET-2] replaces one,
and 679's diagnostic keeps the live-affine-target-with-an-unrelated-right-hand-side
case); [SET-1] 481-505, which becomes this rule's `n = 1`, copy-target case; [SET-2]
513-529, whose "it establishes no fact" sentence becomes false for this form, whose
target may be linear or region-bearing because nothing is rebound, and whose exchange
exception to [OWN-5] 591 this rule inherits; [GRAM-4]'s `set_stmt` production (a
target list and a value-list right-hand side); [OWN-1] 566-567, replaced as [LIV-1]
states; [ENT-2] 2683's term-identity paragraph (a target resolving to no binding is a
declaration event); and [ENT-3.S12] 2833's destination list, through [CALL-4].
*Verified today:* probes `x5`, `t8`, `x2` and `x3` are [STOR-1] `AffineSetTarget`,
probe `p10` is `AffineSetTarget` at a live target and probe `w6` is [OWN-1]
`UseAfterMove` at a dead one, and probe `w8` accepts a `set` at a `match` arm binder,
so every half of this rule is new capability and not a compiler defect. *Law:* L10,
L16, L17, L18. *History:* 6.10, the owner's D2 and F4 findings 5 and 8; 6.9, F3
defects 1 and 3, F1 attacks 9 and 10.

#### 3.K.6 `[CALL]`: what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the
callee's declared parameter modes and types and its declared contract.** These are
the owner's three call rules of 2026-09-03. Three rules beside them make the transports
usable: [CALL-4] is the vocabulary a contract may be written in, [CALL-6] is how a
declared relation becomes a fact, and [CALL-7] is the obligation that a contract be
complete about what it handed back.

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

*Judgment:* none; the absence of a kill, which is [MSR-2]'s judgment not firing.
*Publishes:* the survival of every such fact. *Amends:* nothing. *Depends:* [OWN-5]
585-606's shared-holder prohibition, which is the whole ground; [MSR-2]'s kill
classification, which is the judgment this rule reads. *Verified today* for
`&'a buffer<u8>`: probe `p6` keeps `len(line) = 10` across the call and the
subsequent `line[9_u64]` is accepted. *Law:* L11. *History:* 6.9, F1 attack 2.

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
have. Round 6 measured how far it composes and the answer is further than any earlier
draft assumed: probes `w2` and `w3` chain published `>=` relations to depth five with
the goal asked once at the end, both accepted, and probe `w5` bridges a published
relation whose one side is a measure term into an L0 fact about an ordinary binding.
Probe `w1` is the control — three helpers with no `ensures` fail at the **second**
link — which is this rule being exactly as strong as the callee's contract and no
stronger, and which is why [CALL-4] and [CALL-7] are load-bearing rather than
convenient.

*Judgment:* the ordinary [ENT-3.S12] establishment, subject to `M(c,q)` as [MSR-3]
amends it. *Publishes:* the callee's declared relations on the result. *Amends:*
nothing beyond [MSR-3]'s. *Verified today:* probe `p1`, `passthrough(out: move a)`
returning the same buffer, then `b[9_u64]`, is **rejected** with residual
`9_u64 < len(b)`; the transport already behaves correctly and what was missing is
the vocabulary to publish across it. *Law:* L11. *History:* 6.10, F4 §3.1; 6.9, R1.

**[CALL-3] An element write through a view never touches a measure of its origin.**
For an argument of **loan-bearing** type — `slice<'r, T>` or `mut_slice<'r, T>`, own
or behind a borrow — a projected callee `writes` occurrence kills every fact whose
support overlaps the viewed **element storage** and kills no measure term over that
origin. For every other parameter type the projected write kills measures as an
ordinary descriptor-storage-overlapping event [MSR-2].

**Its premise is what a view can write, and that is a judgment rather than a
claim.** The sixth draft rested this classification on [VIEW-4]'s sentence that no
operation produces a view of a different length, which no rule judged and which round
6 falsified. What the classification actually needs is that a callee holding a view
can reach **element storage and not descriptor storage**, and three rules judge
exactly that: [EFF-1] 1386 as [PROV-3] amends it makes a view parameter's effect path
name the viewed backing; [PROV-3] use 1 judges every access through a view at the
range it reaches; and [SET-1] as [PROV-3] amends it admits a target path through a
view only at exclusive loan strength, which reaches an element and never a descriptor.
So the caller's `len(origin)` survives because no occurrence the callee can exhibit
projects onto the origin's descriptor storage — the same ground [CALL-1] has, one
type over.

**What a caller learns about a view a callee handed *back* is [CALL-2]'s and not
this rule's.** A returned view is a fresh binding carrying exactly the callee's
verified relations, so a helper that returns the shorter of two views tells its caller
nothing about the result's length and every subscript of it is undischarged. That is
the sound answer and it needed no rule: the danger was never a view a callee returns,
it was a view a callee **installs**, and [BLK-4] refuses the parameter that installs
one.

**"Every other parameter type" no longer includes a borrowed run**, because [BLK-4]
refuses it. The conservative kill therefore remains as the default for every parameter
type this design does not classify, and it is no longer load-bearing for D1 — which is
the point, because round 5 showed a kill can be defeated by a fact published after it
and by an action that is not a write, and a design whose central defect is closed by a
kill has one door per channel.

*Judgment:* the kill classification per parameter type, which is [MSR-2]'s judgment
parameterized by [PROV-3]'s access classification. *Publishes:* the surviving
measures. *Amends:* nothing beyond [MSR-2]'s. *Depends:* [EFF-1] 1386 as [PROV-3]
amends it, without which a view parameter's projected write reaches the descriptor and
not the element storage this rule names; [PROV-3] use 1 and [SET-1] as amended, the
two judgments that fix what a view can write; [BLK-4]'s fourth clause, which is why
the default reaches no run. *Law:* L11. *History:* 6.10, F1 attack 1; 6.9, R1.

**[CALL-4] Contract vocabulary, the ordered result list, the routes, and where the
relations land.** [FN-9]'s clause operands are terms [MSR-5], so `len(P)`, `cap(P)`,
`room(P)` and `head(P)` over an admitted formal place are operands with no per-family
admission. The same terms over an admitted **result** place are operands too, which
today's result-datum restriction to fragment integers forbids: probe `e1` is the parse
rejection and probe `x2` is the resolution rejection, so this is a semantic addition
at both levels.

```wf-design
fn collect['s](out: own Vector<'s, u8>, source: own slice<u8>)
    -> (rest: own Vector<'s, u8>, written: own u64)
    reads(out, source), writes(out) contract {
  requires len(source) <= room(out);
  ensures len(rest) == len(out) + written;
  ensures room(rest) == room(out) - written;
  ensures cap(rest) == cap(out);
  ensures head(rest) == head(out);
  ensures written == len(source);
} { ... }
```

The ordered result list is [S16] and the clause operands are [S17]. **No clause names
two states of one term, and under R1 no clause needs to.** A parameter is an input and
has exactly one state; a result is an output and has exactly one state; a relation
between them is single-state in both. There is no `old()`, no frame rule, and no
entry/exit convention to remember. **This is also where L14's retired guarantee comes
back**: `len(rest) >= len(out)` says the helper did not shorten what it was handed,
and it is an ordinary clause with no special machinery anywhere. The five clauses
above are what [CALL-6] requires of this signature, and [MSR-5]'s six `compare_op`s
are why each exact one is one clause rather than two.

**A function may declare an ordered result tuple [S16]**, and each result binding is
a datum of every clause of that function, so one clause may name more than one.

**A relation is published per enum variant and per result ordinal, and a result
datum admits field projection [S24].** [FN-9] 1307 admits exactly one routed shape,
`when Ok(value: r):` for `own Result<T, E>` with `T` a fragment integer, and 1314
excludes a nested result projection outright. That is the narrowest useful surface a
contract could have, and round 5 measured what it costs: **no library constructor
can publish a fact about the run it built and no fallible helper can publish that it
succeeded.** Probe `x1` is the rejection for an `Option` result and probe `x2` for a
projected result datum; probe `x13` shows a routed clause read at the caller's arm
once the route is admitted. The generalization is four sentences:

> A routed clause is admitted as `when b is V(f: r):` where `b` **names the result
> ordinal** the route applies to and `V` is any variant of that ordinal's enum type,
> with `r` that clause's fresh symbolic payload datum. The ordinal binder may be
> omitted exactly when one ordinal of the result list has that enum type. An unrouted
> clause is admitted for a written result of any **measured** type as well as any
> fragment integer. Every ordinal is a datum of every clause. `len(P)`, `cap(P)`,
> `room(P)` and `head(P)` are operands for an admitted place `P` formed from a result
> datum with field and `deref` projections, on exactly the terms [FN-9] 1313 already
> grants a parameter datum, whenever `P`'s final selected type is measured.

**The ordinal binder is round 6's**, and the precedent for refusing the ambiguity
rather than resolving it is [VIEW-6]'s. The sixth draft's variant route named a
variant and the ordinal route named an ordinal binder, and a function with two
same-typed enum results — `-> (rest, near: own Option<Lease<'s>>, far: own
Option<Lease<'s>>)` — left `ensures when Some(value: l): room(l.run) >= 256_u64;`
routed to the first, to both, or to neither, with three readings all consistent with
the text and one of them unsound. Naming the ordinal decides it, and the omission is
admitted exactly where it cannot be ambiguous.

The relation stays a comparison over two `u64` terms in every case; what widens is
which datums may occur in it, not what a relation is.

**Those relations reach the caller through one added [ENT-3.S12] destination
clause**, and without it a multi-result contract publishes nothing: 2833 fixes a
closed destination list of four, and a destructuring `let`, a `set` target list and a
`match` arm binder are none of them. The clause is the single-binder route quantified
over ordinals and stated over every writing form:

> Each binder of a destructuring `let`, **each target of a [LIV-2] `set`**, **each
> arm binder of an own-place `match`**, and each binder of a destructuring consume
> [S13] is the S12 destination for every published relation naming the value that
> lands there, established after the call's ordinary transfer, consumes, borrow
> commits, target commit and kills in [ENT-5] 2898-2905's existing order, with
> `M(c,q)` requiring every other referenced support to be live at establishment.

Four forms, one clause, which is what the register's sixth condition asks for. **The
`match` arm binder is round 6's addition**: every allocating row in A.2 publishes its
relations on a `Some` or an `Ok` payload, so the arm binder is the destination the
whole inventory depends on, and the sixth draft named it only in [BLK-0]'s `Amends:`
line for a different rule. The same clause carries [MSR-3]'s rebind, payload and field
placements, so a datum minted at one of the three new placements lands on its binding
by the route every other relation takes. [FN-9] 1357's narrow direct-set route is
subsumed rather than joined: its extra premises exist only because that route
substitutes a receiver that is *also* an argument, and this clause covers the case
where it is not.

*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set, the
widened result shape and the widened route set; the ordinal-binder requirement above,
a hard error citing CALL-4 at the clause when a variant route is ambiguous; and the
S12 establishment at each of the four destinations, which is the judgment [CALL-2],
[LIV-2], [MSR-3] and [PROV-6] all read. *Publishes:* the clause relations, on every
result ordinal and on every admitted variant route, at each of the four destinations.
*Amends:* [FN-9] 1301-1365 (measured and multi-ordinal results, per-variant routes
with an ordinal binder, result field projection, multi-datum clauses, and 1357's
narrow route subsumed), [ENT-3.S12] 2833's destination list (one added clause, serving
four forms), [GRAM-2]'s `fn_decl` result shape, [GRAM-4]'s `let_stmt`, `set_stmt` and
`return_stmt`, [FORM-2] 52-78's rendering, and [FN-1] 1005-1019's result shape.
*Verified today:* probes `e1` and `x2` show a measured or projected result operand
does not parse and does not resolve, probe `x1` shows a variant route on an `Option`
is `[FN-9] InvalidPostconditionSelector`, probe `x3` shows the multi-return signature
does not parse, and probe `x13` shows an admitted route read at a caller's arm.
*Law:* L10, L11, L16. *History:* 6.10, F1 attack 9 and F4 finding 12; 6.9, R1, F4
blocking 2, F3 defects 3 and 9.

**[CALL-6] Publication: how a declared relation becomes a fact.** This is the rule
round 6 found missing, and it is one rule because the design has one publication
surface. Every `Publishes:` line in 3.K names this rule or names [FN-9]'s existing
[ENT-3.S12] route, and nothing else publishes anything.

**[ENT-3] gains one enumerated source, `S13`.**

> **S13 (declaration-domain relations).** At an admitted [BLK-0] kernel-domain or
> [SYS-2] system-domain call, each declared relation of the resolved row is
> **instantiated** by substituting, in each operand, a formal naming a place the row's
> `writes` occurrence covers by that place's **post-state**, every other formal by
> that actual's [MSR-3] **call datum**, and each result operand by its destination
> below. It is **established** on the call's normal continuation, after the call's
> ordinary transfer, consumes, borrow commits, target commit and kills, exactly in
> [ENT-5] 2898-2905's order; a relation routed to a variant is established at entry to
> that variant's selected arm instead. Its **support** is the ordinary L0 support of
> its substituted terms, so a relation over a call datum has empty support and one
> over a post-state place is killed by the next write of that place [MSR-2].

That single sentence is what every proof route in 3.L and in both worked programs
begins with, and the sixth draft named it in an `Amends:` line and stated it nowhere:
`S13` occurred twice in the whole file, both times as a citation, and the "arm route"
the same line named referred to no sentence.

**The destinations, in one list.** A published relation lands on a place, and
[ENT-3.S12] 2833's closed list of four gains these:

```text
a result binder of a destructuring `let`                       [CALL-4]
each target of a [LIV-2] `set`                                 [CALL-4], [LIV-2]
each arm binder of an own-place `match`                        [CALL-4], [OWN-13]
each field binder of a destructuring consume                   [CALL-4], [S13-form]
the resolved place of a `&uniq` state actual, for a relation
  over that state parameter's measures                         this rule
```

**The last destination is the one a provider needs, and it is admitted here and
nowhere else.** A refused `seq_arena` publishes `room(arena) < advance<T>` and a
successful one publishes the cursor's two-sided bound; [RES-6] requires the first,
L8's second half rests on it, and [RES-10]'s route over a store's capacity reads the
second. None of those relations contains a result datum, so [FN-9] 1313 admits none of
them, and [ENT-3.S12]'s four destinations all key on a result, so even an admissible
one had nowhere to land. The admission is stated exactly as wide as [PROV-2]'s own
argument for keeping the provider borrow:

> A relation all of whose operands are measure terms over a **`&uniq` state parameter
> of a declaration-domain row**, together with constants and that row's own call
> datums, is admitted without a result datum and is established on the actual's
> resolved place. **No other relation may omit the result datum**, and **no relation a
> source-declared `fn` writes may be established on a caller's place at all** —
> [MSR-3]'s denotation table makes a `&uniq` parameter's measure inadmissible in an
> `ensures`, and this rule does not reopen it.

Those two sentences are one boundary read from two sides. A compiler-owned row is a
declaration record whose relations are complete over everything it writes [BLK-0], so
a caller reading its post-state is reading a declaration; a wf body is a body, so a
caller reading its post-state would be reading a claim about an object at a point the
callee cannot name, which is L11's second sentence. The cost — a user helper that
lends a provider onward publishes nothing about that store — is [PROV-2]'s and is
recorded in 5.1's Q17.

**And a `replace` publishes nothing.** [SET-2] 528 says its commit "establishes no
fact", and this rule does not change that: a value whose measures must survive is
**constructed into its owner** [MSR-3]'s construct placement, not replaced into it.
Round 6 found `bs_reserve`'s whole `ensures` resting on a plain `replace`, with the
replaced run's own measures dead at the `move`, and 3.L.0 now carries the sentence as
a discipline rule.

*Judgment:* the S13 instantiation and establishment above, at every declaration-domain
call, and the admission test on a relation that omits the result datum, a hard error
citing CALL-6 at the row or clause. This is the judgment [BLK-0], [BLK-2], [BLK-3],
[PROV-2], [PROV-6], [RES-6] and [RES-10] read wherever they publish or consume a
store's or a run's post-state. *Publishes:* the source, the substitution, the
destination list and the support of every declared relation in the language.
*Amends:* [ENT-3] 2730-2837, which gains the enumerated source S13; [ENT-3.S12] 2833's
closed destination list, which gains the four forms and the state-actual place;
[FN-9] 1313, whose result-datum requirement is lifted for exactly the relations named
above and for no others. *Depends:* [ENT-5] 2898-2905, whose establishment order this
source reuses verbatim; [SET-2] 528, whose "it establishes no fact" this rule keeps
true; [MSR-3], which supplies the call datum and the denotation of every operand.
*Law:* L11, L15, L16. *History:* 6.10, F3 DEFECTs 1, 2 and 5.

**[CALL-5] No transport reads the actual's spelling.** The three transports above
are selected by the callee's declared parameter mode and type and by its declared
contract. No rule of this design consults the argument expression's shape, the
callee's body, its name, or any per-parameter summary derived from its body. A
parameter type for which no transport is selected kills conservatively.

**Two rules of this design tested that sentence and both now satisfy it.** [RES-8]'s
saturation flag was defined in the fifth draft as a property of "every acquisition
the function performs, transitively"; it now reads a **declared** row. And [CALL-7]'s
completeness obligation is a **declaration-site** check of a written contract against
a body, exactly as [EFF-2] checks an effect row — the caller reads clauses the writer
wrote, never a summary the compiler derived from a body — which is what keeps this
sentence true of the widest addition this draft makes.

*Judgment:* the conservative default for every unselected parameter type.
*Publishes:* the absence of a call-site-derived fact. *Amends:* [ENT-5] 2876's
clause (b), whose projected-callee-write kill is now classified by [CALL-1..3] and
by nothing else. *Law:* L11. *History:* 6.10, [CALL-7]; 6.9, F2 F5-9.

**[CALL-7] A hand-back contract is complete.** L15's completeness sentence, over the
population [BLK-0]'s cannot reach.

> A source-declared `fn` whose result list contains a result of **measured** type, or
> a measured place reachable from a result by field selection whose descriptor storage
> the body wrote, where that value was **constructed** by the function or **received as
> an `own` parameter and returned**,
> **must declare, for each measure of that result, its exact value or relation to the
> corresponding input measure where the body establishes one, and a two-sided bound
> where it does not.** The declaration is checked against the body by the both-ways
> discipline [EFF-2] already applies to an effect row: a clause the body does not
> establish is a hard error, and a measure the contract does not mention is a hard
> error citing CALL-7 at the `fn_decl`, `IncompleteHandBackContract`, naming the
> result, the measure and the invariant that would carry it. A result the function
> merely forwarded from a callee carries the callee's own published relations and
> needs no clause of its own.

**This is round 6's largest single repair and it is the disease behind five of its
findings.** [BLK-0]'s sentence binds thirteen compiler-owned rows; every function in
3.L and section 4 is a wf function; and the three functions this design prints in full
each failed it in a way that stops a program. `filled` and `vacant` published no
`head`, so **no constructed run could ever be viewed** and every I/O site in every
program was refused at [VIEW-2]. `collect` published no `room`, so **no run could be
appended to twice** and `bs_append_slice`'s loop was undischarged after its first
iteration. `bs_reserve` bounded a plain `u64` payload field instead of
`room(ready.v)`, so 4.2's central call was undischarged. In each case the writer
discovers it at a *caller*, as an [FN-8] rejection naming a goal in the **callee's**
vocabulary, long after the helper compiled cleanly — which is the diagnostic hole F4
ranked most expensive.

**Why a declaration-site obligation rather than a derived summary.** A derived
publication would be a body summary a caller reads, which [CALL-5] forbids and which
is the shape of D1's own flag. Requiring the *writer* to declare it and checking both
ways puts the fact in the signature, where a caller reads a declaration; it makes the
failure land at the declaration rather than at a caller; and it costs nothing a
correct program was not going to state anyway, because a helper whose result a caller
cannot reason about is a helper nobody can call twice.

**Why the measure population and not every result.** A measure is the one class of
fact whose absence silently deletes a caller's ability to *use* the value it was
handed — a subscript, a capacity proof, a view formation. An ordinary integer result
is an opaque value the caller had no fact about before the call either. The obligation
is therefore exactly as wide as the harm.

**What it costs, measured.** In `byte_string.wf` it is two clauses on
`bs_append_slice`, three on `bs_reserve` and two on `collect`, against four
undischargeable call sites without them. Inside a construction loop it costs one
header invariant per measure the function publishes exactly, because [INV-1] 3105
admits four ordered relations and not `==`, so `room(out) + at == before_room` is two
invariants; 5.1's Q14 records that and recommends admitting `==` in a header invariant
over measure terms, which is a change this design does not need and a writer will want.

*Judgment:* the both-ways check of the declared measure relations against the body,
citing CALL-7 at the `fn_decl`, which is the judgment [VIEW-2]'s premise, [CALL-2]'s
transport and every loop in 3.L read at a call site. *Publishes:* every measure of
every handed-back result, as ordinary [FN-9] clauses on the callee's declaration.
*Amends:* [FN-9] 1301-1312's clause list (a completeness obligation over a declared
result), and [FN-1] 1005-1012's boundary list (the obligation is part of the callable
boundary). *Depends:* [EFF-2] 1432's both-ways discipline, which is the check this
rule reuses and the reason a caller reads a declaration; [MSR-5], which is why an
exact relation is one clause. *Law:* L11, L15. *History:* 6.10, F4 finding 1, F1
attack 5, F2 F6-3.

#### 3.K.7 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

```text
| class              | members                                                                        |
|--------------------|--------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context, including every       |
|                    | frame-placed arena [PROV-5] and the release walk's own straight-line frame       |
|                    | cost; every extent-placed arena; every worker-lane stack; the guard-page floor's |
|                    | alternate stack; allocator and runtime metadata; the adapter's persistent        |
|                    | mappings                                                                         |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots;  |
|                    | the runtime's fixed handle table; every other runtime-owned store                |
| host objects       | every countable host object a qualified runtime holds for the program's          |
|                    | duration: a ring file descriptor, a device handle                                |
```

The third class is round 5's and **its membership is round 6's**: `handle(kind, count)`
records that there are three of something and loses every byte, alignment and
dependence on the ring's entry capacity, so a deployment cannot commit them. An `mmap`
is an **extent** and L6 already has a shape for one, so the class boundary is drawn at
*countable versus extent* rather than at *runtime-owned versus program-owned*.

*Judgment:* none; it fixes the domains [RES-3] quantifies over. *Publishes:* the
covered set. *Amends:* nothing. *Law:* L1, L5, L6. *History:* 6.10, F2 F6-12; 6.9, F2
F5-12.

**[RES-2] The envelope `E`, over the target's profile table.** `E = E(P, T, B)` is,
for one program `P`, one selected target and ABI `T` [STOR-6], **and one build
`B`**, a finite table with one row for each lane count `W` the target's runtime
supports. Each row is a finite list of shaped items:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count)                           interchangeable fixed-size records
stack(context, bytes, alignment)             one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
handle(kind, count)                          countable host objects the runtime holds
```

**The `stack` item carries an alignment**, which repairs a place where L6 and [PROV-5]
disagreed: `arena_frame::<65536, 4096, 'a>()` lays a page-aligned extent in the
reserving frame and `stack(context, bytes)` had nowhere to put the 4096, which for the
one use a large frame alignment has — device descriptors — is a correctness failure in
an accepted, marked program. [RUN-4] creates each stack at both figures.

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
its digest. *Amends:* nothing. *Law:* L1, L6. *History:* 6.10, F2 F6-9 and F6-12;
6.9, F2 F5-11.

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
bound**, and premise 3 fails at the loop, the call **or the acquisition** that
introduced it, with that value named. **The acquisition case is round 6's**: a bump
domain's acquire quantity is `advance<T>(count)` and `count` is an ordinary `u64`
operand, so `seq_arena::<u8>(arena: &uniq scratch, count: wanted)` for a runtime
`wanted` fails premise 3 at that statement, in straight-line code, with `wanted`
named. That is this sentence given a home on the row rather than a new restriction,
and it is why a marked program's runtime-sized take is written
`requires count <= k` for a closed `k` and composed at `k`.

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
`E`. *Amends:* [STOR-6] 738-769, whose "the language defines no numeric
per-function frame ceiling" sentence keeps its scope for the *language* and is
joined, for a resource-closed build, by a computed per-context envelope, and whose
target-stage obligations gain `E`-materialization. *Law:* L1, L8, L9. *History:*
6.10, F2 F6-17; 6.8, F2 NB17.

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
them, whose own alternate stack is, for a marked build, a `region` item of `E`
[RES-1].

**One thing the marker no longer selects is whether a program may abort.** The fifth
draft made a cyclic containment graph a premise-3 denial, which is a hard error only
under this marker, so every unmarked program kept the release walk that aborts.
[PROV-6] now refuses that type in every program — and refuses it over the graph the
walk actually follows, so an arena- or frame-backed recursive structure with an empty
walk keeps compiling — and L3's last clause is true rather than aspirational.

A program whose call graph reaches a `Heap<'s>` is not resource-closed, and a
`main` selecting `command.heap` is by itself the rejection. A bounded general store
is still a general store: an envelope item can promise bytes, and cannot promise
that the next contiguous aligned request has a home.

*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage one
is a hard error naming its own cause: the heap-reaching path, rendered from `main`
to the allocation and located at the offending `input_label` or the deepest `call`;
the call-graph cycle [STK-2]; or the unbounded store [RES-5], naming the loop, the
call or the acquisition and the runtime value. *Publishes:* the property as a
compilation fact, and the scope of [SCOPE-3]'s deferrals. *Amends:* [FN-7] 1217,
which fixes main's marker set; [GRAM-2]'s `program_kind` production; and [SCOPE-3]
27-31. *Law:* L1, L6. *History:* 6.9, F2 F5-7.

**[RES-5] Four algebras, and a domain is a store.** Every covered store presents
its state through [MSR-1]'s measures. Exactly four **algebras** are defined, and a
**domain** is one pair (algebra, store identity), where a store's identity is
[PROV-1]'s region for a program store and the profile's row name for a runtime
store. Nothing else is admitted, and a store outside this list contributes no
envelope item and denies [RES-3].

```text
| algebra                    | state         | acquire            | release        | kind        |
|----------------------------|---------------|--------------------|----------------|-------------|
| uniform slots              | len, cap      | +1 record          | -1, on the     | reusable    |
|  (lane, task, queue,       |               |                    | store's own    | capacity    |
|   completion and handle    |               |                    | release event  |             |
|   records of the runtime)  |               |                    | [RES-9]        |             |
| bump extent                | len bounded,  | + advance<T>       | nothing; the   | consumable  |
|  (Arena<'s, bytes, align>) |  in bytes,    |   (bytes)          | store resets   | budget      |
|                            |  cap = bytes  |                    | with 's        |             |
| general heap (Heap<'s>)    | -             | -                  | per run, by    | undecidable |
|                            |               |                    | dispose        | from E      |
| static and frame placement | fixed offsets | none at run time   | none           | decided at  |
|                            |               |                    |                | compile time|
```

**Domain is a store, not a kind.** If a domain were a kind, two arenas in one program
would share one domain, their peaks would add, one arena's reset would be invisible to
the other's accounting, and [RES-10]'s capacity route would have no referent. If it is
a store, a store minted inside a loop body has a domain whose life is one iteration,
which is exactly what makes its reset a zero rather than a mystery. [RES-8]'s map and
its declared saturation fact are keyed by the same pair.

**The `kind` column is not decoration: [RES-10]'s routes read it.** A **reusable
capacity** domain is bounded by what is held at once, so a store's own refusal bounds
it; a **consumable budget** domain is bounded by what is spent and not returned, and a
refusal there is *exhaustion*, not boundedness. Conflating them is the single most
common way to get a wrong answer (L9), and round 6 built the program that does:
a frame arena taken from inside a service loop, whose demand is trips × size and
which the sixth draft's capacity route certified bounded at 65536 while the program
stopped making progress after 256 turns.

**The cleanup-scratch domain is deleted.** A containment level is entered inside a
compiler-derived walk, which is not a statement of [FN-1]'s graph, so [RES-10] has no
site for its transfer; and if its storage is frame-resident it is already inside
`frame(f)`, which [STK-3] measures post-codegen. **A domain exists to carry a runtime
stock; this one has none**, so [RES-1] lists the walk's straight-line frame cost as
execution memory and no fifth algebra exists.

**`advance<T>` is a closed expression, and the store's own alignment is what makes
it one.**

> Every take advances the cursor by `round_up(size_ceiling(T) * count, align)`,
> where `align` is the **store's** own type constant, and both acquiring rows require
> `align >= align_ceiling(T)` as a compile-time comparison of two constants.

The cursor is then a multiple of `align` at every program point, the padding at a
take is zero, the advance is a closed expression in exactly two type-level constants
and one operand, and the run's padding is charged **once** rather than per element.
There is no "otherwise" clause: a requirement and a fallback cannot both govern one
premise. Whether the **operand** is closed is [RES-3]'s question and is answered
there.

*Judgment:* the composition of [RES-10] per domain, over the kind column this rule
fixes. *Publishes:* per program point, per domain, the store's `len` bound; and each
domain's acquire quantity and kind, which [RES-10]'s transfers and routes read.
*Amends:* [OP-9] 974-1003, whose allocation-fit predicate stays and gains the
acquiring rows of [BLK-0] as its callers, whose ceiling table gains Appendix A.1's
derived rows, whose region-bearing exclusion is lifted, and which additionally fixes
`advance<T>`. *Law:* L3, L6, L8, L9, L16. *History:* 6.10, F2 F6-1, F6-17 and F6-19;
6.9, F2 F5-3, F5-6 and F5-16.

**[RES-6] Typed failure, and the two spellings.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume. **The kernel declares no failure
nominal**, because no kernel acquisition takes an affine input: a count is copy and
a provider is borrowed, so `Option<Vector<'s, T>>` carries everything a refusal has
to carry. A library operation that consumes an owner and may refuse declares its
own nominal over its own type; 3.L.5's `Grown` is one.

Each covered-store acquisition with a measure comes in exactly two spellings, on the
model of `+` and `+checked`: a proved form admitted only when [MSR-4] discharges its
goal, and a checked form that is total. **The `Heap` has no proved form** (L6). A
store with measures publishes more: a refused `seq_arena` establishes
`room(arena) < advance<T>`, which is L8's second half — **and which is a fact only
because [CALL-6] gives a provider relation a source and a destination**; without that
rule this sentence, L8's second half and [RES-10]'s capacity route all read a relation
the language cannot establish.

**A library release should be the proved spelling wherever its caller can discharge
it**, and that sentence is here rather than in the library because it is what makes a
directional obligation real. A checked release hands its refusal back as an `Option`,
and a linear value inside one can be legally destructured and discarded — which is
must-consume behaving correctly [PROV-6] and is not must-return. A **proved** release
under `requires room(pool.free) > 0_u64` has no refusal arm at all, so on every path
the value goes back. 4.1 is written on the proved spelling for exactly this reason and
Q0b records what the sixth draft claimed instead.

The runtime's handle table is a covered store, and its refusal joins the **existing**
`IoError` channel: `reserve_file` **gains** `own Result<FilePermit, IoError>` in
place of the total `own FilePermit` [SYS-2] 2261 declares today (owner-decided
2026-09-03, [S25]), and its `Err` edge establishes `room(factory) == Z` when the class
is `ResourceExhausted`. The principle the owner stated with the decision is the one
this whole family is built on: **a failure the environment can produce is exposed as a
typed value; a failure we create ourselves is eliminated, and the type system carries
it.** [SYS-7]'s "the class is the sole portable semantic discriminator" is the reason
no second nominal is added.

**The cost is measured on the right alternative**: a `match` or a `propagate` at
**eleven sites across five corpus programs**, none inside a `propagate` chain today.
The alternative weighed was a total `reserve_file` over a proved capacity, costing
nothing at the call sites and one header invariant per loop; the decision took the
typed value, because the handle table's capacity is the environment's.

No covered-resource failure is a trap, an abort, a process exit, a retry, or a
promotion to a larger store, in the writer's code or in the runtime. The batch-0079
floor's `wf_resource_abort` site loses its **allocation-refusal** caller once
allocation returns a value, and loses its **release-walk** callers once [PROV-6]
refuses a capability-released cycle outright; the doubling-overflow arm goes with the
worklist that needed it.

*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result` or an `Option`,
plus [MSR-4] discharge at the proved spelling. *Publishes:* the returned owner's
identity on the refusal edge, and the store's own refusal relation where the store
has measures, through [CALL-6]'s S13 and its state-actual destination. *Amends:*
[SYS-2] 2261 and 2457, `reserve_file`'s outcome row, which gains a recoverable failure
outcome; [SYS-7] 2473-2486's closed class set, which is **unchanged** and is the
reason no nominal is added; the batch 0079 exhaustion floor as stated above; and
[SCOPE-3] 29, whose "heap exhaustion ... may stop execution at the host boundary
without a Whitefoot value" ceases to be true. *Depends:* [CALL-6], which is what makes
a store's refusal relation a fact. *Law:* L3, L6, L8, L16. *History:* 6.10, F3
DEFECT 2, F2 F6-2 and the owner's S25 decision; 6.9, F2 F5-15.

**[RES-7] What bare resource-closedness does not cover, and where the exclusion is
decided.** Disk space, the successful acquisition of a file, socket or other host
object not exclusively reserved before start, network reachability and throughput,
CPU time, deadlines, scheduler fairness, power, device health, host termination,
and OS quota revocation are outside [RES-1] and outside every judgment in this
file. They remain typed system outcomes where the operation defines one, and
environment conditions where it does not.

**Which store an action acquires from is derived from data the declaration already
carries, and the derivation quantifies over actions rather than over operations:**

> An **action** — a [SYS-2] operation or a [SYS-5] release action — acquires one
> submission record and one completion record exactly when **its declared target
> contract is `may-suspend`**, because [SYS-2]'s own contract says a may-suspend
> action has a logical record that exists before target handoff and a
> `wait-capacity` submission outcome that retains a bundle in the runtime; and
> `reserve_file` acquires one handle record [RES-9].

**Quantifying over actions is round 6's.** [SYS-5]'s release table declares a
`ReadFile` close as `may-suspend; terminal` and the shipping adapter routes it through
the same fixed submission table every read uses, so a marked program opening files in a
loop performed one uncounted submission per handle release and `E` promised a figure
the program exceeds. The column is derived from the **target-contract** column, which
[SYS-2] and [SYS-5] both carry; the sixth draft read one table.

**And the exclusion is split at the stage boundary L1 draws.** The sixth draft's test
read "a store it acquires from has count zero **in the selected row of `E`**" and
issued a **source** rejection from it — while `E`'s `slots` and `handle` figures are
published by the runtime [RUN-2] at step 8 of 3.K.7.1 and the row is selected at step
11, so one program text and one compiler version gave two verdicts on two runtimes.
[RES-7] does not appear at any source step of 3.K.7.1 in that form. The split is:

- **Source stage, step 5.** A marked program's composition publishes, per runtime
  store, the demand it computes [RES-10]. That is a function of program text alone. A
  demand of zero needs no row; a positive demand is a **declared requirement** on the
  runtime and is part of `E`.
- **Qualification, [QUAL-2].** A target whose profile cannot carry a declared
  requirement — a store whose published count is below the demand, zero included —
  fails qualification, stops compilation, and cites no language rule.

The "unavailable operation" message is therefore a target failure naming the store,
the demand and the row, which is what it always described.

*Judgment:* the derived column above, per action, from declared data, which is the
judgment [RES-10]'s may-suspend transfer reads; the composition's per-store demand at
step 5; and no source rejection of its own. *Publishes:* the boundary, and each marked
program's declared runtime-store requirements. *Amends:* [SYS-2] 2164-2307's
declaration records and [SYS-5] 2397-2432's release actions, which gain the derived
column; [QUAL-2] 2369, whose qualification obligations gain an unmet declared
requirement; [ERR-4] 1484, whose "unavailable external resources remain outside the
source outcome model" gains the two families [RES-6] and [RES-4] move inside.
*Depends:* [SYS-2]'s and [SYS-5]'s may-suspend target contracts, which are the data
the column is derived from. *Law:* L1. *History:* 6.10, F2 F6-4 and F6-5; 6.9, F2
F5-5.

**[RES-8] The per-function summary is part of the callable boundary, in three
pieces, and every piece is declared.** Each function's boundary [FN-1] gains three
derived components:

- a **source-stage per-domain map** over that function's formal provider and
  measure terms, substitutable at a call site, keyed by (algebra, store) [RES-5];
- a **declared saturation fact per store region**, written `saturating('s)`
  **[S26]** in the function's contract; and
- a **target-stage own-storage figure** covering every store it reserves [PROV-5]
  and its own frame.

**The saturation fact is declared and not derived, and it is keyed by a store.** The
fifth draft defined it as a transitive body property, which [CALL-5] forbids a caller
to read and which [ENT-1] 2661 forbids twice over by reading which premise discharged
a goal. The sixth draft made it a declaration and keyed it to a **provider
parameter** — and round 6 showed the shape it was built for has none: a library pool
takes `pool: own BlockPool<'s>` and [PROV-2] forbids a provider in a stored position,
so there is no `p` to name. Keying it to the **store region** the signature names
gives it the same key [RES-5] made a domain, and a `BlockPool<'s>` parameter does name
`'s`.

`saturating('s)` says *this function performs no acquisition on `'s`'s store that
could succeed when that store is full*. It is checked **one way** — declared implies
exhibited — and not by [EFF-2]'s set equality, because saturation is a negative
universally quantified property whose "derived set" would be every store the function
does not over-draw, so a both-ways check would force the clause onto every signature
that never touches a store.

A kernel row's own saturation is table data on the row: a **checked** acquisition
spelling is saturating and a **proved** one is not, because a proved acquisition is
one the caller has already bounded by its own [MSR-4] discharge.

The three components are separate because they belong to different stages, and
splitting them keeps [PROV-4]'s framing honest: a self-reserved store contributes to
the third, so [RES-10]'s call rule never meets a callee demand with no actual to
substitute. The map composes across the one closed compilation unit [PROG-1] and no
further.

*Judgment:* the one-way check of each declared `saturating('s)` against the body,
citing RES-8 at the `contract_block`, which is the judgment [RES-10]'s capacity route
reads at a call. *Publishes:* all three components. *Amends:* [FN-1] 1005-1012's
boundary list; [GRAM-2]'s `contract_block` (one added clause form). *Depends:*
[PROG-1] 1492, the one closed unit the composition claim is scoped to; [ENT-1] 2661,
whose "a retained witness changes diagnostic parent choice only, never the derivable
set or acceptance" is why proof provenance may not be read. *Law:* L1, L5.
*History:* 6.10, F2 F6-10; 6.9, F2 F5-9.

**[RES-9] The runtime's own stores, and a release event stated over the record.** A
covered store needs five things written in one place: a **capacity**, an **acquire
event**, a **release event**, a **refusal relation**, and a **multiplicity**. The
program's own stores have all five from [PROV-5], [BLK-2] and [MSR-2]. The runtime's
have them from the profile row and the actions that touch them.

[SYS-10] 2554-2574 **is amended.** Its sentence "Reserving it promises no native
descriptor, **handle-table entry**, kernel memory, or host quota" is replaced by:
*reserving a `FilePermit` consumes one record of a runtime store whose capacity the
target's profile publishes; host exhaustion at the open is a different condition and
remains the ordinary `ResourceExhausted` member of the open operation's typed
`IoError` result, outside `E`.* And its "This first slice never returns or recycles
the permit" is replaced by the release event below.

**The release event is stated over the record, not over three type names.**

> A handle record returns when the value holding it is released, when it is consumed
> by an operation that produces no successor holder, or when the operation it
> authorized returns any outcome that produces no holder. For each covered runtime
> store, the set of acquire sites and the set of release sites must together cover
> every path of every action that touches it, and a target that cannot exhibit
> that coverage fails [QUAL-2].

Stating it as a **closure obligation on the store** rather than as a checklist is
what stops the next open-like operation from extending the enumeration.

[SYS-2] 2283-2285's closed proposition set is **amended too.** It says today that the
only system-result propositions available to source invariants are [SYS-9]'s
enumerated relations and the facts of selecting one typed outcome. The measure
relations of a covered system store join that enumeration as a named source, and
[CALL-6]'s S13 is how they land; without both `cap(factory)` dies at the first
`reserve_file` and no marked program can open a file in a loop.

**The multiplicity is one table per process.** The handle table is per process, not
per context or per lane, and a successor that gives a lane its own table gives it its
own store identity [RES-5] with it.

**The release row's second subject** goes where every other release in this design is
made visible: in the release action's own effect row. [STOR-3] 709-712 already gives
a system resource type "one ordinary state-effect row" for its release action and
already substitutes a formal path for its table-local `owner` subject. A type whose
backing is a covered store names that store in its release row, so `ReadFile`'s
release exhibits `writes(owner)` **and** the runtime handle-table path, named by the
profile rather than by a formal — and, by [RES-7]'s widened quantifier, that action's
own may-suspend records too.

Reclassifying `ReadFile` as linear was considered and refused: its release needs no
capability [PROV-6], so the criterion does not reach it, and marking it would put a
`dispose` on every close site in the corpus and retire the release-completeness
[SYS-5] 2397-2400 grants.

*Judgment:* none by itself; it supplies the fact sources [RES-5] and [RES-10] read,
and its failure is a runtime's [QUAL-2] qualification failure. *Publishes:* each
runtime store's capacity, acquire event, release event, refusal relation and
multiplicity, established through [CALL-6]'s S13 at each action that touches them.
*Amends:* [SYS-10] 2554-2558 (the reservation's promise and the permit's recycling),
[SYS-2] 2283-2285 (the closed proposition set), [STOR-3] 709-712 (the release
contract's second subject), and [SYS-5] 2397-2400's release-completeness, which is
**kept**. *Depends:* [QUAL-2] 2369, which is where a runtime that cannot publish a
capacity fails. *Law:* L1, L3, L5. *History:* 6.10, F3 DEFECT 2; 6.9, F2 F5-4 and
F5-15.

**[RES-10] How `E` is composed.** This is the arithmetic every promise about `E` is
computed by, and round 6 found seven holes in it. It is a rule so that it states a
judgment, publishes a fact, names its law, and is read by the same conditions as
everything else — and every one of the seven is exactly the class §2.1's accounting
sentence was written to catch, which is why the check has to be run **against**
[RES-10] as well as by it.

Every covered resource is one of three kinds, and [RES-5]'s table assigns each domain
its own:

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
has two members no ordinary edge carries.** The labels of a statement are its
fallthrough, each variant of a result it produces, each `break` label it may take,
`propagate`, **`return`**, and **`retained`**.

`return` is round 6's smaller repair. [FN-1]'s graph carries an edge to the
function-return sink, and the sixth draft's label set had no entry for it, so a peak
reached only on a returning path left the map entirely and [RES-8]'s per-function map
inherited the loss — which makes §2.1's own "every edge of the graph carries an entry"
false at that edge. `return` carries what the statement holds on an edge to the sink,
and at a call the callee's `return` entry composes into the caller's map at that
call's own label.

`retained` is round 5's rank-two repair. [STK-4] admits a loop no `break` resolves
to, which is what makes a kernel's idle loop and a driver's service loop entries at
all. Under a label set without it that loop's exit-label set is **empty**, its map has
no entries, and every acquisition the loop performs reaches `E` as nothing at all.

```text
retained   what the statement holds that no edge of it will release: for a loop with
             no fallthrough, the body's own peak composed with its backedge delta
             discharged by the routes below; for every other statement, composed
             through a sequence exactly as `peak` is
```

**Per domain `r` [RES-5], the primitive transfers are stated per algebra rather than
per event**, which is round 6's second repair: the sixth draft fixed
`acquire one (peak 1, delta +1)` for **every** domain while [RES-5]'s bump row is
`len + advance<T>`, so a 256-byte take was charged as one and the design's own
walkthrough computed `+255` on a backedge the table said was `+1`.

```text
acquire           (peak a, delta +a)      on the success exit; (0, 0) on a refusal exit,
                                          where a is the domain's own acquire quantity
                                          [RES-5]: one record for uniform slots,
                                          round_up(size_ceiling(T) * count, align) for a
                                          bump extent
release           (peak 0, delta -a)      the exact inverse, at a dispose or at a store's
                                          own release event
derived release   (peak 0, delta -a)      contributed by a scope-exit edge, per released
                                          value
may-suspend       (peak 1, delta  0)      one submission and one completion record, on the
  action                                  statement or edge that performs the action
                                          [RES-7]; a scope exit carrying k may-suspend
                                          releases is (peak k, delta 0)
reset a store     (peak 0, delta -len(store))
                                          contributed by the release action of a store whose
                                          [RES-5] algebra reclaims with its region [PROV-5]
move an owner     (peak 0, delta  0)      moving into a run acquires nothing
borrow an owner   (peak 0, delta  0)
```

The **reset** transfer is what makes a region block re-entered by a loop compose: its
delta is the exact inverse of everything the block's own map accumulated, so
`delta(region_block) = 0` **falls out of the arithmetic** instead of being asserted in
prose. The **may-suspend** transfer is round 6's third repair, and without it [RES-7]'s
derived column names a store the composition has no site to charge.

A delta may be an integer or an interval `[min, max]`. **An interval enters the
peak equation as its `max` and the delta equation as an interval, and every test
below reads its `max`.** The compositions are:

```text
sequence   when A has a fallthrough exit, for each label L of B:
             peak(A;B)[L]  = max( peak(A)[fallthrough], max(delta(A)[fallthrough]) + peak(B)[L] )
             delta(A;B)[L] = delta(A)[fallthrough] + delta(B)[L]     (interval sum)
           for each non-fallthrough label L of A, A;B carries A's own (peak, delta)[L]
           when A has no fallthrough exit, A;B is exactly A's map and B contributes nothing
           `retained` and `return` compose by the SAME formula as every other label

branch     the union of the arms' maps, keyed by label; two arms reaching one
           label contribute the componentwise max of peak and, when their deltas
           differ, the interval [min, max] of delta

call       substitute the callee's source-stage map [RES-8] at the call site, replacing
           its formal measure and provider terms by the actual ones — EVERY entry,
           `retained` and `return` included — and read its declared saturating('s) for
           the capacity route below

loop       let d be the backedge delta and p one iteration's peak.
             max(d) <= 0  -> peak(loop) = p; delta(loop) = d; no iteration bound is needed
             max(d) >  0  -> the loop is bounded on a domain exactly when its backedge
               delta on that domain is discharged, by the FIRST of the three routes below
               that applies, tried in this fixed order:
                 (i)   a compile-time constant trip count T:
                         peak(loop) = p + (T - 1) * max(d);  delta(loop) = T * d
                 (ii)  a writer [INV-1] invariant over the measure terms of that domain's
                         own store, from which `delta <= 0` on that domain is derivable
                         under [MSR-4]:  peak(loop) = p;  delta(loop) = 0
                 (iii) the domain's kind is REUSABLE CAPACITY, its store's cap is a
                         standing fact [MSR-2], and every acquisition on the loop's paths
                         is saturating, read from the row and from each callee's declared
                         saturating('s) [RES-8]:
                         peak(loop) = cap(store);  delta(loop) = 0
               Otherwise there is no finite E and premise 3 fails here.
           a loop with no fallthrough carries no fallthrough entry and its retained entry
             is p composed with d discharged by the same routes
           each other label of the loop carries the loop's own map, not the map of the
             edge that reaches it

par        a `par` construct occurs in no marked build [RUN-1] and an unmarked program
           carries no `E` promise, so the composition treats a permitted window as the
           ordinary sequence of its members and the overlap's own replicated storage is
           [PAR-3]'s and is an item of no envelope this design promises
```

**Four things about the loop rule are round 6's and each closes a hole.**

**The order is fixed and the routes are tried in it.** The sixth draft said a loop is
bounded "exactly through one of three discharges, and the loop's own map is stated per
discharge", and the three publish *different* maps with nothing choosing — so two
conforming compilers could publish two `E` tables for one program, which L1 forbids in
terms. Trip count first, then the writer's invariant, then the store's capacity,
because each is a tighter bound than the next and a writer who states an invariant has
said which bound they mean.

**Every route discharges the backedge delta rather than naming a level.** The sixth
draft's route over an invariant read "the invariant's own target", and a
`header_invariant` is two affine expressions with no single target; worse,
`invariant held: len(scratch) <= 65536_u64` is **vacuously** provable from [MSR-2]'s
standing `len <= cap` and a type-level `cap`, so it discharged **every** loop over a
store with a type-level capacity, whatever its body did. A peak is a level and a delta
is a net change, and the loop rule's question is whether the *delta* is bounded.
Setting `delta(loop)` to a level also made a balanced loop charge its whole capacity to
everything after it, which is why routes (ii) and (iii) now publish `delta = 0`.

**The capacity route is for reusable capacity only, and this is where round 6's
largest resource finding lands.** A store's own refusal bounds what is **held**; on a
consumable budget a refusal is exhaustion, not boundedness. So:

```wf-design
  region 'a {
    let scratch = arena_frame::<65536, 16, 'a>();
    loop @serve {
      let taken = seq_arena::<u8>(arena: &uniq scratch, count: 256_u64);
      match taken {
        None() => {
        }
        Some(value: staging) => {
        }
      }
    }
  }
```

is **not resource-closed**. The bump domain's backedge delta is `[0, 256]`, `max(d) >
0`, route (i) has no trip count on a divergent loop, route (ii) has no invariant from
which `delta <= 0` follows, and route (iii) does not apply because the domain's kind is
consumable budget. Premise 3 fails at the loop, naming the domain. **Owner-decided
2026-09-03**: an arena or frame take inside a loop that is **not enclosed by a region
block entered and reset per iteration** is charged trips × size, and unbounded — a
divergent loop, or a runtime trip count with no bound — makes the program not
resource-closed. The recommended idiom is the region block **inside** the loop, whose
reset makes the backedge delta exactly zero:

```wf-design
  loop @serve {
    region 'a {
      let scratch = arena_frame::<4096, 16, 'a>();
      let staging = seq_arena_proved::<u8>(arena: &uniq scratch, count: 256_u64);
    }
  }
```

Round 6's F2 read the same program as evidence that **linearity** is keyed to the
wrong question and proposed re-keying it to *returns its backing before its store's
lifetime ends*, with an `abandon` statement to discharge the new obligation. **The
owner refused both.** The criterion stands, frame- and region-reclaimed values are
affine, and the defect is here: an accounting rule that certified an exhausting loop
bounded because it read a *held* bound on a *spent* quantity. Keying linearity to the
store's lifetime would have made every arena value linear, put a `dispose` or an
`abandon` on every scratch value in every goal-A program, and moved a resource
question into the ownership system where L13's own criterion says it does not belong.

**And `retained` composes like the level it is.** The sixth draft gave the sequence
rule a `retained`-specific clause — "the componentwise max of the two retained
entries" — beside a general per-label formula that also applies to `retained`, so the
two disagreed exactly when a statement acquires and falls through: everything a
program acquires *before* entering a divergent loop vanished from the only entry `E`
can read for it, which is the whole pre-loop acquisition of a kernel or a driver. The
special clause is deleted and the general formula applies.

*Judgment:* the composition itself, per domain, over the checked program;
deterministic, ordered and free of search, which is the judgment [RES-3] premise 3 and
[RES-2]'s figures both read. *Publishes:* per statement, per domain, one map from
label to `(peak, delta)`, `return` and `retained` included. *Amends:* nothing in
v0.41; this is new machinery over [FN-1]'s existing graph. *Depends:* [FN-1] 1076's
conservative structural graph as [STK-4] corrects it, which is where the label set
comes from; [RES-5]'s kind column and acquire quantity, which the transfers and routes
read; [CALL-6], without which route (iii)'s `cap(store)` is not a fact. *Law:* L1, L8,
L9. *History:* 6.10, F2 F6-1, F6-7, F6-11, F6-18 and F3 DEFECT 10, and the owner's
accounting ruling; 6.9, F2 F5-2, F5-3 and F5-8.

##### 3.K.7.1 Which stage decides what

```text
 1  tail-SCC rewrite, source premise [STK-1]        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-4]    source stage   compiler
 4  per-function source-stage demand map and
      declared saturation facts [RES-8]             source stage   compiler
 5  loop and branch composition [RES-10], and the
      per-runtime-store declared demand [RES-7]     source stage   compiler
 6  concrete sizes, strides, static image           target stage   compiler
 7  per-context frame envelope [STK-3]              target stage   compiler, post-codegen
 8  runtime profile row for each supported W        target stage   runtime data
 9  matching each declared demand against the
      profile row [RES-7, QUAL-2]                   target stage   compiler
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
qualifies. Steps 11 to 16 decide whether this run is admitted. **Every rule that
issues a source rejection appears at one of steps 1 to 5**, which is the check round
6 ran and which [RES-7] failed in the sixth draft.

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
mutually exclusive, and a writer who needs both writes the loop. And under D1 the
linear clause bites harder in a hosted program than the fifth draft implied, because a
linear binding live at the jump is now common: R3's "two fallbacks" is closer to one
there, and the way back to two is a region block.

*Judgment:* per edge, from the ownership and loan state [LIV-1] and [PROV-6] already
compute; no proof search. *Publishes:* an acyclic call graph, or a component that is
still cyclic, and the strongly connected components [PROV-5]'s activation refusal
reads **after** this rewrite. *Amends:* nothing; this is a lowering and not an
admission rule, so recursion stays permitted. *Depends:* [PROV-6]'s linear predicate
and [LIV-1]'s liveness, which are the judgments the premise reads. *Verified today:*
probes `f2b` and `f8_tailframe` are mutual tail recursions carrying a live borrow of a
caller local and are accepted, so the premise refuses a shape the syntactic list
admitted. *Law:* L7. *History:* 6.8, F2 NB14.

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
excluded from [RES-4] rather than rejected. *Law:* L7. *History:* 6.5, F2-A2.

**[STK-3] The frame envelope, over the whole chain.** For each execution context,
the `stack` item of `E` is measured over the context's **whole chain**, from the
point at which the environment hands that context a stack to the point at which it
takes it back: process entry through `ProgramFinished` for the entry context.
`main`'s own chain is one segment of it, and the runtime's start-up trampoline, its
teardown, its drop glue, the release walk's straight-line frame cost [RES-5] and the
exhaustion floor's own frames are other segments. Within one segment,

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

**The entry context's stack is materialized, not read.** The fifth draft made it
"part of the deployment grant" that [RUN-4] compares;
`compiler/src/backend/wf_floor.c:303-329` shows the shipping floor *creating* it with
`pthread_attr_setstacksize` at `WF_FLOOR_STACK_BYTES` and silently falling back to the
host thread on failure — so the one item [RUN-5]'s theorem is most conditioned on was
being downgraded without a report. [RUN-4] creates it at the figure **and the
alignment** the row names [RES-2] and reports `StartFailed` when it cannot. A **worker
lane's** chain is measured the same way; [RUN-2] fixes `W = 1` for every
resource-closed build, so in this version there is exactly one.

**Two segments the floor holds are items and not chain.** The floor `mmap`s
`WF_FLOOR_ALTSTACK_BYTES` per attaching thread, the entry thread included, and it runs
the entry on a **created** thread while the host thread's own stack stays live. The
alternate stack is a `region` item [RES-1], the created thread's stack is the entry
context's `stack` item, and the host thread's surviving stack is a second context the
row must name or a qualification failure — which is the honest reading of a ledger that
today prints one number for "the entry thread and every worker lane".

`E` is an **output** of code generation, recomputed after every optimization, which
is why [RES-2] makes it a function of the build and carries its digest.
*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic discipline,
which is the judgment [RES-3] stage two reads. *Publishes:* one
`stack(context, bytes, alignment)` item per context per profile row. *Amends:*
[STOR-6] 762-766. *Law:* L5, L6. *History:* 6.10, F2 F6-12; 6.9, F2 F5-11.

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
visible.** A scope whose exit edge is unreachable carries no compiler-derived release
and no [LIV-1] check, so a linear binding live on a path reaching only such a loop is
not an error, and what the loop holds — that binding and every store record its body
acquired and did not release — is the `retained` entry of its own map, which composes
outward by the same formula every other label uses and reaches `E`. No reset runs on
that absent edge either, so nothing observes the retained store.

*Judgment:* [FN-1]'s existing reachability and fallthrough judgment over the
corrected edge set, which is the judgment [RES-10]'s label set reads.
*Publishes:* the graph, hence [RES-10]'s label set.
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

**The no-permission obligation is a build obligation, not a runtime one.** [PAR-1]
permission is a *compile-time* grant to overlap, and a runtime's translation units
contain no record of a permission decision to audit:

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
*Depends:* [PAR-1] 1993, whose unobservability sentence is why the no-permission
obligation is not a rule. *Law:* L3, L5. *History:* 6.10, the [PAR-1] citation
repair; 6.9, F2 F5-12.

**[RUN-2] `par` enters `E` as an open profile, and a marked build publishes
`lanes(1)`.** For each supported lane count `W`, the runtime publishes one finite
profile row. **The row is open, not enumerated**: it publishes one figure per item of
[RES-1] that the runtime owns, enumerated by the runtime and not by this rule, so an
adapter mapping is a `region` item, an alternate stack is a `region` item, and a ring
descriptor is a `handle` item [RES-2]. A rule that enumerated what a profile contains
would be wrong at the next adapter. The number of iterations of a `par`-permitted loop
never appears in `E`, and [RES-10]'s composition names no such number.

What this rule keeps is exactly what **is** a function of program text: **the
profile row a marked build publishes is the `W = 1` row.** Two consequences follow
for free: [PAR-3]'s replicated places, which are execution memory no envelope item
counts, cannot occur in a marked build; and [STK-3]'s worker-lane chain, though now
defined (1.5), has exactly one instance to measure.
*Judgment:* the published-row rule on a marked program, and [RES-3] stage two's
match of each declared demand against the row [RES-7]; the compiler emits no per-`W`
clone. *Publishes:* the `region`, `lanes`, `slots` and `handle` items of each row.
*Amends:* the sentence common to [PAR-1] 1995, [PAR-2] 2030 and [PAR-3] 2055,
"exhaustion of the execution resources an implementation spends on overlapping is a
resource condition under [SCOPE-3] and is not an observable of this rule": for a
program resource-closed on this target that exhaustion is unreachable, because no
`par` construct is emitted [RUN-1]. *Law:* L5, L9. *History:* 6.10, F2 F6-18; 6.9,
F2 F5-12.

**[RUN-3] The parallel footprint of an allocation is its provider place, of a view
its logical origin range, and 1981's intervening list is a footprint property plus
two premises.** In [PAR-1]'s written-footprint clause, "the caller region each
`allocates(arena 'r)` entry names after region substitution" is replaced by "the
places each `allocates` path reaches under the [EFF-2] call-boundary projection",
the same projection the rule already applies to `reads` and `writes`. Two statements
that allocate from one provider therefore conflict, and two that allocate from
distinct providers do not. With [PROV-6] the same is true of two statements that only
dispose, because a `dispose` writes its resolved provider place.

[PAR-2]'s permission for a fill through a `mut_slice` needs two amendments. The
**loan** condition is stated over **iteration-formed** loans: every exclusive loan
formed by a statement of `B` is rooted in a binding `B` introduces, and a loan formed
before `L` on a root every footprint of `B` reaches only through 2005-2008's refined
single-element ranges does not deny. And the **write footprint** of `set m[at] = v;`
contains its origin at the **logical** range `[a*at+b, a*at+b+1)` rather than at whole
place ([PROV-3] use 1), carried to a storage conclusion by [MSR-1]'s injectivity
sentence — which is the sentence the sixth draft left three rules to infer separately.

**[PAR-1] 1981's admitted intervening-statement list becomes the property it is
reaching for, and its two non-footprint premises stay premises.** The sixth draft
replaced the whole sentence with "an intervening statement denies permission exactly
when its footprint conflicts ... and not otherwise", and round 6 found that 1981 ends
with two denials that are **not** footprint conflicts: a statement carrying an exit
edge denies, because [PAR-1]'s own closing premise is that every normal continuation
of `s1` reaches `s2` and a `break` has the empty footprint; and a non-call statement
that forms a borrow denies, because the window's soundness argument assumes no such
loan exists and forming a borrow writes nothing. So:

> The **form** enumeration becomes a footprint property: an intervening statement of
> any form is admitted when its footprint and its loans satisfy this rule's stated
> conditions. **1981's two other denials are kept as the separate premises they
> are**: a statement carrying an exit edge denies permission, and a non-call statement
> that forms a borrow denies permission.

Every new statement form then arrives permitted or denied by its own footprint —
`dispose` [S12], the destructuring consume [S13] and the destructuring `let` of a
multi-result call [S16] all did — and nothing that was denied for a reason other than a
footprint becomes permitted.
*Judgment:* the existing [PAR-1] and [PAR-2] permission judgments, with the form
enumeration replaced by a footprint test, the two non-footprint premises kept, one
added loan clause, and logical ranged origins. *Publishes:* permission. *Amends:*
[PAR-1] 1975 and 1981, [PAR-2] 2000-2033, and [PAR-3] 2035-2063 through their "forms
every footprint exactly as [PAR-1] forms one" clauses. *Depends:* [PAR-2] 2005's
single-binder affine element-write refinement, which is the disjointness argument the
range clause composes with; [MSR-1]'s injectivity sentence, which carries it to
storage. *Law:* L2, L5, L10. *History:* 6.10, F2 F6-8 and F6-14; 6.9, F2 F5-14.

**[RUN-4] The startup protocol.** Program start has four points, and the covered
guarantee spans the last three:

```text
PreStart
    select a row of E from the target's profile table, largest supported W first
    refuse a row whose digest does not match the module being started [RES-2]
    materialize every item of that row:
        commit each region (committed backing, not a reserved address range), at its
            bytes and its alignment
        create each stack, the entry context's own included, at the row's figure and
            alignment
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
`StartFailed` on the first attempt. That is the right behaviour; the descent is real
for an unmarked program, which carries no `E` promise anyway.

*Judgment:* a target obligation, not a source judgment. *Publishes:* the selected
row. *Amends:* [PROG-3] 1505-1539, whose start-time obligation gains the
materialization of `E`, the digest match and the entry-stack creation at its figure
and alignment, and whose `ProgramFinished` boundary is now named. *Law:* L1, L5.
*History:* 6.10, F2 F6-9; 6.9, F2 F5-11.

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
the selected row. *Amends:* nothing. *Law:* L1. *History:* 6.9, F3 I22.

#### 3.K.10 One name per concept

**Every spelling in this table is now the owner's, decided 2026-09-03** (3.S records
each decision with the alternatives weighed). One name per concept, and no concept
with two names.

```text
| concept                    | spelling              | why                                                     |
|----------------------------|-----------------------|---------------------------------------------------------|
| a run of slots, frame-      | FixedVector<T, n>     | the settled name; its capacity is in its type because   |
|   resident [S2]            |                       | layout needs it before the run exists                   |
| a run of slots, store-      | Vector<'s, T>         | one type at two regions; its capacity is a measure      |
|   resident [S1]            |   (brand elided)      | because a growth policy must change it; buffer<T> goes  |
| the store's handle [S3, S4]| Heap<'s>, Arena<..>   | a value you must hold in order to allocate; the         |
|                            |                       | parameter is written at an allocation and nowhere else  |
| the brand's spelling       | written iff the       | 3.K.0's determination principle: decidable from the     |
|                            | operands do not       | declaration text alone                                  |
|                            | determine it          |                                                         |
| build an empty run [S7]    | seq_fixed, seq_arena, | the placement is in the name, because it decides which  |
|                            | seq_arena_proved,     | item of E the run becomes (L6)                          |
|                            | seq_heap              |                                                         |
| reserve a bump store [S9]  | arena_frame,          | as above; nothing else reserves                         |
|                            | arena_extent          |                                                         |
| append at either end [S8]  | seq_place,            | one name per end, whatever the backing                  |
|                            | seq_place_front       |                                                         |
| remove at either end [S8]  | seq_take,             | the window is two-sided, so L12's last clause is true   |
|                            | seq_take_front        |                                                         |
| return a wrapped window    | seq_rebase            | the fifth boundary operation; without it `head` is an   |
|   to its origin [S29]      |                       | absorbing state and a ring is unviewable for life       |
| read a measure [S11]       | len, cap, room, head  | one quantity, one name, term and reader alike           |
| a read-only view [S5]      | slice<'r, T>          | v0.41's own name, kept: the Rust precedent is exact and |
|                            |                       | the semantics do not differ. It is copy [S27]           |
| a writable view [S6]       | mut_slice<'r, T>      | element writes only; affine, because two exclusive      |
|                            |                       | loans on one range are what [OWN-5] refuses             |
| form a view [S10]          | seq_slice,            | the two formers follow the two type names               |
|                            | seq_mut_slice         |                                                         |
| destroy a store-backed     | dispose p;            | one statement, closed under ownership as linearity is;  |
|   value [S12]              |                       | the capability is determined by the brand, not written  |
| take a value apart [S13]   | let N(f: a) = move v; | the inverse of construct, so the closure covers         |
|                            |                       | disassembly too                                         |
| oblige a value to be       | linear struct N {..}  | for a logical obligation only; the storage obligation   |
|   consumed [S18]           |                       | is derived from the type                                |
| write places               | set (p, q) = rhs;     | one commit rule, n-ary: transformation, rebind, swap    |
|                            |                       | and rotation are all this one statement (D2)            |
| a refusal                   | Option<T>             | the kernel consumes no affine input, so it declares no  |
|                            |                       | failure nominal; a library one declares its own         |
| the property [S19]          | resource_closed       | the long spelling is the one in use                     |
| the failure variant field   | Err(error: e)         | [PRE-1] declares Err(error: E)                          |
```

`FixedRing`, `PoolVector`, `HeapVector`, `ArenaVector`, `AppendView`, `absorb`,
`update`, `seq_frame`, `seq_exchange`, `swap`, `Span`, `MutSpan`, `HeapBox`,
`ArenaBox`, `PoolSlot`, `heap_take`, `arena_take`, `pool_take` as a kernel row,
`Full<T>`, `TooSmall`, `OutOfMemory`, `PoolExhausted`, `NeedCapacity` and `NoRecord`
are **not** in the kernel vocabulary. The first four are library names for kernel
types (3.L.1); `update` and every swap spelling are [LIV-2]; `seq_frame` and
`seq_exchange` are the fifth and sixth drafts' removals; `Span` and `MutSpan` are the
sixth draft's names for `slice` and `mut_slice` and are gone; the three box and slot
names are runs of capacity one or library nominals; the `*_take` operation names
belong to the library's own functions; and the last six are library nominals a writer
declares over their own type.

**One naming consequence is recorded rather than hidden.** The owner adopted S10's
two operation names as proposed, and separately decided that the two view types keep
`slice` and gain `mut_slice`. `seq_span` and `seq_mut_span` would then be formers
named after a type the language does not have, which is the one defect this table
exists to prevent, so they are written `seq_slice` and `seq_mut_slice`. That is a
consequential re-spelling of an adopted name and not a new proposal, and 6.10 records
it as such.

#### 3.K.11 Amendment register

**This register is a collation of the `Amends:` and `Depends:` lines of every rule
in 3.K, and it carries nothing else.** It was written last, from the rules. It
covers 3.K only: 3.L amends nothing, because it is ordinary wf, and 3.S records
decisions rather than amending rules.

Seven conditions make it checkable rather than remembered, and each is a defect of
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
   row and there only**; a third-list row for the same sentence is redundant and is
   removed. Round 6 found thirteen sentences on both lists, which is over-reporting
   and is still the register contradicting its own condition;
5. **an `Amends:` line must state a change for every sentence in its cited range
   that the amending rule's own body contradicts**;
6. every `*Publishes:* X on Y` names the [ENT-3] source that establishes X and the
   destination clause that puts it on Y. **[CALL-6] is that rule**, and a
   `Publishes:` line with no source or no destination is the same defect as an
   `Amends:` line with no row. Round 6 found four `Publishes:` lines publishing a
   store's post-state measure with neither; and
7. **every fact a rule states appears in that rule's `Judgment:` or `Publishes:`
   line, and every rule that reads such a fact names the judgment it comes from.**
   This is round 6's condition. All four of its memory-soundness BREAKS are one
   shape — a rule that states a property it does not judge, and a second rule that
   reads the property as a premise — and this condition catches three of the four
   mechanically. `[CALL-3]`'s `Depends:` line named `[VIEW-4]`, and `[VIEW-4]`'s
   `Judgment:` did not contain the thing depended on.

**Changed.** Line numbers are `spec/kernel-spec.md` **v0.41** at 30602914,
**re-derived mechanically in this session**: every rule's first and last non-blank
line was extracted from the file and every cited range now ends on its rule's last
content line. Round 6 found six wrong citations and fifteen ranges ending on a blank
line, thirteen of them one line past their rule; the six are corrected here and named
in 6.10, and the ranges below are the extracted ones. Each row's `by` column names the
rules whose `Amends:` lines reach it; a row that also records a surviving depended
sentence marks it **bold** (condition 4).

```text
| rule            | line      | change                                                          | by                          |
|-----------------|-----------|-----------------------------------------------------------------|-----------------------------|
| [SCOPE-3]       | 27-31     | heap exhaustion leaves the deferred set; stack and covered-store | [RES-4], [RES-6], [RUN-2]   |
|                 |           | exhaustion leave it for marked programs                          |                             |
| [FORM-2]        | 39-89     | +4 renderings: result list, destructuring let and consume, set   | [CALL-4], [LIV-2], [PROV-6] |
|                 |           | target list and value list, dispose, the linear modifier         |                             |
| [GRAM-2]        | 168-202   | result list; resource_closed; region_params on nominals; the     | [CALL-4], [RES-4], [BLK-4], |
|                 |           | linear modifier; a saturating clause; requires/ensures (185-186) | [MSR-5], [PROV-6], [RES-8]  |
|                 |           | take a clause_expr                                               |                             |
| [GRAM-3]        | 204-215   | box/arena/buffer productions retire; runs are ordinary TYPEIDs   | [PROV-1], [VIEW-1]          |
|                 |           | with targs; slice is joined by mut_slice                         |                             |
| [GRAM-4]        | 217-256   | destructuring let and consume; comma return; set target list and | [CALL-4], [LIV-2], [MSR-5], |
|                 |           | value list; affine_factor GAINS terms; stmt gains dispose        | [PROV-6]                    |
| [GRAM-5]        | 258-280   | +clause_expr; atom and atom_list untouched                       | [MSR-5]                     |
| [GRAM-9]        | 328-332   | unchanged; named because [MSR-5] moves the amendment away        | [MSR-5]                     |
| [GRAM-11]       | 345-352   | a fourth callee class in all three sentences                     | [BLK-0]                     |
| [TYPE-2]        | 357-360   | +5 nominals (2 providers, 2 runs, mut_slice); box/arena/buffer   | [PROV-1], [BLK-1], [BLK-2], |
|                 |           | retire; the flat-element restriction is not inherited            | [VIEW-1]                    |
| [TYPE-5]        | 370-394   | the written-argument criterion covers a fourth callee class and  | [BLK-0]                     |
|                 |           | becomes per-argument. **379 survives and [PROV-1] depends on it; |                             |
|                 |           | 383-386's mandatory construct arguments survive and 3.K.0 puts   |                             |
|                 |           | a construct outside its criterion for exactly that reason**      |                             |
| [TYPE-6]        | 396-473   | the domain's spellings, nominals and region parameters; 401's    | [BLK-0], [MSR-6]            |
|                 |           | callee IDENT admission; 401's pbase gains a const generic        |                             |
| [TYPE-7]        | 475-479   | the deref domain becomes the two borrow modes alone              | [PROV-1]                    |
| [SET-1]         | 481-511   | loan-strength target traversal; SUBSUMED by [LIV-2] as its n=1,  | [PROV-3], [LIV-2]           |
|                 |           | copy-target case                                                 |                             |
| [SET-2]         | 513-528   | region-bearing rejection replaced by [PROV-3] use 3 and          | [PROV-3], [LIV-2], [VIEW-4] |
|                 |           | [VIEW-4]; its exchange exception to [OWN-5] 591 is inherited by  |                             |
|                 |           | [LIV-2]. **528's "it establishes no fact" survives UNCHANGED     |                             |
|                 |           | and [CALL-6] depends on it: a replace is a kill, never a         |                             |
|                 |           | publication**                                                    |                             |
| [CONST-2]       | 546-561   | its naming of buffer and slice_of follows the retirements        | [VIEW-1]                    |
| [OWN-1]         | 563-571   | 563 gains mut_slice as affine and MOVES slice to copy; linear    | [PROV-6], [VIEW-1],         |
|                 |           | refines affine; 569 gains the partial-consume refusal and        | [LIV-1], [LIV-2]            |
|                 |           | dispose as a consuming use; 566-567 is REPLACED by [LIV-2]'s     |                             |
|                 |           | commit premise                                                   |                             |
| [OWN-4]         | 582-583   | the lent-onward child's loan ends at its receiving statement     | [PROV-7]                    |
| [OWN-5]         | 585-611   | origins generalize to loan-bearing values, carry a logical range | [PROV-3], [VIEW-2]          |
|                 |           | and are copied with a copy view; two ranged access clauses; the  |                             |
|                 |           | address-computation freeze; 601 and 608 restated. **606 survives |                             |
|                 |           | and [VIEW-2] and [PROV-6] depend on it; 591 is outside this      |                             |
|                 |           | range and [LIV-2] and [PROV-6] depend on it**                     |                             |
| [OWN-6]         | 613-627   | a child reborrow may name a caller-supplied region under the     | [PROV-7]                    |
|                 |           | result-type condition, for every reborrow. **614 survives and    |                             |
|                 |           | [PROV-2] and [VIEW-2] depend on it**                             |                             |
| [OWN-7]         | 629-633   | 629's overlap test extends to logical ranges. **630's subscript  | [PROV-3]                    |
|                 |           | conservatism survives and [PROV-3] use 2 depends on it** (4a)    |                             |
| [OWN-10]        | 640-644   | 643's arena content clause becomes one over Vector content.      | [PROV-1]                    |
|                 |           | **641 survives and [PROV-2] depends on it** (4a and 4b)          |                             |
| [OWN-11]        | 646-648   | 646's move prohibition is replaced by [LIV-1]'s join agreement;  | [LIV-1]                     |
|                 |           | 647's loop-body region restriction is vacuous once a borrow      |                             |
|                 |           | expression names no region (3.K.0), which is why 4.1's `&task`   |                             |
|                 |           | and `&held` need no inner block                                  |                             |
| [STOR-1]        | 675-683   | 675's storage-class list gains the two runs; the writable-place  | [PROV-1], [LIV-2]           |
|                 |           | partition (678-679) becomes [SET-1]/[LIV-2] write and [SET-2]    |                             |
|                 |           | replace, with 679's diagnostic kept for a live affine target     |                             |
|                 |           | whose right-hand side does not consume it; 681's growable        |                             |
|                 |           | paragraph and 682's arena-index-pool and keyed-collection        |                             |
|                 |           | rejections are superseded by the library, which recycles values  |                             |
|                 |           | without recycling slots                                          |                             |
| [STOR-2]        | 685-686   | box_new and arena_new retire; a store take is a kernel row       | [PROV-2]                    |
| [STOR-3]        | 688-719   | a linear type has no derived release; the box and buffer HEAP    | [PROV-5], [PROV-6], [RES-9] |
|                 |           | rows retire, so derived release covers exactly region-end and    |                             |
|                 |           | frame reclamation and the system-resource release; the store     |                             |
|                 |           | reset joins the table and 690's edge enumeration gains the       |                             |
|                 |           | propagate error edge; 709-712 gains a second subject.            |                             |
|                 |           | **699-705's drop order survives and [PROV-6] reuses it** (4a)    |                             |
| [STOR-4]        | 721       | confinement becomes the outlives relation over the region set    | [BLK-4]                     |
| [STOR-5]        | 723-736   | the position list becomes the three-way intensional split; the   | [BLK-4], [PROV-2]           |
|                 |           | per-leaf-provenance deferral is withdrawn as unnecessary         |                             |
| [STOR-6]        | 738-769   | E-materialization joins the target-stage obligations; 762-766's  | [RES-3], [STK-3]            |
|                 |           | frame sentences gain the per-context envelope                    |                             |
| [OP-1]          | 771-849   | +cap, +room, +head, pure, over runs, views and providers; five   | [PROV-2], [BLK-0], [BLK-2], |
|                 |           | constructors retire; ReservedLowerNames +3; 838 gains the class  | [VIEW-1]                    |
| [OP-4]          | 914-924   | indexable bases extend to the runs and views; the obligation is  | [BLK-1], [MSR-1]            |
|                 |           | against len, in logical coordinates; a subscripted measure place |                             |
|                 |           | in an erased clause discharges at its own attach site            |                             |
| [OP-5]          | 926-931   | "and contract predicate" narrows to a source condition           | [MSR-5]                     |
| [OP-7]          | 939-947   | slice_of retires; cap, room and head join the structural         | [VIEW-1]                    |
|                 |           | operations                                                       |                             |
| [OP-9]          | 974-1003  | the ceiling table gains A.1's derived rows, the region-bearing   | [RES-5], [BLK-0]            |
|                 |           | exclusion is lifted, advance<T> is fixed, and the predicate      |                             |
|                 |           | gains [BLK-0]'s acquiring rows as its callers                    |                             |
| [FN-1]          | 1005-1091 | the view ceiling and its duplicate-result rejection; an ordered  | [VIEW-6], [CALL-4],         |
|                 |           | result list; the &uniq referent refusal in a parameter list;     | [CALL-7], [RES-8], [STK-4], |
|                 |           | four boundary components; a loop_stmt's normal-successor edge    | [BLK-4]                     |
|                 |           | (1076). **1041-1047 survives and [PROV-3] depends on it**        |                             |
| [FN-2]          | 1093-1100 | the rejection narrows to loan-bearing and provider arguments;    | [BLK-4], [BLK-0]            |
|                 |           | explicit instantiation covers nominals and the kernel domain     |                             |
| [FN-3]          | 1102-1147 | the allocation component becomes the set of allocates paths      | [PROV-4]                    |
| [FN-7]          | 1216-1255 | command.heap; resource_closed; 1218's "declares no region        | [PROV-1], [RES-4]           |
|                 |           | parameters" is KEPT; allocates over a labelled input;            |                             |
|                 |           | 1245-1246's byte sequence gains the row                          |                             |
| [FN-8]          | 1257-1299 | clause operands are a clause_expr; 1267 becomes a GoalTemplate-  | [MSR-5]                     |
|                 |           | formation sentence. **1275 survives and [MSR-3] depends on it**  |                             |
| [FN-9]          | 1301-1367 | terms as operands; measured and multi-ordinal results; variant   | [MSR-3], [MSR-4], [MSR-5],  |
|                 |           | routes carrying an ordinal binder; result field projection;      | [CALL-4], [CALL-6],         |
|                 |           | multi-datum clauses; 1313's result-datum requirement is lifted   | [CALL-7]                    |
|                 |           | for a declaration-domain relation over a &uniq state parameter   |                             |
|                 |           | and for nothing else; a &uniq parameter's measure is             |                             |
|                 |           | inadmissible in an ensures; the entry datum replaces 1316;       |                             |
|                 |           | 1345's M(c,q) admits a datum; 1357's narrow direct-set route is  |                             |
|                 |           | subsumed; a completeness obligation over a handed-back measured  |                             |
|                 |           | result. **1312's closed compare_op set is what [MSR-5] reuses**  |                             |
| [EFF-1]         | 1369-1390 | allocates takes formal-rooted paths; heap and arena retire;      | [PROV-4], [PROV-3]          |
|                 |           | 1386 generalizes to a loan-bearing parameter, which [CALL-3]     |                             |
|                 |           | and [VIEW-7] depend on (4a). **1369's canonical order (reads,    |                             |
|                 |           | writes, allocates) survives UNCHANGED and every row of A.2,      |                             |
|                 |           | 3.L and §4 is written in it; 1389's both-categories sentence     |                             |
|                 |           | survives and [PROV-4]'s allocating row reads it**                |                             |
| [EFF-2]         | 1392-1439 | the slice projection generalizes; 1427 stays TRUE for the        | [PROV-3], [PROV-6],         |
|                 |           | actions that survive and is joined by the disposal walk's        | [CALL-7]                    |
|                 |           | contribution and the set commit's read and write. **1432's       |                             |
|                 |           | both-ways discipline survives and [CALL-7] reuses it**           |                             |
| [ERR-3]         | 1472-1482 | the retained judgments gain the live-linear-binding refusal      | [PROV-6]                    |
| [ERR-4]         | 1484-1490 | the deferral gains the two families that move inside. **1487     | [RES-7]                     |
|                 |           | survives and [PROV-5] depends on it**                            |                             |
| [PROG-3]        | 1505-1539 | PreStart materializes E at each item's figure and alignment,     | [RUN-4]                     |
|                 |           | matches its digest and creates the entry stack; ProgramFinished  |                             |
|                 |           | is named                                                         |                             |
| [DIAG-1]        | 1541-1883 | rank 5 covers the kernel domain; +container_declaration_ordinal  | [BLK-0]                     |
| [PAR-1]         | 1975,1981,| the provider-place projection (1975); the intervening-statement  | [RUN-3], [RUN-2], [PROV-6]  |
|                 | 1995      | FORM list (1981) becomes a footprint property while 1981's       |                             |
|                 |           | exit-edge and borrow-forming denials are KEPT as premises;       |                             |
|                 |           | dispose enters a footprint; 1995's exhaustion sentence is        |                             |
|                 |           | unreachable when marked. **1993 survives and [RUN-1] depends     |                             |
|                 |           | on it**                                                          |                             |
| [PAR-2]         | 2000-2033 | iteration-formed loans; a view's ranged write footprint in       | [RUN-3], [RUN-2]            |
|                 |           | logical coordinates; the element-write form. **2005 survives     |                             |
|                 |           | and [RUN-3] depends on it**                                      |                             |
| [PAR-3]         | 2035-2063 | the exhaustion sentence; replicated places cannot occur marked   | [RUN-3], [RUN-2]            |
| [SYS-1]         | 2136-2162 | a fourth admitted declaration source                             | [BLK-0]                     |
| [SYS-2]         | 2164-2307 | views at the range-bearing operations; a derived "acquires from" | [VIEW-7], [RUN-1], [RES-6], |
|                 |           | column over its target-contract column; 2261's reserve_file      | [RES-7], [RES-9], [CALL-6]  |
|                 |           | gains a recoverable outcome; 2283-2285's proposition set gains   |                             |
|                 |           | covered-store measures; its rows publish through S13. **2270 is  |                             |
|                 |           | kept and [RUN-1] reads it**                                      |                             |
| [SYS-3]         | 2309-2313 | the kernel domain is admitted to every unit                      | [BLK-0]                     |
| [SYS-5]         | 2397-2432 | release-completeness (2397-2400) is KEPT; the release action     | [RES-9], [RES-7]            |
|                 |           | (2407-2428) gains the handle-table subject; its target-contract  |                             |
|                 |           | column is a second source of [RES-7]'s derived column            |                             |
| [SYS-7]         | 2473-2486 | the class set is UNCHANGED, which is why no nominal is added     | [RES-6]                     |
| [SYS-8]         | 2488-2527 | the seven range-bearing operations take mut_slice and slice      | [VIEW-7]                    |
| [SYS-9,11,12,14]| 2529-2646 | their prose naming buffer<u8> is restated over views             | [VIEW-7]                    |
| [SYS-10]        | 2554-2574 | a reservation consumes a runtime record with a published         | [RES-9]                     |
|                 |           | capacity, and the record returns by a stated closure             |                             |
| [QUAL-2]        | 2369-2382 | +three failures: an unmet declared runtime-store requirement, an | [RES-7], [RUN-1], [RES-3]   |
|                 |           | emitted module containing par in a marked program, and a         |                             |
|                 |           | runtime that cannot exhibit release coverage.                    |                             |
|                 |           | **2369's own sentence survives and [RES-9] depends on it** (4a)  |                             |
| [ENT-2]         | 2677-2728 | measure terms over a subscriptable place; +the measure datum;    | [MSR-1], [MSR-3], [LIV-2],  |
|                 |           | a set target resolving to no binding is a declaration event; a   | [MSR-2], [MSR-6]            |
|                 |           | const generic is admitted at an endpoint; +standing facts.       |                             |
|                 |           | **2681 clause (c) and 2693 survive and [MSR-6] and [MSR-3]       |                             |
|                 |           | depend on them**                                                 |                             |
| [ENT-3]         | 2730-2837 | +the enumerated source S13 and its four parts; S5 gains the      | [CALL-6], [BLK-0], [MSR-3], |
|                 |           | construct, rebind, payload and field placements; S6 generalizes  | [CALL-4], [LIV-2]           |
|                 |           | over four measures; S12's destination list gains four forms and  |                             |
|                 |           | the state-actual place                                           |                             |
| [ENT-5]         | 2863-2967 | descriptor-storage support; the effect-row kill; 2893(a) LOSES   | [MSR-2], [MSR-3], [CALL-5], |
|                 |           | its element-position carve-out; the datum and the denotation     | [CALL-6]                    |
|                 |           | table replace the call-boundary and 2887-2891 paragraphs;        |                             |
|                 |           | clause (b) is classified by [CALL-1..3]. **2898-2905's           |                             |
|                 |           | establishment order survives and [CALL-6] reuses it; 2942-2946   |                             |
|                 |           | survives and [MSR-2] and [MSR-3] depend on it**                  |                             |
| [ENT-6]         | 2969-3100 | one goal disposition; measures carry images; 3007 gains          | [MSR-3], [MSR-4], [MSR-2]   |
|                 |           | len + room = cap as two members; the four per-family route       |                             |
|                 |           | grants keep their normalization and lose their route grant.      |                             |
|                 |           | **3015's two-premise family and 3024's determinability sentence  |                             |
|                 |           | survive UNWIDENED, which is why [BLK-0] and [CALL-7] and not     |                             |
|                 |           | [MSR-4] carry the arithmetic repair**                            |                             |
| [INV-1]         | 3101-3156 | 3105's relation restriction is reused by [MSR-5]; 3109-3113's    | [MSR-3], [MSR-5], [MSR-6]   |
|                 |           | atom admission gains terms, named consts and const generics,     |                             |
|                 |           | and [MSR-3]'s atom-identity sentence. **3105 survives and        |                             |
|                 |           | [MSR-5] depends on it**                                          |                             |
| batch 0079      | docs/done/| the abort site loses its allocation caller and its release-walk  | [RES-6]                     |
| exhaustion floor| 0079-...  | callers, and the doubling-overflow arm with them                 |                             |
```

**Depended on and unchanged.** Each row is the collation of one or more `Depends:`
lines, and each names the rule that depends on it. A later batch changing one of
these sentences changes a rule of this design without touching it. **A dependency
that falls inside changed text, or that names a retired subject, is on its changed row
above and is not repeated here** (condition 4); round 6 found thirteen such sentences
on both lists and they are deduplicated.

```text
| rule       | line | the sentence, and who depends on it                                       |
|------------|------|---------------------------------------------------------------------------|
| OWN-3      | 578  | region identifiers are unique within a function: [PROV-1], which is why a  |
|            |      | store region's spelling denotes one store                                 |
| OWN-3      | 580  | distinct caller-supplied regions are incomparable and every ordering rule  |
|            |      | fails closed: [PROV-1] and [BLK-4], the whole invariance argument          |
| OWN-12     | 650  | region substitution controls type equality: [PROV-1], which is why two     |
|            |      | stores are distinguished by their types                                   |
| OWN-13     | 654  | an own-place match moves the scrutinee and binds payloads own: [PROV-6],   |
|            |      | why a match is a destructuring, and [MSR-3]'s payload placement            |
| FN-6       | 1211 | recursion is permitted: [STK-2], which excludes a program from [RES-4]     |
|            |      | rather than rejecting it                                                  |
| PROG-1     | 1492 | one closed compilation unit with no function values: [PROV-4]'s exact      |
|            |      | reachability closure and [RES-8]'s composition claim                       |
| ENT-1      | 2661 | a retained witness changes diagnostic parent choice only, never the        |
|            |      | derivable set or acceptance: [RES-8], which is why saturation is declared  |
| ENT-4      | 2860 | L0's uniqueness and finiteness rests on the difference-bound shape:        |
|            |      | [MSR-2], which is why len + room = cap is an affine premise                |
```

**META-5 delta**, declared here because the register is its natural home. Numbered
language rules: 131 today, plus the 51 of 3.K, none reusing a live or retired id; the
region-spelling amendment (3.K.0) is counted with its own batch and not here. Unique
fixed lowercase grammar atoms: minus 5 for the retired `heap` and `arena` effect atoms
and the retired `buffer` and `box` type productions and `slice_of` (`arena` is one
atom serving both a production and an effect entry, and retires once), plus 5 for
`resource_closed`, `dispose`, `linear`, `saturating` and `mut_slice`; net zero. **The
`using` atom the sixth draft counted is gone with the `using` list.** Grammar
productions: plus 2, being `clause_expr` and `dispose_stmt`; changed, 10, being
`let_stmt`, `return_stmt`, `set_stmt`, `result_binding`, `program_kind`,
`struct_decl`, `enum_decl`, `contract_block`, `effect`, `affine_factor`, with
`requires_clause`/`ensures_clause` counted once as a pair. **Statement forms** — a
different count from productions, which round 6 noted the sixth draft conflating —
plus 1, `dispose_stmt`; the destructuring consume is a `let_stmt` alternative and the
set target list is a changed `set_stmt`. `ReservedLowerNames`: plus 3, `cap`, `room`
and `head`. Nominal types: plus 5, being 2 providers, 2 runs and `mut_slice`; `slice`
is unchanged. Declaration domains: plus 1, with one
`container_declaration_ordinal`. Entry input rows: plus 1. Compound punctuation
tokens: unchanged. [SYS-2]'s normative inventory counts change with [VIEW-7], [RES-6],
[RES-7] and [RES-9] and are recomputed when those rules are written into the spec, not
asserted here.

**Retired outright, with no successor.** The fourth draft's five owner types
([BLK-1]); its `AppendView`, `absorb` and the abandoned-window disposition; its
`update` statement and its three atoms; its `Pool` store, `PoolSlot`, `PoolVector`,
`seq_lease`, `pool_frame`, `pool_extent`, `pool_take`, `pool_release` and the pool
seam; its `FixedRing` and four ring rows; its `HeapBox` and `ArenaBox`; its three
failure structs and its `NoRecord`; its `seq_filled`, `seq_vacant`, `seq_take_at`,
`seq_clear`, `seq_truncate`, `seq_reserve_heap`, `seq_reserve_arena`, `seq_shrink`,
`seq_heap_filled`, `seq_push`, `seq_try_push`, `seq_pop` and every `try` row; the
`&uniq buffer<T>` and `&uniq Container` prohibition **[CNT-7], whose effect [BLK-4]'s
fourth clause now restores as a rule**; the effect-row atoms `heap` and `arena`;
`slice_of`, `box_new` and `arena_new`; the first draft's `Builder<'r, T>` and `[BLD]`;
`[CNT-5]`; L14; the fifth draft's `seq_frame` and `seq_exchange` rows, `[CALL-4]`'s
exit datum and `[MSR-3]`'s exit placement; and **this draft's own three**: the sixth
draft's `[LIV-3]`, merged into `[LIV-2]` by D2; its `dispose ... using (...)` list,
removed because the capability is determined by the brand; and its `Span`/`MutSpan`
type names. **Every rule id in that list is retired and none is reused.** The second
draft's reentrancy *premise* was deleted from a live rule and no id was retired with
it, which is the sentence round 6 found the sixth draft getting wrong: `[STK-4]` is a
live rule of this draft and was never a retired id.

**Writer doctrine this design invalidates**, which `docs/patterns.md` must carry in
the same batch. **P16** ("One length fact above the writes") rests on hoisting a
length above a sequence of `&uniq` callee writes; [BLK-4] refuses the parameter it
hoists across, so the pattern is rewritten over `&uniq mut_slice<u8>`, where [CALL-3]
keeps the fact for the reason P16 states, and over the value-in / value-out form,
where the fact is the result's. P16 gains a second correction from [MSR-2] — a length
fact survives a write to a **sibling field**, which probe `r2_4` shows today's compiler
killing. **P17**'s field-by-field fold is **narrowed** to non-linear aggregates,
because [PROV-6] refuses a partial consume of a linear one, and its `replace` note
gains [LIV-2]'s one commit rule. **P19** is unchanged and gains a case: a measure term
joins by the same delta-atom rule. **P15** is unchanged and both worked programs
follow it. **P8** should gain what probes `q5'`, `m10` and `x1b` bought: an exact `-`
or `+` carries an ordering into a backedge where the wrapping form gives the checker a
fresh atom. **Seven new patterns are owed**: structural disposal with no capability
written, the linear destructuring consume, the `propagate`-free allocating helper,
3.L.3's two-invariant construction loop plus its `flat` invariant, the value-in /
value-out helper whose contract is complete over every measure it hands back, the
element borrow that hoists a window's modulo out of a descriptor loop (probe `x10`
shows it unsupported today), and the checked release that a caller proves away.

---

### 3.S Surface decisions

**This section was a proposal table for five drafts and is now a decision record.**
On 2026-09-03 the owner decided every language-surface addition this design rests on,
and the rules of 3.K use those spellings as **decided**. Each entry below states the
spelling, why the kernel needs it, why no wf program has its effect, at least two
alternatives with their costs, and the decision. **Three entries are still open**, and
each is one this draft added after the owner's decisions: S28, S29 and S30 are marked
**PROPOSED** and are the only things in this file a reader should not read as settled.

```text
| id  | spelling                                    | kind                    | status   |
|-----|---------------------------------------------|-------------------------|----------|
| S1  | Vector<'s, T>, brand elided                 | compiler-owned nominal  | ADOPTED  |
| S2  | FixedVector<T, n>                           | compiler-owned nominal  | ADOPTED  |
| S3  | Heap<'s>                                    | compiler-owned nominal  | ADOPTED  |
| S4  | Arena<'s, bytes, align>                     | compiler-owned nominal  | ADOPTED  |
| S5  | slice<'r, T> keeps its v0.41 name           | naming decision         | ADOPTED  |
| S6  | mut_slice<'r, T>                            | compiler-owned nominal  | ADOPTED  |
| S7  | seq_fixed, seq_arena, seq_arena_proved,     | operation names         | ADOPTED  |
|     |   seq_heap                                  |                         |          |
| S8  | seq_place, seq_place_front, seq_take,       | operation names         | ADOPTED  |
|     |   seq_take_front                            |                         |          |
| S9  | arena_frame, arena_extent                   | operation names         | ADOPTED  |
| S10 | seq_slice, seq_mut_slice                    | operation names         | ADOPTED  |
| S11 | cap, room, head                             | operation names         | ADOPTED  |
| S12 | dispose p;                                  | statement form          | ADOPTED  |
| S13 | let N(f1: b1, ..., fk: bk) = move v;        | let alternative         | ADOPTED  |
| S14 | (retired into D2)                           | -                       | DECIDED  |
| S15 | (retired into D2)                           | -                       | DECIDED  |
| S16 | -> (a: own T, b: own U), let (a, b) = ...,  | result list and its     | ADOPTED  |
|     |   return e1, e2;                            |   binding and return    |          |
| S17 | clause_expr over measure terms              | grammar production      | ADOPTED  |
| S18 | linear struct N { ... }                     | declaration modifier    | ADOPTED  |
| S19 | resource_closed command fn main             | entry marker            | ADOPTED  |
| S20 | struct N['s] { ... }                        | region params on a      | ADOPTED  |
|     |                                             |   nominal               |          |
| S21 | a const generic as a value, an endpoint     | resolution admission    | ADOPTED  |
|     |   and a clause operand                      |                         |          |
| S22 | command.heap as heap: own Heap              | entry input row         | ADOPTED  |
| S23 | allocates(path)                             | effect production       | ADOPTED  |
| S24 | ensures when b is V(f: r): ... over any     | contract routes         | ADOPTED  |
|     |   variant and any result ordinal            |                         |          |
| S25 | reserve_file -> own Result<FilePermit, ..>  | system-row change       | ADOPTED  |
| S26 | saturating('s)                              | contract clause         | ADOPTED  |
| S27 | slice<'r, T> is copy; mut_slice is affine   | ownership class         | ADOPTED  |
| S28 | on_propagate { ... }                        | scope section           | PROPOSED |
| S29 | seq_rebase                                  | operation name and row  | PROPOSED |
| S30 | the seven [SYS-8] range-bearing operations  | system-row change       | PROPOSED |
|     |   take mut_slice and slice                  |                         |          |
```

**D1 and D2, the two decisions that removed entries rather than adding them.** `[S18]`
is adopted **together with derived linearity**, so `[PROV-6]` states one criterion and
one modifier and the writer marks only a logical obligation. `[S14]` and `[S15]` are
**retired into one commit rule**: `[LIV-2]` replaces `[SET-1]`'s copy overwrite, the
sixth draft's reinitializing `set` and its in-place exchange with one n-ary statement,
so the multi-target form and the exchange admission are no longer two additions but one
rule that is smaller than the three it replaces. Their ids are retired and not reused.

**S1-S2, the two run nominals.** *Needed because* a run of initialized slots with a
checker-maintained boundary is the one thing 1.4's criterion says a writer cannot
express: `array<T, n>` requires `n` live values, which for affine `T` is exactly what
the writer does not have. *Alternatives:* (a) do not add them — the language keeps
`buffer<T>`, which is heap-only, has no affine element domain (probe `p9`), and cannot
carry a store brand, so goal A has no container and D1's repair has nothing to be
stated over; (b) one nominal with capacity always a measure — costs `FixedVector`'s
layout-before-existence property, so no run is frame-resident; (c) keep `buffer<T>`'s
spelling for the store-resident one — a rename saved, a reader's expectation broken.
*Decided:* adopted as `Vector<'s, T>` and `FixedVector<T, n>`, with `buffer<T>`
retired.

**S3-S4, the two provider nominals.** *Needed because* L2 makes a store a value a
program must hold, and no wf declaration can produce an unforgeable one: a writer's
`struct Heap {}` is constructible. *Alternatives:* (a) do not add them — allocation
stays ambient, probe `p5_ambient` stays accepted, and goals A and B both fail at their
first sentence; (b) one provider nominal with a kind field — costs the type-level
distinction that makes `allocates(env.heap)` a heap-reaching row and an arena's `cap` a
type constant. *Decided:* adopted.

**S5-S6, the two views.** *Needed because* [SET-1] 488-490 makes every slice-rooted
target unwritable, so no writable view exists and a system operation cannot fill a
caller's run without taking the run itself; probe `p7` is the refusal.
*Alternatives:* (a) do not add `mut_slice` — every element-writing helper takes the
run by value and returns it, which is correct but forces a copy-out/copy-in discipline
on I/O and deletes [CALL-3], the third of the owner's three call rules; (b) rename
`slice` to `Span` and add `MutSpan`, which the fifth and sixth drafts did — costs a
corpus-wide rename and buys nothing, because the semantics do not differ from Rust's
`&[T]`. *Decided:* `slice<'r, T>` keeps its name; the writable view is
`mut_slice<'r, T>`; **rename only where semantics differ.**

**S7-S11, the operation and reader names.** *Needed because* each moves a
checker-maintained boundary, mints a store, forms a view, or reads a measure no run
exposes; [BLK-3]'s own text shows the one row that *was* expressible (`seq_exchange`)
leaving under L18. *Alternatives:* (a) do not add them — as S1-S4; (b) a different
scheme, `run_*` or an associated-name form — costs nothing this design depends on,
since no rule reads a name; (c) fold the front operations into the back ones with a
direction argument — costs a runtime branch on a compile-time constant in the one loop
a driver cares about. *Decided:* adopted. **One consequence**: S10's two names were
proposed as `seq_span`/`seq_mut_span` and the view types kept `slice`/`mut_slice`, so
the formers are spelled `seq_slice` and `seq_mut_slice`; that is a consequential
re-spelling of an adopted name, recorded in 3.K.10.

**S12, `dispose p;`.** *Needed because* a store-backed value's release requires a
capability [PROV-6], and no wf statement performs a **structural walk** of a type the
writer did not declare, releasing every capability-released leaf to the store its own
type names. **The `using (...)` list the sixth draft carried is removed, and what the
list held was capability *values* — a `Heap` or an `Arena` binding — never regions and
never the things being released.** It is removed because the capability is
**determined**: identity and permission are different things, the brand in `p`'s type
gives the identity, one store has one provider, and at any program point at most one
live binding can lend `&uniq` to it, so under [FORM-1] there is nothing to write
(3.K.0). The release still *spends* the capability — the statement writes the resolved
provider binding, that write is in the effect row, `par` sees it, and there is no
ambient heap. *Alternatives:* (a) do not add the statement and reuse `move` into a
compiler-owned release operation, `let done = heap_release(heap: &uniq heap, run: move
p);` — no new statement and no new atoms, at the cost that the walk must then be
per-type, so a writer disposing a `Bytes` calls one operation per leaf and a nested
aggregate is a hand-written traversal that gets it wrong silently when a field is
added; (b) region-end reclamation only, with no per-value release — the heap then has
no release operation and goal B becomes a program that cannot free; (c) keep the
`using` list — costs one written binding per disposal for a value the rule can resolve,
and makes the same statement legal or illegal according to which of two names for one
store the writer chose. *Decided:* adopted as `dispose p;`.

**S13, the destructuring consume.** *Needed because* linearity is closed under
ownership, so a linear aggregate must be takeable apart in one statement that leaves no
residual; without it [PROV-6]'s partial-consume refusal has no mechanical fix and a
slab free list has no spelling. *Alternatives:* (a) do not add it — a linear aggregate
can only be moved or disposed whole, so a writer who needs one field out of a linear
record cannot get it; (b) `let N { a, b } = move v;`, brace-form — a readability
judgment, where the paren form is `construct`'s exact inverse and [GRAM-8]'s field-name
discipline carries over; (c) an own-place `match` with one arm — already legal for an
enum ([OWN-13] 654), so for enums the alternative *is* the answer and only structs need
the form. *Decided:* adopted.

**S16, the ordered result list.** *Needed because* value-in / value-out (R1) makes
every transforming operation return the value it was handed plus what it computed.
**Its L18 status is recorded honestly**: a two-field struct per operation is writable
in wf, so this half of [CALL-4] is admitted on cost and not on expressibility — the
kernel would grow one nominal per row, and every hand-back helper in every program
would declare one. [CALL-4]'s other two halves (the per-variant route and the S12
destination clause) are what no wf program has. *Alternatives:* (a) do not add it — a
nominal per operation; (b) a prelude `Pair<A, B>` — one nominal instead of many, at the
cost of a `match` or two field reads at every call site and of losing the per-ordinal
contract route S24 needs. *Decided:* adopted.

**S17, `clause_expr` over measure terms.** *Needed because* [GRAM-5] 269's `atom` has
no `call` alternative, so `len(source) <= room(out)` derives nowhere; probe `e1` is the
parse rejection. *Alternatives:* (a) keep hoisting into `define` — legal today and it
works for a *parameter*, but a `define` is erased into the clause by alpha-expansion, so
it cannot name a **result**'s measure at all (probe `x2`), which is exactly what R1's
contracts and [CALL-7] need; (b) extend `atom` with a call alternative — reaches far
more, since `atom` occurs in argument lists, subscripts and infix operands, so it would
admit nested calls everywhere and [GRAM-9]'s three-address discipline would go with it.
*Decided:* adopted.

**S18, the `linear` modifier.** *Needed because* the capability criterion sees storage
obligations and not logical ones, and a writer cannot write *a type whose silent drop
is refused*: every wf mechanism for it is a runtime field a program can forget to read.
*Alternatives:* (a) derive linearity entirely from types — the storage half still
works, and a library pool's lease is affine, a dropped lease loses a block, and nothing
reports it in the fact state or in `E`; (b) `must_consume` as the spelling — the same
semantics under a name that says what it does rather than what class it joins, and it
avoids overloading a word the derived predicate also uses; (c) make the pool a kernel
store, so its lease is linear by the criterion — restores the fourth draft's `Pool`,
`PoolSlot` and six operation rows, which the minimality ruling removed. *Decided:*
adopted as `linear`, **together with derived linearity** (D1). What it buys is stated
in [PROV-6] and 3.L.7: must-consume, visibly. A **directional** obligation is bought by
proving the return, not by the modifier.

**S19, `resource_closed`.** *Needed because* the judgment must be a compile error for
the program that asks for it and a note for every other, and no wf declaration changes
the severity of a compiler judgment. *Alternatives:* (a) do not add it — `E` is
computed and reported and never enforced; (b) a compiler flag — makes acceptance a
function of the invocation, which [SCOPE-2] 18 and L1 forbid. *Decided:* adopted.

**S20, region parameters on a nominal.** *Needed because* a store's identity is in the
type [PROV-1] and a nominal holding a store-backed value must name that store; probes
`r2_6` and `m05` are the parse errors today. *Alternatives:* (a) do not add it — no
nominal may hold a store-backed value, so `Bytes`, `BlockPool`, `Chunk` and every
library structure are unwritable; (b) infer the nominal's region from its fields — an
inference [TYPE-5]'s statement-local discipline forbids, and it makes two
instantiations of one nominal silently different types. *Decided:* adopted.

**S21, a const generic as a value.** *Needed because* every capacity-parametric
function reads its bound as a value, a loop endpoint or a clause operand; probes `t1`,
`t2` and `t3` are the three rejections and `t4` shows a named const already works in
all three. *Alternatives:* (a) named consts only — every parametric function written
once per capacity, about forty-three bodies for fourteen algorithms; (b) admit it as a
clause operand only — closes the contract half and leaves the loop and value halves.
*Decided:* adopted.

**S22, `command.heap`.** *Needed because* the heap must enter as a value [L2] and the
entry table [FN-7] 1227 is closed. *Alternatives:* (a) do not add the row — there is no
heap value, so goal B has no honest allocation story; (b) add the row **and** give
`main` a region parameter so a signature can name the entry heap — costs one more
declaration on the entry and one more spelling in every signature that relates two
positions to the entry store. *Decided:* **the row is adopted and `main` declares no
region parameter.** The entry heap's region is the elided default at every stored
position, no signature names it, and a helper that must relate two of its own positions
to one store binds its own region name for them exactly as any other helper does.

**S23, `allocates(path)`.** *Needed because* [EFF-1] 1369's fixed atoms cannot name a
provider a function received as a field of an aggregate, so [PROV-4]'s closure is
inexact exactly where a program threads an environment struct. *Alternatives:* (a) do
not change it — heap-reachability is computed per region name, which is sound but
refuses a program holding two arenas in one record; (b) keep the atoms and forbid a
provider in an aggregate — enforceable, and it costs the `Env`-struct pattern P5
teaches. *Decided:* adopted, in [EFF-1] 1369's canonical order.

**S24, per-variant and per-ordinal contract routes.** *Needed because* [FN-9] 1307
admits exactly `when Ok(value: r):` over `Result<int, E>`, so no library constructor
can publish a fact about what it built and no fallible helper can publish that it
succeeded; probes `x1` and `x2` are the rejections and `x13` is an admitted route read
at a caller's arm. *Alternatives:* (a) do not add them — every capacity proof collapses
into the function that owns the run and every helper boundary costs a statically-true
runtime branch; (b) the variant route without the projection — leaves a constructor
unable to publish; (c) a witness integer beside every result, legal today — one result
per fact. *Decided:* adopted, **with the ordinal binder** [CALL-4] requires, because a
variant route over two same-typed enum results is otherwise ambiguous in three ways and
one of them is unsound.

**S25, `reserve_file` becomes fallible.** *Needed because* the handle table is a
covered store with a finite capacity [RES-9], and L3 requires its refusal to be a
value. *Alternatives:* (a) do not change it — the store has no refusal edge, so a
marked program that opens files in a loop either cannot be accepted or is accepted with
a promise the runtime cannot keep; (b) a total `reserve_file` over a proved capacity —
costs nothing at the eleven corpus call sites and one header invariant per loop, which
is less than the `match` this adds. *Decided:* adopted, on the principle the owner
stated with it: **a failure the environment can produce is exposed as a typed value; a
failure we create ourselves is eliminated, and the type system carries it.** The handle
table's capacity is the environment's, so its refusal is a value.

**S26, `saturating('s)`.** *Needed because* [RES-10]'s capacity route must compose
across a call, and the fact it needs — *this function performs no acquisition on `'s`'s
store that could succeed when that store is full* — is a property of a body, which
[CALL-5] forbids a caller to derive. *Alternatives:* (a) do not add it — the route does
not compose, so a retaining loop is refused the moment its acquisition is one function
down; (b) derive it from the body, as the fifth draft did — makes [CALL-5] false; (c)
infer it from the callee's `allocates` row — a path says which provider is reached,
never which spelling was used. *Decided:* adopted, **keyed to a store region and not to
a provider parameter**, because the shape it was built for — a library pool taken by
value — has no provider parameter at all.

**S27, the shared view is `copy`.** *Needed because* [OWN-1] 564 makes `slice` affine
and affinity there buys nothing: duplicating a shared view is a second **shared** loan
on the same range, which [OWN-5] admits without limit, and a loan-bearing value owns
nothing [PROV-3], so a copy can double-free nothing. *Alternatives:* (a) leave `slice`
affine — costs a re-formation, a fresh borrow and a `seq_slice` call at every second
use, in the middle of loops that had one; (b) make **both** views copy — unsound, since
two exclusive loans on one range are exactly what [OWN-5] 606 refuses. *Decided:*
adopted: `slice<'r, T>` is copy, `mut_slice<'r, T>` is affine, exactly as Rust's
`&[T]` and `&mut [T]` are.

**S28, `on_propagate { ... }`. PROPOSED — added by this draft after the owner's
decisions.** One section per scope, whose ordinary written statements run on every
`propagate` error edge leaving that scope, before the enclosing scope's own section, in
[STOR-3]'s derived-drop order, checked per edge by exactly [LIV-1]: at every
`propagate` in the scope the live linear set must be exactly the set the section
discharges, and a mismatch is a hard error naming the binding and the site. *Needed
because* [PROV-6] refuses a `propagate` while a linear binding is live, D1 makes that
refusal common, and the mechanical fix does not compose: round 6 measured eleven lines
at indent two becoming forty-six at indent twenty-two for five error exits (probes `w6`,
`w7`), and `tests/programs/raw_deflate_dynamic_decode.wf`'s `decode_dynamic` is 214
lines with seven `propagate` sites and three live heap runs, which becomes seven
hand-maintained cleanup lists that differ per site because by line 296 one of the runs
has been moved into a callee and disposing it is a use-after-move. No wf construct runs
statements on an edge the writer cannot name. *Alternatives:* (a) leave the refusal, as
this design does today — ships a language in which the corpus's own `propagate` chains
do not compile, and pushes the `Heap` parameter into functions that neither allocate
nor release; (b) a per-statement release list, `let x = propagate e disposing (heap);`
— repeats the cleanup once per `propagate`, which is six of the seven copies kept, and
it is the shape 5.1's Q10 recorded; (c) admit a `propagate` while a linear binding is
live and derive the release — makes the release conditional on a runtime edge, which
L17 forbids. *Cost if adopted:* one grammar production, one [LIV-1] per-edge check, and
a section a reader must find; the section is ordinary code, so no compiler-derived
action and no drop flag is introduced. *Recommendation:* adopt with D1 rather than
after it, because D1 is what makes the refusal common. 5.1's Q10 is the owner's
question.

**S29, `seq_rebase`. PROPOSED — added by this draft.** One added [BLK-3] row,
`seq_rebase(vector: own V) -> own V`, publishing `head(result) = 0_u64` with `len`,
`cap` and `room` unchanged and requiring nothing; its lowering is a rotate in place.
*Needed because* without it `head` is an **absorbing state**: A.1 makes it the one
bounded cell, no other row republishes `head = 0`, so after one `seq_take_front` a run
can never satisfy [VIEW-2]'s premise again — not after a drain, not after a refill — and
every transmit path over a ring owes a permanent second run of full capacity plus an
O(n) copy per flush. It is unwritable in wf for exactly the reason the other four
boundary rows are: it moves a checker-maintained boundary. *Alternatives:* (a) do not
add it, and weaken [VIEW-2]'s premise alone — a drained ring becomes viewable, which is
useless, and a ring that has wrapped while holding data stays unviewable; (b) publish a
conditional `head(rest) = 0` on `seq_take_front` when the run becomes empty — a
two-armed relation the affine domain does not carry, and it still leaves a wrapped
non-empty ring stranded; (c) keep the permanent staging run — costs a full second
buffer per viewed ring in `E`, which is the one figure a marked driver is sized against.
*Cost if adopted:* one row, one more line in A.2, and an O(len) operation a writer
chooses. *Recommendation:* adopt; it is the smallest of the three open items and it
removes a cost the window's own justification did not price.

**S30, the seven [SYS-8] range-bearing operations over views. PROPOSED — added by this
draft.** `read_at`, `write_once` and the five others take `&uniq 'd mut_slice<'r, u8>`
for a destination and `&'s slice<'r, u8>` for a source in place of `buffer<u8>`
[VIEW-7]. *Needed because* it is goal A's container half: without it a heap-free
program cannot do I/O, since `buffer<u8>` is heap-only. It carries no id in the sixth
draft, which is why it is listed now — S25 gives one system-row change its own entry and
this one changes seven. *Alternatives:* (a) do not change them — a marked program has no
I/O at all; (b) take the destination `own` and hand it back — correct under R1 and it
deletes the loop of reads a caller can write into one destination, because an `own`
destination is consumed by the first call; (c) take the run itself rather than a view —
reintroduces the `&uniq` container parameter [BLK-4] refuses. *Cost if adopted:* seven
signature rows, [SYS-2]'s normative counts, and the prose of four [SYS] rules.
*Recommendation:* adopt with S6, because it is the only reason S6 exists.

### 3.L The library, written in wf

#### 3.L.0 How to read this section

Everything below is **ordinary wf**, written against 3.K and against the unchanged
v0.41 rules. It defines no rule, amends no rule, and is named by no rule. It exists
to discharge L18's obligation: an item the kernel no longer carries is written out
here, or the kernel lacked a primitive and 3.L.6 says which. Every spelling it uses is
one 3.S records as decided, except the three 3.S marks PROPOSED.

Each item states its **proof route** — which kernel rule discharges each obligation,
and which of those v0.41 already proves today, naming the probe where one exists. The
code is design text; the standard it is held to is that every statement is accepted by
a compiler implementing 3.K and the unchanged v0.41 rules.

Six discipline sentences are stated once here rather than repeated, and each is a
falsifier finding about this section rather than about the rules:

- **Every body is three-address.** `let mirror = count -wrap 1_u64 -wrap at;` is two
  operations in one expression and is a [GRAM-4] parse error (probe `t13`); [GRAM-6]
  282 says composition is by `let`.
- **`Z` is the term language's zero and appears only in rule prose.** wf source and
  every inventory row write `0_u64`; probe `t11` is the [GRAM-5] rejection of the other
  spelling.
- **An effect row is written in [EFF-1] 1369's canonical order, `reads`, `writes`,
  `allocates`.** [FORM-1] 35 admits one legal byte sequence, so the sixth draft's
  `reads, allocates, writes` in six rows of this section and of the appendix was a hard
  error beside the correct order in the same file. This is the second byte-level slip
  in two drafts and both were introduced by a repair.
- **A measure read is `pure` at the operation and an ordinary `reads` at the
  caller**, so a helper that reads `len` of a borrowed run names it in its row (probe
  `t10`), and a helper that declares a row its body does not exhibit is refused the
  same way. [EFF-2] 1432 admits no wider and no narrower declaration.
- **A `replace` is a kill and never a publication.** [SET-2] 528 says its commit
  establishes no fact, and [CALL-6] keeps that true. A value whose measures must survive
  is **constructed into its owner** through [MSR-3]'s construct placement, not replaced
  into it. The sixth draft's `bs_reserve` rested its whole `ensures` on a plain
  `replace` whose replaced run's own measures were already dead at the `move`.
- **A writer's generic over an element type cannot serve a copy and an affine
  instantiation from one body** — probes `x14` and `x15` show one rejected at `u8` and
  accepted at `box<u64>` — so a function that *reuses* or *moves* a `T` is written per
  element class and says so. That is Q8, not a partition finding. Capacity genericity is
  available: [MSR-6] makes a const generic a value.

**And one obligation this section is now checked against**: [CALL-7]. Every function
below that hands a measured value back declares every measure of it. That is what the
sixth draft's three printed functions did not do, and it is why none of their callers
compiled.

#### 3.L.1 The owner names

`FixedVector<T, n>` is the kernel type and needs no library. `HeapVector<T>` and
`ArenaVector<'a, T>` are what a writer *calls* a `Vector<'s, T>` whose store is the
heap and a named arena respectively; they are one kernel type at two regions and the
library adds nothing to them (footnote 1). Under 3.K.0 a heap run in a stored position
is written `Vector<u8>` and an arena run `Vector<'a, u8>`, which is the whole visible
difference between them. **A ring is not a library type at all**: under [BLK-1]'s
window a ring is a `FixedVector<T, n>` used from both ends, so `FixedRing` has no
successor rather than a library one (footnote 2).

#### 3.L.2 The partition, item by item

Every item is written in wf in `CONTAINERS.md` §3 against 3.K and against the
unchanged v0.41 rules, with its proof obligations walked there. This table is the
result; the items §4 calls are written out below, because a worked program may not
call a function this file does not declare.

```text
| item                          | written as                          | route, and what discharges it       |
|-------------------------------|-------------------------------------|-------------------------------------|
| FixedVector<T, n>             | the kernel type itself              | nothing to write                    |
| HeapVector, ArenaVector       | Vector<'s, T> at two regions        | nothing to write                    |
| a ring, a queue, a deque      | a run used from both ends [BLK-1]   | nothing to write; no Option, no tag |
| vacant<T, const n>            | a counted loop of seq_place over    | three header invariants; the exit   |
|                               | None<T>(), 3.L.3 below              | ordering, not an equality; x1c, x1d |
| filled<T, const n>            | the same, reusing one copy value    | as above; per element class (Q8)    |
| the transposition of one      | seq_take, one element replace,      | three statements; 3.L.2 below, and  |
|   element with the last       | seq_place                           | its requires is at + 2 <= len       |
| take_at                       | the transposition, then seq_take,   | the requires plus a dominating      |
|                               | with a branch for the last position | branch at the last position         |
| clear, truncate               | a counted drain, two invariants     | as vacant; a linear T disposes each |
|                               |                                     | and the signature says so [PROV-6]  |
| growth policy, HeapVector     | seq_heap, drain from the front,     | seven invariants; the window is what|
|                               | append at the back, construct,      | makes order preservation free;      |
|                               | dispose                             | 3.L.5 below                         |
| block pool with a lease       | linear struct Lease['s] plus a      | a branch on len and on room, which  |
|                               | FixedVector<Vector<'s,u8>, m> free  | needs [ENT-3.S6] over four measures;|
|                               | list, and a PROVED release          | 3.L.4 below                         |
| collect and the appenders     | a counted loop, value in and value  | five invariants and a complete      |
|                               | out, 3.L.3 below                    | hand-back contract [CALL-7]         |
| keyed families                | vacant plus element replace         | [OP-4] from the requires; x7        |
| try_place, try_take           | a branch on room or len and two     | [ENT-3.S6] again; 3.L.4 below       |
|                               | returns                             |                                     |
| update p by op(...)           | set p = op(vector: move p, ...)     | [LIV-2]                             |
| update p by op(...) into x    | set (p, x) = op(vector: move p,...) | [LIV-2], the n-ary case             |
| swap two whole places         | set (p, q) = move q, move p;        | [LIV-2]; no operation exists        |
| OutOfMemory<T> and its family | an ordinary one-field struct over   | [BLK-4] admits it; the kernel needs |
|                               | the writer's own type               | none                                |
```

**The transposition, written out, because it is the fifth draft's removal and the
sixth draft priced it wrong.** `seq_exchange` was a kernel row; it is three statements
over rows the kernel already has:

```wf-design
fn take_at<T, const n: u64>(vector: own FixedVector<T, n>, at: own u64)
    -> (rest: own FixedVector<T, n>, taken: own T)
    reads(vector), writes(vector) contract {
  requires at + 2_u64 <= len(vector);
  ensures len(rest) + 1_u64 == len(vector);
  ensures room(rest) == room(vector) + 1_u64;
  ensures cap(rest) == cap(vector);
  ensures head(rest) == head(vector);
} {
  doc "Removes the element at at, moving the last element into its place.";
  let (short, endv) = seq_take(vector: move vector);
  let old = replace short[at] = move endv;
  return move short, move old;
}
```

**What it costs, priced against a program that compiles.** The `replace` at `short[at]`
carries [OP-4]'s `at < len(short)`, and `seq_take` published
`len(short) = len(vector) - 1`, so the caller must prove `at + 2_u64 <= len(vector)` —
**not** `at + 1_u64 <= len(vector)`, which is what the sixth draft wrote and which over
`u64` is the same proposition as `at < len(vector)`, one unit short of the obligation
the body carries. The consequence is real: this form cannot address the **last**
position, where the transposition is the identity, so a caller that may remove the last
element writes a dominating branch on `at + 2_u64 <= len(vector)` and a plain `seq_take`
on the other arm. And the three statements kill and re-establish `len` twice where one
row would have published one relation. That is a proof-surface cost a writer pays for a
capability the kernel does not owe them, it is what L18's last sentence now requires a
removal to state, and if the owner judges it too high the row comes back.

#### 3.L.3 Construction and appending, written out

`vacant` is the more interesting because round 3 concluded no loop could publish
`len = n`; it is right that no loop publishes the *equality*, and wrong that the
equality is what a subscript needs.

```wf-design
fn vacant<T, const n: u64>() -> result: own FixedVector<Option<T>, n> pure contract {
  ensures len(result) >= n;
  ensures cap(result) == n;
  ensures room(result) <= 0_u64;
  ensures head(result) <= 0_u64;
} {
  doc "Builds a run of n slots, every one holding None.";
  let built = seq_fixed::<Option<T>, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: len(built) >= at,
    invariant spare: room(built) + at >= n,
    invariant flat: head(built) <= 0_u64
  ) {
    let empty = None<T>();
    set built = seq_place(vector: move built, value: move empty);
  }
  return move built;
}
```

**Proof route.** `seq_fixed` publishes `len(built) = 0`, `cap(built) = n`,
`room(built) = n` and `head(built) = 0` — all four exactly, which is [BLK-0]'s
completeness sentence doing the work round 5 found missing. `grown`'s base is `0 >= 0`;
`spare`'s is `n + 0 >= n`; `flat`'s is `0 <= 0`. `seq_place`'s own requirement
`room(built) > 0` discharges from `spare` and the counted loop's `at < n`
([ENT-3.S11]) by [MSR-4] step 5. On the backedge `seq_place` declares
`len(result) = len(vector) + 1`, `room(result) = room(vector) - 1`,
`cap(result) = cap(vector)` and `head(result) = head(vector)`, over that call's own
datum, which has empty support [MSR-3] and which reaches `built` through [CALL-6]'s S13
and [CALL-4]'s `set`-target destination; each invariant is preserved by **one**
published premise, which is what puts the derivation inside [ENT-6] 3015's two-premise
budget. Probe `g4` is that shape accepted at v0.41 scale and probe `g3` is the same
shape rejected when the relation is missing. The `set` target names a binding in scope,
so it keeps its term [LIV-2] and the three atoms survive. At the exit `at = n`, so
`len(built) >= n`; `cap` is `seq_fixed`'s standing constant; `room <= 0` follows from
`len >= n`, `cap = n` and [MSR-2]'s identity; and `flat` exports `head(built) <= 0`.

**`flat` is round 6's, and without it nothing built by a loop can ever be viewed.**
[BLK-1] said the `head = 0` chain is "exact equalities" that state nothing — true
inside straight-line code and false across a backedge, where [ENT-5] 2942-2946 removes
every fact whose support the body writes. Every `seq_slice` and `seq_mut_slice` in every
program in the sixth draft was undischarged for that reason. One invariant, one clause,
base and backedge each one published premise; a run that is never viewed omits both.

`n` is read as a loop endpoint, which is [MSR-6] and probe `t2`'s rejection today.
`vacant` is generic over `T` with no copy bound, because `None<T>()` is built fresh
each iteration. `filled` is not, because it reuses one `value`:

```wf-design
fn filled<T, const n: u64>(value: own T) -> result: own FixedVector<T, n> pure contract {
  ensures len(result) >= n;
  ensures cap(result) == n;
  ensures room(result) <= 0_u64;
  ensures head(result) <= 0_u64;
} {
  doc "Builds a run of n slots, every one holding a copy of value.";
  let built = seq_fixed::<T, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: len(built) >= at,
    invariant spare: room(built) + at >= n,
    invariant flat: head(built) <= 0_u64
  ) {
    set built = seq_place(vector: move built, value: value);
  }
  return move built;
}
```

Same route, and it is written for a **copy** `T` only: the bare `value` use is
[OWN-1] 564's copy-on-use, and at an affine instantiation the same body needs `move`
and would consume it on the first iteration. That is Q8 and 3.L.0 states it once.
This is the function [VIEW-7] needs for an addressable I/O destination.

**`collect`, the one program every draft has carried.**

```wf-design
fn collect['s](out: own Vector<'s, u8>, source: own slice<u8>)
    -> (rest: own Vector<'s, u8>, written: own u64)
    reads(out, source), writes(out) contract {
  requires len(source) <= room(out);
  ensures written == len(source);
  ensures len(rest) == len(out) + written;
  ensures room(rest) + written == room(out);
  ensures cap(rest) == cap(out);
  ensures head(rest) <= 0_u64;
} {
  doc "Appends every byte of source into the destination's spare room.";
  let count = len(source);
  let before = len(out);
  let before_room = room(out);
  for @copy (
    at in 0_u64..count,
    invariant grown_lo: len(out) >= before + at,
    invariant grown_hi: len(out) <= before + at,
    invariant spare_lo: room(out) + at >= before_room,
    invariant spare_hi: room(out) + at <= before_room,
    invariant flat: head(out) <= 0_u64
  ) {
    let byte = source[at];
    set out = seq_place(vector: move out, value: byte);
  }
  return move out, count;
}
```

`collect` writes **one** region name, `'s`, at its binder and at the two positions
whose store must be the same one; `source`'s loan region relates nothing and is
elided, and so is `'s` at every call site, because the `out` operand determines it.
One written identifier per hand-back helper is R1's spelling cost and it is the whole
of it.

**Proof route, and what [CALL-7] costs here.** `let count = len(source);`,
`let before = len(out);` and `let before_room = room(out);` are [ENT-3.S6] equalities
over the live terms generalized to the four measures [BLK-0], and at that point each
live term equals its **entry datum** [MSR-3], so the `requires` transports into the
loop's base: `spare_lo` at `at = 0` is `room(out) >= before_room`, which is the
equality. `seq_place`'s `room > 0` discharges from `spare_lo`, `before_room >= count`
and `at < count` by [MSR-4] step 5; probes `k21` and `k21b` are that arithmetic at
v0.41 scale, accepted and then rejected when the invariant is deleted. Each of the five
invariants is preserved by exactly one published relation of `seq_place`. At the exit
`at = count` and the four two-sided invariants give the two exact `ensures`; **`cap` is
not an invariant and needs none**, because [MSR-2]'s identity gives
`cap(out) = len(out) + room(out) = (before + count) + (before_room - count) =
before + before_room`, which is `cap` at entry. `written == len(source)` reads off
`count`'s own equality.

**The cost is the two-sided pairs**, and it is [CALL-7]'s honest price: an exact
measure relation a helper publishes costs **two** header invariants, because [INV-1]
3105 admits the four ordered relations and not `==`. Five invariants where the sixth
draft wrote two is what makes `collect` callable twice; the sixth draft published no
`room` at all, so a second `collect` into the same run was undischargeable and
`bs_append_slice`'s loop stopped after one iteration. 5.1's Q14 records the one change
that would halve the count.

#### 3.L.4 The pool and the two `try` forms, written out

§4 calls these, so this file declares them.

```wf-design
linear struct Lease['s] {
  run: Vector<'s, u8>;
}

struct BlockPool['s] {
  free: FixedVector<Vector<'s, u8>, 8>;
}

fn pool_new['s](arena: &uniq Arena<'s, 65536, 16>) -> made: own Option<BlockPool<'s>>
    reads(arena), writes(arena), allocates(arena) contract {
  ensures when Some(value: pool): len(pool.free) >= 8_u64;
  ensures when Some(value: pool): cap(pool.free) == 8_u64;
  ensures when Some(value: pool): head(pool.free) <= 0_u64;
} {
  doc "Carves eight 256-byte runs out of the arena and holds them as a free list.";
  let free = seq_fixed::<Vector<'s, u8>, 8>();
  for @carve (
    at in 0_u64..8_u64,
    invariant grown: len(free) >= at,
    invariant spare: room(free) + at >= 8_u64,
    invariant flat: head(free) <= 0_u64
  ) {
    let taken = seq_arena::<u8>(arena: &uniq deref(arena), count: 256_u64);
    match taken {
      None() => {
        return None<BlockPool<'s>>();
      }
      Some(value: run) => {
        set free = seq_place(vector: move free, value: move run);
      }
    }
  }
  let pool = BlockPool<'s>(free: move free);
  return Some<BlockPool<'s>>(value: move pool);
}

fn pool_take['s](pool: own BlockPool<'s>)
    -> (rest: own BlockPool<'s>, leased: own Option<Lease<'s>>)
    reads(pool.free), writes(pool.free) contract {
  ensures cap(rest.free) == cap(pool.free);
  ensures head(rest.free) == head(pool.free);
  ensures len(rest.free) <= len(pool.free);
  ensures room(rest.free) <= room(pool.free) + 1_u64;
  ensures when leased is Some(value: got): room(rest.free) >= 1_u64;
  ensures when leased is None(): len(rest.free) <= 0_u64;
} {
  doc "Leases one run, or reports that the free list is empty.";
  let spare = len(pool.free);
  let any = spare > 0_u64;
  if any {
    set (pool.free, one) = seq_take(vector: move pool.free);
    let ticket = Lease<'s>(run: move one);
    return move pool, Some<Lease<'s>>(value: move ticket);
  }
  return move pool, None<Lease<'s>>();
}

fn pool_release['s](pool: own BlockPool<'s>, lease: own Lease<'s>)
    -> rest: own BlockPool<'s>
    reads(pool.free), writes(pool.free) contract {
  requires room(pool.free) > 0_u64;
  ensures cap(rest.free) == cap(pool.free);
  ensures head(rest.free) == head(pool.free);
  ensures len(rest.free) == len(pool.free) + 1_u64;
  ensures room(rest.free) + 1_u64 == room(pool.free);
} {
  doc "Returns one lease to the free list; the caller has proved there is room.";
  let Lease(run: back) = move lease;
  set pool.free = seq_place(vector: move pool.free, value: move back);
  return move pool;
}
```

**`pool_release` is the *proved* spelling, and that is the round-6 repair.** The sixth
draft wrote the checked one — `-> (rest, unreturned: own Option<Lease<'s>>)` — and
round 6 showed what the refusal arm then costs: `Lease` is linear, so the arm is
mandatory, and the only thing a writer can do on it is `let Lease(run: orphan) = move
lost;`, which is a legal destructuring consume that throws the block away. That is
`linear` behaving correctly — it is a **must-consume** obligation and a destructuring
is a consume — and it is not the must-return property a pool needs. The proved
spelling gives the property by proof instead of by marking: `requires room(pool.free) >
0_u64` is discharged at the call site from `pool_take`'s own
`when leased is Some(value: got): room(rest.free) >= 1_u64`, one published premise, so
**there is no refusal arm and the lease has exactly one route on every path**. Q0b
records what changed; the general sentence is [RES-6]'s.

`pool_take` cannot state `room(got.run) >= 256_u64`, because a `Vector<'s, u8>` carries
its capacity as a measure and not in its type, so putting one into a `FixedVector`
element and taking it out loses the figure `pool_new` established. A caller that needs
room reads it and branches, once per lease. That is the honest price of the pool being
library data rather than a kernel store, and 4.1 pays it in the open.

```wf-design
fn try_place<T, const n: u64>(vector: own FixedVector<T, n>, value: own T)
    -> (rest: own FixedVector<T, n>, unplaced: own Option<T>)
    reads(vector), writes(vector) contract {
  ensures cap(rest) == cap(vector);
  ensures head(rest) == head(vector);
  ensures len(rest) <= len(vector) + 1_u64;
  ensures len(rest) >= len(vector);
  ensures room(rest) <= room(vector);
} {
  doc "Appends one value, handing it back when the run is full.";
  let spare = room(vector);
  let fits = spare > 0_u64;
  if fits {
    set vector = seq_place(vector: move vector, value: move value);
    return move vector, None<T>();
  }
  return move vector, Some<T>(value: move value);
}

fn try_take<T, const n: u64>(vector: own FixedVector<T, n>)
    -> (rest: own FixedVector<T, n>, taken: own Option<T>)
    reads(vector), writes(vector) contract {
  ensures cap(rest) == cap(vector);
  ensures head(rest) == head(vector);
  ensures len(rest) <= len(vector);
  ensures room(rest) >= room(vector);
} {
  doc "Removes one value from the end, or reports that the run is empty.";
  let held = len(vector);
  let any = held > 0_u64;
  if any {
    set (vector, one) = seq_take(vector: move vector);
    return move vector, Some<T>(value: move one);
  }
  return move vector, None<T>();
}
```

Both rest on [ENT-3.S6]'s generalization over the four measures [BLK-0], and both are
written per element class where the body moves a `T` (probes `x14`, `x15`). Their
`ensures` lists are two-sided rather than exact because a branch joins two arms, which
is [CALL-7]'s "a two-sided bound where the body establishes no exact value" doing
exactly what it is for.

#### 3.L.5 The growth policy, and what a hosted program pays

```wf-design
struct Bytes {
  v: Vector<u8>;
}

enum Grown {
  Grew(value: Bytes);
  Refused(value: Bytes);
}

fn bs_new(heap: &uniq Heap) -> made: own Option<Bytes>
    reads(heap), writes(heap), allocates(heap) contract {
  ensures when Some(value: fresh): len(fresh.v) <= 0_u64;
  ensures when Some(value: fresh): cap(fresh.v) <= 0_u64;
  ensures when Some(value: fresh): head(fresh.v) <= 0_u64;
} {
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

fn bs_reserve(s: own Bytes, heap: &uniq Heap, total: own u64) -> grown: own Grown
    reads(s.v, heap), writes(s.v, heap), allocates(heap) contract {
  requires total >= len(s.v);
  ensures when Grew(value: ready): cap(ready.v) == total;
  ensures when Grew(value: ready): len(ready.v) == len(s.v);
  ensures when Grew(value: ready): room(ready.v) + len(ready.v) == total;
  ensures when Grew(value: ready): head(ready.v) <= 0_u64;
  ensures when Refused(value: back): len(back.v) == len(s.v);
} {
  doc "Grows the backing run to total slots, preserving element order, or reports that the store refused.";
  let count = len(s.v);
  let taken = seq_heap::<u8>(heap: &uniq deref(heap), count: total);
  match taken {
    None() => {
      return Refused(value: move s);
    }
    Some(value: fresh) => {
      let built = move fresh;
      for @move (
        at in 0_u64..count,
        invariant left: len(s.v) + at >= count,
        invariant gone: len(s.v) + at <= count,
        invariant made_lo: len(built) >= at,
        invariant made_hi: len(built) <= at,
        invariant spare_lo: room(built) + at >= total,
        invariant spare_hi: room(built) + at <= total,
        invariant flat: head(built) <= 0_u64
      ) {
        set (s.v, byte) = seq_take_front(vector: move s.v);
        set built = seq_place(vector: move built, value: byte);
      }
      let Bytes(v: old) = move s;
      dispose old;
      let ready = Bytes(v: move built);
      return Grew(value: move ready);
    }
  }
}
```

**Proof route, and the three round-6 repairs it carries.** `seq_heap` publishes all
four measures of `built` on its `Some` arm [BLK-0], and they reach `built` through
[CALL-6]'s S13 at the arm binder [CALL-4]. `left` and `gone` bound the source, `made_*`
and `spare_*` the destination, `flat` the head; each is preserved by exactly one
published relation of `seq_take_front` or `seq_place`. At the exit `at = count`, so
`len(built) = count = len(s.v)` at entry, `room(built) = total - count`, and `cap` falls
out of [MSR-2]'s identity at `total`.

1. **The tail constructs rather than replaces.** The sixth draft wrote `let old =
   replace s.v = move built;` and then claimed its `ensures` from "[MSR-2]'s identity at
   the constructed field" — but [SET-2] 528 says a `replace` commit establishes no fact
   and `built`'s own measures die at the `move`, so the whole contract had no premise
   left. Destructuring `s`, disposing the old run and **constructing** `Bytes(v: move
   built)` routes `built`'s measures into the result through [MSR-3]'s construct
   placement, which is one of 3.L.6's eight and which the sixth draft added for exactly
   this and did not use here.
2. **The contract publishes the result's own measures.** The sixth draft published
   `spare + len(ready.v) >= total` over a separate `u64` payload field, and nothing
   related `spare` to `room(ready.v)`, so 4.2's next statement was undischarged. `Grown`
   loses its `room` field; a caller that wants the number reads `room(kept.v)`.
3. **`bs_new` publishes.** The sixth draft's `bs_new` declared no contract at all, so
   `bs_reserve`'s `requires total >= len(s.v)` was undischarged at its only call site.

`old` is linear, so [LIV-1] would refuse the return edge without the `dispose`, and
**`dispose old;` names no capability**: `old`'s brand is the entry heap's region, that
store's provider is `Heap`, and the innermost live binding of that type is the `heap`
parameter, reached through its borrow [PROV-6]. The statement writes it, which is why
`bs_reserve`'s row carries `writes(heap)`. `Grown` has a linear field, so it is linear
by ownership, so neither arm can be dropped by the caller — which is what makes 4.2's
two `dispose` statements mandatory rather than conscientious.

`bs_shrink` is the same function with `total < count` and `requires total <= len(s.v)`,
with the drain bounded by `total`. **Its `dispose old;` then releases a run still
holding `count - total` elements, and that is correct**: [PROV-6]'s walk visits a
container's elements before its backing, so the statement needs no emptiness premise —
worth one sentence because a writer reading "drain then dispose" assumes otherwise.

**The store region is elided, and seven disposals arrive.** `byte_string.wf` has
exactly one store, so under [PROV-1] nothing in it names a region: the whole region
parameter list leaves every struct and signature, fifteen brand occurrences leave the
written types, and twelve call-site brand arguments go with them. And `Bytes` is linear
because `Vector<u8>`'s release needs the `Heap`, so every one of the program's seven
points at which a `Bytes` value stops being used is a `dispose s;` — five in `main` and
two inside `bs_reserve`. None of them existed before, because today the compiler frees
the `buffer<u8>` at a scope exit under no effect row at all (probe `r2_5`). Of the
roughly twenty-nine writer-visible items the program then carries, twenty-two buy
something a systems programmer wants: five provider parameters, seven disposals, five
`match`es on a typed refusal, five hand-back result binders. The way to carry fewer is
an arena, whose values are affine.

#### 3.L.6 What the partition test found the kernel lacked

Nine, each named with the library function that demanded it and the probe that shows
it is new capability rather than a compiler defect. Round 6 added the ninth and
sharpened the sixth.

```text
| # | kernel addition                      | demanded by                       | today                 |
|---|--------------------------------------|-----------------------------------|-----------------------|
| 1 | the one `set` commit rule over a      | collect, bs_reserve, pool_take,   | x5, t8, x2, x3        |
|   | place that is not a bare binding      | vacant, filled, clear, try_place  | REJECTED [STOR-1]     |
|   | [LIV-2]                              | — every library function that     | AffineSetTarget       |
|   |                                      | transforms a place it does not    |                       |
|   |                                      | own outright                      |                       |
| 2 | its n-ary form and the ordered        | pool_take, bs_reserve's drain,    | new grammar; x3       |
|   | result list [S16]                    | clear, collect's caller           | REJECTED [GRAM-2]     |
| 3 | [ENT-3.S6] over the four measures    | every try_ form, pool_take,       | S6 2785 covers len    |
|   | [BLK-0]                              | pool_release — every branch on a  | alone                 |
|   |                                      | capacity                          |                       |
| 4 | the construct placement of the       | Bytes, BlockPool, bs_reserve's    | construct kills the   |
|   | measure datum [MSR-3]                | tail — every library nominal      | operand's measures    |
|   |                                      | wrapping a run                    |                       |
| 5 | a const generic as a value, an       | vacant, filled, try_place, and    | t1, t2, t3 REJECTED   |
|   | endpoint and a clause operand        | every capacity-parametric         | [TYPE-5]; t4 ACCEPTED |
|   | [MSR-6, S21]                         | function; ~43 bodies for 14       |                       |
|   |                                      | algorithms without it             |                       |
| 6 | a relation published per enum        | pool_take, pool_new, try_place,   | x1 [FN-9] Invalid-    |
|   | variant and per result ordinal, with | bs_reserve, bs_new — every        | PostconditionSelector;|
|   | field projection on a result datum   | library constructor               | x2 [TYPE-5] on        |
|   | [CALL-4, S24]                        |                                   | len(result)           |
| 7 | the window's front operations and    | every queue, ring, deque and FIFO | no analogue; a        |
|   | `seq_rebase` [BLK-1, BLK-3, S8, S29] | — and the growth policy, whose    | shifting take_front   |
|   |                                      | order preservation is free under  | IS writable, so only  |
|   |                                      | a window                          | the head-carrying     |
|   |                                      |                                   | forms enter           |
| 8 | linearity by declaration             | the pool's Lease, and every       | a dropped lease is    |
|   | [PROV-6, S18]                        | library that recycles values      | silent today          |
| 9 | the publication of a declared        | EVERY function in this section    | [ENT-3] has no source |
|   | relation [CALL-6]                    | and both worked programs, at      | for a declaration-    |
|   |                                      | their first statement             | domain relation       |
```

**Item 9 is round 6's and it is the one with no partial workaround.** Every proof route
above begins with "the row publishes"; the sixth draft named an [ENT-3] source for that
and stated it nowhere, and a provider's post-state relation was not an admissible
clause and had no destination at all. Nothing in 3.L compiles without it.

**What left the list, and why.** The fifth draft's exit datum went with R1, its
`seq_exchange` is 3.L.2's three statements, and its `&uniq` run parameter is [BLK-4]'s
refusal. And the list that matters as much: **what the partition did *not* need.** A
queue needed no kernel ring, a pool needed no kernel store, a keyed table needed no
kernel occupancy, a growth policy needed no kernel growth row, middle removal needed no
kernel row, filled and vacant construction needed no kernel row, and the `try` family
needed nothing at all. Five owner types became two, thirty-odd operations became
thirteen, three views became two, sixteen added nominals became five, and three writing
statements became one.

One item was **not** resolved by writing it, and it is the honest residue: a writer's
generic cannot serve a copy and an affine element type from one body, so `filled` and
both `try` forms are written per element class (3.L.0). That is Q8 and not a missing
primitive.

#### 3.L.7 When to write `linear`, and what it buys

The storage obligation is derived and a writer never marks it: a heap-backed run is
linear because its release needs the `Heap`, an arena-backed run and a frame-resident
run are affine because their reclamation needs nothing, and any type that **owns** a
linear value is linear by ownership [PROV-6]. A **view** owns nothing, so a view of
runs is not linear. **Marking a store-derived type is always redundant and is a sign
the writer has misread the criterion.**

The modifier is for a **logical** obligation, and the whole test is one question:

> **Would silently dropping this value be a bug?**

If the answer is yes for a reason that is not about storage, the type is `linear`. The
shapes that pass are recognizable:

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

**And what the modifier buys is stated, because round 6 showed the sixth draft implying
more.** `linear` makes a discard **visible and deliberate**: the value must be moved,
destructured or disposed, and a destructuring is a legal consume that can throw the
contents away. For a transaction, a request and a builder that is exactly right — the
honour path *is* a destructuring inside the code that owns the type. For a **lease** it
is not enough by itself, because the obligation is directional: it must go back to a
specific holder. **A directional obligation is bought by proving the return.** Write
the library's release as the **proved** spelling — total, under a `requires` the caller
discharges from the take's own published relation — and the value has exactly one route
on every path; write it as the checked spelling and the refusal arm is a legal place to
destructure and discard. 3.L.4's `pool_release` is the first form and the sixth draft's
was the second.

And the shapes that do **not** take the modifier: a value whose only cost of being
dropped is memory the language already reclaims; a value the writer merely wants to
remember to use, for which the modifier is a type-level answer to a review question;
and a value whose obligation is conditional, since the modifier is unconditional and a
writer who marks one will meet [LIV-1] on the arm where the obligation does not apply.

The cost of a wrong `linear` is paid at every scope exit of every value of that type,
including in code the writer does not own, and the diagnostic names a binding rather
than the obligation the marker meant. When in doubt, the shapes above are the guide and
the question is the test.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. The
standard they are held to is that every statement is accepted by a compiler
implementing 3.K's rules, **the library functions of 3.L**, and the unchanged v0.41
rules — **and every function either program calls is declared in 3.L**, which is round
6's requirement and which the sixth draft failed for six of them. Both were walked
statement by statement against all three, and this time the walk was held to two
standards: *for every loop, the facts live at its head and the rule that keeps them
there*, and *for every obligation, the published relation that discharges it and the
function that published it*.

Round 6 found the sixth draft's pair failing at seven obligations between them, and
all seven were of one kind: **a fact true where it is established with no channel to
where it is needed.** 4.1's queue invariant was false and unprovable, 4.2's central
`set` redeclared a live binding, `bs_reserve`'s whole contract rested on a `replace`
that establishes nothing, and every view formation in both programs was undischarged
because no construction loop carried its `head`. Each is repaired at the rule, not at
the program: [CALL-6] gives a relation a source and a destination, [CALL-7] makes a
hand-back contract complete, [LIV-2] makes a `set` target that names a binding in
scope a commit rather than a redeclaration, and 3.L's `flat` invariant carries the
head.

Byte figures are symbolic. No implementation computed any of them, and where a
figure depends on code generation the table says so instead of inventing a number.

### 4.1 A cooperative run queue with the heap absent

A fixed run queue of tasks, a 256-byte transmit ring, and an eight-block pool with a
**linear lease and a proved release**. Each task is a state machine that advances one
step per turn and re-queues itself while it wants another. No heap, no recursion, an
acyclic call graph, and a queue loop whose resource state is restored on every
backedge. It is **not** a context-switching scheduler, and 1.5 says why. It uses
`try_place`, `try_take`, `pool_new`, `pool_take` and `pool_release` from 3.L.4, and
nothing else the kernel does not declare.

```wf-design
struct Task {
  state: u32;
  arg: u64;
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
  ensures written == 8_u64;
  ensures len(rest.run) == len(block.run) + 8_u64;
  ensures room(rest.run) + 8_u64 == room(block.run);
  ensures cap(rest.run) == cap(block.run);
  ensures head(rest.run) <= 0_u64;
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
  let before = len(block.run);
  let before_room = room(block.run);
  for @fill (
    at in 0_u64..8_u64,
    invariant grown_lo: len(block.run) >= before + at,
    invariant grown_hi: len(block.run) <= before + at,
    invariant spare_lo: room(block.run) + at >= before_room,
    invariant spare_hi: room(block.run) + at <= before_room,
    invariant flat: head(block.run) <= 0_u64
  ) {
    set block.run = seq_place(vector: move block.run, value: mark);
  }
  return move block, 8_u64;
}

fn drain['s](ring: own FixedVector<u8, 256>, block: &Lease<'s>, count: own u64)
    -> (rest: own FixedVector<u8, 256>, sent: own u64)
    reads(ring, block.run), writes(ring) contract {
  requires count <= len(deref(block).run);
  ensures sent <= count;
  ensures len(rest) == len(ring) + sent;
  ensures room(rest) + sent == room(ring);
  ensures cap(rest) == cap(ring);
  ensures head(rest) <= 0_u64;
} {
  doc "Copies one prefix of the leased block into the ring when the ring has room, and reports what it sent.";
  let before = len(ring);
  let before_room = room(ring);
  let fits = count <= before_room;
  if fits {
    for @copy (
      at in 0_u64..count,
      invariant grown_lo: len(ring) >= before + at,
      invariant grown_hi: len(ring) <= before + at,
      invariant spare_lo: room(ring) + at >= before_room,
      invariant spare_hi: room(ring) + at <= before_room,
      invariant flat: head(ring) <= 0_u64
    ) {
      let byte = deref(block).run[at];
      set ring = seq_place(vector: move ring, value: byte);
    }
    return move ring, count;
  }
  return move ring, 0_u64;
}

resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Runs a cooperative queue of state machines over a pooled block store and a transmit ring.";
  let ring = seq_fixed::<u8, 256>();
  let pending = seq_fixed::<Task, 32>();
  let first = Task(state: 0_u32, arg: 65_u64);
  set (pending, unplaced) = try_place(vector: move pending, value: move first);
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
    let made = pool_new(arena: &uniq scratch);
    match made {
      None() => {
        set code = 1_u8;
      }
      Some(value: pool) => {
        loop @queue {
          set (pending, next) = try_take(vector: move pending);
          match next {
            None() => {
              break @queue;
            }
            Some(value: task) => {
              set (pool, leased) = pool_take(pool: move pool);
              match leased {
                None() => {
                }
                Some(value: held) => {
                  let spare = room(held.run);
                  let big = spare >= 8_u64;
                  if big {
                    set (held, written) = render(block: move held, task: &task);
                    set (ring, sent) = drain(ring: move ring, block: &held, count: written);
                  }
                  set pool = pool_release(pool: move pool, lease: move held);
                }
              }
              let stepped = advance(task: move task);
              match stepped {
                None() => {
                }
                Some(value: again) => {
                  set (pending, refused) = try_place(vector: move pending, value: move again);
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
  region  floor.altstack      bytes  <runtime>       align  <target>  contiguous
  stack   entry               bytes  <post-codegen>  align  <ABI>
  lanes                       count  1
  slots   task.records        count  0
  slots   completion.records  count  0
  slots   handle.table        count  0
```

`static.image` is the const items and the static parts of the emitted module
[STOR-6]. `floor.altstack` is the guard-page floor's per-thread alternate stack, which
[RES-4] makes an item of a marked build and which is an **extent**, so it is a `region`
item and not a `handle` one [RES-1]; the sixth draft's envelope omitted it entirely.
`stack.entry` is `main`'s frame — the `FixedVector<u8, 256>` ring, the
`FixedVector<Task, 32>`, the `BlockPool`'s `FixedVector<Vector<'a, u8>, 8>` and the one
`arena_frame` occurrence's 65536-byte extent, whose written alignment of 16 the item now
carries [RES-2] — plus `render`, `drain`, `advance` and the library, plus the runtime
frames beneath `main`, its bounded teardown and the release walk's straight-line frame
cost; measured post-codegen over the whole chain [STK-3]. `lanes` is 1 because no `par`
construct is emitted [RUN-1]. Every `slots` row is zero because there is no `par`
permission, no may-suspend action and no system handle — **and because the program
declares no demand on any runtime store**, which is what [RES-7] now publishes at
source stage instead of comparing a count against a row the runtime writes.

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
| arena demand bounded  | the eight 256-byte takes are inside pool_new, called ONCE before the queue   |
|                       | loop, so the bump domain's backedge delta on @queue is 0 and [RES-10]'s      |
|                       | loop rule needs no route at all. Had a take been inside the loop with no     |
|                       | region block around it, the delta would be +256 per trip, route (i) has no   |
|                       | trip count, route (ii) has no invariant and route (iii) does not apply to a  |
|                       | consumable budget, and the program would be refused at that loop [RES-10]    |
| the free list         | a FixedVector in a frame; frame placement's [RES-5] algebra has no acquire   |
|                       | and no release, so it is not a domain and premise 3 says nothing about it.   |
|                       | What keeps it full is the PROVED pool_release, whose requires the caller     |
|                       | discharges from pool_take's own published room — not the envelope, and not   |
|                       | the modifier by itself                                                       |
| queue and ring        | FixedVector<Task, 32> and FixedVector<u8, 256> are frame placement           |
| release walk          | every type reachable from main has an acyclic capability-released-leaf graph |
|                       | — in fact an empty one, since nothing here is heap-backed — so [PROV-6]'s    |
|                       | walk is straight-line and its frame cost is ordinary frame cost [STK-3]      |
| L9's displacement     | try_place hands its value back and the refusal is matched; pool_release      |
|                       | cannot refuse; try_take's None is the loop's exit. Nothing is displaced      |
|                       | silently                                                                     |
| stack bounded         | one context, one chain, measured after code generation [STK-3]              |
| runtime closed        | W = 1, no task or completion records; the program declares no runtime-store  |
|                       | demand, so [RES-7] excludes nothing and [QUAL-2] has nothing to match        |
| return and retained   | the queue loop has a break, so it has a fallthrough entry and both its       |
|                       | `return` and `retained` entries are empty; a variant with no break would     |
|                       | publish its steady state in `retained`, composed through the sequence by the |
|                       | same formula every other label uses [RES-10], [STK-4]                       |
```

#### The writer's-eye walkthrough

**`set (held, written) = render(block: move held, task: &task);`** is the statement
three drafts could not write. Under the fourth draft `render` took a `&uniq` container
and [CNT-7] refused it. Under the fifth it took one and published its post-state
through an exit datum, which round 5 turned back into D1. Under the sixth it took the
lease by value — correctly — but published only `written <= len(rest.run)`, an upper
bound on the wrong side, so its caller learned nothing it could use. Here [CALL-7]
requires the contract to be complete over every measure of what it hands back, so the
caller receives `written == 8_u64`, `len(rest.run) == len(block.run) + 8_u64` and the
other three, and every later obligation reads one of them. The `set` is [LIV-2] at an
arm binder (probe `w8` accepts that shape today); both targets name bindings in scope,
so both are commits and neither redeclares anything, and `held` stays live through the
commit so [LIV-1]'s join agreement is met on both arms of the enclosing `match`.

**`requires room(block.run) >= 8_u64;` is discharged by a dominating branch**, and that
branch is the honest price of the pool being library data rather than a kernel store. A
`Vector<'s, u8>` carries its capacity as a *measure* and not in its type [BLK-1], so
putting one into a `FixedVector` element and taking it out again loses the figure
`pool_new` established, and no clause `pool_take` could write would recover it.
`let spare = room(held.run); let big = spare >= 8_u64; if big { ... }` is one runtime
branch per lease, its first statement is a fact only because [ENT-3.S6] generalizes over
the four measures [BLK-0], and the branch is [BLK-0]'s own second mechanical fix. 5.1's
Q6 records that a container whose element capacity is in its type is the next candidate
and has to justify itself against this branch.

**There is no header invariant on the queue loop, and that is round 6's repair.** The
sixth draft carried `invariant slots: len(ring) + 8_u64 <= 256_u64` and claimed it
preserved "from `drain`'s `ensures sent <= len(rest)` together with the standing
`len(ring) <= cap(ring)`". Three lenses found the same two things: the derivation does
not exist — `sent <= len(rest)` bounds `sent`, not `len(rest)`, and `len <= cap = 256`
gives `len + 8 <= 264` where the goal is `<= 256` — and **the invariant is false of the
program**, because nothing ever removes a byte from `ring` and `advance` re-queues each
task twice, so `len(ring)` grows monotonically past 256. A header invariant over a
quantity a loop only increases cannot be made true by any wording. The repair is to make
the program correct rather than merely provable: **`drain` is the checked spelling**. It
takes no `room` requirement, branches on the ring's own room, copies when it fits, and
reports `sent`. A full ring then stops being written instead of being asserted not to
fill, which is L3's and L9's discipline and is the shape [RES-10]'s routes are written
for. `drain`'s one remaining requirement, `count <= len(deref(block).run)`, discharges
from `render`'s `len(rest.run) == len(block.run) + 8_u64` with `written == 8_u64` and the
standing `Z <= len` — one published premise and one standing fact.

**Inside `render`**, whose two borrows and one brand are the only regions it names:

```wf-design
  for @fill (
    at in 0_u64..8_u64,
    invariant grown_lo: len(block.run) >= before + at,
    invariant grown_hi: len(block.run) <= before + at,
    invariant spare_lo: room(block.run) + at >= before_room,
    invariant spare_hi: room(block.run) + at <= before_room,
    invariant flat: head(block.run) <= 0_u64
  ) {
    set block.run = seq_place(vector: move block.run, value: mark);
  }
```

The **backedge** is the derivation the whole container surface rests on. The `set` is
[LIV-2] at a **field of a linear value**, and its target names a binding in scope, so
the root's [ENT-2] term survives [MSR-3]; the facts over `len`, `room` and `head` of
`block.run` die by [MSR-2] because the commit writes that descriptor storage; and
`seq_place`'s declared `len(result) = len(vector) + 1`, `room(result) = room(vector) -
1`, `cap(result) = cap(vector)` and `head(result) = head(vector)` re-establish them on
the same term, through [CALL-6]'s S13 and [CALL-4]'s `set`-target destination. **Each
invariant is preserved by exactly one published premise**, which is what puts the
derivation inside [ENT-6] 3015's two-premise budget; under the fifth draft, which
published two of three measures and left `room` to the standing identity, it needed
three and the loop was refused. The five invariants are two more than the sixth draft
wrote, and the two extra buy the *exact* relations [CALL-7] requires — which is exactly
the cost 5.1's Q14 would remove.

**`set pool = pool_release(pool: move pool, lease: move held);`** is where round 6's
second resource finding lands, and the repair is in the library rather than in the
modifier. The sixth draft called a **checked** `pool_release` returning
`unreturned: own Option<Lease<'a>>`; `Lease` is linear, so `Option<Lease>` is linear by
ownership and the arm was mandatory — but the only thing the arm could do was
`let Lease(run: orphan) = move lost;`, a legal destructuring consume that throws the
block away. Eight of those and the pool is empty for the life of the program, with the
same observable the fifth draft's silent leak had. `linear` had made the discard
**visible**, which is what it buys, and not **impossible**, which Q0b claimed. 3.L.4's
`pool_release` is the **proved** spelling: `requires room(pool.free) > 0_u64`,
discharged here from `pool_take`'s `when leased is Some(value: got): room(rest.free) >=
1_u64` — one published premise, surviving the intervening `render` and `drain` because
neither writes `pool`'s descriptor storage [MSR-2]. There is no refusal arm, so on every
path the lease goes back, and the pool's fullness is a proof rather than a hope.

**`&task` and `&held` are loop-body borrows and need no inner block.** [OWN-11] 647
restricts a `borrow_expr` in a loop body to regions introduced inside that body; under
the region-spelling amendment a borrow expression names no region at all, so the
restriction is vacuous and the fifth draft's inner `region { }` wrappers are gone. That
dependency is recorded on [OWN-11]'s register row rather than assumed.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one library function with a typed
failure, disposal is one statement that names no capability, the append helper takes
the destination by value and hands it back, and **not one region names the store**,
because the program has exactly one and `main` declares no region parameter.

```wf-design
const ceiling: u64 = 4096_u64;

command fn main(command.stdout as sink: own Output, command.heap as heap: own Heap)
    -> status: own ExitStatus
    reads(sink, heap), writes(sink, heap), allocates(heap) {
  doc "Collects one fixed input run into a heap-backed run and writes it out, reporting a refusal instead of dying.";
  let input = filled::<u8, 4096>(value: 65_u8);
  let code = 0_u8;
  let total = 0_u64;
  let made = bs_new(heap: &uniq heap);
  match made {
    None() => {
      set code = 70_u8;
    }
    Some(value: holder) => {
      let grown = bs_reserve(s: move holder, heap: &uniq heap, total: ceiling);
      match grown {
        Grew(value: ready) => {
          let kept = move ready;
          region {
            let line = seq_slice(vector: &input);
            set (kept.v, total) = collect(out: move kept.v, source: line);
          }
          region {
            let body = seq_slice(vector: &kept.v);
            let outcome = write_once(output: &uniq sink, source: &body, start: 0_u64, end: total);
            match outcome {
              Ok(value: next) => {
              }
              Err(error: problem) => {
                set code = 74_u8;
              }
            }
          }
          dispose kept;
        }
        Refused(value: back) => {
          set code = 70_u8;
          dispose back;
        }
      }
    }
  }
  return exit_status(code: code);
}
```

#### The writer's-eye walkthrough

**`set (kept.v, total) = collect(out: move kept.v, source: line);`** is R1's central
statement, at a **field** place, and it is where round 6's consistency lens found the
sixth draft breaking its own repair. Under [LIV-3] every later target of a multi-target
`set` was "an ordinary `let` binding introduced at the statement", so this line
*declared* `total` inside the `region` block: a [TYPE-6] `DeclarationCollision` with the
live outer `total` — verified against the gate — and, had it resolved, a binding out of
scope at the `write_once` two statements later, which would have written zero bytes.
D2's one commit rule decides it by name resolution alone: **a target that names a
binding in scope is a place and is committed to**, so `total` is the outer `u64` and
`kept.v` is the field, both non-overlapping, both dead at the commit (`kept.v` because
the right-hand side consumed it, `total` because its type is copy). The relations reach
both targets through [CALL-6]'s S13 and [CALL-4]'s `set`-target destination; under the
sixth draft a plain `set` receiver was not an [ENT-3.S12] destination at all.

`collect`'s `requires len(source) <= room(out)` discharges from `bs_reserve`'s
`room(ready.v) + len(ready.v) == 4096_u64` and `len(ready.v) == len(holder.v)` with
`bs_new`'s `len(fresh.v) <= 0_u64`, giving `room(kept.v) >= 4096`, against
`seq_slice`'s published `len(result) = <datum of len(input)>` and `filled`'s
`len(result) >= 4096` with `cap == 4096`. **The sixth draft's chain was broken at all
three of those links**: `bs_new` declared no contract at all, `bs_reserve` published a
bound on a separate `u64` payload field that nothing related to `room(ready.v)`, and
`bs_reserve`'s own clause rested on a plain `replace` that [SET-2] 528 says establishes
nothing. 3.L.5 repairs all three, the third by **constructing** the result rather than
replacing into it.

**`let line = seq_slice(vector: &input);`** discharges [VIEW-2]'s
`head(input) + len(input) <= cap(input)` from `filled`'s `head(result) <= 0_u64` and the
standing `len <= cap` — one clause and one standing fact, in the unordered-pair family.
Under the sixth draft `filled` published no `head` and the premise was
`head(input) <= 0_u64` with nothing to prove it from, so this statement and its sibling
were both refused; [CALL-7] and 3.L.3's `flat` invariant are the two halves of the
repair, and the weakened non-wrap premise is what makes a drained run viewable at all.
**`line` is a `slice` and is therefore `copy` [S27]**, so it is passed without `move`
and could be formed once and used twice; that is the whole practical difference S27
buys.

**`write_once(output: &uniq sink, source: &body, start: 0_u64, end: total)`** is
[VIEW-7] over a view. Its obligations are `0_u64 <= total`, implicit, and
`total <= len(deref(body))`, which discharges from [VIEW-2]'s
`len(body) = <datum of len(kept.v)>` and `collect`'s
`len(rest) == len(out) + written` with `written == len(source)`. This is the statement
that makes goal A's container half real. Its three regions all relate nothing, so all
three are elided (3.K.0); the two inner blocks still exist because [OWN-10] 641 requires
the borrow's region to be introduced within the borrowed binding's scope, so `kept.v`
must be bound before the block opens, and under 3.K.0 the blocks carry no name because
nothing inside them names one.

**`dispose kept;` and `dispose back;`** are [PROV-6] [S12], once per arm that holds a
value, **and neither names a capability**. `Bytes` is a nominal that **owns** a field
whose release needs the `Heap`, so it is linear, so the `match` cannot be left with one
alive on either arm. The walk drops each `u8` element, which derives nothing, then
releases the backing to the store `Vector<u8>`'s type names; that store's provider type
is `Heap`, the innermost live binding of that type is the entry's own `own Heap`, and
the statement writes it — which is why `main`'s row carries `writes(heap)` and why two
overlapped disposals from one store would conflict under [RUN-3]. Identity comes from
the brand and permission from the store, so there is nothing left for the writer to
choose and nothing to write; had no `Heap` binding been in scope the statement would be
a hard error naming the parameter it needs. The walk's depth is `Bytes`'s ownership
height, a constant, so no `wf_resource_abort` is reachable from it. **There is no path
on which the process disappears**, which is the whole of goal B — and R2's cost is the
two statements themselves.

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
    every release of heap-owned storage is a statement whose effect row names the heap
      [PROV-6]
    every release walk's depth is a compile-time constant [PROV-6]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

Six of the diagnostics the design owes a writer, each citing a rule that exists in
3.K. `SecondStoreInOneRegion` is stated inside [PROV-1], `ConfinedFieldWithoutRegion`
inside [BLK-4], and `IncompleteHandBackContract` inside [CALL-7].

```text
Semantics/Source [BLK-0]: UndischargedOperationDomain
  operation: seq_place
  residual:  "0_u64 < room(block.run)"
  mechanical_fix: state a header invariant over room(block.run) [INV-1, MSR-5],
    dominate the place with a branch on room(block.run), take a larger run
    before the loop, or use the library's try_place

Semantics/Source [BLK-4]: UniqueParameterReachesContainer
  parameter "handle" is &uniq Vector<u8>, whose referent is a container nominal
  a callee that replaces a caller's run leaves the caller a measure of a value the
    callee destroyed, at a point the callee cannot name [L11]
  mechanical_fix: take the run by value and return it, whose length the caller then
    reads from your own contract [CALL-2, CALL-7], or take a length-fixed view of it

Semantics/Source [PROV-6]: LinearValueNotDisposed
  binding "kept" of type Bytes is live on the edge leaving this match arm
  its capability-released leaf Bytes.v of type Vector<u8> is backed by the store the
    entry heap names
  mechanical_fix: move the value out of this scope, destructure it, or write
    dispose kept; a store-backed run has no compiler-derived release, so nothing
    else can free it

Semantics/Source [PROV-6]: DisposeHasNoProvider
  "dispose old;" needs the provider of the store its leaf Vector<u8> names
  no binding of type Heap is live here, and the capability is determined by the
    brand rather than written
  mechanical_fix: take heap: &uniq Heap as a parameter of this function

Semantics/Source [PROV-6]: LinearValuePartiallyConsumed
  "move chunk.page" takes one leaf out of "chunk" of type Chunk, which is linear
  the residual leaf Chunk.spare of type Vector<u8> would then leave this scope by
    neither a move, a destructuring consume, nor a dispose
  mechanical_fix: write let Chunk(page: page, spare: spare) = move chunk; and handle
    both leaves, or dispose the whole value

Semantics/Source [CALL-7]: IncompleteHandBackContract
  "filled" returns result: own FixedVector<T, n>, which it constructed, and its
    contract states no relation for head(result)
  a caller that forms a view of this run needs it [VIEW-2]
  mechanical_fix: carry invariant flat: head(built) <= 0_u64 on the construction
    loop and publish ensures head(result) <= 0_u64;
```

The last two are new in this draft; probe `x4` is the program that gets the fifth and
the sixth is the one round 6 called the most expensive diagnostic hole in the file.

---

## 5. Open questions

Everything the owner's rulings and decisions settle is dropped and not restated. So is
everything earlier drafts asked and this one answers: the length-class terms and the
goal disposition are [MSR-1] and [MSR-4]; the arithmetic residual is [MSR-3]'s datums
and images; the arena's reclamation is [RES-5]'s cursor domain; the profile table is
[RES-2]; **how a declared relation becomes a fact is [CALL-6]**, which is round 6's
question and not an open one any more.

### 5.0 The decisions the rulings forced, and what each traded

These are the decisions the minimality ruling, R1, R2 and the owner's D1 and D2
forced. Every spelling any of them names is now decided (3.S).

**Q0a. `AppendView` and `absorb` are gone, and under R1 nothing is lost**
(footnote 3). The guarantee is an ordinary clause, `ensures len(rest) >= len(out)`,
relating a result to an input with no `old()`, no frame rule and no third view. *This
is no longer a trade.*

**Q0b. What `linear` buys is must-consume, visibly — and a pool's lease is
must-return by proof, not by the modifier.** The sixth draft wrote "a pool's lease is
`linear` by declaration, and 4.1's leak is a compile error", and **that sentence is
deleted**. Round 6 showed why: `linear`'s three routes include a destructuring consume,
which for a `Lease<'s>` whose field is an arena-backed run legally throws the block
away in one visible statement, so the checked `pool_release`'s refusal arm was a legal
place to discard and eight of them empty the pool with the same observable the fifth
draft's silent leak had. What the modifier buys is that the discard is **visible and
deliberate** — which is the right semantics for a transaction, a request and a builder,
whose honour path *is* a destructuring. A **directional** obligation is bought by
proving the return: 3.L.4's `pool_release` is total under `requires room(pool.free) >
0_u64`, the caller discharges it from `pool_take`'s own published relation, there is no
refusal arm, and the lease has exactly one route on every path. 4.1 is rewritten on it.
*Recommend the modifier, and recommend that the doctrine say plainly what it buys;
3.L.7 does.*

**Q0c. Every heap-derived value in a hosted program is disposed explicitly, and the
statement names no capability.** `dispose p;` is the whole spelling: identity comes
from the brand and permission from the store, so there is nothing to choose and nothing
to write (3.K.0, [PROV-6]). 3.L.5 counts it at seven statements for `byte_string.wf`.
The alternative is an implicit scope-exit free, which would have to reach a `Heap` the
scope may not hold, so it is not available while L2 stands. *Recommend it, and recommend
that the doctrine say plainly that a region block or an arena is how a writer writes
fewer.*

**Q0d and Q0e together, and *recommend both*.** Five owners became two and `FixedRing`
became nothing at all (footnotes 1, 2). `update`, `swap` and `seq_exchange` all became
one commit rule (footnote 4), which is three grammar forms fewer than the sixth draft
and two fewer rules. And the kernel declares no failure nominal ([BLK-2], [RES-6]).

**Q0f, the window's own trade, and its fifth cost.** [BLK-1] states five costs where
the sixth draft stated four and called them the whole bill. The fifth is `seq_rebase`
[S29]: without it `head` is an **absorbing state**, a ring that has wrapped can never be
viewed again, and every transmit path over a ring owes a permanent second run of full
capacity plus an O(n) copy per flush — which is a `region` item in `E` for the one
program shape the window was added for. A sixth cost is a runtime one and belongs in
the doctrine rather than the bill: in a ring `head` is genuinely nonzero, so no
optimizer removes the modulo, and a completion handler touching six fields of a
descriptor pays it six times unless it borrows the element once (probe `x10` shows the
compiler does not support that today). *Recommend the window, recommend `seq_rebase`
with it, and record that a writer who never removes from the front pays one descriptor
word and nothing else.*

**Q0g is decided and is recorded rather than asked — and it has landed.** The
region-spelling amendment is separate and mechanical (3.K.0). **It is no longer only
assumed**: a build in this session rejects a written region name at a position no other
position of the declaration names, citing `[FORM-8] RegionSpelling` with the mechanical
fix "drop the region name", and accepts the fully elided spelling of the same program
(6.1, probes `r1` and `r5`). What this design owes it is one property — the spelling is
decidable from the declaration text alone — and what it gets back is measured in 3.K.0.
**And the same probes show it changes nothing about D1**, which that build accepts in
its elided spelling: only [BLK-4]'s fourth clause refuses it.

**Q0h, the ruling's own question: should any of 3.L ship?** The owner leans toward no
standard library at all, and 3.L proves the partition whether or not a line of it is
committed. Five items are load-bearing for this design's evidence — `filled` for
[VIEW-7]'s addressable destinations, `collect` for the append story, `vacant` for the
keyed families, the pool for 4.1, and `bs_reserve` for 4.2. *Recommend: no `std`; those
five land as test programs under `tests/programs/`, where a rot check already reaches
them.*

### 5.1 The questions this design genuinely does not decide

**Q1. May a marked program handle a typed refusal, or must it prove every
acquisition?** **Permissive**: both spellings are admitted, since neither can ask for
more than `E`. But the two are not interchangeable and this draft says which is which:
a **checked** acquisition is what [RES-10]'s capacity route reads and is right for a
reusable-capacity store; a **proved** release is what makes a directional obligation
real (Q0b), because a checked one leaves a refusal a writer may legally discard.

**Q2. Where does a hosted marked program's large memory come from?** **Frame and
extent placement only**, as [PROV-5] and [BLK-2] provide; an entry row delivering a
committed region becomes right the day a program needs a store whose *size* is a
deployment decision rather than a source constant.

**Q3. Does the range relation need `seq_split_at`?** Not in this version. The
relation it needs already exists in [PROV-3]; what is missing is only the row.

**Q4. How does a marked program reach a device?** `main`'s effect row names only its
own labelled inputs and the `command` table is closed, so 4.1 has a transmit ring and
no way to flush it. **A second program kind** under [FN-7]'s existing closed-table
discipline, arriving with the execution-context design of 1.5. Until it exists, 4.1's
ring is written in the checked spelling so that a full ring stops being written rather
than being asserted not to fill.

**Q5. When does `par` become usable inside a marked program?** [RUN-1] forbids the
emitted module a `par` construct and [RUN-2] publishes `lanes(1)`, because the current
runtime's wait path runs a stolen task on the waiting lane's own stack. The answer is
the compiler-managed work-first continuation representation, then lifting the
prohibition. **[PROV-5]'s activation refusal is written for that day.**

**Q6. Does this version want a keyed or sparse container family?** Not yet.
Stable-identity storage is a vacant run plus element-position `replace`, which is
sound, L12-clean, and compiles in shape today (probe `x7`). A `FixedTable<T, n>` whose
typestate is an occupancy set is the next candidate, and under L18 it has to justify
itself against that — **and against 4.1's per-lease branch**, which is what a container
whose element capacity is in its type would remove.

**Q7. Should a system operation be able to append?** **Yes, in the batch that lands
[CALL-4]'s widened result vocabulary and [CALL-6]'s S13 in the [SYS-2] declaration
domain, and not here.** Then the bytes the host wrote become the run's own `len` and the
caller reads it from the operation's published relation, instead of [VIEW-7]'s
addressable destination and a `u64` beside the run.

**Q8. Is `copy` structural over aggregates, and does a generic body's `move` survive a
copy instantiation?** [OWN-1] 564 makes every owned composite affine regardless of its
field types, which is why `filled` and both `try` forms are written per element class
and why probes `x14` and `x15` disagree. **A `struct` or `enum` all of whose field
types are copy should be copy** — and the half that matters more here is the second: **a
generic body's `move` of a type parameter should be admitted at a copy instantiation,
where it is a no-op.** Without that half the first does not remove the wall, because the
*template* is checked as if `T` were affine, and the diagnostic fires inside a library
body with a mechanical fix that breaks the other instantiation. **A third axis is new
this round**: the same shape exists for affine-versus-linear, since a body that lets a
`T` reach a scope exit is accepted at one instantiation and refused at the other.
[PROV-6]'s declaration-site obligation over a region parameter closes the *region* axis;
the *type* axis would need a declared bound on a generic parameter, which is not this
design's and which the owner should weigh with Q8's other two halves.

**Q9. Is `E` part of program identity?** **An emitted machine-readable table beside
the object, carrying the module's content digest and explicitly not part of [PROG-2]
compilation-unit identity.** The digest is what makes the table a promise rather than a
document. One residue is recorded: the digest is of *the module*, while the figures
under it include [RUN-2]'s runtime-published rows and [STK-3]'s chain through the linked
runtime and adapter, so a swapped runtime satisfies the check while changing what `E`
describes. A second digest over the qualified runtime is the obvious repair and is not
proposed here.

**Q10. Should a `propagate` carry a disposition, and in what shape?** [PROV-6] refuses
a `propagate` while a linear binding is live, and D1 makes that refusal common. Round 6
measured the cost: eleven lines at indent two become forty-six at indent twenty-two for
five error exits (probes `w6`, `w7`), and `raw_deflate_dynamic_decode.wf`'s
`decode_dynamic` — 214 lines, seven `propagate` sites, three live heap runs — becomes
seven hand-maintained cleanup lists that differ per site, because by line 296 one of the
runs has been moved into a callee and disposing it is a use-after-move. **3.S [S28]
proposes `on_propagate { ... }`, one section per scope**, and the alternative it
replaces is Q10's own earlier answer, a per-statement release list, which keeps six of
the seven copies. **The owner should choose between the section, the list, and leaving
the refusal**, and this draft recommends the section, landed in the same batch as D1
rather than after it, because D1 is what makes the refusal common.

**Q11 is answered and is retained only as a record.** A view-forming borrow needs no
written region: the region relates nothing, so the amendment elides it and the
enclosing block keeps its braces and loses its name — which the build in 6.1 now
enforces.

**Q12 is answered by the owner.** [S25] is adopted: `reserve_file` becomes fallible,
on the principle that a failure the environment can produce is a typed value.

**Q13. A run whose element type is linear *by declaration* has no route out**, and
§2.1's release row now marks the notion open at exactly this shape. It is not a nominal,
so the destructuring consume does not reach it; it has no capability-released leaf, so
`dispose` refuses it; and it cannot be moved out of the function that built it. A writer
meets it the moment they put a lease, a ticket or a transaction into a `FixedVector`.
This design avoids the shape by putting the obligation on the value that is handed out
and not on the container of spares (3.L.4), which is the right modelling and is not a
rule. **The principled fix is a fourth route: a run whose element type is linear is
discharged when it is proved empty — `len(v) <= 0_u64` at the scope exit — and a drain
loop's [INV-1] exact-exhaustion conclusion is what proves it.** That is one sentence and
it reuses machinery 3.L already writes; it is not proposed here because it needs a
falsifier pass of its own against a linear element type that is also capability-released.

**Q14 is new. Should [INV-1] admit `==` in a header invariant over measure terms?**
[INV-1] 3105 admits the four ordered symbols, so an **exact** measure relation costs
**two** header invariants: `collect` carries five where three would do, `bs_reserve`
seven where four would, and `render` five where three would. [CALL-7] makes a complete
hand-back contract mandatory, so this cost is now paid by every helper a loop calls.
The relation is still one `compare_op` performing no [OP-1] operation, [INV-1]'s own
normalization already handles both directions, and [FN-9] 1312 admits all six in a
contract clause already. **Recommend admitting `==` for an invariant whose operands are
measure terms**, which is the smallest change that halves the invariant count in every
function this design prints. It is not proposed in 3.S because this design does not
need it to be correct — only to be writable.

**Q15 is new. Should L18 gain a cost clause, and then one bulk-move row?** L18 asks
only whether a writer *can* express the effect. For a byte copy the answer is yes and
the cost class differs by about two orders of magnitude: every growth policy, every ring
drain and every `collect` is element-at-a-time, roughly five operations per byte
facts-off where a `memcpy` is a small fraction of one. Round 6 proposed the law read
*a rule enters the kernel when no wf program has its effect, **or when every wf program
that has its effect has a strictly worse cost class and the measurement is recorded***,
and then one row, `seq_move_prefix(dst: own V, src: own V, count: own u64) -> (dst2,
src2)` publishing all four measures on both runs. **Recommend the law change and the
row, and recommend that the measurement be taken before either lands** — this design has
measured no timing anywhere and would not start with its own proposal.

**Q16 is new. Should a destructuring consume be able to bind some fields and dispose
the rest?** [S13] binds **every** field with no `_`, so taking one page out of a
five-field record is five binders and a `dispose`, three of them dead, and a
three-level nest is three such statements — the ceremony round 4 asked to have removed
and which has been moved rather than removed. The shape round 6 proposed is
`let Chunk(page: page, ...) = move chunk;`, disposing every unnamed field by exactly
[PROV-6]'s walk. **Recommend it as a later convenience and not now**: L18 says a
convenience is not a rule, the walk it needs already exists, and the cost is
verbosity rather than expressibility.

**Q17 is new, and it is a cost this design pays rather than a question it avoids.**
[MSR-3]'s denotation table makes a `&uniq` parameter's measure **inadmissible in an
`ensures`**, which is what closed round 6's second BREAK. The consequence is that **a
user `fn` that lends a provider onward can publish nothing about that store's
post-state**: a caller's `room(scratch)` fact dies at the call and every subsequent
proved acquisition in that caller is undischargeable, so an arena-lending helper forces
its caller to the checked spelling. [PROV-2]'s justifying sentence is corrected to say
so. The alternative — admitting such a clause for a user `fn` — is exactly the
caller-side claim L11's second sentence forbids, so it is not available while L11
stands; what *is* available, and what a later batch should weigh, is **restricting
[EFF-1]'s `reads` and `writes` paths to formals reached through a borrow plus any path
whose leaf is a provider or a loan-bearing type**, which round 6 measured as one or two
items removed from every value-in / value-out signature in the library and a whole class
of wrong-effect-row defects removed with them. *Recommend it, and record that it is an
[EFF-1] amendment with a wide blast radius that no current experiment needs.*

---

## 6. Verified versus reasoned

**Verified** means a compiler executed it, against a gate-profile `whitefootc` built
from this tree, in this session or in one of the twenty-one falsifier sessions whose
probe names are quoted. **Probes named `r1` to `r8` were run in this session**; probes
named `d1`, `e1`-`e8`, `v1`-`v3`, `c1`-`c3` and `m1` are round 6's memory-lens set,
`a1`-`a5` its resource-lens set, `w1`-`w9b` and `x1`-`x15` its writer-lens set, and
`t1`-`t14`, `q*`, `k*`, `n*`, `p*`, `f*`, `g*`, `m*`, `s*` and `r2_*` are the earlier
rounds'. **No name denotes two probe sets**, which the sixth draft's own §6.1 did.
No timing figure appears anywhere in this file.

**Two binaries, and the difference between them is itself a result.** The v0.41 gate
built from this tree is the baseline for every verdict below. A second build made
later in this session **implements the region-spelling amendment 3.K.0 assumes**, and
rejects a written region name at a position no other position of its declaration names,
citing `[FORM-8] RegionSpelling` with the mechanical fix *"drop the region name: no
other position of this declaration names this region, so the position denotes one
region of its own"*. Where the two disagree the table says so.

### 6.1 What the compiler did in this session

```text
| probe | program                                                        | verdict                                   |
|-------|----------------------------------------------------------------|-------------------------------------------|
| r1    | D1 verbatim: `replace deref(handle)` in a callee through        | **ACCEPTED, exit 0** on the v0.41 gate    |
|       | `&uniq 'a buffer<u8>`, caller subscripts offset 9              |                                           |
| r5    | the same program with every region name elided                 | **ACCEPTED, exit 0** on the amended build |
| r1    | the same program with its region names written                 | REJECTED [FORM-8] RegionSpelling on the   |
|       |                                                                | amended build                             |
| r2    | `set c = bump(cell: move c);` at a live affine local           | REJECTED [STOR-1] AffineSetTarget, on     |
|       |                                                                | both builds                               |
| r3    | `fn split(v: own u64) -> (low: own u64, high: own u64)`        | REJECTED [GRAM-2] at parse, expected      |
|       |                                                                | IDENT, on both builds                     |
| r4    | `let total = 0_u64;` then a nested `let total` under it        | REJECTED [TYPE-6] DeclarationCollision,   |
|       |                                                                | spelling "total", on both builds          |
| r6    | a const generic `n` read as a value in `buffer_new(n, 0_u8)`   | REJECTED [TYPE-5] at resolution           |
| r7    | a subscript after a loop whose body writes only ELEMENTS of    | **ACCEPTED, exit 0**                      |
|       | the same run                                                   |                                           |
| r8    | `let b = move a;` for a copy `u64`                             | REJECTED [OWN-1] MoveOfCopy               |
```

What each establishes, and which rule it decided rather than confirmed.

**`r1` is D1 at this tip and it is still an unsound accept**, which is the whole reason
this design exists. **`r1` against `r5` on the amended build is the result worth
stating twice**: the region-spelling amendment has landed and is checkable, it makes
the elided spelling the only legal one, **and it changes nothing about D1** — the same
program, spelled the way 3.K.0 requires, is accepted. Only [BLK-4]'s fourth clause
refuses it, which is why R1 had to become a rule and could not stay doctrine. **`r2`**
is the shape [LIV-2] admits and [STOR-1] 679 refuses today, and its mechanical fix
("bind the result under a new let, and combine it with the old value field by field") is
exactly the ceremony the one commit rule removes. **`r3`** is the ordered result list.
**`r4`** is round 6's DEFECT 3 executed: the sixth draft's multi-target `set` made every
later target a declaration, and a declaration of a spelling a live outer binding already
has is a collision — the compiler's own message says a binding whose value was moved is
still a live *declaration* — so 4.2's central statement did not resolve. D2's
per-target resolution is what removes it. **`r7`** is [MSR-2]'s first consequence
across a loop: an element write does not kill a length, and the subscript after the loop
is accepted. **`r8`** is the copy/affine wall from the other side and is why [S27]'s
`slice` operands are written without `move`. **`r6`** is [MSR-6].

Inherited verdicts this draft rests on, by what each group establishes. Round 6's
memory lens: `d1` is D1 accepted; `e2` is a callee whose `ensures` names a borrowed
run's measure across a `replace`, **rejected** by [FN-9]'s entry-image-stability
paragraph, and `e3` is the same program with the mutation deleted, **accepted** — the
pair that locates [MSR-3]'s `&uniq` row to the byte; `v1`/`v1b` show a function
returning either of two same-region view parameters is legal today; `v2` shows [SET-2]
518 is the only rule refusing an installed view at v0.41; `c1`, `c2`, `c3` and `m1` are
the declaration-collision and match-field machinery that makes [MSR-6] and [S13] safe.
Round 6's resource lens: `a1` is a `pure`, heap-free arena loop losing store bytes per
iteration and **accepted** with no allocation effect anywhere — the program [RES-10]'s
consumable-budget rule now refuses; `a2` is a live heap value across an arithmetic
`propagate`, **accepted**, which is the shape [PROV-6] refuses and [S28] would relieve;
`a4` is `[OP-9] UndischargedAllocationFitObligation` on a run whose count comes from the
environment, which is the judgment [BLK-0] restores to the formation rows. Round 6's
writer lens: `w1` against `w2`, `w3` and `w5` price [CALL-2] exactly — a chain of
published relations composes to depth five and a chain without them fails at the
*second* link; `w6` against `w7` measure the `propagate` cost; `w8` accepts a `set` at a
`match` arm binder; `x1`, `x2` and `x3` are [CALL-4]'s three rejections and `x13` is an
admitted variant route read at a caller's arm; `x11` and `x12` are the move-rebind and
enum-payload measure losses [MSR-3]'s new placements close; `x14` against `x15` is the
copy/affine wall with its diagnostic inside a library body; `x10` shows an element
borrow is `Semantics/Unsupported: RegionsAndBorrows`.

From the earlier rounds: [CALL-1], [CALL-2] and [CALL-5] already behave and the
struct-field route already kills correctly (`p1`, `p6`, `f7`, `m04`, `s7`).
`mut_slice` writes, affine elements and multi-return are new capability rather than
compiler defects (`p7`, `p9`, `k12`, `p2`, `p8`, `k09`, `r1_multi`). Allocation while
holding nothing, and a free inside a `pure` callee, are accepted (`p5_ambient`, `n4`,
`r1_ambient`, `r2_5`, `q9`, `w7`, `m02`). A view value, not its argument borrow, holds
the loan (`f1c`, `f1d`, `f2b`, `r1_twouniq`, `r2_1`, `r2_2`, `c4`, `w8`). [LIV-1]
replaces three avoidances (`f3`, `f5`, `f6`, `r1_own11`, `s5`, `s6`). The syntactic tail
conditions are refuted (`f2b_tail`, `f8_tailframe`, `p3_rec`) and the idle and driver
loops are `FunctionFallthrough` (`n2_idle`, `f3_forever`, `k30`, `n3_propagate_loop`).
[BLK-4]'s nominal region parameters are new syntax (`f7_regionresult`, `r2_6`, `m05`).
The measure kill is root-granular today (`r2_4`, `r2_4b`, `r2_4c`); element-position
replace keeps a `len` (`r2_7`, `k24`, `n13`); a partial move kills the root and its
residual is freed (`q3`, `q7`, `x4`, `g7`, `p6_partial`); no loop publishes `len = N` as
an equality (`n14`, `n15`, `n19`); a by-value transformation is not `pure` (`c8`);
[PROV-7] has a reason (`r1_relend`, `r1_relend_affine`, `m19`); the fill loop's
arithmetic and its two- and three-invariant shapes are accepted (`k21`, `k21b`, `k08`,
`k31`, `x1c`, `x1d`, `g4`) and the three-term header without a published relation is not
(`g3`); `+checked` publishes only for a constant addend (`g1`, `g2`); the arena-content
stop, the recursive region and the release walk's `realloc`'d worklist with its
`wf_resource_abort` are all executed (`a1`, `a5`, `a6`, `a8`, `x6`, `x8`, `p2_recarena`,
`p3_rectype`); `reserve_file` lowers to `ret i1 true` and the io_uring adapter reserves
an entry on every submission (`p1_reclose`, and the three source reads of 6.2); and
`par` eligibility plus three disjoint chain roots are the ledger read (`n7_par`,
`--stack-ledger`).

### 6.2 The runtime sources this design reads

Four reads, because [RES-7], [RES-9], [RES-1] and [STK-3] are stated over them.

```text
| source                                                   | what it shows                                   |
|----------------------------------------------------------|-------------------------------------------------|
| completion/linux_io_uring.c:425-450, 587-640             | every submission calls wf_linux_reserve_entry   |
|                                                          | on a fixed entry_capacity table and waits when  |
|                                                          | it is full; WF_LINUX_FILE_CLOSE is one of the   |
|                                                          | submitted request kinds                         |
| completion/bridge.c:660-720, 780-796, 900-1240, 1504     | read_at, write_once, open_file, open_read,      |
|                                                          | open_directory, open_directory_source and       |
|                                                          | directory_next all take that path, and so does  |
|                                                          | wf_bridge_submit_linux_close; the adapter       |
|                                                          | initializes under pthread_once inside submit    |
| completion/linux_io_uring.c:242-300                      | the adapter holds three mmaps, sized from the   |
|                                                          | kernel's returned ring parameters               |
| emitter/system.rs:2892-2901, backend/wf_floor.c:55, 78,  | reserve_file lowers to `ret i1 true`; the floor |
|   234-247, 303-329                                       | mmaps a 64 KiB alternate stack per attaching    |
|                                                          | thread, creates the entry thread and falls back |
|                                                          | silently                                        |
```

The first two are why [RES-7]'s column is derived from the `may-suspend` target
contract **and quantifies over [SYS-5]'s release actions as well as [SYS-2]'s
operations**: a `ReadFile` close is a may-suspend action that reserves from the same
fixed table every read uses, and the sixth draft counted none of them. The third is why
[RES-1]'s host-object class is drawn at *countable versus extent* and a runtime mapping
is a `region` item. The fourth is why [RES-9]'s store is a design addition rather than a
compiler defect, why [STK-3] materializes the entry stack, and why 4.1's envelope
carries a `region floor.altstack` item.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in 3.K.** None is implemented, and no compiler has seen any of the
  new types, operations, terms, statements, modifiers or markers.
- **Every function in 3.L** and **every program in section 4**, written against 3.K
  and the unchanged v0.41 rules and walked against both, with each loop's head state
  and each obligation's publishing function stated; none was compiled.
- **Every figure in 4.1's envelope**, which is why every one is written as a
  composition or as `<post-codegen>`.
- **[CALL-6]'s S13.** The newest rule in the file and the one everything else reads.
  That its four parts — source, substitution, destination, support — are the four an
  [ENT-3] source needs, that its provider-relation admission is exactly as wide as
  [PROV-2]'s argument and no wider, and that widening [FN-9] 1313 there readmits nothing
  L11 forbids, are argued and not executed. **6.4 asks for this first.**
- **[PROV-1]'s brand.** Argued from rule text — and all six falsifier rounds attacked
  it from every position they could build and none moved it, which is the strongest
  evidence any part of this design has.
- **[BLK-4]'s fourth clause.** That refusing a container nominal or a loan-bearing type
  as the referent of a `&uniq` parameter of a source-declared `fn` costs no program a
  writer needs, that [PROV-4]'s closure is the right depth, and that scoping the clause
  to source declarations is principled rather than an exception for `seq_mut_slice` and
  `read_at`, are argued. Probes `r1` and `r5` are the program it refuses, accepted today
  under both spellings.
- **[PROV-6]'s ownership closure and its `dispose` operand domain.** That "owns" is the
  right closure, that a loan-bearing value owns nothing, that the two loan-bearing
  conditions on `dispose` are not redundant, and that the declaration-linear exclusion
  costs no legitimate program are argued. Probe `a8` is the mechanism the walk replaces.
- **The `dispose p;` resolution.** That a store region names one store, a store one
  provider, and a program point at most one live `&uniq` binding to it, together make
  the capability determined rather than chosen, is argued from [PROV-1], [PROV-2] and
  [OWN-5] 606 and is not executed.
- **[LIV-2]'s one commit rule.** That one statement subsumes [SET-1]'s overwrite, the
  rebind and the transformation, that the non-overlap condition is the right refusal for
  two subscripts of one run, and that a target naming a binding in scope keeps its term
  while a fresh identifier declares are argued. Probes `r2` and `r4` are the two halves
  the rule replaces.
- **[BLK-1]'s window and `seq_rebase`.** That `head` costs exactly the five things
  [BLK-1] lists and that a rotate-in-place is the right escape from the absorbing state
  are argued and not executed.
- **[RES-10]'s algebra.** Its sequence, branch and call rules over a label map are
  standard; the fixed route order, the per-algebra transfers, the `return` label, the
  may-suspend transfer and the consumable-budget restriction are all new this draft and
  none has been composed against a program by hand. **6.4 asks for this second.**
- **The compiler defect at `[SET-2]`'s arena half**, found in round 3 and confirmed
  since: [SET-2] 517 makes a region-bearing `replace` target a hard error for
  `slice<'r, U>` **and** `arena<'r, U>`, and `check_mutation_target_class`
  (`compiler/src/semantic/check/expressions.rs:310-326`) tests only the slice variant.
  It is benign at this tip and load-bearing for the batch that implements [PROV-3] use 3
  and [VIEW-4].
- **[MSR-3]'s six placements**, checked by enumeration and not by execution; **the
  current runtime's closure**, which no existing target can be certified to meet; and
  **the claim that `wfgrep` becomes heap-free**, whose substitution was never compiled.

### 6.4 Falsifiers this design asks for next

1. **Attack [CALL-6]**, which is the newest and most load-bearing rule here: find a
   declared relation whose substitution S13 does not determine, a destination two of the
   five clauses both claim, a support S13's sentence gets wrong at a joined arm, or a
   provider relation the admission lets through that L11's second sentence forbids.
2. **Hand-execute [RES-10]** on the corrected 4.1, on 3.L.4's pool, on a divergent
   service loop and on an arena take inside a loop, checking the route order, the
   per-algebra transfers, `return`, `retained` and `reset` against all four. No round has
   run a program through this rule.
3. **Attack [BLK-4]'s fourth clause**: find a program a writer needs that it refuses,
   a `&uniq` parameter that reaches a container through a path [PROV-4]'s closure does
   not walk, or a compiler-owned row whose behaviour is not fixed by its record and for
   which the source-declaration scoping is therefore wrong.
4. **Attack [CALL-7]** with a helper whose complete contract is unwritable — a result
   built on two arms with no common exact relation, a measure the body establishes only
   under a branch — and with the two-invariant cost, to see whether Q14 is a convenience
   or a requirement.
5. **Attack [PROV-6]'s `dispose` resolution** with two live bindings of one provider
   type at different depths, with a provider reached only through a reborrow, and with a
   type whose leaves name two stores of which one provider is out of scope.
6. **Write 3.L against 3.K by hand, one function at a time, and find the tenth kernel
   addition.** Rounds 5 and 6 each found the yield high.
7. **Attack the `linear` modifier** with a type that is linear by declaration and
   affine by criterion inside a generic, and with Q13's run of declaration-linear
   elements, which §2.1 now marks as the release notion's open shape.
8. **Rewrite `wfgrep` and `byte_string` by hand** against [VIEW-7], [PROV-6], R1,
   3.K.0's landed amendment and Q10's refusal, and count what remains.
### 6.5 Falsifier round 1: what each finding hit, and what refuses it now

**6.5 to 6.9 are carried from the sixth draft and are history**, and they are written in the vocabulary of the draft that made
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
| the ceremony of §2.1          | eight notions; ten in this draft, with publication      |
| the spellings of 3.K          | unchanged in the rules; all of them DECIDED in this     |
|                               |   draft's 3.S, which is a decision record               |
```

### 6.10 Falsifier round 6: what each finding hit, and what refuses it now

Every BREAKS, GAP, DEFECT, BLOCKING, FRICTION, INCONSISTENCY and NOTE of the four
round-6 reports, one line each. Round 6's diagnosis was one sentence in four voices,
and it is one level out from round 5's: **a claim that carries a premise past the
judgment that was supposed to establish it**, and, in the fourth voice, **a
`Publishes:` line that is asserted and never constructed.** 3.K.11's seventh condition
and §2.1's tenth notion are this draft's answers to the pattern; the rows below are its
answers to the instances. The reports are superseded.

**Five dispositions the sixth draft's own §6.9 claimed and round 6 falsified**, recorded
because a false disposition is worse than an open finding: I1-I7's "every citation
re-derived" (six were wrong and fifteen ranges ended on a blank line); I18/I19's "both
corrected in A.2" (the free identifiers were not); F1-8's "4.1 rewritten ... the queue
loop carries a header invariant" (the invariant is false); F5-6's `advance<T>` (true of
the formula, while the transfer table stayed `+1` for every domain); and I27's
"`ConfinedFieldWithoutRegion` is stated inside [BLK-4]" (it was claimed and stated
nowhere). Each is corrected in the text of this draft rather than in a table.

```text
| F1 (memory and fact soundness)                                 | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| 1 BREAKS [VIEW-4]'s length-fixedness is a claim its judgment    | **[BLK-4]'s fourth clause** refuses the `&uniq mut_slice`   |
|   does not make; [LIV-2]'s exchange at a `deref` of a `&uniq`   | parameter at its declaration, so no callee installs a view; |
|   mut_slice installs a shorter view and [CALL-3] keeps the      | **[CALL-3]** is restated over what a view can WRITE, which  |
|   caller's length                                               | [PROV-3] and [EFF-1] 1386 judge; **[VIEW-4]** keeps only the|
|                                                                 | `replace` refusal and its headline claim is deleted. The    |
|                                                                 | type-level view length F1 recommended is NOT adopted        |
| 2 BREAKS [MSR-3]'s entry datum republishes an entry fact at a   | **[MSR-3]** gains the denotation table: a `&uniq`           |
|   post-kill point, and no rule refuses the `&uniq` run          | parameter's measure is **inadmissible in an `ensures`**,    |
|   parameter                                                     | and at a caller a parameter operand substitutes to the CALL |
|                                                                 | datum and never to a live term. **[BLK-4]** makes R1 a rule |
| 3 BREAKS a view is own-mode and owns nothing, so containment    | **[PROV-6]**: the linearity closure is under **ownership**, |
|   makes `slice<'r, Vector<u8>>` linear and `dispose` frees the  | and a loan-bearing type owns nothing [PROV-3, L10];         |
|   caller's runs through a shared loan                           | `dispose`'s operand must be rooted in a non-loan-bearing    |
|                                                                 | own binding and its walk may traverse no loan-bearing value |
| 4 BREAKS `dispose`'s walk silently discharges a                 | **[PROV-6]**: `dispose` is admitted only when no leaf is    |
|   declaration-linear leaf                                       | linear by the modifier or owns one; the fix is an admission |
|                                                                 | condition and not a walk row, because there is no action to |
|                                                                 | run and inventing one would make the modifier a destructor  |
| 5 GAP [VIEW-2]'s head premise does not cross a function         | **[CALL-7]** makes a hand-back contract complete over every |
|   boundary; both `seq_slice` sites in 4.2 are undischarged      | measure, **[VIEW-2]**'s premise weakens to the non-wrap     |
|                                                                 | form, and 3.L.3 carries `invariant flat`                    |
| 6 GAP 4.1's `slots` invariant is false and its proof invalid    | 4.1 rewritten: `drain` is the **checked** spelling, so the  |
|                                                                 | invariant is deleted rather than repaired, and `render`     |
|                                                                 | publishes `written == 8_u64`                                |
| 7 GAP `swap_with_last`'s requires is off by one, so L18's       | 3.L.2 states the obligation correctly at                    |
|   removal rests on a program that does not compile              | `at + 2_u64 <= len(vector)`, records that the last position |
|                                                                 | needs a dominating branch, and **L18 gains the sentence**   |
|                                                                 | that a removal is priced against a walked program           |
| 8 GAP/DEFECT [PROV-2]'s justification is false; one spelling    | **[MSR-3]**'s denotation table states one denotation per    |
|   has three denotations                                         | position; **[PROV-2]**'s sentence is corrected and the cost |
|                                                                 | recorded as Q17                                             |
| 9 GAP the variant route and the ordinal route do not compose    | **[CALL-4]**: a routed clause names its ordinal binder,     |
|                                                                 | `when b is V(f: r):`, and may omit it only when one ordinal |
|                                                                 | has that enum type; the precedent is [VIEW-6]'s refusal     |
| 10 GAP F5-13 reopened at a WRITTEN nominal region parameter     | **[PROV-6]** states the obligation at the declaration over  |
|                                                                 | the whole region parameter list, of which 3.K.0's elided    |
|                                                                 | sentence is one instance                                    |
| 11 GAP a run of declaration-linear elements has no exit         | recorded as **Q13 and marked OPEN in §2.1's release row**,  |
|                                                                 | with the empty-run fourth route recommended and not adopted |
| 12, 13, 14, 15 HOLDS (the window's arithmetic, rotation under   | preserved and not weakened; attack 13's wording note is     |
|   a live view, const generics, the destructuring consume)       | taken — [PROV-3] use 3's verb list gains **consume**        |
| 16 DEFECT 3.L.4's premise count                                 | corrected: two premises and a bridge, and the exit          |
|                                                                 | statement is not the backedge budget                        |
| Part 2: round-5 findings 1, 2 (views), 4 (residue), 6, 8, 9, 12 | each is one of attacks 1-6 and 10 above; findings 3, 5, 7,  |
|   REOPENED                                                      | 10, 11 and part-2 finding 1 HOLD and are not weakened       |
```

```text
| F2 (resource-closedness)                                       | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| F6-1 BREAKS an arena take in a service loop is certified        | **ACCOUNTING, by the owner's ruling.** [RES-5] gives every  |
|   bounded by route (ii) while the store exhausts                | domain a **kind**; [RES-10]'s capacity route applies to a   |
|                                                                 | REUSABLE-CAPACITY domain only, because a store's refusal    |
|                                                                 | bounds what is held and says nothing about a spent budget.  |
|                                                                 | The program is refused at its loop. F2's re-keying of       |
|                                                                 | linearity and its `abandon` statement are **REJECTED**: the |
|                                                                 | criterion stands and frame- and region-reclaimed values are |
|                                                                 | affine                                                      |
| F6-2 BREAKS `linear` is must-consume; a pool needs must-return  | **Correct, and Q0b's claim is deleted.** 3.L.4's            |
|                                                                 | `pool_release` is the **proved** spelling, so there is no   |
|                                                                 | refusal arm to destructure and the lease has one route on   |
|                                                                 | every path; [RES-6] and 3.L.7 state the general rule. The   |
|                                                                 | lease's run stays arena-backed: making it pool-backed needs |
|                                                                 | a third kernel store, which is [S18]'s rejected alternative |
| F6-3 BREAKS 4.1's invariant is unprovable and `drain`'s         | as F1-6; `render` publishes an exact result bound and 3.L.0 |
|   requirement undischarged                                      | carries the general sentence about a lower bound            |
| F6-4 BREAKS the derived column misses [SYS-5]'s may-suspend     | **[RES-7]** quantifies over **actions**, so it reaches      |
|   release actions                                               | [SYS-5]'s three; **[RES-10]** gains the may-suspend         |
|                                                                 | transfer `(peak 1, delta 0)` so the composition has a site  |
| F6-5 BREAKS the exclusion test is a source rejection reading a  | **[RES-7]** splits at the stage boundary: the source half   |
|   runtime-published row                                         | publishes a declared demand per store, the capacity match   |
|                                                                 | is [QUAL-2]'s, and 3.K.7.1 carries it at step 5 and step 9  |
| F6-6 BREAKS every formation row drops [OP-9]'s allocation fit   | **[BLK-0]** requires it of every acquiring row and A.2      |
|                                                                 | carries `requires fits::<T>(count)` on all three            |
| F6-7 BREAKS route (iii) is not a function and the deltas are    | **[RES-10]**: the routes are tried in a **fixed order**,    |
|   levels                                                        | each discharges the **backedge delta**, and a saturating or |
|                                                                 | invariant-discharged loop publishes `delta = 0`             |
| F6-8 BREAKS [RUN-3] deletes two non-footprint denials           | **[RUN-3]** replaces the FORM enumeration only; 1981's      |
|                                                                 | exit-edge and borrow-forming denials are kept as premises   |
| F6-9 GAP L6 and [PROV-5] disagree; `stack` carries no alignment | **[RES-2]**: `stack(context, bytes, alignment)`, and        |
|                                                                 | [RUN-4] creates each stack at both figures                  |
| F6-10 GAP `saturating(p)` is keyed to a parameter the shape it  | **[RES-8]**: keyed to a **store region**, checked ONE way   |
|   was built for does not have                                   | (declared implies exhibited), and the free-list route-(ii)  |
|                                                                 | sentence is deleted from `RESOURCES.md` rather than repaired|
| F6-11 BREAKS `retained` composes two ways and loses the         | **[RES-10]**: the retained-specific clause is deleted and   |
|   pre-loop acquisition                                          | every label composes by the one formula                     |
| F6-12 GAP `handle(kind, count)` cannot carry an mmap; 4.1's     | **[RES-1]** draws the class at countable-versus-extent, a   |
|   envelope omits the floor's alternate stack                    | runtime mapping is a `region` item, and 4.1's envelope      |
|                                                                 | carries `region floor.altstack`                             |
| F6-13 GAP the cyclic refusal is over containment, not the walk  | **[PROV-6]** states it over the **capability-released-leaf  |
|                                                                 | graph**, so an arena- or frame-backed recursive structure   |
|                                                                 | with an empty walk keeps compiling                          |
| F6-14 GAP logical versus physical coordinates are unstated      | **[MSR-1]** states the coordinate system once and carries   |
|                                                                 | the injectivity sentence; [PROV-3] and [RUN-3] cite it      |
| F6-15 GAP `head` is absorbing, so a ring needs a permanent      | **[VIEW-2]**'s premise becomes the non-wrap form and        |
|   duplicate                                                     | **[BLK-3]** gains `seq_rebase` [S29]; [BLK-1]'s cost list   |
|                                                                 | has five items                                              |
| F6-16 GAP `propagate` is unusable in error-heavy code           | measured and recorded: **[S28]** proposes `on_propagate`,   |
|                                                                 | Q10 puts the three options to the owner, and [PROV-6]       |
|                                                                 | carries the measurement                                     |
| F6-17 GAP `advance<T>` names a runtime `count`                  | **[RES-3]** states it at the acquisition, with the value    |
|                                                                 | named, which is its own closed-expression sentence given a  |
|                                                                 | home rather than a new restriction                          |
| F6-18 GAP `retained` is exempt from call substitution; the      | **[RES-10]**: every entry is substituted, `retained` and    |
|   `par` rule names a quantity [RUN-2] forbids                   | `return` included, and the `par` rule is **deleted**        |
| F6-19 GAP the cleanup-scratch domain has no source site         | **[RES-5]**: the domain is **deleted**; the walk's frame    |
|                                                                 | cost is ordinary frame cost [STK-3] and [RES-1] lists it    |
| Round-5 re-verification: F5-1, F5-3 (reset), F5-4, F5-7 (heap), | preserved and not weakened                                  |
|   F5-9 ([CALL-5]), F5-10, F5-13, F5-15 REFUSED                  |                                                            |
```

```text
| F3 (consistency)                                               | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| 1 DEFECT [BLK-0] names an [ENT-3] source S13 no rule states     | **[CALL-6]** states S13 with its four parts, and §2.1 gains |
|                                                                 | **publication** as its tenth notion. This is the finding    |
|                                                                 | the whole draft is reorganized around                       |
| 2 DEFECT a provider's post-state relation is not an admissible  | **[CALL-6]** admits a relation over a `&uniq` state         |
|   [FN-9] clause and has no destination                          | parameter of a DECLARATION-DOMAIN row without a result      |
|                                                                 | datum, and gives it the actual's resolved place as its      |
|                                                                 | destination; a source-declared `fn` gets neither [MSR-3]    |
| 3 DEFECT 4.2's `set (kept.v, total) = ...` redeclares a live    | **D2**: a target that names a binding in scope is a         |
|   binding                                                       | **place** and is committed to. Probe `r4` is the collision  |
|                                                                 | executed                                                    |
| 4 DEFECT 4.1's queue invariant is false                         | as F1-6                                                     |
| 5 DEFECT 4.2's chain is broken at three links, one of them a    | 3.L.5 rewrites all three: `bs_new` publishes, `bs_reserve`  |
|   `replace` establishing nothing                                | publishes its result's own measures, and its tail           |
|                                                                 | **constructs** rather than replaces. 3.L.0 gains the        |
|                                                                 | discipline sentence                                         |
| 6 DEFECT the `head` premise is undischarged for every           | as F1-5; and it is [BLK-1]'s **fourth cost restated**, with |
|   loop-built run                                                | `invariant flat` in every construction loop                 |
| 7 DEFECT the effect row is written in two orders                | [EFF-1] 1369's canonical order everywhere; 3.L.0 carries it |
|                                                                 | as a discipline sentence and the register row states it     |
| 8 DEFECT [LIV-2] and [LIV-3] admit one statement two ways       | moot under **D2**: there is one statement and one           |
|                                                                 | consequence, and [MSR-3]'s atom-identity sentence is one    |
|                                                                 | line where it was three plus a diagnostic                   |
| 9 DEFECT 3.K.0's criterion has three scopes and two undecided   | 3.K.0 states the scope **once**, puts a `construct`'s       |
|   positions, and disagrees with ten call sites                  | arguments and a declaration's binders outside the           |
|                                                                 | criterion, decides a `region_stmt` binder, and §4 elides    |
|                                                                 | every determined argument                                   |
| 10 DEFECT [RES-10] is non-deterministic, its acquire transfer   | as F6-7, F6-1 and F6-11; the `return` label is added        |
|   is wrong for the bump algebra, and it has no `return` label   |                                                            |
| 11 DEFECT 4.2 calls `bs_reserve` with a name it does not have   | `bs_reserve`, `bs_new`, `pool_new`, `pool_take`,            |
|                                                                 | `pool_release`, `try_place` and `try_take` are **declared   |
|                                                                 | in 3.L**, and §4 calls nothing this file does not declare   |
| 12 DEFECT six wrong citations and fifteen blank-line ranges     | every range re-derived mechanically from the spec in this   |
|                                                                 | session; the six are [PAR-1] 1975→1981 and 1987→1993,       |
|                                                                 | [ENT-6] 3019→3015 and 3026→3024, [SYS-2] 2301→2283-2285,    |
|                                                                 | and [OWN-7] 630→629 for the overlap test                    |
| I1 §2.1's release sentence is unqualified                       | the row is marked **OPEN at Q13's shape**                   |
| I2 the acquire transfer against [RES-5]'s bump row              | as F6-7                                                     |
| I3, I4 [VIEW-2]'s `v`/`Z` renderings                            | [VIEW-2] writes `V` and `0_u64`; `Z` appears only in prose  |
| I5 two condition-1 failures and one condition-5 failure         | [PROV-1] amends [STOR-1] 675 and the batch-0079 row is by   |
|                                                                 | [RES-6]                                                     |
| I6 thirteen dependencies on both register lists                 | condition 4 says **and there only**; the third list is      |
|                                                                 | eight rows and every duplicate is removed                   |
| I7 [ENT-1] has a row in the changed table reading UNCHANGED     | removed; it is a third-list row                             |
| I8 A.2's free identifiers; `ConfinedFieldWithoutRegion`         | A.2 declares `bytes` and `align`; [BLK-4] states the        |
|                                                                 | diagnostic name                                             |
| I9 4.1's loop-body borrows against [OWN-11] 647                 | the [OWN-11] row states 647's disposition: it is vacuous    |
|                                                                 | once a borrow expression names no region                    |
| I10 §6.1's probe naming                                         | 6.1 names one set per binary and says which                 |
| I11 [VIEW-7] has no 3.S entry                                   | **[S30]**, PROPOSED, added this round                       |
| I12 [LIV-2] has no 3.S entry                                    | moot: it is **D2**, the owner's decision                    |
| I13 [STK-4]'s id is reused                                      | the retirement list says the second draft's reentrancy      |
|                                                                 | **premise** was deleted and no id retired with it           |
| I14 S16 is writable in wf                                       | 3.S records it: adopted on cost, not on expressibility, and |
|                                                                 | [CALL-4]'s other two halves carry L18                       |
| I15 [LIV-2] has no demander                                     | moot under D2: every function of 3.L and both programs use  |
|                                                                 | the one commit rule                                         |
| N1 statement forms versus productions                           | META-5 counts both, separately and by name                  |
| N2 [PROV-5]'s "every edge" against [STOR-3] 690                 | stated in [PROV-5] and on the [STOR-3] row                  |
| N3 [BLK-0]'s first-parameter sentence has no case for the view  | the sentence gains its third case: an operation that        |
|   formers                                                       | neither transforms nor provides names the value it observes |
| N4 [LIV-1] is in two batches                                    | §7 puts it in one                                           |
| N5 [MSR-4]'s `Amends:` names `prove_ordering`                   | replaced by the [FN-9] sentence it meant                    |
| N6 4.2's [OWN-10] 641 note is reversed                          | 4.2 states it correctly: the borrowed binding must exist    |
|                                                                 | before the block opens                                      |
| N7 §5's preamble and 3.L.0's missing banner                     | moot: 3.S is a decision record and 3.L.0 names it           |
```

```text
| F4 (writer)                                                    | disposition                                                |
|----------------------------------------------------------------|------------------------------------------------------------|
| 1 BLOCKING [BLK-0]'s completeness binds thirteen rows and not   | **[CALL-7]**, the rule this draft adds, with               |
|   the wf functions written over them                            | `IncompleteHandBackContract` at the declaration; 3.L.3,    |
|                                                                 | 3.L.4 and 3.L.5 carry the clauses and the invariants        |
| 2 BLOCKING the measure datum has placements for three of the    | **[MSR-3]** has six: entry, call, construct, **rebind**,    |
|   six naming events                                             | **payload** and **field**, all sharing the closure sentence |
| 3 FRICTION `propagate` with a live linear binding               | as F2 F6-16; **[S28]** and Q10                              |
| 4 FRICTION 4.1's header invariant is not preserved              | as F1-6                                                    |
| 5 FRICTION a multi-target `set` exchanges the first target only | **D2**: every target is resolved independently, so P8's     |
|                                                                 | demux writes three owners into their own places and needs   |
|                                                                 | no bundling nominal                                         |
| 6 FRICTION no bulk-move row; L18 says nothing about cost class  | **Q15**, open, with the law change and the row recommended  |
|                                                                 | and the measurement asked for first                         |
| 7 FRICTION `head` is absorbing                                  | as F2 F6-15: **[S29]** `seq_rebase`                         |
| 8 FRICTION the destructuring consume binds every field          | **Q16**, open, recommended as a later convenience           |
| 9 FRICTION linearity is per instantiation at a written region   | as F1-10 for the region axis; the **type** axis joins Q8    |
|   parameter and at a type parameter                             | as its third half                                           |
| 10 FRICTION `reads`/`writes` over an `own` parameter buy        | **Q17**, open, with the [EFF-1] restriction recommended and |
|   nothing                                                       | its blast radius recorded                                   |
| 11 FRICTION A.1 charges `cap` in a FixedVector descriptor       | A.1 charges `len` and `head` only: `cap` is the type        |
|                                                                 | constant [MSR-2] already makes a standing fact              |
| 12 FRICTION register condition 6 fails for [CALL-4]'s arm route | **[CALL-6]** and [CALL-4] both name the arm binder as a     |
|                                                                 | destination                                                 |
| 13 CLEAN (the window, [MSR-6], [S24], the construct placement,  | preserved and not weakened                                  |
|   [LIV-2] at an arm binder, R1's chains, [PROV-6]'s criterion,  |                                                            |
|   [PROV-1]'s brand, [MSR-4], [CALL-1/2/3/5])                    |                                                            |
```

**Where a sixth-draft rule's content went**, for a reader holding 6.5 to 6.9:

```text
| sixth-draft rule or row      | now                                                     |
|------------------------------|---------------------------------------------------------|
| [LIV-2] + [LIV-3]            | [LIV-2], one commit rule (D2); [LIV-3] retired          |
| [VIEW-4]'s length-fixedness  | deleted; [BLK-4] refuses the parameter and [CALL-3]     |
|                              |   reads what a view can write                           |
| [PROV-6]'s containment       | closure under **ownership**; a loan-bearing type owns   |
|   closure                    |   nothing                                               |
| `dispose p using (q1, ...)`  | `dispose p;`; the capability is determined by the brand |
| [MSR-3]'s three placements   | six, with rebind, payload and field                     |
| [BLK-0]'s completeness       | kept for the domain; [CALL-7] carries it for a wf fn    |
| the [ENT-3] source S13       | [CALL-6], stated rather than cited                      |
| [RES-5]'s five algebras      | four; the cleanup-scratch domain is deleted             |
| [RES-10]'s three discharges  | three routes in a fixed order, each over the backedge   |
|                              |   delta, and the capacity route for reusable capacity   |
| [RES-7]'s exclusion test     | a source-stage declared demand and a [QUAL-2] match     |
| Span, MutSpan                | slice, mut_slice; `slice` is copy [S27]                 |
| §3.S's 26 proposals          | a decision record; three items remain PROPOSED          |
```

---

## 7. Implementation order

**This is an implementation order and nothing else.** The owner's ruling of
2026-09-03 says so in terms: batches are an order of work, not spec versions, and a
single implementation is fine if it is correct. Nothing below is an approval, a
schedule, or a licence to trade a rule away for a cheaper batch; one batch that lands
all fifty-one rules correctly is the better outcome. The order is *for* naming, at each
step, a test writable before the next step exists. **Every rule is in exactly one
batch**, which round 6 found the sixth draft claiming for [LIV-1] and not doing.

**B0 is not one of these batches.** The region-spelling amendment (3.K.0) lands
first, separately and mechanically, and is not this design's work; every batch below
assumes it. **It has landed in a build**: 6.1's probes `r1` and `r5` show a written
region name at an undetermined position rejected at `[FORM-8] RegionSpelling` and the
elided spelling of the same program accepted.

**B1. The proof surface.** Rules: [MSR-1], [MSR-2], [MSR-4], [MSR-5], [MSR-6].
First because every later batch's contracts and invariants are unwritable without it,
and because it is a specification amendment with no new construct. Tests: probes
`t1`-`t3` and `r6` accepted after [MSR-6] and `t4` still accepted; a clause whose
operands are two `len` terms, accepted where probe `e1` is a [GRAM-5] parse failure
today; a literal and a parenthesized group still affine factors; a goal discharged from
`len + room = cap` as an affine premise; an element-position `replace` of a
*descriptor* killing its measures and of a *scalar* killing nothing, which is the
carve-out's removal under test; **`r2_4`'s program accepted**, because [MSR-2]'s
descriptor-precise support is a repair of a live over-kill; **and a subscript in logical
coordinates whose [OP-4] obligation is against `len`**, with [MSR-1]'s injectivity
sentence exercised by two disjoint ranges over one wrapped run.

**B2. Type-derived call transports.** Rules: [CALL-1], [CALL-2], [CALL-3],
[CALL-5]. Second because it is the live defect and needs none of the new types:
today's `&uniq buffer<T>` keeps its spelling and gets [CALL-5]'s type-derived
classification. Test: **`ent5-neg-callee-uniq-buffer-replace-kills-length.wf` turns
XPASS**, rejecting at [OP-4] with residual `9_u64 < len(line)`; plus probe `r1`'s
program, whose accept becomes the same rejection under both spellings; plus one positive
case pinning [CALL-1]. `docs/patterns.md` P16 is corrected in the same change. **This
batch flips a conformance case from `xfail`, which is conformance evidence; the
disposition is recorded in `governance/APPROVALS.md` with the merge**, as B6's
supersession is.

**B3. The publication surface, multi-return, and one commit rule.** Rules:
[CALL-4], [CALL-6], [LIV-1], [LIV-2]. Third because B6 and B7 are written in this
syntax and because **nothing downstream publishes anything without [CALL-6]**. Tests:
probe `r3`'s signature parses and binds, and a two-result contract reaches both binders
of a destructuring `let`, both targets of a `set` target list and both arms of a
`match`; **probe `r2`'s program is accepted**, which is the one commit rule at a bare
binding, and the same at a `deref`, a field and a subscript; a `set` whose two targets
are `v[i]` and `v[j]` **rejected** at [LIV-2]'s non-overlap condition; **probe `r4`'s
program accepted when the inner `let` becomes a `set`**, which is D2's per-target
resolution under test and round 6's DEFECT 3; a swap `set (p, q) = move q, move p;` and
a three-target rotation accepted; probe `f3`'s program a [LIV-1] error naming both
predecessors instead of `SemanticUnsupported`; a loop moving and restoring an outer
binding accepted where probe `f5` is [OWN-11] today; probe `x1`'s per-variant `ensures`
accepted and read at the caller's arm; **and a declaration-domain row's relation
establishing at a caller, with its provider post-state landing on the actual's resolved
place**, which is [CALL-6]'s two halves and which no earlier batch list contained.

**B4. Measure datums, images, and atom identity.** Rules: [MSR-3]. Separated from
B1 because it touches [ENT-2]'s term list, [ENT-5]'s call boundary and [ENT-6]'s
transfer machinery, and because it needs [LIV-2] from B3. Tests: a helper whose
`ensures` names `len` of a consumed `own` parameter is accepted and its caller
establishes the relation where `M(c,q)` refuses it today; **a helper whose `ensures`
names `len` of a `&uniq` parameter is rejected**, which is round 6's second BREAK under
test and which probe `e2`/`e3` locates today; **probes `x11` and `x12` accepted after
the rebind and payload placements**, and a destructuring consume's binder subscriptable;
a `set` target that names a binding in scope keeping its header invariant across the
backedge, with the fresh-identifier variant pinned apart; and a `construct` carrying a
measured operand publishing the field's measure.

**B5. Linearity, structural release, and the destructuring forms.** Rules:
[PROV-6]. Ahead of the container batch because D1's criterion is stated over release
actions the language already has and because every later test needs the diagnostics.
Tests: probes `r2_5`, `w7` and `m02` rejected with `LinearValueNotDisposed` and their
repairs compiling; **probe `x4`'s program rejected with `LinearValuePartiallyConsumed`**
and its destructuring-consume repair compiling; a `dispose` through a shared borrow
rejected at [OWN-1]; **a `dispose` of a `slice<'r, Vector<u8>>` rejected at the
loan-bearing operand condition**, which is round 6's third BREAK; **a `dispose` of a
type reaching a declaration-linear leaf rejected**, which is its fourth; a `dispose`
with no live provider binding rejected as `DisposeHasNoProvider` and accepted once the
parameter is added, with the resolved binding appearing in the effect row; a `linear
struct` whose value is dropped rejected and whose value is destructured accepted; probes
`w5` and `m03` rejected with `LinearValueAcrossPropagate`; **probe `x6`'s
self-referential heap type rejected at its declaration** in a program with no marker,
naming the cycle, **while its arena-backed sibling still compiles**, which is the walk-
versus-containment repair under test.

**B6. The brand, the runs, the window, confinement, and the declaration domain.**
Rules: [PROV-1], [BLK-0], [BLK-1], [BLK-2], [BLK-3], [BLK-4], [CALL-7]. Retires
`buffer<T>`, `box<T>` and `arena<'r, T>` from the writer surface. Carries
monomorphization for a compiler-owned generic domain. Tests: a `FixedVector<Handle, 64>`
object table with affine elements, filled by 3.L.3's `vacant`, accepted, where probe
`p9` is [OP-1] today; a `vacant` result whose `len >= n` discharges a subscript with no
equality anywhere; **a queue built from `seq_place` and `seq_take_front` with no
`Option` anywhere**, whose `len` is exact and whose elements are mutated in place; **a
`seq_slice` over a run that has had a front removal rejected, accepted after a
`seq_rebase`, and accepted over the same run drained to empty**, which is the non-wrap
premise and [S29] under test; **a helper that hands a run back without publishing its
`head` rejected at [CALL-7] with `IncompleteHandBackContract`**; `struct Chunk['s]`
accepted where probes `r2_6` and `m05` are parse errors today, with two instances at
different regions rejected as distinct types; **a `&uniq Vector<u8>` parameter rejected
at [BLK-4], and a `&uniq Env` whose `Env` holds a `FixedVector` rejected the same way**,
which is the wrapper defeat closed; and two reserving occurrences naming one region
rejected at the second. This batch supersedes B2's conformance case, whose program no
longer typechecks; that disposition is conformance evidence and is recorded in
`governance/APPROVALS.md`.

**B7. Views, loans, ranges.** Rules: [VIEW-1], [VIEW-2], [VIEW-4], [VIEW-6],
[PROV-3]. [PROV-3] lands here because views are its only user and because [SET-1] and
[SET-2] must change in the same batch that admits the `mut_slice` write. Tests: an
element write through a `mut_slice` accepted where probe `p7` is [SET-1] today; **a
`slice` used twice without `move` accepted and a `move` of one rejected at
`MoveOfCopy`**, which is [S27]; **a `replace` at a place holding a `mut_slice` rejected
by [VIEW-4]**, and so is a `replace` of a `Vector` place under a live origin set, which
probe `w2` shows the compiler accepts today for the arena spelling; two `mut_slice`s on
one run rejected at the second formation and two `slice`s accepted; a write to `k` while
a view formed at `table[k]` is live rejected citing the view's loan; and a two-result
signature with two same-region view results rejected at [VIEW-6].

**B8. Stores, the heap as a value, and reservation.** Rules: [PROV-2], [PROV-4],
[PROV-5], [PROV-7], [RES-6]. Tests: probe `p5_ambient`'s program **rejected**; a `main`
that omits `command.heap` cannot reach any allocation; a run released to a store of a
different region failing to typecheck with the two types rendered; a region block
entered twice by a loop republishing `len(store) = 0_u64` truthfully; **probe `x8`'s
program rejected with `ExtentReservedOnACallCycle` under `arena_extent` and accepted
under `arena_frame`**, with the graph read after [STK-1]; an arena-backed run of
`ReadFile` closing every handle at its scope exit; a helper lending a provider onward
compiling, where `r1_relend` and `m19` are [OWN-6] today; and two overlapped disposals
from one store denied [PAR-1] permission while a window containing one is not.

**B9. System I/O over views, and the handle table.** Rules: [VIEW-7], [RES-9].
Tests: `tests/programs/wfgrep.wf` migrated to 3.L.3's `filled` and `mut_slice`,
compiling with no `allocates` entry anywhere on its call graph — the first program that
demonstrates goal A's container half end to end; **a marked `main` selecting
`command.files` and `command.cwd` that opens one file in a loop, reads it into a
`filled` destination over a `mut_slice`, and publishes a handle demand of one**; **an
open that fails on every attempt, whose handle records all come back**; and **a
`ReadFile` close counted as a may-suspend acquisition**, which is round 6's F6-4 under
test and which no earlier batch list would have caught.

**B10. The stack judgment and the divergent entry.** Rules: [STK-1], [STK-2],
[STK-3], [STK-4]. Tests: probes `f2b_tail` and `f8_tailframe` **not** rewritten by
[STK-1]'s premise and rejected by [STK-2] under the marker; their borrow-free variants
rewritten into one dispatcher with one frame; a member holding a live linear binding
across the jump not rewritten, nor one that opens a region for an `arena_frame`; probe
`p3_rec` still accepted without the marker; a `--stack-ledger` run reporting one chain
per context rather than disjoint roots, **and the floor's alternate stack appearing as a
`region` item**; probe `f3_forever`'s idle loop accepted; **probe `n3_propagate_loop`'s
driver loop accepted**; and a loop with a reachable `break` still requiring a return.

**B11. The envelope and the judgment.** Rules: [RES-1] to [RES-5], [RES-7],
[RES-8], [RES-10], [RUN-1], [RUN-4], [RUN-5]. Tests: 4.1 source-resource-closed and its
`E` matching a pinned symbolic expectation, the `stack` item's alignment and the
`region floor.altstack` item included; 4.2 reported not resource-closed with the
heap-reaching path rendered; **an arena take inside a loop with no region block refused
at that loop, and the same take inside a per-iteration region block accepted**, which is
the consumable-budget rule and the reset transfer under test; a retaining loop whose
trip count is a runtime value rejected at that loop with the value named, and a
runtime-sized `seq_arena` rejected at the acquisition with `count` named; one loop with
both a constant trip count and a saturating acquisition publishing the trip-count map,
which pins the route order; **a vacuously true `len <= cap` invariant discharging
nothing**; a loop of four iterations followed by one more acquisition publishing a peak
of five and not two; the same loop with its acquisition one function down accepted
through a declared `saturating('s)`; **a service loop with no `break` whose pre-loop
acquisition appears in the enclosing `retained` entry**, which is F6-11; a peak reached
only on a returning path appearing in the `return` entry; B9's marked file program
composing its handle demand and failing **[QUAL-2] qualification** rather than a source
rejection when the profile cannot carry it; and a program whose demand exceeds every
profile row failing target qualification citing no language rule.

**B12. `par` and the envelope.** Rules: [RUN-2], [RUN-3]. Tests: a `filled` plus
`mut_slice` plus counted subscript fill receiving [PAR-2] permission in an unmarked
program, which needs the logical ranged origin and [MSR-1]'s injectivity sentence; the
same loop inside a `resource_closed` entry emitting no `par` construct and publishing
`lanes(1)`; two overlapped statements allocating from distinct providers permitted and
two from one provider not; a window containing a `dispose`, a destructuring consume and
a multi-result `let` **each judged by its own footprint**; and **a `break` and a
borrow-forming `let` between two members still denying permission**, which is F6-8.

**3.L is not a batch.** It is written against the rules, not implemented beside
them; where its functions are useful as evidence — `filled` in B9, `collect`, `vacant`
and the pool in B6 and B11, `bs_reserve` in B6 — they land as test programs under
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
carries its elements. **Every cell is one of `exact`, `bounded` or `absent`**, which
is what [MSR-1] requires.

```text
| measured type            | len                | cap             | room      | head       |
|--------------------------|--------------------|-----------------|-----------|------------|
| array<T, n>              | n, exact           | n, exact        | 0, exact  | 0, exact   |
| FixedVector<T, n>        | initialized slots, | n, exact        | cap - len,| window     |
|                          |   exact            |                 |   exact   |   origin,  |
|                          |                    |                 |           |   bounded  |
| Vector<'s, T>            | initialized slots, | slots taken,    | cap - len,| as above   |
|                          |   exact            |   exact         |   exact   |            |
| slice, mut_slice         | viewed elements,   | len, exact      | 0, exact  | 0, exact   |
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
publishes `head(result) = head(vector)` exactly, `seq_rebase` republishes
`head(result) = 0_u64` exactly, and only `seq_place_front` and `seq_take_front` publish
the two-sided `0_u64 <= head(result)`, `head(result) <= cap(result)`.

```text
| nominal                     | (size_ceiling, align_ceiling)                          |
|-----------------------------|--------------------------------------------------------|
| Heap<'s>, Arena<..>         | (32, 16)   proof-only representation, one word         |
| Vector<'s, T>               | (32, 16)   a descriptor: pointer, cap, len, head       |
| FixedVector<T, n>           | T's pair repeated n times, plus (16, 8) for len and    |
|                             |   head, with aggregate alignment max(align(T), 8)      |
| slice<'r,T>, mut_slice<'r,T>| (32, 16)                                               |
| array<T, n>                 | T's pair repeated n times, as [OP-9] 992 already fixes |
```

**A `FixedVector`'s descriptor carries `len` and `head` and not `cap`**, which is round
6's correction: `n` is the type constant and [MSR-2] already makes it a standing fact
with empty support, so charging a word for it costs sixteen bytes of metadata where
eight suffice, on runs as small as four bytes and per run in a struct of arrays.

`advance<T>` for the bump domain is `round_up(size_ceiling(T) * count, align)`, where
`align` is the store's own type constant and both acquiring rows require
`align >= align_ceiling(T)` as a compile-time comparison of two constants. There is
no fallback: the requirement refuses the other case rather than charging a ceiling
for it. Whether `count` is a closed expression is [RES-3]'s question.

### A.2 The kernel operation inventory

Thirteen rows, plus the four readers, which are [OP-1] table rows and not this
domain. `V` is either run type. Every row is complete over **every** measure it writes,
on every exit, as [BLK-0] requires, and every effect row is written in [EFF-1] 1369's
canonical order `reads, writes, allocates`. Every relation below is established by
[CALL-6]'s S13.

```text
Formation                                                                          [S7]
  seq_fixed<T, const n: u64>()                       -> own FixedVector<T, n>       pure
      len(result) = 0, cap(result) = n, room(result) = n, head(result) = 0
  seq_arena<T, const bytes: u64, const align: u64>['s](
        arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Option<Vector<'s, T>>       reads(arena), writes(arena), allocates(arena)
      requires align >= align_ceiling(T)
      requires fits::<T>(count)
      Some(value: r): len(r) = 0, cap(r) = count, room(r) = count, head(r) = 0,
                      <datum of len(arena)> <= len(arena)
                        <= <datum> + round_up(size_ceiling(T) * count, align)
      None:           len(arena) = <datum of len(arena)>,
                      room(arena) < round_up(size_ceiling(T) * count, align)
      both:           cap(arena) = <datum of cap(arena)>
  seq_arena_proved<T, const bytes: u64, const align: u64>['s](
        arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Vector<'s, T>               reads(arena), writes(arena), allocates(arena)
      requires align >= align_ceiling(T)
      requires fits::<T>(count)
      requires room(arena) >= round_up(size_ceiling(T) * count, align)
      as the Some row above
  seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)
      -> own Option<Vector<'s, T>>       reads(heap), writes(heap), allocates(heap)
      requires fits::<T>(count)
      Some(value: r): len(r) = 0, cap(r) = count, room(r) = count, head(r) = 0
      None:           nothing; a general store publishes no measure (L6)

Reservation                                                                        [S9]
  arena_frame<const bytes: u64, const align: u64>['s]()
      -> own Arena<'s, bytes, align>                                                pure
      len(result) = 0, cap(result) = bytes, room(result) = bytes
                          its contribution to stack(context, bytes, align) [PROV-5]
  arena_extent<const bytes: u64, const align: u64>['s]()
      -> own Arena<'s, bytes, align>                                                pure
      len(result) = 0, cap(result) = bytes, room(result) = bytes
                          its own region(name, bytes, align, contiguous) item [PROV-5]

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
  seq_rebase(vector: own V) -> own V                   reads(vector), writes(vector)
                                                                              [S29]
      len(result) = len(vector),     room(result) = room(vector),
      cap(result) = cap(vector),     head(result) = 0

Readers                       ([OP-1] table rows, not this domain)                 [S11]
  len(p) / cap(p) / room(p) / head(p)                -> own u64                     pure

Views                                                                             [S10]
  seq_slice['r, T](vector: &'r V)          -> own slice<'r, T>        reads(vector)
      requires head(vector) + len(vector) <= cap(vector)
      len(result) = <datum of len(vector)>, cap(result) = <datum of len(vector)>,
      room(result) = 0, head(result) = 0
  seq_mut_slice['r, T](vector: &uniq 'r V) -> own mut_slice<'r, T>    reads(vector)
      requires head(vector) + len(vector) <= cap(vector)
      as the row above
```

Two statements are not rows and are stated in [PROV-6]: `dispose p;` [S12] and the
destructuring consume `let N(f1: b1, ...) = move v;` [S13].

Notes on the inventory. **`seq_place` is the operation the whole design exists for**:
total under its requirement, allocation-free on every backing, one store plus one
length increment. **The four per-slot rows are two-sided because L12 is**, and the
front pair is what makes a queue a run rather than a run of `Option`. **`seq_rebase` is
the fifth boundary operation and the only addition this draft makes**, and without it
`head` is an absorbing state. **Nothing here is total at a capacity boundary**, because
an overwriting form would need L9's published displacement. **Nothing here removes from
the middle, clears, truncates, grows, exchanges, swaps, or constructs a filled or vacant
run** — each is 3.L, and 3.L.6 records that none needed a row the five boundary
operations do not have.
