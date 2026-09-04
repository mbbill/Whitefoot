# Containers and resources: the integrated design

The single design for batch 0116: one set of laws, one set of rules, one vocabulary, one
amendment register. `RESOURCES.md` beside it keeps the writer's-eye resource migrations
and `CONTAINERS.md` the longer library functions of 3.L; neither carries rule text, and a
reader who reads only this file has the whole design.

**Eighth draft, after falsifier round 7 and the owner's decisions of 2026-09-03 and
2026-09-04.** Round
7 confirmed the architecture — the brand, the partition, R1, R2, D1, D2, `[CALL-6]`'s
publication surface, the window, the copy `Slice` — and broke the *arithmetic* around it
in one shape, stated by two lenses in the same week. F1: **a fact computed at one point
and used at another, with the rule naming only the judgment and not the point.** F2: **a
repair that relocates the defect to the key rather than removing it.** `[MSR-3]` keyed a
declaration-domain operand's denotation on `writes` coverage instead of the parameter's
mode, so `seq_place` published `len(P) = len(P) + 1` and every loop in the file proved
`false` from `[MSR-4]` step 1. `[CALL-6]` established a routed relation *at* the arm
instead of restricting it *to* the arm, so a store's post-state outran its own kill.
`[MSR-2]` drew an element-write consequence that `[MSR-1]`'s own subscripted measure
place falsifies. `[CALL-3]` was stated over "the viewed element storage" for a view whose
elements are now descriptors. `[RES-8]`'s saturation fact was re-keyed to a store region
that no reusable-capacity domain has. Each is repaired at its root below, and 3.K.11
gains its **eighth condition** for the class: *a rule that computes a fact at one program
point and uses it at another states both points, and every quantity a rule tests is one
the language has.*

The owner decided two things this round, and both are written below as **decided, not
proposed**.

> **D3 (owner-decided 2026-09-03). Linearity is scope-relative.** A store-backed value is
> **linear only in a scope that does not hold the capability its release needs**. In a
> scope that holds it — a function whose signature carries `heap: &uniq Heap`, or `main`
> with the entry heap — the compiler **derives the release on every leaving edge**,
> exactly as `[STOR-3]` does today for a frame-resident value, and charges it to that
> scope's `writes(heap)`. A scope that does not hold the capability must move the value
> out or consume it. The `linear` modifier is unchanged and is a **logical** must-consume
> obligation in **every** scope. The capability cannot be smuggled, because it is in scope
> **by signature**: a function that gets the derived release says so in its parameter list
> and in its effect row.
>
> **D4 (owner-decided 2026-09-03). Every loop body is implicitly a region block.** A
> borrow of an outer binding inside a `loop_stmt` or `for_stmt` body is written **bare** —
> `&n`, `&uniq n` — and lives in that body's implicit per-iteration region; a
> `region_stmt` that is the loop body's only statement, named or not, is a `[FORM-8]`
> rejection unless a type argument inside it must write its name, because one semantics
> has one spelling; a block beside other statements keeps its own `[OWN-6]` statement
> scope and stays legal. `[OWN-11]`'s per-iteration guarantee is unchanged, because the
> implicit block ends every iteration. This is the v0.43 candidate's first amendment
> (branch `batch/0120`), drafted and tested but not yet merged.

**D3 is the largest change in this draft and it removes rather than adds.** Round 7's
writer lens counted what R2 costs when linearity is scope-blind: `[LIV-1]` is a
per-**edge** obligation, `byte_string.wf`'s `main` has eleven return edges and up to five
live heap values, so it needs **forty** `dispose` statements where the seventh draft
counted five, and `decode_dynamic` needs **sixty-eight** — and the repair a writer would
take is to invert every hosted function into a single exit, deleting early-return style
from hosted code. Under D3 all one hundred and eight disappear: those scopes hold the
capability by signature, so the release is derived on the edge it belongs to,
unconditionally, at `[LIV-1]`'s existing join. Nothing about L2 is weakened — the
capability is still a held value named at a parameter, and the free is now *more* visible
than forty scattered statements, because it appears in an effect row where today's
compiler emits it under **no** effect row at all (probe `r2_5`). `dispose p;` survives as
the **early** release a writer chooses, which no wf program can perform.

Three dispositions the owner made on the seventh draft's three open items:

- **`[S28]` `on_propagate` is REJECTED and is removed from this file.** It fails against
  its own motivating program: `propagate` is seven of `decode_dynamic`'s thirty abnormal
  edges, the live linear set changes four times inside one lexical scope so one section
  per scope cannot discharge it, an inner and an outer section each pass their own
  per-point check and double-free on one edge, and after `[BLK-4]` every one of those
  seven sites is a multi-result call that `[ERR-3]` 1472 cannot propagate. **D3 removes
  the problem instead**: a `propagate` in a scope holding the capability runs the derived
  release on its error edge, `[STOR-3]` 690's own edge enumeration.
- **`[S29]` `seq_rebase` moves to the LIBRARY.** The replacement drains the wrapped run
  front-to-back into a fresh run under the same `flat` invariant every construction loop
  carries. 3.L.8 walks it and prices it honestly, and Q18 puts the kernel row back to the
  owner if a driver's `E` cannot afford the second run.
- **`[S30]` the seven `[SYS-8]` range operations over views is ADOPTED.** The gap round 7
  found beside it, that a helper handed `&uniq MutSlice<u8>` cannot publish what it
  filled, is closed by `[S31]` below, and it is closed without a row.

**Three further decisions of 2026-09-04 empty 3.S's PROPOSED list. Nothing in this file
is proposed now.**

> **`[S31]` `seq_reslice` is NOT adopted as an operation (owner-decided 2026-09-04).**
> Forming a shared `Slice<'r, T>` from a `MutSlice<'r, T>` is the ordinary
> **shared child reborrow of a unique loan**, which `[OWN-6]` already admits for places: a
> probe on the v0.42 build accepts `peek(x: &deref(x))` inside a region block where
> `x: &uniq u64`. This design states it as that rule applied to **views** rather than as a
> kernel row. The child `Slice` carries the parent's origin set and range, its loan is a
> shared child of the exclusive one under `[OWN-6]`, and the parent may not be written
> while the child lives. `seq_reslice` is deleted from this file, `[VIEW-6]`'s
> "no helper library over views" restriction is restated, and the fill-and-publish helper
> is writable.
>
> **`[S32]` a linearity bound on a generic parameter is ADOPTED (owner-decided
> 2026-09-04).** `fn f<T: affine>`, `fn f<T: linear>` and `fn f['s: affine]` are written at
> the declaration and checked at the instantiation. `[PROV-6]` and `[BLK-4]` each gain one
> sentence reading the bound, and `gparam` and `region_params` each gain an optional
> bound.
>
> **`[S33]` `reserve_file -> own ReserveOutcome` is ADOPTED (owner-decided 2026-09-04)**
> in place of `[S25]`'s `Result`. `Reserved(value: FilePermit)`, `Exhausted()`
> and `Failed(error: IoError)` are three variants, so `[CALL-4]`'s existing route
> publishes `room(factory) = 0` on the refusal arm and `[RES-6]`'s gap closes.

Tree read: `batch/0116-containers-and-resources` at `main` 30602914,
`spec/kernel-spec.md` **v0.41 ACTIVE** at that tip, with **v0.42 merging**: v0.42 adds
`[FORM-8]` canonical region spelling and changes nothing else. Bare three- and four-digit
line numbers are v0.41 at 30602914 and every range below was re-derived mechanically from
that file in this session. Region spellings are v0.42's. **Type and const arguments are
always written** ([FN-2] 1124, probe `q4`).

**B1's four rules are implemented and the rest of this file is not.** [MSR-3], [MSR-5],
[CALL-4] and [CALL-6] landed as the v0.44 candidate (PR #17), each in a narrower form than
this draft wrote; 6.0 records what landed and §3.K carries **four corrections decided
2026-09-04** at their rules, being [MSR-5]'s production, [MSR-3]'s table row, [CALL-4]'s
deferred routes and [ENT-3.S13]'s population. Every other rule of section 3.K is draft
rule text for a work branch, not an amendment; section 3.L is design text for programs
that compile nowhere. Section 6 separates what a compiler executed from what is argued on
paper.

Settled by the owner, and not reopened anywhere below:

- The heap is an explicit capability **value** handed to `main`, so heap-freedom is a
  signature fact.
- `resource-closed` is a derived, writer-requirable property over an envelope `E` of
  tangible resources; a general heap, including a bounded one, is never part of `E`.
- No frame-accumulating recursion in v1; tail recursion is lowered.
- `FixedVector<T, n>` holds affine `T` through a checker-maintained typestate.
- The core is a contiguous run of initialized slots; keyed containers are fixed families
  over it, later.
- Owners versus affine views, transformed by value, with single-state `ensures` under
  [FN-9]. Two-state `ensures` is rejected.
- Append helpers take the owner by value and return it. Pass-by-pointer is only an ABI.
- Three call rules: through a shared borrow all facts survive; through a value passed and
  returned only contract facts survive; an element write through a view never touches its
  origin's own measures.
- Mutation of container state through `&uniq` is retired, and refused by a rule [BLK-4].
- Multi-return `-> (a: own T, b: own U)` with `let (a, b) = f(...)`.
- System I/O goes over views [S30].
- A shared view formed over a writable one is [OWN-6]'s child reborrow and not an
  operation [S31].
- A generic parameter may carry a linearity bound, written at the declaration and checked
  at the instantiation [S32].
- A covered store's refusal is a variant of the operation's own outcome and not a class of
  an error payload: `reserve_file` returns `ReserveOutcome` [S33].
- Every rule is a deterministic function of program text and compiler version.
- Linearity is derived from one criterion and the `linear` modifier exists for a logical
  obligation (D1); **the criterion is read against the scope** (D3).
- `set` has one commit rule, n-ary, with every target dead through the right-hand side
  and every target reinitialised at the commit (D2).

Four footnotes, because the minimality ruling and R1 move material the settled list
names.

1. **The owner inventory.** `FixedVector<T, n>` is unchanged. `HeapVector`, `ArenaVector`
   and `PoolVector` were three names for one shape at three stores; with [PROV-1] putting
   the store in the brand they are one kernel nominal `Vector<'s, T>` at two regions, and
   the three names survive in 3.L as what a writer calls an instance.
2. **`FixedRing`.** [BLK-1]'s typestate is a window rather than a prefix, so a ring *is* a
   run. 3.K.3 states what the window costs; the fifth cost is now a **library** drain
   (3.L.8) rather than a kernel row.
3. **`AppendView`.** Under R1 an appending helper takes the run by value and returns it,
   so the caller's length is the *result's* length and no device is needed.
4. **`update`, `swap` and `seq_exchange`.** Under D2 there is one assignment statement; a
   transformation is `set p = f(p: move p, ...)`, a swap of two whole places is
   `set (p, q) = move q, move p;`, and a swap of two elements of one run is refused by
   D2's non-overlap condition and written in three statements (3.L.2).

## Contents

1 [The problem](#1-the-problem) · 2 [The laws](#2-the-laws) and
[the ten notions](#21-the-ten-notions-and-their-closures) ·
3 [The rules](#3-the-rules): [3.K.0 the assumed amendments](#3k0-the-two-assumed-amendments-and-the-determination-principle),
[3.K kernel](#3k-kernel-rules), [3.S decisions](#3s-surface-decisions),
[3.L library](#3l-the-library-written-in-wf) ·
4 [Two worked programs](#4-two-worked-programs) · 5 [Open questions](#5-open-questions) ·
6 [Verified versus reasoned](#6-verified-versus-reasoned) ·
7 [Implementation order](#7-implementation-order) ·
[Appendix A](#appendix-a-generated-data)

---

## 1. The problem

### 1.1 Two goals, one language

**Goal A: the heap is off, and only logic errors remain.** A writer building an OS
kernel, a bootloader, a flight controller, or a device driver wants a program that cannot
corrupt memory, cannot race, cannot read an uninitialized byte, cannot silently overflow,
and also cannot die because a store ran out. Today the language delivers the first four
and not the fifth: [SCOPE-3] 27-31 leaves heap exhaustion, stack exhaustion, OS quotas
and runtime-start resources outside the source outcome model, so an accepted program may
stop at the host boundary with no Whitefoot value, no status, and no cleanup. A program
that can vanish at three in the morning has not removed the class of failure the writer
came here to remove. Neither has one that silently stops making progress because it lost
the last block of a store it owns — which round 5 found in this design's own flagship
program and round 6 found again in the **arena**. 1.1's promise is kept by a program
whose demand on every covered store is a bound, and [RES-10] is where this draft makes
that true for a bump extent as well as for a heap.

**Goal B: with a heap, be honest.** A hosted program wants the heap and should have it.
What it must not have is a hidden trap, and it has two today. Allocation is ambient: any
function may allocate while holding nothing, and refusal ends the process. And release is
invisible: probe `r2_5` compiles a function that takes `own box<u64>`, never returns it,
and declares `pure`. Goal B asks for both halves to be values: allocation is an operation
on a provider the caller holds, refusal is an ordinary typed outcome that hands back
every affine input it did not consume, and release is an action of a scope that **holds**
the provider and says so in its row (D3).

Both goals are one language. There is no subset mode, no second prelude, and no dialect:
the same rules judge every program, and one entry marker turns the failure to establish
the property into a compile error instead of a note.

### 1.2 The concrete failure: D1

The sweep of 2026-09-03 found an unsound accept that is exactly the defect this design
has to make unrepresentable, recorded as
`tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`, manifest
line 165, status `xfail`. **Re-run in this session against the v0.42 gate binary in its
fully elided spelling as probe `q8`: accepted, exit 0.**

```wf
fn shrink(handle: &uniq buffer<u8>) -> discarded: own buffer<u8> reads(handle), writes(handle), allocates(heap) {
  let smaller = buffer_new(2_u64, 0_u8);
  let old = replace deref(handle) = move smaller;
  return move old;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let line = buffer_new(10_u64, 0_u8);
  region {
    let dropped = shrink(handle: &uniq line);
  }
  let tail = line[9_u64];
  return exit_status(code: 0_u8);
}
```

`buffer_new(10_u64, ...)` establishes `len(line) = 10`. The callee replaces the whole
referent with a two-byte allocation. The caller keeps the stale length and uses it to
discharge offset 9 of what is now a two-byte object: an accepted out-of-bounds heap read,
and (with `set line[9_u64] = 7_u8;` instead) an accepted out-of-bounds heap write.

**Under this draft the program has no shape, and a rule says so.** [BLK-4]'s fourth
clause refuses a container nominal as the direct or indirect referent of a `&uniq`
parameter of a source-declared `fn`. A helper that transforms a run takes it by value and
returns it; a helper that only writes elements takes a view. Neither can change a
caller's length behind the caller's back, because in the first case the caller's length is
the *result's* length and in the second the view reaches element storage only.

**This is the fourth disposition D1 has had and the first that is a judgment.** The
fourth draft refused the parameter by its direct type ([CNT-7]), which a one-field wrapper
struct nullified; the fifth relied on a conservative kill, which round 5 defeated with a
fact published *after* the kill; the sixth withdrew the parameter by doctrine, which
refuses no declaration. [BLK-4]'s clause is stated over the reachability closure [PROV-4]
already computes, and now refuses a `&uniq` referent that reaches an unbounded type
parameter as well.

### 1.3 What the design therefore has to do

Turn every resource a program can exhaust into a value it must hold in order to consume
**and in order to release**, so that "this subtree cannot touch the heap" is a signature
fact and "this program's peak demand is this list of extents and slot counts" is a
compiler judgment. Give the writer one declaration that turns the second into a
compilation requirement. Make every failure to obtain a resource a typed value that
returns the affine inputs it did not consume. Put the runtime inside the same envelope as
the writer's code. Make every fact that survives a call readable from the callee's
declared parameter modes, declared types, and declared contract, so D1 has no expressible
form. Make every value's store readable from the value's own type, so D1's sibling has
none either.

And make each of those a property that is **closed**, that some rule **judges**, and whose
facts are **computed and used at points the rule names**. §2.1 answers the first (rounds
3-5 each found a notion introduced without its closure), 3.K.11's seventh condition the
second (round 6: a closure sentence with no judgment) and its eighth the third (round 7:
a named judgment with the wrong substitution or establishment point). All three checks are
mechanical and are run over every rule below.

### 1.4 The minimality ruling, and the partition test

The ruling asks one question of every candidate rule: *could a writer implement this in
wf, given the rest of the kernel?* If yes, it is not spec.

> The kernel specification is the **minimal** set: it admits only what cannot be
> implemented in wf itself. Anything a writer could implement in wf on top of the kernel
> does not enter the spec; it belongs to a standard library — and the owner leans toward
> not having one at all — or to user code. Container capabilities are abstracted to the
> lowest common primitive, and only the truly unimplementable part enters the spec.
> Non-normative content (bound tables, operation inventories) never goes in the spec body.
> Batches are an implementation order only, not spec versions; a single implementation is
> fine if correct. Human-factors conveniences are not spec content.

Applying that question needs a criterion for the container half, and the criterion is
storage. A writer can express **values**: construct them, move them, place them into
fields and elements, match on them, and let them go. A writer cannot express **storage
that holds no value**: a slot outside the initialized set is typed, addressable and
uninitialized, and wf has no spelling that reaches it, none that declares it, and no way
to make the boundary a checker-maintained fact rather than a killable data field.
`array<T, n>` is the shape a writer *can* have, and it requires `n` live values, which
for affine `T` is exactly what the writer does not have. So the run of initialized slots
is the lowest common primitive of every container this design ever proposed.

Everything above it is arithmetic over that primitive and is written in 3.L: a pool is a
run of runs, a growable vector is a run plus a growth policy, a keyed table is a full run
of `Option<T>` with element `replace`, middle removal is a take and an element `replace`,
filled and vacant construction are counted loops, and — new this draft — **returning a
wrapped window to its origin is a drain into a fresh run** (3.L.8). The store half
divides the same way: a **store** cannot be written, because it manages storage; a
**pool** — a thing that hands out *values* that happen to be runs, and takes them back —
is ordinary data and is written.

Rounds 5, 6 and 7 each applied the test in the removal direction and found the standard
too low: **an L18 removal must be priced against a walked program** (the sixth draft's
`seq_exchange` mistake) **and so must an L18 addition** (this draft's `seq_rebase`
correction). 3.L.2 walks the first and 3.L.8 the second. Two amendments this design
**assumes and does not draft** are stated in 3.K.0, and two consequences hold once and
for all: **the library is not part of the language** — no rule of 3.K names a library
function and a program that never reads 3.L is complete — **and it is not blessed.**

### 1.5 What this design does not decide: execution contexts

A scheduler that switches contexts, an interrupt handler, and a per-task kernel stack are
**out of scope for this batch, by the orchestrator's ruling**. No source construct in
v0.41 or in this design creates, enters, or switches an execution context;
`program_kind := "command"` is the whole production (181) and [FN-7] 1216 admits exactly
one entry, so an `interrupt fn` does not parse. Program 4.1 is written accordingly: a
cooperative run queue of state machines that advance on one chain, not a scheduler that
switches stacks. **A worker lane is an execution context**: [RES-1] counts its stack,
[STK-3] gives it an item, and [RUN-4] creates it. What is true of source is narrower:
*no source construct creates a context whose chain the program controls.*

```text
| this design fixes                | what a context switch must do with it                    |
|----------------------------------|----------------------------------------------------------|
| E carries one stack item per      | **owed**: a per-context stack is an extent acquired at   |
| execution context [STK-3]         | run time and [RES-5] has no algebra for one. The         |
|                                   | successor owes either a fifth algebra whose acquire is a |
|                                   | shaped extent, or the rule [RUN-4] already follows for   |
|                                   | the entry context and the lanes — every context's stack  |
|                                   | is established before SourceStart and is a PreStart item |
|                                   | rather than an acquisition. The second is the likely     |
|                                   | answer and neither is written here                       |
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
| release is derived in a scope     | **owed**: [LIV-1] is a per-join check over one function's|
| that holds the capability [PROV-6]| [FN-1] graph. A context that dies is not an edge of any  |
|                                   | such graph, and the successor owes a rule that a context |
|                                   | may not be abandoned holding a linear value              |
| a loan is held by a value for its  | **owed**: [PROV-3]'s exclusivity argument is over one    |
| whole life [L10, VIEW-2]          | thread of control, so a suspension point needs the same  |
|                                   | no-live-loan premise [STK-1] gives a tail edge           |
```

## 2. The laws

Seventeen live laws. Every rule in 3.K is an instance of one of them, and **a rule that
cannot name its law is not admitted.** L1-L9 are the resource laws, L10-L15 the container
laws, L16 and L17 the two the first falsifier round added, L18 the minimality ruling
stated as law. **L14 is retired**, its id never reused (footnote 3). Each law states its
ground in one or two sentences; ruling ids cite `EVIDENCE-owner-discussion-2026-08-31.md`.

**L1. The envelope is the program's promise, and the promise is made in two stages.**
*A resource-closed program declares one finite, shaped envelope `E` and promises that on
every legal execution, and on every finite prefix of an infinite one, its demand for
covered resources never exceeds `E`. The judgment that a program makes that promise is a
source judgment, a deterministic function of program text and compiler version alone, and
every quantity it tests is a compile-time integer or a closed expression in compile-time
constants, type-level constants and runtime-profile symbols. Computing `E`'s concrete
figures, and checking that a selected target and runtime can carry them, is a
target-stage qualification obligation whose failure cites no language rule.*
Because acceptance may not depend on a register allocator or a linked runtime: R13
(`L7036`), B8, [SCOPE-3] 27-31, [STOR-6] 738-767. Round 7 found the sixth draft's stage
error **relocated rather than removed**, so the second sentence now says *what* the
judgment may test, and [RES-5], [RES-9] and [RES-10] are written against it.

**L2. No resource is ambient.** *Every covered resource enters the program as a
capability value the runtime hands to `main`, or as a store the program reserves in a
region block it owns, and travels only by ordinary ownership; there is no ambient
allocator, thread source, or stack pool.*
Because only a held value makes heap-freedom a signature fact: probe `p5_ambient`
allocates while holding nothing and is **accepted today**. **D3 strengthens this law**: a
scope gets a derived release exactly when it holds the provider, so the release is as
signature-visible as the allocation.

**L3. Nothing fails silently, and nothing grows behind the writer.** *Every operation
that can fail to obtain a covered resource returns a typed value naming the failure and
handing back every affine input it did not consume; no operation traps, aborts, retries,
falls back, or promotes a store to a larger one, and no compiler-derived action does
either.*
Because v0.41 claims zero writer-reachable runtime-trap families (spec line 6) while heap
exhaustion still ends a process with no source value: R12 (`L5657-5666`), B3, Q8. Round 7
showed the last clause's repair **reaching nothing at all** — it was stated over a
sub-graph "reached through leaves" and nothing is reached through a terminal — so
[PROV-6] states it over the **release graph** the walk actually traverses.

**L4. No hidden growth.** *No operation both uses existing capacity and acquires new
capacity; every operation that may acquire capacity takes a provider, names its
allocation effect, and returns a typed failure, while every operation that only uses
existing capacity is total under a proved capacity requirement and can allocate on no
path.*
Because one `push` cannot carry one return type and one effect row across backings: R5
(`L2332`), B2, B3, X1.

**L5. The runtime is inside the envelope.** *The artifact `E` describes is the writer's
code, the compiler-derived cleanup and drop glue, the `par` runtime, the allocator and
the target adapter together, from the frame the environment hands the program to the
frame it takes back; a resource any of them needs is an item of `E`, or the program is
not resource-closed on that target.*
Because a guarantee that stops at the edge of generated code is not one: R12, B12. Round
7 read the shipping floor and found two alternate stacks where the seventh draft carried
one, the host thread's stack named nowhere, and both materializations failing silently;
[RES-1], [RES-2], [STK-3] and [RUN-4] are corrected.

**L6. Shape, not bytes.** *`E` is a list of tangible resources (contiguous aligned
extents, per-class slot counts, per-context stacks, lane counts, host handles) and never
one byte total. Every item carries the shape its members have: an extent carries bytes
and an alignment, a countable object carries a count **and its member's size and
alignment**, and a store the program itself reserves is shaped by the same rule.*
Because sixteen bytes holding four four-byte objects, the first and third released,
cannot serve an eight-byte request: R12, B9, B11. The member-shape clause is round 7's —
`slots(kind, count)` gave a deployment a count with nothing to multiply it by.

**L7. Lowering before judgment, and a tail call is a dead caller frame.** *Tail
recursion, including mutual tail recursion, is rewritten into one dispatcher function
before any resource judgment runs; an intra-component call edge is a tail edge exactly
when the caller's activation record is dead at the jump, and never because the call is
written in a return statement; and the dispatcher the rewrite produces is an ordinary
concrete function, checked by every rule any other function is checked by.*
Because an optimization that may or may not fire cannot be a premise of a guarantee and a
syntactic condition cannot see a live loan into a caller's frame: R3 (`L989`), R12, B10,
probes `f2b`, `f8_tailframe`. The last clause is round 7's: the premise was read on one
program and the frame measured on another.

**L8. Demand is computed as if every acquisition succeeds; a store's own refusal is an
ordinary fact.** *The judgment replays each execution assuming every covered acquire
succeeds, and may never conclude that demand is small because a failed acquisition would
have ended the program. It does read the store's own post-state relation on a refusal
edge, because `room(store) = Z` is a fact about the store — where the store's refusal is
a **variant of the operation's own outcome**, and not a member of a portable class set no
route can read.*
The first half removes the circularity; the second is what makes a checked spelling worth
writing: B8, R12. Round 6 showed the second half read too far, and round 7 showed the
handle table's refusal conditioned on an `IoError` **class** no route publishes; [S33] is
the owner's repair, and [RES-6] states the relation the `Exhausted` variant publishes.

**L9. Stock, not flow, and a total operation at a capacity boundary must say what it
dropped.** *Resource-closedness bounds what is held at once and what is consumed
irreversibly; it never bounds how many times a program acts. An operation may be total at
a capacity boundary only when the value it displaces is copy and its displacement is a
published relation the caller can read; a silent drop of an affine value is a refusal
wearing a disguise and is inadmissible under L3.*
The first half is why a service loop runs forever with one live slot; the second is why
"overwriting is this ring's semantics" cannot be written on every bounded store: B8, R12.

**L10. A view is a value, it holds its own loan, and it owns nothing.** *A view is a
value with a static type, not a reference the callee writes through; it holds, for its
whole life, a loan of its own strength on the range it reaches of every place in its
resolved origin set, beginning at formation and ending where that view value's own
liveness ends; a function that changes a view's state consumes it and returns the new
one. A loan covers every binding the address computation of its place reads, for the
loan's whole life. **A loan-bearing value owns nothing**: what it reaches belongs to its
origin, so no obligation and no release action of what it reaches is ever a property of
the view.*
The first clause answers write-back without a hidden protocol, the range is what makes a
`par` fill over one owner expressible: owner's settled decision of 2026-09-03, B6, probes
`f1c`, `f1d`, `f2b`, `r1_twouniq`. The **end** of the loan is round 7's: the seventh
draft ended it at a consume, which a **copy** view never has, so one shared view of a
heap-backed run made that run unreleasable for the rest of its scope.

**L11. Length is a type fact or a contract fact, never a guess.** *At every program point
the checker's knowledge of a sequence's measures comes from exactly one of: the type, an
established fact with live support, a compiler-owned measure datum, or a verified
contract relation; no rule infers a measure from the shape of an argument, the name of a
callee, the absence of a write, or what a body was seen to do. A relation about a value a
callee received names that value; no relation describes a state of a caller's object at a
point the callee cannot name. **A container nominal, a loan-bearing type and an unbounded
generic type parameter are therefore not reachable through a `&uniq` parameter of a
source-declared function at all**, because such a parameter is the one position from
which a callee can leave a caller holding a measure of a value the callee replaced.*
This is D1 stated as a law. The third sentence is R1 stated as a *rule*'s premise; the
generic clause is round 7's, which compiled `&uniq Holder<T>` at `T = buffer<u8>` today.
`EVIDENCE-sweep-D1.md`, probes `q8`, `w3`, `x11`, all accepted today.

**L12. The initialized region is a window, and the language says so.** *A run of slots is
exactly the `len` slots beginning at `head` modulo `cap`, initialized, with the rest raw;
the boundary is checker-maintained typestate carried by the run's own value, and no
per-slot tag, occupancy bitmap, or runtime discriminant is language state. The kernel
admits exactly append and removal at each end; every other order is arithmetic a writer
performs over those four. A logical offset is the coordinate every rule of this design
speaks in, and the map from it to storage is stated once.*
With no per-slot state the checker never needs a quantified proposition over slots. Under
a *prefix*, "every other order is arithmetic" was **false for a queue**, at a measured
seven times a hand-written byte ring; round 7 removed the fifth boundary row again,
because returning a wrapped window to its origin **is** such arithmetic (3.L.8). Owner's
settled decision; Q2, Q4, Q10.

**L13. A value's store is a component of its type; acquisition, release and activation
are all closed over it; and linearity is read against the scope.** *Every store the
program can exhaust is named by one region, minted where the store is reserved or where
the runtime hands it in, and every value that store backs carries that region in its own
type. A region names **at most one live store at any program point**, and a placement
whose storage cannot be per activation is refused wherever more than one activation of
its region block can reach it. A value whose backing is reclaimed per value has a release
action that **requires a capability**, and such a value is **linear exactly in a scope
that does not hold that capability**: `affine` carries ownership and lifetime, `linear`
carries an unmet reclamation obligation, and the criterion is stated once — **a value
whose release action requires a capability the scope does not hold is linear there; a
value whose release requires nothing, or whose capability the scope holds, is affine
there.** A value whose declaration carries the `linear` modifier is linear in every
scope, for a logical obligation rather than a storage one. Linearity is a property of an
affine type at a scope and not a third class, **it is closed under ownership** — a value
is linear when it *owns*, at any depth, a linear value, and a loan-bearing type owns
nothing — and a linear value leaves a scope only by being moved out whole or by being
destructured whole. An affine value leaves additionally by a disposal or by the one
compiler-derived release. No source construct selects, replaces, or observes a release
action, and a store's storage reclamation never stands in for its content's own release.*
Sentence one is round 3's rank-one repair and has survived every position attacked since.
The capability criterion is R2. **Reading it against the scope is D3**, which makes the
derived release available exactly where the capability is a held value: a scope holding
`heap: &uniq Heap` cannot smuggle it, because it declared it. Closure under ownership
rather than containment is round 6's, and it is the difference between a rule that frees
a caller's runs through a shared view and one that does not.

**L14 is retired.** It stated that an `AppendView` reaches only what it appended and
never decreases its owner's length; the type is gone (footnote 3). Under R1 the guarantee
it bought is an ordinary clause — `ensures len(rest) >= len(out)` — so nothing replaces
it and nothing is lost.

**L15. The descriptor's measures are values; the allocator's extent is not; and a measure
a caller needs is published by whoever wrote it.** *`len(v)`, `cap(v)`, `room(v)` and
`head(v)` are a run's own logical measures and are readable as ordinary `u64` values. No
operation observes the physical extent the allocator provided. Every operation that
writes a measured place publishes, for each measure of that place, its exact new value
where that measure is exact and a two-sided bound where it is not, including the measures
it did not change, on every exit including a refusal. **That obligation is on every
operation, and a function that hands a measured value back is an operation.** A row never
leaves a measure to be reconstructed from the standing identity, and a clause both of
whose sides follow from the standing identities alone does not discharge it.*
The first draft forbade reading `cap` and `room` on a rationale that only forbids reading
the allocator's size: B3, Q9, probes `q24`, `v25`, `v26`. **The last clause is round
7's**, which satisfied [CALL-7] with `ensures head(result) <= cap(result);` — a standing
fact — and left every view formation undischarged again.

**L16. One measure algebra, one goal disposition, one denotation per position, and one
establishment point per fact.** *`len`, `cap`, `room` and `head` are one-place terms of
the term language, defined once with their support, their kills and their standing
identities, over every measured place. Every consumer of a numeric goal asks one
question, whose complete admitted derivation is stated once; no rule grants a proof route
to a construct by name. **One spelling has one denotation at each position at which it
can occur, keyed on the parameter's mode**, stated once in a table rather than
distributed over three rules. **And every published fact names the point at which it is
computed and the point at which it is established**, which are the same point unless a
rule says otherwise.*
A language in which "can this inequality be derived?" depends on which construct is
asking has several provers and a writer can reason about none of them; probes `v25` and
`v26` are the same proof asked twice with opposite verdicts. [ENT-1] 2648. **Keying the
denotation on mode is round 7's**, which found the seventh draft keying a
declaration-domain operand on `writes` coverage so that `seq_place`'s own relation read
`len(P) = len(P) + 1`; the establishment sentence is round 7's second, where a relation
instantiated at a call and established at a later arm outran the kill meant to bound it.

**L17. Affine liveness agrees at every join, and a linear value never reaches a scope
exit alive.** *A binding's live-or-dead status must be the same on every predecessor of
every join and at every loop head; a disagreement is a hard error at the join.
Consequently **whether** a compiler-derived release runs on a scope-exit edge is not
runtime state and the edge's disposition is unconditional; **which** release runs inside
a value may be, exactly as an enum's derived drop selects on its discriminant today. A
binding that is **linear in its scope** [L13] and is live on any edge leaving that scope,
a `propagate` error edge included, is the error, because in that scope no derived release
exists to carry it.*
The reinitializing `set` makes liveness non-monotone, and [OWN-11] and today's
`Semantics/Unsupported: OwnershipJoin` avoid the question rather than answering it; the
same per-edge check makes the linear obligation checkable. Probe `f3`; [ENT-5]'s own
all-predecessor join.

**L18. The kernel admits only what wf cannot express, and both a removal and an addition
are priced against a walked program.** *A rule enters the kernel exactly when no program
a writer can write in wf over the remaining kernel has its effect. A capability a writer
can build is not a rule, a convenience is not a rule, and a table of data is not a rule:
the rule is the sentence that says such a table exists and what it must contain, and the
table is generated data beside it. **A row removed under this law carries, beside it, the
replacement program walked to the standard 3.L.0 states — its obligations, the rule that
discharges each, and the probe where one exists — and the cost the replacement carries. A
row added under this law carries the same walk of the program that needs it and the
per-call cost that program pays.***
The owner's ruling of 2026-09-03, stated as law so every rule below can be checked
against it. **The addition clause is round 7's**, which found `seq_rebase` admitted on a
stated ground that is false — its own rejected alternative is the writable program.

### 2.1 The ten notions and their closures

Rounds 3, 4 and 5 produced one finding each and it was the same finding: a notion
introduced, used by several rules, closed by none. This subsection names every notion the
design has and states its closure in one sentence. Round 7 produced none of that shape
and instead broke four rules whose closure sentence is right and whose *substitution* or
*establishment point* is wrong, which is 3.K.11's eighth condition rather than an
eleventh notion. **A rule that mentions a notion without respecting its sentence, that
states a fact its own `Judgment:` line does not produce, that publishes a fact with no
source and no destination, or that computes a fact at one point and uses it at another
without naming both, is a defect of this file.**

```text
| notion       | closure property, in one sentence                                                        |
|--------------|------------------------------------------------------------------------------------------|
| identity     | a value's store is a component of its type, so every value-forming and value-transporting |
|              | step preserves it, and no rule admits a store region by outlives rather than by exact     |
|              | identity                                                                                  |
| activation   | a region names at most one live store at any program point, and every placement whose     |
|              | storage cannot be per activation is refused wherever more than one activation of its      |
|              | region block can be live at once                                                          |
| release      | every value has exactly one disposition on every edge leaving its scope — moved,          |
|              | destructured, disposed, or one compiler-derived release; the last two are available       |
|              | exactly in a scope that holds the released capability, which it holds by signature; a     |
|              | store's storage reclamation never stands in for its content's release; the capability a   |
|              | release spends is determined by the brand rather than written; and the walk that performs |
|              | it terminates because the release graph is acyclic. **OPEN at one shape**: a run whose    |
|              | element type is linear by declaration is linear, has no capability leaf and is not a      |
|              | nominal, so neither a destructuring nor a disposal reaches it (Q13)                       |
| accounting   | every covered store is one domain of the map, every edge of the graph carries an entry of |
|              | that map including the retention entry of an edge that never runs, every entry composes   |
|              | by the same arithmetic at every position, every quantity the composition tests is a       |
|              | compile-time integer or a closed expression, and one stated extraction turns the map into |
|              | `E`'s figure                                                                              |
| linearity    | a value is linear **in a scope** exactly when it **owns**, at any depth, a value whose    |
|              | release action requires a capability that scope does not hold or whose declaration        |
|              | carries the modifier; a loan-bearing value owns nothing; and the predicate is discharged  |
|              | there only by a move or a destructuring of the whole value                                |
| loan-bearing | a loan-bearing value holds, for its whole life, a loan of its own strength on the range   |
|              | it reaches of every place in its resolved origin set, ending where that value's own       |
|              | liveness ends, may occupy no position from which it could outlive or hide that set, and   |
|              | may be the referent of no `&uniq` parameter of a source-declared function                 |
| measure data | every measure a program can name is a term with descriptor-storage support, published     |
|              | exactly and completely by every operation that writes its place, killed exactly by an     |
|              | event that writes that storage, and given a datum at **every** event by which a measured  |
|              | value acquires a name and the language undertakes to carry its measures                   |
| publication  | every fact a rule publishes names the [ENT-3] source that establishes it, the             |
|              | substitution that instantiates it, the point at which it is instantiated and the point at |
|              | which it is established, the destination it lands on, and the support that keeps it alive |
| set commit   | one statement writes places: its right-hand side is evaluated with every target dead from |
|              | its own read-out, every target is reinitialised at one commit, the targets are pairwise   |
|              | non-overlapping, and a target that names a binding in scope keeps that binding's term     |
| elision      | whether a **region** is written at a position is decided by the declaration text alone;   |
|              | type and const arguments are always written for a user generic and are written for a      |
|              | compiler-owned row exactly where no operand supplies them                                 |
```

Where each is carried, and what round 7 moved. **identity** — [PROV-1]; attacked from
every position in seven rounds and not moved, except at the one place identity is not
carried by a type, an `arena_extent` occurrence's envelope item name, which [PROV-5]
states. **activation** — [PROV-1]'s invariant and [PROV-5]'s refusal, plus [PROV-5]'s new
sentence that a reserving occurrence must be the loop-free statement of its own region
block. **release** — [PROV-6], [LIV-1], [STOR-3]'s table; round 7 broke the walk's
termination twice (the cyclic refusal reached nothing; the walk and the refusal
quantified two graphs) and [PROV-6]'s **release graph** is now one object both quantify
over, with D3 adding the scope. **accounting** — [RES-5], [RES-8], [RES-10], 3.K.7.1;
round 7 found five holes and each is repaired at its rule (6.11). **linearity** —
[PROV-6]; D3 is the scope, and round 7's writer lens is what made the cost visible.
**loan-bearing** — [PROV-3], [BLK-4], [VIEW-4]; the copy view's loan had no end condition
and [VIEW-4]'s ground was false at a copy target. **measure data** — [MSR-1]-[MSR-3],
[BLK-0], [CALL-7]; the closure sentence is narrowed to the events the language undertakes
to carry rather than widened to cover a `replace`, whose commit establishes no fact
[SET-2] 528. **publication** — [CALL-6], whose own establishment point was round 7's
second BREAK. **set commit** — [LIV-2], whose commit paragraph and first admission
condition contradicted each other; the read-out sentence is the repair. **elision** —
3.K.0 and [PROV-1]: the seventh draft's criterion covered region, type and const while
the landed [FORM-8] covers regions only, and its two elided-brand candidate sets never
intersected.

---

#### 3.K.0 The two assumed amendments, and the determination principle

This design rests on two amendments it does not draft. **The first has landed.** A build
in this session rejects a written region name at a position no other position of its
declaration names, citing `[FORM-8] RegionSpelling`, and accepts the fully elided
spelling of the same program (probes `q2`, `q3`). That is v0.42's `[FORM-8]`, uniform
over every **region** position, and it is not a rule of this design, not in 3.K's count,
and not registered by 3.K.11.

> **The scope is regions, and only regions.** v0.42's `[FORM-8]` is titled *Canonical
> region spelling* and every clause of it is about a REGIONID; v0.42's `[FN-2]` 1124
> keeps *"type and const instantiation arguments are always explicit"* and `[TYPE-5]`
> keeps its three callee classes. The seventh draft stated the criterion over "region,
> type or const" and wrote ten call sites to it; probe `q4` is
> `[FN-2] TypeMismatch { expected: "1 written type argument" }` on the amended build and
> probe `q5` is the same call with the argument written, accepted. **Every call site in
> this file writes every type and const argument a user generic declares.**

**The second has not landed, and this design writes its programs in it** (D4).

> **Every loop body is implicitly a region block.** Inside a `loop_stmt` or `for_stmt`
> body a `borrow_expr` of an outer binding is written **bare** and denotes that body's
> implicit per-iteration region. Writing an explicit `region { ... }` as the loop body's
> only enclosing block is a `[FORM]` rejection, because one semantics has one spelling.
> `[OWN-11]`'s judgment is unchanged — a borrow inside a loop body still denotes only a
> region introduced inside that body — because the implicit block *is* such a region and
> it ends every iteration.

This amends `[OWN-11]`, `[FORM-8]` and `[GRAM-4]`, it is small and mechanical, and it
lands as its own batch (§7, B0b). **It has not landed and this file does not claim it
has**: probe `q2` shows the amended build rejecting a bare loop-body borrow at
`[FORM-8] RegionSpelling` with the fix *"write the region this borrow takes, or place the
borrow inside the `region` block whose region it takes"*, and probe `q3` shows the
explicit-block form compiling. Both worked programs and 3.L are written in the decided
spelling.

**The rejected spelling is the one that adds nothing, which is the unnamed one.** The
implicit per-iteration block has no binder, so a loop body that must **name** its region —
because a reserving occurrence writes that name into `arena_frame::<bytes, align, 'a>()`
[PROV-5], and 3.K.7's per-iteration scratch idiom is exactly that shape — writes
`region 'a { ... }` whose name a reserving type argument inside it must write, and is
outside the rejection for that reason: the implicit region has no name to put there, so
that block is the only spelling of its region. Every other `region_stmt` that is a loop
body's only statement, named or not, has exactly the implicit block's extent and is the
second spelling the rejection removes; a block beside other statements is narrower than
the body, carries [OWN-6]'s statement scope, and stays legal. This is the criterion the
v0.43 candidate (`batch/0120`) implements, decided by reading the loop body alone.

**Why this design needs the first amendment at all.** [FORM-1] 35 admits exactly one
spelling per semantic construct. Putting a store's identity in the type means a region in
every type that names a store, unless the text determines it — in which case *writing* it
is a second spelling. So the brand cannot be in the type without that amendment, and the
amendment cannot be brand-specific, because a brand is one more region argument.

**The criterion is derivation, not repetition, and it is about regions.**

> A **region** argument is **written** at a position exactly when the declaration's own
> operands do not determine it, and **elided** exactly when they do. Written and elided
> are decided per argument, not per list. A **type or const** argument of a user generic
> is always written [FN-2]; a type or const argument of a compiler-owned row is written
> exactly where no operand of that row supplies it, which is [TYPE-5] 370-394's own
> retained-argument sentence applied to a fourth callee class.
>
> **Two region positions are outside the criterion and are always written.** A
> `construct`'s arguments, because a `construct` consults no expected type
> [TYPE-5] 383-386; and a **declaration's own parameter binders**, because a binder is
> where a name comes into existence. A `region_stmt`'s binder is written exactly when
> some position of its block names it.

Applied to every spelling in this file, the criterion and the text agree:

```text
| occurrence                                                   | determined by an operand?      | spelling             |
|--------------------------------------------------------------|--------------------------------|----------------------|
| arena_frame<const bytes, const align>['s]()                  | no operands exist              | all three written    |
| seq_fixed<T, const n>()                                      | no operands exist              | both written         |
| seq_heap<T>['s](heap: &uniq Heap<'s>, count)                 | 's from heap; T from nothing   | seq_heap::<u8>(...)  |
| seq_arena<T, const bytes, const align>['s](arena, count)     | 's, bytes, align from arena    | seq_arena::<u8>(...) |
| seq_place(vector: own V, value: own T)                       | V and T from the operands      | seq_place(...)       |
| try_place<T, const n>(vector, value)  — a user generic       | always written [FN-2]          | try_place::<Task,32> |
| a user fn's own region parameter list                        | supplied by the actuals        | elided at the call   |
| render['s](block: move held, task: &task)                    | 's from the block operand      | render(...)          |
| Some<Task>(value: move ready)                                | a construct: outside the rule  | T written            |
| region 'a { ... arena_frame::<4096, 16, 'a>() ... }          | the block's own binder, named  | 'a written           |
| region { let body = seq_slice(vector: &kept.v); ... }        | the block's binder, unnamed    | no name written      |
| a borrow of an outer binding inside a loop body (D4)         | the implicit block's region    | bare                 |
| struct BlockPool['s] { free: FixedVector<Lease<'s>, 8>; }    | a declaration mints its own    | 's written at both   |
```

**This is a principle about determination, not a rule about regions, and one other
position obeys it.** A release spends the capability of every store its operand's type
names at a capability-released leaf, and **that capability is determined by the brand**: a
store region names at most one live store [PROV-1], a store has exactly one provider
[PROV-2], and at any program point at most one live binding can lend `&uniq` to that
provider [OWN-5] 606. There is nothing for the writer to choose, so under [FORM-1] there
is nothing to write, and the statement is `dispose p;`. **Allocation is the opposite case
and keeps its written provider**: there the writer *chooses* which store backs the new
value. Determination decides both, in opposite directions, from one sentence.

**Where the elided brand resolves is [PROV-1]'s rule, not this section's.** The seventh
draft stated two candidate sets here that never intersect, so
`bs_reserve(s: own Bytes, heap: &uniq Heap, ...)` had `s.v` at one region and `heap` at
another, did not typecheck, and could not resolve a release. [PROV-1] states one
resolution, and 3.L.5 and 4.2 are written on it.

**Measured on this worktree**, `tests/programs` is 28 files and 131 top-level function
declarations, of which **67 carry a region parameter list**, and across all 67 no region
name is used at two positions outside its own parameter list. The corpus also writes 484
named borrow annotations, 251 call-site region arguments and 232 region-block names;
essentially all become [FORM-1] rejections under the landed amendment and disappear, no
program's meaning changes, and one mechanical migration pass converts them — by a tool
that ships nowhere, because [FORM-1] 36 says the toolchain never auto-formats. Round 7
measured the trade on the program most exposed to it: about 197 written region identifiers
leave `wfgrep.wf`, and fourteen arrive in the one program with two genuine stores.

## 3. The rules

Section 3 is two sections, read differently. **3.K is the kernel**: nine families,
**fifty-one rules**, five added nominals, twelve declaration-domain operations plus four
readers, one added statement form and one added `let` alternative, one added declaration
modifier, and 3.K.0's two separate amendments. Every rule answers L18's question with *no
writer can write this in wf* — except one half of [CALL-4], whose status 3.S records
honestly — and 3.L.6 lists the nine that only the partition test proved. **3.L is the
library**, written in wf against 3.K; it is not part of the language, it is not blessed,
and no rule of 3.K names any of it.

The count is unchanged at fifty-one and the inventory shrank by one: `seq_rebase` is
**withdrawn from [BLK-3] to the library** (3.L.8, [S29]), so the declaration domain has
twelve operations rather than thirteen.

**Every kernel rule states four things — the judgment it creates, the fact it publishes,
what it amends, and its law — plus a `Depends:` line exactly when it rests on a v0.41
sentence no `Amends:` line in this file changes.** A rule that creates no judgment writes
`*Judgment:* none`. **Every fact a rule states appears in its `Judgment:` or `Publishes:`
line, every rule that reads a fact another rule states names the judgment it comes from,
and every rule that computes a fact at one program point and uses it at another names
both points** (3.K.11 conditions 7 and 8). `*History:*` names the falsifier round and
finding that produced the rule's current shape and nothing else; rounds 1-6 resolve in
6.5 and round 7 in 6.11. 3.K.11 is a **collation of the `Amends:` and `Depends:` lines
and carries nothing else**.

### 3.K Kernel rules

#### 3.K.1 `[MSR]`: measures, terms, and the one goal disposition

This family is first because everything else consumes it. It adds no statement form and
no type; it is a specification amendment.

**[MSR-1] Four measure terms, over one place, for every measured value.** `len(P)`,
`cap(P)`, `room(P)` and `head(P)` **[S11]** are terms of the [ENT-2] term language, of
fragment type `u64`, where `P` is an admitted place. Which measures a type has, and
whether each is **exact** or **bounded**, is table data (A.1); the rule is that the table
exists, gives every measured type a row, and gives every cell one of *exact*, *bounded*
or *absent*. An **exact** measure is one every writing operation publishes a value for; a
**bounded** one is one some writing operation can publish only a two-sided range for.
**Exactly one measure is bounded anywhere**: a run's `head` after a front operation,
whose new value is a modular expression the affine domain does not carry. An `Arena`'s
`len` is exact, because [RES-5]'s alignment requirement makes the padding at a take zero.

**A measure is a logical quantity and `head` is the origin of the logical coordinate
system, stated once here because four rules read it.** A run's initialized set is the
`len` slots beginning at `head` taken modulo `cap` [BLK-1], and a **logical offset** `i`
names the slot at physical offset `(head + i) mod cap`. Every measure term, [OP-4]
obligation, [PROV-3] range, [PAR-2] disjointness argument and [RUN-3] footprint is stated
in logical coordinates, and one sentence carries a logical conclusion to a storage
conclusion:

> `i |-> (head + i) mod cap` is injective on `[Z, len)` because `len <= cap`, so two
> disjoint logical ranges of one run describe disjoint storage.

An admitted place is a `place` [GRAM-5] formed with field selections, `deref` wrappings
**and subscripts**, whose final selected type is measured. The subscript admission is the
change — `len(table[i])` is a term, so a run of runs has provable operations — and it is
why [MSR-2]'s granularity and [CALL-3]'s classification are stated over **storage**
rather than over the word *element*.

*Judgment:* the [OP-4] admission above at every subscripted measure place; the
injectivity sentence is a definition proved by `len <= cap`, which [MSR-2] publishes as a
standing fact. *Publishes:* the four terms, the logical coordinate system, the
injectivity sentence [PROV-3] use 4 and [RUN-3] read, and the exact/bounded
classification A.1 tabulates. *Amends:* [ENT-2] 2677-2728 clause (b), which today admits
`len(P)` only for `array`, `slice` and `buffer` and only for subscript-free places;
[OP-4] 914-924, whose obligation gains the erased-clause attach-site case. *Law:* L12,
L15, L16. *History:* r7 F3-10; r6 F2-14, F1-4.

**[MSR-2] Support is descriptor storage, a kill is an ordinary [ENT-5] event, and a
standing fact has empty support.** A measured value's storage is two disjoint parts: its
**descriptor storage**, the measure words its value carries, and its **element storage**.
The support of a measure term over `P` is `P`'s descriptor storage, every borrow or
content holder any prefix of `P` reads through, and the support of **every** offset
occurring anywhere in `P`.

The kill is [ENT-5]'s own rule with no new overlap notion: a measure term dies exactly on
an [ENT-5] event whose written place overlaps its support, where an event is any [LIV-2]
commit, [SET-2] commit, consume, scope exit, or **any action carrying a `writes`
occurrence that projects onto that storage under [EFF-2]** — a call, a `dispose`
[PROV-6] and a compiler-derived release alike. Stating the kill over the effect row keeps
it closed when a later family derives a new action.

**The granularity is stated once, over storage, and nothing is derived from the word
*element*.**

> A write at an element position of `P` overlaps the **descriptor storage of `P[i]`** and
> none of `P`'s own descriptor storage. It therefore kills every measure of `P[i]` and no
> measure of `P`, whether the write is a [LIV-2] commit, a [SET-2] `replace`, or an
> element write of a scalar — for which the set of killed measures is empty because a
> scalar has none.

Two consequences follow as derivations rather than clauses: a write to a **sibling
field** does not kill, because `deref(r).flags`'s descriptor storage and `deref(r).tail`
do not overlap (probes `r2_4`, `r2_4b`, `r2_4c` show today's compiler root-granular where
[EFF-2] is field-precise); and a write to an **offset** kills at every level. [ENT-5]
2893 clause (a)'s element-position carve-out is **removed rather than narrowed**, because
it is true in v0.41 only for a reason [MSR-1] deletes.

At every point at which `P` is live these hold implicitly:

```text
Z <= len(P)     Z <= room(P)     Z <= head(P)     len(P) <= cap(P)     head(P) <= cap(P)
```

and `len(P) + room(P) = cap(P)` is appended, as two inequalities, to [ENT-6] 3007's
automatic affine-premise sequence with the empty support every standing fact has. **The
identity is a convenience for the writer and never a route by which an operation's own
post-state is derived**, and a contract clause both of whose sides follow from these
standing facts alone discharges no [CALL-7] obligation. **A measure whose value is a
compile-time constant or a runtime-profile symbol is a standing fact with empty support.**

*Judgment:* the kill classification at every [ENT-5] event, which is the judgment
[CALL-1], [CALL-3], [CALL-7] and [MSR-3] read. *Publishes:* the implicit facts above, the
two automatic premises, and the standing-fact class. *Amends:* [ENT-2] 2677-2728's
implicit-fact sentence; [ENT-5] 2863-2967's support and kill sentences, whose length-term
support becomes the descriptor-storage relation above, whose kill classes gain the
effect-row statement, and whose clause (a) loses its carve-out; [ENT-6] 2969-3100 at
3007. *Depends:* [ENT-4] 2860, whose difference-bound uniqueness argument is why the
identity is a premise and not an L0 fact; [ENT-5] 2942-2946, which keeps an empty-support
fact from crossing a backedge. *Law:* L15, L16. *History:* r7 F1-3; r6 F1-2; r4 F1-a3.

**[MSR-3] Measure datums, one denotation per position keyed on mode, and what an atom is
keyed by.** A **measure datum** is a compiler-owned immutable [ENT-2] term of fragment
type `u64` with **empty support**: no [ENT-5] event kills it, no place occurs in it, no
later write retargets it. There is one former, keyed on what a datum denotes:

```text
a datum is identified by (program point, admitted place P, measure), is
compiler-owned and immutable, and is established equal to <measure>(P) at that point
```

**Six placements exist, and no seventh. The closure sentence is that a measured value
acquires a name at exactly six kinds of event at which the language undertakes to carry
its measures, and every one is a point at which the function forming the datum can read
the value.**

```text
entry     body entry, per parameter of measured type and per measure
call      one call's pre-transfer point [ENT-5], per operand place of measured type,
            reading a borrow operand through its resolved referent and an own operand
            as its value before transfer
construct one `construct` [GRAM-8] or enum-payload construction, per measured operand
rebind    one `let` or one [LIV-2] `set` whose right-hand side at an ordinal is
            `move P` for a measured place `P`, read before transfer
payload   an own-place `match`'s arm binders [OWN-13] 653-662 of measured type
field     a destructuring consume's field binders [S13] of measured type
```

**Two naming events are outside the list and both fail safe**, which is why the closure
sentence says *the language undertakes to carry*: a `replace`'s displaced binding — a
`replace` publishes nothing [SET-2] 528, so such a value carries no measures and a
function returning one is refused by [CALL-7] rather than silently unusable (3.L.2's
`take_at` is the worked cost) — and a borrow-mode `match` arm binder, which names a
borrow and not an own value.

**One denotation per position, keyed on the parameter's MODE (L16).**

```text
| the operand occurs in                                              | it denotes                       |
|---------------------------------------------------------------------|----------------------------------|
| a [BLK-0] or [SYS-2] declared relation, naming an `own` parameter    | that call's CALL datum           |
| a [BLK-0] or [SYS-2] declared relation, naming a shared-borrow param | the live term                    |
| a [BLK-0] or [SYS-2] `requires`, naming a `&uniq` state parameter    | that call's CALL datum           |
| a [BLK-0] or [SYS-2] declared relation, naming a `&uniq` state param | the operation's POST-state       |
| a [BLK-0] or [SYS-2] declared relation, naming a result              | that result                      |
| a [FN-8] `requires`, naming a parameter                              | that parameter's ENTRY datum     |
| a [FN-9] `ensures`, naming an `own` or shared-borrow parameter       | that parameter's ENTRY datum     |
| a [FN-9] `ensures`, naming a `&uniq` parameter's MEASURE             | **inadmissible**                 |
| a [FN-9] clause, naming a result binder                              | that result                      |
| any of the above, read at the CALLER after substitution              | that call's CALL datum for a     |
|                                                                      | parameter operand, the result    |
|                                                                      | for a result operand             |
```

Two rows carry round 7's first BREAK and each has one reason. An `own` operand denotes
the **call datum**, because an `own` parameter is a value the operation received and its
post-state is not a thing; keyed on `writes` coverage instead, `seq_place`'s own relation
read `len(P) = len(P) + 1` and [MSR-4] step 1 discharged every goal in every loop in this
file from a contradiction. And a `&uniq` **state** parameter denotes the post-state in a
**declaration-domain** relation and is **inadmissible in a source-declared `fn`'s
`ensures`** — one mode, two callee classes, on [CALL-6]'s two-sided boundary: a
compiler-owned row is a declaration record complete over everything it writes [BLK-0],
while a wf body is a body, so a caller reading its post-state would be reading a claim
about an object at a point the callee cannot name (L11); probes `e2` and `e3` are that
pair at v0.41. **What it costs:** after [BLK-4] the only `&uniq` parameter reaching a
measured place is a provider, so *a user `fn` that lends a provider onward publishes
nothing about that store's post-state* (Q17).

> **Correction, decided 2026-09-04, from B1's implementation.** The eighth draft's table
> row read "naming a `&uniq` parameter" while its *Judgment* line read "a `&uniq`
> parameter's **measure**". Those are two rules, and the measure is the narrower one and
> the one that landed; the row above now says the measure and the two agree. **A
> non-measure operand over a `&uniq` parameter stays admissible in an `ensures`**,
> `deref(p).count` for one. The reason is L11 and it reaches exactly that far: the callee
> cannot name the caller's object at a point after its own writes, and only a measure the
> callee's writes change is a claim about that point. A plain field read is a live term,
> and the caller's own kill rules already govern it.

**One sentence fixes what an [INV-1] affine atom over a measured place is keyed by.**

> An [INV-1] affine atom over a measured place is keyed by the [ENT-2] term. **A [LIV-2]
> `set` target that names a binding in scope keeps that binding's term**: the statement is
> a write of the place, the facts over it die by [MSR-2], and the right-hand side's
> declared relations re-establish them on the same term through [CALL-6]. A target that
> resolves to no binding introduces one and is a declaration event, exactly as a `let` is.

*Judgment:* the atom-identity resolution at every [INV-1] atom over a measured place, and
the inadmissibility of a `&uniq` parameter's measure in a source-declared `ensures`, a
hard error citing MSR-3 at the clause with the restructuring `take the value by value and
relate the result, or state the fact as a requires`. A datum is formed, never proved.
*Publishes:* the datum at each of the six placements, the denotation table, and the
atom-identity rule. *Amends:* [ENT-2] 2677-2728's term list; [ENT-5] 2863-2967's
call-boundary paragraph 2898-2905 and its entry-image paragraph 2887-2891, replaced by
the datum and the table rather than repaired; [FN-9] 1301-1365's `M(c,q)` at 1345, its
parameter-entry-image sentence at 1316, and its operand admission, which loses a `&uniq`
parameter's measure in an `ensures`; [ENT-6] 2969-3100's image formation, join and
loop-header paragraphs; [ENT-3.S5] 2774-2781's copy-equality clause, which gains the
construct, rebind, payload and field placements' measured operands; [INV-1] 3101-3156 at
3109-3113. *Depends:* [ENT-2] 2693, whose one-static-term-per-statement argument is why a
per-point datum is sound; [ENT-5] 2942-2946; [FN-8] 1275, whose borrow-versus-own actual
split the call placement reuses; [OWN-13] 654, the event the payload placement attaches
to. *Verified today:* probes `e2` and `e3`; **the call placement landed**, conformance
cases `msr3-pos-own-operand-call-datum` and `msr3-neg-uniq-state-measure-in-ensures`
(6.0), the other five placements deferred with it. *Law:* L11, L16. *History:* r7 F1-1,
F1-16; r6 F1-2, F4-2; B1 (row versus judgment).

**[MSR-4] One numeric goal disposition, shared by every consumer.** [ENT-6] states once
the complete ordered derivation of a numeric goal:

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

The consumers are exactly: [OP-4] subscript bounds, [SYS-8] system range, [OP-2] integer
domain, [OP-9] allocation fit, [FN-8] requirements, [FN-9] normal-result relations,
[INV-1] invariant targets, [BLK-0]'s operation-domain obligations, and [RES-10]'s
trip-count bound. **The per-family route lists retire.**

**This rule is not widened, and step 1 is why the other repairs are at their rules.**
Widening `AUTO` would change the prover's complexity class and destroy [ENT-6] 3024's
promise that an author can determine from that rule alone whether a target is automatic.
Round 7 reached memory three times through step 1, which is its own hazard: *in this
language an inconsistent published relation is not a wrong fact, it is every fact.*
[CALL-6] carries the consistency check that follows.

*Judgment:* the disposition itself, at every goal every consumer submits. *Publishes:*
the disposition of every numeric goal. *Amends:* [ENT-6] 2969-3100's four per-family
route and attach-site grants at 3040, 3047, 3075 and 3084, which keep their normalization
and lose their route grant, and [FN-9] 1301-1365's direct-affine ordering branch, which
becomes one of the six steps. *Note:* an operation adds a goal, never a route. *Law:*
L16. *History:* r7 F1 (step-1 hazard); r6 F1-4; r1 F4-3.

**[MSR-5] The contract clause is the relation an invariant already is, over a wider
operand set.** **[S17]** A `requires` or `ensures` operand is a **term** of the [ENT-2]
term language, not an `atom` of [GRAM-5]; a `header_invariant`, an `invariant_stmt` and a
`proof_use` reach the same operand set at [MSR-4] in B2 and keep their atom set until
then. v0.41 does half the work: a clause's root is already one `compare_op` over two
`expr`s and a `header_invariant` is already `affine_expr compare_op affine_expr`. What is
left is the operand set — [GRAM-5] 258-280's `atom` has no `call` alternative, so
`len(source) <= room(out)` derives nowhere and probe `q7` is that rejection:

```text
clause_expr    := (atom | call | construct)
                  ((infix_op | compare_op) (atom | call | construct))?
```

`requires_clause` and `ensures_clause` ([GRAM-2] 185-186) take a `clause_expr`. It differs
from an `expr` in exactly one way, a `call` and a `construct` standing where an `expr`
admits only an `atom`, which is what lets a clause name a measure of a place on either
side of its comparison; every other position keeps [GRAM-9]'s one-operation-over-two-atoms
shape. The judgment is unchanged and is the one [FN-8] and [FN-9] already apply: the root
has exact value mode and type `own Bool` under [OP-5], and every operand is a
non-consuming datum or an operation-table form pure and total over its selected operand
domain. A contract clause admits all six comparison symbols where [INV-1] 3105 admits
four, which is what lets [CALL-7]'s clauses state an exact relation in one clause where a
header invariant costs two (Q14). The measure formers are **table data** over the measured
types, one row `len(P)` in v0.44 and `cap`, `room` and `head` when B7's types exist, each
admitted for exactly the places [ENT-2] clause (b) admits a length term for. A clause
operand that is neither an [ENT-2] term nor a constant stays an ordinary pure total
operand and contributes no L0 projection; clause position makes nothing a term. `Z` has no
source spelling; wf source writes `0_u64`.

**The affine surface is not widened here.** A `header_invariant`, an `invariant_stmt` and
a `proof_use` keep [INV-1] 3109-3113's atom admission, so a measure term is a clause
operand and not yet an affine atom; [MSR-4] widens the affine domain in B2, which is where
the affine index has to range over measure terms.

> **Correction, decided 2026-09-04, from B1's implementation.** The eighth draft wrote
> `clause_expr := affine_expr compare_op affine_expr`. That production has a comparison at
> the root and nothing else, so it drops every Bool-rooted clause the corpus writes today:
> `requires ok;`, `requires band(nonzero, not_neg1);`, `requires buffer_fits::<T>(length);` and
> `requires total /defined steps;`, the last being how a caller fixes an integer-domain
> predicate for [OP-2]. It also contradicts [FN-8]'s retained `.defined` admission, which
> this design keeps. The production above is what landed, and it is the correction: one
> operand, or two operands around one `infix_op` or `compare_op`, with the Bool root
> carried by the [OP-5] judgment rather than by the grammar.

*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set, unchanged
by the widening. *Publishes:* nothing new. *Amends:* [GRAM-5] 258-280 (a new
`clause_expr`; `atom` and `atom_list` unchanged), [GRAM-2] 168-202 at 185-186, [OP-5]
926-931, [FN-8] 1257-1299 at 1262-1267, and [FN-9] 1301-1365's operand list; [GRAM-4]
217-256's `affine_factor`, [INV-1] 3101-3156 at 3109-3113 and [GRAM-9] 328-332 are
unchanged. *Depends:* [INV-1] 3105, whose relation restriction this rule reads in
[FN-9]'s wider form. *Verified today:* probe `q7`; **landed**, conformance case
`msr5-pos-two-measure-clause` (6.0). *Law:* L16. *History:* r6 F3-8; r4 F3-5; B1
(production defect).

**[MSR-6] A const generic is a value wherever a named const is.** **[S21]**
[TYPE-6] 396-473's `pbase` admission at 401 gains **an in-scope const generic**, and
[ENT-2]'s `for_stmt` endpoint admission and [MSR-5]'s clause operand gain it with it. It
is a monomorphization-time constant, so [ENT-2] 2681 clause (c) already makes it a
symbolic constant *term*; this rule is the one sentence that lets a program **name** it.
Every capacity-parametric function in 3.L reads its bound in one of those positions, so
without it the library is unparseable. Probe `q10` is the rejection —
`available: [ConstGeneric]` — and it introduces no shadowing hazard, because a const
generic is already in the `LexicalIdentifier` declaration domain and a colliding `let` is
a [TYPE-6] `DeclarationCollision` today.

*Judgment:* the ordinary [TYPE-6] resolution over the widened admission and the ordinary
[TYPE-5] check at each use. *Publishes:* the const generic as a `pbase`. *Amends:*
[TYPE-6] 396-473 at 401, [ENT-2] 2677-2728 at 2685-2687, and [MSR-5]'s clause operand
through `ent2_place`. *Depends:* [ENT-2] 2681 clause (c), which is why this rule adds a
spelling and not a fact source. *Verified today:* probe `q10`. *Law:* L16, L18.
*History:* r6 F1-14; r5 F4-1.

#### 3.K.2 `[PROV]`: stores, brand, activation, and release

**[PROV-1] A store's identity is a region, the region is in the type, a region names at
most one live store, and every elided brand resolves by one rule.** A **store region** is
a region that names one store. A region becomes one by being named as the store argument
of a reserving occurrence [PROV-5], or, for the heap, by being minted for the entry heap
before `main`. **There is no third way.** A region may be named by **at most one**
reserving occurrence; a second is a hard error citing PROV-1 at that occurrence's `targ`,
`SecondStoreInOneRegion`, with the restructuring `open one region per store` ([OWN-3] 578
makes region identifiers unique within a function; probe `w1`).

Every value a store backs carries that store's region in its own type:

```text
| store       | provider [S3, S4]        | one run [S1, S2]     | release needs         |
|-------------|--------------------------|----------------------|-----------------------|
| general     | Heap<'s>                 | Vector<'s, T>        | the Heap capability   |
| bump extent | Arena<'s, bytes, align>  | Vector<'s, T>        | nothing; 's resets it |
| (none)      | (none)                   | FixedVector<T, n>    | nothing; the frame    |
```

The fourth column is [PROV-6]'s criterion's **type** half; whether a value of such a type
is linear at a point is its **scope** half (D3), and the two are deliberately separate.
The fourth column also determines the capability a release spends, because one store has
one provider [PROV-2]. `FixedVector<T, n>` has no store region because it has no store;
its capacity is in its type because a frame-resident run must have a size before layout
runs, and a store-resident run's capacity is a measure fixed at the take because a growth
policy must be able to change it.

**Preservation is a closure property and needs no clause.** A value's store is a
component of its type; no value-forming step changes a value's type; therefore none
changes a value's store. That covers `construct`, field projection, element placement and
removal, enum payload construction and `match` binding, multi-return, a join, a value-in /
value-out result, an argument transfer and a return. Two values have the same store
exactly when their types name the same region, which [OWN-12] 650 and [TYPE-5] 379 decide
by exact identity.

> **Brand resolution.** A **store** region elided at any position denotes:
> 1. at a **stored** position — a field, an enum payload, a run element, a written type
>    argument — the enclosing nominal's sole region parameter when it declares exactly
>    one, and the **entry heap's store region** otherwise;
> 2. at a **parameter or result** position, the **entry heap's store region** when the
>    position's type is `Heap` or is a type whose own brand resolves by clause 1 to the
>    entry heap, and an **implicit region parameter**, one per occurrence, otherwise.
>
> A **loan** region — the region of `Slice<'r, T>` or `MutSlice<'r, T>` — is never a
> store region, and an elided one is always an implicit region parameter. The entry
> heap's store region has no written spelling, because `main` declares no region
> parameter ([S22]). When the entry selects no `command.heap` there is no entry heap's
> store region, and a position that would resolve to it is a hard error citing BLK-4 at
> the complete contained `type`, `ConfinedTypeWithoutStore`.

Clause 2 is round 7's second BREAK repaired: the seventh draft stated two candidate sets
that never intersect, so `bs_reserve(s: own Bytes, heap: &uniq Heap, ...)` gave `s.v` and
`heap` two invariant regions and no helper anywhere could hold or release entry-heap
storage. Its cost is stated: a helper generic over the store writes its own region name,
and a helper that writes `Vector<u8>` bare is a helper over the entry heap — the right
default, because a function that releases entry-heap storage can only exist in a program
with a heap.

`Heap<'s>` is delivered as an `own` entry parameter and lives for the program; the
`command` standard-input table [FN-7] 1227-1233 gains ordinal 5, `command.heap` as
`own Heap`, supplying the one general store the runtime minted first. The `Heap` `main`
receives is dropped on the return edge with the **empty** release row.

*Judgment:* one live store per store region, established by [PROV-5]; the
`SecondStoreInOneRegion` error; the brand resolution above at every elided region
position, which is the judgment [PROV-6]'s capability resolution, [BLK-4]'s confinement
check and [RES-5]'s domain identity read; the ordinary [FN-7] checks; and the
exact-identity type equality [OWN-12] 650 and [TYPE-5] 379 already perform. *Publishes:*
each value's store, as a component of its type; the store-to-provider map [PROV-6]
resolves against; and the whole-program fact `heap-unreachable` when the entry row is
absent. Each store's measures are its rows' declared relations and are published through
[CALL-6]'s S13. *Amends:* [TYPE-2] 357-360, which gains the five branded and container
nominals and from which `box<T>`, `arena<'r, T>` and `buffer<T>` retire; [TYPE-7]
475-479, whose deref domain becomes the two borrow modes alone; [GRAM-3] 204-215, whose
`box`, `arena` and `buffer` productions retire in favour of TYPEIDs with `targs` and
whose `Slice` production is joined by `MutSlice`; [STOR-1] 675-683, whose storage-class
list at 675 gains the two runs; [OWN-10] 640-644 at 643; [FN-7] 1216-1255, whose table
gains ordinal 5, whose 1218 is **kept**, whose 1245-1246 gains the row, and whose 1220
gains `allocates` over a labelled input. *Depends:* [OWN-3] 578 and 580; [OWN-12] 650 and
[TYPE-5] 379, whose exact-identity equality is the invariance argument. *Law:* L2, L13,
L16. *History:* r7 F1-6, F5-19; r6 F1-5, F3-13.

**[PROV-2] Unforgeable, uncopyable, taken as a loan, and never stored.** No source
construct produces a provider; a `Heap<'s>` exists only because the runtime minted
exactly one before `main`, and an `Arena` only as the result of a reserving operation
[PROV-5]. No operation duplicates, reconstructs, compares, serializes or derives one.

An operation that **allocates** takes that store's provider as a written `&uniq 'b`
parameter and exhibits it. A provider is never passed `own`: it is confined to its own
store region, and a moved provider strands its own store. The one `own` provider is the
`Heap` the entry receives. **A release takes no provider parameter and could not choose
one**: the store is determined by the released value's brand and the provider by the
store, so a release resolves rather than takes.

**A provider parameter is the one `&uniq` [BLK-4] does not refuse**, because no operation
changes a provider's *identity*, only its measures. **What a caller may keep across such
a call is what the callee's declaration publishes, and for a user `fn` that is nothing**
[MSR-3]; only [BLK-0] and [SYS-2] rows hand a provider's post-state back (Q17).

*Judgment:* a `construct` [GRAM-8] naming a provider or container nominal, and every
other source route to one, is a hard error citing PROV-2 at the complete `construct`,
with the restructuring `receive the provider as a parameter, or reserve one with
arena_frame`; a provider type in a stored position is a hard error citing PROV-2 at the
complete contained `type`; and an allocation call whose provider argument is missing, is
not a provider place, or is not writable is a hard error citing PROV-2 at the `call`.
*Publishes:* uniqueness of the `Heap`, and the one-provider-per-store map [PROV-6] reads;
each store's post-state measures are published by [BLK-0]'s rows through [CALL-6]'s S13
and not by this rule. *Amends:* [OP-1] 771-849 at 798-803, from which `box_new` and
`arena_new` retire, and [STOR-2] 685-686, which defined them; [STOR-5] 723-736, whose
stored-content positions gain the provider prohibition. *Depends:* [OWN-10] 641, which is
why `'s` and `'b` are always distinct; [OWN-6] 614, which makes an argument borrow a
call-scoped temporary (probe `w8`) and is why store identity may not rest on what stands
at a place between two calls. *Law:* L2, L3, L4, L13, L16. *History:* r7 F3-I3; r6 F1-8;
r4 F1-a7.

**[PROV-3] Provenance is for loans, a loan reaches a logical range, a loan ends where its
value's liveness ends, and a loan-bearing value owns nothing.** [OWN-5]'s finite origin
set, today defined for `Slice<'r, T>`, generalizes to the two views and to nothing else.
A **loan-bearing** type is `Slice<'r,T>` or `MutSlice<'r,T>`; a value of one carries a
finite set of origins, each an origin place paired with the half-open **logical** index
range the value reaches of it [MSR-1].

Formation makes a **singleton**. A named const maps to the distinguished
`immutable-const` origin. Binding, moving, **copying**, passing and returning preserve
the set and its ranges; a control-flow join takes the union; a parameter of loan-bearing
type starts with the singleton containing its own formal origin, substituted at a call
boundary by [FN-1] 1041-1047's rule. The **resolved** set is the set minus
`immutable-const`, which creates no conflicting access [OWN-5] 607.

**A loan-bearing value owns nothing** (L10): what it reaches belongs to its origin, so no
obligation of what it reaches is ever a property of the view. [PROV-6] reads that twice,
and it is why a `Slice<'r, T>` can be **copy** ([S27]).

**A loan begins where its value is formed or copied and ends where that value's own
liveness ends** — for an affine view its consume or release, for a **copy** view its last
use, which [ENT-5] already computes. Each copy of a `Slice` holds **its own** shared loan
on the same ranges, which [OWN-5] admits without limit; a `MutSlice` stays affine
because [OWN-5] 606 refuses two exclusive loans on one range.

Four uses, and no fifth:

1. **Access strength, over the range.** An access through a shared-strength value is one
   shared access to the range of every resolved origin; through an exclusive-strength
   value, one exclusive access to the same. An ordinary access to a resolved origin is
   judged at the range that access reaches.
2. **A loan covers its address computation.** While a loan on a resolved place is live,
   every binding that place's address computation reads is frozen: a write to it is the
   ordinary [OWN-5] conflict, at the write, naming the loan.
3. **A live origin set fixes its storage.** While a value's origin set is live, no
   statement may write, replace, exchange, **or consume** the storage any resolved origin
   describes. This clause is **storage-keyed and says nothing about the view
   descriptor**; [VIEW-4] governs a commit at a loan-bearing place.
4. **Disjointness.** [OWN-7] 629's overlap test extends to logical ranges: two origins
   with the same resolved place overlap exactly when their ranges intersect, judged as
   [PAR-2] 2005 already judges, and carried to storage by [MSR-1]'s injectivity sentence.

*Judgment:* a loan-bearing value in a prohibited position [BLK-4] is a hard error there;
a write to, or consume of, the storage a live resolved origin describes is the ordinary
[OWN-5] conflict; a write to a binding a live loan's address computation reads is the
same; the loan's begin and end points, which is the judgment [VIEW-2] and [PROV-6]'s
release read; and use 4's range-overlap test, which [RUN-3] and [PAR-2] read.
*Publishes:* the origin set, the resolved set, each origin's logical range, the loan's
extent, and the sentence that a loan-bearing value owns nothing. *Amends:* [OWN-5]
585-611, whose slice-origin paragraphs generalize to loan-bearing values and gain the
copy clause and the extent clause, whose one access clause becomes the two of use 1 over
ranges, which gains the address-computation and resolved-set sentences, whose 608 becomes
"a formal view origin has a writable storage path inside its callee exactly when that
view's loan strength on its resolved origin set is exclusive", and whose 601-604 is
restated over the loan-bearing predicate; [OWN-7] 629-633; [SET-1] 481-511, whose 488-490
becomes *a target path may traverse a view value exactly when that view's loan strength
is exclusive*, admitting the `MutSlice` element write probe `p7` refuses today; [SET-2]
513-528, whose region-bearing target rejection is replaced by use 3 and [VIEW-4]; [EFF-1]
1369-1390 at 1386, which generalizes to a loan-bearing **parameter** and to no other
position; [EFF-2] 1392-1439 at 1406-1410. *Depends:* [FN-1] 1041-1047; [OWN-7] 630, whose
conservative subscript overlap makes use 2 checkable. *Law:* L10, L12. *History:* r7
F1-12, F5-14; r6 F2-14, F1-13.

**[PROV-4] `allocates` names a provider path, and reachability reads the leaf.** The
effect grammar's `allocates` entry takes formal-rooted [EFF-1] paths naming provider
state, in canonical order, replacing the fixed atoms:

```text
effect := "reads" "(" effect_path ("," effect_path)* ")"      // [S23]
        | "writes" "(" effect_path ("," effect_path)* ")"
        | "allocates" "(" effect_path ("," effect_path)* ")"
```

An `allocates(p)` entry is exhibited exactly when the body reaches an allocation whose
provider argument projects to `p` under [EFF-2]'s call-boundary projection. A function
*reaches a store* when its own row carries an `allocates` or `writes` entry whose path's
**selected type at the leaf** is that store's provider type, or when it calls one that
does. Because the compilation unit is closed [PROG-1] 1492, there are no function values
and no ambient store, the transitive closure over the call graph is exact and is computed
from signatures alone.

**The same closure computes reachability through a type**, and it has three readers:
[BLK-4]'s fourth clause, [PROV-6]'s release graph, and this rule. One closure, and the
one-field-wrapper defeat that killed [CNT-7] is closed by construction.

**An allocating row names the same provider path in all three categories, in [EFF-1]
1369's canonical order `reads(p), writes(p), allocates(p)`**; an allocator observes its
prior state while changing it, the both-categories case [EFF-1] 1389 already states. **A
release exhibits `writes` of the resolved provider place and no `allocates`**, because it
spends the store's capability without acquiring from it.

*Judgment:* [EFF-2]'s both-ways row check, unchanged, which [PROV-6], [RES-4] and [RUN-3]
read. *Publishes:* the provider-reachability closure; the type-reachability closure
[BLK-4] and [PROV-6] read; and the heap-reaching path [RES-4] prints. *Amends:* [EFF-1]
1369-1390's `effect` production, retiring the atoms `heap` and `arena`; [FN-3] 1102-1147,
whose conformance effect-row normalization over "the allocation set whose members are
`heap` and each alpha-mapped `arena` region" becomes the set of `allocates` paths under
the same ordinal identity 1127 fixes for `reads` and `writes`. *Depends:* [PROG-1] 1492.
*Law:* L2. *History:* r6 F3-6; r3 F3-12.

**[PROV-5] Reservation is an event of the region block, one live activation is the
condition, and the envelope item's name is post-monomorphization.** Two reserving
operations exist, differing only in placement:

```text
arena_frame<const bytes: u64, const align: u64>['s]()  -> own Arena<'s, bytes, align>   [S9, S4]
arena_extent<const bytes: u64, const align: u64>['s]() -> own Arena<'s, bytes, align>   [S9, S4]
```

No operand supplies any of those parameters, so each call writes its complete list in
[GRAM-2] 196-198's declaration order: `arena_frame::<4096, 16, 'a>()`. The written `'s`
must be a region introduced by an enclosing `region_stmt` of the reserving function; a
caller-supplied region parameter is not admitted, and [PROV-1] admits at most one
reserving occurrence per region.

**Each reserves one store per activation of the region block naming `'s`, and the
occurrence is a statement of that block and of no loop inside it.** The second half is
round 7's: one occurrence inside a loop whose `region_stmt` is outside it has one
activation and executes on every trip, and three readings were equally consistent with
the seventh draft — a fresh store each trip, breaking [PROV-1]; the same store each
trip, making `arena_frame`'s published `len(result) = 0` false from trip two; or a
refusal no rule stated. It is now a hard error citing PROV-5 at the `targ`, with the
restructuring `move the region block inside the loop, so the store is reserved and reset
per iteration`, which is the idiom [RES-10] recommends.

The `frame` form lays the extent out in the reserving activation's frame, so it enters
that context's `stack` item, and [STK-3] states that `frame(f)` includes the alignment
slack of every frame-placed arena in `f`. The `extent` form produces its own
`region(name, bytes, alignment, contiguous)` item, **named by the pair (concrete
function instance, `region_stmt` NodePath)** — round 7's other repair here, because
"derived from the reserving occurrence" left two live instantiations of one generic
sharing one committed extent, which is aliasing rather than exhaustion in an accepted
marked program. [FN-2] 1093-1100 already makes instantiation the point at which checking
happens, so `E`'s item count is a function of the expanded program.

**On every edge leaving `'s`'s block the store's release action resets it to its
initial state**: the bump cursor to zero, and nothing else. That action joins [STOR-3]'s
table and [RES-10]'s `reset` transfer is its arithmetic.

> An `arena_extent` occurrence is a hard error at its `targ` when **more than one
> activation of its region block can be live at one program point**. Three sources are
> refused by name: membership of a strongly connected component of the call graph, read
> **after** [STK-1]'s rewrite; reachability from more than one execution context, a
> worker lane included (1.5); and reachability from a statement an implementation may
> execute with overlapping execution under [PAR-1], [PAR-2] or [PAR-3]. The
> restructuring is `reserve the store in the caller and lend the provider down [PROV-7],
> or use the frame form`.

*Judgment:* the ordinary region, confinement and [OWN-5] judgments, plus the
region-locality check, the loop-free-occurrence check, [PROV-1]'s one-store-per-region
check, and the activation refusal, each a hard error citing PROV-5 at the `targ`.
*Publishes:* its store region and its envelope item — one `stack` contribution, or one
`region` item named by the pair above; the reserved store's measures are the row's
declared relations, published through [CALL-6]'s S13 at the reserving call. *Amends:*
[STOR-3] 688-719, whose release-action table gains the store reset. *Depends:* [ERR-4]
1487, whose "absence of a complete permission derivation ... never rejects the source"
is why the `par` source is read as *may execute with overlapping execution*; [EFF-3]
1441-1444, whose guard list is why a `pure` reserving call is not deduplicated or
reordered, and [BLK-4]'s confinement, which is why it is not hoisted out of its block.
*Law:* L2, L5, L6, L13. *History:* r7 F2-5, F2-8, F2-20; r6 F2-9; r5 F1-12.

**[PROV-6] Linearity is the reclamation half of affine, read against the scope, closed
under ownership; the release is derived where the capability is held.**
**Owner-decided 2026-09-03**: D1 settles the criterion and the modifier together, and
**D3 settles that the criterion is read against the scope.**

**The criterion, in two halves.** The **type** half: *a type's release action requires a
capability when its own reclamation is a release to a store whose provider is a value*
[PROV-1]. `Vector<'s, T>` at a `Heap` region requires the `Heap`; at an `Arena` region
it requires nothing, because the region's reset reclaims it; `FixedVector<T, n>`
requires nothing; a compiler-owned system resource requires nothing, because its release
is the runtime's [RES-9]. The **scope** half:

```text
a value is LINEAR IN A SCOPE exactly when it OWNS, at any depth, either
    a value whose release action requires a capability THAT SCOPE DOES NOT HOLD,
 or a value whose declaration carries the `linear` modifier
and it is AFFINE in that scope otherwise
```

A type **owns** its fields, its enum payloads, and the elements of a run it is. **A
loan-bearing type owns nothing** [PROV-3, L10]. The seventh draft's fourth ownership
clause — *the values a written type argument of it stands for* — is **deleted**: a type
argument is owned through the field, payload or element it lands in, and as written the
clause made `struct Tag<T> { n: u64; }` at `T = Vector<u8>` linear while owning nothing.

> **A scope holds a capability** exactly when a binding of that store's provider type is
> live at that point in that function, reached directly or through a borrow. The
> capability cannot be smuggled, because a binding of a provider type enters a function
> only as a parameter or as the entry input: **a scope that gets a derived release says
> so in its own parameter list, and its effect row carries `writes` of that provider.**

**Why the scope reading is the correct shape.** [LIV-1] is a per-**edge** obligation, so
a scope-blind criterion costs one written statement per (value, edge): round 7 counted
**forty** in `byte_string.wf`'s `main` and **sixty-eight** in `decode_dynamic`, and the
repair a writer takes is to invert every hosted function into a single exit, deleting
early-return style from hosted code. Under D3 those are the ordinary derived release
[STOR-3] already defines, running unconditionally at [LIV-1]'s existing join with no
drop flag. **L2 and L3 are untouched**: the capability is a held value named at a
parameter, and the free is more visible than forty scattered statements because it
appears in an effect row where today's compiler emits it under none (probe `r2_5`).

**The `linear` modifier.** `linear struct N { ... }` and `linear enum N { ... }` are one
added modifier on [GRAM-2]'s `struct_decl` and `enum_decl`, for a **logical** obligation
only, holding in **every** scope. Its admission condition is round 7's:

> The modifier is admitted only on a nominal [OWN-1] 563-564 classifies as **affine**.
> `linear` on a tag-only enum — which 563-564 makes **copy** and probe `q11` confirms is
> used twice bare today — is a hard error citing PROV-6 at the `enum_decl`, with the
> restructuring `give a variant a payload, or put the obligation on the value the issuer
> hands out`.

**What it buys is must-consume, visibly — and not must-return.** `destructure whole`
legally throws an affine-fielded nominal's contents away in one visible statement, so **a
directional obligation is bought by proving the return**, which round 7 showed is true of
every shape 3.L.7 lists and not only of a pool's lease. It is also the one thing in this
family no wf program can have: a writer can write a pool, not *a type whose silent drop
is refused*.

**The routes out of a scope.**

```text
a value LINEAR in this scope:  moved out whole, or destructured whole [S13]
a value AFFINE in this scope:  those two, plus `dispose p;` [S12], plus the one
                                 compiler-derived release on every leaving edge
```

An own-place `match` [OWN-13] 653-662 is a destructuring. **Destructure whole** is
`let N(f1: b1, ..., fk: bk) = move v;` **[S13]**, one added `let_stmt` alternative that
consumes a value of nominal type `N` and binds every field in declaration order to a
fresh IDENT, judged exactly as [CALL-4]'s multi-result destructuring `let` is: each
binder receives its field's declared type and `own` mode, each measured binder receives a
**field placement** datum [MSR-3], and no residual exists.

**The release walk, its graph, and why it terminates.** One walk performs both the
compiler-derived release and `dispose`, and one graph bounds it:

> **The release graph of a type `T`.** Its nodes are the types reachable from `T`
> through fields, enum payloads and run elements; a loan-bearing value contributes no
> node. A type's **release action is non-empty** is the least fixed point of: a
> capability-released leaf is non-empty; a compiler-owned system resource type [STOR-3]
> 709-712 is non-empty; and any type owning a non-empty type is non-empty. The graph has
> an edge from a node to a sub-node exactly when that sub-node's release action is
> non-empty. **The walk visits exactly the nodes of the release graph**, in [STOR-3]
> 700-706's order — every field of a struct in that order, an enum's active variant's
> payload selected by the discriminant, every element of a run's initialized window in
> ascending logical order — releasing at each capability-released leaf to the store its
> own type names and spending that store's resolved provider, and running each other
> non-empty leaf's ordinary release action. A field whose release action is empty is
> never visited.
>
> **A type whose release graph has a cycle is a hard error citing PROV-6 at its
> `struct_decl` or `enum_decl`, in every program**, naming the cycle, with the
> restructuring `hold the cells in a run and link by index`. Because the graph is
> acyclic and finite, the walk's depth is a compile-time constant and it uses no
> auxiliary storage.

Round 7 broke both halves of the seventh draft's version — its refusal was stated over a
sub-graph "reached **through leaves**", and nothing is reached through a terminal, and its
walk and its refusal quantified different graphs — so an arena-recursive `Node` inside a
heap-backed `Root` was accepted while its walk recursed to a runtime tree depth. One graph
closes both: `Node`'s release action is empty, so the walk never enters it and the depth
is one, while `tests/programs/recursive_tree.wf` over heap-backed runs is refused in every
program, which is the honest consequence. **A container's elements are visited before its
backing is released**, so a release on a full container needs no emptiness premise.

**`dispose p;` is the early release, and it names no capability.** **[S12]** One added
statement form, admitted exactly where the value is affine in this scope, running the
same walk the scope exit would run:

> **Admission.** `dispose p;` is admitted only when `p`'s release graph contains at
> least one capability-released leaf, when this scope holds the capability of every such
> leaf, and when **no node of that graph — `p`'s own type included — is linear by the
> modifier**. A type containing a modifier-linear node is taken apart by a destructuring
> consume first. The "`p`'s own type included" clause is round 7's: the modifier can only
> be written on a struct or an enum, which the walk never treats as a leaf, so a
> condition quantified over leaves reached nothing and `dispose lease;` silently
> discharged the obligation the modifier exists to create.
>
> **Resolution.** For each capability-released leaf, let `'s` be the store region its
> type names and `P('s)` the provider type of `'s`'s store. The statement resolves the
> **innermost live binding of this function whose type is `P('s)`**, reached directly or
> through a borrow, and **writes** it; probe `p7` shows two live bindings of one spelling
> are a [TYPE-6] `DeclarationCollision`, so "innermost live" is determinate. No such
> binding in scope, or only one reached through a **shared** borrow, is
> `DisposeHasNoProvider` with the missing parameter rendered.

The statement is **one consuming use** [OWN-1] of `p`'s root — `p` must be rooted in a
live own-mode binding *of this function* **whose type is not loan-bearing** and whose
release graph **contains no loan-bearing node** — and **one write of `p`'s ultimate
storage origin** beside the write of each resolved provider, so [EFF-2] projects it,
[MSR-2] kills over it and [PAR-1] 1975's footprint contains it. The two loan-bearing
conditions are two notions: the ownership closure read at the operand, and the walk's own
domain.

**Why `dispose` survives D3.** The derived release runs at the scope exit; `dispose`
runs it **earlier**, which is the difference between a peak of one buffer and a peak of
two in `bs_reserve`, and between one and `n` in a loop whose scope is the whole program.
No wf statement performs a structural walk of a type the writer did not declare.

**A partial consume of a value that is linear in this scope is a hard error, and
[LIV-2]'s commit is not one.** [OWN-1] 569's "after any consuming use, the whole binding
rooting `p` is dead" is the one event that makes a linear binding *not live* without
discharging it; the refusal is stated over the **consume**, so it reaches
`dispose chunk.page` as well as `move chunk.page` (probes `x4`, `g7`, `p6_partial`).
Round 7 found it refusing the design's own central statements, so the exception is stated
where the reason is:

> A consume of a proper sub-place of a value linear in this scope is a **partial
> consume** exactly when that sub-place is **not reinitialised at the same statement's
> commit**. A [LIV-2] target list every member of which is reinitialised at one commit
> leaves no residual leaf, so the refusal does not reach it; every other consume of a
> sub-place does.

That admits `set (kept.v, total) = collect(...)`, `set block.run = seq_place(...)` inside
a `linear Lease`, and `bs_reserve`'s drain, and keeps the refusal where the residual
really is abandoned.

**A declaration has one verdict, and a region parameter's store class is read from the
declaration.**

> A declared region parameter `'s` is an **arena** region when the declaration writes a
> parameter of type `Arena<'s, ...>`, a **heap** region when it writes one of type
> `Heap<'s>`, and **unconstrained** otherwise; a value branded by an unconstrained `'s`
> is treated fail-closed as capability-released. A function that declares a region
> parameter `'s` may not let an **own-mode** value that owns, at any depth, a leaf
> branded `'s` reach a scope exit by a compiler-derived release unless `'s`'s class is
> arena or the declaration holds `'s`'s provider. Its four routes are: move it out by a
> result, destructure it whole, dispose it, or take the compiler-derived release — the
> last two available exactly under that condition. The check is at the declaration, over
> the body, once; a hard error citing PROV-6 at the `fn_decl` names the region, the
> binding, and both repairs.
>
> **[S32]'s bound is read here** (owner-decided 2026-09-04): a region parameter written
> `'s: affine` or `'s: linear`, and a type parameter written `T: affine` or `T: linear`,
> is **bounded** rather than unconstrained, so this rule is checked once against the bound
> instead of fail-closed, and an instantiation whose region or type argument does not
> satisfy the bound is a hard error citing PROV-6 at the call, naming the parameter, the
> bound and the argument.

The population is the one the notion is about: it costs nothing to `pool_new` (its
`Arena<'s, ...>` parameter fixes the class) or to `collect`, `render`, `drain`,
`pool_take` and `pool_release` (each moves the value out), and it refuses exactly the
**consuming** helper over an unconstrained region — `fn checksum['s](v: own Vector<'s,
u8>) -> sum: own u64` — whose relief is [S32], now adopted. Written
`fn checksum['s: affine](v: own Vector<'s, u8>) -> sum: own u64`, the declaration states
the class its body was written for, the fail-closed treatment does not apply, and a
heap-branded instantiation is refused at the call instead of the declaration being refused
at every one. **`propagate` is [LIV-1]'s judgment and
this rule does not restate it**; the seventh draft's second, wider sentence here refused a
`propagate` on account of a binding it had nothing to do with, and is deleted.

*Judgment:* the linearity predicate, computed per scope from the criterion above, which
is the judgment [LIV-1], [BLK-1], [STK-1] and [RES-10] read; the modifier's
affine-nominal admission; `dispose`'s admission, resolution and two operand conditions;
the release graph's acyclicity, a hard error at the `struct_decl` or `enum_decl`; the
partial-consume refusal and its reinitialisation exception, with the restructuring
`destructure the whole value with let N(f: a, ...) = move v;`; the declaration-site
region obligation; and [S32]'s bound, read at the declaration and checked at every
instantiation. *Publishes:* the linear predicate per scope; the release events and
their order; the statement's write of `p` and of each resolved provider; and the walk's
effect contribution, which [RES-10] charges and [RUN-3] reads. *Amends:* [STOR-3]
688-719, whose `box<T>` and `buffer<T>` heap rows retire with their types and are
replaced by the release-graph walk, whose table gains the store reset [PROV-5], and whose
690 edge enumeration gains the `propagate` error edge; [OWN-1] 563-571, whose
classification at 563-564 is unchanged and which gains the linear refinement, the
partial-consume refusal at 569, and `dispose` in its consuming-use list; [GRAM-2]
168-202's `struct_decl` and `enum_decl`, [GRAM-4] 217-256's `stmt` and `let_stmt`, and
[FORM-2] 39-89, which renders each on one line; [EFF-2] 1392-1439 at 1427, whose "each of
these memory-reclamation actions carries the empty effect row" becomes *carries the empty
effect row exactly when the walk spends no capability, and otherwise `writes` of each
resolved provider place*; [PAR-1] 1971-1998 at 1975; and [ERR-3] 1472-1482, whose
retained judgments gain [LIV-1]'s per-edge refusal. *Depends:* [STOR-3] 700-706, the
order the walk reuses; [OWN-5] 591, what the consume half of `dispose` inherits, and 606,
why at most one live binding lends `&uniq` to a provider; [OWN-13] 654; [PROV-4]'s
type-reachability closure, which computes the release graph's nodes. *Law:* L3, L5, L13,
L17. *History:* r7 the owner's D3, F5-1, F5-4, F5-5, F5-12, F5-15, F5-23, F1-5, F1-7,
F3-3, F3-4, F4-1; r6 F1-3, F1-4, F1-10.

**[PROV-7] A provider can be lent onward, generally.** A helper that receives a
provider as `&uniq 'b P` must be able to hand it to the operation that allocates. Today
it cannot: [OWN-6] 613-627's child reborrow admits only a locally-introduced region whose
block does not extend beyond the enclosing statement. The amendment is [OWN-6]'s own
reasoning applied one position further, stated over every child reborrow:

> A child reborrow may name a caller-supplied region `'b` that resolved(`holder`)'s
> region outlives-or-equals **when the receiving call's result type does not name `'b`**.
> That child's loan ends at the end of its receiving statement, and the parent resumes
> there.

*Judgment:* [OWN-6]'s admission with one more admitted region source under the stated
result-type condition, which is the judgment [PROV-6]'s capability resolution reads when
the resolved binding is itself a borrow. *Publishes:* the child loan's extent. *Amends:*
[OWN-6] 613-627 at 616 and [OWN-4] 582-583. *Verified today:* probes `r1_relend` and
`m19` are `[OWN-6] InvalidChildReborrow`, and `r1_relend_affine` shows the local-region
escape cannot carry an affine result out. *Note:* this unblocks `docs/patterns.md` P17's
threaded-factory shape. *Law:* L2. *History:* r4 F4-9; r2 F2-N3.

#### 3.K.3 `[BLK]`: the branded run of slots

**[BLK-0] The kernel declaration domain.** The container and store operations are one
compiler-owned **generic** declaration domain, built as [SYS-1] 2136-2162 and [SYS-2]
2164-2307 build the system domain and admitted to every compilation unit on the same
terms. Each operation is one complete signature record: named parameters in declared
order [GRAM-11] 345-350, its type, const and region parameters as [GRAM-2] 196-198 orders
them, one declared effect row, one declared result mode and type or one ordered result
list, one declared requirement list, and one declared relation list. **The first declared
parameter is the value the operation transforms and returns; an operation that transforms
nothing names its provider first; and one that neither transforms nor provides names the
value it observes first.** The inventory is A.2; the rule is that it exists and that
every row satisfies the six sentences below, the first-parameter ordering included.

**Written arguments, per argument.** A row writes each **region** argument exactly when
no operand of that row determines it [FORM-8], and writes each **type or const** argument
exactly when no operand of that row supplies it — [TYPE-5] 370-394's own
retained-argument sentence applied to a fourth callee class, not [FORM-8]'s criterion. So
`seq_heap::<u8>(heap: ..., count: ...)` writes `T` and elides `'s`;
`seq_arena::<u8>(arena: ..., count: ...)` writes `T` and elides `'s`, `bytes` and
`align`, all three of which the `arena` operand supplies; and `seq_place(vector: ...,
value: ...)` writes nothing. **A user `fn` generic is the other class and always writes
its type and const arguments** [FN-2] 1093-1100, probes `q4` and `q5`.

**The argument form is named.** A kernel-domain call writes its value arguments as a
`fieldinit_list` in declared order. [GRAM-11] 346 admits that form for exactly "a user
`fn` or ... an admitted system operation", 348 forces positional operands for an [OP-1]
table operation, and 350 resolves callee kind by the partition [OP-1] 838 states; a
kernel-domain operation is a fourth class in all four sentences and in [TYPE-6] 401's
`callee` IDENT admission.

**Every row is complete over every measure it writes, on every exit.** A row carrying
`writes(P)` for a measured `P` publishes, for **each** measure of `P`, its exact new
value where that measure is exact and a two-sided bound where it is bounded, including
the measures it did not change and **on every exit including a refusal** (L15). The
arithmetic it buys is why it exists: reconstructing `room` from `len` and `cap` costs two
premises before the goal is reached, and the design's own `spare` invariant then needs
three where [ENT-6] 3015 admits two (probes `g4`, `g3`).

**A row's operands are terms, constants, and the compiler-owned formers A.1 defines.**
`advance<T>(count)` is one term of fragment type `u64` with the support of `count`, whose
value A.1 fixes; it is a symbolic constant when `count` is closed and an opaque term
otherwise, so `room(arena) >= advance<T>(count)` is an ordinary difference bound between
two terms that [ENT-4]'s L0 holds. Round 7 found the seventh draft writing `round_up`,
`size_ceiling` and `align_ceiling` into rows as operands no rule admits, with a non-affine
shape for a symbolic `count`. `fits::<T>(n)` is **not** a term: it names [OP-9] 974-1001's
allocation-fit obligation, discharged by [OP-9]'s judgment.

**Every acquiring row carries [OP-9]'s allocation-fit obligation** as
`requires fits::<T>(count)` — probe `a4` is that judgment firing today — and `seq_fixed`
carries none, because `n` is a type constant [STOR-6] 738-767 covers.

**The readers are not in this domain.** `len`, `cap`, `room` and `head` are four [OP-1]
table operations taking a bare non-consuming place operand, returning `own u64`, and
**`pure`**: [EFF-2] attributes the operand's own read as for any other non-consuming
table operand, so a **caller** reading a measure of a borrowed place exhibits `reads` of
it (probes `r2_10`, `t10`). **A `let` binding one of them establishes an equality**:
[ENT-3.S6] 2782-2786's row generalizes over [MSR-1]'s four measures, and without it no
`cap`, `room` or `head` value is ever a fact.

*Judgment:* row resolution by name, receiver type and written arguments; the per-row
requirement discharge under [MSR-4], the allocation-fit obligation included; the
[GRAM-11] named-argument check; the completeness check over each row's published relation
set; and the first-parameter ordering check over the inventory. A diagnostic cites
**[BLK-0]** and names the operation in its payload, as an [OP-1] diagnostic cites [OP-1].
*Publishes:* every declared relation of every row, at the denotation [MSR-3]'s table gives
it and through [CALL-6]'s S13, which is the source and the destination. *Amends:* [SYS-1]
2136-2162 (a fourth admitted declaration source), [SYS-3] 2309-2311, [TYPE-6] 396-473 (the
domain's spellings and 401's `callee` admission), [DIAG-1] 1541-1883 (collision rank 5 and
a `container_declaration_ordinal`), [ENT-3.S6] 2782-2786 (the equality row generalizes),
[OP-1] 771-849 (`len` gains `cap`, `room` and `head` over runs, views and providers;
`slice_of`, `buffer_new`, `buffer_vacant`, `box_new` and `arena_new` retire;
`ReservedLowerNames` gains three; 838 gains the class), [OP-9] 974-1001, [TYPE-5] 370-394
(the written-argument criterion covers a fourth callee class and becomes per-argument),
[GRAM-11] 345-350, and [FN-2] 1093-1100 through [TYPE-5]'s retained-argument sentence.
*Depends:* [CALL-6]'s S13, which is how this domain's declared relations become facts;
[ENT-6] 3015, whose two-premise family is why completeness is stated at all. *Law:* L11,
L15, L16. *History:* r7 F1-1, F3-I14, F3-9; r6 F2-6, F1-4.

**[BLK-1] Two runs, one shape, one window, and what a slot may hold.**

```text
| type [S1, S2]       | capacity            | storage              | release needs  |
|---------------------|---------------------|----------------------|----------------|
| FixedVector<T, n>   | the type constant n | inline in its owner  | nothing        |
|                     |                     | or the stack frame   |                |
| Vector<'s, T>       | a measure, fixed at | one run taken from   | what 's needs  |
|                     | the take            | the store 's names   |                |
```

**Each is a run of slots whose initialized storage is a window** (L12, owner-decided):
exactly the `len` slots beginning at `head` modulo `cap`, the rest raw. A run carries no
other state — no per-slot tag, no occupancy bitmap, no runtime discriminant. A subscript
`v[i]` selects the element at **logical** offset `i` [MSR-1] and carries the ordinary
[OP-4] obligation `i < len(v)`, against `len` and never against `cap` or `head`. A
`Vector<'s, T>` of capacity one is a single stored value, so the language needs no box
nominal. `array<T, n>` **retires** [S34]: it was the `len = cap = n`, `head = Z` case, and a
`FixedVector<T, n>` whose four measures are standing facts is that case with no runtime
descriptor word, so a `const` of `FixedVector<T, n>` type with exactly `n` literal
entries is the const-eligible form [CONST-1], lowers to element storage only, and
materializes its descriptor from the standing facts at each use.

**Why a window and not a prefix, and what it costs.** The prefix made L12's last clause
false: a queue is not arithmetic over append and remove-at-the-end, and the price was a
library ring over `Option<T>` measured at **2072 bytes against a hand-written 280** for a
256-byte ring under [OP-9] 992's ceiling, with in-place slot mutation deleted. Its cost is
five things and no sixth: one word per descriptor (A.1); one more measure term, `head`;
one standing fact, `head(P) <= cap(P)`; one requirement on view formation,
`head + len <= cap` [VIEW-2]; and **an O(len) drain to return a wrapped window to its
origin**, because after a front operation `head` is known only as `Z <= head <= cap` and
no back operation re-establishes it. The seventh draft answered the fifth with a fifth
kernel row; **round 7 wrote the replacement in wf**, so under L18 it is not a kernel row
and 3.L.8 walks and prices it. Q18 is the owner's question if a driver's `E` cannot afford
the second run.

Lowering pays one add and one conditional subtract per subscript — a runtime cost and not
a proof cost, and an optimizer that proves `head` identically zero emits the ordinary
`base + i * stride`. In a ring `head` is genuinely nonzero, so a completion handler
touching six fields pays it six times; the repair is to borrow the element once, and probe
`x10` shows that shape unsupported today.

`T` may be copy, affine, or linear ([OWN-1] 563-564 plus [PROV-6]'s refinement). The
window is what makes an affine element sound: an element enters and leaves only through an
operation that moves a boundary. A run over a `T` linear in some scope **owns** its
elements, so it is linear there too and the release walk visits its window.

*Judgment:* the ordinary nominal-resolution and construction judgments; a `construct`
naming a container nominal is a hard error citing BLK-1; [OP-4] at every subscript against
`len`, which is the judgment [PROV-3] use 4 and [RUN-3] read after [MSR-1]'s injectivity
sentence. *Publishes:* the two types, their measure rows and their window typestate.
*Amends:* [TYPE-2] 357-360, two added composite types and its flat-element restriction,
which the runs do not inherit; [OP-4] 914-924, whose indexable bases extend to the two
runs and the two views and whose obligation is against `len`. *Verified today:*
`array_new::<box<u64>, 4>` is [OP-1] `InvalidOperation` (probe `p9`). *Law:* L12, L13.
*History:* r7 F3-7; r6 F2-15, F4-7.

**[BLK-2] Formation, one row per placement and one per store.** Four rows, and no fifth:

```text
seq_fixed<T, const n: u64>()                          -> own FixedVector<T, n>   pure  // [S7]
seq_arena<T, const bytes: u64, const align: u64>['s](
      arena: &uniq Arena<'s, bytes, align>, count: own u64)
                                                      -> own Option<Vector<'s, T>>
seq_arena_proved<T, const bytes, const align>['s](arena: ..., count: own u64)
                                                      -> own Vector<'s, T>
seq_heap<T>['s](heap: &uniq Heap<'s>, count: own u64)  -> own Option<Vector<'s, T>>
```

Each acquiring row carries [OP-9]'s allocation-fit obligation over `(T, count)`, and each
arena row additionally requires `align >= align_ceiling(T)` as a compile-time comparison
of two constants — which is what makes the cursor a multiple of `align`, the padding at a
take zero, and `len(arena)` **exact** [MSR-1, RES-5].

**Every failure is an `Option` and the kernel declares no failure nominal**, because no
kernel acquisition takes an affine input: a count is copy and a provider is borrowed. The
`Heap` has **no proved form**, because no honest domain predicate exists for a general
store (L6); the arena has one, whose requirement [MSR-4] discharges and whose failure is a
static rejection with no fallback.

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge at the proved spelling and at
every allocation-fit and alignment obligation. *Publishes:* each run's measures, and each
store's post-state measures and refusal relation, through [CALL-6]'s S13. *Amends:* [OP-1]
771-849 at 798-803; [TYPE-2] 357-360. *Depends:* [CALL-6]'s S13; [EFF-3] 1441-1444, whose
guard list is why a `pure` formation is not deduplicated across a store's state. *Law:*
L3, L4, L6, L8, L18. *History:* r7 F2-14, F2-20; r6 F2-6.

**[BLK-3] Four operations move a boundary, and nothing else does.** `V` is either run
type.

```text
seq_place(vector: own V, value: own T)        -> own V   // [S8]  requires room(vector) > Z
seq_place_front(vector: own V, value: own T)  -> own V           requires room(vector) > Z
seq_take(vector: own V)                       -> (rest: own V, value: own T)
                                                                 requires len(vector) > Z
seq_take_front(vector: own V)                 -> (rest: own V, value: own T)
                                                                 requires len(vector) > Z
```

Element access is the ordinary v0.41 surface over the initialized window: `v[i]` reads,
`set v[i] = e;` writes a copy element [LIV-2], and `let old = replace v[i] = e;` exchanges
an affine one [SET-2] (probe `x7`). Each row takes the run **by value** and returns it,
carries `reads(vector), writes(vector)`, and publishes its complete measure row on every
exit. **Its `vector` operand is `own`, so every occurrence of `len(vector)` in its
published relation denotes that call's call datum** [MSR-3] — round 7's first BREAK stated
where it bites: under the seventh draft's `writes`-keyed table
`len(result) = len(vector) + 1` read `len(P) = len(P) + 1` and every loop in this file
proved `false`.

**There is no `seq_rebase`, no swap and no exchange operation, anywhere.** Returning a
wrapped window to its origin is a library drain (3.L.8). A swap of two whole
non-overlapping places is `set (p, q) = move q, move p;`; a swap of two elements of **one**
run is refused by [LIV-2]'s non-overlap condition and is three statements over the rows
above:

```wf-design
let (rest, endv) = seq_take(vector: move vector);
let old = replace rest[at] = move endv;
let back = seq_place(vector: move rest, value: move old);
```

**What it costs is stated correctly**: the three statements kill and re-establish `len`
twice, and the middle statement's obligation is `at < len(rest)` where
`len(rest) = len(vector) - 1`, so a caller must prove `at + 2_u64 <= len(vector)` and the
last position needs a dominating branch. 3.L.2 walks it. There is **no removal from the
middle, no clear, no truncate, no growth, no filled construction and no vacant
construction** in the kernel.

*Judgment:* [BLK-0] row resolution and [MSR-4] discharge of each requirement.
*Publishes:* each row's declared relations, through [CALL-6]'s S13. *Amends:* nothing
beyond [BLK-0]'s. *Verified today:* probe `c8` shows a function writing one position of an
`own buffer<u8>` parameter and returning it must exhibit `writes(vector)`, and probe `c8a`
is the same check on the row this design writes. *Law:* L4, L9, L12, L15, L18. *History:*
r7 F1-1, F3-7; r6 F2-15, F1-7.

**[BLK-4] Confinement, the one position closure, and the `&uniq` parameter refusal.** A
type is **confined** when its complete type after substitution names a region. The
confinement of a value is the **set** of regions its complete type names, and it may be
moved, returned, or bound to a destination that **every** member outlives-or-equals
[OWN-3] 576-580 — the quantifier is the whole rule, because [OWN-3] 580 makes two
caller-supplied regions incomparable and fail-closed is the right answer.

A confined value may occupy any position whose owning value's own complete type names the
same region, so the position is itself confined and [STOR-4] 721 governs it. That admits a
store-branded value into a field, a run element and an enum payload, and it is safe
because the store's identity travels in the type [PROV-1]. A **loan-bearing** type
[PROV-3] may occupy no position from which a value could outlive or hide its origin set:
no field, no enum payload, no run element, no generic type argument, and no result outside
[VIEW-6]'s ceiling. A **provider** type may occupy none of the same positions, for
[PROV-2]'s reason. A store-branded run may occupy any of them.

**And no container nominal, no loan-bearing type and no unbounded generic type parameter
may be the referent of a `&uniq` parameter of a source-declared `fn`.** This is R1 as a
rule:

> In the parameter list of a source-declared `fn`, a parameter of mode `&uniq` is a hard
> error citing BLK-4 at the complete `param`, `UniqueParameterReachesContainer`, when its
> referent type **is, or reaches at any depth, a container nominal, a loan-bearing type,
> or a generic type parameter carrying no bound that excludes both**. Depth is the
> reachability closure [PROV-4] computes over fields, enum payloads, run elements and
> written type arguments. The restructuring is `take the run by value and return it, or
> take a view of it`.

Three things about it are deliberate. **The closure closes the round-4 defeat**, where a
one-field wrapper struct nullified [CNT-7]. **The type-parameter clause is round 7's**:
the closure is computed at the declaration, where a type parameter is opaque, and probe
`gen3` compiles `fn swapin<T>(handle: &uniq Holder<T>, made: own T)` at `T = buffer<u8>`
**today** — so R1 held for every declaration except the one a library writer naturally
writes, and the defence was again [CALL-3]'s conservative kill, which round 5 defeated
twice. Refusing the position rather than the reached type is fail-closed and decidable at
the declaration. **[S32]'s bound is what the clause reads** (owner-decided 2026-09-04): a
type parameter carrying a linearity bound is decided by that bound at the declaration
rather than fail-closed, so a `&uniq` referent that reaches it is admitted exactly when
the bound admits no container nominal and no loan-bearing argument at any instantiation,
the verdict stays one per declaration, and probe `gen3`'s unbounded `fn swapin<T>` stays
refused. **And the clause quantifies over a
source-declared `fn`** and not over the compiler-owned domains, because a [BLK-0] or
[SYS-2] row is a declaration record whose relations are complete over everything it writes
and whose behaviour no body can vary — L11's second sentence — so
`seq_mut_slice(vector: &uniq 'r v)` and `read_at(destination: &uniq MutSlice<u8>, ...)`
are unaffected.

**A source nominal may declare region parameters** — `struct Chunk['s] { page:
Vector<'s, u8>; }`, [S20] — and is confined by them; under [PROV-1] a nominal over the
entry heap declares none and is written `Chunk`. Region parameters on a nominal are
**invariant** ([OWN-12] 650, [TYPE-5] 379). **A stored position with no admissible store
is this rule's error**, `ConfinedTypeWithoutStore`: a nominal with no region parameter, an
entry selecting no `command.heap`, and a field needing a store brand is a hard error at
the complete contained `type`, with the restructuring `give this nominal a region
parameter and confine the field to it`.

*Judgment:* the `&uniq` parameter refusal over [PROV-4]'s closure, which is the judgment
[CALL-3] and [MSR-3]'s `&uniq` row read; a loan-bearing or provider type in a prohibited
position, or a confined type in a position whose owner does not name its region, is a hard
error citing BLK-4 at the complete contained `type`; and a confined value bound to a
destination some member of its region set does not outlive is a hard error at the binding,
rendering every member. *Publishes:* the confinement set, and the fact that no
source-declared `&uniq` parameter reaches a container nominal, a loan-bearing type or an
unbounded type parameter. *Amends:* [STOR-4] 721; [STOR-5] 723-736, whose position list is
replaced by the intensional split and whose per-leaf-provenance deferral is **withdrawn as
unnecessary**; [FN-2] 1093-1100, whose blanket rejection of a region-bearing generic
argument narrows to loan-bearing and provider arguments; [GRAM-2] 168-202's `struct_decl`
and `enum_decl`, which gain `region_params?`; [FN-1] 1005-1091 at 1005-1012. *Depends:*
[OWN-3] 580. *Verified today:* probe `f7_regionresult` is [FN-2]
`RegionBearingGenericArgument`, probes `r2_6` and `m05` are [GRAM-2] parse errors at
`struct Wrap['p]`, probe `q8` compiles the `&uniq` container parameter this clause refuses,
and probe `gen3` compiles the generic wrapper its third clause refuses. *Law:* L10, L11,
L13. *History:* r7 F1-8; r6 F1-1, F1-2.

*[CNT-1] through [CNT-7] and [SEQ-0] are deleted.* **[CNT-7]'s effect is restored by
[BLK-4]'s fourth clause**; its id stays retired and is not reused.

#### 3.K.4 `[VIEW]`: views and loans

**[VIEW-1] The two views.**

```text
| type              | reads | writes elements | changes length     | loan      | class  |
|-------------------|-------|-----------------|--------------------|-----------|--------|
| Slice<'r, T>      | yes   | no              | no                 | shared    | copy   |
| MutSlice<'r, T>   | yes   | yes             | no                 | exclusive | affine |
```

`Slice<'r, T>` is v0.41's `slice` under [S35]'s capitalization; its semantics are
Rust's slice's, which is why nothing but the case changes. `MutSlice<'r, T>` **[S6, S35]** is the one added view, because
[SET-1] 488-490 makes every slice-rooted target unwritable and probe `p7` is the refusal.
Each is an `own` value carrying a region `'r`, each is loan-bearing [PROV-3], and its
measures are [MSR-1]'s rows with `head` exact at `Z` because a view is formed only over an
unwrapped window [VIEW-2].

**The shared view is `copy` and the writable one is affine** (owner-decided, [S27]).
Affinity on the shared view buys no safety — a second copy is a second **shared** loan,
which [OWN-5] admits without limit, and a value that owns nothing has nothing to
double-free — and costs a re-formation at every second use (probes `s4`, `s5`).
`MutSlice` stays affine because [OWN-5] 606 refuses two exclusive loans on one range.
**What the decision also costs is now stated where round 7 found it**: a copy view is
never consumed, so its loan ends at its last use [PROV-3], and [LIV-2]'s copy case would
admit a `set` at a view target with no consume, which [VIEW-4] now refuses.

Three consequences rules read. A `Slice` operand is used **without `move`** [OWN-1] 564,
so `collect(out: move buf, source: line)` is the call spelling and a `move` is
`[OWN-1] MoveOfCopy` (probe `x14`). A `Slice` is never linear, never released and never
destructured. And an exclusive view and a shared read of one run cannot both be live,
which is [OWN-5]'s ordinary conflict (probe `s6`) and which Q19 records as the cost it is.

*Judgment:* the [OWN-1] classification of the two view types, which [PROV-3] use 1,
[LIV-2] condition 1, [VIEW-4] and [CALL-3] read. *Publishes:* the two types, their loan
strengths, their ownership classes, and the loan-bearing predicate. *Amends:* [TYPE-2]
357-360, [OWN-1] 563-571 at 563-564, which gains `MutSlice` as affine and **moves `Slice`
to copy**, and [CONST-2] 546-559, [OP-7] 939-947 and [OP-1] 771-849's `slice_of` row.
*Law:* L10. *History:* r7 F1-6, F1-12, F5-7; r6 the owner's [S27].

**[VIEW-2] Formation, the loan the view value holds, and the non-wrap premise.**

```text
seq_slice['r, T](vector: &'r V)          -> own Slice<'r, T>      reads(vector)   // [S10]
    requires head(vector) + len(vector) <= cap(vector)
seq_mut_slice['r, T](vector: &uniq 'r V) -> own MutSlice<'r, T>  reads(vector)
    requires head(vector) + len(vector) <= cap(vector)
```

**The view value, not the argument borrow, holds the loan**, and its extent is
[PROV-3]'s. The argument borrow is a call-scoped temporary [OWN-6] 614, which probes
`f2b`, `r1_twouniq` and `w8` confirm by accepting two of them on one place in one region
with an ordinary write between; it could not be the freeze.

**The `requires` is the window's one visible cost**, stated over the property a
contiguous view needs: a view is one contiguous range and a wrapped window is two. Three
things then hold: every formation row publishes `head = Z` and every back operation
preserves it; **an empty run satisfies it from the standing `head <= cap` alone**, so a
drained ring is viewable; and a wrapped run is returned to the premise by 3.L.8's drain.
**And the premise crosses a contract**, which is [CALL-7]: the chain of exact equalities
is exact inside one function and a loop backedge removes it ([ENT-5] 2942-2946), so a
caller of `filled::<u8, 4096>()` knows `head(input)` only because `filled`'s contract
publishes it and 3.L.3's `flat` invariant establishes it.

*Judgment:* [OWN-5] at the formation borrow, [MSR-4] discharge of the non-wrap
requirement, and the ordinary [BLK-0] relation establishment through [CALL-6].
*Publishes:* the two formation rows' relations, through [CALL-6]. *Amends:* nothing beyond [PROV-3]'s
amendment of [OWN-5]. *Depends:* [OWN-5] 606; [OWN-6] 614; [PROV-3], which fixes the
loan's extent. *Law:* L10, L15. *History:* r7 F1-12; r6 F1-5, F2-15, F4-1.

*[VIEW-3] and [VIEW-5] are deleted* with `AppendView`; their ids are not reused.

**[VIEW-4] A commit may not displace a live loan.** A commit that displaces a value of
loan-bearing type is admitted exactly when the displaced value is **consumed by that same
statement's right-hand side**. Two forms are therefore refused:

> `let old = replace p = e;` where `p`'s type is loan-bearing is a hard error citing
> VIEW-4 at the complete target `place`, because the displaced view survives as `old` and
> its loan would outlive the descriptor whose place it was held from.
>
> **`set p = e;` where `p`'s type is loan-bearing is a hard error on the same terms.**
> [LIV-2] condition 1's third disjunct makes a **copy** target dead at the commit with
> nothing consumed, and [S27] made `Slice<'r, T>` copy — so the seventh draft's stated
> ground for not reaching a `set` ("which for an affine target means its previous value
> was consumed") is exactly false at the one target type that matters. The mechanical fix
> is `bind a new view under a new let`.

Round 7 built the program: `set big = pick(a: big, b: small);` over two same-region shared
views inside one function, with `pick`'s [CALL-7]-mandated `len(chosen) <= len(b)` landing
on the same term as a surviving `len(big) = 4096` — a contradiction, [MSR-4] step 1, every
goal in the function provable. That is round 6's attack 1 with the `&uniq` parameter
removed, and [BLK-4] does not reach it because there is no parameter. `replace` and `set`
at a non-loan-bearing place are untouched.

*Judgment:* the two refusals, which are the only facts this rule states. *Publishes:*
nothing. *Amends:* [SET-2] 513-528's admitted commits and [SET-1] 481-511's, beyond
[PROV-3]'s amendment of them. *Depends:* [PROV-3] use 3, whose storage-keyed sentence is
why this rule is about the descriptor; [LIV-2]'s footprint sentence at a loan-bearing
target, which makes the kill fire when a commit is admitted at all; [VIEW-1]'s copy
classification, which is why the second clause is necessary. *Law:* L10, L11. *History:*
r7 F1-6, F5-7; r6 F1-1.

**[VIEW-6] Views are never stored, and a view result declares its origin.** A view is
never stored [BLK-4] and never returned except under this rule. [FN-1] 1023-1036's
slice-result ceiling applies unchanged to each view type: a function whose written result
is `own Slice<'r, T>` (respectively `MutSlice`) has the ceiling containing
`immutable-const` and the formal-view origin of every parameter whose written mode and
type are exactly that same view type with the same formal region and element type.

**An ordered result list containing two results of the same view type and the same formal
region is a hard error citing VIEW-6 at the `result_binding` of the second**, with the
restructuring `give each result its own formal region`; without it a three-output demux
written with one region returns three views each aliasing all three inputs.

**One real restriction is recorded, and it is narrower than the seventh draft's two.**
[FN-1]'s containment check forbids a helper from manufacturing a view of storage it
reaches through a borrow, so [VIEW-2]'s two formers are usable only in the function that
directly owns the run: **no helper library forms a view over a run it does not own**. What
the seventh draft recorded beside it, that a helper handed `&uniq MutSlice<u8>` can fill
its destination and cannot publish it, is **no longer a restriction of this design**.

**A helper handed a view may reborrow it shared, so the fill-and-publish helper is
writable** (owner-decided 2026-09-04, [S31]). Forming a shared `Slice<'r, T>` from a
`MutSlice<'r, T>` is the ordinary **shared child reborrow of a unique loan** that
[OWN-6] 613-627 already admits for places, applied to a view rather than to a place; a
probe on the v0.42 build accepts `peek(x: &deref(x))` inside a region block where
`x: &uniq u64`. The child `Slice` carries the parent's **origin set and range**, its loan
is a shared child of the parent's exclusive one, **the parent may not be written while the
child lives**, and the parent resumes where the child's own liveness ends [PROV-3]. So a
helper whose parameter is `destination: &uniq MutSlice<'r, u8>` fills that destination,
forms the child, and hands it to `write_once`; the child's origin is the parameter's
formal-view origin, which is inside [FN-1]'s ceiling, so the child may also be that
helper's result. **No row is added for it**: `seq_reslice` is not adopted, because a
kernel operation for a reborrow the language already performs is a second spelling for one
semantics. A release is not confined this way, because [PROV-6]'s walk compares types and
not places.

*Judgment:* [FN-1]'s ceiling containment at every `return_stmt`, the same-region result
rejection, and [OWN-6]'s child reborrow admission with a view as the parent. *Publishes:*
the result's origin set, and the child loan's strength and range. *Amends:* [FN-1]
1023-1036, by generalizing "slice" to "view" and by adding the same-region rejection.
*Depends:* [OWN-6] 613-627's shared child reborrow, which is the formation this rule
reads; [PROV-3], which fixes the child loan's extent. *Law:* L10, L11. *History:* r7
F4-5; r1 F4-7; the owner's [S31].

**[VIEW-7] System operations over views.** **[S30], ADOPTED.** The seven range-bearing
operations [SYS-8] 2488-2527 take views instead of `buffer<u8>`, with fixed modes:

```text
a destination the operation writes  ->  &uniq 'd MutSlice<'r, u8>
a source the operation reads        ->  &'s Slice<'r, u8>
```

so `read_at(file: &ReadFile, destination: &uniq MutSlice<u8>, file_offset: own u64,
start: own u64, end: own u64) -> result: own ReadOutcome`, whose three regions relate
nothing and are all elided. Both are borrows of the **descriptor**, so the view survives
the call and a destination can be filled by a loop of reads, which an `own` destination
could not; both write element storage only, so [CALL-3] gives the caller its measures
back. **The two range obligations keep their form and their order, each stated over the
operation's own range-bearing parameter** — `len(deref(destination))` for `read_at` and
its siblings, `len(deref(source))` for `write_once`, `host_copy_bytes` and
`host_copy_utf8` — which is round 7's correction of a sentence that named the destination
for all seven.

**[BLK-4]'s fourth clause does not reach these**, by the clause's own scope: a [SYS-2]
declaration record's behaviour is fixed by its record and it has no body in which an
unnamed point could exist. This is the change that lets a heap-free program do I/O. Its
cost is that a destination must be **addressable** first, so it is built by 3.L.3's
`filled` and the count the host produced is an ordinary `u64` beside the run; Q7 records
the fix.

*Judgment:* [SYS-8]'s two range obligations, restated over `len` of the borrowed
range-bearing view. *Publishes:* the endpoint facts [ENT-3.S10] enumerates, now over a
view. *Amends:* [SYS-8] 2488-2527, [SYS-2] 2164-2307's declaration records and normative
counts, and the prose of [SYS-9] 2529-2552, [SYS-11] 2576-2585, [SYS-12] 2587-2603 and
[SYS-14] 2615-2644. *Depends:* [EFF-1] 1386 as [PROV-3] amends it, which is the judgment
[CALL-3] reads. *Law:* L11. *History:* r7 F3-I8, the owner's [S30]; r4 F1-a9.

#### 3.K.5 `[LIV]`: liveness and the one commit rule

**[LIV-1] Liveness is join-checked, and that is what makes release unconditional.** A
binding's live-or-dead status is a property of a program point, not of a path: at every
join of the conservative structural graph [FN-1] and at every loop head, every predecessor
must agree on the status of every binding in scope. A disagreement is a hard error citing
LIV-1 at the join, naming the two predecessors and the binding.

**On every edge leaving a scope — a `propagate` error edge, a `break`, a `give` and the
function-return edge included — every binding of that scope that is linear in that scope
[PROV-6] must be dead**, because in that scope no derived release exists to carry it; and
**every other binding takes its compiler-derived release on that edge**, unconditionally.
Under D3 the second half is what a hosted program lives on: the scope holds the capability
by signature, so the release is derived on the edge it belongs to.

**This rule states its own two amendments.** [OWN-11] 646's "a binding declared outside
that body may not be moved inside it" is **replaced** by the join agreement: the
prohibition exists because a moved-and-not-restored binding makes the loop head disagree
with the preheader, and the join check decides exactly that. And [OWN-1] 566-567's
recheck sentence is **replaced** by [LIV-2]'s own commit premise, stated over the state at
the commit.

*Judgment:* a per-join and per-scope-exit structural check over the ownership state the
checker already computes; no search. This is the judgment [PROV-6]'s scope-exit refusal,
[STK-1]'s tail premise and [STK-4]'s unreachable-exit sentence read. *Publishes:* the
unconditional release set of every edge. *Amends:* [OWN-1] 563-571 at 563 and 566-567, and
[OWN-11] 646-648 at 646. *Depends:* [PROV-6]'s per-scope linear predicate. *Law:* L17.
*History:* r7 F4-1, F5-9, F5-15; r6 F3-I9; r1 F1-1, F1-2.

**[LIV-2] One `set` commit rule.** **Owner-decided 2026-09-03 (D2).** One statement
writes places, and it replaces three: [SET-1]'s copy overwrite, the sixth draft's
reinitializing `set` at a dead binding, and its in-place exchange. `[LIV-3]` is retired
into this rule and its id is not reused.

```wf-design
set p = e;
set p = f(vector: move p, value: byte);
set (p, taken) = seq_take(vector: move p);
set (p, q) = move q, move p;
set (a, b, c) = move c, move a, move b;
```

**The form.** `set (p1, ..., pn) = rhs;` for `n >= 1`, parentheses omitted at `n = 1`. The
right-hand side is either **one call with `n` results** or a **value list of `n`
expressions**, evaluated left to right. Each target is a `place` [GRAM-5] — a bare
binding, a field selection, a `deref`, or a subscript — or an identifier that resolves to
no binding in scope, which introduces one exactly as a `let` does.

**The commit, stated as a read-out.** Round 7 found the seventh draft's commit paragraph
("through that evaluation every target is dead") contradicting its own first admission
condition ("the right-hand side consumed its previous value, `move p` occurring in it"),
so the required `move p` was a use of a dead binding and the rule had **no admissible
instance**. The paragraph is [SET-2]'s:

> **Every target place is resolved once, before the right-hand side is evaluated, and
> the resolution is not re-taken at the commit. Each target's previous value is read out
> of `resolved(p)` at the start of that evaluation, and the target is dead for the
> remainder of it. Then all targets are reinitialised at one commit**, in declared order,
> each from its own ordinal's value. There is no writer-observable program point between
> the read-out and the commit (spec 520), so there is no partial move, no dead root and no
> uninitialized hole, and every target is live afterwards.
>
> **A `move` of a target place, or of a sub-place of a target place, occurring in that
> statement's right-hand side, is that target's read-out** and is not [OWN-1] 569's
> root-killing partial consume; [PROV-6]'s partial-consume refusal names the same
> exception from the other side.

**The three admission conditions, and no fourth.**

1. **Every target is dead at the commit** — when the right-hand side consumed its
   previous value, when the target was already dead, or when the target's type is copy. A
   **live affine target whose previous value the right-hand side does not consume** is
   [STOR-1] 679's error, kept for exactly the case it was written for, with the
   restructuring `use replace`.
2. **The targets are pairwise non-overlapping places.** A place and **any place reached
   through it** (`p` and `p.f`, `v` and `v[i]`, `grid[k]` and `grid[i][j]`), and two
   subscripts of one run, are refused, because the commit order would decide the result;
   a hard error citing LIV-2 at the second target.
3. **Arity and type.** The right-hand side supplies exactly `n` values and each ordinal's
   type is exactly its target's type [TYPE-5] 379.

**What falls out, rather than being asked for.** `set p = f(vector: move p, ...)` is the
transformation at a bare binding, a field, a `deref` or a subscript alike; `set p = e;` at
a dead `p` is the reinitialization; `set v[i] = 7_u8;` is v0.41's own `set`; and
`set (p, q) = move q, move p;` is a swap, with three targets rotating.

**Its judgment is [SET-2]'s, not [SET-1]'s, and that is what makes it not sugar.** At a
bare binding a writer could sometimes rebind in two statements; at every other place they
cannot, because `move p[i]` and `move p.f` are partial moves that kill the root and
`move deref(handle)` is forbidden by [OWN-5] 591 with [SET-2]'s exchange as the sole
exception. **And the two-statement form is not equivalent even at a bare binding**:
`let next = ...; set p = move next;` is a move-rebind, and without [MSR-3]'s rebind
placement it destroys every measure fact about the run (probe `q12`).

**Its effect footprint, and what "ultimate storage origin" means at each target shape.**
The statement exhibits one read and one write of each target's ultimate storage origin,
and the right-hand side's own projected row in addition. Round 7 found that phrase
undefined at the two shapes its own repairs created:

> For a **bare binding, a field or a `deref`**, the ultimate storage origin is that
> place's own storage. For a **subscript** `P[i]`, it is the descriptor storage of `P[i]`
> and none of `P`'s own — the granularity [MSR-2] states over storage. For a place whose
> type is **loan-bearing**, it is that target's **descriptor** storage and never the viewed
> backing: [EFF-1] 1386 as [PROV-3] amends it governs an effect **path** through a view
> **parameter**, not a target place. That is what makes [MSR-2]'s kill fire at a view
> target, which is half of [VIEW-4]'s repair.

Deriving a field-precise footprint from a callee's row would be wrong, because the value
written back is a whole new value of the target's type.

**Term identity, in one sentence** [MSR-3]. A target that names a binding in scope
**keeps that binding's [ENT-2] term**; a target that resolves to no binding introduces one
and is a declaration event. Probe `r4` is the [TYPE-6] `DeclarationCollision` the seventh
draft's per-statement rule produced and D2's per-target resolution removes.

*Judgment:* the read-out, the three admission conditions and the commit, each a hard error
citing LIV-2 at the target or statement named there; this is the judgment [MSR-2]'s kill,
[MSR-3]'s atom identity, [PAR-1]'s footprint and [CALL-4]'s destination clause read.
*Publishes:* the right-hand side's declared relations on each target through [CALL-6], the
statement's read and write of each target's ultimate storage origin as defined above, and
the term-identity rule. *Amends:* [STOR-1] 675-683 (the writable-place partition at
678-679, with 679's diagnostic kept); [SET-1] 481-511, which becomes this rule's `n = 1`,
copy-target case; [SET-2] 513-528, whose "it establishes no fact" at 528 becomes false for
this form, whose target may be linear or region-bearing because nothing is rebound, and
whose exchange exception to [OWN-5] 591 this rule inherits; [GRAM-4] 217-256's `set_stmt`;
[OWN-1] 563-571 at 566-567; [ENT-2] 2677-2728 at 2683; and [ENT-3.S12] 2822-2837's
destination list through [CALL-4]. *Depends:* [PROV-6]'s partial-consume judgment, whose
reinitialisation exception is the other side of the read-out sentence; [OWN-5] 591.
*Verified today:* probes `q9`, `x5`, `t8`, `x2` and `x3` are [STOR-1] `AffineSetTarget`,
probe `g5` is the same at a field, probe `p10` is `AffineSetTarget` at a live target and
probe `w6` is [OWN-1] `UseAfterMove` at a dead one, and probe `w8` accepts a `set` at a
`match` arm binder. *Law:* L10, L16, L17, L18. *History:* r7 F1-9, F1-15, F3-3, F4-8; r6
the owner's D2, F3-1, F3-3.

#### 3.K.6 `[CALL]`: what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the callee's
declared parameter modes and types and its declared contract.** Three rules beside them
make the transports usable: [CALL-4] is the vocabulary a contract may be written in,
[CALL-6] is how a declared relation becomes a fact and where, and [CALL-7] is the
obligation that a contract be complete about what it handed back.

**[CALL-1] Through a shared borrow, every fact survives.** For an argument whose
parameter mode is `&'r`, of any type, run and view included, the call is not a kill event
for any fact supported by the actual's resolved place. Ground: [OWN-5] admits no write
through a shared holder, so [EFF-2] can project no `writes` occurrence onto that place, so
[MSR-2]'s kill does not fire. **That ground is exactly as strong as the set of actions
classified as writes**, which is why [PROV-6] makes a release one: a callee that takes
`&Vector<'s, u8>` and releases its referent satisfies every clause of this rule while
falsifying its conclusion unless the release is a write.

*Judgment:* none; the absence of a kill, which is [MSR-2]'s judgment not firing.
*Publishes:* the survival of every such fact. *Amends:* nothing. *Depends:* [OWN-5]
585-611's shared-holder prohibition, the whole ground; [MSR-2]'s kill classification.
*Verified today* for `&'a buffer<u8>`: probe `p6` keeps `len(line) = 10` across the call.
*Law:* L11. *History:* r6 F1-2.

**[CALL-2] Through a value passed and returned, only the contract's facts exist on the
result.** An `own` argument is a consuming use, so every fact whose support contains that
binding's root dies. The result is a fresh binding carrying exactly the callee's verified
relations, and nothing else. Those relations may name the consumed parameter's measure,
which denotes that call's **call datum** [MSR-3]: `len(rest) = len(out) + 1` means what it
reads as, and it is establishable at the caller precisely because a datum has empty
support and the consume the same statement performs cannot kill it.

**Under R1 this is the transport a container helper uses**, and it is the reason R1 costs
the writer nothing: a helper's `ensures` names its own result and its own inputs, both of
which the caller can see. Probes `w2`, `w3` and `w5` chain published relations to depth
five with the goal asked once at the end; probe `w1` is the control, where three helpers
with no `ensures` fail at the **second** link.

*Judgment:* the ordinary [ENT-3.S12] establishment, subject to `M(c,q)` as [MSR-3] amends
it. *Publishes:* the callee's declared relations on the result, established by [CALL-6]. *Amends:* nothing beyond
[MSR-3]'s. *Verified today:* probe `p1` is **rejected** with residual `9_u64 < len(b)`;
the transport already behaves correctly and what was missing is the vocabulary to publish
across it. *Law:* L11. *History:* r6 F4-3.1.

**[CALL-3] A write through a view reaches the range's storage and no measure of the
origin place itself.** For an argument of **loan-bearing** type, own or behind a borrow, a
projected callee `writes` occurrence:

> kills every fact whose support overlaps the **viewed range's storage**, which for an
> element type that has descriptor storage of its own **includes that element's
> measures**; and kills **no measure term over the origin place itself**.

For every other parameter type the projected write kills measures as an ordinary
descriptor-storage-overlapping event [MSR-2].

**Round 7's fourth BREAK is why the classification is stated over storage.** The seventh
draft said the write "kills every fact whose support overlaps the viewed **element
storage** and kills no measure term over that origin". When `T` is itself measured —
`MutSlice<'r, Vector<u8>>` over a `FixedVector<Vector<u8>, 8>`, which [PROV-6]'s
ownership closure newly makes affine and therefore passable by value — the viewed element
storage **is** the descriptor storage of the origin's elements, so clause 1 killed exactly
what clause 2 preserved, and a callee could replace and free a caller's inner run while the
caller kept `len(origin[0])`. Stating it over storage makes `len(origin)` survive — all
this rule was ever for — and `len(origin[i])` die, which is correct because an
exclusive-strength view can replace an element descriptor. It is the same repair [MSR-2]
makes one rule over: **the descriptor/element split is a property of the element type, not
of the word "element".**

**Its premise is what a view can write, and three rules judge it**: [EFF-1] 1386 as
[PROV-3] amends it makes a view **parameter**'s effect path name the viewed backing;
[PROV-3] use 1 judges every access through a view at the range it reaches; and [SET-1] as
[PROV-3] amends it admits a target path through a view only at exclusive loan strength.
**What a caller learns about a view a callee handed *back* is [CALL-2]'s**: a returned
view carries exactly the callee's verified relations, so a helper returning the shorter of
two views tells its caller nothing. The danger was never a view a callee returns; it was
one a callee **installs**, and [BLK-4] refuses the parameter while [VIEW-4] refuses the
local statement.

*Judgment:* the kill classification per parameter type, which is [MSR-2]'s judgment
parameterized by [PROV-3]'s access classification. *Publishes:* the surviving measures.
*Amends:* nothing beyond [MSR-2]'s. *Depends:* [EFF-1] 1386 as [PROV-3] amends it;
[PROV-3] use 1 and [SET-1] as amended; [BLK-4]'s fourth clause, which is why the default
reaches no run. *Law:* L11. *History:* r7 F1-7; r6 F1-1.

**[CALL-4] Contract vocabulary, the ordered result list, the routes, and where the
relations land.** [FN-9]'s clause operands are terms [MSR-5], so the four measures over an
admitted formal place are operands with no per-family admission, and so are they over an
admitted **result** place, which today's result-datum restriction to fragment integers
forbids (probe `q7`).

```wf-design
fn collect['s](out: own Vector<'s, u8>, source: own Slice<u8>)
    -> (rest: own Vector<'s, u8>, written: own u64)
    reads(out, source), writes(out) contract {
  requires len(source) <= room(out);
  ensures len(rest) == len(out) + written;
  ensures room(rest) + written == room(out);
  ensures head(rest) <= 0_u64;
  ensures written == len(source);
} { ... }
```

The ordered result list is [S16] and the clause operands are [S17]. **No clause names two
states of one term, and under R1 none needs to**: a parameter is an input with one state,
a result is an output with one state, and a relation between them is single-state in both.
There is no `old()`, no frame rule and no entry/exit convention. This is also where L14's
retired guarantee comes back as `len(rest) >= len(out)`. **A function may declare an
ordered result tuple [S16]**, and each result binding is a datum of every clause.

**A relation is published per enum variant and per result ordinal, and a result datum
admits field projection [S24].** [FN-9] 1307 admits exactly `when Ok(value: r):` for
`own Result<T, E>` with `T` a fragment integer, and 1314 excludes a nested result
projection (probes `x1`, `x2`). The generalization is four sentences:

> A routed clause is admitted as `when b is V(f: r):` where `b` **names the result
> ordinal** the route applies to and `V` is any variant of that ordinal's enum type, with
> `r` that clause's fresh symbolic payload datum. The ordinal binder may be omitted
> exactly when one ordinal has that enum type. An unrouted clause is admitted for a
> written result of any **measured** type as well as any fragment integer. Every ordinal
> is a datum of every clause. The four measures are operands for an admitted place `P`
> formed from a result datum with field and `deref` projections, on exactly the terms
> [FN-9] 1313 already grants a parameter datum.

The ordinal binder is round 6's, on [VIEW-6]'s precedent: a function with two same-typed
enum results left a variant route ambiguous in three ways and one of them unsound.

**Those relations reach the caller through one added [ENT-3.S12] destination clause**,
without which a multi-result contract publishes nothing, because 2822-2837 fixes a closed
list of four and a destructuring `let`, a `set` target list and a `match` arm binder are
none of them:

> Each binder of a destructuring `let`, **each target of a [LIV-2] `set`**, **each arm
> binder of an own-place `match`**, and each binder of a destructuring consume [S13] is
> the S12 destination for every published relation naming the value that lands there,
> established as [CALL-6] states.

The `match` arm binder is the destination the whole allocating inventory depends on. The
same clause carries [MSR-3]'s rebind, payload and field placements, and [FN-9] 1357's
narrow direct-set route is subsumed.

*Judgment:* the ordinary [FN-8]/[FN-9] admission over the widened operand set, result
shape and route set; the ordinal-binder requirement, a hard error citing CALL-4 at the
clause when a variant route is ambiguous; and the S12 establishment at each of the four
destinations, which is the judgment [CALL-2], [LIV-2], [MSR-3] and [PROV-6] read.
*Publishes:* the clause relations, established by [CALL-6], on every result ordinal and every admitted variant
route, at each of the four destinations. *Amends:* [FN-9] 1301-1365, [ENT-3.S12]
2822-2837's destination list, [GRAM-2] 168-202's `fn_decl` result shape, [GRAM-4]
217-256's `let_stmt`, `set_stmt` and `return_stmt`, [FORM-2] 39-89's rendering, and
[FN-1] 1005-1091 at 1005-1019. *Depends:* [CALL-6], which fixes where each lands.
*Verified today:* probes `q7` and `x2`, `x1`, `q6`, `x13`; **the single-result vocabulary
landed**, conformance case `call4-neg-measured-result-not-admitted` (6.0). *Law:* L10,
L11, L16. *History:* r6 F1-9, F4-12; r5 F4-2, F3-3; B1 (three routes deferred).

> **Correction, decided 2026-09-04, from B1's implementation.** Three of this rule's
> admissions need machinery B1 does not have and **land in B7**, not here: a result of
> **measured** type and a measure over a result place, each of which needs the result
> binder to be a place-like datum rather than a fragment integer, and a route over **any**
> variant of any returned enum, which needs resolver identity for variants beyond the
> prelude `Ok`/`value`. `call4-neg-measured-result-not-admitted` pins the measured-result
> refusal as a semantic admission refusal at [FN-9] and no longer a [GRAM-5] parse
> rejection, which is the state the widening leaves it in. **[S16]'s ordered result list
> did not land either** and goes to §7's B1b with the destinations that read it.

> **Correction, decided 2026-09-04, from B1b's implementation.** [S16]'s ordered result
> list, the destructuring `let` binder list, the `set` target list, the multi-expression
> `return`, the result ordinal, the ordinal-named route `when b is V(f: r):`, its
> omitted-binder condition and its ambiguity refusal **landed in v0.45**, and so did two
> of this rule's three added destinations: **each binder of a destructuring `let`** and
> **each target of a `set` target list**. The third, **the arm binder of an own-place
> `match`**, did **not**, beyond the direct-scrutinee route [FN-9] already had, and it is
> not reachable by a multi-result call in any case: a call that hands back two or more
> ordinals is no `match` scrutinee, so reaching an arm binder needs the relation to
> survive the destructuring binder that names the ordinal first. That is a pending
> summary token across a naming event — [MSR-3]'s deferred rebind and binder placement —
> rather than a destination of its own, and it goes with B7's measured result. The
> specification records it as this rule's DEFERRED clause.
>
> Two implementation limits are compiler capability and not language: a **borrow-mode or
> `slice` ordinal** of a result list is refused as an unimplemented capability, because
> [FN-1]'s return-origin ceiling and borrow-result provenance are not derived per ordinal
> yet; and a **subscript target in a `set` target list** is refused for the same reason,
> because one statement's several indexed commits need [SET-1]'s offset-evaluation order
> stated over a list. Neither is written into the language.

**[CALL-6] Publication: how a declared relation becomes a fact, where it is computed,
and where it is established.** This is the rule round 6 found missing and round 7 found
computing at one point and establishing at another. Every `Publishes:` line in 3.K names
this rule or [FN-9]'s existing [ENT-3.S12] route, and nothing else publishes anything.

**[ENT-3] gains one enumerated source, `S13`.**

> **S13 (call datums, and declaration-domain relations).** At an ordinary source call
> whose callee has an atomically published summary, and at an admitted [BLK-0] or [SYS-2]
> call, each declared relation of the resolved callee or row is **instantiated at the
> call**, by substituting each operand at the denotation [MSR-3]'s table gives its
> parameter's **mode** — an `own` formal by that actual's call datum, a shared-borrow
> formal by the live term, a `&uniq` state formal by that place's post-state, a result
> binder by its destination below — and its **support** is the ordinary L0 support of
> those substituted terms, taken at the call.
>
> It is **established** on the call's normal continuation, after the call's ordinary
> transfer, consumes, borrow commits, target commit and kills, exactly in [ENT-5]
> 2898-2905's order. **A relation routed to a variant is established there too and is
> restricted to that variant's arm**: it is available exactly on the paths on which that
> arm is entered, and it is not deferred to the arm.
>
> **A relation over a post-state place is killed by any [ENT-5] event writing that place
> at or after the call**, whether that event lies before or after the arm on which the
> relation is available; a relation whose support is dead is not available at all. A
> relation over a call datum has empty support and no event kills it.

> **Correction, decided 2026-09-04, from B1's implementation.** The eighth draft stated
> S13's population as the declared relations of an admitted [BLK-0] or [SYS-2] call.
> **No compiler-owned row carries a declared relation set in v0.44**, so that population
> is empty at the tip and a source stated only over it would establish nothing. What
> landed is the substitution half, over the population the language already has: at an
> ordinary source call whose callee has an atomically published summary, each `own`
> operand of each declared relation of the resolved callee mints one call datum [MSR-3]
> and is established equal to that operand's exact pre-transfer term, at the call's
> pre-transfer point and before that boundary's consumes, borrow commits, callee-effect
> kills and target kills. The two halves are one source because the denotation table is
> one table. **B7 extends S13's population to [BLK-0] rows** when those rows exist; it
> does not take the label, and the label is not reused.

**The establishment sentence is round 7's second BREAK.** The seventh draft deferred a
routed relation's establishment *to* the arm and killed it from the establishment point,
so every write between the call and the arm happened "before" it and killed nothing. Round
7 wrote the program: two checked `seq_arena` takes from one frame arena, the second
unmatched, then a `match` on the first whose `Some` arm re-establishes `len(scratch) <=
256` after the second take advanced the cursor — and a `seq_arena_proved` on that arm
discharges `room(scratch) >= 65008` and hands back a run running 64728 bytes past the
extent, in a `pure`, heap-free, `resource_closed` program [RES-3] accepts. Instantiating
at the call and **restricting** rather than **deferring** is the repair, and it is the
mechanism [ENT-5] already has for a branch-conditioned fact.

**The destinations, in one list.** [ENT-3.S12] 2822-2837's closed list of four gains:

```text
a result binder of a destructuring `let`                       [CALL-4]
each target of a [LIV-2] `set`                                 [CALL-4], [LIV-2]
each arm binder of an own-place `match`                        [CALL-4], [OWN-13]
each field binder of a destructuring consume                   [CALL-4], [S13]
the resolved place of a `&uniq` state actual, for a relation
  over that state parameter's measures                         this rule
```

**The last destination is the one a provider needs.** A refused `seq_arena` publishes
`room(arena) < advance<T>(count)` and a successful one publishes the cursor's new value;
[RES-6] requires the first, L8's second half rests on it, [RES-10] reads the second, and
none contains a result datum, so [FN-9] 1313 admits none and [ENT-3.S12]'s four
destinations all key on a result.

> A relation all of whose operands are measure terms over a **`&uniq` state parameter of
> a declaration-domain row**, together with constants, that row's own call datums, and the
> compiler-owned formers A.1 defines, is admitted without a result datum and is
> established on the actual's resolved place. **No other relation may omit the result
> datum**, and **no relation a source-declared `fn` writes may be established on the
> resolved place of a borrow actual**.

The last clause is stated over a **borrow actual** and not "a caller's place at all",
which is round 7's correction: a [LIV-2] `set` target *is* a caller's place and [CALL-4]'s
destination clause needs it, so the seventh draft's wording deleted the destination on
which 4.2's central statement and every loop in 3.L depend. The two sentences are one
boundary read from two sides: a compiler-owned row is a declaration record whose relations
are complete over everything it writes [BLK-0]; a wf body is a body, so a caller reading
its post-state would be reading a claim about an object at a point the callee cannot name
(L11). The cost is [PROV-2]'s and is Q17.

**And a `replace` publishes nothing.** [SET-2] 528 says its commit "establishes no fact"
and this rule keeps that true: a value whose measures must survive is **constructed into
its owner** [MSR-3], not replaced into it. **A value obtained by `replace` therefore
carries no measures**, and a function returning one either publishes them from its own
body or is refused by [CALL-7]; 3.L.2's `take_at` is the worked case.

**Every published relation set is checked for consistency.** A row or contract whose
instantiated relations are contradictory at the establishment point is a hard error citing
CALL-6 at the row or the `fn_decl`, because [MSR-4] step 1 discharges every goal from a
contradiction — which is how three of round 7's four memory BREAKS reached memory.

*Judgment:* the S13 instantiation at the call, the establishment and restriction, the kill
from the call, the admission test on a relation that omits the result datum, and the
consistency check, each a hard error citing CALL-6 at the row or clause. This is the
judgment [BLK-0], [BLK-2], [BLK-3], [PROV-2], [PROV-6], [RES-6] and [RES-10] read
wherever they publish or consume a post-state. *Publishes:* the source, the substitution,
the instantiation point, the establishment point, the destination list and the support of
every declared relation in the language. *Amends:* [ENT-3] 2730-2837, which gains S13;
[ENT-3.S12] 2822-2837's closed destination list; [FN-9] 1301-1365 at 1313, whose
result-datum requirement is lifted for exactly the relations named above. *Depends:*
[ENT-5] 2898-2905, whose establishment order this source reuses verbatim; [SET-2] 528;
[MSR-3], which supplies the call datum and the denotation of every operand. *Verified
today:* **the source-call half landed**, conformance cases
`call6-pos-routed-relation-over-a-call-datum` and
`call6-neg-contradictory-published-relations` (6.0). *Law:* L11, L15, L16. *History:* r7
F1-2, F1-14, F3-I14; r6 F3-1, F3-2, F3-5; B1 (S13's population).

**[CALL-5] No transport reads the actual's spelling.** The three transports are selected
by the callee's declared parameter mode and type and by its declared contract. No rule of
this design consults the argument expression's shape, the callee's body, its name, or any
per-parameter summary derived from its body. A parameter type for which no transport is
selected kills conservatively. **Two rules of this design tested that sentence and both
satisfy it**: [RES-8]'s saturation fact is a **declared** clause, and [CALL-7]'s
completeness obligation is a **declaration-site** check of a written contract against a
body, exactly as [EFF-2] 1432 checks an effect row.

*Judgment:* the conservative default for every unselected parameter type. *Publishes:*
the absence of a call-site-derived fact. *Amends:* [ENT-5] 2863-2967's clause (b) at 2876,
whose projected-callee-write kill is now classified by [CALL-1..3] and by nothing else.
*Law:* L11. *History:* r6 F2-9.

**[CALL-7] A hand-back contract is complete, and the obligation is decidable.** L15's
completeness sentence, over the population [BLK-0]'s cannot reach.

> A source-declared `fn` whose result list contains a result of **measured** type, or a
> measured place reachable from a result by field selection whose descriptor storage the
> body wrote, where that value was **constructed** by the function or **received as an
> `own` parameter and returned**, must, **for each measure of that result on each admitted
> route**, state at least one clause one of whose two sides is that measure and whose
> other side is a term over the function's inputs, the constants, or another measure of
> that result.
>
> **Three exclusions, each decidable from the declared type and none from the body.** A
> measure that is a **standing fact of the result's own type** needs no clause: `cap` of a
> `FixedVector<T, n>` is the type constant `n`, which [MSR-2] already makes an
> empty-support fact the caller has too. A clause **both of whose sides follow from
> [MSR-2]'s implicit facts alone** does not satisfy the obligation. And a result of
> loan-bearing type is outside the population, because [VIEW-2] fixes its measures at
> formation. **No exclusion reads the body** ([CALL-5]): a measure the body leaves at its
> standing bound — a `head` after a front operation is the only one — has no non-vacuous
> clause and therefore no admissible signature, so the function returns the drained run's
> `len` and lets the run itself die rather than handing back a value no caller can use
> (3.L.8 is the worked case).
>
> A measure with no such clause is a hard error citing CALL-7 at the `fn_decl`,
> `IncompleteHandBackContract`, naming the result, the measure and the invariant that
> would carry it. Whether a stated clause **holds** is the ordinary [MSR-4] question,
> checked by the both-ways discipline [EFF-2] 1432 applies to an effect row.

**Round 7 found the seventh draft's version vacuously satisfiable and undecidable, and
those are one defect.** It required "the exact value or relation to the corresponding
input measure **where the body establishes one**, and a two-sided bound where it does not"
— deciding which form is demanded is deciding whether the body establishes an exact value,
which no test defines; and the only enforceable half was *mention every measure*, which
`ensures head(result) <= cap(result);` satisfies with a standing fact, clearing the
diagnostic and leaving [VIEW-2]'s premise as undischarged as the sixth draft left it. The
shape above is a syntactic condition plus one [MSR-4] query per stated clause: decidable
by counting, refusing the standing-fact clause by name, admitting
`ensures head(result) <= 0_u64;`.

**The "merely forwarded from a callee" exemption is deleted.** It named a transport
[CALL-2] and [CALL-5] forbid: `build`'s inner callee's relations are facts inside
`build`'s body with no rule putting them on `build`'s signature. Either the function
declares the measure or the caller does not get it.

**Why a declaration-site obligation rather than a derived summary.** A derived
publication would be a body summary a caller reads, which [CALL-5] forbids and which is
the shape of D1's own flag. **Why the measure population and not every result.** A measure
is the one class of fact whose absence silently deletes a caller's ability to *use* the
value it was handed. **What it costs, measured.** Round 7 counted contract clauses and the
header invariants that establish them at about 120 of 190 library items across sixteen
programs — a price worth paying rather than going back. The exclusions above remove `cap`
from every `FixedVector` result, which is `vacant` and `filled` from four clauses to three
and `pool_new` from four to three. Inside a construction loop the remaining cost is one
header invariant per exactly-published measure, because [INV-1] 3105 admits four ordered
relations and not `==`; Q14 records the change that halves it and recommends landing it
with this rule. And round 7 found **five of the seven functions 3.L prints violating this
rule**; 3.L carries the missing clauses and 6.11 lists them.

*Judgment:* the syntactic per-measure, per-route clause requirement and the both-ways
check of each stated clause against the body, citing CALL-7 at the `fn_decl` — which is
the judgment [VIEW-2]'s premise, [CALL-2]'s transport and every loop in 3.L read at a call
site. *Publishes:* every measure of every handed-back result, as ordinary [FN-9] clauses that [CALL-6] establishes at a caller.
*Amends:* [FN-9] 1301-1365's clause list, and [FN-1] 1005-1091 at 1005-1012. *Depends:*
[EFF-2] 1432's both-ways discipline; [MSR-2]'s standing facts, over which the exclusions
and the vacuity test are stated; [MSR-5], which is why an exact relation is one clause.
*Law:* L11, L15. *History:* r7 F1-10, F1-11, F3-5, F3-6, F4-4, F4-6; r6 F4-1.

#### 3.K.7 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for exactly
this set:

```text
| class              | members                                                                        |
|--------------------|--------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context, including every       |
|                    | frame-placed arena [PROV-5] and the release walk's own straight-line frame       |
|                    | cost; every extent-placed arena; the adapter's persistent mappings               |
| execution stacks   | one chain per execution context the artifact holds live: the entry context, each |
|                    | worker lane, the host thread whose stack survives an entry the floor created,    |
|                    | and **each alternate stack**, which is a stack and whose handler chain [STK-3]   |
|                    | measures by its own formula                                                      |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots;  |
|                    | the runtime's fixed handle table; every other runtime-owned store                |
| host objects       | every countable host object a qualified runtime holds for the program's          |
|                    | duration: a ring file descriptor, a device handle                                |
```

The second class is round 7's. The seventh draft filed the guard-page floor's alternate
stack as a `region` item, which gives it bytes and no **chain**: the handler's report path
runs *on* it, no `Stack(f)` computation covered that chain, and an overflow inside the
overflow reporter ended the process with no Whitefoot value — inside the mechanism
[RES-4] added to make stack exhaustion reportable. The third and fourth classes are drawn
at *countable versus extent*.

*Judgment:* none; it fixes the domains [RES-3] quantifies over. *Publishes:* the covered
set. *Amends:* nothing. *Law:* L1, L5, L6. *History:* r7 F2-12; r6 F2-12.

**[RES-2] The envelope `E`, over the target's profile table.** `E = E(P, T, B)` is, for
one program `P`, one selected target and ABI `T` [STOR-6] 738-767, **and one build `B`**,
a finite table with one row per lane count `W` the target's runtime supports. Each row is
a finite list of shaped items:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count, bytes, alignment)         interchangeable fixed-size records
stack(context, bytes, alignment)             one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
handle(kind, count)                          countable host objects the runtime holds
```

**A `slots` item carries its member's size and alignment**, which is L6's own sentence
applied where round 7 found it unapplied: `slots(task.records, 64)` gave a deployment a
count with nothing to multiply. A `stack` item's `alignment` is the **stack base**
alignment the ABI requires; a frame-placed arena's alignment slack is inside `frame(f)`
and [STK-3] says so. **A `region` item's name is the pair (concrete function instance,
`region_stmt` NodePath)** [PROV-5], so `E`'s item count is a function of the **expanded**
program. **`E` is a function of three things and not two**, because [STK-3] makes it an
output of code generation; **and `E` carries the content digest of the artifact it
describes**, because a table from an earlier build otherwise satisfies every check
[RUN-4] performs while describing a different artifact (Q9 records the residue).

**Which items carry a source-stage figure is stated rather than quantified over.** A
`region` item's bytes and alignment, a `slots` item's count, size and alignment and a
`handle` item's count are [RES-5]'s target-independent arithmetic and are read by
acceptance; each additionally carries the target-stage exact figure. A `stack` item has
**no source-stage figure at all**, so stage one's entire stack content is premise 2 of
[RES-3].

*Judgment:* `E` is well-formed only if every item's arithmetic was performed in the
unbounded mathematical domain and is representable on `T`, the standard [STOR-6] already
applies. *Publishes:* `E` itself, with its digest. *Amends:* nothing. *Law:* L1, L6.
*History:* r7 F2-5, F2-19; r6 F2-9.

**[RES-3] The judgment, in two stages.** For a program `P`,
`source-resource-closed(P)` holds exactly when, on the rewritten call graph [STK-1],
every premise below is established from program text alone:

```text
1  no reachable store is a Heap                                    [PROV-4, RES-4]
2  the call graph is acyclic                                       [STK-2]
3  every covered store's demand is bounded, per domain, by the
     symbolic composition of [RES-10]                              [RES-5, RES-10]
```

**Every quantity premise 3 tests is a compile-time integer or a closed expression in
compile-time constants, type-level constants and runtime-profile symbols** (L1). A
per-domain figure that names a runtime value is not a bound, and premise 3 fails at the
loop, the call **or the acquisition** that introduced it, `UnboundedStoreDemand`, with
that value named: `seq_arena::<u8>(arena: &uniq scratch, count: wanted)` for a runtime
`wanted` fails at that statement, in straight-line code. A marked program's runtime-sized
take is written `requires count <= k` for a closed `k` and composed at `k` — and [RES-10]
route (i) gives a loop's **trip count** the same `requires`-based route, which is round
7's correction of an asymmetry the seventh draft had no reason for.

**What premise 3 is for is stated.** It is a boundedness filter over the published
envelope: it decides whether a finite `E` exists and what its figures are. It is *not*
what stops an acquisition from over-drawing a store; that is the per-acquisition
obligation — a checked spelling refuses with a value [RES-6], a proved spelling discharges
under [MSR-4] — and it holds with or without the marker. Round 7 defeated the second half
through [CALL-6] rather than through this rule.

For a selected target `T` and its runtime, `E-materializes(P, T, B)` holds when every
symbolic figure of stage one has a concrete value (frame sizes post-codegen [STK-3],
strides and alignments [STOR-6], the runtime's profile capacities [RUN-2]), every row is
representable and is one the runtime's published profile can carry. Failure here is a
**target-qualification failure** under [STOR-6] and [QUAL-2] 2369-2382: it stops
compilation, cites no language rule, and is not a source rejection.

*Judgment:* stage one, per domain, over the checked program; deterministic, terminating,
and free of search, budget or timeout. *Publishes:* the property, and `E`. *Amends:*
[STOR-6] 738-767, whose "the language defines no numeric per-function frame ceiling"
keeps its scope for the *language* and is joined, for a resource-closed build, by a
computed per-context envelope, and whose target-stage obligations gain
`E`-materialization. *Depends:* [QUAL-2] 2369-2382, where stage two's failure lands.
*Law:* L1, L8, L9. *History:* r7 F2-11; r6 F2-17.

**[RES-4] The entry requirement, the heap, and the deferrals it moves.** The entry may
carry the marker `resource_closed` **[S19]** before its `command` program-kind marker:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
```

The marker changes no acceptance judgment: every program is judged by the same rules. It
changes two things. It makes the failure of [RES-3] stage one a hard error rather than a
reported property. And it selects which [SCOPE-3] 27-31 deferrals apply: for a marked
program, **stack exhaustion and covered-store exhaustion are inside the model**, and for
every other program they stay deferred. **One thing the marker does not select is whether
a program may abort**: [PROV-6] refuses a type whose release graph has a cycle in every
program, so L3's last clause is true rather than aspirational and the release walk has no
worklist and no `wf_resource_abort` caller.

A program whose call graph reaches a `Heap<'s>` is not resource-closed, and a `main`
selecting `command.heap` is by itself the rejection. A bounded general store is still a
general store: an envelope item can promise bytes, and cannot promise that the next
contiguous aligned request has a home.

*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage one is a
hard error naming its own cause: the heap-reaching path, the call-graph cycle [STK-2], or
the unbounded store [RES-5] as `UnboundedStoreDemand`, naming the loop, the call or the
acquisition and the runtime value. *Publishes:* the property as a compilation fact, and
the scope of [SCOPE-3]'s deferrals. *Amends:* [FN-7] 1216-1255 at 1217; [GRAM-2]
168-202's `program_kind`; and [SCOPE-3] 27-31. *Law:* L1, L6. *History:* r6 F2-7.

**[RES-5] Four algebras, one kind set, and a domain is a store.** Every covered store
presents its state through [MSR-1]'s measures. Exactly four **algebras** are defined, and
a **domain** is one pair (algebra, **store designator**), where a store's designator is
[PROV-1]'s region for a program store and the **spec-fixed name** [RES-9] gives it for a
runtime store. A store outside this list contributes no envelope item and denies [RES-3].

```text
| algebra                    | state         | acquire            | release        | kind        |
|----------------------------|---------------|--------------------|----------------|-------------|
| uniform slots              | len, cap      | +1 record          | -1, on the     | reusable    |
|  (lane, task, queue,       |               |                    | store's own    | capacity    |
|   completion and handle    |               |                    | release event  |             |
|   records of the runtime)  |               |                    | [RES-9]        |             |
| bump extent                | len exact,    | + advance<T>       | nothing; the   | consumable  |
|  (Arena<'s, bytes, align>) |  in bytes,    |   (count)          | store resets   | budget      |
|                            |  cap = bytes  |                    | with 's        |             |
| general heap (Heap<'s>)    | -             | -                  | per value, by  | undecidable |
|                            |               |                    | the release    | from E      |
| static and frame placement | fixed offsets | none at run time   | none           | compile-    |
|                            |               |                    |                | time        |
```

**One kind set, and [RES-10]'s table shares it**: *reusable capacity*, *consumable
budget*, *undecidable from E*, *compile-time*, and *external effect flow* — the last
belonging to no store row and describing opens, writes and submissions, which are not in
`E`. Round 7 found the seventh draft's two tables naming two different sets, so route
tests read cells whose values were outside the set the test was stated over.

**Domain is a store, not a kind**: two arenas in one program are two domains, so one
arena's reset is not invisible to the other's accounting, and a store minted inside a loop
body has a domain whose life is one iteration. **A runtime store's designator is fixed by
the specification and not by a profile row**, which is round 7's stage repair: the seventh
draft moved the exclusion *test* off the runtime's published row and left the domain
**key** and the membership condition reading it, so one program text and one compiler
version still gave two source verdicts on two runtimes.

**`advance<T>(count)` is a closed expression, and the store's own alignment is what makes
it one.**

> Every take advances the cursor by `round_up(size_ceiling(T) * count, align)`, where
> `align` is the **store's** own type constant, and both acquiring rows require
> `align >= align_ceiling(T)` as a compile-time comparison of two constants.

The cursor is then a multiple of `align` at every point, **the padding at a take is
exactly zero**, and therefore `len(arena)` is **exact** — which [MSR-1] and A.1 now say,
and which is what makes [RES-10]'s reset cancel. Round 7 found A.1 and this rule
disagreeing, with the recommended per-iteration idiom refused in both spellings as the
consequence. Whether the **operand** is closed is [RES-3]'s question.

*Judgment:* the composition of [RES-10] per domain, over the kind column this rule fixes.
*Publishes:* per program point, per domain, the store's `len` bound; and each domain's
acquire quantity and kind, which [RES-10]'s transfers and routes read. *Amends:* [OP-9]
974-1001, whose allocation-fit predicate gains [BLK-0]'s acquiring rows as callers, whose
ceiling table gains A.1's derived rows, whose region-bearing exclusion is lifted, and which
fixes `advance<T>`. *Law:* L3, L6, L8, L9, L16. *History:* r7 F2-7, F2-14, F2-18; r6 F2-1,
F2-17, F2-19.

**[RES-6] Typed failure, and the two spellings.** Every operation that can fail to obtain
a covered resource returns a typed value that names the failure and hands back every
affine input it did not consume. **The kernel declares no failure nominal**, because no
kernel acquisition takes an affine input; a library operation that consumes an owner and
may refuse declares its own nominal (3.L.5's `Grown`).

Each covered-store acquisition with a measure comes in exactly two spellings, on the model
of `+` and `+checked`: a proved form admitted only when [MSR-4] discharges its goal, and a
checked form that is total. **The `Heap` has no proved form** (L6). A store with measures
publishes more: a refused `seq_arena` establishes `room(arena) < advance<T>(count)`, which
is L8's second half and which is a fact only because [CALL-6] gives a provider relation a
source, an establishment point and a destination.

**A library release should be the proved spelling wherever its caller can discharge it.**
A checked release hands its refusal back as an `Option`, and a value inside one can be
legally destructured and discarded — must-consume behaving correctly [PROV-6], and not
must-return. A **proved** release under `requires room(pool.free) > 0_u64` has no refusal
arm, so on every path the value goes back; 4.1 is written on it.

**The runtime's handle table is a covered store, and its refusal is a variant.**
`reserve_file` **gains** `own ReserveOutcome` in place of the total `own FilePermit`
[SYS-2] 2261 declares today (owner-decided, [S25] and then [S33]), on the principle the
owner stated with it: **a failure the environment can produce is exposed as a typed value;
a failure we create ourselves is eliminated, and the type system carries it.** Its three
variants, and the relation each publishes:

```text
reserve_file(factory: &uniq FileFactory) -> outcome: own ReserveOutcome
  Reserved(value: FilePermit):  len(factory) = <call datum> + 1
  Exhausted():                  room(factory) = 0, len(factory) = <call datum>
  Failed(error: IoError):       len(factory) = <call datum>
```

**The refusal relation is published on the `Exhausted` arm and there only**, by
[CALL-4]'s existing per-variant route through [CALL-6]'s S13, so a marked program that
matches that arm derives `room(factory) = 0` and [RES-10]'s reusable-capacity route reads
it beside `saturating` and `cap(store)`. [SYS-7] 2473-2486's closed class set is
**unchanged** and the `Failed` arm carries it, so a portable class set stays payload
vocabulary and never becomes proof vocabulary. Round 7's finding is what forced the
partition: the seventh draft claimed the edge establishes `room(factory) == Z` *"when the
class is `ResourceExhausted`"*, and under S25's `Result` that was false twice over, because
a class is a member of the payload's class set and not a variant, no route in [CALL-4] is
conditioned on one, and publishing the relation unconditionally over `Err` is false for a
`PermissionDenied` at a table that is not full. Two variants remove both, which is the
shape Q20 puts to every covered store. **The cost is measured on the right alternative**:
a third arm at eleven call sites across five corpus programs, against a total
`reserve_file` over a proved capacity costing one header invariant per loop.

No covered-resource failure is a trap, an abort, a process exit, a retry, or a promotion
to a larger store. The batch-0079 floor's `wf_resource_abort` site loses its
allocation-refusal caller once allocation returns a value, and its release-walk callers
once [PROV-6]'s release graph refuses a cycle outright.

*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result`, an `Option` or a system
outcome nominal, plus [MSR-4] discharge at the proved spelling. *Publishes:* the returned
owner's identity on the refusal edge, and the store's own refusal relation where the store
has measures and a variant to carry it, through [CALL-6]'s S13. *Amends:* [SYS-2]
2164-2307 at 2261, whose row gains `ReserveOutcome` and its three published relations, and
2283-2285's proposition set; [SYS-7] 2473-2486, **unchanged**, which is why the class set
carries the `Failed` arm and no route is conditioned on a class; the batch 0079 exhaustion
floor; and [SCOPE-3] 27-31, whose heap-exhaustion sentence ceases to be true. *Depends:*
[CALL-6]; [CALL-4]'s per-variant route, which publishes the `Exhausted` relation. *Law:*
L3, L6, L8, L16. *History:* r7 F2-6; r6 F3-2, F2-2; the owner's [S33].

**[RES-7] What bare resource-closedness does not cover, and where the exclusion is
decided.** Disk space, the successful acquisition of a host object not exclusively
reserved before start, network reachability and throughput, CPU time, deadlines,
fairness, power, device health, host termination and quota revocation are outside [RES-1]
and outside every judgment in this file; they remain typed system outcomes where the
operation defines one, and environment conditions where it does not.

> An **action** — a [SYS-2] operation or a [SYS-5] 2397-2432 release action — acquires one
> submission record and one completion record exactly when **its declared target contract
> is `may-suspend`**, because [SYS-2]'s own contract says a may-suspend action has a
> logical record before target handoff and a `wait-capacity` submission outcome that
> retains a bundle; and `reserve_file` acquires one handle record [RES-9].

Quantifying over actions is round 6's: [SYS-5]'s release table declares a `ReadFile` close
as `may-suspend; terminal` and the shipping adapter routes it through the same fixed
submission table every read uses.

**And the exclusion is split at the stage boundary L1 draws, over designators the
specification fixes.**

- **Source stage, step 5.** A marked program's composition publishes, per runtime store
  **named by [RES-9]**, the demand it computes [RES-10]. That is a function of program text
  alone, because the store's identity is a spec-fixed name and the set of stores the source
  judgment quantifies over is closed by the specification. A positive demand is a
  **declared requirement**.
- **Qualification, [QUAL-2] 2369-2382.** A target whose profile cannot carry a declared
  requirement — a store whose published capacity is below the demand, zero included — fails
  qualification, stops compilation, and cites no language rule.

*Judgment:* the derived column above, per action, from declared data, which [RES-10]'s
may-suspend transfer reads; the composition's per-store demand at step 5; and no source
rejection of its own. *Publishes:* the boundary, and each marked program's declared
runtime-store requirements. *Amends:* [SYS-2] 2164-2307's declaration records and [SYS-5]
2397-2432's release actions, which gain the derived column; [QUAL-2] 2369-2382; [ERR-4]
1484-1488, whose deferral gains the two families [RES-6] and [RES-4] move inside.
*Depends:* [SYS-2]'s and [SYS-5]'s may-suspend target contracts; [RES-9]'s spec-fixed
names, without which this rule's source half reads runtime data. *Law:* L1. *History:*
r7 F2-7; r6 F2-4, F2-5.

**[RES-8] The per-function summary is part of the callable boundary, in three pieces,
and every piece is declared.** Each function's boundary [FN-1] 1005-1012 gains a
**source-stage per-domain map** over its formal provider and measure terms, substitutable
at a call site and keyed by (algebra, store designator) [RES-5]; a **declared saturation
fact per store**, written `saturating(d)` **[S26]**, where `d` is a **store designator** —
a region name in scope, or one of [RES-9]'s six spec-fixed names; and a **target-stage
own-storage figure** covering every store it reserves and its own frame.

**The designator is round 7's repair and it is what makes the fact usable at all.** The
sixth draft keyed the fact to a **provider parameter** and the shape it was built for has
none; the seventh keyed it to a **store region**, and every reusable-capacity domain in
[RES-5] is a **runtime** store whose identity is not a region and which no source
declaration can name — so `saturating` could not be written for any domain the route that
reads it applies to, and a permit pool behind one helper was refused while the same code
inlined into `main` was accepted.

`saturating(d)` says *this function performs no acquisition on `d`'s store that could
succeed when that store is full*. It is checked **one way** — declared implies exhibited —
and not by [EFF-2]'s set equality, because saturation is a negative universally quantified
property. A kernel or system row's own saturation is table data on the row: a **checked**
spelling is saturating and a **proved** one is not.

*Judgment:* the one-way check of each declared `saturating(d)` against the body, citing
RES-8 at the `contract_block`, which [RES-10]'s reusable-capacity route reads at a call.
*Publishes:* all three components. *Amends:* [FN-1] 1005-1091 at 1005-1012; [GRAM-2]
168-202's `contract_block`. *Depends:* [PROG-1] 1492, the one closed unit the composition
claim is scoped to; [ENT-1] 2661, why proof provenance may not be read; [RES-9]'s
spec-fixed names. *Law:* L1, L5. *History:* r7 F2-3; r6 F2-10.

**[RES-9] The runtime's own stores have names, and a release event is stated over the
record.** A covered store needs **six** things written in one place: an **identity**, a
**capacity**, an **acquire event**, a **release event**, a **refusal relation**, and a
**multiplicity**. The program's own stores have all six from [PROV-5], [BLK-2] and
[MSR-2], with the region as the identity. **The specification fixes six identities for the
runtime's:**

```text
handles       submissions       completions       tasks       lanes       queue
```

These are the designators [RES-5] keys a domain by, [RES-7] publishes a demand for and
[RES-8]'s `saturating(d)` names. **The profile publishes a capacity for a named store; it
does not name the store.** That is the whole of round 7's stage repair: the set of covered
runtime stores a source judgment quantifies over is closed by the specification, so
premise 3 is a function of program text, and a runtime that owns another store publishes a
capacity for no name and contributes no source demand.

[SYS-10] 2554-2574 **is amended.** Its "Reserving it promises no native descriptor,
**handle-table entry**, kernel memory, or host quota" is replaced by: *reserving a
`FilePermit` consumes one record of the runtime store `handles`, whose capacity the
target's profile publishes; host exhaustion at the open is a different condition and
remains the ordinary `ResourceExhausted` member of the open operation's typed `IoError`
result, outside `E`.* And its "This first slice never returns or recycles the permit" is
replaced by:

> A handle record returns when the value holding it is released, when it is consumed by
> an operation that produces no successor holder, or when the operation it authorized
> returns any outcome that produces no holder. For each covered runtime store, the set of
> acquire sites and the set of release sites must together cover every path of every
> action that touches it, and a target that cannot exhibit that coverage fails [QUAL-2].

Stating it as a **closure obligation on the store** is what stops the next open-like
operation from extending an enumeration. [SYS-2] 2283-2285's closed proposition set is
**amended too**: the measure relations of a covered system store join that enumeration as
a named source, landed by [CALL-6]'s S13. **The multiplicity is one table per process.**
**The release row's second subject** goes in the release action's own effect row, so
`ReadFile`'s release exhibits `writes(owner)` **and** the `handles` path — and, by
[RES-7]'s widened quantifier, that action's own may-suspend records too. Reclassifying
`ReadFile` as capability-released was considered and refused: its release needs no
capability, so [PROV-6]'s criterion does not reach it.

*Judgment:* none by itself; it supplies the fact sources [RES-5], [RES-7], [RES-8] and
[RES-10] read, and its failure is a runtime's [QUAL-2] qualification failure. *Publishes:*
each runtime store's spec-fixed name, capacity, acquire event, release event, refusal
relation and multiplicity, established through [CALL-6]'s S13. *Amends:* [SYS-10]
2554-2574, [SYS-2] 2164-2307 at 2283-2285, [STOR-3] 688-719 at 709-712, and [SYS-5]
2397-2432's release-completeness at 2397-2400, which is **kept**. *Depends:* [QUAL-2]
2369-2382. *Law:* L1, L3, L5. *History:* r7 F2-3, F2-7; r6 F3-2, F2-4.

**[RES-10] How `E` is composed.** Round 6 found seven holes in this arithmetic and round
7 five more, each of one shape: a test whose object is not a quantity the language has, or
a composition with no site. **Every quantity tested below is a compile-time integer or a
closed expression** (L1), and **the backedge delta is computed by this composition from
the rows' declared deltas — never proved by [MSR-4]**. Every covered resource has one of
[RES-5]'s five kinds and [RES-5]'s kind column assigns it: *reusable capacity* is bounded
by peak `len`, *consumable budget* by net consumed, *external effect flow* is not in `E`,
and the other two contribute no run-time acquisition.

**A statement's summary is one map from label to `(peak, delta)`, and the label set has
two members no ordinary edge carries.** The labels are its fallthrough, each variant of a
result it produces, each `break` label it may take, `propagate`, **`return`** and
**`retained`**. `return` carries what the statement holds on an edge to [FN-1]'s
function-return sink. `retained` is what a statement holds that no edge of it will
release: [STK-4] admits a loop no `break` resolves to, which is what makes an idle loop
and a service loop entries at all, and under a label set without it that loop's map has no
entries.

**The primitive transfers are per algebra**, so a 256-byte take is charged 256:

```text
acquire           (peak a, delta +a)      on the success exit; (0, 0) on a refusal exit,
                                          a being the domain's own acquire quantity
                                          [RES-5]: one record for uniform slots,
                                          advance<T>(count) for a bump extent
release           (peak 0, delta -a)      at a `dispose` or a store's own release event
derived release   (peak 0, delta -a)      per released value on a scope-exit edge, which
                                          under D3 is where a hosted program's heap
                                          releases are
may-suspend       (peak 1, delta  0)      one submission and one completion record, on the
  action                                  statement or edge that performs it [RES-7]; a
                                          scope exit carrying k of them is (peak k, 0)
reset a store     see the scope rule      contributed by the release action of a store
                                          whose [RES-5] algebra reclaims with its region
move / borrow     (peak 0, delta  0)
```

A delta may be an integer or an interval `[min, max]`. **An interval enters the peak
equation as its `max` and the delta equation as an interval, and every test reads its
`max`.** The compositions are:

```text
sequence   when A has a fallthrough exit, for each label L of B:
             peak(A;B)[L]  = max( peak(A)[fallthrough], max(delta(A)[fallthrough]) + peak(B)[L] )
             delta(A;B)[L] = delta(A)[fallthrough] + delta(B)[L]     (interval sum)
           for each non-fallthrough label L of A, A;B carries A's own (peak, delta)[L]
           when A has no fallthrough exit, A;B is exactly A's map
           `retained` and `return` compose by the SAME formula as every other label

branch     the union of the arms' maps, keyed by label; two arms reaching one label
           contribute the componentwise max of peak and, when their deltas differ, the
           interval [min, max] of delta

scope      a block's map is its body's map with, applied AT EVERY LABEL at which that edge
           leaves the block, each binding's derived release and each store's reset. A
           `reset` at a label L cancels, BY DEFINITION, exactly the composed delta of that
           block's own map at L on that store's domain, so delta(block)[L] = 0 there

call       substitute the callee's source-stage map [RES-8] at the call site, replacing its
           formal measure and provider terms by the actual ones — EVERY entry, `retained`
           and `return` included — and read its declared saturating(d) for route (ii)

loop       let d be the backedge delta COMPUTED by this composition from the declared
             deltas of the rows on the loop's paths, and p one iteration's peak.
             max(d) <= 0  -> peak(loop) = p; delta(loop) = d; no iteration bound is needed
             max(d) >  0  -> bounded on a domain exactly when the FIRST of the two routes
               below applies, tried in this fixed order:
                 (i)   a trip-count bound T that is a compile-time integer, or a closed
                         expression [MSR-4] establishes as an upper bound on the loop's own
                         trip count from its endpoints and this function's verified [FN-8]
                         requirements:
                         peak(loop) = p + (T - 1) * max(d);  delta(loop) = T * d
                 (ii)  the domain's kind is REUSABLE CAPACITY, its store's cap is a standing
                         fact [MSR-2], and every acquisition on the loop's paths is
                         saturating, read from the row and from each callee's declared
                         saturating(d) [RES-8]:
                         peak(loop) = cap(store);  delta(loop) = 0
               Otherwise there is no finite E and premise 3 fails here.
           a loop with no fallthrough carries no fallthrough entry and its retained entry is
             p composed with d discharged by the same routes

overlap    for a set of statements an implementation may execute with overlapping execution
           under [PAR-1], [PAR-2] or [PAR-3], the map is the componentwise SUM of the
           members' peaks and the sum of their deltas; for a staged permission over a loop
           [PAR-3], the peak is k * p, where k is the runtime's published outstanding-work
           bound for that store [RUN-1]. A marked build takes no such permission, so this
           rule fires in no marked build; it is stated because the composition must be total
```

**The extraction, stated once.** [RES-3] premise 3 asks whether a demand is bounded and
[RES-2] asks for a figure; this is the sentence that turns the map into the figure:

> For the entry function's body, the per-domain figure of `E` is `max` over every label of
> that map of `peak[L]`. Labels are alternatives on any one execution, so the maximum is
> the correct bound and no sum is admitted.

**Five things are round 7's and each closes a hole.** **The invariant route is deleted**,
because it asked [MSR-4] to derive `delta <= 0` and `delta` is a component of this rule's
own map — not an [ENT-2] term — while the only thing an [INV-1] header invariant can say
is a **level**, the vacuous shape round 6 killed; the trip-count route absorbs the
writer-controlled half. **The reset is a definition, not an arithmetic accident**: the
take and the reset are the same quantity, and summing them as two independent intervals
always widens, so under a bounded arena `len` the recommended per-iteration idiom composed
to `[-256, 256]` and was refused in both spellings; [RES-5] makes `len(arena)` exact and
the reset cancels per label. **The scope composition is new and it is what the reset
needs**: without it a `break`, a `give`, a `propagate` or a `return` out of a region block
carried the block's positive delta with the reset charged nowhere. **The overlap
composition replaces a line that charged an overlap like a sequence**, under which a
marked driver's `read_at` loop held `k` submission records where `E` promised one — and
§6.10's disposition that "the `par` rule is deleted" was false, the rule having been
rewritten into the unsoundness. **And `retained` composes like the level it is**, by the
one formula.
*Judgment:* the composition itself, per domain, over the checked program; deterministic,
ordered and free of search, and every quantity a compile-time integer or a closed
expression — which is the judgment [RES-3] premise 3 and [RES-2]'s figures both read.
*Publishes:* per statement, per domain, one map from label to `(peak, delta)`, `return`
and `retained` included; and the per-domain figure of `E` through the extraction.
*Amends:* nothing in v0.41; this is new machinery over [FN-1]'s existing graph.
*Depends:* [FN-1] 1076 as [STK-4] corrects it, where the label set comes from; [RES-5]'s
kind column and acquire quantity; [RES-8]'s declared saturation fact and its designator;
[CALL-6], without which route (ii)'s `cap(store)` is not a fact; [RUN-1]'s published
outstanding-work bound, the overlap rule's `k`. *Law:* L1, L8, L9. *History:* r7 F2-1,
F2-2, F2-4, F2-9, F2-10, F2-11; r6 F2-1, F2-7, F2-11, F2-18.

##### 3.K.7.1 Which stage decides what

```text
 1  tail-SCC rewrite, source premise [STK-1]        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-4]    source stage   compiler
 4  per-function source-stage demand map and
      declared saturation facts [RES-8]             source stage   compiler
 5  loop, branch and scope composition [RES-10],
      its extraction, and the per-runtime-store
      declared demand [RES-7]                       source stage   compiler
 6  concrete sizes, strides, static image           target stage   compiler
 7  per-context frame envelope [STK-3]              target stage   compiler, post-codegen
 8  runtime profile capacity for each named store   target stage   runtime data
 9  matching each declared demand against the
      profile [RES-7, QUAL-2]                       target stage   compiler
10  assembling E, its digest, and emitting it       target stage   compiler
11  selecting W for this run                        PreStart       launcher
12  matching E's digest against the module          PreStart       launcher
13  committing every region, stack, slots and
      handle item                                   PreStart       launcher
14  creating lanes and reaching the ready barrier   PreStart       runtime
15  initializing every adapter record and queue     PreStart       runtime
16  crossing SourceStart and invoking main          PreStart -> Running  runtime
```

Steps 1 to 5 decide whether the program is source-resource-closed and are the only steps a
source rejection may cite; steps 6 to 10 decide whether this build qualifies; steps 11 to
16 decide whether this run is admitted. **Every rule that issues a source rejection appears
at one of steps 1 to 5**, and every quantity those steps test is a compile-time integer or
a closed expression.

#### 3.K.8 `[STK]`: the stack

**[STK-1] A tail edge is one whose caller frame is dead, the rewrite removes the
transfer, and the dispatcher is an ordinary function.** For each strongly connected
component of the call graph in which every intra-component call edge is a tail edge, the
compiler rewrites the component into **one dispatcher function with one frame** before
frames are measured; the intra-component edges are then not calls at all.

An intra-component edge is a tail edge exactly when, at that edge: no loan, borrow, view,
region or reborrow the caller introduced is live; **no compiler-derived release with a
non-empty effect row remains to run after the call**; no binding of the caller that is
linear in its scope is still live [PROV-6]; no `par` join is outstanding; and no place the
caller declared is read by any value live across the call. **The release clause names the
property it is reaching for**, which is round 7's correction: written as "no
compiler-derived drop remains to run", it denied the tail edge to any function holding a
live frame-resident affine local at the jump, and [EFF-2] 1427 already computes exactly
the distinction.

**And the dispatcher is checked.** L7 says the rewrite runs before any resource judgment;
the converse is stated here: *the dispatcher is an ordinary concrete function and is
checked by [LIV-1], [PROV-6] and [EFF-2] exactly as any other; a component whose
dispatcher fails any of them is refused at the component, naming the member whose live set
disagrees at the dispatcher's loop head.* Without it the premise is read on one program
and the frame measured on another. **There is no separate target obligation**, because an
activation record and a frame are target-stage objects.

Two costs are recorded rather than discovered. A component member that opens a region for
an `arena_frame` has a live region at the jump, so its edge is not a tail edge and [STK-2]
refuses the component. And under D3 the linear clause bites much less in a hosted program
than the seventh draft implied, because a heap-backed value in a scope holding the `Heap`
is affine and its release carries a non-empty row — which the second clause, not the
third, is what tests.

*Judgment:* per edge, from the ownership and loan state [LIV-1] and [PROV-6] already
compute, plus the dispatcher check; no proof search. *Publishes:* an acyclic call graph,
or a component that is still cyclic, and the strongly connected components [PROV-5]'s
activation refusal reads **after** this rewrite. *Amends:* nothing; this is a lowering, so
recursion stays permitted. *Depends:* [PROV-6]'s per-scope linear predicate and [LIV-1]'s
liveness; [EFF-2] 1427, which distinguishes an empty release row from a capability spend.
*Verified today:* probes `f2b` and `f8_tailframe` are mutual tail recursions carrying a
live borrow of a caller local and are accepted. *Law:* L7. *History:* r7 F2-13; r4
F2-NB14.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After [STK-1],
a program whose call graph still contains a cycle has no finite stack envelope and is not
resource-closed. A `requires` bound on a recursion parameter, a proof that a recursion
argument decreases, and every other depth certificate are **not** admitted as a
substitute. *Judgment:* under [RES-4], a hard error citing STK-2 rendering the complete
cycle in call order, with the restructuring `rewrite the recursion as a loop over an
explicit FixedVector work list, or make every recursive call a tail call whose caller
frame is dead at the jump`. *Publishes:* nothing. *Amends:* nothing. *Depends:* [FN-6]
1211-1214, whose permission of recursion is why a recursive program is excluded from
[RES-4] rather than rejected. *Law:* L7. *History:* r1 F2-A2.

**[STK-3] The frame envelope, over the whole chain, for every context the artifact holds
live.** For each execution context, the `stack` item of `E` is measured over the context's
**whole chain**, from the point at which the environment hands that context a stack to the
point at which it takes it back. `main`'s own chain is one segment; the runtime's start-up
trampoline, its teardown, its drop glue, the release walk's straight-line frame cost and
the exhaustion floor's own frames are others. Within one segment,

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

**`frame(f)` includes the alignment slack of every frame-placed arena in `f`**, measured
post-codegen, which is where `arena_frame::<65536, 4096, 'a>()`'s page alignment is
discharged; a `stack` item's own `alignment` cell is the ABI's stack-base alignment.

**Every context the artifact holds live is a named item, and there are more of them than
the seventh draft's envelope carried.** `wf_floor.c:279` attaches the **host** thread,
`:292` the **created entry** thread, and `par_runtime.c:527` each worker lane; `:328`
joins, so the host thread's stack stays live for the whole run; and `:234-246` `mmap`s an
alternate stack **per attaching thread**, so a `lanes(1)` marked build holds **two**. Each
is a `stack` item [RES-1], an alternate stack included, and its handler chain is measured
by the formula above; [RUN-1] carries the converse, that a qualified runtime holds no live
stack `E` does not name.

**Every named stack is materialized, not read.** `wf_floor.c:303-329` shows the floor
creating the entry stack and **silently falling back to the host thread on failure**, and
`:234-246` returning silently on `MAP_FAILED` — so two items of `E` may be absent with no
report. [RUN-4] creates each at the figure and alignment the row names and reports
`StartFailed(item)`; §6.2 records both as [QUAL-2] failures of the shipping
implementation. `E` is an **output** of code generation, recomputed after every
optimization.

*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic discipline, which
[RES-3] stage two reads. *Publishes:* one `stack(context, bytes, alignment)` item per live
context per profile row. *Amends:* [STOR-6] 738-767. *Law:* L5, L6. *History:* r7 F2-12,
F2-19; r6 F2-12.

**[STK-4] A loop with no resolved break has no normal successor.** [FN-1]'s conservative
structural graph gains exactly one sentence, and it replaces one:

> A `loop_stmt` has an edge to `normal_successor(loop_stmt)` **if and only if some
> `break_stmt` resolves to it.**

No second clause. A `return`, a `propagate` `Err` edge, and a `give` delivering outside
the loop are edges to the function-return sink or to an enclosing construct. Probe
`n3_propagate_loop` is the driver loop this admits and is `[FN-1] FunctionFallthrough`
today. This is the rule that lets an idle loop and a service loop be entries at all, and
**[RES-10]'s `retained` label is what makes their demand visible.** A scope whose exit
edge is unreachable carries no compiler-derived release and no [LIV-1] check, so a binding
live on a path reaching only such a loop is not an error, and no reset runs on that absent
edge either.

*Judgment:* [FN-1]'s existing reachability and fallthrough judgment over the corrected
edge set, which [RES-10]'s label set reads. *Publishes:* the graph, hence [RES-10]'s label
set. *Amends:* [FN-1] 1005-1091 at 1076. *Verified today:* probes `n2_idle` and
`f3_forever`. *Law:* L1, L9. *History:* r5 F2-2; r4 F2-NB9.

#### 3.K.9 `[RUN]`: runtime closure and admission

**[RUN-1] The artifact, runtime closure, and the no-overlap obligation stated over a real
object.** For every judgment in this file the artifact is the writer's code, the
compiler-derived cleanup and drop glue, the monomorphized instances, the `par` runtime,
the allocator and its metadata, and the qualified target adapter.

A runtime qualified for resource-closed programs performs, after the `SourceStart` barrier
and until `ProgramFinished`, **no covered acquisition whatsoever**: no allocator call for
runtime-owned storage, no thread or helper creation, no stack, queue, table or worklist
growth, no lazy TLS or adapter initialization, no first-use mapping, and no first-error
formatting buffer. Every runtime record is established before the barrier or carved from a
fixed store already an item of `E`. **And it holds no live stack `E` does not name**
[STK-3]. Today's adapter meets neither: `bridge.c:670` initializes under `pthread_once`
inside the submit path, and the floor holds two unnamed stacks. Both are honest [QUAL-2]
failures of one implementation.

**The no-overlap obligation is stated over the permission judgment, because there is no
`par` construct.** [CAP-1] 1968-1969: "This version defines no thread construct." [PAR-1],
[PAR-2] and [PAR-3] are permissions an implementation *may take*, and [PAR-1] 1993 says
whether an overlap was performed is not observable. The seventh draft's auditable property
— *"the emitted module of a marked program contains no `par` construct"* — therefore
quantified over an object that does not exist:

> **A marked build is qualified only when the compiler's own permission judgment grants no
> [PAR-1], [PAR-2] or [PAR-3] permission anywhere in the module, and the compiler emits
> that verdict beside `E`.** The judgment is a deterministic function of the program
> [PAR-1], the verdict is what `--par-ledger` already prints, and the obligation lands on
> the party that computes it.

The obligation is soundness-critical and the hazard is executed: the current runtime's
wait path runs a stolen task on the waiting lane's own stack, so `stack(lane_i)` as
[STK-3] computes it is wrong by a factor bounded only by the outstanding-task count.

**Acquisition and admission control are different obligations.** A qualified runtime must
additionally have, per store, a **bounded admission discipline** whose bound is that
store's published capacity: it declines work for which no record is available and resumes
when one is, acquiring nothing. **That bound is the `k` [RES-10]'s overlap rule reads.**
What stays forbidden is **inline execution**, which nests a task's chain inside a lane's
current activation and which no term of [STK-3] counts, and **unbounded waiting** on a
store no other frame will release.

*Judgment:* a target- and build-qualification obligation, auditable from the emitted
permission verdict and the runtime's own translation units; its failure is a [QUAL-2]
failure, not a source rejection. *Publishes:* the runtime's own items and capacities, and
the per-store outstanding-work bound `k`. *Amends:* [SYS-2] 2164-2307 at 2270, kept and
given its companion; [QUAL-2] 2369-2382, which gains the emitted permission verdict.
*Depends:* [PAR-1] 1993, whose unobservability sentence is why the obligation is over the
judgment and not over the emission. *Law:* L3, L5. *History:* r7 F2-1; r6 F2-12.

**[RUN-2] `par` enters `E` as an open profile, and a marked build publishes `lanes(1)`.**
For each supported lane count `W`, the runtime publishes one finite profile row. **The row
is open, not enumerated**: it publishes one capacity per **named** store [RES-9] and one
figure per item of [RES-1] the runtime owns, enumerated by the runtime and not by this
rule — and the **set of named stores a source judgment quantifies over is closed by
[RES-9]**, which is what keeps premise 3 a function of program text while the row stays
open. What is a function of program text is **the profile row a marked build publishes is
the `W = 1` row**. Two consequences follow: [PAR-3]'s replicated places cannot occur in a
marked build, because [RUN-1]'s verdict grants no permission; and [STK-3]'s worker-lane
chain has exactly one instance.

*Judgment:* the published-row rule on a marked program, and [RES-3] stage two's match of
each declared demand against the row [RES-7]; the compiler emits no per-`W` clone.
*Publishes:* the `region`, `lanes`, `slots`, `stack` and `handle` items of each row.
*Amends:* the sentence common to [PAR-1] 1995, [PAR-2] 2000-2033 and [PAR-3] 2035-2061,
whose overlapping-exhaustion resource condition is unreachable for a program
resource-closed on this target. *Depends:* [RES-9]'s spec-fixed names. *Law:* L5, L9.
*History:* r7 F2-1, F2-7; r6 F2-18.

**[RUN-3] The parallel footprint of an allocation is its provider place, of a view its
logical origin range, and 1981's intervening list is a footprint property plus two
premises.** In [PAR-1] 1971-1998's written-footprint clause at 1975, "the caller region
each `allocates(arena 'r)` entry names after region substitution" is replaced by "the
places each `allocates` path reaches under the [EFF-2] call-boundary projection", the same
projection the rule already applies to `reads` and `writes`. Two statements that allocate
from one provider therefore conflict, and two from distinct providers do not; with
[PROV-6] the same is true of two that only release.

[PAR-2] 2000-2033's permission for a fill through a `MutSlice` needs two amendments. The
**loan** condition is stated over **iteration-formed** loans: every exclusive loan formed
by a statement of `B` is rooted in a binding `B` introduces, and a loan formed before `L`
on a root every footprint of `B` reaches only through 2005's refined single-element ranges
does not deny. And the **write footprint** of `set m[at] = v;` contains its origin at the
**logical** range `[a*at+b, a*at+b+1)` ([PROV-3] use 1), carried to storage by [MSR-1]'s
injectivity sentence.

> [PAR-1] 1981's **form** enumeration becomes a footprint property: an intervening
> statement of any form is admitted when its footprint and its loans satisfy this rule's
> conditions. **1981's two other denials are kept as the separate premises they are**: a
> statement carrying an exit edge denies permission, and a non-call statement that forms a
> borrow denies permission.

Every new statement form then arrives permitted or denied by its own footprint, and
nothing denied for a reason other than a footprint becomes permitted.

*Judgment:* the existing [PAR-1] and [PAR-2] permission judgments, with the form
enumeration replaced by a footprint test, the two non-footprint premises kept, one added
loan clause, and logical ranged origins. *Publishes:* permission, which [RUN-1]'s
marked-build verdict reads and [RES-10]'s overlap rule composes over. *Amends:* [PAR-1]
1971-1998 at 1975 and 1981, [PAR-2] 2000-2033, and [PAR-3] 2035-2061. *Depends:* [PAR-2]
2005; [MSR-1]'s injectivity sentence. *Law:* L2, L5, L10. *History:* r6 F2-8, F2-14.

**[RUN-4] The startup protocol.**

```text
PreStart
    select a row of E from the target's profile table, largest supported W first
    refuse a row whose digest does not match the module being started [RES-2]
    materialize every item of that row:
        commit each region (committed backing, not a reserved address range), at its
            bytes and its alignment
        create each stack the row names — the entry context, each lane, the host
            context, and each alternate stack — at the row's figure and alignment
        commit each slots item at its count, member size and alignment
        commit each handle item
        create W-1 lanes and park them at the ready barrier
        establish every queue, task, completion and wait record
        initialize every adapter record, TLS block and runtime table
    a step that fails -> report StartFailed(item); a silent fallback is a [QUAL-2]
        failure of the implementation, not an option. For an unmarked program the
        launcher may select the next smaller row and start over

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

A marked build publishes exactly one row [RUN-2], so there is no next smaller row and the
failure of any item is `StartFailed` on the first attempt.

*Judgment:* a target obligation, not a source judgment. *Publishes:* the selected row.
*Amends:* [PROG-3] 1505-1537, whose start-time obligation gains the materialization of
`E`, the digest match, the creation of every named stack at its figure and alignment, and
the mandatory `StartFailed`, and whose `ProgramFinished` boundary is now named. *Law:* L1,
L5. *History:* r7 F2-12; r6 F2-9.

**[RUN-5] Admission, and the theorem.** `Admitted(H, row)` holds when an environment `H`
has actually established a grant implementing every item of the selected row before the
barrier, every named stack included, and for the duration of the run does not revoke it
and permits no unmodelled competitor to consume from it. Then:

```text
source-resource-closed(P)  and  E-materializes(P, T, B)  and  Admitted(H, row)
------------------------------------------------------------------------------
no covered-resource exhaustion in run(H, T(P))
```

*Judgment:* none by the compiler. *Publishes:* the deployment contract, which is the
selected row. *Amends:* nothing. *Law:* L1. *History:* r5 F3-I22.

#### 3.K.10 One name per concept

**Every spelling in this table is the owner's, decided 2026-09-03 or 2026-09-04** (3.S
records each decision with the alternatives weighed). Nothing in it is proposed.

```text
| concept                    | spelling              | why                                                     |
|----------------------------|-----------------------|---------------------------------------------------------|
| a run of slots, frame-      | FixedVector<T, n>     | its capacity is in its type because layout needs it     |
|   resident [S2]            |                       | before the run exists                                    |
| a run of slots, store-      | Vector<'s, T>         | one type at two regions; its capacity is a measure      |
|   resident [S1]            |   (brand elided)      | because a growth policy must change it                   |
| a run that is always full  | FixedVector<T, n>,    | array<T, n> retires [S34]: a full FixedVector with four  |
|   [S34]                    |   len = cap = n       | standing-fact measures is that case, rodata as a const   |
| the store's handle [S3, S4]| Heap<'s>, Arena<..>   | a value you must hold to allocate — and, under D3, to    |
|                            |                       | get the derived release                                  |
| the brand's spelling       | written iff the       | 3.K.0's determination principle, over regions only;      |
|                            | operands do not       | type and const arguments are always written [FN-2]       |
|                            | determine it          |                                                          |
| build an empty run [S7]    | seq_fixed, seq_arena, | the placement is in the name, because it decides which   |
|                            | seq_arena_proved,     | item of E the run becomes (L6)                           |
|                            | seq_heap              |                                                          |
| reserve a bump store [S9]  | arena_frame,          | as above; nothing else reserves                          |
|                            | arena_extent          |                                                          |
| append at either end [S8]  | seq_place,            | one name per end, whatever the backing                   |
|                            | seq_place_front       |                                                          |
| remove at either end [S8]  | seq_take,             | the window is two-sided, so L12's last clause is true    |
|                            | seq_take_front        |                                                          |
| return a wrapped window    | a library drain,      | writable in wf, so L18 keeps it out of the kernel;       |
|   to its origin            |   3.L.8               | [S29] is withdrawn and Q18 is the owner's question       |
| read a measure [S11]       | len, cap, room, head  | one quantity, one name, term and reader alike            |
| a read-only view [S35]     | Slice<'r, T>          | copy [S27]; capitalized like every compiler-owned nominal |
| a writable view [S35]      | MutSlice<'r, T>       | element writes only; affine, because [OWN-5] refuses two |
|                            |                       | exclusive loans on one range                             |
| form a view [S10]          | seq_slice,            | the two formers follow the two type names                |
|                            | seq_mut_slice         |                                                          |
| re-view a writable view    | no operation; the     | [S31]: a shared child reborrow of the exclusive loan,    |
|                            |   ordinary child      | [OWN-6]'s own machinery with a view as the parent, so a  |
|                            |   reborrow [OWN-6]    | fill-and-publish helper is writable [VIEW-6]             |
| release a store-backed     | the compiler-derived  | D3: the scope holding the capability gets the release on |
|   value                    | release; dispose p;   | every leaving edge; `dispose` [S12] is the early one,    |
|                            |   to release early    | and neither writes a capability                          |
| take a value apart [S13]   | let N(f: a) = move v; | the inverse of construct                                 |
| oblige a value to be       | linear struct N {..}  | for a logical obligation only; the storage obligation is |
|   consumed [S18]           |                       | derived from the type and the scope                      |
| write places               | set (p, q) = rhs;     | one commit rule, n-ary (D2)                              |
| a refusal                   | Option<T>             | the kernel consumes no affine input, so it declares no   |
|                            |                       | failure nominal                                          |
| a covered store's refusal  | a variant [S33]       | reserve_file -> own ReserveOutcome; a class of an error  |
|                            |                       | payload is not readable by a [CALL-4] route [RES-6]      |
| name a covered runtime      | handles, submissions, | spec-fixed, so a domain key and a saturating clause are  |
|   store [RES-9]            | completions, tasks,   | functions of program text and the profile publishes a    |
|                            | lanes, queue          | capacity FOR a name rather than naming the store         |
| declare a saturation fact   | saturating(d) [S26]   | keyed to a store designator, which a signature can name  |
| the property [S19]          | resource_closed       | the long spelling is the one in use                      |
| the failure variant field   | Err(error: e)         | [PRE-1] declares Err(error: E)                           |
```

`FixedRing`, `PoolVector`, `HeapVector`, `ArenaVector`, `AppendView`, `absorb`, `update`,
`seq_frame`, `seq_exchange`, `seq_rebase`, `seq_reslice`, `swap`, `Span`, `MutSpan`,
`HeapBox`, `ArenaBox`, `PoolSlot`, `heap_take`, `arena_take`, `pool_take` as a kernel row,
`on_propagate`, `Full<T>`, `TooSmall`, `OutOfMemory`, `PoolExhausted`, `NeedCapacity` and
`NoRecord` are **not** in the kernel vocabulary. The first four are library names for
kernel types (3.L.1); `update` and every swap spelling are [LIV-2]; `seq_frame`,
`seq_exchange` and `seq_rebase` are the fifth, sixth and seventh drafts' removals and
`seq_reslice` is the eighth draft's, because forming a shared view over a writable one is
[OWN-6]'s child reborrow [VIEW-6]; `Span` and `MutSpan` are the sixth draft's names;
`on_propagate` is the owner's rejection this round; the three box and slot names are runs
of capacity one or library nominals; the `*_take` names belong to the library's own
functions; and the last six are library nominals a writer declares over their own type.

#### 3.K.11 Amendment register

**This register is a collation of the `Amends:` and `Depends:` lines of every rule
in 3.K, and it carries nothing else.** It was written last, from the rules, and every
range in it was re-derived mechanically from `spec/kernel-spec.md` v0.41 at 30602914 in
this session by extracting each rule's first and last non-blank, non-heading line. It
covers 3.K only: 3.L amends nothing, and 3.S records decisions rather than amending
rules. The two assumed amendments of 3.K.0 are not registered here; they are their own
batches.

Eight conditions make it checkable rather than remembered, and each is a defect of
this file when it fails:

1. a changed row whose `by` column names no rule whose `Amends:` line reaches it;
2. an `Amends:` line no changed row carries;
3. a `Depends:` line no third-list row carries, or a third-list row no `Depends:`
   line produces;
4. **(a)** a `Depends:` citation whose sentence lies inside a range some `Amends:`
   line changes; and **(b)** a `Depends:` citation, or any sentence in the depended
   rule, whose subject type, operation spelling, or effect atom any `Amends:` line
   in this file renames, retires or redefines. When a dependency really does fall
   inside changed text, it is recorded **on the changed row and there only**;
5. **an `Amends:` line must state a change for every sentence in its cited range
   that the amending rule's own body contradicts**;
6. every `*Publishes:* X on Y` names the [ENT-3] source that establishes X and the
   destination clause that puts it on Y. **[CALL-6] is that rule**;
7. **every fact a rule states appears in that rule's `Judgment:` or `Publishes:`
   line, and every rule that reads such a fact names the judgment it comes from**;
   and
8. **every rule that computes a fact at one program point and uses it at another
   states both points, and every quantity a rule tests is one the language has.**
   This is round 7's condition. Its four memory BREAKS and three of its resource
   BREAKS are one shape: [MSR-3] named the judgment that decides a denotation and
   decided it on the wrong key; [CALL-6] named the source, the substitution, the
   destination and the support and then established at a point the substitution was
   not computed at; [RES-10] route (ii) tested `delta`, which is not a term of the
   language; and [CALL-7] demanded a form no test decides. Conditions 7 and 8 are
   the two halves of *a rule says what it does and where*.

**Changed.** Each row's `by` column names the rules whose `Amends:` lines reach it; a
row that also records a surviving depended sentence marks it **bold** (condition 4).

```text
| rule            | line      | change                                                          | by                          |
|-----------------|-----------|-----------------------------------------------------------------|-----------------------------|
| [SCOPE-3]       | 27-31     | heap exhaustion leaves the deferred set; stack and covered-store | [RES-4], [RES-6]            |
|                 |           | exhaustion leave it for marked programs                          |                             |
| [FORM-2]        | 39-89     | +3 renderings: result list, destructuring let and consume, set   | [CALL-4], [PROV-6]          |
|                 |           | target list and value list, dispose, the linear modifier         |                             |
| [GRAM-2]        | 168-202   | result list; resource_closed; region_params on nominals; the     | [CALL-4], [RES-4], [BLK-4], |
|                 |           | linear modifier; a saturating clause; requires/ensures (185-186) | [MSR-5], [PROV-6], [RES-8]  |
|                 |           | take a clause_expr                                               |                             |
| [GRAM-3]        | 204-215   | box/arena/buffer productions retire; runs are ordinary TYPEIDs   | [PROV-1]                    |
|                 |           | with targs; slice is joined by mut_slice                         |                             |
| [GRAM-4]        | 217-256   | destructuring let and consume; set target list and value list;   | [CALL-4], [LIV-2], [MSR-4], |
|                 |           | affine_factor GAINS terms at [MSR-4] in B2, not at [MSR-5];      | [PROV-6]                    |
|                 |           | stmt gains dispose                                               |                             |
| [GRAM-5]        | 258-280   | +clause_expr; atom and atom_list untouched. LANDED in v0.44      | [MSR-5]                     |
| [GRAM-9]        | 328-332   | unchanged; named because [MSR-5] moves the amendment away        | [MSR-5]                     |
| [GRAM-11]       | 345-350   | a fourth callee class in all three sentences                     | [BLK-0]                     |
| [TYPE-2]        | 357-360   | +5 nominals (2 providers, 2 runs, MutSlice); box, arena, buffer  | [PROV-1], [BLK-1], [BLK-2], |
|                 |           | and array retire [S34]; the flat-element restriction is not      | [VIEW-1]                    |
|                 |           | inherited; a full FixedVector const is const-eligible [CONST-1]  |                             |
| [TYPE-5]        | 370-394   | the written-argument criterion covers a fourth callee class and  | [BLK-0]                     |
|                 |           | becomes per-argument. **379 survives and [PROV-1], [BLK-4] and   |                             |
|                 |           | [LIV-2] depend on it; 383-386's mandatory construct arguments    |                             |
|                 |           | survive and 3.K.0 puts a construct outside its criterion**       |                             |
| [TYPE-6]        | 396-473   | the domain's spellings, nominals and region parameters; 401's    | [BLK-0], [MSR-6]            |
|                 |           | callee IDENT admission; 401's pbase gains a const generic        |                             |
| [TYPE-7]        | 475-479   | the deref domain becomes the two borrow modes alone              | [PROV-1]                    |
| [SET-1]         | 481-511   | loan-strength target traversal; SUBSUMED by [LIV-2] as its n=1,  | [PROV-3], [LIV-2], [VIEW-4] |
|                 |           | copy-target case; its commit at a loan-bearing place is refused  |                             |
| [SET-2]         | 513-528   | region-bearing rejection replaced by [PROV-3] use 3 and          | [PROV-3], [LIV-2], [VIEW-4] |
|                 |           | [VIEW-4]; its exchange exception to [OWN-5] 591 is inherited by  |                             |
|                 |           | [LIV-2]. **528's "it establishes no fact" survives UNCHANGED     |                             |
|                 |           | and [CALL-6] depends on it: a replace is a kill, never a         |                             |
|                 |           | publication, so a replaced-out value carries no measures**       |                             |
| [CONST-2]       | 546-559   | its naming of buffer and slice_of follows the retirements        | [VIEW-1]                    |
| [OWN-1]         | 563-571   | 563-564 gains mut_slice as affine and MOVES slice to copy;       | [PROV-6], [VIEW-1],         |
|                 |           | linear refines affine per scope; 569 gains the partial-consume   | [LIV-1], [LIV-2]            |
|                 |           | refusal and dispose as a consuming use; 566-567 is REPLACED by   |                             |
|                 |           | [LIV-2]'s commit premise                                         |                             |
| [OWN-4]         | 582-583   | the lent-onward child's loan ends at its receiving statement     | [PROV-7]                    |
| [OWN-5]         | 585-611   | origins generalize to loan-bearing values, carry a logical range | [PROV-3]                    |
|                 |           | and are copied with a copy view; the loan's extent is stated;    |                             |
|                 |           | two ranged access clauses; the address-computation freeze; 601-  |                             |
|                 |           | 604 and 608 restated. **606 survives and [VIEW-2] and [PROV-6]   |                             |
|                 |           | depend on it; 591 is outside this range and [LIV-2] and          |                             |
|                 |           | [PROV-6] depend on it**                                          |                             |
| [OWN-6]         | 613-627   | a child reborrow may name a caller-supplied region under the     | [PROV-7]                    |
|                 |           | result-type condition, for every reborrow. **614 survives and    |                             |
|                 |           | [PROV-2] and [VIEW-2] depend on it; the shared child reborrow    |                             |
|                 |           | survives and [VIEW-6] depends on it, with a view as the parent   |                             |
|                 |           | [S31]**                                                          |                             |
| [OWN-7]         | 629-633   | 629's overlap test extends to logical ranges. **630's subscript  | [PROV-3]                    |
|                 |           | conservatism survives and [PROV-3] use 2 depends on it** (4a)    |                             |
| [OWN-10]        | 640-644   | 643's arena content clause becomes one over Vector content.      | [PROV-1]                    |
|                 |           | **641 survives and [PROV-2] depends on it** (4a and 4b)          |                             |
| [OWN-11]        | 646-648   | 646's move prohibition is replaced by [LIV-1]'s join agreement.  | [LIV-1]                     |
|                 |           | **647 is UNCHANGED by this design and every loop-body borrow in  |                             |
|                 |           | §4 and 3.L relies on 3.K.0's SECOND assumed amendment (D4), not  |                             |
|                 |           | on 647 being vacuous. Probe `q2` shows the amended build         |                             |
|                 |           | refusing the bare form today**                                   |                             |
| [STOR-1]        | 675-683   | 675's storage-class list gains the two runs; the writable-place  | [PROV-1], [LIV-2]           |
|                 |           | partition (678-679) becomes [SET-1]/[LIV-2] write and [SET-2]    |                             |
|                 |           | replace, with 679's diagnostic kept for a live affine target     |                             |
|                 |           | whose right-hand side does not consume it; 681-682's growable    |                             |
|                 |           | and keyed-collection rejections are superseded by the library    |                             |
| [STOR-2]        | 685-686   | box_new and arena_new retire; a store take is a kernel row       | [PROV-2]                    |
| [STOR-3]        | 688-719   | the box and buffer HEAP rows retire with their types and are     | [PROV-5], [PROV-6], [RES-9] |
|                 |           | replaced by [PROV-6]'s release-graph walk; the store reset joins |                             |
|                 |           | the table; 690's edge enumeration gains the propagate error      |                             |
|                 |           | edge; 709-712 gains a second subject.                            |                             |
|                 |           | **700-706's drop order survives and [PROV-6] reuses it** (4a)    |                             |
| [STOR-4]        | 721       | confinement becomes the outlives relation over the region set    | [BLK-4]                     |
| [STOR-5]        | 723-736   | the position list becomes the three-way intensional split; the   | [BLK-4], [PROV-2]           |
|                 |           | per-leaf-provenance deferral is withdrawn as unnecessary         |                             |
| [STOR-6]        | 738-767   | E-materialization joins the target-stage obligations; the frame  | [RES-3], [STK-3]            |
|                 |           | sentences gain the per-context envelope and the frame-placed     |                             |
|                 |           | arena's alignment slack                                          |                             |
| [OP-1]          | 771-849   | +cap, +room, +head, pure, over runs, views and providers; five   | [PROV-2], [BLK-0], [BLK-2], |
|                 |           | constructors retire; ReservedLowerNames +3; 838 gains the class  | [VIEW-1]                    |
| [OP-4]          | 914-924   | indexable bases extend to the runs and views; the obligation is  | [BLK-1], [MSR-1]            |
|                 |           | against len, in logical coordinates; a subscripted measure place |                             |
|                 |           | in an erased clause discharges at its own attach site            |                             |
| [OP-5]          | 926-931   | "and contract predicate" narrows to a source condition           | [MSR-5]                     |
| [OP-7]          | 939-947   | slice_of and array_new retire; cap, room and head join the       | [VIEW-1]                    |
|                 |           | operations                                                       |                             |
| [OP-9]          | 974-1001  | the ceiling table gains A.1's derived rows, the region-bearing   | [RES-5], [BLK-0]            |
|                 |           | exclusion is lifted, advance<T> is fixed, and the predicate      |                             |
|                 |           | gains [BLK-0]'s acquiring rows as its callers                    |                             |
| [FN-1]          | 1005-1091 | the view ceiling and its duplicate-result rejection; an ordered  | [VIEW-6], [CALL-4],         |
|                 |           | result list; the &uniq referent refusal in a parameter list;     | [CALL-7], [RES-8], [STK-4], |
|                 |           | four boundary components; a loop_stmt's normal-successor edge    | [BLK-4]                     |
|                 |           | (1076). **1041-1047 survives and [PROV-3] depends on it**        |                             |
| [FN-2]          | 1093-1100 | the rejection narrows to loan-bearing and provider arguments;    | [BLK-4]                     |
|                 |           | explicit instantiation covers nominals. **Its "type and const    |                             |
|                 |           | instantiation arguments are always explicit" survives UNCHANGED  |                             |
|                 |           | and 3.K.0 and [BLK-0] depend on it** (probe `q4`)                |                             |
| [FN-3]          | 1102-1147 | the allocation component becomes the set of allocates paths      | [PROV-4]                    |
| [FN-7]          | 1216-1255 | command.heap; resource_closed; 1218's "declares no region        | [PROV-1], [RES-4]           |
|                 |           | parameters" is KEPT; allocates over a labelled input;            |                             |
|                 |           | 1245-1246's byte sequence gains the row                          |                             |
| [FN-8]          | 1257-1299 | clause operands are a clause_expr. **1275 survives and [MSR-3]   | [MSR-5]                     |
|                 |           | depends on it**                                                  |                             |
| [FN-9]          | 1301-1365 | terms as operands; measured and multi-ordinal results; variant   | [MSR-3], [MSR-4], [MSR-5],  |
|                 |           | routes carrying an ordinal binder; result field projection;      | [CALL-4], [CALL-6],         |
|                 |           | multi-datum clauses; 1313's result-datum requirement is lifted   | [CALL-7]                    |
|                 |           | for a declaration-domain relation over a &uniq state parameter   |                             |
|                 |           | and for nothing else; a &uniq parameter's measure is             |                             |
|                 |           | inadmissible in an ensures; the entry datum replaces 1316;       |                             |
|                 |           | 1345's M(c,q) admits a datum; 1357's narrow direct-set route is  |                             |
|                 |           | subsumed; a decidable completeness obligation over a handed-back |                             |
|                 |           | measured result. **1312's closed compare_op set is what [MSR-5]  |                             |
|                 |           | reuses**                                                         |                             |
| [EFF-1]         | 1369-1390 | allocates takes formal-rooted paths; heap and arena retire;      | [PROV-4], [PROV-3]          |
|                 |           | 1386 generalizes to a loan-bearing PARAMETER and to no other     |                             |
|                 |           | position, which [CALL-3] and [VIEW-7] depend on and which        |                             |
|                 |           | [LIV-2]'s footprint sentence is stated against (4a). **1369's    |                             |
|                 |           | canonical order (reads, writes, allocates) survives UNCHANGED    |                             |
|                 |           | and every row of A.2, 3.L and §4 is written in it; 1389's        |                             |
|                 |           | both-categories sentence survives and [PROV-4] reads it**        |                             |
| [EFF-2]         | 1392-1439 | the slice projection generalizes; 1427 becomes "the empty row    | [PROV-3], [PROV-6]          |
|                 |           | exactly when the walk spends no capability, and otherwise        |                             |
|                 |           | writes of each resolved provider". **1432's both-ways discipline |                             |
|                 |           | survives and [CALL-7] and [RES-8] reuse it**                     |                             |
| [ERR-3]         | 1472-1482 | the retained judgments gain [LIV-1]'s per-edge refusal           | [PROV-6]                    |
| [ERR-4]         | 1484-1488 | the deferral gains the two families that move inside. **1487     | [RES-7]                     |
|                 |           | survives and [PROV-5] depends on it**                            |                             |
| [PROG-3]        | 1505-1537 | PreStart materializes E at each item's figure and alignment,     | [RUN-4]                     |
|                 |           | matches its digest, creates every named stack, and makes         |                             |
|                 |           | StartFailed mandatory; ProgramFinished is named                  |                             |
| [DIAG-1]        | 1541-1883 | rank 5 covers the kernel domain; +container_declaration_ordinal  | [BLK-0]                     |
| [PAR-1]         | 1971-1998 | the provider-place projection (1975); the intervening-statement  | [RUN-3], [RUN-2]            |
|                 |           | FORM list (1981) becomes a footprint property while 1981's       |                             |
|                 |           | exit-edge and borrow-forming denials are KEPT as premises;       |                             |
|                 |           | a release enters a footprint; 1995's exhaustion sentence is      |                             |
|                 |           | unreachable when marked. **1993 survives and [RUN-1] depends     |                             |
|                 |           | on it: it is why the no-overlap obligation is over the           |                             |
|                 |           | permission JUDGMENT and not over an emitted construct**          |                             |
| [PAR-2]         | 2000-2033 | iteration-formed loans; a view's ranged write footprint in       | [RUN-3], [RUN-2]            |
|                 |           | logical coordinates. **2005 survives and [RUN-3] depends on it** |                             |
| [PAR-3]         | 2035-2061 | the exhaustion sentence; replicated places cannot occur marked   | [RUN-3], [RUN-2]            |
| [SYS-1]         | 2136-2162 | a fourth admitted declaration source                             | [BLK-0]                     |
| [SYS-2]         | 2164-2307 | views at the range-bearing operations; a derived "acquires from" | [VIEW-7], [RUN-1], [RES-6], |
|                 |           | column over its target-contract column; 2261's reserve_file      | [RES-7], [RES-9]            |
|                 |           | gains own ReserveOutcome and its three published relations       |                             |
|                 |           | [S33]; 2283-2285's proposition set gains                         |                             |
|                 |           | covered-store measures. **2270 is kept and [RUN-1] reads it**    |                             |
| [SYS-3]         | 2309-2311 | the kernel domain is admitted to every unit                      | [BLK-0]                     |
| [SYS-5]         | 2397-2432 | release-completeness (2397-2400) is KEPT; the release action     | [RES-9], [RES-7]            |
|                 |           | gains the handles subject; its target-contract column is a       |                             |
|                 |           | second source of [RES-7]'s derived column                        |                             |
| [SYS-7]         | 2473-2486 | the class set is UNCHANGED, which is why the Failed arm carries  | [RES-6]                     |
|                 |           | it and no route is conditioned on a class; the handle table's    |                             |
|                 |           | exhaustion is the Exhausted VARIANT instead [S33]                |                             |
| [SYS-8]         | 2488-2527 | the seven range-bearing operations take mut_slice and slice,     | [VIEW-7]                    |
|                 |           | each obligation over its own range-bearing parameter             |                             |
| [SYS-9,11,12,14]| 2529-2644 | their prose naming buffer<u8> is restated over views             | [VIEW-7]                    |
| [SYS-10]        | 2554-2574 | a reservation consumes one record of the named store `handles`   | [RES-9]                     |
|                 |           | with a published capacity, and the record returns by a stated    |                             |
|                 |           | closure                                                          |                             |
| [QUAL-2]        | 2369-2382 | +three failures: an unmet declared runtime-store requirement, a  | [RES-7], [RUN-1]            |
|                 |           | marked build whose permission judgment grants a [PAR] permission,|                             |
|                 |           | and a runtime that cannot exhibit release coverage or names a    |                             |
|                 |           | live stack E does not carry. **2369 survives and [RES-3] and     |                             |
|                 |           | [RES-9] depend on it** (4a)                                      |                             |
| [ENT-2]         | 2677-2728 | measure terms over a subscriptable place; +the measure datum;    | [MSR-1], [MSR-3], [LIV-2],  |
|                 |           | a set target resolving to no binding is a declaration event; a   | [MSR-2], [MSR-6]            |
|                 |           | const generic is admitted at an endpoint; +standing facts.       |                             |
|                 |           | **2681 clause (c) and 2693 survive and [MSR-6] and [MSR-3]       |                             |
|                 |           | depend on them**                                                 |                             |
| [ENT-3]         | 2730-2837 | +the enumerated source S13 and its five parts; S5 gains the      | [CALL-6], [BLK-0], [MSR-3], |
|                 |           | construct, rebind, payload and field placements; S6 generalizes  | [CALL-4], [LIV-2]           |
|                 |           | over four measures; S12's destination list gains four forms and  |                             |
|                 |           | the state-actual place                                           |                             |
| [ENT-5]         | 2863-2967 | descriptor-storage support; the effect-row kill; 2893(a) LOSES   | [MSR-2], [MSR-3], [CALL-5]  |
|                 |           | its element-position carve-out; the datum and the denotation     |                             |
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
| [INV-1]         | 3101-3156 | 3109-3113's atom admission gains terms, named consts and const   | [MSR-3], [MSR-5]            |
|                 |           | generics, and [MSR-3]'s atom-identity sentence. **3105 survives  |                             |
|                 |           | and [MSR-5] depends on it**                                      |                             |
| batch 0079      | docs/done/| the abort site loses its allocation caller and its release-walk  | [RES-6]                     |
| exhaustion floor| 0079-...  | callers, and the doubling-overflow arm with them                 |                             |
```

**Depended on and unchanged.** Each row is the collation of one or more `Depends:`
lines. A dependency that falls inside changed text, or that names a retired subject, is
on its changed row above and is not repeated here (condition 4).

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
| EFF-3      | 1441 | a pure call's deduplication is guarded by ownership, control, result and   |
|            |      | release proofs: [PROV-5] and [BLK-2], which is why a pure reserving or     |
|            |      | forming call is not deduplicated across a store's state                   |
| FN-6       | 1211 | recursion is permitted: [STK-2], which excludes a program from [RES-4]     |
|            |      | rather than rejecting it                                                  |
| PROG-1     | 1492 | one closed compilation unit with no function values: [PROV-4]'s exact      |
|            |      | reachability closure and [RES-8]'s composition claim                       |
| ENT-1      | 2661 | a retained witness changes diagnostic parent choice only, never the        |
|            |      | derivable set or acceptance: [RES-8], which is why saturation is declared  |
| ENT-4      | 2860 | L0's uniqueness and finiteness rests on the difference-bound shape:        |
|            |      | [MSR-2], which is why len + room = cap is an affine premise; and [BLK-0],  |
|            |      | which is why advance<T>(count) is one term rather than an expression       |
```

**META-5 delta**, declared here because the register is its natural home. Numbered
language rules: 132 at v0.42, plus the 51 of 3.K, none reusing a live or retired id;
3.K.0's two amendments are counted with their own batches and not here. Unique fixed
lowercase grammar atoms: minus 5 for the retired `heap` and `arena` effect atoms, the
retired `buffer` and `box` type productions and `slice_of` (`arena` is one atom serving
both a production and an effect entry, and retires once), plus 5 for `resource_closed`,
`dispose`, `linear`, `saturating` and `MutSlice`; net zero. Grammar productions: plus
2, being `clause_expr` and `dispose_stmt`; changed, 10, being `let_stmt`,
`return_stmt`, `set_stmt`, `result_binding`, `program_kind`, `struct_decl`, `enum_decl`,
`contract_block`, `effect`, `affine_factor`, with `requires_clause`/`ensures_clause`
counted once as a pair. **Statement forms** — a different count from productions — plus
1, `dispose_stmt`; the destructuring consume is a `let_stmt` alternative and the set
target list is a changed `set_stmt`. `ReservedLowerNames`: plus 3, `cap`, `room` and
`head`; [RES-9]'s six store designators are a closed set resolved inside a `saturating`
clause and enter no general lexical domain. Nominal types: plus 5, being 2 providers, 2
runs and `MutSlice`; `Slice` is unchanged. Declaration domains: plus 1, with one
`container_declaration_ordinal`. Entry input rows: plus 1. Compound punctuation tokens:
unchanged. [SYS-2]'s normative inventory counts change with [VIEW-7], [RES-6], [RES-7]
and [RES-9] and are recomputed when those rules are written into the spec, not asserted
here.

**Retired outright, with no successor.** The fourth draft's five owner types; its
`AppendView`, `absorb` and the abandoned-window disposition; its `update` statement and
its three atoms; its `Pool` store, `PoolSlot`, `PoolVector`, `seq_lease`, `pool_frame`,
`pool_extent`, `pool_take`, `pool_release` and the pool seam; its `FixedRing` and four
ring rows; its `HeapBox` and `ArenaBox`; its three failure structs and its `NoRecord`;
its `seq_filled`, `seq_vacant`, `seq_take_at`, `seq_clear`, `seq_truncate`,
`seq_reserve_heap`, `seq_reserve_arena`, `seq_shrink`, `seq_heap_filled`, `seq_push`,
`seq_try_push`, `seq_pop` and every `try` row; the `&uniq buffer<T>` and
`&uniq Container` prohibition **[CNT-7], whose effect [BLK-4]'s fourth clause restores
as a rule**; the effect-row atoms `heap` and `arena`; `slice_of`, `box_new` and
`arena_new`; the first draft's `Builder<'r, T>` and `[BLD]`; `[CNT-5]`; L14; the fifth
draft's `seq_frame` and `seq_exchange` rows, `[CALL-4]`'s exit datum and `[MSR-3]`'s
exit placement; the sixth draft's `[LIV-3]`, its `dispose ... using (...)` list and its
`Span`/`MutSpan` names; and **this draft's own two**: the seventh draft's `seq_rebase`
row, withdrawn to the library under L18, and its `on_propagate` section, rejected by the
owner. **Every rule id in that list is retired and none is reused.**

**Writer doctrine this design invalidates**, which `docs/patterns.md` must carry in
the same batch. **P16** ("One length fact above the writes") rests on hoisting a length
above a sequence of `&uniq` callee writes; [BLK-4] refuses the parameter it hoists
across, so the pattern is rewritten over `&uniq MutSlice<u8>`, where [CALL-3] keeps the
fact, and over the value-in / value-out form, where the fact is the result's. P16 gains
a second correction from [MSR-2] — a length fact survives a write to a **sibling
field**, which probe `r2_4` shows today's compiler killing. **P17**'s field-by-field
fold is unchanged for copy fields and gains [LIV-2]'s one commit rule for the rest.
**P19** is unchanged and gains a case: a measure term joins by the same delta-atom rule;
§6.4 records the join-shape asymmetry round 7 measured as a **compiler** investigation
and not a doctrine change. **P15** is unchanged and both worked programs follow it.
**P8** should gain what probes `q5'`, `m10` and `x1b` bought: an exact `-` or `+`
carries an ordering into a backedge where the wrapping form gives the checker a fresh
atom. **Six new patterns are owed**: the scope that holds the capability and gets the
derived release (D3), the linear destructuring consume, 3.L.3's two-invariant
construction loop plus its `flat` invariant, the value-in / value-out helper whose
contract is complete over every measure it hands back, the element borrow that hoists a
window's modulo out of a descriptor loop (probe `x10` shows it unsupported today), and
the checked release that a caller proves away.

---

### 3.S Surface decisions

**This section is a decision record.** On 2026-09-03 the owner decided every
language-surface addition this design rests on, and on 2026-09-04 the last three entries.
The rules of 3.K use those spellings as **decided**, and **nothing in this file is
proposed**.

**The PROPOSED list, at a glance: none.** It held `seq_reslice` [S31], a linearity bound
on a generic parameter [S32] and `ReserveOutcome` [S33] until 2026-09-04, and each is
recorded below with the disposition the owner gave it.

**The decided list.** Seven entries changed status this round and are marked, the last
three of them on 2026-09-04.

```text
| id  | spelling                                    | kind                    | status    |
|-----|---------------------------------------------|-------------------------|-----------|
| S1  | Vector<'s, T>, brand elided                 | compiler-owned nominal  | ADOPTED   |
| S2  | FixedVector<T, n>                           | compiler-owned nominal  | ADOPTED   |
| S3  | Heap<'s>                                    | compiler-owned nominal  | ADOPTED   |
| S4  | Arena<'s, bytes, align>                     | compiler-owned nominal  | ADOPTED   |
| S5  | slice<'r, T> keeps its v0.41 name           | naming decision         | see S35   |
| S6  | mut_slice<'r, T>                            | compiler-owned nominal  | see S35   |
| S7  | seq_fixed, seq_arena, seq_arena_proved,     | operation names         | ADOPTED   |
|     |   seq_heap                                  |                         |           |
| S8  | seq_place, seq_place_front, seq_take,       | operation names         | ADOPTED   |
|     |   seq_take_front                            |                         |           |
| S9  | arena_frame, arena_extent                   | operation names         | ADOPTED   |
| S10 | seq_slice, seq_mut_slice                    | operation names         | ADOPTED   |
| S11 | cap, room, head                             | operation names         | ADOPTED   |
| S12 | dispose p;                                  | statement form          | ADOPTED   |
| S13 | let N(f1: b1, ..., fk: bk) = move v;        | let alternative         | ADOPTED   |
| S14 | (retired into D2)                           | -                       | DECIDED   |
| S15 | (retired into D2)                           | -                       | DECIDED   |
| S16 | -> (a: own T, b: own U), let (a, b) = ...,  | result list and its     | ADOPTED   |
|     |   return e1, e2;                            |   binding and return    |           |
| S17 | clause_expr over measure terms              | grammar production      | ADOPTED   |
| S18 | linear struct N { ... }                     | declaration modifier    | ADOPTED   |
| S19 | resource_closed command fn main             | entry marker            | ADOPTED   |
| S20 | struct N['s] { ... }                        | region params on a      | ADOPTED   |
|     |                                             |   nominal               |           |
| S21 | a const generic as a value, an endpoint     | resolution admission    | ADOPTED   |
|     |   and a clause operand                      |                         |           |
| S22 | command.heap as heap: own Heap              | entry input row         | ADOPTED   |
| S23 | allocates(path)                             | effect production       | ADOPTED   |
| S24 | ensures when b is V(f: r): ... over any     | contract routes         | ADOPTED   |
|     |   variant and any result ordinal            |                         |           |
| S25 | reserve_file -> own Result<FilePermit, ..>  | system-row change       | ADOPTED,  |
|     |                                             |                         | then S33  |
| S26 | saturating(d) over a store DESIGNATOR       | contract clause         | ADOPTED,  |
|     |                                             |                         | AMENDED   |
| S27 | Slice<'r, T> is copy; mut_slice is affine   | ownership class         | ADOPTED   |
| S28 | on_propagate { ... }                        | scope section           | REJECTED  |
| S29 | seq_rebase                                  | operation row           | WITHDRAWN |
| S30 | the seven [SYS-8] range-bearing operations  | system-row change       | ADOPTED   |
|     |   take mut_slice and slice                  |                         |           |
| S31 | seq_reslice                                 | operation row           | REJECTED  |
|     |   (the reborrow is [OWN-6]'s, [VIEW-6])     |                         |           |
| S32 | a linearity bound on a generic parameter    | generics surface        | ADOPTED   |
| S33 | reserve_file -> own ReserveOutcome          | system-row change       | ADOPTED   |
| S34 | array<T, n> retires; FixedVector<T, n> is   | type retirement         | ADOPTED   |
|     |   the one fixed run, const-eligible full    |                         |           |
| S35 | Slice<'r, T>, MutSlice<'r, T>               | naming decision         | ADOPTED   |
|     |   (supersede S5 and S6's spellings)         |                         |           |
```

**Two entries decided 2026-09-04, after B1 landed, when the owner asked why
`array<T, n>` had survived the redesign.** The seven falsifier rounds asked whether the
design was sound, closed, consistent, writable and linear; none asked whether the old
surface was completely replaced, and 1.4's partition test asks what must enter the kernel,
not what must leave it. `buffer`, `box` and `arena` retired because their semantics
blocked a rule; `array` blocked nothing and was kept in one sentence of [BLK-1] with no
ground. **S34**: `array<T, n>` retires with its `array_new` row. It was exactly the
`len = cap = n`, `head = Z` case of a run, which A.1 already tabulated as four exact
constants, so a `FixedVector<T, n>` whose four measures are standing facts is that case
with no runtime descriptor word: a `const` of `FixedVector<T, n>` type with exactly `n`
literal entries is the const-eligible form [CONST-1], lowers to element storage only, and
materializes its descriptor from the standing facts at each use; a subscript's `i < len`
discharges from `len = n`. One fixed run, one spelling. **S35**: every compiler-owned
container, store and view nominal is capitalized — `Vector`, `FixedVector`, `Heap`,
`Arena`, `Slice`, `MutSlice` — and only the primitive types stay lowercase. This
supersedes S5's name (kept on 2026-09-03 because only semantics earn a rename) and S6's
spelling; the operation names `seq_slice` and `seq_mut_slice` [S10] are unchanged. Every
normative section of this file now writes `Slice` and `MutSlice`; section 6 quotes
probes as they were run, in the old spelling.

**The decided entries, one ground each.** **S1-S2**: `array<T, n>` requires `n` live
values, which for affine `T` is exactly what a writer building a run does not have, and
`buffer<T>` is heap-only, has no affine element domain (probe `p9`) and carries no store
brand; `buffer<T>` retires. **S3-S4**: L2 makes a store a value a program must hold and
no wf declaration produces an unforgeable one — a writer's `struct Heap {}` is
constructible; **under D3 they carry a second job**, since the presence of one in a
signature is what makes the derived release available in that scope. **S5-S6**:
[SET-1] 488-490 makes every slice-rooted target unwritable, so no writable view exists
and a system operation cannot fill a caller's run without taking the run (probe `p7`);
`slice` kept its name on 2026-09-03 because **only semantics earn a rename**; S35 renames it `Slice` on 2026-09-04 for the one-rule naming scheme. **S7-S11**: each moves a
checker-maintained boundary, mints a store, forms a view, or reads a measure no run
exposes; no rule reads a name, so the scheme is a readability choice. **S12**: a
store-backed value's release is a structural walk of a type the writer did not declare,
and **under D3 `dispose` is the early release** — the difference between a peak of one
run and a peak of two in `bs_reserve`, and between one and `n` in a loop whose scope is
the whole program; its capability is determined by the brand and never written.
**S13**: linearity is closed under ownership, so a value linear in this scope must be
takeable apart in one statement that leaves no residual, which is [PROV-6]'s
partial-consume fix (Q16 records the `...` convenience). **S16**: R1 makes every
transforming operation return the value it was handed plus what it computed; **its L18
status is recorded honestly** — a two-field struct per operation is writable in wf, so
this half of [CALL-4] is admitted on cost and not on expressibility, while the
per-variant route and the S12 destination clause are what no wf program has. **S17**:
[GRAM-5]'s `atom` has no `call` alternative, so `len(source) <= room(out)` derives
nowhere (probe `q7`), and a `define` is erased by alpha-expansion so it cannot name a
**result**'s measure (probe `x2`). **S18**: the capability criterion sees storage
obligations and not logical ones; 3.L.7 states what the modifier buys, and its admission
condition — an affine nominal, never a tag-only enum (probe `q11`) — is [PROV-6]'s.
**S19**: the judgment must be a compile error for the program that asks for it and a note
for every other, and a compiler flag would make acceptance a function of the invocation.
**S20**: a store's identity is in the type [PROV-1] and a nominal holding a store-backed
value must name that store; probes `r2_6` and `m05` are the parse errors today.
**S21**: every capacity-parametric function reads its bound as a value, a loop endpoint
or a clause operand (probe `q10`). **S22**: the heap must enter as a value and [FN-7]'s
entry table is closed; **`main` declares no region parameter**, and [PROV-1]'s brand
resolution is what makes that decision consistent — without it every hosted helper is
undeclarable. **S23**: [EFF-1]'s fixed atoms cannot name a provider received as a field
of an aggregate, so [PROV-4]'s closure would be inexact exactly where a program threads
an environment struct. **S24**: [FN-9] 1307 admits exactly `when Ok(value: r):` over
`Result<int, E>`, so no library constructor can publish a fact about what it built
(probes `x1`, `x2`, `x13`); adopted **with the ordinal binder** [CALL-4] requires.
**S27**: duplicating a shared view is a second **shared** loan, which [OWN-5] admits
without limit, and a loan-bearing value owns nothing; two costs follow — a copy view is
never consumed, so its loan ends at its **last use** [PROV-3], and [VIEW-4] must refuse a
`set` at a loan-bearing target.

**D1, D2 and D3 removed entries rather than adding them.** `[S18]` is adopted together
with derived linearity, so [PROV-6] states one criterion and one modifier and the writer
marks only a logical obligation. `[S14]` and `[S15]` are retired into one commit rule.
And **D3 removed the need for `[S28]` before the owner rejected it**.

**S25, `reserve_file` becomes fallible. ADOPTED.** The handle table is a covered store
with a finite capacity [RES-9] and L3 requires its refusal to be a value; a total
`reserve_file` over a proved capacity was the alternative, costing one header invariant
per loop at eleven corpus call sites. **What round 7 found, recorded here rather than in
a report:** with `Result<FilePermit, IoError>` the store's *exhaustion* is a **class** of
the error payload, not a variant, and no route in [CALL-4] is conditioned on a class — so
the `Err` edge publishes only `len(factory) = <call datum>` and no marked program can
derive `room(factory)` after a refusal. **S33 repairs that and is adopted**, so what
[SYS-2] 2261 declares is the outcome nominal and not the `Result`; S25 stands as the
decision that made the operation fallible at all, and [RES-6] states the relation each arm
publishes.

**S26, `saturating(d)`. ADOPTED, and AMENDED this round.** [RES-10]'s reusable-capacity
route must compose across a call, and the fact it needs — *this function performs no
acquisition on `d`'s store that could succeed when that store is full* — is a property of
a body, which [CALL-5] forbids a caller to derive; deriving it from the body, as the
fifth draft did, makes [CALL-5] false. **The amendment**: its operand is a **store
designator**, either a region name in scope or one of [RES-9]'s six spec-fixed
runtime-store names. Keyed to a store **region**, as the seventh draft had it, the clause
could not be written for any domain the route that reads it applies to, because every
reusable-capacity domain in [RES-5] is a **runtime** store whose identity is not a region.

**S28, `on_propagate { ... }`. REJECTED by the owner, and removed from this file.** It
proposed one section per scope whose statements run on every `propagate` error edge
leaving that scope. *Why it is rejected, measured rather than argued:* `propagate` is
**7 of `decode_dynamic`'s 30 abnormal edges** and 14 of its 68 disposals, so a relief for
`propagate` alone leaves four fifths of the cost; the live linear set changes **four
times inside that function's own top-level scope**, at each `move` of a run into a
callee, and a scope boundary cannot be placed there, so [LIV-1]'s "exactly the set the
section discharges" forces about four artificial wrapper blocks and five sections —
roughly 32 lines to remove 14; an inner and an outer section each pass their own
per-point check and **each run on the same edge**, which is a double free; the section
admits "ordinary written statements", which by its own words includes `return`, `break`
and a nested `propagate`, none of which it defines; and after [BLK-4] every one of those
seven sites is a **multi-result call**, which [ERR-3] 1472 cannot propagate at all. **And
D3 removes the problem the section was for**: those scopes hold the capability, so the
derived release runs on the `propagate` edge [STOR-3] 690. What remains of Q10 is the
smaller question of whether `propagate` should reach a multi-result call at all.

**S29, `seq_rebase`. WITHDRAWN to the library.** It proposed one added [BLK-3] row
publishing `head(result) = 0_u64` with `len`, `cap` and `room` unchanged. *Why it is
withdrawn:* L18 asks whether a writer can express the effect, and **round 7 wrote the
program** — drain the wrapped run front-to-back into a fresh `seq_fixed::<T, n>()` under
the `flat` invariant every construction loop already carries, and the result has
`head = 0` with `len` and `cap` preserved. The seventh draft's own alternative (c),
"keep the permanent staging run", *is* that program, so the entry priced its own
alternative and then denied the alternative exists. 3.L.8 writes it and prices it: one
extra run of the same capacity for the life of the rebase, and the same O(len) copy the
rotate would have performed. **Two things are recorded rather than hidden.** The memory
cost is real and is in `E`: a driver that rebases a 256-byte ring needs a second
256-byte run live across the drain, which for a `FixedVector` is a frame contribution and
for a `Vector<'s, u8>` is a take from its store. And a real ring driver does not rotate
at all — it hands the host two `iovec`s over the two halves of the wrapped window, and
this language has no spelling for a view of two ranges. Q18 puts the kernel row back to
the owner if `E` cannot afford the second run.

**S30, the seven [SYS-8] range-bearing operations over views. ADOPTED.** `read_at`,
`write_once` and the five others take `&uniq 'd MutSlice<'r, u8>` for a destination and
`&'s Slice<'r, u8>` for a source in place of `buffer<u8>` [VIEW-7]. *Needed because* it
is goal A's container half: without it a heap-free program cannot do I/O, since
`buffer<u8>` is heap-only, and no wf program can change a [SYS-2] declaration record.
*Alternatives:* (a) do not change them — a marked program has no I/O at all; (b) take the
destination `own` and hand it back — correct under R1 and it deletes the loop of reads a
caller writes into one destination, because an `own` destination is consumed by the first
call; (c) take the run itself — reintroduces the `&uniq` container parameter [BLK-4]
refuses. *Cost:* seven signature rows, [SYS-2]'s normative counts, and the prose of four
[SYS] rules. *Decided:* adopted. Round 7 found the adopted form incomplete in one place,
and **[S31] closes it without a row**.

**S31, `seq_reslice`. NOT ADOPTED as an operation, and the capability is admitted without
one (owner-decided 2026-09-04).** The proposal was one added [VIEW] row,
`seq_reslice['r, T](window: &MutSlice<'r, T>) -> own Slice<'r, T>`. *The gap it was for*
is real: a helper handed `&uniq MutSlice<u8>` can fill its destination and, under S30
alone, could not publish it, because `write_once` wants a `&slice`, A.2's `seq_slice`
forms a view from a **run** borrow and not from a view, and forming a second loan on the
run itself is [OWN-5]'s ordinary conflict (probe `s6`). *The owner's ruling:* forming a
shared `Slice<'r, T>` from a `MutSlice<'r, T>` is the **ordinary shared child reborrow of
a unique loan** that [OWN-6] 613-627 already admits for places, and a probe on the v0.42
build accepts `peek(x: &deref(x))` inside a region block where `x: &uniq u64`. So this
design states it as **that rule applied to views** and not as a kernel row: a `Slice`
formed from a `MutSlice` carries the parent's origin set and range, its loan is a shared
child of the exclusive one under [OWN-6], and the parent cannot be written while the child
lives. *Why a row would have been wrong:* two spellings for one semantics, which 3.K.10
exists to prevent, and [VIEW-6]'s ceiling already contains the child, whose origin is the
parameter's own formal-view origin. *What it settles:* [VIEW-6] records **one**
restriction and not two, the fill-and-publish helper is writable, `wfgrep`'s `publish_all`
call site stays where it is instead of moving back inside `search_file` and `walk`, and
A.2 keeps twelve rows. *What it does not settle:* Q19's alternation cost, because a shared
child still forbids a write of the parent while it lives.

**S32, a linearity bound on a generic parameter. ADOPTED (owner-decided 2026-09-04).**
`fn f<T: affine>(...)`, `fn f<T: linear>(...)` and `fn f['s: affine](...)`, read at the
declaration and checked at the instantiation. *Needed because* a value's release
disposition depends on its type and region arguments, and the language has no position at
which a writer can say which
disposition a generic body was written for — so this design fails closed three times and
each refusal costs a program a writer needs. **On the region axis**, [PROV-6]'s
declaration obligation refuses a **consuming** helper over an unconstrained region —
`fn checksum['s](v: own Vector<'s, u8>) -> sum: own u64` must hand `v` back or take a
provider it cannot name. **On the type axis**, no generic body can serve an affine and a
capability-released instantiation from one body: at `T = u64` a body containing a release
is refused because the type reaches no capability leaf, and at `T = Vector<u8>` a body
without one is refused because the value reaches a scope exit — and the two bodies need
**different signatures**, since one needs `heap: &uniq Heap` and the other cannot use it.
3.L.2's `clear`/`truncate` row describes exactly that function. **And [BLK-4]'s fourth
clause** refused a `&uniq Holder<T>` outright for the same reason. *Alternatives:* (a) do
not add it — costs the three refusals above, each with a stated diagnostic, and the
library writes two functions with two signatures wherever it can; (b) per-instantiation
checking — makes one declaration have two verdicts, which is the defect rounds 6 and 7
both found; (c) a whole-design retreat to a runtime disposition field — a value a program
can forget to read, which is what the modifier exists to replace. **The grammar note:**
`gparam` gains an optional `: affine` or `: linear` bound and a member of `region_params`
gains the same, both written and never inferred, which keeps [FN-2] 1124's
always-written discipline; the bound is a linearity class and not a user trait, so no
trait surface arrives with it. *Cost:* that one production, one instantiation check, and
one sentence in [PROV-6] and [BLK-4] each reading the bound. *Decided:* adopted, and the
verdict stays one per declaration, which is what refuses (b). **What it does not settle:**
Q8's other two halves, so 3.L still writes `clear` and `truncate` as **two bounded
generics** rather than one function, and `filled` and both `try` forms still meet the
copy/affine wall.

**S33, `reserve_file -> own ReserveOutcome`. ADOPTED (owner-decided 2026-09-04)**, in
place of S25's `Result`. A three-way system outcome:

```text
reserve_file(factory: &uniq FileFactory) -> outcome: own ReserveOutcome
  Reserved(value: FilePermit):  len(factory) = <call datum> + 1
  Exhausted():                  room(factory) = 0, len(factory) = <call datum>
  Failed(error: IoError):       len(factory) = <call datum>
```

*Needed because* the handle table is a covered store whose refusal L8's second half
reads, and under S25 that refusal is an `IoError` **class** rather than a variant: no
route in [CALL-4] is conditioned on a class, publishing `room(factory) == Z`
unconditionally over `Err` is false for a `PermissionDenied` at a table that is not full,
and there is no `when Err(error: e) is ResourceExhausted:` form anywhere. *Why no wf
program has it:* it changes a [SYS-2] declaration record. *Alternatives:* (a) do not
change it — the store's exhaustion fact stays unpublishable and [RES-6] can only say so;
(b) add a class-conditioned route form — a new route family for one relation, and it makes
a portable class set into proof vocabulary, which [SYS-7] 2473-2486 exists to prevent;
(c) a total `reserve_file` over a proved capacity — S25's own rejected alternative.
*Cost:* one system nominal, one row change, [SYS-2]'s counts, and eleven corpus call sites
gaining a third arm. *Decided:* adopted; it is the same partition this design draws
everywhere else — a failure the environment can produce is a typed value, and a failure of
a store we account for is a variant with a published post-state. **What it settles:**
[RES-6] publishes `room(factory) = 0` on the `Exhausted` arm through [CALL-4]'s existing
per-variant route, L8's second half is readable for that store, and Q20 keeps only the
general question of writing the partition once as a rule about covered stores.

### 3.L The library, written in wf

#### 3.L.0 How to read this section

Everything below is **ordinary wf**, written against 3.K and against the unchanged v0.41
rules. It defines no rule, amends no rule, and is named by no rule. It exists to discharge
L18's obligation in both directions: an item the kernel no longer carries is written out
here, or 3.L.6 says which primitive the kernel lacked. Every spelling it uses is one 3.S
records as decided. Each item states its **proof route** — which kernel rule discharges
each obligation, and which of those v0.41 already proves today, naming the probe where
one exists.

Seven discipline sentences are stated once here rather than repeated:

- **Every body is three-address.** `let mirror = count -wrap 1_u64 -wrap at;` is two
  operations in one expression and is a [GRAM-4] parse error (probe `t13`).
- **`Z` is the term language's zero and appears only in rule prose.** wf source and every
  inventory row write `0_u64`.
- **An effect row is written in [EFF-1] 1369's canonical order, `reads`, `writes`,
  `allocates`**, and a function that writes an `own` parameter and hands it back must
  carry `writes` of it (probe `c8a`).
- **A measure read is `pure` at the operation and an ordinary `reads` at the caller**
  (probe `t10`); [EFF-2] 1432 admits no wider and no narrower declaration.
- **A `replace` is a kill and never a publication** [SET-2] 528, so a value obtained by
  one carries **no measures at all** and a function returning one is refused by [CALL-7].
- **Every type and const argument of a user generic is written** [FN-2] 1093-1100, probe
  `q4`; a region argument is written exactly where [FORM-8] writes it; a compiler-owned
  row writes what no operand of it supplies [BLK-0].
- **A borrow of an outer binding inside a loop body is written bare** (D4, 3.K.0),
  because the loop body is an implicit region block. That amendment has **not** landed:
  probe `q2` shows the amended build refusing the bare form and `q3` the explicit block
  compiling.

**And one obligation this section is checked against**: [CALL-7]. Every function below
that hands a measured value back declares, for each measure of it on each route, one
clause the rule admits; round 7 found five of the seven functions the seventh draft
printed failing that check, and 6.11 lists what the re-run added.

#### 3.L.1 The owner names

`FixedVector<T, n>` is the kernel type and needs no library. `HeapVector<T>` and
`ArenaVector<'a, T>` are what a writer *calls* a `Vector<'s, T>` whose store is the heap
and a named arena; they are one kernel type at two regions (footnote 1). Under [PROV-1] a
heap run in a stored position is written `Vector<u8>` and an arena run `Vector<'a, u8>`,
which is the whole visible difference. **A ring is not a library type at all**: under
[BLK-1]'s window a ring is a `FixedVector<T, n>` used from both ends (footnote 2).

#### 3.L.2 The partition, item by item

Every item is written in wf in `CONTAINERS.md` §3 against 3.K, with its proof obligations
walked there. This table is the result; the items §4 calls are written out below, because
a worked program may not call a function this file does not declare.

```text
| item                          | written as                          | route, and what discharges it       |
|-------------------------------|-------------------------------------|-------------------------------------|
| FixedVector<T, n>             | the kernel type itself              | nothing to write                    |
| HeapVector, ArenaVector       | Vector<'s, T> at two regions        | nothing to write                    |
| a ring, a queue, a deque      | a run used from both ends [BLK-1]   | nothing to write; no Option, no tag |
| return a wrapped window to    | a drain into a fresh run, 3.L.8     | seven invariants; two runs live     |
|   its origin                  |                                     | across the drain; [S29] withdrawn   |
| vacant<T, const n>            | a counted loop of seq_place over    | three header invariants; the exit   |
|                               | None<T>(), 3.L.3 below              | ordering, not an equality; x1c, x1d |
| filled<T, const n>            | the same, reusing one copy value    | as above; per element class (Q8)    |
| the transposition of one      | seq_take, one element replace,      | three statements; below, and its    |
|   element with the last       | seq_place                           | requires is at + 2 <= len           |
| take_at                       | the transposition, then seq_take,   | the requires plus a dominating      |
|                               | with a branch for the last position | branch; NON-MEASURED T only         |
| clear, truncate               | a counted drain, two invariants     | two bounded generics, T: affine and |
|                               |                                     | T: linear [S32]; Q8's copy wall     |
| growth policy, HeapVector     | seq_heap, drain from the front,     | seven invariants; the window is what|
|                               | append at the back, construct       | makes order preservation free; 3.L.5|
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

**The transposition, written out, because it is the fifth draft's removal.**

```wf-design
fn take_at<T, const n: u64>(vector: own FixedVector<T, n>, at: own u64)
    -> (rest: own FixedVector<T, n>, taken: own T)
    reads(vector), writes(vector) contract {
  requires at + 2_u64 <= len(vector);
  ensures len(rest) + 1_u64 == len(vector);
  ensures room(rest) == room(vector) + 1_u64;
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
**not** `at + 1_u64 <= len(vector)`, which over `u64` is the same proposition as
`at < len(vector)`. The consequence is real: this form cannot address the **last**
position, where the transposition is the identity, so a caller that may remove the last
element writes a dominating branch and a plain `seq_take` on the other arm. **And it is
declarable only for a non-measured `T`**: `old` comes out of a `replace`, which publishes
nothing, so at a measured `T` the `taken` result has no measures, no clause [CALL-7]
admits exists for it, and the function is refused at its `fn_decl`. `cap` needs no clause
because it is the type constant [CALL-7] excludes.

#### 3.L.3 Construction and appending, written out

```wf-design
fn vacant<T, const n: u64>() -> result: own FixedVector<Option<T>, n> pure contract {
  ensures len(result) >= n;
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
completeness sentence — and each denotes what it reads as, because `seq_fixed` has no
operands and every relation is over its result [MSR-3]. `grown`'s base is `0 >= 0`;
`spare`'s is `n + 0 >= n`; `flat`'s is `0 <= 0`. `seq_place`'s own requirement
`room(built) > 0` discharges from `spare` and the counted loop's `at < n` ([ENT-3.S11])
by [MSR-4] step 5. On the backedge `seq_place` declares `len(result) = len(vector) + 1`,
`room(result) = room(vector) - 1` and `head(result) = head(vector)`, **each over that
call's own call datum because `vector` is an `own` parameter** [MSR-3], reaching `built`
through [CALL-6]'s S13 and [CALL-4]'s `set`-target destination; each invariant is
preserved by **one** published premise, which is what puts the derivation inside
[ENT-6] 3015's two-premise budget (probes `g4`, `g3`). The `set` target names a binding in
scope, so it keeps its term [LIV-2]. At the exit `at = n`, so `len(built) >= n`;
`room <= 0` follows from `len >= n`, `cap = n` and [MSR-2]'s identity; and `flat` exports
`head(built) <= 0`. **`cap` needs no clause**, being the type constant [CALL-7] excludes.

**`flat` is what makes anything built by a loop viewable.** [ENT-5] 2942-2946 removes
every fact whose support the body writes at the backedge, so the `head = 0` chain is
exact inside straight-line code and gone across a loop; one invariant, one clause, base
and backedge each one published premise. A run that is never viewed omits both.

`n` is read as a loop endpoint, which is [MSR-6] and probe `q10`'s rejection today.
`vacant` is generic over `T` with no copy bound, because `None<T>()` is built fresh each
iteration. `filled` is not, because it reuses one `value`:

```wf-design
fn filled<T, const n: u64>(value: own T) -> result: own FixedVector<T, n> pure contract {
  ensures len(result) >= n;
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

Same route, written for a **copy** `T` only: the bare `value` use is [OWN-1] 564's
copy-on-use, and at an affine instantiation the same body needs `move` and would consume
it on the first iteration. That is Q8, and [S32], now adopted, is the relief on the
linearity axis only: each body declares the class it was written for, while the
copy/affine half stays Q8's. This is the function [VIEW-7] needs for an addressable I/O
destination.

**`collect`, the one program every draft has carried.**

```wf-design
fn collect['s](out: own Vector<'s, u8>, source: own Slice<u8>)
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

`collect` writes **one** region name, `'s`, at its binder and at the two positions whose
store must be the same one; `source`'s loan region relates nothing and is elided, and so
is `'s` at every call site, because the `out` operand determines it. One written
identifier per hand-back helper is R1's whole spelling cost.

**Proof route, and what [CALL-7] costs here.** The three `let`s are [ENT-3.S6]
equalities over the live terms generalized to the four measures [BLK-0], and at that
point each live term equals its entry datum [MSR-3], so the `requires` transports into
the loop's base: `spare_lo` at `at = 0` is `room(out) >= before_room`, the equality.
`seq_place`'s `room > 0` discharges from `spare_lo`, `before_room >= count` and
`at < count` by [MSR-4] step 5 (probes `k21`, `k21b`). Each of the five invariants is
preserved by exactly one published relation. At the exit `at = count` and the four
two-sided invariants give the two exact `ensures`; `cap(rest) == cap(out)` follows from
[MSR-2]'s identity and needs no invariant, but it does need a clause, because `cap` of a
`Vector<'s, T>` is a measure and not a type constant. **`collect`'s `'s` is
unconstrained and it hands `out` back as `rest`, so [PROV-6]'s declaration obligation is
discharged by the first of its four routes** and the function is declarable at a heap
`'s` and at an arena `'s` alike. **The cost is the two-sided pairs**, [CALL-7]'s honest
price: an exact measure relation costs **two** header invariants, because [INV-1] 3105
admits the four ordered relations and not `==`. Q14 records the change that halves it.

#### 3.L.4 The pool and the two `try` forms, written out

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
  ensures when Some(value: pool): room(pool.free) <= 0_u64;
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
```

**`pool_new`'s `None()` arm is legal, and [PROV-6]'s declaration obligation is what says
so.** On that arm `free : FixedVector<Vector<'s, u8>, 8>` is live and holds up to seven
arena-backed runs; it is not moved out and not destructured. The obligation admits it
because **`'s`'s store class is fixed by this declaration's own `Arena<'s, ...>`
parameter**, so every `Vector<'s, u8>` is affine and the run takes the ordinary
compiler-derived release on that edge.

```wf-design
fn pool_take['s](pool: own BlockPool<'s>)
    -> (rest: own BlockPool<'s>, leased: own Option<Lease<'s>>)
    reads(pool.free), writes(pool.free) contract {
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

**`pool_release` is the *proved* spelling, and that is what makes the return
unavoidable.** A checked one — `-> (rest, unreturned: own Option<Lease<'s>>)` — has a
mandatory refusal arm, and the only thing a writer can do on it is
`let Lease(run: orphan) = move lost;`, a legal destructuring consume that throws the
block away: that is `linear` behaving correctly and it is not must-return. The proved
spelling's `requires room(pool.free) > 0_u64` is discharged at the call site from
`pool_take`'s own `when leased is Some(value: got): room(rest.free) >= 1_u64` — one
published premise — so **there is no refusal arm and the lease has exactly one route on
every path**. `cap(rest.free)` needs no clause; it is the type constant.

`pool_take` cannot state `room(got.run) >= 256_u64`, because a `Vector<'s, u8>` carries
its capacity as a measure and not in its type, so putting one into a `FixedVector`
element and taking it out loses the figure `pool_new` established. `got.run` is therefore
outside [CALL-7]'s population — neither constructed by this function nor received as an
`own` parameter and returned — and a caller that needs room reads it and branches, once
per lease. That is the honest price of the pool being library data, and 4.1 pays it in
the open.

```wf-design
fn try_place<T, const n: u64>(vector: own FixedVector<T, n>, value: own T)
    -> (rest: own FixedVector<T, n>, unplaced: own Option<T>)
    reads(vector), writes(vector) contract {
  ensures head(rest) == head(vector);
  ensures len(rest) <= len(vector) + 1_u64;
  ensures len(rest) >= len(vector);
  ensures room(rest) <= room(vector);
  ensures room(rest) + 1_u64 >= room(vector);
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
  ensures head(rest) == head(vector);
  ensures len(rest) <= len(vector);
  ensures len(rest) + 1_u64 >= len(vector);
  ensures room(rest) >= room(vector);
  ensures room(rest) <= room(vector) + 1_u64;
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
written per element class where the body moves a `T` (probes `x14`, `x15`; [S32] is
adopted and relieves the linearity axis, and Q8 keeps the copy/affine half). **Their
`len` and `room` bounds are two-sided**, which is round 7's addition:
the seventh draft published one side of each, which satisfies no caller and, under
[CALL-7] as stated, no longer satisfies the rule either.

#### 3.L.5 The growth policy, and what a hosted program pays under D3

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
  ensures when Some(value: fresh): room(fresh.v) <= 0_u64;
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
  ensures when Refused(value: back): cap(back.v) == cap(s.v);
  ensures when Refused(value: back): room(back.v) == room(s.v);
  ensures when Refused(value: back): head(back.v) == head(s.v);
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

**Proof route.** `seq_heap` publishes all four measures of `built` on its `Some` arm
[BLK-0], and they reach `built` through [CALL-6]'s S13 at the arm binder [CALL-4].
`left` and `gone` bound the source, `made_*` and `spare_*` the destination, `flat` the
head; each is preserved by exactly one published relation of `seq_take_front` or
`seq_place`, whose `vector` operand is `own` and therefore denotes that call's call datum
[MSR-3]. At the exit `at = count`, so `len(built) = count = len(s.v)` at entry,
`room(built) = total - count`, and `cap` falls out of [MSR-2]'s identity at `total`.
**The tail constructs rather than replaces**, which routes `built`'s measures into the
result through [MSR-3]'s construct placement; a `replace` would publish nothing. **The
`Refused` arm's four clauses are round 7's addition**: `back` is `s`, received as an
`own` parameter and returned, which is [CALL-7]'s population verbatim, and all four are
`== ...(s.v)` because nothing on that arm was written.

**`set (s.v, byte) = seq_take_front(vector: move s.v);` is a consume of a proper
sub-place of `s`**, and it is legal for two reasons stated at their rules: under D3
`Bytes` is **affine** in this scope, because `bs_reserve` holds `heap: &uniq Heap`; and
even for a value linear in its scope, [PROV-6]'s partial-consume refusal excepts a
sub-place the same statement's commit reinitialises.

**`dispose old;` is the early release, and it is the only `dispose` in the library.**
Under D3 `old` is affine here and its compiler-derived release would run on the arm's
leaving edge anyway; the statement moves it **before** the new run is handed back, which
is the difference between a peak of `count + total` slots and a peak of `total`. It names
no capability: `old`'s brand is the entry heap's store region [PROV-1], that store's
provider type is `Heap`, and the innermost live binding of that type is the `heap`
parameter reached through its borrow — which is why `bs_reserve`'s row carries
`writes(heap)`.

`bs_shrink` is the same function with `total < count` and `requires total <= len(s.v)`,
with the drain bounded by `total`. Its `dispose old;` then releases a run still holding
`count - total` elements, and that is **correct**: [PROV-6]'s walk visits a container's
elements before its backing.

**What a hosted program pays, recounted under D3.** `byte_string.wf` has exactly one
store, so nothing in it names a region: the whole region parameter list leaves every
struct and signature, fifteen brand occurrences leave the written types, and twelve
call-site brand arguments go with them. `Bytes` is capability-released, so **every scope
that holds it must hold the `Heap`** — which `main` does by its entry row and every
helper does by its parameter — and the release is derived on all eleven of `main`'s
return edges with **no writer statement on any of them**. The seventh draft counted five
`dispose` statements and the per-edge obligation actually required **forty**; this draft
requires **one**, the early release above. Of the roughly twenty-two writer-visible items
the program then carries, six are provider parameters, five are `match`es on a typed
refusal, nine are result-list binder groups and two are destructuring consumes; the way
to carry fewer is an arena, whose values are affine everywhere.

#### 3.L.6 What the partition test found the kernel lacked

Nine, each named with the library function that demanded it and the probe that shows it
is new capability rather than a compiler defect.

```text
| # | kernel addition                      | demanded by                       | today                 |
|---|--------------------------------------|-----------------------------------|-----------------------|
| 1 | the one `set` commit rule over a      | collect, bs_reserve, pool_take,   | q9, x5, t8, x2, x3    |
|   | place that is not a bare binding      | vacant, filled, clear, try_place  | REJECTED [STOR-1]     |
|   | [LIV-2]                              |                                   | AffineSetTarget       |
| 2 | its n-ary form and the ordered        | pool_take, bs_reserve's drain,    | new grammar; q6       |
|   | result list [S16]                    | clear, collect's caller           | REJECTED [GRAM-2]     |
| 3 | [ENT-3.S6] over the four measures    | every try_ form, pool_take,       | S6 covers len alone   |
|   | [BLK-0]                              | pool_release — every branch on a  |                       |
|   |                                      | capacity                          |                       |
| 4 | the construct placement of the       | Bytes, BlockPool, bs_reserve's    | construct kills the   |
|   | measure datum [MSR-3]                | tail — every library nominal      | operand's measures    |
| 5 | a const generic as a value, an       | vacant, filled, try_place, and    | q10 REJECTED          |
|   | endpoint and a clause operand        | every capacity-parametric         | [TYPE-5]              |
|   | [MSR-6, S21]                         | function; ~43 bodies for 14       |                       |
|   |                                      | algorithms without it             |                       |
| 6 | a relation published per enum        | pool_take, pool_new, try_place,   | x1 [FN-9] Invalid-    |
|   | variant and per result ordinal, with | bs_reserve, bs_new — every        | PostconditionSelector;|
|   | field projection on a result datum   | library constructor               | x2 [TYPE-5]           |
|   | [CALL-4, S24]                        |                                   |                       |
| 7 | the window's front operations        | every queue, ring, deque and FIFO | no analogue; a        |
|   | [BLK-1, BLK-3, S8]                   | — and the growth policy, whose    | shifting take_front   |
|   |                                      | order preservation is free under  | IS writable, so only  |
|   |                                      | a window                          | the head-carrying     |
|   |                                      |                                   | forms enter           |
| 8 | linearity by declaration             | the pool's Lease, and every       | a dropped lease is    |
|   | [PROV-6, S18]                        | library that recycles values      | silent today          |
| 9 | the publication of a declared        | EVERY function in this section    | [ENT-3] has no source |
|   | relation [CALL-6]                    | and both worked programs, at      | for a declaration-    |
|   |                                      | their first statement             | domain relation       |
```

**What left the list this round, and why.** `seq_rebase` was a tenth in the seventh draft
and 3.L.8 writes it, so it leaves under L18. And the list that matters as much: **what
the partition did *not* need.** A queue needed no kernel ring, a pool needed no kernel
store, a keyed table needed no kernel occupancy, a growth policy needed no kernel growth
row, middle removal needed no kernel row, filled and vacant construction needed no kernel
row, returning a wrapped window needed no kernel row, and the `try` family needed nothing
at all. Five owner types became two, thirty-odd operations became twelve, three views
became two, sixteen added nominals became five, and three writing statements became one.

Two items were **not** resolved by writing them, and both are honest residue. A writer's
generic cannot serve a copy and an affine element type from one body, nor an affine and a
capability-released one, so `filled`, both `try` forms and `clear` are written per element
class — Q8, narrowed by [S32]: with the bound adopted, `clear`'s two bodies are **two
bounded generics**, `T: affine` and `T: linear`, rather than one copy per concrete element
type, and what remains is the copy/affine half. And a value obtained by `replace` carries
no measures, so `take_at` is declarable only at a non-measured `T`.

#### 3.L.7 When to write `linear`, and what it buys

The storage obligation is derived and a writer never marks it: a heap-backed run is
capability-released and is linear only in a scope that does not hold the `Heap` (D3), an
arena-backed run and a frame-resident run are affine everywhere, and any type that
**owns** a linear value is linear by ownership [PROV-6]. A **view** owns nothing.
**Marking a store-derived type is always redundant and is a sign the writer has misread
the criterion.** The modifier is for a **logical** obligation, and the whole test is one
question: **would silently dropping this value be a bug?**

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

**And what the modifier buys is one sentence, which round 7 corrected.** `linear` makes a
discard **visible and deliberate**: the value must be moved out whole or destructured
whole, and a destructuring is a legal consume that can throw the contents away. **Every
one of the five shapes above has a *directional* obligation** — the value must reach a
specific holder — and destructuring a request is not answering it, so the seventh draft's
advice was right for the lease and wrong for the other four by the same argument. The
rule is therefore one line and applies to the whole table:

> **A directional obligation is bought by proving the return.** Write the library's
> return operation as the **proved** spelling — total, under a `requires` the caller
> discharges from the take's own published relation — and the value has exactly one route
> on every path. The `linear` modifier is the visibility insurance beside that proof, not
> a substitute for it: it makes the discard a written statement rather than a silence.

3.L.4's `pool_release` is that spelling. The modifier's own admission condition is
[PROV-6]'s: an affine nominal only, never a tag-only enum, which probe `q11` shows is
copy today. And the shapes that do **not** take it: a value whose only cost of being
dropped is memory the language already reclaims; a value the writer merely wants to
remember to use; and a value whose obligation is conditional, since the modifier is
unconditional. The cost of a wrong `linear` is paid at every scope exit of every value of
that type, including in code the writer does not own. **Q17 records the shape D3 leaves
open**: a type with one capability-released variant needs the provider at every consuming
scope.

#### 3.L.8 Returning a wrapped window to its origin, written out

This is the item [S29] proposed as a kernel row and round 7 wrote in wf, and it is where
L18's addition clause is discharged.

```wf-design
fn rebase<T, const n: u64>(vector: own FixedVector<T, n>, spare: own FixedVector<T, n>)
    -> rebased: own FixedVector<T, n>
    reads(vector, spare), writes(vector, spare) contract {
  requires len(spare) <= 0_u64;
  requires head(spare) <= 0_u64;
  ensures len(rebased) >= len(vector);
  ensures len(rebased) <= len(vector);
  ensures room(rebased) + len(vector) >= n;
  ensures room(rebased) + len(vector) <= n;
  ensures head(rebased) <= 0_u64;
} {
  doc "Moves every element of a wrapped run into a fresh run, in order, so the result does not wrap.";
  let count = len(vector);
  let built = move spare;
  for @rot (
    at in 0_u64..count,
    invariant left: len(vector) + at >= count,
    invariant gone: len(vector) + at <= count,
    invariant made_lo: len(built) >= at,
    invariant made_hi: len(built) <= at,
    invariant spare_lo: room(built) + at >= n,
    invariant spare_hi: room(built) + at <= n,
    invariant flat: head(built) <= 0_u64
  ) {
    set (vector, one) = seq_take_front(vector: move vector);
    set built = seq_place(vector: move built, value: move one);
  }
  return move built;
}
```

**Proof route.** The caller's `spare` comes from `seq_fixed::<T, n>()`, which publishes
`len = 0`, `cap = n`, `room = n` and `head = 0` exactly, so both `requires` discharge at
the call. `left` and `gone` bound the source, `made_*` and `spare_*` the destination,
`flat` the head; each is preserved by one published relation of `seq_take_front` or
`seq_place`. `seq_take_front`'s `len(vector) > 0` discharges from `left` and `at <
count`; `seq_place`'s `room(built) > 0` from `spare_lo` and `count <= n`, the standing
`len <= cap` at entry. At the exit `at = count`, giving the two exact `len` clauses and
the two exact `room` clauses, and `flat` exports the head. **`cap` needs no clause**
[CALL-7]. **The drained `vector` is not handed back**, because after front removals its
`head` is known only as the standing bound and no non-vacuous clause about it exists — so
it dies at the return edge by its ordinary derived release, and a caller that wants to
rebase again allocates a fresh `spare` with `seq_fixed`.

**What it costs, walked against a program (L18).** A ring driver that flushes a 256-byte
`FixedVector<u8, 256>` writes
`let fresh = seq_fixed::<u8, 256>(); set rx = rebase::<u8, 256>(vector: move rx, spare: move fresh);`
before each `seq_slice`, and pays three things. **Memory**: two runs of `n` slots are live
across the drain, so `E`'s `stack` item for that context carries `2n` where a kernel
rotate would have carried `n` — 256 bytes for a driver with one ring, and for a
`Vector<'s, u8>` version a second take the store's capacity must cover. **Time**: the same
O(len) element copy the rotate would have performed, plus the fresh run's formation.
**Proof**: seven header invariants at the library, once, and two `requires` at each call
site, both discharged from `seq_fixed`'s own published relations.

**And two things a writer should know that no arithmetic shows.** A rebase must be paid
before **every** view of a run that has had a front removal, not once — `head` is
absorbing, and [VIEW-2]'s premise is what needs it. And a real ring driver does not rotate
at all: it hands the host two `iovec`s over the two halves of the wrapped window, and this
language has no spelling for a view of two ranges. That is the cost [S29] was proposed to
remove, it is a genuine cost, and Q18 is where the owner decides whether `E` can afford it.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. The
standard they are held to is that every statement is accepted by a compiler
implementing 3.K's rules, **the library functions of 3.L**, and the unchanged v0.41
rules — **and every function either program calls is declared in 3.L**. Both were walked
statement by statement against all three, and the walk was held to three standards:
*for every loop, the facts live at its head and the rule that keeps them there*; *for
every obligation, the published relation that discharges it and the function that
published it*; and, new this round, *for every value, the scope that holds its
capability and the edge on which its release runs.*

Round 7 found the seventh draft's pair failing at four places, and each is repaired at
the rule rather than at the program: every type and const argument is now written
([FN-2], probe `q4`); every loop-body borrow is bare under D4 and 3.K.0 records that
the amendment has not landed; `bs_reserve` publishes its `Refused` arm; and the release
of every heap-backed value is derived by D3 rather than written on eleven edges.

Byte figures are symbolic. No implementation computed any of them.

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
    let made = pool_new(arena: &uniq scratch);
    match made {
      None() => {
        set code = 1_u8;
      }
      Some(value: pool) => {
        loop @queue {
          set (pending, next) = try_take::<Task, 32>(vector: move pending);
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
  stack   entry               bytes  <post-codegen>  align  <ABI>
  stack   host                bytes  <post-codegen>  align  <ABI>
  stack   entry.alt           bytes  <runtime>       align  <ABI>
  stack   host.alt            bytes  <runtime>       align  <ABI>
  lanes                       count  1
  slots   task.records        count  0  bytes <runtime>  align <runtime>
  slots   completion.records  count  0  bytes <runtime>  align <runtime>
  slots   handles             count  0  bytes <runtime>  align <runtime>
```

`static.image` is the const items and the static parts of the emitted module. **Four
`stack` items, not one and not one plus a `region`**: the floor creates the entry
thread and joins it, so the host thread's stack stays live for the whole run, and it
`mmap`s one alternate stack per attaching thread, both of which are chains the artifact
runs and which [RES-1] and [STK-3] therefore make `stack` items with their own
measured chains. `stack.entry` is `main`'s frame — the `FixedVector<u8, 256>` ring, the
`FixedVector<Task, 32>`, the `BlockPool`'s `FixedVector<Vector<'a, u8>, 8>` and the one
`arena_frame` occurrence's 65536-byte extent **plus that extent's alignment slack**,
which `frame(main)` carries post-codegen [STK-3] — plus `render`, `drain`, `advance`,
the library, the runtime frames beneath `main`, its bounded teardown and the release
walk's straight-line frame cost. `lanes` is 1 because [RUN-1]'s permission judgment
grants no [PAR] permission in a marked build. Every `slots` row is zero because the
program declares no demand on any named runtime store [RES-7, RES-9], and each carries
its member size and alignment so a deployment can commit it [L6, RES-2].

#### Why it is source-resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | main declares pure, selects no command.heap, and arena_frame is pure         |
|                       | [BLK-2], so [PROV-4]'s closure is empty and [RES-4] does not fire            |
| acyclic call graph    | main -> {render, drain, advance, the library, the kernel domain}. No cycle,  |
|                       | so [STK-1] rewrites nothing and [STK-2] passes; [PROV-5]'s activation        |
|                       | refusal reads the same post-rewrite graph, and its loop-free condition holds |
|                       | because the reserving occurrence is a statement of `'a`'s block and of no    |
|                       | loop inside it                                                              |
| arena demand bounded  | the eight 256-byte takes are inside pool_new, called ONCE before the queue   |
|                       | loop, so the bump domain's backedge delta on @queue is 0, max(d) <= 0, and   |
|                       | [RES-10]'s loop rule needs no route. Had a take been inside the loop with no |
|                       | region block around it, the delta would be +256 per trip, route (i) has no   |
|                       | trip-count bound on a divergent loop and route (ii) is the wrong kind, and   |
|                       | the program would be refused at that loop                                    |
| the free list         | a FixedVector in a frame; frame placement's [RES-5] algebra has no acquire   |
|                       | and no release, so it is not a domain and premise 3 says nothing about it.   |
|                       | What keeps it full is the PROVED pool_release, whose requires the caller     |
|                       | discharges from pool_take's own published room — not the envelope, and not   |
|                       | the modifier by itself                                                       |
| queue and ring        | FixedVector<Task, 32> and FixedVector<u8, 256> are frame placement           |
| release walk          | every type reachable from main has an acyclic release graph — in fact an     |
|                       | empty one, since nothing here is capability-released — so [PROV-6]'s walk is |
|                       | straight-line, its depth is zero, and its frame cost is ordinary frame cost  |
| L9's displacement     | try_place hands its value back and the refusal is matched; pool_release      |
|                       | cannot refuse; try_take's None is the loop's exit. Nothing is displaced      |
|                       | silently                                                                     |
| stack bounded         | four contexts, four chains, each measured after code generation [STK-3]      |
| runtime closed        | W = 1, no permission granted anywhere in the module [RUN-1], no task or      |
|                       | completion records, and no declared demand on any named runtime store        |
| return and retained   | the queue loop has a break, so it has a fallthrough entry and both its       |
|                       | `return` and `retained` entries are empty; a variant with no break would     |
|                       | publish its steady state in `retained`, composed by the one formula          |
| the extraction        | the per-domain figure of E is the max over the labels of main's map          |
|                       | [RES-10], which for the bump domain is 65536 at `'a`'s block and 0 outside   |
|                       | it, because the block's exit reset cancels its own composed delta per label  |
```

#### The writer's-eye walkthrough

**`set (held, written) = render(block: move held, task: &task);`** is the statement
three drafts could not write. Under the fourth `render` took a `&uniq` container and
[CNT-7] refused it; under the fifth it published its post-state through an exit datum,
which round 5 turned back into D1; under the sixth it published only an upper bound on
the wrong side. Here [CALL-7] requires the contract to be complete over every measure
of what it hands back **on the one route it has**, so the caller receives
`written == 8_u64`, `len(rest.run) == len(block.run) + 8_u64` and the other three, and
every later obligation reads one of them. The `set` is [LIV-2] at an arm binder (probe
`w8` accepts that shape today); both targets name bindings in scope, so both are commits
and neither redeclares anything.

**`task: &task` is a loop-body borrow and is written bare** (D4). Under v0.42 a
`borrow_expr` inside a loop body that no `region_stmt` encloses is a `[FORM-8]`
RegionSpelling error whose mechanical fix is to place the borrow inside a `region`
block — probe `q2` is that rejection and probe `q3` is the same program with the block,
compiling. D4 makes the loop body itself that block, so the two inner `region { }`
wrappers earlier drafts carried are gone and the explicit form becomes a `[FORM]`
rejection. **This amendment has not landed**, 3.K.0 says so, and §7's B0b is where it
does.

**`try_place::<Task, 32>` and `try_take::<Task, 32>` write their type and const
arguments**, because [FN-2] 1124 makes them always explicit for a user generic. Probe
`q4` is the elided form rejected with `expected: "1 written type argument"` and probe
`q5` is the written form accepted, so this is what the landed amendment actually says —
the seventh draft wrote them elided under a criterion that covered regions only.

**`requires room(block.run) >= 8_u64;` is discharged by a dominating branch**, and that
branch is the honest price of the pool being library data. A `Vector<'s, u8>` carries
its capacity as a measure and not in its type [BLK-1], so putting one into a
`FixedVector` element and taking it out again loses the figure `pool_new` established.
`let spare = room(held.run); let big = spare >= 8_u64; if big { ... }` is one runtime
branch per lease, and its first statement is a fact only because [ENT-3.S6] generalizes
over the four measures [BLK-0]. Q6 records that a container whose element capacity is in
its type is the next candidate and has to justify itself against this branch.

**There is no header invariant on the queue loop**, and `drain` is the **checked**
spelling: it takes no `room` requirement, branches on the ring's own room, copies when
it fits, and reports `sent`. A full ring then stops being written instead of being
asserted not to fill, which is L3's and L9's discipline. `drain`'s one remaining
requirement, `count <= len(deref(block).run)`, discharges from `render`'s
`len(rest.run) == len(block.run) + 8_u64` with `written == 8_u64` and the standing
`Z <= len` — one published premise and one standing fact.

**Inside `render`, the backedge is the derivation the whole container surface rests
on.** The `set` is [LIV-2] at a **field of a value that is linear by the modifier**, and
it is admitted because [PROV-6]'s partial-consume refusal excepts a sub-place the same
statement's commit reinitialises — which is round 7's reconciliation of two rules that
otherwise refuse the design's own central statement. Its target names a binding in
scope, so the root's [ENT-2] term survives [MSR-3]; the facts over `len`, `room` and
`head` of `block.run` die by [MSR-2] because the commit writes that descriptor storage;
and `seq_place`'s declared relations re-establish them on the same term through
[CALL-6]'s S13 and [CALL-4]'s `set`-target destination. **Each of those relations reads
`len(vector)` as that call's call datum, because `vector` is an `own` parameter**
[MSR-3] — under the seventh draft's `writes`-keyed table it read the post-state,
`len(P) = len(P) + 1` was in the state, and every goal in this function was provable
from [MSR-4] step 1. Each invariant is preserved by exactly one published premise, which
is what puts the derivation inside [ENT-6] 3015's two-premise budget.

**`set pool = pool_release(pool: move pool, lease: move held);`** is the proved
release. `requires room(pool.free) > 0_u64` is discharged from `pool_take`'s
`when leased is Some(value: got): room(rest.free) >= 1_u64` — one published premise,
surviving the intervening `render` and `drain` because neither writes `pool`'s
descriptor storage [MSR-2]. There is no refusal arm, so on every path the lease goes
back. `held` is `Lease<'a>`, **linear by the modifier in every scope** (D3 changes
nothing for a modifier-linear value), so [LIV-1] requires it dead on every edge leaving
the arm and this statement is what makes it so.

**Nothing in this program is capability-released, so nothing has a `dispose` and
nothing needs one.** `main` declares `pure`, and it is true: every value is
frame-resident or arena-backed, every derived release carries the empty effect row
[EFF-2] 1427, and the arena's own reclamation is `'a`'s reset [PROV-5]. That asymmetry
is D3 read from goal A's side: goal A's programs pay nothing for R2 at all.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one library function with a typed
failure, the append helper takes the destination by value and hands it back, **not one
region names the store**, and — under D3 — **not one statement releases anything**,
because `main` holds the entry `Heap` and the compiler derives the release on every
edge leaving each arm.

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
        }
        Refused(value: back) => {
          set code = 70_u8;
        }
      }
    }
  }
  return exit_status(code: code);
}
```

#### The writer's-eye walkthrough

**`filled::<u8, 4096>(value: 65_u8)` writes both arguments** [FN-2], and its
`ensures head(result) <= 0_u64;` is what makes the next statement possible.

**`set (kept.v, total) = collect(out: move kept.v, source: line);`** is R1's central
statement, at a **field** place, and three rules have to agree about it. D2 decides the
targets by name resolution: `total` names the live outer `u64` and is a commit, not a
declaration — probe `r4` is the `[TYPE-6] DeclarationCollision` the seventh draft's
per-statement rule produced. `kept.v` is a proper sub-place of `kept`, and consuming it
is **not** a partial consume for two independent reasons [PROV-6]: under D3 `Bytes` is
**affine** in `main`'s scope, because `main` holds `heap: own Heap`; and the same
statement's commit reinitialises the sub-place, which is the exception [PROV-6] states
for a value that *is* linear in its scope. Both targets are dead at the commit —
`kept.v` because the right-hand side consumed it, `total` because its type is copy — and
non-overlapping. The relations reach both targets through [CALL-6]'s S13 and [CALL-4]'s
`set`-target destination.

`collect`'s `requires len(source) <= room(out)` discharges from `bs_reserve`'s
`room(ready.v) + len(ready.v) == 4096_u64` and `len(ready.v) == len(holder.v)` with
`bs_new`'s `len(fresh.v) <= 0_u64`, giving `room(kept.v) >= 4096`, against
`seq_slice`'s published `len(result) = <call datum of len(input)>` and `filled`'s
`len(result) >= 4096`. All four links are published clauses of functions 3.L declares.

**`let line = seq_slice(vector: &input);`** discharges [VIEW-2]'s
`head(input) + len(input) <= cap(input)` from `filled`'s `head(result) <= 0_u64` and the
standing `len <= cap` — one clause and one standing fact, in the unordered-pair family.
`line` is a `Slice` and is therefore **copy** [S27], so it is passed without `move`; its
loan begins at the formation and ends at its **last use** [PROV-3], which is the
`collect` call. That end condition is round 7's: under the seventh draft a copy view's
loan ended at a consume it never has, so it froze `input` for the rest of the function
and, one block down, would have made `kept` unreleasable while `body` was live.

**The two `region { }` blocks stay, and they are not the loop-body case.** [OWN-10]
641 requires a borrow's region to be introduced within the borrowed binding's scope, so
`kept.v` must be bound before the block opens; under [FORM-8] a `borrow_expr` that no
`region_stmt` encloses always writes its region, and these blocks are what let both
borrows elide theirs. D4 replaces such a block only where the loop body already is one.

**`write_once(output: &uniq sink, source: &body, start: 0_u64, end: total)`** is
[VIEW-7] over a view, and it is the statement that makes goal A's container half real.
Its obligations are `0_u64 <= total`, implicit, and `total <= len(deref(source))` —
stated over `source`, which is `write_once`'s own range-bearing parameter, and not over
a destination it does not have. It discharges from [VIEW-2]'s
`len(body) = <call datum of len(kept.v)>` and `collect`'s
`len(rest) == len(out) + written` with `written == len(source)`. Its three regions all
relate nothing and are all elided.

**There is no `dispose` in this program, and that is D3.** `Bytes` owns a
`Vector<u8>` whose release requires the `Heap`; `main` holds the entry `Heap` by its own
input row; so `kept` and `back` are **affine in `main`'s scope**, and the compiler
derives their release on the edge leaving each `match` arm — unconditionally, at
[LIV-1]'s existing join, with no drop flag. `main`'s row carries `writes(heap)` because
that release spends the capability [EFF-2] 1427, so the free is signature-visible where
today's compiler emits it under no effect row at all (probe `r2_5`). The walk's depth is
`Bytes`'s release-graph height, which is one, so no auxiliary storage and no
`wf_resource_abort` is reachable. **There is no path on which the process disappears**,
which is the whole of goal B — and under the seventh draft this same program owed a
written statement on each of `main`'s eleven return edges for each of up to five live
values.

**What a writer would still write `dispose` for** is inside `bs_reserve`, and 3.L.5
shows it: `dispose old;` releases the source run **before** the grown one is handed
back, which is the difference between a peak of `count + total` slots and a peak of
`total`. That is the whole of [S12]'s remaining justification and it is a real one.

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
    every release of heap-owned storage runs in a scope whose effect row names the
      heap [PROV-6, D3]
    every release walk's depth is a compile-time constant [PROV-6]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

Six of the diagnostics the design owes a writer, each citing a rule that states it.

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
    reads from your own contract [CALL-2, CALL-7], or take a view of it

Semantics/Source [PROV-6]: LinearValueNotConsumed
  binding "kept" of type Bytes is live on the edge leaving this match arm
  its leaf Bytes.v of type Vector<u8> is released to the store the entry heap names,
    and no binding of type Heap is live in this scope, so the value is linear here
  mechanical_fix: move the value out of this scope, destructure it, or take
    heap: &uniq Heap as a parameter of this function, which makes the release
    compiler-derived on every leaving edge [D3]

Semantics/Source [PROV-6]: DisposeHasNoProvider
  "dispose old;" needs the provider of the store its leaf Vector<u8> names
  no binding of type Heap is live here, and the capability is determined by the
    brand rather than written
  mechanical_fix: take heap: &uniq Heap as a parameter of this function

Semantics/Source [PROV-6]: LinearValuePartiallyConsumed
  "move chunk.page" takes one leaf out of "chunk" of type Chunk, which is linear here
  the residual leaf Chunk.spare of type Vector<u8> is not reinitialised at this
    statement's commit, so it would leave this scope by neither a move nor a
    destructuring
  mechanical_fix: write let Chunk(page: page, spare: spare) = move chunk; and handle
    both leaves, or make this statement a set whose target list covers both

Semantics/Source [CALL-7]: IncompleteHandBackContract
  "filled" returns result: own FixedVector<T, n>, which it constructed, and its
    contract states no admissible clause for head(result)
  "ensures head(result) <= cap(result);" is not one: both sides follow from [MSR-2]'s
    standing facts, so it publishes nothing a caller did not already have
  a caller that forms a view of this run needs it [VIEW-2]
  mechanical_fix: carry invariant flat: head(built) <= 0_u64 on the construction
    loop and publish ensures head(result) <= 0_u64;
```

The third and the sixth are new in this draft: the third is D3's own diagnostic, which
names the scope rather than the value and offers the parameter as the repair, and the
sixth is [CALL-7]'s vacuity test, which round 7 showed the seventh draft's version had
no way to state.

---

## 5. Open questions

Everything the owner's rulings and decisions settle is dropped and not restated, and
so is everything earlier drafts asked and this one answers. **The seventh draft's
§5.0 is deleted**: it restated eight decisions 3.S already records with their
alternatives and their costs, and a decision belongs in one place.

**Q1. May a marked program handle a typed refusal, or must it prove every
acquisition?** **Permissive**: both spellings are admitted, since neither can ask for
more than `E`. But the two are not interchangeable: a **checked** acquisition is what
[RES-10]'s reusable-capacity route reads, and a **proved** release is what makes a
directional obligation real, because a checked one leaves a refusal a writer may legally
discard.

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

**Q5. When does `par` become usable inside a marked program?** [RUN-1] requires the
permission judgment to grant nothing and [RUN-2] publishes `lanes(1)`, because the
current runtime's wait path runs a stolen task on the waiting lane's own stack. The
answer is the compiler-managed work-first continuation representation, then lifting the
prohibition. **[PROV-5]'s activation refusal and [RES-10]'s overlap rule are both
written for that day**, which is why the overlap rule is stated even though it fires in
no marked build.

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
and why probes `x14` and `x15` disagree. **A `struct` or `enum` all of whose field types
are copy should be copy** — and the half that matters more is the second: **a generic
body's `move` of a type parameter should be admitted at a copy instantiation, where it
is a no-op.** Without that half the first does not remove the wall, because the
*template* is checked as if `T` were affine. **The third axis was [S32]**, a linearity
bound on a generic parameter, and it is decided: adopted 2026-09-04, so what is left of
Q8 is the two halves above, and the design's per-element-class functions are now two
bounded generics wherever the split is a linearity split (3.L.6).

**Q9. Is `E` part of program identity?** **An emitted machine-readable table beside
the object, carrying the module's content digest and explicitly not part of [PROG-2]
compilation-unit identity.** One residue is recorded: the digest is of *the module*,
while the figures under it include [RUN-2]'s runtime-published capacities and [STK-3]'s
chains through the linked runtime and adapter, so a swapped runtime satisfies the check
while changing what `E` describes. A second digest over the qualified runtime is the
obvious repair and is not proposed here.

**Q10. Should `propagate` reach a multi-result call?** This is what is left of the
seventh draft's Q10 after D3 removed the cleanup cost and the owner rejected [S28].
[ERR-3] 1472 requires the operand to be a `Result`, and a hand-back helper under R1
returns a result **list**, which is not one — so `set (p1, ..., pk) = propagate f(...);`
does not exist, and every fallible transforming helper costs a `set` and a `match` where
it once cost one line. `raw_deflate_dynamic_decode.wf`'s `decode_dynamic` has seven such
sites. The shape that would work is one grammar alternative and one [FN-1] edge: admit
the form where `f`'s **last** result ordinal is `own Result<T, E>`, committing the
earlier ordinals on both edges and leaving on `Err` — which is exactly L3, every affine
input the callee did not consume coming back on both edges. **It is not proposed in 3.S
because this design does not need it to be correct**, only to be pleasant, and because
it is [ERR-3]'s question rather than this batch's.

**Q11 is answered and is retained only as a record.** A view-forming borrow needs no
written region: the region relates nothing, so [FORM-8] elides it, and the enclosing
block keeps its braces and loses its name.

**Q12 is answered by the owner.** [S25] is adopted: `reserve_file` becomes fallible,
on the principle that a failure the environment can produce is a typed value. **[S33]
completed it on 2026-09-04**: the outcome is a three-variant nominal, so the store's own
refusal is a variant rather than a class of an error payload, and nothing of Q12 stays
open.

**Q13. A run whose element type is linear *by declaration* has no route out**, and
§2.1's release row marks the notion open at exactly this shape. It is not a nominal, so
the destructuring consume does not reach it; it has no capability-released leaf, so no
release reaches it; and it cannot be moved out of the function that built it. A writer
meets it the moment they put a lease, a ticket or a transaction into a `FixedVector`.
This design avoids the shape by putting the obligation on the value that is handed out
and not on the container of spares (3.L.4), which is the right modelling and is not a
rule. **The principled fix is a fourth route: a run whose element type is linear is
discharged when it is proved empty — `len(v) <= 0_u64` at the scope exit — and a drain
loop's [INV-1] exact-exhaustion conclusion is what proves it.** That is one sentence and
it reuses machinery 3.L already writes; it is not proposed here because it needs a
falsifier pass of its own.

**Q14. Should [INV-1] admit `==` in a header invariant over measure terms?**
[INV-1] 3105 admits the four ordered symbols, so an **exact** measure relation costs
**two** header invariants: `collect` carries five where three would do, `bs_reserve`
seven where four would, `render` five where three would, and 3.L.8's `rebase` seven
where four would. [CALL-7] makes a complete hand-back contract mandatory, so this cost
is paid by every helper a loop calls. The relation is still one `compare_op` performing
no [OP-1] operation, [INV-1]'s own normalization already handles both directions, and
[FN-9] 1312 admits all six in a contract clause already. **Recommend admitting `==` for
an invariant whose operands are measure terms**, landed **with** [CALL-7] rather than
after it, which is the smallest change that halves the invariant count in every function
this design prints.

**Q15. Should L18 gain a cost clause, and then one bulk-move row?** L18 asks whether a
writer *can* express the effect. For a byte copy the answer is yes and the cost class
differs by about two orders of magnitude: every growth policy, every ring drain, every
`collect` and now 3.L.8's `rebase` is element-at-a-time, roughly five operations per byte
facts-off where a `memcpy` is a small fraction of one. The law would read *a rule enters
the kernel when no wf program has its effect, **or when every wf program that has its
effect has a strictly worse cost class and the measurement is recorded***, and then one
row, `seq_move_prefix(dst: own V, src: own V, count: own u64) -> (dst2, src2)`
publishing all four measures on both runs. **Recommend the law change and the row, and
recommend that the measurement be taken before either lands** — this design has measured
no timing anywhere and would not start with its own proposal. **This question got larger
this round**, because withdrawing `seq_rebase` to the library makes a second O(n) copy a
library obligation rather than a kernel one.

**Q16. Should a destructuring consume be able to bind some fields and dispose the
rest?** [S13] binds **every** field with no `_`, so taking one page out of a five-field
record is five binders, three of them dead. The shape is
`let Chunk(page: page, ...) = move chunk;`, releasing every unnamed field by exactly
[PROV-6]'s walk. **Recommend it as a later convenience and not now**: L18 says a
convenience is not a rule, the walk it needs already exists, and D3 shrank the cost,
because in a capability-holding scope the unnamed fields take their derived release
rather than needing a written one.

**Q17 is a cost this design pays rather than a question it avoids.** [MSR-3]'s
denotation table makes a `&uniq` parameter's measure **inadmissible in an `ensures`**.
The consequence is that **a user `fn` that lends a provider onward can publish nothing
about that store's post-state**: a caller's `room(scratch)` fact dies at the call and
every subsequent proved acquisition in that caller is undischargeable, so an
arena-lending helper forces its caller to the checked spelling. The alternative —
admitting such a clause for a user `fn` — is exactly the caller-side claim L11's second
sentence forbids. What *is* available, and what a later batch should weigh, is
**restricting [EFF-1]'s `reads` and `writes` paths to formals reached through a borrow
plus any path whose leaf is a provider or a loan-bearing type**, which round 6 measured
as one or two items removed from every value-in / value-out signature in the library.
*Recommend it, and record that it is an [EFF-1] amendment with a wide blast radius that
no current experiment needs.*

**Q18 is new. Should returning a wrapped window to its origin be a kernel row after
all?** [S29] is **withdrawn** because round 7 wrote the replacement, and 3.L.8 walks and
prices it: two runs of `n` slots live across the drain, the same O(len) copy a rotate
would have performed, seven header invariants once at the library, and a fresh spare per
rebase because the drained run's `head` has no non-vacuous clause. **The memory half is
the one that can be unaffordable**: a driver with three viewed rings carries `2n` in
`E`'s `stack` item for each, where a kernel `seq_rebase` would carry `n`. If the owner
judges that too high for the marked-driver shape goal A exists for, the row comes back
as one [BLK-3] operation with `head(result) = 0` and `len`, `cap`, `room` unchanged — and
L18's addition clause then requires this exact walk beside it, which is what this draft
has now written. **Recommend the library form until a real driver's `E` is computed.**

**Q19 is new. What does an exclusive view cost a run that is also read?**
A `MutSlice` holds an exclusive loan for its whole life [PROV-3], so a run may not have
a live writable view and a live shared read at the same time — probe `s6` is that
conflict at v0.41. Every program that alternates writing a run through a view with
reading it re-forms the writable view at each alternation, which for `wfgrep`'s
`search_file` is once per matched line. [S27] removed the same cost for the **shared**
view and nothing removes it here, because two exclusive loans on one range are what
[OWN-5] 606 refuses and that refusal is the single-writer argument. **The question is
whether a `seq_split_at` (Q3) covers enough of the shape**; this design does not answer
it, and it is the friction a writer of an I/O loop meets first. [S31] does not remove it:
the shared child reborrow that [VIEW-6] admits lets a helper publish what it filled, and
it still forbids a write of the parent for exactly as long as the child lives.

**Q20 is new, and half of it is answered.** [S33] is **adopted** for `reserve_file`
(2026-09-04), so the handle table's exhaustion is a variant and [RES-6] publishes
`room(factory) = 0` on that arm. The general form of the question stays open: **every
covered store whose refusal a route must read needs its refusal to be a variant of the
operation's own outcome, not a class of an error payload.** The arena has that shape
already, because a refused `seq_arena` returns `None` and publishes over it, and the
handle table now has it. What remains is that the partition should be written **once as a
rule about covered stores** rather than once per operation, and that rule is not drafted
here.

---

## 6. Verified versus reasoned

**Verified** means a compiler executed it, against a gate-profile `whitefootc` built
from this tree, in this session or in one of the twenty-six falsifier sessions whose
probe names are quoted. **Probes `q1`-`q16` were run in this session** against the
**v0.42** gate binary — v0.41 plus `[FORM-8]`, confirmed by `q2` — and every verdict in
6.1 is my own run. Probes `d1`, `e1`-`e8`, `v1`-`v3`, `c1`-`c8`, `a1`-`a8`, `w1`-`w9b`,
`x1`-`x15`, `gen1`-`gen3`, `pick`, `setslice`, `arm7`-`arm13`, `s1`-`s7`, `k*`, `n*`,
`p*`, `f*`, `g*`, `m*`, `r1_*`, `r2_*` and `t1`-`t14` are earlier rounds'. **No name
denotes two probe sets.** No timing figure appears anywhere in this file.

### 6.0 B1 landed (v0.44 candidate)

**Four of 3.K's rules are no longer paper.** [MSR-3], [MSR-5], [CALL-4] and [CALL-6] are
implemented as the v0.44 candidate (PR #17), each in the narrower form the four
corrections above record. Six conformance cases carry them:

```text
| case                                        | expected verdict                    |
|---------------------------------------------|-------------------------------------|
| msr5-pos-two-measure-clause                 | run, exit 0                         |
| msr3-pos-own-operand-call-datum             | run, exit 0                         |
| msr3-neg-uniq-state-measure-in-ensures      | reject, MSR-3                       |
| call6-pos-routed-relation-over-a-call-datum | run, exit 0                         |
| call6-neg-contradictory-published-relations | reject, CALL-6                      |
| call4-neg-measured-result-not-admitted      | reject, FN-9                        |
```

**The corpus consequence is the one this design argued for.** `tests/programs/wfgrep.wf`
and `tests/programs/raw_deflate_boundary.wf` both carried an `append_slice` that published
a bound through a **measure of a `&uniq` parameter**. Both now take `capacity: own u64`
and state `requires capacity == len(deref(destination));` instead, so the bound is a fact
about a value the caller supplied and the callee names no post-state it cannot reach
(L11). That is [MSR-3]'s refusal taken as the writer's repair, and it is the first corpus
evidence that the refusal is affordable rather than merely correct.

**What did not land is recorded at its rule and re-batched in §7**: [CALL-4]'s measured
result, its measure over a result place and its route over any variant of any returned
enum go to B7; [S16]'s ordered result list and the destinations that read it go to
§7's B1b; and [MSR-5]'s affine widening stays [MSR-4]'s in B2.

### 6.0b B1b landed (v0.45)

**[S16] and the result ordinal are no longer paper.** The ordered result list, the
destructuring `let` binder list, the `set` target list, and the multi-expression `return`
are written into [GRAM-2] and [GRAM-4]; [FN-1] states the result ordinal and reads every
result judgment per ordinal; [TYPE-5] derives binder i and target i from ordinal i;
[SET-1] commits a target list in written order; [CALL-4] takes the ordinal-named route,
its omitted-binder condition and its ambiguity refusal, and adds two of the three
destinations. No rule id was added: `is` is the one added grammar atom. Five conformance
cases carry it:

```text
| case                                              | expected verdict |
|---------------------------------------------------|------------------|
| s16-pos-result-list-reaches-both-let-binders      | run, exit 0      |
| s16-pos-result-list-reaches-both-set-targets      | run, exit 0      |
| call4-pos-route-names-a-result-ordinal            | run, exit 0      |
| call4-pos-omitted-route-binder-with-one-enum-ordinal | run, exit 0   |
| call4-neg-ambiguous-route-over-two-enum-ordinals  | reject, CALL-4   |
```

**A declaration that writes a list hands its ordinals back as one owned aggregate**, and
the two binder forms are its projections; that is a result shape and the qualification
review says so. It is the whole implementation cost of R1's transforming-operation shape:
no new representation, no new transport, and every ordinal an ordinary owned value under
the ordinary transfer, drop and release rules.

**What did not land is recorded at [CALL-4] and re-batched below**: the third destination,
the arm binder of an own-place `match` whose scrutinee is not the call itself, needs a
relation to survive a naming event between the call and its destination, which is
[MSR-3]'s deferred binder placement rather than a destination of its own. It goes with
B7's measured result. A **borrow-mode or `slice` ordinal** of a result list is an explicit
compiler-capability refusal rather than a language restriction: [FN-1] states that each
ordinal receives the ceiling and provenance judgments independently, and deriving them per
ordinal is the work that has not been done.

### 6.0c B2 landed (v0.45)

**The proof surface is no longer paper.** [MSR-1], [MSR-2], [MSR-4] and [MSR-6] are written
into the active specification as four added rules, and [OP-1], [OP-4], [OP-7], [TYPE-6],
[ENT-2], [ENT-5], [ENT-6], [INV-1] and [MSR-5] are amended in place to read them. The
compiler carries all four: the four measure terms and their [OP-1] readers, descriptor
storage as the support of a measure, one numeric goal disposition with a
compiler-owned affine atom per live measure term, and the const generic as a value.
Eight conformance cases carry it:

```text
| case                                                        | expected verdict |
|-------------------------------------------------------------|------------------|
| msr1-pos-the-four-measure-readers                           | run, exit 0      |
| msr1-pos-subscript-obligation-against-len                   | run, exit 0      |
| msr2-pos-sibling-field-write-keeps-the-measure              | run, exit 0      |
| msr2-pos-element-write-keeps-the-measure                    | run, exit 0      |
| msr2-neg-descriptor-write-kills-the-measure                 | reject, OP-4     |
| msr4-pos-capacity-requirement-discharges-a-length-obligation| run, exit 0      |
| msr6-pos-const-generic-as-a-value                           | run, exit 0      |
| msr6-pos-const-generic-in-a-clause-and-an-endpoint          | run, exit 0      |
```

**Probe `r2_4` is accepted, and that is the batch's own measured result.** The program
this section recorded as root-granular — a `struct` with a `flags: u64` beside a
`tail: buffer<u8>`, `let size = len(frame.tail);`, then `set frame.flags = 1_u64;`, then
`frame.tail[3_u64]` — was an `[OP-4] UndischargedBoundsObligation` with the residual
`3_u64 < len(frame.tail)` at this branch's tip and compiles now. Descriptor storage is
the whole repair: the support of the measure is the resolved place of `frame.tail`, not
of `frame`, so the sibling write overlaps nothing. `docs/patterns.md` P16 carries the
correction, because the pattern said *root binding* where [MSR-2] says descriptor
storage.

**Probe `q10` is accepted.** `fn capacity_of<const n: u64>(...) { return n; }` compiles,
and the same parameter is a `for_stmt` endpoint and a clause operand in the same
program. That is [S21] in all three positions.

> **Correction, decided 2026-09-04, from B2's implementation.** [MSR-1] said a measure
> former over an unmeasured type "is an [OP-1] rejection at the `call`". The compiler's
> pre-existing and correct judgment is the ordinary [TYPE-5] operand rejection at the
> place, carrying the measured types the table has a row for; [OP-1] owns the arity and
> written-type-argument failures, which is what [MSR-5] already said. The rule text now
> says [TYPE-5], and no compiler behaviour changed.

> **Correction, decided 2026-09-04, from B2's implementation.** [S11] priced the three
> reader names as "a readability choice", on the ground that no rule reads a name. One
> rule does: [OP-1]'s `ReservedLowerNames` is exactly the dotless IDENT-shaped operation
> spellings union the mode words, and no source declaration may use a member of it. Adding
> `cap`, `room` and `head` as reader rows therefore takes all three spellings away from
> every writer declaration, and `let room = len(line);` — the exact line P16 of
> `docs/patterns.md` recommended — is now a [FORM-3] `ReservedName` rejection. Measured on
> this branch before the repair: **28 of 525 conformance cases and 72 of 491 snapshot
> cases stopped reaching their recorded verdict, every one of them that rejection and none
> of them a semantic change**; `room` accounted for 63 of the 72, `head` for 6 and `cap`
> for 3. The repair is the writer's own: the corpus renames its bindings (`room` to
> `spare`, `head` to `front`, `cap` to `limit`) and every verdict returns, with no
> expectation and no snapshot row edited. **The cost is real and it is the owner's to keep
> or spend differently** — the alternative is a reader name no writer wants for a local,
> which is a change to [S11] and not one this batch may make.

**What did not land, and why it could not.** Four of §7's B2 tests name a *run of runs* or
a *wrapped run*, and neither exists in v0.45's type system. [TYPE-2] admits only a **flat
element type** — an integer, a float, `Bool`, `unit`, or a struct or enum of those — so no
measured type is an admitted element type: `buffer<buffer<u8>>` is
`Semantics/Unsupported: CompositeValues` at the parameter, `array<buffer<u8>, 4>` is a
[TYPE-2] rejection, and `array<array<u8, 4>, 4>` is the same. Consequently:

- **`len(P[i])` is stated and unexercised.** [MSR-1]'s subscript admission is written into
  [ENT-2] clause (b) and the compiler forms a measure term over any admitted measure
  place, but no program can reach a subscripted one. This is not a compiler-capability
  refusal — there is nothing to refuse — and it needs the container types.
- **The `set` at an element position of a run of runs is not expressible**, for the same
  reason. What is expressible is the half [MSR-2] states over storage: an element write of
  a **scalar** kills nothing, because the killed set is empty rather than excepted, and
  `msr2-pos-element-write-keeps-the-measure` pins it.
- **An element-position `replace` is not expressible at all.** [SET-2] requires the
  target's final selected type to be affine, and every flat element type is copy, so
  `replace p[i] = e;` is a [SET-2] hard error before any measure question arises. Both
  halves of §7's `replace` test — the descriptor's measures dying, the scalar's nothing —
  wait on an affine element type, which is B7's.
- **The wrapped run is B7's.** Every row of this version's measure table gives `head` the
  exact value zero, so the injectivity sentence is exercised only at the identity map.
  `msr1-pos-subscript-obligation-against-len` runs two disjoint ranges over one run at
  `head = 0`; the two-disjoint-ranges-over-a-**wrapped**-run test needs a row whose `head`
  can be nonzero.

**The compiler still classifies a projected callee write from the actual's shape.** A
write through a `&uniq buffer<T>` actual is read as an element write regardless of what
the callee does, which is why `ent5-neg-callee-uniq-buffer-replace-kills-length` remains
`xfail` and did not turn XPASS here. [MSR-2] restated the *granularity* over storage; the
*classification* of what a callee write touches is [CALL-3]'s, and B3 is where that case
flips. Nothing in B2 was allowed to flip it early.

**One surface question the design does not settle, left to the owner.** `clause_expr`
admits one operand, or two operands around one `infix_op` or `compare_op` [MSR-5], so a
clause cannot write `len(run) + room(run) == cap(run)` or any other three-operand relation
over measures. The capacity identity is therefore reachable by the checker as an automatic
premise and unwritable by the writer as a clause. That is the production B1 landed and
this batch did not touch it.

### 6.0d B3 written, and blocked on its own order (v0.45)

**D1 is closed, and the price is that §7's B3 paragraph is wrong about its own
prerequisites.** [CALL-1], [CALL-2], [CALL-3] and [CALL-5] are written into the
active specification as four added rules; [ENT-5]'s clause (b), [FN-9]'s
entry-image sentence and [SYS-8] are amended in place; the compiler selects the
transport from the callee's declared parameter mode and type and, for a
declaration record with no body, from its declared contract; and the sweep's
recorded unsound accept is an ordinary rejection:

```text
| case                                                        | expected verdict |
|-------------------------------------------------------------|------------------|
| ent5-neg-callee-uniq-buffer-replace-kills-length            | reject, OP-4     |
| call1-pos-a-shared-borrow-keeps-every-fact                  | run, exit 0      |
| call2-pos-an-own-operand-measure-reaches-the-result         | run, exit 0      |
| call2-neg-a-result-carries-only-the-contract                | reject, OP-4     |
| call3-pos-a-sibling-field-write-through-a-unique-borrow-... | run, exit 0      |
| call3-neg-a-descriptor-write-through-a-unique-borrow-...    | reject, OP-4     |
| call5-neg-an-element-only-body-kills-the-measure-the-same   | reject, OP-4     |
| call5-neg-a-bound-borrow-actual-kills-the-same              | reject, OP-4     |
```

**The blocker, measured 2026-09-04 on `batch/0127-containers`.** With the rules
in place the branch gate is not green. Three conformance cases, eight snapshot
rows, forty-one compiler library tests and twelve of the twenty-eight sources in
`tests/programs` move from accept to reject, and every one of them is the same
sentence: a `&uniq` parameter whose referent type is
measured selects no transport, so a call through it kills the caller's measures
*whatever the callee's body does*, and a helper that only fills elements takes
its caller's length with it. The three conformance cases are
`x-requires-output-capacity-run` (a `copy_bytes(out: &uniq buffer<u8>, ...)`
whose caller then reads `output[3_u64]`), `x-base64-rfc-vectors-run` (the same
shape at `man_output[0_u64]`), and `x-child-reborrow-run`, which is the sharp
one: its `proxy_byte` declares `requires 1_u64 < len(deref(out))`, calls
`write_byte(out: &uniq deref(out))` through the same borrow, and then loses its
**own precondition** at `set deref(out)[1_u64] = 9_u8;`.

**Neither destination this design gives such a helper exists yet, and that is
the whole finding.** §7 says B3 needs "none of the new types". It needs one of
two things this version does not have:

- **The by-value hand-back** [CALL-2] — `fn fill(out: own Vector<u8>, ...) -> (out: own Vector<u8>, ...) ensures len(out) == ...` — requires a measure over a **result place**, which [CALL-4] explicitly defers and §7 lands in **B7**.
- **The view** [CALL-3] — a callee taking `MutSlice<'r, u8>` and writing element storage — requires a **writable view type**, which §7 lands in **B7**. Today's `slice<'r, T>` is the only loan-bearing row and no [SET-1] target reaches through it.

So the two migrations B3's own rules name are both downstream of B3. The
repair a writer can actually write today is a source branch re-establishing the
bound below the call, whose false edge is not intended program behaviour — the
shape this project's rules name as a source or compiler defect — and for
`x-child-reborrow-run` there is no repair at all, because the fact the body
loses is the one its contract already promised.

> **Correction, decided 2026-09-04, from B3's implementation.** §7's B3
> paragraph says this batch is "second in the live-defect order and needing none
> of the new types". That is false as stated. [CALL-5]'s conservative default is
> sound and its ordering is not: it withdraws a transport before either
> replacement exists, so between B3 and B7 the language admits no way to write a
> helper that fills a caller's run. B3 belongs **with or after** the batch that
> lands the writable view and the measured result, or the corpus and the writer
> pay for the interval. The rules themselves are unchanged by this correction;
> only their place in the order is.

> **Correction, decided 2026-09-04, from B3's implementation.** §7's B3 test
> list names "probe `q8`'s program, whose accept becomes the same rejection".
> `q8` is D1 written with every region elided, and at this branch's tip that
> program is a [FORM-8] `RegionSpelling` rejection before any transport question
> arises — v0.42's canonical region spelling, which §7's own B0a records as
> landed, refuses the bare borrow. The [CALL-5] case that carries `q8`'s point
> is therefore written with the borrow **bound to a name and moved into the
> call**, which is a different checked argument shape reaching the same declared
> parameter and is the same claim about the actual's spelling.

**What [CALL-3] could not exercise, and where it goes.** §7 asks for a callee
writing through a `MutSlice<'r, Vector<u8>>` that kills `len(origin[0])` and
keeps `len(origin)`. Neither `Vector`, `MutSlice`, nor any measured element type
exists in this version, so that program has no shape and the viewed-element half
of [CALL-3] is stated and unexercised, exactly as [MSR-1]'s subscript admission
is. It lands in **B7/B8** with the runs and the views, against the row whose
element type is itself measured. What is expressible now is the storage
restatement over today's types, and the two cases above are that: a callee whose
declared row names a sibling field keeps the run's measure, and one whose row
names the run-bearing field kills it — one minimal pair over one declared
effect path.

**The system operations were the one place the classification had to read a
declared contract rather than a type.** `read_at`, `directory_next`,
`host_copy_bytes` and `host_copy_utf8` all take `destination: &uniq buffer<u8>`
and declare `writes(destination)`; classified by parameter type alone they
would kill every caller's length and take the whole I/O corpus with them. They
have no body, and [SYS-8] already states that the range-bearing family accesses
its declared buffer over `[start, end)` — so that declared extent is the
contract [CALL-5]'s second selector names, and [SYS-8] now says in one sentence
that the extent is the complete one and is element storage. This is the same
move [CALL-5] itself records for [RES-8]'s saturation fact: a declared clause is
a selector, a body is not.

### 6.1 What the compiler did in this session

```text
| probe | program                                                     | verdict                               |
|-------|------------------------------------------------------------|---------------------------------------|
| q1    | a minimal `command fn main`                                | ACCEPTED, exit 0                      |
| q2    | a loop-body borrow of an outer binding, written bare       | REJECTED [FORM-8] RegionSpelling, fix |
|       |                                                            | "place the borrow inside the region   |
|       |                                                            | block whose region it takes"          |
| q3    | the same program with an explicit `region { }` loop body   | ACCEPTED, exit 0                      |
| q4    | a user generic called with its type argument elided        | REJECTED [FN-2] TypeMismatch,         |
|       |                                                            | expected "1 written type argument"    |
| q5    | the same call written `count::<u64>(...)`                  | ACCEPTED, exit 0                      |
| q6    | `fn split(v: own u64) -> (low: own u64, high: own u64)`    | REJECTED [GRAM-2], expected IDENT     |
| q7    | `ensures len(kept) >= 1_u64;` on a run result              | REJECTED [GRAM-5] at the comparison   |
| q8    | D1 verbatim, fully elided regions                          | **ACCEPTED, exit 0**                  |
| q9    | `set c = bump(cell: move c);` at a live affine local       | REJECTED [STOR-1] AffineSetTarget     |
| q10   | a const generic read as a value                            | REJECTED [TYPE-5] UnresolvedUse,      |
|       |                                                            | available: [ConstGeneric]             |
| q11   | tag-only `enum Ticket { Open(); Closed(); }` used twice    | ACCEPTED, exit 0 — it is copy         |
| q12   | `let b = move a;` then `b[3_u64]`                          | REJECTED [OP-4], residual             |
|       |                                                            | "3_u64 < len(b)"                      |
| q13   | the control: the same subscript with no rebind             | ACCEPTED, exit 0                      |
| q14   | element writes in a loop, then a subscript of the same run | ACCEPTED, exit 0                      |
| q15   | three counters advanced on a flat three-arm `match`, six   | REJECTED [INV-1] `a_hi`, Backedge,    |
|       | header invariants                                          | required "ca <= (at + 1_u64)"         |
| q16   | the identical program as an `if / else if / else` chain    | REJECTED identically                  |
```

**`q8` is D1 at this tip and it is still an unsound accept**, which is why this design
exists; only [BLK-4]'s fourth clause refuses it. **`q2` against `q3` is the result this
draft's programs depend on**: under v0.42 a `borrow_expr` in a loop body that no
`region_stmt` encloses is a hard error and the explicit block is the fix the compiler
itself names — so **D4 is a real amendment that has not landed**, and 3.K.0 and §7's B0b
say so. **`q4` against `q5`** is the landed amendment's actual scope: regions only, type
and const arguments always written — the criterion the seventh draft got wrong at ten
call sites. **`q6`** is the ordered result list and **`q7`** the contract vocabulary;
`q7` means the *whole* contract surface of this design is new capability, not only
[CALL-7]'s completeness obligation. **`q9`** is the shape [LIV-2] admits and [STOR-1] 679
refuses. **`q10`** is [MSR-6]. **`q11`** is [PROV-6]'s admission condition on `linear`: a
tag-only enum is copy, so the modifier would mark a value the language duplicates.
**`q12` against `q13`** is [MSR-3]'s rebind placement. **`q14`** is [MSR-2]'s
element-write sentence across a loop.

**The join-shape asymmetry is reproduced, root-caused, and repaired outside this
design.** Round 7's writer lens reported six identical header invariants over a three-way
demux accepted as a flat `match` and rejected at [INV-1] on the backedge as nested
`if/else`. The orchestrator re-ran the two programs on the v0.42 gate (`c6` accepted,
`c7` rejected with `bb <= lb` at the backedge), and a separate investigation on `main`
found the cause in [ENT-6]'s control-flow join as v0.42 states it: a delta atom minted by
an earlier join counts as an ordinary nonconstant term at the next join, so the nested
shape's outer join compares `{h}` against `{h, d1}`, gives the binding a fresh atom, and
severs the header premise; the flat `match` joins once and keeps it. Controls pin it: the
same nesting with both inner arms incrementing (no delta) is accepted, and the
`invariant_stmt` escape in the arms does not help. The repair is a specification
amendment, not a design item: the v0.43 candidate (`batch/0120`) folds every earlier
join's delta atom into its interval before comparing images and takes the hull, so nested
joins reach exactly the flat join's image. This draft's own `q15`/`q16` pair failed on
`a_hi` for a different reason and is not evidence either way. Nothing in 3.K rests on the
answer; L16's one-goal-disposition claim is what the amendment restores.

Inherited verdicts this draft rests on. `d1` is D1 accepted. `e2`/`e3` — a callee whose
`ensures` names a borrowed run's measure across a `replace`, rejected then accepted with
the mutation deleted — locate [MSR-3]'s `&uniq` row. `gen1`-`gen3` compile
`&uniq Holder<T>` at `T = buffer<u8>`, which is [BLK-4]'s third clause. `pick4`/`pick`
show a function returning either of two same-region view parameters is legal, and
`setslice` shows a `set` at a `slice` binding is `AffineSetTarget` today, so [VIEW-4]'s
second refusal is new capability. `c8a`/`c8b` show [EFF-2] pinning an
`own`-parameter-writing row to exactly `writes(vector)`, which fires [MSR-3]'s mode
repair. `a1` is a `pure`, heap-free arena loop losing store bytes per iteration,
accepted; `a4` is `[OP-9] UndischargedAllocationFitObligation`; `a8` is the aborting
release walk with its `realloc`'d worklist. `w1` against `w2`, `w3`, `w5` price [CALL-2]
exactly — published relations compose to depth five, a chain without them fails at the
*second* link; `w6`/`w7` measured the seventh draft's `propagate` cost; `w8` accepts a
`set` at a `match` arm binder. `x1`, `x2`, `x3` are [CALL-4]'s three rejections, `x13` an
admitted route read at a caller's arm, `x14`/`x15` the copy/affine wall, `x10` an element
borrow as `Semantics/Unsupported: RegionsAndBorrows`. `s4`, `s5`, `s6` price [S27] and
Q19. `p5_ambient`, `n4`, `r2_5`, `q9` show allocation while holding nothing and a free
inside a `pure` callee accepted today. `f1c`, `f1d`, `f2b`, `r1_twouniq`, `w8` show a
view value, not its argument borrow, holds the loan. `f2b_tail`, `f8_tailframe`,
`p3_rec` refute the syntactic tail conditions; `n2_idle`, `f3_forever`,
`n3_propagate_loop` are `FunctionFallthrough`. `r2_6` and `m05` are the nominal
region-parameter parse errors; `r2_4`, `r2_4b`, `r2_4c` show the measure kill is
root-granular today; `q3`, `q7`, `x4`, `g7`, `p6_partial` show a partial move kills the
root and its residual is freed; `n14`, `n15`, `n19` show no loop publishes `len = N` as
an equality; `c8` shows a by-value transformation is not `pure`; `r1_relend`,
`r1_relend_affine`, `m19` are [PROV-7]'s reason; `k21`, `k21b`, `k08`, `k31`, `x1c`,
`x1d`, `g4` accept the fill loop's arithmetic and `g3` rejects it without the published
relation; `g1`/`g2` show `+checked` publishes only for a constant addend; `x6`, `x8`,
`p2_recarena`, `p3_rectype` are the recursive-type and recursive-region shapes;
`p1_reclose` and the three source reads of 6.2 are `reserve_file` and the io_uring
adapter; `p3_par` and `--par-ledger` are the permission verdict [RUN-1] makes auditable;
`p4_chain` and `p2_recur` are the stack ledger's one number and its uncomposed chain.

### 6.2 The runtime sources this design reads

```text
| source                                                   | what it shows                                 |
|----------------------------------------------------------|-----------------------------------------------|
| completion/linux_io_uring.c:425-450, 587-640             | every submission calls wf_linux_reserve_entry |
|                                                          | on a fixed entry_capacity table and waits     |
|                                                          | when full; WF_LINUX_FILE_CLOSE is one of the  |
|                                                          | submitted request kinds                       |
| completion/bridge.c:660-720, 780-796, 900-1240, 1504     | read_at, write_once, open_file, open_read,    |
|                                                          | open_directory, open_directory_source and     |
|                                                          | directory_next all take that path, and so     |
|                                                          | does wf_bridge_submit_linux_close; the        |
|                                                          | adapter initializes under pthread_once inside |
|                                                          | submit                                        |
| completion/linux_io_uring.c:242-300                      | the adapter holds three mmaps, sized from the |
|                                                          | kernel's returned ring parameters             |
| emitter/system.rs:2892-2901, backend/wf_floor.c:55, 78,  | reserve_file lowers to `ret i1 true`; the     |
|   234-247, 279, 292, 303-329; par_runtime.c:520-527      | floor mmaps a 64 KiB alternate stack PER      |
|                                                          | ATTACHING THREAD and returns silently on      |
|                                                          | MAP_FAILED; it attaches the host thread,      |
|                                                          | creates the entry thread, falls back to the   |
|                                                          | host thread silently when                     |
|                                                          | pthread_attr_setstacksize fails, and joins    |
```

The first two are why [RES-7]'s column is derived from the `may-suspend` target contract
**and quantifies over [SYS-5]'s release actions as well as [SYS-2]'s operations**. The
third is why [RES-1]'s host-object class is drawn at *countable versus extent*. The
fourth is why [RES-9]'s store is a design addition rather than a compiler defect, why
[STK-3] materializes every named stack, and why 4.1's envelope carries **four** `stack`
items. **Three [QUAL-2] failures of the shipping implementation are recorded rather than
hidden**: `bridge.c:670`'s first-use mapping inside the submit path, the floor's silent
`MAP_FAILED` return, and its silent fallback to the host thread.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in 3.K except B1's four.** [MSR-3], [MSR-5], [CALL-4] and [CALL-6] landed
  as the v0.44 candidate in their narrowed form (6.0); of the rest none is implemented and
  no compiler has seen any of the new types, operations, terms, statements, modifiers or
  markers. `q7` is answered for the clause operands and stands for the remainder of the
  contract surface: [CALL-4]'s measured result and widened routes, [S16]'s result list and
  [CALL-7]'s completeness do not parse or do not admit today.
- **Every function in 3.L** and **every program in section 4**, written against 3.K and
  the unchanged v0.41 rules and walked against both; none was compiled.
- **D3 itself.** That reading linearity against the scope removes one hundred and eight
  written statements without weakening L2, L3 or L17 is argued from [STOR-3]'s
  derived-release table, [LIV-1]'s join and [EFF-2] 1427, and is not executed. **6.4 asks
  for this first.**
- **[CALL-6]'s S13 over a [BLK-0] row.** The source-call half is implemented and
  `call6-pos-routed-relation-over-a-call-datum` is instantiation at the call with
  restriction to the arm under test (6.0). What stays reasoned is the same claim over a
  declaration-domain row, whose relations no v0.44 row carries, and the post-state
  destination that only such a row can reach.
- **[PROV-6]'s release graph** — that its least fixed point is the set of nodes the walk
  visits, that its acyclicity is the walk's termination condition, and that it keeps an
  arena-recursive structure compiling while refusing a heap-backed one. Probe `a8` is the
  mechanism it replaces and `x6` the type accepted today.
- **[PROV-1]'s brand and its resolution rule.** The brand has been attacked from every
  position in seven rounds and not moved, the strongest evidence any part of this design
  has; the **resolution** rule is one draft old and is what makes every hosted helper
  declarable.
- **[BLK-4]'s fourth clause** including its type-parameter half: that refusing the
  position rather than the reached type costs no program a writer needs. `gen3` is the
  declaration it refuses, compiling today.
- **[LIV-2]'s read-out** and its footprint at a subscript and at a loan-bearing target.
  `q9` and `r4` are the halves it replaces.
- **[RES-10]'s algebra** — sequence, branch, scope, call, loop, overlap over a label map,
  two routes, paired reset, extraction. The scope rule, the overlap rule and the
  extraction are new this draft and none has been composed against a program by hand.
  **6.4 asks for this second.**
- **The compiler defect at [SET-2]'s arena half**, found in round 3 and confirmed since:
  [SET-2] 517 makes a region-bearing `replace` target a hard error for `slice<'r, U>`
  **and** `arena<'r, U>`, and `check_mutation_target_class`
  (`compiler/src/semantic/check/expressions.rs:310-326`) tests only the slice variant.
  Benign at this tip; load-bearing for the batch that implements [PROV-3] use 3.
- **[MSR-3]'s other five placements**, checked by enumeration; the call placement is
  implemented (6.0) and the entry, construct, rebind, payload and field placements are
  not. **The current runtime's closure**, which no existing target can be certified to
  meet; and **the claim that `wfgrep` becomes heap-free**, whose substitution was never
  compiled, though its `append_slice` now compiles under [MSR-3]'s refusal.

### 6.4 Falsifiers this design asks for next

1. **Attack D3**, the largest change here: find a scope that holds a capability by
   signature and must not get the derived release, a program where the derived release
   runs on an edge a writer needed to control, a type whose leaves name two stores of
   which the scope holds one, or a way to make the release conditional on a runtime edge
   (L17).
2. **Hand-execute [RES-10]** on the corrected 4.1, on 3.L.4's pool, on a divergent
   service loop, on an arena take inside a loop, and on a `break` out of a per-iteration
   region block, checking route order, per-algebra transfers, `return`, `retained`, the
   scope rule, the paired reset and the extraction against all five.
3. **Attack [CALL-6]'s establishment point** with a routed relation whose support is
   written between the call and the arm, with two arms of one `match` establishing
   contradictory relations, and with a relation established at an arm reached from two
   predecessors.
4. **Attack [CALL-7] as now stated**: find a helper whose complete contract is unwritable
   under the syntactic clause condition, a measure whose only available fact is a
   standing bound, and a clause that satisfies the condition and tells a caller nothing.
5. **Attack [PROV-6]'s release graph** with a type whose non-emptiness fixed point is
   subtle — a mutually recursive pair, an enum one of whose variants is empty, a run of
   runs of arena runs — and with the modifier at a node the walk would otherwise visit.
6. **Write 3.L against 3.K by hand, one function at a time, and find the tenth kernel
   addition.** Rounds 5, 6 and 7 each found the yield high.
7. **Attack [BLK-4]'s type-parameter clause** with a `&uniq` parameter a real library
   needs, now that [S32]'s bound is adopted and the clause reads it.
8. **Rewrite `wfgrep` and `byte_string` by hand** against [VIEW-7], D3, R1 and 3.K.0's
   two amendments, and count what remains.

### 6.5 Falsifier rounds 1 to 6: what each finding hit, and what refuses it now

**Rounds 1 to 6 are history, collapsed into one table**, one line per finding group, with
the rule that refuses it in **this** draft. The five per-round tables the seventh draft
carried are deleted: they were written in the vocabulary of the drafts that made them,
named rules this draft has deleted, and cost about seven hundred lines to say what the
right column says. The reports are superseded and the audit trail is in git.

```text
| round and findings                                             | refused now by                             |
|----------------------------------------------------------------|--------------------------------------------|
| r1 F1-1,2 [OWN-11] refuses value-in/value-out; reinit set makes | [LIV-1] join agreement                     |
|   liveness path-dependent                                       |                                            |
| r1 F1-3,11-14,16 terms killed by their own operation; no        | [MSR-1] subscripted places; [BLK-1]'s      |
|   subscripted len; no FIFO, no exchange, no runtime target      | window; [BLK-3]; 3.L.2's transposition     |
| r1 F1-4..7 views have no loan strength; [BLD] cannot release    | [VIEW-2] the view value holds the loan;    |
|                                                                 | [BLD] deleted                              |
| r1 F1-8,9,10 a heap free exhibits nothing; the Heap may die     | [PROV-6] the release is a write; [LIV-1];  |
|   first; [STOR-5] omits container elements                      | [BLK-4]'s intensional split                |
| r1 F2-A1..A18 checked acquisition untypeable; tail lowering;    | [PROV-5], [STK-1], [RES-1..3], [RES-5..8], |
|   static providers; E's stack; composition not a function       | [RUN-1], [STK-3], [STK-4]                  |
| r1 F3-R1..R7, 2.2, 4.1-4.17, D1-D6 unregistered rules; clause   | 3.K.11's eight conditions; [MSR-5];        |
|   operands; the publishes column has no source                  | [CALL-6]                                   |
| r1 F4-1..9 room has no reader; no filled construction; per-     | [BLK-0]'s readers; 3.L.3; [MSR-4];         |
|   family proof routes; same-region view results alias           | [MSR-5]; [VIEW-6]                          |
| r2 F1-a1..a16 a move equality names a dead root; D1 on &uniq    | [MSR-3]'s datum; [PROV-3] uses 2 and 3;    |
|   MutSpan; a view at table[k]; release target not a function    | [PROV-6]; [PROV-7]; [RUN-3]                |
|   of the type; M(c,q); [PAR-2] denies the fill                  |                                            |
| r2 F2-N1..N16 two pools in one region; a frame extent confined  | [PROV-1]; [PROV-5]; [PROV-7]; [RUN-1];     |
|   to a caller; the loop rule discharged by an identity; stage   | [STK-4]; [RES-2], [RES-3], [RES-5],        |
|   one reads target data; a lane's chain undefined               | [RES-8]; [STK-3]                           |
| r2 F3-1..15, 5a-5k, 2a-3c container rows write unadmitted       | [BLK-0]; [CALL-6]; [MSR-5]; [VIEW-7];      |
|   arguments; per-arm relations have no route; [SYS-8] modes     | [RES-6]; [RES-7]; the register             |
| r2 F4-1..12 no stable-identity storage; FixedRing has no        | [BLK-1]'s window; 1.5's ruling; [BLK-4];   |
|   element access; no execution context; the rebind tax          | [LIV-2]; [MSR-2]; Q8                       |
| r3 F1-1..13 a provider-derived value in a field has no origin;  | [PROV-1] the store is in the type;         |
|   [CNT-5] and [PROV-6] disagree; one store per activation; a    | [PROV-5]; [PROV-6]; [MSR-2]; [MSR-3];      |
|   datum no producer mints; the kill fires on every element      | [PROV-3]                                   |
| r3 F2-NA1..NA13 a move plus a reinit set hands a lease to the   | [PROV-1]; [RES-5]; [RES-7]; [STK-1];       |
|   wrong store; an arena's cap dies; no ceiling data             | [PROV-5]; [RUN-1]; [RES-10]                |
| r3 F3-1..14, I1..I19 the datum has one producer and five        | [MSR-3]; [CALL-4]; [MSR-5]; [MSR-2];       |
|   consumers; a user multi-return publishes nothing              | [PROV-4]; the register; §7                 |
| r3 F4-1..12 a linear container has no disposal; no preservation | [PROV-6]; [PROV-1]'s closure argument;     |
|   clause; disposal is non-modular                               | [LIV-1]; [MSR-3]; 3.L.3                    |
| r4 F1-a1..a10 recursion makes two activations; a partial move   | [PROV-5]; [PROV-6]; [MSR-2]; [RES-9];      |
|   abandons a leaf; the element-position carve-out                | [PROV-2]; [MSR-1]                          |
| r4 F2-NB1..NB17 the disposal walk's scratch aborts; recursion   | [PROV-6]'s release graph; [PROV-5];        |
|   re-enters an extent; the handle table has no fact source;     | [RES-9]; [RES-7]; [RES-5]; [RES-10];       |
|   no (peak, delta) for a derived release; K<T> is a ceiling     | [RES-3]                                    |
| r4 F3, F4 the register's conditions cannot catch a retired      | 3.K.11 conditions 4a, 4b, 5; [VIEW-6];     |
|   subject; L17's ground excludes enums; diagnostics             | L17's whether/which split; §4's six        |
| r5 F1-1..12 an exit datum is D1 again; a view of runs is        | R1 as [BLK-4]; [PROV-6]'s ownership        |
|   linear; the walk has no action for a marked leaf; the queue   | closure and admission condition;           |
|   invariant is false                                            | [CALL-7]; 4.1's checked drain              |
| r5 F2-F5-1..F5-16 the arena take in a service loop; saturating; | [RES-5]'s kind column; [RES-8]'s           |
|   the exclusion test reads a runtime row; retained composes     | designator; [RES-7]'s stage split;         |
|   two ways; the stack item has no alignment                     | [RES-10]; [RES-2]                          |
| r5 F3-1..12, I1..I15 [BLK-0] names a source no rule states; a   | [CALL-6]; D2; 3.L.5; [VIEW-2]; [EFF-1]'s   |
|   provider relation has no destination; 4.2 redeclares          | canonical order; §7's partition            |
| r5 F4-1..13 [BLK-0]'s completeness binds thirteen rows; the     | [CALL-7]; [MSR-3]'s six placements; D2;    |
|   datum has three placements; a multi-target set exchanges one  | Q14, Q15, Q16, Q17; A.1                    |
| r6 F1-1..16 [VIEW-4]'s length-fixedness; the entry datum        | [BLK-4]; [CALL-3]; [MSR-3]'s denotation    |
|   republishes; containment makes a view linear; the walk        | table; [PROV-6]'s ownership closure and    |
|   discharges a marked leaf; three denotations                   | admission condition; [CALL-7]; 4.1; L18;   |
|                                                                 | Q13                                        |
| r6 F2-F6-1..F6-19 the arena loop certified bounded; linear is   | the owner's accounting ruling; [RES-5];    |
|   must-consume; the derived column misses [SYS-5]; route (iii)  | [RES-10]; [RES-7]; [BLK-0]; [RES-2];       |
|   is not a function; retained composes twice                    | [RES-8]; [RES-1]; [PROV-6]; [MSR-1]        |
| r6 F3-1..12, I1..I17 [BLK-0] names S13; a provider relation is  | [CALL-6]; D2; 3.L.5; [VIEW-2]; [CALL-4];   |
|   inadmissible; the chain is broken at three links              | [RES-10]; the register; 3.S                |
| r6 F4-1..13 the completeness quantifier; three placements; a    | [CALL-7]; [MSR-3]; D2; Q14-Q17; A.1        |
|   multi-target set; no bulk-move row; A.1 charges cap           |                                            |
```

Where a round-6 disposition was **false about its own draft**, round 7 found it and 6.11
records the correction rather than this table.

### 6.11 Falsifier round 7: what each finding hit, and what refuses it now

Round 7's diagnosis is one sentence in two voices. F1: **a fact computed at one point and
used at another, with the rule naming only the judgment and not the point.** F2: **a
repair that relocates the defect to the key rather than removing it.** 3.K.11's eighth
condition is the mechanical answer to both; the rows below answer the instances. All five
reports are superseded.

```text
| F1 (memory and fact soundness)                                | disposition                                 |
|---------------------------------------------------------------|---------------------------------------------|
| 1 BREAKS [MSR-3] keys a declaration-domain operand on `writes` | **[MSR-3]** keys every denotation on the    |
|   coverage, so seq_place publishes len(P) = len(P) + 1         | parameter's MODE: `own` = that call's CALL  |
|                                                                | datum, shared borrow = the live term,       |
|                                                                | `&uniq` = post-state in a declaration row   |
|                                                                | and inadmissible in a wf `ensures`.         |
|                                                                | [CALL-6] repeats the key; A.2's `<datum of  |
|                                                                | ...>` marker is deleted as redundant        |
| 2 BREAKS [CALL-6] establishes a routed relation AT the arm, so | **[CALL-6]**: every relation is             |
|   a store's post-state outruns its own kill                    | instantiated AT THE CALL and established    |
|                                                                | there, RESTRICTED to the arm, and killed by |
|                                                                | any write of the place at or after the      |
|                                                                | call. Deferring the point was the defect    |
| 3 BREAKS [MSR-2]'s element-write consequence is false once     | **[MSR-2]** states the granularity once,    |
|   len(table[i]) is a term                                      | over storage: an element-position write     |
|                                                                | kills every measure of P[i] and none of P,  |
|                                                                | for commit, replace and scalar write alike  |
| 4 BREAKS [CALL-3]'s two clauses contradict for a view whose    | **[CALL-3]** is stated over the viewed      |
|   element type is measured                                     | RANGE'S STORAGE — an element's own measures |
|                                                                | in, the origin place's out                  |
| 5 BREAKS/GAP [VIEW-4]'s ground is false at a copy view target  | **[VIEW-4]** refuses a `set` at a           |
|   and [LIV-2]'s footprint there is undetermined                | loan-bearing target on the same terms as a  |
|                                                                | `replace`; **[LIV-2]** defines its          |
|                                                                | footprint at a subscript and there          |
| 6 GAP 3.K.0's two candidate sets never intersect, so           | **[PROV-1]** states one brand resolution:   |
|   bs_reserve is ill-typed                                      | an elided store brand at a parameter or     |
|                                                                | result denotes the entry heap when the type |
|                                                                | is `Heap` or heap-derived, an implicit      |
|                                                                | region parameter otherwise                  |
| 7 GAP [PROV-6]'s declaration obligation refuses pool_new,      | **[PROV-6]**: the subject is an OWN-mode    |
|   collect, drain, render and every fn with a slice parameter   | value owning a leaf branded `'s`; the four  |
|                                                                | routes include the derived release; `'s`'s  |
|                                                                | store class is read from the declaration    |
| 8 GAP [BLK-4]'s closure stops at a generic type parameter      | **[BLK-4]**'s fourth clause refuses a       |
|                                                                | referent reaching an unbounded type         |
|                                                                | parameter; [S32]'s bound, ADOPTED, is what  |
|                                                                | the clause now reads                        |
| 9 DEFECT [LIV-2]'s commit paragraph contradicts condition 1    | **[LIV-2]** states the read-out: each       |
|                                                                | target's previous value is read out before  |
|                                                                | the evaluation, then the target is dead     |
| 10, 11 GAP/DEFECT [CALL-7] is vacuously satisfiable and its    | **[CALL-7]** is a syntactic per-measure,    |
|   "merely forwarded" exemption names a forbidden transport     | per-route clause condition with three       |
|                                                                | type-decidable exclusions; exemption gone   |
| 12 GAP a copy view's loan has two end conditions               | **[PROV-3]** fixes one: the loan ends where |
|                                                                | the view value's own liveness ends;         |
|                                                                | [VIEW-2] defers to it                       |
| 13 DEFECT the cyclic refusal reaches nothing ("through leaves")| **[PROV-6]**'s release graph: the walk      |
|                                                                | visits exactly its nodes and its acyclicity |
|                                                                | is the walk's termination condition         |
| 14 DEFECT [CALL-6]'s "at all" deletes [CALL-4]'s destination   | **[CALL-6]** says "the resolved place of a  |
|                                                                | BORROW actual"                              |
| 15 HOLDS (the window, dispose's resolution, [LIV-2]'s overlap, | preserved; [LIV-2] condition 2 gains "any   |
|   the static route)                                            | place reached through it"                   |
| 16 GAP two naming events are outside [MSR-3]'s six             | the closure sentence is narrowed to the     |
|                                                                | events the language undertakes to carry;    |
|                                                                | [CALL-6] says a `replace` publishes no      |
|                                                                | measures and 3.L.2 states take_at's cost    |
```

```text
| F2 (resource-closedness)                                      | disposition                                 |
|---------------------------------------------------------------|---------------------------------------------|
| F7-1 BREAKS the composition charges an overlap like a          | **[RES-10]** gains an OVERLAP rule (sum of  |
|   sequence, and the no-`par` obligation names an object that   | peaks, k*p for a staged permission);        |
|   does not exist                                               | **[RUN-1]** states the obligation over the  |
|                                                                | PERMISSION JUDGMENT, which is auditable     |
| F7-2, F7-14 BREAKS/DEFECT the arena's delta is an interval so  | **[RES-5]** makes len(arena) EXACT (the     |
|   the reset never cancels; A.1 and [RES-5] disagree            | alignment requirement makes padding zero);  |
|                                                                | **[RES-10]**'s reset is a PAIRED transfer   |
|                                                                | cancelling the block's own delta per label; |
|                                                                | A.1 has one bounded cell                    |
| F7-3, F7-7 BREAKS route (ii) can never fire and the domain key | **[RES-9]** fixes six SPEC-FIXED store      |
|   reads runtime data                                           | names; **[RES-8]**'s saturating takes a     |
|                                                                | store DESIGNATOR; **[RES-5]** keys a domain |
|                                                                | by it; **[RES-7]**'s source half quantifies |
|                                                                | over the closed set                         |
| F7-4 BREAKS route (ii) asks [MSR-4] for a goal about `delta`   | **[RES-10]**: the invariant route is        |
|                                                                | DELETED; two routes remain, both testing    |
|                                                                | compile-time data; the backedge delta is    |
|                                                                | computed, never proved                      |
| F7-5 BREAKS the extent item's identity across monomorphization | **[PROV-5]**: named by (concrete instance,  |
|                                                                | region_stmt NodePath); [RES-2] counts over  |
|                                                                | the expanded program                        |
| F7-6 BREAKS the handle table's refusal is keyed on an IoError  | **[S33]** is ADOPTED: the refusal is the    |
|   CLASS and no route publishes one                             | Exhausted VARIANT, and **[RES-6]** publishes|
|                                                                | room(factory) = 0 on that arm through       |
|                                                                | [CALL-4]'s existing route                   |
| F7-8 GAP a reserving occurrence inside a loop whose region     | **[PROV-5]**: the occurrence must be a      |
|   block is outside it has no stated meaning                    | statement of its block and of no loop in it |
| F7-9 GAP no rule extracts E's figure from the map              | **[RES-10]** states the extraction: the max |
|                                                                | over labels, never a sum                    |
| F7-10 GAP no composition for a scope                           | **[RES-10]** gains the SCOPE rule, applying |
|                                                                | each release and reset at every label at    |
|                                                                | which that edge leaves                      |
| F7-11 GAP route (i) refuses a requires-bounded trip count      | **[RES-10]** route (i) admits any closed    |
|                                                                | upper bound [MSR-4] establishes from the    |
|                                                                | endpoints and the requirements              |
| F7-12 GAP two altstacks, an unnamed host stack, no chain,      | **[RES-1]** makes an alternate stack a      |
|   silent materialization failures                              | STACK item; **[STK-3]** measures every live |
|                                                                | context; **[RUN-4]** makes StartFailed      |
|                                                                | mandatory; 6.2 records three [QUAL-2]       |
|                                                                | failures                                    |
| F7-13 GAP [STK-1]'s premise is read on one program and the     | **[STK-1]**: the dispatcher is an ordinary  |
|   frame measured on another                                    | function and is checked; the clause reads   |
|                                                                | "no derived release with a NON-EMPTY row"   |
| F7-16 GAP `seq_rebase`'s cost is understated: a driver pays it   | **[S29] WITHDRAWN** and 3.L.8 states the   |
|   before EVERY submission and cannot pay it while an I/O is      | full price — two runs of `n` live across   |
|   outstanding                                                    | the drain, an O(len) copy, a fresh spare   |
|                                                                  | per rebase, and the fact that a real ring  |
|                                                                  | driver hands the host two `iovec`s instead |
|                                                                  | and this language has no view of two       |
|                                                                  | ranges; Q18 puts the row to the owner      |
| F7-15, F7-17..F7-20 [S28] fails on its own program; 1.5 marks  | **[S28] REJECTED** by the owner; 1.5's row  |
|   an owed row inherited; two kind sets; slots carries no       | reads OWED; [RES-5] and [RES-10] share one  |
|   member size; pure reserving rows have no Depends             | kind set; [RES-2] gives slots a member size |
|                                                                | and alignment; [PROV-5] and [BLK-2] cite    |
|                                                                | [EFF-3] 1441                                |
| F2 round-6 re-verification: F6-2, F6-3, F6-6, F6-8, F6-11,     | preserved and not weakened                  |
|   F6-13, F6-14, F6-19 REFUSED                                  |                                             |
```

```text
| F3 (consistency)                                              | disposition                                 |
|---------------------------------------------------------------|---------------------------------------------|
| 1 DEFECT 4.1's loop-body borrows are hard errors under the    | **D4**: the loop body IS the region block,  |
|   amendment the draft says has landed                          | borrows are bare, an explicit block there   |
|                                                                | is a [FORM] rejection — and 3.K.0 and §7    |
|                                                                | say the amendment has NOT landed (q2, q3)   |
| 2 DEFECT 3.K.0's criterion covers region, type and const      | **3.K.0** is narrowed to REGIONS; every     |
|                                                                | type and const argument is written [FN-2];  |
|                                                                | [BLK-0] states the compiler-owned row's own |
|                                                                | retained-argument rule (q4, q5)             |
| 3 DEFECT [PROV-6]'s partial-consume refuses [LIV-2]'s field   | **[PROV-6]**: a consume of a sub-place      |
|   form on a linear value                                       | reinitialised at the same commit is not a   |
|                                                                | partial consume; [LIV-2] names the judgment |
| 4 DEFECT the declaration obligation lists three routes where  | **[PROV-6]** lists four, the derived        |
|   §2.1 lists four                                              | release included, over the right subject    |
| 5, 6 DEFECT [CALL-7] is violated by five printed functions    | **[CALL-7]** is a decidable clause          |
|   and is not decidable                                         | condition; 3.L adds nine clauses and        |
|                                                                | take_at is restricted to a non-measured T   |
| 7 DEFECT S29's L18 ground is false and seq_rebase has three   | **[S29] WITHDRAWN** to 3.L.8, which walks   |
|   statuses                                                     | and prices the wf program; Q18 puts the row |
|                                                                | back to the owner                           |
| 8 DEFECT twenty overshooting citations; three fixes only in   | every range re-derived mechanically in this |
|   the register; prove_ordering occurs nowhere                  | session from each rule's first and last     |
|                                                                | non-blank non-heading line; [MSR-4]'s       |
|                                                                | Amends names the [FN-9] sentence            |
| 9 DEFECT [BLK-0]'s Amends carries the "arm route"; §2.1       | [BLK-0] DEPENDS on [CALL-6] and amends      |
|   attributes S13 to [CALL-7]                                   | [ENT-3.S6] only; §2.1 names [CALL-6]        |
| 10 DEFECT A.1 marks four cells bounded, the prose says two    | A.1 has ONE bounded cell, a run's head;     |
|                                                                | [MSR-1] says so and [RES-5] is why          |
| I1-I17 register conditions 1, 3, 5, 6; five unstated          | the register is re-derived; every Publishes |
|   diagnostic names; [VIEW-2]'s lowercase v; seq_arena's free  | line names [CALL-6]; ConfinedTypeWithout-   |
|   identifiers; [VIEW-7]'s wrong parameter; twelve-versus-     | Store and LinearValueNotConsumed are stated |
|   thirteen operations; clear_bytes has no head invariant       | in [BLK-4] and §4; A.2 declares its consts; |
|                                                                | [VIEW-7] states each obligation over its    |
|                                                                | own parameter; the domain has TWELVE        |
|                                                                | operations everywhere                       |
```

```text
| F4 (writer) and F5 (linearity)                                | disposition                                 |
|---------------------------------------------------------------|---------------------------------------------|
| F4-1 BLOCKING [LIV-1] is per EDGE and the draft counts values:| **D3**, the owner's decision and F4's own   |
|   40 dispose statements in byte_string's main, 68 in          | recommended repair: a scope holding the     |
|   decode_dynamic                                               | capability gets the derived release on      |
|                                                                | every leaving edge. All 108 go              |
| F4-2 BLOCKING S28 is aimed at the wrong edge class            | **[S28] REJECTED**; D3 removes the problem  |
|                                                                | and Q10 keeps the multi-result question     |
| F4-3 BLOCKING acceptance depends on the shape of a control    | reproduced (c6/c7) and root-caused in      |
|   join                                                         | [ENT-6]'s join; repaired by the v0.43       |
|                                                                | candidate's associative join (batch/0120)   |
| F4-4..F4-10 FRICTION [CALL-7]'s cost; the exclusive loan;     | [CALL-7]'s three exclusions and Q14; Q19;   |
|   [MSR-3]'s quantifier; D2's read-out; no conditional grow;   | 3.L's nine clauses; [MSR-3]'s closure       |
|   the copy/affine wall                                         | sentence; [LIV-2]'s read-out; Q15; [S32]    |
| F5-1 BREAKS a modifier-linear nominal is never a walk leaf    | **[PROV-6]**: "no node of that graph — p's  |
|                                                                | own type included — is linear by modifier"  |
| F5-12, F5-13 BREAKS/DEFECT the walk is unbounded and the      | **[PROV-6]**'s release graph, quantified    |
|   acyclicity test looks at another graph                       | once by both                                |
| F5-19, F5-25, F5-26 BREAKS dispose is writable only in main   | **[PROV-1]**'s brand resolution (F1-6)      |
| F5-23, F5-2, F5-3 GAP the declaration obligation refuses      | **[PROV-6]** (F1-7) for the region axis;    |
|   every consuming helper; no value at a region parameter       | **[S32]** ADOPTED for both axes             |
| F5-4 GAP `linear` on a tag-only enum                          | **[PROV-6]** refuses the modifier on a      |
|                                                                | non-affine nominal (probe q11)              |
| F5-5 GAP the fourth ownership clause makes Tag<Vector<u8>>    | **[PROV-6]**: the clause is DELETED; a type |
|   linear while owning nothing                                  | owns its fields, payloads and elements      |
| F5-7, F5-14 DEFECT/GAP the copy view's loan never ends        | **[PROV-3]** (F1-12) and **[VIEW-4]**       |
| F5-15 DEFECT propagate's refusal is written twice, once wider | the [PROV-6] sentence is DELETED; [LIV-1]   |
|                                                                | states it per edge and [PROV-6] names it    |
| F5-16, F5-17, F5-18 BREAKS/GAP an inner and an outer [S28]    | **[S28] REJECTED** by the owner; every one  |
|   section each consume the same binding; the "exactly the     | of the three is a defect of the section it  |
|   same live set" condition fails on its own program; what a   | attacked, and D3 removes the cost it was    |
|   section may contain is unstated and written twice           | proposed to relieve                         |
| F5-6, F5-24 GAP a heap-backed variant infects every consume   | recorded in 3.L.7 and Q17: such a type      |
|   site; 3.L.7's test asks about a program                      | needs the provider at every consuming scope |
|                                                                | and 3.L.7's five shapes all need the PROVED |
|                                                                | return rather than the modifier             |
| F5-8, F5-9, F5-11 GAP value_if across two linear values;      | **D3 removes most**: in a capability-       |
|   break/give ladders; [S13] binds every field                  | holding scope every one is affine and the   |
|                                                                | release is derived. What survives is the    |
|                                                                | modifier-linear case, in Q13 and Q16        |
| F5-10, F5-20..F5-22 HOLDS (the no-exit loop's exemption, a    | preserved and not weakened                  |
|   shadowed capability, two candidates, a dispose in a loop)    |                                             |
```

**Five dispositions the seventh draft claimed and round 7 falsified**, recorded because a
false disposition is worse than an open finding: "every range re-derived mechanically"
(twenty overshot, three fixes existed only in the register); "[BLK-4] states the
diagnostic name" (`ConfinedFieldWithoutRegion` was claimed and stated nowhere — it is now
`ConfinedTypeWithoutStore` and is stated); "the [OWN-11] row states 647's disposition: it
is vacuous" (v0.42 keeps 647 explicitly under elision, probe `q2`); "the `par` rule is
deleted" (it was rewritten, and the rewrite was the unsoundness); and "[MSR-4]'s
`Amends:` names the [FN-9] sentence it meant" (it still named `prove_ordering`, which
occurs nowhere in the spec). Each is corrected in this draft's text.

---

## 7. Implementation order

**This is an implementation order and nothing else.** The owner's ruling of
2026-09-03 says so in terms: batches are an order of work, not spec versions, and a
single implementation is fine if it is correct. Nothing below is an approval, a
schedule, or a licence to trade a rule away for a cheaper batch; one batch that lands
all fifty-one rules correctly is the better outcome. The order is *for* naming, at each
step, a test writable before the next step exists. **Every rule is in exactly one
batch**: 4+4+4+2+1+1+6+5+5+2+4+11+2 = 51.

**Two amendments come before B1 and neither is this design's work.**

**B0a. Canonical region spelling.** v0.42's `[FORM-8]`. **It has landed**: probes `q2`
and `q3` show a bare loop-body borrow rejected at `[FORM-8] RegionSpelling` and the
explicit-block form accepted, and probes `q4` and `q5` show its scope is regions only.
Every batch below assumes it.

**B0b. The loop body is an implicit region block.** D4. A small, mechanical amendment
to `[OWN-11]`, `[FORM-8]` and `[GRAM-4]`: a `borrow_expr` inside a `loop_stmt` or
`for_stmt` body denotes that body's implicit per-iteration region and is written bare,
and a `region_stmt` that is the loop body's only statement is a `[FORM-8]` rejection
unless a type argument inside it must write its name. **Drafted and tested as the v0.43
candidate's first amendment on `batch/0120`, not yet merged.** Tests: probe `q2`'s program accepted; probe `q3`'s
program rejected at `[FORM]` with the mechanical fix `drop the block`; a loop-body
borrow whose loan is still live at the backedge refused by [OWN-11]'s unchanged
per-iteration judgment; and every loop-body borrow in `tests/programs` migrated. §4 and
3.L are written in this spelling and 3.K.0 says so.

**B1. The fact machinery. Landed as the v0.44 candidate (PR #17).** Rules: [MSR-3],
[MSR-5], [CALL-4], [CALL-6]. **First, because round 7's two memory BREAKS are both in it
and because nothing downstream is a fact without it.** Three things had to be pinned here
before anything read them: *where a declared operand's denotation comes from* ([MSR-3]'s
mode-keyed table), *where a declared relation is instantiated and where it is established*
([CALL-6]'s S13), and *what a contract may be written over* ([MSR-5] and [CALL-4]).
**Probe `q7` is why this was a batch and not a preamble**: `ensures len(kept) >= 1_u64;`
on a run result was a `[GRAM-5]` **parse** error, so the contract surface of this design
was new capability and no later batch's test could be written until it existed. Tests, all
six landed as conformance cases (6.0): a two-`len` clause accepted where `q7` is a parse
failure; a callee's relation over an `own` operand establishing at a caller **as the call
datum**, with the negative case pinned, a contract whose relations instantiate to a
contradiction refused at the `fn_decl` [CALL-6]; a routed relation instantiated at the
call and available only on its arm; a `&uniq` parameter's measure in a wf `ensures`
**rejected**, which probes `e2`/`e3` located; and a measure over a result of measured type
refused at [FN-9] rather than at the grammar, which pins the widening's boundary.

**What B1 could not reach, and where it went** (decided 2026-09-04). Three of the eighth
draft's B1 tests assumed machinery this batch does not build. *A measured result and*
`len(result)` need the result binder to be a place-like datum and not a fragment integer,
and *a per-variant route over any enum* needs resolver identity for variants beyond the
prelude `Ok`/`value`: both are [CALL-4] admissions and **land in B7** with the runs and
the measured types. *A two-result contract reaching both binders of a destructuring `let`,
both targets of a `set` target list and both arms of a `match`* needs [S16] and **lands in
B1b**. Probes `q12` and `q13` follow [MSR-3]'s rebind and payload placements and go with
them.

**B1b. Multi-return and the added destinations.** [S16]'s ordered result list, its
`let (a, b) = f(...)` and `set` target-list binders, and [CALL-4]'s three added
[ENT-3.S12] destinations, which only a multi-result contract exercises. **No rule of 3.K
is added here**: [CALL-4] stays B1's and this batch lands admissions B1 deferred, exactly
as B7 does for the measured result, so the arithmetic above is unchanged. Tests: a
two-result contract reaching both binders of a destructuring `let`, both targets of a
`set` target list and both arms of a `match`; a route naming a result ordinal, and the
omitted binder accepted only when one ordinal has that enum type; and a two-result
declaration whose two results are the same enum type refused when the route is ambiguous
[CALL-4].

**B2. The proof surface.** Rules: [MSR-1], [MSR-2], [MSR-4], [MSR-6]. Tests: probe
`q10` accepted after [MSR-6]; a goal discharged from `len + room = cap` as an affine
premise; an element-position `replace` of a **descriptor** killing its measures and of a
**scalar** killing nothing, which is the carve-out's removal under test; **probe
`r2_4`'s program accepted**, because [MSR-2]'s descriptor-precise support repairs a live
over-kill; a subscript in logical coordinates whose [OP-4] obligation is against `len`,
with [MSR-1]'s injectivity sentence exercised by two disjoint ranges over one wrapped
run; and a `set` at an element position of a run of runs killing `len(P[i])` and not
`len(P)`.

**B3. Type-derived call transports.** Rules: [CALL-1], [CALL-2], [CALL-3], [CALL-5].
Second in the live-defect order and needing none of the new types: today's
`&uniq buffer<T>` keeps its spelling and gets [CALL-5]'s type-derived classification.
Test: **`ent5-neg-callee-uniq-buffer-replace-kills-length.wf` turns XPASS**, rejecting
at [OP-4] with residual `9_u64 < len(line)`; plus probe `q8`'s program, whose accept
becomes the same rejection; plus one positive case pinning [CALL-1]; plus a callee
writing through a `MutSlice<'r, Vector<u8>>` killing `len(origin[0])` and keeping
`len(origin)`, which is [CALL-3]'s storage restatement. `docs/patterns.md` P16 is
corrected in the same change. **This batch flips a conformance case from `xfail`, which
is conformance evidence; the disposition is recorded in `governance/APPROVALS.md` with
the merge**, as B7's supersession is.

**B4. Liveness and one commit rule.** Rules: [LIV-1], [LIV-2]. Tests: **probe `q9`'s
program accepted**, and the same at a `deref`, a field and a subscript; a `set` whose two
targets are `v[i]` and `v[j]`, and one whose targets are `grid[k]` and `grid[i][j]`,
both **rejected** at condition 2; **probe `r4`'s program accepted when the inner `let`
becomes a `set`**; a swap and a three-target rotation accepted; a `move` of a target's
sub-place in the right-hand side **not** killing the root, which is the read-out
sentence; probe `f3`'s program a [LIV-1] error naming both predecessors instead of
`SemanticUnsupported`; and a loop moving and restoring an outer binding accepted where
probe `f5` is [OWN-11] today.

**B5. Linearity, the release, and the destructuring forms.** Rules: [PROV-6]. Ahead of
the container batch because D3's criterion is stated over release actions the language
already has. Tests: **probe `r2_5`'s program accepted with the release derived and
`writes(heap)` appearing in the row**, which is D3 under test, and the same program with
the `heap` parameter removed **rejected** as `LinearValueNotConsumed` naming the scope;
**probe `x4`'s program rejected with `LinearValuePartiallyConsumed`** and its
destructuring-consume repair compiling, while the same consume reinitialised by a
[LIV-2] commit is **accepted**; a `dispose` through a shared borrow rejected at [OWN-1];
a `dispose` of a `Slice<'r, Vector<u8>>` rejected at the loan-bearing operand condition;
a `dispose` of a type one of whose release-graph nodes is modifier-linear rejected; a
`dispose` with no live provider binding rejected as `DisposeHasNoProvider` and accepted
once the parameter is added, with the resolved binding appearing in the effect row;
**probe `q11`'s tag-only enum rejected when marked `linear`**;
**`fn checksum['s: affine](v: own Vector<'s, u8>) -> sum: own u64` accepted where the
unbounded declaration is refused, and a heap-branded instantiation of it rejected at the
call**, which is [S32]'s region axis under test; and **probe `x6`'s
self-referential heap type rejected at its declaration** in a program with no marker,
naming the cycle, **while its arena-backed sibling still compiles** — the release graph
under test.

**B6. Hand-back completeness.** Rules: [CALL-7]. Separated from B1 because it is a
declaration-site check over the vocabulary B1 lands. Tests: **a helper that hands a run
back without a clause for `head` rejected with `IncompleteHandBackContract`**; the same
helper with `ensures head(result) <= cap(result);` **still rejected**, because both sides
follow from [MSR-2]'s standing facts — which is the vacuity test and the half round 7
showed missing; the same helper with `ensures head(result) <= 0_u64;` accepted; a
`FixedVector<T, n>` result with no `cap` clause **accepted**, which is the type-decidable
exclusion; a routed contract missing a measure on one arm rejected naming that arm; and
3.L's nine added clauses compiled as a corpus.

**B7. The brand, the runs, the window, confinement, and the declaration domain.**
Rules: [PROV-1], [BLK-0], [BLK-1], [BLK-2], [BLK-3], [BLK-4]. Retires `buffer<T>`,
`box<T>` and `arena<'r, T>` from the writer surface, and carries monomorphization for a
compiler-owned generic domain. Tests: a `FixedVector<Handle, 64>` with affine elements
filled by 3.L.3's `vacant`, accepted, where probe `p9` is [OP-1] today; a queue built
from `seq_place` and `seq_take_front` with no `Option` anywhere; **a `seq_slice` over a
run that has had a front removal rejected, and accepted over the same run drained to
empty**, which is the non-wrap premise; **`bs_reserve` declared and compiling**, which is
[PROV-1]'s brand resolution under test, and the same function with `Bytes` given a
region parameter also compiling; `struct Chunk['s]` accepted where probes `r2_6` and
`m05` are parse errors today, with two instances at different regions rejected as
distinct types; **a `&uniq Vector<u8>` parameter rejected at [BLK-4], a `&uniq Env` whose
`Env` holds a `FixedVector` rejected the same way, and probe `gen3`'s `&uniq Holder<T>`
rejected at the type-parameter clause**, with the same declaration accepted under a [S32]
bound that excludes a container nominal and a loan-bearing argument; and two reserving
occurrences naming one region rejected at the second. **B1's three deferred [CALL-4]
admissions land here** (decided 2026-09-04), because this batch is where a result of
measured type first exists: a result of **measured** type carrying `ensures
len(result) >= 1_u64;` accepted where `call4-neg-measured-result-not-admitted` refuses it
at [FN-9] today; a measure over a result place formed with a field projection; a route
over a variant of a returned enum that is not the prelude `Ok`; and S13's population
**extended** to [BLK-0] rows, with a row's declared relation establishing at a caller
beside the source-call datums B1 landed. This batch supersedes B3's
conformance case, whose program no longer typechecks; that disposition is conformance
evidence and is recorded in `governance/APPROVALS.md`.

**B8. Views, loans, ranges.** Rules: [VIEW-1], [VIEW-2], [VIEW-4], [VIEW-6], [PROV-3].
[PROV-3] lands here because views are its only user. Tests: an element write through a
`MutSlice` accepted where probe `p7` is [SET-1] today; **a `Slice` used twice without
`move` accepted and a `move` of one rejected at `MoveOfCopy`**; **a `set` at a `Slice`
binding rejected by [VIEW-4]**, which probe `setslice` shows is new capability, and a
`replace` at one rejected the same way; **a run appended to after a copy view of it went
dead accepted, and the same append while the view is still used rejected**, which is the
loan's new end condition; two `MutSlice`s on one run rejected at the second formation
and two `Slice`s accepted; a write to `k` while a view formed at `table[k]` is live
rejected citing the view's loan; **a `Slice` formed as a shared child of a live
`MutSlice` accepted, an element write through the parent while that child lives
rejected, and the same write accepted after the child's last use**, which is [S31]'s
ruling and [PROV-3]'s end condition; a fill-and-publish helper that fills its
`&uniq MutSlice<u8>` destination, forms the child and returns it accepted at [VIEW-6]'s
ceiling; and a two-result signature with two same-region view results rejected at
[VIEW-6].

**B9. Stores, the heap as a value, and reservation.** Rules: [PROV-2], [PROV-4],
[PROV-5], [PROV-7], [RES-6]. Tests: probe `p5_ambient`'s program **rejected**; a `main`
that omits `command.heap` cannot reach any allocation; a run released to a store of a
different region failing to typecheck with the two types rendered; **a reserving
occurrence inside a loop whose region block is outside it rejected at [PROV-5]**, and
the same program with the block inside the loop accepted; a generic function carrying an
`arena_extent` occurrence, instantiated twice, publishing **two** `region` items;
**probe `x8`'s program rejected with `ExtentReservedOnACallCycle` under `arena_extent`
and accepted under `arena_frame`**; a helper lending a provider onward compiling, where
`r1_relend` and `m19` are [OWN-6] today; and two overlapped releases from one store
denied [PAR-1] permission.

**B10. System I/O over views, and the handle table.** Rules: [VIEW-7], [RES-9]. Tests:
`tests/programs/wfgrep.wf` migrated to 3.L.3's `filled` and `MutSlice`, compiling with
no `allocates` entry anywhere on its call graph — the first program that demonstrates
goal A's container half end to end; **a marked `main` that opens one file in a loop,
reads it into a `filled` destination over a `MutSlice`, and publishes a demand of one
on the named store `handles`**; an open that fails on every attempt whose handle records
all come back; **a `match` over `reserve_file`'s three arms deriving `room(handles) = 0`
on `Exhausted` and deriving nothing about `room` on `Failed`**, which is [S33] and
[RES-6] under test; **a `ReadFile` close counted as a may-suspend acquisition**; and
`write_once`'s range obligation stated over `source` and not over a destination it does
not have.

**B11. The stack judgment and the divergent entry.** Rules: [STK-1], [STK-2], [STK-3],
[STK-4]. Tests: probes `f2b_tail` and `f8_tailframe` **not** rewritten by [STK-1]'s
premise and rejected by [STK-2] under the marker; their borrow-free variants rewritten
into one dispatcher with one frame, **and the dispatcher itself checked by [LIV-1]**,
with a component whose members' live sets disagree refused at the component; a member
holding a live capability-released binding across the jump **rewritten**, because its
derived release carries a non-empty row only in a scope holding the provider — which is
the clause's new wording under test; probe `p3_rec` still accepted without the marker; a
`--stack-ledger` run reporting one chain per **live context** — the entry, the host, and
both alternate stacks — rather than one number; probe `f3_forever`'s idle loop and probe
`n3_propagate_loop`'s driver loop accepted; and a loop with a reachable `break` still
requiring a return.

**B12. The envelope and the judgment.** Rules: [RES-1], [RES-2], [RES-3], [RES-4],
[RES-5], [RES-7], [RES-8], [RES-10], [RUN-1], [RUN-4], [RUN-5]. Tests: 4.1
source-resource-closed and its `E` matching a pinned symbolic expectation, its four
`stack` items and its `slots` member sizes included; 4.2 reported not resource-closed
with the heap-reaching path rendered; **an arena take inside a loop with no region block
refused at that loop, and the same take inside a per-iteration region block accepted**,
which is the paired reset under test; **a `break` out of that per-iteration block
accepted**, which is the scope rule; a retaining loop whose trip count is a runtime value
rejected at that loop with the value named, **and the same loop under
`requires rows <= 8_u64` accepted and composed at 8**, which is route (i)'s new bound; a
loop with both a constant trip count and a saturating acquisition publishing the
trip-count map, which pins the route order; **a permit pool behind one helper accepted
through a declared `saturating(handles)`**, which is [RES-8]'s designator and [RES-9]'s
name under test; a service loop with no `break` whose pre-loop acquisition appears in the
enclosing `retained` entry; a peak reached only on a returning path appearing in the
`return` entry; **`E`'s per-domain figure equal to the max over labels and not the sum**;
and B10's marked file program failing **[QUAL-2] qualification** rather than a source
rejection when the profile cannot carry its `handles` demand.

**B13. `par` and the envelope.** Rules: [RUN-2], [RUN-3]. Tests: a `filled` plus
`MutSlice` plus counted subscript fill receiving [PAR-2] permission in an unmarked
program; **the same loop inside a `resource_closed` entry failing [QUAL-2] because the
permission judgment granted a permission**, and passing once the loop is written so it
does not — which is [RUN-1]'s auditable obligation under test, and which the
`--par-ledger` verdict emits beside `E`; **an unmarked overlapping window composing as a
SUM and not a max**; two overlapped statements allocating from distinct providers
permitted and two from one provider not; a window containing a release, a destructuring
consume and a multi-result `let` **each judged by its own footprint**; and **a `break`
and a borrow-forming `let` between two members still denying permission**.

**3.L is not a batch.** It is written against the rules, not implemented beside
them; where its functions are useful as evidence — `filled` in B10, `collect`, `vacant`
and the pool in B7 and B12, `bs_reserve` in B7, `rebase` in B7 — they land as test
programs under `tests/programs/`, which is where 5.1's Q on a standard library
recommends they stay.

---

## Appendix A: generated data

Two tables the rule text refers to and does not contain. **Neither is a rule.**
[BLK-0] says that an operation inventory exists and what every row of it must satisfy;
[MSR-1] and [RES-5] say that a measure table and a ceiling table exist and what every
row of them must contain. The tables themselves are **generated data**, carried the way
[SYS-2]'s declaration records are carried, and a diagnostic cites the rule and names the
row in its payload rather than citing the row.

### A.1 Measures and ceilings

Derived from [BLK-1]'s storage column rather than written per nominal. **Every cell is
one of `exact`, `bounded` or `absent`**, which is what [MSR-1] requires.

```text
| measured type            | len                | cap             | room      | head       |
|--------------------------|--------------------|-----------------|-----------|------------|
| FixedVector<T, n>        | initialized slots, | n, exact        | cap - len,| window     |
|                          |   exact            |                 |   exact   |   origin,  |
|                          |                    |                 |           |   bounded  |
| Vector<'s, T>            | initialized slots, | slots taken,    | cap - len,| as above   |
|                          |   exact            |   exact         |   exact   |            |
| slice, mut_slice         | viewed elements,   | len, exact      | 0, exact  | 0, exact   |
|                          |   exact            |                 |           |            |
| Arena<'s, bytes, align>  | cursor bytes,      | bytes, exact    | cap - len,| absent     |
|                          |   exact            |                 |   exact   |            |
| FileFactory              | live handle        | the profile's   | cap - len,| absent     |
|                          |   records, exact   |   `handles`     |   exact   |            |
|                          |                    |   capacity,exact|           |            |
| Heap<'s>                 | absent             | absent          | absent    | absent     |
```

`Heap<'s>` has no measure because L6 says a general store has no measure that means
anything; that is the absence of table data, not an exception clause. **Exactly one
measure is bounded anywhere — a run's `head` — and it is the one cell the two run rows
share.** An `Arena`'s `len` was the second in the seventh draft and it is **exact**:
[RES-5] requires `align >= align_ceiling(T)` at every take, so the cursor is a multiple
of `align` at every program point and the padding at a take is zero. Round 7 found the
two statements disagreeing and the recommended per-iteration idiom refused as a
consequence. Every formation row publishes `head = 0_u64` exactly, every back operation
publishes `head(result) = head(vector)` exactly, and only `seq_place_front` and
`seq_take_front` publish the two-sided `0_u64 <= head(result)`,
`head(result) <= cap(result)`.

```text
| nominal                     | (size_ceiling, align_ceiling)                          |
|-----------------------------|--------------------------------------------------------|
| Heap<'s>, Arena<..>         | (32, 16)   proof-only representation, one word         |
| Vector<'s, T>               | (32, 16)   a descriptor: pointer, cap, len, head       |
| FixedVector<T, n>           | T's pair repeated n times, plus (16, 8) for len and    |
|                             |   head, with aggregate alignment max(align(T), 8)      |
| Slice<'r,T>, MutSlice<'r,T>| (32, 16)                                               |
```

A `const` of `FixedVector<T, n>` type is element storage only [S34], because its `len`
and `head` are standing facts; the descriptor is materialized at each use.

**A `FixedVector`'s descriptor carries `len` and `head` and not `cap`**: `n` is the type
constant and [MSR-2] already makes it a standing fact with empty support.

**`advance<T>(count)`**, the bump domain's acquire quantity and the one compiler-owned
term former A.2's rows name, is `round_up(size_ceiling(T) * count, align)`, where
`align` is the store's own type constant. It is one [ENT-2] term of fragment type `u64`
with the support of `count`: a symbolic constant when `count` is a closed expression,
and an opaque term otherwise, so a relation over it is an ordinary difference bound
between two terms. Whether `count` is closed is [RES-3]'s question and is answered at the
acquisition.

### A.2 The kernel operation inventory

**Twelve rows**, plus the four readers, which are [OP-1] table rows and not this
domain. `V` is either run type. Every row is complete over **every** measure it writes,
on every exit, as [BLK-0] requires; every effect row is written in [EFF-1] 1369's
canonical order; every relation is established by [CALL-6]'s S13; and **every operand
denotes what [MSR-3]'s table gives its parameter's mode** — an `own` operand is that
call's call datum and a `&uniq` state operand is the post-state — so the seventh draft's
`<datum of ...>` marker is deleted as redundant rather than load-bearing on three rows
and absent on ten.

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
                      len(arena) = len(arena at the call) + advance<T>(count)
      None:           len(arena) = len(arena at the call),
                      room(arena) < advance<T>(count)
      both:           cap(arena) = cap(arena at the call)
  seq_arena_proved<T, const bytes: u64, const align: u64>['s](
        arena: &uniq Arena<'s, bytes, align>, count: own u64)
      -> own Vector<'s, T>               reads(arena), writes(arena), allocates(arena)
      requires align >= align_ceiling(T)
      requires fits::<T>(count)
      requires room(arena) >= advance<T>(count)
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
                          its contribution to stack(context, ...) [PROV-5, STK-3]
  arena_extent<const bytes: u64, const align: u64>['s]()
      -> own Arena<'s, bytes, align>                                                pure
      len(result) = 0, cap(result) = bytes, room(result) = bytes
                          its own region item, named by (instance, NodePath) [PROV-5]

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
  seq_slice['r, T](vector: &'r V)          -> own Slice<'r, T>        reads(vector)
      requires head(vector) + len(vector) <= cap(vector)
      len(result) = len(vector), cap(result) = len(vector),
      room(result) = 0,          head(result) = 0
  seq_mut_slice['r, T](vector: &uniq 'r V) -> own MutSlice<'r, T>    reads(vector)
      requires head(vector) + len(vector) <= cap(vector)
      as the row above
```

Two statements are not rows and are stated in [PROV-6]: `dispose p;` [S12] and the
destructuring consume `let N(f1: b1, ...) = move v;` [S13].

Notes on the inventory. **`seq_place` is the operation the whole design exists for**:
total under its requirement, allocation-free on every backing, one store plus one length
increment — and its `vector` operand is `own`, so every occurrence of `len(vector)` in
its relation is the length it was handed [MSR-3]. **The four per-slot rows are two-sided
because L12 is**, and the front pair is what makes a queue a run rather than a run of
`Option`. **There is no fifth boundary row**: returning a wrapped window to its origin is
3.L.8's drain, which L18 keeps out of the kernel and Q18 puts back to the owner. **And
there is no third view row**: forming a shared `Slice` from a `MutSlice` is [OWN-6]'s
child reborrow [VIEW-6], so `seq_reslice` is not adopted [S31] and the count stays at
twelve.
**Nothing here is total at a capacity boundary**, because an overwriting form would need
L9's published displacement. **Nothing here removes from the middle, clears, truncates,
grows, exchanges, swaps, rebases, or constructs a filled or vacant run** — each is 3.L,
and 3.L.6 records that none needed a row the four boundary operations do not have.
