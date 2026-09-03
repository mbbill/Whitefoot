# Containers and resources: the integrated design

The single design for batch 0116. It merges the two drafts beside it,
`RESOURCES.md` (providers, the envelope `E`, the `resource-closed` judgment) and
`CONTAINERS.md` (owners, views, and the facts that cross a call), into one set of
laws, one set of rules, one vocabulary, and one amendment register. A reader who
has not read either draft can read this file alone. The drafts remain for their
detailed rationale, their rejected alternatives, and their probe registers; every
rule they stated normatively now lives here.

**Second draft, after falsifier round 1.** Four adversarial reports attacked the
first draft: memory soundness, the resource-closed judgment, internal and
specification consistency, and writer usability. They found five kinds of defect
that were not repairable by patching a sentence: the arithmetic of the operation
table had no term semantics, the proof surface was granted per consumer family,
the view types had no stated loan strength, the resource judgment quietly took
target and runtime data as premises of a source rejection, and a whole family
(`Builder`) certified something other than what it claimed. Section 6.5 lists
every finding and the rule that now refuses it; the reports themselves need not
be kept.

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
- Append helpers take the owner by value and return it:
  `set buf = collect(source: move line, out: move buf);`. Pass-by-pointer is only
  an ABI.
- Three call rules: through a shared borrow all facts survive; through a value
  passed and returned only contract facts survive; an element write through a
  length-fixed view never touches length.
- Mutation of container state through `&uniq` is retired.
- Multi-return `-> (a: own T, b: own U)` with `let (a, b) = f(...)`.
- System I/O goes over views.
- Every rule is a deterministic function of program text and compiler version,
  never of time or of a work budget.

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
whose whole lifetime is deterministic in the sense that matters to that trade:
it cannot corrupt memory, it cannot race, it cannot read an uninitialized byte,
it cannot silently overflow, and it also cannot die because a store ran out.
Today the language delivers the first four and not the fifth. [SCOPE-3] (27-31)
still leaves heap exhaustion, stack exhaustion, operating-system quotas and
runtime-start resources outside the source outcome model, so an accepted program
may stop at the host boundary with no Whitefoot value, no status, and no cleanup.
For goal A that is the whole ballgame: a program that can vanish at three in the
morning has not removed the class of failure the writer came here to remove.

The owner's shape for goal A is a **promise**, not a guarantee about the world.
The compiler computes one finite, shaped envelope `E` of tangible resources; the
program promises never to demand more than `E`; the environment then decides
whether it can deliver `E`. Only the conjunction gives freedom from exhaustion.
A program that reaches the heap makes no such promise, because total free bytes
cannot answer a request for a contiguous aligned extent.

**Goal B: with a heap, be honest.** A hosted program wants the heap and should
have it. What it must not have is a hidden trap. Today it has one: allocation is
ambient (any function may allocate while holding nothing, verified below), the
only record is an effect-row atom, and refusal ends the process. Goal B asks for
allocation to be an operation on a value the caller holds, and for refusal to be
an ordinary typed outcome that hands back every affine input it did not consume.

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
(`compiler/src/semantic/places.rs:349-355`) classifies a projected callee write
as an *element* write for every `&uniq buffer<T>` actual, from the **actual's
syntactic shape**; `event_kills_term`
(`compiler/src/semantic/entailment/flow.rs:2927`, `TermKind::Length` arm at
2951-2956) then honours [ENT-5]'s true sentence "an element write never kills a
length fact" and keeps the stale length. The specification is right and the
compiler is wrong, but repairing the flag repairs one sentence. The defect class
is that a caller inferred a fact-preserving property of a callee from information
about the **call site**. The callee's signature `writes(handle)` does not
distinguish an element write from a whole-referent replace, and nothing in
[EFF-2]'s row can be made to.

That class is what a container design multiplies. A caller wants to keep, across
a call: a container's length, its capacity, its initialized prefix, the remaining
spare of an append window, the disjointness of a range handed to a lane. Every
one of those, if justified by anything other than the callee's declared types and
declared contract, is another D1 waiting to be found by the next sweep.

### 1.3 What the design therefore has to do

Turn every resource a program can exhaust into a value it must hold in order to
consume, so that "this subtree cannot touch the heap" is a signature fact and
"this program's peak demand is this list of extents and slot counts" is a
compiler judgment. Give the writer one declaration that turns the second into a
compilation requirement. Make every failure to obtain a resource a typed value
that returns the affine inputs it did not consume. Put the runtime inside the
same envelope as the writer's code. And make every fact that survives a call
readable from the callee's declared parameter modes, declared types, and declared
contract, so D1 has no expressible form.

The first draft did all of that and was still unusable, because it had no answer
to a second question: *what is a length, arithmetically?* A container surface
whose central operation cannot be discharged inside a loop is a specification of
a language nobody can write. Section 3.1 is therefore the first family in this
draft rather than an open question at the back of it.

---

## 2. The laws

Seventeen laws. Every rule in section 3 is an instance of one of them, and **a
rule that cannot name its law is not admitted.** L1 through L9 are the resource
laws; L10 through L15 are the container laws; L16 and L17 are the two the
falsifier round added, and they are the two that carry the most rules. Each
states its rationale in one sentence and names the owner ruling or the evidence
it rests on. Owner rulings cite `EVIDENCE-owner-discussion-2026-08-31.md` by
ruling id (R2 through R14) and accepted-conclusion id (B1 through B12).

**L1. The envelope is the program's promise, and the promise is made in two
stages.** *A resource-closed program declares one finite, shaped envelope `E` and
promises that on every legal execution, and on every finite prefix of an infinite
one, its demand for covered resources never exceeds `E`. The judgment that a
program makes that promise is a source judgment, a deterministic function of
program text and compiler version alone. The computation of `E`'s concrete
figures, and the check that a selected target and runtime can carry them, is a
target-stage qualification obligation whose failure cites no language rule.
Whether an environment then supplies `E` is a third fact, about the deployment.*
Rationale: acceptance may not depend on a register allocator, an optimizer, or a
linked runtime; those decide whether this build of an accepted program can be
shipped, which is what [STOR-6] and [QUAL-2] already decide for every other
post-codegen figure.
Rests on: owner ruling R13 (`L7036`), "do not get the direction backwards"; B8;
[SCOPE-2] 18; [STOR-6] 745.

**L2. No resource is ambient.** *Every covered resource enters the program as a
capability value the runtime hands to `main`, or as a store the program reserves
in an activation it owns, and travels only by ordinary ownership; there is no
ambient allocator, ambient thread source, or ambient stack pool.*
Rationale: an effect row describes what a body did, while a held value is an
authority the body had, and only the second makes "this call graph cannot reach
the allocator" a signature fact rather than a whole-program re-derivation.
Rests on: probe `p5_ambient` (section 6), a nullary leaf function that allocates
while holding nothing, **accepted today**; and [FN-7] 1242, "there is no ambient
system state", whose last exception this law removes.

**L3. Nothing fails silently, and nothing grows behind the writer.** *Every
operation that can fail to obtain a covered resource returns a typed value naming
the failure and handing back every affine input it did not consume; no operation
traps, aborts, retries, falls back, or promotes a store to a larger one.*
Rationale: v0.40 has zero writer-reachable runtime-trap families (spec line 6)
and yet heap exhaustion still ends a process with no source value, so the
trap-freedom claim is not yet honest for this one family.
Rests on: owner ruling R12 (`L5657-5666`), a pool with a silent fallback is worth
nothing; B3; audit answer Q8.

**L4. No hidden growth.** *No operation both uses existing capacity and acquires
new capacity; every operation that may acquire capacity takes an owner and a
provider, names its allocation effect, and returns a typed failure, while every
operation that only uses existing capacity is total under a proved capacity
requirement and can allocate on no path.*
Rationale: one `push` cannot carry one return type and one effect row across
backings, and a growing push inside a loop leaves partial commitments that no
clean semantics describes.
Rests on: owner ruling R5 (`L2332`) killed the no-growth-at-all form; B2, B3, X1.

**L5. The runtime is inside the envelope.** *The artifact `E` describes is the
writer's code, the compiler-derived cleanup and drop glue, the `par` runtime, and
the target adapter together, from the frame the environment hands the program to
the frame it takes back; a resource any of them needs is an item of `E`, or the
program is not resource-closed on that target.*
Rationale: a guarantee that stops at the edge of generated code is not a
guarantee; the current runtime creates a worker thread on first `par`, maps a
diagnostic stack when a lane starts, initializes a completion ring lazily, and
reallocates a cleanup worklist, and the existing `--stack-ledger` reports `main`
and the entry body as two disjoint roots, so the frames beneath `main` are in no
number today.
Rests on: owner ruling R12, "the runtime must meet every requirement of
`res-closed`, and if it cannot, you must tell me why"; B12; the ledger read in
section 6.

**L6. Shape, not bytes.** *`E` is a list of tangible resources (contiguous
aligned extents, per-class slot counts, per-context stacks, lane counts) and
never one byte total, because a byte total cannot express the request a fragmented
store cannot serve.*
Rationale: sixteen bytes holding four four-byte objects, with the first and third
released, have eight free bytes and cannot serve an eight-byte request; alignment
is an independent counterexample.
Rests on: owner ruling R12, "even giving the heap a cap does not guarantee space
is available: the heap also has internal fragmentation"; B9, B11.

**L7. Lowering before judgment, and a tail call is a dead caller frame.** *Tail
recursion, including mutual tail recursion, is rewritten into loops by the
compiler before any resource judgment runs; an intra-component call edge is a
tail edge exactly when the caller's activation record is dead at the jump, and
never because the call is written in a return statement.*
Rationale: an optimization that may or may not fire cannot be a premise of a
guarantee, and an enumeration of syntactic conditions cannot see the transitive
case where an earlier member of the chain still has a live loan into its own
frame.
Rests on: owner rulings R3 (`L989`) and R12 (no depth certificates); B10; probes
`f2b`/`f8_tailframe` (section 6), a mutual tail recursion carrying a live borrow
of a caller local, **accepted today**.

**L8. Demand is computed as if every acquisition succeeds; a store's own refusal
is an ordinary fact.** *The resource judgment replays each execution assuming
every covered acquire succeeds, and may never conclude that demand is small
because a failed acquisition would have ended the program. It does read the
store's own post-state relation on a refusal edge, because `len(store) = cap(store)`
is a fact about the store, not a claim about the program's survival.*
Rationale: the first half removes the circularity, and without the second half
the checked spelling of an acquisition changes a loop's summary by nothing, so
the typed-refusal protocol L3 requires is unusable in exactly the loops it exists
for.
Rests on: B8's "every legal execution and every finite prefix"; owner ruling R12,
which requires the `Result` form to be worth having.

**L9. Stock, not flow, and a total operation at a capacity boundary must say what
it dropped.** *Resource-closedness bounds what is held at once and what is
consumed irreversibly; it never bounds how many times a program acts. An
operation may be total at a capacity boundary only when the value it displaces is
copy and its displacement is a published relation the caller can read; a silent
drop of an affine value, or a total operation with no published relation, is a
refusal wearing a disguise and is inadmissible under L3.*
Rationale: the first half is why a service loop that takes a slot, uses it, and
releases it runs forever with one live slot; the second half is why "overwriting
is this ring's defined semantics" cannot be written on every bounded store until
the judgment has no content left.
Rests on: B8 (finite prefixes, not finite lifetimes); owner ruling R12 on silent
fallbacks.

**L10. A view is a value, and it holds its own loan.** *A view is an affine value
with a static type, not a reference the callee writes through and not a hidden
pointer to the owner's header; it holds, for its whole life, a loan of its own
strength on every place in its origin set, beginning at formation and ending when
the view value is consumed or released; a function that changes a view's state
consumes it and returns the new one.*
Rationale: the first clause answers the write-back problem without a hidden
protocol, because the advanced `len` is the result value; the second is what the
first draft asserted and no rule supplied, and its absence admitted two
`AppendView`s over one owner and an uninitialized read.
Rests on: owner's settled decision of 2026-09-03 (views transformed by value,
`set buf = collect(...)`); B6; probes `f1c`, `f1d`, `f2b` (section 6).

**L11. Length is a type fact or a contract fact, never a guess.** *At every
program point the checker's knowledge of a sequence's measures comes from exactly
one of: the type, an established fact with live support, or a verified contract
relation; no rule infers a measure from the shape of an argument, the name of a
callee, the absence of a write, or what a body was seen to do.*
Rationale: this is D1 stated as a law, and it is why the repair is not "fix the
flag" but "have no flag derived from an actual to be wrong".
Rests on: `EVIDENCE-sweep-D1.md`; probe `d1` accepted today (section 6).

**L12. The initialized prefix is a stack, and the language says so.** *A prefix
sequence's storage is exactly `[0, len)` initialized and `[len, cap)` raw; the
boundary is checker-maintained typestate carried by the owner's static type, and
no per-slot tag, `Option` wrapper, occupancy bitmap, or runtime discriminant
exists. A prefix admits append at the end, removal from the end, removal from the
middle by exchange with the end, and exchange of two positions; it does not admit
removal from the front, and a kernel that needs a queue gets a second owner whose
initialized region is a rotation of the prefix, not a weakening of this law.*
Rationale: with no per-slot state the checker never needs a quantified
proposition over slots, only the scalar relation `len <= cap`; and the first
draft advertised `FixedVector<Handle, 64>` as an object table while providing a
stack, which the falsifier demonstrated by writing the object table and failing.
Rests on: owner's settled decision (`FixedVector<T, N>` holds affine `T` through
an initialized-prefix typestate); audit answers Q2, Q4, Q10.

**L13. Release belongs to the owner's backing type, and every provider-owned
release names its provider.** *The release action of a value is fixed by its type
under [STOR-3] and by nothing else: drop `[0, len)` in ascending index order,
then the backing's own release. A release of provider-owned storage exhibits
`writes` on the path naming its provider, and the edge carrying it must reach a
live writable provider place. No source construct selects, replaces, or observes
a release action.*
Rationale: the first draft gave a heap free an empty row and a pool release a
`writes(pool)` row for the same event, which made two concurrent frees invisible
to [PAR-1] and left nothing ordering a `Heap`'s death after its allocations';
one rule for all providers removes both defects and a special case with them.
Rests on: B2's drop order; audit answer Q10; [STOR-3] 683; [EFF-2] 1421.

**L14. An `AppendView` reaches only what it appended.** *An `AppendView` presents
the spare window `[base, cap)` of its owner, where `base` is the owner's length at
formation; its own `len` counts what was appended through it and starts at zero,
no operation on it reaches an index below `base`, and no operation on it decreases
the owner's length.*
Rationale: this is what lets a caller's length fact stay alive across a callee
that appends, so the design does not buy soundness by discarding every length
fact at every call.
Rests on: B6; the owner's third call rule of 2026-09-03.

**L15. The descriptor's capacity is a value; the allocator's extent is not.**
*`len(v)`, `cap(v)` and `room(v)` are the descriptor's own logical measures and
are readable as ordinary `u64` values. No operation observes the physical extent
the allocator actually provided, and every operation that changes a descriptor's
capacity publishes the exact new capacity, so an allocator that rounds a request
up changes no accepted program's behavior and no program can observe whether
growth was exact, 1.5x, or 2x.*
Rationale: the first draft forbade reading `cap` and `room` at all, on a
rationale that only forbids reading the allocator's size; the consequence was
that `len` was readable so every pop proved and `room` was not so no push ever
did, which is not a decision anyone made.
Rests on: B3's "logical capacity is a language value in the descriptor and is not
the allocator's usable size", read as written; audit answer Q9; probes `q24`,
`v25`, `v26` (section 6).

**L16. One measure algebra, and one goal disposition.** *`len`, `cap` and `room`
are one-place terms of the term language, defined once with their support, their
kills and their standing identities, over every measured place: sequence owners,
views, and providers alike. Every consumer of a numeric goal asks one question,
whose complete admitted derivation is stated once; no rule grants a proof route
to a construct by name.*
Rationale: a language in which "can this inequality be derived?" has a different
answer depending on which construct is asking has several provers, and a writer
can reason about none of them; probes `v25` and `v26` are the same proof asked
twice with opposite verdicts, inside one function in probe pair `q2b`/`q3b`.
Rests on: [ENT-1] 2645, "a closed, deterministic, terminating derivation system
fixed completely by this specification", read as a promise the route lists break.

**L17. Affine liveness agrees at every join.** *A binding's live-or-dead status
must be the same on every predecessor of every join and at every loop head; a
disagreement is a hard error at the join. Consequently a compiler-derived release
on a scope-exit edge is unconditional, exactly as [STOR-3] requires, and there is
no runtime state that says whether it should run.*
Rationale: the reinitializing `set` the value-in/value-out idiom needs makes
liveness non-monotone, and [OWN-11] and today's `SemanticUnsupported: OwnershipJoin`
are two ways of avoiding the question rather than answering it; without this law
one static edge has two runtime dispositions and [STOR-3] must either double-free
or leak.
Rests on: probe `f3` (section 6), `Semantics/Unsupported: OwnershipJoin`; [ENT-5]'s
own all-predecessor join, which is the same discipline for facts.

---

## 3. The rules

Ten families. `[MSR]` is the measure terms and the proof surface, `[PROV]` the
capability values and their operations, `[RES]` the covered set, the envelope and
the judgment, `[STK]` the stack, `[RUN]` the runtime's own closure and the
environment's half of the bargain, `[CNT]` the sequence owners and their
typestate, `[VIEW]` the views and the commit event, `[LIV]` affine liveness and
reinitialization, `[CALL]` what survives a call, and `[SEQ]` the operation
inventory. Each rule states the judgment it creates, the fact it publishes, and
what it amends; section 3.13 collects every amendment in one register so nothing
is changed silently.

The family is `[PROV]` and not `[CAP]` because [CAP-1] already exists (1962) and
rule ids are never reused. The collision is worth a sentence: [CAP-1] says the
kernel defines *no writer-visible capability category and no system-specific
permission*, and this design does not add one. A provider is an ordinary affine
value, held under `own` or `&uniq`, judged by place overlap and by the ordinary
effect row, and interfering with other statements through exactly the vocabulary
[CAP-1] names. "Capability" here means *a value you must hold in order to act*,
which is what `FilePermit` already is, not a second permission system beside
ownership.

Two families the first draft had are gone. `[BLD]`, the `par` builder, is deleted
outright: its coverage certificate certified an index range rather than a set of
writes, its abandonment release could not be written under L12, and the [PAR-2]
permission it existed to obtain was denied by a condition it did not amend. What
it wanted is served with no new type by [SEQ]'s filled construction plus a
`MutSpan` plus direct subscript writes, which is exactly the shape [PAR-2]
already permits. The old `[BLD]` ids are retired and not reused.

### 3.1 `[MSR]`: measures, and the one goal disposition

This family is first because everything else consumes it. It is also the family
that is a specification amendment rather than a new construct: it adds no
statement form and no type, and it deletes two proof-route lists.

**[MSR-1] Three measure terms, over one place, for every measured value.**
`len(P)`, `cap(P)` and `room(P)` are terms of the [ENT-2] term language, of
fragment type `u64`, where `P` is an admitted place. They are defined once, here,
for every *measured* type: `array<T, N>`, the sequence owners [CNT-1], the views
[VIEW-1], and the providers [PROV-1]. Their meanings are fixed per type by
[CNT-1], [VIEW-1] and [PROV-1] and by nothing else; for a provider they are the
store's own occupancy, capacity and spare, so a pool's `len` is its live slot
count and an arena's `room` is its remaining bytes.

An admitted place for a measure term is a `place` [GRAM-5] formed with field
selections, `deref` wrappings **and subscripts**, whose final selected type is a
measured type. The subscript admission is the change: `len(table[i])` is a term,
so a container of containers has provable operations. A subscripted place's own
[OP-4] obligation is judged independently and is not weakened by occurring under
a measure term.

*Judgment:* none by itself. *Publishes:* the term. *Amends:* [ENT-2] clause (b),
which today admits `len(P)` only for `array`, `slice` and `buffer`, and only for
subscript-free places. *Retires:* the separate provider vocabulary `live`,
`capacity` and `remaining` that the first draft proposed; six terms with two
algebras were three terms with one. *Law:* L16.

**[MSR-2] Support, kills, and the standing identities.** The support of a measure
term over `P` is `P`'s root binding, every borrow or `box`/`arena`/`slot` holder
`P` reads through, and, when `P` ends in a subscript, the offset's own support and
the collection's element storage. It is **not** the measured value's element
storage: an element write never kills a measure, exactly as [ENT-5] already says
for `len`. A whole-place write, replace, consume, projected callee write, or
scope exit reaching the root kills every measure over it.

At every program point at which `P` is live, these hold implicitly:

```text
Z <= len(P)          Z <= room(P)          len(P) <= cap(P)
len(P) + room(P) = cap(P)
cap(P) = N           for a type whose capacity is the constant N
```

The first three are difference bounds and live in L0. The fourth is a three-term
identity and lives in the affine domain, where [INV-1] already carries relations
of that shape; it is not copied into L0, whose uniqueness and finiteness argument
[ENT-4] 2854 rests on the difference-bound shape. Deleting that identity is what
made the first draft's `room` an unrelated quantity that happened to share a
spelling with a capacity and a length.

A move of a measured value into a fresh binding carries its measures: `let x = move p;`
and a measured `own` result bound by an ordinary `let` establish
`len(x) = len(p)`, `cap(x) = cap(p)` and `room(x) = room(p)` at the binding and
transfer their images [MSR-3]. This is [ENT-3.S5]'s copy-equality row read for
measured values, and without it every `let acc = move out;` in section 4 would
lose the contract fact its `requires` had just established.

*Judgment:* none. *Publishes:* the implicit facts and the move equalities.
*Amends:* [ENT-2]'s implicit fact sentence (2722), [ENT-3.S5]'s equality rows, and
[ENT-5]'s support sentence (2857). *Law:* L16.

**[MSR-3] Measures carry affine value images.** [ENT-6]'s affine value-image map
holds a current image for every measure term of a live measured place, formed and
transferred exactly as it is for a live own integer binding: an operation's
declared relation over its parameter's entry image and its result ([SEQ-0])
installs the result's image, a whole-binding `set` [LIV-2] makes the target denote
that image, a join keeps an identical image or the common nonconstant form plus
one fresh delta atom over the incoming constant range, and a loop's continuing
kill replaces a loop-carried measure by a fresh header atom.

This is the rule that makes an append inside a loop provable, and it is worth one
worked line. In

```wf-design
for @copy (
  at in 0_u64..count,
  invariant spare: ige(room(acc) + at, count)
) {
  let byte = source[at];
  set acc = seq_push(view: move acc, value: byte);
}
```

`seq_push` declares `room(result) = room(view) - 1`, so `room(acc)`'s image
decreases by one across the statement while the binder's increases by one, the
header target is preserved with no writer premise, and [SEQ-5]'s own requirement
`igt(room(view), Z)` follows from the header target and S11's `at < count` by
[MSR-4]'s two-premise family. The mirror counter the first draft forced on every
append loop is gone.

*Judgment:* none by itself. *Publishes:* the image. *Amends:* [ENT-6]'s image
formation, join and loop-header paragraphs (2970-2996). *Law:* L16.

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
6  the affine-left / L0-right bridge, for every live measure term and
     every live own integer binding having a current image
```

The consumers are exactly: [OP-4] subscript bounds, [SYS-8] system range, [OP-2]
integer domain, [OP-9] allocation fit, [FN-8] requirements, [FN-9] normal-result
relations, [INV-1] invariant targets, and the operation-domain obligations of
[SEQ-0]. **The per-family route lists retire.** [ENT-6]'s `SubscriptBounds`
paragraph, which grants the bridge to subscripts by name, and its `SystemRange`
paragraph, which grants it again, state the family's normalization only.

*Judgment:* the disposition itself. *Publishes:* the disposition of every numeric
goal. *Amends:* [ENT-6] 3034 and 3078 (the two named grants), and [FN-9]'s
`prove_ordering` route, whose undocumented direct-affine branch becomes one of
the six steps rather than a private one. *Note:* this rule is the reason the
design does not have to be revisited when [SEQ] adds an operation: an operation
adds a goal, never a route. *Law:* L16.

**[MSR-5] The contract clause language is terms, not atoms.** A `requires`,
`ensures`, `header_invariant`, `invariant_stmt` or `proof_use` operand is a
**term** of the [ENT-2] term language, not an `atom` of [GRAM-5]. [GRAM-9]'s
flat-computation rule exists to keep runtime evaluation three-address and does
not apply to erased proof syntax, which evaluates nothing. Therefore
`requires ile(len(source), room(out));` is writable directly, and so is
`invariant fill: ile(r.fill, 8_u64);` over a struct field path, and so is
`invariant order: ile(table[i], n);` over a subscript.

Correspondingly `affine_factor` [GRAM-4] admits exactly the [ENT-2] place grammar
and the three measure terms over one, in place of `literal | IDENT | "(" affine_expr ")"`.
One definition of "what may name a quantity" is used by [ENT-2], [ENT-5],
[ENT-6], [FN-8], [FN-9] and [INV-1] alike.

*Judgment:* the ordinary [FN-8]/[FN-9]/[INV-1] admission over the widened operand
set. *Publishes:* nothing new. *Amends:* [GRAM-4]'s `affine_factor` production,
[FN-8]'s clause-expression judgment, [FN-9]'s operand list, and [INV-1]'s atom
sentence (3106); [GRAM-9] is unchanged and gains a stated scope. *Verified
today:* probes `q1`, `q9`, `a11_len_in_requires` are [GRAM-4] and [GRAM-9] at
parse, so all three are amendments and not compiler defects. *Law:* L16.

### 3.2 `[PROV]`: capability values

**[PROV-1] Providers.** A *provider* is a value of one of the compiler-known
opaque nominal types `Heap`, `Arena<'p>`, and `Pool<'p, T, N>`. A provider is
affine [OWN-1], has no writer-visible component, and is the sole authority for
allocating from the store it names: `Heap` names one general-purpose growable
store, `Arena<'p>` names one contiguous extent served by a bump cursor, and
`Pool<'p, T, N>` names `N` interchangeable slots each holding exactly one `T`.
Its measures [MSR-1] are the store's own: for `Arena<'p>`, `cap` is the extent in
bytes, `len` the cursor and `room` the remainder; for `Pool<'p, T, N>`, `cap` is
`N`, `len` the live slot count and `room` the free count. `Heap` has no measures,
because L6 says a general store has none that mean anything.

`Arena<'p>` and `Pool<'p, T, N>` are *confined* types under [CNT-6]; `Heap` is
not, because it is delivered as an `own` entry parameter and lives for the
program.
*Judgment:* provider types are nominal and closed; no source declaration
introduces another. *Publishes:* the store's measures, and the store identity
[RES-6]'s domain algebra tracks. *Law:* L2, L16.

**[PROV-2] Unforgeable and uncopyable.** No source construct produces a `Heap`. A
`Heap` value exists only because the runtime minted exactly one before `main` and
transferred it through the [FN-7] standard-input table; it is affine, so it is
moved rather than copied, and a program holds at most one for its whole
execution. An `Arena<'p>` or `Pool<'p, T, N>` exists only as the result of a
reserving operation [PROV-8]. No operation duplicates, reconstructs, compares,
serializes, or derives a provider from a non-provider value.
*Judgment:* a `construct` [GRAM-8] naming a provider nominal, and every other
source route to one, is a hard error citing PROV-2 at the complete `construct`,
with the restructuring `receive the provider as a parameter, or reserve one with
pool_static or arena_static`. *Publishes:* uniqueness of the `Heap`. *Law:* L2.

**[PROV-3] Every covered-store operation takes its provider, and takes it as a
loan.** An operation that allocates from a store takes that store's provider as a
written `&uniq 'p` parameter and exhibits it. A provider is never passed `own`:
it is confined [PROV-1, CNT-6], and a moved provider is exactly the shape that
strands a live lease with no reachable release target. The one `own` provider in
the language is the `Heap` the entry receives, which is not confined and is moved
once, into `main`.

Every provider operation declares two regions where the store is confined: `'p`,
the store's own confinement region, which appears in the provider's type and in
the type of anything it produces, and `'b`, the region of the loan the call holds.
They are distinct because the caller's borrow is taken in whatever region the call
site opens, which inside a loop body must be one introduced there [OWN-11], while
the store's confinement is fixed at reservation.

```text
| op                  | signature                                                                                    | effects                  |
|---------------------|----------------------------------------------------------------------------------------------|--------------------------|
| box_new             | ['h](heap: &uniq 'h Heap, value: own T) -> own Result<box<T>, OutOfMemory<T>>                  | allocates(heap), writes(heap) |
| box_free            | ['h](heap: &uniq 'h Heap, item: own box<T>) -> own T                                           | writes(heap)             |
| arena_new           | ['p, 'b](arena: &uniq 'b Arena<'p>, value: own T) -> own arena<'p, T>                          | allocates(arena), writes(arena) |
| arena_new_checked   | ['p, 'b](arena: &uniq 'b Arena<'p>, value: own T) -> own Result<arena<'p, T>, NeedCapacity<T>> | allocates(arena), writes(arena) |
| pool_take           | ['p, 'b](pool: &uniq 'b Pool<'p, T, N>, value: own T) -> own slot<'p, T>                       | allocates(pool), writes(pool) |
| pool_take_checked   | ['p, 'b](pool: &uniq 'b Pool<'p, T, N>, value: own T) -> own Result<slot<'p,T>, PoolExhausted<T>> | allocates(pool), writes(pool) |
| pool_release        | ['p, 'b](pool: &uniq 'b Pool<'p, T, N>, item: own slot<'p, T>) -> own T                        | writes(pool)             |
```

The sequence rows that consume a provider are `[SEQ]`'s, not this table's.
`buffer_new` and `buffer_vacant` do not appear, because [CNT-1] retires
`buffer<T>` from the writer surface entirely.
*Judgment:* an allocation call whose provider argument is missing, is not a
provider place, or is not writable is a hard error citing PROV-3 at the `call`.
*Publishes:* the provider place each allocation reaches, and the store's
post-state measures ([SEQ-0]'s declared relations: `len(pool)' = len(pool) + 1` at
a take, `len(pool)' = len(pool) - 1` at a release, `len(arena)' <= len(arena) + K`
at an arena allocation for the fixed constant `K` of [RES-8]). *Amends:* the
`box_new`, `arena_new`, `buffer_new` and `buffer_vacant` rows of [OP-1] (793-798).
*Law:* L2, L3, L4, L16.

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
A function *reaches a store* when its own row carries an `allocates` entry whose
path's **selected type at the leaf** is that store's provider type, or when it
calls a function that does; the leaf's selected type is what [EFF-2] already
computes for every path, so `allocates(env.heap)` on an `Env`-typed formal is a
heap-reaching row and the closure stays exact for aggregates.

A body that allocates only from a provider it reserved itself frames out of its
own signature exactly as any other fresh-local state does, and [PROV-8] makes the
reserved extent an ordinary place of that activation, so nothing invisible is
left behind.
*Judgment:* [EFF-2]'s both-ways row check, unchanged. *Publishes:* the
provider-reachability edge [PROV-6] closes over. *Amends:* [EFF-1]'s `effect`
production (1363-1372); *retires* the effect-row atoms `heap` and `arena`
(META-5: unique fixed lowercase grammar atoms minus 2). *Law:* L2.

**[PROV-5] The entry gains one row.** The `command` standard-input table [FN-7]
gains ordinal 5:

```text
| ordinal | label        | written mode and type | supplied value                                       |
|---------|--------------|-----------------------|------------------------------------------------------|
| 5       | command.heap | own Heap              | the one general store the runtime minted before main  |
```

The row is optional like every other. A `main` that omits it receives no `Heap`,
and by [PROV-2] cannot obtain one.
*Judgment:* the ordinary [FN-7] label, order, mode and type checks. *Publishes:*
the whole-program fact `heap-unreachable` when the row is absent. *Amends:*
[FN-7]'s table (1221-1227), its canonical five-input byte sequence (1239), and
its effect-row sentence (1214), whose `allocates(heap)` becomes `allocates` over
the entry's own labelled provider input. *Law:* L2.

**[PROV-6] Heap-reachability is a closed signature fact.** Because the
compilation unit is closed [PROG-1], there are no function values, and there is
no ambient store [PROV-2], the transitive closure of [PROV-4]'s reaching relation
over the call graph is exact and is computed from signatures alone.
*Judgment:* none by itself; it is the premise of [RES-5]. *Publishes:* the
heap-reaching path, the ordered call chain from `main` to the allocation, which
is the diagnostic [RES-5] prints. *Law:* L2.

**[PROV-7] One release rule for every provider.** A value allocated from a
provider is released to that provider and to no other, and **every such release
exhibits `writes` on the path naming that provider**. This is one rule with no
exception: a `box<T>` drop, a `HeapVector` backing free, a `slot<'p, T>` return,
and a `PoolVector` lease return all carry it, and it is the ordinary [STOR-3]
release row that [EFF-2] already collects. An `arena<'p, T>` value is the one
case that releases nothing at the value, because the whole extent returns when
`'p` ends [STOR-4]; that is a release row of nothing, not an exception.

Two consequences follow from the rule rather than from added clauses. A free is
visible to [EFF-2]'s both-ways check and to [PAR-1]'s written footprint, so two
statements whose only interference is that each frees into one store conflict and
are not overlapped. And the edge carrying a release must reach a live writable
provider place, so a `HeapVector` whose `Heap` was already consumed, and a
`slot<'p, T>` whose pool was moved away, are both refused by the ordinary
reachability premise.

*Judgment:* on every edge carrying a provider-owned release, the provider place
named by the released value's type must be live, reachable and writable; a
release edge on which it is not is a hard error citing PROV-7 at the owning scope
exit, with the restructuring `move the owner out before the provider dies, or
release it explicitly with pool_release or box_free`. *Publishes:* the release
event and the store's post-state measure. *Amends:* [STOR-3]'s release-action
list and [EFF-2]'s "each of these memory-reclamation actions carries the empty
effect row" sentence, which is replaced by the total rule above rather than by an
exception ([META-3]). *Law:* L3, L5, L13.

**[PROV-8] Reserving operations, and where the extent lives.**
`pool_static<'p, T, N>()` and `arena_static<'p, BYTES, ALIGN>()` each reserve one
extent **per activation of the reserving function**, laid out in that
activation's frame, and return the provider confined to `'p`. The extent is an
ordinary frame-resident place [STOR-1] for [OWN-5], for [PAR-1]'s footprint, and
for every rule that reads a place.

The first draft gave the extent static placement and one identity per source
occurrence, and then framed the allocation out of the reserving function's row.
The two together made one extent invisible to [PAR-1] and shared by every
overlapped activation, and gave a reentrant context no extent of its own. Frame
placement removes all three defects at once, at the price of a bigger frame item
in `E`, which is a figure the writer can read and act on. A program that wants
one extent for the whole program reserves it in `main` and passes the provider
down; that is the same shape and it is visible in one signature.
*Judgment:* the ordinary region and confinement judgments [OWN-3, OWN-4, CNT-6],
plus [OWN-5] exclusivity on the extent's place. *Publishes:* the reserved extent's
size and alignment, which enter the reserving context's `stack` item of `E`.
*Amends:* nothing; adds two operation rows. *Law:* L2, L5, L6.

**[PROV-9] A provider can be lent onward.** A helper that receives a provider as
`&uniq 'b P` must be able to hand it to the operation that allocates. Today it
cannot: [OWN-6]'s child reborrow admits only a locally-introduced region whose
block does not extend beyond the enclosing statement, and explicitly excludes a
caller-supplied region parameter, so a reborrow into `'b` is inadmissible and a
reborrow into a fresh local region cannot outlive the statement that binds the
result. Without an amendment, every allocation in the language is writable only in
the function that owns the store, and the capability story ends at `main`.

The amendment is [OWN-6]'s own reasoning applied one position further: a child
reborrow may name a caller-supplied region `'b` that resolved(`h`)'s region
outlives-or-equals **when the receiving call's result type is region-free**,
because then nothing derived from the child appears in the result and the loan
ends with the call, exactly as the rule already argues for the
provenance-candidate position. The parent holder is suspended for the statement
and resumes at its end, unchanged.
*Judgment:* [OWN-6]'s admission, with one more admitted region source under the
stated result-type condition. *Amends:* [OWN-6] 611. *Verified today:* probe
`r1_relend` is `[OWN-6] InvalidChildReborrow`, and its own mechanical fix states
the wall in the compiler's words: "a child reborrow's region admits exactly one
statement, and a value that statement binds dies at the region's end". Probe
`r1_relend_local` shows the escape is available only for a **copy** result, and
`r1_relend_affine` shows an affine result cannot leave the region at all, which is
exactly `seq_reserve`'s shape. *Note:* this also unblocks `docs/patterns.md` P17's
threaded-factory shape, which is the same wall one capability over. *Law:* L2.

### 3.3 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

```text
| class              | members                                                                       |
|--------------------|-------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context, including every      |
|                    | provider extent reserved by [PROV-8]; every worker-lane stack; allocator and   |
|                    | runtime metadata; compiler-derived cleanup scratch; the adapter's persistent   |
|                    | buffers                                                                        |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots; |
|                    | the runtime's fixed handle table; every other runtime-owned store              |
```

Every member of this set presents its state as one of [RES-6]'s domains; a
runtime-owned store that does not is a qualification failure of that runtime
[RUN-2], not a source condition. An extension is written and never implied:
`resource_closed(core + file_handles)` is a different, stronger declaration, and
no such extension is defined in this version.
*Judgment:* fixes the domains [RES-3] quantifies over. *Law:* L1, L5.

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
obligation, so `W = 1` must always be legal and a program that is closed only at
`W = 8` would make the permission load-bearing.
*Judgment:* `E` is well-formed only if every item's arithmetic was performed in
the unbounded mathematical domain and is representable on `T`, the same standard
[STOR-6] already applies. *Publishes:* `E` itself, as a compilation artifact.
*Law:* L1, L6.

**[RES-3] The judgment, in two stages.** For a program `P`,
`source-resource-closed(P)` holds exactly when, on the rewritten call graph
[STK-1], every premise below is established from program text alone:

```text
1  no reachable store is a Heap                                    [PROV-6, RES-5]
2  the call graph is acyclic                                       [STK-2]
3  every covered store's demand is bounded, per domain, by the
     symbolic composition of 3.3.1                                 [RES-6]
4  no execution context reachable from source can be re-entered
     from outside the call graph                                   [STK-4]
```

Those four are deterministic functions of program text and compiler version. They
are what the `resource_closed` marker makes a hard error [RES-4].

For a selected target `T` and its runtime, `E-materializes(P, T)` holds when every
symbolic figure of stage one has a concrete value on `T` (frame sizes measured
after code generation [STK-3], strides and alignments [STOR-6], the runtime's own
profile rows [RUN-3]) and every row of the resulting table is representable and
is one the runtime's published profile can carry [RUN-2]. Failure here is a
**target-qualification failure** under [STOR-6] and [QUAL-2]: it stops
compilation, cites no language rule, and is not a source rejection. A program is
*resource-closed on `T`* when both hold.
*Judgment:* stage one, per domain, over the checked program; deterministic,
terminating, and free of search, budget or timeout. *Publishes:* the property,
and `E`. *Law:* L1, L8, L9.

**[RES-4] The entry requirement.** The entry may carry the marker
`resource_closed` before its `command` program-kind marker:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
```

The marker changes no acceptance judgment: every program is judged by exactly the
same rules. It changes two things, and the first draft's "the marker changes no
other rule" was false about both. It makes the failure of [RES-3] stage one a
hard error citing RES-4 rather than a reported property. And it selects which
[SCOPE-3] deferrals apply to the program: for a marked program stack exhaustion
and covered-store exhaustion are inside the model [STK-5], and for every other
program they stay deferred.
*Judgment:* on a marked entry, the first unestablished premise of [RES-3] stage
one is a hard error naming its own cause: the heap-reaching path [RES-5], the
call-graph cycle [STK-2], the unbounded store [RES-6], or the reentrant context
[STK-4]. *Amends:* [FN-7], which fixes main's marker set, and [GRAM-2]'s
`program_kind` production. *Law:* L1.

**[RES-5] The heap excludes resource-closedness.** A program whose call graph
reaches a `Heap` [PROV-6] is not resource-closed, and a `main` selecting
`command.heap` is by itself the rejection. A bounded general store is still a
general store: an envelope item can promise bytes, and cannot promise that the
next contiguous aligned request has a home.
*Judgment:* under [RES-4], a hard error citing RES-5 at the offending
`input_label` or at the deepest `call` of the heap-reaching path, rendering the
whole chain. *Law:* L6.

**[RES-6] Store domains and their algebras.** Every covered store presents its
state through [MSR-1]'s measures, and exactly three domains are defined. Nothing
else is admitted, and a store outside this list contributes no envelope item and
denies [RES-3].

```text
| domain                        | state         | acquire                      | release          | serviceable when |
|-------------------------------|---------------|------------------------------|------------------|------------------|
| uniform slots                 | len, cap = N  | len + 1                      | len - 1          | room >= 1        |
|  (Pool; lane, task, queue,    |               |                              |  [PROV-7]        |                  |
|   completion and handle       |               |                              |                  |                  |
|   records of the runtime)     |               |                              |                  |                  |
| bump extent (Arena<'p>)       | len, cap      | len advances by              | nothing; the     | room >= that     |
|                               |  in bytes     | round_up(len, align(T))      | whole extent     | advance          |
|                               |               | - len + size(T)              | returns with 'p  |                  |
| static and frame placement    | fixed offsets | none at run time             | none             | decided at       |
|                               |               |                              |                  | compile time     |
| general heap (Heap)           | -             | -                            | -                | undecidable      |
|                               |               |                              |                  | from E [RES-5]   |
```

The runtime's own tables are uniform-slot stores of this list, with their `cap`
published by the profile row [RUN-3] and their `len` composed from the program by
exactly the algebra of 3.3.1. That is what makes L5 true of the runtime's tables
and not only of its stacks.
*Judgment:* the composition of 3.3.1 per domain. *Publishes:* per program point,
per domain, the store's `len` bound. *Law:* L6, L16.

**[RES-7] Typed failure, and what it retires.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume: `OutOfMemory<T>`, `PoolExhausted<T>`
and `NeedCapacity<T>`, each with one payload field `rejected` carrying the
unconsumed input. No covered-resource failure is a trap, an abort, a process
exit, a retry, or a promotion to a larger store, in the writer's code or in the
runtime.
*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result`. *Publishes:* on
the `Err` edge, the returned owner's identity, and the store's own refusal
relation `ieq(room(store), Z)` (L8). *Retires:* the heap arm of the exhaustion
floor; the `wf_resource_abort` site for allocation refusal (batch 0079 item F4,
`docs/done/0079-exhaustion-floor.md`) has no reachable caller once allocation
returns a value. *Amends:* [SCOPE-3] 29, whose "heap exhaustion ... may stop
execution at the host boundary without a Whitefoot value" ceases to be true.
*Law:* L3, L8.

**[RES-8] A covered acquisition is a partial operation.** Each covered-store
acquisition comes in exactly two spellings, on the model of `+` and `+checked`:

```text
pool_take(pool: p, value: v)          requires igt(room(p), Z)        -> own slot<'p, T>
pool_take_checked(pool: p, value: v)  total                           -> own Result<slot<'p,T>, PoolExhausted<T>>
arena_new(arena: a, value: v)         requires ige(room(a), K<T>)     -> own arena<'p, T>
arena_new_checked(arena: a, value: v) total                           -> own Result<arena<'p,T>, NeedCapacity<T>>
```

`K<T>` is the compile-time constant `align_ceiling(T) - 1 + size_ceiling(T)`,
computed by [OP-9]'s existing ceiling arithmetic; it is target-independent, so
the requirement is a two-term difference bound over `room(a)` and a constant. The
proved form is admitted only when [MSR-4]'s disposition discharges its goal; an
unproved goal is a static rejection with no fallback, exactly as an unproved
subscript is [OP-4]. **The `Heap` has no proved form**: no honest domain predicate
exists for a general store (L6), so every heap acquisition is total and returns
`Result` unconditionally.
*Judgment:* [MSR-4] discharge at the proved spelling; nothing at the checked one.
*Publishes:* the post-state measure relation ([SEQ-0]). *Law:* L3, L6, L16.

**[RES-9] What bare resource-closedness does not cover.** Disk space, the
successful acquisition of a file, socket or other host object not exclusively
reserved before start, network reachability and throughput, CPU time, deadlines,
scheduler fairness, power, device health, host termination, and OS quota
revocation are outside [RES-1] and outside every judgment in this file. They
remain typed system outcomes where the operation defines one ([SYS-7]'s error
classes), and environment conditions where it does not ([RUN-6]).
*Judgment:* none; a boundary statement. *Law:* L1.

**[RES-10] The per-function summary is part of the callable boundary.** Each
function's per-domain demand summary [3.3.1] is one finite derived component of
its [FN-1] boundary, beside its requirement templates, relation templates and
target summary. It is already computed bottom-up on the call-graph DAG; stating
that it is published makes `E` composable the day separate compilation exists and
costs nothing today.
*Judgment:* none; a boundary statement. *Publishes:* the summary. *Amends:*
[FN-1]'s boundary list. *Law:* L1, L5.

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

A statement's summary is **one map from exit label to `(peak, delta)`**, not one
pair. The exit labels of a statement are its fallthrough, each variant of a
result it produces, each `break` label it may take, and `propagate`. Making the
summary a map is what the first draft's branch rule needed and its sequence rule
could not consume; with the map, per-variant retention is the domain of the map
rather than an exception to a rule.

Per resource kind `r`, the primitive transfers are fixed:

```text
acquire one       (peak 1, delta +1)     on the success exit; (0, 0) on a refusal exit
release one       (peak 0, delta -1)
move an owner     (peak 0, delta  0)     moving into a container acquires nothing
borrow an owner   (peak 0, delta  0)
```

and the compositions are:

```text
sequence   for each exit label L of B:
             peak(A;B)[L]  = max( peak(A)[fallthrough], delta(A)[fallthrough] + peak(B)[L] )
             delta(A;B)[L] = delta(A)[fallthrough] + delta(B)[L]
           for each non-fallthrough exit label L of A: A;B carries A's own (peak, delta)[L]

branch     the union of the arms' maps, keyed by exit label; two arms reaching one
           label contribute the componentwise max of peak and, when their deltas
           differ, the interval [min, max] of delta

call       substitute the callee's map at the call site, with its formal measure
           and provider terms replaced by the actual ones

loop       for the backedge label: delta = 0  -> peak is one iteration's peak, and no
             iteration bound is needed
           delta > 0 -> a counted range, a structural capacity cutoff (len <= cap), or
             a writer-supplied resource invariant is required; otherwise no finite E
           each exit label of the loop carries the map of the edge that reaches it

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration peak p
                                        and retained d, and K the profile's window
```

`K` is the runtime profile's window, not the lane count `W`, because an iteration
that submits a `may-suspend` operation and suspends leaves its resources
outstanding while its lane starts another. `K` is target-stage data: a `par`
composition therefore produces a symbolic figure in stage one and a number in
stage two, which is exactly the two-stage split L1 requires.

**What needs no writer annotation:** straight-line acquire, move, borrow and
release; lexical scopes and compiler-derived cleanup edges; branch joins;
per-variant retention a `Result` or `Option` already distinguishes;
`FixedVector`'s `len <= cap` and its initialized prefix; moving an owner into or
out of a container; a loop whose backedge restores the state; a counted loop whose
per-iteration delta is a fixed affine expression; a non-recursive call with a
computed map; and a `par` loop composed by the formula above.

**What needs one:** a loop that may retain with no structural cutoff; a relation
across two containers (`len(active) + len(waiting) <= cap(pool)`); a resource
returned only at a later milestone; an acquisition whose size is a computed value;
a `par` window the profile does not fix; and any place where the writer wants a
tighter answer than the per-branch maximum. These are written as ordinary [INV-1]
invariants over the measure terms of [MSR-1], which are affine atoms by [MSR-5],
and the checker verifies base, preservation and exit exactly as for any other
invariant. The checker never searches for an invariant: it does not enumerate
paths, guess loop invariants, choose allocator placements, or divide a store
between claimants.

#### 3.3.2 Which stage decides what

```text
 1  tail-SCC rewrite [STK-1]                        source stage   compiler
 2  call-graph acyclicity [STK-2]                   source stage   compiler
 3  provider reachability, heap-freedom [PROV-6]    source stage   compiler
 4  per-function per-domain summary maps            source stage   compiler
 5  loop and branch composition (3.3.1)             source stage   compiler
 6  reentrancy check [STK-4]                        source stage   compiler
 7  concrete sizes, strides, static image           target stage   compiler
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

Steps 1 to 6 decide whether the program is source-resource-closed, and are the
only steps a source rejection may cite. Steps 7 to 11 decide whether this build
qualifies. Steps 12 to 16 decide whether this run is admitted.

### 3.4 `[STK]`: the stack

**[STK-1] A tail edge is one whose caller frame is dead.** For each strongly
connected component of the call graph in which every intra-component call edge is
a tail edge, the compiler rewrites the component into one dispatcher loop before
frames are measured. **An intra-component edge is a tail edge exactly when, at
that edge, the caller's activation record is dead**: no loan, borrow, view,
region or reborrow the caller introduced is live; no compiler-derived drop or
release remains to run after the call; no `par` join is outstanding; and no place
the caller's frame holds is reachable from any argument of the call or from any
value live across it.

That one premise replaces the first draft's five syntactic conditions, and it is
the only formulation that sees the case those conditions missed: a member that
borrows its own local and passes the borrow forward keeps its frame live across
the jump, transitively along the whole chain, so the component's live set is the
sum along the chain rather than the maximum over its members. Being written as
the complete `expr` of a `return_stmt` is a consequence of the premise, not a
condition beside it.

A component in which some edge is not a tail edge is **not rewritten**, and is
then refused by [STK-2]. It is never rewritten with a smaller frame.
*Judgment:* per edge, from the ownership and loan state the checker already has;
no proof search. *Publishes:* an acyclic call graph, or a component that is still
cyclic. *Amends:* nothing in [FN-6], which continues to permit recursion; this is
a lowering, not an admission rule. *Verified today:* probes `f2b` and
`f8_tailframe` are mutual tail recursions carrying a live borrow of a caller
local and are accepted, so the premise refuses a shape the syntactic list
admitted. *Law:* L7.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After
[STK-1], a program whose call graph still contains a cycle has no finite stack
envelope and is not resource-closed. A `requires` bound on a recursion parameter,
a proof that a recursion argument decreases, and every other depth certificate
are **not** admitted as a substitute.
*Judgment:* under [RES-4], a hard error citing STK-2 that renders the complete
cycle in call order and the restructuring `rewrite the recursion as a loop over
an explicit FixedVector work list, or make every recursive call a tail call whose
caller frame is dead at the jump`. *Law:* L7.

**[STK-3] The frame envelope, over the whole chain.** For each execution context,
the `stack` item of `E` is measured over the context's **whole chain**, from the
point at which the environment hands that context a stack to the point at which it
takes it back: process entry through `ProgramFinished` for the entry context, lane
creation through lane teardown for a worker. `main`'s own chain is one segment of
it, and the runtime's start-up trampoline, its teardown, its drop glue, and the
exhaustion floor's own frames are other segments. Within one segment,

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

over the acyclic graph of [STK-2], where the possibly-active callees include those
reached on error and propagation edges, compiler-derived drop glue, the target
adapter's helpers, the `par` worker entry and resume paths, and the ABI save area.
`frame(f)` is measured **after final code generation**, which is why this is a
target-stage figure and not a source one: it is made of things that do not exist
earlier, the ABI frame record a non-leaf keeps and the callee-saved registers the
allocator chose to spill.

An optimization that would raise a computed envelope is handled the only coherent
way: `E` is an **output** of code generation and never an input to it, so the
compiler recomputes `E` after every optimization it performs and publishes the
figure the emitted code actually needs. A published figure a deployment has
already sized against is a fact about that build; the next build publishes its
own.
*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic
discipline. *Publishes:* one `stack(context, bytes)` item per context per profile
row. *Amends:* [STOR-6] 757-761, whose "the language therefore defines no numeric
per-array, per-object, or per-function frame ceiling" keeps its scope for the
*language* and is joined, for a resource-closed build, by a computed per-context
envelope. *Law:* L5, L6.

**[STK-4] One item per execution context, and reentrancy is refused.** `E` carries
a `stack` item for the entry context and for each of the `W - 1` worker lanes,
plus one for every context the target profile introduces: a completion helper, a
bounded blocking helper, a guard-page or other signal stack, an FFI callback
stack. Each is measured by [STK-3] over its own whole chain.

A context that can re-enter the Whitefoot call graph from outside it, meaning a
signal handler, an interrupt handler, an FFI callback or a host reentrancy path,
**denies
source-resource-closedness in this version**. The refusal is a source judgment,
because whether source can be re-entered is a property of the program's own
declarations: a program that declares no such entry point has none. The successor,
which admits reentrancy with a separately reserved stack item per reentrant
context, is open question Q9.
*Judgment:* a source check over the program's declared entry points; under
[RES-4], a hard error citing STK-4. *Law:* L5.

**[STK-5] Stack exhaustion moves inside the model, for these programs only.** For
a program that is resource-closed on its target, stack exhaustion is not a
deferred external resource condition: [STK-2] and [STK-3] make the maximum chain a
computed item of `E`, and under an admitted run [RUN-6] it is unreachable. For
every other program, [SCOPE-3]'s deferral stands unchanged, and so does the
guard-page floor that reports it, whose own alternate stack is, for a
resource-closed build, an item of `E` under [STK-4].
*Judgment:* none; a scope statement. *Amends:* [SCOPE-3] 29-31. *Law:* L1.

### 3.5 `[RUN]`: runtime closure and admission

**[RUN-1] The artifact.** For every judgment in this file the artifact is the
writer's code, the compiler-derived cleanup and drop glue, the monomorphized
instances, the `par` runtime, the allocator and its metadata, and the qualified
target adapter: everything the process runs between process entry and
`ProgramFinished`. *Law:* L5.

**[RUN-2] Runtime closure, stated as one obligation.** A runtime qualified for
resource-closed programs performs, after the `SourceStart` barrier and until
`ProgramFinished`, no covered acquisition whatsoever: no allocator call for
runtime-owned storage, no thread or helper creation, no stack, queue, table or
worklist growth, no lazy TLS or adapter initialization, no first-use mapping, and
no first-error formatting buffer. Every runtime record is established before the
barrier or is carved from a fixed store that is already an item of `E`.

**Saturation is not answered; it is made unreachable.** Every runtime-owned store
is a uniform-slot domain [RES-6] whose `cap` the profile publishes and whose peak
`len` the program's own composition computes; `E-materializes` fails when the
program's peak exceeds the row. A qualified runtime therefore never meets a full
queue, a full task table, or an unavailable lane while running a program that
qualified. The first draft instead listed four permitted answers, waiting, reuse,
inline execution and a sequential path, and constrained none of them: inline
execution nests a task's chain inside a lane's current activation, which no term
of [STK-3] counts, and waiting on a saturated queue in a configuration where no
other frame is ready is a hang, which is not a logic error of the program. Neither
is admitted here. A runtime that cannot publish a bounded capacity for one of its
stores does not support the marker on that target.
*Judgment:* a target-qualification obligation, auditable from the emitted code and
the runtime's own translation units; its failure is a [QUAL-2] qualification
failure, not a source rejection, and no source construct can weaken or waive it.
*Publishes:* the runtime's own items and capacities. *Amends:* [SYS-2] 2264's "no
system operation allocates", which is kept and given its companion: an adapter
record, a host-string lease's backing, and a path buffer are runtime-owned stores
of [RES-1] with published capacities, or the operations that need them are
excluded from resource-closed programs by name. *Law:* L3, L5.

**[RUN-3] `par` enters `E` as a profile, not as an iteration count.** For each
supported lane count `W`, the runtime publishes one finite profile row: `W` lanes
(of which `W - 1` are host worker threads), `W - 1` worker stacks, a task-record
capacity `K(W, d)` where `d` is the program's maximum nested `par` depth, fixed
queue capacities, a fixed completion-record capacity, and the handle-table
capacity. The number of iterations of a `par`-permitted loop never appears in `E`:
the runtime chunks the index range lazily, so a loop of a billion iterations holds
no more task records than one of a thousand.
*Judgment:* a fixed-arithmetic composition (3.3.1's `par` rule) against each
profile row; the compiler emits no per-`W` clone of the program. *Publishes:* the
`lanes` and `slots` items of each row. *Amends:* the sentence common to [PAR-1]
1989, [PAR-2] 2024 and [PAR-3] 2049, "exhaustion of the execution resources an
implementation spends on overlapping is a resource condition under [SCOPE-3] and
is not an observable of this rule": for a program resource-closed on this target
that exhaustion is not merely unobservable, it is unreachable. *Law:* L5, L9.

**[RUN-4] The parallel footprint of an allocation is its provider place.** In
[PAR-1]'s written-footprint clause, "the caller region each `allocates(arena 'r)`
entry names after region substitution" is replaced by "the places each `allocates`
path reaches under the [EFF-2] call-boundary projection", the same projection the
rule already applies to `reads` and `writes`. Two statements that allocate from
one provider therefore conflict, and two that allocate from distinct providers do
not. With [PROV-7] the same is now true of two statements that only *free*, which
was the first draft's soundness hole.
*Judgment:* the existing [PAR-1] overlap judgment, with one fewer special case.
*Amends:* [PAR-1] 1969, and [PAR-2]/[PAR-3] through their "forms every footprint
exactly as [PAR-1] forms one" clauses. *Law:* L2, L5.

**[RUN-5] The startup protocol.** Program start has four points, and the covered
guarantee spans the last three:

```text
PreStart
    select a row of E from the target's profile table, largest supported W first
    materialize every item of that row:
        commit each region (committed backing, not a reserved address range)
        commit each stack, including every worker lane's
        create W-1 lanes and park them at the ready barrier
        establish every queue, task, completion and wait record
        initialize every adapter record, TLS block and runtime table
    a step that fails -> select the next smaller row and start over; when no row
        remains, report StartFailed(item); nothing below happens

SourceStart  (the barrier)
    every item of the selected row is established; no covered acquisition remains
    the runtime enters its closed mode [RUN-2]

Running
    main executes; source and runtime draw only on the selected row

ProgramFinished
    main returns an ExitStatus, or the program is one that does not return [FN-1]
    every compiler-derived release on the return edge has run
    every outstanding par task and completion record has drained
    the runtime's bounded teardown is complete
```

Descending the table is not a retry of a failed acquisition and does not violate
L3: it is the selection step of step 12 being made with better information, and
[PAR-1] 1988 already guarantees `W = 1` is legal for every program. A `PreStart`
failure at `W = 1` is reported as `StartFailed(item)` on an
implementation-defined channel using fixed, preallocated storage; no source
statement executes, no owner comes into existence, no language cleanup runs, and
no `ExitStatus` is produced. It is not a source `Result`, not `main`'s return
value, not a language trap, and not a source-language rejection [DIAG-1].
*Judgment:* a target obligation, not a source judgment. *Amends:* [PROG-3]
1499-1509, whose start-time obligation gains the materialization of `E` and whose
`ProgramFinished` boundary is now named. *Law:* L1, L5.

**[RUN-6] Admission, and the theorem.** `Admitted(H, row)` holds when an
environment `H` has actually established a grant implementing every item of the
selected row before the barrier (committed backing rather than a reserved address
range, real lanes at their ready barrier, real queues and records) and, for the
duration of the run, does not revoke it and permits no unmodelled competitor to
consume from it. Then:

```text
source-resource-closed(P)  and  E-materializes(P, T)  and  Admitted(H, row)
--------------------------------------------------------------------------
no covered-resource exhaustion in run(H, T(P))
```

An environment that later revokes the grant, kills the process, or violates the
target profile has falsified `Admitted`; it has not falsified the program's
property, and no rule of this file is stated in terms of it.
*Judgment:* none by the compiler. *Publishes:* the deployment contract, which is
the selected row. *Law:* L1.

### 3.6 `[CNT]`: owners, typestate, and confinement

**[CNT-1] The owner inventory.** Exactly five sequence owners, each with a static
backing fixed by its type. Four are prefix owners (L12); the fifth is the rotation
a queue needs and a prefix cannot express.

```text
| type                 | shape  | backing              | provider  | cap        | growth      |
|----------------------|--------|----------------------|-----------|------------|-------------|
| FixedVector<T, N>    | prefix | inline, N slots      | none      | N          | never       |
| HeapVector<T>        | prefix | one heap allocation, | Heap      | runtime    | seq_reserve |
|                      |        | none while empty     |           |            |             |
| ArenaVector<'r, T>   | prefix | one arena block      | the arena | runtime    | seq_reserve |
|                      |        | in 'r                | of 'r     |            | in 'r       |
| PoolVector<'p, T, N> | prefix | one pool lease of a  | the pool  | N, from    | never       |
|                      |        | FixedVector<T, N>    | of 'p     | the slot   |             |
|                      |        | slot                 |           | type       |             |
| FixedRing<T, N>      | ring   | inline, N slots      | none      | N          | never       |
```

A prefix owner's initialized storage is exactly `[0, len)`. A ring owner carries
one further piece of typestate, a head offset, and its initialized storage is
`[head, head + len)` taken modulo `N`; that is still one scalar relation and still
no per-slot state, so L12 holds of it unchanged. A ring's element access is by
logical index `0 <= i < len`, and a ring yields **no view**, because its
initialized region is not contiguous. That refusal is the whole cost of having a
FIFO, and it is a smaller cost than the hand-rolled ring the first draft's own
kernel program was forced to write from an `array` plus two `u64` fields whose
correspondence nothing checked.

A container type is a compiler-owned nominal: it has no writer-visible field, is
constructed only by the `[SEQ]` operations, and has no source construction form
[GRAM-8]. An ordinary struct whose invariants are reproved at every use is
refused, because `len <= cap` would then be a fact with support the writer can
kill, and [ENT-5] would delete it at the first unrelated `set`.
*Amends:* [TYPE-2], five added composite types. *Law:* L4, L12.

**[CNT-2] Container state is typestate, and its measures are [MSR-1]'s.** Each
owner carries `len` and, where it is not a constant, `cap`; a ring carries a head
the writer cannot name. `len(v)`, `cap(v)` and `room(v)` are the measure terms of
[MSR-1], with the implicit facts and the identity [MSR-2] states, and with the
readers [SEQ] provides. There is no second definition of a length here, which was
the first draft's error: it defined `room` as denoting `cap - len` and then
declared that the relation between them was not a fact.
*Amends:* nothing beyond [MSR-1] and [MSR-2]. *Law:* L11, L15, L16.

**[CNT-3] Raw slots are unreachable.** No `[SEQ]` operation, no subscript, and no
borrow yields a place outside a container's initialized region. A subscript on an
owner or view carries the ordinary [OP-4] obligation `ilt(index, len(base))`,
against `len` and never against `cap`. There is no uninitialized read to reject,
because there is no spelling that reaches one (L12). *Law:* L12.

**[CNT-4] Affine elements.** `T` may be affine in every owner. The initialized
region is what makes this sound: an element enters and leaves only through an
operation that moves the boundary or exchanges two initialized positions, so no
slot is read before it is written or after it is taken. `FixedVector<Handle, 64>`
is an object table with [SEQ-11]'s middle removal and [SEQ-13]'s exchange, which
is what makes the claim true rather than advertised.
*Amends:* [TYPE-2]'s `array` restriction only by not inheriting it; `array<T, N>`
keeps its copy-only element domain, because `array` carries no length separate
from `N`, so every slot is live at once and there is no boundary to make an
affine element's entry and exit unambiguous. *Verified today:*
`array_new<box<u64>, 4>` is [OP-1] `InvalidOperation` (probe `p9`), so this is
new capability. *Law:* L12.

**[CNT-5] Release.** The release action of every owner, under [STOR-3]:

```text
drop each initialized element      ascending logical index, each element's own
                                     compiler-derived drop
release backing:
  FixedVector, FixedRing    nothing (inline in its owner)
  HeapVector                one compiler-derived heap free, exhibiting writes on the
                              Heap path [PROV-7]
  ArenaVector               nothing at the value; the block goes with 'r  [STOR-4]
  PoolVector                one lease return, exhibiting writes on the pool path
```

*Amends:* [STOR-3]'s `buffer<T>` drop sentence by superseding it. *Law:* L13.

**[CNT-6] Confined types and provenance-bearing types.** [STOR-5]'s prohibition is
restated intensionally, because its enumerated position list did not mention the
container element positions that did not exist when it was written, and because
it conflated two properties that need opposite treatments.

A type is **provenance-bearing** when it carries an [OWN-5] origin set: exactly
the three views [VIEW-1]. A provenance-bearing type may occupy **no** position
from which a value could outlive or hide its origin set: no struct field, no enum
payload, no element of any sequence owner, no `box`, `arena` or `slot` content, no
generic type argument, and no result outside [VIEW-10]'s ceiling. That is
[STOR-5]'s existing sentence with its enumeration replaced by its reason, and it
closes the container element positions the first draft left open.

A type is **confined** when its complete type after substitution names a region
that is not an origin set: `arena<'r, T>`, `slot<'p, T>`, `Arena<'p>`,
`Pool<'p, T, N>`, `ArenaVector<'r, T>`, `PoolVector<'p, T, N>`, and every generic
instance one of them is an argument of. A confined value may occupy any position
whose owning value's own complete type names the same region, so that the
position is itself confined and [STOR-4] governs it. Two consequences follow, and
both are things the first draft could not express:

- `Result<slot<'p, T>, PoolExhausted<T>>` is a legal type. The instance's complete
  type names `'p`, so the instance is confined and [STOR-4] governs it; it may not
  be stored or returned outside `'p`, which is exactly the protection [STOR-5]
  wanted. Without this the checked spelling of every acquisition is untypeable and
  the design collapses into the strict-proof form owner ruling R12 rejects.
- A source `struct` field may still not hold a confined type, because a source
  nominal declares no region parameter and so its instance type does not name the
  region. Region-parametric nominals are open question Q8, and they are what a
  kernel slab allocator needs.

*Judgment:* a provenance-bearing type in a prohibited position is a hard error
citing CNT-6 at the complete contained `type`, with the restructuring `keep the
view as a direct local, parameter, or result; do not store it inside another
value`. A confined type in a position whose owner does not name its region is a
hard error citing CNT-6 at the same node. *Amends:* [STOR-5]'s position list and
its region-bearing relation, and [FN-2]'s blanket rejection of a region-bearing
generic argument, which narrows to provenance-bearing arguments. *Verified today:*
probe `f7_regionresult` is `[FN-2] RegionBearingGenericArgument`, so this is the
amendment that makes 3.9's own example a program. *Law:* L10, L13.

**[CNT-7] Confinement is an outlives judgment.** A confined value may be moved,
returned, or bound to a destination whose region its own region outlives-or-equals
[OWN-3], and to no other. [STOR-4]'s "may not be returned" becomes the ordinary
[OWN-4] relation the language already uses for borrows: a helper may return
`own PoolVector<'p, Record, 256>` when `'p` is one of its caller-supplied region
parameters, and may not when `'p` is one of its own local regions. Without this a
pool-backed sequence cannot leave the function that leased it, and the pool seam
of 3.9 buys a container no helper can produce.
*Amends:* [STOR-4] 716. *Law:* L13.

**[CNT-8] Acquiring capacity is owner-level and provider-bearing.** Every
operation that may change `cap(v)` takes the owner **by value**, takes the
provider, names its allocation effect, and returns `Result`, handing the untouched
owner back inside the error. There is no capacity-changing operation on a borrow
and none on a view, and there is none that keeps a larger backing on failure: a
fallback is what L3 forbids.
*Law:* L3, L4.

**[CNT-9] A container type never appears behind `&uniq`.** A `param`, `rtype`, or
`let`-bound holder whose mode is `&uniq 'r` and whose direct type is a container
type is a hard error citing CNT-9 at the complete `param` (or `rtype`), with the
restructuring `pass a MutSpan or AppendView for element and append work, or take
the owner by value and return it`. A shared `&'r` container parameter remains
legal: it can observe measures and read elements and can change nothing.

This is the rule that retires D1's shape. *Retires:* the writer-facing
`&uniq buffer<T>` and `&uniq Container` state-borrow forms. `&uniq` survives
everywhere its referent's measures are type facts rather than state: a `&uniq` to
a struct holding `array<T, N>` fields, to a `slot<'p, T>`, or to a `MutSpan` is
legal, because no operation on any of them can change a length. *Law:* L11.

**[CNT-10] `array<T, N>` is retained unchanged**, as the `len = cap = N` case. A
program that needs no length carries no length, and `tests/programs/fir_filter.wf`
is untouched by this design. *Law:* L11, L12.

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
does today, and each is provenance-bearing under [CNT-6]. `Span<'r, T>` **is**
today's `slice<'r, T>` renamed; the rename is the whole of the change to it. Its
measures are `len` = the viewed element count, `cap` = `len` for `Span` and
`MutSpan`, and for an `AppendView` `cap` = the window size fixed at formation and
`len` = what this view appended.
*Amends:* [TYPE-2] (two added view types, one renamed), [STOR-5] through [CNT-6],
[OWN-1] (all three are affine). *Law:* L10.

**[VIEW-2] Formation, and the loan the view value holds.** A view is formed from a
borrow of the owner:

```text
seq_span<'r>(vector: &'r v)          -> own Span<'r, T>
seq_mut_span<'r>(vector: &uniq 'r v) -> own MutSpan<'r, T>
seq_append_view<'r>(vector: &uniq 'r v) -> own AppendView<'r, T>
```

and **the view value, not the argument borrow, holds the loan**. For its whole
life, a view value holds a loan of its own strength, shared for `Span` and
exclusive for `MutSpan` and `AppendView`, on every place in its [VIEW-3] origin
set. The
loan begins at formation and ends when the view value is consumed or released.

The first draft said "the freeze is the existing loan" and that was false twice.
The argument borrow is a call-scoped temporary ([OWN-6], and probe `f2b` shows two
of them on one place in one region are accepted), so it does not survive the
formation statement. And [OWN-5]'s origin clause judges a slice access as a
*shared* access through every origin, so under the imported clause nothing
prevented a second `AppendView` on one owner, two `absorb`s publishing a summed
length, and a discharged [OP-4] on raw slots.

Two clauses of [OWN-5] therefore replace its one: an access through a
shared-strength view value is judged as one shared access through every origin,
and an access through an exclusive-strength view value is judged as one exclusive
access through every origin. Exclusivity then refuses the second `AppendView` at
its formation, by the rule that already refuses two overlapping exclusive loans.

Ending the loan at the consume rather than at the end of `'r` is the other half,
and it is what makes the surface usable: `absorb` returns the owner to the writer
at the statement that consumes the view, so append-commit-read needs no nested
region per phase.

Formation publishes:

```text
seq_span         len(s) = len(v),  cap(s) = len(v)
seq_mut_span     len(m) = len(v),  cap(m) = len(v)
seq_append_view  len(a) = Z,       cap(a) = room(v)
```

Each is a two-term relation; `room(a) = room(v)` follows from [MSR-2]'s identity
and is not separately published. The first draft's `cap(a) + len(v) = cap(v)` is a
three-term relation that L0 cannot hold and is gone.
*Amends:* [OWN-5]'s slice-origin access clause. *Law:* L10, L14, L15.

**[VIEW-3] View provenance is slice provenance.** Every view value carries the
finite origin set [OWN-5] defines for slices, formed and preserved by the same
sentences: formation makes a singleton, including when the formed-from place ends
in a subscript, so `seq_append_view<'w>(vector: &uniq 'w table[i])` has the
singleton origin `table[i]` and [OWN-7]'s conservative subscript overlap does the
rest; and binding, moving, passing, and returning preserve the set. An access
through a view is judged as one access of that view's strength through every
origin [VIEW-2].
*Amends:* [OWN-5] by generalizing "`slice<'r, T>` value" to "view value".
*Law:* L10.

**[VIEW-4] A view descriptor's length cannot be changed through a borrow.** No
operation takes a `MutSpan` or a `&uniq` to one and produces a different length,
and none changes its owner's length. The ground is stated once, as a property of
view descriptors rather than per type: a view is affine, so [STOR-1] refuses a
`set` of it, and it is provenance-bearing, so [SET-2] refuses a `replace` of it.
Therefore no callee can perform D1's whole-referent exchange on a view descriptor
it received behind `&uniq`, and `MutSpan<'r,T>`, `&uniq 'b MutSpan<'r,T>` and
`&uniq 'b AppendView<'r,T>` are all length-fixed for [CALL-3]. This dependency is
load-bearing and is recorded because the day a view type becomes region-free,
[CALL-3] becomes D1. *Law:* L11.

**[VIEW-5] `AppendView` is a spare window.** Its `base` is the owner's length at
formation and is not a source-visible value. `len(a)` counts what this view
appended and `cap(a)` is the window. Every `[SEQ]` operation on an `AppendView`
acts on `[base + i]` for `0 <= i < len(a)`; `seq_truncate` on an `AppendView` may
reduce `len(a)` to zero and no further. A callee that receives an `AppendView`
therefore cannot reduce its caller's `len(v)`, which is why [CALL-3] can leave the
caller's length facts alive. *Law:* L14.

**[VIEW-6] `absorb` is the commit event.**

```wf-design
let written = absorb(view: move a);
```

`absorb` consumes the `AppendView`, ends its loan, and returns `own u64`. Its
judgment, in this order:

1. the operand's origin set is resolved to one owner place `P` ([VIEW-7]);
2. the result value is bound to the commit value `w`, with `w = len(a)`
   established at it;
3. every fact whose support contains `P`'s root dies, as a whole-place event on
   `P` ([ENT-5] clause (b), the projected-write clause, which is the clause a call
   uses);
4. `written = w` is established, and `P`'s measure images are transferred:
   `len(P)` becomes `image(len(P)) + image(w)` and `room(P)` becomes
   `image(room(P)) - image(w)` [MSR-3].

Step 4 is an affine image transfer, not an L0 three-term relation, which is why it
is exact rather than "carried only when the old length was a constant". That
restriction, and the open question attached to it, are gone.
*Law:* L10, L14, L16.

**[VIEW-7] `absorb` is admitted only in the formation function.** The operand's
origin set must be a singleton resolved place of the current function. An
`AppendView` reaching a function as a parameter has a formal-view origin [OWN-5],
not a resolved place, so a callee cannot commit its caller's length behind the
caller's back. A violation is a hard error citing VIEW-7 at the operand `atom`,
with the restructuring `return the view to the function that formed it and absorb
it there`. *Law:* L14.

**[VIEW-8] An abandoned `AppendView` drops what it appended.** Its
compiler-derived release action under [STOR-3] drops the elements of
`[base, base + len(a))` in ascending order, then nothing. The owner's `len` is
unchanged, so the abandoned elements are neither leaked nor double-dropped, and no
fact about `len(P)` was ever published. Not absorbing is therefore a well-defined,
safe program that discards work, which is what makes `absorb` an ordinary
operation rather than a must-use obligation. *Law:* L13, L14.

**[VIEW-9] Views are never stored** [CNT-6], and never returned except under
[VIEW-10]. *Law:* L10.

**[VIEW-10] View return provenance, and the same-region trap made an error.**
[FN-1]'s slice-result ceiling applies unchanged to each view type: a function whose
written result is `own Span<'r, T>` (respectively `MutSpan`, `AppendView`) has the
ceiling containing `immutable-const` and the formal-view origin of every parameter
whose written mode and type are exactly that same view type with the same formal
region and element type. A borrow-mode result of direct view type stays rejected
for [FN-1]'s stated reason.

The consequence a caller cannot see is made a declaration error rather than a
discovery: **an ordered result list [CALL-5] containing two results of the same
view type and the same formal region is a hard error citing VIEW-10 at the
`result_binding` of the second**, with the restructuring `give each result its own
formal region`. Without this rule a three-output demux written with one region
`'o` returns three views each aliasing all three inputs, and every later judgment
about them is conservative for a reason nothing in the signature shows. A caller
inferring a property from something other than the callee's declared types is D1;
a *callee* declaring a summary that silently collapses three outputs into one
alias set is the same defect from the other side.
*Amends:* [FN-1] by generalizing "slice" to "view" and by adding the same-region
result rejection. *Law:* L10, L11.

### 3.8 `[LIV]`: affine liveness and reinitialization

**[LIV-1] Liveness is join-checked, and that is what makes release
unconditional.** A binding's live-or-dead status is a property of a program point,
not of a path: at every join of the conservative structural graph [FN-1], and at
every loop head, every predecessor must agree on the status of every binding in
scope. A disagreement is a hard error citing LIV-1 at the join, naming the two
predecessors and the binding, with the restructuring `move or reinitialize the
binding on both paths, or move it into the branch that consumes it`.

Today the compiler answers this class with `Semantics/Unsupported: OwnershipJoin`
(probe `f3`) and [OWN-11] avoids it a second way, by forbidding an outer binding
to be moved inside a loop body. Both are avoidance. Once [LIV-2] admits
reinitialization, liveness stops being monotone and the question has to be
answered, because [STOR-3] carries releases on edges and "no release action is
conditional": one static edge with two runtime dispositions is a double free on
one path or a leak on the other, and L12 forbids the runtime discriminant that
would tell them apart.

With LIV-1 in hand, [OWN-11]'s move prohibition is **replaced** rather than
lifted: a binding declared outside a loop body may be moved inside it exactly when
the loop head and every exit edge see it with one status, which for the
value-in/value-out idiom means it is restored before the backedge. [OWN-11]'s
borrow half is unchanged: a `borrow_expr` inside a loop body still names only
regions introduced inside that body, which the programs of section 4 satisfy by
opening a region inside the loop, exactly as `docs/patterns.md` P15 already
prescribes.
*Judgment:* a per-join structural check over the ownership state the checker
already computes; no search. *Publishes:* the unconditional release set of every
edge. *Amends:* [OWN-1] and [OWN-11]. *Law:* L17.

**[LIV-2] Reinitializing `set`.** `set p = e;` is additionally admitted when `p` is
a bare binding of affine type declared in the current function, a `let` binding or
a parameter, **whose current value has already been consumed** (by `e` itself, or
by an earlier statement of the same lexical block) and `e` produces exactly `p`'s
type.

```wf-design
set buf = collect(source: move line, out: move buf);
```

Its judgment: evaluate `e` under ordinary rules, including the consume of `p`
inside it; every fact whose support contains `p`'s root dies at that consume
([ENT-5] clause (c)); then the binding is reinitialized with `e`'s value, live and
usable, with no observable program point between. It derives no drop and no
release, because the target holds no value, exactly as [SET-2]'s commit derives
none. Its measure images transfer by [MSR-3]: `e`'s declared relations over its
consumed operand's entry image and its result install `p`'s new images, which is
the mechanism that carries a length or a spare across a loop backedge.

The premise is one fact the checker already tracks: **the target is dead**.
[STOR-1]'s existing rejection of a `set` on a *live* affine place keeps its exact
wording and its `replace` mechanical fix; only the dead case is added. Because the
premise is deadness rather than "the right-hand side is a call", the same statement
carries an owner out of a multi-return binding into a loop-carried name:

```wf-design
let (rest, next) = seq_try_take(vector: move pending);
set pending = move rest;
```

*Amends:* [OWN-1] (one reinitialization route that is not a new `let`), [STOR-1]
and [SET-1] (whose affine-target rejections narrow to a live target). It is the
sole writer-facing cost of L10, and it buys the whole write-back story. Lowering
is the ABI note the owner already made: a parameter moved in and returned on every
path is passed by pointer, so `set view = seq_push(view: move view, value: byte);`
lowers to a store and a length increment on one in-place descriptor, with no copy.
*Verified today:* probe `p10` is [STOR-1] `AffineSetTarget` and probe `p11` is
[OWN-1] `UseAfterMove`, the two halves of the premise. *Law:* L10, L16, L17.

**[LIV-3] No second assignment form.** The statement form already exists; only its
premise widens, so the language gains no second way to write an assignment and
[GRAM-4] is untouched. In particular this design does not add a receiver-position
call form: `view.seq_push(value: byte)` would be a second call syntax whose
resolution [GRAM-5] does not have and whose cost is larger than the noise of
naming a target three times. *Law:* L10.

### 3.9 `[CALL]`: what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the
callee's declared parameter modes and types and its declared contract.** These are
the owner's three call rules of 2026-09-03.

**[CALL-1] Through a shared borrow, every fact survives.** For an argument whose
parameter mode is `&'r`, of any type, container and view included, the call is not
a kill event for any fact supported by the actual's resolved place. Ground:
[OWN-5] admits no write through a shared holder, so [EFF-2] can project no
`writes` occurrence onto that place, so [ENT-5] clause (b) does not fire.
*Verified today* for `&'a buffer<u8>`: probe `p6` keeps `len(line) = 10` across the
call and the subsequent `line[9_u64]` is accepted. *Law:* L11.

**[CALL-2] Through a value passed and returned, only the contract's facts exist on
the result.** An `own` argument is a consuming use [OWN-1], so [ENT-5] clause (c)
kills every fact whose support contains that binding's root. The result is a fresh
binding carrying exactly the callee's verified relations under [ENT-3.S12], and
nothing else. Those relations may name the consumed parameter's **entry image**,
which is [FN-9]'s existing machinery and needs no new term: `len(result) = len(view) + 1`
means what it reads as, and [MSR-3] turns it into the result's image.
*Verified today:* probe `p1`, `passthrough(out: move a)` returning the same buffer,
then `b[9_u64]`, is **rejected** with residual `9_u64 < len(b)`. The transport
already behaves correctly; what was missing is the vocabulary to publish across
it, which is [CALL-4]. *Law:* L11.

**[CALL-3] An element write through a length-fixed view never touches length
facts.** For an argument whose parameter's declared type is `MutSpan<'r, T>` or
`&uniq 'b MutSpan<'r, T>`, which [VIEW-4] fixes a length for, a projected callee
`writes` occurrence kills every fact whose support overlaps the viewed **element
storage** and kills no measure term over that origin. For an argument whose
parameter's declared type is `AppendView<'r, T>` or `&uniq 'b AppendView<'r, T>`,
the same holds, and in addition: the callee cannot decrease the owner's length
(L14, [VIEW-5]), and cannot increase it, because only `absorb` publishes an
increase and [VIEW-7] denies `absorb` to a callee. For every other parameter type
the projected write kills measures as an ordinary whole-place event. *Law:* L11,
L14.

**[CALL-4] Contract vocabulary for containers and views.** [FN-9]'s clause
operands are terms [MSR-5], so `len(P)`, `cap(P)` and `room(P)` over an admitted
formal place are operands with no per-family admission, and so are `len(result)`,
`cap(result)` and `room(result)` when the written result type is measured, which
is the one addition beyond [MSR-5], since today's result-datum restriction to
fragment integers forbids it. So the canonical append contract is writable:

```wf-design
fn append_span['o, 'i](output: own AppendView<'o, u8>, input: own Span<'i, u8>) -> written: own AppendView<'o, u8> reads(output, input), writes(output) contract {
  requires ile(len(input), room(output));
  ensures ile(len(written), cap(output));
} { ... }
```

The clause is single-state: `output` denotes the entry image of the parameter and
`written` the result, both under [FN-9]'s existing entry-image machinery, with no
second state, no `old()`, and no frame rule. Two-state `ensures` is rejected by the
owner and is not proposed anywhere in this design.

Two further sentences of [FN-9] are amended, and the first is load-bearing for
every helper in this design. **A consuming use of an `own` parameter does not
invalidate that parameter's entry image.** [FN-9] today makes an entry image
"permanently unavailable on the first structural edge whose [ENT-5] kill overlaps
the datum", and a consume is such a kill; but the entry image denotes the value the
caller supplied, and moving a value does not change it. Only a write through a path
reaching that state can. Without this amendment the value-in / value-out shape and
the contract vocabulary are mutually exclusive: `let acc = move out;` would delete
`cap(out)` from the clause that names it, so no helper that consumes its owner
could state anything about it, which is every helper here. And **when the written
result is an ordered list [CALL-5], each result binding is a datum of every clause
of that function, and one clause may name more than one**, which is what
`ensures ile(written, len(rest))` in 4.1 needs.

*Verified today:* probe `p4` compiles a single-state `ensures` anchored on
`len(deref(destination))` with a fragment result, so the entry-image half works;
probe `p2` shows `len(result)` does not parse today. *Law:* L11, L16.

**[CALL-5] Multi-return.** A function may declare an ordered result tuple:

```wf-design
fn seq_take(vector: own FixedVector<Task, 32>) -> (rest: own FixedVector<Task, 32>, value: own Task) ...

let (rest, task) = seq_take(vector: move pending);
```

Each element has its own mode, type and contract relations; the destructuring
`let` binds each as an ordinary fresh binding. The result is not a value: there is
no tuple type, no tuple place, and no way to store or pass one. It is a
return-and-bind form only, which keeps [CNT-6] and [TYPE-2] untouched.
Multi-return is load-bearing, not a convenience: `seq_take` must return an owner
and an element, and no single value can carry both, since an enum payload holding
a confined or provenance-bearing value is refused by [CNT-6].
Three productions change together and none of them may be left implicit:
`result_binding` may be one binding or a parenthesized list of them, `let_stmt`
may bind one IDENT or a parenthesized list, and `return_stmt` may carry one `expr`
or a comma-separated list whose length equals the function's. Every element is
judged independently by the ordinary [FN-1] return rule.
*Amends:* [GRAM-2]'s `fn_decl` result shape, [GRAM-4]'s `let_stmt` and
`return_stmt`, [FORM-2]'s rendering (below), [FN-1] and [FN-9]'s result shapes.
*Verified today:* the syntax does not parse ([GRAM-2], probe `p8`), so this is new
syntax. *Law:* L10.

Its canonical rendering is stated rather than left to [FORM-2]'s attachment sets,
which would produce `->(a: own T)` and `let(a, b)`: a result list renders as
`-> (` then its comma-separated bindings then `)`, with the stated space after
`->` overriding the generic right attachment of `(`; a destructuring `let` renders
as `let (` then its comma-separated binders then `) = `, with the stated space
after `let`; and a multi-value `return` renders its expressions comma-separated on
one line, which the existing comma attachment already fixes. This is the
`for_stmt` precedent (spec 71) applied where the attachment sets would otherwise
decide silently.

**[CALL-6] No transport reads the actual's spelling.** The three transports above
are selected by the callee's declared parameter mode and type and by its declared
contract. No rule of this design consults the argument expression's shape, the
callee's body, its name, or any per-parameter summary derived from its body. A
parameter type for which no transport is selected kills conservatively.

*This is D1 stated as a rule.* The located mechanism of D1, `argument_referent`
returning `element = true` for every `&uniq buffer<T>` actual, is a fact derived
from the actual's shape, and under CALL-6 no such fact exists to be derived. The
precision it was buying is bought instead by the type: a `MutSpan` argument is
element-only **because its type admits nothing else**. Applying CALL-6 to the
residual `&uniq buffer<T>` spelling yields `element = false`, which is exactly the
sweep's minimal sound repair, and is why the D1 conformance case turns XPASS at
[OP-4] in the batch that lands this family (section 7). *Law:* L11.

### 3.10 `[SEQ]`: the operation inventory

**[SEQ-0] The container declaration domain.** The container and provider
operations are one compiler-owned declaration domain, built exactly as [SYS-1] and
[SYS-2] build the system domain: each operation is one complete signature record
with named parameters in declared order [GRAM-11], region parameters written as
`targs`, one declared effect row, one declared result mode and type, one declared
requirement list, and **one declared relation list**, which is the fact source
[ENT-3] needs and which the first draft's "publishes" column did not have. Their
relations are established on the result exactly as [ENT-3.S12] establishes a
verified user summary, and their parameter operands denote entry images exactly as
[FN-9]'s do.

`len`, `cap` and `room` are **not** in this domain: they are three [OP-1] table
operations taking a bare non-consuming place operand and returning `own u64`,
because their spelling is also the term spelling of [MSR-1] and one quantity
should have one name. That is table data, not an exception clause: [OP-1] lists
three rows and the container domain lists the rest.
One naming note, because a diagnostic makes it visible. `SEQ-1` through `SEQ-22`
are **inventory row ids**, not rules: `SEQ-0` is the only rule of this family. A
diagnostic citing `SEQ-5` therefore cites a row of an inventory the way a
diagnostic citing an [OP-1] operation cites a row of that table, and its owning
rule is `SEQ-0`. Every other family id in this file names a rule with a judgment,
a published fact and a law.

*Amends:* [ENT-3] gains one enumerated source, S13, for a container-domain
operation's declared relations; [OP-1] gains three rows and loses `slice_of`,
`buffer_new`, `buffer_vacant` and the `len` domain restriction; `ReservedLowerNames`
gains `cap` and `room` (META-5). *Law:* L11, L16.

The inventory. `V` ranges over the four prefix owners; `Prov` is the owner's
provider.

```text
| id      | op              | receiver     | signature                                                                                    |
|---------|-----------------|--------------|----------------------------------------------------------------------------------------------|
| SEQ-1   | seq_fixed<T,N>  | -            | () -> own FixedVector<T, N>                                                                    |
|         | seq_ring<T,N>   | -            | () -> own FixedRing<T, N>                                                                      |
|         | seq_heap<T>     | -            | () -> own HeapVector<T>                                                                        |
|         | seq_arena<'r,T> | -            | () -> own ArenaVector<'r, T>                                                                   |
| SEQ-2   | seq_filled<T,N> | -            | (value: own T) -> own FixedVector<T, N>                       T copy                           |
|         | seq_heap_filled | -            | (heap: &uniq 'h Heap, count: own u64, value: own T)                                            |
|         |                 |              |   -> own Result<HeapVector<T>, OutOfMemory<unit>>             T copy                            |
| SEQ-3   | seq_lease       | -            | ['p, 'b](pool: &uniq 'b Pool<'p, FixedVector<T,N>, K>)                                         |
|         |                 |              |   -> own Result<PoolVector<'p,T,N>, PoolExhausted<unit>>                                       |
|         | seq_lease_proved| -            | ['p, 'b](pool: &uniq 'b Pool<'p, FixedVector<T,N>, K>) -> own PoolVector<'p, T, N>             |
| SEQ-4   | len, cap, room  | owner, view, | bare place -> own u64                                          [OP-1] rows                     |
|         |                 | provider     |                                                                                                |
| SEQ-5   | seq_push        | AppendView   | (view: own AppendView<'r,T>, value: own T) -> own AppendView<'r, T>                            |
| SEQ-6   | seq_try_push    | AppendView   | (view: own AppendView<'r,T>, value: own T) -> (rest: own AppendView<'r,T>, unplaced: own Option<T>) |
| SEQ-7   | seq_pop         | AppendView   | (view: own AppendView<'r,T>) -> (rest: own AppendView<'r,T>, value: own T)                     |
| SEQ-8   | seq_truncate    | AppendView   | (view: own AppendView<'r,T>, keep: own u64) -> own AppendView<'r, T>                           |
| SEQ-9   | seq_place       | owner, ring  | (vector: own V<T>, value: own T) -> own V<T>                                                   |
| SEQ-10  | seq_try_place   | owner, ring  | (vector: own V<T>, value: own T) -> (rest: own V<T>, unplaced: own Option<T>)                  |
| SEQ-11  | seq_take        | owner        | (vector: own V<T>) -> (rest: own V<T>, value: own T)                                           |
|         | seq_take_at     | owner        | (vector: own V<T>, index: own u64) -> (rest: own V<T>, value: own T)                           |
|         | seq_take_front  | ring         | (ring: own FixedRing<T,N>) -> (rest: own FixedRing<T,N>, value: own T)                         |
| SEQ-12  | seq_try_take    | owner, ring  | (vector: own V<T>) -> (rest: own V<T>, value: own Option<T>)                                   |
| SEQ-13  | seq_exchange    | owner, ring  | (vector: own V<T>, first: own u64, second: own u64) -> own V<T>                                |
| SEQ-14  | p[i]            | prefix owner,| element place                                                                                  |
|         |                 | Span, MutSpan|                                                                                                |
| SEQ-15  | seq_at          | ring         | (ring: &'r FixedRing<T,N>, index: own u64) -> own T             T copy                          |
| SEQ-16  | seq_span        | prefix owner | (vector: &'r v) -> own Span<'r, T>                                                             |
| SEQ-17  | seq_mut_span    | prefix owner | (vector: &uniq 'r v) -> own MutSpan<'r, T>                                                     |
| SEQ-18  | seq_append_view | prefix owner | (vector: &uniq 'r v) -> own AppendView<'r, T>                                                  |
| SEQ-19  | absorb          | AppendView   | (view: own AppendView<'r,T>) -> own u64                                                        |
| SEQ-20  | seq_reserve     | HeapVector,  | (vector: own V<T>, provider: Prov, additional: own u64)                                        |
|         |                 | ArenaVector  |   -> own Result<V<T>, OutOfMemory<V<T>>>                                                       |
| SEQ-21  | seq_clear       | owner, ring  | (vector: own V<T>) -> own V<T>                                                                 |
| SEQ-22  | seq_shrink      | HeapVector   | (vector: own HeapVector<T>, heap: &uniq 'h Heap)                                               |
|         |                 |              |   -> own Result<HeapVector<T>, OutOfMemory<HeapVector<T>>>                                     |
```

Requirements, declared relations, effects and failures:

```text
| id      | requires                | declares                                                              | effects            |
|---------|-------------------------|-----------------------------------------------------------------------|--------------------|
| SEQ-1   | -                       | len(result) = Z, cap(result) = N (or Z for a growable)                 | pure               |
| SEQ-2   | -                       | seq_filled: len(result) = N, cap(result) = N                           | pure / heap:       |
|         | heap: buffer_fits<T>    | seq_heap_filled Ok(value: r): len(r) = count, cap(r) = count           | allocates(heap),   |
|         |                         |                                                                       | writes(heap)       |
| SEQ-3   | proved: igt(room(pool), | Ok(value: r): len(r) = Z, cap(r) = N; len(pool)' = len(pool) + 1       | allocates(pool),   |
|         | Z)                      | Err: room(pool) = Z                                                    | writes(pool)       |
| SEQ-4   | -                       | n = len(v) / cap(v) / room(v)                                          | reads(v)           |
| SEQ-5   | igt(room(view), Z)      | len(result) = len(view) + 1, room(result) = room(view) - 1             | reads(view),       |
|         |                         | cap(result) = cap(view)                                                | writes(view)       |
| SEQ-6   | -                       | None: len(rest) = len(view) + 1, room(rest) = room(view) - 1           | reads, writes(view)|
|         |                         | Some: len(rest) = len(view), room(rest) = Z                            |                    |
| SEQ-7   | igt(len(view), Z)       | len(rest) = len(view) - 1, room(rest) = room(view) + 1                 | reads, writes(view)|
| SEQ-8   | ile(keep, len(view))    | len(result) = keep, cap(result) = cap(view)                            | reads, writes(view)|
| SEQ-9   | igt(room(vector), Z)    | len(result) = len(vector) + 1, cap(result) = cap(vector)               | reads, writes      |
|         |                         |                                                                       | (vector), plus the |
|         |                         |                                                                       | owner's provider   |
| SEQ-10  | -                       | None: len(rest) = len(vector) + 1;  Some: len(rest) = len(vector),     | as SEQ-9           |
|         |                         | room(rest) = Z                                                        |                    |
| SEQ-11  | igt(len(vector), Z)     | len(rest) = len(vector) - 1, cap(rest) = cap(vector)                   | as SEQ-9           |
|         | seq_take_at additionally|                                                                       |                    |
|         | ilt(index, len(vector)) |                                                                       |                    |
| SEQ-12  | -                       | Some: len(rest) = len(vector) - 1;  None: len(rest) = Z,               | as SEQ-9           |
|         |                         | len(vector) = Z                                                       |                    |
| SEQ-13  | ilt(first, len(vector)),| len(result) = len(vector), cap(result) = cap(vector)                   | pure               |
|         | ilt(second, len(vector))|                                                                       |                    |
| SEQ-14  | ilt(i, len(p))  [OP-4]  | -                                                                     | per access         |
| SEQ-15  | ilt(index, len(ring))   | -                                                                     | reads(ring)        |
| SEQ-16  | -                       | [VIEW-2]                                                              | reads(vector)      |
| SEQ-17  | -                       | [VIEW-2]                                                              | reads(vector)      |
| SEQ-18  | -                       | [VIEW-2]                                                              | reads(vector)      |
| SEQ-19  | -                       | [VIEW-6]                                                              | reads, writes(view)|
| SEQ-20  | -                       | Ok(value: r): cap(r) = cap(vector) + additional,                       | reads(vector,      |
|         |                         | len(r) = len(vector);  Err: the vector returns inside the error        | provider), writes  |
|         |                         |                                                                       | (both), allocates  |
|         |                         |                                                                       | (provider)         |
| SEQ-21  | -                       | len(result) = Z, cap(result) = cap(vector)                             | as SEQ-9           |
| SEQ-22  | -                       | Ok(value: r): len(r) = len(vector), cap(r) = len(vector);              | allocates(heap),   |
|         |                         | Err: the vector returns inside the error                              | writes(heap)       |
```

Notes on the inventory.

- **[SEQ-5] is the operation the whole design exists for.** It is total,
  allocation-free on every backing, and lowers to `store` plus `len + 1` with no
  capacity branch, because its requirement is discharged before lowering. With
  `room` readable [SEQ-4], with the identity of [MSR-2], and with the image
  transfer of [MSR-3], it is discharged in a loop by a header invariant, by a
  dominating branch on `room`, or by a `requires` at the boundary: three routes
  where the first draft had none.
- **Every `try` row publishes per-arm relations**, which is free: the rows are
  declared relations and [ERR-3]/[OWN-13] already dispatch on the variant. The
  first draft published one relation on both arms, so a conditional append loop
  could never re-establish its header invariant.
- **There is no growing `push` anywhere.** A writer who wants push-with-growth
  writes the shell: reserve, form the view, push, absorb (L4).
- **[SEQ-11]'s three rows are what makes L12's stack honest.** `seq_take` removes
  from the end; `seq_take_at` removes from the middle by exchanging the removed
  position with the last and shortening, so the prefix stays contiguous and no
  vacancy state exists; `seq_take_front` is the ring's, because a prefix cannot
  remove from the front and the design says so rather than pretending.
- **[SEQ-13] `seq_exchange` is the primitive compaction, partition and sort need.**
  A two-index permutation needs no vacancy state, so [STOR-1]'s no-hole sentence
  is untouched; without it an object table of affine handles could be pushed and
  popped and nothing else.
- **[SEQ-2]'s filled construction is what makes a random-access table
  constructible.** `seq_fixed` gives `len = 0`, and under [CNT-3] a zero-length
  container is unreadable and unwritable until elements have been placed one at a
  time; an open-addressed table, a page table and a read destination all address
  every slot from the first use. `seq_filled` is what `array_new` and `buffer_new`
  already mean, so nothing about L12 is weakened.
- **A `par` fill needs no new type.** `seq_filled`, then `seq_mut_span`, then a
  counted loop writing `set m[i] = ...;` is exactly [PAR-2]'s proved single-binder
  affine element write, whose one amendment is that its "direct array or buffer
  subscript" reads "direct subscript of an array, a prefix owner, or a `MutSpan`".
  That is the whole of the `par` builder, with no `Builder` type, no coverage
  certificate, and no second [PAR-2] refinement.
- **[SEQ-20] returns the vector inside its error**, so a failed reserve loses
  nothing and changes nothing. The order is fixed: compute the new capacity and
  discharge its arithmetic and allocation-domain obligations [OP-9]; acquire; move
  elements; commit the descriptor; release the old backing. Nothing observable
  changes before the acquisition succeeds. **[SEQ-22] does the same**, because
  "keeps the larger backing on failure" is a fallback and L3 forbids fallbacks.
- **Nothing in the inventory is total at a capacity boundary.** A `FixedRing` at
  capacity refuses through [SEQ-10] like every other owner; the overwriting ring
  the first draft wrote by hand would need L9's published-displacement relation,
  and no program here needs it.

### 3.11 The pool seam, resolved

`Pool<'p, T, N>` names `N` interchangeable single-`T` slots, and a `PoolVector`
needs one **contiguous run** of them. A pool that serves *runs* of `k` slots is not
a uniform-slot domain: whether a run of 3 is serviceable is not decided by `len`,
and L6's sixteen-byte counterexample reappears at slot granularity. Adding a
run-lease would take the pool out of [RES-6]'s admitted domains and out of every
envelope.

The shape that keeps the algebra is to lease **one slot whose content is the run**:

```wf-design
region 'p {
  let blocks = pool_static<'p, FixedVector<Record, 256>, 8>();
  let leased = seq_lease<'p, 'p>(pool: &uniq 'p blocks);
  match leased {
    Ok(value: block) => { ... }
    Err(error: refused) => { ... }
  }
}
```

The pool still holds eight interchangeable slots of one type, `room >= 1` still
decides serviceability, and `PoolVector<'p, Record, 256>` is exactly a lease of
such a slot. A `FixedVector` is frame-resident storage and is region-free, so it is
a legal slot content type. Two consequences, both recorded rather than hidden: the
capacity is fixed at reservation, not at lease, so `PoolVector` carries `N` in its
type and `seq_lease` takes no runtime capacity argument; and a program wanting two
block sizes reserves two pools, so `E` names both, which is the shape L6 says an
envelope has to have.

The `Result` above is a program only because [CNT-6] admits a confined generic
argument, and the leased block is returnable from a helper only because [CNT-7]
makes confinement an outlives judgment. Those two amendments are what this section
costs.

### 3.12 One name per concept

```text
| concept                   | chosen               | why                                                     |
|---------------------------|----------------------|---------------------------------------------------------|
| construct an empty owner   | seq_fixed<T,N> etc.  | one prefix names one family; a row is selected by name   |
|                            |                      | and receiver type, never by expected result type         |
| construct a filled owner   | seq_filled<T,N>      | what array_new and buffer_new already mean               |
| append one element         | seq_push (view),     | the backing is in the receiver type, not the name        |
|                            | seq_place (owner)    |                                                          |
| remove one element         | seq_pop (view),      | a view cannot remove what another view appended (L14)    |
|                            | seq_take, seq_take_at,|                                                         |
|                            | seq_take_front       |                                                          |
| read-only view             | Span<'r, T>          | the rename is the whole change to slice<'r, T>           |
| the three measures         | len, cap, room       | one quantity, one name, term and reader alike            |
| lease a pool block         | seq_lease            | capacity comes from the pool's slot type (3.11)          |
| growth failure             | OutOfMemory<V>,      | L3 requires the failure to hand back the affine input,   |
|                            | PoolExhausted<T>,    | so a payload-carrying family wins over one opaque union; |
|                            | NeedCapacity<T>      | each has one field, rejected                             |
| rebind a consumed owner    | set p = e;           | the owner's spelling; the premise is deadness, so the    |
|                            |                      | language gains no second assignment form                 |
| the property               | resource-closed      | the long spelling is the one in use                      |
| the failure variant field  | Err(error: e)        | [PRE-1] declares Err(error: E)                           |
```

`Full<T>` and `TooSmall` are **not** in the vocabulary: the first draft named them
and no row produced either, because the `try` forms return `Option<T>` instead.

### 3.13 Amendment register

Three lists, because a register that mixes them cannot be read as a change set.

**Changed.** Line numbers are `spec/kernel-spec.md` at a40c7e70.

```text
| rule           | line      | change                                                                    | by                |
|----------------|-----------|---------------------------------------------------------------------------|-------------------|
| [SCOPE-3]      | 27-31     | heap exhaustion leaves the deferred set; stack and covered-store           | [RES-7], [STK-5]  |
|                |           | exhaustion leave it for resource-closed programs; start-resource failure   |                   |
|                |           | stays outside, by name                                                    |                   |
| [FORM-2]       | 52-76     | +2 rendering sentences: the result list and the destructuring let, on the  | [CALL-5]          |
|                |           | for_stmt precedent                                                        |                   |
| [GRAM-2]       | 176-179   | fn_decl admits an ordered result list; program_kind admits the            | [CALL-5], [RES-4] |
|                |           | resource_closed marker (+1 fixed atom)                                    |                   |
| [GRAM-4]       | 220-243   | let_stmt admits a destructuring binder list and return_stmt a comma-       | [CALL-5], [MSR-5] |
|                |           | separated expression list; affine_factor admits the [ENT-2] place grammar  |                   |
|                |           | and the three measure terms                                               |                   |
| [GRAM-9]       | 323-327   | unchanged in force, given a stated scope: it governs runtime evaluation    | [MSR-5]           |
|                |           | and not erased proof syntax                                               |                   |
| [GRAM-11]      | 340-345   | unchanged: container-domain operations are named-argument operations of a  | [SEQ-0]           |
|                |           | declaration domain, exactly like [SYS-2]'s                                |                   |
| [TYPE-2]       | 352       | +3 provider nominals, +1 slot, +5 owners, +2 views, slice renamed Span,    | [PROV-1], [CNT-1],|
|                |           | buffer<T> retires from the writer surface                                 | [VIEW-1]          |
| [TYPE-5]       | 369       | the retained-argument list of table operations gains nothing; the          | [SEQ-0]           |
|                |           | container domain writes region targs and named arguments instead          |                   |
| [TYPE-7]       | 471       | slot<'p, T> joins the closed deref domain beside box and arena             | [PROV-1]          |
| [SET-1]        | 500       | the affine-target rejection is narrowed to a live affine target            | [LIV-2]           |
| [OWN-1]        | 558       | providers, slots, owners and views are affine; one reinitialization route  | [LIV-1], [LIV-2]  |
|                |           | that is not a new let; liveness must agree at every join                   |                   |
| [OWN-5]        | 580       | the one slice-origin access clause becomes two, shared-strength and        | [VIEW-2],[VIEW-3] |
|                |           | exclusive-strength, and "slice value" generalizes to "view value"          |                   |
| [OWN-6]        | 611       | a child reborrow may name a caller-supplied region the parent's region     | [PROV-9]          |
|                |           | outlives-or-equals when the receiving call's result type is region-free,   |                   |
|                |           | which is the one amendment that lets a helper lend a provider onward       |                   |
| [OWN-11]       | 641       | the move prohibition is replaced by [LIV-1]'s join agreement; the borrow   | [LIV-1]           |
|                |           | half is unchanged                                                         |                   |
| [STOR-1]       | 670       | the owners join the storage-class table and slot<'p,T> is pool-owned, so   | [CNT-1], [CNT-5], |
|                |           | its content is writable through deref exactly as box and arena content is; | [LIV-2], [PROV-1] |
|                |           | buffer<T>'s sentence and the growable-collection paragraph are superseded; |                   |
|                |           | the affine-set rejection is narrowed to a live target                     |                   |
| [STOR-3]       | 683       | every provider-owned release carries a nonempty row naming its provider;   | [PROV-7], [CNT-5] |
|                |           | the owner release actions supersede the buffer<T> drop sentence           |                   |
| [STOR-4]       | 716       | confinement becomes the ordinary outlives relation, so a confined value    | [CNT-7]           |
|                |           | may be returned into a region it outlives                                 |                   |
| [STOR-5]       | 718       | the enumerated position list is replaced by the intensional split of       | [CNT-6]           |
|                |           | provenance-bearing and confined types                                     |                   |
| [STOR-6]       | 733-761   | the "no numeric frame ceiling" sentence keeps its scope for the language;  | [STK-3], [RES-3]  |
|                |           | E-materialization joins the target-stage obligations and its failure is a  |                   |
|                |           | qualification failure citing no language rule                             |                   |
| [OP-1]         | 793-828   | +cap and +room rows beside len, whose domain extends to owners, views and  | [SEQ-0], [PROV-3] |
|                |           | providers; box_new and arena_new take a provider; buffer_new,              |                   |
|                |           | buffer_vacant and slice_of retire; ReservedLowerNames +2                   |                   |
| [OP-4]         | 880       | indexable bases extend to the prefix owners, Span and MutSpan; the         | [CNT-3]           |
|                |           | obligation is against len, never cap                                       |                   |
| [OP-9]         | 968       | buffer_fits stays a representability predicate and additionally fixes the  | [RES-8]           |
|                |           | constant K<T> the arena requirement uses                                   |                   |
| [FN-1]         | 999-1030  | the slice-return ceiling generalizes to views and gains the same-region    | [VIEW-10],        |
|                |           | duplicate-result rejection; the result shape admits an ordered list; a     | [CALL-5],[RES-10],|
|                |           | loop with no resolved break and no other exit has no edge to its normal    | [FN-1 divergence] |
|                |           | successor; the boundary publishes the per-domain demand summary            |                   |
| [FN-2]         | 1087      | the region-bearing generic-argument rejection narrows to                   | [CNT-6]           |
|                |           | provenance-bearing arguments                                              |                   |
| [FN-7]         | 1210-1253 | one new input row command.heap; one new entry marker resource_closed;      | [PROV-5], [RES-4] |
|                |           | main's effect row admits allocates over its own labelled provider          |                   |
| [FN-8]         | 1256      | clause operands are terms [MSR-5], not atoms                               | [MSR-5]           |
| [FN-9]         | 1295      | clause operands are terms; a measured result admits len/cap/room; an       | [MSR-5], [CALL-4],|
|                |           | ordered result list gives one clause more than one result datum; a         | [CALL-5], [MSR-4] |
|                |           | consuming use of an own parameter does not invalidate its entry image;     |                   |
|                |           | the direct-affine route is one step of [MSR-4], not an unstated branch     |                   |
| [EFF-1]        | 1363-1372 | allocates takes formal-rooted effect paths; the atoms heap and arena       | [PROV-4]          |
|                |           | retire (META-5: fixed lowercase atoms -2)                                  |                   |
| [EFF-2]        | 1386-1421 | the empty-release-row sentence is replaced by the total rule that a        | [PROV-7],[VIEW-3] |
|                |           | provider-owned release names its provider; "slice parameter names the      |                   |
|                |           | backing" generalizes to "view parameter"                                   |                   |
| [PROG-3]       | 1499      | the start-time obligation includes materializing the selected row of E;    | [RUN-5]           |
|                |           | ProgramFinished is named; PreStart may descend the profile table           |                   |
| [PAR-1]        | 1969      | the allocates(arena 'r) region clause becomes the ordinary provider-place  | [RUN-4]           |
|                |           | projection                                                                |                   |
| [PAR-1/2/3]    | 1989,2024,| "execution-resource exhaustion is a [SCOPE-3] condition" gains the         | [RUN-3]           |
|                | 2049      | resource-closed case, in which it is unreachable                           |                   |
| [PAR-2]        | 1994      | its proved single-binder affine element write reads "a direct subscript of | [SEQ-0]           |
|                |           | an array, a prefix owner, or a MutSpan"                                    |                   |
| [SYS-2]        | 2158-2301 | the seven range-bearing operations' buffer parameters become MutSpan or    | [RUN-2], [SYS-8]  |
|                |           | Span, changing the inventory's normative counts (18 nominals, 44 operation |                   |
|                |           | value parameters, 203 declaration records); "no system operation allocates" |                   |
|                |           | gains its companion, that every adapter-owned store is an item of E        |                   |
| [SYS-8]        | 2482      | read_at, write_once, directory_next, host_copy_bytes, host_copy_utf8,      | [CNT-1], [VIEW-1] |
|                |           | open_directory and open_file take MutSpan<'r,u8> or Span<'r,u8>; the       |                   |
|                |           | start <= end and end <= len obligations keep their form with len(view).    |                   |
|                |           | This is the change that lets a heap-free program do I/O                    |                   |
| [SYS-9,11,12,  | 2523-2620 | their normative prose naming buffer<u8> is restated over views             | [SYS-8]           |
|  14]           |           |                                                                           |                   |
| [ENT-2]        | 2671-2722 | the three measure terms are one-place terms over an admitted place that    | [MSR-1], [MSR-2]  |
|                |           | may end in a subscript; the implicit-fact sentence gains their four        |                   |
|                |           | standing facts                                                            |                   |
| [ENT-3]        | 2724      | +1 enumerated source S13, the declared relations of a container-domain     | [SEQ-0], [VIEW-6] |
|                |           | operation, established as S12 is; S5 gains absorb's commit value           |                   |
| [ENT-5]        | 2857-2887 | the support sentence covers the three measure terms uniformly; the         | [MSR-2], [CALL-1],|
|                |           | element-storage exception keeps its meaning; clause (b) is the clause a    | [CALL-3],[CALL-6] |
|                |           | projected callee write uses, and [CALL-1..3] fix what it kills by type     |                   |
| [ENT-6]        | 2963-3092 | one numeric goal disposition replaces the per-family route lists;          | [MSR-3], [MSR-4]  |
|                |           | measures carry affine value images                                        |                   |
| [INV-1]        | 3095-3107 | affine atoms are the [ENT-2] place grammar and the measure terms           | [MSR-5]           |
| batch 0079     | docs/done/| the heap-refusal abort site loses its last reachable caller; the           | [RES-7]           |
| exhaustion     | 0079-...  | guard-page record survives, and for a resource-closed build its alternate  |                   |
| floor          |           | stack is an item of E [STK-4]                                             |                   |
```

**Retired outright, with no successor.** The writer-facing `&uniq buffer<T>` and
`&uniq Container` state-borrow forms ([CNT-9]); `buffer_vacant`'s `Option`-element
construction, which [CNT-4] makes unnecessary; the effect-row atoms `heap` and
`arena` ([PROV-4]); `slice_of` in favour of `seq_span`; and the first draft's
`Builder<'r, T>` type, its `[BLD]` family and its coverage certificate, whose
purpose [SEQ-2] plus [SEQ-17] plus [PAR-2] serve with no new rule.

**Deliberately unchanged, and why.** [CAP-1] 1962: providers add no capability
category, no permission kind, and no second interference vocabulary. [SET-2] 508: a
provenance-bearing target type is refused, which is exactly what keeps [CALL-3]
sound ([VIEW-4]). [FN-6] 1205: recursion stays permitted; it merely excludes a
program from [RES-4]. [OWN-7] 613: subscript overlap stays conservative, which is
what makes a view formed on `table[i]` sound. [PAR-2] and [PAR-3] beyond the one
subscript-domain word above: no builder refinement is proposed, and no
callee-projected write is promoted.

**Writer doctrine this design invalidates**, which `docs/patterns.md` must carry in
the same batch. **P16** ("One length fact above the writes") states that the
compiler honours the element-write exception across a callee boundary through a
`&uniq buffer` actual; [CALL-6] makes that kill conservative for exactly that
spelling, so P16's shape is invalid from B1 until B5 restores it over `MutSpan`,
where it is sound by type rather than by the actual. **P17**'s advice to fold a
returned record field by field remains right, and its `replace` note gains
[LIV-2]'s dead-target `set` as the third commit form. **P19**'s join rule is
unchanged and gains a case: a measure term joins by the same delta-atom rule, so a
conditional append re-establishes its header invariant where a conditionally
advanced binding does. **P15** is unchanged. **P8** should gain the sentence probe
`q5'` bought: an exact `-` carries an ordering into a backedge where `-wrap` gives
the checker a fresh atom.

---

## 4. Two worked programs

Both are **design text**, and the forms this design adds compile nowhere. What
changed since the first draft is the standard they are held to: every statement
below is written to be accepted by a compiler implementing section 3's rules
**and the unchanged v0.40 rules**. The first draft's two programs were walked
statement by statement by a falsifier and about twenty-five statements were
refused by rules the design never claimed to amend: nested `construct`s and
`call`s in atom positions [GRAM-9], missing region arguments [FN-2], bare `&uniq`
holders passed as arguments [OWN-1], one-line invariant-bearing loop headers
[FORM-2], a `TYPEID`-shaped named const [FORM-3], a suffixless literal [FORM-5],
over-wide effect rows [EFF-2], and a slot content access [TYPE-7] never
registered. One more that no report caught is fixed here too: [FORM-2] puts a
function's complete header through its body `{`, or through `contract {`, on **one
line**, so every wrapped signature in the first draft was a canonical-form error
before any semantic rule ran. Those are fixed, and where a form is genuinely new it
is named.

Byte figures in the envelopes are illustrative; no implementation computed them.

### 4.1 A kernel program with the heap absent

A fixed run queue of tasks, a 256-byte transmit ring, and an eight-block pool with
typed exhaustion. No heap, no recursion, an acyclic call graph, and a scheduler
loop whose resource state is restored on every backedge.

```wf-design
struct Task {
  kind: u32;
  arg: u64;
}

fn render['w](block: own PoolVector<'w, u8, 256>, task: own Task) -> (rest: own PoolVector<'w, u8, 256>, written: own u64) reads(block), writes(block) contract {
  requires ige(room(block), 8_u64);
  ensures ile(written, len(rest));
} {
  doc "Writes one eight-byte record for a task into the block and reports the count.";
  let narrowed = cvt<u32, u8>(task.kind);
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
    let view = seq_append_view<'f>(vector: &uniq 'f block);
    for @fill (
      at in 0_u64..8_u64,
      invariant spare: ige(room(view) + at, 8_u64)
    ) {
      set view = seq_push(view: move view, value: mark);
    }
    set total = absorb(view: move view);
  }
  return move block, total;
}

fn drain['w, 'b](block: &'b PoolVector<'w, u8, 256>, ring: own FixedRing<u8, 256>, count: own u64) -> (rest: own FixedRing<u8, 256>, sent: own u64) reads(block, ring), writes(ring) contract {
  requires ile(count, len(deref(block)));
} {
  doc "Copies one prefix of the block into the transmit ring and reports how many bytes it placed.";
  let out = move ring;
  let placed = 0_u64;
  for @copy (at in 0_u64..count) {
    let byte = deref(block)[at];
    let (rest, unplaced) = seq_try_place(vector: move out, value: byte);
    set out = move rest;
    match unplaced {
      None() => {
        set placed = placed +wrap 1_u64;
      }
      Some(value: dropped) => {
      }
    }
  }
  return move out, placed;
}

resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Runs a fixed scheduler over a leased block pool and a transmit ring until the run queue empties.";
  let ring = seq_ring<u8, 256>();
  let pending = seq_fixed<Task, 32>();
  let first = Task(kind: 65_u32, arg: 0_u64);
  let (queued, unplaced) = seq_try_place(vector: move pending, value: move first);
  set pending = move queued;
  match unplaced {
    None() => {
    }
    Some(value: rejected) => {
      return exit_status(code: 1_u8);
    }
  }
  region 'p {
    let blocks = pool_static<'p, FixedVector<u8, 256>, 8>();
    loop @scheduler {
      let (rest, next) = seq_try_take(vector: move pending);
      set pending = move rest;
      match next {
        None() => {
          break @scheduler;
        }
        Some(value: task) => {
          region 'c {
            let leased = seq_lease<'p, 'c>(pool: &uniq 'c blocks);
            match leased {
              Ok(value: block) => {
                let (filled, written) = render<'p>(block: move block, task: move task);
                let (fed, sent) = drain<'p, 'c>(block: &'c filled, ring: move ring, count: written);
                set ring = move fed;
              }
              Err(error: refused) => {
                let (fed, lost) = seq_try_place(vector: move ring, value: 33_u8);
                set ring = move fed;
                match lost {
                  None() => {
                  }
                  Some(value: gone) => {
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
E(scheduler.wf, <embedded target>) row W = 1

  region  static.image          bytes     1_024   align   8   contiguous
  stack   entry                 bytes     6_912   align  16   contiguous
  lanes                         count         1
  slots   task.records          count         0
  slots   completion.records    count         0
  slots   handle.table          count         0
```

```text
| item                | where it comes from                                                | rule            |
|---------------------|--------------------------------------------------------------------|-----------------|
| static.image        | the const items and the static parts of the emitted module         | [STOR-6]        |
| stack.entry         | main's frame, which holds the ring (256 B), the FixedVector<Task,  | [STK-3],        |
|                     | 32> (512 B) and the one pool_static occurrence's extent            | [PROV-8]        |
|                     | (8 * 256 B = 2 KiB), plus render and drain, plus the runtime       |                 |
|                     | frames beneath main and its bounded teardown; measured             |                 |
|                     | post-codegen over the whole chain                                  |                 |
| lanes = 1           | no par in the program; the entry lane only                         | [RUN-3]         |
| every slots row = 0 | no par statement, no may-suspend operation, no system handle       | [RUN-3],        |
|                     |                                                                    | [RES-6]         |
```

The pool is a frame item and not a static one, which is [PROV-8] and is a change
from the first draft. It costs 2 KiB of `stack.entry`, and it buys three things
the static form could not have: the extent is a place [PAR-1] can see, a second
activation would get a second extent rather than silently sharing one, and the
figure a deployment sizes against is one number rather than two.

#### Why it is source-resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | main declares pure and selects no command.heap, so [PROV-6]'s closure over   |
|                       | the call graph is empty and [RES-5] does not fire                           |
| acyclic call graph    | main -> {render, drain, the container domain}; render -> the container       |
|                       | domain; drain -> the container domain. No cycle, so [STK-1] rewrites nothing |
|                       | and [STK-2] passes                                                          |
| pool demand bounded   | the lease and its derived release are on the same path, so the arm's map is  |
|                       | (peak 1, delta 0) on 'p and the Err exit is (0, 0). The scheduler loop's     |
|                       | backedge delta is 0, so by 3.3.1's loop rule no iteration bound is needed;   |
|                       | the loop runs as long as tasks keep arriving, and len(blocks) <= 1 throughout|
| ring bounded          | fixed storage in main's frame; seq_try_place refuses at capacity and returns |
|                       | the byte, and drain reports what it placed, so the displacement is published |
|                       | (L9) rather than silent                                                     |
| queue bounded         | FixedVector<Task, 32>, storage in main's frame, len <= cap structurally      |
| stack bounded         | one context, one chain, measured after code generation [STK-3]              |
| no reentrancy         | the program declares no signal handler, interrupt handler or FFI callback,   |
|                       | so [STK-4] is satisfied from source                                         |
| runtime closed        | W = 1, no task or completion records; every runtime store's peak is zero     |
```

#### The writer's-eye walkthrough

`let ring = seq_ring<u8, 256>();` publishes `len = Z`, `cap = 256`, `room = 256`.
It replaces the first draft's hand-rolled `UartRing`, an `array<u8, 256>` beside a
`head` and a `fill` whose correspondence nothing checked, which is exactly the
two-values-that-must-agree shape the design rejects elsewhere, and which forced a
`ring_index` helper, two `invariant` statements, and an `if head_ok` guard whose
false edge could not be taken. All of that is gone because the ring is an owner
with typestate.

`let (queued, unplaced) = seq_try_place(vector: move pending, value: move first);`
is **[CALL-2]** and **[CALL-5]**. `move pending` kills every fact supported by
`pending`; what survives on the result is exactly [SEQ-10]'s per-arm relations,
and *per-arm* is the change: on the `None()` arm the checker learns
`len(queued) = len(pending) + 1`, and on the `Some` arm it learns
`room(queued) = Z`. The first draft published one relation on both arms, so no
conditional append loop could ever re-establish an invariant.
`set pending = move queued;` is **[LIV-2]**.

`region 'p { let blocks = pool_static<'p, FixedVector<u8, 256>, 8>(); }` reserves
one extent in main's frame [PROV-8] and publishes `len(blocks) = Z`,
`cap(blocks) = 8`, `room(blocks) = 8`. A provider's measures are [MSR-1]'s, the
same three terms a container has, which is why 3.3.1's algebra and the writer's
invariants read the same over both.

`loop @scheduler { ... }` moves `pending` and `ring` from inside the loop body,
which [OWN-11] forbade outright. **[LIV-1]** replaces that prohibition with the
condition that actually matters: both bindings are restored on every backedge and
are live on the `break` edge, so every predecessor of the loop head and of the
continuation agrees, and the compiler-derived release on the region-exit edge is
unconditional. A program that moved `ring` out on one arm and not the other would
be refused at the join, naming both predecessors, instead of leaving [STOR-3] with
one edge and two runtime dispositions.

`region 'c { ... }` is opened **inside** the loop body, which is what lets
`&uniq 'c blocks` and `&'c filled` be written there at all: [OWN-11]'s borrow half
is unchanged, and this is the shape `docs/patterns.md` P15 already prescribes.

`let leased = seq_lease<'p, 'c>(pool: &uniq 'c blocks);` is the resource statement,
and it is a program only because of two amendments. Its result type is
`Result<PoolVector<'p, u8, 256>, PoolExhausted<unit>>`, which the first draft could
not write at all: [FN-2] rejects a region-bearing generic argument and [STOR-5]
rejects a region-bearing enum payload, so every checked acquisition in the first
draft was untypeable and the design silently became the strict-proof form owner
ruling R12 rejects. [CNT-6] admits it because the instance's own type names `'p`,
so the instance is confined and [STOR-4] governs it, which is the protection
[STOR-5] wanted, obtained without a hole.

The `Err` arm pushes one byte and drops nothing else; the `Ok` arm binds `block`,
whose derived release at the end of `region 'c` returns the lease to the pool and
exhibits `writes` on the pool path [PROV-7]. That release edge reaches `blocks`,
which is live and writable there, so the premise [PROV-7] states is discharged
rather than assumed, and a program that moved the pool away first would be refused
at the scope exit with the provider named.

`render<'p>(block: move block, task: move task)` is the value-in / value-out
helper the whole surface is built on, and its body is the statement the first
draft could not prove:

```wf-design
    for @fill (
      at in 0_u64..8_u64,
      invariant spare: ige(room(acc) + at, 8_u64)
    ) {
      set acc = seq_push(view: move acc, value: mark);
    }
```

The header target is an affine relation over a measure term, an ordinary integer
binder and a literal, which [MSR-5] admits as atoms and [GRAM-4] refused at parse
before. Its base holds because [VIEW-2] publishes `cap(acc) = room(block)` and the
`requires` gives `room(block) >= 8`. Its backedge holds because [MSR-3] transfers
`room(acc)`'s image by [SEQ-5]'s declared `room(result) = room(view) - 1` while the
binder's image grows by one, so the sum is unchanged and no writer premise is
needed. And [SEQ-5]'s own requirement `igt(room(acc), Z)` follows from the header
target and S11's `at < 8` by [MSR-4]'s unordered-pair family. The first draft had
none of this: `room` had no relation to `len` and `cap`, no reader, and no image,
so its central operation was dischargeable only in straight-line code and every
append loop needed a hand-maintained mirror counter.

`set total = absorb(view: move acc);` is the commit ([VIEW-6]). The origin resolves
to `block`; the commit value `w` is bound with `w = len(acc)`; the facts supported
by `block` die; then `len(block)`'s image becomes `image(len(block)) + image(w)`.
That is an image transfer and therefore exact, where the first draft could carry
the sum only when the old length happened to be a constant. `ensures ile(written, len(rest))`
discharges from it.

`drain<'p, 'c>(block: &'c filled, ...)` is **[CALL-1]**: the block is passed as a
shared borrow, so the call is a kill event for nothing, and `len(filled)` survives
it. Its `requires ile(count, len(deref(block)))` is discharged at the call site
from `render`'s `ensures`.

**One deferral, stated rather than hidden.** The ring is a transmit buffer and
this program has no way to reach a device: `main`'s effect row may name only its
own labelled inputs [FN-7], and the `command` table has no `command.uart` and no
bare-metal program kind. That is open question Q10, and it is why 4.1 is a
scheduler rather than a driver. The first draft wrote `writes(uart)` on [RES-4]'s
example entry, which names nothing and is an [EFF-1] rejection.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one named operation with a typed
failure, the append helper takes the owner by value and returns it, and
`OutOfMemory` is a value on an ordinary edge.

```wf-design
const ceiling: u64 = 4096_u64;

fn collect['o, 's](out: own AppendView<'o, u8>, source: own Span<'s, u8>) -> filled: own AppendView<'o, u8> reads(out, source), writes(out) contract {
  requires ile(len(source), room(out));
  ensures ile(len(filled), cap(out));
} {
  doc "Appends every byte of source into the view's spare window.";
  let count = len(source);
  let acc = move out;
  for @copy (
    at in 0_u64..count,
    invariant spare: ige(room(acc) + at, count)
  ) {
    let byte = source[at];
    set acc = seq_push(view: move acc, value: byte);
  }
  return move acc;
}

fn grow['h](buf: own HeapVector<u8>, heap: &uniq 'h Heap, additional: own u64) -> outcome: own Result<HeapVector<u8>, OutOfMemory<HeapVector<u8>>> reads(buf, heap), writes(buf, heap), allocates(heap) {
  doc "Reserves spare capacity, handing the vector back unchanged when the store refuses.";
  let outcome = seq_reserve<'h>(vector: move buf, provider: &uniq 'h deref(heap), additional: additional);
  return move outcome;
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
        let buf = move ready;
        region 'fill {
          let view = seq_append_view<'fill>(vector: &uniq 'fill buf);
          region 's {
            let line = seq_span<'s>(vector: &'s input);
            let done = collect<'fill, 's>(out: move view, source: move line);
            set total = absorb(view: move done);
          }
        }
        region 'w {
          let body = seq_span<'w>(vector: &'w buf);
          let outcome = write_once<'h, 'w>(output: &uniq 'h sink, source: move body, start: 0_u64, end: total);
          match outcome {
            Ok(value: next) => {
            }
            Err(error: problem) => {
              set code = 74_u8;
            }
          }
        }
      }
      Err(error: refused) => {
        let recovered = move refused.rejected;
        set code = 70_u8;
      }
    }
  }
  let status = exit_status(code: code);
  return move status;
}
```

#### The writer's-eye walkthrough

`let input = seq_filled<u8, 4096>(value: 65_u8);` is [SEQ-2], and it is the row
whose absence made the first draft's own `wfgrep` migration unreachable.
`seq_fixed` gives `len = Z`, and under [CNT-3] a zero-length container is
unreadable and unwritable until 4096 elements have been placed one at a time; a
`MutSpan` formed on it has `len(m) = Z` and names no bytes, so `read_at` over a
view could never fill it. `seq_filled` is what `array_new` and `buffer_new` already
mean and weakens nothing.

`let empty = seq_heap<u8>();` publishes `len = Z`, `cap = Z`, `room = Z` and
**allocates nothing**: an empty growable sequence owns no backing. That is L4 at
the constructor.

`grow<'h>(buf: move empty, heap: &uniq 'h heap, additional: ceiling)` is
**[CALL-2]** on `buf` and the single acquisition point of the program. It is also
the statement that shows why [PROV-9] exists: `grow` receives the provider as
`&uniq 'h Heap` and has to lend it onward to `seq_reserve`, and [OWN-6]'s
child-reborrow clause admits only a locally-introduced region, which no result can
outlive. Without the amendment no helper anywhere in the language can thread a
provider, and the whole capability story is writable only in `main`.

On the `Err` arm, `refused.rejected` is the original owner handed back unchanged
(L3, [SEQ-20]); the program moves it out, drops it on the return edge, and exits
with a status. **There is no path on which the process disappears**, which is the
whole of goal B. That drop exhibits `writes` on the heap path [PROV-7], which is
why `main`'s row names `heap` under `writes` and not only under `allocates`, and
why two such drops in overlapped statements would conflict under [PAR-1]. The
first draft gave a heap free an empty row and admitted exactly that race.

On the `Ok` arm, [SEQ-20]'s relations arrive: `cap(ready) = cap(empty) + ceiling`
and `len(ready) = len(empty)`. Note that this is an **equality on the capacity, not
a lower bound**, which is what keeps L15 honest: the descriptor records exactly
what was asked for, an allocator that rounds the request up puts the extra bytes
in the backing where no program can see them, and `room` is therefore a
deterministic readable value rather than an allocator observation.

`let view = seq_append_view<'fill>(vector: &uniq 'fill buf);` publishes
`len(view) = Z` and `cap(view) = room(buf)`. **The view value holds the loan**
[VIEW-2], exclusively, for as long as the value lives. That is the rule the first
draft asserted and did not have: its formation borrow was a call-scoped temporary
and its imported [OWN-5] clause judged view accesses as *shared*, so nothing
stopped a second `AppendView` on `buf`, two `absorb`s summing to a length neither
window had filled, and a discharged [OP-4] on raw slots. Here the second formation
is refused by ordinary exclusivity, at its own statement, citing [OWN-5].

`set total = absorb(view: move done);` ends the loan at the consume, not at the end
of `'fill`. That is the other half of [VIEW-2] and it is what makes the next region
usable: `region 'w { let body = seq_span<'w>(vector: &'w buf); ... }` reads `buf`
immediately after the commit, with no nested region-per-phase dance.

`write_once<'h, 'w>(... source: move body, start: 0_u64, end: total)` is [SYS-8]
over a view. Its two obligations are `ile(0_u64, total)`, implicit, and
`ile(total, len(body))`, which discharges because [VIEW-2] gave `len(body) = len(buf)`
and [VIEW-6] gave `len(buf)`'s image as `Z + total`. This is the statement that
makes goal A's container half real: a heap-free program can do I/O only when the
system operations take views rather than `buffer<u8>`.

#### What the compiler reports

```text
note: scheduler.wf is source-resource-closed; envelope written to scheduler.E
note: collector.wf is not source-resource-closed
  [RES-5] main selects command.heap
    heap-reaching path:  main -> grow -> seq_reserve
  a general store cannot appear in an envelope [L6], so no envelope is computed
  still true of this program:
    no covered-resource failure is a trap [RES-7]; seq_reserve returns a value
    the heap is reachable only through the parameter above [PROV-6]
    every free of heap-owned storage names the heap in a signature [PROV-7]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

Two diagnostics the design owes a writer, each citing a rule that exists in
section 3. The first is what a push without a capacity proof reports:

```text
Semantics/Source [SEQ-5]: UndischargedOperationDomain
  operation: seq_push
  residual:  "Z < room(view)"
  mechanical_fix: state a header invariant over room(view) [INV-1, MSR-5],
    dominate the push with a branch on room(view) [SEQ-4], or use seq_try_push
```

The first draft's version of this message cited `[SEQ-4]`, which is `len`; stated
the residual `len(view) < cap(view)`, which is not the requirement [SEQ-5] carries
and is equivalent to it only under an identity the draft declared not to be a
fact; and named three repairs of which two could not be written and the third did
not reach the goal. All three named here are writable and all three discharge.

The second is the new declaration-time refusal [VIEW-10] adds:

```text
Semantics/Source [VIEW-10]: SameRegionViewResults
  results "data_out" and "bad_out" are both AppendView<'o, u8>
  each therefore aliases every AppendView<'o, u8> parameter of this function
  mechanical_fix: give each result its own formal region
```

A three-output demultiplexer written with one formal region compiles and then
behaves conservatively for a reason nothing in its signature shows. That is the
same defect class as D1 seen from the callee's side, and it belongs at the
declaration.

---

## 5. Open questions

Everything the owner's rulings settle is dropped and not restated: whether the
heap is a capability value (it is); whether a bounded general heap may enter `E`
(it may not); whether recursion may accumulate frames (it may not); whether
`FixedVector` admits affine `T` (it does); whether container state may be mutated
through `&uniq` (it may not); whether an `ensures` may be two-state (it may not);
and the multi-return spelling.

Ten of the first draft's seventeen questions are also gone, because this draft
answers them rather than asking them: the length-class terms and the goal
disposition are [MSR-1] and [MSR-4]; the three-term arithmetic residual is
[MSR-3]'s image transfer; the `absorb` commit is [VIEW-6]; the coverage
certificate died with `Builder`; the arena's reclamation is [RES-6]'s cursor
domain; the optimizer-versus-envelope question is [STK-3]'s "`E` is an output of
code generation"; the profile table is [RES-2]; and the view formation syntax is
[SEQ-0]'s declaration domain. What remains is what this design genuinely does not
decide.

**Q1. May a resource-closed program handle a typed refusal, or must it prove every
acquisition?** *(a)* Strict: every covered acquisition uses the proved spelling.
*(b)* Permissive: both spellings are admitted, since neither can ask for more than
`E`.
**Recommend (b), and L8 is now written to make it real.** The first draft
recommended (b) and then stated L8 in a form under which the checked spelling
changed a loop's summary by nothing, so the recommendation was empty; the split
in L8 is what gives a refusal edge the store's own `room(store) = Z`.

**Q2. What disposes a provider-owned value: a derived release, or a linear
obligation?** *(a)* [PROV-7]'s uniform derived release, exhibiting `writes` on the
provider path and requiring the provider reachable at the release edge. *(b)* A
**linear** `slot<'p, T>` and `HeapVector<T>`, disposed only by an explicit
`pool_release` or `box_free`, so no release is derived at all.
**Recommend (a) as drafted, and record (b) as the cleaner endpoint.** (a) is one
rule with no new ownership class and it derives the lifetime ordering rather than
stating it. (b) removes [STOR-3]'s new row and [EFF-2]'s changed sentence
entirely, at the cost of a linear class the language will probably want anyway; it
should be evaluated on its own before v1, not folded in here.

**Q3. Where does a hosted resource-closed program's memory come from?**
*(a)* Frame and static storage only, as in 4.1. *(b)* One more entry row delivering
a committed region, `command.region as store: own Arena<'store>`.
**Recommend (a).** [FN-7]'s table earns its closedness by containing only inputs
every `command` program can meaningfully receive; a committed extent's size is a
property of one deployment. (b) becomes right the day a program needs a store
larger than its frames can hold.

**Q4. Do region-parametric nominals belong in this version?** [CNT-6] admits a
confined type wherever the owning value's own type names the same region, which
covers every generic instance and no source `struct` field, because a source
nominal declares no region parameter. *(a)* Leave it: a pool-backed buffer cannot
be a field of a task record, and a slab allocator is unwritable. *(b)* Add region
parameters to `struct_decl` and `enum_decl` exactly as `region_params` gives them
to functions; a nominal with region parameters is confined by them, and a field may
be confined to one of them.
**Recommend (b), in the version that lands the kernel slab, and not here.** It is
the correct answer and it is a language feature with its own soundness burden;
naming it as the successor is honest, and folding it into a container design would
be the kind of scope creep that produces a rule nobody checked. The concrete
programs it blocks are exactly the ones a kernel writes: a page cache, a slab
allocator, a buffer cache, a DMA descriptor ring.

**Q5. What relation admits a `par` fill over disjoint ranges of one view?**
[OWN-5] says two accesses through one origin conflict, which is true and is what
makes views sound; a `par` fill, a `seq_split_at`, and divide-and-conquer over a
span all need a *second* relation, disjointness of ranges over one origin.
*(a)* Write that relation once in [VIEW-3], maintained by every rule that forms,
moves, passes, returns and reborrows a view. *(b)* Do not; a `par` fill goes
through [SEQ-2] plus a `MutSpan` plus direct subscript writes, which [PAR-2]
already permits.
**Recommend (b) now and (a) as the successor.** (b) is complete for the shape the
owner named and costs one word of [PAR-2]. (a) is the general answer and it should
be written properly, in one place, rather than approximated by refining one loan,
which is what the deleted `Builder` family tried and failed to do.

**Q6. Should the value-in / value-out spelling get sugar?** Every loop body reads
`set x = f(x: move x, ...)`, naming its target three times. *(a)* A through-form
`view.seq_push(value: byte);` defined as exactly that statement. *(b)* No sugar.
**Recommend (b), and [LIV-3] states it.** (a) is a second call syntax whose
resolution [GRAM-5] does not have, and one spelling per construct is [FORM-1]. The
noise is real and it is the honest price of L10; if a corpus later shows it
dominating, the right change is a spelling for the whole shape, not a receiver
position bolted onto one operation.

**Q7. Are non-constant offsets needed in contract relations?**
`ile(len(written), len(output) + n)` with `n` a parameter is not a difference
bound. `room(P)` removes the common instance and [MSR-3]'s images remove
[VIEW-6]'s sum. *(a)* Admit three-term relations into L0. *(b)* Route them through
the affine domain, which [MSR-4] makes available to every consumer.
**Recommend (b).** L0 is a difference-bound domain by design and its determinism
argument rests on that shape.

**Q8. How do `len(view)` and `len(owner)` relate in the fact domain?**
*(a)* Distinct terms plus the [VIEW-2] and [VIEW-6] equalities. *(b)* One term.
**Recommend (a).** It is the only candidate compatible with [ENT-2], whose term
identity is spelling identity and which declines to model aliasing; its cost is
near zero because the owner is frozen while the view lives.

**Q9. What about control entering the call graph from outside it?**
*(a)* [STK-4] as drafted: a reentrant context denies source-resource-closedness.
*(b)* Admit it, with a separately reserved stack item per reentrant context in `E`.
**Recommend (a) for this version.** (b) is the right long-run answer, and it is
what a real interrupt handler needs; admitting it now means bounding the depth of a
call graph the compiler does not fully see.

**Q10. How does a resource-closed program reach a device?** `main`'s effect row
names only its own labelled inputs and the `command` table is closed, so 4.1 has a
transmit ring and no way to flush it. *(a)* A sixth row on the hosted table.
*(b)* A second program kind under [FN-7]'s existing closed-table discipline, with
its own standard-input table naming memory-mapped regions and interrupt vectors.
**Recommend (b), as a named deferral.** (a) would put a device on every hosted
program's entry. This is the missing piece between "4.1 is a scheduler" and "4.1
is a driver", and it is a batch of its own.

**Q11. `par` and the stack.** *(a)* Restrict resource-closed programs to `par`
shapes whose lowering cannot nest a stolen task on a waiting lane's stack, and
execute every other shape sequentially. *(b)* Require the compiler-managed
work-first continuation redesign before any resource-closed program may contain a
`par`.
**Recommend (a) now and (b) as the real target.** This is the largest engineering
item the design implies, and [RUN-2]'s refusal to admit inline execution as a
saturation answer makes it unavoidable rather than optional: the current runtime's
wait path executes a stolen task on the waiting lane's own stack, and no term of
[STK-3] counts that.

**Q12. Is `E` part of program identity?** *(a)* Diagnostic output only. *(b)* An
emitted machine-readable table beside the object.
**Recommend (b), and explicitly not part of [PROG-2] compilation-unit identity.**
The envelope is useless if the deployment cannot read it, and keeping it out of
unit identity keeps it a derived fact about a build.

---

## 6. Verified versus reasoned

**Verified** means a compiler executed it. The binary is the gate-profile
`whitefootc` built from this tree; every probe below was run against it, either in
the session that wrote this file or in the four falsifier sessions whose verdicts
are quoted with their probe names. No timing figure from any machine appears
anywhere in this file.

### 6.1 What the current compiler does

```text
| probe             | program                                                    | verdict                                    |
|-------------------|------------------------------------------------------------|--------------------------------------------|
| conformance case  | ent5-neg-callee-uniq-buffer-replace-kills-length.wf         | ACCEPTED, exit 0. The xfail is live and D1  |
|                   |                                                            | reproduces at this tip                     |
| d1                | D1's shrink through &uniq buffer<u8>, then line[9_u64]      | ACCEPTED                                   |
| p1                | passthrough(out: own buffer<u8>) -> own buffer<u8>, b[9]    | REJECTED [OP-4], residual "9_u64 < len(b)" |
| p6                | observe['a](handle: &'a buffer<u8>), then line[9_u64]       | ACCEPTED                                   |
| p7                | set view[0_u64] = 1_u8; on a slice_of result                | REJECTED [SET-1], root_class "slice view"  |
| p4                | single-state ensures over len(deref(destination))           | ACCEPTED                                   |
| p2                | ensures ige(len(result), capacity);                         | REJECTED [GRAM-9]                          |
| p8                | fn pair() -> (first: own u64, second: own u64)              | REJECTED [GRAM-2], expected IDENT          |
| p9                | array_new<box<u64>, 4>(move cell)                           | REJECTED [OP-1] InvalidOperation           |
| p10               | set a = take(b: move a); for a : own buffer<u8>             | REJECTED [STOR-1] AffineSetTarget          |
| p11               | let old = replace a = take(b: move a);                      | REJECTED [OWN-1] UseAfterMove              |
| p1_noinput        | a command entry selecting no standard input                 | ACCEPTED                                   |
| p2_forever        | an entry whose only statement is a loop with no break       | REJECTED [FN-1] FunctionFallthrough        |
| p3_rec            | an ordinary self-recursive function called from main        | ACCEPTED                                   |
| p4_undeclared     | a body that allocates while declaring pure                  | REJECTED [EFF-2] EffectMismatch            |
| p5_ambient        | a nullary leaf function that allocates while holding nothing| ACCEPTED                                   |
| p6_unproved       | buffer_new on an unbounded runtime length                   | REJECTED at target layout                  |
| f1c               | two slice_of(&'r line) in one region, both read             | ACCEPTED                                   |
| f1d               | slice's last use, then set line[0] = 3, inside 'r           | REJECTED [OWN-5] BorrowConflict            |
| f2b               | two touch<'r>(handle: &uniq 'r n) calls in one region       | ACCEPTED                                   |
| f3                | let gone = move line; on one arm of an if                   | REJECTED Semantics/Unsupported OwnershipJoin|
| f5                | move of an outer binding inside a loop body                 | REJECTED [OWN-11] MoveOuterBindingInLoop   |
| f6                | &uniq 'r n inside a for body, 'r introduced outside         | REJECTED [OWN-11] BorrowRegionOutsideLoop  |
| f7                | D1's shape through a struct field, then w.buf[9]            | REJECTED [OP-4], residual "9_u64 < len(w.buf)" |
| f2b_tail,         | mutual tail recursion carrying a live &'r borrow of a       | ACCEPTED (both)                            |
| f8_tailframe      | 512-byte caller local across the tail call                  |                                            |
| f3_forever        | command fn main whose only statement is a break-free loop   | REJECTED [FN-1] FunctionFallthrough        |
| f4_par, f6_par2   | two calls / a counted loop over a nullary pure callee       | PAR permitted, eligible                    |
| f5b               | an opaque affine capability in a struct field, borrowed to  | ACCEPTED; both rows silent about it, main  |
|                   | a callee                                                    | declares pure                              |
| f7_regionresult   | -> own Result<arena<'r, u64>, NeedCapacity>                 | REJECTED [FN-2] RegionBearingGenericArgument|
| --stack-ledger    | tests/programs/recursive_tree.wf                            | wf__main_body and main are two disjoint    |
|                   |                                                            | roots; the entry chain reaches             |
|                   |                                                            | wf_resource_abort through drop glue        |
| q6                | bubble sort over &uniq buffer<u64>, one header invariant    | ACCEPTED, exit 0                           |
| q13, q14          | open-addressed table and 3-way demux over parallel buffers  | ACCEPTED                                   |
| q16               | two own buffer parameters, only one returned                | ACCEPTED (a consumed owner need not be     |
|                   |                                                            | returned)                                  |
| q18, q20          | set deref(ring).bytes[at] = byte; through &uniq             | Semantics/Unsupported: RegionsAndBorrows   |
| q21               | reads(ring) where the body reads ring.head                  | REJECTED [EFF-2], expected reads(ring.head)|
| q24               | let n = len(d); if igt(n, 0) { callee requires igt(len, 0) }| ACCEPTED; a len read reaches a goal        |
| tests/programs/   | the migration baseline                                      | COMPILES, exit 0; 11 buffer_new calls      |
| wfgrep.wf         |                                                            |                                            |
```

Eight probes were run against the gate binary in the session that wrote this
draft, to check the claims this draft newly rests on rather than to re-inherit the
falsifiers'.

```text
| probe             | program                                                     | verdict                                    |
|-------------------|-------------------------------------------------------------|--------------------------------------------|
| r1_d1             | D1 verbatim, with the field-precise row EFF-2 demands       | ACCEPTED, exit 0; D1 reproduces at this tip|
| r1_twouniq        | two &uniq 'r n argument borrows of one place in one region  | ACCEPTED; the formation borrow cannot be   |
|                   |                                                             | the freeze [VIEW-2] needs                  |
| r1_own11          | an outer binding moved inside a loop body                    | REJECTED [OWN-11] MoveOuterBindingInLoop   |
| r1_lenatom        | invariant fill_bound: ile(at, len(line));                    | REJECTED [GRAM-4] at parse                 |
| r1_field          | invariant bounded: ile(r.fill, 8_u64);                       | REJECTED [GRAM-4] at parse                 |
| r1_multi          | fn pair() -> (first: own u64, second: own u64)               | REJECTED [GRAM-2], expected IDENT          |
| r1_const          | const CEILING: u64 = 4096_u64;                               | REJECTED [FORM-3], IDENT is lowercase      |
| r1_ambient        | a nullary leaf that allocates while holding nothing          | ACCEPTED                                   |
| r1_relend         | a helper lending its own &uniq onward into the caller's      | REJECTED [OWN-6] InvalidChildReborrow      |
|                   | region                                                       |                                            |
| r1_relend_local   | the same through a local region, copy result                 | ACCEPTED                                   |
| r1_relend_affine  | the same through a local region, affine result                | REJECTED [OWN-6] InvalidChildReborrow      |
```

Two of these changed the design rather than confirming it. `r1_relend` and
`r1_relend_affine` are why [PROV-9] exists: without it no function that receives a
provider can allocate from it, so the whole capability story would have been
writable only in `main` and neither worked program's helper would compile. And
`r1_twouniq` accepted, together with the falsifiers' `f1c` and `f1d`, is the
complete evidence for [VIEW-2]: the argument borrow does not survive its
statement, the origin loan is shared and region-scoped, and therefore the freeze a
view needs is a property of the view value or of nothing.

What each establishes: `d1` and the conformance case make [CALL-6] and [CNT-9] a
repair of a live defect. `p1` shows [CALL-2] already behaves correctly. `p6` and
`q24` show [CALL-1] already holds and that a hoisted `len` chains into a goal,
which is why [SEQ-4]'s readers matter. `p7` shows `MutSpan` is new capability.
`p4`/`p2` bound [CALL-4]. `p8` shows multi-return is new syntax. `p9` shows affine
elements have no construction route today. `p10`/`p11` are the two halves of
[LIV-2]'s premise. `f1c`, `f1d` and `f2b` are the three verdicts that make [VIEW-2]
necessary: two shared views coexist, the origin loan is region-scoped rather than
value-scoped, and an argument borrow is a call-scoped temporary that cannot be the
freeze. `f3`, `f5` and `f6` are the three ways today's language avoids [LIV-1]
rather than answering it. `f7` shows D1 is narrow, so [CALL-6]'s conservative
default is right for everything else. `f2b_tail` and `f8_tailframe` are the
witnesses that refute the first draft's syntactic tail conditions. `f7_regionresult`
is why [CNT-6] exists. `f5b` is why [PROV-4] roots reachability in the leaf's type.
`q18`/`q20`/`q21` are why 4.1 no longer hand-rolls a ring from a struct.
`p5_ambient` is L2's evidence and the single fact the capability half exists to
change.

### 6.2 The proof surface, isolated

```text
| probe                      | shape                                                   | verdict                          |
|----------------------------|---------------------------------------------------------|----------------------------------|
| v23_param_anchored         | counted loop, header invariant, ensures over a parameter | ACCEPTED                         |
| v24_len_anchored           | identical, ensures over len(deref(destination))          | REJECTED [FN-9]                  |
| v25_subscript_consumer     | identical loop, consumer is a subscript                  | ACCEPTED under [OP-4]            |
| v26_ensures_consumer       | identical loop, consumer is an ensures                   | REJECTED [FN-9]                  |
| q2b / q3b                  | one file differing in one token: the body's subscript     | ACCEPTED then REJECTED, in one   |
|                            | proves at [OP-4] and the ensures fails at [FN-9] three    | compilation                      |
|                            | statements later, from the same facts                     |                                  |
| v2_len_atom, q1            | invariant over len(deref(destination))                    | REJECTED [GRAM-4] at parse       |
| q9                         | invariant over a struct field path ile(r.fill, 8_u64)     | REJECTED [GRAM-4] at parse       |
| q5 / q5' / q5''            | one-line invariant header; -wrap on the backedge; exact - | [FORM-2]; [INV-1] Backedge; ACCEPTED |
| v22_loop_then_inv_stmt     | the [INV-1] conclusion proves and still does not reach    | REJECTED [FN-9]                  |
|                            | [FN-9]                                                    |                                  |
```

`q2b`/`q3b` is the pair that makes [MSR-4] a law-level change rather than a repair:
the same proof, asked by two consumers, inside one accepted-then-rejected
compilation. `v2_len_atom`, `q1` and `q9` are the parse-level half, and `q9` is the
one the first draft did not record at all, even though its own kernel program kept
its whole state in two struct fields.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in section 3.** None is implemented, and no compiler has seen any
  of the new types, operations, terms or markers.
- **Every program in section 4** and every diagnostic quoted there. They are
  written against the unchanged v0.40 rules as well as this design's, which the
  first draft's were not; that is a stronger claim and it is still a claim, not a
  verdict.
- **Every byte figure in 4.1's envelope.**
- **The composition algebra of 3.3.1.** Its sequence and branch rules over an
  exit-label map are standard. Its `par` rule depends on a runtime profile that
  does not exist. Its loop rule's claim that a zero-delta backedge needs no
  iteration bound is still the claim to attack first: a depth-first walk restores
  its stack on every backedge and a breadth-first walk does not, and no rule of
  3.3.1 tells them apart from the source.
- **[MSR-3]'s image transfer through [LIV-2].** The mechanism is [ENT-6]'s
  existing whole-binding `set` transfer applied to measure terms; that it composes
  correctly with [ENT-5]'s pre-kill closure at the reinitializing `set` is argued
  and not executed.
- **[STK-1]'s deadness premise.** It refuses the two witnesses the syntactic list
  admitted. Whether it admits every component a correct rewrite could take is not
  proved and the rejected shapes were not enumerated.
- **Everything about the current runtime's closure.** [RUN-2] is written as a
  qualification obligation precisely because no existing target can be certified
  to meet it, and the `--stack-ledger` read above shows the entry chain is
  presently two disjoint roots reaching the abort site.
- **The claim that `wfgrep` becomes heap-free.** Its eleven `buffer_new` calls
  reach three declared rows, all of which [SEQ-2] and [SEQ-17] replace. The
  substitution was not performed and compiled. It also moves roughly 95 KiB out of
  the heap and into frames, which is a [STK-3] question rather than a free win.

### 6.4 Falsifiers this design asks for next

1. Hand-execute 3.3.1 on 4.1 and on a breadth-first walk, and check that the loop
   rule distinguishes them or admit that it does not.
2. Attack [MSR-3] with a measure image carried across a `propagate` edge and a
   `value_if` delivery.
3. Attack [VIEW-2]'s exclusive-view clause with a view formed on `table[i]` and a
   second on `table[j]`, where [OWN-7]'s conservative subscript overlap is the only
   thing standing between them.
4. Attack [LIV-1] with a binding whose liveness agrees at the join and whose
   *type* differs on the two paths.
5. Attack [PROV-7] with a release edge inside a `par` window.
6. Rewrite one existing corpus program against [SEQ-20] and [RES-7] by hand and
   count what the `Result` return costs at every call site.
7. Attack [CNT-6] with a confined value inside a confined generic inside another
   confined generic, and check that the outlives judgment composes.

### 6.5 Falsifier round 1: what each finding hit, and what refuses it now

Every BREAKS, DEFECT and BLOCKING finding of the four reports, one line each, with
the rule that now refuses it. The reports are superseded by this table.

```text
| finding                                                        | disposition                                  |
|----------------------------------------------------------------|----------------------------------------------|
| F1-1 [OWN-11] refuses every value-in/value-out loop             | [LIV-1] replaces the move prohibition        |
| F1-2 reinitializing set makes liveness path-dependent           | [LIV-1] join agreement; release unconditional |
| F1-3 [SEQ] publishes terms the operation killed                 | [SEQ-0] declared relations over entry images |
| F1-4 views have no stated loan strength; two AppendViews        | [VIEW-2] the view value holds its own loan   |
| F1-5 [BLD-3] certifies the index range, not the writes          | [BLD] deleted; [SEQ-2] + MutSpan + [PAR-2]   |
| F1-6 [BLD-2] abandonment release unwritable under L12           | [BLD] deleted                                |
| F1-7 [BLD-4]'s [PAR-2] permission is denied anyway              | [BLD] deleted; Q5 states the real relation   |
| F1-8 a heap free exhibits nothing, so two frees race            | [PROV-7] one release rule for every provider |
| F1-9 the Heap may die before its allocations                    | [PROV-7]'s reachability premise              |
| F1-10 [STOR-5]'s position list omits container elements         | [CNT-6] intensional prohibition              |
| F1-11 len(P) forbids a subscript, so nested containers          | [MSR-1] admits a subscripted place           |
| F1-12/13/14 no FIFO, no exchange, no runtime-chosen target      | L12 states the limit; [CNT-1] FixedRing,     |
|                                                                | [SEQ-11] seq_take_at, [SEQ-13] seq_exchange, |
|                                                                | [VIEW-3] formation on a subscripted place    |
| F1-16 the try rows publish nothing arm-specific                 | [SEQ-0] per-arm declared relations           |
| F2-A1 every checked acquisition is untypeable                   | [CNT-6] confined generic arguments           |
| F2-A2 tail lowering rewrites a live-frame component             | [STK-1] caller-frame deadness                |
| F2-A3 static providers frame out; par over one extent           | [PROV-8] one extent per activation, in frame |
| F2-A4 L8 kills the checked-acquisition escape hatch             | L8 split; [RES-7] publishes the refusal      |
| F2-A5 [RUN-2] licenses inline execution and waiting             | [RUN-2] saturation is unreachable, not       |
|                                                                | answered                                     |
| F2-A6 E's stack item starts at main                             | [STK-3] the whole chain, both directions     |
| F2-A7 a lease outlives a moved provider; PROV-1 vs PROV-3       | [PROV-3] &uniq only; [CNT-7] outlives        |
| F2-A8 [RES-1] and [RES-6] disagree about runtime stores         | [RES-6] every covered store is one domain    |
| F2-A9 a resource-closed program cannot leave main               | [FN-1] a break-free loop has no normal-exit  |
|                                                                | edge                                         |
| F2-A10 confined containers are neither storable nor returnable  | [CNT-7] resolves the returnable half; Q4     |
|                                                                | states the field half with a recommendation  |
| F2-A11 L9's defined overwrite makes the judgment vacuous        | L9's second clause                           |
| F2-A12 live/capacity/remaining are not terms                    | [MSR-1] retires them into len/cap/room       |
| F2-A13 the composition algebra is not a function                | 3.3.1's exit-label map                       |
| F2-A14 [PROV-6] roots reachability in the formal's type         | [PROV-4] roots it in the leaf's type         |
| F2-A15 acceptance depends on target and runtime                 | [RES-3] two stages; L1                       |
| F2-A16 the profile table leaves the promise unquantified        | [RES-2] the table is the promise; [RUN-5]    |
|                                                                | descends it                                  |
| F2-A17 E does not compose across units                          | [RES-10] the summary is in the boundary      |
| F2-A18 [RES-4]'s example entry does not typecheck               | 4.1's entry is pure; Q10 states the device   |
|                                                                | route                                        |
| F3-R1 [OWN-11] unregistered                                     | registered; [LIV-1]                          |
| F3-R2 [GRAM-11]/[TYPE-5]: named vs positional arguments         | [SEQ-0] one declaration domain               |
| F3-R3 len/cap/room cannot appear in a clause                    | [MSR-5] clause operands are terms            |
| F3-R4 [GRAM-2]/[GRAM-4]/[FORM-2] unregistered                   | registered; [CALL-5] states the rendering    |
| F3-R5 the publishes column has no fact source                   | [SEQ-0] and [ENT-3] source S13               |
| F3-R6/R7 [SYS-2] and [TYPE-7] unregistered                      | registered                                   |
| F3-2.2 unchanged rules listed as amended; an EFF-2 exception    | the register is three lists; [PROV-7] is a   |
|                                                                | total rule [META-3]                          |
| F3-4.1 seq_shrink contradicts L3                                | [SEQ-22] returns Result                      |
| F3-4.2 [RES-8]'s [SYS-9] analogy is backwards                   | deleted; [SEQ-0] is the fact source          |
| F3-4.3 reserve<T>() is undefined                                | [RES-8]'s constant K<T> from [OP-9]          |
| F3-4.4 cap(a) + len(v) = cap(v) is not an L0 fact               | [VIEW-2] publishes cap(a) = room(v)          |
| F3-4.12/4.13 the builder rules do not close                     | [BLD] deleted; [VIEW-4] states the ground    |
| F3-4.16 the implementation order is unsatisfiable               | section 7 re-derived                         |
| F3-4.17 B1 silently invalidates patterns.md P16                 | named in the register                        |
| F3-D1..D6 the judgment depends on target, runtime and codegen   | [RES-3] two stages; [STK-3]; [RUN-2]         |
| F3-3 about 25 statements of section 4 are refused               | both programs rewritten                      |
| F4-1 room has no reader and no relation                         | L15 restated; [MSR-2] identity; [SEQ-4]      |
| F4-2 no filled construction, no middle removal                  | [SEQ-2], [SEQ-11] seq_take_at, [SEQ-13]      |
| F4-3 proof routes are granted per consumer family               | [MSR-4]                                      |
| F4-4 [INV-1] atoms are identifiers                              | [MSR-5]                                      |
| F4-5 conditional append has no join-preserving image            | [MSR-3]'s delta-atom join                    |
| F4-7 [VIEW-10] silently aliases same-region results             | [VIEW-10] is a declaration error             |
| F4-8 diagnostics cite wrong or nonexistent rules                | section 4's diagnostics cite section 3       |
| F4-9 both worked programs are untested transcriptions           | both rewritten against unchanged v0.40 rules |
```

Findings the reports rated HOLDS or CLEAN, preserved here and not weakened:
[CALL-1] and [CALL-2] survive every shape attacked, and their grounds are
unchanged. [CNT-9] plus [CALL-6] close D1's class, and the struct-wrapped variant
is already refused today (`f7`). A race in a `par` fill is not expressible.
[CALL-3]'s conclusion follows from the types, and [VIEW-4] now states the ground it
was resting on silently. A `MutSpan` sort, `fir_filter`, [SEQ-20]'s error return
and [VIEW-8]'s abandoned-view release are the four places the design already costs
a writer nothing or less than today. The pool seam of 3.11 is the right resolution
and its reasoning from L6 stands. `array<T, N>` is untouched, and a program that
needs no length carries none.

---

## 7. Implementation order

Twelve batches, re-derived from the rules this draft states rather than the ones
the first draft did. Each names the rules it implements and the test it adds. This
is an ordering, not a design choice; nothing here may be read as trading a rule
away for a cheaper batch, and nothing here is an approval or a schedule.

The ordering has one hard constraint the first draft violated: the operation
inventory of B5 and B6 is written in the syntax B3 introduces, so multi-return and
the reinitializing `set` come before any operation that returns two results.

**B1. Type-derived call transports, and the retirement of container state mutation
through `&uniq`.** Rules: [CALL-1], [CALL-2], [CALL-3], [CALL-6], [CNT-9]. First
because it is the live defect and because it needs none of the new types: today's
`&uniq buffer<T>` keeps its spelling and gets [CALL-6]'s type-derived
classification, `element = false`, which is exactly the sweep's minimal sound
repair. Test: **`ent5-neg-callee-uniq-buffer-replace-kills-length.wf` turns
XPASS**, rejecting at [OP-4] with residual `9_u64 < len(line)`; plus one positive
case pinning [CALL-1]. `docs/patterns.md` P16 is corrected in the same change.

**B2. The proof surface.** Rules: [MSR-1], [MSR-2], [MSR-4], [MSR-5]. Second
because every later batch's contracts and invariants are unwritable without it,
and because it is a specification amendment with no new construct. Tests: a
conformance pair mirroring `v23`/`v24` (both accepted after the amendment), one
mirroring `v25`/`v26` so two consumers of one exported invariant agree, one
mirroring `q9` (a struct field path as an affine atom), and one negative case
pinning that a route granted to no consumer is granted to none.

**B3. Multi-return, the destructuring `let`, and join-checked liveness.** Rules:
[CALL-5], [LIV-1], [LIV-2], [LIV-3]. Third because B5 and B6 are written in this
syntax. Test: probe `p8`'s signature parses and binds; probe `p10`'s program is
accepted and probe `p11`'s repair is unnecessary; probe `f3`'s program is a
[LIV-1] error naming both predecessors instead of `SemanticUnsupported`; and a
loop moving and restoring an outer binding is accepted where probe `f5` is
[OWN-11] today.

**B4. Measure images.** Rules: [MSR-3]. Separated from B2 because it touches
[ENT-6]'s transfer machinery rather than its route lists, and because it needs
[LIV-2] from B3. Test: `render`'s fill loop from 4.1, with its header invariant
over `room`, is accepted, and the same loop with the invariant deleted is rejected
at [SEQ-5] with the residual `Z < room(acc)`.

**B5. Owners, typestate, release, and confinement.** Rules: [CNT-1] to [CNT-10],
[SEQ-0], and the constructor, place, take, exchange and clear rows. Retires
`buffer<T>` from the writer surface. Tests: a `FixedVector<Handle, 64>` object
table with affine elements, filled by [SEQ-2], compacted by [SEQ-11]'s
`seq_take_at` and reordered by [SEQ-13], accepted, where probe `p9` is [OP-1]
today; a `FixedRing` FIFO; and `Result<slot<'p, T>, E>` accepted where
`f7_regionresult` is [FN-2] today. This batch supersedes B1's conformance case,
whose program no longer typechecks; that disposition is conformance evidence and is
recorded in `governance/APPROVALS.md` with the merge.

**B6. Views, loans, and the commit event.** Rules: [VIEW-1] to [VIEW-10], and the
view rows of [SEQ]. Tests: an element write through a `MutSpan` is accepted where
probe `p7` is [SET-1] today; **two `AppendView`s formed on one owner are rejected
at the second formation citing [OWN-5]**, which is the falsifier's uninitialized
read made unrepresentable; an owner is readable immediately after `absorb` with no
enclosing region; an abandoned `AppendView` drops what it appended and publishes
nothing; and a two-result signature with two same-region view results is rejected
at [VIEW-10].

**B7. Providers, and the heap as a value.** Rules: [PROV-1] to [PROV-9], [SEQ-3],
[SEQ-20], [SEQ-22], [RES-7], [RES-8]. Tests: probe `p5_ambient`'s program is
**rejected**; a `main` that omits `command.heap` cannot reach any allocation; a
failed `seq_reserve` returns the vector unchanged; a `HeapVector` outliving its
`Heap` is rejected at the scope exit citing [PROV-7]; and two overlapped statements
that only free are denied [PAR-1] permission.

**B8. System I/O over views.** Rules: the [SYS-8] and [SYS-2] amendments. Test:
`tests/programs/wfgrep.wf` migrated to `seq_filled` and `MutSpan`, compiling with
no `allocates` entry anywhere on its call graph. It is the first program that
demonstrates goal A's container half end to end.

**B9. The stack judgment.** Rules: [STK-1] to [STK-5]. Tests: probes `f2b_tail`
and `f8_tailframe` are **not** rewritten by [STK-1] and are rejected by [STK-2]
under the marker, where the first draft's conditions would have rewritten them;
their borrow-free variants are rewritten and accepted; probe `p3_rec` stays
accepted without the marker; and a `--stack-ledger` run reports one chain per
context rather than disjoint roots.

**B10. The divergent entry.** Rules: the [FN-1] amendment. Small and separable:
test that probe `f3_forever`'s kernel idle loop is accepted, and that a loop with
a reachable `break` still requires a return.

**B11. The envelope and the judgment.** Rules: [RES-1] to [RES-6], [RES-9],
[RES-10], [RUN-1] to [RUN-6]. Tests: section 4.1's program is source-resource-closed
and its `E` table matches a pinned expectation; section 4.2's is reported not
resource-closed with the heap-reaching path rendered; a program whose runtime
demand exceeds every profile row fails **target qualification** citing no language
rule, which is the two-stage split under test.

**B12. `par` and the envelope.** Rules: [RUN-3], [RUN-4], and [PAR-2]'s one
subscript-domain word. Test: a `seq_filled` plus `MutSpan` plus counted subscript
fill receives [PAR-2] permission with no new refinement, and the `par` rule of
3.3.1 composes against a pinned profile row.

Q11 (`par` and the stack) sits across B11 and B12 and is the largest engineering
item any of this implies; under its recommendation (a), B11 ships with `par` shapes
executed sequentially inside resource-closed programs, and B12 makes the fill
usable without changing that.
